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

fn verbose(rfen : &str) {
    if cfg!(feature="bitboard") {
            match bitboard::BitBoard::from(rfen) {
            Err(msg) => {println!("{}", msg)},
            Ok(ban) => {
                ban.put();

                let st = Instant::now();
                let (val, node) =
                    nodebb::NodeBB::thinko_ab_extract2( &ban, 7).unwrap();
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
                    node::Node::vb_think_ab( &ban, 7).unwrap();
                let ft = st.elapsed();
                println!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
            }
        }
    }
}

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

fn duel(ev1 : &str, ev2 : &str) {
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
                    //     7, &w1, &w2).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab, 7, &w1, &w2).unwrap()
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_extract2, 7, &w1, &w2).unwrap()
                },
                "all" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko, 7, &w1, &w2).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think, 7, &w1, &w2).unwrap()
                },
                // "" => {
                //     // g.startsh_with_2et(shnode::ShNode::think_ab_extract2, 7, &w1, &w2).unwrap()
                //     g.startsh_with_2et(shnode::ShNode::think_ab, 7, &w1, &w2).unwrap()
                //     // g.startsh_with_2et(shnode::ShNode::think, 7, &w1, &w2).unwrap()
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
                7, &w1, &w2).unwrap();
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
                    // g.start_with_2et(nodebb::NodeBB::think_ab_extract2, 7, &w1, &w2).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think_ab, 7, &w2, &w1).unwrap()
                    g.starto_with_2et(nodebb::NodeBB::thinko_ab_extract2, 7, &w2, &w1).unwrap()
                    // g.starto_with_2et(nodebb::NodeBB::thinko_ab, 7, &w2, &w1).unwrap()
                },
                "all" => {
                    g.starto_with_2et(nodebb::NodeBB::thinko, 7, &w2, &w1).unwrap()
                    // g.start_with_2et(nodebb::NodeBB::think, 7, &w2, &w1).unwrap()
                },
                // "" => {
                //     // g.startsh_with_2et(shnode::ShNode::think_ab_extract2, 7, &w2, &w1).unwrap()
                //     g.startsh_with_2et(shnode::ShNode::think_ab, 7, &w2, &w1).unwrap()
                //     // g.startsh_with_2et(shnode::ShNode::think, 7, &w2, &w1).unwrap()
                // },
                _ => { panic!("unknown thinking method.") }
            }
            let dresult = g.kifu.winner();
            teban = g.kifu.nth(1).teban;
            result = dresult.unwrap();
        } else {
            // prepare game
            let mut g = game::Game::from(rfen);
            // g.start_with_2et(node::Node::think_ab_extract2, 7, &w2, &w1).unwrap();
            g.start_with_2et(node::Node::think_ab, 7, &w2, &w1).unwrap();
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

fn play(turnh: i8) {
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
        //     }, 7, turnh).unwrap();
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
            }, 7, turnh).unwrap();
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
            }, 7, turnh).unwrap();
        }
}

fn edax(turnh: i8) {
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
        //     }, 7, turnh).unwrap();
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
            }, 7, turnh).unwrap();
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
        }, 7, turnh).unwrap();
    }
}

fn main() {
    println!("Hello, reversi world!");

    MYOPT
        .set(myoption::MyOption::new(std::env::args().collect()))
        .unwrap();

    if cfg!(feature="bitboard") {
        nodebb::init_weight();
    } else {
        node::init_weight();
    }

    // trial();

    // read command options

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

    let mode = &MYOPT.get().unwrap().mode;
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
        duel(ev1, ev2);
    }
    if *mode == myoption::Mode::Play {
        let turn = MYOPT.get().unwrap().turn;
        let opp = &MYOPT.get().unwrap().opponent;
        match opp{
            myoption::Opponent::CUI => {
                play(
                    if turn == board::NONE {
                        let mut rng = rand::thread_rng();
                        if rng.gen::<bool>() {board::SENTE} else {board::GOTE}
                    } else {
                        turn
                    });
            },
            myoption::Opponent::Edax => {
                edax(
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
        verbose(rfen);
    }
}
