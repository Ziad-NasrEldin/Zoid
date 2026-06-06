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

fn resolve_linked_repository_workdir(linked_repository: Option<String>) -> Result<Option<PathBuf>, String> {
    let Some(repository) = linked_repository.map(|value| value.trim().to_string()) else {
        return Ok(None);
    };

    if repository.is_empty() || repository == "Unlinked" {
        return Ok(None);
    }

    let path = PathBuf::from(repository);
    if !path.exists() {
        return Err(format!("Linked repository does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Linked repository must be a directory: {}", path.display()));
    }

    Ok(Some(path))
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
    pub async fn send_hermes_cli_message(
        messages: Vec<HermesCliMessage>,
        linked_repository: Option<String>,
    ) -> Result<HermesCliResponse, String> {
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
        let repository_workdir = resolve_linked_repository_workdir(linked_repository)?;
        let mut command = Command::new(&path);
        if let Some(workdir) = repository_workdir {
            command.current_dir(workdir);
        }
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
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(label: &str) -> PathBuf {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("zoid-{label}-{timestamp}"))
    }

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

    #[test]
    fn linked_repository_workdir_requires_existing_directory() {
        let temp_dir = std::env::temp_dir();
        let resolved = resolve_linked_repository_workdir(Some(temp_dir.to_string_lossy().to_string())).unwrap();
        assert_eq!(resolved, Some(temp_dir));
        assert!(resolve_linked_repository_workdir(Some("/definitely/not/a/zoid/repo".to_string())).is_err());
        assert_eq!(resolve_linked_repository_workdir(Some("Unlinked".to_string())).unwrap(), None);
    }

    #[test]
    fn hermes_cli_message_runs_inside_linked_repository() {
        let root = unique_temp_path("hermes-workdir");
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        let fake_hermes = root.join("hermes");
        fs::write(
            &fake_hermes,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'hermes fake'; exit 0; fi\nprintf 'workdir:%s\\n' \"$PWD\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_hermes, permissions).unwrap();

        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        std::env::set_var("ZOID_HERMES_CLI", &fake_hermes);
        let response = tauri::async_runtime::block_on(commands::send_hermes_cli_message(
            vec![HermesCliMessage { role: "user".to_string(), content: "pwd".to_string() }],
            Some(repo.to_string_lossy().to_string()),
        ))
        .unwrap();
        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let expected_repo = repo.canonicalize().unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(response.content, format!("workdir:{}", expected_repo.display()));
    }
}
