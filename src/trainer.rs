use rand::prelude::SliceRandom;
use std::{sync::Arc, collections::VecDeque};
use super::*;

const BIT_OUT_NONE : u32 = 0x0;
const BIT_OUT_PROGESS : u32 = 0x1;
const BIT_OUT_EXRFENS : u32 = 0x2;
const BIT_OUT_SUMMARY : u32 = 0x4;
const BIT_OUT_TIME : u32 = 0x8;
pub const BIT_OUT_NOSAVE : u32 = 0x10;
pub const BIT_OUT_DEFAULT : u32 =
        BIT_OUT_PROGESS | BIT_OUT_SUMMARY | BIT_OUT_TIME;
static mut RFENCACHE : Vec<(String, i8)> = Vec::new();
static mut BOARDCACHE : Vec<(bitboard::BitBoard, i8)> = Vec::new();
const STOP : u32 = 0xffffffff_u32;
const PROGRESS : u32 = 0xfffffffe_u32;

pub struct Trainer {
    eta: f32,  // 学習率
    repeat: usize,  // 学習回数
    path: String,  // 評価関数ファイルパス
    progress: Vec<u32>,  // 途中経過出力回数
    pub nfiles: usize,  // 棋譜ファイル数
    pub total: i32,  // 棋譜ファイル数
    pub win: i32,  // 先手勝ち数
    pub draw: i32,  // 引き分け数
    pub lose: i32,  // 先手負け数
    pub output: u32,  // 出力設定
}

