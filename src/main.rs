use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use std::sync::{Arc, Mutex};

mod cassio;
mod board;
mod bitboard;
mod edaxrunner;
// mod extractrfen;
mod game;
mod gtprotocol;
mod initialpos;
mod myoption;
mod node;
mod nodebb;
mod shnode;
mod kifu;
// mod trainer;
mod transptable;
mod weight;


/// global settings.
static MYOPT: once_cell::sync::OnceCell<myoption::MyOption> = once_cell::sync::OnceCell::new();

#[allow(dead_code)]
#[cfg(target_arch="x86_64")]
fn trial() {
    if false {
        let die = Uniform::from(-1..=1);
        let mut rng = rand::thread_rng();
        let mut cells : [i8 ; 64] = [0 ; 64];
        for _i in 0..50 {
            for c in cells.iter_mut() {
                *c = die.sample(&mut rng);
            }
            let tbn = die.sample(&mut rng);
            let byb = board::Board::fromarray(cells, tbn);
            let bib = bitboard::BitBoard::from(&byb.to_str()).unwrap();
            if false {
                let yres;
                let ires;
                unsafe {
                    // yres = nodebb::WEIGHT.as_ref().unwrap().evaluatev3(&byb);
                    yres = nodebb::WEIGHT.as_ref().unwrap().evaluatev3_simd(&byb);
                    // ires = nodebb::WEIGHT.as_ref().unwrap().evaluatev3bb(&bib);
                    ires = nodebb::WEIGHT.as_ref().unwrap().evaluatev3bb_simdavx(&bib);
                    // ires = nodebb::WEIGHT.as_ref().unwrap().evaluatev3bb_simd(&bib);
                }
                if yres != ires {
                    println!("eval: {} == {}", yres, ires);
                }
            } else {
            }
        }
        panic!("stoppppppp!!!!");
    }
    let files = std::fs::read_dir("./kifu/").unwrap();
    let files = files.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name().and_then(|n|
                n.to_str().map(|s| String::from(s))
            )
        )}).collect::<Vec<String>>().iter().filter(|&fnm| {
            fnm.contains(".txt")
        }).cloned().collect::<Vec<String>>();
    println!("{:?}", files);

    let ban = board::Board::new();
    ban.put();
    let rfen = "aAaAaAaA/BbBb/C2c/dD/E3/2f/g1/H b";
    println!("rfen: {}", rfen);
    let ban = board::Board::from(rfen).unwrap();
    ban.put();
    println!("RFEN:{}", ban.to_str());
    let mut ban = board::Board::init();
    ban.flipturn();
    ban.put();
    let st = Instant::now();
    let (val, node) = node::Node::think( &ban, 7).unwrap();
    let ft = st.elapsed();
    println!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());

    println!("candidate:{:?}", ban.genmove());
    let ban2 = ban.r#move(3, 4).unwrap();
    ban2.put();
    println!("candidate:{:?}", ban2.genmove());
    let ban2 = ban2.r#move(3, 3).unwrap();
    ban2.put();
    println!("candidate:{:?}", ban2.genmove());

    let (tx, rx) = mpsc::channel();
    let th = thread::spawn(move ||
        for i in 0..10 {
            let msg = format!("thread: -- {} -- -- -- --", i);
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_secs_f32(0.5))
        }
    );

    let mut kifu = kifu::Kifu::new();
    kifu.append(0, 0, 1, String::new());
    kifu.append(1, 1, 1, String::new());
    kifu.append(2, 2, -1, String::new());
    kifu.append(3, 3, 1, String::new());
    kifu.append(4, 4, -1, String::new());
    kifu.append(5, 5, 1, String::new());
    kifu.append(6, 6, -1, String::new());
    kifu.append(7, 7, 0, String::new());
    print!("{}", kifu.to_str());

    th.join().unwrap();
    loop {
        let received = rx.recv();
        if received.is_err() {
            break;
        }
        println!("{}", received.unwrap());
    }

    let mut g = game::Game::new();
    g.start(node::Node::think, 7).unwrap();
}

#[allow(dead_code)]
#[cfg(target_arch="aarch64")]
fn trial() {
}

/// think about a given situation.
/// # Arguments
/// - rfen : RFEN text to be thought.
/// - depth : depth to think.
fn verbose(rfen : &str, depth : u8, treepath : &Option<String>) {
    if cfg!(feature="bitboard") {
            match bitboard::BitBoard::from(rfen) {
            Err(msg) => {println!("{}", msg)},
            Ok(ban) => {
                ban.put();

                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::thinko_ab_simple(&ban, depth).unwrap();
                    // nodebb::NodeBB::thinko_ab(&ban, depth).unwrap();
                    // nodebb::NodeBB::thinko_ab_extract2(&ban, depth).unwrap();
                let ft = st.elapsed();
                println!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
                if let Some(path) = treepath {
                    if let Err(e) = node.dumptree(0, path) {
                        eprintln!("{}@{} {}", e.to_string(),file!(), line!());
                    } else {
                        println!("put tree into {path}.")
                    }
                }
            }
        }
    } else {
        match board::Board::from(rfen) {
            Err(msg) => {println!("{}", msg)},
            Ok(ban) => {
                ban.put();

                let st = Instant::now();
                let (val, node) =
                    node::Node::vb_think_ab(&ban, depth).unwrap();
                let ft = st.elapsed();
                println!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
            }
        }
    }
}

