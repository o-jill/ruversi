use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};
use std::arch::x86_64;

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 2(fixedstones) + 1
 * hidden: 8 + 1
 * output: 1
 */
const N_INPUT : usize = board::CELL_2D + 1 + 2;
const N_HIDDEN : usize = 8;
const N_OUTPUT : usize = 1;
const N_WEIGHT: usize = (N_INPUT + 1) * N_HIDDEN + N_HIDDEN + 1;

#[allow(dead_code)]
const WSZV1 : usize = (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN + N_HIDDEN + 1;

#[derive(PartialEq)]
enum EvalFile{
    Unknown,
    V1,
}

impl EvalFile {
    pub fn to_str(&self) -> &str {
        match self {
            EvalFile::Unknown => {"unknown eval file format."},
            EvalFile::V1 => {"# 64+1+2-8-1"},
        }
    }

    pub fn from(txt : &str) -> Option<EvalFile> {
        match txt {
            "# 64+1+2-8-1" => Some(EvalFile::V1),
            _ => None
        }
    }
}

/// soft sign function
macro_rules! softsign {
    ($x : expr) => {
        // ($x * 0.5 / ($x.abs() + 1.0) + 0.5)  // 0 ~ 1
        ($x / ($x.abs() + 1.0))  // -1 ~ 1
    };
}

pub struct Weight {
    pub weight : Vec<f32>
}

impl Weight {
    pub fn new() -> Weight {
        Weight {
            weight: vec![0.0 ; N_WEIGHT]
        }
    }

    pub fn init(&mut self) {
        let mut rng = rand::thread_rng();
        let range =
            f64::sqrt(6.0) / f64::sqrt((N_INPUT + N_HIDDEN + N_OUTPUT) as f64);

        for a in self.weight.iter_mut() {
            *a = (rng.gen::<f64>() * 2.0 * range - range) as f32;
        }
    }

    pub fn copy(&mut self, src : &Weight) {
        for (d, s) in self.weight.iter_mut().zip(src.weight.iter()) {
            *d = *s;
        }
    }

    /// read eval table from a file.
    /// 
    /// # Arguments
    /// - `path` file path to a eval table.  
    ///   "RANDOM" is a special text to fill every paramerter w/ random numbers.
    pub fn read(&mut self, path : &str) -> Result<(), String> {
        if path == "RANDOM" {
            self.init();
            return Ok(());
        }
        let mut format = EvalFile::Unknown;
        let file = File::open(path);
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if l.starts_with("#") {
                        if format != EvalFile::Unknown {
                            continue;
                        }
                        let res = EvalFile::from(&l);
                        if res.is_some() {
                            format = res.unwrap();
                        }
                        continue;
                    }
                    match format {
                        EvalFile::V1 => {return self.readv1(&l)},
                        _ => {}
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }

        Err("no weight".to_string())
    }

    fn readv1(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV1 != nsz {
            return Err(String::from("size mismatch"));
        }
        self.weight = newtable;
        // println!("v1:{:?}", self.weight);
        Ok(())
    }

    fn write(f : &mut File, w : &Vec<f32>, ver : &EvalFile) {
        let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        f.write(format!("{}\n", ver.to_str()).as_bytes()).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
    }

    pub fn writev1(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V1);
    }

    pub fn evaluatev1(&self, ban : &board::Board) -> f32 {
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = wdc[i];
            for (idx, c)  in cells.iter().enumerate() {
                hidsum += *c as f32 * w1[idx];
            }
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;

            // sum += wh[i] / (f32::exp(-hidsum) + 1.0);  // sigmoid
            sum += wh[i] * softsign!(hidsum);
        }
        sum
    }

    pub fn evaluatev3_simd(&self, ban : &board::Board) -> f32 {
        self.evaluatev1(ban)
    }

    pub fn evaluatev1bb(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[bitboard::CELL_2D * N_HIDDEN .. (bitboard::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(bitboard::CELL_2D + 1) * N_HIDDEN .. (bitboard::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(bitboard::CELL_2D + 1 + 2) * N_HIDDEN .. (bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * bitboard::CELL_2D .. (i + 1) * bitboard::CELL_2D];
            let mut hidsum : f32 = wdc[i];
            for y in 0..bitboard::NUMCELL {
                let mut bit = bitboard::LSB_CELL << y;
                for x in 0..bitboard::NUMCELL {
                    let w = w1[x + y * bitboard::NUMCELL];
                    hidsum +=
                        if (bit & black) != 0 {w}
                        else if (bit & white) != 0 {-w}
                        else {0.0};
                    bit <<= bitboard::NUMCELL;
                }
            }
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;

            // sum += wh[i] / (f32::exp(-hidsum) + 1.0);  // sigmoid
            sum += wh[i] * softsign!(hidsum);
        }
        sum
    }

    pub fn evaluatev1bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 4;
        let mut sumarr : [f32 ; N] = [0.0 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * N] = [0.0 ; N * N];

            for n in 0..N {
                let res4 = sum44[n * N..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                const M : usize = 16;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> 2 * j;
                    let w81 = (bit8 & white) >> 2 * j;
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> 2 * j + 1;
                    let w82 = (bit8 & white) >> 2 * j + 1;
                    bit8 <<= 1;

                    unsafe {
                        let b08 = x86_64::_mm_set_epi64x(b82 as i64, b81 as i64);
                        let w08 = x86_64::_mm_set_epi64x(w82 as i64, w81 as i64);
                        let one = x86_64::_mm_set1_epi8(1);
                        let bm08 = x86_64::_mm_cmpeq_epi8(b08, one);
                        let wm08 = x86_64::_mm_cmpeq_epi8(w08, one);
                        let b16l = x86_64::_mm_unpacklo_epi8(bm08, bm08);
                        let b16h = x86_64::_mm_unpackhi_epi8(bm08, bm08);
                        let w16l = x86_64::_mm_unpacklo_epi8(wm08, wm08);
                        let w16h = x86_64::_mm_unpackhi_epi8(wm08, wm08);
                        let bm1 = x86_64::_mm_unpacklo_epi16(b16l, b16l);
                        let bm2 = x86_64::_mm_unpackhi_epi16(b16l, b16l);
                        let bm3 = x86_64::_mm_unpacklo_epi16(b16h, b16h);
                        let bm4 = x86_64::_mm_unpackhi_epi16(b16h, b16h);
                        let wm1 = x86_64::_mm_unpacklo_epi16(w16l, w16l);
                        let wm2 = x86_64::_mm_unpackhi_epi16(w16l, w16l);
                        let wm3 = x86_64::_mm_unpacklo_epi16(w16h, w16h);
                        let wm4 = x86_64::_mm_unpackhi_epi16(w16h, w16h);
                        let ex1 = x86_64::_mm_or_si128(bm1, wm1);
                        let ex2 = x86_64::_mm_or_si128(bm2, wm2);
                        let ex3 = x86_64::_mm_or_si128(bm3, wm3);
                        let ex4 = x86_64::_mm_or_si128(bm4, wm4);
                        let x41 = x86_64::_mm_load_ps(w1.as_ptr().add(idx));
                        let x42 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 4));
                        let x43 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 8));
                        let x44 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 12));
                        let minus = x86_64::_mm_set1_ps(-0.0);
                        let mn1 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm1), minus);
                        let mn2 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm2), minus);
                        let mn3 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm3), minus);
                        let mn4 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm4), minus);
                        let m41 = x86_64::_mm_xor_ps(x41, mn1);
                        let m42 = x86_64::_mm_xor_ps(x42, mn2);
                        let m43 = x86_64::_mm_xor_ps(x43, mn3);
                        let m44 = x86_64::_mm_xor_ps(x44, mn4);
                        let w1 = x86_64::_mm_and_ps(m41, x86_64::_mm_castsi128_ps(ex1));
                        let w2 = x86_64::_mm_and_ps(m42, x86_64::_mm_castsi128_ps(ex2));
                        let w3 = x86_64::_mm_and_ps(m43, x86_64::_mm_castsi128_ps(ex3));
                        let w4 = x86_64::_mm_and_ps(m44, x86_64::_mm_castsi128_ps(ex4));

                        let sum12 = x86_64::_mm_add_ps(w1, w2);
                        let sum34 = x86_64::_mm_add_ps(w3, w4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs[hidx..].as_ptr());
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm_load_ps(wfs[hidx + N_HIDDEN..].as_ptr());
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);

                if true {  // softsign
                    let sign4 = x86_64::_mm_set1_epi32(i32::MAX);
                    let abs4 = x86_64::_mm_and_ps(h1234, x86_64::_mm_castsi128_ps(sign4));
                    let one = x86_64::_mm_set1_ps(1.0);
                    let abs4p1 = x86_64::_mm_add_ps(abs4, one);
                    let ssgn = x86_64::_mm_div_ps(h1234, abs4p1);

                    // ($x * 0.5 / ($x.abs() + 1.0) + 0.5)  // 0 ~ 1
                    // let half = x86_64::_mm_set1_ps(0.5);
                    // let ssgn05 = x86_64::_mm_mul_ps(half, ssgn);
                    // let ssgnadj = x86_64::_mm_add_ps(ssgn05, half);
                    // let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
                    // let y4 = x86_64::_mm_mul_ps(wh4, ssgnadj);

                    // ($x / ($x.abs() + 1.0))  // -1 ~ 1
                    let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
                    let y4 = x86_64::_mm_mul_ps(wh4, ssgn);

                    x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
                } else {// sigmoid
                let emx4 = weight::Weight::expmx_ps_simd(h1234);
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
                let y4 = x86_64::_mm_div_ps(wh4, hsp14);

                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
                }
            }
            // for n in 0..N {
            //     sum += sumarr[n];
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    pub fn evaluatev3bb(&self, ban : &bitboard::BitBoard) -> f32 {
        self.evaluatev1bb(ban)
    }
    pub fn evaluatev3bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        self.evaluatev1bb_simd(ban)
    }
    pub fn evaluatev3bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        self.evaluatev1bb_simd(ban)
    }

    pub fn forwardv1(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];

        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = wdc[i];
            for (&w, &c) in w1.iter().zip(cells.iter()) {
                hidsum += w * c as f32;
            }
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;
            hidden[i] = hidsum;

            hidsig[i] = softsign!(hidsum);

            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    pub fn forwardv3(&self, ban : &board::Board) 
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        self.forwardv1(ban)
    }

    pub fn forwardv1bb(&self, ban : &bitboard::BitBoard)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];

        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = wdc[i];
            for y in 0..bitboard::NUMCELL {
                let mut bit = bitboard::LSB_CELL << y;
                for x in 0..bitboard::NUMCELL {
                    let w = w1[x + y * bitboard::NUMCELL];
                    let cb = (black & bit) != 0;
                    let cw = (white & bit) != 0;
                    hidsum += if cb {w} else if cw {-w} else {0.0};
                    bit <<= bitboard::NUMCELL;
                }
            }
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;
            hidden[i] = hidsum;
            hidsig[i] = softsign!(hidsum);
            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    pub fn forwardv1bb_simd(&self, ban : &bitboard::BitBoard)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 4;
        let mut sumarr : [f32 ; N] = [0.0 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * N] = [0.0 ; N * N];

            for n in 0..N {
                let res4 = sum44[n * N..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                const M : usize = 16;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> 2 * j;
                    let w81 = (bit8 & white) >> 2 * j;
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> 2 * j + 1;
                    let w82 = (bit8 & white) >> 2 * j + 1;
                    bit8 <<= 1;

                    unsafe {
                        let b08 = x86_64::_mm_set_epi64x(b82 as i64, b81 as i64);
                        let w08 = x86_64::_mm_set_epi64x(w82 as i64, w81 as i64);
                        let one = x86_64::_mm_set1_epi8(1);
                        let bm08 = x86_64::_mm_cmpeq_epi8(b08, one);
                        let wm08 = x86_64::_mm_cmpeq_epi8(w08, one);
                        let b16l = x86_64::_mm_unpacklo_epi8(bm08, bm08);
                        let b16h = x86_64::_mm_unpackhi_epi8(bm08, bm08);
                        let w16l = x86_64::_mm_unpacklo_epi8(wm08, wm08);
                        let w16h = x86_64::_mm_unpackhi_epi8(wm08, wm08);
                        let bm1 = x86_64::_mm_unpacklo_epi16(b16l, b16l);
                        let bm2 = x86_64::_mm_unpackhi_epi16(b16l, b16l);
                        let bm3 = x86_64::_mm_unpacklo_epi16(b16h, b16h);
                        let bm4 = x86_64::_mm_unpackhi_epi16(b16h, b16h);
                        let wm1 = x86_64::_mm_unpacklo_epi16(w16l, w16l);
                        let wm2 = x86_64::_mm_unpackhi_epi16(w16l, w16l);
                        let wm3 = x86_64::_mm_unpacklo_epi16(w16h, w16h);
                        let wm4 = x86_64::_mm_unpackhi_epi16(w16h, w16h);
                        let ex1 = x86_64::_mm_or_si128(bm1, wm1);
                        let ex2 = x86_64::_mm_or_si128(bm2, wm2);
                        let ex3 = x86_64::_mm_or_si128(bm3, wm3);
                        let ex4 = x86_64::_mm_or_si128(bm4, wm4);
                        let x41 = x86_64::_mm_load_ps(w1.as_ptr().add(idx));
                        let x42 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 4));
                        let x43 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 8));
                        let x44 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 12));
                        let minus = x86_64::_mm_set1_ps(-0.0);
                        let mn1 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm1), minus);
                        let mn2 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm2), minus);
                        let mn3 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm3), minus);
                        let mn4 = x86_64::_mm_and_ps(x86_64::_mm_castsi128_ps(wm4), minus);
                        let m41 = x86_64::_mm_xor_ps(x41, mn1);
                        let m42 = x86_64::_mm_xor_ps(x42, mn2);
                        let m43 = x86_64::_mm_xor_ps(x43, mn3);
                        let m44 = x86_64::_mm_xor_ps(x44, mn4);
                        let w1 = x86_64::_mm_and_ps(m41, x86_64::_mm_castsi128_ps(ex1));
                        let w2 = x86_64::_mm_and_ps(m42, x86_64::_mm_castsi128_ps(ex2));
                        let w3 = x86_64::_mm_and_ps(m43, x86_64::_mm_castsi128_ps(ex3));
                        let w4 = x86_64::_mm_and_ps(m44, x86_64::_mm_castsi128_ps(ex4));

                        let sum12 = x86_64::_mm_add_ps(w1, w2);
                        let sum34 = x86_64::_mm_add_ps(w3, w4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs[hidx..].as_ptr());
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm_load_ps(wfs[hidx + N_HIDDEN..].as_ptr());
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);
                x86_64::_mm_store_ps(hidden.as_mut_ptr(), h1234);

                if true {  // softsign
                    let sign4 = x86_64::_mm_set1_epi32(i32::MAX);
                    let abs4 = x86_64::_mm_and_ps(h1234, x86_64::_mm_castsi128_ps(sign4));
                    let one = x86_64::_mm_set1_ps(1.0);
                    let abs4p1 = x86_64::_mm_add_ps(abs4, one);
                    let ssgn = x86_64::_mm_div_ps(h1234, abs4p1);

                    // ($x * 0.5 / ($x.abs() + 1.0) + 0.5)  // 0 ~ 1
                    // let half = x86_64::_mm_set1_ps(0.5);
                    // let ssgn05 = x86_64::_mm_mul_ps(half, ssgn);
                    // let ssgnadj = x86_64::_mm_add_ps(ssgn05, half);
                    // let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
                    // let y4 = x86_64::_mm_mul_ps(wh4, ssgnadj);

                    // ($x / ($x.abs() + 1.0))  // -1 ~ 1
                    let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
                    x86_64::_mm_store_ps(hidsig.as_mut_ptr(), ssgn);
                    let y4 = x86_64::_mm_mul_ps(wh4, ssgn);

                    x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
                } else {// sigmoid
                    let emx4 = weight::Weight::expmx_ps_simd(h1234);
                    let one = x86_64::_mm_set1_ps(1.0);
                    let hsp14 = x86_64::_mm_add_ps(emx4, one);
                    let sigm = x86_64::_mm_rcp_ps(hsp14);
                    x86_64::_mm_store_ps(hidsig.as_mut_ptr(), sigm);

                    let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
                    // let y4 = x86_64::_mm_div_ps(wh4, hsp14);
                    let y4 = x86_64::_mm_mul_ps(wh4, sigm);
                    x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
                }
            }
            // for n in 0..N {
            //     sum += sumarr[n];
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    pub fn forwardv3bb(&self, ban : &bitboard::BitBoard)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        self.forwardv1bb(ban)
    }

    pub fn forwardv3bb_simd(&self, ban : &bitboard::BitBoard)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        self.forwardv1bb(ban)
    }

    pub fn forwardv3bb_simdavx(&self, ban : &bitboard::BitBoard)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        self.forwardv1bb(ban)
    }

    pub fn forwardv3bb_simdavx2(&self, ban : &bitboard::BitBoard)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8)) {
        self.forwardv1bb(ban)
    }

    pub fn backwardv1(&mut self,
        ban : &board::Board, winner : i8, eta : f32,
        (hidden , hidsig , output , fs) : &([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8))) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let wh = &mut ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        let deta = diff * eta;
        for i in 0..N_HIDDEN {
            wh[i] -= hidsig[i] * deta;
        }
        wh[N_HIDDEN] -= deta;

        let mut dhid = [0.0 as f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = wh[i] * diff;
            if true {
                // softsign
                // ($x * 0.5 / ($x.abs() + 1.0) + 0.5)  // 0 ~ 1
                // $x.abs() x 0.5 / ($x.abs() + 1.0)^2 + 0.5 / ($x.abs() + 1.0)
                // ($x.abs() x 0.5 + ($x.abs() + 1.0) x 0.5) / ($x.abs() + 1.0)^2
                // ($x.abs() + 0.5) / ($x.abs() + 1.0)^2
                // let abshid = hidden[i].abs();
                // *h = tmp * (abshid + 0.5) / ((abshid + 1.0) * (abshid + 1.0));

                // $x / ($x.abs() + 1.0)  // -1 ~ 1
                // 1 / ($x.abs() + 1.0) + $x.abs() / ($x.abs() + 1.0)^2
                // (2 x $x + 1) / ($x.abs() + 1.0)^2
                let abshid = hidden[i].abs();
                *h = tmp * (abshid * 2.0 + 1.0) / ((abshid + 1.0) * (abshid + 1.0));
            } else {
                // sigmoid
                let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
                *h = tmp * sig * (1.0 - sig);
            }
        }
        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let w1 = &mut ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let heta = *h * eta;
            for (&c, w) in cells.iter().zip(w1.iter_mut()) {
                *w -= c as f32 * heta;
            }
            let wtbn = &mut ow[board::CELL_2D * N_HIDDEN ..];
            wtbn[i] -= teban * heta;
            let wfs = &mut ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
            wfs[i] -= fs.0 as f32 * heta;
            wfs[i + N_HIDDEN] -= fs.1 as f32 * heta;
            let wdc = &mut ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
            wdc[i] -= heta;
        }
    }

    pub fn backwardv1bb(&mut self,
        ban : &bitboard::BitBoard, winner : i8, eta : f32,
        (hidden , hidsig , output , fs) : &([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8))) {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let wh = &mut ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        let deta = diff * eta;
        for i in 0..N_HIDDEN {
            wh[i] -= hidsig[i] * deta;
        }
        wh[N_HIDDEN] -= deta;

        let mut dhid = [0.0 as f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            // tmp = wo x diff
            let tmp = wh[i] * diff;
            if true {
                // softsign
                // ($x * 0.5 / ($x.abs() + 1.0) + 0.5)  // 0 ~ 1
                // $x.abs() x 0.5 / ($x.abs() + 1.0)^2 + 0.5 / ($x.abs() + 1.0)
                // ($x.abs() x 0.5 + ($x.abs() + 1.0) x 0.5) / ($x.abs() + 1.0)^2
                // ($x.abs() + 0.5) / ($x.abs() + 1.0)^2
                // let abshid = hidden[i].abs();
                // *h = tmp * (abshid + 0.5) / ((abshid + 1.0) * (abshid + 1.0));

                // $x / ($x.abs() + 1.0)  // -1 ~ 1
                // 1 / ($x.abs() + 1.0) + $x.abs() / ($x.abs() + 1.0)^2
                // (2 x $x.abs() + 1) / ($x.abs() + 1.0)^2
                let abshid = hidden[i].abs();
                *h = tmp * (abshid * 2.0 + 1.0) / ((abshid + 1.0) * (abshid + 1.0));
                // let _2abshid1 = abshid * 2.0 + 1.0;
                // *h = tmp * _2abshid1 / (abshid * abshid + _2abshid1);
            } else {
                // sigmoid
                // sig = 1 / (1 + exp(-hidden[i]))
                let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
                // h = wo x diff x sig x (1 - sig)
                *h = tmp * sig * (1.0 - sig);
            }
        }

        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let w1 = &mut ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let heta = *h * eta;

            for y in 0..bitboard::NUMCELL {
                let mut bit = bitboard::LSB_CELL << y;
                for x in 0..bitboard::NUMCELL {
                    // let w = w1[x + y * bitboard::NUMCELL];
                    let cb = (black & bit) != 0;
                    let cw = (white & bit) != 0;
                    let diff = if cb {heta} else if cw {-heta} else {0.0};
                    w1[x + y * bitboard::NUMCELL] -= diff;

                    bit <<= bitboard::NUMCELL;
                }
            }

            let wtbn = &mut ow[board::CELL_2D * N_HIDDEN ..];
            wtbn[i] -= teban * heta;
            let wfs = &mut ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
            wfs[i] -= fs.0 as f32 * heta;
            wfs[i + N_HIDDEN] -= fs.1 as f32 * heta;
            let wdc = &mut ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
            wdc[i] -= heta;
        }
    }

    pub fn backwardv3bb(&mut self,
        ban : &bitboard::BitBoard, winner : i8, eta : f32,
        (hidden , hidsig , output , fs) : &([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8))) {
        self.backwardv1bb(ban, winner, eta, &(*hidden , *hidsig , *output , *fs))
    }

    pub fn backwardv1bb_simd(&mut self,
        ban : &bitboard::BitBoard, winner : i8, eta : f32,
        (hidden , hidsig , output , fs) : &([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT], (i8, i8))) {
        self.backwardv1bb(ban, winner, eta, &(*hidden , *hidsig , *output , *fs))
    }

    /// train weights
    /// 
    /// # Arguments
    /// - `self` : self
    /// - `rfen` : RFEN
    /// - `winner` : winner or # of stones.
    /// - `eta` : learning ratio.
    /// - `mid` : last (mid) moves will not be used.
    /// 
    /// # Returns
    /// - OK(()) if succeeded.
    /// - Err(String) if some error happened.
    pub fn train(&mut self, rfen : &str, winner : i8, eta : f32, mid : i8)
             -> Result<(), String> {
        if cfg!(feature="bitboard") {
            let ban = match bitboard::BitBoard::from(rfen) {
                Ok(b) => {b},
                Err(e) => {return Err(e)}
            };
            if ban.count() > 64 - mid {return Ok(());}

            self.learnbb(&ban, winner, eta);

            let ban = ban.rotate180();
            self.learnbb(&ban, winner, eta);
        } else {
            let ban = match board::Board::from(rfen) {
                Ok(b) => {b},
                Err(e) => {return Err(e)}
            };
            if ban.count() > 64 - mid {return Ok(());}

            self.learn(&ban, winner, eta);

            let ban = ban.rotate180();
            self.learn(&ban, winner, eta);
        }
        Ok(())
    }

    fn learn(&mut self, ban : &board::Board, winner : i8, eta : f32) {
        // forward
        let res = self.forwardv1(&ban);
        // backward
        self.backwardv1(ban, winner, eta, &res);
    }

    fn learnbb(&mut self, ban : &bitboard::BitBoard, winner : i8, eta : f32) {
        // forward
        let res = self.forwardv1bb(&ban);
        // backward
        self.backwardv1bb(ban, winner, eta, &res);
    }
}

