use super::*;

use std::io::{self};

pub struct Game {
    ban : board::Board,
    pub kifu : kifu::Kifu,
}

impl Game {
    pub fn new() -> Game {
        Game {
            ban : board::Board::init(),
            kifu : kifu::Kifu::new(),
        }
    }

    pub fn from(rfen : &str) -> Game {
        Game {
            ban: board::Board::from(rfen).unwrap(),
            kifu: kifu::Kifu::new()
        }
    }

    pub fn start(&mut self, f : fn(&board::Board, usize) -> Option<(f32, node::Node)>, depth : usize) -> Result<(), String> {
        loop {
            // show
            // self.ban.put();
            println!("{}", self.ban.to_str());
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            println!("val:{:.3} {} {}msec", val, node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x, y, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_str();
                let teban = self.ban.teban;
                self.kifu.append(0, 0, teban, rfen);
                break;
            }
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        println!("{}", self.kifu.to_str());
        // show
        self.ban.put();
        Ok(())
    }

    pub fn start_against_stdin(&mut self, f : fn(&board::Board, usize) -> Option<(f32, node::Node)>, depth : usize, turnin : i8) -> Result<(), String> {
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.len() == 0 {
                    println!("auto pass.");
                    x = 0;
                    y = 0;
                    break;
                } else {
                    loop {
                        print!("your turn[a1 ~ h8]:");
                        io::stdout().flush().unwrap();
                        let mut txt = String::new();
                        io::stdin().read_line(&mut txt).unwrap();
                        txt.pop();
                        let xx = "0abcdefgh".find(txt.chars().nth(0).unwrap()).unwrap_or(10);
                        let yy = txt.chars().nth(1).unwrap().to_digit(10);
                        if yy.is_none() {
                            println!("invalid position : {}", txt);
                            continue;
                        }
                        let yy = yy.unwrap();
                        if xx > 8 || yy > 8 {
                            println!("invalid position : {}", txt);
                            continue;
                        }
                        let pos = (xx, yy as usize);
                        if movable.contains(&pos) {
                            x = xx;
                            y = yy as usize;
                            break;
                        }
                        println!("{} is not allowed.", txt);
                    }
                }
           } else {
                println!("{}", self.ban.to_str());
                // think
                let st = Instant::now();
                let (val, node) = f(&self.ban, depth).unwrap();
                // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
                // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
                let ft = st.elapsed();
                println!("val:{:.3} {} {}msec", val, node.dump(), ft.as_millis());
                let best = node.best.unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x, y, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_str();
                let teban = self.ban.teban;
                self.kifu.append(0, 0, teban, rfen);
                break;
            }
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        println!("{}", self.kifu.to_str());
        // show
        self.ban.put();
        Ok(())
    }

    pub fn start_with_2et(&mut self,
            f : fn(&board::Board, usize) -> Option<(f32, node::Node)>,
            depth : usize, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        loop {
            // show
            // self.ban.put();
            println!("{}", self.ban.to_str());
            // switch weight
            if self.ban.teban == board::SENTE {
                unsafe {
                    node::WEIGHT.as_mut().unwrap().copy(et1);
                }
            } else {
                unsafe {
                    node::WEIGHT.as_mut().unwrap().copy(et2);
                }
            }
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            println!("val:{:.3} {} {}msec", val, node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x, y, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_str();
                let teban = self.ban.teban;
                self.kifu.append(0, 0, teban, rfen);
                break;
            }
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        println!("{}", self.kifu.to_str());
        // show
        self.ban.put();
        Ok(())
    }
}
