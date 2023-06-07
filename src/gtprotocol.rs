use super::*;
use std::fs::OpenOptions;
// use std::io::{self, Write};

fn not_implemented_yet() {
    unimplemented!()
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(PartialEq)]
enum Status {
    Play,
    Quit,
    Error,
}

pub struct GTP {
    cmd : String,
    resp : String,
    status : Status,
    emsg : String,
    ban : bitboard::BitBoard,
    ev: String,
}

impl GTP {
    pub fn new() -> GTP {
        GTP {
            cmd : String::new(),
            resp : String::new(),
            status : Status::Play,
            emsg : String::new(),
            ban : bitboard::BitBoard::new(),
            ev : String::new(),
        }
    }

    pub fn is_quit(&self) -> bool {
        self.status == Status::Quit
    }

    pub fn is_error(&self) -> bool {
        self.status == Status::Error
    }

    pub fn start(&mut self, path : &str) -> Result<String, String> {

        let log = OpenOptions::new()
                .write(true).append(true).open("/tmp/ruversi.log");
        if log.is_err() {
            return Err(log.err().unwrap().to_string());
        }

        let mut log = log.unwrap();
        log.write(format!("started!!!\n").as_bytes()).unwrap();

        self.ev = String::from(path);

        loop {
            let mut txt = String::new();
            std::io::stdin().read_line(&mut txt).unwrap();
            if txt.is_empty() {continue;}

            log.write(format!("in:{txt}").as_bytes()).unwrap();
            self.read(&txt.trim_end());
            log.write(format!("out:{}\n", self.resp).as_bytes()).unwrap();
            if self.is_error() {
                log.write(format!("ERROR: {}\n", self.emsg).as_bytes()).unwrap();
                return Err(format!("{}", self.emsg));
            }

            if self.is_quit() {
                break;
            }
        }
        Ok(String::from("Done."))
    }

    fn readpos(xypos : &str) -> Option<(u8, u8)> {
        let x = xypos.chars().nth(0).unwrap() as u8 - 'A' as u8 + 1;
        let y = xypos.chars().nth(1).unwrap() as u8 - '0' as u8;

        Some((x, y))
    }

