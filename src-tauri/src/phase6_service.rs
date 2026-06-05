use crate::{
    create_entity_link, create_event_record, map_repository_error, read_confirmation_decision,
    redact_secrets, validate_no_secret_json, ConfirmationDecisionState, EntityLinkCreateRequest,
    EntityLinkRecord, EventCreateInput, EventTargetInput, RepoResult, RepositoryError,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

fn next_id(prefix: &str) -> String {
    format!("{prefix}-{}", crate::now_millis())
}

fn json_or_default(value: Option<String>, default: &str) -> String {
    value.unwrap_or_else(|| default.to_string())
}

fn validate_json_field(field: &'static str, value: &str) -> RepoResult<()> {
    serde_json::from_str::<serde_json::Value>(value).map_err(|error| {
        RepositoryError::InvalidJson {
            field,
            message: error.to_string(),
        }
    })?;
    Ok(())
}

fn validate_safe_json(field: &'static str, value: &str) -> RepoResult<()> {
    validate_json_field(field, value)?;
    validate_no_secret_json(field, value)
}

fn validate_safe_text(
    entity: &'static str,
    field: &'static str,
    value: Option<&str>,
) -> RepoResult<()> {
    if let Some(value) = value {
        if redact_secrets(value).redaction_count > 0 {
            return Err(RepositoryError::SecretRejected {
                field,
                message: format!("{entity}.{field} contains secret-like material; store raw secrets only in Keychain"),
            });
        }
    }
    Ok(())
}

fn write_phase6_event(
    connection: &Connection,
    action_type: &'static str,
    workspace_key: &'static str,
    entity_type: &'static str,
    entity_id: &str,
    summary: &str,
) -> RepoResult<()> {
    let metadata_json = serde_json::json!({ "entity_id": entity_id }).to_string();
    create_event_record(
        connection,
        EventCreateInput {
            action_type,
            outcome: "succeeded",
            actor_type: "system",
            actor_id: None,
            workspace_key: Some(workspace_key),
            summary,
            source: "phase6_service",
            metadata_json: &metadata_json,
            targets: vec![EventTargetInput {
                entity_type,
                entity_id,
                relation_type: "primary",
            }],
        },
    )
    .map(|_| ())
}

fn reject_no_mutation(entity: &'static str, key: &str, action: &str) -> RepositoryError {
    RepositoryError::Constraint {
        entity,
        message: format!("{action} did not mutate an active {entity} row for id {key}"),
    }
}

fn require_approved_confirmation(
    connection: &Connection,
    confirmation_id: Option<&str>,
    expected_category: &str,
    entity: &'static str,
) -> RepoResult<String> {
    let id = confirmation_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| RepositoryError::Constraint {
            entity,
            message: format!("{expected_category} requires an approved confirmation_id"),
        })?;
    let confirmation =
        read_confirmation_decision(connection, id)?.ok_or_else(|| RepositoryError::NotFound {
            entity: "confirmation_decisions",
            key: id.to_string(),
        })?;
    if confirmation.action_category != expected_category {
        return Err(RepositoryError::Constraint {
            entity,
            message: format!(
                "confirmation category must be {expected_category}, got {}",
                confirmation.action_category
            ),
        });
    }
    if confirmation.decision != ConfirmationDecisionState::Approved {
        return Err(RepositoryError::Constraint {
            entity,
            message: "confirmation decision must be approved".to_string(),
        });
    }
    Ok(confirmation.id)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase6IntegrationStateRecord {
    pub(crate) key: String,
    pub(crate) state: String,
    pub(crate) safe_copy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BusinessCompanyRecord {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) domain: Option<String>,
    pub(crate) status: String,
    pub(crate) notes: Option<String>,
    pub(crate) metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BusinessContactRecord {
    pub(crate) id: String,
    pub(crate) company_id: Option<String>,
    pub(crate) full_name: String,
    pub(crate) email: Option<String>,
    pub(crate) phone: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) status: String,
    pub(crate) notes: Option<String>,
    pub(crate) metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FollowUpRecord {
    pub(crate) id: String,
    pub(crate) subject: String,
    pub(crate) due_at: Option<String>,
    pub(crate) state: String,
    pub(crate) priority: String,
    pub(crate) contact_id: Option<String>,
    pub(crate) company_id: Option<String>,
    pub(crate) product_id: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) note_id: Option<String>,
    pub(crate) email_ref_id: Option<String>,
    pub(crate) calendar_event_id: Option<String>,
    pub(crate) metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProductRecord {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) summary: Option<String>,
    pub(crate) owner_contact_id: Option<String>,
    pub(crate) metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EmailRefRecord {
    pub(crate) id: String,
    pub(crate) external_id: Option<String>,
    pub(crate) thread_id: Option<String>,
    pub(crate) subject: String,
    pub(crate) sender: Option<String>,
    pub(crate) recipients_json: String,
    pub(crate) snippet: Option<String>,
    pub(crate) state: String,
    pub(crate) confirmation_id: Option<String>,
    pub(crate) metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CalendarRefRecord {
    pub(crate) id: String,
    pub(crate) external_id: Option<String>,
    pub(crate) title: String,
    pub(crate) starts_at: String,
    pub(crate) ends_at: String,
    pub(crate) location: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) state: String,
    pub(crate) confirmation_id: Option<String>,
    pub(crate) metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct InboxAggregateRecord {
    pub(crate) id: String,
    pub(crate) item_type: String,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) state: String,
    pub(crate) priority: String,
    pub(crate) route: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Phase6OverviewRecord {
    pub(crate) integrations: Vec<Phase6IntegrationStateRecord>,
    pub(crate) inbox: Vec<InboxAggregateRecord>,
    pub(crate) calendar: Vec<CalendarRefRecord>,
    pub(crate) emails: Vec<EmailRefRecord>,
    pub(crate) companies: Vec<BusinessCompanyRecord>,
    pub(crate) contacts: Vec<BusinessContactRecord>,
    pub(crate) follow_ups: Vec<FollowUpRecord>,
    pub(crate) products: Vec<ProductRecord>,
    pub(crate) product_links: Vec<EntityLinkRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CompanyInput {
    pub(crate) name: String,
    pub(crate) domain: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ContactInput {
    pub(crate) company_id: Option<String>,
    pub(crate) full_name: String,
    pub(crate) email: Option<String>,
    pub(crate) phone: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FollowUpInput {
    pub(crate) subject: String,
    pub(crate) due_at: Option<String>,
    pub(crate) priority: Option<String>,
    pub(crate) contact_id: Option<String>,
    pub(crate) company_id: Option<String>,
    pub(crate) product_id: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) note_id: Option<String>,
    pub(crate) email_ref_id: Option<String>,
    pub(crate) calendar_event_id: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProductInput {
    pub(crate) name: String,
    pub(crate) status: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) owner_contact_id: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProductLinkInput {
    pub(crate) product_id: String,
    pub(crate) target_type: String,
    pub(crate) target_id: String,
    pub(crate) relation_type: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CalendarEventInput {
    pub(crate) title: String,
    pub(crate) starts_at: String,
    pub(crate) ends_at: String,
    pub(crate) location: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) confirmation_id: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EmailDraftInput {
    pub(crate) subject: String,
    pub(crate) recipients_json: Option<String>,
    pub(crate) snippet: Option<String>,
    pub(crate) thread_id: Option<String>,
    pub(crate) metadata_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EmailSendInput {
    pub(crate) confirmation_id: Option<String>,
}

fn ensure_non_empty(entity: &'static str, field: &str, value: &str) -> RepoResult<()> {
    if value.trim().is_empty() {
        return Err(RepositoryError::Constraint {
            entity,
            message: format!("{field} is required"),
        });
    }
    Ok(())
}

pub(crate) fn phase6_integration_states() -> Vec<Phase6IntegrationStateRecord> {
    vec![
        Phase6IntegrationStateRecord { key: "eventkit".to_string(), state: "needs_permission".to_string(), safe_copy: "Calendar access needs macOS permission before Apple Calendar sync can run.".to_string() },
        Phase6IntegrationStateRecord { key: "gmail".to_string(), state: "not_configured".to_string(), safe_copy: "Mail is safe and unconfigured until the account is connected; no message can be sent silently.".to_string() },
    ]
}

pub(crate) fn list_phase6_overview(connection: &Connection) -> RepoResult<Phase6OverviewRecord> {
    Ok(Phase6OverviewRecord {
        integrations: phase6_integration_states(),
        inbox: list_phase6_inbox(connection)?,
        calendar: list_calendar_events(connection)?,
        emails: list_emails(connection, None)?,
        companies: list_companies(connection)?,
        contacts: list_contacts(connection)?,
        follow_ups: list_follow_ups(connection)?,
        products: list_products(connection)?,
        product_links: list_product_links(connection)?,
    })
}

pub(crate) fn create_company(
    connection: &Connection,
    input: CompanyInput,
) -> RepoResult<BusinessCompanyRecord> {
    ensure_non_empty("companies", "name", &input.name)?;
    validate_safe_text("companies", "name", Some(&input.name))?;
    validate_safe_text("companies", "domain", input.domain.as_deref())?;
    validate_safe_text("companies", "notes", input.notes.as_deref())?;
    let id = next_id("company");
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    connection.execute(
        "insert into companies (id, name, domain, notes, metadata_json) values (?1, ?2, ?3, ?4, ?5)",
        params![id, input.name.trim(), input.domain, input.notes, metadata_json],
    ).map_err(|e| map_repository_error("companies", e))?;
    let record = read_company(connection, &id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "companies",
        key: id.clone(),
    })?;
    write_phase6_event(
        connection,
        "company.created",
        "business",
        "company",
        &id,
        &format!("Created company: {}", record.name),
    )?;
    Ok(record)
}

pub(crate) fn list_companies(connection: &Connection) -> RepoResult<Vec<BusinessCompanyRecord>> {
    let mut statement = connection.prepare("select id, name, domain, status, notes, metadata_json from companies where status != 'archived' order by updated_at desc, name asc")
        .map_err(|e| map_repository_error("companies", e))?;
    let rows = statement
        .query_map([], company_from_row)
        .map_err(|e| map_repository_error("companies", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("companies", e))?;
    Ok(rows)
}

fn read_company(connection: &Connection, id: &str) -> RepoResult<Option<BusinessCompanyRecord>> {
    connection
        .query_row(
            "select id, name, domain, status, notes, metadata_json from companies where id=?1",
            params![id],
            company_from_row,
        )
        .optional()
        .map_err(|e| map_repository_error("companies", e))
}

fn company_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<BusinessCompanyRecord> {
    Ok(BusinessCompanyRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        domain: row.get(2)?,
        status: row.get(3)?,
        notes: row.get(4)?,
        metadata_json: row.get(5)?,
    })
}

pub(crate) fn create_contact(
    connection: &Connection,
    input: ContactInput,
) -> RepoResult<BusinessContactRecord> {
    ensure_non_empty("contacts", "full_name", &input.full_name)?;
    validate_safe_text("contacts", "full_name", Some(&input.full_name))?;
    validate_safe_text("contacts", "email", input.email.as_deref())?;
    validate_safe_text("contacts", "phone", input.phone.as_deref())?;
    validate_safe_text("contacts", "role", input.role.as_deref())?;
    validate_safe_text("contacts", "notes", input.notes.as_deref())?;
    let id = next_id("contact");
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    connection.execute(
        "insert into contacts (id, company_id, full_name, email, phone, role, notes, metadata_json) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, input.company_id, input.full_name.trim(), input.email, input.phone, input.role, input.notes, metadata_json],
    ).map_err(|e| map_repository_error("contacts", e))?;
    let record = read_contact(connection, &id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "contacts",
        key: id.clone(),
    })?;
    write_phase6_event(
        connection,
        "contact.created",
        "business",
        "contact",
        &id,
        &format!("Created contact: {}", record.full_name),
    )?;
    Ok(record)
}

pub(crate) fn list_contacts(connection: &Connection) -> RepoResult<Vec<BusinessContactRecord>> {
    let mut statement = connection.prepare("select id, company_id, full_name, email, phone, role, status, notes, metadata_json from contacts where status != 'archived' order by updated_at desc, full_name asc")
        .map_err(|e| map_repository_error("contacts", e))?;
    let rows = statement
        .query_map([], contact_from_row)
        .map_err(|e| map_repository_error("contacts", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("contacts", e))?;
    Ok(rows)
}

fn read_contact(connection: &Connection, id: &str) -> RepoResult<Option<BusinessContactRecord>> {
    connection.query_row("select id, company_id, full_name, email, phone, role, status, notes, metadata_json from contacts where id=?1", params![id], contact_from_row)
        .optional().map_err(|e| map_repository_error("contacts", e))
}

fn contact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<BusinessContactRecord> {
    Ok(BusinessContactRecord {
        id: row.get(0)?,
        company_id: row.get(1)?,
        full_name: row.get(2)?,
        email: row.get(3)?,
        phone: row.get(4)?,
        role: row.get(5)?,
        status: row.get(6)?,
        notes: row.get(7)?,
        metadata_json: row.get(8)?,
    })
}

pub(crate) fn create_follow_up(
    connection: &Connection,
    input: FollowUpInput,
) -> RepoResult<FollowUpRecord> {
    ensure_non_empty("follow_ups", "subject", &input.subject)?;
    validate_safe_text("follow_ups", "subject", Some(&input.subject))?;
    validate_safe_text("follow_ups", "due_at", input.due_at.as_deref())?;
    let id = next_id("followup");
    let priority = input.priority.unwrap_or_else(|| "normal".to_string());
    validate_safe_text("follow_ups", "priority", Some(&priority))?;
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    connection.execute(
        "insert into follow_ups (id, subject, due_at, priority, contact_id, company_id, product_id, task_id, note_id, email_ref_id, calendar_event_id, metadata_json) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![id, input.subject.trim(), input.due_at, priority, input.contact_id, input.company_id, input.product_id, input.task_id, input.note_id, input.email_ref_id, input.calendar_event_id, metadata_json],
    ).map_err(|e| map_repository_error("follow_ups", e))?;
    let record = read_follow_up(connection, &id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "follow_ups",
        key: id.clone(),
    })?;
    write_phase6_event(
        connection,
        "follow_up.created",
        "business",
        "follow_up",
        &id,
        &format!("Created follow-up: {}", record.subject),
    )?;
    Ok(record)
}

pub(crate) fn list_follow_ups(connection: &Connection) -> RepoResult<Vec<FollowUpRecord>> {
    let mut statement = connection.prepare("select id, subject, due_at, state, priority, contact_id, company_id, product_id, task_id, note_id, email_ref_id, calendar_event_id, metadata_json from follow_ups where state not in ('done','dismissed') order by due_at is null, due_at asc, updated_at desc")
        .map_err(|e| map_repository_error("follow_ups", e))?;
    let rows = statement
        .query_map([], follow_up_from_row)
        .map_err(|e| map_repository_error("follow_ups", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("follow_ups", e))?;
    Ok(rows)
}

fn read_follow_up(connection: &Connection, id: &str) -> RepoResult<Option<FollowUpRecord>> {
    connection.query_row("select id, subject, due_at, state, priority, contact_id, company_id, product_id, task_id, note_id, email_ref_id, calendar_event_id, metadata_json from follow_ups where id=?1", params![id], follow_up_from_row)
        .optional().map_err(|e| map_repository_error("follow_ups", e))
}

fn follow_up_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FollowUpRecord> {
    Ok(FollowUpRecord {
        id: row.get(0)?,
        subject: row.get(1)?,
        due_at: row.get(2)?,
        state: row.get(3)?,
        priority: row.get(4)?,
        contact_id: row.get(5)?,
        company_id: row.get(6)?,
        product_id: row.get(7)?,
        task_id: row.get(8)?,
        note_id: row.get(9)?,
        email_ref_id: row.get(10)?,
        calendar_event_id: row.get(11)?,
        metadata_json: row.get(12)?,
    })
}

pub(crate) fn create_product(
    connection: &Connection,
    input: ProductInput,
) -> RepoResult<ProductRecord> {
    ensure_non_empty("products", "name", &input.name)?;
    validate_safe_text("products", "name", Some(&input.name))?;
    validate_safe_text("products", "status", input.status.as_deref())?;
    validate_safe_text("products", "summary", input.summary.as_deref())?;
    let id = next_id("product");
    let status = input.status.unwrap_or_else(|| "active".to_string());
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    connection.execute("insert into products (id, name, status, summary, owner_contact_id, metadata_json) values (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, input.name.trim(), status, input.summary, input.owner_contact_id, metadata_json]).map_err(|e| map_repository_error("products", e))?;
    let record = read_product(connection, &id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "products",
        key: id.clone(),
    })?;
    write_phase6_event(
        connection,
        "product.created",
        "products",
        "product",
        &id,
        &format!("Created product: {}", record.name),
    )?;
    Ok(record)
}

pub(crate) fn list_products(connection: &Connection) -> RepoResult<Vec<ProductRecord>> {
    let mut statement = connection.prepare("select id, name, status, summary, owner_contact_id, metadata_json from products where status != 'archived' order by updated_at desc, name asc")
        .map_err(|e| map_repository_error("products", e))?;
    let rows = statement
        .query_map([], product_from_row)
        .map_err(|e| map_repository_error("products", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("products", e))?;
    Ok(rows)
}

fn read_product(connection: &Connection, id: &str) -> RepoResult<Option<ProductRecord>> {
    connection.query_row("select id, name, status, summary, owner_contact_id, metadata_json from products where id=?1", params![id], product_from_row)
        .optional().map_err(|e| map_repository_error("products", e))
}

fn product_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProductRecord> {
    Ok(ProductRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        status: row.get(2)?,
        summary: row.get(3)?,
        owner_contact_id: row.get(4)?,
        metadata_json: row.get(5)?,
    })
}

pub(crate) fn link_product_entity(
    connection: &Connection,
    input: ProductLinkInput,
) -> RepoResult<EntityLinkRecord> {
    if read_product(connection, &input.product_id)?.is_none() {
        return Err(RepositoryError::NotFound {
            entity: "products",
            key: input.product_id,
        });
    }
    validate_safe_text("entity_links", "target_type", Some(&input.target_type))?;
    validate_safe_text("entity_links", "target_id", Some(&input.target_id))?;
    let id = next_id("product-link");
    let relation_type = input.relation_type.unwrap_or_else(|| "related".to_string());
    validate_safe_text("entity_links", "relation_type", Some(&relation_type))?;
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    let link = create_entity_link(
        connection,
        EntityLinkCreateRequest {
            id: &id,
            source_type: "product",
            source_id: &input.product_id,
            target_type: &input.target_type,
            target_id: &input.target_id,
            relation_type: &relation_type,
            created_by_actor_type: "system",
            metadata_json: &metadata_json,
        },
    )?;
    write_phase6_event(
        connection,
        "product.linked",
        "products",
        "entity_link",
        &id,
        &format!(
            "Linked product {} to {}",
            input.product_id, input.target_type
        ),
    )?;
    Ok(link)
}

pub(crate) fn list_product_links(connection: &Connection) -> RepoResult<Vec<EntityLinkRecord>> {
    let mut statement = connection.prepare("select id, source_type, source_id, target_type, target_id, relation_type, created_by_actor_type, metadata_json from entity_links where source_type='product' order by created_at desc")
        .map_err(|e| map_repository_error("entity_links", e))?;
    let rows = statement
        .query_map([], |row| {
            Ok(EntityLinkRecord {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_id: row.get(2)?,
                target_type: row.get(3)?,
                target_id: row.get(4)?,
                relation_type: row.get(5)?,
                created_by_actor_type: row.get(6)?,
                metadata_json: row.get(7)?,
            })
        })
        .map_err(|e| map_repository_error("entity_links", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("entity_links", e))?;
    Ok(rows)
}

pub(crate) fn create_calendar_event(
    connection: &Connection,
    input: CalendarEventInput,
) -> RepoResult<CalendarRefRecord> {
    ensure_non_empty("calendar_refs", "title", &input.title)?;
    ensure_non_empty("calendar_refs", "starts_at", &input.starts_at)?;
    ensure_non_empty("calendar_refs", "ends_at", &input.ends_at)?;
    validate_safe_text("calendar_refs", "title", Some(&input.title))?;
    validate_safe_text("calendar_refs", "starts_at", Some(&input.starts_at))?;
    validate_safe_text("calendar_refs", "ends_at", Some(&input.ends_at))?;
    validate_safe_text("calendar_refs", "location", input.location.as_deref())?;
    validate_safe_text("calendar_refs", "notes", input.notes.as_deref())?;
    let confirmation_id = require_approved_confirmation(
        connection,
        input.confirmation_id.as_deref(),
        "create_calendar_event",
        "calendar_refs",
    )?;
    let id = next_id("calendar");
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    connection.execute("insert into calendar_refs (id, title, starts_at, ends_at, location, notes, state, confirmation_id, metadata_json) values (?1, ?2, ?3, ?4, ?5, ?6, 'created', ?7, ?8)",
        params![id, input.title.trim(), input.starts_at, input.ends_at, input.location, input.notes, confirmation_id, metadata_json]).map_err(|e| map_repository_error("calendar_refs", e))?;
    let record =
        read_calendar_event(connection, &id)?.ok_or_else(|| RepositoryError::NotFound {
            entity: "calendar_refs",
            key: id.clone(),
        })?;
    write_phase6_event(
        connection,
        "calendar.created",
        "calendar",
        "calendar_event",
        &id,
        &format!("Created calendar event: {}", record.title),
    )?;
    Ok(record)
}

pub(crate) fn update_calendar_event(
    connection: &Connection,
    event_id: &str,
    input: CalendarEventInput,
) -> RepoResult<CalendarRefRecord> {
    ensure_non_empty("calendar_refs", "title", &input.title)?;
    ensure_non_empty("calendar_refs", "starts_at", &input.starts_at)?;
    ensure_non_empty("calendar_refs", "ends_at", &input.ends_at)?;
    validate_safe_text("calendar_refs", "title", Some(&input.title))?;
    validate_safe_text("calendar_refs", "starts_at", Some(&input.starts_at))?;
    validate_safe_text("calendar_refs", "ends_at", Some(&input.ends_at))?;
    validate_safe_text("calendar_refs", "location", input.location.as_deref())?;
    validate_safe_text("calendar_refs", "notes", input.notes.as_deref())?;
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("metadata_json", &metadata_json)?;
    let confirmation_id = require_approved_confirmation(
        connection,
        input.confirmation_id.as_deref(),
        "edit_delete_calendar_event",
        "calendar_refs",
    )?;
    let changed = connection.execute("update calendar_refs set title=?2, starts_at=?3, ends_at=?4, location=?5, notes=?6, state='updated', confirmation_id=?7, metadata_json=?8, updated_at=current_timestamp where id=?1 and state != 'deleted'",
        params![event_id, input.title.trim(), input.starts_at, input.ends_at, input.location, input.notes, confirmation_id, metadata_json]).map_err(|e| map_repository_error("calendar_refs", e))?;
    if changed != 1 {
        return Err(reject_no_mutation("calendar_refs", event_id, "update"));
    }
    let record =
        read_calendar_event(connection, event_id)?.ok_or_else(|| RepositoryError::NotFound {
            entity: "calendar_refs",
            key: event_id.to_string(),
        })?;
    write_phase6_event(
        connection,
        "calendar.updated",
        "calendar",
        "calendar_event",
        event_id,
        &format!("Updated calendar event: {}", record.title),
    )?;
    Ok(record)
}

pub(crate) fn delete_calendar_event(
    connection: &Connection,
    event_id: &str,
    confirmation_id: Option<&str>,
) -> RepoResult<CalendarRefRecord> {
    let confirmation_id = require_approved_confirmation(
        connection,
        confirmation_id,
        "edit_delete_calendar_event",
        "calendar_refs",
    )?;
    let changed = connection.execute("update calendar_refs set state='deleted', confirmation_id=?2, updated_at=current_timestamp where id=?1 and state != 'deleted'", params![event_id, confirmation_id]).map_err(|e| map_repository_error("calendar_refs", e))?;
    if changed != 1 {
        return Err(reject_no_mutation("calendar_refs", event_id, "delete"));
    }
    let record =
        read_calendar_event(connection, event_id)?.ok_or_else(|| RepositoryError::NotFound {
            entity: "calendar_refs",
            key: event_id.to_string(),
        })?;
    write_phase6_event(
        connection,
        "calendar.deleted",
        "calendar",
        "calendar_event",
        event_id,
        &format!("Deleted calendar event: {}", record.title),
    )?;
    Ok(record)
}

pub(crate) fn list_calendar_events(connection: &Connection) -> RepoResult<Vec<CalendarRefRecord>> {
    let mut statement = connection.prepare("select id, external_id, title, starts_at, ends_at, location, notes, state, confirmation_id, metadata_json from calendar_refs where state != 'deleted' order by starts_at asc")
        .map_err(|e| map_repository_error("calendar_refs", e))?;
    let rows = statement
        .query_map([], calendar_from_row)
        .map_err(|e| map_repository_error("calendar_refs", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("calendar_refs", e))?;
    Ok(rows)
}

fn read_calendar_event(connection: &Connection, id: &str) -> RepoResult<Option<CalendarRefRecord>> {
    connection.query_row("select id, external_id, title, starts_at, ends_at, location, notes, state, confirmation_id, metadata_json from calendar_refs where id=?1", params![id], calendar_from_row)
        .optional().map_err(|e| map_repository_error("calendar_refs", e))
}

fn calendar_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarRefRecord> {
    Ok(CalendarRefRecord {
        id: row.get(0)?,
        external_id: row.get(1)?,
        title: row.get(2)?,
        starts_at: row.get(3)?,
        ends_at: row.get(4)?,
        location: row.get(5)?,
        notes: row.get(6)?,
        state: row.get(7)?,
        confirmation_id: row.get(8)?,
        metadata_json: row.get(9)?,
    })
}

pub(crate) fn create_email_draft(
    connection: &Connection,
    input: EmailDraftInput,
) -> RepoResult<EmailRefRecord> {
    ensure_non_empty("email_refs", "subject", &input.subject)?;
    validate_safe_text("email_refs", "subject", Some(&input.subject))?;
    validate_safe_text("email_refs", "thread_id", input.thread_id.as_deref())?;
    validate_safe_text("email_refs", "snippet", input.snippet.as_deref())?;
    let id = next_id("email");
    let recipients_json = json_or_default(input.recipients_json, "[]");
    let metadata_json = json_or_default(input.metadata_json, "{}");
    validate_safe_json("recipients_json", &recipients_json)?;
    validate_safe_json("metadata_json", &metadata_json)?;
    connection.execute("insert into email_refs (id, thread_id, subject, recipients_json, snippet, state, metadata_json) values (?1, ?2, ?3, ?4, ?5, 'draft', ?6)",
        params![id, input.thread_id, input.subject.trim(), recipients_json, input.snippet, metadata_json]).map_err(|e| map_repository_error("email_refs", e))?;
    let record = read_email(connection, &id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "email_refs",
        key: id.clone(),
    })?;
    write_phase6_event(
        connection,
        "email.draft_created",
        "inbox",
        "email",
        &id,
        &format!("Created email draft: {}", record.subject),
    )?;
    Ok(record)
}

pub(crate) fn send_email_draft(
    connection: &Connection,
    email_id: &str,
    input: EmailSendInput,
) -> RepoResult<EmailRefRecord> {
    let confirmation_id = require_approved_confirmation(
        connection,
        input.confirmation_id.as_deref(),
        "send_email",
        "email_refs",
    )?;
    let changed = connection.execute("update email_refs set state='sent', confirmation_id=?2, updated_at=current_timestamp where id=?1 and state='draft'", params![email_id, confirmation_id]).map_err(|e| map_repository_error("email_refs", e))?;
    if changed != 1 {
        return Err(reject_no_mutation("email_refs", email_id, "send"));
    }
    let record = read_email(connection, email_id)?.ok_or_else(|| RepositoryError::NotFound {
        entity: "email_refs",
        key: email_id.to_string(),
    })?;
    write_phase6_event(
        connection,
        "email.sent",
        "inbox",
        "email",
        email_id,
        &format!("Marked email draft sent: {}", record.subject),
    )?;
    Ok(record)
}

pub(crate) fn list_emails(
    connection: &Connection,
    query: Option<String>,
) -> RepoResult<Vec<EmailRefRecord>> {
    if let Some(query) = query.filter(|q| !q.trim().is_empty()) {
        let pattern = format!("%{}%", query.trim());
        let mut statement = connection.prepare("select id, external_id, thread_id, subject, sender, recipients_json, snippet, state, confirmation_id, metadata_json from email_refs where subject like ?1 or snippet like ?1 order by updated_at desc limit 100")
            .map_err(|e| map_repository_error("email_refs", e))?;
        let rows = statement
            .query_map(params![pattern], email_from_row)
            .map_err(|e| map_repository_error("email_refs", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| map_repository_error("email_refs", e))?;
        return Ok(rows);
    }
    let mut statement = connection.prepare("select id, external_id, thread_id, subject, sender, recipients_json, snippet, state, confirmation_id, metadata_json from email_refs order by updated_at desc limit 100")
        .map_err(|e| map_repository_error("email_refs", e))?;
    let rows = statement
        .query_map([], email_from_row)
        .map_err(|e| map_repository_error("email_refs", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| map_repository_error("email_refs", e))?;
    Ok(rows)
}

fn read_email(connection: &Connection, id: &str) -> RepoResult<Option<EmailRefRecord>> {
    connection.query_row("select id, external_id, thread_id, subject, sender, recipients_json, snippet, state, confirmation_id, metadata_json from email_refs where id=?1", params![id], email_from_row)
        .optional().map_err(|e| map_repository_error("email_refs", e))
}

fn email_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EmailRefRecord> {
    Ok(EmailRefRecord {
        id: row.get(0)?,
        external_id: row.get(1)?,
        thread_id: row.get(2)?,
        subject: row.get(3)?,
        sender: row.get(4)?,
        recipients_json: row.get(5)?,
        snippet: row.get(6)?,
        state: row.get(7)?,
        confirmation_id: row.get(8)?,
        metadata_json: row.get(9)?,
    })
}

pub(crate) fn list_phase6_inbox(connection: &Connection) -> RepoResult<Vec<InboxAggregateRecord>> {
    let mut items = Vec::new();
    for follow_up in list_follow_ups(connection)? {
        items.push(InboxAggregateRecord {
            id: follow_up.id,
            item_type: "follow_up".to_string(),
            title: follow_up.subject,
            detail: follow_up
                .due_at
                .unwrap_or_else(|| "No due date".to_string()),
            state: follow_up.state,
            priority: follow_up.priority,
            route: Some("business".to_string()),
        });
    }
    for email in list_emails(connection, None)?
        .into_iter()
        .filter(|email| email.state == "draft" || email.state == "send_blocked")
    {
        items.push(InboxAggregateRecord {
            id: email.id,
            item_type: "email".to_string(),
            title: email.subject,
            detail: "Draft requires review before sending".to_string(),
            state: email.state,
            priority: "high".to_string(),
            route: Some("inbox".to_string()),
        });
    }
    for event in list_calendar_events(connection)? {
        items.push(InboxAggregateRecord {
            id: event.id,
            item_type: "calendar".to_string(),
            title: event.title,
            detail: format!("{} → {}", event.starts_at, event.ends_at),
            state: event.state,
            priority: "normal".to_string(),
            route: Some("calendar".to_string()),
        });
    }
    Ok(items)
}
