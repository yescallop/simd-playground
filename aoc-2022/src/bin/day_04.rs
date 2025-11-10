use std::{fs, io};

fn main() -> io::Result<()> {
    let input = fs::read_to_string("input/04.txt")?;
    let ans1 = aoc_simd::day_04::part1_avx512(input.as_bytes());
    assert_eq!(ans1, 547);
    println!("{ans1}");
    Ok(())
}
