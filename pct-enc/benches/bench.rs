use std::fs;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use pct_enc::{naive::*, *};

criterion_group!(benches, bench_validate);
criterion_main!(benches);

fn bench_validate(c: &mut Criterion) {
    let src = fs::read("enc.txt").unwrap();

    let mut group = c.benchmark_group("validate");
    group.throughput(Throughput::Bytes(src.len() as u64));

    group.bench_function("naive_bitset", |b| {
        b.iter(|| table_bitset::PATH.validate(&src))
    });
    group.bench_function("naive_bool_array", |b| {
        b.iter(|| table_bool_array::PATH.validate(&src))
    });
    group.bench_function("ssse3_triple_loadu", |b| {
        b.iter(|| unsafe { validate_ssse3_triple_loadu(&src) })
    });
    group.bench_function("ssse3_alignr", |b| {
        b.iter(|| unsafe { validate_ssse3_alignr(&src) })
    });
    group.bench_function("ssse3_bsrli", |b| {
        b.iter(|| unsafe { validate_ssse3_bsrli(&src) })
    });
}
