use super::*;

use std::io::{self, Write};
// use std::sync::{Arc, RwLock};

type SearchFn = fn(&bitboard::BitBoard, u8, &mut nodebb::NodeBB, &weight::Weight, &mut transptable::TranspositionTable) -> Option<f32>;

pub struct GameBB {
    ban : bitboard::BitBoard,
    pub kifu : kifu::Kifu,
    cachesize : usize,
    verbose : myoption::Verbose
}

impl GameBB {
    pub fn new() -> GameBB {
        GameBB {
            ban : bitboard::BitBoard::new(),
            kifu : kifu::Kifu::new(),
            cachesize : 100,
            verbose : myoption::Verbose::Normal,
        }
    }

    pub fn from(rfen : &str) -> GameBB {
        GameBB {
            ban: bitboard::BitBoard::from(rfen).unwrap(),
            kifu: kifu::Kifu::new(),
            cachesize : 100,
            verbose : myoption::Verbose::Normal,
        }
    }

    pub fn set_cachesize(&mut self, cachesz : usize) {
        self.cachesize = cachesz;
    }

    pub fn set_verbose(&mut self, vb : &myoption::Verbose) {
        self.verbose = vb.clone();
    }

    pub fn is_verbose(&self) -> bool {self.verbose == myoption::Verbose::Full}
    pub fn not_silent(&self) -> bool {self.verbose != myoption::Verbose::Silent}

