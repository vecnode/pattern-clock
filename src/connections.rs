use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Error type for LLM connection operations
#[derive(Debug)]
pub enum ConnectionError {
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
    ApiError { status: u16, message: String },
    ModelNotFound(String),
    ConfigurationError(String),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::NetworkError(e) => write!(f, "Network error: {}", e),
            ConnectionError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConnectionError::ApiError { status, message } => {
                write!(f, "API error (status {}): {}", status, message)
            }
            ConnectionError::ModelNotFound(model) => {
                write!(f, "Model not found: {}", model)
            }
            ConnectionError::ConfigurationError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConnectionError {}

impl From<reqwest::Error> for ConnectionError {
    fn from(err: reqwest::Error) -> Self {
        ConnectionError::NetworkError(err)
    }
}

impl From<serde_json::Error> for ConnectionError {
    fn from(err: serde_json::Error) -> Self {
        ConnectionError::ParseError(err)
    }
}

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: Option<u64>,
    pub modified_at: Option<String>,
}

/// Options for message generation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub repeat_penalty: Option<f64>,
}

/// Full response from LLM with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub total_duration: Option<u64>,
}

/// Trait for LLM providers - allows extensibility to other providers
pub trait LlmProvider: Send + Sync {
    /// List all available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ConnectionError>;

    /// Send a message and get a response (simple version)
    async fn send_message(&self, prompt: &str, model: &str) -> Result<String, ConnectionError> {
        let response = self.send_message_with_options(prompt, model, &MessageOptions::default()).await?;
        Ok(response.text)
    }

    /// Send a message with options and get full response
    async fn send_message_with_options(
        &self,
        prompt: &str,
        model: &str,
        options: &MessageOptions,
    ) -> Result<LlmResponse, ConnectionError>;

    /// Get the provider name
    fn provider_name(&self) -> &'static str;
}

/// Configuration for Ollama provider
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout_secs: u64,
    pub default_model: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_secs: 30,
            default_model: None,
        }
    }
}

/// Ollama LLM provider implementation
pub struct OllamaProvider {
    client: reqwest::Client,
    config: OllamaConfig,
}

impl OllamaProvider {
    /// Create a new Ollama provider with default configuration
    pub fn new(base_url: Option<String>) -> Result<Self, ConnectionError> {
        let config = OllamaConfig {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            ..Default::default()
        };
        Self::new_with_config(config)
    }

    /// Create a new Ollama provider with custom configuration
    pub fn new_with_config(config: OllamaConfig) -> Result<Self, ConnectionError> {
        let mut builder = reqwest::Client::builder();
        
        // timeout() is not available for wasm32 targets
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder.timeout(Duration::from_secs(config.timeout_secs));
        }
        
        let client = builder
            .build()
            .map_err(|e| ConnectionError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Create a new Ollama provider with a custom HTTP client
    pub fn new_with_client(client: reqwest::Client, base_url: String) -> Self {
        Self {
            client,
            config: OllamaConfig {
                base_url,
                ..Default::default()
            },
        }
    }

    /// Get the default model if configured
    pub fn default_model(&self) -> Option<&str> {
        self.config.default_model.as_deref()
    }
}

// Ollama API request/response structures
#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaGenerateOptions>,
}

#[derive(Serialize)]
struct OllamaGenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_penalty: Option<f64>,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    model: String,
    response: String,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    modified_at: Option<String>,
}

impl LlmProvider for OllamaProvider {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ConnectionError> {
        let url = format!("{}/api/tags", self.config.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ConnectionError::ApiError {
                status: response.status().as_u16(),
                message: format!("Failed to list models: {}", response.status()),
            });
        }

        let tags_response: OllamaTagsResponse = response.json().await?;
        Ok(tags_response
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size: m.size,
                modified_at: m.modified_at,
            })
            .collect())
    }

    async fn send_message_with_options(
        &self,
        prompt: &str,
        model: &str,
        options: &MessageOptions,
    ) -> Result<LlmResponse, ConnectionError> {
        let url = format!("{}/api/generate", self.config.base_url);

        // Build options for Ollama
        let ollama_options = if options.temperature.is_some()
            || options.max_tokens.is_some()
            || options.top_p.is_some()
            || options.top_k.is_some()
            || options.repeat_penalty.is_some()
        {
            Some(OllamaGenerateOptions {
                temperature: options.temperature,
                num_predict: options.max_tokens,
                top_p: options.top_p,
                top_k: options.top_k,
                repeat_penalty: options.repeat_penalty,
            })
        } else {
            None
        };

        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: ollama_options,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ConnectionError::ApiError {
                status,
                message: error_text,
            });
        }

        let generate_response: OllamaGenerateResponse = response.json().await?;

        Ok(LlmResponse {
            text: generate_response.response,
            model: generate_response.model,
            tokens_used: generate_response.eval_count,
            total_duration: generate_response.total_duration,
        })
    }

    fn provider_name(&self) -> &'static str {
        "Ollama"
    }
}

/// Helper function to create a default Ollama provider
pub fn create_ollama_provider() -> Result<OllamaProvider, ConnectionError> {
    OllamaProvider::new(None)
}
