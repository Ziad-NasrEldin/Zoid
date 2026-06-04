create table if not exists repo_profiles (
    id text primary key,
    display_name text not null check (length(trim(display_name)) > 0 and length(cast(display_name as blob)) <= 256),
    root_path text not null check (length(trim(root_path)) > 0 and length(cast(root_path as blob)) <= 2048),
    profile_type text not null check (profile_type in ('product_app', 'website', 'library', 'experiment', 'client_project', 'content_docs', 'other')),
    default_branch text check (default_branch is null or (length(trim(default_branch)) > 0 and length(cast(default_branch as blob)) <= 128)),
    package_manager text check (package_manager is null or (length(trim(package_manager)) > 0 and length(cast(package_manager as blob)) <= 64)),
    linked_product_id text,
    status text not null default 'active' check (status in ('active', 'archived', 'missing')),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    unique (root_path)
);

create index if not exists idx_repo_profiles_type_updated on repo_profiles(profile_type, updated_at, id);
create index if not exists idx_repo_profiles_product on repo_profiles(linked_product_id) where linked_product_id is not null;

create table if not exists launch_gates (
    id text primary key,
    repo_id text not null references repo_profiles(id) on delete restrict,
    product_id text,
    task_id text references tasks(id) on delete set null,
    state text not null default 'verification_blocked' check (state in ('not_ready', 'ready_to_verify', 'verification_blocked', 'verified', 'failed')),
    final_verdict text,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    check (final_verdict is null or length(trim(final_verdict)) > 0)
);

create index if not exists idx_launch_gates_repo_updated on launch_gates(repo_id, updated_at, id);
create index if not exists idx_launch_gates_state_updated on launch_gates(state, updated_at, id);
create index if not exists idx_launch_gates_task on launch_gates(task_id) where task_id is not null;

create table if not exists launch_gate_evidence (
    id text primary key,
    launch_gate_id text not null references launch_gates(id) on delete cascade,
    evidence_type text not null check (evidence_type in ('manual_note', 'url_status', 'screenshot', 'test_output', 'deployment_record')),
    label text not null check (length(trim(label)) > 0 and length(cast(label as blob)) <= 256),
    url text check (url is null or length(cast(url as blob)) <= 2048),
    status_code integer check (status_code is null or (status_code >= 100 and status_code <= 599)),
    manual_note text check (manual_note is null or length(cast(manual_note as blob)) <= 4096),
    created_at text not null default current_timestamp,
    metadata_json text not null default '{}' check (json_valid(metadata_json))
);

create index if not exists idx_launch_gate_evidence_gate_created on launch_gate_evidence(launch_gate_id, created_at, id);

insert or ignore into integration_statuses (
    integration_key, display_name, status, config_json, credential_ref, last_checked_at, updated_at
) values
    ('github', 'GitHub', 'not_configured', '{"phase":"4","scope":"state_only","note":"No deep GitHub automation or git read operations are implemented."}', null, null, current_timestamp),
    ('vercel', 'Vercel', 'not_configured', '{"phase":"4","scope":"state_only","note":"No deploy execution is implemented; Launch Gate evidence is local/truthful."}', null, null, current_timestamp);
