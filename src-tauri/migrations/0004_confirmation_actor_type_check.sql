drop table if exists confirmation_decisions_actor_type_upgrade;

create table confirmation_decisions_actor_type_upgrade (
    id text primary key,
    action_category text not null,
    decision text not null check (decision in ('approved', 'denied', 'cancelled', 'expired')),
    actor_type text not null check (actor_type in ('human', 'reviewer', 'clear_task', 'reviewed_clear_task', 'system')),
    actor_id text,
    summary text not null,
    event_id text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    foreign key (action_category) references action_policies(category) on update cascade,
    foreign key (event_id) references events(id) on delete set null
);

insert into confirmation_decisions_actor_type_upgrade (
    id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json, created_at
)
select id, action_category, decision, actor_type, actor_id, summary, event_id, metadata_json, created_at
from confirmation_decisions;

drop table confirmation_decisions;
alter table confirmation_decisions_actor_type_upgrade rename to confirmation_decisions;

create index if not exists idx_confirmation_decisions_category_created on confirmation_decisions(action_category, created_at);
create index if not exists idx_confirmation_decisions_event on confirmation_decisions(event_id);

pragma foreign_key_check;
