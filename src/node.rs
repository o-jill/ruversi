use super::*;
use rand::Rng;
use std::fs;

static mut INITIALIZED : bool = false;

/*
 * input: NUMCELL * NUMCELL + 1(teban) + 1
 * hidden: 4 + 1
 * output: 1
 */
static mut WEIGHT : Option<Vec<f32>> = None;

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
            if self.teban == board::SENTE { "@@" } else { "[]" },
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

    let mut rng = rand::thread_rng();
    let sz = board::CELL_2D * 4 + 4 + 4 + 4 + 1;
    let range =
        f64::sqrt(6.0) / f64::sqrt((board::CELL_2D + 1 + 4 + 1) as f64);
    unsafe {
        let mut array = Vec::<f32>::new();
        array.resize_with(sz, || {
            (rng.gen::<f64>() * 2.0 * range - range) as f32
        });
        WEIGHT = Some(array);

        INITIALIZED = true;
    }
}

pub fn read_weight(path : &str) -> Result<(), String> {
    let content = fs::read_to_string(path).unwrap();
    let csv = content.split(",").collect::<Vec<_>>();
    let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
    unsafe {
        let wsz = WEIGHT.as_ref().unwrap().len();
        let nsz = newtable.len();
        if wsz != nsz {
            return Err(String::from("size mismatch"));
        }
        WEIGHT = Some(newtable);
    }
    Ok(())
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
    fn evaluate2(ban : &board::Board) -> f32 {
        let mut sum : f32;
        let cells = &ban.cells;
        let teban = ban.teban;
        let mut hidden : [f32 ; 5] = [0.0 ; 5];
        let w1sz = board::CELL_2D + 1 + 1;
        unsafe {
            let ow = WEIGHT.as_ref().unwrap();
            let w2 = &ow.as_slice()[w1sz * 4..];

            sum = *ow.last().unwrap();

            for i in 0..4 {
                let w1 = &ow.as_slice()[i * w1sz .. (i + 1) * w1sz];
                let mut hidsum : f32 = *w1.last().unwrap();
                for (idx, c)  in cells.iter().enumerate() {
                    hidsum += *c as f32 * w1[idx];
                }
                hidsum += teban as f32 * w1[w1sz - 2];
                hidden[i] = w2[i] / (f32::exp(hidsum) + 1.0);
                sum += hidden[i];
            }
            // hidden -> output
        }
        sum
    }

    pub fn think(node:&mut Node, ban : &board::Board) -> Option<f32> {
        let depth = node.depth;
        if depth == 0 {
            node.kyokumen = 1;
            // return Some(Node::evaluate(&ban));
            return Some(Node::evaluate2(&ban));
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
            node.best = Some(Best::new(val.unwrap(), 0, 0, teban));
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
            let best = node.best.as_ref();
            let val = val.unwrap();
            if best.is_none() {
                node.best = Some(Best::new(val, x, y, teban));
            } else if teban == board::SENTE && best.unwrap().hyoka < val {
                node.best = Some(Best::new(val, x, y, teban));
            } else if teban == board::GOTE && best.unwrap().hyoka > val {
                node.best = Some(Best::new(val, x, y, teban));
            } else {
                // node.child[node.child.len() - 1].as_ref().unwrap().release();
                node.child[idx].release();
            }
        }
        return Some(node.best.as_ref().unwrap().hyoka);
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
            ret += &format!("{}", best.to_str());
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
