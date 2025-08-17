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
    pub hit : u16,
    pub teban : i8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry::new(0, 0, 0, 0, 9999f32)
    }
}

impl TTEntry {
    pub fn new(hash : u64, black : u64, white : u64, teban : i8, hyoka : f32) -> Self {
        Self {
            hash,
            black,
            white,
            hyoka,
            hit : 0,
            teban,
        }
    }

    pub fn from(hash : u64, b : &bitboard::BitBoard, hyoka : f32) -> Self {
        Self::new(hash, b.black, b.white, b.teban, hyoka)
    }

    pub fn is_hit(&self, b : &bitboard::BitBoard) -> bool {
        self.black == b.black && self.white == b.white && self.teban == b.teban
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

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for l in self.list.iter_mut() {
            // l.hit = 0;
            l.hash = 0;
        }
    }

    pub fn check(&mut self, b : &bitboard::BitBoard) -> Option<f32> {
        let h = b.hash();
        let idx = (h & (MAXSIZE - 1) as u64) as usize;
        if self.list[idx].is_hit(b) {
        // if self.list[idx].is_hash(h, b.teban) {
            Some(self.list[idx].hyoka)
        } else {
            None
        }
    }

    pub fn append(&mut self, b : &bitboard::BitBoard, hy : f32) {
        let h = b.hash();
        let idx = (h & (MAXSIZE - 1) as u64) as usize;
        self.list[idx] = TTEntry::from(h, b, hy);
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
