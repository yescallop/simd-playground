use std::{fs, io};

use aoc_simd::day_06::*;

fn main() -> io::Result<()> {
    let input = fs::read_to_string("input/06.txt")?;
    let ans1 = unsafe { part1_avx512(input.as_bytes()) };
    let ans2 = unsafe { part2_avx512(input.as_bytes()) };
    assert_eq!((ans1, ans2), (Some(1794), Some(2851)));
    println!("{ans1:?}\n{ans2:?}");
    Ok(())
}
