# Error Handling Examples

## suzunari-error with snafu

This project uses `suzunari-error` macro to enhance error handling with stack trace context.

### Basic Pattern

```rust
use snafu::ResultExt;
use suzunari_error::*;

#[suzunari_error]
pub enum FileError {
    #[snafu(display("Failed to read file: {}", path.display()))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Failed to write file: {}", path.display()))]
    WriteFile {
        source: std::io::Error,
        path: PathBuf,
    },
}

// Usage
fn read_config(path: &Path) -> Result<String, FileError> {
    std::fs::read_to_string(path)
        .context(ReadFileSnafu { path })
}
```

### Error Propagation

The `#[suzunari_error]` macro automatically adds stack trace context when errors are propagated:

```rust
use suzunari_error::*;

#[suzunari_error]
pub enum CliError {
    #[snafu(display("Either provide a message as argument or use --input-file"))]
    NoMessageOrInput,

    #[snafu(transparent)]
    File { source: FileError },
}

#[suzunari_error]
pub enum MainError {
    #[snafu(transparent)]
    Cli { source: CliError },

    #[snafu(transparent)]
    Template { source: TemplateError },
}

// Error automatically includes stack trace showing:
// MainError::Cli -> CliError::File -> FileError::ReadFile
```

### Benefits

1. **Automatic Context**: Each error includes the call stack
2. **snafu Integration**: Works seamlessly with snafu's context patterns
3. **Transparent Propagation**: Use `#[snafu(transparent)]` for error chains
4. **Better Debugging**: See exactly where errors originated and how they propagated

### Real Example

From `cli-app/anflug/src/`:

```rust
// cli.rs
#[suzunari_error]
pub enum CliError {
    #[snafu(display("Either provide a message as argument or use --input-file"))]
    NoMessageOrInput,
    // ...
}

// file.rs
#[suzunari_error]
pub enum FileError {
    #[snafu(display("Failed to read file: {}", path.display()))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },
}

// main.rs
#[suzunari_error]
pub enum MainError {
    #[snafu(transparent)]
    Cli { source: CliError },

    #[snafu(transparent)]
    File { source: FileError },
}

fn main() -> Result<(), MainError> {
    // Errors automatically include full propagation path
    run_cli()?;
    Ok(())
}
```

### When to Use

- ✅ Complex applications with multiple error layers
- ✅ When detailed error tracing is critical
- ✅ Applications where debugging production errors is important
- ❌ Simple scripts with flat error structures
- ❌ Performance-critical hot paths (adds minimal overhead)

### Dependencies

```toml
[dependencies]
suzunari-error = { git = "https://github.com/ryotan/suzunari-error.git", version = "0.1.0" }
suzunari-error-macro-impl = { git = "https://github.com/ryotan/suzunari-error.git", version = "0.1.0" }
snafu = "0.8"
```
