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
    pub x : usize,
    pub y : usize,
    pub teban : i8,
}

impl Best {
    pub fn new(h : f32, x : usize, y : usize, t : i8) -> Best {
        Best { hyoka: h, x: x, y: y, teban: t }
    }

    pub fn pos(&self) -> String {
        format!("{}{}{}",
            if self.teban == board::SENTE {
                board::STONE_SENTE
            } else {
                board::STONE_GOTE
            },
            board::STR_GOTE.chars().nth(self.x).unwrap(), self.y)
    }

    pub fn to_str(&self) -> String {
        format!("h:{} {}", self.hyoka, self.pos())
    }
}

pub struct Node {
    child : Vec<Node>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    pub best : Option<Best>,
    pub x : usize,
    pub y : usize,
    depth : usize,
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
    pub fn new(x : usize, y : usize, depth : usize) -> Node {
        Node {
            child : Vec::<Node>::new(),
            hyoka : None,
            kyokumen : 0,
            best : None,
            x : x,
            y : y,
            depth : depth,
        }
    }

    fn evaluate(ban : &board::Board) -> f32 {
        unsafe {
            if cfg!(feature="nnv1") {
                if cfg!(feature = "nosimd") {
                    WEIGHT.as_ref().unwrap().evaluatev1(ban)
                } else {
                    WEIGHT.as_ref().unwrap().evaluatev1_simd(ban)
                }
            } else {
                if cfg!(feature = "nosimd") {
                    WEIGHT.as_ref().unwrap().evaluatev2(ban)
                } else {
                    // WEIGHT.as_ref().unwrap().evaluatev2_simd(ban)
                    WEIGHT.as_ref().unwrap().evaluatev2_simd2(ban)
                }
            }
        }
    }

