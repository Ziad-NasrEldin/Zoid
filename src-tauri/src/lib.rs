use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const WORKSPACES: &[(&str, &str, &str)] = &[
    (
        "today",
        "Today",
        "Command center, attention, and current work.",
    ),
    (
        "tasks",
        "Tasks",
        "First-class tasks, review states, and follow-ups.",
    ),
    ("notes", "Notes", "Markdown notes with local metadata."),
    (
        "agents",
        "Agents",
        "CLI profiles, sessions, runs, and reviews.",
    ),
    ("code", "Code", "Repositories, Launch Gate, and git work."),
    (
        "content",
        "Content",
        "Planning, review, and OmniSocials publishing state.",
    ),
    (
        "automations",
        "Automations",
        "Visible recurring jobs and run history.",
    ),
    (
        "business",
        "Business",
        "Contacts, companies, follow-ups, and linked work.",
    ),
    (
        "products",
        "Products",
        "First-class product hubs and timelines.",
    ),
    (
        "files",
        "Files",
        "Local file manager and Zoid-aware attachments.",
    ),
    ("browser", "Browser", "Work webview/capture workspace."),
    (
        "inbox",
        "Inbox",
        "Notifications, approvals, blockers, and Gmail state.",
    ),
    (
        "calendar",
        "Calendar",
        "Built-in calendar with Apple Calendar integration gates.",
    ),
    (
        "history",
        "History",
        "Universal timeline and linked event history.",
    ),
];

const VISIBLE_DIRS: &[&str] = &[
    "Notes", "Content", "Assets", "Exports", "Imports", "Backups",
];

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "foundation_schema",
        sql: include_str!("../migrations/0001_foundation.sql"),
    },
    Migration {
        version: 2,
        name: "event_schema_backfill",
        sql: include_str!("../migrations/0002_event_schema_backfill.sql"),
    },
];

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

#[derive(Debug, Serialize)]
struct WorkspaceRecord {
    id: String,
    label: String,
    description: String,
    position: i64,
}

#[derive(Debug, Serialize)]
struct FoundationStatus {
    visible_root: String,
    app_support_dir: String,
    database_path: String,
    logs_dir: String,
    migration_version: i64,
    workspace_count: i64,
    event_count: i64,
    workspaces: Vec<WorkspaceRecord>,
    secure_services: SecureFoundationStatus,
}

