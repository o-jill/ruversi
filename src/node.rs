use super::*;

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

static mut INITIALIZED : bool = false;

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
// static mut WEIGHT : Option<Vec<f32>> = None;
pub static mut WEIGHT : Option<weight::Weight> = None;

pub struct Best {
    pub hyoka : f32,
    pub x : u8,
    pub y : u8,
}

impl Best {
    pub fn new(h : f32, x : u8, y : u8) -> Best {
        Best { hyoka: h, x: x, y: y }
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

pub struct Node {
    child : Vec<Node>,
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

        INITIALIZED = true;
    }
}

impl Node {
    pub fn new(x : u8, y : u8, depth : u8, t : i8) -> Node {
        Node {
            child : Vec::<Node>::new(),
            hyoka : None,
            kyokumen : 0,
            best : None,
            x : x,
            y : y,
            depth : depth,
            teban : t,
        }
    }

    #[cfg(target_arch="x86_64")]
    fn evaluate(ban : &board::Board) -> f32 {
        unsafe {
            if cfg!(feature="nnv1") {
                if cfg!(feature = "nosimd") {
                    WEIGHT.as_ref().unwrap().evaluatev1(ban)
                } else {
                    WEIGHT.as_ref().unwrap().evaluatev1_simd(ban)
                }
            } else if cfg!(feature="nnv2") {
                if cfg!(feature = "nosimd") {
                    WEIGHT.as_ref().unwrap().evaluatev2(ban)
                } else {
                    WEIGHT.as_ref().unwrap().evaluatev2_simd2(ban)
                }
            } else {
                if cfg!(feature = "nosimd") {
                    WEIGHT.as_ref().unwrap().evaluatev3(ban)
                } else {
                    WEIGHT.as_ref().unwrap().evaluatev3_simd(ban)
                }
            }
        }
    }

    #[cfg(target_arch="aarch64")]
    fn evaluate(ban : &board::Board) -> f32 {
        unsafe {
            if cfg!(feature="nnv1") {
                WEIGHT.as_ref().unwrap().evaluatev1(ban)
            } else if cfg!(feature="nnv2") {
                WEIGHT.as_ref().unwrap().evaluatev2(ban)
            } else {
                WEIGHT.as_ref().unwrap().evaluatev3(ban)
            }
        }
    }

    fn evalwtt(ban : &board::Board, tt : &mut transptable::TranspositionTable) -> f32 {
        let id = if cfg!(feature="nosimd") {ban.to_id()} else {ban.to_id_simd()};
        tt.check_or_append(&id, || Node::evaluate(ban))
    }

