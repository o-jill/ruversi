use super::*;
use typed_arena::Arena;

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
static mut ND_ROOT : Option<&mut NodeBB> = None;

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

    pub fn is_invalid(&self) -> bool {
        self.x == 255
    }
}

impl Default for Best {
    fn default() -> Self {
        Best { hyoka : -999999.0, x: 255, y: 255 }
    }
}

pub struct NodeBB<'a> {
    child : Vec<&'a NodeBB<'a>>,
    hyoka : f32,
    pub kyokumen : usize,
    pub best : Best,
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
        ND_ROOT = None;
        INITIALIZED = true;
    }
}

impl <'a> NodeBB<'a> {
    pub fn new(x : u8, y : u8, depth : u8, teban : i8) -> NodeBB<'a> {
        NodeBB {
            child : Vec::<&'a NodeBB>::new(),
            hyoka : -999999.0,
            kyokumen : 1,
            best : Best::default(),
            x,
            y,
            depth,
            teban,
        }
    }

    fn asignbest(&mut self, hyoka : f32, x : u8, y : u8) {
        self.hyoka = hyoka;
        self.best.hyoka = hyoka;
        self.best.x = x;
        self.best.y = y;
    }

    #[cfg(target_arch="x86_64")]
    fn evaluate(ban : &bitboard::BitBoard, wei : &weight::Weight) -> f32 {
        if cfg!(feature="nosimd") {
            wei.evaluatev7bb(ban)
        } else if cfg!(feature="avx") {
            wei.evaluatev7bb_simdavx(ban)
        } else {
            wei.evaluatev7bb_simd(ban)
        }
    }

    #[cfg(target_arch="aarch64")]
    fn evaluate(ban : &bitboard::BitBoard, wei : &weight::Weight) -> f32 {
        if cfg!(feature="nosimd") {
            wei.evaluatev7bb(ban)
        } else {
            wei.evaluatev7bb_simd_mul(ban)
        }
    }

    fn evalwtt(ban : &bitboard::BitBoard, wei : &weight::Weight, tt : &mut transptable::TranspositionTable) -> f32 {
        let id = if cfg!(feature="nosimd") {ban.to_id()} else {ban.to_id_simd()};
        tt.check_or_append(&id, || NodeBB::evaluate(ban, wei))
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
            if n.best.is_invalid() {break;}

            let best = &n.best;
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
        let best = &self.best;
        if best.is_invalid() {
            x = 99;
            y = 99;
        } else {
            x = best.x;
            y = best.y;
        }
        for ch in self.child.iter() {
            let best = if ch.x == x && ch.y == y {
                "!"
            } else { "" };
            ret += &format!("{} {}{} {:.1}\n",
                "*".repeat(offset), best, &ch.to_xy(), ch.hyoka);

            if !ch.child.is_empty() {
                ret += &ch.dumptree_sub(offset + 1);
            }
        }
        ret
    }
}

pub fn think_internal<'a>(ban : &bitboard::BitBoard, depth : u8, x : u8, y : u8, wei : &weight::Weight, arena: &'a Arena<NodeBB<'a>>)
        -> Option<(f32, &'a NodeBB<'a>)> {
    let teban = ban.teban;
    let node = arena.alloc(NodeBB::new(x, y, depth, teban));
    if ban.nblank() == 0 || ban.is_passpass() {
        let val = ban.countf32();
        node.hyoka = val;
        return Some((val, node));
    }
    if depth == 0 {
        let val = NodeBB::evaluate(ban, wei);
        node.hyoka = val;
        return Some((val, node));
    }
    let teban = ban.teban;
    // let sum = 0;
    let moves = ban.genmove();

    // no more empty cells
    if moves.is_none() {
        let val = ban.countf32();
        node.hyoka = val;
        return Some((val, node));
    }
    let moves = moves.unwrap();

    for (mvx, mvy) in moves {
        let newban = ban.r#move(mvx, mvy).unwrap();
        // let idx = node.child.len();
        let (val, ch) = think_internal(&newban, depth, mvx, mvy, wei, arena)?;

        node.kyokumen += ch.kyokumen;
        node.child.push(ch);
        let best = &node.best;
        let fteban = teban as f32;
        if best.is_invalid() || best.hyoka * fteban < val * fteban {
            node.asignbest(val, mvx, mvy);
        } else {
            // node.child[node.child.len() - 1].as_ref().unwrap().release();
            // node.child[idx].release();
        }
    }
    Some((node.hyoka, node))
}

