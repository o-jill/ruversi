use super::*;
use std::sync::{Arc, RwLock};

const SORT_PRI : [i32 ; 64]= [
    0, 3, 1, 2, 2, 1, 3, 0,
    3, 3, 4, 4, 4, 4, 3, 3,
    1, 4, 5, 5, 5, 5, 4, 1,
    2, 4, 5, 5, 5, 5, 4, 2,
    2, 4, 5, 5, 5, 5, 4, 2,
    1, 4, 5, 5, 5, 5, 4, 1,
    3, 3, 4, 4, 4, 4, 3, 3,
    0, 3, 1, 2, 2, 1, 3, 0,
];

// fn move_priority(&(x, y) : &(u8, u8)) -> i32 {
//     let idx = if x == 0 || y == 0 {0} else {x + y * 8 - 9};
//     SORT_PRI[idx as usize]
// }

#[allow(dead_code)]
fn move_priority2(&(x1, y1, x2, y2) : &(u8, u8, u8, u8)) -> i32 {
    let idx1 = if x1 == 0 || y1 == 0 {0} else {x1 + y1 * 8 - 9};
    let idx2 = if x2 == 0 || y2 == 0 {0} else {x2 + y2 * 8 - 9};
    SORT_PRI[idx1 as usize] * 10 + SORT_PRI[idx2 as usize]
}

// fn move_priority3(&(x1, y1, x2, y2, x3, y3)
//         : &(u8, u8, u8, u8, u8, u8)) -> i32 {
//     let idx1 = if x1 == 0 || y1 == 0 {0} else {x1 + y1 * 8 - 9};
//     let idx2 = if x2 == 0 || y2 == 0 {0} else {x2 + y2 * 8 - 9};
//     let idx3 = if x3 == 0 || y3 == 0 {0} else {x3 + y3 * 8 - 9};
//     SORT_PRI[idx1 as usize] * 100 + SORT_PRI[idx2 as usize] * 10
//         + SORT_PRI[idx3 as usize]
// }

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
// static mut WEIGHT : Option<Vec<f32>> = None;
#[allow(dead_code)]
pub static mut WEIGHT : &Option<weight::Weight> = unsafe {&nodebb::WEIGHT};

#[derive(Clone)]
pub struct Best {
    pub hyoka : f32,
    pub x : u8,
    pub y : u8,
}

impl Best {
    #[allow(dead_code)]
    pub fn new(hyoka : f32, x : u8, y : u8) -> Best {
        Best { hyoka, x, y }
    }

    pub fn pos(&self) -> String {
        format!("{}{}",
            board::STR_GOTE.chars().nth(self.x as usize).unwrap(), self.y)
    }

    #[allow(dead_code)]
    pub fn to_str(&self) -> String {
        format!("h:{} {}", self.hyoka, self.pos())
    }
}

pub type ShNodeResult = (f32, Arc<RwLock<ShNode>>);

pub struct ShNode {
    child : Vec<Arc<RwLock<ShNode>>>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    pub best : Option<Best>,
    pub x : u8,
    pub y : u8,
    depth : u8,
    pub teban : i8,
}

impl ShNode {
    #[allow(dead_code)]
    pub fn new(x : u8, y : u8, depth : u8, teban : i8) -> ShNode {
        ShNode {
            child : Vec::<Arc<RwLock<ShNode>>>::new(),
            hyoka : None,
            kyokumen : 0,
            best : None,
            x,
            y,
            depth,
            teban,
        }
    }

    #[allow(dead_code)]
    #[cfg(target_arch="x86_64")]
    fn evaluate(ban : &bitboard::BitBoard) -> f32 {
        // unsafe{ return WEIGHT.as_ref().unwrap().evaluatev3bb(ban)}
        unsafe {
            if cfg!(feature="nosimd") {
                WEIGHT.as_ref().unwrap().evaluatev7bb(ban)
            } else if cfg!(feature="avx") {
                WEIGHT.as_ref().unwrap().evaluatev7bb_simdavx(ban)
            } else {
                WEIGHT.as_ref().unwrap().evaluatev7bb_simd(ban)
            }
        }
    }

