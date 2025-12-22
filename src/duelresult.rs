use super::*;

const SENTE :usize = 0;
const GOTE :usize = 1;
const SENGO : usize = 2;

pub struct DuelResult {
    pub win : [u32 ; SENGO],
    pub draw : [u32 ; SENGO],
    pub lose : [u32 ; SENGO],
    pub total : u32,
}

impl Default for DuelResult {
    fn default() -> Self {
        DuelResult::new()
    }
}

impl std::fmt::Display for DuelResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.dump())
    }
}

impl DuelResult {
    pub fn new() -> DuelResult {
        DuelResult {
            win : [0 ; SENGO],
            draw : [0 ; SENGO],
            lose : [0 ; SENGO],
            total : 0,
        }
    }

    pub fn sresult(&mut self, winner : i8) {
        self.total += 1;
        match winner {
            kifu::SENTEWIN => {self.win[SENTE] += 1;},
            kifu::DRAW => {self.draw[SENTE] += 1;},
            kifu::GOTEWIN => {self.lose[SENTE] += 1;},
            _ => {}
        }
    }
    pub fn gresult(&mut self, winner : i8) {
        self.total += 1;
        match winner {
            kifu::SENTEWIN => {self.lose[GOTE] += 1;},
            kifu::DRAW => {self.draw[GOTE] += 1;},
            kifu::GOTEWIN => {self.win[GOTE] += 1;},
            _ => {}
        }
    }

    pub fn summary(win : &[u32], draw : &[u32], lose : &[u32], total : u32) -> String {
        let twin = win[SENTE] + win[GOTE];
        let tdraw = draw[SENTE] + draw[GOTE];
        let tlose = lose[SENTE] + lose[GOTE];
        let tsen = win[SENTE] + lose[GOTE] + tdraw;
        let tgo = win[GOTE] + lose[SENTE] + tdraw;
        let winrate = (twin as f64 - tdraw as f64 * 0.5) / total as f64;
        let winrate100 = 100.0 * winrate;
        const ELO : f64 = 400f64;
        let r = ELO * (twin as f64 / tlose as f64).log10();

        let err_margin = if total != 0 {
            let se = (winrate * (1.0 - winrate) / total as f64).sqrt();
            ELO / std::f64::consts::LN_10 * se
        } else {
            0.0
        };
        let confidence_interval = err_margin * 1.96;  // 95%信頼区間

        format!(
            r"total,win,draw,lose,balance-s,balance-g,winrate,R,95%
{total},{twin},{tdraw},{tlose},{tsen},{tgo},{winrate100:.2}%,{r:+.1},{confidence_interval:.1}
ev1   ,win,draw,lose
ev1 @@,{},{},{}
ev1 [],{},{},{}",
            win[SENTE], draw[SENTE], lose[SENTE],
            win[GOTE], draw[GOTE], lose[GOTE])
    }

    pub fn dump(&self) -> String {
        Self::summary(&self.win, &self.draw, &self.lose, self.total)
    }
}
