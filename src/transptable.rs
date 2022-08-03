// use super::*;

// use std::sync::Mutex;

// static mut MLOCK : Option<Mutex<TranspositionTable>> = None;

// pub fn init_ttable() {
//     unsafe {MLOCK = Some(Mutex::new(TranspositionTable::new()))};
// }

#[derive(Clone)]
struct TTEntry {
    pub id : [u8 ; 16],
    pub hyoka : f32,
    pub hit : u32,  // ~ 4G leaves
}

impl TTEntry {
    pub fn new(i : &[u8 ; 16], hy : f32, hi : u32) -> TTEntry {
        TTEntry {
            id : *i,
            hyoka : hy,
            hit : hi,
        }
    }
}

pub struct TranspositionTable {
    list : Vec<TTEntry>,
    sz : usize,
    nhit : usize,
    nmiss : usize,
}

const MAXSIZE : usize = 1024 * 1024;
const ENTRYSIZE : usize = 1024 * 16;

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            list : vec![TTEntry::new(&[0 ; 16], -9999.9, 0); ENTRYSIZE],
            sz : 0,
            nhit : 0,
            nmiss : 0,
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for l in self.list.iter_mut() {
            l.hit = 0;
        }
        self.sz = 0;
        self.nhit = 0;
        self.nmiss = 0;
    }

    pub fn check_or_append<F>(&mut self, id : &[u8 ; 16], mut f : F) -> f32 where
            F : FnMut() -> f32 {
        let ttl = &mut self.list;
        for i in 0..self.sz {
            if ttl[i].id.cmp(id).is_eq() {
                self.nhit += 1;
                ttl[i].hit += 1;
                let ret = ttl[i].hyoka;
                for j in (0 .. i).rev() {
                    if ttl[j].hit < ttl[j + 1].hit {
                        ttl.swap(j, j + 1);
                    } else {
                        break;
                    }
                }
                return ret;
            }
        }
        let hyoka = f();
        self.nmiss += 1;
        if self.sz < ENTRYSIZE {
            self.sz += 1;
        }
        let idx = self.sz - 1;
        ttl[idx].id = *id;
        ttl[idx].hyoka = hyoka;
        ttl[idx].hit = 1;
        // ttl[self.sz - 1] = TTEntry::new(id, hyoka, 1);
        // self.dumpsz();
        hyoka
    }

    #[allow(dead_code)]
    pub fn dumpsz(&self) {
        for i in 0 .. 100 {
        // for i in 0 .. ENTRYSIZE {
                print!("{},", self.list[i].hit);
        }
        println!("hit,{},miss,{}", self.nhit, self.nmiss);
    }
}

struct TTEntryv1 {
    pub id : [u8 ; 16],
    pub hyoka : f32,
}

impl TTEntryv1 {
    pub fn new(i : &[u8 ; 16], h : f32) -> TTEntryv1 {
        TTEntryv1 {
            id : *i,
            hyoka : h,
        }
    }
}

pub struct TranspositionTablev2 {
    list : Vec< Vec<TTEntryv1> >,
    nhit : usize,
    nmiss : usize,
}

#[allow(dead_code)]
impl TranspositionTablev2 {
    pub fn new() -> TranspositionTablev2 {
        let mut ret = TranspositionTablev2 {
            list : Vec::with_capacity(256),
            nhit : 0,
            nmiss : 0,
        };
        ret.list.resize_with(256, || Vec::with_capacity(32));
        ret
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for l in self.list.iter_mut() {
            l.clear();
        }
        self.nhit = 0;
        self.nmiss = 0;
    }

    #[allow(dead_code)]
    pub fn check(&self, id : &[u8 ; 16]) -> Option<f32> {
        let lid = id[4] as usize;
        let ttl = &self.list[lid];
        let item = ttl.binary_search_by(|a| {
            a.id.cmp(id)
        });
        match item {
            Ok(t) => {
                Some(ttl[t].hyoka)
            },
            Err(_) => {None}
        }
    }

