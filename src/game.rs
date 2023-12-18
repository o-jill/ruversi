use super::*;

use std::io::{self, Write};
use std::sync::{Arc, RwLock};

pub struct GameBB {
    ban : bitboard::BitBoard,
    pub kifu : kifu::Kifu,
}

pub struct Game {
    ban : board::Board,
    pub kifu : kifu::Kifu,
}

impl GameBB {
    pub fn new() -> GameBB {
        GameBB {
            ban : bitboard::BitBoard::new(),
            kifu : kifu::Kifu::new(),
        }
    }

    pub fn from(rfen : &str) -> GameBB {
        GameBB {
            ban: bitboard::BitBoard::from(rfen).unwrap(),
            kifu: kifu::Kifu::new()
        }
    }

    #[allow(dead_code)]
    pub fn start(&mut self, f : fn(&bitboard::BitBoard, u8) -> Option<(f32, nodebb::NodeBB)>, depth : u8)
            -> Result<(), String> {
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
            println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    pub fn starto(&mut self, f : fn(&bitboard::BitBoard, u8) -> Option<(f32, &nodebb::NodeBB)>, depth : u8)
            -> Result<(), String> {
        loop {
            // show
            // self.ban.put();
            println!("{}", self.ban.to_str());
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();

            let ft = st.elapsed();
            println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    pub fn startgk(&mut self, f : fn(&bitboard::BitBoard, u8, &mut nodebb::NodeBB) -> Option<f32>, depth : u8)
            -> Result<(), String> {
        loop {
            let mut node = nodebb::NodeBB::new(0, 0, depth, bitboard::NONE);
            // show
            // self.ban.put();
            // println!("{}", self.ban.to_str());
            // think
            let st = Instant::now();
            let val = f(&self.ban, depth, &mut node).unwrap();

            let ft = st.elapsed();
            // println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    #[allow(dead_code)]
    pub fn startsh(&mut self, f : fn(&bitboard::BitBoard, u8) -> Option<(f32, Arc<RwLock<shnode::ShNode>>)>, depth : u8)
            -> Result<(), String> {
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
            let nd = node.read().unwrap();
            println!("val:{val:+5.1} {} {}msec", nd.dump(), ft.as_millis());
            let best = nd.best.as_ref().unwrap();
            
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    #[allow(dead_code)]
    pub fn start_against_stdin(&mut self,
            f : fn(&bitboard::BitBoard, u8) -> Option<(f32, nodebb::NodeBB)>,
            depth : u8, turnin : i8) -> Result<(), String> {
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
                } else if movable.len() == 1 {
                    x = movable[0].0;
                    y = movable[0].1;
                } else {
                    loop {
                        print!("your turn[a1 ~ h8]:");
                        io::stdout().flush().unwrap();
                        let mut txt = String::new();
                        io::stdin().read_line(&mut txt).unwrap();
                        txt.pop();
                        let xx = "0abcdefgh".find(txt.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                        let yy = txt.chars().nth(1).unwrap().to_digit(10);
                        if yy.is_none() {
                            println!("invalid position : {}", txt);
                            continue;
                        }
                        let yy = yy.unwrap() as u8;
                        if xx > 8 || yy > 8 {
                            println!("invalid position : {}", txt);
                            continue;
                        }
                        let pos = (xx, yy);
                        if movable.contains(&pos) {
                            x = xx;
                            y = yy;
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
                println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
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
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    pub fn starto_against_stdin(&mut self,
            f : fn(&bitboard::BitBoard, u8) -> Option<(f32, &nodebb::NodeBB)>,
            depth : u8, turnin : i8) -> Result<(), String> {
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    println!("auto pass.");
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0].0;
                    y = movable[0].1;
                } else {
                    loop {
                        print!("your turn[a1 ~ h8]:");
                        io::stdout().flush().unwrap();
                        let mut txt = String::new();
                        io::stdin().read_line(&mut txt).unwrap();
                        txt.pop();
                        let xx = "0abcdefgh".find(txt.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                        let yy = txt.chars().nth(1).unwrap().to_digit(10);
                        if yy.is_none() {
                            println!("invalid position : {}", txt);
                            continue;
                        }
                        let yy = yy.unwrap() as u8;
                        if xx > 8 || yy > 8 {
                            println!("invalid position : {}", txt);
                            continue;
                        }
                        let pos = (xx, yy);
                        if movable.contains(&pos) {
                            x = xx;
                            y = yy;
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
                println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
                let best = node.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    #[allow(dead_code)]
    pub fn start_against_edax(&mut self,
            f : fn(&bitboard::BitBoard, u8) -> Option<(f32, nodebb::NodeBB)>,
            depth : u8, turnin : i8) -> Result<(), String> {
        let er = edaxrunner::EdaxRunner::new();
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                println!("{}", self.ban.to_str());
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.len() == 0 {
                    println!("auto pass.");
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0].0;
                    y = movable[0].1;
                } else {
                    // println!("put board to a file...");
                    er.obf2file(&self.ban.to_obf());
                    // launch edax
                    match er.run() {
                        Ok((pos, _)) => {
                            x = "0abcdefgh".find(pos.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                            y = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                        },
                        Err(msg) => panic!("error running edax... [{msg}]"),
                    }
                }
           } else {
                println!("{}", self.ban.to_str());
                // think
                let st = Instant::now();
                let (val, node) = f(&self.ban, depth).unwrap();
                let ft = st.elapsed();
                println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
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
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    /// play a game against Edax.  
    /// # Arguments  
    /// - f : fn for searching.  
    /// - depth : searching depth.  
    /// - turnin : Edax's turn.  
    /// # Returns  
    /// () or Error message.
    pub fn starto_against_edax(&mut self,
            f : fn(&bitboard::BitBoard, u8) -> Option<(f32, &nodebb::NodeBB)>,
            depth : u8, turnin : i8, econf : &str) -> Result<(), String> {
        let er = match edaxrunner::EdaxRunner::from_config(econf) {
            Ok(e) => e,
            Err(msg) => return Err(msg),
        };
        loop {
            // show
            println!("{}", self.ban.to_str());
            let x;
            let y;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    println!("auto pass.");
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0].0;
                    y = movable[0].1;
                } else {
                    // println!("put board to a file...");
                    er.obf2file(&self.ban.to_obf());
                    // launch edax
                    match er.run() {
                        Ok((pos, _)) => {
                            x = "0abcdefgh".find(pos.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                            y = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                        },
                        Err(msg) => panic!("error running edax... [{msg}]"),
                    }
                }
            } else {
                // think
                let st = Instant::now();
                let (val, node) = f(&self.ban, depth).unwrap();
                let ft = st.elapsed();
                println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
                let best = node.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    #[allow(dead_code)]
    pub fn start_with_2et(&mut self,
            f : fn(&bitboard::BitBoard, u8) -> Option<(f32, nodebb::NodeBB)>,
            depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
                -> Result<(), String> {
        loop {
            // show
            // self.ban.put();
            println!("{}", self.ban.to_str());
            // switch weight
            if self.ban.teban == bitboard::SENTE {
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et1);
                }
            } else {
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et2);
                }
            }
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    pub fn starto_with_2et(&mut self,
        f : fn(&bitboard::BitBoard, u8) -> Option<(f32, &nodebb::NodeBB)>,
        depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        loop {
            // show
            // self.ban.put();
            println!("{}", self.ban.to_str());
            // switch weight
            if self.ban.teban == bitboard::SENTE {
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et1);
                }
            } else {
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et2);
                }
            }
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    #[allow(dead_code)]
    pub fn startsh_with_2et(&mut self,
            f : fn(&bitboard::BitBoard, u8) -> Option<(f32, Arc<RwLock<shnode::ShNode>>)>,
            // f : fn(&bitboard::BitBoard, u8) -> Option<(f32, Arc<Mutex<shnode::ShNode>>)>,
            depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
                -> Result<(), String> {
        loop {
            // show
            // self.ban.put();
            println!("{}", self.ban.to_str());
            // switch weight
            if self.ban.teban == bitboard::SENTE {
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et1);
                }
            } else {
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et2);
                }
            }
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            let x;
            let y;
            let ban;
            {
                let nd = node.read().unwrap();
                println!("val:{val:+5.1} {} {}msec", nd.dump(), ft.as_millis());
                let best = nd.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
                // apply move
                ban = self.ban.r#move(x, y).unwrap();
            }
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    pub fn start(&mut self, f : fn(&board::Board, u8) -> Option<(f32, node::Node)>, depth : u8) -> Result<(), String> {
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
            println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    pub fn start_against_stdin(&mut self,
            f : fn(&board::Board, u8) -> Option<(f32, node::Node)>,
            depth : u8, turnin : i8) -> Result<(), String> {
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
                } else if movable.len() == 1 {
                    x = movable[0].0;
                    y = movable[0].1;
                } else {
                    loop {
                        print!("your turn[a1 ~ h8]:");
                        io::stdout().flush().unwrap();
                        let mut txt = String::new();
                        io::stdin().read_line(&mut txt).unwrap();
                        txt.pop();
                        let xx = "0abcdefgh".find(txt.chars().nth(0).unwrap()).unwrap_or(10) as u8;
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
                        let pos = (xx, yy as u8);
                        if movable.contains(&pos) {
                            x = xx;
                            y = yy as u8;
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
                println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
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
            self.kifu.append(x as usize, y as usize, teban, rfen);

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

    /// play a game against Edax.  
    /// # Arguments  
    /// - f : fn for searching.  
    /// - depth : searching depth.  
    /// - turnin : Edax's turn.  
    /// # Returns  
    /// () or Error message.
    pub fn start_against_edax(&mut self,
            f : fn(&board::Board, u8) -> Option<(f32, node::Node)>,
            depth : u8, turnin : i8) -> Result<(), String> {
        let er = edaxrunner::EdaxRunner::new();
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                println!("{}", self.ban.to_str());
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.len() == 0 {
                    println!("auto pass.");
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0].0;
                    y = movable[0].1;
                } else {
                    // println!("put board to a file...");
                    er.obf2file(&self.ban.to_obf());
                    // launch edax
                    match er.run() {
                        Ok((pos, _)) => {
                            x = "0abcdefgh".find(pos.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                            y = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                        },
                        Err(msg) => panic!("error running edax... [{msg}]"),
                    }
                }
           } else {
                println!("{}", self.ban.to_str());
                // think
                let st = Instant::now();
                let (val, node) = f(&self.ban, depth).unwrap();
                let ft = st.elapsed();
                println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
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
            self.kifu.append(x as usize, y as usize, teban, rfen);

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
            f : fn(&board::Board, u8) -> Option<(f32, node::Node)>,
            depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
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
            println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(x as usize, y as usize, teban, rfen);

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
