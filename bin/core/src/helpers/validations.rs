//! # Input Validation Module
//!
//! This module provides validation functions for user inputs to prevent
//! invalid data from entering the system and improve security.

use anyhow::Context;
use mogh_validations::{StringValidator, StringValidatorMatches};

use crate::config::core_config;

/// Minimum length for usernames
pub const MIN_USERNAME_LENGTH: usize = 1;
/// Maximum length for usernames
pub const MAX_USERNAME_LENGTH: usize = 100;

/// Validate usernames
///
/// - Between [MIN_USERNAME_LENGTH] and [MAX_USERNAME_LENGTH] characters
/// - Matches `^[a-zA-Z0-9._@-]+$`
pub fn validate_username(username: &str) -> anyhow::Result<()> {
  StringValidator::default()
    .min_length(MIN_USERNAME_LENGTH)
    .max_length(MAX_USERNAME_LENGTH)
    .matches(StringValidatorMatches::Username)
    .validate(username)
    .context("Failed to validate username")
}

/// Maximum length for passwords
pub const MAX_PASSWORD_LENGTH: usize = 1000;

/// Validate passwords
///
/// - Between [CoreConfig::min_password_length][komodo_client::entities::config::core::CoreConfig::min_password_length] and [MAX_PASSWORD_LENGTH] characters
pub fn validate_password(password: &str) -> anyhow::Result<()> {
  StringValidator::default()
    .min_length(core_config().min_password_length as usize)
    .max_length(MAX_PASSWORD_LENGTH)
    .validate(password)
    .context("Failed to validate password")
}

/// Maximum length for API key names
pub const MAX_API_KEY_NAME_LENGTH: usize = 200;

/// Validate api key names
///
/// - Greater than [MAX_API_KEY_NAME_LENGTH] characters
pub fn validate_api_key_name(name: &str) -> anyhow::Result<()> {
  StringValidator::default()
    .max_length(MAX_API_KEY_NAME_LENGTH)
    .validate(name)
    .context("Failed to validate api key name")
}

/// Minimum length for variable names
pub const MIN_VARIABLE_NAME_LENGTH: usize = 1;
/// Maximum length for variable names
pub const MAX_VARIABLE_NAME_LENGTH: usize = 500;

/// Validate variable names
///
/// - Between [MIN_VARIABLE_NAME_LENGTH] and [MAX_VARIABLE_NAME_LENGTH] characters
/// - Reserves the `vault` / `vault.*` namespace for external Secret Providers
/// - Matches `^[a-zA-Z_][a-zA-Z0-9_]*$`
pub fn validate_variable_name(name: &str) -> anyhow::Result<()> {
  let trimmed = name.trim();
  if trimmed.eq_ignore_ascii_case("vault")
    || trimmed.as_bytes().get(..6).is_some_and(|p| {
      p.eq_ignore_ascii_case(b"vault.")
        || p.eq_ignore_ascii_case(b"vault:")
        || p.eq_ignore_ascii_case(b"vault/")
    })
  {
    anyhow::bail!(
      "Variable name '{name}' is reserved for external Secret Providers ('vault.<provider>.<key>')"
    );
  }
  StringValidator::default()
    .min_length(MIN_VARIABLE_NAME_LENGTH)
    .max_length(MAX_VARIABLE_NAME_LENGTH)
    .matches(StringValidatorMatches::VariableName)
    .validate(name)
    .context("Failed to validate variable name")
}

/// Maximum length for variable values
pub const MAX_VARIABLE_VALUE_LENGTH: usize = 10000;

/// Validate variable values
///
/// - Less than [MAX_VARIABLE_VALUE_LENGTH] characters
pub fn validate_variable_value(value: &str) -> anyhow::Result<()> {
  StringValidator::default()
    .max_length(MAX_VARIABLE_VALUE_LENGTH)
    .validate(value)
    .context("Failed to validate variable value")
}

#[cfg(test)]
mod tests {
  use super::validate_variable_name;

  #[test]
  fn reserves_vault_namespace_for_external_secret_providers() {
    assert!(validate_variable_name("APP_ENV").is_ok());
    assert!(validate_variable_name("VAULT_ADDR").is_ok());
    assert!(validate_variable_name("vault").is_err());
    assert!(validate_variable_name("VAULT").is_err());
    assert!(
      validate_variable_name("vault.infisical.DB_PASSWORD").is_err()
    );
    assert!(
      validate_variable_name("VAULT.infisical.DB_PASSWORD").is_err()
    );
    assert!(validate_variable_name("vault:secret").is_err());
  }
}
