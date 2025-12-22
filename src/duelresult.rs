use super::*;

const SENTE :usize = 0;
const GOTE :usize = 1;
const SENGO : usize = 2;

#[derive(Debug)]
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
        write!(f, "{}", &self.to_string())
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

    pub fn winrate(&self) -> f64 {
        let twin = (self.win[SENTE] + self.win[GOTE]) as f64;
        let tdraw = (self.draw[SENTE] + self.draw[GOTE]) as f64;
        let winrate = (twin + tdraw * 0.5) / self.total as f64;
        winrate
    }

    /// calculate elo rating and 95% confidence interval
    /// 
    pub fn elo(&self) -> (f64, f64) {
        const ELO : f64 = 400f64;
        let winrate = self.winrate();

        let r = ELO * (winrate / (1.0 - winrate)).log10();

        let err_margin = if self.total != 0 {
            let se = (winrate * (1.0 - winrate) / self.total as f64).sqrt();
            ELO / std::f64::consts::LN_10 * se
        } else {
            0.0
        };

        (r, err_margin * 1.96)
    }

    fn to_string(&self) -> String {
        let twin = self.win[SENTE] + self.win[GOTE];
        let tdraw = self.draw[SENTE] + self.draw[GOTE];
        let tlose = self.lose[SENTE] + self.lose[GOTE];
        let tsen = self.win[SENTE] + self.lose[GOTE] + tdraw;
        let tgo = self.win[GOTE] + self.lose[SENTE] + tdraw;

        let winrate = self.winrate();
        let winrate100 = 100.0 * winrate;

        let (r, confidence_interval) = self.elo();

        format!(
            r"total,win,draw,lose,balance-s,balance-g,winrate,R,95%
{},{twin},{tdraw},{tlose},{tsen},{tgo},{winrate100:.2}%,{r:+.1},{confidence_interval:.1}
ev1   ,win,draw,lose
ev1 @@,{},{},{}
ev1 [],{},{},{}",
            self.total,
            self.win[SENTE], self.draw[SENTE], self.lose[SENTE],
            self.win[GOTE], self.draw[GOTE], self.lose[GOTE])
    }
}
