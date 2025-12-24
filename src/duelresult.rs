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
        let twin = self.win[SENTE] + self.win[GOTE];
        let tdraw = self.draw[SENTE] + self.draw[GOTE];
        let tlose = self.lose[SENTE] + self.lose[GOTE];
        let tsen = self.win[SENTE] + self.lose[GOTE] + tdraw;
        let tgo = self.win[GOTE] + self.lose[SENTE] + tdraw;

        let winrate = self.winrate();
        let winrate100 = 100.0 * winrate;

        let (r, confidence_interval) = self.elo();

        write!(f,
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

impl DuelResult {
    pub fn new() -> DuelResult {
        DuelResult {
            win : [0 ; SENGO],
            draw : [0 ; SENGO],
            lose : [0 ; SENGO],
            total : 0,
        }
    }

    /// # Returns
    ///   new instance whose sen-go result was switched.
    #[allow(dead_code)]
    pub fn exchanged(&self) -> DuelResult {
        let mut ret = Self::new();
        ret.win[SENTE] = self.win[GOTE];
        ret.win[SENTE] = self.win[GOTE];
        ret.lose[SENTE] = self.win[GOTE];
        ret.lose[GOTE] = self.win[SENTE];
        ret.draw[GOTE] = self.win[SENTE];
        ret.draw[GOTE] = self.win[SENTE];
        ret
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
        (twin + tdraw * 0.5) / self.total as f64
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
}


#[test]
fn test_duel_result() {
    let dr = DuelResult::new();
    assert_eq!(dr.win, [0 ; SENGO]);
    assert_eq!(dr.lose, [0 ; SENGO]);
    assert_eq!(dr.draw, [0 ; SENGO]);
    assert_eq!(dr.total, 0);

    let mut dr = DuelResult::new();
    dr.sresult(kifu::SENTEWIN);
    assert_eq!(dr.win[SENTE], 1);
    assert_eq!(dr.win[GOTE], 0);
    assert_eq!(dr.lose[SENTE], 0);
    assert_eq!(dr.lose[GOTE], 0);
    assert_eq!(dr.draw[SENTE], 0);
    assert_eq!(dr.draw[GOTE], 0);
    assert_eq!(dr.total, 1);

    let mut dr = DuelResult::new();
    dr.sresult(kifu::GOTEWIN);
    assert_eq!(dr.win[SENTE], 0);
    assert_eq!(dr.win[GOTE], 0);
    assert_eq!(dr.lose[SENTE], 1);
    assert_eq!(dr.lose[GOTE], 0);
    assert_eq!(dr.draw[SENTE], 0);
    assert_eq!(dr.draw[GOTE], 0);
    assert_eq!(dr.total, 1);

    let mut dr = DuelResult::new();
    dr.sresult(kifu::DRAW);
    assert_eq!(dr.win[SENTE], 0);
    assert_eq!(dr.win[GOTE], 0);
    assert_eq!(dr.lose[SENTE], 0);
    assert_eq!(dr.lose[GOTE], 0);
    assert_eq!(dr.draw[SENTE], 1);
    assert_eq!(dr.draw[GOTE], 0);
    assert_eq!(dr.total, 1);

    let mut dr = DuelResult::new();
    dr.gresult(kifu::SENTEWIN);
    assert_eq!(dr.win[SENTE], 0);
    assert_eq!(dr.win[GOTE], 0);
    assert_eq!(dr.lose[SENTE], 0);
    assert_eq!(dr.lose[GOTE], 1);
    assert_eq!(dr.draw[SENTE], 0);
    assert_eq!(dr.draw[GOTE], 0);
    assert_eq!(dr.total, 1);

    let mut dr = DuelResult::new();
    dr.gresult(kifu::GOTEWIN);
    assert_eq!(dr.win[SENTE], 0);
    assert_eq!(dr.win[GOTE], 1);
    assert_eq!(dr.lose[SENTE], 0);
    assert_eq!(dr.lose[GOTE], 0);
    assert_eq!(dr.draw[SENTE], 0);
    assert_eq!(dr.draw[GOTE], 0);
    assert_eq!(dr.total, 1);

    let mut dr = DuelResult::new();
    dr.gresult(kifu::DRAW);
    assert_eq!(dr.win[SENTE], 0);
    assert_eq!(dr.win[GOTE], 0);
    assert_eq!(dr.lose[SENTE], 0);
    assert_eq!(dr.lose[GOTE], 0);
    assert_eq!(dr.draw[SENTE], 0);
    assert_eq!(dr.draw[GOTE], 1);
    assert_eq!(dr.total, 1);

    let mut dr = DuelResult::new();
    dr.win[SENTE] = 2;
    dr.lose[SENTE] = 3;
    dr.draw[SENTE] = 4;
    dr.win[GOTE] = 3;
    dr.lose[GOTE] = 7;
    dr.draw[GOTE] = 16;
    dr.total = 35;
    assert!((dr.winrate() - 0.428571429).abs() < 0.0001);
    let (elo, margin) = dr.elo();
    assert!((elo - (-49.9755)).abs() < 0.0001);
    assert!((margin - 14.5312463 * 1.96).abs() < 0.0001);

    assert_eq!(dr.to_string(),
               r"total,win,draw,lose,balance-s,balance-g,winrate,R,95%
35,5,20,10,29,26,42.86%,-50.0,28.5
ev1   ,win,draw,lose
ev1 @@,2,4,3
ev1 [],3,16,7");
}
