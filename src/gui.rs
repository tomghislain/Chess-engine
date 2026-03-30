use crate::engine::{Board, Move, MoveList, Piece, Square, types};
use crossterm::{cursor, event, execute, queue, style, terminal};
use std::io::Write;

// Size of a square of the board in characters
const SQ_HEIGHT: u16 = 3;
const SQ_WIDTH: u16 = 6;
const EMPTY_SQUARE_STRING: [&str; SQ_HEIGHT as usize] = ["      ", "      ", "      "];
const DARK_SQUARE_COLOR: style::Color = style::Color::DarkGrey;
const LIGHT_SQUARE_COLOR: style::Color = style::Color::Grey;
const PIECE_STRINGS: [[&str; SQ_HEIGHT as usize]; 6] = [
    [" ▄▄▄  ", " ▀█▀  ", "▄███▄ "],
    [" ▄▄▄  ", "▀▀██  ", "▄███▄ "],
    [" ▄    ", " █▄█  ", "▄▄█▄▄ "],
    ["▄ ▄ ▄ ", "▀███▀ ", "▄███▄ "],
    ["  ▄   ", "▀▄█▄▀ ", "█████ "],
    ["  ▄   ", "▄▀█▀▄ ", "█████ "],
];
const BLACK_PIECES_COLOR: style::Color = style::Color::Black;
const WHITE_PIECES_COLOR: style::Color = style::Color::White;
const PREVIOUS_MOVE_COLOR: style::Color = style::Color::Yellow;
const SELECTED_SQUARE_COLOR: style::Color = style::Color::Blue;
const POSSIBLE_SQUARES_COLOR: style::Color = style::Color::Red;
const PROMOTION_STRING: [&str; SQ_HEIGHT as usize] = [" N  B ", "      ", " R  Q "];
const PROMOTION_COLOR: style::Color = style::Color::DarkRed;

fn sq_to_top_left(sq: u8) -> (u16, u16) {
    let (a1_col, a1_row) = (0, 7 * SQ_HEIGHT);

    let (file, rank) = ((sq % 8) as u16, (sq / 8) as u16);
    let (col_offset, row_offset) = (file * SQ_WIDTH, rank * SQ_HEIGHT);
    let (col, row) = (a1_col + col_offset, a1_row - row_offset);

    (col, row)
}

pub fn display_board(
    board: &Board,
    previous_move_squares: Option<(Square, Square)>,
    move_from: Option<u8>,
    possible_to_squares: Vec<Square>,
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
        let mut sq_color = sq_colors[index];
        if let Some((previous_from, previous_to)) = previous_move_squares
            && (sq == previous_from || sq == previous_to)
        {
            sq_color = PREVIOUS_MOVE_COLOR;
        }

        if possible_to_squares.contains(&Square(sq_index)) {
            sq_color = POSSIBLE_SQUARES_COLOR;
        }

        if let Some(from_sq) = move_from
            && sq_index == from_sq
        {
            sq_color = SELECTED_SQUARE_COLOR;
        }

        let _ = queue!(stdout, crossterm::style::SetBackgroundColor(sq_color),);
        let piece = board.piece_at(Square(sq_index));
        // TODO: match piece enum with corresponding ascii character
        let pieces_colors = [WHITE_PIECES_COLOR, BLACK_PIECES_COLOR];
        let (piece_string, piece_color) = if piece != Piece::None {
            (
                PIECE_STRINGS[piece as usize % 6],
                pieces_colors[piece as usize / 6],
            )
        } else {
            (EMPTY_SQUARE_STRING, style::Color::Red)
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

fn coords_to_sq(col: u16, row: u16) -> Option<u8> {
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
        Some((rank * 8 + file) as u8)
    } else {
        None
    }
}

pub fn get_player_move(
    board: &mut Board,
    previous_move_squares: Option<(Square, Square)>,
) -> Option<Move> {
    if !board.has_legal_moves() || board.is_draw() {
        return None;
    };
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, event::EnableMouseCapture);
    let _ = execute!(stdout, cursor::Hide);
    let mut from: Option<u8> = None;
    let mut moves = MoveList::new();
    board.get_legal_moves_list(&mut moves);
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
                    let valid_moves: Vec<&Move> = moves
                        .iter()
                        .filter(|m| m.to() == Square(clicked_sq))
                        .collect();
                    let move_number = valid_moves.len();
                    if move_number == 1 {
                        return Some(*valid_moves[0]);
                    } else if move_number > 1 {
                        let promotion_piece = get_promotion_piece(clicked_sq, board.side_to_move);
                        return Some(
                            **valid_moves
                                .iter()
                                .filter(|&m| m.promotion() == promotion_piece)
                                .collect::<Vec<&&Move>>()[0],
                        );
                    } else {
                        from = Some(clicked_sq);
                        moves = moves
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
                } else {
                    from = Some(clicked_sq);
                    moves = moves
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
                moves.iter().map(|m| m.to()).collect()
            } else {
                Vec::new()
            };
            display_board(board, previous_move_squares, from, possible_to_squares);
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
                    (0, 0) | (0, 1) | (0, 2) => {
                        return match color {
                            types::Color::White => Piece::WN,
                            types::Color::Black => Piece::BN,
                        };
                    }
                    (0, 3) | (0, 4) | (0, 5) => {
                        return match color {
                            types::Color::White => Piece::WB,
                            types::Color::Black => Piece::BB,
                        };
                    }
                    (2, 0) | (2, 1) | (2, 2) => {
                        return match color {
                            types::Color::White => Piece::WR,
                            types::Color::Black => Piece::BR,
                        };
                    }
                    (2, 3) | (2, 4) | (2, 5) => {
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

pub fn print_game_result(board: &Board, result: bool) {
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
