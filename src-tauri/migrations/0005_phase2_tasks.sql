create table if not exists tasks (
    id text primary key,
    title text not null check (length(trim(title)) > 0 and length(cast(title as blob)) <= 256),
    detail text check (detail is null or (length(trim(detail)) > 0 and length(cast(detail as blob)) <= 4096)),
    status text not null default 'inbox' check (status in ('inbox', 'planned', 'active', 'waiting', 'review_required', 'blocked', 'completed', 'failed', 'cancelled', 'archived', 'deleted')),
    priority text not null default 'normal' check (priority in ('low', 'normal', 'high', 'urgent')),
    workspace_key text not null default 'tasks',
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    archived_at text,
    deleted_at text,
    metadata_json text not null default '{}' check (json_valid(metadata_json))
);

create index if not exists idx_tasks_active_priority_time on tasks(deleted_at, archived_at, priority, updated_at, id);
create index if not exists idx_tasks_status on tasks(status);
create index if not exists idx_tasks_workspace_active on tasks(workspace_key, deleted_at, archived_at, updated_at);
create index if not exists idx_tasks_archived_at on tasks(archived_at);
create index if not exists idx_tasks_deleted_at on tasks(deleted_at);
