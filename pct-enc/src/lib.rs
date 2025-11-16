#![feature(fn_align)]

pub mod avx2;
pub mod naive;
pub mod sse41;

use crate::naive::table_bitset;

#[inline(always)]
fn validate_first_two(src: &[u8]) -> bool {
    let (a, b) = (src[0], src[1]);
    if a == b'%' {
        if !table_bitset::HEXDIG.allows_ascii(b) {
            return false;
        }
    } else if b == b'%' {
        if !table_bitset::PATH.allows_ascii(a) {
            return false;
        }
    } else if !table_bitset::PATH.allows_ascii(a) || !table_bitset::PATH.allows_ascii(b) {
        return false;
    }
    true
}
