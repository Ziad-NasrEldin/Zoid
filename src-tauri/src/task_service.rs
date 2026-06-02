use rusqlite::{params, Connection};
use serde_json::Value;

use crate::{
    archive_task, create_event_record, create_task_record, list_active_tasks,
    normalize_task_detail, normalize_task_title, normalize_task_workspace_key, read_task_record,
    soft_delete_task, update_task_status, validate_no_secret_json, EventCreateInput,
    EventTargetInput, RepoResult, TaskCreateInput, TaskPriority, TaskRecord, TaskStatus,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct TaskServiceCreateInput {
    pub title: String,
    pub detail: Option<String>,
    pub priority: Option<TaskPriority>,
    pub workspace_key: Option<String>,
    pub metadata_json: String,
}

#[allow(dead_code)]
impl TaskServiceCreateInput {
    pub(crate) fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            detail: None,
            priority: None,
            workspace_key: None,
            metadata_json: "{}".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(crate) struct TaskServiceUpdateInput {
    pub title: Option<String>,
    pub detail: Option<String>,
    pub priority: Option<TaskPriority>,
    pub workspace_key: Option<String>,
    pub metadata_json: Option<String>,
}

#[allow(dead_code)]
pub(crate) fn create_task_service(
    connection: &Connection,
    input: TaskServiceCreateInput,
) -> RepoResult<TaskRecord> {
    create_task_record(
        connection,
        TaskCreateInput {
            title: input.title,
            detail: input.detail,
            status: None,
            priority: input.priority,
            workspace_key: input.workspace_key,
            metadata_json: input.metadata_json,
        },
    )
}

#[allow(dead_code)]
pub(crate) fn list_task_service(connection: &Connection) -> RepoResult<Vec<TaskRecord>> {
    list_active_tasks(connection)
}

#[allow(dead_code)]
pub(crate) fn read_task_service(connection: &Connection, task_id: &str) -> RepoResult<TaskRecord> {
    read_task_record(connection, task_id)
}

#[allow(dead_code)]
pub(crate) fn update_task_service(
    connection: &Connection,
    task_id: &str,
    input: TaskServiceUpdateInput,
) -> RepoResult<TaskRecord> {
    let before = read_task_record(connection, task_id)?;
    let title = match input.title.as_deref() {
        Some(title) => normalize_task_title(title)?,
        None => before.title.clone(),
    };
    let detail = match input.detail.as_deref() {
        Some(detail) => normalize_task_detail(Some(detail))?,
        None => before.detail.clone(),
    };
    let priority = input.priority.unwrap_or(before.priority);
    let workspace_key = match input.workspace_key.as_deref() {
        Some(workspace_key) => normalize_task_workspace_key(Some(workspace_key))?,
        None => before.workspace_key.clone(),
    };
    let metadata_json = input
        .metadata_json
        .unwrap_or_else(|| before.metadata_json.clone());
    validate_no_secret_json("metadata_json", &metadata_json)?;

    if title == before.title
        && detail == before.detail
        && priority == before.priority
        && workspace_key == before.workspace_key
        && metadata_json == before.metadata_json
    {
        return Ok(before);
    }

    connection
        .execute_batch("savepoint update_task_service")
        .map_err(|error| crate::map_repository_error("tasks", error))?;

    let update_result = (|| -> RepoResult<TaskRecord> {
        connection
            .execute(
                "
                update tasks
                set title = ?2,
                    detail = ?3,
                    priority = ?4,
                    workspace_key = ?5,
                    metadata_json = ?6,
                    updated_at = current_timestamp
                where id = ?1
                ",
                params![
                    task_id,
                    title,
                    detail,
                    priority.as_str(),
                    workspace_key,
                    metadata_json
                ],
            )
            .map_err(|error| crate::map_repository_error("tasks", error))?;
        let task = read_task_record(connection, task_id)?;
        let changed_fields = task_changed_fields(&before, &task);
        let metadata = serde_json::json!({
            "task_id": task.id,
            "changed_fields": changed_fields,
            "title": task.title,
            "status": task.status.as_str(),
            "priority": task.priority.as_str(),
            "input_metadata": serde_json::from_str::<Value>(&task.metadata_json).unwrap_or(Value::Null),
        })
        .to_string();
        create_event_record(
            connection,
            EventCreateInput {
                action_type: "task.updated",
                outcome: "succeeded",
                actor_type: "system",
                actor_id: None,
                workspace_key: Some(&task.workspace_key),
                summary: &format!("Updated task: {}", task.title),
                source: "task_service",
                metadata_json: &metadata,
                targets: vec![EventTargetInput {
                    entity_type: "task",
                    entity_id: &task.id,
                    relation_type: "primary",
                }],
            },
        )?;
        Ok(task)
    })();

    match update_result {
        Ok(task) => {
            connection
                .execute_batch("release savepoint update_task_service")
                .map_err(|error| crate::map_repository_error("tasks", error))?;
            Ok(task)
        }
        Err(error) => {
            let _ = connection.execute_batch(
                "rollback to savepoint update_task_service; release savepoint update_task_service",
            );
            Err(error)
        }
    }
}

#[allow(dead_code)]
pub(crate) fn update_task_service_status(
    connection: &Connection,
    task_id: &str,
    status: TaskStatus,
) -> RepoResult<TaskRecord> {
    update_task_status(connection, task_id, status)
}

#[allow(dead_code)]
pub(crate) fn archive_task_service(
    connection: &Connection,
    task_id: &str,
) -> RepoResult<TaskRecord> {
    archive_task(connection, task_id)
}

#[allow(dead_code)]
pub(crate) fn delete_task_service(
    connection: &Connection,
    task_id: &str,
) -> RepoResult<TaskRecord> {
    soft_delete_task(connection, task_id)
}

fn task_changed_fields(before: &TaskRecord, after: &TaskRecord) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if before.title != after.title {
        fields.push("title");
    }
    if before.detail != after.detail {
        fields.push("detail");
    }
    if before.priority != after.priority {
        fields.push("priority");
    }
    if before.workspace_key != after.workspace_key {
        fields.push("workspace_key");
    }
    if before.metadata_json != after.metadata_json {
        fields.push("metadata_json");
    }
    fields
}
