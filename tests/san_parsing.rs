use chess_notation::{parse_san, CastleSide, Piece, Square};

struct Case {
    input: &'static str,
    piece: Piece,
    from_file: Option<u8>,
    from_rank: Option<u8>,
    capture: bool,
    to: Option<(u8, u8)>,
    promotion: Option<Piece>,
    check: bool,
    checkmate: bool,
    castle: Option<CastleSide>,
}

// Each row is a case that has tripped up a naive "letter + digit" parser at
// some point: disambiguation by file, by rank, by both, pawn captures that
// need a source file, promotion combined with capture, and castling with a
// check suffix stuck on the end.
const CASES: &[Case] = &[
    Case {
        input: "e4",
        piece: Piece::Pawn,
        from_file: None,
        from_rank: None,
        capture: false,
        to: Some((4, 3)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "Nf3",
        piece: Piece::Knight,
        from_file: None,
        from_rank: None,
        capture: false,
        to: Some((5, 2)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "exd5",
        piece: Piece::Pawn,
        from_file: Some(4),
        from_rank: None,
        capture: true,
        to: Some((3, 4)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "Nxe5",
        piece: Piece::Knight,
        from_file: None,
        from_rank: None,
        capture: true,
        to: Some((4, 4)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "Nbd7",
        piece: Piece::Knight,
        from_file: Some(1),
        from_rank: None,
        capture: false,
        to: Some((3, 6)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "R1a3",
        piece: Piece::Rook,
        from_file: None,
        from_rank: Some(0),
        capture: false,
        to: Some((0, 2)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "Qh4e1",
        piece: Piece::Queen,
        from_file: Some(7),
        from_rank: Some(3),
        capture: false,
        to: Some((4, 0)),
        promotion: None,
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "e8=Q",
        piece: Piece::Pawn,
        from_file: None,
        from_rank: None,
        capture: false,
        to: Some((4, 7)),
        promotion: Some(Piece::Queen),
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "exd8=N",
        piece: Piece::Pawn,
        from_file: Some(4),
        from_rank: None,
        capture: true,
        to: Some((3, 7)),
        promotion: Some(Piece::Knight),
        check: false,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "Qh5+",
        piece: Piece::Queen,
        from_file: None,
        from_rank: None,
        capture: false,
        to: Some((7, 4)),
        promotion: None,
        check: true,
        checkmate: false,
        castle: None,
    },
    Case {
        input: "Qh5#",
        piece: Piece::Queen,
        from_file: None,
        from_rank: None,
        capture: false,
        to: Some((7, 4)),
        promotion: None,
        check: false,
        checkmate: true,
        castle: None,
    },
    Case {
        input: "O-O",
        piece: Piece::King,
        from_file: None,
        from_rank: None,
        capture: false,
        to: None,
        promotion: None,
        check: false,
        checkmate: false,
        castle: Some(CastleSide::Kingside),
    },
    Case {
        input: "O-O-O",
        piece: Piece::King,
        from_file: None,
        from_rank: None,
        capture: false,
        to: None,
        promotion: None,
        check: false,
        checkmate: false,
        castle: Some(CastleSide::Queenside),
    },
    Case {
        input: "O-O+",
        piece: Piece::King,
        from_file: None,
        from_rank: None,
        capture: false,
        to: None,
        promotion: None,
        check: true,
        checkmate: false,
        castle: Some(CastleSide::Kingside),
    },
];

#[test]
fn parses_well_formed_moves() {
    for case in CASES {
        let mv = parse_san(case.input)
            .unwrap_or_else(|e| panic!("{}: expected Ok, got Err({e})", case.input));
        assert_eq!(mv.piece, case.piece, "{}: piece", case.input);
        assert_eq!(mv.from_file, case.from_file, "{}: from_file", case.input);
        assert_eq!(mv.from_rank, case.from_rank, "{}: from_rank", case.input);
        assert_eq!(mv.capture, case.capture, "{}: capture", case.input);
        assert_eq!(
            mv.to,
            case.to.map(|(file, rank)| Square { file, rank }),
            "{}: to",
            case.input
        );
        assert_eq!(mv.promotion, case.promotion, "{}: promotion", case.input);
        assert_eq!(mv.check, case.check, "{}: check", case.input);
        assert_eq!(mv.checkmate, case.checkmate, "{}: checkmate", case.input);
        assert_eq!(mv.castle, case.castle, "{}: castle", case.input);
    }
}

#[test]
fn round_trips_through_display() {
    // Reformatting a parsed move should reproduce the same canonical text,
    // even for the awkward disambiguated and promoted cases.
    for case in CASES {
        let mv = parse_san(case.input).unwrap();
        assert_eq!(mv.to_string(), case.input, "{}: round trip", case.input);
    }
}

#[test]
fn rejects_malformed_moves() {
    let cases = [
        "",             // nothing to parse
        "+",            // just a check suffix
        "e9",           // rank out of range
        "j4",           // file out of range
        "Zf3",          // not a real piece letter
        "N",            // piece with no destination
        "e",            // pawn move with no destination
        "Qh4e1e2",      // more disambiguation than SAN allows
        "e8=P",         // can't promote to a pawn
        "e8=K",         // can't promote to a king
    ];

    for input in cases {
        assert!(
            parse_san(input).is_err(),
            "{input}: expected Err, got Ok"
        );
    }
}
