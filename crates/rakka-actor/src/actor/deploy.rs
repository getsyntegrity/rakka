// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Deployer.scala (Apache-2.0).
//! `Deploy` and `Scope`: the final, combinable description of how an actor is deployed
//! (`Deployer.scala:28-205`), and the router slot it carries.

use std::collections::BTreeSet;

use crate::config::Config;

/// `Deploy.NoDispatcherGiven`.
pub const NO_DISPATCHER_GIVEN: &str = "";
/// `Deploy.NoMailboxGiven`.
pub const NO_MAILBOX_GIVEN: &str = "";

/// `Scope`: where a deployment is created. Pekko leaves the trait open so that a remote module can add its own; here
/// only the two scopes of the actor module exist, and the enum is `non_exhaustive` so that another one can be added later.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scope {
    /// `LocalScope`.
    Local,
    /// `NoScopeGiven`, the default.
    NoScopeGiven,
}

impl Scope {
    /// `withFallback`: `LocalScope` keeps itself, `NoScopeGiven` takes the other scope.
    pub fn with_fallback(&self, other: &Scope) -> Scope {
        match self {
            Scope::Local => Scope::Local,
            Scope::NoScopeGiven => other.clone(),
        }
    }
}

/// The router of a [`Deploy`]. Only what the deployer needs to carry a router around: no router (`NoRouter`, the
/// default and what `from-code` means) or a router that is named but not built here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouterConfig {
    /// `NoRouter`.
    NoRouter,
    /// A router named by its class, whose settings and merge rules belong to the routing module.
    Opaque(String),
}

impl RouterConfig {
    /// A router that is only named.
    pub fn opaque(class_name: impl Into<String>) -> Self {
        RouterConfig::Opaque(class_name.into())
    }

    /// `withFallback`: `NoRouter` takes the other router; any other router keeps itself.
    pub fn with_fallback(&self, other: &RouterConfig) -> RouterConfig {
        match self {
            RouterConfig::NoRouter => other.clone(),
            RouterConfig::Opaque(_) => self.clone(),
        }
    }
}

/// `Deploy`: path, configuration, router, scope, dispatcher, mailbox and tags of one deployment.
#[derive(Clone, Debug)]
pub struct Deploy {
    path: String,
    config: Config,
    router_config: RouterConfig,
    scope: Scope,
    dispatcher: String,
    mailbox: String,
    tags: BTreeSet<String>,
}

impl Default for Deploy {
    /// `Deploy()`.
    fn default() -> Self {
        Deploy {
            path: String::new(),
            config: Config::empty(),
            router_config: RouterConfig::NoRouter,
            scope: Scope::NoScopeGiven,
            dispatcher: NO_DISPATCHER_GIVEN.to_owned(),
            mailbox: NO_MAILBOX_GIVEN.to_owned(),
            tags: BTreeSet::new(),
        }
    }
}

impl PartialEq for Deploy {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.config.same_value(&other.config)
            && self.router_config == other.router_config
            && self.scope == other.scope
            && self.dispatcher == other.dispatcher
            && self.mailbox == other.mailbox
            && self.tags == other.tags
    }
}

impl Deploy {
    /// `Deploy(path, config, routerConfig, scope, dispatcher, mailbox)`: a deployment without tags.
    pub fn new(
        path: &str,
        config: Config,
        router_config: RouterConfig,
        scope: Scope,
        dispatcher: &str,
        mailbox: &str,
    ) -> Self {
        Deploy {
            path: path.to_owned(),
            config,
            router_config,
            scope,
            dispatcher: dispatcher.to_owned(),
            mailbox: mailbox.to_owned(),
            tags: BTreeSet::new(),
        }
    }

    /// `Deploy.local`: `Deploy(scope = LocalScope)`.
    pub fn local() -> Self {
        Deploy {
            scope: Scope::Local,
            ..Deploy::default()
        }
    }

    /// `withTags`.
    pub fn with_tags(&self, tags: impl IntoIterator<Item = String>) -> Self {
        Deploy {
            tags: tags.into_iter().collect(),
            ..self.clone()
        }
    }

    /// `path`.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// `config`.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// `routerConfig`.
    pub fn router_config(&self) -> &RouterConfig {
        &self.router_config
    }

    /// `scope`.
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    /// `dispatcher`.
    pub fn dispatcher(&self) -> &str {
        &self.dispatcher
    }

    /// `mailbox`.
    pub fn mailbox(&self) -> &str {
        &self.mailbox
    }

    /// `tags`.
    pub fn tags(&self) -> &BTreeSet<String> {
        &self.tags
    }

