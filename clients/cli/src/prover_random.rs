use std::fs;
use rand::Rng;
use crate::generated::pb::ClientProgramProofRequest;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rand::prelude::IteratorRandom;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    count: u64,
    sum: i64,
    max: Option<i32>,
    min: Option<i32>,
    mode: i32,
    mode_count: u32,
    values: HashMap<i32, u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProofStats {
    pub steps_in_trace: Stats,
    pub steps_proven: Stats,
    pub step_to_start: Stats,
    pub proof_duration_millis: Stats,
    pub k: Stats,
}


impl Stats {
    pub fn new() -> Self {
        Stats {
            count: 0,
            sum: 0,
            max: None,
            min: None,
            mode: 0,
            mode_count: 0,
            values: HashMap::new(),
        }
    }

    pub fn update(&mut self, value: i32) {
        self.count += 1;
        self.sum += value as i64;
        self.max = Some(match self.max {
            Some(current_max) => current_max.max(value),
            None => value,
        });

        self.min = Some(match self.min {
            Some(current_min) => current_min.min(value),
            None => value,
        });
        // 更新众数
        let count = self.values.entry(value).or_insert(0);
        *count += 1;
        if *count > self.mode_count {
            self.mode = value;
            self.mode_count = *count;
        }
    }

    fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

pub fn save_stats(stats: &ProofStats) -> Result<(), Box<dyn std::error::Error>> {
    let stats_path = "src/stats.json";
    let stats_json = serde_json::to_string_pretty(stats)?;
    fs::write(stats_path, stats_json + "\n")?;
    Ok(())
}

fn random_value_from_map(map: &HashMap<i32, u32>) -> i32 {
    let mut rng = rand::thread_rng();
    *map.keys().choose(&mut rng).unwrap_or(&0) // 默认值为 0
}

pub fn load_stats() -> Option<ProofStats> {
    let stats_path = "src/stats.json";
    fs::read_to_string(&stats_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

pub fn generate_random_proof_request(program_name: &str, stats: &ProofStats) -> ClientProgramProofRequest {
    let mut rng = rand::thread_rng();
    // let steps_in_trace = rng.gen_range(stats.steps_in_trace.min..=stats.steps_in_trace.max);
    let steps_in_trace = 196i32;
    let steps_proven = rng.gen_range(
        stats.steps_proven.min.unwrap_or(0)..=stats.steps_proven.max.unwrap_or(0)
    );
    let proof_duration_millis = rng.gen_range(
        stats.proof_duration_millis.min.unwrap_or(0)..=stats.proof_duration_millis.max.unwrap_or(0)
    );
    // let step_to_start = random_value_from_map(&stats.step_to_start.values);
    let step_to_start = 0;
    // let k = random_value_from_map(&stats.k.values);
    let k = 4;
    ClientProgramProofRequest {
        steps_in_trace,
        steps_proven,
        step_to_start,
        program_id: program_name.to_string(),
        client_id_token: None,
        proof_duration_millis,
        k,
        cli_prover_id: None,
    }
}