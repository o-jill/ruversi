// use std::arch::x86_64;

pub const SENTE : i8 = 1;
pub const BLANK : i8 = 0;
pub const GOTE : i8 = -1;
pub const NONE : i8 = 127;
pub const NUMCELL : usize = 8;
const CELL_2D : usize = NUMCELL * NUMCELL;
const STR_SENTE : &str = "0ABCDEFGH";
pub const STR_GOTE : &str = "0abcdefgh";
const STR_NUM : &str = "012345678";
pub const STONE_SENTE : &str = "@@";
pub const STONE_GOTE : &str = "[]";
pub const MSB_CELL : u64 = 0x8000000000000000;
const LT_CELL : u64 = MSB_CELL;
const RT_CELL : u64 = 0x0100000000000000;
const LB_CELL : u64 = 0x0000000000000080;
const RB_CELL : u64 = 0x0000000000000001;

pub struct BitBoard {
    pub black: u64,
    pub white: u64,
    pub teban: i8,
    pub pass: i8,
}

impl BitBoard {
    pub fn new() -> BitBoard {
        BitBoard {
            black : 0x0000001008000000,
            white : 0x0000000810000000,
            teban : SENTE,
            pass : 0,
        }
    }

    pub fn from(rfen : &str) -> Result<BitBoard, String> {
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
        let mut ret = BitBoard {
            black : 0,
            white : 0,
            teban : teban,
            pass : 0,
        };
        let mut bit : u64 = MSB_CELL;
        for ch in elem[0].chars() {
            match ch {
                'A'..='H' => {
                    let n = ch as i32 + 1 - 'A' as i32;
                    for _i in 0..n {
                        ret.black |= bit;
                        bit >>= 1;
                    }
                },
                'a'..='h' => {
                    let n = ch as i32 + 1 - 'a' as i32;
                    for _i in 0..n {
                        ret.white |= bit;
                        bit >>= 1;
                    }
                },
                '1'..='8' => {
                    let n = ch as i32 - '0' as i32;
                    bit >>= n;
                },
                '/' => {},
                _ => {
                    return Err(format!("unknown letter rfen [{}]", ch));
                }
            }
        }

        Ok(ret)
    }

