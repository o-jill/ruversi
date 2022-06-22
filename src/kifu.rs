use super::*;

pub const SENTEWIN : i8 = 1;
pub const DRAW : i8 = 0;
pub const GOTEWIN : i8 = -1;

pub struct Te {
    x : usize,
    y : usize,
    teban : i8,
    pub rfen : String,
}

impl Te {
    pub fn new(x : usize, y : usize, t : i8, rfen : String) -> Te {
        Te {
            x : x,
            y : y,
            teban : t,
            rfen : rfen
        }
    }

    pub fn pos(&self) -> String {
        if self.x == 0 || self.y == 0 {
            return String::from("PS")
        }
        format!("{}{}", board::STR_GOTE.chars().nth(self.x).unwrap(), self.y)
    }

    pub fn to_str(&self, i : usize) -> String {
        format!(
            "{} {} {} {}\n",
            i, match self.teban {
                board::SENTE => { "@@" },
                board::GOTE => { "[]" },
                _ => { "  "},
            },
            self.pos(), self.rfen)
    }
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

    pub fn append(&mut self, x : usize, y : usize, t : i8, rfen : String) {
        self.list.push(Te::new(x, y, t, rfen));
    }

    pub fn to_str(&self) -> String {
        let lines = self.list.iter().enumerate().map(
            |(i, a)| a.to_str(i)).collect::<Vec<String>>();
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
