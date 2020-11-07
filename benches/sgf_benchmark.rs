use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hive::sgf_parser::read_sgf_file;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random game", |b| b.iter(|| read_sgf_file("./test_data/HV-jimthelynx-biboelmo-2020-03-28-2235.sgf")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
