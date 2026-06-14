use serde::{Deserialize, Serialize};
use tauri::Emitter;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use wait_timeout::ChildExt;

const DEFAULT_HERMES_SESSION: &str = "most-recent-hermes-cli-session";
const HERMES_TIMEOUT_SECONDS: u64 = 300;
const HERMES_CRON_TIMEOUT_SECONDS: u64 = 90;
const GIT_TIMEOUT_SECONDS: u64 = 120;
#[allow(dead_code)]
const APPLE_NOTES_TIMEOUT_SECONDS: u64 = 30;
const REPOSITORY_SCAN_MAX_DEPTH: usize = 3;
const IGNORED_SCAN_DIRS: [&str; 7] = [
    ".git",
    "node_modules",
    "target",
    "dist",
    ".next",
    "Library",
    ".hermes",
];

const MAX_ACTIVE_HERMES_RUNS: usize = 4;

#[derive(Debug, Clone)]
struct HermesRunSlot {
    session_id: String,
    run_id: String,
    starting: bool,
    started_at: String,
    active_pid: Option<u32>,
    active_process_group: Option<u32>,
    cancel_requested: bool,
    signal_delivered: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HermesRunSnapshot {
    session_id: String,
    run_id: String,
    started_at: String,
    status: String,
    pid: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentRunEvent {
    #[serde(rename = "type")]
    event_type: String,
    run_id: String,
    session_id: String,
    timestamp: String,
    sequence: u64,
    channel: Option<String>,
    chunk: Option<String>,
    message: Option<String>,
    exit_code: Option<i32>,
}

fn emit_agent_run_event(app_handle: Option<&tauri::AppHandle>, event: AgentRunEvent) {
    if let Some(app_handle) = app_handle {
        let _ = app_handle.emit("agent-run-event", event);
    }
}

impl HermesRunSlot {
    fn starting(session_id: String, run_id: String) -> Self {
        Self {
            session_id,
            run_id,
            starting: true,
            started_at: now_millis_string(),
            active_pid: None,
            active_process_group: None,
            cancel_requested: false,
            signal_delivered: false,
        }
    }
}

#[derive(Debug, Default)]
struct HermesRunRegistry {
    runs: HashMap<String, HermesRunSlot>,
}

fn hermes_run_registry() -> &'static Mutex<HermesRunRegistry> {
    static REGISTRY: OnceLock<Mutex<HermesRunRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HermesRunRegistry::default()))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesCliStatus {
    ok: bool,
    status: String,
    message: String,
    command: Option<String>,
    session: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HermesCliMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesCliResponse {
    content: String,
    session: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponseEmailNotificationRequest {
    to: Option<String>,
    subject: String,
    summary: String,
    session_title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponseEmailNotificationResult {
    ok: bool,
    message: String,
    sent_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HermesSlashCommand {
    name: String,
    aliases: Vec<String>,
    description: String,
    category: String,
    args_hint: Option<String>,
    subcommands: Vec<String>,
    cli_only: bool,
    gateway_only: bool,
    zoid_behavior: String,
    panel: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HermesSlashExecutionResult {
    kind: String,
    content: Option<String>,
    session: Option<String>,
    panel: Option<String>,
    requires_confirmation: bool,
    command: String,
    scope: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LatestCommit {
    hash: String,
    message: String,
    date: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileManagerEntry {
    name: String,
    path: String,
    kind: String,
    size: Option<u64>,
    modified: Option<String>,
    hidden: bool,
    readonly: bool,
    children_count: Option<usize>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileManagerDirectoryListing {
    path: String,
    name: String,
    parent: Option<String>,
    entries: Vec<FileManagerEntry>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeRepository {
    id: String,
    name: String,
    path: String,
    remote_url: Option<String>,
    branch: Option<String>,
    default_branch: Option<String>,
    dirty: bool,
    latest_commit: Option<LatestCommit>,
    added_at: String,
    source: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GithubBranch {
    name: String,
    is_default: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FilePermissionMarker {
    #[serde(default)]
    updated_at: String,
    #[serde(default)]
    touched_paths: Vec<String>,
    #[serde(default)]
    remembered_paths: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrainStore {
    version: u64,
    sources: Vec<AppleNotesSource>,
    notes: Vec<BrainNote>,
    extractions: Vec<BrainExtraction>,
    task_candidates: Vec<TaskCandidate>,
    clarification_sessions: Vec<BrainClarificationSession>,
    conflicts: Vec<BrainSyncConflict>,
    updated_at: String,
}

impl Default for BrainStore {
    fn default() -> Self {
        Self {
            version: 1,
            sources: Vec::new(),
            notes: Vec::new(),
            extractions: Vec::new(),
            task_candidates: Vec::new(),
            clarification_sessions: Vec::new(),
            conflicts: Vec::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppleNotesSource {
    id: String,
    source_type: String,
    account_name: String,
    folder_name: String,
    sync_mode: String,
    enabled: bool,
    created_by_zoid: bool,
    last_synced_at: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrainNote {
    id: String,
    source_type: String,
    source_id: String,
    apple_note_id: String,
    title: String,
    body: String,
    source_folder: String,
    account_name: String,
    apple_created_at: Option<String>,
    apple_modified_at: Option<String>,
    zoid_modified_at: Option<String>,
    imported_at: String,
    last_synced_at: Option<String>,
    last_synced_title: String,
    last_synced_body: String,
    last_synced_hash: String,
    current_hash: String,
    sync_status: String,
    archived: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrainExtraction {
    id: String,
    note_id: String,
    summary: String,
    topics: Vec<String>,
    entities: Vec<String>,
    references: Vec<String>,
    decisions: Vec<String>,
    open_questions: Vec<String>,
    ambiguity_score: f64,
    extracted_at: String,
    extractor: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskCandidate {
    id: String,
    note_id: String,
    title: String,
    extracted_description: String,
    status: String,
    priority_guess: String,
    readiness_score: f64,
    clarification_session_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrainClarificationMessage {
    role: String,
    content: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrainClarificationSession {
    id: String,
    note_id: String,
    task_candidate_ids: Vec<String>,
    status: String,
    transcript: Vec<BrainClarificationMessage>,
    resolved_brief: String,
    open_questions: Vec<String>,
    hermes_session_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrainSyncConflict {
    id: String,
    note_id: String,
    apple_title: String,
    apple_body: String,
    zoid_title: String,
    zoid_body: String,
    detected_at: String,
    resolved_at: Option<String>,
    resolution: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppleNotesFolder {
    account_name: String,
    folder_name: String,
    id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppleNotesRawNote {
    account_name: String,
    folder_name: String,
    apple_note_id: Option<String>,
    title: String,
    body: String,
    created_at: Option<String>,
    modified_at: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCronJob {
    job_id: String,
    name: String,
    schedule: String,
    repeat: String,
    deliver: String,
    next_run_at: Option<String>,
    last_run_at: Option<String>,
    last_status: Option<String>,
    last_delivery_error: Option<String>,
    enabled: bool,
    state: String,
    paused_at: Option<String>,
    paused_reason: Option<String>,
    script: Option<String>,
    no_agent: bool,
    skills: Vec<String>,
    prompt_preview: String,
    enabled_toolsets: Vec<String>,
    protected: bool,
    protection_reason: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HermesWatcher {
    id: String,
    name: String,
    state: String,
    source: String,
    last_seen_at: Option<String>,
    last_status: Option<String>,
    detail: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationList {
    jobs: Vec<AutomationCronJob>,
    watchers: Vec<HermesWatcher>,
    watcher_source_status: String,
    count: usize,
    refreshed_at: String,
    hermes_command: Option<String>,
    active_profile: String,
}


const MAVOID_SOCIAL_WORKSPACE_DEFAULT: &str = "/Users/ziadnasreldin/MaVoid/social-automation-buffer";
const MAVOID_BUFFER_ENDPOINT: &str = "https://api.buffer.com/graphql";
const MAVOID_CREATOR_JOB_ID: &str = "12fd35ec77e2";
const MAVOID_MONITOR_JOB_ID: &str = "9562e7cb93b6";

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidSocialCounts {
    total_posts: usize,
    needs_review: usize,
    ready_to_schedule: usize,
    scheduled_verified: usize,
    posted: usize,
    blocked: usize,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidCredentialPresence {
    buffer_access_token: bool,
    buffer_organization_id: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidBufferHealth {
    ok: bool,
    http_status: Option<i64>,
    rate_limited: bool,
    rate_limit_window: Option<String>,
    credentials_present: MavoidCredentialPresence,
    last_checked_at: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidAutomationStatus {
    creator_job_id: String,
    creator_enabled: bool,
    creator_state: String,
    creator_next_run_at: Option<String>,
    monitor_job_id: String,
    monitor_enabled: bool,
    monitor_state: String,
    monitor_next_run_at: Option<String>,
    cooldown_job_id: Option<String>,
    cooldown_next_run_at: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidSocialSlot {
    id: String,
    date: String,
    slot_type: String,
    local_publish_time: String,
    utc_publish_time: Option<String>,
    status: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidReviewReport {
    verdict: String,
    reviewer: Option<String>,
    report_path: Option<String>,
    required_fixes: Vec<String>,
    approved_at: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidMediaAsset {
    path: String,
    public_url: Option<String>,
    content_type: Option<String>,
    bytes: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
    validated_at: Option<String>,
    provider: Option<String>,
    temporary: bool,
    validation_status: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidBufferPost {
    buffer_id: Option<String>,
    platform: String,
    channel_id: Option<String>,
    channel_display_name: Option<String>,
    scheduled_at_utc: Option<String>,
    scheduled_at_local: Option<String>,
    state: String,
    read_back_verified_at: Option<String>,
    published_url: Option<String>,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidReportRef {
    label: String,
    path: String,
    kind: String,
    created_at: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidSocialEvent {
    timestamp: String,
    actor: String,
    event_type: String,
    message: String,
    severity: String,
    evidence_path: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidSocialPost {
    id: String,
    post_date: String,
    slot_type: String,
    title: String,
    topic_or_news_item: String,
    caption: String,
    platforms: Vec<String>,
    status: String,
    review: Option<MavoidReviewReport>,
    media_assets: Vec<MavoidMediaAsset>,
    buffer_posts: Vec<MavoidBufferPost>,
    reports: Vec<MavoidReportRef>,
    events: Vec<MavoidSocialEvent>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidSocialOverview {
    workspace_path: String,
    overall_status: String,
    active_blocker: Option<String>,
    buffer_endpoint: String,
    buffer_health: MavoidBufferHealth,
    automation: MavoidAutomationStatus,
    counts: MavoidSocialCounts,
    next_slots: Vec<MavoidSocialSlot>,
    latest_report_path: Option<String>,
    updated_at: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavoidMediaValidation {
    url: String,
    ok: bool,
    http_status: Option<i32>,
    content_type: Option<String>,
    bytes: Option<u64>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct HermesProfileSettings {
    user_name: String,
    role: String,
    timezone: String,
    communication_style: String,
    preferences: String,
    hermes_memory: String,
    hermes_soul: String,
    personality_preset: String,
    response_mode: String,
    model_provider: String,
    model_name: String,
    reasoning_effort: String,
    auxiliary_model_notes: String,
    profile_mode: String,
    access_mode: String,
    approval_mode: String,
    default_workdir: String,
    trusted_projects: String,
    toolsets: String,
    mcp_servers: String,
    plugins: String,
    enabled_skills: String,
    gateway_platforms: String,
    notification_preference: String,
    voice_preference: String,
    memory_enabled: bool,
    user_profile_enabled: bool,
    auto_context_enabled: bool,
    web_search_enabled: bool,
    browser_tools_enabled: bool,
    terminal_tools_enabled: bool,
    file_tools_enabled: bool,
    cron_enabled: bool,
    checkpoints_enabled: bool,
    secret_redaction_enabled: bool,
    pii_redaction_enabled: bool,
    profile: String,
    storage_path: String,
    available_models: std::collections::BTreeMap<String, Vec<String>>,
    available_skills: Vec<String>,
    available_toolsets: Vec<String>,
    available_mcp_servers: Vec<String>,
    available_plugins: Vec<String>,
    style_templates: std::collections::BTreeMap<String, String>,
    memory_char_limit: u64,
    user_char_limit: u64,
    soul_char_limit: u64,
    updated_at: String,
}

impl Default for HermesProfileSettings {
    fn default() -> Self {
        Self {
            user_name: "Ziad Salah".to_string(),
            role: "Founder / product owner / technical operator".to_string(),
            timezone: "Africa/Cairo".to_string(),
            communication_style: "Direct, concise, practical, no fluff.".to_string(),
            preferences: String::new(),
            hermes_memory: String::new(),
            hermes_soul: String::new(),
            personality_preset: "concise".to_string(),
            response_mode: "Ask only for critical blockers; proceed on obvious defaults.".to_string(),
            model_provider: "openai-codex".to_string(),
            model_name: "gpt-5.5".to_string(),
            reasoning_effort: "medium".to_string(),
            auxiliary_model_notes: "Auto-select auxiliary models for title, compression, vision, approval, and critique unless pinned.".to_string(),
            profile_mode: "default".to_string(),
            access_mode: "full".to_string(),
            approval_mode: "off".to_string(),
            default_workdir: "~/Zoid".to_string(),
            trusted_projects: "~/Zoid\n~/.hermes/hermes-agent".to_string(),
            toolsets: "terminal\nfile\nbrowser\nweb\nskills\nmemory\nsession_search\ndelegation\ncronjob".to_string(),
            mcp_servers: "lean-ctx\ncodegraph\nstitch".to_string(),
            plugins: "security-guidance".to_string(),
            enabled_skills: "tauri-desktop-feature-development\nfeature-critique-workflow\nsubagent-driven-development\nhermes-agent".to_string(),
            gateway_platforms: "Discord #hermes".to_string(),
            notification_preference: "important".to_string(),
            voice_preference: "off".to_string(),
            memory_enabled: true,
            user_profile_enabled: true,
            auto_context_enabled: true,
            web_search_enabled: true,
            browser_tools_enabled: true,
            terminal_tools_enabled: true,
            file_tools_enabled: true,
            cron_enabled: true,
            checkpoints_enabled: false,
            secret_redaction_enabled: true,
            pii_redaction_enabled: false,
            profile: "default".to_string(),
            storage_path: String::new(),
            available_models: default_available_models(),
            available_skills: default_available_skills(),
            available_toolsets: default_available_toolsets(),
            available_mcp_servers: vec!["lean-ctx".to_string(), "codegraph".to_string(), "stitch".to_string()],
            available_plugins: vec!["security-guidance".to_string()],
            style_templates: default_style_templates(),
            memory_char_limit: 2500,
            user_char_limit: 1600,
            soul_char_limit: 0,
            updated_at: String::new(),
        }
    }
}

fn default_available_models() -> std::collections::BTreeMap<String, Vec<String>> {
    let mut models = std::collections::BTreeMap::new();
    models.insert(
        "openai-codex".to_string(),
        vec![
            "gpt-5.5".to_string(),
            "gpt-5.4".to_string(),
            "gpt-5.4-mini".to_string(),
            "gpt-5.3-codex-spark".to_string(),
        ],
    );
    models.insert(
        "openai".to_string(),
        vec![
            "gpt-5.5".to_string(),
            "gpt-5.5-pro".to_string(),
            "gpt-5.4".to_string(),
            "gpt-5.4-pro".to_string(),
            "gpt-5.4-mini".to_string(),
            "gpt-5.3".to_string(),
            "gpt-5.3-mini".to_string(),
            "gpt-5-mini".to_string(),
        ],
    );
    models
}

fn default_available_toolsets() -> Vec<String> {
    [
        "browser",
        "cronjob",
        "delegation",
        "file",
        "memory",
        "session_search",
        "skills",
        "terminal",
        "todo",
        "vision",
        "web",
    ]
    .iter()
    .map(|value| (*value).to_string())
    .collect()
}

fn default_available_skills() -> Vec<String> {
    [
        "tauri-desktop-feature-development",
        "feature-critique-workflow",
        "subagent-driven-development",
        "hermes-agent",
    ]
    .iter()
    .map(|value| (*value).to_string())
    .collect()
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn lines_to_sorted_vec(value: &str) -> Vec<String> {
    sorted_unique(
        value
            .split(|ch| ch == '\n' || ch == ',' || ch == '\r')
            .map(str::to_string),
    )
}

fn vec_to_lines(values: Vec<String>) -> String {
    sorted_unique(values).join("\n")
}

fn yaml_string_list(root: &serde_yaml::Value, path: &[&str]) -> Vec<String> {
    let mut current = root;
    for key in path {
        let Some(next) = current.get(*key) else {
            return Vec::new();
        };
        current = next;
    }
    match current {
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        serde_yaml::Value::String(value) => lines_to_sorted_vec(value),
        _ => Vec::new(),
    }
}

fn yaml_mapping_keys(root: &serde_yaml::Value, path: &[&str]) -> Vec<String> {
    let mut current = root;
    for key in path {
        let Some(next) = current.get(*key) else {
            return Vec::new();
        };
        current = next;
    }
    current
        .as_mapping()
        .map(|mapping| {
            mapping
                .keys()
                .filter_map(|key| key.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn yaml_string_sequence(values: Vec<String>) -> serde_yaml::Value {
    serde_yaml::Value::Sequence(
        sorted_unique(values)
            .into_iter()
            .map(serde_yaml::Value::String)
            .collect(),
    )
}

fn push_model(
    models: &mut std::collections::BTreeMap<String, Vec<String>>,
    provider: &str,
    model: &str,
) {
    let provider = provider.trim();
    let model = model.trim();
    if provider.is_empty() || model.is_empty() {
        return;
    }
    models
        .entry(provider.to_string())
        .or_default()
        .push(model.to_string());
}

fn merge_provider_model_cache(
    models: &mut std::collections::BTreeMap<String, Vec<String>>,
    cache_path: &Path,
) {
    let Ok(raw) = fs::read_to_string(cache_path) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    let Some(providers) = value.as_object() else {
        return;
    };
    for (provider, entry) in providers {
        if let Some(items) = entry.get("models").and_then(serde_json::Value::as_array) {
            for item in items {
                if let Some(model) = item.as_str() {
                    push_model(models, provider, model);
                }
            }
        }
    }
}

fn merge_model_catalog_cache(
    models: &mut std::collections::BTreeMap<String, Vec<String>>,
    catalog_path: &Path,
) {
    let Ok(raw) = fs::read_to_string(catalog_path) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    let Some(providers) = value
        .get("providers")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    for (provider, entry) in providers {
        if let Some(items) = entry.get("models").and_then(serde_json::Value::as_array) {
            for item in items {
                if let Some(model) = item.get("id").and_then(serde_json::Value::as_str) {
                    push_model(models, provider, model);
                    if let Some((model_provider, short_model)) = model.split_once('/') {
                        push_model(models, model_provider, short_model);
                    }
                }
            }
        }
    }
}

fn merge_live_model_caches(models: &mut std::collections::BTreeMap<String, Vec<String>>) {
    if let Ok(home) = hermes_profile_home() {
        merge_provider_model_cache(models, &home.join("provider_models_cache.json"));
        merge_model_catalog_cache(models, &home.join("cache/model_catalog.json"));
    }
    if let Ok(home) = env::var("HOME") {
        let shared = PathBuf::from(home).join(".hermes");
        merge_provider_model_cache(models, &shared.join("provider_models_cache.json"));
        merge_model_catalog_cache(models, &shared.join("cache/model_catalog.json"));
    }
}

fn set_mcp_server_enabled(
    config: &mut serde_yaml::Value,
    enabled_names: &[String],
    available_names: &[String],
) {
    let enabled: BTreeSet<String> = enabled_names.iter().cloned().collect();
    if let Some(servers) = config
        .get_mut("mcp_servers")
        .and_then(serde_yaml::Value::as_mapping_mut)
    {
        for name in available_names {
            let key = serde_yaml::Value::String(name.clone());
            if let Some(server) = servers.get_mut(&key) {
                if !server.is_mapping() {
                    *server = serde_yaml::Value::Mapping(Default::default());
                }
                if let Some(server_map) = server.as_mapping_mut() {
                    server_map.insert(
                        serde_yaml::Value::String("enabled".to_string()),
                        serde_yaml::Value::Bool(enabled.contains(name)),
                    );
                }
            }
        }
    }
}

fn toolsets_from_feature_toggles(
    settings: &HermesProfileSettings,
    base_toolsets: Vec<String>,
) -> Vec<String> {
    let mut toolsets: BTreeSet<String> = base_toolsets.into_iter().collect();
    for (enabled, names) in [
        (settings.web_search_enabled, ["web"].as_slice()),
        (
            settings.browser_tools_enabled,
            ["browser", "vision"].as_slice(),
        ),
        (settings.terminal_tools_enabled, ["terminal"].as_slice()),
        (settings.file_tools_enabled, ["file"].as_slice()),
        (settings.cron_enabled, ["cronjob"].as_slice()),
    ] {
        for name in names {
            if enabled {
                if settings
                    .available_toolsets
                    .iter()
                    .any(|toolset| toolset == name)
                {
                    toolsets.insert((*name).to_string());
                }
            } else {
                toolsets.remove(*name);
            }
        }
    }
    toolsets.into_iter().collect()
}

fn apply_toolset_toggles_from_lines(settings: &mut HermesProfileSettings) {
    let enabled: BTreeSet<String> = lines_to_sorted_vec(&settings.toolsets)
        .into_iter()
        .collect();
    settings.web_search_enabled = enabled.contains("web");
    settings.browser_tools_enabled = enabled.contains("browser");
    settings.terminal_tools_enabled = enabled.contains("terminal");
    settings.file_tools_enabled = enabled.contains("file");
    settings.cron_enabled = enabled.contains("cronjob");
}

fn safe_runtime_toolsets() -> Vec<String> {
    vec!["session_search".to_string()]
}

fn parse_hermes_skill_table(output: &str) -> Option<(Vec<String>, Vec<String>)> {
    let mut available = BTreeSet::new();
    let mut enabled = BTreeSet::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('│') {
            continue;
        }
        let cells = trimmed
            .trim_matches('│')
            .split('│')
            .map(|cell| cell.trim())
            .collect::<Vec<_>>();
        if cells.len() < 5 || cells[0] == "Name" {
            continue;
        }
        let name = cells[0];
        if name.is_empty() || name.contains('…') {
            continue;
        }
        available.insert(name.to_string());
        if cells[4].eq_ignore_ascii_case("enabled") {
            enabled.insert(name.to_string());
        }
    }
    if available.is_empty() {
        None
    } else {
        Some((
            available.into_iter().collect(),
            enabled.into_iter().collect(),
        ))
    }
}

fn discover_hermes_skill_status_from_cli() -> Option<(Vec<String>, Vec<String>)> {
    let (path, _) = find_hermes_cli()?;
    let mut command = Command::new(path);
    command.args(["skills", "list", "--source", "all"]);
    command.env("COLUMNS", "400");
    let Ok((true, stdout, _stderr)) =
        run_command_with_timeout(&mut command, Duration::from_secs(10))
    else {
        return None;
    };
    parse_hermes_skill_table(&stdout)
}

fn discover_hermes_skills() -> Vec<String> {
    if let Some((available, _enabled)) = discover_hermes_skill_status_from_cli() {
        return available;
    }
    let mut skills = BTreeSet::new();
    let mut roots = Vec::new();
    if let Ok(home) = hermes_profile_home() {
        roots.push(home.join("skills"));
    }
    if let Ok(home) = env::var("HOME") {
        roots.push(PathBuf::from(home).join(".hermes/skills"));
    }
    for root in roots {
        discover_skill_names_in_dir(&root, &mut skills);
    }
    if skills.is_empty() {
        return default_available_skills();
    }
    skills.into_iter().collect()
}

fn discover_skill_names_in_dir(dir: &Path, skills: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.join("SKILL.md").exists() {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    skills.insert(name.to_string());
                }
            } else {
                discover_skill_names_in_dir(&path, skills);
            }
        }
    }
}

fn default_style_templates() -> std::collections::BTreeMap<String, String> {
    let mut templates = std::collections::BTreeMap::new();
    templates.insert(
        "concise".to_string(),
        "Direct, concise, practical, no fluff.".to_string(),
    );
    templates.insert(
        "technical".to_string(),
        "Technical, precise, evidence-backed, with implementation details.".to_string(),
    );
    templates.insert(
        "executive".to_string(),
        "Executive summary first, risks and decisions called out clearly.".to_string(),
    );
    templates.insert(
        "teacher".to_string(),
        "Patient, explanatory, with examples and tradeoffs.".to_string(),
    );
    templates.insert(
        "creative".to_string(),
        "Creative, exploratory, concept-first, still grounded in constraints.".to_string(),
    );
    templates
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagedProvider {
    id: String,
    display_name: String,
    provider_type: String,
    provider_id: String,
    api_key_env: String,
    default_model: String,
    model_options: Vec<String>,
    base_url: String,
    status: String,
    applied: bool,
    key_stored: bool,
    created_at: String,
    updated_at: String,
    last_validated_at: String,
    last_applied_at: String,
    last_error: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInput {
    id: Option<String>,
    display_name: String,
    provider_type: String,
    provider_id: String,
    api_key_env: String,
    api_key: Option<String>,
    default_model: String,
    model_options: Vec<String>,
    base_url: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderValidationResult {
    ok: bool,
    status: String,
    message: String,
    available_models: Vec<String>,
    validated_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderApplyResult {
    ok: bool,
    message: String,
    provider: ManagedProvider,
    config_path: String,
    env_path: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderKeyReveal {
    provider_id: String,
    api_key: String,
}

fn sanitize_identifier(value: &str, fallback: &str) -> String {
    let sanitized = value
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

fn unique_models(models: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for model in models {
        let trimmed = model.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
            output.push(trimmed.to_string());
        }
    }
    output
}

fn provider_storage_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("zoid-providers.json"))
}

#[allow(dead_code)]
fn brain_storage_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("zoid-brain.json"))
}

#[allow(dead_code)]
fn load_brain_store_inner() -> Result<BrainStore, String> {
    let path = brain_storage_path()?;
    if !path.exists() {
        return Ok(BrainStore::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read Brain store: {error}"))?;
    serde_json::from_str(&raw).map_err(|error| format!("Failed to parse Brain store: {error}"))
}

#[allow(dead_code)]
fn save_brain_store_inner(store: &BrainStore) -> Result<(), String> {
    let path = brain_storage_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create Brain store directory: {error}"))?;
    }
    let _ = backup_file(&path, "zoid-brain-save")?;
    let serialized = serde_json::to_string_pretty(store)
        .map_err(|error| format!("Failed to serialize Brain store: {error}"))?;
    fs::write(&path, serialized).map_err(|error| format!("Failed to save Brain store: {error}"))
}

fn hermes_env_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join(".env"))
}

fn load_managed_providers_inner() -> Result<Vec<ManagedProvider>, String> {
    let path = provider_storage_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read managed providers: {error}"))?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("Failed to parse managed providers: {error}"))
}

fn save_managed_providers_inner(providers: &[ManagedProvider]) -> Result<(), String> {
    let path = provider_storage_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create provider settings directory: {error}"))?;
    }
    let _ = backup_file(&path, "zoid-provider-save")?;
    let serialized = serde_json::to_string_pretty(providers)
        .map_err(|error| format!("Failed to serialize managed providers: {error}"))?;
    fs::write(&path, serialized)
        .map_err(|error| format!("Failed to save managed providers: {error}"))
}

fn keychain_service() -> &'static str {
    "com.mavoid.zoid25.providers"
}
fn keychain_account(provider_id: &str) -> String {
    format!("{}:{}", active_hermes_profile(), provider_id)
}

fn validate_api_key_material(api_key: &str) -> Result<String, String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err("API key cannot be empty.".to_string());
    }
    if trimmed.chars().any(|character| {
        character == '\n' || character == '\r' || character == '\0' || character.is_control()
    }) {
        return Err(
            "API key cannot contain newlines, NUL bytes, or control characters.".to_string(),
        );
    }
    Ok(trimmed.to_string())
}

fn store_provider_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    let api_key = validate_api_key_material(api_key)?;
    let account = keychain_account(provider_id);
    if cfg!(target_os = "macos") {
        let _ = Command::new("security")
            .args([
                "delete-generic-password",
                "-s",
                keychain_service(),
                "-a",
                &account,
            ])
            .output();
        let output = Command::new("security")
            .args([
                "add-generic-password",
                "-U",
                "-s",
                keychain_service(),
                "-a",
                &account,
                "-w",
                api_key.trim(),
            ])
            .output()
            .map_err(|error| format!("Failed to call macOS Keychain: {error}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "macOS Keychain rejected the provider key: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    } else {
        Err(
            "Secure provider key storage is only implemented for macOS Keychain in this build."
                .to_string(),
        )
    }
}

fn read_provider_key(provider_id: &str) -> Result<String, String> {
    let account = keychain_account(provider_id);
    if cfg!(target_os = "macos") {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-w",
                "-s",
                keychain_service(),
                "-a",
                &account,
            ])
            .output()
            .map_err(|error| format!("Failed to read macOS Keychain: {error}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("No API key is stored for this provider yet.".to_string())
        }
    } else {
        Err(
            "Secure provider key reveal is only implemented for macOS Keychain in this build."
                .to_string(),
        )
    }
}

fn provider_from_input(
    input: ProviderInput,
    existing: Option<ManagedProvider>,
) -> Result<ManagedProvider, String> {
    let display_name = input.display_name.trim();
    if display_name.is_empty() {
        return Err("Provider display name is required.".to_string());
    }
    let provider_id = sanitize_identifier(&input.provider_id, "custom");
    let id = input
        .id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| provider_id.clone());
    let api_key_env = input.api_key_env.trim().to_uppercase();
    if api_key_env.is_empty()
        || !api_key_env.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return Err(
            "API key env must be an uppercase env var name such as GOOGLE_API_KEY.".to_string(),
        );
    }
    let mut models = unique_models(
        std::iter::once(input.default_model.clone())
            .chain(input.model_options.clone())
            .collect(),
    );
    if models.is_empty() {
        return Err("At least one model option is required.".to_string());
    }
    let default_model = input.default_model.trim().to_string();
    if default_model.is_empty() {
        return Err("Default model is required.".to_string());
    }
    if !models.iter().any(|model| model == &default_model) {
        models.insert(0, default_model.clone());
    }
    let now = now_millis_string();
    let existing = existing.unwrap_or_else(|| ManagedProvider {
        id: id.clone(),
        display_name: String::new(),
        provider_type: String::new(),
        provider_id: provider_id.clone(),
        api_key_env: String::new(),
        default_model: String::new(),
        model_options: Vec::new(),
        base_url: String::new(),
        status: "draft".to_string(),
        applied: false,
        key_stored: false,
        created_at: now.clone(),
        updated_at: String::new(),
        last_validated_at: String::new(),
        last_applied_at: String::new(),
        last_error: String::new(),
    });
    Ok(ManagedProvider {
        id,
        display_name: display_name.to_string(),
        provider_type: input.provider_type.trim().to_string(),
        provider_id,
        api_key_env,
        default_model,
        model_options: models,
        base_url: input.base_url.trim().to_string(),
        status: existing.status,
        applied: existing.applied,
        key_stored: existing.key_stored
            || input
                .api_key
                .as_deref()
                .is_some_and(|key| !key.trim().is_empty()),
        created_at: if existing.created_at.is_empty() {
            now.clone()
        } else {
            existing.created_at
        },
        updated_at: now,
        last_validated_at: existing.last_validated_at,
        last_applied_at: existing.last_applied_at,
        last_error: existing.last_error,
    })
}

fn save_managed_provider_inner(input: ProviderInput) -> Result<ManagedProvider, String> {
    let mut providers = load_managed_providers_inner()?;
    let lookup_id = input
        .id
        .as_ref()
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .map(|id| id.to_string())
        .unwrap_or_else(|| sanitize_identifier(&input.provider_id, "custom"));
    let existing_index = providers
        .iter()
        .position(|provider| provider.id == lookup_id);
    let existing = existing_index.map(|index| providers[index].clone());
    let mut provider = provider_from_input(input.clone(), existing)?;
    if providers.iter().any(|existing_provider| {
        existing_provider.id != provider.id && existing_provider.provider_id == provider.provider_id
    }) {
        return Err(format!(
            "Provider id '{}' is already used by another managed provider.",
            provider.provider_id
        ));
    }
    if let Some(api_key) = input
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
    {
        store_provider_key(&provider.id, api_key)?;
        provider.key_stored = true;
    }
    provider.applied = false;
    if provider.status == "applied" {
        provider.status = "validated".to_string();
    }
    if let Some(index) = existing_index {
        providers[index] = provider.clone();
    } else {
        providers.push(provider.clone());
    }
    save_managed_providers_inner(&providers)?;
    Ok(provider)
}

fn update_managed_provider(updated: ManagedProvider) -> Result<ManagedProvider, String> {
    let mut providers = load_managed_providers_inner()?;
    let Some(index) = providers
        .iter()
        .position(|provider| provider.id == updated.id)
    else {
        return Err("Managed provider no longer exists.".to_string());
    };
    providers[index] = updated.clone();
    save_managed_providers_inner(&providers)?;
    Ok(updated)
}

fn get_managed_provider(provider_id: &str) -> Result<ManagedProvider, String> {
    load_managed_providers_inner()?
        .into_iter()
        .find(|provider| provider.id == provider_id)
        .ok_or_else(|| "Managed provider not found.".to_string())
}

#[derive(Debug, Deserialize)]
struct GoogleModelsResponse {
    models: Option<Vec<GoogleModel>>,
}
#[derive(Debug, Deserialize)]
struct GoogleModel {
    name: String,
}

fn validate_google_provider(api_key: &str) -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| format!("Failed to create provider validator: {error}"))?;
    let response = client
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .header("x-goog-api-key", api_key)
        .send()
        .map_err(|error| format!("Google Gemini validation request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Google Gemini rejected the key or request with HTTP {}.",
            response.status()
        ));
    }
    let body: GoogleModelsResponse = response
        .json()
        .map_err(|error| format!("Google Gemini model response was not valid JSON: {error}"))?;
    Ok(unique_models(
        body.models
            .unwrap_or_default()
            .into_iter()
            .map(|model| model.name.trim_start_matches("models/").to_string())
            .collect(),
    ))
}

fn validate_managed_provider_inner(provider_id: &str) -> Result<ProviderValidationResult, String> {
    let mut provider = get_managed_provider(provider_id)?;
    let api_key = read_provider_key(&provider.id)?;
    let validated_at = now_millis_string();
    let validation = if provider.provider_type == "google-gemini"
        || provider.provider_id == "google"
        || provider.provider_id == "gemini"
    {
        validate_google_provider(&api_key)
    } else if api_key.trim().is_empty() {
        Err("Stored API key is empty.".to_string())
    } else {
        Ok(provider.model_options.clone())
    };
    match validation {
        Ok(models) => {
            let models = unique_models(
                std::iter::once(provider.default_model.clone())
                    .chain(provider.model_options.clone())
                    .chain(models.clone())
                    .collect(),
            );
            provider.model_options = models.clone();
            provider.status = "validated".to_string();
            provider.last_validated_at = validated_at.clone();
            provider.last_error.clear();
            provider.applied = false;
            update_managed_provider(provider)?;
            Ok(ProviderValidationResult {
                ok: true,
                status: "validated".to_string(),
                message: "Provider key validated and model list refreshed.".to_string(),
                available_models: models,
                validated_at,
            })
        }
        Err(error) => {
            provider.status = "invalid".to_string();
            provider.last_validated_at = validated_at.clone();
            provider.last_error = error.clone();
            provider.applied = false;
            update_managed_provider(provider)?;
            Ok(ProviderValidationResult {
                ok: false,
                status: "invalid".to_string(),
                message: error,
                available_models: Vec::new(),
                validated_at,
            })
        }
    }
}

fn write_env_var(path: &Path, key: &str, value: &str) -> Result<(), String> {
    let value = validate_api_key_material(value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create Hermes env directory: {error}"))?;
    }
    let _ = backup_file(path, "zoid-provider-apply")?;
    let raw = if path.exists() {
        fs::read_to_string(path).map_err(|error| format!("Failed to read Hermes .env: {error}"))?
    } else {
        String::new()
    };
    let prefix = format!("{key}=");
    let mut replaced = false;
    let mut lines = raw
        .lines()
        .map(|line| {
            if line.starts_with(&prefix) {
                replaced = true;
                format!("{key}={value}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if !replaced {
        lines.push(format!("{key}={value}"));
    }
    fs::write(path, format!("{}\n", lines.join("\n")))
        .map_err(|error| format!("Failed to write Hermes .env: {error}"))
}

fn apply_managed_provider_inner(provider_id: &str) -> Result<ProviderApplyResult, String> {
    let mut provider = get_managed_provider(provider_id)?;
    let api_key = read_provider_key(&provider.id)?;
    if api_key.trim().is_empty() {
        return Err(
            "Stored API key is empty; edit the provider and enter a key first.".to_string(),
        );
    }
    let env_path = hermes_env_path()?;
    write_env_var(&env_path, &provider.api_key_env, &api_key)?;
    if provider.provider_id == "google" && provider.api_key_env != "GEMINI_API_KEY" {
        write_env_var(&env_path, "GEMINI_API_KEY", &api_key)?;
    }
    let config_path = hermes_config_path()?;
    let raw = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|error| format!("Failed to read Hermes config: {error}"))?
    } else {
        String::new()
    };
    let mut config: serde_yaml::Value = if raw.trim().is_empty() {
        serde_yaml::Value::Mapping(Default::default())
    } else {
        serde_yaml::from_str(&raw)
            .map_err(|error| format!("Failed to parse Hermes config: {error}"))?
    };
    let _ = backup_file(&config_path, "zoid-provider-apply")?;
    yaml_set(
        &mut config,
        &["model", "provider"],
        serde_yaml::Value::String(provider.provider_id.clone()),
    );
    yaml_set(
        &mut config,
        &["model", "default"],
        serde_yaml::Value::String(provider.default_model.clone()),
    );
    yaml_set(
        &mut config,
        &["providers", &provider.provider_id, "model"],
        serde_yaml::Value::String(provider.default_model.clone()),
    );
    if !provider.base_url.trim().is_empty() {
        yaml_set(
            &mut config,
            &["model", "base_url"],
            serde_yaml::Value::String(provider.base_url.clone()),
        );
        yaml_set(
            &mut config,
            &["providers", &provider.provider_id, "base_url"],
            serde_yaml::Value::String(provider.base_url.clone()),
        );
    }
    let serialized = serde_yaml::to_string(&config)
        .map_err(|error| format!("Failed to serialize Hermes config: {error}"))?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create Hermes config directory: {error}"))?;
    }
    fs::write(&config_path, serialized)
        .map_err(|error| format!("Failed to save Hermes config: {error}"))?;
    provider.status = "applied".to_string();
    provider.applied = true;
    provider.last_applied_at = now_millis_string();
    provider.last_error.clear();
    let provider = update_managed_provider(provider)?;
    Ok(ProviderApplyResult { ok: true, message: "Provider applied to Hermes config and .env. Restart Hermes sessions to use the new key if one is already running.".to_string(), provider, config_path: config_path.to_string_lossy().to_string(), env_path: env_path.to_string_lossy().to_string() })
}

fn hermes_session_name() -> String {
    env::var("ZOID_HERMES_SESSION").unwrap_or_else(|_| DEFAULT_HERMES_SESSION.to_string())
}

fn candidate_hermes_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(explicit) = env::var("ZOID_HERMES_CLI") {
        paths.push(PathBuf::from(explicit));
    }
    paths.push(PathBuf::from("hermes"));
    if let Ok(home) = env::var("HOME") {
        paths.push(PathBuf::from(format!(
            "{home}/.hermes/hermes-agent/venv/bin/hermes"
        )));
        paths.push(PathBuf::from(format!("{home}/.local/bin/hermes")));
        paths.push(PathBuf::from(format!("{home}/.cargo/bin/hermes")));
    }
    paths.push(PathBuf::from("/opt/homebrew/bin/hermes"));
    paths.push(PathBuf::from("/usr/local/bin/hermes"));
    paths
}

fn command_display(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[allow(dead_code)]
fn jxa_json_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

#[allow(dead_code)]
fn run_apple_notes_script(script: &str) -> Result<String, String> {
    let mut command = Command::new("osascript");
    command.arg("-l").arg("JavaScript").arg("-e").arg(script);
    let (success, stdout, stderr) = run_command_with_timeout(
        &mut command,
        Duration::from_secs(APPLE_NOTES_TIMEOUT_SECONDS),
    )?;
    if success {
        Ok(stdout)
    } else {
        let error = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        Err(format!("Apple Notes automation failed: {error}"))
    }
}

fn parse_apple_notes_folders_json(raw: &str) -> Result<Vec<AppleNotesFolder>, String> {
    let mut folders: Vec<AppleNotesFolder> = serde_json::from_str(raw.trim())
        .map_err(|error| format!("Failed to parse Apple Notes folders JSON: {error}"))?;
    folders.retain(|folder| {
        !folder.account_name.trim().is_empty() && !folder.folder_name.trim().is_empty()
    });
    folders.sort_by(|a, b| {
        a.account_name
            .cmp(&b.account_name)
            .then(a.folder_name.cmp(&b.folder_name))
    });
    folders.dedup_by(|a, b| {
        a.account_name == b.account_name && a.folder_name == b.folder_name && a.id == b.id
    });
    Ok(folders)
}

fn parse_apple_notes_raw_notes_json(raw: &str) -> Result<Vec<AppleNotesRawNote>, String> {
    serde_json::from_str(raw.trim())
        .map_err(|error| format!("Failed to parse Apple Notes notes JSON: {error}"))
}

fn simple_stable_hash(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn note_content_hash(title: &str, body: &str) -> String {
    simple_stable_hash(&format!("title\0{title}\0body\0{body}"))
}

fn apple_source_id(account_name: &str, folder_name: &str) -> String {
    format!(
        "apple-notes:{}:{}",
        sanitize_identifier(account_name, "account"),
        sanitize_identifier(folder_name, "folder")
    )
}

fn apple_note_fallback_id(raw: &AppleNotesRawNote) -> String {
    raw.apple_note_id
        .as_ref()
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "fallback-{}",
                simple_stable_hash(&format!(
                    "{}\0{}\0{}\0{}\0{}",
                    raw.account_name,
                    raw.folder_name,
                    raw.title,
                    raw.created_at.clone().unwrap_or_default(),
                    raw.modified_at.clone().unwrap_or_default()
                ))
            )
        })
}

fn brain_note_id(source: &AppleNotesSource, apple_key: &str) -> String {
    format!(
        "brain-note-{}",
        simple_stable_hash(&format!("{}\0{}", source.id, apple_key))
    )
}

fn upsert_apple_notes_source(store: &mut BrainStore, source: AppleNotesSource) -> AppleNotesSource {
    let existing_index = store.sources.iter().position(|existing| {
        existing.source_type == "appleNotes"
            && existing.account_name == source.account_name
            && existing.folder_name == source.folder_name
    });
    if let Some(index) = existing_index {
        let existing = &mut store.sources[index];
        existing.id = source.id.clone();
        existing.source_type = "appleNotes".to_string();
        existing.enabled = source.enabled;
        existing.created_by_zoid = existing.created_by_zoid || source.created_by_zoid;
        existing.sync_mode = source.sync_mode.clone();
        if source.last_synced_at.is_some() {
            existing.last_synced_at = source.last_synced_at.clone();
        }
        if source.last_error.is_some() {
            existing.last_error = source.last_error.clone();
        }
        existing.clone()
    } else {
        store.sources.push(source.clone());
        source
    }
}

fn source_from_folder(
    folder: &AppleNotesFolder,
    created_by_zoid: bool,
    sync_mode: &str,
) -> AppleNotesSource {
    AppleNotesSource {
        id: apple_source_id(&folder.account_name, &folder.folder_name),
        source_type: "appleNotes".to_string(),
        account_name: folder.account_name.clone(),
        folder_name: folder.folder_name.clone(),
        sync_mode: match sync_mode {
            "readOnly" | "twoWay" | "ignored" => sync_mode,
            _ => "readOnly",
        }
        .to_string(),
        enabled: sync_mode != "ignored",
        created_by_zoid,
        last_synced_at: None,
        last_error: None,
    }
}

fn normalize_apple_notes_body(raw: &str) -> String {
    let mut text = raw.replace("\r\n", "\n").replace('\r', "\n");
    for needle in [
        "<br>", "<br/>", "<br />", "</div>", "</p>", "</li>", "</h1>", "</h2>", "</h3>",
    ] {
        text = text.replace(needle, "\n");
        text = text.replace(&needle.to_ascii_uppercase(), "\n");
    }
    text = text.replace("<li>", "- ").replace("<LI>", "- ");
    text = text
        .replace("<div>", "")
        .replace("<DIV>", "")
        .replace("<p>", "")
        .replace("<P>", "");
    text = text
        .replace("<ul>", "")
        .replace("</ul>", "")
        .replace("<ol>", "")
        .replace("</ol>", "");
    let mut out = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'");
    decoded
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .split('\n')
        .fold(Vec::<String>::new(), |mut acc, line| {
            if !(line.trim().is_empty() && acc.last().is_some_and(|last| last.trim().is_empty())) {
                acc.push(line.to_string());
            }
            acc
        })
        .join("\n")
        .trim()
        .to_string()
}

fn merge_apple_notes_raw_notes(
    store: &mut BrainStore,
    source: &AppleNotesSource,
    raw_notes: &[AppleNotesRawNote],
    synced_at: &str,
) {
    let mut seen_keys = HashSet::new();
    for raw in raw_notes.iter().filter(|note| {
        note.account_name == source.account_name && note.folder_name == source.folder_name
    }) {
        let apple_key = apple_note_fallback_id(raw);
        seen_keys.insert(apple_key.clone());
        let normalized_body = normalize_apple_notes_body(&raw.body);
        let new_hash = note_content_hash(&raw.title, &normalized_body);
        if let Some(note_index) = store.notes.iter().position(|note| {
            note.source_id == source.id
                && (note.apple_note_id == apple_key
                    || (!note.apple_note_id.is_empty()
                        && raw.apple_note_id.as_deref() == Some(note.apple_note_id.as_str())))
        }) {
            let note = &mut store.notes[note_index];
            let zoid_unchanged = note.current_hash == note.last_synced_hash
                || note.current_hash
                    == note_content_hash(&note.last_synced_title, &note.last_synced_body);
            let apple_changed = new_hash != note.last_synced_hash;
            let mut pending_conflict: Option<BrainSyncConflict> = None;
            match (apple_changed, zoid_unchanged) {
                (true, false) => {
                    note.sync_status = "conflict".to_string();
                    let conflict_id = format!(
                        "brain-conflict-{}",
                        simple_stable_hash(&format!(
                            "{}\0{}\0{}",
                            note.id, note.current_hash, new_hash
                        ))
                    );
                    pending_conflict = Some(BrainSyncConflict {
                        id: conflict_id,
                        note_id: note.id.clone(),
                        apple_title: raw.title.clone(),
                        apple_body: normalized_body.clone(),
                        zoid_title: note.title.clone(),
                        zoid_body: note.body.clone(),
                        detected_at: synced_at.to_string(),
                        resolved_at: None,
                        resolution: None,
                    });
                }
                (false, false) => {
                    note.apple_note_id = apple_key.clone();
                    note.apple_created_at = raw.created_at.clone();
                    note.apple_modified_at = raw.modified_at.clone();
                    note.sync_status = "changedInZoid".to_string();
                }
                (true, true) | (false, true) => {
                    note.apple_note_id = apple_key.clone();
                    note.title = raw.title.clone();
                    note.body = normalized_body.clone();
                    note.apple_created_at = raw.created_at.clone();
                    note.apple_modified_at = raw.modified_at.clone();
                    note.last_synced_title = raw.title.clone();
                    note.last_synced_body = normalized_body.clone();
                    note.last_synced_hash = new_hash.clone();
                    note.current_hash = new_hash;
                    note.last_synced_at = Some(synced_at.to_string());
                    note.sync_status = "synced".to_string();
                }
            }
            if let Some(conflict) = pending_conflict {
                if !store
                    .conflicts
                    .iter()
                    .any(|existing| existing.id == conflict.id)
                {
                    store.conflicts.push(conflict);
                }
            }
        } else {
            store.notes.push(BrainNote {
                id: brain_note_id(source, &apple_key),
                source_type: "appleNotes".to_string(),
                source_id: source.id.clone(),
                apple_note_id: apple_key,
                title: raw.title.clone(),
                body: normalized_body.clone(),
                source_folder: source.folder_name.clone(),
                account_name: source.account_name.clone(),
                apple_created_at: raw.created_at.clone(),
                apple_modified_at: raw.modified_at.clone(),
                zoid_modified_at: None,
                imported_at: synced_at.to_string(),
                last_synced_at: Some(synced_at.to_string()),
                last_synced_title: raw.title.clone(),
                last_synced_body: normalized_body.clone(),
                last_synced_hash: new_hash.clone(),
                current_hash: new_hash,
                sync_status: "synced".to_string(),
                archived: false,
            });
        }
    }
    for note in store
        .notes
        .iter_mut()
        .filter(|note| note.source_id == source.id && !note.archived)
    {
        if !seen_keys.contains(&note.apple_note_id) {
            note.sync_status = "missingInApple".to_string();
        }
    }
}

fn list_apple_notes_folders_inner() -> Result<Vec<AppleNotesFolder>, String> {
    let script = r#"
const Notes = Application('Notes');
Notes.includeStandardAdditions = true;
const out = [];
for (const account of Notes.accounts()) {
  const accountName = account.name();
  for (const folder of account.folders()) {
    let id = null;
    try { id = folder.id(); } catch (e) {}
    out.push({ accountName, folderName: folder.name(), id });
  }
}
JSON.stringify(out);
"#;
    parse_apple_notes_folders_json(&run_apple_notes_script(script)?)
}

fn ensure_zoid_brain_folder_inner() -> Result<AppleNotesSource, String> {
    let folder_name = jxa_json_string_literal("Zoid Brain");
    let script = format!(
        r#"
const Notes = Application('Notes');
Notes.includeStandardAdditions = true;
const targetName = {folder_name};
const account = Notes.defaultAccount ? Notes.defaultAccount() : Notes.accounts()[0];
let created = false;
let target = null;
for (const folder of account.folders()) {{ if (folder.name() === targetName) {{ target = folder; break; }} }}
if (!target) {{ target = Notes.Folder({{name: targetName}}); account.folders.push(target); created = true; }}
let id = null;
try {{ id = target.id(); }} catch (e) {{}}
JSON.stringify({{accountName: account.name(), folderName: target.name(), id, createdByZoid: created}});
"#
    );
    let value: serde_json::Value = serde_json::from_str(&run_apple_notes_script(&script)?)
        .map_err(|error| format!("Failed to parse Zoid Brain folder JSON: {error}"))?;
    let folder = AppleNotesFolder {
        account_name: value
            .get("accountName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Apple Notes")
            .to_string(),
        folder_name: value
            .get("folderName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Zoid Brain")
            .to_string(),
        id: value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    };
    let created_by_zoid = value
        .get("createdByZoid")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut store = load_brain_store_inner()?;
    let source = upsert_apple_notes_source(
        &mut store,
        source_from_folder(&folder, created_by_zoid, "twoWay"),
    );
    store.updated_at = now_millis_string();
    save_brain_store_inner(&store)?;
    Ok(source)
}

fn link_apple_notes_folder_inner(
    account_name: String,
    folder_name: String,
    sync_mode: String,
) -> Result<AppleNotesSource, String> {
    let account_name = account_name.trim().to_string();
    let folder_name = folder_name.trim().to_string();
    if account_name.is_empty() || folder_name.is_empty() {
        return Err("Apple Notes account and folder are required.".to_string());
    }
    if !matches!(sync_mode.as_str(), "readOnly" | "twoWay" | "ignored") {
        return Err("Unsupported Apple Notes sync mode.".to_string());
    }
    let folders = list_apple_notes_folders_inner()?;
    let folder = folders
        .into_iter()
        .find(|folder| folder.account_name == account_name && folder.folder_name == folder_name)
        .ok_or_else(|| {
            "Apple Notes folder was not found. Refresh folders and try again.".to_string()
        })?;
    let mut store = load_brain_store_inner()?;
    let source =
        upsert_apple_notes_source(&mut store, source_from_folder(&folder, false, &sync_mode));
    store.updated_at = now_millis_string();
    save_brain_store_inner(&store)?;
    Ok(source)
}

fn apple_notes_source_notes_script(source: &AppleNotesSource) -> String {
    let account_name = jxa_json_string_literal(&source.account_name);
    let folder_name = jxa_json_string_literal(&source.folder_name);
    format!(
        r#"
const Notes = Application('Notes');
Notes.includeStandardAdditions = true;
const accountName = {account_name};
const folderName = {folder_name};
const out = [];
let targetAccount = null;
for (const account of Notes.accounts()) {{ if (account.name() === accountName) {{ targetAccount = account; break; }} }}
if (targetAccount) {{
  let targetFolder = null;
  for (const folder of targetAccount.folders()) {{ if (folder.name() === folderName) {{ targetFolder = folder; break; }} }}
  if (targetFolder) {{
    for (const note of targetFolder.notes()) {{
      let id = null, createdAt = null, modifiedAt = null, body = '';
      try {{ id = note.id(); }} catch (e) {{}}
      try {{ createdAt = String(note.creationDate()); }} catch (e) {{}}
      try {{ modifiedAt = String(note.modificationDate()); }} catch (e) {{}}
      try {{ body = note.body(); }} catch (e) {{}}
      out.push({{accountName, folderName, appleNoteId: id, title: note.name(), body, createdAt, modifiedAt}});
    }}
  }}
}}
JSON.stringify(out);
"#
    )
}

fn apple_notes_syncable_sources(store: &BrainStore) -> Vec<AppleNotesSource> {
    store
        .sources
        .iter()
        .filter(|source| {
            source.source_type == "appleNotes" && source.enabled && source.sync_mode != "ignored"
        })
        .cloned()
        .collect::<Vec<_>>()
}

fn apply_apple_notes_source_sync_result(
    store: &mut BrainStore,
    source: &AppleNotesSource,
    result: Result<Vec<AppleNotesRawNote>, String>,
    synced_at: &str,
) {
    match result {
        Ok(raw_notes) => {
            merge_apple_notes_raw_notes(store, source, &raw_notes, synced_at);
            if let Some(existing) = store
                .sources
                .iter_mut()
                .find(|existing| existing.id == source.id)
            {
                existing.last_synced_at = Some(synced_at.to_string());
                existing.last_error = None;
            }
        }
        Err(error) => {
            if let Some(existing) = store
                .sources
                .iter_mut()
                .find(|existing| existing.id == source.id)
            {
                existing.last_error = Some(error);
            }
        }
    }
}

fn sync_apple_notes_sources_inner() -> Result<BrainStore, String> {
    let mut store = load_brain_store_inner()?;
    let sources = apple_notes_syncable_sources(&store);
    let synced_at = now_millis_string();
    for source in sources {
        let script = apple_notes_source_notes_script(&source);
        let result =
            run_apple_notes_script(&script).and_then(|raw| parse_apple_notes_raw_notes_json(&raw));
        apply_apple_notes_source_sync_result(&mut store, &source, result, &synced_at);
    }
    store.updated_at = synced_at;
    save_brain_store_inner(&store)?;
    Ok(store)
}

fn meaningful_note_text(note: &BrainNote) -> String {
    [note.title.as_str(), note.body.as_str()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate_words(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut out = String::new();
    for word in trimmed.split_whitespace() {
        let next_len =
            out.chars().count() + if out.is_empty() { 0 } else { 1 } + word.chars().count();
        if next_len > max_chars {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    if out.is_empty() {
        trimmed.chars().take(max_chars).collect::<String>()
    } else {
        format!("{}…", out.trim_end_matches(&['.', ',', ';', ':'][..]))
    }
}

fn note_summary(note: &BrainNote) -> String {
    let text = meaningful_note_text(note);
    let first = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("Untitled note");
    let sentence_end = first
        .find(['.', '!', '?'])
        .map(|idx| idx + 1)
        .unwrap_or(first.len());
    truncate_words(&first[..sentence_end], 140)
}

fn extract_topics(text: &str) -> Vec<String> {
    sorted_unique(text.split_whitespace().filter_map(|word| {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '#');
        clean
            .strip_prefix('#')
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
    }))
}

fn extract_references(text: &str) -> Vec<String> {
    sorted_unique(
        text.split_whitespace()
            .filter(|word| word.starts_with("http://") || word.starts_with("https://"))
            .map(|word| {
                word.trim_matches(|c: char| matches!(c, ')' | ']' | ',' | '.'))
                    .to_string()
            }),
    )
}

fn extract_entities(text: &str) -> Vec<String> {
    let stop = [
        "I", "The", "This", "That", "TODO", "Need", "Maybe", "Could", "Please",
    ];
    let mut entities = Vec::new();
    for word in text.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.len() > 2
            && clean.chars().next().is_some_and(|c| c.is_uppercase())
            && !stop.contains(&clean)
        {
            entities.push(clean.to_string());
        }
    }
    sorted_unique(entities).into_iter().take(12).collect()
}

fn is_task_like_line(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_start_matches(['•', '*']).trim();
    let prefixes = ["- [ ]", "- TODO", "TODO:", "TODO ", "todo:", "todo "];
    for prefix in prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let title = rest
                .trim_matches(|c: char| matches!(c, '-' | ':' | '[' | ']'))
                .trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    let mut chars = trimmed.chars().peekable();
    let mut digits = String::new();
    while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
        digits.push(chars.next().unwrap());
    }
    if !digits.is_empty() && matches!(chars.peek(), Some('.') | Some(')')) {
        chars.next();
        let rest = chars.collect::<String>().trim().to_string();
        if !rest.is_empty() {
            return Some(rest);
        }
    }
    let words = trimmed.split_whitespace().count();
    let first = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let imperative = [
        "write", "draft", "send", "make", "create", "fix", "call", "email", "review", "ship",
        "plan", "prepare", "book", "schedule", "update", "ask",
    ];
    if (2..=12).contains(&words) && imperative.contains(&first.as_str()) {
        return Some(trimmed.to_string());
    }
    None
}

fn readiness_for_task(title: &str, note_text: &str) -> f64 {
    let mut score: f64 = 0.45;
    if title.split_whitespace().count() >= 4 {
        score += 0.15;
    }
    if note_text.contains('@') || note_text.contains("http") {
        score += 0.05;
    }
    if note_text.contains('?') {
        score -= 0.15;
    }
    let lower = note_text.to_ascii_lowercase();
    for vague in [
        "maybe", "thing", "stuff", "later", "somehow", "idk", "???", "tbd",
    ] {
        if lower.contains(vague) {
            score -= 0.1;
        }
    }
    score.clamp(0.05, 0.85)
}

fn status_for_readiness(readiness: f64) -> String {
    if readiness < 0.45 {
        "needsClarification".to_string()
    } else {
        "needsReview".to_string()
    }
}

fn placeholder_note_title(title: &str) -> bool {
    let normalized = title.trim().to_ascii_lowercase();
    normalized.is_empty()
        || matches!(
            normalized.as_str(),
            "untitled" | "untitled note" | "new note" | "new note title"
        )
}

fn fallback_task_title(note: &BrainNote) -> Option<String> {
    let title = note.title.trim();
    if title.len() >= 4 && !placeholder_note_title(title) {
        return Some(title.to_string());
    }
    meaningful_note_text(note)
        .split(['.', '!', '?', '\n'])
        .map(str::trim)
        .find(|segment| segment.split_whitespace().count() >= 3)
        .map(|segment| truncate_words(segment, 14))
}

fn extract_task_titles(note: &BrainNote) -> Vec<String> {
    let extracted = sorted_unique(note.body.lines().filter_map(is_task_like_line));
    if extracted.is_empty() {
        fallback_task_title(note).into_iter().collect()
    } else {
        extracted
    }
}

fn ambiguity_for_note(note: &BrainNote, task_count: usize) -> f64 {
    let text = meaningful_note_text(note);
    let mut score: f64 = if text.len() < 80 { 0.65 } else { 0.25 };
    if task_count == 0 {
        score += 0.15;
    }
    if text.contains('?') {
        score += 0.15;
    }
    let lower = text.to_ascii_lowercase();
    for vague in [
        "maybe", "thing", "stuff", "later", "somehow", "idk", "???", "tbd", "scribble",
    ] {
        if lower.contains(vague) {
            score += 0.1;
        }
    }
    score.clamp(0.0, 1.0)
}

fn open_questions_for_note(note: &BrainNote, ambiguity: f64, task_count: usize) -> Vec<String> {
    let mut questions = Vec::new();
    if ambiguity >= 0.5 {
        questions.push("What outcome should Zoid prepare from this note?".to_string());
    }
    if task_count == 0 {
        questions.push("Which concrete task, if any, should become an agent brief?".to_string());
    }
    let text = meaningful_note_text(note).to_ascii_lowercase();
    if text.contains("tbd") || text.contains("later") {
        questions.push("What deadline or timing constraint should be used?".to_string());
    }
    sorted_unique(questions)
}

fn extract_brain_note_in_store(
    store: &mut BrainStore,
    note_id: &str,
    now: &str,
) -> Result<(), String> {
    let note = store
        .notes
        .iter()
        .find(|note| note.id == note_id && !note.archived)
        .cloned()
        .ok_or_else(|| "Brain note not found or archived.".to_string())?;
    if note.sync_status != "synced" {
        return Err(format!(
            "Brain note must be synced before extraction. Resolve or refresh its Apple Notes sync status first: {}.",
            note.sync_status
        ));
    }
    let text = meaningful_note_text(&note);
    let task_titles = extract_task_titles(&note);
    let ambiguity = ambiguity_for_note(&note, task_titles.len());
    let open_questions = open_questions_for_note(&note, ambiguity, task_titles.len());
    store
        .extractions
        .retain(|extraction| extraction.note_id != note_id);
    store.task_candidates.retain(|candidate| {
        candidate.note_id != note_id || matches!(candidate.status.as_str(), "sentToAgent" | "done")
    });
    store.extractions.push(BrainExtraction {
        id: format!(
            "brain-extraction-{}",
            simple_stable_hash(&format!("{}\0{}", note.id, note.current_hash))
        ),
        note_id: note.id.clone(),
        summary: note_summary(&note),
        topics: extract_topics(&text),
        entities: extract_entities(&text),
        references: extract_references(&text),
        decisions: Vec::new(),
        open_questions,
        ambiguity_score: ambiguity,
        extracted_at: now.to_string(),
        extractor: "localHeuristic".to_string(),
    });
    for (index, title) in task_titles.into_iter().enumerate() {
        let readiness = readiness_for_task(&title, &text);
        store.task_candidates.push(TaskCandidate {
            id: format!(
                "task-candidate-{}",
                simple_stable_hash(&format!("{}\0{}\0{}", note.id, index, title))
            ),
            note_id: note.id.clone(),
            title: truncate_words(&title, 100),
            extracted_description: title,
            status: status_for_readiness(readiness),
            priority_guess: if text.to_ascii_lowercase().contains("urgent") {
                "high".to_string()
            } else {
                "normal".to_string()
            },
            readiness_score: readiness,
            clarification_session_id: None,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        });
    }
    store.updated_at = now.to_string();
    Ok(())
}

fn extract_brain_note_inner(note_id: &str) -> Result<BrainStore, String> {
    let mut store = load_brain_store_inner()?;
    let now = now_millis_string();
    extract_brain_note_in_store(&mut store, note_id, &now)?;
    save_brain_store_inner(&store)?;
    Ok(store)
}

fn create_brain_clarifying_session_in_store(
    store: &mut BrainStore,
    note_id: &str,
    task_candidate_ids: Vec<String>,
    now: &str,
) -> Result<(), String> {
    if task_candidate_ids.is_empty() {
        return Err("Select at least one task candidate.".to_string());
    }
    if !store
        .notes
        .iter()
        .any(|note| note.id == note_id && !note.archived)
    {
        return Err("Brain note not found or archived.".to_string());
    }
    let selected: Vec<TaskCandidate> = store
        .task_candidates
        .iter()
        .filter(|candidate| {
            candidate.note_id == note_id && task_candidate_ids.contains(&candidate.id)
        })
        .cloned()
        .collect();
    if selected.len() != task_candidate_ids.len() {
        return Err("Selected task candidates must belong to the same note.".to_string());
    }
    let extraction_questions = store
        .extractions
        .iter()
        .find(|extraction| extraction.note_id == note_id)
        .map(|e| e.open_questions.clone())
        .unwrap_or_default();
    let mut questions = extraction_questions;
    for candidate in &selected {
        if candidate.readiness_score < 0.65 || candidate.status == "needsClarification" {
            questions.push(format!(
                "For '{}', what exact success criteria, owner, and deadline should the agent use?",
                candidate.title
            ));
        }
    }
    if questions.is_empty() {
        questions.push("Confirm the desired outcome, constraints, and deadline before creating the agent brief.".to_string());
    }
    let questions = sorted_unique(questions);
    let session_id = format!(
        "brain-clarification-{}",
        simple_stable_hash(&format!(
            "{}\0{}\0{}",
            note_id,
            task_candidate_ids.join(","),
            now
        ))
    );
    let transcript = vec![BrainClarificationMessage {
        role: "assistant".to_string(),
        content: format!(
            "I can prepare an agent brief, but I will not execute it yet. Please answer:\n- {}",
            questions.join("\n- ")
        ),
        created_at: now.to_string(),
    }];
    for candidate in store
        .task_candidates
        .iter_mut()
        .filter(|candidate| task_candidate_ids.contains(&candidate.id))
    {
        candidate.clarification_session_id = Some(session_id.clone());
        candidate.updated_at = now.to_string();
        if candidate.status == "needsReview" {
            candidate.status = "needsClarification".to_string();
        }
    }
    store
        .clarification_sessions
        .push(BrainClarificationSession {
            id: session_id,
            note_id: note_id.to_string(),
            task_candidate_ids,
            status: "questioning".to_string(),
            transcript,
            resolved_brief: String::new(),
            open_questions: questions,
            hermes_session_id: None,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        });
    store.updated_at = now.to_string();
    Ok(())
}

fn create_brain_clarifying_session_inner(
    note_id: &str,
    task_candidate_ids: Vec<String>,
) -> Result<BrainStore, String> {
    let mut store = load_brain_store_inner()?;
    let now = now_millis_string();
    create_brain_clarifying_session_in_store(&mut store, note_id, task_candidate_ids, &now)?;
    save_brain_store_inner(&store)?;
    Ok(store)
}

fn brain_agent_brief_for_session(
    store: &BrainStore,
    session: &BrainClarificationSession,
) -> String {
    let note = store.notes.iter().find(|note| note.id == session.note_id);
    let note_title = note
        .map(|note| note.title.trim())
        .filter(|title| !title.is_empty())
        .unwrap_or("Untitled note");
    let candidates: Vec<&TaskCandidate> = store
        .task_candidates
        .iter()
        .filter(|candidate| session.task_candidate_ids.contains(&candidate.id))
        .collect();
    let tasks = if candidates.is_empty() {
        "- Review the source note and prepare the requested work.".to_string()
    } else {
        candidates
            .iter()
            .map(|candidate| format!("- {}: {}", candidate.title, candidate.extracted_description))
            .collect::<Vec<String>>()
            .join("\n")
    };
    let answers = session
        .transcript
        .iter()
        .filter(|message| message.role == "user")
        .enumerate()
        .map(|(index, message)| format!("{}. {}", index + 1, message.content.trim()))
        .collect::<Vec<String>>()
        .join("\n");
    let source_context = note
        .map(|note| meaningful_note_text(note))
        .unwrap_or_default()
        .chars()
        .take(1200)
        .collect::<String>();
    format!(
        "# Agent Brief\n\nSource note: {}\n\nTasks\n{}\n\nClarifying answers\n{}\n\nSource context\n{}\n\nExecution rule: Do not run automatically from Brain. Paste this into Hermes only after explicit user approval.",
        note_title,
        tasks,
        if answers.is_empty() { "- No answers captured." } else { answers.as_str() },
        if source_context.trim().is_empty() { "No source context captured." } else { source_context.trim() }
    )
}

fn answer_brain_clarifying_session_in_store(
    store: &mut BrainStore,
    session_id: &str,
    answer: &str,
    now: &str,
) -> Result<(), String> {
    let answer = answer.trim();
    if answer.is_empty() {
        return Err("Answer cannot be empty.".to_string());
    }
    let session_index = store
        .clarification_sessions
        .iter()
        .position(|session| session.id == session_id)
        .ok_or_else(|| "Clarifying session not found.".to_string())?;
    let next_question = store.clarification_sessions[session_index]
        .open_questions
        .first()
        .cloned()
        .ok_or_else(|| "This clarifying session has no open questions.".to_string())?;
    {
        let session = &mut store.clarification_sessions[session_index];
        if matches!(session.status.as_str(), "sentToAgent" | "archived") {
            return Err("This clarifying session is closed.".to_string());
        }
        session.transcript.push(BrainClarificationMessage {
            role: "assistant".to_string(),
            content: next_question,
            created_at: now.to_string(),
        });
        session.transcript.push(BrainClarificationMessage {
            role: "user".to_string(),
            content: answer.to_string(),
            created_at: now.to_string(),
        });
        session.open_questions.remove(0);
        session.updated_at = now.to_string();
        if let Some(question) = session.open_questions.first() {
            session.status = "questioning".to_string();
            session.transcript.push(BrainClarificationMessage {
                role: "assistant".to_string(),
                content: format!("Next question: {question}"),
                created_at: now.to_string(),
            });
        }
    }
    if store.clarification_sessions[session_index]
        .open_questions
        .is_empty()
    {
        let session_snapshot = store.clarification_sessions[session_index].clone();
        let brief = brain_agent_brief_for_session(store, &session_snapshot);
        let session = &mut store.clarification_sessions[session_index];
        session.status = "briefReady".to_string();
        session.resolved_brief = brief;
        session.updated_at = now.to_string();
        for candidate in store
            .task_candidates
            .iter_mut()
            .filter(|candidate| session.task_candidate_ids.contains(&candidate.id))
        {
            candidate.status = "readyForAgent".to_string();
            candidate.updated_at = now.to_string();
        }
    }
    store.updated_at = now.to_string();
    Ok(())
}

fn answer_brain_clarifying_session_inner(
    session_id: &str,
    answer: &str,
) -> Result<BrainStore, String> {
    let mut store = load_brain_store_inner()?;
    let now = now_millis_string();
    answer_brain_clarifying_session_in_store(&mut store, session_id, answer, &now)?;
    save_brain_store_inner(&store)?;
    Ok(store)
}

fn spawn_output_reader<R>(mut reader: R) -> thread::JoinHandle<Result<String, String>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Failed to read command output: {error}"))?;
        Ok(String::from_utf8_lossy(&bytes).trim().to_string())
    })
}

fn join_output_reader(
    handle: thread::JoinHandle<Result<String, String>>,
    stream_name: &str,
) -> Result<String, String> {
    handle
        .join()
        .map_err(|_| format!("Failed to join {stream_name} reader."))?
}

fn capped_partial_output(value: &str) -> String {
    const MAX_PARTIAL_OUTPUT_CHARS: usize = 800;
    if value.chars().count() <= MAX_PARTIAL_OUTPUT_CHARS {
        return value.to_string();
    }
    let clipped = value
        .chars()
        .take(MAX_PARTIAL_OUTPUT_CHARS)
        .collect::<String>();
    format!("{clipped}…")
}

fn timeout_error_with_partial_output(stdout: &str, stderr: &str) -> String {
    let mut message = "Command timed out before returning a response.".to_string();
    if !stderr.is_empty() {
        message.push_str(&format!(
            " Partial stderr: {}",
            capped_partial_output(stderr)
        ));
    }
    if !stdout.is_empty() {
        message.push_str(&format!(
            " Partial stdout: {}",
            capped_partial_output(stdout)
        ));
    }
    message
}

fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<(bool, String, String), String> {
    #[cfg(unix)]
    command.process_group(0);

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start command: {error}"))?;

    let process_group = child.id();
    let stdout_handle = child
        .stdout
        .take()
        .map(spawn_output_reader)
        .ok_or_else(|| "Failed to capture command stdout.".to_string())?;
    let stderr_handle = child
        .stderr
        .take()
        .map(spawn_output_reader)
        .ok_or_else(|| "Failed to capture command stderr.".to_string())?;

    let Some(status) = child
        .wait_timeout(timeout)
        .map_err(|error| format!("Failed while waiting for command: {error}"))?
    else {
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(format!("-{process_group}"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = child.wait_timeout(Duration::from_millis(100));
            let _ = Command::new("kill")
                .arg("-KILL")
                .arg(format!("-{process_group}"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        #[cfg(not(unix))]
        {
            let _ = child.kill();
        }
        let _ = child.wait();
        let stdout = join_output_reader(stdout_handle, "stdout").unwrap_or_default();
        let stderr = join_output_reader(stderr_handle, "stderr").unwrap_or_default();
        return Err(timeout_error_with_partial_output(&stdout, &stderr));
    };

    let stdout = join_output_reader(stdout_handle, "stdout")?;
    let stderr = join_output_reader(stderr_handle, "stderr")?;
    Ok((status.success(), stdout, stderr))
}

fn signal_hermes_process_group(process_group: u32, signal: &str) -> Result<(), String> {
    let status = Command::new("kill")
        .arg(signal)
        .arg(format!("-{process_group}"))
        .status()
        .map_err(|error| format!("Failed to signal Hermes run: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to send {signal} to the active Hermes run."))
    }
}

fn normalize_hermes_run_identifier(value: Option<String>, fallback: &str) -> String {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn hermes_run_key(session_id: &str, run_id: &str) -> String {
    format!("{session_id}\u{1f}{run_id}")
}

fn list_hermes_run_snapshots_inner() -> Vec<HermesRunSnapshot> {
    let registry = hermes_run_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    registry
        .runs
        .values()
        .map(|slot| HermesRunSnapshot {
            session_id: slot.session_id.clone(),
            run_id: slot.run_id.clone(),
            started_at: slot.started_at.clone(),
            status: if slot.cancel_requested { "stopping" } else { "running" }.to_string(),
            pid: slot.active_pid,
        })
        .collect()
}

fn clear_session_hermes_run(session_id: &str, run_id: &str, pid: u32) {
    let key = hermes_run_key(session_id, run_id);
    let mut registry = hermes_run_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let should_remove = registry
        .runs
        .get(&key)
        .map(|slot| slot.active_pid == Some(pid) || slot.starting)
        .unwrap_or(false);
    if should_remove {
        registry.runs.remove(&key);
    }
}

fn run_hermes_command_with_cancel(
    command: &mut Command,
    timeout: Duration,
) -> Result<(bool, String, String), String> {
    run_hermes_command_for_session_with_cancel(
        command,
        timeout,
        DEFAULT_HERMES_SESSION.to_string(),
        "compat-run".to_string(),
        None,
    )
}

fn run_hermes_command_for_session_with_cancel(
    command: &mut Command,
    timeout: Duration,
    session_id: String,
    run_id: String,
    app_handle: Option<tauri::AppHandle>,
) -> Result<(bool, String, String), String> {
    let run_key = hermes_run_key(&session_id, &run_id);
    {
        let mut registry = hermes_run_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if registry
            .runs
            .values()
            .any(|slot| slot.session_id == session_id)
        {
            return Err(
                "Hermes is already responding in this session. Stop the current run before starting another one."
                    .to_string(),
            );
        }
        if registry.runs.len() >= MAX_ACTIVE_HERMES_RUNS {
            return Err("Too many Hermes runs are active. Stop an existing run before starting another one.".to_string());
        }
        registry.runs.insert(
            run_key.clone(),
            HermesRunSlot::starting(session_id.clone(), run_id.clone()),
        );
    }

    #[cfg(unix)]
    command.process_group(0);

    let mut child = match command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            clear_session_hermes_run(&session_id, &run_id, 0);
            return Err(format!("Failed to start Hermes command: {error}"));
        }
    };
    let pid = child.id();
    let process_group = pid;
    let stdout_accumulator = Arc::new(Mutex::new(String::new()));
    let stderr_accumulator = Arc::new(Mutex::new(String::new()));
    let event_sequence = Arc::new(Mutex::new(0_u64));
    emit_agent_run_event(app_handle.as_ref(), AgentRunEvent { event_type: "agent-run-started".to_string(), run_id: run_id.clone(), session_id: session_id.clone(), timestamp: now_millis_string(), sequence: 0, channel: Some("system".to_string()), chunk: None, message: None, exit_code: None });

    let stdout_thread = child.stdout.take().map(|stdout| {
        let app_handle = app_handle.clone();
        let session_id = session_id.clone();
        let run_id = run_id.clone();
        let output = Arc::clone(&stdout_accumulator);
        let sequence = Arc::clone(&event_sequence);
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let chunk = format!("{line}\n");
                if let Ok(mut content) = output.lock() { content.push_str(&chunk); }
                let next_sequence = if let Ok(mut guard) = sequence.lock() { *guard += 1; *guard } else { 0 };
                emit_agent_run_event(app_handle.as_ref(), AgentRunEvent { event_type: "agent-run-output".to_string(), run_id: run_id.clone(), session_id: session_id.clone(), timestamp: now_millis_string(), sequence: next_sequence, channel: Some("stdout".to_string()), chunk: Some(chunk), message: None, exit_code: None });
            }
        })
    });
    let stderr_thread = child.stderr.take().map(|stderr| {
        let app_handle = app_handle.clone();
        let session_id = session_id.clone();
        let run_id = run_id.clone();
        let output = Arc::clone(&stderr_accumulator);
        let sequence = Arc::clone(&event_sequence);
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let chunk = format!("{line}\n");
                if let Ok(mut content) = output.lock() { content.push_str(&chunk); }
                let next_sequence = if let Ok(mut guard) = sequence.lock() { *guard += 1; *guard } else { 0 };
                emit_agent_run_event(app_handle.as_ref(), AgentRunEvent { event_type: "agent-run-output".to_string(), run_id: run_id.clone(), session_id: session_id.clone(), timestamp: now_millis_string(), sequence: next_sequence, channel: Some("stderr".to_string()), chunk: Some(chunk), message: None, exit_code: None });
            }
        })
    });

    let should_deliver_starting_cancel = {
        let mut registry = hermes_run_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let slot = registry
            .runs
            .get_mut(&run_key)
            .ok_or_else(|| "Hermes run registry lost the active run slot.".to_string())?;
        slot.starting = false;
        slot.active_pid = Some(pid);
        slot.active_process_group = Some(process_group);
        slot.cancel_requested
    };

    if should_deliver_starting_cancel {
        let _ = signal_hermes_process_group(process_group, "-INT");
        let mut registry = hermes_run_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(slot) = registry.runs.get_mut(&run_key) {
            if slot.active_pid == Some(pid) {
                slot.signal_delivered = true;
            }
        }
    }

    let started_at = Instant::now();
    let mut cancel_started_at: Option<Instant> = None;
    let poll_interval = Duration::from_millis(100);
    let cancel_grace_period = Duration::from_secs(3);

    let status = loop {
        if let Some(status) = child
            .wait_timeout(poll_interval)
            .map_err(|error| format!("Failed while waiting for Hermes command: {error}"))?
        {
            break status;
        }

        let (cancel_requested, signal_delivered, active_process_group) = {
            let registry = hermes_run_registry()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(slot) = registry.runs.get(&run_key) {
                (
                    slot.active_pid == Some(pid) && slot.cancel_requested,
                    slot.signal_delivered,
                    slot.active_process_group,
                )
            } else {
                (false, false, None)
            }
        };

        if cancel_requested {
            let cancel_started = *cancel_started_at.get_or_insert_with(Instant::now);
            if !signal_delivered {
                if let Some(group) = active_process_group {
                    if signal_hermes_process_group(group, "-INT").is_ok() {
                        let mut registry = hermes_run_registry()
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        if let Some(slot) = registry.runs.get_mut(&run_key) {
                            if slot.active_pid == Some(pid) {
                                slot.signal_delivered = true;
                            }
                        }
                    }
                }
            }
            if cancel_started.elapsed() >= cancel_grace_period {
                if let Some(group) = active_process_group {
                    let _ = signal_hermes_process_group(group, "-KILL");
                }
                let _ = child.kill();
                let _ = child.wait();
                clear_session_hermes_run(&session_id, &run_id, pid);
                return Err("Hermes run was stopped by the user.".to_string());
            }
            continue;
        }

        if started_at.elapsed() >= timeout {
            if let Some(group) = active_process_group {
                let _ = signal_hermes_process_group(group, "-KILL");
            }
            let _ = child.kill();
            let _ = child.wait();
            clear_session_hermes_run(&session_id, &run_id, pid);
            return Err("Hermes command timed out before returning a response.".to_string());
        }
    };

    if let Some(handle) = stdout_thread { let _ = handle.join(); }
    if let Some(handle) = stderr_thread { let _ = handle.join(); }
    let stdout = stdout_accumulator.lock().map(|content| content.trim().to_string()).unwrap_or_default();
    let stderr = stderr_accumulator.lock().map(|content| content.trim().to_string()).unwrap_or_default();
    let was_cancelled = {
        let mut registry = hermes_run_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let was_cancelled = registry
            .runs
            .get(&run_key)
            .map(|slot| slot.active_pid == Some(pid) && slot.cancel_requested)
            .unwrap_or(false);
        registry.runs.remove(&run_key);
        was_cancelled
    };

    if was_cancelled {
        let terminal_sequence = if let Ok(mut guard) = event_sequence.lock() { *guard += 1; *guard } else { 0 };
        emit_agent_run_event(app_handle.as_ref(), AgentRunEvent { event_type: "agent-run-stopped".to_string(), run_id: run_id.clone(), session_id: session_id.clone(), timestamp: now_millis_string(), sequence: terminal_sequence, channel: Some("system".to_string()), chunk: None, message: Some("Hermes run was stopped by the user.".to_string()), exit_code: None });
        return Err("Hermes run was stopped by the user.".to_string());
    }

    let terminal_sequence = if let Ok(mut guard) = event_sequence.lock() { *guard += 1; *guard } else { 0 };
    emit_agent_run_event(app_handle.as_ref(), AgentRunEvent { event_type: "agent-run-completed".to_string(), run_id: run_id.clone(), session_id: session_id.clone(), timestamp: now_millis_string(), sequence: terminal_sequence, channel: Some("system".to_string()), chunk: None, message: None, exit_code: status.code() });
    Ok((
        status.success(),
        stdout,
        stderr,
    ))
}

fn cancel_active_hermes_run_inner() -> Result<bool, String> {
    let maybe_first = {
        let registry = hermes_run_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        registry
            .runs
            .values()
            .next()
            .map(|slot| (slot.session_id.clone(), slot.run_id.clone()))
    };
    let Some((session_id, run_id)) = maybe_first else {
        return Ok(false);
    };
    cancel_hermes_run_inner(Some(session_id), Some(run_id))
}

fn cancel_hermes_run_inner(
    session_id: Option<String>,
    run_id: Option<String>,
) -> Result<bool, String> {
    let session_id = normalize_hermes_run_identifier(session_id, DEFAULT_HERMES_SESSION);
    let run_id = normalize_hermes_run_identifier(run_id, "compat-run");
    let run_key = hermes_run_key(&session_id, &run_id);
    let (pid, process_group, already_delivered) = {
        let mut registry = hermes_run_registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(slot) = registry.runs.get_mut(&run_key) else {
            return Ok(false);
        };
        if slot.starting && slot.active_pid.is_none() {
            slot.cancel_requested = true;
            return Ok(true);
        }
        let Some(pid) = slot.active_pid else {
            return Ok(false);
        };
        let Some(process_group) = slot.active_process_group else {
            return Ok(false);
        };
        slot.cancel_requested = true;
        (pid, process_group, slot.signal_delivered)
    };

    if already_delivered {
        return Ok(true);
    }

    match signal_hermes_process_group(process_group, "-INT") {
        Ok(()) => {
            let mut registry = hermes_run_registry()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(slot) = registry.runs.get_mut(&run_key) {
                if slot.active_pid == Some(pid) {
                    slot.signal_delivered = true;
                }
            }
            Ok(true)
        }
        Err(error) => {
            if Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .status()
                .map(|status| !status.success())
                .unwrap_or(false)
            {
                clear_session_hermes_run(&session_id, &run_id, pid);
                Ok(false)
            } else {
                Err(format!("Failed to send Ctrl+C to Hermes run: {error}"))
            }
        }
    }
}

fn find_hermes_cli() -> Option<(PathBuf, String)> {
    for path in candidate_hermes_paths() {
        let mut command = Command::new(&path);
        command.arg("--version");
        if let Ok((true, stdout, stderr)) =
            run_command_with_timeout(&mut command, Duration::from_secs(8))
        {
            let version = if stdout.is_empty() { stderr } else { stdout };
            return Some((path, version));
        }
    }
    None
}

fn strip_terminal_noise(output: &str) -> String {
    output
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("Session ID:")
                && !trimmed.starts_with("session_id:")
                && !trimmed.starts_with("Cost:")
                && !trimmed.starts_with("Tokens:")
                && !trimmed.starts_with("Provider:")
                && !trimmed.starts_with("Warning:")
                && !trimmed.starts_with("↻")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || matches!(
                character,
                '/' | '.' | '_' | '-' | ':' | '=' | ',' | '@' | '+'
            )
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn command_usage(command_path: &Path, args: &[String], workdir: Option<&Path>) -> String {
    let command = std::iter::once(shell_quote(&command_display(&command_path.to_path_buf())))
        .chain(args.iter().map(|arg| shell_quote(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    match workdir {
        Some(path) => format!("cd {} && {}", shell_quote(&path.to_string_lossy()), command),
        None => command,
    }
}

fn with_terminal_usage(_usage: &str, content: &str) -> String {
    content.trim().to_string()
}

fn split_command_line(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut argument_started = false;

    while let Some(character) = chars.next() {
        if escaped {
            current.push(character);
            escaped = false;
            argument_started = true;
            continue;
        }
        if character == '\\' {
            escaped = true;
            argument_started = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                current.push(character);
            }
            argument_started = true;
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
            argument_started = true;
            continue;
        }
        if character.is_whitespace() {
            if argument_started {
                args.push(current.clone());
                current.clear();
                argument_started = false;
            }
            while chars.peek().is_some_and(|next| next.is_whitespace()) {
                chars.next();
            }
            continue;
        }
        current.push(character);
        argument_started = true;
    }

    if escaped {
        current.push('\\');
    }
    if quote.is_some() {
        return Err("Command contains an unclosed quote.".to_string());
    }
    if argument_started {
        args.push(current);
    }
    Ok(args)
}

fn hermes_cli_args_from_prompt(prompt: &str) -> Result<Option<Vec<String>>, String> {
    let trimmed = prompt.trim();
    let Some(rest) = trimmed.strip_prefix("hermes") else {
        return Ok(None);
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return Ok(None);
    }
    let mut args = split_command_line(rest.trim())?;
    if args
        .iter()
        .any(|arg| arg == "--yolo" || arg.starts_with("--yolo=") || arg == "uninstall")
    {
        return Err("Zoid chat blocks high-risk Hermes CLI invocations. Run that exact command in Terminal if you really intend it.".to_string());
    }
    if args.iter().any(|arg| {
        arg == "gateway" || arg == "model" || arg == "setup" || arg == "tools" || arg == "skills"
    }) && args.len() == 1
    {
        return Err(format!("`hermes {}` is interactive. Use a non-interactive subcommand like `hermes {} list`, or run it directly in Terminal.", args[0], args[0]));
    }
    if args.is_empty() {
        args.push("--help".to_string());
    }
    Ok(Some(args))
}

fn candidate_hermes_source_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(explicit) = env::var("ZOID_HERMES_SOURCE") {
        roots.push(PathBuf::from(explicit));
    }
    if let Ok(home) = env::var("HOME") {
        roots.push(PathBuf::from(format!("{home}/.hermes/hermes-agent")));
    }
    roots.push(PathBuf::from("/Users/ziadnasreldin/.hermes/hermes-agent"));
    roots
}

fn find_hermes_source_root() -> Option<PathBuf> {
    candidate_hermes_source_roots()
        .into_iter()
        .find(|path| path.join("hermes_cli/commands.py").exists())
}

fn classify_zoid_command_behavior(name: &str) -> (String, Option<String>) {
    match name {
        "model" | "reasoning" => ("native-panel".to_string(), Some("model".to_string())),
        "tools" | "toolsets" => ("native-panel".to_string(), Some("tools".to_string())),
        "skills" | "skill" | "reload-skills" => {
            ("native-panel".to_string(), Some("skills".to_string()))
        }
        "cron" => ("native-panel".to_string(), Some("cron".to_string())),
        "agents" | "tasks" | "background" | "queue" | "steer" => {
            ("native-panel".to_string(), Some("agents".to_string()))
        }
        "profile" => ("native-panel".to_string(), Some("profile".to_string())),
        "history" | "resume" | "save" => ("native-panel".to_string(), Some("history".to_string())),
        "usage" | "insights" | "status" => ("native-panel".to_string(), Some("usage".to_string())),
        "debug" => ("native-panel".to_string(), Some("debug".to_string())),
        "browser" => ("native-panel".to_string(), Some("browser".to_string())),
        "redraw" | "skin" | "statusbar" | "indicator" => ("noop".to_string(), None),
        "yolo" | "rollback" | "snapshot" | "stop" | "restart" | "update" | "uninstall" => {
            ("confirm-forward".to_string(), None)
        }
        _ => ("forward".to_string(), None),
    }
}

fn slash_token(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let without_slash = trimmed.strip_prefix('/')?;
    without_slash
        .split_whitespace()
        .next()
        .map(|value| value.to_ascii_lowercase())
}

fn command_requires_confirmation(raw_command: &str, canonical: &str) -> bool {
    let lower = raw_command.to_ascii_lowercase();
    matches!(
        canonical,
        "yolo" | "rollback" | "stop" | "restart" | "update" | "uninstall"
    ) || (canonical == "snapshot" && lower.contains("restore"))
        || (canonical == "cron" && lower.contains("remove"))
        || (canonical == "sessions" && lower.contains("delete"))
        || (canonical == "profile" && lower.contains("delete"))
}

fn parse_hermes_registry_json(raw: &str) -> Result<Vec<HermesSlashCommand>, String> {
    let mut commands: Vec<HermesSlashCommand> = serde_json::from_str(raw)
        .map_err(|error| format!("Failed to parse Hermes command registry JSON: {error}"))?;
    for command in &mut commands {
        let (behavior, panel) = classify_zoid_command_behavior(&command.name);
        command.zoid_behavior = behavior;
        command.panel = panel;
    }
    if !commands.iter().any(|command| command.name == "plan") {
        commands.push(HermesSlashCommand {
            name: "plan".to_string(),
            aliases: vec!["p".to_string()],
            description: "Prepare an implementation plan".to_string(),
            category: "zoid core".to_string(),
            args_hint: Some("<request>".to_string()),
            subcommands: Vec::new(),
            cli_only: false,
            gateway_only: false,
            zoid_behavior: "forward".to_string(),
            panel: None,
        });
    }
    commands.sort_by(|a, b| a.category.cmp(&b.category).then(a.name.cmp(&b.name)));
    Ok(commands)
}

fn candidate_hermes_registry_pythons(root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for relative in ["venv/bin/python", ".venv/bin/python"] {
        let candidate = root.join(relative);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));
    candidates
}

fn load_hermes_slash_commands_inner() -> Result<Vec<HermesSlashCommand>, String> {
    let root = find_hermes_source_root().ok_or_else(|| "Hermes source was not found. Set ZOID_HERMES_SOURCE or install Hermes under ~/.hermes/hermes-agent.".to_string())?;
    let script = r#"
import json
from hermes_cli.commands import COMMAND_REGISTRY
out=[]
for c in COMMAND_REGISTRY:
    out.append({
        "name": c.name,
        "aliases": list(c.aliases or []),
        "description": c.description,
        "category": c.category,
        "argsHint": c.args_hint or None,
        "subcommands": list(c.subcommands or []),
        "cliOnly": bool(c.cli_only),
        "gatewayOnly": bool(c.gateway_only),
        "zoidBehavior": "forward",
        "panel": None,
    })
print(json.dumps(out))
"#;
    for python in candidate_hermes_registry_pythons(&root) {
        let mut command = Command::new(&python);
        command.current_dir(&root).arg("-c").arg(script);
        if let Ok((true, stdout, _stderr)) =
            run_command_with_timeout(&mut command, Duration::from_secs(12))
        {
            return parse_hermes_registry_json(&stdout);
        }
    }
    Err(format!(
        "Failed to import Hermes command registry from {} using its venv python or system python3/python.",
        root.display()
    ))
}

fn resolve_slash_command(
    raw_command: &str,
    commands: &[HermesSlashCommand],
) -> Option<HermesSlashCommand> {
    let token = slash_token(raw_command)?;
    commands
        .iter()
        .find(|command| {
            command.name == token || command.aliases.iter().any(|alias| alias == &token)
        })
        .cloned()
}

fn hermes_slash_execution_result(kind: &str, command: &str) -> HermesSlashExecutionResult {
    HermesSlashExecutionResult {
        kind: kind.to_string(),
        content: None,
        session: None,
        panel: None,
        requires_confirmation: false,
        command: command.to_string(),
        scope: "current-session".to_string(),
    }
}

fn run_hermes_cli(
    path: &Path,
    args: &[String],
    workdir: Option<&Path>,
    timeout: Duration,
) -> Result<(String, String), String> {
    let mut command = Command::new(path);
    if let Some(workdir) = workdir {
        command.current_dir(workdir);
    }
    command.args(args);
    let usage = command_usage(path, args, workdir);
    let (success, stdout, stderr) = run_hermes_command_with_cancel(&mut command, timeout)?;
    if !success {
        let error = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!(
            "Hermes CLI returned an error while running `$ {usage}`: {error}"
        ));
    }
    Ok((stdout, stderr))
}

fn execute_hermes_slash_command_inner(
    raw_command: &str,
    linked_repository: Option<String>,
    hermes_session: Option<String>,
    confirmed: bool,
) -> Result<HermesSlashExecutionResult, String> {
    let trimmed = raw_command.trim();
    if !trimmed.starts_with('/') {
        return Err("Slash command must start with /.".to_string());
    }
    let commands = load_hermes_slash_commands_inner().unwrap_or_default();
    let command_def = resolve_slash_command(trimmed, &commands);
    let canonical = command_def
        .as_ref()
        .map(|command| command.name.clone())
        .or_else(|| slash_token(trimmed))
        .unwrap_or_default();
    let (behavior, panel) = command_def
        .as_ref()
        .map(|command| (command.zoid_behavior.clone(), command.panel.clone()))
        .unwrap_or_else(|| classify_zoid_command_behavior(&canonical));

    if command_requires_confirmation(trimmed, &canonical) && !confirmed {
        let mut result = hermes_slash_execution_result("confirmation", trimmed);
        result.requires_confirmation = true;
        result.scope = if matches!(
            canonical.as_str(),
            "restart" | "tools" | "toolsets" | "cron" | "profile" | "sessions" | "update" | "yolo"
        ) {
            "global-hermes".to_string()
        } else {
            "current-session".to_string()
        };
        result.content = Some(format!("Confirm running {trimmed}."));
        return Ok(result);
    }

    if behavior == "noop" {
        let mut result = hermes_slash_execution_result("text", trimmed);
        result.content = Some("Not needed in Zoid.".to_string());
        return Ok(result);
    }

    if matches!(canonical.as_str(), "new" | "reset" | "clear") {
        let mut result = hermes_slash_execution_result("new-session", trimmed);
        result.content = Some("Started a new Zoid session.".to_string());
        result.session = Some(format!("zoid-session-{}", now_millis_string()));
        return Ok(result);
    }

    if matches!(canonical.as_str(), "quit" | "exit" | "q") {
        let mut result = hermes_slash_execution_result("close-session", trimmed);
        result.content = Some("Closed the active Zoid session.".to_string());
        return Ok(result);
    }

    if behavior == "native-panel" && trimmed.split_whitespace().count() == 1 {
        let mut result = hermes_slash_execution_result("panel", trimmed);
        result.panel = panel;
        result.content = Some(format!(
            "Opened {}.",
            result.panel.clone().unwrap_or_else(|| "panel".to_string())
        ));
        return Ok(result);
    }

    let (path, _) = find_hermes_cli().ok_or_else(|| {
        "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH.".to_string()
    })?;
    let repository_workdir = resolve_linked_repository_workdir(linked_repository)?;

    if canonical == "restart" {
        let before_args = vec!["gateway".to_string(), "status".to_string()];
        let restart_args = vec!["gateway".to_string(), "restart".to_string()];
        let after_args = vec!["gateway".to_string(), "status".to_string()];
        let before = run_hermes_cli(
            &path,
            &before_args,
            repository_workdir.as_deref(),
            Duration::from_secs(30),
        )
        .map(|(out, err)| {
            [out, err]
                .into_iter()
                .filter(|v| !v.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|error| format!("Before status failed: {error}"));
        let restart = run_hermes_cli(
            &path,
            &restart_args,
            repository_workdir.as_deref(),
            Duration::from_secs(90),
        )
        .map(|(out, err)| {
            [out, err]
                .into_iter()
                .filter(|v| !v.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let after = run_hermes_cli(
            &path,
            &after_args,
            repository_workdir.as_deref(),
            Duration::from_secs(30),
        )
        .map(|(out, err)| {
            [out, err]
                .into_iter()
                .filter(|v| !v.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|error| format!("After status failed: {error}"));
        let mut result = hermes_slash_execution_result("text", trimmed);
        result.scope = "global-hermes".to_string();
        result.content = Some(format!("Gateway status before:\n{before}\n\nRestart result:\n{restart}\n\nGateway status after:\n{after}"));
        return Ok(result);
    }

    let args = hermes_chat_args(trimmed, hermes_session.as_deref());
    let usage = command_usage(&path, &args, repository_workdir.as_deref());
    let (stdout, stderr) = run_hermes_cli(
        &path,
        &args,
        repository_workdir.as_deref(),
        Duration::from_secs(HERMES_TIMEOUT_SECONDS),
    )?;
    let combined_output = [stdout.as_str(), stderr.as_str()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let content = strip_terminal_noise(&combined_output);
    let mut result = hermes_slash_execution_result("text", trimmed);
    result.session = Some(parse_hermes_session_id(
        &combined_output,
        hermes_session.as_deref().unwrap_or(&hermes_session_name()),
    ));
    result.content = Some(with_terminal_usage(
        &usage,
        if content.is_empty() {
            "Command completed."
        } else {
            &content
        },
    ));
    if matches!(
        canonical.as_str(),
        "tools"
            | "toolsets"
            | "cron"
            | "profile"
            | "restart"
            | "update"
            | "skills"
            | "skill"
            | "reload-skills"
            | "reload-mcp"
    ) {
        result.scope = "global-hermes".to_string();
    }
    Ok(result)
}

fn parse_hermes_session_id(output: &str, fallback: &str) -> String {
    output
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();
            lower
                .strip_prefix("session_id:")
                .or_else(|| lower.strip_prefix("session id:"))
                .map(|prefix| trimmed[trimmed.len() - prefix.len()..].trim())
        })
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn hermes_chat_args(prompt: &str, hermes_session: Option<&str>) -> Vec<String> {
    let mut args = vec!["chat".to_string(), "--cli".to_string()];
    if let Some(session) = hermes_session
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        args.push("--resume".to_string());
        args.push(session.to_string());
    }
    args.extend([
        "--quiet".to_string(),
        "--source".to_string(),
        "desktop".to_string(),
        "--query".to_string(),
        prompt.to_string(),
    ]);
    args
}

fn apply_profile_runtime_args_from_settings(
    args: &mut Vec<String>,
    settings: &HermesProfileSettings,
) {
    let Some(query_index) = args.iter().position(|arg| arg == "--query") else {
        return;
    };
    let mut runtime_args = Vec::new();
    if !settings.model_provider.trim().is_empty() {
        runtime_args.push("--provider".to_string());
        runtime_args.push(settings.model_provider.clone());
    }
    if !settings.model_name.trim().is_empty() {
        runtime_args.push("--model".to_string());
        runtime_args.push(settings.model_name.clone());
    }
    let enabled_toolsets =
        toolsets_from_feature_toggles(settings, lines_to_sorted_vec(&settings.toolsets));
    if settings.access_mode == "safe" {
        runtime_args.push("--toolsets".to_string());
        runtime_args.push(safe_runtime_toolsets().join(","));
    } else if settings.access_mode == "workspace" && !enabled_toolsets.is_empty() {
        runtime_args.push("--toolsets".to_string());
        runtime_args.push(enabled_toolsets.join(","));
    }
    if !runtime_args.is_empty() {
        args.splice(query_index..query_index, runtime_args);
    }
}

#[cfg(test)]
fn apply_profile_runtime_args(args: &mut Vec<String>) -> Result<(), String> {
    let settings = load_hermes_profile_settings_inner()?;
    apply_profile_runtime_args_from_settings(args, &settings);
    Ok(())
}

fn enabled_profile_prompt_context(settings: &HermesProfileSettings) -> String {
    let mut sections: Vec<String> = Vec::new();
    if settings.user_profile_enabled {
        let mut profile_lines = Vec::new();
        for (label, value) in [
            ("Name", settings.user_name.as_str()),
            ("Role", settings.role.as_str()),
            ("Timezone", settings.timezone.as_str()),
            ("Communication style", settings.communication_style.as_str()),
            ("Response mode", settings.response_mode.as_str()),
            ("Personality preset", settings.personality_preset.as_str()),
            ("Preferences", settings.preferences.as_str()),
        ] {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                profile_lines.push(format!("- {label}: {trimmed}"));
            }
        }
        if !profile_lines.is_empty() {
            sections.push(format!(
                "[Zoid enabled user profile]\n{}",
                profile_lines.join("\n")
            ));
        }
    }
    if settings.memory_enabled {
        let memory = settings.hermes_memory.trim();
        if !memory.is_empty() {
            sections.push(format!("[Zoid enabled Hermes memory]\n{memory}"));
        }
        let soul = settings.hermes_soul.trim();
        if !soul.is_empty() {
            sections.push(format!("[Zoid enabled Hermes soul]\n{soul}"));
        }
    }
    sections.join("\n\n")
}

fn prompt_with_enabled_profile_context_from_settings(
    prompt: &str,
    settings: &HermesProfileSettings,
) -> String {
    let context = enabled_profile_prompt_context(settings);
    if context.is_empty() {
        prompt.to_string()
    } else {
        format!("{context}\n\n[User message]\n{prompt}")
    }
}

fn build_profiled_hermes_chat_args(
    prompt: &str,
    hermes_session: Option<&str>,
) -> Result<Vec<String>, String> {
    let settings = load_hermes_profile_settings_inner().map_err(|error| {
        format!("Failed to load Hermes profile settings before launching Hermes: {error}")
    })?;
    let prompt = prompt_with_enabled_profile_context_from_settings(prompt, &settings);
    let mut args = hermes_chat_args(&prompt, hermes_session);
    apply_profile_runtime_args_from_settings(&mut args, &settings);
    Ok(args)
}

fn hermes_invocation_args(cli_args: Vec<String>) -> Vec<String> {
    cli_args
}

fn expand_home_path(path: &str) -> Result<PathBuf, String> {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = env::var("HOME").map_err(|_| "HOME is not available.".to_string())?;
        return Ok(PathBuf::from(home).join(rest));
    }
    if path == "~" {
        let home = env::var("HOME").map_err(|_| "HOME is not available.".to_string())?;
        return Ok(PathBuf::from(home));
    }
    Ok(PathBuf::from(path))
}

fn file_permission_bootstrap_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("zoid-file-permissions.json"))
}

fn load_file_permission_marker(path: &Path) -> FilePermissionMarker {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<FilePermissionMarker>(&content).ok())
        .unwrap_or_default()
}

fn persist_file_permission_marker(
    marker_path: &Path,
    touched_paths: &HashSet<String>,
    remembered_paths: &HashSet<String>,
) -> Result<(), String> {
    if let Some(parent) = marker_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create permissions marker directory: {error}"))?;
    }

    let mut sorted_paths = touched_paths.iter().cloned().collect::<Vec<_>>();
    sorted_paths.sort();
    let mut sorted_remembered_paths = remembered_paths.iter().cloned().collect::<Vec<_>>();
    sorted_remembered_paths.sort();
    let serialized = FilePermissionMarker {
        updated_at: now_millis_string(),
        touched_paths: sorted_paths,
        remembered_paths: sorted_remembered_paths,
    };

    fs::write(
        marker_path,
        serde_json::to_string_pretty(&serialized)
            .map_err(|error| format!("Failed to serialize permissions marker: {error}"))?,
    )
    .map_err(|error| format!("Failed to save permissions marker: {error}"))
}

fn touch_file_permission_path(path: &Path) -> Result<String, String> {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let access_result = if resolved.is_dir() {
        fs::read_dir(&resolved)
            .map(|mut entries| {
                let _ = entries.next();
            })
            .map_err(|error| error.to_string())
    } else {
        fs::metadata(&resolved)
            .map(|_| ())
            .map_err(|error| error.to_string())
    };

    access_result
        .map_err(|error| format!("Zoid could not access {}: {error}", resolved.display()))?;
    Ok(resolved.to_string_lossy().to_string())
}

fn file_permission_path_keys(path: &Path) -> Vec<String> {
    let mut keys = vec![path.to_string_lossy().to_string()];
    if let Ok(canonical) = path.canonicalize() {
        let canonical = canonical.to_string_lossy().to_string();
        if !keys.contains(&canonical) {
            keys.push(canonical);
        }
    }
    keys
}

fn raw_path_is_covered_by_touched_root(path: &Path, touched_paths: &HashSet<String>) -> bool {
    if touched_paths.is_empty() {
        return false;
    }

    let path_candidates = file_permission_path_keys(path);

    touched_paths.iter().any(|root_key| {
        if root_key.trim().is_empty() || root_key == "/" {
            return false;
        }
        let root = Path::new(root_key);
        path == root
            || path.starts_with(root)
            || path_candidates.iter().any(|candidate| {
                let candidate = Path::new(candidate);
                candidate == root || candidate.starts_with(root)
            })
    })
}

fn path_is_covered_by_touched_root(path: &Path, touched_paths: &HashSet<String>) -> bool {
    if raw_path_is_covered_by_touched_root(path, touched_paths) {
        return true;
    }

    file_permission_path_keys(path).iter().any(|path_key| {
        let candidate = Path::new(path_key);
        raw_path_is_covered_by_touched_root(candidate, touched_paths)
    })
}

fn remember_file_permission_path(path: &Path) -> Result<(), String> {
    let marker_path = file_permission_bootstrap_path()?;
    let marker = load_file_permission_marker(&marker_path);
    let mut touched_paths = marker.touched_paths.into_iter().collect::<HashSet<_>>();
    let remembered_paths = marker.remembered_paths.into_iter().collect::<HashSet<_>>();
    if raw_path_is_covered_by_touched_root(path, &touched_paths) {
        return Ok(());
    }
    let path_keys = file_permission_path_keys(path);
    if path_keys
        .iter()
        .any(|path_key| touched_paths.contains(path_key))
        || path_is_covered_by_touched_root(path, &touched_paths)
    {
        return Ok(());
    }

    let touched_path = touch_file_permission_path(path)?;
    let mut changed = touched_paths.insert(touched_path);
    for path_key in path_keys {
        changed = touched_paths.insert(path_key) || changed;
    }
    if changed {
        persist_file_permission_marker(&marker_path, &touched_paths, &remembered_paths)?;
    }
    Ok(())
}

fn is_permission_marker_persistence_error(error: &str) -> bool {
    error.starts_with("Failed to create permissions marker directory:")
        || error.starts_with("Failed to save permissions marker:")
}

fn remember_file_permission_path_best_effort(path: &Path) -> Result<(), String> {
    match remember_file_permission_path(path) {
        Ok(()) => Ok(()),
        Err(error) if is_permission_marker_persistence_error(&error) => {
            touch_file_permission_path(path).map(|_| ())
        }
        Err(error) => Err(error),
    }
}

fn remember_file_permission_path_without_touch_best_effort(path: &Path) -> Result<(), String> {
    match remember_file_permission_path_without_touch(path) {
        Ok(()) => Ok(()),
        Err(error) if is_permission_marker_persistence_error(&error) => Ok(()),
        Err(error) => Err(error),
    }
}

fn remember_file_permission_path_without_touch(path: &Path) -> Result<(), String> {
    let marker_path = file_permission_bootstrap_path()?;
    let marker = load_file_permission_marker(&marker_path);
    let touched_paths = marker.touched_paths.into_iter().collect::<HashSet<_>>();
    let mut remembered_paths = marker.remembered_paths.into_iter().collect::<HashSet<_>>();
    let path_key = path.to_string_lossy().to_string();
    if path_key.trim().is_empty()
        || touched_paths.contains(&path_key)
        || remembered_paths.contains(&path_key)
    {
        return Ok(());
    }
    remembered_paths.insert(path_key);
    persist_file_permission_marker(&marker_path, &touched_paths, &remembered_paths)?;
    Ok(())
}

fn warm_file_permissions_inner(force: bool) -> Result<Vec<String>, String> {
    let marker_path = file_permission_bootstrap_path()?;
    let marker = load_file_permission_marker(&marker_path);
    let remembered_paths = marker.remembered_paths.into_iter().collect::<HashSet<_>>();
    let mut touched_paths = marker.touched_paths.into_iter().collect::<HashSet<_>>();

    let mut touched = Vec::new();
    let mut targets = vec![
        "~".to_string(),
        "~/Documents".to_string(),
        "~/Desktop".to_string(),
        "~/Downloads".to_string(),
        "~/.hermes".to_string(),
        "~/.config".to_string(),
        "~/.cache".to_string(),
        "~/.local".to_string(),
        "~/Library/Application Support".to_string(),
        "~/Library/Caches".to_string(),
        "~/Library/Preferences".to_string(),
        "~/Zoid".to_string(),
    ];

    if let Ok(settings) = load_hermes_profile_settings_inner() {
        targets.push(settings.default_workdir);
        targets.extend(settings.trusted_projects.lines().map(str::to_string));
    }

    let mut seen = HashSet::new();
    for target in targets {
        let trimmed = target.trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_string()) {
            continue;
        }
        let Ok(path) = expand_home_path(trimmed) else {
            continue;
        };
        if !path.exists() {
            continue;
        }
        if !force && raw_path_is_covered_by_touched_root(&path, &touched_paths) {
            continue;
        }

        let path_key = path
            .canonicalize()
            .unwrap_or_else(|_| path.clone())
            .to_string_lossy()
            .to_string();
        if !force
            && (touched_paths.contains(&path_key)
                || path_is_covered_by_touched_root(&path, &touched_paths))
        {
            continue;
        }
        if let Ok(touched_path) = touch_file_permission_path(&path) {
            touched_paths.insert(touched_path.clone());
            touched.push(touched_path);
        }
    }

    persist_file_permission_marker(&marker_path, &touched_paths, &remembered_paths)?;

    Ok(touched)
}

fn resolve_linked_repository_workdir(
    linked_repository: Option<String>,
) -> Result<Option<PathBuf>, String> {
    let Some(repository) = linked_repository.map(|value| value.trim().to_string()) else {
        return Ok(None);
    };

    if repository.is_empty() || repository == "Unlinked" {
        return Ok(None);
    }

    let path = PathBuf::from(repository);
    if !path.exists() {
        return Err(format!(
            "Linked repository does not exist: {}",
            path.display()
        ));
    }
    if !path.is_dir() {
        return Err(format!(
            "Linked repository must be a directory: {}",
            path.display()
        ));
    }

    Ok(Some(path))
}

fn is_git_repository(path: &Path) -> bool {
    let git_path = path.join(".git");
    git_path.is_dir() || git_path.is_file()
}

fn repository_id(path: &Path) -> String {
    let stable_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    repository_id_from_raw_path(&stable_path.to_string_lossy())
}

fn repository_id_from_raw_path(raw_path: &str) -> String {
    let encoded = raw_path
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    format!("repo-{encoded}")
}

fn now_millis_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

fn email_header_value(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut truncated: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        truncated.push('…');
    }
    truncated
}

fn bounded_email_header(value: &str, max_chars: usize) -> String {
    truncate_chars(&email_header_value(value), max_chars)
}

fn bounded_email_body(value: &str, max_chars: usize) -> String {
    truncate_chars(value.trim(), max_chars)
}

fn send_agent_response_email_notification_inner(
    request: AgentResponseEmailNotificationRequest,
) -> Result<AgentResponseEmailNotificationResult, String> {
    let host = env::var("ZOID_NOTIFY_SMTP_HOST").map_err(|_| {
        "Email notifications are not configured: set ZOID_NOTIFY_SMTP_HOST.".to_string()
    })?;
    let port = env::var("ZOID_NOTIFY_SMTP_PORT").unwrap_or_else(|_| "587".to_string());
    let username = env::var("ZOID_NOTIFY_SMTP_USERNAME").map_err(|_| {
        "Email notifications are not configured: set ZOID_NOTIFY_SMTP_USERNAME.".to_string()
    })?;
    let password = env::var("ZOID_NOTIFY_SMTP_PASSWORD").map_err(|_| {
        "Email notifications are not configured: set ZOID_NOTIFY_SMTP_PASSWORD.".to_string()
    })?;
    let from = env::var("ZOID_NOTIFY_EMAIL_FROM").unwrap_or_else(|_| username.clone());
    let to = request
        .to
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env::var("ZOID_NOTIFY_EMAIL_TO").ok())
        .unwrap_or_else(|| "ziad.ahmed.25.25.25@gmail.com".to_string());
    let subject = bounded_email_header(&request.subject, 160);
    let session_title = bounded_email_header(&request.session_title, 160);
    let summary = bounded_email_body(&request.summary, 16 * 1024);
    let body = format!("Session: {session_title}\n\n{}\n", summary);

    let python_script = r#"
import os
import smtplib
import sys
from email.message import EmailMessage

message = sys.stdin.read()
email = EmailMessage()
email["From"] = os.environ["ZOID_NOTIFY_EMAIL_FROM_RUNTIME"]
email["To"] = os.environ["ZOID_NOTIFY_EMAIL_TO_RUNTIME"]
email["Subject"] = os.environ["ZOID_NOTIFY_EMAIL_SUBJECT_RUNTIME"]
email.set_content(message)

host = os.environ["ZOID_NOTIFY_SMTP_HOST"]
port = int(os.environ.get("ZOID_NOTIFY_SMTP_PORT", "587"))
username = os.environ["ZOID_NOTIFY_SMTP_USERNAME"]
password = os.environ["ZOID_NOTIFY_SMTP_PASSWORD"]

with smtplib.SMTP(host, port, timeout=35) as smtp:
    smtp.starttls()
    smtp.login(username, password)
    smtp.send_message(email)
"#;

    let mut command = Command::new("python3");
    command
        .arg("-c")
        .arg(python_script)
        .env("ZOID_NOTIFY_SMTP_HOST", host.trim())
        .env("ZOID_NOTIFY_SMTP_PORT", port.trim())
        .env("ZOID_NOTIFY_SMTP_USERNAME", username.trim())
        .env("ZOID_NOTIFY_SMTP_PASSWORD", password)
        .env("ZOID_NOTIFY_EMAIL_FROM_RUNTIME", email_header_value(&from))
        .env("ZOID_NOTIFY_EMAIL_TO_RUNTIME", email_header_value(&to))
        .env("ZOID_NOTIFY_EMAIL_SUBJECT_RUNTIME", subject)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Failed to start SMTP email sender: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(body.as_bytes())
            .map_err(|error| format!("Failed to write SMTP email payload: {error}"))?;
    }
    let Some(_status) = child
        .wait_timeout(Duration::from_secs(45))
        .map_err(|error| format!("Failed while waiting for SMTP email sender: {error}"))?
    else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("SMTP email sender timed out.".to_string());
    };
    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to read SMTP email sender output: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("Email notification failed: {stderr}"));
    }

    Ok(AgentResponseEmailNotificationResult {
        ok: true,
        message: format!("Sent agent response email notification to {to}."),
        sent_at: now_millis_string(),
    })
}

fn run_git(path: &Path, args: &[&str]) -> Option<String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(path).args(args);
    match run_command_with_timeout(&mut command, Duration::from_secs(GIT_TIMEOUT_SECONDS)) {
        Ok((true, stdout, _)) if !stdout.trim().is_empty() => Some(stdout.trim().to_string()),
        _ => None,
    }
}

fn read_latest_commit(path: &Path) -> Option<LatestCommit> {
    let output = run_git(
        path,
        &[
            "log",
            "-1",
            "--date=short",
            "--pretty=format:%h%x00%s%x00%cd",
        ],
    )?;
    let mut parts = output.splitn(3, '\0');
    let hash = parts.next()?.trim().to_string();
    let message = parts.next().unwrap_or("").trim().to_string();
    let date = parts.next().unwrap_or("").trim().to_string();
    if hash.is_empty() || date.is_empty() {
        None
    } else {
        Some(LatestCommit {
            hash,
            message,
            date,
        })
    }
}

fn normalize_default_branch(branch: Option<String>) -> Option<String> {
    branch
        .map(|value| value.trim().trim_start_matches("origin/").to_string())
        .filter(|value| !value.is_empty())
}

fn read_default_branch(path: &Path) -> Option<String> {
    normalize_default_branch(run_git(
        path,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ))
    .or_else(|| {
        run_git(path, &["remote", "show", "origin"]).and_then(|output| {
            output.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("HEAD branch:")
                    .map(|branch| branch.trim().to_string())
                    .filter(|branch| !branch.is_empty() && branch != "(unknown)")
            })
        })
    })
    .or_else(|| run_git(path, &["branch", "--show-current"]))
}

fn read_repository_details(path: &Path, source: &str) -> Result<CodeRepository, String> {
    remember_file_permission_path_best_effort(path)?;
    if !path.exists() || !path.is_dir() {
        return Err(format!(
            "Repository path must be an existing directory: {}",
            path.display()
        ));
    }
    if !is_git_repository(path) {
        return Err(format!(
            "Folder is not a Git repository: {}",
            path.display()
        ));
    }

    let canonical_path = path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve repository path: {error}"))?;
    let name = canonical_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| canonical_path.display().to_string());
    let status = run_git(&canonical_path, &["status", "--porcelain"]).unwrap_or_default();

    Ok(CodeRepository {
        id: repository_id(&canonical_path),
        name,
        path: canonical_path.to_string_lossy().to_string(),
        remote_url: run_git(&canonical_path, &["remote", "get-url", "origin"]),
        branch: run_git(&canonical_path, &["branch", "--show-current"]),
        default_branch: read_default_branch(&canonical_path),
        dirty: !status.trim().is_empty(),
        latest_commit: read_latest_commit(&canonical_path),
        added_at: now_millis_string(),
        source: source.to_string(),
    })
}

fn default_file_manager_root() -> Result<PathBuf, String> {
    env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "Cannot resolve macOS home folder for file manager.".to_string())
}

fn format_file_manager_time(time: SystemTime) -> Option<String> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs().to_string())
}

fn file_manager_kind(_path: &Path, metadata: &fs::Metadata) -> String {
    if metadata.file_type().is_symlink() {
        "symlink".to_string()
    } else if metadata.is_dir() {
        "directory".to_string()
    } else if metadata.is_file() {
        "file".to_string()
    } else {
        "other".to_string()
    }
}

fn path_is_protected_macos_user_folder(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    matches!(
        name,
        "Desktop" | "Documents" | "Downloads" | "Library" | "Movies" | "Music" | "Pictures"
    )
}

fn count_visible_children(path: &Path) -> Option<usize> {
    if path_is_protected_macos_user_folder(path) {
        return None;
    }

    let entries = fs::read_dir(path).ok()?;
    Some(
        entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| !name.starts_with('.'))
            })
            .take(501)
            .count(),
    )
}

fn list_file_manager_directory_inner(
    path: Option<String>,
) -> Result<FileManagerDirectoryListing, String> {
    let requested_path = path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| PathBuf::from(value.trim()))
        .unwrap_or(default_file_manager_root()?);
    remember_file_permission_path_best_effort(&requested_path)?;

    let directory_path = requested_path
        .canonicalize()
        .map_err(|error| format!("Cannot open {}: {error}", requested_path.display()))?;
    if !directory_path.is_dir() {
        return Err(format!("{} is not a folder.", directory_path.display()));
    }

    let read_dir = fs::read_dir(&directory_path)
        .map_err(|error| format!("Cannot read {}: {error}", directory_path.display()))?;

    let mut entries = Vec::new();
    for entry in read_dir.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let kind = file_manager_kind(&path, &metadata);
        let children_count = if metadata.is_dir() {
            count_visible_children(&path)
        } else {
            None
        };
        entries.push(FileManagerEntry {
            name,
            path: path.to_string_lossy().to_string(),
            kind,
            size: metadata.is_file().then_some(metadata.len()),
            modified: metadata.modified().ok().and_then(format_file_manager_time),
            hidden: entry.file_name().to_string_lossy().starts_with('.'),
            readonly: metadata.permissions().readonly(),
            children_count,
        });
    }

    entries.sort_by(|left, right| {
        let left_dir = left.kind == "directory";
        let right_dir = right.kind == "directory";
        right_dir
            .cmp(&left_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    entries.truncate(500);

    Ok(FileManagerDirectoryListing {
        name: directory_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_else(|| directory_path.to_str().unwrap_or("Macintosh HD"))
            .to_string(),
        parent: directory_path
            .parent()
            .map(|value| value.to_string_lossy().to_string()),
        path: directory_path.to_string_lossy().to_string(),
        entries,
    })
}

fn should_skip_scan_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|name| IGNORED_SCAN_DIRS.contains(&name))
        .unwrap_or(false)
}

fn scan_repositories_recursive(
    path: &Path,
    depth: usize,
    seen: &mut HashSet<PathBuf>,
    repositories: &mut Vec<CodeRepository>,
) {
    if should_skip_scan_dir(path) || depth > REPOSITORY_SCAN_MAX_DEPTH {
        return;
    }

    if is_git_repository(path) {
        if let Ok(canonical_path) = path.canonicalize() {
            if seen.insert(canonical_path.clone()) {
                if let Ok(repository) = read_repository_details(&canonical_path, "scanned") {
                    repositories.push(repository);
                }
            }
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let child = entry.path();
        if child.is_dir() {
            scan_repositories_recursive(&child, depth + 1, seen, repositories);
        }
    }
}

fn scan_repository_folder(folder: &str) -> Result<Vec<CodeRepository>, String> {
    let root = PathBuf::from(folder.trim());
    remember_file_permission_path_best_effort(&root)?;
    if !root.exists() {
        return Err(format!("Scan folder does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!(
            "Scan folder must be a directory: {}",
            root.display()
        ));
    }

    let mut seen = HashSet::new();
    let mut repositories = Vec::new();
    scan_repositories_recursive(&root, 0, &mut seen, &mut repositories);
    repositories.sort_by(|left, right| left.name.cmp(&right.name).then(left.path.cmp(&right.path)));
    Ok(repositories)
}

fn github_repo_folder_name(repo_url: &str) -> Result<String, String> {
    let trimmed = repo_url.trim().trim_end_matches('/');
    let slug = if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("http://github.com/") {
        rest
    } else {
        return Err("Only GitHub repository links are supported.".to_string());
    };

    let parts = slug.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("GitHub repository link must look like github.com/org/repo.".to_string());
    }
    let repo_name = parts[1].trim_end_matches(".git");
    if repo_name.is_empty() || repo_name.contains(':') || repo_name.contains(' ') {
        return Err("GitHub repository name is invalid.".to_string());
    }
    Ok(repo_name.to_string())
}

fn github_repo_slug_from_remote(remote_url: &str) -> Result<String, String> {
    let trimmed = remote_url
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let slug = if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("http://github.com/") {
        rest
    } else {
        return Err("Default branch editing requires an origin remote on GitHub.".to_string());
    };

    let parts = slug.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("GitHub origin remote must look like github.com/org/repo.".to_string());
    }
    Ok(format!("{}/{}", parts[0], parts[1]))
}

fn github_branch_slug_from_remote_url(remote_url: &str) -> Result<String, String> {
    github_repo_slug_from_remote(remote_url)
}

fn has_remote_url(remote_url: &Option<String>) -> bool {
    remote_url
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

fn update_default_branch(
    repository_path: &str,
    remote_url: Option<String>,
    default_branch: &str,
) -> Result<CodeRepository, String> {
    let requested_branch = default_branch.trim();
    if requested_branch.is_empty()
        || requested_branch.contains(' ')
        || requested_branch.starts_with('-')
    {
        return Err("Default branch must be a non-empty Git branch name.".to_string());
    }

    let repository_path = repository_path.trim();
    if !repository_path.is_empty() {
        let repository_path_buf = PathBuf::from(repository_path);
        if has_remote_url(&remote_url) {
            remember_file_permission_path_without_touch_best_effort(&repository_path_buf)?;
        } else {
            remember_file_permission_path_best_effort(&repository_path_buf)?;
        }
    }
    let repository_remote_url = remote_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            if repository_path.is_empty() {
                None
            } else {
                run_git(
                    &PathBuf::from(repository_path),
                    &["remote", "get-url", "origin"],
                )
            }
        })
        .ok_or_else(|| "Default branch editing requires a GitHub origin remote.".to_string())?;
    let slug = github_branch_slug_from_remote_url(&repository_remote_url)?;

    let mut command = Command::new("gh");
    command.args(["repo", "edit", &slug, "--default-branch", requested_branch]);
    let (success, stdout, stderr) =
        run_command_with_timeout(&mut command, Duration::from_secs(GIT_TIMEOUT_SECONDS))?;
    if !success {
        let error = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("GitHub default branch update failed: {error}"));
    }

    if !repository_path.is_empty() {
        let path = PathBuf::from(repository_path);
        if let Some(mut refreshed) = (!remote_url
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()))
        .then(|| read_repository_details(&path, "scanned").ok())
        .flatten()
        {
            let _ = run_git(&path, &["remote", "set-head", "origin", "-a"]);
            refreshed.default_branch = Some(requested_branch.to_string());
            return Ok(refreshed);
        }
    }

    let name = slug.rsplit('/').next().unwrap_or(&slug).to_string();
    Ok(CodeRepository {
        id: repository_id_from_raw_path(repository_path),
        name,
        path: repository_path.to_string(),
        remote_url: Some(repository_remote_url),
        branch: None,
        default_branch: Some(requested_branch.to_string()),
        dirty: false,
        latest_commit: None,
        added_at: now_millis_string(),
        source: "github".to_string(),
    })
}

fn list_remote_branches(
    repository_path: &str,
    remote_url: Option<String>,
    current_default_branch: Option<String>,
) -> Result<Vec<GithubBranch>, String> {
    let repository_path = repository_path.trim();
    if !repository_path.is_empty() {
        let repository_path_buf = PathBuf::from(repository_path);
        if has_remote_url(&remote_url) {
            remember_file_permission_path_without_touch_best_effort(&repository_path_buf)?;
        } else {
            remember_file_permission_path_best_effort(&repository_path_buf)?;
        }
    }
    let repository_remote_url = remote_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            if repository_path.is_empty() {
                None
            } else {
                run_git(
                    &PathBuf::from(repository_path),
                    &["remote", "get-url", "origin"],
                )
            }
        })
        .ok_or_else(|| "Default branch selection requires a GitHub origin remote.".to_string())?;
    let slug = github_branch_slug_from_remote_url(&repository_remote_url)?;

    let api_path = format!("repos/{slug}/branches");
    let mut command = Command::new("gh");
    command.args(["api", api_path.as_str(), "--paginate", "--jq", ".[].name"]);
    let (success, stdout, stderr) =
        run_command_with_timeout(&mut command, Duration::from_secs(GIT_TIMEOUT_SECONDS))?;
    if !success {
        let error = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("GitHub branch lookup failed: {error}"));
    }

    let default_branch = current_default_branch
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let mut seen = HashSet::new();
    let mut branches = stdout
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .filter(|name| seen.insert((*name).to_string()))
        .map(|name| GithubBranch {
            name: name.to_string(),
            is_default: name == default_branch,
        })
        .collect::<Vec<_>>();

    if branches.is_empty() {
        return Err("GitHub branch lookup returned no branches.".to_string());
    }
    branches.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then(left.name.cmp(&right.name))
    });
    Ok(branches)
}