impl Trainer {
    pub fn new(eta: f32, repeat: usize, path: &str) -> Trainer {
        Trainer {
            eta: eta,
            repeat: repeat,
            path: String::from(path),
            progress: Vec::new(),
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

    pub fn set_progress(&mut self, prgs : &Vec<u32>) {
        self.progress = prgs.to_vec();
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
    #[allow(dead_code)]
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
                        if weight.train_rotate(&rfen, score, eta, 10).is_err() {
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
        let (tosub, frmain) = std::sync::mpsc::channel::<Vec<u32>>();
        let (tomain, frsub) = std::sync::mpsc::channel::<()>();
        let (txprogress, rxprogress) = std::sync::mpsc::channel();
        let eta = self.eta;

        let sub = std::thread::spawn(move || {
            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};

            tomain.send(()).unwrap();

            loop {
                match frmain.recv() {
                    Ok( rfenidxgrp ) => {
                        // print!("{rfen:?},{score} \r");
                        if rfenidxgrp.is_empty() {
                            tomain.send(()).unwrap();
                            continue;
                        }
                        if rfenidxgrp[0] == STOP  {
                            // println!("score > 64");
                            break;
                        }
                        if rfenidxgrp[0] == PROGRESS {
                            let mut w = weight::Weight::new();
                            w.copy(&weight);
                            txprogress.send(w).unwrap();
                            continue;
                        }
                        //
                        for i in rfenidxgrp {
                            let (rfen, score) = unsafe {&RFENCACHE[i as usize]};
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
        let prgstbl = self.progress.clone();
        let repeat = self.repeat as u32;
        let subprgs = std::thread::spawn(move || {
            for prgs in prgstbl {
                if prgs >= repeat {
                    println!("WARNING: progress {prgs} >= {repeat}...");
                    break;
                }
                let weight = rxprogress.recv().unwrap();
                if cfg!(feature="nnv4") {
                    weight.writev4(&format!("kifu/newevaltable.r{prgs}.txt"));
                } else {
                    weight.writev5(&format!("kifu/newevaltable.r{prgs}.txt"));
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
        // let mut rfencache : Vec<(String, i8)> = Vec::new();
        for fname in files.iter() {
            let path = format!("{}{}", self.path, fname);
            if showprgs {print!("reading {path}\r");}

            let content = std::fs::read_to_string(&path).unwrap();
            let lines:Vec<&str> = content.split("\n").collect();

            let kifu = kifu::Kifu::from(&lines);
            let score = kifu.score.unwrap();
            for te in kifu.list.iter() {
                let rfen = te.rfen.clone();
                // 最終手はカウントで良いので学習しなくて良い
                if bitboard::count_emptycells(&rfen).unwrap() < 1 {
                    continue;
                }
                unsafe {RFENCACHE.push((rfen.clone(), score));}
                let b = bitboard::BitBoard::from(&rfen).unwrap();
                let b90 = b.rotate90();
                unsafe {RFENCACHE.push((b90.to_str(), score));}
                let b180 = b.rotate180();
                unsafe {RFENCACHE.push((b180.to_str(), score));}
                let b270 = b90.rotate180();
                unsafe {RFENCACHE.push((b270.to_str(), score));}
                /*if score.abs() > 32 {
                    // 大差がついている棋譜は多めに覚える
                    unsafe {RFENCACHE.push((rfen, score));}
                }*/
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

        let n = unsafe {RFENCACHE.len()};
        // println!("{n} rfens.");
        let mut numbers : Vec<u32> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        // for (i, it) in numbers.iter_mut().enumerate() {
        //     *it = i;
        // }
        for i in 0..n {
            numbers[i] = i as u32;
        }
        let invalidprogress = 99999999;
        let mut prgs = VecDeque::from(self.progress.clone());
        let mut nprgs = prgs.pop_front().unwrap_or(invalidprogress) as usize;
        for i in 0..self.repeat {
            if showprgs {
                print!("{i} / {}\r", self.repeat);
                std::io::stdout().flush().unwrap();
            }
            numbers.shuffle(&mut rng);
            for idx in 0..10 {
                let grp = numbers[idx * n / 10 .. (idx + 1) * n / 10].to_vec();
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
            if nprgs == i {
                match tosub.send(vec![PROGRESS]) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
                nprgs = prgs.pop_front().unwrap_or(invalidprogress) as usize;
            }
            frsub.recv().unwrap();
        }
        if showprgs {println!("");}
        // println!("_ _ _");
        match tosub.send(vec![STOP]) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }
        subprgs.join().unwrap();
        sub.join().unwrap();
    }

    /**
     * 棋譜の読み込みと学習を別スレでやる版
     * boardを纏めてやり取りする版
     */
    #[allow(dead_code)]
    pub fn learn_stones_para_boardgrp(&mut self) {
        let (tosub, frmain) = std::sync::mpsc::channel::<Vec<u32>>();
        let (tomain, frsub) = std::sync::mpsc::channel::<()>();
        let (txprogress, rxprogress) = std::sync::mpsc::channel();
        let eta = self.eta;

        let sub = std::thread::spawn(move || {
            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};

            tomain.send(()).unwrap();

            loop {
                match frmain.recv() {
                    Ok( rfenidxgrp ) => {
                        // print!("{rfen:?},{score} \r");
                        if rfenidxgrp.is_empty() {
                            tomain.send(()).unwrap();
                            continue;
                        }
                        if rfenidxgrp[0] == STOP  {
                            // println!("score > 64");
                            break;
                        }
                        if rfenidxgrp[0] == PROGRESS {
                            let mut w = weight::Weight::new();
                            w.copy(&weight);
                            txprogress.send(w).unwrap();
                            continue;
                        }
                        //
                        for i in rfenidxgrp {
                            let (ban, score) = unsafe {&BOARDCACHE[i as usize]};
                            if weight.train_bitboard(ban, *score, eta, 0).is_err() {
                                println!("error while training");
                                break;
                            }
                        }
                    },
                    Err(e) => {panic!("{}", e.to_string())}
                }
            }
        });
        let prgstbl = self.progress.clone();
        let repeat = self.repeat as u32;
        let subprgs = std::thread::spawn(move || {
            for prgs in prgstbl {
                if prgs >= repeat {
                    println!("WARNING: progress {prgs} >= {repeat}...");
                    break;
                }
                let weight = rxprogress.recv().unwrap();
                if cfg!(feature="nnv4") {
                    weight.writev4(&format!("kifu/newevaltable.r{prgs}.txt"));
                } else {
                    weight.writev5(&format!("kifu/newevaltable.r{prgs}.txt"));
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
        // let mut rfencache : Vec<(String, i8)> = Vec::new();
        for fname in files.iter() {
            let path = format!("{}{}", self.path, fname);
            if showprgs {print!("reading {path}\r");}

            let content = std::fs::read_to_string(&path).unwrap();
            let lines:Vec<&str> = content.split("\n").collect();

            let kifu = kifu::Kifu::from(&lines);
            let score = kifu.score.unwrap();
            for te in kifu.list.iter() {
                let rfen = te.rfen.clone();
                // 最終手はカウントで良いので学習しなくて良い
                if bitboard::count_emptycells(&rfen).unwrap() < 1 {
                    continue;
                }
                let b = bitboard::BitBoard::from(&rfen).unwrap();
                let b90 = b.rotate90();
                let b180 = b.rotate180();
                let b270 = b90.rotate180();
                unsafe {
                    BOARDCACHE.push((b, score));
                    BOARDCACHE.push((b90, score));
                    BOARDCACHE.push((b180, score));
                    BOARDCACHE.push((b270, score));
                }
                /*if score.abs() > 32 {
                    // 大差がついている棋譜は多めに覚える
                    unsafe {RFENCACHE.push((rfen, score));}
                }*/
            }
            unsafe {
                BOARDCACHE.sort_by(|a, b| {
                    a.0.black.cmp(&b.0.black).then(a.0.white.cmp(&b.0.white))});
                BOARDCACHE.dedup_by(|a, b| {
                    a.0.black == b.0.black && a.0.white == b.0.white && a.0.teban == b.0.teban});
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

        let n = unsafe {BOARDCACHE.len()};
        println!("{n} rfens.");
        let mut numbers : Vec<u32> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        // for (i, it) in numbers.iter_mut().enumerate() {
        //     *it = i;
        // }
        for i in 0..n {
            numbers[i] = i as u32;
        }
        let invalidprogress = 99999999;
        let mut prgs = VecDeque::from(self.progress.clone());
        let mut nprgs = prgs.pop_front().unwrap_or(invalidprogress) as usize;
        for i in 0..self.repeat {
            if showprgs {
                print!("{i} / {}\r", self.repeat);
                std::io::stdout().flush().unwrap();
            }
            numbers.shuffle(&mut rng);
            for idx in 0..10 {
                let grp = numbers[idx * n / 10 .. (idx + 1) * n / 10].to_vec();
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
            if nprgs == i {
                match tosub.send(vec![PROGRESS]) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
                nprgs = prgs.pop_front().unwrap_or(invalidprogress) as usize;
            }
            frsub.recv().unwrap();
        }
        if showprgs {println!("");}
        // println!("_ _ _");
        match tosub.send(vec![STOP]) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }
        subprgs.join().unwrap();
        sub.join().unwrap();
    }

    /**
     * 棋譜の読み込みと学習を別スレでやる版
     * boardを纏めてやり取りする版
     * minibatch版
     */
    #[allow(dead_code)]
    pub fn learn_stones_para_boardgrp_minibatch(&mut self) {
        let (tosub, frmain) = std::sync::mpsc::channel::<Vec<u32>>();
        let (tomain, frsub) = std::sync::mpsc::channel::<()>();
        let (txprogress, rxprogress) = std::sync::mpsc::channel();
        let eta = self.eta;

        let sub = std::thread::spawn(move || {
            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};
            let mut bufweight = weight::Weight::new();

            tomain.send(()).unwrap();

            loop {
                match frmain.recv() {
                    Ok( rfenidxgrp ) => {
                        // print!("{rfen:?},{score} \r");
                        if rfenidxgrp.is_empty() {
                            tomain.send(()).unwrap();
                            continue;
                        }
                        if rfenidxgrp[0] == STOP  {
                            // println!("score > 64");
                            break;
                        }
                        if rfenidxgrp[0] == PROGRESS {
                            let mut w = weight::Weight::new();
                            w.copy(&weight);
                            txprogress.send(w).unwrap();
                            continue;
                        }
                        //
                        let n = rfenidxgrp.len();
let use_subthread = false;
// let use_subthread = true;
if use_subthread {
                        let mut banscores = Vec::with_capacity(n / 2 + 1);
                        let mut banscores2 = Vec::with_capacity(n / 2 + 1);
                        for i in 0..n / 2 {
                            banscores.push(unsafe {&BOARDCACHE[rfenidxgrp[i] as usize]});
                        }
                        for i in n / 2..n {
                            banscores2.push(unsafe {&BOARDCACHE[rfenidxgrp[i] as usize]});
                        }

                        let subsub = std::thread::spawn(move || {
                            let weight = unsafe {nodebb::WEIGHT.as_mut().unwrap()};
                            let mut bufweight2 = weight::Weight::new();
                            // bufweight2.clear();
                            if weight.train_bitboard_mb(&banscores2, eta, &mut bufweight2).is_err() {
                                println!("error while training");
                            }
                            weight.updatemb(&bufweight2, n);
                        });

                        bufweight.clear();
                        if weight.train_bitboard_mb(&banscores, eta, &mut bufweight).is_err() {
                            println!("error while training");
                            break;
                        }
                        subsub.join().unwrap();
} else {
    let mut banscores = Vec::with_capacity(n);
    for i in rfenidxgrp {
        banscores.push(unsafe {&BOARDCACHE[i as usize]});
    }
    bufweight.clear();
    if weight.train_bitboard_mb(&banscores, eta, &mut bufweight).is_err() {
        println!("error while training");
        break;
    }
}
                        weight.updatemb(&bufweight, n);
                    },
                    Err(e) => {panic!("{}", e.to_string())}
                }
            }
        });
        let prgstbl = self.progress.clone();
        let repeat = self.repeat as u32;
        let subprgs = std::thread::spawn(move || {
            for prgs in prgstbl {
                if prgs >= repeat {
                    println!("WARNING: progress {prgs} >= {repeat}...");
                    break;
                }
                let weight = rxprogress.recv().unwrap();
                if cfg!(feature="nnv4") {
                    weight.writev4(&format!("kifu/newevaltable.r{prgs}.txt"));
                } else {
                    weight.writev5(&format!("kifu/newevaltable.r{prgs}.txt"));
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
        // let mut rfencache : Vec<(String, i8)> = Vec::new();
        for fname in files.iter() {
            let path = format!("{}{}", self.path, fname);
            if showprgs {print!("reading {path}\r");}

            let content = std::fs::read_to_string(&path).unwrap();
            let lines:Vec<&str> = content.split("\n").collect();

            let kifu = kifu::Kifu::from(&lines);
            let score = kifu.score.unwrap();
            for te in kifu.list.iter() {
                let rfen = te.rfen.clone();
                // 最終手はカウントで良いので学習しなくて良い
                if bitboard::count_emptycells(&rfen).unwrap() < 1 {
                    continue;
                }
                let b = bitboard::BitBoard::from(&rfen).unwrap();
                let b90 = b.rotate90();
                let b180 = b.rotate180();
                let b270 = b90.rotate180();
                unsafe {
                    BOARDCACHE.push((b, score));
                    BOARDCACHE.push((b90, score));
                    BOARDCACHE.push((b180, score));
                    BOARDCACHE.push((b270, score));
                }
                /*if score.abs() > 32 {
                    // 大差がついている棋譜は多めに覚える
                    unsafe {RFENCACHE.push((rfen, score));}
                }*/
            }
            unsafe {
                BOARDCACHE.sort_by(|a, b| {
                    a.0.black.cmp(&b.0.black).then(a.0.white.cmp(&b.0.white))});
                BOARDCACHE.dedup_by(|a, b| {
                    a.0.black == b.0.black && a.0.white == b.0.white && a.0.teban == b.0.teban});
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

        let n = unsafe {BOARDCACHE.len()};
        println!("{n} rfens.");
        let mut numbers : Vec<u32> = Vec::with_capacity(n);
        unsafe { numbers.set_len(n); }
        // for (i, it) in numbers.iter_mut().enumerate() {
        //     *it = i;
        // }
        for i in 0..n {
            numbers[i] = i as u32;
        }
        let invalidprogress = 99999999;
        let mut prgs = VecDeque::from(self.progress.clone());
        let mut nprgs = prgs.pop_front().unwrap_or(invalidprogress) as usize;
        for i in 0..self.repeat {
            if showprgs {
                print!("{i} / {}\r", self.repeat);
                std::io::stdout().flush().unwrap();
            }
            numbers.shuffle(&mut rng);
            /*for idx in 0..10 {
                let grp = numbers[idx * n / 10 .. (idx + 1) * n / 10].to_vec();
                match tosub.send(grp) {
                    Ok(_) => {},
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
            }*/
            const MB_SIZE : usize = 128;
            let mut j = MB_SIZE;
            while j < numbers.len() {
                let grp = numbers[j - MB_SIZE .. j].to_vec();
                j += MB_SIZE;
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
            if nprgs == i {
                match tosub.send(vec![PROGRESS]) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("{}", e.to_string());
                    },
                }
                nprgs = prgs.pop_front().unwrap_or(invalidprogress) as usize;
            }
            frsub.recv().unwrap();
        }
        if showprgs {println!("");}
        // println!("_ _ _");
        match tosub.send(vec![STOP]) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            },
        }
        subprgs.join().unwrap();
        sub.join().unwrap();
    }
}
