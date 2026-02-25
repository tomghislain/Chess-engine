use crate::game;
use crossterm::{queue, style, terminal};
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

pub fn sq_to_top_left(sq: u8, term_width: u16, term_height: u16) -> (u16, u16) {
    let (a1_col, a1_row) = (
        (term_width - 8 * SQ_WIDTH) / 2,
        (term_height + (6 * SQ_HEIGHT)) / 2,
    );

    let (file, rank) = ((sq % 8) as u16, (sq / 8) as u16);
    let (col_offset, row_offset) = (file * SQ_WIDTH, rank * SQ_HEIGHT);
    let (col, row) = (a1_col + col_offset, a1_row - row_offset);

    (col, row)
}

pub fn display_board(board: &game::Board) {
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
        let sq_color = sq_colors[index];
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

pub fn get_player_move(board: &mut game::Board) -> game::Move {
    let legal_moves = game::get_legal_moves(board);
    legal_moves[0]
}
