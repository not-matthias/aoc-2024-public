use arrayvec::ArrayVec;
use core::{
    hint::unreachable_unchecked,
    simd::{cmp::SimdPartialOrd, simd_swizzle, u8x64, u8x8, Simd},
};

#[inline(always)]
fn simd_is_any_valid(level: &[u8]) -> bool {
    unsafe { branches::assume(level.len() <= 8) };

    if simd_is_valid(level) {
        return true;
    }

    let item_mask = (1 << level.len() - 2) - 1;
    let curr = u8x64::load_or_default(level);

    #[rustfmt::skip]
    let curr = simd_swizzle!(
        curr,
        [
            1, 2, 3, 4, 5, 6, 7, 8,
            0, 2, 3, 4, 5, 6, 7, 8,
            0, 1, 3, 4, 5, 6, 7, 8,
            0, 1, 2, 4, 5, 6, 7, 8,
            0, 1, 2, 3, 5, 6, 7, 8,
            0, 1, 2, 3, 4, 6, 7, 8,
            0, 1, 2, 3, 4, 5, 7, 8,
            0, 1, 2, 3, 4, 5, 6, 8,
        ]
    );

    simd_is_valid_multi(curr, item_mask)
}

#[inline(always)]
fn simd_is_valid_multi(curr: u8x64, item_mask: u64) -> bool {
    let next = curr.rotate_elements_left::<1>();
    let is_asc = next.simd_gt(curr).to_bitmask();
    let is_desc = next.simd_lt(curr).to_bitmask();

    let asc0 = (is_asc & item_mask) == item_mask;
    let asc1 = (is_asc >> 8 & item_mask) == item_mask;
    let asc2 = (is_asc >> 16 & item_mask) == item_mask;
    let asc3 = (is_asc >> 24 & item_mask) == item_mask;
    let asc4 = (is_asc >> 32 & item_mask) == item_mask;
    let asc5 = (is_asc >> 40 & item_mask) == item_mask;
    let asc6 = (is_asc >> 48 & item_mask) == item_mask;
    let asc7 = (is_asc >> 56 & item_mask) == item_mask;

    let desc0 = (is_desc & item_mask) == item_mask;
    let desc1 = (is_desc >> 8 & item_mask) == item_mask;
    let desc2 = (is_desc >> 16 & item_mask) == item_mask;
    let desc3 = (is_desc >> 24 & item_mask) == item_mask;
    let desc4 = (is_desc >> 32 & item_mask) == item_mask;
    let desc5 = (is_desc >> 40 & item_mask) == item_mask;
    let desc6 = (is_desc >> 48 & item_mask) == item_mask;
    let desc7 = (is_desc >> 56 & item_mask) == item_mask;

    if (!asc0 && !desc0)
        && (!asc1 && !desc1)
        && (!asc2 && !desc2)
        && (!asc3 && !desc3)
        && (!asc4 && !desc4)
        && (!asc5 && !desc5)
        && (!asc6 && !desc6)
        && (!asc7 && !desc7)
    {
        return false;
    }

    let mask = curr.simd_gt(next);
    let diff = mask.select(curr - next, next - curr);
    let mask = diff.simd_ge(Simd::splat(1)) & diff.simd_le(Simd::splat(3));
    let mask = mask.to_bitmask();

    let diff0 = (mask & item_mask) == item_mask;
    let diff1 = (mask >> 8 & item_mask) == item_mask;
    let diff2 = (mask >> 16 & item_mask) == item_mask;
    let diff3 = (mask >> 24 & item_mask) == item_mask;
    let diff4 = (mask >> 32 & item_mask) == item_mask;
    let diff5 = (mask >> 40 & item_mask) == item_mask;
    let diff6 = (mask >> 48 & item_mask) == item_mask;
    let diff7 = (mask >> 56 & item_mask) == item_mask;

    ((asc0 || desc0) && diff0)
        || ((asc1 || desc1) && diff1)
        || ((asc2 || desc2) && diff2)
        || ((asc3 || desc3) && diff3)
        || ((asc4 || desc4) && diff4)
        || ((asc5 || desc5) && diff5)
        || ((asc6 || desc6) && diff6)
        || ((asc7 || desc7) && diff7)
}

