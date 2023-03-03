use rand::prelude::SliceRandom;
use std::sync::Arc;
use super::*;

const BIT_OUT_NONE : u32 = 0x0;
const BIT_OUT_PROGESS : u32 = 0x1;
const BIT_OUT_EXRFENS : u32 = 0x2;
const BIT_OUT_SUMMARY : u32 = 0x4;
const BIT_OUT_TIME : u32 = 0x8;
pub const BIT_OUT_NOSAVE : u32 = 0x10;
pub const BIT_OUT_DEFAULT : u32 =
        BIT_OUT_PROGESS | BIT_OUT_SUMMARY | BIT_OUT_TIME;

pub struct Trainer {
    eta: f32,
    repeat: usize,
    path: String,
    pub nfiles: usize,
    pub total: i32,
    pub win: i32,
    pub draw: i32,
    pub lose: i32,
    pub output: u32,
}

impl Trainer {
    pub fn new(eta: f32, repeat: usize, path: &str) -> Trainer {
        Trainer {
            eta: eta,
            repeat: repeat,
            path: String::from(path),
            nfiles: 0,
            total: 0,
            win: 0,
            draw: 0,
            lose: 0,
            output: BIT_OUT_DEFAULT,
        }
    }

    /// read comma separated options to control outputs.
    ///
    /// - `txt` : COMMA separated option.
    ///   - exrfens  : put RFENs in 2nd moves in every kifus.
    ///   - nosave   : skip saving weights.
    ///   - progress : show progress
    ///   - summary  : show play summary.
    ///   - time     : show processing time.
    ///   - default: progress,summary,time
    pub fn read_opt_out(&mut self, txt : &str) {
        if txt.is_empty() {return;}

        let slist : Vec<_> = txt.split(',').collect();
        let mut res = BIT_OUT_NONE;
        for &opt in slist.iter() {
            let lopt = opt.to_ascii_lowercase();
            match lopt.as_str() {
                "exrfens" => { res |= BIT_OUT_EXRFENS },
                "nosave" => { res |= BIT_OUT_NOSAVE },
                "progress" => { res |= BIT_OUT_PROGESS },
                "summary" => { res |= BIT_OUT_SUMMARY },
                "time" => { res |= BIT_OUT_TIME },
                _ => { panic!("unknown option: \"{opt}\"")}
            }
        }
        // println!("res:{res:x}");
        self.output = res;
    }

    pub fn need_exrfens(&self) -> bool {
        (self.output & BIT_OUT_EXRFENS) != 0
    }

    pub fn need_progress(&self) -> bool {
        (self.output & BIT_OUT_PROGESS) != 0
    }

    pub fn need_summay(&self) -> bool {
        (self.output & BIT_OUT_SUMMARY) != 0
    }

    #[allow(dead_code)]
    pub fn noneed_save(&self) -> bool {
        (self.output & BIT_OUT_NOSAVE) != 0
    }

    pub fn need_save(&self) -> bool {
        (self.output & BIT_OUT_NOSAVE) == 0
    }

    pub fn need_time(&self) -> bool {
        (self.output & BIT_OUT_TIME) != 0
    }

