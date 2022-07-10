use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;

mod board;
mod extractrfen;
mod game;
mod initialpos;
mod myoption;
mod node;
mod kifu;
mod trainer;
mod weight;


/// global settings.
static MYOPT: once_cell::sync::OnceCell<myoption::MyOption> = once_cell::sync::OnceCell::new();

fn trial() {
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
    // aaa;
}

fn gen_kifu(n : Option<usize>) {
    let grp;
    let rfentbl = if n.is_none() {
        grp = 0;
        initialpos::RFENTBL.to_vec()
    } else {
        let n = n.unwrap();
        grp = n;
        let sz = initialpos::RFENTBL.len();
        let b = sz * n / 10;
        let e = sz * (n + 1) / 10;
        initialpos::RFENTBL[b..e].to_vec()
    };

    for (idx, &rfen) in rfentbl.iter().enumerate() {
        // prepare game
        let mut g = game::Game::from(rfen);
        // play
        let think = MYOPT.get().unwrap().think.as_str();
        match think {
            "" | "ab" => {
                g.start(node::Node::think_ab, 7).unwrap()
            },
            "all" => {
                g.start(node::Node::think, 7).unwrap()
            },
            _ => { panic!("unknown thinking method.") }
        }
        ;
        // store kifu
        let kifuname = format!("./kifu/kifu{}{:06}.txt", grp, idx);
        let mut f = File::create(kifuname).unwrap();
        let content = format!("{}{}", kifu::HEADER, g.kifu.to_str());
        f.write(content.as_bytes()).unwrap();
    }
}

fn training(repeat : Option<usize>, eta : Option<f32>) {
    let repeat = repeat.unwrap_or(10000);
    let eta = eta.unwrap_or(0.0001);

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

    // train
    let tr = trainer::Trainer::new(eta, repeat);
    tr.learn_stones(&mut files);
    // tr.learn_win(&mut files);

    // put new eval table
    unsafe {
        node::WEIGHT.as_ref().unwrap().write("./kifu/newevaltable.txt");
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

    for i in (1 + 4 + 12)..(1 + 4 + 12 + 56) {
        let rfen = initialpos::RFENTBL[i];
        let mut g = game::Game::from(rfen);
        g.start_with_2et(node::Node::think_ab, 7, &w1, &w2).unwrap();
        let result = g.kifu.winner();
        total += 1;
        let teban = g.kifu.nth(0).teban;
        let result = result.unwrap();
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
        let mut g = game::Game::from(rfen);
        g.start_with_2et(node::Node::think_ab, 7, &w2, &w1).unwrap();
        let result = g.kifu.winner();
        total += 1;
        let teban = g.kifu.nth(1).teban;
        let result = result.unwrap();
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
    unsafe {
        match node::WEIGHT.as_mut().unwrap().read(path) {
            Err(msg) => {println!("{}", msg)},
            _ => {}
        }
    }
}

fn main() {
    println!("Hello, reversi world!");

    MYOPT
        .set(myoption::MyOption::new(std::env::args().collect()))
        .unwrap();

    node::init_weight();

    // trial();

    // read command options

    // read eval table
    let path = &MYOPT.get().unwrap().evaltable1;
    if path.is_empty() {
        let path = "./evaltable.txt";
        if std::path::Path::new(path).exists() {
            readeval(path);
        }
    } else {
        readeval(path);
    }

    let mode = &MYOPT.get().unwrap().mode;
    if mode.is_empty() || mode == "genkifu" {
        let n = MYOPT.get().unwrap().n;
        gen_kifu(n);
    }
    if mode.is_empty() || mode == "learn" {
        let repeat = MYOPT.get().unwrap().repeat;
        let eta = MYOPT.get().unwrap().eta;
        training(repeat, eta);
    }
    if mode == "duel" {
        let ev1 = &MYOPT.get().unwrap().evaltable1;
        let ev2 = &MYOPT.get().unwrap().evaltable2;
        duel(ev1, ev2);
    }
    if mode == "rfen" {
        let rfen = &MYOPT.get().unwrap().rfen;
        verbose(rfen);
    }
}
