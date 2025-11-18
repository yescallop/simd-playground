use crate::naive::table_bitset;
use std::arch::x86_64::*;

#[rustc_align(64)]
#[target_feature(enable = "avx512bw")]
pub unsafe fn validate_3load(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 64 + 2 {
        if !super::validate_first_two(src) {
            return false;
        }

        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);
        let allowed = _mm256_broadcastsi128_si256(allowed);
        let allowed = _mm512_broadcast_i64x4(allowed);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);
        let hexdig = _mm256_broadcastsi128_si256(hexdig);
        let hexdig = _mm512_broadcast_i64x4(hexdig);

        let pct = _mm512_set1_epi8(b'%' as _);
        let byte_lo_4_mask = _mm512_set1_epi8(0xf);
        let mask_table = _mm512_set1_epi64(0x8040201008040201u64 as _);

        while i <= len - 64 - 2 {
            let chunk = _mm512_loadu_si512(ptr.add(i + 2).cast()); // <=8 0.5 1*p23

            // for non-ASCII, this is 0
            let mask_per_byte = _mm512_shuffle_epi8(mask_table, chunk); // 1 1 1*p5

            // loadu and cmpeq are combined into vpcmpeqb (k, zmm, m512)
            // unlike with AVX2, it is insignificant whether we put them here or below
            let chunk_l1 = _mm512_loadu_si512(ptr.add(i + 1).cast());
            let chunk_l2 = _mm512_loadu_si512(ptr.add(i).cast());

            let after_pct_1 = _mm512_cmpeq_epi8_mask(chunk_l1, pct); // 3 1 1*p5; with load: n/a
            let after_pct_2 = _mm512_cmpeq_epi8_mask(chunk_l2, pct); // 3 1 1*p5; with load: n/a
            let after_pct = after_pct_1 | after_pct_2; // korq: 1 1 1*p0

            let word_shr_3 = _mm512_srli_epi16::<3>(chunk); // 1 1 1*p0

            let table_idx_per_byte = _mm512_and_si512(word_shr_3, byte_lo_4_mask); // 1 0.5 1*p05

            let allowed_per_byte = _mm512_shuffle_epi8(allowed, table_idx_per_byte); // 1 1 1*p5
            let hexdig_per_byte = _mm512_shuffle_epi8(hexdig, table_idx_per_byte); // writemasked: 3 1 1*p5

            // this actually translates to writemasked shuffle
            let table_per_byte =
                _mm512_mask_blend_epi8(after_pct, allowed_per_byte, hexdig_per_byte);

            // this actually sets bit when AND is zero
            let is_invalid = _mm512_testn_epi8_mask(table_per_byte, mask_per_byte); // 3 1 1*p5

            // kortestq: 1 1 1*p0
            if is_invalid != 0 {
                return false;
            }
            i += 64;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}

