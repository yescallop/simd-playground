#![feature(test)]

extern crate test;

use std::{fs, io};

use aoc_simd::day_04::*;
use test::Bencher;

#[bench]
fn d4p1_avx512(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/04.txt")?;
    b.bytes = input.len() as u64;
    b.iter(|| part1_avx512(input.as_bytes()));
    Ok(())
}

#[bench]
fn d4p1_naive(b: &mut Bencher) -> io::Result<()> {
    let input = fs::read_to_string("input/04.txt")?;
    b.iter(|| part1_naive(&input));
    Ok(())
}
