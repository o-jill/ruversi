use std::time::{Instant};
use std::thread;
use std::sync::mpsc;

mod board;
mod game;
mod node;
mod kifu;
mod weight;

fn main() {
    println!("Hello, reversi world!");
    node::init_weight();
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
    let mut node = node::Node::new(0, 0, 7);
    let val = node::Node::think(&mut node, &ban);
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
            thread::sleep_ms(1000 as u32);
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
    g.start();
}
