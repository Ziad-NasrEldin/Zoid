mod agent_execution_service;
mod history_service;
mod notification_service;
mod review_service;
mod task_service;

#[allow(unused_imports)]
pub(crate) use agent_execution_service::*;
#[allow(unused_imports)]
pub(crate) use history_service::*;
#[allow(unused_imports)]
pub(crate) use notification_service::*;
#[allow(unused_imports)]
pub(crate) use review_service::*;
#[allow(unused_imports)]
pub(crate) use task_service::*;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static CONFIRMATION_DECISION_COUNTER: AtomicU64 = AtomicU64::new(0);
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);
static TASK_COUNTER: AtomicU64 = AtomicU64::new(0);
static CLI_SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);
static AGENT_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);
static REVIEW_RECORD_COUNTER: AtomicU64 = AtomicU64::new(0);
static NOTIFICATION_COUNTER: AtomicU64 = AtomicU64::new(0);
static ACTIVE_RUN_CHILDREN: OnceLock<Mutex<HashMap<String, Arc<Mutex<std::process::Child>>>>> =
    OnceLock::new();

const SAFE_LOG_MAX_BYTES: usize = 4096;
const SAFE_LOG_ROTATED_SUFFIX: &str = "1";
const EVENT_CREATE_MAX_TARGETS: usize = 25;
const EVENT_CREATE_MAX_SUMMARY_BYTES: usize = 4096;
const EVENT_CREATE_MAX_METADATA_JSON_BYTES: usize = 16_384;
const EVENT_CREATE_MAX_SMALL_FIELD_BYTES: usize = 256;
const EVENT_CREATE_MAX_SOURCE_BYTES: usize = 512;
const TASK_TITLE_MAX_BYTES: usize = 256;
const TASK_DETAIL_MAX_BYTES: usize = 4_096;

