//! FEN (Forsyth-Edwards Notation) board parsing.
//!
//! FEN describes a full board position as a single line of text: piece
//! placement, side to move, castling rights, en passant target square,
//! halfmove clock, and fullmove number. This is the piece needed to resolve
//! SAN move text (disambiguation, legality) against an actual position,
//! rather than just parsing its shape.

use std::fmt;

use crate::{Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    fn from_char(c: char) -> Option<Color> {
        match c {
            'w' => Some(Color::White),
            'b' => Some(Color::Black),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == CastlingRights::default() {
            return write!(f, "-");
        }
        if self.white_kingside {
            write!(f, "K")?;
        }
        if self.white_queenside {
            write!(f, "Q")?;
        }
        if self.black_kingside {
            write!(f, "k")?;
        }
        if self.black_queenside {
            write!(f, "q")?;
        }
        Ok(())
    }
}

/// A full board position as described by a FEN string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// Indexed `[rank][file]`, both zero-based (rank 0 = rank 1, file 0 = 'a').
    squares: [[Option<(Piece, Color)>; 8]; 8],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.squares[square.rank as usize][square.file as usize]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            let mut empty_run = 0;
            for file in 0..8 {
                match self.squares[rank][file] {
                    None => empty_run += 1,
                    Some((piece, color)) => {
                        if empty_run > 0 {
                            write!(f, "{empty_run}")?;
                            empty_run = 0;
                        }
                        write!(f, "{}", piece_to_fen_char(piece, color))?;
                    }
                }
            }
            if empty_run > 0 {
                write!(f, "{empty_run}")?;
            }
            if rank > 0 {
                write!(f, "/")?;
            }
        }
        write!(
            f,
            " {} {} {} {} {}",
            self.side_to_move.to_char(),
            self.castling,
            self.en_passant
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string()),
            self.halfmove_clock,
            self.fullmove_number,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    WrongFieldCount(usize),
    WrongRankCount(usize),
    RankTooLong(String),
    RankTooShort(String),
    InvalidPieceChar(char),
    BadSideToMove(String),
    BadCastling(String),
    BadEnPassant(String),
    BadHalfmoveClock(String),
    BadFullmoveNumber(String),
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::WrongFieldCount(n) => {
                write!(f, "expected 6 space-separated fields, found {n}")
            }
            FenError::WrongRankCount(n) => write!(f, "expected 8 ranks, found {n}"),
            FenError::RankTooLong(s) => write!(f, "rank has more than 8 files: '{s}'"),
            FenError::RankTooShort(s) => write!(f, "rank has fewer than 8 files: '{s}'"),
            FenError::InvalidPieceChar(c) => write!(f, "invalid piece character '{c}'"),
            FenError::BadSideToMove(s) => write!(f, "invalid side to move: '{s}'"),
            FenError::BadCastling(s) => write!(f, "invalid castling rights: '{s}'"),
            FenError::BadEnPassant(s) => write!(f, "invalid en passant square: '{s}'"),
            FenError::BadHalfmoveClock(s) => write!(f, "invalid halfmove clock: '{s}'"),
            FenError::BadFullmoveNumber(s) => write!(f, "invalid fullmove number: '{s}'"),
        }
    }
}

impl std::error::Error for FenError {}

fn piece_from_fen_char(c: char) -> Option<(Piece, Color)> {
    let color = if c.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };
    let piece = match c.to_ascii_uppercase() {
        'P' => Piece::Pawn,
        'N' => Piece::Knight,
        'B' => Piece::Bishop,
        'R' => Piece::Rook,
        'Q' => Piece::Queen,
        'K' => Piece::King,
        _ => return None,
    };
    Some((piece, color))
}

fn piece_to_fen_char(piece: Piece, color: Color) -> char {
    let ch = match piece {
        Piece::Pawn => 'p',
        Piece::Knight => 'n',
        Piece::Bishop => 'b',
        Piece::Rook => 'r',
        Piece::Queen => 'q',
        Piece::King => 'k',
    };
    match color {
        Color::White => ch.to_ascii_uppercase(),
        Color::Black => ch,
    }
}

/// Parses a full FEN string (all six fields) into a `Board`.
pub fn parse_fen(input: &str) -> Result<Board, FenError> {
    let fields: Vec<&str> = input.split_whitespace().collect();
    if fields.len() != 6 {
        return Err(FenError::WrongFieldCount(fields.len()));
    }

    let squares = parse_piece_placement(fields[0])?;

    if fields[1].chars().count() != 1 {
        return Err(FenError::BadSideToMove(fields[1].to_string()));
    }
    let side_to_move = Color::from_char(fields[1].chars().next().unwrap())
        .ok_or_else(|| FenError::BadSideToMove(fields[1].to_string()))?;

    let castling = parse_castling(fields[2])?;

    let en_passant = if fields[3] == "-" {
        None
    } else {
        Some(
            Square::from_coord(fields[3])
                .ok_or_else(|| FenError::BadEnPassant(fields[3].to_string()))?,
        )
    };

    let halfmove_clock = fields[4]
        .parse::<u32>()
        .map_err(|_| FenError::BadHalfmoveClock(fields[4].to_string()))?;

    let fullmove_number = fields[5]
        .parse::<u32>()
        .map_err(|_| FenError::BadFullmoveNumber(fields[5].to_string()))?;
    if fullmove_number == 0 {
        return Err(FenError::BadFullmoveNumber(fields[5].to_string()));
    }

    Ok(Board {
        squares,
        side_to_move,
        castling,
        en_passant,
        halfmove_clock,
        fullmove_number,
    })
}

fn parse_piece_placement(field: &str) -> Result<[[Option<(Piece, Color)>; 8]; 8], FenError> {
    let ranks: Vec<&str> = field.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::WrongRankCount(ranks.len()));
    }

    let mut squares = [[None; 8]; 8];
    for (i, rank_str) in ranks.iter().enumerate() {
        // FEN lists ranks from 8 down to 1; our internal rank is 0-based from 1.
        let rank = 7 - i;
        let mut file = 0usize;
        for c in rank_str.chars() {
            if let Some(skip) = c.to_digit(10) {
                if skip == 0 || file + skip as usize > 8 {
                    return Err(FenError::RankTooLong(rank_str.to_string()));
                }
                file += skip as usize;
            } else {
                let piece = piece_from_fen_char(c).ok_or(FenError::InvalidPieceChar(c))?;
                if file >= 8 {
                    return Err(FenError::RankTooLong(rank_str.to_string()));
                }
                squares[rank][file] = Some(piece);
                file += 1;
            }
        }
        if file != 8 {
            return Err(FenError::RankTooShort(rank_str.to_string()));
        }
    }

    Ok(squares)
}

fn parse_castling(field: &str) -> Result<CastlingRights, FenError> {
    if field == "-" {
        return Ok(CastlingRights::default());
    }

    let mut rights = CastlingRights::default();
    for c in field.chars() {
        match c {
            'K' if !rights.white_kingside => rights.white_kingside = true,
            'Q' if !rights.white_queenside => rights.white_queenside = true,
            'k' if !rights.black_kingside => rights.black_kingside = true,
            'q' if !rights.black_queenside => rights.black_queenside = true,
            _ => return Err(FenError::BadCastling(field.to_string())),
        }
    }
    Ok(rights)
}
