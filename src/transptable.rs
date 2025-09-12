use super::*;

// use std::sync::Mutex;

// static mut MLOCK : Option<Mutex<TranspositionTable>> = None;

// pub fn init_ttable() {
//     unsafe {MLOCK = Some(Mutex::new(TranspositionTable::new()))};
// }

const MAXSIZE : usize = if cfg!(feature="withtt") {1024 * 1024 * 1} else {1};

#[repr(C)]
#[derive(Clone, Copy)]
struct TTEntry {
    pub hash : u64,
    pub black : u64,
    pub white : u64,
    pub hyoka : f32,
    pub hyoka_search : Option<f32>,
    pub hit : u16,
    pub depth : u8,  // この数字が大きいほうがより確からしい評価値
    pub teban : i8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry::new(0, 0, 0, bitboard::NONE, 9999f32, None, 0)
    }
}

impl TTEntry {
    pub fn new(hash : u64, black : u64, white : u64, teban : i8,
               hyoka : f32, hyoka_search : Option<f32>, depth : u8) -> Self {
        Self {
            hash,
            black,
            white,
            hyoka,
            hyoka_search,
            hit : 0,
            depth,
            teban,
        }
    }

    pub fn from(hash : u64, b : &bitboard::BitBoard,
            hyoka : f32, depth : u8) -> Self {
        Self::new(hash, b.black, b.white, b.teban, hyoka, None, depth)
    }

    #[allow(dead_code)]
    pub fn from_search(hash : u64, b : &bitboard::BitBoard,
            hyoka_search : f32, depth : u8) -> Self {
        Self::new(hash, b.black, b.white, b.teban, 9999f32, Some(hyoka_search), depth)
    }

    pub fn update(&mut self, hash : u64, b : &bitboard::BitBoard, hyoka_search : f32, depth : u8) {
        if self.depth > depth {return;}

        // self.hash = hash;
        // self.black = b.black;
        // self.white = b.white;
        // self.teban = self.teban;
        self.hyoka_search = Some(hyoka_search);
        self.depth = depth;
    }

    pub fn set(&mut self, hash : u64, b : &bitboard::BitBoard, hyoka : f32, hyoka_search : f32, depth : u8) {
        self.hash = hash;
        self.black = b.black;
        self.white = b.white;
        self.hyoka = hyoka;
        self.hyoka_search = Some(hyoka_search);
        self.hit = 0;
        self.depth = depth;
        self.teban = b.teban;
    }

    pub fn is_hit(&self, b : &bitboard::BitBoard) -> bool {
        // self.black == b.black && self.white == b.white && self.teban == b.teban
        let ret = self.black == b.black && self.white == b.white && self.teban == b.teban;
        // if self.hash != 0 && !ret {
        //     println!("crashed!!{:16x}", self.hash);
        // }
        ret
    }
    /**
     * hit 且つ 引数のdepth以上なら使って良い
     * true:hyokaを使って良い, false:hyokaの値はあっても信用ならないので使わない
     */
    pub fn is_available(&self, b : &bitboard::BitBoard, depth : u8) -> bool {
        self.is_hit(b) && depth <= self.depth
    }

    /**
     * 探索した評価があれば探索した評価を返す。
     * なければ保存されている静止探索の評価を返す。
     */
    pub fn better_hyoka(&self) -> f32 {
        if let Some(hyoka) = self.hyoka_search {
            hyoka
        } else {
            self.hyoka
        }
    }

    /**
     * 保存されているデータのdepthを更新する
     * ちょっと前の局面から探索した結果なので価値を下げる。
     */
    pub fn update_depth(&mut self, diff : u8) {
        self.depth =
            if self.depth > diff {
                self.depth - diff
            } else {
                0
            }
    }

    #[allow(dead_code)]
    pub fn is_hash(&self, hash : u64, teban : i8) -> bool {
        self.hash == hash && self.teban == teban
    }
}

pub struct TranspositionTable {
    list : Vec<TTEntry>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        TranspositionTable::new()
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self::with_capacity(MAXSIZE)
    }

    pub fn with_capacity(sz : usize) -> Self {
        Self { list: vec![TTEntry::default() ; sz] }
    }

    pub fn clear(&mut self) {
        for l in self.list.iter_mut() {
            l.teban = bitboard::NONE;
            l.hit = 0;
            l.hash = 0;
            l.hyoka_search = None;
        }
    }

    pub fn check(&self, b : &bitboard::BitBoard) -> Option<f32> {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        if self.list[idx].is_hit(b) {
            Some(self.list[idx].better_hyoka())
        } else {
            None
        }
    }

    /**
     * 末端まで降りなくても置換表の値を信じて使ってよいか
     */
    pub fn check_available(&self, b : &bitboard::BitBoard, depth : u8) -> Option<f32> {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        if self.list[idx].is_available(b, depth) {
        // if self.list[idx].is_hit(b) {
        // if self.list[idx].is_hash(h, b.teban) {
            Some(self.list[idx].better_hyoka())
        } else {
            None
        }
    }

    pub fn append(&mut self, b : &bitboard::BitBoard, hy : f32, depth : u8) {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        self.list[idx] = TTEntry::from(h, b, hy, depth);
    }

    pub fn set(&mut self, b : &bitboard::BitBoard, hy : f32, depth : u8) {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        self.list[idx].set(h, b, hy, hy, depth);
    }

    pub fn update(&mut self, b : &bitboard::BitBoard, hy : f32, depth : u8) {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        self.list[idx].update(h, b, hy, depth);
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        print!("ht:");
        // for i in self.list.iter() {
        //     print!("{},", i.hit);
        // }
        println!();
    }
}
