use crate::naive::table_bitset;
use std::arch::x86_64::*;

#[rustc_align(64)]
#[target_feature(enable = "sse4.1")]
pub unsafe fn validate_3load(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 16 + 2 {
        if !super::validate_first_two(src) {
            return false;
        }

        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

        let pct = _mm_set1_epi8(b'%' as _);
        let byte_lo_4_mask = _mm_set1_epi8(0xf);
        let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);

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

            // in theory it can be faster to blend the tables first,
            // but that is slower in practice
            //
            // for non-ASCII, these are 0
            let mask_disallowed = _mm_andnot_si128(allowed_per_byte, mask_per_byte); // 1 0.33 1*p015
            let mask_not_hexdig = _mm_andnot_si128(hexdig_per_byte, mask_per_byte); // 1 0.33 1*p015

            let mask_invalid = _mm_blendv_epi8(mask_disallowed, mask_not_hexdig, after_pct); // 1 0.33 1*p015

            // in theory it can be faster to compare with 0 instead,
            // but that is also slower in practice, possibly due to
            // the use of more REX prefixes from allocating xmm6 for 0
            // and shifting some temporaries to xmm8 and above
            //
            // for non-ASCII, both operands are 0
            let is_invalid = _mm_cmpeq_epi8(mask_invalid, mask_per_byte); // 1 0.5 1*p01
            let is_invalid = _mm_movemask_epi8(is_invalid); // 3 1 1*p0

            if is_invalid != 0 {
                return false;
            }
            i += 16;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}

#[rustc_align(64)]
#[target_feature(enable = "sse4.1")]
pub unsafe fn validate_alignr(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 32 {
        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

        let pct = _mm_set1_epi8(b'%' as _);
        let byte_lo_4_mask = _mm_set1_epi8(0xf);
        let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);
        let zero = _mm_setzero_si128();

        let mut chunk = _mm_loadu_si128(ptr.cast());

        // for non-ASCII, this is 0
        let mut mask_per_byte = _mm_shuffle_epi8(mask_table, chunk);
        let word_shr_3 = _mm_srli_epi16::<3>(chunk);
        let mut table_idx_per_byte = _mm_and_si128(word_shr_3, byte_lo_4_mask);
        let hexdig_per_byte = _mm_shuffle_epi8(hexdig, table_idx_per_byte);
        let mut nz_if_hexdig = _mm_and_si128(hexdig_per_byte, mask_per_byte);

        while i + 32 <= len {
            let next_chunk = _mm_loadu_si128(ptr.add(i + 16).cast()); // <=7 0.5 1*p23

            let is_pct = _mm_cmpeq_epi8(chunk, pct); // 1 0.5 1*p01

            let allowed_per_byte = _mm_shuffle_epi8(allowed, table_idx_per_byte); // 1 0.5 1*p15
            let nz_if_allowed = _mm_and_si128(allowed_per_byte, mask_per_byte); // 1 0.33 1*p015

            mask_per_byte = _mm_shuffle_epi8(mask_table, next_chunk); // 1 0.5 1*p15
            let word_shr_3 = _mm_srli_epi16::<3>(next_chunk); // 1 0.5 1*p01
            table_idx_per_byte = _mm_and_si128(word_shr_3, byte_lo_4_mask); // 1 0.33 1*p015
            let hexdig_per_byte = _mm_shuffle_epi8(hexdig, table_idx_per_byte); // 1 0.5 1*p15
            let next_nz_if_hexdig = _mm_and_si128(hexdig_per_byte, mask_per_byte); // 1 0.33 1*p015

            let nz_if_hexdig_1 = _mm_alignr_epi8::<1>(next_nz_if_hexdig, nz_if_hexdig); // 1 1 1*p5
            let nz_if_hexdig_2 = _mm_alignr_epi8::<2>(next_nz_if_hexdig, nz_if_hexdig); // 1 1 1*p5
            let nz_if_hexdig_1_and_2 = _mm_min_epu8(nz_if_hexdig_1, nz_if_hexdig_2); // 1 0.5 1*p01

            let nz_if_valid = _mm_blendv_epi8(nz_if_allowed, nz_if_hexdig_1_and_2, is_pct); // 1 0.33 1*p015

            let is_invalid = _mm_cmpeq_epi8(nz_if_valid, zero); // 1 0.5 1*p01
            let is_invalid = _mm_movemask_epi8(is_invalid); // 3 1 1*p0

            if is_invalid != 0 {
                return false;
            }

            chunk = next_chunk;
            nz_if_hexdig = next_nz_if_hexdig;
            i += 16;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}

#[rustc_align(64)]
#[target_feature(enable = "sse4.1")]
pub unsafe fn validate_shift(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    // the corresponding bit for % is set in this table
    let allowed = table_bitset::PATH.bits();
    let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

    let hexdig = table_bitset::HEXDIG.bits();
    let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

    let pct = _mm_set1_epi8(b'%' as _);
    let byte_lo_4_mask = _mm_set1_epi8(0xf);
    let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);

    let mut is_pct_prev = _mm_setzero_si128();

    let mut i = 0;
    while i + 16 <= len {
        let chunk = _mm_loadu_si128(ptr.add(i).cast()); // <=7 0.5 1*p23

        // for non-ASCII, this is 0
        let mask_per_byte = _mm_shuffle_epi8(mask_table, chunk); // 1 0.5 1*p15

        let is_pct = _mm_cmpeq_epi8(chunk, pct); // 1 0.5 1*p01

        let after_pct_1 = _mm_bslli_si128::<1>(is_pct); // 1 0.5 1*p15
        let after_pct_2 = _mm_bslli_si128::<2>(is_pct); // 1 0.5 1*p15
        let after_pct_1_prev = _mm_bsrli_si128::<15>(is_pct_prev); // 1 0.5 1*p15
        let after_pct_2_prev = _mm_bsrli_si128::<14>(is_pct_prev); // 1 0.5 1*p15

        let mut after_pct = _mm_or_si128(after_pct_1, after_pct_2); // 1 0.33 1*p015
        after_pct = _mm_or_si128(after_pct, after_pct_1_prev); // 1 0.33 1*p015
        after_pct = _mm_or_si128(after_pct, after_pct_2_prev); // 1 0.33 1*p015

        is_pct_prev = is_pct;

        let word_shr_3 = _mm_srli_epi16::<3>(chunk); // 1 0.5 1*p01

        let table_idx_per_byte = _mm_and_si128(word_shr_3, byte_lo_4_mask); // 1 0.33 1*p015

        let allowed_per_byte = _mm_shuffle_epi8(allowed, table_idx_per_byte); // 1 0.5 1*p15
        let hexdig_per_byte = _mm_shuffle_epi8(hexdig, table_idx_per_byte); // 1 0.5 1*p15

        // for non-ASCII, these are 0
        let mask_disallowed = _mm_andnot_si128(allowed_per_byte, mask_per_byte); // 1 0.33 1*p015
        let mask_not_hexdig = _mm_andnot_si128(hexdig_per_byte, mask_per_byte); // 1 0.33 1*p015

        let mask_invalid = _mm_blendv_epi8(mask_disallowed, mask_not_hexdig, after_pct); // 1 0.33 1*p015

        let is_invalid = _mm_cmpeq_epi8(mask_invalid, mask_per_byte); // 1 0.5 1*p01
        let is_invalid = _mm_movemask_epi8(is_invalid); // 3 1 1*p0

        if is_invalid != 0 {
            return false;
        }
        i += 16;
    }
    table_bitset::PATH.validate(&src[i..])
}

