#![feature(stdsimd)]

use std::{fs, io};

fn main() -> io::Result<()> {
    let input = fs::read_to_string("input/02.txt")?;
    let ans = aoc_simd::day_02::solve_avx512(input.as_bytes());
    assert_eq!(ans, (11906, 11186));
    println!("{ans:?}");
    Ok(())
}
