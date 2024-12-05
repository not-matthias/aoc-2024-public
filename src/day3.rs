#[inline(always)]
fn parse_3b(input: &[u8]) -> Option<(u8, u16)> {
    match input {
        [num1 @ b'0'..=b'9', num2 @ b'0'..=b'9', num3 @ b'0'..=b'9', ..] => {
            let num = (num1 - b'0') as u16 * 100 + (num2 - b'0') as u16 * 10 + (num3 - b'0') as u16;
            Some((3, num))
        }
        [num1 @ b'0'..=b'9', num2 @ b'0'..=b'9', ..] => {
            let num = (num1 - b'0') as u16 * 10 + (num2 - b'0') as u16;
            Some((2, num))
        }
        [num1 @ b'0'..=b'9', ..] => Some((1, (num1 - b'0') as u16)),
        _ => None,
    }
}

#[inline(always)]
pub fn parse_mul(memory: &[u8], index: &mut usize) -> Option<(u16, u16)> {
    unsafe {
        if *memory.get_unchecked(*index) != b'm'
            && *memory.get_unchecked(*index + 1) != b'u'
            && *memory.get_unchecked(*index + 2) != b'l'
            && *memory.get_unchecked(*index + 3) != b'('
        {
            *index += 1;
            return None;
        }
        *index += 4;

        let Some((cnt, first)) = parse_3b(&memory.get_unchecked(*index..)) else {
            return None;
        };
        *index += cnt as usize;

        if *memory.get_unchecked(*index) != b',' {
            return None;
        }
        *index += 1;

        let Some((cnt, second)) = parse_3b(&memory.get_unchecked(*index..)) else {
            return None;
        };
        *index += cnt as usize;

        if *memory.get_unchecked(*index) != b')' {
            return None;
        }
        *index += 1;

        Some((first, second))
    }
}

#[target_feature(enable = "avx2,bmi1,bmi2,cmpxchg16b,lzcnt,movbe,popcnt")]
unsafe fn part1_inner(memory: &[u8]) -> u32 {
    let mut index = 0;
    let mut result = 0;

    let mul_finder = memchr::memmem::Finder::new("m");
    while index < memory.len() {
        let Some(next_mul) = mul_finder.find(memory.get_unchecked(index..)) else {
            break;
        };
        index += next_mul;

        if let Some((first, second)) = parse_mul(memory, &mut index) {
            result += first as u32 * second as u32;
        }
    }

    result
}

pub fn part1(input: &str) -> u32 {
    unsafe { part1_inner(input.as_bytes()) }
}

pub fn part2(input: &str) -> u32 {
    #[target_feature(enable = "avx2,bmi1,bmi2,cmpxchg16b,lzcnt,movbe,popcnt")]
    unsafe fn part2_inner(input: &str) -> u32 {
        let memory = input.as_bytes();

        let mut index = 0;
        let mut result = 0;

        const DO_SIZE: usize = 4;
        const DONT_SIZE: usize = 6;

        let dont_finder = memchr::memmem::Finder::new("don't()");
        let do_finder = memchr::memmem::Finder::new("do()");

        while let Some(dont_offset) = dont_finder.find(memory.get_unchecked(index..)) {
            let dont_idx = index + dont_offset;
            result += unsafe { part1_inner(&memory.get_unchecked(index..dont_idx)) };

            let do_offset = do_finder
                .find(memory.get_unchecked(dont_idx + DONT_SIZE..))
                .unwrap_or(input.len() - DO_SIZE);
            index = dont_idx + do_offset + DO_SIZE;
        }

        result + unsafe { part1_inner(&memory.get_unchecked(index..)) }
    }

    unsafe { part2_inner(input) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1_simple() {
        const INPUT: &str =
            "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
        assert_eq!(part1(INPUT), 161);
    }

    #[test]
    fn test_part2_simple() {
        const INPUT: &str =
            "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";
        assert_eq!(part2(INPUT), 48);
    }
}
