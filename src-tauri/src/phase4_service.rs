use crate::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RepoProfileRecord {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) root_path: String,
    pub(crate) profile_type: String,
    pub(crate) default_branch: Option<String>,
    pub(crate) package_manager: Option<String>,
    pub(crate) linked_product_id: Option<String>,
    pub(crate) status: String,
    pub(crate) metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RepoProfileInput {
    pub(crate) display_name: String,
    pub(crate) root_path: String,
    pub(crate) profile_type: String,
    pub(crate) default_branch: Option<String>,
    pub(crate) package_manager: Option<String>,
    pub(crate) linked_product_id: Option<String>,
    pub(crate) metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RepoEntityLinkInput {
    pub(crate) repo_id: String,
    pub(crate) target_type: String,
    pub(crate) target_id: String,
    pub(crate) relation_type: String,
    pub(crate) metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LaunchGateState {
    NotReady,
    ReadyToVerify,
    VerificationBlocked,
    Verified,
    Failed,
}

impl LaunchGateState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NotReady => "not_ready",
            Self::ReadyToVerify => "ready_to_verify",
            Self::VerificationBlocked => "verification_blocked",
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "not_ready" => Ok(Self::NotReady),
            "ready_to_verify" => Ok(Self::ReadyToVerify),
            "verification_blocked" => Ok(Self::VerificationBlocked),
            "verified" => Ok(Self::Verified),
            "failed" => Ok(Self::Failed),
            _ => Err(RepositoryError::Constraint {
                entity: "launch_gates",
                message: format!("unsupported launch gate state: {value}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LaunchGateRecord {
    pub(crate) id: String,
    pub(crate) repo_id: String,
    pub(crate) product_id: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) state: LaunchGateState,
    pub(crate) final_verdict: Option<String>,
    pub(crate) metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LaunchGateCreateInput {
    pub(crate) repo_id: String,
    pub(crate) product_id: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LaunchGateEvidenceRecord {
    pub(crate) id: String,
    pub(crate) launch_gate_id: String,
    pub(crate) evidence_type: String,
    pub(crate) label: String,
    pub(crate) url: Option<String>,
    pub(crate) status_code: Option<i64>,
    pub(crate) manual_note: Option<String>,
    pub(crate) metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LaunchGateEvidenceInput {
    pub(crate) launch_gate_id: String,
    pub(crate) evidence_type: String,
    pub(crate) label: String,
    pub(crate) url: Option<String>,
    pub(crate) status_code: Option<i64>,
    pub(crate) manual_note: Option<String>,
    pub(crate) metadata_json: String,
}

fn phase4_validate_text_field(field: &'static str, value: &str, min: usize, max_bytes: usize) -> RepoResult<()> {
    let byte_len = value.trim().as_bytes().len();
    if byte_len < min || byte_len > max_bytes {
        return Err(RepositoryError::Constraint {
            entity: "phase4",
            message: format!("{field} must be between {min} and {max_bytes} bytes"),
        });
    }
    Ok(())
}

fn repo_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RepoProfileRecord> {
    Ok(RepoProfileRecord {
        id: row.get(0)?,
        display_name: row.get(1)?,
        root_path: row.get(2)?,
        profile_type: row.get(3)?,
        default_branch: row.get(4)?,
        package_manager: row.get(5)?,
        linked_product_id: row.get(6)?,
        status: row.get(7)?,
        metadata_json: row.get(8)?,
    })
}

fn launch_gate_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LaunchGateRecord> {
    let state: String = row.get(4)?;
    let state = LaunchGateState::from_str(&state)
        .map_err(|error| rusqlite::Error::InvalidParameterName(repository_error_message(error)))?;
    Ok(LaunchGateRecord {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        product_id: row.get(2)?,
        task_id: row.get(3)?,
        state,
        final_verdict: row.get(5)?,
        metadata_json: row.get(6)?,
    })
}

fn launch_gate_evidence_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LaunchGateEvidenceRecord> {
    Ok(LaunchGateEvidenceRecord {
        id: row.get(0)?,
        launch_gate_id: row.get(1)?,
        evidence_type: row.get(2)?,
        label: row.get(3)?,
        url: row.get(4)?,
        status_code: row.get(5)?,
        manual_note: row.get(6)?,
        metadata_json: row.get(7)?,
    })
}

fn validate_repo_profile_input(input: &RepoProfileInput) -> RepoResult<()> {
    phase4_validate_text_field("display_name", &input.display_name, 1, 256)?;
    phase4_validate_text_field("root_path", &input.root_path, 1, 2048)?;
    if !["product_app", "website", "library", "experiment", "client_project", "content_docs", "other"].contains(&input.profile_type.as_str()) {
        return Err(RepositoryError::Constraint { entity: "repo_profiles", message: format!("unsupported repo profile type: {}", input.profile_type) });
    }
    if let Some(value) = input.default_branch.as_deref() { phase4_validate_text_field("default_branch", value, 1, 128)?; }
    if let Some(value) = input.package_manager.as_deref() { phase4_validate_text_field("package_manager", value, 1, 64)?; }
    if let Some(value) = input.linked_product_id.as_deref() { phase4_validate_text_field("linked_product_id", value, 1, 128)?; }
    validate_json_field("metadata_json", &input.metadata_json)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)
}

#[allow(dead_code)]
pub(crate) fn add_repo_profile(connection: &Connection, input: RepoProfileInput) -> RepoResult<RepoProfileRecord> {
    validate_repo_profile_input(&input)?;
    let id = format!("repo-{}-{}", now_millis(), REPO_PROFILE_COUNTER.fetch_add(1, Ordering::Relaxed));
    connection.execute(
        "insert into repo_profiles (id, display_name, root_path, profile_type, default_branch, package_manager, linked_product_id, metadata_json) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, input.display_name.trim(), input.root_path.trim(), input.profile_type, input.default_branch.as_deref().map(str::trim), input.package_manager.as_deref().map(str::trim), input.linked_product_id.as_deref().map(str::trim), input.metadata_json],
    ).map_err(|error| map_repository_error("repo_profiles", error))?;
    read_repo_profile(connection, &id)?.ok_or_else(|| RepositoryError::NotFound { entity: "repo_profiles", key: id })
}

#[allow(dead_code)]
pub(crate) fn read_repo_profile(connection: &Connection, repo_id: &str) -> RepoResult<Option<RepoProfileRecord>> {
    connection.query_row(
        "select id, display_name, root_path, profile_type, default_branch, package_manager, linked_product_id, status, metadata_json from repo_profiles where id = ?1",
        params![repo_id],
        repo_profile_from_row,
    ).optional().map_err(|error| map_repository_error("repo_profiles", error))
}

#[allow(dead_code)]
pub(crate) fn list_repo_profiles(connection: &Connection) -> RepoResult<Vec<RepoProfileRecord>> {
    connection.prepare(
        "select id, display_name, root_path, profile_type, default_branch, package_manager, linked_product_id, status, metadata_json from repo_profiles where status != 'archived' order by updated_at desc, id asc"
    ).and_then(|mut statement| {
        let rows = statement.query_map([], repo_profile_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
    }).map_err(|error| map_repository_error("repo_profiles", error))
}

#[allow(dead_code)]
pub(crate) fn link_repo_entity(connection: &Connection, input: RepoEntityLinkInput) -> RepoResult<EntityLinkRecord> {
    read_repo_profile(connection, &input.repo_id)?.ok_or_else(|| RepositoryError::NotFound { entity: "repo_profiles", key: input.repo_id.clone() })?;
    if !["task", "product"].contains(&input.target_type.as_str()) {
        return Err(RepositoryError::Constraint { entity: "entity_links", message: "Phase 4 repo links only support task/product targets".to_string() });
    }
    phase4_validate_text_field("target_id", &input.target_id, 1, 128)?;
    phase4_validate_text_field("relation_type", &input.relation_type, 1, 64)?;
    validate_json_field("metadata_json", &input.metadata_json)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    let id = format!("repo-link-{}-{}", now_millis(), EVENT_COUNTER.fetch_add(1, Ordering::Relaxed));
    insert_or_get_entity_link(connection, EntityLinkInput {
        id: &id,
        source_type: "repo",
        source_id: &input.repo_id,
        target_type: &input.target_type,
        target_id: &input.target_id,
        relation_type: &input.relation_type,
        created_by_actor_type: "user",
        metadata_json: &input.metadata_json,
    })
}

#[allow(dead_code)]
pub(crate) fn list_phase4_repo_integration_states(connection: &Connection) -> RepoResult<Vec<IntegrationStatusRecord>> {
    for (key, label, note) in [
        ("github", "GitHub", "State only; no deep GitHub automation or git reads are implemented."),
        ("vercel", "Vercel", "State only; deploy execution is not implemented."),
    ] {
        let config_json = serde_json::json!({ "phase": "4", "scope": "state_only", "note": note }).to_string();
        connection.execute(
            "insert or ignore into integration_statuses (integration_key, display_name, status, config_json, credential_ref, last_checked_at, updated_at) values (?1, ?2, 'not_configured', ?3, null, null, current_timestamp)",
            params![key, label, config_json],
        ).map_err(|error| map_repository_error("integration_statuses", error))?;
    }
    let mut rows = list_integration_statuses(connection)?;
    rows.retain(|row| row.integration_key == "github" || row.integration_key == "vercel");
    Ok(rows)
}

#[allow(dead_code)]
pub(crate) fn create_launch_gate(connection: &Connection, input: LaunchGateCreateInput) -> RepoResult<LaunchGateRecord> {
    read_repo_profile(connection, &input.repo_id)?.ok_or_else(|| RepositoryError::NotFound { entity: "repo_profiles", key: input.repo_id.clone() })?;
    if let Some(value) = input.product_id.as_deref() { phase4_validate_text_field("product_id", value, 1, 128)?; }
    if let Some(value) = input.task_id.as_deref() { phase4_validate_text_field("task_id", value, 1, 128)?; }
    validate_json_field("metadata_json", &input.metadata_json)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    let id = format!("launch-gate-{}-{}", now_millis(), LAUNCH_GATE_COUNTER.fetch_add(1, Ordering::Relaxed));
    connection.execute(
        "insert into launch_gates (id, repo_id, product_id, task_id, state, final_verdict, metadata_json) values (?1, ?2, ?3, ?4, 'verification_blocked', 'blocked_missing_evidence', ?5)",
        params![id, input.repo_id, input.product_id.as_deref().map(str::trim), input.task_id.as_deref().map(str::trim), input.metadata_json],
    ).map_err(|error| map_repository_error("launch_gates", error))?;
    read_launch_gate(connection, &id)?.ok_or_else(|| RepositoryError::NotFound { entity: "launch_gates", key: id })
}

#[allow(dead_code)]
pub(crate) fn read_launch_gate(connection: &Connection, launch_gate_id: &str) -> RepoResult<Option<LaunchGateRecord>> {
    connection.query_row(
        "select id, repo_id, product_id, task_id, state, final_verdict, metadata_json from launch_gates where id = ?1",
        params![launch_gate_id],
        launch_gate_from_row,
    ).optional().map_err(|error| map_repository_error("launch_gates", error))
}

#[allow(dead_code)]
pub(crate) fn add_launch_gate_evidence(connection: &Connection, input: LaunchGateEvidenceInput) -> RepoResult<LaunchGateEvidenceRecord> {
    read_launch_gate(connection, &input.launch_gate_id)?.ok_or_else(|| RepositoryError::NotFound { entity: "launch_gates", key: input.launch_gate_id.clone() })?;
    if !["manual_note", "url_status", "screenshot", "test_output", "deployment_record"].contains(&input.evidence_type.as_str()) {
        return Err(RepositoryError::Constraint { entity: "launch_gate_evidence", message: format!("unsupported evidence type: {}", input.evidence_type) });
    }
    phase4_validate_text_field("label", &input.label, 1, 256)?;
    if let Some(value) = input.url.as_deref() { phase4_validate_text_field("url", value, 1, 2048)?; }
    if let Some(code) = input.status_code { if !(100..=599).contains(&code) { return Err(RepositoryError::Constraint { entity: "launch_gate_evidence", message: "status_code must be 100-599".to_string() }); } }
    if let Some(value) = input.manual_note.as_deref() { phase4_validate_text_field("manual_note", value, 1, 4096)?; }
    validate_json_field("metadata_json", &input.metadata_json)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    let id = format!("launch-evidence-{}-{}", now_millis(), LAUNCH_GATE_EVIDENCE_COUNTER.fetch_add(1, Ordering::Relaxed));
    connection.execute(
        "insert into launch_gate_evidence (id, launch_gate_id, evidence_type, label, url, status_code, manual_note, metadata_json) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, input.launch_gate_id, input.evidence_type, input.label.trim(), input.url.as_deref().map(str::trim), input.status_code, input.manual_note.as_deref().map(str::trim), input.metadata_json],
    ).map_err(|error| map_repository_error("launch_gate_evidence", error))?;
    connection.query_row(
        "select id, launch_gate_id, evidence_type, label, url, status_code, manual_note, metadata_json from launch_gate_evidence where id = ?1",
        params![id],
        launch_gate_evidence_from_row,
    ).map_err(|error| map_repository_error("launch_gate_evidence", error))
}

#[allow(dead_code)]
pub(crate) fn evaluate_launch_gate(connection: &Connection, launch_gate_id: &str) -> RepoResult<LaunchGateRecord> {
    read_launch_gate(connection, launch_gate_id)?.ok_or_else(|| RepositoryError::NotFound { entity: "launch_gates", key: launch_gate_id.to_string() })?;
    let evidence_count: i64 = connection.query_row(
        "select count(*) from launch_gate_evidence where launch_gate_id = ?1",
        params![launch_gate_id],
        |row| row.get(0),
    ).map_err(|error| map_repository_error("launch_gate_evidence", error))?;
    let (state, verdict) = if evidence_count > 0 {
        (LaunchGateState::Verified, "verified_with_evidence")
    } else {
        (LaunchGateState::VerificationBlocked, "blocked_missing_evidence")
    };
    connection.execute(
        "update launch_gates set state = ?1, final_verdict = ?2, updated_at = current_timestamp where id = ?3",
        params![state.as_str(), verdict, launch_gate_id],
    ).map_err(|error| map_repository_error("launch_gates", error))?;
    read_launch_gate(connection, launch_gate_id)?.ok_or_else(|| RepositoryError::NotFound { entity: "launch_gates", key: launch_gate_id.to_string() })
}

pub(crate) fn normalize_launch_action_policy_category(action: &str) -> Result<String, String> {
    let normalized = normalize_action_category(action);
    match normalized.as_str() {
        "commit" | "push" | "merge" | "commit_push_merge" => Ok("commit_push_merge".to_string()),
        "deploy" | "redeploy" | "rollback" | "deploy_redeploy_rollback" => Ok("deploy_redeploy_rollback".to_string()),
        _ => Err(format!("unsupported Phase 4 protected action preview: {action}")),
    }
}
