use super::*;
// use std::sync::Arc;

const SORT_PRI : [i32 ; 81]= [
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 3, 1, 2, 2, 1, 3, 0,
    0, 3, 3, 4, 4, 4, 4, 3, 3,
    0, 1, 4, 5, 5, 5, 5, 4, 1,
    0, 2, 4, 5, 5, 5, 5, 4, 2,
    0, 2, 4, 5, 5, 5, 5, 4, 2,
    0, 1, 4, 5, 5, 5, 5, 4, 1,
    0, 3, 3, 4, 4, 4, 4, 3, 3,
    0, 0, 3, 1, 2, 2, 1, 3, 0,
];

fn move_priority(&(x, y) : &(u8, u8)) -> i32 {
    let idx = x + y * 9;
    SORT_PRI[idx as usize]
}

fn move_priority2(&(x1, y1, x2, y2) : &(u8, u8, u8, u8)) -> i32 {
    let idx1 = x1 + y1 * 9;
    let idx2 = x2 + y2 * 9;
    SORT_PRI[idx1 as usize] * 10 + SORT_PRI[idx2 as usize]
}

#[allow(dead_code)]
fn move_priority3(&(x1, y1, x2, y2, x3, y3)
        : &(u8, u8, u8, u8, u8, u8)) -> i32 {
    let idx1 = x1 + y1 * 9;
    let idx2 = x2 + y2 * 9;
    let idx3 = x3 + y3 * 9;
    SORT_PRI[idx1 as usize] * 100 + SORT_PRI[idx2 as usize] * 10
        + SORT_PRI[idx3 as usize]
}

static mut INITIALIZED : bool = false;

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
// static mut WEIGHT : Option<Vec<f32>> = None;
pub static mut WEIGHT : Option<weight::Weight> = None;
static mut ND_ROOT : Option<NodeBB> = None;

pub struct Best {
    pub hyoka : f32,
    pub x : u8,
    pub y : u8,
}

impl Best {
    pub fn new(hyoka : f32, x : u8, y : u8) -> Best {
        Best { hyoka, x, y }
    }

    pub fn pos(&self) -> String {
        format!("{}{}",
            // if teban == board::SENTE {
            //     board::STONE_SENTE
            // } else {
            //     board::STONE_GOTE
            // },
            board::STR_SENTE.chars().nth(self.x as usize).unwrap(), self.y)
    }

    #[allow(dead_code)]
    pub fn to_str(&self) -> String {
        format!("h:{} {}", self.hyoka, self.pos())
    }
}

pub struct NodeBB {
    child : Vec<NodeBB>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    pub best : Option<Best>,
    pub x : u8,
    pub y : u8,
    depth : u8,
    pub teban : i8,
}

pub fn init_weight() {
    unsafe {
        if INITIALIZED {
            return;
        }
    }

    let mut weight = weight::Weight::new();
    weight.init();

    unsafe {
        WEIGHT = Some(weight);
        ND_ROOT = Some(NodeBB::new(0, 0, 0, bitboard::NONE));
        INITIALIZED = true;
    }
}

impl NodeBB {
    pub fn new(x : u8, y : u8, depth : u8, teban : i8) -> NodeBB {
        NodeBB {
            child : Vec::<NodeBB>::new(),
            hyoka : None,
            kyokumen : 1,
            best : None,
            x,
            y,
            depth,
            teban,
        }
    }

    fn asignbest(&mut self, hyoka : f32, x : u8, y : u8) {
        self.hyoka = Some(hyoka);
        self.best = Some(Best::new(hyoka, x, y));
    }

    #[cfg(target_arch="x86_64")]
    fn evaluate(ban : &bitboard::BitBoard, wei : &weight::Weight) -> f32 {
        if cfg!(feature="nosimd") {
            wei.evaluatev9bb(ban)
        } else if cfg!(feature="avx") {
            wei.evaluatev9bb_simdavx(ban)
        } else {
            wei.evaluatev9bb_simd(ban)
        }
    }

    #[cfg(target_arch="aarch64")]
    fn evaluate(ban : &bitboard::BitBoard, wei : &weight::Weight) -> f32 {
        if cfg!(feature="nosimd") {
            wei.evaluatev9bb(ban)
        } else {
            wei.evaluatev9bb_simd_mul(ban)
        }
    }

    fn evalwtt(ban : &bitboard::BitBoard, wei : &weight::Weight, tt : &mut transptable::TranspositionTable) -> f32 {
        let id = if cfg!(feature="nosimd") {ban.to_id()} else {ban.to_id_simd()};
        tt.check_or_append(&id, || NodeBB::evaluate(ban, wei))
    }

    pub fn thinko(ban : &bitboard::BitBoard, depth : u8)
            -> Option<(f32, &NodeBB)> {
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }

        // let sum = 0;
        // no more empty cells
        let moves = ban.genmove()?;
        // let mut moves = ban.genmove()?;

        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        let node;
        let teban = ban.teban;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth, 0));
            node = ND_ROOT.as_mut().unwrap();
        }
        let n = moves.len();
        for (mvx, mvy) in moves.iter() {
            node.child.push(NodeBB::new(*mvx, *mvy, depth - 1, teban));
        }