    #[cfg(target_arch="aarch64")]
    fn evaluate(ban : &bitboard::BitBoard) -> f32 {
        unsafe {WEIGHT.as_ref().unwrap().evaluatev7bb(ban)}
    }

    #[allow(dead_code)]
    fn evalwtt(ban : &bitboard::BitBoard, tt : &mut transptable::TranspositionTable) -> f32 {
        let id = if cfg!(feature="nosimd") {ban.to_id()} else {ban.to_id_simd()};
        tt.check_or_append(&id, || ShNode::evaluate(ban))
    }

    #[allow(dead_code)]
    pub fn think(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<ShNodeResult> {
// println!("shnode::think(ban, d:{depth})");
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let mut moves = ban.genmove()?;

        let node = Arc::new(RwLock::new(ShNode::new(0, 0, depth, bitboard::NONE)));
        // println!("{}", node.lock().unwrap().dump());
        if moves.is_empty() {  // pass
            moves.push((0, 0));
            node.write().unwrap().depth += 1;
            depth += 1;
        }
        // let teban = ban.teban;
        // let ddd = node.lock().unwrap().depth;
        let n = moves.len();
        let teban = ban.teban;
        let mut leaves = Vec::<Arc<RwLock<ShNode>>>::new();
        for (mvx, mvy) in moves {
            let n = Arc::new(RwLock::new(ShNode::new(mvx, mvy, depth - 1, teban)));
            node.write().unwrap().child.push(n.clone());
            leaves.push(n);
        }
        let mut leaves1 = Vec::from_iter(leaves[0..n/2].iter().cloned());
        let mut leaves2 = Vec::from_iter(leaves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let sub =
                thread::spawn(move || {
            for leaf in leaves1.iter_mut() {
                let x;
                let y;
                {
                    let lf = leaf.read().unwrap();
                    x = lf.x;
                    y = lf.y;
                }
                let newban = ban2.r#move(x, y).unwrap();
                let val = ShNode::think_internal(leaf, &newban);
                leaf.write().unwrap().hyoka = val;
            }
        });

        let teban = ban.teban;
        for leaf in leaves2.iter_mut() {
            let x;
            let y;
            {
                let lf = leaf.read().unwrap();
                x = lf.x;
                y = lf.y;
            }
            let newban = ban.r#move(x, y).unwrap();
            let val = ShNode::think_internal(leaf, &newban);
            leaf.write().unwrap().hyoka = val;
        }
        sub.join().unwrap();
        // tt.dumpsz();

        let fteban = teban as f32;
        let hyoka;
        {
            let nd = &mut node.write().unwrap();
            let mut be : Option<Best> = None;
            let mut km = 0;
            for leaf in nd.child.iter() {
                let lf = leaf.read().unwrap();
                km += lf.kyokumen;

                let lb = lf.best.as_ref();
                if be.is_none() {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y));
                } else if lb.is_none() {
                    // nothing to do.
                } else if be.as_ref().unwrap().hyoka * fteban < lb.as_ref().unwrap().hyoka * fteban {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y));
                }
            }
            hyoka = be.as_ref().unwrap().hyoka;
            nd.hyoka = Some(hyoka);
            nd.best = be;
            nd.kyokumen = km;
        }
        // println!("done.");
        Some((hyoka, node.clone()))
    }

    pub fn think_internal(node:&Arc<RwLock<ShNode>>, ban : &bitboard::BitBoard)
            -> Option<f32> {
        let mut nod = node.write().unwrap();
        let mut depth = nod.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            nod.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            nod.kyokumen = 1;
            return Some(ShNode::evaluate(ban));
        }

        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            nod.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.is_empty() {  // pass
            moves.push((0, 0));
            depth += 1;
        }

        let mut hyoka = -9999999.0;
        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = nod.child.len();
            let leaf = Arc::new(RwLock::new(ShNode::new(mvx, mvy, depth - 1, teban)));
            nod.child.push(leaf.clone());
            let val = ShNode::think_internal(
                &nod.child[idx], &newban);

            {
                let mut lf = leaf.write().unwrap();
                lf.hyoka = val;
                nod.kyokumen += lf.kyokumen;
            }

            let best = nod.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if nod.best.is_none() {
                nod.best = Some(Best::new(val, mvx, mvy));
                hyoka = val;
                // println!("n{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                // println!("b{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
                nod.best = Some(Best::new(val, mvx, mvy));
                hyoka = val;
            } else {
                // println!("b{depth}{}{} != b{mvx}{mvy}", nod.x, nod.y);
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                nod.child[idx].write().unwrap().release();
            }
        }
        Some(hyoka)
    }

    #[allow(dead_code)]
    pub fn think_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, Arc<RwLock<ShNode>>)> {
        let node = Arc::new(RwLock::new(ShNode::new(0, 0, depth, bitboard::NONE)));
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let mut moves = ban.genmove()?;

        // let mut tt = transptable::TranspositionTable::new();
        if moves.is_empty() {  // pass
            moves.push((0, 0));
            depth += 1;
            node.write().unwrap().depth += 1;
        }
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }
        let n = moves.len();
        let teban = ban.teban;
        let mut leaves = Vec::<Arc<RwLock<ShNode>>>::new();
        for (mvx, mvy) in moves {
            let n = Arc::new(RwLock::new(
                    ShNode::new(mvx, mvy, depth - 1, teban)));
            node.write().unwrap().child.push(n.clone());
            leaves.push(n);
        }
        let mut leaves1 = Vec::from_iter(leaves[0..n/2].iter().cloned());
        let mut leaves2 = Vec::from_iter(leaves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let salpha = Arc::new(std::sync::Mutex::new(-100000.0f32));
        let sbeta = Arc::new(std::sync::Mutex::new(100000.0f32));
        let sal = salpha.clone();
        let sbe = sbeta.clone();

        let sub =
                thread::spawn(move || {
            leaves1.sort_by(|a, b| {
                let aa = a.read().unwrap();
                let bb = b.read().unwrap();
                let ia = aa.x + aa.y * 8 - 9;
                let ib = bb.x + bb.y * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
            //let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut alpha : f32 = *sal.lock().unwrap();
            let mut beta : f32 = *sbe.lock().unwrap();
            for leaf in leaves1.iter_mut() {
                    let x;
                let y;
                {
                    let lf = leaf.read().unwrap();
                    x = lf.x;
                    y = lf.y;
                }
                let newban = ban2.r#move(x, y).unwrap();
                let val = ShNode::think_internal_ab(leaf, &newban, alpha, beta);
                leaf.write().unwrap().hyoka = val;
                let val = val.unwrap();
                if teban == bitboard::SENTE {
                    let mut sa = sal.lock().unwrap();
                    if val > *sa {
                        *sa = val;
                        alpha = val;
                    } else {
                        alpha = *sa;
                    }
                } else if teban == bitboard::GOTE {
                    let mut sb = sbe.lock().unwrap();
                    if val < *sb {
                        *sb = val;
                        beta = val;
                    } else {
                        beta = *sb;
                    }
                }
            }
        });

        leaves2.sort_by(|a, b| {
            let aa = a.read().unwrap();
            let bb = b.read().unwrap();
            let ia = aa.x + aa.y * 8 - 9;
            let ib = bb.x + bb.y * 8 - 9;
            let pa = SORT_PRI[ia as usize];
            let pb = SORT_PRI[ib as usize];
            pa.partial_cmp(&pb).unwrap()
        });
        //let mut tt = transptable::TranspositionTable::new();
        let teban = ban.teban;
        let mut alpha : f32 = *salpha.lock().unwrap();
        let mut beta : f32 = *sbeta.lock().unwrap();
    // let mut alpha : f32 = -100000.0;
        // let mut beta : f32 = 100000.0;
        for leaf in leaves2.iter_mut() {
            let x;
            let y;
            {
                let lf = leaf.read().unwrap();
                x = lf.x;
                y = lf.y;
            }
            let newban = ban.r#move(x, y).unwrap();
            let val = ShNode::think_internal_ab(leaf, &newban, alpha, beta);
            leaf.write().unwrap().hyoka = val;
            let val = val.unwrap();
            if teban == bitboard::SENTE {
                let mut sa = salpha.lock().unwrap();
                if val > *sa {
                    *sa = val;
                    alpha = val;
                } else {
                    alpha = *sa;
                }
            } else if teban == bitboard::GOTE {
                let mut sb = sbeta.lock().unwrap();
                if val < *sb {
                    *sb = val;
                    beta = val;
                } else {
                    beta = *sb;
                }
            }
        }

        sub.join().unwrap();

        let fteban = teban as f32;
        let hyoka;
        {
            let nd = &mut node.write().unwrap();
            let mut be : Option<Best> = None;
            let mut km = 0;
            for leaf in nd.child.iter() {
                let lf = leaf.read().unwrap();
                km += lf.kyokumen;

                let lb = lf.best.as_ref();
                if be.is_none() {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y));
                } else if lb.is_none() {
                    // nothing to do.
                } else if be.as_ref().unwrap().hyoka * fteban < lb.as_ref().unwrap().hyoka * fteban {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y));
                }
            }
            hyoka = be.as_ref().unwrap().hyoka;
            nd.hyoka = Some(hyoka);
            nd.best = be;
            nd.kyokumen = km;
        }
        // println!("done.");
        Some((hyoka, node.clone()))
    }

    #[allow(dead_code)]
    pub fn think_ab_extract2(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, Arc<RwLock<ShNode>>)> {
        let node = Arc::new(RwLock::new(
                ShNode::new(0, 0, depth, bitboard::NONE)));
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;

        // no more empty cells
        let mut moves = ban.genmove()?;

        // let mut tt = transptable::TranspositionTable::new();
        if moves.is_empty() {  // pass
            moves.push((0, 0));
            depth += 1;
            node.write().unwrap().depth += 1;
        }
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }
        let n = moves.len();
        let teban = ban.teban;
        let mut leaves = Vec::<Arc<RwLock<ShNode>>>::new();
        for (mvx, mvy) in moves {
            let n = Arc::new(RwLock::new(
                    ShNode::new(mvx, mvy, depth - 1, teban)));
            node.write().unwrap().child.push(n.clone());
            leaves.push(n);
        }

        let mut leaves2nd = Vec::<(u8, u8, Arc<RwLock<ShNode>>)>::new();
        for leaf in leaves.iter() {
            let x = leaf.read().unwrap().x;
            let y = leaf.read().unwrap().y;
            let ban = ban.r#move(x, y).unwrap();
            let moves = ban.genmove();
            if moves.is_none() {
                let n = Arc::new(RwLock::new(
                        ShNode::new(0, 0, depth - 2, -teban)));
                leaf.write().unwrap().child.push(n.clone());
                leaves2nd.push((x, y, n.clone()));
// println!("{x} {y} 0 0  --");
                continue;
            }
            let moves = moves.unwrap();
            if moves.is_empty() {
                let n = Arc::new(RwLock::new(
                        ShNode::new(0, 0, depth - 2, -teban)));
                leaf.write().unwrap().child.push(n.clone());
                leaves2nd.push((x, y, n.clone()));
// println!("{x} {y} 0 0--");
                continue;
            }
            for (mvx, mvy) in moves {
// println!("{x} {y} {mvx} {mvy}  --");
                let n = Arc::new(RwLock::new(
                        ShNode::new(mvx, mvy, depth - 2, teban)));
                leaf.write().unwrap().child.push(n.clone());
                leaves2nd.push((x, y, n.clone()));
            }
        }
        let mut leaves2 = Vec::from_iter(leaves2nd[0..n/2].iter().cloned());
        let mut leaves1 = Vec::from_iter(leaves2nd[n/2..].iter().cloned());
        // let mut leaves1 = Vec::new();
        // let mut leaves2 = Vec::new();
        // for (idx, l) in leaves2nd.iter().enumerate() {
        //     if idx & 1 == 0 {
        //         leaves2.push((l.0, l.1, l.2.clone()));
        //     } else {
        //         leaves1.push((l.0, l.1, l.2.clone()));
        //     }
        // }
        let ban2 = ban.clone();
        let salpha = Arc::new(std::sync::Mutex::new(-100000.0f32));
        let sbeta = Arc::new(std::sync::Mutex::new(100000.0f32));
        let sal = salpha.clone();
        let sbe = sbeta.clone();
        let sub =
                thread::spawn(move || {
                    leaves1.sort_by(|(ax, ay, a), (bx, by, b)| {
                let pa;
                let pb;
                {let aa = a.read().unwrap();
                let bb = b.read().unwrap();
                let ia = move_priority2(&(*ax, *ay, aa.x, aa.y));
                let ib = move_priority2(&(*bx, *by, bb.x, bb.y));
                pa = SORT_PRI[ia as usize];
                pb = SORT_PRI[ib as usize];}
                pa.partial_cmp(&pb).unwrap()
            });
            //let mut tt = transptable::TranspositionTable::new();
            let teban = -ban2.teban;
            let mut alpha : f32 = *sal.lock().unwrap();
            let mut beta : f32 = *sbe.lock().unwrap();
    // let mut km = 0;
            for (x, y, leaf) in leaves1.iter_mut() {
                let xx;
                let yy;
                {
                    let lf = leaf.read().unwrap();
                    xx = lf.x;
                    yy = lf.y;
                }
                let newban = ban2.r#move(*x, *y).unwrap();
                let newban = newban.r#move(xx, yy).unwrap();
                let val = ShNode::think_internal_ab(leaf, &newban, alpha, beta);
// if xx == 0 {println!("{x} {y} {xx} {yy} - {:?}", val);}
                leaf.write().unwrap().hyoka = val;
                let val = val.unwrap();
        // km += leaf.lock().unwrap().kyokumen;
                if teban == bitboard::SENTE {
                    let mut sa = sal.lock().unwrap();
                    if val > *sa {
                        *sa = val;
                        alpha = val;
                    } else {
                        alpha = *sa;
                    }
                } else if teban == bitboard::GOTE {
                    let mut sb = sbe.lock().unwrap();
                    if val < *sb {
                        *sb = val;
                        beta = val;
                    } else {
                        beta = *sb;
                    }
                }
            }
    // println!(" ++ {km}");
        });

        leaves2.sort_by(|(ax, ay, a), (bx, by, b)| {
            let aa = a.read().unwrap();
            let bb = b.read().unwrap();
            let ia = move_priority2(&(*ax, *ay, aa.x, aa.y));
            let ib = move_priority2(&(*bx, *by, bb.x, bb.y));
            let pa = SORT_PRI[ia as usize];
            let pb = SORT_PRI[ib as usize];
            pa.partial_cmp(&pb).unwrap()
        });
        //let mut tt = transptable::TranspositionTable::new();
        let teban = -ban.teban;
        let mut alpha : f32 = *salpha.lock().unwrap();
        let mut beta : f32 = *sbeta.lock().unwrap();
