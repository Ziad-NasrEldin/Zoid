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

const ACTION_POLICY_CATEGORIES: &[&str] = &[
    "read_local_app_data",
    "read_gmail_calendar",
    "create_local_task",
    "create_private_markdown_note",
    "import_migrate_data",
    "modify_visible_non_code_file",
    "move_rename_copy_file",
    "bulk_file_operations",
    "delete_trash_files",
    "modify_code_repo_files",
    "commit_push_merge",
    "deploy_redeploy_rollback",
    "send_email",
    "publish_schedule_content",
    "change_automation_schedule",
    "run_existing_automation",
    "change_credentials_settings_integrations",
    "create_calendar_event",
    "edit_delete_calendar_event",
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
    Migration {
        version: 3,
        name: "core_schema_p105",
        sql: include_str!("../migrations/0003_core_schema_p105.sql"),
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
    config_dir: String,
    config_path: String,
    visible_user: VisibleUserPathStatus,
    app_support: AppSupportPathStatus,
    migration_version: i64,
    workspace_count: i64,
    event_count: i64,
    workspaces: Vec<WorkspaceRecord>,
    secure_services: SecureFoundationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppSupportPaths {
    root: PathBuf,
    logs_dir: PathBuf,
    database_parent: PathBuf,
    database_path: PathBuf,
    config_dir: PathBuf,
    config_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AppSupportPathStatus {
    root: String,
    logs_dir: String,
    database_parent: String,
    database_path: String,
    config_dir: String,
    config_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisibleUserPaths {
    root: PathBuf,
    starter_directories: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct VisibleUserPathStatus {
    root: String,
    starter_directories: Vec<String>,
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

#[allow(dead_code)]
type RepoResult<T> = Result<T, RepositoryError>;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum RepositoryError {
    NotFound {
        entity: &'static str,
        key: String,
    },
    Constraint {
        entity: &'static str,
        message: String,
    },
    InvalidJson {
        field: &'static str,
        message: String,
    },
    Database {
        message: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct AppSettingRecord {
    key: String,
    value_json: String,
    value_type: String,
    scope: String,
    description: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct AppSettingInput<'a> {
    key: &'a str,
    value_json: &'a str,
    value_type: &'a str,
    scope: &'a str,
    description: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct AppSettingUpdate<'a> {
    value_json: &'a str,
    value_type: &'a str,
    scope: &'a str,
    description: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityLinkRecord {
    id: String,
    source_type: String,
    source_id: String,
    target_type: String,
    target_id: String,
    relation_type: String,
    created_by_actor_type: String,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct EntityLinkInput<'a> {
    id: &'a str,
    source_type: &'a str,
    source_id: &'a str,
    target_type: &'a str,
    target_id: &'a str,
    relation_type: &'a str,
    created_by_actor_type: &'a str,
    metadata_json: &'a str,
}

#[tauri::command]
fn get_foundation_status() -> Result<FoundationStatus, String> {
    ensure_foundation().map_err(|error| error.to_string())
}

impl AppSupportPaths {
    fn for_home(home: &Path) -> Self {
        let root = home
            .join("Library")
            .join("Application Support")
            .join("Zoid");
        let logs_dir = root.join("logs");
        let database_parent = root.clone();
        let database_path = root.join("zoid.sqlite");
        let config_dir = root.join("config");
        let config_path = config_dir.join("settings.json");

        Self {
            root,
            logs_dir,
            database_parent,
            database_path,
            config_dir,
            config_path,
        }
    }

    fn status(&self) -> AppSupportPathStatus {
        AppSupportPathStatus {
            root: display_path(&self.root),
            logs_dir: display_path(&self.logs_dir),
            database_parent: display_path(&self.database_parent),
            database_path: display_path(&self.database_path),
            config_dir: display_path(&self.config_dir),
            config_path: display_path(&self.config_path),
        }
    }
}

impl VisibleUserPaths {
    fn for_home(home: &Path) -> Self {
        let root = home.join("Zoid");
        let starter_directories = VISIBLE_DIRS
            .iter()
            .map(|directory| root.join(directory))
            .collect();

        Self {
            root,
            starter_directories,
        }
    }

    fn status(&self) -> VisibleUserPathStatus {
        VisibleUserPathStatus {
            root: display_path(&self.root),
            starter_directories: self
                .starter_directories
                .iter()
                .map(|path| display_path(path))
                .collect(),
        }
    }
}

fn ensure_visible_user_paths(paths: &VisibleUserPaths) -> std::io::Result<()> {
    ensure_directory(&paths.root)?;
    for starter_directory in &paths.starter_directories {
        ensure_directory(starter_directory)?;
    }
    Ok(())
}

fn ensure_app_support_paths(paths: &AppSupportPaths) -> std::io::Result<()> {
    ensure_directory(&paths.root)?;
    ensure_directory(&paths.logs_dir)?;
    ensure_directory(&paths.database_parent)?;
    ensure_directory(&paths.config_dir)?;
    Ok(())
}

fn ensure_directory(path: &Path) -> std::io::Result<()> {
    if let Some(result) = validate_existing_directory(path)? {
        return result;
    }

    fs::create_dir_all(path)?;
    validate_existing_directory(path)?.unwrap_or_else(|| {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} was not created", display_path(path)),
        ))
    })
}

fn validate_existing_directory(path: &Path) -> std::io::Result<Option<std::io::Result<()>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                Ok(Some(Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("{} exists and is a symlink", display_path(path)),
                ))))
            } else if metadata.is_dir() {
                Ok(Some(Ok(())))
            } else {
                Ok(Some(Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("{} exists and is not a directory", display_path(path)),
                ))))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn validate_managed_file_path(path: &Path, label: &str) -> std::io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{} {} exists and is a symlink", label, display_path(path)),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn open_foundation_database(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    validate_managed_file_path(path, "database")?;
    let connection = Connection::open(path)?;
    enable_sqlite_foreign_keys(&connection)?;
    Ok(connection)
}

fn ensure_foundation() -> Result<FoundationStatus, Box<dyn std::error::Error>> {
    let home = home_dir()?;
    let visible_user_paths = VisibleUserPaths::for_home(&home);
    let app_support_paths = AppSupportPaths::for_home(&home);

    ensure_visible_user_paths(&visible_user_paths)?;
    ensure_app_support_paths(&app_support_paths)?;

    let connection = open_foundation_database(&app_support_paths.database_path)?;
    run_migrations(&connection)?;
    ensure_workspace_schema_compatibility(&connection)?;
    seed_workspaces(&connection)?;
    write_foundation_event(&connection)?;
    let safe_log_probe = write_safe_log(
        &app_support_paths.logs_dir,
        "foundation",
        "foundation.ready secure services checked",
    )?;

    let workspaces = list_workspaces(&connection)?;

    Ok(FoundationStatus {
        visible_root: display_path(&visible_user_paths.root),
        app_support_dir: display_path(&app_support_paths.root),
        database_path: display_path(&app_support_paths.database_path),
        logs_dir: display_path(&app_support_paths.logs_dir),
        config_dir: display_path(&app_support_paths.config_dir),
        config_path: display_path(&app_support_paths.config_path),
        visible_user: visible_user_paths.status(),
        app_support: app_support_paths.status(),
        migration_version: get_migration_version(&connection)?,
        workspace_count: workspaces.len() as i64,
        event_count: count_table(&connection, "events")?,
        workspaces,
        secure_services: secure_foundation_status(&safe_log_probe),
    })
}

fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    enable_sqlite_foreign_keys(connection)?;

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

    seed_action_policies(connection)?;

    Ok(())
}

