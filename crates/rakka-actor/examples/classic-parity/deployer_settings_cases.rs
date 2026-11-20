// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! Cases of the `DeployerSettings` oracle (`DeployerSettingsOracle.java`): the scripts of `deployer_settings.tsv`, each the body
//! of `pekko.actor.deployment` over the reference configuration and lookups on the `Deployer` built from it.

use rakka_actor::actor::{ActorPath, Deploy, Deployer, DeployerError, RouterConfig, Scope};
use rakka_actor::config::{Config, Settings};

use crate::deployer_cases::{elements, units_to_text, word};
use crate::text_escape::escape_non_ascii_units;

const SCRIPTS: &str = include_str!("deployer_settings.tsv");

/// Cases that Rust does not produce, with the reason: several keys give the same path and Pekko's winner is decided by the
/// iteration order of a JVM `HashSet` of entries (hash of key and value), where Rust takes the last key in alphabetical order.
/// Temporary policy authorised by the Owner (Q-CFG-36, proposed): the Rust unit tests pin it.
pub const PENDING: &[(&str, &str)] = &[
    (
        "CO-04 a plain key and three slashes",
        "Q-CFG-36: Pekko deploys /svc last, Rust deploys /svc/// last (alphabetical order)",
    ),
    (
        "CO-05 two and three slashes",
        "Q-CFG-36: Pekko deploys /svc// last, Rust deploys /svc/// last (alphabetical order)",
    ),
];

fn text(section: &str) -> String {
    word(&section.replace("\\n", "\n"))
}

fn show(found: Option<Deploy>) -> String {
    let Some(deploy) = found else {
        return "none".to_owned();
    };
    let router = match deploy.router_config() {
        RouterConfig::NoRouter => "NoRouter".to_owned(),
        _ => deploy
            .config()
            .get_string("router")
            .expect("a router that is not from-code was configured by name"),
    };
    let scope = match deploy.scope() {
        Scope::NoScopeGiven => "NoScopeGiven",
        _ => "other",
    };
    format!(
        "{{path=\"{}\" dispatcher=\"{}\" mailbox=\"{}\" router={router} scope={scope} config={}}}",
        deploy.path(),
        deploy.dispatcher(),
        deploy.mailbox(),
        deploy.config().render_concise()
    )
}

fn failure(error: &DeployerError) -> String {
    let message = match error {
        DeployerError::Deploy(e) => units_to_text(&e.message_utf16()),
        other => other.message().to_owned(),
    };
    format!("ERR {} [{message}]", error.simple_class_name())
}

fn script(section: &str, operations: &str) -> String {
    let user = Config::parse_string(&format!("pekko.actor.deployment {{\n{}\n}}", text(section)))
        .expect("a valid configuration");
    let config = user
        .with_fallback(&Config::default_reference())
        .resolve()
        .expect("a resolvable configuration");
    let settings = Settings::new("oracle", config).expect("valid settings");
    let deployer = match Deployer::from_settings(&settings) {
        Ok(deployer) => deployer,
        Err(error) => return failure(&error),
    };
    let tokens: Vec<String> = operations
        .split("; ")
        .filter(|operation| !operation.is_empty())
        .map(|operation| {
            let words: Vec<&str> = operation.split(' ').collect();
            match words[0] {
                "find" => {
                    let parts = elements(words[1]);
                    show(deployer.lookup(parts.iter().map(String::as_str)))
                }
                "lookup" => {
                    let path = ActorPath::from_string(&word(words[1])).expect("a valid actor path");
                    show(deployer.lookup_path(&path))
                }
                other => panic!("unknown operation {other}"),
            }
        })
        .collect();
    if tokens.is_empty() {
        "built".to_owned()
    } else {
        tokens.join(" | ")
    }
}

/// Every case of the oracle that Rust produces: the id as the oracle prints it and the whole output line.
pub fn rust_lines() -> Vec<(String, String)> {
    SCRIPTS
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let (id, section, operations) = (fields.next()?, fields.next()?, fields.next()?);
            if PENDING.iter().any(|(pending, _)| *pending == id) {
                return None;
            }
            let result = script(section, operations);
            let output = format!("{id} -> ok {result}");
            Some((escape_non_ascii_units(id), escape_non_ascii_units(&output)))
        })
        .collect()
}
