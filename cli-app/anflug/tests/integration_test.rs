use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_basic_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("test-output.md");

    // Test basic message append
    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            output_file.to_str().unwrap(),
            "Hello, integration test!",
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());
    assert!(output_file.exists());

    let content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "Hello, integration test!");
}

#[test]
fn test_cli_template_processing() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("template-test.md");

    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            output_file.to_str().unwrap(),
            "-t",
            "Message: {message}",
            "test content",
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());

    let content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "Message: test content");
}

#[test]
fn test_cli_pattern_matching() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("pattern-test.md");

    // Create initial file
    fs::write(&test_file, "# Header\n## Section\nExisting content\n").unwrap();

    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            test_file.to_str().unwrap(),
            "-p",
            "## Section",
            "New content",
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());

    let content = fs::read_to_string(&test_file).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "# Header");
    assert_eq!(lines[1], "## Section");
    assert_eq!(lines[2], "New content");
    assert_eq!(lines[3], "Existing content");
}

#[test]
fn test_cli_input_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.md");

    // Create input file
    fs::write(&input_file, "Content from file").unwrap();

    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-i",
            input_file.to_str().unwrap(),
            "-f",
            output_file.to_str().unwrap(),
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());

    let content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "Content from file");
}

#[test]
fn test_cli_backup_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("backup-test.txt");
    let backup_file = temp_dir.path().join("backup-test.txt.bak");

    // Create initial file
    fs::write(&test_file, "Original content").unwrap();

    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            test_file.to_str().unwrap(),
            "--backup",
            "New content",
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());
    assert!(backup_file.exists());

    let backup_content = fs::read_to_string(&backup_file).unwrap();
    assert_eq!(backup_content, "Original content");

    let new_content = fs::read_to_string(&test_file).unwrap();
    assert!(new_content.contains("Original content"));
    assert!(new_content.contains("New content"));
}

#[test]
fn test_cli_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("dry-run-test.txt");

    // Create initial file
    fs::write(&test_file, "Original content").unwrap();

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            test_file.to_str().unwrap(),
            "--dry-run",
            "New content",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    // File should be unchanged
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "Original content");

    // Output should show what would be written
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would write to"));
    assert!(stdout.contains("Original content"));
    assert!(stdout.contains("New content"));
}

#[test]
fn test_cli_validation_errors() {
    // Test no message and no input file
    let output = std::process::Command::new("cargo")
        .args(["run", "--"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Either provide a message"));

    // Test both message and input file
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.txt");
    fs::write(&input_file, "content").unwrap();

    let output = std::process::Command::new("cargo")
        .args(["run", "--", "-i", input_file.to_str().unwrap(), "message"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot specify both"));
}

#[test]
fn test_cli_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test-config.toml");
    let output_file = temp_dir.path().join("config-test.md");

    // Create config file
    let config_content = format!(
        r#"
file = "{}"
template = "Config: {{message}}"

[execution]
backup_enabled = false
match_mode = "first"
"#,
        output_file.to_str().unwrap()
    );
    fs::write(&config_file, config_content).unwrap();

    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-c",
            config_file.to_str().unwrap(),
            "test message",
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());

    let content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "Config: test message");
}

#[test]
fn test_cli_multiline_template() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("multiline-test.md");

    let exit_code = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            output_file.to_str().unwrap(),
            "-t",
            "  Item: {message}\n  Status: pending",
            "test task",
        ])
        .status()
        .unwrap();

    assert!(exit_code.success());

    let content = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "  Item: test task");
    assert_eq!(lines[1], "  Status: pending");
}

#[test]
fn test_cli_environment_variables() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("env-test.md");

    let exit_code = std::process::Command::new("cargo")
        .args(["run", "--", "Environment test"])
        .env("ANFLUG_FILE", output_file.to_str().unwrap())
        .env("ANFLUG_TEMPLATE", "Env: {message}")
        .status()
        .unwrap();

    assert!(exit_code.success());

    let content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "Env: Environment test");
}
