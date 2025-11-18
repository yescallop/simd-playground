use crate::naive::table_bitset;
use std::arch::x86_64::*;

#[rustc_align(64)]
#[target_feature(enable = "avx2")]
pub unsafe fn validate_3load(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 32 + 2 {
        if !super::validate_first_two(src) {
            return false;
        }

        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);
        let allowed = _mm256_broadcastsi128_si256(allowed);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);
        let hexdig = _mm256_broadcastsi128_si256(hexdig);

        let pct = _mm256_set1_epi8(b'%' as _);
        let byte_lo_4_mask = _mm256_set1_epi8(0xf);
        let mask_table = _mm256_set1_epi64x(0x8040201008040201u64 as _);
        let zero = _mm256_setzero_si256();

        while i + 32 + 2 <= len {
            let chunk = _mm256_loadu_si256(ptr.add(i + 2).cast()); // <=8 0.5 1*p23

            // for non-ASCII, this is 0
            let mask_per_byte = _mm256_shuffle_epi8(mask_table, chunk); // 1 0.5 1*p15

            let word_shr_3 = _mm256_srli_epi16::<3>(chunk); // 1 0.5 1*p01

            let table_idx_per_byte = _mm256_and_si256(word_shr_3, byte_lo_4_mask); // 1 0.33 1*p015

            let allowed_per_byte = _mm256_shuffle_epi8(allowed, table_idx_per_byte); // 1 0.5 1*p15
            let hexdig_per_byte = _mm256_shuffle_epi8(hexdig, table_idx_per_byte); // 1 0.5 1*p15

            // loadu and cmpeq are combined into vpcmpeqb (ymm, ymm, m256)
            // it's faster if we put them here than above
            let chunk_l1 = _mm256_loadu_si256(ptr.add(i + 1).cast());
            let chunk_l2 = _mm256_loadu_si256(ptr.add(i).cast());

            let after_pct_1 = _mm256_cmpeq_epi8(chunk_l1, pct); // with load: <=9 0.5 1*p01+1*p23
            let after_pct_2 = _mm256_cmpeq_epi8(chunk_l2, pct); // with load: <=9 0.5 1*p01+1*p23
            let after_pct = _mm256_or_si256(after_pct_1, after_pct_2); // 1 0.33 1*p015

            // unlike with SSE4.1, it is faster to blend the tables first
            let table_per_byte = _mm256_blendv_epi8(allowed_per_byte, hexdig_per_byte, after_pct); // 1 1 2*p015
            let nz_if_valid = _mm256_and_si256(table_per_byte, mask_per_byte); // 1 0.33 1*p015

            // unlike with SSE4.1, it isn't slower to compare with 0
            let is_invalid = _mm256_cmpeq_epi8(nz_if_valid, zero); // 1 0.5 1*p01
            let is_invalid = _mm256_movemask_epi8(is_invalid); // <=4 1 1*p0

            if is_invalid != 0 {
                return false;
            }
            i += 32;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}

#[rustc_align(64)]
#[target_feature(enable = "avx2")]
pub unsafe fn validate_alignr(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 64 {
        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);
        let allowed = _mm256_broadcastsi128_si256(allowed);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);
        let hexdig = _mm256_broadcastsi128_si256(hexdig);

        let pct = _mm256_set1_epi8(b'%' as _);
        let low_nibble_mask = _mm256_set1_epi8(0xf);
        let mask_table = _mm256_set1_epi64x(0x8040201008040201u64 as _);
        let zero = _mm256_setzero_si256();

        let mut chunk = _mm256_loadu_si256(ptr.cast());

        // for non-ASCII, this is 0
        let mut mask_per_byte = _mm256_shuffle_epi8(mask_table, chunk);
        let word_shr_3 = _mm256_srli_epi16::<3>(chunk);
        let mut table_idx_per_byte = _mm256_and_si256(word_shr_3, low_nibble_mask);
        let hexdig_per_byte = _mm256_shuffle_epi8(hexdig, table_idx_per_byte);
        let mut nz_if_hexdig = _mm256_and_si256(hexdig_per_byte, mask_per_byte);

        while i + 64 <= len {
            let next_chunk = _mm256_loadu_si256(ptr.add(i + 32).cast()); // <=8 0.5 1*p23

            let is_pct = _mm256_cmpeq_epi8(chunk, pct); // 1 0.5 1*p01

            let allowed_per_byte = _mm256_shuffle_epi8(allowed, table_idx_per_byte); // 1 0.5 1*p15
            let nz_if_allowed = _mm256_and_si256(allowed_per_byte, mask_per_byte); // 1 0.33 1*p015

            mask_per_byte = _mm256_shuffle_epi8(mask_table, next_chunk); // 1 0.5 1*p15
            let word_shr_3 = _mm256_srli_epi16::<3>(next_chunk); // 1 0.5 1*p01
            table_idx_per_byte = _mm256_and_si256(word_shr_3, low_nibble_mask); // 1 0.33 1*p015
            let hexdig_per_byte = _mm256_shuffle_epi8(hexdig, table_idx_per_byte); // 1 0.5 1*p15
            let next_nz_if_hexdig = _mm256_and_si256(hexdig_per_byte, mask_per_byte); // 1 0.33 1*p015

            let nz_if_hexdig_16 =
                _mm256_permute2x128_si256::<0x21>(nz_if_hexdig, next_nz_if_hexdig);
            let nz_if_hexdig_1 = _mm256_alignr_epi8::<1>(nz_if_hexdig_16, nz_if_hexdig); // 1 1 1*p5
            let nz_if_hexdig_2 = _mm256_alignr_epi8::<2>(nz_if_hexdig_16, nz_if_hexdig); // 1 1 1*p5
            let nz_if_hexdig_1_and_2 = _mm256_min_epu8(nz_if_hexdig_1, nz_if_hexdig_2); // 1 0.5 1*p01

            let nz_if_valid = _mm256_blendv_epi8(nz_if_allowed, nz_if_hexdig_1_and_2, is_pct); // 1 0.67 2*p015

            let is_invalid = _mm256_cmpeq_epi8(nz_if_valid, zero); // 1 0.5 1*p01
            let is_invalid = _mm256_movemask_epi8(is_invalid); // <=4 1 1*p0

            if is_invalid != 0 {
                return false;
            }

            chunk = next_chunk;
            nz_if_hexdig = next_nz_if_hexdig;
            i += 32;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}
