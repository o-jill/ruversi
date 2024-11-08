use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};

#[cfg(target_arch="x86_64")]
use std::arch::x86_64;

#[cfg(target_arch="aarch64")]
use std::arch::aarch64::*;


/*
 * input: NUMCELL * NUMCELL + 1(teban) + 2(fixedstones) + 1
 * hidden: 8 + 1
 * output: 1
 */
const N_INPUT : usize = board::CELL_2D + 1 + 2;
const N_HIDDEN : usize = 32;
const N_OUTPUT : usize = 1;
const N_WEIGHT: usize = (N_INPUT + 1) * N_HIDDEN + N_HIDDEN + 1;

#[allow(dead_code)]
const WSZV1 : usize = (board::CELL_2D + 1 + 1) * 4 + 4 + 1;
#[allow(dead_code)]
const WSZV2 : usize = WSZV1;
const WSZV3 : usize = (board::CELL_2D + 1 + 2 + 1) * 4 + 4 + 1;
const WSZV4 : usize = (board::CELL_2D + 1 + 2 + 1) * 8 + 8 + 1;
const WSZV5 : usize = (board::CELL_2D + 1 + 2 + 1) * 16 + 16 + 1;
const WSZV6 : usize = (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN + N_HIDDEN + 1;

const EXP_HI : f32 = 88.3762626647949;
const EXP_LO : f32 = -EXP_HI;

const CEPHES_LOG2EF : f32 = 1.44269504088896341;
const CEPHES_EXP_C1 : f32 = 0.693359375;
const CEPHES_EXP_C2 : f32 = -2.12194440e-4;

const CEPHES_EXP_P0 : f32 = 1.9875691500E-4;
const CEPHES_EXP_P1 : f32 = 1.3981999507E-3;
const CEPHES_EXP_P2 : f32 = 8.3334519073E-3;
const CEPHES_EXP_P3 : f32 = 4.1665795894E-2;
const CEPHES_EXP_P4 : f32 = 1.6666665459E-1;
const CEPHES_EXP_P5 : f32 = 5.0000001201E-1;


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
    V4,
    V5,
    V6,
}

impl EvalFile {
    pub fn to_str(&self) -> &str {
        match self {
            EvalFile::Unknown => {"unknown eval file format."},
            EvalFile::V1 => {"# 65-4-1"},
            EvalFile::V2 => {"# 64+1-4-1"},
            EvalFile::V3 => {"# 64+1+2-4-1"},
            EvalFile::V4 => {"# 64+1+2-8-1"},
            EvalFile::V5 => {"# 64+1+2-16-1"},
            EvalFile::V6 => {"# 64+1+2-32-1"},
        }
    }

    pub fn from(txt : &str) -> Option<EvalFile> {
        match txt {
            "# 65-4-1" => Some(EvalFile::V1),
            "# 64+1-4-1" => Some(EvalFile::V2),
            "# 64+1+2-4-1" => Some(EvalFile::V3),
            "# 64+1+2-8-1" => Some(EvalFile::V4),
            "# 64+1+2-16-1" => Some(EvalFile::V5),
            "# 64+1+2-32-1" => Some(EvalFile::V6),
            _ => None
        }
    }
}

#[repr(align(32))]
pub struct Weight {
    pub weight : [f32 ; N_WEIGHT]
}

impl Weight {
    pub fn new() -> Weight {
        Weight {
            weight: [0.0 ; N_WEIGHT]
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

    /// fill zero.
    pub fn clear(&mut self) {
        self.weight.iter_mut().for_each(|m| *m = 0.0);
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
                        EvalFile::V2 => {return self.readv2(&l)},
                        EvalFile::V3 => {return self.readv3(&l)},
                        EvalFile::V4 => {return self.readv4(&l)},
                        EvalFile::V5 => {return self.readv5(&l)},
                        EvalFile::V6 => {return self.readv6(&l)},
                        _ => {}
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }

        Err("no weight".to_string())
    }

    fn readv1(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v1 format is not supported any more."))
    }

    fn readv2(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v2 format is not supported any more."))
    }

    fn readv3(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV3 != nsz {
            return Err(format!("size mismatch {WSZV3} != {nsz}"));
        }
        self.fromv3tov6(&newtable);
        // println!("v3:{:?}", self.weight);
        Ok(())
    }

    fn readv4(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV4 != nsz {
            return Err(String::from("size mismatch"));
        }
        self.fromv4tov6(&newtable);
        // println!("v4:{:?}", self.weight);
        Ok(())
    }

    fn readv5(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV5 != nsz {
            return Err(String::from("size mismatch"));
        }
        self.fromv5tov6(&newtable);
        // println!("v5:{:?}", self.weight);
        Ok(())
    }

    fn readv6(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV6 != nsz {
            return Err(String::from("size mismatch"));
        }
        self.weight.copy_from_slice(&newtable);
        // println!("v6:{:?}", self.weight);
        Ok(())
    }

    fn write(f : &mut File, w : &[f32], ver : &EvalFile) {
        let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        f.write_all(format!("{}\n", ver.to_str()).as_bytes()).unwrap();
        f.write_all(sv.join(",").as_bytes()).unwrap();
    }

    #[allow(dead_code)]
    pub fn writev1(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V1);
    }

