use super::*;
use std::path::Component;

fn temp_home(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zoid-{name}-{}", now_millis()))
}

fn count_rows(connection: &Connection, sql: &str) -> i64 {
    connection
        .query_row(sql, [], |row| row.get(0))
        .expect("count rows")
}

fn valid_event_create_request() -> EventCreateRequest {
    EventCreateRequest {
        action_type: "create_local_task".to_string(),
        outcome: "succeeded".to_string(),
        actor_type: "system".to_string(),
        actor_id: Some("tauri_bridge_test".to_string()),
        workspace_key: Some("tasks".to_string()),
        summary: "Created task".to_string(),
        source: "tauri_bridge_test".to_string(),
        metadata_json: "{}".to_string(),
        targets: vec![EventTargetRequest {
            entity_type: "task".to_string(),
            entity_id: "task-1".to_string(),
            relation_type: "primary".to_string(),
        }],
    }
}

fn parse_generate_handler_command_names(source: &str) -> Vec<&str> {
    let block_start = source
        .find(".invoke_handler(tauri::generate_handler![")
        .expect("generate_handler block must exist");
    let block = &source[block_start..];
    let block_end = block.find("])").expect("generate_handler block must end");
    block[..block_end]
        .lines()
        .skip(1)
        .map(|line| line.trim().trim_end_matches(','))
        .filter(|line| !line.is_empty())
        .collect()
}

fn migrated_in_memory_connection() -> Connection {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    connection
}

fn migrated_file_connection(name: &str) -> (Connection, PathBuf) {
    let dir = temp_home(name);
    std::fs::create_dir_all(&dir).expect("create sqlite temp dir");
    let path = dir.join("foundation.sqlite");
    let connection = open_foundation_database(&path).expect("open file-backed sqlite");
    run_migrations(&connection).expect("run migrations");
    (connection, path)
}

fn p204_task(connection: &Connection, title: &str) -> TaskRecord {
    create_task_record(connection, TaskCreateInput::new(title, None)).expect("create task")
}

fn p204_profile(connection: &Connection, configured: bool) -> AgentProfileRecord {
    upsert_agent_profile(
        connection,
        AgentProfileInput {
            id: "profile-hermes".to_string(),
            label: "Hermes CLI".to_string(),
            configured,
            command: if configured {
                Some("hermes".to_string())
            } else {
                None
            },
            config_json: "{\"model\":\"default\"}".to_string(),
            capabilities_json: "{\"local_cli\":true}".to_string(),
            credential_ref: Some("keychain://zoid/hermes".to_string()),
            env_refs_json: "[\"HERMES_API_KEY\"]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("upsert profile")
}

fn p204_session(
    connection: &Connection,
    task: &TaskRecord,
    profile: &AgentProfileRecord,
) -> CliSessionRecord {
    create_cli_session(
        connection,
        CliSessionCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            mode: "clean_session".to_string(),
            cwd: "/tmp".to_string(),
            status_summary: "Queued".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create session")
}

fn p204_log_reference(connection: &Connection) -> String {
    connection
            .execute(
                "insert into log_references (id, log_scope, relative_path, redaction_count, byte_count, metadata_json) values ('logref-p204', 'agent_run', 'agent-run-p204.log', 1, 42, '{}')",
                [],
            )
            .expect("insert log reference");
    "logref-p204".to_string()
}

#[test]
fn p204_schema_version_six_has_agent_profile_session_run_tables() {
    let connection = migrated_in_memory_connection();
    assert!(
        get_migration_version(&connection).expect("migration version") >= 6,
        "P2.04 schema must remain present after later migrations"
    );
    assert_table_has_columns(
        &connection,
        "agent_profiles",
        &[
            "id",
            "label",
            "configured",
            "command",
            "config_json",
            "capabilities_json",
            "credential_ref",
            "env_refs_json",
            "metadata_json",
        ],
    );
    assert_table_has_columns(
        &connection,
        "cli_sessions",
        &[
            "id",
            "task_id",
            "profile_id",
            "mode",
            "cwd",
            "status",
            "status_summary",
            "metadata_json",
        ],
    );
    assert_table_has_columns(
        &connection,
        "agent_runs",
        &[
            "id",
            "task_id",
            "profile_id",
            "session_id",
            "cwd",
            "command_snapshot",
            "profile_snapshot_json",
            "status",
            "started_at",
            "completed_at",
            "duration_ms",
            "exit_code",
            "log_reference_id",
            "output_summary",
            "error_summary",
            "review_state",
            "metadata_json",
        ],
    );

    let task = p204_task(&connection, "FK restrict task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id,
            profile_id: profile.id,
            session_id: session.id.clone(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run for session FK restrict");
    let delete_session = connection.execute(
        "delete from cli_sessions where id = ?1",
        params![session.id],
    );
    assert!(
        delete_session.is_err(),
        "mandatory session FK must restrict deleting a referenced session"
    );
}

#[test]
fn p204_agent_profile_rejects_secret_like_command_before_persistence() {
    let connection = migrated_in_memory_connection();
    let rejected = upsert_agent_profile(
        &connection,
        AgentProfileInput {
            id: "profile-secret-command".to_string(),
            label: "Secret Command".to_string(),
            configured: true,
            command: Some("hermes --api-key sk-raw-secret-token".to_string()),
            config_json: "{}".to_string(),
            capabilities_json: "{}".to_string(),
            credential_ref: Some("keychain://zoid/hermes".to_string()),
            env_refs_json: "[\"HERMES_API_KEY\"]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("secret-like command must be rejected");
    assert!(matches!(
        rejected,
        RepositoryError::SecretRejected {
            field: "command",
            ..
        }
    ));
    assert!(
        read_agent_profile(&connection, "profile-secret-command")
            .expect("read profile")
            .is_none(),
        "rejected command profile must not persist"
    );
}

#[test]
fn p204_run_creation_rejects_missing_deleted_task_and_unconfigured_profile() {
    let connection = migrated_in_memory_connection();
    let configured = p204_profile(&connection, true);
    let missing = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: "missing-task".to_string(),
            profile_id: configured.id.clone(),
            session_id: "missing-session".to_string(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("missing task rejected");
    assert!(matches!(
        missing,
        RepositoryError::NotFound {
            entity: "tasks",
            ..
        }
    ));

    let deleted = p204_task(&connection, "Deleted run target");
    soft_delete_task(&connection, &deleted.id).expect("delete task");
    let deleted_error = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: deleted.id,
            profile_id: configured.id.clone(),
            session_id: "missing-session".to_string(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("deleted task rejected");
    assert!(matches!(
        deleted_error,
        RepositoryError::Constraint {
            entity: "agent_runs",
            ..
        }
    ));

    let task = p204_task(&connection, "Unconfigured profile target");
    let unconfigured = p204_profile(&connection, false);
    let blocked = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: unconfigured.id,
            session_id: "missing-session".to_string(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("unconfigured profile rejected before fake success");
    assert!(matches!(
        blocked,
        RepositoryError::Constraint {
            entity: "agent_profiles",
            ..
        }
    ));

    let configured_again = p204_profile(&connection, true);
    let no_session = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id,
            profile_id: configured_again.id,
            session_id: "missing-session".to_string(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("run must reference an existing session");
    assert!(matches!(
        no_session,
        RepositoryError::NotFound {
            entity: "cli_sessions",
            ..
        }
    ));
}

#[test]
fn p204_session_run_link_to_task_and_lifecycle_events_omit_raw_logs() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Run linked task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);

    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            session_id: session.id.clone(),
            cwd: "/tmp".to_string(),
            metadata_json: "{\"safe\":true}".to_string(),
        },
    )
    .expect("create run");

    assert_eq!(session.task_id, task.id);
    assert_eq!(run.task_id, task.id);
    assert_eq!(run.session_id, session.id);
    assert_eq!(run.profile_id, profile.id);
    assert_eq!(run.status, AgentRunStatus::Queued);
    assert_eq!(run.command_snapshot, "hermes");

    let link_count = count_rows(
            &connection,
            "select count(*) from entity_links where source_type = 'task' and target_type in ('cli_session', 'agent_run')",
        );
    assert_eq!(link_count, 2);

    transition_agent_run_status(
        &connection,
        &run.id,
        AgentRunStatus::Running,
        AgentRunTransitionInput {
            output_summary: Some("Started without raw stdout".to_string()),
            error_summary: None,
            metadata_json: "{\"log_path\":\"agent-run-p204.log\"}".to_string(),
        },
    )
    .expect("transition running");

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: None,
            outcome: Some("succeeded"),
            source: Some("agent_run_repository"),
            limit: 10,
        },
    )
    .expect("list events");
    assert!(events.iter().any(|event| event.action_type == "run.queued"));
    assert!(events
        .iter()
        .any(|event| event.action_type == "run.started"));
    let raw_log = "RAW_STDOUT_SECRET_SHOULD_NOT_BE_IN_SQLITE";
    assert!(events
        .iter()
        .all(|event| !event.summary.contains(raw_log) && !event.metadata_json.contains(raw_log)));
}

#[test]
fn p204_terminal_transitions_are_immutable_and_completion_stores_evidence() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Completing run task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id,
            profile_id: profile.id,
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run");
    let log_reference_id = p204_log_reference(&connection);

    let missing_exit_code = complete_agent_run(
        &connection,
        &run.id,
        AgentRunCompletionInput {
            status: AgentRunStatus::Completed,
            duration_ms: 1_234,
            exit_code: None,
            log_reference_id: Some(log_reference_id.clone()),
            output_summary: "Completed with summarized output".to_string(),
            error_summary: None,
            review_state: ReviewState::NotRequired,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("completed run requires exit code");
    assert!(matches!(
        missing_exit_code,
        RepositoryError::Constraint {
            entity: "agent_runs",
            ..
        }
    ));

    let missing_log_reference = complete_agent_run(
        &connection,
        &run.id,
        AgentRunCompletionInput {
            status: AgentRunStatus::Completed,
            duration_ms: 1_234,
            exit_code: Some(0),
            log_reference_id: None,
            output_summary: "Completed with summarized output".to_string(),
            error_summary: None,
            review_state: ReviewState::NotRequired,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("completed run requires log reference");
    assert!(matches!(
        missing_log_reference,
        RepositoryError::Constraint {
            entity: "agent_runs",
            ..
        }
    ));

    let completed = complete_agent_run(
        &connection,
        &run.id,
        AgentRunCompletionInput {
            status: AgentRunStatus::Completed,
            duration_ms: 1_234,
            exit_code: Some(0),
            log_reference_id: Some(log_reference_id.clone()),
            output_summary: "Completed with summarized output".to_string(),
            error_summary: None,
            review_state: ReviewState::NotRequired,
            metadata_json: "{\"log_reference_path\":\"agent-run-p204.log\"}".to_string(),
        },
    )
    .expect("complete run");

    assert_eq!(completed.status, AgentRunStatus::Completed);
    assert_eq!(completed.duration_ms, Some(1_234));
    assert_eq!(completed.exit_code, Some(0));
    assert_eq!(
        completed.log_reference_id.as_deref(),
        Some(log_reference_id.as_str())
    );
    assert_eq!(
        completed.output_summary.as_deref(),
        Some("Completed with summarized output")
    );
    assert!(!completed.metadata_json.contains("raw stdout"));

    let same_terminal_update = complete_agent_run(
        &connection,
        &run.id,
        AgentRunCompletionInput {
            status: AgentRunStatus::Completed,
            duration_ms: 2_000,
            exit_code: Some(0),
            log_reference_id: Some(log_reference_id.clone()),
            output_summary: "Should not rewrite terminal evidence".to_string(),
            error_summary: None,
            review_state: ReviewState::NotRequired,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("terminal run cannot be mutated even to same status");
    assert!(matches!(
        same_terminal_update,
        RepositoryError::Constraint {
            entity: "agent_runs",
            ..
        }
    ));

    let illegal = transition_agent_run_status(
        &connection,
        &run.id,
        AgentRunStatus::Running,
        AgentRunTransitionInput {
            output_summary: Some("retry should be a new run".to_string()),
            error_summary: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("terminal run cannot mutate into new work");
    assert!(matches!(
        illegal,
        RepositoryError::Constraint {
            entity: "agent_runs",
            ..
        }
    ));
}

#[test]
fn p204_failed_cancelled_blocked_and_waiting_for_input_are_distinct() {
    let connection = migrated_in_memory_connection();
    let profile = p204_profile(&connection, true);
    let mut observed = Vec::new();
    for status in [
        AgentRunStatus::Failed,
        AgentRunStatus::Cancelled,
        AgentRunStatus::Blocked,
        AgentRunStatus::WaitingForInput,
    ] {
        let task = p204_task(&connection, status.as_str());
        let session = p204_session(&connection, &task, &profile);
        let run = create_agent_run(
            &connection,
            AgentRunCreateInput {
                task_id: task.id,
                profile_id: profile.id.clone(),
                session_id: session.id,
                cwd: "/tmp".to_string(),
                metadata_json: "{}".to_string(),
            },
        )
        .expect("create run");
        let updated = transition_agent_run_status(
            &connection,
            &run.id,
            status,
            AgentRunTransitionInput {
                output_summary: Some(format!("status {} summary", status.as_str())),
                error_summary: None,
                metadata_json: "{}".to_string(),
            },
        )
        .expect("transition status");
        observed.push(updated.status.as_str().to_string());
    }
    assert_eq!(
        observed,
        vec!["failed", "cancelled", "blocked", "waiting_for_input"]
    );
}

fn p209_profile_with_command(
    connection: &Connection,
    id: &str,
    command: &str,
    configured: bool,
) -> AgentProfileRecord {
    upsert_agent_profile(
        connection,
        AgentProfileInput {
            id: id.to_string(),
            label: format!("Profile {id}"),
            configured,
            command: if command.is_empty() {
                None
            } else {
                Some(command.to_string())
            },
            config_json: "{\"mode\":\"local_command\"}".to_string(),
            capabilities_json: "{\"local_cli\":true,\"safe_command\":true}".to_string(),
            credential_ref: None,
            env_refs_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("upsert p209 profile")
}

#[test]
fn p209_preflight_blocks_unconfigured_missing_command_and_bad_cwd_before_run_records() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Preflight command task");
    let logs_dir = temp_home("p209-preflight-logs");
    let unconfigured =
        p209_profile_with_command(&connection, "profile-unconfigured-p209", "", false);

    let blocked = run_agent_command_service(
        &connection,
        AgentCommandRunRequest {
            task_id: task.id.clone(),
            profile_id: unconfigured.id,
            cwd: "/tmp".to_string(),
            argv: vec![],
            stdin: None,
            timeout_ms: None,
            logs_dir: logs_dir.clone(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("unconfigured profile must block before fake success");
    assert!(matches!(
        blocked,
        RepositoryError::Constraint {
            entity: "agent_profiles",
            ..
        }
    ));
    assert_eq!(
        count_rows(&connection, "select count(*) from cli_sessions"),
        0
    );
    assert_eq!(
        count_rows(&connection, "select count(*) from agent_runs"),
        0
    );

    let missing_command = p209_profile_with_command(
        &connection,
        "profile-missing-command-p209",
        "/definitely/not/a/zoid-command",
        true,
    );
    let command_error = run_agent_command_service(
        &connection,
        AgentCommandRunRequest {
            task_id: task.id.clone(),
            profile_id: missing_command.id,
            cwd: "/tmp".to_string(),
            argv: vec![],
            stdin: None,
            timeout_ms: None,
            logs_dir: logs_dir.clone(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("missing command must block before launch");
    assert!(matches!(
        command_error,
        RepositoryError::Constraint {
            entity: "agent_profiles",
            ..
        }
    ));
    assert_eq!(
        count_rows(&connection, "select count(*) from cli_sessions"),
        0
    );
    assert_eq!(
        count_rows(&connection, "select count(*) from agent_runs"),
        0
    );

    let shell = p209_profile_with_command(&connection, "profile-shell-p209", "/bin/sh", true);
    let bad_cwd = run_agent_command_service(
        &connection,
        AgentCommandRunRequest {
            task_id: task.id,
            profile_id: shell.id,
            cwd: "/definitely/missing/zoid/cwd".to_string(),
            argv: vec!["-c".to_string(), "printf nope".to_string()],
            stdin: None,
            timeout_ms: None,
            logs_dir,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("missing cwd must block before launch");
    assert!(matches!(
        bad_cwd,
        RepositoryError::Constraint {
            entity: "cli_sessions",
            ..
        }
    ));
    assert_eq!(
        count_rows(&connection, "select count(*) from cli_sessions"),
        0
    );
    assert_eq!(
        count_rows(&connection, "select count(*) from agent_runs"),
        0
    );
}

#[test]
fn p210_p212_p213_runner_captures_output_persists_redacted_log_and_writes_events() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Run safe local command");
    let profile =
        p209_profile_with_command(&connection, "profile-shell-success-p210", "/bin/sh", true);
    let logs_dir = temp_home("p210-logs");
    let secret = "sk-live-should-redact-12345678901234567890";

    let outcome = run_agent_command_service(
        &connection,
        AgentCommandRunRequest {
            task_id: task.id.clone(),
            profile_id: profile.id,
            cwd: "/tmp".to_string(),
            argv: vec![
                "-c".to_string(),
                format!("printf 'clean stdout'; printf 'stderr {secret}' >&2"),
            ],
            stdin: None,
            timeout_ms: None,
            logs_dir: logs_dir.clone(),
            metadata_json: "{\"source\":\"test\"}".to_string(),
        },
    )
    .expect("run safe command");

    assert_eq!(outcome.run.status, AgentRunStatus::Completed);
    assert_eq!(outcome.run.exit_code, Some(0));
    assert!(outcome.run.duration_ms.unwrap_or_default() >= 1);
    assert!(outcome.stdout.contains("clean stdout"));
    assert!(outcome.stderr.contains(secret));
    assert!(outcome.run.log_reference_id.is_some());
    assert_eq!(outcome.run.review_state, ReviewState::Required);

    let persisted_log =
        std::fs::read_to_string(&outcome.log_path).expect("read safe persisted log");
    assert!(persisted_log.contains("clean stdout"));
    assert!(
        !persisted_log.contains(secret),
        "raw secret must not be persisted in log file"
    );
    assert!(persisted_log.contains("[REDACTED]"));

    let sqlite_secret_count = count_rows(
        &connection,
        "select count(*) from agent_runs where output_summary like '%sk-live%' or error_summary like '%sk-live%' or metadata_json like '%sk-live%'",
    );
    assert_eq!(
        sqlite_secret_count, 0,
        "SQLite run metadata/summaries must not store raw secret material"
    );

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: None,
            outcome: Some("succeeded"),
            source: Some("agent_run_repository"),
            limit: 20,
        },
    )
    .expect("list lifecycle events");
    assert!(events.iter().any(|event| event.action_type == "run.queued"));
    assert!(events
        .iter()
        .any(|event| event.action_type == "run.started"));
    assert!(events
        .iter()
        .any(|event| event.action_type == "run.completed"));
    assert!(events
        .iter()
        .all(|event| !event.summary.contains(secret) && !event.metadata_json.contains(secret)));

    let notification_count = count_rows(
        &connection,
        "select count(*) from notifications where task_id is not null and run_id is not null and notification_type = 'completion'",
    );
    assert_eq!(notification_count, 1);
}

#[test]
fn p210_timeout_kills_process_and_records_cancelled_cleanup() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Timed out command");
    let profile =
        p209_profile_with_command(&connection, "profile-shell-timeout-p210", "/bin/sh", true);
    let logs_dir = temp_home("p210-timeout-logs");

    let outcome = run_agent_command_service(
        &connection,
        AgentCommandRunRequest {
            task_id: task.id,
            profile_id: profile.id,
            cwd: "/tmp".to_string(),
            argv: vec![
                "-c".to_string(),
                "sleep 2; printf should-not-finish".to_string(),
            ],
            stdin: None,
            timeout_ms: Some(50),
            logs_dir,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("timeout should kill child and record cancellation");

    assert_eq!(outcome.run.status, AgentRunStatus::Cancelled);
    assert!(outcome.run.log_reference_id.is_some());
    assert!(outcome.log_path.is_file());
    assert!(!outcome.stdout.contains("should-not-finish"));

    let cancelled_events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: Some("run.cancelled"),
            outcome: Some("succeeded"),
            source: Some("agent_run_repository"),
            limit: 10,
        },
    )
    .expect("list cancelled events");
    assert_eq!(cancelled_events.len(), 1);
    let notification_count = count_rows(
        &connection,
        "select count(*) from notifications where run_id is not null and notification_type = 'attention' and title = 'Agent run cancelled'",
    );
    assert_eq!(notification_count, 1);
}

#[test]
fn p211_failed_command_is_recorded_as_failed_with_exit_code_log_and_failure_notification() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Failing local command");
    let profile =
        p209_profile_with_command(&connection, "profile-shell-fail-p211", "/bin/sh", true);
    let logs_dir = temp_home("p211-logs");

    let outcome = run_agent_command_service(
        &connection,
        AgentCommandRunRequest {
            task_id: task.id,
            profile_id: profile.id,
            cwd: "/tmp".to_string(),
            argv: vec![
                "-c".to_string(),
                "printf 'bad stderr' >&2; exit 7".to_string(),
            ],
            stdin: None,
            timeout_ms: None,
            logs_dir,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("failed process still records observed outcome");

    assert_eq!(outcome.run.status, AgentRunStatus::Failed);
    assert_eq!(outcome.run.exit_code, Some(7));
    assert!(outcome
        .run
        .error_summary
        .as_deref()
        .unwrap_or_default()
        .contains("bad stderr"));
    assert!(outcome.run.log_reference_id.is_some());
    assert!(outcome.log_path.is_file());

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: Some("run.failed"),
            outcome: Some("succeeded"),
            source: Some("agent_run_repository"),
            limit: 10,
        },
    )
    .expect("list failed events");
    assert_eq!(events.len(), 1);
    let notification_count = count_rows(
        &connection,
        "select count(*) from notifications where run_id is not null and notification_type = 'failure' and severity = 'error'",
    );
    assert_eq!(notification_count, 1);
}

fn p205_run(connection: &Connection, task: &TaskRecord) -> AgentRunRecord {
    let profile = p204_profile(connection, true);
    let session = p204_session(connection, task, &profile);
    create_agent_run(
        connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id,
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create p205 run")
}

#[test]
fn p205_schema_version_seven_has_review_records_table() {
    let connection = migrated_in_memory_connection();
    assert!(
        get_migration_version(&connection).expect("migration version") >= 7,
        "P2.05 schema must remain present after later migrations"
    );
    assert_table_has_columns(
        &connection,
        "review_records",
        &[
            "id",
            "subject_type",
            "subject_id",
            "task_id",
            "run_id",
            "reviewer_profile_id",
            "state",
            "verdict",
            "evidence_summary",
            "required_fixes_json",
            "metadata_json",
            "created_at",
            "updated_at",
        ],
    );

    let task = p204_task(&connection, "DB rejects unverifiable related review");
    let direct_related_insert = connection.execute(
        "
            insert into review_records (
                id, subject_type, subject_id, task_id, run_id, reviewer_profile_id,
                state, verdict, evidence_summary, required_fixes_json, metadata_json
            ) values (?1, 'related_entity', 'unverified_related', ?2, null, null,
                'approved', 'approved', 'Direct insert should fail', '[]', '{}')
            ",
        params!["review_direct_related", task.id],
    );
    assert!(
        direct_related_insert.is_err(),
        "schema must reject unverifiable related_entity reviews until typed support exists"
    );

    let task_a = p204_task(&connection, "Review DB task A");
    let run_a = p205_run(&connection, &task_a);
    let task_b = p204_task(&connection, "Review DB task B");
    let mismatched_run_insert = connection.execute(
        "
            insert into review_records (
                id, subject_type, subject_id, task_id, run_id, reviewer_profile_id,
                state, verdict, evidence_summary, required_fixes_json, metadata_json
            ) values (?1, 'agent_run', ?2, ?3, ?2, null,
                'approved', 'approved', 'Mismatched run should fail', '[]', '{}')
            ",
        params!["review_mismatched_run", run_a.id, task_b.id],
    );
    assert!(
        mismatched_run_insert.is_err(),
        "schema must reject agent_run reviews whose run_id belongs to a different task_id"
    );

    let state_mismatch_insert = connection.execute(
        "
            insert into review_records (
                id, subject_type, subject_id, task_id, run_id, reviewer_profile_id,
                state, verdict, evidence_summary, required_fixes_json, metadata_json
            ) values (?1, 'task', ?2, ?2, null, null,
                'approved', 'required_fixes', 'State mismatch should fail', '[\"fix\"]', '{}')
            ",
        params!["review_state_mismatch", task_a.id],
    );
    assert!(
        state_mismatch_insert.is_err(),
        "schema must reject contradictory review state/verdict rows"
    );
}

#[test]
fn p205_manual_review_can_be_created_for_task_and_run() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Reviewable task");
    let run = p205_run(&connection, &task);

    let task_review = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::Task,
            subject_id: task.id.clone(),
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Manual reviewer verified task evidence".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create task review");
    assert_eq!(task_review.subject_type, ReviewSubjectType::Task);
    assert_eq!(task_review.task_id, task.id);
    assert_eq!(task_review.run_id, None);
    assert_eq!(task_review.reviewer_profile_id, None);
    assert_eq!(task_review.state, ReviewState::Approved);
    assert_eq!(task_review.verdict, ReviewVerdict::Approved);

    let run_review = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::RequiredFixes,
            evidence_summary: "Manual reviewer found a gap".to_string(),
            required_fixes_json: "[{\"fix\":\"add cited evidence\"}]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run review");
    assert_eq!(run_review.subject_type, ReviewSubjectType::AgentRun);
    assert_eq!(run_review.subject_id, run.id);
    assert_eq!(run_review.run_id.as_deref(), Some(run.id.as_str()));
    assert_eq!(run_review.state, ReviewState::RequiredFixes);
}

#[test]
fn p205_required_fixes_and_insufficient_evidence_require_non_empty_payloads() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Rejected review task");

    let missing_fixes = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::Task,
            subject_id: task.id.clone(),
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::RequiredFixes,
            evidence_summary: "Reviewed output".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("required fixes must be non-empty");
    assert!(matches!(
        missing_fixes,
        RepositoryError::Constraint {
            entity: "review_records",
            ..
        }
    ));

    let object_fixes = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::Task,
            subject_id: task.id.clone(),
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::RequiredFixes,
            evidence_summary: "Reviewed output".to_string(),
            required_fixes_json: "{\"fix\":\"not an array\"}".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("required fixes payload must be a non-empty JSON array");
    assert!(matches!(
        object_fixes,
        RepositoryError::Constraint {
            entity: "review_records",
            ..
        }
    ));

    let insufficient_evidence = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::Task,
            subject_id: task.id.clone(),
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::BlockedInsufficientEvidence,
            evidence_summary: "   ".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("insufficient evidence must explain the missing evidence");
    assert!(matches!(
        insufficient_evidence,
        RepositoryError::Constraint {
            entity: "review_records",
            ..
        }
    ));
}

#[test]
fn p205_review_events_are_written_and_linked_to_review_task_and_run() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Review events task");
    let run = p205_run(&connection, &task);

    let review = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Run summary and log reference are sufficient".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{\"safe\":true}".to_string(),
        },
    )
    .expect("create approved review");

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: None,
            outcome: Some("succeeded"),
            source: Some("review_record_repository"),
            limit: 10,
        },
    )
    .expect("list review events");
    assert!(events
        .iter()
        .any(|event| event.action_type == "review.created"));
    let approved = events
        .iter()
        .find(|event| event.action_type == "review.approved")
        .expect("approved event");
    assert!(approved.targets.iter().any(|target| {
        target.entity_type == "review_record"
            && target.entity_id == review.id
            && target.relation_type == "primary"
    }));
    assert!(approved.targets.iter().any(|target| {
        target.entity_type == "task"
            && target.entity_id == task.id
            && target.relation_type == "owner"
    }));
    assert!(approved.targets.iter().any(|target| {
        target.entity_type == "agent_run"
            && target.entity_id == run.id
            && target.relation_type == "run"
    }));

    let run_review_links = count_rows(
            &connection,
            "select count(*) from entity_links where source_type = 'agent_run' and target_type = 'review_record' and relation_type = 'reviewed_by'",
        );
    assert_eq!(run_review_links, 1);
}

#[test]
fn p205_approved_review_satisfies_gate_and_blocking_verdicts_do_not() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Review gate task");
    let run = p205_run(&connection, &task);

    assert!(!review_gate_satisfied_for_task(&connection, &task.id).expect("gate before review"));
    create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::BlockedInsufficientEvidence,
            evidence_summary: "Missing log reference for truthful verification".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create blocking review");
    assert!(!review_gate_satisfied_for_task(&connection, &task.id).expect("blocked gate"));

    create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Manual reviewer approved final evidence".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create approving review");
    assert!(review_gate_satisfied_for_task(&connection, &task.id).expect("approved gate"));

    create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::RequiredFixes,
            evidence_summary: "Later reviewer found missing proof".to_string(),
            required_fixes_json: "[{\"fix\":\"attach final log reference\"}]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create later blocking review");
    assert!(
        !review_gate_satisfied_for_task(&connection, &task.id).expect("later blocking gate"),
        "latest blocking review must make the review gate unsatisfied"
    );
}

#[test]
fn p205_related_entity_review_is_rejected_until_verifiable_subject_support_exists() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Related entity review task");

    let arbitrary = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::RelatedEntity,
            subject_id: "unverified_external_subject".to_string(),
            task_id: task.id,
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Cannot verify arbitrary related subject".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("related entity reviews need verifiable subject support");
    assert!(matches!(
        arbitrary,
        RepositoryError::Constraint {
            entity: "review_records",
            ..
        }
    ));
}

fn p206_review(connection: &Connection, task: &TaskRecord, run: &AgentRunRecord) -> ReviewRecord {
    create_review_record(
        connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Reviewer verified summarized evidence".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create p206 review")
}

#[test]
fn p206_schema_version_eight_has_notifications_table_and_constraints() {
    let connection = migrated_in_memory_connection();
    assert!(
        get_migration_version(&connection).expect("migration version") >= 8,
        "P2.06 notification schema must remain present after later migrations"
    );
    assert_table_has_columns(
        &connection,
        "notifications",
        &[
            "id",
            "notification_type",
            "title",
            "message",
            "severity",
            "state",
            "action_route",
            "task_id",
            "run_id",
            "review_record_id",
            "read_at",
            "dismissed_at",
            "resolved_at",
            "created_at",
            "updated_at",
            "metadata_json",
        ],
    );

    let task_a = p204_task(&connection, "Notification DB task A");
    let run_a = p205_run(&connection, &task_a);
    let review_a = p206_review(&connection, &task_a, &run_a);
    let task_b = p204_task(&connection, "Notification DB task B");
    let run_b = p205_run(&connection, &task_b);
    let task_level_review_a = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::Task,
            subject_id: task_a.id.clone(),
            task_id: task_a.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Task-level review for task A".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create task-level review");
    let mismatched_run = connection.execute(
            "
            insert into notifications (
                id, notification_type, title, message, severity, state,
                task_id, run_id, review_record_id, metadata_json
            ) values (?1, 'completion', 'Bad run', 'Mismatched run should fail', 'info', 'pending', ?2, ?3, null, '{}')
            ",
            params!["notification_bad_run", task_b.id, run_a.id],
        );
    assert!(
        mismatched_run.is_err(),
        "schema must reject run/task mismatches"
    );

    let mismatched_review = connection.execute(
            "
            insert into notifications (
                id, notification_type, title, message, severity, state,
                task_id, run_id, review_record_id, metadata_json
            ) values (?1, 'review_required', 'Bad review', 'Mismatched review should fail', 'warning', 'pending', ?2, null, ?3, '{}')
            ",
            params!["notification_bad_review", task_b.id, review_a.id],
        );
    assert!(
        mismatched_review.is_err(),
        "schema must reject review/task mismatches"
    );

    let mismatched_review_run = connection.execute(
            "
            insert into notifications (
                id, notification_type, title, message, severity, state,
                task_id, run_id, review_record_id, metadata_json
            ) values (?1, 'review_required', 'Bad review run', 'Task-level review cannot point at another task run', 'warning', 'pending', null, ?2, ?3, '{}')
            ",
            params!["notification_bad_review_run", run_b.id, task_level_review_a.id],
        );
    assert!(
        mismatched_review_run.is_err(),
        "schema must reject review/run ownership mismatches even when task_id is omitted"
    );

    for (id, state, read_at, dismissed_at, resolved_at) in [
        (
            "notification_read_missing",
            "read",
            Option::<&str>::None,
            Option::<&str>::None,
            Option::<&str>::None,
        ),
        (
            "notification_dismissed_missing",
            "dismissed",
            Option::<&str>::None,
            Option::<&str>::None,
            Option::<&str>::None,
        ),
        (
            "notification_resolved_missing",
            "resolved",
            Option::<&str>::None,
            Option::<&str>::None,
            Option::<&str>::None,
        ),
    ] {
        let rejected = connection.execute(
                "
                insert into notifications (
                    id, notification_type, title, message, severity, state,
                    task_id, read_at, dismissed_at, resolved_at, metadata_json
                ) values (?1, 'attention', 'Timestamp contradiction', 'Terminal state needs its timestamp',
                    'info', ?2, ?3, ?4, ?5, ?6, '{}')
                ",
                params![id, state, task_a.id, read_at, dismissed_at, resolved_at],
            );
        assert!(
            rejected.is_err(),
            "state {state} must require its timestamp"
        );
    }
}

#[test]
fn p206_notification_create_read_list_events_and_links() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Notification linked task");
    let run = p205_run(&connection, &task);
    let review = p206_review(&connection, &task, &run);

    let notification = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::ReviewRequired,
            title: "Review required".to_string(),
            message: "A run needs manual review based on summarized evidence".to_string(),
            severity: NotificationSeverity::Warning,
            action_route: Some(format!("zoid://reviews/{}", review.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: "{\"safe\":true}".to_string(),
        },
    )
    .expect("create notification");

    assert!(notification.id.starts_with("notification_"));
    assert_eq!(
        notification.notification_type,
        NotificationType::ReviewRequired
    );
    assert_eq!(notification.severity, NotificationSeverity::Warning);
    assert_eq!(notification.state, NotificationState::Pending);
    assert_eq!(notification.task_id.as_deref(), Some(task.id.as_str()));
    assert_eq!(notification.run_id.as_deref(), Some(run.id.as_str()));
    assert_eq!(
        notification.review_record_id.as_deref(),
        Some(review.id.as_str())
    );
    assert!(notification.read_at.is_none());
    assert!(notification.dismissed_at.is_none());
    assert!(notification.resolved_at.is_none());

    let read = read_notification(&connection, &notification.id)
        .expect("read notification")
        .expect("notification exists");
    assert_eq!(read, notification);
    let listed = list_inbox_notifications(&connection, true, 10).expect("list active inbox");
    assert_eq!(
        listed
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec![notification.id.as_str()]
    );

    let created_events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("inbox"),
            action_type: Some("notification.created"),
            outcome: Some("succeeded"),
            source: Some("notification_repository"),
            limit: 10,
        },
    )
    .expect("list notification events");
    assert_eq!(created_events.len(), 1);
    let event = &created_events[0];
    assert!(event
        .targets
        .iter()
        .any(|target| target.entity_type == "notification" && target.entity_id == notification.id));
    assert!(event
        .targets
        .iter()
        .any(|target| target.entity_type == "task" && target.entity_id == task.id));
    assert!(event
        .targets
        .iter()
        .any(|target| target.entity_type == "agent_run" && target.entity_id == run.id));
    assert!(event
        .targets
        .iter()
        .any(|target| target.entity_type == "review_record" && target.entity_id == review.id));

    for (source_type, source_id) in [
        ("task", task.id.as_str()),
        ("agent_run", run.id.as_str()),
        ("review_record", review.id.as_str()),
    ] {
        let link_count = count_rows(
                &connection,
                &format!(
                    "select count(*) from entity_links where source_type = '{source_type}' and source_id = '{source_id}' and target_type = 'notification' and relation_type = 'notifies'"
                ),
            );
        assert_eq!(link_count, 1, "{source_type} should link to notification");
    }
}

