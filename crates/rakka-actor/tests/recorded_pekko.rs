//! Replays, in-process, every case recorded from real Pekko 2.0.0 (`rust/app-cases`) against the
//! library, so that the recorded behaviour also counts as verified by `cargo test`/`cargo llvm-cov`.
//! The harness code (JSON, scenario runner, path rendering) is shared with the `classic-parity` CLI.

#[path = "../examples/classic-parity/actor_system_setup_cases.rs"]
mod actor_system_setup_cases;
#[path = "../examples/classic-parity/bytes_cases.rs"]
mod bytes_cases;
#[path = "../examples/classic-parity/child_names.rs"]
mod child_names;
#[path = "../examples/classic-parity/config_hocon_cases.rs"]
mod config_hocon_cases;
#[path = "../examples/classic-parity/config_settings_cases.rs"]
mod config_settings_cases;
#[path = "../examples/classic-parity/deploy_cases.rs"]
mod deploy_cases;
#[path = "../examples/classic-parity/deployer_cases.rs"]
mod deployer_cases;
#[path = "../examples/classic-parity/deployer_settings_cases.rs"]
mod deployer_settings_cases;
#[path = "../examples/classic-parity/dynamic_access_cases.rs"]
mod dynamic_access_cases;
#[path = "../examples/classic-parity/fronteras_cases.rs"]
mod fronteras_cases;
#[path = "../examples/classic-parity/guardian_strategies.rs"]
mod guardian_strategies;
#[path = "../examples/classic-parity/json.rs"]
#[allow(dead_code)]
mod json;
#[path = "../examples/classic-parity/mailbox_cases.rs"]
mod mailbox_cases;
#[path = "../examples/classic-parity/path_address_cases.rs"]
mod path_address_cases;
#[path = "../examples/classic-parity/path_case.rs"]
mod path_case;
#[path = "../examples/classic-parity/provider_selection_cases.rs"]
mod provider_selection_cases;
#[path = "../examples/classic-parity/scenario.rs"]
mod scenario;
#[path = "../examples/classic-parity/scheduler_cases.rs"]
#[allow(dead_code)] // the CLI also lists the ids
mod scheduler_cases;
#[path = "../examples/classic-parity/structures_cases.rs"]
mod structures_cases;
#[path = "../examples/classic-parity/text_escape.rs"]
mod text_escape;
#[path = "../examples/classic-parity/util_core_cases.rs"]
mod util_core_cases;
#[path = "../examples/classic-parity/xorshift.rs"]
mod xorshift;

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
        let produced_line = format!("{line}\n");
        assert!(
            produced_line == c.stdout || only_jdk17_double_text_differs(&c.stdout, &produced_line),
            "case {} ({id}): expected {:?}, got {:?}",
            c.name,
            c.stdout,
            produced_line
        );
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

/// A decimal number as `Double.toString` writes it: `-?D+.D+(E-?D+)?`, not glued to a letter, digit or point.
fn double_texts(text: &str) -> (Vec<&str>, Vec<&str>) {
    let bytes = text.as_bytes();
    let (mut pieces, mut numbers, mut piece_start, mut i) = (Vec::new(), Vec::new(), 0, 0);
    while i < bytes.len() {
        let glued_before = i > 0
            && (bytes[i - 1].is_ascii_alphanumeric()
                || bytes[i - 1] == b'_'
                || bytes[i - 1] == b'.');
        let start = i;
        let mut j = if bytes[i] == b'-' { i + 1 } else { i };
        let digits_from = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        let whole_len = j - digits_from;
        if glued_before || whole_len == 0 || j >= bytes.len() || bytes[j] != b'.' {
            i += 1;
            continue;
        }
        let fraction_from = j + 1;
        j = fraction_from;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j == fraction_from {
            i += 1;
            continue;
        }
        if j < bytes.len() && bytes[j] == b'E' {
            let mut k = j + 1;
            if k < bytes.len() && bytes[k] == b'-' {
                k += 1;
            }
            let power_from = k;
            while k < bytes.len() && bytes[k].is_ascii_digit() {
                k += 1;
            }
            if k > power_from {
                j = k;
            }
        }
        if j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'.') {
            i += 1;
            continue;
        }
        pieces.push(&text[piece_start..start]);
        numbers.push(&text[start..j]);
        piece_start = j;
        i = j;
    }
    pieces.push(&text[piece_start..]);
    (pieces, numbers)
}

