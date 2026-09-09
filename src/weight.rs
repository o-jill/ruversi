use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};
use aligned_vec::AVec;

#[cfg(target_arch="x86_64")]
use std::arch::x86_64;

#[cfg(target_arch="aarch64")]
use std::arch::aarch64::*;


/*
 * input: NUMCELL * NUMCELL + 1(teban) + 2(fixedstones) + 1
 * hidden: 8 + 1
 * output: 1
 */
pub const N_INPUT_BLACK : usize = bitboard::CELL_2D;
pub const N_INPUT_WHITE : usize = N_INPUT_BLACK + bitboard::CELL_2D;
pub const N_INPUT : usize = N_INPUT_WHITE;

const N_HIDDEN : usize = 128;
pub const N_HIDDEN2 : usize = 16;
const N_OUTPUT : usize = 1;

#[allow(dead_code)]
const N_WEIGHT_INPUT : usize = 0;
const N_WEIGHT_INPUTBIAS : usize = N_INPUT * N_HIDDEN;
const N_WEIGHT_LAYER1 : usize = N_WEIGHT_INPUTBIAS + N_HIDDEN;
const N_WEIGHT_LAYER1BIAS : usize = N_WEIGHT_LAYER1 + N_HIDDEN * N_HIDDEN2;
const N_WEIGHT_LAYER2 : usize = N_WEIGHT_LAYER1BIAS + N_HIDDEN2;
const N_WEIGHT_LAYER2BIAS : usize = N_WEIGHT_LAYER2 + N_HIDDEN2;
const N_WEIGHT : usize =
  (N_INPUT + 1) * N_HIDDEN + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

const N_WEIGHT_PAD :usize = N_WEIGHT.div_ceil(8) * 8;
pub const N_PROGRESS_DIV : usize = 6;  // 序盤中盤終盤x手番

#[allow(dead_code)]
const WSZV1 : usize = (bitboard::CELL_2D + 1 + 1) * 4 + 4 + 1;
#[allow(dead_code)]
const WSZV2 : usize = WSZV1;
#[allow(dead_code)]
const WSZV3 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 4 + 4 + 1;
#[allow(dead_code)]
const WSZV4 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 8 + 8 + 1;
#[allow(dead_code)]
const WSZV5 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 16 + 16 + 1;
#[allow(dead_code)]
const WSZV6 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN + N_HIDDEN + 1;
#[allow(dead_code)]
const WSZV7 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 32
        + (32 + 1) * 16 + 16 + 1;
#[allow(dead_code)]
const WSZV8 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;
#[allow(dead_code)]
const WSZV9 : usize = WSZV8;
#[allow(dead_code)]
const WSZV10 : usize = (bitboard::CELL_2D * 2 + 1 + 2 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;
#[allow(dead_code)]
const WSZV11 : usize = (bitboard::CELL_2D * 2 + 1 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;
#[allow(dead_code)]
const WSZV12 : usize = (bitboard::CELL_2D * 2 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

// v2
// 8/8/1A6/2Ab3/2C3/8/8/8 w
// val:-273.121 val:Some(-273.1215), 268965 nodes. []b6@@b5[]c6@@a7[]a5@@a6[]a8 60msec
// 8/8/1A6/2Ab3/2aB3/1a6/8/8 b
// val:-3.506 val:Some(-3.5055861), 334278 nodes. @@c3[]c2@@d1[]c1@@b1[]a4@@a2 80msec

// const CONVERT_TO_FIXPOINT_I16 : f32 = 128.0;
// const CONVERT_TO_FIXPOINT_I16_SHIFT : i32 = 7;
const CONVERT_TO_FIXPOINT_I16 : f32 = 256.0;
const CONVERT_TO_FIXPOINT_I16_SHIFT : i32 = 8;
// const CONVERT_TO_FIXPOINT_I16 : f32 = 256.0 * 2.0;
// const CONVERT_TO_FIXPOINT_I16_SHIFT : i32 = 8 + 1;

#[derive(PartialEq)]
enum EvalFile{
    Unknown,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
}

impl std::fmt::Display for EvalFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
            EvalFile::Unknown => {"unknown eval file format."},
            EvalFile::V1 => {"# 65-4-1"},
            EvalFile::V2 => {"# 64+1-4-1"},
            EvalFile::V3 => {"# 64+1+2-4-1"},
            EvalFile::V4 => {"# 64+1+2-8-1"},
            EvalFile::V5 => {"# 64+1+2-16-1"},
            EvalFile::V6 => {"# 64+1+2-32-1"},
            EvalFile::V7 => {"# 64+1+2-32-16-1"},
            EvalFile::V8 => {"# 64+1+2-128-16-1"},
            EvalFile::V9 => {"# 3x 64+1+2-128-16-1"},
            EvalFile::V10 => {"# 3x 128+1+2-128-16-1"},
            EvalFile::V11 => {"# 3x 128+1-128-16-1"},
            EvalFile::V12 => {"# 6x 128-128-16-1"},
            }
        )
    }
}

impl EvalFile {
    pub fn from(txt : &str) -> Option<EvalFile> {
        match txt {
            "# 65-4-1" => Some(EvalFile::V1),
            "# 64+1-4-1" => Some(EvalFile::V2),
            "# 64+1+2-4-1" => Some(EvalFile::V3),
            "# 64+1+2-8-1" => Some(EvalFile::V4),
            "# 64+1+2-16-1" => Some(EvalFile::V5),
            "# 64+1+2-32-1" => Some(EvalFile::V6),
            "# 64+1+2-32-16-1" => Some(EvalFile::V7),
            "# 64+1+2-128-16-1" => Some(EvalFile::V8),
            "# 3x 64+1+2-128-16-1" => Some(EvalFile::V9),
            "# 3x 128+1+2-128-16-1" => Some(EvalFile::V10),
            "# 3x 128+1-128-16-1" => Some(EvalFile::V11),
            "# 6x 128-128-16-1" => Some(EvalFile::V12),
            _ => {
                None
            }
        }
    }
}

const MEM_ALIGN : usize = 64;

pub struct Weight {
    // 128xH1 + H1 + H1x2 + H1 + H1 x (H2+1) + H2 + 1
    pub weight : AVec<f32>,
    // H1x128 + H1 + H1x2 + H1 + H1 x (H2+1) + H2 + 1
    vweight : AVec<f32>,
    iweight : AVec<i16>,
    ivweight : AVec<i16>,
    i32vweight : AVec<i32>,
    iweightdc : AVec<i32>,
}

impl Default for Weight {
    fn default() -> Self {
        let mut w = Self::new();
        w.exchange();
        w
    }
}

impl Weight {
    pub fn new() -> Weight {
        Weight {
            weight: {
                let mut w = AVec::with_capacity(
                        MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                w.resize(w.capacity(), 0f32);
                w
            },
            vweight: {
                let mut w = AVec::with_capacity(
                    MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                w.resize(w.capacity(), 0f32);
                w
            },
            iweight: {
                let mut w = AVec::with_capacity(
                    MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                w.resize(w.capacity(), 0i16);
                w
            },
            ivweight: {
                let mut w = AVec::with_capacity(
                    MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                w.resize(w.capacity(), 0i16);
                w
            },
            i32vweight: {
                let mut w = AVec::with_capacity(
                    MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                w.resize(w.capacity(), 0i32);
                w
            },
            iweightdc: {
                let mut w = AVec::with_capacity(
                    MEM_ALIGN, (N_HIDDEN + N_HIDDEN2) * N_PROGRESS_DIV);
                w.resize(w.capacity(), 0i32);
                w
            },
        }
    }

    fn exchange(&mut self) {
        for p in 0..N_PROGRESS_DIV {
            let mut check = [0i8 ; N_WEIGHT_INPUTBIAS];
            let offset = p * N_WEIGHT_PAD;
            let wei = &self.weight[offset..offset + N_WEIGHT_PAD];

            let vwei = &mut self.vweight[offset..offset + N_WEIGHT_PAD];
            vwei.copy_from_slice(wei);
            for (i, &w) in wei.iter().enumerate().take(N_INPUT * N_HIDDEN) {
                let hidx = i / N_INPUT;

                let x = i % N_INPUT;  // b: 0~63, w:64~127
                let bw = x / bitboard::CELL_2D;  // 0:b, 1:w
                let x = x % bitboard::CELL_2D;

                let idx = hidx + x * N_HIDDEN * 2 + bw * N_HIDDEN;
                vwei[idx] = w;
                // if idx >= check.len() {
                //     panic!("hidx:{hidx}, bw:{bw}, x:{x}, i:{i}");
                // }
                check[idx] = 1;
            }
            for (i, &c) in check.iter().enumerate() {
                if c == 0 {
                    panic!("check error @ {i}!");
                }
            }
            let wei = &self.weight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1 + N_HIDDEN * N_HIDDEN2];
            let vwei = &mut self.vweight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1 + N_HIDDEN * N_HIDDEN2];
            // let mut check = [0i8 ; N_HIDDEN * N_HIDDEN2];
            for (i, &w) in wei.iter().enumerate().take(N_HIDDEN2 * N_HIDDEN) {
                let ii = i;
                let hidx2 = ii / N_HIDDEN;
                let hidx1 = ii % N_HIDDEN;
                let idx = hidx1 * N_HIDDEN2 + hidx2;
                vwei[idx] = w;
                // check[idx] = 1;
            }
            // for (i, &c) in check.iter().enumerate() {
            //     if c == 0 {
            //         panic!("check error2 @ {i}! {}", check2[i]);
            //     }
            // }
        }
        self.quantize();
    }

    fn quantize(&mut self) {
        for (w, i) in self.weight.iter().zip(self.iweight.iter_mut()){
            *i = (*w * CONVERT_TO_FIXPOINT_I16 + 0.5) as i16;
            // *i = (*v * CONVERT_TO_FIXPOINT_I32 + 0.5) as i32;
        }
        for (v, (i, i_32)) in self.vweight.iter().zip(self.ivweight.iter_mut().zip(self.i32vweight.iter_mut())){
            *i = (*v * CONVERT_TO_FIXPOINT_I16 + 0.5) as i16;
            *i_32 = *i as i32;
            // *i = (*v * CONVERT_TO_FIXPOINT_I32 + 0.5) as i32;
        }
        // dc
        for progress in 0..N_PROGRESS_DIV {
            for i in 0..N_HIDDEN {
                let idx = progress * (N_HIDDEN + N_HIDDEN2);
                let wdc = self.wibias(progress)[i];
                self.iweightdc[idx + i] =
                    (wdc * CONVERT_TO_FIXPOINT_I16 + 0.5) as i32;
            }
            for i in 0..N_HIDDEN2 {
                let idx = progress * (N_HIDDEN + N_HIDDEN2) + N_HIDDEN;
                let wdc = self.wl1bias(progress)[i];
                self.iweightdc[idx + i] =
                    (wdc * CONVERT_TO_FIXPOINT_I16 * CONVERT_TO_FIXPOINT_I16 + 0.5) as i32
                    + (CONVERT_TO_FIXPOINT_I16 * 0.5) as i32;
            }
        }
    }

    pub fn init(&mut self) {
        let mut rng = rand::thread_rng();
        let range =
            f64::sqrt(6.0) /
                f64::sqrt((N_INPUT + N_HIDDEN + N_HIDDEN2 + N_OUTPUT) as f64);

        for a in self.weight.iter_mut() {
            *a = (rng.gen::<f64>() * 2.0 * range - range) as f32;
        }
        self.exchange();
    }

    /// fill zero.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.weight.iter_mut().for_each(|m| *m = 0.0);
    }

    #[allow(dead_code)]
    pub fn wban(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset..]
    }

    pub fn wbanv(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.vweight[offset..]
    }

    pub fn wbani(&self, progress : usize) -> &[i16] {
    // pub fn wbani(&self, progress : usize) -> &[i32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.ivweight[offset..]
    }

    pub fn wbani32(&self, progress : usize) -> &[i32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.i32vweight[offset..]
    }

    pub fn wibias(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_INPUTBIAS..offset + N_WEIGHT_LAYER1]
    }

    pub fn wibiasi(&self, progress : usize) -> &[i16] {
    // pub fn wibiasi(&self, progress : usize) -> &[i32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.ivweight[offset + N_WEIGHT_INPUTBIAS..offset + N_WEIGHT_LAYER1]
    }

    pub fn wibiasi32(&self, progress : usize) -> &[i32] {
        let offset = progress * (N_HIDDEN + N_HIDDEN2);
        &self.iweightdc[offset..offset + N_HIDDEN]
    }

