// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! `Deployer.deploy` and `Deployer.lookup` (AC-CFG-012.2, `Deployer.scala:252-268`): the deployments of an actor system
//! by path, found with the wildcards `*`, `prefix*` and a terminal `**`. Reading `pekko.actor.deployment` into deployments
//! (`parseConfig`, AC-CFG-012.1) and building the routers (AC-CFG-012.3/.4) are other slices.

use std::sync::{Arc, Mutex, PoisonError};

use crate::actor::{ActorError, ActorPath, Deploy, InvalidActorNameError, validate_path_element};
use crate::util::wildcard_index::WildcardIndex;

/// `Deployer`: maps actor paths to deployments. Its index is an immutable snapshot that `deploy` replaces as a whole,
/// so a lookup never sees a deployment half added and a failed `deploy` changes nothing.
#[derive(Debug, Default)]
pub struct Deployer {
    deployments: Mutex<Arc<WildcardIndex<Deploy>>>,
}

impl Deployer {
    /// A deployer without deployments.
    pub fn new() -> Self {
        Self::default()
    }

    /// `deploy(d)`.
    ///
    /// The path is split like `d.path.split("/").drop(1)`; each element must be non-empty and a valid path element,
    /// and the new index is built before it replaces the current one, so a rejected deployment adds nothing. Two
    /// threads deploying at once are serialised, which gives the same result as Pekko's retry on a lost
    /// `compareAndSet`: every deployment is kept.
    pub fn deploy(&self, deploy: Deploy) -> Result<(), ActorError> {
        let path = deploy.path().to_owned();
        let parts = java_split_slash(&path);
        let elements = parts.get(1..).unwrap_or_default();
        for element in elements {
            if element.is_empty() {
                return Err(InvalidActorNameError::new(format!(
                    "Actor name in deployment [{}] must not be empty",
                    path
                ))
                .into());
            }
            validate_path_element(element, Some(&path))?;
        }
        let mut current = self
            .deployments
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let updated = current.insert(elements, deploy)?;
        *current = Arc::new(updated);
        Ok(())
    }

    /// `lookup(path: Iterable[String])`: the deployment for these elements, which are looked up as given.
    pub fn lookup<'a>(&self, elements: impl IntoIterator<Item = &'a str>) -> Option<Deploy> {
        let snapshot = Arc::clone(
            &self
                .deployments
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        snapshot.find(elements).cloned()
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
