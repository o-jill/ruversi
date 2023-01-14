use super::*;
use std::sync::{Arc, RwLock, Mutex};

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

fn move_priority(&(x, y) : &(u8, u8)) -> i32 {
    let idx = if x == 0 || y == 0 {0} else {x + y * 8 - 9};
    SORT_PRI[idx as usize]
}

fn move_priority2(&(x1, y1, x2, y2) : &(u8, u8, u8, u8)) -> i32 {
    let idx1 = if x1 == 0 || y1 == 0 {0} else {x1 + y1 * 8 - 9};
    let idx2 = if x2 == 0 || y2 == 0 {0} else {x2 + y2 * 8 - 9};
    SORT_PRI[idx1 as usize] * 10 + SORT_PRI[idx2 as usize]
}

fn move_priority3(&(x1, y1, x2, y2, x3, y3)
        : &(u8, u8, u8, u8, u8, u8)) -> i32 {
    let idx1 = if x1 == 0 || y1 == 0 {0} else {x1 + y1 * 8 - 9};
    let idx2 = if x2 == 0 || y2 == 0 {0} else {x2 + y2 * 8 - 9};
    let idx3 = if x3 == 0 || y3 == 0 {0} else {x3 + y3 * 8 - 9};
    SORT_PRI[idx1 as usize] * 100 + SORT_PRI[idx2 as usize] * 10
        + SORT_PRI[idx3 as usize]
}

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
// static mut WEIGHT : Option<Vec<f32>> = None;
pub static mut WEIGHT : &Option<weight::Weight> = unsafe {&nodebb::WEIGHT};

#[derive(Clone)]
pub struct Best {
    pub hyoka : f32,
    pub x : u8,
    pub y : u8,
    pub teban : i8,
}

impl Best {
    pub fn new(h : f32, x : u8, y : u8, t : i8) -> Best {
        Best { hyoka: h, x: x, y: y, teban: t }
    }

    pub fn pos(&self) -> String {
        format!("{}{}{}",
            if self.teban == board::SENTE {
                board::STONE_SENTE
            } else {
                board::STONE_GOTE
            },
            board::STR_GOTE.chars().nth(self.x as usize).unwrap(), self.y)
    }

    #[allow(dead_code)]
    pub fn to_str(&self) -> String {
        format!("h:{} {}", self.hyoka, self.pos())
    }
}

pub struct ShNode {
    child : Vec<Arc<Mutex<ShNode>>>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    pub best : Option<Best>,
    pub x : u8,
    pub y : u8,
    depth : u8,
}

impl ShNode {
    pub fn new(x : u8, y : u8, depth : u8) -> ShNode {
        ShNode {
            child : Vec::<Arc<Mutex<ShNode>>>::new(),
            hyoka : None,
            kyokumen : 0,
            best : None,
            x : x,
            y : y,
            depth : depth,
        }
    }

    fn evaluate(ban : &bitboard::BitBoard) -> f32 {
        // unsafe{ return WEIGHT.as_ref().unwrap().evaluatev3bb(ban)}
        unsafe {
            if cfg!(feature="nosimd") {
                WEIGHT.as_ref().unwrap().evaluatev3bb(ban)
            } else if cfg!(feature="avx") {
                WEIGHT.as_ref().unwrap().evaluatev3bb_simdavx(ban)
            } else {
                WEIGHT.as_ref().unwrap().evaluatev3bb_simd(ban)
            }
        }
    }

    fn evalwtt(ban : &bitboard::BitBoard, tt : &mut transptable::TranspositionTable) -> f32 {
        let id = if cfg!(feature="nosimd") {ban.to_id()} else {ban.to_id_simd()};
        tt.check_or_append(&id, || ShNode::evaluate(ban))
    }

    pub fn think(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, Arc<Mutex<ShNode>>)> {
// println!("shnode::think(ban, d:{depth})");
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            return None;
        }