    #[allow(dead_code)]
    pub fn writev2(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V2);
    }

    #[allow(dead_code)]
    pub fn writev3(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V3);
    }

    #[allow(dead_code)]
    pub fn writev4(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V4);
    }

    pub fn writev5(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V5);
    }

    pub fn writev6(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V6);
    }

    pub fn writev1asv2(&self, path : &str) {
        let mut w = Weight::new();
        w.fromv1tov2(&self.weight);
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V2);
    }

    pub fn writev2asv3(&self, path : &str) {
        let mut w = Weight::new();
        w.fromv2tov3(&self.weight);
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V2);
    }

    pub fn copy(&mut self, src : &Weight) {
        for (d, s) in self.weight.iter_mut().zip(src.weight.iter()) {
            *d = *s;
        }
    }

    fn fromv1tov2(&mut self, tbl : &[f32]) {
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

    #[allow(dead_code)]
    fn fromv1tov3(&mut self, tbl : &[f32]) {
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

    fn fromv2tov3(&mut self, tbl : &[f32]) {
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

    /// copy v3 data into v4.
    #[allow(dead_code)]
    fn fromv3tov4(&mut self, tbl : &[f32]) {
        self.weight = [0.0f32 ; N_WEIGHT];
        // ban
        let n = 4 * board::CELL_2D;
        let we = &mut self.weight[0..n];
        let tb = &tbl[0..n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // teban
        let idx3 = 4 * board::CELL_2D;
        let idx4 = N_HIDDEN * board::CELL_2D;
        let n = 4;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // fixed stone
        let idx3 = 4 * (board::CELL_2D + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1);
        let n = 4;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }
        let idx3 = 4 * (board::CELL_2D + 1 + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 1);
        let n = 4;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // dc
        let idx3 = 4 * (board::CELL_2D + 1 + 2);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 2);
        let n = 4;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // w2
        let idx3 = 4 * (board::CELL_2D + 1 + 2 + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 2 + 1);
        let n = 4;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // dc2
        let idx3 = 4 * (board::CELL_2D + 1 + 2 + 1 + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 2 + 1 + 1);
        self.weight[idx4] =  tbl[idx3];
        // println!("tbl:{tbl:?}");
        // println!("we:{:?}", self.weight);
    }

    /// copy v3 data into v4.
    fn convert(&mut self, tbl : &Vec<f32>, nhid : usize) {
        self.weight = [0.0f32 ; N_WEIGHT];
        // ban
        let n = nhid * board::CELL_2D;
        let we = &mut self.weight[0..n];
        let tb = &tbl[0..n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // teban
        let idx3 = nhid * board::CELL_2D;
        let idx4 = N_HIDDEN * board::CELL_2D;
        let n = nhid;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // fixed stone
        let idx3 = nhid * (board::CELL_2D + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1);
        let n = nhid;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }
        let idx3 = nhid * (board::CELL_2D + 1 + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 1);
        let n = nhid;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // dc
        let idx3 = nhid * (board::CELL_2D + 1 + 2);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 2);
        let n = nhid;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // w2
        let idx3 = nhid * (board::CELL_2D + 1 + 2 + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 2 + 1);
        let n = nhid;
        let we = &mut self.weight[idx4..idx4 + n];
        let tb = &tbl[idx3..idx3 + n];
        for (w, t) in we.iter_mut().zip(tb.iter()) {
            *w = *t;
        }

        // dc2
        let idx3 = nhid * (board::CELL_2D + 1 + 2 + 1 + 1);
        let idx4 = N_HIDDEN * (board::CELL_2D + 1 + 2 + 1 + 1);
        self.weight[idx4] =  tbl[idx3];
        // println!("tbl:{tbl:?}");
        // println!("we:{:?}", self.weight);
    }

    /// copy v3 data into v6.
    fn fromv3tov6(&mut self, tbl : &Vec<f32>) {
        self.convert(tbl, 4);
    }

    /// copy v4 data into v6.
    fn fromv4tov6(&mut self, tbl : &Vec<f32>) {
        self.convert(tbl, 8);
    }

    /// copy v5 data into v6.
    fn fromv5tov6(&mut self, tbl : &Vec<f32>) {
        self.convert(tbl, 16);
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

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev1_simd(&self, ban : &board::Board) -> f32 {
        use std::arch::x86_64;
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
            let mut sumarr : [f32 ; 4] = [0.0f32 ; 4];
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

    #[allow(dead_code)]
    #[cfg(target_arch="x86_64")]
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
            let mut sumarr : [f32 ; 4] = [0.0f32 ; 4];
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
     * 
     * # Arguments  
     * * `x` - [4 ; f32]  
     * * `y` - [exp(-x[0]), exp(-x[1]), exp(-x[2]), exp(-x[3])]  
     */
    #[cfg(target_arch="x86_64")]
    fn expmx_ps( x : *const f32, y : *mut f32) {
        unsafe {
            let x4 = x86_64::_mm_load_ps(x);
            let y4 = Weight::expmx_ps_simd(x4);
            x86_64::_mm_store_ps(y, y4);
        }
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x` - [4 ; f32]  
     * * `y` - [exp(-x[0]), exp(-x[1]), exp(-x[2]), exp(-x[3])]  
     */
    #[cfg(target_arch="aarch64")]
    fn expmx_ps( x : *const f32, y : *mut f32) {
        unsafe {
            let x4 = vld1q_f32(x);
            let y4 = Weight::expmx_ps_simd(x4);
            vst1q_f32(y, y4);
        }
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x4` - [4 ; f32] as x86_64::__m128
     * 
     * # Return value  
     * [exp(-x4[0]), exp(-x4[1]), exp(-x4[2]), exp(-x4[3])] as x86_64::__m128.
     */
    #[cfg(target_arch="x86_64")]
    unsafe fn expmx_ps_simd(x4 : x86_64::__m128) -> x86_64::__m128 {
        // let x4 = x86_64::_mm_load_ps(x);
        // clip x
        let max4 = x86_64::_mm_set1_ps(EXP_HI);
        let x4 = x86_64::_mm_min_ps(x4, max4);
        let min4 = x86_64::_mm_set1_ps(EXP_LO);
        let x4 = x86_64::_mm_max_ps(x4, min4);
        let m1 = x86_64::_mm_set1_ps(-1.0);
        let x4 = x86_64::_mm_mul_ps(x4, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = x86_64::_mm_set1_ps(CEPHES_LOG2EF);
        let fx = x86_64::_mm_mul_ps(x4, log2ef);
        let zp5 = x86_64::_mm_set1_ps(CEPHES_EXP_P5);
        let fx = x86_64::_mm_add_ps(fx, zp5);
        let emm0 = x86_64::_mm_cvtps_epi32(fx);
        let tmp = x86_64::_mm_cvtepi32_ps(emm0);

        let mask = x86_64::_mm_cmpgt_ps(tmp, fx);
        let one = x86_64::_mm_set1_ps(1.0);
        let mask = x86_64::_mm_and_ps(mask, one);
        let fx = x86_64::_mm_sub_ps(tmp, mask);

        let c1 = x86_64::_mm_set1_ps(CEPHES_EXP_C1);
        let tmp = x86_64::_mm_mul_ps(fx, c1);
        let c2 = x86_64::_mm_set1_ps(CEPHES_EXP_C2);
        let z4 = x86_64::_mm_mul_ps(fx, c2);
        let x4 = x86_64::_mm_sub_ps(x4, tmp);
        let x4 = x86_64::_mm_sub_ps(x4, z4);

        let z4 = x86_64::_mm_mul_ps(x4, x4);

        let y4 = x86_64::_mm_set1_ps(CEPHES_EXP_P0);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p1 = x86_64::_mm_set1_ps(CEPHES_EXP_P1);
        let y4 = x86_64::_mm_add_ps(y4, exp_p1);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p2 = x86_64::_mm_set1_ps(CEPHES_EXP_P2);
        let y4 = x86_64::_mm_add_ps(y4, exp_p2);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p3 = x86_64::_mm_set1_ps(CEPHES_EXP_P3);
        let y4 = x86_64::_mm_add_ps(y4, exp_p3);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p4 = x86_64::_mm_set1_ps(CEPHES_EXP_P4);
        let y4 = x86_64::_mm_add_ps(y4, exp_p4);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p5 = x86_64::_mm_set1_ps(CEPHES_EXP_P5);
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
        y4
        // x86_64::_mm_store_ps(y, y4);
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x4` - [4 ; f32] as x86_64::__m128
     * 
     * # Return value  
     * [exp(-x4[0]), exp(-x4[1]), exp(-x4[2]), exp(-x4[3])] as x86_64::__m128.
     */
    #[cfg(target_arch="aarch64")]
    unsafe fn expmx_ps_simd(x4 : float32x4_t) -> float32x4_t {
        // clip x
        let max4 = vmovq_n_f32(EXP_HI);
        let x4 = vminq_f32(x4, max4);
        let min4 = vmovq_n_f32(EXP_LO);
        let x4 = vmaxq_f32(x4, min4);
        let m1 = vmovq_n_f32(-1.0);
        let x4 = vmulq_f32(x4, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = vmovq_n_f32(CEPHES_LOG2EF);
        let zp5 = vmovq_n_f32(CEPHES_EXP_P5);
        let fx = vmlaq_f32(zp5, x4, log2ef);
        let emm0 = vcvtq_s32_f32(fx);
        let tmp = vcvtq_f32_s32(emm0);

        let mask = vcgtq_f32(tmp, fx);
        let one = vmovq_n_f32(1.0);
        let mask = vreinterpretq_f32_u32(vandq_u32(
                mask, vreinterpretq_u32_f32(one)));
        let fx = vsubq_f32(tmp, mask);

        let c1 = vmovq_n_f32(CEPHES_EXP_C1);
        let tmp = vmulq_f32(fx, c1);
        let c2 = vmovq_n_f32(CEPHES_EXP_C2);
        let z4 = vmulq_f32(fx, c2);
        let x4 = vsubq_f32(x4, tmp);
        let x4 = vsubq_f32(x4, z4);

        let z4 = vmulq_f32(x4, x4);

        let y4 = vmovq_n_f32(CEPHES_EXP_P0);
        let exp_p1 = vmovq_n_f32(CEPHES_EXP_P1);
        let y4 = vmlaq_f32(exp_p1, y4, x4);
        let exp_p2 = vmovq_n_f32(CEPHES_EXP_P2);
        let y4 = vmlaq_f32(exp_p2, y4, x4);
        let exp_p3 = vmovq_n_f32(CEPHES_EXP_P3);
        let y4 = vmlaq_f32(exp_p3, y4, x4);
        let exp_p4 = vmovq_n_f32(CEPHES_EXP_P4);
        let y4 = vmlaq_f32(exp_p4, y4, x4);
        let exp_p5 = vmovq_n_f32(CEPHES_EXP_P5);
        let y4 = vmlaq_f32(exp_p5, y4, x4);
        let y4 = vmlaq_f32(x4, y4, z4);
        let y4 = vaddq_f32(y4, one);

        let emm0 = vcvtq_s32_f32(fx);
        let _pi32_0x7f = vmovq_n_s32(0x7f);
        let emm0 = vaddq_s32(emm0, _pi32_0x7f);
        let emm0 = vshlq_n_s32(emm0, 23);
        let pow2n = vreinterpretq_f32_s32(emm0);

        let y4 = vmulq_f32(y4, pow2n);
        y4
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x4` - [4 ; f32] as x86_64::__m128
     * 
     * # Return value  
     * [exp(-x4[0]), exp(-x4[1]), exp(-x4[2]), exp(-x4[3])] as x86_64::__m128.
     */
    #[cfg(target_arch="x86_64")]
    unsafe fn expmx_ps_simd256(x4 : x86_64::__m256) -> x86_64::__m256 {
        // let x4 = x86_64::_mm_load_ps(x);
        // clip x
        let max4 = x86_64::_mm256_set1_ps(EXP_HI);
        let x4 = x86_64::_mm256_min_ps(x4, max4);
        let min4 = x86_64::_mm256_set1_ps(EXP_LO);
        let x4 = x86_64::_mm256_max_ps(x4, min4);
        let m1 = x86_64::_mm256_set1_ps(-1.0);
        let x4 = x86_64::_mm256_mul_ps(x4, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = x86_64::_mm256_set1_ps(CEPHES_LOG2EF);
        let fx = x86_64::_mm256_mul_ps(x4, log2ef);
        let zp5 = x86_64::_mm256_set1_ps(CEPHES_EXP_P5);
        let fx = x86_64::_mm256_add_ps(fx, zp5);
        let emm0 = x86_64::_mm256_cvtps_epi32(fx);
        let tmp = x86_64::_mm256_cvtepi32_ps(emm0);

        let mask = x86_64::_mm256_cmp_ps(tmp, fx, x86_64::_CMP_GT_OS);
        let one = x86_64::_mm256_set1_ps(1.0);
        let mask = x86_64::_mm256_and_ps(mask, one);
        let fx = x86_64::_mm256_sub_ps(tmp, mask);

        let c1 = x86_64::_mm256_set1_ps(CEPHES_EXP_C1);
        let tmp = x86_64::_mm256_mul_ps(fx, c1);
        let c2 = x86_64::_mm256_set1_ps(CEPHES_EXP_C2);
        let z4 = x86_64::_mm256_mul_ps(fx, c2);
        let x4 = x86_64::_mm256_sub_ps(x4, tmp);
        let x4 = x86_64::_mm256_sub_ps(x4, z4);

        let z4 = x86_64::_mm256_mul_ps(x4, x4);

        let y4 = x86_64::_mm256_set1_ps(CEPHES_EXP_P0);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p1 = x86_64::_mm256_set1_ps(CEPHES_EXP_P1);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p1);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p2 = x86_64::_mm256_set1_ps(CEPHES_EXP_P2);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p2);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p3 = x86_64::_mm256_set1_ps(CEPHES_EXP_P3);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p3);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p4 = x86_64::_mm256_set1_ps(CEPHES_EXP_P4);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p4);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p5 = x86_64::_mm256_set1_ps(CEPHES_EXP_P5);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p5);
        let y4 = x86_64::_mm256_mul_ps(y4, z4);
        let y4 = x86_64::_mm256_add_ps(y4, x4);
        let y4 = x86_64::_mm256_add_ps(y4, one);

        let emm0 = x86_64::_mm256_cvttps_epi32(fx);
        let _pi32_0x7f = x86_64::_mm256_set1_epi32(0x7f);
        let emm0 = x86_64::_mm256_add_epi32(emm0, _pi32_0x7f);
        let emm0 = x86_64::_mm256_slli_epi32(emm0, 23);
        let pow2n = x86_64::_mm256_castsi256_ps(emm0);

        let y4 = x86_64::_mm256_mul_ps(y4, pow2n);
        y4
        // x86_64::_mm_store_ps(y, y4);
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev2_simd2(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        sum = *ow.last().unwrap();

        let tbn = &ow[board::CELL_2D * N_HIDDEN .. board::CELL_2D * N_HIDDEN + N_HIDDEN];
        let dc = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 2) * N_HIDDEN];
        let w2 = &ow[(board::CELL_2D + 2) * N_HIDDEN ..];

        let mut hidsum : [f32 ; 4] = [0.0f32 ; 4];
        let mut emx : [f32 ; 4] = [0.0f32 ; 4];
        let mut sumarr : [f32 ; 4] = [0.0f32 ; 4];

        for i in 0..N_HIDDEN / 4 {
            let hidx = i * 4;
            let mut sum44 : [f32 ; 4 * 4] = [0.0f32 ; 4 * 4];

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
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
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

    pub fn evaluatev3bb(&self, ban : &bitboard::BitBoard) -> f32 {
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
            let mut hidsum : f32 = 0.0;  // wdc[i];
            for y in 0..bitboard::NUMCELL {
                let mut bit = bitboard::LSB_CELL << y;
                for x in 0..bitboard::NUMCELL {
                    let w = w1[x + y * bitboard::NUMCELL];
                    // let idx = x * bitboard::NUMCELL + y;
                    // let diff = ((bit & black) >> idx) as i32 - ((bit & white) >> idx) as i32;
                    // hidsum += diff as f32 * w;
                    // hidsum += w * ban.at(x, y) as f32;
                    hidsum +=
                        if (bit & black) != 0 {w}
                        else if (bit & white) != 0 {-w}
                        else {0.0};
                    bit <<= bitboard::NUMCELL;
                }
            }
            hidsum = teban.mul_add(wtbn[i], hidsum);
            hidsum = wfs[i].mul_add(fs.0 as f32, hidsum);
            hidsum = wfs[i + N_HIDDEN].mul_add(fs.1 as f32, hidsum + wdc[i]);
            sum = wh[i].mul_add(((-hidsum).exp() + 1.0).recip(), sum);
            // sum += wh[i] / ((-hidsum).exp() + 1.0);
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
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
        let mut hidsum : [f32 ; N] = [0.0f32 ; N];
        let mut emx : [f32 ; N] = [0.0f32 ; N];
        let mut sumarr : [f32 ; N] = [0.0f32 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * N] = [0.0f32 ; N * N];

            for n in 0..N {
                let res4 = sum44[n * N..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
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

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
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
        let mut sumarr : [f32 ; N] = [0.0f32 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * N] = [0.0f32 ; N * N];

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
                        let mn1 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm1), minus);
                        let mn2 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm2), minus);
                        let mn3 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm3), minus);
                        let mn4 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm4), minus);
                        let m41 = x86_64::_mm_xor_ps(x41, mn1);
                        let m42 = x86_64::_mm_xor_ps(x42, mn2);
                        let m43 = x86_64::_mm_xor_ps(x43, mn3);
                        let m44 = x86_64::_mm_xor_ps(x44, mn4);
                        let w1 = x86_64::_mm_and_ps(
                                m41, x86_64::_mm_castsi128_ps(ex1));
                        let w2 = x86_64::_mm_and_ps(
                                m42, x86_64::_mm_castsi128_ps(ex2));
                        let w3 = x86_64::_mm_and_ps(
                                m43, x86_64::_mm_castsi128_ps(ex3));
                        let w4 = x86_64::_mm_and_ps(
                                m44, x86_64::_mm_castsi128_ps(ex4));

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

                let emx4 = Weight::expmx_ps_simd(h1234);
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

    #[cfg(target_arch="aarch64")]
    pub fn evaluatev3bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        use std::arch::aarch64::*;
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let (fsb, fsw) = ban.fixedstones();

        let mut res = *ow.last().unwrap();

        let wtbn = &ow[bitboard::CELL_2D * N_HIDDEN .. ];
        let wfs = &ow[(bitboard::CELL_2D + 1) * N_HIDDEN .. ];
        let wdc = &ow[(bitboard::CELL_2D + 1 + 2) * N_HIDDEN .. ];
        let wh = &ow[(bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN .. ];
        const N : usize = 8;

        for i in (0..N_HIDDEN).step_by(N) {
            let mut sumn = [0.0f32 ; N];

            for n in 0..N {
                let w1 = &ow[(i + n) * bitboard::CELL_2D .. ];
                const M : usize = 16;
                let bit8 = 0x0101010101010101u64;
                for j in 0..bitboard::CELL_2D / M {
                    let idx = j * M;
                    let b81 = bit8 & (black >> j * 2);
                    let w81 = bit8 & (white >> j * 2);
                    let b82 = bit8 & (black >> j * 2 + 1);
                    let w82 = bit8 & (white >> j * 2 + 1);

                    unsafe {
                        let b08 = vmov_n_u64(b81 * 0xffu64);
                        let b082 = vmov_n_u64(b82 * 0xffu64);
                        let w08 = vmov_n_u64(w81 * 0xffu64);
                        let w082 = vmov_n_u64(w82 * 0xffu64);
                        let b04 = vmovl_s8(vreinterpret_s8_u64(b08));
                        let b042 = vmovl_s8(vreinterpret_s8_u64(b082));
                        let w04 = vmovl_s8(vreinterpret_s8_u64(w08));
                        let w042 = vmovl_s8(vreinterpret_s8_u64(w082));
                        let b02 = vmovl_s16(vget_low_s16(b04));
                        let b022 = vmovl_s16(vget_low_s16(b042));
                        let b12 = vmovl_high_s16(b04);
                        let b122 = vmovl_high_s16(b042);
                        let w02 = vmovl_s16(vget_low_s16(w04));
                        let w022 = vmovl_s16(vget_low_s16(w042));
                        let w12 = vmovl_high_s16(w04);
                        let w122 = vmovl_high_s16(w042);
                        let ex1 = vorrq_s32(b02, w02);
                        let ex12 = vorrq_s32(b022, w022);
                        let ex2 = vorrq_s32(b12, w12);
                        let ex22 = vorrq_s32(b122, w122);
                        let minus = vreinterpretq_s32_f32(vmovq_n_f32(-0.0));
                        let mn1 = vandq_s32(minus, w02);
                        let mn12 = vandq_s32(minus, w022);
                        let mn2 = vandq_s32(minus, w12);
                        let mn22 = vandq_s32(minus, w122);
                        let w41 = vld1q_f32_x4(w1.as_ptr().add(idx));
                        let w1 = veorq_s32(mn1, vreinterpretq_s32_f32(w41.0));
                        let w12 = veorq_s32(mn12, vreinterpretq_s32_f32(w41.2));
                        let w2 = veorq_s32(mn2, vreinterpretq_s32_f32(w41.1));
                        let w22 = veorq_s32(mn22, vreinterpretq_s32_f32(w41.3));
                        let w1 = vandq_s32(ex1, w1);
                        let w12 = vandq_s32(ex12, w12);
                        let w2 = vandq_s32(ex2, w2);
                        let w22 = vandq_s32(ex22, w22);
                        let sum = vaddq_f32(vreinterpretq_f32_s32(w1),
                                                        vreinterpretq_f32_s32(w2));
                        let sum2 = vaddq_f32(vreinterpretq_f32_s32(w12),
                                                        vreinterpretq_f32_s32(w22));
                        let sum = vaddvq_f32(vaddq_f32(sum, sum2));
                        sumn[n] += sum;
                    }
                }
            }
            unsafe {
                let sum4 = vld1q_f32_x2(sumn.as_ptr());

                let tbn = vmovq_n_f32(teban);
                let wtb = vld1q_f32_x2(wtbn.as_ptr().add(i));
                let sum41 = vmlaq_f32(sum4.0, tbn, wtb.0);
                let sum42 = vmlaq_f32(sum4.1, tbn, wtb.1);

                let fsb4 = vmovq_n_f32(fsb as f32);
                let wfsb = vld1q_f32_x2(wfs.as_ptr().add(i));
                let sum4 = vmlaq_f32(sum41, fsb4, wfsb.0);
                let sum42 = vmlaq_f32(sum42, fsb4, wfsb.1);

                let fsw4 = vmovq_n_f32(fsw as f32);
                let wfsw = vld1q_f32_x2(wfs.as_ptr().add(i + N_HIDDEN));
                let sum4 = vmlaq_f32(sum4, fsw4, wfsw.0);
                let sum42 = vmlaq_f32(sum42, fsw4, wfsw.1);

                let wdc4 = vld1q_f32_x2(wdc.as_ptr().add(i));
                let sum4 = vaddq_f32(sum4, wdc4.0);
                let sum42 = vaddq_f32(sum42, wdc4.1);
                vst1q_f32(sumn.as_mut_ptr(), sum4);
                vst1q_f32(sumn.as_mut_ptr().add(4), sum42);

                // let expmx = Self::expmx_ps_simd(sum4);
                // let expmx1 = vaddq_f32(expmx, vmovq_n_f32(1.0));
                // let wh4 = vld1q_f32(wh.as_ptr().add(i));
                // res += vaddvq_f32(vdivq_f32(wh4, expmx1));
                // 1950nps

                // let expmx = Self::expmx_ps_simd(sum4);
                // let expmx1 = vaddq_f32(expmx, vmovq_n_f32(1.0));
                // let remx = vrecpeq_f32(expmx1);
                // let wh4 = vld1q_f32(wh.as_ptr().add(i));
                // res += vaddvq_f32(vmulq_f32(remx, wh4));
                // expmx_ps_simd is slower than exp()x4 on M2 ...
                // 1950nps
            }
            for n in 0 .. N {
                // sumn[n] = (-sumn[n]).exp();
                res += wh[i + n] / ((-sumn[n]).exp() + 1.0);
            }  // 2050nps
            // unsafe {
            //     let expmx4 = vld1q_f32_x2(sumn.as_ptr());
            //     // let expmx = vld1q_f32(sumn.as_ptr());
            //     // let expmx2 = vld1q_f32(sumn.as_ptr().add(4));
            //     let one = vmovq_n_f32(1.0);
            //     let expmx1 = vaddq_f32(expmx4.0, one);
            //     let expmx12 = vaddq_f32(expmx4.1, one);
            //     let wh4 = vld1q_f32_x2(wh.as_ptr().add(i));
            //     // let wh4 = vld1q_f32(wh.as_ptr().add(i));
            //     // let wh42 = vld1q_f32(wh.as_ptr().add(i + 4));
            //     res += vaddvq_f32(vaddq_f32(vdivq_f32(wh4.0, expmx1),
            //                              vdivq_f32(wh4.1, expmx12)));
            // }
    }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
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

        const N : usize = 8;
        let mut sumarr : [f32 ; N] = [0.0 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum88 : [f32 ; N * 8] = [0.0 ; N * 8];

            for n in 0..N {
                let res8 = sum88[n * 8..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum8: x86_64::__m256;
                unsafe {
                    sum8 = x86_64::_mm256_setzero_ps();
                }
                const M : usize = 32;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> 4 * j;
                    let w81 = (bit8 & white) >> 4 * j;
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> 4 * j + 1;
                    let w82 = (bit8 & white) >> 4 * j + 1;
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> 4 * j + 2;
                    let w83 = (bit8 & white) >> 4 * j + 2;
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> 4 * j + 3;
                    let w84 = (bit8 & white) >> 4 * j + 3;
                    bit8 <<= 1;

                    unsafe {
                        let c1 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b81 as i64),
                                x86_64::_mm_set1_epi64x(w81 as i64));
                        let c2 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b82 as i64),
                                x86_64::_mm_set1_epi64x(w82 as i64));
                        let c3 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b83 as i64),
                                x86_64::_mm_set1_epi64x(w83 as i64));
                        let c4 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b84 as i64),
                                x86_64::_mm_set1_epi64x(w84 as i64));

                        let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                        let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                        let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                        let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                        let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                        let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                        let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                        let f84 = x86_64::_mm256_cvtepi32_ps(c84);

                        let x81 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx));
                        let x82 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx + 8));
                        let x83 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx + 16));
                        let x84 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx + 24));

                        if true {  // fma
                            sum8 = x86_64::_mm256_fmadd_ps(x81, f81, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x82, f82, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x83, f83, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x84, f84, sum8);
                        } else {
                            let mul1 = x86_64::_mm256_mul_ps(x81, f81);
                            let mul2 = x86_64::_mm256_mul_ps(x82, f82);
                            let mul3 = x86_64::_mm256_mul_ps(x83, f83);
                            let mul4 = x86_64::_mm256_mul_ps(x84, f84);

                            let sum12 = x86_64::_mm256_add_ps(mul1, mul2);
                            let sum34 = x86_64::_mm256_add_ps(mul3, mul4);
                            let sum1234 = x86_64::_mm256_add_ps(sum12, sum34);
                            sum8 = x86_64::_mm256_add_ps(sum8, sum1234);
                        }
                    }
                }
                unsafe {
                    x86_64::_mm256_storeu_ps(res8, sum8);
                }
            }

            unsafe {
                let x1 = x86_64::_mm256_load_ps(sum88.as_ptr());
                let x2 = x86_64::_mm256_load_ps(sum88.as_ptr().add(8));
                let x3 = x86_64::_mm256_load_ps(sum88.as_ptr().add(16));
                let x4 = x86_64::_mm256_load_ps(sum88.as_ptr().add(24));
                let x5 = x86_64::_mm256_load_ps(sum88.as_ptr().add(32));
                let x6 = x86_64::_mm256_load_ps(sum88.as_ptr().add(40));
                let x7 = x86_64::_mm256_load_ps(sum88.as_ptr().add(48));
                let x8 = x86_64::_mm256_load_ps(sum88.as_ptr().add(56));

                let xl12 = x86_64::_mm256_unpacklo_ps(x1, x2);
                let xh12 = x86_64::_mm256_unpackhi_ps(x1, x2);
                let xl34 = x86_64::_mm256_unpacklo_ps(x3, x4);
                let xh34 = x86_64::_mm256_unpackhi_ps(x3, x4);
                let xl56 = x86_64::_mm256_unpacklo_ps(x5, x6);
                let xh56 = x86_64::_mm256_unpackhi_ps(x5, x6);
                let xl78 = x86_64::_mm256_unpacklo_ps(x7, x8);
                let xh78 = x86_64::_mm256_unpackhi_ps(x7, x8);

                let x12 = x86_64::_mm256_add_ps(xl12, xh12);
                let x34 = x86_64::_mm256_add_ps(xl34, xh34);
                let x56 = x86_64::_mm256_add_ps(xl56, xh56);
                let x78 = x86_64::_mm256_add_ps(xl78, xh78);

                let x1234 = x86_64::_mm256_shuffle_ps(x12, x34, 0x44);
                let xabcd = x86_64::_mm256_shuffle_ps(x12, x34, 0xee);
                let x5678 = x86_64::_mm256_shuffle_ps(x56, x78, 0x44);
                let xefgh = x86_64::_mm256_shuffle_ps(x56, x78, 0xee);

                let xabcd = x86_64::_mm256_add_ps(x1234, xabcd);
                let xefgh = x86_64::_mm256_add_ps(x5678, xefgh);

                let x1234 = x86_64::_mm256_permute2f128_ps(xabcd, xefgh, 0x20);
                let x5678 = x86_64::_mm256_permute2f128_ps(xabcd, xefgh, 0x31);

                let h18 = x86_64::_mm256_add_ps(x1234, x5678);

                // teban
                let wtbn = x86_64::_mm256_load_ps(wtbn.as_ptr().add(hidx));
                let tbn = x86_64::_mm256_set1_ps(teban);
                let tbn4 = x86_64::_mm256_mul_ps(wtbn, tbn);
                let h18 = x86_64::_mm256_add_ps(h18, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx));
                let fsb = x86_64::_mm256_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm256_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm256_load_ps(
                        wfs.as_ptr().add(hidx + N_HIDDEN));
                let fsw = x86_64::_mm256_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm256_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm256_add_ps(fsb4, fsw4);
                let h18 = x86_64::_mm256_add_ps(h18, fsbw);
                // dc
                let wdc4 = x86_64::_mm256_load_ps(wdc.as_ptr().add(hidx));
                let h1234 = x86_64::_mm256_add_ps(h18, wdc4);

                let emx4 = Weight::expmx_ps_simd256(h1234);
                let one = x86_64::_mm256_set1_ps(1.0);
                let hsp14 = x86_64::_mm256_add_ps(emx4, one);
                let wh4 = x86_64::_mm256_load_ps(wh.as_ptr().add(hidx));

                let y4 = x86_64::_mm256_div_ps(wh4, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm256_store_ps(sumarr.as_mut_ptr(), y4);
            }
            for n in 0..N {
                sum += sumarr[n];
            }
            // sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3bb_simdavx2(&self, ban : &bitboard::BitBoard) -> f32 {
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
            let mut sum48 : [f32 ; N * 8] = [0.0 ; N * 8];

            for n in 0..N {
                let res8 = sum48[n * 8..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum8: x86_64::__m256;
                unsafe {
                    sum8 = x86_64::_mm256_setzero_ps();
                }
                const M : usize = 32;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> 4 * j;
                    let w81 = (bit8 & white) >> 4 * j;
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> 4 * j + 1;
                    let w82 = (bit8 & white) >> 4 * j + 1;
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> 4 * j + 2;
                    let w83 = (bit8 & white) >> 4 * j + 2;
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> 4 * j + 3;
                    let w84 = (bit8 & white) >> 4 * j + 3;
                    bit8 <<= 1;

                    unsafe {
                        let c1 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b81 as i64),
                                x86_64::_mm_set1_epi64x(w81 as i64));
                        let c2 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b82 as i64),
                                x86_64::_mm_set1_epi64x(w82 as i64));
                        let c3 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b83 as i64),
                                x86_64::_mm_set1_epi64x(w83 as i64));
                        let c4 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b84 as i64),
                                x86_64::_mm_set1_epi64x(w84 as i64));

                        let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                        let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                        let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                        let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                        let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                        let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                        let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                        let f84 = x86_64::_mm256_cvtepi32_ps(c84);

                        let x81 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx));
                        let x82 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx + 8));
                        let x83 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx + 16));
                        let x84 = x86_64::_mm256_load_ps(
                            w1.as_ptr().add(idx + 24));

                        if true {  // fma
                            sum8 = x86_64::_mm256_fmadd_ps(x81, f81, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x82, f82, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x83, f83, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x84, f84, sum8);
                        } else {
                            let mul1 = x86_64::_mm256_mul_ps(x81, f81);
                            let mul2 = x86_64::_mm256_mul_ps(x82, f82);
                            let mul3 = x86_64::_mm256_mul_ps(x83, f83);
                            let mul4 = x86_64::_mm256_mul_ps(x84, f84);

                            let sum12 = x86_64::_mm256_add_ps(mul1, mul2);
                            let sum34 = x86_64::_mm256_add_ps(mul3, mul4);
                            let sum1234 = x86_64::_mm256_add_ps(sum12, sum34);
                            sum8 = x86_64::_mm256_add_ps(sum8, sum1234);
                        }
                    }
                }
                unsafe {
                    x86_64::_mm256_storeu_ps(res8, sum8);
                }
            }

            unsafe {
                let x11 = x86_64::_mm_load_ps(sum48.as_ptr());
                let x12 = x86_64::_mm_load_ps(sum48.as_ptr().add(4));
                let x21 = x86_64::_mm_load_ps(sum48.as_ptr().add(8));
                let x22 = x86_64::_mm_load_ps(sum48.as_ptr().add(12));
                let mut x1 = x86_64::_mm_add_ps(x11, x12);
                let mut x2 = x86_64::_mm_add_ps(x21, x22);

                let x31 = x86_64::_mm_load_ps(sum48.as_ptr().add(16));
                let x32 = x86_64::_mm_load_ps(sum48.as_ptr().add(20));
                let x41 = x86_64::_mm_load_ps(sum48.as_ptr().add(24));
                let x42 = x86_64::_mm_load_ps(sum48.as_ptr().add(28));
                let mut x3 = x86_64::_mm_add_ps(x31, x32);
                let mut x4 = x86_64::_mm_add_ps(x41, x42);

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);

                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn.as_ptr().add(hidx));
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs.as_ptr().add(hidx));
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm_load_ps(
                        wfs.as_ptr().add(hidx + N_HIDDEN));
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc.as_ptr().add(hidx));
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);

                let emx4 = Weight::expmx_ps_simd(h1234);
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let wh4 = x86_64::_mm_load_ps(wh.as_ptr().add(hidx));

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
            -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT]) {
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

    #[cfg(target_arch="x86_64")]
    pub fn forwardv1_simd(&self, ban : &board::Board)
            -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT]) {
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
            -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT]) {
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

    #[cfg(target_arch="x86_64")]
    pub fn forwardv2_simd(&self, ban : &board::Board)
            -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT]) {
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

    #[allow(dead_code)]
    #[cfg(target_arch="x86_64")]
    pub fn forwardv2_simd2(&self, ban : &board::Board)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT]) {
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
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
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

    pub fn forwardv3(&self, ban : &board::Board)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    #[cfg(target_arch="x86_64")]
    pub fn forwardv3_simd(&self, ban : &board::Board)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
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

                    let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                    let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                    let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                    let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                    if true {  // fma
                        sum4 = x86_64::_mm_fmadd_ps(x41, f41, sum4);
                        sum4 = x86_64::_mm_fmadd_ps(x42, f42, sum4);
                        sum4 = x86_64::_mm_fmadd_ps(x43, f43, sum4);
                        sum4 = x86_64::_mm_fmadd_ps(x44, f44, sum4);
                    } else {
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
            }

            let mut sumarr : [f32 ; 4] = [0.0 ; 4];
            unsafe {
                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    pub fn forwardv3bb(&self, ban : &bitboard::BitBoard)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    #[cfg(target_arch="x86_64")]
    pub fn forwardv3bb_simd(&self, ban : &bitboard::BitBoard)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
            let w1 = unsafe{ow.as_ptr().add(i * board::CELL_2D)};
            let mut hidsum : f32 = wdc[i];
            let mut sum4: x86_64::__m128;
            unsafe {
                sum4 = x86_64::_mm_setzero_ps();
            }
            let mut bit8 = 0x0101010101010101;
            for j in 0..board::CELL_2D / 16 {
                let idx = j * 16;
                let b81 = (bit8 & black) >> 2 * j;
                let w81 = (bit8 & white) >> 2 * j;
                bit8 <<= 1;
                let b82 = (bit8 & black) >> 2 * j + 1;
                let w82 = (bit8 & white) >> 2 * j + 1;
                bit8 <<= 1;

                unsafe {
                    x86_64::_mm_prefetch(
                            w1.add(idx) as *const i8, x86_64::_MM_HINT_T0);

                    let b16 = x86_64::_mm_set_epi64x(b82 as i64, b81 as i64);
                    let w16 = x86_64::_mm_set_epi64x(w82 as i64, w81 as i64);
                    let c16 = x86_64::_mm_sub_epi8(b16, w16);
                    let zero = x86_64::_mm_setzero_si128();
                    // to i16
                    let s16 = x86_64::_mm_cmpgt_epi8(zero, c16);
                    let c4l = x86_64::_mm_unpacklo_epi8(c16, s16);
                    let c4h = x86_64::_mm_unpackhi_epi8(c16, s16);
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

                    let x41 = x86_64::_mm_load_ps(w1.add(idx));
                    let x42 = x86_64::_mm_load_ps(w1.add(idx + 4));
                    let x43 = x86_64::_mm_load_ps(w1.add(idx + 8));
                    let x44 = x86_64::_mm_load_ps(w1.add(idx + 12));

                    if true {  // fma
                        sum4 = x86_64::_mm_fmadd_ps(x41, f41, sum4);
                        sum4 = x86_64::_mm_fmadd_ps(x42, f42, sum4);
                        sum4 = x86_64::_mm_fmadd_ps(x43, f43, sum4);
                        sum4 = x86_64::_mm_fmadd_ps(x44, f44, sum4);
                    } else {
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
            }

            let mut sumarr : [f32 ; 4] = [0.0 ; 4];
            unsafe {
                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    #[cfg(target_arch="x86_64")]
    pub fn forwardv3bb_simdavx2(&self, ban : &bitboard::BitBoard)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
            let w1 = unsafe{ow.as_ptr().add(i * board::CELL_2D)};
            let mut hidsum : f32 = wdc[i];
            let mut sum8 = unsafe {x86_64::_mm256_setzero_ps()};
            const M : usize = 32;
            let mut bit8 = 0x0101010101010101;
            for j in 0..board::CELL_2D / M {
                let idx = j * M;
                let b81 = (bit8 & black) >> 4 * j;
                let w81 = (bit8 & white) >> 4 * j;
                bit8 <<= 1;
                let b82 = (bit8 & black) >> 4 * j + 1;
                let w82 = (bit8 & white) >> 4 * j + 1;
                bit8 <<= 1;
                let b83 = (bit8 & black) >> 4 * j + 2;
                let w83 = (bit8 & white) >> 4 * j + 2;
                bit8 <<= 1;
                let b84 = (bit8 & black) >> 4 * j + 3;
                let w84 = (bit8 & white) >> 4 * j + 3;
                bit8 <<= 1;

                unsafe {
                    let c1 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b81 as i64), x86_64::_mm_set1_epi64x(w81 as i64));
                    let c2 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b82 as i64), x86_64::_mm_set1_epi64x(w82 as i64));
                    let c3 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b83 as i64), x86_64::_mm_set1_epi64x(w83 as i64));
                    let c4 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b84 as i64), x86_64::_mm_set1_epi64x(w84 as i64));

                    let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                    let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                    let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                    let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                    let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                    let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                    let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                    let f84 = x86_64::_mm256_cvtepi32_ps(c84);

                    x86_64::_mm_prefetch(w1.add(idx) as *const i8, x86_64::_MM_HINT_T0);
                    let x81 = x86_64::_mm256_load_ps(w1.add(idx));
                    let x82 = x86_64::_mm256_load_ps(w1.add(idx + 8));
                    let x83 = x86_64::_mm256_load_ps(w1.add(idx + 16));
                    let x84 = x86_64::_mm256_load_ps(w1.add(idx + 24));

                    if true {  // fma
                        sum8 = x86_64::_mm256_fmadd_ps(x81, f81, sum8);
                        sum8 = x86_64::_mm256_fmadd_ps(x82, f82, sum8);
                        sum8 = x86_64::_mm256_fmadd_ps(x83, f83, sum8);
                        sum8 = x86_64::_mm256_fmadd_ps(x84, f84, sum8);
                    } else {
                        let mul1 = x86_64::_mm256_mul_ps(x81, f81);
                        let mul2 = x86_64::_mm256_mul_ps(x82, f82);
                        let mul3 = x86_64::_mm256_mul_ps(x83, f83);
                        let mul4 = x86_64::_mm256_mul_ps(x84, f84);

                        let sum12 = x86_64::_mm256_add_ps(mul1, mul2);
                        let sum34 = x86_64::_mm256_add_ps(mul3, mul4);
                        let sum1234 = x86_64::_mm256_add_ps(sum12, sum34);
                        sum8 = x86_64::_mm256_add_ps(sum8, sum1234);
                    }
                }
            }

            let mut sumarr : [f32 ; 8] = [0.0 ; 8];
            unsafe {
                x86_64::_mm256_store_ps(sumarr.as_mut_ptr(), sum8);
            }
            for s in sumarr {
                hidsum += s;
            }
            // hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += wh[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    #[cfg(target_arch="x86_64")]
    pub fn forwardv3bb_simdavx3(&self, ban : &bitboard::BitBoard)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
        const N : usize = 8;
        let mut sumarr : [f32 ; N] = [0.0 ; N];
        let mut sum88 : [f32 ; N * 8] = [0.0 ; N * 8];
        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            for n in 0..N {
                let res8 = sum88[n * N..].as_mut_ptr();
                let w1 = unsafe{ow.as_ptr().add((hidx + n) * board::CELL_2D)};
                let mut sum8 = unsafe {x86_64::_mm256_setzero_ps()};
                const M : usize = 32;
                let mut bit8 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> 4 * j;
                    let w81 = (bit8 & white) >> 4 * j;
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> 4 * j + 1;
                    let w82 = (bit8 & white) >> 4 * j + 1;
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> 4 * j + 2;
                    let w83 = (bit8 & white) >> 4 * j + 2;
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> 4 * j + 3;
                    let w84 = (bit8 & white) >> 4 * j + 3;
                    bit8 <<= 1;

                    unsafe {
                        let c1 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b81 as i64), x86_64::_mm_set1_epi64x(w81 as i64));
                        let c2 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b82 as i64), x86_64::_mm_set1_epi64x(w82 as i64));
                        let c3 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b83 as i64), x86_64::_mm_set1_epi64x(w83 as i64));
                        let c4 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b84 as i64), x86_64::_mm_set1_epi64x(w84 as i64));

                        let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                        let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                        let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                        let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                        let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                        let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                        let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                        let f84 = x86_64::_mm256_cvtepi32_ps(c84);

                        x86_64::_mm_prefetch(w1.add(idx) as *const i8, x86_64::_MM_HINT_T0);
                        let x81 = x86_64::_mm256_load_ps(w1.add(idx));
                        let x82 = x86_64::_mm256_load_ps(w1.add(idx + 8));
                        let x83 = x86_64::_mm256_load_ps(w1.add(idx + 16));
                        let x84 = x86_64::_mm256_load_ps(w1.add(idx + 24));

                        if true {  // fma
                            sum8 = x86_64::_mm256_fmadd_ps(x81, f81, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x82, f82, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x83, f83, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x84, f84, sum8);
                        } else {
                            let mul1 = x86_64::_mm256_mul_ps(x81, f81);
                            let mul2 = x86_64::_mm256_mul_ps(x82, f82);
                            let mul3 = x86_64::_mm256_mul_ps(x83, f83);
                            let mul4 = x86_64::_mm256_mul_ps(x84, f84);

                            let sum12 = x86_64::_mm256_add_ps(mul1, mul2);
                            let sum34 = x86_64::_mm256_add_ps(mul3, mul4);
                            let sum1234 = x86_64::_mm256_add_ps(sum12, sum34);
                            sum8 = x86_64::_mm256_add_ps(sum8, sum1234);
                        }
                    }
                }
                unsafe {
                    x86_64::_mm256_storeu_ps(res8, sum8);
                }
            }
            unsafe {
                let x1 = x86_64::_mm256_load_ps(sum88.as_ptr());
                let x2 = x86_64::_mm256_load_ps(sum88.as_ptr().add(8));
                let x3 = x86_64::_mm256_load_ps(sum88.as_ptr().add(16));
                let x4 = x86_64::_mm256_load_ps(sum88.as_ptr().add(24));
                let x5 = x86_64::_mm256_load_ps(sum88.as_ptr().add(32));
                let x6 = x86_64::_mm256_load_ps(sum88.as_ptr().add(40));
                let x7 = x86_64::_mm256_load_ps(sum88.as_ptr().add(48));
                let x8 = x86_64::_mm256_load_ps(sum88.as_ptr().add(56));

                let xl12 = x86_64::_mm256_unpacklo_ps(x1, x2);
                let xh12 = x86_64::_mm256_unpackhi_ps(x1, x2);
                let xl34 = x86_64::_mm256_unpacklo_ps(x3, x4);
                let xh34 = x86_64::_mm256_unpackhi_ps(x3, x4);
                let xl56 = x86_64::_mm256_unpacklo_ps(x5, x6);
                let xh56 = x86_64::_mm256_unpackhi_ps(x5, x6);
                let xl78 = x86_64::_mm256_unpacklo_ps(x7, x8);
                let xh78 = x86_64::_mm256_unpackhi_ps(x7, x8);

                let x12 = x86_64::_mm256_add_ps(xl12, xh12);
                let x34 = x86_64::_mm256_add_ps(xl34, xh34);
                let x56 = x86_64::_mm256_add_ps(xl56, xh56);
                let x78 = x86_64::_mm256_add_ps(xl78, xh78);

                let x1234 = x86_64::_mm256_shuffle_ps(x12, x34, 0x44);
                let xabcd = x86_64::_mm256_shuffle_ps(x12, x34, 0xee);
                let x5678 = x86_64::_mm256_shuffle_ps(x56, x78, 0x44);
                let xefgh = x86_64::_mm256_shuffle_ps(x56, x78, 0xee);

                let xabcd = x86_64::_mm256_add_ps(x1234, xabcd);
                let xefgh = x86_64::_mm256_add_ps(x5678, xefgh);

                let x1234 = x86_64::_mm256_permute2f128_ps(xabcd, xefgh, 0x20);
                let x5678 = x86_64::_mm256_permute2f128_ps(xabcd, xefgh, 0x31);

                let h18 = x86_64::_mm256_add_ps(x1234, x5678);

                // teban
                let wtbn = x86_64::_mm256_load_ps(wtbn.as_ptr().add(hidx));
                let tbn = x86_64::_mm256_set1_ps(teban);
                let tbn8 = x86_64::_mm256_mul_ps(wtbn, tbn);
                let h18 = x86_64::_mm256_add_ps(h18, tbn8);
                // fixed stones
                let wfsb8 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx));
                let fsb = x86_64::_mm256_set1_ps(fs.0 as f32);
                let fsb8 = x86_64::_mm256_mul_ps(wfsb8, fsb);
                let wfsw8 = x86_64::_mm256_load_ps(
                        wfs.as_ptr().add(hidx + N_HIDDEN));
                let fsw = x86_64::_mm256_set1_ps(fs.1 as f32);
                let fsw8 = x86_64::_mm256_mul_ps(wfsw8, fsw);
                let fsbw = x86_64::_mm256_add_ps(fsb8, fsw8);
                let h18 = x86_64::_mm256_add_ps(h18, fsbw);
                // dc
                let wdc8 = x86_64::_mm256_load_ps(wdc.as_ptr().add(hidx));
                let h1234 = x86_64::_mm256_add_ps(h18, wdc8);

                // hidden[i] = hidsum;
                x86_64::_mm256_storeu_ps(hidden.as_mut_ptr().add(hidx), h1234);

                // exp(-x)
                let emx8 = Weight::expmx_ps_simd256(h1234);
                let one = x86_64::_mm256_set1_ps(1.0);
                // 1 + exp(-x)
                let hsp18 = x86_64::_mm256_add_ps(emx8, one);
                // 1 / (1 + exp(-x))
                let rhsp18 = x86_64::_mm256_rcp_ps(hsp18);
                let muls = x86_64::_mm256_mul_ps(
                        hsp18, x86_64::_mm256_mul_ps(rhsp18, rhsp18));
                let rhsp18 = x86_64::_mm256_sub_ps(
                        x86_64::_mm256_add_ps(rhsp18, rhsp18), muls);
                // let rhsp18 = x86_64::_mm256_div_ps(one, hsp18);

                // hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
                x86_64::_mm256_storeu_ps(hidsig.as_mut_ptr().add(hidx), rhsp18);

                let wh4 = x86_64::_mm256_load_ps(wh.as_ptr().add(hidx));
                let y4 = x86_64::_mm256_mul_ps(wh4, rhsp18);
                // let y4 = x86_64::_mm256_div_ps(wh4, hsp18);

                x86_64::_mm256_store_ps(sumarr.as_mut_ptr(), y4);
            }
            for s in sumarr {
                sum += s;
            }
        }
        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    // note: calc hidden layer w/ simd is slow now...
    #[cfg(target_arch="x86_64")]
    pub fn forwardv3bb_simdavx(&self, ban : &bitboard::BitBoard)
         -> ([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8)) {
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
        let mut hidsum : [f32 ; N] = [0.0 ; N];
        let mut emx : [f32 ; N] = [0.0 ; N];
        // let mut sumarr : [f32 ; N] = [0.0 ; N];
        let mut sm1 = [0.0f32 ; 1];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * 4] = [0.0 ; N * 4];

            for n in 0..N {
                let res8 = sum44[n * 4..].as_mut_ptr();
                let w1 = unsafe {ow.as_ptr().add((hidx + n) * board::CELL_2D)};
                let mut sum4: x86_64::__m128 =
                        unsafe {x86_64::_mm_setzero_ps()};
                const M : usize = 32;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;

                    let b81 = (bit8 & black) >> 4 * j;
                    let w81 = (bit8 & white) >> 4 * j;
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> 4 * j + 1;
                    let w82 = (bit8 & white) >> 4 * j + 1;
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> 4 * j + 2;
                    let w83 = (bit8 & white) >> 4 * j + 2;
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> 4 * j + 3;
                    let w84 = (bit8 & white) >> 4 * j + 3;
                    bit8 <<= 1;

                    unsafe {
                        let c1 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b81 as i64), x86_64::_mm_set1_epi64x(w81 as i64));
                        let c2 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b82 as i64), x86_64::_mm_set1_epi64x(w82 as i64));
                        let c3 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b83 as i64), x86_64::_mm_set1_epi64x(w83 as i64));
                        let c4 = x86_64::_mm_sub_epi8(x86_64::_mm_set1_epi64x(b84 as i64), x86_64::_mm_set1_epi64x(w84 as i64));

                        let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                        let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                        let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                        let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                        let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                        let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                        let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                        let f84 = x86_64::_mm256_cvtepi32_ps(c84);

                        x86_64::_mm_prefetch(w1.add(idx) as *const i8, x86_64::_MM_HINT_T0);
                        let x81 = x86_64::_mm256_load_ps(w1.add(idx));
                        let x82 = x86_64::_mm256_load_ps(w1.add(idx + 8));
                        let x83 = x86_64::_mm256_load_ps(w1.add(idx + 16));
                        let x84 = x86_64::_mm256_load_ps(w1.add(idx + 24));

                        let mut sum8;
                        if true {  // fma
                            sum8 = x86_64::_mm256_mul_ps(x81, f81);
                            sum8 = x86_64::_mm256_fmadd_ps(x82, f82, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x83, f83, sum8);
                            sum8 = x86_64::_mm256_fmadd_ps(x84, f84, sum8);
                        } else {
                            let mul1 = x86_64::_mm256_mul_ps(x81, f81);
                            let mul2 = x86_64::_mm256_mul_ps(x82, f82);
                            let mul3 = x86_64::_mm256_mul_ps(x83, f83);
                            let mul4 = x86_64::_mm256_mul_ps(x84, f84);

                            let sum12 = x86_64::_mm256_add_ps(mul1, mul2);
                            let sum34 = x86_64::_mm256_add_ps(mul3, mul4);
                            let sum1234 = x86_64::_mm256_add_ps(sum12, sum34);
                            sum8 = sum1234;
                        }
                        let hi4 = x86_64::_mm256_extractf128_ps(sum8, 1);
                        let lo4 = x86_64::_mm256_castps256_ps128(sum8);
                        let sm4 = x86_64::_mm_add_ps(lo4, hi4);
                        sum4 = x86_64::_mm_add_ps(sum4, sm4);
                    }
                }
                unsafe {
                    x86_64::_mm_storeu_ps(res8, sum4);
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
                x86_64::_mm_store_ps(hidsum.as_mut_ptr(), h1234);
            }
            hidden[i * N .. i * N + N].copy_from_slice(&hidsum);
            Weight::expmx_ps(hidsum.as_ptr(), emx.as_mut_ptr());
            unsafe {
                let emx4 = x86_64::_mm_load_ps(emx.as_ptr());
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());

                let y4 = x86_64::_mm_div_ps(wh4, hsp14);

                let hidsig4 = x86_64::_mm_div_ps(one, hsp14);
                x86_64::_mm_store_ps(hidsig[i * N..].as_mut_ptr(), hidsig4);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                // x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
                // sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
                let hi2 = x86_64::_mm_movehl_ps(y4, y4);
                let sm2 = x86_64::_mm_add_ps(y4, hi2);
                let lo2 = x86_64::_mm_shuffle_ps(sm2, sm2, 0x1);
                let sm = x86_64::_mm_add_ss(sm2, lo2);
                x86_64::_mm_store_ss(sm1.as_mut_ptr(), sm);
            }
            sum += sm1[0];
            // for n in 0..N {
            //     sum += sumarr[n];
            // }
            // sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }

        output[0] = sum;
        (hidden, hidsig, output, fs)
    }

    /// train weights
    /// 
    /// # Arguments
    /// - `self` : self
    /// - `rfen` : RFEN
    /// - `winner` : winner or # of stones.
    /// - `eta` : learning ratio.
    /// - `mid` : last (mid) moves will not be used. all rfen will be used w/ zero.
    /// 
    /// # Returns
    /// - OK(()) if succeeded.
    /// - Err(String) if some error happened.
    pub fn train(&mut self, rfen : &str, winner : i8, eta : f32, mid : u32)
             -> Result<(), String> {
        if cfg!(feature="bitboard") {
            let ban = match bitboard::BitBoard::from(rfen) {
                Ok(b) => {b},
                Err(e) => {return Err(e)}
            };
            if ban.nblank() < mid {return Ok(());}

            self.learnbb(&ban, winner, eta);
        } else {
            let ban = match board::Board::from(rfen) {
                Ok(b) => {b},
                Err(e) => {return Err(e)}
            };
            if ban.nblank() < mid {return Ok(());}

            self.learn(&ban, winner, eta);

            let ban = ban.rotate180();
            self.learn(&ban, winner, eta);
        }
        Ok(())
    }

    /// train weights
    /// 
    /// # Arguments
    /// - `self` : self
    /// - `ban` : Bitboard
    /// - `winner` : winner or # of stones.
    /// - `eta` : learning ratio.
    /// - `mid` : last (mid) moves will not be used. all rfen will be used w/ zero.
    /// 
    /// # Returns
    /// - OK(()) if succeeded.
    /// - Err(String) if some error happened.
    pub fn train_bitboard(&mut self,
            ban : &bitboard::BitBoard, winner : i8, eta : f32, mid : u32)
             -> Result<(), String> {
        if ban.nblank() < mid {return Ok(());}

        self.learnbb(&ban, winner, eta);
        Ok(())
    }

    /// train weights w/ mini batch
    /// 
    /// # Arguments
    /// - `self` : self
    /// - `banscores` : Bitboard and result(# of stones)
    /// - `eta` : learning ratio.
    /// - `dfw` : buffer to store weight difference.
    /// 
    /// # Returns
    /// - OK(()) if succeeded.
    /// - Err(String) if some error happened.
    pub fn train_bitboard_mb(&mut self,
        banscores : &[&(bitboard::BitBoard, i8)], eta : f32, dfw : &mut Weight)
         -> Result<(), String> {
        self.learnbbminib(banscores, eta, dfw);
        Ok(())
    }

    /// train weights
    /// 
    /// # Arguments
    /// - `self` : self
    /// - `rfen` : RFEN
    /// - `winner` : winner or # of stones.
    /// - `eta` : learning ratio.
    /// - `mid` : last (mid) moves will not be used. all rfen will be used w/ zero.
    /// 
    /// # Returns
    /// - OK(()) if succeeded.
    /// - Err(String) if some error happened.
    pub fn train_rotate(&mut self, rfen : &str, winner : i8, eta : f32, mid : u32)
             -> Result<(), String> {
        if cfg!(feature="bitboard") {
            let ban = match bitboard::BitBoard::from(rfen) {
                Ok(b) => {b},
                Err(e) => {return Err(e)}
            };
            if ban.nblank() < mid {return Ok(());}

            self.learnbb(&ban, winner, eta);

            let ban = ban.rotate90();
            self.learnbb(&ban, winner, eta);

            let ban = ban.rotate180();
            self.learnbb(&ban, winner, eta);

            let ban = ban.rotate90();
            self.learnbb(&ban, winner, eta);
        } else {
            let ban = match board::Board::from(rfen) {
                Ok(b) => {b},
                Err(e) => {return Err(e)}
            };
            if ban.nblank() < mid {return Ok(());}

            self.learn(&ban, winner, eta);

            let ban = ban.rotate180();
            self.learn(&ban, winner, eta);
        }
        Ok(())
    }

    #[cfg(target_arch="x86_64")]
    fn backwardv1(&mut self,
            ban : &board::Board, winner : i8, eta : f32,
            hidden : &[f32 ; N_HIDDEN], hidsig : &[f32 ; N_HIDDEN],
            output : &[f32 ; N_OUTPUT]) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let w2 = &mut ow[w1sz * 4..];
        for i in 0..N_HIDDEN {
            w2[i] -= hidsig[i] * diff * eta;
        }
        w2[N_HIDDEN] -= diff * eta;

        let mut dhid = [0.0f32 ; N_HIDDEN];
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

    #[cfg(target_arch="aarch64")]
    fn backwardv1(&mut self,
            ban : &board::Board, winner : i8, eta : f32,
            hidden : &[f32 ; N_HIDDEN], hidsig : &[f32 ; N_HIDDEN],
            output : &[f32 ; N_OUTPUT]) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let w2 = &mut ow[w1sz * 4..];
        for i in 0..N_HIDDEN {
            w2[i] -= hidsig[i] * diff * eta;
        }
        w2[N_HIDDEN] -= diff * eta;

        let mut dhid = [0.0f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = w2[i] * diff;
            let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
            *h = tmp * sig * (1.0 - sig);
        }
        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let w1 = &mut ow[i * w1sz .. (i + 1) * w1sz];
            let heta = *h * eta;
            for (&c, w) in cells.iter().zip(w1.iter_mut()) {
                *w -= c as f32 * heta;
            }
            w1[board::CELL_2D] -= teban * heta;
            w1[board::CELL_2D + 1] -= heta;
        }        
    }

    #[cfg(target_arch="x86_64")]
    fn backwardv2(&mut self,
            ban : &board::Board, winner : i8, eta : f32,
            hidden : &[f32 ; N_HIDDEN], hidsig : &[f32 ; N_HIDDEN],
            output : &[f32 ; N_OUTPUT]) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
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

        let mut dhid = [0.0f32 ; N_HIDDEN];
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
                        let c8 = x86_64::_mm_load_si128(
                            cells[idx..].as_ptr() as *const x86_64::__m128i);
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

    #[cfg(target_arch="aarch64")]
    fn backwardv2(&mut self,
        ban : &board::Board, winner : i8, eta : f32,
        hidden : &[f32 ; N_HIDDEN], hidsig : &[f32 ; N_HIDDEN],
            output : &[f32 ; N_OUTPUT]) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let w2 = &mut ow[(board::CELL_2D + 2) * N_HIDDEN ..];
        let deta = diff * eta;
        for i in 0..N_HIDDEN {
            w2[i] -= hidsig[i] * deta;
        }
        w2[N_HIDDEN] -= deta;

        let mut dhid = [0.0f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = w2[i] * diff;
            let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
            *h = tmp * sig * (1.0 - sig);
        }
        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let w1 = &mut ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let heta = *h * eta;
            for (&c, w) in cells.iter().zip(w1.iter_mut()) {
                *w -= c as f32 * heta;
            }

            let tbndc = &mut ow[board::CELL_2D * N_HIDDEN ..];
            tbndc[i] -= teban * heta;
            tbndc[i + N_HIDDEN] -= heta;
        }
    }

    #[cfg(target_arch="x86_64")]
    pub fn backwardv3(&mut self,
        ban : &board::Board, winner : i8, eta : f32,
        (hidden , hidsig , output , fs)
            : &([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8))) {
        let cells = &ban.cells;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let wh = &mut ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        let deta = diff * eta;
        // if cfg!(feature="nosimd") {
            for i in 0..N_HIDDEN {
                wh[i] -= hidsig[i] * deta;
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
        wh[N_HIDDEN] -= deta;

        let mut dhid = [0.0f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = wh[i] * diff;
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
                        let c8 = x86_64::_mm_load_si128(
                            cells[idx..].as_ptr() as *const x86_64::__m128i);
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
            let wtbn = &mut ow[board::CELL_2D * N_HIDDEN ..];
            wtbn[i] -= teban * heta;
            let wfs = &mut ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
            wfs[i] -= fs.0 as f32 * heta;
            wfs[i + N_HIDDEN] -= fs.1 as f32 * heta;
            let wdc = &mut ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
            wdc[i] -= heta;
        }
    }

    #[cfg(target_arch="aarch64")]
    pub fn backwardv3(&mut self,
        ban : &board::Board, winner : i8, eta : f32,
        (hidden , hidsig , output , fs)
            : &([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8))) {
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

        let mut dhid = [0.0f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = wh[i] * diff;
            let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
            *h = tmp * sig * (1.0 - sig);
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

    pub fn backwardv3bb(&mut self,
        ban : &bitboard::BitBoard, winner : i8, eta : f32,
        (hidden , hidsig , output , fs)
            : &([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8))) {
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

        let mut dhid = [0.0f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            // tmp = wo x diff
            let tmp = wh[i] * diff;
            // sig = 1 / (1 + exp(-hidden[i]))
            let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
            // h = wo x diff x sig x (1 - sig)
            *h = tmp * sig * (1.0 - sig);
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

    #[cfg(target_arch="x86_64")]
    pub fn backwardv3bb_simd(&mut self,
        ban : &bitboard::BitBoard, winner : i8, eta : f32,
        (hidden , hidsig , output , fs)
            : &([f32 ; N_HIDDEN], [f32 ; N_HIDDEN], [f32 ; N_OUTPUT], (i8, i8))) {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - winner as f32;
        let wh = &mut ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];
        let deta = diff * eta;
        // if cfg!(feature="nosimd") {
            for i in 0..N_HIDDEN {
                wh[i] -= hidsig[i] * deta;
            }
        // } else if cfg!(feature="avx") {
        // } else {  // sse
            // // impossible for N_HIDDEN:8
            // for i in 0..N_HIDDEN / 16 {
            //     let hidx = i * 16;
            //     unsafe {
            //         // if true {
            //         //     let h41 = x86_64::_mm256_load_ps(hidsig[hidx..].as_ptr());
            //         //     let w41 = x86_64::_mm256_load_ps(wh[hidx..].as_ptr());
            //         //     let h42 =
            //         //     x86_64::_mm256_load_ps(hidsig[hidx + 8..].as_ptr());
            //         //     let w42 = x86_64::_mm256_load_ps(wh[hidx + 8..].as_ptr());
            //         //     // -a x b + c;
            //         //     let deta4 = x86_64::_mm256_set1_ps(deta);
            //         //     let y41 = x86_64::_mm256_fnmadd_ps(deta4, h41, w41);
            //         //     let y42 = x86_64::_mm256_fnmadd_ps(deta4, h42, w42);
            //         //     x86_64::_mm256_store_ps(wh[hidx..].as_mut_ptr(), y41);
            //         //     x86_64::_mm256_store_ps(
            //         //             wh[hidx + 8..].as_mut_ptr(), y42);
            //         // } else {
            //         //     let h41 = x86_64::_mm256_load_ps(hidsig[hidx..].as_ptr());
            //         //     let h42 =
            //         //         x86_64::_mm256_load_ps(hidsig[hidx + 8..].as_ptr());
            //         //     let w41 = x86_64::_mm256_load_ps(wh[hidx..].as_ptr());
            //         //     let w42 = x86_64::_mm256_load_ps(wh[hidx + 8..].as_ptr());
            //         //     let deta4 = x86_64::_mm256_set1_ps(deta);
            //         //     let hdeta1 = x86_64::_mm256_mul_ps(deta4, h41);
            //         //     let hdeta2 = x86_64::_mm256_mul_ps(deta4, h42);
            //         //     let y41 = x86_64::_mm256_sub_ps(w41, hdeta1);
            //         //     let y42 = x86_64::_mm256_sub_ps(w42, hdeta2);
            //         //     x86_64::_mm256_store_ps(wh[hidx..].as_mut_ptr(), y41);
            //         //     x86_64::_mm256_store_ps(
            //         //             wh[hidx + 8..].as_mut_ptr(), y42);
            //         // }
            //     }
            // }
        // } else {  // sse
        // slow for N_HIDDEN:4
        //     for i in 0..N_HIDDEN / 4 {
        //         let hidx = i * 4;
        //         unsafe {
        //             let w4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());
        //             let h4 = x86_64::_mm_load_ps(hidsig[hidx..].as_ptr());
        //             let deta4 = x86_64::_mm_set1_ps(deta);
        //             let hdeta = x86_64::_mm_mul_ps(deta4, h4);
        //             let y4 = x86_64::_mm_sub_ps(w4, hdeta);
        //             x86_64::_mm_storeu_ps(wh[hidx..].as_mut_ptr(), y4);
        //         }
        //     }
        // }
        wh[N_HIDDEN] -= deta;

        let mut dhid = [0.0f32 ; N_HIDDEN];
        if cfg!(feature="avx") {
            unsafe {
                let diff4 = x86_64::_mm256_set1_ps(diff);
                let one = x86_64::_mm256_set1_ps(1.0);
                for i in 0..N_HIDDEN / 8 {
                    let idx = i * 8;
                    let wh4 = x86_64::_mm256_loadu_ps(wh[idx..].as_ptr());
                    // tmp = wh x diff
                    let tmp = x86_64::_mm256_mul_ps(wh4, diff4);
                    // sig = 1 / (1 + exp(-hidden[i]))
                    let hid4 = x86_64::_mm256_loadu_ps(hidden[idx..].as_ptr());
                    let emx = Weight::expmx_ps_simd256(hid4);
                    let onemx = x86_64::_mm256_add_ps(one, emx);

                    // let sig = x86_64::_mm256_div_ps(one, onemx);
                    let rcp = x86_64::_mm256_rcp_ps(onemx);
                    let sqrcp = x86_64::_mm256_mul_ps(rcp, rcp);
                    let rcp2rcp = x86_64::_mm256_mul_ps(onemx, sqrcp);
                    let rcp2 = x86_64::_mm256_add_ps(rcp, rcp);
                    let sig = x86_64::_mm256_sub_ps(rcp2, rcp2rcp);

                    // h = wh x diff x sig x (1 - sig)
                    let tmp2 = x86_64::_mm256_mul_ps(tmp, sig);
                    let onessig = x86_64::_mm256_sub_ps(one, sig);
                    let h4 = x86_64::_mm256_mul_ps(tmp2, onessig);

                    x86_64::_mm256_storeu_ps(dhid[idx..].as_mut_ptr(), h4);
                }
            }
        } else {
            for (i, h) in dhid.iter_mut().enumerate() {
                // tmp = wo x diff
                let tmp = wh[i] * diff;
                // sig = 1 / (1 + exp(-hidden[i]))
                let sig = 1.0 / (1.0 + f32::exp(-hidden[i]));
                // h = wo x diff x sig x (1 - sig)
                *h = tmp * sig * (1.0 - sig);
            }
        }

        let wtbn = unsafe {ow.as_mut_ptr().add(board::CELL_2D * N_HIDDEN)};
        let wfs =
            unsafe {ow.as_mut_ptr().add((board::CELL_2D + 1) * N_HIDDEN)};
        let wdc =
            unsafe {ow.as_mut_ptr().add((board::CELL_2D + 1 + 2) * N_HIDDEN)};

        // back to input
        // for (i, h) in dhid.iter().enumerate() {
        for k in 0..N_HIDDEN / 8 {
            for i in 0..8 {
                let hidx = k * 8 + i;
                let heta = dhid[hidx] * eta;
                let w1 = unsafe {ow.as_mut_ptr().add(hidx * board::CELL_2D)};
                let heta8 = unsafe {x86_64::_mm256_set1_ps(heta)};
                let mut bit8 = 0x0101010101010101u64;
                for j in 0..board::CELL_2D / 32 {
                    let idx = j * 32;
                    let b81 = (bit8 & black) >> (4 * j);
                    let w81 = (bit8 & white) >> (4 * j);
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> (4 * j + 1);
                    let w82 = (bit8 & white) >> (4 * j + 1);
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> (4 * j + 2);
                    let w83 = (bit8 & white) >> (4 * j + 2);
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> (4 * j + 3);
                    let w84 = (bit8 & white) >> (4 * j + 3);
                    bit8 <<= 1;

                    unsafe {
                        let b161 = x86_64::_mm_set1_epi64x(b81 as i64);
                        let w161 = x86_64::_mm_set1_epi64x(w81 as i64);
                        let b162 = x86_64::_mm_set1_epi64x(b82 as i64);
                        let w162 = x86_64::_mm_set1_epi64x(w82 as i64);
                        let b163 = x86_64::_mm_set1_epi64x(b83 as i64);
                        let w163 = x86_64::_mm_set1_epi64x(w83 as i64);
                        let b164 = x86_64::_mm_set1_epi64x(b84 as i64);
                        let w164 = x86_64::_mm_set1_epi64x(w84 as i64);

                        let c161 = x86_64::_mm_sub_epi8(b161, w161);
                        let c162 = x86_64::_mm_sub_epi8(b162, w162);
                        let c163 = x86_64::_mm_sub_epi8(b163, w163);
                        let c164 = x86_64::_mm_sub_epi8(b164, w164);

                        let imul1 = x86_64::_mm256_cvtepi8_epi32(c161);
                        let imul2 = x86_64::_mm256_cvtepi8_epi32(c162);
                        let imul3 = x86_64::_mm256_cvtepi8_epi32(c163);
                        let imul4 = x86_64::_mm256_cvtepi8_epi32(c164);
                        let mul1 = x86_64::_mm256_cvtepi32_ps(imul1);
                        let mul2 = x86_64::_mm256_cvtepi32_ps(imul2);
                        let mul3 = x86_64::_mm256_cvtepi32_ps(imul3);
                        let mul4 = x86_64::_mm256_cvtepi32_ps(imul4);
                        let df1 = x86_64::_mm256_mul_ps(mul1, heta8);
                        let df2 = x86_64::_mm256_mul_ps(mul2, heta8);
                        let df3 = x86_64::_mm256_mul_ps(mul3, heta8);
                        let df4 = x86_64::_mm256_mul_ps(mul4, heta8);
                        // w = x - h x eta x sengo
                        x86_64::_mm_prefetch(w1.add(idx) as *const i8, x86_64::_MM_HINT_NTA);
                        let x41 = x86_64::_mm256_load_ps(w1.add(idx));
                        let x42 = x86_64::_mm256_load_ps(w1.add(idx + 8));
                        let x43 = x86_64::_mm256_load_ps(w1.add(idx + 16));
                        let x44 = x86_64::_mm256_load_ps(w1.add(idx + 24));
                        let w41 = x86_64::_mm256_sub_ps(x41, df1);
                        let w42 = x86_64::_mm256_sub_ps(x42, df2);
                        let w43 = x86_64::_mm256_sub_ps(x43, df3);
                        let w44 = x86_64::_mm256_sub_ps(x44, df4);
                        x86_64::_mm256_store_ps(w1.add(idx), w41);
                        x86_64::_mm256_store_ps(w1.add(idx + 8), w42);
                        x86_64::_mm256_store_ps(w1.add(idx + 16), w43);
                        x86_64::_mm256_store_ps(w1.add(idx + 24), w44);
                    }
                }
            }
            unsafe {
                if cfg!(feature="avx") {
                    let hid = x86_64::_mm256_loadu_ps(dhid[k * 8..].as_ptr());
                    let eta = x86_64::_mm256_set1_ps(eta);
                    let heta8 = x86_64::_mm256_mul_ps(hid, eta);
                    let tbn = x86_64::_mm256_set1_ps(teban);
                    let htbn = x86_64::_mm256_mul_ps(tbn, heta8);
                    let tbn = x86_64::_mm256_load_ps(wtbn.add(k * 8));
                    let tbn = x86_64::_mm256_sub_ps(tbn, htbn);
                    x86_64::_mm256_store_ps(wtbn.add(k * 8), tbn);
                    let fs0 = x86_64::_mm256_set1_ps(fs.0 as f32);
                    let hfs0 = x86_64::_mm256_mul_ps(fs0, heta8);
                    let fs0 = x86_64::_mm256_load_ps(wfs.add(k * 8));
                    let fs0 = x86_64::_mm256_sub_ps(fs0, hfs0);
                    x86_64::_mm256_store_ps(wfs.add(k * 8), fs0);
                    let fs1 = x86_64::_mm256_set1_ps(fs.1 as f32);
                    let hfs1 = x86_64::_mm256_mul_ps(fs1, heta8);
                    let fs1 = x86_64::_mm256_load_ps(
                            wfs.add(k * 8 + N_HIDDEN));
                    let fs1 = x86_64::_mm256_sub_ps(fs1, hfs1);
                    x86_64::_mm256_store_ps(wfs.add(k * 8 + N_HIDDEN), fs1);
                    let dc = x86_64::_mm256_load_ps(wdc.add(k * 8));
                    let dc = x86_64::_mm256_sub_ps(dc, heta8);
                    x86_64::_mm256_store_ps(wdc.add(k * 8), dc);
                } else if cfg!(feature="nosimd") {
                    for n in 0..8 {
                        let idx = k * 8 + n;
                        let heta = dhid[idx] * eta;
                        *wtbn.add(idx) -= teban * heta;
                        *wfs.add(idx) -= fs.0 as f32 * heta;
                        *wfs.add(idx + N_HIDDEN) -= fs.1 as f32 * heta;
                        *wdc.add(idx) -= heta;
                    }
                } else {
                    let hid1 = x86_64::_mm_loadu_ps(dhid[k * 8..].as_ptr());
                    let hid2 = x86_64::_mm_loadu_ps(dhid[k * 8 + 4..].as_ptr());
                    let eta = x86_64::_mm_set1_ps(eta);
                    let heta1 = x86_64::_mm_mul_ps(hid1, eta);
                    let heta2 = x86_64::_mm_mul_ps(hid2, eta);
                    let tbn = x86_64::_mm_set1_ps(teban);
                    let htbn1 = x86_64::_mm_mul_ps(tbn, heta1);
                    let htbn2 = x86_64::_mm_mul_ps(tbn, heta2);
                    let tbn1 = x86_64::_mm_load_ps(wtbn.add(k * 8));
                    let tbn2 = x86_64::_mm_load_ps(wtbn.add(k * 8 + 4));
                    let tbn1 = x86_64::_mm_sub_ps(tbn1, htbn1);
                    let tbn2 = x86_64::_mm_sub_ps(tbn2, htbn2);
                    x86_64::_mm_store_ps(wtbn.add(k * 8), tbn1);
                    x86_64::_mm_store_ps(wtbn.add(k * 8 + 4), tbn2);
                    let fs0 = x86_64::_mm_set1_ps(fs.0 as f32);
                    let hfs01 = x86_64::_mm_mul_ps(fs0, heta1);
                    let hfs02 = x86_64::_mm_mul_ps(fs0, heta2);
                    let fs01 = x86_64::_mm_load_ps(wfs.add(k * 8));
                    let fs02 = x86_64::_mm_load_ps(wfs.add(k * 8 + 4));
                    let fs01 = x86_64::_mm_sub_ps(fs01, hfs01);
                    let fs02 = x86_64::_mm_sub_ps(fs02, hfs02);
                    x86_64::_mm_store_ps(wfs.add(k * 8), fs01);
                    x86_64::_mm_store_ps(wfs.add(k * 8 + 4), fs02);
                    let fs1 = x86_64::_mm_set1_ps(fs.1 as f32);
                    let hfs11 = x86_64::_mm_mul_ps(fs1, heta1);
                    let hfs12 = x86_64::_mm_mul_ps(fs1, heta2);
                    let fs11 = x86_64::_mm_load_ps(wfs.add(k * 8 + N_HIDDEN));
                    let fs12 = x86_64::_mm_load_ps(
                            wfs.add(k * 8 + N_HIDDEN + 4));
                    let fs11 = x86_64::_mm_sub_ps(fs11, hfs11);
                    let fs12 = x86_64::_mm_sub_ps(fs12, hfs12);
                    x86_64::_mm_store_ps(wfs.add(k * 8 + N_HIDDEN), fs11);
                    x86_64::_mm_store_ps(wfs.add(k * 8 + N_HIDDEN + 4), fs12);
                    let dc1 = x86_64::_mm_load_ps(wdc.add(k * 8));
                    let dc2 = x86_64::_mm_load_ps(wdc.add(k * 8 + 4));
                    let dc1 = x86_64::_mm_sub_ps(dc1, heta1);
                    let dc2 = x86_64::_mm_sub_ps(dc2, heta2);
                    x86_64::_mm_store_ps(wdc.add(k * 8), dc1);
                    x86_64::_mm_store_ps(wdc.add(k * 8 + 4), dc2);
                }
            }
        }
    }

    #[cfg(target_arch="x86_64")]
    pub fn updatemb(&mut self, dfw : &Weight, n : usize) {
        if cfg!(feature="nosimd") {
            for (wi, wo) in dfw.weight.iter().zip(self.weight.iter_mut()) {
                *wo += *wi / n as f32;
            }
        } else if cfg!(feature="avx") {
            let m = N_WEIGHT / 32;
            let wi = &dfw.weight;
            let wo = &mut self.weight;
            for i in 0..m {
                let idx = i * 32;
                unsafe {
                    let i1 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx));
                    let i2 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx + 8));
                    let i3 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx + 16));
                    let i4 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx + 24));
                    let nn = x86_64::_mm256_set1_ps(1.0 / n as f32);
                    let in1 = x86_64::_mm256_mul_ps(i1, nn);
                    let in2 = x86_64::_mm256_mul_ps(i2, nn);
                    let in3 = x86_64::_mm256_mul_ps(i3, nn);
                    let in4 = x86_64::_mm256_mul_ps(i4, nn);
                    let o1 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx));
                    let o2 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx + 8));
                    let o3 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx + 16));
                    let o4 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx + 24));
                    let nw1 = x86_64::_mm256_add_ps(o1, in1);
                    let nw2 = x86_64::_mm256_add_ps(o2, in2);
                    let nw3 = x86_64::_mm256_add_ps(o3, in3);
                    let nw4 = x86_64::_mm256_add_ps(o4, in4);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx), nw1);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx + 8), nw2);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx + 16), nw3);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx + 24), nw4);
                }
            }
            for i in m * 32..N_WEIGHT {
                wo[i] += wi[i] / n as f32;
            }
        } else {
            let m = N_WEIGHT / 16;
            let wi = &dfw.weight;
            let wo = &mut self.weight;
            for i in 0..m {
                let idx = i * 16;
                unsafe {
                    let i1 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx));
                    let i2 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx + 4));
                    let i3 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx + 8));
                    let i4 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx + 12));
                    let nn = x86_64::_mm_set1_ps(1.0 / n as f32);
                    let in1 = x86_64::_mm_mul_ps(i1, nn);
                    let in2 = x86_64::_mm_mul_ps(i2, nn);
                    let in3 = x86_64::_mm_mul_ps(i3, nn);
                    let in4 = x86_64::_mm_mul_ps(i4, nn);
                    let o1 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx));
                    let o2 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx + 4));
                    let o3 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx + 8));
                    let o4 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx + 12));
                    let nw1 = x86_64::_mm_add_ps(o1, in1);
                    let nw2 = x86_64::_mm_add_ps(o2, in2);
                    let nw3 = x86_64::_mm_add_ps(o3, in3);
                    let nw4 = x86_64::_mm_add_ps(o4, in4);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx), nw1);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx + 4), nw2);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx + 8), nw3);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx + 12), nw4);
                }
            }
            for i in m * 16..N_WEIGHT {
                wo[i] += wi[i] / n as f32;
            }
        }
    }

    #[cfg(target_arch="x86_64")]
    pub fn updatemb2(&mut self, dfw : &Weight, dfw2 : &Weight, n : usize) {
        if cfg!(feature="nosimd") {
            for (wi, (wi2, wo)) in dfw.weight.iter().zip(dfw2.weight.iter().zip(self.weight.iter_mut())) {
                *wo += (*wi + *wi2) / n as f32;
            }
        } else if cfg!(feature="avx") {
            let m = N_WEIGHT / 32;
            let wi = &dfw.weight;
            let wi2 = &dfw2.weight;
            let wo = &mut self.weight;
            for i in 0..m {
                let idx = i * 32;
                unsafe {
                    let i1 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx));
                    let i2 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx + 8));
                    let i3 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx + 16));
                    let i4 = x86_64::_mm256_loadu_ps(wi.as_ptr().add(idx + 24));
                    let i12 = x86_64::_mm256_loadu_ps(wi2.as_ptr().add(idx));
                    let i22 = x86_64::_mm256_loadu_ps(wi2.as_ptr().add(idx + 8));
                    let i32 = x86_64::_mm256_loadu_ps(wi2.as_ptr().add(idx + 16));
                    let i42 = x86_64::_mm256_loadu_ps(wi2.as_ptr().add(idx + 24));
                    let in1 = x86_64::_mm256_add_ps(i1, i12);
                    let in2 = x86_64::_mm256_add_ps(i2, i22);
                    let in3 = x86_64::_mm256_add_ps(i3, i32);
                    let in4 = x86_64::_mm256_add_ps(i4, i42);
                    let nn = x86_64::_mm256_set1_ps(1.0 / n as f32);
                    let in1 = x86_64::_mm256_mul_ps(in1, nn);
                    let in2 = x86_64::_mm256_mul_ps(in2, nn);
                    let in3 = x86_64::_mm256_mul_ps(in3, nn);
                    let in4 = x86_64::_mm256_mul_ps(in4, nn);
                    let o1 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx));
                    let o2 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx + 8));
                    let o3 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx + 16));
                    let o4 = x86_64::_mm256_loadu_ps(wo.as_ptr().add(idx + 24));
                    let nw1 = x86_64::_mm256_add_ps(o1, in1);
                    let nw2 = x86_64::_mm256_add_ps(o2, in2);
                    let nw3 = x86_64::_mm256_add_ps(o3, in3);
                    let nw4 = x86_64::_mm256_add_ps(o4, in4);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx), nw1);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx + 8), nw2);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx + 16), nw3);
                    x86_64::_mm256_storeu_ps(wo.as_mut_ptr().add(idx + 24), nw4);
                }
            }
            for i in m * 32..N_WEIGHT {
                wo[i] += (wi[i] + wi2[i]) / n as f32;
            }
        } else {
            let m = N_WEIGHT / 16;
            let wi = &dfw.weight;
            let wi2 = &dfw2.weight;
            let wo = &mut self.weight;
            for i in 0..m {
                let idx = i * 16;
                unsafe {
                    let i1 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx));
                    let i2 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx + 8));
                    let i3 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx + 16));
                    let i4 = x86_64::_mm_loadu_ps(wi.as_ptr().add(idx + 24));
                    let i12 = x86_64::_mm_loadu_ps(wi2.as_ptr().add(idx));
                    let i22 = x86_64::_mm_loadu_ps(wi2.as_ptr().add(idx + 8));
                    let i32 = x86_64::_mm_loadu_ps(wi2.as_ptr().add(idx + 16));
                    let i42 = x86_64::_mm_loadu_ps(wi2.as_ptr().add(idx + 24));
                    let in1 = x86_64::_mm_add_ps(i1, i12);
                    let in2 = x86_64::_mm_add_ps(i2, i22);
                    let in3 = x86_64::_mm_add_ps(i3, i32);
                    let in4 = x86_64::_mm_add_ps(i4, i42);
                    let nn = x86_64::_mm_set1_ps(1.0 / n as f32);
                    let in1 = x86_64::_mm_mul_ps(in1, nn);
                    let in2 = x86_64::_mm_mul_ps(in2, nn);
                    let in3 = x86_64::_mm_mul_ps(in3, nn);
                    let in4 = x86_64::_mm_mul_ps(in4, nn);
                    let o1 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx));
                    let o2 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx + 8));
                    let o3 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx + 16));
                    let o4 = x86_64::_mm_loadu_ps(wo.as_ptr().add(idx + 24));
                    let nw1 = x86_64::_mm_add_ps(o1, in1);
                    let nw2 = x86_64::_mm_add_ps(o2, in2);
                    let nw3 = x86_64::_mm_add_ps(o3, in3);
                    let nw4 = x86_64::_mm_add_ps(o4, in4);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx), nw1);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx + 8), nw2);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx + 16), nw3);
                    x86_64::_mm_storeu_ps(wo.as_mut_ptr().add(idx + 24), nw4);
                }
            }
            for i in m * 16..N_WEIGHT {
                wo[i] += (wi[i] + wi2[i]) / n as f32;
            }
        }
    }

    #[cfg(target_arch="x86_64")]
    fn learn(&mut self, ban : &board::Board, winner : i8, eta : f32) {
        if cfg!(feature="nnv1") {
            // forward
            let (hidden, hidsig, output) =
                if cfg!(feature="nosimd") {
                    self.forwardv1(&ban)
                } else {
                    self.forwardv1_simd(&ban)
                };
            // backward
            self.backwardv1(ban, winner, eta, &hidden, &hidsig, &output);
        } else if cfg!(feature="nnv2") {
            // forward
            let (hidden, hidsig, output) =
                if cfg!(feature="nosimd") {
                    self.forwardv2(&ban)
                } else {
                    self.forwardv2_simd(&ban)
                    // self.forwardv2_simd2(&ban)
                };
            // backward
            self.backwardv2(ban, winner, eta, &hidden, &hidsig, &output);
        } else {
            // forward
            let res = if cfg!(feature="nosimd") {
                    self.forwardv3(&ban)
                } else {
                    self.forwardv3_simd(&ban)
                };
            // backward
            self.backwardv3(ban, winner, eta, &res);
        }
    }

    #[cfg(target_arch="aarch64")]
    fn learn(&mut self, ban : &board::Board, winner : i8, eta : f32) {
        if cfg!(feature="nnv1") {
            // forward
            let (hidden, hidsig, output) = self.forwardv1(&ban);
            // backward
            self.backwardv1(ban, winner, eta, &hidden, &hidsig, &output);
        } else if cfg!(feature="nnv2") {
            // forward
            let (hidden, hidsig, output) = self.forwardv2(&ban);
            // backward
            self.backwardv2(ban, winner, eta, &hidden, &hidsig, &output);
        } else {
            // forward
            let res = self.forwardv3(&ban);
            // backward
            self.backwardv3(ban, winner, eta, &res);
        }
    }

    #[cfg(target_arch="x86_64")]
    fn learnbb(&mut self, ban : &bitboard::BitBoard, winner : i8, eta : f32) {
        // forward
        let res = if cfg!(feature="nosimd") {
                self.forwardv3bb(&ban)  // 27s(w/ h8)
            } else if cfg!(feature="avx") {
                self.forwardv3bb_simdavx3(&ban)  // 15s(w/ h8)
                // self.forwardv3bb_simdavx2(&ban)  // 15s(w/ h8)
                // self.forwardv3bb_simdavx(&ban)  // 28s(w/ h8)
            } else {
                self.forwardv3bb_simd(&ban)  // 15s(w/ h8)
            };
        // backward
        if cfg!(feature="nosimd") {
            self.backwardv3bb(ban, winner, eta, &res);
        } else {
            self.backwardv3bb_simd(ban, winner, eta, &res);
        }
    }

    #[cfg(target_arch="aarch64")]
    fn learnbb(&mut self, ban : &bitboard::BitBoard, winner : i8, eta : f32) {
        // forward
        let res = self.forwardv3bb(&ban);
        // backward
        self.backwardv3bb(ban, winner, eta, &res);
    }

    #[cfg(target_arch="x86_64")]
    fn learnbbdiff(&self, ban : &bitboard::BitBoard, winner : i8, eta : f32, dfw : &mut Weight) {
        // forward
        let res = if cfg!(feature="nosimd") {
                self.forwardv3bb(&ban)  // 27s(w/ h8)
            } else if cfg!(feature="avx") {
                self.forwardv3bb_simdavx3(&ban)  // 15s(w/ h8)
                // self.forwardv3bb_simdavx2(&ban)  // 15s(w/ h8)
                // self.forwardv3bb_simdavx(&ban)  // 28s(w/ h8)
            } else {
                self.forwardv3bb_simd(&ban)  // 15s(w/ h8)
            };
        // backward
        if cfg!(feature="nosimd") {
            dfw.backwardv3bb(ban, winner, eta, &res);
        } else {
            dfw.backwardv3bb_simd(ban, winner, eta, &res);
        }
    }

    #[cfg(target_arch="aarch64")]
    fn learnbbdiff(&self, ban : &bitboard::BitBoard, winner : i8, eta : f32, dfw : &mut Weight) {
        // forward
        let res = self.forwardv3bb(&ban);
        // backward
        dfw.backwardv3bb(ban, winner, eta, &res);
    }

    fn learnbbminib(&self, banscores : &[&(bitboard::BitBoard, i8)], eta : f32, dfw : &mut Weight) {
        for (ban , winner) in banscores.iter() {
            self.learnbbdiff(ban, *winner, eta, dfw);
        }
    }
}

