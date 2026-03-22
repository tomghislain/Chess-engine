mod game;
mod gui;
mod mcts;
mod negamax;

use std::env;

#[derive(Clone, Copy)]
enum PlayerType {
    Input,
    RandomMove,
    Negamax,
    Mcts,
}

impl std::fmt::Display for PlayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerType::Input => write!(f, "Input"),
            PlayerType::RandomMove => write!(f, "RandomMove"),
            PlayerType::Negamax => write!(f, "Negamax"),
            PlayerType::Mcts => write!(f, "Monte Carlo Tree Search"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().any(|arg| arg == "--bench") {
        play_batch_game(true)
    } else if env::args().any(|arg| arg == "--perft") {
        let mut board = game::Board::new();
        let depth = 7;
        let start = std::time::Instant::now();
        println!("Number of moves at depth {}: {}", depth, board.perft(depth));
        println!("Time taken: {:?}", start.elapsed());
        Ok(())
    } else if env::args().any(|arg| arg == "--mcts") {
        let mut board = game::Board::new();
        let iterations = 100_000;
        mcts::mcts_search(&mut board, iterations);
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
) -> Result<Option<game::Color>, Box<dyn std::error::Error>> {
    let mut board = game::Board::new();
    let mut last_move = game::Move::new(
        0,
        0,
        game::Piece::WK,
        game::Piece::None,
        game::Piece::None,
        0,
    );

    loop {
        if show_gui {
            crossterm::terminal::enable_raw_mode()?;
            gui::display_board(
                &board,
                Some((last_move.from(), last_move.to())),
                None,
                Vec::new(),
            );
        }

        let current_player = match board.side_to_move {
            game::Color::White => white,
            game::Color::Black => black,
        };

        let mv_result = match current_player {
            PlayerType::Input => {
                if !show_gui {
                    return Err("Input not supported in batch mode.".into());
                }
                gui::get_player_move(&mut board, Some((last_move.from(), last_move.to())))
            }
            PlayerType::RandomMove => mcts::get_random_move(&mut board),
            PlayerType::Negamax => negamax::search_root(&mut board, 4),
            PlayerType::Mcts => mcts::mcts_search(&mut board, 100),
        };

        if let Some(mv) = mv_result {
            if show_gui {
                crossterm::terminal::disable_raw_mode()?;
            }

            board.make_move(mv);
            last_move = mv;
        } else {
            let result = if board.get_legal_moves().is_empty() {
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
                    winner == game::Color::White
                } else {
                    winner == game::Color::Black
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
    let player_types = [P::Input, P::RandomMove, P::Negamax, P::Mcts];
    println!("Choose {color} player type");
    for (i, t) in player_types.iter().enumerate() {
        println!("{i}. {t}");
    }
    let index = read_usize_input()?;
    player_types
        .get(index)
        .copied()
        .ok_or("Choice out of range".into())
}
