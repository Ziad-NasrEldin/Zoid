create table if not exists notes (
    id text primary key,
    title text not null check (length(trim(title)) > 0 and length(cast(title as blob)) <= 256),
    slug text check (slug is null or (length(trim(slug)) > 0 and length(cast(slug as blob)) <= 160)),
    relative_path text not null check (
        length(trim(relative_path)) > 0
        and length(cast(relative_path as blob)) <= 1024
        and relative_path not like '/%'
        and relative_path not like '%..%'
    ),
    status text not null default 'active' check (status in ('active', 'draft', 'trashed', 'deleted', 'missing', 'conflicted')),
    conflict_state text not null default 'none' check (conflict_state in ('none', 'duplicate_id', 'path_missing', 'external_edit', 'manual_rename', 'metadata_mismatch')),
    frontmatter_json text not null default '{}' check (json_valid(frontmatter_json)),
    body_digest text check (body_digest is null or length(trim(body_digest)) > 0),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    deleted_at text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    check (status != 'deleted' or deleted_at is not null),
    check (status != 'conflicted' or conflict_state != 'none')
);

create unique index if not exists idx_notes_relative_path_active
on notes(relative_path)
where deleted_at is null and status != 'deleted';
create index if not exists idx_notes_status_updated on notes(status, updated_at, id);
create index if not exists idx_notes_conflict_state on notes(conflict_state, updated_at, id);
create index if not exists idx_notes_slug on notes(slug) where slug is not null;

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

create table if not exists file_references_p3_new (
    id text primary key,
    root_key text not null check (root_key in ('zoid_visible', 'notes', 'content', 'assets', 'imports', 'exports', 'backups')),
    relative_path text not null check (
        length(trim(relative_path)) > 0
        and length(cast(relative_path as blob)) <= 1024
        and relative_path not like '/%'
        and relative_path not like '%..%'
    ),
    display_name text not null check (length(trim(display_name)) > 0 and length(cast(display_name as blob)) <= 256),
    file_kind text not null check (file_kind in ('markdown_note', 'document', 'image', 'audio', 'video', 'archive', 'code', 'folder', 'other')),
    mime_type text check (mime_type is null or (length(trim(mime_type)) > 0 and length(cast(mime_type as blob)) <= 128)),
    extension text check (extension is null or (length(trim(extension)) > 0 and length(cast(extension as blob)) <= 32)),
    byte_size integer check (byte_size is null or byte_size >= 0),
    content_fingerprint text check (content_fingerprint is null or length(trim(content_fingerprint)) > 0),
    status text not null default 'indexed' check (status in ('indexed', 'missing', 'trashed', 'deleted', 'conflicted', 'ignored')),
    conflict_state text not null default 'none' check (conflict_state in ('none', 'path_missing', 'manual_rename', 'external_edit', 'duplicate_reference', 'metadata_mismatch')),
    last_seen_at text,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    deleted_at text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    check (status != 'deleted' or deleted_at is not null),
    check (status != 'conflicted' or conflict_state != 'none')
);

insert or ignore into file_references_p3_new (
    id,
    root_key,
    relative_path,
    display_name,
    file_kind,
    mime_type,
    content_fingerprint,
    status,
    conflict_state,
    created_at,
    updated_at,
    metadata_json
)
select
    id,
    case
        when workspace_key in ('notes', 'content', 'assets', 'imports', 'exports', 'backups') then workspace_key
        else 'zoid_visible'
    end,
    relative_path,
    display_name,
    case
        when lower(relative_path) like '%.md' then 'markdown_note'
        else 'document'
    end,
    mime_type,
    content_hash,
    'indexed',
    'none',
    created_at,
    updated_at,
    metadata_json
from file_references;

drop table file_references;
alter table file_references_p3_new rename to file_references;

create unique index if not exists idx_file_references_root_path_active
on file_references(root_key, relative_path)
where deleted_at is null and status != 'deleted';
create index if not exists idx_file_references_status_updated on file_references(status, updated_at, id);
create index if not exists idx_file_references_conflict_state on file_references(conflict_state, updated_at, id);
create index if not exists idx_file_references_kind on file_references(file_kind, updated_at, id);

create table if not exists knowledge_index_entries (
    id text primary key,
    entity_type text not null check (entity_type in ('note', 'file')),
    entity_id text not null check (length(trim(entity_id)) > 0),
    source_type text not null check (source_type in ('markdown_frontmatter', 'markdown_body', 'file_metadata', 'file_preview')),
    title text check (title is null or (length(trim(title)) > 0 and length(cast(title as blob)) <= 256)),
    excerpt text check (excerpt is null or length(cast(excerpt as blob)) <= 2048),
    search_text text check (search_text is null or length(cast(search_text as blob)) <= 65536),
    content_digest text check (content_digest is null or length(trim(content_digest)) > 0),
    source_modified_at text,
    scan_state text not null default 'current' check (scan_state in ('current', 'stale', 'missing', 'conflicted', 'failed')),
    indexed_at text not null default current_timestamp,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    check (entity_type = 'note' or source_type in ('file_metadata', 'file_preview')),
    check (entity_type = 'file' or source_type in ('markdown_frontmatter', 'markdown_body'))
);

create unique index if not exists idx_knowledge_index_entity_source
on knowledge_index_entries(entity_type, entity_id, source_type);
create index if not exists idx_knowledge_index_scan_state on knowledge_index_entries(scan_state, indexed_at, id);
create index if not exists idx_knowledge_index_title on knowledge_index_entries(title) where title is not null;