#[derive(Debug, Serialize)]
struct SecureFoundationStatus {
    redaction_ready: bool,
    safe_logging_ready: bool,
    action_policy_ready: bool,
    event_writer_ready: bool,
    keychain_status: String,
    sample_policy: ActionPolicyDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ActionPolicy {
    Allow,
    AskBeforeAction,
    BlockUntilConfirmed,
    RequireClearTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewerRequirement {
    None,
    Maybe,
    Usually,
    Yes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HumanConfirmation {
    None,
    Maybe,
    Yes,
    Always,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ActionPolicyDecision {
    category: String,
    policy: ActionPolicy,
    reviewer_required: ReviewerRequirement,
    human_confirmation: HumanConfirmation,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RedactionOutcome {
    text: String,
    redaction_count: usize,
}

#[derive(Debug)]
struct SafeLogWrite {
    path: PathBuf,
    redaction_count: usize,
    bytes_written: usize,
}

#[derive(Debug)]
struct EventInput<'a> {
    event_type: &'a str,
    actor_type: &'a str,
    actor_id: Option<&'a str>,
    workspace_key: Option<&'a str>,
    summary: &'a str,
    severity: &'a str,
    source: &'a str,
    metadata_json: &'a str,
    targets: Vec<(&'a str, &'a str, &'a str)>,
}

#[tauri::command]
fn get_foundation_status() -> Result<FoundationStatus, String> {
    ensure_foundation().map_err(|error| error.to_string())
}

fn ensure_foundation() -> Result<FoundationStatus, Box<dyn std::error::Error>> {
    let home = home_dir()?;
    let visible_root = home.join("Zoid");
    let app_support_dir = home.join("Library/Application Support/Zoid");
    let logs_dir = app_support_dir.join("logs");
    let database_path = app_support_dir.join("zoid.sqlite");

    fs::create_dir_all(&visible_root)?;
    for child in VISIBLE_DIRS {
        fs::create_dir_all(visible_root.join(child))?;
    }
    fs::create_dir_all(&logs_dir)?;

    let connection = Connection::open(&database_path)?;
    run_migrations(&connection)?;
    ensure_workspace_schema_compatibility(&connection)?;
    seed_workspaces(&connection)?;
    write_foundation_event(&connection)?;
    let safe_log_probe = write_safe_log(
        &logs_dir,
        "foundation",
        "foundation.ready secure services checked",
    )?;

    let workspaces = list_workspaces(&connection)?;

    Ok(FoundationStatus {
        visible_root: display_path(&visible_root),
        app_support_dir: display_path(&app_support_dir),
        database_path: display_path(&database_path),
        logs_dir: display_path(&logs_dir),
        migration_version: get_migration_version(&connection)?,
        workspace_count: workspaces.len() as i64,
        event_count: count_table(&connection, "events")?,
        workspaces,
        secure_services: secure_foundation_status(&safe_log_probe),
    })
}

fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "
        create table if not exists schema_migrations (
            version integer primary key,
            name text not null,
            applied_at text not null default current_timestamp
        );
        ",
    )?;

    for migration in MIGRATIONS {
        let applied = migration_applied(connection, migration.version)?;

        if migration.version == 2 {
            ensure_event_schema_compatibility(connection)?;
        }

        if !applied {
            connection.execute_batch(migration.sql)?;
            connection.execute(
                "insert or ignore into schema_migrations (version, name) values (?1, ?2)",
                params![migration.version, migration.name],
            )?;
        }
    }

    Ok(())
}

fn migration_applied(connection: &Connection, version: i64) -> rusqlite::Result<bool> {
    let applied = connection
        .query_row(
            "select 1 from schema_migrations where version = ?1",
            params![version],
            |_| Ok(()),
        )
        .optional()?;
    Ok(applied.is_some())
}

fn ensure_event_schema_compatibility(connection: &Connection) -> rusqlite::Result<()> {
    let columns = table_columns(connection, "events")?;

    if !columns.contains("timestamp") {
        connection.execute_batch(
            "alter table events add column timestamp text; update events set timestamp = coalesce(created_at, current_timestamp) where timestamp is null;",
        )?;
    }
    if !columns.contains("actor_type") {
        connection.execute_batch(
            "alter table events add column actor_type text; update events set actor_type = coalesce(actor, 'system') where actor_type is null;",
        )?;
    }
    if !columns.contains("actor_id") {
        connection.execute_batch("alter table events add column actor_id text;")?;
    }
    if !columns.contains("workspace_key") {
        connection.execute_batch("alter table events add column workspace_key text;")?;
    }

    Ok(())
}

fn table_columns(connection: &Connection, table: &str) -> rusqlite::Result<HashSet<String>> {
    let mut statement = connection.prepare(&format!("pragma table_info({})", table))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let mut columns = HashSet::new();
    for row in rows {
        columns.insert(row?);
    }
    Ok(columns)
}

fn ensure_workspace_schema_compatibility(connection: &Connection) -> rusqlite::Result<()> {
    let columns = table_columns(connection, "workspaces")?;

    if !columns.contains("description") {
        connection.execute_batch(
            "alter table workspaces add column description text not null default '';",
        )?;
    }
    if !columns.contains("position") {
        connection.execute_batch(
            "alter table workspaces add column position integer not null default 0;",
        )?;
    }
    if !columns.contains("enabled") {
        connection.execute_batch(
            "alter table workspaces add column enabled integer not null default 1;",
        )?;
    }
    if !columns.contains("created_at") {
        connection.execute_batch(
            "alter table workspaces add column created_at text not null default '';",
        )?;
    }
    if !columns.contains("updated_at") {
        connection.execute_batch(
            "alter table workspaces add column updated_at text not null default '';",
        )?;
    }

    Ok(())
}

fn seed_workspaces(connection: &Connection) -> rusqlite::Result<()> {
    for (position, (id, label, description)) in WORKSPACES.iter().enumerate() {
        connection.execute(
            "
            insert into workspaces (id, label, description, position)
            values (?1, ?2, ?3, ?4)
            on conflict(id) do update set
                label = excluded.label,
                description = excluded.description,
                position = excluded.position,
                updated_at = current_timestamp
            ",
            params![id, label, description, position as i64],
        )?;
    }
    Ok(())
}

fn list_workspaces(connection: &Connection) -> rusqlite::Result<Vec<WorkspaceRecord>> {
    let mut statement = connection.prepare(
        "select id, label, description, position from workspaces where enabled = 1 order by position asc",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(WorkspaceRecord {
            id: row.get(0)?,
            label: row.get(1)?,
            description: row.get(2)?,
            position: row.get(3)?,
        })
    })?;

    let mut workspaces = Vec::new();
    for row in rows {
        workspaces.push(row?);
    }
    Ok(workspaces)
}

fn write_foundation_event(connection: &Connection) -> rusqlite::Result<()> {
    let existing_event_id: Option<String> = connection
        .query_row(
            "select id from events where type = 'foundation.ready' limit 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let event_id = if let Some(event_id) = existing_event_id {
        connection.execute(
            "
            update events
            set actor_type = coalesce(actor_type, 'system'),
                actor_id = coalesce(actor_id, 'zoid'),
                workspace_key = coalesce(workspace_key, 'today'),
                timestamp = coalesce(timestamp, created_at, current_timestamp)
            where id = ?1
            ",
            params![event_id],
        )?;
        event_id
    } else {
        write_event(
            connection,
            EventInput {
                event_type: "foundation.ready",
                actor_type: "system",
                actor_id: Some("zoid"),
                workspace_key: Some("today"),
                summary: "Zoid foundation initialized",
                severity: "info",
                source: "app_shell",
                metadata_json: "{\"phase\":\"secure_foundation\"}",
                targets: vec![("workspace", "today", "primary")],
            },
        )?
    };

    connection.execute(
        "insert or ignore into event_targets (event_id, entity_type, entity_id, relation_type) values (?1, 'workspace', 'today', 'primary')",
        params![event_id],
    )?;

    Ok(())
}

fn write_event(connection: &Connection, input: EventInput<'_>) -> rusqlite::Result<String> {
    let event_id = format!("evt_{}", now_millis());
    let redacted_summary = redact_secrets(input.summary).text;
    let redacted_metadata = redact_metadata_json(input.metadata_json);

    connection.execute(
        "
        insert into events (id, type, timestamp, actor_type, actor_id, workspace_key, summary, severity, source, metadata_json)
        values (?1, ?2, current_timestamp, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            event_id,
            input.event_type,
            input.actor_type,
            input.actor_id,
            input.workspace_key,
            redacted_summary,
            input.severity,
            input.source,
            redacted_metadata
        ],
    )?;

    for (entity_type, entity_id, relation_type) in input.targets {
        connection.execute(
            "insert or ignore into event_targets (event_id, entity_type, entity_id, relation_type) values (?1, ?2, ?3, ?4)",
            params![event_id, entity_type, entity_id, relation_type],
        )?;
    }

    Ok(event_id)
}

fn secure_foundation_status(safe_log_probe: &SafeLogWrite) -> SecureFoundationStatus {
    SecureFoundationStatus {
        redaction_ready: redact_secrets("api_key=secret-value").redaction_count == 1,
        safe_logging_ready: safe_log_probe.path.is_file()
            && safe_log_probe.bytes_written > 0
            && safe_log_probe.redaction_count == 0,
        action_policy_ready: true,
        event_writer_ready: true,
        keychain_status: "blocked_unverified_native_keychain_not_tested".to_string(),
        sample_policy: evaluate_action_policy("send_email"),
    }
}

fn redact_secrets(input: &str) -> RedactionOutcome {
    let mut redaction_count = 0;
    let text = input
        .split_inclusive('\n')
        .map(|line| redact_line(line, &mut redaction_count))
        .collect::<String>();

    RedactionOutcome {
        text,
        redaction_count,
    }
}

fn redact_metadata_json(input: &str) -> String {
    match serde_json::from_str::<Value>(input) {
        Ok(mut value) => {
            redact_json_value(&mut value, None);
            serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
        }
        Err(_) => {
            let redacted = redact_secrets(input).text;
            serde_json::json!({
                "redaction_notice": "metadata_was_not_valid_json",
                "redacted_text": redacted
            })
            .to_string()
        }
    }
}

fn redact_json_value(value: &mut Value, key_hint: Option<&str>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object.iter_mut() {
                redact_json_value(child, Some(key));
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_json_value(item, key_hint);
            }
        }
        Value::String(text) => {
            if key_hint.is_some_and(is_secret_key) || redact_secrets(text).redaction_count > 0 {
                *text = "[REDACTED]".to_string();
            }
        }
        _ => {
            if key_hint.is_some_and(is_secret_key) {
                *value = Value::String("[REDACTED]".to_string());
            }
        }
    }
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "auth_token",
        "token",
        "password",
        "secret",
        "client_secret",
        "authorization",
    ]
    .iter()
    .any(|secret_key| lower.contains(secret_key))
}

