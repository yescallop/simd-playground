pub mod day_02;
pub mod day_04;
pub mod day_06;

use std::fmt::Debug;

use bytemuck::{AnyBitPattern, NoUninit};

#[allow(unused)]
fn print_bytes_of<T: NoUninit>(t: &T) {
    let bytes = bytemuck::bytes_of(t);
    print_bytes(bytes);
}

#[allow(unused)]
fn print_as_slice<B: AnyBitPattern + Debug>(a: &impl NoUninit) {
    let bytes = bytemuck::bytes_of(a);
    let slice: &[B] = bytemuck::cast_slice(bytes);
    println!("{slice:?}");
}

#[allow(unused)]
fn print_bytes(bytes: &[u8]) {
    for &(mut b) in bytes {
        if b == b'\n' {
            b = b'\\';
        } else if b == 0 {
            b = b'/';
        }
        print!("{}", b as char)
    }
    println!();
}
