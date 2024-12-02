use iterator_ilp::IteratorILP;

// https://rust.godbolt.org/z/coxTaWhYc
#[inline(always)]
pub fn atoi_see(input: &[u8]) -> u32 {
    assert!(input.len() <= 5, "Input must be up to 5 characters");
    let input_bytes = input;

    // Constants matching the .LCPI1_0 and .LCPI1_1 in the original assembly.
    static GODBOLT_0: [u8; 16] = [
        208, 208, 208, 208, // -48 in 2's complement
        0, 0, 0, 0, // Padding
        0, 0, 0, 0, // Padding
        0, 0, 0, 0, // Padding
    ];

    static GODBOLT_1: [u16; 8] = [10000, 0, 1000, 0, 100, 0, 10, 0];

    let result: u32;
    unsafe {
        core::arch::asm!(
            // Load 4 bytes from `input` into XMM0
            "vmovd xmm0, dword ptr [rip + {input_ptr}]",

            // Add adjustment bytes for ASCII-to-integer conversion
            "vpaddb xmm0, xmm0, xmmword ptr [rip + {adjustment_table}]",

            // Zero-extend the bytes to integers
            "vpmovzxbd xmm0, xmm0",

            // Multiply and accumulate digits
            "vpmaddwd xmm0, xmm0, xmmword ptr [rip + {multipliers}]",

            // Handle the final digit
            "movzx eax, byte ptr [rip + {input_ptr} + 4]",
            "add al, -48",
            "movzx ecx, al",

            // Sum the results
            "vpshufd xmm1, xmm0, 238",
            "vpaddd xmm0, xmm0, xmm1",
            "vpsrlq xmm1, xmm0, 32",
            "vpaddd xmm0, xmm0, xmm1",
            "vmovd eax, xmm0",
            "add eax, ecx",

            // Output
            out("eax") result,

            // Inputs
            input_ptr = in(reg) input_bytes.as_ptr(),
            adjustment_table = sym GODBOLT_0,
            multipliers = sym GODBOLT_1,

            // Clobbers
            out("ecx") _,
            out("xmm0") _,
            out("xmm1") _,
        );
    }
    result
}

#[inline(always)]
fn atoi(bytes: &[u8]) -> i32 {
    macro b($mult:expr, $idx:expr) {{
        let value: u8 = unsafe { *bytes.get_unchecked($idx) };
        (value - b'0') as i32 * $mult
    }}
    assert_eq!(bytes.len(), 5);
    b!(10000, 0) + b!(1000, 1) + b!(100, 2) + b!(10, 3) + b!(1, 4)

    // Vectorized
    // atoi_see(bytes) as i32
}

pub fn part1(input: &str) -> impl core::fmt::Display {
    let input = input.as_bytes();

    unsafe {
        let mut A: [i32; 1000] = [0; 1000];
        let mut B: [i32; 1000] = [0; 1000];

        for i in 0..1000 {
            let line = i * 14;

            let a_range = line..line + 5;
            let b_range = line + 8..line + 13;
            let a = atoi(&input.get_unchecked(a_range));
            let b = atoi(&input.get_unchecked(b_range));

            *A.get_unchecked_mut(i) = a;
            *B.get_unchecked_mut(i) = b;
        }

        radsort::sort(&mut A);
        radsort::sort(&mut B);

        A.iter()
            .zip(B.iter())
            .map(|(l, r)| l.abs_diff(*r) as i32)
            .sum_ilp::<32, i32>()
    }
}

pub fn part2(input: &str) -> impl core::fmt::Display {
    let input = input.as_bytes();

    unsafe {
        let mut A: [i32; 1000] = [0; 1000];
        let mut freq: [u8; 100_000] = [0; 100_000];

        for i in 0..1000 {
            let line = i * 14;
            let a_range = line..line + 5;
            let b_range = line + 8..line + 13;

            let a = atoi(&input.get_unchecked(a_range));
            *A.get_unchecked_mut(i) = a;

            let b = atoi(&input.get_unchecked(b_range));

            *freq.get_unchecked_mut(b as usize) += 1;
        }

        A.iter()
            .map(|a| a * *freq.get_unchecked(*a as usize) as i32)
            .sum_ilp::<32, i32>()
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_part1_opt() {
        let input = include_str!("../input/2024/day1.txt");
        assert_eq!(part1(input).to_string(), "2164381");
    }

    #[bench]
    fn bench_part1(b: &mut test::Bencher) {
        let input = include_str!("../input/2024/day1.txt");
        b.iter(|| assert_eq!(part1(input).to_string(), "2164381"));
    }

    #[test]
    fn test_part2_opt() {
        let input = include_str!("../input/2024/day1.txt");
        assert_eq!(part2(input).to_string(), "20719933");
    }

    #[bench]
    fn bench_part2(b: &mut test::Bencher) {
        let input = include_str!("../input/2024/day1.txt");
        b.iter(|| assert_eq!(part2(input).to_string(), "20719933"));
    }
}
