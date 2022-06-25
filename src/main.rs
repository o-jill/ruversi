use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;

mod board;
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
    let s = board::Teban::Sente;
    let g = board::Teban::Gote;
    let b = board::Teban::Blank;
    kifu.append(0, 0, s, String::new());
    kifu.append(1, 1, s, String::new());
    kifu.append(2, 2, g, String::new());
    kifu.append(3, 3, s, String::new());
    kifu.append(4, 4, g, String::new());
    kifu.append(5, 5, s, String::new());
    kifu.append(6, 6, g, String::new());
    kifu.append(7, 7, b, String::new());
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
    g.start().unwrap();

    let tr = trainer::Trainer::new(0.01, 100);
    unsafe {
        tr.run(&g.kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
    }
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
        g.start().unwrap();
        // store kifu
        let kifuname = format!("./kifu/kifu{}{:06}.txt", grp, idx);
        let mut f = File::create(kifuname).unwrap();
        f.write(g.kifu.to_str().as_bytes()).unwrap();
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
    let mut tr = trainer::Trainer::new(eta, repeat);
    tr.learn(&mut files);

    // put new eval table
    unsafe {
        node::WEIGHT.as_ref().unwrap().write("./kifu/newevaltable.txt");
    }
}

fn readeval(path: &str) {
    println!("read eval table: {}", path);
    unsafe {
        node::WEIGHT.as_mut().unwrap().read(path).unwrap();
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
    let mut path = &MYOPT.get().unwrap().evaltable1;
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

}
