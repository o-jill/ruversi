pub struct Node {
    child : Vec<Node>,
    hyoka : Option<f32>,
    pub kyokumen : usize,
    best : Option<i32>,
    pub x : usize,
    pub y : usize,
}

impl Node {
    pub fn new(x : usize, y : usize) -> Node {
        Node {
            child : Vec::<Node>::new(),
            hyoka : None,
            kyokumen : 0,
            best : None,
            x : x,
            y : y,
        }
    }
}