create table if not exists review_records (
    id text primary key,
    subject_type text not null check (subject_type in ('task', 'agent_run')),
    subject_id text not null check (length(trim(subject_id)) > 0),
    task_id text not null,
    run_id text,
    reviewer_profile_id text,
    state text not null check (state in ('approved', 'required_fixes', 'blocked_insufficient_evidence')),
    verdict text not null check (verdict in ('approved', 'required_fixes', 'blocked_insufficient_evidence')),
    evidence_summary text not null check (length(trim(evidence_summary)) > 0),
    required_fixes_json text not null default '[]' check (json_valid(required_fixes_json)),
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    foreign key (task_id) references tasks(id) on delete restrict,
    foreign key (run_id) references agent_runs(id) on delete restrict,
    foreign key (reviewer_profile_id) references agent_profiles(id) on delete set null,
    check (subject_type != 'task' or subject_id = task_id),
    check (subject_type != 'agent_run' or (run_id is not null and subject_id = run_id)),
    check (state = verdict),
    check (verdict != 'required_fixes' or json_type(required_fixes_json) = 'array' and json_array_length(required_fixes_json) > 0)
);

create trigger if not exists trg_review_records_agent_run_task_match_insert
before insert on review_records
when new.subject_type = 'agent_run'
begin
    select raise(abort, 'review agent_run task_id mismatch')
    where not exists (
        select 1 from agent_runs
        where id = new.run_id and task_id = new.task_id
    );
end;

create trigger if not exists trg_review_records_agent_run_task_match_update
before update on review_records
when new.subject_type = 'agent_run'
begin
    select raise(abort, 'review agent_run task_id mismatch')
    where not exists (
        select 1 from agent_runs
        where id = new.run_id and task_id = new.task_id
    );
end;

create index if not exists idx_review_records_task on review_records(task_id, created_at);
create index if not exists idx_review_records_run on review_records(run_id, created_at);
create index if not exists idx_review_records_state on review_records(state, updated_at);
create index if not exists idx_review_records_subject on review_records(subject_type, subject_id, created_at);
