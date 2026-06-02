use rusqlite::Connection;

use crate::{
    create_notification, dismiss_notification, list_inbox_notifications,
    mark_notification_delivered, mark_notification_failed, mark_notification_read,
    read_notification, require_notification_action, resolve_notification, NotificationCreateInput,
    NotificationRecord, NotificationSeverity, NotificationType, RepoResult, RepositoryError,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct NotificationServiceCreateInput {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub action_route: Option<String>,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
    pub review_record_id: Option<String>,
    pub metadata_json: String,
}

#[allow(dead_code)]
pub(crate) fn create_notification_service(
    connection: &Connection,
    input: NotificationServiceCreateInput,
) -> RepoResult<NotificationRecord> {
    create_notification(
        connection,
        NotificationCreateInput {
            notification_type: input.notification_type,
            title: input.title,
            message: input.message,
            severity: input.severity,
            action_route: input.action_route,
            task_id: input.task_id,
            run_id: input.run_id,
            review_record_id: input.review_record_id,
            metadata_json: input.metadata_json,
        },
    )
}

#[allow(dead_code)]
pub(crate) fn read_notification_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    read_notification(connection, notification_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "notifications",
        key: notification_id.to_string(),
    })
}

#[allow(dead_code)]
pub(crate) fn list_inbox_notification_service(
    connection: &Connection,
    active_only: bool,
    limit: i64,
) -> RepoResult<Vec<NotificationRecord>> {
    list_inbox_notifications(connection, active_only, limit)
}

#[allow(dead_code)]
pub(crate) fn deliver_notification_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    mark_notification_delivered(connection, notification_id)
}

#[allow(dead_code)]
pub(crate) fn require_notification_action_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    require_notification_action(connection, notification_id)
}

#[allow(dead_code)]
pub(crate) fn fail_notification_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    mark_notification_failed(connection, notification_id)
}

#[allow(dead_code)]
pub(crate) fn read_mark_notification_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    mark_notification_read(connection, notification_id)
}

#[allow(dead_code)]
pub(crate) fn dismiss_notification_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    dismiss_notification(connection, notification_id)
}

#[allow(dead_code)]
pub(crate) fn resolve_notification_service(
    connection: &Connection,
    notification_id: &str,
) -> RepoResult<NotificationRecord> {
    resolve_notification(connection, notification_id)
}
