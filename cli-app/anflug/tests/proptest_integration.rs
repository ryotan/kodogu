use anflug::{Config, FileProcessor, ProcessFileOptions, MatchMode, TemplateProcessor};
use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

// Property-based test for file processing with various inputs
proptest! {
    #[test]
    fn test_file_processing_with_arbitrary_content(
        content in "[a-zA-Z0-9 \n.!?-]{0,1000}",
        message in "[a-zA-Z0-9 .!?-]{1,100}"
    ) {
        let config = Config::default();
        let processor = FileProcessor::new(&config);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Write initial content
        fs::write(&file_path, &content).unwrap();

        let options = ProcessFileOptions::builder()
            .append_mode(true)
            .build();

        // Process should not panic
        let result = processor.process_file(&file_path, &message, &options);
        prop_assert!(result.is_ok());

        // File should exist after processing
        prop_assert!(file_path.exists());

        // File should contain both original content and new message
        let final_content = fs::read_to_string(&file_path).unwrap();
        if !content.is_empty() {
            prop_assert!(final_content.contains(&content));
        }
        prop_assert!(final_content.contains(&message));
    }

    #[test]
    fn test_pattern_matching_robustness(
        lines in prop::collection::vec("[a-zA-Z0-9 #.-]{1,50}", 1..100),
        pattern in "[a-zA-Z0-9 #.-]{1,20}",
        message in "[a-zA-Z0-9 .!?-]{1,50}"
    ) {
        let config = Config::default();
        let processor = FileProcessor::new(&config);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let content = lines.join("\n");
        fs::write(&file_path, &content).unwrap();

        let options = ProcessFileOptions::builder()
            .pattern(&pattern)
            .match_mode(MatchMode::First)
            .build();

        // Processing should either succeed or fail gracefully
        let result = processor.process_file(&file_path, &message, &options);

        // Should not panic - either success or well-formed error
        match result {
            Ok(_) => {
                // If successful, file should still exist and be readable
                prop_assert!(file_path.exists());
                let final_content = fs::read_to_string(&file_path).unwrap();
                prop_assert!(!final_content.is_empty());
            }
            Err(e) => {
                // Errors should be well-formed (contain some descriptive text)
                let error_msg = e.to_string();
                prop_assert!(!error_msg.is_empty());
            }
        }
    }

    #[test]
    fn test_template_processing_robustness(
        template in "[a-zA-Z0-9 {}._-]{1,200}",
        message in "[a-zA-Z0-9 .!?-]{1,100}"
    ) {
        let config = Config::default();
        let processor = TemplateProcessor::new(&config);

        // Template processing should not panic
        let result = processor.process(&template, &message);

        // Should always produce some output or a well-formed error
        match result {
            Ok(output) => {
                // Output should not be empty if template or message is not empty
                if !template.is_empty() && !message.is_empty() {
                    prop_assert!(!output.is_empty());
                }

                // Should contain message if template has {message} placeholder
                if template.contains("{message}") {
                    prop_assert!(output.contains(&message));
                }
            }
            Err(e) => {
                // Errors should be descriptive
                let error_msg = e.to_string();
                prop_assert!(!error_msg.is_empty());
            }
        }
    }

    #[test]
    fn test_builder_pattern_properties(
        pattern in proptest::option::of("[a-zA-Z0-9 #.-]{1,30}"),
        insert_before in any::<bool>(),
        append_mode in any::<bool>(),
        backup in any::<bool>(),
        dry_run in any::<bool>()
    ) {
        let mut builder = ProcessFileOptions::builder();

        if let Some(p) = pattern.as_ref() {
            builder = builder.pattern(p);
        }

        let options = builder
            .insert_before(insert_before)
            .append_mode(append_mode)
            .backup(backup)
            .dry_run(dry_run)
            .build();

        // Verify that builder correctly sets all properties
        prop_assert_eq!(options.pattern, pattern);
        prop_assert_eq!(options.insert_before, insert_before);
        prop_assert_eq!(options.append_mode, append_mode);
        prop_assert_eq!(options.backup, backup);
        prop_assert_eq!(options.dry_run, dry_run);
    }

    #[test]
    fn test_multiline_content_handling(
        lines in prop::collection::vec("[a-zA-Z0-9 .!?-]{0,80}", 1..50),
        message in "[a-zA-Z0-9 .!?-]{1,50}"
    ) {
        let config = Config::default();
        let processor = FileProcessor::new(&config);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = lines.join("\n");
        fs::write(&file_path, &content).unwrap();

        let options = ProcessFileOptions::builder()
            .append_mode(true)
            .build();

        let result = processor.process_file(&file_path, &message, &options);
        prop_assert!(result.is_ok());

        let final_content = fs::read_to_string(&file_path).unwrap();
        let final_lines: Vec<&str> = final_content.lines().collect();

        // Should preserve original line count plus at least the new message
        prop_assert!(final_lines.len() >= lines.len());

        // Should contain the new message
        prop_assert!(final_content.contains(&message));
    }

    #[test]
    fn test_regex_cache_correctness(
        patterns in prop::collection::vec("[a-zA-Z0-9 .-]{1,20}", 1..10),
        content in "[a-zA-Z0-9 \n.-]{50,200}"
    ) {
        let config = Config::default();
        let processor = FileProcessor::new(&config);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, &content).unwrap();

        // Process with same pattern multiple times - should be consistent
        for pattern in &patterns {
            let options1 = ProcessFileOptions::builder()
                .pattern(pattern)
                .match_mode(MatchMode::First)
                .dry_run(true) // Don't modify file
                .build();

            let options2 = ProcessFileOptions::builder()
                .pattern(pattern)
                .match_mode(MatchMode::First)
                .dry_run(true)
                .build();

            let result1 = processor.process_file(&file_path, "test1", &options1);
            let result2 = processor.process_file(&file_path, "test2", &options2);

            // Both should succeed or both should fail with same pattern
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }
    }
}