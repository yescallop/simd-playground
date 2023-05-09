use std::arch::x86_64::*;

pub fn part1_avx512(input: &[u8]) -> Option<usize> {
    unsafe { solve_avx512::<4>(input) }
}

pub fn part2_avx512(input: &[u8]) -> Option<usize> {
    unsafe { solve_avx512::<14>(input) }
}

unsafe fn solve_avx512<const N: usize>(input: &[u8]) -> Option<usize> {
    assert!(N >= 4);
    let len = input.len();
    let ptr = input.as_ptr();

    let mut i = 0;
    while i + 64 + 3 <= len {
        let chunk = _mm512_loadu_si512(ptr.add(i).cast());

        let off_by_1 = _mm512_loadu_si512(ptr.add(i + 1).cast());
        let off_by_2 = _mm512_loadu_si512(ptr.add(i + 2).cast());
        let off_by_3 = _mm512_loadu_si512(ptr.add(i + 3).cast());

        let mut neq = _mm512_cmpneq_epi8_mask(chunk, off_by_1);
        neq &= _mm512_cmpneq_epi8_mask(chunk, off_by_2);
        neq &= _mm512_cmpneq_epi8_mask(chunk, off_by_3);
        neq &= _mm512_cmpneq_epi8_mask(off_by_1, off_by_2);
        neq &= _mm512_cmpneq_epi8_mask(off_by_1, off_by_3);
        neq &= _mm512_cmpneq_epi8_mask(off_by_2, off_by_3);

        if neq != 0 {
            i += neq.trailing_zeros() as usize;
            if N == 4 {
                return Some(i + 4);
            }
            break;
        }
        i += 64;
    }

    Some(solve_xor::<N>(&input[i..])? + i)
}

pub fn part1_xor(input: &[u8]) -> Option<usize> {
    solve_xor::<4>(input)
}

pub fn part2_xor(input: &[u8]) -> Option<usize> {
    solve_xor::<14>(input)
}

fn solve_xor<const N: usize>(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < N {
        return None;
    }

    // flag = number of a byte value in the current window
    //    0 = zero or even
    //    1 = one or odd
    // current window contains no duplicates iff `flags.count_ones() == N`
    let mut flags = 0u32;
    for in_byte in &bytes[..N] {
        // Subtracting with 96 (b'a' - 1) is no-op.
        // See also: comments in Day 3.
        flags ^= 1 << (in_byte - 96);
    }
    if flags.count_ones() == N as u32 {
        return Some(4);
    }

    let pos = bytes.windows(N + 1).position(|one_larger_window| {
        let &[out_byte, .., in_byte] = one_larger_window else {
            unreachable!();
        };
        flags ^= 1 << (out_byte - 96);
        flags ^= 1 << (in_byte - 96);
        flags.count_ones() == N as u32
    });
    pos.map(|i| i + N + 1)
}

pub fn part1_naive(input: &[u8]) -> Option<usize> {
    Some(input.windows(4).position(all_distinct)? + 4)
}

pub fn part2_naive(input: &[u8]) -> Option<usize> {
    Some(input.windows(14).position(all_distinct)? + 14)
}

fn all_distinct(bytes: &[u8]) -> bool {
    let mut flags = 0u32;
    for x in bytes {
        flags |= 1 << (x - 96);
    }
    flags.count_ones() == bytes.len() as u32
}

pub fn part1_naive_short_circuit(input: &[u8]) -> Option<usize> {
    Some(input.windows(4).position(all_distinct_short_circuit)? + 4)
}

pub fn part2_naive_short_circuit(input: &[u8]) -> Option<usize> {
    Some(input.windows(14).position(all_distinct_short_circuit)? + 14)
}

fn all_distinct_short_circuit(bytes: &[u8]) -> bool {
    let mut flags = 0u32;
    for x in bytes {
        let mask = 1 << (x - 96);
        if flags & mask != 0 {
            return false;
        }
        flags |= mask;
    }
    true
}
