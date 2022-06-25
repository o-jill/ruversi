// pub const SENTE : i8 = 1;
// pub const BLANK : i8 = 0;
// pub const GOTE : i8 = -1;
// pub const NONE : i8 = 127;
pub const NUMCELL : usize = 8;
pub const CELL_2D : usize = NUMCELL * NUMCELL;
const STR_SENTE : &str = "0ABCDEFGH";
pub const STR_GOTE : &str = "0abcdefgh";
const STR_NUM : &str = "012345678";
pub const STONE_SENTE : &str = "@@";
pub const STONE_GOTE : &str = "[]";

#[derive(Clone, Copy, PartialEq)]
pub enum Teban {
    Sente,
    Gote,
    Blank,
    None,
}

impl Teban {
    pub fn into(&self) -> i8 {
        if self.is_sente() {
            1
        } else if self.is_gote() {
            -1
        } else {
            0
        }
    }

    /* pub fn into(&self) -> f32 {
        match self {
            Sente => 1.0,
            Gote => -1.0,
            Blank => 0.0,
        }
    }*/

    pub fn from(s : &str) -> Option<Teban> {
        match s {
            "b" => Some(Teban::Sente),
            "w" => Some(Teban::Gote),
            "f" => Some(Teban::Blank),
            _ => None
        }
    }

    pub fn is_sente(&self) -> bool {
        *self == Teban::Sente
    }
    pub fn is_gote(&self) -> bool {
        *self == Teban::Gote
    }
    pub fn is_blank(&self) -> bool {
        *self == Teban::Blank
    }

    pub fn flip(&mut self) {
        *self =
            if self.is_sente() {
                Teban::Gote
            } else if self.is_gote() {
                Teban::Sente
            } else {
                Teban::Blank
            }
    }
}

impl From<Teban> for f32 {
    fn from(t: Teban) -> Self {
        if t.is_sente() {
            1.0
        } else if t.is_gote() {
            -1.0
        } else {
            0.0
        }
    }
}

pub struct Board {
    pub cells: Vec<Teban>,
    pub teban: Teban,
    pub pass: i8,
}

impl Board {
    pub fn new() -> Board {
        let mut ret = Board {
            cells : Vec::new(),
            teban : Teban::Sente,
            pass : 0,
        };
        ret.cells.resize(8 * 8, Teban::Blank);
        ret.cells[Board::index(3, 3)] = Teban::Sente;
        ret.cells[Board::index(4, 4)] = Teban::Sente;
        ret.cells[Board::index(3, 4)] = Teban::Gote;
        ret.cells[Board::index(4, 3)] = Teban::Gote;
        ret
    }

    pub fn from(rfen : &str) -> Result<Board, String> {
        let elem = rfen.split_whitespace().collect::<Vec<_>>();

        if elem.len() != 2 || elem[1].find(|c:char| c == 'b' || c == 'f' || c == 'w').is_none() {
            return Err(String::from("Invalid rfen"));
        }
        let teban = Teban::from(elem[1]);
        if teban.is_none() {
            return Err(format!("Invalid teban: {}", elem[1]));
        }
        let teban = teban.unwrap();
        let mut ret = Board {
            cells : Vec::new(),
            teban : teban,
            pass : 0,
        };
        ret.cells.resize(CELL_2D, Teban::Blank);
        let mut idx = 0;
        for ch in elem[0].chars() {
            match ch {
                'A'..='H' => {
                    let n = ch as i32 + 1 - 'A' as i32;
                    for _i in 0..n {
                        ret.cells[idx] = Teban::Sente;
                        idx += 1;
                    }
                },
                'a'..='h' => {
                    let n = ch as i32 + 1 - 'a' as i32;
                    for _i in 0..n {
                        ret.cells[idx] = Teban::Gote;
                        idx += 1;
                    }
                },
                '1'..='8' => {
                    idx += ch as usize - '0' as usize;
                },
                '/' => {},
                _ => {
                    return Err(format!("unknown letter rfen [{}]", ch));
                }
            }
        }
        Ok(ret)
    }

    pub fn init() -> Board {
        Board::from("8/8/8/3Aa3/3aA3/8/8/8 b").unwrap()
    }

    pub fn to_str(&self) -> String {
        let mut ban = Vec::<String>::new();
        for y in 0..NUMCELL {
            let mut old = Teban::None;
            let mut count = 0;
            let mut line = String::new();
            for x in 0..NUMCELL {
                let c = self.at(x, y);
                if c == old {
                    count += 1;
                    continue;
                }
                if old.is_sente() {
                    line += &STR_SENTE.chars().nth(count).unwrap().to_string();
                } else if old.is_gote() {
                    line += &STR_GOTE.chars().nth(count).unwrap().to_string();
                } else if old == Teban::Blank {
                    line += &STR_NUM.chars().nth(count).unwrap().to_string();
                }
                old = c;
                count = 1;
            }
            if old.is_sente() {
                line += &STR_SENTE.chars().nth(count).unwrap().to_string();
            } else if old.is_gote() {
                line += &STR_GOTE.chars().nth(count).unwrap().to_string();
            } else if old.is_blank() {
                line += &STR_NUM.chars().nth(count).unwrap().to_string();
            }
            ban.push(line);
        }
        ban.join("/") + match self.teban {
            Teban::Sente => { " b"}, Teban::Gote => {" w"}, _ => {" f"}
        }
    }

    pub fn put(&self) {
        for y in 0..NUMCELL {
            let mut line = String::new();
            for x in 0..NUMCELL {
                let c = self.at(x, y);
                line += "|";
                line +=
                    if c.is_sente() {
                        STONE_SENTE
                    } else if c.is_gote() {
                        STONE_GOTE
                    } else {
                        "__"
                    };
            }
            println!("{}|", line);
        }
        println!("{}", 
            match self.teban {
                Teban::Sente => { format!("{}'s turn.", STONE_SENTE)},
                Teban::Gote => { format!("{}'s turn.", STONE_GOTE)},
                _ => {"finished.".to_string()}
            }
        )
    }

