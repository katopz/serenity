//! Configuration abstraction for cross-platform support.
//!
//! This module provides configuration loading from environment variables,
//! which works consistently across native platforms and Cloudflare Workers (WASM).

use secrecy::{ExposeSecret, SecretString};
use std::env;

/// Error types for configuration loading.
#[derive(Debug)]
pub enum ConfigError {
    /// Discord token environment variable not found.
    TokenNotFound,
    /// Application ID environment variable not found.
    ApplicationIdNotFound,
    /// Failed to parse application ID.
    ApplicationIdParseError(String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenNotFound => write!(f, "DISCORD_TOKEN environment variable not found"),
            Self::ApplicationIdNotFound => {
                write!(f, "DISCORD_APPLICATION_ID environment variable not found")
            },
            Self::ApplicationIdParseError(e) => {
                write!(f, "Failed to parse DISCORD_APPLICATION_ID: {}", e)
            },
        }
    }
}

/// Get Discord token from environment variable.
///
/// The token is wrapped in `SecretString` to prevent accidental logging.
///
/// # Environment Variable
///
/// - `DISCORD_TOKEN`: Your Discord bot token
///
/// # Errors
///
/// Returns `ConfigError::TokenNotFound` if the environment variable is not set.
///
/// # Examples
///
/// ```rust,no_run
/// use serenity::internal::config::get_token;
///
/// let token = get_token().expect("DISCORD_TOKEN must be set");
/// ```
pub fn get_token() -> Result<SecretString, ConfigError> {
    env::var("DISCORD_TOKEN").map(SecretString::new).map_err(|_| ConfigError::TokenNotFound)
}

/// Get Discord application ID from environment variable.
///
/// # Environment Variable
///
/// - `DISCORD_APPLICATION_ID`: Your Discord application ID
///
/// # Errors
///
/// Returns `ConfigError::ApplicationIdNotFound` if the environment variable is not set.
/// Returns `ConfigError::ApplicationIdParseError` if the value cannot be parsed as u64.
///
/// # Examples
///
/// ```rust,no_run
/// use serenity::internal::config::get_application_id;
///
/// let app_id = get_application_id().expect("DISCORD_APPLICATION_ID must be set");
/// ```
pub fn get_application_id() -> Result<u64, ConfigError> {
    let value =
        env::var("DISCORD_APPLICATION_ID").map_err(|_| ConfigError::ApplicationIdNotFound)?;

    value.parse().map_err(|e| ConfigError::ApplicationIdParseError(e.to_string()))
}

/// Check if all required configuration is available.
///
/// This is useful for early validation before starting the application.
///
/// # Examples
///
/// ```rust,no_run
/// use serenity::internal::config::validate_config;
///
/// if let Err(e) = validate_config() {
///     eprintln!("Configuration error: {}", e);
///     std::process::exit(1);
/// }
/// ```
pub fn validate_config() -> Result<(), ConfigError> {
    // Token is required
    let _ = get_token()?;

    // Application ID is optional, so we only validate if it's set
    if env::var("DISCORD_APPLICATION_ID").is_ok() {
        let _ = get_application_id()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_not_found() {
        // Remove token if it exists
        env::remove_var("DISCORD_TOKEN");

        assert!(matches!(get_token(), Err(ConfigError::TokenNotFound)));
    }

    #[test]
    fn test_application_id_not_found() {
        env::remove_var("DISCORD_APPLICATION_ID");

        assert!(matches!(get_application_id(), Err(ConfigError::ApplicationIdNotFound)));
    }

    #[test]
    fn test_application_id_parse_error() {
        env::set_var("DISCORD_APPLICATION_ID", "not_a_number");

        assert!(matches!(get_application_id(), Err(ConfigError::ApplicationIdParseError(_))));

        env::remove_var("DISCORD_APPLICATION_ID");
    }

    #[test]
    fn test_application_id_valid() {
        env::set_var("DISCORD_APPLICATION_ID", "123456789");

        assert_eq!(get_application_id().unwrap(), 123456789);

        env::remove_var("DISCORD_APPLICATION_ID");
    }

    #[test]
    fn test_validate_config_missing_token() {
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");

        assert!(matches!(validate_config(), Err(ConfigError::TokenNotFound)));
    }

    #[test]
    fn test_validate_config_with_valid_token() {
        env::set_var("DISCORD_TOKEN", "test_token_123");
        env::remove_var("DISCORD_APPLICATION_ID");

        assert!(validate_config().is_ok());

        env::remove_var("DISCORD_TOKEN");
    }
}
