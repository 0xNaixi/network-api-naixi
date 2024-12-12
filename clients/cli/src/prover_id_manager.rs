use colored::Colorize;
use rand::{thread_rng, Rng, RngCore};
use random_word::Lang;
use std::{fs, path::Path, path::PathBuf};
use fireauth::RequestCustomData;
use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use crate::generate_firebase_client;

lazy_static! {
    static ref DOMAINS: Vec<&'static str> = vec![
		"mymail.buzz",
		"ddmygod.shop",
		"whatyourmail.shop",
		"acio.eu.org",
		"aeri.eu.org",
		"aois.eu.org",
		"aure.eu.org",
		"aver.eu.org",
		"bcio.eu.org",
		"brui.eu.org",
		"bufo.eu.org",
		"bure.eu.org",
		"bver.eu.org",
		"ccio.eu.org",
		"ceri.eu.org",
		"crui.eu.org",
		"cufo.eu.org",
		"dcio.eu.org",
		"deri.eu.org",
		"drui.eu.org",
		"dufo.eu.org",
		"dure.eu.org",
		"ecio.eu.org",
		"erui.eu.org",
		"eufo.eu.org",
		"fcio.eu.org",
		"frui.eu.org",
		"fufo.eu.org",
		"fure.eu.org",
		"gcio.eu.org",
		"geri.eu.org",
		"grui.eu.org",
		"gufo.eu.org",
		"gure.eu.org",
		"hcio.eu.org",
		"hrui.eu.org",
		"hufo.eu.org",
		"hure.eu.org",
		"icio.eu.org",
		"ieri.eu.org",
		"iois.eu.org",
		"irui.eu.org",
		"iufo.eu.org",
		"iure.eu.org",
		"jcio.eu.org",
		"jrui.eu.org",
		"jufo.eu.org",
		"kcio.eu.org",
		"krui.eu.org",
		"kufo.eu.org",
		"kure.eu.org",
		"lrui.eu.org",
		"lufo.eu.org",
		"nure.eu.org",
		"pois.eu.org",
		"rois.eu.org",
		"seri.eu.org",
		"sois.eu.org",
		"tois.eu.org",
		"uois.eu.org",
		"wois.eu.org",
		"yois.eu.org",
		"fffy.eu.org",
		"fffz.eu.org",
		"ffhh.eu.org",
		"ffii.eu.org",
		"ffll.eu.org",
		"ffpp.eu.org",
		"ffqq.eu.org",
		"ffuu.eu.org",
		"ffvv.eu.org",
		"ffww.eu.org",
		"fhhh.eu.org",
		"fiii.eu.org",
		"fjjj.eu.org",
		"fkkk.eu.org",
		"fmmm.eu.org",
		"fnnn.eu.org",
		"fppp.eu.org",
		"fqqq.eu.org",
		"frfr.eu.org",
		"frrr.eu.org",
		"fsss.eu.org",
		"fuuu.eu.org",
		"hddmail.me",
		"fvvv.eu.org",
		"ethereums.sbs",
		"ethereums.cfd",
		"arbitrums.sbs",
		"arbitrums.cfd",
		"fridahook.com",
		"fyyy.eu.org",
		"gaaa.eu.org",
		"gbbb.eu.org",
		"gccc.eu.org",
		"gddd.eu.org",
		"gfff.eu.org",
		"gggc.eu.org",
		"gggd.eu.org",
		"ggge.eu.org",
		"gggf.eu.org",
		"gggh.eu.org",
		"gggj.eu.org",
		"gggk.eu.org",
		"gggl.eu.org",
		"gggn.eu.org",
		"gjjj.eu.org",
		"guozi.eu.org",
		"hgtr.eu.org",
		"oray.eu.org",
		"sdfew.eu.org",
		"sgdh.eu.org",
		"yssw.eu.org",
		"fahoo.sbs",
		"ggmail.cfd",
		"zkscam.cfd",
		"zkscam.cfd",
		"boyemo.cyou",
		"kiriot.cfd",
		"sioio.sbs",
		"openchain.buzz",
		"misakahelper.com",
		"tenderlygame.com",
		"hautefroid.com",
		"odosprotocol.com",
		"trashmailgenerator.us",
		"injectlib.de"
    ];
}


