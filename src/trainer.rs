use rand::prelude::SliceRandom;

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

    pub fn learn_win(&self, files : &mut Vec<String>) {
        let mut rng = rand::thread_rng();
        for i in 0..self.repeat {
            // println!("{} / {}", i, self.repeat);
            // rng.shuffle(files);
            files.shuffle(&mut rng);
            for fname in files.iter() {
                let path = format!("kifu/{}", fname);
                print!("{} / {} : {}\r", i, self.repeat, path);
                let content =
                    std::fs::read_to_string(path).unwrap();
                let lines:Vec<&str> = content.split("\n").collect();
                let kifu = kifu::Kifu::from(&lines);
                unsafe {
                    self.run4win(&kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
                }
            }
            println!("");
        }
        println!("Done.");
    }

    pub fn run4win(&self, kifu: &kifu::Kifu, weight: &mut weight::Weight) -> Result<(), String> {
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

    pub fn learn_stones(&self, files : &mut Vec<String>) {
        let mut rng = rand::thread_rng();
        for i in 0..self.repeat {
            // println!("{} / {}", i, self.repeat);
            // rng.shuffle(files);
            files.shuffle(&mut rng);
            for fname in files.iter() {
                let path = format!("kifu/{}", fname);
                print!("{} / {} : {}\r", i, self.repeat, path);
                let content =
                    std::fs::read_to_string(path).unwrap();
                let lines:Vec<&str> = content.split("\n").collect();
                let kifu = kifu::Kifu::from(&lines);
                unsafe {
                    self.run4stones(&kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
                }
            }
            println!("");
        }
        println!("Done.");
    }

    /**
     * 読み込んだ棋譜をキャッシュする版
     */
    pub fn learn_stones_cache(&self, files : &mut Vec<String>) {
        let mut rng = rand::thread_rng();
        let mut kifucache : Vec<(String, kifu::Kifu)> = Vec::new();
        for i in 0..1 {
            for fname in files.iter() {
                let path = format!("kifu/{}", fname);
                print!("{} / {} : {}\r", i, self.repeat, path);
                let content = std::fs::read_to_string(&path).unwrap();
                let lines:Vec<&str> = content.split("\n").collect();
                let kifu = kifu::Kifu::from(&lines);
                let p = String::from(&path);
                kifucache.push((p, kifu.copy()));
                unsafe {
                    self.run4stones(&kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
                }
            }
            println!("");
        }
        for i in 1..self.repeat {
            files.shuffle(&mut rng);
            for fname in files.iter() {
                let path = format!("kifu/{}", fname);
                print!("{} / {} : {}\r", i, self.repeat, path);
                let idx = kifucache.binary_search_by(|x| {
                    x.0.cmp(&path)
                }).unwrap();
                let kifu = kifucache.iter().nth(idx).unwrap();
                unsafe {
                    self.run4stones(&kifu.1, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
                }
            }
            println!("");
        }
        println!("Done.");
    }

    pub fn run4stones(&self, kifu: &kifu::Kifu, weight: &mut weight::Weight) -> Result<(), String> {
        let score = kifu.score;
        if score.is_none() {
            return Err(String::from("invalid score."));
        }
        let score = score.unwrap();
        for l in kifu.list.iter() {
            if weight.train(&l.rfen, score, self.eta).is_err() {
                return Err(String::from("error while training"));
            }
        }
        Ok(())
    }
}