#[test]
fn p206_completion_blocker_failure_and_review_required_notifications_sort_actionable_inbox() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Notification sorting task");
    let run = p205_run(&connection, &task);
    let review = p206_review(&connection, &task, &run);

    let completion = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::Completion,
            title: "Completion notice".to_string(),
            message: "Run completed with summarized evidence".to_string(),
            severity: NotificationSeverity::Success,
            action_route: Some(format!("zoid://tasks/{}", task.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create completion notification");
    let blocker = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::Blocker,
            title: "Blocker notice".to_string(),
            message: "Run is blocked by missing evidence".to_string(),
            severity: NotificationSeverity::Critical,
            action_route: Some(format!("zoid://runs/{}", run.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create blocker notification");
    let failure = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::Failure,
            title: "Failure notice".to_string(),
            message: "Run failed after process exit".to_string(),
            severity: NotificationSeverity::Error,
            action_route: Some(format!("zoid://runs/{}", run.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create failure notification");
    let review_required = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::ReviewRequired,
            title: "Review required notice".to_string(),
            message: "Manual reviewer approval is needed".to_string(),
            severity: NotificationSeverity::Warning,
            action_route: Some(format!("zoid://reviews/{}", review.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create review-required notification");

    mark_notification_read(&connection, &failure.id).expect("read failure notification");
    assert!(!read_notification(&connection, &failure.id)
        .expect("read failure")
        .expect("failure exists")
        .state
        .is_active_inbox());

    let active = list_inbox_notifications(&connection, true, 10).expect("list active inbox");
    assert_eq!(
        active
            .iter()
            .map(|notification| notification.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            blocker.id.as_str(),
            review_required.id.as_str(),
            completion.id.as_str()
        ],
        "active inbox should exclude read items and sort by severity before time"
    );
}

#[test]
fn p206_repository_rejects_review_run_ownership_mismatch_without_task_id() {
    let connection = migrated_in_memory_connection();
    let task_a = p204_task(&connection, "Notification repository task A");
    let task_b = p204_task(&connection, "Notification repository task B");
    let run_b = p205_run(&connection, &task_b);
    let task_level_review_a = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::Task,
            subject_id: task_a.id.clone(),
            task_id: task_a.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Task-level approval for task A".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create task-level review");

    let rejected = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::ReviewRequired,
            title: "Bad review/run pairing".to_string(),
            message: "A task-level review cannot be paired with another task run".to_string(),
            severity: NotificationSeverity::Warning,
            action_route: Some(format!("zoid://runs/{}", run_b.id)),
            task_id: None,
            run_id: Some(run_b.id.clone()),
            review_record_id: Some(task_level_review_a.id.clone()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("repository must reject mismatched review/run ownership");
    assert!(matches!(
        rejected,
        RepositoryError::Constraint {
            entity: "notifications",
            ..
        }
    ));
    assert_eq!(
        count_rows(&connection, "select count(*) from notifications"),
        0
    );
}

#[test]
fn p206_notification_rejects_secret_material_before_persistence() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Secret notification task");
    let rejected = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::Failure,
            title: "Failure".to_string(),
            message: "Raw secret sk-1234567890abcdef1234567890abcdef1234567890abcdef leaked"
                .to_string(),
            severity: NotificationSeverity::Error,
            action_route: Some("zoid://tasks/secret?token=secret-token-value".to_string()),
            task_id: Some(task.id),
            run_id: None,
            review_record_id: None,
            metadata_json: "{\"api_key\":\"sk-raw-secret-value\"}".to_string(),
        },
    )
    .expect_err("notification secret material must be rejected");
    assert!(matches!(rejected, RepositoryError::SecretRejected { .. }));
    assert_eq!(
        count_rows(&connection, "select count(*) from notifications"),
        0
    );
}

#[test]
fn p206_notification_state_transitions_do_not_mutate_linked_task_run_or_review() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "Notification transition task");
    let run = p205_run(&connection, &task);
    let review = p206_review(&connection, &task, &run);
    let before_task = read_task_record(&connection, &task.id).expect("read task before");
    let before_run = read_agent_run_required(&connection, &run.id).expect("read run before");
    let before_review = read_review_record(&connection, &review.id)
        .expect("read review before")
        .expect("review before");

    let notification = create_notification(
        &connection,
        NotificationCreateInput {
            notification_type: NotificationType::Completion,
            title: "Completed".to_string(),
            message: "Run completed and is ready for attention".to_string(),
            severity: NotificationSeverity::Success,
            action_route: Some(format!("zoid://tasks/{}", task.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create transition notification");

    let delivered =
        mark_notification_delivered(&connection, &notification.id).expect("mark delivered");
    assert_eq!(delivered.state, NotificationState::Delivered);
    assert!(delivered.read_at.is_none());
    let action_required =
        require_notification_action(&connection, &notification.id).expect("require action");
    assert_eq!(action_required.state, NotificationState::ActionRequired);
    let failed = mark_notification_failed(&connection, &notification.id).expect("mark failed");
    assert_eq!(failed.state, NotificationState::Failed);
    assert!(failed.dismissed_at.is_none());
    let read = mark_notification_read(&connection, &notification.id).expect("mark read");
    assert_eq!(read.state, NotificationState::Read);
    assert!(read.read_at.is_some());
    let dismissed = dismiss_notification(&connection, &notification.id).expect("dismiss");
    assert_eq!(dismissed.state, NotificationState::Dismissed);
    assert!(dismissed.dismissed_at.is_some());
    let resolved = resolve_notification(&connection, &notification.id).expect("resolve");
    assert_eq!(resolved.state, NotificationState::Resolved);
    assert!(resolved.resolved_at.is_some());

    assert_eq!(
        read_task_record(&connection, &task.id).expect("read task after"),
        before_task
    );
    assert_eq!(
        read_agent_run_required(&connection, &run.id).expect("read run after"),
        before_run
    );
    assert_eq!(
        read_review_record(&connection, &review.id)
            .expect("read review after")
            .expect("review after"),
        before_review
    );

    for action in [
        "notification.delivered",
        "notification.action_required",
        "notification.failed",
        "notification.read",
        "notification.dismissed",
        "notification.resolved",
    ] {
        let events = list_event_records(
            &connection,
            EventListFilter {
                workspace_key: Some("inbox"),
                action_type: Some(action),
                outcome: Some("succeeded"),
                source: Some("notification_repository"),
                limit: 10,
            },
        )
        .expect("list transition events");
        assert_eq!(events.len(), 1, "{action} should be written once");
        assert!(events[0].targets.iter().any(|target| {
            target.entity_type == "notification"
                && target.entity_id == notification.id
                && target.relation_type == "primary"
        }));
    }
}

#[test]
fn p203_task_create_defaults_and_created_event_target_are_persisted() {
    let connection = migrated_in_memory_connection();

    let task = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Draft P2 task".to_string(),
            detail: Some("Implement repository coverage".to_string()),
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{\"source\":\"test\"}".to_string(),
        },
    )
    .expect("create task");

    assert!(task.id.starts_with("task_"));
    assert_eq!(task.title, "Draft P2 task");
    assert_eq!(
        task.detail.as_deref(),
        Some("Implement repository coverage")
    );
    assert_eq!(task.status, TaskStatus::Inbox);
    assert_eq!(task.priority, TaskPriority::Normal);
    assert_eq!(task.workspace_key, "tasks");
    assert_eq!(task.metadata_json, "{\"source\":\"test\"}");
    assert!(task.archived_at.is_none());
    assert!(task.deleted_at.is_none());
    assert!(!task.created_at.is_empty());
    assert!(!task.updated_at.is_empty());

    let read = read_task_record(&connection, &task.id).expect("read task");
    assert_eq!(read, task);

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("tasks"),
            action_type: Some("task.created"),
            outcome: Some("succeeded"),
            source: Some("task_repository"),
            limit: 10,
        },
    )
    .expect("list events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].targets.len(), 1);
    assert_eq!(events[0].targets[0].entity_type, "task");
    assert_eq!(events[0].targets[0].entity_id, task.id);
    assert_eq!(events[0].targets[0].relation_type, "primary");
}

#[test]
fn p203_task_validation_rejects_empty_oversized_invalid_metadata_and_invalid_enums() {
    let connection = migrated_in_memory_connection();

    let empty_title_error = create_task_record(
        &connection,
        TaskCreateInput {
            title: "   \n\t".to_string(),
            detail: None,
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("empty title must fail");
    assert!(matches!(
        empty_title_error,
        RepositoryError::Constraint {
            entity: "tasks",
            ..
        }
    ));

    let oversized_title = "x".repeat(TASK_TITLE_MAX_BYTES + 1);
    let oversized_title_error = create_task_record(
        &connection,
        TaskCreateInput {
            title: oversized_title,
            detail: None,
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("oversized title must fail");
    assert!(matches!(
        oversized_title_error,
        RepositoryError::Constraint {
            entity: "tasks",
            ..
        }
    ));

    let empty_detail_error = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Empty detail".to_string(),
            detail: Some("   \n\t".to_string()),
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("empty detail must fail");
    assert!(matches!(
        empty_detail_error,
        RepositoryError::Constraint {
            entity: "tasks",
            ..
        }
    ));

    let oversized_detail_error = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Oversized detail".to_string(),
            detail: Some("x".repeat(TASK_DETAIL_MAX_BYTES + 1)),
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("oversized detail must fail");
    assert!(matches!(
        oversized_detail_error,
        RepositoryError::Constraint {
            entity: "tasks",
            ..
        }
    ));

    let invalid_json_error = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Bad metadata".to_string(),
            detail: None,
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{not json}".to_string(),
        },
    )
    .expect_err("invalid metadata json must fail");
    assert!(matches!(
        invalid_json_error,
        RepositoryError::InvalidJson {
            field: "metadata_json",
            ..
        }
    ));

    let secret_metadata_error = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Secret metadata".to_string(),
            detail: None,
            status: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{\"token\":\"raw-secret-value\"}".to_string(),
        },
    )
    .expect_err("secret-like metadata must fail");
    assert!(matches!(
        secret_metadata_error,
        RepositoryError::SecretRejected {
            field: "metadata_json",
            ..
        }
    ));

    assert!(TaskStatus::from_str("done").is_err());
    assert!(TaskPriority::from_str("medium").is_err());
}

#[test]
fn p203_task_list_active_excludes_archived_and_deleted_and_orders_deterministically() {
    let connection = migrated_in_memory_connection();
    let low = create_task_record(
        &connection,
        TaskCreateInput::new("Low", Some(TaskPriority::Low)),
    )
    .expect("create low");
    let urgent = create_task_record(
        &connection,
        TaskCreateInput::new("Urgent", Some(TaskPriority::Urgent)),
    )
    .expect("create urgent");
    let high = create_task_record(
        &connection,
        TaskCreateInput::new("High", Some(TaskPriority::High)),
    )
    .expect("create high");

    archive_task(&connection, &low.id).expect("archive low");
    soft_delete_task(&connection, &high.id).expect("delete high");

    let archived = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Initially archived".to_string(),
            detail: None,
            status: Some(TaskStatus::Archived),
            priority: Some(TaskPriority::Urgent),
            workspace_key: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create initially archived");
    let deleted = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Initially deleted".to_string(),
            detail: None,
            status: Some(TaskStatus::Deleted),
            priority: Some(TaskPriority::Urgent),
            workspace_key: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create initially deleted");

    let active = list_active_tasks(&connection).expect("list active tasks");
    assert_eq!(
        active
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>(),
        vec![urgent.id.as_str()]
    );
    assert!(active[0].archived_at.is_none());
    assert!(active[0].deleted_at.is_none());
    assert!(active.iter().all(|task| task.id != archived.id));
    assert!(active.iter().all(|task| task.id != deleted.id));
}

#[test]
fn p203_task_status_update_writes_meaningful_event_and_secret_text_is_redacted() {
    let connection = migrated_in_memory_connection();
    let task = create_task_record(
        &connection,
        TaskCreateInput {
            title: "Rotate api_key=secret-value".to_string(),
            detail: None,
            status: None,
            priority: Some(TaskPriority::High),
            workspace_key: None,
            metadata_json: "{\"safe\":\"kept\"}".to_string(),
        },
    )
    .expect("create task");

    update_task_status(&connection, &task.id, TaskStatus::Active).expect("update status");
    update_task_status(&connection, &task.id, TaskStatus::Active).expect("same status no event");
    archive_task(&connection, &task.id).expect("archive task");
    soft_delete_task(&connection, &task.id).expect("delete task");

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("tasks"),
            action_type: Some("task.status_changed"),
            outcome: Some("succeeded"),
            source: Some("task_repository"),
            limit: 10,
        },
    )
    .expect("list status events");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].targets[0].entity_id, task.id);
    assert!(!events[0].summary.contains("secret-value"));
    assert!(!events[0].metadata_json.contains("raw-secret-value"));

    let archived_events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("tasks"),
            action_type: Some("task.archived"),
            outcome: Some("succeeded"),
            source: Some("task_repository"),
            limit: 10,
        },
    )
    .expect("list archived events");
    assert_eq!(archived_events.len(), 1);
    assert_eq!(archived_events[0].targets[0].entity_id, task.id);

    let deleted_events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("tasks"),
            action_type: Some("task.deleted"),
            outcome: Some("succeeded"),
            source: Some("task_repository"),
            limit: 10,
        },
    )
    .expect("list deleted events");
    assert_eq!(deleted_events.len(), 1);
    assert_eq!(deleted_events[0].targets[0].entity_id, task.id);
}

#[test]
fn visible_user_paths_have_expected_starter_directory_layout() {
    let home = PathBuf::from("/Users/example");
    let paths = VisibleUserPaths::for_home(&home);
    let expected: Vec<PathBuf> = VISIBLE_DIRS
        .iter()
        .map(|directory| PathBuf::from("/Users/example/Zoid").join(directory))
        .collect();

    assert_eq!(paths.root, PathBuf::from("/Users/example/Zoid"));
    assert_eq!(paths.starter_directories, expected);
    assert_eq!(paths.status().root, "/Users/example/Zoid");
    assert_eq!(paths.status().starter_directories.len(), VISIBLE_DIRS.len());
    for path in std::iter::once(&paths.root).chain(paths.starter_directories.iter()) {
        assert!(path.starts_with(&paths.root) || path == &paths.root);
        assert!(!path
            .components()
            .any(|component| matches!(component, Component::ParentDir)));
    }
}

#[test]
fn visible_user_creation_is_idempotent_and_non_destructive() {
    let home = temp_home("visible-idempotent");
    let paths = VisibleUserPaths::for_home(&home);
    let marker_path = paths.root.join("Notes").join("existing-note.md");

    ensure_visible_user_paths(&paths).expect("create visible paths");
    fs::write(&marker_path, "preserve this note").expect("write marker file");
    ensure_visible_user_paths(&paths).expect("create visible paths again");

    assert!(paths.root.is_dir());
    for starter_directory in &paths.starter_directories {
        assert!(starter_directory.is_dir(), "missing {starter_directory:?}");
    }
    assert_eq!(
        fs::read_to_string(marker_path).expect("read marker file"),
        "preserve this note"
    );

    fs::remove_dir_all(home).ok();
}

#[test]
fn visible_user_creation_fails_when_required_starter_directory_is_a_file() {
    let home = temp_home("visible-file-conflict");
    let paths = VisibleUserPaths::for_home(&home);
    fs::create_dir_all(&paths.root).expect("create visible root");
    fs::write(paths.root.join("Notes"), "not a directory").expect("write conflicting notes file");

    let error = ensure_visible_user_paths(&paths)
        .expect_err("starter directory file must block visible setup");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(error.to_string().contains("not a directory"));
    assert!(paths.root.join("Notes").is_file());

    fs::remove_dir_all(home).ok();
}

#[cfg(unix)]
#[test]
fn visible_user_creation_rejects_required_starter_directory_symlink() {
    let home = temp_home("visible-symlink-conflict");
    let paths = VisibleUserPaths::for_home(&home);
    let target = temp_home("visible-symlink-target");
    fs::create_dir_all(&paths.root).expect("create visible root");
    fs::create_dir_all(&target).expect("create symlink target");
    std::os::unix::fs::symlink(&target, paths.root.join("Notes")).expect("create notes symlink");

    let error = ensure_visible_user_paths(&paths)
        .expect_err("starter directory symlink must block visible setup");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(error.to_string().contains("symlink"));
    assert!(fs::symlink_metadata(paths.root.join("Notes"))
        .unwrap()
        .file_type()
        .is_symlink());

    fs::remove_dir_all(home).ok();
    fs::remove_dir_all(target).ok();
}

#[cfg(unix)]
#[test]
fn visible_user_creation_rejects_visible_root_symlink() {
    let home = temp_home("visible-root-symlink");
    let paths = VisibleUserPaths::for_home(&home);
    let target = temp_home("visible-root-symlink-target");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&target).expect("create target");
    std::os::unix::fs::symlink(&target, &paths.root).expect("create visible root symlink");

    let error = ensure_visible_user_paths(&paths)
        .expect_err("visible root symlink must block visible setup");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(error.to_string().contains("symlink"));
    assert!(fs::symlink_metadata(&paths.root)
        .unwrap()
        .file_type()
        .is_symlink());

    fs::remove_dir_all(home).ok();
    fs::remove_dir_all(target).ok();
}

#[test]
fn app_support_paths_have_expected_layout_without_user_input_segments() {
    let home = PathBuf::from("/Users/example");
    let paths = AppSupportPaths::for_home(&home);

    assert_eq!(
        paths.root,
        PathBuf::from("/Users/example/Library/Application Support/Zoid")
    );
    assert_eq!(paths.logs_dir, paths.root.join("logs"));
    assert_eq!(paths.database_parent, paths.root);
    assert_eq!(paths.database_path, paths.root.join("zoid.sqlite"));
    assert_eq!(paths.config_dir, paths.root.join("config"));
    assert_eq!(paths.config_path, paths.config_dir.join("settings.json"));
    assert!(paths.logs_dir.starts_with(&paths.root));
    assert!(paths.database_parent.starts_with(&paths.root));
    assert!(paths.database_path.starts_with(&paths.root));
    assert!(paths.config_path.starts_with(&paths.root));

    for path in [
        &paths.root,
        &paths.logs_dir,
        &paths.database_parent,
        &paths.database_path,
        &paths.config_dir,
        &paths.config_path,
    ] {
        assert!(!path
            .components()
            .any(|component| matches!(component, Component::ParentDir)));
    }
}

#[test]
fn app_support_creation_is_idempotent_and_non_destructive() {
    let home = temp_home("app-support-idempotent");
    let paths = AppSupportPaths::for_home(&home);
    let marker_path = paths.config_dir.join("existing-user-setting.json");

    ensure_app_support_paths(&paths).expect("create app support paths");
    fs::write(&marker_path, "preserve me").expect("write marker file");
    ensure_app_support_paths(&paths).expect("create app support paths again");

    assert!(paths.root.is_dir());
    assert!(paths.logs_dir.is_dir());
    assert!(paths.config_dir.is_dir());
    assert!(paths.database_parent.is_dir());
    assert!(
        !paths.database_path.exists(),
        "directory creation must not create the DB file"
    );
    assert!(
        !paths.config_path.exists(),
        "directory creation must not create the config file"
    );
    assert_eq!(fs::read_to_string(marker_path).unwrap(), "preserve me");

    fs::remove_dir_all(home).ok();
}

#[test]
fn app_support_creation_fails_when_required_directory_is_a_file() {
    let home = temp_home("app-support-file-conflict");
    let paths = AppSupportPaths::for_home(&home);
    fs::create_dir_all(&paths.root).expect("create app support root");
    fs::write(&paths.logs_dir, "not a directory").expect("write conflicting logs file");

    let error = ensure_app_support_paths(&paths).expect_err("logs file must block directory setup");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(paths.logs_dir.is_file());

    fs::remove_dir_all(home).ok();
}

#[cfg(unix)]
#[test]
fn app_support_creation_rejects_managed_directory_symlink_to_directory() {
    let home = temp_home("app-support-symlink-dir");
    let paths = AppSupportPaths::for_home(&home);
    let target = temp_home("app-support-symlink-dir-target");
    fs::create_dir_all(paths.root.parent().unwrap()).expect("create app support parent");
    fs::create_dir_all(&target).expect("create symlink target directory");
    std::os::unix::fs::symlink(&target, &paths.root).expect("create root symlink");

    let error =
        ensure_app_support_paths(&paths).expect_err("root symlink must block directory setup");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(fs::symlink_metadata(&paths.root)
        .unwrap()
        .file_type()
        .is_symlink());

    fs::remove_dir_all(home).ok();
    fs::remove_dir_all(target).ok();
}

#[cfg(unix)]
#[test]
fn app_support_creation_rejects_managed_directory_symlink_to_file() {
    let home = temp_home("app-support-symlink-file");
    let paths = AppSupportPaths::for_home(&home);
    let target = home.join("target-file");
    fs::create_dir_all(&paths.root).expect("create app support root");
    fs::write(&target, "not a directory").expect("write symlink target file");
    std::os::unix::fs::symlink(&target, &paths.logs_dir).expect("create logs symlink");

    let error =
        ensure_app_support_paths(&paths).expect_err("logs symlink must block directory setup");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(fs::symlink_metadata(&paths.logs_dir)
        .unwrap()
        .file_type()
        .is_symlink());

    fs::remove_dir_all(home).ok();
}

#[cfg(unix)]
#[test]
fn foundation_database_open_rejects_symlinked_database_file_before_sqlite() {
    let home = temp_home("foundation-db-symlink");
    let paths = AppSupportPaths::for_home(&home);
    let target = home.join("target.sqlite");
    fs::create_dir_all(&paths.root).expect("create app support root");
    fs::write(&target, "not opened by sqlite").expect("write symlink target");
    std::os::unix::fs::symlink(&target, &paths.database_path).expect("create database symlink");

    let error = open_foundation_database(&paths.database_path)
        .expect_err("database symlink must be rejected before sqlite open");

    assert!(error.to_string().contains("database"));
    assert!(error.to_string().contains("symlink"));
    assert_eq!(
        fs::read_to_string(&target).expect("read target after rejected open"),
        "not opened by sqlite"
    );
    assert!(fs::symlink_metadata(&paths.database_path)
        .unwrap()
        .file_type()
        .is_symlink());

    fs::remove_dir_all(home).ok();
}

#[test]
fn open_foundation_database_enables_foreign_keys() {
    let home = temp_home("database-foreign-keys");
    let paths = AppSupportPaths::for_home(&home);
    fs::create_dir_all(&paths.database_parent).expect("create database parent");

    let connection = open_foundation_database(&paths.database_path).expect("open database");
    let foreign_keys_enabled: i64 = connection
        .query_row("pragma foreign_keys", [], |row| row.get(0))
        .expect("read foreign key pragma");
    assert_eq!(foreign_keys_enabled, 1);

    drop(connection);
    fs::remove_dir_all(home).ok();
}

#[test]
fn file_backed_sqlite_migrations_seed_counts_and_foreign_keys_are_reenabled_after_reopen() {
    let home = temp_home("sqlite-migrations-reopen");
    let paths = AppSupportPaths::for_home(&home);
    fs::create_dir_all(&paths.database_parent).expect("create database parent");
    let expected_migration_version = MIGRATIONS
        .last()
        .expect("at least one migration is registered")
        .version;
    let expected_integration_statuses = canonical_workspace_registry()
        .iter()
        .map(|workspace| workspace.integrations.len() as i64)
        .sum::<i64>();

    {
        let connection = open_foundation_database(&paths.database_path).expect("open database");
        run_migrations(&connection).expect("run migrations");
        seed_workspaces(&connection).expect("seed workspaces");
        seed_default_integration_statuses(&connection).expect("seed integration statuses");

        assert_eq!(
            get_migration_version(&connection).unwrap(),
            expected_migration_version
        );
        assert_eq!(
            count_rows(&connection, "select count(*) from schema_migrations"),
            MIGRATIONS.len() as i64
        );
        assert_eq!(
            count_rows(&connection, "select count(*) from action_policies"),
            ACTION_POLICY_CATEGORIES.len() as i64
        );
        assert_eq!(
            count_rows(&connection, "select count(*) from workspaces"),
            canonical_workspace_registry().len() as i64
        );
        assert_eq!(
            count_rows(&connection, "select count(*) from integration_statuses"),
            expected_integration_statuses
        );
    }

    {
        let reopened = open_foundation_database(&paths.database_path).expect("reopen database");

        let foreign_keys_enabled: i64 = reopened
            .query_row("pragma foreign_keys", [], |row| row.get(0))
            .expect("read foreign key pragma after reopen");
        assert_eq!(foreign_keys_enabled, 1);
        assert_eq!(
            get_migration_version(&reopened).unwrap(),
            expected_migration_version
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from schema_migrations"),
            MIGRATIONS.len() as i64
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from action_policies"),
            ACTION_POLICY_CATEGORIES.len() as i64
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from workspaces"),
            canonical_workspace_registry().len() as i64
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from integration_statuses"),
            expected_integration_statuses
        );

        run_migrations(&reopened).expect("rerun migrations");
        seed_workspaces(&reopened).expect("reseed workspaces");
        seed_default_integration_statuses(&reopened).expect("reseed integration statuses");
        assert_eq!(
            get_migration_version(&reopened).unwrap(),
            expected_migration_version
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from schema_migrations"),
            MIGRATIONS.len() as i64
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from action_policies"),
            ACTION_POLICY_CATEGORIES.len() as i64
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from workspaces"),
            canonical_workspace_registry().len() as i64
        );
        assert_eq!(
            count_rows(&reopened, "select count(*) from integration_statuses"),
            expected_integration_statuses
        );
    }

    fs::remove_dir_all(home).ok();
}

#[test]
fn file_backed_repository_event_and_entity_links_persist_across_reopen() {
    let home = temp_home("sqlite-repository-reopen");
    let paths = AppSupportPaths::for_home(&home);
    fs::create_dir_all(&paths.database_parent).expect("create database parent");
    let created_event_id;

    {
        let connection = open_foundation_database(&paths.database_path).expect("open database");
        run_migrations(&connection).expect("run migrations");
        seed_workspaces(&connection).expect("seed workspaces");

        upsert_app_setting(
            &connection,
            AppSettingInput {
                key: "p1.25.persistence",
                value_json: "{\"enabled\":true}",
                value_type: "json",
                scope: "app",
                description: "P1.25 file-backed persistence probe",
            },
        )
        .expect("write app setting");

        let created_event = create_event_record(
            &connection,
            EventCreateInput {
                action_type: "p1_25_file_backed_roundtrip",
                outcome: "succeeded",
                actor_type: "system",
                actor_id: Some("sqlite_integration_test"),
                workspace_key: Some("tasks"),
                summary: "Persist event across file-backed reopen",
                source: "p1.25.sqlite.integration_test",
                metadata_json: "{\"probe\":\"event\"}",
                targets: vec![EventTargetInput {
                    entity_type: "task",
                    entity_id: "task-p1-25",
                    relation_type: "primary",
                }],
            },
        )
        .expect("write event");
        created_event_id = created_event.id.clone();

        create_entity_link(
            &connection,
            EntityLinkCreateRequest {
                id: "link-p1-25-task-event",
                source_type: "task",
                source_id: "task-p1-25",
                target_type: "event",
                target_id: &created_event_id,
                relation_type: "emitted_event",
                created_by_actor_type: "system",
                metadata_json: "{\"probe\":\"entity_link\"}",
            },
        )
        .expect("write entity link");
    }

    {
        let reopened = open_foundation_database(&paths.database_path).expect("reopen database");
        run_migrations(&reopened).expect("rerun migrations");

        let setting = read_app_setting(&reopened, "p1.25.persistence")
            .expect("read app setting after reopen")
            .expect("app setting persisted");
        assert_eq!(setting.value_json, "{\"enabled\":true}");
        assert_eq!(setting.scope, "app");

        let event =
            read_event_record(&reopened, &created_event_id).expect("read event after reopen");
        assert_eq!(event.action_type, "p1_25_file_backed_roundtrip");
        assert_eq!(event.outcome, "succeeded");
        assert_eq!(event.workspace_key.as_deref(), Some("tasks"));
        assert_eq!(event.targets.len(), 1);
        assert_eq!(event.targets[0].entity_type, "task");
        assert_eq!(event.targets[0].entity_id, "task-p1-25");
        assert_eq!(event.targets[0].relation_type, "primary");

        let link = get_entity_link(&reopened, "link-p1-25-task-event")
            .expect("read entity link after reopen")
            .expect("entity link persisted");
        assert_eq!(link.source_type, "task");
        assert_eq!(link.source_id, "task-p1-25");
        assert_eq!(link.target_type, "event");
        assert_eq!(link.target_id, created_event_id);
        assert_eq!(link.relation_type, "emitted_event");
    }

    fs::remove_dir_all(home).ok();
}

#[test]
fn migrations_seed_core_workspaces() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    let workspace_ids: Vec<String> = list_workspaces(&connection)
        .unwrap()
        .into_iter()
        .map(|workspace| workspace.id)
        .collect();

    assert_eq!(
        get_migration_version(&connection).unwrap(),
        MIGRATIONS
            .last()
            .expect("at least one migration is registered")
            .version
    );
    assert_eq!(workspace_ids, canonical_workspace_ids());
}

#[test]
fn canonical_workspace_registry_has_exactly_fourteen_unique_workspaces_in_order() {
    let registry = canonical_workspace_registry();
    let ids: Vec<&str> = registry.iter().map(|workspace| workspace.key).collect();
    let unique_ids: HashSet<&str> = ids.iter().copied().collect();

    assert_eq!(registry.len(), 14);
    assert_eq!(unique_ids.len(), 14);
    assert_eq!(
        ids,
        vec![
            "today",
            "tasks",
            "notes",
            "agents",
            "code",
            "content",
            "automations",
            "business",
            "products",
            "files",
            "browser",
            "inbox",
            "calendar",
            "history"
        ]
    );
    for (expected_position, workspace) in registry.iter().enumerate() {
        assert_eq!(workspace.position, expected_position as i64);
        assert!(!workspace.label.trim().is_empty());
        assert!(!workspace.description.trim().is_empty());
        assert!(!workspace.status_note.trim().is_empty());
    }
}

#[test]
fn canonical_workspace_registry_does_not_claim_unbuilt_integrations_are_connected_or_ready() {
    let guarded_integrations = [
        "gmail",
        "apple_calendar",
        "omnisocials",
        "browser_webview",
        "hermes_cli",
        "git_cli",
        "automation_cli",
    ];
    let forbidden_states = ["connected", "ready", "functional"];

    for workspace in canonical_workspace_registry() {
        for integration in workspace.integrations {
            if guarded_integrations.contains(&integration.key) {
                assert!(
                    !forbidden_states.contains(&integration.state.as_str()),
                    "{} incorrectly claims {} is {}",
                    workspace.key,
                    integration.key,
                    integration.state.as_str()
                );
                assert!(
                    !integration.note.trim().is_empty(),
                    "{} integration {} must explain truthful state",
                    workspace.key,
                    integration.key
                );
            }
        }
    }
}

#[test]
fn workspace_records_list_truthful_registry_metadata_for_foundation_ui() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    let workspaces = list_workspaces(&connection).expect("list workspaces");
    let inbox = workspaces
        .iter()
        .find(|workspace| workspace.id == "inbox")
        .expect("inbox workspace exists");
    let calendar = workspaces
        .iter()
        .find(|workspace| workspace.id == "calendar")
        .expect("calendar workspace exists");

    assert_eq!(workspaces.len(), 14);
    assert_eq!(inbox.availability, WorkspaceAvailability::Available);
    assert_eq!(
        inbox.integration_state("gmail"),
        Some(WorkspaceIntegrationState::NotConfigured)
    );
    assert_eq!(
        calendar.integration_state("apple_calendar"),
        Some(WorkspaceIntegrationState::NeedsPermission)
    );
    assert!(inbox.status_note.contains("local"));
    assert!(calendar.status_note.contains("permission"));
}

#[test]
fn workspace_seeding_is_registry_backed_and_idempotent() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces first time");
    seed_workspaces(&connection).expect("seed workspaces second time");

    let workspaces = list_workspaces(&connection).expect("list workspaces");
    let ids: Vec<String> = workspaces
        .iter()
        .map(|workspace| workspace.id.clone())
        .collect();
    let db_count = count_table(&connection, "workspaces").expect("count workspaces");

    assert_eq!(ids, canonical_workspace_ids());
    assert_eq!(db_count, canonical_workspace_registry().len() as i64);
    for (definition, record) in canonical_workspace_registry().iter().zip(workspaces.iter()) {
        assert_eq!(record.id, definition.key);
        assert_eq!(record.label, definition.label);
        assert_eq!(record.description, definition.description);
        assert_eq!(record.position, definition.position);
        assert_eq!(record.availability, definition.availability);
        assert_eq!(record.integrations, definition.integrations);
        assert_eq!(record.status_note, definition.status_note);
    }
}

#[test]
fn repository_upserts_reads_lists_and_updates_app_settings() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    upsert_app_setting(
        &connection,
        AppSettingInput {
            key: "ui.theme",
            value_json: "\"dark\"",
            value_type: "string",
            scope: "app",
            description: "Current theme",
        },
    )
    .expect("upsert app setting");
    upsert_app_setting(
        &connection,
        AppSettingInput {
            key: "workspace.sort",
            value_json: "{\"by\":\"updated_at\"}",
            value_type: "json",
            scope: "workspace",
            description: "Workspace sort",
        },
    )
    .expect("upsert workspace setting");

    let setting = read_app_setting(&connection, "ui.theme")
        .expect("read setting")
        .expect("setting exists");
    assert_eq!(setting.key, "ui.theme");
    assert_eq!(setting.value_json, "\"dark\"");
    assert_eq!(setting.value_type, "string");
    assert_eq!(setting.scope, "app");
    assert_eq!(setting.description, "Current theme");

    let app_settings = list_app_settings_by_scope(&connection, "app").expect("list app settings");
    assert_eq!(
        app_settings
            .iter()
            .map(|setting| setting.key.as_str())
            .collect::<Vec<_>>(),
        vec!["ui.theme"]
    );
    let all_settings = list_app_settings(&connection).expect("list all settings");
    assert_eq!(
        all_settings
            .iter()
            .map(|setting| setting.key.as_str())
            .collect::<Vec<_>>(),
        vec!["ui.theme", "workspace.sort"]
    );

    let updated = update_app_setting(
        &connection,
        "ui.theme",
        AppSettingUpdate {
            value_json: "\"light\"",
            value_type: "string",
            scope: "workspace",
            description: "Updated theme",
        },
    )
    .expect("update setting");
    assert_eq!(updated.value_json, "\"light\"");
    assert_eq!(updated.scope, "workspace");
    assert_eq!(updated.description, "Updated theme");

    let missing_error = update_app_setting(
        &connection,
        "missing.setting",
        AppSettingUpdate {
            value_json: "null",
            value_type: "json",
            scope: "app",
            description: "missing",
        },
    )
    .expect_err("missing setting should be typed not found");
    assert!(
        matches!(missing_error, RepositoryError::NotFound { entity, key } if entity == "app_settings" && key == "missing.setting")
    );
}

#[test]
fn repository_classifies_invalid_json_and_setting_constraint_errors() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let invalid_json_error = upsert_app_setting(
        &connection,
        AppSettingInput {
            key: "bad.json",
            value_json: "{not json}",
            value_type: "json",
            scope: "app",
            description: "Bad JSON",
        },
    )
    .expect_err("invalid json should be typed");
    assert!(
        matches!(invalid_json_error, RepositoryError::InvalidJson { field, .. } if field == "value_json")
    );

    let constraint_error = upsert_app_setting(
        &connection,
        AppSettingInput {
            key: "bad.scope",
            value_json: "null",
            value_type: "json",
            scope: "invalid_scope",
            description: "Bad scope",
        },
    )
    .expect_err("invalid scope should be typed constraint");
    assert!(matches!(
        constraint_error,
        RepositoryError::Constraint { .. }
    ));
}

#[test]
fn settings_service_saves_reads_and_lists_local_non_secret_preferences() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let saved = upsert_local_app_preference(
        &connection,
        LocalPreferenceInput {
            key: "ui.theme",
            value_json: "\"dark\"",
            value_type: "string",
            scope: SettingScope::App,
            description: "Current local theme preference",
        },
    )
    .expect("save local preference");

    assert_eq!(saved.key, "ui.theme");
    assert_eq!(saved.value_json, "\"dark\"");
    assert_eq!(saved.scope, SettingScope::App);

    let read = read_local_app_preference(&connection, "ui.theme")
        .expect("read local preference")
        .expect("preference exists");
    assert_eq!(read, saved);

    upsert_local_app_preference(
        &connection,
        LocalPreferenceInput {
            key: "workspace.sidebar.collapsed",
            value_json: "true",
            value_type: "boolean",
            scope: SettingScope::Workspace,
            description: "Sidebar state",
        },
    )
    .expect("save workspace preference");

    let all = list_local_app_preferences(&connection).expect("list local preferences");
    assert_eq!(
        all.iter()
            .map(|setting| setting.key.as_str())
            .collect::<Vec<_>>(),
        vec!["ui.theme", "workspace.sidebar.collapsed"]
    );
    let app_only = list_local_app_preferences_by_scope(&connection, SettingScope::App)
        .expect("list app preferences");
    assert_eq!(app_only.len(), 1);
    assert_eq!(app_only[0].key, "ui.theme");

    let db_count: i64 = connection
        .query_row("select count(*) from app_settings", [], |row| row.get(0))
        .expect("count app_settings rows");
    assert_eq!(db_count, 2);
}

#[test]
fn settings_service_rejects_secret_like_local_preferences_and_invalid_json() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let secret_key_error = upsert_local_app_preference(
        &connection,
        LocalPreferenceInput {
            key: "gmail.access_token",
            value_json: "\"not-a-secret-for-test\"",
            value_type: "string",
            scope: SettingScope::App,
            description: "must reject secret-like keys",
        },
    )
    .expect_err("secret-like preference keys must be rejected");
    assert!(
        matches!(secret_key_error, RepositoryError::SecretRejected { field, .. } if field == "key")
    );

    let secret_value_error = upsert_local_app_preference(
        &connection,
        LocalPreferenceInput {
            key: "integration.gmail",
            value_json: "{\"api_key\":\"abc123\"}",
            value_type: "json",
            scope: SettingScope::App,
            description: "must reject secret-like JSON",
        },
    )
    .expect_err("secret-like preference JSON must be rejected");
    assert!(
        matches!(secret_value_error, RepositoryError::SecretRejected { field, .. } if field == "value_json")
    );

    let invalid_json_error = upsert_local_app_preference(
        &connection,
        LocalPreferenceInput {
            key: "ui.bad_json",
            value_json: "{not json}",
            value_type: "json",
            scope: SettingScope::App,
            description: "must reject invalid JSON",
        },
    )
    .expect_err("invalid preference JSON must be typed");
    assert!(
        matches!(invalid_json_error, RepositoryError::InvalidJson { field, .. } if field == "value_json")
    );

    let db_count: i64 = connection
        .query_row("select count(*) from app_settings", [], |row| row.get(0))
        .expect("count app_settings rows");
    assert_eq!(db_count, 0);
}

