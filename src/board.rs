// use std::arch::x86_64;

pub const SENTE : i8 = 1;
pub const BLANK : i8 = 0;
pub const GOTE : i8 = -1;
pub const NONE : i8 = 127;
pub const NUMCELL : usize = 8;
pub const CELL_2D : usize = NUMCELL * NUMCELL;
pub const STR_SENTE : &str = "0ABCDEFGH";
pub const STR_GOTE : &str = "0abcdefgh";
pub const STR_NUM : &str = "012345678";
pub const STONE_SENTE : &str = "@@";
pub const STONE_GOTE : &str = "[]";

pub trait Board/*<T>*/ {
    // fn from(rfen : &str) -> Result<Board, String> ;
    fn to_str(&self) -> String ;
    fn to_obf(&self) -> String ;
    fn put(&self) ;
    fn flipturn(&mut self) ;
    fn resetpass(&mut self) ;

    fn pass(&mut self) ;

    fn is_passpass(&self) -> bool ;

    // fn clone(&self) -> Self ;

    fn teban(&self) -> i8 ;

    fn nblank(&self) -> u32 ;
    fn count(&self) -> i8 ;

    /* fn index(x: usize, y: usize) -> usize {
        x + y * NUMCELL
    } */
    /* fn at(&self, x: usize, y: usize) -> i8 {
        self.cells[x + y * NUMCELL]
    } */

    /* fn set(&mut self, x : usize, y : usize) {
        self.cells[Board::index(x, y)] = self.teban;
    } */

    fn reverse(&mut self, x : usize, y : usize) ;
    fn checkreverse(&self, x : usize, y : usize) -> bool ;
    // fn r#move(&self, x : u8, y : u8) -> Result<T, &str> ;
    fn genmove(&self) -> Option<Vec<(u8, u8)>> ;

    fn is_full(&self) -> bool ;

    // fn rotate180(&self) -> Self ;

    fn fixedstones(&self) -> (i8, i8) ;
}

/* *
pub struct Board {
    pub cells: [i8 ; CELL_2D],
    pub teban: i8,
    pub pass: i8,
}

impl Board {
    pub fn new() -> Board {
        let mut ret = Board {
            cells : [BLANK; CELL_2D],
            teban : SENTE,
            pass : 0,
        };
        ret.cells[Board::index(3, 3)] = SENTE;
        ret.cells[Board::index(4, 4)] = SENTE;
        ret.cells[Board::index(3, 4)] = GOTE;
        ret.cells[Board::index(4, 3)] = GOTE;
        ret
    }

    pub fn from(rfen : &str) -> Result<Board, String> {}

    pub fn fromarray(cells : [i8 ; CELL_2D], tbn : i8) -> Board {
        Board { cells: cells, teban: tbn, pass: 0 }
    }

    pub fn init() -> Board {
        Board::from("8/8/8/3Aa3/3aA3/8/8/8 b").unwrap()
    }

    pub fn to_str(&self) -> String {
    }

    // othello board file format
    // init:
    // ---------------------------XO------OX--------------------------- X
    //
    pub fn to_obf(&self) -> String {
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

    pub fn put(&self) {    }
    pub fn flipturn(&mut self) {    }

    pub fn resetpass(&mut self) {    }

    pub fn pass(&mut self) {    }

    pub fn is_passpass(&self) -> bool {    }
    pub fn clone(&self) -> Board {
        Board {
            cells: self.cells, teban: self.teban , pass: self.pass
        }
    }

    pub fn nblank(&self) -> u32 {
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
    fn reverse(&mut self, x : usize, y : usize) {    }

    pub fn checkreverse(&self, x : usize, y : usize) -> bool {    }

    pub fn r#move(&self, x : u8, y : u8) -> Result<Board, &str> {    }

    pub fn genmove(&self) -> Option<Vec<(u8, u8)>> {
    }

    pub fn count(&self) -> i8 {
    }

    pub fn is_full(&self) -> bool {
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

    pub fn fixedstones(&self) -> (i8, i8) {    }
}
* */