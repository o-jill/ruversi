const SENTE : i8 = 1;
const BLANK : i8 = 0;
const GOTE : i8 = -1;

pub struct Board {
    cells: Vec<i8>,
}

fn index(x: usize, y: usize) -> usize {
    x + y * 8
}

impl Board {
  pub fn new() -> Board {
    let mut ret = Board {
        cells : Vec::new(),
    };
    ret.cells.resize(8 * 8, BLANK);
    ret.cells[index(3, 3)] = SENTE;
    ret.cells[index(4, 4)] = SENTE;
    ret.cells[index(3, 4)] = GOTE;
    ret.cells[index(4, 3)] = GOTE;
    ret
  }

  pub fn put(&self) {
    println!("Hello, reversi board! sz:{}", self.cells.len());
    for y in 0..8 {
        for x in 0..8 {
            let c = self.cells[index(x, y)];
            print!("|{}", String::from(
                if c == SENTE {
                    "@@"
                } else if c == GOTE {
                    "[]"
                } else {
                    "__"
                })
            );
        }
        println!("|");
    }
  }
}