fn enable_sqlite_foreign_keys(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "foreign_keys", "ON")
}

fn seed_action_policies(connection: &Connection) -> rusqlite::Result<()> {
    for category in ACTION_POLICY_CATEGORIES {
        let policy = evaluate_action_policy(category);
        connection.execute(
            "
            insert into action_policies (category, policy, reviewer_required, human_confirmation, reason, updated_at)
            values (?1, ?2, ?3, ?4, ?5, current_timestamp)
            on conflict(category) do update set
                policy = excluded.policy,
                reviewer_required = excluded.reviewer_required,
                human_confirmation = excluded.human_confirmation,
                reason = excluded.reason,
                updated_at = current_timestamp
            ",
            params![
                policy.category,
                action_policy_as_str(policy.policy),
                reviewer_requirement_as_str(policy.reviewer_required),
                human_confirmation_as_str(policy.human_confirmation),
                policy.reason
            ],
        )?;
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

#[allow(dead_code)]
fn validate_json_field(field: &'static str, value: &str) -> RepoResult<()> {
    serde_json::from_str::<Value>(value)
        .map(|_| ())
        .map_err(|error| RepositoryError::InvalidJson {
            field,
            message: error.to_string(),
        })
}

#[allow(dead_code)]
fn map_repository_error(entity: &'static str, error: rusqlite::Error) -> RepositoryError {
    match error {
        rusqlite::Error::SqliteFailure(ref sqlite_error, ref message)
            if sqlite_error.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            RepositoryError::Constraint {
                entity,
                message: message.clone().unwrap_or_else(|| error.to_string()),
            }
        }
        other => RepositoryError::Database {
            message: other.to_string(),
        },
    }
}

#[allow(dead_code)]
fn app_setting_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppSettingRecord> {
    Ok(AppSettingRecord {
        key: row.get(0)?,
        value_json: row.get(1)?,
        value_type: row.get(2)?,
        scope: row.get(3)?,
        description: row.get(4)?,
    })
}

#[allow(dead_code)]
fn upsert_app_setting(
    connection: &Connection,
    input: AppSettingInput<'_>,
) -> RepoResult<AppSettingRecord> {
    validate_json_field("value_json", input.value_json)?;
    connection
        .execute(
            "
            insert into app_settings (key, value_json, value_type, scope, description, updated_at)
            values (?1, ?2, ?3, ?4, ?5, current_timestamp)
            on conflict(key) do update set
                value_json = excluded.value_json,
                value_type = excluded.value_type,
                scope = excluded.scope,
                description = excluded.description,
                updated_at = current_timestamp
            ",
            params![
                input.key,
                input.value_json,
                input.value_type,
                input.scope,
                input.description
            ],
        )
        .map_err(|error| map_repository_error("app_settings", error))?;
    read_app_setting(connection, input.key)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "app_settings",
        key: input.key.to_string(),
    })
}

