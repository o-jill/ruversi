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
const N_INPUT : usize = bitboard::CELL_2D * 2 + 1 + 2;
const N_HIDDEN : usize = 128;
pub const N_HIDDEN2 : usize = 16;
const N_OUTPUT : usize = 1;
const N_WEIGHT_TEBAN : usize =  bitboard::CELL_2D * N_HIDDEN;
const N_WEIGHT_FIXST_B : usize = N_WEIGHT_TEBAN + N_HIDDEN;
const N_WEIGHT_FIXST_W : usize = N_WEIGHT_FIXST_B + N_HIDDEN;
const N_WEIGHT_INPUTBIAS : usize = N_WEIGHT_FIXST_W + N_HIDDEN;
const N_WEIGHT_LAYER1 : usize = N_WEIGHT_INPUTBIAS + N_HIDDEN;
const N_WEIGHT_LAYER1BIAS : usize = N_WEIGHT_LAYER1 + N_HIDDEN * N_HIDDEN2;
const N_WEIGHT_LAYER2 : usize = N_WEIGHT_LAYER1BIAS + N_HIDDEN2;
const N_WEIGHT_LAYER2BIAS : usize = N_WEIGHT_LAYER2 + N_HIDDEN2;
const N_WEIGHT : usize =
  (N_INPUT + 1) * N_HIDDEN + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

const N_WEIGHT_PAD :usize = N_WEIGHT.div_ceil(8) * 8;
pub const N_PROGRESS_DIV : usize = 3;  // 序盤中盤終盤

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
const WSZV10 : usize = (bitboard::CELL_2D * 2 + 1 + 2 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

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
    V7,
    V8,
    V9,
    V10,
}

