use super::*;

pub fn extract(path : &str) -> kifu::Kifu {
    let content = std::fs::read_to_string(path).unwrap();
    let lines:Vec<&str> = content.split("\n").collect();
    let kifu = kifu::Kifu::from(&lines);
    let move1 = &kifu.list[0];
    let move2 = &kifu.list[1];
    let rfen = &kifu.list[2].rfen;
    let n = countmoves(&rfen);
    println!("\"{}\", // {} {} {}", rfen, move1.pos(), move2.pos(), n);
    return kifu;
}

const BLK : &str = " ABCDEFGH";
const WHT : &str = " abcdefgh";

fn countmoves(rfen : &str) -> i32{
    let mut moves : i32 = -4;

    for rank in rfen.split("/").collect::<Vec<&str>>() {
        for ch in rank.chars() {
            if ch == ' ' {
                break;
            }
             let idx = BLK.find(ch);
            if idx.is_some() {
                moves += idx.unwrap() as i32;
                continue;
            }
            let idx = WHT.find(ch);
            if idx.is_some() {
                moves += idx.unwrap() as i32;
            }
        }
    }
    moves
}
