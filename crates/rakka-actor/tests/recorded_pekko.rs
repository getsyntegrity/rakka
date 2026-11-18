//! Replays, in-process, every case recorded from real Pekko 2.0.0 (`rust/app-cases`) against the
//! library, so that the recorded behaviour also counts as verified by `cargo test`/`cargo llvm-cov`.
//! The harness code (JSON, scenario runner, path rendering) is shared with the `classic-parity` CLI.

#[path = "../examples/classic-parity/bytes_cases.rs"]
mod bytes_cases;
#[path = "../examples/classic-parity/json.rs"]
#[allow(dead_code)]
mod json;
#[path = "../examples/classic-parity/path_address_cases.rs"]
mod path_address_cases;
#[path = "../examples/classic-parity/path_case.rs"]
mod path_case;
#[path = "../examples/classic-parity/scenario.rs"]
mod scenario;
#[path = "../examples/classic-parity/structures_cases.rs"]
mod structures_cases;
#[path = "../examples/classic-parity/util_core_cases.rs"]
mod util_core_cases;

use std::fs;
use std::path::PathBuf;

struct Case {
    name: String,
    command: String,
    stdin: String,
    stdout: String,
}

fn cases(commands: &[&str]) -> Vec<Case> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rust/app-cases");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("rust/app-cases exists") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let toml = fs::read_to_string(&path).unwrap();
        let args = toml
            .lines()
            .find_map(|l| l.strip_prefix("args = [\""))
            .and_then(|r| r.split('"').next())
            .unwrap()
            .to_string();
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

/// Cases recorded from Pekko whose input is outside the API domain (`cases/not-applicable.tsv`, Q-X-17).
fn not_applicable() -> Vec<String> {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cases/not-applicable.tsv");
    fs::read_to_string(file)
        .unwrap()
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap().to_string())
        .collect()
}

fn path_out(input: &str, lines: bool) -> String {
    let render = |lit: &str| {
        path_case::path_from_string(&json::parse_str(lit).expect("valid JSON string literal"))
    };
    if lines {
        input
            .split('\n')
            .filter(|l| !l.is_empty())
            .map(|l| render(l) + "\n")
            .collect()
    } else {
        render(input.strip_suffix('\n').unwrap_or(input)) + "\n"
    }
}

#[test]
fn recorded_path_from_string_cases_match_pekko() {
    let all = cases(&["path-from-string", "path-from-string-lines"]);
    assert!(all.len() > 100, "recorded cases not found: {}", all.len());
    let out_of_domain = not_applicable();
    for c in &all {
        if out_of_domain.contains(&c.name) {
            // Not skipped: the harness must refuse the input explicitly instead of decoding it lossily.
            let error = json::parse_str(case_id(&c.stdin)).unwrap_err();
            assert_eq!(error, json::OUT_OF_DOMAIN, "case {}", c.name);
            continue;
        }
        assert_eq!(
            path_out(&c.stdin, c.command.ends_with("lines")),
            c.stdout,
            "case {}",
            c.name
        );
    }
    assert_eq!(
        all.iter()
            .filter(|c| out_of_domain.contains(&c.name))
            .count(),
        out_of_domain.len(),
        "stale NOT_APPLICABLE list"
    );
}

#[test]
fn recorded_scenario_cases_match_pekko() {
    let all = cases(&["scenario", "scenario-lines"]);
    assert!(all.len() > 200, "recorded cases not found: {}", all.len());
    for c in &all {
        let got: String = if c.command == "scenario" {
            scenario::run(&c.stdin).map(|t| t + "\n").unwrap()
        } else {
            c.stdin
                .split('\n')
                .filter(|l| !l.is_empty())
                .map(|l| scenario::run(l).unwrap() + "\n")
                .collect()
        };
        assert_eq!(got, c.stdout, "case {}", c.name);
    }
}

fn case_id(stdin: &str) -> &str {
    stdin.strip_suffix('\n').unwrap_or(stdin)
}

/// One recorded `<family>-case`: stdin is an oracle case id, stdout the line Pekko 2.0.0 printed for it.
fn oracle_family_matches_pekko(
    command: &str,
    ids_file: &str,
    lines: Vec<(String, String)>,
    unproduced: Vec<&str>,
) {
    let all = cases(&[command]);
    assert!(!all.is_empty(), "no recorded {command} cases");
    let produced: std::collections::HashMap<&str, &str> = lines
        .iter()
        .map(|(id, line)| (id.as_str(), line.as_str()))
        .collect();
    assert_eq!(produced.len(), lines.len(), "duplicate Rust case ids");
    for c in &all {
        let id = case_id(&c.stdin);
        let line = produced
            .get(id)
            .unwrap_or_else(|| panic!("no Rust case for {id}"));
        assert_eq!(format!("{line}\n"), c.stdout, "case {} ({id})", c.name);
    }
    // Every id the oracle has is either recorded or explicitly listed as not produced (PENDING / UNIT_TESTED).
    let oracle_ids = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../cases/oracle-ids")
            .join(ids_file),
    )
    .unwrap();
    let recorded: std::collections::HashSet<&str> = all.iter().map(|c| case_id(&c.stdin)).collect();
    for id in oracle_ids.lines() {
        assert!(
            recorded.contains(id) || unproduced.contains(&id),
            "oracle case {id} is neither recorded nor listed as not produced"
        );
    }
    assert_eq!(
        recorded.len() + unproduced.len(),
        oracle_ids.lines().count(),
        "stale not-produced list for {command}"
    );
}

#[test]
fn recorded_path_address_cases_match_pekko() {
    let mut unproduced: Vec<&str> = path_address_cases::PENDING
        .iter()
        .map(|(id, _)| *id)
        .collect();
    unproduced.extend(path_address_cases::UNIT_TESTED);
    oracle_family_matches_pekko(
        "path-address-case",
        "path-address.txt",
        path_address_cases::rust_lines(),
        unproduced,
    );
}

#[test]
fn recorded_util_core_cases_match_pekko() {
    let unproduced = util_core_cases::PENDING.iter().map(|(id, _)| *id).collect();
    oracle_family_matches_pekko(
        "util-core-case",
        "util-core.txt",
        util_core_cases::rust_lines(),
        unproduced,
    );
}
