//! Error types for agent-mcp.

use thiserror::Error;

/// Result type for agent-mcp operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for agent-mcp.
#[derive(Debug, Error)]
pub enum Error {
    /// No providers available.
    #[error("no providers available: {0}")]
    NoProviders(String),

    /// Provider error.
    #[error("provider error: {0}")]
    Provider(#[from] embeddenator_webpuppet::Error),

    /// Workflow error.
    #[error("workflow error: {0}")]
    Workflow(String),

    /// Invalid workflow state.
    #[error("invalid workflow state: {0}")]
    InvalidState(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Rate limited.
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// Timeout.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Invalid parameters.
    #[error("invalid parameters: {0}")]
    InvalidParams(String),

    /// Protocol error.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Map a domain [`Error`] onto an MCP JSON-RPC error (`rmcp::ErrorData`).
///
/// Never-silent (G2): bad-input variants surface as `invalid_params`; everything
/// else is an `internal_error`. The message is always carried through verbatim.
impl From<Error> for rmcp::ErrorData {
    fn from(e: Error) -> Self {
        match &e {
            Error::InvalidParams(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
            Error::NoProviders(_) => rmcp::ErrorData::invalid_request(e.to_string(), None),
            _ => rmcp::ErrorData::internal_error(e.to_string(), None),
        }
    }
}
