use criterion::{criterion_group, criterion_main, Criterion};

use mylib::weight::Weight;
use mylib::bitboard::BitBoard;


#[cfg(target_arch="x86_64")]
fn criterion_benchmark_weight(c : &mut Criterion) {
    let w = Weight::new();
    let ban = BitBoard::new();
    c.bench_function("weight_nosimd", |b| b.iter(|| w.evaluatev7bb(&ban)));
    c.bench_function("weight_simd_sse", |b| b.iter(|| w.evaluatev7bb_simd(&ban)));
    c.bench_function("weight_simd_avx", |b| b.iter(|| w.evaluatev7bb_simdavx(&ban)));
}

#[cfg(target_arch="aarch64")]
fn criterion_benchmark_weight(c : &mut Criterion) {
    let w = Weight::new();
    let ban = BitBoard::new();
    c.bench_function("weight_nosimd", |b| b.iter(|| w.evaluatev7bb(&ban)));
    c.bench_function("weight_simd_neon", |b| b.iter(|| w.evaluatev7bb_simd(&ban)));
}

criterion_group!(benches, criterion_benchmark_weight);
criterion_main!(benches);
