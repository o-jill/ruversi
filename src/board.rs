const SENTE : i8 = 1;
const BLANK : i8 = 0;
const GOTE : i8 = -1;
const NONE : i8 = 127;
const NUMCELL : usize = 8;
const CELL_2D : usize = NUMCELL * NUMCELL;
const STR_SENTE : &str = "0ABCDEFGH";
const STR_GOTE : &str = "0abcdefgh";
const STR_NUM : &str = "012345678";

pub struct Board {
    cells: Vec<i8>,
    teban: i8,
}

impl Board {
    pub fn new() -> Board {
        let mut ret = Board {
            cells : Vec::new(),
            teban : SENTE,
        };
        ret.cells.resize(8 * 8, BLANK);
        ret.cells[index(3, 3)] = SENTE;
        ret.cells[index(4, 4)] = SENTE;
        ret.cells[index(3, 4)] = GOTE;
        ret.cells[index(4, 3)] = GOTE;
        ret
    }

    pub fn from(rfen : &str) -> Result<Board, String> {
        let elem = rfen.split_whitespace().collect::<Vec<_>>();

        if elem.len() != 2 || elem[1].find(|c:char| c == 'b' || c == 'f' || c == 'w').is_none() {
            return Err(String::from("Invalid rfen"));
        }
        let teban;
        match elem[1] {
            "b" => {teban = SENTE},
            "w" => {teban = GOTE},
            "f" => {teban = BLANK}
            _ => { return Err(format!("Invalid teban: {}", elem[1])); }
        }
        let mut ret = Board {
            cells : Vec::new(),
            teban : teban,
        };
        ret.cells.resize(CELL_2D, BLANK);
        let mut idx = 0;
        for ch in elem[0].chars() {
            match ch {
                'A'..='H' => {
                    let n = ch as i32 + 1 - 'A' as i32;
                    for _i in 0..n {
                        ret.cells[idx] = SENTE;
                        idx += 1;
                    }
                },
                'a'..='h' => {
                    let n = ch as i32 + 1 - 'a' as i32;
                    for _i in 0..n {
                        ret.cells[idx] = GOTE;
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
            let mut old = NONE;
            let mut count = 0;
            let mut line = String::new();
            for x in 0..NUMCELL {
                let c = self.cells[index(x, y)];
                if c == old {
                    count += 1;
                    continue;
                }
                if old == SENTE {
                    line += &STR_SENTE.chars().nth(count).unwrap().to_string();
                } else if old == GOTE {
                    line += &STR_GOTE.chars().nth(count).unwrap().to_string();
                } else if old == BLANK {
                    line += &STR_NUM.chars().nth(count).unwrap().to_string();
                }
                old = c;
                count = 1;
            }
            if old == SENTE {
                line += &STR_SENTE.chars().nth(count).unwrap().to_string();
            } else if old == GOTE {
                line += &STR_GOTE.chars().nth(count).unwrap().to_string();
            } else if old == BLANK {
                line += &STR_NUM.chars().nth(count).unwrap().to_string();
            }
            ban.push(line);
        }
        ban.join("/") + match self.teban {
            SENTE => { " b"}, GOTE => {" w"}, _ => {" f"}
        }
    }

    pub fn put(&self) {
        for y in 0..NUMCELL {
            let mut line = String::new();
            for x in 0..NUMCELL {
                let c = self.cells[index(x, y)];
                line +=
                    if c == SENTE {
                        "|@@"
                    } else if c == GOTE {
                        "|[]"
                    } else {
                        "|__"
                    };
            }
            println!("{}|", line);
        }
        println!("{}", 
            match self.teban {
                SENTE => { "@@'s turn."},
                GOTE => { "[]'s turn."},
                _ => {"finished."}
            }
        )
    }

    pub fn flipturn(&mut self) {
        self.teban = -self.teban;
    }

    pub fn clone(&self) -> Board {
        Board { cells: self.cells.to_vec(), teban: self.teban }
    }

    fn index(x: usize, y: usize) -> usize {
        x + y * NUMCELL
    }

    pub fn at(&self, x: usize, y: usize) -> i8 {
        self.cells[x + y * NUMCELL]
    }

    pub fn set(&mut self, x : usize, y : usize) {
        self.cells[index(x, y)] = self.teban;
    }
}
