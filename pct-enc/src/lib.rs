use std::arch::x86_64::*;

use bytemuck::bytes_of;

use crate::naive::table_bitset;

pub mod naive;

#[target_feature(enable = "sse3")]
pub unsafe fn validate_ssse3_triple_loadu(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    // the corresponding bit for % is set in this table
    let allowed = table_bitset::PATH.as_u64s();
    let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

    let hexdig = table_bitset::HEXDIG.as_u64s();
    let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

    let pct = _mm_set1_epi8(b'%' as _);
    let low_nibble_mask = _mm_set1_epi8(0xf);
    let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);
    let zero = _mm_setzero_si128();

    let mut i = 0;
    while i + 16 + 2 <= len {
        let bytes_0 = _mm_loadu_si128(ptr.add(i).cast()); // 4 0.5 1*p23
        let bytes_1 = _mm_loadu_si128(ptr.add(i + 1).cast()); // 4 0.5 1*p23
        let bytes_2 = _mm_loadu_si128(ptr.add(i + 2).cast()); // 4 0.5 1*p23

        // if non-ASCII mask will be 0, which disallows the byte
        let mask_per_byte_0 = _mm_shuffle_epi8(mask_table, bytes_0); // 1 0.5 1*p15
        let mask_per_byte_1 = _mm_shuffle_epi8(mask_table, bytes_1); // 1 0.5 1*p15
        let mask_per_byte_2 = _mm_shuffle_epi8(mask_table, bytes_2); // 1 0.5 1*p15

        let word_shr_3_0 = _mm_srli_epi16::<3>(bytes_0); // 1 0.5 1*p01
        let word_shr_3_1 = _mm_srli_epi16::<3>(bytes_1); // 1 0.5 1*p01
        let word_shr_3_2 = _mm_srli_epi16::<3>(bytes_2); // 1 0.5 1*p01

        let table_idx_per_byte_0 = _mm_and_si128(word_shr_3_0, low_nibble_mask); // 1 0.33 1*p015
        let table_idx_per_byte_1 = _mm_and_si128(word_shr_3_1, low_nibble_mask); // 1 0.33 1*p015
        let table_idx_per_byte_2 = _mm_and_si128(word_shr_3_2, low_nibble_mask); // 1 0.33 1*p015

        let table_per_byte_0 = _mm_shuffle_epi8(allowed, table_idx_per_byte_0); // 1 0.5 1*p15
        let table_per_byte_1 = _mm_shuffle_epi8(hexdig, table_idx_per_byte_1); // 1 0.5 1*p15
        let table_per_byte_2 = _mm_shuffle_epi8(hexdig, table_idx_per_byte_2); // 1 0.5 1*p15

        let zero_if_disallowed = _mm_and_si128(table_per_byte_0, mask_per_byte_0); // 1 0.33 1*p015
        let zero_if_not_hexdig_1 = _mm_and_si128(table_per_byte_1, mask_per_byte_1); // 1 0.33 1*p015
        let zero_if_not_hexdig_2 = _mm_and_si128(table_per_byte_2, mask_per_byte_2); // 1 0.33 1*p015

        let is_pct = _mm_cmpeq_epi8(bytes_0, pct); // 1 0.5 1*p01

        let is_disallowed = _mm_cmpeq_epi8(zero_if_disallowed, zero); // 1 0.5 1*p01
        let is_not_hexdig_1 = _mm_cmpeq_epi8(zero_if_not_hexdig_1, zero); // 1 0.5 1*p01
        let is_not_hexdig_2 = _mm_cmpeq_epi8(zero_if_not_hexdig_2, zero); // 1 0.5 1*p01

        let is_not_hexdig_1_or_2 = _mm_or_si128(is_not_hexdig_1, is_not_hexdig_2); // 1 0.33 1*p015
        let is_invalid_pct = _mm_and_si128(is_pct, is_not_hexdig_1_or_2); // 1 0.33 1*p015

        let is_invalid = _mm_or_si128(is_disallowed, is_invalid_pct); // 1 0.33 1*p015
        let is_invalid = _mm_movemask_epi8(is_invalid); // 3 1 1*p0
        // let is_valid = _mm_testz_si128(is_invalid, is_invalid); // 4 1 1*p0+1*p5

        if is_invalid != 0 {
            return false;
        }
        i += 16;
    }

    table_bitset::PATH.validate(&src[i..])
}

#[allow(unused)]
fn print_bytes(x: __m128i) {
    println!("{:?}", bytes_of(&x));
}

#[allow(unused)]
unsafe fn print_str(x: __m128i) {
    println!("{}", str::from_utf8_unchecked(bytes_of(&x)));
}

#[allow(unused)]
unsafe fn print_mask(x: __m128i) {
    let mask = _mm_movemask_epi8(x);
    println!("{:016b}", (mask as u16).reverse_bits());
}
