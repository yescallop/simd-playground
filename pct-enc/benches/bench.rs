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

    group.bench_function("sse41_3load", |b| {
        b.iter(|| unsafe { sse41::validate_3load(&src) })
    });
    group.bench_function("sse41_alignr", |b| {
        b.iter(|| unsafe { sse41::validate_alignr(&src) })
    });
    group.bench_function("sse41_shift", |b| {
        b.iter(|| unsafe { sse41::validate_shift(&src) })
    });
    group.bench_function("sse41_shift_transposed", |b| {
        b.iter(|| unsafe { sse41::validate_shift_transposed(&src) })
    });

    group.bench_function("avx2_3load", |b| {
        b.iter(|| unsafe { avx2::validate_3load(&src) })
    });
    group.bench_function("avx2_alignr", |b| {
        b.iter(|| unsafe { avx2::validate_alignr(&src) })
    });

    group.bench_function("avx512_3load", |b| {
        b.iter(|| unsafe { avx512::validate_3load(&src) })
    });
    group.bench_function("avx512_3load_gf2p8affine", |b| {
        b.iter(|| unsafe { avx512::validate_3load_gf2p8affine(&src) })
    });
    group.bench_function("avx512_3load_perm", |b| {
        b.iter(|| unsafe { avx512::validate_3load_perm(&src) })
    });
}
