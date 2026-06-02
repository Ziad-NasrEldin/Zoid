use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::{
    complete_agent_run, create_agent_run, create_cli_session, create_notification,
    read_agent_profile, read_task_record, transition_agent_run_status, write_safe_log,
    AgentRunCompletionInput, AgentRunCreateInput, AgentRunRecord, AgentRunStatus,
    AgentRunTransitionInput, CliSessionCreateInput, NotificationCreateInput, NotificationSeverity,
    NotificationType, RepoResult, RepositoryError, ReviewState,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct AgentCommandRunRequest {
    pub task_id: String,
    pub profile_id: String,
    pub cwd: String,
    pub argv: Vec<String>,
    pub stdin: Option<String>,
    pub timeout_ms: Option<u64>,
    pub logs_dir: PathBuf,
    pub metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentCommandRunOutcome {
    pub session_id: String,
    pub run: AgentRunRecord,
    pub log_path: PathBuf,
    pub stdout: String,
    pub stderr: String,
}

#[allow(dead_code)]
pub(crate) fn run_agent_command_service(
    connection: &Connection,
    request: AgentCommandRunRequest,
) -> RepoResult<AgentCommandRunOutcome> {
    preflight_agent_command(connection, &request)?;

    let session = create_cli_session(
        connection,
        CliSessionCreateInput {
            task_id: request.task_id.clone(),
            profile_id: request.profile_id.clone(),
            mode: "clean_session".to_string(),
            cwd: request.cwd.clone(),
            status_summary: "Agent command queued".to_string(),
            metadata_json: request.metadata_json.clone(),
        },
    )?;
    let run = create_agent_run(
        connection,
        AgentRunCreateInput {
            task_id: request.task_id.clone(),
            profile_id: request.profile_id.clone(),
            session_id: session.id.clone(),
            cwd: request.cwd.clone(),
            metadata_json: request.metadata_json.clone(),
        },
    )?;

    transition_agent_run_status(
        connection,
        &run.id,
        AgentRunStatus::Running,
        AgentRunTransitionInput {
            output_summary: Some("Process started".to_string()),
            error_summary: None,
            metadata_json: request.metadata_json.clone(),
        },
    )?;

    let started = Instant::now();
    let profile = read_agent_profile(connection, &request.profile_id)?.ok_or_else(|| {
        RepositoryError::NotFound {
            entity: "agent_profiles",
            key: request.profile_id.clone(),
        }
    })?;
    let command = profile.command.clone().unwrap_or_default();
    let output_result = execute_command(
        &command,
        &request.argv,
        &request.cwd,
        request.stdin.as_deref(),
        request.timeout_ms,
    );
    let duration_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;

    let (status, exit_code, stdout, stderr) = match output_result {
        Ok(CommandExecutionResult::Exited(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().map(i64::from);
            let status = if output.status.success() {
                AgentRunStatus::Completed
            } else {
                AgentRunStatus::Failed
            };
            (status, code, stdout, stderr)
        }
        Ok(CommandExecutionResult::KilledAfterTimeout(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            (
                AgentRunStatus::Cancelled,
                output.status.code().map(i64::from),
                stdout,
                stderr,
            )
        }
        Err(error) => (
            AgentRunStatus::Failed,
            None,
            String::new(),
            format!("process failed: {error}"),
        ),
    };

    let log_body = format!(
        "run_id={}\nstatus={}\nexit_code={:?}\nstdout:\n{}\nstderr:\n{}",
        run.id,
        status.as_str(),
        exit_code,
        stdout,
        stderr
    );
    let log_write =
        write_safe_log(connection, &request.logs_dir, &run.id, &log_body).map_err(|error| {
            RepositoryError::Constraint {
                entity: "log_references",
                message: format!("failed to persist redacted run log: {error}"),
            }
        })?;
    let log_reference_id = read_log_reference_id(connection, &run.id, &format!("{}.log", run.id))?;

    let output_summary = summarize_output(&stdout, status);
    let error_summary = if stderr.trim().is_empty() {
        None
    } else {
        Some(summarize_text(&stderr))
    };
    let completed = complete_agent_run(
        connection,
        &run.id,
        AgentRunCompletionInput {
            status,
            duration_ms: duration_ms.max(1),
            exit_code,
            log_reference_id: Some(log_reference_id),
            output_summary,
            error_summary,
            review_state: if status == AgentRunStatus::Completed {
                ReviewState::Required
            } else {
                ReviewState::NotRequired
            },
            metadata_json: request.metadata_json,
        },
    )?;

    create_run_result_notification(connection, &completed)?;

    Ok(AgentCommandRunOutcome {
        session_id: session.id,
        run: completed,
        log_path: log_write.path,
        stdout,
        stderr,
    })
}

fn preflight_agent_command(
    connection: &Connection,
    request: &AgentCommandRunRequest,
) -> RepoResult<()> {
    let task = read_task_record(connection, &request.task_id)?;
    if task.deleted_at.is_some() {
        return Err(RepositoryError::Constraint {
            entity: "tasks",
            message: "cannot run command for deleted task".to_string(),
        });
    }
    let profile = read_agent_profile(connection, &request.profile_id)?.ok_or_else(|| {
        RepositoryError::NotFound {
            entity: "agent_profiles",
            key: request.profile_id.clone(),
        }
    })?;
    let command = profile.command.as_deref().unwrap_or_default().trim();
    if !profile.configured || command.is_empty() {
        return Err(RepositoryError::Constraint {
            entity: "agent_profiles",
            message: "agent profile is unconfigured".to_string(),
        });
    }
    if command.contains(char::is_whitespace) {
        return Err(RepositoryError::Constraint {
            entity: "agent_profiles",
            message: "agent profile command must be an executable path/name; args belong in argv"
                .to_string(),
        });
    }
    let cwd = Path::new(&request.cwd);
    if !cwd.is_dir() {
        return Err(RepositoryError::Constraint {
            entity: "cli_sessions",
            message: format!("working directory does not exist: {}", request.cwd),
        });
    }
    if !command_is_available(command) {
        return Err(RepositoryError::Constraint {
            entity: "agent_profiles",
            message: format!("command not found before launch: {command}"),
        });
    }
    Ok(())
}

enum CommandExecutionResult {
    Exited(std::process::Output),
    KilledAfterTimeout(std::process::Output),
}

fn execute_command(
    command: &str,
    argv: &[String],
    cwd: &str,
    stdin_body: Option<&str>,
    timeout_ms: Option<u64>,
) -> std::io::Result<CommandExecutionResult> {
    let mut child = Command::new(command)
        .args(argv)
        .current_dir(cwd)
        .stdin(if stdin_body.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin_body) = stdin_body {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(stdin_body.as_bytes())?;
        }
    }

    if let Some(timeout_ms) = timeout_ms {
        let timeout = Duration::from_millis(timeout_ms.max(1));
        let started = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                return child.wait_with_output().map(CommandExecutionResult::Exited);
            }
            if started.elapsed() >= timeout {
                child.kill()?;
                return child
                    .wait_with_output()
                    .map(CommandExecutionResult::KilledAfterTimeout);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    child.wait_with_output().map(CommandExecutionResult::Exited)
}

fn command_is_available(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.is_file();
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(command).is_file()))
        .unwrap_or(false)
}

fn read_log_reference_id(
    connection: &Connection,
    log_scope: &str,
    relative_path: &str,
) -> RepoResult<String> {
    connection
        .query_row(
            "select id from log_references where log_scope = ?1 and relative_path = ?2",
            params![log_scope, relative_path],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| crate::map_repository_error("log_references", error))?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "log_references",
            key: format!("{log_scope}/{relative_path}"),
        })
}

