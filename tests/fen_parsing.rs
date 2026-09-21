use chess_notation::{parse_fen, CastlingRights, Color, Piece, Square};

struct Case {
    input: &'static str,
    side_to_move: Color,
    castling: CastlingRights,
    en_passant: Option<(u8, u8)>,
    halfmove_clock: u32,
    fullmove_number: u32,
    // A handful of (coord, expected piece/color) spot checks per position,
    // rather than asserting all 64 squares.
    spots: &'static [(&'static str, Option<(Piece, Color)>)],
}

const CASES: &[Case] = &[
    Case {
        input: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        side_to_move: Color::White,
        castling: CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        },
        en_passant: None,
        halfmove_clock: 0,
        fullmove_number: 1,
        spots: &[
            ("a1", Some((Piece::Rook, Color::White))),
            ("e1", Some((Piece::King, Color::White))),
            ("e8", Some((Piece::King, Color::Black))),
            ("d8", Some((Piece::Queen, Color::Black))),
            ("e4", None),
        ],
    },
    Case {
        input: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        side_to_move: Color::Black,
        castling: CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        },
        en_passant: Some((4, 2)),
        halfmove_clock: 0,
        fullmove_number: 1,
        spots: &[
            ("e4", Some((Piece::Pawn, Color::White))),
            ("e2", None),
        ],
    },
    Case {
        input: "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        side_to_move: Color::Black,
        castling: CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        },
        en_passant: None,
        halfmove_clock: 1,
        fullmove_number: 2,
        spots: &[
            ("f3", Some((Piece::Knight, Color::White))),
            ("c5", Some((Piece::Pawn, Color::Black))),
            ("g1", None),
        ],
    },
    Case {
        input: "8/8/8/4k3/8/8/4K3/8 w - - 0 1",
        side_to_move: Color::White,
        castling: CastlingRights::default(),
        en_passant: None,
        halfmove_clock: 0,
        fullmove_number: 1,
        spots: &[
            ("e2", Some((Piece::King, Color::White))),
            ("e5", Some((Piece::King, Color::Black))),
            ("a1", None),
        ],
    },
];

#[test]
fn parses_well_formed_positions() {
    for case in CASES {
        let board = parse_fen(case.input)
            .unwrap_or_else(|e| panic!("{}: expected Ok, got Err({e})", case.input));
        assert_eq!(board.side_to_move, case.side_to_move, "{}: side_to_move", case.input);
        assert_eq!(board.castling, case.castling, "{}: castling", case.input);
        assert_eq!(
            board.en_passant,
            case.en_passant.map(|(file, rank)| Square { file, rank }),
            "{}: en_passant",
            case.input
        );
        assert_eq!(
            board.halfmove_clock, case.halfmove_clock,
            "{}: halfmove_clock",
            case.input
        );
        assert_eq!(
            board.fullmove_number, case.fullmove_number,
            "{}: fullmove_number",
            case.input
        );
        for (coord, expected) in case.spots {
            let square = Square::from_coord(coord).unwrap();
            assert_eq!(
                board.piece_at(square),
                *expected,
                "{}: piece at {coord}",
                case.input
            );
        }
    }
}

#[test]
fn round_trips_through_display() {
    for case in CASES {
        let board = parse_fen(case.input).unwrap();
        assert_eq!(board.to_string(), case.input, "{}: round trip", case.input);
    }
}

#[test]
fn rejects_malformed_positions() {
    let cases = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0", // missing fullmove field
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra", // too many fields
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1",        // only 7 ranks
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR/8 w KQkq - 0 1", // 9 ranks
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPP9/RNBQKBNR w KQkq - 0 1", // rank too long
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPP/RNBQKBNR w KQkq - 0 1", // rank too short
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPX/RNBQKBNR w KQkq - 0 1", // invalid piece char
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1", // bad side to move
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkqx - 0 1", // bad castling char
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KKQkq - 0 1", // duplicate castling char
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e9 0 1", // bad en passant square
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - x 1", // non-numeric halfmove clock
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0", // fullmove number can't be 0
    ];

    for input in cases {
        assert!(parse_fen(input).is_err(), "{input}: expected Err, got Ok");
    }
}
