create table if not exists schema_migrations (
    version integer primary key,
    name text not null,
    applied_at text not null default current_timestamp
);

create table if not exists workspaces (
    id text primary key,
    label text not null,
    description text not null default '',
    position integer not null,
    enabled integer not null default 1,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists events (
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

create table if not exists event_targets (
    event_id text not null,
    entity_type text not null,
    entity_id text not null,
    relation_type text not null,
    primary key (event_id, entity_type, entity_id, relation_type),
    foreign key (event_id) references events(id) on delete cascade
);

create index if not exists idx_events_workspace_timestamp on events(workspace_key, timestamp);
create index if not exists idx_event_targets_entity on event_targets(entity_type, entity_id);