fn genkifu_single(rfentbl : &[String], depth : u8, grp : &str) {
    for (idx, rfen) in rfentbl.iter().enumerate() {
        let think = MYOPT.get().unwrap().think.as_str();
        let kifutxt = if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            // play
            match think {
                "" | "ab" => {
                    // g.start(
                    //     // nodebb::NodeBB::think_ab_extract2,
                    //     nodebb::NodeBB::think_ab,
                    //     depth).unwrap()
                    g.startgk(nodebb::NodeBB::thinko_ab_simple_gk, depth).unwrap();
                    //g.starto(nodebb::NodeBB::thinko_ab_simple, depth).unwrap();
                    // g.starto(nodebb::NodeBB::thinko_ab_extract2, depth).unwrap();
                },
                "all" => {
                    // g.start(nodebb::NodeBB::think, depth).unwrap()
                    g.starto(nodebb::NodeBB::thinko, depth).unwrap();
                },
                _ => { panic!("unknown thinking method.") }
            }
            g.kifu.to_str()
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            // play
            match think {
                "" | "ab" => {
                    // g.start(node::Node::think_ab_extract2, depth).unwrap()
                    g.start(node::Node::think_ab, depth).unwrap()
                },
                "all" => {
                    g.start(node::Node::think, depth).unwrap()
                },
                _ => { panic!("unknown thinking method.") }
            }
            g.kifu.to_str()
        };

        // store kifu
        let kifuname = format!("./kifu/kifu{grp}{idx:05}.txt");
        let mut f = File::create(kifuname).unwrap();
        let content = format!("{}{}", kifu::HEADER, kifutxt);
        f.write_all(content.as_bytes()).unwrap();
    }
}

fn genkifu_para(rfentbl : &[String], depth : u8, grp : &str) {
    let n = rfentbl.len();
    let rfentbl1 = rfentbl[0..n/2].to_vec();
    let rfentbl2 = &rfentbl[n/2..];

    let grp1 = format!("{grp}0");
    let sub = thread::spawn(move || {
            genkifu_single(&rfentbl1, depth, &grp1);
        });

    genkifu_single(rfentbl2, depth, &format!("{grp}1"));

    sub.join().unwrap();
}

/// generate kifu
/// # Arguments
/// - n : None or Some(0 - 19). index in 20 group.
fn gen_kifu(n : Option<usize>, depth : u8) {
    let mut ip = initialpos::InitialPos::read(initialpos::INITIALPOSFILE).unwrap();
    ip.append(initialpos::INITIALPOSFILE7).unwrap();
    let rfentbl =
            ip.rfens_uniq(&["ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN"]);

    let grp;
    let rfentbl = if n.is_none() {
        grp = 0;
        &rfentbl
    } else {
        let group = 100;
        grp = n.unwrap();
        let sz = rfentbl.len();
        let b = sz * grp / group;
        let e = sz * (grp + 1) / group;
        &rfentbl[b..e]
    };

    genkifu_para(rfentbl, depth, &format!("{grp:02}"));
    // genkifu_single(rfentbl, depth, &format!("{grp:02}"));
}

struct DuelResult {
    pub win : [u32 ; 2],
    pub draw : [u32 ; 2],
    pub lose : [u32 ; 2],
    pub total : u32,
}

impl DuelResult {
    pub fn new() -> DuelResult {
        DuelResult {
            win : [0 ; 2],
            draw : [0 ; 2],
            lose : [0 ; 2],
            total : 0,
        }
    }

    pub fn sresult(&mut self, winner : i8) {
        self.total += 1;
        match winner {
            kifu::SENTEWIN => {self.win[0] += 1;},
            kifu::DRAW => {self.draw[0] += 1;},
            kifu::GOTEWIN => {self.lose[0] += 1;},
            _ => {}
        }
    }
    pub fn gresult(&mut self, winner : i8) {
        self.total += 1;
        match winner {
            kifu::SENTEWIN => {self.lose[1] += 1;},
            kifu::DRAW => {self.draw[1] += 1;},
            kifu::GOTEWIN => {self.win[1] += 1;},
            _ => {}
        }
    }

    pub fn duel_summary(win : &[u32], lose : &[u32], draw : &[u32], total : &u32) {
        let twin = win[0] + win[1];
        let tdraw = draw[0] + draw[1];
        let tlose = lose[0] + lose[1];
        let tsen = win[0] + lose[1] + tdraw;
        let tgo = win[1] + lose[0] + tdraw;
        let winrate = twin as f64 / (total - tdraw) as f64;
        let winrate100 = 100.0 * winrate;
        let r = 400.0 * (twin as f64 / tlose as f64).log10();

        let err_margin = if *total > tdraw {
            let se = (winrate * (1.0 - winrate) / *total as f64).sqrt();
            400.0 / std::f64::consts::LN_10 * se
        } else {
            0.0
        };
        let confidence_interval = err_margin * 1.96;  // 95%信頼区間

        println!(
            "total,{total},win,{twin},draw,{tdraw},lose,{tlose},balance,{tsen},{tgo},{winrate100:.2}%,R,{r:+.1},{confidence_interval:+.1}");
        println!("ev1 @@,win,{},draw,{},lose,{}", win[0], draw[0], lose[0]);
        println!("ev1 [],win,{},draw,{},lose,{}", win[1], draw[1], lose[1]);
    }

