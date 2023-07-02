use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn demon_core_str(string: &str) -> demon_core::Str {
    demon_core::Str::from(string)
}

fn std_str(string: &str) -> String {
    String::from(string)
}

fn demon_core_len(str: demon_core::Str) -> usize {
    str.len()
}

fn std_len(str: String) -> usize {
    str.len()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("demon core Str, create short", |b| {
        b.iter(|| demon_core_str(black_box("valid utf-8")))
    });
    c.bench_function("demon core Str, create long", |b| {
        b.iter(|| demon_core_str(black_box("this is a long and valid utf-8")))
    });
    c.bench_function("std String create", |b| {
        b.iter(|| std_str(black_box("valid utf-8")))
    });

    c.bench_function("demon core Str, len short", |b| {
        b.iter(|| demon_core_len(black_box(demon_core::Str::from("valid utf-8"))))
    });
    c.bench_function("std String len", |b| {
        b.iter(|| std_len(black_box(String::from("valid utf-8"))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
