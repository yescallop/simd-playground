use crate::naive::table_bitset;
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
pub unsafe fn validate_alignr(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 64 {
        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.as_u64s();
        let disallowed = _mm_set_epi64x(!allowed.1 as _, !allowed.0 as _);
        let disallowed = _mm256_broadcastsi128_si256(disallowed);

        let hexdig = table_bitset::HEXDIG.as_u64s();
        let not_hexdig = _mm_set_epi64x(!hexdig.1 as _, !hexdig.0 as _);
        let not_hexdig = _mm256_broadcastsi128_si256(not_hexdig);

        let pct = _mm256_set1_epi8(b'%' as _);
        let low_nibble_mask = _mm256_set1_epi8(0xf);
        let mask_table = _mm256_set1_epi64x(0x8040201008040201u64 as _);

        let mut chunk = _mm256_loadu_si256(ptr.cast());

        // if non-ASCII, mask will be 0, which disallows the byte
        let mut mask_per_byte = _mm256_shuffle_epi8(mask_table, chunk);
        let word_shr_3 = _mm256_srli_epi16::<3>(chunk);
        let mut table_idx_per_byte = _mm256_and_si256(word_shr_3, low_nibble_mask);
        let not_hexdig_per_byte = _mm256_shuffle_epi8(not_hexdig, table_idx_per_byte);
        let mut nz_if_not_hexdig = _mm256_and_si256(not_hexdig_per_byte, mask_per_byte);

        while i + 64 <= len {
            let next_chunk = _mm256_loadu_si256(ptr.add(i + 32).cast()); // 4 0.5 1*p23

            let is_pct = _mm256_cmpeq_epi8(chunk, pct); // 1 0.5 1*p01

            let disallowed_per_byte = _mm256_shuffle_epi8(disallowed, table_idx_per_byte); // 1 0.5 1*p15
            let nz_if_disallowed = _mm256_and_si256(disallowed_per_byte, mask_per_byte); // 1 0.33 1*p015

            mask_per_byte = _mm256_shuffle_epi8(mask_table, next_chunk); // 1 0.5 1*p15
            let word_shr_3 = _mm256_srli_epi16::<3>(next_chunk); // 1 0.5 1*p01
            table_idx_per_byte = _mm256_and_si256(word_shr_3, low_nibble_mask); // 1 0.33 1*p015
            let not_hexdig_per_byte = _mm256_shuffle_epi8(not_hexdig, table_idx_per_byte); // 1 0.5 1*p15
            let next_nz_if_not_hexdig = _mm256_and_si256(not_hexdig_per_byte, mask_per_byte); // 1 0.33 1*p015

            let nz_if_not_hexdig_16 =
                _mm256_permute2x128_si256::<0x21>(nz_if_not_hexdig, next_nz_if_not_hexdig);
            let nz_if_not_hexdig_1 = _mm256_alignr_epi8::<1>(nz_if_not_hexdig_16, nz_if_not_hexdig); // 1 1 1*p5
            let nz_if_not_hexdig_2 = _mm256_alignr_epi8::<2>(nz_if_not_hexdig_16, nz_if_not_hexdig); // 1 1 1*p5
            let nz_if_not_hexdig_1_or_2 = _mm256_or_si256(nz_if_not_hexdig_1, nz_if_not_hexdig_2); // 1 0.33 1*p015

            let nz_if_invalid_pct = _mm256_min_epu8(is_pct, nz_if_not_hexdig_1_or_2); // 1 0.5 1*p01

            let nz_if_invalid = _mm256_or_si256(nz_if_disallowed, nz_if_invalid_pct); // 1 0.33 1*p015
            let is_valid = _mm256_testz_si256(nz_if_invalid, nz_if_invalid); // 4 1 1*p0+1*p5

            if is_valid == 0 {
                return false;
            }

            chunk = next_chunk;
            nz_if_not_hexdig = next_nz_if_not_hexdig;
            i += 32;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}
