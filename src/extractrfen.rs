use super::*;

pub fn extract(path : &str) {
    let content = std::fs::read_to_string(path).unwrap();
    let lines:Vec<&str> = content.split("\n").collect();
    let kifu = kifu::Kifu::from(&lines);
    let move1 = &kifu.list[0];
    let move2 = &kifu.list[1];
    let rfen = &kifu.list[2].rfen;
    let n = countmoves(&rfen);
    println!("\"{}\",  # {} {} {}", rfen, move1.pos(), move2.pos(), n);
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

/*class ExtractRfen

  def extract(path)
    # path = "./kifu/kifu000000000.txt"
    lines = read(path)

    # line1 move1
    move1 = lines[0].split(' ')[1]
    # line2 move2
    move2 = lines[1].split(' ')[1]
    # line3 rfen
    elem = lines[2].split(' ')
    rfen = elem[-2] + ' ' + elem[-1]
    puts "'#{rfen}', # #{move1} #{move2} #{countmoves(elem[-2])}"
  end
end

 */