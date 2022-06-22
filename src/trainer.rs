use rand::{prelude::SliceRandom, Rng};

use super::*;

pub struct Trainer {
    eta: f32,
    repeat: usize,
}

impl Trainer {
    pub fn new(eta: f32, repeat: usize) -> Trainer {
        Trainer {
            eta: eta,
            repeat: repeat,
        }
    }

    pub fn learn(&self, files : &mut Vec<String>) {
        let mut rng = rand::thread_rng();
        for i in 0..self.repeat {
            println!("{} / {}", i, self.repeat);
            // rng.shuffle(files);
            files.shuffle(&mut rng);
            for fname in files.iter() {
                let path = format!("kifu/{}", fname);
                print!("{}", path);
                let content =
                    std::fs::read_to_string(path).unwrap();
                let lines:Vec<&str> = content.split("\n").collect();
                let kifu = kifu::Kifu::from(&lines);
                unsafe {
                    self.run(&kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
                }
            }
        }
        println!("Done.");
    }
    pub fn run(&self, kifu: &kifu::Kifu, weight: &mut weight::Weight) -> Result<(), String> {
        let winner = kifu.winner();
        if winner.is_none() {
            return Err(String::from("invalid kifu."));
        }
        let winner = winner.unwrap();
        for l in kifu.list.iter() {
            if weight.train(&l.rfen, winner, self.eta).is_err() {
                return Err(String::from("error while training"));
            }
        }
        Ok(())
    }
}