// let mut km = 0;
        for (x, y, leaf) in leaves2.iter_mut() {
            let xx;
            let yy;
            {
                let lf = leaf.read().unwrap();
                xx = lf.x;
                yy = lf.y;
            }
            let newban = ban.r#move(*x, *y).unwrap();
            let newban = newban.r#move(xx, yy).unwrap();
            let val = ShNode::think_internal_ab(leaf, &newban, alpha, beta);
// if xx == 0 {println!("{x} {y} {xx} {yy} + {:?}", val);}
            leaf.write().unwrap().hyoka = val;
            let val = val.unwrap();
    // km += leaf.lock().unwrap().kyokumen;
            if teban == bitboard::SENTE {
                let mut sa = salpha.lock().unwrap();
                if val > *sa {
                    *sa = val;
                    alpha = val;
                } else {
                    alpha = *sa;
                }
            } else if teban == bitboard::GOTE {
                let mut sb = sbeta.lock().unwrap();
                if val < *sb {
                    *sb = val;
                    beta = val;
                } else {
                    beta = *sb;
                }
            }
        }
// println!(" -- {km}");

        sub.join().unwrap();

        let teban = ban.teban;
        let fteban = teban as f32;
        let mut hyoka : Option<f32> = None;
        {
            let teban2 = -teban;
            let fteban2 = teban2 as f32;
            let mut km = 0;
            let nd = &mut node.write().unwrap();
            for leaf in nd.child.iter() {
                let mut lf = leaf.write().unwrap();
                let mut km2 = 0;
                let mut hyo : Option<f32> = None;
                let mut bes : Option<Best> = None;
                for leaf2 in lf.child.iter() {
                    let lf2 = leaf2.read().unwrap();
                    km2 += lf2.kyokumen;
                    let hk = lf2.hyoka;
                    if hk.is_none() {
// println!("continue;;");
                        continue;
                    }
                    if hyo.is_none() || hk.unwrap() * fteban2 > hyo.unwrap() * fteban2 {
                        hyo = hk;
                        bes = Some(Best::new(hk.unwrap(), lf2.x, lf2.y));
                    }
                }
                lf.hyoka = hyo;
                lf.best = bes;
                lf.kyokumen = km2;
                km += km2;
// println!("{}{} {:?}", lf.x, lf.y, lf.hyoka);
            }

            let mut be : Option<Best> = None;
            for leaf in nd.child.iter() {
                let lf = leaf.read().unwrap();
// println!("{}{} {:?}", lf.x, lf.y, lf.hyoka);
                let lb = lf.best.as_ref();
                if lf.hyoka.is_none() {
                    continue;
                }
                if hyoka.is_none() {
                    hyoka = lf.hyoka;
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y));
                } else if lb.is_none() {
                    // nothing to do.
                } else if hyoka.unwrap() * fteban < lf.hyoka.unwrap() * fteban {
                    hyoka = lf.hyoka;
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y));
                }
            }
            nd.hyoka = hyoka;
            nd.best = be;
            nd.kyokumen = km;
        }
        // println!("done.");
        Some((hyoka.unwrap(), node.clone()))
    }

    pub fn think_internal_ab(node:&Arc<RwLock<ShNode>>, ban : &bitboard::BitBoard,
        alpha : f32, beta : f32)
            -> Option<f32> {
        let mut nod = node.write().unwrap();
        let mut newalpha = alpha;
        let mut depth = nod.depth;
        if ban.is_full() || ban.is_passpass() {
            nod.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            nod.kyokumen = 1;
            return Some(ShNode::evaluate(ban));
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            nod.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.is_empty() {  // pass
            moves.push((0, 0));
            depth += 1;
        } else {
            // sort moves
            moves.sort_by(|a, b| {
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
        }

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = nod.child.len();
            let leaf = Arc::new(RwLock::new(
                    ShNode::new(mvx, mvy, depth - 1, teban)));
            nod.child.push(leaf.clone());
            let val = ShNode::think_internal_ab(
                &nod.child[idx], &newban, -beta, -newalpha);

            {
                let mut lf = leaf.write().unwrap();
                lf.hyoka = val;
                nod.kyokumen += lf.kyokumen;
            }

            let best = nod.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }

            let fteban = teban as f32;
            if nod.best.is_none() {
                nod.best = Some(Best::new(val, mvx, mvy));
                // println!("n{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                // println!("b{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
                nod.best = Some(Best::new(val, mvx, mvy));
            } else if newalpha >= beta {
                // cut
                return Some(nod.best.as_ref().unwrap().hyoka);
            } else {
                // println!("b{depth}{}{} != b{mvx}{mvy}", nod.x, nod.y);
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                nod.child[idx].write().unwrap().release();
            }
        }
        Some(nod.best.as_ref().unwrap().hyoka)
    }

    fn release(&mut self) {
        self.child.clear();
    }

    pub fn to_xy(&self) -> String {
        format!("{}{}{}",
            if self.teban == board::SENTE {
                board::STONE_SENTE
            } else {
                board::STONE_GOTE
            },
            board::STR_GOTE.chars().nth(self.x as usize).unwrap(), self.y)
    }

    pub fn dump(&self) -> String {
        // let mut logg = String::new();
        let mut ret = String::new();
        ret += &format!("val:{:?}, {} nodes. ", self.hyoka, self.kyokumen);

        let best = self.best.as_ref();
        if best.is_none() {
            return ret;
        }
        let best = best.unwrap();
        // ret += &best.pos();
        let x = best.x;
        let y = best.y;
        let mut m = self.child.iter().find(|&a|
            {
                let n = a.read().unwrap();
                // logg += &format!("{}{},", n.x, n.y);
                n.x == x && n.y == y
            }
        );
        if m.is_none() {
            return ret;
        }
        // logg += "#";
        let mut nd = m.unwrap().clone();
        loop {
            ret += &nd.read().unwrap().to_xy();
            {
                let ndt = nd.clone();
                let nod = ndt.read().unwrap();
                let besto = nod.best.as_ref();
                if besto.is_none() {
                    // logg += "b.is_none";
                    return ret;
                }
                let besto = besto.unwrap();
                // ret += &besto.pos();

                let x = besto.x;
                let y = besto.y;
                // logg += &format!("b is({x}{y}) ");
                m = nod.child.iter().find(|&a|
                    {
                        let n = a.read().unwrap();
                        // logg += &format!("{}{},", n.x, n.y);
                        n.x == x && n.y == y
                    }
                );
                if m.is_none() {
                    // logg += &format!("m.is_none for {x}{y}");
                    // panic!("{}", logg);
                    return ret;
                }
                nd = m.unwrap().clone();
            }
        }
    }
}

