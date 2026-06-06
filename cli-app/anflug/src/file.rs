use regex::Regex;
use std::fs;
use std::path::Path;
use suzunari_error::*;

use crate::cli::MatchMode;
use crate::config::Config;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum FileError {
    #[suzu(display("Failed to read file: {}", path.display()))]
    ReadFile { source: std::io::Error, path: std::path::PathBuf },
    #[suzu(display("Failed to read input file: {}", path.display()))]
    ReadInputFile { source: std::io::Error, path: std::path::PathBuf },
    #[suzu(display("Failed to write to file: {}", path.display()))]
    WriteFile { source: std::io::Error, path: std::path::PathBuf },
    #[suzu(display("Failed to create backup: {}", path.display()))]
    CreateBackup { source: std::io::Error, path: std::path::PathBuf },
    #[suzu(display("Failed to create directory: {}", path.display()))]
    CreateDirectory { source: std::io::Error, path: std::path::PathBuf },
    #[suzu(display("Invalid regex pattern: {}", pattern))]
    InvalidRegex { source: regex::Error, pattern: String },
    #[suzu(display("Directory does not exist: {}", path.display()))]
    DirectoryNotExists { path: std::path::PathBuf },
    #[suzu(display("Pattern not found"))]
    PatternNotFound,
    #[suzu(display("Invalid no_directory setting: {}", setting))]
    InvalidNoDirectorySetting { setting: String },
    #[suzu(display("Invalid no_match setting: {}", setting))]
    InvalidNoMatchSetting { setting: String },
}

/// Options for processing files with pattern-based insertion
#[derive(Debug, Clone)]
pub struct ProcessFileOptions {
    /// Pattern to match for insertion point (regex)
    pub pattern: Option<String>,
    /// How to handle multiple pattern matches
    pub match_mode: MatchMode,
    /// Insert before pattern match instead of after
    pub insert_before: bool,
    /// Append to end of file without pattern matching
    pub append_mode: bool,
    /// Create backup file before making changes
    pub backup: bool,
    /// Show what would be done without making changes
    pub dry_run: bool,
}

/// Builder for ProcessFileOptions
#[derive(Debug, Clone)]
pub struct ProcessFileOptionsBuilder {
    pattern: Option<String>,
    match_mode: MatchMode,
    insert_before: bool,
    append_mode: bool,
    backup: bool,
    dry_run: bool,
}

impl ProcessFileOptionsBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            pattern: None,
            match_mode: MatchMode::First,
            insert_before: false,
            append_mode: false,
            backup: false,
            dry_run: false,
        }
    }

    /// Set the pattern for matching
    pub fn pattern<T: Into<String>>(mut self, pattern: T) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set the pattern for matching with Option
    pub fn pattern_opt<T: Into<String>>(mut self, pattern: Option<T>) -> Self {
        self.pattern = pattern.map(|p| p.into());
        self
    }

    /// Set the match mode
    pub fn match_mode(mut self, match_mode: MatchMode) -> Self {
        self.match_mode = match_mode;
        self
    }

    /// Set whether to insert before the pattern match
    pub fn insert_before(mut self, insert_before: bool) -> Self {
        self.insert_before = insert_before;
        self
    }

    /// Set whether to use append mode
    pub fn append_mode(mut self, append_mode: bool) -> Self {
        self.append_mode = append_mode;
        self
    }

    /// Set whether to create backup
    pub fn backup(mut self, backup: bool) -> Self {
        self.backup = backup;
        self
    }

    /// Set whether to run in dry-run mode
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Build the ProcessFileOptions
    pub fn build(self) -> ProcessFileOptions {
        ProcessFileOptions {
            pattern: self.pattern,
            match_mode: self.match_mode,
            insert_before: self.insert_before,
            append_mode: self.append_mode,
            backup: self.backup,
            dry_run: self.dry_run,
        }
    }
}

impl Default for ProcessFileOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProcessFileOptions {
    fn default() -> Self {
        ProcessFileOptionsBuilder::new().build()
    }
}

impl ProcessFileOptions {
    /// Create a new builder for ProcessFileOptions
    pub fn builder() -> ProcessFileOptionsBuilder {
        ProcessFileOptionsBuilder::new()
    }
}

