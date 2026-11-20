// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! Cases of the `Deploy` oracle (`DeployOracle.java`): the values of a `Deploy`, its equality, `withFallback` of the
//! deploy, of the scope and of the router. A router is a named class, as in the Java oracle that prints its class only.

use rakka_actor::actor::{Deploy, RouterConfig, Scope};
use rakka_actor::config::Config;

fn conf(text: &str) -> Config {
    Config::parse_string(text).expect("a valid configuration")
}

fn pool(class_name: &str) -> RouterConfig {
    RouterConfig::opaque(class_name)
}

fn deploy(
    path: &str,
    config: &str,
    router: RouterConfig,
    scope: Scope,
    dispatcher: &str,
    mailbox: &str,
) -> Deploy {
    Deploy::new(path, conf(config), router, scope, dispatcher, mailbox)
}

fn plain(path: &str, dispatcher: &str, mailbox: &str) -> Deploy {
    deploy(
        path,
        "",
        RouterConfig::NoRouter,
        Scope::NoScopeGiven,
        dispatcher,
        mailbox,
    )
}

fn tags(names: &[&str]) -> impl Iterator<Item = String> {
    names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

fn scope(s: &Scope) -> &'static str {
    match s {
        Scope::Local => "Local",
        Scope::NoScopeGiven => "NoScopeGiven",
        _ => "other",
    }
}

fn router(r: &RouterConfig) -> String {
    match r {
        RouterConfig::NoRouter => "NoRouter".to_owned(),
        RouterConfig::Opaque(class_name) => class_name.clone(),
    }
}

fn show(d: &Deploy) -> String {
    let names: Vec<&str> = d.tags().iter().map(String::as_str).collect();
    format!(
        "path=\"{}\" config={} router={} scope={} dispatcher=\"{}\" mailbox=\"{}\" tags=[{}]",
        d.path(),
        d.config().render_concise(),
        router(d.router_config()),
        scope(d.scope()),
        d.dispatcher(),
        d.mailbox(),
        names.join(", ")
    )
}

fn with_config(config: &str) -> Deploy {
    deploy(
        "/a",
        config,
        RouterConfig::NoRouter,
        Scope::NoScopeGiven,
        "",
        "",
    )
}

