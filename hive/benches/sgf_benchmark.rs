use criterion::{criterion_group, criterion_main, Criterion};
use hive::sgf_parser::read_sgf_file;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random game", |b| b.iter(|| read_sgf_file("./test_data/T!HV-stepanzo-tzimarou-2020-07-31-0524.sgf")));
}

criterion_group!(sgf_benches, criterion_benchmark);
criterion_main!(sgf_benches);