#[test]
fn integration_status_service_seeds_truthful_registry_statuses_idempotently() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    seed_default_integration_statuses(&connection).expect("seed integration statuses");
    seed_default_integration_statuses(&connection).expect("seed integration statuses again");

    let statuses = list_integration_statuses(&connection).expect("list integration statuses");
    let gmail = statuses
        .iter()
        .find(|status| status.integration_key == "gmail")
        .expect("gmail status exists");
    let apple_calendar = statuses
        .iter()
        .find(|status| status.integration_key == "apple_calendar")
        .expect("apple calendar status exists");
    let browser = statuses
        .iter()
        .find(|status| status.integration_key == "browser_webview")
        .expect("browser status exists");

    assert_eq!(gmail.status, IntegrationStatus::NotConfigured);
    assert_eq!(apple_calendar.status, IntegrationStatus::NotConfigured);
    assert!(apple_calendar.config_json.contains("needs_permission"));
    assert_eq!(browser.status, IntegrationStatus::Disabled);
    assert!(statuses
        .iter()
        .all(|status| status.status != IntegrationStatus::Connected));
    assert!(statuses
        .iter()
        .all(|status| status.credential_ref.is_none()));

    let db_count: i64 = connection
        .query_row("select count(*) from integration_statuses", [], |row| {
            row.get(0)
        })
        .expect("count integration_statuses rows");
    let registry_integration_count: usize = canonical_workspace_registry()
        .iter()
        .map(|workspace| workspace.integrations.len())
        .sum();
    assert_eq!(db_count, registry_integration_count as i64);
}

#[test]
fn integration_status_service_safely_upserts_reads_lists_and_preserves_seed_idempotence() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_default_integration_statuses(&connection).expect("seed integration statuses");

    let updated = upsert_integration_status(
        &connection,
        IntegrationStatusInput {
            integration_key: "gmail",
            display_name: "Gmail",
            status: IntegrationStatus::Configured,
            config_json: "{\"account\":\"ziad@example.com\",\"scopes\":[\"metadata.readonly\"]}",
            credential_ref: Some("keychain://zoid/integrations/gmail/default"),
            last_checked_at: Some("2026-06-01T00:00:00Z"),
        },
    )
    .expect("safe integration status update");
    assert_eq!(updated.status, IntegrationStatus::Configured);
    assert_eq!(
        updated.credential_ref.as_deref(),
        Some("keychain://zoid/integrations/gmail/default")
    );

    seed_default_integration_statuses(&connection)
        .expect("seeding must not overwrite explicit status");
    let read = read_integration_status(&connection, "gmail")
        .expect("read integration status")
        .expect("gmail status exists");
    assert_eq!(read.status, IntegrationStatus::Configured);
    assert!(read.config_json.contains("ziad@example.com"));

    let statuses = list_integration_statuses(&connection).expect("list integration statuses");
    assert_eq!(statuses.first().unwrap().integration_key, "apple_calendar");

    let db_status: String = connection
        .query_row(
            "select status from integration_statuses where integration_key = 'gmail'",
            [],
            |row| row.get(0),
        )
        .expect("read raw integration status");
    assert_eq!(db_status, "configured");
}

#[test]
fn integration_status_service_rejects_secret_config_invalid_json_and_raw_credential_refs() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let secret_config_error = upsert_integration_status(
        &connection,
        IntegrationStatusInput {
            integration_key: "gmail",
            display_name: "Gmail",
            status: IntegrationStatus::Configured,
            config_json: "{\"refresh_token\":\"abc123\"}",
            credential_ref: None,
            last_checked_at: None,
        },
    )
    .expect_err("secret-like integration config must be rejected");
    assert!(
        matches!(secret_config_error, RepositoryError::SecretRejected { field, .. } if field == "config_json")
    );

    let bearer_config_error = upsert_integration_status(
        &connection,
        IntegrationStatusInput {
            integration_key: "gmail",
            display_name: "Gmail",
            status: IntegrationStatus::Configured,
            config_json: "{\"header\":\"Authorization: Bearer sk-live-1234567890\"}",
            credential_ref: None,
            last_checked_at: None,
        },
    )
    .expect_err("bearer token config must be rejected");
    assert!(matches!(
        bearer_config_error,
        RepositoryError::SecretRejected { .. }
    ));

    let invalid_json_error = upsert_integration_status(
        &connection,
        IntegrationStatusInput {
            integration_key: "gmail",
            display_name: "Gmail",
            status: IntegrationStatus::NotConfigured,
            config_json: "{not json}",
            credential_ref: None,
            last_checked_at: None,
        },
    )
    .expect_err("invalid integration config JSON must be typed");
    assert!(
        matches!(invalid_json_error, RepositoryError::InvalidJson { field, .. } if field == "config_json")
    );

    let raw_credential_ref_error = upsert_integration_status(
        &connection,
        IntegrationStatusInput {
            integration_key: "gmail",
            display_name: "Gmail",
            status: IntegrationStatus::Configured,
            config_json: "{}",
            credential_ref: Some("sk-live-1234567890abcdef"),
            last_checked_at: None,
        },
    )
    .expect_err("raw token-like credential refs must be rejected");
    assert!(
        matches!(raw_credential_ref_error, RepositoryError::SecretRejected { field, .. } if field == "credential_ref")
    );

    let db_count: i64 = connection
        .query_row("select count(*) from integration_statuses", [], |row| {
            row.get(0)
        })
        .expect("count integration_statuses rows");
    assert_eq!(db_count, 0);
}

#[test]
fn p401_phase4_schema_has_lightweight_repo_and_launch_gate_tables() {
    let connection = migrated_in_memory_connection();
    assert!(get_migration_version(&connection).expect("migration version") >= 10);
    assert_table_has_columns(
        &connection,
        "repo_profiles",
        &[
            "id",
            "display_name",
            "root_path",
            "profile_type",
            "default_branch",
            "package_manager",
            "linked_product_id",
            "status",
            "metadata_json",
        ],
    );
    assert_table_has_columns(
        &connection,
        "launch_gates",
        &[
            "id",
            "repo_id",
            "product_id",
            "task_id",
            "state",
            "final_verdict",
            "metadata_json",
        ],
    );
    assert_table_has_columns(
        &connection,
        "launch_gate_evidence",
        &[
            "id",
            "launch_gate_id",
            "evidence_type",
            "label",
            "url",
            "status_code",
            "manual_note",
            "metadata_json",
        ],
    );
}

