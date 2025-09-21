use anflug::{Config, TemplateProcessor};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_template_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_processing");

    let config = Config::default();
    let processor = TemplateProcessor::new(&config);

    // Simple template
    group.bench_function("simple_template", |b| {
        let template = "Hello {message}!";
        b.iter(|| {
            processor
                .process(black_box(template), black_box("World"))
                .unwrap();
        });
    });

    // Date template
    group.bench_function("date_template", |b| {
        let template = "Entry for {date}: {message}";
        b.iter(|| {
            processor
                .process(black_box(template), black_box("Task completed"))
                .unwrap();
        });
    });

    // Complex template with multiple placeholders
    group.bench_function("complex_template", |b| {
        let template = "[{date}] [{time}] {message} - Timestamp: {timestamp}";
        b.iter(|| {
            processor
                .process(black_box(template), black_box("Complex log entry"))
                .unwrap();
        });
    });

    // Multiline template
    group.bench_function("multiline_template", |b| {
        let template = "  Task: {message}\n  Date: {date}\n  Time: {time}\n  Status: pending";
        b.iter(|| {
            processor
                .process(black_box(template), black_box("Important task"))
                .unwrap();
        });
    });

    // Large template (simulating a document template)
    group.bench_function("large_template", |b| {
        let template = (0..100)
            .map(|i| format!("Line {}: {{message}} - {{date}}", i))
            .collect::<Vec<_>>()
            .join("\n");

        b.iter(|| {
            processor
                .process(black_box(&template), black_box("Repeated content"))
                .unwrap();
        });
    });

    // Filename processing
    group.bench_function("filename_processing", |b| {
        let filename = "notes-{date}-{time}.md";
        b.iter(|| {
            processor.process_filename(black_box(filename)).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_template_processing);
criterion_main!(benches);