    #[allow(dead_code)]
    pub fn start(&mut self, f : SearchFn, depth : u8)
            -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban);}
            // think
            let st = Instant::now();
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            if self.not_silent() {
                println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            }
            let best = node.best.unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn starto(&mut self, f : SearchFn, depth : u8) -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban);}
            // think
            let st = Instant::now();
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();

            let ft = st.elapsed();
            if self.not_silent() {
                println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            }
            let best = node.best.as_ref().unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    pub fn startgk(&mut self, f : SearchFn, depth : u8) -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let mut node = nodebb::NodeBB::root(depth);
            // show
            // self.ban.put();
            // println!("{}", self.ban);
            // think
            // let st = Instant::now();
            let _val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();

            // let ft = st.elapsed();
            // println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            let best = node.best.as_ref().unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_against_stdin(
            &mut self, f : SearchFn, depth : u8, turnin : i8)
                -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let xy;
            if self.ban.teban == turnin {
                // show
                if self.is_verbose() {self.ban.put();}
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.not_silent() {println!("auto pass.");}
                    xy = bitboard::PASS;
                } else if movable.len() == 1 {
                    xy = if movable[0] == bitboard::PASS {  // pass
                        bitboard::PASS
                    } else {
                        movable[0]
                    };
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
                        let pos = bitboard::cell(xx, yy);
                        if movable.contains(&pos) {
                            xy = pos;
                            break;
                        }
                        if self.is_verbose() {
                            println!("{txt} is not allowed.");
                        }
                    }
                }
            } else {
                if self.is_verbose() {println!("{}", self.ban);}
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
                // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
                let ft = st.elapsed();
                if self.not_silent() {
                    println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
                }
                let best = node.best.unwrap();
                xy = best.xypos();
            }
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    pub fn starto_against_stdin(&mut self,
            f : SearchFn, depth : u8, turnin : i8) -> Result<(), String> {
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let xy;
            if self.ban.teban == turnin {
                // show
                if self.is_verbose() {self.ban.put();}
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.not_silent() {println!("auto pass.");}
                    xy = bitboard::PASS;
                } else if movable.len() == 1 {
                    xy = movable[0];
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
                        let pos = bitboard::cell(xx, yy);
                        if movable.contains(&pos) {
                            xy = pos;
                            break;
                        }
                        if self.is_verbose() {println!("{txt} is not allowed.");}
                    }
                }
            } else {
                if self.is_verbose() {println!("{}", self.ban);}
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
                // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
                let ft = st.elapsed();
                if self.not_silent() {
                    println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                xy = best.xypos();
            }
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_against_edax(&mut self,
            f : SearchFn, depth : u8, turnin : i8) -> Result<(), String> {
        let er = edaxrunner::EdaxRunner::new();
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            let xy;
            if self.ban.teban == turnin {
                // show
                if self.is_verbose() {println!("{}", self.ban);}
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.not_silent() {println!("auto pass.");}
                    xy = bitboard::PASS;
                } else if movable.len() == 1 {
                    xy = movable[0];
                } else {
                    // launch edax
                    match er.run(&self.ban.to_obf()) {
                        Ok((pos, _)) => {
                            let xx = "0abcdefgh".find(pos.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                            let yy = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                            xy = bitboard::cell(xx, yy);
                        },
                        Err(msg) => panic!("error running edax... [{msg}]"),
                    }
                }
           } else {
                println!("{}", self.ban);
                // think
                let st = Instant::now();
                let mut node = nodebb::NodeBB::root(depth);
                let val = f(&self.ban, depth, &mut node, wei, &mut tt).unwrap();
                let ft = st.elapsed();
                if self.not_silent() {
                    println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
                }
                let best = node.best.unwrap();
                xy = best.xypos();
            }
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
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
            f : SearchFn, depth : u8, turnin : i8, econf : &std::path::PathBuf)
                -> Result<(), String> {
        let er = edaxrunner::EdaxRunner::from_config(econf)?;
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        let mut tt = transptable::TranspositionTable::with_capacity(self.cachesize);
        loop {
            // show
            if self.is_verbose() {println!("{}", self.ban);}
            let xy;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.not_silent() {println!("auto pass.");}
                    xy = bitboard::PASS;
                } else if movable.len() == 1 {
                    xy = movable[0];
                } else {
                    // launch edax
                    match er.run(&self.ban.to_obf()) {
                        Ok((pos, _)) => {
                            let xx = "0abcdefgh".find(pos.chars().nth(0).unwrap()).unwrap_or(10) as u8;
                            let yy = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                            xy = bitboard::cell(xx, yy);
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
                if self.not_silent() {
                    println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                xy = best.xypos();
            }
            // apply move
            let ban = match self.ban.r#move(xy) {
                Err(msg) => {
                    self.ban.put();
                    println!("rfen:{}", self.ban);
                    panic!("{msg} @ {xy}");
                },
                Ok(ban) => {ban},
            };
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
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
        let mut rr = edaxrunner::RuversiRunner::from_config(
                &std::path::PathBuf::from(econf))?;
        rr.set_verbose(self.is_verbose());
        loop {
            // show
            if self.is_verbose() {println!("{}", self.ban);}
            let xy;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.not_silent() {println!("auto pass.");}
                    xy = bitboard::PASS;
                } else if movable.len() == 1 {
                    xy = if movable[0] == bitboard::PASS {
                        if self.not_silent() {println!("auto pass.");}
                        bitboard::PASS
                    } else {
                        movable[0]
                    };
                } else {
                    // launch another Ruversi
                    match rr.run(&self.ban.to_string()) {
                        Ok((pos, _)) => {
                            let xx = "0abcdefgh".find(
                                    pos.chars().nth(0).unwrap()
                                ).unwrap_or(10) as u8;
                            let yy = pos.chars().nth(1).unwrap()
                                    .to_digit(10).unwrap() as u8;
                            xy = bitboard::cell(xx, yy);
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
                if self.not_silent() {
                    println!("  val:{val:+5.1} {node} {}msec", ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                xy = best.xypos();
            }
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
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
        let er = edaxrunner::CassioRunner::from_config(
                &std::path::PathBuf::from(cconf))?;
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
            if self.is_verbose() {println!("{}", self.ban);}
            let xy;
            if self.ban.teban == turnin {
                // self.ban.put();
                let movable = self.ban.genmove().unwrap();
                if movable.is_empty() {
                    if self.not_silent() {println!("auto pass.");}
                    xy = bitboard::PASS;
                } else if movable.len() == 1 {
                    xy = movable[0];
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
                                xy = bitboard::PASS;
                            },
                            "--" => {
                                xy = bitboard::NONE as u8;
                            },
                            _ => {
                                let xx = mv[1].chars().nth(0).unwrap()
                                    .to_ascii_uppercase() as u8 - b'A' + 1;
                                let yy = mv[1].chars().nth(1).unwrap() as u8 - b'0';
                                xy = bitboard::cell(xx, yy);
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
                if self.not_silent() {
                    println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
                }
                let best = node.best.as_ref().unwrap();
                xy = best.xypos();
            }

            if xy != bitboard::NONE as u8 {
                // apply move
                let ban = self.ban.r#move(xy).unwrap();
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.ban = ban;

                // save to kifu
                self.kifu.append(xy, teban, rfen);
            }

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        cassio.quit().unwrap();

        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_with_2et(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt1 = transptable::TranspositionTable::with_capacity(self.cachesize);
        let mut tt2 = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban);}
            // switch weight
            let tt;
            if self.ban.teban == bitboard::SENTE {
                tt = &mut tt1;
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et1);
                }
            } else {
                tt = &mut tt2;
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et2);
                }
            }
            // think
            let st = Instant::now();
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, tt).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            if self.not_silent() {
                println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            }
            let best = node.best.unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    pub fn starto_with_2et(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt1 = transptable::TranspositionTable::with_capacity(self.cachesize);
        let mut tt2 = transptable::TranspositionTable::with_capacity(self.cachesize);
        let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban);}
            // switch weight and tt
            let tt;
            if self.ban.teban == bitboard::SENTE {
                tt = &mut tt1;
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et1);
                }
            } else {
                tt = &mut tt2;
                unsafe {
                    nodebb::WEIGHT.as_mut().unwrap().copy(et2);
                }
            }
            // think
            let st = Instant::now();
            let mut node = nodebb::NodeBB::root(depth);
            let val = f(&self.ban, depth, &mut node, wei, tt).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
            let ft = st.elapsed();
            if self.not_silent() {
                println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            }
            let best = node.best.as_ref().unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    #[allow(dead_code)]
    pub fn starto_with_2et_mt(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt1 = transptable::TranspositionTable::with_capacity(self.cachesize);
        let mut tt2 = transptable::TranspositionTable::with_capacity(self.cachesize);
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban);}
            // switch weight
            let mut node = nodebb::NodeBB::root(depth);
            let tt;
            let wei = if self.ban.teban == bitboard::SENTE {
                    tt = &mut tt1;
                    et1
                } else {
                    tt = &mut tt2;
                    et2
                };
            // think
            let st = Instant::now();
            let val = f(&self.ban, depth, &mut node, wei, tt).unwrap();
            let ft = st.elapsed();
            if self.not_silent() {
                println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            }
            let best = node.best.as_ref().unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }

    /// # Arguments
    /// - et1 : SENTE
    /// - et2 : GOTE
    pub fn starto_with_2et_mt_tt(&mut self,
        f : SearchFn, depth : u8, et1 : &weight::Weight, et2 : &weight::Weight)
            -> Result<(), String> {
        let mut tt1 = transptable::TranspositionTable::with_capacity(self.cachesize);
        let mut tt2 = transptable::TranspositionTable::with_capacity(self.cachesize);
        loop {
            // show
            // self.ban.put();
            if self.is_verbose() {println!("{}", self.ban);}
            // switch weight
            let mut node = nodebb::NodeBB::root(depth);
            let tt;
            let wei = if self.ban.teban == bitboard::SENTE {
                    tt = &mut tt1;
                    et1
                } else {
                    tt = &mut tt2;
                    et2
                };
            // think
            let st = Instant::now();
            let val = f(&self.ban, depth, &mut node, wei, tt).unwrap();
            let ft = st.elapsed();
            if self.not_silent() {
                println!("val:{val:+5.1} {node} {}msec", ft.as_millis());
            }
            let best = node.best.as_ref().unwrap();
            let xy = best.xypos();
            // apply move
            let ban = self.ban.r#move(xy).unwrap();
            let rfen = self.ban.to_string();
            let teban = self.ban.teban;
            self.ban = ban;

            // save to kifu
            self.kifu.append(xy, teban, rfen);

            // check finished
            if self.ban.is_passpass() {
                break;
            }
            if self.ban.is_full() {
                let rfen = self.ban.to_string();
                let teban = self.ban.teban;
                self.kifu.append(bitboard::PASS, teban, rfen);
                break;
            }

            tt.next();
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        if self.not_silent() {println!("{}", self.kifu.to_str());}
        // show
        if self.not_silent() {self.ban.put();}
        Ok(())
    }
}