    pub fn to_str(&self) -> String {
        let mut ban = Vec::<String>::new();
        let mut bit : u64 = MSB_CELL;
        let black = self.black;
        let white = self.white;
        for _y in 0..NUMCELL {
            let mut old = NONE;
            let mut count = 0;
            let mut line = String::new();
            for _x in 0..NUMCELL {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                // println!("bit:0x{:016x}, cb:{}, cw:{}", bit, cb, cw);
                bit >>= 1;
                let c = if cb {SENTE} else if cw {GOTE} else {BLANK};
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

    // othello BitBoard file format
    // init:
    // ---------------------------XO------OX--------------------------- X
    //
    pub fn to_obf(&self) -> String {
        let mut ban = String::new();
        let mut bit : u64 = MSB_CELL;
        let black = self.black;
        let white = self.white;
        for _i in 0..CELL_2D {
            let cb = (bit & black) != 0;
            let cw = (bit & white) != 0;
            bit >>= 1;
            ban += if cb {"X"} else if cw {"O"} else {"-"};
        }
        ban + match self.teban {
            SENTE => " X",
            GOTE => " O",
            _ => " -",
        }
    }

    pub fn to_id(&self)-> [u8 ; 16] {
        let mut res : [u8 ; 16] = [0 ; 16];
        let mut bit : u64 = MSB_CELL;
        let black = self.black;
        let white = self.white;
        let tbn : u8 = if self.teban == SENTE { 0x00 } else { 0x80 };
        let mut idx = 0;
        for _y in 0..NUMCELL {
            let mut id : u8 = 0;
            for _x in 0..4 {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit >>= 1;
                let c = if cb {SENTE} else if cw {GOTE} else {BLANK};

                id = id * 3 + (c + 1) as u8;
            }
            res[idx] = id | tbn;
            idx += 1;

            id = 0;
            for _x in 0..4 {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit >>= 1;
                let c = if cb {SENTE} else if cw {GOTE} else {BLANK};

                id = id * 3 + (c + 1) as u8;
            }
            res[idx] = id | tbn;
            idx += 1;
        }
        res
    }

    pub fn to_id_simd(&self)-> [u8 ; 16] {
        self.to_id()
        // let mut res : [u8 ; 16] = [0 ; 16];
        // let tbn : i8 = if self.teban == SENTE { 0x00 } else { -128 };
        // unsafe {
        //     let mut sum16 = x86_64::_mm_setzero_si128();
        //     for i in 0..CELL_2D / 16 {
        //         let ci816 = x86_64::_mm_load_si128(
        //             self.cells[i * 16..].as_ptr() as *const x86_64::__m128i);
        //         // -1 ~ +1 -> 0 ~ 2
        //         let one16 = x86_64::_mm_set1_epi8(1);
        //         let cu816 = x86_64::_mm_add_epi8(ci816, one16);

        //         let three8 = x86_64::_mm_set1_epi16(3);
        //         sum16 = x86_64::_mm_mullo_epi16(three8, sum16);
        //         sum16 = x86_64::_mm_add_epi16(sum16, cu816);
        //     }
        //     let tbn16 = x86_64::_mm_set1_epi8(tbn);
        //     let sum16 = x86_64::_mm_or_si128(tbn16, sum16);
        //     x86_64::_mm_store_si128(
        //         res.as_mut_ptr() as *mut x86_64::__m128i, sum16);
        // }
        // res
    }

    pub fn put(&self) {
        let mut bit : u64 = MSB_CELL;
        let black = self.black;
        let white = self.white;
        for _y in 0..NUMCELL {
            let mut line = String::new();
            for _x in 0..NUMCELL {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit >>= 1;

                line += "|";
                line +=
                    if cb {
                        STONE_SENTE
                    } else if cw {
                        STONE_GOTE
                    } else {
                        "__"
                    };
            }
            println!("{}|", line);
        }
        println!("{}", 
            match self.teban {
                SENTE => { format!("{}'s turn.", STONE_SENTE)},
                GOTE => { format!("{}'s turn.", STONE_GOTE)},
                _ => {"finished.".to_string()}
            }
        )
    }

    pub fn flipturn(&mut self) {
        self.teban = -self.teban;
    }

    pub fn resetpass(&mut self) {
        self.pass = 0;
    }

    pub fn pass(&mut self) {
        self.teban = -self.teban;
        self.pass += 1;
    }

    pub fn is_passpass(&self) -> bool {
        self.pass >= 2
    }
    pub fn clone(&self) -> BitBoard {
        BitBoard {
            black : self.black, white : self.white,
            teban: self.teban , pass: self.pass
        }
    }

    fn index(x: usize, y: usize) -> usize {
        x + y * NUMCELL
    }

    pub fn at(&self, x: usize, y: usize) -> i8 {
        let bit : u64 = MSB_CELL >> (x + y * NUMCELL);
        let cb = (bit & self.black) != 0;
        let cw = (bit & self.white) != 0;

        if cb {SENTE} else if cw {GOTE} else {BLANK}
    }

    pub fn set(&mut self, x : usize, y : usize) {
        let bit = MSB_CELL >> (x + y * NUMCELL);
        let mask = !bit;
        if self.teban == SENTE {
            self.black |= bit;
            self.white &= mask;
        } else {
            self.black &= mask;
            self.white |= bit;
        }
    }

    /**
     * x : 0~7
     * y : 0~7
     */
    fn reverse(&mut self, x : usize, y : usize) {
        if x > 7 || y > 7 {
            panic!("{},{} is out of range.", x, y);
        }

        let color = self.teban;
        let mut mine = if color == SENTE {self.black} else {self.white};
        let mut oppo = if color == SENTE {self.white} else {self.black};

        let pos = MSB_CELL >> BitBoard::index(x, y);

        let mask = !pos;
        mine |= pos;
        oppo &= mask;

        // 左
        let mut bit : u64 = pos << 1;
        let mut rev : u64 = 0;
        for _i in 0..x {
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 1;
        }

        // 右
        let mut bit : u64 = pos >> 1;
        let mut rev : u64 = 0;
        for _i in x..NUMCELL {
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 1;
        }

        // 上
        let mut bit : u64 = pos << NUMCELL;
        let mut rev : u64 = 0;
        for _i in 0..y {
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 8;
        }

        // 下
        let mut bit : u64 = pos >> NUMCELL;
        let mut rev : u64 = 0;
        for _i in y..NUMCELL {
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 8;
        }

        // 左上
        let mut bit : u64 = pos << (NUMCELL + 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x < i || y < i {
                break;
            }
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 9;
        }

        // 右上
        let mut bit : u64 = pos << (NUMCELL - 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y < i {
                break;
            }
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 7;
        }

        // 右下
        let mut bit : u64 = pos >> (NUMCELL + 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y + i >= NUMCELL {
                break;
            }
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 9;
        }

        // 左下
        let mut bit : u64 = pos >> (NUMCELL - 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x < i || y + i >= NUMCELL {
                break;
            }
            if (mine & bit) != 0 {
                oppo &= !rev;
                mine |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 7;
        }

        if color == SENTE {
            self.black = mine;
            self.white = oppo;
        } else {
            self.white = mine;
            self.black = oppo;
        }
    }

    pub fn checkreverse(&self, x : usize, y : usize) -> bool {
        let color = self.teban;
        let &mut mine = &mut if color == SENTE {self.black} else {self.white};
        let &mut oppo = &mut if color == SENTE {self.white} else {self.black};
        let pos = MSB_CELL >> BitBoard::index(x, y);
        // 左
        let mut bit : u64 = pos << 1;
        let mut rev : u64 = 0;
        for _i in 0..x {
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 1;
        }

        // 右
        let mut bit : u64 = pos >> 1;
        let mut rev : u64 = 0;
        for _i in x..NUMCELL {
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 1;
        }

        // 上
        let mut bit : u64 = pos << NUMCELL;
        let mut rev : u64 = 0;
        for _i in 0..y {
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 8;
        }

        // 下
        let mut bit : u64 = pos >> NUMCELL;
        let mut rev : u64 = 0;
        for _i in y..NUMCELL {
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 8;
        }

        // 左上
        let mut bit : u64 = pos << (NUMCELL + 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x < i || y < i {
                break;
            }
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 9;
        }

        // 右上
        let mut bit : u64 = pos << (NUMCELL - 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y < i {
                break;
            }
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit <<= 7;
        }

        // 右下
        let mut bit : u64 = pos >> (NUMCELL + 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x + i >= NUMCELL || y + i >= NUMCELL {
                break;
            }
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 9;
        }

        // 左下
        let mut bit : u64 = pos >> (NUMCELL - 1);
        let mut rev : u64 = 0;
        for i in 1..NUMCELL {
            if x < i || y + i >= NUMCELL {
                break;
            }
            if (mine & bit) != 0 {
                if rev != 0 {return true;}
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }
            bit >>= 7;
        }

        false
    }

    /**
     * @param x 0 : pass, 1 ~ 8 : column index.
     * @param y 0 : pass, 1 ~ 8 : row index.
     */
    pub fn r#move(&self, x : usize, y : usize) -> Result<BitBoard, &str> {
        if x == 0 && y == 0 {  // pass
            let mut ban = self.clone();
            ban.pass();
            return Ok(ban);
        }

        let xc = x - 1;
        let yc = y - 1;
        if self.at(xc, yc) != BLANK {
            return Err("stone exists.");
        }
        let mut ban = self.clone();
        // ban.set(xc, yc);
        ban.reverse(xc, yc);
        ban.flipturn();
        ban.resetpass();

        Ok(ban)
    }

    pub fn genmove(&self) -> Option<Vec<(usize, usize)>> {
        let mut ret = Vec::<(usize, usize)>::new();
        let mut nblank = 0;
        let mut bit = MSB_CELL;
        let black = self.black;
        let white = self.white;
        for y in 0..NUMCELL {
            for x in 0..NUMCELL {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit >>= 1;
                if cb || cw {
                    continue;
                }
                nblank += 1;
                if self.checkreverse(x, y) {
                    ret.push((x + 1, y + 1));
                }
            }
        }
        if ret.is_empty() {  // pass
            return if nblank == 0 {
                None
            } else {
                Some(vec![])
                // Some(vec![(0, 0)])
            }
        }
        Some(ret)
    }

    pub fn count(&self) -> i8 {
        self.black.count_ones() as i8 - self.white.count_ones() as i8
    }

    pub fn is_full(&self) -> bool {
        (self.black | self.white) == 0xffffffffffffffff
    }

    pub fn rotate180(&self) -> BitBoard {
        let mut b = BitBoard::new();
        b.teban = self.teban;
        b.black = self.black.reverse_bits();
        b.white = self.white.reverse_bits();
        b
    }

    pub fn fixedstones(&self) -> (i8, i8) {
        // return (0, 0);  // この関数が遅いのかを見極める用
        let mut count = 0;
        let mut fcellsb : u64 = 0;
        let mut fcellsw : u64 = 0;
        let black = self.black;
        let white = self.white;
    
        // 四隅と辺
        let mut bit = LT_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsb |= bit;
                bit >>= 1;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = LT_CELL;
            for _i in 1..7 {  // ↓
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit >>= 8;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsw |= bit;
                bit >>= 1;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = LT_CELL;
            for _i in 1..7 {  // ↓
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit >>= 8;
                count += 1;
            }
        }
        let mut bit = RT_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsb |= bit;
                bit <<= 1;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = RT_CELL;
            for _i in 1..7 {  // ↓
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit >>= 8;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsw |= bit;
                bit <<= 1;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = RT_CELL;
            for _i in 1..7 {  // ↓
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit >>= 8;
                count += 1;
            }
        }
        let mut bit = LB_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsb |= bit;
                bit >>= 1;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = LB_CELL;
            for _i in 1..7 {  // ↑
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit <<= 8;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsw |= bit;
                bit >>= 1;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = LB_CELL;
            for _i in 1..7 {  // ↑
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit <<= 8;
                count += 1;
            }
        }
        let mut bit = RB_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsb |= bit;
                bit <<= 1;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = RB_CELL;
            for _i in 1..7 {  // ↑
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit <<= 8;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsw |= bit;
                bit <<= 1;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = RB_CELL;
            for _i in 1..7 {  // ↑
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit <<= 8;
                count += 1;
            }
        }
        if count < 4 {
            return (fcellsb.count_ones() as i8, fcellsw.count_ones() as i8);
        }
        // 中身
        // こんな感じなら確定石
        // xx?  x??
        // x@?  x@?
        // x??  xx?
        for x in 1..7 {
            let mut cnt = 0;
            for y in 1..7 {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 左3つ fcells[] == @
                // 上 fcells[] == @
                let wbit = 0xC080800000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            for y in (1..7).rev() {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 左3つ fcells[] == @
                // 下 fcells[] == @
                let wbit = 0x8080C00000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            if cnt == 0 {break;}
        }
        // xxx  xxx
        // x@?  ?@x
        // ???  ???
        for y in 1..7 {
            let mut cnt = 0;
            for x in 1..7 {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 上3つ fcells[] == @
                // 左 fcells[] == @
                let wbit = 0xE080000000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            for x in (1..7).rev() {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 上3つ fcells[] == @
                // 右 fcells[] == @
                let wbit = 0xE020000000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            if cnt == 0 {break;}
        }
        //
        // ?xx  ??x
        // ?@x  ?@x
        // ??x  ?xx
        for x in (1..7).rev() {
            let mut cnt = 0;
            for y in 1..7 {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 右3つ fcells[] == @
                // 上 fcells[] == @
                let wbit = 0x6020200000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            for y in (1..7).rev() {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 右3つ fcells[] == @
                // 下 fcells[] == @
                let wbit = 0x2020600000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            if cnt == 0 {break;}
        }

        // ???  ???
        // ?@x  x@?
        // xxx  xxx
        for y in (1..7).rev() {
            let mut cnt = 0;
            for x in 1..7 {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 下3つ fcells[] == @
                // 左 fcells[] == @
                let wbit = 0x0080E00000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            for x in (1..7).rev() {
                let bit : u64 = MSB_CELL >> BitBoard::index(x, y);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    continue;
                }
                // 下3つ fcells[] == @
                // 右 fcells[] == @
                let wbit = 0x0020E00000000000 >> BitBoard::index(x - 1, y - 1);
                if cb {
                    if fcellsb & wbit != wbit {
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                } else if cw {
                    if fcellsw & wbit != wbit {
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                }
            }
            if cnt == 0 {break;}
        }
        //
        // xは@と同じ色の確定石
        // println!("fc:{:?}, {:?}", fcells, count);
        // println!("fc:{:?}", count);
        // for i in 0..8 {
        //     for j in 0..8 {
        //         print!("{},", fcells[i * 8 + j]);
        //     }
        //     println!("");
        // }
        (fcellsb.count_ones() as i8, fcellsw.count_ones() as i8)
    }
}

#[test]
fn testbitbrd() {
    let b = BitBoard::new();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 0);
    // println!("b.black:0x{:016X}", b.black);
    // println!("b.white:0x{:016X}", b.white);
    assert_eq!(b.black, 0x0000001008000000);
    assert_eq!(b.white, 0x0000000810000000);
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 0);
    assert_eq!(b.to_str(), "8/8/8/3Aa3/3aA3/8/8/8 b");
    assert_eq!(b.to_obf(),
        "---------------------------XO------OX--------------------------- X");
    let b = BitBoard::from("H/H/H/H/H/H/H/H b").unwrap();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 0);
    assert_eq!(b.black, 0xffffffffffffffff);
    assert_eq!(b.white, 0);
    assert_eq!(b.fixedstones(), (64, 0));
    assert_eq!(b.count(), 64);
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X");
    let mut b = BitBoard::from("h/h/h/h/h/h/h/h w").unwrap();
    assert_eq!(b.teban, GOTE);
    assert_eq!(b.pass, 0);
    assert_eq!(b.black, 0);
    assert_eq!(b.white, 0xffffffffffffffff);
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO O");
    b.pass();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 1);
    assert!(!b.is_passpass());
    assert!(b.is_full());
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO X");
    b.pass();
    assert_eq!(b.teban, GOTE);
    assert_eq!(b.pass, 2);
    assert!(b.is_passpass());
    assert!(b.is_full());
    assert_eq!(b.fixedstones(), (0, 64));
    assert_eq!(b.count(), -64);
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO O");
    let b = BitBoard::from("1Fa/Bf/AaAe/AbAd/AcAc/AdAb/AeAa/h w").unwrap();
    // b.put();
    assert_eq!(b.black, 0x7EC0A09088848200);
    assert_eq!(b.white, 0x013f5f6f777b7dff);
    assert!(b.checkreverse(0, 0));
    assert_eq!(b.fixedstones(), (0, 15));
    assert_eq!(b.count(),
      6 + 2 + 2 + 2 + 2 + 2 + 2 -
      (1 + 6 + 1 + 5 + 2 + 4 + 3 + 3 + 4 + 2 + 5 + 1 + 8));
    assert_eq!(b.to_obf(),
      "-XXXXXXOXXOOOOOOXOXOOOOOXOOXOOOOXOOOXOOOXOOOOXOOXOOOOOXOOOOOOOOO O");
    let b = b.r#move(1, 1);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.black, 0x0);
    assert_eq!(b.white, 0xffffffffffffffff);
    assert_eq!(b.to_str(), "h/h/h/h/h/h/h/h b");
    let b = BitBoard::from("1Fa/Bf/AaAe/AbAd/AcAc/AdAb/AeAa/h w").unwrap();
    // b.put();
    let b = b.rotate180();
    // b.put();
    assert_eq!(b.fixedstones(), (0, 15));
    let b = b.r#move(8, 8);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_str(), "h/h/h/h/h/h/h/h b");
    assert_eq!(b.fixedstones(), (0, 64));
    assert_eq!(b.count(), -64);
    let b = BitBoard::from("H/G1/F2/E3/D4/C5/B6/A7 w").unwrap();
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXX-XXXXXX--XXXXX---XXXX----XXX-----XX------X------- O");
    assert_eq!(b.fixedstones(), (36, 0));
    let b = b.rotate180();
    assert_eq!(b.fixedstones(), (36, 0));
    let b = BitBoard::from("h/1g/2f/3e/4d/5c/6b/7a w").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOOOOO-OOOOOOO--OOOOOO---OOOOO----OOOO-----OOO------OO-------O O");
    assert_eq!(b.fixedstones(), (0, 36));
    let b = b.rotate180();
    assert_eq!(b.fixedstones(), (0, 36));
    let b = BitBoard::from("H/8/8/8/8/8/8/8 b").unwrap();
    assert_eq!(b.to_obf(),
        "XXXXXXXX-------------------------------------------------------- X");
    assert_eq!(b.fixedstones(), (8, 0));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "--------------------------------------------------------XXXXXXXX X");
    assert_eq!(b.fixedstones(), (8, 0));
    let b = BitBoard::from("a7/a7/a7/a7/a7/a7/a7/a7 b").unwrap();
    assert_eq!(b.to_obf(),
        "O-------O-------O-------O-------O-------O-------O-------O------- X");
    assert_eq!(b.fixedstones(), (0, 8));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "-------O-------O-------O-------O-------O-------O-------O-------O X");
    assert_eq!(b.fixedstones(), (0, 8));
    let b = BitBoard::from("dD/dD/dD/dD/dD/dD/dD/dD b").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXX X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOO X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = BitBoard::from("h/h/h/h/H/H/H/H b").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = BitBoard::from("h/h/8/8/8/8/H/H b").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOO--------------------------------XXXXXXXXXXXXXXXX X");
    assert_eq!(b.fixedstones(), (16, 16));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXX--------------------------------OOOOOOOOOOOOOOOO X");
    assert_eq!(b.fixedstones(), (16, 16));
    let b = BitBoard::from("b4B/b4B/b4B/b4B/b4B/b4B/b4B/b4B w").unwrap();
    assert_eq!(b.to_obf(),
        "OO----XXOO----XXOO----XXOO----XXOO----XXOO----XXOO----XXOO----XX O");
    assert_eq!(b.fixedstones(), (16, 16));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XX----OOXX----OOXX----OOXX----OOXX----OOXX----OOXX----OOXX----OO O");
    assert_eq!(b.fixedstones(), (16, 16));
    // difficult to count correctly
    // let b = BitBoard::from("H/AaF/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap();
    // assert_eq!(b.fixedstones(), (34, 1));
    // let b = b.rotate180();
    // assert_eq!(b.fixedstones(), (34, 1));
    // difficult to count correctly
    // let b = BitBoard::from("H/aG/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap();
    // assert_eq!(b.fixedstones(), (31, 1));
    // let b = b.rotate180();
    // assert_eq!(b.fixedstones(), (31, 1));
}