fn clone_repository(repo_url: &str, destination_root: &str) -> Result<CodeRepository, String> {
    let repo_url = repo_url.trim();
    let repo_name = github_repo_folder_name(repo_url)?;
    let root = PathBuf::from(destination_root.trim());
    fs::create_dir_all(&root)
        .map_err(|error| format!("Failed to create destination root: {error}"))?;
    remember_file_permission_path(&root)?;
    if !root.is_dir() {
        return Err(format!(
            "Destination root must be a directory: {}",
            root.display()
        ));
    }

    let destination = root.join(repo_name);
    if destination.exists()
        && fs::read_dir(&destination)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(true)
    {
        return Err(format!(
            "Destination folder already exists and is not empty: {}",
            destination.display()
        ));
    }

    let mut command = Command::new("git");
    command.arg("clone").arg(repo_url).arg(&destination);
    let (success, stdout, stderr) =
        run_command_with_timeout(&mut command, Duration::from_secs(GIT_TIMEOUT_SECONDS))?;
    if !success {
        let error = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("git clone failed: {error}"));
    }

    read_repository_details(&destination, "cloned")
}

fn validate_hermes_profile_name(profile: &str) -> Result<String, String> {
    let trimmed = profile.trim();
    if trimmed.is_empty() {
        return Err("HERMES_PROFILE cannot be empty.".to_string());
    }
    if trimmed == "." || trimmed == ".." || trimmed.contains('/') || trimmed.contains('\\') {
        return Err("HERMES_PROFILE must be a simple profile name, not a path.".to_string());
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(
            "HERMES_PROFILE may only contain letters, numbers, underscore, and hyphen.".to_string(),
        );
    }
    Ok(trimmed.to_string())
}

