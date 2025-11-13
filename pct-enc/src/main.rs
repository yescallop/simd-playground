use std::fs;

use pct_enc::*;

fn main() {
    let src = fs::read("enc.txt").unwrap();

    unsafe {
        println!("{}", sse41::validate_alignr(&src));
    }
}
