use crate::game;
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

fn sq_to_top_left(sq: u8, term_width: u16, term_height: u16) -> (u16, u16) {
    let (a1_col, a1_row) = (
        // (term_width - 8 * SQ_WIDTH) / 2,
        0,
        // (term_height + (6 * SQ_HEIGHT)) / 2,
        7 * SQ_HEIGHT,
    );

    let (file, rank) = ((sq % 8) as u16, (sq / 8) as u16);
    let (col_offset, row_offset) = (file * SQ_WIDTH, rank * SQ_HEIGHT);
    let (col, row) = (a1_col + col_offset, a1_row - row_offset);

    (col, row)
}

pub fn display_board(
    board: &game::Board,
    previous_move_squares: Option<(u8, u8)>,
    move_from: Option<u8>,
    possible_to_squares: Vec<u8>,
) {
    //TODO: Handle terminal too small for board
    let mut stdout = std::io::stdout();
    let _ = queue!(
        stdout,
        crossterm::cursor::Hide,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    );
    let (term_width, term_height) = if let Ok(size) = terminal::size() {
        (size.0, size.1)
    } else {
        (0, 0)
    };
    for sq in 0..64 {
        let index = ((sq ^ (sq >> 3)) & 1) as usize;
        let sq_colors = [LIGHT_SQUARE_COLOR, DARK_SQUARE_COLOR];
        let mut sq_color = sq_colors[index];
        if let Some((previous_from, previous_to)) = previous_move_squares
            && (sq == previous_from || sq == previous_to)
        {
            sq_color = PREVIOUS_MOVE_COLOR;
        }

        if possible_to_squares.contains(&sq) {
            sq_color = POSSIBLE_SQUARES_COLOR;
        }

        if let Some(from_sq) = move_from
            && sq == from_sq
        {
            sq_color = SELECTED_SQUARE_COLOR;
        }

        let _ = queue!(stdout, crossterm::style::SetBackgroundColor(sq_color),);
        let piece = board.piece_at(sq);
        // TODO: match piece enum with corresponding ascii character
        let pieces_colors = [WHITE_PIECES_COLOR, BLACK_PIECES_COLOR];
        let (piece_string, piece_color) = piece
            .map(|p| (PIECE_STRINGS[p as usize % 6], pieces_colors[p as usize / 6]))
            .unwrap_or((EMPTY_SQUARE_STRING, style::Color::Red));
        let _ = queue!(stdout, crossterm::style::SetForegroundColor(piece_color),);
        let (col, row) = sq_to_top_left(sq, term_width, term_height);
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
    board: &mut game::Board,
    previous_move_squares: Option<(u8, u8)>,
) -> game::Move {
    let moves = board.get_pseudo_legal_moves();
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, event::EnableMouseCapture);
    let _ = execute!(stdout, cursor::Hide);
    let mut from: Option<u8> = None;
    let mut moves_from = moves.clone();
    loop {
        if let Ok(event::Event::Mouse(event::MouseEvent {
            kind,
            column,
            row,
            modifiers: _,
        })) = event::read()
        {
            if kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                let clicked = coords_to_sq(column, row);
                if let Some(clicked_sq) = clicked {
                    if from.is_some() {
                        let valid_moves: Vec<&game::Move> =
                            moves_from.iter().filter(|m| m.to == clicked_sq).collect();
                        if !valid_moves.is_empty() {
                            return *valid_moves[0];
                        } else {
                            from = Some(clicked_sq);
                            moves_from = moves
                                .iter()
                                .filter_map(|&m| if m.from == clicked_sq { Some(m) } else { None })
                                .collect();
                        }
                    } else {
                        from = Some(clicked_sq);
                        moves_from = moves_from
                            .iter()
                            .filter_map(|&m| if m.from == clicked_sq { Some(m) } else { None })
                            .collect();
                    }
                }
            }
            let possible_to_squares = if moves_from != moves {
                moves_from.iter().map(|m| m.to).collect()
            } else {
                Vec::new()
            };
            display_board(board, previous_move_squares, from, possible_to_squares);
        }
    }
}
