use anflug::{Config, FileProcessor, MatchMode, ProcessFileOptions};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use tempfile::TempDir;

fn bench_regex_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_caching");

    let config = Config::default();
    let processor = FileProcessor::new(&config);

    // Create test content
    let content = (0..100)
        .map(|i| format!("## Section {}\nSome content here\n", i))
        .collect::<String>();

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.md");
    fs::write(&file_path, &content).unwrap();

    // Test cache hit performance
    group.bench_function("cache_hit", |b| {
        let options = ProcessFileOptions::builder()
            .pattern("## Section")
            .match_mode(MatchMode::First)
            .build();

        // First call to populate cache
        processor
            .process_file(&file_path, "test", &options)
            .unwrap();

        b.iter(|| {
            // Subsequent calls should hit cache
            processor
                .process_file(
                    black_box(&file_path),
                    black_box("New content"),
                    black_box(&options),
                )
                .unwrap();
        });
    });

    // Test cache miss performance (different patterns)
    group.bench_function("cache_miss", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let options = ProcessFileOptions::builder()
                .pattern(&format!("## Section {}", counter % 1000))
                .match_mode(MatchMode::First)
                .build();

            processor
                .process_file(
                    black_box(&file_path),
                    black_box("New content"),
                    black_box(&options),
                )
                .unwrap();
        });
    });

    // Test concurrent cache access
    group.bench_function("concurrent_access", |b| {
        use std::thread;

        let processor_clone = processor.clone();
        let file_path_clone = file_path.clone();

        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let processor = processor_clone.clone();
                    let file_path = file_path_clone.clone();
                    thread::spawn(move || {
                        let options = ProcessFileOptions::builder()
                            .pattern("## Section")
                            .match_mode(MatchMode::First)
                            .build();

                        processor
                            .process_file(&file_path, "content", &options)
                            .unwrap();
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_regex_cache_performance);
criterion_main!(benches);
