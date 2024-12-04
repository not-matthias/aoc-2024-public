use aoc_2024_public::day3::{part1, part2};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const INPUT: &str = include_str!("../input/2024/day3.txt");

fn bench_part1(c: &mut Criterion) {
    c.bench_function("part1", |b| {
        b.iter(|| part1(black_box(INPUT)))
        // b.iter(|| assert_eq!(part1(black_box(input)), 257))
    });
}

fn bench_part2(c: &mut Criterion) {
    c.bench_function("part2", |b| {
        b.iter(|| part2(black_box(INPUT)))
        // b.iter(|| assert_eq!(part2(black_box(input)), 328))
    });
}

criterion_group!(benches, bench_part1, bench_part2);
criterion_main!(benches);
