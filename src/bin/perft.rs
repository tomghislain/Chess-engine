use chess_engine::engine::{Board, GameStruct, MoveList};

fn main() {
    // GameStruct with undo_stack
    let mut game = GameStruct::new();
    let depth = 7;
    let mut move_lists = vec![MoveList::new(); depth as usize + 1];
    let start = std::time::Instant::now();
    println!(
        "Number of moves at depth {}: {}",
        depth,
        game.perft(depth, &mut move_lists)
    );
    println!("Time taken: {:?}", start.elapsed());

    // Board only
    let mcts_board = Board::new();
    let start = std::time::Instant::now();
    println!(
        "Number of moves at depth (mcts) {}: {}",
        depth,
        mcts_board.perft(depth)
    );
    println!("Time taken: {:?}", start.elapsed());
}
