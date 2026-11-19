// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Cases of the AC-CFG-008 oracle (`ProviderSelectionOracle.java`): the `ProviderSelection`, class name and cluster flag that
//! `Settings` derives from the value of `pekko.actor.provider` (`ActorSystem.scala:102-144` and `:411-421`).

use rakka_actor::config::{
    CLUSTER_ACTOR_REF_PROVIDER, Config, LOCAL_ACTOR_REF_PROVIDER, REMOTE_ACTOR_REF_PROVIDER,
    Settings,
};

use crate::text_escape::escape_non_ascii_units;

/// Settings over `pekko.actor.provider = <hocon_value>` merged on the base (typed over actor reference.conf, resolved).
fn settings_with(hocon_value: &str) -> Result<Settings, String> {
    let config = Config::parse_string(&format!("pekko.actor.provider = {hocon_value}"))
        .and_then(|c| {
            c.with_fallback(&Config::default_reference_with_typed())
                .resolve()
        })
        .map_err(|e| format!("ERR {} [{}]", e.kind().java_name(), e.message()))?;
    Settings::new("oracle", config).map_err(|e| format!("ERR {} [{}]", e.java_name(), e.message()))
}

fn selected(value: &str) -> String {
    match settings_with(&format!("\"{value}\"")) {
        Ok(s) => format!(
            "ok identifier={} fqcn={} hasCluster={} ProviderClass={} HasCluster={}",
            s.provider_selection.identifier(),
            s.provider_selection.fqcn(),
            s.provider_selection.has_cluster(),
            s.provider_class(),
            s.has_cluster()
        ),
        Err(line) => line,
    }
}

fn same(left: &str, right: &str) -> String {
    match (
        settings_with(&format!("\"{left}\"")),
        settings_with(&format!("\"{right}\"")),
    ) {
        (Ok(a), Ok(b)) => format!("ok {}", a.provider_selection == b.provider_selection),
        (Err(line), _) | (_, Err(line)) => line,
    }
}

/// `(escaped case id, escaped line)` in the registration order of `ProviderSelectionOracle`.
pub fn rust_lines() -> Vec<(String, String)> {
    let value = |hocon_value: &str| match settings_with(hocon_value) {
        Ok(s) => format!(
            "ok identifier={} ProviderClass={}",
            s.provider_selection.identifier(),
            s.provider_class()
        ),
        Err(line) => line,
    };
    let number = value("5");
    let cases: Vec<(String, String)> = vec![
        ("SEL-01 provider = local".to_owned(), selected("local")),
        ("SEL-02 provider = remote".to_owned(), selected("remote")),
        ("SEL-03 provider = cluster".to_owned(), selected("cluster")),
        (
            format!("SEL-04 provider = {REMOTE_ACTOR_REF_PROVIDER}"),
            selected(REMOTE_ACTOR_REF_PROVIDER),
        ),
        (
            format!("SEL-05 provider = {CLUSTER_ACTOR_REF_PROVIDER}"),
            selected(CLUSTER_ACTOR_REF_PROVIDER),
        ),
        (
            "SEL-06 provider = other.ActorRefProvider".to_owned(),
            selected("other.ActorRefProvider"),
        ),
        (
            format!(
                "SEL-07 provider = {LOCAL_ACTOR_REF_PROVIDER} (the class name of Local is Custom)"
            ),
            selected(LOCAL_ACTOR_REF_PROVIDER),
        ),
        (
            "SEL-08 provider = LOCAL (case differs)".to_owned(),
            selected("LOCAL"),
        ),
        (
            "SEL-09 provider = Remote (case differs)".to_owned(),
            selected("Remote"),
        ),
        (
            "SEL-10 provider = \" local\" (leading space)".to_owned(),
            selected(" local"),
        ),
        (
            "SEL-11 provider = \"cluster \" (trailing space)".to_owned(),
            selected("cluster "),
        ),
        ("SEL-12 provider = empty string".to_owned(), selected("")),
        ("SEL-13 provider = 5 (a number)".to_owned(), number),
        (
            format!("EQ-01 remote equals {REMOTE_ACTOR_REF_PROVIDER}"),
            same("remote", REMOTE_ACTOR_REF_PROVIDER),
        ),
        (
            format!("EQ-02 cluster equals {CLUSTER_ACTOR_REF_PROVIDER}"),
            same("cluster", CLUSTER_ACTOR_REF_PROVIDER),
        ),
        (
            format!("EQ-03 local equals {LOCAL_ACTOR_REF_PROVIDER}"),
            same("local", LOCAL_ACTOR_REF_PROVIDER),
        ),
        (
            "EQ-04 two Custom with the same name".to_owned(),
            same("other.ActorRefProvider", "other.ActorRefProvider"),
        ),
        (
            "EQ-05 two Custom with different names".to_owned(),
            same("other.A", "other.B"),
        ),
        (
            "EQ-06 remote equals cluster".to_owned(),
            same("remote", "cluster"),
        ),
        (
            "SEL-14 provider = [local] (a list)".to_owned(),
            value("[local]"),
        ),
        (
            "SEL-15 provider = [] (an empty list)".to_owned(),
            value("[]"),
        ),
        (
            "SEL-16 provider = {a=1} (an object)".to_owned(),
            value("{a=1}"),
        ),
        ("SEL-17 provider = null".to_owned(), value("null")),
        (
            "SEL-18 provider = true (a boolean)".to_owned(),
            value("true"),
        ),
        (
            "SEL-19 provider = 1.5 (a decimal number)".to_owned(),
            value("1.5"),
        ),
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