/// File processor that handles file operations with pattern matching
#[derive(Clone)]
pub struct FileProcessor {
    config: Config,
}

impl FileProcessor {
    /// Create a new FileProcessor with the given configuration
    pub fn new(config: &Config) -> Self {
        Self { config: config.clone() }
    }

    /// Process a file with the given content and options
    pub fn process_file(
        &self,
        file_path: &Path,
        content: &str,
        options: &ProcessFileOptions,
    ) -> Result<(), FileError> {
        // Create backup if requested
        if options.backup && !options.dry_run && file_path.exists() {
            self.create_backup(file_path)?;
        }

        // Read existing file content or create empty if doesn't exist
        let existing_content = if file_path.exists() {
            fs::read_to_string(file_path)
                .context(ReadFileError { path: file_path.to_path_buf() })?
        } else {
            self.ensure_parent_directory(file_path)?;
            String::new()
        };

        let new_content = match options.pattern.as_ref() {
            // Pattern-based insertion
            Some(pattern) if !options.append_mode => self.insert_with_pattern(
                &existing_content,
                content,
                pattern,
                &options.match_mode,
                options.insert_before,
            )?,
            // Simple append to end
            _ => self.append_content(&existing_content, content),
        };

        if options.dry_run {
            self.print_dry_run_output(file_path, &new_content);
        } else {
            self.write_file(file_path, &new_content)?;
        }

        Ok(())
    }

    /// Read content from an input file
    pub fn read_input_file(&self, path: &Path) -> Result<String, FileError> {
        fs::read_to_string(path).context(ReadInputFileError { path: path.to_path_buf() })
    }

    /// Append content to existing content with proper newline handling
    fn append_content(&self, existing_content: &str, new_content: &str) -> String {
        if existing_content.is_empty() {
            new_content.to_string()
        } else if existing_content.ends_with('\n') {
            format!("{}{}", existing_content, new_content)
        } else {
            format!("{}\n{}", existing_content, new_content)
        }
    }

    /// Print dry-run output to console
    fn print_dry_run_output(&self, file_path: &Path, content: &str) {
        println!("Would write to {}:", file_path.display());
        println!("---");
        println!("{}", content);
        println!("---");
    }

    /// Write content to file
    fn write_file(&self, file_path: &Path, content: &str) -> Result<(), FileError> {
        fs::write(file_path, content).context(WriteFileError { path: file_path.to_path_buf() })
    }

    /// Create a backup of the file
    fn create_backup(&self, file_path: &Path) -> Result<(), FileError> {
        let backup_path = self.get_backup_path(file_path);
        fs::copy(file_path, &backup_path)
            .context(CreateBackupError { path: backup_path.clone() })?;
        Ok(())
    }

    /// Get the backup file path
    fn get_backup_path(&self, file_path: &Path) -> std::path::PathBuf {
        if let Some(extension) = file_path.extension() {
            // file.txt -> file.txt.bak
            file_path.with_extension(format!("{}.bak", extension.to_str().unwrap_or("")))
        } else {
            // file -> file.bak
            file_path.with_extension("bak")
        }
    }

    /// Ensure the parent directory exists
    fn ensure_parent_directory(&self, file_path: &Path) -> Result<(), FileError> {
        if let Some(parent) = file_path.parent()
            && !parent.exists()
        {
            let no_dir_action = self.get_no_directory_action();
            self.handle_missing_directory(parent, &no_dir_action)?;
        }
        Ok(())
    }

