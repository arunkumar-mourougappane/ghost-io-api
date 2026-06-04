//! Error types for the Ghost API client.
//!
//! This module provides the [`GhostError`] enum which encompasses all possible
//! errors that can occur when interacting with the Ghost API.
//!
//! # Examples
//!
//! ```
//! use ghost_io_api::error::{GhostError, Result};
//!
//! fn example_function() -> Result<String> {
//!     // Your code here
//!     Ok("success".to_string())
//! }
//! ```

/// A specialized `Result` type for Ghost API operations.
///
/// This type is used throughout the crate for any operation that may produce
/// a [`GhostError`].
pub type Result<T> = std::result::Result<T, GhostError>;

/// The error type for Ghost API operations.
///
/// This enum represents all possible errors that can occur when interacting
/// with the Ghost API, including network errors, serialization errors, API
/// errors, and authentication errors.
#[derive(thiserror::Error, Debug)]
pub enum GhostError {
    /// An error returned by the Ghost API.
    ///
    /// Ghost returns structured JSON errors with a message, error type,
    /// and optional context information.
    #[error("Ghost API error ({error_type}): {message}")]
    Api {
        /// The human-readable error message.
        message: String,
        /// The error type identifier (e.g., "NotFoundError", "ValidationError").
        error_type: String,
        /// Optional additional context about the error.
        context: Option<String>,
    },

    /// An HTTP error from the underlying HTTP client.
    ///
    /// This includes network errors, connection timeouts, DNS resolution
    /// failures, and HTTP status code errors.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A JSON serialization or deserialization error.
    ///
    /// This occurs when the response from Ghost cannot be parsed or when
    /// a request body cannot be serialized.
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// An authentication error.
    ///
    /// This includes JWT generation failures, invalid API keys, or other
    /// authentication-related issues.
    #[error("Authentication error: {0}")]
    Auth(String),
}

impl GhostError {
    /// Creates a new API error with the given message and error type.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghost_io_api::error::GhostError;
    ///
    /// let error = GhostError::api(
    ///     "Resource not found",
    ///     "NotFoundError",
    ///     None,
    /// );
    /// ```
    pub fn api(
        message: impl Into<String>,
        error_type: impl Into<String>,
        context: Option<String>,
    ) -> Self {
        Self::Api {
            message: message.into(),
            error_type: error_type.into(),
            context,
        }
    }

    /// Creates a new authentication error.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghost_io_api::error::GhostError;
    ///
    /// let error = GhostError::auth("Invalid API key");
    /// ```
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Returns `true` if this is an API error.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghost_io_api::error::GhostError;
    ///
    /// let error = GhostError::api("Not found", "NotFoundError", None);
    /// assert!(error.is_api_error());
    /// ```
    pub fn is_api_error(&self) -> bool {
        matches!(self, Self::Api { .. })
    }

    /// Returns `true` if this is an HTTP error.
    pub fn is_http_error(&self) -> bool {
        matches!(self, Self::Http(_))
    }

    /// Returns `true` if this is a JSON serialization error.
    pub fn is_json_error(&self) -> bool {
        matches!(self, Self::Json(_))
    }

