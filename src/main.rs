use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;

mod board;
mod game;
mod initialpos;
mod node;
mod kifu;
mod trainer;
mod weight;

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
    g.start().unwrap();

    let tr = trainer::Trainer::new(0.01, 100);
    unsafe {
        tr.run(&g.kifu, &mut node::WEIGHT.as_mut().unwrap()).unwrap();
    }
}

fn gen_kifu() {
    for (idx, &rfen) in initialpos::RFENTBL.iter().enumerate() {
        // prepare game
        let mut g = game::Game::from(rfen);
        // play
        g.start().unwrap();
        // store kifu
        let kifuname = format!("./kifu/kifu{:06}.txt", idx);
        let mut f = File::create(kifuname).unwrap();
        f.write(g.kifu.to_str().as_bytes()).unwrap();
    }
}

fn training() {
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
    let mut tr = trainer::Trainer::new(0.001, 1000);
    tr.learn(&mut files);

    // put new eval table
    unsafe {
        node::WEIGHT.as_ref().unwrap().write("./kifu/newevaltable.txt");
    }
}

fn main() {
    println!("Hello, reversi world!");

    node::init_weight();

    // trial();

    // read command options

    // read eval table
    let path = "./evaltable.txt";
    if std::path::Path::new(path).exists() {
        println!("read eval table: {}", path);
        unsafe {
            node::WEIGHT.as_mut().unwrap().read(path).unwrap();
        }
    }

    // gen_kifu();

    training();
}
