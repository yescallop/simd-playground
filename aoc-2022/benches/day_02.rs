#![feature(test)]

extern crate test;

use std::{fs, io};

use aoc_simd::day_02::*;
use test::Bencher;

#[bench]
fn d2_avx512(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/02.txt")?;
    b.bytes = input.len() as u64;
    b.iter(|| solve_avx512(input.as_bytes()));
    Ok(())
}

#[bench]
fn d2_shift(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/02.txt")?;
    b.iter(|| solve_shift(input.as_bytes()));
    Ok(())
}

#[bench]
fn d2_naive(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/02.txt")?;
    b.iter(|| solve_naive(&input));
    Ok(())
}
