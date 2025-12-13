//! AI provider-specific implementations

/// OpenAI API models and constants
pub mod openai {
    pub const API_BASE: &str = "https://api.openai.com/v1";
    pub const CHAT_COMPLETIONS_ENDPOINT: &str = "/chat/completions";

    /// Common OpenAI models
    pub const GPT_4: &str = "gpt-4";
    pub const GPT_4_TURBO: &str = "gpt-4-turbo-preview";
    pub const GPT_35_TURBO: &str = "gpt-3.5-turbo";
}

/// Anthropic API models and constants
pub mod anthropic {
    pub const API_BASE: &str = "https://api.anthropic.com/v1";
    pub const MESSAGES_ENDPOINT: &str = "/messages";

    /// Common Anthropic models
    pub const CLAUDE_3_OPUS: &str = "claude-3-opus-20240229";
    pub const CLAUDE_3_SONNET: &str = "claude-3-sonnet-20240229";
    pub const CLAUDE_3_HAIKU: &str = "claude-3-haiku-20240307";
}

// TODO: Add request/response types for each provider
// TODO: Implement provider-specific API clients