    pub fn flipturn(&mut self) {
        self.teban.flip();
    }

    pub fn resetpass(&mut self) {
        self.pass = 0;
    }

    pub fn pass(&mut self) {
        self.teban.flip();
        self.pass += 1;
    }

    pub fn is_passpass(&self) -> bool {
        self.pass >= 2
    }
    pub fn clone(&self) -> Board {
        Board {
            cells: self.cells.to_vec(), teban: self.teban , pass: self.pass
        }
    }

    fn index(x: usize, y: usize) -> usize {
        x + y * NUMCELL
    }

    pub fn at(&self, x: usize, y: usize) -> Teban {
        self.cells[x + y * NUMCELL]
    }

    pub fn set(&mut self, x : usize, y : usize) {
        self.cells[Board::index(x, y)] = self.teban;
    }

    fn reverse(&mut self, x : usize, y : usize) {
        let color = self.teban;
        // 左
        for i in (0..x).rev() {
            let val = self.at(i, y);
            if val == color {
                for n in (i + 1)..x {
                    self.set(n, y);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 右
        for i in  (x + 1)..NUMCELL {
            let val = self.at(i, y);
            if val == color {
                for n in (x + 1)..i {
                    self.set(n, y);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 上
        for i in (0..y).rev() {
            let val = self.at(x, i);
            if val == color {
                for n in (i + 1)..y {
                    self.set(x, n);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 下
        for i in (y + 1)..NUMCELL {
            let val = self.at(x, i);
            if val == color {
                for n in (y + 1)..i {
                    self.set(x, n);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 左上
        for i in 1..NUMCELL {
            if x < i || y < i {
                break;
            }
            let val = self.at(x - i, y - i);
            if val == color {
                for n in 1..i {
                    self.set(x - n, y - n);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 右上
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y < i {
                break;
            }
            let val = self.at(x + i, y - i);
            if val == color {
                for n in 1..i {
                    self.set(x + n, y - n);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 右下
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y + i >= NUMCELL {
                break;
            }
            let val = self.at(x + i, y + i);
            if val == color {
                for n in 1..i {
                    self.set(x + n, y + n);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 左下
        for i in 1..NUMCELL {
            if x < i || y + i >= NUMCELL {
                break;
            }
            let val = self.at(x - i, y + i);
            if val == color {
                for n in 1..i {
                    self.set(x - n, y + n);
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }
    }

    pub fn checkreverse(&self, x : usize, y : usize) -> bool {
        let color = self.teban;
        // 左
        for i in (0..x).rev() {
            let val = self.at(i, y);
            if val == color {
                if i + 1 < x {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 右
        for i in  (x + 1)..NUMCELL {
            let val = self.at(i, y);
            if val == color {
                if x + 1 < i {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 上
        for i in (0..y).rev() {
            let val = self.at(x, i);
            if val == color {
                if i + 1 < y {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 下
        for i in (y + 1)..NUMCELL {
            let val = self.at(x, i);
            if val == color {
                if y + 1 < i {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 左上
        for i in 1..NUMCELL {
            if x < i || y < i {
                break;
            }
            let val = self.at(x - i, y - i);
            if val == color {
                if 1 < i {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 右上
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y < i {
                break;
            }
            let val = self.at(x + i, y - i);
            if val == color {
                if 1 < i {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 右下
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y + i >= NUMCELL {
                break;
            }
            let val = self.at(x + i, y + i);
            if val == color {
                if 1 < i {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }

        // 左下
        for i in 1..NUMCELL {
            if x < i || y + i >= NUMCELL {
                break;
            }
            let val = self.at(x - i, y + i);
            if val == color {
                if 1 < i {
                    return true;
                }
                break;
            }
            if val.is_blank() {
                break;
            }
        }
        false
    }

    pub fn r#move(&self, x : usize, y : usize) -> Result<Board, &str> {
        if x == 0 && y == 0 {  // pass
            let mut ban = self.clone();
            ban.pass();
            return Ok(ban);
        }

        let xc = x - 1;
        let yc = y - 1;
        if self.at(xc, yc) != Teban::Blank {
            return Err("stone exists.");
        }
        let mut ban = self.clone();
        ban.set(xc, yc);
        ban.reverse(xc, yc);
        ban.flipturn();
        ban.resetpass();

        Ok(ban)
    }

    pub fn genmove(&self) -> Option<Vec<(usize, usize)>> {
        let mut ret = Vec::<(usize, usize)>::new();
        let mut nblank = 0;
        for y in 0..8 {
            for x in 0..8 {
                let c = self.at(x, y);
                if !c.is_blank() {
                    continue;
                }
                nblank += 1;
                if self.checkreverse(x, y) {
                    ret.push((x + 1, y + 1));
                }
            }
        }
        if nblank == 0 {  // no more move
            return None;
        }
        if ret.is_empty() {  // pass
            return Some(vec![(0, 0)]);
        }
        Some(ret)
    }

    pub fn count(&self) -> i8 {
        let mut sum : i8 = 0;
        for c in self.cells.iter() {
            sum += *c as i8;
        }
        sum
    }

    pub fn is_full(&self) -> bool {
        for c in self.cells.iter() {
            if c.is_blank() {
                return false;
            }
        }
        true
    }

    pub fn rotate180(&self) -> Board {
        let mut b = Board::new();
        b.teban = self.teban;
        let cells = &self.cells;
        for (i, c) in b.cells.iter_mut().enumerate() {
            *c = cells[i];
        }
        b
    }
}