const WORKSPACE_REGISTRY: &[WorkspaceDefinition] = &[
    WorkspaceDefinition {
        key: "today",
        label: "Today",
        description: "Command center, attention, and current work.",
        position: 0,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local foundation surface is available; external integrations are surfaced through their owning workspaces.",
    },
    WorkspaceDefinition {
        key: "tasks",
        label: "Tasks",
        description: "First-class tasks, review states, and follow-ups.",
        position: 1,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local task foundation is available; later tasks will add richer task commands.",
    },
    WorkspaceDefinition {
        key: "notes",
        label: "Notes",
        description: "Markdown notes with local metadata.",
        position: 2,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local notes foundation is available in the visible Zoid directory; no remote sync is claimed.",
    },
    WorkspaceDefinition {
        key: "agents",
        label: "Agents",
        description: "CLI profiles, sessions, runs, and reviews.",
        position: 3,
        availability: WorkspaceAvailability::Available,
        integrations: &[WorkspaceIntegration {
            key: "hermes_cli",
            label: "Hermes CLI",
            state: WorkspaceIntegrationState::NotConfigured,
            note: "CLI/session discovery is not wired in this backend registry task.",
        }],
        status_note: "Local agent workspace shell is available; CLI integration is not configured by this service.",
    },
    WorkspaceDefinition {
        key: "code",
        label: "Code",
        description: "Repositories, Launch Gate, and git work.",
        position: 4,
        availability: WorkspaceAvailability::Available,
        integrations: &[WorkspaceIntegration {
            key: "git_cli",
            label: "Git CLI",
            state: WorkspaceIntegrationState::NotConfigured,
            note: "Repository and git probing are intentionally deferred to later backend work.",
        }],
        status_note: "Local code workspace shell is available; git/Launch Gate integrations are not connected here.",
    },
    WorkspaceDefinition {
        key: "content",
        label: "Content",
        description: "Planning, review, and OmniSocials publishing state.",
        position: 5,
        availability: WorkspaceAvailability::Available,
        integrations: &[WorkspaceIntegration {
            key: "omnisocials",
            label: "OmniSocials",
            state: WorkspaceIntegrationState::Planned,
            note: "Publishing integration is planned and has no OAuth/API connection in this task.",
        }],
        status_note: "Local content planning surface is available; publishing integrations are planned only.",
    },
    WorkspaceDefinition {
        key: "automations",
        label: "Automations",
        description: "Visible recurring jobs and run history.",
        position: 6,
        availability: WorkspaceAvailability::Available,
        integrations: &[WorkspaceIntegration {
            key: "automation_cli",
            label: "Automation CLI",
            state: WorkspaceIntegrationState::NotConfigured,
            note: "Automation runner discovery/control is not implemented in this registry task.",
        }],
        status_note: "Local automation workspace shell is available; runner integration remains not configured.",
    },
    WorkspaceDefinition {
        key: "business",
        label: "Business",
        description: "Contacts, companies, follow-ups, and linked work.",
        position: 7,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local business workspace foundation is available; external CRM sync is not claimed.",
    },
    WorkspaceDefinition {
        key: "products",
        label: "Products",
        description: "First-class product hubs and timelines.",
        position: 8,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local product workspace foundation is available; no external product tool integration is claimed.",
    },
    WorkspaceDefinition {
        key: "files",
        label: "Files",
        description: "Local file manager and Zoid-aware attachments.",
        position: 9,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local visible-file foundation is available; no cloud drive connection is claimed.",
    },
    WorkspaceDefinition {
        key: "browser",
        label: "Browser",
        description: "Work webview/capture workspace.",
        position: 10,
        availability: WorkspaceAvailability::Planned,
        integrations: &[WorkspaceIntegration {
            key: "browser_webview",
            label: "Browser Webview",
            state: WorkspaceIntegrationState::Planned,
            note: "Browser/web capture is planned and not implemented by this backend registry task.",
        }],
        status_note: "Browser workspace is listed for canonical navigation, but webview/capture functionality is planned.",
    },
    WorkspaceDefinition {
        key: "inbox",
        label: "Inbox",
        description: "Notifications, approvals, blockers, and Gmail state.",
        position: 11,
        availability: WorkspaceAvailability::Available,
        integrations: &[WorkspaceIntegration {
            key: "gmail",
            label: "Gmail",
            state: WorkspaceIntegrationState::NotConfigured,
            note: "No Gmail OAuth, token, or API check is performed in this task.",
        }],
        status_note: "local inbox/approval foundation is available; Gmail is not configured or connected.",
    },
    WorkspaceDefinition {
        key: "calendar",
        label: "Calendar",
        description: "Built-in calendar with Apple Calendar integration gates.",
        position: 12,
        availability: WorkspaceAvailability::Available,
        integrations: &[WorkspaceIntegration {
            key: "apple_calendar",
            label: "Apple Calendar",
            state: WorkspaceIntegrationState::NeedsPermission,
            note: "No EventKit prompt or permission check is performed in this task.",
        }],
        status_note: "Local calendar workspace foundation is available; Apple Calendar access needs permission and later integration work.",
    },
    WorkspaceDefinition {
        key: "history",
        label: "History",
        description: "Universal timeline and linked event history.",
        position: 13,
        availability: WorkspaceAvailability::Available,
        integrations: &[],
        status_note: "Local history/event foundation is available from the application database.",
    },
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
    "external_api_write",
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
    Migration {
        version: 4,
        name: "confirmation_actor_type_check",
        sql: include_str!("../migrations/0004_confirmation_actor_type_check.sql"),
    },
    Migration {
        version: 5,
        name: "phase2_tasks",
        sql: include_str!("../migrations/0005_phase2_tasks.sql"),
    },
    Migration {
        version: 6,
        name: "phase2_agent_runs",
        sql: include_str!("../migrations/0006_phase2_agent_runs.sql"),
    },
    Migration {
        version: 7,
        name: "phase2_review_records",
        sql: include_str!("../migrations/0007_phase2_review_records.sql"),
    },
    Migration {
        version: 8,
        name: "phase2_notifications",
        sql: include_str!("../migrations/0008_phase2_notifications.sql"),
    },
    Migration {
        version: 9,
        name: "phase3_notes_files_knowledge",
        sql: include_str!("../migrations/0009_phase3_notes_files_knowledge.sql"),
    },
];

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum WorkspaceAvailability {
    Available,
    Planned,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum WorkspaceIntegrationState {
    NotConfigured,
    NeedsPermission,
    Planned,
    Blocked,
}

impl WorkspaceIntegrationState {
    fn as_str(self) -> &'static str {
        match self {
            WorkspaceIntegrationState::NotConfigured => "not_configured",
            WorkspaceIntegrationState::NeedsPermission => "needs_permission",
            WorkspaceIntegrationState::Planned => "planned",
            WorkspaceIntegrationState::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct WorkspaceIntegration {
    key: &'static str,
    label: &'static str,
    state: WorkspaceIntegrationState,
    note: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkspaceDefinition {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    position: i64,
    availability: WorkspaceAvailability,
    integrations: &'static [WorkspaceIntegration],
    status_note: &'static str,
}

#[derive(Debug, Serialize)]
struct WorkspaceRecord {
    id: String,
    label: String,
    description: String,
    position: i64,
    availability: WorkspaceAvailability,
    integrations: Vec<WorkspaceIntegration>,
    status_note: String,
}

impl WorkspaceRecord {
    #[cfg(test)]
    fn integration_state(&self, key: &str) -> Option<WorkspaceIntegrationState> {
        self.integrations
            .iter()
            .find(|integration| integration.key == key)
            .map(|integration| integration.state)
    }
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
    keychain: KeychainReadinessStatus,
    keychain_status: String,
    sample_policy: ActionPolicyDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct KeychainReadinessStatus {
    ready: bool,
    status: String,
    reason: String,
    credential_storage_enabled: bool,
    test_path_exercised: bool,
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
    allowed_now: bool,
    requires_confirmation: bool,
    requires_reviewer: bool,
    requires_clear_task: bool,
    requires_gate: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ActionType {
    Read,
    Create,
    Update,
    Delete,
    Send,
    Publish,
    Deploy,
    File,
    Process,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ActionScope {
    LocalPrivate,
    LocalVisible,
    CodeRepository,
    Integration,
    External,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ActionConsequence {
    HarmlessLocal,
    LocalWrite,
    ExternalWrite,
    PublicRelease,
    Destructive,
    AutomationExecution,
    CredentialOrIntegrationChange,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ActionRequest {
    action_type: ActionType,
    target: Option<String>,
    scope: Option<ActionScope>,
    consequence: Option<ActionConsequence>,
    bulk: bool,
    destructive: bool,
}

#[allow(dead_code)]
impl ActionRequest {
    fn new(action_type: ActionType) -> Self {
        Self {
            action_type,
            target: None,
            scope: None,
            consequence: None,
            bulk: false,
            destructive: false,
        }
    }

    fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    fn scope(mut self, scope: ActionScope) -> Self {
        self.scope = Some(scope);
        self
    }

    fn consequence(mut self, consequence: ActionConsequence) -> Self {
        self.consequence = Some(consequence);
        self
    }

    fn bulk(mut self, bulk: bool) -> Self {
        self.bulk = bulk;
        self
    }

    fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConfirmationDecisionState {
    Approved,
    Denied,
    Cancelled,
    Expired,
}

#[allow(dead_code)]
impl ConfirmationDecisionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Denied => "denied",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "approved" => Ok(Self::Approved),
            "denied" => Ok(Self::Denied),
            "cancelled" => Ok(Self::Cancelled),
            "expired" => Ok(Self::Expired),
            other => Err(RepositoryError::Constraint {
                entity: "confirmation_decisions",
                message: format!("invalid confirmation decision: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConfirmationActorType {
    Human,
    Reviewer,
    ClearTask,
    ReviewedClearTask,
    System,
}

#[allow(dead_code)]
impl ConfirmationActorType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Reviewer => "reviewer",
            Self::ClearTask => "clear_task",
            Self::ReviewedClearTask => "reviewed_clear_task",
            Self::System => "system",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "human" => Ok(Self::Human),
            "reviewer" => Ok(Self::Reviewer),
            "clear_task" => Ok(Self::ClearTask),
            "reviewed_clear_task" => Ok(Self::ReviewedClearTask),
            "system" => Ok(Self::System),
            other => Err(RepositoryError::Constraint {
                entity: "confirmation_decisions",
                message: format!("invalid confirmation actor_type: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfirmationActor {
    actor_type: ConfirmationActorType,
    actor_id: Option<String>,
}

#[allow(dead_code)]
impl ConfirmationActor {
    fn human(actor_id: Option<&str>) -> Self {
        Self::new(ConfirmationActorType::Human, actor_id)
    }

    fn reviewer(actor_id: Option<&str>) -> Self {
        Self::new(ConfirmationActorType::Reviewer, actor_id)
    }

    fn new(actor_type: ConfirmationActorType, actor_id: Option<&str>) -> Self {
        Self {
            actor_type,
            actor_id: actor_id.map(str::to_string),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ConfirmationDecisionRequest<'a> {
    action_category: &'a str,
    decision: ConfirmationDecisionState,
    actor: ConfirmationActor,
    summary: &'a str,
    event_id: Option<&'a str>,
    metadata_json: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ConfirmationDecisionRecord {
    id: String,
    action_category: String,
    decision: ConfirmationDecisionState,
    actor_type: ConfirmationActorType,
    actor_id: Option<String>,
    summary: String,
    event_id: Option<String>,
    metadata_json: String,
    created_at: String,
}

#[allow(dead_code)]
impl ConfirmationDecisionRecord {
    fn new_for_test(
        id: &str,
        action_category: &str,
        decision: ConfirmationDecisionState,
        actor_type: ConfirmationActorType,
    ) -> Self {
        Self {
            id: id.to_string(),
            action_category: normalize_action_category(action_category),
            decision,
            actor_type,
            actor_id: None,
            summary: "test confirmation".to_string(),
            event_id: None,
            metadata_json: "{}".to_string(),
            created_at: "test".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ExecutionGateResult {
    allowed_now: bool,
    reason: String,
    action_category: String,
    requires_confirmation: bool,
    requires_reviewer: bool,
    requires_clear_task: bool,
    confirmation_id: Option<String>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EventTargetInput<'a> {
    entity_type: &'a str,
    entity_id: &'a str,
    relation_type: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EventTargetRecord {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EventCreateInput<'a> {
    action_type: &'a str,
    outcome: &'a str,
    actor_type: &'a str,
    actor_id: Option<&'a str>,
    workspace_key: Option<&'a str>,
    summary: &'a str,
    source: &'a str,
    metadata_json: &'a str,
    targets: Vec<EventTargetInput<'a>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EventRecord {
    id: String,
    action_type: String,
    outcome: String,
    timestamp: String,
    actor_type: String,
    actor_id: Option<String>,
    workspace_key: Option<String>,
    summary: String,
    source: String,
    metadata_json: String,
    targets: Vec<EventTargetRecord>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct EventListFilter<'a> {
    workspace_key: Option<&'a str>,
    action_type: Option<&'a str>,
    outcome: Option<&'a str>,
    source: Option<&'a str>,
    limit: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TaskStatus {
    Inbox,
    Planned,
    Active,
    Waiting,
    ReviewRequired,
    Blocked,
    Completed,
    Failed,
    Cancelled,
    Archived,
    Deleted,
}

#[allow(dead_code)]
impl TaskStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Waiting => "waiting",
            Self::ReviewRequired => "review_required",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "inbox" => Ok(Self::Inbox),
            "planned" => Ok(Self::Planned),
            "active" => Ok(Self::Active),
            "waiting" => Ok(Self::Waiting),
            "review_required" => Ok(Self::ReviewRequired),
            "blocked" => Ok(Self::Blocked),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "archived" => Ok(Self::Archived),
            "deleted" => Ok(Self::Deleted),
            other => Err(RepositoryError::Constraint {
                entity: "tasks",
                message: format!("invalid task status: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[allow(dead_code)]
impl TaskPriority {
    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "low" => Ok(Self::Low),
            "normal" => Ok(Self::Normal),
            "high" => Ok(Self::High),
            "urgent" => Ok(Self::Urgent),
            other => Err(RepositoryError::Constraint {
                entity: "tasks",
                message: format!("invalid task priority: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TaskRecord {
    id: String,
    title: String,
    detail: Option<String>,
    status: TaskStatus,
    priority: TaskPriority,
    workspace_key: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
    deleted_at: Option<String>,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TaskCreateInput {
    title: String,
    detail: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
    workspace_key: Option<String>,
    metadata_json: String,
}

#[allow(dead_code)]
impl TaskCreateInput {
    fn new(title: &str, priority: Option<TaskPriority>) -> Self {
        Self {
            title: title.to_string(),
            detail: None,
            status: None,
            priority,
            workspace_key: None,
            metadata_json: "{}".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AgentProfileRecord {
    id: String,
    label: String,
    configured: bool,
    command: Option<String>,
    config_json: String,
    capabilities_json: String,
    credential_ref: Option<String>,
    env_refs_json: String,
    metadata_json: String,
    created_at: String,
    updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AgentProfileInput {
    id: String,
    label: String,
    configured: bool,
    command: Option<String>,
    config_json: String,
    capabilities_json: String,
    credential_ref: Option<String>,
    env_refs_json: String,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CliSessionRecord {
    id: String,
    task_id: String,
    profile_id: String,
    mode: String,
    cwd: String,
    status: String,
    status_summary: String,
    metadata_json: String,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CliSessionCreateInput {
    task_id: String,
    profile_id: String,
    mode: String,
    cwd: String,
    status_summary: String,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum AgentRunStatus {
    Queued,
    Starting,
    Running,
    WaitingForInput,
    ReviewRequired,
    Completed,
    Failed,
    Cancelled,
    Blocked,
}

#[allow(dead_code)]
impl AgentRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::WaitingForInput => "waiting_for_input",
            Self::ReviewRequired => "review_required",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Blocked => "blocked",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "queued" => Ok(Self::Queued),
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "waiting_for_input" => Ok(Self::WaitingForInput),
            "review_required" => Ok(Self::ReviewRequired),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "blocked" => Ok(Self::Blocked),
            other => Err(RepositoryError::Constraint {
                entity: "agent_runs",
                message: format!("invalid agent run status: {other}"),
            }),
        }
    }

    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Blocked
        )
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Queued => "run.queued",
            Self::Starting | Self::Running => "run.started",
            Self::WaitingForInput => "run.waiting_for_input",
            Self::ReviewRequired => "run.review_required",
            Self::Completed => "run.completed",
            Self::Failed => "run.failed",
            Self::Cancelled => "run.cancelled",
            Self::Blocked => "run.blocked",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewState {
    NotRequired,
    Required,
    Requested,
    InProgress,
    Approved,
    RequiredFixes,
    BlockedInsufficientEvidence,
    Failed,
    Cancelled,
}

#[allow(dead_code)]
impl ReviewState {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not_required",
            Self::Required => "required",
            Self::Requested => "requested",
            Self::InProgress => "in_progress",
            Self::Approved => "approved",
            Self::RequiredFixes => "required_fixes",
            Self::BlockedInsufficientEvidence => "blocked_insufficient_evidence",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "not_required" => Ok(Self::NotRequired),
            "required" => Ok(Self::Required),
            "requested" => Ok(Self::Requested),
            "in_progress" => Ok(Self::InProgress),
            "approved" => Ok(Self::Approved),
            "required_fixes" => Ok(Self::RequiredFixes),
            "blocked_insufficient_evidence" => Ok(Self::BlockedInsufficientEvidence),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(RepositoryError::Constraint {
                entity: "agent_runs",
                message: format!("invalid review state: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AgentRunRecord {
    id: String,
    task_id: String,
    profile_id: String,
    session_id: String,
    cwd: String,
    command_snapshot: String,
    profile_snapshot_json: String,
    status: AgentRunStatus,
    created_at: String,
    updated_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    duration_ms: Option<i64>,
    exit_code: Option<i64>,
    log_reference_id: Option<String>,
    output_summary: Option<String>,
    error_summary: Option<String>,
    review_state: ReviewState,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AgentRunCreateInput {
    task_id: String,
    profile_id: String,
    session_id: String,
    cwd: String,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AgentRunTransitionInput {
    output_summary: Option<String>,
    error_summary: Option<String>,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AgentRunCompletionInput {
    status: AgentRunStatus,
    duration_ms: i64,
    exit_code: Option<i64>,
    log_reference_id: Option<String>,
    output_summary: String,
    error_summary: Option<String>,
    review_state: ReviewState,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewSubjectType {
    Task,
    AgentRun,
    RelatedEntity,
}

#[allow(dead_code)]
impl ReviewSubjectType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::AgentRun => "agent_run",
            Self::RelatedEntity => "related_entity",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "task" => Ok(Self::Task),
            "agent_run" => Ok(Self::AgentRun),
            "related_entity" => Ok(Self::RelatedEntity),
            other => Err(RepositoryError::Constraint {
                entity: "review_records",
                message: format!("invalid review subject_type: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewVerdict {
    Approved,
    RequiredFixes,
    BlockedInsufficientEvidence,
}

#[allow(dead_code)]
impl ReviewVerdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::RequiredFixes => "required_fixes",
            Self::BlockedInsufficientEvidence => "blocked_insufficient_evidence",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "approved" => Ok(Self::Approved),
            "required_fixes" => Ok(Self::RequiredFixes),
            "blocked_insufficient_evidence" => Ok(Self::BlockedInsufficientEvidence),
            other => Err(RepositoryError::Constraint {
                entity: "review_records",
                message: format!("invalid review verdict: {other}"),
            }),
        }
    }

    fn state(self) -> ReviewState {
        match self {
            Self::Approved => ReviewState::Approved,
            Self::RequiredFixes => ReviewState::RequiredFixes,
            Self::BlockedInsufficientEvidence => ReviewState::BlockedInsufficientEvidence,
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Approved => "review.approved",
            Self::RequiredFixes => "review.required_fixes",
            Self::BlockedInsufficientEvidence => "review.blocked_insufficient_evidence",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ReviewRecord {
    id: String,
    subject_type: ReviewSubjectType,
    subject_id: String,
    task_id: String,
    run_id: Option<String>,
    reviewer_profile_id: Option<String>,
    state: ReviewState,
    verdict: ReviewVerdict,
    evidence_summary: String,
    required_fixes_json: String,
    metadata_json: String,
    created_at: String,
    updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ReviewRecordCreateInput {
    subject_type: ReviewSubjectType,
    subject_id: String,
    task_id: String,
    run_id: Option<String>,
    reviewer_profile_id: Option<String>,
    verdict: ReviewVerdict,
    evidence_summary: String,
    required_fixes_json: String,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NotificationType {
    Completion,
    Blocker,
    Failure,
    ReviewRequired,
    Attention,
}

#[allow(dead_code)]
impl NotificationType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completion => "completion",
            Self::Blocker => "blocker",
            Self::Failure => "failure",
            Self::ReviewRequired => "review_required",
            Self::Attention => "attention",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "completion" => Ok(Self::Completion),
            "blocker" => Ok(Self::Blocker),
            "failure" => Ok(Self::Failure),
            "review_required" => Ok(Self::ReviewRequired),
            "attention" => Ok(Self::Attention),
            other => Err(RepositoryError::Constraint {
                entity: "notifications",
                message: format!("invalid notification_type: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

#[allow(dead_code)]
impl NotificationSeverity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "info" => Ok(Self::Info),
            "success" => Ok(Self::Success),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            "critical" => Ok(Self::Critical),
            other => Err(RepositoryError::Constraint {
                entity: "notifications",
                message: format!("invalid notification severity: {other}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NotificationState {
    Pending,
    Delivered,
    Read,
    ActionRequired,
    Resolved,
    Dismissed,
    Failed,
}

#[allow(dead_code)]
impl NotificationState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivered => "delivered",
            Self::Read => "read",
            Self::ActionRequired => "action_required",
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
            Self::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "delivered" => Ok(Self::Delivered),
            "read" => Ok(Self::Read),
            "action_required" => Ok(Self::ActionRequired),
            "resolved" => Ok(Self::Resolved),
            "dismissed" => Ok(Self::Dismissed),
            "failed" => Ok(Self::Failed),
            other => Err(RepositoryError::Constraint {
                entity: "notifications",
                message: format!("invalid notification state: {other}"),
            }),
        }
    }

    fn is_active_inbox(self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Delivered | Self::ActionRequired | Self::Failed
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NotificationRecord {
    id: String,
    notification_type: NotificationType,
    title: String,
    message: String,
    severity: NotificationSeverity,
    state: NotificationState,
    action_route: Option<String>,
    task_id: Option<String>,
    run_id: Option<String>,
    review_record_id: Option<String>,
    read_at: Option<String>,
    dismissed_at: Option<String>,
    resolved_at: Option<String>,
    created_at: String,
    updated_at: String,
    metadata_json: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct NotificationCreateInput {
    notification_type: NotificationType,
    title: String,
    message: String,
    severity: NotificationSeverity,
    action_route: Option<String>,
    task_id: Option<String>,
    run_id: Option<String>,
    review_record_id: Option<String>,
    metadata_json: String,
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
    SecretRejected {
        field: &'static str,
        message: String,
    },
    Database {
        message: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SettingScope {
    App,
    Workspace,
}

impl SettingScope {
    fn as_str(self) -> &'static str {
        match self {
            SettingScope::App => "app",
            SettingScope::Workspace => "workspace",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "app" => Ok(SettingScope::App),
            "workspace" => Ok(SettingScope::Workspace),
            _ => Err(RepositoryError::Constraint {
                entity: "app_settings",
                message: format!("unsupported local preference scope: {value}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalPreferenceRecord {
    key: String,
    value_json: String,
    value_type: String,
    scope: SettingScope,
    description: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct LocalPreferenceInput<'a> {
    key: &'a str,
    value_json: &'a str,
    value_type: &'a str,
    scope: SettingScope,
    description: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum IntegrationStatus {
    NotConfigured,
    Configured,
    Connected,
    Degraded,
    Disabled,
    Error,
}

impl IntegrationStatus {
    fn as_str(self) -> &'static str {
        match self {
            IntegrationStatus::NotConfigured => "not_configured",
            IntegrationStatus::Configured => "configured",
            IntegrationStatus::Connected => "connected",
            IntegrationStatus::Degraded => "degraded",
            IntegrationStatus::Disabled => "disabled",
            IntegrationStatus::Error => "error",
        }
    }

    fn from_str(value: &str) -> RepoResult<Self> {
        match value {
            "not_configured" => Ok(IntegrationStatus::NotConfigured),
            "configured" => Ok(IntegrationStatus::Configured),
            "connected" => Ok(IntegrationStatus::Connected),
            "degraded" => Ok(IntegrationStatus::Degraded),
            "disabled" => Ok(IntegrationStatus::Disabled),
            "error" => Ok(IntegrationStatus::Error),
            _ => Err(RepositoryError::Constraint {
                entity: "integration_statuses",
                message: format!("unsupported integration status: {value}"),
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NoteConflictState {
    None,
    DuplicateId,
    PathMissing,
    ExternalEdit,
    ManualRename,
    MetadataMismatch,
}

#[allow(dead_code)]
impl NoteConflictState {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::DuplicateId => "duplicate_id",
            Self::PathMissing => "path_missing",
            Self::ExternalEdit => "external_edit",
            Self::ManualRename => "manual_rename",
            Self::MetadataMismatch => "metadata_mismatch",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NoteIdentityMetadata {
    id: String,
    title: String,
    slug: String,
    relative_path: String,
    frontmatter_json: String,
    body_digest: String,
    conflict_state: NoteConflictState,
}

#[allow(dead_code)]
fn derive_note_identity_from_markdown(
    relative_path: &str,
    markdown: &str,
) -> RepoResult<NoteIdentityMetadata> {
    validate_note_relative_path(relative_path)?;
    let (frontmatter, body) = split_markdown_frontmatter(markdown);
    let parsed_id = frontmatter
        .as_ref()
        .and_then(|lines| yaml_scalar_value(lines, "zoid_id"))
        .or_else(|| {
            frontmatter
                .as_ref()
                .and_then(|lines| yaml_scalar_value(lines, "id"))
        });
    let title = frontmatter
        .as_ref()
        .and_then(|lines| yaml_scalar_value(lines, "title"))
        .or_else(|| first_markdown_heading(body))
        .unwrap_or_else(|| title_from_relative_path(relative_path));
    let slug = frontmatter
        .as_ref()
        .and_then(|lines| yaml_scalar_value(lines, "slug"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| slugify_note_title(&title));
    let id = match parsed_id {
        Some(value) => {
            validate_note_id(&value)?;
            value
        }
        None => stable_note_id_from_relative_path(relative_path),
    };
    validate_note_title(&title)?;
    validate_note_slug(&slug)?;

    let body_digest = format!("fnv1a64:{:016x}", fnv1a64(body.as_bytes()));
    let frontmatter_json = serde_json::json!({
        "zoid_id": id,
        "title": title,
        "slug": slug,
        "relative_path": relative_path,
        "body_digest": body_digest,
    })
    .to_string();

    Ok(NoteIdentityMetadata {
        id,
        title,
        slug,
        relative_path: relative_path.to_string(),
        frontmatter_json,
        body_digest,
        conflict_state: NoteConflictState::None,
    })
}

#[allow(dead_code)]
fn write_note_identity_frontmatter(
    markdown: &str,
    identity: &NoteIdentityMetadata,
) -> RepoResult<String> {
    validate_note_id(&identity.id)?;
    validate_note_title(&identity.title)?;
    validate_note_slug(&identity.slug)?;
    validate_note_relative_path(&identity.relative_path)?;

    let (frontmatter, body) = split_markdown_frontmatter(markdown);
    let mut lines = frontmatter.unwrap_or_default();
    set_yaml_scalar(&mut lines, "zoid_id", &identity.id);
    set_yaml_scalar(&mut lines, "title", &identity.title);
    set_yaml_scalar(&mut lines, "slug", &identity.slug);

    let mut output = String::from("---\n");
    for line in lines {
        output.push_str(&line);
        output.push('\n');
    }
    output.push_str("---\n");
    if !body.starts_with('\n') {
        output.push('\n');
    }
    output.push_str(body);
    Ok(output)
}

#[allow(dead_code)]
fn upsert_note_identity_metadata(
    connection: &Connection,
    identity: &NoteIdentityMetadata,
) -> RepoResult<()> {
    validate_note_id(&identity.id)?;
    validate_note_title(&identity.title)?;
    validate_note_slug(&identity.slug)?;
    validate_note_relative_path(&identity.relative_path)?;
    validate_json_field("frontmatter_json", &identity.frontmatter_json)?;

    let existing_path = connection
        .query_row(
            "select relative_path from notes where id = ?1 and deleted_at is null",
            params![identity.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| map_repository_error("notes", error))?;
    if let Some(existing_path) = existing_path {
        if existing_path != identity.relative_path {
            connection
                .execute(
                    "update notes set status = 'conflicted', conflict_state = 'duplicate_id', updated_at = current_timestamp where id = ?1",
                    params![identity.id],
                )
                .map_err(|error| map_repository_error("notes", error))?;
            return Err(RepositoryError::Constraint {
                entity: "notes",
                message: format!(
                    "duplicate_id: note {} already indexed at {}",
                    identity.id, existing_path
                ),
            });
        }
    }

    connection
        .execute(
            "insert into notes (id, title, slug, relative_path, status, conflict_state, frontmatter_json, body_digest, metadata_json)
             values (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?7, '{}')
             on conflict(id) do update set
                title = excluded.title,
                slug = excluded.slug,
                relative_path = excluded.relative_path,
                status = 'active',
                conflict_state = excluded.conflict_state,
                frontmatter_json = excluded.frontmatter_json,
                body_digest = excluded.body_digest,
                updated_at = current_timestamp,
                deleted_at = null",
            params![
                identity.id,
                identity.title,
                identity.slug,
                identity.relative_path,
                identity.conflict_state.as_str(),
                identity.frontmatter_json,
                identity.body_digest
            ],
        )
        .map_err(|error| map_repository_error("notes", error))?;

    let index_id = format!("knowledge_note_frontmatter_{}", identity.id);
    connection
        .execute(
            "insert into knowledge_index_entries (id, entity_type, entity_id, source_type, title, excerpt, search_text, content_digest, scan_state, metadata_json)
             values (?1, 'note', ?2, 'markdown_frontmatter', ?3, ?4, ?5, ?6, 'current', ?7)
             on conflict(entity_type, entity_id, source_type) do update set
                title = excluded.title,
                excerpt = excluded.excerpt,
                search_text = excluded.search_text,
                content_digest = excluded.content_digest,
                scan_state = 'current',
                indexed_at = current_timestamp,
                metadata_json = excluded.metadata_json",
            params![
                index_id,
                identity.id,
                identity.title,
                identity.slug,
                format!("{} {} {}", identity.title, identity.slug, identity.relative_path),
                identity.body_digest,
                identity.frontmatter_json
            ],
        )
        .map_err(|error| map_repository_error("knowledge_index_entries", error))?;

    Ok(())
}

fn split_markdown_frontmatter(markdown: &str) -> (Option<Vec<String>>, &str) {
    if !markdown.starts_with("---\n") {
        return (None, markdown);
    }
    let rest = &markdown[4..];
    if let Some(end) = rest.find("\n---") {
        let frontmatter = rest[..end].lines().map(|line| line.to_string()).collect();
        let body_start = end + "\n---".len();
        let body = rest[body_start..]
            .strip_prefix('\n')
            .unwrap_or(&rest[body_start..]);
        return (Some(frontmatter), body);
    }
    (None, markdown)
}

fn yaml_scalar_value(lines: &[String], key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    lines.iter().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix(&prefix)
            .map(|value| parse_yaml_scalar_value(value.trim()))
    })
}

fn parse_yaml_scalar_value(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return unescape_yaml_double_quoted_scalar(&value[1..value.len() - 1]);
    }
    value.trim_matches('\'').to_string()
}

fn unescape_yaml_double_quoted_scalar(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                match escaped {
                    '"' => output.push('"'),
                    '\\' => output.push('\\'),
                    'n' => output.push('\n'),
                    't' => output.push('\t'),
                    other => {
                        output.push('\\');
                        output.push(other);
                    }
                }
            } else {
                output.push('\\');
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn set_yaml_scalar(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{key}:");
    let rendered = format!("{key}: {}", yaml_safe_scalar(value));
    if let Some(line) = lines
        .iter_mut()
        .find(|line| line.trim().starts_with(&prefix))
    {
        *line = rendered;
    } else {
        lines.push(rendered);
    }
}

fn yaml_safe_scalar(value: &str) -> String {
    let normalized = value.replace('\n', " ").trim().to_string();
    let escaped = normalized.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn first_markdown_heading(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(str::to_string)
    })
}

fn title_from_relative_path(relative_path: &str) -> String {
    let stem = relative_path
        .rsplit('/')
        .next()
        .unwrap_or(relative_path)
        .trim_end_matches(".md");
    stem.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slugify_note_title(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn validate_note_id(id: &str) -> RepoResult<()> {
    let valid = id.starts_with("note_")
        && id.len() <= 96
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if valid {
        Ok(())
    } else {
        Err(RepositoryError::Constraint {
            entity: "notes",
            message: format!("invalid stable note id: {id}"),
        })
    }
}

fn validate_note_title(title: &str) -> RepoResult<()> {
    if !title.trim().is_empty() && title.len() <= 256 {
        Ok(())
    } else {
        Err(RepositoryError::Constraint {
            entity: "notes",
            message: "note title must be non-empty and at most 256 bytes".to_string(),
        })
    }
}

fn validate_note_slug(slug: &str) -> RepoResult<()> {
    if !slug.trim().is_empty()
        && slug.len() <= 160
        && slug
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        Ok(())
    } else {
        Err(RepositoryError::Constraint {
            entity: "notes",
            message: format!("invalid note slug: {slug}"),
        })
    }
}

fn validate_note_relative_path(relative_path: &str) -> RepoResult<()> {
    let valid = !relative_path.trim().is_empty()
        && relative_path.len() <= 1024
        && !relative_path.starts_with('/')
        && !relative_path.contains("..")
        && relative_path.ends_with(".md");
    if valid {
        Ok(())
    } else {
        Err(RepositoryError::Constraint {
            entity: "notes",
            message: format!("unsafe note relative path: {relative_path}"),
        })
    }
}

fn stable_note_id_from_relative_path(relative_path: &str) -> String {
    format!("note_{:016x}", fnv1a64(relative_path.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct IntegrationStatusRecord {
    integration_key: String,
    display_name: String,
    status: IntegrationStatus,
    config_json: String,
    credential_ref: Option<String>,
    last_checked_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct IntegrationStatusInput<'a> {
    integration_key: &'a str,
    display_name: &'a str,
    status: IntegrationStatus,
    config_json: &'a str,
    credential_ref: Option<&'a str>,
    last_checked_at: Option<&'a str>,
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct EntityLinkCreateRequest<'a> {
    id: &'a str,
    source_type: &'a str,
    source_id: &'a str,
    target_type: &'a str,
    target_id: &'a str,
    relation_type: &'a str,
    created_by_actor_type: &'a str,
    metadata_json: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct EntityLinkListFilter<'a> {
    entity_type: &'a str,
    entity_id: &'a str,
    relation_type: Option<&'a str>,
    counterpart_type: Option<&'a str>,
}

const ALLOWED_ENTITY_LINK_TYPES: &[&str] = &[
    "task",
    "note",
    "product",
    "file",
    "repo",
    "run",
    "email",
    "event",
    "browser_capture",
];

#[cfg(test)]
const TAURI_BRIDGE_COMMAND_NAMES: &[&str] = &[
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
];

#[derive(Debug, Clone, Deserialize)]
struct TaskCommandCreateRequest {
    title: String,
    detail: Option<String>,
    priority: Option<String>,
    workspace_key: Option<String>,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaskCommandUpdateRequest {
    title: Option<String>,
    detail: Option<String>,
    priority: Option<String>,
    workspace_key: Option<String>,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaskCommandStatusRequest {
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentRunCommandStartRequest {
    task_id: String,
    profile_id: String,
    cwd: String,
    argv: Vec<String>,
    stdin: Option<String>,
    timeout_ms: Option<u64>,
    logs_dir: PathBuf,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentRunCommandStreamRequest {
    run_id: String,
    logs_dir: PathBuf,
    offset: Option<u64>,
    max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentRunCommandCancelRequest {
    reason: Option<String>,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManualReviewCommandCreateRequest {
    task_id: String,
    run_id: Option<String>,
    reviewer_profile_id: Option<String>,
    verdict: String,
    evidence_summary: String,
    required_fixes_json: String,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationCommandCreateRequest {
    notification_type: String,
    title: String,
    message: String,
    severity: String,
    action_route: Option<String>,
    task_id: Option<String>,
    run_id: Option<String>,
    review_record_id: Option<String>,
    metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InboxNotificationCommandListRequest {
    active_only: Option<bool>,
    limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationCommandStateRequest {
    state: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HistoryCommandCursorRequest {
    timestamp: String,
    event_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HistoryCommandListRequest {
    limit: Option<usize>,
    before: Option<HistoryCommandCursorRequest>,
}

#[derive(Debug, Clone, Deserialize)]
struct HistoryCommandEntityListRequest {
    entity_type: String,
    entity_id: String,
    include_related: Option<bool>,
    limit: Option<usize>,
    before: Option<HistoryCommandCursorRequest>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentRunCommandOutcome {
    session_id: String,
    run: AgentRunRecord,
    log_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct AgentRunCommandStreamChunk {
    run_id: String,
    log_reference_id: String,
    offset: u64,
    next_offset: u64,
    eof: bool,
    status: AgentRunStatus,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalPreferenceRequest {
    key: String,
    value_json: String,
    value_type: String,
    scope: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct IntegrationStatusRequest {
    integration_key: String,
    display_name: String,
    status: String,
    config_json: String,
    credential_ref: Option<String>,
    last_checked_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct EventTargetRequest {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct EventCreateRequest {
    action_type: String,
    outcome: String,
    actor_type: String,
    actor_id: Option<String>,
    workspace_key: Option<String>,
    summary: String,
    source: String,
    metadata_json: String,
    targets: Vec<EventTargetRequest>,
}

#[derive(Debug, Clone, Deserialize)]
struct EventListRequest {
    workspace_key: Option<String>,
    action_type: Option<String>,
    outcome: Option<String>,
    source: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct PolicyPreviewRequest {
    category: Option<String>,
    action_type: Option<String>,
    target: Option<String>,
    scope: Option<String>,
    consequence: Option<String>,
    bulk: Option<bool>,
    destructive: Option<bool>,
}

#[tauri::command]
fn create_task_command(request: TaskCommandCreateRequest) -> Result<TaskRecord, String> {
    let connection = open_ready_connection()?;
    create_task_command_with_connection(&connection, request)
}

#[tauri::command]
fn read_task_command(task_id: String) -> Result<TaskRecord, String> {
    let connection = open_ready_connection()?;
    read_task_command_with_connection(&connection, task_id)
}

#[tauri::command]
fn list_tasks_command() -> Result<Vec<TaskRecord>, String> {
    let connection = open_ready_connection()?;
    list_tasks_command_with_connection(&connection)
}

#[tauri::command]
fn update_task_command(
    task_id: String,
    request: TaskCommandUpdateRequest,
) -> Result<TaskRecord, String> {
    let connection = open_ready_connection()?;
    update_task_command_with_connection(&connection, task_id, request)
}

#[tauri::command]
fn update_task_status_command(
    task_id: String,
    request: TaskCommandStatusRequest,
) -> Result<TaskRecord, String> {
    let connection = open_ready_connection()?;
    update_task_status_command_with_connection(&connection, task_id, request)
}

#[tauri::command]
fn archive_task_command(task_id: String) -> Result<TaskRecord, String> {
    let connection = open_ready_connection()?;
    archive_task_command_with_connection(&connection, task_id)
}

#[tauri::command]
fn delete_task_command(task_id: String) -> Result<TaskRecord, String> {
    let connection = open_ready_connection()?;
    delete_task_command_with_connection(&connection, task_id)
}

#[tauri::command]
fn start_agent_run_command(
    request: AgentRunCommandStartRequest,
) -> Result<AgentRunCommandOutcome, String> {
    let connection = open_ready_connection()?;
    start_agent_run_command_with_connection(&connection, request)
}

#[tauri::command]
fn read_run_status_command(run_id: String) -> Result<AgentRunRecord, String> {
    let connection = open_ready_connection()?;
    read_run_status_command_with_connection(&connection, run_id)
}

#[tauri::command]
fn stream_run_output_command(
    request: AgentRunCommandStreamRequest,
) -> Result<AgentRunCommandStreamChunk, String> {
    let connection = open_ready_connection()?;
    stream_run_output_command_with_connection(&connection, request)
}

#[tauri::command]
fn cancel_run_command(
    run_id: String,
    request: AgentRunCommandCancelRequest,
) -> Result<AgentRunRecord, String> {
    let connection = open_ready_connection()?;
    cancel_run_command_with_connection(&connection, run_id, request)
}

#[tauri::command]
fn create_manual_review_command(
    request: ManualReviewCommandCreateRequest,
) -> Result<ReviewRecord, String> {
    let connection = open_ready_connection()?;
    create_manual_review_command_with_connection(&connection, request)
}

#[tauri::command]
fn read_review_record_command(review_record_id: String) -> Result<ReviewRecord, String> {
    let connection = open_ready_connection()?;
    read_review_record_command_with_connection(&connection, review_record_id)
}

#[tauri::command]
fn create_notification_command(
    request: NotificationCommandCreateRequest,
) -> Result<NotificationRecord, String> {
    let connection = open_ready_connection()?;
    create_notification_command_with_connection(&connection, request)
}

#[tauri::command]
fn read_notification_command(notification_id: String) -> Result<NotificationRecord, String> {
    let connection = open_ready_connection()?;
    read_notification_command_with_connection(&connection, notification_id)
}

#[tauri::command]
fn list_inbox_notifications_command(
    request: InboxNotificationCommandListRequest,
) -> Result<Vec<NotificationRecord>, String> {
    let connection = open_ready_connection()?;
    list_inbox_notifications_command_with_connection(&connection, request)
}

#[tauri::command]
fn update_notification_state_command(
    notification_id: String,
    request: NotificationCommandStateRequest,
) -> Result<NotificationRecord, String> {
    let connection = open_ready_connection()?;
    update_notification_state_command_with_connection(&connection, notification_id, request)
}

#[tauri::command]
fn list_task_history_command(
    task_id: String,
    request: HistoryCommandListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    let connection = open_ready_connection()?;
    list_task_history_command_with_connection(&connection, task_id, request)
}

#[tauri::command]
fn list_run_history_command(
    run_id: String,
    request: HistoryCommandListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    let connection = open_ready_connection()?;
    list_run_history_command_with_connection(&connection, run_id, request)
}

#[tauri::command]
fn list_notification_history_command(
    notification_id: String,
    request: HistoryCommandListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    let connection = open_ready_connection()?;
    list_notification_history_command_with_connection(&connection, notification_id, request)
}

#[tauri::command]
fn list_entity_history_command(
    request: HistoryCommandEntityListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    let connection = open_ready_connection()?;
    list_entity_history_command_with_connection(&connection, request)
}

#[tauri::command]
fn get_foundation_status() -> Result<FoundationStatus, String> {
    ensure_foundation().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_workspace_registry() -> Result<Vec<WorkspaceRecord>, String> {
    let connection = open_ready_connection()?;
    get_workspace_registry_with_connection(&connection)
}

#[tauri::command]
fn read_local_preference(key: String) -> Result<Option<LocalPreferenceRecord>, String> {
    let connection = open_ready_connection()?;
    read_local_preference_with_connection(&connection, key)
}

#[tauri::command]
fn list_local_preferences(scope: Option<String>) -> Result<Vec<LocalPreferenceRecord>, String> {
    let connection = open_ready_connection()?;
    list_local_preferences_with_connection(&connection, scope)
}

#[tauri::command]
fn upsert_local_preference(
    request: LocalPreferenceRequest,
) -> Result<LocalPreferenceRecord, String> {
    let connection = open_ready_connection()?;
    upsert_local_preference_with_connection(&connection, request)
}

#[tauri::command]
fn read_integration_status_command(
    integration_key: String,
) -> Result<Option<IntegrationStatusRecord>, String> {
    let connection = open_ready_connection()?;
    read_integration_status_with_connection(&connection, integration_key)
}

#[tauri::command]
fn list_integration_statuses_command() -> Result<Vec<IntegrationStatusRecord>, String> {
    let connection = open_ready_connection()?;
    list_integration_statuses_with_connection(&connection)
}

#[tauri::command]
fn upsert_integration_status_command(
    request: IntegrationStatusRequest,
) -> Result<IntegrationStatusRecord, String> {
    let connection = open_ready_connection()?;
    upsert_integration_status_with_connection(&connection, request)
}

#[tauri::command]
fn create_event(request: EventCreateRequest) -> Result<EventRecord, String> {
    let connection = open_ready_connection()?;
    create_event_with_connection(&connection, request)
}

#[tauri::command]
fn read_event(event_id: String) -> Result<EventRecord, String> {
    let connection = open_ready_connection()?;
    read_event_with_connection(&connection, event_id)
}

#[tauri::command]
fn list_events(request: EventListRequest) -> Result<Vec<EventRecord>, String> {
    let connection = open_ready_connection()?;
    list_events_with_connection(&connection, request)
}

#[tauri::command]
fn preview_action_policy(request: PolicyPreviewRequest) -> Result<ActionPolicyDecision, String> {
    if let Some(category) = request
        .category
        .as_deref()
        .filter(|category| !category.trim().is_empty())
    {
        return Ok(evaluate_action_policy(category));
    }

    let action_request = ActionRequest {
        action_type: parse_action_type(request.action_type.as_deref())?,
        target: request.target,
        scope: parse_action_scope(request.scope.as_deref())?,
        consequence: parse_action_consequence(request.consequence.as_deref())?,
        bulk: request.bulk.unwrap_or(false),
        destructive: request.destructive.unwrap_or(false),
    };
    Ok(evaluate_action_request(&action_request))
}

fn open_ready_connection() -> Result<Connection, String> {
    ensure_foundation().map_err(|error| error.to_string())?;
    let database_path =
        AppSupportPaths::for_home(&home_dir().map_err(|error| error.to_string())?).database_path;
    open_foundation_database(&database_path).map_err(|error| error.to_string())
}

fn task_command_priority(value: Option<String>) -> Result<Option<TaskPriority>, String> {
    value
        .as_deref()
        .map(TaskPriority::from_str)
        .transpose()
        .map_err(repository_error_message)
}

fn task_command_status(value: &str) -> Result<TaskStatus, String> {
    TaskStatus::from_str(value).map_err(repository_error_message)
}

fn create_task_command_with_connection(
    connection: &Connection,
    request: TaskCommandCreateRequest,
) -> Result<TaskRecord, String> {
    let priority = task_command_priority(request.priority)?;
    create_task_service(
        connection,
        TaskServiceCreateInput {
            title: request.title,
            detail: request.detail,
            priority,
            workspace_key: request.workspace_key,
            metadata_json: request.metadata_json.unwrap_or_else(|| "{}".to_string()),
        },
    )
    .map_err(repository_error_message)
}

fn read_task_command_with_connection(
    connection: &Connection,
    task_id: String,
) -> Result<TaskRecord, String> {
    read_task_service(connection, &task_id).map_err(repository_error_message)
}

fn list_tasks_command_with_connection(connection: &Connection) -> Result<Vec<TaskRecord>, String> {
    list_task_service(connection).map_err(repository_error_message)
}

fn update_task_command_with_connection(
    connection: &Connection,
    task_id: String,
    request: TaskCommandUpdateRequest,
) -> Result<TaskRecord, String> {
    let priority = task_command_priority(request.priority)?;
    update_task_service(
        connection,
        &task_id,
        TaskServiceUpdateInput {
            title: request.title,
            detail: request.detail,
            priority,
            workspace_key: request.workspace_key,
            metadata_json: request.metadata_json,
        },
    )
    .map_err(repository_error_message)
}

fn update_task_status_command_with_connection(
    connection: &Connection,
    task_id: String,
    request: TaskCommandStatusRequest,
) -> Result<TaskRecord, String> {
    update_task_service_status(connection, &task_id, task_command_status(&request.status)?)
        .map_err(repository_error_message)
}

fn archive_task_command_with_connection(
    connection: &Connection,
    task_id: String,
) -> Result<TaskRecord, String> {
    archive_task_service(connection, &task_id).map_err(repository_error_message)
}

fn delete_task_command_with_connection(
    connection: &Connection,
    task_id: String,
) -> Result<TaskRecord, String> {
    delete_task_service(connection, &task_id).map_err(repository_error_message)
}

fn start_agent_run_command_with_connection(
    connection: &Connection,
    request: AgentRunCommandStartRequest,
) -> Result<AgentRunCommandOutcome, String> {
    let metadata_json = request.metadata_json.unwrap_or_else(|| "{}".to_string());
    let command_request = AgentCommandRunRequest {
        task_id: request.task_id.clone(),
        profile_id: request.profile_id.clone(),
        cwd: request.cwd.clone(),
        argv: request.argv.clone(),
        stdin: request.stdin.clone(),
        timeout_ms: request.timeout_ms,
        logs_dir: request.logs_dir.clone(),
        metadata_json: metadata_json.clone(),
    };
    preflight_agent_command(connection, &command_request).map_err(repository_error_message)?;
    let database_path = database_path_for_connection(connection)?;

    let session = create_cli_session(
        connection,
        CliSessionCreateInput {
            task_id: request.task_id.clone(),
            profile_id: request.profile_id.clone(),
            mode: "clean_session".to_string(),
            cwd: request.cwd.clone(),
            status_summary: "Agent command queued".to_string(),
            metadata_json: metadata_json.clone(),
        },
    )
    .map_err(repository_error_message)?;
    let run = create_agent_run(
        connection,
        AgentRunCreateInput {
            task_id: request.task_id.clone(),
            profile_id: request.profile_id.clone(),
            session_id: session.id.clone(),
            cwd: request.cwd.clone(),
            metadata_json: metadata_json.clone(),
        },
    )
    .map_err(repository_error_message)?;
    let running = transition_agent_run_status(
        connection,
        &run.id,
        AgentRunStatus::Running,
        AgentRunTransitionInput {
            output_summary: Some("Process started".to_string()),
            error_summary: None,
            metadata_json: metadata_json.clone(),
        },
    )
    .map_err(repository_error_message)?;
    let log_path = request.logs_dir.join(format!("{}.log", running.id));

    let worker_ready = spawn_agent_run_worker(database_path, running.id.clone(), command_request);
    worker_ready
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "agent run worker did not register a cancellable process in time".to_string())?
        .map_err(|error| format!("agent run worker failed to start process: {error}"))?;

    Ok(AgentRunCommandOutcome {
        session_id: session.id,
        run: running,
        log_path: display_path(&log_path),
    })
}

fn read_run_status_command_with_connection(
    connection: &Connection,
    run_id: String,
) -> Result<AgentRunRecord, String> {
    read_agent_run_required(connection, &run_id).map_err(repository_error_message)
}

fn stream_run_output_command_with_connection(
    connection: &Connection,
    request: AgentRunCommandStreamRequest,
) -> Result<AgentRunCommandStreamChunk, String> {
    let run =
        read_agent_run_required(connection, &request.run_id).map_err(repository_error_message)?;
    let log_reference_id = run
        .log_reference_id
        .clone()
        .unwrap_or_else(|| format!("pending_{}", run.id));
    let path = if let Some(actual_log_reference_id) = run.log_reference_id.as_deref() {
        let relative_path = read_log_reference_relative_path(connection, actual_log_reference_id)
            .map_err(repository_error_message)?;
        request.logs_dir.join(relative_path)
    } else {
        request.logs_dir.join(format!("{}.log", run.id))
    };
    let offset = request.offset.unwrap_or(0);
    let max_bytes = request.max_bytes.unwrap_or(4096).clamp(1, 64 * 1024);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            return Err(format!(
                "failed to read run log stream {}: {error}",
                display_path(&path)
            ))
        }
    };
    let start = (offset as usize).min(bytes.len());
    let end = (start + max_bytes).min(bytes.len());
    let content = String::from_utf8_lossy(&bytes[start..end]).to_string();
    let next_offset = end as u64;
    let eof = run.status.is_terminal() && next_offset as usize >= bytes.len();

    Ok(AgentRunCommandStreamChunk {
        run_id: run.id,
        log_reference_id,
        offset,
        next_offset,
        eof,
        status: run.status,
        content,
    })
}

fn cancel_run_command_with_connection(
    connection: &Connection,
    run_id: String,
    request: AgentRunCommandCancelRequest,
) -> Result<AgentRunRecord, String> {
    if let Some(child) = active_run_children()
        .lock()
        .map_err(|_| "active run registry lock poisoned".to_string())?
        .get(&run_id)
        .cloned()
    {
        let _ = child
            .lock()
            .map_err(|_| "active child lock poisoned".to_string())?
            .kill();
    }
    transition_agent_run_status(
        connection,
        &run_id,
        AgentRunStatus::Cancelled,
        AgentRunTransitionInput {
            output_summary: Some(
                request
                    .reason
                    .unwrap_or_else(|| "Run cancelled from Tauri bridge".to_string()),
            ),
            error_summary: None,
            metadata_json: request.metadata_json.unwrap_or_else(|| "{}".to_string()),
        },
    )
    .map_err(repository_error_message)
}

fn create_manual_review_command_with_connection(
    connection: &Connection,
    request: ManualReviewCommandCreateRequest,
) -> Result<ReviewRecord, String> {
    let verdict = ReviewVerdict::from_str(&request.verdict).map_err(repository_error_message)?;
    create_manual_review_service(
        connection,
        ManualReviewServiceCreateInput {
            task_id: request.task_id,
            run_id: request.run_id,
            reviewer_profile_id: request.reviewer_profile_id,
            verdict,
            evidence_summary: request.evidence_summary,
            required_fixes_json: request.required_fixes_json,
            metadata_json: request.metadata_json.unwrap_or_else(|| "{}".to_string()),
        },
    )
    .map_err(repository_error_message)
}

fn read_review_record_command_with_connection(
    connection: &Connection,
    review_record_id: String,
) -> Result<ReviewRecord, String> {
    read_review_record(connection, &review_record_id)
        .map_err(repository_error_message)?
        .ok_or_else(|| {
            repository_error_message(RepositoryError::NotFound {
                entity: "review_records",
                key: review_record_id,
            })
        })
}

fn create_notification_command_with_connection(
    connection: &Connection,
    request: NotificationCommandCreateRequest,
) -> Result<NotificationRecord, String> {
    let notification_type =
        NotificationType::from_str(&request.notification_type).map_err(repository_error_message)?;
    let severity =
        NotificationSeverity::from_str(&request.severity).map_err(repository_error_message)?;
    create_notification_service(
        connection,
        NotificationServiceCreateInput {
            notification_type,
            title: request.title,
            message: request.message,
            severity,
            action_route: request.action_route,
            task_id: request.task_id,
            run_id: request.run_id,
            review_record_id: request.review_record_id,
            metadata_json: request.metadata_json.unwrap_or_else(|| "{}".to_string()),
        },
    )
    .map_err(repository_error_message)
}

fn read_notification_command_with_connection(
    connection: &Connection,
    notification_id: String,
) -> Result<NotificationRecord, String> {
    read_notification_service(connection, &notification_id).map_err(repository_error_message)
}

fn list_inbox_notifications_command_with_connection(
    connection: &Connection,
    request: InboxNotificationCommandListRequest,
) -> Result<Vec<NotificationRecord>, String> {
    list_inbox_notification_service(
        connection,
        request.active_only.unwrap_or(true),
        request.limit.unwrap_or(50),
    )
    .map_err(repository_error_message)
}

fn update_notification_state_command_with_connection(
    connection: &Connection,
    notification_id: String,
    request: NotificationCommandStateRequest,
) -> Result<NotificationRecord, String> {
    match request.state.trim() {
        "delivered" => deliver_notification_service(connection, &notification_id),
        "action_required" => require_notification_action_service(connection, &notification_id),
        "failed" => fail_notification_service(connection, &notification_id),
        "read" => read_mark_notification_service(connection, &notification_id),
        "dismissed" => dismiss_notification_service(connection, &notification_id),
        "resolved" => resolve_notification_service(connection, &notification_id),
        other => Err(RepositoryError::Constraint {
            entity: "notifications",
            message: format!("unsupported notification bridge state action: {other}"),
        }),
    }
    .map_err(repository_error_message)
}

fn list_task_history_command_with_connection(
    connection: &Connection,
    task_id: String,
    request: HistoryCommandListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    list_task_history(
        connection,
        &task_id,
        request.limit.unwrap_or(50),
        history_command_cursor(request.before),
    )
    .map_err(repository_error_message)
}

fn list_run_history_command_with_connection(
    connection: &Connection,
    run_id: String,
    request: HistoryCommandListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    list_run_history(
        connection,
        &run_id,
        request.limit.unwrap_or(50),
        history_command_cursor(request.before),
    )
    .map_err(repository_error_message)
}

fn list_notification_history_command_with_connection(
    connection: &Connection,
    notification_id: String,
    request: HistoryCommandListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    list_notification_history(
        connection,
        &notification_id,
        request.limit.unwrap_or(50),
        history_command_cursor(request.before),
    )
    .map_err(repository_error_message)
}

fn list_entity_history_command_with_connection(
    connection: &Connection,
    request: HistoryCommandEntityListRequest,
) -> Result<Vec<HistoryTimelineItem>, String> {
    list_entity_history(
        connection,
        HistoryQuery {
            primary: HistoryEntityRef {
                entity_type: request.entity_type,
                entity_id: request.entity_id,
            },
            include_related: request.include_related.unwrap_or(true),
            limit: request.limit.unwrap_or(50),
            before: history_command_cursor(request.before),
        },
    )
    .map_err(repository_error_message)
}

fn history_command_cursor(cursor: Option<HistoryCommandCursorRequest>) -> Option<HistoryCursor> {
    cursor.map(|cursor| HistoryCursor {
        timestamp: cursor.timestamp,
        event_id: cursor.event_id,
    })
}

fn active_run_children() -> &'static Mutex<HashMap<String, Arc<Mutex<std::process::Child>>>> {
    ACTIVE_RUN_CHILDREN.get_or_init(|| Mutex::new(HashMap::new()))
}

fn database_path_for_connection(connection: &Connection) -> Result<PathBuf, String> {
    let database_path: String = connection
        .query_row("pragma database_list", [], |row| row.get(2))
        .map_err(|error| format!("failed to inspect sqlite database path: {error}"))?;
    if database_path.trim().is_empty() {
        return Err("async run bridge requires a file-backed SQLite database".to_string());
    }
    Ok(PathBuf::from(database_path))
}

fn spawn_agent_run_worker(
    database_path: PathBuf,
    run_id: String,
    request: AgentCommandRunRequest,
) -> mpsc::Receiver<Result<(), String>> {
    let (ready_sender, ready_receiver) = mpsc::channel();
    std::thread::spawn(move || {
        if let Err(error) = run_agent_command_worker(&database_path, &run_id, request, ready_sender)
        {
            eprintln!("agent run worker failed for {run_id}: {error}");
        }
        if let Ok(mut active) = active_run_children().lock() {
            active.remove(&run_id);
        }
    });
    ready_receiver
}

fn run_agent_command_worker(
    database_path: &Path,
    run_id: &str,
    request: AgentCommandRunRequest,
    ready_sender: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let connection = open_foundation_database(database_path).map_err(|error| error.to_string())?;
    let profile = read_agent_profile(&connection, &request.profile_id)
        .map_err(repository_error_message)?
        .ok_or_else(|| format!("agent profile not found: {}", request.profile_id))?;
    let command = profile.command.clone().unwrap_or_default();
    ensure_directory(&request.logs_dir).map_err(|error| error.to_string())?;

    let mut child = match Command::new(&command)
        .args(&request.argv)
        .current_dir(&request.cwd)
        .stdin(if request.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let message = format!("process failed: {error}");
            let _ = ready_sender.send(Err(message.clone()));
            return Err(message);
        }
    };

    if let Some(stdin_body) = request.stdin.as_deref() {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_body.as_bytes())
                .map_err(|error| format!("failed to write stdin: {error}"))?;
        }
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child = Arc::new(Mutex::new(child));
    active_run_children()
        .lock()
        .map_err(|_| "active run registry lock poisoned".to_string())?
        .insert(run_id.to_string(), child.clone());
    let _ = ready_sender.send(Ok(()));

    let (sender, receiver) = mpsc::channel::<(&'static str, Vec<u8>)>();
    spawn_stream_reader("stdout", stdout, sender.clone());
    spawn_stream_reader("stderr", stderr, sender);

    let started = Instant::now();
    let timeout = request
        .timeout_ms
        .map(|value| Duration::from_millis(value.max(1)));
    let mut stdout_text = String::new();
    let mut stderr_text = String::new();
    let mut timed_out = false;
    let mut exit_code = None;
    let mut finished = false;

    while !finished {
        while let Ok((stream, chunk)) = receiver.try_recv() {
            let text = String::from_utf8_lossy(&chunk).to_string();
            if stream == "stdout" {
                stdout_text.push_str(&text);
            } else {
                stderr_text.push_str(&text);
            }
            append_run_log_chunk(&connection, &request.logs_dir, run_id, stream, &text)
                .map_err(repository_error_message)?;
        }

        if let Some(timeout) = timeout {
            if started.elapsed() >= timeout {
                timed_out = true;
                let _ = child
                    .lock()
                    .map_err(|_| "active child lock poisoned".to_string())?
                    .kill();
            }
        }

        if let Some(status) = child
            .lock()
            .map_err(|_| "active child lock poisoned".to_string())?
            .try_wait()
            .map_err(|error| format!("failed to poll process: {error}"))?
        {
            exit_code = status.code().map(i64::from);
            finished = true;
        } else {
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    while let Ok((stream, chunk)) = receiver.recv_timeout(Duration::from_millis(25)) {
        let text = String::from_utf8_lossy(&chunk).to_string();
        if stream == "stdout" {
            stdout_text.push_str(&text);
        } else {
            stderr_text.push_str(&text);
        }
        append_run_log_chunk(&connection, &request.logs_dir, run_id, stream, &text)
            .map_err(repository_error_message)?;
    }

    let status = if timed_out
        || read_agent_run_required(&connection, run_id)
            .map_err(repository_error_message)?
            .status
            == AgentRunStatus::Cancelled
    {
        AgentRunStatus::Cancelled
    } else if exit_code == Some(0) {
        AgentRunStatus::Completed
    } else {
        AgentRunStatus::Failed
    };
    if status == AgentRunStatus::Cancelled && stdout_text.is_empty() && stderr_text.is_empty() {
        append_run_log_chunk(
            &connection,
            &request.logs_dir,
            run_id,
            "system",
            "Run cancelled\n",
        )
        .map_err(repository_error_message)?;
    }
    let log_reference_id = read_log_reference_id(&connection, run_id, &format!("{run_id}.log"))
        .map_err(repository_error_message)?;
    let output_summary = summarize_output(&stdout_text, status);
    let error_summary = if stderr_text.trim().is_empty() {
        None
    } else {
        Some(summarize_text(&stderr_text))
    };
    let current = read_agent_run_required(&connection, run_id).map_err(repository_error_message)?;
    let completed =
        if status == AgentRunStatus::Cancelled && current.status == AgentRunStatus::Cancelled {
            finalize_cancelled_run_evidence(
                &connection,
                run_id,
                AgentRunCompletionInput {
                    status,
                    duration_ms: started.elapsed().as_millis().min(i64::MAX as u128).max(1) as i64,
                    exit_code,
                    log_reference_id: Some(log_reference_id),
                    output_summary,
                    error_summary,
                    review_state: ReviewState::NotRequired,
                    metadata_json: request.metadata_json,
                },
            )
        } else {
            complete_agent_run(
                &connection,
                run_id,
                AgentRunCompletionInput {
                    status,
                    duration_ms: started.elapsed().as_millis().min(i64::MAX as u128).max(1) as i64,
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
            )
        }
        .map_err(repository_error_message)?;
    create_run_result_notification(&connection, &completed).map_err(repository_error_message)?;
    Ok(())
}

fn finalize_cancelled_run_evidence(
    connection: &Connection,
    run_id: &str,
    input: AgentRunCompletionInput,
) -> RepoResult<AgentRunRecord> {
    if input.status != AgentRunStatus::Cancelled {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "cancelled evidence finalization requires cancelled status".to_string(),
        });
    }
    if let Some(log_reference_id) = input.log_reference_id.as_deref() {
        validate_log_reference_exists(connection, log_reference_id)?;
    }
    let output_summary = redact_secrets(&input.output_summary).text;
    let error_summary = input
        .error_summary
        .as_deref()
        .map(|value| redact_secrets(value).text);
    connection
        .execute(
            "
            update agent_runs
            set updated_at = current_timestamp,
                duration_ms = coalesce(duration_ms, ?2),
                exit_code = coalesce(exit_code, ?3),
                log_reference_id = coalesce(log_reference_id, ?4),
                output_summary = coalesce(output_summary, ?5),
                error_summary = coalesce(error_summary, ?6),
                review_state = ?7,
                metadata_json = ?8
            where id = ?1 and status = 'cancelled'
            ",
            params![
                run_id,
                input.duration_ms,
                input.exit_code,
                input.log_reference_id,
                output_summary,
                error_summary,
                input.review_state.as_str(),
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("agent_runs", error))?;
    read_agent_run_required(connection, run_id)
}

fn spawn_stream_reader(
    stream: &'static str,
    reader: Option<impl Read + Send + 'static>,
    sender: mpsc::Sender<(&'static str, Vec<u8>)>,
) {
    if let Some(mut reader) = reader {
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => {
                        if sender.send((stream, buffer[..count].to_vec())).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

fn append_run_log_chunk(
    connection: &Connection,
    logs_dir: &Path,
    run_id: &str,
    stream: &str,
    text: &str,
) -> RepoResult<()> {
    let log_body = format!("{stream}:\n{text}");
    write_safe_log(connection, logs_dir, run_id, &log_body).map_err(|error| {
        RepositoryError::Constraint {
            entity: "log_references",
            message: format!("failed to persist redacted run log chunk: {error}"),
        }
    })?;
    Ok(())
}

fn read_log_reference_relative_path(
    connection: &Connection,
    log_reference_id: &str,
) -> RepoResult<String> {
    connection
        .query_row(
            "select relative_path from log_references where id = ?1",
            params![log_reference_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| map_repository_error("log_references", error))?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "log_references",
            key: log_reference_id.to_string(),
        })
}

fn get_workspace_registry_with_connection(
    connection: &Connection,
) -> Result<Vec<WorkspaceRecord>, String> {
    list_workspaces(connection).map_err(|error| error.to_string())
}

fn read_local_preference_with_connection(
    connection: &Connection,
    key: String,
) -> Result<Option<LocalPreferenceRecord>, String> {
    read_local_app_preference(connection, &key).map_err(repository_error_message)
}

fn list_local_preferences_with_connection(
    connection: &Connection,
    scope: Option<String>,
) -> Result<Vec<LocalPreferenceRecord>, String> {
    match scope.as_deref() {
        Some(value) if !value.trim().is_empty() => list_local_app_preferences_by_scope(
            connection,
            SettingScope::from_str(value).map_err(repository_error_message)?,
        )
        .map_err(repository_error_message),
        _ => list_local_app_preferences(connection).map_err(repository_error_message),
    }
}

fn upsert_local_preference_with_connection(
    connection: &Connection,
    request: LocalPreferenceRequest,
) -> Result<LocalPreferenceRecord, String> {
    upsert_local_app_preference(
        connection,
        LocalPreferenceInput {
            key: request.key.as_str(),
            value_json: request.value_json.as_str(),
            value_type: request.value_type.as_str(),
            scope: SettingScope::from_str(request.scope.as_str())
                .map_err(repository_error_message)?,
            description: request.description.as_str(),
        },
    )
    .map_err(repository_error_message)
}

fn read_integration_status_with_connection(
    connection: &Connection,
    integration_key: String,
) -> Result<Option<IntegrationStatusRecord>, String> {
    read_integration_status(connection, &integration_key).map_err(repository_error_message)
}

fn list_integration_statuses_with_connection(
    connection: &Connection,
) -> Result<Vec<IntegrationStatusRecord>, String> {
    list_integration_statuses(connection).map_err(repository_error_message)
}

fn upsert_integration_status_with_connection(
    connection: &Connection,
    request: IntegrationStatusRequest,
) -> Result<IntegrationStatusRecord, String> {
    upsert_integration_status(
        connection,
        IntegrationStatusInput {
            integration_key: request.integration_key.as_str(),
            display_name: request.display_name.as_str(),
            status: IntegrationStatus::from_str(request.status.as_str())
                .map_err(repository_error_message)?,
            config_json: request.config_json.as_str(),
            credential_ref: request.credential_ref.as_deref(),
            last_checked_at: request.last_checked_at.as_deref(),
        },
    )
    .map_err(repository_error_message)
}

fn create_event_with_connection(
    connection: &Connection,
    request: EventCreateRequest,
) -> Result<EventRecord, String> {
    validate_event_create_request(&request)?;
    validate_json_field("metadata_json", request.metadata_json.as_str())
        .map_err(repository_error_message)?;
    let targets = request
        .targets
        .iter()
        .map(|target| EventTargetInput {
            entity_type: target.entity_type.as_str(),
            entity_id: target.entity_id.as_str(),
            relation_type: target.relation_type.as_str(),
        })
        .collect();
    create_event_record(
        connection,
        EventCreateInput {
            action_type: request.action_type.as_str(),
            outcome: request.outcome.as_str(),
            actor_type: request.actor_type.as_str(),
            actor_id: request.actor_id.as_deref(),
            workspace_key: request.workspace_key.as_deref(),
            summary: request.summary.as_str(),
            source: request.source.as_str(),
            metadata_json: request.metadata_json.as_str(),
            targets,
        },
    )
    .map_err(repository_error_message)
}

fn validate_event_create_request(request: &EventCreateRequest) -> Result<(), String> {
    reject_over_limit(
        "action_type",
        request.action_type.len(),
        EVENT_CREATE_MAX_SMALL_FIELD_BYTES,
    )?;
    reject_over_limit(
        "outcome",
        request.outcome.len(),
        EVENT_CREATE_MAX_SMALL_FIELD_BYTES,
    )?;
    reject_over_limit(
        "actor_type",
        request.actor_type.len(),
        EVENT_CREATE_MAX_SMALL_FIELD_BYTES,
    )?;
    if let Some(actor_id) = request.actor_id.as_deref() {
        reject_over_limit("actor_id", actor_id.len(), EVENT_CREATE_MAX_SOURCE_BYTES)?;
    }
    if let Some(workspace_key) = request.workspace_key.as_deref() {
        reject_over_limit(
            "workspace_key",
            workspace_key.len(),
            EVENT_CREATE_MAX_SMALL_FIELD_BYTES,
        )?;
    }
    reject_over_limit(
        "summary",
        request.summary.len(),
        EVENT_CREATE_MAX_SUMMARY_BYTES,
    )?;
    reject_over_limit(
        "source",
        request.source.len(),
        EVENT_CREATE_MAX_SOURCE_BYTES,
    )?;
    reject_over_limit(
        "metadata_json",
        request.metadata_json.len(),
        EVENT_CREATE_MAX_METADATA_JSON_BYTES,
    )?;
    reject_over_limit("targets", request.targets.len(), EVENT_CREATE_MAX_TARGETS)?;

    for target in &request.targets {
        reject_over_limit(
            "target.entity_type",
            target.entity_type.len(),
            EVENT_CREATE_MAX_SMALL_FIELD_BYTES,
        )?;
        reject_over_limit(
            "target.entity_id",
            target.entity_id.len(),
            EVENT_CREATE_MAX_SOURCE_BYTES,
        )?;
        reject_over_limit(
            "target.relation_type",
            target.relation_type.len(),
            EVENT_CREATE_MAX_SMALL_FIELD_BYTES,
        )?;
    }

    Ok(())
}

fn reject_over_limit(field: &'static str, actual: usize, limit: usize) -> Result<(), String> {
    if actual > limit {
        return Err(format!("event request exceeds bridge limit: {field}"));
    }
    Ok(())
}

fn read_event_with_connection(
    connection: &Connection,
    event_id: String,
) -> Result<EventRecord, String> {
    read_event_record(connection, &event_id).map_err(repository_error_message)
}

fn list_events_with_connection(
    connection: &Connection,
    request: EventListRequest,
) -> Result<Vec<EventRecord>, String> {
    list_event_records(
        connection,
        EventListFilter {
            workspace_key: request.workspace_key.as_deref(),
            action_type: request.action_type.as_deref(),
            outcome: request.outcome.as_deref(),
            source: request.source.as_deref(),
            limit: request.limit.unwrap_or(50),
        },
    )
    .map_err(repository_error_message)
}

fn parse_action_type(value: Option<&str>) -> Result<ActionType, String> {
    match value
        .unwrap_or("unknown")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "read" => Ok(ActionType::Read),
        "create" => Ok(ActionType::Create),
        "update" => Ok(ActionType::Update),
        "delete" => Ok(ActionType::Delete),
        "send" => Ok(ActionType::Send),
        "publish" => Ok(ActionType::Publish),
        "deploy" => Ok(ActionType::Deploy),
        "file" => Ok(ActionType::File),
        "process" | "run" | "execute" => Ok(ActionType::Process),
        "" | "unknown" => Ok(ActionType::Unknown),
        other => Err(format!("unsupported action_type: {other}")),
    }
}

fn parse_action_scope(value: Option<&str>) -> Result<Option<ActionScope>, String> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(value) => match value.to_ascii_lowercase().as_str() {
            "local_private" => Ok(Some(ActionScope::LocalPrivate)),
            "local_visible" => Ok(Some(ActionScope::LocalVisible)),
            "code_repository" => Ok(Some(ActionScope::CodeRepository)),
            "integration" => Ok(Some(ActionScope::Integration)),
            "external" => Ok(Some(ActionScope::External)),
            "unknown" => Ok(Some(ActionScope::Unknown)),
            other => Err(format!("unsupported scope: {other}")),
        },
    }
}

fn parse_action_consequence(value: Option<&str>) -> Result<Option<ActionConsequence>, String> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(value) => match value.to_ascii_lowercase().as_str() {
            "harmless_local" => Ok(Some(ActionConsequence::HarmlessLocal)),
            "local_write" => Ok(Some(ActionConsequence::LocalWrite)),
            "external_write" => Ok(Some(ActionConsequence::ExternalWrite)),
            "public_release" => Ok(Some(ActionConsequence::PublicRelease)),
            "destructive" => Ok(Some(ActionConsequence::Destructive)),
            "automation_execution" => Ok(Some(ActionConsequence::AutomationExecution)),
            "credential_or_integration_change" => {
                Ok(Some(ActionConsequence::CredentialOrIntegrationChange))
            }
            "unknown" => Ok(Some(ActionConsequence::Unknown)),
            other => Err(format!("unsupported consequence: {other}")),
        },
    }
}

fn repository_error_message(error: RepositoryError) -> String {
    format!("{error:?}")
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
    seed_default_integration_statuses(&connection).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to seed integration statuses: {error:?}"),
        )
    })?;
    write_foundation_event(&connection)?;
    let safe_log_probe = write_safe_log(
        &connection,
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

fn canonical_workspace_registry() -> &'static [WorkspaceDefinition] {
    WORKSPACE_REGISTRY
}

#[cfg(test)]
fn canonical_workspace_ids() -> Vec<String> {
    canonical_workspace_registry()
        .iter()
        .map(|workspace| workspace.key.to_string())
        .collect()
}

fn workspace_definition_by_key(key: &str) -> Option<&'static WorkspaceDefinition> {
    canonical_workspace_registry()
        .iter()
        .find(|workspace| workspace.key == key)
}

fn workspace_record_from_row(
    id: String,
    label: String,
    description: String,
    position: i64,
) -> WorkspaceRecord {
    let definition = workspace_definition_by_key(&id);
    WorkspaceRecord {
        id,
        label,
        description,
        position,
        availability: definition
            .map(|workspace| workspace.availability)
            .unwrap_or(WorkspaceAvailability::Blocked),
        integrations: definition
            .map(|workspace| workspace.integrations.to_vec())
            .unwrap_or_else(|| {
                vec![WorkspaceIntegration {
                    key: "registry",
                    label: "Workspace Registry",
                    state: WorkspaceIntegrationState::Blocked,
                    note: "Workspace is missing from the canonical backend registry.",
                }]
            }),
        status_note: definition
            .map(|workspace| workspace.status_note.to_string())
            .unwrap_or_else(|| {
                "Workspace is missing from the canonical backend registry.".to_string()
            }),
    }
}

fn seed_workspaces(connection: &Connection) -> rusqlite::Result<()> {
    for workspace in canonical_workspace_registry() {
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
            params![
                workspace.key,
                workspace.label,
                workspace.description,
                workspace.position
            ],
        )?;
    }
    Ok(())
}

fn list_workspaces(connection: &Connection) -> rusqlite::Result<Vec<WorkspaceRecord>> {
    let mut statement = connection.prepare(
        "select id, label, description, position from workspaces where enabled = 1 order by position asc",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(workspace_record_from_row(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
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
fn create_task_record(connection: &Connection, input: TaskCreateInput) -> RepoResult<TaskRecord> {
    let title = normalize_task_title(&input.title)?;
    let detail = normalize_task_detail(input.detail.as_deref())?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;

    let status = input.status.unwrap_or(TaskStatus::Inbox);
    let priority = input.priority.unwrap_or(TaskPriority::Normal);
    let workspace_key = normalize_task_workspace_key(input.workspace_key.as_deref())?;
    let task_id = next_task_id();

    connection
        .execute_batch("savepoint create_task_record")
        .map_err(|error| map_repository_error("tasks", error))?;

    let create_result = (|| -> RepoResult<TaskRecord> {
        connection
            .execute(
                "
                insert into tasks (id, title, detail, status, priority, workspace_key, metadata_json)
                values (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                params![
                    task_id,
                    title,
                    detail,
                    status.as_str(),
                    priority.as_str(),
                    workspace_key,
                    input.metadata_json
                ],
            )
            .map_err(|error| map_repository_error("tasks", error))?;

        let task = read_task_record(connection, &task_id)?;
        create_event_record(
            connection,
            EventCreateInput {
                action_type: "task.created",
                outcome: "succeeded",
                actor_type: "system",
                actor_id: None,
                workspace_key: Some(&task.workspace_key),
                summary: &format!("Created task: {}", task.title),
                source: "task_repository",
                metadata_json: &format!(
                    "{{\"task_id\":{},\"title\":{},\"status\":{},\"priority\":{},\"metadata\":{}}}",
                    serde_json::to_string(&task.id).unwrap_or_else(|_| "null".to_string()),
                    serde_json::to_string(&task.title).unwrap_or_else(|_| "null".to_string()),
                    serde_json::to_string(status.as_str()).unwrap_or_else(|_| "null".to_string()),
                    serde_json::to_string(priority.as_str()).unwrap_or_else(|_| "null".to_string()),
                    task.metadata_json
                ),
                targets: vec![EventTargetInput {
                    entity_type: "task",
                    entity_id: &task.id,
                    relation_type: "primary",
                }],
            },
        )?;
        Ok(task)
    })();

    match create_result {
        Ok(task) => {
            connection
                .execute_batch("release savepoint create_task_record")
                .map_err(|error| map_repository_error("tasks", error))?;
            Ok(task)
        }
        Err(error) => {
            let _ = connection.execute_batch(
                "rollback to savepoint create_task_record; release savepoint create_task_record",
            );
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn read_task_record(connection: &Connection, task_id: &str) -> RepoResult<TaskRecord> {
    connection
        .query_row(
            "
            select id, title, detail, status, priority, workspace_key, created_at, updated_at, archived_at, deleted_at, metadata_json
            from tasks
            where id = ?1
            ",
            params![task_id],
            task_record_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("tasks", error))?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "tasks",
            key: task_id.to_string(),
        })
}

#[allow(dead_code)]
fn list_active_tasks(connection: &Connection) -> RepoResult<Vec<TaskRecord>> {
    let mut statement = connection
        .prepare(
            "
            select id, title, detail, status, priority, workspace_key, created_at, updated_at, archived_at, deleted_at, metadata_json
            from tasks
            where archived_at is null
                and deleted_at is null
                and status not in ('archived', 'deleted')
            order by
                case priority when 'urgent' then 0 when 'high' then 1 when 'normal' then 2 when 'low' then 3 else 4 end,
                updated_at desc,
                created_at desc,
                id asc
            ",
        )
        .map_err(|error| map_repository_error("tasks", error))?;
    let rows = statement
        .query_map([], task_record_from_row)
        .map_err(|error| map_repository_error("tasks", error))?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|error| map_repository_error("tasks", error))?);
    }
    Ok(records)
}

#[allow(dead_code)]
fn update_task_status(
    connection: &Connection,
    task_id: &str,
    status: TaskStatus,
) -> RepoResult<TaskRecord> {
    let before = read_task_record(connection, task_id)?;
    if before.status == status {
        return Ok(before);
    }

    connection
        .execute_batch("savepoint update_task_status")
        .map_err(|error| map_repository_error("tasks", error))?;

    let update_result = (|| -> RepoResult<TaskRecord> {
        connection
            .execute(
                "
                update tasks
                set status = ?2,
                    updated_at = current_timestamp,
                    archived_at = case when ?2 = 'archived' then coalesce(archived_at, current_timestamp) else archived_at end,
                    deleted_at = case when ?2 = 'deleted' then coalesce(deleted_at, current_timestamp) else deleted_at end
                where id = ?1
                ",
                params![task_id, status.as_str()],
            )
            .map_err(|error| map_repository_error("tasks", error))?;

        let task = read_task_record(connection, task_id)?;
        create_event_record(
            connection,
            EventCreateInput {
                action_type: "task.status_changed",
                outcome: "succeeded",
                actor_type: "system",
                actor_id: None,
                workspace_key: Some(&task.workspace_key),
                summary: &format!(
                    "Task status changed from {} to {}: {}",
                    before.status.as_str(),
                    status.as_str(),
                    task.title
                ),
                source: "task_repository",
                metadata_json: &format!(
                    "{{\"task_id\":{},\"from_status\":{},\"to_status\":{},\"title\":{}}}",
                    serde_json::to_string(&task.id).unwrap_or_else(|_| "null".to_string()),
                    serde_json::to_string(before.status.as_str())
                        .unwrap_or_else(|_| "null".to_string()),
                    serde_json::to_string(status.as_str()).unwrap_or_else(|_| "null".to_string()),
                    serde_json::to_string(&task.title).unwrap_or_else(|_| "null".to_string())
                ),
                targets: vec![EventTargetInput {
                    entity_type: "task",
                    entity_id: &task.id,
                    relation_type: "primary",
                }],
            },
        )?;
        Ok(task)
    })();

    match update_result {
        Ok(task) => {
            connection
                .execute_batch("release savepoint update_task_status")
                .map_err(|error| map_repository_error("tasks", error))?;
            Ok(task)
        }
        Err(error) => {
            let _ = connection.execute_batch(
                "rollback to savepoint update_task_status; release savepoint update_task_status",
            );
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn archive_task(connection: &Connection, task_id: &str) -> RepoResult<TaskRecord> {
    let task = update_task_status(connection, task_id, TaskStatus::Archived)?;
    create_task_lifecycle_event(connection, &task, "task.archived", "Archived task")?;
    Ok(task)
}

#[allow(dead_code)]
fn soft_delete_task(connection: &Connection, task_id: &str) -> RepoResult<TaskRecord> {
    let task = update_task_status(connection, task_id, TaskStatus::Deleted)?;
    create_task_lifecycle_event(connection, &task, "task.deleted", "Deleted task")?;
    Ok(task)
}

#[allow(dead_code)]
fn create_task_lifecycle_event(
    connection: &Connection,
    task: &TaskRecord,
    action_type: &str,
    summary_prefix: &str,
) -> RepoResult<()> {
    create_event_record(
        connection,
        EventCreateInput {
            action_type,
            outcome: "succeeded",
            actor_type: "system",
            actor_id: None,
            workspace_key: Some(&task.workspace_key),
            summary: &format!("{summary_prefix}: {}", task.title),
            source: "task_repository",
            metadata_json: &format!(
                "{{\"task_id\":{},\"status\":{},\"title\":{}}}",
                serde_json::to_string(&task.id).unwrap_or_else(|_| "null".to_string()),
                serde_json::to_string(task.status.as_str()).unwrap_or_else(|_| "null".to_string()),
                serde_json::to_string(&task.title).unwrap_or_else(|_| "null".to_string())
            ),
            targets: vec![EventTargetInput {
                entity_type: "task",
                entity_id: &task.id,
                relation_type: "primary",
            }],
        },
    )?;
    Ok(())
}

#[allow(dead_code)]
fn task_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRecord> {
    let status_text: String = row.get(3)?;
    let priority_text: String = row.get(4)?;
    let status = TaskStatus::from_str(&status_text).map_err(repository_error_to_rusqlite)?;
    let priority = TaskPriority::from_str(&priority_text).map_err(repository_error_to_rusqlite)?;

    Ok(TaskRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        detail: row.get(2)?,
        status,
        priority,
        workspace_key: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        archived_at: row.get(8)?,
        deleted_at: row.get(9)?,
        metadata_json: row.get(10)?,
    })
}

#[allow(dead_code)]
fn normalize_task_title(title: &str) -> RepoResult<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(RepositoryError::Constraint {
            entity: "tasks",
            message: "task title cannot be empty".to_string(),
        });
    }
    if trimmed.len() > TASK_TITLE_MAX_BYTES {
        return Err(RepositoryError::Constraint {
            entity: "tasks",
            message: format!("task title exceeds {TASK_TITLE_MAX_BYTES} bytes"),
        });
    }
    Ok(trimmed.to_string())
}

#[allow(dead_code)]
fn normalize_task_detail(detail: Option<&str>) -> RepoResult<Option<String>> {
    match detail {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(RepositoryError::Constraint {
                    entity: "tasks",
                    message: "task detail cannot be empty when provided".to_string(),
                });
            }
            if trimmed.len() > TASK_DETAIL_MAX_BYTES {
                return Err(RepositoryError::Constraint {
                    entity: "tasks",
                    message: format!("task detail exceeds {TASK_DETAIL_MAX_BYTES} bytes"),
                });
            }
            Ok(Some(trimmed.to_string()))
        }
        None => Ok(None),
    }
}

#[allow(dead_code)]
fn normalize_task_workspace_key(workspace_key: Option<&str>) -> RepoResult<String> {
    match workspace_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) if value.len() <= EVENT_CREATE_MAX_SMALL_FIELD_BYTES => Ok(value.to_string()),
        Some(_) => Err(RepositoryError::Constraint {
            entity: "tasks",
            message: "task workspace_key is too large".to_string(),
        }),
        None => Ok("tasks".to_string()),
    }
}

#[allow(dead_code)]
fn next_task_id() -> String {
    let sequence = TASK_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "task_{}_{:010}_{:020}",
        now_millis(),
        process::id(),
        sequence
    )
}

#[allow(dead_code)]
fn upsert_agent_profile(
    connection: &Connection,
    input: AgentProfileInput,
) -> RepoResult<AgentProfileRecord> {
    let id = normalize_small_text("agent_profiles", "id", &input.id)?;
    let label = normalize_small_text("agent_profiles", "label", &input.label)?;
    let command = normalize_optional_command(input.command.as_deref())?;
    validate_no_secret_command(command.as_deref())?;
    if input.configured && command.is_none() {
        return Err(RepositoryError::Constraint {
            entity: "agent_profiles",
            message: "configured agent profile requires a command".to_string(),
        });
    }
    validate_no_secret_json("config_json", &input.config_json)?;
    validate_no_secret_json("capabilities_json", &input.capabilities_json)?;
    validate_credential_ref(input.credential_ref.as_deref())?;
    validate_no_secret_json("env_refs_json", &input.env_refs_json)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;

    connection
        .execute(
            "
            insert into agent_profiles (
                id, label, configured, command, config_json, capabilities_json,
                credential_ref, env_refs_json, metadata_json, updated_at
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, current_timestamp)
            on conflict(id) do update set
                label = excluded.label,
                configured = excluded.configured,
                command = excluded.command,
                config_json = excluded.config_json,
                capabilities_json = excluded.capabilities_json,
                credential_ref = excluded.credential_ref,
                env_refs_json = excluded.env_refs_json,
                metadata_json = excluded.metadata_json,
                updated_at = current_timestamp
            ",
            params![
                id,
                label,
                if input.configured { 1_i64 } else { 0_i64 },
                command,
                input.config_json,
                input.capabilities_json,
                input.credential_ref,
                input.env_refs_json,
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("agent_profiles", error))?;
    read_agent_profile(connection, &input.id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "agent_profiles",
        key: input.id,
    })
}

#[allow(dead_code)]
fn read_agent_profile(connection: &Connection, id: &str) -> RepoResult<Option<AgentProfileRecord>> {
    connection
        .query_row(
            "
            select id, label, configured, command, config_json, capabilities_json,
                   credential_ref, env_refs_json, metadata_json, created_at, updated_at
            from agent_profiles where id = ?1
            ",
            params![id],
            agent_profile_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("agent_profiles", error))
}

#[allow(dead_code)]
fn agent_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentProfileRecord> {
    let configured: i64 = row.get(2)?;
    Ok(AgentProfileRecord {
        id: row.get(0)?,
        label: row.get(1)?,
        configured: configured != 0,
        command: row.get(3)?,
        config_json: row.get(4)?,
        capabilities_json: row.get(5)?,
        credential_ref: row.get(6)?,
        env_refs_json: row.get(7)?,
        metadata_json: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

#[allow(dead_code)]
fn create_cli_session(
    connection: &Connection,
    input: CliSessionCreateInput,
) -> RepoResult<CliSessionRecord> {
    validate_runnable_task(connection, &input.task_id, "cli_sessions")?;
    let profile = read_agent_profile(connection, &input.profile_id)?.ok_or_else(|| {
        RepositoryError::NotFound {
            entity: "agent_profiles",
            key: input.profile_id.clone(),
        }
    })?;
    validate_configured_profile(&profile)?;
    let mode = normalize_small_text("cli_sessions", "mode", &input.mode)?;
    let cwd = normalize_cwd("cli_sessions", &input.cwd)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    let session_id = next_cli_session_id();

    connection
        .execute(
            "
            insert into cli_sessions (id, task_id, profile_id, mode, cwd, status, status_summary, metadata_json)
            values (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7)
            ",
            params![
                session_id,
                input.task_id,
                input.profile_id,
                mode,
                cwd,
                redact_secrets(&input.status_summary).text,
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("cli_sessions", error))?;

    insert_or_get_entity_link(
        connection,
        EntityLinkInput {
            id: &format!("link_task_session_{session_id}"),
            source_type: "task",
            source_id: &input.task_id,
            target_type: "cli_session",
            target_id: &session_id,
            relation_type: "owns",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )?;
    read_cli_session(connection, &session_id)?.ok_or(RepositoryError::NotFound {
        entity: "cli_sessions",
        key: session_id,
    })
}

#[allow(dead_code)]
fn read_cli_session(connection: &Connection, id: &str) -> RepoResult<Option<CliSessionRecord>> {
    connection
        .query_row(
            "
            select id, task_id, profile_id, mode, cwd, status, status_summary,
                   metadata_json, created_at, updated_at, completed_at
            from cli_sessions where id = ?1
            ",
            params![id],
            |row| {
                Ok(CliSessionRecord {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    profile_id: row.get(2)?,
                    mode: row.get(3)?,
                    cwd: row.get(4)?,
                    status: row.get(5)?,
                    status_summary: row.get(6)?,
                    metadata_json: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    completed_at: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(|error| map_repository_error("cli_sessions", error))
}

#[allow(dead_code)]
fn create_agent_run(
    connection: &Connection,
    input: AgentRunCreateInput,
) -> RepoResult<AgentRunRecord> {
    validate_runnable_task(connection, &input.task_id, "agent_runs")?;
    let profile = read_agent_profile(connection, &input.profile_id)?.ok_or_else(|| {
        RepositoryError::NotFound {
            entity: "agent_profiles",
            key: input.profile_id.clone(),
        }
    })?;
    validate_configured_profile(&profile)?;
    let session = read_cli_session(connection, &input.session_id)?.ok_or_else(|| {
        RepositoryError::NotFound {
            entity: "cli_sessions",
            key: input.session_id.clone(),
        }
    })?;
    if session.task_id != input.task_id || session.profile_id != input.profile_id {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "run session must belong to the same task and profile".to_string(),
        });
    }
    let cwd = normalize_cwd("agent_runs", &input.cwd)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    let run_id = next_agent_run_id();
    let command_snapshot = profile.command.clone().unwrap_or_default();
    let profile_snapshot_json = serde_json::json!({
        "profile_id": profile.id,
        "label": profile.label,
        "configured": profile.configured,
        "command": command_snapshot,
        "capabilities": serde_json::from_str::<Value>(&profile.capabilities_json).unwrap_or(Value::Null),
        "credential_ref_present": profile.credential_ref.is_some(),
        "env_refs": serde_json::from_str::<Value>(&profile.env_refs_json).unwrap_or(Value::Null),
    })
    .to_string();

    connection
        .execute(
            "
            insert into agent_runs (
                id, task_id, profile_id, session_id, cwd, command_snapshot,
                profile_snapshot_json, status, metadata_json
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8)
            ",
            params![
                run_id,
                input.task_id,
                input.profile_id,
                input.session_id,
                cwd,
                command_snapshot,
                profile_snapshot_json,
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("agent_runs", error))?;
    insert_or_get_entity_link(
        connection,
        EntityLinkInput {
            id: &format!("link_task_run_{run_id}"),
            source_type: "task",
            source_id: &input.task_id,
            target_type: "agent_run",
            target_id: &run_id,
            relation_type: "owns",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )?;
    let run = read_agent_run(connection, &run_id)?.ok_or(RepositoryError::NotFound {
        entity: "agent_runs",
        key: run_id,
    })?;
    create_agent_run_event(connection, &run, AgentRunStatus::Queued, None, None, "{}")?;
    Ok(run)
}

#[allow(dead_code)]
fn transition_agent_run_status(
    connection: &Connection,
    run_id: &str,
    status: AgentRunStatus,
    input: AgentRunTransitionInput,
) -> RepoResult<AgentRunRecord> {
    let before = read_agent_run_required(connection, run_id)?;
    if before.status.is_terminal() {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "terminal agent run cannot mutate into new work; create a new run attempt"
                .to_string(),
        });
    }
    if status == AgentRunStatus::Completed {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "completed transition requires completion evidence".to_string(),
        });
    }
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    connection
        .execute(
            "
            update agent_runs
            set status = ?2,
                updated_at = current_timestamp,
                started_at = case when ?2 in ('starting', 'running') then coalesce(started_at, current_timestamp) else started_at end,
                completed_at = case when ?2 in ('failed', 'cancelled', 'blocked') then coalesce(completed_at, current_timestamp) else completed_at end,
                output_summary = coalesce(?3, output_summary),
                error_summary = coalesce(?4, error_summary),
                metadata_json = ?5
            where id = ?1
            ",
            params![
                run_id,
                status.as_str(),
                input.output_summary.as_deref().map(|value| redact_secrets(value).text),
                input.error_summary.as_deref().map(|value| redact_secrets(value).text),
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("agent_runs", error))?;
    let run = read_agent_run_required(connection, run_id)?;
    create_agent_run_event(
        connection,
        &run,
        status,
        input.output_summary.as_deref(),
        input.error_summary.as_deref(),
        &input.metadata_json,
    )?;
    Ok(run)
}

#[allow(dead_code)]
fn complete_agent_run(
    connection: &Connection,
    run_id: &str,
    input: AgentRunCompletionInput,
) -> RepoResult<AgentRunRecord> {
    let before = read_agent_run_required(connection, run_id)?;
    if before.status.is_terminal() {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "terminal agent run cannot mutate into new work; create a new run attempt"
                .to_string(),
        });
    }
    if !input.status.is_terminal() {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "completion requires a terminal status".to_string(),
        });
    }
    if input.status == AgentRunStatus::Completed
        && (input.exit_code.is_none() || input.log_reference_id.is_none())
    {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "completed run requires exit code and log reference".to_string(),
        });
    }
    if input.duration_ms < 0 || input.output_summary.trim().is_empty() {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: "completion requires duration and summary".to_string(),
        });
    }
    if let Some(log_reference_id) = input.log_reference_id.as_deref() {
        validate_log_reference_exists(connection, log_reference_id)?;
    }
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    let output_summary = redact_secrets(&input.output_summary).text;
    let error_summary = input
        .error_summary
        .as_deref()
        .map(|value| redact_secrets(value).text);
    connection
        .execute(
            "
            update agent_runs
            set status = ?2,
                updated_at = current_timestamp,
                started_at = coalesce(started_at, current_timestamp),
                completed_at = coalesce(completed_at, current_timestamp),
                duration_ms = ?3,
                exit_code = ?4,
                log_reference_id = ?5,
                output_summary = ?6,
                error_summary = ?7,
                review_state = ?8,
                metadata_json = ?9
            where id = ?1
            ",
            params![
                run_id,
                input.status.as_str(),
                input.duration_ms,
                input.exit_code,
                input.log_reference_id,
                output_summary,
                error_summary,
                input.review_state.as_str(),
                input.metadata_json
            ],
        )
        .map_err(|error| map_repository_error("agent_runs", error))?;
    let run = read_agent_run_required(connection, run_id)?;
    create_agent_run_event(
        connection,
        &run,
        input.status,
        Some(&input.output_summary),
        input.error_summary.as_deref(),
        &input.metadata_json,
    )?;
    Ok(run)
}

#[allow(dead_code)]
fn read_agent_run_required(connection: &Connection, run_id: &str) -> RepoResult<AgentRunRecord> {
    read_agent_run(connection, run_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "agent_runs",
        key: run_id.to_string(),
    })
}

#[allow(dead_code)]
fn read_agent_run(connection: &Connection, run_id: &str) -> RepoResult<Option<AgentRunRecord>> {
    connection
        .query_row(
            "
            select id, task_id, profile_id, session_id, cwd, command_snapshot,
                   profile_snapshot_json, status, created_at, updated_at, started_at,
                   completed_at, duration_ms, exit_code, log_reference_id,
                   output_summary, error_summary, review_state, metadata_json
            from agent_runs where id = ?1
            ",
            params![run_id],
            agent_run_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("agent_runs", error))
}

#[allow(dead_code)]
fn agent_run_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentRunRecord> {
    let status_text: String = row.get(7)?;
    let review_state_text: String = row.get(17)?;
    Ok(AgentRunRecord {
        id: row.get(0)?,
        task_id: row.get(1)?,
        profile_id: row.get(2)?,
        session_id: row.get(3)?,
        cwd: row.get(4)?,
        command_snapshot: row.get(5)?,
        profile_snapshot_json: row.get(6)?,
        status: AgentRunStatus::from_str(&status_text).map_err(repository_error_to_rusqlite)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        duration_ms: row.get(12)?,
        exit_code: row.get(13)?,
        log_reference_id: row.get(14)?,
        output_summary: row.get(15)?,
        error_summary: row.get(16)?,
        review_state: ReviewState::from_str(&review_state_text)
            .map_err(repository_error_to_rusqlite)?,
        metadata_json: row.get(18)?,
    })
}

#[allow(dead_code)]
fn create_agent_run_event(
    connection: &Connection,
    run: &AgentRunRecord,
    status: AgentRunStatus,
    output_summary: Option<&str>,
    error_summary: Option<&str>,
    metadata_json: &str,
) -> RepoResult<()> {
    let summary = output_summary
        .or(error_summary)
        .map(str::to_string)
        .unwrap_or_else(|| format!("Agent run {}", status.as_str()));
    let metadata = serde_json::json!({
        "run_id": run.id,
        "task_id": run.task_id,
        "session_id": run.session_id,
        "profile_id": run.profile_id,
        "status": status.as_str(),
        "duration_ms": run.duration_ms,
        "exit_code": run.exit_code,
        "log_reference_id": run.log_reference_id,
        "input_metadata": serde_json::from_str::<Value>(metadata_json).unwrap_or(Value::Null),
    })
    .to_string();
    let mut targets = vec![
        EventTargetInput {
            entity_type: "agent_run",
            entity_id: &run.id,
            relation_type: "primary",
        },
        EventTargetInput {
            entity_type: "task",
            entity_id: &run.task_id,
            relation_type: "owner",
        },
    ];
    targets.push(EventTargetInput {
        entity_type: "cli_session",
        entity_id: &run.session_id,
        relation_type: "session",
    });
    create_event_record(
        connection,
        EventCreateInput {
            action_type: status.event_type(),
            outcome: "succeeded",
            actor_type: "system",
            actor_id: None,
            workspace_key: Some("agents"),
            summary: &summary,
            source: "agent_run_repository",
            metadata_json: &metadata,
            targets,
        },
    )?;
    Ok(())
}

#[allow(dead_code)]
fn create_review_record(
    connection: &Connection,
    input: ReviewRecordCreateInput,
) -> RepoResult<ReviewRecord> {
    validate_review_record_input(connection, &input)?;
    let review_id = next_review_record_id();
    let state = input.verdict.state();
    let evidence_summary = redact_secrets(input.evidence_summary.trim()).text;
    let metadata_json = redact_metadata_json(&input.metadata_json);
    let required_fixes_json = redact_metadata_json(&input.required_fixes_json);

    connection
        .execute_batch("savepoint create_review_record")
        .map_err(|error| map_repository_error("review_records", error))?;

    let create_result = (|| -> RepoResult<ReviewRecord> {
        connection
            .execute(
                "
                insert into review_records (
                    id, subject_type, subject_id, task_id, run_id, reviewer_profile_id,
                    state, verdict, evidence_summary, required_fixes_json, metadata_json
                ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ",
                params![
                    review_id,
                    input.subject_type.as_str(),
                    input.subject_id,
                    input.task_id,
                    input.run_id,
                    input.reviewer_profile_id,
                    state.as_str(),
                    input.verdict.as_str(),
                    evidence_summary,
                    required_fixes_json,
                    metadata_json
                ],
            )
            .map_err(|error| map_repository_error("review_records", error))?;

        let review =
            read_review_record(connection, &review_id)?.ok_or(RepositoryError::NotFound {
                entity: "review_records",
                key: review_id.clone(),
            })?;
        link_review_record(connection, &review)?;
        create_review_event(connection, &review, "review.created")?;
        create_review_event(connection, &review, review.verdict.event_type())?;
        Ok(review)
    })();

    match create_result {
        Ok(review) => {
            connection
                .execute_batch("release savepoint create_review_record")
                .map_err(|error| map_repository_error("review_records", error))?;
            Ok(review)
        }
        Err(error) => {
            let _ = connection.execute_batch(
                "rollback to savepoint create_review_record; release savepoint create_review_record",
            );
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn read_review_record(connection: &Connection, id: &str) -> RepoResult<Option<ReviewRecord>> {
    connection
        .query_row(
            "
            select id, subject_type, subject_id, task_id, run_id, reviewer_profile_id,
                   state, verdict, evidence_summary, required_fixes_json, metadata_json,
                   created_at, updated_at
            from review_records where id = ?1
            ",
            params![id],
            review_record_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("review_records", error))
}

#[allow(dead_code)]
fn review_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReviewRecord> {
    let subject_type: String = row.get(1)?;
    let state: String = row.get(6)?;
    let verdict: String = row.get(7)?;
    Ok(ReviewRecord {
        id: row.get(0)?,
        subject_type: ReviewSubjectType::from_str(&subject_type)
            .map_err(repository_error_to_rusqlite)?,
        subject_id: row.get(2)?,
        task_id: row.get(3)?,
        run_id: row.get(4)?,
        reviewer_profile_id: row.get(5)?,
        state: ReviewState::from_str(&state).map_err(repository_error_to_rusqlite)?,
        verdict: ReviewVerdict::from_str(&verdict).map_err(repository_error_to_rusqlite)?,
        evidence_summary: row.get(8)?,
        required_fixes_json: row.get(9)?,
        metadata_json: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

#[allow(dead_code)]
fn validate_review_record_input(
    connection: &Connection,
    input: &ReviewRecordCreateInput,
) -> RepoResult<()> {
    let task = read_task_record(connection, &input.task_id)?;
    if task.status == TaskStatus::Deleted || task.deleted_at.is_some() {
        return Err(RepositoryError::Constraint {
            entity: "review_records",
            message: "deleted task cannot receive review".to_string(),
        });
    }
    normalize_small_text("review_records", "subject_id", input.subject_id.as_str())?;
    if input.evidence_summary.trim().is_empty() {
        return Err(RepositoryError::Constraint {
            entity: "review_records",
            message: "evidence_summary must be non-empty".to_string(),
        });
    }
    validate_no_secret_json("required_fixes_json", &input.required_fixes_json)?;
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    if input.verdict == ReviewVerdict::RequiredFixes
        && !required_fixes_payload_is_non_empty_array(&input.required_fixes_json)?
    {
        return Err(RepositoryError::Constraint {
            entity: "review_records",
            message: "required_fixes verdict requires non-empty required_fixes_json".to_string(),
        });
    }
    match input.subject_type {
        ReviewSubjectType::Task => {
            if input.subject_id != input.task_id || input.run_id.is_some() {
                return Err(RepositoryError::Constraint {
                    entity: "review_records",
                    message: "task review subject must match task_id and omit run_id".to_string(),
                });
            }
        }
        ReviewSubjectType::AgentRun => {
            let run_id = input.run_id.as_deref().ok_or(RepositoryError::Constraint {
                entity: "review_records",
                message: "agent_run review requires run_id".to_string(),
            })?;
            if input.subject_id != run_id {
                return Err(RepositoryError::Constraint {
                    entity: "review_records",
                    message: "agent_run review subject must match run_id".to_string(),
                });
            }
            let run = read_agent_run_required(connection, run_id)?;
            if run.task_id != input.task_id {
                return Err(RepositoryError::Constraint {
                    entity: "review_records",
                    message: "review run must belong to task_id".to_string(),
                });
            }
        }
        ReviewSubjectType::RelatedEntity => {
            return Err(RepositoryError::Constraint {
                entity: "review_records",
                message: "related_entity reviews require verifiable related subject support before persistence".to_string(),
            });
        }
    }
    if let Some(reviewer_profile_id) = input.reviewer_profile_id.as_deref() {
        if read_agent_profile(connection, reviewer_profile_id)?.is_none() {
            return Err(RepositoryError::NotFound {
                entity: "agent_profiles",
                key: reviewer_profile_id.to_string(),
            });
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn required_fixes_payload_is_non_empty_array(value: &str) -> RepoResult<bool> {
    match serde_json::from_str::<Value>(value).map_err(|error| RepositoryError::InvalidJson {
        field: "required_fixes_json",
        message: error.to_string(),
    })? {
        Value::Array(values) => Ok(!values.is_empty()),
        _ => Err(RepositoryError::Constraint {
            entity: "review_records",
            message:
                "required_fixes_json must be a non-empty JSON array for required_fixes verdict"
                    .to_string(),
        }),
    }
}

#[allow(dead_code)]
fn link_review_record(connection: &Connection, review: &ReviewRecord) -> RepoResult<()> {
    insert_or_get_entity_link(
        connection,
        EntityLinkInput {
            id: &format!("link_task_review_{}", review.id),
            source_type: "task",
            source_id: &review.task_id,
            target_type: "review_record",
            target_id: &review.id,
            relation_type: "reviewed_by",
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )?;
    if let Some(run_id) = review.run_id.as_deref() {
        insert_or_get_entity_link(
            connection,
            EntityLinkInput {
                id: &format!("link_run_review_{}", review.id),
                source_type: "agent_run",
                source_id: run_id,
                target_type: "review_record",
                target_id: &review.id,
                relation_type: "reviewed_by",
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
fn create_review_event(
    connection: &Connection,
    review: &ReviewRecord,
    action_type: &str,
) -> RepoResult<()> {
    let metadata = serde_json::json!({
        "review_id": review.id,
        "task_id": review.task_id,
        "run_id": review.run_id,
        "subject_type": review.subject_type.as_str(),
        "subject_id": review.subject_id,
        "state": review.state.as_str(),
        "verdict": review.verdict.as_str(),
        "required_fixes": serde_json::from_str::<Value>(&review.required_fixes_json).unwrap_or(Value::Null),
        "input_metadata": serde_json::from_str::<Value>(&review.metadata_json).unwrap_or(Value::Null),
    })
    .to_string();
    let mut targets = vec![
        EventTargetInput {
            entity_type: "review_record",
            entity_id: &review.id,
            relation_type: "primary",
        },
        EventTargetInput {
            entity_type: "task",
            entity_id: &review.task_id,
            relation_type: "owner",
        },
    ];
    if let Some(run_id) = review.run_id.as_deref() {
        targets.push(EventTargetInput {
            entity_type: "agent_run",
            entity_id: run_id,
            relation_type: "run",
        });
    }
    create_event_record(
        connection,
        EventCreateInput {
            action_type,
            outcome: "succeeded",
            actor_type: "manual_reviewer",
            actor_id: review.reviewer_profile_id.as_deref(),
            workspace_key: Some("agents"),
            summary: &format!(
                "Review {} for {} {}",
                review.verdict.as_str(),
                review.subject_type.as_str(),
                review.subject_id
            ),
            source: "review_record_repository",
            metadata_json: &metadata,
            targets,
        },
    )?;
    Ok(())
}

#[allow(dead_code)]
fn create_notification(
    connection: &Connection,
    input: NotificationCreateInput,
) -> RepoResult<NotificationRecord> {
    validate_notification_input(connection, &input)?;
    let notification_id = next_notification_id();
    let title = input.title.trim().to_string();
    let message = input.message.trim().to_string();
    let metadata_json = redact_metadata_json(&input.metadata_json);

    connection
        .execute_batch("savepoint create_notification")
        .map_err(|error| map_repository_error("notifications", error))?;

    let create_result = (|| -> RepoResult<NotificationRecord> {
        connection
            .execute(
                "
                insert into notifications (
                    id, notification_type, title, message, severity, state,
                    action_route, task_id, run_id, review_record_id, metadata_json
                ) values (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7, ?8, ?9, ?10)
                ",
                params![
                    notification_id,
                    input.notification_type.as_str(),
                    title,
                    message,
                    input.severity.as_str(),
                    input.action_route,
                    input.task_id,
                    input.run_id,
                    input.review_record_id,
                    metadata_json
                ],
            )
            .map_err(|error| map_repository_error("notifications", error))?;

        let notification =
            read_notification(connection, &notification_id)?.ok_or(RepositoryError::NotFound {
                entity: "notifications",
                key: notification_id.clone(),
            })?;
        link_notification(connection, &notification)?;
        create_notification_event(connection, &notification, "notification.created")?;
        Ok(notification)
    })();

    match create_result {
        Ok(notification) => {
            connection
                .execute_batch("release savepoint create_notification")
                .map_err(|error| map_repository_error("notifications", error))?;
            Ok(notification)
        }
        Err(error) => {
            let _ = connection.execute_batch(
                "rollback to savepoint create_notification; release savepoint create_notification",
            );
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn read_notification(connection: &Connection, id: &str) -> RepoResult<Option<NotificationRecord>> {
    connection
        .query_row(
            "
            select id, notification_type, title, message, severity, state, action_route,
                   task_id, run_id, review_record_id, read_at, dismissed_at, resolved_at,
                   created_at, updated_at, metadata_json
            from notifications where id = ?1
            ",
            params![id],
            notification_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("notifications", error))
}

#[allow(dead_code)]
fn list_inbox_notifications(
    connection: &Connection,
    active_only: bool,
    limit: i64,
) -> RepoResult<Vec<NotificationRecord>> {
    let bounded_limit = limit.clamp(1, 100);
    connection
        .prepare(
            "
            select id, notification_type, title, message, severity, state, action_route,
                   task_id, run_id, review_record_id, read_at, dismissed_at, resolved_at,
                   created_at, updated_at, metadata_json
            from notifications
            where (?1 = 0 or state in ('pending', 'delivered', 'action_required', 'failed'))
            order by case severity
                         when 'critical' then 0
                         when 'error' then 1
                         when 'warning' then 2
                         when 'success' then 3
                         else 4
                     end,
                     created_at desc,
                     id desc
            limit ?2
            ",
        )
        .and_then(|mut statement| {
            let rows = statement.query_map(
                params![if active_only { 1 } else { 0 }, bounded_limit],
                notification_from_row,
            )?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("notifications", error))
}

#[allow(dead_code)]
fn notification_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NotificationRecord> {
    let notification_type: String = row.get(1)?;
    let severity: String = row.get(4)?;
    let state: String = row.get(5)?;
    Ok(NotificationRecord {
        id: row.get(0)?,
        notification_type: NotificationType::from_str(&notification_type)
            .map_err(repository_error_to_rusqlite)?,
        title: row.get(2)?,
        message: row.get(3)?,
        severity: NotificationSeverity::from_str(&severity)
            .map_err(repository_error_to_rusqlite)?,
        state: NotificationState::from_str(&state).map_err(repository_error_to_rusqlite)?,
        action_route: row.get(6)?,
        task_id: row.get(7)?,
        run_id: row.get(8)?,
        review_record_id: row.get(9)?,
        read_at: row.get(10)?,
        dismissed_at: row.get(11)?,
        resolved_at: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        metadata_json: row.get(15)?,
    })
}

#[allow(dead_code)]
fn mark_notification_delivered(
    connection: &Connection,
    id: &str,
) -> RepoResult<NotificationRecord> {
    transition_notification_state(
        connection,
        id,
        NotificationState::Delivered,
        "notification.delivered",
    )
}

#[allow(dead_code)]
fn require_notification_action(
    connection: &Connection,
    id: &str,
) -> RepoResult<NotificationRecord> {
    transition_notification_state(
        connection,
        id,
        NotificationState::ActionRequired,
        "notification.action_required",
    )
}

#[allow(dead_code)]
fn mark_notification_failed(connection: &Connection, id: &str) -> RepoResult<NotificationRecord> {
    transition_notification_state(
        connection,
        id,
        NotificationState::Failed,
        "notification.failed",
    )
}

#[allow(dead_code)]
fn mark_notification_read(connection: &Connection, id: &str) -> RepoResult<NotificationRecord> {
    transition_notification_state(connection, id, NotificationState::Read, "notification.read")
}

#[allow(dead_code)]
fn dismiss_notification(connection: &Connection, id: &str) -> RepoResult<NotificationRecord> {
    transition_notification_state(
        connection,
        id,
        NotificationState::Dismissed,
        "notification.dismissed",
    )
}

#[allow(dead_code)]
fn resolve_notification(connection: &Connection, id: &str) -> RepoResult<NotificationRecord> {
    transition_notification_state(
        connection,
        id,
        NotificationState::Resolved,
        "notification.resolved",
    )
}

#[allow(dead_code)]
fn transition_notification_state(
    connection: &Connection,
    id: &str,
    state: NotificationState,
    action_type: &str,
) -> RepoResult<NotificationRecord> {
    if read_notification(connection, id)?.is_none() {
        return Err(RepositoryError::NotFound {
            entity: "notifications",
            key: id.to_string(),
        });
    }
    let (read_expr, dismissed_expr, resolved_expr) = match state {
        NotificationState::Read => (
            "coalesce(read_at, current_timestamp)",
            "dismissed_at",
            "resolved_at",
        ),
        NotificationState::Dismissed => (
            "read_at",
            "coalesce(dismissed_at, current_timestamp)",
            "resolved_at",
        ),
        NotificationState::Resolved => (
            "read_at",
            "dismissed_at",
            "coalesce(resolved_at, current_timestamp)",
        ),
        NotificationState::Pending
        | NotificationState::Delivered
        | NotificationState::ActionRequired
        | NotificationState::Failed => ("null", "null", "null"),
    };
    let sql = format!(
        "
        update notifications
        set state = ?2,
            read_at = {read_expr},
            dismissed_at = {dismissed_expr},
            resolved_at = {resolved_expr},
            updated_at = current_timestamp
        where id = ?1
        "
    );
    connection
        .execute(&sql, params![id, state.as_str()])
        .map_err(|error| map_repository_error("notifications", error))?;
    let notification = read_notification(connection, id)?.ok_or(RepositoryError::NotFound {
        entity: "notifications",
        key: id.to_string(),
    })?;
    create_notification_event(connection, &notification, action_type)?;
    Ok(notification)
}

#[allow(dead_code)]
fn validate_notification_input(
    connection: &Connection,
    input: &NotificationCreateInput,
) -> RepoResult<()> {
    let title = normalize_small_text("notifications", "title", input.title.as_str())?;
    validate_notification_text_no_secret("title", &title)?;
    if input.message.trim().is_empty() || input.message.len() > EVENT_CREATE_MAX_SUMMARY_BYTES {
        return Err(RepositoryError::Constraint {
            entity: "notifications",
            message: "message must be non-empty and within safe summary limits".to_string(),
        });
    }
    validate_notification_text_no_secret("message", &input.message)?;
    if let Some(action_route) = input.action_route.as_deref() {
        normalize_small_text("notifications", "action_route", action_route)?;
        validate_notification_text_no_secret("action_route", action_route)?;
    }
    validate_no_secret_json("metadata_json", &input.metadata_json)?;
    if input.task_id.is_none() && input.run_id.is_none() && input.review_record_id.is_none() {
        return Err(RepositoryError::Constraint {
            entity: "notifications",
            message: "notification must link to at least one task, run, or review".to_string(),
        });
    }
    if let Some(task_id) = input.task_id.as_deref() {
        let task = read_task_record(connection, task_id)?;
        if task.status == TaskStatus::Deleted || task.deleted_at.is_some() {
            return Err(RepositoryError::Constraint {
                entity: "notifications",
                message: "deleted task cannot receive notification".to_string(),
            });
        }
    }
    if let Some(run_id) = input.run_id.as_deref() {
        let run = read_agent_run_required(connection, run_id)?;
        if let Some(task_id) = input.task_id.as_deref() {
            if run.task_id != task_id {
                return Err(RepositoryError::Constraint {
                    entity: "notifications",
                    message: "notification run must belong to task_id".to_string(),
                });
            }
        }
    }
    if let Some(review_id) = input.review_record_id.as_deref() {
        let review =
            read_review_record(connection, review_id)?.ok_or(RepositoryError::NotFound {
                entity: "review_records",
                key: review_id.to_string(),
            })?;
        if let Some(task_id) = input.task_id.as_deref() {
            if review.task_id != task_id {
                return Err(RepositoryError::Constraint {
                    entity: "notifications",
                    message: "notification review must belong to task_id".to_string(),
                });
            }
        }
        if let Some(run_id) = input.run_id.as_deref() {
            let run = read_agent_run_required(connection, run_id)?;
            if run.task_id != review.task_id {
                return Err(RepositoryError::Constraint {
                    entity: "notifications",
                    message: "notification review task and run must share task ownership"
                        .to_string(),
                });
            }
            if let Some(review_run_id) = review.run_id.as_deref() {
                if review_run_id != run_id {
                    return Err(RepositoryError::Constraint {
                        entity: "notifications",
                        message: "notification review must belong to run_id".to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_notification_text_no_secret(field: &'static str, value: &str) -> RepoResult<()> {
    let lower = value.to_ascii_lowercase();
    if lower.contains("token=")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("authorization:")
        || lower.contains("authorization=")
        || value.split_whitespace().any(looks_like_secret_material)
    {
        return Err(reject_secret(
            field,
            "notification field contains secret-like material; store raw secrets only in Keychain",
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn link_notification(connection: &Connection, notification: &NotificationRecord) -> RepoResult<()> {
    if let Some(task_id) = notification.task_id.as_deref() {
        insert_or_get_entity_link(
            connection,
            EntityLinkInput {
                id: &format!("link_task_notification_{}", notification.id),
                source_type: "task",
                source_id: task_id,
                target_type: "notification",
                target_id: &notification.id,
                relation_type: "notifies",
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )?;
    }
    if let Some(run_id) = notification.run_id.as_deref() {
        insert_or_get_entity_link(
            connection,
            EntityLinkInput {
                id: &format!("link_run_notification_{}", notification.id),
                source_type: "agent_run",
                source_id: run_id,
                target_type: "notification",
                target_id: &notification.id,
                relation_type: "notifies",
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )?;
    }
    if let Some(review_id) = notification.review_record_id.as_deref() {
        insert_or_get_entity_link(
            connection,
            EntityLinkInput {
                id: &format!("link_review_notification_{}", notification.id),
                source_type: "review_record",
                source_id: review_id,
                target_type: "notification",
                target_id: &notification.id,
                relation_type: "notifies",
                created_by_actor_type: "system",
                metadata_json: "{}",
            },
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
fn create_notification_event(
    connection: &Connection,
    notification: &NotificationRecord,
    action_type: &str,
) -> RepoResult<()> {
    let metadata = serde_json::json!({
        "notification_id": notification.id,
        "notification_type": notification.notification_type.as_str(),
        "severity": notification.severity.as_str(),
        "state": notification.state.as_str(),
        "task_id": notification.task_id,
        "run_id": notification.run_id,
        "review_record_id": notification.review_record_id,
        "action_route": notification.action_route,
        "input_metadata": serde_json::from_str::<Value>(&notification.metadata_json).unwrap_or(Value::Null),
    })
    .to_string();
    let mut targets = vec![EventTargetInput {
        entity_type: "notification",
        entity_id: &notification.id,
        relation_type: "primary",
    }];
    if let Some(task_id) = notification.task_id.as_deref() {
        targets.push(EventTargetInput {
            entity_type: "task",
            entity_id: task_id,
            relation_type: "owner",
        });
    }
    if let Some(run_id) = notification.run_id.as_deref() {
        targets.push(EventTargetInput {
            entity_type: "agent_run",
            entity_id: run_id,
            relation_type: "run",
        });
    }
    if let Some(review_id) = notification.review_record_id.as_deref() {
        targets.push(EventTargetInput {
            entity_type: "review_record",
            entity_id: review_id,
            relation_type: "review",
        });
    }
    create_event_record(
        connection,
        EventCreateInput {
            action_type,
            outcome: "succeeded",
            actor_type: "system",
            actor_id: None,
            workspace_key: Some("inbox"),
            summary: &format!(
                "Notification {}: {}",
                notification.state.as_str(),
                notification.title
            ),
            source: "notification_repository",
            metadata_json: &metadata,
            targets,
        },
    )?;
    Ok(())
}

#[allow(dead_code)]
fn next_notification_id() -> String {
    let sequence = NOTIFICATION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "notification_{}_{:010}_{:020}",
        now_millis(),
        process::id(),
        sequence
    )
}

#[allow(dead_code)]
fn review_gate_satisfied_for_task(connection: &Connection, task_id: &str) -> RepoResult<bool> {
    read_task_record(connection, task_id)?;
    let latest: Option<(String, String)> = connection
        .query_row(
            "
            select state, verdict from review_records
            where task_id = ?1
            order by created_at desc, id desc
            limit 1
            ",
            params![task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| map_repository_error("review_records", error))?;
    Ok(latest.is_some_and(|(state, verdict)| state == "approved" && verdict == "approved"))
}

#[allow(dead_code)]
fn next_review_record_id() -> String {
    let sequence = REVIEW_RECORD_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "review_{}_{:010}_{:020}",
        now_millis(),
        process::id(),
        sequence
    )
}

#[allow(dead_code)]
fn validate_runnable_task(
    connection: &Connection,
    task_id: &str,
    entity: &'static str,
) -> RepoResult<TaskRecord> {
    let task = read_task_record(connection, task_id)?;
    if task.status == TaskStatus::Deleted || task.deleted_at.is_some() {
        return Err(RepositoryError::Constraint {
            entity,
            message: format!("task is deleted and cannot start a run: {task_id}"),
        });
    }
    Ok(task)
}

#[allow(dead_code)]
fn validate_configured_profile(profile: &AgentProfileRecord) -> RepoResult<()> {
    if !profile.configured
        || profile
            .command
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return Err(RepositoryError::Constraint {
            entity: "agent_profiles",
            message: "agent profile is unconfigured or missing command".to_string(),
        });
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_log_reference_exists(
    connection: &Connection,
    log_reference_id: &str,
) -> RepoResult<()> {
    let exists = connection
        .query_row(
            "select 1 from log_references where id = ?1",
            params![log_reference_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| map_repository_error("log_references", error))?;
    if exists.is_none() {
        return Err(RepositoryError::Constraint {
            entity: "agent_runs",
            message: format!("log_reference_id does not exist: {log_reference_id}"),
        });
    }
    Ok(())
}

#[allow(dead_code)]
fn normalize_small_text(entity: &'static str, field: &str, value: &str) -> RepoResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > EVENT_CREATE_MAX_SMALL_FIELD_BYTES {
        return Err(RepositoryError::Constraint {
            entity,
            message: format!("{field} must be non-empty and small"),
        });
    }
    Ok(trimmed.to_string())
}

#[allow(dead_code)]
fn normalize_optional_command(command: Option<&str>) -> RepoResult<Option<String>> {
    match command.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.len() <= EVENT_CREATE_MAX_SOURCE_BYTES => Ok(Some(value.to_string())),
        Some(_) => Err(RepositoryError::Constraint {
            entity: "agent_profiles",
            message: "command is too large".to_string(),
        }),
        None => Ok(None),
    }
}

#[allow(dead_code)]
fn validate_no_secret_command(command: Option<&str>) -> RepoResult<()> {
    let Some(command) = command else {
        return Ok(());
    };
    let lower = command.to_ascii_lowercase();
    let secret_flag_present = [
        "--api-key",
        "--apikey",
        "--token",
        "--password",
        "--secret",
        "api_key=",
        "apikey=",
        "token=",
        "password=",
        "secret=",
        "authorization:",
        "authorization=",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    let token_secret_present = command
        .split(|character: char| character.is_whitespace() || character == '\'' || character == '"')
        .any(looks_like_secret_material);
    if secret_flag_present || token_secret_present {
        return Err(reject_secret(
            "command",
            "command contains secret-like material; use credential_ref/env_refs instead",
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn normalize_cwd(entity: &'static str, cwd: &str) -> RepoResult<String> {
    let trimmed = cwd.trim();
    if trimmed.is_empty() || trimmed.len() > EVENT_CREATE_MAX_SOURCE_BYTES {
        return Err(RepositoryError::Constraint {
            entity,
            message: "cwd must be present before execution".to_string(),
        });
    }
    Ok(trimmed.to_string())
}

#[allow(dead_code)]
fn next_cli_session_id() -> String {
    let sequence = CLI_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "session_{}_{:010}_{:020}",
        now_millis(),
        process::id(),
        sequence
    )
}

#[allow(dead_code)]
fn next_agent_run_id() -> String {
    let sequence = AGENT_RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "run_{}_{:010}_{:020}",
        now_millis(),
        process::id(),
        sequence
    )
}

#[allow(dead_code)]
fn confirmation_decision_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ConfirmationDecisionRecord> {
    let decision_text: String = row.get(2)?;
    let actor_type_text: String = row.get(3)?;
    let decision = ConfirmationDecisionState::from_str(&decision_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{error:?}"),
            )),
        )
    })?;
    let actor_type = ConfirmationActorType::from_str(&actor_type_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{error:?}"),
            )),
        )
    })?;

    Ok(ConfirmationDecisionRecord {
        id: row.get(0)?,
        action_category: row.get(1)?,
        decision,
        actor_type,
        actor_id: row.get(4)?,
        summary: row.get(5)?,
        event_id: row.get(6)?,
        metadata_json: row.get(7)?,
        created_at: row.get(8)?,
    })
}

#[allow(dead_code)]
fn next_confirmation_decision_id() -> String {
    let sequence = CONFIRMATION_DECISION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("confirm_{}_{}", now_millis(), sequence)
}

#[allow(dead_code)]
fn validate_seeded_action_category(connection: &Connection, category: &str) -> RepoResult<String> {
    let normalized = normalize_action_category(category);
    if !ACTION_POLICY_CATEGORIES.contains(&normalized.as_str()) {
        return Err(RepositoryError::Constraint {
            entity: "confirmation_decisions",
            message: format!("unknown action policy category: {normalized}"),
        });
    }

    let exists: Option<i64> = connection
        .query_row(
            "select 1 from action_policies where category = ?1",
            params![normalized],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| map_repository_error("action_policies", error))?;

    if exists.is_none() {
        return Err(RepositoryError::Constraint {
            entity: "confirmation_decisions",
            message: format!("action policy category is not seeded: {normalized}"),
        });
    }

    Ok(normalized)
}

#[allow(dead_code)]
fn validate_event_link(connection: &Connection, event_id: Option<&str>) -> RepoResult<()> {
    if let Some(event_id) = event_id {
        let exists: Option<i64> = connection
            .query_row(
                "select 1 from events where id = ?1",
                params![event_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| map_repository_error("events", error))?;
        if exists.is_none() {
            return Err(RepositoryError::Constraint {
                entity: "confirmation_decisions",
                message: format!("event_id does not reference an existing event: {event_id}"),
            });
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn create_confirmation_decision(
    connection: &Connection,
    input: ConfirmationDecisionRequest<'_>,
) -> RepoResult<ConfirmationDecisionRecord> {
    let action_category = validate_seeded_action_category(connection, input.action_category)?;
    validate_json_field("metadata_json", input.metadata_json)?;
    validate_event_link(connection, input.event_id)?;

    let id = next_confirmation_decision_id();
    let summary = redact_secrets(input.summary).text;
    let metadata_json = redact_metadata_json(input.metadata_json);

    connection
        .execute(
            "
            insert into confirmation_decisions (
                id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json
            )
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                id,
                action_category,
                input.decision.as_str(),
                input.actor.actor_type.as_str(),
                input.actor.actor_id.as_deref(),
                summary,
                input.event_id,
                metadata_json
            ],
        )
        .map_err(|error| map_repository_error("confirmation_decisions", error))?;

    read_confirmation_decision(connection, &id)?.ok_or(RepositoryError::NotFound {
        entity: "confirmation_decisions",
        key: id,
    })
}

#[allow(dead_code)]
fn read_confirmation_decision(
    connection: &Connection,
    id: &str,
) -> RepoResult<Option<ConfirmationDecisionRecord>> {
    connection
        .query_row(
            "
            select id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json, created_at
            from confirmation_decisions
            where id = ?1
            ",
            params![id],
            confirmation_decision_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("confirmation_decisions", error))
}

#[allow(dead_code)]
fn list_confirmation_decisions(
    connection: &Connection,
    action_category: Option<&str>,
    limit: i64,
) -> RepoResult<Vec<ConfirmationDecisionRecord>> {
    let bounded_limit = limit.clamp(1, 100);
    let mut records = Vec::new();

    if let Some(category) = action_category {
        let normalized = validate_seeded_action_category(connection, category)?;
        let mut statement = connection
            .prepare(
                "
                select id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json, created_at
                from confirmation_decisions
                where action_category = ?1
                order by created_at desc, rowid desc
                limit ?2
                ",
            )
            .map_err(|error| map_repository_error("confirmation_decisions", error))?;
        let rows = statement
            .query_map(
                params![normalized, bounded_limit],
                confirmation_decision_from_row,
            )
            .map_err(|error| map_repository_error("confirmation_decisions", error))?;
        for row in rows {
            records
                .push(row.map_err(|error| map_repository_error("confirmation_decisions", error))?);
        }
    } else {
        let mut statement = connection
            .prepare(
                "
                select id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json, created_at
                from confirmation_decisions
                order by created_at desc, rowid desc
                limit ?1
                ",
            )
            .map_err(|error| map_repository_error("confirmation_decisions", error))?;
        let rows = statement
            .query_map(params![bounded_limit], confirmation_decision_from_row)
            .map_err(|error| map_repository_error("confirmation_decisions", error))?;
        for row in rows {
            records
                .push(row.map_err(|error| map_repository_error("confirmation_decisions", error))?);
        }
    }

    Ok(records)
}

#[allow(dead_code)]
fn policy_requires_hard_reviewer(policy: &ActionPolicyDecision) -> bool {
    matches!(
        policy.reviewer_required,
        ReviewerRequirement::Usually | ReviewerRequirement::Yes
    )
}

#[allow(dead_code)]
fn policy_requires_human_actor(policy: &ActionPolicyDecision) -> bool {
    matches!(
        policy.human_confirmation,
        HumanConfirmation::Yes | HumanConfirmation::Always
    )
}

#[allow(dead_code)]
fn actor_satisfies_human_confirmation(actor_type: ConfirmationActorType) -> bool {
    matches!(
        actor_type,
        ConfirmationActorType::Human
            | ConfirmationActorType::Reviewer
            | ConfirmationActorType::ReviewedClearTask
    )
}

#[allow(dead_code)]
fn actor_satisfies_clear_task(actor_type: ConfirmationActorType) -> bool {
    matches!(
        actor_type,
        ConfirmationActorType::ClearTask | ConfirmationActorType::ReviewedClearTask
    )
}

#[allow(dead_code)]
fn actor_satisfies_reviewer(actor_type: ConfirmationActorType) -> bool {
    matches!(
        actor_type,
        ConfirmationActorType::Reviewer | ConfirmationActorType::ReviewedClearTask
    )
}

#[allow(dead_code)]
fn execution_gate_result(
    allowed_now: bool,
    reason: &str,
    policy: Option<&ActionPolicyDecision>,
    confirmation: Option<&ConfirmationDecisionRecord>,
) -> ExecutionGateResult {
    ExecutionGateResult {
        allowed_now,
        reason: reason.to_string(),
        action_category: policy
            .map(|decision| decision.category.clone())
            .or_else(|| confirmation.map(|decision| decision.action_category.clone()))
            .unwrap_or_else(|| "unknown_action".to_string()),
        requires_confirmation: policy.is_some_and(|decision| decision.requires_confirmation),
        requires_reviewer: policy.is_some_and(policy_requires_hard_reviewer),
        requires_clear_task: policy.is_some_and(|decision| decision.requires_clear_task),
        confirmation_id: confirmation.map(|decision| decision.id.clone()),
    }
}

#[allow(dead_code)]
fn require_policy_clearance_before_execution(
    request: &ActionRequest,
    policy: Option<&ActionPolicyDecision>,
    confirmation: Option<&ConfirmationDecisionRecord>,
) -> ExecutionGateResult {
    let Some(policy) = policy else {
        return execution_gate_result(false, "missing_policy_decision", None, confirmation);
    };

    let request_category = normalize_action_category(classify_action_request(request));
    let policy_category = normalize_action_category(&policy.category);
    if policy_category == "unknown_action"
        || !ACTION_POLICY_CATEGORIES.contains(&policy_category.as_str())
    {
        return execution_gate_result(false, "unknown_action_category", Some(policy), confirmation);
    }
    if request_category != policy_category {
        return execution_gate_result(
            false,
            "policy_request_category_mismatch",
            Some(policy),
            confirmation,
        );
    }

    if policy.allowed_now {
        return execution_gate_result(
            true,
            "policy_allows_without_confirmation",
            Some(policy),
            None,
        );
    }

    let Some(confirmation) = confirmation else {
        return execution_gate_result(false, "confirmation_required", Some(policy), None);
    };

    if normalize_action_category(&confirmation.action_category) != policy_category {
        return execution_gate_result(
            false,
            "confirmation_category_mismatch",
            Some(policy),
            Some(confirmation),
        );
    }

    if confirmation.decision != ConfirmationDecisionState::Approved {
        return execution_gate_result(
            false,
            &format!("confirmation_{}", confirmation.decision.as_str()),
            Some(policy),
            Some(confirmation),
        );
    }

    if policy_requires_human_actor(policy)
        && !actor_satisfies_human_confirmation(confirmation.actor_type)
    {
        return execution_gate_result(
            false,
            "human_confirmation_required",
            Some(policy),
            Some(confirmation),
        );
    }

    if policy.requires_clear_task && !actor_satisfies_clear_task(confirmation.actor_type) {
        return execution_gate_result(
            false,
            "clear_task_required",
            Some(policy),
            Some(confirmation),
        );
    }

    if policy_requires_hard_reviewer(policy) && !actor_satisfies_reviewer(confirmation.actor_type) {
        return execution_gate_result(false, "reviewer_required", Some(policy), Some(confirmation));
    }

    execution_gate_result(
        true,
        "confirmation_approved",
        Some(policy),
        Some(confirmation),
    )
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
fn parse_json_field(field: &'static str, value: &str) -> RepoResult<Value> {
    serde_json::from_str::<Value>(value).map_err(|error| RepositoryError::InvalidJson {
        field,
        message: error.to_string(),
    })
}

#[allow(dead_code)]
fn reject_secret(field: &'static str, message: impl Into<String>) -> RepositoryError {
    RepositoryError::SecretRejected {
        field,
        message: message.into(),
    }
}

#[allow(dead_code)]
fn validate_no_secret_json(field: &'static str, value: &str) -> RepoResult<()> {
    let parsed = parse_json_field(field, value)?;
    if json_contains_secret_like_material(&parsed, None) {
        return Err(reject_secret(
            field,
            "JSON contains a secret-like key or value; store raw secrets only in Keychain",
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn json_contains_secret_like_material(value: &Value, key_hint: Option<&str>) -> bool {
    match value {
        Value::Object(object) => object.iter().any(|(key, child)| {
            is_secret_key(key) || json_contains_secret_like_material(child, Some(key))
        }),
        Value::Array(items) => items
            .iter()
            .any(|item| json_contains_secret_like_material(item, key_hint)),
        Value::String(text) => {
            key_hint.is_some_and(is_secret_key) || looks_like_secret_material(text)
        }
        _ => key_hint.is_some_and(is_secret_key),
    }
}

#[allow(dead_code)]
fn looks_like_secret_material(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if (lower.contains("authorization:") || lower.contains("authorization="))
        && lower.contains("bearer ")
    {
        return true;
    }
    if lower.starts_with("bearer ") || lower.starts_with("sk-") || lower.starts_with("ghp_") {
        return true;
    }
    if let Some((key, value)) = trimmed.split_once(['=', ':']) {
        if is_secret_key(key) && !value.trim().is_empty() {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn validate_local_preference_input(input: LocalPreferenceInput<'_>) -> RepoResult<()> {
    if is_secret_key(input.key) {
        return Err(reject_secret(
            "key",
            "local preference key is secret-like; store credentials in Keychain",
        ));
    }
    validate_no_secret_json("value_json", input.value_json)
}

#[allow(dead_code)]
fn local_preference_from_record(record: AppSettingRecord) -> RepoResult<LocalPreferenceRecord> {
    Ok(LocalPreferenceRecord {
        key: record.key,
        value_json: record.value_json,
        value_type: record.value_type,
        scope: SettingScope::from_str(&record.scope)?,
        description: record.description,
    })
}

#[allow(dead_code)]
fn upsert_local_app_preference(
    connection: &Connection,
    input: LocalPreferenceInput<'_>,
) -> RepoResult<LocalPreferenceRecord> {
    validate_local_preference_input(input)?;
    let record = upsert_app_setting(
        connection,
        AppSettingInput {
            key: input.key,
            value_json: input.value_json,
            value_type: input.value_type,
            scope: input.scope.as_str(),
            description: input.description,
        },
    )?;
    local_preference_from_record(record)
}

#[allow(dead_code)]
fn read_local_app_preference(
    connection: &Connection,
    key: &str,
) -> RepoResult<Option<LocalPreferenceRecord>> {
    read_app_setting(connection, key)?
        .map(local_preference_from_record)
        .transpose()
}

#[allow(dead_code)]
fn list_local_app_preferences(connection: &Connection) -> RepoResult<Vec<LocalPreferenceRecord>> {
    list_app_settings(connection)?
        .into_iter()
        .filter(|record| record.scope == "app" || record.scope == "workspace")
        .map(local_preference_from_record)
        .collect()
}

#[allow(dead_code)]
fn list_local_app_preferences_by_scope(
    connection: &Connection,
    scope: SettingScope,
) -> RepoResult<Vec<LocalPreferenceRecord>> {
    list_app_settings_by_scope(connection, scope.as_str())?
        .into_iter()
        .map(local_preference_from_record)
        .collect()
}

#[allow(dead_code)]
fn integration_status_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<IntegrationStatusRecord> {
    let status_text: String = row.get(2)?;
    let status = IntegrationStatus::from_str(&status_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{error:?}"),
            )),
        )
    })?;
    Ok(IntegrationStatusRecord {
        integration_key: row.get(0)?,
        display_name: row.get(1)?,
        status,
        config_json: row.get(3)?,
        credential_ref: row.get(4)?,
        last_checked_at: row.get(5)?,
    })
}

#[allow(dead_code)]
fn validate_credential_ref(credential_ref: Option<&str>) -> RepoResult<()> {
    if let Some(reference) = credential_ref {
        if is_secret_key(reference) || looks_like_secret_material(reference) {
            return Err(reject_secret(
                "credential_ref",
                "credential_ref must be a Keychain reference label/path/id, not raw secret material",
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_integration_status_input(input: IntegrationStatusInput<'_>) -> RepoResult<()> {
    validate_no_secret_json("config_json", input.config_json)?;
    validate_credential_ref(input.credential_ref)
}

#[allow(dead_code)]
fn upsert_integration_status(
    connection: &Connection,
    input: IntegrationStatusInput<'_>,
) -> RepoResult<IntegrationStatusRecord> {
    validate_integration_status_input(input)?;
    connection
        .execute(
            "
            insert into integration_statuses (
                integration_key, display_name, status, config_json, credential_ref,
                last_checked_at, updated_at
            )
            values (?1, ?2, ?3, ?4, ?5, ?6, current_timestamp)
            on conflict(integration_key) do update set
                display_name = excluded.display_name,
                status = excluded.status,
                config_json = excluded.config_json,
                credential_ref = excluded.credential_ref,
                last_checked_at = excluded.last_checked_at,
                updated_at = current_timestamp
            ",
            params![
                input.integration_key,
                input.display_name,
                input.status.as_str(),
                input.config_json,
                input.credential_ref,
                input.last_checked_at
            ],
        )
        .map_err(|error| map_repository_error("integration_statuses", error))?;
    read_integration_status(connection, input.integration_key)?.ok_or_else(|| {
        RepositoryError::NotFound {
            entity: "integration_statuses",
            key: input.integration_key.to_string(),
        }
    })
}

#[allow(dead_code)]
fn read_integration_status(
    connection: &Connection,
    integration_key: &str,
) -> RepoResult<Option<IntegrationStatusRecord>> {
    connection
        .query_row(
            "
            select integration_key, display_name, status, config_json, credential_ref, last_checked_at
            from integration_statuses
            where integration_key = ?1
            ",
            params![integration_key],
            integration_status_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("integration_statuses", error))
}

#[allow(dead_code)]
fn list_integration_statuses(connection: &Connection) -> RepoResult<Vec<IntegrationStatusRecord>> {
    connection
        .prepare(
            "
            select integration_key, display_name, status, config_json, credential_ref, last_checked_at
            from integration_statuses
            order by integration_key asc
            ",
        )
        .and_then(|mut statement| {
            let rows = statement.query_map([], integration_status_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("integration_statuses", error))
}

#[allow(dead_code)]
fn default_status_for_registry_state(state: WorkspaceIntegrationState) -> IntegrationStatus {
    match state {
        WorkspaceIntegrationState::NotConfigured | WorkspaceIntegrationState::NeedsPermission => {
            IntegrationStatus::NotConfigured
        }
        WorkspaceIntegrationState::Planned | WorkspaceIntegrationState::Blocked => {
            IntegrationStatus::Disabled
        }
    }
}

#[allow(dead_code)]
fn seed_default_integration_statuses(connection: &Connection) -> RepoResult<()> {
    for workspace in canonical_workspace_registry() {
        for integration in workspace.integrations {
            let config_json = serde_json::json!({
                "workspace_key": workspace.key,
                "registry_state": integration.state.as_str(),
                "registry_note": integration.note,
                "seed_source": "canonical_workspace_registry"
            })
            .to_string();
            connection
                .execute(
                    "
                    insert or ignore into integration_statuses (
                        integration_key, display_name, status, config_json, credential_ref,
                        last_checked_at, updated_at
                    )
                    values (?1, ?2, ?3, ?4, null, null, current_timestamp)
                    ",
                    params![
                        integration.key,
                        integration.label,
                        default_status_for_registry_state(integration.state).as_str(),
                        config_json
                    ],
                )
                .map_err(|error| map_repository_error("integration_statuses", error))?;
        }
    }
    Ok(())
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
            if let Some(link_for_id) = read_entity_link(connection, input.id)? {
                if !entity_link_matches_logical_tuple(&link_for_id, input) {
                    return Err(constraint_error);
                }
            }

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
fn entity_link_matches_logical_tuple(link: &EntityLinkRecord, input: EntityLinkInput<'_>) -> bool {
    link.source_type == input.source_type
        && link.source_id == input.source_id
        && link.target_type == input.target_type
        && link.target_id == input.target_id
        && link.relation_type == input.relation_type
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

#[allow(dead_code)]
fn create_entity_link(
    connection: &Connection,
    input: EntityLinkCreateRequest<'_>,
) -> RepoResult<EntityLinkRecord> {
    validate_entity_link_request(input)?;
    validate_json_field("metadata_json", input.metadata_json)?;
    let metadata_json = redact_metadata_json(input.metadata_json);

    insert_or_get_entity_link(
        connection,
        EntityLinkInput {
            id: input.id,
            source_type: input.source_type,
            source_id: input.source_id,
            target_type: input.target_type,
            target_id: input.target_id,
            relation_type: input.relation_type,
            created_by_actor_type: input.created_by_actor_type,
            metadata_json: &metadata_json,
        },
    )
}

#[allow(dead_code)]
fn get_entity_link(connection: &Connection, id: &str) -> RepoResult<Option<EntityLinkRecord>> {
    validate_non_empty_entity_link_field("id", id)?;
    read_entity_link(connection, id)
}

#[allow(dead_code)]
fn list_entity_links_by_source(
    connection: &Connection,
    filter: EntityLinkListFilter<'_>,
) -> RepoResult<Vec<EntityLinkRecord>> {
    validate_entity_link_list_filter(filter)?;
    connection
        .prepare(
            "
            select id, source_type, source_id, target_type, target_id, relation_type,
                   created_by_actor_type, metadata_json
            from entity_links
            where source_type = ?1
              and source_id = ?2
              and (?3 is null or relation_type = ?3)
              and (?4 is null or target_type = ?4)
            order by relation_type asc, target_type asc, target_id asc, id asc
            ",
        )
        .and_then(|mut statement| {
            let rows = statement.query_map(
                params![
                    filter.entity_type,
                    filter.entity_id,
                    filter.relation_type,
                    filter.counterpart_type
                ],
                entity_link_from_row,
            )?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("entity_links", error))
}

#[allow(dead_code)]
fn list_entity_links_by_target(
    connection: &Connection,
    filter: EntityLinkListFilter<'_>,
) -> RepoResult<Vec<EntityLinkRecord>> {
    validate_entity_link_list_filter(filter)?;
    connection
        .prepare(
            "
            select id, source_type, source_id, target_type, target_id, relation_type,
                   created_by_actor_type, metadata_json
            from entity_links
            where target_type = ?1
              and target_id = ?2
              and (?3 is null or relation_type = ?3)
              and (?4 is null or source_type = ?4)
            order by source_type asc, source_id asc, relation_type asc, id asc
            ",
        )
        .and_then(|mut statement| {
            let rows = statement.query_map(
                params![
                    filter.entity_type,
                    filter.entity_id,
                    filter.relation_type,
                    filter.counterpart_type
                ],
                entity_link_from_row,
            )?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| map_repository_error("entity_links", error))
}

fn validate_entity_link_request(input: EntityLinkCreateRequest<'_>) -> RepoResult<()> {
    validate_non_empty_entity_link_field("id", input.id)?;
    validate_entity_link_type("source_type", input.source_type)?;
    validate_non_empty_entity_link_field("source_id", input.source_id)?;
    validate_entity_link_type("target_type", input.target_type)?;
    validate_non_empty_entity_link_field("target_id", input.target_id)?;
    validate_non_empty_entity_link_field("relation_type", input.relation_type)?;
    validate_non_empty_entity_link_field("created_by_actor_type", input.created_by_actor_type)?;
    Ok(())
}

fn validate_entity_link_list_filter(filter: EntityLinkListFilter<'_>) -> RepoResult<()> {
    validate_entity_link_type("entity_type", filter.entity_type)?;
    validate_non_empty_entity_link_field("entity_id", filter.entity_id)?;
    if let Some(relation_type) = filter.relation_type {
        validate_non_empty_entity_link_field("relation_type", relation_type)?;
    }
    if let Some(counterpart_type) = filter.counterpart_type {
        validate_entity_link_type("counterpart_type", counterpart_type)?;
    }
    Ok(())
}

fn validate_entity_link_type(field: &'static str, entity_type: &str) -> RepoResult<()> {
    validate_non_empty_entity_link_field(field, entity_type)?;
    if ALLOWED_ENTITY_LINK_TYPES.contains(&entity_type) {
        Ok(())
    } else {
        Err(entity_link_constraint(format!(
            "unsupported {field}: {entity_type}"
        )))
    }
}

fn validate_non_empty_entity_link_field(field: &'static str, value: &str) -> RepoResult<()> {
    if value.trim().is_empty() {
        Err(entity_link_constraint(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn entity_link_constraint(message: String) -> RepositoryError {
    RepositoryError::Constraint {
        entity: "entity_links",
        message,
    }
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
    let targets = input
        .targets
        .iter()
        .map(|(entity_type, entity_id, relation_type)| EventTargetInput {
            entity_type,
            entity_id,
            relation_type,
        })
        .collect();
    create_event_record(
        connection,
        EventCreateInput {
            action_type: input.event_type,
            outcome: input.severity,
            actor_type: input.actor_type,
            actor_id: input.actor_id,
            workspace_key: input.workspace_key,
            summary: input.summary,
            source: input.source,
            metadata_json: input.metadata_json,
            targets,
        },
    )
    .map(|record| record.id)
    .map_err(repository_error_to_rusqlite)
}

#[allow(dead_code)]
fn create_event_record(
    connection: &Connection,
    input: EventCreateInput<'_>,
) -> RepoResult<EventRecord> {
    validate_json_field("metadata_json", input.metadata_json)?;

    let event_id = next_event_id();
    let redacted_summary = redact_secrets(input.summary).text;
    let redacted_metadata = redact_metadata_json(input.metadata_json);

    connection
        .execute_batch("savepoint create_event_record")
        .map_err(|error| map_repository_error("events", error))?;

    let create_result = (|| -> RepoResult<EventRecord> {
        let event_columns = table_columns(connection, "events")
            .map_err(|error| map_repository_error("events", error))?;
        if event_columns.contains("actor") {
            connection
                .execute(
                    "
                    insert into events (id, type, timestamp, actor, actor_type, actor_id, workspace_key, summary, severity, source, metadata_json)
                    values (?1, ?2, current_timestamp, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ",
                    params![
                        event_id,
                        input.action_type,
                        input.actor_id.unwrap_or(input.actor_type),
                        input.actor_type,
                        input.actor_id,
                        input.workspace_key,
                        redacted_summary,
                        input.outcome,
                        input.source,
                        redacted_metadata
                    ],
                )
                .map_err(|error| map_repository_error("events", error))?;
        } else {
            connection
                .execute(
                    "
                    insert into events (id, type, timestamp, actor_type, actor_id, workspace_key, summary, severity, source, metadata_json)
                    values (?1, ?2, current_timestamp, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    ",
                    params![
                        event_id,
                        input.action_type,
                        input.actor_type,
                        input.actor_id,
                        input.workspace_key,
                        redacted_summary,
                        input.outcome,
                        input.source,
                        redacted_metadata
                    ],
                )
                .map_err(|error| map_repository_error("events", error))?;
        }

        for target in input.targets {
            connection
                .execute(
                    "insert or ignore into event_targets (event_id, entity_type, entity_id, relation_type) values (?1, ?2, ?3, ?4)",
                    params![event_id, target.entity_type, target.entity_id, target.relation_type],
                )
                .map_err(|error| map_repository_error("event_targets", error))?;
        }

        read_event_record(connection, &event_id)
    })();

    match create_result {
        Ok(record) => {
            connection
                .execute_batch("release savepoint create_event_record")
                .map_err(|error| map_repository_error("events", error))?;
            Ok(record)
        }
        Err(error) => {
            let _ = connection.execute_batch(
                "rollback to savepoint create_event_record; release savepoint create_event_record",
            );
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn read_event_record(connection: &Connection, event_id: &str) -> RepoResult<EventRecord> {
    let mut record = connection
        .query_row(
            "
            select id, type, severity, timestamp, actor_type, actor_id, workspace_key, summary, source, metadata_json
            from events
            where id = ?1
            ",
            params![event_id],
            event_record_from_row,
        )
        .optional()
        .map_err(|error| map_repository_error("events", error))?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "events",
            key: event_id.to_string(),
        })?;
    record.targets = read_event_targets(connection, event_id)?;
    Ok(record)
}

#[allow(dead_code)]
fn list_event_records(
    connection: &Connection,
    filter: EventListFilter<'_>,
) -> RepoResult<Vec<EventRecord>> {
    let limit = normalize_event_list_limit(filter.limit);
    let mut statement = connection
        .prepare(
            "
            select id, type, severity, timestamp, actor_type, actor_id, workspace_key, summary, source, metadata_json
            from events
            where (?1 is null or workspace_key = ?1)
              and (?2 is null or type = ?2)
              and (?3 is null or severity = ?3)
              and (?4 is null or source = ?4)
            order by rowid desc
            limit ?5
            ",
        )
        .map_err(|error| map_repository_error("events", error))?;
    let rows = statement
        .query_map(
            params![
                filter.workspace_key,
                filter.action_type,
                filter.outcome,
                filter.source,
                limit
            ],
            event_record_from_row,
        )
        .map_err(|error| map_repository_error("events", error))?;

    let mut records = Vec::new();
    for row in rows {
        let mut record = row.map_err(|error| map_repository_error("events", error))?;
        record.targets = read_event_targets(connection, &record.id)?;
        records.push(record);
    }
    Ok(records)
}

#[allow(dead_code)]
fn event_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EventRecord> {
    Ok(EventRecord {
        id: row.get(0)?,
        action_type: row.get(1)?,
        outcome: row.get(2)?,
        timestamp: row.get(3)?,
        actor_type: row.get(4)?,
        actor_id: row.get(5)?,
        workspace_key: row.get(6)?,
        summary: row.get(7)?,
        source: row.get(8)?,
        metadata_json: row.get(9)?,
        targets: Vec::new(),
    })
}

#[allow(dead_code)]
fn read_event_targets(
    connection: &Connection,
    event_id: &str,
) -> RepoResult<Vec<EventTargetRecord>> {
    let mut statement = connection
        .prepare(
            "
            select entity_type, entity_id, relation_type
            from event_targets
            where event_id = ?1
            order by
                case relation_type when 'primary' then 0 else 1 end,
                entity_type,
                entity_id,
                relation_type
            ",
        )
        .map_err(|error| map_repository_error("event_targets", error))?;
    let rows = statement
        .query_map(params![event_id], |row| {
            Ok(EventTargetRecord {
                entity_type: row.get(0)?,
                entity_id: row.get(1)?,
                relation_type: row.get(2)?,
            })
        })
        .map_err(|error| map_repository_error("event_targets", error))?;

    let mut targets = Vec::new();
    for row in rows {
        targets.push(row.map_err(|error| map_repository_error("event_targets", error))?);
    }
    Ok(targets)
}

#[allow(dead_code)]
fn normalize_event_list_limit(limit: usize) -> i64 {
    let bounded = if limit == 0 { 50 } else { limit.min(200) };
    i64::try_from(bounded).unwrap_or(200)
}

fn next_event_id() -> String {
    let sequence = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_{}_{:010}_{:020}",
        now_millis(),
        process::id(),
        sequence
    )
}

fn repository_error_to_rusqlite(error: RepositoryError) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("{error:?}"),
    )))
}

fn secure_foundation_status(safe_log_probe: &SafeLogWrite) -> SecureFoundationStatus {
    let keychain = keychain_readiness_status();
    SecureFoundationStatus {
        redaction_ready: redact_secrets("api_key=secret-value").redaction_count == 1,
        safe_logging_ready: safe_log_probe.path.is_file()
            && safe_log_probe.bytes_written > 0
            && safe_log_probe.redaction_count == 0,
        action_policy_ready: true,
        event_writer_ready: true,
        keychain_status: keychain.status.clone(),
        keychain,
        sample_policy: evaluate_action_policy("send_email"),
    }
}

fn keychain_readiness_status() -> KeychainReadinessStatus {
    KeychainReadinessStatus {
        ready: false,
        status: "blocked_unverified_native_keychain_not_tested".to_string(),
        reason: "Native macOS Keychain read/write/delete probe is not implemented in this backend slice; credential storage remains disabled rather than claiming readiness."
            .to_string(),
        credential_storage_enabled: false,
        test_path_exercised: false,
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
                if is_secret_key(key) {
                    redact_json_subtree(child);
                } else {
                    redact_json_value(child, Some(key));
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                if key_hint.is_some_and(is_secret_key) {
                    redact_json_subtree(item);
                } else {
                    redact_json_value(item, key_hint);
                }
            }
        }
        Value::String(text) => {
            if key_hint.is_some_and(is_secret_key)
                || looks_like_secret_material(text)
                || redact_secrets(text).redaction_count > 0
            {
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

fn redact_json_subtree(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for child in object.values_mut() {
                redact_json_subtree(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_json_subtree(item);
            }
        }
        _ => {
            *value = Value::String("[REDACTED]".to_string());
        }
    }
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    secret_key_markers()
        .iter()
        .any(|secret_key| lower.contains(secret_key))
}

fn secret_key_markers() -> &'static [&'static str] {
    &[
        "api_key",
        "api-key",
        "x-api-key",
        "apikey",
        "access_token",
        "access-token",
        "refresh_token",
        "refresh-token",
        "auth_token",
        "auth-token",
        "token",
        "password",
        "passwd",
        "pwd",
        "secret",
        "client_secret",
        "client-secret",
        "authorization",
        "credential",
        "credentials",
        "private_key",
        "private-key",
        "privatekey",
        "bearer",
    ]
}

fn redact_line(line: &str, redaction_count: &mut usize) -> String {
    let mut cursor = 0;
    let mut redacted_line = String::new();

    while let Some(assignment) = find_secret_assignment(line, cursor) {
        redacted_line.push_str(&redact_standalone_secrets(
            &line[cursor..assignment.value_start],
            redaction_count,
        ));

        let mut value_start = assignment.value_start;
        while value_start < line.len() {
            let Some(character) = line[value_start..].chars().next() else {
                break;
            };
            if !character.is_whitespace() || character == '\n' || character == '\r' {
                break;
            }
            value_start += character.len_utf8();
            redacted_line.push(character);
        }

        let value_end = if assignment.value_has_leading_space {
            secret_value_end(line, value_start)
        } else {
            compact_secret_value_end(line, value_start)
        };
        if value_start < value_end {
            if value_start == assignment.value_start {
                redacted_line.push(' ');
            }
            redacted_line.push_str("[REDACTED]");
            *redaction_count += 1;
            cursor = value_end;
        } else {
            cursor = assignment.value_start;
        }
    }

    redacted_line.push_str(&redact_standalone_secrets(&line[cursor..], redaction_count));
    redacted_line
}

#[derive(Debug, Clone, Copy)]
struct SecretAssignment {
    value_start: usize,
    value_has_leading_space: bool,
}

fn find_secret_assignment(line: &str, from: usize) -> Option<SecretAssignment> {
    let lower = line.to_ascii_lowercase();
    secret_key_markers()
        .iter()
        .filter_map(|secret_key| {
            let mut search_from = from;
            while search_from < line.len() {
                let relative_key_index = lower[search_from..].find(secret_key)?;
                let key_index = search_from + relative_key_index;
                let mut separator_index = key_index + secret_key.len();
                separator_index = skip_inline_spaces(line, separator_index);
                if matches!(line.as_bytes().get(separator_index), Some(b'=' | b':')) {
                    return Some(SecretAssignment {
                        value_start: separator_index + 1,
                        value_has_leading_space: line[separator_index + 1..]
                            .chars()
                            .next()
                            .is_some_and(|character| {
                                character.is_whitespace() && !matches!(character, '\n' | '\r')
                            }),
                    });
                }
                search_from = key_index + secret_key.len();
            }
            None
        })
        .min_by_key(|assignment| assignment.value_start)
}

fn secret_value_end(line: &str, value_start: usize) -> usize {
    let mut index = value_start;
    let mut last_non_newline = value_start;
    while index < line.len() {
        let character = line[index..].chars().next().unwrap();
        if matches!(character, '\n' | '\r' | ',' | ';') {
            break;
        }
        if character.is_whitespace() && assignment_starts_after_spaces(line, index) {
            break;
        }
        if !character.is_whitespace() {
            last_non_newline = index + character.len_utf8();
        }
        index += character.len_utf8();
    }
    last_non_newline.max(value_start)
}

fn compact_secret_value_end(line: &str, value_start: usize) -> usize {
    let first_token_end = next_secret_token_end(line, value_start);
    if line[value_start..first_token_end].eq_ignore_ascii_case("bearer") {
        let next_token_start = skip_inline_spaces(line, first_token_end);
        if next_token_start > first_token_end {
            return next_secret_token_end(line, next_token_start);
        }
    }
    first_token_end
}

fn next_secret_token_end(line: &str, value_start: usize) -> usize {
    let mut index = value_start;
    while index < line.len() {
        let character = line[index..].chars().next().unwrap();
        if character.is_whitespace() || matches!(character, ',' | ';') {
            break;
        }
        index += character.len_utf8();
    }
    index
}

fn assignment_starts_after_spaces(line: &str, whitespace_start: usize) -> bool {
    let key_start = skip_inline_spaces(line, whitespace_start);
    if key_start == whitespace_start || key_start >= line.len() {
        return false;
    }
    let Some(separator_offset) = line[key_start..].find(['=', ':']) else {
        return false;
    };
    let candidate_key = &line[key_start..key_start + separator_offset];
    !candidate_key.trim().is_empty()
        && candidate_key
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn skip_inline_spaces(line: &str, mut index: usize) -> usize {
    while index < line.len() {
        let character = line[index..].chars().next().unwrap();
        if !character.is_whitespace() || matches!(character, '\n' | '\r') {
            break;
        }
        index += character.len_utf8();
    }
    index
}

fn redact_standalone_secrets(segment: &str, redaction_count: &mut usize) -> String {
    segment
        .split_inclusive(char::is_whitespace)
        .map(|token| {
            let token_without_whitespace = token.trim_end_matches(char::is_whitespace);
            let whitespace_suffix = &token[token_without_whitespace.len()..];
            let redacted =
                redact_standalone_secret_token(token_without_whitespace, redaction_count);
            format!("{redacted}{whitespace_suffix}")
        })
        .collect()
}

fn redact_standalone_secret_token(token: &str, redaction_count: &mut usize) -> String {
    if token.is_empty() {
        return String::new();
    }

    let prefix_len = token
        .find(|character: char| !secret_token_boundary(character))
        .unwrap_or(token.len());
    let suffix_start = token
        .rfind(|character: char| !secret_token_boundary(character))
        .map(|index| index + token[index..].chars().next().unwrap().len_utf8())
        .unwrap_or(prefix_len);
    let prefix = &token[..prefix_len];
    let core = &token[prefix_len..suffix_start];
    let suffix = &token[suffix_start..];

    if looks_like_secret_material(core) {
        *redaction_count += 1;
        format!("{prefix}[REDACTED]{suffix}")
    } else {
        token.to_string()
    }
}

fn secret_token_boundary(character: char) -> bool {
    matches!(
        character,
        ',' | ';' | ')' | ']' | '}' | '(' | '[' | '{' | '"' | '\''
    )
}

fn write_safe_log(
    connection: &Connection,
    logs_dir: &Path,
    scope: &str,
    content: &str,
) -> std::io::Result<SafeLogWrite> {
    ensure_directory(logs_dir)?;
    let safe_scope = safe_log_scope(scope);
    let relative_path = format!("{}.log", safe_scope);
    let path = logs_dir.join(&relative_path);
    ensure_safe_log_child_path(logs_dir, &path)?;
    validate_managed_file_path(&path, "log file")?;
    let redacted = redact_secrets(content);
    let mut line = redacted.text;
    if !line.ends_with('\n') {
        line.push('\n');
    }
    let (line, truncated) = truncate_safe_log_line(line);
    let rotated = rotate_safe_log_if_needed(&path, line.len())?;
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    file.write_all(line.as_bytes())?;

    upsert_log_reference(
        connection,
        &safe_scope,
        &relative_path,
        redacted.redaction_count,
        line.len(),
        rotated,
        truncated,
    )?;

    Ok(SafeLogWrite {
        path,
        redaction_count: redacted.redaction_count,
        bytes_written: line.len(),
    })
}

fn ensure_safe_log_child_path(logs_dir: &Path, path: &Path) -> std::io::Result<()> {
    let relative = path.strip_prefix(logs_dir).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "log file {} is outside logs directory {}",
                display_path(path),
                display_path(logs_dir)
            ),
        )
    })?;

    if relative.components().count() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("log file {} is not a direct logs child", display_path(path)),
        ));
    }

    Ok(())
}

fn truncate_safe_log_line(mut line: String) -> (String, bool) {
    if line.len() <= SAFE_LOG_MAX_BYTES {
        return (line, false);
    }

    const MARKER: &str = "\n[TRUNCATED]\n";
    let mut limit = SAFE_LOG_MAX_BYTES.saturating_sub(MARKER.len());
    while !line.is_char_boundary(limit) {
        limit = limit.saturating_sub(1);
    }
    line.truncate(limit);
    line.push_str(MARKER);
    (line, true)
}

fn rotate_safe_log_if_needed(path: &Path, append_len: usize) -> std::io::Result<bool> {
    validate_managed_file_path(path, "log file")?;
    let current_len = match fs::metadata(path) {
        Ok(metadata) => metadata.len() as usize,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error),
    };

    if current_len == 0 || current_len + append_len <= SAFE_LOG_MAX_BYTES {
        return Ok(false);
    }

    let rotated_path = rotated_log_path(path);
    validate_managed_file_path(&rotated_path, "rotated log file")?;
    if rotated_path.exists() {
        fs::remove_file(&rotated_path)?;
    }
    fs::rename(path, rotated_path)?;
    Ok(true)
}

fn rotated_log_path(path: &Path) -> PathBuf {
    let mut extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    if extension.is_empty() {
        extension.push_str(SAFE_LOG_ROTATED_SUFFIX);
    } else {
        extension.push('.');
        extension.push_str(SAFE_LOG_ROTATED_SUFFIX);
    }
    path.with_extension(extension)
}

fn upsert_log_reference(
    connection: &Connection,
    log_scope: &str,
    relative_path: &str,
    redaction_count: usize,
    byte_count: usize,
    rotated: bool,
    truncated: bool,
) -> std::io::Result<()> {
    let metadata_json = serde_json::json!({
        "writer": "safe_log_writer",
        "last_bytes_written": byte_count,
        "last_redaction_count": redaction_count,
        "rotated": rotated,
        "truncated": truncated,
        "max_bytes": SAFE_LOG_MAX_BYTES,
    })
    .to_string();
    let id = format!("logref_{}_{}", log_scope, now_millis());

    connection
        .execute(
            "
            insert into log_references (id, log_scope, relative_path, redaction_count, byte_count, metadata_json)
            values (?1, ?2, ?3, ?4, ?5, ?6)
            on conflict(log_scope, relative_path) do update set
                redaction_count = log_references.redaction_count + excluded.redaction_count,
                byte_count = log_references.byte_count + excluded.byte_count,
                metadata_json = excluded.metadata_json
            ",
            params![
                id,
                log_scope,
                relative_path,
                redaction_count as i64,
                byte_count as i64,
                metadata_json,
            ],
        )
        .map(|_| ())
        .map_err(std::io::Error::other)
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

#[allow(dead_code)]
fn evaluate_action_request(request: &ActionRequest) -> ActionPolicyDecision {
    let category = classify_action_request(request);
    evaluate_action_policy(category)
}

#[allow(dead_code)]
fn classify_action_request(request: &ActionRequest) -> &'static str {
    let target = request
        .target
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if matches!(request.action_type, ActionType::Unknown)
        || matches!(request.scope, Some(ActionScope::Unknown))
        || matches!(request.consequence, Some(ActionConsequence::Unknown))
    {
        return "unknown_action";
    }

    if target_contains(
        &target,
        &[
            "credential",
            "credentials",
            "secret",
            "token",
            "settings",
            "integration",
            "oauth",
        ],
    ) || matches!(
        request.consequence,
        Some(ActionConsequence::CredentialOrIntegrationChange)
    ) {
        return "change_credentials_settings_integrations";
    }

    if target_contains(&target, &["commit", "push", "merge", "pull request", "git"]) {
        return "commit_push_merge";
    }

    if target_contains(
        &target,
        &["deploy", "redeploy", "rollback", "production", "staging"],
    ) || matches!(request.action_type, ActionType::Deploy)
    {
        return "deploy_redeploy_rollback";
    }

    if target_contains(&target, &["email", "gmail", "message", "recipient"])
        || matches!(request.action_type, ActionType::Send)
    {
        return "send_email";
    }

    if target_contains(&target, &["automation schedule", "cron", "recurring job"]) {
        return "change_automation_schedule";
    }

    if target_contains(
        &target,
        &["publish", "schedule", "social", "post", "content"],
    ) || matches!(request.action_type, ActionType::Publish)
        || matches!(request.consequence, Some(ActionConsequence::PublicRelease))
    {
        return "publish_schedule_content";
    }

    if is_bulk_file_request(request, &target) {
        return "bulk_file_operations";
    }

    if request.destructive || matches!(request.consequence, Some(ActionConsequence::Destructive)) {
        if matches!(request.scope, Some(ActionScope::External)) {
            return "unknown_action";
        }
        return "delete_trash_files";
    }

    if target_contains(
        &target,
        &[
            "automation",
            "process",
            "script",
            "command",
            "execute",
            "run",
        ],
    ) || matches!(request.action_type, ActionType::Process)
        || matches!(
            request.consequence,
            Some(ActionConsequence::AutomationExecution)
        )
    {
        return "run_existing_automation";
    }

    if target_contains(&target, &["calendar", "event"]) {
        return match request.action_type {
            ActionType::Delete | ActionType::Update => "edit_delete_calendar_event",
            _ => "create_calendar_event",
        };
    }

    if matches!(request.consequence, Some(ActionConsequence::ExternalWrite))
        || target_contains(
            &target,
            &["external api", "api record", "remote api", "remote record"],
        )
        || (matches!(
            request.scope,
            Some(ActionScope::External | ActionScope::Integration)
        ) && !matches!(request.action_type, ActionType::Read))
    {
        return "external_api_write";
    }

    if matches!(request.scope, Some(ActionScope::CodeRepository))
        || target_contains(
            &target,
            &["code repo", "repository", "source file", "code file"],
        )
    {
        return "modify_code_repo_files";
    }

    match request.action_type {
        ActionType::Read => {
            if target_contains(&target, &["gmail", "calendar"])
                || matches!(request.scope, Some(ActionScope::Integration))
            {
                "read_gmail_calendar"
            } else {
                "read_local_app_data"
            }
        }
        ActionType::Create => {
            if target_contains(&target, &["note", "markdown"])
                || matches!(request.scope, Some(ActionScope::LocalPrivate))
            {
                "create_private_markdown_note"
            } else {
                "create_local_task"
            }
        }
        ActionType::Update => {
            if target_contains(&target, &["file", "visible"])
                || matches!(request.scope, Some(ActionScope::LocalVisible))
            {
                "modify_visible_non_code_file"
            } else {
                "unknown_action"
            }
        }
        ActionType::Delete => "delete_trash_files",
        ActionType::File => "move_rename_copy_file",
        ActionType::Send => "send_email",
        ActionType::Publish => "publish_schedule_content",
        ActionType::Deploy => "deploy_redeploy_rollback",
        ActionType::Process => "run_existing_automation",
        ActionType::Unknown => "unknown_action",
    }
}

fn is_bulk_file_request(request: &ActionRequest, target: &str) -> bool {
    let has_bulk_hint = request.bulk || target_contains(target, &["bulk", "many", "batch", "mass"]);
    let has_file_hint = target_contains(
        target,
        &[
            "file", "files", "folder", "folders", "path", "paths", "rename", "copy", "move",
        ],
    );

    has_bulk_hint
        && (matches!(request.action_type, ActionType::File)
            || matches!(request.scope, Some(ActionScope::LocalVisible))
            || has_file_hint)
}

#[allow(dead_code)]
fn target_contains(target: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| target.contains(needle))
}

fn evaluate_action_policy(category: &str) -> ActionPolicyDecision {
    let normalized = normalize_action_category(category);
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
        "external_api_write" => decision(
            category,
            ActionPolicy::AskBeforeAction,
            ReviewerRequirement::Maybe,
            HumanConfirmation::Yes,
            "External API writes need target/service, payload diff, and rollback preview.",
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

fn normalize_action_category(category: &str) -> String {
    category
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
}

fn decision(
    category: &str,
    policy: ActionPolicy,
    reviewer_required: ReviewerRequirement,
    human_confirmation: HumanConfirmation,
    reason: &str,
) -> ActionPolicyDecision {
    let category = normalize_action_category(category);
    let requires_confirmation = !matches!(human_confirmation, HumanConfirmation::None);
    let requires_reviewer = !matches!(reviewer_required, ReviewerRequirement::None);
    let requires_clear_task = matches!(policy, ActionPolicy::RequireClearTask);
    let allowed_now = matches!(policy, ActionPolicy::Allow)
        && !requires_confirmation
        && !requires_reviewer
        && !requires_clear_task;
    let requires_gate = !allowed_now;

    ActionPolicyDecision {
        category,
        policy,
        reviewer_required,
        human_confirmation,
        reason: reason.to_string(),
        allowed_now,
        requires_confirmation,
        requires_reviewer,
        requires_clear_task,
        requires_gate,
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
        .invoke_handler(tauri::generate_handler![
            get_foundation_status,
            get_workspace_registry,
            read_local_preference,
            list_local_preferences,
            upsert_local_preference,
            read_integration_status_command,
            list_integration_statuses_command,
            upsert_integration_status_command,
            create_event,
            read_event,
            list_events,
            preview_action_policy,
            create_task_command,
            read_task_command,
            list_tasks_command,
            update_task_command,
            update_task_status_command,
            archive_task_command,
            delete_task_command,
            start_agent_run_command,
            read_run_status_command,
            stream_run_output_command,
            cancel_run_command,
            create_manual_review_command,
            read_review_record_command,
            create_notification_command,
            read_notification_command,
            list_inbox_notifications_command,
            update_notification_state_command,
            list_task_history_command,
            list_run_history_command,
            list_notification_history_command,
            list_entity_history_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
