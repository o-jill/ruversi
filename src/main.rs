use std::fs::File;
use std::io::Write;
use std::time::{Instant};
use std::thread;
use rand::Rng;
use std::sync::{Arc, Mutex};

mod cassio;
mod bitboard;
mod duelresult;
mod edaxrunner;
// mod extractrfen;
mod game;
mod gtprotocol;
mod initialpos;
mod myoption;
mod nodebb;
mod kifu;
// mod trainer;
mod transptable;
mod weight;


/// global settings.
static MYOPT: once_cell::sync::OnceCell<myoption::MyOption> = once_cell::sync::OnceCell::new();

#[allow(dead_code)]
#[cfg(target_arch="x86_64")]
fn trial() {
    let mut kifu = kifu::Kifu::new();
    kifu.append(0,1, String::new());
    kifu.append(1,1, String::new());
    kifu.append(2,-1, String::new());
    kifu.append(3,1, String::new());
    kifu.append(4,-1, String::new());
    kifu.append(5,1, String::new());
    kifu.append(6,-1, String::new());
    kifu.append(7,0, String::new());
    print!("{}", kifu.to_str());

    println!();
    let ban = bitboard::BitBoard::from("4A3/2AaB2/3aAa2/2Ca2/2Ad1/1BaAa2/2aBa2/1f1 b").unwrap();
    ban.put();
    let moves = ban.genmove();
    println!("moves:{moves:?}");

    panic!();
}

#[allow(dead_code)]
#[cfg(target_arch="aarch64")]
fn trial() {
}

/// think about a given situation.
/// # Arguments
/// - rfen : RFEN text to be thought.
/// - depth : depth to think.
fn verbose(rfen : &str, depth : u8,
        treepath : &Option<String>, cachesz : usize, show_children : bool) {
    let think = MYOPT.get().unwrap().think.as_str();
    let f = match think {
            "all" => {
                nodebb::NodeBB::think_simple_gk_tt
            },
            _ => {  // "" | "ab" => {
            // nodebb::NodeBB::think_mtdf(&ban, depth, &mut node, wei, &mut tt).unwrap()
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab(&ban, depth).unwrap()
            // nodebb::NodeBB::think_ab_extract2(&ban, depth).unwrap()
            },
        };
    let mut tt = transptable::TranspositionTable::with_capacity(cachesz);
    let wei = unsafe{nodebb::WEIGHT.as_ref().unwrap()};
    let ban = match bitboard::BitBoard::from(rfen) {
        Err(msg) => {panic!("{msg}");},
        Ok(ban) => {ban},
    };

    ban.put();

    let st = Instant::now();
    let mut node = nodebb::NodeBB::root(depth);
    let val = f(&ban, depth, &mut node, wei, &mut tt).unwrap();
    let ft = st.elapsed();
    println!("val:{val:.4?} {node} {}msec", ft.as_millis());
    if let Some(path) = treepath {
        if let Err(e) = node.dumptree(0, path) {
            eprintln!("{e}@{} {}", file!(), line!());
        } else {
            println!("put tree into {path}.")
        }
    }

    if !show_children || ban.is_last1_or_full() {return;}

    // 子供の局面の探索結果を出力
    // thinkall以外はあまり正確ではない。
    for nd in node.child.iter() {
        let newban = ban.r#move(nd.xy).unwrap();
        println!("val,{},{newban}", nd.best.as_ref().unwrap().hyoka);
    }
}

fn genkifu_single(rfentbl : &[String], depth : u8, grp : &str, cachesz: usize) {
    let think = MYOPT.get().unwrap().think.as_str();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    for (idx, rfen) in rfentbl.iter().enumerate() {
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        // play
        g.startgk(f, depth).unwrap();
        let kifutxt = g.kifu.to_str();

        // store kifu
        let kifuname = format!("./kifu/kifu{grp}{idx:05}.txt");
        let mut f = File::create(kifuname).unwrap();
        let content = format!("{}{kifutxt}", kifu::HEADER);
        f.write_all(content.as_bytes()).unwrap();
    }
}

