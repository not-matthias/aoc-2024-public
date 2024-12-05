use core::simd::prelude::*;
use std::ops::{BitAnd, Shl, Shr};

const LINE_COUNT: usize = 140;
const LINE_LEN: usize = 140 + 1 /* \n */;

type LineMask = LineMaskStruct;

#[derive(Debug, Copy, Clone)]
#[repr(align(64))]
struct LineMaskStruct {
    low: u64,
    mid: u64,
    high: u64,
}

impl LineMaskStruct {
    pub const fn new(low: u64, mid: u64, high: u64) -> Self {
        Self { low, mid, high }
    }

    #[inline(always)]
    pub fn count_ones(&self) -> u32 {
        self.low.count_ones() + self.mid.count_ones() + self.high.count_ones()
    }
}

impl Shl<usize> for LineMaskStruct {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: usize) -> Self::Output {
        Self {
            low: self.low << rhs,
            mid: self.mid << rhs | (self.low >> (64 - rhs)),
            high: self.high << rhs | (self.mid >> (64 - rhs)),
        }
    }
}

impl Shr<usize> for LineMaskStruct {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: usize) -> Self::Output {
        // We can shift from a to b.
        Self {
            low: self.low >> rhs | (self.mid << (64 - rhs)),
            mid: self.mid >> rhs | (self.high << (64 - rhs)),
            high: self.high >> rhs,
        }
    }
}

impl BitAnd for LineMaskStruct {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low & rhs.low,
            mid: self.mid & rhs.mid,
            high: self.high & rhs.high,
        }
    }
}

#[derive(Debug)]
struct Line {
    a: u8x64,
    b: u8x64,
    c: u8x64,
}

impl Line {
    #[inline(always)]
    pub fn from_input(input: &[u8]) -> Self {
        unsafe { branches::assume(input.len() >= 64) };
        unsafe { branches::assume(input[64..].len() >= 64) };

        // IMPORTANT: Never use load_or_default since it'll include lots of other
        // instructions and slow down the overall solution.

        let a = u8x64::from_slice(&input);
        let b = u8x64::from_slice(&input[64..]);
        let c = u8x64::load_or_default(unsafe { input.get_unchecked(128..140) }); // exclude newline

        Self { a, b, c }
    }

    #[inline(always)]
    pub fn simd_eq(&self, byte: u8) -> LineMask {
        let splat = Simd::splat(byte);

        let a = self.a.simd_eq(splat).to_bitmask();
        let b = self.b.simd_eq(splat).to_bitmask();
        let c = self.c.simd_eq(splat).to_bitmask();

        LineMask::new(a, b, c)
    }
}

impl Line {
    pub fn count_hori(&self) -> usize {
        let x = self.simd_eq(b'X');
        let m = self.simd_eq(b'M');
        let a = self.simd_eq(b'A');
        let s = self.simd_eq(b'S');

        let is_xmas = (x << 3) & (m << 2) & (a << 1) & s;
        let is_samx = (s << 3) & (a << 2) & (m << 1) & x;

        is_xmas.count_ones() as usize + is_samx.count_ones() as usize
    }

    pub fn check_vert(line1: &Line, line2: &Line, line3: &Line, line4: &Line) -> usize {
        let mut count = 0;

        {
            let l1x = line1.simd_eq(b'X');
            let l2m = line2.simd_eq(b'M');
            let l3a = line3.simd_eq(b'A');
            let l4s = line4.simd_eq(b'S');
            let is_xmas = l1x & l2m & l3a & l4s;
            count += is_xmas.count_ones() as usize;
        }

        {
            let l1s = line1.simd_eq(b'S');
            let l2a = line2.simd_eq(b'A');
            let l3m = line3.simd_eq(b'M');
            let l4x = line4.simd_eq(b'X');
            let is_samx = l1s & l2a & l3m & l4x;
            count += is_samx.count_ones() as usize;
        }

        count
    }

