use super::*;
use rand::Rng;
use std::fs;

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
        let content = fs::read_to_string(path).unwrap();
        let csv = content.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let wsz = self.weight.len();
        let nsz = newtable.len();
        if wsz != nsz {
            return Err(String::from("size mismatch"));
        }
        self.weight = newtable;

        Ok(())
    }

    pub fn write(&self, path : &str) {
        let sv = self.weight.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        let mut f = fs::File::create(path).unwrap();
        f.write(sv.join(",").as_bytes()).unwrap();
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
            sum += w2[i] / (f32::exp(hidsum) + 1.0);
        }
        sum
    }

    pub fn forward(&self, ban : &board::Board)
            -> ([f32;N_HIDDEN], [f32;N_HIDDEN], [f32;N_OUTPUT]) {
        let mut ret = Vec::<Vec<f32>>::new();
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
            hidsig[i] = 1.0 / (f32::exp(hidsum) + 1.0);
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
        let (hidden, hidsig, output) = self.forward(&ban);
        // backword
        let w1sz = board::CELL_2D + 1 + 1;
        let mut ow = &mut self.weight;
        // back to hidden
        let diff : f32 = output[0] - 10.0 * winner as f32;
        let mut w2 = &mut ow.as_mut_slice()[w1sz * 4..];
        for i in 0..N_HIDDEN {
            w2[i] -= hidsig[i] * diff * eta;
        }
        w2[N_HIDDEN] -= diff * eta;

        let mut dhid = [0.0 as f32 ; N_HIDDEN];
        for (i, h) in dhid.iter_mut().enumerate() {
            let tmp = w2[i] * diff;
            let sig = 1.0 / (1.0 + f32::exp(hidden[i]));
            *h = tmp * sig * (1.0 - sig);
        }
        // back to input
        for (i, h) in dhid.iter().enumerate() {
            let mut w1 = &mut ow.as_mut_slice()[i * w1sz .. (i + 1) * w1sz];
            for (j, c) in cells.iter().enumerate() {
                w1[j] -= *h * *c as f32 * eta;
            }
            w1[board::CELL_2D] -= *h * teban as f32 * eta;
            w1[board::CELL_2D + 1] -= *h * eta;
        }
    }
}