fn redact_line(line: &str, redaction_count: &mut usize) -> String {
    let lower = line.to_ascii_lowercase();
    let secret_keys = [
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "auth_token",
        "token",
        "password",
        "secret",
        "client_secret",
    ];

    if lower.contains("authorization: bearer ") {
        if let Some(index) = lower.find("bearer ") {
            *redaction_count += 1;
            return format!("{}bearer [REDACTED]{}", &line[..index], line_suffix(line));
        }
    }

    for key in secret_keys {
        if let Some(key_index) = lower.find(key) {
            let after_key = &line[key_index + key.len()..];
            if let Some(separator_offset) = after_key.find(['=', ':']) {
                let separator_index = key_index + key.len() + separator_offset;
                *redaction_count += 1;
                return format!(
                    "{}{} [REDACTED]{}",
                    &line[..separator_index],
                    &line[separator_index..separator_index + 1],
                    line_suffix(line)
                );
            }
        }
    }

    line.to_string()
}

fn line_suffix(line: &str) -> &str {
    if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn write_safe_log(logs_dir: &Path, scope: &str, content: &str) -> std::io::Result<SafeLogWrite> {
    fs::create_dir_all(logs_dir)?;
    let safe_scope = safe_log_scope(scope);
    let path = logs_dir.join(format!("{}.log", safe_scope));
    let redacted = redact_secrets(content);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let mut line = redacted.text;
    if !line.ends_with('\n') {
        line.push('\n');
    }
    file.write_all(line.as_bytes())?;

    Ok(SafeLogWrite {
        path,
        redaction_count: redacted.redaction_count,
        bytes_written: line.len(),
    })
}

fn safe_log_scope(scope: &str) -> String {
    let sanitized: String = scope
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.trim_matches('_').is_empty() {
        "app".to_string()
    } else {
        sanitized
    }
}

fn evaluate_action_policy(category: &str) -> ActionPolicyDecision {
    let normalized = category
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_");
    match normalized.as_str() {
        "read_local_app_data" => decision(
            category,
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
            "Local reads are allowed without confirmation.",
        ),
        "read_gmail_calendar" => decision(
            category,
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
            "Authenticated integration reads are allowed inside granted permissions.",
        ),
        "create_local_task" => decision(
            category,
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
            "Simple local tasks are low risk but must write an event.",
        ),
        "create_private_markdown_note" => decision(
            category,
            ActionPolicy::Allow,
            ReviewerRequirement::None,
            HumanConfirmation::None,
            "Private notes are allowed inside the current task/session context.",
        ),
        "import_migrate_data" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Usually,
            HumanConfirmation::Yes,
            "Imports and migrations need count/source/destination preview.",
        ),
        "modify_visible_non_code_file" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Maybe,
            "Visible non-code file edits depend on scope and destructiveness.",
        ),
        "move_rename_copy_file" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::None,
            HumanConfirmation::Yes,
            "File movement requires exact path preview; deletes should use Trash.",
        ),
        "bulk_file_operations" => decision(
            category,
            ActionPolicy::BlockUntilConfirmed,
            ReviewerRequirement::Yes,
            HumanConfirmation::Yes,
            "Bulk operations need exact affected path preview.",
        ),
        "delete_trash_files" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Yes,
            "Deletion requires confirmation and should prefer Trash.",
        ),
        "modify_code_repo_files" => decision(
            category,
            ActionPolicy::RequireClearTask,
            ReviewerRequirement::Yes,
            HumanConfirmation::None,
            "Consequential code changes require a clear task, diff, verification, and review gate.",
        ),
        "commit_push_merge" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Yes,
            HumanConfirmation::Yes,
            "Git publication requires branch/diff/remote preview.",
        ),
        "deploy_redeploy_rollback" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Yes,
            HumanConfirmation::Yes,
            "Deployments require Launch Gate evidence.",
        ),
        "send_email" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Always,
            "Email needs recipient/body/attachment preview.",
        ),
        "publish_schedule_content" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Yes,
            HumanConfirmation::Maybe,
            "Publishing needs applicable review policy and sensitive-content checks.",
        ),
        "change_automation_schedule" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Yes,
            "Automation changes require before/after preview.",
        ),
        "run_existing_automation" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Maybe,
            "Consequential automations need policy-specific confirmation.",
        ),
        "change_credentials_settings_integrations" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Always,
            "Credential and integration changes must never reveal raw secrets.",
        ),
        "create_calendar_event" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::None,
            HumanConfirmation::Yes,
            "Calendar writes need title/time/calendar preview.",
        ),
        "edit_delete_calendar_event" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Always,
            "Calendar edits/deletes require before/after preview.",
        ),
        _ => decision(
            category,
            ActionPolicy::BlockUntilConfirmed,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Yes,
            "Unknown action categories fail closed until explicitly classified.",
        ),
    }
}