        let node = Arc::new(Mutex::new(ShNode::new(0, 0, depth)));
        // println!("{}", node.lock().unwrap().dump());
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            node.lock().unwrap().depth += 1;
            depth += 1;
        }
        let teban = ban.teban;
        // let ddd = node.lock().unwrap().depth;
        let n = moves.len();
        let mut leaves = Vec::<Arc<Mutex<ShNode>>>::new();
        for (mvx, mvy) in moves {
            let n = Arc::new(Mutex::new(ShNode::new(mvx, mvy, depth - 1)));
            node.lock().unwrap().child.push(n.clone());
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
                    let lf = leaf.lock().unwrap();
                    x = lf.x;
                    y = lf.y;
                }
                let newban = ban2.r#move(x, y).unwrap();
                let val = ShNode::think_internal(leaf, &newban);
                leaf.lock().unwrap().hyoka = val;
            }
        });

        let teban = ban.teban;
        for leaf in leaves2.iter_mut() {
            let x;
            let y;
            {
                let lf = leaf.lock().unwrap();
                x = lf.x;
                y = lf.y;
            }
            let newban = ban.r#move(x, y).unwrap();
            let val = ShNode::think_internal(leaf, &newban);
            leaf.lock().unwrap().hyoka = val;
        }
        sub.join().unwrap();
        // tt.dumpsz();

        let fteban = teban as f32;
        let hyoka;
        {
            let nd = &mut node.lock().unwrap();
            let mut be : Option<Best> = None;
            let mut km = 0;
            for leaf in nd.child.iter() {
                let lf = leaf.lock().unwrap();
                km += lf.kyokumen;

                let lb = lf.best.as_ref();
                if be.is_none() {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y, teban));
                } else if lb.is_none() {
                    // nothing to do.
                } else if be.as_ref().unwrap().hyoka * fteban < lb.as_ref().unwrap().hyoka * fteban {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y, teban));
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

    pub fn think_internal(node:&Arc<Mutex<ShNode>>, ban : &bitboard::BitBoard)
            -> Option<f32> {
        let mut nod = node.lock().unwrap();
        let mut depth = nod.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            nod.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            nod.kyokumen = 1;
            return Some(ShNode::evaluate(&ban));
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
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
        }

        let mut hyoka = -9999999.0;
        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = nod.child.len();
            let leaf = Arc::new(Mutex::new(ShNode::new(mvx, mvy, depth - 1)));
            nod.child.push(leaf.clone());
            let val = ShNode::think_internal(
                &mut nod.child[idx], &newban);

            {
                let mut lf = leaf.lock().unwrap();
                lf.hyoka = val;
                nod.kyokumen += lf.kyokumen;
            }

            let best = nod.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if nod.best.is_none() {
                nod.best = Some(Best::new(val, mvx, mvy, teban));
                hyoka = val;
                // println!("n{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                // println!("b{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
                nod.best = Some(Best::new(val, mvx, mvy, teban));
                hyoka = val;
            } else {
                // println!("b{depth}{}{} != b{mvx}{mvy}", nod.x, nod.y);
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                nod.child[idx].lock().unwrap().release();
            }
        }
        Some(hyoka)
    }

    pub fn think_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, Arc<Mutex<ShNode>>)> {
        let node = Arc::new(Mutex::new(ShNode::new(0, 0, depth)));
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            return None;
        }
        // let mut tt = transptable::TranspositionTable::new();
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
            node.lock().unwrap().depth += 1;
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
        let mut leaves = Vec::<Arc<Mutex<ShNode>>>::new();
        for (mvx, mvy) in moves {
            let n = Arc::new(Mutex::new(ShNode::new(mvx, mvy, depth - 1)));
            node.lock().unwrap().child.push(n.clone());
            leaves.push(n);
        }
        let mut leaves1 = Vec::from_iter(leaves[0..n/2].iter().cloned());
        let mut leaves2 = Vec::from_iter(leaves[n/2..].iter().cloned());
        let ban2 = ban.clone();

        let sub =
                thread::spawn(move || {
            leaves1.sort_by(|a, b| {
                let aa = a.lock().unwrap();
                let bb = b.lock().unwrap();
                let ia = aa.x + aa.y * 8 - 9;
                let ib = bb.x + bb.y * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
            //let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for leaf in leaves1.iter_mut() {
                    let x;
                let y;
                {
                    let lf = leaf.lock().unwrap();
                    x = lf.x;
                    y = lf.y;
                }
                let newban = ban2.r#move(x, y).unwrap();
                let val = ShNode::think_internal_ab(leaf, &newban, alpha, beta);
                leaf.lock().unwrap().hyoka = val;
                let val = val.unwrap();
                if teban == bitboard::SENTE && alpha < val {
                    alpha = val;
                } else if teban == bitboard::GOTE && beta > val {
                    beta = val;
                }
            }
        });

        leaves2.sort_by(|a, b| {
            let aa = a.lock().unwrap();
            let bb = b.lock().unwrap();
            let ia = aa.x + aa.y * 8 - 9;
            let ib = bb.x + bb.y * 8 - 9;
            let pa = SORT_PRI[ia as usize];
            let pb = SORT_PRI[ib as usize];
            pa.partial_cmp(&pb).unwrap()
        });
        //let mut tt = transptable::TranspositionTable::new();
        let teban = ban.teban;
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        for leaf in leaves2.iter_mut() {
                let x;
            let y;
            {
                let lf = leaf.lock().unwrap();
                x = lf.x;
                y = lf.y;
            }
            let newban = ban.r#move(x, y).unwrap();
            let val = ShNode::think_internal_ab(leaf, &newban, alpha, beta);
            leaf.lock().unwrap().hyoka = val;
            let val = val.unwrap();
            if teban == bitboard::SENTE && alpha < val {
                alpha = val;
            } else if teban == bitboard::GOTE && beta > val {
                beta = val;
            }
        }

        sub.join().unwrap();

        let fteban = teban as f32;
        let hyoka;
        {
            let nd = &mut node.lock().unwrap();
            let mut be : Option<Best> = None;
            let mut km = 0;
            for leaf in nd.child.iter() {
                let lf = leaf.lock().unwrap();
                km += lf.kyokumen;

                let lb = lf.best.as_ref();
                if be.is_none() {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y, teban));
                } else if lb.is_none() {
                    // nothing to do.
                } else if be.as_ref().unwrap().hyoka * fteban < lb.as_ref().unwrap().hyoka * fteban {
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y, teban));
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

    pub fn think_ab_extract2(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, Arc<Mutex<ShNode>>)> {
        let node = Arc::new(Mutex::new(ShNode::new(0, 0, depth)));
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            return None;
        }
        // let mut tt = transptable::TranspositionTable::new();
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
            node.lock().unwrap().depth += 1;
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
        let mut leaves = Vec::<Arc<Mutex<ShNode>>>::new();
        for (mvx, mvy) in moves {
            let n = Arc::new(Mutex::new(ShNode::new(mvx, mvy, depth - 1)));
            node.lock().unwrap().child.push(n.clone());
            leaves.push(n);
        }

        let mut leaves2nd = Vec::<(u8, u8, Arc<Mutex<ShNode>>)>::new();
        for leaf in leaves.iter() {
            let x = leaf.lock().unwrap().x;
            let y = leaf.lock().unwrap().y;
            let ban = ban.r#move(x, y).unwrap();
            let moves = ban.genmove();
            if moves.is_none() {
                let n = Arc::new(Mutex::new(ShNode::new(0, 0, depth - 2)));
                leaf.lock().unwrap().child.push(n.clone());
                leaves2nd.push((x, y, n.clone()));
// println!("{x} {y} 0 0  --");
                continue;
            }
            let moves = moves.unwrap();
            if moves.is_empty() {
                let n = Arc::new(Mutex::new(ShNode::new(0, 0, depth - 2)));
                leaf.lock().unwrap().child.push(n.clone());
                leaves2nd.push((x, y, n.clone()));
// println!("{x} {y} 0 0--");
                continue;
            }
            for (mvx, mvy) in moves {
// println!("{x} {y} {mvx} {mvy}  --");
                let n = Arc::new(Mutex::new(ShNode::new(mvx, mvy, depth - 2)));
                leaf.lock().unwrap().child.push(n.clone());
                leaves2nd.push((x, y, n.clone()));
            }
        }
        // let mut leaves2 = Vec::from_iter(leaves2nd[0..n/2].iter().cloned());
        // let mut leaves1 = Vec::from_iter(leaves2nd[n/2..].iter().cloned());
        let mut leaves1 = Vec::new();
        let mut leaves2 = Vec::new();
        for (idx, l) in leaves2nd.iter().enumerate() {
            if idx & 1 == 0 {
                leaves2.push((l.0, l.1, l.2.clone()));
            } else {
                leaves1.push((l.0, l.1, l.2.clone()));
            }
        }
        let ban2 = ban.clone();

        let sub =
                thread::spawn(move || {
            leaves1.sort_by(|(ax, ay, a), (bx, by, b)| {
                let pa;
                let pb;
                {let aa = a.lock().unwrap();
                let bb = b.lock().unwrap();
                let ia = move_priority2(&(*ax, *ay, aa.x, aa.y));
                let ib = move_priority2(&(*bx, *by, bb.x, bb.y));
                pa = SORT_PRI[ia as usize];
                pb = SORT_PRI[ib as usize];}
                pa.partial_cmp(&pb).unwrap()
            });
            //let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
    // let mut km = 0;
            for (x, y, leaf) in leaves1.iter_mut() {
                let xx;
                let yy;
                {
                    let lf = leaf.lock().unwrap();
                    xx = lf.x;
                    yy = lf.y;
                }
                let newban = ban2.r#move(*x, *y).unwrap();
                let newban = newban.r#move(xx, yy).unwrap();
                let val = ShNode::think_internal_ab(&leaf, &newban, alpha, beta);
// if xx == 0 {println!("{x} {y} {xx} {yy} - {:?}", val);}
                leaf.lock().unwrap().hyoka = val;
                let val = val.unwrap();
        // km += leaf.lock().unwrap().kyokumen;
                if teban == bitboard::SENTE && alpha < val {
                    alpha = val;
                } else if teban == bitboard::GOTE && beta > val {
                    beta = val;
                }
            }
    // println!(" ++ {km}");
        });

        leaves2.sort_by(|(ax, ay, a), (bx, by, b)| {
            let aa = a.lock().unwrap();
            let bb = b.lock().unwrap();
            let ia = move_priority2(&(*ax, *ay, aa.x, aa.y));
            let ib = move_priority2(&(*bx, *by, bb.x, bb.y));
            let pa = SORT_PRI[ia as usize];
            let pb = SORT_PRI[ib as usize];
            pa.partial_cmp(&pb).unwrap()
        });
        //let mut tt = transptable::TranspositionTable::new();
        let teban = ban.teban;
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
    // let mut km = 0;
        for (x, y, leaf) in leaves2.iter_mut() {
            let xx;
            let yy;
            {
                let lf = leaf.lock().unwrap();
                xx = lf.x;
                yy = lf.y;
            }
            let newban = ban.r#move(*x, *y).unwrap();
            let newban = newban.r#move(xx, yy).unwrap();
            let val = ShNode::think_internal_ab(&leaf, &newban, alpha, beta);
// if xx == 0 {println!("{x} {y} {xx} {yy} + {:?}", val);}
            leaf.lock().unwrap().hyoka = val;
            let val = val.unwrap();
    // km += leaf.lock().unwrap().kyokumen;
            if teban == bitboard::SENTE && alpha < val {
                alpha = val;
            } else if teban == bitboard::GOTE && beta > val {
                beta = val;
            }
        }