    pub fn think(ban : &board::Board, mut depth : usize) -> Option<(f32,Node)> {
        let mut node = node::Node::new(0, 0, depth);
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
            let mut node2 = node::Node::new(0, 0, depth);
            let teban = ban2.teban;
            for mv in moves1 {
                let x = mv.0;
                let y = mv.1;
                let newban = ban2.r#move(x, y).unwrap();
                let idx = node2.child.len();
                node2.child.push(Node::new(x, y, depth - 1));
                let val = Node::think_internal(
                    &mut node2.child[idx], &newban);

                let mut ch = &mut node2.child[idx];
                ch.hyoka = val;
                node2.kyokumen += ch.kyokumen;
                let best = node2.best.as_ref();
                let val = val.unwrap();
                let fteban = teban as f32;
                if best.is_none() {
                    node2.best = Some(Best::new(val, x, y, teban));
                    node2.hyoka = Some(val);
                } else if fteban * best.unwrap().hyoka < fteban * val {
                    node2.best = Some(Best::new(val, x, y, teban));
                    node2.hyoka = Some(val);
                } else {
                    // node2.child[node.child.len() - 1].as_ref().unwrap().release();
                    node2.child[idx].release();
                }
            }
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        let teban = ban.teban;
        for mv in moves2 {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::think_internal(
                &mut node.child[idx], &newban);

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, x, y, teban));
                node.hyoka = Some(val);
                // println!("best : {}", val);
            } else if fteban * best.unwrap().hyoka < fteban * val {
                node.best = Some(Best::new(val, x, y, teban));
                node.hyoka = Some(val);
                // println!("best : -> {}", val);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        sub.join().unwrap();
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
            // return Some(Node::evaluate(&ban));
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
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::think_internal(
                &mut node.child[idx], &newban);

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            let fteban = teban as f32;
            if best.is_none() {
                node.best = Some(Best::new(val, x, y, teban));
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y, teban));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        Some(node.best.as_ref().unwrap().hyoka)
    }

    pub fn think_ab(ban : &board::Board, mut depth : usize) -> Option<(f32,Node)> {
        let mut node = node::Node::new(0, 0, depth);
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
            depth += 1;
            node.depth += 1;
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
                let pa = SORT_PRI[ia];
                let pb = SORT_PRI[ib];
                pa.partial_cmp(&pb).unwrap()
            });
            let teban = ban2.teban;
            let mut node2 = node::Node::new(0, 0, depth);
            let mut alpha : f32 = -100000.0;
            let mut beta : f32 = 100000.0;
            for mv in moves1 {
                let x = mv.0;
                let y = mv.1;
                let newban = ban2.r#move(x, y).unwrap();
                let idx = node2.child.len();
                node2.child.push(Node::new(x, y, depth - 1));
                let val = Node::think_internal_ab(
                    &mut node2.child[idx], &newban, alpha, beta);

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
                    node2.best = Some(Best::new(val, x, y, teban));
                    node2.hyoka = Some(val);
                } else if best.unwrap().hyoka * fteban < val * fteban {
                    node2.best = Some(Best::new(val, x, y, teban));
                    node2.hyoka = Some(val);
                } else {
                    // node2.child[node.child.len() - 1].as_ref().unwrap().release();
                    node2.child[idx].release();
                }
            }
            tx.send(node2).unwrap();
            // return Some(node.best.as_ref().unwrap().hyoka);
        });

        moves2.sort_by(|a, b| {
            let ia = a.0 + a.1 * 8 - 9;
            let ib = b.0 + b.1 * 8 - 9;
            let pa = SORT_PRI[ia];
            let pb = SORT_PRI[ib];
            pa.partial_cmp(&pb).unwrap()
        });
        let mut alpha : f32 = -100000.0;
        let mut beta : f32 = 100000.0;
        let teban = ban.teban;
        for mv in moves2 {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::think_internal_ab(
                &mut node.child[idx], &newban, alpha, beta);

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
                node.best = Some(Best::new(val, x, y, teban));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y, teban));
                node.hyoka = Some(val);
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        sub.join().unwrap();
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

    pub fn think_internal_ab(node:&mut Node, ban : &board::Board, alpha : f32, beta : f32) -> Option<f32> {
        let mut newalpha = alpha;
        let mut depth = node.depth;
        if depth == 0 {
            node.kyokumen = 1;
            // return Some(Node::evaluate(&ban));
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
        } else {
            moves.sort_by(|a, b| {
                let ia = a.0 + a.1 * 8 - 9;
                let ib = b.0 + b.1 * 8 - 9;
                let pa = SORT_PRI[ia];
                let pb = SORT_PRI[ib];
                pa.partial_cmp(&pb).unwrap()
            });
        }

        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::think_internal_ab(
                &mut node.child[idx], &newban, -beta, -alpha);
            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x, y, teban));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y, teban));
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

    pub fn vb_think_ab(ban : &board::Board, mut depth : usize) -> Option<(f32,Node)> {
        let mut node = node::Node::new(0, 0, depth);
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
                let pa = SORT_PRI[ia];
                let pb = SORT_PRI[ib];
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
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::vb_think_internal_ab(
                &mut node.child[idx], &newban, alpha, beta);
    println!("({},{})@{} {:?}", x, y, depth - 1, val);
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
                node.best = Some(Best::new(val, x, y, teban));
                node.hyoka = Some(val);
            } else if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y, teban));
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
            // return Some(Node::evaluate(&ban));
            return Some(Node::evaluate(&ban));
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
                let pa = SORT_PRI[ia];
                let pb = SORT_PRI[ib];
                pa.partial_cmp(&pb).unwrap()
            });
        }
        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::vb_think_internal_ab(
                &mut node.child[idx], &newban, -beta, -alpha);
    println!("({},{})@{} {:?} {}", x, y, depth-1, val, ban.to_str());
                let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let best = node.best.as_ref();
            let val = val.unwrap();
            if newalpha < -val {
                newalpha = -val;
            }
            if best.is_none() {
                node.best = Some(Best::new(val, x, y, teban));
                continue;
            }
            let fteban = teban as f32;
            if best.unwrap().hyoka * fteban < val * fteban {
                node.best = Some(Best::new(val, x, y, teban));
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
            let m = n.child.iter().find(|&a| a.x == x && a.y == y);
            if m.is_none() {
                break;
            }
            n = m.unwrap();
            // ret += &format!("{}", best.pos());
            ret += &best.pos();
            if n.best.is_none() {
                break;
            }
        }
        ret
    }
}
