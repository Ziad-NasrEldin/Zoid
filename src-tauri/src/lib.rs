use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const DEFAULT_HERMES_SESSION: &str = "most-recent-hermes-cli-session";
const HERMES_TIMEOUT_SECONDS: u64 = 300;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesCliStatus {
    ok: bool,
    status: String,
    message: String,
    command: Option<String>,
    session: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HermesCliMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesCliResponse {
    content: String,
    session: String,
}

fn hermes_session_name() -> String {
    env::var("ZOID_HERMES_SESSION").unwrap_or_else(|_| DEFAULT_HERMES_SESSION.to_string())
}

fn candidate_hermes_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(explicit) = env::var("ZOID_HERMES_CLI") {
        paths.push(PathBuf::from(explicit));
    }
    paths.push(PathBuf::from("hermes"));
    if let Ok(home) = env::var("HOME") {
        paths.push(PathBuf::from(format!("{home}/.local/bin/hermes")));
        paths.push(PathBuf::from(format!("{home}/.cargo/bin/hermes")));
    }
    paths.push(PathBuf::from("/opt/homebrew/bin/hermes"));
    paths.push(PathBuf::from("/usr/local/bin/hermes"));
    paths
}

fn command_display(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn run_command_with_timeout(command: &mut Command, timeout: Duration) -> Result<(bool, String, String), String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start Hermes CLI: {error}"))?;

    let Some(_status) = child
        .wait_timeout(timeout)
        .map_err(|error| format!("Failed while waiting for Hermes CLI: {error}"))?
    else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("Hermes CLI timed out before returning a response.".to_string());
    };

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to read Hermes CLI output: {error}"))?;

    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
        String::from_utf8_lossy(&output.stderr).trim().to_string(),
    ))
}

fn find_hermes_cli() -> Option<(PathBuf, String)> {
    for path in candidate_hermes_paths() {
        let mut command = Command::new(&path);
        command.arg("--version");
        if let Ok((true, stdout, stderr)) = run_command_with_timeout(&mut command, Duration::from_secs(8)) {
            let version = if stdout.is_empty() { stderr } else { stdout };
            return Some((path, version));
        }
    }
    None
}

fn strip_terminal_noise(output: &str) -> String {
    output
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("Session ID:")
                && !trimmed.starts_with("session_id:")
                && !trimmed.starts_with("Cost:")
                && !trimmed.starts_with("Tokens:")
                && !trimmed.starts_with("Provider:")
                && !trimmed.starts_with("Warning:")
                && !trimmed.starts_with("↻")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn check_hermes_cli() -> Result<HermesCliStatus, String> {
        let session = hermes_session_name();
        match find_hermes_cli() {
            Some((path, version)) => Ok(HermesCliStatus {
                ok: true,
                status: "online".to_string(),
                message: format!("Hermes CLI is available: {version}"),
                command: Some(command_display(&path)),
                session,
            }),
            None => Ok(HermesCliStatus {
                ok: false,
                status: "offline".to_string(),
                message: "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH.".to_string(),
                command: None,
                session,
            }),
        }
    }

    #[tauri::command]
    pub async fn send_hermes_cli_message(messages: Vec<HermesCliMessage>) -> Result<HermesCliResponse, String> {
        let prompt = messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .map(|message| message.content.trim().to_string())
            .filter(|message| !message.is_empty())
            .ok_or_else(|| "Cannot send an empty message to Hermes CLI.".to_string())?;

        let (path, _) = find_hermes_cli().ok_or_else(|| {
            "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH.".to_string()
        })?;
        let session = hermes_session_name();
        let mut command = Command::new(&path);
        command.args([
            "chat",
            "--continue",
            "--quiet",
            "--source",
            "desktop",
            "--query",
            &prompt,
        ]);

        let (success, stdout, stderr) = run_command_with_timeout(
            &mut command,
            Duration::from_secs(HERMES_TIMEOUT_SECONDS),
        )?;

        if !success {
            let error = if stderr.is_empty() { stdout } else { stderr };
            return Err(format!("Hermes CLI returned an error: {error}"));
        }

        let content = strip_terminal_noise(&stdout);
        if content.is_empty() {
            return Err("Hermes CLI returned an empty response.".to_string());
        }

        Ok(HermesCliResponse { content, session })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::check_hermes_cli,
            commands::send_hermes_cli_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zoid 25");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_session_name_is_stable() {
        std::env::remove_var("ZOID_HERMES_SESSION");
        assert_eq!(hermes_session_name(), DEFAULT_HERMES_SESSION);
    }

    #[test]
    fn terminal_noise_is_removed_from_cli_output() {
        let raw = "hello from Hermes\nSession ID: abc\nCost: $0.01";
        assert_eq!(strip_terminal_noise(raw), "hello from Hermes");
    }
}
