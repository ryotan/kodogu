use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationError};

/// Configuration for the anflug application with validation support.
///
/// This structure holds all configuration options that can be set via
/// configuration files, environment variables, or command-line arguments.
/// All fields are validated to ensure they contain sensible values.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    /// Default file pattern
    #[validate(length(min = 1, max = 1000, message = "File path must be between 1 and 1000 characters"))]
    pub file: Option<String>,

    /// Default pattern for insertion
    #[validate(custom(function = "validate_regex_pattern"))]
    pub pattern: Option<String>,

    /// Default template for formatting
    #[validate(length(max = 10000, message = "Template must not exceed 10000 characters"))]
    pub template: Option<String>,

    /// Date format configurations
    pub date_formats: Option<DateFormats>,

    /// Error handling settings
    pub error_handling: Option<ErrorHandling>,

    /// Execution settings
    pub execution: Option<ExecutionSettings>,
}

/// Validates that a pattern is a valid regex.
fn validate_regex_pattern(pattern: &str) -> Result<(), ValidationError> {
    match regex::Regex::new(pattern) {
        Ok(_) => Ok(()),
        Err(e) => {
            let mut error = ValidationError::new("invalid_regex");
            error.message = Some(format!("Invalid regex pattern: {}", e).into());
            Err(error)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFormats {
    pub date: Option<String>,
    pub time: Option<String>,
    pub custom_date: Option<String>,
    pub custom_time: Option<String>,
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandling {
    /// What to do when pattern doesn't match: "append_end" | "error"
    pub no_match: Option<String>,

    /// What to do when file doesn't exist: "create" | "error"
    pub no_file: Option<String>,

    /// What to do when directory doesn't exist: "create" | "error"
    pub no_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSettings {
    /// Enable automatic backup
    pub backup_enabled: Option<bool>,

    /// Default match mode: "first" | "last" | "all"
    pub match_mode: Option<String>,

    /// Default insert position: "after" | "before"
    pub insert_position: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file: Some("notes-{date}.md".to_string()),
            pattern: None,
            template: Some("{message}".to_string()),
            date_formats: Some(DateFormats::default()),
            error_handling: Some(ErrorHandling::default()),
            execution: Some(ExecutionSettings::default()),
        }
    }
}

impl Default for DateFormats {
    fn default() -> Self {
        Self {
            date: Some("%Y-%m-%d".to_string()),
            time: Some("%H:%M:%S".to_string()),
            custom_date: Some("%Y年%m月%d日".to_string()),
            custom_time: Some("%H時%M分".to_string()),
            custom: HashMap::new(),
        }
    }
}

impl Default for ErrorHandling {
    fn default() -> Self {
        Self {
            no_match: Some("append_end".to_string()),
            no_file: Some("create".to_string()),
            no_directory: Some("create".to_string()),
        }
    }
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            backup_enabled: Some(false),
            match_mode: Some("first".to_string()),
            insert_position: Some("after".to_string()),
        }
    }
}

pub fn load_config(config_path: Option<std::path::PathBuf>) -> Result<Config, confy::ConfyError> {
    match config_path {
        Some(path) => {
            let config: Config = confy::load_path(path)?;
            Ok(config)
        }
        None => {
            let config: Config = confy::load("anflug", None)?;
            Ok(config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.file, Some("notes-{date}.md".to_string()));
        assert_eq!(config.template, Some("{message}".to_string()));
        assert!(config.date_formats.is_some());
        assert!(config.error_handling.is_some());
        assert!(config.execution.is_some());
    }

    #[test]
    fn test_date_formats_default() {
        let date_formats = DateFormats::default();
        assert_eq!(date_formats.date, Some("%Y-%m-%d".to_string()));
        assert_eq!(date_formats.time, Some("%H:%M:%S".to_string()));
        assert_eq!(date_formats.custom_date, Some("%Y年%m月%d日".to_string()));
        assert_eq!(date_formats.custom_time, Some("%H時%M分".to_string()));
    }

    #[test]
    fn test_error_handling_default() {
        let error_handling = ErrorHandling::default();
        assert_eq!(error_handling.no_match, Some("append_end".to_string()));
        assert_eq!(error_handling.no_file, Some("create".to_string()));
        assert_eq!(error_handling.no_directory, Some("create".to_string()));
    }

    #[test]
    fn test_execution_settings_default() {
        let execution = ExecutionSettings::default();
        assert_eq!(execution.backup_enabled, Some(false));
        assert_eq!(execution.match_mode, Some("first".to_string()));
        assert_eq!(execution.insert_position, Some("after".to_string()));
    }

    #[test]
    fn test_load_config_none() {
        let result = load_config(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_config_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let toml_file = temp_dir.path().join("test-config.toml");

        let toml_content = r#"
file = "test-{date}.md"
template = "- {message}"

[date_formats]
date = "%Y-%m-%d"
time = "%H:%M:%S"

[error_handling]
no_match = "append_end"
no_file = "create"
no_directory = "create"

[execution]
backup_enabled = true
match_mode = "last"
insert_position = "before"
"#;
        std::fs::write(&toml_file, toml_content).unwrap();

        let config = load_config(Some(toml_file)).unwrap();
        assert_eq!(config.file, Some("test-{date}.md".to_string()));
        assert_eq!(config.template, Some("- {message}".to_string()));

        // Test execution settings
        let execution = config.execution.as_ref().unwrap();
        assert_eq!(execution.backup_enabled, Some(true));
        assert_eq!(execution.match_mode, Some("last".to_string()));
        assert_eq!(execution.insert_position, Some("before".to_string()));
    }
}
