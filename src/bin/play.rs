use chess_engine::ai::{mcts, negamax};
use chess_engine::engine::{Board, Color, GameStruct, Square};
use chess_engine::gui;
use rand::rngs::SmallRng;

#[derive(Clone, Copy)]
enum PlayerType {
    Input,
    RandomMove,
    Negamax { depth: u8 },
    Mcts { iterations: usize },
}

impl std::fmt::Display for PlayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => write!(f, "Input"),
            Self::RandomMove => write!(f, "RandomMove"),
            Self::Negamax { depth } => write!(f, "Negamax (depth {depth})"),
            Self::Mcts { iterations } => {
                write!(f, "Monte Carlo Tree Search ({iterations} iterations)")
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

fn read_usize_input() -> Result<usize, Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().parse()?)
}

fn run_game(
    white: PlayerType,
    black: PlayerType,
    show_gui: bool,
) -> Result<Option<Color>, Box<dyn std::error::Error>> {
    let mut game = GameStruct::new();
    let mut last_move_squares: Option<(Square, Square)> = None;
    let mut rng: SmallRng = rand::make_rng();

    loop {
        if show_gui {
            gui::display_board(&game, last_move_squares, None, &Vec::new());
        }

        let current_player = match game.b.side_to_move {
            Color::White => white,
            Color::Black => black,
        };

        let mv_result = match current_player {
            PlayerType::Input => {
                if !show_gui {
                    return Err("Input not supported in batch mode.".into());
                }
                gui::get_player_move(&game, last_move_squares)
            }
            PlayerType::RandomMove => game.get_random_move(&mut rng),
            PlayerType::Negamax { depth } => negamax::search_root(&mut game, depth),
            PlayerType::Mcts { iterations } => mcts::mcts_search(&game, iterations, &mut rng),
        };

        if let Some(mv) = mv_result {
            game.make_move(mv);
            last_move_squares = Some((mv.from(), mv.to()));
        } else {
            let result = if game.b.has_legal_moves() {
                false
            } else {
                game.b.is_king_checked()
            };

            if show_gui {
                gui::print_game_result(&game, result);
            }

            if result {
                return Ok(Some(game.b.side_to_move.flip()));
            }
            return Ok(None);
        }
    }
}

fn play_batch_game(is_bench: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (num_games, player_a, player_b) = if is_bench {
        (10_000, PlayerType::RandomMove, PlayerType::RandomMove)
    } else {
        (
            {
                println!("Number of games:");
                read_usize_input()?
            },
            loop {
                match ask_player_type("A") {
                    Ok(t) => break t,
                    Err(e) => println!("Error: {e:?}, Try again."),
                }
            },
            loop {
                match ask_player_type("B") {
                    Ok(t) => break t,
                    Err(e) => println!("Error: {e:?}, Try again."),
                }
            },
        )
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
        }
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
            let depth = u8::try_from(read_usize_input()?)?;
            P::Negamax { depth }
        }
        3 => {
            println!("Number of iterations :");
            let iterations = read_usize_input()?;
            P::Mcts { iterations }
        }
        _ => return Err("Choice out of range".into()),
    };
    Ok(player_type)
}