fn decision(
    category: &str,
    policy: ActionPolicy,
    reviewer_required: ReviewerRequirement,
    human_confirmation: HumanConfirmation,
    reason: &str,
) -> ActionPolicyDecision {
    ActionPolicyDecision {
        category: category.to_string(),
        policy,
        reviewer_required,
        human_confirmation,
        reason: reason.to_string(),
    }
}

fn get_migration_version(connection: &Connection) -> rusqlite::Result<i64> {
    connection.query_row(
        "select coalesce(max(version), 0) from schema_migrations",
        [],
        |row| row.get(0),
    )
}

fn count_table(connection: &Connection, table: &str) -> rusqlite::Result<i64> {
    match table {
        "events" => connection.query_row("select count(*) from events", [], |row| row.get(0)),
        "workspaces" => {
            connection.query_row("select count(*) from workspaces", [], |row| row.get(0))
        }
        _ => Err(rusqlite::Error::InvalidParameterName(table.to_string())),
    }
}

fn home_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".into())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_| {
            ensure_foundation()
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_foundation_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(get_migration_version(&connection).unwrap(), 2);
        assert_eq!(
            workspace_ids,
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

        assert_eq!(get_migration_version(&connection).unwrap(), 2);
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
    fn safe_log_writer_sanitizes_scope_and_persists_redacted_content() {
        let logs_dir = std::env::temp_dir().join(format!("zoid-log-test-{}", now_millis()));
        let write = write_safe_log(&logs_dir, "../agent/run 1", "token=abc123\nvisible output")
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
}
