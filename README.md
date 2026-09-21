# chess-notation

A small Rust library (with a thin CLI on top) for parsing standard algebraic
notation (SAN) - the move text you see in PGN files and chess books, like
`Nf3`, `exd5`, `O-O`, or `exd8=Q+`.

## Why this is harder than it looks

SAN packs a lot into a few characters, and the rules for what's optional
depend on context:

- A move is usually `<piece><disambiguation><capture><destination><promotion><check>`,
  but pawns omit the piece letter, and most fields are optional.
- Disambiguation can be a source file (`Nbd7`), a source rank (`R1a3`), or
  both (`Qh4e1`), depending on how many pieces of that type could reach the
  destination.
- Pawn captures need a source file even though no other pawn move does
  (`exd5`, not `xd5`).
- Castling (`O-O`, `O-O-O`) doesn't fit the piece/destination shape at all,
  and can still carry a check or mate suffix (`O-O+`).
- `+` and `#` are mutually exclusive and always trail everything else,
  including a promotion.

This library turns that text into a structured `SanMove` and gives back a
specific parse error when it can't, rather than a generic "invalid move."

It also parses FEN (Forsyth-Edwards Notation), the standard way to write
down a full board position - piece placement, side to move, castling
rights, en passant target, halfmove clock, and fullmove number:

```rust
use chess_notation::parse_fen;

let board = parse_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
assert_eq!(board.fullmove_number, 1);
assert_eq!(board.en_passant.unwrap().to_string(), "e3");
```

## What it does not do

SAN parsing here handles the *shape* of move text; FEN parsing handles the
*shape* of a position. Nothing yet connects the two - resolving a SAN
move's disambiguation against a `Board`'s actual piece placement, checking
legality, or generating SAN from a move plus position. That's the natural
next step.

## Usage

As a library:

```rust
use chess_notation::parse_san;

let mv = parse_san("exd8=Q+").unwrap();
assert!(mv.capture);
assert_eq!(mv.promotion, Some(chess_notation::Piece::Queen));
assert!(mv.check);
assert_eq!(mv.to_string(), "exd8=Q+");
```

From the command line:

```
$ cargo run -- e4 Nbd7 O-O exd8=Q+ e9
e4         piece=Pawn capture=false to=e4 check=false checkmate=false canonical=e4
Nbd7       piece=Knight capture=false to=d7 check=false checkmate=false canonical=Nbd7
O-O        piece=King capture=false to=- check=false checkmate=false canonical=O-O
exd8=Q+    piece=Pawn capture=true to=d8 check=true checkmate=false canonical=exd8=Q+
e9         error: not a valid square: 'e9'
```

The CLI exits non-zero if any argument fails to parse.

## Testing

`tests/san_parsing.rs` is a table-driven suite: one array of cases covering
the awkward corners above (file/rank/full disambiguation, pawn captures,
promotion combined with capture, castling with check, and a handful of
malformed inputs), checked against every field of the parsed result plus a
round trip back through `Display`.

`tests/fen_parsing.rs` follows the same pattern for board positions: a few
full games' worth of FEN strings checked field by field plus spot checks on
individual squares, a `Display` round trip, and a set of malformed FENs
(wrong field count, wrong rank count, ranks that are too long or short,
invalid piece letters, bad castling/en-passant/move-counter fields).

## Status

Early. No third-party dependencies - standard library only.
