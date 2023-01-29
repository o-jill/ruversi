use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};

mod board;
mod bitboard;
mod extractrfen;
mod game;
mod initialpos;
mod myoption;
mod node;
mod nodebb;
mod shnode;
mod kifu;
mod trainer;
mod transptable;
mod weight;


/// global settings.
static MYOPT: once_cell::sync::OnceCell<myoption::MyOption> = once_cell::sync::OnceCell::new();

#[allow(dead_code)]
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
                let yres;
                let ires;
                unsafe {
                    yres = nodebb::WEIGHT.as_ref().unwrap().forwardv3(&byb);
                    // yres = nodebb::WEIGHT.as_ref().unwrap().forwardv3_simd(&byb);
                    // ires = nodebb::WEIGHT.as_ref().unwrap().forwardv3bb(&bib);
                    // ires = nodebb::WEIGHT.as_ref().unwrap().forwardv3bb_simd(&bib);
                    ires = nodebb::WEIGHT.as_ref().unwrap().forwardv3bb_simdavx(&bib);
                }
                if yres.2 != ires.2 {
                    println!("0: {:?} == {:?}", yres.0, ires.0);
                    println!("1: {:?} == {:?}", yres.1, ires.1);
                    println!("2: {:?} == {:?}", yres.2, ires.2);
                    println!("3: {:?} == {:?}", yres.3, ires.3);
                }
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
            fnm.find(".txt").is_some()
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

    let tr = trainer::Trainer::new(0.01, 100);
    unsafe {
        tr.run4stones(&g.kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
        // tr.run4win(&g.kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
    }
}

/// think about a given situation.
/// # Arguments
/// - rfen : RFEN text to be thought.
/// - depth : depth to think.
fn verbose(rfen : &str, depth : u8) {
    if cfg!(feature="bitboard") {
            match bitboard::BitBoard::from(rfen) {
            Err(msg) => {println!("{}", msg)},
            Ok(ban) => {
                ban.put();

                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::thinko_ab_extract2(&ban, depth).unwrap();
                let ft = st.elapsed();
                println!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
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

/// generate kifu
/// # Arguments
/// - n : None or Some(0 - 9). index in 10 group.
fn gen_kifu(n : Option<usize>) {
    let ip = initialpos::InitialPos::read(initialpos::INITIALPOSFILE).unwrap();
    let rfentbl =
            ip.rfens_uniq(&["ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX"]);

    let grp;
    let rfentbl = if n.is_none() {
        grp = 0;
        &rfentbl
    } else {
        let n = n.unwrap();
        grp = n;
        let sz = rfentbl.len();
        let b = sz * n / 10;
        let e = sz * (n + 1) / 10;
        &rfentbl[b..e]
    };

    for (idx, rfen) in rfentbl.iter().enumerate() {
        let kifutxt;
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    // g.start(
                    //     // nodebb::NodeBB::think_ab_extract2,
                    //     nodebb::NodeBB::think_ab,
                    //     7).unwrap()
                    g.starto(nodebb::NodeBB::thinko_ab_extract2, 7).unwrap();
                },
                "all" => {
                    // g.start(nodebb::NodeBB::think, 7).unwrap()
                    g.starto(nodebb::NodeBB::thinko, 7).unwrap();
                },
                _ => { panic!("unknown thinking method.") }
            }
            kifutxt = g.kifu.to_str()
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    // g.start(node::Node::think_ab_extract2, 7).unwrap()
                    g.start(node::Node::think_ab, 7).unwrap()
                },
                "all" => {
                    g.start(node::Node::think, 7).unwrap()
                },
                _ => { panic!("unknown thinking method.") }
            }
            kifutxt = g.kifu.to_str()
        }

        // store kifu
        let kifuname = format!("./kifu/kifu{}{:06}.txt", grp, idx);
        let mut f = File::create(kifuname).unwrap();
        let content = format!("{}{}", kifu::HEADER, kifutxt);
        f.write(content.as_bytes()).unwrap();
    }
}

