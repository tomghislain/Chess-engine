use chess_engine::engine::{Board, GameStruct, MoveList};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} <fen> <depth>", args[0]);
        std::process::exit(1)
    }

    let fen = &args[1];
    let depth: u8 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("depth must be an integer 0..=255");
        std::process::exit(1)
    });

    println!("FEN: {fen}");
    println!("Depth: {depth}");

    // Board only
    let mcts_board = Board::from_fen(fen);
    let start = std::time::Instant::now();
    println!(
        "Number of moves at depth (copy-only) :{}",
        mcts_board.perft(depth)
    );
    println!("Time taken: {:?}", start.elapsed());
}