impl EvalFile {
    #[allow(dead_code)]
    pub fn to_str(&self) -> &str {
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
            "# 64+1+2-32-16-1" => Some(EvalFile::V7),
            "# 64+1+2-128-16-1" => Some(EvalFile::V8),
            "# 3x 64+1+2-128-16-1" => Some(EvalFile::V9),
            "# 3x 128+1+2-128-16-1" => Some(EvalFile::V10),
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
    vweight : AVec<f32>
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
                let mut w =
                    AVec::with_capacity(
                        MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                unsafe {w.set_len(w.capacity());}
                w
            },
            vweight: AVec::with_capacity(
                MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV)
        }
    }

    fn exchange(&mut self) {
        unsafe {self.vweight.set_len(self.weight.len());}
        for p in 0..N_PROGRESS_DIV {
            let mut check = [0i8 ; bitboard::CELL_2D * 2 * N_HIDDEN];
            let offset = p * N_WEIGHT_PAD;
            let wei = &self.weight[offset..offset + N_WEIGHT_PAD];
            let vwei = &mut self.vweight[offset..offset + N_WEIGHT_PAD];
            for (i, &w) in wei.iter().enumerate().take(bitboard::CELL_2D * 2 * N_HIDDEN) {
                let hidx = i / (bitboard::CELL_2D * 2);

                let x = i % (bitboard::CELL_2D * 2);  // b: 0~63, w:64~127
                let bw = x / bitboard::CELL_2D;  // 0:b, 1:w
                let x = x % bitboard::CELL_2D;

                let idx = hidx + x * N_HIDDEN * 2 + bw * N_HIDDEN;
                vwei[idx] = w;
                check[idx] = 1;
            }
            for (i, &c) in check.iter().enumerate() {
                if c == 0 {
                    panic!("check error @ {i}!");
                }
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

    pub fn wban(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset..]
    }

    pub fn wbanv(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.vweight[offset..]
    }

    pub fn wteban(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_TEBAN..offset + N_WEIGHT_FIXST_W]
    }

    pub fn wfixedstones(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_FIXST_B..offset + N_WEIGHT_INPUTBIAS]
    }

    #[allow(dead_code)]
    pub fn wfixedstone_b(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_FIXST_B..offset + N_WEIGHT_FIXST_W]
    }

    #[allow(dead_code)]
    pub fn wfixedstone_w(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_FIXST_W..offset + N_WEIGHT_INPUTBIAS]
    }

    pub fn wibias(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_INPUTBIAS..offset + N_WEIGHT_LAYER1]
    }

    pub fn wlayer1(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1BIAS]
    }

    pub fn wl1bias(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER1BIAS..offset + N_WEIGHT_LAYER2]
    }

    pub fn wlayer2(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER2..offset + N_WEIGHT_LAYER2BIAS]
    }

    pub fn wl2bias(&self, progress : usize) -> f32 {
        let offset = progress * N_WEIGHT_PAD;
        self.weight[offset + N_WEIGHT - 1]
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
                        if format != EvalFile::Unknown {continue;}

                        if let Some(fmt) = EvalFile::from(&l) {
                            format = fmt
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
                            if idx >= N_PROGRESS_DIV {return Ok(());}
                        },
                        EvalFile::V10 => {
                            self.readv10(&l, idx)?;
                            idx += 1;
                            if idx >= N_PROGRESS_DIV {return Ok(());}
                        },
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
        self.weight[offset..offset + N_WEIGHT].copy_from_slice(&newtable);
        self.exchange();
        // println!("v9:{:?}", self.weight);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn writev10(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(
            format!("{}\n", EvalFile::V10.to_str()).as_bytes()).unwrap();
        for prgs in 0..N_PROGRESS_DIV {
            let offset = prgs * N_WEIGHT_PAD;
            let w = &self.weight[offset..offset + N_WEIGHT];
            let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            f.write_all((sv.join(",") + "¥n").as_bytes()).unwrap();
        }
    }

    pub fn copy(&mut self, src : &Weight) {
        self.weight.copy_from_slice(&src.weight);
        if self.vweight.is_empty() {
            unsafe {self.vweight.set_len(self.vweight.capacity());}
        }
        self.vweight.copy_from_slice(&src.vweight);
    }

    pub fn evaluatev9bb(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wbanv(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        let mut black = ban.black;
        let mut white = ban.white;
        for idx in 0..bitboard::CELL_2D {
            let bit = bitboard::LSB_CELL;
            let b = black & bit;
            let w = white & bit;
            black >>= 1;
            white >>= 1;
            if b | w == 0 {continue;}  // no stone

            let start = idx * N_HIDDEN * 2 + if b != 0 {0} else {N_HIDDEN};
            for (h, w) in hid.iter_mut().zip(ow.iter().skip(start)) {
                *h += w;
            }
        }
        for (i, h) in hid.iter_mut().enumerate() {
            let mut hidsum = teban.mul_add(wtbn[i], *h);
            hidsum = wfs[i].mul_add(fs.0 as f32, hidsum);
            hidsum = wfs[i + N_HIDDEN].mul_add(fs.1 as f32, hidsum);
            // relu
            *h = hidsum.max(0f32);
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

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev9bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let mut black = ban.black;
        let mut white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wbanv(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);

        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        const N : usize = 16;
        for idx in 0..bitboard::CELL_2D {
            let bit = bitboard::LSB_CELL;
            let b = black & bit;
            let w = white & bit;
            black >>= 1;
            white >>= 1;
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

        for i in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let h1 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i));
                let h2 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i + 4));
                let h3 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i + 8));
                let h4 = x86_64::_mm_loadu_ps(hid.as_ptr().add(i + 12));
                // teban
                let wtbn1 = x86_64::_mm_load_ps(wtbn.as_ptr().add(i));
                let wtbn2 = x86_64::_mm_load_ps(wtbn.as_ptr().add(i + 4));
                let wtbn3 = x86_64::_mm_load_ps(wtbn.as_ptr().add(i + 8));
                let wtbn4 = x86_64::_mm_load_ps(wtbn.as_ptr().add(i + 12));
                let tbn = x86_64::_mm_set1_ps(teban);
                let h1 = x86_64::_mm_fmadd_ps(wtbn1, tbn, h1);
                let h2 = x86_64::_mm_fmadd_ps(wtbn2, tbn, h2);
                let h3 = x86_64::_mm_fmadd_ps(wtbn3, tbn, h3);
                let h4 = x86_64::_mm_fmadd_ps(wtbn4, tbn, h4);
                // fixed stones
                let wfsb1 = x86_64::_mm_load_ps(wfs.as_ptr().add(i));
                let wfsb2 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + 4));
                let wfsb3 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + 8));
                let wfsb4 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + 12));
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let h1 = x86_64::_mm_fmadd_ps(wfsb1, fsb, h1);
                let h2 = x86_64::_mm_fmadd_ps(wfsb2, fsb, h2);
                let h3 = x86_64::_mm_fmadd_ps(wfsb3, fsb, h3);
                let h4 = x86_64::_mm_fmadd_ps(wfsb4, fsb, h4);
                let wfsw1 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + N_HIDDEN));
                let wfsw2 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + 4 + N_HIDDEN));
                let wfsw3 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + 8 + N_HIDDEN));
                let wfsw4 = x86_64::_mm_load_ps(wfs.as_ptr().add(i + 12 + N_HIDDEN));
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let h1 = x86_64::_mm_fmadd_ps(wfsw1, fsw, h1);
                let h2 = x86_64::_mm_fmadd_ps(wfsw2, fsw, h2);
                let h3 = x86_64::_mm_fmadd_ps(wfsw3, fsw, h3);
                let h4 = x86_64::_mm_fmadd_ps(wfsw4, fsw, h4);
                // dc
                // let wdc1 = x86_64::_mm_load_ps(wdc.as_ptr().add(i));
                // let wdc2 = x86_64::_mm_load_ps(wdc.as_ptr().add(i + 4));
                // let wdc3 = x86_64::_mm_load_ps(wdc.as_ptr().add(i + 8));
                // let wdc4 = x86_64::_mm_load_ps(wdc.as_ptr().add(i + 12));
                // let h1 = x86_64::_mm_add_ps(wdc1, h1);
                // let h2 = x86_64::_mm_add_ps(wdc2, h2);
                // let h3 = x86_64::_mm_add_ps(wdc3, h3);
                // let h4 = x86_64::_mm_add_ps(wdc4, h4);
                // relu
                let zero = x86_64::_mm_setzero_ps();
                let y1 = x86_64::_mm_max_ps(h1, zero);
                let y2 = x86_64::_mm_max_ps(h2, zero);
                let y3 = x86_64::_mm_max_ps(h3, zero);
                let y4 = x86_64::_mm_max_ps(h4, zero);

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
                // x86_64::_mm_store_ps(hid2.add(i), s4);
                x86_64::_mm_store_ps(hid2.as_mut_ptr().add(i), s4);
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
    pub fn evaluatev9bb_simd_mul(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let mut black = ban.black;
        let mut white = ban.white;
        let teban = ban.teban as f32;

        let (fsb, fsw) = ban.fixedstones();

        let ow = self.wbanv(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 16;
        let mut hid = [0f32 ; N_HIDDEN];
        // cells
        for idx in 0..bitboard::CELL_2D {
            let bit = bitboard::LSB_CELL;
            let b = black & bit;
            let w = white & bit;
            black >>= 1;
            white >>= 1;
            if b | w == 0 {continue;}  // no stone

            let we1 = &ow[idx * N_HIDDEN .. ];
            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w = vld1q_f32_x4(we1.as_ptr().add(i));
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

                let tbn = vmovq_n_f32(teban);
                let wtb = vld1q_f32_x4(wtbn.as_ptr().add(i));
                let sum41 = vmlaq_f32(sum4.0, tbn, wtb.0);
                let sum42 = vmlaq_f32(sum4.1, tbn, wtb.1);
                let sum43 = vmlaq_f32(sum4.2, tbn, wtb.2);
                let sum44 = vmlaq_f32(sum4.3, tbn, wtb.3);

                let fsb4 = vmovq_n_f32(fsb as f32);
                let wfsb = vld1q_f32_x4(wfs.as_ptr().add(i));
                let sum41 = vmlaq_f32(sum41, fsb4, wfsb.0);
                let sum42 = vmlaq_f32(sum42, fsb4, wfsb.1);
                let sum43 = vmlaq_f32(sum43, fsb4, wfsb.2);
                let sum44 = vmlaq_f32(sum44, fsb4, wfsb.3);

                let fsw4 = vmovq_n_f32(fsw as f32);
                let wfsw = vld1q_f32_x4(wfs.as_ptr().add(i + N_HIDDEN));
                let sum41 = vmlaq_f32(sum41, fsw4, wfsw.0);
                let sum42 = vmlaq_f32(sum42, fsw4, wfsw.1);
                let sum43 = vmlaq_f32(sum43, fsw4, wfsw.2);
                let sum44 = vmlaq_f32(sum44, fsw4, wfsw.3);

                let wdc4 = vld1q_f32_x4(wdc.as_ptr().add(i));
                let sum41 = vaddq_f32(sum41, wdc4.0);
                let sum42 = vaddq_f32(sum42, wdc4.1);
                let sum43 = vaddq_f32(sum43, wdc4.2);
                let sum44 = vaddq_f32(sum44, wdc4.3);
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
    pub fn evaluatev9bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        let prgs = ban.progress();
        let mut black = ban.black;
        let mut white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wbanv(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 32;
        let mut hid = [0f32 ; N_HIDDEN];
        hid.copy_from_slice(wdc);
        for idx in 0..bitboard::CELL_2D {
            let bit = bitboard::LSB_CELL;
            let b = black & bit;
            let w = white & bit;
            black >>= 1;
            white >>= 1;
            if b | w == 0 {continue;}  // no stone

            let wei = if b != 0 {
                &ow[idx * N_HIDDEN * 2 .. ]
            } else {
                &ow[idx * N_HIDDEN * 2 + N_HIDDEN.. ]
            };
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
                let x1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx));
                let x2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx + 8));
                let x3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx + 16));
                let x4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(hidx + 24));

                // teban
                let wtbn1 = x86_64::_mm256_load_ps(wtbn.as_ptr().add(hidx));
                let wtbn2 = x86_64::_mm256_load_ps(
                    wtbn.as_ptr().add(hidx + 8));
                let wtbn3 = x86_64::_mm256_load_ps(
                    wtbn.as_ptr().add(hidx + 16));
                let wtbn4 = x86_64::_mm256_load_ps(
                    wtbn.as_ptr().add(hidx + 24));
                let tbn = x86_64::_mm256_set1_ps(teban);
                let h1 = x86_64::_mm256_fmadd_ps(wtbn1, tbn, x1);
                let h2 = x86_64::_mm256_fmadd_ps(wtbn2, tbn, x2);
                let h3 = x86_64::_mm256_fmadd_ps(wtbn3, tbn, x3);
                let h4 = x86_64::_mm256_fmadd_ps(wtbn4, tbn, x4);
                // fixed stones
                let wfsb1 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx));
                let wfsb2 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx + 8));
                let wfsb3 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + 16));
                let wfsb4 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + 24));
                let fsb = x86_64::_mm256_set1_ps(fs.0 as f32);
                let h1 = x86_64::_mm256_fmadd_ps(wfsb1, fsb, h1);
                let h2 = x86_64::_mm256_fmadd_ps(wfsb2, fsb, h2);
                let h3 = x86_64::_mm256_fmadd_ps(wfsb3, fsb, h3);
                let h4 = x86_64::_mm256_fmadd_ps(wfsb4, fsb, h4);
                let wfsw1 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + N_HIDDEN));
                let wfsw2 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + N_HIDDEN + 8));
                let wfsw3 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + N_HIDDEN + 16));
                let wfsw4 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + N_HIDDEN + 24));
                let fsw = x86_64::_mm256_set1_ps(fs.1 as f32);
                let h1 = x86_64::_mm256_fmadd_ps(wfsw1, fsw, h1);
                let h2 = x86_64::_mm256_fmadd_ps(wfsw2, fsw, h2);
                let h3 = x86_64::_mm256_fmadd_ps(wfsw3, fsw, h3);
                let h4 = x86_64::_mm256_fmadd_ps(wfsw4, fsw, h4);
                // relu
                let zero = x86_64::_mm256_setzero_ps();
                let y1 = x86_64::_mm256_max_ps(zero, h1);
                let y2 = x86_64::_mm256_max_ps(zero, h2);
                let y3 = x86_64::_mm256_max_ps(zero, h3);
                let y4 = x86_64::_mm256_max_ps(zero, h4);
                // x86_64::_mm256_store_ps(hid.add(hidx), y1);
                // x86_64::_mm256_store_ps(hid.add(hidx + 8), y2);
                // x86_64::_mm256_store_ps(hid.add(hidx + 16), y3);
                // x86_64::_mm256_store_ps(hid.add(hidx + 24), y4);
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
                    x86_64::_mm_store_ps(hid2.as_mut_ptr().add(i / 8), s4);
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

#[test]
fn test_exchange_weight() {
        let mut w = weight::Weight::new();
        w.init();
        for p in 0..N_PROGRESS_DIV {
            let wei = w.wban(p);
            let vwei = w.wbanv(p);
            for i in 0..N_WEIGHT_TEBAN {
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
    for rfen in rfens.iter() {
        let bban = bitboard::BitBoard::from(rfen).unwrap();
        let ban = bitboard::BitBoard::from(rfen).unwrap();
        ban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimde = w.evaluatev9bb(&bban);
        let res_simd = w.evaluatev9bb_simd(&bban);
        let res_simdavx = w.evaluatev9bb_simdavx(&bban);
        assert!(dbg_assert_eq(&res_nosimde, &res_simd));
        assert!(dbg_assert_eq(&res_nosimde, &res_simdavx));
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
    for rfen in rfens.iter() {
        let bban = bitboard::BitBoard::from(rfen).unwrap();
        bban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimdi = w.evaluatev9bb(&bban);
        let res_simdmul = w.evaluatev9bb_simd_mul(&bban);
        // let res_simd = w.evaluatev9bb_simd(&bban);
        assert!(dbg_assert_eq(&res_nosimdi, &res_simdmul));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}
