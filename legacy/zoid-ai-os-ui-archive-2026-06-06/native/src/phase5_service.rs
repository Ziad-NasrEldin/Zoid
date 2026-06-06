use crate::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContentPlanRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) pillar: String,
    pub(crate) status: String,
    pub(crate) owner_actor_type: String,
    pub(crate) metadata_json: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentPlanInput {
    pub(crate) title: String,
    pub(crate) pillar: Option<String>,
    pub(crate) owner_actor_type: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContentPieceRecord {
    pub(crate) id: String,
    pub(crate) plan_id: String,
    pub(crate) title: String,
    pub(crate) body_markdown: String,
    pub(crate) status: String,
    pub(crate) platforms_json: String,
    pub(crate) required_gate: String,
    pub(crate) metadata_json: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentPieceInput {
    pub(crate) plan_id: String,
    pub(crate) title: String,
    pub(crate) body_markdown: Option<String>,
    pub(crate) platforms: Option<Vec<String>>,
    pub(crate) required_gate: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentPieceDraftInput {
    pub(crate) piece_id: String,
    pub(crate) body_markdown: String,
    pub(crate) status: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MediaAssetRecord {
    pub(crate) id: String,
    pub(crate) piece_id: String,
    pub(crate) asset_kind: String,
    pub(crate) storage_ref: String,
    pub(crate) alt_text: String,
    pub(crate) metadata_json: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MediaAssetInput {
    pub(crate) piece_id: String,
    pub(crate) asset_kind: String,
    pub(crate) storage_ref: String,
    pub(crate) mime_type: Option<String>,
    pub(crate) byte_size: Option<i64>,
    pub(crate) width: Option<i64>,
    pub(crate) height: Option<i64>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) alt_text: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContentReviewGateRecord {
    pub(crate) id: String,
    pub(crate) piece_id: String,
    pub(crate) gate_type: String,
    pub(crate) status: String,
    pub(crate) evidence_summary: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentReviewGateInput {
    pub(crate) piece_id: String,
    pub(crate) gate_type: String,
    pub(crate) reviewer_actor_type: Option<String>,
    pub(crate) reviewer_actor_id: Option<String>,
    pub(crate) evidence_summary: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentReviewGateDecisionInput {
    pub(crate) evidence_summary: String,
    pub(crate) reviewer_actor_type: Option<String>,
    pub(crate) reviewer_actor_id: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContentScheduleRecord {
    pub(crate) id: String,
    pub(crate) piece_id: String,
    pub(crate) platform: String,
    pub(crate) scheduled_for: String,
    pub(crate) status: String,
    pub(crate) confirmation_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentScheduleInput {
    pub(crate) piece_id: String,
    pub(crate) platform: String,
    pub(crate) scheduled_for: String,
    pub(crate) confirmation_id: Option<String>,
    pub(crate) metadata_json: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContentVerificationRecord {
    pub(crate) id: String,
    pub(crate) piece_id: Option<String>,
    pub(crate) schedule_id: Option<String>,
    pub(crate) platform: String,
    pub(crate) action_type: String,
    pub(crate) outcome: String,
    pub(crate) provider_status: Option<String>,
    pub(crate) failure_report: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentListRequest {
    pub(crate) limit: Option<i64>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContentVerificationListRequest {
    pub(crate) piece_id: Option<String>,
    pub(crate) schedule_id: Option<String>,
    pub(crate) limit: Option<i64>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OmniSocialsActionInput {
    pub(crate) piece_id: String,
    pub(crate) platform: String,
    pub(crate) schedule_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OmniSocialsStatusRecord {
    pub(crate) state: String,
    pub(crate) platform: String,
    pub(crate) credential_ref: Option<String>,
    pub(crate) status_note: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PlatformConstraintResult {
    pub(crate) platform: String,
    pub(crate) passed: bool,
    pub(crate) violations: Vec<String>,
}

fn p5_json(v: Option<String>) -> RepoResult<String> {
    let value = v.unwrap_or_else(|| "{}".to_string());
    validate_json_field("metadata_json", &value)?;
    validate_no_secret_json("metadata_json", &value)?;
    Ok(value)
}
fn p5_text(entity: &'static str, field: &'static str, value: &str) -> RepoResult<String> {
    normalize_small_text(entity, field, value)
}
fn p5_id(prefix: &str, counter: &std::sync::atomic::AtomicU64) -> String {
    format!(
        "{prefix}-{}-{}",
        now_millis(),
        counter.fetch_add(1, Ordering::Relaxed)
    )
}
fn emit_content_event(
    c: &Connection,
    event_type: &str,
    summary: &str,
    targets: Vec<(&str, &str, &str)>,
) -> RepoResult<()> {
    write_event(
        c,
        EventInput {
            event_type,
            actor_type: "system",
            actor_id: None,
            workspace_key: Some("content"),
            summary,
            severity: "info",
            source: "phase5_content_omnisocials",
            metadata_json: "{}",
            targets,
        },
    )
    .map(|_| ())
    .map_err(|e| map_repository_error("events", e))
}
fn validate_content_status(status: &str) -> RepoResult<()> {
    match status {
        "draft" | "review_ready" | "approved" | "scheduled" | "blocked" | "archived" => Ok(()),
        "published" => Err(RepositoryError::Constraint {
            entity: "content_pieces",
            message: "published status is reserved for future real provider success".to_string(),
        }),
        other => Err(RepositoryError::Constraint {
            entity: "content_pieces",
            message: format!("invalid content status: {other}"),
        }),
    }
}
fn validate_platform(platform: &str) -> RepoResult<String> {
    let normalized =
        normalize_small_text("content_schedules", "platform", platform)?.to_lowercase();
    match normalized.as_str() {
        "linkedin" | "instagram" | "tiktok" | "x" | "facebook" => Ok(normalized),
        other => Err(RepositoryError::Constraint {
            entity: "content_schedules",
            message: format!("unsupported content platform: {other}"),
        }),
    }
}
fn validate_asset_input(input: &MediaAssetInput) -> RepoResult<()> {
    match input.asset_kind.as_str() {
        "image" | "video" | "document" | "link" => {}
        other => {
            return Err(RepositoryError::Constraint {
                entity: "media_assets",
                message: format!("unsupported media asset kind: {other}"),
            })
        }
    }
    let storage_ref = input.storage_ref.trim();
    if storage_ref.is_empty()
        || storage_ref.contains("..")
        || storage_ref.starts_with('/')
        || storage_ref.to_ascii_lowercase().contains("secret")
        || storage_ref.to_ascii_lowercase().contains("token")
    {
        return Err(RepositoryError::Constraint {
            entity: "media_assets",
            message: "unsafe media storage reference".to_string(),
        });
    }
    if let Some(byte_size) = input.byte_size {
        if byte_size <= 0 || byte_size > 1024 * 1024 * 1024 {
            return Err(RepositoryError::Constraint {
                entity: "media_assets",
                message: "media byte size is out of bounds".to_string(),
            });
        }
    }
    if let Some(width) = input.width {
        if width <= 0 || width > 10000 {
            return Err(RepositoryError::Constraint {
                entity: "media_assets",
                message: "media width is out of bounds".to_string(),
            });
        }
    }
    if let Some(height) = input.height {
        if height <= 0 || height > 10000 {
            return Err(RepositoryError::Constraint {
                entity: "media_assets",
                message: "media height is out of bounds".to_string(),
            });
        }
    }
    if let Some(duration) = input.duration_seconds {
        if duration < 0 || duration > 60 * 60 {
            return Err(RepositoryError::Constraint {
                entity: "media_assets",
                message: "media duration is out of bounds".to_string(),
            });
        }
    }
    Ok(())
}
fn platforms_json(platforms: Option<Vec<String>>) -> RepoResult<String> {
    serde_json::to_string(&platforms.unwrap_or_default()).map_err(|e| {
        RepositoryError::InvalidJson {
            field: "platforms_json",
            message: e.to_string(),
        }
    })
}
fn content_plan_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentPlanRecord> {
    Ok(ContentPlanRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        pillar: row.get(2)?,
        status: row.get(3)?,
        owner_actor_type: row.get(4)?,
        metadata_json: row.get(5)?,
    })
}
fn content_piece_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentPieceRecord> {
    Ok(ContentPieceRecord {
        id: row.get(0)?,
        plan_id: row.get(1)?,
        title: row.get(2)?,
        body_markdown: row.get(3)?,
        status: row.get(4)?,
        platforms_json: row.get(5)?,
        required_gate: row.get(6)?,
        metadata_json: row.get(7)?,
    })
}
fn schedule_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentScheduleRecord> {
    Ok(ContentScheduleRecord {
        id: row.get(0)?,
        piece_id: row.get(1)?,
        platform: row.get(2)?,
        scheduled_for: row.get(3)?,
        status: row.get(4)?,
        confirmation_id: row.get(5)?,
    })
}
fn verification_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentVerificationRecord> {
    Ok(ContentVerificationRecord {
        id: row.get(0)?,
        piece_id: row.get(1)?,
        schedule_id: row.get(2)?,
        platform: row.get(3)?,
        action_type: row.get(4)?,
        outcome: row.get(5)?,
        provider_status: row.get(6)?,
        failure_report: row.get(7)?,
    })
}
fn media_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaAssetRecord> {
    Ok(MediaAssetRecord {
        id: row.get(0)?,
        piece_id: row.get(1)?,
        asset_kind: row.get(2)?,
        storage_ref: row.get(3)?,
        alt_text: row.get(4)?,
        metadata_json: row.get(5)?,
    })
}
fn gate_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentReviewGateRecord> {
    Ok(ContentReviewGateRecord {
        id: row.get(0)?,
        piece_id: row.get(1)?,
        gate_type: row.get(2)?,
        status: row.get(3)?,
        evidence_summary: row.get(4)?,
    })
}

pub(crate) fn create_content_plan(
    c: &Connection,
    input: ContentPlanInput,
) -> RepoResult<ContentPlanRecord> {
    let title = p5_text("content_plans", "title", &input.title)?;
    let id = p5_id("content-plan", &CONTENT_PLAN_COUNTER);
    let metadata = p5_json(input.metadata_json)?;
    c.execute("insert into content_plans (id,title,pillar,owner_actor_type,metadata_json) values (?1,?2,?3,?4,?5)", params![id,title,input.pillar.unwrap_or_default(),input.owner_actor_type.unwrap_or_else(||"human".to_string()),metadata]).map_err(|e|map_repository_error("content_plans",e))?;
    emit_content_event(
        c,
        "content_plan_created",
        "Content plan created",
        vec![("content_plan", &id, "created")],
    )?;
    c.query_row("select id,title,pillar,status,owner_actor_type,metadata_json from content_plans where id=?1", params![id], content_plan_from_row).map_err(|e|map_repository_error("content_plans",e))
}
pub(crate) fn list_content_plans(
    c: &Connection,
    req: ContentListRequest,
) -> RepoResult<Vec<ContentPlanRecord>> {
    let limit = req.limit.unwrap_or(50).clamp(1, 200);
    c.prepare("select id,title,pillar,status,owner_actor_type,metadata_json from content_plans order by updated_at desc limit ?1").and_then(|mut st| st.query_map(params![limit], content_plan_from_row)?.collect()).map_err(|e|map_repository_error("content_plans",e))
}
pub(crate) fn create_content_piece(
    c: &Connection,
    input: ContentPieceInput,
) -> RepoResult<ContentPieceRecord> {
    let title = p5_text("content_pieces", "title", &input.title)?;
    let id = p5_id("content-piece", &CONTENT_PIECE_COUNTER);
    let metadata = p5_json(input.metadata_json)?;
    let platforms = platforms_json(input.platforms)?;
    let gate = input
        .required_gate
        .unwrap_or_else(|| "specialist_review".to_string());
    c.execute("insert into content_pieces (id,plan_id,title,body_markdown,platforms_json,required_gate,metadata_json) values (?1,?2,?3,?4,?5,?6,?7)", params![id,input.plan_id,title,input.body_markdown.unwrap_or_default(),platforms,gate,metadata]).map_err(|e|map_repository_error("content_pieces",e))?;
    emit_content_event(
        c,
        "content_piece_created",
        "Content draft created",
        vec![("content_piece", &id, "created")],
    )?;
    read_content_piece(c, &id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "content_pieces",
        key: id,
    })
}
pub(crate) fn read_content_piece(
    c: &Connection,
    id: &str,
) -> RepoResult<Option<ContentPieceRecord>> {
    c.query_row("select id,plan_id,title,body_markdown,status,platforms_json,required_gate,metadata_json from content_pieces where id=?1", params![id], content_piece_from_row).optional().map_err(|e|map_repository_error("content_pieces",e))
}
pub(crate) fn list_content_pieces(
    c: &Connection,
    req: ContentListRequest,
) -> RepoResult<Vec<ContentPieceRecord>> {
    let limit = req.limit.unwrap_or(50).clamp(1, 200);
    c.prepare("select id,plan_id,title,body_markdown,status,platforms_json,required_gate,metadata_json from content_pieces order by updated_at desc limit ?1").and_then(|mut st| st.query_map(params![limit], content_piece_from_row)?.collect()).map_err(|e|map_repository_error("content_pieces",e))
}
pub(crate) fn update_content_piece_draft(
    c: &Connection,
    input: ContentPieceDraftInput,
) -> RepoResult<ContentPieceRecord> {
    let metadata = p5_json(input.metadata_json)?;
    let status = input.status.unwrap_or_else(|| "draft".to_string());
    validate_content_status(&status)?;
    c.execute("update content_pieces set body_markdown=?1,status=?2,metadata_json=?3,updated_at=current_timestamp where id=?4", params![input.body_markdown,status,metadata,input.piece_id]).map_err(|e|map_repository_error("content_pieces",e))?;
    emit_content_event(
        c,
        "content_piece_updated",
        "Content draft updated",
        vec![("content_piece", &input.piece_id, "updated")],
    )?;
    read_content_piece(c, &input.piece_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "content_pieces",
        key: input.piece_id,
    })
}
pub(crate) fn add_media_asset_reference(
    c: &Connection,
    input: MediaAssetInput,
) -> RepoResult<MediaAssetRecord> {
    read_content_piece(c, &input.piece_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "content_pieces",
        key: input.piece_id.clone(),
    })?;
    validate_asset_input(&input)?;
    let id = p5_id("media-asset", &MEDIA_ASSET_COUNTER);
    let metadata = p5_json(input.metadata_json)?;
    c.execute("insert into media_assets (id,piece_id,asset_kind,storage_ref,mime_type,byte_size,width,height,duration_seconds,alt_text,metadata_json) values (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![id,input.piece_id,input.asset_kind,input.storage_ref,input.mime_type,input.byte_size,input.width,input.height,input.duration_seconds,input.alt_text.unwrap_or_default(),metadata]).map_err(|e|map_repository_error("media_assets",e))?;
    emit_content_event(
        c,
        "media_asset_referenced",
        "Media asset reference added",
        vec![("media_asset", &id, "created")],
    )?;
    c.query_row("select id,piece_id,asset_kind,storage_ref,alt_text,metadata_json from media_assets where id=?1", params![id], media_from_row).map_err(|e|map_repository_error("media_assets",e))
}
pub(crate) fn list_media_asset_references(
    c: &Connection,
    piece_id: &str,
) -> RepoResult<Vec<MediaAssetRecord>> {
    c.prepare("select id,piece_id,asset_kind,storage_ref,alt_text,metadata_json from media_assets where piece_id=?1 order by created_at desc")
        .and_then(|mut st| st.query_map(params![piece_id], media_from_row)?.collect())
        .map_err(|e| map_repository_error("media_assets", e))
}
pub(crate) fn create_content_review_gate(
    c: &Connection,
    input: ContentReviewGateInput,
) -> RepoResult<ContentReviewGateRecord> {
    read_content_piece(c, &input.piece_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "content_pieces",
        key: input.piece_id.clone(),
    })?;
    let id = p5_id("content-gate", &CONTENT_REVIEW_GATE_COUNTER);
    let metadata = p5_json(input.metadata_json)?;
    c.execute("insert into content_review_gates (id,piece_id,gate_type,reviewer_actor_type,reviewer_actor_id,evidence_summary,metadata_json) values (?1,?2,?3,?4,?5,?6,?7)", params![id,input.piece_id,input.gate_type,input.reviewer_actor_type,input.reviewer_actor_id,input.evidence_summary.unwrap_or_default(),metadata]).map_err(|e|map_repository_error("content_review_gates",e))?;
    emit_content_event(
        c,
        "content_review_gate_created",
        "Content review gate created",
        vec![("content_review_gate", &id, "created")],
    )?;
    c.query_row("select id,piece_id,gate_type,status,evidence_summary from content_review_gates where id=?1", params![id], gate_from_row).map_err(|e|map_repository_error("content_review_gates",e))
}
pub(crate) fn decide_content_review_gate(
    c: &Connection,
    id: &str,
    approved: bool,
    input: ContentReviewGateDecisionInput,
) -> RepoResult<ContentReviewGateRecord> {
    let metadata = p5_json(input.metadata_json)?;
    let status = if approved { "approved" } else { "rejected" };
    c.execute("update content_review_gates set status=?1, reviewer_actor_type=?2, reviewer_actor_id=?3, evidence_summary=?4, metadata_json=?5, decided_at=current_timestamp where id=?6", params![status,input.reviewer_actor_type,input.reviewer_actor_id,input.evidence_summary,metadata,id]).map_err(|e|map_repository_error("content_review_gates",e))?;
    let gate = c.query_row("select id,piece_id,gate_type,status,evidence_summary from content_review_gates where id=?1", params![id], gate_from_row).map_err(|e|map_repository_error("content_review_gates",e))?;
    if approved {
        c.execute("update content_pieces set status='approved', updated_at=current_timestamp where id=?1 and status in ('draft','review_ready','blocked')", params![gate.piece_id]).map_err(|e| map_repository_error("content_pieces", e))?;
    }
    emit_content_event(
        c,
        "content_review_gate_decided",
        "Content review gate decided",
        vec![("content_review_gate", id, status)],
    )?;
    Ok(gate)
}
pub(crate) fn list_content_review_gates(
    c: &Connection,
    piece_id: &str,
) -> RepoResult<Vec<ContentReviewGateRecord>> {
    c.prepare("select id,piece_id,gate_type,status,evidence_summary from content_review_gates where piece_id=?1 order by created_at desc").and_then(|mut st| st.query_map(params![piece_id], gate_from_row)?.collect()).map_err(|e|map_repository_error("content_review_gates",e))
}
pub(crate) fn validate_platform_media_constraints(
    c: &Connection,
    piece_id: &str,
    platform: &str,
) -> RepoResult<PlatformConstraintResult> {
    let media_count:i64=c.query_row("select count(*) from media_assets where piece_id=?1 and asset_kind in ('image','video')",params![piece_id],|r|r.get(0)).map_err(|e|map_repository_error("media_assets",e))?;
    let mut violations = Vec::new();
    if ["instagram", "tiktok"].contains(&platform) && media_count == 0 {
        violations.push(format!("{platform}_requires_media"));
    }
    Ok(PlatformConstraintResult {
        platform: platform.to_string(),
        passed: violations.is_empty(),
        violations,
    })
}
pub(crate) fn create_content_schedule(
    c: &Connection,
    input: ContentScheduleInput,
) -> RepoResult<ContentScheduleRecord> {
    let piece =
        read_content_piece(c, &input.piece_id)?.ok_or_else(|| RepositoryError::NotFound {
            entity: "content_pieces",
            key: input.piece_id.clone(),
        })?;
    let platform = validate_platform(&input.platform)?;
    let constraint = validate_platform_media_constraints(c, &input.piece_id, &platform)?;
    if !constraint.passed {
        record_verification(
            c,
            Some(&input.piece_id),
            None,
            &platform,
            "validation",
            "blocked",
            Some(&format!(
                "platform media constraints failed: {}",
                constraint.violations.join(",")
            )),
        )?;
        return Err(RepositoryError::Constraint {
            entity: "content_schedules",
            message: format!(
                "platform media constraints failed: {}",
                constraint.violations.join(",")
            ),
        });
    }
    if piece.required_gate == "specialist_review" {
        let approved:i64=c.query_row("select count(*) from content_review_gates where piece_id=?1 and gate_type='specialist_review' and status='approved'",params![input.piece_id],|r|r.get(0)).map_err(|e|map_repository_error("content_review_gates",e))?;
        if approved == 0 {
            record_verification(
                c,
                Some(&piece.id),
                None,
                &platform,
                "review",
                "blocked",
                Some("schedule blocked by review gate"),
            )?;
            return Err(RepositoryError::Constraint {
                entity: "content_schedules",
                message: "schedule blocked by review gate".to_string(),
            });
        }
    }
    let confirmation = input
        .confirmation_id
        .as_deref()
        .map(|id| read_confirmation_decision(c, id))
        .transpose()?
        .flatten();
    let mut policy = evaluate_action_policy("publish_schedule_content");
    policy.reviewer_required = ReviewerRequirement::None;
    policy.requires_reviewer = false;
    policy.human_confirmation = HumanConfirmation::Yes;
    policy.requires_confirmation = true;
    policy.allowed_now = false;
    let request = ActionRequest::new(ActionType::Publish)
        .target("schedule content intent")
        .scope(ActionScope::External)
        .consequence(ActionConsequence::PublicRelease);
    let gate =
        require_policy_clearance_before_execution(&request, Some(&policy), confirmation.as_ref());
    if !gate.allowed_now {
        record_verification(
            c,
            Some(&piece.id),
            None,
            &platform,
            "schedule",
            "blocked",
            Some(&gate.reason),
        )?;
        return Err(RepositoryError::Constraint {
            entity: "content_schedules",
            message: format!(
                "schedule requires approved human confirmation: {}",
                gate.reason
            ),
        });
    }
    let id = p5_id("content-schedule", &CONTENT_SCHEDULE_COUNTER);
    let metadata = p5_json(input.metadata_json)?;
    c.execute("insert into content_schedules (id,piece_id,platform,scheduled_for,confirmation_id,metadata_json) values (?1,?2,?3,?4,?5,?6)",params![id,input.piece_id,platform,input.scheduled_for,input.confirmation_id,metadata]).map_err(|e|map_repository_error("content_schedules",e))?;
    c.execute(
        "update content_pieces set status='scheduled', updated_at=current_timestamp where id=?1",
        params![piece.id],
    )
    .map_err(|e| map_repository_error("content_pieces", e))?;
    emit_content_event(
        c,
        "content_schedule_intent_created",
        "Content schedule intent created",
        vec![("content_schedule", &id, "created")],
    )?;
    c.query_row("select id,piece_id,platform,scheduled_for,status,confirmation_id from content_schedules where id=?1",params![id],schedule_from_row).map_err(|e|map_repository_error("content_schedules",e))
}
pub(crate) fn list_content_schedules(
    c: &Connection,
    piece_id: Option<String>,
) -> RepoResult<Vec<ContentScheduleRecord>> {
    if let Some(pid) = piece_id {
        c.prepare("select id,piece_id,platform,scheduled_for,status,confirmation_id from content_schedules where piece_id=?1 order by scheduled_for asc").and_then(|mut st| st.query_map(params![pid],schedule_from_row)?.collect()).map_err(|e|map_repository_error("content_schedules",e))
    } else {
        c.prepare("select id,piece_id,platform,scheduled_for,status,confirmation_id from content_schedules order by scheduled_for asc limit 100").and_then(|mut st| st.query_map([],schedule_from_row)?.collect()).map_err(|e|map_repository_error("content_schedules",e))
    }
}
pub(crate) fn cancel_content_schedule(
    c: &Connection,
    schedule_id: &str,
) -> RepoResult<ContentScheduleRecord> {
    c.execute(
        "update content_schedules set status='cancelled', updated_at=current_timestamp where id=?1",
        params![schedule_id],
    )
    .map_err(|e| map_repository_error("content_schedules", e))?;
    c.query_row("select id,piece_id,platform,scheduled_for,status,confirmation_id from content_schedules where id=?1",params![schedule_id],schedule_from_row).map_err(|e|map_repository_error("content_schedules",e))
}
pub(crate) fn get_omnisocials_status(c: &Connection) -> RepoResult<OmniSocialsStatusRecord> {
    c.query_row("select state,platform,credential_ref,status_note from omnisocials_accounts where id='omnisocials-default'",[],|r|Ok(OmniSocialsStatusRecord{state:r.get(0)?,platform:r.get(1)?,credential_ref:r.get(2)?,status_note:r.get(3)?})).map_err(|e|map_repository_error("omnisocials_accounts",e))
}
fn record_verification(
    c: &Connection,
    piece_id: Option<&str>,
    schedule_id: Option<&str>,
    platform: &str,
    action: &str,
    outcome: &str,
    failure: Option<&str>,
) -> RepoResult<ContentVerificationRecord> {
    let id = p5_id("content-verification", &CONTENT_VERIFICATION_COUNTER);
    let failure_report = failure.map(|value| redact_secrets(value).text);
    c.execute("insert into content_verification_records (id,piece_id,schedule_id,platform,action_type,outcome,provider_status,failure_report,metadata_json) values (?1,?2,?3,?4,?5,?6,?7,?8,'{}')",params![id,piece_id,schedule_id,platform,action,outcome,Some("local_fail_closed"),failure_report]).map_err(|e|map_repository_error("content_verification_records",e))?;
    emit_content_event(
        c,
        "content_verification_recorded",
        "Content verification recorded",
        vec![("content_verification", &id, outcome)],
    )?;
    c.query_row("select id,piece_id,schedule_id,platform,action_type,outcome,provider_status,failure_report from content_verification_records where id=?1",params![id],verification_from_row).map_err(|e|map_repository_error("content_verification_records",e))
}
pub(crate) fn omnisocials_fail_closed(
    c: &Connection,
    piece_id: &str,
    schedule_id: Option<&str>,
    platform: &str,
    action: &str,
) -> RepoResult<ContentVerificationRecord> {
    read_content_piece(c, piece_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "content_pieces",
        key: piece_id.to_string(),
    })?;
    let platform = validate_platform(platform)?;
    if action != "upload" {
        let constraint = validate_platform_media_constraints(c, piece_id, &platform)?;
        if !constraint.passed {
            return record_verification(
                c,
                Some(piece_id),
                schedule_id,
                &platform,
                action,
                "blocked",
                Some(&format!(
                    "platform media constraints failed: {}",
                    constraint.violations.join(",")
                )),
            );
        }
    }
    if let Some(schedule_id) = schedule_id {
        c.query_row(
            "select id from content_schedules where id=?1 and piece_id=?2",
            params![schedule_id, piece_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| map_repository_error("content_schedules", e))?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "content_schedules",
            key: schedule_id.to_string(),
        })?;
    }
    let status = get_omnisocials_status(c)?;
    if status.state == "connected" && status.credential_ref.is_some() {
        return record_verification(c,Some(piece_id),schedule_id,&platform,action,"manual",Some("provider connection is present; external write still requires manual execution in Phase 5."));
    }
    record_verification(
        c,
        Some(piece_id),
        schedule_id,
        &platform,
        action,
        "blocked",
        Some(&status.status_note),
    )
}
pub(crate) fn omnisocials_upload_media(
    c: &Connection,
    piece_id: &str,
    platform: &str,
) -> RepoResult<ContentVerificationRecord> {
    omnisocials_fail_closed(c, piece_id, None, platform, "upload")
}
pub(crate) fn list_content_verification_records(
    c: &Connection,
    req: ContentVerificationListRequest,
) -> RepoResult<Vec<ContentVerificationRecord>> {
    let limit = req.limit.unwrap_or(50).clamp(1, 200);
    match (req.piece_id, req.schedule_id) { (Some(pid),_) => c.prepare("select id,piece_id,schedule_id,platform,action_type,outcome,provider_status,failure_report from content_verification_records where piece_id=?1 order by created_at desc limit ?2").and_then(|mut st| st.query_map(params![pid,limit],verification_from_row)?.collect()).map_err(|e|map_repository_error("content_verification_records",e)), (_,Some(sid)) => c.prepare("select id,piece_id,schedule_id,platform,action_type,outcome,provider_status,failure_report from content_verification_records where schedule_id=?1 order by created_at desc limit ?2").and_then(|mut st| st.query_map(params![sid,limit],verification_from_row)?.collect()).map_err(|e|map_repository_error("content_verification_records",e)), _ => c.prepare("select id,piece_id,schedule_id,platform,action_type,outcome,provider_status,failure_report from content_verification_records order by created_at desc limit ?1").and_then(|mut st| st.query_map(params![limit],verification_from_row)?.collect()).map_err(|e|map_repository_error("content_verification_records",e)) }
}
