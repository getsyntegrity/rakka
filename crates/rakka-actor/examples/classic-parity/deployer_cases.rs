// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! Cases of the `Deployer` oracle (`DeployerOracle.java`): the scripts of `deployer.tsv` (deployments with wildcards added
//! and looked up) and two concurrency cases that print invariants. The header of `deployer.tsv` explains the operations.

use std::sync::Barrier;
use std::thread;

use rakka_actor::actor::{ActorError, ActorPath, Deploy, Deployer, RouterConfig, Scope};
use rakka_actor::config::Config;

use crate::text_escape::escape_non_ascii_units;

const SCRIPTS: &str = include_str!("deployer.tsv");

fn deployment(path: &str, dispatcher: &str) -> Deploy {
    Deploy::new(
        path,
        Config::empty(),
        RouterConfig::NoRouter,
        Scope::NoScopeGiven,
        dispatcher,
        "",
    )
}

/// One UTF-16 unit as text, spelling an unpaired surrogate as the `\uXXXX` escape the oracle prints for it.
pub fn units_to_text(units: &[u16]) -> String {
    char::decode_utf16(units.iter().copied())
        .map(|unit| match unit {
            Ok(c) => c.to_string(),
            Err(lone) => format!("\\u{:04X}", lone.unpaired_surrogate()),
        })
        .collect()
}

/// A word of a script with its `\uXXXX` escapes decoded (a lone surrogate stays an escape, which `esc` leaves alone);
/// `""` is the empty string.
pub fn word(text: &str) -> String {
    if text == "\"\"" {
        return String::new();
    }
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut decoded: Vec<u16> = Vec::new();
    let mut i = 0;
    while i < units.len() {
        let escape = units
            .get(i..i + 6)
            .filter(|window| window[0] == u16::from(b'\\') && window[1] == u16::from(b'u'))
            .and_then(|window| String::from_utf16(&window[2..]).ok())
            .filter(|hex| {
                hex.chars()
                    .all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c))
            })
            .and_then(|hex| u16::from_str_radix(&hex, 16).ok());
        match escape {
            Some(unit) => {
                decoded.push(unit);
                i += 6;
            }
            None => {
                decoded.push(units[i]);
                i += 1;
            }
        }
    }
    units_to_text(&decoded)
}

pub fn elements(text: &str) -> Vec<String> {
    if text == "-" {
        return Vec::new();
    }
    word(text).split('/').map(str::to_owned).collect()
}

fn found(deploy: Option<Deploy>) -> String {
    deploy.map_or_else(
        || "none".to_owned(),
        |d| format!("some({}@{})", d.path(), d.dispatcher()),
    )
}

fn failure(error: &ActorError) -> String {
    format!(
        "ERR {} [{}]",
        error
            .class()
            .java_name()
            .rsplit('.')
            .next()
            .unwrap_or_default(),
        units_to_text(&error.message_utf16())
    )
}

fn script(text: &str) -> String {
    let deployer = Deployer::new();
    let mut deploys = 0;
    let mut tokens = Vec::new();
    for operation in text.split("; ") {
        let words: Vec<&str> = operation.split(' ').collect();
        match words[0] {
            "deploy" => {
                deploys += 1;
                let result = deployer.deploy(deployment(&word(words[1]), &format!("d{deploys}")));
                tokens.push(result.map_or_else(|e| failure(&e), |()| "ok".to_owned()));
            }
            "find" => {
                let parts = elements(words[1]);
                tokens.push(found(deployer.lookup(parts.iter().map(String::as_str))));
            }
            "lookup" => {
                let path = ActorPath::from_string(&word(words[1])).expect("a valid actor path");
                tokens.push(found(deployer.lookup_path(&path)));
            }
            other => panic!("unknown operation {other}"),
        }
    }
    tokens.join(" | ")
}

fn concurrent_deploys(threads: usize, per_thread: usize) -> String {
    let deployer = Deployer::new();
    let start = Barrier::new(threads);
    let failures = thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let (deployer, start) = (&deployer, &start);
                scope.spawn(move || {
                    start.wait();
                    (0..per_thread)
                        .filter(|n| {
                            deployer
                                .deploy(deployment(&format!("/t{t}/n{n}"), "d"))
                                .is_err()
                        })
                        .count()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().expect("a worker"))
            .sum::<usize>()
    });
    let present = (0..threads)
        .flat_map(|t| (0..per_thread).map(move |n| (format!("t{t}"), format!("n{n}"))))
        .filter(|(first, second)| deployer.lookup([first.as_str(), second.as_str()]).is_some())
        .count();
    format!(
        "failures={failures} present={present} of {}",
        threads * per_thread
    )
}

fn concurrent_same_path(threads: usize) -> String {
    let deployer = Deployer::new();
    let start = Barrier::new(threads);
    thread::scope(|scope| {
        for t in 0..threads {
            let (deployer, start) = (&deployer, &start);
            scope.spawn(move || {
                start.wait();
                deployer
                    .deploy(deployment("/same/path", &format!("d{t}")))
                    .expect("a valid deployment");
            });
        }
    });
    let one_of_them = deployer
        .lookup(["same", "path"])
        .is_some_and(|d| (0..threads).any(|t| d.dispatcher() == format!("d{t}")));
    format!("found one of the deployments={one_of_them}")
}

/// Every case of the oracle that Rust produces: the id as the oracle prints it and the whole output line.
pub fn rust_lines() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = SCRIPTS
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (id, text) = line.split_once('\t').expect("an id and a script");
            (id.to_owned(), script(text))
        })
        .collect();
    cases.push((
        "DC-01 concurrent deploys of different paths are all kept".to_owned(),
        concurrent_deploys(8, 25),
    ));
    cases.push((
        "DC-02 concurrent deploys of one path keep one of them".to_owned(),
        concurrent_same_path(8),
    ));
    cases
        .into_iter()
        .map(|(id, text)| {
            let line = format!("{id} -> ok {text}");
            (escape_non_ascii_units(&id), escape_non_ascii_units(&line))
        })
        .collect()
}
