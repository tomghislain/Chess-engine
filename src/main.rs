mod game;

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

fn main() {
    // Ask the user to choose the PlayerType of white and black
    let (white, black) = get_player_types();
    println!("Starting game with white ({white}) vs black ({black})");

    // Initialize the board

    // Game Loop
    // Display the board
    // Get white player's move
    // (call the function corresponding to the PlayerType)
    // Display the board
    // Get black player's move
    // (call the function corresponding to the PlayerType)
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
