use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_HERMES_BASE_URL: &str = "http://127.0.0.1:8642";
const DEFAULT_HERMES_MODEL: &str = "hermes-agent";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesHealth {
    ok: bool,
    status: String,
    message: String,
    model: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HermesChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesChatResponse {
    content: String,
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<HermesChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Option<Vec<ModelInfo>>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

fn hermes_base_url() -> String {
    std::env::var("ZOID_HERMES_API_BASE_URL")
        .or_else(|_| std::env::var("HERMES_API_BASE_URL"))
        .unwrap_or_else(|_| DEFAULT_HERMES_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn hermes_api_key() -> Option<String> {
    std::env::var("ZOID_HERMES_API_KEY")
        .or_else(|_| std::env::var("API_SERVER_KEY"))
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn hermes_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| format!("Failed to build Hermes HTTP client: {error}"))
}

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn check_hermes_health() -> Result<HermesHealth, String> {
        let base_url = hermes_base_url();
        let client = hermes_client()?;

        let health_url = format!("{base_url}/health");
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => {
                return Ok(HermesHealth {
                    ok: false,
                    status: "offline".to_string(),
                    message: format!("Hermes API server health check returned HTTP {}.", response.status()),
                    model: None,
                });
            }
            Err(_) => {
                return Ok(HermesHealth {
                    ok: false,
                    status: "offline".to_string(),
                    message: "Hermes API server is offline. Enable API_SERVER_ENABLED and restart hermes gateway.".to_string(),
                    model: None,
                });
            }
        }

        let Some(api_key) = hermes_api_key() else {
            return Ok(HermesHealth {
                ok: false,
                status: "unauthorized".to_string(),
                message: "Hermes API server is reachable, but ZOID_HERMES_API_KEY/API_SERVER_KEY is not set for chat requests.".to_string(),
                model: None,
            });
        };

        let response = client
            .get(format!("{base_url}/v1/models"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|error| format!("Failed to check Hermes models endpoint: {error}"))?;

        if response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN {
            return Ok(HermesHealth {
                ok: false,
                status: "unauthorized".to_string(),
                message: "Hermes API key was rejected by /v1/models.".to_string(),
                model: None,
            });
        }

        if !response.status().is_success() {
            return Ok(HermesHealth {
                ok: false,
                status: "error".to_string(),
                message: format!("Hermes /v1/models returned HTTP {}.", response.status()),
                model: None,
            });
        }

        let models = response.json::<ModelsResponse>().await.ok();
        let model = models
            .and_then(|body| body.data)
            .and_then(|mut data| data.drain(..).next())
            .map(|model| model.id);

        Ok(HermesHealth {
            ok: true,
            status: "online".to_string(),
            message: "Hermes API server is online and authorized.".to_string(),
            model,
        })
    }

    #[tauri::command]
    pub async fn send_hermes_message(messages: Vec<HermesChatMessage>) -> Result<HermesChatResponse, String> {
        if messages.is_empty() {
            return Err("Cannot send an empty Hermes conversation.".to_string());
        }

        let base_url = hermes_base_url();
        let api_key = hermes_api_key().ok_or_else(|| {
            "ZOID_HERMES_API_KEY/API_SERVER_KEY is required to send messages to Hermes.".to_string()
        })?;
        let client = hermes_client()?;
        let request = ChatCompletionRequest {
            model: std::env::var("ZOID_HERMES_MODEL").unwrap_or_else(|_| DEFAULT_HERMES_MODEL.to_string()),
            messages,
            stream: false,
        };

        let response = client
            .post(format!("{base_url}/v1/chat/completions"))
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await
            .map_err(|error| format!("Failed to send Hermes message: {error}"))?;

        if response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN {
            return Err("Hermes API key was rejected by /v1/chat/completions.".to_string());
        }

        if !response.status().is_success() {
            return Err(format!(
                "Hermes chat endpoint returned HTTP {}.",
                response.status()
            ));
        }

        let body = response
            .json::<ChatCompletionResponse>()
            .await
            .map_err(|error| format!("Hermes returned an unreadable chat response: {error}"))?;
        let content = body
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .unwrap_or_default();

        Ok(HermesChatResponse {
            content,
            model: body.model,
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::check_hermes_health,
            commands::send_hermes_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zoid 25");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_local_hermes_base_url() {
        std::env::remove_var("ZOID_HERMES_API_BASE_URL");
        std::env::remove_var("HERMES_API_BASE_URL");
        assert_eq!(hermes_base_url(), DEFAULT_HERMES_BASE_URL);
    }

    #[test]
    fn trims_base_url_trailing_slash() {
        std::env::set_var("ZOID_HERMES_API_BASE_URL", "http://127.0.0.1:8642/");
        assert_eq!(hermes_base_url(), DEFAULT_HERMES_BASE_URL);
        std::env::remove_var("ZOID_HERMES_API_BASE_URL");
    }
}
