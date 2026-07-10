#![allow(dead_code)]

use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Ordered fallback chain — tries from first to last until one succeeds.
const FALLBACK_MODELS: &[&str] = &[
    "gemini-3.5-flash",
    "gemini-flash-latest",
    "gemini-3.1-pro-preview",
    "gemini-3-flash-preview",
    "gemini-3.1-flash-lite",
    "gemini-pro-latest",
    "gemini-flash-lite-latest",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_token_count: u32,
    #[serde(default)]
    pub candidates_token_count: u32,
    #[serde(default)]
    pub total_token_count: u32,
}

// --- Google native request types (outgoing) ---

#[derive(Debug, Serialize, Clone)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GeminiContent {
    role: String,
    parts: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize, Clone)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: Option<serde_json::Value>,
}

// --- Google native response types (incoming, camelCase) ---

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<serde_json::Value>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<Usage>,
}

pub struct NimClient {
    client: Client,
    api_key: String,
    model: String,
    working_model: Arc<RwLock<Option<String>>>,
    request_count: Arc<RwLock<u32>>,
    last_request_time: Arc<RwLock<std::time::Instant>>,
}

impl NimClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .no_proxy()
                .build()
                .expect("Failed to build reqwest client with no_proxy"),
            api_key,
            model,
            working_model: Arc::new(RwLock::new(None)),
            request_count: Arc::new(RwLock::new(0)),
            last_request_time: Arc::new(RwLock::new(std::time::Instant::now())),
        }
    }

    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[serde_json::Value]>,
    ) -> Result<ChatCompletion, Box<dyn std::error::Error>> {
        self.rate_limit().await;

        let contents = self.build_contents(messages);
        let gemini_tools = self.build_tools(tools);
        let body = GeminiRequest {
            contents,
            tools: gemini_tools,
        };

        let models = self.build_model_chain();
        let mut last_err = String::new();

        for model_name in &models {
            let url = format!("{}/{}:generateContent", GEMINI_BASE_URL, model_name);

            let response = match self
                .client
                .post(&url)
                .header("X-goog-api-key", &self.api_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    last_err = format!("{}: network error: {}", model_name, e);
                    continue;
                }
            };

            let status = response.status();

            if status == 404 || status == 405 || status == 503 || status == 429 {
                let err_text = response.text().await.unwrap_or_default();
                tracing::warn!("Model {} returned {} — trying next", model_name, status);
                last_err = format!("{} ({}): {}", model_name, status, err_text);
                continue;
            }

            if !status.is_success() {
                let err_text = response.text().await.unwrap_or_default();
                last_err = format!("{} ({}): {}", model_name, status, err_text);
                tracing::warn!("Model {} error — trying next", model_name);
                continue;
            }

            let raw_text = match response.text().await {
                Ok(t) => t,
                Err(e) => {
                    last_err = format!("{}: failed to read body: {}", model_name, e);
                    continue;
                }
            };

            let gemini_resp: GeminiResponse = match serde_json::from_str(&raw_text) {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!("Model {} parse error: {}\nRaw: {}", model_name, e, &raw_text[..raw_text.len().min(500)]);
                    last_err = format!("{}: parse error: {}", model_name, e);
                    continue;
                }
            };

            // Cache working model
            {
                let mut wm = self.working_model.write();
                *wm = Some(model_name.to_string());
            }

            tracing::info!("Using model: {}", model_name);
            return self.parse_response(gemini_resp);
        }

        Err(format!("All models failed. Last error: {}", last_err).into())
    }

    fn build_contents(&self, messages: &[ChatMessage]) -> Vec<GeminiContent> {
        messages
            .iter()
            .map(|m| {
                let role = if m.role == "assistant" {
                    "model"
                } else {
                    &m.role
                };
                GeminiContent {
                    role: role.to_string(),
                    parts: vec![serde_json::json!({"text": m.content})],
                }
            })
            .collect()
    }

    fn build_tools(&self, tools: Option<&[serde_json::Value]>) -> Option<Vec<GeminiTool>> {
        tools.map(|t| {
            vec![GeminiTool {
                function_declarations: t
                    .iter()
                    .filter_map(|tool| {
                        let func = tool.get("function")?;
                        Some(GeminiFunctionDeclaration {
                            name: func.get("name")?.as_str()?.to_string(),
                            description: func.get("description")?.as_str()?.to_string(),
                            parameters: func.get("parameters").cloned(),
                        })
                    })
                    .collect(),
            }]
        })
    }

    fn build_model_chain(&self) -> Vec<String> {
        let mut models = Vec::new();

        if let Some(ref wm) = *self.working_model.read() {
            models.push(wm.clone());
        }

        if !models.iter().any(|m| m == &self.model) {
            models.push(self.model.clone());
        }

        for &fb in FALLBACK_MODELS {
            if !models.iter().any(|m| m == fb) {
                models.push(fb.to_string());
            }
        }

        models
    }

    fn parse_response(
        &self,
        gemini_resp: GeminiResponse,
    ) -> Result<ChatCompletion, Box<dyn std::error::Error>> {
        let candidate = gemini_resp
            .candidates
            .and_then(|c| c.into_iter().next());

        let mut content = None;
        let mut tool_calls = None;

        if let Some(candidate_val) = candidate {
            // Extract functionCalls from candidate (camelCase)
            if let Some(fc_arr) = candidate_val.get("functionCalls").and_then(|v| v.as_array()) {
                if !fc_arr.is_empty() {
                    tool_calls = Some(
                        fc_arr
                            .iter()
                            .enumerate()
                            .filter_map(|(i, fc)| {
                                let name = fc.get("name")?.as_str()?.to_string();
                                let args = fc.get("args").cloned().unwrap_or(serde_json::json!({}));
                                Some(ToolCall {
                                    id: format!("call_{}", i),
                                    name,
                                    arguments: args.to_string(),
                                })
                            })
                            .collect(),
                    );
                }
            }

            // Extract text from content.parts
            if let Some(content_val) = candidate_val.get("content") {
                if let Some(parts) = content_val.get("parts").and_then(|v| v.as_array()) {
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                            content = Some(text.to_string());
                            break;
                        }
                    }
                }
            }
        }

        Ok(ChatCompletion {
            content,
            tool_calls,
            usage: gemini_resp.usage_metadata,
        })
    }

    async fn rate_limit(&self) {
        let wait_time = {
            let mut count = self.request_count.write();
            let mut last_time = self.last_request_time.write();

            let now = std::time::Instant::now();
            let elapsed = now.duration_since(*last_time);

            if elapsed.as_secs() < 60 && *count >= 15 {
                let wait = 60 - elapsed.as_secs();
                tracing::info!("Rate limit reached, waiting {} seconds", wait);
                *count = 0;
                *last_time = std::time::Instant::now();
                Some(wait)
            } else {
                if elapsed.as_secs() >= 60 {
                    *count = 0;
                    *last_time = now;
                }
                *count += 1;
                None
            }
        };

        if let Some(secs) = wait_time {
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
        }
    }
}