#[test]
fn p403_repo_registry_adds_lists_and_links_without_git_status_diff_surface() {
    let connection = migrated_in_memory_connection();
    let repo = add_repo_profile(
        &connection,
        RepoProfileInput {
            display_name: "Zoid app".to_string(),
            root_path: "/Users/ziadnasreldin/Zoid".to_string(),
            profile_type: "product_app".to_string(),
            default_branch: Some("main".to_string()),
            package_manager: Some("npm".to_string()),
            linked_product_id: Some("zoid".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("add repo profile");
    assert_eq!(repo.status, "active");
    assert_eq!(
        read_repo_profile(&connection, &repo.id)
            .expect("read repo")
            .expect("repo exists")
            .id,
        repo.id
    );
    assert_eq!(
        list_repo_profiles(&connection).expect("list repos").len(),
        1
    );
    let link = link_repo_entity(
        &connection,
        RepoEntityLinkInput {
            repo_id: repo.id.clone(),
            target_type: "product".to_string(),
            target_id: "zoid".to_string(),
            relation_type: "source_repo".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("link repo to product");
    assert_eq!(link.source_type, "repo");
    assert_eq!(link.target_type, "product");
    assert!(!TAURI_BRIDGE_COMMAND_NAMES
        .iter()
        .any(|name| name.contains("git_status")
            || name.contains("git_diff")
            || name.contains("commit_command")
            || name.contains("push_command")));
}

#[test]
fn p404_repo_integration_states_are_truthful_not_connected() {
    let connection = migrated_in_memory_connection();
    let states = list_phase4_repo_integration_states(&connection).expect("integration states");
    let github = states
        .iter()
        .find(|state| state.integration_key == "github")
        .expect("github state");
    let vercel = states
        .iter()
        .find(|state| state.integration_key == "vercel")
        .expect("vercel state");
    assert_eq!(github.status, IntegrationStatus::NotConfigured);
    assert_eq!(vercel.status, IntegrationStatus::NotConfigured);
    assert!(github.credential_ref.is_none());
    assert!(vercel.credential_ref.is_none());
}

#[test]
fn p405_commit_push_merge_deploy_are_policy_previews_not_executions() {
    assert_eq!(
        normalize_launch_action_policy_category("commit").expect("commit category"),
        "commit_push_merge"
    );
    assert_eq!(
        normalize_launch_action_policy_category("push").expect("push category"),
        "commit_push_merge"
    );
    assert_eq!(
        normalize_launch_action_policy_category("deploy").expect("deploy category"),
        "deploy_redeploy_rollback"
    );
    let policy =
        preview_launch_action_policy_command("deploy".to_string()).expect("policy preview");
    assert_eq!(policy.category, "deploy_redeploy_rollback");
}

#[test]
fn p408_launch_gate_fails_closed_until_real_evidence_exists() {
    let connection = migrated_in_memory_connection();
    let repo = add_repo_profile(
        &connection,
        RepoProfileInput {
            display_name: "Launch repo".to_string(),
            root_path: "/tmp/launch-repo".to_string(),
            profile_type: "product_app".to_string(),
            default_branch: Some("main".to_string()),
            package_manager: None,
            linked_product_id: Some("zoid".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("add repo");
    let gate = create_launch_gate(
        &connection,
        LaunchGateCreateInput {
            repo_id: repo.id,
            product_id: Some("zoid".to_string()),
            task_id: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create gate");
    assert_eq!(gate.state, LaunchGateState::VerificationBlocked);
    assert_eq!(
        gate.final_verdict.as_deref(),
        Some("blocked_missing_evidence")
    );
    let blocked = evaluate_launch_gate(&connection, &gate.id).expect("evaluate empty gate");
    assert_eq!(blocked.state, LaunchGateState::VerificationBlocked);
    add_launch_gate_evidence(
        &connection,
        LaunchGateEvidenceInput {
            launch_gate_id: gate.id.clone(),
            evidence_type: "test_output".to_string(),
            label: "cargo test focused pass".to_string(),
            url: None,
            status_code: None,
            manual_note: Some("Focused Phase 4 test output captured locally.".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("add evidence");
    let verified = evaluate_launch_gate(&connection, &gate.id).expect("evaluate with evidence");
    assert_eq!(verified.state, LaunchGateState::Verified);
    assert_eq!(
        verified.final_verdict.as_deref(),
        Some("verified_with_evidence")
    );
}

#[test]
fn p501_phase5_schema_has_content_and_omnisocials_tables() {
    let connection = migrated_in_memory_connection();
    assert!(get_migration_version(&connection).expect("migration version") >= 11);
    for (table, columns) in [
        (
            "content_plans",
            vec![
                "id",
                "title",
                "pillar",
                "status",
                "owner_actor_type",
                "metadata_json",
            ],
        ),
        (
            "content_pieces",
            vec![
                "id",
                "plan_id",
                "title",
                "body_markdown",
                "status",
                "platforms_json",
                "required_gate",
            ],
        ),
        (
            "media_assets",
            vec![
                "id",
                "piece_id",
                "asset_kind",
                "storage_ref",
                "alt_text",
                "metadata_json",
            ],
        ),
        (
            "content_review_gates",
            vec!["id", "piece_id", "gate_type", "status", "evidence_summary"],
        ),
        (
            "content_schedules",
            vec![
                "id",
                "piece_id",
                "platform",
                "scheduled_for",
                "status",
                "confirmation_id",
            ],
        ),
        (
            "content_verification_records",
            vec![
                "id",
                "piece_id",
                "schedule_id",
                "platform",
                "action_type",
                "outcome",
                "failure_report",
            ],
        ),
        (
            "omnisocials_accounts",
            vec!["id", "platform", "state", "credential_ref", "status_note"],
        ),
    ] {
        assert_table_has_columns(&connection, table, &columns);
    }
}

#[test]
fn p503_phase5_content_draft_asset_review_schedule_flow_is_draft_first() {
    let connection = migrated_in_memory_connection();
    let plan = create_content_plan(
        &connection,
        ContentPlanInput {
            title: "Enterprise ops carousel".to_string(),
            pillar: Some("operational_control".to_string()),
            owner_actor_type: Some("human".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("create content plan");
    let piece = create_content_piece(
        &connection,
        ContentPieceInput {
            plan_id: plan.id,
            title: "Visibility beats guessing".to_string(),
            body_markdown: Some("Draft caption".to_string()),
            platforms: Some(vec!["linkedin".to_string(), "instagram".to_string()]),
            required_gate: Some("specialist_review".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("create content piece");
    assert_eq!(piece.status, "draft");
    let instagram_constraints =
        validate_platform_media_constraints(&connection, &piece.id, "instagram")
            .expect("validate constraints");
    assert!(!instagram_constraints.passed);
    assert!(instagram_constraints
        .violations
        .contains(&"instagram_requires_media".to_string()));
    add_media_asset_reference(
        &connection,
        MediaAssetInput {
            piece_id: piece.id.clone(),
            asset_kind: "image".to_string(),
            storage_ref: "assets/content/visibility.png".to_string(),
            mime_type: Some("image/png".to_string()),
            byte_size: Some(1024),
            width: Some(1080),
            height: Some(1080),
            duration_seconds: None,
            alt_text: Some("Dashboard visibility illustration".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("add media");
    let gate = create_content_review_gate(
        &connection,
        ContentReviewGateInput {
            piece_id: piece.id.clone(),
            gate_type: "specialist_review".to_string(),
            reviewer_actor_type: Some("reviewer".to_string()),
            reviewer_actor_id: Some("designer-reviewer".to_string()),
            evidence_summary: Some("Designer/reviewer approval required.".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("create review gate");
    let blocked = create_content_schedule(
        &connection,
        ContentScheduleInput {
            piece_id: piece.id.clone(),
            platform: "instagram".to_string(),
            scheduled_for: "2026-06-05T18:00:00+02:00".to_string(),
            confirmation_id: None,
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect_err("schedule blocked before review and confirmation");
    let blocked_message = repository_error_message(blocked);
    assert!(blocked_message.contains("review gate") || blocked_message.contains("confirmation"));
    decide_content_review_gate(
        &connection,
        &gate.id,
        true,
        ContentReviewGateDecisionInput {
            evidence_summary: "Approved after specialist design/review.".to_string(),
            reviewer_actor_type: Some("reviewer".to_string()),
            reviewer_actor_id: Some("designer-reviewer".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("approve review gate");
    let confirmation = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "publish_schedule_content",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(Some("p5-test")),
            summary: "Approve schedule intent only; no external publish yet.",
            event_id: None,
            metadata_json: "{}",
        },
    )
    .expect("persist confirmation decision");
    let schedule = create_content_schedule(
        &connection,
        ContentScheduleInput {
            piece_id: piece.id.clone(),
            platform: "instagram".to_string(),
            scheduled_for: "2026-06-05T18:00:00+02:00".to_string(),
            confirmation_id: Some(confirmation.id),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("create schedule intent");
    assert_eq!(schedule.status, "intent");
}

#[test]
fn p504_phase5_omnisocials_fails_closed_and_records_failure() {
    let connection = migrated_in_memory_connection();
    let plan = create_content_plan(
        &connection,
        ContentPlanInput {
            title: "Ops content".to_string(),
            pillar: None,
            owner_actor_type: None,
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("plan");
    let piece = create_content_piece(
        &connection,
        ContentPieceInput {
            plan_id: plan.id,
            title: "Fail closed post".to_string(),
            body_markdown: Some("draft".to_string()),
            platforms: Some(vec!["linkedin".to_string()]),
            required_gate: Some("none".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("piece");
    let status = get_omnisocials_status(&connection).expect("omnisocials status");
    assert_eq!(status.state, "not_configured");
    let verification = omnisocials_upload_media(&connection, &piece.id, "linkedin")
        .expect("record blocked verification");
    assert_eq!(verification.outcome, "blocked");
    assert_eq!(verification.action_type, "upload");
    assert!(verification
        .failure_report
        .unwrap_or_default()
        .contains("not configured"));
    let records = list_content_verification_records(
        &connection,
        ContentVerificationListRequest {
            piece_id: Some(piece.id),
            schedule_id: None,
            limit: Some(10),
        },
    )
    .expect("verification records");
    assert_eq!(records.len(), 1);
}

#[test]
fn p502_phase5_schema_has_indexes_and_richer_status_states() {
    let connection = migrated_in_memory_connection();
    for index_name in [
        "idx_content_plans_status_updated",
        "idx_content_pieces_plan_status",
        "idx_media_assets_piece_kind",
        "idx_content_review_gates_piece_type_status",
        "idx_content_schedules_piece_platform_status",
        "idx_content_verification_piece_action",
        "idx_content_verification_schedule_action",
        "idx_omnisocials_accounts_platform_state",
    ] {
        let exists: i64 = connection
            .query_row(
                "select count(*) from sqlite_master where type='index' and name=?1",
                params![index_name],
                |row| row.get(0),
            )
            .expect("index lookup");
        assert_eq!(exists, 1, "missing index {index_name}");
    }
    connection
        .execute(
            "update omnisocials_accounts set state='needs_permission' where id='omnisocials-default'",
            [],
        )
        .expect("needs_permission is valid");
    connection
        .execute(
            "update omnisocials_accounts set state='disabled_by_policy' where id='omnisocials-default'",
            [],
        )
        .expect("disabled_by_policy is valid");
}

#[test]
fn p505_phase5_schedule_blocks_bad_confirmation_and_records_evidence() {
    let connection = migrated_in_memory_connection();
    let plan = create_content_plan(
        &connection,
        ContentPlanInput {
            title: "Ops content".to_string(),
            pillar: None,
            owner_actor_type: None,
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("plan");
    let piece = create_content_piece(
        &connection,
        ContentPieceInput {
            plan_id: plan.id,
            title: "Blocked confirmation post".to_string(),
            body_markdown: Some("draft".to_string()),
            platforms: Some(vec!["linkedin".to_string()]),
            required_gate: Some("specialist_review".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("piece");
    let gate = create_content_review_gate(
        &connection,
        ContentReviewGateInput {
            piece_id: piece.id.clone(),
            gate_type: "specialist_review".to_string(),
            reviewer_actor_type: Some("reviewer".to_string()),
            reviewer_actor_id: None,
            evidence_summary: Some("pending".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("gate");
    decide_content_review_gate(
        &connection,
        &gate.id,
        true,
        ContentReviewGateDecisionInput {
            evidence_summary: "approved".to_string(),
            reviewer_actor_type: Some("reviewer".to_string()),
            reviewer_actor_id: None,
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("approve");
    let denied = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "publish_schedule_content",
            decision: ConfirmationDecisionState::Denied,
            actor: ConfirmationActor::human(Some("p5-test")),
            summary: "Do not schedule this content.",
            event_id: None,
            metadata_json: "{}",
        },
    )
    .expect("denied confirmation");
    let err = create_content_schedule(
        &connection,
        ContentScheduleInput {
            piece_id: piece.id.clone(),
            platform: "linkedin".to_string(),
            scheduled_for: "2026-06-05T18:00:00+02:00".to_string(),
            confirmation_id: Some(denied.id),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect_err("denied confirmation blocks schedule");
    assert!(repository_error_message(err).contains("confirmation_denied"));
    let records = list_content_verification_records(
        &connection,
        ContentVerificationListRequest {
            piece_id: Some(piece.id),
            schedule_id: None,
            limit: Some(10),
        },
    )
    .expect("records");
    assert!(records
        .iter()
        .any(|record| record.action_type == "schedule" && record.outcome == "blocked"));
}

#[test]
fn p506_phase5_media_constraints_events_and_secret_safety() {
    let connection = migrated_in_memory_connection();
    let plan = create_content_plan(
        &connection,
        ContentPlanInput {
            title: "Ops content".to_string(),
            pillar: None,
            owner_actor_type: None,
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("plan");
    let piece = create_content_piece(
        &connection,
        ContentPieceInput {
            plan_id: plan.id,
            title: "Asset safety post".to_string(),
            body_markdown: Some("draft".to_string()),
            platforms: Some(vec!["instagram".to_string()]),
            required_gate: Some("none".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("piece");
    let unsafe_asset = add_media_asset_reference(
        &connection,
        MediaAssetInput {
            piece_id: piece.id.clone(),
            asset_kind: "image".to_string(),
            storage_ref: "../secret-token.png".to_string(),
            mime_type: Some("image/png".to_string()),
            byte_size: Some(1024),
            width: Some(1080),
            height: Some(1080),
            duration_seconds: None,
            alt_text: None,
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect_err("unsafe refs are rejected");
    assert!(repository_error_message(unsafe_asset).contains("unsafe media storage reference"));
    add_media_asset_reference(
        &connection,
        MediaAssetInput {
            piece_id: piece.id.clone(),
            asset_kind: "image".to_string(),
            storage_ref: "assets/content/safe.png".to_string(),
            mime_type: Some("image/png".to_string()),
            byte_size: Some(1024),
            width: Some(1080),
            height: Some(1080),
            duration_seconds: None,
            alt_text: Some("Safe content asset".to_string()),
            metadata_json: Some("{}".to_string()),
        },
    )
    .expect("safe asset");
    let assets = list_media_asset_references(&connection, &piece.id).expect("asset list");
    assert_eq!(assets.len(), 1);
    let event_count: i64 = connection
        .query_row(
            "select count(*) from events where source='phase5_content_omnisocials'",
            [],
            |row| row.get(0),
        )
        .expect("phase5 event count");
    assert!(event_count >= 3, "expected plan/piece/asset events");
}

#[test]
fn tauri_bridge_command_surface_lists_registered_p116_commands() {
    assert!(TAURI_BRIDGE_COMMAND_NAMES.len() >= 33);
    for command_name in [
        "get_foundation_status",
        "get_workspace_registry",
        "read_local_preference",
        "list_local_preferences",
        "upsert_local_preference",
        "read_integration_status_command",
        "list_integration_statuses_command",
        "upsert_integration_status_command",
        "create_event",
        "read_event",
        "list_events",
        "preview_action_policy",
        "create_task_command",
        "read_task_command",
        "list_tasks_command",
        "update_task_command",
        "update_task_status_command",
        "archive_task_command",
        "delete_task_command",
        "start_agent_run_command",
        "read_run_status_command",
        "stream_run_output_command",
        "cancel_run_command",
        "create_manual_review_command",
        "read_review_record_command",
        "create_notification_command",
        "read_notification_command",
        "list_inbox_notifications_command",
        "update_notification_state_command",
        "list_task_history_command",
        "list_run_history_command",
        "list_notification_history_command",
        "list_entity_history_command",
        "list_content_entity_links_by_source_command",
    ] {
        assert!(
            TAURI_BRIDGE_COMMAND_NAMES.contains(&command_name),
            "missing command registration marker for {command_name}"
        );
    }

    let source_commands = parse_generate_handler_command_names(include_str!("lib.rs"));
    assert_eq!(source_commands.len(), TAURI_BRIDGE_COMMAND_NAMES.len());
    for command_name in TAURI_BRIDGE_COMMAND_NAMES {
        assert!(
            source_commands.contains(command_name),
            "missing command in generate_handler block for {command_name}"
        );
    }
}

#[test]
fn p310_tauri_bridge_command_surface_registers_notes_and_files_commands() {
    assert_eq!(TAURI_BRIDGE_COMMAND_NAMES.len(), 77);
    for command_name in [
        "list_content_entity_links_by_source_command",
        "add_repo_profile_command",
        "list_repo_profiles_command",
        "read_repo_profile_command",
        "link_repo_entity_command",
        "list_repo_integration_states_command",
        "create_launch_gate_command",
        "read_launch_gate_command",
        "add_launch_gate_evidence_command",
        "evaluate_launch_gate_command",
        "preview_launch_action_policy_command",
        "create_markdown_note_command",
        "read_note_command",
        "list_notes_command",
        "edit_markdown_note_command",
        "trash_markdown_note_command",
        "delete_markdown_note_command",
        "scan_markdown_notes_command",
        "list_note_conflicts_command",
        "accept_note_conflict_command",
        "browse_files_command",
        "open_file_reference_command",
        "preview_file_command",
        "perform_file_action_command",
        "create_content_plan_command",
        "list_content_plans_command",
        "create_content_piece_command",
        "read_content_piece_command",
        "list_content_pieces_command",
        "update_content_piece_draft_command",
        "add_media_asset_reference_command",
        "list_media_asset_references_command",
        "create_content_review_gate_command",
        "approve_content_review_gate_command",
        "reject_content_review_gate_command",
        "list_content_review_gates_command",
        "create_content_schedule_command",
        "list_content_schedules_command",
        "cancel_content_schedule_command",
        "get_omnisocials_status_command",
        "omnisocials_upload_media_command",
        "omnisocials_schedule_content_command",
        "omnisocials_publish_content_command",
        "list_content_verification_records_command",
    ] {
        assert!(
            TAURI_BRIDGE_COMMAND_NAMES.contains(&command_name),
            "missing P3.10 command registration marker for {command_name}"
        );
    }
    let source_commands = parse_generate_handler_command_names(include_str!("lib.rs"));
    for command_name in TAURI_BRIDGE_COMMAND_NAMES {
        assert!(
            source_commands.contains(command_name),
            "missing command in generate_handler block for {command_name}"
        );
    }
}

#[test]
fn p310_note_bridge_commands_cover_crud_scan_and_conflicts() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p310-note-bridge");
    let created = create_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        NoteCommandCreateRequest {
            title: "Bridge Note".to_string(),
            body_markdown: "Initial body".to_string(),
            relative_path: None,
            metadata_json: Some("{\"safe\":\"yes\"}".to_string()),
        },
    )
    .expect("create note through bridge helper");
    assert_eq!(created.title, "Bridge Note");

    let listed = list_notes_command_with_connection(
        &connection,
        &visible_root,
        NoteCommandListRequest {
            status: Some("active".to_string()),
            include_markdown: Some(true),
            limit: Some(20),
        },
    )
    .expect("list notes through bridge helper");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);
    assert!(listed[0].markdown.contains("Initial body"));

    let read = read_note_command_with_connection(&connection, &visible_root, created.id.clone())
        .expect("read note through bridge helper");
    assert_eq!(read.id, created.id);

    let edited = edit_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        created.id.clone(),
        NoteCommandEditRequest {
            markdown: "Updated body".to_string(),
        },
    )
    .expect("edit note through bridge helper");
    assert!(edited.markdown.contains("Updated body"));

    fs::write(visible_root.join("Notes/external.md"), "# External\n\nBody")
        .expect("write external note");
    let scan = scan_markdown_notes_command_with_connection(&connection, &visible_root)
        .expect("scan notes through bridge helper");
    assert!(scan.scanned_files >= 2);
    assert!(scan.indexed_notes >= 1);

    let trashed =
        trash_markdown_note_command_with_connection(&connection, &visible_root, created.id.clone())
            .expect("trash note through bridge helper");
    assert_eq!(trashed.status, "trashed");
    let deleted = delete_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        created.id.clone(),
    )
    .expect("delete note through bridge helper");
    assert_eq!(deleted.status, "deleted");

    let conflicts = list_note_conflicts_command_with_connection(&connection)
        .expect("list conflicts through bridge helper");
    assert!(conflicts.is_empty());
    let missing_accept = accept_note_conflict_command_with_connection(
        &connection,
        &visible_root,
        "missing-note".to_string(),
    )
    .expect_err("accept missing conflict should fail truthfully");
    assert!(missing_accept.contains("NotFound"));
}

#[test]
fn p310_note_bridge_reports_missing_markdown_files_truthfully() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p310-note-bridge-missing-file");
    let created = create_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        NoteCommandCreateRequest {
            title: "Missing File Note".to_string(),
            body_markdown: "Body".to_string(),
            relative_path: None,
            metadata_json: None,
        },
    )
    .expect("create note through bridge helper");
    fs::remove_file(visible_root.join(&created.relative_path)).expect("remove markdown file");

    let read_error =
        read_note_command_with_connection(&connection, &visible_root, created.id.clone())
            .expect_err("read bridge should report missing markdown file");
    assert!(read_error.contains("notes"));
    assert!(read_error.contains("No such file") || read_error.contains("os error"));

    let list_error = list_notes_command_with_connection(
        &connection,
        &visible_root,
        NoteCommandListRequest {
            status: Some("active".to_string()),
            include_markdown: Some(true),
            limit: Some(20),
        },
    )
    .expect_err("list bridge with markdown should report missing markdown file");
    assert!(list_error.contains("notes"));
}

#[test]
fn p310_file_bridge_commands_browse_preview_and_require_persisted_confirmation_for_actions() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p310-file-bridge");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(visible_root.join("Files/source.md"), "# Source").expect("write source");

    let entries = browse_files_command_with_connection(
        &connection,
        &visible_root,
        FileBrowseCommandRequest {
            root_key: "zoid_visible".to_string(),
            relative_path: "Files".to_string(),
        },
    )
    .expect("browse files through bridge helper");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].relative_path, "Files/source.md");

    let opened = open_file_reference_command_with_connection(
        &connection,
        &visible_root,
        FileReferenceCommandRequest {
            root_key: "zoid_visible".to_string(),
            relative_path: "Files/source.md".to_string(),
        },
    )
    .expect("open file through bridge helper");
    assert_eq!(opened.display_name, "source.md");

    let preview = preview_file_command_with_connection(
        &connection,
        &visible_root,
        FileReferenceCommandRequest {
            root_key: "zoid_visible".to_string(),
            relative_path: "Files/source.md".to_string(),
        },
    )
    .expect("preview file through bridge helper");
    assert_eq!(preview.preview_text, "# Source");

    let blocked = perform_file_action_command_with_connection(
        &connection,
        &visible_root,
        FileActionCommandRequest {
            action: "copy".to_string(),
            root_key: "zoid_visible".to_string(),
            source_relative_path: "Files/source.md".to_string(),
            destination_relative_path: Some("Files/copy.md".to_string()),
            confirmation_id: None,
        },
    )
    .expect_err("file action bridge must not accept frontend-only confirmation");
    assert!(blocked.contains("confirmation_required"));
    assert!(!visible_root.join("Files/copy.md").exists());

    let confirmation = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "move_rename_copy_file",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(Some("p310-test")),
            summary: "Approve copy for bridge test",
            event_id: None,
            metadata_json: "{}",
        },
    )
    .expect("persist confirmation decision");

    let copied = perform_file_action_command_with_connection(
        &connection,
        &visible_root,
        FileActionCommandRequest {
            action: "copy".to_string(),
            root_key: "zoid_visible".to_string(),
            source_relative_path: "Files/source.md".to_string(),
            destination_relative_path: Some("Files/copy.md".to_string()),
            confirmation_id: Some(confirmation.id),
        },
    )
    .expect("confirmed copy succeeds through bridge helper");
    assert_eq!(copied.action, "copy");
    assert!(visible_root.join("Files/copy.md").exists());

    let unsupported = perform_file_action_command_with_connection(
        &connection,
        &visible_root,
        FileActionCommandRequest {
            action: "shred".to_string(),
            root_key: "zoid_visible".to_string(),
            source_relative_path: "Files/source.md".to_string(),
            destination_relative_path: None,
            confirmation_id: None,
        },
    )
    .expect_err("unsupported file action should fail closed");
    assert!(unsupported.contains("unsupported file action"));
}

#[test]
fn p319_manual_note_workflow_persists_after_restart_and_matches_disk_state() {
    let (connection, database_path) = migrated_file_connection("p319-note-manual");
    let visible_root = temp_home("p319-note-visible");

    let created = create_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        NoteCommandCreateRequest {
            title: "Manual Disk Note".to_string(),
            body_markdown: "Initial manual body".to_string(),
            relative_path: Some("Notes/manual-disk.md".to_string()),
            metadata_json: None,
        },
    )
    .expect("create manual note");
    assert!(visible_root.join(&created.relative_path).exists());

    let edited = edit_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        created.id.clone(),
        NoteCommandEditRequest {
            markdown: "# Manual Disk Note\n\nEdited manual body".to_string(),
        },
    )
    .expect("edit manual note");
    assert!(edited.markdown.contains("Edited manual body"));

    let trashed =
        trash_markdown_note_command_with_connection(&connection, &visible_root, created.id.clone())
            .expect("trash manual note");
    assert_eq!(trashed.status, "trashed");
    assert!(visible_root.join(&trashed.relative_path).exists());

    let deleted = delete_markdown_note_command_with_connection(
        &connection,
        &visible_root,
        created.id.clone(),
    )
    .expect("soft delete manual note");
    assert_eq!(deleted.status, "deleted");
    assert!(visible_root.join(&deleted.relative_path).exists());

    drop(connection);
    let reopened = open_foundation_database(&database_path).expect("reopen p319 database");
    let persisted = read_note_command_with_connection(&reopened, &visible_root, created.id.clone())
        .expect("read note after restart");
    assert_eq!(persisted.status, "deleted");
    assert!(persisted.markdown.contains("Edited manual body"));
    assert!(visible_root.join(&persisted.relative_path).exists());
}

#[test]
fn p320_manual_file_workflow_browses_previews_and_performs_confirmed_safe_operation() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p320-file-manual");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(
        visible_root.join("Files/source.md"),
        "# Source\n\nManual preview",
    )
    .expect("write source file");

    let entries = browse_files_command_with_connection(
        &connection,
        &visible_root,
        FileBrowseCommandRequest {
            root_key: "zoid_visible".to_string(),
            relative_path: "Files".to_string(),
        },
    )
    .expect("browse manual files");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].relative_path, "Files/source.md");

    let opened = open_file_reference_command_with_connection(
        &connection,
        &visible_root,
        FileReferenceCommandRequest {
            root_key: "zoid_visible".to_string(),
            relative_path: "Files/source.md".to_string(),
        },
    )
    .expect("open manual file");
    assert_eq!(opened.display_name, "source.md");

    let preview = preview_file_command_with_connection(
        &connection,
        &visible_root,
        FileReferenceCommandRequest {
            root_key: "zoid_visible".to_string(),
            relative_path: "Files/source.md".to_string(),
        },
    )
    .expect("preview manual file");
    assert!(preview.preview_text.contains("Manual preview"));

    let blocked = perform_file_action_command_with_connection(
        &connection,
        &visible_root,
        FileActionCommandRequest {
            action: "copy".to_string(),
            root_key: "zoid_visible".to_string(),
            source_relative_path: "Files/source.md".to_string(),
            destination_relative_path: Some("Files/confirmed-copy.md".to_string()),
            confirmation_id: None,
        },
    )
    .expect_err("manual copy without confirmation is blocked");
    assert!(blocked.contains("confirmation_required"));
    assert!(!visible_root.join("Files/confirmed-copy.md").exists());

    let confirmation = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "move_rename_copy_file",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(Some("p320-test")),
            summary: "Approve safe manual copy",
            event_id: None,
            metadata_json: "{}",
        },
    )
    .expect("persist manual copy confirmation");

    perform_file_action_command_with_connection(
        &connection,
        &visible_root,
        FileActionCommandRequest {
            action: "copy".to_string(),
            root_key: "zoid_visible".to_string(),
            source_relative_path: "Files/source.md".to_string(),
            destination_relative_path: Some("Files/confirmed-copy.md".to_string()),
            confirmation_id: Some(confirmation.id),
        },
    )
    .expect("confirmed manual copy succeeds");
    assert_eq!(
        fs::read_to_string(visible_root.join("Files/confirmed-copy.md"))
            .expect("read copied bytes"),
        "# Source\n\nManual preview"
    );
}

#[test]
fn tauri_bridge_workspace_registry_command_returns_all_14_workspaces() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    let workspaces = get_workspace_registry_with_connection(&connection)
        .expect("workspace registry command succeeds");

    assert_eq!(workspaces.len(), 14);
    assert_eq!(workspaces[0].id, "today");
    assert_eq!(workspaces[13].id, "history");
    assert_eq!(
        workspaces
            .iter()
            .map(|workspace| workspace.id.clone())
            .collect::<Vec<_>>(),
        canonical_workspace_ids()
    );
}

#[test]
fn tauri_bridge_policy_preview_is_read_only_and_gates_high_risk_action() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    let before_count = count_table(&connection, "events").expect("count events before");

    let decision = preview_action_policy(PolicyPreviewRequest {
        category: None,
        action_type: Some("deploy".to_string()),
        target: Some("production deploy".to_string()),
        scope: Some("external".to_string()),
        consequence: Some("public_release".to_string()),
        bulk: Some(false),
        destructive: Some(false),
    })
    .expect("preview policy");

    let after_count = count_table(&connection, "events").expect("count events after");
    assert_eq!(before_count, after_count);
    assert_eq!(decision.category, "deploy_redeploy_rollback");
    assert!(decision.requires_gate);
    assert!(!decision.allowed_now);
}

#[test]
fn tauri_bridge_policy_preview_allows_low_risk_and_rejects_invalid_parse_inputs() {
    let low_risk = preview_action_policy(PolicyPreviewRequest {
        category: None,
        action_type: Some("read".to_string()),
        target: Some("local note".to_string()),
        scope: Some("local_private".to_string()),
        consequence: Some("harmless_local".to_string()),
        bulk: Some(false),
        destructive: Some(false),
    })
    .expect("low-risk policy preview should parse and allow");

    assert_eq!(low_risk.category, "read_local_app_data");
    assert_eq!(low_risk.policy, ActionPolicy::Allow);
    assert!(low_risk.allowed_now);
    assert!(!low_risk.requires_gate);

    for (field, request, expected_error) in [
        (
            "action_type",
            PolicyPreviewRequest {
                category: None,
                action_type: Some("exfiltrate".to_string()),
                target: None,
                scope: None,
                consequence: None,
                bulk: None,
                destructive: None,
            },
            "unsupported action_type: exfiltrate",
        ),
        (
            "scope",
            PolicyPreviewRequest {
                category: None,
                action_type: Some("read".to_string()),
                target: None,
                scope: Some("planetary".to_string()),
                consequence: None,
                bulk: None,
                destructive: None,
            },
            "unsupported scope: planetary",
        ),
        (
            "consequence",
            PolicyPreviewRequest {
                category: None,
                action_type: Some("read".to_string()),
                target: None,
                scope: None,
                consequence: Some("catastrophic".to_string()),
                bulk: None,
                destructive: None,
            },
            "unsupported consequence: catastrophic",
        ),
    ] {
        let error = match preview_action_policy(request) {
            Ok(decision) => panic!("{field} must reject unsupported value, got {decision:?}"),
            Err(error) => error,
        };
        assert_eq!(error, expected_error);
    }
}

#[test]
fn tauri_bridge_event_write_redacts_and_read_list_return_record() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    let created = create_event_with_connection(
        &connection,
        EventCreateRequest {
            action_type: "create_local_task".to_string(),
            outcome: "succeeded".to_string(),
            actor_type: "system".to_string(),
            actor_id: Some("tauri_bridge_test".to_string()),
            workspace_key: Some("tasks".to_string()),
            summary: "Created task with api_key=super-secret".to_string(),
            source: "tauri_bridge_test".to_string(),
            metadata_json: "{\"token\":\"ghp_secretvalue\",\"safe\":\"kept\"}".to_string(),
            targets: vec![EventTargetRequest {
                entity_type: "task".to_string(),
                entity_id: "task-1".to_string(),
                relation_type: "primary".to_string(),
            }],
        },
    )
    .expect("create event through bridge");

    assert!(created.summary.contains("[REDACTED]"));
    assert!(!created.summary.contains("super-secret"));
    assert!(created.metadata_json.contains("[REDACTED]"));
    assert!(!created.metadata_json.contains("ghp_secretvalue"));
    assert_eq!(created.targets.len(), 1);

    let read = read_event_with_connection(&connection, created.id.clone()).expect("read event");
    assert_eq!(read, created);
    let listed = list_events_with_connection(
        &connection,
        EventListRequest {
            workspace_key: Some("tasks".to_string()),
            action_type: Some("create_local_task".to_string()),
            outcome: None,
            source: Some("tauri_bridge_test".to_string()),
            limit: Some(25),
        },
    )
    .expect("list events");
    assert_eq!(listed, vec![created]);
}

#[test]
fn read_event_missing_id_returns_not_found() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let error = read_event_with_connection(&connection, "event-missing".to_string())
        .expect_err("missing event should return not found");

    assert!(error.contains("NotFound"));
    assert!(error.contains("events"));
    assert!(error.contains("event-missing"));
}

#[test]
fn tauri_bridge_event_write_rejects_over_limit_targets_without_persisting() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    let mut request = valid_event_create_request();
    request.targets = (0..26)
        .map(|index| EventTargetRequest {
            entity_type: "task".to_string(),
            entity_id: format!("task-{index}"),
            relation_type: "related".to_string(),
        })
        .collect();

    let error = create_event_with_connection(&connection, request)
        .expect_err("over-limit targets must be rejected");

    assert_eq!(error, "event request exceeds bridge limit: targets");
    assert_eq!(
        count_table(&connection, "events").expect("count events after rejected request"),
        0
    );
}

#[test]
fn tauri_bridge_event_write_rejects_over_limit_payload_without_persisting() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    for (field, over_limit_value) in [
        ("summary", "x".repeat(4097)),
        (
            "metadata_json",
            format!("{{\"payload\":\"{}\"}}", "x".repeat(16_384)),
        ),
    ] {
        let mut request = valid_event_create_request();
        if field == "summary" {
            request.summary = over_limit_value;
        } else {
            request.metadata_json = over_limit_value;
        }

        let error = create_event_with_connection(&connection, request)
            .expect_err("over-limit event payload must be rejected");

        assert_eq!(
            error,
            format!("event request exceeds bridge limit: {field}")
        );
    }

    assert_eq!(
        count_table(&connection, "events").expect("count events after rejected requests"),
        0
    );
}

#[test]
fn tauri_bridge_settings_reject_secrets_invalid_json_and_do_not_persist() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let secret_preference = upsert_local_preference_with_connection(
        &connection,
        LocalPreferenceRequest {
            key: "ui.api_key".to_string(),
            value_json: "\"not stored\"".to_string(),
            value_type: "string".to_string(),
            scope: "app".to_string(),
            description: "secret-like key".to_string(),
        },
    )
    .expect_err("secret-like preference must be rejected");
    assert!(secret_preference.contains("secret-like"));

    let invalid_preference = upsert_local_preference_with_connection(
        &connection,
        LocalPreferenceRequest {
            key: "ui.bad".to_string(),
            value_json: "{not json}".to_string(),
            value_type: "json".to_string(),
            scope: "app".to_string(),
            description: "invalid json".to_string(),
        },
    )
    .expect_err("invalid preference JSON must be rejected");
    assert!(invalid_preference.contains("InvalidJson"));

    let secret_integration = upsert_integration_status_with_connection(
        &connection,
        IntegrationStatusRequest {
            integration_key: "gmail".to_string(),
            display_name: "Gmail".to_string(),
            status: "configured".to_string(),
            config_json: "{\"refresh_token\":\"abc123\"}".to_string(),
            credential_ref: None,
            last_checked_at: None,
        },
    )
    .expect_err("secret-like integration metadata must be rejected");
    assert!(secret_integration.contains("secret-like"));

    assert!(list_local_preferences_with_connection(&connection, None)
        .expect("list preferences")
        .is_empty());
    assert!(list_integration_statuses_with_connection(&connection)
        .expect("list statuses")
        .is_empty());
}

#[test]
fn repository_inserts_entity_links_and_handles_unique_conflicts() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let link = insert_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-001",
            source_type: "workspace",
            source_id: "today",
            target_type: "task",
            target_id: "task-001",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{\"rank\":1}",
        },
    )
    .expect("insert entity link");
    assert_eq!(link.id, "link-001");
    assert_eq!(link.metadata_json, "{\"rank\":1}");

    let conflict_error = insert_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-002",
            source_type: "workspace",
            source_id: "today",
            target_type: "task",
            target_id: "task-001",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err("duplicate logical link should be a typed conflict");
    assert!(matches!(conflict_error, RepositoryError::Constraint { .. }));

    let links =
        list_entity_links_for_source(&connection, "workspace", "today").expect("list source links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].id, "link-001");

    let same_link = insert_or_get_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-002",
            source_type: "workspace",
            source_id: "today",
            target_type: "task",
            target_id: "task-001",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("idempotent entity link insert");
    assert_eq!(same_link.id, "link-001");
}

#[test]
fn insert_or_get_entity_link_preserves_constraint_for_id_collision() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    insert_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-001",
            source_type: "workspace",
            source_id: "today",
            target_type: "task",
            target_id: "task-001",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("insert original entity link");

    let error = insert_or_get_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-001",
            source_type: "workspace",
            source_id: "tomorrow",
            target_type: "task",
            target_id: "task-002",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err("primary key collision with a different logical tuple should remain a constraint");

    assert!(matches!(error, RepositoryError::Constraint { .. }));
}

#[test]
fn insert_or_get_entity_link_preserves_constraint_for_id_collision_with_existing_logical_tuple() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    insert_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-001",
            source_type: "workspace",
            source_id: "today",
            target_type: "task",
            target_id: "task-001",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("insert original entity link A");

    insert_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-002",
            source_type: "workspace",
            source_id: "tomorrow",
            target_type: "task",
            target_id: "task-002",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("insert original entity link B");

    let error = insert_or_get_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-001",
            source_type: "workspace",
            source_id: "tomorrow",
            target_type: "task",
            target_id: "task-002",
            relation_type: "contains",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err(
        "primary key collision with a different row's logical tuple should remain a constraint",
    );

    assert!(matches!(error, RepositoryError::Constraint { .. }));
}

#[test]
fn entity_link_service_creates_reads_and_lists_task_note_links() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let created = create_entity_link(
        &connection,
        EntityLinkCreateRequest {
            id: "link-task-note-001",
            source_type: "task",
            source_id: "task-001",
            target_type: "note",
            target_id: "note-001",
            relation_type: "references",
            created_by_actor_type: "agent",
            metadata_json: "{\"rank\":1}",
        },
    )
    .expect("create task to note entity link");

    assert_eq!(created.id, "link-task-note-001");
    assert_eq!(created.source_type, "task");
    assert_eq!(created.source_id, "task-001");
    assert_eq!(created.target_type, "note");
    assert_eq!(created.target_id, "note-001");
    assert_eq!(created.relation_type, "references");
    assert_eq!(created.created_by_actor_type, "agent");
    assert_eq!(created.metadata_json, "{\"rank\":1}");

    assert_eq!(
        get_entity_link(&connection, "link-task-note-001")
            .expect("read created link")
            .expect("created link exists"),
        created
    );

    let source_links = list_entity_links_by_source(
        &connection,
        EntityLinkListFilter {
            entity_type: "task",
            entity_id: "task-001",
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list by source");
    assert_eq!(source_links, vec![created.clone()]);

    let target_links = list_entity_links_by_target(
        &connection,
        EntityLinkListFilter {
            entity_type: "note",
            entity_id: "note-001",
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list by target");
    assert_eq!(target_links, vec![created]);
}

#[test]
fn entity_link_service_is_idempotent_but_rejects_id_collision_for_different_tuple() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let first = create_entity_link(
        &connection,
        EntityLinkCreateRequest {
            id: "link-idempotent-001",
            source_type: "task",
            source_id: "task-001",
            target_type: "note",
            target_id: "note-001",
            relation_type: "references",
            created_by_actor_type: "agent",
            metadata_json: "{}",
        },
    )
    .expect("create original link");

    let duplicate = create_entity_link(
        &connection,
        EntityLinkCreateRequest {
            id: "link-idempotent-002",
            source_type: "task",
            source_id: "task-001",
            target_type: "note",
            target_id: "note-001",
            relation_type: "references",
            created_by_actor_type: "agent",
            metadata_json: "{\"ignored\":true}",
        },
    )
    .expect("duplicate logical tuple returns existing row");
    assert_eq!(duplicate, first);

    let id_collision = create_entity_link(
        &connection,
        EntityLinkCreateRequest {
            id: "link-idempotent-001",
            source_type: "task",
            source_id: "task-002",
            target_type: "note",
            target_id: "note-002",
            relation_type: "references",
            created_by_actor_type: "agent",
            metadata_json: "{}",
        },
    )
    .expect_err("same id with different logical tuple fails");
    assert!(matches!(id_collision, RepositoryError::Constraint { .. }));
}

#[test]
fn entity_link_service_validates_allowed_entity_types_and_required_fields_before_persistence() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    for entity_type in [
        "task",
        "note",
        "product",
        "file",
        "repo",
        "run",
        "email",
        "event",
        "browser_capture",
    ] {
        create_entity_link(
            &connection,
            EntityLinkCreateRequest {
                id: &format!("link-{entity_type}"),
                source_type: entity_type,
                source_id: &format!("{entity_type}-source"),
                target_type: "task",
                target_id: "task-target",
                relation_type: "related_to",
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )
        .unwrap_or_else(|error| panic!("{entity_type} should be allowed: {error:?}"));
    }

    for (field_name, id, source_type, source_id, target_type, target_id, relation_type) in [
        (
            "source_type",
            "bad-source-type",
            "unknown",
            "source",
            "task",
            "target",
            "relates",
        ),
        (
            "source_type",
            "empty-source-type",
            "",
            "source",
            "task",
            "target",
            "relates",
        ),
        (
            "target_type",
            "bad-target-type",
            "task",
            "source",
            "unknown",
            "target",
            "relates",
        ),
        (
            "target_type",
            "empty-target-type",
            "task",
            "source",
            "",
            "target",
            "relates",
        ),
        (
            "source_id",
            "empty-source-id",
            "task",
            "",
            "note",
            "target",
            "relates",
        ),
        (
            "target_id",
            "empty-target-id",
            "task",
            "source",
            "note",
            "",
            "relates",
        ),
        (
            "relation_type",
            "empty-relation",
            "task",
            "source",
            "note",
            "target",
            "",
        ),
    ] {
        let error = match create_entity_link(
            &connection,
            EntityLinkCreateRequest {
                id,
                source_type,
                source_id,
                target_type,
                target_id,
                relation_type,
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        ) {
            Ok(_) => panic!("{field_name} should be rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            RepositoryError::Constraint {
                entity: "entity_links",
                ..
            }
        ));
    }

    let persisted_count: i64 = connection
        .query_row("select count(*) from entity_links", [], |row| row.get(0))
        .expect("count entity links");
    assert_eq!(persisted_count, 9);
}

#[test]
fn entity_link_service_validates_and_redacts_metadata_before_persistence() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let link = create_entity_link(
            &connection,
            EntityLinkCreateRequest {
                id: "link-redacted-metadata",
                source_type: "task",
                source_id: "task-001",
                target_type: "note",
                target_id: "note-001",
                relation_type: "references",
                created_by_actor_type: "agent",
                metadata_json: "{\"api_key\":\"raw-secret\",\"nested\":{\"password\":\"pw\"},\"safe\":\"visible\"}",
            },
        )
        .expect("create redacted metadata link");
    let persisted_metadata: Value =
        serde_json::from_str(&link.metadata_json).expect("metadata remains valid JSON");
    assert_eq!(persisted_metadata["api_key"], "[REDACTED]");
    assert_eq!(persisted_metadata["nested"]["password"], "[REDACTED]");
    assert_eq!(persisted_metadata["safe"], "visible");

    let invalid_error = create_entity_link(
        &connection,
        EntityLinkCreateRequest {
            id: "link-invalid-metadata",
            source_type: "task",
            source_id: "task-002",
            target_type: "note",
            target_id: "note-002",
            relation_type: "references",
            created_by_actor_type: "agent",
            metadata_json: "{not json}",
        },
    )
    .expect_err("invalid metadata is rejected");
    assert!(matches!(
        invalid_error,
        RepositoryError::InvalidJson {
            field: "metadata_json",
            ..
        }
    ));
    assert!(get_entity_link(&connection, "link-invalid-metadata")
        .expect("read invalid id")
        .is_none());
}

#[test]
fn entity_link_service_lists_directionally_with_deterministic_filtering() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    for (id, source_type, source_id, target_type, target_id, relation_type) in [
        ("link-3", "task", "task-001", "note", "note-c", "mentions"),
        ("link-1", "task", "task-001", "file", "file-a", "attaches"),
        ("link-2", "task", "task-001", "note", "note-b", "references"),
        (
            "link-4",
            "email",
            "email-001",
            "note",
            "note-b",
            "references",
        ),
    ] {
        create_entity_link(
            &connection,
            EntityLinkCreateRequest {
                id,
                source_type,
                source_id,
                target_type,
                target_id,
                relation_type,
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )
        .expect("create list fixture link");
    }

    let source_links = list_entity_links_by_source(
        &connection,
        EntityLinkListFilter {
            entity_type: "task",
            entity_id: "task-001",
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list task source links");
    assert_eq!(
        source_links
            .iter()
            .map(|link| link.id.as_str())
            .collect::<Vec<_>>(),
        vec!["link-1", "link-3", "link-2"]
    );

    let filtered_source_links = list_entity_links_by_source(
        &connection,
        EntityLinkListFilter {
            entity_type: "task",
            entity_id: "task-001",
            relation_type: Some("references"),
            counterpart_type: Some("note"),
        },
    )
    .expect("list filtered task source links");
    assert_eq!(
        filtered_source_links
            .iter()
            .map(|link| link.id.as_str())
            .collect::<Vec<_>>(),
        vec!["link-2"]
    );

    let target_links = list_entity_links_by_target(
        &connection,
        EntityLinkListFilter {
            entity_type: "note",
            entity_id: "note-b",
            relation_type: Some("references"),
            counterpart_type: None,
        },
    )
    .expect("list target links");
    assert_eq!(
        target_links
            .iter()
            .map(|link| link.id.as_str())
            .collect::<Vec<_>>(),
        vec!["link-4", "link-2"]
    );
}

#[test]
fn entity_link_list_filter_rejects_invalid_or_empty_filter_fields() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    for (field, filter) in [
        (
            "entity_type",
            EntityLinkListFilter {
                entity_type: "calendar_event",
                entity_id: "task-001",
                relation_type: None,
                counterpart_type: None,
            },
        ),
        (
            "entity_id",
            EntityLinkListFilter {
                entity_type: "task",
                entity_id: "   ",
                relation_type: None,
                counterpart_type: None,
            },
        ),
        (
            "relation_type",
            EntityLinkListFilter {
                entity_type: "task",
                entity_id: "task-001",
                relation_type: Some(""),
                counterpart_type: None,
            },
        ),
        (
            "counterpart_type",
            EntityLinkListFilter {
                entity_type: "task",
                entity_id: "task-001",
                relation_type: None,
                counterpart_type: Some("calendar_event"),
            },
        ),
    ] {
        let error = list_entity_links_by_source(&connection, filter)
            .expect_err("invalid entity-link filter should be rejected before query");
        assert!(
            matches!(error, RepositoryError::Constraint { entity, ref message } if entity == "entity_links" && message.contains(field)),
            "expected {field} constraint, got {error:?}"
        );
    }
}

fn assert_table_has_columns(connection: &Connection, table: &str, expected_columns: &[&str]) {
    let columns = table_columns(connection, table).expect("read table columns");
    for expected_column in expected_columns {
        assert!(
            columns.contains(*expected_column),
            "missing {table}.{expected_column}; columns were {columns:?}"
        );
    }
}

fn assert_index_exists(connection: &Connection, table: &str, expected_index: &str) {
    let mut statement = connection
        .prepare(&format!("pragma index_list({table})"))
        .expect("prepare index list");
    let indexes = statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query index list")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("collect index list");

    assert!(
        indexes.iter().any(|index| index == expected_index),
        "missing index {expected_index} on {table}; indexes were {indexes:?}"
    );
}

#[test]
fn p307_file_service_browses_opens_and_previews_safe_visible_files() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p307-file-service-basic");
    fs::create_dir_all(visible_root.join("Files/docs")).expect("create files dir");
    fs::write(
        visible_root.join("Files/docs/brief.md"),
        "# Brief\n\nThis is a local preview file.",
    )
    .expect("write preview file");
    fs::write(visible_root.join("Files/image.png"), [0_u8, 159, 146, 150])
        .expect("write binary-ish file");

    let root_entries = browse_files_service(&connection, &visible_root, "zoid_visible", "Files")
        .expect("browse Files");
    assert!(root_entries.iter().any(|entry| {
        entry.relative_path == "Files/docs" && entry.file_kind == "folder" && entry.is_directory
    }));
    assert!(root_entries.iter().any(|entry| {
        entry.relative_path == "Files/image.png"
            && entry.file_kind == "image"
            && !entry.preview_available
    }));

    let open = open_file_reference_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/docs/brief.md",
    )
    .expect("open file reference");
    assert_eq!(open.relative_path, "Files/docs/brief.md");
    assert_eq!(open.file_kind, "markdown_note");
    assert!(open.absolute_path.ends_with("Files/docs/brief.md"));

    let preview = preview_file_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/docs/brief.md",
    )
    .expect("preview text file");
    assert_eq!(preview.relative_path, "Files/docs/brief.md");
    assert_eq!(preview.file_kind, "markdown_note");
    assert!(preview.preview_text.contains("local preview file"));
    assert!(!preview.truncated);

    let file_status: String = connection
        .query_row(
            "select status from file_references where root_key = 'zoid_visible' and relative_path = 'Files/docs/brief.md'",
            [],
            |row| row.get(0),
        )
        .expect("read file reference status");
    assert_eq!(file_status, "indexed");
    let preview_index_state: String = connection
        .query_row(
            "select scan_state from knowledge_index_entries where entity_type = 'file' and source_type = 'file_preview' and title = 'brief.md'",
            [],
            |row| row.get(0),
        )
        .expect("read file preview index");
    assert_eq!(preview_index_state, "current");
}

#[test]
fn p307_file_service_rejects_unsafe_binary_and_missing_paths_without_indexing() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p307-file-service-safety");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(visible_root.join("Files/blob.bin"), [0_u8, 1, 2, 3, 4, 5])
        .expect("write binary file");

    let escape = browse_files_service(&connection, &visible_root, "zoid_visible", "../Secrets")
        .expect_err("escape should fail");
    assert!(format!("{escape:?}").contains("relative path"));
    let missing = preview_file_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/missing.md",
    )
    .expect_err("missing preview should fail");
    assert!(matches!(missing, RepositoryError::NotFound { .. }));
    let binary = preview_file_service(&connection, &visible_root, "zoid_visible", "Files/blob.bin")
        .expect_err("binary preview should fail closed");
    assert!(format!("{binary:?}").contains("not previewable"));

    let indexed_count: i64 = connection
        .query_row(
            "select count(*) from knowledge_index_entries where entity_type = 'file'",
            [],
            |row| row.get(0),
        )
        .expect("count file index rows");
    assert_eq!(indexed_count, 0);
}

#[test]
fn p307_file_preview_reads_and_indexes_only_bounded_preview_bytes() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p307-file-service-bounded-preview");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    let large_text = format!("{}TAIL-SHOULD-NOT-BE-INDEXED", "a".repeat(8192));
    fs::write(visible_root.join("Files/large.txt"), large_text).expect("write large text file");

    let preview = preview_file_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/large.txt",
    )
    .expect("preview large text file");
    assert_eq!(preview.preview_text.len(), 4096);
    assert!(preview.truncated);
    assert!(!preview.preview_text.contains("TAIL-SHOULD-NOT-BE-INDEXED"));

    let search_text: String = connection
        .query_row(
            "select search_text from knowledge_index_entries where entity_type = 'file' and source_type = 'file_preview' and title = 'large.txt'",
            [],
            |row| row.get(0),
        )
        .expect("read bounded file preview index");
    assert_eq!(search_text.len(), 4096);
    assert!(!search_text.contains("TAIL-SHOULD-NOT-BE-INDEXED"));
}

#[cfg(unix)]
#[test]
fn p307_file_service_rejects_symlinked_files_and_directories() {
    use std::os::unix::fs::symlink;

    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p307-file-service-symlink");
    let outside_root = temp_home("p307-file-service-outside");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::create_dir_all(outside_root.join("secret-dir")).expect("create outside dir");
    fs::write(outside_root.join("secret.md"), "# Secret").expect("write outside file");
    symlink(
        outside_root.join("secret.md"),
        visible_root.join("Files/secret-link.md"),
    )
    .expect("create file symlink");
    symlink(
        outside_root.join("secret-dir"),
        visible_root.join("Files/secret-dir-link"),
    )
    .expect("create dir symlink");
    fs::create_dir_all(visible_root.join("RealDir")).expect("create real dir");
    fs::write(visible_root.join("RealDir/inside.md"), "# Inside").expect("write inside file");
    symlink(
        visible_root.join("RealDir"),
        visible_root.join("Files/inside-link"),
    )
    .expect("create internal intermediate symlink");

    let file_error = open_file_reference_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/secret-link.md",
    )
    .expect_err("file symlink should be rejected");
    assert!(format!("{file_error:?}").contains("symlink"));

    let browse_error = browse_files_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/secret-dir-link",
    )
    .expect_err("dir symlink should be rejected");
    assert!(format!("{browse_error:?}").contains("symlink"));

    let intermediate_error = preview_file_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/inside-link/inside.md",
    )
    .expect_err("intermediate symlink component should be rejected");
    assert!(format!("{intermediate_error:?}").contains("symlink"));
}

#[test]
fn p308_file_actions_block_without_confirmation_and_preserve_state() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p308-confirmation-block");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(visible_root.join("Files/source.md"), "# Source").expect("write source");
    preview_file_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/source.md",
    )
    .expect("seed source reference and index");
    let before_refs = count_rows(&connection, "select count(*) from file_references");
    let before_index = count_rows(
        &connection,
        "select count(*) from knowledge_index_entries where entity_type = 'file'",
    );

    let error = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::copy("zoid_visible", "Files/source.md", "Files/copy.md"),
        None,
        None,
    )
    .expect_err("copy must be blocked without confirmation");

    assert!(format!("{error:?}").contains("confirmation_required"));
    assert!(visible_root.join("Files/source.md").exists());
    assert!(!visible_root.join("Files/copy.md").exists());
    assert_eq!(
        before_refs,
        count_rows(&connection, "select count(*) from file_references")
    );
    assert_eq!(
        before_index,
        count_rows(
            &connection,
            "select count(*) from knowledge_index_entries where entity_type = 'file'"
        )
    );
}

#[test]
fn p308_file_actions_allow_confirmed_copy_rename_move_and_reject_collisions() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p308-confirmed-actions");
    fs::create_dir_all(visible_root.join("Files/archive")).expect("create files dir");
    fs::write(visible_root.join("Files/source.md"), "# Source").expect("write source");
    fs::write(visible_root.join("Files/existing.md"), "# Existing").expect("write existing");
    let policy = evaluate_action_request(
        &ActionRequest::new(ActionType::File)
            .target("copy file")
            .scope(ActionScope::LocalVisible)
            .consequence(ActionConsequence::LocalWrite),
    );
    let confirmation = ConfirmationDecisionRecord::new_for_test(
        "confirm-copy",
        "move_rename_copy_file",
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );

    let collision = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::copy("zoid_visible", "Files/source.md", "Files/existing.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect_err("destination collision should be rejected");
    assert!(format!("{collision:?}").contains("destination already exists"));

    let copied = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::copy("zoid_visible", "Files/source.md", "Files/copy.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect("confirmed copy should succeed");
    assert_eq!(copied.action, "copy");
    assert_eq!(copied.source_relative_path, "Files/source.md");
    assert_eq!(
        copied.destination_relative_path.as_deref(),
        Some("Files/copy.md")
    );
    assert!(visible_root.join("Files/source.md").exists());
    assert_eq!(
        fs::read_to_string(visible_root.join("Files/copy.md")).expect("read copied file"),
        "# Source"
    );

    let renamed = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::rename("zoid_visible", "Files/copy.md", "renamed.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect("confirmed rename should succeed");
    assert_eq!(
        renamed.destination_relative_path.as_deref(),
        Some("Files/renamed.md")
    );
    assert!(!visible_root.join("Files/copy.md").exists());
    assert!(visible_root.join("Files/renamed.md").exists());

    let moved = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::move_to("zoid_visible", "Files/renamed.md", "Files/archive/moved.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect("confirmed move should succeed");
    assert_eq!(
        moved.destination_relative_path.as_deref(),
        Some("Files/archive/moved.md")
    );
    assert!(!visible_root.join("Files/renamed.md").exists());
    assert!(visible_root.join("Files/archive/moved.md").exists());

    let copied_status: String = connection
        .query_row(
            "select status from file_references where root_key = 'zoid_visible' and relative_path = 'Files/archive/moved.md'",
            [],
            |row| row.get(0),
        )
        .expect("moved destination reference indexed");
    assert_eq!(copied_status, "indexed");
}

#[cfg(unix)]
#[test]
fn p308_file_actions_reject_path_escape_and_symlink_components() {
    use std::os::unix::fs::symlink;

    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p308-path-safety");
    let outside_root = temp_home("p308-path-outside");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::create_dir_all(outside_root.join("secret-dir")).expect("create outside dir");
    fs::write(visible_root.join("Files/source.md"), "# Source").expect("write source");
    symlink(
        outside_root.join("secret-dir"),
        visible_root.join("Files/outside-link"),
    )
    .expect("create symlinked parent");
    let policy = evaluate_action_request(
        &ActionRequest::new(ActionType::File)
            .target("move file")
            .scope(ActionScope::LocalVisible)
            .consequence(ActionConsequence::LocalWrite),
    );
    let confirmation = ConfirmationDecisionRecord::new_for_test(
        "confirm-move",
        "move_rename_copy_file",
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );

    let escape = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::move_to("zoid_visible", "Files/source.md", "../outside.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect_err("destination escape should be rejected");
    assert!(format!("{escape:?}").contains("relative path"));
    assert!(visible_root.join("Files/source.md").exists());

    let symlinked_destination = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::copy(
            "zoid_visible",
            "Files/source.md",
            "Files/outside-link/copy.md",
        ),
        Some(&policy),
        Some(&confirmation),
    )
    .expect_err("symlinked destination parent should be rejected");
    assert!(format!("{symlinked_destination:?}").contains("symlink"));
    assert!(!outside_root.join("secret-dir/copy.md").exists());

    symlink(
        outside_root.join("secret-dir/broken-copy.md"),
        visible_root.join("Files/final-link.md"),
    )
    .expect("create broken final symlink");
    let final_symlink_destination = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::copy("zoid_visible", "Files/source.md", "Files/final-link.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect_err("final destination symlink should be rejected even when broken");
    assert!(
        format!("{final_symlink_destination:?}").contains("symlink")
            || format!("{final_symlink_destination:?}").contains("destination already exists")
    );
    assert!(!outside_root.join("secret-dir/broken-copy.md").exists());
}

#[test]
fn p308_file_actions_invalid_preview_bytes_do_not_turn_safe_copy_into_partial_error() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p308-invalid-preview-copy");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(visible_root.join("Files/source.md"), [0xff_u8, 0xfe, b'A'])
        .expect("write invalid utf8 markdown");
    let policy = evaluate_action_request(
        &ActionRequest::new(ActionType::File)
            .target("copy file")
            .scope(ActionScope::LocalVisible)
            .consequence(ActionConsequence::LocalWrite),
    );
    let confirmation = ConfirmationDecisionRecord::new_for_test(
        "confirm-invalid-copy",
        "move_rename_copy_file",
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );

    let copied = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::copy("zoid_visible", "Files/source.md", "Files/copy.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect("invalid preview bytes should not make a successful copy report failure");

    assert_eq!(
        copied.destination_relative_path.as_deref(),
        Some("Files/copy.md")
    );
    assert_eq!(
        fs::read(visible_root.join("Files/copy.md")).expect("read copied bytes"),
        vec![0xff_u8, 0xfe, b'A']
    );
    let copied_status: String = connection
        .query_row(
            "select status from file_references where root_key = 'zoid_visible' and relative_path = 'Files/copy.md'",
            [],
            |row| row.get(0),
        )
        .expect("copied file reference indexed despite skipped preview text");
    assert_eq!(copied_status, "indexed");
    let copied_index_count = count_rows(
        &connection,
        "select count(*) from knowledge_index_entries where entity_type = 'file' and title = 'copy.md'",
    );
    assert_eq!(copied_index_count, 0);
}

#[test]
fn p308_file_trash_is_non_destructive_and_marks_old_index_stale() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p308-trash");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(visible_root.join("Files/source.md"), "# Trash Me").expect("write source");
    preview_file_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/source.md",
    )
    .expect("seed source reference and index");
    let policy = evaluate_action_request(
        &ActionRequest::new(ActionType::Delete)
            .target("trash file")
            .scope(ActionScope::LocalVisible)
            .consequence(ActionConsequence::Destructive)
            .destructive(true),
    );
    let confirmation = ConfirmationDecisionRecord::new_for_test(
        "confirm-trash",
        "delete_trash_files",
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );

    let trashed = perform_file_action_service(
        &connection,
        &visible_root,
        FileActionInput::trash("zoid_visible", "Files/source.md"),
        Some(&policy),
        Some(&confirmation),
    )
    .expect("confirmed trash should succeed");

    let trash_path = trashed
        .destination_relative_path
        .as_deref()
        .expect("trash destination path");
    assert!(trash_path.starts_with("Trash/Files/source"));
    assert!(!visible_root.join("Files/source.md").exists());
    assert_eq!(
        fs::read_to_string(visible_root.join(trash_path)).expect("read trashed file"),
        "# Trash Me"
    );

    let old_status: String = connection
        .query_row(
            "select status from file_references where root_key = 'zoid_visible' and relative_path = 'Files/source.md'",
            [],
            |row| row.get(0),
        )
        .expect("read old file status");
    assert_eq!(old_status, "trashed");
    let trash_status: String = connection
        .query_row(
            "select status from file_references where root_key = 'zoid_visible' and relative_path = ?1",
            params![trash_path],
            |row| row.get(0),
        )
        .expect("read trash file status");
    assert_eq!(trash_status, "indexed");
    let old_index_state: String = connection
        .query_row(
            "select scan_state from knowledge_index_entries where entity_type = 'file' and entity_id = ?1",
            params![file_reference_entity_id("zoid_visible", "Files/source.md")],
            |row| row.get(0),
        )
        .expect("old preview index marked stale");
    assert_eq!(old_index_state, "stale");
}

#[test]
fn p309_note_links_to_tasks_products_and_runs_with_directional_queries() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p309-note-links");
    let note = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Linked Note".to_string(),
            body_markdown: "Capture context".to_string(),
            relative_path: Some("Notes/linked-note.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create note");
    let task = p204_task(&connection, "Linked task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            session_id: session.id.clone(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run");

    let task_link = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "task",
            target_id: &task.id,
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: r#"{"safe":"visible"}"#,
        },
    )
    .expect("link note to task");
    let duplicate = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "task",
            target_id: &task.id,
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: r#"{"ignored":"duplicate"}"#,
        },
    )
    .expect("idempotent duplicate note task link");
    assert_eq!(duplicate.id, task_link.id);

    create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "product",
            target_id: "product-alpha",
            relation_type: "documents",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("link note to future product id");
    create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "run",
            target_id: &run.id,
            relation_type: "evidence_for",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("link note to run");

    let note_links = list_content_entity_links_by_source(
        &connection,
        ContentEntityLinkListFilter {
            entity_type: "note",
            entity_id: &note.id,
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list note links");
    assert_eq!(note_links.len(), 3);
    assert_eq!(
        note_links
            .iter()
            .map(|link| link.target_type.as_str())
            .collect::<Vec<_>>(),
        vec!["product", "run", "task"]
    );
    let task_sources = list_content_entity_links_by_target(
        &connection,
        ContentEntityLinkListFilter {
            entity_type: "task",
            entity_id: &task.id,
            relation_type: Some("supports"),
            counterpart_type: Some("note"),
        },
    )
    .expect("list task note links");
    assert_eq!(task_sources, vec![task_link]);
}

#[test]
fn p313_content_entity_link_command_lists_note_source_links_without_fake_fallbacks() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p313-content-link-command");
    let note = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput::new("Panel Note", "Real link source"),
    )
    .expect("create note");
    let task = p204_task(&connection, "Panel task");

    create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "task",
            target_id: &task.id,
            relation_type: "mentions",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("create persisted note link");

    let links = list_content_entity_links_by_source_command_with_connection(
        &connection,
        ContentEntityLinkCommandListRequest {
            entity_type: "note".to_string(),
            entity_id: note.id.clone(),
            relation_type: Some("mentions".to_string()),
            counterpart_type: Some("task".to_string()),
        },
    )
    .expect("list real source links through bridge helper");

    assert_eq!(links.len(), 1);
    assert_eq!(links[0].source_type, "note");
    assert_eq!(links[0].source_id, note.id);
    assert_eq!(links[0].target_type, "task");
    assert_eq!(links[0].target_id, task.id);
    assert_eq!(links[0].relation_type, "mentions");
}

#[test]
fn p309_file_links_to_tasks_products_and_runs_after_file_reference_exists() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p309-file-links");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(
        visible_root.join("Files/evidence.md"),
        "# Evidence\n\nUseful",
    )
    .expect("write evidence file");
    open_file_reference_service(
        &connection,
        &visible_root,
        "zoid_visible",
        "Files/evidence.md",
    )
    .expect("open file to create reference");
    let file_id = file_reference_entity_id("zoid_visible", "Files/evidence.md");
    let task = p204_task(&connection, "File linked task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            session_id: session.id.clone(),
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run");

    for (target_type, target_id, relation_type) in [
        ("task", task.id.as_str(), "attachment_for"),
        ("product", "product-beta", "asset_for"),
        ("run", run.id.as_str(), "artifact_for"),
    ] {
        create_content_entity_link_service(
            &connection,
            ContentEntityLinkCreateRequest {
                source_type: "file",
                source_id: &file_id,
                target_type,
                target_id,
                relation_type,
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )
        .expect("link file source");
    }

    let file_links = list_content_entity_links_by_source(
        &connection,
        ContentEntityLinkListFilter {
            entity_type: "file",
            entity_id: &file_id,
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list file links");
    assert_eq!(file_links.len(), 3);
    assert!(file_links.iter().all(|link| link.source_type == "file"));
    assert_eq!(
        count_rows(
            &connection,
            "select count(*) from entity_links where source_type = 'file' and target_type in ('task', 'product', 'run')"
        ),
        3
    );
}

#[test]
fn p309_content_links_reject_invalid_direction_missing_entities_and_secret_metadata() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p309-rejects");
    let note = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput::new("Reject Note", "Body"),
    )
    .expect("create note");
    let task = p204_task(&connection, "Reject task");

    let invalid_direction = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "task",
            source_id: &task.id,
            target_type: "note",
            target_id: &note.id,
            relation_type: "backlink",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err("task to note is not a P3.09 content link");
    assert!(format!("{invalid_direction:?}").contains("unsupported content source_type"));

    let missing_note = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: "note_missing_p309",
            target_type: "task",
            target_id: &task.id,
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err("missing note rejected");
    assert!(matches!(
        missing_note,
        RepositoryError::NotFound {
            entity: "notes",
            ..
        }
    ));

    let missing_file = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "file",
            source_id: "file_ref_missing_p309",
            target_type: "task",
            target_id: &task.id,
            relation_type: "attachment_for",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err("missing file rejected");
    assert!(matches!(
        missing_file,
        RepositoryError::NotFound {
            entity: "file_references",
            ..
        }
    ));

    let missing_task = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "task",
            target_id: "task_missing_p309",
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect_err("missing task rejected");
    assert!(matches!(
        missing_task,
        RepositoryError::NotFound {
            entity: "tasks",
            ..
        }
    ));

    let secret_metadata = create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "task",
            target_id: &task.id,
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: r#"{"api_key":"dummy-secret","safe":"visible"}"#,
        },
    )
    .expect("metadata is redacted through generic entity-link service");
    assert!(secret_metadata.metadata_json.contains("[REDACTED]"));
    assert!(!secret_metadata.metadata_json.contains("dummy-secret"));
}

#[test]
fn p309_content_link_source_queries_survive_later_note_and_file_state_changes() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p309-state-listing");
    let note = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput::new("Stateful Note", "Body"),
    )
    .expect("create note");
    fs::create_dir_all(visible_root.join("Files")).expect("create files dir");
    fs::write(visible_root.join("Files/state.md"), "# State").expect("write file");
    open_file_reference_service(&connection, &visible_root, "zoid_visible", "Files/state.md")
        .expect("open file reference");
    let file_id = file_reference_entity_id("zoid_visible", "Files/state.md");
    let task = p204_task(&connection, "State task");

    create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "note",
            source_id: &note.id,
            target_type: "task",
            target_id: &task.id,
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("link note before state change");
    create_content_entity_link_service(
        &connection,
        ContentEntityLinkCreateRequest {
            source_type: "file",
            source_id: &file_id,
            target_type: "task",
            target_id: &task.id,
            relation_type: "attachment_for",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("link file before state change");

    connection
        .execute(
            "update notes set status = 'trashed', updated_at = current_timestamp where id = ?1",
            params![note.id],
        )
        .expect("mark note trashed");
    connection
        .execute(
            "update file_references set status = 'trashed', updated_at = current_timestamp where id = ?1",
            params![file_id],
        )
        .expect("mark file trashed");

    let note_links = list_content_entity_links_by_source(
        &connection,
        ContentEntityLinkListFilter {
            entity_type: "note",
            entity_id: &note.id,
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("trashed note links remain queryable");
    let file_links = list_content_entity_links_by_source(
        &connection,
        ContentEntityLinkListFilter {
            entity_type: "file",
            entity_id: &file_id,
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("trashed file links remain queryable");
    assert_eq!(note_links.len(), 1);
    assert_eq!(file_links.len(), 1);
}

#[test]
fn p306_note_scanner_detects_manual_rename_without_mutating_original_identity() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p306-manual-rename");
    fs::create_dir_all(visible_root.join("Notes/archive")).expect("create notes");
    let created = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Rename Me".to_string(),
            body_markdown: "Keep identity".to_string(),
            relative_path: Some("Notes/rename-me.md".to_string()),
            metadata_json: r#"{"tag":"client","keep":true}"#.to_string(),
        },
    )
    .expect("create note");
    fs::rename(
        visible_root.join("Notes/rename-me.md"),
        visible_root.join("Notes/archive/renamed.md"),
    )
    .expect("external rename");

    let scan = scan_markdown_notes_service(&connection, &visible_root).expect("scan rename");
    assert_eq!(scan.conflicted_notes, 1);
    let conflict = list_note_conflicts_service(&connection)
        .expect("list conflicts")
        .into_iter()
        .find(|record| record.id == created.id)
        .expect("manual rename conflict record");
    assert_eq!(conflict.conflict_state, "manual_rename");
    assert_eq!(conflict.relative_path, "Notes/rename-me.md");
    assert_eq!(
        conflict.detected_relative_path.as_deref(),
        Some("Notes/archive/renamed.md")
    );

    let row_path: String = connection
        .query_row(
            "select relative_path from notes where id = ?1",
            params![created.id],
            |row| row.get(0),
        )
        .expect("read original path");
    assert_eq!(row_path, "Notes/rename-me.md");

    let accepted = accept_note_conflict_service(&connection, &visible_root, &created.id)
        .expect("accept manual rename");
    assert_eq!(accepted.status, "active");
    assert_eq!(accepted.conflict_state, "none");
    assert_eq!(accepted.relative_path, "Notes/archive/renamed.md");
    assert!(visible_root.join("Notes/archive/renamed.md").exists());
    assert!(accepted.metadata_json.contains("\"tag\":\"client\""));
    assert!(accepted.metadata_json.contains("\"keep\":true"));
    assert!(!accepted.metadata_json.contains("detected_relative_path"));
}

#[test]
fn p306_note_scanner_detects_external_edit_and_accepts_without_losing_file_content() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p306-external-edit");
    fs::create_dir_all(visible_root.join("Notes")).expect("create notes");
    let created = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "External Edit".to_string(),
            body_markdown: "Original body".to_string(),
            relative_path: Some("Notes/external-edit.md".to_string()),
            metadata_json: r#"{"origin":"manual","keep":true}"#.to_string(),
        },
    )
    .expect("create note");
    let edited_markdown = created
        .markdown
        .replace("Original body", "Externally edited body");
    fs::write(
        visible_root.join("Notes/external-edit.md"),
        &edited_markdown,
    )
    .expect("external edit");

    let scan = scan_markdown_notes_service(&connection, &visible_root).expect("scan external edit");
    assert_eq!(scan.conflicted_notes, 1);
    let conflict = list_note_conflicts_service(&connection)
        .expect("list conflicts")
        .into_iter()
        .find(|record| record.id == created.id)
        .expect("external edit conflict record");
    assert_eq!(conflict.conflict_state, "external_edit");
    assert_eq!(conflict.relative_path, "Notes/external-edit.md");
    assert_ne!(conflict.disk_digest, conflict.stored_digest);

    let stored_digest_before: String = connection
        .query_row(
            "select body_digest from notes where id = ?1",
            params![created.id],
            |row| row.get(0),
        )
        .expect("read stored digest before accept");
    assert_eq!(stored_digest_before, created.body_digest);

    let accepted = accept_note_conflict_service(&connection, &visible_root, &created.id)
        .expect("accept external edit");
    assert_eq!(accepted.status, "active");
    assert_eq!(accepted.conflict_state, "none");
    assert!(accepted.markdown.contains("Externally edited body"));
    assert_ne!(accepted.body_digest, created.body_digest);
    assert!(accepted.metadata_json.contains("\"origin\":\"manual\""));
    assert!(accepted.metadata_json.contains("\"keep\":true"));
    assert!(!accepted.metadata_json.contains("stored_digest"));
    assert!(!accepted.metadata_json.contains("disk_digest"));
}

#[test]
fn p306_duplicate_id_acceptance_is_rejected_without_mutating_files_or_metadata() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p306-duplicate-reject");
    fs::create_dir_all(visible_root.join("Notes")).expect("create notes");
    let first_markdown = "---\nzoid_id: \"note_duplicate_p306\"\ntitle: \"Original\"\nslug: \"original\"\n---\n\n# Original\n\nOne";
    let duplicate_markdown = "---\nzoid_id: \"note_duplicate_p306\"\ntitle: \"Duplicate\"\nslug: \"duplicate\"\n---\n\n# Duplicate\n\nTwo";
    fs::write(visible_root.join("Notes/a-original.md"), first_markdown).expect("write original");
    fs::write(
        visible_root.join("Notes/z-duplicate.md"),
        duplicate_markdown,
    )
    .expect("write duplicate");

    let scan = scan_markdown_notes_service(&connection, &visible_root).expect("scan duplicate");
    assert_eq!(scan.conflicted_notes, 1);
    let before_metadata: String = connection
        .query_row(
            "select metadata_json from notes where id = 'note_duplicate_p306'",
            [],
            |row| row.get(0),
        )
        .expect("read metadata before reject");

    let error = accept_note_conflict_service(&connection, &visible_root, "note_duplicate_p306")
        .expect_err("duplicate accept should be rejected");
    assert!(format!("{error:?}").contains("duplicate_id"));
    let after: (String, String) = connection
        .query_row(
            "select conflict_state, metadata_json from notes where id = 'note_duplicate_p306'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read duplicate after reject");
    assert_eq!(after.0, "duplicate_id");
    assert_eq!(after.1, before_metadata);
    assert_eq!(
        fs::read_to_string(visible_root.join("Notes/z-duplicate.md")).expect("read duplicate file"),
        duplicate_markdown
    );
}

#[test]
fn p305_note_scanner_writes_missing_frontmatter_indexes_and_marks_missing_files() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p305-note-scanner-indexes");
    fs::create_dir_all(visible_root.join("Notes/nested")).expect("create nested notes");
    fs::write(
        visible_root.join("Notes/nested/scanned.md"),
        "# Scanned Note\n\nScanner body",
    )
    .expect("write markdown without frontmatter");
    fs::write(visible_root.join("Notes/ignore.txt"), "not markdown").expect("write txt");
    fs::create_dir_all(visible_root.join("Notes/.Trash")).expect("create trash dir");
    fs::write(visible_root.join("Notes/.Trash/old.md"), "# Old Trash").expect("write trash file");

    let first = scan_markdown_notes_service(&connection, &visible_root).expect("scan notes");
    assert_eq!(first.scanned_files, 1);
    assert_eq!(first.indexed_notes, 1);
    assert_eq!(first.frontmatter_written, 1);
    assert_eq!(first.conflicted_notes, 0);
    assert_eq!(first.missing_notes_marked, 0);

    let rewritten = fs::read_to_string(visible_root.join("Notes/nested/scanned.md"))
        .expect("read rewritten markdown");
    assert!(rewritten.starts_with("---\n"));
    assert!(rewritten.contains("zoid_id: \"note_"));
    assert!(rewritten.contains("title: \"Scanned Note\""));
    assert!(rewritten.contains("# Scanned Note"));

    let note_id: String = connection
        .query_row(
            "select id from notes where relative_path = 'Notes/nested/scanned.md' and status = 'active'",
            [],
            |row| row.get(0),
        )
        .expect("read scanned note id");
    let index_state: String = connection
        .query_row(
            "select scan_state from knowledge_index_entries where entity_type = 'note' and entity_id = ?1 and source_type = 'markdown_frontmatter'",
            params![note_id],
            |row| row.get(0),
        )
        .expect("read scanned index state");
    assert_eq!(index_state, "current");

    fs::remove_file(visible_root.join("Notes/nested/scanned.md")).expect("remove scanned file");
    let second =
        scan_markdown_notes_service(&connection, &visible_root).expect("rescan missing note");
    assert_eq!(second.scanned_files, 0);
    assert_eq!(second.indexed_notes, 0);
    assert_eq!(second.missing_notes_marked, 1);
    let missing_state: String = connection
        .query_row(
            "select status from notes where relative_path = 'Notes/nested/scanned.md'",
            [],
            |row| row.get(0),
        )
        .expect("read missing note state");
    assert_eq!(missing_state, "conflicted");
    let missing_index_state: String = connection
        .query_row(
            "select scan_state from knowledge_index_entries where entity_type = 'note' and entity_id = ?1 and source_type = 'markdown_frontmatter'",
            params![note_id],
            |row| row.get(0),
        )
        .expect("read missing index state");
    assert_eq!(missing_index_state, "missing");
}

#[test]
fn p305_note_scanner_preserves_existing_ids_and_flags_duplicates_non_destructively() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p305-note-scanner-duplicates");
    fs::create_dir_all(visible_root.join("Notes")).expect("create notes");
    let first_markdown = "---\nzoid_id: \"note_duplicate_p305\"\ntitle: \"Original\"\nslug: \"original\"\n---\n\n# Original\n\nOne";
    let duplicate_markdown = "---\nzoid_id: \"note_duplicate_p305\"\ntitle: \"Duplicate\"\nslug: \"duplicate\"\n---\n\n# Duplicate\n\nTwo";
    fs::write(visible_root.join("Notes/a-original.md"), first_markdown).expect("write original");
    fs::write(
        visible_root.join("Notes/z-duplicate.md"),
        duplicate_markdown,
    )
    .expect("write duplicate");

    let result =
        scan_markdown_notes_service(&connection, &visible_root).expect("scan duplicate notes");
    assert_eq!(result.scanned_files, 2);
    assert_eq!(result.indexed_notes, 1);
    assert_eq!(result.conflicted_notes, 1);
    assert_eq!(result.frontmatter_written, 0);

    let stored: (String, String) = connection
        .query_row(
            "select relative_path, conflict_state from notes where id = 'note_duplicate_p305'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read duplicate note row");
    assert_eq!(stored.0, "Notes/a-original.md");
    assert_eq!(stored.1, "duplicate_id");
    assert_eq!(
        fs::read_to_string(visible_root.join("Notes/z-duplicate.md")).expect("read duplicate file"),
        duplicate_markdown,
        "duplicate file must not be rewritten or destroyed"
    );
}

#[test]
fn p305_note_scanner_rewrites_existing_yaml_missing_zoid_id_preserving_custom_keys() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p305-note-scanner-existing-yaml");
    fs::create_dir_all(visible_root.join("Notes")).expect("create notes");
    fs::write(
        visible_root.join("Notes/existing-yaml.md"),
        "---\ntitle: Existing YAML Title\ncustom: keep-me\n---\n\n# Body Heading\n\nBody text",
    )
    .expect("write existing yaml note");

    let result =
        scan_markdown_notes_service(&connection, &visible_root).expect("scan existing yaml");
    assert_eq!(result.scanned_files, 1);
    assert_eq!(result.indexed_notes, 1);
    assert_eq!(result.frontmatter_written, 1);

    let rewritten = fs::read_to_string(visible_root.join("Notes/existing-yaml.md"))
        .expect("read rewritten yaml note");
    assert!(rewritten.contains("zoid_id: \"note_"));
    assert!(rewritten.contains("title: \"Existing YAML Title\""));
    assert!(rewritten.contains("slug: \"existing-yaml-title\""));
    assert!(rewritten.contains("custom: keep-me"));
    assert!(rewritten.contains("# Body Heading"));

    let stored_id: String = connection
        .query_row(
            "select id from notes where relative_path = 'Notes/existing-yaml.md' and status = 'active'",
            [],
            |row| row.get(0),
        )
        .expect("read indexed id");
    assert!(rewritten.contains(&stored_id));
}

