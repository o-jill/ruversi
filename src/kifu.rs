use super::*;

pub const SENTEWIN : i8 = 1;
pub const DRAW : i8 = 0;
pub const GOTEWIN : i8 = -1;
pub const STR_POSX : &str = "0abcdefgh";
pub const HEADER : &str = "# reversi kifu revision 1.0\n";

pub struct Te {
    x : usize,
    y : usize,
    pub teban : i8,
    pub rfen : String,
}

impl Te {
    pub fn new(x : usize, y : usize, teban : i8, rfen : String) -> Te {
        Te {
            x,
            y,
            teban,
            rfen,
        }
    }

    pub fn from(line : &str) -> Option<Te> {
        if line.starts_with("#") {
            return None;
        }
        let elem = line.split_whitespace().collect::<Vec<&str>>();
        // nth teban posxy rfen rfen-teban
        if elem.len() != 5 {
            return None;
        }
        let teban = match elem[1] {
            board::STONE_SENTE => board::SENTE,
            board::STONE_GOTE => board::GOTE,
            _ => return None
        };
        let x : usize;
        let y : usize;
        if elem[2] == "PS" {
            x = 0;
            y = 0;
        } else {
            let c = elem[2].chars().nth(0).unwrap();
            x = STR_POSX.find(c)?;
            y = elem[2].chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
        }
        let rfen = format!("{} {}", elem[3], elem[4]);
        Some(Te {x, y, teban, rfen})
    }

    pub fn pos(&self) -> String {
        if self.x == 0 || self.y == 0 {
            return String::from("PS")
        }
        format!("{}{}", STR_POSX.chars().nth(self.x).unwrap(), self.y)
    }

    pub fn to_str(&self, i : usize) -> String {
        format!(
            "{} {} {} {}\n",
            i, match self.teban {
                board::SENTE => { board::STONE_SENTE },
                board::GOTE => { board::STONE_GOTE },
                _ => { "  " },
            },
            self.pos(), self.rfen)
    }
}

#[test]
fn testte() {
    let te = Te::new(0, 0, board::SENTE, "abcdefgh".to_string());
    assert_eq!(0, te.x);
    assert_eq!(0, te.y);
    assert_eq!(board::SENTE, te.teban);
    assert_eq!("abcdefgh", te.rfen);
    assert_eq!("PS", te.pos());
    assert_eq!("99 @@ PS abcdefgh\n", te.to_str(99));

    let te = Te::new(3, 4, board::GOTE, "ABCDEFGH".to_string());
    assert_eq!(3, te.x);
    assert_eq!(4, te.y);
    assert_eq!(board::GOTE, te.teban);
    assert_eq!("ABCDEFGH", te.rfen);
    assert_eq!("c4", te.pos());
    assert_eq!("23 [] c4 ABCDEFGH\n", te.to_str(23));

    let te = Te::from("");
    assert!(te.is_none());

    let te = Te::from("1 @@ a1 rfenb");
    assert!(te.is_none());

    let te = Te::from("1  @@ a1 rfenb");
    assert!(te.is_none());

    let te = Te::from("1 @@ a1 rfen b");
    assert!(te.is_some());
    let te = te.unwrap();
    assert_eq!(1, te.x);
    assert_eq!(1, te.y);
    assert_eq!(board::SENTE, te.teban);
    assert_eq!("rfen b", te.rfen);
    assert_eq!("a1", te.pos());
    assert_eq!("1 @@ a1 rfen b\n", te.to_str(1));

    let te = Te::from("2 [] h8 rfen w");
    assert!(te.is_some());
    let te = te.unwrap();
    assert_eq!(8, te.x);
    assert_eq!(8, te.y);
    assert_eq!(board::GOTE, te.teban);
    assert_eq!("rfen w", te.rfen);
    assert_eq!("h8", te.pos());
    assert_eq!("2 [] h8 rfen w\n", te.to_str(2));
}

pub struct Kifu {
    pub list: Vec<Te>,
    pub score : Option<i8>,
}

impl From<&Vec<&str>> for Kifu {
    fn from(lines : &Vec<&str>) -> Kifu {
        let mut ret = Kifu {
            list : Vec::<Te>::new(),
            score : None,
        };
        for &l in lines {
            let te = Te::from(l);
            if te.is_none() {
                continue;
            }
            ret.list.push(te.unwrap());
        }
        // score?
        let result = lines.last().unwrap();
        // println!("{:?}", result);
        let score = result.split(" ").collect::<Vec<&str>>();
        let score = score.last().unwrap().parse::<i8>().unwrap_or(0);
        ret.winneris(score);
        // if result.find("SENTE won.").is_some() {
        //     ret.score = Some(1);
        // } else if result.find("GOTE won.").is_some() {
        //     ret.score = Some(-1);
        // } else if result.find("DRAW.").is_some() {
        //     ret.score = Some(0);
        // }
        ret
    }
}

