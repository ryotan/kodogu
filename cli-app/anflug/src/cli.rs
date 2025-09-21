use clap::{Parser, ValueEnum};
use snafu::ensure;
use std::path::PathBuf;
use suzunari_error::*;

#[suzunari_error]
pub enum CliError {
    #[snafu(display("Either provide a message as argument or use --input-file"))]
    NoMessageOrInput,
    #[snafu(display("Cannot specify both message and --input-file"))]
    BothMessageAndInput,
    #[snafu(display("Cannot use --append-mode with --pattern"))]
    AppendModeWithPattern,
    #[snafu(display("--insert-before requires --pattern"))]
    InsertBeforeWithoutPattern,
}

#[derive(Parser, Debug)]
#[command(name = "anflug")]
#[command(about = "A simple CLI tool to append text to files with pattern matching")]
#[command(version)]
pub struct Cli {
    /// The message to append to the file
    pub message: Option<String>,

    /// Output file name (supports parameters like {date}, {time})
    #[arg(short, long, env = "ANFLUG_FILE")]
    pub file: Option<String>,

    /// Pattern to match for insertion point (regex)
    #[arg(short, long, env = "ANFLUG_PATTERN")]
    pub pattern: Option<String>,

    /// Template for output formatting (supports {message}, {time}, {date}, etc.)
    #[arg(short, long, env = "ANFLUG_TEMPLATE")]
    pub template: Option<String>,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Read message from file instead of command line
    #[arg(short, long)]
    pub input_file: Option<PathBuf>,

    /// Insert before pattern match instead of after
    #[arg(long)]
    pub insert_before: bool,

    /// Append to end of file without pattern matching
    #[arg(long)]
    pub append_mode: bool,

    /// Show what would be done without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Create backup file before making changes
    #[arg(long)]
    pub backup: bool,

    /// How to handle multiple pattern matches
    #[arg(long, value_enum)]
    pub match_mode: Option<MatchMode>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum MatchMode {
    /// Match only the first occurrence
    First,
    /// Match only the last occurrence
    Last,
    /// Match all occurrences
    All,
}

impl Cli {
    pub fn validate(&self) -> Result<(), CliError> {
        ensure!(
            !(self.message.is_none() && self.input_file.is_none()),
            NoMessageOrInputSnafu
        );

        ensure!(
            !(self.message.is_some() && self.input_file.is_some()),
            BothMessageAndInputSnafu
        );

        ensure!(
            !(self.append_mode && self.pattern.is_some()),
            AppendModeWithPatternSnafu
        );

        ensure!(
            !(self.insert_before && self.pattern.is_none()),
            InsertBeforeWithoutPatternSnafu
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_validation_no_message_no_input() {
        let cli = Cli {
            message: None,
            input_file: None,
            file: None,
            pattern: None,
            template: None,
            config: None,
            insert_before: false,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: None,
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_cli_validation_both_message_and_input() {
        let cli = Cli {
            message: Some("test".to_string()),
            input_file: Some(PathBuf::from("test.txt")),
            file: None,
            pattern: None,
            template: None,
            config: None,
            insert_before: false,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: Some(MatchMode::First),
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_cli_validation_append_mode_with_pattern() {
        let cli = Cli {
            message: Some("test".to_string()),
            input_file: None,
            file: None,
            pattern: Some("pattern".to_string()),
            template: None,
            config: None,
            insert_before: false,
            append_mode: true,
            dry_run: false,
            backup: false,
            match_mode: Some(MatchMode::First),
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_cli_validation_insert_before_without_pattern() {
        let cli = Cli {
            message: Some("test".to_string()),
            input_file: None,
            file: None,
            pattern: None,
            template: None,
            config: None,
            insert_before: true,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: Some(MatchMode::First),
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_cli_validation_valid_message() {
        let cli = Cli {
            message: Some("test".to_string()),
            input_file: None,
            file: None,
            pattern: None,
            template: None,
            config: None,
            insert_before: false,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: Some(MatchMode::First),
        };
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn test_cli_validation_valid_input_file() {
        let cli = Cli {
            message: None,
            input_file: Some(PathBuf::from("test.txt")),
            file: None,
            pattern: None,
            template: None,
            config: None,
            insert_before: false,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: Some(MatchMode::First),
        };
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn test_match_mode_enum() {
        assert_eq!(MatchMode::First, MatchMode::First);
        assert_eq!(MatchMode::Last, MatchMode::Last);
        assert_eq!(MatchMode::All, MatchMode::All);
        assert_ne!(MatchMode::First, MatchMode::Last);
    }
}