#[cfg(unix)]
#[test]
fn p305_note_scanner_write_failure_does_not_leave_active_index_row() {
    use std::os::unix::fs::PermissionsExt;

    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p305-note-scanner-write-failure");
    let notes_dir = visible_root.join("Notes");
    let note_path = notes_dir.join("locked.md");
    fs::create_dir_all(&notes_dir).expect("create notes");
    fs::write(&note_path, "# Locked\n\nBody").expect("write locked note");
    let original_dir_mode = fs::metadata(&notes_dir)
        .expect("notes metadata")
        .permissions()
        .mode();
    let original_file_mode = fs::metadata(&note_path)
        .expect("note metadata")
        .permissions()
        .mode();
    fs::set_permissions(&notes_dir, fs::Permissions::from_mode(0o555)).expect("lock notes dir");
    fs::set_permissions(&note_path, fs::Permissions::from_mode(0o444)).expect("lock note file");

    let scan_error = scan_markdown_notes_service(&connection, &visible_root)
        .expect_err("scanner write failure should fail closed");

    fs::set_permissions(&notes_dir, fs::Permissions::from_mode(original_dir_mode))
        .expect("restore notes dir");
    fs::set_permissions(&note_path, fs::Permissions::from_mode(original_file_mode))
        .expect("restore note file");

    assert!(format!("{scan_error:?}").len() > 0);
    let active_count: i64 = connection
        .query_row(
            "select count(*) from notes where relative_path = 'Notes/locked.md' and status = 'active'",
            [],
            |row| row.get(0),
        )
        .expect("count active locked note rows");
    assert_eq!(active_count, 0);
    let index_count: i64 = connection
        .query_row(
            "select count(*) from knowledge_index_entries where entity_type = 'note' and source_type = 'markdown_frontmatter' and scan_state = 'current'",
            [],
            |row| row.get(0),
        )
        .expect("count current note indexes");
    assert_eq!(index_count, 0);
}

#[test]
fn p304_note_service_create_edit_persists_file_db_index_and_events() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p304-note-service-create-edit");
    fs::create_dir_all(visible_root.join("Notes")).expect("create Notes directory");

    let created = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Today Note".to_string(),
            body_markdown: "Initial body".to_string(),
            relative_path: Some("Notes/today-note.md".to_string()),
            metadata_json: r#"{"source":"test"}"#.to_string(),
        },
    )
    .expect("create note");

    assert_eq!(created.title, "Today Note");
    assert_eq!(created.slug, "today-note");
    assert_eq!(created.relative_path, "Notes/today-note.md");
    assert_eq!(created.status, "active");
    assert!(created
        .markdown
        .contains(&format!("zoid_id: \"{}\"", created.id)));
    assert!(visible_root.join("Notes/today-note.md").exists());

    let edited_markdown = format!(
        "---\nzoid_id: \"{}\"\ntitle: \"Today Updated\"\nslug: \"today-updated\"\n---\n\n# Today Updated\n\nEdited body",
        created.id
    );
    let edited =
        edit_markdown_note_service(&connection, &visible_root, &created.id, &edited_markdown)
            .expect("edit note");
    assert_eq!(edited.id, created.id);
    assert_eq!(edited.relative_path, created.relative_path);
    assert_eq!(edited.title, "Today Updated");
    assert_eq!(edited.slug, "today-updated");
    assert!(edited
        .markdown
        .contains(&format!("zoid_id: \"{}\"", created.id)));
    assert!(edited.markdown.contains("Edited body"));

    let indexed_title: String = connection
        .query_row(
            "select title from knowledge_index_entries where entity_type = 'note' and entity_id = ?1 and source_type = 'markdown_frontmatter'",
            params![created.id],
            |row| row.get(0),
        )
        .expect("read note index title");
    assert_eq!(indexed_title, "Today Updated");

    let note_event_count: i64 = connection
        .query_row(
            "select count(*) from events where type in ('note.created', 'note.updated') and source = 'note_service'",
            [],
            |row| row.get(0),
        )
        .expect("count note events");
    assert_eq!(note_event_count, 2);
}

#[test]
fn p304_note_service_trash_and_delete_are_non_destructive_soft_states() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p304-note-service-trash-delete");
    fs::create_dir_all(visible_root.join("Notes")).expect("create Notes directory");
    let created = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Trash Me".to_string(),
            body_markdown: "Keep a recoverable file".to_string(),
            relative_path: Some("Notes/trash-me.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create note");

    let trashed =
        trash_markdown_note_service(&connection, &visible_root, &created.id).expect("trash note");
    assert_eq!(trashed.status, "trashed");
    assert_eq!(
        trashed.relative_path,
        format!("Notes/.Trash/{}.md", created.id)
    );
    assert!(!visible_root.join("Notes/trash-me.md").exists());
    assert!(visible_root.join(&trashed.relative_path).exists());
    assert!(trashed.metadata_json.contains("original_relative_path"));
    assert!(trashed.metadata_json.contains("Notes/trash-me.md"));
    let trashed_index_state: String = connection
        .query_row(
            "select scan_state from knowledge_index_entries where entity_type = 'note' and entity_id = ?1 and source_type = 'markdown_frontmatter'",
            params![created.id],
            |row| row.get(0),
        )
        .expect("trashed index state");
    assert_eq!(trashed_index_state, "missing");

    let re_trashed = trash_markdown_note_service(&connection, &visible_root, &created.id)
        .expect("double trash is idempotent");
    assert_eq!(re_trashed.status, "trashed");
    assert!(re_trashed.metadata_json.contains("Notes/trash-me.md"));
    assert!(!re_trashed
        .metadata_json
        .contains("original_relative_path\":\"Notes/.Trash"));

    let deleted = delete_markdown_note_service(&connection, &visible_root, &created.id)
        .expect("soft delete note");
    assert_eq!(deleted.status, "deleted");
    assert!(visible_root.join(&trashed.relative_path).exists());
    let deleted_index_state: String = connection
        .query_row(
            "select scan_state from knowledge_index_entries where entity_type = 'note' and entity_id = ?1 and source_type = 'markdown_frontmatter'",
            params![created.id],
            |row| row.get(0),
        )
        .expect("deleted index state");
    assert_eq!(deleted_index_state, "missing");

    let deleted_at: Option<String> = connection
        .query_row(
            "select deleted_at from notes where id = ?1",
            params![created.id],
            |row| row.get(0),
        )
        .expect("read deleted_at");
    assert!(deleted_at.is_some());

    let second = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Collision".to_string(),
            body_markdown: "Do not overwrite trash destination".to_string(),
            relative_path: Some("Notes/collision.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create collision note");
    let collision_path = visible_root.join(format!("Notes/.Trash/{}.md", second.id));
    fs::write(&collision_path, "pre-existing trash file").expect("write collision trash file");
    let collision_error = trash_markdown_note_service(&connection, &visible_root, &second.id)
        .expect_err("trash destination collision must fail closed");
    assert!(matches!(
        collision_error,
        RepositoryError::Constraint {
            entity: "notes",
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(&collision_path).expect("read collision file"),
        "pre-existing trash file"
    );
    assert!(visible_root.join("Notes/collision.md").exists());

    let third = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Double Trash".to_string(),
            body_markdown: "Preserve original path".to_string(),
            relative_path: Some("Notes/double-trash.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create double-trash note");
    let first_trash = trash_markdown_note_service(&connection, &visible_root, &third.id)
        .expect("trash third note");
    let second_trash = trash_markdown_note_service(&connection, &visible_root, &third.id)
        .expect("already-trashed note is idempotent");
    assert_eq!(second_trash.status, "trashed");
    assert_eq!(second_trash.relative_path, first_trash.relative_path);
    assert!(first_trash.metadata_json.contains("Notes/double-trash.md"));
    assert!(second_trash.metadata_json.contains("Notes/double-trash.md"));
    assert!(!second_trash
        .metadata_json
        .contains("original_relative_path\":\"Notes/.Trash"));
}

#[test]
fn p304_note_service_rejects_unsafe_paths_existing_files_and_secret_metadata() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p304-note-service-rejections");
    fs::create_dir_all(visible_root.join("Notes")).expect("create Notes directory");
    fs::write(visible_root.join("Notes/existing.md"), "existing").expect("write existing note");

    let unsafe_path = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Unsafe".to_string(),
            body_markdown: "Body".to_string(),
            relative_path: Some("Content/unsafe.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("non-Notes path rejected");
    assert!(matches!(
        unsafe_path,
        RepositoryError::Constraint {
            entity: "notes",
            ..
        }
    ));

    let existing_file = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Existing".to_string(),
            body_markdown: "Body".to_string(),
            relative_path: Some("Notes/existing.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("existing note file rejected");
    assert!(matches!(
        existing_file,
        RepositoryError::Constraint {
            entity: "notes",
            ..
        }
    ));

    let raw_secret = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Secret".to_string(),
            body_markdown: "Body".to_string(),
            relative_path: Some("Notes/secret.md".to_string()),
            metadata_json: r#"{"api_key":"sk-live-123"}"#.to_string(),
        },
    )
    .expect_err("secret metadata rejected");
    assert!(matches!(
        raw_secret,
        RepositoryError::SecretRejected {
            field: "metadata_json",
            ..
        }
    ));
}