    pub fn dump(&self) {
        Self::duel_summary(&self.win, &self.draw, &self.lose, &self.total);
    }
}

/// duel between 2 eval tables.
/// # Arguments
/// - ev1 : eval table 1.
/// - ev2 : eval table 2.
fn duel_para(ev1 : &str, ev2 : &str, duellv : i8, depth : u8) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut w1 = weight::Weight::new();
    w1.read(ev1).unwrap();
    let mut w2 = weight::Weight::new();
    w2.read(ev2).unwrap();
    let mut w3 = weight::Weight::new();
    w3.copy(&w1);
    let mut w4 = weight::Weight::new();
    w4.copy(&w2);
    let dresult = Arc::new(Mutex::new(DuelResult::new()));
    let dresult2 = dresult.clone();

    let verbose = &MYOPT.get().unwrap().verbose;
    let silent = verbose.is_silent();
    let verbose = verbose.is_verbose();
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &mut ip.rfens_all();
    let n = rfentbl.len() / 2;
    let rfen1 = rfentbl.drain(n..).collect::<Vec<String>>();
    let think = MYOPT.get().unwrap().think.as_str();

    let thrd = thread::spawn(move || {
        let verbose = !silent;
        let mut result;
        for rfen in rfen1.iter() {
            if cfg!(feature="bitboard") {
                // prepare game
                let mut g = game::GameBB::from(rfen);
                g.set_verbose(verbose);
                // play
                match think {
                    "" | "ab" => {
                        // g.starto_with_2et(nodebb::NodeBB::thinko_ab_simple, depth, &w3, &w4).unwrap()
                        g.starto_with_2et_mt(nodebb::NodeBB::thinko_ab_simple_gk, depth, &w3, &w4).unwrap()
                    },
                    "all" => {
                        g.starto_with_2et(nodebb::NodeBB::thinko, depth, &w3, &w4).unwrap()
                    },
                    _ => { panic!("unknown thinking method.") }
                }
                let dresult = g.kifu.winner();
                result = dresult.unwrap();
            } else {
                // prepare game
                let mut g = game::Game::from(rfen);
                g.set_verbose(verbose);
                g.start_with_2et(
                    node::Node::think_ab,
                    depth, &w3, &w4).unwrap();
                let dresult = g.kifu.winner();
                result = dresult.unwrap();
            }
            {
                let mut dr = dresult2.lock().unwrap();
                dr.sresult(result);
            }
            if cfg!(feature="bitboard") {
                // prepare game
                let mut g = game::GameBB::from(rfen);
                g.set_verbose(verbose);
                // play
                let think = MYOPT.get().unwrap().think.as_str();
                match think {
                    "" | "ab" => {
                        g.starto_with_2et_mt(nodebb::NodeBB::thinko_ab_simple_gk, depth, &w4, &w3).unwrap()
                    },
                    "all" => {
                        g.starto_with_2et(nodebb::NodeBB::thinko, depth, &w4, &w3).unwrap()
                    },
                    _ => { panic!("unknown thinking method.") }
                }
                let dresult = g.kifu.winner();
                result = dresult.unwrap();
            } else {
                // prepare game
                let mut g = game::Game::from(rfen);
                g.set_verbose(verbose);
                g.start_with_2et(node::Node::think_ab, depth, &w4, &w3).unwrap();
                let dresult = g.kifu.winner();
                result = dresult.unwrap();
            }
            {
                let mut dr = dresult2.lock().unwrap();
                dr.gresult(result);
                dr.dump();
            }
        }});

    let mut result;
    for rfen in rfentbl.iter() {
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_simple, depth, &w1, &w2).unwrap()
                },
                "all" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko, depth, &w1, &w2).unwrap()
                },
                _ => { panic!("unknown thinking method.") }
            }
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            g.start_with_2et(
                // node::Node::think_ab_extract2,
                node::Node::think_ab,
                depth, &w1, &w2).unwrap();
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        }
        {
            let mut dr = dresult.lock().unwrap();
            dr.sresult(result);
        }
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_simple, depth, &w2, &w1).unwrap()
                },
                "all" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko, depth, &w2, &w1).unwrap()
                },
                _ => { panic!("unknown thinking method.") }
            }
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            // g.start_with_2et(node::Node::think_ab_extract2, depth, &w2, &w1).unwrap();
            g.start_with_2et(node::Node::think_ab, depth, &w2, &w1).unwrap();
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        }
        {
            let mut dr = dresult.lock().unwrap();
            dr.gresult(result);
            dr.dump();
        }
    }

    thrd.join().unwrap();

    {
        let dr = dresult.lock().unwrap();
        dr.dump();
    }
    println!("ev1:{}", MYOPT.get().unwrap().evaltable1);
    println!("ev2:{}", MYOPT.get().unwrap().evaltable2);
}

