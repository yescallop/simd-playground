use std::arch::x86_64::*;

pub fn solve_avx512(input: &[u8]) -> (u32, u32) {
    unsafe { _solve_avx512(input) }
}

#[target_feature(enable = "avx512bw,avx512vl")]
unsafe fn _solve_avx512(input: &[u8]) -> (u32, u32) {
    let len = input.len();
    let ptr = input.as_ptr();

    const LUT1: [u8; 16] = [0, 4, 1, 7, 0, 8, 5, 2, 0, 3, 9, 6, 0, 0, 0, 0];
    const LUT2: [u8; 16] = [0, 3, 1, 2, 0, 4, 5, 6, 0, 8, 9, 7, 0, 0, 0, 0];

    let mask = _mm512_set1_epi32(0xf);
    let lut1 = _mm512_broadcast_i32x4(_mm_loadu_epi8(LUT1.as_ptr().cast()));
    let lut2 = _mm512_broadcast_i32x4(_mm_loadu_epi8(LUT2.as_ptr().cast()));

    let mut sum1 = _mm512_setzero_epi32();
    let mut sum2 = _mm512_setzero_epi32();

    let mut i = 0;
    while i + 64 <= len {
        let chunk = _mm512_loadu_si512(ptr.add(i).cast());
        let srl = _mm512_srli_epi32::<14>(chunk);
        let or = _mm512_or_epi32(chunk, srl);
        let and = _mm512_and_epi32(or, mask);
        sum1 = _mm512_add_epi32(sum1, _mm512_shuffle_epi8(lut1, and));
        sum2 = _mm512_add_epi32(sum2, _mm512_shuffle_epi8(lut2, and));
        i += 64;
    }

    let mut sum1 = _mm512_reduce_add_epi32(sum1) as u32;
    let mut sum2 = _mm512_reduce_add_epi32(sum2) as u32;
    for chunk in input[i..].chunks_exact(4) {
        let chunk = u32::from_le_bytes(chunk.try_into().unwrap());
        let t = (chunk | chunk >> 14) & 0xf;
        sum1 += LUT1[t as usize] as u32;
        sum2 += LUT2[t as usize] as u32;
    }

    (sum1, sum2)
}

pub fn solve_shift(input: &[u8]) -> (u32, u32) {
    let mut sum = (0, 0);
    for chunk in input.chunks_exact(4) {
        let chunk = u32::from_le_bytes(chunk.try_into().unwrap());
        let t = ((chunk | chunk >> 14) & 0xf) * 4;
        sum.0 += ((0x693025807140u64 >> t) & 0xf) as u32;
        sum.1 += ((0x798065402130u64 >> t) & 0xf) as u32;
    }
    sum
}

pub fn solve_naive(input: &str) -> (u32, u32) {
    let mut sum = (0, 0);
    for line in input.lines() {
        let [a @ b'A'..=b'C', _, x @ b'X'..=b'Z'] = line.as_bytes() else {
            panic!();
        };
        let (opponent, x) = (a - b'A', x - b'X');

        // me, opponent: 0 = Rock, 1 = Paper, 2 = Scissors
        // outcome: 0 = Loss, 1 = Draw, 2 = Win
        // 1 + me - opponent ≡ outcome (mod 3)
        let me = x;
        let outcome = (4 + me - opponent) % 3;
        sum.0 += (1 + me + outcome * 3) as u32;

        let outcome = x;
        let me = (2 + outcome + opponent) % 3;
        sum.1 += (1 + me + outcome * 3) as u32;
    }
    sum
}