/// Gets an existing prover ID from the filesystem or generates a new one
/// This is the main entry point for getting a prover ID
pub fn get_or_generate_prover_id() -> String {
    let default_prover_id = generate_default_id();

    let home_path = match get_home_directory() {
        Ok(path) => path,
        Err(_) => return default_prover_id,
    };

    let nexus_dir = home_path.join(".nexus");
    let prover_id_path = nexus_dir.join("prover-id");

    // 1. If the .nexus directory doesn't exist, we need to create it
    if !nexus_dir.exists() {
        return handle_first_time_setup(&nexus_dir, &prover_id_path, &default_prover_id);
    }

    // 2. If the .nexus directory exists, we need to read the prover-id file
    match read_existing_prover_id(&prover_id_path) {
        // 2.1 Happy path - we successfully read the prover-id file
        Ok(id) => {
            println!("Successfully read existing prover-id from file: {}", id);
            id
        }
        // 2.2 We couldn't read the prover-id file, so we may need to create a new one
        Err(e) => {
            eprintln!(
                "{}: {}",
                "Warning: Could not read prover-id file"
                    .to_string()
                    .yellow(),
                e
            );
            handle_read_error(e, &prover_id_path, &default_prover_id);
            default_prover_id
        }
    }
}

fn generate_default_id() -> String {
    format!(
        "{}-{}-{}",
        random_word::gen(Lang::En),
        random_word::gen(Lang::En),
        rand::thread_rng().next_u32() % 100,
    )
}

fn get_home_directory() -> Result<PathBuf, &'static str> {
    match home::home_dir() {
        Some(path) if !path.as_os_str().is_empty() => Ok(path),
        _ => {
            println!("Could not determine home directory");
            Err("No home directory found")
        }
    }
}

fn handle_first_time_setup(
    nexus_dir: &Path,
    prover_id_path: &Path,
    default_prover_id: &str,
) -> String {
    println!("Attempting to create .nexus directory");
    if let Err(e) = fs::create_dir(nexus_dir) {
        eprintln!(
            "{}: {}",
            "Warning: Failed to create .nexus directory"
                .to_string()
                .yellow(),
            e
        );
        return default_prover_id.to_string();
    }

    save_prover_id(prover_id_path, default_prover_id);
    default_prover_id.to_string()
}

fn read_existing_prover_id(prover_id_path: &Path) -> Result<String, std::io::Error> {
    let buf = fs::read(prover_id_path)?;
    let id = String::from_utf8(buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        .trim()
        .to_string();

    if id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Prover ID file is empty",
        ));
    }

    Ok(id)
}

fn save_prover_id(path: &Path, id: &str) {
    if let Err(e) = fs::write(path, id) {
        println!("Failed to save prover-id to file: {}", e);
    } else {
        println!("Successfully saved new prover-id to file: {}", id);
    }
}

fn handle_read_error(e: std::io::Error, path: &Path, default_id: &str) {
    match e.kind() {
        std::io::ErrorKind::NotFound => {
            save_prover_id(path, default_id);
        }
        std::io::ErrorKind::PermissionDenied => {
            eprintln!(
                "{}: {}",
                "Error: Permission denied when accessing prover-id file"
                    .to_string()
                    .yellow(),
                e
            );
        }
        std::io::ErrorKind::InvalidData => {
            eprintln!(
                "{}: {}",
                "Error: Prover-id file is corrupted".to_string().yellow(),
                e
            );
        }
        _ => {
            eprintln!(
                "{}: {}",
                "Error: Unexpected IO error when reading prover-id file"
                    .to_string()
                    .yellow(),
                e
            );
        }
    }
}


