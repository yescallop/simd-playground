use crate::naive::table_bitset;
use std::arch::x86_64::*;

#[rustc_align(64)]
#[target_feature(enable = "ssse3")]
pub unsafe fn validate_3load(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 16 + 2 {
        if !super::validate_first_two(src) {
            return false;
        }

        // the corresponding bits for % and hexdig are set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

        let pct = _mm_set1_epi8(b'%' as _);
        let byte_lo_4_mask = _mm_set1_epi8(0xf);
        let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);
        let zero = _mm_setzero_si128();

        i = 2;
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(ptr.add(i).cast()); // <=7 0.5 1*p23
            let chunk_minus_1 = _mm_loadu_si128(ptr.add(i - 1).cast()); // <=7 0.5 1*p23
            let chunk_minus_2 = _mm_loadu_si128(ptr.add(i - 2).cast()); // <=7 0.5 1*p23

            // for non-ASCII, this is 0
            let mask_per_byte = _mm_shuffle_epi8(mask_table, chunk); // 1 0.5 1*p15

            let after_pct_1 = _mm_cmpeq_epi8(chunk_minus_1, pct); // 1 0.5 1*p01
            let after_pct_2 = _mm_cmpeq_epi8(chunk_minus_2, pct); // 1 0.5 1*p01
            let after_pct = _mm_or_si128(after_pct_1, after_pct_2); // 1 0.33 1*p015

            let word_shr_3 = _mm_srli_epi16::<3>(chunk); // 1 0.5 1*p01

            let table_idx_per_byte = _mm_and_si128(word_shr_3, byte_lo_4_mask); // 1 0.33 1*p015

            let allowed_per_byte = _mm_shuffle_epi8(allowed, table_idx_per_byte); // 1 0.5 1*p15
            let hexdig_per_byte = _mm_shuffle_epi8(hexdig, table_idx_per_byte); // 1 0.5 1*p15

            let table_per_byte = _mm_andnot_si128(after_pct, allowed_per_byte); // 1 0.33 1*p015
            let table_per_byte = _mm_or_si128(table_per_byte, hexdig_per_byte); // 1 0.33 1*p015

            let nz_if_valid = _mm_and_si128(table_per_byte, mask_per_byte); // 1 0.33 1*p015

            let is_invalid = _mm_cmpeq_epi8(nz_if_valid, zero); // 1 0.5 1*p01
            let is_invalid = _mm_movemask_epi8(is_invalid); // 3 1 1*p0

            if is_invalid != 0 {
                return false;
            }
            i += 16;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}
