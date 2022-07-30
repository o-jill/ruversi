use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};
use std::arch::x86_64;

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 2(fixedstones) + 1
 * hidden: 4 + 1
 * output: 1
 */
const N_INPUT : usize = board::CELL_2D + 1 + 2;
const N_HIDDEN : usize = 4;
const N_OUTPUT : usize = 1;
const N_WEIGHT: usize = (N_INPUT + 1) * N_HIDDEN + N_HIDDEN + 1;

const WSZV1 : usize = (board::CELL_2D + 1 + 1) * N_HIDDEN + N_HIDDEN + 1;
const WSZV2 : usize = WSZV1;
const WSZV3 : usize = (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN + N_HIDDEN + 1;

// v2
// 8/8/1A6/2Ab3/2C3/8/8/8 w
// val:-273.121 val:Some(-273.1215), 268965 nodes. []b6@@b5[]c6@@a7[]a5@@a6[]a8 60msec
// 8/8/1A6/2Ab3/2aB3/1a6/8/8 b
// val:-3.506 val:Some(-3.5055861), 334278 nodes. @@c3[]c2@@d1[]c1@@b1[]a4@@a2 80msec

#[derive(PartialEq)]
enum EvalFile{
    Unknown,
    V1,
    V2,
    V3,
}

impl EvalFile {
    pub fn to_str(&self) -> &str {
        match self {
            EvalFile::Unknown => {"unknown eval file format."},
            EvalFile::V1 => {"# 65-4-1"},
            EvalFile::V2 => {"# 64+1-4-1"},
            EvalFile::V3 => {"# 64+1+2-4-1"},
        }
    }

    pub fn from(txt : &str) -> Option<EvalFile> {
        match txt {
            "# 65-4-1" => Some(EvalFile::V1),
            "# 64+1-4-1" => Some(EvalFile::V2),
            "# 64+1+2-4-1" => Some(EvalFile::V3),
            _ => None
        }
    }
}

pub struct Weight {
    weight : Vec<f32>
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

    pub fn read(&mut self, path : &str) -> Result<(), String> {
        let mut format = EvalFile::Unknown;
        let file = File::open(path).unwrap();
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
                        EvalFile::V2 => {return self.readv2(&l)},
                        EvalFile::V3 => {return self.readv3(&l)},
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
        if cfg!(feature="nnv1") {
            self.weight = newtable;
        } else if cfg!(feature="nnv2") {
            self.fromv1tov2(&newtable);
            // println!("self.fromv1tov2(&newtable);");
        } else {
            self.fromv1tov3(&newtable);
            // println!("self.fromv1tov3(&newtable);");
        }
        // println!("v1:{:?}", self.weight);
        Ok(())
    }

    fn readv2(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let wsz = self.weight.len();
        let nsz = newtable.len();
        if WSZV2 != nsz {
            return Err(String::from("size mismatch"));
        }
        if cfg!(feature="nnv2") {
            self.weight = newtable;
        } else {
            self.fromv2tov3(&newtable);
            // println!("self.fromv2tov3(&newtable);");
        }
        // println!("v2:{:?}", self.weight);
        Ok(())
    }

    fn readv3(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let wsz = self.weight.len();
        let nsz = newtable.len();
        if WSZV3 != nsz {
            return Err(String::from("size mismatch"));
        }
        self.weight = newtable;
        // println!("v3:{:?}", self.weight);
        Ok(())
    }

    fn write(f : File, w : &Vec<f32>, ver : &EvalFile) {
        let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        f.write(format!("{}\n", ver.to_str()).as_bytes()).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
    }
    pub fn writev1(&self, path : &str) {
        let f = fs::File::create(path).unwrap();
        Weight::write(f, &self.weight, &EvalFile::V1);
    }

    pub fn writev2(&self, path : &str) {
        let f = fs::File::create(path).unwrap();
        Weight::write(f, &self.weight, &EvalFile::V2);
    }

    pub fn writev3(&self, path : &str) {
        let f = fs::File::create(path).unwrap();
        Weight::write(f, &self.weight, &EvalFile::V3);
    }

    pub fn writev1asv2(&self, path : &str) {
        let mut w = Weight::new();
        w.fromv1tov2(&self.weight);
        let f = fs::File::create(path).unwrap();
        Weight::write(f, &self.weight, &EvalFile::V2);
    }

    pub fn copy(&mut self, src : &Weight) {
        for (d, s) in self.weight.iter_mut().zip(src.weight.iter()) {
            *d = *s;
        }
    }

    fn fromv1tov2(&mut self, tbl : &Vec<f32>) {
        // ban
        for i in 0..N_HIDDEN {
            let we = &mut self.weight[i * board::CELL_2D..(i + 1) * board::CELL_2D];
            let tb = &tbl[i * (board::CELL_2D + 1 + 1)..(i + 1) * (board::CELL_2D + 1 + 1)];
            for (w, t) in we.iter_mut().zip(tb.iter()) {
                *w = *t;
            }
            let teb = &mut self.weight[
                N_HIDDEN * board::CELL_2D + i..=N_HIDDEN * board::CELL_2D + N_HIDDEN * 2 + i];
            // teban
            teb[0] = tbl[i * (board::CELL_2D + 1 + 1) + board::CELL_2D];
            // dc
            teb[N_HIDDEN] = tbl[i * (board::CELL_2D + 1 + 1) + board::CELL_2D + 1];
            // hidden
            teb[N_HIDDEN * 2] = tbl[4 * (board::CELL_2D + 1 + 1) + i];
        }
        // dc
        *self.weight.last_mut().unwrap() = *tbl.last().unwrap();
    }

    fn fromv1tov3(&mut self, tbl : &Vec<f32>) {
        // ban
        for i in 0..N_HIDDEN {
            let we = &mut self.weight[i * board::CELL_2D..(i + 1) * board::CELL_2D];
            let tb = &tbl[i * (board::CELL_2D + 1 + 1)..(i + 1) * (board::CELL_2D + 1 + 1)];
            for (w, t) in we.iter_mut().zip(tb.iter()) {
                *w = *t;
            }
            let teb = &mut self.weight[N_HIDDEN * board::CELL_2D + i..];
            // teban
            teb[0] = tbl[i * (board::CELL_2D + 1 + 1) + board::CELL_2D];
            // fixed stone
            teb[N_HIDDEN] = 0.0;
            teb[N_HIDDEN * 2] = 0.0;
            // dc
            teb[N_HIDDEN * 3] = tbl[i * (board::CELL_2D + 1 + 1) + board::CELL_2D + 1];
            // hidden
            teb[N_HIDDEN * 4] = tbl[4 * (board::CELL_2D + 1 + 1) + i];
        }
        // dc
        *self.weight.last_mut().unwrap() = *tbl.last().unwrap();
    }

    fn fromv2tov3(&mut self, tbl : &Vec<f32>) {
        // ban + teban
        let we = &mut self.weight[0..N_HIDDEN * (board::CELL_2D + 1)];
        let tb = &tbl[0 .. N_HIDDEN * (board::CELL_2D + 1)];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // fixed stone
        let we = &mut self.weight[N_HIDDEN * (board::CELL_2D + 1) .. N_HIDDEN * (board::CELL_2D + 1 + 2)];
        we.fill(0.0);

        // dc + w2 + dc2
        let we = &mut self.weight[N_HIDDEN * (board::CELL_2D + 1 + 2)..];
        let dcw2 = &tbl[N_HIDDEN * (board::CELL_2D + 1)..];
        for (w, t) in we.iter_mut().zip(dcw2.iter()) {
            *w = *t;
        }
    }

    pub fn evaluatev1(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            for (&w, &c) in w1.iter().zip(cells.iter()) {
                hidsum += w * c as f32;
            }
            hidsum += teban * w1[w1sz - 2];
            sum += w2[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn evaluatev1_simd(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            // for (idx, c)  in cells.iter().enumerate() {
            //     hidsum += *c as f32 * w1[idx];
            // }
            // let mut sum4 = f32x4::splat(0.0);
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
            }
            for i in 0..board::CELL_2D / 4 {
                // let x4 = f32x4::load(w1[i + 4], 4);
                // let y4 = f32x4::new(cells[i * 4], cells[i * 4 + 1], cells[i * 4 + 2], cells[i * 4 + 3]);
                // sum4 += x4 * y4;
                let idx = i * 4;
                unsafe {
                    let x4 = x86_64::_mm_loadu_ps(w1[idx..].as_ptr());
                    // let y4 = x86_64::_mm_set_ps(
                    //     cells[idx] as f32, cells[idx + 1] as f32,
                    //     cells[idx + 2] as f32, cells[idx + 3] as f32);
                    let y4 = x86_64::_mm_set_epi32(
                        cells[idx + 3] as i32, cells[idx + 2] as i32,
                        cells[idx + 1] as i32, cells[idx + 0] as i32);
                    // let y4 = x86_64::_mm_set_epi32(
                    //     cells[idx] as i32, cells[idx + 1] as i32,
                    //     cells[idx + 2] as i32, cells[idx + 3] as i32);
                    let y4 = x86_64::_mm_cvtepi32_ps(y4);
                    let mul = x86_64::_mm_mul_ps(x4, y4);
                    sum4 = x86_64::_mm_add_ps(sum4, mul);
                }
            }
            let mut sumarr : [f32 ; 4] = [0.0 ; 4];
            unsafe {
                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
                // x86_64::_mm_store_ps(sumarr.as_mut_ptr(),
                //     x86_64::_mm_hadd_ps(sum4, sum4));
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            // hidsum += sumarr[0] + sumarr[2];
            hidsum += teban * w1[w1sz - 2];
            sum += w2[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn evaluatev2(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = dc[i];
            for (idx, c)  in cells.iter().enumerate() {
                hidsum += *c as f32 * w1[idx];
            }
            hidsum += teban * tbn[i];
            sum += w2[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn evaluatev2_simd(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = dc[i];
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
            }
            for j in 0..board::CELL_2D / 16 {
                let idx = j * 16;
                unsafe {
                    let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                    let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                    let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                    let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                    let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                    let zero = x86_64::_mm_setzero_si128();
                    // to i16
                    let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                    let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                    let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                    // to i32
                    let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                    let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                    let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                    let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                    let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                    let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                    let f41 = x86_64::_mm_cvtepi32_ps(c41);
                    let f42 = x86_64::_mm_cvtepi32_ps(c42);
                    let f43 = x86_64::_mm_cvtepi32_ps(c43);
                    let f44 = x86_64::_mm_cvtepi32_ps(c44);

                    let mul1 = x86_64::_mm_mul_ps(x41, f41);
                    let mul2 = x86_64::_mm_mul_ps(x42, f42);
                    let mul3 = x86_64::_mm_mul_ps(x43, f43);
                    let mul4 = x86_64::_mm_mul_ps(x44, f44);

                    let sum12 = x86_64::_mm_add_ps(mul1, mul2);
                    let sum34 = x86_64::_mm_add_ps(mul3, mul4);
                    let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                    sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                }
            }
            let mut sumarr : [f32 ; 4] = [0.0, 0.0, 0.0, 0.0];
            unsafe {
                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            hidsum += teban * tbn[i];
            sum += w2[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    /**
     * exp(-x)
     */
    fn expmx_ps( x : *const f32, y : *mut f32) {
        let exp_hi : f32 = 88.3762626647949;
        let exp_lo : f32 = -exp_hi;

        let cephes_log2ef : f32 = 1.44269504088896341;
        let cephes_exp_c1 : f32 = 0.693359375;
        let cephes_exp_c2 : f32 = -2.12194440e-4;

        let cephes_exp_p0 : f32 = 1.9875691500E-4;
        let cephes_exp_p1 : f32 = 1.3981999507E-3;
        let cephes_exp_p2 : f32 = 8.3334519073E-3;
        let cephes_exp_p3 : f32 = 4.1665795894E-2;
        let cephes_exp_p4 : f32 = 1.6666665459E-1;
        let cephes_exp_p5 : f32 = 5.0000001201E-1;
        unsafe {
            let x4 = x86_64::_mm_load_ps(x);
            // clip x
            let max4 = x86_64::_mm_set1_ps(exp_hi);
            let x4 = x86_64::_mm_min_ps(x4, max4);
            let min4 = x86_64::_mm_set1_ps(exp_lo);
            let x4 = x86_64::_mm_max_ps(x4, min4);
            let m1 = x86_64::_mm_set1_ps(-1.0);
            let x4 = x86_64::_mm_mul_ps(x4, m1);

            /* express exp(x) as exp(g + n*log(2)) */
            let log2ef = x86_64::_mm_set1_ps(cephes_log2ef);
            let fx = x86_64::_mm_mul_ps(x4, log2ef);
            let zp5 = x86_64::_mm_set1_ps(cephes_exp_p5);
            let fx = x86_64::_mm_add_ps(fx, zp5);
            let emm0 = x86_64::_mm_cvtps_epi32(fx);
            let tmp = x86_64::_mm_cvtepi32_ps(emm0);

            let mask = x86_64::_mm_cmpgt_ps(tmp, fx);
            let one = x86_64::_mm_set1_ps(1.0);
            let mask = x86_64::_mm_and_ps(mask, one);
            let fx = x86_64::_mm_sub_ps(tmp, mask);

            let c1 = x86_64::_mm_set1_ps(cephes_exp_c1);
            let tmp = x86_64::_mm_mul_ps(fx, c1);
            let c2 = x86_64::_mm_set1_ps(cephes_exp_c2);
            let z4 = x86_64::_mm_mul_ps(fx, c2);
            let x4 = x86_64::_mm_sub_ps(x4, tmp);
            let x4 = x86_64::_mm_sub_ps(x4, z4);

            let z4 = x86_64::_mm_mul_ps(x4, x4);

            let y4 = x86_64::_mm_set1_ps(cephes_exp_p0);
            let y4 = x86_64::_mm_mul_ps(y4, x4);
            let exp_p1 = x86_64::_mm_set1_ps(cephes_exp_p1);
            let y4 = x86_64::_mm_add_ps(y4, exp_p1);
            let y4 = x86_64::_mm_mul_ps(y4, x4);
            let exp_p2 = x86_64::_mm_set1_ps(cephes_exp_p2);
            let y4 = x86_64::_mm_add_ps(y4, exp_p2);
            let y4 = x86_64::_mm_mul_ps(y4, x4);
            let exp_p3 = x86_64::_mm_set1_ps(cephes_exp_p3);
            let y4 = x86_64::_mm_add_ps(y4, exp_p3);
            let y4 = x86_64::_mm_mul_ps(y4, x4);
            let exp_p4 = x86_64::_mm_set1_ps(cephes_exp_p4);
            let y4 = x86_64::_mm_add_ps(y4, exp_p4);
            let y4 = x86_64::_mm_mul_ps(y4, x4);
            let exp_p5 = x86_64::_mm_set1_ps(cephes_exp_p5);
            let y4 = x86_64::_mm_add_ps(y4, exp_p5);
            let y4 = x86_64::_mm_mul_ps(y4, z4);
            let y4 = x86_64::_mm_add_ps(y4, x4);
            let y4 = x86_64::_mm_add_ps(y4, one);

            let emm0 = x86_64::_mm_cvttps_epi32(fx);
            let _pi32_0x7f = x86_64::_mm_set1_epi32(0x7f);
            let emm0 = x86_64::_mm_add_epi32(emm0, _pi32_0x7f);
            let emm0 = x86_64::_mm_slli_epi32(emm0, 23);
            let pow2n = x86_64::_mm_castsi128_ps(emm0);

            let y4 = x86_64::_mm_mul_ps(y4, pow2n);
            x86_64::_mm_store_ps(y, y4);
        }
    }

    pub fn evaluatev2_simd2(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];

        let mut hidsum : [f32 ; 4] = [0.0 ; 4];
        let mut emx : [f32 ; 4] = [0.0 ; 4];
        let mut sumarr : [f32 ; 4] = [0.0 ; 4];

        for i in 0..N_HIDDEN / 4 {
            let hidx = i * 4;
            let mut sum44 : [f32 ; 4 * 4] = [0.0 ; 4 * 4];

            for n in 0..4 {
                let res4 = sum44[n * 4..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                for j in 0..board::CELL_2D / 16 {
                    let idx = j * 16;
                    unsafe {
                        let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                        let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                        let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                        let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                        let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                        let zero = x86_64::_mm_setzero_si128();
                        // to i16
                        let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                        let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                        let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                        // to i32
                        let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                        let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                        let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                        let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                        let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                        let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                        let f41 = x86_64::_mm_cvtepi32_ps(c41);
                        let f42 = x86_64::_mm_cvtepi32_ps(c42);
                        let f43 = x86_64::_mm_cvtepi32_ps(c43);
                        let f44 = x86_64::_mm_cvtepi32_ps(c44);

                        let mul1 = x86_64::_mm_mul_ps(x41, f41);
                        let mul2 = x86_64::_mm_mul_ps(x42, f42);
                        let mul3 = x86_64::_mm_mul_ps(x43, f43);
                        let mul4 = x86_64::_mm_mul_ps(x44, f44);

                        let sum12 = x86_64::_mm_add_ps(mul1, mul2);
                        let sum34 = x86_64::_mm_add_ps(mul3, mul4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());
                // _mm_transpose_ps
                let tmp0   = x86_64::_mm_shuffle_ps(x1, x2, 0x44);
                let tmp2   = x86_64::_mm_shuffle_ps(x1, x2, 0xEE);
                let tmp1   = x86_64::_mm_shuffle_ps(x3, x4, 0x44);
                let tmp3   = x86_64::_mm_shuffle_ps(x3, x4, 0xEE);
                let h1 = x86_64::_mm_shuffle_ps(tmp0, tmp1, 0x88);
                let h2 = x86_64::_mm_shuffle_ps(tmp0, tmp1, 0xDD);
                let h3 = x86_64::_mm_shuffle_ps(tmp2, tmp3, 0x88);
                let h4 = x86_64::_mm_shuffle_ps(tmp2, tmp3, 0xDD);

                let h12 = x86_64::_mm_add_ps(h1, h2);
                let h34 = x86_64::_mm_add_ps(h3, h4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(tbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // dc
                let dc4 = x86_64::_mm_load_ps(dc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, dc4);
                x86_64::_mm_store_ps(hidsum.as_mut_ptr(), h1234);
            }
            Weight::expmx_ps(hidsum.as_ptr(), emx.as_mut_ptr());
            unsafe {
                let emx4 = x86_64::_mm_load_ps(emx.as_ptr());
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let w24 = x86_64::_mm_load_ps(w2[hidx..].as_ptr());

                let y4 = x86_64::_mm_div_ps(w24, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
            }
            // for n in 0..4 {
            //     sum += sumarr[n];
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    pub fn evaluatev3(&self, ban : &board::Board) -> f32 {
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
            sum += wh[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn evaluatev3_simd(&self, ban : &board::Board) -> f32 {
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 4;
        let mut hidsum : [f32 ; N] = [0.0 ; N];
        let mut emx : [f32 ; N] = [0.0 ; N];
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
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    unsafe {
                        let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                        let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                        let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                        let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                        let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                        let zero = x86_64::_mm_setzero_si128();
                        // to i16
                        let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                        let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                        let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                        // to i32
                        let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                        let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                        let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                        let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                        let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                        let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                        let f41 = x86_64::_mm_cvtepi32_ps(c41);
                        let f42 = x86_64::_mm_cvtepi32_ps(c42);
                        let f43 = x86_64::_mm_cvtepi32_ps(c43);
                        let f44 = x86_64::_mm_cvtepi32_ps(c44);

                        let mul1 = x86_64::_mm_mul_ps(x41, f41);
                        let mul2 = x86_64::_mm_mul_ps(x42, f42);
                        let mul3 = x86_64::_mm_mul_ps(x43, f43);
                        let mul4 = x86_64::_mm_mul_ps(x44, f44);

                        let sum12 = x86_64::_mm_add_ps(mul1, mul2);
                        let sum34 = x86_64::_mm_add_ps(mul3, mul4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());
                // _mm_transpose_ps
                let tmp0   = x86_64::_mm_shuffle_ps(x1, x2, 0x44);
                let tmp2   = x86_64::_mm_shuffle_ps(x1, x2, 0xEE);
                let tmp1   = x86_64::_mm_shuffle_ps(x3, x4, 0x44);
                let tmp3   = x86_64::_mm_shuffle_ps(x3, x4, 0xEE);
                let h1 = x86_64::_mm_shuffle_ps(tmp0, tmp1, 0x88);
                let h2 = x86_64::_mm_shuffle_ps(tmp0, tmp1, 0xDD);
                let h3 = x86_64::_mm_shuffle_ps(tmp2, tmp3, 0x88);
                let h4 = x86_64::_mm_shuffle_ps(tmp2, tmp3, 0xDD);

                let h12 = x86_64::_mm_add_ps(h1, h2);
                let h34 = x86_64::_mm_add_ps(h3, h4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                // let wfsb4 = x86_64::_mm_load_ps(wfs[hidx..].as_ptr());
                // let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                // let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                // let wfsw4 = x86_64::_mm_load_ps(wfs[hidx + N_HIDDEN..].as_ptr());
                // let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                // let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                // let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                // let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);
                x86_64::_mm_store_ps(hidsum.as_mut_ptr(), h1234);
            }
            Weight::expmx_ps(hidsum.as_ptr(), emx.as_mut_ptr());
            unsafe {
                let emx4 = x86_64::_mm_load_ps(emx.as_ptr());
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
            // for n in 0..N {
            //     sum += sumarr[n];
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    pub fn forwardv1(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            for (&w, &c) in w1.iter().zip(cells.iter()) {
                hidsum += w * c as f32;
            }
            hidsum += teban * w1[w1sz - 2];
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += w2[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output)
    }

    pub fn forwardv1_simd(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            // let mut hidsum2 : f32 = 0.0;//*w1.last().unwrap();
            // let mut hidsum3 : f32 = 0.0;
            // let mut hidsum4 : f32 = 0.0;
            // let mut hidsum5 : f32 = 0.0;
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
            }
            // for (idx, c)  in cells.iter().enumerate() {
            //     hidsum2 += *c as f32 * w1[idx];
            // }
            // for j in 0..board::CELL_2D / 4 {
            //     hidsum2 += cells[j * 4] as f32 * w1[j * 4];
            //     hidsum3 += cells[j * 4 + 1] as f32 * w1[j * 4 + 1];
            //     hidsum4 += cells[j * 4 + 2] as f32 * w1[j * 4 + 2];
            //     hidsum5 += cells[j * 4 + 3] as f32 * w1[j * 4 + 3];
            // }
            // hidsum2 += hidsum3 + hidsum4 + hidsum5 + *w1.last().unwrap();
            for j in 0..board::CELL_2D / 4 {
                    let idx = j * 4;
                unsafe {
                    let x4 = x86_64::_mm_loadu_ps(w1[idx..].as_ptr());

                    let c4 = x86_64::_mm_set_epi32(
                        cells[idx + 3] as i32, cells[idx + 2] as i32,
                        cells[idx + 1] as i32, cells[idx + 0] as i32);
                    let c4 = x86_64::_mm_cvtepi32_ps(c4);

                    let mul = x86_64::_mm_mul_ps(x4, c4);

                    sum4 = x86_64::_mm_add_ps(sum4, mul);
                }
            }
            let mut sumarr : [f32 ; 4] = [0.0 ; 4];
            unsafe {
                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
                // x86_64::_mm_store_ps(sumarr.as_mut_ptr(),
                //     x86_64::_mm_hadd_ps(sum4, sum4));
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            // if f32::abs(hidsum - hidsum2) > 0.001 {
            //     println!("{} - {} > 0.001", hidsum, hidsum2);
            //     panic!("diffffffffffff");
            // }
            hidsum += teban * w1[w1sz - 2];
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += w2[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output)
    }

    pub fn forwardv2(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = dc[i];
            for (&w, &c) in w1.iter().zip(cells.iter()) {
                hidsum += w * c as f32;
            }
            hidsum += teban * tbn[i];
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += w2[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output)
    }

    pub fn forwardv2_simd(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let mut sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = dc[i];
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
            }
            for j in 0..board::CELL_2D / 16 {
                let idx = j * 16;
                unsafe {
                    let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                    let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                    let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                    let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                    let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                    let zero = x86_64::_mm_setzero_si128();
                    // to i16
                    let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                    let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                    let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                    // to i32
                    let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                    let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                    let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                    let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                    let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                    let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                    let f41 = x86_64::_mm_cvtepi32_ps(c41);
                    let f42 = x86_64::_mm_cvtepi32_ps(c42);
                    let f43 = x86_64::_mm_cvtepi32_ps(c43);
                    let f44 = x86_64::_mm_cvtepi32_ps(c44);

                    let mul1 = x86_64::_mm_mul_ps(x41, f41);
                    let mul2 = x86_64::_mm_mul_ps(x42, f42);
                    let mul3 = x86_64::_mm_mul_ps(x43, f43);
                    let mul4 = x86_64::_mm_mul_ps(x44, f44);

                    let sum12 = x86_64::_mm_add_ps(mul1, mul2);
                    let sum34 = x86_64::_mm_add_ps(mul3, mul4);
                    let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                    sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                }
            }

            let mut sumarr : [f32 ; 4] = [0.0 ; 4];
            unsafe {
                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            hidsum += teban * tbn[i];
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += w2[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output)
    }

    pub fn forwardv2_simd2(&self, ban : &board::Board)
        -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];

        let mut emx : [f32 ; 4] = [0.0 ; 4];
        let mut sumarr : [f32 ; 4] = [0.0 ; 4];

        for i in 0..N_HIDDEN / 4 {
            let hidx = i * 4;
            let mut sum44 : [f32 ; 4 * 4] = [0.0 ; 4 * 4];

            for n in 0..4 {
                let res4 = sum44[n * 4..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                for j in 0..board::CELL_2D / 16 {
                    let idx = j * 16;
                    unsafe {
                        let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                        let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                        let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                        let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                        let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                        let zero = x86_64::_mm_setzero_si128();
                        // to i16
                        let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                        let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                        let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                        // to i32
                        let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                        let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                        let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                        let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                        let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                        let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                        let f41 = x86_64::_mm_cvtepi32_ps(c41);
                        let f42 = x86_64::_mm_cvtepi32_ps(c42);
                        let f43 = x86_64::_mm_cvtepi32_ps(c43);
                        let f44 = x86_64::_mm_cvtepi32_ps(c44);

                        let mul1 = x86_64::_mm_mul_ps(x41, f41);
                        let mul2 = x86_64::_mm_mul_ps(x42, f42);
                        let mul3 = x86_64::_mm_mul_ps(x43, f43);
                        let mul4 = x86_64::_mm_mul_ps(x44, f44);

                        let sum12 = x86_64::_mm_add_ps(mul1, mul2);
                        let sum34 = x86_64::_mm_add_ps(mul3, mul4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            let hidsum = hidden[hidx..hidx + 4].as_mut_ptr();
            unsafe {
                let x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());
                // _mm_transpose_ps
                let tmp0   = x86_64::_mm_shuffle_ps(x1, x2, 0x44);
                let tmp2   = x86_64::_mm_shuffle_ps(x1, x2, 0xEE);
                let tmp1   = x86_64::_mm_shuffle_ps(x3, x4, 0x44);
                let tmp3   = x86_64::_mm_shuffle_ps(x3, x4, 0xEE);
                let h1 = x86_64::_mm_shuffle_ps(tmp0, tmp1, 0x88);
                let h2 = x86_64::_mm_shuffle_ps(tmp0, tmp1, 0xDD);
                let h3 = x86_64::_mm_shuffle_ps(tmp2, tmp3, 0x88);
                let h4 = x86_64::_mm_shuffle_ps(tmp2, tmp3, 0xDD);

                let h12 = x86_64::_mm_add_ps(h1, h2);
                let h34 = x86_64::_mm_add_ps(h3, h4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(tbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // dc
                let dc4 = x86_64::_mm_load_ps(dc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, dc4);
                x86_64::_mm_store_ps(hidsum, h1234);
            }
            Weight::expmx_ps(hidsum, emx.as_mut_ptr());
            unsafe {
                let emx4 = x86_64::_mm_load_ps(emx.as_ptr());
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let sig4 = x86_64::_mm_div_ps(one, hsp14);
                x86_64::_mm_store_ps(hidsig[hidx..].as_mut_ptr(), sig4);
                let w24 = x86_64::_mm_load_ps(w2[hidx..].as_ptr());
                let y4 = x86_64::_mm_mul_ps(w24, sig4);
                // let y4 = x86_64::_mm_div_ps(w24, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
            }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        output[0] = sum;
        (hidden, hidsig, output)
    }

    pub fn train(&mut self, rfen : &str, winner : i8, eta : f32) -> Result<(), String> {
        let ban = board::Board::from(rfen).unwrap();
        self.learn(&ban, winner, eta);

        let ban = ban.rotate180();
        self.learn(&ban, winner, eta);
        Ok(())
    }

    fn backwordv1(&mut self,
            ban : &board::Board, winner : i8, eta : f32,
            hidden : &[f32 ; N_HIDDEN], hidsig : &[f32 ; N_HIDDEN], output : &[f32 ; N_OUTPUT]) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - 10.0 * winner as f32;
        let w2 = &mut ow[w1sz * 4..];
        for i in 0..N_HIDDEN {
            w2[i] -= hidsig[i] * diff * eta;
        }
        w2[N_HIDDEN] -= diff * eta;

        let mut dhid = [0.0 as f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = w2[i] * diff;
            let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
            *h = tmp * sig * (1.0 - sig);
        }
        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let w1 = &mut ow[i * w1sz .. (i + 1) * w1sz];
            let heta = *h * eta;
            if cfg!(feature="nosimd") {
                for (&c, w) in cells.iter().zip(w1.iter_mut()) {
                    *w -= c as f32 * heta;
                }
            } else {
                let heta4: x86_64::__m128;
                unsafe {
                    heta4 = x86_64::_mm_set1_ps(*h * eta);
                }
                for j in 0..board::CELL_2D / 4 {
                    let idx = j * 4;
                    unsafe {
                        let y4 = x86_64::_mm_set_epi32(
                            cells[idx + 3] as i32, cells[idx + 2] as i32,
                            cells[idx + 1] as i32, cells[idx + 0] as i32);
                        let y4 = x86_64::_mm_cvtepi32_ps(y4);
                        let diff4 = x86_64::_mm_mul_ps(heta4, y4);

                        let x4 = x86_64::_mm_loadu_ps(w1[idx..].as_ptr());
                        let w4 = x86_64::_mm_sub_ps(x4, diff4);
                        x86_64::_mm_storeu_ps(w1[idx..].as_mut_ptr(), w4);
                    }
                }
            }
            w1[board::CELL_2D] -= teban * heta;
            w1[board::CELL_2D + 1] -= heta;
        }        
    }

    fn backwordv2(&mut self,
        ban : &board::Board, winner : i8, eta : f32,
        hidden : &[f32 ; N_HIDDEN], hidsig : &[f32 ; N_HIDDEN], output : &[f32 ; N_OUTPUT]) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - 10.0 * winner as f32;
        let w2 = &mut ow[(board::CELL_2D + 2) * N_HIDDEN ..];
        let deta = diff * eta;
        // if cfg!(feature="nosimd") {
            for i in 0..N_HIDDEN {
                w2[i] -= hidsig[i] * deta;
            }
        // } else {
        // slow for N_HIDDEN:4
        //     for i in 0..N_HIDDEN / 4 {
        //         let hidx = i * 4;
        //         unsafe {
        //             let w4 = x86_64::_mm_load_ps(w2[hidx..].as_ptr());
        //             let h4 = x86_64::_mm_load_ps(w2[hidx..].as_ptr());
        //             let deta4 = x86_64::_mm_set1_ps(deta);
        //             let hdeta = x86_64::_mm_mul_ps(deta4, h4);
        //             let y4 = x86_64::_mm_sub_ps(w4, hdeta);
        //             x86_64::_mm_storeu_ps(w2[hidx..].as_mut_ptr(), y4);
        //         }
        //     }
        // }
        w2[N_HIDDEN] -= deta;

        let mut dhid = [0.0 as f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = w2[i] * diff;
            let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
            *h = tmp * sig * (1.0 - sig);
        }
        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let w1 = &mut ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let heta = *h * eta;
            if cfg!(feature="nosimd") {
                for (&c, w) in cells.iter().zip(w1.iter_mut()) {
                    *w -= c as f32 * heta;
                }
            } else {
                let heta4: x86_64::__m128;
                unsafe {
                    heta4 = x86_64::_mm_set1_ps(*h * eta);
                }
                for j in 0..board::CELL_2D / 16 {
                    let idx = j * 16;
                    unsafe {
                        let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                        let zero = x86_64::_mm_setzero_si128();
                        // to i16
                        let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                        let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                        let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                        // to i32
                        let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                        let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                        let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                        let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                        let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                        let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                        let f41 = x86_64::_mm_cvtepi32_ps(c41);
                        let f42 = x86_64::_mm_cvtepi32_ps(c42);
                        let f43 = x86_64::_mm_cvtepi32_ps(c43);
                        let f44 = x86_64::_mm_cvtepi32_ps(c44);

                        let diff41 = x86_64::_mm_mul_ps(heta4, f41);
                        let diff42 = x86_64::_mm_mul_ps(heta4, f42);
                        let diff43 = x86_64::_mm_mul_ps(heta4, f43);
                        let diff44 = x86_64::_mm_mul_ps(heta4, f44);

                        let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                        let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                        let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                        let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());
                        let w41 = x86_64::_mm_sub_ps(x41, diff41);
                        let w42 = x86_64::_mm_sub_ps(x42, diff42);
                        let w43 = x86_64::_mm_sub_ps(x43, diff43);
                        let w44 = x86_64::_mm_sub_ps(x44, diff44);
                        x86_64::_mm_store_ps(w1[idx..].as_mut_ptr(), w41);
                        x86_64::_mm_store_ps(w1[idx + 4..].as_mut_ptr(), w42);
                        x86_64::_mm_store_ps(w1[idx + 8..].as_mut_ptr(), w43);
                        x86_64::_mm_store_ps(w1[idx + 12..].as_mut_ptr(), w44);
                    }
                }
            }
            let tbndc = &mut ow[board::CELL_2D * N_HIDDEN ..];
            tbndc[i] -= teban * heta;
            tbndc[i + N_HIDDEN] -= heta;
        }
    }

    fn learn(&mut self, ban : &board::Board, winner : i8, eta : f32) {
        // forward
        let (hidden, hidsig, output) = 
            if cfg!(feature="nnv1") {
                if cfg!(feature="nosimd") {
                    self.forwardv1(&ban)
                } else {
                    self.forwardv1_simd(&ban)
                }
            } else {
                if cfg!(feature="nosimd") {
                    self.forwardv2(&ban)
                } else {
                    self.forwardv2_simd(&ban)
                    // self.forwardv2_simd2(&ban)
                }
            };
        // backword
        if cfg!(feature="nnv1") {
            self.backwordv1(ban, winner, eta, &hidden, &hidsig, &output);
        } else {
            self.backwordv2(ban, winner, eta, &hidden, &hidsig, &output);
        }
    }
}