    /// Returns `true` if this is an authentication error.
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
    }

    /// Returns the error type for API errors.
    ///
    /// Returns `None` for non-API errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use ghost_io_api::error::GhostError;
    ///
    /// let error = GhostError::api("Not found", "NotFoundError", None);
    /// assert_eq!(error.api_error_type(), Some("NotFoundError"));
    /// ```
    pub fn api_error_type(&self) -> Option<&str> {
        match self {
            Self::Api { error_type, .. } => Some(error_type),
            _ => None,
        }
    }

    /// Returns the error message for API errors.
    ///
    /// Returns `None` for non-API errors.
    pub fn api_message(&self) -> Option<&str> {
        match self {
            Self::Api { message, .. } => Some(message),
            _ => None,
        }
    }

    /// Returns the context for API errors.
    ///
    /// Returns `None` for non-API errors or if no context is available.
    pub fn api_context(&self) -> Option<&str> {
        match self {
            Self::Api { context, .. } => context.as_deref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let error = GhostError::api("Resource not found", "NotFoundError", None);
        assert!(error.is_api_error());
        assert_eq!(error.api_error_type(), Some("NotFoundError"));
        assert_eq!(error.api_message(), Some("Resource not found"));
        assert_eq!(error.api_context(), None);
    }

    #[test]
    fn test_api_error_with_context() {
        let error = GhostError::api(
            "Validation failed",
            "ValidationError",
            Some("Title is required".to_string()),
        );
        assert!(error.is_api_error());
        assert_eq!(error.api_error_type(), Some("ValidationError"));
        assert_eq!(error.api_message(), Some("Validation failed"));
        assert_eq!(error.api_context(), Some("Title is required"));
    }

    #[test]
    fn test_auth_error_creation() {
        let error = GhostError::auth("Invalid API key");
        assert!(error.is_auth_error());
        assert!(!error.is_api_error());
        assert_eq!(error.api_error_type(), None);
    }

    #[test]
    fn test_http_error_from_trait() {
        // Test that From trait works for reqwest::Error
        // We can't easily create a reqwest::Error without actually making a request,
        // so we'll test the type signature and behavior through documentation
        //
        // In real usage:
        // let reqwest_error: reqwest::Error = ...;
        // let ghost_error: GhostError = reqwest_error.into();
        // assert!(ghost_error.is_http_error());

        // We can verify the From trait exists at compile time
        fn _assert_from_impl(e: reqwest::Error) -> GhostError {
            e.into()
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("not valid json").unwrap_err();
        let error: GhostError = json_error.into();
        assert!(error.is_json_error());
        assert!(!error.is_api_error());
    }

    #[test]
    fn test_error_display() {
        let error = GhostError::api("Not found", "NotFoundError", None);
        let display = format!("{}", error);
        assert!(display.contains("NotFoundError"));
        assert!(display.contains("Not found"));
    }

    #[test]
    fn test_error_display_with_context() {
        let error = GhostError::api(
            "Validation failed",
            "ValidationError",
            Some("Title required".to_string()),
        );
        let display = format!("{}", error);
        assert!(display.contains("ValidationError"));
        assert!(display.contains("Validation failed"));
    }

    #[test]
    fn test_auth_error_display() {
        let error = GhostError::auth("JWT generation failed");
        let display = format!("{}", error);
        assert!(display.contains("Authentication error"));
        assert!(display.contains("JWT generation failed"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<String> {
            Ok("success".to_string())
        }

        let result = returns_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_result_with_error() {
        fn returns_error() -> Result<String> {
            Err(GhostError::auth("Failed"))
        }

        let result = returns_error();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_auth_error());
    }

    #[test]
    fn test_error_type_checks() {
        let api_error = GhostError::api("Test", "TestError", None);
        assert!(api_error.is_api_error());
        assert!(!api_error.is_http_error());
        assert!(!api_error.is_json_error());
        assert!(!api_error.is_auth_error());

        let auth_error = GhostError::auth("Test");
        assert!(!auth_error.is_api_error());
        assert!(!auth_error.is_http_error());
        assert!(!auth_error.is_json_error());
        assert!(auth_error.is_auth_error());
    }

    #[test]
    fn test_api_error_accessors() {
        let error = GhostError::api("Test message", "TestType", Some("Test context".to_string()));

        assert_eq!(error.api_message(), Some("Test message"));
        assert_eq!(error.api_error_type(), Some("TestType"));
        assert_eq!(error.api_context(), Some("Test context"));
    }

    #[test]
    fn test_non_api_error_accessors_return_none() {
        let error = GhostError::auth("Test");

        assert_eq!(error.api_message(), None);
        assert_eq!(error.api_error_type(), None);
        assert_eq!(error.api_context(), None);
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<GhostError>();
        assert_sync::<GhostError>();
    }

    #[test]
    fn test_error_debug() {
        let error = GhostError::api("Debug test", "DebugError", None);
        let debug = format!("{:?}", error);
        assert!(debug.contains("Api"));
        assert!(debug.contains("Debug test"));
        assert!(debug.contains("DebugError"));
    }
}