    pub fn think(ban : &board::Board, mut depth : u8) -> Option<(f32,Node)> {
        let mut node = node::Node::new(0, 0, depth, board::NONE);
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
            let mut node2 = node::Node::new(0, 0, depth, board::NONE);
            let teban = ban2.teban;
            let mut tt = transptable::TranspositionTable::new();
            for mv in moves1 {
                let x = mv.0;
                let y = mv.1;
                let newban = ban2.r#move(x, y).unwrap();
                let idx = node2.child.len();
                node2.child.push(Node::new(x, y, depth - 1, teban));
                let val = if cfg!(feature="withtt") {
                        Node::think_internal_tt(
                            &mut node2.child[idx], &newban, &mut tt)
                    } else {
                        Node::think_internal(
                            &mut node2.child[idx], &newban)
                    };
                let ch = &mut node2.child[idx];
                ch.hyoka = val;
                node2.kyokumen += ch.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if best.is_none() {
                    node2.best = Some(Best::new(val, x, y));
                    node2.hyoka = Some(val);
                } else if fteban * best.unwrap().hyoka < fteban * val {
                    node2.best = Some(Best::new(val, x, y));
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
        for mv in moves2 {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = if cfg!(feature="withtt") {
                    Node::think_internal_tt(
                        &mut node.child[idx], &newban, &mut tt)
                } else {
                    Node::think_internal(
                        &mut node.child[idx], &newban)
                };

            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
                node.hyoka = Some(val);
                // println!("best : {}", val);
            } else if fteban * best.unwrap().hyoka < fteban * val {
                node.best = Some(Best::new(val, x, y));
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

    pub fn think_internal(node:&mut Node, ban : &board::Board) -> Option<f32> {
        let mut depth = node.depth;
        if depth == 0 {
            node.kyokumen = 1;
            return Some(Node::evaluate(&ban));
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

        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = Node::think_internal(
                &mut node.child[idx], &newban);

            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    pub fn think_internal_tt(node:&mut Node, ban : &board::Board,
        tt : &mut transptable::TranspositionTable) -> Option<f32> {
        let mut depth = node.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            node.kyokumen = 1;
            return Some(Node::evalwtt(&ban, tt));
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

        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = Node::think_internal_tt(
                &mut node.child[idx], &newban, tt);

            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    pub fn think_ab(ban : &board::Board, mut depth : u8) -> Option<(f32,Node)> {
        let mut node = node::Node::new(0, 0, depth, board::NONE);
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
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut node2 = node::Node::new(0, 0, depth, board::NONE);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for mv in moves1 {
                let (x, y) = mv;
                let newban = ban2.r#move(x, y).unwrap();
                let idx = node2.child.len();
                node2.child.push(Node::new(x, y, depth - 1, teban));
                let val = if cfg!(feature="withtt") {
                        Node::think_internal_ab_tt(
                            &mut node2.child[idx], &newban, alpha, beta, &mut tt)
                    } else {
                        Node::think_internal_ab(
                            &mut node2.child[idx], &newban, alpha, beta)
                    };

                let ch = &mut node2.child[idx];
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
                    node2.best = Some(Best::new(val, x, y));
                    node2.hyoka = Some(val);
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    node2.best = Some(Best::new(val, x, y));
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
            let ia = a.0 + a.1 * 8 - 9;
            let ib = b.0 + b.1 * 8 - 9;
            let pa = SORT_PRI[ia as usize];
            let pb = SORT_PRI[ib as usize];
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for mv in moves2 {
            let (x, y) = mv;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = if cfg!(feature="withtt") {
                    Node::think_internal_ab_tt(
                        &mut node.child[idx], &newban, alpha, beta, &mut tt)
                } else {
                    Node::think_internal_ab(
                        &mut node.child[idx], &newban, alpha, beta)
                };

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
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
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

    #[allow(dead_code)]
    pub fn think_ab_extract2(ban : &board::Board, mut depth : u8)
            -> Option<(f32, Node)> {
        let mut node = Node::new(0, 0, depth, board::NONE);
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

        let n = moves.len();
        let mut moves1 = Vec::from_iter(moves[0..n/2].iter().cloned());
        let mut moves2 = Vec::from_iter(moves[n/2..].iter().cloned());
        let ban2 = ban.clone();
        let (tx, rx) = mpsc::channel();

        let sub =
                thread::spawn(move || {
            moves1.sort_by(|a, b| {
                let ia1 = if a.0 == 0 { 0 } else { a.0 + a.1 * 8 - 9 };
                let ia2 = if a.2 == 0 { 0 } else { a.2 + a.3 * 8 - 9 };
                let ib1 = if b.0 == 0 { 0 } else { b.0 + b.1 * 8 - 9 };
                let ib2 = if b.2 == 0 { 0 } else { b.2 + b.3 * 8 - 9 };
                let pa = SORT_PRI[ia1 as usize] * 10 + SORT_PRI[ia2 as usize];
                let pb = SORT_PRI[ib1 as usize] * 10 + SORT_PRI[ib2 as usize];
                pa.partial_cmp(&pb).unwrap()
            });
            let mut tt = transptable::TranspositionTable::new();
            let teban = ban2.teban;
            let mut node2 = Node::new(0, 0, depth, board::NONE);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for mv in moves1 {
                let (x1, y1, x2, y2) = mv;
                let newban = ban2.r#move(x1, y1).unwrap();
                let newban2 = newban.r#move(x2, y2).unwrap();

                let nd1 = match node2.child.iter_mut().find(
                        |a| a.x == x1 && a.y == y1) {
                        None => {
                            node2.child.push(Node::new(x1, y1, depth - 1, teban));
                            node2.child.last_mut().unwrap()
                        },
                        Some(n) => n,
                    };
                nd1.child.push(Node::new(x2, y2, depth - 2, -teban));
                let mut nd2 = nd1.child.last_mut().unwrap();

                let val = if cfg!(feature="withtt") {
                        Node::think_internal_ab_tt(
                            &mut nd2,
                            &newban2, alpha, beta, &mut tt)
                    } else {
                        Node::think_internal_ab(
                            &mut nd2,
                            &newban2, alpha, beta)
                    };

                nd1.hyoka = val;
                node2.kyokumen += nd2.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if teban == board::SENTE && alpha < val {
                    alpha = val;
                } else if teban == board::GOTE && beta > val {
                    beta = val;
                }
                if best.is_none() {
                    node2.best = Some(Best::new(val, x1, y1));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2));
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    node2.best = Some(Best::new(val, x1, y1));
                    node2.hyoka = Some(val);
                    nd1.best = Some(Best::new(val, x2, y2));
                } else {
                    nd2.release();
                }
            }
            // tt.dumpsz();
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        moves2.sort_by(|a, b| {
            let ia1 = if a.0 == 0 { 0 } else { a.0 + a.1 * 8 - 9 };
            let ia2 = if a.2 == 0 { 0 } else { a.2 + a.3 * 8 - 9 };
            let ib1 = if b.0 == 0 { 0 } else { b.0 + b.1 * 8 - 9 };
            let ib2 = if b.2 == 0 { 0 } else { b.2 + b.3 * 8 - 9 };
            let pa = SORT_PRI[ia1 as usize] * 10 + SORT_PRI[ia2 as usize];
            let pb = SORT_PRI[ib1 as usize] * 10 + SORT_PRI[ib2 as usize];
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for mv in moves2 {
            let (x1, y1, x2, y2) = mv;
            let newban = ban.r#move(x1, y1).unwrap();
            let newban2 = newban.r#move(x2, y2).unwrap();

            let nd1 = match node.child.iter_mut().find(
                    |a| a.x == x1 && a.y == y1) {
                    None => {
                        node.child.push(Node::new(x1, y1, depth - 1, teban));
                        node.child.last_mut().unwrap()
                    },
                    Some(n) => n,
                };
            nd1.child.push(Node::new(x2, y2, depth - 2, -teban));
            let mut nd2 = nd1.child.last_mut().unwrap();
            let val = if cfg!(feature="withtt") {
                    Node::think_internal_ab_tt(
                        &mut nd2, &newban2, alpha, beta, &mut tt)
                } else {
                    Node::think_internal_ab(
                        &mut nd2, &newban2, alpha, beta)
                };

            nd1.hyoka = val;
            node.kyokumen += nd2.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if teban == board::SENTE && alpha < val {
                alpha = val;
            } else if teban == board::GOTE && beta > val {
                beta = val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x1, y1));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x1, y1));
                node.hyoka = Some(val);
                nd1.best = Some(Best::new(val, x2, y2));
            } else {
                nd2.release();
            }
        }
        sub.join().unwrap();
        // tt.dumpsz();
        let mut subresult = rx.recv().unwrap();
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

    pub fn think_internal_ab_tt(node:&mut Node, ban : &board::Board, alpha : f32, beta : f32,
            tt : &mut transptable::TranspositionTable) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            node.kyokumen = 1;
            return Some(Node::evalwtt(&ban, tt));
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
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
        }

        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = Node::think_internal_ab_tt(
                &mut node.child[idx], &newban, -beta, -alpha, tt);
            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
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

    pub fn think_internal_ab(node:&mut Node, ban : &board::Board, alpha : f32, beta : f32) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        if ban.nblank() == 0 || ban.is_passpass() {
            node.kyokumen = 1;
            return Some(ban.count()  as f32 * 10.0);
        }
        if depth == 0 {
            node.kyokumen = 1;
            return Some(Node::evaluate(&ban));
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
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
        }

        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = Node::think_internal_ab(
                &mut node.child[idx], &newban, -beta, -alpha);
            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
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

    pub fn vb_think_ab(ban : &board::Board, mut depth : u8) -> Option<(f32,Node)> {
        let mut node = node::Node::new(0, 0, depth, board::NONE);
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
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
        }
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = Node::vb_think_internal_ab(
                &mut node.child[idx], &newban, alpha, beta);
    println!("({},{})@{} {:?}", x, y, depth - 1, val);
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
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
                node.hyoka = Some(val);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some((node.best.as_ref().unwrap().hyoka, node))
    }

    pub fn vb_think_internal_ab(node:&mut Node, ban : &board::Board, alpha : f32, beta : f32) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        if depth == 0 {
            println!("depth zero");
            node.kyokumen = 1;
            return Some(Node::evaluate(&ban));
            // return Some(Node::evalwtt(&ban));
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
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia as usize];
                let pb = SORT_PRI[ib as usize];
                pa.partial_cmp(&pb).unwrap()
            });
        }
        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1, teban));
            let val = Node::vb_think_internal_ab(
                &mut node.child[idx], &newban, -beta, -alpha);
    println!("({},{})@{} {:?} {}", x, y, depth-1, val, ban.to_str());
            let ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x, y));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y));
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
        format!("{}{}{}",
            if self.teban == board::SENTE {
                board::STONE_SENTE
            } else {
                board::STONE_GOTE
            },
            board::STR_GOTE.chars().nth(self.x as usize).unwrap(), self.y)
    }

    pub fn dump(&self) -> String {
        let mut ret = String::new();
        ret += &format!("val:{:?}, {} nodes. ", self.hyoka, self.kyokumen);
        // ret += &format!("{:?}", self.best);
        // let x = self.best.unwrap().1;
        // let y = self.best.unwrap().2;
        // let n = self.child.iter().find(|&a| a.x == x && a.y == y);
        // ret += &format!("{:?}", n.unwrap().best);
        if self.best.is_none() {
            return  ret;
        }
        let mut n = self;
        loop {
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

            if n.best.is_none() {
                break;
            }
        }
        ret
    }
}

