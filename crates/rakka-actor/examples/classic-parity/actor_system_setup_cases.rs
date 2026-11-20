// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/setup/ActorSystemSetup.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Cases of the AC-CFG-015 oracle (`ActorSystemSetupOracle.java`): `ActorSystemSetup`, its `toString`, `BootstrapSetup` and the
//! priority of the setup's provider in `Settings`. Where the Java oracle compares two references with `==`, the Rust twin
//! compares the contents (a stored setup is moved into the `ActorSystemSetup`, so there is no reference to compare).

use rakka_actor::config::{
    ActorSystemSetup, BootstrapSetup, Config, ProviderSelection, Settings, Setup,
};

use crate::text_escape::escape_non_ascii_units;

/// Cases the Rust twin does not produce, with the reason (the JVM-only fields of `BootstrapSetup`, Q-CFG-32).
#[allow(dead_code)]
pub const PENDING: &[(&str, &str)] = &[
    (
        "BS-15 withClassloader",
        "Q-CFG-32: BootstrapSetup has no classLoader in Rust (the registry of Q-X-2 replaces class loading)",
    ),
    (
        "BS-16 withDefaultExecutionContext",
        "Q-CFG-32: BootstrapSetup has no defaultExecutionContext in Rust until AC-DSP-006 defines the executor",
    ),
];

#[derive(Debug, PartialEq, Eq)]
struct DummySetup(String);
#[derive(Debug, PartialEq, Eq)]
struct DummySetup2(String);
#[derive(Debug, PartialEq, Eq)]
struct DummySetup3(String);
#[derive(Debug, PartialEq, Eq)]
struct JavaSetup(String);

impl Setup for DummySetup {
    fn class_name() -> &'static str {
        "pilot.ActorSystemSetupOracle$DummySetup"
    }
}
impl Setup for DummySetup2 {
    fn class_name() -> &'static str {
        "pilot.ActorSystemSetupOracle$DummySetup2"
    }
}
impl Setup for DummySetup3 {
    fn class_name() -> &'static str {
        "pilot.ActorSystemSetupOracle$DummySetup3"
    }
}
impl Setup for JavaSetup {
    fn class_name() -> &'static str {
        "pilot.ActorSystemSetupOracle$JavaSetup"
    }
}

fn dummy(name: &str) -> DummySetup {
    DummySetup(name.to_owned())
}
fn dummy2(name: &str) -> DummySetup2 {
    DummySetup2(name.to_owned())
}

fn names(setups: &ActorSystemSetup) -> String {
    let show = |found: Option<&str>| found.unwrap_or("absent").to_owned();
    format!(
        "DummySetup={} DummySetup2={}",
        show(setups.get::<DummySetup>().map(|d| d.0.as_str())),
        show(setups.get::<DummySetup2>().map(|d| d.0.as_str()))
    )
}

fn presence(present: bool) -> &'static str {
    if present { "present" } else { "absent" }
}

fn bootstrap(b: &BootstrapSetup) -> String {
    let config = match b.config() {
        Some(c) => format!("x={}", c.get_int("x").unwrap_or_default()),
        None => "absent".to_owned(),
    };
    let provider = b
        .actor_ref_provider()
        .map_or("absent", ProviderSelection::identifier);
    format!("config={config} provider={provider}")
}

fn config_with_x(x: i32) -> Config {
    Config::parse_string(&format!("x = {x}")).expect("a valid configuration")
}

fn provider_of(configured: &str, setup: ActorSystemSetup) -> String {
    let config = Config::parse_string(&format!("pekko.actor.provider = \"{configured}\""))
        .and_then(|c| {
            c.with_fallback(&Config::default_reference_with_typed())
                .resolve()
        });
    match config {
        Err(e) => format!("ERR {} [{}]", e.kind().java_name(), e.message()),
        Ok(config) => match Settings::with_setup("oracle", config, setup) {
            Ok(s) => format!(
                "ok identifier={} ProviderClass={} HasCluster={}",
                s.provider_selection.identifier(),
                s.provider_class(),
                s.has_cluster()
            ),
            Err(e) => format!("ERR {} [{}]", e.java_name(), e.message()),
        },
    }
}

fn with_bootstrap(b: BootstrapSetup) -> ActorSystemSetup {
    ActorSystemSetup::empty().with_setup(b)
}

