use anflug::{Config, FileProcessor, MatchMode, ProcessFileOptions};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use tempfile::TempDir;

fn generate_test_content(lines: usize) -> String {
    let mut content = String::with_capacity(lines * 50);
    for i in 0..lines {
        if i % 10 == 0 {
            content.push_str(&format!("## Section {}\n", i / 10));
        } else {
            content.push_str(&format!("Line {} with some content that makes it realistic\n", i));
        }
    }
    content
}

fn bench_file_processing_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_processing_by_size");

    let config = Config::default();
    let processor = FileProcessor::new(&config);

    for size in [100, 1000, 10000].iter() {
        let content = generate_test_content(*size);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, &content).unwrap();

        group.bench_with_input(BenchmarkId::new("append_mode", size), size, |b, &_size| {
            let options = ProcessFileOptions::builder().append_mode(true).build();

            b.iter(|| {
                processor
                    .process_file(
                        black_box(&file_path),
                        black_box("New content line"),
                        black_box(&options),
                    )
                    .unwrap();
            });
        });

        group.bench_with_input(BenchmarkId::new("pattern_match", size), size, |b, &_size| {
            let options = ProcessFileOptions::builder()
                .pattern("## Section")
                .match_mode(MatchMode::First)
                .build();

            b.iter(|| {
                processor
                    .process_file(
                        black_box(&file_path),
                        black_box("New content line"),
                        black_box(&options),
                    )
                    .unwrap();
            });
        });
    }

    group.finish();
}

fn bench_pattern_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_complexity");

    let config = Config::default();
    let processor = FileProcessor::new(&config);
    let content = generate_test_content(1000);
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.md");
    fs::write(&file_path, &content).unwrap();

    let patterns = [
        ("simple", "## Section"),
        ("regex", r"^## Section \d+"),
        ("complex", r"^## Section (?:1[0-9]|[2-9][0-9])"),
    ];

    for (name, pattern) in patterns.iter() {
        group.bench_function(*name, |b| {
            let options = ProcessFileOptions::builder()
                .pattern(*pattern)
                .match_mode(MatchMode::All)
                .build();

            b.iter(|| {
                processor
                    .process_file(
                        black_box(&file_path),
                        black_box("New content"),
                        black_box(&options),
                    )
                    .unwrap();
            });
        });
    }

    group.finish();
}

fn bench_match_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_modes");

    let config = Config::default();
    let processor = FileProcessor::new(&config);
    let content = generate_test_content(1000);
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.md");
    fs::write(&file_path, &content).unwrap();

    let modes = [("first", MatchMode::First), ("last", MatchMode::Last), ("all", MatchMode::All)];

    for (name, mode) in modes.iter() {
        group.bench_function(*name, |b| {
            let options = ProcessFileOptions::builder()
                .pattern("## Section")
                .match_mode(mode.clone())
                .build();

            b.iter(|| {
                processor
                    .process_file(
                        black_box(&file_path),
                        black_box("New content"),
                        black_box(&options),
                    )
                    .unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_file_processing_sizes,
    bench_pattern_complexity,
    bench_match_modes
);
criterion_main!(benches);
