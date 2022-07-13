use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
const N_INPUT : usize = board::CELL_2D + 1;
const N_HIDDEN : usize = 4;
const N_OUTPUT : usize = 1;
const N_WEIGHT: usize = (N_INPUT + 1) * N_HIDDEN + N_HIDDEN + 1;

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
        let file = File::open(path).unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if l.starts_with("#") {
                        continue;
                    }
                    let csv = l.split(",").collect::<Vec<_>>();
                    let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
                    let wsz = self.weight.len();
                    let nsz = newtable.len();
                    if wsz != nsz {
                        return Err(String::from("size mismatch"));
                    }
                    self.weight = newtable;
                    return Ok(());
                },
                Err(err) => {return Err(err.to_string())}
            }
        }

        Err("no weight".to_string())
    }

    pub fn write(&self, path : &str) {
        let sv = self.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        let mut f = fs::File::create(path).unwrap();
        f.write(format!("# {}-{}-{}\n", N_INPUT, N_HIDDEN, N_OUTPUT).as_bytes()).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
    }

    pub fn copy(&mut self, src : &Weight) {
        for (d, s) in self.weight.iter_mut().zip(src.weight.iter()) {
            *d = *s;
        }
    }

    pub fn evaluate(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            for (idx, c)  in cells.iter().enumerate() {
                hidsum += *c as f32 * w1[idx];
            }
            hidsum += teban as f32 * w1[w1sz - 2];
            sum += w2[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn evaluate_simd(&self, ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban;
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
            let mut sum4: std::arch::x86_64::__m128;
            unsafe {
                sum4 = std::arch::x86_64::_mm_setzero_ps();
            }
            for i in 0..board::CELL_2D / 4 {
                // let x4 = f32x4::load(w1[i + 4], 4);
                // let y4 = f32x4::new(cells[i * 4], cells[i * 4 + 1], cells[i * 4 + 2], cells[i * 4 + 3]);
                // sum4 += x4 * y4;
                let idx = i * 4;
                unsafe {
                    let x4 = std::arch::x86_64::_mm_loadu_ps(w1[idx..].as_ptr());
                    // let y4 = std::arch::x86_64::_mm_set_ps(
                    //     cells[idx] as f32, cells[idx + 1] as f32,
                    //     cells[idx + 2] as f32, cells[idx + 3] as f32);
                    let y4 = std::arch::x86_64::_mm_set_epi32(
                        cells[idx + 3] as i32, cells[idx + 2] as i32,
                        cells[idx + 1] as i32, cells[idx + 0] as i32);
                    // let y4 = std::arch::x86_64::_mm_set_epi32(
                    //     cells[idx] as i32, cells[idx + 1] as i32,
                    //     cells[idx + 2] as i32, cells[idx + 3] as i32);
                    let y4 = std::arch::x86_64::_mm_cvtepi32_ps(y4);
                    let mul = std::arch::x86_64::_mm_mul_ps(x4, y4);
                    sum4 = std::arch::x86_64::_mm_add_ps(sum4, mul);
                }
            }
            let mut sumarr : [f32 ; 4] = [0.0, 0.0, 0.0, 0.0];
            unsafe {
                std::arch::x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
                // std::arch::x86_64::_mm_store_ps(sumarr.as_mut_ptr(),
                //     std::arch::x86_64::_mm_hadd_ps(sum4, sum4));
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            // hidsum += sumarr[0] + sumarr[2];
            hidsum += teban as f32 * w1[w1sz - 2];
            sum += w2[i] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn forward(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        for i in 0..N_HIDDEN {
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            for (idx, c)  in cells.iter().enumerate() {
                hidsum += *c as f32 * w1[idx];
            }
            hidsum += teban as f32 * w1[w1sz - 2];
            hidden[i] = hidsum;
            hidsig[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
            sum += w2[i] * hidsig[i];
        }
        output[0] = sum;
        (hidden, hidsig, output)
    }

    pub fn forward_simd(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut hidden : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut hidsig : [f32 ; N_HIDDEN] = [0.0 ; N_HIDDEN];
        let mut output : [f32 ; N_OUTPUT] = [0.0 ; N_OUTPUT];
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban;
        let w1sz = board::CELL_2D + 1 + 1;
        let ow = &self.weight;
        let w2 = &ow.as_slice()[w1sz * 4..];

        sum = *ow.last().unwrap();

        let mut sum4: std::arch::x86_64::__m128;
        for i in 0..N_HIDDEN {
            let mut sum4: std::arch::x86_64::__m128;
            unsafe {
                sum4 = std::arch::x86_64::_mm_setzero_ps();
            }
            let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
            let mut hidsum : f32 = *w1.last().unwrap();
            for i in 0..board::CELL_2D / 4 {
                // let x4 = f32x4::load(w1[i + 4], 4);
                // let y4 = f32x4::new(cells[i * 4], cells[i * 4 + 1],
                //     cells[i * 4 + 2], cells[i * 4 + 3]);
                // sum4 += x4 * y4;
                let idx = i * 4;
                unsafe {
                    let x4 = std::arch::x86_64::_mm_loadu_ps(w1[idx..].as_ptr());
                    // let y4 = std::arch::x86_64::_mm_set_ps(
                    //     cells[idx] as f32, cells[idx + 1] as f32,
                    //     cells[idx + 2] as f32, cells[idx + 3] as f32);
                    let y4 = std::arch::x86_64::_mm_set_epi32(
                        cells[idx + 3] as i32, cells[idx + 2] as i32,
                        cells[idx + 1] as i32, cells[idx + 0] as i32);
                    // let y4 = std::arch::x86_64::_mm_set_epi32(
                    //     cells[idx] as i32, cells[idx + 1] as i32,
                    //     cells[idx + 2] as i32, cells[idx + 3] as i32);
                    let y4 = std::arch::x86_64::_mm_cvtepi32_ps(y4);
                    let mul = std::arch::x86_64::_mm_mul_ps(x4, y4);
                    sum4 = std::arch::x86_64::_mm_add_ps(sum4, mul);
                }
            }
            let mut sumarr : [f32 ; 4] = [0.0, 0.0, 0.0, 0.0];
            unsafe {
                std::arch::x86_64::_mm_store_ps(sumarr.as_mut_ptr(), sum4);
                // std::arch::x86_64::_mm_store_ps(sumarr.as_mut_ptr(),
                //     std::arch::x86_64::_mm_hadd_ps(sum4, sum4));
            }
            hidsum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
            hidsum += teban as f32 * w1[w1sz - 2];
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

    fn learn(&mut self, ban : &board::Board, winner : i8, eta : f32) {
        let cells = &ban.cells;
        let teban = ban.teban;
        // forward
        let (hidden, hidsig, output) = 
            if cfg!(feature="nosimd") {
                self.forward(&ban);
            } else {
                self.forward_simd(&ban);
            }
        // backword
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
            for (j, c) in cells.iter().enumerate() {
                w1[j] -= *h * *c as f32 * eta;
            }
            w1[board::CELL_2D] -= *h * teban as f32 * eta;
            w1[board::CELL_2D + 1] -= *h * eta;
        }
    }
}
