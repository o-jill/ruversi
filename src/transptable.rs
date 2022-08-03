// use super::*;

use std::sync::Mutex;

// static mut MLOCK : Option<Mutex<TranspositionTable>> = None;

// pub fn init_ttable() {
//     unsafe {MLOCK = Some(Mutex::new(TranspositionTable::new()))};
// }

struct TTEntry {
    pub id : [u8 ; 16],
    pub hyoka : f32,
}

impl TTEntry {
    pub fn new(i : &[u8 ; 16], h : f32) -> TTEntry {
        TTEntry {
            id : *i,
            hyoka : h,
        }
    }
}

pub struct TranspositionTable {
    list : Vec< Vec<TTEntry> >,
    nhit : usize,
    nmiss : usize,
}

const MAXSIZE : usize = 1024 * 1024;

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        let mut ret = TranspositionTable {
            list : Vec::with_capacity(256),
            nhit : 0,
            nmiss : 0,
        };
        ret.list.resize_with(256, || Vec::with_capacity(32));
        ret
    }

    pub fn clear(&mut self) {
        for l in self.list.iter_mut() {
            l.clear();
        }
        self.nhit = 0;
        self.nmiss = 0;
    }

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
                ttl.insert(t, TTEntry::new(id, hyoka));
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
                ttl.push(TTEntry::new(id, hyoka));
                self.nmiss += 1;
                hyoka
            }
        }
    }

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
        ttl.insert(idx, TTEntry::new(id, hyoka));
        // println!("tt.append size:{}, {:?}", n, id);
    }

    #[allow(dead_code)]
    pub fn dumpsz(&self) {
        for i in 0.. 256 {
            print!("{}:{},", i, self.list[i].len());
        }
    }
}

pub struct TranspositionTablev1 {
    list : Vec<TTEntry>,
}

impl TranspositionTablev1 {
    pub fn new() -> TranspositionTablev1 {
        TranspositionTablev1 {
            list : Vec::with_capacity(MAXSIZE)
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

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
                ttl.insert(t, TTEntry::new(id, hyoka));
                hyoka
            }
        }
    }

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
        ttl.insert(idx, TTEntry::new(id, hyoka));
        // println!("tt.append size:{}, {:?}", n, id);
    }
}