/// Significant digits (no leading or trailing zeros) and decimal exponent of the first one.
fn significant_digits(number: &str) -> (String, i32) {
    let unsigned = number.trim_start_matches('-');
    let (mantissa, power) = unsigned.split_once('E').unwrap_or((unsigned, "0"));
    let (whole, fraction) = mantissa.split_once('.').expect("a point");
    let all = format!("{whole}{fraction}");
    let leading = all.len() - all.trim_start_matches('0').len();
    let exponent =
        whole.len() as i32 - 1 - leading as i32 + power.parse::<i32>().expect("exponent");
    (all.trim_matches('0').to_owned(), exponent)
}

/// Q-CFG-30, in-process version of `tools/parity/na_q_cfg_30.py`: the recorded JDK 17 output and ours differ in exactly
/// one number, both identify the same double, share the layout, and JDK 17 shows 16 or 17 digits (JDK 17 sometimes
/// prints a text that is not the shortest closest one; JDK 19+ and rakka do). The replay (`replay.py`) applies the
/// full signature with an independent shortest-digits computation; this test only needs to tell that case from a real
/// difference without a second implementation of the algorithm.
fn only_jdk17_double_text_differs(oracle: &str, rakka: &str) -> bool {
    let ((oracle_pieces, oracle_numbers), (rakka_pieces, rakka_numbers)) =
        (double_texts(oracle), double_texts(rakka));
    if oracle_pieces != rakka_pieces || oracle_numbers.len() != rakka_numbers.len() {
        return false;
    }
    let differing: Vec<_> = oracle_numbers
        .iter()
        .zip(&rakka_numbers)
        .filter(|(o, r)| o != r)
        .collect();
    let [(oracle_number, rakka_number)] = differing[..] else {
        return false;
    };
    let same_double = match (oracle_number.parse::<f64>(), rakka_number.parse::<f64>()) {
        (Ok(o), Ok(r)) => o.to_bits() == r.to_bits(),
        _ => false,
    };
    let (oracle_digits, oracle_exponent) = significant_digits(oracle_number);
    let (rakka_digits, rakka_exponent) = significant_digits(rakka_number);
    same_double
        && oracle_number.contains('E') == rakka_number.contains('E')
        && oracle_exponent == rakka_exponent
        && (16..=17).contains(&oracle_digits.len())
        && rakka_digits.len() <= oracle_digits.len()
}

