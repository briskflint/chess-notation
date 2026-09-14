//! Parsing for standard algebraic notation (SAN) chess move text.
//!
//! This crate handles the shape of a move string - piece, disambiguation,
//! capture marker, destination square, promotion, and check/mate suffix -
//! without knowing anything about a board position. A string like "Nbd7"
//! or "exd8=Q+" parses into a structured `SanMove`; whether that move is
//! actually legal in some game is out of scope here.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    fn from_char(c: char) -> Option<Piece> {
        match c {
            'N' => Some(Piece::Knight),
            'B' => Some(Piece::Bishop),
            'R' => Some(Piece::Rook),
            'Q' => Some(Piece::Queen),
            'K' => Some(Piece::King),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        }
    }
}

/// A board square, stored as zero-based file (a=0..h=7) and rank (1=0..8=7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    pub file: u8,
    pub rank: u8,
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Option<Square> {
        if file < 8 && rank < 8 {
            Some(Square { file, rank })
        } else {
            None
        }
    }

    /// Parses a two-character coordinate like "e4". Anything else, including
    /// off-board files/ranks or extra characters, returns `None`.
    pub fn from_coord(s: &str) -> Option<Square> {
        let mut chars = s.chars();
        let file_ch = chars.next()?;
        let rank_ch = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        let file = match file_ch {
            'a'..='h' => file_ch as u8 - b'a',
            _ => return None,
        };
        let rank = match rank_ch {
            '1'..='8' => rank_ch as u8 - b'1',
            _ => return None,
        };
        Some(Square { file, rank })
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", (b'a' + self.file) as char, self.rank + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleSide {
    Kingside,
    Queenside,
}

/// A parsed SAN move. Castling moves leave `to` as `None` since the
/// destination depends on which side is moving, which SAN text alone
/// doesn't say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanMove {
    pub piece: Piece,
    pub from_file: Option<u8>,
    pub from_rank: Option<u8>,
    pub capture: bool,
    pub to: Option<Square>,
    pub promotion: Option<Piece>,
    pub check: bool,
    pub checkmate: bool,
    pub castle: Option<CastleSide>,
}

impl fmt::Display for SanMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(side) = self.castle {
            let base = match side {
                CastleSide::Kingside => "O-O",
                CastleSide::Queenside => "O-O-O",
            };
            write!(f, "{base}")?;
        } else {
            if self.piece != Piece::Pawn {
                write!(f, "{}", self.piece.to_char())?;
            }
            if let Some(file) = self.from_file {
                write!(f, "{}", (b'a' + file) as char)?;
            }
            if let Some(rank) = self.from_rank {
                write!(f, "{}", rank + 1)?;
            }
            if self.capture {
                write!(f, "x")?;
            }
            if let Some(to) = self.to {
                write!(f, "{to}")?;
            }
            if let Some(promo) = self.promotion {
                write!(f, "={}", promo.to_char())?;
            }
        }
        if self.checkmate {
            write!(f, "#")?;
        } else if self.check {
            write!(f, "+")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    UnknownPiece(char),
    BadSquare(String),
    BadPromotion(char),
    TrailingGarbage(String),
    TooShort,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty move text"),
            ParseError::UnknownPiece(c) => write!(f, "unknown piece letter '{c}'"),
            ParseError::BadSquare(s) => write!(f, "not a valid square: '{s}'"),
            ParseError::BadPromotion(c) => write!(f, "invalid promotion piece '{c}'"),
            ParseError::TrailingGarbage(s) => write!(f, "unexpected trailing text: '{s}'"),
            ParseError::TooShort => write!(f, "move text too short"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a single SAN move (no move numbers, no annotations like "!?").
pub fn parse_san(input: &str) -> Result<SanMove, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    // Check/checkmate suffix is stripped first since it can trail either a
    // normal move or a castle.
    let (body, check, checkmate) = if let Some(stripped) = input.strip_suffix('#') {
        (stripped, false, true)
    } else if let Some(stripped) = input.strip_suffix('+') {
        (stripped, true, false)
    } else {
        (input, false, false)
    };

    if body == "O-O" || body == "0-0" {
        return Ok(SanMove {
            piece: Piece::King,
            from_file: None,
            from_rank: None,
            capture: false,
            to: None,
            promotion: None,
            check,
            checkmate,
            castle: Some(CastleSide::Kingside),
        });
    }
    if body == "O-O-O" || body == "0-0-0" {
        return Ok(SanMove {
            piece: Piece::King,
            from_file: None,
            from_rank: None,
            capture: false,
            to: None,
            promotion: None,
            check,
            checkmate,
            castle: Some(CastleSide::Queenside),
        });
    }

    let chars: Vec<char> = body.chars().collect();
    if chars.is_empty() {
        return Err(ParseError::TooShort);
    }

    let mut idx = 0;
    let piece = if chars[0].is_ascii_uppercase() {
        let p = Piece::from_char(chars[0]).ok_or(ParseError::UnknownPiece(chars[0]))?;
        idx += 1;
        p
    } else {
        Piece::Pawn
    };

    // Optional promotion suffix "=Q" at the end.
    let mut promotion = None;
    let mut end = chars.len();
    if end >= 2 && chars[end - 2] == '=' {
        let promo_char = chars[end - 1];
        let p = Piece::from_char(promo_char).ok_or(ParseError::BadPromotion(promo_char))?;
        if p == Piece::King {
            return Err(ParseError::BadPromotion(promo_char));
        }
        promotion = Some(p);
        end -= 2;
    }

    if end < idx + 2 {
        return Err(ParseError::TooShort);
    }

    // The destination square is always the last two characters remaining
    // once the piece letter and any promotion suffix are stripped off.
    let dest_str: String = chars[end - 2..end].iter().collect();
    let to = Square::from_coord(&dest_str).ok_or(ParseError::BadSquare(dest_str))?;

    // What's left between the piece letter and the destination is an
    // optional capture marker plus up to two disambiguation characters.
    let middle = &chars[idx..end - 2];
    let mut capture = false;
    let mut disambig: Vec<char> = Vec::new();
    for &c in middle {
        if c == 'x' {
            capture = true;
        } else {
            disambig.push(c);
        }
    }

    if disambig.len() > 2 {
        let s: String = disambig.iter().collect();
        return Err(ParseError::TrailingGarbage(s));
    }

    let mut from_file = None;
    let mut from_rank = None;
    for c in disambig {
        match c {
            'a'..='h' => from_file = Some(c as u8 - b'a'),
            '1'..='8' => from_rank = Some(c as u8 - b'1'),
            other => return Err(ParseError::TrailingGarbage(other.to_string())),
        }
    }

    Ok(SanMove {
        piece,
        from_file,
        from_rank,
        capture,
        to: Some(to),
        promotion,
        check,
        checkmate,
        castle: None,
    })
}