#[test]
fn p304_note_service_rejects_trash_destination_collision_and_symlink_escape() {
    let connection = migrated_in_memory_connection();
    let visible_root = temp_home("p304-note-service-safe-paths");
    fs::create_dir_all(visible_root.join("Notes/link-parent")).expect("create Notes child");
    let outside = temp_home("p304-note-service-outside");
    fs::create_dir_all(&outside).expect("create outside dir");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = visible_root.join("Notes/symlinked");
        symlink(&outside, &link_path).expect("create symlink escape");
        let escaped = create_markdown_note_service(
            &connection,
            &visible_root,
            NoteCreateInput {
                title: "Escaped".to_string(),
                body_markdown: "Body".to_string(),
                relative_path: Some("Notes/symlinked/escaped.md".to_string()),
                metadata_json: "{}".to_string(),
            },
        )
        .expect_err("symlink parent escape rejected");
        assert!(matches!(
            escaped,
            RepositoryError::Constraint {
                entity: "notes",
                ..
            }
        ));
        assert!(!outside.join("escaped.md").exists());
    }

    let created = create_markdown_note_service(
        &connection,
        &visible_root,
        NoteCreateInput {
            title: "Collision".to_string(),
            body_markdown: "Recoverable body".to_string(),
            relative_path: Some("Notes/link-parent/collision.md".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create collision note");
    let trash_path = visible_root.join(format!("Notes/.Trash/{}.md", created.id));
    fs::create_dir_all(trash_path.parent().expect("trash parent")).expect("trash parent dir");
    fs::write(&trash_path, "existing recoverable content").expect("seed trash collision");

    let collision = trash_markdown_note_service(&connection, &visible_root, &created.id)
        .expect_err("trash destination collision rejected");
    assert!(matches!(
        collision,
        RepositoryError::Constraint {
            entity: "notes",
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(&trash_path).expect("read trash collision file"),
        "existing recoverable content"
    );
    assert!(visible_root.join("Notes/link-parent/collision.md").exists());
}

#[test]
fn p303_note_identity_frontmatter_is_stable_and_round_trips() {
    let markdown = "# Original title\n\nBody text\n";
    let identity = derive_note_identity_from_markdown("Notes/original-title.md", markdown)
        .expect("derive note identity");

    assert!(identity.id.starts_with("note_"));
    assert_eq!(identity.title, "Original title");
    assert_eq!(identity.slug, "original-title");
    assert_eq!(identity.relative_path, "Notes/original-title.md");
    assert_eq!(identity.conflict_state.as_str(), "none");

    let with_frontmatter =
        write_note_identity_frontmatter(markdown, &identity).expect("write frontmatter");
    assert!(with_frontmatter.starts_with("---\n"));
    assert!(with_frontmatter.contains(&format!("zoid_id: \"{}\"", identity.id)));
    assert!(with_frontmatter.contains("title: \"Original title\""));
    assert!(with_frontmatter.contains("slug: \"original-title\""));
    assert!(with_frontmatter.contains("# Original title"));

    let special_title = NoteIdentityMetadata {
        id: "note_yaml_safe_p303".to_string(),
        title: "Meeting: Client \"A\"".to_string(),
        slug: "meeting-client".to_string(),
        relative_path: "Notes/meeting-client.md".to_string(),
        frontmatter_json: "{}".to_string(),
        body_digest: "fnv1a64:0000000000000000".to_string(),
        conflict_state: NoteConflictState::None,
    };
    let special_frontmatter =
        write_note_identity_frontmatter("# Meeting: Client\n\nBody\n", &special_title)
            .expect("write YAML-safe frontmatter");
    assert!(special_frontmatter.contains("title: \"Meeting: Client \\\"A\\\"\""));
    let reparsed_special =
        derive_note_identity_from_markdown("Notes/meeting-client.md", &special_frontmatter)
            .expect("reparse YAML-safe frontmatter");
    assert_eq!(reparsed_special.title, "Meeting: Client \"A\"");
    assert_eq!(reparsed_special.id, "note_yaml_safe_p303");

    let renamed_title = with_frontmatter.replace("# Original title", "# Retitled manually");
    let reparsed = derive_note_identity_from_markdown("Notes/original-title.md", &renamed_title)
        .expect("reparse identity");
    assert_eq!(
        reparsed.id, identity.id,
        "frontmatter ID must remain stable across title edits"
    );
}

#[test]
fn p303_note_identity_rejects_unsafe_paths_and_invalid_frontmatter_ids() {
    let unsafe_path = derive_note_identity_from_markdown("../outside.md", "# Outside");
    assert!(matches!(
        unsafe_path,
        Err(RepositoryError::Constraint {
            entity: "notes",
            ..
        })
    ));

    let invalid_id = derive_note_identity_from_markdown(
        "Notes/bad.md",
        "---\nzoid_id: apple-note-remote\ntitle: Bad\n---\n\nBody",
    );
    assert!(matches!(
        invalid_id,
        Err(RepositoryError::Constraint {
            entity: "notes",
            ..
        })
    ));
}

#[test]
fn p303_note_identity_upsert_records_index_metadata_and_duplicate_conflicts() {
    let connection = migrated_in_memory_connection();
    let first = derive_note_identity_from_markdown(
        "Notes/first.md",
        "---\nzoid_id: note_duplicate_for_p303\ntitle: First\nslug: first\n---\n\n# First",
    )
    .expect("first identity");
    upsert_note_identity_metadata(&connection, &first).expect("insert first note identity");

    let stored_frontmatter: String = connection
        .query_row(
            "select frontmatter_json from notes where id = 'note_duplicate_for_p303'",
            [],
            |row| row.get(0),
        )
        .expect("stored frontmatter json");
    assert!(stored_frontmatter.contains("note_duplicate_for_p303"));

    let index_title: String = connection
        .query_row(
            "select title from knowledge_index_entries where entity_type = 'note' and entity_id = 'note_duplicate_for_p303' and source_type = 'markdown_frontmatter'",
            [],
            |row| row.get(0),
        )
        .expect("frontmatter index title");
    assert_eq!(index_title, "First");

    let duplicate = derive_note_identity_from_markdown(
        "Notes/second.md",
        "---\nzoid_id: note_duplicate_for_p303\ntitle: Second\nslug: second\n---\n\n# Second",
    )
    .expect("duplicate identity parses before persistence");
    let duplicate_error = upsert_note_identity_metadata(&connection, &duplicate)
        .expect_err("duplicate frontmatter ID across paths must fail closed");
    assert!(
        matches!(duplicate_error, RepositoryError::Constraint { entity: "notes", ref message } if message.contains("duplicate_id"))
    );

    let conflict_state: String = connection
        .query_row(
            "select conflict_state from notes where id = 'note_duplicate_for_p303'",
            [],
            |row| row.get(0),
        )
        .expect("existing note conflict state");
    assert_eq!(conflict_state, "duplicate_id");
}

#[test]
fn p302_schema_version_nine_has_notes_files_and_knowledge_index_tables() {
    let connection = migrated_in_memory_connection();
    assert!(
        get_migration_version(&connection).expect("migration version") >= 9,
        "P3.02 schema migration must advance to version 9 or later"
    );

    assert_table_has_columns(
        &connection,
        "notes",
        &[
            "id",
            "title",
            "slug",
            "relative_path",
            "status",
            "conflict_state",
            "frontmatter_json",
            "body_digest",
            "created_at",
            "updated_at",
            "deleted_at",
            "metadata_json",
        ],
    );
    assert_index_exists(&connection, "notes", "idx_notes_status_updated");
    assert_index_exists(&connection, "notes", "idx_notes_relative_path_active");
    assert_index_exists(&connection, "notes", "idx_notes_conflict_state");

    assert_table_has_columns(
        &connection,
        "file_references",
        &[
            "id",
            "root_key",
            "relative_path",
            "display_name",
            "file_kind",
            "mime_type",
            "extension",
            "byte_size",
            "content_fingerprint",
            "status",
            "conflict_state",
            "last_seen_at",
            "created_at",
            "updated_at",
            "deleted_at",
            "metadata_json",
        ],
    );
    assert_index_exists(
        &connection,
        "file_references",
        "idx_file_references_root_path_active",
    );
    assert_index_exists(
        &connection,
        "file_references",
        "idx_file_references_status_updated",
    );
    assert_index_exists(
        &connection,
        "file_references",
        "idx_file_references_conflict_state",
    );

    assert_table_has_columns(
        &connection,
        "knowledge_index_entries",
        &[
            "id",
            "entity_type",
            "entity_id",
            "source_type",
            "title",
            "excerpt",
            "search_text",
            "content_digest",
            "source_modified_at",
            "scan_state",
            "indexed_at",
            "metadata_json",
        ],
    );
    assert_index_exists(
        &connection,
        "knowledge_index_entries",
        "idx_knowledge_index_entity_source",
    );
    assert_index_exists(
        &connection,
        "knowledge_index_entries",
        "idx_knowledge_index_scan_state",
    );
}

#[test]
fn p302_notes_files_and_index_constraints_fail_closed() {
    let connection = migrated_in_memory_connection();

    connection
        .execute(
            "insert into notes (id, title, slug, relative_path, status, conflict_state, frontmatter_json, body_digest, metadata_json)
             values (?1, ?2, ?3, ?4, 'active', 'none', '{}', ?5, '{}')",
            params![
                "note-p302-valid",
                "P3.02 valid note",
                "p302-valid-note",
                "Notes/p302-valid-note.md",
                "sha256:note"
            ],
        )
        .expect("valid note row should insert");

    let invalid_note_status = connection.execute(
        "insert into notes (id, title, relative_path, status, conflict_state, frontmatter_json, metadata_json)
         values ('note-bad-status', 'Bad', 'Notes/bad.md', 'synced_to_apple_notes', 'none', '{}', '{}')",
        [],
    );
    assert!(
        invalid_note_status.is_err(),
        "invalid note status must fail closed"
    );

    let invalid_frontmatter = connection.execute(
        "insert into notes (id, title, relative_path, status, conflict_state, frontmatter_json, metadata_json)
         values ('note-bad-json', 'Bad JSON', 'Notes/bad-json.md', 'active', 'none', '{bad', '{}')",
        [],
    );
    assert!(
        invalid_frontmatter.is_err(),
        "invalid frontmatter JSON must fail closed"
    );

    connection
        .execute(
            "insert into file_references (id, root_key, relative_path, display_name, file_kind, mime_type, extension, byte_size, content_fingerprint, status, conflict_state, metadata_json)
             values ('file-p302-valid', 'zoid_visible', 'Assets/p302.txt', 'p302.txt', 'document', 'text/plain', 'txt', 12, 'sha256:file', 'indexed', 'none', '{}')",
            [],
        )
        .expect("valid file reference should insert");

    let invalid_file_kind = connection.execute(
        "insert into file_references (id, root_key, relative_path, display_name, file_kind, status, conflict_state, metadata_json)
         values ('file-bad-kind', 'zoid_visible', 'Assets/bad', 'bad', 'whole_home_crawler', 'indexed', 'none', '{}')",
        [],
    );
    assert!(
        invalid_file_kind.is_err(),
        "invalid file kind must fail closed"
    );

    let invalid_byte_size = connection.execute(
        "insert into file_references (id, root_key, relative_path, display_name, file_kind, byte_size, status, conflict_state, metadata_json)
         values ('file-bad-size', 'zoid_visible', 'Assets/bad-size', 'bad-size', 'document', -1, 'indexed', 'none', '{}')",
        [],
    );
    assert!(
        invalid_byte_size.is_err(),
        "negative byte size must fail closed"
    );

    connection
        .execute(
            "insert into knowledge_index_entries (id, entity_type, entity_id, source_type, title, excerpt, search_text, content_digest, scan_state, metadata_json)
             values ('index-note-p302', 'note', 'note-p302-valid', 'markdown_frontmatter', 'P3.02 valid note', 'excerpt', 'search text', 'sha256:index', 'current', '{}')",
            [],
        )
        .expect("valid index entry should insert");

    let invalid_entity_type = connection.execute(
        "insert into knowledge_index_entries (id, entity_type, entity_id, source_type, scan_state, metadata_json)
         values ('index-bad-entity', 'apple_note', 'apple-1', 'markdown_frontmatter', 'current', '{}')",
        [],
    );
    assert!(
        invalid_entity_type.is_err(),
        "unsupported index entity type must fail closed"
    );

    let invalid_scan_state = connection.execute(
        "insert into knowledge_index_entries (id, entity_type, entity_id, source_type, scan_state, metadata_json)
         values ('index-bad-state', 'note', 'note-p302-valid', 'markdown_frontmatter', 'synced_remote', '{}')",
        [],
    );
    assert!(
        invalid_scan_state.is_err(),
        "unsupported scan state must fail closed"
    );
}

#[test]
fn p302_notes_and_files_link_to_existing_tasks_through_entity_links() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "P3.02 linked task");

    connection
        .execute(
            "insert into notes (id, title, slug, relative_path, status, conflict_state, frontmatter_json, metadata_json)
             values ('note-p302-link', 'Linked note', 'linked-note', 'Notes/linked-note.md', 'active', 'none', '{}', '{}')",
            [],
        )
        .expect("insert linked note");
    connection
        .execute(
            "insert into file_references (id, root_key, relative_path, display_name, file_kind, status, conflict_state, metadata_json)
             values ('file-p302-link', 'zoid_visible', 'Assets/linked.txt', 'linked.txt', 'document', 'indexed', 'none', '{}')",
            [],
        )
        .expect("insert linked file");

    insert_or_get_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-p302-task-note",
            source_type: "task",
            source_id: &task.id,
            target_type: "note",
            target_id: "note-p302-link",
            relation_type: "references",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("link task to note");
    insert_or_get_entity_link(
        &connection,
        EntityLinkInput {
            id: "link-p302-file-task",
            source_type: "file",
            source_id: "file-p302-link",
            target_type: "task",
            target_id: &task.id,
            relation_type: "supports",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .expect("link file to task");

    let outgoing_task_links = list_entity_links_by_source(
        &connection,
        EntityLinkListFilter {
            entity_type: "task",
            entity_id: &task.id,
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list outgoing task links");
    let incoming_task_links = list_entity_links_by_target(
        &connection,
        EntityLinkListFilter {
            entity_type: "task",
            entity_id: &task.id,
            relation_type: None,
            counterpart_type: None,
        },
    )
    .expect("list incoming task links");

    assert!(outgoing_task_links
        .iter()
        .any(|link| link.target_type == "note" && link.target_id == "note-p302-link"));
    assert!(incoming_task_links
        .iter()
        .any(|link| link.source_type == "file" && link.source_id == "file-p302-link"));
}

#[test]
fn migrations_create_p105_core_schema_tables() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    assert_eq!(
        get_migration_version(&connection).unwrap(),
        MIGRATIONS
            .last()
            .expect("at least one migration is registered")
            .version
    );

    assert_table_has_columns(
        &connection,
        "tasks",
        &[
            "id",
            "title",
            "detail",
            "status",
            "priority",
            "workspace_key",
            "created_at",
            "updated_at",
            "archived_at",
            "deleted_at",
            "metadata_json",
        ],
    );
    assert_index_exists(&connection, "tasks", "idx_tasks_active_priority_time");
    assert_index_exists(&connection, "tasks", "idx_tasks_status");
    assert_index_exists(&connection, "tasks", "idx_tasks_workspace_active");

    assert_table_has_columns(
        &connection,
        "app_settings",
        &[
            "key",
            "value_json",
            "value_type",
            "scope",
            "description",
            "created_at",
            "updated_at",
        ],
    );
    assert_table_has_columns(
        &connection,
        "integration_statuses",
        &[
            "integration_key",
            "display_name",
            "status",
            "config_json",
            "credential_ref",
            "last_checked_at",
            "created_at",
            "updated_at",
        ],
    );
    assert_table_has_columns(
        &connection,
        "entity_links",
        &[
            "id",
            "source_type",
            "source_id",
            "target_type",
            "target_id",
            "relation_type",
            "created_at",
            "created_by_actor_type",
            "metadata_json",
        ],
    );
    assert_table_has_columns(
        &connection,
        "log_references",
        &[
            "id",
            "log_scope",
            "relative_path",
            "redaction_count",
            "byte_count",
            "created_at",
            "metadata_json",
        ],
    );
    assert_table_has_columns(
        &connection,
        "file_references",
        &[
            "id",
            "root_key",
            "relative_path",
            "display_name",
            "file_kind",
            "mime_type",
            "content_fingerprint",
            "status",
            "conflict_state",
            "metadata_json",
            "created_at",
            "updated_at",
        ],
    );
    assert_table_has_columns(
        &connection,
        "action_policies",
        &[
            "category",
            "policy",
            "reviewer_required",
            "human_confirmation",
            "reason",
            "created_at",
            "updated_at",
        ],
    );
    assert_table_has_columns(
        &connection,
        "confirmation_decisions",
        &[
            "id",
            "action_category",
            "decision",
            "actor_type",
            "actor_id",
            "summary",
            "event_id",
            "metadata_json",
            "created_at",
        ],
    );

    assert_index_exists(&connection, "entity_links", "idx_entity_links_source");
    assert_index_exists(&connection, "entity_links", "idx_entity_links_target");
    assert_index_exists(
        &connection,
        "log_references",
        "idx_log_references_scope_created",
    );
    assert_index_exists(
        &connection,
        "file_references",
        "idx_file_references_root_path_active",
    );
    assert_index_exists(
        &connection,
        "confirmation_decisions",
        "idx_confirmation_decisions_category_created",
    );
}

#[test]
fn migrations_enable_foreign_keys_and_reject_invalid_core_references() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let foreign_keys_enabled: i64 = connection
        .query_row("pragma foreign_keys", [], |row| row.get(0))
        .expect("read foreign key pragma");
    assert_eq!(foreign_keys_enabled, 1);

    let invalid_event_target = connection.execute(
            "insert into event_targets (event_id, entity_type, entity_id, relation_type) values ('evt_missing', 'workspace', 'today', 'primary')",
            [],
        );
    assert!(
        matches!(
            invalid_event_target,
            Err(rusqlite::Error::SqliteFailure(_, _))
        ),
        "missing event target FK must be rejected; got {invalid_event_target:?}"
    );

    let invalid_confirmation_decision = connection.execute(
            "insert into confirmation_decisions (id, action_category, decision, actor_type, summary) values ('confirm_missing_category', 'not_a_policy', 'approved', 'human', 'invalid category')",
            [],
        );
    assert!(
        matches!(
            invalid_confirmation_decision,
            Err(rusqlite::Error::SqliteFailure(_, _))
        ),
        "missing action policy FK must be rejected; got {invalid_confirmation_decision:?}"
    );
}

#[test]
fn migrations_seed_action_policies_for_confirmation_decisions() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let send_email_policy: (String, String, String, String) = connection
            .query_row(
                "select policy, reviewer_required, human_confirmation, reason from action_policies where category = 'send_email'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("send_email policy should be seeded");
    assert_eq!(send_email_policy.0, "ask_before_action");
    assert_eq!(send_email_policy.1, "maybe");
    assert_eq!(send_email_policy.2, "always");
    assert!(send_email_policy.3.contains("Email"));

    connection
            .execute(
                "insert into confirmation_decisions (id, action_category, decision, actor_type, summary) values ('confirm_send_email', 'send_email', 'approved', 'human', 'approved email draft')",
                [],
            )
            .expect("known action category should satisfy confirmation_decisions FK");
}

#[test]
fn p105_core_schema_upgrade_from_v2_is_idempotent_and_non_secret() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    connection
        .execute_batch(
            "
                create table schema_migrations (
                    version integer primary key,
                    name text not null,
                    applied_at text not null default current_timestamp
                );
                insert into schema_migrations (version, name) values (1, 'foundation_schema');
                insert into schema_migrations (version, name) values (2, 'event_schema_backfill');

                create table workspaces (
                    id text primary key,
                    label text not null,
                    description text not null default '',
                    position integer not null,
                    enabled integer not null default 1,
                    created_at text not null default current_timestamp,
                    updated_at text not null default current_timestamp
                );

                create table events (
                    id text primary key,
                    type text not null,
                    timestamp text not null default current_timestamp,
                    actor_type text not null,
                    actor_id text,
                    workspace_key text,
                    summary text not null,
                    severity text not null default 'info',
                    source text not null,
                    metadata_json text not null default '{}',
                    created_at text not null default current_timestamp
                );

                create table event_targets (
                    event_id text not null,
                    entity_type text not null,
                    entity_id text not null,
                    relation_type text not null,
                    primary key (event_id, entity_type, entity_id, relation_type),
                    foreign key (event_id) references events(id) on delete cascade
                );
                ",
        )
        .expect("seed v2 schema");

    run_migrations(&connection).expect("run p105 migration first time");
    run_migrations(&connection).expect("run p105 migration second time");

    assert_eq!(
        get_migration_version(&connection).unwrap(),
        MIGRATIONS
            .last()
            .expect("at least one migration is registered")
            .version
    );

    let p105_rows: i64 = connection
        .query_row(
            "select count(*) from schema_migrations where version = 3",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(p105_rows, 1);

    assert_table_has_columns(&connection, "app_settings", &["key", "value_json"]);
    assert_table_has_columns(
        &connection,
        "integration_statuses",
        &["integration_key", "config_json", "credential_ref"],
    );

    let forbidden_secret_columns: i64 = connection
            .query_row(
                "
                select count(*)
                from sqlite_schema as schema
                join pragma_table_info(schema.name) as columns
                where schema.type = 'table'
                  and schema.name in ('app_settings', 'integration_statuses')
                  and lower(columns.name) in ('secret', 'secret_value', 'token', 'access_token', 'refresh_token', 'api_key', 'password')
                ",
                [],
                |row| row.get(0),
            )
            .unwrap();
    assert_eq!(forbidden_secret_columns, 0);
}

#[test]
fn foundation_event_is_idempotent_and_linked() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    write_foundation_event(&connection).expect("write event");
    write_foundation_event(&connection).expect("write event again");

    let event_count: i64 = connection
            .query_row(
                "select count(*) from events where type = 'foundation.ready' and actor_type = 'system' and actor_id = 'zoid' and workspace_key = 'today'",
                [],
                |row| row.get(0),
            )
            .unwrap();
    assert_eq!(event_count, 1);

    let target_count: i64 = connection
        .query_row("select count(*) from event_targets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(target_count, 1);
}

#[test]
fn migrations_upgrade_existing_foundation_database() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    connection
            .execute_batch(
                "
                create table schema_migrations (
                    version integer primary key,
                    name text not null,
                    applied_at text not null default current_timestamp
                );
                insert into schema_migrations (version, name) values (1, 'foundation_schema');

                create table workspaces (
                    id text primary key,
                    label text not null,
                    created_at text not null default current_timestamp
                );

                create table events (
                    id text primary key,
                    type text not null,
                    summary text not null,
                    actor text not null,
                    severity text not null default 'info',
                    source text not null,
                    metadata_json text not null default '{}',
                    created_at text not null default current_timestamp
                );
                insert into events (id, type, summary, actor, source)
                values ('evt_existing', 'foundation.ready', 'Zoid foundation initialized', 'system', 'app_shell');
                ",
            )
            .expect("seed old foundation schema");

    run_migrations(&connection).expect("run compatibility migrations");
    ensure_workspace_schema_compatibility(&connection).expect("upgrade workspaces");
    seed_workspaces(&connection).expect("seed new workspaces");
    write_foundation_event(&connection).expect("backfill event target");

    assert_eq!(
        get_migration_version(&connection).unwrap(),
        MIGRATIONS
            .last()
            .expect("at least one migration is registered")
            .version
    );
    assert_eq!(count_table(&connection, "workspaces").unwrap(), 14);

    let event_fields: (String, String, String) = connection
        .query_row(
            "select actor_type, actor_id, workspace_key from events where id = 'evt_existing'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        event_fields,
        (
            "system".to_string(),
            "zoid".to_string(),
            "today".to_string()
        )
    );

    let target_count: i64 = connection
        .query_row("select count(*) from event_targets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(target_count, 1);
}

#[test]
fn secure_redaction_masks_obvious_secrets() {
    let redacted = redact_secrets(
            "Authorization: Bearer sk-live-123\napi_key=abc123\nclient_secret: keep-me-private\nnormal line",
        );

    assert_eq!(redacted.redaction_count, 3);
    assert!(!redacted.text.contains("sk-live-123"));
    assert!(!redacted.text.contains("abc123"));
    assert!(!redacted.text.contains("keep-me-private"));
    assert!(redacted.text.contains("[REDACTED]"));
    assert!(redacted.text.contains("normal line"));
}

#[test]
fn redaction_masks_multiple_obvious_key_value_and_bearer_forms_per_line() {
    let redacted = redact_secrets(
            "credential=dummy-credential-1 private_key: dummy-private-key Authorization=Bearer dummy-bearer-token visible=true",
        );

    assert!(redacted.redaction_count >= 3);
    assert!(!redacted.text.contains("dummy-credential-1"));
    assert!(!redacted.text.contains("dummy-private-key"));
    assert!(!redacted.text.contains("dummy-bearer-token"));
    assert!(redacted.text.contains("visible=true"));
    assert!(redacted.text.matches("[REDACTED]").count() >= 3);
}

#[test]
fn redaction_masks_spaced_secret_key_value_forms() {
    let redacted = redact_secrets(
            "password : hunter2\napi_key = sk-live-spaced\nprivate_key : dummy-private-key\nrefresh_token : abc\nvisible=true",
        );

    assert!(redacted.redaction_count >= 4);
    assert!(!redacted.text.contains("hunter2"));
    assert!(!redacted.text.contains("sk-live-spaced"));
    assert!(!redacted.text.contains("dummy-private-key"));
    assert!(!redacted.text.contains("abc"));
    assert!(redacted.text.contains("password : [REDACTED]"));
    assert!(redacted.text.contains("api_key = [REDACTED]"));
    assert!(redacted.text.contains("private_key : [REDACTED]"));
    assert!(redacted.text.contains("refresh_token : [REDACTED]"));
    assert!(redacted.text.contains("visible=true"));
}

#[test]
fn redaction_masks_multi_token_values_after_secret_keys() {
    let redacted = redact_secrets(
            "password: correct horse battery staple; visible=true\nclient_secret: line one continued, safe tail",
        );

    assert!(redacted.redaction_count >= 2);
    assert!(!redacted.text.contains("correct"));
    assert!(!redacted.text.contains("horse"));
    assert!(!redacted.text.contains("battery"));
    assert!(!redacted.text.contains("staple"));
    assert!(!redacted.text.contains("line one continued"));
    assert!(redacted.text.contains("password: [REDACTED]; visible=true"));
    assert!(redacted
        .text
        .contains("client_secret: [REDACTED], safe tail"));
}

#[test]
fn metadata_redaction_recurses_under_secret_keys_and_keeps_json_valid() {
    let redacted = redact_metadata_json(
        r#"{
                "safe":"kept",
                "authorization":{"scheme":"Bearer","value":"dummy-bearer-token"},
                "nested":[{"client_secret":"dummy-client-secret","safe_number":7}],
                "passwords":["dummy-password-one",{"note":"dummy-password-two"}],
                "flags":{"enabled":true}
            }"#,
    );
    let parsed: Value =
        serde_json::from_str(&redacted).expect("redacted metadata remains valid JSON");

    assert_eq!(parsed["safe"], "kept");
    assert_eq!(parsed["flags"]["enabled"], true);
    assert_eq!(parsed["nested"][0]["safe_number"], 7);
    assert!(!redacted.contains("dummy-bearer-token"));
    assert!(!redacted.contains("dummy-client-secret"));
    assert!(!redacted.contains("dummy-password-one"));
    assert!(!redacted.contains("dummy-password-two"));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn redact_metadata_json_invalid_json_returns_redacted_notice_and_no_raw_secret() {
    let redacted = redact_metadata_json(
        "not json api_key=dummy-api-key, Authorization: Bearer supersecretvalue123, visible text",
    );
    let parsed: Value = serde_json::from_str(&redacted)
        .expect("invalid metadata fallback should still be valid JSON");

    assert_eq!(parsed["redaction_notice"], "metadata_was_not_valid_json");
    let redacted_text = parsed["redacted_text"]
        .as_str()
        .expect("notice should include redacted_text");
    assert!(redacted_text.contains("visible text"));
    assert!(redacted_text.contains("[REDACTED]"));
    assert!(!redacted.contains("dummy-api-key"));
    assert!(!redacted.contains("supers"));
}

#[test]
fn event_writer_uses_common_redaction_for_nested_metadata_and_summary() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_workspaces(&connection).expect("seed workspaces");

    let created = create_event_record(
        &connection,
        EventCreateInput {
            action_type: "create_local_task",
            outcome: "succeeded",
            actor_type: "system",
            actor_id: Some("redaction_test"),
            workspace_key: Some("tasks"),
            summary: "ran with refresh_token=dummy-refresh-token and visible summary",
            source: "redaction_test",
            metadata_json: r#"{"credential":{"value":"dummy-credential-value"},"safe":"kept"}"#,
            targets: vec![],
        },
    )
    .expect("create redacted event");

    assert!(!created.summary.contains("dummy-refresh-token"));
    assert!(created.summary.contains("visible summary"));
    assert!(!created.metadata_json.contains("dummy-credential-value"));
    assert!(created.metadata_json.contains("\"safe\":\"kept\""));
    serde_json::from_str::<Value>(&created.metadata_json)
        .expect("event metadata remains valid JSON");
}

#[test]
fn keychain_readiness_is_truthful_when_native_probe_is_not_implemented() {
    let readiness = keychain_readiness_status();

    assert_eq!(
        readiness.status,
        "blocked_unverified_native_keychain_not_tested"
    );
    assert!(!readiness.ready);
    assert!(!readiness.credential_storage_enabled);
    assert!(!readiness.test_path_exercised);
    assert!(readiness.reason.contains("not implemented"));
    assert!(!readiness.reason.contains("secret"));
    assert!(!readiness.reason.contains("token"));
}

#[test]
fn secure_foundation_status_embeds_truthful_keychain_readiness_without_claiming_ready() {
    let logs_dir = std::env::temp_dir().join(format!("zoid-secure-status-{}", now_millis()));
    fs::create_dir_all(&logs_dir).expect("create logs dir");
    let probe_path = logs_dir.join("foundation.log");
    fs::write(&probe_path, "foundation.ready secure services checked")
        .expect("write safe log probe");
    let safe_log_probe = SafeLogWrite {
        path: probe_path,
        redaction_count: 0,
        bytes_written: 40,
    };

    let status = secure_foundation_status(&safe_log_probe);

    assert_eq!(
        status.keychain_status, status.keychain.status,
        "legacy string status must match typed keychain readiness"
    );
    assert!(!status.keychain.ready);
    assert!(!status.keychain.credential_storage_enabled);
    assert!(!status.keychain.test_path_exercised);
    assert!(status.keychain.reason.contains("not implemented"));

    fs::remove_dir_all(logs_dir).ok();
}

#[test]
fn safe_log_writer_sanitizes_scope_and_persists_redacted_content() {
    let logs_dir = std::env::temp_dir().join(format!("zoid-log-test-{}", now_millis()));
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let write = write_safe_log(
        &connection,
        &logs_dir,
        "../agent/run 1",
        "token=abc123\nvisible output",
    )
    .expect("write safe log");
    let stored = fs::read_to_string(&write.path).expect("read safe log");

    assert_eq!(write.path, logs_dir.join("___agent_run_1.log"));
    assert!(write.path.starts_with(&logs_dir));
    assert_eq!(write.redaction_count, 1);
    assert!(write.bytes_written > 0);
    assert!(!stored.contains("abc123"));
    assert!(stored.contains("token= [REDACTED]"));
    assert!(stored.contains("visible output"));

    fs::remove_dir_all(logs_dir).ok();
}

#[test]
fn safe_log_scope_falls_back_to_app_for_empty_or_all_unsafe_scope() {
    for unsafe_scope in ["", "   ", "../", "***"] {
        assert_eq!(safe_log_scope(unsafe_scope), "app");
    }
}

#[test]
fn safe_log_writer_upserts_reference_without_raw_secret_metadata() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    let logs_dir = std::env::temp_dir().join(format!("zoid-log-ref-{}", now_millis()));

    let first = write_safe_log(
        &connection,
        &logs_dir,
        "../../agent/secret run",
        "Authorization: Bearer sk-live-reference\nvisible one",
    )
    .expect("write first safe log");
    let second = write_safe_log(
        &connection,
        &logs_dir,
        "../../agent/secret run",
        "password=super-secret-value\nvisible two",
    )
    .expect("write second safe log");

    assert_eq!(first.path, logs_dir.join("______agent_secret_run.log"));
    assert_eq!(first.path, second.path);

    let row_count: i64 = connection
        .query_row("select count(*) from log_references", [], |row| row.get(0))
        .expect("count log refs");
    assert_eq!(row_count, 1);

    let (scope, relative_path, redaction_count, byte_count, metadata_json): (
            String,
            String,
            i64,
            i64,
            String,
        ) = connection
            .query_row(
                "select log_scope, relative_path, redaction_count, byte_count, metadata_json from log_references",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .expect("read log ref");

    assert_eq!(scope, "______agent_secret_run");
    assert_eq!(relative_path, "______agent_secret_run.log");
    assert_eq!(redaction_count, 2);
    assert_eq!(
        byte_count,
        (first.bytes_written + second.bytes_written) as i64
    );
    assert!(!metadata_json.contains("sk-live-reference"));
    assert!(!metadata_json.contains("super-secret-value"));
    assert!(metadata_json.contains("last_bytes_written"));

    let stored = fs::read_to_string(&first.path).expect("read safe log");
    assert!(!stored.contains("sk-live-reference"));
    assert!(!stored.contains("super-secret-value"));

    fs::remove_dir_all(logs_dir).ok();
}

#[test]
fn safe_log_writer_rotates_before_append_and_keeps_reference_safe() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    let logs_dir = std::env::temp_dir().join(format!("zoid-log-rotate-{}", now_millis()));
    fs::create_dir_all(&logs_dir).expect("create logs dir");
    let log_path = logs_dir.join("agent.log");
    fs::write(&log_path, "x".repeat(SAFE_LOG_MAX_BYTES - 8)).expect("seed oversized log");

    let write = write_safe_log(
        &connection,
        &logs_dir,
        "agent",
        "api_key=raw-rotation-secret\npost rotation content",
    )
    .expect("write rotating safe log");

    let active = fs::read_to_string(&log_path).expect("read active rotated log");
    let rotated = fs::read_to_string(logs_dir.join("agent.log.1")).expect("read rotated log");
    assert_eq!(write.path, log_path);
    assert!(rotated.starts_with('x'));
    assert!(active.contains("post rotation content"));
    assert!(!active.contains("raw-rotation-secret"));
    assert!(fs::metadata(&log_path).expect("active metadata").len() <= SAFE_LOG_MAX_BYTES as u64);
    assert!(
        fs::metadata(logs_dir.join("agent.log.1"))
            .expect("rotated metadata")
            .len()
            <= SAFE_LOG_MAX_BYTES as u64
    );

    let metadata_json: String = connection
        .query_row(
            "select metadata_json from log_references where relative_path = 'agent.log'",
            [],
            |row| row.get(0),
        )
        .expect("read metadata");
    assert!(metadata_json.contains("\"rotated\":true"));
    assert!(!metadata_json.contains("raw-rotation-secret"));

    fs::remove_dir_all(logs_dir).ok();
}

#[test]
fn safe_log_writer_truncates_oversized_line_and_records_truncated_metadata() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    let logs_dir = std::env::temp_dir().join(format!("zoid-log-truncate-{}", now_millis()));
    let oversized_content = format!("visible-start {}", "x".repeat(SAFE_LOG_MAX_BYTES + 512));

    let write = write_safe_log(&connection, &logs_dir, "agent", &oversized_content)
        .expect("write oversized safe log line");
    let stored = fs::read_to_string(&write.path).expect("read truncated safe log");
    let (byte_count, metadata_json): (i64, String) = connection
        .query_row(
            "select byte_count, metadata_json from log_references where log_scope = 'agent'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read truncated log reference");
    let metadata: Value = serde_json::from_str(&metadata_json).expect("metadata is JSON");

    assert_eq!(write.bytes_written, SAFE_LOG_MAX_BYTES);
    assert_eq!(byte_count, SAFE_LOG_MAX_BYTES as i64);
    assert!(stored.starts_with("visible-start"));
    assert!(stored.ends_with("\n[TRUNCATED]\n"));
    assert_eq!(metadata["truncated"], true);
    assert_eq!(metadata["last_bytes_written"], SAFE_LOG_MAX_BYTES as i64);
    assert_eq!(metadata["max_bytes"], SAFE_LOG_MAX_BYTES as i64);

    fs::remove_dir_all(logs_dir).ok();
}

#[cfg(unix)]
#[test]
fn safe_log_writer_rejects_symlinked_log_file_before_append() {
    let logs_dir = std::env::temp_dir().join(format!("zoid-log-symlink-{}", now_millis()));
    let target = logs_dir.with_extension("target.log");
    fs::create_dir_all(&logs_dir).expect("create logs dir");
    fs::write(&target, "original target content\n").expect("write symlink target");
    std::os::unix::fs::symlink(&target, logs_dir.join("agent.log"))
        .expect("create managed log symlink");

    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let error = write_safe_log(&connection, &logs_dir, "agent", "new content")
        .expect_err("log symlink must be rejected before append");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert!(error.to_string().contains("log file"));
    assert!(error.to_string().contains("symlink"));
    assert_eq!(
        fs::read_to_string(&target).expect("read target after rejected append"),
        "original target content\n"
    );

    fs::remove_dir_all(logs_dir).ok();
    fs::remove_file(target).ok();
}

#[test]
fn action_policy_evaluates_generic_requests_into_canonical_categories_and_gates() {
    let email = evaluate_action_request(
        &ActionRequest::new(ActionType::Send)
            .target("gmail")
            .consequence(ActionConsequence::ExternalWrite),
    );
    assert_eq!(email.category, "send_email");
    assert_eq!(email.policy, ActionPolicy::AskBeforeAction);
    assert!(!email.allowed_now);
    assert!(email.requires_confirmation);
    assert!(email.requires_gate);
    assert!(email.reason.contains("Email"));

    let harmless_note = evaluate_action_request(
        &ActionRequest::new(ActionType::Create)
            .target("local markdown note")
            .scope(ActionScope::LocalPrivate),
    );
    assert_eq!(harmless_note.category, "create_private_markdown_note");
    assert_eq!(harmless_note.policy, ActionPolicy::Allow);
    assert!(harmless_note.allowed_now);
    assert!(!harmless_note.requires_confirmation);
    assert!(!harmless_note.requires_reviewer);
    assert!(!harmless_note.requires_clear_task);
}

#[test]
fn action_policy_covers_all_action_dimensions_and_consequential_hints() {
    let cases = [
        (
            ActionRequest::new(ActionType::Read).target("local app data"),
            "read_local_app_data",
        ),
        (
            ActionRequest::new(ActionType::Create).target("task"),
            "create_local_task",
        ),
        (
            ActionRequest::new(ActionType::Update).target("code repo"),
            "modify_code_repo_files",
        ),
        (
            ActionRequest::new(ActionType::Delete)
                .target("file")
                .destructive(true),
            "delete_trash_files",
        ),
        (
            ActionRequest::new(ActionType::Send).target("email"),
            "send_email",
        ),
        (
            ActionRequest::new(ActionType::Publish).target("social post"),
            "publish_schedule_content",
        ),
        (
            ActionRequest::new(ActionType::Deploy).target("production"),
            "deploy_redeploy_rollback",
        ),
        (
            ActionRequest::new(ActionType::File)
                .target("bulk rename")
                .bulk(true),
            "bulk_file_operations",
        ),
        (
            ActionRequest::new(ActionType::Process).target("automation run"),
            "run_existing_automation",
        ),
        (
            ActionRequest::new(ActionType::Update).target("credentials integration"),
            "change_credentials_settings_integrations",
        ),
        (
            ActionRequest::new(ActionType::Create).target("calendar event"),
            "create_calendar_event",
        ),
        (
            ActionRequest::new(ActionType::Update)
                .target("external api record")
                .consequence(ActionConsequence::ExternalWrite),
            "external_api_write",
        ),
        (
            ActionRequest::new(ActionType::Update).target("git commit"),
            "commit_push_merge",
        ),
    ];

    for (request, category) in cases {
        let decision = evaluate_action_request(&request);
        assert_eq!(
            decision.category, category,
            "category mismatch for {request:?}"
        );
        assert!(
            !decision.reason.trim().is_empty(),
            "missing reason for {category}"
        );
        if decision.policy == ActionPolicy::Allow {
            assert!(
                decision.allowed_now,
                "allowed policy should execute now for {category}"
            );
        } else {
            assert!(
                !decision.allowed_now,
                "gated policy must not execute now for {category}"
            );
            assert!(
                decision.requires_gate,
                "gated policy should expose gate boolean for {category}"
            );
        }
    }
}

#[test]
fn action_policy_classifier_preserves_high_risk_precedence_over_generic_terms() {
    let automation_schedule = evaluate_action_request(
        &ActionRequest::new(ActionType::Update).target("automation schedule"),
    );
    assert_eq!(
        automation_schedule.category, "change_automation_schedule",
        "specific automation schedule changes must not be shadowed by generic publish/schedule"
    );

    for target in ["bulk email", "mass email"] {
        let decision =
            evaluate_action_request(&ActionRequest::new(ActionType::Send).target(target));
        assert_eq!(
            decision.category, "send_email",
            "category mismatch for {target}"
        );
        assert_eq!(decision.human_confirmation, HumanConfirmation::Always);
        assert!(decision.requires_confirmation);
    }

    let destructive_deploy = evaluate_action_request(
        &ActionRequest::new(ActionType::Deploy)
            .target("destructive bulk deploy to production")
            .bulk(true)
            .destructive(true),
    );
    assert_eq!(destructive_deploy.category, "deploy_redeploy_rollback");
    assert_ne!(destructive_deploy.category, "bulk_file_operations");
    assert_ne!(destructive_deploy.category, "delete_trash_files");

    let bulk_process = evaluate_action_request(
        &ActionRequest::new(ActionType::Process)
            .target("batch process automation run")
            .bulk(true),
    );
    assert_eq!(bulk_process.category, "run_existing_automation");
    assert_ne!(bulk_process.category, "bulk_file_operations");

    let destructive_send = evaluate_action_request(
        &ActionRequest::new(ActionType::Send)
            .target("mass email recipients")
            .bulk(true)
            .destructive(true),
    );
    assert_eq!(destructive_send.category, "send_email");
    assert_eq!(
        destructive_send.human_confirmation,
        HumanConfirmation::Always
    );
    assert_ne!(destructive_send.category, "bulk_file_operations");
    assert_ne!(destructive_send.category, "delete_trash_files");
}

#[test]
fn action_policy_classifier_gates_external_and_integration_creates() {
    let external_create = evaluate_action_request(
        &ActionRequest::new(ActionType::Create)
            .scope(ActionScope::External)
            .target("remote record"),
    );
    assert_eq!(external_create.category, "external_api_write");
    assert_eq!(external_create.policy, ActionPolicy::AskBeforeAction);
    assert!(!external_create.allowed_now);
    assert!(external_create.requires_confirmation);
    assert!(external_create.requires_gate);

    let integration_create = evaluate_action_request(
        &ActionRequest::new(ActionType::Create)
            .scope(ActionScope::Integration)
            .target("contact"),
    );
    assert_eq!(integration_create.category, "external_api_write");
    assert_eq!(integration_create.policy, ActionPolicy::AskBeforeAction);
    assert!(!integration_create.allowed_now);
    assert!(integration_create.requires_confirmation);
    assert!(integration_create.requires_gate);
}

#[test]
fn action_policy_unknown_or_unsafe_requests_fail_closed() {
    let unknown =
        evaluate_action_request(&ActionRequest::new(ActionType::Unknown).target("mystery"));
    assert_eq!(unknown.category, "unknown_action");
    assert_eq!(unknown.policy, ActionPolicy::BlockUntilConfirmed);
    assert!(!unknown.allowed_now);
    assert!(unknown.requires_confirmation);
    assert!(unknown.requires_gate);
    assert!(unknown.reason.to_ascii_lowercase().contains("fail closed"));

    let unsafe_delete = evaluate_action_request(
        &ActionRequest::new(ActionType::Delete)
            .target("unclassified production resource")
            .destructive(true)
            .scope(ActionScope::External),
    );
    assert!(!unsafe_delete.allowed_now);
    assert!(unsafe_delete.requires_confirmation);
    assert!(unsafe_delete.requires_reviewer);
}

fn migrated_confirmation_connection() -> Connection {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_action_policies(&connection).expect("seed action policies");
    connection
}

#[test]
fn confirmation_guard_allows_low_risk_without_record() {
    let request = ActionRequest::new(ActionType::Create)
        .target("private markdown note")
        .scope(ActionScope::LocalPrivate);
    let policy = evaluate_action_request(&request);

    let result = require_policy_clearance_before_execution(&request, Some(&policy), None);

    assert!(
        result.allowed_now,
        "low-risk request should pass: {result:?}"
    );
    assert_eq!(result.reason, "policy_allows_without_confirmation");
}

#[test]
fn confirmation_guard_blocks_send_email_without_approval_and_allows_approved_confirmation() {
    let connection = migrated_confirmation_connection();
    let request = ActionRequest::new(ActionType::Send)
        .target("send email to customer")
        .scope(ActionScope::External)
        .consequence(ActionConsequence::ExternalWrite);
    let policy = evaluate_action_request(&request);

    let blocked = require_policy_clearance_before_execution(&request, Some(&policy), None);
    assert!(!blocked.allowed_now);
    assert_eq!(blocked.reason, "confirmation_required");

    let confirmation = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: &policy.category,
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(Some("user-1")),
            summary: "User approved recipient/body preview",
            event_id: None,
            metadata_json: "{\"preview\":\"recipient and body shown\"}",
        },
    )
    .expect("store approval");

    let allowed =
        require_policy_clearance_before_execution(&request, Some(&policy), Some(&confirmation));
    assert!(
        allowed.allowed_now,
        "approved email should execute: {allowed:?}"
    );
    assert_eq!(allowed.reason, "confirmation_approved");
}

#[test]
fn confirmation_guard_blocks_denied_cancelled_and_expired_decisions() {
    let request = ActionRequest::new(ActionType::Send).target("send email");
    let policy = evaluate_action_request(&request);

    for state in [
        ConfirmationDecisionState::Denied,
        ConfirmationDecisionState::Cancelled,
        ConfirmationDecisionState::Expired,
    ] {
        let decision = ConfirmationDecisionRecord::new_for_test(
            "confirm_state",
            &policy.category,
            state,
            ConfirmationActorType::Human,
        );
        let result =
            require_policy_clearance_before_execution(&request, Some(&policy), Some(&decision));
        assert!(!result.allowed_now, "{state:?} must block");
        assert_eq!(result.reason, format!("confirmation_{}", state.as_str()));
    }
}

#[test]
fn confirmation_guard_fails_closed_for_missing_policy_unknown_request_or_category_mismatch() {
    let known_request = ActionRequest::new(ActionType::Send).target("send email");
    let missing_policy = require_policy_clearance_before_execution(&known_request, None, None);
    assert!(!missing_policy.allowed_now);
    assert_eq!(missing_policy.reason, "missing_policy_decision");

    let unknown_request = ActionRequest::new(ActionType::Unknown).target("mystery");
    let unknown_policy = evaluate_action_request(&unknown_request);
    let unknown_result =
        require_policy_clearance_before_execution(&unknown_request, Some(&unknown_policy), None);
    assert!(!unknown_result.allowed_now);
    assert_eq!(unknown_result.reason, "unknown_action_category");

    let wrong_decision = ConfirmationDecisionRecord::new_for_test(
        "confirm_wrong",
        "create_calendar_event",
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );
    let policy = evaluate_action_request(&known_request);
    let mismatch = require_policy_clearance_before_execution(
        &known_request,
        Some(&policy),
        Some(&wrong_decision),
    );
    assert!(!mismatch.allowed_now);
    assert_eq!(mismatch.reason, "confirmation_category_mismatch");
}

#[test]
fn confirmation_guard_does_not_bypass_reviewer_or_clear_task_requirements() {
    let review_request = ActionRequest::new(ActionType::Publish).target("publish social post");
    let review_policy = evaluate_action_request(&review_request);
    assert!(review_policy.requires_reviewer);
    let human_approval = ConfirmationDecisionRecord::new_for_test(
        "confirm_publish",
        &review_policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );
    let review_result = require_policy_clearance_before_execution(
        &review_request,
        Some(&review_policy),
        Some(&human_approval),
    );
    assert!(!review_result.allowed_now);
    assert_eq!(review_result.reason, "reviewer_required");

    let code_request = ActionRequest::new(ActionType::Update)
        .target("code repository file")
        .scope(ActionScope::CodeRepository);
    let code_policy = evaluate_action_request(&code_request);
    assert!(code_policy.requires_clear_task);
    let generic_human = ConfirmationDecisionRecord::new_for_test(
        "confirm_code",
        &code_policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );
    let code_result = require_policy_clearance_before_execution(
        &code_request,
        Some(&code_policy),
        Some(&generic_human),
    );
    assert!(!code_result.allowed_now);
    assert_eq!(code_result.reason, "clear_task_required");

    let clear_task = ConfirmationDecisionRecord::new_for_test(
        "confirm_code_clear",
        &code_policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::ClearTask,
    );
    let clear_result = require_policy_clearance_before_execution(
        &code_request,
        Some(&code_policy),
        Some(&clear_task),
    );
    assert!(!clear_result.allowed_now);
    assert_eq!(clear_result.reason, "reviewer_required");
}

#[test]
fn confirmation_guard_requires_human_actor_for_human_confirmation() {
    let request = ActionRequest::new(ActionType::Send)
        .target("send email to customer")
        .scope(ActionScope::External)
        .consequence(ActionConsequence::ExternalWrite);
    let policy = evaluate_action_request(&request);
    assert!(matches!(
        policy.human_confirmation,
        HumanConfirmation::Yes | HumanConfirmation::Always
    ));

    let system_approval = ConfirmationDecisionRecord::new_for_test(
        "confirm_system_email",
        &policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::System,
    );
    let result =
        require_policy_clearance_before_execution(&request, Some(&policy), Some(&system_approval));

    assert!(!result.allowed_now);
    assert_eq!(result.reason, "human_confirmation_required");
}

#[test]
fn confirmation_guard_combined_reviewed_clear_task_actor_satisfies_code_policy() {
    let request = ActionRequest::new(ActionType::Update)
        .target("code repository file")
        .scope(ActionScope::CodeRepository);
    let policy = evaluate_action_request(&request);
    assert!(policy.requires_clear_task);
    assert!(policy_requires_hard_reviewer(&policy));

    let plain_human = ConfirmationDecisionRecord::new_for_test(
        "confirm_code_human",
        &policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Human,
    );
    let human_result =
        require_policy_clearance_before_execution(&request, Some(&policy), Some(&plain_human));
    assert!(!human_result.allowed_now);
    assert_eq!(human_result.reason, "clear_task_required");

    let plain_clear_task = ConfirmationDecisionRecord::new_for_test(
        "confirm_code_clear_only",
        &policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::ClearTask,
    );
    let clear_result =
        require_policy_clearance_before_execution(&request, Some(&policy), Some(&plain_clear_task));
    assert!(!clear_result.allowed_now);
    assert_eq!(clear_result.reason, "reviewer_required");

    let plain_reviewer = ConfirmationDecisionRecord::new_for_test(
        "confirm_code_reviewer_only",
        &policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::Reviewer,
    );
    let reviewer_result =
        require_policy_clearance_before_execution(&request, Some(&policy), Some(&plain_reviewer));
    assert!(!reviewer_result.allowed_now);
    assert_eq!(reviewer_result.reason, "clear_task_required");

    let reviewed_clear_task = ConfirmationDecisionRecord::new_for_test(
        "confirm_code_reviewed_clear_task",
        &policy.category,
        ConfirmationDecisionState::Approved,
        ConfirmationActorType::ReviewedClearTask,
    );
    let combined_result = require_policy_clearance_before_execution(
        &request,
        Some(&policy),
        Some(&reviewed_clear_task),
    );
    assert!(
        combined_result.allowed_now,
        "reviewed clear task evidence should satisfy both gates: {combined_result:?}"
    );
    assert_eq!(combined_result.reason, "confirmation_approved");
}