// let moves1 = &moves[0..n/2];
        let moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();

        let sub = thread::spawn(move || {
            let node2;
            unsafe {
                node2 = ND_ROOT.as_mut().unwrap();
            }
            for (mvx, mvy) in moves1 {
                let nd = node2.child.iter_mut().find(|a| {
                        a.x == mvx && a.y == mvy
                    });
                if nd.is_none() {
                    panic!("node2.child.iter_mut().find(|a|");
                }
                let nd = nd.unwrap();
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let val = NodeBB::think_internal(nd, &newban, wei);
                nd.hyoka = val;
            }
            });

        for (mvx, mvy) in moves2 {
            let nd = node.child.iter_mut().find(|a|
                    a.x == mvx && a.y == mvy
                ).unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let val = NodeBB::think_internal(nd, &newban, wei);

            nd.hyoka = val;
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut hyoka = None;
        let mut be = None;
        let mut km = 0;
        let teban = ban.teban;
        let fteban = teban as f32;
        for c in node.child.iter() {
            km += c.kyokumen;
            if c.hyoka.is_none() {
                continue;
            }
            if hyoka.is_none() || hyoka.unwrap() * fteban < c.hyoka.unwrap() * fteban {
                hyoka = c.hyoka;
                be = Some(Best::new(hyoka.unwrap(), c.x, c.y));
            }
        }
        node.hyoka = hyoka;
        node.best = be;
        node.kyokumen = km;
        Some((hyoka.unwrap(), node))
    }

    #[allow(dead_code)]
    pub fn think(ban : &bitboard::BitBoard, depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth, bitboard::NONE);
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let moves = ban.genmove()?;
        // let mut moves = ban.genmove()?;

        let n = moves.len();
        // let moves1 = &moves[0..n/2];
        let moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            let wei2 = unsafe{WEIGHT.as_ref().unwrap()};
            let mut node2 = NodeBB::new(0, 0, depth, bitboard::NONE);
            let teban = ban2.teban;
            let mut tt = transptable::TranspositionTable::new();
            for (mvx, mvy) in moves1 {
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let idx = node2.child.len();
                node2.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_tt(
                            &mut node2.child[idx], &newban, wei2, &mut tt)
                    } else {
                        NodeBB::think_internal(
                            &mut node2.child[idx], &newban, wei2)
                    };
                let ch = &mut node2.child[idx];
                ch.hyoka = val;
                node2.kyokumen += ch.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if best.is_none()
                    || fteban * best.unwrap().hyoka < fteban * val {
                    node2.asignbest(val, mvx, mvy);
                } else {
                    // node2.child[node.child.len() - 1].as_ref().unwrap().release();
                    node2.child[idx].release();
                }
            }
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        let mut tt = transptable::TranspositionTable::new();
        let teban = ban.teban;
        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        for (mvx, mvy) in moves2 {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_tt(
                        &mut node.child[idx], &newban, wei, &mut tt)
                } else {
                    NodeBB::think_internal(
                        &mut node.child[idx], &newban, wei)
                };

            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none()
                || fteban * best.unwrap().hyoka < fteban * val {
                node.asignbest(val, mvx, mvy);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut subresult = rx.recv().unwrap();
        if subresult.best.is_none() ||
            node.best.as_ref().unwrap().hyoka * teban as f32
                > subresult.best.as_ref().unwrap().hyoka * teban as f32 {
            node.kyokumen += subresult.kyokumen;
            return Some((node.best.as_ref().unwrap().hyoka, node));
        }
        subresult.kyokumen += node.kyokumen;
        Some((subresult.best.as_ref().unwrap().hyoka, subresult))
    }

    pub fn think_internal(node:&mut NodeBB, ban : &bitboard::BitBoard, wei : &weight::Weight)
            -> Option<f32> {
        let depth = node.depth;
        if ban.is_full() || ban.is_passpass() || depth == 0 {
            return Some(NodeBB::evaluate(ban, wei));
        }

        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            return Some(ban.countf32());
        }
        let moves = moves.unwrap();

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let val = NodeBB::think_internal(
                &mut node.child[idx], &newban, wei);

            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none()
                || best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    #[allow(dead_code)]
    pub fn think_internal_tt(node:&mut NodeBB, ban : &bitboard::BitBoard, wei : &weight::Weight,
        tt : &mut transptable::TranspositionTable) -> Option<f32> {
        let depth = node.depth;
        if depth == 0 {
            return Some(NodeBB::evalwtt(ban, wei, tt));
        }
        if ban.is_passpass() {
            return Some(ban.countf32());
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            return Some(ban.countf32());
        }
        let moves = moves.unwrap();

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let val = NodeBB::think_internal_tt(
                &mut node.child[idx], &newban, wei, tt);

            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() || best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    #[allow(dead_code)]
    pub fn think_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth, bitboard::NONE);
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let moves = ban.genmove()?;

        let mut tt = transptable::TranspositionTable::new();
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }
        let n = moves.len();
        // let moves1 = &moves[0..n/2];
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let pa = move_priority(a);
                let pb = move_priority(b);
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let wei2 = unsafe{WEIGHT.as_ref().unwrap()};
            let mut node2 = NodeBB::new(0, 0, depth, bitboard::NONE);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for (mvx, mvy) in moves1 {
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let idx = node2.child.len();
                node2.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_ab_tt(
                            &mut node2.child[idx], &newban, alpha, beta, wei2, &mut tt)
                    } else {
                        NodeBB::think_internal_ab(
                            &mut node2.child[idx], &newban, alpha, beta, wei2)
                    };

                let ch = &mut node2.child[idx];
                let val = if teban == bitboard::SENTE {val} else {-val};
                ch.hyoka = Some(val);
                node2.kyokumen += ch.kyokumen;
                let best = node2.best.as_ref();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none()
                    || best.unwrap().hyoka * fteban < val * fteban {
                    node2.asignbest(val, mvx, mvy);
                } else {
                    // node2.child[node.child.len() - 1].as_ref().unwrap().release();
                    node2.child[idx].release();
                }
            }
            // tt.dumpsz();
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        moves2.sort_by(|a, b| {
            let pa = move_priority(a);
            let pb = move_priority(b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        for (mvx, mvy) in moves2 {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_ab_tt(
                        &mut node.child[idx], &newban, alpha, beta, wei, &mut tt)
                } else {
                    NodeBB::think_internal_ab(
                        &mut node.child[idx], &newban, alpha, beta, wei)
                };

            let ch = &mut node.child[idx];
            let val = if teban == bitboard::SENTE {val} else {-val};
            ch.hyoka = Some(val);
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let fteban = teban as f32;
            // println!("val:{}, a:{}, b:{}", val, alpha, beta);
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() || best.unwrap().hyoka * fteban < val * fteban {
                    node.asignbest(val, mvx, mvy);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut subresult = rx.recv().unwrap();
        if subresult.best.is_none() ||
            node.best.as_ref().unwrap().hyoka * teban as f32
                > subresult.best.as_ref().unwrap().hyoka * teban as f32 {
            node.kyokumen += subresult.kyokumen;
            return Some((node.best.as_ref().unwrap().hyoka, node));
        }
        subresult.kyokumen += node.kyokumen;
        Some((subresult.best.as_ref().unwrap().hyoka, subresult))
    }

    #[allow(dead_code)]
    pub fn thinko_ab_simple(ban : &bitboard::BitBoard, depth : u8)
            -> Option<(f32, &NodeBB)> {
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // no more empty cells
        let _moves = ban.genmove()?;

        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        let depth =
            if nblank <= yomikiri {
                yomikiri as u8
            } else if nblank <= yose {
                depth + 2
            } else {
                depth
            };

        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        let node;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth, bitboard::NONE));
            node = ND_ROOT.as_mut().unwrap();
        }

        let alpha : f32 = -123456.7;
        let beta : f32 = 123456.7;
        let val = NodeBB::think_internal_ab(node, ban, alpha, beta, wei);
        let val = val * ban.teban as f32;

        Some((val, node))
    }

    #[allow(dead_code)]
    pub fn thinko_ab_simple_gk(ban : &bitboard::BitBoard, depth : u8, nd : &mut NodeBB, wei : &weight::Weight)
            -> Option<f32> {
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // no more empty cells
        let _moves = ban.genmove()?;

        let node = nd;

        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        node.depth =
            if nblank <= yomikiri {
                yomikiri as u8
            } else if nblank <= yose {
                depth + 2
            } else {
                depth
            };

        let alpha : f32 = -123456.7;
        let beta : f32 = 123456.7;
        let val = NodeBB::think_internal_ab(node, ban, alpha, beta, wei);
        let val = val * ban.teban as f32;

        Some(val)
    }

    #[allow(dead_code)]
    pub fn thinko_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, &NodeBB)> {
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let moves = ban.genmove()?;

        let node;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth, bitboard::NONE));
            node = ND_ROOT.as_mut().unwrap();
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
        node.child.reserve(moves.len());
        for (mvx, mvy) in moves.iter() {
            node.child.push(NodeBB::new(*mvx, *mvy, depth - 1, teban));
        }
        // let moves1 = &moves[0..n/2];
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();

        // let salpha = Arc::new(std::sync::Mutex::new(-100000.0f32));
        // let sbeta = Arc::new(std::sync::Mutex::new(100000.0f32));
        // let sal = salpha.clone();
        // let sbe = sbeta.clone();

        let sub =
                thread::spawn(move || {
            let wei2 = unsafe{WEIGHT.as_ref().unwrap()};
            moves1.sort_by(|a, b| {
                let pa = move_priority(a);
                let pb = move_priority(b);
                pa.partial_cmp(&pb).unwrap()
            });
            let teban = ban2.teban;
            let node2;
            unsafe {
                node2 = ND_ROOT.as_mut().unwrap();
            }
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            // let mut alpha : f32 = *sal.lock().unwrap();
            // let mut beta : f32 = *sbe.lock().unwrap();
            for (mvx, mvy) in moves1 {
                let nd = node2.child.iter_mut().find(|a| {
                        a.x == mvx && a.y == mvy
                    });
                // if nd.is_none() {
                //     panic!("node2.child.iter_mut().find(|a|");
                // }
                let nd = nd.unwrap();
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta, wei2);
                let val = val * teban as f32;
                nd.hyoka = Some(val);
                if teban == bitboard::SENTE {
                    if val > alpha {
                        alpha = val;
                    }
                    // let mut sa = sal.lock().unwrap();
                    // if val > *sa {
                    //     *sa = val;
                    //     alpha = val;
                    // } else {
                    //     alpha = *sa;
                    // }
                } else if teban == bitboard::GOTE {
                    if val < beta {
                        beta = val;
                    }
                    // let mut sb = sbe.lock().unwrap();
                    // if val < *sb {
                    //     *sb = val;
                    //     beta = val;
                    // } else {
                    //     beta = *sb;
                    // }
                }
            }
        });

        moves2.sort_by(|a, b| {
            let pa = move_priority(a);
            let pb = move_priority(b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha = -100000.0f32;
        let mut beta = 100000.0f32;
        // let mut alpha : f32 = *salpha.lock().unwrap();
        // let mut beta : f32 = *sbeta.lock().unwrap();
        let teban = ban.teban;
        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        for (mvx, mvy) in moves2 {
            let nd = node.child.iter_mut().find(|a| {
                    a.x == mvx && a.y == mvy
                });
            // if nd.is_none() {
            //     panic!("node2.child.iter_mut().find(|a|");
            // }
            let nd = nd.unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta, wei);
            let val = if teban == bitboard::SENTE {val} else {-val};
            nd.hyoka = Some(val);
            if teban == bitboard::SENTE {
                if val > alpha {
                    alpha = val;
                }
                // let mut sa = salpha.lock().unwrap();
                // if val > *sa {
                //     *sa = val;
                //     alpha = val;
                // } else {
                //     alpha = *sa;
                // }
            } else if teban == bitboard::GOTE {
                if val < beta {
                    beta = val;
                }
                // let mut sb = sbeta.lock().unwrap();
                // if val < *sb {
                //     *sb = val;
                //     beta = val;
                // } else {
                //     beta = *sb;
                // }
            }
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut km = 0;
        let teban = ban.teban;
        let fteban = teban as f32;
        let mut hyoka = -98765.0 * fteban;
        let mut bx = 0;
        let mut by = 0;
        for c in node.child.iter() {
            km += c.kyokumen;
            if c.hyoka.is_none() {
                continue;
            }

            let chyoka = c.hyoka.unwrap();
            if hyoka * fteban < chyoka * fteban {
                hyoka = chyoka;
                bx = c.x;
                by = c.y;
            }
        }
        node.hyoka = Some(hyoka);
        node.best = Some(Best::new(hyoka, bx, by));
        node.kyokumen = km;
        Some((hyoka, node))
    }

    pub fn thinko_ab_extract2(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, &NodeBB)> {
        if depth <= 1 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let moves = ban.genmove()?;

        let node;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth, bitboard::NONE));
            node = ND_ROOT.as_mut().unwrap();
        }
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }
        let teban = ban.teban;
        let teban2 = -teban;
        let mut moves4 = Vec::<(u8, u8, u8, u8)>::new();
        for (mvx, mvy) in moves.iter() {
            node.child.push(NodeBB::new(*mvx, *mvy, depth - 1, teban));
            let nd = node.child.last_mut().unwrap();
            let newban = ban.r#move(*mvx, *mvy).unwrap();
            let moves = newban.genmove();
            if moves.is_none() {
                // println!("moves.len() == 0");
                nd.child.push(NodeBB::new(0, 0, depth - 1, teban2));
                moves4.push((*mvx, *mvy, 0, 0));
                continue;
            }

            let moves = moves.unwrap();

            for (mvx2, mvy2) in moves {
                nd.child.push(NodeBB::new(mvx2, mvy2, depth - 2, teban2));
                moves4.push((*mvx, *mvy, mvx2, mvy2));
            }
        }
        // let mut moves1 = Vec::from_iter(moves4[0..n/2].iter().cloned());
        // let mut moves2 = Vec::from_iter(moves4[n/2..].iter().cloned());
        let mut moves1 = Vec::new();
        let mut moves2 = Vec::new();
        for (idx, mv) in moves4.iter().enumerate() {
            if idx & 1 == 0 {
                moves2.push(*mv);
            } else {
                moves1.push(*mv);
            }
        }
        let ban2 = ban.clone();

        // let salpha = Arc::new(std::sync::Mutex::new(-100000.0f32));
        // let sbeta = Arc::new(std::sync::Mutex::new(100000.0f32));
        // let sal = salpha.clone();
        // let sbe = sbeta.clone();

        let sub =
                thread::spawn(move || {
            let wei2 = unsafe{WEIGHT.as_ref().unwrap()};
            moves1.sort_by(|a, b| {
                let pa = move_priority2(a);
                let pb = move_priority2(b);
                pa.partial_cmp(&pb).unwrap()
            });
            let teban = -ban2.teban;
            let node2;
            unsafe {
                node2 = ND_ROOT.as_mut().unwrap();
            }
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            // let mut alpha : f32 = *sal.lock().unwrap();
            // let mut beta : f32 = *sbe.lock().unwrap();
            for (mvx, mvy, mvx2, mvy2) in moves1 {
                let nd = node2.child.iter_mut().find(|a| {
                        a.x == mvx && a.y == mvy
                    }).unwrap();
                let nd2 = nd.child.iter_mut().find(|a| {
                        a.x == mvx2 && a.y == mvy2
                    });
                // if nd.is_none() {
                //     panic!("node2.child.iter_mut().find(|a|");
                // }
                let nd = nd2.unwrap();
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let newban = newban.r#move(mvx2, mvy2).unwrap();
                let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta, wei2);
                let val = if teban == bitboard::SENTE {val} else {-val};
                nd.hyoka = Some(val);
                if teban == bitboard::SENTE {
                    if val > alpha {
                        alpha = val;
                    }
                    // let mut sa = sal.lock().unwrap();
                    // if val > *sa {
                    //     *sa = val;
                    //     alpha = val;
                    // } else {
                    //     alpha = *sa;
                    // }
                } else if teban == bitboard::GOTE {
                    if val < beta {
                        beta = val;
                    }
                    // let mut sb = sbe.lock().unwrap();
                    // if val < *sb {
                    //     *sb = val;
                    //     beta = val;
                    // } else {
                    //     beta = *sb;
                    // }
                }
            }
        });

        moves2.sort_by(|a, b| {
            let pa = move_priority2(a);
            let pb = move_priority2(b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha = -100000.0f32;
        let mut beta = 100000.0f32;
        // let mut alpha : f32 = *salpha.lock().unwrap();
        // let mut beta : f32 = *sbeta.lock().unwrap();
        let teban = -ban.teban;
        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        for (mvx, mvy, mvx2, mvy2) in moves2 {
            let nd = node.child.iter_mut().find(|a| {
                    a.x == mvx && a.y == mvy
                }).unwrap();
            let nd2 = nd.child.iter_mut().find(|a| {
                    a.x == mvx2 && a.y == mvy2
                });
            // if nd.is_none() {
            //     panic!("node2.child.iter_mut().find(|a|");
            // }
            let nd = nd2.unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let newban = newban.r#move(mvx2, mvy2).unwrap();
            let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta, wei);
            let val = if teban == bitboard::SENTE {val} else {-val};
            nd.hyoka = Some(val);
            if teban == bitboard::SENTE {
                if val > alpha {
                    alpha = val;
                }
                // let mut sa = salpha.lock().unwrap();
                // if val > *sa {
                //     *sa = val;
                //     alpha = val;
                // } else {
                //     alpha = *sa;
                // }
            } else if teban == bitboard::GOTE {
                if val < beta {
                    beta = val;
                }
                // let mut sb = sbeta.lock().unwrap();
                // if val < *sb {
                //     *sb = val;
                //     beta = val;
                // } else {
                //     beta = *sb;
                // }
            }
        }
        sub.join().unwrap();
        // tt.dumpsz();

        let teban = ban.teban;
        let mut km = 0;
        for c in node.child.iter_mut() {
            let mut be = None;
            let mut km2 = 0;
            let teban2 = -teban;
            let fteban2 = teban2 as f32;
            let mut hyoka = -98765.0 * fteban2;
            for c2 in c.child.iter() {
                km2 += c2.kyokumen;
                if c2.hyoka.is_none() {
                    continue;
                }

                let chyoka = c2.hyoka.unwrap();
                if hyoka * fteban2 < chyoka * fteban2 {
                    hyoka = chyoka;
                    be = Some(Best::new(hyoka, c2.x, c2.y));
                }
            }
            c.hyoka = Some(hyoka);
            c.best = be;
            c.kyokumen = km;
            km += km2;
        }

        let fteban = teban as f32;
        let mut hyoka = -98765.0 * fteban;
        let mut be = None;
        for c in node.child.iter() {
            // println!("ch:{}{}", c.x, c.y);
            if c.hyoka.is_none() {
                // println!("c.hyoka.is_none");
                continue;
            }

            let chyoka = c.hyoka.unwrap();
            if hyoka * fteban < chyoka * fteban {
                // println!("update hyoka");
                hyoka = chyoka;
                be = Some(Best::new(hyoka, c.x, c.y));
            }
        }
        node.hyoka = Some(hyoka);
        node.best = be;
        node.kyokumen = km;
        Some((hyoka, node))
    }

    #[allow(dead_code)]
    pub fn think_ab_extract2(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth, bitboard::NONE);
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let moves1 = ban.genmove()?;

        let mut tt = transptable::TranspositionTable::new();
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }

        let mut moves = Vec::new();
        for &(a, b) in moves1.iter() {
            let ban = ban.r#move(a, b).unwrap();
            match ban.genmove() {
                None => {
                    moves.push((a, b, 0, 0));
                },
                Some(mvs) => {
                    for &(c, d) in mvs.iter() {
                        moves.push((a, b, c, d));
                    }
                },
            }
        }
        // println!("{:?}", moves);

        let n = moves.len();
        // let moves1 = &moves[0..n/2];
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let pa = move_priority2(a);
                let pb = move_priority2(b);
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let wei2 = unsafe{WEIGHT.as_ref().unwrap()};
            let mut node2 = NodeBB::new(0, 0, depth, bitboard::NONE);
            let mut alpha = -100000.0f32;
            let mut beta  = 100000.0f32;
            for mv in moves1 {
                let (x1, y1, x2, y2) = mv;

                let newban = ban2.r#move(x1, y1).unwrap();
                let newban2 = newban.r#move(x2, y2).unwrap();

                let nd1 =
                    match node2.child.iter_mut().find(
                        |a| a.x == x1 && a.y == y1) {
                        None => {
                            node2.child.push(NodeBB::new(x1, y1, depth - 1, teban));
                            node2.child.last_mut().unwrap()
                        },
                        Some(m) => m
                    };
                nd1.child.push(NodeBB::new(x2, y2, depth - 2, -teban));
                let nd2 = nd1.child.last_mut().unwrap();

                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_ab_tt(
                            nd2, &newban2, alpha, beta, wei2, &mut tt)
                    } else {
                        NodeBB::think_internal_ab(
                            nd2, &newban2, -beta, -alpha, wei2)
                            // &mut nd2, &newban2, alpha, beta)
                    };

                let val = if teban == bitboard::GOTE {val} else {-val};
                node2.kyokumen += nd2.kyokumen;
                nd2.hyoka = Some(val);
                let best = nd1.best.as_ref();
                let teban2 = newban.teban;
                let fteban2 = teban2 as f32;
                // println!("val:{}, a:{}, b:{}", val, alpha, beta);
                if teban2 == board::SENTE && -beta < val {
                    beta = -val;
                } else if teban2 == board::GOTE && -alpha > val {
                    alpha = -val;
                }
                // if teban2 == board::SENTE && alpha < val {
                //     alpha = val;
                // } else if teban2 == board::GOTE && beta > val {
                //     beta = val;
                // }
                if best.is_none()
                    || best.unwrap().hyoka * fteban2 < val * fteban2 {
                    nd1.asignbest(val, x2, y2);
                } else {
                    nd2.release();
                    continue;
                }
                let best = node2.best.as_ref();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none()
                    || best.unwrap().hyoka * fteban < val * fteban {
                    node2.asignbest(val, x1, y1);
                } else {
                    // nd2.release();
                }
                // if alpha >= beta {break;}
            }
            // tt.dumpsz();
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        moves2.sort_by(|a, b| {
            let pa = move_priority2(a);
            let pb = move_priority2(b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha = -100000.0f32;
        let mut beta = 100000.0f32;
        let teban = ban.teban;
        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        for mv in moves2 {
            // println!("{:?}", mv);
            let (x1, y1, x2, y2) = mv;

            let newban = ban.r#move(x1, y1).unwrap();
            let newban2 = newban.r#move(x2, y2).unwrap();
            let teban = ban.teban;
            let teban2 = -teban;
            let nd1 =
                match node.child.iter_mut().find(
                    |a| a.x == x1 && a.y == y1) {
                    None => {
                        node.child.push(NodeBB::new(x1, y1, depth - 1, teban));
                        node.child.last_mut().unwrap()
                    },
                    Some(m) => m
                };

            nd1.child.push(NodeBB::new(x2, y2, depth - 2, teban2));
            let nd2 = nd1.child.last_mut().unwrap();
            // println!("lets think! {}{} {}{}", nd1.x, nd1.y, nd2.x, nd2.y);
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_ab_tt(
                        nd2, &newban2, alpha, beta, wei, &mut tt)
                } else {
                    NodeBB::think_internal_ab(
                        // &mut node.child[idx], &newban2, alpha, beta)
                        nd2, &newban2, -beta, -alpha, wei)
                        // &mut nd2, &newban2, alpha, beta)
                };
            let val = if teban == bitboard::GOTE {val} else {-val};

            node.kyokumen += nd2.kyokumen;
            nd2.hyoka = Some(val);
            let best = nd1.best.as_ref();
            let teban2 = newban.teban;
            let fteban2 = teban2 as f32;
            // println!("val:{}, a:{}, b:{}", val, alpha, beta);
            if teban2 == board::SENTE && alpha < val {
                alpha = val;
            } else if teban2 == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none()
                || best.unwrap().hyoka * fteban2 < val * fteban2 {
                nd1.asignbest(val, x2, y2);
            } else {
                nd2.release();
                continue;
            }
            let best = node.best.as_ref();
            let fteban = teban as f32;
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none()
                || best.unwrap().hyoka * fteban < val * fteban {
                node.asignbest(val, x1, y1);
            } else {
                // nd2.release();
            }
            if alpha > beta {break;}
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut subresult = rx.recv().unwrap();
        println!("node:{}", node.dump());
        println!("subr:{}", subresult.dump());
        if subresult.best.is_none() {
            node.kyokumen += subresult.kyokumen;
            // println!("node:{}", node.dump());
            return Some((node.best.as_ref().unwrap().hyoka, node));
        }
        let nb = node.best.as_ref().unwrap();
        let sb = subresult.best.as_ref().unwrap();
        let fteban = teban as f32;
        if nb.x == sb.x && nb.y == sb.y {
            if nb.hyoka * fteban < sb.hyoka * fteban {
                node.kyokumen += subresult.kyokumen;
                // println!("node:{}", node.dump());
                return Some((node.best.as_ref().unwrap().hyoka, node));
            }
        } else if nb.hyoka * fteban > sb.hyoka * fteban {
            node.kyokumen += subresult.kyokumen;
            // println!("node:{}", node.dump());
            return Some((node.best.as_ref().unwrap().hyoka, node));
        }
        subresult.kyokumen += node.kyokumen;
        // println!("subresult:{}", subresult.dump());
        Some((subresult.best.as_ref().unwrap().hyoka, subresult))
    }

    #[allow(dead_code)]
    pub fn think_ab_extract3(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth, bitboard::NONE);
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let moves1 = ban.genmove()?;

        let mut tt = transptable::TranspositionTable::new();
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }

        let mut moves = Vec::new();
        for &(a, b) in moves1.iter() {
            let ban2 = ban.r#move(a, b).unwrap();
            let m = match ban2.genmove() {
                None => {vec![(0, 0)]},
                Some(mvs) => {mvs}
            };
            for &(c, d) in m.iter() {
                let ban3 = ban2.r#move(c, d).unwrap();
                match ban3.genmove() {
                    None => {
                        moves.push((a, b, c, d, 0, 0));
                    },
                    Some(mvs) => {
                        for &(e, f) in mvs.iter() {
                            moves.push((a, b, c, d, e, f));
                        }
                    },
                }
            }
        }
        // println!("{:?}", moves);

        let n = moves.len();
        // let moves1 = &moves[0..n/2];
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let pa = move_priority3(a);
                let pb = move_priority3(b);
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let wei2 = unsafe{WEIGHT.as_ref().unwrap()};
            let mut node2 = NodeBB::new(0, 0, depth, bitboard::NONE);
            let mut alpha = -100000.0f32;
            let mut beta = 100000.0f32;
            for mv in moves1 {
                let (x1, y1, x2, y2, x3, y3) = mv;

                let newban = ban2.r#move(x1, y1).unwrap();
                let newban2 = newban.r#move(x2, y2).unwrap();
                let newban3 = newban2.r#move(x3, y3).unwrap();
                let teban2 = -teban;
                let teban3 = teban;
                let nd1 = match node2.child.iter_mut().find(
                        |a| a.x == x1 && a.y == y1) {
                    None => {
                        node2.child.push(NodeBB::new(x1, y1, depth - 1, teban));
                        node2.child.last_mut().unwrap()
                    },
                    Some(n) => n,
                };
                let nd2 = match nd1.child.iter_mut().find(
                        |a| a.x == x2 && a.y == y2) {
                    None => {
                        nd1.child.push(NodeBB::new(x2, y2, depth - 2, teban2));
                        nd1.child.last_mut().unwrap()
                    },
                    Some(n) => n,
                };
                nd2.child.push(NodeBB::new(x3, y3, depth - 3, teban3));
                let nd3 = nd2.child.last_mut().unwrap();

                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_ab_tt(
                            nd3, &newban3, alpha, beta, wei2, &mut tt)
                    } else {
                        NodeBB::think_internal_ab(
                            nd3, &newban3, alpha, beta, wei2)
                    };
                let val = if teban == bitboard::SENTE {val} else {-val};

                nd1.hyoka = Some(val);
                nd2.hyoka = Some(val);
                // let nd2 = &nd1.child[0];
                node2.kyokumen += nd3.kyokumen;
                let best = node2.best.as_ref();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none()
                    || best.unwrap().hyoka * fteban < val * fteban {
                    // node2.asignbest(val, x1, y1);
                    node2.best = Some(Best::new(val, x1, y1));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2));
                    nd2.best = Some(Best::new(val, x3, y3));
                } else {
                    nd3.release();
                }
            }
            // tt.dumpsz();
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });
        moves2.sort_by(|a, b| {
            let pa = move_priority3(a);
            let pb = move_priority3(b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha = -100000.0f32;
        let mut beta = 100000.0f32;
        let teban = ban.teban;
        let wei = unsafe{WEIGHT.as_ref().unwrap()};

        for mv in moves2 {
            let (x1, y1, x2, y2, x3, y3) = mv;

            let newban = ban.r#move(x1, y1).unwrap();
            let newban2 = newban.r#move(x2, y2).unwrap();
            let newban3 = newban2.r#move(x3, y3).unwrap();

            let teban2 = -teban;
            let teban3 = teban;
            let nd1 = match node.child.iter_mut().find(
                    |a| a.x == x1 && a.y == y1) {
                None => {
                    node.child.push(NodeBB::new(x1, y1, depth - 1, teban));
                    node.child.last_mut().unwrap()
                },
                Some(n) => n,
            };
            let nd2 = match nd1.child.iter_mut().find(
                    |a| a.x == x2 && a.y == y2) {
                None => {
                    nd1.child.push(NodeBB::new(x2, y2, depth - 2, teban2));
                    nd1.child.last_mut().unwrap()
                },
                Some(n) => n,
            };
            nd2.child.push(NodeBB::new(x3, y3, depth - 3, teban3));
            let nd3 = nd2.child.last_mut().unwrap();

            // println!("lets think! {}{} {}{}", nd1.x, nd1.y, nd2.x, nd2.y);
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_ab_tt(
                        nd3, &newban3, alpha, beta, wei, &mut tt)
                } else {
                    NodeBB::think_internal_ab(
                        // &mut node.child[idx], &newban2, alpha, beta)
                        nd3, &newban3, alpha, beta, wei)
                };
            let val = if teban == bitboard::SENTE {val} else {-val};

            nd1.hyoka = Some(val);
            nd2.hyoka = Some(val);
            node.kyokumen += nd3.kyokumen;
            let best = node.best.as_ref();
            let fteban = teban as f32;
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none()
                || best.unwrap().hyoka * fteban < val * fteban {
                // node.asignbest(val, x1, y1);
                node.best = Some(Best::new(val, x1, y1));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2));
                nd2.best = Some(Best::new(val, x3, y3));
            } else {
                nd3.release();
            }
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut subresult = rx.recv().unwrap();
        println!("node:{}", node.dump());
        println!("subr:{}", subresult.dump());
        if subresult.best.is_none() ||
            node.best.as_ref().unwrap().hyoka * teban as f32
                > subresult.best.as_ref().unwrap().hyoka * teban as f32 {
            node.kyokumen += subresult.kyokumen;
            // println!("node:{}", node.dump());
            return Some((node.best.as_ref().unwrap().hyoka, node));
        }
        subresult.kyokumen += node.kyokumen;
        // println!("subresult:{}", subresult.dump());
        Some((subresult.best.as_ref().unwrap().hyoka, subresult))
    }

    #[allow(dead_code)]
    pub fn think_internal_ab_tt(node:&mut NodeBB, ban : &bitboard::BitBoard, alpha : f32, beta : f32,
            wei : &weight::Weight, tt : &mut transptable::TranspositionTable) -> f32 {
        let mut newalpha = alpha;
        let depth = node.depth;
        if ban.nblank() == 0 {
            return ban.countf32();
        }
        if ban.is_passpass() {
            return -ban.countf32();
        }
        if depth == 0 {
            return -NodeBB::evalwtt(ban, wei, tt);
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            panic!("moves.is_none() nblank == 0 should work!");
            // node.kyokumen = 1;
            // return Some(ban.countf32());
        }
        let mut moves = moves.unwrap();
        moves.sort_by(|a, b| {
            let pa = move_priority(a);
            let pb = move_priority(b);
            pa.partial_cmp(&pb).unwrap()
        });

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let ch = &mut node.child[idx];
            let val = -NodeBB::think_internal_ab_tt(
                ch, &newban, -beta, -newalpha, wei, tt);
            ch.hyoka = Some(val);
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            if newalpha < val {
                newalpha = val;

                node.best = Some(Best::new(val, mvx, mvy));
            } else if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy));
            } else {
                ch.release();
            }
            if newalpha >= beta {
                // cut
                return newalpha;
            }
        }
        newalpha
    }

    pub fn think_internal_ab(node : &mut NodeBB, ban : &bitboard::BitBoard,
            alpha : f32, beta : f32, wei : &weight::Weight) -> f32 {
        let mut newalpha = alpha;
        let depth = node.depth;
        let teban = ban.teban;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            panic!("moves.is_none() nblank == 0 should work!");
            // return -ban.countf32();
        }
        let mut moves = moves.unwrap();
        if moves.len() > 1 {
            // shallow search for move ordering.
            let fteban = teban as f32;
            let mut aval = moves.iter().enumerate().map(|(i, &(x, y))| {
                const D : u8 = 6;
                if depth < D {  // depth:1
                    let newban = ban.r#move(x, y).unwrap();
                    (i, NodeBB::evaluate(&newban, wei) * fteban)
                } else {  // depth:2
                    let newban = ban.r#move(x, y).unwrap();
                    (i,
                    match newban.genmove() {
                        None => {
                            newban.countf32() * fteban
                        },
                        Some(mvs) => {
                            mvs.iter().map(|&(x, y)| {
                                    let newban = newban.r#move(x, y).unwrap();
                                    NodeBB::evaluate(&newban, wei) * fteban
                                }
                            ).collect::<Vec<_>>().into_iter().reduce(f32::min).unwrap()
                        },
                    })
                }
            }).collect::<Vec<_>>();
            aval.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            moves = aval.iter().map(|(i, _val)| moves[*i]).collect::<Vec<_>>();
        }

        let fteban = teban as f32;
        node.child.reserve(moves.len());
        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let ch = node.child.last_mut().unwrap();
            let val =
                if newban.nblank() == 0 || newban.is_passpass() || depth <= 1 {
                    NodeBB::evaluate(&newban, wei) * fteban
                } else {
                    -NodeBB::think_internal_ab(ch, &newban, -beta, -newalpha, wei)
                };
            ch.hyoka = Some(val);
            node.kyokumen += ch.kyokumen;
            if newalpha < val {
                newalpha = val;
                node.best = Some(Best::new(val, mvx, mvy));
            } else if node.best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy));
            } else {
                ch.release();
            }
            if newalpha >= beta {
                // cut
                return newalpha;
            }
        }
        newalpha
    }

    #[allow(dead_code)]
    pub fn vb_think_ab(ban : &bitboard::BitBoard, depth : u8)
            -> Option<(f32, NodeBB)> {
        let wei = unsafe{WEIGHT.as_ref().unwrap()};
        let mut node = NodeBB::new(0, 0, depth, bitboard::NONE);
        if depth == 0 {
            return None;
        }
        if ban.is_passpass() {
            return None;
        }
        // let sum = 0;
        // no more empty cells
        let mut moves = ban.genmove()?;

        moves.sort_by(|a, b| {
            let pa = move_priority(a);
            let pb = move_priority(b);
            pa.partial_cmp(&pb).unwrap()
        });

        let mut alpha = -100000.0f32;
        let mut beta = 100000.0f32;
        let teban = ban.teban;
        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let val = NodeBB::vb_think_internal_ab(
                &mut node.child[idx], &newban, alpha, beta, wei);
    println!("({mvx},{mvy})@{} {:?}", depth - 1, val);
            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() || best.unwrap().hyoka * fteban < val * fteban {
                node.asignbest(val, mvx, mvy);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some((node.best.as_ref().unwrap().hyoka, node))
    }

    #[allow(dead_code)]
    pub fn vb_think_internal_ab(node:&mut NodeBB, ban : &bitboard::BitBoard,
            alpha : f32, beta : f32, wei : &weight::Weight) -> Option<f32> {
        let mut newalpha = alpha;
        let depth = node.depth;
        if depth == 0 {
            println!("depth zero");
            return Some(NodeBB::evaluate(ban, wei));
            // return Some(NodeBB::evalwtt(&ban));
        }
        if ban.is_passpass() {
            println!("pass pass");
            return Some(ban.countf32());
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            println!("no more empty cells");
            return Some(ban.countf32());
        }
        let mut moves = moves.unwrap();
        moves.sort_by(|a, b| {
            let pa = move_priority(a);
            let pb = move_priority(b);
            pa.partial_cmp(&pb).unwrap()
        });

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1, teban));
            let val = NodeBB::vb_think_internal_ab(
                &mut node.child[idx], &newban, -beta, -alpha, wei);
    println!("({mvx},{mvy})@{} {:?} {}", depth-1, val, ban.to_str());
            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            let fteban = teban as f32;
            if best.is_none()
                || best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy));
                continue;
            }
            if newalpha >= beta {
                // cut
                return Some(node.best.as_ref().unwrap().hyoka);
            }
            node.child[idx].release();
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    fn release(&mut self) {
        self.child.clear();
    }

    pub fn to_xy(&self) -> String {
        format!("{}{}",
            if self.teban == board::SENTE {
                board::STR_SENTE
            } else {
                board::STR_GOTE
            }.chars().nth(self.x as usize).unwrap(),
            self.y)
    }

    pub fn bestorder(&self) -> String {
        let mut ret = String::default();
        let mut n = self;
        loop {
            if n.best.is_none() {break;}

            let best = n.best.as_ref().unwrap();
            let x = best.x;
            let y = best.y;
            if n.child.len() == 1 {
                n = &n.child[0];
            } else {
                let m = n.child.iter().find(|&a| a.x == x && a.y == y);
                if m.is_none() {
                    return ret;
                }
                n = m.unwrap();
            }
            ret += &n.to_xy();
        }
        ret
    }

    pub fn dump(&self) -> String {
        format!("{} nodes. ", self.kyokumen) + &self.bestorder()
    }

    #[allow(dead_code)]
    pub fn dumpv(&self) -> String {
        format!("val:{:?}, {} nodes. ", self.hyoka, self.kyokumen)
            + &self.bestorder()
    }

    pub fn dumptree(&self, offset : usize, path : &str) -> Result<(), std::io::Error>{
        // let mut file = std::fs::File::open(path)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(
        format!("@startmindmap\n*{} root\n{}\n@endmindmap\n",
            "*".repeat(offset),
            &self.dumptree_sub(offset + 2)).as_bytes())?;

        Ok(())
    }

    pub fn dumptree_sub(&self, offset : usize) -> String {
        let mut ret = String::default();

        let x;let y;
        if let Some(best) = self.best.as_ref() {
            x = best.x;
            y = best.y;
        } else {
            x = 99;
            y = 99;
        }
        for ch in self.child.iter() {
            let best = if ch.x == x && ch.y == y {
                "!"
            } else { "" };
            ret += &format!("{} {}{} {:.1}\n",
                "*".repeat(offset), best, &ch.to_xy(), ch.hyoka.unwrap_or(-99f32));

            if !ch.child.is_empty() {
                ret += &ch.dumptree_sub(offset + 1);
            }
        }
        ret
    }
}