fn active_hermes_profile_checked() -> Result<String, String> {
    validate_hermes_profile_name(
        &env::var("HERMES_PROFILE").unwrap_or_else(|_| "default".to_string()),
    )
}

fn active_hermes_profile() -> String {
    active_hermes_profile_checked().unwrap_or_else(|_| "default".to_string())
}

fn hermes_profile_home() -> Result<PathBuf, String> {
    if let Ok(explicit_home) = env::var("HERMES_HOME") {
        return Ok(PathBuf::from(explicit_home));
    }
    let home = env::var("HOME").map_err(|_| {
        "HOME is not available, so Hermes profile settings cannot be resolved.".to_string()
    })?;
    let profile = active_hermes_profile_checked()?;
    if profile == "default" {
        Ok(PathBuf::from(home).join(".hermes"))
    } else {
        let profiles_root = PathBuf::from(home).join(".hermes").join("profiles");
        Ok(profiles_root.join(profile))
    }
}

fn hermes_config_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("config.yaml"))
}

fn hermes_memories_dir() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("memories"))
}

fn read_text_file_if_exists(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(path).map_err(|error| format!("Failed to read {}: {error}", path.display()))
}

fn write_text_file(path: &Path, value: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    let _ = backup_file(path, "zoid-save")?;
    fs::write(path, value).map_err(|error| format!("Failed to write {}: {error}", path.display()))
}

