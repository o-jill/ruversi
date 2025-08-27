use super::*;

use std::io::{self, Write};
use std::sync::{Arc, RwLock};

type SearchFn = fn(&bitboard::BitBoard, u8, &mut nodebb::NodeBB, &weight::Weight, &mut transptable::TranspositionTable) -> Option<f32>;

pub struct GameBB {
    ban : bitboard::BitBoard,
    pub kifu : kifu::Kifu,
    cachesize : usize,
    verbose : bool
}

impl GameBB {
    pub fn new() -> GameBB {
        GameBB {
            ban : bitboard::BitBoard::new(),
            kifu : kifu::Kifu::new(),
            cachesize : 100,
            verbose : true,
        }
    }

    pub fn from(rfen : &str) -> GameBB {
        GameBB {
            ban: bitboard::BitBoard::from(rfen).unwrap(),
            kifu: kifu::Kifu::new(),
            cachesize : 100,
            verbose : true,
        }
    }

    pub fn set_cachesize(&mut self, cachesz : usize) {
        self.cachesize = cachesz;
    }

    pub fn set_verbose(&mut self, vb : bool) {
        self.verbose = vb;
    }

    pub fn is_verbose(&self) -> bool {self.verbose}

    #[allow(dead_code)]
    pub fn start(&mut self, f : SearchFn, depth : u8)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            // think
            let st = Instant::now();
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            if self.is_verbose() {
                println!("val:{val:+5.1} {} {}msec",
                    node.dump(), ft.as_millis());
            }
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    pub fn starto(&mut self, f : SearchFn, depth : u8) -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            // think
            let st = Instant::now();
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();

            let ft = st.elapsed();
            if self.is_verbose() {
                println!("val:{val:+5.1} {} {}msec",
                    node.dump(), ft.as_millis());
            }
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    pub fn startgk(&mut self, f : SearchFn, depth : u8) -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // tt.clear();
            let mut node = nodebb::NodeBB::root(depth);
            // show
            // self.ban.put();
            // println!("{}", self.ban.to_str());
            // think
            // let st = Instant::now();
            let _val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();