/// duel between 2 eval tables.
/// # Arguments
/// - ev1 : eval table 1.
/// - ev2 : eval table 2.
fn duel(ev1 : &str, ev2 : &str, duellv : i8, depth : u8) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut w1 = weight::Weight::new();
    w1.read(ev1).unwrap();
    let mut w2 = weight::Weight::new();
    w2.read(ev2).unwrap();
    let mut win = [0, 0];
    let mut draw = [0, 0];
    let mut lose = [0, 0];
    let mut total = 0;
    let mut result;

    let verbose = !MYOPT.get().unwrap().verbose.is_silent();
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &ip.rfens_all();
    for rfen in rfentbl.iter() {
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    // g.start_with_2et(
                    //     // nodebb::NodeBB::think_ab_extract3,
                    //     // nodebb::NodeBB::think_ab_extract2,
                    //     nodebb::NodeBB::think_ab,
                    //     depth, &w1, &w2).unwrap()
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_simple, depth, &w1, &w2).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab, depth, &w1, &w2).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab_extract2, depth, &w1, &w2).unwrap()
                },
                "all" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko, depth, &w1, &w2).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think, depth, &w1, &w2).unwrap()
                },
                // "" => {
                //     // g.startsh_with_2et(shnode::ShNode::think_ab_extract2, depth, &w1, &w2).unwrap()
                //     g.startsh_with_2et(shnode::ShNode::think_ab, depth, &w1, &w2).unwrap()
                //     // g.startsh_with_2et(shnode::ShNode::think, depth, &w1, &w2).unwrap()
                // },
                _ => { panic!("unknown thinking method.") }
            }
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            g.start_with_2et(
                // node::Node::think_ab_extract2,
                node::Node::think_ab,
                depth, &w1, &w2).unwrap();
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        }
        total += 1;
        match result {
            kifu::SENTEWIN => {win[0] += 1;},
            kifu::DRAW => {draw[0] += 1;},
            kifu::GOTEWIN => {lose[0] += 1;},
            _ => {}
        }
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    // g.start_with_2et(nodebb::NodeBB::think_ab_extract2, depth, &w1, &w2).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think_ab, depth, &w2, &w1).unwrap()
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_simple, depth, &w2, &w1).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab, depth, &w2, &w1).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab_extract2, depth, &w2, &w1).unwrap()
                },
                "all" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko, depth, &w2, &w1).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think, depth, &w2, &w1).unwrap()
                },
                // "" => {
                //     // g.startsh_with_2et(shnode::ShNode::think_ab_extract2, depth, &w2, &w1).unwrap()
                //     g.startsh_with_2et(shnode::ShNode::think_ab, depth, &w2, &w1).unwrap()
                //     // g.startsh_with_2et(shnode::ShNode::think, depth, &w2, &w1).unwrap()
                // },
                _ => { panic!("unknown thinking method.") }
            }
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            // g.start_with_2et(node::Node::think_ab_extract2, depth, &w2, &w1).unwrap();
            g.start_with_2et(node::Node::think_ab, depth, &w2, &w1).unwrap();
            let dresult = g.kifu.winner();
            result = dresult.unwrap();
        }
        total += 1;
        match result {
            kifu::SENTEWIN => {lose[1] += 1;},
            kifu::DRAW => {draw[1] += 1;},
            kifu::GOTEWIN => {win[1] += 1;},
            _ => {}
        }

        DuelResult::duel_summary(&win, &draw, &lose, &total);
    }
    println!("ev1:{}", MYOPT.get().unwrap().evaltable1);
    println!("ev2:{}", MYOPT.get().unwrap().evaltable2);
}

