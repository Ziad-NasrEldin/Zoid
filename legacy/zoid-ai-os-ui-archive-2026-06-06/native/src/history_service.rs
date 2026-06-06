use rusqlite::{params, params_from_iter, types::Value as SqlValue, Connection};
use serde::Serialize;
use std::collections::HashSet;

use crate::{
    map_repository_error, read_agent_run_required, read_event_record, read_notification,
    read_task_record, EventRecord, EventTargetRecord, RepoResult, RepositoryError,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct HistoryEntityRef {
    pub entity_type: String,
    pub entity_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct HistoryCursor {
    pub timestamp: String,
    pub event_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryQuery {
    pub primary: HistoryEntityRef,
    pub include_related: bool,
    pub limit: usize,
    pub before: Option<HistoryCursor>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct HistoryTimelineItem {
    pub event: EventRecord,
    pub matched_entities: Vec<EventTargetRecord>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct HistoryTimeline {
    pub primary: HistoryEntityRef,
    pub items: Vec<HistoryTimelineItem>,
}

#[allow(dead_code)]
pub(crate) fn list_task_history(
    connection: &Connection,
    task_id: &str,
    limit: usize,
    before: Option<HistoryCursor>,
) -> RepoResult<Vec<HistoryTimelineItem>> {
    read_task_record(connection, task_id)?;
    list_entity_history(
        connection,
        HistoryQuery {
            primary: HistoryEntityRef {
                entity_type: "task".to_string(),
                entity_id: task_id.to_string(),
            },
            include_related: true,
            limit,
            before,
        },
    )
}

#[allow(dead_code)]
pub(crate) fn list_run_history(
    connection: &Connection,
    run_id: &str,
    limit: usize,
    before: Option<HistoryCursor>,
) -> RepoResult<Vec<HistoryTimelineItem>> {
    read_agent_run_required(connection, run_id)?;
    list_history_for_entities(
        connection,
        HistoryEntityRef {
            entity_type: "agent_run".to_string(),
            entity_id: run_id.to_string(),
        },
        run_history_entity_set(connection, run_id)?,
        limit,
        before,
    )
}

#[allow(dead_code)]
pub(crate) fn list_notification_history(
    connection: &Connection,
    notification_id: &str,
    limit: usize,
    before: Option<HistoryCursor>,
) -> RepoResult<Vec<HistoryTimelineItem>> {
    read_notification(connection, notification_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "notifications",
        key: notification_id.to_string(),
    })?;
    list_entity_history(
        connection,
        HistoryQuery {
            primary: HistoryEntityRef {
                entity_type: "notification".to_string(),
                entity_id: notification_id.to_string(),
            },
            include_related: true,
            limit,
            before,
        },
    )
}

#[allow(dead_code)]
pub(crate) fn list_entity_history(
    connection: &Connection,
    query: HistoryQuery,
) -> RepoResult<Vec<HistoryTimelineItem>> {
    let primary = normalize_history_entity(query.primary)?;
    let entities = history_entity_set(connection, &primary, query.include_related)?;
    list_history_for_entities(connection, primary, entities, query.limit, query.before)
}

fn list_history_for_entities(
    connection: &Connection,
    _primary: HistoryEntityRef,
    entities: Vec<(String, String)>,
    limit: usize,
    before: Option<HistoryCursor>,
) -> RepoResult<Vec<HistoryTimelineItem>> {
    if entities.is_empty() {
        return Ok(Vec::new());
    }

    let mut clauses = Vec::new();
    let mut values = Vec::new();
    for (entity_type, entity_id) in &entities {
        clauses.push("(t.entity_type = ? and t.entity_id = ?)".to_string());
        values.push(SqlValue::Text(entity_type.clone()));
        values.push(SqlValue::Text(entity_id.clone()));
    }

    let mut sql = format!(
        "
        select distinct e.id, e.timestamp
        from events e
        join event_targets t on t.event_id = e.id
        where ({})
        ",
        clauses.join(" or ")
    );
    if let Some(cursor) = &before {
        sql.push_str(" and (e.timestamp < ? or (e.timestamp = ? and e.id < ?))");
        values.push(SqlValue::Text(cursor.timestamp.clone()));
        values.push(SqlValue::Text(cursor.timestamp.clone()));
        values.push(SqlValue::Text(cursor.event_id.clone()));
    }
    sql.push_str(" order by e.timestamp desc, e.id desc limit ?");
    values.push(SqlValue::Integer(normalize_history_limit(limit)));

    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| map_repository_error("events", error))?;
    let rows = statement
        .query_map(params_from_iter(values), |row| row.get::<_, String>(0))
        .map_err(|error| map_repository_error("events", error))?;

    let entity_lookup: HashSet<(String, String)> = entities.into_iter().collect();
    let mut items = Vec::new();
    for row in rows {
        let event_id = row.map_err(|error| map_repository_error("events", error))?;
        let event = read_event_record(connection, &event_id)?;
        let matched_entities = event
            .targets
            .iter()
            .filter(|target| {
                entity_lookup.contains(&(target.entity_type.clone(), target.entity_id.clone()))
            })
            .cloned()
            .collect::<Vec<_>>();
        items.push(HistoryTimelineItem {
            event,
            matched_entities,
        });
    }
    Ok(items)
}

fn normalize_history_entity(entity: HistoryEntityRef) -> RepoResult<HistoryEntityRef> {
    let entity_type = entity.entity_type.trim();
    let entity_id = entity.entity_id.trim();
    if entity_type.is_empty() || entity_id.is_empty() {
        return Err(RepositoryError::Constraint {
            entity: "history",
            message: "history entity_type and entity_id are required".to_string(),
        });
    }
    if entity_type.len() > 128 || entity_id.len() > 512 {
        return Err(RepositoryError::Constraint {
            entity: "history",
            message: "history entity_type or entity_id is too large".to_string(),
        });
    }
    Ok(HistoryEntityRef {
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
    })
}

fn history_entity_set(
    connection: &Connection,
    primary: &HistoryEntityRef,
    include_related: bool,
) -> RepoResult<Vec<(String, String)>> {
    let mut seen = HashSet::new();
    let mut entities = Vec::new();
    push_entity(
        &mut seen,
        &mut entities,
        &primary.entity_type,
        &primary.entity_id,
    );

    if include_related {
        let mut statement = connection
            .prepare(
                "
                select target_type, target_id
                from entity_links
                where source_type = ?1 and source_id = ?2
                union
                select source_type, source_id
                from entity_links
                where target_type = ?1 and target_id = ?2
                ",
            )
            .map_err(|error| map_repository_error("entity_links", error))?;
        let rows = statement
            .query_map(params![primary.entity_type, primary.entity_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| map_repository_error("entity_links", error))?;
        for row in rows {
            let (entity_type, entity_id) =
                row.map_err(|error| map_repository_error("entity_links", error))?;
            push_entity(&mut seen, &mut entities, &entity_type, &entity_id);
        }
    }

    Ok(entities)
}

fn run_history_entity_set(
    connection: &Connection,
    run_id: &str,
) -> RepoResult<Vec<(String, String)>> {
    let mut seen = HashSet::new();
    let mut entities = Vec::new();
    push_entity(&mut seen, &mut entities, "agent_run", run_id);

    let mut review_ids = Vec::new();
    let mut statement = connection
        .prepare(
            "
            select target_type, target_id
            from entity_links
            where source_type = 'agent_run'
                and source_id = ?1
                and target_type in ('review_record', 'notification')
            ",
        )
        .map_err(|error| map_repository_error("entity_links", error))?;
    let rows = statement
        .query_map(params![run_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| map_repository_error("entity_links", error))?;
    for row in rows {
        let (entity_type, entity_id) =
            row.map_err(|error| map_repository_error("entity_links", error))?;
        if entity_type == "review_record" {
            review_ids.push(entity_id.clone());
        }
        push_entity(&mut seen, &mut entities, &entity_type, &entity_id);
    }

    for review_id in review_ids {
        let mut notification_statement = connection
            .prepare(
                "
                select target_id
                from entity_links
                where source_type = 'review_record'
                    and source_id = ?1
                    and target_type = 'notification'
                ",
            )
            .map_err(|error| map_repository_error("entity_links", error))?;
        let notification_rows = notification_statement
            .query_map(params![review_id], |row| row.get::<_, String>(0))
            .map_err(|error| map_repository_error("entity_links", error))?;
        for row in notification_rows {
            let notification_id =
                row.map_err(|error| map_repository_error("entity_links", error))?;
            push_entity(&mut seen, &mut entities, "notification", &notification_id);
        }
    }

    Ok(entities)
}

fn push_entity(
    seen: &mut HashSet<(String, String)>,
    entities: &mut Vec<(String, String)>,
    entity_type: &str,
    entity_id: &str,
) {
    let key = (entity_type.to_string(), entity_id.to_string());
    if seen.insert(key.clone()) {
        entities.push(key);
    }
}

fn normalize_history_limit(limit: usize) -> i64 {
    let bounded = if limit == 0 { 50 } else { limit.min(200) };
    i64::try_from(bounded).unwrap_or(200)
}
