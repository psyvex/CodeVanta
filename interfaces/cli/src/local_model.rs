use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors talking to a local OpenAI-compatible chat completions endpoint
/// (Ollama or llama.cpp's `llama-server`, for example).
#[derive(Debug, Error)]
pub enum LocalModelError {
    #[error("could not reach local model endpoint '{endpoint}': {message}")]
    Request { endpoint: String, message: String },
    #[error(
        "local model endpoint '{endpoint}' returned a response CodeVanta could not parse: {message}"
    )]
    InvalidResponse { endpoint: String, message: String },
    #[error("local model endpoint '{endpoint}' returned no choices")]
    EmptyResponse { endpoint: String },
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

/// A client for a locally running OpenAI-compatible chat completions
/// endpoint. Decoupled from whether a CodeVanta-trained adapter exists:
/// any local model already serving that API works.
pub struct LocalModelClient {
    endpoint: String,
    model: String,
    agent: ureq::Agent,
}

impl LocalModelClient {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
            agent: ureq::Agent::new_with_defaults(),
        }
    }

    /// Sends a system + user message pair and returns the model's reply text.
    pub fn chat(&self, system: &str, user: &str) -> Result<String, LocalModelError> {
        let request = ChatRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system,
                },
                ChatMessage {
                    role: "user",
                    content: user,
                },
            ],
            stream: false,
        };

        let response = self
            .agent
            .post(&self.endpoint)
            .send_json(&request)
            .map_err(|error| LocalModelError::Request {
                endpoint: self.endpoint.clone(),
                message: error.to_string(),
            })?;

        let body: ChatCompletionResponse =
            response
                .into_body()
                .read_json()
                .map_err(|error| LocalModelError::InvalidResponse {
                    endpoint: self.endpoint.clone(),
                    message: error.to_string(),
                })?;

        body.choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| LocalModelError::EmptyResponse {
                endpoint: self.endpoint.clone(),
            })
    }
}
