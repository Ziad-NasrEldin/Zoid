create table if not exists browser_tabs (
    id text primary key,
    workspace_key text not null default 'browser',
    profile_key text not null default 'default',
    url text not null,
    title text not null default '',
    http_status integer,
    state text not null default 'saved',
    opened_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    closed_at text,
    manual_note text not null default '',
    metadata_json text not null default '{}',
    check (state in ('open','saved','closed','blocked','unsupported'))
);

create index if not exists idx_browser_tabs_workspace_profile_updated on browser_tabs(workspace_key, profile_key, updated_at desc);

create table if not exists browser_captures (
    id text primary key,
    tab_id text,
    workspace_key text not null default 'browser',
    profile_key text not null default 'default',
    url text not null,
    title text not null default '',
    captured_at text not null default current_timestamp,
    screenshot_path text,
    screenshot_supported integer not null default 0,
    capture_mode text not null default 'metadata_fallback',
    http_status integer,
    manual_note text not null default '',
    metadata_json text not null default '{}',
    foreign key (tab_id) references browser_tabs(id) on delete set null,
    check (capture_mode in ('screenshot','metadata_fallback'))
);

create index if not exists idx_browser_captures_workspace_profile_captured on browser_captures(workspace_key, profile_key, captured_at desc);

create table if not exists browser_capture_links (
    capture_id text not null,
    entity_type text not null,
    entity_id text not null,
    relation_type text not null default 'evidence',
    created_at text not null default current_timestamp,
    primary key (capture_id, entity_type, entity_id, relation_type),
    foreign key (capture_id) references browser_captures(id) on delete cascade,
    check (entity_type in ('launch_gate','task','note','product','content_piece'))
);

create index if not exists idx_browser_capture_links_entity on browser_capture_links(entity_type, entity_id);

create table if not exists widget_configs (
    workspace_key text not null,
    profile_key text not null default 'default',
    widget_key text not null,
    visible integer not null default 1,
    position integer not null,
    size text not null default 'medium',
    updated_at text not null default current_timestamp,
    primary key (workspace_key, profile_key, widget_key),
    check (size in ('small','medium','large'))
);
