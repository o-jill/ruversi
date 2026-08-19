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
    // c.bench_function("weight_nosimd_init", |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    // c.bench_function("weight_simd_sse_init", |b| b.iter(|| w.evaluatev12bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx_init", |b| b.iter(|| w.evaluatev12bb_simdavx(black_box(&ban))));
    c.bench_function("weight_simd_avx_init_2", |b| b.iter(|| w.evaluatev12bb_simdavx_2(black_box(&ban))));
    // c.bench_function("weight_nosimd_init_i16", |b| b.iter(|| w.evaluatev12bb_i16(black_box(&ban))));
    c.bench_function("weight_simd_sse_init_i16", |b| b.iter(|| w.evaluatev12bb_simd_i16(black_box(&ban))));
    c.bench_function("weight_simd_avx_init_i16", |b| b.iter(|| w.evaluatev12bb_simdavx_i16(black_box(&ban))));
    // c.bench_function("weight_simd_avx_init_i16_3", |b| b.iter(|| w.evaluatev12bb_simdavx_i16_3(black_box(&ban))));
    // c.bench_function("weight_simd_avx_init_i16_2", |b| b.iter(|| w.evaluatev12bb_simdavx_i16_2(black_box(&ban))));
    // let ban = BitBoard::try_from("h/h/h/h/H/H/H/H b").unwrap();
    let ban = BitBoard::try_from(
        "aAaAaAaA/AaAaAaAa/aCaC/AcAc/bBb/BbBb/dD/Dd w").unwrap();
    // c.bench_function("weight_nosimd", |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    // c.bench_function("weight_simd_sse", |b| b.iter(|| w.evaluatev12bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx", |b| b.iter(|| w.evaluatev12bb_simdavx(black_box(&ban))));
    c.bench_function("weight_simd_avx_2", |b| b.iter(|| w.evaluatev12bb_simdavx_2(black_box(&ban))));
    // c.bench_function("weight_nosimd_i16", |b| b.iter(|| w.evaluatev12bb_i16(black_box(&ban))));
    c.bench_function("weight_simd_sse_i16", |b| b.iter(|| w.evaluatev12bb_simd_i16(black_box(&ban))));
    c.bench_function("weight_simd_avx_i16", |b| b.iter(|| w.evaluatev12bb_simdavx_i16(black_box(&ban))));
    // c.bench_function("weight_simd_avx_i16_3", |b| b.iter(|| w.evaluatev12bb_simdavx_i16_3(black_box(&ban))));
    // c.bench_function("weight_simd_avx_i16_2", |b| b.iter(|| w.evaluatev12bb_simdavx_i16_2(black_box(&ban))));

    let ban = BitBoard::try_from(
        "aA1aA1aA/A1aA1aA1/Bb2b/1Cc1/1cC1/B2bB/Aa1Aa1Aa/1Aa1Aa1a b").unwrap();
    // c.bench_function("weight_nosimd_46", |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    // c.bench_function("weight_simd_sse_46", |b| b.iter(|| w.evaluatev12bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx_46", |b| b.iter(|| w.evaluatev12bb_simdavx(black_box(&ban))));
    c.bench_function("weight_simd_avx_46_2", |b| b.iter(|| w.evaluatev12bb_simdavx_2(black_box(&ban))));
    // c.bench_function("weight_nosimd_46_i16", |b| b.iter(|| w.evaluatev12bb_i16(black_box(&ban))));
    c.bench_function("weight_simd_sse_46_i16", |b| b.iter(|| w.evaluatev12bb_simd_i16(black_box(&ban))));
    c.bench_function("weight_simd_avx_46_i16", |b| b.iter(|| w.evaluatev12bb_simdavx_i16(black_box(&ban))));

    let ban = BitBoard::try_from(
        "aA2aA2/A2aA2a/B4b/2Bb2/2bB2/b4B/A2aA2a/aA2aA2 b").unwrap();
    // c.bench_function("weight_nosimd_32", |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    // c.bench_function("weight_simd_sse_32", |b| b.iter(|| w.evaluatev12bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx_32", |b| b.iter(|| w.evaluatev12bb_simdavx(black_box(&ban))));
    c.bench_function("weight_simd_avx_32_2", |b| b.iter(|| w.evaluatev12bb_simdavx_2(black_box(&ban))));
    // c.bench_function("weight_nosimd_32_i16", |b| b.iter(|| w.evaluatev12bb_i16(black_box(&ban))));
    c.bench_function("weight_simd_sse_32_i16", |b| b.iter(|| w.evaluatev12bb_simd_i16(black_box(&ban))));
    c.bench_function("weight_simd_avx_32_i16", |b| b.iter(|| w.evaluatev12bb_simdavx_i16(black_box(&ban))));

    let ban = BitBoard::try_from(
        "8/4a3/2D2/2c3/2C3/2d2/3A4/8 w").unwrap();
    // c.bench_function("weight_nosimd_16", |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    // c.bench_function("weight_simd_sse_16", |b| b.iter(|| w.evaluatev12bb_simd(black_box(&ban))));
    c.bench_function("weight_simd_avx_16", |b| b.iter(|| w.evaluatev12bb_simdavx(black_box(&ban))));
    c.bench_function("weight_simd_avx_16_2", |b| b.iter(|| w.evaluatev12bb_simdavx_2(black_box(&ban))));
    // c.bench_function("weight_nosimd_16_i16", |b| b.iter(|| w.evaluatev12bb_i16(black_box(&ban))));
    c.bench_function("weight_simd_sse_16_i16", |b| b.iter(|| w.evaluatev12bb_simd_i16(black_box(&ban))));
    c.bench_function("weight_simd_avx_16_i16", |b| b.iter(|| w.evaluatev12bb_simdavx_i16(black_box(&ban))));

    let ban = BitBoard::new();
    c.bench_function("genmove_init", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::try_from(
        "h/aFa/aC1Ba/aFa/aFa/aFa/aFa/h w").unwrap();
    c.bench_function("genmove_last1", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::try_from(
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
        |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    c.bench_function(
        "weight_simd_neon_mul_init",
        |b| b.iter(|| w.evaluatev12bb_simd_mul(black_box(&ban))));
    // let ban = BitBoard::try_from("h/h/h/h/H/H/H/H b").unwrap();
    let ban = BitBoard::try_from(
        "aAaAaAaA/AaAaAaAa/aCaC/AcAc/bBb/BbBb/dD/Dd w").unwrap();
    c.bench_function(
        "weight_nosimd",
        |b| b.iter(|| w.evaluatev12bb(black_box(&ban))));
    c.bench_function(
        "weight_simd_neon_mul",
        |b| b.iter(|| w.evaluatev12bb_simd_mul(black_box(&ban))));
    let ban = BitBoard::new();
    c.bench_function("genmove_init", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::try_from(
        "h/aFa/aC1Ba/aFa/aFa/aFa/aFa/h w").unwrap();
    c.bench_function("genmove_last1", |b| {
        b.iter(|| {
            ban.genmove()
        })
    });
    let ban = BitBoard::try_from(
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
