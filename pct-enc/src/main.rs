use std::fs;

use pct_enc::*;

fn main() {
    let src = fs::read("enc.txt").unwrap();

    unsafe {
        println!("{}", validate_sse41_triple_loadu(&src));
    }
}
