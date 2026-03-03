mod ai;
mod game;
mod gui;

#[derive(Clone, Copy)]
enum PlayerType {
    Input,
    RandomMove,
    MirrorMove,
}

impl std::fmt::Display for PlayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerType::Input => write!(f, "Input"),
            PlayerType::RandomMove => write!(f, "RandomMove"),
            PlayerType::MirrorMove => write!(f, "MirrorMove"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ask_action()
    // Initialize the board
    // let depth = 5;
    // println!(
    //     "Number of moves at depth {}: {}",
    //     depth,
    //     game::perft(&mut board, depth)
    // );
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
        1 => play_batch_game()?,
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
    let mut last_move = game::Move::new(65, 65, 0, None);

    loop {
        if show_gui {
            crossterm::terminal::enable_raw_mode()?;
            gui::display_board(
                &board,
                Some((last_move.from, last_move.to)),
                None,
                Vec::new(),
            );
        }

        let current_player = match board.side_to_move {
            game::Color::White => white,
            game::Color::Black => black,
        };

        let mv = match current_player {
            PlayerType::Input => {
                if !show_gui {
                    return Err("Input not supported in batch mode.".into());
                }
                gui::get_player_move(&mut board, Some((last_move.from, last_move.to)))
            }
            PlayerType::RandomMove => ai::get_random_move(&mut board),
            _ => unimplemented!(),
        };

        if show_gui {
            crossterm::terminal::disable_raw_mode()?;
        }

        board.make_move(mv);
        last_move = mv;

        if let Some(result) = board.game_result() {
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

fn play_batch_game() -> Result<(), Box<dyn std::error::Error>> {
    println!("Number of games:");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    let num_games: usize = buffer.trim().parse().expect("Invalid number");

    let player_a = loop {
        match ask_player_type("A") {
            Ok(t) => break t,
            Err(e) => println!("Error: {e:?}, Try again."),
        }
    };

    let player_b = loop {
        match ask_player_type("B") {
            Ok(t) => break t,
            Err(e) => println!("Error: {e:?}, Try again."),
        }
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
    let player_types = [P::Input, P::RandomMove, P::MirrorMove];
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
