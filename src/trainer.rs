use super::*;

pub struct Trainer {
    eta: f32,
    repeat: usize,
}

impl Trainer {
    pub fn new(eta: f32, repeat: usize) -> Trainer {
        Trainer {
            eta: eta,
            repeat: repeat,
        }
    }

    pub fn run(&self, kifu: &kifu::Kifu, weight: &mut weight::Weight) -> Result<(), String> {
        let winner = kifu.winner();
        if winner.is_none() {
            return Err(String::from("invalid kifu."));
        }
        let winner = winner.unwrap();
        for l in kifu.list.iter() {
            if weight.train(&l.rfen, winner, self.eta).is_err() {
                return Err(String::from("error while training"));
            }
        }
        Ok(())
    }
}
