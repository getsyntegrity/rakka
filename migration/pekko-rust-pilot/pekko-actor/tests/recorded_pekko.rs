//! Replays, in-process, every case recorded from real Pekko 2.0.0 (`rust/app-cases`) against the
//! library, so that the recorded behaviour also counts as verified by `cargo test`/`cargo llvm-cov`.
//! The harness code (JSON, scenario runner, path rendering) is shared with the `classic-parity` CLI.

#[path = "../examples/classic-parity/json.rs"]
#[allow(dead_code)]
mod json;
#[path = "../examples/classic-parity/path_case.rs"]
mod path_case;
#[path = "../examples/classic-parity/scenario.rs"]
mod scenario;

use std::fs;
use std::path::PathBuf;

struct Case {
    name: String,
    command: String,
    stdin: String,
    stdout: String,
}

fn cases(commands: &[&str]) -> Vec<Case> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rust/app-cases");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("rust/app-cases exists") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let toml = fs::read_to_string(&path).unwrap();
        let args = toml.lines().find_map(|l| l.strip_prefix("args = [\"")).and_then(|r| r.split('"').next()).unwrap().to_string();
        if !commands.contains(&args.as_str()) || !toml.contains("status.code = 0") {
            continue;
        }
        let base = path.with_extension("");
        out.push(Case {
            name: base.file_name().unwrap().to_string_lossy().into_owned(),
            command: args,
            stdin: fs::read_to_string(base.with_extension("stdin")).unwrap(),
            stdout: fs::read_to_string(base.with_extension("stdout")).unwrap(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn path_out(input: &str, lines: bool) -> String {
    let render = |lit: &str| path_case::path_from_string(&json::parse_str(lit).expect("valid JSON string literal"));
    if lines {
        input.split('\n').filter(|l| !l.is_empty()).map(|l| render(l) + "\n").collect()
    } else {
        render(input.strip_suffix('\n').unwrap_or(input)) + "\n"
    }
}

#[test]
fn recorded_path_from_string_cases_match_pekko() {
    let all = cases(&["path-from-string", "path-from-string-lines"]);
    assert!(all.len() > 100, "recorded cases not found: {}", all.len());
    for c in &all {
        assert_eq!(path_out(&c.stdin, c.command.ends_with("lines")), c.stdout, "case {}", c.name);
    }
}

#[test]
fn recorded_scenario_cases_match_pekko() {
    let all = cases(&["scenario", "scenario-lines"]);
    assert!(all.len() > 200, "recorded cases not found: {}", all.len());
    for c in &all {
        let got: String = if c.command == "scenario" {
            scenario::run(&c.stdin).map(|t| t + "\n").unwrap()
        } else {
            c.stdin.split('\n').filter(|l| !l.is_empty()).map(|l| scenario::run(l).unwrap() + "\n").collect()
        };
        assert_eq!(got, c.stdout, "case {}", c.name);
    }
}
