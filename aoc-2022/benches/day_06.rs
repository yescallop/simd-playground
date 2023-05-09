#![feature(test)]

extern crate test;

use std::{fs, io};

use aoc_simd::day_06::*;
use test::Bencher;

#[bench]
fn d6p1_avx512(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.bytes = input.len() as u64;
    b.iter(|| part1_avx512(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p1_xor(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.iter(|| part1_xor(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p1_naive(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.iter(|| part1_naive(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p1_naive_short_circuit(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.iter(|| part1_naive_short_circuit(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p2_avx512(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.bytes = input.len() as u64;
    b.iter(|| part2_avx512(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p2_xor(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.iter(|| part2_xor(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p2_naive(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.iter(|| part2_naive(input.as_bytes()));
    Ok(())
}

#[bench]
fn d6p2_naive_short_circuit(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    b.iter(|| part2_naive_short_circuit(input.as_bytes()));
    Ok(())
}
