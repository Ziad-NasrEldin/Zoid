use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LogRetentionSettings {
    pub scope: String,
    pub retention_days: i64,
    pub max_total_bytes: i64,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LogRetentionSettingsUpdateRequest {
    pub scope: String,
    pub retention_days: i64,
    pub max_total_bytes: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct LogCleanupRunRecord {
    pub id: String,
    pub scope: String,
    pub dry_run: bool,
    pub files_considered: i64,
    pub files_deleted: i64,
    pub bytes_deleted: i64,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SafeErrorPayload {
    pub code: String,
    pub category: String,
    pub message: String,
    pub recoverable: bool,
}

fn validate_scope(scope: &str) -> Result<(), String> {
    if scope.is_empty()
        || scope.len() > 64
        || !scope
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("invalid log retention scope".into());
    }
    Ok(())
}

pub(crate) fn map_safe_error(error: impl ToString) -> SafeErrorPayload {
    let redacted = crate::redact_secrets(&error.to_string()).text;
    let lower = redacted.to_ascii_lowercase();
    let (code, category, recoverable) = if lower.contains("permission") || lower.contains("denied")
    {
        ("permission_denied", "permission", true)
    } else if lower.contains("not found") || lower.contains("no such file") {
        ("not_found", "filesystem", true)
    } else if lower.contains("migration") || lower.contains("schema") {
        ("migration_failed", "database", false)
    } else if lower.contains("secret") || lower.contains("token") || lower.contains("password") {
        ("redacted_security_error", "security", false)
    } else {
        ("internal_error", "internal", false)
    };
    SafeErrorPayload {
        code: code.into(),
        category: category.into(),
        message: redacted,
        recoverable,
    }
}

pub(crate) fn list_log_retention_settings(
    conn: &Connection,
) -> Result<Vec<LogRetentionSettings>, String> {
    let mut stmt = conn.prepare("select scope, retention_days, max_total_bytes, enabled, updated_at from log_retention_settings order by scope")
        .map_err(|e| map_safe_error(e).message)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LogRetentionSettings {
                scope: row.get(0)?,
                retention_days: row.get(1)?,
                max_total_bytes: row.get(2)?,
                enabled: row.get::<_, i64>(3)? == 1,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| map_safe_error(e).message)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_safe_error(e).message)
}

pub(crate) fn upsert_log_retention_settings(
    conn: &Connection,
    req: LogRetentionSettingsUpdateRequest,
) -> Result<LogRetentionSettings, String> {
    validate_scope(&req.scope)?;
    if !(1..=3650).contains(&req.retention_days) {
        return Err("retention_days must be 1..=3650".into());
    }
    if !(1024..=1_073_741_824).contains(&req.max_total_bytes) {
        return Err("max_total_bytes must be 1024..=1073741824".into());
    }
    conn.execute("insert into log_retention_settings(scope, retention_days, max_total_bytes, enabled, updated_at) values (?1, ?2, ?3, ?4, current_timestamp)
        on conflict(scope) do update set retention_days=excluded.retention_days, max_total_bytes=excluded.max_total_bytes, enabled=excluded.enabled, updated_at=current_timestamp",
        params![req.scope, req.retention_days, req.max_total_bytes, if req.enabled {1} else {0}]).map_err(|e| map_safe_error(e).message)?;
    list_log_retention_settings(conn)?
        .into_iter()
        .find(|s| s.scope == req.scope)
        .ok_or_else(|| "log retention setting was not persisted".into())
}

pub(crate) fn cleanup_logs(
    conn: &Connection,
    logs_dir: &Path,
    scope: &str,
    dry_run: bool,
) -> Result<LogCleanupRunRecord, String> {
    validate_scope(scope)?;
    let settings = list_log_retention_settings(conn)?
        .into_iter()
        .find(|s| s.scope == scope)
        .or_else(|| {
            list_log_retention_settings(conn)
                .ok()?
                .into_iter()
                .find(|s| s.scope == "default")
        })
        .ok_or("missing log retention settings")?;
    let id = format!(
        "log_cleanup_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let mut considered = 0i64;
    let mut deleted = 0i64;
    let mut bytes = 0i64;
    if settings.enabled && logs_dir.is_dir() {
        let cutoff = SystemTime::now()
            - std::time::Duration::from_secs((settings.retention_days as u64) * 86_400);
        let mut candidates: Vec<(std::path::PathBuf, fs::Metadata, SystemTime)> = Vec::new();
        let mut total_log_bytes = 0i64;
        for entry in fs::read_dir(logs_dir).map_err(|e| map_safe_error(e).message)? {
            let entry = entry.map_err(|e| map_safe_error(e).message)?;
            let path = entry.path();
            if path.parent() != Some(logs_dir)
                || path.extension().and_then(|s| s.to_str()) != Some("log")
            {
                continue;
            }
            let meta = fs::symlink_metadata(&path).map_err(|e| map_safe_error(e).message)?;
            if meta.file_type().is_symlink() || !meta.is_file() {
                continue;
            }
            considered += 1;
            total_log_bytes += meta.len() as i64;
            let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            candidates.push((path, meta, modified));
        }
        candidates.sort_by_key(|(_, _, modified)| *modified);
        let mut remove_paths = std::collections::BTreeSet::new();
        for (path, meta, modified) in &candidates {
            if *modified < cutoff {
                bytes += meta.len() as i64;
                remove_paths.insert(path.clone());
            }
        }
        let mut retained_bytes = total_log_bytes - bytes;
        for (path, meta, _) in &candidates {
            if retained_bytes <= settings.max_total_bytes {
                break;
            }
            if remove_paths.insert(path.clone()) {
                bytes += meta.len() as i64;
                retained_bytes -= meta.len() as i64;
            }
        }
        for path in remove_paths {
            if !dry_run {
                fs::remove_file(&path).map_err(|e| map_safe_error(e).message)?;
                deleted += 1;
            }
        }
    }
    let status = "completed".to_string();
    conn.execute("insert into log_cleanup_runs(id, scope, dry_run, files_considered, files_deleted, bytes_deleted, status) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![id, scope, if dry_run {1} else {0}, considered, deleted, bytes, status]).map_err(|e| map_safe_error(e).message)?;
    Ok(LogCleanupRunRecord {
        id,
        scope: scope.into(),
        dry_run,
        files_considered: considered,
        files_deleted: deleted,
        bytes_deleted: bytes,
        status,
        error_message: None,
        created_at: "current_timestamp".into(),
    })
}

pub(crate) fn create_pre_migration_backup(
    db_path: &Path,
    backup_dir: &Path,
) -> Result<Option<std::path::PathBuf>, String> {
    if !db_path.exists() {
        return Ok(None);
    }
    fs::create_dir_all(backup_dir).map_err(|e| map_safe_error(e).message)?;
    let backup = backup_dir.join(format!(
        "zoid-pre-migration-{}.sqlite.bak",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));
    fs::copy(db_path, &backup).map_err(|e| map_safe_error(e).message)?;
    Ok(Some(backup))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tempdir_path(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "zoid_phase8_{}_{}",
            name,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn p827_log_retention_settings_validate_and_cleanup_old_logs() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "create table log_retention_settings (scope text primary key, retention_days integer not null, max_total_bytes integer not null, enabled integer not null, updated_at text not null default current_timestamp);
             create table log_cleanup_runs (id text primary key, scope text not null, dry_run integer not null, files_considered integer not null, files_deleted integer not null, bytes_deleted integer not null, status text not null, error_message text, created_at text not null default current_timestamp);
             insert into log_retention_settings(scope, retention_days, max_total_bytes, enabled) values ('default', 30, 10485760, 1);",
        )
        .unwrap();
        let setting = upsert_log_retention_settings(
            &conn,
            LogRetentionSettingsUpdateRequest {
                scope: "agent".into(),
                retention_days: 3650,
                max_total_bytes: 1024,
                enabled: true,
            },
        )
        .unwrap();
        assert_eq!(setting.scope, "agent");
        let dir = tempdir_path("retention");
        let first = dir.join("agent-a.log");
        let second = dir.join("agent-b.log");
        let notes = dir.join("agent-not-log.txt");
        let nested_dir = dir.join("nested");
        fs::create_dir_all(&nested_dir).unwrap();
        let nested = nested_dir.join("nested.log");
        fs::write(&first, vec![b'a'; 800]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        fs::write(&second, vec![b'b'; 800]).unwrap();
        fs::write(&notes, vec![b'n'; 900]).unwrap();
        fs::write(&nested, vec![b'c'; 900]).unwrap();
        #[cfg(unix)]
        {
            let symlink = dir.join("agent-symlink.log");
            std::os::unix::fs::symlink(&second, &symlink).unwrap();
        }

        let dry_run = cleanup_logs(&conn, &dir, "agent", true).unwrap();
        assert_eq!(dry_run.files_considered, 2);
        assert_eq!(dry_run.files_deleted, 0);
        assert!(dry_run.bytes_deleted >= 800);
        assert!(first.exists());
        assert!(second.exists());

        let run = cleanup_logs(&conn, &dir, "agent", false).unwrap();
        assert_eq!(run.files_considered, 2);
        assert_eq!(run.files_deleted, 1);
        assert!(
            !first.exists(),
            "oldest direct child .log should be removed to enforce max_total_bytes"
        );
        assert!(
            second.exists(),
            "newer direct child .log should remain under cap"
        );
        assert!(notes.exists(), "non-.log file must not be removed");
        assert!(nested.exists(), "nested .log file must not be removed");
        #[cfg(unix)]
        assert!(
            dir.join("agent-symlink.log").exists(),
            "symlink .log must not be followed or removed"
        );
    }

    #[test]
    fn p828_migration_backup_copies_existing_database_before_destructive_work() {
        let dir = tempdir_path("backup");
        let db = dir.join("zoid.sqlite");
        fs::write(&db, b"sqlite bytes").unwrap();
        let backup = create_pre_migration_backup(&db, &dir.join("Backups"))
            .unwrap()
            .unwrap();
        assert_eq!(fs::read(backup).unwrap(), b"sqlite bytes");
    }

    #[test]
    fn p829_safe_error_mapping_redacts_secret_material() {
        let payload = map_safe_error("migration failed password=super-secret-token");
        assert_eq!(payload.code, "migration_failed");
        assert!(!payload.message.contains("super-secret-token"));
    }
}