fn genkifu_para(rfentbl : &[String], depth : u8, grp : &str, cachesz: usize) {
    let n = rfentbl.len();
    let rfentbl1 = rfentbl[0..n/2].to_vec();
    let rfentbl2 = &rfentbl[n/2..];

    let grp1 = format!("{grp}0");
    let sub = thread::spawn(move || {
            genkifu_single(&rfentbl1, depth, &grp1, cachesz);
        });

    genkifu_single(rfentbl2, depth, &format!("{grp}1"), cachesz);

    sub.join().unwrap();
}

/// generate kifu
/// # Arguments
/// - n : None or Some(0 - 19). index in 20 group.
fn gen_kifu(n : Option<usize>, depth : u8, cachesz : usize) {
    let mut ip = initialpos::InitialPos::read(initialpos::INITIALPOSFILE).unwrap();
    ip.append(initialpos::INITIALPOSFILE7).unwrap();
    let rfentbl_src =
            ip.rfens_uniq(&["ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN"]);

    let grp;
    let rfentbl = if let Some(group) = n {
        grp = group;
        const N_GROUP : usize = 100;
        let sz = rfentbl_src.len();
        let b = sz * grp / N_GROUP;
        let e = sz * (grp + 1) / N_GROUP;
        &rfentbl_src[b..e]
    } else {
        grp = 0;
        &rfentbl_src
    };

    genkifu_para(rfentbl, depth, &format!("{grp:02}"), cachesz);
    // genkifu_single(rfentbl, depth, &format!("{grp:02}"), cachesz);
}

/// duel between 2 eval tables.
/// # Arguments
/// - ev1 : eval table 1.
/// - ev2 : eval table 2.
fn duel_para(ev1 : &str, ev2 : &str, duellv : i8, depth : u8, cachesz : usize) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut w1 = Box::new(weight::Weight::new());
    w1.read(ev1).unwrap();
    let mut w2 = Box::new(weight::Weight::new());
    w2.read(ev2).unwrap();
    let mut w3 = Box::new(weight::Weight::new());
    w3.copy(&w1);
    let mut w4 = Box::new(weight::Weight::new());
    w4.copy(&w2);
    let dresult = Arc::new(Mutex::new(duelresult::DuelResult::new()));
    let dresult2 = dresult.clone();

    let verbose = MYOPT.get().unwrap().verbose;
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &mut ip.rfens_all();
    let n = rfentbl.len() / 2;
    let rfen1 = rfentbl.drain(n..).collect::<Vec<String>>();
    let think = MYOPT.get().unwrap().think.as_str();

    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    let thrd = thread::spawn(move || {
        for rfen in rfen1.iter() {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_cachesize(cachesz);
            g.set_verbose(&verbose);
            // play
            match think {
                "" | "ab" => {
                    g.starto_with_2et_mt_tt(f, depth, &w3, &w4).unwrap()
                },
                "all" => {
                    g.starto_with_2et(f, depth, &w3, &w4).unwrap()
                },
                _ => { panic!("unknown thinking method.") }
            }
            let result = g.kifu.winner().unwrap();
            {
                let mut dr = dresult2.lock().unwrap();
                dr.sresult(result);
            }

            // prepare game
            let mut g = game::GameBB::from(rfen);
            g.set_cachesize(cachesz);
            g.set_verbose(&verbose);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    g.starto_with_2et_mt_tt(f, depth, &w4, &w3).unwrap()
                },
                "all" => {
                    g.starto_with_2et(f, depth, &w4, &w3).unwrap()
                },
                _ => { panic!("unknown thinking method.") }
            }
            let result = g.kifu.winner().unwrap();
            {
                let mut dr = dresult2.lock().unwrap();
                dr.gresult(result);
                println!("{dr}");
            }
        }});

    for rfen in rfentbl.iter() {
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        let think = MYOPT.get().unwrap().think.as_str();
        match think {
            "" | "ab" => {
                g.starto_with_2et_mt_tt(f, depth, &w2, &w1).unwrap()
            },
            "all" => {
                g.starto_with_2et(f, depth, &w1, &w2).unwrap()
            },
            _ => { panic!("unknown thinking method.") }
        }
        let result = g.kifu.winner().unwrap();
        {
            let mut dr = dresult.lock().unwrap();
            dr.sresult(result);
        }

        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        let think = MYOPT.get().unwrap().think.as_str();
        match think {
            "" | "ab" => {
                g.starto_with_2et_mt_tt(f, depth, &w2, &w1).unwrap()
            },
            "all" => {
                g.starto_with_2et(f, depth, &w2, &w1).unwrap()
            },
            _ => { panic!("unknown thinking method.") }
        }
        let result = g.kifu.winner().unwrap();
        {
            let mut dr = dresult.lock().unwrap();
            dr.gresult(result);
            println!("{dr}");
        }
    }

    thrd.join().unwrap();

    {
        let dr = dresult.lock().unwrap();
        println!("{dr}");
    }
    println!("ev1:{}", MYOPT.get().unwrap().evaltable1);
    println!("ev2:{}", MYOPT.get().unwrap().evaltable2);
}

