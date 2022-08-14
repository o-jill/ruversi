use std::arch::x86_64;

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

pub struct Board {
    pub cells: Vec<i8>,
    pub teban: i8,
    pub pass: i8,
}

impl Board {
    pub fn new() -> Board {
        let mut ret = Board {
            cells : Vec::new(),
            teban : SENTE,
            pass : 0,
        };
        ret.cells.resize(8 * 8, BLANK);
        ret.cells[Board::index(3, 3)] = SENTE;
        ret.cells[Board::index(4, 4)] = SENTE;
        ret.cells[Board::index(3, 4)] = GOTE;
        ret.cells[Board::index(4, 3)] = GOTE;
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
            pass : 0,
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
                let c = self.at(x, y);
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

    // othello board file format
    // init:
    // ---------------------------XO------OX--------------------------- X
    //
    pub fn to_obf(&self) -> String {
        let ban = self.cells.iter().map(|&c| {
            match c {
                SENTE => "X".to_string(),
                GOTE => "O".to_string(),
                _ => "-".to_string(),
            }
        }).collect::<Vec<String>>().join("");
        ban + match self.teban {
            SENTE => " X",
            GOTE => " O",
            _ => " -",
        }
    }

    pub fn to_id(&self)-> [u8 ; 16] {
        let mut res : [u8 ; 16] = [0 ; 16];
        let tbn : u8 = if self.teban == SENTE { 0x00 } else { 0x80 };
        for i in 0..CELL_2D / 4 {
            let c = &self.cells[i * 4..];
            let mut id : u8 = 0;
            for j in 0..4 {
                id = id * 3 + (c[j] + 1) as u8;
            }
            res[i] = id | tbn;
        }
        res
    }

    pub fn to_id_simd(&self)-> [u8 ; 16] {
        let mut res : [u8 ; 16] = [0 ; 16];
        let tbn : i8 = if self.teban == SENTE { 0x00 } else { -128 };
        unsafe {
            let mut sum16 = x86_64::_mm_setzero_si128();
            for i in 0..CELL_2D / 16 {
                let ci816 = x86_64::_mm_load_si128(
                    self.cells[i * 16..].as_ptr() as *const x86_64::__m128i);
                // -1 ~ +1 -> 0 ~ 2
                let one16 = x86_64::_mm_set1_epi8(1);
                let cu816 = x86_64::_mm_add_epi8(ci816, one16);

                let three8 = x86_64::_mm_set1_epi16(3);
                sum16 = x86_64::_mm_mullo_epi16(three8, sum16);
                sum16 = x86_64::_mm_add_epi16(sum16, cu816);
            }
            let tbn16 = x86_64::_mm_set1_epi8(tbn);
            let sum16 = x86_64::_mm_or_si128(tbn16, sum16);
            x86_64::_mm_store_si128(
                res.as_mut_ptr() as *mut x86_64::__m128i, sum16);
        }
        res
    }

    pub fn put(&self) {
        for y in 0..NUMCELL {
            let mut line = String::new();
            for x in 0..NUMCELL {
                let c = self.at(x, y);
                line += "|";
                line +=
                    if c == SENTE {
                        STONE_SENTE
                    } else if c == GOTE {
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
    pub fn clone(&self) -> Board {
        Board {
            cells: self.cells.to_vec(), teban: self.teban , pass: self.pass
        }
    }

    fn index(x: usize, y: usize) -> usize {
        x + y * NUMCELL
    }

    pub fn at(&self, x: usize, y: usize) -> i8 {
        self.cells[x + y * NUMCELL]
    }

    pub fn set(&mut self, x : usize, y : usize) {
        self.cells[Board::index(x, y)] = self.teban;
    }

    /**
     * x : 0~7
     * y : 0~7
     */
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
            if val == BLANK {
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
        if self.at(xc, yc) != BLANK {
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
                if c != BLANK {
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
        let mut sum : i8 = 0;
        for c in self.cells.iter() {
            sum += *c;
        }
        sum
    }

    pub fn is_full(&self) -> bool {
        for c in self.cells.iter() {
            if *c == BLANK {
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
            let x = i % 8;
            let y = i / 8;
            *c = cells[8 - 1 - x + 8 * (8 - 1 - y)];
        }
        b
    }

    pub fn fixedstones(&self) -> (i8, i8) {
        let mut count = [0 ; 3];
        let mut fcells = [0;CELL_2D];
    
        // 四隅と辺
        let c = self.at(0, 0);
        if c != BLANK {
            fcells[0] = c;
            count[(c + 1) as usize] += 1;
            for i in 1..7 {  // →
                if self.at(i, 0) != c {
                    break;
                }
                fcells[i] = c;
                count[(c + 1) as usize] += 1;
            }
            for i in 1..7 {  // ↓
                if self.at(0, i) != c {
                    break;
                }
                fcells[i * 8] = c;
                count[(c + 1) as usize] += 1;
            }
        }
        let c = self.at(7, 0);
        if c != BLANK {
            fcells[7] = c;
            count[(c + 1) as usize] += 1;
            for i in (1..7).rev() {  // ←
                if self.at(i, 0) != c {
                    break;
                }
                if fcells[i] != BLANK {
                    continue;
                }
                fcells[i] = c;
                count[(c + 1) as usize] += 1;
            }
            for i in 1..7 {  // ↓
                if self.at(7, i) != c {
                    break;
                }
                if fcells[7 + i * 8] != BLANK {
                    continue;
                }
                fcells[7 + i * 8] = c;
                count[(c + 1) as usize] += 1;
            }
        }
        let c = self.at(0, 7);
        if c != BLANK {
            fcells[7 * 8] = c;
            count[(c + 1) as usize] += 1;
            for i in 1..7 {  // →
                if self.at(i, 7) != c {
                    break;
                }
                if fcells[i + 7 * 8] != BLANK {
                    continue;
                }
                fcells[i + 7 * 8] = c;
                count[(c + 1) as usize] += 1;
            }
            for i in (1..7).rev() {  // ↑
                if self.at(0, i) != c {
                    break;
                }
                if fcells[i * 8] != BLANK {
                    continue;
                }
                fcells[i * 8] = c;
                count[(c + 1) as usize] += 1;
            }
        }
        let c = self.at(7, 7);
        if c != BLANK {
            fcells[7 + 7 * 8] = c;
            count[(c + 1) as usize] += 1;
            for i in (1..7).rev() {  // ↑
                if self.at(7, i) != c {
                    break;
                }
                if fcells[7 + i * 8] != BLANK {
                    continue;
                }
                fcells[7 + i * 8] = c;
                count[(c + 1) as usize] += 1;
            }
            for i in (1..7).rev() {  // ←
                if self.at(i, 7) != c {
                    break;
                }
                if fcells[i + 7 * 8] != BLANK {
                    continue;
                }
                fcells[i + 7 * 8] = c;
                count[(c + 1) as usize] += 1;
            }
        }
        if count[0] < 4 && count[2] < 4 {
            // println!("fc:{:?}", fcells);
            return (count[2], count[0]);
        }
        // 中身
        // こんな感じなら確定石
        // xx?  x??
        // x@?  x@?
        // x??  xx?
        for x in 1..7 {
            let mut cnt = 0;
            let xh = x - 1;
            for y in 1..7 {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 左3つ fcells[] == @
                let fc = &fcells[xh..];
                if fc[(y - 1) * 8] != c || fc[y * 8] != c || fc[(y + 1) * 8] != c {
                    break;
                }
                // 上 fcells[] == @
                if fcells[x + y * 8 - 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
                    cnt += 1;
                }
            }
            for y in (1..7).rev() {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 左3つ fcells[] == @
                let fc = &fcells[xh..];
                if fc[(y - 1) * 8] != c || fc[y * 8] != c || fc[(y + 1) * 8] != c {
                    break;
                }
                // 下 fcells[] == @
                if fcells[x + y * 8 + 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
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
            let yu = y - 1;
            for x in 1..7 {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 上3つ fcells[] == @
                let fc = &fcells[yu * 8..];
                if fc[x - 1] != c || fc[x] != c || fc[x + 1] != c {
                    break;
                }
                // 左 fcells[] == @
                if fcells[x - 1 + y * 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
                    cnt += 1;
                }
            }
            for x in (1..7).rev() {
                let c = self.at(x, y);
                if c == BLANK {
                    continue;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 上3つ fcells[] == @
                let fc = &fcells[yu * 8..];
                if fc[x - 1] != c || fc[x] != c || fc[x + 1] != c {
                    continue;
                }
                // 右 fcells[] == @
                if fcells[x + 1 + y * 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
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
            let xm = x + 1;
            for y in 1..7 {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 右3つ fcells[] == @
                let fc = &fcells[xm..];
                if fc[(y - 1) * 8] != c || fc[y * 8] != c || fc[(y + 1) * 8] != c {
                    break;
                }
                // 上 fcells[] == @
                if fcells[x + y * 8 - 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
                    cnt += 1;
                }
            }
            for y in (1..7).rev() {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 右3つ fcells[] == @
                let fc = &fcells[xm..];
                if fc[(y - 1) * 8] != c || fc[y * 8] != c || fc[(y + 1) * 8] != c {
                    break;
                }
                // 下 fcells[] == @
                if fcells[x + y * 8 + 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
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
            let ys = y + 1;
            for x in 1..7 {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 下3つ fcells[] == @
                let fc = &fcells[ys * 8..];
                if fc[x - 1] != c || fc[x] != c || fc[x + 1] != c {
                    break;
                }
                // 左 fcells[] == @
                if fcells[x - 1 + y * 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
                    cnt += 1;
                }
            }
            for x in (1..7).rev() {
                let c = self.at(x, y);
                if c == BLANK {
                    break;
                }
                if fcells[x + y * 8] != BLANK {
                    continue;
                }
                // 下3つ fcells[] == @
                let fc = &fcells[ys * 8..];
                if fc[x - 1] != c || fc[x] != c || fc[x + 1] != c {
                    break;
                }
                // 右 fcells[] == @
                if fcells[x + 1 + y * 8] == c {
                    fcells[x + y * 8] = c;
                    count[(c + 1) as usize] += 1;
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
        (count[2], count[0])
    }
}

#[test]
fn testbrd() {
    let b = Board::new();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 0);
    for (i, c) in b.cells.iter().zip(
        [
            BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,SENTE,GOTE,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,GOTE,SENTE,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,
            BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,BLANK,].iter()) {
        assert_eq!(*i, *c);
    }
    assert_eq!(b.fixedstones(), (0, 0));
    assert_eq!(b.count(), 0);
    assert_eq!(b.to_str(), "8/8/8/3Aa3/3aA3/8/8/8 b");
    assert_eq!(b.to_obf(),
        "---------------------------XO------OX--------------------------- X");
    let b = Board::from("H/H/H/H/H/H/H/H b").unwrap();
    assert_eq!(b.teban, SENTE);
    assert_eq!(b.pass, 0);
    for i in b.cells.iter() {
        assert_eq!(*i, SENTE);
    }
    assert_eq!(b.fixedstones(), (64, 0));
    assert_eq!(b.count(), 64);
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X");
    let mut b = Board::from("h/h/h/h/h/h/h/h w").unwrap();
    assert_eq!(b.teban, GOTE);
    assert_eq!(b.pass, 0);
    for i in b.cells.iter() {
        assert_eq!(*i, GOTE);
    }
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
    let b = Board::from("1Fa/Bf/AaAe/AbAd/AcAc/AdAb/AeAa/h w").unwrap();
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
    assert_eq!(b.to_str(), "h/h/h/h/h/h/h/h b");
    let b = Board::from("1Fa/Bf/AaAe/AbAd/AcAc/AdAb/AeAa/h w").unwrap();
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
    let b = Board::from("H/G1/F2/E3/D4/C5/B6/A7 w").unwrap();
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXX-XXXXXX--XXXXX---XXXX----XXX-----XX------X------- O");
    assert_eq!(b.fixedstones(), (36, 0));
    let b = b.rotate180();
    assert_eq!(b.fixedstones(), (36, 0));
    let b = Board::from("h/1g/2f/3e/4d/5c/6b/7a w").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOOOOO-OOOOOOO--OOOOOO---OOOOO----OOOO-----OOO------OO-------O O");
    assert_eq!(b.fixedstones(), (0, 36));
    let b = b.rotate180();
    assert_eq!(b.fixedstones(), (0, 36));
    let b = Board::from("H/8/8/8/8/8/8/8 b").unwrap();
    assert_eq!(b.to_obf(),
        "XXXXXXXX-------------------------------------------------------- X");
    assert_eq!(b.fixedstones(), (8, 0));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "--------------------------------------------------------XXXXXXXX X");
    assert_eq!(b.fixedstones(), (8, 0));
    let b = Board::from("a7/a7/a7/a7/a7/a7/a7/a7 b").unwrap();
    assert_eq!(b.to_obf(),
        "O-------O-------O-------O-------O-------O-------O-------O------- X");
    assert_eq!(b.fixedstones(), (0, 8));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "-------O-------O-------O-------O-------O-------O-------O-------O X");
    assert_eq!(b.fixedstones(), (0, 8));
    let b = Board::from("dD/dD/dD/dD/dD/dD/dD/dD b").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXX X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOOXXXXOOOO X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = Board::from("h/h/h/h/H/H/H/H b").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO X");
    assert_eq!(b.fixedstones(), (32, 32));
    let b = Board::from("h/h/8/8/8/8/H/H b").unwrap();
    assert_eq!(b.to_obf(),
        "OOOOOOOOOOOOOOOO--------------------------------XXXXXXXXXXXXXXXX X");
    assert_eq!(b.fixedstones(), (16, 16));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XXXXXXXXXXXXXXXX--------------------------------OOOOOOOOOOOOOOOO X");
    assert_eq!(b.fixedstones(), (16, 16));
    let b = Board::from("b4B/b4B/b4B/b4B/b4B/b4B/b4B/b4B w").unwrap();
    assert_eq!(b.to_obf(),
        "OO----XXOO----XXOO----XXOO----XXOO----XXOO----XXOO----XXOO----XX O");
    assert_eq!(b.fixedstones(), (16, 16));
    let b = b.rotate180();
    assert_eq!(b.to_obf(),
        "XX----OOXX----OOXX----OOXX----OOXX----OOXX----OOXX----OOXX----OO O");
    assert_eq!(b.fixedstones(), (16, 16));
    // difficult to count correctly
    // let b = Board::from("H/AaF/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap();
    // assert_eq!(b.fixedstones(), (34, 1));
    // let b = b.rotate180();
    // assert_eq!(b.fixedstones(), (34, 1));
    // difficult to count correctly
    // let b = Board::from("H/aG/C5/D4/C1A3/C2A2/C3A1/C4A b").unwrap();
    // assert_eq!(b.fixedstones(), (31, 1));
    // let b = b.rotate180();
    // assert_eq!(b.fixedstones(), (31, 1));
}
