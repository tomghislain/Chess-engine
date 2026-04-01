mod ai;
mod engine;
mod gui;

use crate::ai::{mcts, mcts2, negamax};
use crate::engine::{Board, Color, MoveList, Square};
use rand::rngs::SmallRng;
use std::env;

#[derive(Clone, Copy)]
enum PlayerType {
    Input,
    RandomMove,
    Negamax { depth: u8 },
    Mcts { iterations: usize },
    Mcts2 { iterations: usize },
}

impl std::fmt::Display for PlayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerType::Input => write!(f, "Input"),
            PlayerType::RandomMove => write!(f, "RandomMove"),
            PlayerType::Negamax { depth } => write!(f, "Negamax (depth {})", depth),
            PlayerType::Mcts { iterations } => {
                write!(f, "Monte Carlo Tree Search ({} iterations)", iterations)
            }
            PlayerType::Mcts2 { iterations } => {
                write!(f, "Monte Carlo Tree Search 2 ({} iterations)", iterations)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().any(|arg| arg == "--bench") {
        play_batch_game(true)
    } else if env::args().any(|arg| arg == "--perft") {
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
        Ok(())
    } else if env::args().any(|arg| arg == "--mcts") {
        let mut board = Board::new();
        let mut rng: SmallRng = rand::make_rng();
        let iterations = 100_000;
        mcts::mcts_search(&mut board, iterations, &mut rng);
        Ok(())
    } else {
        ask_action()
    }
}

fn read_usize_input() -> Result<usize, Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().parse()?)
}

fn ask_action() -> Result<(), Box<dyn std::error::Error>> {
    let player_types = ["Single game", "Batch game"];
    for (index, r#type) in player_types.iter().enumerate() {
        println!("{index}. {type}");
    }

    match read_usize_input()? {
        0 => play_single_game()?,
        1 => play_batch_game(false)?,
        _ => println!("Invalid input."),
    };

    Ok(())
}

fn run_game(
    white: PlayerType,
    black: PlayerType,
    show_gui: bool,
) -> Result<Option<Color>, Box<dyn std::error::Error>> {
    let mut board = Board::new();
    let mut last_move_squares: Option<(Square, Square)> = None;
    let mut rng: SmallRng = rand::make_rng();
    let mut tree = mcts2::Tree::new(&board);

    loop {
        if show_gui {
            crossterm::terminal::enable_raw_mode()?;
            gui::display_board(&board, last_move_squares, None, Vec::new());
        }

        let current_player = match board.side_to_move {
            Color::White => white,
            Color::Black => black,
        };

        let mv_result = match current_player {
            PlayerType::Input => {
                if !show_gui {
                    return Err("Input not supported in batch mode.".into());
                }
                gui::get_player_move(&mut board, last_move_squares)
            }
            PlayerType::RandomMove => mcts::get_random_move(&board, &mut rng),
            PlayerType::Negamax { depth } => negamax::search_root(&mut board, depth),
            PlayerType::Mcts { iterations } => mcts::mcts_search(&mut board, iterations, &mut rng),
            PlayerType::Mcts2 { iterations } => tree.search(&mut board, iterations, &mut rng),
        };

        if let Some(mv) = mv_result
            && !board.is_draw()
        {
            if show_gui {
                crossterm::terminal::disable_raw_mode()?;
            }

            board.make_move(mv);
            tree.advance_root(&board, mv);
            last_move_squares = Some((mv.from(), mv.to()));
        } else {
            let result = if !board.has_legal_moves() {
                board.is_king_checked()
            } else {
                false
            };

            if show_gui {
                gui::print_game_result(&board, result);
            }

            if result {
                return Ok(Some(board.side_to_move.flip()));
            } else {
                return Ok(None);
            }
        }
    }
}

fn play_batch_game(is_bench: bool) -> Result<(), Box<dyn std::error::Error>> {
    let num_games;
    let player_a;
    let player_b;
    if is_bench {
        num_games = 10_000;
        player_a = PlayerType::RandomMove;
        player_b = PlayerType::RandomMove;
    } else {
        println!("Number of games:");
        num_games = read_usize_input()?;

        player_a = loop {
            match ask_player_type("A") {
                Ok(t) => break t,
                Err(e) => println!("Error: {e:?}, Try again."),
            }
        };

        player_b = loop {
            match ask_player_type("B") {
                Ok(t) => break t,
                Err(e) => println!("Error: {e:?}, Try again."),
            }
        };
    };

    let mut a_wins = 0;
    let mut b_wins = 0;
    let mut draws = 0;

    for game_index in 0..num_games {
        // Swap colors every game
        let (white_player, black_player, a_is_white) = if game_index % 2 == 0 {
            (player_a, player_b, true)
        } else {
            (player_b, player_a, false)
        };

        let result = run_game(white_player, black_player, false)?;

        match result {
            Some(winner) => {
                let a_won = if a_is_white {
                    winner == Color::White
                } else {
                    winner == Color::Black
                };

                if a_won {
                    a_wins += 1;
                } else {
                    b_wins += 1;
                }
            }
            None => draws += 1,
        }
        if !is_bench {
            println!("Finished game {game_index}");
        };
    }
    println!("\nBatch finished!");
    println!("Player A ({player_a}) wins: {a_wins}");
    println!("Player B ({player_b}) wins: {b_wins}");
    println!("Draws: {draws}");

    Ok(())
}

fn play_single_game() -> Result<(), Box<dyn std::error::Error>> {
    // Ask the user to choose the PlayerType of white and black
    let (white_player, black_player) = get_player_types()?;
    println!("Starting game with white ({white_player}) vs black ({black_player})");

    run_game(white_player, black_player, true)?;
    Ok(())
}

fn get_player_types() -> Result<(PlayerType, PlayerType), Box<dyn std::error::Error>> {
    Ok((ask_player_type("White")?, ask_player_type("Black")?))
}

fn ask_player_type(color: &str) -> Result<PlayerType, Box<dyn std::error::Error>> {
    use PlayerType as P;
    let player_types = [
        P::Input,
        P::RandomMove,
        P::Negamax { depth: 0 },
        P::Mcts { iterations: 0 },
        P::Mcts2 { iterations: 0 },
    ];
    println!("Choose {color} player type");
    for (i, t) in player_types.iter().enumerate() {
        println!("{i}. {t}");
    }
    let index = read_usize_input()?;
    let player_type = match index {
        0 => P::Input,
        1 => P::RandomMove,
        2 => {
            println!("Search depth :");
            let depth = read_usize_input()? as u8;
            P::Negamax { depth }
        }
        3 => {
            println!("Number of iterations :");
            let iterations = read_usize_input()?;
            P::Mcts { iterations }
        }
        4 => {
            println!("Number of iterations :");
            let iterations = read_usize_input()?;
            P::Mcts2 { iterations }
        }
        _ => return Err("Choice out of range".into()),
    };
    Ok(player_type)
}
