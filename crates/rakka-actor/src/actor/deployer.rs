// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! The `Deployer` (`Deployer.scala:225-276`): the deployments of an actor system by path, found with the wildcards `*`,
//! `prefix*` and a terminal `**`, and read from `pekko.actor.deployment`. A router that is not `from-code` is only named
//! here; building routers (type mapping, pools, resizers) belongs to the routing module.

use std::sync::{Arc, Mutex, PoisonError};

use crate::actor::{
    ActorError, ActorPath, Deploy, InvalidActorNameError, RouterConfig, Scope,
    validate_path_element,
};
use crate::config::{Config, ConfigError, Settings};
use crate::util::wildcard_index::WildcardIndex;

/// What building a [`Deployer`] from settings can throw: a configuration error from reading `pekko.actor.deployment`, or the
/// rejection of one deployment path by [`Deployer::deploy`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeployerError {
    /// `ConfigException` (a missing key or a value of the wrong type).
    Config(ConfigError),
    /// `InvalidActorNameException` or `IllegalArgumentException` from `deploy`.
    Deploy(ActorError),
}

impl DeployerError {
    /// The message of the wrapped error (`getMessage`).
    pub fn message(&self) -> &str {
        match self {
            Self::Config(e) => e.message(),
            Self::Deploy(e) => e.message(),
        }
    }

    /// The JVM class of the wrapped error, without the package.
    pub fn simple_class_name(&self) -> &'static str {
        match self {
            Self::Config(e) => e.kind().java_name().rsplit('.').next().unwrap_or_default(),
            Self::Deploy(e) => e.class().java_name().rsplit('.').next().unwrap_or_default(),
        }
    }
}

