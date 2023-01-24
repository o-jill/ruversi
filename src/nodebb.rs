use super::*;
use std::sync::Arc;

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

pub struct NodeBB {
    child : Vec<NodeBB>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    pub best : Option<Best>,
    pub x : u8,
    pub y : u8,
    depth : u8,
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
        ND_ROOT = Some(NodeBB::new(0, 0, 0));
        INITIALIZED = true;
    }
}

impl NodeBB {
    pub fn new(x : u8, y : u8, depth : u8) -> NodeBB {
        NodeBB {
            child : Vec::<NodeBB>::new(),
            hyoka : None,
            kyokumen : 0,
            best : None,
            x : x,
            y : y,
            depth : depth,
        }
    }

    fn evaluate(ban : &bitboard::BitBoard) -> f32 {
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
        tt.check_or_append(&id, || NodeBB::evaluate(ban))
    }

    pub fn thinko(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, &NodeBB)> {
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

        let node;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth));
            node = ND_ROOT.as_mut().unwrap();
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            node.depth += 1;
            depth += 1;
        }
        let n = moves.len();
        for (mvx, mvy) in moves.iter() {
            node.child.push(NodeBB::new(*mvx, *mvy, depth - 1));
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
                let mut nd = nd.unwrap();
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let val = NodeBB::think_internal(nd, &newban);
                nd.hyoka = val;
            }
            });

        for (mvx, mvy) in moves2 {
            let nd = node.child.iter_mut().find(|a|
                    a.x == mvx && a.y == mvy
                ).unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let val = NodeBB::think_internal(nd, &newban);

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
                be = Some(Best::new(hyoka.unwrap(), c.x, c.y, teban));
            }
        }
        node.hyoka = hyoka;
        node.best = be;
        node.kyokumen = km;
        Some((hyoka.unwrap(), node))
    }

    pub fn think(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth);
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

        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            node.depth += 1;
            depth += 1;
        }
        let n = moves.len();
        // let moves1 = &moves[0..n/2];
        let moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            let mut node2 = NodeBB::new(0, 0, depth);
            let teban = ban2.teban;
            let mut tt = transptable::TranspositionTable::new();
            for (mvx, mvy) in moves1 {
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let idx = node2.child.len();
                node2.child.push(NodeBB::new(mvx, mvy, depth - 1));
                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_tt(
                            &mut node2.child[idx], &newban, &mut tt)
                    } else {
                        NodeBB::think_internal(
                            &mut node2.child[idx], &newban)
                    };
                let mut ch = &mut node2.child[idx];
                ch.hyoka = val;
                node2.kyokumen += ch.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if best.is_none() {
                    node2.best = Some(Best::new(val, mvx, mvy, teban));
                    node2.hyoka = Some(val);
                } else if fteban * best.unwrap().hyoka < fteban * val {
                    node2.best = Some(Best::new(val, mvx, mvy, teban));
                    node2.hyoka = Some(val);
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
        for (mvx, mvy) in moves2 {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_tt(
                        &mut node.child[idx], &newban, &mut tt)
                } else {
                    NodeBB::think_internal(
                        &mut node.child[idx], &newban)
                };

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                node.hyoka = Some(val);
                // println!("best : {}", val);
            } else if fteban * best.unwrap().hyoka < fteban * val {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                node.hyoka = Some(val);
                // println!("best : -> {}", val);
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

    pub fn think_internal(node:&mut NodeBB, ban : &bitboard::BitBoard)
            -> Option<f32> {
        let mut depth = node.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            node.kyokumen = 1;
            return Some(NodeBB::evaluate(&ban));
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
        }

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let val = NodeBB::think_internal(
                &mut node.child[idx], &newban);

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy, teban));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    pub fn think_internal_tt(node:&mut NodeBB, ban : &bitboard::BitBoard,
        tt : &mut transptable::TranspositionTable) -> Option<f32> {
        let mut depth = node.depth;
        if depth == 0 {
            node.kyokumen = 1;
            return Some(NodeBB::evalwtt(&ban, tt));
        }
        if ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
        }

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let val = NodeBB::think_internal_tt(
                &mut node.child[idx], &newban, tt);

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy, teban));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    pub fn think_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth);
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
        let mut tt = transptable::TranspositionTable::new();
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
            node.depth += 1;
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
        // let moves1 = &moves[0..n/2];
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let pa = move_priority(&a);
                let pb = move_priority(&b);
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut node2 = NodeBB::new(0, 0, depth);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for (mvx, mvy) in moves1 {
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let idx = node2.child.len();
                node2.child.push(NodeBB::new(mvx, mvy, depth - 1));
                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_ab_tt(
                            &mut node2.child[idx], &newban, alpha, beta, &mut tt)
                    } else {
                        NodeBB::think_internal_ab(
                            &mut node2.child[idx], &newban, alpha, beta)
                    };

                let mut ch = &mut node2.child[idx];
                ch.hyoka = val;
                node2.kyokumen += ch.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none() {
                    node2.best = Some(Best::new(val, mvx, mvy, teban));
                    node2.hyoka = Some(val);
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    node2.best = Some(Best::new(val, mvx, mvy, teban));
                    node2.hyoka = Some(val);
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
            let pa = move_priority(&a);
            let pb = move_priority(&b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for (mvx, mvy) in moves2 {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_ab_tt(
                        &mut node.child[idx], &newban, alpha, beta, &mut tt)
                } else {
                    NodeBB::think_internal_ab(
                        &mut node.child[idx], &newban, alpha, beta)
                };

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            // println!("val:{}, a:{}, b:{}", val, alpha, beta);
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                node.hyoka = Some(val);
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

    pub fn thinko_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, &NodeBB)> {
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

        let node;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth));
            node = ND_ROOT.as_mut().unwrap();
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            node.depth += 1;
            depth += 1;
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
        for (mvx, mvy) in moves.iter() {
            node.child.push(NodeBB::new(*mvx, *mvy, depth - 1));
        }
        // let moves1 = &moves[0..n/2];
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();

        let salpha = Arc::new(std::sync::Mutex::new(-100000.0 as f32));
        let sbeta = Arc::new(std::sync::Mutex::new(100000.0 as f32));
        let sal = salpha.clone();
        let sbe = sbeta.clone();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let pa = move_priority(&a);
                let pb = move_priority(&b);
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
                let mut nd = nd.unwrap();
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta);
                nd.hyoka = val;
                let val = val.unwrap();
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
            let pa = move_priority(&a);
            let pb = move_priority(&b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        // let mut alpha : f32 = *salpha.lock().unwrap();
        // let mut beta : f32 = *sbeta.lock().unwrap();
        let teban = ban.teban;
        for (mvx, mvy) in moves2 {
            let nd = node.child.iter_mut().find(|a| {
                    a.x == mvx && a.y == mvy
                });
            // if nd.is_none() {
            //     panic!("node2.child.iter_mut().find(|a|");
            // }
            let mut nd = nd.unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta);
            nd.hyoka = val;
            let val = val.unwrap();
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
        let mut hyoka = None;
        let mut km = 0;
        let teban = ban.teban;
        let fteban = teban as f32;
        let mut bx = 0;
        let mut by = 0;
        for c in node.child.iter() {
            km += c.kyokumen;
            if c.hyoka.is_none() {
                continue;
            }
            if hyoka.is_none() || hyoka.unwrap() * fteban < c.hyoka.unwrap() * fteban {
                hyoka = c.hyoka;
                bx = c.x;
                by = c.y;
            }
        }
        node.hyoka = hyoka;
        node.best = Some(Best::new(hyoka.unwrap(), bx, by, teban));
        node.kyokumen = km;
        Some((hyoka.unwrap(), node))
    }

    pub fn thinko_ab_extract2(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, &NodeBB)> {
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

        let node;
        unsafe {
            ND_ROOT = Some(NodeBB::new(0, 0, depth));
            node = ND_ROOT.as_mut().unwrap();
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            node.depth += 1;
            depth += 1;
        }
        let yomikiri = 12;
        let yose = 18;
        let nblank = ban.nblank();
        if nblank <= yomikiri {
            depth = yomikiri as u8;
        } else if nblank <= yose {
            depth += 2;
        }
        for (mvx, mvy) in moves.iter() {
            node.child.push(NodeBB::new(*mvx, *mvy, depth - 1));
        }

        let mut moves4 = Vec::<(u8, u8, u8, u8)>::new();
        for (mvx, mvy) in moves {
            let nd = node.child.iter_mut().find(|a| {
                    a.x == mvx && a.y == mvy
                });
            // if nd.is_none() {
            //     panic!("node2.child.iter_mut().find(|a|");
            // }
            let nd = nd.unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let moves = newban.genmove();
            if moves.is_none() {
                nd.child.push(NodeBB::new(0, 0, depth - 1));
                moves4.push((mvx, mvy, 0, 0));
                continue;
            }
            for (mvx2, mvy2) in moves.unwrap() {
                nd.child.push(NodeBB::new(mvx2, mvy2, depth - 2));
                moves4.push((mvx, mvy, mvx2, mvy2));
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

        // let salpha = Arc::new(std::sync::Mutex::new(-100000.0 as f32));
        // let sbeta = Arc::new(std::sync::Mutex::new(100000.0 as f32));
        // let sal = salpha.clone();
        // let sbe = sbeta.clone();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let pa = move_priority2(&a);
                let pb = move_priority2(&b);
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
                let mut nd = nd2.unwrap();
                let newban = ban2.r#move(mvx, mvy).unwrap();
                let newban = newban.r#move(mvx2, mvy2).unwrap();
                let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta);
                nd.hyoka = val;
                let val = val.unwrap();
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
            let pa = move_priority2(&a);
            let pb = move_priority2(&b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        // let mut alpha : f32 = *salpha.lock().unwrap();
        // let mut beta : f32 = *sbeta.lock().unwrap();
        let teban = -ban.teban;
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
            let mut nd = nd2.unwrap();
            let newban = ban.r#move(mvx, mvy).unwrap();
            let newban = newban.r#move(mvx2, mvy2).unwrap();
            let val = NodeBB::think_internal_ab(nd, &newban, alpha, beta);
            nd.hyoka = val;
            let val = val.unwrap();
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
            let mut hyoka = None;
            let mut be = None;
            let mut km2 = 0;
            let teban2 = -teban;
            let fteban2 = teban2 as f32;
            for c2 in c.child.iter() {
                km2 += c2.kyokumen;
                if c2.hyoka.is_none() {
                    continue;
                }
                if hyoka.is_none() {
                    hyoka = c2.hyoka;
                    be = Some(Best::new(hyoka.unwrap(), c2.x, c2.y, teban2));
                    continue;
                }
                if hyoka.unwrap() * fteban2 < c2.hyoka.unwrap() * fteban2 {
                    hyoka = c2.hyoka;
                    let best = be.as_mut().unwrap();
                    best.x = c2.x;
                    best.y = c2.y;
                    best.hyoka = hyoka.unwrap();
                }
            }
            c.hyoka = hyoka;
            c.best = be;
            c.kyokumen = km;
            km += km2;
        }

        let fteban = teban as f32;
        let mut hyoka = None;
        let mut be = None;
        for c in node.child.iter() {
            if c.hyoka.is_none() {
                continue;
            }
            if hyoka.is_none() {
                hyoka = c.hyoka;
                be = Some(Best::new(hyoka.unwrap(), c.x, c.y, teban));
                continue;
            }
            if hyoka.unwrap() * fteban < c.hyoka.unwrap() * fteban {
                hyoka = c.hyoka;
                let best = be.as_mut().unwrap();
                best.x = c.x;
                best.y = c.y;
                best.hyoka = hyoka.unwrap();
            }
        }
        node.hyoka = hyoka;
        node.best = be;
        node.kyokumen = km;
        Some((hyoka.unwrap(), node))
    }

    pub fn think_ab_extract2(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth);
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
        let mut tt = transptable::TranspositionTable::new();
        let mut moves1 = moves.unwrap();
        if moves1.len() == 0 {  // pass
            moves1.push((0, 0));
            depth += 1;
            node.depth += 1;
        }
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
                    if mvs.is_empty() {
                        moves.push((a, b, 0, 0));
                    } else {
                        for &(c, d) in mvs.iter() {
                            moves.push((a, b, c, d));
                        }
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
                let pa = move_priority2(&a);
                let pb = move_priority2(&b);
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut node2 = NodeBB::new(0, 0, depth);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for mv in moves1 {
                let (x1, y1, x2, y2) = mv;

                let newban = ban2.r#move(x1, y1).unwrap();
                let newban2 = newban.r#move(x2, y2).unwrap();

                let nd1 =
                    match node2.child.iter_mut().find(
                        |a| a.x == x1 && a.y == y1) {
                        None => {
                            node2.child.push(NodeBB::new(x1, y1, depth - 1));
                            node2.child.last_mut().unwrap()
                        },
                        Some(m) => m
                    };
                nd1.child.push(NodeBB::new(x2, y2, depth - 2));
                let mut nd2 = nd1.child.last_mut().unwrap();

                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_ab_tt(
                            &mut nd2, &newban2, alpha, beta, &mut tt)
                    } else {
                        NodeBB::think_internal_ab(
                            &mut nd2, &newban2, -beta, -alpha)
                            // &mut nd2, &newban2, alpha, beta)
                    };

                node2.kyokumen += nd2.kyokumen;
        if true {  // ---------------
                nd2.hyoka = val;
                let best = nd1.best.as_ref();
                let val = val.unwrap();
                let teban2 = newban.teban;
                let fteban2 = teban2 as f32;
                println!("val:{}, a:{}, b:{}", val, alpha, beta);
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
                if best.is_none() {
                    nd1.best = Some(Best::new(val, x2, y2, teban2));
                    nd1.hyoka = Some(val);
                } else if best.unwrap().hyoka * fteban2 < val * fteban2 {
                    nd1.best = Some(Best::new(val, x2, y2, teban2));
                    nd1.hyoka = Some(val);
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
                if best.is_none() {
                    node2.best = Some(Best::new(val, x1, y1, teban));
                    node2.hyoka = Some(val);
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    node2.best = Some(Best::new(val, x1, y1, teban));
                    node2.hyoka = Some(val);
                } else {
                    // nd2.release();
                }
                // if alpha >= beta {break;}
        } else {
                nd1.hyoka = val;

                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none() {
                    node2.best = Some(Best::new(val, x1, y1, teban));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2, -teban));
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    node2.best = Some(Best::new(val, x1, y1, teban));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2, -teban));
                } else {
                    nd2.release();
                }
        }
            }
            // tt.dumpsz();
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        moves2.sort_by(|a, b| {
            let pa = move_priority2(&a);
            let pb = move_priority2(&b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for mv in moves2 {
            // println!("{:?}", mv);
            let (x1, y1, x2, y2) = mv;

            let newban = ban.r#move(x1, y1).unwrap();
            let newban2 = newban.r#move(x2, y2).unwrap();

            let nd1 =
                match node.child.iter_mut().find(
                    |a| a.x == x1 && a.y == y1) {
                    None => {
                        node.child.push(NodeBB::new(x1, y1, depth - 1));
                        node.child.last_mut().unwrap()
                    },
                    Some(m) => m
                };

            nd1.child.push(NodeBB::new(x2, y2, depth - 2));
            let mut nd2 = nd1.child.last_mut().unwrap();
            // println!("lets think! {}{} {}{}", nd1.x, nd1.y, nd2.x, nd2.y);
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_ab_tt(
                        &mut nd2, &newban2, alpha, beta, &mut tt)
                } else {
                    NodeBB::think_internal_ab(
                        // &mut node.child[idx], &newban2, alpha, beta)
                        &mut nd2, &newban2, -beta, -alpha)
                        // &mut nd2, &newban2, alpha, beta)
                };

            node.kyokumen += nd2.kyokumen;
if true {  // ---------------
            nd2.hyoka = val;
            let best = nd1.best.as_ref();
            let val = val.unwrap();
            let teban2 = newban.teban;
            let fteban2 = teban2 as f32;
            // println!("val:{}, a:{}, b:{}", val, alpha, beta);
            if teban2 == board::SENTE && alpha < val {
                alpha = val;
            } else if teban2 == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() {
                nd1.best = Some(Best::new(val, x2, y2, teban2));
                nd1.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban2 < val * fteban2 {
                nd1.best = Some(Best::new(val, x2, y2, teban2));
                nd1.hyoka = Some(val);
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
            if best.is_none() {
                node.best = Some(Best::new(val, x1, y1, teban));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x1, y1, teban));
                node.hyoka = Some(val);
            } else {
                // nd2.release();
            }
            if alpha > beta {break;}
} else {
            nd1.hyoka = val;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            println!("val:{}, a:{}, b:{}", val, alpha, beta);
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x1, y1, teban));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2, -teban));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x1, y1, teban));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2, -teban));
            } else {
                nd2.release();
            }
}
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

    pub fn think_ab_extract3(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth);
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
        let mut tt = transptable::TranspositionTable::new();
        let mut moves1 = moves.unwrap();
        if moves1.is_empty() {  // pass
            moves1.push((0, 0));
            depth += 1;
            node.depth += 1;
        }
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
                None => {
                    vec![(0, 0)]
                },
                Some(mvs) => {
                    if mvs.is_empty() {
                        vec![(0, 0)]
                    } else {
                        mvs
                    }
                }
            };
            for &(c, d) in m.iter() {
                let ban3 = ban2.r#move(c, d).unwrap();
                match ban3.genmove() {
                    None => {
                        moves.push((a, b, c, d, 0, 0));
                    },
                    Some(mvs) => {
                        if mvs.is_empty() {
                            moves.push((a, b, c, d, 0, 0));
                        } else {
                            for &(e, f) in mvs.iter() {
                                moves.push((a, b, c, d, e, f));
                            }
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
                let pa = move_priority3(&a);
                let pb = move_priority3(&b);
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut node2 = NodeBB::new(0, 0, depth);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for mv in moves1 {
                let (x1, y1, x2, y2, x3, y3) = mv;

                let newban = ban2.r#move(x1, y1).unwrap();
                let newban2 = newban.r#move(x2, y2).unwrap();
                let newban3 = newban2.r#move(x3, y3).unwrap();

                let mut nd1 = match node2.child.iter_mut().find(
                        |a| a.x == x1 && a.y == y1) {
                    None => {
                        node2.child.push(NodeBB::new(x1, y1, depth - 1));
                        node2.child.last_mut().unwrap()
                    },
                    Some(n) => n,
                };
                let mut nd2 = match nd1.child.iter_mut().find(
                        |a| a.x == x2 && a.y == y2) {
                    None => {
                        nd1.child.push(NodeBB::new(x2, y2, depth - 2));
                        nd1.child.last_mut().unwrap()
                    },
                    Some(n) => n,
                };
                nd2.child.push(NodeBB::new(x3, y3, depth - 3));
                let mut nd3 = nd2.child.last_mut().unwrap();

                let val = if cfg!(feature="withtt") {
                        NodeBB::think_internal_ab_tt(
                            &mut nd3, &newban3, alpha, beta, &mut tt)
                    } else {
                        NodeBB::think_internal_ab(
                            &mut nd3, &newban3, alpha, beta)
                    };

                nd1.hyoka = val;
                nd2.hyoka = val;
                // let nd2 = &nd1.child[0];
                node2.kyokumen += nd3.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none() {
                    node2.best = Some(Best::new(val, x1, y1, teban));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2, -teban));
                    nd2.best = Some(Best::new(val, x3, y3, teban));
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    // print!("UPDT {} -> ", node2.dump());
                    node2.best = Some(Best::new(val, x1, y1, teban));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2, -teban));
                    nd2.best = Some(Best::new(val, x3, y3, teban));
                } else {
                    nd3.release();
                }
            }
            // tt.dumpsz();
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });
        moves2.sort_by(|a, b| {
            let pa = move_priority3(&a);
            let pb = move_priority3(&b);
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;

        for mv in moves2 {
            let (x1, y1, x2, y2, x3, y3) = mv;

            let newban = ban.r#move(x1, y1).unwrap();
            let newban2 = newban.r#move(x2, y2).unwrap();
            let newban3 = newban2.r#move(x3, y3).unwrap();

            let mut nd1 = match node.child.iter_mut().find(
                    |a| a.x == x1 && a.y == y1) {
                None => {
                    node.child.push(NodeBB::new(x1, y1, depth - 1));
                    node.child.last_mut().unwrap()
                },
                Some(n) => n,
            };
            let mut nd2 = match nd1.child.iter_mut().find(
                    |a| a.x == x2 && a.y == y2) {
                None => {
                    nd1.child.push(NodeBB::new(x2, y2, depth - 2));
                    nd1.child.last_mut().unwrap()
                },
                Some(n) => n,
            };
            nd2.child.push(NodeBB::new(x3, y3, depth - 3));
            let mut nd3 = nd2.child.last_mut().unwrap();

            // println!("lets think! {}{} {}{}", nd1.x, nd1.y, nd2.x, nd2.y);
            let val = if cfg!(feature="withtt") {
                    NodeBB::think_internal_ab_tt(
                        &mut nd3, &newban3, alpha, beta, &mut tt)
                } else {
                    NodeBB::think_internal_ab(
                        // &mut node.child[idx], &newban2, alpha, beta)
                        &mut nd3, &newban3, alpha, beta)
                };

            nd1.hyoka = val;
            nd2.hyoka = val;
            node.kyokumen += nd3.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x1, y1, teban));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2, -teban));
                nd2.best = Some(Best::new(val, x3, y3, teban));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x1, y1, teban));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2, -teban));
                nd2.best = Some(Best::new(val, x3, y3, teban));
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

    pub fn think_internal_ab_tt(node:&mut NodeBB, ban : &bitboard::BitBoard, alpha : f32, beta : f32,
            tt : &mut transptable::TranspositionTable) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            node.kyokumen = 1;
            return Some(NodeBB::evalwtt(&ban, tt));
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
        } else {
            moves.sort_by(|a, b| {
                let pa = move_priority(&a);
                let pb = move_priority(&b);
                pa.partial_cmp(&pb).unwrap()
            });
        }

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let mut ch = &mut node.child[idx];
            let val = NodeBB::think_internal_ab_tt(
                ch, &newban, -beta, -newalpha, tt);
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy, teban));
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

    pub fn think_internal_ab(node : &mut NodeBB, ban : &bitboard::BitBoard,
            alpha : f32, beta : f32) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        // println!("d:{}",depth);
        if ban.nblank() == 0 || ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            node.kyokumen = 1;
            return Some(NodeBB::evaluate(&ban));
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            moves.push((0, 0));
            depth += 1;
        } else {
            moves.sort_by(|a, b| {
                let pa = move_priority(&a);
                let pb = move_priority(&b);
                pa.partial_cmp(&pb).unwrap()
            });
        }

        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let mut ch = &mut node.child[idx];
            let val = NodeBB::think_internal_ab(ch, &newban, -beta, -newalpha);
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_mut();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                continue;
            }
            let fteban = teban as f32;
            let mut be = best.unwrap();
            if be.hyoka * fteban < val * fteban {
                be.x = mvx;
                be.y = mvy;
                be.hyoka = val;
                continue;
            }
            if newalpha >= beta {
                // cut
                return Some(be.hyoka);
            }
            node.child[idx].release();
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    pub fn vb_think_ab(ban : &bitboard::BitBoard, mut depth : u8)
            -> Option<(f32, NodeBB)> {
        let mut node = NodeBB::new(0, 0, depth);
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
        let mut moves = moves.unwrap();
        // let n = moves.len();
        if moves.len() == 0 {  // pass
            depth += 1;
            moves.push((0, 0));
            println!("pass");
        } else {
            moves.sort_by(|a, b| {
                let pa = move_priority(&a);
                let pb = move_priority(&b);
                pa.partial_cmp(&pb).unwrap()
            });
        }
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let val = NodeBB::vb_think_internal_ab(
                &mut node.child[idx], &newban, alpha, beta);
    println!("({mvx},{mvy})@{} {:?}", depth - 1, val);
            let mut ch = &mut node.child[idx];
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
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                node.hyoka = Some(val);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some((node.best.as_ref().unwrap().hyoka, node))
    }

    pub fn vb_think_internal_ab(node:&mut NodeBB, ban : &bitboard::BitBoard, alpha : f32, beta : f32) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        if depth == 0 {
            println!("depth zero");
            node.kyokumen = 1;
            return Some(NodeBB::evaluate(&ban));
            // return Some(NodeBB::evalwtt(&ban));
        }
        if ban.is_passpass() {
            println!("pass pass");
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        // no more empty cells
        if moves.is_none() {
            println!("no more empty cells");
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        let mut moves = moves.unwrap();
        if moves.len() == 0 {  // pass
            depth += 1;
            moves.push((0, 0));
        } else {
            moves.sort_by(|a, b| {
                let pa = move_priority(&a);
                let pb = move_priority(&b);
                pa.partial_cmp(&pb).unwrap()
            });
        }
        for (mvx, mvy) in moves {
            let newban = ban.r#move(mvx, mvy).unwrap();
            let idx = node.child.len();
            node.child.push(NodeBB::new(mvx, mvy, depth - 1));
            let val = NodeBB::vb_think_internal_ab(
                &mut node.child[idx], &newban, -beta, -alpha);
    println!("({mvx},{mvy})@{} {:?} {}", depth-1, val, ban.to_str());
                let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, mvx, mvy, teban));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, mvx, mvy, teban));
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

    pub fn dump(&self) -> String {
        let mut ret = String::new();
        ret += &format!("val:{:?}, {} nodes. ", self.hyoka, self.kyokumen);
        // ret += &format!("{:?}", self.best);
        // let x = self.best.unwrap().1;
        // let y = self.best.unwrap().2;
        // let n = self.child.iter().find(|&a| a.x == x && a.y == y);
        // ret += &format!("{:?}", n.unwrap().best);
        let mut n = self;
        loop {
            let best = n.best.as_ref().unwrap();
            // ret += &format!("{}", best.to_str());
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
            // ret += &format!("{}", best.pos());
            ret += &best.pos();
            if n.best.is_none() {
                // ret += &format!("best:none {:?}", n.hyoka);
                // if n.child.is_empty() {break;}
                // panic!("n.child is not empty... {} ret:{}", n.child.len(), ret);
                break;
            }
        }
        ret
    }
}