#[test]
fn test_shnode() {
    let node = Arc::new(RwLock::new(ShNode::new(99, 2, 8, bitboard::NONE)));
    let node12 = Arc::new(RwLock::new(ShNode::new(1, 2, 7, bitboard::SENTE)));
    node12.write().unwrap().kyokumen = 8765;
    let node34 = Arc::new(RwLock::new(ShNode::new(3, 4, 7, bitboard::GOTE)));
    node34.write().unwrap().kyokumen = 7654;
    node.write().unwrap().child.push(node12.clone());
    node.write().unwrap().child.push(node34.clone());
    node.write().unwrap().hyoka = Some(99.9);
    node.write().unwrap().kyokumen = 9876;
    node.write().unwrap().best = Some(Best::new(99.9, 1, 2));
    let node56 = Arc::new(RwLock::new(ShNode::new(5, 6, 6, bitboard::NONE)));
    node56.write().unwrap().kyokumen = 6543;
    let node78 = Arc::new(RwLock::new(ShNode::new(7, 8, 6, bitboard::NONE)));
    node78.write().unwrap().kyokumen = 5432;
    node12.write().unwrap().child.push(node56.clone());
    node12.write().unwrap().child.push(node78.clone());
    node12.write().unwrap().hyoka = Some(99.9);
    node12.write().unwrap().best = Some(Best::new(99.9, 7, 8));
    let node9a = Arc::new(RwLock::new(ShNode::new(2, 1, 5, bitboard::SENTE)));
    node9a.write().unwrap().kyokumen = 4321;
    let nodebc = Arc::new(RwLock::new(ShNode::new(4, 3, 5, bitboard::NONE)));
    nodebc.write().unwrap().kyokumen = 3210;
    node78.write().unwrap().child.push(node9a.clone());
    node78.write().unwrap().child.push(nodebc.clone());
    node78.write().unwrap().hyoka = Some(99.9);
    node78.write().unwrap().best = Some(Best::new(99.9, 2, 1));
    let nodede = Arc::new(RwLock::new(ShNode::new(6, 5, 4, bitboard::NONE)));
    let nodefg = Arc::new(RwLock::new(ShNode::new(8, 7, 4, bitboard::NONE)));
    node9a.write().unwrap().child.push(nodede.clone());
    node9a.write().unwrap().child.push(nodefg.clone());
    node9a.write().unwrap().hyoka = Some(99.9);
    node9a.write().unwrap().best = Some(Best::new(99.9, 8, 7));

    assert_eq!(node.read().unwrap().dump(), "val:Some(99.9), 9876 nodes. @@a2[]g8@@b1[]h7");
    assert_eq!(node12.read().unwrap().dump(), "val:Some(99.9), 8765 nodes. []g8@@b1[]h7");
    assert_eq!(node34.read().unwrap().dump(), "val:None, 7654 nodes. ");
    assert_eq!(node56.read().unwrap().dump(), "val:None, 6543 nodes. ");
    assert_eq!(node78.read().unwrap().dump(), "val:Some(99.9), 5432 nodes. @@b1[]h7");
    assert_eq!(node9a.read().unwrap().dump(), "val:Some(99.9), 4321 nodes. []h7");
    assert_eq!(nodebc.read().unwrap().dump(), "val:None, 3210 nodes. ");
}