impl Kifu {
    pub fn new() -> Kifu {
        Kifu {
            list : Vec::<Te>::new(),
            score : None,
        }
    }

    #[allow(dead_code)]
    pub fn invalid() -> Kifu {
        Kifu {
            list : Vec::<Te>::new(),
            score : Some(100),
        }
    }

    #[allow(dead_code)]
    pub fn copy(&self) -> Kifu {
        let mut ret = Kifu::new();
        ret.score = self.score;
        for te in self.list.iter() {
            ret.append(te.x, te.y, te.teban, te.rfen.clone());
        }
        ret
    }
    pub fn append(&mut self, x : usize, y : usize, t : i8, rfen : String) {
        self.list.push(Te::new(x, y, t, rfen));
    }

    pub fn to_str(&self) -> String {
        let lines = self.list.iter().enumerate().map(
            |(i, a)| a.to_str(i + 1)).collect::<Vec<String>>();
        lines.join("") + &self.score2str()
    }

    #[allow(dead_code)]
    pub fn nth(&self, idx : usize) -> &Te {
        &self.list[idx]
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[allow(dead_code)]
    pub fn is_none(&self) -> bool {
        self.score.is_none()
    }

    #[allow(dead_code)]
    pub fn is_invalid(&self) -> bool {
        self.score.unwrap().abs() > 64
    }

    fn score2str(&self) -> String {
        if self.score.is_none() {
            return String::from("on going...");
        }
        let score = self.score.unwrap();
        if score.is_negative() {
            return format!("GOTE won. {score}");
        }
        if score.is_positive() {
            return format!("SENTE won. {score}");
        }
        String::from("DRAW.")
    }

    pub fn winneris(&mut self, n : i8) {
        self.score = Some(n);
    }

    pub fn winner(&self) -> Option<i8> {
        let score = self.score?;
        if score.is_positive() {
            return Some(SENTEWIN);
        }
        if score.is_negative() {
            return Some(GOTEWIN);
        }
        Some(DRAW)
    }
}

#[test]
fn testkifu() {
    // new, from, append, to_str, winneris, winner
    let kifu = Kifu::new();
    assert_eq!(0, kifu.list.len());
    assert_eq!(None, kifu.score);
    assert_eq!("on going...", kifu.to_str());

    let lines = "55 @@ h5 dD/AdC/BcC/BaAbAa/Af1/AaAaA1a1/BcC/G1 b\n\
    56 [] f6 dD/AdC/BcC/BaAbB/H/AaAaA1A1/BcC/G1 w\n\
    57 @@ PS dD/AdC/BcC/BdB/DbB/AaAcA1/BcC/G1 b\n\
    58 [] h6 dD/AdC/BcC/BdB/DbB/AaAcA1/BcC/G1 w\n\
    59 @@ PS dD/AdC/BcC/BdB/DcA/AaAe/BcC/G1 b\n\
    60 [] h8 dD/AdC/BcC/BdB/DcA/AaAe/BcC/G1 w\n\
    SENTE won. 4";
    let kifu = Kifu::from(&lines.split("\n").collect::<Vec<&str>>());
    let te = kifu.nth(0);
    assert_eq!(te.pos(), "h5");
    assert_eq!(te.x, 8);
    assert_eq!(te.y, 5);
    assert_eq!(te.teban, board::SENTE);
    assert_eq!(te.rfen, "dD/AdC/BcC/BaAbAa/Af1/AaAaA1a1/BcC/G1 b");
    let te = kifu.nth(1);
    assert_eq!(te.pos(), "f6");
    assert_eq!(te.x, 6);
    assert_eq!(te.y, 6);
    assert_eq!(te.teban, board::GOTE);
    assert_eq!(te.rfen, "dD/AdC/BcC/BaAbB/H/AaAaA1A1/BcC/G1 w");
    let te = kifu.nth(2);
    assert_eq!(te.pos(), "PS");
    assert_eq!(te.x, 0);
    assert_eq!(te.y, 0);
    assert_eq!(te.teban, board::SENTE);
    assert_eq!(te.rfen, "dD/AdC/BcC/BdB/DbB/AaAcA1/BcC/G1 b");
    assert_eq!(kifu.score, Some(4));
    assert_eq!(kifu.winner(), Some(SENTEWIN));
    assert_eq!(kifu.score2str(), "SENTE won. 4");
    let kifu2 = kifu.copy();
    assert_eq!(kifu.score, kifu2.score);
    for ((i, a), b) in kifu.list.iter().enumerate().zip(kifu2.list.iter()) {
        assert_eq!(a.to_str(i), b.to_str(i));
    }
}
