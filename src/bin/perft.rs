use chess_engine::engine::{Board, MoveList};

fn main() {
    let mut board = Board::new();
    let depth = 7;
    let mut move_lists = vec![MoveList::new(); depth as usize + 1];
    let start = std::time::Instant::now();
    println!(
        "Number of moves at depth {}: {}",
        depth,
        board.perft(depth, &mut move_lists)
    );
    println!("Time taken: {:?}", start.elapsed());
}