#[allow(dead_code)]
fn read_app_setting(connection: &Connection, key: &str) -> RepoResult<Option<AppSettingRecord>> {
    connection
        .query_row(
            "select key, value_json, value_type, scope, description from app_settings where key = ?1",
            params![key],
            app_setting_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("app_settings", error))
}

#[allow(dead_code)]
fn list_app_settings(connection: &Connection) -> RepoResult<Vec<AppSettingRecord>> {
    connection
        .prepare(
            "select key, value_json, value_type, scope, description from app_settings order by key asc",
        )
        .and_then(|mut statement| {
            let rows = statement.query_map([], app_setting_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("app_settings", error))
}

#[allow(dead_code)]
fn list_app_settings_by_scope(
    connection: &Connection,
    scope: &str,
) -> RepoResult<Vec<AppSettingRecord>> {
    connection
        .prepare("select key, value_json, value_type, scope, description from app_settings where scope = ?1 order by key asc")
        .and_then(|mut statement| {
            let rows = statement.query_map(params![scope], app_setting_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("app_settings", error))
}

#[allow(dead_code)]
fn update_app_setting(
    connection: &Connection,
    key: &str,
    update: AppSettingUpdate<'_>,
) -> RepoResult<AppSettingRecord> {
    validate_json_field("value_json", update.value_json)?;
    let changed = connection
        .execute(
            "
            update app_settings
            set value_json = ?2,
                value_type = ?3,
                scope = ?4,
                description = ?5,
                updated_at = current_timestamp
            where key = ?1
            ",
            params![
                key,
                update.value_json,
                update.value_type,
                update.scope,
                update.description
            ],
        )
        .map_err(|error| map_repository_error("app_settings", error))?;
    if changed == 0 {
        return Err(RepositoryError::NotFound {
            entity: "app_settings",
            key: key.to_string(),
        });
    }
    read_app_setting(connection, key)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "app_settings",
        key: key.to_string(),
    })
}

#[allow(dead_code)]
fn entity_link_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EntityLinkRecord> {
    Ok(EntityLinkRecord {
        id: row.get(0)?,
        source_type: row.get(1)?,
        source_id: row.get(2)?,
        target_type: row.get(3)?,
        target_id: row.get(4)?,
        relation_type: row.get(5)?,
        created_by_actor_type: row.get(6)?,
        metadata_json: row.get(7)?,
    })
}

#[allow(dead_code)]
fn insert_entity_link(
    connection: &Connection,
    input: EntityLinkInput<'_>,
) -> RepoResult<EntityLinkRecord> {
    validate_json_field("metadata_json", input.metadata_json)?;
    connection
        .execute(
            "
            insert into entity_links (
                id, source_type, source_id, target_type, target_id, relation_type,
                created_by_actor_type, metadata_json
            )
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                input.id,
                input.source_type,
                input.source_id,
                input.target_type,
                input.target_id,
                input.relation_type,
                input.created_by_actor_type,
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("entity_links", error))?;
    read_entity_link(connection, input.id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "entity_links",
        key: input.id.to_string(),
    })
}

#[allow(dead_code)]
fn insert_or_get_entity_link(
    connection: &Connection,
    input: EntityLinkInput<'_>,
) -> RepoResult<EntityLinkRecord> {
    match insert_entity_link(connection, input) {
        Ok(link) => Ok(link),
        Err(constraint_error @ RepositoryError::Constraint { .. }) => {
            match read_entity_link_by_unique(
                connection,
                input.source_type,
                input.source_id,
                input.target_type,
                input.target_id,
                input.relation_type,
            )? {
                Some(link) => Ok(link),
                None => Err(constraint_error),
            }
        }
        Err(error) => Err(error),
    }
}

#[allow(dead_code)]
fn read_entity_link(connection: &Connection, id: &str) -> RepoResult<Option<EntityLinkRecord>> {
    connection
        .query_row(
            "
            select id, source_type, source_id, target_type, target_id, relation_type,
                   created_by_actor_type, metadata_json
            from entity_links
            where id = ?1
            ",
            params![id],
            entity_link_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("entity_links", error))
}

#[allow(dead_code)]
fn read_entity_link_by_unique(
    connection: &Connection,
    source_type: &str,
    source_id: &str,
    target_type: &str,
    target_id: &str,
    relation_type: &str,
) -> RepoResult<Option<EntityLinkRecord>> {
    connection
        .query_row(
            "
            select id, source_type, source_id, target_type, target_id, relation_type,
                   created_by_actor_type, metadata_json
            from entity_links
            where source_type = ?1 and source_id = ?2 and target_type = ?3 and target_id = ?4 and relation_type = ?5
            ",
            params![source_type, source_id, target_type, target_id, relation_type],
            entity_link_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("entity_links", error))
}

#[allow(dead_code)]
fn list_entity_links_for_source(
    connection: &Connection,
    source_type: &str,
    source_id: &str,
) -> RepoResult<Vec<EntityLinkRecord>> {
    connection
        .prepare(
            "
            select id, source_type, source_id, target_type, target_id, relation_type,
                   created_by_actor_type, metadata_json
            from entity_links
            where source_type = ?1 and source_id = ?2
            order by target_type asc, target_id asc, relation_type asc, id asc
            ",
        )
        .and_then(|mut statement| {
            let rows =
                statement.query_map(params![source_type, source_id], entity_link_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("entity_links", error))
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

    if lower.contains("authorization:") && lower.contains("bearer ") {
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
    ensure_directory(logs_dir)?;
    let safe_scope = safe_log_scope(scope);
    let path = logs_dir.join(format!("{}.log", safe_scope));
    validate_managed_file_path(&path, "log file")?;
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

fn action_policy_as_str(policy: ActionPolicy) -> &'static str {
    match policy {
        ActionPolicy::Allow => "allow",
        ActionPolicy::AskBeforeAction => "ask_before_action",
        ActionPolicy::BlockUntilConfirmed => "block_until_confirmed",
        ActionPolicy::RequireClearTask => "require_clear_task",
    }
}

fn reviewer_requirement_as_str(requirement: ReviewerRequirement) -> &'static str {
    match requirement {
        ReviewerRequirement::None => "none",
        ReviewerRequirement::Maybe => "maybe",
        ReviewerRequirement::Usually => "usually",
        ReviewerRequirement::Yes => "yes",
    }
}

fn human_confirmation_as_str(confirmation: HumanConfirmation) -> &'static str {
    match confirmation {
        HumanConfirmation::None => "none",
        HumanConfirmation::Maybe => "maybe",
        HumanConfirmation::Yes => "yes",
        HumanConfirmation::Always => "always",
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
    use std::path::Component;

    fn temp_home(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zoid-{name}-{}", now_millis()))
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
        fs::write(paths.root.join("Notes"), "not a directory")
            .expect("write conflicting notes file");

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
        std::os::unix::fs::symlink(&target, paths.root.join("Notes"))
            .expect("create notes symlink");

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

        let error =
            ensure_app_support_paths(&paths).expect_err("logs file must block directory setup");
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
    fn migrations_seed_core_workspaces() {
        let connection = Connection::open_in_memory().expect("open in-memory sqlite");
        run_migrations(&connection).expect("run migrations");
        seed_workspaces(&connection).expect("seed workspaces");

        let workspace_ids: Vec<String> = list_workspaces(&connection)
            .unwrap()
            .into_iter()
            .map(|workspace| workspace.id)
            .collect();

        assert_eq!(get_migration_version(&connection).unwrap(), 3);
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

        let app_settings =
            list_app_settings_by_scope(&connection, "app").expect("list app settings");
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

        let links = list_entity_links_for_source(&connection, "workspace", "today")
            .expect("list source links");
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
        .expect_err(
            "primary key collision with a different logical tuple should remain a constraint",
        );

        assert!(matches!(error, RepositoryError::Constraint { .. }));
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
    fn migrations_create_p105_core_schema_tables() {
        let connection = Connection::open_in_memory().expect("open in-memory sqlite");
        run_migrations(&connection).expect("run migrations");

        assert_eq!(get_migration_version(&connection).unwrap(), 3);

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
                "workspace_key",
                "relative_path",
                "display_name",
                "mime_type",
                "content_hash",
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
            "idx_file_references_workspace_path",
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

        assert_eq!(get_migration_version(&connection).unwrap(), 3);

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

        assert_eq!(get_migration_version(&connection).unwrap(), 3);
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

    #[cfg(unix)]
    #[test]
    fn safe_log_writer_rejects_symlinked_log_file_before_append() {
        let logs_dir = std::env::temp_dir().join(format!("zoid-log-symlink-{}", now_millis()));
        let target = logs_dir.with_extension("target.log");
        fs::create_dir_all(&logs_dir).expect("create logs dir");
        fs::write(&target, "original target content\n").expect("write symlink target");
        std::os::unix::fs::symlink(&target, logs_dir.join("agent.log"))
            .expect("create managed log symlink");

        let error = write_safe_log(&logs_dir, "agent", "new content")
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