/// duel between 2 eval tables.
/// # Arguments
/// - ev1 : eval table 1.
/// - ev2 : eval table 2.
#[allow(dead_code)]
fn duel(ev1 : &str, ev2 : &str, duellv : i8, depth : u8, cachesz : usize) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut w1 = weight::Weight::new();
    w1.read(ev1).unwrap();
    let mut w2 = weight::Weight::new();
    w2.read(ev2).unwrap();
    let mut dr = duelresult::DuelResult::default();
    let mut result;

    let verbose = MYOPT.get().unwrap().verbose;
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &ip.rfens_all();
    let think = MYOPT.get().unwrap().think.as_str();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    for rfen in rfentbl.iter() {
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        g.starto_with_2et(f, depth, &w1, &w2).unwrap();
        let dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.sresult(result);
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        g.starto_with_2et(f, depth, &w2, &w1).unwrap();
        let dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.gresult(result);

        println!("{dr}");
    }
    println!("ev1:{}", MYOPT.get().unwrap().evaltable1);
    println!("ev2:{}", MYOPT.get().unwrap().evaltable2);
}

/// duel between 2 eval tables.
/// # Arguments
/// - duellv : duel level.
/// - depth : searching depth.
fn duel_vs_edax(duellv : i8, depth : u8, cachesz : usize) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut dr = duelresult::DuelResult::default();
    let mut dresult;
    let mut result;

    let econf = std::path::PathBuf::from(
            MYOPT.get().unwrap().edaxconfig.as_str());
    let think = MYOPT.get().unwrap().think.as_str();
    let verbose = MYOPT.get().unwrap().verbose;
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    let rfentbl = &ip.rfens_all();
    for rfen in rfentbl.iter() {
        let turn = bitboard::SENTE;
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        g.starto_against_edax(f, depth, turn, &econf).unwrap();
        dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.sresult(result);
        let turn = bitboard::GOTE;
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        g.starto_against_edax(f, depth, turn, &econf).unwrap();
        dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.gresult(result);

        println!("{}", dr.opponent());
    }
}

/// duel between 2 eval tables.
/// # Arguments
/// - duellv : duel level.
/// - depth : searching depth.
fn duel_vs_cassio(duellv : i8, depth : u8, cachesz : usize) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut dr = duelresult::DuelResult::default();
    let mut dresult;
    let mut result;

    let econf = MYOPT.get().unwrap().edaxconfig.as_str();
    // println!("econf:{econf}");
    let think = MYOPT.get().unwrap().think.as_str();
    let verbose = MYOPT.get().unwrap().verbose;
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    let rfentbl = &ip.rfens_all();
    for rfen in rfentbl.iter() {
        let turn = bitboard::SENTE;
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        g.start_against_via_cassio(f, depth, turn, econf).unwrap();
        dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.sresult(result);

        let turn = bitboard::GOTE;
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        g.start_against_via_cassio(f, depth, turn, econf).unwrap();
        dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.gresult(result);

        println!("{}", dr.opponent());
    }
}

