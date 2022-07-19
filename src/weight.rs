use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};
use std::arch::x86_64;

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
const N_INPUT : usize = board::CELL_2D + 1;
const N_HIDDEN : usize = 4;
const N_OUTPUT : usize = 1;
const N_WEIGHT: usize = (N_INPUT + 1) * N_HIDDEN + N_HIDDEN + 1;

#[derive(PartialEq)]
enum EvalFile{
    Unknown,
    V1,
    V2,
}

impl EvalFile {
    pub fn to_str(&self) -> &str {
        match self {
            EvalFile::Unknown => {"unknown eval file format."},
            EvalFile::V1 => {"# 65-4-1"},
            EvalFile::V2 => {"# 64+1-4-1"},
        }
    }

    pub fn from(txt : &str) -> Option<EvalFile> {
        match txt {
            "# 65-4-1" => Some(EvalFile::V1),
            "# 64+1-4-1" => Some(EvalFile::V2),
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
        let wsz = self.weight.len();
        let nsz = newtable.len();
        if wsz != nsz {
            return Err(String::from("size mismatch"));
        }
        if cfg!(feature="nnv1") {
            self.weight = newtable;
        } else {
            self.fromv1(&newtable);
            // println!("self.fromv1(&newtable);");
        }
        Ok(())
    }

    fn readv2(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let wsz = self.weight.len();
        let nsz = newtable.len();
        if wsz != nsz {
            return Err(String::from("size mismatch"));
        }
        self.weight = newtable;
        Ok(())
    }

    pub fn writev1(&self, path : &str) {
        let sv = self.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        let mut f = fs::File::create(path).unwrap();
        // f.write(format!("# {}-{}-{}\n", N_INPUT, N_HIDDEN, N_OUTPUT).as_bytes()).unwrap();
        f.write(format!("{}\n", EvalFile::V1.to_str()).as_bytes()).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
    }

    pub fn writev2(&self, path : &str) {
        let sv = self.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        let mut f = fs::File::create(path).unwrap();
        f.write(format!("{}\n", EvalFile::V2.to_str()).as_bytes()).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
    }

    pub fn writev1asv2(&self, path : &str) {
        let mut w = Weight::new();
        w.fromv1(&self.weight);
        let sv = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        let mut f = fs::File::create(path).unwrap();
        f.write(format!("{}\n", EvalFile::V2.to_str()).as_bytes()).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
    }

    pub fn copy(&mut self, src : &Weight) {
        for (d, s) in self.weight.iter_mut().zip(src.weight.iter()) {
            *d = *s;
        }
    }

    fn fromv1(&mut self, tbl : &Vec<f32>) {
        // ban
        for i in 0..N_HIDDEN {
            let mut we = &mut self.weight[i * board::CELL_2D..(i + 1) * board::CELL_2D];
            let mut tb = &tbl[i * (board::CELL_2D + 1 + 1)..(i + 1) * (board::CELL_2D + 1 + 1)];
            for (w, t) in we.iter_mut().zip(tb.iter()) {
                *w = *t;
            }
            let mut teb = &mut self.weight[
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

    pub fn evaluatev1(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
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
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
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
            let mut sumarr : [f32 ; 4] = [0.0, 0.0, 0.0, 0.0];
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

        let tbn = &ow.as_slice()[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow.as_slice()[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow.as_slice()[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
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

        let tbn = &ow.as_slice()[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow.as_slice()[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow.as_slice()[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = dc[i];
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
            }
            for i in 0..board::CELL_2D / 4 {
                let idx = i * 4;
                unsafe {
                    let x4 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                    let y4 = x86_64::_mm_set_epi32(
                        cells[idx + 3] as i32, cells[idx + 2] as i32,
                        cells[idx + 1] as i32, cells[idx + 0] as i32);
                    let y4 = x86_64::_mm_cvtepi32_ps(y4);
                    let mul = x86_64::_mm_mul_ps(x4, y4);
                    sum4 = x86_64::_mm_add_ps(sum4, mul);
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
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
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
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
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
            let mut sumarr : [f32 ; 4] = [0.0, 0.0, 0.0, 0.0];
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

        let tbn = &ow.as_slice()[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow.as_slice()[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow.as_slice()[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
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

        let tbn = &ow.as_slice()[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow.as_slice()[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow.as_slice()[(board::CELL_2D + 2) * N_HIDDEN ..];
        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
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

                    let c8 = x86_64::_mm_loadl_epi64(cells[idx..].as_ptr() as *const x86_64::__m128i);
                    let zero = x86_64::_mm_setzero_si128();
                    // to i16
                    let c4l = x86_64::_mm_unpacklo_epi16(zero, c8);
                    let c4h = x86_64::_mm_unpackhi_epi16(zero, c8);
                    // to i32
                    let c41 = x86_64::_mm_unpacklo_epi32(zero, c4l);
                    let c42 = x86_64::_mm_unpackhi_epi32(zero, c4l);
                    let c43 = x86_64::_mm_unpacklo_epi32(zero, c4h);
                    let c44 = x86_64::_mm_unpackhi_epi32(zero, c4h);
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
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += w2[i] * hidsig[i];
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
        let w2 = &mut ow.as_mut_slice()[w1sz * 4..];
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
            let w1 = &mut ow.as_mut_slice()[i * w1sz .. (i + 1) * w1sz];
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
        let w2 = &mut ow.as_mut_slice()[(board::CELL_2D + 2) * N_HIDDEN ..];
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
            let w1 = &mut ow.as_mut_slice()[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
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

                        let x4 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                        let w4 = x86_64::_mm_sub_ps(x4, diff4);
                        x86_64::_mm_store_ps(w1[idx..].as_mut_ptr(), w4);
                    }
                }
            }
            let tbndc = &mut ow.as_mut_slice()[board::CELL_2D * N_HIDDEN ..];
            tbndc[i] -= teban * heta;
            tbndc[i + N_HIDDEN] -= heta;
        }
    }

    fn learn(&mut self, ban : &board::Board, winner : i8, eta : f32) {
        let cells = &ban.cells;
        let teban = ban.teban;
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