#[test]
fn only_a_jdk17_double_text_difference_is_tolerated() {
    let line = |number: &str| format!("NUM-SHORT x -> ok {number}\n");
    // the five bugs of the Fork D sample and 2e23
    assert!(only_jdk17_double_text_differs(
        &line("9.841100260380799E18"),
        &line("9.8411002603808E18")
    ));
    assert!(only_jdk17_double_text_differs(
        &line("1.9999999999999998E23"),
        &line("2.0E23")
    ));
    // a different double, another layout, a 15-digit oracle text, another part of the line, two numbers
    assert!(!only_jdk17_double_text_differs(
        &line("9.841100260380799E18"),
        &line("9.841100260380801E18")
    ));
    assert!(!only_jdk17_double_text_differs(
        &line("9.999999999999999E22"),
        &line("1.0E23")
    ));
    assert!(!only_jdk17_double_text_differs(
        &line("9841100260380799000.0"),
        &line("9.8411002603808E18")
    ));
    assert!(!only_jdk17_double_text_differs(
        &line("1.00000000000001E1"),
        &line("1.0000000000000099E1")
    ));
    assert!(!only_jdk17_double_text_differs(
        "a 9.841100260380799E18\n",
        "b 9.8411002603808E18\n"
    ));
    assert!(!only_jdk17_double_text_differs(
        "9.841100260380799E18 2.3851354565708683E25",
        "9.8411002603808E18 2.3851354565708684E25"
    ));
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
fn recorded_config_hocon_cases_match_pekko() {
    let unproduced = config_hocon_cases::PENDING
        .iter()
        .map(|(id, _)| *id)
        .collect();
    oracle_family_matches_pekko(
        "config-hocon-case",
        "config-hocon.txt",
        config_hocon_cases::rust_lines(),
        unproduced,
    );
}

#[test]
fn recorded_provider_selection_cases_match_pekko() {
    oracle_family_matches_pekko(
        "provider-selection-case",
        "provider-selection.txt",
        provider_selection_cases::rust_lines(),
        Vec::new(),
    );
}

#[test]
fn recorded_scheduler_cases_match_pekko() {
    oracle_family_matches_pekko(
        "scheduler-case",
        "scheduler.txt",
        scheduler_cases::rust_lines_where(|_| true),
        scheduler_cases::PENDING.iter().map(|(id, _)| *id).collect(),
    );
}

#[test]
fn recorded_mailbox_status_cases_match_pekko() {
    oracle_family_matches_pekko(
        "mailbox-case",
        "mailbox.txt",
        mailbox_cases::rust_lines(),
        mailbox_cases::PENDING.iter().map(|(id, _)| *id).collect(),
    );
}

#[test]
fn recorded_deployer_settings_cases_match_pekko() {
    oracle_family_matches_pekko(
        "deployer-settings-case",
        "deployer-settings.txt",
        deployer_settings_cases::rust_lines(),
        deployer_settings_cases::PENDING
            .iter()
            .map(|(id, _)| *id)
            .collect(),
    );
}

#[test]
fn recorded_deployer_cases_match_pekko() {
    oracle_family_matches_pekko(
        "deployer-case",
        "deployer.txt",
        deployer_cases::rust_lines(),
        Vec::new(),
    );
}

#[test]
fn recorded_deploy_cases_match_pekko() {
    oracle_family_matches_pekko(
        "deploy-case",
        "deploy.txt",
        deploy_cases::rust_lines(),
        Vec::new(),
    );
}

#[test]
fn recorded_actor_system_setup_cases_match_pekko() {
    let unproduced = actor_system_setup_cases::PENDING
        .iter()
        .map(|(id, _)| *id)
        .collect();
    oracle_family_matches_pekko(
        "actor-system-setup-case",
        "actor-system-setup.txt",
        actor_system_setup_cases::rust_lines(),
        unproduced,
    );
}

#[test]
fn recorded_dynamic_access_cases_match_pekko() {
    oracle_family_matches_pekko(
        "dynamic-access-case",
        "dynamic-access.txt",
        dynamic_access_cases::rust_lines(),
        Vec::new(),
    );
}

#[test]
fn recorded_config_settings_cases_match_pekko() {
    let unproduced = config_settings_cases::PENDING
        .iter()
        .map(|(id, _)| *id)
        .collect();
    oracle_family_matches_pekko(
        "config-settings-case",
        "config-settings.txt",
        config_settings_cases::rust_lines(),
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

/// Single-case evaluation of the util-core family: the CLI asks for one id per process, and the expensive cases of the structures group (the script `FF-04`, 270 ms of the 310 ms
/// the family costs once the concurrent cases are skipped, and the concurrent cases themselves) must not be paid for by the other ids. The witnesses are deterministic counters
/// (never a clock); the equivalence is against the full list, which `recorded_util_core_cases_match_pekko` ties to Pekko.
mod lazy_single_case {
    use super::{structures_cases, util_core_cases};

    const A_LIGHT_CASE: &str = "RG-01 ReentrantGuard is absent from the 2.0.0 jar";
    const A_CONCURRENT_CASE: &str = "CC-02 Switch: exactly one of 8 racing switchOn wins";
    const THE_EXPENSIVE_SCRIPT: &str = "FF-04 fast sketch: constructor requirements";
    const EVERY_CONCURRENT_CASE: usize = 15;
    const EVERY_SCRIPT: usize = 207;

    fn evaluated() -> (usize, usize) {
        (
            structures_cases::SCRIPTS_EVALUATED.with(|n| n.get()),
            structures_cases::CONCURRENT_CASES_EVALUATED.with(|n| n.get()),
        )
    }

    fn reset_witnesses() {
        structures_cases::SCRIPTS_EVALUATED.with(|n| n.set(0));
        structures_cases::CONCURRENT_CASES_EVALUATED.with(|n| n.set(0));
    }

    fn lookup<'a>(lines: &'a [(String, String)], id: &str) -> Option<&'a str> {
        lines
            .iter()
            .find(|(case, _)| case == id)
            .map(|(_, line)| line.as_str())
    }

    #[test]
    fn a_case_of_no_script_and_no_concurrent_kind_evaluates_neither() {
        reset_witnesses();
        let lines = util_core_cases::rust_lines_for(Some(A_LIGHT_CASE));
        assert_eq!(evaluated(), (0, 0));
        assert!(lookup(&lines, A_LIGHT_CASE).is_some());
    }

    #[test]
    fn a_script_case_is_evaluated_alone_and_gives_the_line_of_the_full_list() {
        let full = util_core_cases::rust_lines();
        reset_witnesses();
        let lines = util_core_cases::rust_lines_for(Some(THE_EXPENSIVE_SCRIPT));
        assert_eq!(evaluated(), (1, 0));
        assert!(lookup(&lines, THE_EXPENSIVE_SCRIPT).is_some());
        assert_eq!(
            lookup(&lines, THE_EXPENSIVE_SCRIPT),
            lookup(&full, THE_EXPENSIVE_SCRIPT)
        );
    }

    #[test]
    fn a_concurrent_case_is_evaluated_alone_and_gives_the_line_of_the_full_list() {
        let full = util_core_cases::rust_lines();
        reset_witnesses();
        let lines = util_core_cases::rust_lines_for(Some(A_CONCURRENT_CASE));
        assert_eq!(evaluated(), (0, 1));
        assert!(lookup(&lines, A_CONCURRENT_CASE).is_some());
        assert_eq!(
            lookup(&lines, A_CONCURRENT_CASE),
            lookup(&full, A_CONCURRENT_CASE)
        );
    }

    #[test]
    fn asking_for_everything_evaluates_every_script_and_concurrent_case_and_keeps_the_order_and_the_content()
     {
        reset_witnesses();
        let everything = util_core_cases::rust_lines_for(None);
        assert_eq!(evaluated(), (EVERY_SCRIPT, EVERY_CONCURRENT_CASE));
        assert_eq!(everything, util_core_cases::rust_lines());
        let ids: Vec<&str> = everything.iter().map(|(id, _)| id.as_str()).collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!((ids.len(), unique.len()), (553, 553));
    }

    /// Every concurrent case, the expensive script and a sample of the others (each single-case evaluation re-runs the cheap cases, which is slow in a debug build): the 553 ids are
    /// compared byte for byte on the real binary, against the unchanged one, by the validation of the change.
    #[test]
    fn the_expensive_cases_and_a_sample_of_the_others_give_the_same_line_when_they_are_the_only_one_asked_for()
     {
        let full = util_core_cases::rust_lines();
        let expensive = |id: &str| {
            id.starts_with("CC-") || id.starts_with("BQ-C") || id == THE_EXPENSIVE_SCRIPT
        };
        let checked: Vec<_> = full
            .iter()
            .enumerate()
            .filter(|(position, (id, _))| position % 70 == 0 || expensive(id))
            .collect();
        assert_eq!(
            checked.iter().filter(|(_, (id, _))| expensive(id)).count(),
            EVERY_CONCURRENT_CASE + 1
        );
        for (_, (id, line)) in checked {
            let single = util_core_cases::rust_lines_for(Some(id));
            assert_eq!(lookup(&single, id), Some(line.as_str()), "{id}");
        }
    }

    #[test]
    fn an_unknown_empty_or_malformed_id_finds_nothing_and_evaluates_nothing_of_the_structures_group()
     {
        for id in [
            "",
            "CC-02",
            "FF-04",
            "no such case",
            "cc-02 switch: exactly one of 8 racing switchon wins",
            "CC-02 Switch: exactly one of 8 racing switchOn wins ",
            "\u{0}",
        ] {
            reset_witnesses();
            let lines = util_core_cases::rust_lines_for(Some(id));
            assert_eq!(lookup(&lines, id), None, "{id:?}");
            assert_eq!(evaluated(), (0, 0), "{id:?}");
        }
    }
}