#[test]
fn confirmation_decisions_redact_standalone_secret_material_in_summary_and_metadata() {
    let connection = migrated_confirmation_connection();

    let decision = create_confirmation_decision(
            &connection,
            ConfirmationDecisionRequest {
                action_category: "send_email",
                decision: ConfirmationDecisionState::Approved,
                actor: ConfirmationActor::human(Some("user-1")),
                summary: "approved token sk-test-standalone and ghp_standaloneSecret",
                event_id: None,
                metadata_json: "{\"notes\":[\"bearer standalone-token\",\"safe\"],\"nested\":{\"value\":\"ghp_nestedStandalone\"}}",
            },
        )
        .expect("store redacted decision");

    assert!(decision.summary.contains("[REDACTED]"));
    assert!(!decision.summary.contains("sk-test-standalone"));
    assert!(!decision.summary.contains("ghp_standaloneSecret"));
    assert!(decision.metadata_json.contains("[REDACTED]"));
    assert!(!decision.metadata_json.contains("standalone-token"));
    assert!(!decision.metadata_json.contains("ghp_nestedStandalone"));
    serde_json::from_str::<Value>(&decision.metadata_json)
        .expect("redacted metadata remains valid JSON");
}

#[test]
fn migrations_reject_invalid_confirmation_actor_type() {
    let connection = migrated_confirmation_connection();

    let invalid_actor = connection.execute(
            "insert into confirmation_decisions (id, action_category, decision, actor_type, summary) values ('confirm_invalid_actor', 'send_email', 'approved', 'robot', 'invalid actor')",
            [],
        );

    assert!(
        matches!(invalid_actor, Err(rusqlite::Error::SqliteFailure(_, _))),
        "invalid actor_type must be rejected by schema; got {invalid_actor:?}"
    );
}

fn create_old_v3_confirmation_database(connection: &Connection) {
    connection
            .execute_batch(
                "
                create table schema_migrations (
                    version integer primary key,
                    name text not null,
                    applied_at text not null default current_timestamp
                );

                create table events (
                    id text primary key,
                    type text not null,
                    timestamp text not null default current_timestamp,
                    actor_type text not null,
                    actor_id text,
                    workspace_key text,
                    summary text not null,
                    severity text not null default 'info',
                    source text not null,
                    metadata_json text not null default '{}',
                    created_at text not null default current_timestamp
                );

                create table action_policies (
                    category text primary key,
                    policy text not null check (policy in ('allow', 'ask_before_action', 'block_until_confirmed', 'require_clear_task')),
                    reviewer_required text not null check (reviewer_required in ('none', 'maybe', 'usually', 'yes')),
                    human_confirmation text not null check (human_confirmation in ('none', 'maybe', 'yes', 'always')),
                    reason text not null,
                    created_at text not null default current_timestamp,
                    updated_at text not null default current_timestamp
                );

                create table confirmation_decisions (
                    id text primary key,
                    action_category text not null,
                    decision text not null check (decision in ('approved', 'denied', 'cancelled', 'expired')),
                    actor_type text not null,
                    actor_id text,
                    summary text not null,
                    event_id text,
                    metadata_json text not null default '{}' check (json_valid(metadata_json)),
                    created_at text not null default current_timestamp,
                    foreign key (action_category) references action_policies(category) on update cascade,
                    foreign key (event_id) references events(id) on delete set null
                );
                create index idx_confirmation_decisions_category_created on confirmation_decisions(action_category, created_at);
                create index idx_confirmation_decisions_event on confirmation_decisions(event_id);

                insert into schema_migrations (version, name) values
                    (1, 'foundation_schema'),
                    (2, 'event_schema_backfill'),
                    (3, 'core_schema_p105');
                insert into action_policies (category, policy, reviewer_required, human_confirmation, reason)
                    values ('send_email', 'block_until_confirmed', 'none', 'yes', 'test policy');
                insert into events (id, type, actor_type, summary, source, metadata_json)
                    values ('event_existing', 'confirmation.test', 'system', 'existing event', 'test', '{}');
                insert into confirmation_decisions (id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json)
                    values ('confirm_existing', 'send_email', 'approved', 'human', 'user-1', 'existing approval', 'event_existing', '{}');
                ",
            )
            .expect("create simulated old v3 database without actor_type check");
}

#[test]
fn migrations_upgrade_existing_v3_confirmation_decisions_actor_type_check() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    enable_sqlite_foreign_keys(&connection).expect("enable foreign keys");
    create_old_v3_confirmation_database(&connection);

    run_migrations(&connection).expect("upgrade old v3 database");

    let existing_actor: String = connection
        .query_row(
            "select actor_type from confirmation_decisions where id = 'confirm_existing'",
            [],
            |row| row.get(0),
        )
        .expect("valid existing row preserved");
    assert_eq!(existing_actor, "human");
    let event_id: Option<String> = connection
        .query_row(
            "select event_id from confirmation_decisions where id = 'confirm_existing'",
            [],
            |row| row.get(0),
        )
        .expect("event foreign key preserved");
    assert_eq!(event_id.as_deref(), Some("event_existing"));

    let invalid_actor = connection.execute(
            "insert into confirmation_decisions (id, action_category, decision, actor_type, summary) values ('confirm_invalid_after_upgrade', 'send_email', 'approved', 'robot', 'invalid actor')",
            [],
        );
    assert!(
        matches!(invalid_actor, Err(rusqlite::Error::SqliteFailure(_, _))),
        "upgraded old v3 schema must reject invalid actor_type; got {invalid_actor:?}"
    );
}

#[test]
fn migration_v4_sql_is_transactional_and_does_not_pre_drop_upgrade_table() {
    let sql = include_str!("../migrations/0004_confirmation_actor_type_check.sql");
    let normalized = sql.trim_start().to_ascii_lowercase();

    assert!(
        normalized.starts_with("begin immediate;"),
        "migration v4 must start an explicit transaction before any rebuild DDL"
    );
    assert!(
        normalized.trim_end().ends_with("commit;"),
        "migration v4 must commit its explicit transaction"
    );
    assert!(
        !normalized.contains("drop table if exists confirmation_decisions_actor_type_upgrade"),
        "migration v4 must not drop a leftover upgrade table before preserving data"
    );
}

#[test]
fn migration_v4_fails_closed_when_leftover_upgrade_table_exists() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    enable_sqlite_foreign_keys(&connection).expect("enable foreign keys");
    create_old_v3_confirmation_database(&connection);
    connection
        .execute_batch(
            "
                create table confirmation_decisions_actor_type_upgrade (
                    id text primary key,
                    marker text not null
                );
                insert into confirmation_decisions_actor_type_upgrade (id, marker)
                    values ('leftover_preserved', 'do-not-drop');
                ",
        )
        .expect("create simulated leftover upgrade table");

    let error = run_migrations(&connection).expect_err("leftover upgrade table must block v4");
    assert!(
        error
            .to_string()
            .contains("confirmation_decisions_actor_type_upgrade"),
        "expected existing upgrade table to block migration; got {error:?}"
    );

    let preserved_decision: String = connection
        .query_row(
            "select summary from confirmation_decisions where id = 'confirm_existing'",
            [],
            |row| row.get(0),
        )
        .expect("original confirmation_decisions row must remain");
    assert_eq!(preserved_decision, "existing approval");

    let preserved_leftover: String = connection
            .query_row(
                "select marker from confirmation_decisions_actor_type_upgrade where id = 'leftover_preserved'",
                [],
                |row| row.get(0),
            )
            .expect("leftover upgrade table must not be dropped");
    assert_eq!(preserved_leftover, "do-not-drop");
    assert_eq!(get_migration_version(&connection).unwrap(), 3);
}

#[test]
fn confirmation_decisions_redact_validate_event_link_and_list_in_newest_order() {
    let connection = migrated_confirmation_connection();
    let event_id = write_event(
        &connection,
        EventInput {
            event_type: "confirmation.test",
            actor_type: "system",
            actor_id: None,
            workspace_key: Some("today"),
            summary: "confirmation event",
            severity: "info",
            source: "test",
            metadata_json: "{}",
            targets: vec![],
        },
    )
    .expect("write event");

    let first = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "send_email",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(Some("user-1")),
            summary: "approved with api_key=raw-secret",
            event_id: Some(&event_id),
            metadata_json: "{\"token\":\"raw-token\",\"note\":\"safe\"}",
        },
    )
    .expect("store first decision");
    let second = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "send_email",
            decision: ConfirmationDecisionState::Denied,
            actor: ConfirmationActor::reviewer(Some("reviewer-1")),
            summary: "denied later",
            event_id: None,
            metadata_json: "{\"rank\":2}",
        },
    )
    .expect("store second decision");

    assert_eq!(first.event_id.as_deref(), Some(event_id.as_str()));
    assert!(first.summary.contains("[REDACTED]"));
    assert!(!first.summary.contains("raw-secret"));
    assert!(first.metadata_json.contains("[REDACTED]"));
    assert!(!first.metadata_json.contains("raw-token"));

    let read_back = read_confirmation_decision(&connection, &first.id)
        .expect("read result")
        .expect("stored decision");
    assert_eq!(read_back, first);

    let listed =
        list_confirmation_decisions(&connection, Some("send_email"), 10).expect("list decisions");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].id, second.id);
    assert_eq!(listed[1].id, first.id);

    let invalid_json = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "send_email",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(None),
            summary: "bad json",
            event_id: None,
            metadata_json: "not json",
        },
    )
    .expect_err("invalid metadata json should fail before persistence");
    assert!(matches!(
        invalid_json,
        RepositoryError::InvalidJson {
            field: "metadata_json",
            ..
        }
    ));

    let unknown_category = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "not_a_policy",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(None),
            summary: "bad category",
            event_id: None,
            metadata_json: "{}",
        },
    )
    .expect_err("unknown policy category should fail");
    assert!(matches!(
        unknown_category,
        RepositoryError::Constraint {
            entity: "confirmation_decisions",
            ..
        }
    ));

    let missing_event = create_confirmation_decision(
        &connection,
        ConfirmationDecisionRequest {
            action_category: "send_email",
            decision: ConfirmationDecisionState::Approved,
            actor: ConfirmationActor::human(None),
            summary: "bad event",
            event_id: Some("evt_missing"),
            metadata_json: "{}",
        },
    )
    .expect_err("missing event link should fail");
    assert!(matches!(
        missing_event,
        RepositoryError::Constraint {
            entity: "confirmation_decisions",
            ..
        }
    ));
}

#[test]
fn seeded_action_policy_rows_match_evaluator_single_source_of_truth() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    seed_action_policies(&connection).expect("idempotent reseed");

    for category in ACTION_POLICY_CATEGORIES {
        let decision = evaluate_action_policy(category);
        let row: (String, String, String, String) = connection
                .query_row(
                    "select policy, reviewer_required, human_confirmation, reason from action_policies where category = ?1",
                    params![category],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .expect("seeded action policy row");
        assert_eq!(row.0, action_policy_as_str(decision.policy));
        assert_eq!(
            row.1,
            reviewer_requirement_as_str(decision.reviewer_required)
        );
        assert_eq!(
            row.2,
            human_confirmation_as_str(decision.human_confirmation)
        );
        assert_eq!(row.3, decision.reason);
    }
}

#[test]
fn action_policy_matrix_matches_documented_consequential_actions() {
    let cases = [
        (
            "read_local_app_data",
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
        ),
        (
            "read_gmail_calendar",
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
        ),
        (
            "create_local_task",
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
        ),
        (
            "create_private_markdown_note",
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
        ),
        (
            "import_migrate_data",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Usually,
            HumanConfirmation::Yes,
        ),
        (
            "modify_visible_non_code_file",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Maybe,
        ),
        (
            "move_rename_copy_file",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::None,
            HumanConfirmation::Yes,
        ),
        (
            "bulk_file_operations",
            ActionPolicy::BlockUntilConfirmed,
            ReviewerRequirement::Yes,
            HumanConfirmation::Yes,
        ),
        (
            "delete_trash_files",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Yes,
        ),
        (
            "modify_code_repo_files",
            ActionPolicy::RequireClearTask,
            ReviewerRequirement::Yes,
            HumanConfirmation::None,
        ),
        (
            "commit_push_merge",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Yes,
            HumanConfirmation::Yes,
        ),
        (
            "deploy_redeploy_rollback",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Yes,
            HumanConfirmation::Yes,
        ),
        (
            "send_email",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Always,
        ),
        (
            "publish_schedule_content",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Yes,
            HumanConfirmation::Maybe,
        ),
        (
            "change_automation_schedule",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Yes,
        ),
        (
            "run_existing_automation",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Maybe,
        ),
        (
            "change_credentials_settings_integrations",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Always,
        ),
        (
            "create_calendar_event",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::None,
            HumanConfirmation::Yes,
        ),
        (
            "edit_delete_calendar_event",
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Always,
        ),
    ];

    for (category, policy, reviewer, confirmation) in cases {
        let decision = evaluate_action_policy(category);
        assert_eq!(decision.policy, policy, "policy mismatch for {category}");
        assert_eq!(
            decision.reviewer_required, reviewer,
            "reviewer mismatch for {category}"
        );
        assert_eq!(
            decision.human_confirmation, confirmation,
            "confirmation mismatch for {category}"
        );
    }

    let unknown = evaluate_action_policy("unclassified risky action");
    assert_eq!(unknown.policy, ActionPolicy::BlockUntilConfirmed);
    assert_eq!(unknown.human_confirmation, HumanConfirmation::Yes);
}

#[test]
fn generic_event_writer_redacts_and_links_targets() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    let event_id = write_event(
        &connection,
        EventInput {
            event_type: "agent.run.completed",
            actor_type: "agent",
            actor_id: Some("local-agent"),
            workspace_key: Some("agents"),
            summary: "Finished with token=abc123",
            severity: "info",
            source: "agent_service",
            metadata_json: "{\"api_key\":\"abc123\"}",
            targets: vec![
                ("workspace", "agents", "primary"),
                ("task", "task_1", "result"),
            ],
        },
    )
    .expect("write generic event");

    let fields: (String, String) = connection
        .query_row(
            "select summary, metadata_json from events where id = ?1",
            params![event_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(!fields.0.contains("abc123"));
    assert!(!fields.1.contains("abc123"));
    assert!(fields.0.contains("[REDACTED]"));

    let parsed_metadata: Value =
        serde_json::from_str(&fields.1).expect("metadata stays valid JSON");
    assert_eq!(parsed_metadata["api_key"], "[REDACTED]");
    let json_valid: i64 = connection
        .query_row("select json_valid(?1)", params![fields.1], |row| row.get(0))
        .unwrap();
    assert_eq!(json_valid, 1);

    let target_count: i64 = connection
        .query_row("select count(*) from event_targets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(target_count, 2);
}

#[test]
fn event_repository_writes_reads_and_lists_redacted_events_with_ordered_targets() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let older = create_event_record(
        &connection,
        EventCreateInput {
            action_type: "agent.run.started",
            outcome: "started",
            actor_type: "agent",
            actor_id: Some("local-agent"),
            workspace_key: Some("agents"),
            summary: "Started with token=sk-live-123",
            source: "agent_service",
            metadata_json: "{\"token\":\"abc123\",\"visible\":\"kept\"}",
            targets: vec![
                EventTargetInput {
                    entity_type: "task",
                    entity_id: "task_2",
                    relation_type: "secondary",
                },
                EventTargetInput {
                    entity_type: "workspace",
                    entity_id: "agents",
                    relation_type: "primary",
                },
            ],
        },
    )
    .expect("create older event");
    let newer = create_event_record(
        &connection,
        EventCreateInput {
            action_type: "agent.run.completed",
            outcome: "success",
            actor_type: "agent",
            actor_id: Some("local-agent"),
            workspace_key: Some("agents"),
            summary: "Completed safely",
            source: "agent_service",
            metadata_json: "{\"duration_ms\":15}",
            targets: vec![EventTargetInput {
                entity_type: "task",
                entity_id: "task_2",
                relation_type: "result",
            }],
        },
    )
    .expect("create newer event");

    assert_eq!(older.action_type, "agent.run.started");
    assert_eq!(older.outcome, "started");
    assert!(!older.summary.contains("sk-live-123"));
    assert_eq!(
        older.metadata_json,
        "{\"token\":\"[REDACTED]\",\"visible\":\"kept\"}"
    );
    assert_eq!(
        older.targets,
        vec![
            EventTargetRecord {
                entity_type: "workspace".to_string(),
                entity_id: "agents".to_string(),
                relation_type: "primary".to_string(),
            },
            EventTargetRecord {
                entity_type: "task".to_string(),
                entity_id: "task_2".to_string(),
                relation_type: "secondary".to_string(),
            },
        ]
    );

    let read_back = read_event_record(&connection, &older.id).expect("read event");
    assert_eq!(read_back, older);

    let newest_only = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: None,
            outcome: None,
            source: None,
            limit: 1,
        },
    )
    .expect("list newest event");
    assert_eq!(newest_only, vec![newer.clone()]);

    let listed = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: Some("agent.run.completed"),
            outcome: Some("success"),
            source: Some("agent_service"),
            limit: 10,
        },
    )
    .expect("list events");
    assert_eq!(listed, vec![newer]);
}

#[test]
fn event_repository_rolls_back_event_when_target_insert_fails() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");
    connection
        .execute_batch(
            "
                create trigger force_event_target_failure
                before insert on event_targets
                begin
                    select raise(abort, 'forced target failure');
                end;
                ",
        )
        .expect("install failing target trigger");

    let result = create_event_record(
        &connection,
        EventCreateInput {
            action_type: "agent.run.completed",
            outcome: "success",
            actor_type: "agent",
            actor_id: Some("local-agent"),
            workspace_key: Some("agents"),
            summary: "target should fail",
            source: "agent_service",
            metadata_json: "{\"duration_ms\":15}",
            targets: vec![EventTargetInput {
                entity_type: "task",
                entity_id: "task_rollback",
                relation_type: "result",
            }],
        },
    );

    assert!(result.is_err());
    assert_eq!(count_table(&connection, "events").unwrap(), 0);
    let target_count: i64 = connection
        .query_row("select count(*) from event_targets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(target_count, 0);
}

#[test]
fn event_repository_lists_more_than_ten_rapid_events_newest_insertion_first() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let mut created = Vec::new();
    for index in 0..12 {
        created.push(
            create_event_record(
                &connection,
                EventCreateInput {
                    action_type: "agent.run.progress",
                    outcome: "info",
                    actor_type: "agent",
                    actor_id: Some("local-agent"),
                    workspace_key: Some("agents"),
                    summary: "rapid event",
                    source: "agent_service",
                    metadata_json: "{}",
                    targets: vec![EventTargetInput {
                        entity_type: "task",
                        entity_id: if index % 2 == 0 {
                            "task_even"
                        } else {
                            "task_odd"
                        },
                        relation_type: "progress",
                    }],
                },
            )
            .expect("create rapid event"),
        );
    }

    let listed = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: Some("agent.run.progress"),
            outcome: Some("info"),
            source: Some("agent_service"),
            limit: 12,
        },
    )
    .expect("list rapid events");
    let listed_ids = listed.iter().map(|event| &event.id).collect::<Vec<_>>();
    let expected_ids = created
        .iter()
        .rev()
        .map(|event| &event.id)
        .collect::<Vec<_>>();
    assert_eq!(listed_ids, expected_ids);
}

#[test]
fn event_repository_rejects_invalid_metadata_before_insert_or_target_insert() {
    let connection = Connection::open_in_memory().expect("open in-memory sqlite");
    run_migrations(&connection).expect("run migrations");

    let result = create_event_record(
        &connection,
        EventCreateInput {
            action_type: "agent.run.completed",
            outcome: "success",
            actor_type: "agent",
            actor_id: Some("local-agent"),
            workspace_key: Some("agents"),
            summary: "bad metadata",
            source: "agent_service",
            metadata_json: "{not valid json",
            targets: vec![EventTargetInput {
                entity_type: "task",
                entity_id: "task_3",
                relation_type: "result",
            }],
        },
    );

    assert!(matches!(
        result,
        Err(RepositoryError::InvalidJson {
            field: "metadata_json",
            ..
        })
    ));
    assert_eq!(count_table(&connection, "events").unwrap(), 0);
    let target_count: i64 = connection
        .query_row("select count(*) from event_targets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(target_count, 0);
}

#[test]
fn p207_history_service_returns_related_task_timeline_without_unrelated_events() {
    let connection = migrated_in_memory_connection();
    let task = create_task_service(
        &connection,
        TaskServiceCreateInput {
            title: "History task".to_string(),
            detail: Some("Trace related timeline".to_string()),
            priority: Some(TaskPriority::High),
            workspace_key: Some("today".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create task through service");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id,
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run");
    let review = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Approved".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create review");
    let notification = create_notification_service(
        &connection,
        NotificationServiceCreateInput {
            notification_type: NotificationType::Completion,
            title: "Done".to_string(),
            message: "Run completed".to_string(),
            severity: NotificationSeverity::Success,
            action_route: Some(format!("/tasks/{}", task.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create notification");
    let unrelated = create_task_service(&connection, TaskServiceCreateInput::new("Unrelated task"))
        .expect("create unrelated task");

    let history = list_task_history(&connection, &task.id, 50, None).expect("task history");
    let actions = history
        .iter()
        .map(|item| item.event.action_type.as_str())
        .collect::<Vec<_>>();

    assert!(actions.contains(&"task.created"));
    assert!(actions.contains(&"run.queued"));
    assert!(actions.contains(&"review.created"));
    assert!(actions.contains(&"review.approved"));
    assert!(actions.contains(&"notification.created"));
    assert!(history.iter().all(|item| {
        !item
            .event
            .targets
            .iter()
            .any(|target| target.entity_id == unrelated.id)
    }));
    assert!(history.iter().any(|item| {
        item.matched_entities.iter().any(|target| {
            target.entity_type == "notification" && target.entity_id == notification.id
        })
    }));
}

#[test]
fn p207_history_service_paginates_deterministically_and_omits_raw_log_bodies() {
    let connection = migrated_in_memory_connection();
    let task = create_task_service(&connection, TaskServiceCreateInput::new("Paged history"))
        .expect("create task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id,
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run");
    let log_reference_id = p204_log_reference(&connection);
    complete_agent_run(
        &connection,
        &run.id,
        AgentRunCompletionInput {
            status: AgentRunStatus::Completed,
            duration_ms: 15,
            exit_code: Some(0),
            log_reference_id: Some(log_reference_id),
            output_summary: "Summarized output only".to_string(),
            error_summary: None,
            review_state: ReviewState::NotRequired,
            metadata_json: "{\"log_reference_path\":\"agent-run-p207.log\"}".to_string(),
        },
    )
    .expect("complete run");

    let first_page = list_run_history(&connection, &run.id, 2, None).expect("first page");
    assert_eq!(first_page.len(), 2);
    let cursor = HistoryCursor {
        timestamp: first_page.last().unwrap().event.timestamp.clone(),
        event_id: first_page.last().unwrap().event.id.clone(),
    };
    let second_page =
        list_run_history(&connection, &run.id, 10, Some(cursor)).expect("second page");
    assert!(second_page.iter().all(|item| !first_page
        .iter()
        .any(|first| first.event.id == item.event.id)));
    for item in first_page.iter().chain(second_page.iter()) {
        assert!(!item.event.summary.contains("RAW_SECRET_SHOULD_BE_REDACTED"));
        assert!(!item
            .event
            .metadata_json
            .contains("RAW_SECRET_SHOULD_BE_REDACTED"));
    }
}

#[test]
fn p207_run_history_excludes_sibling_run_events_on_same_task() {
    let connection = migrated_in_memory_connection();
    let task = create_task_service(&connection, TaskServiceCreateInput::new("Sibling runs"))
        .expect("create task");
    let profile = p204_profile(&connection, true);
    let session_a = p204_session(&connection, &task, &profile);
    let run_a = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            session_id: session_a.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run a");
    let session_b = p204_session(&connection, &task, &profile);
    let run_b = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id,
            session_id: session_b.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run b");
    let review_b = create_review_record(
        &connection,
        ReviewRecordCreateInput {
            subject_type: ReviewSubjectType::AgentRun,
            subject_id: run_b.id.clone(),
            task_id: task.id.clone(),
            run_id: Some(run_b.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "Sibling approved".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create sibling review");
    let notification_b = create_notification_service(
        &connection,
        NotificationServiceCreateInput {
            notification_type: NotificationType::Completion,
            title: "Sibling done".to_string(),
            message: "Sibling run completed".to_string(),
            severity: NotificationSeverity::Success,
            action_route: Some(format!("/runs/{}", run_b.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run_b.id.clone()),
            review_record_id: Some(review_b.id.clone()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create sibling notification");

    let history = list_run_history(&connection, &run_a.id, 50, None).expect("run a history");
    assert!(history.iter().any(|item| {
        item.matched_entities
            .iter()
            .any(|target| target.entity_type == "agent_run" && target.entity_id == run_a.id)
    }));
    assert!(history.iter().all(|item| {
        !item.event.targets.iter().any(|target| {
            target.entity_id == run_b.id
                || target.entity_id == review_b.id
                || target.entity_id == notification_b.id
        })
    }));
}

#[test]
fn p217_task_bridge_commands_create_read_list_update_archive_delete_and_write_events() {
    let connection = migrated_in_memory_connection();

    let created = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: " Bridge task ".to_string(),
            detail: Some(" Bridge detail ".to_string()),
            priority: Some("urgent".to_string()),
            workspace_key: Some("today".to_string()),
            metadata_json: Some("{\"source\":\"tauri_bridge\"}".to_string()),
        },
    )
    .expect("create task via bridge helper");
    assert_eq!(created.title, "Bridge task");
    assert_eq!(created.detail.as_deref(), Some("Bridge detail"));
    assert_eq!(created.priority, TaskPriority::Urgent);

    let listed =
        list_tasks_command_with_connection(&connection).expect("list tasks via bridge helper");
    assert!(listed.iter().any(|task| task.id == created.id));

    let updated = update_task_command_with_connection(
        &connection,
        created.id.clone(),
        TaskCommandUpdateRequest {
            title: Some("Updated bridge task".to_string()),
            detail: Some("Updated bridge detail".to_string()),
            priority: Some("high".to_string()),
            workspace_key: None,
            metadata_json: Some("{\"source\":\"bridge_update\"}".to_string()),
        },
    )
    .expect("update task via bridge helper");
    assert_eq!(updated.title, "Updated bridge task");
    assert_eq!(updated.priority, TaskPriority::High);

    let active = update_task_status_command_with_connection(
        &connection,
        created.id.clone(),
        TaskCommandStatusRequest {
            status: "active".to_string(),
        },
    )
    .expect("update status via bridge helper");
    assert_eq!(active.status, TaskStatus::Active);

    let detail = read_task_command_with_connection(&connection, created.id.clone())
        .expect("read task via bridge helper");
    assert_eq!(detail.id, created.id);

    let events = list_task_history(&connection, &created.id, 20, None).expect("task history");
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "task.created"));
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "task.updated"));
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "task.status_changed"));

    let archived = archive_task_command_with_connection(&connection, created.id.clone())
        .expect("archive task via bridge helper");
    assert_eq!(archived.status, TaskStatus::Archived);
    let active_after_archive = list_tasks_command_with_connection(&connection)
        .expect("list after archive via bridge helper");
    assert!(active_after_archive
        .iter()
        .all(|task| task.id != created.id));

    let deleted = delete_task_command_with_connection(&connection, created.id.clone())
        .expect("delete task via bridge helper");
    assert_eq!(deleted.status, TaskStatus::Deleted);
}

#[test]
fn p217_task_bridge_commands_preserve_validation_and_secret_guards() {
    let connection = migrated_in_memory_connection();

    let invalid_priority = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: "Bad priority".to_string(),
            detail: None,
            priority: Some("critical".to_string()),
            workspace_key: None,
            metadata_json: None,
        },
    )
    .expect_err("invalid priority must fail before persistence");
    assert!(invalid_priority.contains("invalid task priority"));
    assert_eq!(count_rows(&connection, "select count(*) from tasks"), 0);

    let secret_metadata = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: "Secret task".to_string(),
            detail: None,
            priority: None,
            workspace_key: None,
            metadata_json: Some("{\"api_key\":\"super-secret\"}".to_string()),
        },
    )
    .expect_err("secret metadata must be rejected");
    assert!(secret_metadata.contains("secret-like key"));
    assert_eq!(count_rows(&connection, "select count(*) from tasks"), 0);

    let task = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: "Valid task".to_string(),
            detail: None,
            priority: None,
            workspace_key: None,
            metadata_json: None,
        },
    )
    .expect("create valid task");
    let invalid_status = update_task_status_command_with_connection(
        &connection,
        task.id,
        TaskCommandStatusRequest {
            status: "doneish".to_string(),
        },
    )
    .expect_err("invalid status must fail");
    assert!(invalid_status.contains("invalid task status"));
}

#[test]
fn p228_task_service_persists_tasks_and_task_events_after_reopen() {
    let (connection, database_path) = migrated_file_connection("p228-task-persistence-db");

    let active = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: "Persisted active task".to_string(),
            detail: Some("Initial detail".to_string()),
            priority: Some("normal".to_string()),
            workspace_key: Some("today".to_string()),
            metadata_json: Some("{\"source\":\"p228\"}".to_string()),
        },
    )
    .expect("create active task");
    let archived = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: "Persisted archived task".to_string(),
            detail: None,
            priority: Some("low".to_string()),
            workspace_key: Some("tasks".to_string()),
            metadata_json: None,
        },
    )
    .expect("create archived task");
    let deleted = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: "Persisted deleted task".to_string(),
            detail: None,
            priority: Some("high".to_string()),
            workspace_key: Some("tomorrow".to_string()),
            metadata_json: None,
        },
    )
    .expect("create deleted task");

    let active = update_task_command_with_connection(
        &connection,
        active.id.clone(),
        TaskCommandUpdateRequest {
            title: Some("Persisted updated active task".to_string()),
            detail: Some("Updated detail".to_string()),
            priority: Some("urgent".to_string()),
            workspace_key: Some("today".to_string()),
            metadata_json: Some("{\"source\":\"p228_update\"}".to_string()),
        },
    )
    .expect("update active task");
    update_task_status_command_with_connection(
        &connection,
        active.id.clone(),
        TaskCommandStatusRequest {
            status: "active".to_string(),
        },
    )
    .expect("activate updated task");
    archive_task_command_with_connection(&connection, archived.id.clone()).expect("archive task");
    delete_task_command_with_connection(&connection, deleted.id.clone()).expect("delete task");

    let pre_reopen_active_count = list_tasks_command_with_connection(&connection)
        .expect("list before reopen")
        .len();
    assert_eq!(pre_reopen_active_count, 1);
    drop(connection);

    let reopened = open_foundation_database(&database_path).expect("reopen file-backed sqlite");
    run_migrations(&reopened).expect("rerun migrations after reopen");

    let reopened_active = read_task_command_with_connection(&reopened, active.id.clone())
        .expect("read active task after reopen");
    assert_eq!(reopened_active.title, "Persisted updated active task");
    assert_eq!(reopened_active.detail.as_deref(), Some("Updated detail"));
    assert_eq!(reopened_active.priority, TaskPriority::Urgent);
    assert_eq!(reopened_active.status, TaskStatus::Active);
    assert_eq!(reopened_active.workspace_key, "today");
    assert_eq!(
        reopened_active.metadata_json,
        "{\"source\":\"p228_update\"}"
    );

    let reopened_archived = read_task_command_with_connection(&reopened, archived.id.clone())
        .expect("read archived task after reopen");
    assert_eq!(reopened_archived.status, TaskStatus::Archived);
    assert!(reopened_archived.archived_at.is_some());

    let reopened_deleted = read_task_command_with_connection(&reopened, deleted.id.clone())
        .expect("read deleted task after reopen");
    assert_eq!(reopened_deleted.status, TaskStatus::Deleted);
    assert!(reopened_deleted.deleted_at.is_some());

    let active_after_reopen =
        list_tasks_command_with_connection(&reopened).expect("list after reopen");
    assert_eq!(active_after_reopen.len(), 1);
    assert_eq!(active_after_reopen[0].id, active.id);
    assert!(active_after_reopen
        .iter()
        .all(|task| task.id != archived.id && task.id != deleted.id));

    let active_history = list_task_history(&reopened, &active.id, 50, None)
        .expect("active task history after reopen");
    for action_type in ["task.created", "task.updated", "task.status_changed"] {
        let item = active_history
            .iter()
            .find(|item| item.event.action_type == action_type)
            .unwrap_or_else(|| panic!("missing persisted active task event {action_type}"));
        assert_eq!(item.event.workspace_key.as_deref(), Some("today"));
        assert_eq!(item.event.outcome, "succeeded");
        assert!(matches!(
            item.event.source.as_str(),
            "task_repository" | "task_service"
        ));
        assert!(item.event.targets.iter().any(|target| {
            target.entity_type == "task"
                && target.entity_id == active.id
                && target.relation_type == "primary"
        }));
        assert!(item.matched_entities.iter().any(|target| {
            target.entity_type == "task"
                && target.entity_id == active.id
                && target.relation_type == "primary"
        }));
    }
    let archived_history = list_task_history(&reopened, &archived.id, 50, None)
        .expect("archived task history after reopen");
    let archived_event = archived_history
        .iter()
        .find(|item| item.event.action_type == "task.archived")
        .expect("archived task history event");
    assert_eq!(archived_event.event.workspace_key.as_deref(), Some("tasks"));
    assert_eq!(archived_event.event.outcome, "succeeded");
    assert_eq!(archived_event.event.source, "task_repository");
    assert!(archived_event.event.targets.iter().any(|target| {
        target.entity_type == "task"
            && target.entity_id == archived.id
            && target.relation_type == "primary"
    }));
    let deleted_history = list_task_history(&reopened, &deleted.id, 50, None)
        .expect("deleted task history after reopen");
    let deleted_event = deleted_history
        .iter()
        .find(|item| item.event.action_type == "task.deleted")
        .expect("deleted task history event");
    assert_eq!(
        deleted_event.event.workspace_key.as_deref(),
        Some("tomorrow")
    );
    assert_eq!(deleted_event.event.outcome, "succeeded");
    assert_eq!(deleted_event.event.source, "task_repository");
    assert!(deleted_event.event.targets.iter().any(|target| {
        target.entity_type == "task"
            && target.entity_id == deleted.id
            && target.relation_type == "primary"
    }));
}

#[test]
fn p218_run_bridge_commands_start_status_stream_and_write_events() {
    let (connection, _database_path) = migrated_file_connection("p218-run-bridge-db");
    let task = p204_task(&connection, "Bridge run task");
    let profile = p209_profile_with_command(&connection, "profile-shell-p218", "/bin/sh", true);
    let logs_dir = temp_home("p218-run-bridge-logs");

    let outcome = start_agent_run_command_with_connection(
        &connection,
        AgentRunCommandStartRequest {
            task_id: task.id.clone(),
            profile_id: profile.id,
            cwd: "/tmp".to_string(),
            argv: vec![
                "-c".to_string(),
                "printf 'bridge stdout'; sleep 0.25; printf 'token=sk-liv...cret' >&2".to_string(),
            ],
            stdin: None,
            timeout_ms: None,
            logs_dir: logs_dir.clone(),
            metadata_json: Some("{\"source\":\"p218_bridge\"}".to_string()),
        },
    )
    .expect("start run via bridge helper");

    assert_eq!(outcome.run.status, AgentRunStatus::Running);
    assert_eq!(outcome.run.task_id, task.id);
    assert!(outcome
        .log_path
        .ends_with(&format!("{}.log", outcome.run.id)));

    let running_status =
        read_run_status_command_with_connection(&connection, outcome.run.id.clone())
            .expect("read running status via bridge helper");
    assert_eq!(running_status.status, AgentRunStatus::Running);

    let mut stream = None;
    for _ in 0..30 {
        let chunk = stream_run_output_command_with_connection(
            &connection,
            AgentRunCommandStreamRequest {
                run_id: outcome.run.id.clone(),
                logs_dir: logs_dir.clone(),
                offset: Some(0),
                max_bytes: Some(1024),
            },
        )
        .expect("stream run output via bridge helper");
        if chunk.content.contains("bridge stdout") {
            stream = Some(chunk);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let stream = stream.expect("stream should expose partial stdout before process exit");
    assert_eq!(stream.run_id, outcome.run.id);
    assert_eq!(stream.status, AgentRunStatus::Running);
    assert!(!stream.eof);
    assert!(stream.next_offset > stream.offset);

    let mut completed = None;
    for _ in 0..40 {
        let status = read_run_status_command_with_connection(&connection, outcome.run.id.clone())
            .expect("read eventual status via bridge helper");
        if status.status == AgentRunStatus::Completed {
            completed = Some(status);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let completed = completed.expect("run should complete asynchronously");
    assert!(completed.log_reference_id.is_some());

    let final_stream = stream_run_output_command_with_connection(
        &connection,
        AgentRunCommandStreamRequest {
            run_id: outcome.run.id.clone(),
            logs_dir,
            offset: Some(0),
            max_bytes: Some(4096),
        },
    )
    .expect("stream final run output via bridge helper");
    assert!(final_stream.content.contains("bridge stdout"));
    assert!(final_stream.content.contains("[REDACTED]"));
    assert!(!final_stream.content.contains("sk-liv...cret"));
    assert!(final_stream.eof);

    let events = list_run_history(&connection, &outcome.run.id, 20, None).expect("run history");
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "run.queued"));
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "run.started"));
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "run.completed"));
}