#[rustc_align(64)]
#[target_feature(enable = "avx512bw,gfni")]
pub unsafe fn validate_3load_gf2p8affine(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 64 + 2 {
        if !super::validate_first_two(src) {
            return false;
        }

        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        let allowed = _mm_set_epi64x(allowed.1 as _, allowed.0 as _);
        let allowed = _mm256_broadcastsi128_si256(allowed);
        let allowed = _mm512_broadcast_i64x4(allowed);

        let hexdig = table_bitset::HEXDIG.bits();
        let hexdig = _mm_set_epi64x(hexdig.1 as _, hexdig.0 as _);
        let hexdig = _mm256_broadcastsi128_si256(hexdig);
        let hexdig = _mm512_broadcast_i64x4(hexdig);

        let pct = _mm512_set1_epi8(b'%' as _);
        let srl_3_matrix = _mm512_set1_epi64(0x0102040810204080 << 3);
        let mask_table = _mm512_set1_epi64(0x8040201008040201u64 as _);

        while i <= len - 64 - 2 {
            let chunk = _mm512_loadu_si512(ptr.add(i + 2).cast()); // <=8 0.5 1*p23

            // for non-ASCII, this is 0
            let mask_per_byte = _mm512_shuffle_epi8(mask_table, chunk); // 1 1 1*p5

            // loadu and cmpeq are combined into vpcmpeqb (k, zmm, m512)
            // unlike with AVX2, it is insignificant whether we put them here or below
            let chunk_l1 = _mm512_loadu_si512(ptr.add(i + 1).cast());
            let chunk_l2 = _mm512_loadu_si512(ptr.add(i).cast());

            let after_pct_1 = _mm512_cmpeq_epi8_mask(chunk_l1, pct); // 3 1 1*p5; with load: n/a
            let after_pct_2 = _mm512_cmpeq_epi8_mask(chunk_l2, pct); // 3 1 1*p5; with load: n/a
            let after_pct = after_pct_1 | after_pct_2; // korq: 1 1 1*p0

            let table_idx_per_byte = _mm512_gf2p8affine_epi64_epi8::<0>(chunk, srl_3_matrix); // 5 1 1*p0

            let allowed_per_byte = _mm512_shuffle_epi8(allowed, table_idx_per_byte); // 1 1 1*p5
            let hexdig_per_byte = _mm512_shuffle_epi8(hexdig, table_idx_per_byte); // writemasked: 3 1 1*p5

            // this actually translates to writemasked shuffle
            let table_per_byte =
                _mm512_mask_blend_epi8(after_pct, allowed_per_byte, hexdig_per_byte);

            // this actually sets bit when AND is zero
            let is_invalid = _mm512_testn_epi8_mask(table_per_byte, mask_per_byte); // 3 1 1*p5

            // kortestq: 1 1 1*p0
            if is_invalid != 0 {
                return false;
            }
            i += 64;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}

#[rustc_align(64)]
#[target_feature(enable = "avx512bw,avx512vbmi")]
pub unsafe fn validate_3load_perm(src: &[u8]) -> bool {
    let len = src.len();
    let ptr = src.as_ptr();

    let mut i = 0;
    if len >= 64 + 2 {
        if !super::validate_first_two(src) {
            return false;
        }

        let mut table_lo = _mm512_setzero_si512();
        let mut table_hi = _mm512_setzero_si512();
        let mask_hexdig = _mm512_set1_epi8(1);
        let mask_allowed = _mm512_set1_epi8(2);

        let hexdig = table_bitset::HEXDIG.bits();
        table_lo = _mm512_mask_add_epi8(table_lo, hexdig.0, table_lo, mask_hexdig);
        table_hi = _mm512_mask_add_epi8(table_hi, hexdig.1, table_hi, mask_hexdig);

        // the corresponding bit for % is set in this table
        let allowed = table_bitset::PATH.bits();
        table_lo = _mm512_mask_add_epi8(table_lo, allowed.0, table_lo, mask_allowed);
        table_hi = _mm512_mask_add_epi8(table_hi, allowed.1, table_hi, mask_allowed);

        let pct = _mm512_set1_epi8(b'%' as _);

        while i <= len - 64 - 2 {
            let chunk = _mm512_loadu_si512(ptr.add(i + 2).cast()); // <=8 0.5 1*p23

            // loadu and cmpeq are combined into vpcmpeqb (k, zmm, m512)
            let chunk_l1 = _mm512_loadu_si512(ptr.add(i + 1).cast());
            let chunk_l2 = _mm512_loadu_si512(ptr.add(i).cast());

            let after_pct_1 = _mm512_cmpeq_epi8_mask(chunk_l1, pct); // 3 1 1*p5; with load: n/a
            let after_pct_2 = _mm512_cmpeq_epi8_mask(chunk_l2, pct); // 3 1 1*p5; with load: n/a
            let after_pct = after_pct_1 | after_pct_2; // korq: 1 1 1*p0

            // cmpge_epi8 turns into vpmovb2m+knotq, while
            // movepi8_mask turns into vpcmpb, which is odd
            let is_ascii = !_mm512_movepi8_mask(chunk); // vpcmpb: 3 1 1*p5

            let table_per_byte =
                _mm512_maskz_permutex2var_epi8(is_ascii, table_lo, chunk, table_hi); // 6 2 1*p05+2*p5

            // this turns into vpblendmd (zmm, k, zmm, m512)
            // which benches faster than (.., zmm) I really don't know why
            let mask_per_byte = _mm512_mask_blend_epi8(after_pct, mask_allowed, mask_hexdig); // <=11 0.5 1*p05+1*p23

            // this actually sets bit when AND is zero
            let is_invalid = _mm512_testn_epi8_mask(table_per_byte, mask_per_byte); // 3 1 1*p5

            // kortestq: 1 1 1*p0
            if is_invalid != 0 {
                return false;
            }
            i += 64;
        }
    }
    table_bitset::PATH.validate(&src[i..])
}