fn yaml_get_string(root: &serde_yaml::Value, path: &[&str]) -> Option<String> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(str::to_string)
}

fn yaml_get_bool(root: &serde_yaml::Value, path: &[&str]) -> Option<bool> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn yaml_get_u64(root: &serde_yaml::Value, path: &[&str]) -> Option<u64> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_u64()
}

fn yaml_get_mapping<'a>(
    root: &'a serde_yaml::Value,
    path: &[&str],
) -> Option<&'a serde_yaml::Mapping> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_mapping()
}

fn yaml_get_string_map(
    root: &serde_yaml::Value,
    path: &[&str],
) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    if let Some(mapping) = yaml_get_mapping(root, path) {
        for (key, value) in mapping {
            if let (Some(key), Some(value)) = (key.as_str(), value.as_str()) {
                out.insert(key.to_string(), value.to_string());
            }
        }
    }
    out
}

fn yaml_get_provider_models(
    root: &serde_yaml::Value,
    active_provider: &str,
    active_model: &str,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut models = default_available_models();
    merge_live_model_caches(&mut models);
    if !active_provider.trim().is_empty() && !active_model.trim().is_empty() {
        models
            .entry(active_provider.to_string())
            .or_default()
            .push(active_model.to_string());
    }
    if let Some(mapping) = yaml_get_mapping(root, &["providers"]) {
        for (key, value) in mapping {
            let Some(provider) = key.as_str() else {
                continue;
            };
            let mut provider_models = Vec::new();
            if let Some(model) = value.get("model").and_then(|value| value.as_str()) {
                provider_models.push(model.to_string());
            }
            if provider_models.is_empty()
                && provider == active_provider
                && !active_model.trim().is_empty()
            {
                provider_models.push(active_model.to_string());
            }
            models
                .entry(provider.to_string())
                .or_default()
                .extend(provider_models);
        }
    }
    for provider_models in models.values_mut() {
        *provider_models = sorted_unique(provider_models.clone());
    }
    models
}

fn backup_file(path: &Path, reason: &str) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("file");
    let backup_path =
        path.with_extension(format!("{extension}.{reason}.{}.bak", now_millis_string()));
    fs::copy(path, &backup_path)
        .map_err(|error| format!("Failed to backup {}: {error}", path.display()))?;
    Ok(Some(backup_path))
}

fn yaml_is_mapping(root: &serde_yaml::Value, path: &[&str]) -> bool {
    let mut current = root;
    for key in path {
        let Some(next) = current.get(*key) else {
            return false;
        };
        current = next;
    }
    current.is_mapping()
}

fn yaml_string_is_allowed(value: &str, allowed: &[&str], fallback: &str) -> String {
    if allowed.iter().any(|allowed_value| *allowed_value == value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

fn yaml_value_has_non_empty_config(value: &serde_yaml::Value) -> bool {
    match value {
        serde_yaml::Value::Bool(value) => *value,
        serde_yaml::Value::Number(_) => true,
        serde_yaml::Value::String(value) => !value.trim().is_empty(),
        serde_yaml::Value::Sequence(items) => items.iter().any(yaml_value_has_non_empty_config),
        serde_yaml::Value::Mapping(mapping) => {
            mapping.values().any(yaml_value_has_non_empty_config)
        }
        _ => false,
    }
}

fn configured_gateway_summary(config: &serde_yaml::Value) -> String {
    let platform_keys = [
        "discord",
        "telegram",
        "slack",
        "whatsapp",
        "signal",
        "email",
        "sms",
        "matrix",
        "mattermost",
        "feishu",
        "wecom",
        "weixin",
        "bluebubbles",
        "qqbot",
        "yuanbao",
    ];
    let mut configured = Vec::new();
    for key in platform_keys {
        if config.get(key).is_some_and(yaml_value_has_non_empty_config) {
            configured.push(key.to_string());
        }
    }
    let discord_channels =
        yaml_get_string(config, &["discord", "allowed_channels"]).unwrap_or_default();
    if !discord_channels.trim().is_empty() {
        format!(
            "Configured platforms: {}\nDiscord allowed channels:\n{}",
            configured.join(", "),
            discord_channels.replace(',', "\n")
        )
    } else if configured.is_empty() {
        "No configured gateway platforms detected in Hermes config.".to_string()
    } else {
        format!("Configured platforms: {}", configured.join(", "))
    }
}

fn yaml_set(root: &mut serde_yaml::Value, path: &[&str], value: serde_yaml::Value) {
    if path.is_empty() {
        *root = value;
        return;
    }
    if !root.is_mapping() {
        *root = serde_yaml::Value::Mapping(Default::default());
    }
    let mut current = root;
    for key in &path[..path.len() - 1] {
        let map = current.as_mapping_mut().expect("mapping ensured");
        let key_value = serde_yaml::Value::String((*key).to_string());
        current = map
            .entry(key_value)
            .or_insert_with(|| serde_yaml::Value::Mapping(Default::default()));
        if !current.is_mapping() {
            *current = serde_yaml::Value::Mapping(Default::default());
        }
    }
    if let Some(map) = current.as_mapping_mut() {
        map.insert(
            serde_yaml::Value::String(path[path.len() - 1].to_string()),
            value,
        );
    }
}

fn load_hermes_config_yaml() -> Result<Option<serde_yaml::Value>, String> {
    let path = hermes_config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read Hermes config: {error}"))?;
    let parsed = serde_yaml::from_str(&raw)
        .map_err(|error| format!("Failed to parse Hermes config: {error}"))?;
    Ok(Some(parsed))
}

fn apply_real_hermes_sources(settings: &mut HermesProfileSettings) -> Result<(), String> {
    let memory_dir = hermes_memories_dir()?;
    let memory = read_text_file_if_exists(&memory_dir.join("MEMORY.md"))?;
    let user = read_text_file_if_exists(&memory_dir.join("USER.md"))?;
    settings.hermes_memory = memory;
    settings.preferences = user;

    let cli_skill_status = discover_hermes_skill_status_from_cli();
    settings.available_skills = cli_skill_status
        .as_ref()
        .map(|(available, _enabled)| available.clone())
        .unwrap_or_else(discover_hermes_skills);

    if let Some(config) = load_hermes_config_yaml()? {
        if let Some(provider) = yaml_get_string(&config, &["model", "provider"]) {
            settings.model_provider = provider;
        }
        if let Some(model) = yaml_get_string(&config, &["model", "default"]) {
            settings.model_name = model;
        }
        settings.available_models =
            yaml_get_provider_models(&config, &settings.model_provider, &settings.model_name);
        let mut toolsets = default_available_toolsets();
        toolsets.extend(yaml_string_list(&config, &["platform_toolsets", "cli"]));
        toolsets.extend(yaml_string_list(&config, &["toolsets"]));
        settings.available_toolsets = sorted_unique(toolsets);
        let disabled_toolsets: BTreeSet<String> =
            yaml_string_list(&config, &["agent", "disabled_toolsets"])
                .into_iter()
                .collect();
        settings.toolsets = vec_to_lines(
            settings
                .available_toolsets
                .iter()
                .filter(|toolset| !disabled_toolsets.contains(*toolset))
                .cloned()
                .collect(),
        );
        settings.available_mcp_servers =
            sorted_unique(yaml_mapping_keys(&config, &["mcp_servers"]));
        settings.mcp_servers = vec_to_lines(
            settings
                .available_mcp_servers
                .iter()
                .filter(|server| {
                    yaml_get_bool(&config, &["mcp_servers", server.as_str(), "enabled"])
                        .unwrap_or(true)
                })
                .cloned()
                .collect(),
        );
        let disabled_skills: BTreeSet<String> = yaml_string_list(&config, &["skills", "disabled"])
            .into_iter()
            .collect();
        settings.enabled_skills = vec_to_lines(
            settings
                .available_skills
                .iter()
                .filter(|skill| !disabled_skills.contains(*skill))
                .cloned()
                .collect(),
        );
        let mut plugins = yaml_string_list(&config, &["plugins", "enabled"]);
        plugins.extend(yaml_string_list(&config, &["plugins", "disabled"]));
        settings.available_plugins = sorted_unique(plugins);
        settings.plugins = vec_to_lines(yaml_string_list(&config, &["plugins", "enabled"]));
        if let Some(reasoning) = yaml_get_string(&config, &["agent", "reasoning_effort"]) {
            settings.reasoning_effort = reasoning;
        }
        let config_personalities = yaml_get_string_map(&config, &["agent", "personalities"]);
        if !config_personalities.is_empty() {
            settings.style_templates = config_personalities;
        }
        if let Some(personality) = yaml_get_string(&config, &["display", "personality"]) {
            settings.personality_preset = personality;
        }
        if let Some(system_prompt) = yaml_get_string(&config, &["agent", "system_prompt"]) {
            settings.hermes_soul = system_prompt;
        }
        if let Some(approval) = yaml_get_string(&config, &["approvals", "mode"]) {
            settings.approval_mode = approval;
        }
        if let Some(timezone) = yaml_get_string(&config, &["timezone", "timezone"])
            .or_else(|| yaml_get_string(&config, &["timezone"]))
        {
            settings.timezone = timezone;
        }
        if let Some(memory_limit) = yaml_get_u64(&config, &["memory", "memory_char_limit"]) {
            settings.memory_char_limit = memory_limit;
        }
        if let Some(user_limit) = yaml_get_u64(&config, &["memory", "user_char_limit"]) {
            settings.user_char_limit = user_limit;
        }
        if let Some(memory_enabled) = yaml_get_bool(&config, &["memory", "memory_enabled"]) {
            settings.memory_enabled = memory_enabled;
        }
        if let Some(user_profile_enabled) =
            yaml_get_bool(&config, &["memory", "user_profile_enabled"])
        {
            settings.user_profile_enabled = user_profile_enabled;
        }
        if let Some(checkpoints_enabled) = yaml_get_bool(&config, &["checkpoints", "enabled"]) {
            settings.checkpoints_enabled = checkpoints_enabled;
        }
        if let Some(secret_redaction) = yaml_get_bool(&config, &["security", "redact_secrets"]) {
            settings.secret_redaction_enabled = secret_redaction;
        }
        if let Some(pii_redaction) = yaml_get_bool(&config, &["privacy", "redact_pii"]) {
            settings.pii_redaction_enabled = pii_redaction;
        }
        let stt_enabled = yaml_get_bool(&config, &["stt", "enabled"]).unwrap_or(false);
        let auto_tts = yaml_get_bool(&config, &["voice", "auto_tts"]).unwrap_or(false);
        settings.voice_preference = match (stt_enabled, auto_tts) {
            (true, _) => "voice".to_string(),
            (false, true) => "tts".to_string(),
            (false, false) => "off".to_string(),
        };
        settings.gateway_platforms = configured_gateway_summary(&config);
        if let Some(notification_mode) =
            yaml_get_string(&config, &["display", "background_process_notifications"])
        {
            settings.notification_preference = yaml_string_is_allowed(
                &notification_mode,
                &["off", "important", "all"],
                "important",
            );
        } else if let Some(bell) = yaml_get_bool(&config, &["display", "bell_on_complete"]) {
            settings.notification_preference = if bell {
                "all".to_string()
            } else {
                "important".to_string()
            };
        }
    }
    if settings.style_templates.is_empty() {
        settings.style_templates = default_style_templates();
    }
    if settings.available_models.is_empty() {
        settings.available_models = default_available_models();
    }
    if settings.available_toolsets.is_empty() {
        settings.available_toolsets = default_available_toolsets();
    }
    apply_toolset_toggles_from_lines(settings);
    if settings.available_mcp_servers.is_empty() {
        settings.available_mcp_servers = lines_to_sorted_vec(&settings.mcp_servers);
    }
    if settings.available_plugins.is_empty() {
        settings.available_plugins = lines_to_sorted_vec(&settings.plugins);
    }
    if settings.available_skills.is_empty() {
        settings.available_skills = discover_hermes_skills();
    }
    Ok(())
}

fn save_real_hermes_sources(settings: &HermesProfileSettings) -> Result<(), String> {
    let memory_dir = hermes_memories_dir()?;
    let config_path = hermes_config_path()?;
    let mut config: serde_yaml::Value = if config_path.exists() {
        let _ = backup_file(&config_path, "zoid-save")?;
        let raw = fs::read_to_string(&config_path)
            .map_err(|error| format!("Failed to read Hermes config: {error}"))?;
        serde_yaml::from_str(&raw)
            .map_err(|error| format!("Failed to parse Hermes config: {error}"))?
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };
    yaml_set(
        &mut config,
        &["model", "provider"],
        serde_yaml::Value::String(settings.model_provider.clone()),
    );
    yaml_set(
        &mut config,
        &["model", "default"],
        serde_yaml::Value::String(settings.model_name.clone()),
    );
    yaml_set(
        &mut config,
        &["agent", "reasoning_effort"],
        serde_yaml::Value::String(settings.reasoning_effort.clone()),
    );
    if yaml_is_mapping(&config, &["timezone"]) {
        yaml_set(
            &mut config,
            &["timezone", "timezone"],
            serde_yaml::Value::String(settings.timezone.clone()),
        );
    } else {
        yaml_set(
            &mut config,
            &["timezone"],
            serde_yaml::Value::String(settings.timezone.clone()),
        );
    }
    yaml_set(
        &mut config,
        &["display", "personality"],
        serde_yaml::Value::String(settings.personality_preset.clone()),
    );
    yaml_set(
        &mut config,
        &["agent", "system_prompt"],
        serde_yaml::Value::String(settings.hermes_soul.clone()),
    );
    yaml_set(
        &mut config,
        &["approvals", "mode"],
        serde_yaml::Value::String(settings.approval_mode.clone()),
    );
    yaml_set(
        &mut config,
        &["memory", "memory_enabled"],
        serde_yaml::Value::Bool(settings.memory_enabled),
    );
    yaml_set(
        &mut config,
        &["memory", "user_profile_enabled"],
        serde_yaml::Value::Bool(settings.user_profile_enabled),
    );
    yaml_set(
        &mut config,
        &["memory", "memory_char_limit"],
        serde_yaml::Value::Number(serde_yaml::Number::from(settings.memory_char_limit)),
    );
    yaml_set(
        &mut config,
        &["memory", "user_char_limit"],
        serde_yaml::Value::Number(serde_yaml::Number::from(settings.user_char_limit)),
    );
    yaml_set(
        &mut config,
        &["checkpoints", "enabled"],
        serde_yaml::Value::Bool(settings.checkpoints_enabled),
    );
    yaml_set(
        &mut config,
        &["security", "redact_secrets"],
        serde_yaml::Value::Bool(settings.secret_redaction_enabled),
    );
    yaml_set(
        &mut config,
        &["privacy", "redact_pii"],
        serde_yaml::Value::Bool(settings.pii_redaction_enabled),
    );
    yaml_set(
        &mut config,
        &["stt", "enabled"],
        serde_yaml::Value::Bool(settings.voice_preference == "voice"),
    );
    yaml_set(
        &mut config,
        &["voice", "auto_tts"],
        serde_yaml::Value::Bool(matches!(
            settings.voice_preference.as_str(),
            "tts" | "voice"
        )),
    );
    yaml_set(
        &mut config,
        &["display", "background_process_notifications"],
        serde_yaml::Value::String(settings.notification_preference.clone()),
    );
    yaml_set(
        &mut config,
        &["display", "bell_on_complete"],
        serde_yaml::Value::Bool(settings.notification_preference == "all"),
    );
    let enabled_toolsets =
        toolsets_from_feature_toggles(settings, lines_to_sorted_vec(&settings.toolsets));
    let all_toolsets = sorted_unique(
        [
            settings.available_toolsets.clone(),
            enabled_toolsets.clone(),
        ]
        .concat(),
    );
    let enabled_set: BTreeSet<String> = enabled_toolsets.iter().cloned().collect();
    let disabled_toolsets = all_toolsets
        .iter()
        .filter(|toolset| !enabled_set.contains(*toolset))
        .cloned()
        .collect::<Vec<_>>();
    yaml_set(
        &mut config,
        &["agent", "disabled_toolsets"],
        yaml_string_sequence(disabled_toolsets),
    );
    let enabled_skills = lines_to_sorted_vec(&settings.enabled_skills);
    let all_skills =
        sorted_unique([settings.available_skills.clone(), enabled_skills.clone()].concat());
    let skill_enabled_set: BTreeSet<String> = enabled_skills.iter().cloned().collect();
    let disabled_skills = all_skills
        .iter()
        .filter(|skill| !skill_enabled_set.contains(*skill))
        .cloned()
        .collect::<Vec<_>>();
    yaml_set(
        &mut config,
        &["skills", "disabled"],
        yaml_string_sequence(disabled_skills),
    );
    let enabled_plugins = lines_to_sorted_vec(&settings.plugins);
    let all_plugins =
        sorted_unique([settings.available_plugins.clone(), enabled_plugins.clone()].concat());
    let plugin_enabled_set: BTreeSet<String> = enabled_plugins.iter().cloned().collect();
    let disabled_plugins = all_plugins
        .iter()
        .filter(|plugin| !plugin_enabled_set.contains(*plugin))
        .cloned()
        .collect::<Vec<_>>();
    yaml_set(
        &mut config,
        &["plugins", "enabled"],
        yaml_string_sequence(enabled_plugins),
    );
    yaml_set(
        &mut config,
        &["plugins", "disabled"],
        yaml_string_sequence(disabled_plugins),
    );
    let available_mcp_servers: BTreeSet<String> =
        settings.available_mcp_servers.iter().cloned().collect();
    let enabled_mcp_servers = lines_to_sorted_vec(&settings.mcp_servers)
        .into_iter()
        .filter(|server| available_mcp_servers.contains(server))
        .collect::<Vec<_>>();
    set_mcp_server_enabled(
        &mut config,
        &enabled_mcp_servers,
        &settings.available_mcp_servers,
    );
    let serialized = serde_yaml::to_string(&config)
        .map_err(|error| format!("Failed to serialize Hermes config: {error}"))?;
    let _: serde_yaml::Value = serde_yaml::from_str(&serialized).map_err(|error| {
        format!("Serialized Hermes config did not parse; no files were written: {error}")
    })?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create Hermes config directory: {error}"))?;
    }
    fs::write(&config_path, serialized)
        .map_err(|error| format!("Failed to save Hermes config: {error}"))?;
    let saved_raw = fs::read_to_string(&config_path)
        .map_err(|error| format!("Failed to read saved Hermes config: {error}"))?;
    let _: serde_yaml::Value = serde_yaml::from_str(&saved_raw).map_err(|error| format!("Saved Hermes config did not parse; restore the .zoid-save backup before restarting Hermes: {error}"))?;
    write_text_file(&memory_dir.join("MEMORY.md"), &settings.hermes_memory)?;
    write_text_file(&memory_dir.join("USER.md"), &settings.preferences)?;
    Ok(())
}

fn hermes_profile_settings_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("zoid-profile-settings.json"))
}

