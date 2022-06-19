use super::*;
use rand::Rng;

static mut INITIALIZED : bool = false;
static mut WEIGHT : Option<Vec<f32>> = None;

pub struct Node {
    child : Vec<Node>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    pub best : Option<(f32, usize, usize, i8)>,
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

    let mut rng = rand::thread_rng();
    unsafe {
        let mut array = Vec::<f32>::new();
        array.resize_with(board::CELL_2D, || {
            rng.gen::<f32>()
        });
        WEIGHT = Some(array);

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
        let mut sum : f32 = 0.0;
        let cells = &ban.cells;
        unsafe {
            let ow = &WEIGHT;
            let w = ow.as_ref().unwrap();
            for (i, we) in w.iter().enumerate() {
                sum += cells[i] as f32 * *we;
            }
            // for i in 0..board::CELL_2D {
            //     sum += cells[i] as f32 * w[i];
            // }
        }
        sum
    }

    pub fn think(node:&mut Node, ban : &board::Board) -> Option<f32> {
        let depth = node.depth;
        if depth == 0 {
            node.kyokumen = 1;
            return Some(Node::evaluate(&ban));
        }

        let teban = ban.teban;
        // let sum = 0;
        let moves = ban.genmove();

        if moves.is_empty() {
            let newban = ban.r#move(0, 0).unwrap();
            node.child.push(Node::new(0, 0, depth -1));
            let val = Node::think(
                &mut node.child[0], &newban);
            node.hyoka = val;
            node.kyokumen += node.child[0].kyokumen;
            node.best = Some((val.unwrap(), 0, 0, teban));
            return val;
        }

        for mv in moves {
            let x = mv.0;
            let y = mv.1;
            let newban = ban.r#move(x, y).unwrap();
            let idx = node.child.len();
            node.child.push(Node::new(x, y, depth - 1));
            let val = Node::think(
                &mut node.child[idx], &newban);

            let mut ch = &mut node.child[idx];
            ch.hyoka = val;
            node.kyokumen += ch.kyokumen;
            let val = val.unwrap();
            if node.best.is_none() {
                node.best = Some((val, x, y, teban));
            } else if teban == board::SENTE && node.best.unwrap().0 < val {
                node.best = Some((val, x, y, teban));
            } else if teban == board::GOTE && node.best.unwrap().0 > val {
                node.best = Some((val, x, y, teban));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        return Some(node.best.unwrap().0);
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
            ret += &format!("{:?}", n.best);
            let x = n.best.unwrap().1;
            let y = n.best.unwrap().2;
            let m = n.child.iter().find(|&a| a.x == x && a.y == y);
            if m.is_none() {
                break;
            }
            n = m.unwrap();
            ret += &format!("{:?}", n.best);
            if n.best.is_none() {
                break;
            }
        }
        ret
    }
}