    pub fn fmt_result(&self) -> String {
        format!("total,{},win,{},draw,{},lose,{}",
                self.total, self.win, self.draw, self.lose)
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn run4win(&self, kifu: &kifu::Kifu, weight: &mut weight::Weight) -> Result<(), String> {
        let winner = kifu.winner();
        if winner.is_none() {
            return Err(String::from("invalid kifu."));
        }
        let winner = winner.unwrap();
        for l in kifu.list.iter() {
            if weight.train(&l.rfen, winner, self.eta, 10).is_err() {
                return Err(String::from("error while training"));
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
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
        let showprgs = self.need_progress();
        let mut rng = rand::thread_rng();
        let mut kifucache : Vec<(String, kifu::Kifu)> = Vec::new();
        for i in 0..1 {
            for fname in files.iter() {
                let path = format!("kifu/{}", fname);
                if showprgs {print!("{} / {} : {}\r", i, self.repeat, path);}
                let content = std::fs::read_to_string(&path).unwrap();
                let lines:Vec<&str> = content.split("\n").collect();
                let kifu = kifu::Kifu::from(&lines);
                let p = String::from(&path);
                kifucache.push((p, kifu.copy()));
                unsafe {
                    match if cfg!(feature="bitboard") {
                            self.run4stones(&kifu, &mut nodebb::WEIGHT.as_mut().unwrap())
                        } else {
                            self.run4stones(&kifu, &mut node::WEIGHT.as_mut().unwrap())
                        } {
                        Err(msg) => {panic!("{}", msg);},
                        _ => {}
                    }
                }
            }
            if showprgs {println!("");}
        }
        let n = files.len();
        let mut numbers : Vec<usize> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        for (i, it) in numbers.iter_mut().enumerate() {*it = i;}
        for i in 1..self.repeat {
            numbers.shuffle(&mut rng);
            if showprgs {print!("{} / {}\r", i, self.repeat);}
            for idx in numbers.iter() {
                let (_path, kifu) = kifucache.iter().nth(*idx).unwrap();
                unsafe {
                    match if cfg!(feature="bitboard") {
                        self.run4stones(
                            &kifu, &mut nodebb::WEIGHT.as_mut().unwrap())
                        } else {
                            self.run4stones(
                                &kifu, &mut node::WEIGHT.as_mut().unwrap())
                        } {
                        Err(msg) => {panic!("{}", msg);},
                        _ => {}
                    }
                }
            }
            if showprgs {println!("");}
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
            if weight.train(&l.rfen, score, self.eta, 10).is_err() {
                return Err(String::from("error while training"));
            }
        }
        Ok(())
    }

    /**
     * 棋譜の読み込みと学習を別スレでやる版
     */
    pub fn learn_stones_para(&mut self) {
        let (tosub, frmain) = std::sync::mpsc::channel::<Arc<kifu::Kifu>>();
        let (tomain, frsub) = std::sync::mpsc::channel::<()>();
        // let rpt = self.repeat;
        let eta = self.eta;

        let sub = std::thread::spawn(move || {
            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};

            tomain.send(()).unwrap();
            let mut i = 0;
            let mut etai = eta;
            loop {
                match frmain.recv() {
                    Ok(kifu) => {
                        // print!("{rfen:?},{score} \r");
                        if kifu.is_none() {
                            i += 1;
                            etai = eta * 10.0 / (10.0 + i as f32);
                            tomain.send(()).unwrap();
                            continue;
                        }
                        if kifu.is_invalid() {
                            // println!("score > 64");
                            break;
                        }
                        for te in kifu.list.iter() {
                            let rfen = &te.rfen;
                            let score = kifu.score.unwrap();
                            if weight.train(rfen, score, etai, 0).is_err() {
                                println!("error while training");
                                break;
                            }
                        }
                    },
                    Err(e) => {panic!("{}", e.to_string())}
                }
            }
        });

        // list up kifu
        let files = std::fs::read_dir(&self.path).unwrap();
        let mut files = files.filter_map(|entry| {
            entry.ok().and_then(|e|
                e.path().file_name().and_then(|n|
                    n.to_str().map(|s| String::from(s))
                )
            )}).collect::<Vec<String>>().iter().filter(|&fnm| {
                fnm.find("kifu").is_some()
                // fnm.find(".txt").is_some()
            }).cloned().collect::<Vec<String>>();
        // println!("{:?}", files);

        self.nfiles = files.len();
        files.sort();

        let showprgs = self.need_progress();
        let mut rng = rand::thread_rng();
        let mut kifucache : Vec<Arc<kifu::Kifu>> = Vec::new();
        for fname in files.iter() {
            let path = format!("{}{}", self.path, fname);
            if showprgs {print!("0 / {} : {}\r", self.repeat, path);}

            let content = std::fs::read_to_string(&path).unwrap();
            let lines:Vec<&str> = content.split("\n").collect();

            let kifu = Arc::new(kifu::Kifu::from(&lines));
            match tosub.send(kifu.clone()) {
                Ok(_) => {
                    kifucache.push(kifu.clone());
                },
                Err(e) => {
                    panic!("{}", e.to_string());
                },
            }

            self.total += 1;
            match kifu.winner().unwrap() {
                kifu::SENTEWIN => {self.win += 1;},
                kifu::DRAW => {self.draw += 1;},
                kifu::GOTEWIN => {self.lose += 1;},
                _ => {}
            }
        }
        let sep = Arc::new(kifu::Kifu::new());
        match tosub.send(sep.clone()) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }
        let _ = frsub.recv().unwrap();
        if showprgs {println!("");}

        let n = kifucache.len();
        // println!("{n} rfens.");
        let mut numbers : Vec<usize> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        // for (i, it) in numbers.iter_mut().enumerate() {
        //     *it = i;
        // }
        for i in 0..n {
            numbers[i] = i;
        }
        for i in 1..self.repeat {
            if showprgs {
                print!("{i} / {}\r", self.repeat);
                std::io::stdout().flush().unwrap();
            }
            numbers.shuffle(&mut rng);
            for &idx in numbers.iter() {
                match tosub.send(kifucache[idx].clone()) {
                    Ok(_) => {},
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
            }
            match tosub.send(sep.clone()) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e.to_string());
                },
            }
            frsub.recv().unwrap();
        }
        if showprgs {println!("");}
        // println!("_ _ _");
        let stop = Arc::new(kifu::Kifu::invalid());
        match tosub.send(stop) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }

        sub.join().unwrap();
    }

    /**
     * 棋譜の読み込みと学習を別スレでやる版
     */
    #[allow(dead_code)]
    pub fn learn_stones_para_rfen(&mut self) {
        let (tosub, frmain) = std::sync::mpsc::channel::<(String, i8)>();
        let (tomain, frsub) = std::sync::mpsc::channel::<()>();
        let eta = self.eta;

        let sub = std::thread::spawn(move || {
            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};

            tomain.send(()).unwrap();

            loop {
                match frmain.recv() {
                    Ok( (rfen, score) ) => {
                        // print!("{rfen:?},{score} \r");
                        if rfen.is_empty() {
                            tomain.send(()).unwrap();
                            continue;
                        }
                        if score > 64 {
                            // println!("score > 64");
                            break;
                        }
                        //
                        if weight.train(&rfen, score, eta, 10).is_err() {
                            println!("error while training");
                            break;
                        }
                    },
                    Err(e) => {panic!("{}", e.to_string())}
                }
            }
        });

        // list up kifu
        let files = std::fs::read_dir(&self.path).unwrap();
        let mut files = files.filter_map(|entry| {
            entry.ok().and_then(|e|
                e.path().file_name().and_then(|n|
                    n.to_str().map(|s| String::from(s))
                )
            )}).collect::<Vec<String>>().iter().filter(|&fnm| {
                fnm.find("kifu").is_some()
                // fnm.find(".txt").is_some()
            }).cloned().collect::<Vec<String>>();
        // println!("{:?}", files);

        self.nfiles = files.len();
        files.sort();

        let showprgs = self.need_progress();
        let mut rng = rand::thread_rng();
        let mut rfencache : Vec<(String, i8)> = Vec::new();
        for fname in files.iter() {
            let path = format!("{}{}", self.path, fname);
            if showprgs {print!("0 / {} : {}\r", self.repeat, path);}

            let content = std::fs::read_to_string(&path).unwrap();
            let lines:Vec<&str> = content.split("\n").collect();

            let kifu = kifu::Kifu::from(&lines);
            let score = kifu.score.unwrap();
            for te in kifu.list.iter() {
                let rfen = te.rfen.clone();
                let rfen2 = rfen.clone();
                match tosub.send((rfen, score)) {
                    Ok(_) => {
                        rfencache.push((rfen2, score));
                    }
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
            }

            self.total += 1;
            match kifu.winner().unwrap() {
                kifu::SENTEWIN => {self.win += 1;},
                kifu::DRAW => {self.draw += 1;},
                kifu::GOTEWIN => {self.lose += 1;},
                _ => {}
            }
        }
        match tosub.send((String::new(), 0)) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }
        let _ = frsub.recv().unwrap();
        if showprgs {println!("");}

        let n = rfencache.len();
        // println!("{n} rfens.");
        let mut numbers : Vec<usize> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        // for (i, it) in numbers.iter_mut().enumerate() {
        //     *it = i;
        // }
        for i in 0..n {
            numbers[i] = i;
        }
        for i in 1..self.repeat {
            if showprgs {print!("{i} / {}", self.repeat);}
            numbers.shuffle(&mut rng);
            for idx in numbers.iter() {
                let (rfen, score) = rfencache.iter().nth(*idx).unwrap();
                match tosub.send((rfen.clone(), *score)) {
                    Ok(_) => {},
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
            }
            match tosub.send((String::new(), 0)) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e.to_string());
                },
            }
            frsub.recv().unwrap();
            if showprgs {println!("");}
        }
        // println!("_ _ _");
        match tosub.send((String::from("stop"), 100)) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }

        sub.join().unwrap();
    }

    /**
     * 棋譜の読み込みと学習を別スレでやる版
     * rfenを纏めてやり取りする版
     */
    #[allow(dead_code)]
    pub fn learn_stones_para_rfengrp(&mut self) {
        let (tosub, frmain) = std::sync::mpsc::channel::<Vec<(String, i8)>>();
        let (tomain, frsub) = std::sync::mpsc::channel::<()>();
        let eta = self.eta;

        let sub = std::thread::spawn(move || {
            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};

            tomain.send(()).unwrap();

            loop {
                match frmain.recv() {
                    Ok( rfengrp ) => {
                        // print!("{rfen:?},{score} \r");
                        if rfengrp.is_empty() {
                            tomain.send(()).unwrap();
                            continue;
                        }
                        let score = rfengrp[0].1;
                        if score > 64 {
                            // println!("score > 64");
                            break;
                        }
                        //
                        for (rfen, score) in rfengrp.iter() {
                            if weight.train(rfen, *score, eta, 0).is_err() {
                                println!("error while training");
                                break;
                            }
                        }
                    },
                    Err(e) => {panic!("{}", e.to_string())}
                }
            }
        });

        // list up kifu
        let files = std::fs::read_dir(&self.path).unwrap();
        let mut files = files.filter_map(|entry| {
            entry.ok().and_then(|e|
                e.path().file_name().and_then(|n|
                    n.to_str().map(|s| String::from(s))
                )
            )}).collect::<Vec<String>>().iter().filter(|&fnm| {
                fnm.find("kifu").is_some()
                // fnm.find(".txt").is_some()
            }).cloned().collect::<Vec<String>>();
        // println!("{:?}", files);

        self.nfiles = files.len();
        files.sort();

        let showprgs = self.need_progress();
        let mut rng = rand::thread_rng();
        let mut rfencache : Vec<(String, i8)> = Vec::new();
        for fname in files.iter() {
            let path = format!("{}{}", self.path, fname);
            if showprgs {print!("reading {path}\r");}

            let content = std::fs::read_to_string(&path).unwrap();
            let lines:Vec<&str> = content.split("\n").collect();

            let kifu = kifu::Kifu::from(&lines);
            let score = kifu.score.unwrap();
            for te in kifu.list.iter() {
                let rfen = te.rfen.clone();
                let rfen2 = rfen.clone();
                rfencache.push((rfen2, score));
            }

            self.total += 1;
            match kifu.winner().unwrap() {
                kifu::SENTEWIN => {self.win += 1;},
                kifu::DRAW => {self.draw += 1;},
                kifu::GOTEWIN => {self.lose += 1;},
                _ => {}
            }
        }
        match tosub.send(Vec::new()) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }
        let _ = frsub.recv().unwrap();
        if showprgs {println!("");}

        let n = rfencache.len();
        // println!("{n} rfens.");
        let mut numbers : Vec<usize> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        // for (i, it) in numbers.iter_mut().enumerate() {
        //     *it = i;
        // }
        for i in 0..n {
            numbers[i] = i;
        }
        for i in 0..self.repeat {
            if showprgs {
                print!("{i} / {}", self.repeat);
                std::io::stdout().flush().unwrap();
            }
            numbers.shuffle(&mut rng);
            for idx in 0..10 {
                let mut grp = Vec::new();
                for i in numbers[idx * n / 10..(idx + 1) * n / 10 - 1].iter() {
                    grp.push(rfencache.iter().nth(*i).unwrap().clone());
                }
                match tosub.send(grp) {
                    Ok(_) => {},
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
            }
            match tosub.send(Vec::new()) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e.to_string());
                },
            }
            frsub.recv().unwrap();
            if showprgs {println!("");}
        }
        // println!("_ _ _");
        match tosub.send(vec![(String::from("stop"), 100)]) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }

        sub.join().unwrap();
    }
}