fn default_hermes_profile_settings(path: &Path) -> HermesProfileSettings {
    HermesProfileSettings {
        profile: active_hermes_profile(),
        storage_path: path.to_string_lossy().to_string(),
        updated_at: now_millis_string(),
        ..HermesProfileSettings::default()
    }
}

fn load_hermes_profile_settings_inner() -> Result<HermesProfileSettings, String> {
    let path = hermes_profile_settings_path()?;
    let mut settings = if !path.exists() {
        default_hermes_profile_settings(&path)
    } else {
        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read Hermes profile settings: {error}"))?;
        serde_json::from_str(&raw)
            .map_err(|error| format!("Failed to parse Hermes profile settings: {error}"))?
    };
    settings.profile = active_hermes_profile();
    settings.storage_path = path.to_string_lossy().to_string();
    apply_real_hermes_sources(&mut settings)?;
    Ok(settings)
}

fn save_hermes_profile_settings_inner(
    mut settings: HermesProfileSettings,
) -> Result<HermesProfileSettings, String> {
    if !matches!(settings.access_mode.as_str(), "safe" | "workspace" | "full") {
        return Err(format!(
            "Invalid access_mode '{}'. Allowed values: safe, workspace, full.",
            settings.access_mode
        ));
    }
    if !matches!(settings.approval_mode.as_str(), "manual" | "smart" | "off") {
        return Err(format!(
            "Invalid approval_mode '{}'. Allowed values: manual, smart, off.",
            settings.approval_mode
        ));
    }
    if !matches!(
        settings.reasoning_effort.as_str(),
        "off" | "minimal" | "low" | "medium" | "high" | "xhigh"
    ) {
        return Err(format!("Invalid reasoning_effort '{}'. Allowed values: off, minimal, low, medium, high, xhigh.", settings.reasoning_effort));
    }
    if !matches!(
        settings.notification_preference.as_str(),
        "off" | "important" | "all"
    ) {
        return Err(format!(
            "Invalid notification_preference '{}'. Allowed values: off, important, all.",
            settings.notification_preference
        ));
    }
    if !matches!(settings.voice_preference.as_str(), "off" | "tts" | "voice") {
        return Err(format!(
            "Invalid voice_preference '{}'. Allowed values: off, tts, voice.",
            settings.voice_preference
        ));
    }
    if settings.memory_char_limit == 0 || settings.user_char_limit == 0 {
        return Err("Memory and user profile character limits must be greater than zero.".to_string());
    }
    let path = hermes_profile_settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("Failed to create Hermes profile settings directory: {error}")
        })?;
    }
    settings.profile = active_hermes_profile();
    settings.storage_path = path.to_string_lossy().to_string();
    settings.updated_at = now_millis_string();
    save_real_hermes_sources(&settings)?;
    let serialized = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("Failed to serialize Hermes profile settings: {error}"))?;
    fs::write(&path, serialized)
        .map_err(|error| format!("Failed to save Hermes profile settings: {error}"))?;
    Ok(settings)
}

