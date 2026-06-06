create table if not exists notifications (
    id text primary key,
    notification_type text not null check (notification_type in ('completion', 'blocker', 'failure', 'review_required', 'attention')),
    title text not null check (length(trim(title)) > 0),
    message text not null check (length(trim(message)) > 0),
    severity text not null check (severity in ('info', 'success', 'warning', 'error', 'critical')),
    state text not null default 'pending' check (state in ('pending', 'delivered', 'read', 'action_required', 'resolved', 'dismissed', 'failed')),
    action_route text,
    task_id text,
    run_id text,
    review_record_id text,
    read_at text,
    dismissed_at text,
    resolved_at text,
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    foreign key (task_id) references tasks(id) on delete restrict,
    foreign key (run_id) references agent_runs(id) on delete restrict,
    foreign key (review_record_id) references review_records(id) on delete restrict,
    check (state != 'read' or read_at is not null),
    check (state != 'dismissed' or dismissed_at is not null),
    check (state != 'resolved' or resolved_at is not null),
    check (state not in ('pending', 'delivered', 'action_required', 'failed') or (read_at is null and dismissed_at is null and resolved_at is null))
);

create trigger if not exists trg_notifications_run_task_match_insert
before insert on notifications
when new.run_id is not null and new.task_id is not null
begin
    select raise(abort, 'notification run task_id mismatch')
    where not exists (
        select 1 from agent_runs
        where id = new.run_id and task_id = new.task_id
    );
end;

create trigger if not exists trg_notifications_run_task_match_update
before update on notifications
when new.run_id is not null and new.task_id is not null
begin
    select raise(abort, 'notification run task_id mismatch')
    where not exists (
        select 1 from agent_runs
        where id = new.run_id and task_id = new.task_id
    );
end;

create trigger if not exists trg_notifications_review_task_match_insert
before insert on notifications
when new.review_record_id is not null and new.task_id is not null
begin
    select raise(abort, 'notification review task_id mismatch')
    where not exists (
        select 1 from review_records
        where id = new.review_record_id and task_id = new.task_id
    );
end;

create trigger if not exists trg_notifications_review_task_match_update
before update on notifications
when new.review_record_id is not null and new.task_id is not null
begin
    select raise(abort, 'notification review task_id mismatch')
    where not exists (
        select 1 from review_records
        where id = new.review_record_id and task_id = new.task_id
    );
end;

create trigger if not exists trg_notifications_review_run_match_insert
before insert on notifications
when new.review_record_id is not null and new.run_id is not null
begin
    select raise(abort, 'notification review run_id mismatch')
    where not exists (
        select 1
        from review_records review
        join agent_runs run on run.id = new.run_id
        where review.id = new.review_record_id
          and run.task_id = review.task_id
          and (review.run_id is null or review.run_id = new.run_id)
    );
end;

create trigger if not exists trg_notifications_review_run_match_update
before update on notifications
when new.review_record_id is not null and new.run_id is not null
begin
    select raise(abort, 'notification review run_id mismatch')
    where not exists (
        select 1
        from review_records review
        join agent_runs run on run.id = new.run_id
        where review.id = new.review_record_id
          and run.task_id = review.task_id
          and (review.run_id is null or review.run_id = new.run_id)
    );
end;

create index if not exists idx_notifications_state_updated on notifications(state, updated_at);
create index if not exists idx_notifications_task on notifications(task_id, created_at);
create index if not exists idx_notifications_run on notifications(run_id, created_at);
create index if not exists idx_notifications_review on notifications(review_record_id, created_at);
create index if not exists idx_notifications_type_severity on notifications(notification_type, severity, created_at);