            // let ft = st.elapsed();
            // println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_against_stdin(
            &mut self, f : SearchFn, depth : u8, turnin : i8)
                -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                if self.is_verbose() {self.ban.put();}
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.is_verbose() {println!("auto pass.");}
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0] % bitboard::NUMCELL as u8;
                    y = movable[0] / bitboard::NUMCELL as u8;
                } else {
                    loop {
                        if self.is_verbose() {print!("your turn[a1 ~ h8]:");}
                        io::stdout().flush().unwrap();
                        let mut txt = String::new();
                        io::stdin().read_line(&mut txt).unwrap();
                        txt.pop();
                        let xx = "0abcdefgh".find(txt.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                        let yy = txt.chars().nth(1).unwrap().to_digit(10);
                        if yy.is_none() {
                            if self.is_verbose() {
                                println!("invalid position : {txt}");
                            }
                            continue;
                        }
                        let yy = yy.unwrap() as u8;
                        if xx > 8 || yy > 8 {
                            if self.is_verbose() {
                                println!("invalid position : {txt}");
                            }
                            continue;
                        }
                        let pos = bitboard::BitBoard::cell(xx, yy);
                        if movable.contains(&pos) {
                            x = xx;
                            y = yy;
                            break;
                        }
                        if self.is_verbose() {
                            println!("{txt} is not allowed.");
                        }
                    }
                }
            } else {
                if self.is_verbose() {println!("{}", self.ban.to_str());}
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
                // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
                let ft = st.elapsed();
                if self.is_verbose() {
                    println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());
                }
                let best = node.best.unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    pub fn starto_against_stdin(&mut self,
            f : SearchFn, depth : u8, turnin : i8) -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                if self.is_verbose() {self.ban.put();}
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.is_verbose() {println!("auto pass.");}
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0] % bitboard::NUMCELL as u8;
                    y = movable[0] / bitboard::NUMCELL as u8;
                } else {
                    loop {
                        if self.is_verbose() {print!("your turn[a1 ~ h8]:");}
                        io::stdout().flush().unwrap();
                        let mut txt = String::new();
                        io::stdin().read_line(&mut txt).unwrap();
                        txt.pop();
                        let xx = "0abcdefgh".find(txt.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                        let yy = txt.chars().nth(1).unwrap().to_digit(10);
                        if yy.is_none() {
                            if self.is_verbose() {
                                println!("invalid position : {txt}");
                            }
                            continue;
                        }
                        let yy = yy.unwrap() as u8;
                        if xx > 8 || yy > 8 {
                            if self.is_verbose() {
                                println!("invalid position : {txt}");
                            }
                            continue;
                        }
                        let pos = bitboard::BitBoard::cell(xx, yy);
                        if movable.contains(&pos) {
                            x = xx;
                            y = yy;
                            break;
                        }
                        if self.is_verbose() {println!("{txt} is not allowed.");}
                    }
                }
            } else {
                if self.is_verbose() {println!("{}", self.ban.to_str());}
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
                // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
                let ft = st.elapsed();
                if self.is_verbose() {
                    println!("val:{val:+5.1} {} {}msec",
                        node.dump(), ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_against_edax(&mut self,
            f : SearchFn, depth : u8, turnin : i8) -> Result<(), String> {
        let er = edaxrunner::EdaxRunner::new();
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let x;
            let y;
            if self.ban.teban == turnin {
                // show
                if self.is_verbose() {println!("{}", self.ban.to_str());}
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.is_verbose() {println!("auto pass.");}
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0] % bitboard::NUMCELL as u8;
                    y = movable[0] / bitboard::NUMCELL as u8;
                } else {
                    // launch edax
                    match er.run(&self.ban.to_obf()) {
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
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                let ft = st.elapsed();
                if self.is_verbose() {
                    println!("val:{val:+5.1} {} {}msec",
                        node.dump(), ft.as_millis());
                }
                let best = node.best.unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
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
            f : SearchFn, depth : u8, turnin : i8, econf : &str)
                -> Result<(), String> {
        let er = edaxrunner::EdaxRunner::from_config(econf)?;
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        loop {
            // show
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            let x;
            let y;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.is_verbose() {println!("auto pass.");}
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0] % bitboard::NUMCELL as u8;
                    y = movable[0] / bitboard::NUMCELL as u8;
                } else {
                    // launch edax
                    match er.run(&self.ban.to_obf()) {
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
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                let ft = st.elapsed();
                if self.is_verbose() {
                    println!("val:{val:+5.1} {} {}msec",
                        node.dump(), ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    /// play a game against another Ruversi.  
    /// # Arguments  
    /// - f : fn for searching.  
    /// - depth : searching depth.  
    /// - turnin : another Ruversi's turn.  
    /// # Returns  
    /// () or Error message.
    pub fn starto_against_ruversi(&mut self,
        f : SearchFn, depth : u8, turnin : i8, econf : &str)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        let mut rr = edaxrunner::RuversiRunner::from_config(econf)?;
        rr.set_verbose(self.verbose);
        loop {
            // show
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            let x;
            let y;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.is_verbose() {println!("auto pass.");}
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0] % bitboard::NUMCELL as u8;
                    y = movable[0] / bitboard::NUMCELL as u8;
                } else {
                    // launch another Ruversi
                    match rr.run(&self.ban.to_str()) {
                        Ok((pos, _)) => {
                            x = "0abcdefgh".find(pos.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                            y = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                        },
                        Err(msg) => panic!("error running ruversi... [{msg}]"),
                    }
                }
            } else {
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                let ft = st.elapsed();
                if self.is_verbose() {
                    println!("  val:{val:+5.1} {} {}msec",
                        node.dump(), ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
            }
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    /// play a game against Edax via othello engine protocol.  
    /// # Arguments  
    /// - f : fn for searching.  
    /// - depth : searching depth.  
    /// - turnin : Edax's turn.  
    /// # Returns  
    /// () or Error message.
    pub fn start_against_via_cassio(&mut self,
            f : SearchFn, depth : u8, turnin : i8, cconf : &str)
                -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        let er = edaxrunner::CassioRunner::from_config(cconf)?;
        let mut cassio =
            cassio::OthelloEngineProtocolServer::new1(er.run().unwrap());
        cassio.setturn(bitboard::SENTE);
        cassio.init().unwrap();
        if self.is_verbose() {
            println!("opponent:{}", cassio.get_version()?);
        }
        cassio.new_position()?;

        loop {
            // show
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            let x;
            let y;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.is_verbose() {println!("auto pass.");}
                    x = 0;
                    y = 0;
                } else if movable.len() == 1 {
                    x = movable[0] % bitboard::NUMCELL as u8;
                    y = movable[0] / bitboard::NUMCELL as u8;
                } else {
                    // launch edax
                    let alpha = -64f32;
                    let beta = 64f32;
                    // let depth = depth;
                    let precision = 50;
                    match cassio.midgame_search(&self.ban.to_obf(), alpha, beta, depth, precision) {
                        Ok(res) => {
                            if self.is_verbose() {println!("{res}");}
                            let elem = res.split(',').collect::<Vec<_>>();
                            if elem.len() < 2 {panic!("lack of response.. {res}");}

                            let mv = elem[1].split_whitespace().collect::<Vec<_>>();
                            if mv.len() < 2 {panic!("lack of response.. {res}");}

                            match mv[1] {
                            "Pa" => {
                                x = 0;
                                y = 0;
                            },
                            "--" => {
                                x = 255;
                                y = 255;
                            },
                            _ => {
                                x = mv[1].chars().nth(0).unwrap()
                                    .to_ascii_uppercase() as u8 - b'A' + 1;
                                y = mv[1].chars().nth(1).unwrap() as u8 - b'0';
                            },
                            }
                        },
                        Err(ermsg) => {
                            panic!("error occured: {ermsg}!!");
                        },
                    }
                }
            } else {
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                let ft = st.elapsed();
                if self.is_verbose() {
                    println!("val:{val:+5.1} {} {}msec",
                        node.dump(), ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                x = best.x;
                y = best.y;
            }

            if x <= 8 {
                // apply move
                let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
                let rfen = self.ban.to_str();
                let teban = self.ban.teban;
                self.ban = ban;

                // save to kifu
                self.kifu.append(x as usize, y as usize, teban, rfen);
            }

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
        cassio.quit().unwrap();

        // check who won
        self.kifu.winneris(self.ban.count());
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_with_2et(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban.to_str());}
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
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            if self.is_verbose() {
                println!("val:{val:+5.1} {} {}msec",
                    node.dump(), ft.as_millis());
            }
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    pub fn starto_with_2et(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban.to_str());}
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
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            if self.is_verbose() {println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());}
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    pub fn starto_with_2et_mt(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            // switch weight
            let wei;
            let mut node =
                if self.ban.teban == bitboard::SENTE {
                    wei = et1;
                    nodebb::NodeBB::new(0, 0, depth, bitboard::NONE)
                } else {
                    wei = et2;
                    nodebb::NodeBB::new(0, 0, depth, bitboard::NONE)
                };
            // think
            let st = Instant::now();
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
            let ft = st.elapsed();
            if self.is_verbose() {println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());}
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    pub fn starto_with_2et_mt_tt(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        loop {
            // tt.clear();
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban.to_str());}
            // switch weight
            let wei;
            let mut node =
                if self.ban.teban == bitboard::SENTE {
                    wei = et1;
                    nodebb::NodeBB::new(0, 0, depth, bitboard::NONE)
                } else {
                    wei = et2;
                    nodebb::NodeBB::new(0, 0, depth, bitboard::NONE)
                };
            // think
            let st = Instant::now();
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
            let ft = st.elapsed();
            if self.is_verbose() {println!("val:{val:+5.1} {} {}msec", node.dump(), ft.as_millis());}
            let best = node.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(bitboard::BitBoard::cell(x, y)).unwrap();
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
        if self.is_verbose() {println!("{}", self.kifu.to_str());}
        // show
        if self.is_verbose() {self.ban.put();}
        Ok(())
    }
}
