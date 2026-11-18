//! CLI mirroring the Java oracle (`oracle/src/main/java/pilot/Cli.java`):
//!
//! ```text
//! classic-parity path-from-string         stdin = one JSON string literal  -> one JSON result
//! classic-parity path-from-string-lines   stdin = one JSON string literal per line -> one JSON result per line
//! classic-parity scenario                 stdin = one scenario JSON (see `scenario`) -> one JSON trace
//! classic-parity scenario-lines           stdin = one scenario JSON per line -> one JSON trace per line
//! ```

mod json;
mod path_case;
mod scenario;

use std::io::{Read, Write};
use std::process::ExitCode;

use json::parse_str;
use path_case::path_from_string;

fn usage() -> ExitCode {
    eprintln!("usage: classic-parity path-from-string | path-from-string-lines | scenario | scenario-lines   (JSON on stdin)");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return usage();
    }
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("stdin is not valid UTF-8");
        return ExitCode::from(2);
    }
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match args[0].as_str() {
        "scenario-lines" => {
            for line in input.split('\n').filter(|l| !l.is_empty()) {
                match scenario::run(line) {
                    Ok(trace) => {
                        writeln!(out, "{trace}").ok();
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        return ExitCode::from(2);
                    }
                }
            }
        }
        "scenario" => match scenario::run(&input) {
            Ok(trace) => {
                writeln!(out, "{trace}").ok();
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(2);
            }
        },
        "path-from-string" => {
            let lit = input.strip_suffix('\n').unwrap_or(&input);
            match parse_str(lit) {
                Ok(s) => writeln!(out, "{}", path_from_string(&s)).ok(),
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(2);
                }
            };
        }
        "path-from-string-lines" => {
            let mut buf = String::new();
            for line in input.split('\n').filter(|l| !l.is_empty()) {
                match parse_str(line) {
                    Ok(s) => {
                        buf.push_str(&path_from_string(&s));
                        buf.push('\n');
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        return ExitCode::from(2);
                    }
                }
            }
            out.write_all(buf.as_bytes()).ok();
        }
        _ => return usage(),
    }
    out.flush().ok();
    ExitCode::SUCCESS
}
