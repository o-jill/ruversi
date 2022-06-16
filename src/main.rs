mod board;

fn main() {
    println!("Hello, reversi world!");
    let ban = board::Board::new();
    ban.put();
}
