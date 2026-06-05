use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BrowserTabRecord {
    pub id: String,
    pub workspace_key: String,
    pub profile_key: String,
    pub url: String,
    pub title: String,
    pub http_status: Option<i64>,
    pub state: String,
    pub opened_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub manual_note: String,
    pub metadata_json: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BrowserOpenTabRequest {
    pub workspace_key: Option<String>,
    pub profile_key: Option<String>,
    pub url: String,
    pub title: Option<String>,
    pub manual_note: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BrowserListRequest {
    pub workspace_key: Option<String>,
    pub profile_key: Option<String>,
    pub limit: Option<i64>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BrowserUpdateTabRequest {
    pub id: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub state: Option<String>,
    pub manual_note: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BrowserCaptureRecord {
    pub id: String,
    pub tab_id: Option<String>,
    pub workspace_key: String,
    pub profile_key: String,
    pub url: String,
    pub title: String,
    pub captured_at: String,
    pub screenshot_path: Option<String>,
    pub screenshot_supported: bool,
    pub capture_mode: String,
    pub http_status: Option<i64>,
    pub manual_note: String,
    pub metadata_json: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BrowserCreateCaptureRequest {
    pub tab_id: Option<String>,
    pub workspace_key: Option<String>,
    pub profile_key: Option<String>,
    pub url: String,
    pub title: Option<String>,
    pub http_status: Option<i64>,
    pub manual_note: Option<String>,
    pub metadata_json: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BrowserAttachCaptureRequest {
    pub capture_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BrowserCaptureLinkRecord {
    pub capture_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: String,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WidgetConfigRecord {
    pub workspace_key: String,
    pub profile_key: String,
    pub widget_key: String,
    pub visible: bool,
    pub position: i64,
    pub size: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WidgetConfigUpdateRequest {
    pub workspace_key: String,
    pub profile_key: Option<String>,
    pub widget_key: String,
    pub visible: bool,
    pub position: i64,
    pub size: String,
}

const ALLOWED_WIDGETS: &[&str] = &[
    "today_tasks",
    "active_runs",
    "blockers",
    "completions",
    "browser_captures",
    "launch_gate_evidence",
    "content_queue",
];
const ALLOWED_SIZES: &[&str] = &["small", "medium", "large"];
const ALLOWED_CAPTURE_ENTITIES: &[&str] =
    &["launch_gate", "task", "note", "product", "content_piece"];

fn id(prefix: &str) -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{prefix}_{ms}")
}
fn workspace(v: &Option<String>) -> String {
    v.clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "browser".into())
}
fn profile(v: &Option<String>) -> String {
    v.clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "default".into())
}

pub(crate) fn redact_secret_like_text(input: &str) -> String {
    let mut out = String::new();
    for (i, part) in input.split('&').enumerate() {
        if i > 0 {
            out.push('&')
        }
        let key = part.split('=').next().unwrap_or("").to_ascii_lowercase();
        if [
            "token",
            "access_token",
            "refresh_token",
            "auth",
            "authorization",
            "password",
            "secret",
            "api_key",
            "apikey",
            "session",
            "cookie",
        ]
        .iter()
        .any(|k| key.contains(k))
        {
            out.push_str(key.as_str());
            out.push_str("=[REDACTED]");
        } else {
            out.push_str(part);
        }
    }
    out
}
fn sanitize_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("browser workspace accepts http(s) work URLs only".into());
    }
    if let Some((base, q)) = trimmed.split_once('?') {
        Ok(format!("{}?{}", base, redact_secret_like_text(q)))
    } else {
        Ok(trimmed.to_string())
    }
}
fn sanitize_text(v: Option<String>) -> String {
    redact_secret_like_text(&crate::redact_secrets(&v.unwrap_or_default()).text)
}
fn sanitize_meta(v: Option<String>) -> String {
    crate::redact_metadata_json(&v.unwrap_or_else(|| "{}".into()))
}
fn safe_link_id(
    capture_id: &str,
    entity_type: &str,
    entity_id: &str,
    relation_type: &str,
) -> String {
    let raw = format!("browser_capture_{capture_id}_{entity_type}_{entity_id}_{relation_type}");
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
fn record_event(
    c: &Connection,
    kind: &str,
    summary: &str,
    entity_type: &str,
    entity_id: &str,
) -> Result<(), String> {
    let eid = id("evt");
    c.execute("insert into events(id,type,actor_type,workspace_key,summary,severity,source,metadata_json) values (?1,?2,'system','browser',?3,'info','phase7','{}')",params![eid,kind,summary]).map_err(|e|e.to_string())?;
    c.execute("insert into event_targets(event_id,entity_type,entity_id,relation_type) values (?1,?2,?3,'subject')",params![eid,entity_type,entity_id]).map_err(|e|e.to_string())?;
    Ok(())
}
fn row_tab(r: &rusqlite::Row) -> rusqlite::Result<BrowserTabRecord> {
    Ok(BrowserTabRecord {
        id: r.get(0)?,
        workspace_key: r.get(1)?,
        profile_key: r.get(2)?,
        url: r.get(3)?,
        title: r.get(4)?,
        http_status: r.get(5)?,
        state: r.get(6)?,
        opened_at: r.get(7)?,
        updated_at: r.get(8)?,
        closed_at: r.get(9)?,
        manual_note: r.get(10)?,
        metadata_json: r.get(11)?,
    })
}
fn row_cap(r: &rusqlite::Row) -> rusqlite::Result<BrowserCaptureRecord> {
    let supported: i64 = r.get(7)?;
    Ok(BrowserCaptureRecord {
        id: r.get(0)?,
        tab_id: r.get(1)?,
        workspace_key: r.get(2)?,
        profile_key: r.get(3)?,
        url: r.get(4)?,
        title: r.get(5)?,
        captured_at: r.get(6)?,
        screenshot_path: r.get(8)?,
        screenshot_supported: supported != 0,
        capture_mode: r.get(9)?,
        http_status: r.get(10)?,
        manual_note: r.get(11)?,
        metadata_json: r.get(12)?,
    })
}

pub(crate) fn browser_open_tab(
    c: &Connection,
    req: BrowserOpenTabRequest,
) -> Result<BrowserTabRecord, String> {
    let url = sanitize_url(&req.url)?;
    let rec_id = id("tab");
    c.execute("insert into browser_tabs(id,workspace_key,profile_key,url,title,http_status,state,manual_note,metadata_json) values (?1,?2,?3,?4,?5,null,'open',?6,'{}')",params![rec_id,workspace(&req.workspace_key),profile(&req.profile_key),url,sanitize_text(req.title),sanitize_text(req.manual_note)]).map_err(|e|e.to_string())?;
    record_event(
        c,
        "browser.opened",
        "Browser work URL opened",
        "browser_tab",
        &rec_id,
    )?;
    browser_read_tab(c, &rec_id)
}
pub(crate) fn browser_read_tab(c: &Connection, rid: &str) -> Result<BrowserTabRecord, String> {
    c.query_row("select id,workspace_key,profile_key,url,title,http_status,state,opened_at,updated_at,closed_at,manual_note,metadata_json from browser_tabs where id=?1",params![rid],row_tab).map_err(|e|e.to_string())
}
pub(crate) fn browser_list_tabs(
    c: &Connection,
    req: BrowserListRequest,
) -> Result<Vec<BrowserTabRecord>, String> {
    let mut stmt=c.prepare("select id,workspace_key,profile_key,url,title,http_status,state,opened_at,updated_at,closed_at,manual_note,metadata_json from browser_tabs where workspace_key=?1 and profile_key=?2 order by updated_at desc limit ?3").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map(
            params![
                workspace(&req.workspace_key),
                profile(&req.profile_key),
                req.limit.unwrap_or(50).clamp(1, 200)
            ],
            row_tab,
        )
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub(crate) fn browser_update_tab(
    c: &Connection,
    req: BrowserUpdateTabRequest,
) -> Result<BrowserTabRecord, String> {
    let mut tab = browser_read_tab(c, &req.id)?;
    if let Some(u) = req.url {
        tab.url = sanitize_url(&u)?
    }
    if let Some(t) = req.title {
        tab.title = sanitize_text(Some(t))
    }
    if let Some(s) = req.state {
        if !["open", "saved", "closed", "blocked", "unsupported"].contains(&s.as_str()) {
            return Err("unsupported browser tab state".into());
        }
        tab.state = s
    }
    if let Some(n) = req.manual_note {
        tab.manual_note = sanitize_text(Some(n))
    }
    c.execute("update browser_tabs set url=?2,title=?3,state=?4,manual_note=?5,updated_at=current_timestamp,closed_at=case when ?4='closed' then current_timestamp else closed_at end where id=?1",params![tab.id,tab.url,tab.title,tab.state,tab.manual_note]).map_err(|e|e.to_string())?;
    record_event(
        c,
        "browser.updated",
        "Browser tab or saved page updated",
        "browser_tab",
        &tab.id,
    )?;
    browser_read_tab(c, &tab.id)
}
pub(crate) fn browser_create_capture(
    c: &Connection,
    req: BrowserCreateCaptureRequest,
) -> Result<BrowserCaptureRecord, String> {
    let url = sanitize_url(&req.url)?;
    let rec_id = id("cap");
    c.execute("insert into browser_captures(id,tab_id,workspace_key,profile_key,url,title,screenshot_path,screenshot_supported,capture_mode,http_status,manual_note,metadata_json) values (?1,?2,?3,?4,?5,?6,null,0,'metadata_fallback',?7,?8,?9)",params![rec_id,req.tab_id,workspace(&req.workspace_key),profile(&req.profile_key),url,sanitize_text(req.title),req.http_status,sanitize_text(req.manual_note),sanitize_meta(req.metadata_json)]).map_err(|e|e.to_string())?;
    record_event(
        c,
        "browser.capture_created",
        "Browser metadata fallback capture created",
        "browser_capture",
        &rec_id,
    )?;
    browser_read_capture(c, &rec_id)
}
pub(crate) fn browser_read_capture(
    c: &Connection,
    rid: &str,
) -> Result<BrowserCaptureRecord, String> {
    c.query_row("select id,tab_id,workspace_key,profile_key,url,title,captured_at,screenshot_supported,screenshot_path,capture_mode,http_status,manual_note,metadata_json from browser_captures where id=?1",params![rid],row_cap).map_err(|e|e.to_string())
}
pub(crate) fn browser_list_captures(
    c: &Connection,
    req: BrowserListRequest,
) -> Result<Vec<BrowserCaptureRecord>, String> {
    let mut stmt=c.prepare("select id,tab_id,workspace_key,profile_key,url,title,captured_at,screenshot_supported,screenshot_path,capture_mode,http_status,manual_note,metadata_json from browser_captures where workspace_key=?1 and profile_key=?2 order by captured_at desc limit ?3").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map(
            params![
                workspace(&req.workspace_key),
                profile(&req.profile_key),
                req.limit.unwrap_or(50).clamp(1, 200)
            ],
            row_cap,
        )
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub(crate) fn browser_attach_capture(
    c: &Connection,
    req: BrowserAttachCaptureRequest,
) -> Result<BrowserCaptureLinkRecord, String> {
    if !ALLOWED_CAPTURE_ENTITIES.contains(&req.entity_type.as_str()) {
        return Err("unsupported browser capture attachment target".into());
    }
    let cap = browser_read_capture(c, &req.capture_id)?;
    if cap.url.trim().is_empty() || cap.title.trim().is_empty() {
        return Err("browser capture evidence requires URL and title before attachment".into());
    }
    let rel = req.relation_type.unwrap_or_else(|| "evidence".into());
    c.execute("insert or ignore into browser_capture_links(capture_id,entity_type,entity_id,relation_type) values (?1,?2,?3,?4)",params![req.capture_id,req.entity_type,req.entity_id,rel]).map_err(|e|e.to_string())?;
    let link_id = safe_link_id(&req.capture_id, &req.entity_type, &req.entity_id, &rel);
    crate::create_entity_link(
        c,
        crate::EntityLinkCreateRequest {
            id: &link_id,
            source_type: "browser_capture",
            source_id: &req.capture_id,
            target_type: &req.entity_type,
            target_id: &req.entity_id,
            relation_type: &rel,
            created_by_actor_type: "system",
            metadata_json: "{}",
        },
    )
    .map_err(|e| format!("browser capture entity-link failed: {e:?}"))?;
    record_event(
        c,
        "browser.capture_attached",
        "Browser capture attached as evidence",
        "browser_capture",
        &req.capture_id,
    )?;
    Ok(BrowserCaptureLinkRecord {
        capture_id: req.capture_id,
        entity_type: req.entity_type,
        entity_id: req.entity_id,
        relation_type: rel,
        created_at: "current_timestamp".into(),
    })
}
pub(crate) fn browser_http_status(_url: String) -> Result<Option<i64>, String> {
    Ok(None)
}
fn default_widgets(workspace_key: &str, profile_key: &str) -> Vec<WidgetConfigRecord> {
    ALLOWED_WIDGETS
        .iter()
        .enumerate()
        .map(|(i, k)| WidgetConfigRecord {
            workspace_key: workspace_key.into(),
            profile_key: profile_key.into(),
            widget_key: k.to_string(),
            visible: true,
            position: i as i64,
            size: "medium".into(),
            updated_at: "default".into(),
        })
        .collect()
}
pub(crate) fn widget_read_configs(
    c: &Connection,
    workspace_key: String,
    profile_key: Option<String>,
) -> Result<Vec<WidgetConfigRecord>, String> {
    let p = profile(&profile_key);
    let count: i64 = c
        .query_row(
            "select count(*) from widget_configs where workspace_key=?1 and profile_key=?2",
            params![workspace_key, p],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if count == 0 {
        return Ok(default_widgets(&workspace_key, &p));
    }
    let mut stmt=c.prepare("select workspace_key,profile_key,widget_key,visible,position,size,updated_at from widget_configs where workspace_key=?1 and profile_key=?2 order by position asc").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map(params![workspace_key, p], |r| {
            let visible: i64 = r.get(3)?;
            Ok(WidgetConfigRecord {
                workspace_key: r.get(0)?,
                profile_key: r.get(1)?,
                widget_key: r.get(2)?,
                visible: visible != 0,
                position: r.get(4)?,
                size: r.get(5)?,
                updated_at: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub(crate) fn widget_update_config(
    c: &Connection,
    req: WidgetConfigUpdateRequest,
) -> Result<WidgetConfigRecord, String> {
    if !ALLOWED_WIDGETS.contains(&req.widget_key.as_str()) {
        return Err("unsupported widget key".into());
    }
    if !ALLOWED_SIZES.contains(&req.size.as_str()) {
        return Err("unsupported widget size".into());
    }
    let p = profile(&req.profile_key);
    c.execute("insert into widget_configs(workspace_key,profile_key,widget_key,visible,position,size,updated_at) values (?1,?2,?3,?4,?5,?6,current_timestamp) on conflict(workspace_key,profile_key,widget_key) do update set visible=excluded.visible,position=excluded.position,size=excluded.size,updated_at=current_timestamp",params![req.workspace_key,p,req.widget_key,if req.visible{1}else{0},req.position,req.size]).map_err(|e|e.to_string())?;
    record_event(
        c,
        "widget.config_changed",
        "Widget configuration changed",
        "widget_config",
        &req.widget_key,
    )?;
    Ok(widget_read_configs(c, req.workspace_key, Some(p))?
        .into_iter()
        .find(|w| w.widget_key == req.widget_key)
        .unwrap())
}
pub(crate) fn widget_reset_configs(
    c: &Connection,
    workspace_key: String,
    profile_key: Option<String>,
) -> Result<Vec<WidgetConfigRecord>, String> {
    let p = profile(&profile_key);
    c.execute(
        "delete from widget_configs where workspace_key=?1 and profile_key=?2",
        params![workspace_key, p],
    )
    .map_err(|e| e.to_string())?;
    for w in default_widgets(&workspace_key, &p) {
        c.execute("insert into widget_configs(workspace_key,profile_key,widget_key,visible,position,size) values (?1,?2,?3,?4,?5,?6)",params![w.workspace_key,w.profile_key,w.widget_key,if w.visible{1}else{0},w.position,w.size]).map_err(|e|e.to_string())?;
    }
    record_event(
        c,
        "widget.config_reset",
        "Widget configuration reset",
        "widget_config",
        &workspace_key,
    )?;
    widget_read_configs(c, workspace_key, Some(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_secret_query() {
        assert_eq!(
            redact_secret_like_text("a=1&token=abc&password=x"),
            "a=1&token=[REDACTED]&password=[REDACTED]"
        );
    }
}
