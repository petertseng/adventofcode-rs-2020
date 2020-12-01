#![allow(unused)]
use adventofcode::day01::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashSet;

pub fn criterion_benchmark(c: &mut Criterion) {
    let s = std::fs::read_to_string("../adventofcode-common/secret-cases/2020/01p.in")
        .expect("couldn't read file");
    let nums: Vec<_> = s
        .lines()
        .map(|line| line.parse::<u32>().expect("can't parse integer"))
        .collect();
    let nums_set = nums.iter().cloned().collect();

    c.bench_function("by combinations", |b| {
        b.iter(|| by_combinations(black_box(&nums), black_box(&nums_set)))
    });
    c.bench_function("over input range", |b| {
        b.iter(|| over_input_range(black_box(&nums), black_box(&nums_set)))
    });
    c.bench_function("by combinations sorted", |b| {
        b.iter(|| by_combinations_sorted(black_box(&nums), black_box(&nums_set)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
