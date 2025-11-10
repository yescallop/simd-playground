use std::fs;

use pct_enc::naive::{Encode, PATH_TABLE};
use rand::Rng;

fn main() {
    let mut rng = rand::rng();

    let mut raw = Vec::with_capacity(1024 * 1024);
    for _ in 0..1024 * 1024 {
        raw.push(rng.random::<u8>());
    }

    let mut enc = Vec::with_capacity(1024 * 1024 * 3);
    for chunk in Encode::new(PATH_TABLE, &raw) {
        enc.extend_from_slice(chunk.as_bytes());
    }

    fs::write("raw.bin", raw).unwrap();
    fs::write("enc.txt", enc).unwrap();
}