    pub fn check_diag(line1: &Line, line2: &Line, line3: &Line, line4: &Line) -> usize {
        let mut count = 0;

        {
            let l1x = line1.simd_eq(b'X');
            let l2m = line2.simd_eq(b'M');
            let l3a = line3.simd_eq(b'A');
            let l4s = line4.simd_eq(b'S');
            let left_xmas = l1x & (l2m >> 1) & (l3a >> 2) & (l4s >> 3);
            let right_xmas = l1x & (l2m << 1) & (l3a << 2) & (l4s << 3);

            count += left_xmas.count_ones() as usize + right_xmas.count_ones() as usize;
        }
        {
            let l1s = line1.simd_eq(b'S');
            let l2a = line2.simd_eq(b'A');
            let l3m = line3.simd_eq(b'M');
            let l4x = line4.simd_eq(b'X');
            let left_samx = l1s & (l2a >> 1) & (l3m >> 2) & (l4x >> 3);
            let right_samx = l1s & (l2a << 1) & (l3m << 2) & (l4x << 3);

            count += left_samx.count_ones() as usize + right_samx.count_ones() as usize;
        }

        count
    }
}

pub fn part1(input: &str) -> usize {
    #[target_feature(enable = "avx2,bmi1,bmi2,cmpxchg16b,lzcnt,movbe,popcnt")]
    unsafe fn part1_inner(input: &[u8]) -> usize {
        let mut count = 0;

        for i in 0..(LINE_COUNT - 3) {
            unsafe { branches::assume(LINE_LEN * (i + 3) < input.len()) };

            let line1 = Line::from_input(&input[LINE_LEN * i..]);
            let line2 = Line::from_input(&input[LINE_LEN * (i + 1)..]);
            let line3 = Line::from_input(&input[LINE_LEN * (i + 2)..]);
            let line4 = Line::from_input(&input[LINE_LEN * (i + 3)..]);

            count += line1.count_hori();
            count += Line::check_vert(&line1, &line2, &line3, &line4);
            count += Line::check_diag(&line1, &line2, &line3, &line4);
        }

        count += Line::from_input(&input[LINE_LEN * (LINE_COUNT - 3)..]).count_hori();
        count += Line::from_input(&input[LINE_LEN * (LINE_COUNT - 2)..]).count_hori();
        count += Line::from_input(&input[LINE_LEN * (LINE_COUNT - 1)..]).count_hori();

        count
    }

    unsafe { part1_inner(input.as_bytes()) }
}

pub fn part2(input: &str) -> usize {
    #[target_feature(enable = "avx2,bmi1,bmi2,cmpxchg16b,lzcnt,movbe,popcnt")]
    unsafe fn part2_inner(input: &[u8]) -> usize {
        let mut count = 0;

        for i in 0..(LINE_COUNT - 2) {
            unsafe { branches::assume(LINE_LEN * (i + 2) < input.len()) };

            let line1 = Line::from_input(&input[LINE_LEN * i..]);
            let line2 = Line::from_input(&input[LINE_LEN * (i + 1)..]);
            let line3 = Line::from_input(&input[LINE_LEN * (i + 2)..]);

            let l1m = line1.simd_eq(b'M');
            let l2a = line2.simd_eq(b'A');
            let l3s = line3.simd_eq(b'S');

            let xmas_left = (l1m >> 1) & l2a & (l3s << 1);
            let xmas_right = (l1m << 1) & l2a & (l3s >> 1);

            let l1s = line1.simd_eq(b'S');
            let l3m = line3.simd_eq(b'M');

            let samx_left = (l1s >> 1) & l2a & (l3m << 1);
            let samx_right = (l1s << 1) & l2a & (l3m >> 1);

            count += (xmas_left & xmas_right).count_ones() as usize;
            count += (samx_left & samx_right).count_ones() as usize;
            count += (xmas_left & samx_right).count_ones() as usize;
            count += (samx_left & xmas_right).count_ones() as usize;
        }

        count
    }

    unsafe { part2_inner(input.as_bytes()) }
}
