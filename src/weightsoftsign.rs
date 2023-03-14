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
const WSZV1 : usize = (board::CELL_2D + 1 + 1) * 4 + 4 + 1;

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

            sum += wh[i] * hidsum * 0.5 / (hidsum.abs() + 1.0) + 0.5;  // 0 ~ 1
            // sum += wh[i] * hidsum / (hidsum.abs() + 1.0);  // -1 ~ 1
        }
        sum
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

            sum += wh[i] * hidsum * 0.5 / (hidsum.abs() + 1.0) + 0.5;  // 0 ~ 1
            // sum += wh[i] * hidsum / (hidsum.abs() + 1.0);  // -1 ~ 1
        }
        sum
    }
}