#[allow(dead_code)]
pub fn think_simple<'a>(ban : &bitboard::BitBoard, depth : u8, arena : &'a Arena<NodeBB<'a>>)
        -> Option<(f32, &'a NodeBB<'a>)> {
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

    let (val, node) = think_internal(ban, depth, 0, 0, wei, &arena)?;
    let val = val * ban.teban as f32;

    Some((val, node))
}

#[allow(dead_code)]
pub fn think_ab_simple<'a>(ban : &bitboard::BitBoard, depth : u8, arena : &'a Arena<NodeBB<'a>>)
        -> Option<(f32, &'a NodeBB<'a>)> {
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

    let alpha : f32 = -123456.7;
    let beta : f32 = 123456.7;
    let (val,  node) = think_internal_ab( ban, depth, 0, 0, alpha, beta, wei, &arena);
    let val = val * ban.teban as f32;

    Some((val, node))
}

#[allow(dead_code)]
pub fn think_ab_simple_gk<'a>(ban : &bitboard::BitBoard, depth : u8, wei : &weight::Weight, arena : &'a Arena<NodeBB<'a>>)
        -> Option<(f32, &'a NodeBB<'a>)> {
    if depth == 0 {
        return None;
    }
    if ban.is_passpass() {
        return None;
    }
    // no more empty cells
    let _moves = ban.genmove()?;

    // let node = nd;
    // let arena = Arena::new();

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

    let alpha : f32 = -123456.7;
    let beta : f32 = 123456.7;
    let (val, node) = think_internal_ab(ban, depth, 0, 0, alpha, beta, wei, arena);
    let val = val * ban.teban as f32;

    Some((val, node))
}

pub fn think_internal_ab<'a>(ban : &bitboard::BitBoard, depth : u8, x : u8, y : u8,
        alpha : f32, beta : f32, wei : &weight::Weight, arena: &'a Arena<NodeBB<'a>>) -> (f32, &'a NodeBB<'a>) {
    let mut newalpha = alpha;
    let teban = ban.teban;

    let node = arena.alloc(NodeBB::new(x, y, depth, teban));
    let fteban = teban as f32;
    if ban.nblank() == 0 || ban.is_passpass() {
        let val = ban.countf32();
        node.hyoka = val;
        return (val * fteban, node);
    } else if depth <= 0 {
        let val = NodeBB::evaluate(&ban, wei);
        node.hyoka = val;
        return (val * fteban, node);
    }

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

    node.child.reserve(moves.len());
    for (mvx, mvy) in moves {
        let newban = ban.r#move(mvx, mvy).unwrap();
        let (val, ch) = think_internal_ab( &newban, depth - 1, mvx, mvy,-beta, -newalpha, wei, arena);
        let val = -val;
        node.kyokumen += ch.kyokumen;
        node.child.push(ch);
        if newalpha < val {
            newalpha = val;
            node.best = Some(Best::new(val, mvx, mvy));
            node.hyoka = Some(val);
        } else if node.best.is_none() {
            node.best = Some(Best::new(val, mvx, mvy));
            node.hyoka = Some(val);
        } else {
            // ch.release();
        }
        if newalpha >= beta {
            // cut
            node.hyoka = Some(newalpha);
            return (newalpha, node);
        }
    }
    node.hyoka = Some(newalpha);
    (newalpha, node)
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
