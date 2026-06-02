create table if not exists agent_profiles (
    id text primary key,
    label text not null check (length(trim(label)) > 0),
    configured integer not null default 0 check (configured in (0, 1)),
    command text check (command is null or length(trim(command)) > 0),
    config_json text not null default '{}' check (json_valid(config_json)),
    capabilities_json text not null default '{}' check (json_valid(capabilities_json)),
    credential_ref text,
    env_refs_json text not null default '[]' check (json_valid(env_refs_json)),
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    check (configured = 0 or command is not null)
);

create table if not exists cli_sessions (
    id text primary key,
    task_id text not null,
    profile_id text not null,
    mode text not null,
    cwd text not null check (length(trim(cwd)) > 0),
    status text not null default 'active' check (status in ('active', 'waiting_for_input', 'review_required', 'completed', 'failed', 'cancelled', 'blocked')),
    status_summary text not null default '',
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    completed_at text,
    foreign key (task_id) references tasks(id) on delete restrict,
    foreign key (profile_id) references agent_profiles(id) on delete restrict
);

create table if not exists agent_runs (
    id text primary key,
    task_id text not null,
    profile_id text not null,
    session_id text not null,
    cwd text not null check (length(trim(cwd)) > 0),
    command_snapshot text not null check (length(trim(command_snapshot)) > 0),
    profile_snapshot_json text not null default '{}' check (json_valid(profile_snapshot_json)),
    status text not null default 'queued' check (status in ('queued', 'starting', 'running', 'waiting_for_input', 'review_required', 'completed', 'failed', 'cancelled', 'blocked')),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    started_at text,
    completed_at text,
    duration_ms integer check (duration_ms is null or duration_ms >= 0),
    exit_code integer,
    log_reference_id text,
    output_summary text,
    error_summary text,
    review_state text not null default 'not_required' check (review_state in ('not_required', 'required', 'requested', 'in_progress', 'approved', 'required_fixes', 'blocked_insufficient_evidence', 'failed', 'cancelled')),
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    foreign key (task_id) references tasks(id) on delete restrict,
    foreign key (profile_id) references agent_profiles(id) on delete restrict,
    foreign key (session_id) references cli_sessions(id) on delete restrict,
    foreign key (log_reference_id) references log_references(id) on delete set null
);

create index if not exists idx_agent_profiles_configured on agent_profiles(configured, updated_at);
create index if not exists idx_cli_sessions_task on cli_sessions(task_id, updated_at);
create index if not exists idx_cli_sessions_profile on cli_sessions(profile_id, updated_at);
create index if not exists idx_agent_runs_task on agent_runs(task_id, created_at);
create index if not exists idx_agent_runs_session on agent_runs(session_id, created_at);
create index if not exists idx_agent_runs_status on agent_runs(status, updated_at);
create index if not exists idx_agent_runs_log_reference on agent_runs(log_reference_id);