    pub fn read(&mut self, line : &str) {
        if line.is_empty() {return;}
        self.cmd = String::from(line);
        let elem : Vec<&str> = line.split(" ").collect();
        // N command arg1 arg2 ...
        let id;
        let idx;
        if elem.len() == 1 {
            id = "";
            idx = 0;
        } else {
            if elem[0].chars().nth(0).unwrap().is_ascii_digit() {
                id = elem[0];
                idx = 1;
            } else {
                id = "";
                idx = 0;
            }
        }
        let cmd = elem[idx];
        // eprintln!("id:{id}, cmd:{cmd}");

        match cmd {
            "boardsize" => {
                // should be 8
                if elem[idx + 1] != "8" {
                    self.status = Status::Quit;
                    self.emsg = format!("error: unknown command [{line}].");
                    self.ng_respond(id);
                } else {
                    self.respond(id);
                }
            },
            "clear_board" => {
// not_implemented_yet();
                self.ban = bitboard::BitBoard::from(
                    "8/8/8/3aA3/3Aa3/8/8/8 b").unwrap();
                self.respond(id);
            },
            "genmove" => {
                //4 genmove white
                // =4 C3
                let color = match elem[idx + 1] {
                    "black" => {board::SENTE},
                    "white" => {board::GOTE},
                    _ => {
                        self.status = Status::Error;
                        self.emsg = format!("error: unknown command [{line}].");
                        return;
                    }
                };
                if color != self.ban.teban {
                    match self.ban.genmove() {
                        Some(a) => {
                            if a.is_empty() {
                                self.ban.pass();
                            } else {    
                                self.status = Status::Error;
                                self.emsg = format!("error: turn mismatch.");
                                self.ng_respond(id);
                                return;
                            }
                        },
                        _ => {
                            self.status = Status::Error;
                            self.emsg = format!("error: no blank cells to play.");
                            self.ng_respond(id);
                            return;
                    }
                    }
                }
                let depth = 7;
                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::thinko_ab_extract2(&self.ban, depth).unwrap();
                let ft = st.elapsed();
                eprintln!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
                let best = node.best.as_ref().unwrap();
                let xy = best.pos();
                if xy == "00" {
                    self.ban.pass();
                    self.respond1(id, "pass");
                } else {
                    self.ban = self.ban.r#move(best.x, best.y).unwrap();
                    self.respond1(id, &xy);
                };
            },
            "komi" => {
                self.respond(id);
            }
            "list_commands" => {
                let supported = vec!["protocol_version", "name", "version",
                    /*"known_command", */"list_commands", "quit", "boardsize",
                    "clear_board", "komi", "play", "genmove", /*"undo",*/
                    /*"time_settings", "time_left", */"set_game", "list_games",
                    /*"loadsgf", "reg_genmove", "showboard"*/];
                self.respond1(id, &supported.join("\n"));
            }
            "list_games" => {
                self.respond1(id, "Othello");
            }
            "name" => {
                self.respond1(id, "ruversi");
            },
            "play" => {
                // black D5
                let color = match elem[idx + 1] {
                    "black" => {board::SENTE},
                    "white" => {board::GOTE},
                    _ => {
                        self.status = Status::Error;
                        self.emsg = format!("error: unknown command [{line}].");
                        return;
                    }
                };
                if color != self.ban.teban {
                    match self.ban.genmove() {
                        Some(a) => {
                            if a.is_empty() {
                                self.ban.pass();
                            } else {
                                self.status = Status::Error;
                                self.emsg = format!("error: turn mismatch.");
                                return;
                            }
                        },
                        _ => {
                            self.status = Status::Error;
                            self.emsg = format!("error: unknown command [{line}].");
                            return;
                        }
                    }
                }
                let (x, y);
                match GTP::readpos(elem[idx + 2]) {
                    Some(xy) => {
                        (x, y) = xy;
                        match self.ban.r#move(x, y) {
                            Err(msg) => {
                                self.status = Status::Error;
                                self.emsg = String::from(msg) + &self.ban.to_str();
                            },
                            Ok(b) => {self.ban = b;}
                        }
                    },
                    _ => {
                        self.status = Status::Error;
                        self.emsg = format!("error: unknown command [{line}].");
                        return;
                    }
                }
                self.respond(id);
            },
            "protocol_version" => {
                self.respond1("", "2");
            }
            "quit" => {
                self.status = Status::Quit;
                self.respond(id);
            },
            "set_game" => {
                if elem[idx + 1] == "Othello" {
                    self.respond(id);
                    if cfg!(feature="bitboard") {
                        nodebb::init_weight();
                    } else {
                        node::init_weight();
                    }
                    unsafe {
                        match nodebb::WEIGHT.as_mut().unwrap().read(&self.ev) {
                            Err(emsg) => {
                                self.status = Status::Error;
                                self.emsg = format!("{emsg} {}", self.ev);
                                self.ng_respond(id);
                                return;
                            },
                            _ => {
                                // self.emsg = format!("read eval: {}.", &self.ev);
                            }
                        }
                    }
                } else {
                    self.ng_respond(id)
                }
            },
            "version" => {
                self.respond1(id, VERSION);
            },
            _ => {
                self.status = Status::Error;
                self.emsg = format!("error: unknown command [{line}].");
            }
        }
   }

    pub fn respond(&mut self, id : &str) {
        let resp = format!("= {id} ");
        println!("{resp}\n");
        self.resp = resp;
    }

    pub fn respond1(&mut self, id : &str, arg : &str) {
        let resp = format!("= {id} {arg}");
        println!("{resp}\n");
        self.resp = resp;
    }

    pub fn ng_respond(&mut self, id : &str) {
        let resp = format!("? {id} ");
        println!("{resp}\n");
        self.resp = resp;
    }

    pub fn ng_respond1(&mut self, id : &str, arg : &str) {
        let resp = format!("? {id} {arg}");
        println!("{resp}\n");
        self.resp = resp;
    }
}