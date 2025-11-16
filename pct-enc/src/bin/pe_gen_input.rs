use std::fs;

use pct_enc::naive::{Encode, table_bitset::PATH};
use rand::Rng;

fn main() {
    let size = 1024 * 1024;

    let mut rng = rand::rng();

    let mut raw = Vec::with_capacity(size);
    for _ in 0..size {
        raw.push(rng.random::<u8>());
    }

    let mut enc = Vec::with_capacity(size * 3);
    for chunk in Encode::new(PATH, &raw) {
        enc.extend_from_slice(chunk.as_bytes());
    }

    fs::write("raw.bin", raw).unwrap();
    fs::write("enc.txt", enc).unwrap();
}