/// `(escaped case id, escaped line)` in the registration order of `DeployOracle`.
pub fn rust_lines() -> Vec<(String, String)> {
    let ok = |text: String| format!("ok {text}");
    let yes = |same: bool| ok(same.to_string());
    let cases: Vec<(&str, String)> = vec![
        (
            "DP-01 Deploy() has the defaults",
            ok(show(&Deploy::default())),
        ),
        (
            "DP-02 Deploy.local has the local scope",
            ok(show(&Deploy::local())),
        ),
        (
            "DP-03 the fields are kept as given",
            ok(show(&deploy(
                "/user/a",
                "x = 1, y = \"s\"",
                pool("RoundRobinPool"),
                Scope::Local,
                "my-dispatcher",
                "my-mailbox",
            ))),
        ),
        (
            "DP-04 withTags keeps the other fields",
            ok(show(&plain("/a", "d", "m").with_tags(tags(&["t2", "t1"])))),
        ),
        (
            "DP-05 equal when the seven fields are equal",
            yes(
                deploy("/a", "x = 1", pool("RandomPool"), Scope::Local, "d", "m")
                    == deploy("/a", "x = 1", pool("RandomPool"), Scope::Local, "d", "m"),
            ),
        ),
        (
            "DP-06 different path",
            yes(plain("/a", "", "") == plain("/b", "", "")),
        ),
        (
            "DP-07 different dispatcher",
            yes(plain("/a", "d1", "") == plain("/a", "d2", "")),
        ),
        (
            "DP-08 different mailbox",
            yes(plain("/a", "", "m1") == plain("/a", "", "m2")),
        ),
        (
            "DP-09 different scope",
            yes(
                deploy("/a", "", RouterConfig::NoRouter, Scope::Local, "", "")
                    == deploy(
                        "/a",
                        "",
                        RouterConfig::NoRouter,
                        Scope::NoScopeGiven,
                        "",
                        "",
                    ),
            ),
        ),
        (
            "DP-10 different router",
            yes(deploy(
                "/a",
                "",
                pool("RoundRobinPool"),
                Scope::NoScopeGiven,
                "",
                "",
            ) == deploy(
                "/a",
                "",
                RouterConfig::NoRouter,
                Scope::NoScopeGiven,
                "",
                "",
            )),
        ),
        (
            "DP-11 different tags",
            yes(plain("/a", "", "").with_tags(tags(&["t"])) == plain("/a", "", "")),
        ),
        (
            "DP-12 same tags",
            yes(plain("/a", "", "").with_tags(tags(&["t", "u"]))
                == plain("/a", "", "").with_tags(tags(&["u", "t"]))),
        ),
        (
            "DP-13 different configuration",
            yes(with_config("x = 1") == with_config("x = 2")),
        ),
        (
            "DP-14 same configuration from other text",
            yes(with_config("x = 1") == with_config("\n\n  x = 1  ")),
        ),
        (
            "DP-15 same configuration written as a path or as an object",
            yes(with_config("a.b = 1") == with_config("a { b = 1 }")),
        ),
        (
            "DP-16 the whole number 1 equals 1.0",
            yes(with_config("x = 1") == with_config("x = 1.0")),
        ),
        (
            "DP-17 the number 1.5 does not equal the string 1.5",
            yes(with_config("x = 1.5") == with_config("x = \"1.5\"")),
        ),
        (
            "DP-18 lists of the same elements",
            yes(with_config("x = [1, 2]") == with_config("x = [1,2]")),
        ),
        (
            "DP-19 lists in another order",
            yes(with_config("x = [1, 2]") == with_config("x = [2, 1]")),
        ),
        (
            "DP-20 an empty object is not a missing key",
            yes(with_config("x = {}") == with_config("")),
        ),
        (
            "DP-21 one more key in the other configuration",
            yes(with_config("x = 1") == with_config("x = 1, y = 2")),
        ),
        (
            "FB-01 the path of the receiver is kept",
            ok(show(
                &plain("/a", "", "").with_fallback(&plain("/b", "", "")),
            )),
        ),
        (
            "FB-02 an empty path of the receiver is kept",
            ok(show(&plain("", "", "").with_fallback(&plain("/b", "", "")))),
        ),
        (
            "FB-03 the configuration of the receiver wins and the rest is merged",
            ok(show(&with_config("a = 1, n { x = 1 }").with_fallback(
                &with_config("a = 9, b = 2, n { x = 9, y = 2 }"),
            ))),
        ),
        (
            "FB-04 a given dispatcher wins",
            ok(show(
                &plain("/a", "mine", "").with_fallback(&plain("/b", "theirs", "")),
            )),
        ),
        (
            "FB-05 an empty dispatcher takes the other one",
            ok(show(
                &plain("/a", "", "").with_fallback(&plain("/b", "theirs", "")),
            )),
        ),
        (
            "FB-06 two empty dispatchers stay empty",
            ok(show(
                &plain("/a", "", "").with_fallback(&plain("/b", "", "")),
            )),
        ),
        (
            "FB-07 a given mailbox wins",
            ok(show(
                &plain("/a", "", "mine").with_fallback(&plain("/b", "", "theirs")),
            )),
        ),
        (
            "FB-08 an empty mailbox takes the other one",
            ok(show(
                &plain("/a", "", "").with_fallback(&plain("/b", "", "theirs")),
            )),
        ),
        (
            "FB-09 dispatcher and mailbox are decided one by one",
            ok(show(&plain("/a", "mine", "").with_fallback(&plain(
                "/b",
                "theirs",
                "their-mailbox",
            )))),
        ),
        (
            "FB-10 the tags are not carried",
            ok(show(
                &plain("/a", "", "")
                    .with_tags(tags(&["t1"]))
                    .with_fallback(&plain("/b", "", "").with_tags(tags(&["t2"]))),
            )),
        ),
        (
            "FB-11 the scope of the receiver wins when it is local",
            ok(show(
                &deploy("/a", "", RouterConfig::NoRouter, Scope::Local, "", "")
                    .with_fallback(&plain("/b", "", "")),
            )),
        ),
        (
            "FB-12 no scope given takes the scope of the other",
            ok(show(&plain("/a", "", "").with_fallback(&deploy(
                "/b",
                "",
                RouterConfig::NoRouter,
                Scope::Local,
                "",
                "",
            )))),
        ),
        (
            "FB-13 the router of the receiver wins",
            ok(show(
                &deploy(
                    "/a",
                    "",
                    pool("RoundRobinPool"),
                    Scope::NoScopeGiven,
                    "",
                    "",
                )
                .with_fallback(&plain("/b", "", "")),
            )),
        ),
        (
            "FB-14 no router takes the router of the other",
            ok(show(&plain("/a", "", "").with_fallback(&deploy(
                "/b",
                "",
                pool("RandomPool"),
                Scope::NoScopeGiven,
                "",
                "",
            )))),
        ),
        (
            "FB-15 chained fallbacks",
            ok(show(
                &plain("/a", "", "")
                    .with_fallback(&plain("/b", "d", ""))
                    .with_fallback(&plain("/c", "e", "m")),
            )),
        ),
        (
            "SC-01 local keeps itself over no scope given",
            ok(scope(&Scope::Local.with_fallback(&Scope::NoScopeGiven)).to_owned()),
        ),
        (
            "SC-02 local keeps itself over local",
            ok(scope(&Scope::Local.with_fallback(&Scope::Local)).to_owned()),
        ),
        (
            "SC-03 no scope given takes local",
            ok(scope(&Scope::NoScopeGiven.with_fallback(&Scope::Local)).to_owned()),
        ),
        (
            "SC-04 no scope given over no scope given",
            ok(scope(&Scope::NoScopeGiven.with_fallback(&Scope::NoScopeGiven)).to_owned()),
        ),
        (
            "RT-01 no router takes a router",
            ok(router(
                &RouterConfig::NoRouter.with_fallback(&pool("RoundRobinPool")),
            )),
        ),
        (
            "RT-02 a router over no router",
            ok(router(
                &pool("RoundRobinPool").with_fallback(&RouterConfig::NoRouter),
            )),
        ),
        (
            "RT-03 no router over no router",
            ok(router(
                &RouterConfig::NoRouter.with_fallback(&RouterConfig::NoRouter),
            )),
        ),
        (
            "RT-04 a router over another router keeps its class",
            ok(router(
                &pool("RoundRobinPool").with_fallback(&pool("RandomPool")),
            )),
        ),
    ];
    cases
        .into_iter()
        .map(|(id, text)| {
            let line = format!("{id} -> {text}");
            (escape_non_ascii_units(id), escape_non_ascii_units(&line))
        })
        .collect()
}

use crate::text_escape::escape_non_ascii_units;