#[inline(always)]
fn simd_is_valid(level: &[u8]) -> bool {
    unsafe { branches::assume(level.len() <= 8) };
    let item_mask = (1 << level.len() - 1) - 1;
    let curr = u8x8::load_or_default(level);
    let next = simd_swizzle!(curr, [1, 2, 3, 4, 5, 6, 7, 7]);
    let is_asc = next.simd_gt(curr).to_bitmask() & item_mask;
    let is_desc = next.simd_lt(curr).to_bitmask() & item_mask;
    if is_asc != item_mask && is_desc != item_mask {
        return false;
    }

    let mask = curr.simd_gt(next);
    let diff = mask.select(curr - next, next - curr);
    let mask = diff.simd_ge(Simd::splat(1)) & diff.simd_le(Simd::splat(3));
    let mask = mask.to_bitmask() & item_mask;
    mask == item_mask
}

#[inline(always)]
const fn to_digit(byte: u8) -> u8 {
    byte.wrapping_sub(b'0') as u8
}

#[target_feature(enable = "avx2,bmi1,bmi2,cmpxchg16b,lzcnt,movbe,popcnt")]
pub fn part1_inner(input: &str) -> u32 {
    let input = input.as_bytes();

    let mut parsed = ArrayVec::<u8, 8>::new_const();
    let mut i = 0;
    let mut count = 0;
    loop {
        if branches::unlikely(i >= input.len()) {
            break;
        }

        let cur = unsafe { *input.get_unchecked(i) };
        let next = *input.get(i + 1).unwrap_or(&b'\n');

        match (cur.is_ascii_digit(), next.is_ascii_digit()) {
            (true, false) => {
                unsafe { parsed.push_unchecked(to_digit(cur)) };
                if branches::likely(next == b' ') {
                    i += 2;
                } else if branches::unlikely(next == b'\n') {
                    count += simd_is_valid(&parsed) as u32;
                    parsed = ArrayVec::new_const();
                    i += 2;
                }
            }
            (true, true) => {
                unsafe { parsed.push_unchecked(to_digit(cur) * 10 + to_digit(next)) };

                let next_next = unsafe { *input.get_unchecked(i + 2) };
                if branches::likely(next_next == b' ') {
                    i += 3;
                } else if branches::unlikely(next_next == b'\n') {
                    count += simd_is_valid(&parsed) as u32;
                    parsed = ArrayVec::new_const();
                    i += 3;
                }
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }

    count
}

pub fn part1(input: &str) -> u32 {
    unsafe { part1_inner(input) }
}

#[target_feature(enable = "avx2,bmi1,bmi2,cmpxchg16b,lzcnt,movbe,popcnt")]
pub fn part2_inner(input: &str) -> u32 {
    let input = input.as_bytes();

    let mut parsed = ArrayVec::<u8, 8>::new_const();
    let mut i = 0;
    let mut count = 0;
    loop {
        if branches::unlikely(i >= input.len()) {
            break;
        }

        let cur = unsafe { *input.get_unchecked(i) };
        let next = *input.get(i + 1).unwrap_or(&b'\n');

        match (cur.is_ascii_digit(), next.is_ascii_digit()) {
            (true, false) => {
                unsafe { parsed.push_unchecked(to_digit(cur)) };

                if branches::likely(next == b' ') {
                    i += 2;
                } else if branches::unlikely(next == b'\n') {
                    count += simd_is_any_valid(&parsed) as u32;
                    parsed = ArrayVec::new_const();
                    i += 2;
                }
            }
            (true, true) => {
                unsafe { parsed.push_unchecked(to_digit(cur) * 10 + to_digit(next)) };

                let next_next = unsafe { *input.get_unchecked(i + 2) };
                if branches::likely(next_next == b' ') {
                    i += 3;
                } else if branches::unlikely(next_next == b'\n') {
                    count += simd_is_any_valid(&parsed) as u32;
                    parsed = ArrayVec::new_const();
                    i += 3;
                }
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }

    count
}

pub fn part2(input: &str) -> u32 {
    unsafe { part2_inner(input) }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";

    #[test]
    fn test_part1_simple() {
        assert_eq!(part1(INPUT), 2);
    }

    #[test]
    fn test_part2_simple() {
        assert_eq!(part2(INPUT), 4);
    }
}