#[allow(dead_code)]
fn dbg_assert_eq_vec(va : &[f32], vb : &[f32]) -> bool {
    for (a, b) in va.iter().zip(vb.iter()) {
        if (a - b).abs() >= 1.4e-6 {
            println!("| {a} - {b} | >= 1.4e-6...");
            return false;
        }
    }
    true
}

#[test]
fn testweight() {
    let rfens = [
        "h/H/h/H/h/H/h/H b",
        "h/H/h/H/h/H/h/H w",
        "H/h/H/h/H/h/H/h b",
        "H/h/H/h/H/h/H/h w",
        "h/H/8/H/h/H/h/H b",
        "h/H/h/8/h/H/h/H w",
        "H/h/H/h/8/h/H/h b",
        "H/h/H/h/H/8/H/h w",
        "aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa b",
        "aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA w",
        "8/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa b",
        "aAaAaAaA/AaAaAaAa/8/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/8/aAaAaAaA/AaAaAaAa/aAaAaAaA w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/8/aAaAaAaA b",
        "aAaAaAaA/8/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/8/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA w",
        "aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa b",
        "1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa w",
        "AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1 w",
        "A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA b",
        "aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A b",
        "aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA w",
    ];
    for rfen in rfens.iter() {
        for winner in -1..=1 {
            let bban = bitboard::BitBoard::from(rfen).unwrap();
            let ban = board::Board::from(rfen).unwrap();
            ban.put();
            let mut w = weightsoftsign::Weight::new();
            w.init();
            let mut w2 = weightsoftsign::Weight::new();
            w2.copy(&w);
            let mut w3 = weightsoftsign::Weight::new();
            w3.copy(&w);
            let res_nosimde = w.evaluatev3bb(&bban);
            let res_simd = w.evaluatev3bb_simd(&bban);
            let res_simdavx = w.evaluatev3bb_simdavx(&bban);
            assert!((res_nosimde - res_simd).abs() < 1e-6);
            assert!((res_nosimde - res_simdavx).abs() < 1e-6);
            // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
            let (bh_ns, ah_ns, res_nosimd, fsns) = w.forwardv3bb(&bban);
            let (bh_s, ah_s, res_simd, fss) = w.forwardv3bb_simd(&bban);
            let (bh_sa, ah_sa, res_simdavx, fssa)
                    = w.forwardv3bb_simdavx(&bban);
            let (bh_sa2, ah_sa2, res_simdavx2, fssa2)
                    = w.forwardv3bb_simdavx2(&bban);
            assert!(dbg_assert_eq_vec(&bh_ns, &bh_s));
            assert!(dbg_assert_eq_vec(&bh_ns, &bh_sa));
            assert!(dbg_assert_eq_vec(&bh_ns, &bh_sa2));
            // println!("{bh_ns:?} == \n{bh_s:?} == \n{bh_sa:?} ???");
            assert!(dbg_assert_eq_vec(&ah_ns, &ah_s));
            assert!(dbg_assert_eq_vec(&ah_ns, &ah_sa));
            assert!(dbg_assert_eq_vec(&ah_ns, &ah_sa2));
            // println!("{ah_ns:?} == \n{ah_s:?} == \n{ah_sa:?} ???");
            assert!((res_nosimde - res_nosimd[0]).abs() < 1e-6);
            // assert_eq!(res_nosimd, res_simd);
            assert!((res_nosimd[0] - res_simd[0]).abs() < 1e-6);
            // assert_eq!(res_nosimd, res_simdavx);
            assert!((res_nosimd[0] - res_simdavx[0]).abs() < 1e-6);
            // assert_eq!(res_nosimd, res_simdavx2);
            assert!((res_nosimd[0] - res_simdavx2[0]).abs() < 1e-6);
            // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
            assert_eq!(fsns, fss);
            assert_eq!(fsns, fssa);
            assert_eq!(fsns, fssa2);
            // println!("{fsns:?} == {fss:?} == {fssa:?} ???");
            let res = w.forwardv1bb(&bban);
            // let winner = 1;
            let eta = 0.001;
            w.backwardv1bb(&bban, winner, eta, &res);
            w2.backwardv1bb_simd(&bban, winner, eta, &res);
            // let sv = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s = sv.join(",");
            // let sv2 = w2.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s2 = sv2.join(",");
            // assert_eq!(s, s2);
            assert!(dbg_assert_eq_vec(&w.weight, &w2.weight));
            let res = w3.forwardv3(&ban);
            w3.backwardv1(&ban, winner, eta, &res);
            // let sv3 = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s3 = sv3.join(",");
            // assert_eq!(s, s3);
            assert!(dbg_assert_eq_vec(&w.weight, &w3.weight));
            let res_nosimde2 = w.evaluatev1bb(&bban);
            let res_nosimde3 = w2.evaluatev1bb(&bban);
            let res_nosimde4 = w3.evaluatev1bb(&bban);
            // println!("{res_nosimde} -> {res_nosimde2}");
            assert_eq!(res_nosimde2, res_nosimde3);
            assert_eq!(res_nosimde2, res_nosimde4);
            let before = (winner as f32 - res_nosimde).abs();
            assert!(before > (winner as f32 - res_nosimde2).abs());
            // assert!(before > (winner as f32 - res_nosimde3).abs());
            // assert!(before > (winner as f32 - res_nosimde4).abs());
        }
    }
}
