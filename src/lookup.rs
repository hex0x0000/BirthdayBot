// Copyright: https://git.bounceme.net/hex0x0000/BirthdayBot/src/branch/master/LICENSE
use anyhow::Context;
use anyhow::*;
use lazy_static::*;
use std::env;
use std::process::Stdio;
use tokio::fs::{create_dir, File};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

lazy_static! {
    static ref FILES: Vec<(&'static str, &'static [u8])> = vec![
        ("server.py", include_bytes!("../username_lookup/server.py")),
        ("client.py", include_bytes!("../username_lookup/client.py")),
        (
            "config.json",
            include_bytes!("../username_lookup/config.json")
        ),
        (
            "requirements.txt",
            include_bytes!("../username_lookup/requirements.txt")
        ),
        ("client.sh", include_bytes!("../username_lookup/client.sh")),
        (
            "initialize.sh",
            include_bytes!("../username_lookup/initialize.sh")
        ),
        (
            "start_server.sh",
            include_bytes!("../username_lookup/start_server.sh")
        ),
    ];
}

pub async fn initialize() -> Result<Child> {
    Command::new("python3")
        .arg("--version")
        .spawn()
        .context("Failed to check python3 presence")?
        .wait()
        .await
        .context("python3 not found: failed to run")?;
    log::info!("python3 found");
    Command::new("virtualenv")
        .arg("--version")
        .spawn()
        .context("Failed to check virtualenv presence")?
        .wait()
        .await
        .context("virtualenv not found: failed to run")?;
    log::info!("virtualenv found");

    let mut exec_path = env::current_exe().context("Failed to get current executable path.")?;
    exec_path.pop();
    env::set_current_dir(&exec_path).context("Failed to change path")?;
    let username_lookup_path = {
        let mut exec_path = exec_path.clone();
        exec_path.push("username_lookup");
        exec_path
    };
    if !username_lookup_path.exists() {
        log::info!("First initialization detected");
        log::info!("Starting file copying...");
        create_dir(&username_lookup_path)
            .await
            .context("Failed to create username_lookup directory")?;
        for (filename, file_content) in FILES.iter() {
            let mut file_path = username_lookup_path.clone();
            file_path.push(filename);
            let mut file = File::create(file_path)
                .await
                .context(format!("Failed to create {}", filename))?;
            file.write_all(file_content)
                .await
                .context(format!("Failed to write to {}", filename))?;
            log::info!("username_lookup/{} successfully written", filename);
        }
        log::info!("File writing finished. Starting initialization...");
        if !Command::new("bash")
            .arg("username_lookup/initialize.sh")
            .spawn()
            .context("Failed to start initialization")?
            .wait()
            .await
            .context("Failed to start initialization")?
            .success()
        {
            return Err(anyhow!("Initialization failed"));
        }
        log::info!("Done! Now you'll have to login with your telegram account as this is the first time activating the lookup script.")
    }

    // Starting server
    Ok(Command::new("bash")
        .arg("username_lookup/start_server.sh")
        .spawn()
        .context("Failed to start script")?)
}

const ALLOWED_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789_@";

pub fn is_username_valid(username: &str) -> bool {
    let username = username.to_ascii_lowercase();
    let len = username.len();
    if len < 5 || len > 32 {
        return false;
    }
    for c in username.chars() {
        if !ALLOWED_CHARS.contains(c) {
            return false;
        }
    }
    true
}

pub async fn get_id(username: &str) -> Result<f64> {
    let output = Command::new("bash")
        .args(&["username_lookup/client.sh", username])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start lookup client")?
        .wait_with_output()
        .await
        .context("Client execution failed")?;
    if !output.status.success() {
        Err(anyhow!("Lookup execution failed"))
    } else {
        Ok(String::from_utf8(output.stdout)
            .context("Invalid UTF-8 output")?
            .replace("\n", "")
            .parse()
            .context("Couldn't convert user_id to f64")?)
    }
}