#[test]
fn p229_run_bridge_records_failed_exit_code_log_notification_and_redacted_stream() {
    let (connection, _database_path) = migrated_file_connection("p229-failed-run-bridge-db");
    let task = p204_task(&connection, "P2.29 failing bridge run task");
    let profile =
        p209_profile_with_command(&connection, "profile-shell-fail-p229", "/bin/sh", true);
    let logs_dir = temp_home("p229-failed-run-bridge-logs");
    let stdout_secret = "dummy-credential-1";
    let stderr_message = "plain failure evidence";

    let outcome = start_agent_run_command_with_connection(
        &connection,
        AgentRunCommandStartRequest {
            task_id: task.id.clone(),
            profile_id: profile.id,
            cwd: "/tmp".to_string(),
            argv: vec![
                "-c".to_string(),
                [
                    "printf 'stdout api_key=",
                    stdout_secret,
                    "\\n'; printf 'stderr ",
                    stderr_message,
                    "\\n' >&2; exit 9",
                ]
                .concat(),
            ],
            stdin: None,
            timeout_ms: None,
            logs_dir: logs_dir.clone(),
            metadata_json: Some("{\"source\":\"p229_bridge\"}".to_string()),
        },
    )
    .expect("start failing run via bridge helper");
    assert_eq!(outcome.run.status, AgentRunStatus::Running);

    let mut failed = None;
    for _ in 0..80 {
        let status = read_run_status_command_with_connection(&connection, outcome.run.id.clone())
            .expect("read eventual failed status");
        if status.status == AgentRunStatus::Failed {
            failed = Some(status);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let failed = failed.expect("run should fail asynchronously");
    assert_eq!(failed.exit_code, Some(9));
    assert!(failed.log_reference_id.is_some());
    assert!(failed
        .error_summary
        .as_deref()
        .unwrap_or_default()
        .contains(stderr_message));
    assert!(!failed
        .error_summary
        .as_deref()
        .unwrap_or_default()
        .contains(stdout_secret));

    let stream = stream_run_output_command_with_connection(
        &connection,
        AgentRunCommandStreamRequest {
            run_id: outcome.run.id.clone(),
            logs_dir: logs_dir.clone(),
            offset: Some(0),
            max_bytes: Some(4096),
        },
    )
    .expect("stream failed run output");
    assert_eq!(stream.status, AgentRunStatus::Failed);
    assert!(stream.eof);
    assert!(stream.content.contains("stdout api_key"));
    assert!(stream.content.contains("[REDACTED]"));
    assert!(stream.content.contains(stderr_message));
    assert!(!stream.content.contains(stdout_secret));
    assert!(stream.log_reference_id.starts_with("logref_"));

    let log_path = logs_dir.join(format!("{}.log", outcome.run.id));
    let persisted_log = fs::read_to_string(&log_path).expect("read persisted failed run log");
    assert!(persisted_log.contains("stdout api_key"));
    assert!(persisted_log.contains("[REDACTED]"));
    assert!(persisted_log.contains(stderr_message));
    assert!(!persisted_log.contains(stdout_secret));

    let events = list_run_history(&connection, &outcome.run.id, 20, None).expect("run history");
    assert!(events
        .iter()
        .any(|item| item.event.action_type == "run.failed"
            && item.event.targets.iter().any(|target| {
                target.entity_type == "agent_run" && target.entity_id == outcome.run.id
            })));
    let serialized_events = serde_json::to_string(&events).expect("serialize run history");
    assert!(!serialized_events.contains(stdout_secret));

    let notifications = list_inbox_notifications(&connection, false, 20).expect("notifications");
    assert!(notifications.iter().any(|notification| {
        notification.run_id.as_deref() == Some(outcome.run.id.as_str())
            && notification.notification_type == NotificationType::Failure
            && notification.severity == NotificationSeverity::Error
            && notification.title == "Agent run failed"
    }));
    let serialized_notifications =
        serde_json::to_string(&notifications).expect("serialize failure notifications");
    assert!(!serialized_notifications.contains(stdout_secret));

    let sqlite_secret_count = count_rows(
        &connection,
        &format!(
            "select \
                (select count(*) from agent_runs where id = '{}' and (coalesce(output_summary, '') like '%{}%' or coalesce(error_summary, '') like '%{}%' or coalesce(metadata_json, '') like '%{}%')) + \
                (select count(*) from events where coalesce(summary, '') like '%{}%' or coalesce(metadata_json, '') like '%{}%') + \
                (select count(*) from event_targets where coalesce(entity_type, '') like '%{}%' or coalesce(entity_id, '') like '%{}%' or coalesce(relation_type, '') like '%{}%') + \
                (select count(*) from notifications where coalesce(title, '') like '%{}%' or coalesce(message, '') like '%{}%' or coalesce(action_route, '') like '%{}%' or coalesce(metadata_json, '') like '%{}%')",
            outcome.run.id,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret,
            stdout_secret
        ),
    );
    assert_eq!(
        sqlite_secret_count, 0,
        "failed-run raw secret must not persist in agent_runs/events/event_targets/notifications text fields"
    );
}

#[test]
fn p229_run_bridge_cancel_kills_active_process_writes_log_and_rejects_terminal_mutation() {
    let (connection, _database_path) = migrated_file_connection("p229-cancel-bridge-db");
    let task = p204_task(&connection, "P2.29 cancelable bridge run task");
    let profile =
        p209_profile_with_command(&connection, "profile-shell-cancel-p229", "/bin/sh", true);
    let logs_dir = temp_home("p229-cancel-bridge-logs");
    let before_marker = "before-cancel-ready";
    let never_release_path = logs_dir.join("never-release-p229-cancel");
    let never_release_path_arg = never_release_path.to_string_lossy().to_string();

    let outcome = start_agent_run_command_with_connection(
        &connection,
        AgentRunCommandStartRequest {
            task_id: task.id,
            profile_id: profile.id,
            cwd: "/tmp".to_string(),
            argv: vec![
                "-c".to_string(),
                format!(
                    "printf '{before_marker}'; while [ ! -f \"$1\" ]; do sleep 1; done; printf 'should-not-finish'"
                ),
                "p229-cancel-sentinel".to_string(),
                never_release_path_arg,
            ],
            stdin: None,
            timeout_ms: None,
            logs_dir: logs_dir.clone(),
            metadata_json: Some("{\"source\":\"p229_cancel\"}".to_string()),
        },
    )
    .expect("start cancelable run");
    assert_eq!(outcome.run.status, AgentRunStatus::Running);

    let mut pre_cancel_stream = None;
    for _ in 0..80 {
        let stream = stream_run_output_command_with_connection(
            &connection,
            AgentRunCommandStreamRequest {
                run_id: outcome.run.id.clone(),
                logs_dir: logs_dir.clone(),
                offset: Some(0),
                max_bytes: Some(4096),
            },
        )
        .expect("stream pre-cancel output");
        if stream.content.contains(before_marker) {
            pre_cancel_stream = Some(stream);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let pre_cancel_stream = pre_cancel_stream.expect("deterministic pre-cancel output");
    assert_eq!(pre_cancel_stream.status, AgentRunStatus::Running);
    assert!(!pre_cancel_stream.eof);

    let cancelled = cancel_run_command_with_connection(
        &connection,
        outcome.run.id.clone(),
        AgentRunCommandCancelRequest {
            reason: Some("User stopped from bridge".to_string()),
            metadata_json: Some("{\"source\":\"p229_cancel\"}".to_string()),
        },
    )
    .expect("cancel active run via bridge helper");
    assert_eq!(cancelled.status, AgentRunStatus::Cancelled);
    assert!(cancelled.completed_at.is_some());

    let mut finalized = None;
    for _ in 0..40 {
        let status = read_run_status_command_with_connection(&connection, outcome.run.id.clone())
            .expect("read cancelled status");
        if status.status == AgentRunStatus::Cancelled
            && status.log_reference_id.is_some()
            && status.duration_ms.is_some()
        {
            finalized = Some(status);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let status = finalized.expect("cancelled run should receive worker evidence");
    assert_eq!(status.status, AgentRunStatus::Cancelled);
    assert!(status.log_reference_id.is_some());
    assert!(status.duration_ms.unwrap_or_default() > 0);

    let mut cancellation_notifications = Vec::new();
    for _ in 0..80 {
        let notifications =
            list_inbox_notifications(&connection, false, 20).expect("notifications");
        cancellation_notifications = notifications
            .into_iter()
            .filter(|item| item.run_id.as_deref() == Some(outcome.run.id.as_str()))
            .collect::<Vec<_>>();
        if cancellation_notifications.iter().any(|item| {
            item.notification_type == NotificationType::Attention
                && item.title == "Agent run cancelled"
                && item.severity == NotificationSeverity::Warning
        }) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    assert!(cancellation_notifications.iter().any(|item| {
        item.notification_type == NotificationType::Attention
            && item.title == "Agent run cancelled"
            && item.severity == NotificationSeverity::Warning
    }));

    let stream = stream_run_output_command_with_connection(
        &connection,
        AgentRunCommandStreamRequest {
            run_id: outcome.run.id.clone(),
            logs_dir: logs_dir.clone(),
            offset: Some(0),
            max_bytes: Some(4096),
        },
    )
    .expect("stream cancelled run output");
    assert!(stream.content.contains(before_marker));
    assert!(!stream.content.contains("should-not-finish"));

    let log_path = logs_dir.join(format!("{}.log", outcome.run.id));
    assert!(log_path.is_file(), "cancelled run log file must exist");
    let persisted_log = fs::read_to_string(&log_path).expect("read cancelled run log");
    assert!(persisted_log.contains(before_marker));
    assert!(!persisted_log.contains("should-not-finish"));
    let log_reference_id = status
        .log_reference_id
        .as_deref()
        .expect("cancelled run log reference id");
    let (log_scope, relative_path, redaction_count, byte_count, metadata_json): (
        String,
        String,
        i64,
        i64,
        String,
    ) = connection
        .query_row(
            "select log_scope, relative_path, redaction_count, byte_count, metadata_json from log_references where id = ?1",
            params![log_reference_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .expect("read cancellation log reference row");
    assert_eq!(log_scope, outcome.run.id);
    assert_eq!(relative_path, format!("{}.log", outcome.run.id));
    assert_eq!(redaction_count, 0);
    assert!(byte_count > 0);
    assert!(metadata_json.contains("bytes_written"));

    let history =
        list_run_history(&connection, &outcome.run.id, 20, None).expect("cancelled run history");
    assert!(history.iter().any(|item| {
        item.event.action_type == "run.cancelled"
            && item.event.targets.iter().any(|target| {
                target.entity_type == "agent_run"
                    && target.entity_id == outcome.run.id
                    && target.relation_type == "primary"
            })
    }));

    let recancel = cancel_run_command_with_connection(
        &connection,
        outcome.run.id,
        AgentRunCommandCancelRequest {
            reason: None,
            metadata_json: None,
        },
    )
    .expect_err("terminal run cannot be cancelled again");
    assert!(recancel.contains("terminal agent run cannot mutate"));
}

#[test]
fn p214_manual_review_service_creates_task_and_run_reviews_with_optional_placeholder_profile() {
    let connection = migrated_in_memory_connection();
    let task = create_task_service(
        &connection,
        TaskServiceCreateInput::new("Manual review target"),
    )
    .expect("create task through service");
    let profile = upsert_agent_profile(
        &connection,
        AgentProfileInput {
            id: "manual-reviewer".to_string(),
            label: "Manual Reviewer".to_string(),
            configured: false,
            command: None,
            config_json: "{}".to_string(),
            capabilities_json: "{\"manual_review\":true}".to_string(),
            credential_ref: None,
            env_refs_json: "[]".to_string(),
            metadata_json: "{\"placeholder\":true}".to_string(),
        },
    )
    .expect("upsert manual reviewer placeholder");

    let task_review = create_manual_review_service(
        &connection,
        ManualReviewServiceCreateInput {
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: " Manual reviewer verified task evidence ".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create task review through service");
    assert_eq!(task_review.subject_type, ReviewSubjectType::Task);
    assert_eq!(task_review.subject_id, task.id);
    assert_eq!(
        task_review.reviewer_profile_id.as_deref(),
        Some(profile.id.as_str())
    );
    assert_eq!(
        task_review.evidence_summary,
        "Manual reviewer verified task evidence"
    );

    let run = p205_run(&connection, &task);
    let run_review = create_manual_review_service(
        &connection,
        ManualReviewServiceCreateInput {
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: ReviewVerdict::RequiredFixes,
            evidence_summary: "Reviewer found missing proof".to_string(),
            required_fixes_json: "[{\"fix\":\"attach final log reference\"}]".to_string(),
            metadata_json: "{\"source\":\"manual_review_ui\"}".to_string(),
        },
    )
    .expect("create run review through service");
    assert_eq!(run_review.subject_type, ReviewSubjectType::AgentRun);
    assert_eq!(run_review.subject_id, run.id);
    assert_eq!(run_review.run_id.as_deref(), Some(run.id.as_str()));
    assert_eq!(
        run_review.reviewer_profile_id.as_deref(),
        Some(profile.id.as_str())
    );

    let events = list_event_records(
        &connection,
        EventListFilter {
            workspace_key: Some("agents"),
            action_type: Some("review.required_fixes"),
            outcome: Some("succeeded"),
            source: Some("review_record_repository"),
            limit: 10,
        },
    )
    .expect("list review events");
    assert!(events
        .iter()
        .any(|event| event.actor_id.as_deref() == Some(profile.id.as_str())));
}

#[test]
fn p214_manual_review_service_allows_manual_reviews_without_placeholder_and_preserves_guards() {
    let connection = migrated_in_memory_connection();
    let task = create_task_service(
        &connection,
        TaskServiceCreateInput::new("No placeholder review"),
    )
    .expect("create task through service");
    let review = create_manual_review_service(
        &connection,
        ManualReviewServiceCreateInput {
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::Approved,
            evidence_summary: "No configured reviewer profile is available yet".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create review without placeholder");
    assert_eq!(review.reviewer_profile_id, None);

    let missing_fixes = create_manual_review_service(
        &connection,
        ManualReviewServiceCreateInput {
            task_id: task.id.clone(),
            run_id: None,
            reviewer_profile_id: None,
            verdict: ReviewVerdict::RequiredFixes,
            evidence_summary: "Needs fixes".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect_err("service must preserve repository validation");
    assert!(matches!(
        missing_fixes,
        RepositoryError::Constraint {
            entity: "review_records",
            ..
        }
    ));
}

#[test]
fn p208_p215_task_and_notification_services_wrap_reviewed_repositories() {
    let connection = migrated_in_memory_connection();
    let created = create_task_service(
        &connection,
        TaskServiceCreateInput {
            title: " Service task ".to_string(),
            detail: Some(" Detail ".to_string()),
            priority: Some(TaskPriority::Urgent),
            workspace_key: Some("today".to_string()),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create task through service");
    assert_eq!(created.title, "Service task");
    assert_eq!(created.detail.as_deref(), Some("Detail"));

    let active = list_task_service(&connection).expect("list active tasks");
    assert!(active.iter().any(|task| task.id == created.id));
    let updated = update_task_service(
        &connection,
        &created.id,
        TaskServiceUpdateInput {
            title: Some("Updated service task".to_string()),
            detail: Some("Updated detail".to_string()),
            priority: Some(TaskPriority::High),
            workspace_key: None,
            metadata_json: Some("{\"source\":\"service\"}".to_string()),
        },
    )
    .expect("update task fields through service");
    assert_eq!(updated.title, "Updated service task");
    assert_eq!(updated.detail.as_deref(), Some("Updated detail"));
    assert_eq!(updated.priority, TaskPriority::High);
    let status_updated = update_task_service_status(&connection, &created.id, TaskStatus::Active)
        .expect("update task status through service");
    assert_eq!(status_updated.status, TaskStatus::Active);
    let detail = read_task_service(&connection, &created.id).expect("read detail through service");
    assert_eq!(detail.id, created.id);
    let task_events = list_task_history(&connection, &created.id, 10, None).expect("task events");
    assert!(task_events
        .iter()
        .any(|item| item.event.action_type == "task.updated"));

    let notification = create_notification_service(
        &connection,
        NotificationServiceCreateInput {
            notification_type: NotificationType::Attention,
            title: "Needs attention".to_string(),
            message: "Review task".to_string(),
            severity: NotificationSeverity::Warning,
            action_route: Some(format!("/tasks/{}", created.id)),
            task_id: Some(created.id.clone()),
            run_id: None,
            review_record_id: None,
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create notification through service");
    require_notification_action_service(&connection, &notification.id)
        .expect("mark action required");
    let inbox = list_inbox_notification_service(&connection, true, 10).expect("list inbox");
    assert_eq!(inbox[0].id, notification.id);

    archive_task_service(&connection, &created.id).expect("archive task");
    let active_after_archive = list_task_service(&connection).expect("list active after archive");
    assert!(active_after_archive
        .iter()
        .all(|task| task.id != created.id));
}

#[test]
fn p230_review_notification_history_bridge_records_state_transitions_and_targets() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "P2.30 review notification history task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id,
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create p230 run");
    let review = create_manual_review_command_with_connection(
        &connection,
        ManualReviewCommandCreateRequest {
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: "required_fixes".to_string(),
            evidence_summary: "Reviewer requires a focused fix".to_string(),
            required_fixes_json: "[{\"fix\":\"tighten notification history assertions\"}]"
                .to_string(),
            metadata_json: Some("{\"source\":\"p230_history\"}".to_string()),
        },
    )
    .expect("create p230 review through bridge");
    let review_route = format!("zoid://reviews/{}", review.id);
    let notification = create_notification_command_with_connection(
        &connection,
        NotificationCommandCreateRequest {
            notification_type: "review_required".to_string(),
            title: "P2.30 review required".to_string(),
            message: "Manual reviewer requested follow-up".to_string(),
            severity: "warning".to_string(),
            action_route: Some(review_route.clone()),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: Some("{\"source\":\"p230_history\"}".to_string()),
        },
    )
    .expect("create review-linked p230 notification through bridge");
    assert_eq!(
        notification.notification_type,
        NotificationType::ReviewRequired
    );
    assert_eq!(notification.severity, NotificationSeverity::Warning);
    assert_eq!(notification.state, NotificationState::Pending);
    assert_eq!(
        notification.action_route.as_deref(),
        Some(review_route.as_str())
    );
    assert_eq!(notification.task_id.as_deref(), Some(task.id.as_str()));
    assert_eq!(notification.run_id.as_deref(), Some(run.id.as_str()));
    assert_eq!(
        notification.review_record_id.as_deref(),
        Some(review.id.as_str())
    );

    let delivered = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "delivered".to_string(),
        },
    )
    .expect("deliver p230 notification");
    assert_eq!(delivered.state, NotificationState::Delivered);
    let action_required = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "action_required".to_string(),
        },
    )
    .expect("require p230 notification action");
    assert_eq!(action_required.state, NotificationState::ActionRequired);
    let read = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "read".to_string(),
        },
    )
    .expect("read p230 notification");
    assert_eq!(read.state, NotificationState::Read);
    assert!(read.read_at.is_some());
    let resolved = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "resolved".to_string(),
        },
    )
    .expect("resolve p230 notification");
    assert_eq!(resolved.state, NotificationState::Resolved);
    assert!(resolved.resolved_at.is_some());

    let history = list_notification_history_command_with_connection(
        &connection,
        notification.id.clone(),
        HistoryCommandListRequest {
            limit: Some(50),
            before: None,
        },
    )
    .expect("p230 notification history through bridge");
    for (action_type, state) in [
        ("notification.created", "pending"),
        ("notification.delivered", "delivered"),
        ("notification.action_required", "action_required"),
        ("notification.read", "read"),
        ("notification.resolved", "resolved"),
    ] {
        let item = history
            .iter()
            .find(|item| item.event.action_type == action_type)
            .unwrap_or_else(|| panic!("missing p230 notification event {action_type}"));
        assert_eq!(item.event.workspace_key.as_deref(), Some("inbox"));
        assert_eq!(item.event.outcome, "succeeded");
        assert_eq!(item.event.source, "notification_repository");
        assert!(item
            .event
            .metadata_json
            .contains(&format!("\"state\":\"{state}\"")));
        assert!(item.event.metadata_json.contains(&notification.id));
        assert!(item.event.metadata_json.contains(&task.id));
        assert!(item.event.metadata_json.contains(&run.id));
        assert!(item.event.metadata_json.contains(&review.id));
        for (entity_type, entity_id, relation_type) in [
            ("notification", notification.id.as_str(), "primary"),
            ("task", task.id.as_str(), "owner"),
            ("agent_run", run.id.as_str(), "run"),
            ("review_record", review.id.as_str(), "review"),
        ] {
            assert!(item.event.targets.iter().any(|target| {
                target.entity_type == entity_type
                    && target.entity_id == entity_id
                    && target.relation_type == relation_type
            }));
            assert!(item.matched_entities.iter().any(|target| {
                target.entity_type == entity_type
                    && target.entity_id == entity_id
                    && target.relation_type == relation_type
            }));
        }
    }

    let run_history = list_run_history_command_with_connection(
        &connection,
        run.id.clone(),
        HistoryCommandListRequest {
            limit: Some(50),
            before: None,
        },
    )
    .expect("p230 run history through bridge");
    assert!(run_history
        .iter()
        .any(|item| item.event.action_type == "notification.resolved"
            && item.matched_entities.iter().any(|target| {
                target.entity_type == "notification" && target.entity_id == notification.id
            })));
    let entity_history = list_entity_history_command_with_connection(
        &connection,
        HistoryCommandEntityListRequest {
            entity_type: "review_record".to_string(),
            entity_id: review.id.clone(),
            include_related: Some(true),
            limit: Some(50),
            before: None,
        },
    )
    .expect("p230 review entity history through bridge");
    assert!(entity_history.iter().any(|item| {
        item.event.action_type == "notification.action_required"
            && item.matched_entities.iter().any(|target| {
                target.entity_type == "notification" && target.entity_id == notification.id
            })
    }));

    for (source_type, source_id) in [
        ("task", task.id.as_str()),
        ("agent_run", run.id.as_str()),
        ("review_record", review.id.as_str()),
    ] {
        let link_count = count_rows(
            &connection,
            &format!(
                "select count(*) from entity_links where source_type = '{source_type}' and source_id = '{source_id}' and target_type = 'notification' and target_id = '{}' and relation_type = 'notifies'",
                notification.id
            ),
        );
        assert_eq!(link_count, 1, "{source_type} notification link");
    }
}

#[test]
fn p219_p230_review_notification_and_inbox_bridge_commands_preserve_services() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "P2.19 bridge review task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create agent run for bridge review");

    let review = create_manual_review_command_with_connection(
        &connection,
        ManualReviewCommandCreateRequest {
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: "required_fixes".to_string(),
            evidence_summary: "Needs follow-up evidence".to_string(),
            required_fixes_json: "[\"add test proof\"]".to_string(),
            metadata_json: Some("{\"source\":\"p219\"}".to_string()),
        },
    )
    .expect("create review through bridge helper");
    assert_eq!(review.task_id, task.id);
    assert_eq!(review.run_id.as_deref(), Some(run.id.as_str()));
    assert_eq!(review.verdict, ReviewVerdict::RequiredFixes);

    let read_review = read_review_record_command_with_connection(&connection, review.id.clone())
        .expect("read review through bridge helper");
    assert_eq!(read_review, review);

    let notification = create_notification_command_with_connection(
        &connection,
        NotificationCommandCreateRequest {
            notification_type: "review_required".to_string(),
            title: "Review attention".to_string(),
            message: "Bridge notification".to_string(),
            severity: "warning".to_string(),
            action_route: Some(format!("zoid://tasks/{}", task.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: Some("{\"source\":\"p219\"}".to_string()),
        },
    )
    .expect("create notification through bridge helper");
    assert_eq!(notification.state, NotificationState::Pending);
    assert_eq!(
        notification.review_record_id.as_deref(),
        Some(review.id.as_str())
    );
    let read_notification =
        read_notification_command_with_connection(&connection, notification.id.clone())
            .expect("read notification through bridge helper");
    assert_eq!(read_notification, notification);

    let review_count_before_secret = count_rows(&connection, "select count(*) from review_records");
    let secret_review = create_manual_review_command_with_connection(
        &connection,
        ManualReviewCommandCreateRequest {
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: "approved".to_string(),
            evidence_summary: "Reviewed output containing api_key=sk-raw...alue".to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: Some("{\"token\":\"sk-raw...alue\"}".to_string()),
        },
    )
    .expect_err("bridge review secret metadata must be rejected before persistence");
    assert!(secret_review.contains("secret"));
    assert_eq!(
        count_rows(&connection, "select count(*) from review_records"),
        review_count_before_secret
    );

    let notification_count_before_secret =
        count_rows(&connection, "select count(*) from notifications");
    let secret_notification = create_notification_command_with_connection(
        &connection,
        NotificationCommandCreateRequest {
            notification_type: "failure".to_string(),
            title: "Secret failure".to_string(),
            message: "Raw secret sk-raw...alue leaked".to_string(),
            severity: "error".to_string(),
            action_route: Some("zoid://tasks/secret?token=secret-token-value".to_string()),
            task_id: Some(task.id.clone()),
            run_id: None,
            review_record_id: None,
            metadata_json: Some("{\"api_key\":\"sk-raw...alue\"}".to_string()),
        },
    )
    .expect_err("bridge notification secret material must be rejected");
    assert!(secret_notification.contains("secret"));
    assert_eq!(
        count_rows(&connection, "select count(*) from notifications"),
        notification_count_before_secret
    );

    let delivered = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "delivered".to_string(),
        },
    )
    .expect("mark delivered");
    assert_eq!(delivered.state, NotificationState::Delivered);
    let action_required = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "action_required".to_string(),
        },
    )
    .expect("mark action required");
    assert_eq!(action_required.state, NotificationState::ActionRequired);

    let inbox = list_inbox_notifications_command_with_connection(
        &connection,
        InboxNotificationCommandListRequest {
            active_only: Some(true),
            limit: Some(10),
        },
    )
    .expect("list inbox through bridge helper");
    assert_eq!(
        inbox.first().map(|item| item.id.as_str()),
        Some(notification.id.as_str())
    );

    let read = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "read".to_string(),
        },
    )
    .expect("mark read");
    assert_eq!(read.state, NotificationState::Read);
    assert!(read.read_at.is_some());

    let resolved = update_notification_state_command_with_connection(
        &connection,
        notification.id.clone(),
        NotificationCommandStateRequest {
            state: "resolved".to_string(),
        },
    )
    .expect("resolve notification");
    assert_eq!(resolved.state, NotificationState::Resolved);
    assert!(resolved.resolved_at.is_some());

    let invalid_state = update_notification_state_command_with_connection(
        &connection,
        notification.id,
        NotificationCommandStateRequest {
            state: "snoozed".to_string(),
        },
    )
    .expect_err("unsupported bridge state action must fail");
    assert!(invalid_state.contains("unsupported notification bridge state action"));
}

#[test]
fn p219_p230_history_bridge_commands_query_task_run_notification_and_entity_without_raw_logs() {
    let connection = migrated_in_memory_connection();
    let task = p204_task(&connection, "P2.19 bridge history task");
    let profile = p204_profile(&connection, true);
    let session = p204_session(&connection, &task, &profile);
    let run = create_agent_run(
        &connection,
        AgentRunCreateInput {
            task_id: task.id.clone(),
            profile_id: profile.id,
            session_id: session.id,
            cwd: "/tmp".to_string(),
            metadata_json: "{}".to_string(),
        },
    )
    .expect("create run");
    let review = create_manual_review_command_with_connection(
        &connection,
        ManualReviewCommandCreateRequest {
            task_id: task.id.clone(),
            run_id: Some(run.id.clone()),
            reviewer_profile_id: None,
            verdict: "approved".to_string(),
            evidence_summary: "Looks good after checking log reference, not raw log body"
                .to_string(),
            required_fixes_json: "[]".to_string(),
            metadata_json: None,
        },
    )
    .expect("create review");
    let notification = create_notification_command_with_connection(
        &connection,
        NotificationCommandCreateRequest {
            notification_type: "completion".to_string(),
            title: "Completed".to_string(),
            message: "Task completed".to_string(),
            severity: "success".to_string(),
            action_route: None,
            task_id: Some(task.id.clone()),
            run_id: Some(run.id.clone()),
            review_record_id: Some(review.id),
            metadata_json: None,
        },
    )
    .expect("create notification");

    let task_history = list_task_history_command_with_connection(
        &connection,
        task.id.clone(),
        HistoryCommandListRequest {
            limit: Some(50),
            before: None,
        },
    )
    .expect("task history through bridge helper");
    assert!(task_history
        .iter()
        .any(|item| item.event.action_type == "task.created"));
    assert!(task_history
        .iter()
        .any(|item| item.event.action_type == "review.created"));
    assert!(task_history
        .iter()
        .any(|item| item.event.action_type == "notification.created"));

    let run_history = list_run_history_command_with_connection(
        &connection,
        run.id.clone(),
        HistoryCommandListRequest {
            limit: Some(50),
            before: None,
        },
    )
    .expect("run history through bridge helper");
    assert!(run_history
        .iter()
        .any(|item| item.event.action_type == "run.queued"));
    assert!(run_history
        .iter()
        .any(|item| item.event.action_type == "review.created"));

    let notification_history = list_notification_history_command_with_connection(
        &connection,
        notification.id.clone(),
        HistoryCommandListRequest {
            limit: Some(50),
            before: None,
        },
    )
    .expect("notification history through bridge helper");
    assert!(notification_history
        .iter()
        .any(|item| item.event.action_type == "notification.created"));

    let entity_history = list_entity_history_command_with_connection(
        &connection,
        HistoryCommandEntityListRequest {
            entity_type: "task".to_string(),
            entity_id: task.id,
            include_related: Some(false),
            limit: Some(1_000),
            before: None,
        },
    )
    .expect("entity history through bridge helper");
    assert!(!entity_history.is_empty());
    assert!(entity_history.len() <= 200);
    let serialized = serde_json::to_string(&entity_history).expect("serialize history");
    assert!(!serialized.contains("raw_log"));
    assert!(!serialized.contains("stdout:"));
}

#[test]
#[ignore]
fn p232_native_app_support_flow_creates_run_review_notification_history_and_persists() {
    if std::env::var("ZOID_P232_REAL_DB").as_deref() != Ok("1") {
        panic!("set ZOID_P232_REAL_DB=1 to run the real app-support P2.32 verification harness");
    }
    if let Ok(pid) = std::env::var("ZOID_P232_NATIVE_PID") {
        let status = std::process::Command::new("/bin/ps")
            .args(["-p", pid.as_str()])
            .status()
            .expect("check native app pid");
        assert!(
            status.success(),
            "native Zoid app pid must be running before verification"
        );
    }

    let home = PathBuf::from(std::env::var("HOME").expect("HOME must be set"));
    let app_paths = AppSupportPaths::for_home(&home);
    assert!(
        app_paths.database_path.exists(),
        "native app-support database must exist before P2.32 verification: {}",
        display_path(&app_paths.database_path)
    );
    ensure_directory(&app_paths.logs_dir).expect("ensure app-support logs dir");

    let connection = open_foundation_database(&app_paths.database_path)
        .expect("open native app-support sqlite database");
    let marker = format!("p232-native-verification-{}", now_millis());
    let profile = upsert_agent_profile(
        &connection,
        AgentProfileInput {
            id: format!("profile-{marker}"),
            label: "P2.32 Native Verification Shell".to_string(),
            configured: true,
            command: Some("/bin/sh".to_string()),
            config_json: "{\"mode\":\"local_command\"}".to_string(),
            capabilities_json:
                "{\"local_cli\":true,\"safe_command\":true,\"p232_verification\":true}".to_string(),
            credential_ref: None,
            env_refs_json: "[]".to_string(),
            metadata_json: format!("{{\"p232_marker\":\"{}\"}}", marker),
        },
    )
    .expect("upsert P2.32 verification profile");

    let task = create_task_command_with_connection(
        &connection,
        TaskCommandCreateRequest {
            title: format!("P2.32 native verification {marker}"),
            detail: Some(
                "Native app-support verification task created by explicit P2.32 harness."
                    .to_string(),
            ),
            priority: Some("normal".to_string()),
            workspace_key: Some("tasks".to_string()),
            metadata_json: Some(format!(
                "{{\"p232_marker\":\"{}\",\"source\":\"native_app_support_harness\"}}",
                marker
            )),
        },
    )
    .expect("create P2.32 task through native bridge helper");

    let outcome = start_agent_run_command_with_connection(
        &connection,
        AgentRunCommandStartRequest {
            task_id: task.id.clone(),
            profile_id: profile.id.clone(),
            cwd: "/tmp".to_string(),
            argv: vec![
                "-lc".to_string(),
                format!("printf 'P2.32 native verification output: {}\\n'", marker),
            ],
            stdin: None,
            timeout_ms: Some(5_000),
            logs_dir: app_paths.logs_dir.clone(),
            metadata_json: Some(format!("{{\"p232_marker\":\"{}\"}}", marker)),
        },
    )
    .expect("start P2.32 native command run");

    let mut final_chunk = None;
    for _ in 0..80 {
        let chunk = stream_run_output_command_with_connection(
            &connection,
            AgentRunCommandStreamRequest {
                run_id: outcome.run.id.clone(),
                logs_dir: app_paths.logs_dir.clone(),
                offset: Some(0),
                max_bytes: Some(16 * 1024),
            },
        )
        .expect("stream P2.32 run output");
        if chunk.eof && chunk.content.contains(&marker) {
            final_chunk = Some(chunk);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let final_chunk = final_chunk.expect("P2.32 run output should stream and reach EOF");
    assert!(final_chunk
        .content
        .contains("P2.32 native verification output"));
    assert_eq!(final_chunk.status, AgentRunStatus::Completed);

    let review = create_manual_review_command_with_connection(
        &connection,
        ManualReviewCommandCreateRequest {
            task_id: task.id.clone(),
            run_id: Some(outcome.run.id.clone()),
            reviewer_profile_id: Some(profile.id.clone()),
            verdict: "approved".to_string(),
            evidence_summary: format!("P2.32 native verification observed streamed output and persisted run marker {marker}."),
            required_fixes_json: "[]".to_string(),
            metadata_json: Some(format!("{{\"p232_marker\":\"{}\"}}", marker)),
        },
    )
    .expect("create P2.32 manual review through native bridge helper");

    let notification = create_notification_command_with_connection(
        &connection,
        NotificationCommandCreateRequest {
            notification_type: "completion".to_string(),
            title: "P2.32 native verification completed".to_string(),
            message: format!("Native verification marker {marker} completed and persisted."),
            severity: "success".to_string(),
            action_route: Some(format!("zoid://tasks/{}", task.id)),
            task_id: Some(task.id.clone()),
            run_id: Some(outcome.run.id.clone()),
            review_record_id: Some(review.id.clone()),
            metadata_json: Some(format!("{{\"p232_marker\":\"{}\"}}", marker)),
        },
    )
    .expect("create P2.32 notification through native bridge helper");

    let inbox = list_inbox_notifications_command_with_connection(
        &connection,
        InboxNotificationCommandListRequest {
            active_only: Some(true),
            limit: Some(50),
        },
    )
    .expect("list active inbox notifications");
    assert!(inbox.iter().any(|item| item.id == notification.id));

    let task_history = list_task_history_command_with_connection(
        &connection,
        task.id.clone(),
        HistoryCommandListRequest {
            limit: Some(100),
            before: None,
        },
    )
    .expect("list P2.32 task history");
    assert!(task_history
        .iter()
        .any(|item| item.event.action_type == "task.created"));
    assert!(task_history
        .iter()
        .any(|item| item.event.action_type == "review.created"));
    assert!(task_history
        .iter()
        .any(|item| item.event.action_type == "notification.created"));

    let run_history = list_run_history_command_with_connection(
        &connection,
        outcome.run.id.clone(),
        HistoryCommandListRequest {
            limit: Some(100),
            before: None,
        },
    )
    .expect("list P2.32 run history");
    assert!(run_history
        .iter()
        .any(|item| item.event.action_type == "run.completed"));
    assert!(run_history
        .iter()
        .any(|item| item.event.action_type == "review.created"));

    drop(connection);

    let reopened = open_foundation_database(&app_paths.database_path)
        .expect("reopen app-support sqlite after P2.32 flow");
    let persisted_task = read_task_command_with_connection(&reopened, task.id.clone())
        .expect("read persisted P2.32 task after reopen");
    assert_eq!(persisted_task.id, task.id);
    let persisted_run = read_run_status_command_with_connection(&reopened, outcome.run.id.clone())
        .expect("read persisted P2.32 run after reopen");
    assert_eq!(persisted_run.status, AgentRunStatus::Completed);
    let persisted_review = read_review_record_command_with_connection(&reopened, review.id.clone())
        .expect("read persisted P2.32 review after reopen");
    assert_eq!(persisted_review.id, review.id);
    let persisted_notification =
        read_notification_command_with_connection(&reopened, notification.id.clone())
            .expect("read persisted P2.32 notification after reopen");
    assert_eq!(persisted_notification.id, notification.id);

    println!("P2.32_NATIVE_VERIFICATION marker={marker}");
    println!("task_id={}", task.id);
    println!("run_id={}", outcome.run.id);
    println!("session_id={}", outcome.session_id);
    println!("review_id={}", review.id);
    println!("notification_id={}", notification.id);
    println!("log_path={}", outcome.log_path);
    println!("database_path={}", display_path(&app_paths.database_path));
}