#[test]
fn test_nodebb() {
    let nodede = NodeBB::new(6, 5, 4, bitboard::NONE);
    let nodefg = NodeBB::new(8, 7, 4, bitboard::NONE);

    let mut nodebc = NodeBB::new(4, 3, 5, bitboard::NONE);
    nodebc.kyokumen = 3210;
    assert_eq!(nodebc.dumpv(), "val:None, 3210 nodes. ");

    let mut node9a = NodeBB::new(2, 1, 5, bitboard::SENTE);
    node9a.kyokumen = 4321;
    node9a.hyoka = Some(99.9);
    node9a.best = Some(Best::new(99.9, 8, 7));
    node9a.child.push(nodede);
    node9a.child.push(nodefg);
    assert_eq!(node9a.dumpv(), "val:Some(99.9), 4321 nodes. h7");

    let mut node56 = NodeBB::new(5, 6, 6, bitboard::NONE);
    node56.kyokumen = 6543;
    assert_eq!(node56.dumpv(), "val:None, 6543 nodes. ");

    let mut node78 = NodeBB::new(7, 8, 6, bitboard::NONE);
    node78.kyokumen = 5432;
    node78.hyoka = Some(99.9);
    node78.best = Some(Best::new(99.9, 2, 1));
    node78.child.push(nodebc);
    node78.child.push(node9a);
    assert_eq!(node78.dumpv(), "val:Some(99.9), 5432 nodes. B1h7");

    let mut node12 = NodeBB::new(1, 2, 7, bitboard::SENTE);
    node12.kyokumen = 8765;
    node12.child.push(node56);
    node12.hyoka = Some(99.9);
    node12.best = Some(Best::new(99.9, 7, 8));
    node12.child.push(node78);
    assert_eq!(node12.dumpv(), "val:Some(99.9), 8765 nodes. g8B1h7");

    let mut node34 = NodeBB::new(3, 4, 7, bitboard::GOTE);
    node34.kyokumen = 7654;
    assert_eq!(node34.dumpv(), "val:None, 7654 nodes. ");

    let mut node = NodeBB::new(99, 2, 8, bitboard::NONE);
    node.hyoka = Some(99.9);
    node.kyokumen = 9876;
    node.best = Some(Best::new(99.9, 1, 2));
    node.child.push(node12);
    node.child.push(node34);
    assert_eq!(node.dumpv(), "val:Some(99.9), 9876 nodes. A2g8B1h7");
}