    pub fn wlayer1(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1BIAS]
    }

    pub fn wlayer1v(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.vweight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1BIAS]
    }

    pub fn wlayer1i(&self, progress : usize) -> &[i16] {
        let offset = progress * N_WEIGHT_PAD;
        &self.iweight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1BIAS]
    }

    pub fn wlayer1iv(&self, progress : usize) -> &[i16] {
        let offset = progress * N_WEIGHT_PAD;
        &self.ivweight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1BIAS]
    }

    pub fn wl1bias(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER1BIAS..offset + N_WEIGHT_LAYER2]
    }

    pub fn wl1biasi(&self, progress : usize) -> &[i16] {
        let offset = progress * N_WEIGHT_PAD;
        &self.iweight[offset + N_WEIGHT_LAYER1BIAS..offset + N_WEIGHT_LAYER2]
    }

    pub fn wl1biasi32(&self, progress : usize) -> &[i32] {
        let offset = progress * (N_HIDDEN + N_HIDDEN2) + N_HIDDEN;
        &self.iweightdc[offset..offset + N_HIDDEN2]
    }

    pub fn wlayer2(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER2..offset + N_WEIGHT_LAYER2BIAS]
    }

    pub fn wl2bias(&self, progress : usize) -> f32 {
        let offset = progress * N_WEIGHT_PAD;
        self.weight[offset + N_WEIGHT - 1]
    }

    pub fn wlayer2i(&self, progress : usize) -> &[i16] {
        let offset = progress * N_WEIGHT_PAD;
        &self.ivweight[offset + N_WEIGHT_LAYER2..offset + N_WEIGHT_LAYER2BIAS]
    }

    pub fn wl2biasi(&self, progress : usize) -> i16 {
        let offset = progress * N_WEIGHT_PAD;
        self.iweight[offset + N_WEIGHT - 1]
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
        if let Err(e) = file {
            eprintln!("path: {path}");
            return Err(e.to_string());
        }

        let mut idx = 0;
        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if l.starts_with("#") {
                        if format != EvalFile::Unknown {
                            // panic!("EvalFile::Unknown: {l}");
                            continue;
                        }

                        if let Some(fmt) = EvalFile::from(&l) {
                            format = fmt;
                            // eprintln!("format:{format}");
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
                        EvalFile::V7 => {return self.readv7(&l)},
                        EvalFile::V8 => {return self.readv8(&l)},
                        EvalFile::V9 => {
                            self.readv9(&l, idx)?;
                            idx += 1;
                            if idx >= 3 {
                                self.exchange();
                                return Ok(());
                            }
                        },
                        EvalFile::V10 => {
                            self.readv10(&l, idx)?;
                            idx += 1;
                            if idx >= 3 {
                                self.exchange();
                                return Ok(());
                            }
                        },
                        EvalFile::V11 => {
                            self.readv11(&l, idx)?;
                            idx += 1;
                            if idx >= 3 {
                                self.exchange();
                                return Ok(());
                            }
                        },
                        EvalFile::V12 => {
                            self.readv12(&l, idx)?;
                            idx += 1;
                            if idx >= N_PROGRESS_DIV {
                                self.exchange();
                                return Ok(());
                            }
                        },
                        _ => {
                            panic!("EvalFile::Unknown...");
                        }
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

    fn readv3(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v3 format is not supported any more."))
    }

    fn readv4(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v4 format is not supported any more."))
    }

    fn readv5(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v5 format is not supported any more."))
    }

    fn readv6(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v6 format is not supported any more."))
    }

    fn readv7(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v7 format is not supported any more."))
    }

    fn readv8(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v8 format is not supported any more."))
    }

    fn readv9(&mut self, _line : &str, _progress : usize) -> Result<(), String> {
        Err(String::from("v9 format is not supported any more."))
    }

    fn readv10(&mut self, line : &str, progress : usize) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> =
                csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV10 != nsz {
            return Err(format!("size mismatch v10:{WSZV10} != {nsz}"));
        }

        let offset = progress * N_WEIGHT_PAD;
        // stones + teban
        self.weight[offset..offset + N_WEIGHT_INPUTBIAS].copy_from_slice(
            &newtable[..N_WEIGHT_INPUTBIAS]);
        // input bias + the others
        self.weight[offset + N_WEIGHT_INPUTBIAS..offset + N_WEIGHT].copy_from_slice(
            &newtable[(bitboard::CELL_2D * 2 + 3) * N_HIDDEN..]);
        // println!("v9:{:?}", self.weight);
        Ok(())
    }

    fn readv11(&mut self, line : &str, progress : usize) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> =
                csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV11 != nsz {
            return Err(format!("size mismatch v11:{WSZV11} != {nsz}"));
        }

        let offset = progress * N_WEIGHT_PAD * 2;
        self.weight[offset..offset + N_WEIGHT_INPUTBIAS].copy_from_slice(
            &newtable[..N_WEIGHT_INPUTBIAS]);
        self.weight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT]
                .copy_from_slice(&newtable[N_WEIGHT_INPUTBIAS + N_HIDDEN * 2..]);
        // self.weight[offset + N_WEIGHT_PAD..offset + N_WEIGHT + N_WEIGHT_PAD]
        //         .copy_from_slice(&newtable);

        // self.weight[offset + N_WEIGHT_PAD..offset + N_WEIGHT_PAD * 2]
        //     .copy_from_slice(&self.weight[offset..offset + N_WEIGHT_PAD]);
        let (dest, src) = self.weight.split_at_mut(offset + N_WEIGHT_PAD);
        src[..N_WEIGHT].copy_from_slice(&dest[offset..offset + N_WEIGHT]);
        // println!("v11:{:?}", self.weight);
        Ok(())
    }

    fn readv12(&mut self, line : &str, progress : usize) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> =
                csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV12 != nsz {
            return Err(format!("size mismatch v12:{WSZV12} != {nsz}"));
        }

        let offset = progress * N_WEIGHT_PAD;
        self.weight[offset..offset + N_WEIGHT].copy_from_slice(&newtable);
        // println!("v12:{:?}", self.weight);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn writev12(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(
            format!("{}\n", EvalFile::V12).as_bytes()).unwrap();
        for prgs in 0..N_PROGRESS_DIV {
            let offset = prgs * N_WEIGHT_PAD;
            let w = &self.weight[offset..offset + N_WEIGHT];
            let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            f.write_all((sv.join(",") + "\n").as_bytes()).unwrap();
        }
    }

    pub fn copy(&mut self, src : &Weight) {
        self.weight.copy_from_slice(&src.weight);
        self.vweight.copy_from_slice(&src.vweight);
    }

    pub fn evaluatev12bb(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();

        let ow = self.wbanv(prgs);

        let wdc = self.wibias(prgs);
        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        let black = ban.black;
        let white = ban.white;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let start = idx * N_HIDDEN * 2 + if b != 0 {0} else {N_HIDDEN};
            for (h, w) in hid.iter_mut().zip(ow.iter().skip(start)) {
                *h += w;
            }
        }
        // relu
        for h in hid.iter_mut() {
                *h = h.max(0f32);
        }

        let mut sum = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let whdc = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i];
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 = h1.mul_add(wh[j + i * N_HIDDEN], hidsum2);
                // hidsum2 += h1 * wh[j + i * N_HIDDEN];
            }
            // relu
            sum += hidsum2.max(0f32) * wh2[i];
        }
        sum
    }

    pub fn evaluatev12bb_i16(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();

        let ow = self.wbani(prgs);

        let wdc = self.wibiasi32(prgs);
        let mut hid = [0i32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        let black = ban.black;
        let white = ban.white;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let start = idx * N_HIDDEN * 2 + if b != 0 {0} else {N_HIDDEN};
            for (h, w) in hid.iter_mut().zip(ow.iter().skip(start)) {
                *h += *w as i32;
            }
        }
        // relu
        for h in hid.iter_mut() {
            *h = 0.max(*h);
        }

        let wh = self.wlayer1iv(prgs);
        let whdc = self.wl1biasi32(prgs);
        let wh2 = self.wlayer2i(prgs);
        let mut res = self.wl2biasi(prgs) as i32
            * CONVERT_TO_FIXPOINT_I16 as i32;
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i];
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 += *h1 as i32 * wh[i + j * N_HIDDEN2] as i32;
            }
            // relu
            let sum = (hidsum2.max(0i32) >> CONVERT_TO_FIXPOINT_I16_SHIFT) as i16;
            // layer2
            res += wh2[i] as i32 * sum as i32;
        }

        // output
        res as f32 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
    }

    pub fn evaluatev12bb_i16_2(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();

        let ow = self.wbani(prgs);

        let wdc = self.wibiasi32(prgs);
        let mut hid = [0i32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        let black = ban.black;
        let white = ban.white;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let start = idx * N_HIDDEN * 2 + if b != 0 {0} else {N_HIDDEN};
            for (h, w) in hid.iter_mut().zip(ow.iter().skip(start)) {
                *h += *w as i32;
            }
        }
        // relu
        for h in hid.iter_mut() {
                *h = 0.max(*h);
        }

        let wh = self.wlayer1iv(prgs);
        let whdc = self.wl1biasi32(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut sum = [0f32 ; N_HIDDEN2];
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i] + (CONVERT_TO_FIXPOINT_I16 * 0.5) as i32;
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 += *h1 as i32 * wh[i + j * N_HIDDEN2] as i32;
            }
            // relu + f32
            sum[i] = (hidsum2.max(0i32) / 256) as f32
                / CONVERT_TO_FIXPOINT_I16;
        }

        let mut res = self.wl2bias(prgs);
        for (s, w) in sum.iter().zip(wh2.iter()) {
            res += *s * *w;
        }

        res
    }

    pub fn evaluatev12bb_i16_1(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();

        let ow = self.wbani(prgs);

        let wdc = self.wibiasi32(prgs);
        let mut hid = [0i32 ; N_HIDDEN];
        let black = ban.black;
        let white = ban.white;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let start = idx * N_HIDDEN * 2 + if b != 0 {0} else {N_HIDDEN};
            for (h, w) in hid.iter_mut().zip(ow.iter().skip(start)) {
                *h += *w as i32;
            }
        }
        // relu
        for (h, dc) in hid.iter_mut().zip(wdc.iter()) {
                *h = 0i32.max(*h + *dc);
        }

        let mut sum = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let whdc = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i];
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 = (*h1 as f32 / CONVERT_TO_FIXPOINT_I16).mul_add(wh[j + i * N_HIDDEN], hidsum2);
                // hidsum2 = (*h1 as f32 / CONVERT_TO_FIXPOINT).mul_add(wh[j + i * N_HIDDEN], hidsum2);
                // hidsum2 += h1 * wh[j + i * N_HIDDEN];
            }
            // relu
            sum += hidsum2.max(0f32) * wh2[i];
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbanv(prgs);

        let wdc = self.wibias(prgs);

        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        const N : usize = 16;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_ps(wei.as_ptr().add(i));
                    let w2 = x86_64::_mm_load_ps(wei.as_ptr().add(i + 4));
                    let w3 = x86_64::_mm_load_ps(wei.as_ptr().add(i + 8));
                    let w4 = x86_64::_mm_load_ps(wei.as_ptr().add(i + 12));
                    let h1 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i));
                    let h2 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i + 4));
                    let h3 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i + 8));
                    let h4 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i + 12));
                    let m1 = x86_64::_mm_add_ps(w1, h1);
                    let m2 = x86_64::_mm_add_ps(w2, h2);
                    let m3 = x86_64::_mm_add_ps(w3, h3);
                    let m4 = x86_64::_mm_add_ps(w4, h4);
                    x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i), m1);
                    x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 4), m2);
                    x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 8), m3);
                    x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 12), m4);
                }
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        let mut sum4 = [0f32 ; N_HIDDEN2 * 4];
        for j in (0..N_HIDDEN).step_by(16) {
            unsafe {
                let x1 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j));
                let x2 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 4));
                let x3 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 8));
                let x4 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 12));

                // relu
                let zero = x86_64::_mm_setzero_ps();
                let x1 = x86_64::_mm_max_ps(x1, zero);
                let x2 = x86_64::_mm_max_ps(x2, zero);
                let x3 = x86_64::_mm_max_ps(x3, zero);
                let x4 = x86_64::_mm_max_ps(x4, zero);

                for i in 0..N_HIDDEN2 {
                    let idx = i * N_HIDDEN + j;
                    let w1 = x86_64::_mm_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 4));
                    let w3 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 8));
                    let w4 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 12));
                    let mul1 = x86_64::_mm_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm_mul_ps(x2, w2);
                    // let mul3 = x86_64::_mm_mul_ps(x3, w3);
                    // let mul4 = x86_64::_mm_mul_ps(x4, w4);
                    // let s12 = x86_64::_mm_add_ps(mul1, mul2);
                    // let s34 = x86_64::_mm_add_ps(mul3, mul4);
                    let s12 = x86_64::_mm_fmadd_ps(x3, w3, mul1);
                    let s34 = x86_64::_mm_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm_add_ps(s12, s34);
                    let s4 = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4));
                    let s4 = x86_64::_mm_add_ps(s1234, s4);
                    x86_64::_mm_storeu_ps(sum4.as_mut_ptr().add(i * 4), s4);
                }
            }
        }
        for i in (0..N_HIDDEN2).step_by(4) {
            unsafe {
                let a = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4));
                let b = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 4));
                let c = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 8));
                let d = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 12));
                let a0c0a1c1 = x86_64::_mm_unpacklo_ps(a, c);
                let b0d0b1d1 = x86_64::_mm_unpacklo_ps(b, d);
                let a2c2a3c3 = x86_64::_mm_unpackhi_ps(a, c);
                let b2d2b3d3 = x86_64::_mm_unpackhi_ps(b, d);
                let a0 = x86_64::_mm_unpacklo_ps(a0c0a1c1, b0d0b1d1);
                let a1 = x86_64::_mm_unpackhi_ps(a0c0a1c1, b0d0b1d1);
                let a2 = x86_64::_mm_unpacklo_ps(a2c2a3c3, b2d2b3d3);
                let a3 = x86_64::_mm_unpackhi_ps(a2c2a3c3, b2d2b3d3);
                let s1 = x86_64::_mm_add_ps(a0, a1);
                let s2 = x86_64::_mm_add_ps(a2, a3);
                let s3 = x86_64::_mm_add_ps(s1, s2);

                let dc = x86_64::_mm_load_ps(wdc1.as_ptr().add(i));
                let s4 = x86_64::_mm_add_ps(s3, dc);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i), s4);
            }
        }
        for j in 0..N_HIDDEN2 / 16 {
            unsafe {  // relu
                let h1 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16));
                let h2 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16 + 4));
                let h3 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16 + 8));
                let h4 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16 + 12));
                let zero = x86_64::_mm_setzero_ps();
                let h1 = x86_64::_mm_max_ps(h1, zero);
                let h2 = x86_64::_mm_max_ps(h2, zero);
                let h3 = x86_64::_mm_max_ps(h3, zero);
                let h4 = x86_64::_mm_max_ps(h4, zero);
                let wh21 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16));
                let wh22 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16 + 4));
                let wh23 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16 + 8));
                let wh24 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16 + 12));

                let y1 = x86_64::_mm_mul_ps(wh21, h1);
                let y2 = x86_64::_mm_mul_ps(wh22, h2);
                let y3 = x86_64::_mm_mul_ps(wh23, h3);
                let y4 = x86_64::_mm_mul_ps(wh24, h4);
                let y12 = x86_64::_mm_add_ps(y1, y2);
                let y34 = x86_64::_mm_add_ps(y3, y4);
                let y1234 = x86_64::_mm_add_ps(y12, y34);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(j * 4), y1234);
            }
        }
        for h in hid2.iter().take(N_HIDDEN2 / 4) {
            res += h;
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simd_i16(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani32(prgs);

        let wdc = self.wibiasi32(prgs);

        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        const N : usize = 16;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            // let wei = if b != 0 {
            //     &ow[idx * N_HIDDEN * 2 .. ]
            // } else {
            //     &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            // };
            let wei =
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN * (b == 0) as usize .. ];
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i) as *const x86_64::__m128i);
                    let w2 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let w3 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let w4 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i + 12) as *const x86_64::__m128i);

                    let h1 = x86_64::_mm_loadu_si128(
                        hidi.as_ptr().add(i) as *const x86_64::__m128i);
                    let h2 = x86_64::_mm_loadu_si128(
                        hidi.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let h3 = x86_64::_mm_loadu_si128(
                        hidi.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let h4 = x86_64::_mm_loadu_si128(
                        hidi.as_ptr().add(i + 12) as *const x86_64::__m128i);

                    let m1 = x86_64::_mm_add_epi32(w1, h1);
                    let m2 = x86_64::_mm_add_epi32(w2, h2);
                    let m3 = x86_64::_mm_add_epi32(w3, h3);
                    let m4 = x86_64::_mm_add_epi32(w4, h4);

                    x86_64::_mm_storeu_si128(
                        hidi.as_mut_ptr().add(i) as *mut x86_64::__m128i, m1);
                    x86_64::_mm_storeu_si128(
                        hidi.as_mut_ptr().add(i + 4) as *mut x86_64::__m128i,
                        m2);
                    x86_64::_mm_storeu_si128(
                        hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i,
                        m3);
                    x86_64::_mm_storeu_si128(
                        hidi.as_mut_ptr().add(i + 12) as *mut x86_64::__m128i,
                        m4);
                }
            }
        }

        // for h in hidi {
        //     if h > 0x7fff {
        //         panic!("h:{h} > 0x7fff");
        //     }
        // }
        let mut hid = [0i16 ; N_HIDDEN];
        for i in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm_loadu_si128(
                    hidi.as_ptr().add(i) as *const x86_64::__m128i);
                let h2 = x86_64::_mm_loadu_si128(
                    hidi.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let h3 = x86_64::_mm_loadu_si128(
                    hidi.as_ptr().add(i + 8) as *const x86_64::__m128i);
                let h4 = x86_64::_mm_loadu_si128(
                    hidi.as_ptr().add(i + 12) as *const x86_64::__m128i);

                // i32 -> i16
                let y12 = x86_64::_mm_packs_epi32(h1, h2);
                let y34 = x86_64::_mm_packs_epi32(h3, h4);

                // relu
                let zero = x86_64::_mm_setzero_si128();
                let y12 = x86_64::_mm_max_epi16(y12, zero);
                let y34 = x86_64::_mm_max_epi16(y34, zero);

                x86_64::_mm_storeu_si128(
                    hid.as_mut_ptr().add(i) as *mut x86_64::__m128i, y12);
                x86_64::_mm_storeu_si128(
                    hid.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i, y34);
            }
        }

        // 2nd layer to output
        let wh = self.wlayer1iv(prgs);
        let mut sum4 = [0i32 ; N_HIDDEN2];
        let dc1 = self.wl1biasi32(prgs);
        sum4.copy_from_slice(dc1);

        for j in 0..N_HIDDEN {
            if hid[j] == 0 {continue;}

            unsafe {
                let x1 = x86_64::_mm_set1_epi16(hid[j]);
                for i in (0..N_HIDDEN2).step_by(16) {
                    let idx = i + N_HIDDEN2 * j;
                    let w1 = x86_64::_mm_load_si128(
                        wh.as_ptr().add(idx) as *const x86_64::__m128i);
                    // let w2 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 4));
                    let w3 = x86_64::_mm_load_si128(
                        wh.as_ptr().add(idx + 8) as *const x86_64::__m128i);
                    // let w4 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 12));
                    let mul1l = x86_64::_mm_mullo_epi16(x1, w1);
                    let mul3l = x86_64::_mm_mullo_epi16(x1, w3);
                    let mul1h = x86_64::_mm_mulhi_epi16(x1, w1);
                    let mul3h = x86_64::_mm_mulhi_epi16(x1, w3);
                    let mul1 = x86_64::_mm_unpacklo_epi16(mul1l, mul1h);
                    let mul3 = x86_64::_mm_unpacklo_epi16(mul3l, mul3h);
                    let mul2 = x86_64::_mm_unpackhi_epi16(mul1l, mul1h);
                    let mul4 = x86_64::_mm_unpackhi_epi16(mul3l, mul3h);

                    let s1 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i) as *const x86_64::__m128i);
                    let s2 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let s3 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let s4 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i + 12) as *const x86_64::__m128i);

                    let s1 = x86_64::_mm_add_epi32(mul1, s1);
                    let s2 = x86_64::_mm_add_epi32(mul2, s2);
                    let s3 = x86_64::_mm_add_epi32(mul3, s3);
                    let s4 = x86_64::_mm_add_epi32(mul4, s4);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i) as *mut x86_64::__m128i,
                        s1);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i + 4) as *mut x86_64::__m128i,
                        s2);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i,
                        s3);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i + 12) as *mut x86_64::__m128i,
                        s4);
                }
            }
        }

        let wh2 = self.wlayer2i(prgs);
        let mut hid2 = [0i32 ; N_HIDDEN2];
        for i in (0..N_HIDDEN2).step_by(16) {
            unsafe {
                let a = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i) as *const x86_64::__m128i);
                let b = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let c = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i + 8) as *const x86_64::__m128i);
                let d = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i + 12) as *const x86_64::__m128i);

                let s1 = x86_64::_mm_srai_epi32(
                        a, CONVERT_TO_FIXPOINT_I16_SHIFT);
                let s2 = x86_64::_mm_srai_epi32(
                        b, CONVERT_TO_FIXPOINT_I16_SHIFT);
                let s3 = x86_64::_mm_srai_epi32(
                        c, CONVERT_TO_FIXPOINT_I16_SHIFT);
                let s4 = x86_64::_mm_srai_epi32(
                        d, CONVERT_TO_FIXPOINT_I16_SHIFT);

                // i32 -> i16
                let y12 = x86_64::_mm_packs_epi32(s1, s2);
                let y34 = x86_64::_mm_packs_epi32(s3, s4);

                // relu
                let zero = x86_64::_mm_setzero_si128();
                let y12 = x86_64::_mm_max_epi16(y12, zero);
                let y34 = x86_64::_mm_max_epi16(y34, zero);

                let w1 = x86_64::_mm_load_si128(
                    wh2.as_ptr().add(i) as *const x86_64::__m128i);
                // let w2 = x86_64::_mm_load_si128(
                //     wh2.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let w3 = x86_64::_mm_load_si128(
                    wh2.as_ptr().add(i + 8) as *const x86_64::__m128i);
                // let w4 = x86_64::_mm_load_si128(
                //     wh2.as_ptr().add(i + 12) as *const x86_64::__m128i);

                let mul1 = x86_64::_mm_madd_epi16(y12, w1);
                let mul3 = x86_64::_mm_madd_epi16(y34, w3);
                let y1234 = x86_64::_mm_add_epi32(mul1, mul3);

                x86_64::_mm_storeu_si128(
                    hid2.as_mut_ptr().add(i / 4) as *mut x86_64::__m128i,
                    y1234);
            }
        }
        let mut res = self.wl2biasi(prgs) as i32
            * CONVERT_TO_FIXPOINT_I16 as i32;
        for h in hid2.iter().take(N_HIDDEN2 / 4) {
            res += h;
        }
        res as f32 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
    }

    // #[cfg(target_arch="aarch64")]
    // pub fn evaluatev12bb_simd_mul_i16(&self, ban : &bitboard::BitBoard) -> f32 {
    //     let prgs = ban.progress();
    //     let black = ban.black;
    //     let white = ban.white;

    //     let ow = self.wbani(prgs);
    //     let wdc = self.wibiasi32(prgs);
    //     const N : usize = 16;
    //     let mut hid = [0i32 ; N_HIDDEN];
    //     let mut bit = bitboard::LSB_CELL;
    //     // cells
    //     for idx in 0..bitboard::CELL_2D {
    //         let b = black & bit;
    //         let w = white & bit;
    //         bit <<= 1;
    //         if b | w == 0 {continue;}  // no stone

    //         let wei = if b != 0 {
    //             &ow[idx * N_HIDDEN * 2 .. ]
    //         } else {
    //             &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
    //         };
    //         for i in (0..N_HIDDEN).step_by(N) {
    //             unsafe {
    //                 let w = vld1_i16_x4(wei.as_ptr().add(i));
    //                 let h = vld1q_i32_x4(hid.as_ptr().add(i));
    //                 // i16 -> i32
    //                 let w0 = vmovl_si16(w.0);
    //                 let w1 = vmovl_si16(w.1);
    //                 let w2 = vmovl_si16(w.2);
    //                 let w3 = vmovl_si16(w.3);
    //                 let w1 = vaddq_s32(h.0, w0);
    //                 let w2 = vaddq_s32(h.1, w1);
    //                 let w3 = vaddq_s32(h.2, w2);
    //                 let w4 = vaddq_s32(h.3, w3);
    //                 vst1q_s32_x4(hid.as_mut_ptr().add(i),
    //                     float32x4x4_t(w1, w2, w3, w4));
    //             }
    //         }
    //     }
    //     for i in (0..N_HIDDEN).step_by(N) {
    //         unsafe {
    //             let sum4 = vld1q_s32_x4(hid.as_ptr().add(i));

    //             let wdc4 = vld1q_s32_x4(wdc.as_ptr().add(i));
    //             let sum41 = vaddq_s32(sum4.0, wdc4.0);
    //             let sum42 = vaddq_s32(sum4.1, wdc4.1);
    //             let sum43 = vaddq_s32(sum4.2, wdc4.2);
    //             let sum44 = vaddq_s32(sum4.3, wdc4.3);

    //             // i32 -> i16
    //             let s1 = vqmovn_s32(sum41);
    //             let s2 = vqmovn_s32(sum42);
    //             let s3 = vqmovn_s32(sum43);
    //             let s4 = vqmovn_s32(sum44);

    //             // relu
    //             let zero = vmov_n_s16(0);
    //             let rl1 = vmax_s16(zero, s1);
    //             let rl2 = vmax_s16(zero, s2);
    //             let rl3 = vmax_s16(zero, s3);
    //             let rl4 = vmax_s16(zero, s4);

    //             vst1_s32_x4(hid.as_mut_ptr().add(i), int16x4x4_t(rl1, rl2, rl3, rl4));
    //             // vst1q_s32(hid.as_mut_ptr().add(i), rl1);
    //             // vst1q_s32(hid.as_mut_ptr().add(i + 4), rl2);
    //             // vst1q_s32(hid.as_mut_ptr().add(i + 8), rl3);
    //             // vst1q_s32(hid.as_mut_ptr().add(i + 12), rl4);
    //         }
    //     }

    //     // 2nd layer to output
    //     let wh = self.wlayer1i(prgs);
    //     let wdc1 = self.wl1biasi32(prgs);
    //     let mut hid2 = [0i32 ; N_HIDDEN2];
    //     hid2.copy_from_slice(wdc1);
    //     for j in 0..N_HIDDEN {
    //         unsafe {
    //             let input = hid[j]
    //             for i in (0..N_HIDDEN).step_by(16) {
    //                 let w = vld1_i16_x4(wh.as_ptr().add(i + N_HIDDEN2 * j));
    //                 // i16 -> i32
    //                 let w0 = vmovl_si16(w.0);
    //                 let w1 = vmovl_si16(w.1);
    //                 let w2 = vmovl_si16(w.2);
    //                 let w3 = vmovl_si16(w.3);
    //                 let mul0 = vmullq_n_i32(w0, input);
    //                 let mul1 = vmullq_n_i32(w1, input);
    //                 let mul2 = vmullq_n_i32(w2, input);
    //                 let mul3 = vmullq_n_i32(w3, input);

    //                 let inp = vld1q_s32_x4(hid2.as_ptr().add(i));
    //                 let s0 = vaddq_s32(w0, mul0);
    //                 let s1 = vaddq_s32(w1, mul1);
    //                 let s2 = vaddq_s32(w2, mul2);
    //                 let s3 = vaddq_s32(w3, mul3);

    //                 vst1q_s32_x4(hid2.as_ptr().add(i), int32x4x4(s0, s1, s2, s3));
    //             }
    //         }
    //     }

    //     let mut res = self.wl2biasi(prgs);
    //     let wh2 = self.wlayer2i(prgs);
    //     for (i, _h) in hid2.iter().enumerate().step_by(16) {
    //         unsafe {
    //             let inp = vld1q_f32_x4(hid2.as_ptr().add(i));
    //             // relu
    //             let zero = vmovq_n_s32(0);
    //             let inp0 = vmaxq_s32(zero, inp.0);
    //             let inp1 = vmaxq_s32(zero, inp.1);
    //             let inp2 = vmaxq_s32(zero, inp.2);
    //             let inp3 = vmaxq_s32(zero, inp.3);
    //             // /256
    //             let s0 = vshrq_n_s32(inp0, CONVERT_TO_FIXPOINT_I16_SHIFT);
    //             let s1 = vshrq_n_s32(inp1, CONVERT_TO_FIXPOINT_I16_SHIFT);
    //             let s2 = vshrq_n_s32(inp2, CONVERT_TO_FIXPOINT_I16_SHIFT);
    //             let s3 = vshrq_n_s32(inp3, CONVERT_TO_FIXPOINT_I16_SHIFT);
    //             // i32 -> i16
    //             let s0 = vqmovn_s32(s0);
    //             let s1 = vqmovn_s32(s1);
    //             let s2 = vqmovn_s32(s2);
    //             let s3 = vqmovn_s32(s3);

    //             let wei = vld1_i16_x4(wh2.as_ptr().add(i));
    //             let mul0 = vmul_s16(wei.0, s0);
    //             let mul1 = vmul_s16(wei.1, s1);
    //             let mul2 = vmul_s16(wei.2, s2);
    //             let mul3 = vmul_s16(wei.3, s3);

    //             let mul12 = vaddq_s32(mul0, mul1);
    //             let mul34 = vaddq_s32(mul2, mul3);
    //             let add4 = vaddq_s32(mul12, mul34);
    //             res += vaddvq_s32(add4);
    //         }
    //     }
    //     res as f32  / CONVERT_TO_FIXPOINT_I16T / CONVERT_TO_FIXPOINT_I16
    // }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simd_i16_2(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani(prgs);

        let wdc = self.wibiasi32(prgs);

        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        const N : usize = 16;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_si128(wei.as_ptr().add(i) as *const x86_64::__m128i);
                    // let w2 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let w3 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    // let w4 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 12) as *const x86_64::__m128i);
                    let h1 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i) as *const x86_64::__m128i);
                    let h2 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let h3 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let h4 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 12) as *const x86_64::__m128i);
                    // i16 -> i32
                    let zero = x86_64::_mm_setzero_si128();
                    let minus1 = x86_64::_mm_cmpgt_epi16(zero, w1);
                    let minus3 = x86_64::_mm_cmpgt_epi16(zero, w3);
                    let w2 =   x86_64::_mm_unpackhi_epi16(w1, minus1);
                    let w1 =   x86_64::_mm_unpacklo_epi16(w1, minus1);
                    let w4 =   x86_64::_mm_unpackhi_epi16(w3, minus3);
                    let w3 =   x86_64::_mm_unpacklo_epi16(w3, minus3);
                    let m1 = x86_64::_mm_add_epi32(w1, h1);
                    let m2 = x86_64::_mm_add_epi32(w2, h2);
                    let m3 = x86_64::_mm_add_epi32(w3, h3);
                    let m4 = x86_64::_mm_add_epi32(w4, h4);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i) as *mut x86_64::__m128i, m1);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i + 4) as *mut x86_64::__m128i, m2);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i, m3);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i + 12) as *mut x86_64::__m128i, m4);
                }
            }
        }

        let mut hid = [0i16 ; N_HIDDEN];
        for i in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i) as *const x86_64::__m128i);
                let h2 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let h3 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 8) as *const x86_64::__m128i);
                let h4 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 12) as *const x86_64::__m128i);

                // relu
                let zero = x86_64::_mm_setzero_si128();
                let y1 = x86_64::_mm_max_epi32(h1, zero);
                let y2 = x86_64::_mm_max_epi32(h2, zero);
                let y3 = x86_64::_mm_max_epi32(h3, zero);
                let y4 = x86_64::_mm_max_epi32(h4, zero);

                // i32 -> i32
                let y12 = x86_64::_mm_packs_epi32(y1, y2);
                let y34 = x86_64::_mm_packs_epi32(y3, y4);

                x86_64::_mm_storeu_si128(hid.as_mut_ptr().add(i) as *mut x86_64::__m128i, y12);
                // x86_64::_mm_storeu_si128(hid.as_mut_ptr().add(i + 4) as *mut x86_64::__m128i, y2);
                x86_64::_mm_storeu_si128(hid.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i, y34);
                // x86_64::_mm_storeu_si128(hid.as_mut_ptr().add(i + 12) as *mut x86_64::__m128i, y4);
            }
        }

        // 2nd layer to output
        let wh = self.wlayer1iv(prgs);
        let mut sum4 = [0i32 ; N_HIDDEN2];
        for j in 0..N_HIDDEN {
            unsafe {
                let x1 = x86_64::_mm_set1_epi16(hid[j]);
                for i in (0..N_HIDDEN2).step_by(16) {
                    let idx = i + N_HIDDEN2 * j;
                    let w1 = x86_64::_mm_load_si128(
                        wh.as_ptr().add(idx) as *const x86_64::__m128i);
                    // let w2 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 4));
                    let w3 = x86_64::_mm_load_si128(
                        wh.as_ptr().add(idx + 8) as *const x86_64::__m128i);
                    // let w4 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 12));
                    let mul1l = x86_64::_mm_mullo_epi16(x1, w1);
                    let mul3l = x86_64::_mm_mullo_epi16(x1, w3);
                    let mul1h = x86_64::_mm_mulhi_epi16(x1, w1);
                    let mul3h = x86_64::_mm_mulhi_epi16(x1, w3);
                    let mul1 = x86_64::_mm_unpacklo_epi16(mul1l, mul1h);
                    let mul3 = x86_64::_mm_unpacklo_epi16(mul3l, mul3h);
                    let mul2 = x86_64::_mm_unpackhi_epi16(mul1l, mul1h);
                    let mul4 = x86_64::_mm_unpackhi_epi16(mul3l, mul3h);

                    let s1 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i) as *const x86_64::__m128i);
                    let s2 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let s3 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let s4 = x86_64::_mm_loadu_si128(
                        sum4.as_ptr().add(i + 12) as *const x86_64::__m128i);

                    let s1 = x86_64::_mm_add_epi32(mul1, s1);
                    let s2 = x86_64::_mm_add_epi32(mul2, s2);
                    let s3 = x86_64::_mm_add_epi32(mul3, s3);
                    let s4 = x86_64::_mm_add_epi32(mul4, s4);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i) as *mut x86_64::__m128i,
                        s1);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i + 4) as *mut x86_64::__m128i,
                        s2);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i,
                        s3);
                    x86_64::_mm_storeu_si128(
                        sum4.as_mut_ptr().add(i + 12) as *mut x86_64::__m128i,
                        s4);
                }
            }
        }

        let dc1 = self.wl1biasi32(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        for i in (0..N_HIDDEN2).step_by(16) {
            unsafe {
                // dc
                let wdc1 = x86_64::_mm_load_si128(dc1.as_ptr().add(i) as *const x86_64::__m128i);
                let wdc2 = x86_64::_mm_load_si128(dc1.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let wdc3 = x86_64::_mm_load_si128(dc1.as_ptr().add(i + 8) as *const x86_64::__m128i);
                let wdc4 = x86_64::_mm_load_si128(dc1.as_ptr().add(i + 12) as *const x86_64::__m128i);

                let a = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i) as *const x86_64::__m128i);
                let b = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let c = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i + 8) as *const x86_64::__m128i);
                let d = x86_64::_mm_loadu_si128(
                    sum4.as_ptr().add(i + 12) as *const x86_64::__m128i);

                let s1 = x86_64::_mm_add_epi32(a, wdc1);
                let s2 = x86_64::_mm_add_epi32(b, wdc2);
                let s3 = x86_64::_mm_add_epi32(c, wdc3);
                let s4 = x86_64::_mm_add_epi32(d, wdc4);

                // relu
                let zero = x86_64::_mm_setzero_si128();
                let r1 = x86_64::_mm_max_epi32(s1, zero);
                let r2 = x86_64::_mm_max_epi32(s2, zero);
                let r3 = x86_64::_mm_max_epi32(s3, zero);
                let r4 = x86_64::_mm_max_epi32(s4, zero);

                let x1 = x86_64::_mm_cvtepi32_ps(r1);
                let x2 = x86_64::_mm_cvtepi32_ps(r2);
                let x3 = x86_64::_mm_cvtepi32_ps(r3);
                let x4 = x86_64::_mm_cvtepi32_ps(r4);
                let mag = x86_64::_mm_set1_ps(
                    1.0 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16);
                // let mag = x86_64::_mm_set1_ps(1.0 / CONVERT_TO_FIXPOINT_I16);
                let x1 = x86_64::_mm_mul_ps(x1, mag);
                let x2 = x86_64::_mm_mul_ps(x2, mag);
                let x3 = x86_64::_mm_mul_ps(x3, mag);
                let x4 = x86_64::_mm_mul_ps(x4, mag);

                let wh21 = x86_64::_mm_load_ps(wh2.as_ptr().add(i));
                let wh22 = x86_64::_mm_load_ps(wh2.as_ptr().add(i + 4));
                let wh23 = x86_64::_mm_load_ps(wh2.as_ptr().add(i + 8));
                let wh24 = x86_64::_mm_load_ps(wh2.as_ptr().add(i + 12));

                let y1 = x86_64::_mm_mul_ps(wh21, x1);
                let y2 = x86_64::_mm_mul_ps(wh22, x2);
                let y3 = x86_64::_mm_mul_ps(wh23, x3);
                let y4 = x86_64::_mm_mul_ps(wh24, x4);

                let y12 = x86_64::_mm_add_ps(y1, y2);
                let y34 = x86_64::_mm_add_ps(y3, y4);
                let y1234 = x86_64::_mm_add_ps(y12, y34);

                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i * 4), y1234);
            }
        }
        let mut res = self.wl2bias(prgs);
        for h in hid2.iter().take(N_HIDDEN2 / 4) {
            res += h;
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simd_i16_1(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani(prgs);

        let wdc = self.wibiasi32(prgs);

        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        const N : usize = 16;
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_si128(wei.as_ptr().add(i) as *const x86_64::__m128i);
                    // let w2 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let w3 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    // let w4 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 12) as *const x86_64::__m128i);
                    let h1 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i) as *const x86_64::__m128i);
                    let h2 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 4) as *const x86_64::__m128i);
                    let h3 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let h4 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 12) as *const x86_64::__m128i);
                    // i16 -> i32
                    let zero = x86_64::_mm_setzero_si128();
                    let minus1 = x86_64::_mm_cmpgt_epi16(zero, w1);
                    let minus3 = x86_64::_mm_cmpgt_epi16(zero, w3);
                    let w2 =   x86_64::_mm_unpackhi_epi16(w1, minus1);
                    let w1 =   x86_64::_mm_unpacklo_epi16(w1, minus1);
                    let w4 =   x86_64::_mm_unpackhi_epi16(w3, minus3);
                    let w3 =   x86_64::_mm_unpacklo_epi16(w3, minus3);
                    let m1 = x86_64::_mm_add_epi32(w1, h1);
                    let m2 = x86_64::_mm_add_epi32(w2, h2);
                    let m3 = x86_64::_mm_add_epi32(w3, h3);
                    let m4 = x86_64::_mm_add_epi32(w4, h4);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i) as *mut x86_64::__m128i, m1);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i + 4) as *mut x86_64::__m128i, m2);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m128i, m3);
                    x86_64::_mm_storeu_si128(hidi.as_mut_ptr().add(i + 12) as *mut x86_64::__m128i, m4);
                }
            }
        }

        let mut hid = [0f32 ; N_HIDDEN];
        for i in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i) as *const x86_64::__m128i);
                let h2 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 4) as *const x86_64::__m128i);
                let h3 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 8) as *const x86_64::__m128i);
                let h4 = x86_64::_mm_loadu_si128(hidi.as_ptr().add(i + 12) as *const x86_64::__m128i);

                // relu
                let zero = x86_64::_mm_setzero_si128();
                let y1 = x86_64::_mm_max_epi32(h1, zero);
                let y2 = x86_64::_mm_max_epi32(h2, zero);
                let y3 = x86_64::_mm_max_epi32(h3, zero);
                let y4 = x86_64::_mm_max_epi32(h4, zero);
                // i32 -> f32
                let y1 = x86_64::_mm_cvtepi32_ps(y1);
                let y2 = x86_64::_mm_cvtepi32_ps(y2);
                let y3 = x86_64::_mm_cvtepi32_ps(y3);
                let y4 = x86_64::_mm_cvtepi32_ps(y4);
                let mag = x86_64::_mm_set1_ps(1.0 / CONVERT_TO_FIXPOINT_I16);
                let y1 = x86_64::_mm_mul_ps(y1, mag);
                let y2 = x86_64::_mm_mul_ps(y2, mag);
                let y3 = x86_64::_mm_mul_ps(y3, mag);
                let y4 = x86_64::_mm_mul_ps(y4, mag);

                x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i), y1);
                x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 4), y2);
                x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 8), y3);
                x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 12), y4);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        let mut sum4 = [0f32 ; N_HIDDEN2 * 4];
        for j in (0..N_HIDDEN).step_by(16) {
            unsafe {
                let x1 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j));
                let x2 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 4));
                let x3 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 8));
                let x4 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 12));
                for i in 0..N_HIDDEN2 {
                    let idx = i * N_HIDDEN + j;
                    let w1 = x86_64::_mm_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 4));
                    let w3 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 8));
                    let w4 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 12));
                    let mul1 = x86_64::_mm_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm_mul_ps(x2, w2);
                    // let mul3 = x86_64::_mm_mul_ps(x3, w3);
                    // let mul4 = x86_64::_mm_mul_ps(x4, w4);
                    // let s12 = x86_64::_mm_add_ps(mul1, mul2);
                    // let s34 = x86_64::_mm_add_ps(mul3, mul4);
                    let s12 = x86_64::_mm_fmadd_ps(x3, w3, mul1);
                    let s34 = x86_64::_mm_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm_add_ps(s12, s34);
                    let s4 = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4));
                    let s4 = x86_64::_mm_add_ps(s1234, s4);
                    x86_64::_mm_storeu_ps(sum4.as_mut_ptr().add(i * 4), s4);
                }
            }
        }
        for i in (0..N_HIDDEN2).step_by(4) {
            unsafe {
                let a = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4));
                let b = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 4));
                let c = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 8));
                let d = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 12));
                let a0c0a1c1 = x86_64::_mm_unpacklo_ps(a, c);
                let b0d0b1d1 = x86_64::_mm_unpacklo_ps(b, d);
                let a2c2a3c3 = x86_64::_mm_unpackhi_ps(a, c);
                let b2d2b3d3 = x86_64::_mm_unpackhi_ps(b, d);
                let a0 = x86_64::_mm_unpacklo_ps(a0c0a1c1, b0d0b1d1);
                let a1 = x86_64::_mm_unpackhi_ps(a0c0a1c1, b0d0b1d1);
                let a2 = x86_64::_mm_unpacklo_ps(a2c2a3c3, b2d2b3d3);
                let a3 = x86_64::_mm_unpackhi_ps(a2c2a3c3, b2d2b3d3);
                let s1 = x86_64::_mm_add_ps(a0, a1);
                let s2 = x86_64::_mm_add_ps(a2, a3);
                let s3 = x86_64::_mm_add_ps(s1, s2);

                let dc = x86_64::_mm_load_ps(wdc1.as_ptr().add(i));
                let s4 = x86_64::_mm_add_ps(s3, dc);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i), s4);
            }
        }
        for j in 0..N_HIDDEN2 / 16 {
            unsafe {  // relu
                let h1 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16));
                let h2 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16 + 4));
                let h3 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16 + 8));
                let h4 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(j * 16 + 12));
                let zero = x86_64::_mm_setzero_ps();
                let h1 = x86_64::_mm_max_ps(h1, zero);
                let h2 = x86_64::_mm_max_ps(h2, zero);
                let h3 = x86_64::_mm_max_ps(h3, zero);
                let h4 = x86_64::_mm_max_ps(h4, zero);
                let wh21 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16));
                let wh22 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16 + 4));
                let wh23 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16 + 8));
                let wh24 = x86_64::_mm_load_ps(wh2.as_ptr().add(j * 16 + 12));

                let y1 = x86_64::_mm_mul_ps(wh21, h1);
                let y2 = x86_64::_mm_mul_ps(wh22, h2);
                let y3 = x86_64::_mm_mul_ps(wh23, h3);
                let y4 = x86_64::_mm_mul_ps(wh24, h4);
                let y12 = x86_64::_mm_add_ps(y1, y2);
                let y34 = x86_64::_mm_add_ps(y3, y4);
                let y1234 = x86_64::_mm_add_ps(y12, y34);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(j * 4), y1234);
            }
        }
        for h in hid2.iter().take(N_HIDDEN2 / 4) {
            res += h;
        }
        res
    }

    #[cfg(target_arch="aarch64")]
    pub fn evaluatev12bb_simd_mul(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbanv(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 16;
        let mut hid = [0f32 ; N_HIDDEN];
        let mut bit = bitboard::LSB_CELL;
        // cells
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w = vld1q_f32_x4(wei.as_ptr().add(i));
                    let h = vld1q_f32_x4(hid.as_ptr().add(i));
                    let w1 = vaddq_f32(h.0, w.0);
                    let w2 = vaddq_f32(h.1, w.1);
                    let w3 = vaddq_f32(h.2, w.2);
                    let w4 = vaddq_f32(h.3, w.3);
                    vst1q_f32_x4(hid.as_mut_ptr().add(i),
                        float32x4x4_t(w1, w2, w3, w4));
                }
            }
        }
        for i in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let sum4 = vld1q_f32_x4(hid.as_ptr().add(i));

                let wdc4 = vld1q_f32_x4(wdc.as_ptr().add(i));
                let sum41 = vaddq_f32(sum4.0, wdc4.0);
                let sum42 = vaddq_f32(sum4.1, wdc4.1);
                let sum43 = vaddq_f32(sum4.2, wdc4.2);
                let sum44 = vaddq_f32(sum4.3, wdc4.3);
                // relu
                let zero = vmovq_n_f32(0.0);
                let rl1 = vmaxq_f32(zero, sum41);
                let rl2 = vmaxq_f32(zero, sum42);
                let rl3 = vmaxq_f32(zero, sum43);
                let rl4 = vmaxq_f32(zero, sum44);
                vst1q_f32_x4(hid.as_mut_ptr().add(i), float32x4x4_t(rl1, rl2, rl3, rl4));
                // vst1q_f32(hid.as_mut_ptr().add(i), rl1);
                // vst1q_f32(hid.as_mut_ptr().add(i + 4), rl2);
                // vst1q_f32(hid.as_mut_ptr().add(i + 8), rl3);
                // vst1q_f32(hid.as_mut_ptr().add(i + 12), rl4);
            }
        }
        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        hid2.copy_from_slice(wdc1);
        for j in (0..N_HIDDEN).step_by(32) {
            unsafe {
                let inp = vld1q_f32_x4(hid.as_ptr().add(j));
                for (i, h2) in hid2.iter_mut().enumerate() {
                    let wei = vld1q_f32_x4(wh.as_ptr().add(i * N_HIDDEN + j));
                    let mul0 = vmulq_f32(inp.0, wei.0);
                    let mul1 = vmulq_f32(inp.1, wei.1);
                    let mul2 = vmlaq_f32(mul0, inp.2, wei.2);
                    let mul3 = vmlaq_f32(mul1, inp.3, wei.3);
                    let add4 = vaddq_f32(mul2, mul3);

                    let inp = vld1q_f32_x4(hid.as_ptr().add(j + 16));
                    let wei = vld1q_f32_x4(wh.as_ptr().add(i * N_HIDDEN + j + 16));
                    let mul0 = vmulq_f32(inp.0, wei.0);
                    let mul1 = vmulq_f32(inp.1, wei.1);
                    let mul2 = vmlaq_f32(mul0, inp.2, wei.2);
                    let mul3 = vmlaq_f32(mul1, inp.3, wei.3);
                    let add42 = vaddq_f32(mul2, mul3);
                    let add4 = vaddq_f32(add4, add42);
                    *h2 += vaddvq_f32(add4);
                }
            }
        }
        for (i, _h) in hid2.iter().enumerate().step_by(16) {
            unsafe {
                let inp = vld1q_f32_x4(hid2.as_ptr().add(i));
                // relu
                let zero = vmovq_n_f32(0.0);
                let inp0 = vmaxq_f32(zero, inp.0);
                let inp1 = vmaxq_f32(zero, inp.1);
                let inp2 = vmaxq_f32(zero, inp.2);
                let inp3 = vmaxq_f32(zero, inp.3);

                let wei = vld1q_f32_x4(wh2.as_ptr().add(i));
                let mul0 = vmulq_f32(wei.0, inp0);
                let mul1 = vmulq_f32(wei.1, inp1);
                let mul2 = vmulq_f32(wei.2, inp2);
                let mul3 = vmulq_f32(wei.3, inp3);
                let mul12 = vaddq_f32(mul0, mul1);
                let mul34 = vaddq_f32(mul2, mul3);
                let add4 = vaddq_f32(mul12, mul34);
                res += vaddvq_f32(add4);
            }
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbanv(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 32;
        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            // let wei = if b != 0 {
            //     &ow[idx * N_HIDDEN * 2 .. ]
            // } else {
            //     &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            // };
            let wei =
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN * (b == 0) as usize .. ];
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm256_load_ps(wei.as_ptr().add(i));
                    let w2 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 8));
                    let w3 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 16));
                    let w4 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 24));
                    let h1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i));
                    let h2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i + 8));
                    let h3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i + 16));
                    let h4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i + 24));
                    let m1 = x86_64::_mm256_add_ps(w1, h1);
                    let m2 = x86_64::_mm256_add_ps(w2, h2);
                    let m3 = x86_64::_mm256_add_ps(w3, h3);
                    let m4 = x86_64::_mm256_add_ps(w4, h4);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i), m1);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 8), m2);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 16), m3);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 24), m4);
                }
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);

        let mut hid2 = [0f32 ; N_HIDDEN2];
        hid2.copy_from_slice(wdc1);
        let mut sumhn = [0f32 ; N_HIDDEN2 * 4 * 2];
        for j in (0..N_HIDDEN).step_by(32) {
            unsafe {
                let h1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j));
                let h2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 8));
                let h3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 16));
                let h4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 24));
                // relu
                let zero = x86_64::_mm256_setzero_ps();
                let x1 = x86_64::_mm256_max_ps(zero, h1);
                let x2 = x86_64::_mm256_max_ps(zero, h2);
                let x3 = x86_64::_mm256_max_ps(zero, h3);
                let x4 = x86_64::_mm256_max_ps(zero, h4);
                for i in 0..N_HIDDEN2 {
                    let idx = i * N_HIDDEN + j;
                    let w1 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 8));
                    let w3 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 16));
                    let w4 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 24));
                    let mul1 = x86_64::_mm256_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm256_mul_ps(x2, w2);
                    // let mul3 = x86_64::_mm256_mul_ps(x3, w3);
                    // let mul4 = x86_64::_mm256_mul_ps(x4, w4);
                    // let s12 = x86_64::_mm256_add_ps(mul1, mul2);
                    // let s34 = x86_64::_mm256_add_ps(mul3, mul4);
                    let s12 = x86_64::_mm256_fmadd_ps(x3, w3, mul1);
                    let s34 = x86_64::_mm256_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm256_add_ps(s12, s34);
                    x86_64::_mm256_storeu_ps(
                            sumhn.as_mut_ptr().add(i * 8), s1234);
                }
                for (k, _hn) in sumhn.iter().enumerate().step_by(32) {
                    use std::arch::x86_64::_mm256_extractf128_ps;

                    let a = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k));  // a0~a7
                    let b = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 8));  // a8~a15
                    let c = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 16));  // b0~b7
                    let d = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 24));  // b8~b15
                    let a0c0 = x86_64::_mm256_unpacklo_ps(a, b);
                    let b0d0 = x86_64::_mm256_unpacklo_ps(c, d);
                    let a2c2 = x86_64::_mm256_unpackhi_ps(a, b);
                    let b2d2 = x86_64::_mm256_unpackhi_ps(c, d);
                    let s1 = x86_64::_mm256_add_ps(a0c0, a2c2);
                    let s2 = x86_64::_mm256_add_ps(b0d0, b2d2);
                    let t1 = x86_64::_mm256_shuffle_ps(s1, s2,
                            0b01000100/*(1 << 6) | (0 << 4) | (1 << 2) | 0*/);
                    let t2 = x86_64::_mm256_shuffle_ps(s1, s2,
                            0b11101110/*(3 << 6) | (2 << 4) | (3 << 2) | 2*/);
                    let s3 = x86_64::_mm256_add_ps(t1, t2);
                    let s4 = _mm256_extractf128_ps(s3, 1);
                    let s5 = x86_64::_mm_add_ps(
                            s4, x86_64::_mm256_castps256_ps128(s3));
                    let hn2 = x86_64::_mm_loadu_ps(
                            hid2.as_mut_ptr().add(k / 8));
                    let s6 = x86_64::_mm_add_ps(s5, hn2);
                    x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(k / 8), s6);
                }
            }
        }
        if N_HIDDEN2 >= 32 {
            for i in (0..N_HIDDEN2).step_by(32) {
                unsafe {  // relu
                    let x1 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i));
                    let x2 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 8));
                    let x3 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 16));
                    let x4 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 24));
                    let zero = x86_64::_mm256_setzero_ps();
                    let h1 = x86_64::_mm256_max_ps(zero, x1);
                    let h2 = x86_64::_mm256_max_ps(zero, x2);
                    let h3 = x86_64::_mm256_max_ps(zero, x3);
                    let h4 = x86_64::_mm256_max_ps(zero, x4);
                    let w1 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i));
                    let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 8));
                    let w3 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 16));
                    let w4 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 24));
                    let y1 = x86_64::_mm256_mul_ps(h1, w1);
                    let y2 = x86_64::_mm256_mul_ps(h2, w2);
                    let y3 = x86_64::_mm256_mul_ps(h3, w3);
                    let y4 = x86_64::_mm256_mul_ps(h4, w4);
                    let y12 = x86_64::_mm256_add_ps(y1, y2);
                    let y34 = x86_64::_mm256_add_ps(y3, y4);
                    let y1234 = x86_64::_mm256_add_ps(y12, y34);
                    let s1 = x86_64::_mm256_castps256_ps128(y1234);
                    let s2 = x86_64::_mm256_extractf128_ps(y1234, 1);
                    let s4 = x86_64::_mm_add_ps(s1, s2);
                    x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i / 8), s4);
                }
            }
            for h in hid2.iter().take(N_HIDDEN2 / 8) {
                res += h;
            }
        } else {
            unsafe {  // relu
                let x1 = x86_64::_mm256_loadu_ps(hid2.as_ptr());
                let x2 = x86_64::_mm256_loadu_ps(hid2.as_ptr().add(8));
                let zero = x86_64::_mm256_setzero_ps();
                let h1 = x86_64::_mm256_max_ps(zero, x1);
                let h2 = x86_64::_mm256_max_ps(zero, x2);
                let w1 = x86_64::_mm256_load_ps(wh2.as_ptr());
                let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(8));
                let y1 = x86_64::_mm256_mul_ps(h1, w1);
                let y2 = x86_64::_mm256_mul_ps(h2, w2);
                let y3 = x86_64::_mm256_add_ps(y1, y2);
                let s1 = x86_64::_mm256_castps256_ps128(y3);
                let s2 = x86_64::_mm256_extractf128_ps(y3, 1);
                let s4 = x86_64::_mm_add_ps(s1, s2);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr(), s4);
            }
            for h in hid2.iter().take(4) {
                res += h;
            }
         }
         res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx_2(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbanv(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 32;
        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            // let wei = if b != 0 {
            //     &ow[idx * N_HIDDEN * 2 .. ]
            // } else {
            //     &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            // };
            let wei =
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN * (b == 0) as usize .. ];
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm256_load_ps(wei.as_ptr().add(i));
                    let w2 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 8));
                    let w3 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 16));
                    let w4 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 24));
                    let h1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i));
                    let h2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i + 8));
                    let h3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i + 16));
                    let h4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(i + 24));
                    let m1 = x86_64::_mm256_add_ps(w1, h1);
                    let m2 = x86_64::_mm256_add_ps(w2, h2);
                    let m3 = x86_64::_mm256_add_ps(w3, h3);
                    let m4 = x86_64::_mm256_add_ps(w4, h4);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i), m1);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 8), m2);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 16), m3);
                    x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 24), m4);
                }
            }
        }

        for hidx in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx));
                let h2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx + 8));
                let h3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx + 16));
                let h4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx + 24));

                // relu
                let zero = x86_64::_mm256_setzero_ps();
                let y1 = x86_64::_mm256_max_ps(zero, h1);
                let y2 = x86_64::_mm256_max_ps(zero, h2);
                let y3 = x86_64::_mm256_max_ps(zero, h3);
                let y4 = x86_64::_mm256_max_ps(zero, h4);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx), y1);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 8), y2);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 16), y3);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 24), y4);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1v(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);

        let mut hid2 = [0f32 ; N_HIDDEN2];
        hid2.copy_from_slice(wdc1);
        for j in 0..N_HIDDEN {
            if hid[j] == 0.0 {continue;}

            unsafe {
                let x1 = x86_64::_mm256_set1_ps(hid[j]);
                for i in (0..N_HIDDEN2).step_by(16) {
                    let idx = i + N_HIDDEN2 * j;
                    let w1 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 8));
                    let h1 = x86_64::_mm256_loadu_ps(hid2.as_ptr().add(i));
                    let h2 = x86_64::_mm256_loadu_ps(hid2.as_ptr().add(i + 8));
                    let mul1 = x86_64::_mm256_fmadd_ps(x1, w1, h1);
                    let mul2 = x86_64::_mm256_fmadd_ps(x1, w2, h2);
                    x86_64::_mm256_storeu_ps(
                            hid2.as_mut_ptr().add(i), mul1);
                    x86_64::_mm256_storeu_ps(
                            hid2.as_mut_ptr().add(i + 8), mul2);
                }
            }
        }
        if N_HIDDEN2 >= 32 {
            for i in (0..N_HIDDEN2).step_by(32) {
                unsafe {  // relu
                    let x1 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i));
                    let x2 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 8));
                    let x3 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 16));
                    let x4 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 24));
                    let zero = x86_64::_mm256_setzero_ps();
                    let h1 = x86_64::_mm256_max_ps(zero, x1);
                    let h2 = x86_64::_mm256_max_ps(zero, x2);
                    let h3 = x86_64::_mm256_max_ps(zero, x3);
                    let h4 = x86_64::_mm256_max_ps(zero, x4);
                    let w1 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i));
                    let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 8));
                    let w3 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 16));
                    let w4 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 24));
                    let y1 = x86_64::_mm256_mul_ps(h1, w1);
                    let y2 = x86_64::_mm256_mul_ps(h2, w2);
                    let y3 = x86_64::_mm256_mul_ps(h3, w3);
                    let y4 = x86_64::_mm256_mul_ps(h4, w4);
                    let y12 = x86_64::_mm256_add_ps(y1, y2);
                    let y34 = x86_64::_mm256_add_ps(y3, y4);
                    let y1234 = x86_64::_mm256_add_ps(y12, y34);
                    let s1 = x86_64::_mm256_castps256_ps128(y1234);
                    let s2 = x86_64::_mm256_extractf128_ps(y1234, 1);
                    let s4 = x86_64::_mm_add_ps(s1, s2);
                    x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i / 8), s4);
                }
            }
            for h in hid2.iter().take(N_HIDDEN2 / 8) {
                res += h;
            }
        } else {
            unsafe {  // relu
                let x1 = x86_64::_mm256_loadu_ps(hid2.as_ptr());
                let x2 = x86_64::_mm256_loadu_ps(hid2.as_ptr().add(8));
                let zero = x86_64::_mm256_setzero_ps();
                let h1 = x86_64::_mm256_max_ps(zero, x1);
                let h2 = x86_64::_mm256_max_ps(zero, x2);
                let w1 = x86_64::_mm256_load_ps(wh2.as_ptr());
                let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(8));
                let y1 = x86_64::_mm256_mul_ps(h1, w1);
                let y2 = x86_64::_mm256_mul_ps(h2, w2);
                let y3 = x86_64::_mm256_add_ps(y1, y2);
                let s1 = x86_64::_mm256_castps256_ps128(y3);
                let s2 = x86_64::_mm256_extractf128_ps(y3, 1);
                let s4 = x86_64::_mm_add_ps(s1, s2);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr(), s4);
            }
            for h in hid2.iter().take(4) {
                res += h;
            }
         }
         res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx_i16(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani32(prgs);
        let wdc = self.wibiasi32(prgs);
        const N : usize = 32;
        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        let mut bit = bitboard::LSB_CELL;

        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            // let wei = if b != 0 {
            //     &ow[idx * N_HIDDEN * 2 .. ]
            // } else {
            //     &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            // };
            let wei =
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN * (b == 0) as usize .. ];
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i) as *const x86_64::__m256i);
                    let w2 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 8) as *const x86_64::__m256i);
                    let w3 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let w4 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 24) as *const x86_64::__m256i);

                    let h1 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i) as *const x86_64::__m256i);
                    let h2 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i + 8) as *const x86_64::__m256i);
                    let h3 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let h4 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i + 24) as *const x86_64::__m256i);

                    let m1 = x86_64::_mm256_add_epi32(w1, h1);
                    let m2 = x86_64::_mm256_add_epi32(w2, h2);
                    let m3 = x86_64::_mm256_add_epi32(w3, h3);
                    let m4 = x86_64::_mm256_add_epi32(w4, h4);

                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i) as *mut x86_64::__m256i, m1);
                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m256i,
                        m2);
                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i + 16) as *mut x86_64::__m256i,
                        m3);
                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i + 24) as *mut x86_64::__m256i,
                        m4);
                }
            }
        }

        let mut hid = [0i16 ; N_HIDDEN];
        const M : usize = 64;
        for hidx in (0..N_HIDDEN).step_by(M) {
            unsafe {
                let h1 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx) as *const x86_64::__m256i);
                let h2 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 8) as *const x86_64::__m256i);
                let h3 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 16) as *const x86_64::__m256i);
                let h4 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 24) as *const x86_64::__m256i);
                let h5 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 32) as *const x86_64::__m256i);
                let h6 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 40) as *const x86_64::__m256i);
                let h7 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 48) as *const x86_64::__m256i);
                let h8 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 56) as *const x86_64::__m256i);

                // i32 -> i16
                let y12 = x86_64::_mm256_packs_epi32(h1, h2);
                let y34 = x86_64::_mm256_packs_epi32(h3, h4);
                let y56 = x86_64::_mm256_packs_epi32(h5, h6);
                let y78 = x86_64::_mm256_packs_epi32(h7, h8);
                let y12 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                let y34 = x86_64::_mm256_permute4x64_epi64(y34, 0xD8);
                let y56 = x86_64::_mm256_permute4x64_epi64(y56, 0xD8);
                let y78 = x86_64::_mm256_permute4x64_epi64(y78, 0xD8);

                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let y12 = x86_64::_mm256_max_epi16(zero, y12);
                let y34 = x86_64::_mm256_max_epi16(zero, y34);
                let y56 = x86_64::_mm256_max_epi16(zero, y56);
                let y78 = x86_64::_mm256_max_epi16(zero, y78);

                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx) as *mut x86_64::__m256i, y12);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 16) as *mut x86_64::__m256i,
                    y34);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 32) as *mut x86_64::__m256i,
                    y56);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 48) as *mut x86_64::__m256i,
                    y78);
            }
        }

        // 1st layer to 2nd layer
        let wh = self.wlayer1i(prgs);
        let wdc1 = self.wl1biasi32(prgs);
        let mut sumhn = [0i32 ; N_HIDDEN2];
        // sumhn.copy_from_slice(wdc1);
        // let mut sumhn4 = [0i32 ; N_HIDDEN / 16];
        // let mut sumhn4 = [0i32 ; 4 * 2];
        for j in (0..N_HIDDEN2).step_by(2) {
            unsafe {
                let mut sum8 = x86_64::_mm256_setzero_si256();
                let mut sum82 = x86_64::_mm256_setzero_si256();
                for i in (0..N_HIDDEN).step_by(64) {
                    let x1 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i) as *const x86_64::__m256i);
                    let x2 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let x3 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i + 32) as *const x86_64::__m256i);
                    let x4 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i + 48) as *const x86_64::__m256i);
                    let idx = i + N_HIDDEN * j;
                    let w1 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx) as *const x86_64::__m256i);
                    let w2 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 16) as *const x86_64::__m256i);
                    let w3 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 32) as *const x86_64::__m256i);
                    let w4 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 48) as *const x86_64::__m256i);
                    let idx = i + N_HIDDEN * j + N_HIDDEN;
                    let w5 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx) as *const x86_64::__m256i);
                    let w6 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 16) as *const x86_64::__m256i);
                    let w7 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 32) as *const x86_64::__m256i);
                    let w8 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 48) as *const x86_64::__m256i);

                    let sum1234 = x86_64::_mm256_madd_epi16(x1, w1);
                    let sum5678 = x86_64::_mm256_madd_epi16(x2, w2);
                    let sum9abc = x86_64::_mm256_madd_epi16(x3, w3);
                    let sumdefg = x86_64::_mm256_madd_epi16(x4, w4);
                    let sum12342 = x86_64::_mm256_madd_epi16(x1, w5);
                    let sum56782 = x86_64::_mm256_madd_epi16(x2, w6);
                    let sum9abc2 = x86_64::_mm256_madd_epi16(x3, w7);
                    let sumdefg2 = x86_64::_mm256_madd_epi16(x4, w8);

                    let sum18 = x86_64::_mm256_add_epi32(sum1234, sum5678);
                    let sum9g = x86_64::_mm256_add_epi32(sum9abc, sumdefg);
                    let sum182 = x86_64::_mm256_add_epi32(sum12342, sum56782);
                    let sum9g2 = x86_64::_mm256_add_epi32(sum9abc2, sumdefg2);
                    let sum1g = x86_64::_mm256_add_epi32(sum18, sum9g);
                    let sum1g2 = x86_64::_mm256_add_epi32(sum182, sum9g2);

                    sum8 = x86_64::_mm256_add_epi32(sum8, sum1g);
                    sum82 = x86_64::_mm256_add_epi32(sum82, sum1g2);
                }

                let x = x86_64::_mm256_permute2x128_si256(sum8, sum82, 0x20);
                let y = x86_64::_mm256_permute2x128_si256(sum8, sum82, 0x31);
                let xy = x86_64::_mm256_add_epi32(x, y);
                let z = x86_64::_mm256_shuffle_epi32(
                        xy, Self::mm_shuffle(3, 2, 3, 2));
                let xyz = x86_64::_mm256_add_epi32(xy, z);
                let w = x86_64::_mm256_shuffle_epi32(
                        xyz, Self::mm_shuffle(1, 1, 1, 1));
                let wxyz = x86_64::_mm256_add_epi32(xyz, w);

                let h1 = x86_64::_mm256_extract_epi32(wxyz, 0);
                let h2 = x86_64::_mm256_extract_epi32(wxyz, 4);

                sumhn[j] = h1 + wdc1[j];
                sumhn[j + 1] = h2 + wdc1[j + 1];
            }
        }

        let wh2 = self.wlayer2i(prgs);
        let mut hid2 = [0i32 ; N_HIDDEN2 / 4];

        if N_HIDDEN2 == 16 {
            let mut result = self.wl2biasi(prgs) as i32 * CONVERT_TO_FIXPOINT_I16 as i32;
            for i in (0..N_HIDDEN2).step_by(16) {
                unsafe {
                    let s1 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                    let s2 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);

                    let s1 = x86_64::_mm256_srai_epi32(
                            s1, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s2 = x86_64::_mm256_srai_epi32(
                            s2, CONVERT_TO_FIXPOINT_I16_SHIFT);

                    // i32 -> i16
                    let y12 = x86_64::_mm256_packs_epi32(s1, s2);
                    let x1 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                    // relu
                    let zero = x86_64::_mm256_setzero_si256();
                    let x1 = x86_64::_mm256_max_epi16(x1, zero);

                    let w1 = x86_64::_mm256_loadu_si256(
                        wh2.as_ptr().add(i) as *const x86_64::__m256i);

                    let y1 = x86_64::_mm256_madd_epi16(x1, w1);

                    let y5678 = x86_64::_mm256_extracti128_si256(y1, 1);
                    let y1234 = x86_64::_mm256_castsi256_si128(y1);
                    let y18 = x86_64::_mm_add_epi32(y1234, y5678);

                    // x86_64::_mm_storeu_si128(
                    //     hid2.as_mut_ptr().add(i / 4) as *mut x86_64::__m128i,
                    //     y18);
                    let y18f = x86_64::_mm_castsi128_ps(y18);
                    let y = x86_64::_mm_movehl_ps(y18f, y18f);
                    let z = x86_64::_mm_add_epi32(y18, x86_64::_mm_castps_si128(y));
                    let z2 = x86_64::_mm_shuffle_epi32(z, 1);
                    let res = x86_64::_mm_add_epi32(z, z2);
                    result += x86_64::_mm_cvtsi128_si32(res);
                }
            }
            return result as f32 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16;
        } else {
            for i in (0..N_HIDDEN2).step_by(32) {
                unsafe {
                    let s1 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                    let s2 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);
                    let s3 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 16) as *const x86_64::__m256i);
                    let s4 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 24) as *const x86_64::__m256i);

                    let s1 = x86_64::_mm256_srai_epi32(
                            s1, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s2 = x86_64::_mm256_srai_epi32(
                            s2, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s3 = x86_64::_mm256_srai_epi32(
                            s3, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s4 = x86_64::_mm256_srai_epi32(
                            s4, CONVERT_TO_FIXPOINT_I16_SHIFT);

                    // i32 -> i16
                    let y12 = x86_64::_mm256_packs_epi32(s1, s2);
                    let y34 = x86_64::_mm256_packs_epi32(s3, s4);
                    let x1 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                    let x2 = x86_64::_mm256_permute4x64_epi64(y34, 0xD8);
                    // relu
                    let zero = x86_64::_mm256_setzero_si256();
                    let x1 = x86_64::_mm256_max_epi16(x1, zero);
                    let x2 = x86_64::_mm256_max_epi16(x2, zero);

                    let w1 = x86_64::_mm256_loadu_si256(
                        wh2.as_ptr().add(i) as *const x86_64::__m256i);
                    let w2 = x86_64::_mm256_loadu_si256(
                        wh2.as_ptr().add(i + 16) as *const x86_64::__m256i);

                    let y1 = x86_64::_mm256_madd_epi16(x1, w1);
                    let y2 = x86_64::_mm256_madd_epi16(x2, w2);

                    let y1 = x86_64::_mm256_add_epi32(y1, y2);

                    let y5678 = x86_64::_mm256_extracti128_si256(y1, 1);
                    let y1234 = x86_64::_mm256_castsi256_si128(y1);
                    let y18 = x86_64::_mm_add_epi32(y1234, y5678);

                    x86_64::_mm_storeu_si128(
                        hid2.as_mut_ptr().add(i / 8) as *mut x86_64::__m128i,
                        y18);
                }
            }
        }

        // let mut res = self.wl2biasi(prgs) as i32 * CONVERT_TO_FIXPOINT_I16 as i32;
        // for i in 0..N_HIDDEN2 / 4 {
        //     res += hid2[i] as i32;
        // }
        // res as f32 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
        hid2.iter().fold(
                self.wl2biasi(prgs) as i32 * CONVERT_TO_FIXPOINT_I16 as i32,
                      |init, a| init + a) as f32
            / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx_i16_4(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani32(prgs);
        let wdc = self.wibiasi32(prgs);
        const N : usize = 32;
        let mut hidi = [0i32 ; N_HIDDEN];
        // hidi.copy_from_slice(wdc);
        let mut hid = [0i16 ; N_HIDDEN];
        let bit = bitboard::LSB_CELL;

        for i in (0..N_HIDDEN).step_by(64) {
            unsafe {
                let mut sum1 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i) as *const x86_64::__m256i);
                let mut sum2 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 8) as *const x86_64::__m256i);
                let mut sum3 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 16) as *const x86_64::__m256i);
                let mut sum4 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 24) as *const x86_64::__m256i);
                let mut sum5 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 32) as *const x86_64::__m256i);
                let mut sum6 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 40) as *const x86_64::__m256i);
                let mut sum7 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 48) as *const x86_64::__m256i);
                let mut sum8 = x86_64::_mm256_loadu_si256(wdc.as_ptr().add(i + 56) as *const x86_64::__m256i);

                for idx in 0..bitboard::CELL_2D {
                    let b = black & (bit << idx);
                    let w = white & (bit << idx);

                    if b | w == 0 {continue;}  // no stone

                    let wei =
                        &ow[idx * N_HIDDEN * 2 + N_HIDDEN * (b == 0) as usize .. ];
                    let w1 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i) as *const x86_64::__m256i);
                    let w2 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 8) as *const x86_64::__m256i);
                    let w3 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let w4 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 24) as *const x86_64::__m256i);

                    // let exist = x86_64::_mm256_set1_epi64x((b | w) as i64);
                    // let zero = x86_64::_mm256_setzero_si256();
                    // let mask = x86_64::_mm256_cmpeq_epi64(exist, zero);

                    // let w1 = x86_64::_mm256_andnot_si256(mask, w1);
                    // let w2 = x86_64::_mm256_andnot_si256(mask, w2);
                    // let w3 = x86_64::_mm256_andnot_si256(mask, w3);
                    // let w4 = x86_64::_mm256_andnot_si256(mask, w4);

                    sum1 = x86_64::_mm256_add_epi32(w1, sum1);
                    sum2 = x86_64::_mm256_add_epi32(w2, sum2);
                    sum3 = x86_64::_mm256_add_epi32(w3, sum3);
                    sum4 = x86_64::_mm256_add_epi32(w4, sum4);

                    let w5 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 32) as *const x86_64::__m256i);
                    let w6 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 40) as *const x86_64::__m256i);
                    let w7 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 48) as *const x86_64::__m256i);
                    let w8 = x86_64::_mm256_load_si256(
                        wei.as_ptr().add(i + 56) as *const x86_64::__m256i);

                    // let w5 = x86_64::_mm256_andnot_si256(mask, w5);
                    // let w6 = x86_64::_mm256_andnot_si256(mask, w6);
                    // let w7 = x86_64::_mm256_andnot_si256(mask, w7);
                    // let w8 = x86_64::_mm256_andnot_si256(mask, w8);

                    sum5 = x86_64::_mm256_add_epi32(w5, sum5);
                    sum6 = x86_64::_mm256_add_epi32(w6, sum6);
                    sum7 = x86_64::_mm256_add_epi32(w7, sum7);
                    sum8 = x86_64::_mm256_add_epi32(w8, sum8);
                }

                // i32 -> i16
                let y12 = x86_64::_mm256_packs_epi32(sum1, sum2);
                let y34 = x86_64::_mm256_packs_epi32(sum3, sum4);
                let y56 = x86_64::_mm256_packs_epi32(sum5, sum6);
                let y78 = x86_64::_mm256_packs_epi32(sum7, sum8);
                let y12 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                let y34 = x86_64::_mm256_permute4x64_epi64(y34, 0xD8);
                let y56 = x86_64::_mm256_permute4x64_epi64(y56, 0xD8);
                let y78 = x86_64::_mm256_permute4x64_epi64(y78, 0xD8);

                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let y12 = x86_64::_mm256_max_epi16(zero, y12);
                let y34 = x86_64::_mm256_max_epi16(zero, y34);
                let y56 = x86_64::_mm256_max_epi16(zero, y56);
                let y78 = x86_64::_mm256_max_epi16(zero, y78);

                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(i) as *mut x86_64::__m256i, y12);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(i + 16) as *mut x86_64::__m256i,
                    y34);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(i + 32) as *mut x86_64::__m256i,
                    y56);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(i + 48) as *mut x86_64::__m256i,
                    y78);
            }
        }

        // 1st layer to 2nd layer
        let wh = self.wlayer1i(prgs);
        let wdc1 = self.wl1biasi32(prgs);
        let mut sumhn = [0i32 ; N_HIDDEN2];
        // sumhn.copy_from_slice(wdc1);
        // let mut sumhn4 = [0i32 ; N_HIDDEN / 16];
        let mut sumhn4 = [0i32 ; 4 * 2];
        for j in (0..N_HIDDEN2).step_by(2) {
            unsafe {
            let mut sum8 = x86_64::_mm256_setzero_si256();
            let mut sum82 = x86_64::_mm256_setzero_si256();
            for i in (0..N_HIDDEN).step_by(64) {
                    let x1 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i) as *const x86_64::__m256i);
                    let x2 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let x3 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i + 32) as *const x86_64::__m256i);
                    let x4 = x86_64::_mm256_loadu_si256(
                        hid.as_ptr().add(i + 48) as *const x86_64::__m256i);
                    let idx = i + N_HIDDEN * j;
                    let w1 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx) as *const x86_64::__m256i);
                    let w2 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 16) as *const x86_64::__m256i);
                    let w3 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 32) as *const x86_64::__m256i);
                    let w4 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 48) as *const x86_64::__m256i);
                    let idx = i + N_HIDDEN * j + N_HIDDEN;
                    let w5 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx) as *const x86_64::__m256i);
                    let w6 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 16) as *const x86_64::__m256i);
                    let w7 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 32) as *const x86_64::__m256i);
                    let w8 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx + 48) as *const x86_64::__m256i);

                    let sum1234 = x86_64::_mm256_madd_epi16(x1, w1);
                    let sum5678 = x86_64::_mm256_madd_epi16(x2, w2);
                    let sum9abc = x86_64::_mm256_madd_epi16(x3, w3);
                    let sumdefg = x86_64::_mm256_madd_epi16(x4, w4);
                    let sum12342 = x86_64::_mm256_madd_epi16(x1, w5);
                    let sum56782 = x86_64::_mm256_madd_epi16(x2, w6);
                    let sum9abc2 = x86_64::_mm256_madd_epi16(x3, w7);
                    let sumdefg2 = x86_64::_mm256_madd_epi16(x4, w8);

                    let sum18 = x86_64::_mm256_add_epi32(sum1234, sum5678);
                    let sum9g = x86_64::_mm256_add_epi32(sum9abc, sumdefg);
                    let sum182 = x86_64::_mm256_add_epi32(sum12342, sum56782);
                    let sum9g2 = x86_64::_mm256_add_epi32(sum9abc2, sumdefg2);
                    let sum1g = x86_64::_mm256_add_epi32(sum18, sum9g);
                    let sum1g2 = x86_64::_mm256_add_epi32(sum182, sum9g2);

                    sum8 = x86_64::_mm256_add_epi32(sum8, sum1g);
                    sum82 = x86_64::_mm256_add_epi32(sum82, sum1g2);
                }

                let suml = x86_64::_mm256_extracti32x4_epi32(sum8, 1);
                let suml2 = x86_64::_mm256_extracti32x4_epi32(sum82, 1);
                let sumh = x86_64::_mm256_castsi256_si128(sum8);
                let sumh2 = x86_64::_mm256_castsi256_si128(sum82);
                let sum12 = x86_64::_mm_add_epi32(suml, sumh);
                let sum122 = x86_64::_mm_add_epi32(suml2, sumh2);

                x86_64::_mm_storeu_si128(
                        sumhn4.as_mut_ptr() as *mut x86_64::__m128i,
                        sum12);
                x86_64::_mm_storeu_si128(
                        sumhn4.as_mut_ptr().add(4) as *mut x86_64::__m128i,
                        sum122);
            }
            let mut s1 = wdc1[j];
            let mut s2 = wdc1[j + 1];
            for i in 0..4 {
                s1 += sumhn4[i];
                s2 += sumhn4[i + 4];
            }
            sumhn[j] = s1;
            sumhn[j + 1] = s2;
        }

        let wh2 = self.wlayer2i(prgs);
        let mut hid2 = [0i32 ; N_HIDDEN2 / 4];

        if N_HIDDEN2 == 16 {
            for i in (0..N_HIDDEN2).step_by(16) {
                unsafe {
                    let s1 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                    let s2 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);

                    let s1 = x86_64::_mm256_srai_epi32(
                            s1, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s2 = x86_64::_mm256_srai_epi32(
                            s2, CONVERT_TO_FIXPOINT_I16_SHIFT);

                    // i32 -> i16
                    let y12 = x86_64::_mm256_packs_epi32(s1, s2);
                    let x1 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                    // relu
                    let zero = x86_64::_mm256_setzero_si256();
                    let x1 = x86_64::_mm256_max_epi16(x1, zero);

                    let w1 = x86_64::_mm256_loadu_si256(
                        wh2.as_ptr().add(i) as *const x86_64::__m256i);

                    let y1 = x86_64::_mm256_madd_epi16(x1, w1);

                    let y5678 = x86_64::_mm256_extracti128_si256(y1, 1);
                    let y1234 = x86_64::_mm256_castsi256_si128(y1);
                    let y18 = x86_64::_mm_add_epi32(y1234, y5678);

                    x86_64::_mm_storeu_si128(
                        hid2.as_mut_ptr().add(i / 4) as *mut x86_64::__m128i,
                        y18);
                }
            }
        } else {
            for i in (0..N_HIDDEN2).step_by(32) {
                unsafe {
                    let s1 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                    let s2 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);
                    let s3 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 16) as *const x86_64::__m256i);
                    let s4 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 24) as *const x86_64::__m256i);

                    let s1 = x86_64::_mm256_srai_epi32(
                            s1, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s2 = x86_64::_mm256_srai_epi32(
                            s2, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s3 = x86_64::_mm256_srai_epi32(
                            s3, CONVERT_TO_FIXPOINT_I16_SHIFT);
                    let s4 = x86_64::_mm256_srai_epi32(
                            s4, CONVERT_TO_FIXPOINT_I16_SHIFT);

                    // i32 -> i16
                    let y12 = x86_64::_mm256_packs_epi32(s1, s2);
                    let y34 = x86_64::_mm256_packs_epi32(s3, s4);
                    let x1 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                    let x2 = x86_64::_mm256_permute4x64_epi64(y34, 0xD8);
                    // relu
                    let zero = x86_64::_mm256_setzero_si256();
                    let x1 = x86_64::_mm256_max_epi16(x1, zero);
                    let x2 = x86_64::_mm256_max_epi16(x2, zero);

                    let w1 = x86_64::_mm256_loadu_si256(
                        wh2.as_ptr().add(i) as *const x86_64::__m256i);
                    let w2 = x86_64::_mm256_loadu_si256(
                        wh2.as_ptr().add(i + 16) as *const x86_64::__m256i);

                    let y1 = x86_64::_mm256_madd_epi16(x1, w1);
                    let y2 = x86_64::_mm256_madd_epi16(x2, w2);

                    let y1 = x86_64::_mm256_add_epi32(y1, y2);

                    let y5678 = x86_64::_mm256_extracti128_si256(y1, 1);
                    let y1234 = x86_64::_mm256_castsi256_si128(y1);
                    let y18 = x86_64::_mm_add_epi32(y1234, y5678);

                    x86_64::_mm_storeu_si128(
                        hid2.as_mut_ptr().add(i / 8) as *mut x86_64::__m128i,
                        y18);
                }
            }
        }

        // let mut res = self.wl2biasi(prgs) as i32 * CONVERT_TO_FIXPOINT_I16 as i32;
        // for i in 0..N_HIDDEN2 / 4 {
        //     res += hid2[i] as i32;
        // }
        // res as f32 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
        hid2.iter().fold(
                self.wl2biasi(prgs) as i32 * CONVERT_TO_FIXPOINT_I16 as i32,
                      |init, a| init + a) as f32
            / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx_i16_3(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani(prgs);
        let wdc = self.wibiasi32(prgs);
        const N : usize = 32;
        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i) as *const x86_64::__m128i);
                    let w2 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let w3 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i + 16) as *const x86_64::__m128i);
                    let w4 = x86_64::_mm_load_si128(
                        wei.as_ptr().add(i + 24) as *const x86_64::__m128i);

                    let h1 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i) as *const x86_64::__m256i);
                    let h2 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i + 8) as *const x86_64::__m256i);
                    let h3 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let h4 = x86_64::_mm256_loadu_si256(
                        hidi.as_ptr().add(i + 24) as *const x86_64::__m256i);

                    let w1 = x86_64::_mm256_cvtepi16_epi32(w1);
                    let w2 = x86_64::_mm256_cvtepi16_epi32(w2);
                    let w3 = x86_64::_mm256_cvtepi16_epi32(w3);
                    let w4 = x86_64::_mm256_cvtepi16_epi32(w4);

                    let m1 = x86_64::_mm256_add_epi32(w1, h1);
                    let m2 = x86_64::_mm256_add_epi32(w2, h2);
                    let m3 = x86_64::_mm256_add_epi32(w3, h3);
                    let m4 = x86_64::_mm256_add_epi32(w4, h4);

                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i) as *mut x86_64::__m256i, m1);
                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m256i,
                        m2);
                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i + 16) as *mut x86_64::__m256i,
                        m3);
                    x86_64::_mm256_storeu_si256(
                        hidi.as_mut_ptr().add(i + 24) as *mut x86_64::__m256i,
                        m4);
                }
            }
        }

        let mut hid = [0i16 ; N_HIDDEN];
        for hidx in (0..N_HIDDEN).step_by(N * 2) {
            unsafe {
                let h1 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx) as *const x86_64::__m256i);
                let h2 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 8) as *const x86_64::__m256i);
                let h3 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 16) as *const x86_64::__m256i);
                let h4 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 24) as *const x86_64::__m256i);
                let h5 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 32) as *const x86_64::__m256i);
                let h6 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 40) as *const x86_64::__m256i);
                let h7 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 48) as *const x86_64::__m256i);
                let h8 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 56) as *const x86_64::__m256i);

                    // i32 -> i16
                let y12 = x86_64::_mm256_packs_epi32(h1, h2);
                let y34 = x86_64::_mm256_packs_epi32(h3, h4);
                let y56 = x86_64::_mm256_packs_epi32(h5, h6);
                let y78 = x86_64::_mm256_packs_epi32(h7, h8);
                let y12 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                let y34 = x86_64::_mm256_permute4x64_epi64(y34, 0xD8);
                let y56 = x86_64::_mm256_permute4x64_epi64(y56, 0xD8);
                let y78 = x86_64::_mm256_permute4x64_epi64(y78, 0xD8);

                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let y12 = x86_64::_mm256_max_epi16(zero, y12);
                let y34 = x86_64::_mm256_max_epi16(zero, y34);
                let y56 = x86_64::_mm256_max_epi16(zero, y56);
                let y78 = x86_64::_mm256_max_epi16(zero, y78);

                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx) as *mut x86_64::__m256i, y12);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 16) as *mut x86_64::__m256i,
                    y34);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 32) as *mut x86_64::__m256i,
                    y56);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 48) as *mut x86_64::__m256i,
                    y78);
            }
        }

        // 1st layer to 2nd layer
        let wh = self.wlayer1iv(prgs);
        let wdc1 = self.wl1biasi32(prgs);
        let mut sumhn = [0i32 ; N_HIDDEN2];
        sumhn.copy_from_slice(wdc1);
        // for j in (0..N_HIDDEN).step_by(2) {
        for j in 0..N_HIDDEN {
            if hid[j] == 0 {continue;}

            unsafe {
                let x1 = x86_64::_mm256_set1_epi16(hid[j]);
                // let x2 = x86_64::_mm256_set1_epi16(hid[j + 1]);
                for i in (0..N_HIDDEN2).step_by(16) {
                    let idx = i + N_HIDDEN2 * j;
                    let w1 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx) as *const x86_64::__m256i);
                    // let w2 = x86_64::_mm256_loadu_si256(
                    //     wh.as_ptr().add(idx + N_HIDDEN2) as *const x86_64::__m256i);

                    let mul1l = x86_64::_mm256_mullo_epi16(x1, w1);
                    // let mul2l = x86_64::_mm256_mullo_epi16(x2, w2);
                    let mul1h = x86_64::_mm256_mulhi_epi16(x1, w1);
                    // let mul2h = x86_64::_mm256_mulhi_epi16(x2, w2);

                    let mul13 = x86_64::_mm256_unpacklo_epi16(mul1l, mul1h);
                    // let mul57 = x86_64::_mm256_unpacklo_epi16(mul2l, mul2h);
                    let mul24 = x86_64::_mm256_unpackhi_epi16(mul1l, mul1h);
                    // let mul68 = x86_64::_mm256_unpackhi_epi16(mul2l, mul2h);
                    let mul12 = x86_64::_mm256_permute2x128_si256(
                        mul13, mul24, 0b00100000);
                    // let mul56 = x86_64::_mm256_permute2x128_si256(
                    //     mul57, mul68, 0b00100000);
                    let mul34 = x86_64::_mm256_permute2x128_si256(
                        mul13, mul24, 0b00110001);
                    // let mul78 = x86_64::_mm256_permute2x128_si256(
                    //     mul57, mul68, 0b00110001);

                    // let sum12 = x86_64::_mm256_add_epi32(mul12, mul56);
                    // let sum34 = x86_64::_mm256_add_epi32(mul34, mul78);

                    let s1 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                    let s3 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);

                    let s1 = x86_64::_mm256_add_epi32(s1, mul12);
                    let s3 = x86_64::_mm256_add_epi32(s3, mul34);
                    // let s1 = x86_64::_mm256_add_epi32(s1, sum12);
                    // let s3 = x86_64::_mm256_add_epi32(s3, sum34);

                    x86_64::_mm256_storeu_si256(
                        sumhn.as_mut_ptr().add(i) as *mut x86_64::__m256i, s1);
                    x86_64::_mm256_storeu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *mut x86_64::__m256i, s3);
                }
            }
        }

        let wh2 = self.wlayer2i(prgs);
        let mut hid2 = [0i32 ; N_HIDDEN2];
        for i in (0..N_HIDDEN2).step_by(16) {
            unsafe {
                let s1 = x86_64::_mm256_loadu_si256(
                    sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                let s2 = x86_64::_mm256_loadu_si256(
                    sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);

                let s1 = x86_64::_mm256_srai_epi32(
                        s1, CONVERT_TO_FIXPOINT_I16_SHIFT);
                let s2 = x86_64::_mm256_srai_epi32(
                        s2, CONVERT_TO_FIXPOINT_I16_SHIFT);

                // i32 -> i16
                let y12 = x86_64::_mm256_packs_epi32(s1, s2);
                let x1 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let x1 = x86_64::_mm256_max_epi16(x1, zero);

                let w1 = x86_64::_mm256_loadu_si256(
                    wh2.as_ptr().add(i) as *const x86_64::__m256i);

                let y1 = x86_64::_mm256_madd_epi16(x1, w1);

                let y5678 = x86_64::_mm256_extracti128_si256(y1, 1);
                let y1234 = x86_64::_mm256_castsi256_si128(y1);
                let y18 = x86_64::_mm_add_epi32(y1234, y5678);

                x86_64::_mm_storeu_si128(
                    hid2.as_mut_ptr().add(i / 4) as *mut x86_64::__m128i,
                    y18);
            }
        }

        let mut res = self.wl2biasi(prgs) as i32 * CONVERT_TO_FIXPOINT_I16 as i32;
        for i in 0..N_HIDDEN2 / 4 {
            res += hid2[i] as i32;
        }
        res as f32 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx_i16_2(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani(prgs);
        let wdc = self.wibiasi32(prgs);
        const N : usize = 32;
        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_si128(wei.as_ptr().add(i) as *const x86_64::__m128i);
                    let w2 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let w3 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 16) as *const x86_64::__m128i);
                    let w4 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 24) as *const x86_64::__m128i);

                    let h1 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i) as *const x86_64::__m256i);
                    let h2 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i + 8) as *const x86_64::__m256i);
                    let h3 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let h4 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i + 24) as *const x86_64::__m256i);

                    let w1 = x86_64::_mm256_cvtepi16_epi32(w1);
                    let w2 = x86_64::_mm256_cvtepi16_epi32(w2);
                    let w3 = x86_64::_mm256_cvtepi16_epi32(w3);
                    let w4 = x86_64::_mm256_cvtepi16_epi32(w4);

                    let m1 = x86_64::_mm256_add_epi32(w1, h1);
                    let m2 = x86_64::_mm256_add_epi32(w2, h2);
                    let m3 = x86_64::_mm256_add_epi32(w3, h3);
                    let m4 = x86_64::_mm256_add_epi32(w4, h4);

                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i) as *mut x86_64::__m256i, m1);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m256i, m2);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i + 16) as *mut x86_64::__m256i, m3);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i + 24) as *mut x86_64::__m256i, m4);
                }
            }
        }

        let mut hid = [0i16 ; N_HIDDEN];
        for hidx in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx) as *const x86_64::__m256i);
                let h2 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 8) as *const x86_64::__m256i);
                let h3 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 16) as *const x86_64::__m256i);
                let h4 = x86_64::_mm256_loadu_si256(
                    hidi.as_ptr().add(hidx + 24) as *const x86_64::__m256i);
                // wdc
                // let wdc1 = x86_64::_mm_load_si128(
                //     wdc.as_ptr().add(hidx) as *const x86_64::__m128i);
                // let wdc2 = x86_64::_mm_load_si128(
                //     wdc.as_ptr().add(hidx + 8) as *const x86_64::__m128i);
                // let wdc3 = x86_64::_mm_load_si128(
                //     wdc.as_ptr().add(hidx + 16) as *const x86_64::__m128i);
                // let wdc4 = x86_64::_mm_load_si128(
                //     wdc.as_ptr().add(hidx + 24) as *const x86_64::__m128i);
                // let wdc1 = x86_64::_mm256_cvtepi16_epi32(wdc1);
                // let wdc2 = x86_64::_mm256_cvtepi16_epi32(wdc2);
                // let wdc3 = x86_64::_mm256_cvtepi16_epi32(wdc3);
                // let wdc4 = x86_64::_mm256_cvtepi16_epi32(wdc4);
                // let h1 = x86_64::_mm256_add_epi32(h1, wdc1);
                // let h2 = x86_64::_mm256_add_epi32(h2, wdc2);
                // let h3 = x86_64::_mm256_add_epi32(h3, wdc3);
                // let h4 = x86_64::_mm256_add_epi32(h4, wdc4);

                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let y1 = x86_64::_mm256_max_epi32(zero, h1);
                let y2 = x86_64::_mm256_max_epi32(zero, h2);
                let y3 = x86_64::_mm256_max_epi32(zero, h3);
                let y4 = x86_64::_mm256_max_epi32(zero, h4);

                // i32 -> i16
                let y12 = x86_64::_mm256_packs_epi32(y1, y2);
                let y34 = x86_64::_mm256_packs_epi32(y3, y4);
                let y12 = x86_64::_mm256_permute4x64_epi64(y12, 0xD8);
                let y34 = x86_64::_mm256_permute4x64_epi64(y34, 0xD8);

                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx) as *mut x86_64::__m256i, y12);
                x86_64::_mm256_storeu_si256(
                    hid.as_mut_ptr().add(hidx + 16) as *mut x86_64::__m256i, y34);
            }
        }

        // 1st layer to 2nd layer
        let wh = self.wlayer1iv(prgs);
        let wdc1 = self.wl1biasi32(prgs);
        let mut sumhn = [0i32 ; N_HIDDEN2];
        sumhn.copy_from_slice(wdc1);
        for j in 0..N_HIDDEN {
            unsafe {
                let x1 = x86_64::_mm256_set1_epi16(hid[j]);
                for i in (0..N_HIDDEN2).step_by(16) {
                    let idx = i + N_HIDDEN2 * j;
                    let w1 = x86_64::_mm256_loadu_si256(
                        wh.as_ptr().add(idx) as *const x86_64::__m256i);

                    let mul1l = x86_64::_mm256_mullo_epi16(x1, w1);
                    let mul1h = x86_64::_mm256_mulhi_epi16(x1, w1);
                    let mul13 = x86_64::_mm256_unpacklo_epi16(mul1l, mul1h);
                    let mul24 = x86_64::_mm256_unpackhi_epi16(mul1l, mul1h);
                    let mul12 = x86_64::_mm256_permute2x128_si256(mul13, mul24, 0b00100000);
                    let mul34 = x86_64::_mm256_permute2x128_si256(mul13, mul24, 0b00110001);

                    let s1 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                    let s3 = x86_64::_mm256_loadu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);

                    let s1 = x86_64::_mm256_add_epi32(s1, mul12);
                    let s3 = x86_64::_mm256_add_epi32(s3, mul34);

                    x86_64::_mm256_storeu_si256(
                        sumhn.as_mut_ptr().add(i) as *mut x86_64::__m256i, s1);
                    x86_64::_mm256_storeu_si256(
                        sumhn.as_mut_ptr().add(i + 8) as *mut x86_64::__m256i, s3);
                }
            }
        }

        // let wdc1 = self.wl1biasi32(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        for i in (0..N_HIDDEN2).step_by(16) {
            unsafe {
                // dc
                // let dc1 = x86_64::_mm256_load_si256(
                //     wdc1.as_ptr().add(i) as *const x86_64::__m256i);
                // let dc2 = x86_64::_mm256_load_si256(
                //     wdc1.as_ptr().add(i + 8) as *const x86_64::__m256i);

                let s1 = x86_64::_mm256_loadu_si256(
                    sumhn.as_mut_ptr().add(i) as *const x86_64::__m256i);
                let s2 = x86_64::_mm256_loadu_si256(
                    sumhn.as_mut_ptr().add(i + 8) as *const x86_64::__m256i);
                // let s1 = x86_64::_mm256_add_epi32(s1, dc1);
                // let s2 = x86_64::_mm256_add_epi32(s2, dc2);

                // let rounder = x86_64::_mm256_set1_epi32(
                //     (CONVERT_TO_FIXPOINT_I16 * 0.5) as i32);
                // let s1 = x86_64::_mm256_add_epi32(s1, rounder);
                // let s2 = x86_64::_mm256_add_epi32(s2, rounder);

                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let r1 = x86_64::_mm256_max_epi32(s1, zero);
                let r2 = x86_64::_mm256_max_epi32(s2, zero);

                let x1 = x86_64::_mm256_cvtepi32_ps(r1);
                let x2 = x86_64::_mm256_cvtepi32_ps(r2);
                let mag = x86_64::_mm256_set1_ps(
                    1.0 / CONVERT_TO_FIXPOINT_I16 / CONVERT_TO_FIXPOINT_I16);
                let x1 = x86_64::_mm256_mul_ps(x1, mag);
                let x2 = x86_64::_mm256_mul_ps(x2, mag);

                let wh21 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i));
                let wh22 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 8));

                let y1 = x86_64::_mm256_mul_ps(wh21, x1);
                let y2 = x86_64::_mm256_mul_ps(wh22, x2);

                let y12 = x86_64::_mm256_add_ps(y1, y2);
                let y5678 = x86_64::_mm256_extractf128_ps(y12, 1);
                let y1234 = x86_64::_mm256_castps256_ps128(y12);
                let y18 = x86_64::_mm_add_ps(y1234, y5678);

                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i / 4), y18);
            }
        }

        let mut res = self.wl2bias(prgs);
        for i in 0..N_HIDDEN2 / 4 {
            res += hid2[i];
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev12bb_simdavx_i16_1(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;

        let ow = self.wbani(prgs);
        let wdc = self.wibiasi32(prgs);
        const N : usize = 32;
        let mut hidi = [0i32 ; N_HIDDEN];
        hidi.copy_from_slice(wdc);
        let mut bit = bitboard::LSB_CELL;
        for idx in 0..bitboard::CELL_2D {
            let b = black & bit;
            let w = white & bit;
            bit <<= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w1 = x86_64::_mm_load_si128(wei.as_ptr().add(i) as *const x86_64::__m128i);
                    let w2 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 8) as *const x86_64::__m128i);
                    let w3 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 16) as *const x86_64::__m128i);
                    let w4 = x86_64::_mm_load_si128(wei.as_ptr().add(i + 24) as *const x86_64::__m128i);
                    // let w1 = x86_64::_mm256_load_si256(wei.as_ptr().add(i) as *const x86_64::__m256i);
                    // let w2 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 8));
                    // let w3 = x86_64::_mm256_load_si256(wei.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    // let w4 = x86_64::_mm256_load_ps(wei.as_ptr().add(i + 24));
                    let h1 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i) as *const x86_64::__m256i);
                    let h2 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i + 8) as *const x86_64::__m256i);
                    let h3 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i + 16) as *const x86_64::__m256i);
                    let h4 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(i + 24) as *const x86_64::__m256i);
                    // let zero = x86_64::_mm256_setzero_si256();
                    // let minus1 = x86_64::_mm256_cmpgt_epi16(zero, w1);
                    // let minus3 = x86_64::_mm256_cmpgt_epi16(zero, w3);
                    // let w2 = x86_64::_mm256_unpackhi_epi16(minus1, w1);
                    // let w1 = x86_64::_mm256_unpacklo_epi16(minus1, w1);
                    // let w4 = x86_64::_mm256_unpackhi_epi16(minus3, w3);
                    // let w3 = x86_64::_mm256_unpacklo_epi16(minus3, w3);
                    let w1 = x86_64::_mm256_cvtepi16_epi32(w1);
                    let w2 = x86_64::_mm256_cvtepi16_epi32(w2);
                    let w3 = x86_64::_mm256_cvtepi16_epi32(w3);
                    let w4 = x86_64::_mm256_cvtepi16_epi32(w4);
                    let m1 = x86_64::_mm256_add_epi32(w1, h1);
                    let m2 = x86_64::_mm256_add_epi32(w2, h2);
                    let m3 = x86_64::_mm256_add_epi32(w3, h3);
                    let m4 = x86_64::_mm256_add_epi32(w4, h4);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i) as *mut x86_64::__m256i, m1);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i + 8) as *mut x86_64::__m256i, m2);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i + 16) as *mut x86_64::__m256i, m3);
                    x86_64::_mm256_storeu_si256(hidi.as_mut_ptr().add(i + 24) as *mut x86_64::__m256i, m4);
                }
            }
        }

        let mut hid = [0f32 ; N_HIDDEN];
        for hidx in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(hidx) as *const x86_64::__m256i);
                let h2 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(hidx + 8) as *const x86_64::__m256i);
                let h3 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(hidx + 16) as *const x86_64::__m256i);
                let h4 = x86_64::_mm256_loadu_si256(hidi.as_ptr().add(hidx + 24) as *const x86_64::__m256i);
                // wdc
                // let wdc1 = x86_64::_mm_load_si128(wdc.as_ptr().add(hidx) as *const x86_64::__m128i);
                // let wdc2 = x86_64::_mm_load_si128(wdc.as_ptr().add(hidx + 8) as *const x86_64::__m128i);
                // let wdc3 = x86_64::_mm_load_si128(wdc.as_ptr().add(hidx + 16) as *const x86_64::__m128i);
                // let wdc4 = x86_64::_mm_load_si128(wdc.as_ptr().add(hidx + 24) as *const x86_64::__m128i);
                // let wdc1 = x86_64::_mm256_cvtepi16_epi32(wdc1);
                // let wdc2 = x86_64::_mm256_cvtepi16_epi32(wdc2);
                // let wdc3 = x86_64::_mm256_cvtepi16_epi32(wdc3);
                // let wdc4 = x86_64::_mm256_cvtepi16_epi32(wdc4);
                // let h1 = x86_64::_mm256_add_epi32(h1, wdc1);
                // let h2 = x86_64::_mm256_add_epi32(h2, wdc2);
                // let h3 = x86_64::_mm256_add_epi32(h3, wdc3);
                // let h4 = x86_64::_mm256_add_epi32(h4, wdc4);

                // relu
                let zero = x86_64::_mm256_setzero_si256();
                let y1 = x86_64::_mm256_max_epi32(zero, h1);
                let y2 = x86_64::_mm256_max_epi32(zero, h2);
                let y3 = x86_64::_mm256_max_epi32(zero, h3);
                let y4 = x86_64::_mm256_max_epi32(zero, h4);

                let y1 = x86_64::_mm256_cvtepi32_ps(y1);
                let y2 = x86_64::_mm256_cvtepi32_ps(y2);
                let y3 = x86_64::_mm256_cvtepi32_ps(y3);
                let y4 = x86_64::_mm256_cvtepi32_ps(y4);
                let mag = x86_64::_mm256_set1_ps(1.0 / CONVERT_TO_FIXPOINT_I16);
                let y1 = x86_64::_mm256_mul_ps(mag, y1);
                let y2 = x86_64::_mm256_mul_ps(mag, y2);
                let y3 = x86_64::_mm256_mul_ps(mag, y3);
                let y4 = x86_64::_mm256_mul_ps(mag, y4);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx), y1);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 8), y2);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 16), y3);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 24), y4);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);

        let mut hid2 = [0f32 ; N_HIDDEN2];
        hid2.copy_from_slice(wdc1);
        let mut sumhn = [0f32 ; N_HIDDEN2 * 4 * 2];
        for j in (0..N_HIDDEN).step_by(32) {
            unsafe {
                let x1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j));
                let x2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 8));
                let x3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 16));
                let x4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 24));
                for i in 0..N_HIDDEN2 {
                    let idx = i * N_HIDDEN + j;
                    let w1 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 8));
                    let w3 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 16));
                    let w4 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 24));
                    let mul1 = x86_64::_mm256_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm256_mul_ps(x2, w2);
                    // let mul3 = x86_64::_mm256_mul_ps(x3, w3);
                    // let mul4 = x86_64::_mm256_mul_ps(x4, w4);
                    // let s12 = x86_64::_mm256_add_ps(mul1, mul2);
                    // let s34 = x86_64::_mm256_add_ps(mul3, mul4);
                    let s12 = x86_64::_mm256_fmadd_ps(x3, w3, mul1);
                    let s34 = x86_64::_mm256_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm256_add_ps(s12, s34);
                    x86_64::_mm256_storeu_ps(
                            sumhn.as_mut_ptr().add(i * 8), s1234);
                }
                for (k, _hn) in sumhn.iter().enumerate().step_by(32) {
                    let a = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k));  // a0~a7
                    let b = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 8));  // a8~a15
                    let c = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 16));  // b0~b7
                    let d = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 24));  // b8~b15
                    let a0c0 = x86_64::_mm256_unpacklo_ps(a, b);
                    let b0d0 = x86_64::_mm256_unpacklo_ps(c, d);
                    let a2c2 = x86_64::_mm256_unpackhi_ps(a, b);
                    let b2d2 = x86_64::_mm256_unpackhi_ps(c, d);
                    let s1 = x86_64::_mm256_add_ps(a0c0, a2c2);
                    let s2 = x86_64::_mm256_add_ps(b0d0, b2d2);
                    let t1 = x86_64::_mm256_shuffle_ps(s1, s2,
                            0b01000100/*(1 << 6) | (0 << 4) | (1 << 2) | 0*/);
                    let t2 = x86_64::_mm256_shuffle_ps(s1, s2,
                            0b11101110/*(3 << 6) | (2 << 4) | (3 << 2) | 2*/);
                    let s3 = x86_64::_mm256_add_ps(t1, t2);
                    let s4 = x86_64::_mm256_extractf128_ps(s3, 1);
                    let s5 = x86_64::_mm_add_ps(
                            s4, x86_64::_mm256_castps256_ps128(s3));
                    let hn2 = x86_64::_mm_loadu_ps(
                            hid2.as_mut_ptr().add(k / 8));
                    let s6 = x86_64::_mm_add_ps(s5, hn2);
                    x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(k / 8), s6);
                }
            }
        }
        if N_HIDDEN2 >= 32 {
            for i in (0..N_HIDDEN2).step_by(32) {
                unsafe {  // relu
                    let x1 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i));
                    let x2 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 8));
                    let x3 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 16));
                    let x4 = x86_64::_mm256_load_ps(hid2.as_ptr().add(i + 24));
                    let zero = x86_64::_mm256_setzero_ps();
                    let h1 = x86_64::_mm256_max_ps(zero, x1);
                    let h2 = x86_64::_mm256_max_ps(zero, x2);
                    let h3 = x86_64::_mm256_max_ps(zero, x3);
                    let h4 = x86_64::_mm256_max_ps(zero, x4);
                    let w1 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i));
                    let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 8));
                    let w3 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 16));
                    let w4 = x86_64::_mm256_load_ps(wh2.as_ptr().add(i + 24));
                    let y1 = x86_64::_mm256_mul_ps(h1, w1);
                    let y2 = x86_64::_mm256_mul_ps(h2, w2);
                    let y3 = x86_64::_mm256_mul_ps(h3, w3);
                    let y4 = x86_64::_mm256_mul_ps(h4, w4);
                    let y12 = x86_64::_mm256_add_ps(y1, y2);
                    let y34 = x86_64::_mm256_add_ps(y3, y4);
                    let y1234 = x86_64::_mm256_add_ps(y12, y34);
                    let s1 = x86_64::_mm256_castps256_ps128(y1234);
                    let s2 = x86_64::_mm256_extractf128_ps(y1234, 1);
                    let s4 = x86_64::_mm_add_ps(s1, s2);
                    x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i / 8), s4);
                }
            }
            for h in hid2.iter().take(N_HIDDEN2 / 8) {
                res += h;
            }
        } else {
            unsafe {  // relu
                let x1 = x86_64::_mm256_loadu_ps(hid2.as_ptr());
                let x2 = x86_64::_mm256_loadu_ps(hid2.as_ptr().add(8));
                let zero = x86_64::_mm256_setzero_ps();
                let h1 = x86_64::_mm256_max_ps(zero, x1);
                let h2 = x86_64::_mm256_max_ps(zero, x2);
                let w1 = x86_64::_mm256_load_ps(wh2.as_ptr());
                let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(8));
                let y1 = x86_64::_mm256_mul_ps(h1, w1);
                let y2 = x86_64::_mm256_mul_ps(h2, w2);
                let y3 = x86_64::_mm256_add_ps(y1, y2);
                let s1 = x86_64::_mm256_castps256_ps128(y3);
                let s2 = x86_64::_mm256_extractf128_ps(y3, 1);
                let s4 = x86_64::_mm_add_ps(s1, s2);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr(), s4);
            }
            for h in hid2.iter().take(4) {
                res += h;
            }
         }
         res
    }

    #[inline]
    // returns bit pattern for shuffle.
    pub const fn mm_shuffle(fp3: i32, fp2: i32, fp1: i32, fp0: i32) -> i32 {
        (fp3 << 6) | (fp2 << 4) | (fp1 << 2) | fp0
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

#[allow(dead_code)]
fn dbg_assert_eql(a : &f32, b : &f32) -> bool {
    if (a - b).abs() >= 2e-2 {
        println!("| {a} - {b} | >= 2e-2...");
        return false;
    }
    true
}

#[allow(dead_code)]
fn dbg_assert_eqi(a : &f32, b : &f32) -> bool {
    // let eps = 0.1;
    let eps = 4.0;
    if (a - b).abs() >= eps {
        println!("| {a} - {b} | >= {eps}...");
        return false;
    }
    true
}

#[test]
fn test_exchange_weight() {
        let mut w = weight::Weight::new();
        w.init();
        for p in 0..N_PROGRESS_DIV {
            let wei = w.wban(p);
            let vwei = w.wbanv(p);
            for i in 0..N_WEIGHT_INPUTBIAS {
                assert_ne!(wei[i], 0.0f32);
                assert_ne!(vwei[i], 0.0f32);

                let wsz = N_HIDDEN * 2;
                let grp = i % wsz;
                let hidx = grp % N_HIDDEN;
                let bw = grp / N_HIDDEN;
                let xy = i / wsz;
                let j= xy + bw * bitboard::CELL_2D + hidx * bitboard::CELL_2D * 2;
                assert_eq!(vwei[i], wei[j]);
            }
        }
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
    for &rfen in rfens.iter() {
        let bban = bitboard::BitBoard::try_from(rfen).unwrap();
        let ban = bitboard::BitBoard::try_from(rfen).unwrap();
        ban.put();
        eprintln!("- - -");
        let mut w = weight::Weight::new();
        eprintln!("--------");
        w.init();
        eprintln!("--------~~~~~~~~~~~~~");
        let res_nosimde = w.evaluatev12bb(&bban);
        eprintln!("--------~~~~~~~~~~~~~***********");
        let res_simd = w.evaluatev12bb_simd(&bban);
        let res_simdavx = w.evaluatev12bb_simdavx(&bban);
        let res_simdavx2 = w.evaluatev12bb_simdavx_2(&bban);
        assert!(dbg_assert_eq(&res_nosimde, &res_simd));
        assert!(dbg_assert_eq(&res_nosimde, &res_simdavx));
        assert!(dbg_assert_eq(&res_nosimde, &res_simdavx2));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}

#[cfg(target_arch="x86_64")]
#[test]
fn testweight_f32_i32() {
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
    let mut w = weight::Weight::new();
    w.init();
    w.read("data/evaltable.txt").unwrap();
    for &rfen in rfens.iter() {
        let ban = bitboard::BitBoard::try_from(rfen).unwrap();
        ban.put();
        let res_nosimde = w.evaluatev12bb(&ban);
        eprintln!("- - -");
        let res_nosimde_i16 = w.evaluatev12bb_i16(&ban);
        eprintln!("--------");
        let res_nosimde_i161 = w.evaluatev12bb_i16_1(&ban);
        eprintln!("--------~~~~~~~~~~~~~");
        let res_nosimde_i162 = w.evaluatev12bb_i16_2(&ban);
        eprintln!("--------~~~~~~~~~~~~~***********");
        assert!(dbg_assert_eqi(&res_nosimde, &res_nosimde_i162));
        assert!(dbg_assert_eqi(&res_nosimde, &res_nosimde_i161));
        assert!(dbg_assert_eqi(&res_nosimde, &res_nosimde_i16));
    }
}

#[cfg(target_arch="x86_64")]
#[test]
fn testweight_i32_sse() {
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
    let mut w = weight::Weight::new();
    w.init();
    w.read("data/evaltable.txt").unwrap();
    for &rfen in rfens.iter() {
        let ban = bitboard::BitBoard::try_from(rfen).unwrap();
        ban.put();
        eprintln!("- - -");
        eprintln!("--------");
        let res_nosimde_i16 = w.evaluatev12bb_i16(&ban);
        let res_nosimde_i161 = w.evaluatev12bb_i16_1(&ban);
        let res_nosimde_i162 = w.evaluatev12bb_i16_2(&ban);
        eprintln!("--------~~~~~~~~~~~~~");
        let res_simd = w.evaluatev12bb_simd_i16(&ban);
        let res_simd1 = w.evaluatev12bb_simd_i16_1(&ban);
        let res_simd2 = w.evaluatev12bb_simd_i16_2(&ban);
        eprintln!("--------~~~~~~~~~~~~~***********");
        assert!(dbg_assert_eql(&res_nosimde_i162, &res_simd2));
        assert!(dbg_assert_eql(&res_nosimde_i161, &res_simd1));
        assert!(dbg_assert_eqi(&res_nosimde_i16, &res_simd));
        // assert!(dbg_assert_eql(&res_nosimde_i162, &res_simdavx2));
    }
}

#[cfg(target_arch="x86_64")]
#[test]
fn testweight_i32_avx() {
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
    let mut w = weight::Weight::new();
    w.init();
    w.read("data/evaltable.txt").unwrap();
    for &rfen in rfens.iter() {
        let ban = bitboard::BitBoard::try_from(rfen).unwrap();
        ban.put();
        eprintln!("- - -");
        eprintln!("--------");
        let res_nosimde_i16 = w.evaluatev12bb_i16(&ban);
        let res_nosimde_i161 = w.evaluatev12bb_i16_1(&ban);
        let res_nosimde_i162 = w.evaluatev12bb_i16_2(&ban);
        eprintln!("--------~~~~~~~~~~~~~");
        eprintln!("--------~~~~~~~~~~~~~***********");
        let res_simdavx = w.evaluatev12bb_simdavx_i16(&ban);
        let res_simdavx1 = w.evaluatev12bb_simdavx_i16_1(&ban);
        let res_simdavx2 = w.evaluatev12bb_simdavx_i16_2(&ban);
        let res_simdavx3 = w.evaluatev12bb_simdavx_i16_3(&ban);
        // assert!(dbg_assert_eql(&res_nosimde_i162, &res_simdavx2));
        assert!(dbg_assert_eql(&res_nosimde_i16, &res_simdavx3));
        assert!(dbg_assert_eql(&res_nosimde_i162, &res_simdavx2));
        assert!(dbg_assert_eql(&res_nosimde_i161, &res_simdavx1));
        assert!(dbg_assert_eqi(&res_nosimde_i16, &res_simdavx));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
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
    for &rfen in rfens.iter() {
        let bban = bitboard::BitBoard::try_from(rfen).unwrap();
        bban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimdi = w.evaluatev12bb(&bban);
        let res_simdmul = w.evaluatev12bb_simd_mul(&bban);
        // let res_simd = w.evaluatev9bb_simd(&bban);
        assert!(dbg_assert_eq(&res_nosimdi, &res_simdmul));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}

#[cfg(target_arch="aarch64")]
#[test]
fn testweight_i32() {
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
    for &rfen in rfens.iter() {
        let bban = bitboard::BitBoard::try_from(rfen).unwrap();
        bban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimdi = w.evaluatev12bb(&bban);
        let res_nosimdi_i16 = w.evaluatev12bb_i16(&bban);
        let res_simdmul_i16 = w.evaluatev12bb_simd_mul_i16(&bban);
        // let res_simd = w.evaluatev9bb_simd(&bban);
        assert!(dbg_assert_eql(&res_nosimdi, &res_nosimd_i16));
        assert!(dbg_assert_eql(&res_nosimdi_i16, &res_simdmul_i16));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}
