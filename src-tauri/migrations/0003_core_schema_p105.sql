create table if not exists app_settings (
    key text primary key,
    value_json text not null default 'null' check (json_valid(value_json)),
    value_type text not null default 'json' check (value_type in ('boolean', 'integer', 'number', 'string', 'json')),
    scope text not null default 'app' check (scope in ('app', 'workspace', 'integration')),
    description text not null default '',
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists integration_statuses (
    integration_key text primary key,
    display_name text not null,
    status text not null default 'not_configured' check (status in ('not_configured', 'configured', 'connected', 'degraded', 'disabled', 'error')),
    config_json text not null default '{}' check (json_valid(config_json)),
    credential_ref text,
    last_checked_at text,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists entity_links (
    id text primary key,
    source_type text not null,
    source_id text not null,
    target_type text not null,
    target_id text not null,
    relation_type text not null,
    created_at text not null default current_timestamp,
    created_by_actor_type text not null default 'system',
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    unique (source_type, source_id, target_type, target_id, relation_type)
);

create table if not exists log_references (
    id text primary key,
    log_scope text not null,
    relative_path text not null,
    redaction_count integer not null default 0 check (redaction_count >= 0),
    byte_count integer not null default 0 check (byte_count >= 0),
    created_at text not null default current_timestamp,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    unique (log_scope, relative_path)
);

create table if not exists file_references (
    id text primary key,
    workspace_key text,
    relative_path text not null,
    display_name text not null,
    mime_type text,
    content_hash text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    unique (relative_path)
);

create table if not exists action_policies (
    category text primary key,
    policy text not null check (policy in ('allow', 'ask_before_action', 'block_until_confirmed', 'require_clear_task')),
    reviewer_required text not null check (reviewer_required in ('none', 'maybe', 'usually', 'yes')),
    human_confirmation text not null check (human_confirmation in ('none', 'maybe', 'yes', 'always')),
    reason text not null,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists confirmation_decisions (
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

create index if not exists idx_app_settings_scope on app_settings(scope);
create index if not exists idx_integration_statuses_status on integration_statuses(status);
create index if not exists idx_entity_links_source on entity_links(source_type, source_id);
create index if not exists idx_entity_links_target on entity_links(target_type, target_id);
create index if not exists idx_entity_links_relation on entity_links(relation_type);
create index if not exists idx_log_references_scope_created on log_references(log_scope, created_at);
create index if not exists idx_file_references_workspace_path on file_references(workspace_key, relative_path);
create index if not exists idx_confirmation_decisions_category_created on confirmation_decisions(action_category, created_at);
create index if not exists idx_confirmation_decisions_event on confirmation_decisions(event_id);