impl std::fmt::Display for DeployerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(e) => e.fmt(f),
            Self::Deploy(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for DeployerError {}

impl From<ConfigError> for DeployerError {
    fn from(e: ConfigError) -> Self {
        Self::Config(e)
    }
}

impl From<ActorError> for DeployerError {
    fn from(e: ActorError) -> Self {
        Self::Deploy(e)
    }
}

/// `Deployer`: maps actor paths to deployments. Its index is an immutable snapshot that `deploy` replaces as a whole
/// (the `Mutex` only guards the swap, never the building of the new index), so a lookup never waits for a deploy, never
/// sees a deployment half added, and a failed `deploy` changes nothing.
#[derive(Debug, Default)]
pub struct Deployer {
    deployments: Mutex<Arc<WildcardIndex<Deploy>>>,
}

impl Deployer {
    /// A deployer without deployments.
    pub fn new() -> Self {
        Self::default()
    }

    /// `new Deployer(settings, dynamicAccess)`: the deployments of `pekko.actor.deployment`.
    pub fn from_settings(settings: &Settings) -> Result<Self, DeployerError> {
        let section = settings.config().get_config("pekko.actor.deployment")?;
        let default = section.get_config("default")?;
        // `flatMap` over the entries reads every one of them before `foreach(deploy)` adds the first: a configuration
        // error in any entry is thrown before any path is rejected. The entries are taken in the alphabetical order of
        // their keys: a temporary policy, where Pekko uses the iteration order of a JVM `HashSet` of entries (hash of key
        // and value).
        let deploys = section
            .object_entries()
            .into_iter()
            .filter(|(key, _)| key != "default")
            .map(|(key, entry)| Self::parse_config(&key, &entry, &default))
            .collect::<Result<Vec<_>, _>>()?;
        let deployer = Self::new();
        for deploy in deploys {
            deployer.deploy(deploy)?;
        }
        Ok(deployer)
    }

    /// `parseConfig(key, config)`: the entry over `default`, with the router named but not built (`from-code` is no
    /// router; any other is kept by the name that is configured, which is all the routing module will need to build it).
    fn parse_config(key: &str, entry: &Config, default: &Config) -> Result<Deploy, ConfigError> {
        let deployment = entry.with_fallback(default);
        let router = match deployment.get_string("router")?.as_str() {
            "from-code" => RouterConfig::NoRouter,
            name => RouterConfig::opaque(name),
        };
        let dispatcher = deployment.get_string("dispatcher")?;
        let mailbox = deployment.get_string("mailbox")?;
        Ok(Deploy::new(
            key,
            deployment,
            router,
            Scope::NoScopeGiven,
            &dispatcher,
            &mailbox,
        ))
    }

    /// `deploy(d)`.
    ///
    /// The path is split like `d.path.split("/").drop(1)`; each element must be non-empty and a valid path element,
    /// and the new index is built before it replaces the current one, so a rejected deployment adds nothing.
    pub fn deploy(&self, deploy: Deploy) -> Result<(), ActorError> {
        self.deploy_between_read_and_swap(deploy, || {})
    }

    /// `deploy` with a hook that runs after the new index was built from a snapshot and before it is swapped in, which
    /// is where Pekko's `compareAndSet` can lose: if another deployment was swapped in meanwhile, the snapshot is
    /// stale, the swap is refused and the whole step is retried on the newer index, so no deployment is lost.
    /// The hook is a no-op in production; a test uses it to make that interleaving happen deterministically.
    fn deploy_between_read_and_swap(
        &self,
        deploy: Deploy,
        mut between_read_and_swap: impl FnMut(),
    ) -> Result<(), ActorError> {
        let path = deploy.path().to_owned();
        let parts = java_split_slash(&path);
        let elements = parts.get(1..).unwrap_or_default();
        for element in elements {
            if element.is_empty() {
                return Err(InvalidActorNameError::new(format!(
                    "Actor name in deployment [{path}] must not be empty"
                ))
                .into());
            }
            validate_path_element(element, Some(&path))?;
        }
        loop {
            let snapshot = self.snapshot();
            let updated = Arc::new(snapshot.insert(elements, deploy.clone())?);
            between_read_and_swap();
            let mut current = self
                .deployments
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if Arc::ptr_eq(&current, &snapshot) {
                *current = updated;
                return Ok(());
            }
        }
    }

    /// `deployments.get`: the index as it is now, which no later `deploy` changes (it replaces it instead).
    fn snapshot(&self) -> Arc<WildcardIndex<Deploy>> {
        Arc::clone(
            &self
                .deployments
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }

    /// `lookup(path: Iterable[String])`: the deployment for these elements, which are looked up as given.
    pub fn lookup<'a>(&self, elements: impl IntoIterator<Item = &'a str>) -> Option<Deploy> {
        self.snapshot().find(elements).cloned()
    }

    /// `lookup(path: ActorPath)`: like [`Deployer::lookup`] without the first element, the guardian (`user`, `system`...).
    pub fn lookup_path(&self, path: &ActorPath) -> Option<Deploy> {
        self.lookup(path.elements().into_iter().skip(1))
    }
}

/// `String.split("/")` of Java: the trailing empty parts are dropped, so `"/"` has none. (Java also gives `[""]` for the
/// empty text, which this does not: `deploy` drops the first part, so the difference cannot be seen.)
fn java_split_slash(text: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = text.split('/').collect();
    while parts.last().is_some_and(|part| part.is_empty()) {
        parts.pop();
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::{RouterConfig, Scope};
    use crate::config::Config;
    use std::sync::Barrier;
    use std::thread;

    fn deploy_at(path: &str, dispatcher: &str) -> Deploy {
        Deploy::new(
            path,
            Config::empty(),
            RouterConfig::NoRouter,
            Scope::NoScopeGiven,
            dispatcher,
            "",
        )
    }

    fn deployer_with(paths: &[&str]) -> Deployer {
        let deployer = Deployer::new();
        for path in paths {
            deployer
                .deploy(deploy_at(path, path))
                .expect("a valid deployment");
        }
        deployer
    }

    fn found(deployer: &Deployer, elements: &[&str]) -> Option<String> {
        deployer
            .lookup(elements.iter().copied())
            .map(|deploy| deploy.path().to_owned())
    }

    fn lookup_uri(deployer: &Deployer, uri: &str) -> Option<String> {
        let path = ActorPath::from_string(uri).expect("a valid actor path");
        deployer
            .lookup_path(&path)
            .map(|deploy| deploy.path().to_owned())
    }

    fn settings_with(deployment: &str) -> Settings {
        let user = Config::parse_string(&format!("pekko.actor.deployment {{\n{deployment}\n}}"))
            .expect("a valid configuration");
        let config = user
            .with_fallback(&Config::default_reference())
            .resolve()
            .expect("a resolvable configuration");
        Settings::new("deployer-test", config).expect("valid settings")
    }

    fn deployer_with_deployment(deployment: &str) -> Deployer {
        Deployer::from_settings(&settings_with(deployment)).expect("a valid deployment section")
    }

    fn build_error(deployment: &str) -> DeployerError {
        Deployer::from_settings(&settings_with(deployment))
            .expect_err("a rejected deployment section")
    }

    #[test]
    fn an_entry_without_values_takes_everything_from_default() {
        let deployer = deployer_with_deployment("/service1 {}");
        let deploy = deployer
            .lookup(["service1"])
            .expect("the entry is deployed");
        assert_eq!(deploy.path(), "/service1");
        assert_eq!(deploy.router_config(), &RouterConfig::NoRouter);
        assert_eq!(deploy.scope(), &Scope::NoScopeGiven);
        assert_eq!(deploy.dispatcher(), "");
        assert_eq!(deploy.mailbox(), "");
        assert_eq!(deploy.config().get_string("router").unwrap(), "from-code");
        assert_eq!(deploy.config().get_int("nr-of-instances").unwrap(), 1);
    }

    #[test]
    fn an_entry_that_is_not_defined_is_not_found() {
        assert!(
            deployer_with_deployment("/service1 {}")
                .lookup(["undefined"])
                .is_none()
        );
    }

    #[test]
    fn the_dispatcher_and_the_mailbox_of_an_entry_are_read_from_its_configuration() {
        let deployer = deployer_with_deployment(
            "/service3 { dispatcher = my-dispatcher }\n/service4 { mailbox = my-mailbox }",
        );
        let with_dispatcher = deployer.lookup(["service3"]).unwrap();
        assert_eq!(with_dispatcher.dispatcher(), "my-dispatcher");
        assert_eq!(with_dispatcher.mailbox(), "");
        let with_mailbox = deployer.lookup(["service4"]).unwrap();
        assert_eq!(with_mailbox.dispatcher(), "");
        assert_eq!(with_mailbox.mailbox(), "my-mailbox");
    }

    #[test]
    fn the_dispatcher_of_default_is_the_fallback_of_every_entry_and_the_entry_wins() {
        let deployer = deployer_with_deployment(
            "default.dispatcher = shared\n/inherits {}\n/overrides { dispatcher = own }",
        );
        assert_eq!(
            deployer.lookup(["inherits"]).unwrap().dispatcher(),
            "shared"
        );
        assert_eq!(deployer.lookup(["overrides"]).unwrap().dispatcher(), "own");
    }

    #[test]
    fn the_default_entry_itself_is_not_a_deployment() {
        assert!(
            deployer_with_deployment("/a {}")
                .lookup(["default"])
                .is_none()
        );
    }

    #[test]
    fn the_entries_of_the_reference_configuration_are_deployed() {
        let deployer = deployer_with_deployment("/a {}");
        let resolver = deployer.lookup(["IO-DNS", "inet-address"]).unwrap();
        assert_eq!(resolver.mailbox(), "unbounded");
        assert_eq!(
            resolver.router_config(),
            &RouterConfig::opaque("consistent-hashing-pool")
        );
        let nested = deployer.lookup(["IO-DNS", "inet-address", "any"]).unwrap();
        assert_eq!(
            nested.dispatcher(),
            "pekko.actor.default-blocking-io-dispatcher"
        );
    }

    #[test]
    fn a_router_that_is_not_from_code_keeps_the_configured_name() {
        let deployer = deployer_with_deployment("/pool { router = round-robin-pool }");
        assert_eq!(
            deployer.lookup(["pool"]).unwrap().router_config(),
            &RouterConfig::opaque("round-robin-pool")
        );
    }

    #[test]
    fn an_entry_that_is_not_an_object_is_ignored() {
        let deployer = deployer_with_deployment("/scalar = 5\n/nothing = null\n/real {}");
        assert!(deployer.lookup(["scalar"]).is_none());
        assert!(deployer.lookup(["nothing"]).is_none());
        assert!(deployer.lookup(["real"]).is_some());
    }

    #[test]
    fn the_wildcards_of_the_entries_are_looked_up_like_any_deployment() {
        let deployer = deployer_with_deployment(
            "\"/some/*\" { dispatcher = d1 }\n\"/double/**\" { dispatcher = d2 }",
        );
        assert_eq!(deployer.lookup(["some", "x"]).unwrap().dispatcher(), "d1");
        assert_eq!(
            deployer.lookup(["double", "a", "b"]).unwrap().dispatcher(),
            "d2"
        );
        assert!(deployer.lookup(["double"]).is_none());
    }

    #[test]
    fn a_value_of_the_wrong_type_is_a_configuration_error() {
        let error = build_error("/bad { dispatcher = [1] }");
        assert_eq!(error.simple_class_name(), "WrongType");
        let error = build_error("/bad { router = { a = 1 } }");
        assert_eq!(error.simple_class_name(), "WrongType");
    }

    #[test]
    fn an_invalid_deployment_path_is_rejected_when_the_deployer_is_built() {
        let error = build_error("\"/gul/ub\u{e5}t\" {}");
        assert_eq!(error.simple_class_name(), "InvalidActorNameException");
        assert!(
            error.message().contains("[ub\u{e5}t]") && error.message().contains("[/gul/ub\u{e5}t]")
        );
        let error = build_error("\"/c**\" {}");
        assert_eq!(error.simple_class_name(), "IllegalArgumentException");
    }

    #[test]
    fn the_error_of_building_a_deployer_displays_the_wrapped_error() {
        let wrong_type = build_error("/bad { dispatcher = [1] }");
        assert_eq!(
            wrong_type.to_string(),
            format!("WrongType: {}", wrong_type.message())
        );
        assert!(
            wrong_type
                .message()
                .ends_with("dispatcher has type LIST rather than STRING")
        );
        let invalid_name = build_error("\"/a//b\" {}");
        assert_eq!(
            invalid_name.to_string(),
            "org.apache.pekko.actor.InvalidActorNameException: Actor name in deployment [/a//b] must not be empty"
        );
    }

    #[test]
    fn every_entry_is_read_before_any_is_deployed_so_a_configuration_error_wins_over_an_invalid_path()
     {
        let error = build_error("\"/a b\" {}\n/z { dispatcher = [1] }");
        assert_eq!(error.simple_class_name(), "WrongType");
    }

    // Temporary policy: Pekko deploys the entries in the iteration order of a JVM `HashSet` of entries (hash of key and
    // value); here they go in the alphabetical order of their keys, so of several keys that give the same path the last one
    // wins. This test pins the Rust policy, not Pekko's behaviour.
    #[test]
    fn of_several_keys_with_the_same_path_the_last_in_alphabetical_order_wins() {
        let deployer = deployer_with_deployment(
            "\"/svc\" { dispatcher = plain }\n\"/svc/\" { dispatcher = slash }\n\"/svc//\" { dispatcher = two }",
        );
        assert_eq!(deployer.lookup(["svc"]).unwrap().dispatcher(), "two");
        let deployer = deployer_with_deployment(
            "\"/a/\" { dispatcher = slash }\n\"/a\" { dispatcher = plain }",
        );
        assert_eq!(deployer.lookup(["a"]).unwrap().dispatcher(), "slash");
    }

    #[test]
    fn split_keeps_a_leading_empty_part_and_drops_the_trailing_ones() {
        assert_eq!(java_split_slash("/a/b"), ["", "a", "b"]);
        assert_eq!(java_split_slash("a/b"), ["a", "b"]);
        assert_eq!(java_split_slash("/a//"), ["", "a"]);
        assert_eq!(java_split_slash("/a//b"), ["", "a", "", "b"]);
        assert_eq!(java_split_slash("//a"), ["", "", "a"]);
    }

    #[test]
    fn split_of_only_slashes_has_no_part_and_of_no_slash_has_the_text() {
        assert_eq!(java_split_slash("/"), Vec::<&str>::new());
        assert_eq!(java_split_slash("///"), Vec::<&str>::new());
        assert_eq!(java_split_slash("abc"), ["abc"]);
    }

    #[test]
    fn a_deployment_is_found_by_its_path_without_the_first_element() {
        let deployer = deployer_with(&["/service1"]);
        assert_eq!(
            found(&deployer, &["service1"]),
            Some("/service1".to_owned())
        );
        assert_eq!(found(&deployer, &["service2"]), None);
        assert_eq!(found(&deployer, &["user", "service1"]), None);
    }

    #[test]
    fn the_first_element_of_the_deployed_path_is_dropped_even_without_a_leading_slash() {
        let deployer = deployer_with(&["a/b"]);
        assert_eq!(found(&deployer, &["b"]), Some("a/b".to_owned()));
        assert_eq!(found(&deployer, &["a", "b"]), None);
    }

    #[test]
    fn the_guardian_of_an_actor_path_is_dropped_by_lookup() {
        let deployer = deployer_with(&["/service1"]);
        assert_eq!(
            lookup_uri(&deployer, "pekko://sys/user/service1"),
            Some("/service1".to_owned())
        );
        assert_eq!(
            lookup_uri(&deployer, "pekko://sys/system/service1"),
            Some("/service1".to_owned())
        );
        assert_eq!(lookup_uri(&deployer, "pekko://sys/service1"), None);
        assert_eq!(lookup_uri(&deployer, "pekko://sys"), None);
    }

    #[test]
    fn a_single_wildcard_matches_exactly_one_element() {
        let deployer = deployer_with(&["/some/*", "/*/other"]);
        assert_eq!(found(&deployer, &["some", "x"]), Some("/some/*".to_owned()));
        assert_eq!(found(&deployer, &["some"]), None);
        assert_eq!(found(&deployer, &["some", "x", "y"]), None);
        assert_eq!(
            found(&deployer, &["x", "other"]),
            Some("/*/other".to_owned())
        );
    }

    #[test]
    fn a_terminal_double_wildcard_matches_what_follows_the_prefix_and_the_most_specific_wins() {
        let deployer = deployer_with(&["/double/**", "/double/more/**"]);
        assert_eq!(found(&deployer, &["double"]), None);
        assert_eq!(
            found(&deployer, &["double", "a", "b"]),
            Some("/double/**".to_owned())
        );
        assert_eq!(
            found(&deployer, &["double", "more", "x"]),
            Some("/double/more/**".to_owned())
        );
    }

    #[test]
    fn an_exact_path_beats_a_single_wildcard_and_that_beats_a_double_wildcard() {
        let deployer = deployer_with(&["/a/**", "/a/*", "/a/b"]);
        assert_eq!(found(&deployer, &["a", "b"]), Some("/a/b".to_owned()));
        assert_eq!(found(&deployer, &["a", "c"]), Some("/a/*".to_owned()));
        assert_eq!(found(&deployer, &["a", "c", "d"]), Some("/a/**".to_owned()));
    }

    #[test]
    fn a_prefix_wildcard_matches_the_elements_that_start_with_the_prefix() {
        let deployer = deployer_with(&["/q", "/a/ab*"]);
        assert_eq!(found(&deployer, &["a", "abc"]), Some("/a/ab*".to_owned()));
        assert_eq!(found(&deployer, &["a", "ab"]), Some("/a/ab*".to_owned()));
        assert_eq!(found(&deployer, &["a", "a"]), None);
    }

    #[test]
    fn deploying_the_same_path_again_replaces_the_deployment() {
        let deployer = Deployer::new();
        deployer.deploy(deploy_at("/a", "first")).unwrap();
        deployer.deploy(deploy_at("/a", "second")).unwrap();
        assert_eq!(deployer.lookup(["a"]).unwrap().dispatcher(), "second");
    }

    #[test]
    fn the_root_the_empty_path_and_one_element_are_not_deployments() {
        for path in ["/", "", "///", "a"] {
            let deployer = deployer_with(&[path]);
            assert_eq!(found(&deployer, &["a"]), None, "{path:?}");
            assert_eq!(found(&deployer, &[""]), None, "{path:?}");
        }
    }

    #[test]
    fn an_empty_element_is_rejected_with_the_whole_path_in_the_message() {
        let error = Deployer::new().deploy(deploy_at("/a//b", "")).unwrap_err();
        assert_eq!(
            error.to_string(),
            "org.apache.pekko.actor.InvalidActorNameException: Actor name in deployment [/a//b] must not be empty"
        );
    }

    #[test]
    fn the_empty_element_after_the_first_is_reported_before_a_later_invalid_one() {
        let error = Deployer::new()
            .deploy(deploy_at("/a//\u{e5}", ""))
            .unwrap_err();
        assert_eq!(
            error.message(),
            "Actor name in deployment [/a//\u{e5}] must not be empty"
        );
    }

    #[test]
    fn an_invalid_element_is_rejected_with_the_element_and_the_path() {
        let error = Deployer::new()
            .deploy(deploy_at("/gul/ub\u{e5}t", ""))
            .unwrap_err();
        assert!(error.message().starts_with("Invalid actor path element [ub\u{e5}t] (in path [/gul/ub\u{e5}t]), illegal character [\u{e5}] at position: 2."));
    }

    #[test]
    fn a_double_wildcard_suffix_is_an_illegal_argument_and_nothing_is_added() {
        let deployer = deployer_with(&["/a/b"]);
        let error = deployer.deploy(deploy_at("/c**", "")).unwrap_err();
        assert_eq!(
            error.class().java_name(),
            "java.lang.IllegalArgumentException"
        );
        assert_eq!(found(&deployer, &["c"]), None);
        assert_eq!(found(&deployer, &["a", "b"]), Some("/a/b".to_owned()));
    }

    #[test]
    fn a_rejected_deployment_leaves_the_previous_ones_alone() {
        let deployer = deployer_with(&["/a/b"]);
        assert!(deployer.deploy(deploy_at("/a/\u{e5}", "")).is_err());
        assert!(deployer.deploy(deploy_at("/a//c", "")).is_err());
        assert_eq!(found(&deployer, &["a", "b"]), Some("/a/b".to_owned()));
        assert_eq!(found(&deployer, &["a", "x"]), None);
    }

    #[test]
    fn a_deployment_that_loses_the_swap_is_retried_on_the_newer_index_and_nothing_is_lost() {
        let deployer = Deployer::new();
        let mut hook_runs = 0;
        deployer
            .deploy_between_read_and_swap(deploy_at("/x/first", "first"), || {
                hook_runs += 1;
                if hook_runs == 1 {
                    // another deployment is swapped in after the first one read its snapshot
                    deployer.deploy(deploy_at("/x/second", "second")).unwrap();
                }
            })
            .unwrap();
        assert_eq!(hook_runs, 2, "the stale swap was refused and retried once");
        assert_eq!(
            found(&deployer, &["x", "first"]),
            Some("/x/first".to_owned())
        );
        assert_eq!(
            found(&deployer, &["x", "second"]),
            Some("/x/second".to_owned())
        );
    }

    #[test]
    fn concurrent_deployments_of_different_paths_are_all_kept() {
        let threads = 8;
        let per_thread = 25;
        let deployer = Deployer::new();
        let start = Barrier::new(threads);
        thread::scope(|scope| {
            for thread_number in 0..threads {
                let (deployer, start) = (&deployer, &start);
                scope.spawn(move || {
                    start.wait();
                    for n in 0..per_thread {
                        deployer
                            .deploy(deploy_at(&format!("/t{thread_number}/n{n}"), "d"))
                            .unwrap();
                    }
                });
            }
        });
        for thread_number in 0..threads {
            for n in 0..per_thread {
                let (first, second) = (format!("t{thread_number}"), format!("n{n}"));
                assert!(deployer.lookup([first.as_str(), second.as_str()]).is_some());
            }
        }
    }
}