#[allow(dead_code)]
fn dbg_assert_eq_vec(va : &[f32], vb : &[f32]) -> bool {
    for (a, b) in va.iter().zip(vb.iter()) {
        if (a - b).abs() >= 2e-6 {
            println!("| {a} - {b} | >= 2e-6...");
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn dbg_assert_eq(a : &f32, b : &f32) -> bool {
    if (a - b).abs() >= 2e-6 {
        println!("| {a} - {b} | >= 2e-6...");
        return false;
    }
    true
}

#[cfg(target_arch="x86_64")]
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
    let limit = 2e-6;
    for rfen in rfens.iter() {
        for winner in -1..=1 {
            let bban = bitboard::BitBoard::from(rfen).unwrap();
            let ban = board::Board::from(rfen).unwrap();
            ban.put();
            let mut w = weight::Weight::new();
            w.init();
            let mut w2 = weight::Weight::new();
            w2.copy(&w);
            let mut w3 = weight::Weight::new();
            w3.copy(&w);
            let res_nosimde = w.evaluatev3bb(&bban);
            let res_simd = w.evaluatev3bb_simd(&bban);
            let res_simdavx = w.evaluatev3bb_simdavx(&bban);
            assert!((res_nosimde - res_simd).abs() < limit);
            assert!((res_nosimde - res_simdavx).abs() < limit);
            // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
            let (bh_ns, ah_ns, res_nosimd, fsns) = w.forwardv3bb(&bban);
            let (bh_s, ah_s, res_simd, fss) = w.forwardv3bb_simd(&bban);
            let (bh_sa, ah_sa, res_simdavx, fssa)
                    = w.forwardv3bb_simdavx(&bban);
            let (bh_sa2, ah_sa2, res_simdavx2, fssa2)
                    = w.forwardv3bb_simdavx3(&bban);
            assert!(dbg_assert_eq_vec(&bh_ns, &bh_s));
            assert!(dbg_assert_eq_vec(&bh_ns, &bh_sa));
            assert!(dbg_assert_eq_vec(&bh_ns, &bh_sa2));
            // println!("{bh_ns:?} == \n{bh_s:?} == \n{bh_sa:?} ???");
            assert!(dbg_assert_eq_vec(&ah_ns, &ah_s));
            assert!(dbg_assert_eq_vec(&ah_ns, &ah_sa));
            assert!(dbg_assert_eq_vec(&ah_ns, &ah_sa2));
            // println!("{ah_ns:?} == \n{ah_s:?} == \n{ah_sa:?} ???");
            assert!((res_nosimde - res_nosimd[0]).abs() < limit);
            // assert_eq!(res_nosimd, res_simd);
            assert!((res_nosimd[0] - res_simd[0]).abs() < limit);
            // assert_eq!(res_nosimd, res_simdavx);
            assert!((res_nosimd[0] - res_simdavx[0]).abs() < limit);
            // assert_eq!(res_nosimd, res_simdavx2);
            assert!((res_nosimd[0] - res_simdavx2[0]).abs() < limit);
            // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
            assert_eq!(fsns, fss);
            assert_eq!(fsns, fssa);
            assert_eq!(fsns, fssa2);
            // println!("{fsns:?} == {fss:?} == {fssa:?} ???");
            let res = w.forwardv3bb(&bban);
            // let winner = 1;
            let eta = 0.1;
            w.backwardv3bb(&bban, winner, eta, &res);
            w2.backwardv3bb_simd(&bban, winner, eta, &res);
            // let sv = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s = sv.join(",");
            // let sv2 = w2.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s2 = sv2.join(",");
            // assert_eq!(s, s2);
            assert!(dbg_assert_eq_vec(&w.weight, &w2.weight));
            let res = w3.forwardv3(&ban);
            w3.backwardv3(&ban, winner, eta, &res);
            // let sv3 = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s3 = sv3.join(",");
            // assert_eq!(s, s3);
            assert!(dbg_assert_eq_vec(&w.weight, &w3.weight));
            let res_nosimde2 = w.evaluatev3bb(&bban);
            let res_nosimde3 = w2.evaluatev3bb(&bban);
            let res_nosimde4 = w3.evaluatev3bb(&bban);
            // println!("{res_nosimde} -> {res_nosimde2}");
            assert!(dbg_assert_eq(&res_nosimde2, &res_nosimde3));
            assert!(dbg_assert_eq(&res_nosimde2, &res_nosimde4));
            let before = (winner as f32 - res_nosimde).abs();
            assert!(before > (winner as f32 - res_nosimde2).abs());
            // assert!(before > (winner as f32 - res_nosimde3).abs());
            // assert!(before > (winner as f32 - res_nosimde4).abs());
        }
    }
}

#[cfg(target_arch="aarch64")]
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
    let limit = 2e-6;
    for rfen in rfens.iter() {
        for winner in -1..=1 {
            let bban = bitboard::BitBoard::from(rfen).unwrap();
            let ban = board::Board::from(rfen).unwrap();
            ban.put();
            let mut w = weight::Weight::new();
            w.init();
            let mut w2 = weight::Weight::new();
            w2.copy(&w);
            let mut w3 = weight::Weight::new();
            w3.copy(&w);
            let res_nosimde = w.evaluatev3bb(&bban);
            let res_simd = w.evaluatev3bb_simd(&bban);
            // let res_simdavx = w.evaluatev3bb_simdavx(&bban);
            assert!(dbg_assert_eq(&res_nosimde, &res_simd));
            // assert!((res_nosimde - res_simd).abs() < limit);
            // assert!((res_nosimde - res_simdavx).abs() < limit);
            // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
            let (bh_ns, ah_ns, res_nosimd, fsns) = w.forwardv3bb(&bban);
            // let (bh_s, ah_s, res_simd, fss) = w.forwardv3bb_simd(&bban);
            // let (bh_sa, ah_sa, res_simdavx, fssa)
            //         = w.forwardv3bb_simdavx(&bban);
            // let (bh_sa2, ah_sa2, res_simdavx2, fssa2)
            //         = w.forwardv3bb_simdavx3(&bban);
            // assert!(dbg_assert_eq_vec(&bh_ns, &bh_s));
            // assert!(dbg_assert_eq_vec(&bh_ns, &bh_sa));
            // assert!(dbg_assert_eq_vec(&bh_ns, &bh_sa2));
            // println!("{bh_ns:?} == \n{bh_s:?} == \n{bh_sa:?} ???");
            // assert!(dbg_assert_eq_vec(&ah_ns, &ah_s));
            // assert!(dbg_assert_eq_vec(&ah_ns, &ah_sa));
            // assert!(dbg_assert_eq_vec(&ah_ns, &ah_sa2));
            // println!("{ah_ns:?} == \n{ah_s:?} == \n{ah_sa:?} ???");
            assert!((res_nosimde - res_nosimd[0]).abs() < limit);
            // assert_eq!(res_nosimd, res_simd);
            // assert!((res_nosimd[0] - res_simd[0]).abs() < limit);
            // assert_eq!(res_nosimd, res_simdavx);
            // assert!((res_nosimd[0] - res_simdavx[0]).abs() < limit);
            // assert_eq!(res_nosimd, res_simdavx2);
            // assert!((res_nosimd[0] - res_simdavx2[0]).abs() < limit);
            // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
            // assert_eq!(fsns, fss);
            // assert_eq!(fsns, fssa);
            // assert_eq!(fsns, fssa2);
            // println!("{fsns:?} == {fss:?} == {fssa:?} ???");
            let res = w.forwardv3bb(&bban);
            // let winner = 1;
            let eta = 0.1;
            w.backwardv3bb(&bban, winner, eta, &res);
            // w2.backwardv3bb_simd(&bban, winner, eta, &res);
            // let sv = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s = sv.join(",");
            // let sv2 = w2.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s2 = sv2.join(",");
            // assert_eq!(s, s2);
            // assert!(dbg_assert_eq_vec(&w.weight, &w2.weight));
            let res = w3.forwardv3(&ban);
            w3.backwardv3(&ban, winner, eta, &res);
            // let sv3 = w.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            // let s3 = sv3.join(",");
            // assert_eq!(s, s3);
            // assert!(dbg_assert_eq_vec(&w.weight, &w3.weight));
            let res_nosimde2 = w.evaluatev3bb(&bban);
            let res_nosimde3 = w2.evaluatev3bb(&bban);
            let res_nosimde4 = w3.evaluatev3bb(&bban);
            // println!("{res_nosimde} -> {res_nosimde2}");
            // assert!(dbg_assert_eq(&res_nosimde2, &res_nosimde3));
            // assert!(dbg_assert_eq(&res_nosimde2, &res_nosimde4));
            let before = (winner as f32 - res_nosimde).abs();
            // assert!(before > (winner as f32 - res_nosimde2).abs());
            // assert!(before > (winner as f32 - res_nosimde3).abs());
            // assert!(before > (winner as f32 - res_nosimde4).abs());
        }
    }
}