// println!(" -- {km}");

        sub.join().unwrap();

        let fteban = teban as f32;
        let mut hyoka : Option<f32> = None;
        {
            let teban2 = -teban;
            let fteban2 = teban2 as f32;
            let mut km = 0;
            let nd = &mut node.lock().unwrap();
            for leaf in nd.child.iter() {
                let mut lf = leaf.lock().unwrap();
                let mut km2 = 0;
                let mut hyo : Option<f32> = None;
                let mut bes : Option<Best> = None;
                for leaf2 in lf.child.iter() {
                    let lf2 = leaf2.lock().unwrap();
                    km2 += lf2.kyokumen;
                    let hk = lf2.hyoka;
                    if hk.is_none() {
// println!("continue;;");
                        continue;
                    }
                    if hyo.is_none() || hk.unwrap() * fteban2 > hyo.unwrap() * fteban2 {
                        hyo = hk;
                        bes = Some(Best::new(hk.unwrap(), lf2.x, lf2.y, teban2));
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
                let lf = leaf.lock().unwrap();
// println!("{}{} {:?}", lf.x, lf.y, lf.hyoka);
                let lb = lf.best.as_ref();
                if lf.hyoka.is_none() {
                    continue;
                }
                if hyoka.is_none() {
                    hyoka = lf.hyoka;
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y, teban));
                } else if lb.is_none() {
                    // nothing to do.
                } else if hyoka.unwrap() * fteban < lf.hyoka.unwrap() * fteban {
                    hyoka = lf.hyoka;
                    be = Some(Best::new(lf.hyoka.unwrap(), lf.x, lf.y, teban));
                }
            }
            nd.hyoka = hyoka;
            nd.best = be;
            nd.kyokumen = km;
        }
        // println!("done.");
        Some((hyoka.unwrap(), node.clone()))
    }

    pub fn think_internal_ab(node:&Arc<Mutex<ShNode>>, ban : &bitboard::BitBoard,
        alpha : f32, beta : f32)
            -> Option<f32> {
        let mut nod = node.lock().unwrap();
        let mut newalpha = alpha;
        let mut depth = nod.depth;
        if ban.is_full() || ban.is_passpass() {
            nod.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            nod.kyokumen = 1;
            return Some(ShNode::evaluate(&ban));
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
        if moves.len() == 0 {  // pass
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
            let leaf = Arc::new(Mutex::new(ShNode::new(mvx, mvy, depth - 1)));
            nod.child.push(leaf.clone());
            let val = ShNode::think_internal_ab(
                &mut nod.child[idx], &newban, -beta, -newalpha);

            {
                let mut lf = leaf.lock().unwrap();
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
                nod.best = Some(Best::new(val, mvx, mvy, teban));
                // println!("n{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                // println!("b{depth}{}{} -> b{mvx}{mvy}", nod.x, nod.y);
                nod.best = Some(Best::new(val, mvx, mvy, teban));
            } else if newalpha >= beta {
                // cut
                return Some(nod.best.as_ref().unwrap().hyoka);
            } else {
                // println!("b{depth}{}{} != b{mvx}{mvy}", nod.x, nod.y);
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                nod.child[idx].lock().unwrap().release();
            }
        }
        Some(nod.best.as_ref().unwrap().hyoka)
    }

    fn release(&mut self) {
        self.child.clear();
    }

    pub fn dump(&self) -> String {
        let mut logg = String::new();
        let mut ret = String::new();
        ret += &format!("val:{:?}, {} nodes. ", self.hyoka, self.kyokumen);

        let best = self.best.as_ref();
        if best.is_none() {
            return ret;
        }
        let best = best.unwrap();
        ret += &best.pos();
        let x = best.x;
        let y = best.y;
        let mut m = self.child.iter().find(|&a|
            {
                let n = a.lock().unwrap();
                logg += &format!("{}{},", n.x, n.y);
                n.x == x && n.y == y
            }
        );
        if m.is_none() {
            return ret;
        }
        logg += "#";
        let mut nd = m.unwrap().clone();
        loop {
            {
                let ndt = nd.clone();
                let nod = ndt.lock().unwrap();
                let besto = nod.best.as_ref();
                if besto.is_none() {
                    logg += "b.is_none";
                    return ret;
                }
                let besto = besto.unwrap();
                ret += &besto.pos();

                let x = besto.x;
                let y = besto.y;
                logg += &format!("b is({x}{y}) ");
                m = nod.child.iter().find(|&a|
                    {
                        let n = a.lock().unwrap();
                        logg += &format!("{}{},", n.x, n.y);
                        n.x == x && n.y == y
                    }
                );
                if m.is_none() {
                    logg += &format!("m.is_none for {x}{y}");
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
    let node = Arc::new(Mutex::new(ShNode::new(99, 2, 8)));
    let node12 = Arc::new(Mutex::new(ShNode::new(1, 2, 7)));
    node12.lock().unwrap().kyokumen = 8765;
    let node34 = Arc::new(Mutex::new(ShNode::new(3, 4, 7)));
    node34.lock().unwrap().kyokumen = 7654;
    node.lock().unwrap().child.push(node12.clone());
    node.lock().unwrap().child.push(node34.clone());
    node.lock().unwrap().hyoka = Some(99.9);
    node.lock().unwrap().kyokumen = 9876;
    node.lock().unwrap().best = Some(Best::new(99.9, 1, 2, bitboard::SENTE));
    let node56 = Arc::new(Mutex::new(ShNode::new(5, 6, 6)));
    node56.lock().unwrap().kyokumen = 6543;
    let node78 = Arc::new(Mutex::new(ShNode::new(7, 8, 6)));
    node78.lock().unwrap().kyokumen = 5432;
    node12.lock().unwrap().child.push(node56.clone());
    node12.lock().unwrap().child.push(node78.clone());
    node12.lock().unwrap().hyoka = Some(99.9);
    node12.lock().unwrap().best = Some(Best::new(99.9, 7, 8, bitboard::GOTE));
    let node9a = Arc::new(Mutex::new(ShNode::new(2, 1, 5)));
    node9a.lock().unwrap().kyokumen = 4321;
    let nodebc = Arc::new(Mutex::new(ShNode::new(4, 3, 5)));
    nodebc.lock().unwrap().kyokumen = 3210;
    node78.lock().unwrap().child.push(node9a.clone());
    node78.lock().unwrap().child.push(nodebc.clone());
    node78.lock().unwrap().hyoka = Some(99.9);
    node78.lock().unwrap().best = Some(Best::new(99.9, 2, 1, bitboard::SENTE));
    let nodede = Arc::new(Mutex::new(ShNode::new(6, 5, 4)));
    let nodefg = Arc::new(Mutex::new(ShNode::new(8, 7, 4)));
    node9a.lock().unwrap().child.push(nodede.clone());
    node9a.lock().unwrap().child.push(nodefg.clone());
    node9a.lock().unwrap().hyoka = Some(99.9);
    node9a.lock().unwrap().best = Some(Best::new(99.9, 8, 7, bitboard::GOTE));

    assert_eq!(node.lock().unwrap().dump(), "val:Some(99.9), 9876 nodes. @@a2[]g8@@b1[]h7");
    assert_eq!(node12.lock().unwrap().dump(), "val:Some(99.9), 8765 nodes. []g8@@b1[]h7");
    assert_eq!(node34.lock().unwrap().dump(), "val:None, 7654 nodes. ");
    assert_eq!(node56.lock().unwrap().dump(), "val:None, 6543 nodes. ");
    assert_eq!(node78.lock().unwrap().dump(), "val:Some(99.9), 5432 nodes. @@b1[]h7");
    assert_eq!(node9a.lock().unwrap().dump(), "val:Some(99.9), 4321 nodes. []h7");
    assert_eq!(nodebc.lock().unwrap().dump(), "val:None, 3210 nodes. ");
}