#[rustc_align(64)]
#[target_feature(enable = "sse4.1")]
pub unsafe fn validate_shift_transposed(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    // the corresponding bit for % is set in this table
    let allowed = table_bitset::PATH.bits_transposed();
    let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);

    let hexdig = table_bitset::HEXDIG.bits_transposed();
    let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);

    let pct = _mm_set1_epi8(b'%' as _);
    let byte_lo_3_mask = _mm_set1_epi8(0b111);
    let mask_table = _mm_set1_epi64x(0x8040201008040201u64 as _);

    let mut is_pct_prev = _mm_setzero_si128();

    let mut i = 0;
    while i + 16 <= len {
        let chunk = _mm_loadu_si128(ptr.add(i).cast()); // <=7 0.5 1*p23

        // for non-ASCII, these are 0
        let allowed_per_byte = _mm_shuffle_epi8(allowed, chunk); // 1 0.5 1*p15
        let hexdig_per_byte = _mm_shuffle_epi8(hexdig, chunk); // 1 0.5 1*p15

        let is_pct = _mm_cmpeq_epi8(chunk, pct); // 1 0.5 1*p01

        let after_pct_1 = _mm_bslli_si128::<1>(is_pct); // 1 0.5 1*p15
        let after_pct_2 = _mm_bslli_si128::<2>(is_pct); // 1 0.5 1*p15
        let after_pct_1_prev = _mm_bsrli_si128::<15>(is_pct_prev); // 1 0.5 1*p15
        let after_pct_2_prev = _mm_bsrli_si128::<14>(is_pct_prev); // 1 0.5 1*p15

        let mut after_pct = _mm_or_si128(after_pct_1, after_pct_2); // 1 0.33 1*p015
        after_pct = _mm_or_si128(after_pct, after_pct_1_prev); // 1 0.33 1*p015
        after_pct = _mm_or_si128(after_pct, after_pct_2_prev); // 1 0.33 1*p015

        is_pct_prev = is_pct;

        let word_shr_4 = _mm_srli_epi16::<4>(chunk); // 1 0.5 1*p01

        let mask_idx_per_byte = _mm_and_si128(word_shr_4, byte_lo_3_mask); // 1 0.33 1*p015
        let mask_per_byte = _mm_shuffle_epi8(mask_table, mask_idx_per_byte); // 1 0.5 1*p15

        // nz: nonzero. for non-ASCII, these are nonzero
        let nz_if_disallowed = _mm_andnot_si128(allowed_per_byte, mask_per_byte); // 1 0.33 1*p015
        let nz_if_not_hexdig = _mm_andnot_si128(hexdig_per_byte, mask_per_byte); // 1 0.33 1*p015

        let nz_if_invalid = _mm_blendv_epi8(nz_if_disallowed, nz_if_not_hexdig, after_pct); // 1 0.33 1*p015

        let is_valid = _mm_testz_si128(nz_if_invalid, nz_if_invalid); // 4 1 1*p0+1*p5

        if is_valid == 0 {
            return false;
        }
        i += 16;
    }
    table_bitset::PATH.validate(&src[i..])
}
