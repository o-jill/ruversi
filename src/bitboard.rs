// use std::arch::x86_64;

pub const SENTE : i8 = 1;
pub const BLANK : i8 = 0;
pub const GOTE : i8 = -1;
pub const NONE : i8 = 127;
pub const NUMCELL : usize = 8;
pub const CELL_2D : usize = NUMCELL * NUMCELL;
const STR_SENTE : &str = "0ABCDEFGH";
pub const STR_GOTE : &str = "0abcdefgh";
const STR_NUM : &str = "012345678";
pub const STONE_SENTE : &str = "@@";
pub const STONE_GOTE : &str = "[]";
pub const LSB_CELL : u64 = 0x0000000000000001;
const LT_CELL : u64 = LSB_CELL;
const RT_CELL : u64 = 0x0100000000000000;
const LB_CELL : u64 = 0x0000000000000080;
const RB_CELL : u64 = 0x8000000000000000;
const BITPTN : [u64 ; 9] = [
    0, 0x1, 0x101, 0x10101, 0x1010101, 0x101010101, 0x10101010101,
    0x1010101010101, 0x101010101010101,
];

pub struct BitBoard {
    pub black: u64,
    pub white: u64,
    pub teban: i8,
    pub pass: i8,
}

impl BitBoard {
    pub fn new() -> BitBoard {
        BitBoard {
            black :
                (LSB_CELL << BitBoard::index(3, 3))
                | (LSB_CELL << BitBoard::index(4, 4)),
            white :
                (LSB_CELL << BitBoard::index(4, 3))
                | (LSB_CELL << BitBoard::index(3, 4)),
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
        let mut x = 0;
        let mut y = 0;
        for ch in elem[0].chars() {
            match ch {
                'A'..='H' => {
                    let n = ch as i32 + 1 - 'A' as i32;
                    ret.black |= BITPTN[n as usize] << (x * NUMCELL + y);
                    x += n as usize;
                },
                'a'..='h' => {
                    let n = ch as i32 + 1 - 'a' as i32;
                    ret.white |= BITPTN[n as usize] << (x * NUMCELL + y);
                    x += n as usize;
                },
                '1'..='8' => {
                    let n = ch as i32 - '0' as i32;
                    x += n as usize;
                },
                '/' => {
                    x = 0;
                    y += 1;
                },
                _ => {
                    return Err(format!("unknown letter rfen [{}]", ch));
                }
            }
        }

        Ok(ret)
    }

    pub fn from_obf(obf : &str) -> BitBoard {
        let elem = obf.split(" ").collect::<Vec<_>>();
        let mut ret = BitBoard {
            black : 0,
            white : 0,
            teban : SENTE,
            pass : 0,
        };
        let mut x = 0;
        let mut y = 0;
        for ch  in elem[0].chars() {
            let bit = LSB_CELL << BitBoard::index(x, y);
            match ch {
            'X' => {ret.black |= bit;},
            'O' => {ret.white |= bit;},
            // '-' => {},
            _ => {},
            }
            x += 1;
            if x >= NUMCELL {
                y += 1;
                x = 0;
            }
        }
        match elem[1] {
        "X" => {ret.teban = SENTE;},
        "O" => {ret.teban = GOTE;},
        _ => {},
        }
        ret
    }

    pub fn to_str(&self) -> String {
        let mut ban = Vec::<String>::new();
        let black = self.black;
        let white = self.white;
        for y in 0..NUMCELL {
            let mut old = NONE;
            let mut count = 0;
            let mut line = String::new();
            let mut bit : u64 = LSB_CELL << y;
            for _x in 0..NUMCELL {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                // println!("bit:0x{:016x}, cb:{}, cw:{}", bit, cb, cw);
                bit <<= NUMCELL;
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
        let black = self.black;
        let white = self.white;
        for y in 0..NUMCELL {
            let mut bit : u64 = LSB_CELL << y;
            for _x in 0..NUMCELL {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit <<= NUMCELL;
                ban += if cb {"X"} else if cw {"O"} else {"-"};
            }
        }
        ban + match self.teban {
            SENTE => " X",
            GOTE => " O",
            _ => " -",
        }
    }

    pub fn to_id(&self)-> [u8 ; 16] {
        let mut res : [u8 ; 16] = [0 ; 16];
        let mut bit : u64 = LSB_CELL;
        let black = self.black;
        let white = self.white;
        let tbn : u8 = if self.teban == SENTE { 0x00 } else { 0x80 };
        let mut idx = 0;
        for _y in 0..NUMCELL {
            let mut id : u8 = 0;
            for _x in 0..4 {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit <<= 1;
                let c = if cb {SENTE} else if cw {GOTE} else {BLANK};

                id = id * 3 + (c + 1) as u8;
            }
            res[idx] = id | tbn;
            idx += 1;

            id = 0;
            for _x in 0..4 {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit <<= 1;
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
        let black = self.black;
        let white = self.white;
        for y in 0..NUMCELL {
            let mut line = String::new();
            let mut bit : u64 = LSB_CELL << y;
            for _x in 0..NUMCELL {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                bit <<= NUMCELL;

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

    pub fn nblank(&self) -> u32 {
        (self.black | self.white).count_zeros()
    }

    fn index(x: usize, y: usize) -> usize {
        x * NUMCELL + y
    }

    pub fn at(&self, x: u8, y: u8) -> i8 {
        let bit : u64 = LSB_CELL << BitBoard::index(x as usize, y as usize);
        let cb = (bit & self.black) != 0;
        let cw = (bit & self.white) != 0;

        if cb {SENTE} else if cw {GOTE} else {BLANK}
    }

    pub fn exist(&self, x: u8, y: u8) -> bool {
        let cells = self.black | self.white;
        (cells & LSB_CELL << BitBoard::index(x as usize, y as usize)) != 0
    }

    #[allow(dead_code)]
    pub fn set(&mut self, x : u8, y : u8) {
        let bit = LSB_CELL << BitBoard::index(x as usize, y as usize);
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
        let mine = if color == SENTE {self.black} else {self.white};
        let oppo = if color == SENTE {self.white} else {self.black};

        let pos = LSB_CELL << BitBoard::index(x, y);

        let mut revall = 0;

        // 下
        let mut bit : u64 = pos << 1;
        let mut rev : u64 = 0;
        for _i in y..NUMCELL {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit <<= 1;
        }

        // 上
        let mut bit : u64 = pos >> 1;
        let mut rev : u64 = 0;
        for _i in 0..y {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit >>= 1;
        }

        // 右
        let mut bit : u64 = pos << NUMCELL;
        let mut rev : u64 = 0;
        for _i in x..NUMCELL {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit <<= NUMCELL;
        }

        // 左
        let mut bit : u64 = pos >> NUMCELL;
        let mut rev : u64 = 0;
        for _i in 0..x {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit >>= NUMCELL;
        }

        // 右下
        let mut bit : u64 = pos << (NUMCELL + 1);
        let mut rev : u64 = 0;
        let sz = if x > y {NUMCELL - 1 - x} else {NUMCELL - 1 - y};
        for _i in 0..sz {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit <<= NUMCELL + 1;
        }

        // 右上
        let mut bit : u64 = pos << (NUMCELL - 1);
        let mut rev : u64 = 0;
        let xx = NUMCELL - 1 - x;
        let yy = y;
        let sz = if xx < yy {xx} else {yy};
        for _i in 0..sz {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit <<= NUMCELL - 1;
        }

        // 左上
        let mut bit : u64 = pos >> (NUMCELL + 1);
        let mut rev : u64 = 0;
        let sz = if x < y {x} else {y};
        for _i in 0..sz {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit >>= NUMCELL + 1;
        }

        // 左下
        let mut bit : u64 = pos >> (NUMCELL - 1);
        let mut rev : u64 = 0;
        let xx = x;
        let yy = NUMCELL - 1 - y;
        let sz = if xx < yy {xx} else {yy};
        for _i in 0..sz {
            if (mine & bit) != 0 {
                revall |= rev;
                break;
            } else if (oppo & bit) != 0 {
                rev |= bit;
            } else {
                break;
            }

            bit >>= NUMCELL - 1;
        }

        if color == SENTE {
            self.black = mine | revall | pos;
            self.white = oppo ^ revall;
        } else {
            self.white = mine | revall | pos;
            self.black = oppo ^ revall;
        }
    }

    pub fn checkreverse(&self, x : usize, y : usize) -> bool {
        let color = self.teban;
        let &mut mine = &mut if color == SENTE {self.black} else {self.white};
        let &mut oppo = &mut if color == SENTE {self.white} else {self.black};
        let pos = LSB_CELL << BitBoard::index(x, y);
        // 下
        let mut bit : u64 = pos << 1;
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
            bit <<= 1;
        }

        // 上
        let mut bit : u64 = pos >> 1;
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
            bit >>= 1;
        }

        // 右
        let mut bit : u64 = pos << NUMCELL;
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
            bit <<= NUMCELL;
        }

        // 左
        let mut bit : u64 = pos >> NUMCELL;
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
            bit >>= NUMCELL;
        }

        // 右下
        let mut bit : u64 = pos << (NUMCELL + 1);
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
            bit <<= NUMCELL + 1;
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
            bit <<= NUMCELL - 1;
        }

        // 左上
        let mut bit : u64 = pos >> (NUMCELL + 1);
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
            bit >>= NUMCELL + 1;
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
            bit >>= NUMCELL - 1;
        }

        false
    }

    /**
     * @param x 0 : pass, 1 ~ 8 : column index.
     * @param y 0 : pass, 1 ~ 8 : row index.
     */
    pub fn r#move(&self, x : u8, y : u8) -> Result<BitBoard, &str> {
        if x == 0 && y == 0 {  // pass
            let mut ban = self.clone();
            ban.pass();
            return Ok(ban);
        }

        let xc = x - 1;
        let yc = y - 1;
        if self.exist(xc, yc) {
            return Err("stone exists.");
        }
        let mut ban = self.clone();
        // ban.set(xc, yc);
        ban.reverse(xc as usize, yc as usize);
        ban.flipturn();
        ban.resetpass();

        Ok(ban)
    }

    /// # Returns
    /// - None : no empty cells.
    /// - Some(vec![])  : no available cells. pass.
    /// - Some(Vec![n]) : available cells.
    pub fn genmove(&self) -> Option<Vec<(u8, u8)>> {
        let mut ret = Vec::<(u8, u8)>::new();
        let stones = self.black | self.white;
        let mut bit = LSB_CELL;
        for x in 0..NUMCELL {
            for y in 0..NUMCELL {
                let exist = bit & stones;
                bit <<= 1;
                if exist != 0 {
                    continue;
                }

                if self.checkreverse(x, y) {
                    ret.push((x as u8 + 1, y as u8 + 1));
                }
            }
        }
        if ret.is_empty() {  // pass
            return if self.is_full() {
                None
            } else {
                Some(ret)
                // Some(vec![])
                // Some(vec![(0, 0)])
            }
        }
        Some(ret)
    }

    pub fn count(&self) -> i8 {
        self.black.count_ones() as i8 - self.white.count_ones() as i8
    }

    pub fn countf32(&self) -> f32 {
        (self.black.count_ones() as i8 - self.white.count_ones() as i8) as f32
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
                bit <<= NUMCELL;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = LT_CELL << 1;
            for _i in 1..7 {  // ↓
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit <<= 1;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsw |= bit;
                bit <<= NUMCELL;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = LT_CELL << 1;
            for _i in 1..7 {  // ↓
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit <<= 1;
                count += 1;
            }
        }
        let mut bit = RT_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsb |= bit;
                bit >>= NUMCELL;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = RT_CELL << 1;
            for _i in 1..7 {  // ↓
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit <<= 1;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsw |= bit;
                bit >>= NUMCELL;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = RT_CELL << 1;
            for _i in 1..7 {  // ↓
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit <<= 1;
                count += 1;
            }
        }
        let mut bit = LB_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsb |= bit;
                bit <<= NUMCELL;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = LB_CELL >> 1;
            for _i in 1..7 {  // ↑
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit >>= 1;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // →
                fcellsw |= bit;
                bit <<= NUMCELL;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = LB_CELL >> 1;
            for _i in 1..7 {  // ↑
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit >>= 1;
                count += 1;
            }
        }
        let mut bit = RB_CELL;
        if (black & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsb |= bit;
                bit >>= NUMCELL;
                count += 1;
                if (black & bit) == 0 {
                    break;
                }
            }
            let mut bit = RB_CELL >> 1;
            for _i in 1..7 {  // ↑
                if (black & bit) == 0 {
                    break;
                }
                fcellsb |= bit;
                bit >>= 1;
                count += 1;
            }
        } else if (white & bit) != 0 {
            for _i in 0..7 {  // ←
                fcellsw |= bit;
                bit >>= NUMCELL;
                count += 1;
                if (white & bit) == 0 {
                    break;
                }
            }
            let mut bit = RB_CELL >> 1;
            for _i in 1..7 {  // ↑
                if (white & bit) == 0 {
                    break;
                }
                fcellsw |= bit;
                bit >>= 1;
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
            // 左3つ fcells[] == @
            // 上 fcells[] == @
            let mut bit = LSB_CELL << BitBoard::index(x, 1);
            let mut wbit = 0x0000000000000107 << BitBoard::index(x - 1, 0);
            for _y in 0..6 {
                // println!("bit:{:b}, wbit:{:b}", bit, wbit);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    // println!("if (cb | cw) == false");
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    // println!("if fcb | fcw");
                    bit <<= 1;
                    wbit <<= 1;
                    continue;
                }
                if cb {
                    if fcellsb & wbit != wbit {
                        // println!("fcellsb : {:b}", fcellsb);
                        break;
                    }
                    fcellsb |= bit;
                    cnt += 1;
                    // println!("fcellsb : {:b}, {}", fcellsb, cnt);
                } else if cw {
                    if fcellsw & wbit != wbit {
                        // println!("fcellsw : {:b}", fcellsw);
                        break;
                    }
                    fcellsw |= bit;
                    cnt += 1;
                } else {
                    println!("cb : {}, cw : {}", cb, cw);
                }
                bit <<= 1;
                wbit <<= 1;
            }
            // 左3つ fcells[] == @
            // 下 fcells[] == @
            let mut bit = LSB_CELL << BitBoard::index(x, NUMCELL - 2);
            let mut wbit = 0x0000000000000407 << BitBoard::index(x - 1, NUMCELL - 3);
            for _y in (0..6).rev() {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit >>= 1;
                    wbit >>= 1;
                    continue;
                }
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
                    // println!("xy: {:x}, {:x}", x, _y);
                    // println!("bit:{:x}, wbit:{:x}", bit, wbit);
                    // println!("fcellsw : {:b}, {}", fcellsw, cnt);
                }
                bit >>= 1;
                wbit >>= 1;
            }
            if cnt == 0 {break;}
        }
        // xxx  xxx
        // x@?  ?@x
        // ???  ???
        for y in 1..7 {
            let mut cnt = 0;
            // 上3つ fcells[] == @
            // 左 fcells[] == @
            let mut bit : u64 = LSB_CELL << BitBoard::index(1, y);
            let mut wbit = 0x0000000000010103 << BitBoard::index(0, y - 1);
            for _x in 1..7 {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit <<= NUMCELL;
                    wbit <<= NUMCELL;
                    continue;
                }
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
                bit <<= NUMCELL;
                wbit <<= NUMCELL;
            }
            // 上3つ fcells[] == @
            // 右 fcells[] == @
            let mut bit : u64 = LSB_CELL << BitBoard::index(NUMCELL - 2, y);
            let mut wbit = 0x0000000000030101 << BitBoard::index(NUMCELL - 3, y - 1);
            for _x in (1..7).rev() {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit >>= NUMCELL;
                    wbit >>= NUMCELL;
                    continue;
                }
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
                bit >>= NUMCELL;
                wbit >>= NUMCELL;
            }
            if cnt == 0 {break;}
        }
        //
        // ?xx  ??x
        // ?@x  ?@x
        // ??x  ?xx
        for x in (1..7).rev() {
            let mut cnt = 0;
            // 右3つ fcells[] == @
            // 上 fcells[] == @
            let mut bit : u64 = LSB_CELL << BitBoard::index(x, 1);
            let mut wbit = 0x0000000000070100 << BitBoard::index(x - 1, 0);
            for _y in 1..7 {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit <<= 1;
                    wbit <<= 1;
                    continue;
                }
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
                bit <<= 1;
                wbit <<= 1;
            }
            // 右3つ fcells[] == @
            // 下 fcells[] == @
            let mut bit : u64 = LSB_CELL << BitBoard::index(x, NUMCELL - 2);
            let mut wbit = 0x0000000000070400 << BitBoard::index(x - 1, NUMCELL - 3);
            for _y in (1..7).rev() {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit >>= 1;
                    wbit >>= 1;
                    continue;
                }
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
                bit >>= 1;
                wbit >>= 1;
            }
            if cnt == 0 {break;}
        }

        // ???  ???
        // ?@x  x@?
        // xxx  xxx
        for y in (1..7).rev() {
            let mut cnt = 0;
            // 下3つ fcells[] == @
            // 左 fcells[] == @
            let mut bit : u64 = LSB_CELL << BitBoard::index(1, y);
            let mut wbit = 0x0000000000040406 << BitBoard::index(0, y - 1);
            for _x in 1..7 {
                // println!("bit:{:08x}, wbit:{:08x}", bit, wbit);
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit <<= NUMCELL;
                    wbit <<= NUMCELL;
                    continue;
                }
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
                    // println!("fcellsw : {:08x}, {}", fcellsw, cnt);
                }
                bit <<= NUMCELL;
                wbit <<= NUMCELL;
            }
            // 下3つ fcells[] == @
            // 右 fcells[] == @
            let mut bit : u64 = LSB_CELL << BitBoard::index(NUMCELL - 2, y);
            let mut wbit = 0x0000000000060404 << BitBoard::index(NUMCELL - 3, y - 1);
            for _x in (1..7).rev() {
                let cb = (bit & black) != 0;
                let cw = (bit & white) != 0;
                if (cb | cw) == false {
                    break;
                }
                let fcb = (bit & fcellsb) != 0;
                let fcw = (bit & fcellsw) != 0;
                if fcb | fcw {
                    bit >>= NUMCELL;
                    wbit >>= NUMCELL;
                    continue;
                }
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
                bit >>= NUMCELL;
                wbit >>= NUMCELL;
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
        // println!("fc:{:b}, {:b}", fcellsb, fcellsw);
        (fcellsb.count_ones() as i8, fcellsw.count_ones() as i8)
    }
}

