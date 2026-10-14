use chrono::{DateTime, Local};
use suzunari_error::*;

use crate::config::{Config, DateFormats};

#[suzunari_error]
#[suzu(display("Failed to format datetime"))]
pub struct TemplateError {}

pub struct TemplateProcessor {
    date_formats: DateFormats,
}

impl TemplateProcessor {
    pub fn new(config: &Config) -> Self {
        Self { date_formats: config.date_formats.clone().unwrap_or_default() }
    }

    pub fn process(&self, template: &str, message: &str) -> Result<String, TemplateError> {
        let now = Local::now();
        let mut result = template.to_string();

        // Replace message parameter
        result = result.replace("{message}", message);

        // Replace date/time parameters
        result = self.replace_datetime_params(result, &now)?;

        // Handle multiline templates with indentation
        if result.contains('\n') {
            result = self.process_multiline_indentation(&result);
        }

        Ok(result)
    }

    pub fn process_filename(&self, filename: &str) -> Result<String, TemplateError> {
        let now = Local::now();
        self.replace_datetime_params(filename.to_string(), &now)
    }

    fn replace_datetime_params(
        &self,
        mut text: String,
        now: &DateTime<Local>,
    ) -> Result<String, TemplateError> {
        // Standard date formats
        if let Some(date_fmt) = &self.date_formats.date {
            text = text.replace("{date}", &now.format(date_fmt).to_string());
        }

        if let Some(time_fmt) = &self.date_formats.time {
            text = text.replace("{time}", &now.format(time_fmt).to_string());
        }

        // Datetime combination
        text = text.replace(
            "{datetime}",
            &format!(
                "{} {}",
                now.format(self.date_formats.date.as_deref().unwrap_or("%Y-%m-%d")),
                now.format(self.date_formats.time.as_deref().unwrap_or("%H:%M:%S"))
            ),
        );

        // Timestamp
        text = text.replace("{timestamp}", &now.timestamp().to_string());

        // Custom formats
        if let Some(custom_date) = &self.date_formats.custom_date {
            text = text.replace("{custom_date}", &now.format(custom_date).to_string());
        }

        if let Some(custom_time) = &self.date_formats.custom_time {
            text = text.replace("{custom_time}", &now.format(custom_time).to_string());
        }

        // Additional custom formats from config
        for (key, format) in &self.date_formats.custom {
            let placeholder = format!("{{{}}}", key);
            text = text.replace(&placeholder, &now.format(format).to_string());
        }

        Ok(text)
    }

    fn process_multiline_indentation(&self, template: &str) -> String {
        let lines: Vec<&str> = template.lines().collect();
        if lines.len() <= 1 {
            return template.to_string();
        }

        // Find the base indentation from the first line
        let first_line = lines[0];
        let base_indent = first_line.chars().take_while(|c| c.is_whitespace()).collect::<String>();

        // Apply base indentation to subsequent lines
        lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    line.to_string()
                } else if line.trim().is_empty() {
                    // Preserve empty lines
                    String::new()
                } else {
                    // Apply base indentation to non-empty lines
                    format!("{}{}", base_indent, line.trim_start())
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DateFormats};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut custom = HashMap::new();
        custom.insert("test_format".to_string(), "%Y/%m/%d".to_string());

        Config {
            file: None,
            pattern: None,
            template: None,
            date_formats: Some(DateFormats {
                date: Some("%Y-%m-%d".to_string()),
                time: Some("%H:%M:%S".to_string()),
                custom_date: Some("%Y年%m月%d日".to_string()),
                custom_time: Some("%H時%M分".to_string()),
                custom,
            }),
            error_handling: None,
            execution: None,
        }
    }

    #[test]
    fn test_template_processor_new() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);
        assert!(processor.date_formats.date.is_some());
    }

    #[test]
    fn test_process_message_replacement() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let result = processor.process("Hello {message}!", "World").unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_process_date_replacement() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let result = processor.process("Date: {date}", "test").unwrap();
        assert!(result.starts_with("Date: "));
        assert!(result.contains("-"));
    }

    #[test]
    fn test_process_custom_date_replacement() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let result = processor.process("日付: {custom_date}", "test").unwrap();
        assert!(result.starts_with("日付: "));
        assert!(result.contains("年"));
    }

    #[test]
    fn test_process_custom_format() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let result = processor.process("Custom: {test_format}", "test").unwrap();
        assert!(result.starts_with("Custom: "));
        assert!(result.contains("/"));
    }

    #[test]
    fn test_process_filename() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let result = processor.process_filename("notes-{date}.md").unwrap();
        assert!(result.starts_with("notes-"));
        assert!(result.ends_with(".md"));
        assert!(result.contains("-"));
    }

    #[test]
    fn test_multiline_indentation() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let template = "  First line: {message}\n  Second line\n  Third line";
        let result = processor.process(template, "test").unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("  First line: test"));
        assert!(lines[1].starts_with("  Second line"));
        assert!(lines[2].starts_with("  Third line"));
    }

    #[test]
    fn test_multiline_with_empty_lines() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let template = "  First line: {message}\n\n  Third line";
        let result = processor.process(template, "test").unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("  First line: test"));
        assert_eq!(lines[1], "");
        assert!(lines[2].starts_with("  Third line"));
    }

    #[test]
    fn test_single_line_no_indentation_processing() {
        let config = create_test_config();
        let processor = TemplateProcessor::new(&config);

        let template = "Single line: {message}";
        let result = processor.process(template, "test").unwrap();
        assert_eq!(result, "Single line: test");
    }
}
