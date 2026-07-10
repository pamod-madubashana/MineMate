#![allow(dead_code)]

use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

const NIM_BASE_URL: &str = "https://integrate.api.nvidia.com/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub index: usize,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    pub function: Option<FunctionCallDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

pub struct NimClient {
    client: Client,
    api_key: String,
    model: String,
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

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 1024,
        });

        if let Some(tools) = tools {
            body["tools"] = serde_json::to_value(tools)?;
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", NIM_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("NIM API returned {}: {}", status, error_text);
            return Err(format!("NIM API error ({}): {}", status, error_text).into());
        }

        let completion: ChatCompletion = response.json().await?;
        Ok(completion)
    }

    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[serde_json::Value]>,
        tx: mpsc::Sender<String>,
    ) -> Result<Option<Vec<ToolCall>>, Box<dyn std::error::Error>> {
        self.rate_limit().await;

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 1024,
            "stream": true,
        });

        if let Some(tools) = tools {
            body["tools"] = serde_json::to_value(tools)?;
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", NIM_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("NIM API returned {}: {}", status, error_text);
            return Err(format!("NIM API error ({}): {}", status, error_text).into());
        }

        let mut buffer = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();

        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(idx) = buffer.find("\n\n") {
                let event = buffer[..idx].to_string();
                buffer.drain(..idx + 2);

                for line in event.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }

                        if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    let _ = tx.send(content.clone()).await;
                                }

                                if let Some(new_tool_calls) = &choice.delta.tool_calls {
                                    for tc in new_tool_calls {
                                        if let Some(func) = &tc.function {
                                            if let Some(name) = &func.name {
                                                tool_calls.push(ToolCall {
                                                    id: tc.id.clone().unwrap_or_default(),
                                                    tool_type: "function".to_string(),
                                                    function: FunctionCall {
                                                        name: name.clone(),
                                                        arguments: func
                                                            .arguments
                                                            .clone()
                                                            .unwrap_or_default(),
                                                    },
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if tool_calls.is_empty() {
            Ok(None)
        } else {
            Ok(Some(tool_calls))
        }
    }

    async fn rate_limit(&self) {
        let wait_time = {
            let mut count = self.request_count.write();
            let mut last_time = self.last_request_time.write();

            let now = std::time::Instant::now();
            let elapsed = now.duration_since(*last_time);

            if elapsed.as_secs() < 60 && *count >= 25 {
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