    /// Get the no_directory action from config
    fn get_no_directory_action(&self) -> String {
        self.config
            .error_handling
            .as_ref()
            .and_then(|eh| eh.no_directory.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("create")
            .to_string()
    }

    /// Handle missing directory based on configuration
    fn handle_missing_directory(&self, parent: &Path, action: &str) -> Result<(), FileError> {
        match action {
            "create" => {
                fs::create_dir_all(parent)
                    .context(CreateDirectoryError { path: parent.to_path_buf() })?;
            }
            "error" => {
                return DirectoryNotExistsError { path: parent.to_path_buf() }.fail();
            }
            _ => {
                return InvalidNoDirectorySettingError { setting: action.to_string() }.fail();
            }
        }
        Ok(())
    }

    /// Insert content at pattern-matched locations
    fn insert_with_pattern(
        &self,
        existing_content: &str,
        new_content: &str,
        pattern: &str,
        match_mode: &MatchMode,
        insert_before: bool,
    ) -> Result<String, FileError> {
        let regex = self.compile_regex(pattern)?;
        let segments = self.split_content_preserving_line_endings(existing_content);
        let matches = self.find_pattern_matches(&segments, &regex);

        if matches.is_empty() {
            return self.handle_no_pattern_match(existing_content, new_content);
        }

        let target_indices = self.resolve_target_indices(&matches, match_mode);
        let eol_seq = self.detect_eol_sequence(existing_content);

        self.build_result_with_insertions(
            &segments,
            new_content,
            &target_indices,
            eol_seq,
            insert_before,
        )
    }

    /// Compile regex pattern
    fn compile_regex(&self, pattern: &str) -> Result<Regex, FileError> {
        Regex::new(pattern).context(InvalidRegexError { pattern: pattern.to_string() })
    }

    /// Split content preserving line endings
    fn split_content_preserving_line_endings<'a>(&self, content: &'a str) -> Vec<&'a str> {
        if content.is_empty() { Vec::new() } else { content.split_inclusive('\n').collect() }
    }

    /// Find segments that match the pattern
    fn find_pattern_matches(&self, segments: &[&str], regex: &Regex) -> Vec<usize> {
        let mut matches = Vec::new();
        for (i, seg) in segments.iter().enumerate() {
            let line = self.strip_line_endings(seg);
            if regex.is_match(line) {
                matches.push(i);
            }
        }
        matches
    }

    /// Strip line endings from a segment for matching
    fn strip_line_endings<'a>(&self, segment: &'a str) -> &'a str {
        let mut line = segment;
        if let Some(stripped) = line.strip_suffix('\n') {
            line = stripped;
            if let Some(stripped_cr) = line.strip_suffix('\r') {
                line = stripped_cr;
            }
        }
        line
    }

    /// Handle case when no pattern matches are found
    fn handle_no_pattern_match(
        &self,
        existing_content: &str,
        new_content: &str,
    ) -> Result<String, FileError> {
        let no_match_action = self.get_no_match_action();

        match no_match_action.as_str() {
            "append_end" => Ok(self.append_content(existing_content, new_content)),
            "error" => PatternNotFoundError.fail(),
            _ => InvalidNoMatchSettingError { setting: no_match_action }.fail(),
        }
    }

    /// Get the no_match action from config
    fn get_no_match_action(&self) -> String {
        self.config
            .error_handling
            .as_ref()
            .and_then(|eh| eh.no_match.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("append_end")
            .to_string()
    }

    /// Resolve which indices to target based on match mode
    fn resolve_target_indices(&self, matches: &[usize], match_mode: &MatchMode) -> Vec<usize> {
        match match_mode {
            MatchMode::First => vec![matches[0]],
            MatchMode::Last => vec![*matches.last().unwrap()],
            MatchMode::All => matches.to_vec(),
        }
    }

    /// Detect the end-of-line sequence used in the content
    fn detect_eol_sequence(&self, content: &str) -> &'static str {
        if content.contains("\r\n") { "\r\n" } else { "\n" }
    }

    /// Build the result by inserting content at target indices
    fn build_result_with_insertions(
        &self,
        segments: &[&str],
        new_content: &str,
        target_indices: &[usize],
        eol_seq: &str,
        insert_before: bool,
    ) -> Result<String, FileError> {
        let mut result = String::with_capacity(
            segments.iter().map(|s| s.len()).sum::<usize>()
                + new_content.len() * target_indices.len()
                + target_indices.len() * eol_seq.len(),
        );

        let mut ti_iter = target_indices.iter();
        let mut next_target = ti_iter.next().copied();

        for (i, &seg) in segments.iter().enumerate() {
            if Some(i) == next_target {
                if insert_before {
                    self.insert_before_segment(&mut result, new_content, eol_seq, seg);
                } else {
                    self.insert_after_segment(
                        &mut result,
                        seg,
                        new_content,
                        eol_seq,
                        i,
                        segments.len(),
                    );
                }
                next_target = ti_iter.next().copied();
            } else {
                result.push_str(seg);
            }
        }

        Ok(result)
    }

    /// Insert content before a segment
    fn insert_before_segment(
        &self,
        result: &mut String,
        new_content: &str,
        eol_seq: &str,
        seg: &str,
    ) {
        result.push_str(new_content);
        result.push_str(eol_seq);
        result.push_str(seg);
    }

    /// Insert content after a segment
    #[allow(clippy::too_many_arguments)]
    fn insert_after_segment(
        &self,
        result: &mut String,
        seg: &str,
        new_content: &str,
        eol_seq: &str,
        index: usize,
        total_segments: usize,
    ) {
        result.push_str(seg);
        let has_eol = seg.ends_with('\n');

        if has_eol {
            // The segment already ends with a line ending, so just add content,
            // and add a separator only if there are more segments following.
            result.push_str(new_content);
            if index < total_segments - 1 {
                result.push_str(eol_seq);
            }
        } else {
            // No line ending on the matched segment (likely file end). Add a separator, then content.
            result.push_str(eol_seq);
            result.push_str(new_content);
            if index < total_segments - 1 {
                result.push_str(eol_seq);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ErrorHandling};
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_config() -> Config {
        Config {
            file: None,
            pattern: None,
            template: None,
            date_formats: None,
            error_handling: Some(ErrorHandling {
                no_match: Some("append_end".to_string()),
                no_file: Some("create".to_string()),
                no_directory: Some("create".to_string()),
            }),
            execution: None,
        }
    }

    fn create_default_options() -> ProcessFileOptions {
        ProcessFileOptions::default()
    }

    #[test]
    fn test_process_file_options_builder() {
        let options = ProcessFileOptionsBuilder::new()
            .pattern_opt(Some("test pattern"))
            .match_mode(MatchMode::Last)
            .insert_before(true)
            .append_mode(false)
            .backup(true)
            .dry_run(false)
            .build();

        assert_eq!(options.pattern, Some("test pattern".to_string()));
        assert_eq!(options.match_mode, MatchMode::Last);
        assert!(options.insert_before);
        assert!(!options.append_mode);
        assert!(options.backup);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_process_file_options_default() {
        let options = ProcessFileOptions::default();
        assert_eq!(options.pattern, None);
        assert_eq!(options.match_mode, MatchMode::First);
        assert!(!options.insert_before);
        assert!(!options.append_mode);
        assert!(!options.backup);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_file_processor_new() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);
        assert!(processor.config.error_handling.is_some());
    }

    #[test]
    fn test_read_input_file() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        let content = processor.read_input_file(temp_file.path()).unwrap();
        assert_eq!(content.trim(), "test content");
    }

    #[test]
    fn test_process_file_append_mode() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "existing content").unwrap();

        let options = ProcessFileOptionsBuilder::new().append_mode(true).build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("existing content"));
        assert!(content.contains("new content"));
    }

    #[test]
    fn test_process_file_new_file() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");

        let options = create_default_options();

        processor.process_file(&file_path, "test content", &options).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_process_file_pattern_match() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Header").unwrap();
        writeln!(temp_file, "## Section").unwrap();
        writeln!(temp_file, "content").unwrap();

        let options = ProcessFileOptionsBuilder::new().pattern("## Section").build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "# Header");
        assert_eq!(lines[1], "## Section");
        assert_eq!(lines[2], "new content");
        assert_eq!(lines[3], "content");
    }

    #[test]
    fn test_process_file_insert_before() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Header").unwrap();
        writeln!(temp_file, "## Section").unwrap();
        writeln!(temp_file, "content").unwrap();

        let options = ProcessFileOptionsBuilder::new()
            .pattern("## Section")
            .insert_before(true)
            .build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "# Header");
        assert_eq!(lines[1], "new content");
        assert_eq!(lines[2], "## Section");
        assert_eq!(lines[3], "content");
    }

    #[test]
    fn test_process_file_match_last() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "## Section 1").unwrap();
        writeln!(temp_file, "content 1").unwrap();
        writeln!(temp_file, "## Section 2").unwrap();
        writeln!(temp_file, "content 2").unwrap();

        let options = ProcessFileOptionsBuilder::new()
            .pattern("## Section")
            .match_mode(MatchMode::Last)
            .build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[2], "## Section 2");
        assert_eq!(lines[3], "new content");
        assert_eq!(lines[4], "content 2");
    }

    #[test]
    fn test_process_file_dry_run() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "original content").unwrap();

        let options = ProcessFileOptionsBuilder::new().dry_run(true).build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content.trim(), "original content"); // File should be unchanged
    }

    #[test]
    fn test_create_backup() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "original content").unwrap();

        processor.create_backup(temp_file.path()).unwrap();

        let backup_path = processor.get_backup_path(temp_file.path());
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content.trim(), "original content");
    }

    #[test]
    fn test_create_backup_with_extension() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        processor.create_backup(&file_path).unwrap();

        let backup_path = temp_dir.path().join("test.txt.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_ensure_parent_directory() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("subdir").join("file.txt");

        processor.ensure_parent_directory(&nested_path).unwrap();
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_pattern_no_match_append_end() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let existing_content = "line 1\nline 2";
        let result = processor
            .insert_with_pattern(
                existing_content,
                "new content",
                "nomatch",
                &MatchMode::First,
                false,
            )
            .unwrap();

        assert!(result.contains("line 1"));
        assert!(result.contains("line 2"));
        assert!(result.contains("new content"));
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let result =
            processor.insert_with_pattern("content", "new", "[invalid", &MatchMode::First, false);

        assert!(result.is_err());
    }

    #[test]
    fn test_process_file_match_all() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "## Section 1").unwrap();
        writeln!(temp_file, "content 1").unwrap();
        writeln!(temp_file, "## Section 2").unwrap();
        writeln!(temp_file, "content 2").unwrap();

        let options = ProcessFileOptionsBuilder::new()
            .pattern("## Section")
            .match_mode(MatchMode::All)
            .build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "## Section 1");
        assert_eq!(lines[1], "new content");
        assert_eq!(lines[2], "content 1");
        assert_eq!(lines[3], "## Section 2");
        assert_eq!(lines[4], "new content");
        assert_eq!(lines[5], "content 2");
    }

    #[test]
    fn test_error_handling_no_directory_error() {
        let mut config = create_test_config();
        config.error_handling = Some(ErrorHandling {
            no_match: Some("append_end".to_string()),
            no_file: Some("create".to_string()),
            no_directory: Some("error".to_string()),
        });
        let processor = FileProcessor::new(&config);

        let temp_dir = TempDir::new().unwrap();
        let non_existent_dir = temp_dir.path().join("nonexistent").join("file.txt");

        let result = processor.ensure_parent_directory(&non_existent_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_no_match_error() {
        let mut config = create_test_config();
        config.error_handling = Some(ErrorHandling {
            no_match: Some("error".to_string()),
            no_file: Some("create".to_string()),
            no_directory: Some("create".to_string()),
        });
        let processor = FileProcessor::new(&config);

        let result = processor.insert_with_pattern(
            "content without pattern",
            "new content",
            "nonexistent",
            &MatchMode::First,
            false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_invalid_no_directory_setting() {
        let mut config = create_test_config();
        config.error_handling = Some(ErrorHandling {
            no_match: Some("append_end".to_string()),
            no_file: Some("create".to_string()),
            no_directory: Some("invalid_setting".to_string()),
        });
        let processor = FileProcessor::new(&config);

        let temp_dir = TempDir::new().unwrap();
        let non_existent_dir = temp_dir.path().join("nonexistent").join("file.txt");

        let result = processor.ensure_parent_directory(&non_existent_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_invalid_no_match_setting() {
        let mut config = create_test_config();
        config.error_handling = Some(ErrorHandling {
            no_match: Some("invalid_setting".to_string()),
            no_file: Some("create".to_string()),
            no_directory: Some("create".to_string()),
        });
        let processor = FileProcessor::new(&config);

        let result = processor.insert_with_pattern(
            "content without pattern",
            "new content",
            "nonexistent",
            &MatchMode::First,
            false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_process_file_with_backup() {
        let config = create_test_config();
        let processor = FileProcessor::new(&config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "original content").unwrap();

        let options = ProcessFileOptionsBuilder::new().backup(true).build();

        processor.process_file(temp_file.path(), "new content", &options).unwrap();

        let backup_path = processor.get_backup_path(temp_file.path());
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content.trim(), "original content");

        let new_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(new_content.contains("original content"));
        assert!(new_content.contains("new content"));
    }
}
