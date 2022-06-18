mod board;
mod node;
mod kifu;

fn main() {
    println!("Hello, reversi world!");
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
    let mut node = node::Node::new(0, 0, 7);
    let val = node::Node::think(&mut node, &ban);
    println!("val:{:?} {}", val, node.dump());

    println!("candidate:{:?}", ban.genmove());
    let ban2 = ban.r#move(3, 4).unwrap();
    ban2.put();
    println!("candidate:{:?}", ban2.genmove());
    let ban2 = ban2.r#move(3, 3).unwrap();
    ban2.put();
    println!("candidate:{:?}", ban2.genmove());

    let mut kifu = kifu::Kifu::new();
    kifu.append(1, 1, 1);
    kifu.append(2, 2, -1);
    kifu.append(3, 3, 1);
    kifu.append(4, 4, -1);
    kifu.append(5, 5, 1);
    kifu.append(6, 6, -1);
    kifu.append(7, 7, 0);
    print!("{}", kifu.to_str());
}