fn summarize_output(stdout: &str, status: AgentRunStatus) -> String {
    let trimmed = summarize_text(stdout);
    if trimmed.is_empty() {
        format!("Agent run {}", status.as_str())
    } else {
        trimmed
    }
}

fn summarize_text(value: &str) -> String {
    let normalized = value.lines().next().unwrap_or_default().trim();
    if normalized.len() > 240 {
        normalized[..240].to_string()
    } else {
        normalized.to_string()
    }
}

fn create_run_result_notification(connection: &Connection, run: &AgentRunRecord) -> RepoResult<()> {
    let (notification_type, severity, title, message) = match run.status {
        AgentRunStatus::Completed => (
            NotificationType::Completion,
            NotificationSeverity::Info,
            "Agent run completed",
            run.output_summary.as_deref().unwrap_or("Command completed"),
        ),
        AgentRunStatus::Failed => (
            NotificationType::Failure,
            NotificationSeverity::Error,
            "Agent run failed",
            run.error_summary
                .as_deref()
                .or(run.output_summary.as_deref())
                .unwrap_or("Command failed"),
        ),
        AgentRunStatus::Cancelled => (
            NotificationType::Attention,
            NotificationSeverity::Warning,
            "Agent run cancelled",
            "Command was cancelled",
        ),
        AgentRunStatus::Blocked => (
            NotificationType::Blocker,
            NotificationSeverity::Error,
            "Agent run blocked",
            "Command was blocked before completion",
        ),
        _ => return Ok(()),
    };

    create_notification(
        connection,
        NotificationCreateInput {
            notification_type,
            title: title.to_string(),
            message: message.to_string(),
            severity,
            task_id: Some(run.task_id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: None,
            action_route: Some(format!("/tasks/{}/runs/{}", run.task_id, run.id)),
            metadata_json: "{\"source\":\"agent_execution_service\"}".to_string(),
        },
    )?;
    Ok(())
}
