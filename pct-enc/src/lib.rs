use std::arch::x86_64::*;

use bytemuck::bytes_of;

use crate::naive::table_bitset;

pub mod naive;

#[target_feature(enable = "ssse3")]
pub unsafe fn validate_ssse3(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    // the corresponding bit for % is set in this table
    let allowed = table_bitset::PATH.as_u64s();
    let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

    let hexdig = table_bitset::HEXDIG.as_u64s();
    let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

    let pct = _mm_set1_epi8(b'%' as _);
    let packed_0xf = _mm_set1_epi8(0xf);
    let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);
    let zero = _mm_setzero_si128();

    let is_not_hexdig = |chunk: __m128i| {
        let word_shr_3 = _mm_srli_epi16::<3>(chunk); // 1 0.5
        let table_idx_per_byte = _mm_and_si128(word_shr_3, packed_0xf); // 1 0.33
        let table_per_byte = _mm_shuffle_epi8(hexdig, table_idx_per_byte); // 1 0.5
        // if non-ASCII mask will be 0, which disallows the byte
        let mask_per_byte = _mm_shuffle_epi8(mask_table, chunk); // 1 0.5

        let zero_if_not_hexdig = _mm_and_si128(table_per_byte, mask_per_byte); // 1 0.33
        _mm_cmpeq_epi8(zero_if_not_hexdig, zero) // 1 0.5
    };

    let mut i = 0;
    while i + 16 + 2 <= len {
        let chunk = _mm_loadu_si128(ptr.add(i).cast()); // 4 0.5
        let off_by_1 = _mm_loadu_si128(ptr.add(i + 1).cast()); // 4 0.5
        let off_by_2 = _mm_loadu_si128(ptr.add(i + 2).cast()); // 4 0.5

        let is_pct = _mm_cmpeq_epi8(chunk, pct); // 1 0.5
        let not_hexdig_1 = is_not_hexdig(off_by_1);
        let not_hexdig_2 = is_not_hexdig(off_by_2);

        let not_hexdig_1_or_2 = _mm_or_si128(not_hexdig_1, not_hexdig_2); // 1 0.33
        let is_invalid_pct = _mm_and_si128(is_pct, not_hexdig_1_or_2); // 1 0.33

        let word_shr_3 = _mm_srli_epi16::<3>(chunk); // 1 0.5
        let table_idx_per_byte = _mm_and_si128(word_shr_3, packed_0xf); // 1 0.33
        let table_per_byte = _mm_shuffle_epi8(allowed, table_idx_per_byte); // 1 0.5
        // if non-ASCII mask will be 0, which disallows the byte
        let mask_per_byte = _mm_shuffle_epi8(mask_table, chunk); // 1 0.5

        let zero_if_disallowed = _mm_and_si128(table_per_byte, mask_per_byte); // 1 0.33
        let is_disallowed = _mm_cmpeq_epi8(zero_if_disallowed, zero); // 1 0.5

        let is_invalid = _mm_or_si128(is_disallowed, is_invalid_pct); // 1 0.33
        let is_invalid = _mm_movemask_epi8(is_invalid); // 3 1

        if is_invalid != 0 {
            return false;
        }
        i += 16;
    }

    i == len || table_bitset::PATH.validate(&src[i..])
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