fn split_csvish(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn protection_reason_for_job(job_id: &str, name: &str, script: Option<&str>) -> Option<String> {
    let candidates = [job_id, name, script.unwrap_or("")]
        .iter()
        .map(|value| value.trim().to_lowercase())
        .collect::<Vec<_>>();
    let protected_markers = [
        "feature-critique-watchdog",
        "feature_critique_watchdog.py",
        "obsidian hermes session archive",
        "obsidian_hermes_session_archive.py",
    ];
    protected_markers
        .iter()
        .find(|marker| candidates.iter().any(|candidate| candidate == *marker))
        .map(|marker| format!("Protected internal/system automation marker: {marker}"))
}

fn parse_last_run(value: &str) -> (Option<String>, Option<String>, Option<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "—" || trimmed.eq_ignore_ascii_case("never") {
        return (None, None, None);
    }
    let mut parts = trimmed.split_whitespace();
    let first = parts.next().unwrap_or("").trim();
    let status = parts.collect::<Vec<_>>().join(" ").trim().to_string();
    let status = if status.is_empty() {
        None
    } else {
        Some(status)
    };
    let delivery_error = status
        .as_ref()
        .filter(|status| status.to_lowercase().starts_with("error"))
        .cloned();
    (Some(first.to_string()), status, delivery_error)
}

fn finalize_cron_job(mut job: AutomationCronJob) -> AutomationCronJob {
    if job.name.is_empty() {
        job.name = job.job_id.clone();
    }
    let reason = protection_reason_for_job(&job.job_id, &job.name, job.script.as_deref());
    job.protected = reason.is_some();
    job.protection_reason = reason;
    job
}

fn empty_cron_job(job_id: &str, state: &str) -> AutomationCronJob {
    AutomationCronJob {
        job_id: job_id.to_string(),
        name: String::new(),
        schedule: "—".to_string(),
        repeat: "—".to_string(),
        deliver: "—".to_string(),
        next_run_at: None,
        last_run_at: None,
        last_status: None,
        last_delivery_error: None,
        enabled: state == "active",
        state: state.to_string(),
        paused_at: None,
        paused_reason: None,
        script: None,
        no_agent: false,
        skills: Vec::new(),
        prompt_preview: String::new(),
        enabled_toolsets: Vec::new(),
        protected: false,
        protection_reason: None,
    }
}

fn is_hermes_cron_job_id(value: &str) -> bool {
    value.len() == 12 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn parse_hermes_cron_list(output: &str, hermes_command: Option<String>) -> AutomationList {
    let mut jobs = Vec::<AutomationCronJob>::new();
    let mut current: Option<AutomationCronJob> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('┌')
            || trimmed.starts_with('│')
            || trimmed.starts_with('└')
        {
            continue;
        }

        if let Some((job_id_part, rest)) = trimmed.split_once(" [") {
            if is_hermes_cron_job_id(job_id_part.trim()) && rest.ends_with(']') {
                if let Some(job) = current.take() {
                    jobs.push(finalize_cron_job(job));
                }
                let state = rest.trim_end_matches(']').trim();
                current = Some(empty_cron_job(job_id_part.trim(), state));
                continue;
            }
        }

        let Some(job) = current.as_mut() else {
            continue;
        };
        let Some((label, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match label.trim() {
            "Name" => job.name = value.to_string(),
            "Schedule" => job.schedule = value.to_string(),
            "Repeat" => job.repeat = value.to_string(),
            "Next run" => {
                job.next_run_at = if value.is_empty() || value == "—" {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            "Deliver" => job.deliver = value.to_string(),
            "Script" => {
                job.script = if value.is_empty() || value == "—" {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            "Mode" => job.no_agent = value.to_lowercase().contains("no-agent"),
            "Skills" => job.skills = split_csvish(value),
            "Toolsets" | "Enabled toolsets" => job.enabled_toolsets = split_csvish(value),
            "Prompt" | "Prompt preview" => job.prompt_preview = value.chars().take(180).collect(),
            "Paused at" => {
                job.paused_at = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            "Paused reason" => {
                job.paused_reason = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            "Last run" => {
                let (last_run_at, last_status, last_delivery_error) = parse_last_run(value);
                job.last_run_at = last_run_at;
                job.last_status = last_status;
                job.last_delivery_error = last_delivery_error;
            }
            _ => {}
        }
    }

    if let Some(job) = current.take() {
        jobs.push(finalize_cron_job(job));
    }

    let count = jobs.len();
    AutomationList {
        jobs,
        watchers: Vec::new(),
        watcher_source_status: "unavailable".to_string(),
        count,
        refreshed_at: now_millis_string(),
        hermes_command,
        active_profile: active_hermes_profile(),
    }
}

fn list_hermes_automations_inner() -> Result<AutomationList, String> {
    let (path, _) = find_hermes_cli().ok_or_else(|| {
        "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH.".to_string()
    })?;
    let mut command = Command::new(&path);
    command.args(["cron", "list", "--all"]);
    let (success, stdout, stderr) = run_command_with_timeout(
        &mut command,
        Duration::from_secs(HERMES_CRON_TIMEOUT_SECONDS),
    )?;
    if !success {
        let error = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("Hermes cron list failed: {error}"));
    }
    Ok(parse_hermes_cron_list(
        &stdout,
        Some(command_display(&path)),
    ))
}

fn manage_hermes_cron_job_inner(job_id: &str, action: &str) -> Result<AutomationList, String> {
    let job_id = job_id.trim();
    if job_id.is_empty() {
        return Err("Cron job id is required.".to_string());
    }

    let action = action.trim();
    if !matches!(action, "pause" | "resume" | "run" | "remove") {
        return Err(format!("Unsupported cron action: {action}"));
    }

    if action == "remove" {
        let current = list_hermes_automations_inner()?;
        let Some(job) = current.jobs.iter().find(|job| job.job_id == job_id) else {
            return Err(
                "Cron job was not found in provider read-back; refusing remove.".to_string(),
            );
        };
        if job.protected {
            return Err(job
                .protection_reason
                .clone()
                .unwrap_or_else(|| "Protected cron job cannot be removed from Zoid.".to_string()));
        }
    }

    let (path, _) = find_hermes_cli().ok_or_else(|| {
        "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH.".to_string()
    })?;
    let mut command = Command::new(&path);
    match action {
        "pause" => command.args(["cron", "pause", job_id]),
        "resume" => command.args(["cron", "resume", job_id]),
        "run" => command.args(["cron", "run", "--accept-hooks", job_id]),
        "remove" => command.args(["cron", "remove", job_id]),
        _ => unreachable!(),
    };

    let (success, stdout, stderr) = run_command_with_timeout(
        &mut command,
        Duration::from_secs(HERMES_CRON_TIMEOUT_SECONDS),
    )?;
    if !success {
        let error = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("Hermes cron {action} failed: {error}"));
    }

    let refreshed = list_hermes_automations_inner()?;
    if action == "remove" && refreshed.jobs.iter().any(|job| job.job_id == job_id) {
        return Err(
            "Hermes reported success, but provider read-back still includes the removed job."
                .to_string(),
        );
    }
    Ok(refreshed)
}


fn mavoid_social_workspace_path() -> PathBuf {
    env::var("ZOID_MAVOID_SOCIAL_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(MAVOID_SOCIAL_WORKSPACE_DEFAULT))
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|item| item.as_str()).map(ToString::to_string)
}

fn read_json_file(path: &Path) -> Result<serde_json::Value, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&content).map_err(|error| format!("Failed to parse {}: {error}", path.display()))
}

fn json_string_at(value: &serde_json::Value, pointer: &str) -> Option<String> {
    value.pointer(pointer).and_then(|item| item.as_str()).map(ToString::to_string)
}

fn mavoid_status_blocker(status: &serde_json::Value) -> Option<String> {
    json_string(status, "current_blocker")
        .or_else(|| json_string_at(status, "/proof_post/not_posted_reason"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn mavoid_file_modified_iso(path: &Path) -> Option<String> {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| format!("{}", duration.as_secs()))
}

fn parse_mavoid_review_verdict(report_text: &str) -> Option<String> {
    report_text.lines().find_map(|line| {
        let value = line.trim().strip_prefix("Verdict:")?.trim();
        if value.contains("APPROVED") {
            Some("APPROVED".to_string())
        } else if value.contains("REQUEST_CHANGES") {
            Some("REQUEST_CHANGES".to_string())
        } else {
            None
        }
    })
}

fn parse_mavoid_required_fixes(manifest: &serde_json::Value, report_text: Option<&str>) -> Vec<String> {
    let mut fixes = manifest.get("required_fixes")
        .and_then(|value| value.as_array())
        .map(|items| items.iter().filter_map(|item| item.as_str().map(|value| value.trim().to_string())).filter(|value| !value.is_empty()).collect::<Vec<_>>())
        .unwrap_or_default();
    if fixes.is_empty() {
        if let Some(report_text) = report_text {
            for line in report_text.lines() {
                let trimmed = line.trim();
                if let Some(value) = trimmed.strip_prefix("Required fix:").or_else(|| trimmed.strip_prefix("Required fixes:")) {
                    let value = value.trim().trim_matches('-').trim();
                    if !value.is_empty() && !value.eq_ignore_ascii_case("none") {
                        fixes.push(value.to_string());
                    }
                }
            }
        }
    }
    fixes
}

fn mavoid_public_media_assets(
    manifest: &serde_json::Value,
    image: &Path,
) -> Vec<MavoidMediaAsset> {
    let mut urls = Vec::<(String, String)>::new();
    if let Some(items) = manifest.get("public_media_urls").and_then(|value| value.as_object()) {
        for (provider, url) in items {
            if let Some(url) = url.as_str() {
                urls.push((provider.to_string(), url.to_string()));
            }
        }
    }
    if urls.is_empty() {
        if let Some(url) = json_string(manifest, "preferred_public_media_url") {
            urls.push(("public-url".to_string(), url));
        }
    }
    if urls.is_empty() {
        urls.push(("local".to_string(), image.to_string_lossy().to_string()));
    }

    urls.into_iter().map(|(provider, url)| MavoidMediaAsset {
        path: image.to_string_lossy().to_string(),
        public_url: Some(url.clone()),
        content_type: Some("image/png".to_string()),
        bytes: fs::metadata(image).ok().map(|item| item.len()),
        width: Some(1080),
        height: Some(1350),
        validated_at: json_string(manifest, "updated_at"),
        provider: Some(provider),
        temporary: !url.contains("mavoid") && (url.contains("catbox") || url.contains("uguu") || url.contains("tmpfiles")),
        validation_status: if url.starts_with("https://") { "valid" } else { "unchecked" }.to_string(),
    }).collect()
}

fn mavoid_empty_counts() -> MavoidSocialCounts {
    MavoidSocialCounts { total_posts: 0, needs_review: 0, ready_to_schedule: 0, scheduled_verified: 0, posted: 0, blocked: 0 }
}

fn mavoid_unknown_buffer_health(message: Option<String>) -> MavoidBufferHealth {
    MavoidBufferHealth {
        ok: false,
        http_status: None,
        rate_limited: false,
        rate_limit_window: None,
        credentials_present: MavoidCredentialPresence { buffer_access_token: false, buffer_organization_id: false },
        last_checked_at: None,
        message,
    }
}

fn mavoid_health_from_status(status: &serde_json::Value) -> MavoidBufferHealth {
    let blocker = mavoid_status_blocker(status).unwrap_or_default();
    let blocker_lower = blocker.to_lowercase();
    let rate_limited = blocker_lower.contains("rate_limit") || blocker_lower.contains("rate limit") || blocker.contains("429");
    MavoidBufferHealth {
        ok: !rate_limited && blocker.is_empty(),
        http_status: if blocker.contains("429") { Some(429) } else { None },
        rate_limited,
        rate_limit_window: if blocker_lower.contains("24h") || blocker_lower.contains("24 hours") { Some("24h".to_string()) } else { None },
        credentials_present: MavoidCredentialPresence { buffer_access_token: false, buffer_organization_id: false },
        last_checked_at: json_string(status, "created_at"),
        message: if blocker.is_empty() { None } else { Some(blocker) },
    }
}

fn mavoid_automation_from_list(status: &serde_json::Value, list: Option<&AutomationList>) -> MavoidAutomationStatus {
    let status_cron = status.get("cron").unwrap_or(&serde_json::Value::Null);
    let creator_job_id = json_string(status_cron, "creator_job_id").unwrap_or_else(|| MAVOID_CREATOR_JOB_ID.to_string());
    let monitor_job_id = json_string(status_cron, "monitor_job_id").unwrap_or_else(|| MAVOID_MONITOR_JOB_ID.to_string());
    let cooldown_job_id = json_string(status_cron, "cooldown_resume_job_id");
    let creator = list.and_then(|item| item.jobs.iter().find(|job| job.job_id == creator_job_id));
    let monitor = list.and_then(|item| item.jobs.iter().find(|job| job.job_id == monitor_job_id));
    let cooldown = cooldown_job_id.as_ref().and_then(|id| list.and_then(|item| item.jobs.iter().find(|job| &job.job_id == id)));
    MavoidAutomationStatus {
        creator_job_id,
        creator_enabled: creator.map(|job| job.enabled).unwrap_or(false),
        creator_state: creator.map(|job| job.state.clone()).unwrap_or_else(|| "unknown".to_string()),
        creator_next_run_at: creator.and_then(|job| job.next_run_at.clone()),
        monitor_job_id,
        monitor_enabled: monitor.map(|job| job.enabled).unwrap_or(false),
        monitor_state: monitor.map(|job| job.state.clone()).unwrap_or_else(|| "unknown".to_string()),
        monitor_next_run_at: monitor.and_then(|job| job.next_run_at.clone()),
        cooldown_job_id,
        cooldown_next_run_at: cooldown.and_then(|job| job.next_run_at.clone()),
    }
}

fn mavoid_post_from_manifest(path: &Path) -> Result<MavoidSocialPost, String> {
    let manifest = read_json_file(path)?;
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let workspace = mavoid_social_workspace_path();
    let status_json = read_json_file(&workspace.join("STATUS.json")).unwrap_or_else(|_| serde_json::json!({}));
    let id = parent.file_name().and_then(|name| name.to_str()).unwrap_or("mavoid-post").to_string();
    let caption = json_string(&manifest, "caption").unwrap_or_default();
    let image = json_string(&manifest, "image")
        .map(PathBuf::from)
        .filter(|candidate| candidate.exists())
        .unwrap_or_else(|| parent.join("mavoid-buffer-proof-2026-06-09.png"));
    let platforms = manifest.get("platforms").and_then(|value| value.as_array()).map(|items| {
        items.iter().filter_map(|item| item.as_str().map(ToString::to_string)).collect::<Vec<_>>()
    }).filter(|items| !items.is_empty()).unwrap_or_else(|| vec!["instagram".to_string(), "facebook".to_string(), "linkedin".to_string()]);
    let review_path = parent.join("review-report.md");
    let review_text = if review_path.exists() { fs::read_to_string(&review_path).ok() } else { None };
    let verdict = review_text.as_deref().and_then(parse_mavoid_review_verdict).or_else(|| json_string(&manifest, "review_verdict")).unwrap_or_else(|| "MISSING".to_string());
    let manifest_status = json_string(&manifest, "status").unwrap_or_default();
    let provider_blocker = mavoid_status_blocker(&status_json);
    let provider_blocker_lower = provider_blocker.as_deref().unwrap_or_default().to_lowercase();
    let status = if manifest_status.contains("rate_limit") || provider_blocker_lower.contains("rate limit") || provider_blocker_lower.contains("rate_limit") || provider_blocker_lower.contains("429") {
        "rate_limited"
    } else if verdict == "APPROVED" {
        "media_hosted"
    } else {
        "review_requested"
    };
    let mut reports = vec![MavoidReportRef { label: "Manifest".to_string(), path: path.to_string_lossy().to_string(), kind: "generation".to_string(), created_at: json_string(&manifest, "created_at").or_else(|| mavoid_file_modified_iso(path)) }];
    if review_path.exists() {
        reports.push(MavoidReportRef { label: "Review report".to_string(), path: review_path.to_string_lossy().to_string(), kind: "review".to_string(), created_at: json_string(&manifest, "updated_at").or_else(|| mavoid_file_modified_iso(&review_path)) });
    }
    if workspace.join("STATUS.json").exists() {
        reports.push(MavoidReportRef { label: "Runtime status".to_string(), path: workspace.join("STATUS.json").to_string_lossy().to_string(), kind: "monitor".to_string(), created_at: json_string(&status_json, "created_at") });
    }

    let mut events = vec![
        MavoidSocialEvent { timestamp: json_string(&manifest, "created_at").unwrap_or_else(now_millis_string), actor: "hermes".to_string(), event_type: "manifest_created".to_string(), message: "Post manifest was created in the local MaVoid social runtime.".to_string(), severity: "info".to_string(), evidence_path: Some(path.to_string_lossy().to_string()) },
        MavoidSocialEvent { timestamp: json_string(&manifest, "updated_at").unwrap_or_else(now_millis_string), actor: "hermes".to_string(), event_type: format!("review_{}", verdict.to_lowercase()), message: format!("Review verdict read from manifest{}: {verdict}.", if review_path.exists() { " and review-report.md" } else { "" }), severity: if verdict == "APPROVED" { "success".to_string() } else { "warning".to_string() }, evidence_path: if review_path.exists() { Some(review_path.to_string_lossy().to_string()) } else { Some(path.to_string_lossy().to_string()) } },
    ];
    if json_string(&manifest, "preferred_public_media_url").is_some() || manifest.get("public_media_urls").is_some() {
        events.push(MavoidSocialEvent { timestamp: json_string(&manifest, "updated_at").unwrap_or_else(now_millis_string), actor: "hermes".to_string(), event_type: "public_media_urls_ready".to_string(), message: "Public HTTPS media URLs are present for validation/opening.".to_string(), severity: "success".to_string(), evidence_path: Some(path.to_string_lossy().to_string()) });
    }
    if let Some(blocker) = provider_blocker.clone() {
        events.push(MavoidSocialEvent { timestamp: json_string(&status_json, "created_at").unwrap_or_else(now_millis_string), actor: "buffer".to_string(), event_type: "provider_blocker".to_string(), message: blocker, severity: "warning".to_string(), evidence_path: Some(workspace.join("STATUS.json").to_string_lossy().to_string()) });
    }

    let required_fixes = parse_mavoid_required_fixes(&manifest, review_text.as_deref());
    let approved_at = if verdict == "APPROVED" { json_string(&manifest, "approved_at").or_else(|| json_string(&manifest, "updated_at")) } else { None };

    Ok(MavoidSocialPost {
        id,
        post_date: json_string(&manifest, "post_date").unwrap_or_else(|| "2026-06-09".to_string()),
        slot_type: json_string(&manifest, "slot_type").unwrap_or_else(|| "manual_campaign".to_string()),
        title: json_string(&manifest, "title").unwrap_or_else(|| "Buffer pipeline proof".to_string()),
        topic_or_news_item: json_string(&manifest, "purpose").unwrap_or_else(|| "Buffer migration proof".to_string()),
        caption,
        platforms: platforms.clone(),
        status: status.to_string(),
        review: Some(MavoidReviewReport { verdict, reviewer: json_string(&manifest, "reviewer"), report_path: if review_path.exists() { Some(review_path.to_string_lossy().to_string()) } else { None }, required_fixes, approved_at }),
        media_assets: mavoid_public_media_assets(&manifest, &image),
        buffer_posts: platforms.into_iter().map(|platform| MavoidBufferPost { buffer_id: None, platform, channel_id: None, channel_display_name: None, scheduled_at_utc: None, scheduled_at_local: None, state: "not_created".to_string(), read_back_verified_at: json_string(&status_json, "created_at"), published_url: None, last_error_code: provider_blocker.as_ref().filter(|blocker| blocker.contains("RATE_LIMIT") || blocker.contains("429")).map(|_| "RATE_LIMIT_EXCEEDED".to_string()), last_error_message: provider_blocker.clone() }).collect(),
        reports,
        events,
    })
}

fn mavoid_social_list_posts_inner() -> Result<Vec<MavoidSocialPost>, String> {
    let workspace = mavoid_social_workspace_path();
    let artifacts = workspace.join("artifacts");
    let mut posts = Vec::new();
    if artifacts.exists() {
        for entry in fs::read_dir(&artifacts).map_err(|error| format!("Failed to scan {}: {error}", artifacts.display()))? {
            let entry = entry.map_err(|error| format!("Failed to read artifact entry: {error}"))?;
            let manifest = entry.path().join("manifest.json");
            if manifest.exists() {
                posts.push(mavoid_post_from_manifest(&manifest)?);
            }
        }
    }
    posts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(posts)
}

fn mavoid_counts(posts: &[MavoidSocialPost]) -> MavoidSocialCounts {
    let mut counts = mavoid_empty_counts();
    for post in posts {
        counts.total_posts += 1;
        if post.status == "review_requested" || post.status == "request_changes" || post.review.as_ref().map(|review| review.verdict.as_str()) == Some("REQUEST_CHANGES") { counts.needs_review += 1; }
        if post.status == "approved" || post.status == "media_hosted" { counts.ready_to_schedule += 1; }
        if post.status == "scheduled_verified" { counts.scheduled_verified += 1; }
        if post.status == "posted" { counts.posted += 1; }
        if matches!(post.status.as_str(), "rate_limited" | "media_blocked" | "buffer_failed" | "failed_closed") { counts.blocked += 1; }
    }
    counts
}

fn mavoid_social_overview_with_health(buffer_health_override: Option<MavoidBufferHealth>) -> Result<MavoidSocialOverview, String> {
    let workspace = mavoid_social_workspace_path();
    let status_path = workspace.join("STATUS.json");
    let status = if status_path.exists() { read_json_file(&status_path)? } else { serde_json::json!({}) };
    let automation_list = list_hermes_automations_inner().ok();
    let posts = mavoid_social_list_posts_inner().unwrap_or_default();
    let buffer_health = buffer_health_override.unwrap_or_else(|| if status.is_object() { mavoid_health_from_status(&status) } else { mavoid_unknown_buffer_health(Some("STATUS.json missing".to_string())) });
    let counts = mavoid_counts(&posts);
    let overall_status = if buffer_health.rate_limited { "rate_limited" } else if counts.needs_review > 0 { "needs_review" } else if counts.ready_to_schedule > 0 { "ready_to_schedule" } else if counts.total_posts == 0 { "unknown" } else { "healthy" };
    Ok(MavoidSocialOverview {
        workspace_path: workspace.to_string_lossy().to_string(),
        overall_status: overall_status.to_string(),
        active_blocker: json_string(&status, "current_blocker").or_else(|| buffer_health.message.clone()),
        buffer_endpoint: MAVOID_BUFFER_ENDPOINT.to_string(),
        buffer_health,
        automation: mavoid_automation_from_list(&status, automation_list.as_ref()),
        counts,
        next_slots: Vec::new(),
        latest_report_path: Some(workspace.join("reports/buffer-blocker-fixes-handoff.md").to_string_lossy().to_string()).filter(|path| Path::new(path).exists()),
        updated_at: now_millis_string(),
    })
}

fn mavoid_social_get_post_inner(post_id: &str) -> Result<MavoidSocialPost, String> {
    mavoid_social_list_posts_inner()?.into_iter().find(|post| post.id == post_id).ok_or_else(|| format!("MaVoid social post not found: {post_id}"))
}

fn parse_mavoid_buffer_check_output(output: &str) -> MavoidBufferHealth {
    let parsed: serde_json::Value = serde_json::from_str(output).unwrap_or_else(|_| serde_json::json!({"message": output}));
    MavoidBufferHealth {
        ok: parsed.get("ok").and_then(|value| value.as_bool()).unwrap_or(false),
        http_status: parsed.get("http_status").and_then(|value| value.as_i64()),
        rate_limited: parsed.get("rate_limited").and_then(|value| value.as_bool()).unwrap_or(false),
        rate_limit_window: json_string(&parsed, "rate_limit_window"),
        credentials_present: MavoidCredentialPresence {
            buffer_access_token: parsed.pointer("/credentials_present/BUFFER_ACCESS_TOKEN").and_then(|value| value.as_bool()).unwrap_or(false),
            buffer_organization_id: parsed.pointer("/credentials_present/BUFFER_ORGANIZATION_ID").and_then(|value| value.as_bool()).unwrap_or(false),
        },
        last_checked_at: Some(now_millis_string()),
        message: parsed.pointer("/response/errors/0/message").and_then(|value| value.as_str()).map(ToString::to_string).or_else(|| json_string(&parsed, "message")),
    }
}

fn mavoid_social_run_buffer_health_check_inner() -> Result<MavoidSocialOverview, String> {
    let script = mavoid_social_workspace_path().join("scripts/buffer_check.py");
    let mut command = Command::new("python3");
    command.arg(&script);
    let (_success, stdout, stderr) = run_command_with_timeout(&mut command, Duration::from_secs(45))?;
    let output = if stdout.trim().is_empty() { stderr } else { stdout };
    let health = parse_mavoid_buffer_check_output(&output);
    mavoid_social_overview_with_health(Some(health))
}

fn mavoid_social_manage_automation_inner(action: &str) -> Result<MavoidSocialOverview, String> {
    let (job_id, cron_action) = match action {
        "run_creator" => (MAVOID_CREATOR_JOB_ID, "run"),
        "pause_creator" => (MAVOID_CREATOR_JOB_ID, "pause"),
        "resume_creator" => (MAVOID_CREATOR_JOB_ID, "resume"),
        "pause_monitor" => (MAVOID_MONITOR_JOB_ID, "pause"),
        "resume_monitor" => (MAVOID_MONITOR_JOB_ID, "resume"),
        _ => return Err(format!("Unsupported MaVoid social automation action: {action}")),
    };
    let _ = manage_hermes_cron_job_inner(job_id, cron_action)?;
    mavoid_social_overview_with_health(None)
}

fn mavoid_social_validate_media_url_inner(url: &str) -> Result<MavoidMediaValidation, String> {
    if !url.starts_with("https://") {
        return Ok(MavoidMediaValidation { url: url.to_string(), ok: false, http_status: None, content_type: None, bytes: None, message: "Only https:// media URLs are allowed.".to_string() });
    }
    let mut command = Command::new("curl");
    command.args(["-L", "-I", "--max-time", "20", url]);
    let (success, stdout, stderr) = run_command_with_timeout(&mut command, Duration::from_secs(30))?;
    let output = if stdout.trim().is_empty() { stderr } else { stdout };
    let status = output.lines().filter_map(|line| line.strip_prefix("HTTP/").and_then(|rest| rest.split_whitespace().next()).and_then(|code| code.parse::<i32>().ok())).last();
    let content_type = output.lines().find_map(|line| line.to_lowercase().strip_prefix("content-type:").map(|_| line.split_once(':').map(|(_, value)| value.trim().to_string()).unwrap_or_default()));
    let bytes = output.lines().find_map(|line| line.to_lowercase().strip_prefix("content-length:").and_then(|_| line.split_once(':').and_then(|(_, value)| value.trim().parse::<u64>().ok())));
    let image = content_type.as_deref().map(|value| value.starts_with("image/png") || value.starts_with("image/jpeg") || value.starts_with("image/webp")).unwrap_or(false);
    Ok(MavoidMediaValidation { url: url.to_string(), ok: success && status == Some(200) && image, http_status: status, content_type, bytes, message: if success && status == Some(200) && image { "Direct image URL is valid.".to_string() } else { "URL did not validate as a direct public image.".to_string() } })
}

fn mavoid_social_open_resource_inner(resource: &str) -> Result<(), String> {
    let resource = resource.trim();
    if resource.starts_with("https://") {
        let mut command = Command::new("open");
        command.arg(resource);
        let (success, _stdout, stderr) = run_command_with_timeout(&mut command, Duration::from_secs(10))?;
        return if success { Ok(()) } else { Err(format!("Failed to open media URL: {stderr}")) };
    }

    let path = PathBuf::from(resource);
    if !path.exists() {
        return Err(format!("Report or media path does not exist: {resource}"));
    }
    let canonical = path.canonicalize().map_err(|error| format!("Failed to resolve {resource}: {error}"))?;
    let workspace = mavoid_social_workspace_path().canonicalize().map_err(|error| format!("Failed to resolve MaVoid social workspace: {error}"))?;
    if !canonical.starts_with(&workspace) {
        return Err("Only MaVoid social workspace reports/media can be opened from this panel.".to_string());
    }
    let mut command = Command::new("open");
    command.arg(&canonical);
    let (success, _stdout, stderr) = run_command_with_timeout(&mut command, Duration::from_secs(10))?;
    if success { Ok(()) } else { Err(format!("Failed to open local report: {stderr}")) }
}

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn check_hermes_cli() -> Result<HermesCliStatus, String> {
        let session = hermes_session_name();
        match find_hermes_cli() {
            Some((path, version)) => Ok(HermesCliStatus {
                ok: true,
                status: "online".to_string(),
                message: format!("Hermes CLI is available: {version}"),
                command: Some(command_display(&path)),
                session,
            }),
            None => Ok(HermesCliStatus {
                ok: false,
                status: "offline".to_string(),
                message:
                    "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH."
                        .to_string(),
                command: None,
                session,
            }),
        }
    }

    #[tauri::command]
    pub async fn load_brain_store() -> Result<BrainStore, String> {
        load_brain_store_inner()
    }

    #[tauri::command]
    pub async fn list_apple_notes_folders() -> Result<Vec<AppleNotesFolder>, String> {
        list_apple_notes_folders_inner()
    }

    #[tauri::command]
    pub async fn ensure_zoid_brain_folder() -> Result<AppleNotesSource, String> {
        ensure_zoid_brain_folder_inner()
    }

    #[tauri::command]
    pub async fn link_apple_notes_folder(
        account_name: String,
        folder_name: String,
        sync_mode: String,
    ) -> Result<AppleNotesSource, String> {
        link_apple_notes_folder_inner(account_name, folder_name, sync_mode)
    }

    #[tauri::command]
    pub async fn sync_apple_notes_sources() -> Result<BrainStore, String> {
        sync_apple_notes_sources_inner()
    }

    #[tauri::command]
    pub async fn extract_brain_note(note_id: String) -> Result<BrainStore, String> {
        extract_brain_note_inner(&note_id)
    }

    #[tauri::command]
    pub async fn create_brain_clarifying_session(
        note_id: String,
        task_candidate_ids: Vec<String>,
    ) -> Result<BrainStore, String> {
        create_brain_clarifying_session_inner(&note_id, task_candidate_ids)
    }

    #[tauri::command]
    pub async fn answer_brain_clarifying_session(
        session_id: String,
        answer: String,
    ) -> Result<BrainStore, String> {
        answer_brain_clarifying_session_inner(&session_id, &answer)
    }

    #[tauri::command]
    pub async fn list_hermes_slash_commands() -> Result<Vec<HermesSlashCommand>, String> {
        load_hermes_slash_commands_inner()
    }

    #[tauri::command]
    pub async fn execute_hermes_slash_command(
        command: String,
        linked_repository: Option<String>,
        hermes_session: Option<String>,
        confirmed: bool,
    ) -> Result<HermesSlashExecutionResult, String> {
        execute_hermes_slash_command_inner(&command, linked_repository, hermes_session, confirmed)
    }

    #[tauri::command]
    pub async fn list_hermes_cli_runs() -> Result<Vec<HermesRunSnapshot>, String> {
        Ok(list_hermes_run_snapshots_inner())
    }

    #[tauri::command]
    pub async fn cancel_hermes_cli_message() -> Result<bool, String> {
        cancel_active_hermes_run_inner()
    }

    #[tauri::command]
    pub async fn cancel_hermes_cli_run(
        session_id: Option<String>,
        run_id: Option<String>,
    ) -> Result<bool, String> {
        cancel_hermes_run_inner(session_id, run_id)
    }

    fn send_hermes_cli_message_inner(
        messages: Vec<HermesCliMessage>,
        linked_repository: Option<String>,
        hermes_session: Option<String>,
        session_id: Option<String>,
        run_id: Option<String>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<HermesCliResponse, String> {
        let prompt = messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .map(|message| message.content.trim().to_string())
            .filter(|message| !message.is_empty())
            .ok_or_else(|| "Cannot send an empty message to Hermes CLI.".to_string())?;

        let (path, _) = find_hermes_cli().ok_or_else(|| {
            "Hermes CLI was not found. Set ZOID_HERMES_CLI or ensure hermes is on PATH.".to_string()
        })?;
        let session = hermes_session_name();
        let run_session_id = normalize_hermes_run_identifier(
            session_id.or_else(|| hermes_session.clone()),
            &session,
        );
        let run_id = normalize_hermes_run_identifier(run_id, "default-run");
        let repository_workdir = resolve_linked_repository_workdir(linked_repository)?;
        let parsed_cli_args = hermes_cli_args_from_prompt(&prompt)?;
        let explicit_hermes_command = parsed_cli_args.is_some();
        let cli_args = hermes_invocation_args(match parsed_cli_args {
            Some(args) => args,
            None => build_profiled_hermes_chat_args(&prompt, hermes_session.as_deref())?,
        });
        if explicit_hermes_command {
            // Explicit terminal-style Hermes commands are left exactly as the user typed them.
        }
        let usage = command_usage(&path, &cli_args, repository_workdir.as_deref());
        let mut command = Command::new(&path);
        if let Some(workdir) = repository_workdir.as_deref() {
            command.current_dir(workdir);
        }
        command.args(&cli_args);

        let (success, stdout, stderr) = run_hermes_command_for_session_with_cancel(
            &mut command,
            Duration::from_secs(HERMES_TIMEOUT_SECONDS),
            run_session_id,
            run_id,
            app_handle,
        )?;

        if !success {
            let error = if stderr.is_empty() { stdout } else { stderr };
            return Err(format!(
                "Hermes CLI returned an error while running `$ {usage}`: {error}"
            ));
        }

        let combined_output = [stdout.as_str(), stderr.as_str()]
            .into_iter()
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let content = strip_terminal_noise(&combined_output);
        if content.is_empty() {
            return Err(format!(
                "Hermes CLI returned an empty response while running `$ {usage}`."
            ));
        }

        let returned_session = parse_hermes_session_id(&combined_output, &session);
        Ok(HermesCliResponse {
            content: with_terminal_usage(&usage, &content),
            session: returned_session,
        })
    }

    #[tauri::command]
    pub async fn send_hermes_cli_message(
        messages: Vec<HermesCliMessage>,
        linked_repository: Option<String>,
        hermes_session: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<HermesCliResponse, String> {
        send_hermes_cli_message_inner(messages, linked_repository, hermes_session, None, None, Some(app_handle))
    }

    #[cfg(test)]
    pub async fn send_hermes_cli_message_for_test(
        messages: Vec<HermesCliMessage>,
        linked_repository: Option<String>,
        hermes_session: Option<String>,
    ) -> Result<HermesCliResponse, String> {
        send_hermes_cli_message_inner(messages, linked_repository, hermes_session, None, None, None)
    }

    #[tauri::command]
    pub async fn send_hermes_cli_run_message(
        messages: Vec<HermesCliMessage>,
        linked_repository: Option<String>,
        hermes_session: Option<String>,
        session_id: Option<String>,
        run_id: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<HermesCliResponse, String> {
        send_hermes_cli_message_inner(
            messages,
            linked_repository,
            hermes_session,
            session_id,
            run_id,
            Some(app_handle),
        )
    }

    #[cfg(test)]
    pub async fn send_hermes_cli_run_message_for_test(
        messages: Vec<HermesCliMessage>,
        linked_repository: Option<String>,
        hermes_session: Option<String>,
        session_id: Option<String>,
        run_id: Option<String>,
    ) -> Result<HermesCliResponse, String> {
        send_hermes_cli_message_inner(messages, linked_repository, hermes_session, session_id, run_id, None)
    }

    #[tauri::command]
    pub async fn send_agent_response_email_notification(
        request: AgentResponseEmailNotificationRequest,
    ) -> Result<AgentResponseEmailNotificationResult, String> {
        send_agent_response_email_notification_inner(request)
    }

    #[tauri::command]
    pub async fn list_file_manager_directory(
        path: Option<String>,
    ) -> Result<FileManagerDirectoryListing, String> {
        list_file_manager_directory_inner(path)
    }

    #[tauri::command]
    pub async fn scan_github_repositories(folder: String) -> Result<Vec<CodeRepository>, String> {
        scan_repository_folder(&folder)
    }

    #[tauri::command]
    pub async fn clone_github_repository(
        repo_url: String,
        destination_root: String,
    ) -> Result<CodeRepository, String> {
        clone_repository(&repo_url, &destination_root)
    }

    #[tauri::command]
    pub async fn list_github_branches(
        repository_path: String,
        remote_url: Option<String>,
        current_default_branch: Option<String>,
    ) -> Result<Vec<GithubBranch>, String> {
        list_remote_branches(&repository_path, remote_url, current_default_branch)
    }

    #[tauri::command]
    pub async fn update_github_default_branch(
        repository_path: String,
        remote_url: Option<String>,
        default_branch: String,
    ) -> Result<CodeRepository, String> {
        update_default_branch(&repository_path, remote_url, &default_branch)
    }

    #[tauri::command]
    pub async fn list_hermes_automations() -> Result<AutomationList, String> {
        list_hermes_automations_inner()
    }

    #[tauri::command]
    pub async fn manage_hermes_cron_job(
        job_id: String,
        action: String,
    ) -> Result<AutomationList, String> {
        manage_hermes_cron_job_inner(&job_id, &action)
    }


    #[tauri::command]
    pub async fn mavoid_social_get_overview() -> Result<MavoidSocialOverview, String> {
        mavoid_social_overview_with_health(None)
    }

    #[tauri::command]
    pub async fn mavoid_social_list_posts() -> Result<Vec<MavoidSocialPost>, String> {
        mavoid_social_list_posts_inner()
    }

    #[tauri::command]
    pub async fn mavoid_social_get_post(post_id: String) -> Result<MavoidSocialPost, String> {
        mavoid_social_get_post_inner(&post_id)
    }

    #[tauri::command]
    pub async fn mavoid_social_run_buffer_health_check() -> Result<MavoidSocialOverview, String> {
        mavoid_social_run_buffer_health_check_inner()
    }

    #[tauri::command]
    pub async fn mavoid_social_manage_automation(action: String) -> Result<MavoidSocialOverview, String> {
        mavoid_social_manage_automation_inner(&action)
    }

    #[tauri::command]
    pub async fn mavoid_social_validate_media_url(url: String) -> Result<MavoidMediaValidation, String> {
        mavoid_social_validate_media_url_inner(&url)
    }

    #[tauri::command]
    pub async fn mavoid_social_open_resource(resource: String) -> Result<(), String> {
        mavoid_social_open_resource_inner(&resource)
    }

    #[tauri::command]
    pub async fn load_hermes_profile_settings() -> Result<HermesProfileSettings, String> {
        load_hermes_profile_settings_inner()
    }

    #[tauri::command]
    pub async fn save_hermes_profile_settings(
        settings: HermesProfileSettings,
    ) -> Result<HermesProfileSettings, String> {
        save_hermes_profile_settings_inner(settings)
    }

    #[tauri::command]
    pub async fn warm_file_permissions(force: bool) -> Result<Vec<String>, String> {
        warm_file_permissions_inner(force)
    }

    #[tauri::command]
    pub async fn list_managed_providers() -> Result<Vec<ManagedProvider>, String> {
        load_managed_providers_inner()
    }

    #[tauri::command]
    pub async fn save_managed_provider(provider: ProviderInput) -> Result<ManagedProvider, String> {
        save_managed_provider_inner(provider)
    }

    #[tauri::command]
    pub async fn validate_managed_provider(
        provider_id: String,
    ) -> Result<ProviderValidationResult, String> {
        validate_managed_provider_inner(&provider_id)
    }

    #[tauri::command]
    pub async fn apply_managed_provider(
        provider_id: String,
    ) -> Result<ProviderApplyResult, String> {
        apply_managed_provider_inner(&provider_id)
    }

    #[tauri::command]
    pub async fn reveal_managed_provider_key(
        provider_id: String,
    ) -> Result<ProviderKeyReveal, String> {
        Ok(ProviderKeyReveal {
            provider_id: provider_id.clone(),
            api_key: read_provider_key(&provider_id)?,
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::check_hermes_cli,
            commands::load_brain_store,
            commands::list_apple_notes_folders,
            commands::ensure_zoid_brain_folder,
            commands::link_apple_notes_folder,
            commands::sync_apple_notes_sources,
            commands::extract_brain_note,
            commands::create_brain_clarifying_session,
            commands::answer_brain_clarifying_session,
            commands::list_hermes_slash_commands,
            commands::execute_hermes_slash_command,
            commands::list_hermes_cli_runs,
            commands::cancel_hermes_cli_message,
            commands::cancel_hermes_cli_run,
            commands::send_hermes_cli_message,
            commands::send_hermes_cli_run_message,
            commands::send_agent_response_email_notification,
            commands::list_file_manager_directory,
            commands::scan_github_repositories,
            commands::clone_github_repository,
            commands::list_github_branches,
            commands::update_github_default_branch,
            commands::list_hermes_automations,
            commands::manage_hermes_cron_job,
            commands::mavoid_social_get_overview,
            commands::mavoid_social_list_posts,
            commands::mavoid_social_get_post,
            commands::mavoid_social_run_buffer_health_check,
            commands::mavoid_social_manage_automation,
            commands::mavoid_social_validate_media_url,
            commands::mavoid_social_open_resource,
            commands::load_hermes_profile_settings,
            commands::save_hermes_profile_settings,
            commands::warm_file_permissions,
            commands::list_managed_providers,
            commands::save_managed_provider,
            commands::validate_managed_provider,
            commands::apply_managed_provider,
            commands::reveal_managed_provider_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zoid 25");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zoid-{label}-{timestamp}"))
    }

    fn init_git_repo(path: &Path) {
        fs::create_dir_all(path).unwrap();
        let mut command = Command::new("git");
        command.arg("init").arg(path);
        let (success, _, stderr) =
            run_command_with_timeout(&mut command, Duration::from_secs(20)).unwrap();
        assert!(success, "git init failed: {stderr}");
    }

    fn brain_source(sync_mode: &str) -> AppleNotesSource {
        AppleNotesSource {
            id: "apple-notes:icloud:Zoid Brain".to_string(),
            source_type: "appleNotes".to_string(),
            account_name: "iCloud".to_string(),
            folder_name: "Zoid Brain".to_string(),
            sync_mode: sync_mode.to_string(),
            enabled: true,
            created_by_zoid: true,
            last_synced_at: None,
            last_error: None,
        }
    }

    fn raw_note(title: &str, body: &str, modified: &str) -> AppleNotesRawNote {
        AppleNotesRawNote {
            account_name: "iCloud".to_string(),
            folder_name: "Zoid Brain".to_string(),
            apple_note_id: Some("note-1".to_string()),
            title: title.to_string(),
            body: body.to_string(),
            created_at: Some("100".to_string()),
            modified_at: Some(modified.to_string()),
        }
    }

    fn brain_note(id: &str, title: &str, body: &str) -> BrainNote {
        BrainNote {
            id: id.to_string(),
            source_type: "appleNotes".to_string(),
            source_id: "source-1".to_string(),
            apple_note_id: format!("apple-{id}"),
            title: title.to_string(),
            body: body.to_string(),
            source_folder: "Zoid Brain".to_string(),
            account_name: "iCloud".to_string(),
            apple_created_at: None,
            apple_modified_at: Some("200".to_string()),
            zoid_modified_at: None,
            imported_at: "201".to_string(),
            last_synced_at: None,
            last_synced_title: title.to_string(),
            last_synced_body: body.to_string(),
            last_synced_hash: note_content_hash(title, body),
            current_hash: note_content_hash(title, body),
            sync_status: "synced".to_string(),
            archived: false,
        }
    }

    #[test]
    fn hermes_registry_python_prefers_project_venv_before_system_python() {
        let root = unique_temp_path("hermes-registry-python");
        fs::create_dir_all(root.join("venv/bin")).unwrap();
        fs::write(root.join("venv/bin/python"), "#!/bin/sh\n").unwrap();

        let candidates = candidate_hermes_registry_pythons(&root);

        assert_eq!(candidates.first(), Some(&root.join("venv/bin/python")));
        assert!(candidates.iter().any(|candidate| candidate == &PathBuf::from("python3")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hermes_registry_loads_more_than_offline_fallback_when_source_is_available() {
        if find_hermes_source_root().is_none() {
            return;
        }

        let commands = load_hermes_slash_commands_inner().unwrap();

        assert!(commands.len() > 4, "live Hermes registry should include more commands than the four offline fallback entries");
        assert!(commands.iter().any(|command| command.name == "plan"));
        assert!(commands.iter().any(|command| command.name == "tools"));
    }

    #[test]
    fn parse_apple_notes_folder_json_accepts_camel_case() {
        let folders = parse_apple_notes_folders_json(
            r#"[{"accountName":"iCloud","folderName":"Zoid Brain","id":"folder-1"}]"#,
        )
        .unwrap();
        assert_eq!(
            folders,
            vec![AppleNotesFolder {
                account_name: "iCloud".to_string(),
                folder_name: "Zoid Brain".to_string(),
                id: Some("folder-1".to_string())
            }]
        );
    }

    #[test]
    fn upsert_zoid_brain_source_deduplicates_and_updates_mode() {
        let mut store = BrainStore::default();
        let source = AppleNotesSource {
            sync_mode: "readOnly".to_string(),
            ..brain_source("readOnly")
        };
        upsert_apple_notes_source(&mut store, source.clone());
        upsert_apple_notes_source(&mut store, brain_source("twoWay"));
        assert_eq!(store.sources.len(), 1);
        assert_eq!(store.sources[0].sync_mode, "twoWay");
        assert_eq!(store.sources[0].source_type, "appleNotes");
        assert!(store.sources[0].enabled);
    }

    #[test]
    fn ignored_apple_notes_sources_are_disabled_and_skipped_for_sync() {
        let folder = AppleNotesFolder {
            id: Some("folder-1".to_string()),
            account_name: "iCloud".to_string(),
            folder_name: "Zoid Brain".to_string(),
        };
        let ignored = source_from_folder(&folder, false, "ignored");
        assert_eq!(ignored.sync_mode, "ignored");
        assert!(!ignored.enabled);

        let mut store = BrainStore::default();
        upsert_apple_notes_source(&mut store, brain_source("readOnly"));
        upsert_apple_notes_source(&mut store, ignored.clone());
        assert_eq!(store.sources[0].sync_mode, "ignored");
        assert!(!store.sources[0].enabled);
        assert!(apple_notes_syncable_sources(&store).is_empty());

        upsert_apple_notes_source(&mut store, source_from_folder(&folder, false, "twoWay"));
        assert_eq!(store.sources[0].sync_mode, "twoWay");
        assert!(store.sources[0].enabled);
        assert_eq!(apple_notes_syncable_sources(&store).len(), 1);
    }

    #[test]
    fn command_timeout_drains_partial_stdout_and_stderr() {
        let started_at = Instant::now();
        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("printf partial-out; printf partial-err >&2; sleep 5");
        let error = run_command_with_timeout(&mut command, Duration::from_millis(150)).unwrap_err();
        assert!(
            started_at.elapsed() < Duration::from_secs(1),
            "timeout should kill descendant processes promptly; elapsed {:?}",
            started_at.elapsed()
        );
        assert!(error.contains("Command timed out"));
        assert!(
            error.contains("partial-out"),
            "stdout should be drained: {error}"
        );
        assert!(
            error.contains("partial-err"),
            "stderr should be drained: {error}"
        );
    }

    #[test]
    fn source_sync_errors_are_recorded_without_discarding_successful_sources() {
        let mut store = BrainStore::default();
        let source_one = brain_source("readOnly");
        let mut source_two = brain_source("readOnly");
        source_two.folder_name = "Second Brain".to_string();
        source_two.id = "apple-notes:icloud:Second Brain".to_string();
        store.sources.push(source_one.clone());
        store.sources.push(source_two.clone());

        apply_apple_notes_source_sync_result(
            &mut store,
            &source_one,
            Ok(vec![raw_note("Synced", "1. Ship synced source", "200")]),
            "300",
        );
        apply_apple_notes_source_sync_result(
            &mut store,
            &source_two,
            Err("Apple Notes automation failed for this source".to_string()),
            "300",
        );

        assert_eq!(store.notes.len(), 1);
        assert_eq!(store.sources[0].last_synced_at, Some("300".to_string()));
        assert_eq!(store.sources[0].last_error, None);
        assert_eq!(
            store.sources[1].last_error.as_deref(),
            Some("Apple Notes automation failed for this source")
        );
    }

    #[test]
    fn merge_unchanged_note_as_synced() {
        let source = brain_source("twoWay");
        let raw = raw_note("Title", "Body", "200");
        let mut store = BrainStore::default();
        merge_apple_notes_raw_notes(&mut store, &source, &[raw.clone()], "300");
        merge_apple_notes_raw_notes(&mut store, &source, &[raw], "400");
        assert_eq!(store.notes.len(), 1);
        assert_eq!(store.notes[0].sync_status, "synced");
        assert_eq!(store.notes[0].last_synced_at, Some("400".to_string()));
    }

    #[test]
    fn apple_changed_note_updates_local_when_zoid_unchanged() {
        let source = brain_source("twoWay");
        let mut store = BrainStore::default();
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title", "Body", "200")],
            "300",
        );
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title 2", "Body 2", "400")],
            "500",
        );
        assert_eq!(store.notes[0].title, "Title 2");
        assert_eq!(store.notes[0].body, "Body 2");
        assert_eq!(store.notes[0].sync_status, "synced");
    }

    #[test]
    fn zoid_changed_note_is_preserved_when_apple_is_unchanged() {
        let source = brain_source("twoWay");
        let mut store = BrainStore::default();
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title", "Body", "200")],
            "300",
        );
        store.notes[0].title = "Local title".to_string();
        store.notes[0].body = "Local body".to_string();
        store.notes[0].current_hash = note_content_hash("Local title", "Local body");
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title", "Body", "200")],
            "500",
        );
        assert_eq!(store.notes[0].title, "Local title");
        assert_eq!(store.notes[0].body, "Local body");
        assert_eq!(
            store.notes[0].current_hash,
            note_content_hash("Local title", "Local body")
        );
        assert_eq!(store.notes[0].last_synced_title, "Title");
        assert_eq!(store.notes[0].last_synced_body, "Body");
        assert_eq!(store.notes[0].sync_status, "changedInZoid");
        let note_id = store.notes[0].id.clone();
        let error = extract_brain_note_in_store(&mut store, &note_id, "600").unwrap_err();
        assert!(error.contains("changedInZoid"));
    }

    #[test]
    fn apple_notes_html_body_is_normalized_before_hashing_and_extraction() {
        let normalized = normalize_apple_notes_body("<div>First&nbsp;line<br>Second &amp; link <a href=\"https://example.com\">https://example.com</a></div><ul><li>TODO: Ship thing</li><li>2. Numbered item</li></ul>");
        assert_eq!(
            normalized,
            "First line\nSecond & link https://example.com\n- TODO: Ship thing\n- 2. Numbered item"
        );
        let source = brain_source("readOnly");
        let mut store = BrainStore::default();
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note(
                "HTML",
                "<p>TODO: Review<br>https://example.com</p>",
                "200",
            )],
            "300",
        );
        assert_eq!(store.notes[0].body, "TODO: Review\nhttps://example.com");
        assert_eq!(
            store.notes[0].current_hash,
            note_content_hash("HTML", "TODO: Review\nhttps://example.com")
        );
    }

    #[test]
    fn conflict_record_is_populated_when_apple_and_zoid_both_changed() {
        let source = brain_source("twoWay");
        let mut store = BrainStore::default();
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title", "Body", "200")],
            "300",
        );
        store.notes[0].title = "Local title".to_string();
        store.notes[0].body = "Local body".to_string();
        store.notes[0].current_hash = note_content_hash("Local title", "Local body");
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Apple title", "<div>Apple body</div>", "400")],
            "500",
        );
        assert_eq!(store.notes[0].sync_status, "conflict");
        assert_eq!(store.conflicts.len(), 1);
        assert_eq!(store.conflicts[0].apple_title, "Apple title");
        assert_eq!(store.conflicts[0].apple_body, "Apple body");
        assert_eq!(store.conflicts[0].zoid_title, "Local title");
        assert_eq!(store.conflicts[0].resolved_at, None);
    }

    #[test]
    fn missing_apple_note_marked_missing_without_delete() {
        let source = brain_source("twoWay");
        let mut store = BrainStore::default();
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title", "Body", "200")],
            "300",
        );
        merge_apple_notes_raw_notes(&mut store, &source, &[], "400");
        assert_eq!(store.notes.len(), 1);
        assert_eq!(store.notes[0].sync_status, "missingInApple");
    }

    #[test]
    fn merge_preserves_read_only_and_updates_source_modes() {
        let source = brain_source("readOnly");
        let mut store = BrainStore::default();
        store.sources.push(source.clone());
        merge_apple_notes_raw_notes(
            &mut store,
            &source,
            &[raw_note("Title", "Body", "200")],
            "300",
        );
        assert_eq!(store.sources[0].sync_mode, "readOnly");
        let mut two_way = source.clone();
        two_way.sync_mode = "twoWay".to_string();
        upsert_apple_notes_source(&mut store, two_way);
        assert_eq!(store.sources[0].sync_mode, "twoWay");
    }

    #[test]
    #[ignore = "mutates macOS Notes by creating and deleting a disposable E2E folder"]
    fn apple_notes_real_e2e_sync_extracts_tasks_and_persists_store() {
        let _guard = env_lock();
        let root = unique_temp_path("brain-real-notes-e2e");
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        let folder_name = format!("Zoid Brain E2E {}", now_millis_string());
        let folder_name_json = jxa_json_string_literal(&folder_name);
        let create_script = format!(
            r#"
const Notes = Application('Notes');
Notes.includeStandardAdditions = true;
const folderName = {folder_name_json};
const account = Notes.defaultAccount ? Notes.defaultAccount() : Notes.accounts()[0];
let target = null;
for (const folder of account.folders()) {{ if (folder.name() === folderName) {{ target = folder; break; }} }}
if (!target) {{ target = Notes.Folder({{name: folderName}}); account.folders.push(target); }}
const note = Notes.Note({{name: 'Zoid Brain E2E Tasks', body: '<div>1. Draft launch memo for Zoid Brain<br>2. Review Apple Notes task extraction with Maya<br>Maybe clarify deadline?</div>'}});
target.notes.push(note);
let id = null;
try {{ id = target.id(); }} catch (e) {{}}
JSON.stringify({{accountName: account.name(), folderName: target.name(), id}});
"#
        );
        let created_raw =
            run_apple_notes_script(&create_script).expect("create disposable Notes folder/note");
        let created: AppleNotesFolder =
            serde_json::from_str(&created_raw).expect("created folder JSON");

        let folders = list_apple_notes_folders_inner().expect("list Apple Notes folders");
        assert!(folders
            .iter()
            .any(|folder| folder.account_name == created.account_name
                && folder.folder_name == created.folder_name));

        let source = link_apple_notes_folder_inner(
            created.account_name.clone(),
            created.folder_name.clone(),
            "readOnly".to_string(),
        )
        .expect("link disposable folder");
        assert_eq!(source.sync_mode, "readOnly");
        assert!(source.enabled);

        let synced = sync_apple_notes_sources_inner().expect("sync disposable folder");
        let note = synced
            .notes
            .iter()
            .find(|note| note.source_id == source.id && note.title == "Zoid Brain E2E Tasks")
            .expect("synced disposable note");
        assert!(note.body.contains("Draft launch memo"));
        assert!(note.body.contains("Review Apple Notes task extraction"));
        let note_id = note.id.clone();

        let extracted = extract_brain_note_inner(&note_id).expect("extract synced note");
        let candidates = extracted
            .task_candidates
            .iter()
            .filter(|candidate| candidate.note_id == note_id)
            .collect::<Vec<_>>();
        assert_eq!(candidates.len(), 2);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.title.contains("Draft launch memo")));
        assert!(candidates.iter().any(|candidate| candidate
            .title
            .contains("Review Apple Notes task extraction")));

        let candidate_ids = candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect::<Vec<_>>();
        let clarified = create_brain_clarifying_session_inner(&note_id, candidate_ids)
            .expect("create clarifying session");
        assert_eq!(clarified.clarification_sessions.len(), 1);
        assert_eq!(clarified.clarification_sessions[0].status, "questioning");

        let reloaded = load_brain_store_inner().expect("reload persisted Brain store");
        assert_eq!(
            reloaded
                .notes
                .iter()
                .filter(|note| note.source_id == source.id)
                .count(),
            1
        );
        assert_eq!(
            reloaded
                .task_candidates
                .iter()
                .filter(|candidate| candidate.note_id == note_id)
                .count(),
            2
        );
        assert_eq!(reloaded.clarification_sessions.len(), 1);

        let cleanup_folder_name = jxa_json_string_literal(&created.folder_name);
        let cleanup_account_name = jxa_json_string_literal(&created.account_name);
        let cleanup_script = format!(
            r#"
const Notes = Application('Notes');
const accountName = {cleanup_account_name};
const folderName = {cleanup_folder_name};
for (const account of Notes.accounts()) {{
  if (account.name() !== accountName) continue;
  for (const folder of account.folders()) {{
    if (folder.name() === folderName) {{ try {{ folder.delete(); }} catch (e) {{}} }}
  }}
}}
'cleanup';
"#
        );
        let _ = run_apple_notes_script(&cleanup_script);
        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn brain_extraction_splits_multiple_numbered_tasks_from_one_note() {
        let mut store = BrainStore::default();
        store.notes.push(brain_note(
            "note-1",
            "Launch",
            "1. Draft launch memo for Zoid Brain
2. Review pricing page with Maya
Some context #launch https://example.com",
        ));
        extract_brain_note_in_store(&mut store, "note-1", "1000").unwrap();
        assert_eq!(store.extractions.len(), 1);
        assert_eq!(store.extractions[0].extractor, "localHeuristic");
        assert!(store.extractions[0].topics.contains(&"launch".to_string()));
        assert!(store.extractions[0]
            .references
            .contains(&"https://example.com".to_string()));
        assert_eq!(store.task_candidates.len(), 2);
        assert!(store
            .task_candidates
            .iter()
            .any(|candidate| candidate.title.contains("Draft launch memo")));
        assert!(store
            .task_candidates
            .iter()
            .all(|candidate| candidate.status == "needsReview"
                || candidate.status == "needsClarification"));
    }

    #[test]
    fn brain_extraction_rejects_stale_missing_and_conflicted_notes() {
        for status in [
            "changedInApple",
            "changedInZoid",
            "conflict",
            "missingInApple",
        ] {
            let mut store = BrainStore::default();
            let mut note = brain_note("note-stale", "Stale", "1. Should not extract");
            note.sync_status = status.to_string();
            store.notes.push(note);
            let error = extract_brain_note_in_store(&mut store, "note-stale", "1000").unwrap_err();
            assert!(
                error.contains(status),
                "error should name stale sync status: {error}"
            );
            assert!(store.extractions.is_empty());
            assert!(store.task_candidates.is_empty());
        }
    }

    #[test]
    fn brain_extraction_falls_back_to_note_title_when_no_imperative_lines() {
        let mut store = BrainStore::default();
        let note = brain_note(
            "note-fallback",
            "Prepare investor follow-up",
            "Notes from the meeting need a careful follow-up with owners and dates.",
        );
        let note_id = note.id.clone();
        store.notes.push(note);

        extract_brain_note_in_store(&mut store, &note_id, "300").expect("extract fallback task");

        let candidates = store
            .task_candidates
            .iter()
            .filter(|candidate| candidate.note_id == note_id)
            .collect::<Vec<_>>();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].title, "Prepare investor follow-up");
    }

    #[test]
    fn brain_extraction_does_not_create_candidate_from_empty_untitled_note() {
        let mut store = BrainStore::default();
        store
            .notes
            .push(brain_note("note-empty", "Untitled", "   "));

        extract_brain_note_in_store(&mut store, "note-empty", "301").expect("extract empty note");

        assert_eq!(store.extractions.len(), 1);
        assert!(store.task_candidates.is_empty());
        assert!(store.extractions[0]
            .open_questions
            .iter()
            .any(|question| question.contains("outcome")));
    }

    #[test]
    fn scribbly_note_needs_clarification_and_does_not_become_ready() {
        let mut store = BrainStore::default();
        store.notes.push(brain_note(
            "note-scribble",
            "maybe",
            "thing for Zoid later ??? tbd",
        ));
        extract_brain_note_in_store(&mut store, "note-scribble", "1000").unwrap();
        assert!(store.extractions[0].ambiguity_score >= 0.5);
        assert!(!store.extractions[0].open_questions.is_empty());
        assert!(store
            .task_candidates
            .iter()
            .all(|candidate| candidate.status != "readyForAgent"));
    }

    #[test]
    fn rerun_extraction_replaces_draft_candidates_but_keeps_sent_or_done() {
        let mut store = BrainStore::default();
        store.notes.push(brain_note(
            "note-1",
            "Tasks",
            "1. First task
2. Second task",
        ));
        store.task_candidates.push(TaskCandidate {
            id: "old-draft".to_string(),
            note_id: "note-1".to_string(),
            title: "Old draft".to_string(),
            extracted_description: "Old draft".to_string(),
            status: "needsReview".to_string(),
            priority_guess: "normal".to_string(),
            readiness_score: 0.5,
            clarification_session_id: None,
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        });
        store.task_candidates.push(TaskCandidate {
            id: "sent".to_string(),
            note_id: "note-1".to_string(),
            title: "Already sent".to_string(),
            extracted_description: "Already sent".to_string(),
            status: "sentToAgent".to_string(),
            priority_guess: "normal".to_string(),
            readiness_score: 0.9,
            clarification_session_id: None,
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        });
        extract_brain_note_in_store(&mut store, "note-1", "1000").unwrap();
        assert!(!store
            .task_candidates
            .iter()
            .any(|candidate| candidate.id == "old-draft"));
        assert!(store
            .task_candidates
            .iter()
            .any(|candidate| candidate.id == "sent"));
        assert_eq!(
            store
                .task_candidates
                .iter()
                .filter(|candidate| candidate.note_id == "note-1")
                .count(),
            3
        );
    }

    #[test]
    fn clarifying_session_links_candidates_and_starts_with_assistant_questions() {
        let mut store = BrainStore::default();
        store.notes.push(brain_note(
            "note-1",
            "Tasks",
            "TODO: Draft unclear launch thing tbd",
        ));
        extract_brain_note_in_store(&mut store, "note-1", "1000").unwrap();
        let candidate_id = store.task_candidates[0].id.clone();
        create_brain_clarifying_session_in_store(
            &mut store,
            "note-1",
            vec![candidate_id.clone()],
            "1001",
        )
        .unwrap();
        assert_eq!(store.clarification_sessions.len(), 1);
        let session = &store.clarification_sessions[0];
        assert_eq!(session.status, "questioning");
        assert!(session
            .open_questions
            .iter()
            .any(|question| question.contains("success criteria")));
        assert_eq!(session.transcript[0].role, "assistant");
        assert!(session.transcript[0]
            .content
            .contains("will not execute it yet"));
        assert_eq!(
            store
                .task_candidates
                .iter()
                .find(|candidate| candidate.id == candidate_id)
                .unwrap()
                .clarification_session_id
                .as_deref(),
            Some(session.id.as_str())
        );
    }

    #[test]
    fn answering_clarifying_questions_generates_brief_without_agent_execution() {
        let mut store = BrainStore::default();
        store.notes.push(brain_note(
            "note-1",
            "Launch note",
            "TODO: Draft unclear launch thing tbd",
        ));
        extract_brain_note_in_store(&mut store, "note-1", "1000").unwrap();
        let candidate_id = store.task_candidates[0].id.clone();
        create_brain_clarifying_session_in_store(
            &mut store,
            "note-1",
            vec![candidate_id.clone()],
            "1001",
        )
        .unwrap();
        let session_id = store.clarification_sessions[0].id.clone();
        while !store.clarification_sessions[0].open_questions.is_empty() {
            answer_brain_clarifying_session_in_store(
                &mut store,
                &session_id,
                "Outcome: launch memo. Owner: Ziad. Deadline: Friday. Acceptance: concise plan.",
                "1002",
            )
            .unwrap();
        }
        let session = &store.clarification_sessions[0];
        assert_eq!(session.status, "briefReady");
        assert!(session.resolved_brief.contains("# Agent Brief"));
        assert!(session.resolved_brief.contains("Launch note"));
        assert!(session
            .resolved_brief
            .contains("Execution rule: Do not run automatically"));
        assert_eq!(session.hermes_session_id, None);
        assert_eq!(
            store
                .task_candidates
                .iter()
                .find(|candidate| candidate.id == candidate_id)
                .unwrap()
                .status,
            "readyForAgent"
        );
    }

    #[test]
    fn brain_store_default_serializes_and_deserializes_empty_collections() {
        let store = BrainStore::default();
        assert_eq!(store.version, 1);
        assert!(store.sources.is_empty());
        assert!(store.notes.is_empty());
        assert!(store.extractions.is_empty());
        assert!(store.task_candidates.is_empty());
        assert!(store.clarification_sessions.is_empty());
        assert!(store.conflicts.is_empty());

        let raw = serde_json::to_string(&store).unwrap();
        assert!(raw.contains("taskCandidates"));
        let round_trip: BrainStore = serde_json::from_str(&raw).unwrap();
        assert_eq!(round_trip, store);
    }

    #[test]
    fn task_candidate_status_round_trips_as_string_backed_field() {
        let candidate = TaskCandidate {
            id: "task-1".to_string(),
            note_id: "note-1".to_string(),
            title: "Clarify launch plan".to_string(),
            extracted_description: "Launch needs owner and deadline.".to_string(),
            status: "needsClarification".to_string(),
            priority_guess: "high".to_string(),
            readiness_score: 0.42,
            clarification_session_id: Some("session-1".to_string()),
            created_at: "100".to_string(),
            updated_at: "101".to_string(),
        };
        let raw = serde_json::to_string(&candidate).unwrap();
        assert!(raw.contains("needsClarification"));
        assert_eq!(
            serde_json::from_str::<TaskCandidate>(&raw).unwrap(),
            candidate
        );
    }

    #[test]
    fn brain_note_sync_status_round_trips_as_string_backed_field() {
        let note = BrainNote {
            id: "note-1".to_string(),
            source_type: "appleNotes".to_string(),
            source_id: "source-1".to_string(),
            apple_note_id: "apple-1".to_string(),
            title: "Idea".to_string(),
            body: "Body".to_string(),
            source_folder: "Zoid Brain".to_string(),
            account_name: "iCloud".to_string(),
            apple_created_at: None,
            apple_modified_at: Some("200".to_string()),
            zoid_modified_at: None,
            imported_at: "201".to_string(),
            last_synced_at: None,
            last_synced_title: "Idea".to_string(),
            last_synced_body: "Body".to_string(),
            last_synced_hash: "abc".to_string(),
            current_hash: "def".to_string(),
            sync_status: "changedInApple".to_string(),
            archived: false,
        };
        let raw = serde_json::to_string(&note).unwrap();
        assert!(raw.contains("changedInApple"));
        assert_eq!(serde_json::from_str::<BrainNote>(&raw).unwrap(), note);
    }

    #[test]
    fn missing_brain_store_file_returns_default_profile_store() {
        let _guard = env_lock();
        let root = unique_temp_path("brain-missing");
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        let store = load_brain_store_inner().unwrap();
        assert_eq!(store, BrainStore::default());
        assert_eq!(brain_storage_path().unwrap(), root.join("zoid-brain.json"));
        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn brain_store_save_then_load_preserves_sources_notes_and_candidates() {
        let _guard = env_lock();
        let root = unique_temp_path("brain-round-trip");
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        let mut store = BrainStore::default();
        store.sources.push(AppleNotesSource {
            id: "source-1".to_string(),
            source_type: "appleNotes".to_string(),
            account_name: "iCloud".to_string(),
            folder_name: "Zoid Brain".to_string(),
            sync_mode: "twoWay".to_string(),
            enabled: true,
            created_by_zoid: true,
            last_synced_at: None,
            last_error: None,
        });
        store.notes.push(BrainNote {
            id: "note-1".to_string(),
            source_type: "appleNotes".to_string(),
            source_id: "source-1".to_string(),
            apple_note_id: "apple-1".to_string(),
            title: "T".to_string(),
            body: "B".to_string(),
            source_folder: "Zoid Brain".to_string(),
            account_name: "iCloud".to_string(),
            apple_created_at: None,
            apple_modified_at: None,
            zoid_modified_at: None,
            imported_at: "1".to_string(),
            last_synced_at: None,
            last_synced_title: "T".to_string(),
            last_synced_body: "B".to_string(),
            last_synced_hash: "h1".to_string(),
            current_hash: "h1".to_string(),
            sync_status: "synced".to_string(),
            archived: false,
        });
        store.task_candidates.push(TaskCandidate {
            id: "task-1".to_string(),
            note_id: "note-1".to_string(),
            title: "Do it".to_string(),
            extracted_description: "desc".to_string(),
            status: "readyForAgent".to_string(),
            priority_guess: "normal".to_string(),
            readiness_score: 0.9,
            clarification_session_id: None,
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        });
        save_brain_store_inner(&store).unwrap();
        save_brain_store_inner(&store).unwrap();
        let loaded = load_brain_store_inner().unwrap();
        assert_eq!(loaded, store);
        assert!(fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .contains("zoid-brain.json.zoid-brain-save")));
        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_brain_store_json_returns_readable_error() {
        let _guard = env_lock();
        let root = unique_temp_path("brain-malformed");
        fs::create_dir_all(&root).unwrap();
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        fs::write(root.join("zoid-brain.json"), "{not-json").unwrap();
        let error = load_brain_store_inner().unwrap_err();
        assert!(error.contains("Failed to parse Brain store"));
        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn jxa_json_string_literal_escapes_user_text_without_script_interpolation() {
        let escaped = jxa_json_string_literal("quote \" slash \\ newline\nemoji 🧠");
        assert_eq!(
            serde_json::from_str::<String>(&escaped).unwrap(),
            "quote \" slash \\ newline\nemoji 🧠"
        );
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
        assert!(escaped.contains("\\n"));
    }

    #[test]
    fn file_manager_listing_is_lazy_and_finder_sorted() {
        let root = unique_temp_path("file-manager");
        fs::create_dir_all(root.join("Projects")).unwrap();
        fs::create_dir_all(root.join("Documents")).unwrap();
        fs::write(root.join("notes.txt"), "hello").unwrap();
        fs::write(root.join(".hidden"), "secret").unwrap();
        fs::write(root.join("Projects").join("child.md"), "child").unwrap();
        fs::write(root.join("Documents").join("protected.md"), "protected").unwrap();

        let listing =
            list_file_manager_directory_inner(Some(root.to_string_lossy().to_string())).unwrap();
        assert_eq!(listing.path, root.canonicalize().unwrap().to_string_lossy());
        assert!(listing
            .entries
            .iter()
            .all(|entry| !entry.name.starts_with('.')));
        assert_eq!(listing.entries[0].name, "Documents");
        assert_eq!(listing.entries[0].kind, "directory");
        assert_eq!(listing.entries[0].children_count, None);
        assert_eq!(listing.entries[1].name, "Projects");
        assert_eq!(listing.entries[1].kind, "directory");
        assert_eq!(listing.entries[1].children_count, Some(1));
        assert_eq!(listing.entries[2].name, "notes.txt");
        assert_eq!(listing.entries[2].kind, "file");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn file_manager_listing_does_not_fail_when_permission_marker_cannot_persist() {
        let _guard = env_lock();
        let root = unique_temp_path("file-manager-marker-denied");
        let blocked_home = root.join("blocked-hermes-home");
        fs::create_dir_all(root.join("visible")).unwrap();
        fs::write(&blocked_home, "not a directory").unwrap();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &blocked_home);

        let listing = list_file_manager_directory_inner(Some(root.to_string_lossy().to_string())).unwrap();
        let expected_path = root.canonicalize().unwrap_or_else(|_| root.clone()).to_string_lossy().to_string();

        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);
        assert_eq!(listing.path, expected_path);
    }

    #[test]
    fn scan_repository_folder_does_not_fail_when_permission_marker_cannot_persist() {
        let _guard = env_lock();
        let root = unique_temp_path("repo-scan-marker-denied");
        let blocked_home = root.join("blocked-hermes-home");
        let repo = root.join("repo");
        init_git_repo(&repo);
        fs::write(&blocked_home, "not a directory").unwrap();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &blocked_home);

        let repositories = scan_repository_folder(root.to_str().unwrap()).unwrap();

        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);
        assert_eq!(repositories.len(), 1);
    }

    #[test]
    fn github_branch_lookup_does_not_fail_when_permission_marker_cannot_persist() {
        let _guard = env_lock();
        let root = unique_temp_path("branch-marker-denied");
        let blocked_home = root.join("blocked-hermes-home");
        let repo = root.join("repo");
        init_git_repo(&repo);
        let mut remote = Command::new("git");
        remote.arg("-C").arg(&repo).args([
            "remote",
            "add",
            "origin",
            "https://github.com/nousresearch/hermes-agent.git",
        ]);
        let (remote_ok, _, remote_stderr) =
            run_command_with_timeout(&mut remote, Duration::from_secs(20)).unwrap();
        assert!(remote_ok, "git remote add failed: {remote_stderr}");
        let fake_gh = root.join("gh");
        fs::write(&fake_gh, "#!/bin/sh\nif [ \"$1\" = \"api\" ]; then printf 'main\\n'; exit 0; fi\nexit 1\n").unwrap();
        let mut permissions = fs::metadata(&fake_gh).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_gh, permissions).unwrap();
        fs::write(&blocked_home, "not a directory").unwrap();
        let previous_path = std::env::var("PATH").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("PATH", format!("{}:{}", root.display(), previous_path.clone().unwrap_or_default()));
        std::env::set_var("HERMES_HOME", &blocked_home);

        let branches = list_remote_branches(
            repo.to_str().unwrap(),
            Some("https://github.com/nousresearch/hermes-agent.git".to_string()),
            Some("main".to_string()),
        )
        .unwrap();

        if let Some(previous_path) = previous_path {
            std::env::set_var("PATH", previous_path);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);
        assert_eq!(branches.len(), 1);
    }

    #[test]
    fn default_session_name_is_stable() {
        let _guard = env_lock();
        std::env::remove_var("ZOID_HERMES_SESSION");
        assert_eq!(hermes_session_name(), DEFAULT_HERMES_SESSION);
    }

    #[test]
    fn terminal_noise_is_removed_from_cli_output() {
        let raw = "hello from Hermes\nSession ID: abc\nCost: $0.01";
        assert_eq!(strip_terminal_noise(raw), "hello from Hermes");
    }

    #[test]
    fn linked_repository_workdir_requires_existing_directory() {
        let temp_dir = std::env::temp_dir();
        let resolved =
            resolve_linked_repository_workdir(Some(temp_dir.to_string_lossy().to_string()))
                .unwrap();
        assert_eq!(resolved, Some(temp_dir));
        assert!(
            resolve_linked_repository_workdir(Some("/definitely/not/a/zoid/repo".to_string()))
                .is_err()
        );
        assert_eq!(
            resolve_linked_repository_workdir(Some("Unlinked".to_string())).unwrap(),
            None
        );
    }

    #[test]
    fn hermes_slash_registry_json_is_annotated_for_zoid_behavior() {
        let commands = parse_hermes_registry_json(r#"[
            {"name":"model","aliases":["m"],"description":"Switch model","category":"Model","argsHint":"[provider/model]","subcommands":[],"cliOnly":false,"gatewayOnly":false,"zoidBehavior":"forward","panel":null},
            {"name":"redraw","aliases":[],"description":"Redraw TUI","category":"Display","argsHint":null,"subcommands":[],"cliOnly":true,"gatewayOnly":false,"zoidBehavior":"forward","panel":null},
            {"name":"help","aliases":["?"],"description":"Show help","category":"Session","argsHint":null,"subcommands":[],"cliOnly":false,"gatewayOnly":false,"zoidBehavior":"forward","panel":null}
        ]"#).unwrap();
        let model = commands
            .iter()
            .find(|command| command.name == "model")
            .unwrap();
        assert_eq!(model.zoid_behavior, "native-panel");
        assert_eq!(model.panel.as_deref(), Some("model"));
        assert_eq!(
            commands
                .iter()
                .find(|command| command.name == "redraw")
                .unwrap()
                .zoid_behavior,
            "noop"
        );
        assert_eq!(resolve_slash_command("/?", &commands).unwrap().name, "help");
    }

    #[test]
    fn hermes_slash_execution_handles_zoid_native_session_semantics_before_forwarding() {
        let clear =
            execute_hermes_slash_command_inner("/clear", None, Some("abc".to_string()), false)
                .unwrap();
        assert_eq!(clear.kind, "new-session");
        assert!(clear.session.unwrap().starts_with("zoid-session-"));

        let quit =
            execute_hermes_slash_command_inner("/quit", None, Some("abc".to_string()), false)
                .unwrap();
        assert_eq!(quit.kind, "close-session");

        let redraw =
            execute_hermes_slash_command_inner("/redraw", None, Some("abc".to_string()), false)
                .unwrap();
        assert_eq!(redraw.kind, "text");
        assert_eq!(redraw.content.as_deref(), Some("Not needed in Zoid."));
    }

    #[test]
    fn hermes_slash_execution_requires_confirmation_for_sensitive_commands() {
        let restart =
            execute_hermes_slash_command_inner("/restart", None, Some("abc".to_string()), false)
                .unwrap();
        assert_eq!(restart.kind, "confirmation");
        assert!(restart.requires_confirmation);
        assert_eq!(restart.scope, "global-hermes");

        let restore = execute_hermes_slash_command_inner(
            "/snapshot restore latest",
            None,
            Some("abc".to_string()),
            false,
        )
        .unwrap();
        assert_eq!(restore.kind, "confirmation");
        assert!(restore.requires_confirmation);
    }

    #[test]
    fn git_repository_detection_accepts_git_directory_and_rejects_plain_folder() {
        let root = unique_temp_path("repo-detection");
        let repo = root.join("repo");
        let plain = root.join("plain");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(&plain).unwrap();

        assert!(is_git_repository(&repo));
        assert!(!is_git_repository(&plain));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn repository_id_is_stable_for_same_path() {
        let root = unique_temp_path("repo-id");
        fs::create_dir_all(&root).unwrap();
        assert_eq!(repository_id(&root), repository_id(&root));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_repository_folder_returns_nested_git_repositories_without_duplicates() {
        let root = unique_temp_path("repo-scan");
        let repo_a = root.join("repo-a");
        let nested_parent = root.join("nested");
        let repo_b = nested_parent.join("repo-b");
        let plain = root.join("plain");
        init_git_repo(&repo_a);
        init_git_repo(&repo_b);
        fs::create_dir_all(&plain).unwrap();

        let repositories = scan_repository_folder(root.to_str().unwrap()).unwrap();
        let paths = repositories
            .iter()
            .map(|repo| repo.path.clone())
            .collect::<Vec<_>>();
        assert_eq!(repositories.len(), 2);
        assert!(paths.iter().any(|path| path.ends_with("repo-a")));
        assert!(paths.iter().any(|path| path.ends_with("repo-b")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_repository_folder_rejects_missing_folder() {
        let missing = unique_temp_path("missing-scan-folder");
        assert!(scan_repository_folder(missing.to_str().unwrap()).is_err());
    }

    #[test]
    fn github_repo_folder_name_accepts_common_github_links() {
        assert_eq!(
            github_repo_folder_name("https://github.com/nousresearch/hermes-agent.git").unwrap(),
            "hermes-agent"
        );
        assert_eq!(
            github_repo_folder_name("git@github.com:nousresearch/hermes-agent.git").unwrap(),
            "hermes-agent"
        );
        assert!(github_repo_folder_name("https://example.com/nousresearch/hermes-agent").is_err());
    }

    #[test]
    fn github_branch_lookup_uses_gh_api_and_marks_default() {
        let _guard = env_lock();
        let root = unique_temp_path("branch-lookup");
        let repo = root.join("repo");
        init_git_repo(&repo);
        let mut remote = Command::new("git");
        remote.arg("-C").arg(&repo).args([
            "remote",
            "add",
            "origin",
            "https://github.com/nousresearch/hermes-agent.git",
        ]);
        let (remote_ok, _, remote_stderr) =
            run_command_with_timeout(&mut remote, Duration::from_secs(20)).unwrap();
        assert!(remote_ok, "git remote add failed: {remote_stderr}");
        let mut set_head = Command::new("git");
        set_head.arg("-C").arg(&repo).args([
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ]);
        let (head_ok, _, head_stderr) =
            run_command_with_timeout(&mut set_head, Duration::from_secs(20)).unwrap();
        assert!(head_ok, "git symbolic-ref failed: {head_stderr}");
        let fake_gh = root.join("gh");
        fs::write(
            &fake_gh,
            "#!/bin/sh\nif [ \"$1\" = \"api\" ]; then printf 'main\\ndevelop\\n'; exit 0; fi\necho unexpected >&2\nexit 1\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_gh).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_gh, permissions).unwrap();
        let previous_path = std::env::var("PATH").ok();
        std::env::set_var(
            "PATH",
            format!(
                "{}:{}",
                root.display(),
                previous_path.clone().unwrap_or_default()
            ),
        );
        let branches = list_remote_branches(
            repo.to_str().unwrap(),
            Some("https://github.com/nousresearch/hermes-agent.git".to_string()),
            Some("main".to_string()),
        )
        .unwrap();
        if let Some(previous_path) = previous_path {
            std::env::set_var("PATH", previous_path);
        } else {
            std::env::remove_var("PATH");
        }
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            branches
                .iter()
                .map(|branch| branch.name.as_str())
                .collect::<Vec<_>>(),
            vec!["main", "develop"]
        );
    }

    #[test]
    fn permission_warmup_treats_touched_home_as_app_wide_root() {
        let _guard = env_lock();
        let root = unique_temp_path("permission-root-scope");
        let home = root.join("home");
        let hermes_home = root.join("hermes-home");
        fs::create_dir_all(home.join("Documents")).unwrap();
        fs::create_dir_all(home.join("Desktop")).unwrap();
        fs::create_dir_all(home.join("Downloads")).unwrap();
        fs::create_dir_all(home.join("Zoid")).unwrap();
        fs::create_dir_all(&hermes_home).unwrap();
        let previous_home = std::env::var("HOME").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HOME", &home);
        std::env::set_var("HERMES_HOME", &hermes_home);

        let touched = warm_file_permissions_inner(false).unwrap();
        assert!(touched
            .iter()
            .any(|path| path == &home.canonicalize().unwrap().to_string_lossy()));
        assert!(!touched.iter().any(|path| path.ends_with("/Documents") || path.ends_with("/Desktop") || path.ends_with("/Downloads")), "warmup should not ask again for child folders once the app has home-root access: {touched:?}");
        assert!(raw_path_is_covered_by_touched_root(
            &home.join("Documents"),
            &touched.iter().cloned().collect::<HashSet<_>>()
        ));

        if let Some(previous_home) = previous_home {
            std::env::set_var("HOME", previous_home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn branch_remote_metadata_paths_do_not_retouch_local_folder_permissions() {
        let _guard = env_lock();
        let root = unique_temp_path("branch-no-local-permission-touch");
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        let missing_repo = root.join("repo-that-should-not-be-read");
        let missing_repo = missing_repo.to_string_lossy().to_string();

        let list_error = list_remote_branches(
            &missing_repo,
            Some("https://example.com/not-github/repo.git".to_string()),
            Some("main".to_string()),
        )
        .unwrap_err();
        assert!(
            list_error.contains("origin remote on GitHub"),
            "unexpected list error: {list_error}"
        );
        assert!(
            !list_error.contains("Zoid could not access"),
            "branch lookup touched local files despite remote metadata: {list_error}"
        );

        let update_error = update_default_branch(
            &missing_repo,
            Some("https://example.com/not-github/repo.git".to_string()),
            "main",
        )
        .unwrap_err();
        assert!(
            update_error.contains("Default branch editing")
                || update_error.contains("origin remote on GitHub"),
            "unexpected update error: {update_error}"
        );
        assert!(
            !update_error.contains("Zoid could not access"),
            "default-branch update touched local files despite remote metadata: {update_error}"
        );

        let marker = load_file_permission_marker(&file_permission_bootstrap_path().unwrap());
        assert!(marker
            .remembered_paths
            .iter()
            .any(|path| path == &missing_repo));
        assert!(!marker
            .touched_paths
            .iter()
            .any(|path| path == &missing_repo));
        let direct_error = remember_file_permission_path(Path::new(&missing_repo)).unwrap_err();
        assert!(direct_error.contains("Zoid could not access"), "remote-only remembered path should not satisfy direct filesystem access: {direct_error}");

        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn default_branch_update_with_remote_metadata_does_not_canonicalize_local_path() {
        let _guard = env_lock();
        let root = unique_temp_path("branch-update-no-local-touch");
        let fake_gh = root.join("gh");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &fake_gh,
            "#!/bin/sh\nif [ \"$1\" = \"repo\" ] && [ \"$2\" = \"edit\" ]; then exit 0; fi\necho unexpected >&2\nexit 1\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_gh).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_gh, permissions).unwrap();
        let hermes_home = root.join("hermes-home");
        let missing_repo = root.join("missing-repo");
        let missing_repo = missing_repo.to_string_lossy().to_string();
        let previous_path = std::env::var("PATH").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var(
            "PATH",
            format!(
                "{}:{}",
                root.display(),
                previous_path.clone().unwrap_or_default()
            ),
        );
        std::env::set_var("HERMES_HOME", &hermes_home);

        let updated = update_default_branch(
            &missing_repo,
            Some("https://github.com/nousresearch/hermes-agent.git".to_string()),
            "main",
        )
        .unwrap();

        assert_eq!(updated.path, missing_repo);
        assert_eq!(updated.default_branch.as_deref(), Some("main"));
        let marker = load_file_permission_marker(&file_permission_bootstrap_path().unwrap());
        assert!(marker
            .remembered_paths
            .iter()
            .any(|path| path == &updated.path));
        assert!(!marker
            .touched_paths
            .iter()
            .any(|path| path == &updated.path));

        if let Some(previous_path) = previous_path {
            std::env::set_var("PATH", previous_path);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hermes_cli_message_runs_inside_linked_repository() {
        let _guard = env_lock();
        let root = unique_temp_path("hermes-workdir");
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        let fake_hermes = root.join("hermes");
        fs::write(
            &fake_hermes,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'hermes fake'; exit 0; fi\nprintf 'workdir:%s\\n' \"$PWD\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_hermes, permissions).unwrap();

        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        std::env::set_var("ZOID_HERMES_CLI", &fake_hermes);
        let response = tauri::async_runtime::block_on(commands::send_hermes_cli_message_for_test(
            vec![HermesCliMessage {
                role: "user".to_string(),
                content: "pwd".to_string(),
            }],
            Some(repo.to_string_lossy().to_string()),
            None,
        ))
        .unwrap();
        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let expected_repo = repo.canonicalize().unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(response
            .content
            .contains(&format!("workdir:{}", expected_repo.display())));
        assert!(!response.content.contains("Terminal command"));
        assert!(!response.content.contains("hermes chat"));
    }

    #[test]
    fn hermes_cli_allows_overlapping_runs_for_different_sessions() {
        let _guard = env_lock();
        let root = unique_temp_path("hermes-parallel-different-sessions");
        fs::create_dir_all(&root).unwrap();
        let fake_hermes = root.join("hermes");
        let started_a = root.join("started-a");
        fs::write(
            &fake_hermes,
            format!(
                "#!/bin/bash\nif [ \"$1\" = \"--version\" ]; then echo 'hermes fake'; exit 0; fi\nprompt=\"${{@: -1}}\"\nif [ \"$prompt\" = \"slow-a\" ]; then touch '{}'; sleep 1; echo done-a; exit 0; fi\necho done-$prompt\n",
                started_a.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_hermes, permissions).unwrap();

        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        std::env::set_var("ZOID_HERMES_CLI", &fake_hermes);
        let first = thread::spawn(|| {
            tauri::async_runtime::block_on(commands::send_hermes_cli_run_message_for_test(
                vec![HermesCliMessage {
                    role: "user".to_string(),
                    content: "hermes chat --query slow-a".to_string(),
                }],
                None,
                None,
                Some("session-a".to_string()),
                Some("run-a".to_string()),
            ))
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while !started_a.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(started_a.exists(), "first fake Hermes run did not start");
        let second = tauri::async_runtime::block_on(commands::send_hermes_cli_run_message_for_test(
            vec![HermesCliMessage {
                role: "user".to_string(),
                content: "hermes chat --query fast-b".to_string(),
            }],
            None,
            None,
            Some("session-b".to_string()),
            Some("run-b".to_string()),
        ))
        .unwrap();
        let first = first.join().unwrap().unwrap();

        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let _ = fs::remove_dir_all(&root);
        assert!(first.content.contains("done-a"));
        assert!(second.content.contains("done-fast-b"));
    }

    #[test]
    fn hermes_cli_rejects_overlapping_runs_in_same_session() {
        let _guard = env_lock();
        let root = unique_temp_path("hermes-parallel-same-session");
        fs::create_dir_all(&root).unwrap();
        let fake_hermes = root.join("hermes");
        let started = root.join("started");
        fs::write(
            &fake_hermes,
            format!("#!/bin/bash\nif [ \"$1\" = \"--version\" ]; then echo 'hermes fake'; exit 0; fi\ntouch '{}'; sleep 1; echo done\n", started.display()),
        ).unwrap();
        let mut permissions = fs::metadata(&fake_hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_hermes, permissions).unwrap();
        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        std::env::set_var("ZOID_HERMES_CLI", &fake_hermes);
        let first = thread::spawn(|| {
            tauri::async_runtime::block_on(commands::send_hermes_cli_run_message_for_test(
                vec![HermesCliMessage {
                    role: "user".to_string(),
                    content: "hermes chat --query slow".to_string(),
                }],
                None,
                None,
                Some("same-session".to_string()),
                Some("run-1".to_string()),
            ))
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while !started.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(started.exists(), "first fake Hermes run did not start");
        let rejected = tauri::async_runtime::block_on(commands::send_hermes_cli_run_message_for_test(
            vec![HermesCliMessage {
                role: "user".to_string(),
                content: "hermes chat --query second".to_string(),
            }],
            None,
            None,
            Some("same-session".to_string()),
            Some("run-2".to_string()),
        ))
        .unwrap_err();
        assert!(
            rejected.contains("already responding in this session"),
            "unexpected rejection: {rejected}"
        );
        first.join().unwrap().unwrap();
        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hermes_cli_cancellation_is_scoped_to_session_and_run() {
        let _guard = env_lock();
        let root = unique_temp_path("hermes-cancel-scoped");
        fs::create_dir_all(&root).unwrap();
        let fake_hermes = root.join("hermes");
        let started_a = root.join("started-a");
        let started_b = root.join("started-b");
        fs::write(&fake_hermes, format!("#!/bin/bash\nif [ \"$1\" = \"--version\" ]; then echo 'hermes fake'; exit 0; fi\nprompt=\"${{@: -1}}\"\nif [ \"$prompt\" = \"a\" ]; then touch '{}'; sleep 10; echo done-a; exit 0; fi\ntouch '{}'; sleep 1; echo done-b\n", started_a.display(), started_b.display())).unwrap();
        let mut permissions = fs::metadata(&fake_hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_hermes, permissions).unwrap();
        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        std::env::set_var("ZOID_HERMES_CLI", &fake_hermes);
        let a = thread::spawn(|| {
            tauri::async_runtime::block_on(commands::send_hermes_cli_run_message_for_test(
                vec![HermesCliMessage {
                    role: "user".to_string(),
                    content: "hermes chat --query a".to_string(),
                }],
                None,
                None,
                Some("cancel-a".to_string()),
                Some("run-a".to_string()),
            ))
        });
        let b = thread::spawn(|| {
            tauri::async_runtime::block_on(commands::send_hermes_cli_run_message_for_test(
                vec![HermesCliMessage {
                    role: "user".to_string(),
                    content: "hermes chat --query b".to_string(),
                }],
                None,
                None,
                Some("cancel-b".to_string()),
                Some("run-b".to_string()),
            ))
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while (!started_a.exists() || !started_b.exists()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            started_a.exists() && started_b.exists(),
            "fake Hermes runs did not both start"
        );
        assert!(
            tauri::async_runtime::block_on(commands::cancel_hermes_cli_run(
                Some("cancel-a".to_string()),
                Some("run-a".to_string())
            ))
            .unwrap()
        );
        let a_error = a.join().unwrap().unwrap_err();
        let b_response = b.join().unwrap().unwrap();
        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let _ = fs::remove_dir_all(&root);
        assert!(
            a_error.contains("stopped by the user"),
            "unexpected cancel error: {a_error}"
        );
        assert!(
            b_response.content.contains("done-b"),
            "scoped cancel killed unrelated run: {}",
            b_response.content
        );
    }

    #[test]
    fn hermes_prompt_can_execute_terminal_style_cli_command() {
        assert_eq!(
            hermes_cli_args_from_prompt("hermes tools list")
                .unwrap()
                .unwrap(),
            vec!["tools".to_string(), "list".to_string()]
        );
        assert_eq!(
            hermes_cli_args_from_prompt("hermes cron list --all")
                .unwrap()
                .unwrap(),
            vec!["cron".to_string(), "list".to_string(), "--all".to_string()]
        );
        assert!(hermes_cli_args_from_prompt("hermes setup")
            .unwrap_err()
            .contains("interactive"));
        assert!(hermes_cli_args_from_prompt("hermes uninstall")
            .unwrap_err()
            .contains("blocks high-risk"));
        assert!(hermes_cli_args_from_prompt("hermes --yolo")
            .unwrap_err()
            .contains("blocks high-risk"));
        assert!(hermes_cli_args_from_prompt("hermes --yolo=true tools list")
            .unwrap_err()
            .contains("blocks high-risk"));
        assert!(hermes_cli_args_from_prompt("hermes chat --yolo --query hi")
            .unwrap_err()
            .contains("blocks high-risk"));
        assert!(
            hermes_cli_args_from_prompt("hermes --profile default chat --yolo --query hi")
                .unwrap_err()
                .contains("blocks high-risk")
        );
        assert_eq!(
            hermes_cli_args_from_prompt("hermes chat --query ''")
                .unwrap()
                .unwrap(),
            vec!["chat".to_string(), "--query".to_string(), "".to_string()]
        );
        assert_eq!(
            hermes_cli_args_from_prompt("hermes chat --query \"\"")
                .unwrap()
                .unwrap(),
            vec!["chat".to_string(), "--query".to_string(), "".to_string()]
        );
    }

    #[test]
    fn enabled_profile_context_includes_memory_soul_and_preferences() {
        let context = enabled_profile_prompt_context(&HermesProfileSettings {
            user_name: "Ziad".to_string(),
            role: "Operator".to_string(),
            timezone: "Africa/Cairo".to_string(),
            communication_style: "Direct".to_string(),
            preferences: "Prefer exact command output.".to_string(),
            hermes_memory: "Remember project conventions.".to_string(),
            hermes_soul: "Be sharp and honest.".to_string(),
            memory_enabled: true,
            user_profile_enabled: true,
            ..HermesProfileSettings::default()
        });
        assert!(context.contains("[Zoid enabled user profile]"));
        assert!(context.contains("- Preferences: Prefer exact command output."));
        assert!(context.contains("[Zoid enabled Hermes memory]\nRemember project conventions."));
        assert!(context.contains("[Zoid enabled Hermes soul]\nBe sharp and honest."));
    }

    #[test]
    fn profile_context_honors_disabled_toggles() {
        let context = enabled_profile_prompt_context(&HermesProfileSettings {
            preferences: "Hidden preference".to_string(),
            hermes_memory: "Hidden memory".to_string(),
            hermes_soul: "Hidden soul".to_string(),
            memory_enabled: false,
            user_profile_enabled: false,
            ..HermesProfileSettings::default()
        });
        assert!(context.is_empty());
    }

    #[test]
    fn terminal_usage_is_kept_out_of_hermes_responses() {
        let args = hermes_chat_args("/help", None);
        let usage = command_usage(
            &PathBuf::from("hermes"),
            &args,
            Some(Path::new("/tmp/my repo")),
        );
        assert_eq!(
            usage,
            "cd '/tmp/my repo' && hermes chat --cli --quiet --source desktop --query /help"
        );
        assert_eq!(with_terminal_usage(&usage, "ok"), "ok");
        assert_eq!(with_terminal_usage(&usage, ""), "");
    }

    #[test]
    fn hermes_chat_args_resume_existing_session_after_first_prompt() {
        assert_eq!(
            hermes_chat_args("continue this", Some("session-123")),
            vec![
                "chat".to_string(),
                "--cli".to_string(),
                "--resume".to_string(),
                "session-123".to_string(),
                "--quiet".to_string(),
                "--source".to_string(),
                "desktop".to_string(),
                "--query".to_string(),
                "continue this".to_string(),
            ]
        );
        assert_eq!(
            parse_hermes_session_id("ok\nSession ID: abc-123\n", "fallback"),
            "abc-123"
        );
    }

    #[test]
    fn clone_repository_creates_missing_destination_root_before_remembering_permission() {
        let _guard = env_lock();
        let root = unique_temp_path("clone-missing-root");
        let home = root.join("home");
        let hermes_home = root.join("hermes-home");
        let clone_root = root.join("new-clones");
        fs::create_dir_all(&home).unwrap();
        let fake_git = root.join("git");
        fs::write(
            &fake_git,
            "#!/bin/sh\nif [ \"$1\" = \"clone\" ]; then mkdir -p \"$3/.git\"; exit 0; fi\nif [ \"$1\" = \"-C\" ]; then shift; repo=\"$1\"; shift; case \"$1 $2\" in 'status --porcelain') exit 0 ;; 'remote get-url') echo 'https://github.com/example/repo.git'; exit 0 ;; 'branch --show-current') echo 'main'; exit 0 ;; 'log -1') printf 'abc123\\000Initial commit\\0002026-06-07'; exit 0 ;; 'symbolic-ref --short') echo 'origin/main'; exit 0 ;; esac; fi\nexit 0\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_git).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_git, permissions).unwrap();

        let previous_path = std::env::var("PATH").ok();
        let previous_home = std::env::var("HOME").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var(
            "PATH",
            format!(
                "{}:{}",
                root.display(),
                previous_path.clone().unwrap_or_default()
            ),
        );
        std::env::set_var("HOME", &home);
        std::env::set_var("HERMES_HOME", &hermes_home);

        let cloned = clone_repository(
            "https://github.com/example/repo.git",
            clone_root.to_str().unwrap(),
        )
        .unwrap();

        if let Some(previous_path) = previous_path {
            std::env::set_var("PATH", previous_path);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(previous_home) = previous_home {
            std::env::set_var("HOME", previous_home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }

        assert!(clone_root.exists());
        assert_eq!(cloned.name, "repo");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn github_default_branch_edit_remembers_repository_path_with_remote_url() {
        let _guard = env_lock();
        let root = unique_temp_path("default-branch-permission");
        let repo = root.join("repo");
        let hermes_home = root.join("hermes-home");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(&hermes_home).unwrap();

        let fake_gh = root.join("gh");
        fs::write(
            &fake_gh,
            "#!/bin/sh\nif [ \"$1\" = \"api\" ]; then printf 'main\\ndev\\n'; fi\nexit 0\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_gh).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_gh, permissions).unwrap();

        let previous_path = std::env::var("PATH").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var(
            "PATH",
            format!(
                "{}:{}",
                root.display(),
                previous_path.clone().unwrap_or_default()
            ),
        );
        std::env::set_var("HERMES_HOME", &hermes_home);

        let branches = list_remote_branches(
            repo.to_str().unwrap(),
            Some("https://github.com/example/repo.git".to_string()),
            Some("main".to_string()),
        )
        .unwrap();
        let marker = load_file_permission_marker(&file_permission_bootstrap_path().unwrap());

        if let Some(previous_path) = previous_path {
            std::env::set_var("PATH", previous_path);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);

        assert_eq!(branches.len(), 2);
        assert!(marker.remembered_paths.iter().any(|path| path.ends_with("repo")), "editing a GitHub default branch should remember the repository path even when the remote URL is already known");
    }

    #[test]
    fn warm_file_permissions_persists_marker_after_first_run() {
        let _guard = env_lock();
        let root = unique_temp_path("permission-warm");
        let home = root.join("home");
        let hermes_home = root.join("hermes-home");
        fs::create_dir_all(home.join("Documents")).unwrap();
        fs::create_dir_all(home.join("Desktop")).unwrap();
        fs::create_dir_all(home.join("Downloads")).unwrap();

        let previous_home = std::env::var("HOME").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HOME", &home);
        std::env::set_var("HERMES_HOME", &hermes_home);

        let first = warm_file_permissions_inner(false).unwrap();
        let second = warm_file_permissions_inner(false).unwrap();
        assert!(first
            .iter()
            .any(|path| path == &home.canonicalize().unwrap().to_string_lossy()));
        assert!(!first.iter().any(|path| path.ends_with("Documents")), "home-root access should cover Documents without a second child-folder touch: {first:?}");
        assert!(
            second.is_empty(),
            "permission warming should be skipped after the persisted marker exists"
        );
        assert!(file_permission_bootstrap_path().unwrap().exists());

        if let Some(previous_home) = previous_home {
            std::env::set_var("HOME", previous_home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn warm_file_permissions_records_new_paths_after_marker_exists() {
        let _guard = env_lock();
        let root = unique_temp_path("permission-warm-new-path");
        let home = root.join("home");
        let hermes_home = root.join("hermes-home");
        let first_project = root.join("project-one");
        let second_project = root.join("project-two");
        fs::create_dir_all(home.join("Documents")).unwrap();
        fs::create_dir_all(&first_project).unwrap();
        fs::create_dir_all(&second_project).unwrap();

        let previous_home = std::env::var("HOME").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HOME", &home);
        std::env::set_var("HERMES_HOME", &hermes_home);

        remember_file_permission_path(&first_project).unwrap();
        remember_file_permission_path(&second_project).unwrap();
        let marker = load_file_permission_marker(&file_permission_bootstrap_path().unwrap());

        assert!(marker
            .touched_paths
            .iter()
            .any(|path| path.ends_with("project-one")));
        assert!(marker
            .touched_paths
            .iter()
            .any(|path| path.ends_with("project-two")));
        let warmed = warm_file_permissions_inner(false).unwrap();
        assert!(warmed
            .iter()
            .any(|path| path == &home.canonicalize().unwrap().to_string_lossy()));
        assert!(!warmed.iter().any(|path| path.ends_with("Documents")), "home-root access should cover Documents without a second child-folder touch: {warmed:?}");

        let removable_project = root.join("remembered-then-removed");
        fs::create_dir_all(&removable_project).unwrap();
        remember_file_permission_path(&removable_project).unwrap();
        fs::remove_dir_all(&removable_project).unwrap();
        remember_file_permission_path(&removable_project)
            .expect("remembered paths should skip repeated filesystem access");

        if let Some(previous_home) = previous_home {
            std::env::set_var("HOME", previous_home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hermes_chat_args_resume_known_cli_session_without_continue() {
        assert_eq!(
            hermes_chat_args("hi", Some("20260607_183407_08f4d7")),
            vec![
                "chat".to_string(),
                "--cli".to_string(),
                "--resume".to_string(),
                "20260607_183407_08f4d7".to_string(),
                "--quiet".to_string(),
                "--source".to_string(),
                "desktop".to_string(),
                "--query".to_string(),
                "hi".to_string(),
            ]
        );
    }

    #[test]
    fn hermes_session_id_is_parsed_from_cli_output() {
        assert_eq!(
            parse_hermes_session_id("answer\nsession_id: 20260607_183407_08f4d7", "fallback"),
            "20260607_183407_08f4d7"
        );
        assert_eq!(
            parse_hermes_session_id("answer only", "fallback"),
            "fallback"
        );
    }

    #[test]
    fn terminal_style_cli_message_runs_requested_hermes_subcommand() {
        let _guard = env_lock();
        let root = unique_temp_path("hermes-cli-command");
        fs::create_dir_all(&root).unwrap();
        let fake_hermes = root.join("hermes");
        fs::write(
            &fake_hermes,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'hermes fake'; exit 0; fi\nprintf 'args:%s\\n' \"$*\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_hermes, permissions).unwrap();

        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        std::env::set_var("ZOID_HERMES_CLI", &fake_hermes);
        let response = tauri::async_runtime::block_on(commands::send_hermes_cli_message_for_test(
            vec![HermesCliMessage {
                role: "user".to_string(),
                content: "hermes tools list".to_string(),
            }],
            None,
            None,
        ))
        .unwrap();
        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let _ = fs::remove_dir_all(&root);

        assert!(!response.content.contains("Terminal command"));
        assert!(!response.content.contains("hermes tools list"));
        assert!(!response.content.contains("hermes --yolo tools list"));
        assert!(response.content.contains("args:tools list"));
        assert!(!response.content.contains("args:--yolo tools list"));
    }

    #[test]
    fn hermes_cron_list_parser_extracts_jobs_and_protection() {
        let raw = r#"
  6f1ab6c40ac9 [active]
    Name:      feature-critique-watchdog
    Schedule:  every 2m
    Repeat:    ∞
    Next run:  2026-06-07T17:33:39.374685+03:00
    Deliver:   origin
    Script:    feature_critique_watchdog.py
    Mode:      no-agent (script stdout delivered directly)
    Last run:  2026-06-07T17:31:39.374685+03:00  ok

  12fd35ec77e2 [active]
    Name:      MaVoid Daily Social Creator Scheduler
    Schedule:  0 8 * * *
    Repeat:    ∞
    Next run:  2026-06-08T08:00:00+03:00
    Deliver:   local
    Skills:    mavoid-social-design-workflow, himalaya
    Last run:  2026-06-07T08:00:27.123971+03:00  error: RuntimeError: HTTP 429
"#;
        let list = parse_hermes_cron_list(raw, Some("/tmp/hermes".to_string()));
        assert_eq!(list.jobs.len(), 2);
        assert_eq!(list.jobs[0].job_id, "6f1ab6c40ac9");
        assert!(list.jobs[0].no_agent);
        assert!(list.jobs[0].protected);
        assert_eq!(
            list.jobs[1].last_delivery_error.as_deref(),
            Some("error: RuntimeError: HTTP 429")
        );
        assert_eq!(
            list.jobs[1].skills,
            vec![
                "mavoid-social-design-workflow".to_string(),
                "himalaya".to_string()
            ]
        );
        assert_eq!(list.watcher_source_status, "unavailable");
    }

    #[test]
    fn hermes_cron_list_parser_ignores_stdout_json_arrays() {
        let raw = r#"
  9562e7cb93b6 [active]
    Name:      MaVoid OmniSocials Publish Monitor
    Schedule:  every 15m
    Repeat:    ∞
    Next run:  2026-06-08T02:06:49.131341+03:00
    Deliver:   local
    Script:    mavoid_publish_monitor_wrapper.py
    Mode:      no-agent (script stdout delivered directly)
    Last run:  2026-06-08T01:51:49.131341+03:00  error: Script exited with code 1
stdout:
{
  "events": [],
  "errors": [],
  "email_retries": []
}
"#;
        let list = parse_hermes_cron_list(raw, Some("/tmp/hermes".to_string()));
        assert_eq!(list.jobs.len(), 1);
        assert_eq!(list.jobs[0].job_id, "9562e7cb93b6");
        assert_eq!(list.jobs[0].name, "MaVoid OmniSocials Publish Monitor");
        assert!(!list
            .jobs
            .iter()
            .any(|job| job.job_id.contains("email_retries")));
    }

    #[test]
    fn protected_marker_detection_blocks_internal_removal_only() {
        assert!(
            protection_reason_for_job("abc", "feature-critique-watchdog", Some("watchdog.py"))
                .is_some()
        );
        assert!(protection_reason_for_job(
            "abc",
            "Other job",
            Some("feature_critique_watchdog.py")
        )
        .is_some());
        assert!(protection_reason_for_job(
            "abc",
            "Obsidian Hermes Session Archive",
            Some("archive.py")
        )
        .is_some());
        assert!(protection_reason_for_job(
            "abc",
            "Client Archive Backup",
            Some("client_archive_backup.py")
        )
        .is_none());
        assert!(protection_reason_for_job(
            "abc",
            "Internal Report Publisher",
            Some("internal_report.py")
        )
        .is_none());
        assert!(protection_reason_for_job(
            "abc",
            "Watchdog uptime report",
            Some("watchdog_uptime.py")
        )
        .is_none());
        assert!(
            protection_reason_for_job("abc", "Client report publisher", Some("publish.py"))
                .is_none()
        );
    }

    #[test]
    fn managed_provider_input_sanitizes_models_and_env() {
        let provider = provider_from_input(
            ProviderInput {
                id: None,
                display_name: "Google Gemini".to_string(),
                provider_type: "google-gemini".to_string(),
                provider_id: "Google Gemini".to_string(),
                api_key_env: "google_api_key".to_string(),
                api_key: Some("secret".to_string()),
                default_model: "gemma-3-27b-it".to_string(),
                model_options: vec![
                    "gemma-3-27b-it".to_string(),
                    "gemini-2.5-pro".to_string(),
                    "".to_string(),
                ],
                base_url: "".to_string(),
            },
            None,
        )
        .unwrap();
        assert_eq!(provider.provider_id, "google-gemini");
        assert_eq!(provider.api_key_env, "GOOGLE_API_KEY");
        assert!(provider.key_stored);
        assert_eq!(
            provider.model_options,
            vec!["gemma-3-27b-it".to_string(), "gemini-2.5-pro".to_string()]
        );
    }

    #[test]
    fn env_var_writer_replaces_without_duplication() {
        let root = unique_temp_path("provider-env");
        fs::create_dir_all(&root).unwrap();
        let env_path = root.join(".env");
        fs::write(&env_path, "OPENAI_API_KEY=old\nGOOGLE_API_KEY=old\n").unwrap();
        write_env_var(&env_path, "GOOGLE_API_KEY", "new-secret").unwrap();
        let raw = fs::read_to_string(&env_path).unwrap();
        assert_eq!(raw.matches("GOOGLE_API_KEY=").count(), 1);
        assert!(raw.contains("GOOGLE_API_KEY=new-secret"));
        assert!(raw.contains("OPENAI_API_KEY=old"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn env_var_writer_rejects_newline_injection() {
        let root = unique_temp_path("provider-env-injection");
        fs::create_dir_all(&root).unwrap();
        let env_path = root.join(".env");
        let error = write_env_var(&env_path, "GOOGLE_API_KEY", "good\nEVIL=1").unwrap_err();
        assert!(error.contains("control characters"));
        assert!(!env_path.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_provider_metadata_round_trip_uses_profile_home() {
        let _guard = env_lock();
        let root = unique_temp_path("providers-round-trip");
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        let saved = save_managed_provider_inner(ProviderInput {
            id: None,
            display_name: "Google Gemini".to_string(),
            provider_type: "google-gemini".to_string(),
            provider_id: "google".to_string(),
            api_key_env: "GOOGLE_API_KEY".to_string(),
            api_key: None,
            default_model: "gemini-2.5-pro".to_string(),
            model_options: vec!["gemini-2.5-pro".to_string(), "gemma-3-27b-it".to_string()],
            base_url: "".to_string(),
        })
        .unwrap();
        assert_eq!(saved.id, "google");
        assert!(!saved.key_stored);
        let providers = load_managed_providers_inner().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(
            providers[0].model_options,
            vec!["gemini-2.5-pro".to_string(), "gemma-3-27b-it".to_string()]
        );
        assert!(root.join("zoid-providers.json").exists());
        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_provider_second_new_save_updates_existing_id() {
        let _guard = env_lock();
        let root = unique_temp_path("providers-no-duplicates");
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);

        let first = save_managed_provider_inner(ProviderInput {
            id: None,
            display_name: "Google Gemini".to_string(),
            provider_type: "google-gemini".to_string(),
            provider_id: "google".to_string(),
            api_key_env: "GOOGLE_API_KEY".to_string(),
            api_key: None,
            default_model: "gemini-2.5-pro".to_string(),
            model_options: vec!["gemini-2.5-pro".to_string()],
            base_url: "".to_string(),
        })
        .unwrap();
        let second = save_managed_provider_inner(ProviderInput {
            id: None,
            display_name: "Google Gemini Updated".to_string(),
            provider_type: "google-gemini".to_string(),
            provider_id: "google".to_string(),
            api_key_env: "GOOGLE_API_KEY".to_string(),
            api_key: None,
            default_model: "gemma-3-27b-it".to_string(),
            model_options: vec!["gemma-3-27b-it".to_string()],
            base_url: "".to_string(),
        })
        .unwrap();

        assert_eq!(first.id, "google");
        assert_eq!(second.id, "google");
        let providers = load_managed_providers_inner().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].display_name, "Google Gemini Updated");
        assert_eq!(providers[0].default_model, "gemma-3-27b-it");

        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_provider_save_rejects_duplicate_provider_ids_on_edit() {
        let _guard = env_lock();
        let root = unique_temp_path("providers-duplicate-provider-id");
        fs::create_dir_all(&root).unwrap();
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);

        let google = save_managed_provider_inner(ProviderInput {
            id: None,
            display_name: "Google Gemini".to_string(),
            provider_type: "google-gemini".to_string(),
            provider_id: "google".to_string(),
            api_key_env: "GOOGLE_API_KEY".to_string(),
            api_key: None,
            default_model: "gemini-2.5-pro".to_string(),
            model_options: vec!["gemini-2.5-pro".to_string()],
            base_url: "".to_string(),
        })
        .unwrap();
        let custom = save_managed_provider_inner(ProviderInput {
            id: None,
            display_name: "Custom Provider".to_string(),
            provider_type: "custom".to_string(),
            provider_id: "custom-provider".to_string(),
            api_key_env: "CUSTOM_PROVIDER_KEY".to_string(),
            api_key: None,
            default_model: "custom-model".to_string(),
            model_options: vec!["custom-model".to_string()],
            base_url: "".to_string(),
        })
        .unwrap();

        let duplicate_error = save_managed_provider_inner(ProviderInput {
            id: Some(custom.id.clone()),
            display_name: "Custom Provider".to_string(),
            provider_type: "custom".to_string(),
            provider_id: google.provider_id.clone(),
            api_key_env: "CUSTOM_PROVIDER_KEY".to_string(),
            api_key: None,
            default_model: "custom-model".to_string(),
            model_options: vec!["custom-model".to_string()],
            base_url: "".to_string(),
        })
        .unwrap_err();
        assert!(duplicate_error.contains("already used"));
        assert_eq!(load_managed_providers_inner().unwrap().len(), 2);

        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hermes_profile_settings_round_trip_uses_active_profile_home() {
        let _guard = env_lock();
        let root = unique_temp_path("profile-settings");
        let previous_home = std::env::var("HERMES_HOME").ok();
        std::env::set_var("HERMES_HOME", &root);
        let initial = load_hermes_profile_settings_inner().unwrap();
        assert_eq!(initial.user_name, "Ziad Salah");
        let saved = save_hermes_profile_settings_inner(HermesProfileSettings {
            user_name: "Ziad".to_string(),
            preferences: "Direct".to_string(),
            hermes_memory: "Remember stable preferences.".to_string(),
            hermes_soul: "Sharp operator.".to_string(),
            profile: "ignored".to_string(),
            storage_path: "ignored".to_string(),
            updated_at: "0".to_string(),
            ..HermesProfileSettings::default()
        })
        .unwrap();
        assert_eq!(saved.profile, active_hermes_profile());
        assert!(PathBuf::from(&saved.storage_path).exists());
        let loaded = load_hermes_profile_settings_inner().unwrap();
        assert_eq!(loaded.preferences, "Direct");
        assert_eq!(loaded.hermes_memory, "Remember stable preferences.");
        assert_eq!(loaded.hermes_soul, "Sharp operator.");
        if let Some(previous_home) = previous_home {
            std::env::set_var("HERMES_HOME", previous_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hermes_skill_table_parser_handles_real_status_rows() {
        let output = r#"
┏━━━━━━━━━━━━━━┳━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━┳━━━━━━━━━┓
┃ Name         ┃ Category ┃ Description          ┃ Source   ┃ Status  ┃
┣━━━━━━━━━━━━━━╋━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━╋━━━━━━━━━┫
│ hermes-agent │ system   │ Configure Hermes     │ user     │ enabled │
│ plan         │ dev      │ Write plans          │ user     │ disabled│
└──────────────┴──────────┴──────────────────────┴──────────┴─────────┘
"#;
        let (available, enabled) = parse_hermes_skill_table(output).unwrap();
        assert!(available.iter().any(|skill| skill == "hermes-agent"));
        assert!(available.iter().any(|skill| skill == "plan"));
        assert!(enabled.iter().any(|skill| skill == "hermes-agent"));
        assert!(!enabled.iter().any(|skill| skill == "plan"));
    }

    #[test]
    fn model_catalog_cache_augments_default_model_seeds() {
        let root = unique_temp_path("model-catalog-cache");
        let cache_dir = root.join("cache");
        fs::create_dir_all(&cache_dir).unwrap();
        let catalog_path = cache_dir.join("model_catalog.json");
        fs::write(
            &catalog_path,
            r#"{"providers":{"openai-codex":{"models":[{"id":"openai-codex/gpt-current-live"}]},"openai":{"models":[{"id":"openai/gpt-live-direct"}]}}}"#,
        ).unwrap();
        let mut models = default_available_models();
        merge_model_catalog_cache(&mut models, &catalog_path);
        assert!(models
            .get("openai-codex")
            .unwrap()
            .iter()
            .any(|model| model == "openai-codex/gpt-current-live"));
        assert!(models
            .get("openai-codex")
            .unwrap()
            .iter()
            .any(|model| model == "gpt-current-live"));
        assert!(models
            .get("openai")
            .unwrap()
            .iter()
            .any(|model| model == "openai/gpt-live-direct"));
        assert!(models
            .get("openai")
            .unwrap()
            .iter()
            .any(|model| model == "gpt-live-direct"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hermes_profile_settings_preserve_real_yaml_shapes() {
        let _guard = env_lock();
        let root = unique_temp_path("profile-yaml-shapes");
        fs::create_dir_all(root.join("memories")).unwrap();
        let config_path = root.join("config.yaml");
        fs::write(
            &config_path,
            r#"model:
  provider: openai-codex
  default: gpt-5.5
agent:
  reasoning_effort: medium
  system_prompt: old soul
  disabled_toolsets:
    - browser
    - web
display:
  personality: concise
  bell_on_complete: false
  background_process_notifications: important
timezone: UTC
stt:
  enabled: false
voice:
  auto_tts: false
approvals:
  mode: smart
memory:
  memory_enabled: true
  user_profile_enabled: true
  memory_char_limit: 2500
  user_char_limit: 1600
security:
  redact_secrets: true
privacy:
  redact_pii: false
checkpoints:
  enabled: false
skills:
  disabled:
    - hermes-agent
discord:
  allowed_channels: '123,456'
telegram:
  bot_token: ''
"#,
        )
        .unwrap();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        let previous_profile = std::env::var("HERMES_PROFILE").ok();
        let previous_zoid_profile = std::env::var("ZOID_HERMES_PROFILE").ok();
        std::env::set_var("HERMES_HOME", &root);
        std::env::remove_var("HERMES_PROFILE");
        std::env::set_var("ZOID_HERMES_PROFILE", "ignored-by-active-profile");

        let loaded = load_hermes_profile_settings_inner().unwrap();
        assert_eq!(loaded.profile, "default");
        assert_eq!(loaded.timezone, "UTC");
        assert_eq!(loaded.voice_preference, "off");
        assert_eq!(loaded.notification_preference, "important");
        assert!(!loaded.checkpoints_enabled);
        assert!(!loaded.web_search_enabled);
        assert!(!loaded.browser_tools_enabled);
        assert!(!loaded
            .enabled_skills
            .lines()
            .any(|skill| skill == "hermes-agent"));
        assert!(loaded
            .gateway_platforms
            .contains("Discord allowed channels"));

        let saved = save_hermes_profile_settings_inner(HermesProfileSettings {
            timezone: "Africa/Cairo".to_string(),
            voice_preference: "tts".to_string(),
            notification_preference: "all".to_string(),
            checkpoints_enabled: true,
            web_search_enabled: false,
            browser_tools_enabled: false,
            terminal_tools_enabled: true,
            file_tools_enabled: false,
            cron_enabled: false,
            hermes_soul: "new soul".to_string(),
            preferences: "User profile".to_string(),
            hermes_memory: "Memory".to_string(),
            memory_char_limit: 2800,
            user_char_limit: 1700,
            ..loaded
        })
        .unwrap();
        assert_eq!(saved.profile, "default");
        let saved_raw = fs::read_to_string(&config_path).unwrap();
        let saved_yaml: serde_yaml::Value = serde_yaml::from_str(&saved_raw).unwrap();
        assert_eq!(
            yaml_get_string(&saved_yaml, &["timezone"]).as_deref(),
            Some("Africa/Cairo")
        );
        assert!(!yaml_is_mapping(&saved_yaml, &["timezone"]));
        assert_eq!(yaml_get_bool(&saved_yaml, &["stt", "enabled"]), Some(false));
        assert_eq!(
            yaml_get_bool(&saved_yaml, &["voice", "auto_tts"]),
            Some(true)
        );
        assert_eq!(
            yaml_get_string(
                &saved_yaml,
                &["display", "background_process_notifications"]
            )
            .as_deref(),
            Some("all")
        );
        assert_eq!(
            yaml_get_bool(&saved_yaml, &["display", "bell_on_complete"]),
            Some(true)
        );
        assert_eq!(
            yaml_get_string(&saved_yaml, &["agent", "system_prompt"]).as_deref(),
            Some("new soul")
        );
        assert_eq!(
            yaml_get_u64(&saved_yaml, &["memory", "memory_char_limit"]),
            Some(2800)
        );
        assert_eq!(
            yaml_get_u64(&saved_yaml, &["memory", "user_char_limit"]),
            Some(1700)
        );
        assert_eq!(
            yaml_get_bool(&saved_yaml, &["checkpoints", "enabled"]),
            Some(true)
        );
        let disabled_toolsets = yaml_string_list(&saved_yaml, &["agent", "disabled_toolsets"]);
        assert!(disabled_toolsets.contains(&"web".to_string()));
        assert!(disabled_toolsets.contains(&"browser".to_string()));
        assert!(disabled_toolsets.contains(&"file".to_string()));
        assert!(disabled_toolsets.contains(&"cronjob".to_string()));
        assert!(!disabled_toolsets.contains(&"terminal".to_string()));
        assert!(yaml_string_list(&saved_yaml, &["skills", "disabled"])
            .contains(&"hermes-agent".to_string()));
        assert!(root.read_dir().unwrap().any(|entry| entry
            .unwrap()
            .path()
            .to_string_lossy()
            .contains("zoid-save")));

        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        if let Some(previous_profile) = previous_profile {
            std::env::set_var("HERMES_PROFILE", previous_profile);
        } else {
            std::env::remove_var("HERMES_PROFILE");
        }
        if let Some(previous_zoid_profile) = previous_zoid_profile {
            std::env::set_var("ZOID_HERMES_PROFILE", previous_zoid_profile);
        } else {
            std::env::remove_var("ZOID_HERMES_PROFILE");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hermes_profile_settings_create_config_when_missing() {
        let _guard = env_lock();
        let root = unique_temp_path("profile-create-config");
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        let previous_profile = std::env::var("HERMES_PROFILE").ok();
        std::env::set_var("HERMES_HOME", &root);
        std::env::remove_var("HERMES_PROFILE");

        let saved = save_hermes_profile_settings_inner(HermesProfileSettings {
            model_provider: "openai-codex".to_string(),
            model_name: "gpt-5.5".to_string(),
            reasoning_effort: "high".to_string(),
            timezone: "Africa/Cairo".to_string(),
            personality_preset: "concise".to_string(),
            approval_mode: "manual".to_string(),
            access_mode: "workspace".to_string(),
            notification_preference: "all".to_string(),
            voice_preference: "tts".to_string(),
            hermes_soul: "created soul".to_string(),
            preferences: "created user".to_string(),
            hermes_memory: "created memory".to_string(),
            ..HermesProfileSettings::default()
        })
        .unwrap();

        let config_path = root.join("config.yaml");
        assert!(config_path.exists());
        let saved_yaml: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(saved.profile, "default");
        assert_eq!(
            yaml_get_string(&saved_yaml, &["model", "provider"]).as_deref(),
            Some("openai-codex")
        );
        assert_eq!(
            yaml_get_string(&saved_yaml, &["model", "default"]).as_deref(),
            Some("gpt-5.5")
        );
        assert_eq!(
            yaml_get_string(&saved_yaml, &["agent", "reasoning_effort"]).as_deref(),
            Some("high")
        );
        assert_eq!(
            yaml_get_string(&saved_yaml, &["agent", "system_prompt"]).as_deref(),
            Some("created soul")
        );
        assert_eq!(
            yaml_get_string(&saved_yaml, &["approvals", "mode"]).as_deref(),
            Some("manual")
        );
        assert_eq!(
            yaml_get_string(
                &saved_yaml,
                &["display", "background_process_notifications"]
            )
            .as_deref(),
            Some("all")
        );

        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        if let Some(previous_profile) = previous_profile {
            std::env::set_var("HERMES_PROFILE", previous_profile);
        } else {
            std::env::remove_var("HERMES_PROFILE");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hermes_profile_name_rejects_paths_without_explicit_home() {
        let _guard = env_lock();
        let previous_home = std::env::var("HOME").ok();
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        let previous_profile = std::env::var("HERMES_PROFILE").ok();
        let home = unique_temp_path("profile-path-safety-home");
        fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);
        std::env::remove_var("HERMES_HOME");
        std::env::set_var("HERMES_PROFILE", "../../Documents");

        let error = hermes_profile_home().unwrap_err();
        assert!(error.contains("simple profile name"));

        if let Some(previous_home) = previous_home {
            std::env::set_var("HOME", previous_home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        if let Some(previous_profile) = previous_profile {
            std::env::set_var("HERMES_PROFILE", previous_profile);
        } else {
            std::env::remove_var("HERMES_PROFILE");
        }
        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn profile_runtime_args_apply_to_normal_chat_but_not_explicit_commands() {
        let _guard = env_lock();
        let root = unique_temp_path("profile-runtime-args");
        let previous_hermes_home = std::env::var("HERMES_HOME").ok();
        let previous_profile = std::env::var("HERMES_PROFILE").ok();
        std::env::set_var("HERMES_HOME", &root);
        std::env::remove_var("HERMES_PROFILE");

        save_hermes_profile_settings_inner(HermesProfileSettings {
            model_provider: "openai-codex".to_string(),
            model_name: "gpt-5.5".to_string(),
            access_mode: "workspace".to_string(),
            toolsets: "terminal\nfile\nskills\nmemory\nsession_search\ntodo".to_string(),
            preferences: "user".to_string(),
            hermes_memory: "memory".to_string(),
            hermes_soul: "soul".to_string(),
            ..HermesProfileSettings::default()
        })
        .unwrap();

        let mut workspace_args = hermes_chat_args("hello", None);
        apply_profile_runtime_args(&mut workspace_args).unwrap();
        assert!(workspace_args
            .windows(2)
            .any(|window| window == ["--provider", "openai-codex"]));
        assert!(workspace_args
            .windows(2)
            .any(|window| window == ["--model", "gpt-5.5"]));
        assert!(
            workspace_args.windows(2).any(|window| window
                == [
                    "--toolsets",
                    "browser,cronjob,file,memory,session_search,skills,terminal,todo,vision,web"
                ]),
            "runtime args were {workspace_args:?}"
        );
        let query_index = workspace_args
            .iter()
            .position(|arg| arg == "--query")
            .unwrap();
        assert!(
            workspace_args
                .iter()
                .position(|arg| arg == "--provider")
                .unwrap()
                < query_index
        );

        let safe_settings = HermesProfileSettings {
            access_mode: "safe".to_string(),
            ..load_hermes_profile_settings_inner().unwrap()
        };
        let mut safe_args = hermes_chat_args("hello", None);
        apply_profile_runtime_args_from_settings(&mut safe_args, &safe_settings);
        assert!(
            safe_args
                .windows(2)
                .any(|window| window == ["--toolsets", "session_search"]),
            "safe args were {safe_args:?}"
        );

        let full_settings = HermesProfileSettings {
            access_mode: "full".to_string(),
            ..safe_settings
        };
        let mut full_args = hermes_chat_args("hello", None);
        apply_profile_runtime_args_from_settings(&mut full_args, &full_settings);
        assert!(
            !full_args.iter().any(|arg| arg == "--toolsets"),
            "full access must not force toolset overrides: {full_args:?}"
        );

        let built_args = build_profiled_hermes_chat_args("hello", None).unwrap();
        assert!(built_args
            .windows(2)
            .any(|window| window == ["--provider", "openai-codex"]));

        fs::write(root.join("config.yaml"), "model: [broken").unwrap();
        let error = build_profiled_hermes_chat_args("hello", None).unwrap_err();
        assert!(error.contains("Failed to load Hermes profile settings before launching Hermes"));

        let explicit = hermes_cli_args_from_prompt("hermes chat --query hi")
            .unwrap()
            .unwrap();
        assert_eq!(
            explicit,
            vec!["chat".to_string(), "--query".to_string(), "hi".to_string()]
        );

        if let Some(previous_hermes_home) = previous_hermes_home {
            std::env::set_var("HERMES_HOME", previous_hermes_home);
        } else {
            std::env::remove_var("HERMES_HOME");
        }
        if let Some(previous_profile) = previous_profile {
            std::env::set_var("HERMES_PROFILE", previous_profile);
        } else {
            std::env::remove_var("HERMES_PROFILE");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hermes_automation_actions_call_cli_and_refresh_provider_state() {
        let _guard = env_lock();
        let root = unique_temp_path("cron-actions");
        fs::create_dir_all(&root).unwrap();
        let hermes = root.join("hermes-fake");
        let log = root.join("calls.log");
        let removed = root.join("removed.flag");
        let previous_cli = std::env::var("ZOID_HERMES_CLI").ok();
        let script = format!(
            r#"#!/usr/bin/env bash
set -e
printf '%s\n' "$*" >> {log}
if [ "${{1:-}}" = "--version" ]; then
  echo 'hermes fake 1.0.0'
  exit 0
fi
if [ "${{1:-}}" = "cron" ] && [ "${{2:-}}" = "list" ]; then
  if [ ! -f {removed} ]; then
    cat <<'LIST'
┌─────────────────────────────────────────────────────────────────────────┐
│                         Scheduled Jobs                                  │
└─────────────────────────────────────────────────────────────────────────┘

  abc123def456 [active]
    Name:      Zoid Cron Action Smoke
    Schedule:  every 1h
    Repeat:    ∞
    Next run:  2099-01-01T00:00:00+02:00
    Deliver:   local
    Last run:  never
LIST
  fi
  exit 0
fi
if [ "${{1:-}}" = "cron" ] && [ "${{2:-}}" = "pause" ]; then echo 'Paused job'; exit 0; fi
if [ "${{1:-}}" = "cron" ] && [ "${{2:-}}" = "resume" ]; then echo 'Resumed job'; exit 0; fi
if [ "${{1:-}}" = "cron" ] && [ "${{2:-}}" = "run" ]; then echo 'Triggered job'; exit 0; fi
if [ "${{1:-}}" = "cron" ] && [ "${{2:-}}" = "remove" ]; then touch {removed}; echo 'Removed job'; exit 0; fi
echo "unexpected args: $*" >&2
exit 2
"#,
            log = shell_quote(log.to_string_lossy().as_ref()),
            removed = shell_quote(removed.to_string_lossy().as_ref())
        );
        fs::write(&hermes, script).unwrap();
        let mut permissions = fs::metadata(&hermes).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hermes, permissions).unwrap();
        std::env::set_var("ZOID_HERMES_CLI", &hermes);

        let paused = manage_hermes_cron_job_inner("abc123def456", "pause").unwrap();
        assert!(paused.jobs.iter().any(|job| job.job_id == "abc123def456"));
        let resumed = manage_hermes_cron_job_inner("abc123def456", "resume").unwrap();
        assert!(resumed.jobs.iter().any(|job| job.job_id == "abc123def456"));
        let run = manage_hermes_cron_job_inner("abc123def456", "run").unwrap();
        assert!(run.jobs.iter().any(|job| job.job_id == "abc123def456"));
        let removed_list = manage_hermes_cron_job_inner("abc123def456", "remove").unwrap();
        assert!(!removed_list
            .jobs
            .iter()
            .any(|job| job.job_id == "abc123def456"));

        let calls = fs::read_to_string(&log).unwrap();
        for expected in [
            "cron pause abc123def456",
            "cron resume abc123def456",
            "cron run --accept-hooks abc123def456",
            "cron remove abc123def456",
            "cron list --all",
        ] {
            assert!(calls.contains(expected), "missing {expected} in {calls}");
        }

        if let Some(previous_cli) = previous_cli {
            std::env::set_var("ZOID_HERMES_CLI", previous_cli);
        } else {
            std::env::remove_var("ZOID_HERMES_CLI");
        }
        let _ = fs::remove_dir_all(&root);
    }
}