/// training a weight.
/// # Arguments
/// - repeat : Number of repeat. None as 10000.
/// - eta : learning ratio. None as 0.0001.
fn training(repeat : Option<usize>, eta : Option<f32>) {
    let repeat = repeat.unwrap_or(10000);
    let eta = eta.unwrap_or(0.0001);

    let st = Instant::now();

    // list up kifu
    let files = std::fs::read_dir("./kifu/").unwrap();
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

    files.sort();

    // train
    let tr = trainer::Trainer::new(eta, repeat);
    tr.learn_stones_cache(&mut files);
    // tr.learn_stones(&mut files);
    // tr.learn_win(&mut files);

    // put new eval table
    unsafe {
        if cfg!(feature="bitboard") {
            nodebb::WEIGHT.as_ref().unwrap().writev3("./kifu/newevaltable.txt");
        } else {
            if cfg!(feature="nnv1") {
                node::WEIGHT.as_ref().unwrap().writev1asv2("./kifu/newevaltable.txt");
            } else if cfg!(feature="nnv2") {
                node::WEIGHT.as_ref().unwrap().writev2asv3("./kifu/newevaltable.txt");
            } else {
                node::WEIGHT.as_ref().unwrap().writev3("./kifu/newevaltable.txt");
            }
        }
    }

    let mut win = 0;
    let mut draw = 0;
    let mut lose = 0;
    let mut total = 0;
    for path in files.iter() {
        let kifu = extractrfen::extract(&format!("kifu/{}", path));
        let result = kifu.winner();
        total += 1;
        let result = result.unwrap();
        match result {
            kifu::SENTEWIN => {win += 1;},
            kifu::DRAW => {draw += 1;},
            kifu::GOTEWIN => {lose += 1;},
            _ => {}
        }
    }
    let ft = st.elapsed();
    let min = ft.as_secs() / 60;
    let sec = ft.as_secs() % 60;
    let spdbatch = ft.as_secs_f64() * 1000.0 / repeat as f64;
    let spdkifu = spdbatch / files.len() as f64;
    println!("processing time: {}min {}sec ({:.1}msec/batch, {:.3}msec/file)", min, sec, spdbatch, spdkifu);
    println!("total,{},win,{},draw,{},lose,{}", total, win, draw, lose);
}

/// duel between 2 eval tables.
/// # Arguments
/// - ev1 : eval table 1.
/// - ev2 : eval table 2.
fn duel(ev1 : &str, ev2 : &str, depth : u8) {
    let mut w1 = weight::Weight::new();
    w1.read(ev1).unwrap();
    let mut w2 = weight::Weight::new();
    w2.read(ev2).unwrap();
    let mut win = [0, 0];
    let mut draw = [0, 0];
    let mut lose = [0, 0];
    let mut total = 0;
    let mut result;
    let mut teban;

    let ip = initialpos::InitialPos::read(initialpos::INITIALPOSFILE).unwrap();
    let rfentbl = &ip.at("THREE").unwrap().rfens;
    for rfen in rfentbl.iter() {
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    // g.start_with_2et(
                    //     // nodebb::NodeBB::think_ab_extract3,
                    //     // nodebb::NodeBB::think_ab_extract2,
                    //     nodebb::NodeBB::think_ab,
                    //     depth, &w1, &w2).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab, depth, &w1, &w2).unwrap()
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_extract2, depth, &w1, &w2).unwrap()
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
            teban = g.kifu.nth(0).teban;
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            g.start_with_2et(
                // node::Node::think_ab_extract2,
                node::Node::think_ab,
                depth, &w1, &w2).unwrap();
            let dresult = g.kifu.winner();
            teban = g.kifu.nth(0).teban;
            result = dresult.unwrap();
        }
        total += 1;
        if teban == board::SENTE {
            match result {
                kifu::SENTEWIN => {win[0] += 1;},
                kifu::DRAW => {draw[0] += 1;},
                kifu::GOTEWIN => {lose[0] += 1;},
                _ => {}
            }
        } else {
            match result {
                kifu::SENTEWIN => {lose[1] += 1;},
                kifu::DRAW => {draw[1] += 1;},
                kifu::GOTEWIN => {win[1] += 1;},
                _ => {}
            }
        }
        if cfg!(feature="bitboard") {
            // prepare game
            let mut g = game::GameBB::from(rfen);
            // play
            let think = MYOPT.get().unwrap().think.as_str();
            match think {
                "" | "ab" => {
                    // g.start_with_2et(nodebb::NodeBB::think_ab_extract2, depth, &w1, &w2).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think_ab, depth, &w2, &w1).unwrap()
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_extract2, depth, &w2, &w1).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab, depth, &w2, &w1).unwrap()
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
            teban = g.kifu.nth(1).teban;
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            // g.start_with_2et(node::Node::think_ab_extract2, depth, &w2, &w1).unwrap();
            g.start_with_2et(node::Node::think_ab, depth, &w2, &w1).unwrap();
            let dresult = g.kifu.winner();
            teban = g.kifu.nth(1).teban;
            result = dresult.unwrap();
        }
        total += 1;
        if teban == board::SENTE {
            match result {
                kifu::SENTEWIN => {win[0] += 1;},
                kifu::DRAW => {draw[0] += 1;},
                kifu::GOTEWIN => {lose[0] += 1;},
                _ => {}
            }
        } else {
            match result {
                kifu::SENTEWIN => {lose[1] += 1;},
                kifu::DRAW => {draw[1] += 1;},
                kifu::GOTEWIN => {win[1] += 1;},
                _ => {}
            }
        }
        let twin = win[0] + win[1];
        let tdraw = draw[0] + draw[1];
        let tlose = lose[0] + lose[1];
        let winrate = 100.0 * twin as f64 / (total - tdraw) as f64;
        let r = 400.0 * (twin as f64 / tlose as f64).log10();
        println!("total,{},win,{},draw,{},lose,{},{:.2}%,R,{:+.1}",
            total, twin, tdraw, tlose, winrate, r);
        println!("ev1 @@,win,{},draw,{},lose,{}", win[0], draw[0], lose[0]);
        println!("ev1 [],win,{},draw,{},lose,{}", win[1], draw[1], lose[1]);
    }
    println!("ev1:{}", MYOPT.get().unwrap().evaltable1);
    println!("ev2:{}", MYOPT.get().unwrap().evaltable2);
}

