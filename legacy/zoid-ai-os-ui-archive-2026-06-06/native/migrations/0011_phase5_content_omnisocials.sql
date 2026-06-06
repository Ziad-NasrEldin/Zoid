create table if not exists content_plans (
    id text primary key,
    title text not null check(length(trim(title)) > 0),
    pillar text not null default '',
    status text not null default 'active' check(status in ('active','archived')),
    owner_actor_type text not null default 'human',
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists content_pieces (
    id text primary key,
    plan_id text not null references content_plans(id) on delete cascade,
    title text not null check(length(trim(title)) > 0),
    body_markdown text not null default '',
    status text not null default 'draft' check(status in ('draft','review_ready','approved','scheduled','published','blocked','archived')),
    platforms_json text not null default '[]' check(json_valid(platforms_json)),
    required_gate text not null default 'specialist_review' check(required_gate in ('none','specialist_review')),
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists media_assets (
    id text primary key,
    piece_id text not null references content_pieces(id) on delete cascade,
    asset_kind text not null check(asset_kind in ('image','video','document','link')),
    storage_ref text not null check(length(trim(storage_ref)) > 0),
    mime_type text,
    byte_size integer,
    width integer,
    height integer,
    duration_seconds integer,
    alt_text text not null default '',
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    created_at text not null default current_timestamp
);

create table if not exists content_review_gates (
    id text primary key,
    piece_id text not null references content_pieces(id) on delete cascade,
    gate_type text not null default 'specialist_review',
    status text not null default 'pending' check(status in ('pending','approved','rejected')),
    reviewer_actor_type text,
    reviewer_actor_id text,
    evidence_summary text not null default '',
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    decided_at text
);

create table if not exists content_schedules (
    id text primary key,
    piece_id text not null references content_pieces(id) on delete cascade,
    platform text not null,
    scheduled_for text not null,
    status text not null default 'intent' check(status in ('intent','scheduled','published','cancelled','blocked','failed')),
    confirmation_id text references confirmation_decisions(id) on delete set null,
    provider_ref text,
    failure_report text,
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists content_verification_records (
    id text primary key,
    piece_id text references content_pieces(id) on delete cascade,
    schedule_id text references content_schedules(id) on delete cascade,
    platform text not null,
    action_type text not null check(action_type in ('upload','schedule','publish','review','validation')),
    outcome text not null check(outcome in ('passed','blocked','failed','manual')),
    provider_status text,
    failure_report text,
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    created_at text not null default current_timestamp
);

create table if not exists omnisocials_accounts (
    id text primary key,
    platform text not null default 'omnisocials',
    state text not null default 'not_configured' check(state in ('not_configured','needs_permission','connected','error','disabled_by_policy','blocked','failed')),
    credential_ref text,
    status_note text not null default 'OmniSocials credentials are not configured; external writes fail closed.',
    metadata_json text not null default '{}' check(json_valid(metadata_json)),
    updated_at text not null default current_timestamp
);

create index if not exists idx_content_plans_status_updated on content_plans(status, updated_at);
create index if not exists idx_content_pieces_plan_status on content_pieces(plan_id, status, updated_at);
create index if not exists idx_media_assets_piece_kind on media_assets(piece_id, asset_kind, created_at);
create index if not exists idx_content_review_gates_piece_type_status on content_review_gates(piece_id, gate_type, status, created_at);
create index if not exists idx_content_schedules_piece_platform_status on content_schedules(piece_id, platform, status, scheduled_for);
create index if not exists idx_content_verification_piece_action on content_verification_records(piece_id, action_type, outcome, created_at);
create index if not exists idx_content_verification_schedule_action on content_verification_records(schedule_id, action_type, outcome, created_at);
create index if not exists idx_omnisocials_accounts_platform_state on omnisocials_accounts(platform, state);

insert into omnisocials_accounts (id, platform, state, credential_ref, status_note, metadata_json)
values ('omnisocials-default', 'omnisocials', 'not_configured', null, 'OmniSocials credentials are not configured; upload/schedule/publish fail closed.', '{}')
on conflict(id) do nothing;