    /// `withFallback`: the values of `self` win.
    pub fn with_fallback(&self, other: &Deploy) -> Deploy {
        Deploy {
            path: self.path.clone(),
            config: self.config.with_fallback(&other.config),
            router_config: self.router_config.with_fallback(&other.router_config),
            scope: self.scope.with_fallback(&other.scope),
            dispatcher: if self.dispatcher == NO_DISPATCHER_GIVEN {
                other.dispatcher.clone()
            } else {
                self.dispatcher.clone()
            },
            mailbox: if self.mailbox == NO_MAILBOX_GIVEN {
                other.mailbox.clone()
            } else {
                self.mailbox.clone()
            },
            tags: BTreeSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conf(text: &str) -> Config {
        Config::parse_string(text).expect("a valid configuration")
    }

    fn deploy(path: &str, dispatcher: &str, mailbox: &str) -> Deploy {
        Deploy::new(
            path,
            Config::empty(),
            RouterConfig::NoRouter,
            Scope::NoScopeGiven,
            dispatcher,
            mailbox,
        )
    }

    #[test]
    fn a_default_deploy_has_no_router_no_scope_and_empty_names() {
        let d = Deploy::default();
        assert_eq!(d.path(), "");
        assert!(d.config().same_value(&Config::empty()));
        assert_eq!(d.router_config(), &RouterConfig::NoRouter);
        assert_eq!(d.scope(), &Scope::NoScopeGiven);
        assert_eq!(d.dispatcher(), NO_DISPATCHER_GIVEN);
        assert_eq!(d.mailbox(), NO_MAILBOX_GIVEN);
        assert!(d.tags().is_empty());
    }

    #[test]
    fn the_local_deploy_is_the_default_with_the_local_scope() {
        let local = Deploy::local();
        assert_eq!(local.scope(), &Scope::Local);
        assert_eq!(local.path(), "");
        assert_eq!(local.router_config(), &RouterConfig::NoRouter);
    }

    #[test]
    fn the_dispatcher_and_mailbox_of_the_receiver_win_unless_they_are_empty() {
        let this = deploy("/a", "mine", "");
        let other = deploy("/b", "theirs", "their-mailbox");
        let merged = this.with_fallback(&other);
        assert_eq!(merged.dispatcher(), "mine");
        assert_eq!(merged.mailbox(), "their-mailbox");
        assert_eq!(
            deploy("/a", "", "").with_fallback(&other).dispatcher(),
            "theirs"
        );
        assert_eq!(
            deploy("/a", "", "")
                .with_fallback(&deploy("/b", "", ""))
                .dispatcher(),
            ""
        );
    }

    #[test]
    fn the_path_of_the_receiver_is_kept_and_the_path_of_the_fallback_is_ignored() {
        assert_eq!(
            deploy("/a", "", "")
                .with_fallback(&deploy("/b", "", ""))
                .path(),
            "/a"
        );
        assert_eq!(
            deploy("", "", "")
                .with_fallback(&deploy("/b", "", ""))
                .path(),
            ""
        );
    }

    #[test]
    fn the_configurations_are_merged_with_the_receiver_first() {
        let this = Deploy::new(
            "/a",
            conf("a = 1"),
            RouterConfig::NoRouter,
            Scope::NoScopeGiven,
            "",
            "",
        );
        let other = Deploy::new(
            "/b",
            conf("a = 9\nb = 2"),
            RouterConfig::NoRouter,
            Scope::NoScopeGiven,
            "",
            "",
        );
        let merged = this.with_fallback(&other);
        assert_eq!(merged.config().get_int("a"), Ok(1));
        assert_eq!(merged.config().get_int("b"), Ok(2));
    }

    #[test]
    fn the_tags_are_not_carried_by_a_fallback() {
        let this = deploy("/a", "", "").with_tags(["t1".to_owned()]);
        let other = deploy("/b", "", "").with_tags(["t2".to_owned()]);
        assert!(this.with_fallback(&other).tags().is_empty());
    }

    #[test]
    fn the_local_scope_keeps_itself_and_no_scope_given_takes_the_other() {
        assert_eq!(
            Scope::Local.with_fallback(&Scope::NoScopeGiven),
            Scope::Local
        );
        assert_eq!(Scope::Local.with_fallback(&Scope::Local), Scope::Local);
        assert_eq!(
            Scope::NoScopeGiven.with_fallback(&Scope::Local),
            Scope::Local
        );
        assert_eq!(
            Scope::NoScopeGiven.with_fallback(&Scope::NoScopeGiven),
            Scope::NoScopeGiven
        );
    }

    #[test]
    fn no_router_takes_the_other_router_and_a_router_keeps_itself() {
        let pool = RouterConfig::opaque("RoundRobinPool");
        let other_pool = RouterConfig::opaque("RandomPool");
        assert_eq!(RouterConfig::NoRouter.with_fallback(&pool), pool);
        assert_eq!(pool.with_fallback(&RouterConfig::NoRouter), pool);
        assert_eq!(pool.with_fallback(&other_pool), pool);
        assert_eq!(
            RouterConfig::NoRouter.with_fallback(&RouterConfig::NoRouter),
            RouterConfig::NoRouter
        );
    }

    #[test]
    fn two_deploys_with_the_same_content_are_equal_whatever_the_text_of_their_configuration() {
        let a = Deploy::new(
            "/a",
            conf("x = 1"),
            RouterConfig::NoRouter,
            Scope::Local,
            "d",
            "m",
        );
        let b = Deploy::new(
            "/a",
            conf("\n\nx = 1"),
            RouterConfig::NoRouter,
            Scope::Local,
            "d",
            "m",
        );
        assert_eq!(a, b);
        assert_ne!(a, deploy("/a", "d", "m"));
        assert_ne!(a, a.with_tags(["t".to_owned()]));
    }
}
