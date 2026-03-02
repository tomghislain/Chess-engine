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

impl PlayerType {
    fn get_move(
        &self,
        board: &mut game::Board,
        previous_move_squares: Option<(u8, u8)>,
    ) -> game::Move {
        match self {
            PlayerType::Input => gui::get_player_move(board, previous_move_squares),
            _ => board.get_pseudo_legal_moves()[0],
        }
    }
}

fn main() {
    play_single_game();
    // Initialize the board
    // let depth = 5;
    // println!(
    //     "Number of moves at depth {}: {}",
    //     depth,
    //     game::perft(&mut board, depth)
    // );
}

fn play_single_game() {
    // Ask the user to choose the PlayerType of white and black
    let (white_player, black_player) = get_player_types();
    println!("Starting game with white ({white_player}) vs black ({black_player})");

    let mut board = game::Board::new();
    let mut mv = game::Move::new(65, 65, 0, None);
    for _ in (0..).step_by(2) {
        let _ = crossterm::terminal::enable_raw_mode();
        gui::display_board(&board, Some((mv.from, mv.to)), None, Vec::new());
        mv = white_player.get_move(&mut board, Some((mv.from, mv.to)));
        let _ = crossterm::terminal::disable_raw_mode();
        board.make_move(mv);
        let maybe_result = board.game_result();
        if let Some(result) = maybe_result {
            gui::print_game_result(&board, result);
            break;
        }
        let _ = crossterm::terminal::enable_raw_mode();
        gui::display_board(&board, Some((mv.from, mv.to)), None, Vec::new());
        mv = black_player.get_move(&mut board, Some((mv.from, mv.to)));
        let _ = crossterm::terminal::disable_raw_mode();
        board.make_move(mv);
        let maybe_result = board.game_result();
        if let Some(result) = maybe_result {
            gui::print_game_result(&board, result);
            break;
        }
    }
}

fn get_player_types() -> (PlayerType, PlayerType) {
    let white = loop {
        match ask_player_type("white") {
            Ok(r#type) => break r#type,
            Err(e) => println!("Error: {e:?}, Try again."),
        }
    };
    let black = loop {
        match ask_player_type("black") {
            Ok(r#type) => break r#type,
            Err(e) => println!("Error: {e:?}, Try again."),
        }
    };
    (white, black)
}

fn ask_player_type(color: &str) -> Result<PlayerType, Box<dyn std::error::Error>> {
    use PlayerType as P;
    let player_types = [P::Input, P::RandomMove, P::MirrorMove];
    println!("Choose {color} player");
    for (index, r#type) in player_types.iter().enumerate() {
        println!("{index}. {type}");
    }
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    let input_index: usize = buffer.trim().parse::<usize>()?;

    player_types
        .get(input_index)
        .copied()
        .ok_or("Choice out of range".into())
}
