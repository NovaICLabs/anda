//! Core data models and traits for the AI agent system.
//!
//! This module defines the fundamental data structures and interfaces used throughout the AI agent system.
//! It includes:
//! - Core message and conversation structures ([`AgentOutput`], [`Message`], [`ToolCall`]).
//! - Function definition and tooling support ([`FunctionDefinition`]).
//! - Knowledge and document handling ([`Document`], [`Documents`]).
//! - Completion request and response structures ([`CompletionRequest`], [`Embedding`]).
//! - Core AI capabilities traits ([`CompletionFeatures`], [`EmbeddingFeatures`]).

use candid::Principal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod completion;
mod embedding;
mod knowledge;
mod resource;
mod thread;

pub use completion::*;
pub use embedding::*;
pub use knowledge::*;
pub use resource::*;
pub use thread::*;

pub const ANONYMOUS: Principal = Principal::anonymous();

/// Represents a request to an agent for processing.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AgentInput {
    /// agent name, use default agent if empty.
    pub name: String,

    /// agent prompt or message.
    pub prompt: String,

    /// The resources to process by the agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Resource>>,

    /// The metadata for the agent request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}

impl AgentInput {
    /// Creates a new agent input with the given name and prompt.
    pub fn new(name: String, prompt: String) -> Self {
        Self {
            name,
            prompt,
            resources: None,
            meta: None,
        }
    }
}

/// Represents the output of an agent execution.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AgentOutput {
    /// The output content from the agent, may be empty.
    pub content: String,

    /// The usage statistics for the agent execution.
    pub usage: Usage,

    /// The unique identifier for the thread.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<ThreadId>,

    /// Indicates failure reason if present, None means successful execution.
    /// Should be None when finish_reason is "stop" or "tool_calls".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_reason: Option<String>,

    /// Tool calls returned by the LLM function calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// full_history will be included in `ctx.completion` response,
    /// but not be included in the engine response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_history: Option<Vec<Value>>,

    /// The resources generated by the agent execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Resource>>,
}

/// Represents a request to a tool for processing.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ToolInput<T> {
    /// tool name.
    pub name: String,

    /// arguments in JSON format.
    pub args: T,

    /// The resources to process by the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Resource>>,

    /// The metadata for the tool request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}

impl<T> ToolInput<T> {
    /// Creates a new tool input with the given name and arguments.
    pub fn new(name: String, args: T) -> Self {
        Self {
            name,
            args,
            resources: None,
            meta: None,
        }
    }
}

/// Represents the output of a tool execution.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ToolOutput<T> {
    /// The output from the tool.
    pub output: T,

    /// The resources generated by the tool execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Resource>>,

    /// The usage statistics for the tool execution.
    pub usage: Usage,
}

impl<T> ToolOutput<T> {
    /// Creates a new tool output with the given output value.
    pub fn new(output: T) -> Self {
        Self {
            output,
            resources: None,
            usage: Usage::default(),
        }
    }
}

/// Represents the metadata for an agent or tool request.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RequestMeta {
    /// The target engine principal for the request.
    pub engine: Option<Principal>,

    /// The target thread for the request. If not provided, a new thread will be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<ThreadId>,

    /// Gets the username from request context.
    /// Note: This is not verified and should not be used as a trusted identifier.
    /// For example, if triggered by a bot of X platform, this might be the username
    /// of the user interacting with the bot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Represents the usage statistics for the agent or tool execution.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Usage {
    /// input tokens sent to the LLM
    pub input_tokens: u64,

    /// output tokens received from the LLM
    pub output_tokens: u64,

    /// number of requests made to agents and tools
    pub requests: u64,
}

impl Usage {
    /// Accumulates the usage statistics from another usage object.
    pub fn accumulate(&mut self, other: &Usage) {
        self.input_tokens = self.input_tokens.saturating_add(other.input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(other.output_tokens);
        self.requests = self.requests.saturating_add(other.requests);
    }
}

/// Represents a tool call response with it's ID, function name, and arguments.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ToolCall {
    /// tool call id.
    pub id: String,

    /// tool function name.
    pub name: String,

    /// tool function  arguments.
    pub args: String,

    /// The result of the tool call, auto processed by agents engine, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Represents a function definition with its metadata.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Function {
    /// Definition of the function.
    pub definition: FunctionDefinition,

    /// The tags of resource that this function supports.
    pub supported_resource_tags: Vec<String>,
}

/// Defines a callable function with its metadata and schema.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FunctionDefinition {
    /// Name of the function.
    pub name: String,

    /// Description of what the function does.
    pub description: String,

    /// JSON schema defining the function's parameters.
    pub parameters: Value,

    /// Whether to enable strict schema adherence when generating the function call. If set to true, the model will follow the exact schema defined in the parameters field. Only a subset of JSON Schema is supported when strict is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl FunctionDefinition {
    /// Modifies the function name with a prefix.
    pub fn name_with_prefix(mut self, prefix: &str) -> Self {
        self.name = format!("{}{}", prefix, self.name);
        self
    }
}

/// Returns the number of tokens in the given content in the simplest way.
pub fn evaluate_tokens(content: &str) -> usize {
    content.len() / 3
}