/// duel between 2 eval tables.
/// # Arguments
/// - duellv : duel level.
/// - depth : searching depth.
fn duel_vs_ruversi(duellv : i8, depth : u8, cachesz : usize) {
    if !(1..=14).contains(&duellv) {
        panic!("duel level:{duellv} is not supported...");
    }

    let mut dr = duelresult::DuelResult::default();
    let mut dresult;
    let mut result;

    let econf = MYOPT.get().unwrap().edaxconfig.as_str();
    let think = MYOPT.get().unwrap().think.as_str();
    let verbose = MYOPT.get().unwrap().verbose;
    let eqfile = initialpos::equalfile(duellv);
    println!("equal file: {eqfile}");
    let ip = initialpos::InitialPos::read(&eqfile).unwrap();
    let rfentbl = &ip.rfens_all();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
        };
    for rfen in rfentbl.iter() {
        let turn = bitboard::SENTE;
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        g.starto_against_ruversi(f, depth, turn, econf).unwrap();
        dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.sresult(result);

        let turn = bitboard::GOTE;
        // prepare game
        let mut g = game::GameBB::from(rfen);
        g.set_cachesize(cachesz);
        g.set_verbose(&verbose);
        // play
        g.starto_against_ruversi(f, depth, turn, econf).unwrap();
        dresult = g.kifu.winner();
        result = dresult.unwrap();
        dr.gresult(result);

        if !verbose.is_silent() {println!("{}", dr.opponent());}
    }
    if verbose.is_silent() {println!("{}", dr.opponent());}
}

/// read eval file.
/// # Arguments
/// - path : file path.
fn readeval(path: &str) {
    println!("read eval table: {path}");
    // println!("read weight for bitboard");
    unsafe {
        if let Err(msg) = nodebb::WEIGHT.as_mut().unwrap().read(path) {
            panic!("{msg}");
        }
    }
}

/// play a game ruversi vs you.
/// # Arguments
/// - depth : depth to think.
/// - turnh : your turn.
fn play(depth : u8, turnh: i8, cachesz : usize) {
    // prepare game
    let mut g = game::GameBB::new();
    g.set_cachesize(cachesz);
    // play
    let think = MYOPT.get().unwrap().think.as_str();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    g.starto_against_stdin(f, depth, turnh).unwrap();
}

/// play a game ruversi vs Edax.
/// # Arguments
/// - depth : depth to think.
/// - turnh : Edax's turn.
fn edax(depth : u8, turnh: i8, cachesz : usize) {
    // prepare game
    let mut g = game::GameBB::new();
    g.set_cachesize(cachesz);
    // play
    let econf = std::path::PathBuf::from(
            MYOPT.get().unwrap().edaxconfig.as_str());
    let think = MYOPT.get().unwrap().think.as_str();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    g.starto_against_edax(f, depth, turnh, &econf).unwrap();
}