/// Retrieves or generates a custom prover ID for a given run ID.
///
/// # Arguments
/// * `run_id` - A unique identifier for the current run
///
/// # Returns
/// A Result containing either the prover ID string or an error
pub async fn get_or_generate_prover_id_custom(run_id: &str) -> Result<String, Box<dyn std::error::Error>>{
    // Check if prover_id_1.txt exists in the current folder, where 1 is the run_id.
    let file_path = format!("prover_id_{}.txt", run_id);
    // Try to read existing prover ID from JSON file
    if let Ok(content) = fs::read_to_string(&file_path) {
        // Parse the JSON content and extract local_id
        let user_info: fireauth::api::User = serde_json::from_str(&content)?;
        return Ok(user_info.local_id);
    }
    // If it doesn't exist, generate a default prover_id and save it to file
    let firebase_client = generate_firebase_client();
    // println!("firebase_client: {:?}", firebase_client);
    let request_custom_data = RequestCustomData{
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
        client_data: "CI62yQEIorbJAQipncoBCKTwygEIlKHLAQiGoM0BCPzQzgEIn9LOARiPzs0B".to_string(),
        client_version: "Chrome/JsCore/10.11.1/FirebaseCore-web".to_string(),
        firebase_client,
        firebase_gmpid: "1:279395003658:web:04ee2c524474d683d75ef3".to_string(),
    };
    let auth = fireauth::FireAuth::new(
        "AIzaSyDqxeUH3_ixM-rTPMHAMWrF2jTFslKUf0U".to_string(),
        Some(request_custom_data),
    );

    // 百分之 30 概率 使用 email
    let mut rng = thread_rng();
    let use_email = rng.gen_ratio(3, 10); // 30% 概率
    let (email, password) = if use_email {
        let len = rng.gen_range(6..11);
        let prefix: String = (0..len)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect::<String>()
            .to_lowercase();
        let domain = DOMAINS[rng.gen_range(0..DOMAINS.len())];
        let email = format!("{}@{}", prefix, domain);

        println!(
            "\n\t✔ Generated email: {}",
            email
        );

        (Some(email), Some(prefix)) // 使用前缀作为密码
    } else {
        (None, None)
    };
    let user = auth.sign_up_email(email.as_deref(), password.as_deref(), true).await?;
    let user_info = auth.get_user_info(&user.id_token).await?;
    // println!("user_info: {:?}", user_info);
    // 将两个结构体转换为 JSON 对象

    let mut combined = serde_json::to_value(&user)?;
    let user_info_value = serde_json::to_value(&user_info)?;
    if let (serde_json::Value::Object(ref mut combined_map), serde_json::Value::Object(user_info_map)) = (combined, user_info_value) {
        // 合并字段，user_info 的值会覆盖 user 中的重复字段
        combined_map.extend(user_info_map);
        // 序列化并保存
        let json_string = serde_json::to_string_pretty(&combined_map)?;
        fs::write(file_path, json_string).unwrap();
    }

    Ok(user_info.local_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir; // This is needed to run tests serially, to remove flakiness

    /// Tests the behavior for a first-time user with no existing configuration.
    /// This simulates a new user scenario where:
    /// 1. No .nexus directory exists yet
    /// 2. No prover-id file exists yet
    /// 3. The program needs to create both directory and file
    #[test]
    #[serial]
    fn test_new_user() {
        let temp_path = {
            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().to_path_buf();
            println!("Directory at start: {:?}", path);
            assert!(path.exists(), "Directory should exist during test");

            // Setup - create temporary home directory
            let original_home = env::var("HOME").ok();
            env::set_var("HOME", temp_dir.path());
            std::thread::sleep(std::time::Duration::from_millis(10)); // Give env time to update

            // Verify .nexus directory doesn't exist yet
            let nexus_dir = temp_dir.path().join(".nexus");
            assert!(!nexus_dir.exists(), "Nexus directory should not exist yet");

            // Get prover ID - should create directory and file
            let id1 = get_or_generate_prover_id();
            println!("Generated ID: {}", id1);

            // Verify ID format (word-word-number)
            let parts: Vec<&str> = id1.split('-').collect();
            assert_eq!(parts.len(), 3, "ID should be in format word-word-number");
            assert!(
                parts[2].parse::<u32>().is_ok(),
                "Last part should be a number"
            );

            // Verify directory and file were created
            assert!(
                nexus_dir.exists(),
                "Nexus directory should have been created"
            );
            let id_path = nexus_dir.join("prover-id");
            assert!(id_path.exists(), "Prover ID file should have been created");

            // Verify saved ID matches what we got
            let saved_id =
                fs::read_to_string(&id_path).expect("Should be able to read prover-id file");
            assert_eq!(saved_id, id1, "Saved ID should match generated ID");

            // Get ID again - should return same ID
            let id2 = get_or_generate_prover_id();
            assert_eq!(id2, id1, "Second call should return same ID");

            // Cleanup
            match original_home {
                None => env::remove_var("HOME"),
                Some(home) => env::set_var("HOME", home),
            }
            std::thread::sleep(std::time::Duration::from_millis(10)); // Give env time to update

            path // Return the path for checking later
        }; // temp_dir is dropped here, cleaning up

        // Verify cleanup happened
        println!("Directory after test: {:?}", temp_path);
        assert!(!temp_path.exists(), "Directory should be cleaned up");
    }

    /// Tests that the function can properly read an existing prover ID configuration.
    /// This simulates a scenario where:
    /// 1. User already has a .nexus directory
    /// 2. User already has a prover-id file with valid content
    /// 3. Function should read and use the existing ID without modification
    #[test]
    #[serial]
    fn test_read_existing_prover_id() {
        // Setup - create temporary home directory
        let original_home = env::var("HOME").ok();
        let temp_dir = TempDir::new().unwrap();
        println!("Created temp dir: {:?}", temp_dir.path());
        env::set_var("HOME", temp_dir.path());
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Create pre-existing configuration
        let nexus_dir = temp_dir.path().join(".nexus");
        fs::create_dir(&nexus_dir).expect("Failed to create .nexus directory");

        let pre_existing_id = "happy-prover-42";
        let id_path = nexus_dir.join("prover-id");
        fs::write(&id_path, pre_existing_id).expect("Failed to create prover-id file");

        // Verify our setup worked
        assert!(nexus_dir.exists(), "Setup: .nexus directory should exist");
        assert!(id_path.exists(), "Setup: prover-id file should exist");
        assert_eq!(
            fs::read_to_string(&id_path).expect("Setup: should be able to read file"),
            pre_existing_id,
            "Setup: file should contain our ID"
        );

        // Test that the function reads the existing ID
        let read_id = get_or_generate_prover_id();
        assert_eq!(
            read_id, pre_existing_id,
            "Should return the pre-existing ID"
        );

        // Verify nothing was modified
        assert!(nexus_dir.exists(), "Directory should still exist");
        assert!(id_path.exists(), "File should still exist");
        assert_eq!(
            fs::read_to_string(&id_path).expect("Should still be able to read file"),
            pre_existing_id,
            "File content should be unchanged"
        );

        // Cleanup
        match original_home {
            None => env::remove_var("HOME"),
            Some(home) => env::set_var("HOME", home),
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    /// Tests handling of corrupted prover-id file
    #[test]
    #[serial]
    fn test_corrupted_prover_id_file() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());

        // Create .nexus directory and corrupted prover-id file
        let nexus_dir = temp_dir.path().join(".nexus");
        fs::create_dir(&nexus_dir).unwrap();

        let id_path = nexus_dir.join("prover-id");
        fs::write(&id_path, vec![0xFF, 0xFE, 0xFF]).unwrap(); // Invalid UTF-8

        let id = get_or_generate_prover_id();
        assert!(id.contains('-'), "Should generate new valid ID");
    }

    /// Tests handling of permission denied scenarios
    #[test]
    #[serial]
    #[cfg(unix)] // This test only works on Unix-like systems
    fn test_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());

        // Create .nexus directory with read-only permissions
        let nexus_dir = temp_dir.path().join(".nexus");
        fs::create_dir(&nexus_dir).unwrap();

        let metadata = fs::metadata(&nexus_dir).unwrap();
        let mut perms = metadata.permissions();
        perms.set_mode(0o444); // read-only
        fs::set_permissions(&nexus_dir, perms).unwrap();

        let id = get_or_generate_prover_id();
        assert!(
            id.contains('-'),
            "Should generate new ID when permissions denied"
        );
    }

    /// Tests that IDs are properly formatted
    #[test]
    #[serial]
    fn test_id_format() {
        let id = get_or_generate_prover_id();
        let parts: Vec<&str> = id.split('-').collect();

        assert_eq!(parts.len(), 3, "ID should have three parts");
        assert!(
            parts[0].chars().all(|c| c.is_ascii_alphabetic()),
            "First word should be alphabetic"
        );
        assert!(
            parts[1].chars().all(|c| c.is_ascii_alphabetic()),
            "Second word should be alphabetic"
        );
        assert!(
            parts[2].parse::<u32>().is_ok(),
            "Last part should be a number"
        );
        assert!(
            parts[2].parse::<u32>().unwrap() < 100,
            "Number should be less than 100"
        );
    }

    /// Tests behavior with empty prover-id file
    #[test]
    #[serial]
    fn test_empty_prover_id_file() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());

        let nexus_dir = temp_dir.path().join(".nexus");
        fs::create_dir(&nexus_dir).unwrap();

        let id_path = nexus_dir.join("prover-id");
        fs::write(&id_path, "").unwrap();

        let id = get_or_generate_prover_id();
        assert!(id.contains('-'), "Should generate new ID for empty file");
    }

    /// Tests behavior when home directory is not available
    #[test]
    #[serial]
    fn test_no_home_directory() {
        env::remove_var("HOME");

        let id = get_or_generate_prover_id();
        assert!(
            id.contains('-'),
            "Should generate temporary ID without home dir"
        );
    }

    /// Tests that generated IDs are unique
    #[test]
    fn test_id_uniqueness() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..100 {
            let id = generate_default_id();
            assert!(ids.insert(id), "Generated IDs should be unique");
        }
    }
}
