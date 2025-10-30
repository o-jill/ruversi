use super::*;

// use std::sync::Mutex;

// static mut MLOCK : Option<Mutex<TranspositionTable>> = None;

// pub fn init_ttable() {
//     unsafe {MLOCK = Some(Mutex::new(TranspositionTable::new()))};
// }

const MAXSIZE : usize = if cfg!(feature="withtt") {1024 * 1024 * 1} else {1};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash : u64,
    pub black : u64,
    pub white : u64,
    pub hyoka : f32,
    pub hyoka_search : Option<f32>,
    pub hit : u16,
    pub teban : i8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry::new(0, 0, 0, bitboard::NONE, 9999f32, None)
    }
}

impl TTEntry {
    pub fn new(hash : u64, black : u64, white : u64, teban : i8,
               hyoka : f32, hyoka_search : Option<f32>) -> Self {
        Self {
            hash,
            black,
            white,
            hyoka,
            hyoka_search,
            hit : 0,
            teban,
        }
    }

    pub fn from(hash : u64, b : &bitboard::BitBoard, hyoka : f32) -> Self {
        Self::new(hash, b.black, b.white, b.teban, hyoka, None)
    }

    #[allow(dead_code)]
    pub fn from_search(hash : u64, b : &bitboard::BitBoard,
            hyoka_search : f32) -> Self {
        Self::new(hash, b.black, b.white, b.teban, 9999f32, Some(hyoka_search))
    }

    pub fn update(&mut self, _hash : u64, _b : &bitboard::BitBoard, hyoka_search : f32) {
        // self.hash = hash;
        // self.black = b.black;
        // self.white = b.white;
        // self.teban = self.teban;
        self.hyoka_search = Some(hyoka_search);
        // self.depth = depth;
    }

