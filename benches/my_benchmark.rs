use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mylib::weight::Weight;
use mylib::bitboard::BitBoard;


#[cfg(target_arch="x86_64")]
fn criterion_benchmark_weight(_c : &mut Criterion) {
    let mut c = Criterion::default()
        .sample_size(1000);
    let mut w = Weight::new();
    w.init();
    let ban = BitBoard::new();
    c.bench_function("weight_nosimd_init", |b| b.iter(|| w.evaluatev9bb(black_box(&ban))));
    c.bench_function("weight_simd_sse_init", |b| b.iter(|| w.evaluatev9bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx_init", |b| b.iter(|| w.evaluatev9bb_simdavx(black_box(&ban))));
    // let ban = BitBoard::from("h/h/h/h/H/H/H/H b").unwrap();
    let ban = BitBoard::from(
        "aAaAaAaA/AaAaAaAa/aCaC/AcAc/bBb/BbBb/dD/Dd w").unwrap();
    c.bench_function("weight_nosimd", |b| b.iter(|| w.evaluatev9bb(black_box(&ban))));
    c.bench_function("weight_simd_sse", |b| b.iter(|| w.evaluatev9bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx", |b| b.iter(|| w.evaluatev9bb_simdavx(black_box(&ban))));

    let ban = BitBoard::new();
    c.bench_function("genmove_init", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::from(
        "h/aFa/aC1Ba/aFa/aFa/aFa/aFa/h w").unwrap();
    c.bench_function("genmove_last1", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::from(
        "2A1A1A1/3c2/Ac1bA/3c2/2cAa1/1a1Aa2A/A3a3/4A3 b").unwrap();
        // --*-*-*-
        // ---###--
        // *###-##*
        // ---###--
        // --###*#-
        // -#-*#--*
        // *---#---
        // ----*---
    c.bench_function("genmove_star", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
}

#[cfg(target_arch="aarch64")]
fn criterion_benchmark_weight(_c : &mut Criterion) {
    let mut c = Criterion::default()
        .sample_size(1000);
    let mut w = Weight::new();
    w.init();
    let ban = BitBoard::new();
    c.bench_function(
        "weight_nosimd_init",
        |b| b.iter(|| w.evaluatev9bb(black_box(&ban))));
    c.bench_function(
        "weight_simd_neon_mul_init",
        |b| b.iter(|| w.evaluatev9bb_simd_mul(black_box(&ban))));
    // let ban = BitBoard::from("h/h/h/h/H/H/H/H b").unwrap();
    let ban = BitBoard::from(
        "aAaAaAaA/AaAaAaAa/aCaC/AcAc/bBb/BbBb/dD/Dd w").unwrap();
    c.bench_function(
        "weight_nosimd",
        |b| b.iter(|| w.evaluatev9bb(black_box(&ban))));
    c.bench_function(
        "weight_simd_neon_mul",
        |b| b.iter(|| w.evaluatev9bb_simd_mul(black_box(&ban))));
    let ban = BitBoard::new();
    c.bench_function("genmove_init", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::from(
        "h/aFa/aC1Ba/aFa/aFa/aFa/aFa/h w").unwrap();
    c.bench_function("genmove_last1", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::from(
        "2A1A1A1/3c2/Ac1bA/3c2/2cAa1/1a1Aa2A/A3a3/4A3 b").unwrap();
        // --*-*-*-
        // ---###--
        // *###-##*
        // ---###--
        // --###*#-
        // -#-*#--*
        // *---#---
        // ----*---
    c.bench_function("genmove_star", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
}

criterion_group!(benches, criterion_benchmark_weight);
criterion_main!(benches);