#[test]
fn test_node() {
    let nodede = Node::new(6, 5, 4, bitboard::NONE);
    let nodefg = Node::new(8, 7, 4, bitboard::NONE);

    let mut nodebc = Node::new(4, 3, 5, bitboard::NONE);
    nodebc.kyokumen = 3210;
    assert_eq!(nodebc.dump(), "val:None, 3210 nodes. ");

    let mut node9a = Node::new(2, 1, 5, bitboard::SENTE);
    node9a.kyokumen = 4321;
    node9a.hyoka = Some(99.9);
    node9a.best = Some(Best::new(99.9, 8, 7));
    node9a.child.push(nodede);
    node9a.child.push(nodefg);
    assert_eq!(node9a.dump(), "val:Some(99.9), 4321 nodes. []h7");

    let mut node56 = Node::new(5, 6, 6, bitboard::NONE);
    node56.kyokumen = 6543;
    assert_eq!(node56.dump(), "val:None, 6543 nodes. ");

    let mut node78 = Node::new(7, 8, 6, bitboard::NONE);
    node78.kyokumen = 5432;
    node78.hyoka = Some(99.9);
    node78.best = Some(Best::new(99.9, 2, 1));
    node78.child.push(nodebc);
    node78.child.push(node9a);
    assert_eq!(node78.dump(), "val:Some(99.9), 5432 nodes. @@b1[]h7");

    let mut node12 = Node::new(1, 2, 7, bitboard::SENTE);
    node12.kyokumen = 8765;
    node12.child.push(node56);
    node12.hyoka = Some(99.9);
    node12.best = Some(Best::new(99.9, 7, 8));
    node12.child.push(node78);
    assert_eq!(node12.dump(), "val:Some(99.9), 8765 nodes. []g8@@b1[]h7");

    let mut node34 = Node::new(3, 4, 7, bitboard::GOTE);
    node34.kyokumen = 7654;
    assert_eq!(node34.dump(), "val:None, 7654 nodes. ");

    let mut node = Node::new(99, 2, 8, bitboard::NONE);
    node.hyoka = Some(99.9);
    node.kyokumen = 9876;
    node.best = Some(Best::new(99.9, 1, 2));
    node.child.push(node12);
    node.child.push(node34);
    assert_eq!(node.dump(), "val:Some(99.9), 9876 nodes. @@a2[]g8@@b1[]h7");
}
