use super::*;

pub struct Game {
    ban : board::Board,
    pub kifu : kifu::Kifu,
}

impl Game {
    pub fn new() -> Game {
        Game {
            ban : board::Board::init(),
            kifu : kifu::Kifu::new(),
        }
    }

    pub fn from(rfen : &str) -> Game {
        Game {
            ban: board::Board::from(rfen).unwrap(),
            kifu: kifu::Kifu::new()
        }
    }

    pub fn start(&mut self, f : fn(&board::Board, usize) -> Option<(f32, node::Node)>, depth : usize) -> Result<(), String> {
        loop {
            // show
            self.ban.put();
            // think
            let st = Instant::now();
            let (val, node) = f(&self.ban, depth).unwrap();
            // let (val, node) = node::Node::think(&self.ban, 7).unwrap();
            // let (val, node) = node::Node::think_ab(&self.ban, 7).unwrap();
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

            // save to kifu
            self.kifu.append(x, y, teban, rfen);

            // check finished
            if self.ban.is_passpass() || self.ban.is_full() {
                break;
            }
        }
        // check who won
        self.kifu.winneris(self.ban.count());
        println!("{}", self.kifu.to_str());
        // show
        self.ban.put();
        Ok(())
    }
}
