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
        ($x * 0.5 / ($x.abs() + 1.0) + 0.5)  // 0 ~ 1
        // $x / ($x.abs() + 1.0)  // -1 ~ 1
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

            sum += wh[i] * softsign!(hidsum);
        }
        sum
    }

    pub fn evaluatev3bb(&self, ban : &bitboard::BitBoard) -> f32 {
        self.evaluatev1bb(ban)
    }
    pub fn evaluatev3bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        self.evaluatev1bb(ban)
    }
    pub fn evaluatev3bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        self.evaluatev1bb(ban)
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
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += wh[i] * hidsig[i];
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
