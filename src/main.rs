use std::env;
use std::process::ExitCode;

use chess_notation::parse_san;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: chess-notation <move> [<move> ...]");
        eprintln!("example: chess-notation e4 Nf3 O-O Qh5+ exd8=Q#");
        return ExitCode::FAILURE;
    }

    let mut had_error = false;
    for arg in &args {
        match parse_san(arg) {
            Ok(mv) => {
                let to_str = mv
                    .to
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".to_string());
                println!(
                    "{arg:<10} piece={:?} capture={} to={to_str} check={} checkmate={} canonical={mv}",
                    mv.piece, mv.capture, mv.check, mv.checkmate,
                );
            }
            Err(err) => {
                eprintln!("{arg:<10} error: {err}");
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