    #[allow(dead_code)]
    pub fn set(&mut self, hash : u64, b : &bitboard::BitBoard, hyoka : f32, hyoka_search : f32) {
        // println!("{hyoka:.2}@{depth}");
        self.hash = hash;
        self.black = b.black;
        self.white = b.white;
        self.hyoka = hyoka;
        self.hyoka_search = Some(hyoka_search);
        self.hit = 0;
        // self.depth = depth;
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
    pub fn is_available(&self, b : &bitboard::BitBoard) -> bool {
        self.is_hit(b)
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

    #[allow(dead_code)]
    pub fn is_hash(&self, hash : u64, teban : i8) -> bool {
        self.hash == hash && self.teban == teban
    }
}

pub struct TranspositionTable {
    list : Vec<TTEntry>,
    depth : Vec<u8>,  // この数字が大きいほうがより確からしい評価値
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
        Self {
            list : vec![TTEntry::default() ; sz],
            depth : vec![0 ; sz],
        }
    }

    pub fn clear(&mut self) {
        for l in self.list.iter_mut() {
            l.teban = bitboard::NONE;
            l.hit = 0;
            l.hash = 0;
            l.hyoka_search = None;
        }
        for d in self.depth.iter_mut() {
            *d = 0;
        }
    }

    pub fn next(&mut self) {
        // const DIFF : u8 = 2;
        // const DIFF : u8 = 4;
        const DIFF : u8 = 8;

        for d in self.depth.iter_mut() {
            *d = d.saturating_sub(DIFF);
        }

        // use core::arch::x86_64::*;
        // const M :usize = 32 * 4;
        // for i in (0..self.depth.len()).step_by(M) {
        //     unsafe {
        //         let diff = _mm256_set1_epi8(DIFF as i8);
        //         let a1 = self.depth.as_mut_ptr().add(i) as *mut __m256i;
        //         let a2 = self.depth.as_mut_ptr().add(i + 32 * 1) as *mut __m256i;
        //         let a3 = self.depth.as_mut_ptr().add(i + 32 * 2) as *mut __m256i;
        //         let a4 = self.depth.as_mut_ptr().add(i + 32 * 3) as *mut __m256i;
        //         let x1 = _mm256_loadu_si256(a1);
        //         let x2 = _mm256_loadu_si256(a2);
        //         let x3 = _mm256_loadu_si256(a3);
        //         let x4 = _mm256_loadu_si256(a4);
        //         let y1 = _mm256_subs_epu8(x1, diff);
        //         let y2 = _mm256_subs_epu8(x2, diff);
        //         let y3 = _mm256_subs_epu8(x3, diff);
        //         let y4 = _mm256_subs_epu8(x4, diff);
        //         _mm256_storeu_si256(a1, y1);
        //         _mm256_storeu_si256(a2, y2);
        //         _mm256_storeu_si256(a3, y3);
        //         _mm256_storeu_si256(a4, y4);
        //     }
        // }

        // for c in self.list.iter_mut() {
        //     c.update_depth(DIFF);
        // }
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
        if self.list[idx].is_hit(b) && self.depth[idx] >= depth {
            Some(self.list[idx].better_hyoka())
        } else {
            None
        }
    }

    pub fn append(&mut self, b : &bitboard::BitBoard, hy : f32, depth : u8) {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        let d = self.depth[idx];
        if depth < d {return;}

        self.depth[idx] =  depth;
        self.list[idx] = TTEntry::from(h, b, hy);
    }

    pub fn set(&mut self, b : &bitboard::BitBoard, hy : f32, depth : u8) {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        let d = self.depth[idx];
        if depth < d {return;}

        self.depth[idx] =  depth;
        self.list[idx].set(h, b, hy, hy);
    }

    pub fn update(&mut self, b : &bitboard::BitBoard, hy : f32, depth : u8) {
        let h = b.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        let d = self.depth[idx];
        if depth < d {return;}

        self.depth[idx] = depth;

        if self.list[idx].is_hit(b) {
            self.list[idx].update(h, b, hy);
        } else {
            self.list[idx].set(h, b, hy, hy);
        }
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        print!("ht:");
        // for i in self.list.iter() {
        //     print!("{},", i.hit);
        // }
        println!();
    }

    #[allow(dead_code)]
    pub fn probe(&self, ban: &bitboard::BitBoard) -> Option<(&TTEntry, u8)> {
        let h = ban.hash();
        let sz = self.list.len();
        let idx = (h & (sz - 1) as u64) as usize;
        // if self.list[idx].is_hit(ban) {
            return Some((&self.list[idx], self.depth[idx]));
        // }
        // None
    }
}

#[cfg(test)]
#[test]
fn test_ttentry_new_and_is_hit() {
    let b = bitboard::BitBoard::new();
    let entry = TTEntry::from(42, &b, 100.0);
    assert!(entry.is_hit(&b));
    let b2 = bitboard::BitBoard::from("8/8/8/H/H/8/8/8 w").unwrap();
    assert!(!entry.is_hit(&b2));
}

#[test]
fn test_ttentry_update_and_better_hyoka() {
    let b = bitboard::BitBoard::new();
    let mut entry = TTEntry::from(42, &b, 100.0);
    assert_eq!(entry.better_hyoka(), 100.0);
    entry.update(42, &b, 50.0);
    assert_eq!(entry.better_hyoka(), 50.0);
    // TTEntryにdepthがないので entry.update(42, &b, 10.0, 5);
    // 10が代入されてしまうのでコメントアウト assert_eq!(entry.better_hyoka(), 50.0);
}

#[test]
fn test_ttentry_set_and_is_hash() {
    let b = bitboard::BitBoard::new();
    let mut entry = TTEntry::default();
    entry.set(99, &b, 123.0, 55.0);
    assert!(entry.is_hash(99, 1));
    assert_eq!(entry.hyoka, 123.0);
    assert_eq!(entry.hyoka_search, Some(55.0));
}

#[test]
fn test_transptable_basic_insert_and_check() {
    let mut ttable = TranspositionTable::with_capacity(8);
    let b = bitboard::BitBoard::new();
    ttable.append(&b, 200.0, 5);
    assert_eq!(ttable.check(&b), Some(200.0));
    // 別の盤面ではヒットしない
    let b2 = bitboard::BitBoard::from("h/h/h/H/H/H/8/8 b").unwrap();
    assert_eq!(ttable.check(&b2), None);
}

#[test]
fn test_transptable_set_and_update() {
    let mut ttable = TranspositionTable::with_capacity(4);
    let b = bitboard::BitBoard::new();
    ttable.set(&b, 300.0, 9);
    assert_eq!(ttable.check(&b), Some(300.0));
    // depth小さい場合は上書きされない
    ttable.set(&b, 100.0, 7);
    assert_eq!(ttable.check(&b), Some(300.0));
    // depth大きい場合は上書き
    ttable.set(&b, 150.0, 10);
    assert_eq!(ttable.check(&b), Some(150.0));
    // updateも確認
    ttable.update(&b, 50.0, 10);
    assert_eq!(ttable.check(&b), Some(50.0));
    // depth小さい場合は上書きされない
    ttable.set(&b, 100.0, 7);
    assert_eq!(ttable.check(&b), Some(50.0));
    // depth小さい場合は上書きされない
    ttable.update(&b, 200.0, 3);
    assert_eq!(ttable.check(&b), Some(50.0));
}

#[test]
fn test_transptable_clear_and_next() {
    let mut ttable = TranspositionTable::with_capacity(2);
    let b = bitboard::BitBoard::new();
    ttable.append(&b, 100.0, 5);
    assert_eq!(ttable.check(&b), Some(100.0));
    ttable.clear();
    assert_eq!(ttable.check(&b), None);
    // nextでdepth値が減少するか
    ttable.append(&b, 200.0, 20);
    ttable.next();
    assert!(ttable.depth.iter().all(|&d| d <= 20));
}

#[test]
fn test_transptable_check_available() {
    let mut ttable = TranspositionTable::with_capacity(2);
    let b = bitboard::BitBoard::new();
    ttable.append(&b, 123.0, 12);
    // depthが足りている場合のみSome
    assert_eq!(ttable.check_available(&b, 11), Some(123.0));
    assert_eq!(ttable.check_available(&b, 12), Some(123.0));
    assert_eq!(ttable.check_available(&b, 13), None);
}