/// duel between 2 eval tables.
/// # Arguments
/// - duellv : duel level.
/// - depth : searching depth.
fn duel_vs_edax(duellv : i8, depth : u8) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut win = [0, 0];
    let mut draw = [0, 0];
    let mut lose = [0, 0];
    let mut total = 0;
    let mut dresult;
    let mut result;

    let econf = MYOPT.get().unwrap().edaxconfig.as_str();
    let think = MYOPT.get().unwrap().think.as_str();
    let verbose = !MYOPT.get().unwrap().verbose.is_silent();
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &ip.rfens_all();
    for rfen in rfentbl.iter() {
        let turn = board::SENTE;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            // g.start_against_edax(
            //     match think {
            //         "" | "ab" => {
            //             // nodebb::NodeBB::think_ab_extract2
            //             nodebb::NodeBB::think_ab
            //         },
            //         "all" => {
            //             nodebb::NodeBB::think
            //         },
            //         _ => { panic!("unknown thinking method.") }
            //     }, depth, turn).unwrap();
            g.starto_against_edax(
                match think {
                    "" | "ab" => {
                        // nodebb::NodeBB::think_ab_extract2
                        nodebb::NodeBB::thinko_ab_simple
                        // nodebb::NodeBB::thinko_ab_extract2
                    },
                    "all" => {
                        nodebb::NodeBB::thinko
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn, econf).unwrap();
            dresult = g.kifu.winner();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            // g.start_against_edax(
            //     match think {
            //         "" | "ab" => {
            //             // nodebb::NodeBB::think_ab_extract2
            //             nodebb::NodeBB::think_ab
            //         },
            //         "all" => {
            //             nodebb::NodeBB::think
            //         },
            //         _ => { panic!("unknown thinking method.") }
            //     }, depth, turn).unwrap();
            g.start_against_edax(
                match think {
                    "" | "ab" => {
                        // nodebb::NodeBB::think_ab_extract2
                        node::Node::think_ab_extract2
                    },
                    "all" => {
                        node::Node::think
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn).unwrap();
            dresult = g.kifu.winner();
        }
        result = dresult.unwrap();
        total += 1;
        match result {
            kifu::SENTEWIN => {win[0] += 1;},
            kifu::DRAW => {draw[0] += 1;},
            kifu::GOTEWIN => {lose[0] += 1;},
            _ => {}
        }
        let turn = board::GOTE;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            // g.start_against_edax(
            //     match think {
            //         "" | "ab" => {
            //             // nodebb::NodeBB::think_ab_extract2
            //             nodebb::NodeBB::think_ab
            //         },
            //         "all" => {
            //             nodebb::NodeBB::think
            //         },
            //         _ => { panic!("unknown thinking method.") }
            //     }, depth, turnh).unwrap();
            g.starto_against_edax(
                match think {
                    "" | "ab" => {
                        // nodebb::NodeBB::think_ab_extract2
                        nodebb::NodeBB::thinko_ab_simple
                        // nodebb::NodeBB::thinko_ab_extract2
                    },
                    "all" => {
                        nodebb::NodeBB::thinko
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn, econf).unwrap();
            dresult = g.kifu.winner();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            // g.start_against_edax(
            //     match think {
            //         "" | "ab" => {
            //             // nodebb::NodeBB::think_ab_extract2
            //             nodebb::NodeBB::think_ab
            //         },
            //         "all" => {
            //             nodebb::NodeBB::think
            //         },
            //         _ => { panic!("unknown thinking method.") }
            //     }, depth, turnh).unwrap();
            g.start_against_edax(
                match think {
                    "" | "ab" => {
                        // nodebb::NodeBB::think_ab_extract2
                        node::Node::think_ab_extract2
                    },
                    "all" => {
                        node::Node::think
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn).unwrap();
            dresult = g.kifu.winner();
        }
        result = dresult.unwrap();
        total += 1;
        match result {
            kifu::SENTEWIN => {lose[1] += 1;},
            kifu::DRAW => {draw[1] += 1;},
            kifu::GOTEWIN => {win[1] += 1;},
            _ => {}
        }

        DuelResult::duel_summary(&win, &draw, &lose, &total);
    }
}

/// duel between 2 eval tables.
/// # Arguments
/// - duellv : duel level.
/// - depth : searching depth.
fn duel_vs_cassio(duellv : i8, depth : u8) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut win = [0, 0];
    let mut draw = [0, 0];
    let mut lose = [0, 0];
    let mut total = 0;
    let mut dresult;
    let mut result;

    let econf = MYOPT.get().unwrap().edaxconfig.as_str();
    // println!("econf:{econf}");
    let think = MYOPT.get().unwrap().think.as_str();
    let verbose = !MYOPT.get().unwrap().verbose.is_silent();
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &ip.rfens_all();
    for rfen in rfentbl.iter() {
        let turn = board::SENTE;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            g.start_against_via_cassio(
                match think {
                    "" | "ab" => {
                        // nodebb::NodeBB::think_ab_extract2
                        nodebb::NodeBB::thinko_ab_simple
                        // nodebb::NodeBB::thinko_ab_extract2
                    },
                    "all" => {
                        nodebb::NodeBB::thinko
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn, econf).unwrap();
            dresult = g.kifu.winner();
        } else {
            unimplemented!()
        }
        result = dresult.unwrap();
        total += 1;
        match result {
            kifu::SENTEWIN => {win[0] += 1;},
            kifu::DRAW => {draw[0] += 1;},
            kifu::GOTEWIN => {lose[0] += 1;},
            _ => {}
        }
        let turn = board::GOTE;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            g.start_against_via_cassio(
                match think {
                    "" | "ab" => {
                        // nodebb::NodeBB::think_ab_extract2
                        nodebb::NodeBB::thinko_ab_simple
                        // nodebb::NodeBB::thinko_ab_extract2
                    },
                    "all" => {
                        nodebb::NodeBB::thinko
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn, econf).unwrap();
            dresult = g.kifu.winner();
        } else {
            unimplemented!()
        }
        result = dresult.unwrap();
        total += 1;
        match result {
            kifu::SENTEWIN => {lose[1] += 1;},
            kifu::DRAW => {draw[1] += 1;},
            kifu::GOTEWIN => {win[1] += 1;},
            _ => {}
        }

        DuelResult::duel_summary(&win, &draw, &lose, &total);
    }
}

/// duel between 2 eval tables.
/// # Arguments
/// - duellv : duel level.
/// - depth : searching depth.
fn duel_vs_ruversi(duellv : i8, depth : u8) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut win = [0, 0];
    let mut draw = [0, 0];
    let mut lose = [0, 0];
    let mut total = 0;
    let mut dresult;
    let mut result;

    let econf = MYOPT.get().unwrap().edaxconfig.as_str();
    let think = MYOPT.get().unwrap().think.as_str();
    let verbose = !MYOPT.get().unwrap().verbose.is_silent();
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &ip.rfens_all();
    for rfen in rfentbl.iter() {
        let turn = board::SENTE;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            g.starto_against_ruversi(
                match think {
                    "" | "ab" => {
                        nodebb::NodeBB::thinko_ab_simple
                    },
                    "all" => {
                        nodebb::NodeBB::thinko
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn, econf).unwrap();
            dresult = g.kifu.winner();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            g.start_against_edax(
                match think {
                    "" | "ab" => {
                        node::Node::think_ab_extract2
                    },
                    "all" => {
                        node::Node::think
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn).unwrap();
            dresult = g.kifu.winner();
        }
        result = dresult.unwrap();
        total += 1;
        match result {
            kifu::SENTEWIN => {win[0] += 1;},
            kifu::DRAW => {draw[0] += 1;},
            kifu::GOTEWIN => {lose[0] += 1;},
            _ => {}
        }
        let turn = board::GOTE;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_verbose(verbose);
            // play
            g.starto_against_ruversi(
                match think {
                    "" | "ab" => {
                        nodebb::NodeBB::thinko_ab_simple
                    },
                    "all" => {
                        nodebb::NodeBB::thinko
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn, econf).unwrap();
            dresult = g.kifu.winner();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.set_verbose(verbose);
            g.start_against_edax(
                match think {
                    "" | "ab" => {
                        node::Node::think_ab_extract2
                    },
                    "all" => {
                        node::Node::think
                    },
                    _ => { panic!("unknown thinking method.") }
                }, depth, turn).unwrap();
            dresult = g.kifu.winner();
        }
        result = dresult.unwrap();
        total += 1;
        match result {
            kifu::SENTEWIN => {lose[1] += 1;},
            kifu::DRAW => {draw[1] += 1;},
            kifu::GOTEWIN => {win[1] += 1;},
            _ => {}
        }

        DuelResult::duel_summary(&win, &draw, &lose, &total);
    }
}

/// read eval file.
/// # Arguments
/// - path : file path.
fn readeval(path: &str) {
    println!("read eval table: {}", path);
    if cfg!(feature="bitboard") {
        // println!("read weight for bitboard");
        unsafe {
            if let Err(msg) = nodebb::WEIGHT.as_mut().unwrap().read(path) {
                panic!("{}", msg);
            }
        }
    } else {
        // println!("read weight for byteboard");
        unsafe {
            if let Err(msg) = node::WEIGHT.as_mut().unwrap().read(path) {
                panic!("{}", msg);
            }
        }
    }
}

/// play a game ruversi vs you.
/// # Arguments
/// - depth : depth to think.
/// - turnh : your turn.
fn play(depth : u8, turnh: i8) {
    if cfg!(feature="bitboard") {
        // prepare game
        let mut g = game::GameBB::new();
        // play
        let think = MYOPT.get().unwrap().think.as_str();
        // g.start_against_stdin(
        //     match think {
        //         "" | "ab" => {
        //             // nodebb::NodeBB::think_ab_extract2
        //             nodebb::NodeBB::think_ab
        //         },
        //         "all" => {
        //             nodebb::NodeBB::think
        //         },
        //         _ => { panic!("unknown thinking method.") }
        //     }, depth, turnh).unwrap();
        g.starto_against_stdin(
            match think {
                "" | "ab" => {
                    nodebb::NodeBB::thinko_ab_simple
                    // nodebb::NodeBB::thinko_ab_extract2
                    // nodebb::NodeBB::think_ab
                },
                "all" => {
                    nodebb::NodeBB::thinko
                },
                _ => { panic!("unknown thinking method.") }
            }, depth, turnh).unwrap();
    } else {
        // prepare game
        let mut g = game::Game::new();
        // play
        let think = MYOPT.get().unwrap().think.as_str();
        g.start_against_stdin(
            match think {
                "" | "ab" => {
                    // node::Node::think_ab_extract2
                    node::Node::think_ab
                },
                "all" => {
                    node::Node::think
                },
                _ => { panic!("unknown thinking method.") }
            }, depth, turnh).unwrap();
        }
}

/// play a game ruversi vs Edax.
/// # Arguments
/// - depth : depth to think.
/// - turnh : Edax's turn.
fn edax(depth : u8, turnh: i8) {
    if cfg!(feature="bitboard") {
        // prepare game
        let mut g = game::GameBB::new();
        // play
        let econf = MYOPT.get().unwrap().edaxconfig.as_str();
        let think = MYOPT.get().unwrap().think.as_str();
        // g.start_against_edax(
        //     match think {
        //         "" | "ab" => {
        //             // nodebb::NodeBB::think_ab_extract2
        //             nodebb::NodeBB::think_ab
        //         },
        //         "all" => {
        //             nodebb::NodeBB::think
        //         },
        //         _ => { panic!("unknown thinking method.") }
        //     }, depth, turnh).unwrap();
        g.starto_against_edax(
            match think {
                "" | "ab" => {
                    // nodebb::NodeBB::think_ab_extract2
                    nodebb::NodeBB::thinko_ab_simple
                    // nodebb::NodeBB::thinko_ab_extract2
                },
                "all" => {
                    nodebb::NodeBB::thinko
                },
                _ => { panic!("unknown thinking method.") }
            }, depth, turnh, econf).unwrap();
    } else {
        // prepare game
        let mut g = game::Game::new();
        // play
        let think = MYOPT.get().unwrap().think.as_str();
        g.start_against_edax(
        match think {
            "" | "ab" => {
                // node::Node::think_ab_extract2
                node::Node::think_ab
            },
            "all" => {
                node::Node::think
            },
            _ => { panic!("unknown thinking method.") }
        }, depth, turnh).unwrap();
    }
}

/// play a game ruversi vs another ruversi.
/// # Arguments
/// - depth : depth to think.
/// - turnh : another ruversi's turn.
fn vs_ruversi(depth : u8, turnh: i8) {
    let verbose = !MYOPT.get().unwrap().verbose.is_silent();
    if cfg!(feature="bitboard") {
        // prepare game
        let mut g = game::GameBB::new();
        g.set_verbose(verbose);
        // play
        let econf = MYOPT.get().unwrap().edaxconfig.as_str();
        let think = MYOPT.get().unwrap().think.as_str();
        g.starto_against_ruversi(
            match think {
                "" | "ab" => {
                    // nodebb::NodeBB::think_ab_extract2
                    nodebb::NodeBB::thinko_ab_simple
                    // nodebb::NodeBB::thinko_ab_extract2
                },
                "all" => {
                    nodebb::NodeBB::thinko
                },
                _ => { panic!("unknown thinking method.") }
            }, depth, turnh, econf).unwrap();
    } else {
        panic!("byteboard is not supported...");

        // // prepare game
        // let mut g = game::Game::new();
        // // play
        // let think = MYOPT.get().unwrap().think.as_str();
        // g.start_against_ruversi(
        // match think {
        //     "" | "ab" => {
        //         // node::Node::think_ab_extract2
        //         node::Node::think_ab
        //     },
        //     "all" => {
        //         node::Node::think
        //     },
        //     _ => { panic!("unknown thinking method.") }
        // }, depth, turnh).unwrap();
    }
}

/// show command options and exit(1).
fn help() {
    myoption::showhelp("a reversi program written in rust.");
    std::process::exit(1);
}

pub fn postxt(x : u8, y : u8) -> String {
    if !(1..=8).contains(&x) || !(1..=8).contains(&y) {
        return String::from("PASS");
    }

    format!("{}{}", bitboard::STR_GOTE.chars().nth(x as usize).unwrap(), y)
}

/// generate RFENs by moving 2 stones with a RFEN.
/// # Arguments
/// - tag : tag name in initialpos.txt to use as start positions.
/// # Returns
/// Ok(()) for success, otherwise Err(error message).
fn geninitpos(tag : &str) -> Result<(), String>{
    if tag.is_empty() {
        return Err(String::from("error: tag is empty."));
    }

    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), initialpos::INITIALPOSFILE);
    let ip = initialpos::InitialPos::read(&path);
    if ip.is_err() {
        return Err(format!("error: failed to read {path}."));
    }

    let ipos = ip.unwrap();
    let pos = ipos.at(tag);
    if pos.is_none() {
        return Err(format!("error: failed to find tag:{tag}."));
    }

    let pos = pos.unwrap();
    for rfen in pos.rfens.iter() {
        let ban = bitboard::BitBoard::from(rfen);
        if ban.is_err() {
            return Err(format!("error: reading rfen \"{rfen}\"."));
        }

        let ban = ban.unwrap();
        let moves = ban.genmove();
        if moves.is_none() {  // no empty cells.
            continue;
        }

        let mut moves = moves.unwrap();
        if moves.is_empty() {
            moves.push((0, 0));
        }
        for (mvx, mvy) in moves.iter() {
            let mvstr = postxt(*mvx, *mvy);

            let ban2 = ban.r#move(*mvx, *mvy).unwrap();
            let moves2 = ban2.genmove();
            if moves2.is_none() {  // no empty cells.
                continue;
            }

            let mut moves2 = moves2.unwrap();
            if moves2.is_empty() {
                moves2.push((0, 0));
            }
            for (mvx2, mvy2) in moves2.iter() {
                let mvstr2 = postxt(*mvx2, *mvy2);
                let ban3 = ban2.r#move(*mvx2, *mvy2).unwrap();
                println!("{},  // ****** {mvstr} {mvstr2}", ban3.to_str());
            }
        }
    }

    Ok(())
}

fn equalrfen() -> Result<(), String> {
    // let input = initialpos::INITIALPOSFILE;
    let input = "data/initialpos.seven.txt";
    let ip = initialpos::InitialPos::read(input).unwrap();
    let rfentbl = &ip.rfens_all();
    // let rfentbl = &ip.at("SIX").unwrap().rfens;
    // let rfentbl = &ip.at("FIVE").unwrap().rfens;
    // let rfentbl = &ip.at("FOUR").unwrap().rfens;
    // let rfentbl = &ip.at("THREE").unwrap().rfens;
    let mut m1_0p1 = [0 ; 3];
    let mut count = 0;
    let mut res = String::new();
    let er = edaxrunner::EdaxRunner::new();
    for rfen in rfentbl.iter() {
        // println!("rfen:{rfen}");
        let ban = bitboard::BitBoard::from(rfen).unwrap();
        // launch edax
        match er.run(&ban.to_obf()) {
            Ok((_, score)) => {
                // println!("score:{score}");
                if let Some(i) = ["-01", "+00", "+01"].iter().position(|&x| x == score) {
                    res += &format!("{rfen}, {score}\n");
                    m1_0p1[i] += 1;
                    count += 1;
                }
            },
            Err(msg) => {panic!("{msg}");}
        };
        if count >= 20 {
            print!("{res}");
            std::io::stdout().flush().unwrap();
            count = 0;
            res.clear();
        }
    }
    println!("{res}\n-1:{}, 00:{}, +01:{}", m1_0p1[0], m1_0p1[1], m1_0p1[2]);
    Ok(())
}

fn gtp() {
    let mut patha;
    let mut path : &str = &MYOPT.get().unwrap().evaltable1;
    if path.is_empty() {
        patha = std::env::current_exe().unwrap().to_str().unwrap().to_string();
        // println!("patha:{patha}");
        match patha.rfind("/") {
            Some(idx) => {
                patha = patha[0..=idx].to_string();
            },
            None => {
                patha = String::new();
            }
        }
        patha += "data/evaltable.txt";
        path = &patha;
    }

    let mut gtp = gtprotocol::Gtp::new();
    match gtp.start(path) {
        Err(msg) => panic!("{msg}"),
        Ok(msg) => println!("{msg}"),
    }
    std::process::exit(0);
}

fn oep() {
    let mut patha;
    let mut path : &str = &MYOPT.get().unwrap().evaltable1;
    if path.is_empty() {
        patha = std::env::current_exe().unwrap().to_str().unwrap().to_string();
        // println!("patha:{patha}");
        match patha.rfind("/") {
            Some(idx) => {
                patha = patha[0..=idx].to_string();
            },
            None => {
                patha = String::new();
            }
        }
        patha += "data/evaltable.txt";
        path = &patha;
    }

    let mut oep = cassio::OthelloEngineProtocol::new();
    match oep.start(path) {
        Err(msg) => panic!("{msg:?}"),
        Ok(msg) => println!("{msg}"),
    }
    std::process::exit(0);
}

fn main() {
    // read command options
    MYOPT.set(
        match myoption::MyOption::new(std::env::args().collect()) {
            Ok(mo) => {mo},
            Err(msg) => {panic!("{msg}")},
        }).unwrap();

    let mode = &MYOPT.get().unwrap().mode;
    if *mode == myoption::Mode::Help {
        help();
    }
    if *mode == myoption::Mode::Gtp {
        gtp();
    }
    if *mode == myoption::Mode::Oep {
        oep();
    }

    println!("Hello, reversi world!");
    println!("mode:{mode:?}");

    if cfg!(feature="bitboard") {
        nodebb::init_weight();
    } else {
        node::init_weight();
    }

    // trial();

    // read eval table
    let path = &MYOPT.get().unwrap().evaltable1;
    if path.is_empty() {
        let path = "data/evaltable.txt";
        if std::path::Path::new(path).exists() {
            readeval(path);
        } else {
            println!("default eval table file was not found!!");
            println!("random numbers are used as eval table.");
        }
    } else {
        readeval(path);
    }

    // trial();
    let depth = MYOPT.get().unwrap().depth;

    if *mode == myoption::Mode::None || *mode == myoption::Mode::GenKifu {
        let n = MYOPT.get().unwrap().n;
        gen_kifu(n, depth);
    }
    if *mode == myoption::Mode::Learn {
        eprintln!("learning was deprecated. please use tigerdenversi instead.");
    }
    if *mode == myoption::Mode::Duel {
        let ev1 = &MYOPT.get().unwrap().evaltable1;
        let ev2 = &MYOPT.get().unwrap().evaltable2;
        let duellv = MYOPT.get().unwrap().duellv;
        duel_para(ev1, ev2, duellv, depth);
        // duel(ev1, ev2, duellv, depth, MYOPT.get().unwrap().verbose);
    }
    if *mode == myoption::Mode::DuelExt {
        let duellv = MYOPT.get().unwrap().duellv;
        let opp = &MYOPT.get().unwrap().opponent;
        println!("opponent:{opp:?}");
        match opp {
            myoption::Opponent::Ruversi => {
                duel_vs_ruversi(duellv, depth);
            },
            myoption::Opponent::Cassio => {
                duel_vs_cassio(duellv, depth);
            },
            _ => {duel_vs_edax(duellv, depth);}
        }
    }
    if *mode == myoption::Mode::Play {
        let turn = MYOPT.get().unwrap().turn;
        let opp = &MYOPT.get().unwrap().opponent;
        match opp {
            myoption::Opponent::Cui => {
                play(
                    depth,
                    if turn == board::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {board::SENTE} else {board::GOTE}
                    } else {
                        turn
                    });
            },
            myoption::Opponent::Edax => {
                edax(
                    depth,
                    if turn == board::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {board::SENTE} else {board::GOTE}
                    } else {
                        turn
                    });
            },
            myoption::Opponent::Ruversi => {
                vs_ruversi(
                    depth,
                    if turn == board::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {board::SENTE} else {board::GOTE}
                    } else {
                        turn
                    });
            },
            _ => {panic!("{:?} is not supported yet.", opp)},
        }
    }
    if *mode == myoption::Mode::Rfen {
        let rfen = &MYOPT.get().unwrap().rfen;
        let treepath = &MYOPT.get().unwrap().treedump;
        verbose(rfen, depth, treepath);
    }
    if *mode == myoption::Mode::InitPos {
        let tag = &MYOPT.get().unwrap().initpos;
        match geninitpos(tag) {
            Ok(_) => {},
            Err(msg) => {eprintln!("{msg}");}
        }
    }
    if *mode == myoption::Mode::Equal {
        match equalrfen() {
            Ok(_) => {},
            Err(msg) => {eprintln!("{msg}");}
        }
    }
}
