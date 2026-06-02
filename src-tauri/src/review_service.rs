use rusqlite::Connection;

use crate::{
    create_review_record, read_agent_profile, read_agent_run_required, RepoResult, ReviewRecord,
    ReviewRecordCreateInput, ReviewSubjectType, ReviewVerdict,
};

const MANUAL_REVIEWER_PLACEHOLDER_PROFILE_ID: &str = "manual-reviewer";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ManualReviewServiceCreateInput {
    pub task_id: String,
    pub run_id: Option<String>,
    pub reviewer_profile_id: Option<String>,
    pub verdict: ReviewVerdict,
    pub evidence_summary: String,
    pub required_fixes_json: String,
    pub metadata_json: String,
}

#[allow(dead_code)]
pub(crate) fn create_manual_review_service(
    connection: &Connection,
    input: ManualReviewServiceCreateInput,
) -> RepoResult<ReviewRecord> {
    let reviewer_profile_id = resolve_reviewer_profile_id(connection, input.reviewer_profile_id)?;
    let (subject_type, subject_id) = match input.run_id.as_deref() {
        Some(run_id) => {
            let run = read_agent_run_required(connection, run_id)?;
            (ReviewSubjectType::AgentRun, run.id)
        }
        None => (ReviewSubjectType::Task, input.task_id.clone()),
    };

    create_review_record(
        connection,
        ReviewRecordCreateInput {
            subject_type,
            subject_id,
            task_id: input.task_id,
            run_id: input.run_id,
            reviewer_profile_id,
            verdict: input.verdict,
            evidence_summary: input.evidence_summary,
            required_fixes_json: input.required_fixes_json,
            metadata_json: input.metadata_json,
        },
    )
}

fn resolve_reviewer_profile_id(
    connection: &Connection,
    requested_profile_id: Option<String>,
) -> RepoResult<Option<String>> {
    match requested_profile_id {
        Some(profile_id) => Ok(Some(profile_id)),
        None => Ok(
            read_agent_profile(connection, MANUAL_REVIEWER_PLACEHOLDER_PROFILE_ID)?
                .map(|profile| profile.id),
        ),
    }
}
