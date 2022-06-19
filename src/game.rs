use super::*;

pub struct Game {
    ban : board::Board,
    pass : i32,
    kifu : kifu::Kifu,
}

impl Game {
    pub fn new() -> Game {
        Game {
            ban : board::Board::init(),
            pass : 0,
            kifu : kifu::Kifu::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        loop {
            // show
            self.ban.put();
            // think
            let st = Instant::now();
            let mut node = node::Node::new(0, 0, 5);
            let val = node::Node::think(&mut node, &self.ban);
            let ft = st.elapsed();
            println!("val:{:?} {} {}msec", val, node.dump(), ft.as_millis());
            let best = node.best.unwrap();
            let x = best.x;
            let y = best.y;
            // apply move
            let ban = self.ban.r#move(x, y).unwrap();
            let rfen = self.ban.to_str();
            let teban = self.ban.teban;
            self.ban = ban;
            if x == 0 && y == 0 {
                self.pass += 1
            } else {
                self.pass = 0
            }

            // save to kifu
            self.kifu.append(x, y, teban, rfen);

            // check finished
            if self.pass == 2 || self.ban.is_full() {
                break;
            }
        }
        // check who won
        self.kifu.winner(self.ban.count());
        println!("{}", self.kifu.to_str());
        Ok(())
    }
}
