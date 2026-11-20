// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! Twin of `ConfigRenderOracle.java`: `Deploy.toString` of a deployment that holds the configuration of one input line.

use rakka_actor::actor::{Deploy, RouterConfig, Scope};
use rakka_actor::config::Config;

use crate::text_escape::escape_non_ascii_units;

pub fn line(text: &str) -> String {
    match Config::parse_string(text) {
        Ok(config) => escape_non_ascii_units(
            &Deploy::new(
                "/a",
                config,
                RouterConfig::NoRouter,
                Scope::NoScopeGiven,
                "",
                "",
            )
            .to_string(),
        ),
        Err(_) => "ERR".to_owned(),
    }
}
