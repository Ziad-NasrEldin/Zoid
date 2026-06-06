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
