use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use rand::{Rng, distr::slice::Choose};

fn main() -> io::Result<()> {
    let nucleotides = Choose::new(b"ATCG").unwrap();
    let mut bw_txt = BufWriter::new(File::create("nucleotides.txt")?);
    let mut bw_bin = BufWriter::new(File::create("nucleotides.bin")?);
    let mut rng = rand::rng();
    for _ in 0..1024 * 1024 / 4 {
        let a = *rng.sample(nucleotides);
        let b = *rng.sample(nucleotides);
        let c = *rng.sample(nucleotides);
        let d = *rng.sample(nucleotides);
        bw_txt.write_all(&[a, b, c, d])?;

        let a = (a >> 1) & 3;
        let b = (b >> 1) & 3;
        let c = (c >> 1) & 3;
        let d = (d >> 1) & 3;
        bw_bin.write_all(&[a | b << 2 | c << 4 | d << 6])?;
    }
    bw_bin.write_all(&[0])?;

    Ok(())
}
