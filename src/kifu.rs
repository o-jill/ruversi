use super::*;

pub const SENTEWIN : i8 = 1;
pub const DRAW : i8 = 0;
pub const GOTEWIN : i8 = -1;
pub const STR_POSX : &str = "0abcdefgh";

pub struct Te {
    x : usize,
    y : usize,
    teban : board::Teban,
    pub rfen : String,
}

impl Te {
    pub fn new(x : usize, y : usize, t : board::Teban, rfen : String) -> Te {
        Te {
            x : x,
            y : y,
            teban : t,
            rfen : rfen
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
            board::STONE_SENTE => board::Teban::Sente,
            board::STONE_GOTE => board::Teban::Gote,
            _ => return None
        };
        let x : usize;
        let y : usize;
        if elem[2] == "PS" {
            x = 0;
            y = 0;
        } else {
            let c = elem[2].chars().nth(0).unwrap();
            let ox = STR_POSX.find(c);
            if ox.is_none() {
                return None;
            }
            x = ox.unwrap();
            y = elem[2].chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
        }
        let rfen = format!("{} {}", elem[3], elem[4]);
        Some(Te {
            x : x,
            y : y,
            teban : teban,
            rfen : rfen
        })
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
                board::Teban::Sente => { board::STONE_SENTE },
                board::Teban::Gote => { board::STONE_GOTE },
                _ => { "  " },
            },
            self.pos(), self.rfen)
    }
}

#[test]
fn testte() {
    let te = Te::new(0, 0, board::Teban::Sente, "abcdefgh".to_string());
    assert_eq!(0, te.x);
    assert_eq!(0, te.y);
    assert!(te.teban.is_sente());
    assert_eq!("abcdefgh", te.rfen);
    assert_eq!("PS", te.pos());
    assert_eq!("99 @@ PS abcdefgh\n", te.to_str(99));

    let te = Te::new(3, 4, board::Teban::Gote, "ABCDEFGH".to_string());
    assert_eq!(3, te.x);
    assert_eq!(4, te.y);
    assert!(te.teban.is_gote());
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
    assert!(te.teban.is_sente());
    assert_eq!("rfen b", te.rfen);
    assert_eq!("a1", te.pos());
    assert_eq!("1 @@ a1 rfen b\n", te.to_str(1));

    let te = Te::from("2 [] h8 rfen w");
    assert!(te.is_some());
    let te = te.unwrap();
    assert_eq!(8, te.x);
    assert_eq!(8, te.y);
    assert!(te.teban.is_gote());
    assert_eq!("rfen w", te.rfen);
    assert_eq!("h8", te.pos());
    assert_eq!("2 [] h8 rfen w\n", te.to_str(2));
}

pub struct Kifu {
    pub list: Vec<Te>,
    score : Option<i8>,
}

impl Kifu {
    pub fn new() -> Kifu {
        Kifu {
            list : Vec::<Te>::new(),
            score : None,
        }
    }

    pub fn from(lines : &Vec<&str>) -> Kifu {
        let mut ret = Kifu {
            list : Vec::<Te>::new(),
            score : None,
        };
        for &l in lines {
            let te = Te::from(&l);
            if te.is_none() {
                continue;
            }
            ret.list.push(te.unwrap());
        }
        // score?
        let result = lines.last().unwrap();
        if result.find("SENTE won.").is_some() {
            ret.score = Some(1);
        } else if result.find("GOTE won.").is_some() {
            ret.score = Some(-1);
        } else if result.find("DRAW.").is_some() {
            ret.score = Some(0);
        }
        ret
    }

    pub fn append(&mut self, x : usize, y : usize, t : board::Teban, rfen : String) {
        self.list.push(Te::new(x, y, t, rfen));
    }

    pub fn to_str(&self) -> String {
        let lines = self.list.iter().enumerate().map(
            |(i, a)| a.to_str(i + 1)).collect::<Vec<String>>();
        lines.join("") + &self.score2str()
    }

    fn score2str(&self) -> String {
        if self.score.is_none() {
            return String::from("on going...");
        }
        let score = self.score.unwrap();
        if score.is_negative() {
            return format!("GOTE won. {}", score);
        }
        if score.is_positive() {
            return format!("SENTE won. {}", score);
        }
        String::from("DRAW.")
    }

    pub fn winneris(&mut self, n : i8) {
        self.score = Some(n);
    }

    pub fn winner(&self) -> Option<i8> {
        if self.score.is_none() {
            return None;
        }
        let score = self.score.unwrap();
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
}