    pub fn check_or_append<F>(&mut self, id : &[u8 ; 16], mut f : F) -> f32 where
            F : FnMut() -> f32 {
        let lid = id[4] as usize;
        let ttl = &mut self.list[lid];
        let item = ttl.binary_search_by(|a| {
            a.id.cmp(id)
        });
        match item {
            Ok(t) => {
                // self.hit();
                self.nhit += 1;
                ttl[t].hyoka
            },
            Err(t) => {
                let hyoka = f();
                let n = ttl.len();
                if n > MAXSIZE {
                    // ttl.remove(0);
                    ttl.pop();
                }
                ttl.insert(t, TTEntryv1::new(id, hyoka));
                self.nmiss += 1;
                hyoka
            }
        }
    }

    #[allow(dead_code)]
    // twice slower than check_or_append()
    pub fn check_or_append_poor<F>(&mut self, id : &[u8 ; 16], mut f : F) -> f32 where 
            F : FnMut() -> f32 {
        let lid = id[4] as usize;
        let ttl = &mut self.list[lid];
        let item = ttl.iter().position(|a| a.id.cmp(id).is_eq() );
        match item {
            Some(t) => {
                self.nhit += 1;
                ttl[t].hyoka
            },
            None => {
                let hyoka = f();
                let n = ttl.len();
                if n > MAXSIZE {
                    ttl.remove(0);
                    // ttl.pop();
                }
                ttl.push(TTEntryv1::new(id, hyoka));
                self.nmiss += 1;
                hyoka
            }
        }
    }

    #[allow(dead_code)]
    pub fn append(&mut self, id : &[u8 ; 16], hyoka : f32) {
        let lid = id[4] as usize;
        let ttl = &mut self.list[lid];
        let n = ttl.len();
        if n > MAXSIZE {
            // ttl.remove(0);
            ttl.pop();
        }

        let item = ttl.binary_search_by(|a| {
            a.id.cmp(id)
        });
        let idx = match item {
            Ok(t) => t,
            Err(_) => ttl.len(),
        };
        ttl.insert(idx, TTEntryv1::new(id, hyoka));
        // println!("tt.append size:{}, {:?}", n, id);
    }

    #[allow(dead_code)]
    pub fn dumpsz(&self) {
        // for i in 0.. 256 {
        //     print!("{}:{},", i, self.list[i].len());
        // }
        println!("hit,{},miss,{}", self.nhit, self.nmiss);
    }
}

#[allow(dead_code)]
pub struct TranspositionTablev1 {
    list : Vec<TTEntryv1>,
}

impl TranspositionTablev1 {
    #[allow(dead_code)]
    pub fn new() -> TranspositionTablev1 {
        TranspositionTablev1 {
            list : Vec::with_capacity(MAXSIZE)
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.list.clear();
    }

    #[allow(dead_code)]
    pub fn check(&self, id : &[u8 ; 16]) -> Option<f32> {
        let ttl = &self.list;
        let item = ttl.binary_search_by(|a| {
            a.id.cmp(id)
        });
        match item {
            Ok(t) => {
                Some(ttl[t].hyoka)
            },
            Err(_) => {None}
        }
    }

    #[allow(dead_code)]
    pub fn check_or_append<F>(&mut self, id : &[u8 ; 16], mut f : F) -> f32 where
            F : FnMut() -> f32 {
        let ttl = &mut self.list;
        let item = ttl.binary_search_by(|a| {
            a.id.cmp(id)
        });
        match item {
            Ok(t) => {
                ttl[t].hyoka
            },
            Err(t) => {
                let hyoka = f();
                let n = ttl.len();
                if n > MAXSIZE {
                    // ttl.remove(0);
                    ttl.pop();
                }
                ttl.insert(t, TTEntryv1::new(id, hyoka));
                hyoka
            }
        }
    }

    #[allow(dead_code)]
    pub fn append(&mut self, id : &[u8 ; 16], hyoka : f32) {
        let ttl = &mut self.list;

        let n = ttl.len();
        if n > MAXSIZE {
            // ttl.remove(0);
            ttl.pop();
        }

        let item = ttl.binary_search_by(|a| {
            a.id.cmp(id)
        });
        let idx = match item {
            Ok(t) => t,
            Err(_) => ttl.len(),
        };
        ttl.insert(idx, TTEntryv1::new(id, hyoka));
        // println!("tt.append size:{}, {:?}", n, id);
    }
}