/// `(escaped case id, escaped line)` in the registration order of `ActorSystemSetupOracle`.
pub fn rust_lines() -> Vec<(String, String)> {
    let ok = |text: String| format!("ok {text}");
    let cases: Vec<(String, String)> = vec![
        (
            "AS-01 store and retrieve a setup".to_owned(),
            ok(names(&ActorSystemSetup::empty().with_setup(dummy("Al Dente")))),
        ),
        (
            "AS-02 get on an empty setup".to_owned(),
            ok(names(&ActorSystemSetup::empty())),
        ),
        (
            "AS-03 replace the setup of the same class".to_owned(),
            ok(names(
                &ActorSystemSetup::empty()
                    .with_setup(dummy("Al Dente"))
                    .with_setup(dummy("Earl E. Bird")),
            )),
        ),
        (
            "AS-04 and with a repeated class".to_owned(),
            ok(names(
                &dummy("Al Dente")
                    .and(dummy("Earl E. Bird"))
                    .with_setup(dummy2("Amanda Reckonwith")),
            )),
        ),
        (
            "AS-05 and with two different classes".to_owned(),
            ok(names(&dummy("Al Dente").and(dummy2("Amanda Reckonwith")))),
        ),
        ("AS-06 create with several setups".to_owned(), {
            let s = ActorSystemSetup::empty()
                .with_setup(dummy("Manny Kin"))
                .with_setup(dummy2("Pepe Roni"));
            ok(format!(
                "DummySetup={} DummySetup2={} DummySetup3={}",
                presence(s.get::<DummySetup>().is_some()),
                presence(s.get::<DummySetup2>().is_some()),
                presence(s.get::<DummySetup3>().is_some())
            ))
        }),
        (
            "AS-07 create with two setups of the same class".to_owned(),
            ok(names(
                &ActorSystemSetup::empty()
                    .with_setup(dummy("first"))
                    .with_setup(dummy("second")),
            )),
        ),
        ("AS-08 the Java API".to_owned(), {
            let s = ActorSystemSetup::empty().with_setup(JavaSetup("Jasmine Rice".to_owned()));
            let found = s.get::<JavaSetup>();
            ok(format!(
                "{} {} {}",
                found.is_some(),
                found == Some(&JavaSetup("Jasmine Rice".to_owned())),
                found.map_or("", |j| j.0.as_str())
            ))
        }),
        ("AS-09 withSetup leaves the original setup unchanged".to_owned(), {
            let original = ActorSystemSetup::empty();
            let changed = original.clone().with_setup(dummy("Al Dente"));
            ok(format!("original {} changed {}", names(&original), names(&changed)))
        }),
        ("AS-10 and on an ActorSystemSetup equals withSetup".to_owned(), {
            let base = ActorSystemSetup::empty().with_setup(dummy("Al Dente"));
            let second = || dummy2("Amanda Reckonwith");
            ok(format!(
                "{} | {}",
                names(&base.clone().with_setup(second())),
                names(&base.with_setup(second()))
            ))
        }),
        (
            "TS-01 toString of an empty setup".to_owned(),
            ok(ActorSystemSetup::empty().to_string()),
        ),
        (
            "TS-02 toString with one setup".to_owned(),
            ok(ActorSystemSetup::empty().with_setup(dummy("a")).to_string()),
        ),
        (
            "TS-03 toString with two setups in insertion order".to_owned(),
            ok(ActorSystemSetup::empty()
                .with_setup(dummy2("b"))
                .with_setup(dummy("a"))
                .to_string()),
        ),
        (
            "TS-04 toString after replacing the first setup keeps its position".to_owned(),
            ok(ActorSystemSetup::empty()
                .with_setup(dummy("a"))
                .with_setup(dummy2("b"))
                .with_setup(dummy("c"))
                .to_string()),
        ),
        (
            "TS-05 toString with four setups".to_owned(),
            ok(ActorSystemSetup::empty()
                .with_setup(DummySetup3("c".to_owned()))
                .with_setup(JavaSetup("j".to_owned()))
                .with_setup(dummy2("b"))
                .with_setup(dummy("a"))
                .to_string()),
        ),
        (
            "TS-06 toString of create with a repeated class".to_owned(),
            ok(ActorSystemSetup::empty()
                .with_setup(dummy("a"))
                .with_setup(dummy2("b"))
                .with_setup(dummy("c"))
                .to_string()),
        ),
        (
            "TS-07 toString with a BootstrapSetup".to_owned(),
            ok(with_bootstrap(BootstrapSetup::new()).to_string()),
        ),
        ("BS-01 BootstrapSetup()".to_owned(), ok(bootstrap(&BootstrapSetup::new()))),
        ("BS-02 BootstrapSetup.create()".to_owned(), ok(bootstrap(&BootstrapSetup::new()))),
        (
            "BS-03 BootstrapSetup(config)".to_owned(),
            ok(bootstrap(&BootstrapSetup::from_config(config_with_x(1)))),
        ),
        (
            "BS-04 BootstrapSetup.create(config)".to_owned(),
            ok(bootstrap(&BootstrapSetup::from_config(config_with_x(2)))),
        ),
        (
            "BS-05 withConfig".to_owned(),
            ok(bootstrap(&BootstrapSetup::new().with_config(config_with_x(3)))),
        ),
        (
            "BS-06 withActorRefProvider(Remote)".to_owned(),
            ok(bootstrap(
                &BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Remote),
            )),
        ),
        (
            "BS-07 withActorRefProvider(Cluster)".to_owned(),
            ok(bootstrap(
                &BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Cluster),
            )),
        ),
        (
            "BS-08 withActorRefProvider(Local)".to_owned(),
            ok(bootstrap(
                &BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Local),
            )),
        ),
        (
            "BS-09 withActorRefProvider(Custom)".to_owned(),
            ok(bootstrap(&BootstrapSetup::new().with_actor_ref_provider(
                ProviderSelection::Custom("other.ActorRefProvider".to_owned()),
            ))),
        ),
        (
            "BS-10 withConfig then withActorRefProvider keeps the config".to_owned(),
            ok(bootstrap(
                &BootstrapSetup::new()
                    .with_config(config_with_x(4))
                    .with_actor_ref_provider(ProviderSelection::Cluster),
            )),
        ),
        (
            "BS-11 withActorRefProvider then withConfig keeps the provider".to_owned(),
            ok(bootstrap(
                &BootstrapSetup::new()
                    .with_actor_ref_provider(ProviderSelection::Remote)
                    .with_config(config_with_x(5)),
            )),
        ),
        (
            "BS-12 withConfig twice replaces the config".to_owned(),
            ok(bootstrap(
                &BootstrapSetup::new()
                    .with_config(config_with_x(6))
                    .with_config(config_with_x(7)),
            )),
        ),
        ("BS-13 withers leave the original unchanged".to_owned(), {
            let original = BootstrapSetup::new();
            let changed = original
                .clone()
                .with_config(config_with_x(8))
                .with_actor_ref_provider(ProviderSelection::Remote);
            ok(format!(
                "original {} changed {}",
                bootstrap(&original),
                bootstrap(&changed)
            ))
        }),
        ("BS-14 a BootstrapSetup stored in an ActorSystemSetup".to_owned(), {
            let stored = BootstrapSetup::new().with_config(config_with_x(9));
            let shown = bootstrap(&stored);
            let setups = with_bootstrap(stored);
            let found = setups.get::<BootstrapSetup>();
            ok(format!(
                "{} {} {}",
                found.is_some(),
                found.map(bootstrap).as_deref() == Some(shown.as_str()),
                found.map(bootstrap).unwrap_or_default()
            ))
        }),
        (
            "PR-01 no setup uses the configuration (local)".to_owned(),
            provider_of("local", ActorSystemSetup::empty()),
        ),
        (
            "PR-02 an empty BootstrapSetup equals none".to_owned(),
            provider_of("remote", with_bootstrap(BootstrapSetup::new())),
        ),
        (
            "PR-03 the provider of the BootstrapSetup wins over the configuration (Remote over local)".to_owned(),
            provider_of(
                "local",
                with_bootstrap(BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Remote)),
            ),
        ),
        (
            "PR-04 the provider of the BootstrapSetup wins (Cluster over remote)".to_owned(),
            provider_of(
                "remote",
                with_bootstrap(BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Cluster)),
            ),
        ),
        (
            "PR-05 the provider of the BootstrapSetup wins (Local over a custom class)".to_owned(),
            provider_of(
                "other.ActorRefProvider",
                with_bootstrap(BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Local)),
            ),
        ),
        (
            "PR-06 a Custom provider of the BootstrapSetup is selected through its identifier".to_owned(),
            provider_of(
                "local",
                with_bootstrap(BootstrapSetup::new().with_actor_ref_provider(
                    ProviderSelection::Custom("other.ActorRefProvider".to_owned()),
                )),
            ),
        ),
        (
            "PR-07 a BootstrapSetup with only a config keeps the configured provider".to_owned(),
            provider_of(
                "cluster",
                with_bootstrap(BootstrapSetup::new().with_config(config_with_x(1))),
            ),
        ),
        (
            "PR-08 other setups do not affect the provider".to_owned(),
            provider_of(
                "remote",
                ActorSystemSetup::empty().with_setup(dummy("Tad Moore")),
            ),
        ),
        ("PR-09 the setup is available from Settings.setup".to_owned(), {
            let config = Config::parse_string("")
                .and_then(|c| c.with_fallback(&Config::default_reference_with_typed()).resolve())
                .expect("a valid configuration");
            match Settings::with_setup("oracle", config, ActorSystemSetup::empty().with_setup(dummy("Tad Moore"))) {
                Ok(s) => ok(names(&s.setup)),
                Err(e) => format!("ERR {} [{}]", e.java_name(), e.message()),
            }
        }),
        ("PR-10 Settings without a setup has an empty one".to_owned(), {
            let config = Config::parse_string("")
                .and_then(|c| c.with_fallback(&Config::default_reference_with_typed()).resolve())
                .expect("a valid configuration");
            match Settings::new("oracle", config) {
                Ok(s) => ok(format!("{} {}", names(&s.setup), s.setup)),
                Err(e) => format!("ERR {} [{}]", e.java_name(), e.message()),
            }
        }),
    ];
    cases
        .into_iter()
        .map(|(id, line)| {
            (
                escape_non_ascii_units(&id),
                escape_non_ascii_units(&format!("{id} -> {line}")),
            )
        })
        .collect()
}
