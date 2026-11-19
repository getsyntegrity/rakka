// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/DynamicAccess.scala and actor/src/main/scala/org/apache/pekko/actor/ReflectiveDynamicAccess.scala (Apache-2.0).
//! Cases of the AC-CFG-016 oracle (`DynamicAccessOracle.java`): what a caller observes when a class name is looked up by
//! name. The Rust twin is `FactoryRegistry`: the oracle's classpath is pekko-actor alone, so the names the cases ask about
//! that exist there (`Actor`, `LocalActorRefProvider`, `UnboundedMailbox`) are registered and nothing else is (no remote,
//! no cluster).

use std::convert::Infallible;

use rakka_actor::config::{CreateError, FactoryRegistry, ProviderSelection};

use crate::text_escape::escape_non_ascii_units;

const REMOTE: &str = "org.apache.pekko.remote.RemoteActorRefProvider";
const CLUSTER: &str = "org.apache.pekko.cluster.ClusterActorRefProvider";
const LOCAL: &str = "org.apache.pekko.actor.LocalActorRefProvider";
const UNBOUNDED_MAILBOX: &str = "org.apache.pekko.dispatch.UnboundedMailbox";
const ACTOR: &str = "org.apache.pekko.actor.Actor";

type Factory = Box<dyn Fn(()) -> Result<String, Infallible>>;

/// The names of the oracle's classpath that the cases mention; each factory "creates" an instance of its own class.
fn classpath() -> FactoryRegistry<Factory> {
    let mut registry = FactoryRegistry::new();
    for name in [ACTOR, LOCAL, UNBOUNDED_MAILBOX] {
        registry.register(name, Box::new(move |()| Ok(name.to_owned())) as Factory);
    }
    registry
}

/// `createInstanceFor(name, Nil)`: `ok created <class>` or the exception the oracle prints.
fn created(name: &str) -> String {
    match classpath().create(name, ()) {
        Ok(class) => format!("ok created {class}"),
        Err(CreateError::ClassNotFound(error)) => {
            let simple = error.java_class().rsplit('.').next().unwrap_or_default();
            format!("ERR {simple} [{}]", error.message())
        }
        Err(CreateError::Construction(never)) => match never {},
    }
}

/// A provider named by `pekko.actor.provider` is loaded by name and the failure propagates unwrapped (`ActorSystem.scala:996-1011`).
fn provider_loads(name: &str) -> String {
    match classpath().lookup(name) {
        Ok(_) => "ok created".to_owned(),
        Err(error) => format!(
            "ERR {} [{}]",
            error.java_class().rsplit('.').next().unwrap_or_default(),
            error.message()
        ),
    }
}

/// The provider named by the value of `pekko.actor.provider` (a short name or a class name): the selection gives the class
/// name that is looked up (`ActorSystem.scala:137-144`).
fn provider_value_loads(value: &str) -> String {
    provider_loads(ProviderSelection::from_identifier(value).fqcn())
}

fn on_classpath(name: &str) -> String {
    format!("ok {}", classpath().contains(name))
}

/// `(escaped case id, escaped line)` in the registration order of `DynamicAccessOracle`.
pub fn rust_lines() -> Vec<(String, String)> {
    let cases: Vec<(String, String)> = vec![
        (
            "CNF-01 createInstanceFor foo.NonExistingClass".to_owned(),
            created("foo.NonExistingClass"),
        ),
        (
            "CNF-02 createInstanceFor org.apache.pekko.actor.foo.NonExistingClass".to_owned(),
            created("org.apache.pekko.actor.foo.NonExistingClass"),
        ),
        (
            "CNF-03 createInstanceFor empty name".to_owned(),
            created(""),
        ),
        (
            "CNF-04 createInstanceFor name differing only in case".to_owned(),
            created("org.apache.pekko.dispatch.unboundedmailbox"),
        ),
        (
            format!("CNF-05 createInstanceFor the provider {REMOTE}"),
            created(REMOTE),
        ),
        (
            format!("CNF-06 createInstanceFor the provider {CLUSTER}"),
            created(CLUSTER),
        ),
        (
            "ON-01 classIsOnClasspath i.just.made.it.up.to.hurt.Myself".to_owned(),
            on_classpath("i.just.made.it.up.to.hurt.Myself"),
        ),
        (
            format!("ON-02 classIsOnClasspath {ACTOR}"),
            on_classpath(ACTOR),
        ),
        (
            format!("ON-03 classIsOnClasspath {REMOTE}"),
            on_classpath(REMOTE),
        ),
        (
            format!("ON-04 classIsOnClasspath {CLUSTER}"),
            on_classpath(CLUSTER),
        ),
        (
            format!("ON-05 classIsOnClasspath {LOCAL}"),
            on_classpath(LOCAL),
        ),
        (
            "ON-06 classIsOnClasspath empty name".to_owned(),
            on_classpath(""),
        ),
        (
            format!("INST-01 createInstanceFor {UNBOUNDED_MAILBOX}"),
            created(UNBOUNDED_MAILBOX),
        ),
        (
            "PRV-01 provider foo.NonExisting".to_owned(),
            provider_loads("foo.NonExisting"),
        ),
        (format!("PRV-02 provider {REMOTE}"), provider_loads(REMOTE)),
        (
            format!("PRV-03 provider {CLUSTER}"),
            provider_loads(CLUSTER),
        ),
        (format!("PRV-04 provider {LOCAL}"), provider_loads(LOCAL)),
        (
            "CNF-07 createInstanceFor name with surrounding spaces".to_owned(),
            created(&format!(" {UNBOUNDED_MAILBOX} ")),
        ),
        (
            "ON-07 classIsOnClasspath name differing only in case".to_owned(),
            on_classpath("org.apache.pekko.actor.actor"),
        ),
        (
            "ON-08 classIsOnClasspath name with a trailing space".to_owned(),
            on_classpath("org.apache.pekko.actor.Actor "),
        ),
        (
            "PRV-05 provider remote".to_owned(),
            provider_value_loads("remote"),
        ),
        (
            "PRV-06 provider cluster".to_owned(),
            provider_value_loads("cluster"),
        ),
        (
            "PRV-07 provider local".to_owned(),
            provider_value_loads("local"),
        ),
        (
            "PRV-08 provider LOCAL (case differs)".to_owned(),
            provider_value_loads("LOCAL"),
        ),
        (
            "PRV-09 provider other.ActorRefProvider".to_owned(),
            provider_value_loads("other.ActorRefProvider"),
        ),
        (
            "PRV-10 provider empty string".to_owned(),
            provider_value_loads(""),
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
