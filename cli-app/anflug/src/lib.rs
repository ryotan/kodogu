//! # Anflug - A Simple CLI Tool for Text File Manipulation
//!
//! Anflug is a command-line tool designed to append text to files with advanced pattern matching,
//! template processing, and configuration management capabilities.
//!
//! ## Features
//!
//! - **Pattern-based insertion**: Insert content at specific locations using regex patterns
//! - **Template processing**: Support for dynamic content with date/time placeholders
//! - **Flexible configuration**: TOML-based configuration with environment variable support
//! - **Multiple match modes**: First, last, or all pattern matches
//! - **Backup support**: Automatic backup creation before modifications
//! - **Dry run mode**: Preview changes without modifying files
//!
//! ## Quick Start
//!
//! ### Basic Usage
//!
//! ```rust
//! use anflug::{FileProcessor, ProcessFileOptions, MatchMode};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a simple configuration
//! let config = anflug::Config::default();
//! let processor = FileProcessor::new(&config);
//!
//! // Process a file with basic options
//! let options = ProcessFileOptions {
//!     pattern: None,
//!     match_mode: MatchMode::First,
//!     insert_before: false,
//!     append_mode: true,
//!     backup: false,
//!     dry_run: false,
//! };
//!
//! // This would append "Hello, World!" to the end of the file
//! // processor.process_file(Path::new("example.txt"), "Hello, World!", &options)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Pattern-based Insertion
//!
//! ```rust
//! use anflug::{FileProcessor, ProcessFileOptions, MatchMode};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = anflug::Config::default();
//! let processor = FileProcessor::new(&config);
//!
//! // Insert content after the first occurrence of "## Notes"
//! let options = ProcessFileOptions {
//!     pattern: Some("## Notes".to_string()),
//!     match_mode: MatchMode::First,
//!     insert_before: false,
//!     append_mode: false,
//!     backup: true,
//!     dry_run: false,
//! };
//!
//! // This would insert content after the "## Notes" heading
//! // processor.process_file(Path::new("notes.md"), "- New note item", &options)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Template Processing
//!
//! ```rust
//! use anflug::{TemplateProcessor, Config};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::default();
//! let processor = TemplateProcessor::new(&config);
//!
//! let template = "Entry for {date}: {message}";
//! let result = processor.process(template, "Task completed")?;
//! // Result: "Entry for 2024-01-15: Task completed"
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod config;
pub mod file;
pub mod template;

#[cfg(feature = "async")]
pub mod async_file;

pub use cli::*;
pub use config::*;
pub use file::*;
pub use template::*;

#[cfg(feature = "async")]
pub use async_file::*;
