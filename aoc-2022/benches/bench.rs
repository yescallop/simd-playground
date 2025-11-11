use std::fs;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

criterion_group!(benches, bench_d2, bench_d4, bench_d6);
criterion_main!(benches);

fn bench_d2(c: &mut Criterion) {
    use aoc_simd::day_02::*;

    let input = fs::read_to_string("input/02.txt").unwrap();

    let mut group = c.benchmark_group("d2");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("avx512", |b| b.iter(|| solve_avx512(input.as_bytes())));
    group.bench_function("shift", |b| b.iter(|| solve_shift(input.as_bytes())));
    group.bench_function("naive", |b| b.iter(|| solve_naive(&input)));
}

fn bench_d4(c: &mut Criterion) {
    use aoc_simd::day_04::*;

    let input = fs::read_to_string("input/04.txt").unwrap();

    let mut group = c.benchmark_group("d4p1");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("avx512", |b| b.iter(|| part1_avx512(input.as_bytes())));
    group.bench_function("naive", |b| b.iter(|| part1_naive(&input)));
}

fn bench_d6(c: &mut Criterion) {
    use aoc_simd::day_06::*;

    let input = fs::read_to_string("input/06.txt").unwrap();

    let mut group = c.benchmark_group("d6p1");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("avx512", |b| {
        b.iter(|| unsafe { part1_avx512(input.as_bytes()) })
    });
    group.bench_function("xor", |b| b.iter(|| part1_xor(input.as_bytes())));
    group.bench_function("naive", |b| b.iter(|| part1_naive(input.as_bytes())));
    group.bench_function("naive_short_circuit", |b| {
        b.iter(|| part1_naive_short_circuit(input.as_bytes()))
    });

    group.finish();

    let mut group = c.benchmark_group("d6p2");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("avx512", |b| {
        b.iter(|| unsafe { part2_avx512(input.as_bytes()) })
    });
    group.bench_function("xor", |b| b.iter(|| part2_xor(input.as_bytes())));
    group.bench_function("naive", |b| b.iter(|| part2_naive(input.as_bytes())));
    group.bench_function("naive_short_circuit", |b| {
        b.iter(|| part2_naive_short_circuit(input.as_bytes()))
    });

    group.finish();
}
