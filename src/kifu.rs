use super::*;

struct Te {
    x : usize,
    y : usize,
    teban : i8,
    rfen : String,
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

    pub fn to_str(&self, i : usize) -> String {
        format!(
            "{} {} {}{} {}",
            i, match self.teban {
                board::SENTE => { "@@" },
                board::GOTE => { "[]" },
                _ => { "  "},
            },
            board::STR_GOTE.chars().nth(self.x).unwrap(), self.y, self.rfen)
    }
}

pub struct Kifu {
    list: Vec<Te>
}

impl Kifu {
    pub fn new() -> Kifu {
        Kifu {
            list : Vec::<Te>::new()
        }
    }

    pub fn append(&mut self, x : usize, y : usize, t : i8, rfen : String) {
        self.list.push(Te::new(x, y, t, rfen));
    }

    pub fn to_str(&self) -> String {
        let lines = self.list.iter().enumerate().map(
            |(i, a)| a.to_str(i)).collect::<Vec<String>>();
        lines.join("\n")
    }
}