/// read eval file.
/// # Arguments
/// - path : file path.
fn readeval(path: &str) {
    println!("read eval table: {}", path);
    if cfg!(feature="bitboard") {
        // println!("read weight for bitboard");
        unsafe {
            match nodebb::WEIGHT.as_mut().unwrap().read(path) {
                Err(msg) => {panic!("{}", msg)},
                _ => {}
            }
        }
    } else {
        // println!("read weight for byteboard");
        unsafe {
            match node::WEIGHT.as_mut().unwrap().read(path) {
                Err(msg) => {panic!("{}", msg)},
                _ => {}
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
                    nodebb::NodeBB::thinko_ab_extract2
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
                    nodebb::NodeBB::thinko_ab_extract2
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

/// show command options and exit(1).
fn help() {
    myoption::showhelp("a reversi program written in rust.");
    std::process::exit(1);
}

pub fn postxt(x : u8, y : u8) -> String {
    if x < 1 || x > 8 || y < 1 || y > 8 {
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
    let obf = "/tmp/test.obf";
    let cd = "../../edax-reversi/";
    let edaxpath = "./bin/lEdax-x64-modern";
    let evfile = "data/eval.dat";
    let scoreptn = regex::Regex::new("%\\s+([+-]\\d\\d)").unwrap();
    let ip = initialpos::InitialPos::read(initialpos::INITIALPOSFILE).unwrap();
    // let rfentbl = &ip.at("FIVE").unwrap().rfens;
    // let rfentbl = &ip.at("FOUR").unwrap().rfens;
    let rfentbl = &ip.at("THREE").unwrap().rfens;
    for rfen in rfentbl.iter() {
        // println!("rfen:{rfen}");
        let ban = bitboard::BitBoard::from(rfen).unwrap();
        {
            // println!("put board to a file...");
            let mut f = File::create(obf).unwrap();
            f.write(ban.to_obf().as_bytes()).unwrap();
            f.write("\n".as_bytes()).unwrap();
            f.flush().unwrap();
        }
        // launch edax
        let cmd = match std::process::Command::new(edaxpath)
            .arg("--solve").arg(obf).current_dir(cd)
            .arg("--eval-file").arg(evfile)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null()).spawn() {
            Err(msg) => panic!("error running edax... [{}]", msg),
            Ok(prcs) => prcs,
        };
        // read stdout and get moves
        let w = cmd.wait_with_output().unwrap();
        let txt = String::from_utf8(w.stdout).unwrap();
        // println!("{txt}");
        let lines : Vec<_> = txt.split("\n").collect();
        // println!("{}", lines[2]);
        match scoreptn.captures(&lines[2]) {
            Some(cap) => {
                let score = &cap[1];
                if vec!["-01", "+00", "+01"].contains(&score) {
                    println!("{rfen}, {score}");
                 } else {
                    eprintln!("{rfen}, {score}");
                 }
            },
            _ => {}
        }
        }
    Ok(())
}

fn main() {
    println!("Hello, reversi world!");

    // read command options
    MYOPT
        .set(myoption::MyOption::new(std::env::args().collect()))
        .unwrap();

    let mode = &MYOPT.get().unwrap().mode;
    if *mode == myoption::Mode::Help {
        help();
    }
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
        gen_kifu(n);
    }
    if *mode == myoption::Mode::None || *mode == myoption::Mode::Learn {
        let repeat = MYOPT.get().unwrap().repeat;
        let eta = MYOPT.get().unwrap().eta;
        training(repeat, eta);
    }
    if *mode == myoption::Mode::Duel {
        let ev1 = &MYOPT.get().unwrap().evaltable1;
        let ev2 = &MYOPT.get().unwrap().evaltable2;
        duel(ev1, ev2, depth);
    }
    if *mode == myoption::Mode::Play {
        let turn = MYOPT.get().unwrap().turn;
        let opp = &MYOPT.get().unwrap().opponent;
        match opp {
            myoption::Opponent::CUI => {
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
            _ => {panic!("{:?} is not supported yet.", opp)},
        }
    }
    if *mode == myoption::Mode::RFEN {
        let rfen = &MYOPT.get().unwrap().rfen;
        verbose(rfen, depth);
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
