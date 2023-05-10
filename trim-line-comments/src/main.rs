#![feature(stdsimd)]
#![feature(bigint_helper_methods)]

use std::{arch::x86_64::*, fs, io};

fn main() -> io::Result<()> {
    let input = fs::read_to_string("trim-line-comments/input.toml")?;
    let mut output = String::new();

    unsafe { trim(&input, &mut output) }

    println!("{output}");
    Ok(())
}

unsafe fn trim(input: &str, output: &mut String) {
    let input = input.as_bytes();
    let output = output.as_mut_vec();

    output.reserve(input.len());

    let ptr = input.as_ptr();
    let buf = output.as_mut_ptr();

    let packed_hashes = _mm512_set1_epi8(b'#' as i8);
    let packed_lfs = _mm512_set1_epi8(b'\n' as i8);

    let mut i = 0;
    let mut diff = 0;
    let mut borrow = 0;
    let mut buf_len = 0;

    while i + 64 <= input.len() {
        let text = _mm512_loadu_si512(ptr.add(i).cast());

        let hash = _mm512_cmpeq_epi8_mask(text, packed_hashes);
        let lf = _mm512_cmpeq_epi8_mask(text, packed_lfs);

        borrow = _subborrow_u64(borrow, lf, hash, &mut diff);

        let mask = !(diff | hash) | lf;

        _mm512_mask_compressstoreu_epi8(buf.add(buf_len), mask, text);
        buf_len += mask.count_ones() as usize;

        i += 64;
    }

    output.set_len(buf_len);

    let mut trimmed = borrow != 0;
    let mut untrimmed_start = i;

    while i < input.len() {
        let byte = input[i];
        if byte == b'#' && !trimmed {
            output.extend_from_slice(&input[untrimmed_start..i]);
            trimmed = true;
        } else if byte == b'\n' && trimmed {
            untrimmed_start = i;
            trimmed = false;
        }
        i += 1;
    }

    if !trimmed {
        output.extend_from_slice(&input[untrimmed_start..]);
    }
}
