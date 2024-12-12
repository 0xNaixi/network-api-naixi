// Copyright (c) 2024 Nexus. All rights reserved.

mod analytics;
mod config;
mod connection;
mod generated;
mod prover_id_manager;
mod updater;
pub mod utils;
mod websocket;
mod prover_random;

use crate::analytics::track;

use crate::connection::{
    connect_to_orchestrator_with_infinite_retry, connect_to_orchestrator_with_limited_retry,
};
use std::borrow::Cow;

use clap::Parser;
use colored::Colorize;
use futures::{SinkExt, StreamExt};

use generated::pb::ClientProgramProofRequest;
use prost::Message as _;
use serde_json::json;
use std::time::Instant;
// Network connection types for WebSocket communication

// WebSocket protocol types for message handling
use tokio_tungstenite::tungstenite::protocol::{
    frame::coding::CloseCode, // Status codes for connection closure (e.g., 1000 for normal)
    CloseFrame,               // Frame sent when closing connection (includes code and reason)
    Message,                  // Different types of WebSocket messages (Binary, Text, Ping, etc.)
};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use crate::prover_random::{generate_random_proof_request, load_stats};
use crate::utils::updater::AutoUpdaterMode;
use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use chrono::Local;
use nexus_core::prover::nova::{
    key::CanonicalSerialize, types::*,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;

// The interval at which to send updates to the orchestrator
const PROOF_PROGRESS_UPDATE_INTERVAL_IN_SECONDS: u64 = 180; // 3 minutes


#[derive(Parser, Debug)]
struct Args {
    /// Hostname at which Orchestrator can be reached
    #[arg(default_value_t = String::from("beta.orchestrator.nexus.xyz"))]
    hostname: String,

    // 运行标识 用于鉴别不同的用户 比如 1，2，3，方便批量运行
    #[arg(short, long, default_value_t = String::from("1"))]
    run_id: String,

    //run_mode 运行模式 默认 整数 0
    #[arg(short, long, default_value_t = 0u8)]
    run_mode: u8,

    /// Port over which to communicate with Orchestrator
    #[arg(short, long, default_value_t = 443u16)]
    port: u16,

    /// Whether to hang up after the first proof
    #[arg(short, long, default_value_t = false)]
    just_once: bool,

    /// Mode for the auto updater (production/test)
    #[arg(short, long, value_enum, default_value_t = AutoUpdaterMode::Production)]
    updater_mode: AutoUpdaterMode,
}

fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut f = File::open(filename).expect("no file found");
    let metadata = fs::metadata(filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");

    buffer
}

fn generate_firebase_client() -> String {
    // 获取当前日期，格式为 YYYY-MM-DD
    let today = Local::now().format("%Y-%m-%d").to_string();

    // 构建 JSON 数据
    let data = json!({
        "version": 2,
        "heartbeats": [{
            "agent": "fire-core/0.10.2 fire-core-esm2017/0.10.2 fire-js/ fire-js-all-cdn/10.11.1 fire-iid/0.6.6 fire-iid-esm2017/0.6.6 fire-analytics/0.10.2 fire-analytics-esm2017/0.10.2 fire-auth/1.7.2 fire-auth-esm2017/1.7.2 fire-fst/4.6.1 fire-fst-esm2017/4.6.1",
            "dates": [today]
        }]
    });

    // 将 JSON 转换为字符串并进行 base64 编码
    BASE64_URL_SAFE.encode(data.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print the banner at startup
    utils::cli_branding::print_banner();

    println!(
        "\n===== {}...\n",
        "Setting up CLI configuration".bold().underline()
    );

    // Configure the tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let args = Args::parse();
    let ws_addr_string = format!(
        "{}://{}:{}/prove",
        if args.port == 443 { "wss" } else { "ws" },
        args.hostname,
        args.port
    );

    //打印 fake_mode
    println!(
        "\n\t✔ Your current run mode is {}",
        args.run_mode
    );

    let k = 4;
    let prover_id = prover_id_manager::get_or_generate_prover_id_custom(&args.run_id).await?;
    println!(
        "\n\t✔ Your current prover identifier is {}",
        prover_id.bright_cyan()
    );
    println!(
        "\n===== {}...\n",
        "Connecting to Nexus Network Fake".bold().underline()
    );
    track(
        "connect".into(),
        format!("Connecting to {}...", &ws_addr_string),
        &ws_addr_string,
        json!({"prover_id": prover_id}),
        false,
    );

    // Connect to the Orchestrator with exponential backoff
    let mut client = connect_to_orchestrator_with_infinite_retry(&ws_addr_string, &prover_id).await;

    println!(
        "\n{}",
        "Success! Connection complete!\n".green().bold().underline()
    );

    track(
        "register".into(),
        format!("Your current prover identifier is {}.", prover_id),
        &ws_addr_string,
        json!({"ws_addr_string": ws_addr_string, "prover_id": prover_id}),
        false,
    );

    let mut timer_since_last_orchestrator_update = Instant::now();
    println!(
        "\n===== {}...\n",
        "Starting proof generation for programs".bold().underline()
    );
    let stats = load_stats().expect("Failed to load stats");

    loop {
        let program_name = utils::prover::get_program_for_prover(&prover_id);
        println!(
            "\n\t✔ Proving program {} with prover {}",
            program_name.bright_cyan(),
            prover_id.bright_cyan()
        );
        let start_time = Instant::now();
        let mut progress_time = start_time;
        let mut steps_proven = 0;
        let steps_to_prove = 10;
        let start = 0u8;
        let end = 10u8;
        for step in start..end {
            if args.run_mode == 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }else{
                // 随机延迟 0.3 - 0.7
                let delay = rand::random::<u64>() % 400 + 300;
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
            let progress = generate_random_proof_request(&program_name, &stats);
            let total_steps = 196i32;
            let mut queued_steps_proven = progress.steps_proven;
            steps_proven = queued_steps_proven;
            let start = progress.step_to_start;
            let mut queued_proof_duration_millis = progress.proof_duration_millis;
            // 随机延迟 0.3 - 0.7 毫秒
            let delay = rand::random::<u64>() % 400 + 300;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            let progress:ClientProgramProofRequest= match &args.run_mode{
                1 => {
                    // queued_steps_proven 随机 10 到 50 万
                    queued_steps_proven = (rand::random::<u32>() % 500000 + 10000) as i32;
                    steps_proven = queued_steps_proven;
                    ClientProgramProofRequest {
                        steps_in_trace: total_steps,
                        steps_proven: queued_steps_proven,
                        step_to_start: start,
                        program_id: program_name.clone(),
                        client_id_token: None,
                        proof_duration_millis: queued_proof_duration_millis,
                        k,
                        cli_prover_id: Some(prover_id.clone()),
                    }
            }
                _ => {
                    ClientProgramProofRequest {
                        steps_in_trace: total_steps,
                        steps_proven: queued_steps_proven,
                        step_to_start: start,
                        program_id: program_name.clone(),
                        client_id_token: None,
                        proof_duration_millis: queued_proof_duration_millis,
                        k,
                        cli_prover_id: Some(prover_id.clone()),
                    }
                }
            };
            println!(
                "\t✔ Generated proof request for program {} with prover {}",
                program_name.bright_cyan(),
                prover_id.bright_cyan()
            );
            progress_time = Instant::now();
            //If it has been three minutes since the last orchestrator update, send the orchestator the update
            if timer_since_last_orchestrator_update.elapsed().as_secs()
                > PROOF_PROGRESS_UPDATE_INTERVAL_IN_SECONDS
            {
                println!(
                    "\tWill try sending update to orchestrator with interval queued_steps_proven: {}",
                    queued_steps_proven
                );

                // Send ping to the websocket connection and wait for pong
                match client.send(Message::Ping(vec![])).await {
                    //The ping was succesfully sent...
                    Ok(_) => {
                        //...wait for pong response from websocket with timeout...
                        match tokio::time::timeout(std::time::Duration::from_secs(5), client.next())
                            .await
                        {
                            //... and the pong was received
                            Ok(Some(Ok(Message::Pong(_)))) => {
                                // Connection is verified working
                                match client.send(Message::Binary(progress.encode_to_vec())).await {
                                    Ok(_) => {
                                        // println!("\t\tSuccesfully sent progress to orchestrator\n");
                                        //高亮显示成功
                                        println!("{:#?}",progress);
                                        println!(
                                            "\t✔ Successfully sent progress to orchestrator\n"
                                        );
                                        // println!("{:#?}", progress);
                                        // Reset the queued values only after successful send
                                        queued_steps_proven = 0;
                                        queued_proof_duration_millis = 0;
                                        // let user_cycles_proved_request = UserCyclesProvedRequest {
                                        //     client_ids: vec![],
                                        // };
                                    }
                                    Err(_) => {
                                        client = match connect_to_orchestrator_with_limited_retry(
                                            &ws_addr_string,
                                            &prover_id,
                                        )
                                            .await
                                        {
                                            Ok(new_client) => new_client,
                                            Err(_) => {
                                                // Continue using the existing client and try again next update
                                                client
                                            }
                                        };

                                        // Don't reset queued values on failure
                                    }
                                }
                            }
                            //... and the pong was not received
                            _ => {
                                // println!(
                                //     "\t\tNo pong from websockets connection received. Will reconnect to orchestrator..."
                                // );
                                client = match connect_to_orchestrator_with_limited_retry(
                                    &ws_addr_string,
                                    &prover_id,
                                )
                                    .await
                                {
                                    Ok(new_client) => new_client,
                                    Err(_) => {
                                        // Continue using the existing client and try again next update
                                        client
                                    }
                                };
                            }
                        }
                    }
                    //The ping failed to send...
                    Err(_) => {
                        // println!(
                        //     "\t\tPing failed, will attempt to reconnect to orchestrator: {:?}",
                        //     e
                        // );
                        client = match connect_to_orchestrator_with_limited_retry(
                            &ws_addr_string,
                            &prover_id,
                        )
                            .await
                        {
                            Ok(new_client) => new_client,
                            Err(_) => {
                                // Continue using the existing client and try again next update
                                client
                            }
                        };
                    }
                }

                //reset the timer regardless of success (to avoid spam)
                timer_since_last_orchestrator_update = Instant::now()
            }

            if step == end - 1 {
                let total_duration = start_time.elapsed();
                let total_minutes = total_duration.as_secs() as f64 / 60.0;
                let cycles_proved = steps_proven * k;
                let proof_cycles_per_minute = cycles_proved as f64 / total_minutes;
                // Send analytics about the proof event
                track(
                    "proof".into(),
                    "Proof generated".into(),
                    &ws_addr_string,
                    json!({
                        "steps_in_trace": total_steps,
                        "steps_to_prove": steps_to_prove,
                        "steps_proven": steps_proven,
                        "cycles_proven": cycles_proved,
                        "k": k,
                        "proof_duration_sec": total_duration.as_secs(),
                        "proof_duration_millis": total_duration.as_millis(),
                        "proof_cycles_per_minute": proof_cycles_per_minute,
                        "program_name": program_name,
                    }),
                    false,
                );
            }
        }
        // TODO(collinjackson): Consider verifying the proof before sending it
        // proof.verify(&public_params, proof.step_num() as _).expect("error verifying execution")

        if args.just_once {
            break;
        } else {
            println!("\n\nWaiting for a new program to prove...\n");
            //随机延迟1000-3000 毫秒
            let delay = rand::random::<u64>() % 2000 + 1000;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }

    client
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: Cow::Borrowed("Finished proving."),
        }))
        .await
        .map_err(|e| {
            track(
                "close_error".into(),
                "Failed to close WebSocket connection".into(),
                &ws_addr_string,
                json!({
                    "prover_id": &prover_id,
                    "program_name": utils::prover::get_program_for_prover(&prover_id),
                    "error": e.to_string(),
                }),
                true,
            );
            format!("Failed to close WebSocket connection: {}", e)
        })?;
    track(
        "disconnect".into(),
        "Sent proof and closed connection...".into(),
        &ws_addr_string,
        json!({
            "prover_id": prover_id,
            "program_name": utils::prover::get_program_for_prover(&prover_id),
        }),
        true,
    );
    Ok(())
}