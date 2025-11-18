#![feature(fn_align)]

pub mod avx2;
pub mod avx512;
pub mod naive;
pub mod sse41;
pub mod ssse3;

use crate::naive::table_bitset;

#[inline(always)]
fn validate_first_two(src: &[u8]) -> bool {
    let (a, b) = (src[0], src[1]);
    if a == b'%' {
        table_bitset::HEXDIG.allows_ascii(b)
    } else {
        table_bitset::PATH.allows_ascii_with_pct(a) || table_bitset::PATH.allows_ascii_with_pct(b)
    }
}
