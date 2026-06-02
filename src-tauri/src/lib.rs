use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static CONFIRMATION_DECISION_COUNTER: AtomicU64 = AtomicU64::new(0);
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);
static TASK_COUNTER: AtomicU64 = AtomicU64::new(0);
static CLI_SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);
static AGENT_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

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
];

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
            preview_action_policy
        ])
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
        assert_eq!(
            get_migration_version(&connection).expect("migration version"),
            6
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
        assert!(events.iter().all(
            |event| !event.summary.contains(raw_log) && !event.metadata_json.contains(raw_log)
        ));
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
        update_task_status(&connection, &task.id, TaskStatus::Active)
            .expect("same status no event");
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
                config_json:
                    "{\"account\":\"ziad@example.com\",\"scopes\":[\"metadata.readonly\"]}",
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
    fn tauri_bridge_command_surface_lists_registered_p116_commands() {
        assert_eq!(TAURI_BRIDGE_COMMAND_NAMES.len(), 12);
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

    #[test]
    fn insert_or_get_entity_link_preserves_constraint_for_id_collision_with_existing_logical_tuple()
    {
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
        assert!(
            fs::metadata(&log_path).expect("active metadata").len() <= SAFE_LOG_MAX_BYTES as u64
        );
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
        let unknown_result = require_policy_clearance_before_execution(
            &unknown_request,
            Some(&unknown_policy),
            None,
        );
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
        let result = require_policy_clearance_before_execution(
            &request,
            Some(&policy),
            Some(&system_approval),
        );

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
        let clear_result = require_policy_clearance_before_execution(
            &request,
            Some(&policy),
            Some(&plain_clear_task),
        );
        assert!(!clear_result.allowed_now);
        assert_eq!(clear_result.reason, "reviewer_required");

        let plain_reviewer = ConfirmationDecisionRecord::new_for_test(
            "confirm_code_reviewer_only",
            &policy.category,
            ConfirmationDecisionState::Approved,
            ConfirmationActorType::Reviewer,
        );
        let reviewer_result = require_policy_clearance_before_execution(
            &request,
            Some(&policy),
            Some(&plain_reviewer),
        );
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

        let listed = list_confirmation_decisions(&connection, Some("send_email"), 10)
            .expect("list decisions");
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
}
