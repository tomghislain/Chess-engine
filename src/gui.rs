use crate::engine::{Board, Move, MoveList, Piece, Square, board::BoardHistory, types};
use crossterm::{cursor, event, execute, queue, style, terminal};
use std::io::Write;

// Size of a square of the board in characters
const SQ_HEIGHT: u16 = 3;
const SQ_WIDTH: u16 = 6;
const EMPTY_SQUARE_STRING: [&str; SQ_HEIGHT as usize] = ["      ", "      ", "      "];
const DARK_SQUARE_COLOR: style::Color = style::Color::Rgb {
    r: 75,
    g: 75,
    b: 75,
};
const LIGHT_SQUARE_COLOR: style::Color = style::Color::Rgb {
    r: 200,
    g: 200,
    b: 200,
};
const PIECE_STRINGS: [[&str; SQ_HEIGHT as usize]; 6] = [
    [" ▄▄▄  ", " ▀█▀  ", "▄███▄ "],
    [" ▄▄▄  ", "▀▀██  ", "▄███▄ "],
    [" ▄    ", " █▄█  ", "▄▄█▄▄ "],
    ["▄ ▄ ▄ ", "▀███▀ ", "▄███▄ "],
    ["  ▄   ", "▀▄█▄▀ ", "█████ "],
    ["  ▄   ", "▄▀█▀▄ ", "█████ "],
];
const BLACK_PIECES_COLOR: style::Color = style::Color::Rgb { r: 0, g: 0, b: 0 };
const WHITE_PIECES_COLOR: style::Color = style::Color::Rgb {
    r: 255,
    g: 255,
    b: 255,
};
const PREVIOUS_MOVE_COLOR: style::Color = style::Color::Rgb {
    r: 200,
    g: 200,
    b: 50,
};
const SELECTED_SQUARE_COLOR: style::Color = style::Color::Rgb {
    r: 50,
    g: 50,
    b: 200,
};
const POSSIBLE_SQUARES_COLOR: style::Color = style::Color::Rgb {
    r: 200,
    g: 50,
    b: 50,
};
const PROMOTION_STRING: [&str; SQ_HEIGHT as usize] = [" N  B ", "      ", " R  Q "];
const PROMOTION_COLOR: style::Color = style::Color::Rgb {
    r: 150,
    g: 10,
    b: 10,
};

const fn sq_to_top_left(sq: u8) -> (u16, u16) {
    let (a1_col, a1_row) = (0, 7 * SQ_HEIGHT);

    let (file, rank) = ((sq % 8) as u16, (sq / 8) as u16);
    let (col_offset, row_offset) = (file * SQ_WIDTH, rank * SQ_HEIGHT);
    let (col, row) = (a1_col + col_offset, a1_row - row_offset);

    (col, row)
}

pub fn display_board<H: BoardHistory>(
    board: &Board<H>,
    previous_move_squares: Option<(Square, Square)>,
    move_from: Option<u8>,
    possible_to_squares: &[Square],
) {
    //TODO: Handle terminal too small for board
    let mut stdout = std::io::stdout();
    let _ = queue!(
        stdout,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    );
    for sq_index in 0..64 {
        let sq = Square(sq_index);

        let index = ((sq_index ^ (sq_index >> 3)) & 1) as usize;
        let sq_colors = [DARK_SQUARE_COLOR, LIGHT_SQUARE_COLOR];
        let sq_color = if possible_to_squares.contains(&Square(sq_index)) {
            POSSIBLE_SQUARES_COLOR
        } else if let Some((previous_from, previous_to)) = previous_move_squares
            && (sq == previous_from || sq == previous_to)
        {
            PREVIOUS_MOVE_COLOR
        } else if let Some(from_sq) = move_from
            && sq_index == from_sq
        {
            SELECTED_SQUARE_COLOR
        } else {
            sq_colors[index]
        };

        let _ = queue!(stdout, crossterm::style::SetBackgroundColor(sq_color),);
        let piece = board.piece_at(Square(sq_index));
        // TODO: match piece enum with corresponding ascii character
        let pieces_colors = [WHITE_PIECES_COLOR, BLACK_PIECES_COLOR];
        let (piece_string, piece_color) = if piece == Piece::None {
            (EMPTY_SQUARE_STRING, style::Color::Red)
        } else {
            (
                PIECE_STRINGS[piece as usize % 6],
                pieces_colors[piece as usize / 6],
            )
        };
        let _ = queue!(stdout, crossterm::style::SetForegroundColor(piece_color),);
        let (col, row) = sq_to_top_left(sq_index);
        for line in 0..SQ_HEIGHT {
            let _ = queue!(
                stdout,
                crossterm::cursor::MoveTo(col, row + line),
                crossterm::style::Print(piece_string[line as usize]),
            );
        }
    }
    let _ = queue!(stdout, crossterm::style::ResetColor,);
    let _ = stdout.flush();
}

const fn coords_to_sq(col: u16, row: u16) -> Option<u8> {
    let (a1_col, a1_row) = (
        // (term_width - 8 * SQ_WIDTH) / 2,
        0,
        // (term_height + (6 * SQ_HEIGHT)) / 2,
        7 * SQ_HEIGHT,
    );

    let board_left = a1_col;
    let board_right = a1_col + 8 * SQ_WIDTH;
    let board_top = a1_row - 7 * SQ_HEIGHT;
    let board_bottom = a1_row + SQ_HEIGHT;

    if col < board_left || col >= board_right || row < board_top || row >= board_bottom {
        return None;
    }

    let file = (col - a1_col) / SQ_WIDTH;
    let rank = (a1_row + SQ_HEIGHT - 1 - row) / SQ_HEIGHT;

    if file < 8 && rank < 8 {
        #[allow(clippy::cast_possible_truncation)]
        Some((rank * 8 + file) as u8)
    } else {
        None
    }
}

