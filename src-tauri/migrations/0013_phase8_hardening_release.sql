-- Phase 8 hardening/release readiness: retention settings, cleanup audit, and query indexes.

create table if not exists log_retention_settings (
    scope text primary key,
    retention_days integer not null check (retention_days between 1 and 3650),
    max_total_bytes integer not null check (max_total_bytes between 1024 and 1073741824),
    enabled integer not null default 1 check (enabled in (0, 1)),
    updated_at text not null default current_timestamp
);

create table if not exists log_cleanup_runs (
    id text primary key,
    scope text not null,
    dry_run integer not null check (dry_run in (0, 1)),
    files_considered integer not null default 0 check (files_considered >= 0),
    files_deleted integer not null default 0 check (files_deleted >= 0),
    bytes_deleted integer not null default 0 check (bytes_deleted >= 0),
    status text not null check (status in ('completed', 'failed')),
    error_message text,
    created_at text not null default current_timestamp
);

insert or ignore into log_retention_settings(scope, retention_days, max_total_bytes, enabled)
values ('default', 30, 10485760, 1), ('agent', 14, 10485760, 1), ('foundation', 30, 5242880, 1);

create index if not exists idx_events_created_id on events(created_at desc, id desc);
create index if not exists idx_events_type_created on events(type, created_at desc, id);
create index if not exists idx_tasks_status_updated on tasks(status, updated_at desc, id);
create index if not exists idx_tasks_workspace_status_updated on tasks(workspace_key, status, updated_at desc);
create index if not exists idx_agent_runs_task_status_created on agent_runs(task_id, status, started_at desc, id);
create index if not exists idx_agent_runs_status_started on agent_runs(status, started_at desc, id);
create index if not exists idx_notifications_state_severity_created on notifications(state, severity, created_at desc, id);
create index if not exists idx_browser_captures_workspace_created on browser_captures(workspace_key, captured_at desc, id);
create index if not exists idx_browser_tabs_workspace_profile_updated on browser_tabs(workspace_key, profile_key, updated_at desc, id);
create index if not exists idx_log_cleanup_runs_scope_created on log_cleanup_runs(scope, created_at desc);
