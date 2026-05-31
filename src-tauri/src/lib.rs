use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
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
        let event_id = format!("evt_{}", now_millis());
        connection.execute(
            "
            insert into events (id, type, timestamp, actor_type, actor_id, workspace_key, summary, severity, source, metadata_json)
            values (?1, 'foundation.ready', current_timestamp, 'system', 'zoid', 'today', 'Zoid foundation initialized', 'info', 'app_shell', ?2)
            ",
            params![event_id, "{\"phase\":\"secure_foundation\"}"],
        )?;
        event_id
    };

    connection.execute(
        "insert or ignore into event_targets (event_id, entity_type, entity_id, relation_type) values (?1, 'workspace', 'today', 'primary')",
        params![event_id],
    )?;

    Ok(())
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
}
