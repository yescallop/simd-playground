use std::fs;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use pct_enc::naive::PATH_TABLE;

criterion_group!(benches, bench_validate);
criterion_main!(benches);

fn bench_validate(c: &mut Criterion) {
    let src = fs::read("input/enc.txt").unwrap();

    let mut group = c.benchmark_group("validate");
    group.throughput(Throughput::Bytes(src.len() as u64));

    group.bench_function("naive", |b| b.iter(|| assert!(PATH_TABLE.validate(&src))));
}
