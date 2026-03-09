pub mod cli;
pub mod config;
pub mod file;
pub mod template;

use clap::Parser;
use std::path::PathBuf;
use suzunari_error::*;

use cli::Cli;
use config::load_config;
use file::{FileProcessor, ProcessFileOptions};
use template::TemplateProcessor;

// Default constants
const DEFAULT_FILE_PATTERN: &str = "notes-{date}.md";
const DEFAULT_TEMPLATE: &str = "{message}";

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum MainError {
    #[suzu(transparent)]
    Cli { source: cli::CliError },
    #[suzu(display("Failed to load configuration: {source}"))]
    Config { source: confy::ConfyError },
    #[suzu(display("Failed to read input file: {source}"))]
    InputFile { source: file::FileError },
    #[suzu(display("Failed to process template: {source}"))]
    Template { source: template::TemplateError },
    #[suzu(display("Failed to process file: {source}"))]
    File { source: file::FileError },
}

#[suzunari_error::report]
fn main() -> Result<(), MainError> {
    let args = Cli::parse();

    // Validate CLI arguments
    args.validate()?;

    // Load configuration
    let config = load_config(args.config.clone()).context(ConfigError)?;

    // Initialize processors
    let template_processor = TemplateProcessor::new(&config);
    let file_processor = FileProcessor::new(&config);

    // Get message content
    let message = if let Some(input_file) = &args.input_file {
        file_processor.read_input_file(input_file).context(InputFileError)?
    } else {
        // Validation ensures message is Some when input_file is None
        args.message
            .expect("Message should be present when input_file is None - validation failed")
    };

    // Determine template
    let template = args
        .template
        .or_else(|| config.template.clone())
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());

    // Process template
    let processed_content =
        template_processor.process(&template, &message).context(TemplateError)?;

    // Determine output file
    let file_pattern = args
        .file
        .or_else(|| config.file.clone())
        .unwrap_or_else(|| DEFAULT_FILE_PATTERN.to_string());

    let output_file = template_processor.process_filename(&file_pattern).context(TemplateError)?;

    let output_path = PathBuf::from(output_file);

    // Determine pattern
    let pattern = args.pattern.as_deref().or(config.pattern.as_deref());

    // Resolve match_mode with precedence: CLI > config > default
    let match_mode = args
        .match_mode
        .or_else(|| {
            config.execution.as_ref().and_then(|e| e.match_mode.as_ref()).and_then(
                |mode| match mode.as_str() {
                    "first" => Some(cli::MatchMode::First),
                    "last" => Some(cli::MatchMode::Last),
                    "all" => Some(cli::MatchMode::All),
                    _ => None,
                },
            )
        })
        .unwrap_or(cli::MatchMode::First);

    let insert_before = args.insert_before
        || config
            .execution
            .as_ref()
            .and_then(|e| e.insert_position.as_ref())
            .map(|pos| pos == "before")
            .unwrap_or(false);

    let backup =
        args.backup || config.execution.as_ref().and_then(|e| e.backup_enabled).unwrap_or(false);

    // Process file
    let options = ProcessFileOptions {
        pattern: pattern.map(|s| s.to_string()),
        match_mode,
        insert_before,
        append_mode: args.append_mode,
        backup,
        dry_run: args.dry_run,
    };

    file_processor
        .process_file(&output_path, &processed_content, &options)
        .context(FileError)?;

    if !args.dry_run {
        println!("Successfully appended to: {}", output_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_main_integration_with_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("test-config.toml");
        let output_file = temp_dir.path().join("test-output.md");

        // Create a config file
        let config_content = format!(
            r#"
file = "{}"
template = "Config test: {{message}}"
"#,
            output_file.to_str().unwrap()
        );
        fs::write(&config_file, config_content).unwrap();

        // Simulate CLI args with config file
        let cli = Cli {
            message: Some("test message".to_string()),
            file: None,
            pattern: None,
            template: None,
            config: Some(config_file),
            input_file: None,
            insert_before: false,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: None,
        };

        // Load config and test the main logic flow
        let config = load_config(cli.config.clone()).unwrap();
        let template_processor = TemplateProcessor::new(&config);
        let file_processor = FileProcessor::new(&config);

        let message = cli.message.unwrap();
        let template = cli
            .template
            .or_else(|| config.template.clone())
            .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());

        let processed_content = template_processor.process(&template, &message).unwrap();

        let file_pattern = cli
            .file
            .or_else(|| config.file.clone())
            .unwrap_or_else(|| DEFAULT_FILE_PATTERN.to_string());

        let output_file_name = template_processor.process_filename(&file_pattern).unwrap();
        let output_path = PathBuf::from(output_file_name);

        // Test successful processing
        let match_mode = cli.match_mode.unwrap_or(cli::MatchMode::First);
        let options = ProcessFileOptions {
            pattern: None,
            match_mode,
            insert_before: false,
            append_mode: false,
            backup: false,
            dry_run: false,
        };
        file_processor.process_file(&output_path, &processed_content, &options).unwrap();

        let content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "Config test: test message");
    }

    #[test]
    fn test_main_with_execution_settings() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("exec-config.toml");
        let output_file = temp_dir.path().join("exec-test.md");

        // Create config with execution settings
        let config_content = format!(
            r#"
file = "{}"

[execution]
match_mode = "last"
insert_position = "before"
backup_enabled = true
"#,
            output_file.to_str().unwrap()
        );
        fs::write(&config_file, config_content).unwrap();

        // Create initial file content
        fs::write(&output_file, "Line 1\nLine 2\nLine 3\n").unwrap();

        let cli = Cli {
            message: Some("test content".to_string()),
            file: None,
            pattern: Some("Line 2".to_string()),
            template: None,
            config: Some(config_file),
            input_file: None,
            insert_before: false,
            append_mode: false,
            dry_run: false,
            backup: false,
            match_mode: None, // Will be resolved from config
        };

        let config = load_config(cli.config.clone()).unwrap();
        let _template_processor = TemplateProcessor::new(&config);
        let _file_processor = FileProcessor::new(&config);

        // Test execution settings application
        let match_mode = cli
            .match_mode
            .or_else(|| {
                config.execution.as_ref().and_then(|e| e.match_mode.as_ref()).and_then(|mode| {
                    match mode.as_str() {
                        "first" => Some(cli::MatchMode::First),
                        "last" => Some(cli::MatchMode::Last),
                        "all" => Some(cli::MatchMode::All),
                        _ => None,
                    }
                })
            })
            .unwrap_or(cli::MatchMode::First);

        assert!(matches!(match_mode, cli::MatchMode::Last));

        let insert_before = cli.insert_before
            || config
                .execution
                .as_ref()
                .and_then(|e| e.insert_position.as_ref())
                .map(|pos| pos == "before")
                .unwrap_or(false);

        assert!(insert_before);

        let backup =
            cli.backup || config.execution.as_ref().and_then(|e| e.backup_enabled).unwrap_or(false);

        assert!(backup);
    }
}