pub fn get_player_move<H: BoardHistory>(
    board: &Board<H>,
    previous_move_squares: Option<(Square, Square)>,
) -> Option<Move> {
    if !board.has_legal_moves() || board.is_draw() {
        return None;
    }
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, event::EnableMouseCapture);
    let _ = execute!(stdout, cursor::Hide);
    let mut from: Option<u8> = None;
    let mut legal_moves = MoveList::new();
    board.get_legal_moves_list(&mut legal_moves);
    let mut possible_moves = MoveList::new();
    loop {
        let read_event = event::read();
        if let Ok(event::Event::Mouse(event::MouseEvent {
            kind, column, row, ..
        })) = read_event
            && let event::MouseEventKind::Down(_) = kind
        {
            let clicked = coords_to_sq(column, row);
            if let Some(clicked_sq) = clicked {
                if from.is_some() {
                    let valid_moves: Vec<&Move> = possible_moves
                        .iter()
                        .filter(|m| m.to() == Square(clicked_sq))
                        .collect();
                    let move_number = valid_moves.len();
                    match move_number.cmp(&1) {
                        std::cmp::Ordering::Equal => {
                            return Some(*valid_moves[0]);
                        }
                        std::cmp::Ordering::Greater => {
                            let promotion_piece =
                                get_promotion_piece(clicked_sq, board.side_to_move);
                            return Some(
                                **valid_moves
                                    .iter()
                                    .filter(|&m| m.promotion() == promotion_piece)
                                    .collect::<Vec<&&Move>>()[0],
                            );
                        }
                        std::cmp::Ordering::Less => {
                            from = Some(clicked_sq);
                            possible_moves = legal_moves
                                .iter()
                                .filter_map(|&m| {
                                    if m.from() == Square(clicked_sq) {
                                        Some(m)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                        }
                    }
                } else {
                    from = Some(clicked_sq);
                    possible_moves = legal_moves
                        .iter()
                        .filter_map(|&m| {
                            if m.from() == Square(clicked_sq) {
                                Some(m)
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
            let possible_to_squares = if from.is_some() {
                possible_moves.iter().map(|m| m.to()).collect()
            } else {
                Vec::new()
            };
            display_board(board, previous_move_squares, from, &possible_to_squares);
        } else if let Ok(event::Event::Key(event::KeyEvent {
            code, modifiers, ..
        })) = read_event
            && code == event::KeyCode::Char('c')
            && modifiers == event::KeyModifiers::CONTROL
        {
            panic!("UserInterrupt");
        }
    }
}

fn get_promotion_piece(sq: u8, color: types::Color) -> Piece {
    let mut stdout = std::io::stdout();
    let (sq_col, sq_row) = sq_to_top_left(sq);

    let _ = queue!(
        stdout,
        style::SetBackgroundColor(PROMOTION_COLOR),
        style::SetForegroundColor(style::Color::White),
    );

    for line in 0..SQ_HEIGHT {
        let _ = queue!(
            stdout,
            cursor::MoveTo(sq_col, sq_row + line),
            style::Print(PROMOTION_STRING[line as usize]),
        );
    }

    let _ = queue!(stdout, style::ResetColor);
    let _ = stdout.flush();

    loop {
        let read_event = event::read();
        if let Ok(event::Event::Mouse(event::MouseEvent {
            kind, column, row, ..
        })) = read_event
            && let event::MouseEventKind::Down(_) = kind
        {
            // Check if click is inside this square
            if column >= sq_col
                && column < sq_col + SQ_WIDTH
                && row >= sq_row
                && row < sq_row + SQ_HEIGHT
            {
                let local_x = column - sq_col;
                let local_y = row - sq_row;

                match (local_y, local_x) {
                    (0, 0..=2) => {
                        return match color {
                            types::Color::White => Piece::WN,
                            types::Color::Black => Piece::BN,
                        };
                    }
                    (0, 3..=5) => {
                        return match color {
                            types::Color::White => Piece::WB,
                            types::Color::Black => Piece::BB,
                        };
                    }
                    (2, 0..=2) => {
                        return match color {
                            types::Color::White => Piece::WR,
                            types::Color::Black => Piece::BR,
                        };
                    }
                    (2, 3..=5) => {
                        return match color {
                            types::Color::White => Piece::WQ,
                            types::Color::Black => Piece::BQ,
                        };
                    }
                    _ => {}
                }
            }
        } else if let Ok(event::Event::Key(event::KeyEvent {
            code, modifiers, ..
        })) = read_event
            && code == event::KeyCode::Char('c')
            && modifiers == event::KeyModifiers::CONTROL
        {
            panic!("UserInterrupt");
        }
    }
}

pub fn print_game_result<H: BoardHistory>(board: &Board<H>, result: bool) {
    let mut stdout = std::io::stdout();

    let result_row = 8 * SQ_HEIGHT;

    let result_string = if result {
        format!("{} has won!", board.side_to_move.flip())
    } else {
        "Draw!".to_string()
    };

    let _ = queue!(
        stdout,
        cursor::MoveTo(0, result_row),
        style::ResetColor,
        style::Print(result_string),
    );

    let _ = stdout.flush();
}