/// count # of empty cells
/// 
/// # Argument
/// - `rfen` : rfen text.
/// 
/// # Returns
/// - Ok(# of empty cells) if succeeded.
/// - Err(msg) if some error happend.
pub fn count_emptycells(rfen : &str) -> Result<i8, String> {
    let mut count = 0;

    for ch in rfen.chars() {
        match ch {
            'A'..='H' => {
                // let n = ch  as i8 + 1 - 'A'  as i8;
                // count += n;
            },
            'a'..='h' => {
                // let n = ch  as i8 + 1 - 'a'  as i8;
                // count += n;
            },
            '1'..='8' => {
                let n = ch as i8 - '0' as i8;
                count += n;
            },
            '/' => {},
            ' ' => {
                return Ok(count);
            },
            _ => {
                return Err(format!("unknown letter rfen [{}]", ch));
            }
        }
    }
    Err(format!("invalid format [{rfen}]"))
}

/// count # of stones
/// 
/// # Argument
/// - `rfen` : rfen text.
/// 
/// # Returns
/// - Ok(# of stones) if succeeded.
/// - Err(msg) if some error happend.
#[allow(dead_code)]
pub fn count_stones(rfen : &str) -> Result<i8, String> {
    match count_emptycells(rfen) {
        Ok(n) => {Ok(64 - n)},
        Err(m) => {Err(m)}
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
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(5, 3), (6, 4), (3, 5), (4, 6)]));
    let b = BitBoard::from("H/H/H/H/H/H/H/H b").unwrap();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 0);
    assert_eq!(b.black, 0xffffffffffffffff);
    assert_eq!(b.white, 0);
    assert_eq!(b.fixedstones(), (64, 0));
    assert_eq!(b.count(), 64);
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X");
    assert!(b.genmove().is_none());
    let mut b = BitBoard::from("h/h/h/h/h/h/h/h w").unwrap();
    assert_eq!(b.teban, GOTE);
    assert_eq!(b.pass, 0);
    assert_eq!(b.black, 0);
    assert_eq!(b.white, 0xffffffffffffffff);
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO O");
    assert!(b.genmove().is_none());
    b.pass();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 1);
    assert!(!b.is_passpass());
    assert!(b.is_full());
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO X");
    assert!(b.genmove().is_none());
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
    assert_eq!(b.black, 0x004121110905037E);
    assert_eq!(b.white, 0xffbedeeef6fafc80);
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
    assert_eq!(b.black, (0x004121110905037E as u64).reverse_bits());
    assert_eq!(b.white, (0xffbedeeef6fafc80 as u64).reverse_bits());
    // b.put();
    assert_eq!(b.fixedstones(), (0, 15));
    let b = b.r#move(8, 8);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_str(), "h/h/h/h/h/h/h/h b");
    assert_eq!(b.fixedstones(), (0, 64));
    assert_eq!(b.count(), -64);
    let b = BitBoard::from("Af1/Fb/EaAa/DaBa/CaCa/BaDa/AaEa/H b").unwrap();
    b.put();
    println!("b.black:0x{:016X}", b.black);
    println!("b.white:0x{:016X}", b.white);
    assert_eq!(b.black, 0x80FCFAF6EEDEBEFF);
    assert_eq!(b.white, 0x7E03050911214100);
    assert!(b.checkreverse(7, 0));
    assert_eq!(b.fixedstones(), (15, 0));
    assert_eq!(b.count(),
      -(6 + 2 + 2 + 2 + 2 + 2 + 2) +
      (1 + 6 + 1 + 5 + 2 + 4 + 3 + 3 + 4 + 2 + 5 + 1 + 8));
    assert_eq!(b.to_obf(),
      "XOOOOOO-XXXXXXOOXXXXXOXOXXXXOXXOXXXOXXXOXXOXXXXOXOXXXXXOXXXXXXXX X");
    let b = b.r#move(8, 1);
    assert!(b.is_ok());
    let b = b.unwrap();
    b.put();
    assert_eq!(b.black, 0xffffffffffffffff);
    assert_eq!(b.white, 0x0);
    assert_eq!(b.to_str(), "H/H/H/H/H/H/H/H w");
    let b = BitBoard::from("Af1/Fb/EaAa/DaBa/CaCa/BaDa/AaEa/H b").unwrap();
    let b = b.rotate180();
    // b.put();
    assert_eq!(b.black, (0x80FCFAF6EEDEBEFF as u64).reverse_bits());
    assert_eq!(b.white, (0x7E03050911214100 as u64).reverse_bits());
    assert!(b.checkreverse(0, 7));
    assert_eq!(b.fixedstones(), (15, 0));
    assert_eq!(b.count(),
      -(6 + 2 + 2 + 2 + 2 + 2 + 2) +
      (1 + 6 + 1 + 5 + 2 + 4 + 3 + 3 + 4 + 2 + 5 + 1 + 8));
    assert_eq!(b.to_obf(),
      "XXXXXXXXOXXXXXOXOXXXXOXXOXXXOXXXOXXOXXXXOXOXXXXXOOXXXXXX-OOOOOOX X");
    let b = b.r#move(1, 8);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.black, 0xffffffffffffffff);
    assert_eq!(b.white, 0x0);
    assert_eq!(b.to_str(), "H/H/H/H/H/H/H/H w");
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
    let b = BitBoard::from("1A6/A1eA/1a6/1a6/1a6/1a6/1a6/1A6 b").unwrap();
    assert_eq!(b.to_obf(),
        "-X------X-OOOOOX-O-------O-------O-------O-------O-------X------ X");
    assert!(b.checkreverse(1, 1));
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 4 - 10);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(2, 2), (4, 3), (3, 4)]));
    let b = b.r#move(2, 2);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "-X------XXXXXXXX-X-------X-------X-------X-------X-------X------ O");
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 15);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("1A6/A1eA/1a6/1a6/1a6/1a6/1a6/1A6 b").unwrap();
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "------X-------O-------O-------O-------O-------O-XOOOOO-X------X- X");
    assert!(b.checkreverse(6, 6));
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 4 - 10);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(6, 5), (5, 6), (7, 7)]));
    let b = b.r#move(7, 7);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "------X-------X-------X-------X-------X-------X-XXXXXXXX------X- O");
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 15);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("2A5/2a5/Aa1dA/2a5/2a5/2a5/2a5/2A5 b").unwrap();
    assert_eq!(b.to_obf(),
        "--X-------O-----XO-OOOOX--O-------O-------O-------O-------X----- X");
    assert!(b.checkreverse(2, 2));
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 4 - 10);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(3, 3)]));
    let b = b.r#move(3, 3);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "--X-------X-----XXXXXXXX--X-------X-------X-------X-------X----- O");
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 15);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("2A5/2a5/Aa1dA/2a5/2a5/2a5/2a5/2A5 b").unwrap();
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "-----X-------O-------O-------O-------O--XOOOO-OX-----O-------X-- X");
    assert!(b.checkreverse(5, 5));
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 4 - 10);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(6, 6)]));
    let b = b.r#move(6, 6);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "-----X-------X-------X-------X-------X--XXXXXXXX-----X-------X-- O");
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 15);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("B6/A1eA/1b5/1a1a4/1a2a3/1a3a2/1a4a1/1A5A b").unwrap();
    assert_eq!(b.to_obf(),
        "XX------X-OOOOOX-OO------O-O-----O--O----O---O---O----O--X-----X X");
    assert!(b.checkreverse(1, 1));
    assert_eq!(b.fixedstones(), (4, 0));
    assert_eq!(b.count(), 6 - 15);
    b.put();
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(2, 2), (4, 3), (3, 4)]));
    let b = b.r#move(2, 2);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "XX------XXXXXXXX-XX------X-X-----X--X----X---X---X----X--X-----X O");
    assert_eq!(b.fixedstones(), (4, 0));
    assert_eq!(b.count(), 22);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("B6/A1eA/1b5/1a1a4/1a2a3/1a3a2/1a4a1/1A5A b").unwrap();
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "X-----X--O----O---O---O----O--O-----O-O------OO-XOOOOO-X------XX X");
    assert!(b.checkreverse(6, 6));
    assert_eq!(b.fixedstones(), (4, 0));
    assert_eq!(b.count(), 6 - 15);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(6, 5), (5, 6), (7, 7)]));
    let b = b.r#move(7, 7);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "X-----X--X----X---X---X----X--X-----X-X------XX-XXXXXXXX------XX O");
    assert_eq!(b.fixedstones(), (4, 0));
    assert_eq!(b.count(), 22);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b").unwrap();
    assert_eq!(b.to_obf(),
        "X-X-X----OOO----XO-OOOOX-OOO----X-O-O-----O--O----O---O---X----X X");
    assert!(b.checkreverse(2, 2));
    assert_eq!(b.fixedstones(), (2, 0));
    assert_eq!(b.count(), 8 - 17);
    let mv = b.genmove();
    // b.put();
    assert_eq!(mv, Some(vec![(3, 3), (6, 4), (4, 6)]));
    let b = b.r#move(3, 3);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "X-X-X----XXX----XXXXXXXX-XXX----X-X-X-----X--X----X---X---X----X O");
    assert_eq!(b.fixedstones(), (2, 0));
    assert_eq!(b.count(), 26);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    let b = BitBoard::from("A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b").unwrap();
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "X----X---O---O----O--O-----O-O-X----OOO-XOOOO-OX----OOO----X-X-X X");
    assert!(b.checkreverse(5, 5));
    assert_eq!(b.fixedstones(), (2, 0));
    assert_eq!(b.count(), 8 - 17);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![(5, 3), (3, 5), (6, 6)]));
    let b = b.r#move(6, 6);
    assert!(b.is_ok());
    let b = b.unwrap();
    assert_eq!(b.to_obf(),
        "X----X---X---X----X--X-----X-X-X----XXX-XXXXXXXX----XXX----X-X-X O");
    assert_eq!(b.fixedstones(), (2, 0));
    assert_eq!(b.count(), 26);
    let mv = b.genmove();
    assert_eq!(mv, Some(vec![]));
    // difficult to count correctly
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
    assert_eq!(count_emptycells("8/8/8/3Aa3/3aA3/8/8/8 b").unwrap(), 60);
    assert_eq!(count_stones("8/8/8/3Aa3/3aA3/8/8/8 b").unwrap(), 4);
    assert_eq!(count_emptycells("H/aG/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap(), 25);
    assert_eq!(count_stones("H/aG/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap(), 39);
    assert_eq!(count_emptycells("H/AaF/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap(), 25);
    assert_eq!(count_stones("H/AaF/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap(), 39);
    let revchktbl = [
        ("-OOOOOOX-------------------------------------------------------- X", 0, 0),
        ("-OOOOOX--------------------------------------------------------- X", 0, 0),
        ("-OOOOX---------------------------------------------------------- X", 0, 0),
        ("-OOOX----------------------------------------------------------- X", 0, 0),
        ("-OOX------------------------------------------------------------ X", 0, 0),
        ("-OX------------------------------------------------------------- X", 0, 0),
        ("--------O-------O-------O-------O-------O-------O-------X------- X", 0, 0),
        ("--------O-------O-------O-------O-------O-------X--------------- X", 0, 0),
        ("--------O-------O-------O-------O-------X----------------------- X", 0, 0),
        ("--------O-------O-------O-------X------------------------------- X", 0, 0),
        ("--------O-------O-------X--------------------------------------- X", 0, 0),
        ("--------O-------X----------------------------------------------- X", 0, 0),
        ("---------O--------O--------O--------O--------O--------O--------X X", 0, 0),
        ("---------O--------O--------O--------O--------O--------X--------X X", 0, 0),
        ("---------O--------O--------O--------O--------X--------O--------X X", 0, 0),
        ("---------O--------O--------O--------X--------O--------O--------X X", 0, 0),
        ("---------O--------O--------X--------O--------O--------O--------X X", 0, 0),
        ("---------O--------X--------O--------O--------O--------O--------X X", 0, 0),
        ("XOOOOOO--------------------------------------------------------- X", 7, 0),
        ("-XOOOOO--------------------------------------------------------- X", 7, 0),
        ("--XOOOO--------------------------------------------------------- X", 7, 0),
        ("---XOOO--------------------------------------------------------- X", 7, 0),
        ("----XOO--------------------------------------------------------- X", 7, 0),
        ("-----XO--------------------------------------------------------- X", 7, 0),
        ("X-------O-------O-------O-------O-------O-------O--------------- X", 0, 7),
        ("X-------X-------O-------O-------O-------O-------O--------------- X", 0, 7),
        ("X-------O-------X-------O-------O-------O-------O--------------- X", 0, 7),
        ("X-------O-------O-------X-------O-------O-------O--------------- X", 0, 7),
        ("X-------O-------O-------O-------X-------O-------O--------------- X", 0, 7),
        ("X-------O-------O-------O-------O-------X-------O--------------- X", 0, 7),
        ("X--------O--------O--------O--------O--------O--------O--------- X", 7, 7),
        ("-------X------O------O------O------O------O------O-------------- X", 0, 7),
        ("-------X------X------O------O------O------O------O-------------- X", 0, 7),
        ("-------X------O------X------O------O------O------O-------------- X", 0, 7),
        ("-------X------O------O------X------O------O------O-------------- X", 0, 7),
        ("-------X------O------O------O------X------O------O-------------- X", 0, 7),
        ("-------X------O------O------O------O------X------O-------------- X", 0, 7),
        ("--------------O------O------O------O------O------O------X------- X", 7, 0),
        ("--------------O------O------O------O------O------X------X------- X", 7, 0),
        ("--------------O------O------O------O------X------O------X------- X", 7, 0),
        ("--------------O------O------O------X------O------O------X------- X", 7, 0),
        ("--------------O------O------X------O------O------O------X------- X", 7, 0),
        ("--------------O------X------O------O------O------O------X------- X", 7, 0),
        ("-----------OOO---OOO-OO----OOO----O-O-O--O--O-------O-------X--- X", 4, 2),
        ("-----------OOO---OOO-OO----OOO----O-O-O--O--O---X---O----------- X", 4, 2),
        ("-----------OOO--XOOO-OO----OOO----O-O-O--O--O-------O----------- X", 4, 2),
        ("--X--------OOO---OOO-OO----OOO----O-O-O--O--O-------O----------- X", 4, 2),
        ("----X------OOO---OOO-OO----OOO----O-O-O--O--O-------O----------- X", 4, 2),
        ("------X----OOO---OOO-OO----OOO----O-O-O--O--O-------O----------- X", 4, 2),
        ("-----------OOO---OOO-OOX---OOO----O-O-O--O--O-------O----------- X", 4, 2),
        ("-----------OOO---OOO-OO----OOO----O-O-O--O--O--X----O----------- X", 4, 2),
        ("X-------OO-------OOOOOO-OO------O-O-----O--O----O---O----------- X", 0, 2),
        ("--X-----OO-------OOOOOO-OO------O-O-----O--O----O---O----------- X", 0, 2),
        ("--------OO-------OOOOOOXOO------O-O-----O--O----O---O----------- X", 0, 2),
        ("--------OO-------OOOOOO-OO------O-O-----O--O----O---O--------X-- X", 0, 2),
        ("--------OO-------OOOOOO-OO------O-O-----O--O----O---O---X------- X", 0, 2),
        ("--X------XOX-----X-X-----XXX------------------------------------ X", 2, 2),
        ("X--------OOO-----O-O-----OOO------------------------------------ X", 2, 2),
        ("---------OOO----XO-O-----OOO------------------------------------ X", 2, 2),
        ("---------OOO-----O-O-----OOO----X------------------------------- X", 2, 2),
        ("---------OOO-----O-O-----OOO--------X--------------------------- X", 2, 2),
        ("---------OOO-----O-OX----OOO------------------------------------ X", 2, 2),
        ("----X----OOO-----O-O-----OOO------------------------------------ X", 2, 2),
        ("--X------OOO-----O-OO----OOO------O-O--------------------------- X", 2, 2),
        ("X--------OOO-----O-OO----OOO------O-O--------------------------- X", 2, 2),
        ("---------OOO----XO-OO----OOO------O-O--------------------------- X", 2, 2),
        ("---------OOO-----O-OO----OOO----X-O-O--------------------------- X", 2, 2),
        ("---------OOO-----O-OO----OOO------O-O-----X--------------------- X", 2, 2),
        ("---------OOO-----O-OO----OOO------O-O--------X------------------ X", 2, 2),
        ("---------OOO-----O-OOX---OOO------O-O--------------------------- X", 2, 2),
        ("----X----OOO-----O-OO----OOO------O-O--------------------------- X", 2, 2),

        ("--X------OOO-----O-OOO---OOO------O-O-----O--O------------------ X", 2, 2),
        ("X--------OOO-----O-OOO---OOO------O-O-----O--O------------------ X", 2, 2),
        ("---------OOO----XO-OOO---OOO------O-O-----O--O------------------ X", 2, 2),
        ("---------OOO-----O-OOO---OOO----X-O-O-----O--O------------------ X", 2, 2),
        ("---------OOO-----O-OOO---OOO------O-O-----O--O----X------------- X", 2, 2),
        ("---------OOO-----O-OOO---OOO------O-O-----O--O--------X--------- X", 2, 2),
        ("---------OOO-----O-OOOX--OOO------O-O-----O--O------------------ X", 2, 2),
        ("----X----OOO-----O-OOO---OOO------O-O-----O--O------------------ X", 2, 2),

        ("--X------OOO-----O-OOOO--OOO------O-O-----O--O----O---O--------- X", 2, 2),
        ("X--------OOO-----O-OOOO--OOO------O-O-----O--O----O---O--------- X", 2, 2),
        ("---------OOO----XO-OOOO--OOO------O-O-----O--O----O---O--------- X", 2, 2),
        ("---------OOO-----O-OOOO--OOO----X-O-O-----O--O----O---O--------- X", 2, 2),
        ("---------OOO-----O-OOOO--OOO------O-O-----O--O----O---O---X----- X", 2, 2),
        ("---------OOO-----O-OOOO--OOO------O-O-----O--O----O---O--------X X", 2, 2),
        ("---------OOO-----O-OOOOX-OOO------O-O-----O--O----O---O--------- X", 2, 2),
        ("----X----OOO-----O-OOOO--OOO------O-O-----O--O----O---O--------- X", 2, 2),
    ];
    for (obf, x, y) in revchktbl {
        println!("obf:{obf}");
        let b = BitBoard::from_obf(obf);
        assert!(b.checkreverse1(x, y));
        assert!(b.checkreverse2(x, y));
        assert!(b.checkreverse(x, y));
        assert!(b.checkreverse4(x, y));
        let b = b.rotate180();
        assert!(b.checkreverse1(NUMCELL - 1 - x, NUMCELL - 1 - y));
        assert!(b.checkreverse2(NUMCELL - 1 - x, NUMCELL - 1 - y));
        assert!(b.checkreverse(NUMCELL - 1 - x, NUMCELL - 1 - y));
        assert!(b.checkreverse4(NUMCELL - 1 - x, NUMCELL - 1 - y));
    }
    for y in 1..NUMCELL - 1 {
        for x in 1..NUMCELL - 1 {
            let mut b = BitBoard::from_obf(
                "XXXXXXXXXOOOOOOXXOOOOOOXXOOOOOOXXOOOOOOXXOOOOOOXXOOOOOOXXXXXXXXX X");
            let bit = LSB_CELL << BitBoard::index(x as usize, y as usize);
            let mask = !bit;
            b.white &= mask;
            assert!(b.checkreverse1(x, y));
            assert!(b.checkreverse2(x, y));
            assert!(b.checkreverse(x, y));
            assert!(b.checkreverse4(x, y));
        }
    }
    let revchktbl = [
        ("---------------------------------------------------------------- X", 2, 2),
        ("XXXXXXXXXXXXXXXXXX-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X", 2, 2),
        ("OOOOOOOOOOOOOOOOOO-OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO X", 2, 2),
        ("XXXXXXXXXXXXXXXXXX-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X", 2, 2),
        ("---------OOOOOO--O-OOOO--OOOOOO--OOOOOO--OOOOOO--OOOOOO--------- X", 2, 2),
        ("---------OOO-----O-O-----OOO------------------------------------ X", 2, 2),
        ("---------OOO-----O-OO----OOO------O-O--------------------------- X", 2, 2),
        ("---------OOO-----O-OOO---OOO------O-O-----O--O------------------ X", 2, 2),
        ("---------XXX-----X-X-----XXX------------------------------------ X", 2, 2),
        ("---------XXX-----X-XX----XXX------X-X--------------------------- X", 2, 2),
        ("---------XXX-----X-XXX---XXX------X-X-----X--X------------------ X", 2, 2),
        ("-X-------------------------------------------------------------- X", 0, 0),
        ("------X--------------------------------------------------------- X", 7, 0),
        ("--------X------------------------------------------------------- X", 0, 0),
        ("---------X--------O--------O--------O--------O--------O--------X X", 0, 0),
        ("X-------O-------O-------O-------O-------O-------X--------------- X", 0, 7),
        ("-------X------O------O------O------O------O------X-------------- X", 0, 7),
        ("--------------X------O------O------O------O------O------X------- X", 7, 0),
    ];
    for (obf, x, y) in revchktbl {
        let b = BitBoard::from_obf(obf);
        assert!(!b.checkreverse1(x, y));
        assert!(!b.checkreverse2(x, y));
        assert!(!b.checkreverse(x, y));
        assert!(!b.checkreverse4(x, y));
        let b = b.rotate180();
        assert!(!b.checkreverse1(NUMCELL - 1 - x, NUMCELL - 1 - y));
        assert!(!b.checkreverse2(NUMCELL - 1 - x, NUMCELL - 1 - y));
        assert!(!b.checkreverse(NUMCELL - 1 - x, NUMCELL - 1 - y));
        assert!(!b.checkreverse4(NUMCELL - 1 - x, NUMCELL - 1 - y));
    }
    for y in 0..NUMCELL {
        for x in 0..NUMCELL {
            let b = BitBoard::from_obf(
                "---------------------------------------------------------------- X");
            assert!(!b.checkreverse1(x, y));
            assert!(!b.checkreverse2(x, y));
            assert!(!b.checkreverse(x, y));
            assert!(!b.checkreverse4(x, y));
            let mut b = BitBoard::from_obf(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X");
            let bit = LSB_CELL << BitBoard::index(x as usize, y as usize);
            let mask = !bit;
            b.black &= mask;
            assert!(!b.checkreverse1(x, y));
            assert!(!b.checkreverse2(x, y));
            assert!(!b.checkreverse(x, y));
            assert!(!b.checkreverse4(x, y));

            let mut b = BitBoard::from_obf(
                "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO X");
            b.white &= mask;
            assert!(!b.checkreverse1(x, y));
            assert!(!b.checkreverse2(x, y));
            assert!(!b.checkreverse(x, y));
            assert!(!b.checkreverse4(x, y));
        }
    }
}
