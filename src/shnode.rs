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
                leaf.lock().unwrap().best = Some(Best::new(val.unwrap(), x, y, teban));
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
            leaf.lock().unwrap().best = Some(Best::new(val.unwrap(), x, y, teban));
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
                    be = Some(lb.unwrap().clone());
                } else if lb.is_none() {
                    // nothing to do.
                } else if be.as_ref().unwrap().hyoka * fteban < lb.as_ref().unwrap().hyoka * fteban {
                    be = Some(lb.unwrap().clone());
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
                // nd.child[idx].lock().unwrap().release();
            }
        }
        Some(hyoka)
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