/// play a game ruversi vs another ruversi.
/// # Arguments
/// - depth : depth to think.
/// - turnh : another ruversi's turn.
fn vs_ruversi(depth : u8, turnh: i8, cachesz : usize) {
    let verbose = MYOPT.get().unwrap().verbose;
    // prepare game
    let mut g = game::GameBB::new();
    g.set_cachesize(cachesz);
    g.set_verbose(&verbose);
    // play
    let econf = MYOPT.get().unwrap().edaxconfig.as_str();
    let think = MYOPT.get().unwrap().think.as_str();
    let f = match think {
        "" | "ab" => {
            // nodebb::NodeBB::think_mtdf
            nodebb::NodeBB::think_ab_simple_gk_tt
            // nodebb::NodeBB::think_ab_simple
        },
        "all" => {
            nodebb::NodeBB::think_simple_gk_tt
        },
        _ => { panic!("unknown thinking method.") }
    };
    g.starto_against_ruversi(f, depth, turnh, econf).unwrap();
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

    format!("{}{y}", bitboard::STR_GOTE.chars().nth(x as usize).unwrap())
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
            moves.push(bitboard::PASS);
        }
        for mv in moves.iter() {
            let mvx = mv % bitboard::NUMCELL as u8;
            let mvy = mv / bitboard::NUMCELL as u8;
            let mvstr = postxt(mvx, mvy);

            let ban2 = ban.r#move(*mv).unwrap();
            let moves2 = ban2.genmove();
            if moves2.is_none() {  // no empty cells.
                continue;
            }

            let mut moves2 = moves2.unwrap();
            if moves2.is_empty() {
                moves2.push(bitboard::PASS);
            }
            for mv2 in moves.iter() {
                let mvx2 = mv2 % bitboard::NUMCELL as u8;
                let mvy2 = mv2 / bitboard::NUMCELL as u8;
                let mvstr2 = postxt(mvx2, mvy2);
                let ban3 = ban2.r#move(*mv2).unwrap();
                println!("{ban3},  // ****** {mvstr} {mvstr2}");
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

    nodebb::init_weight();

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
    } else if *mode != myoption::Mode::Duel {
        readeval(path);
    }

    // trial();
    let depth = MYOPT.get().unwrap().depth;
    let cachesz = MYOPT.get().unwrap().cachesize_actual();

    if *mode == myoption::Mode::None || *mode == myoption::Mode::GenKifu {
        let n = MYOPT.get().unwrap().n;
        gen_kifu(n, depth, cachesz);
    }
    if *mode == myoption::Mode::Learn {
        eprintln!("learning was deprecated. please use tigerdenversi instead.");
    }
    if *mode == myoption::Mode::Duel {
        let ev1 = &MYOPT.get().unwrap().evaltable1;
        let ev2 = &MYOPT.get().unwrap().evaltable2;
        let duellv = MYOPT.get().unwrap().duellv;
        duel_para(ev1, ev2, duellv, depth, cachesz);
        // duel(ev1, ev2, duellv, depth, cachesz);
    }
    if *mode == myoption::Mode::DuelExt {
        let duellv = MYOPT.get().unwrap().duellv;
        let opp = &MYOPT.get().unwrap().opponent;
        println!("opponent:{opp:?}");
        match opp {
            myoption::Opponent::Ruversi => {
                duel_vs_ruversi(duellv, depth, cachesz);
            },
            myoption::Opponent::Cassio => {
                duel_vs_cassio(duellv, depth, cachesz);
            },
            _ => {duel_vs_edax(duellv, depth, cachesz);}
        }
    }
    if *mode == myoption::Mode::Play {
        let turn = MYOPT.get().unwrap().turn;
        let opp = &MYOPT.get().unwrap().opponent;
        match opp {
            myoption::Opponent::Cui => {
                play(
                    depth,
                    if turn == bitboard::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {bitboard::SENTE} else {bitboard::GOTE}
                    } else {
                        turn
                    }, cachesz);
            },
            myoption::Opponent::Edax => {
                edax(
                    depth,
                    if turn == bitboard::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {bitboard::SENTE} else {bitboard::GOTE}
                    } else {
                        turn
                    }, cachesz);
            },
            myoption::Opponent::Ruversi => {
                vs_ruversi(
                    depth,
                    if turn == bitboard::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {bitboard::SENTE} else {bitboard::GOTE}
                    } else {
                        turn
                    }, cachesz);
            },
            _ => {panic!("{opp:?} is not supported yet.")},
        }
    }
    if *mode == myoption::Mode::Rfen {
        let rfen = &MYOPT.get().unwrap().rfen;
        let treepath = &MYOPT.get().unwrap().treedump;
        let show_children = MYOPT.get().unwrap().children;
        verbose(rfen, depth, treepath, cachesz, show_children);
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
