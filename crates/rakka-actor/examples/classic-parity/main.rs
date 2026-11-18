//! CLI mirroring the Java oracle (`oracle/src/main/java/pilot/Cli.java`):
//!
//! ```text
//! classic-parity path-from-string         stdin = one JSON string literal  -> one JSON result
//! classic-parity path-from-string-lines   stdin = one JSON string literal per line -> one JSON result per line
//! classic-parity scenario                 stdin = one scenario JSON (see `scenario`) -> one JSON trace
//! classic-parity scenario-lines           stdin = one scenario JSON per line -> one JSON trace per line
//! ```

mod bytes_cases;
mod fronteras_cases;
mod json;
mod path_address_cases;
mod path_case;
mod scenario;
mod structures_cases;
mod util_core_cases;

use std::io::{Read, Write};
use std::process::ExitCode;

use json::parse_str;
use path_case::path_from_string;

fn usage() -> ExitCode {
    eprintln!(
        "usage: classic-parity path-from-string | path-from-string-lines | scenario | scenario-lines | path-address-case | util-core-case | util-core-all   (JSON on stdin)"
    );
    ExitCode::from(2)
}

/// `<family>-case`: stdin is one oracle case id (ASCII-escaped); prints the line Rust produces for it.
fn oracle_case(out: &mut impl Write, input: &str, lines: Vec<(String, String)>) -> ExitCode {
    let id = input.strip_suffix('\n').unwrap_or(input);
    match lines.iter().find(|(case, _)| case == id) {
        Some((_, line)) => {
            writeln!(out, "{line}").ok();
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("no Rust case for id: {id}");
            ExitCode::from(2)
        }
    }
}

/// `<family>-ids`: the case ids Rust implements (the oracle lists its own with `<family>-ids` too).
fn print_ids(out: &mut impl Write, lines: Vec<(String, String)>) {
    for (id, _) in lines {
        writeln!(out, "{id}").ok();
    }
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
        "path-address-ids" => print_ids(&mut out, path_address_cases::rust_lines()),
        "util-core-ids" => print_ids(&mut out, util_core_cases::rust_lines()),
        "util-core-all" => {
            for (_, line) in util_core_cases::rust_lines() {
                writeln!(out, "{line}").ok();
            }
        }
        "path-address-case" => {
            return oracle_case(&mut out, &input, path_address_cases::rust_lines());
        }
        "util-core-case" => return oracle_case(&mut out, &input, util_core_cases::rust_lines()),
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
