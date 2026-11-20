// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/setup/ActorSystemSetup.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Programmatic configuration of an actor system (AC-CFG-015): [`Setup`], [`ActorSystemSetup`] and [`BootstrapSetup`].
//!
//! `ActorSystemSetup` holds at most one setup per concrete type (Pekko indexes by `getClass`, Q-CFG-13 (a): here by
//! `TypeId`); adding a second setup of the same type replaces the first one in its position. `BootstrapSetup` carries
//! the core bootstrap settings: the configuration and the actor-ref provider. Pekko's `classLoader` and
//! `defaultExecutionContext` have no Rust counterpart and are not part of it (Q-CFG-32).

use std::any::{Any, TypeId};
use std::fmt;
use std::sync::Arc;

use super::{Config, ProviderSelection};

/// A programmatic setting of an actor system (`Setup`): a type that can be stored in an [`ActorSystemSetup`].
pub trait Setup: Any + Send + Sync {
    /// The name `ActorSystemSetup`'s `Display` prints for this type: the Java class name for Pekko's own setups, the
    /// Rust type name by default.
    fn class_name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }

    /// `a and b`: an [`ActorSystemSetup`] with both setups; `other` replaces `self` when both have the same type.
    fn and<S: Setup>(self, other: S) -> ActorSystemSetup
    where
        Self: Sized,
    {
        ActorSystemSetup::empty().with_setup(self).with_setup(other)
    }
}

#[derive(Clone)]
struct Entry {
    type_id: TypeId,
    class_name: &'static str,
    value: Arc<dyn Any + Send + Sync>,
}

/// `ActorSystemSetup`: the setups of an actor system, at most one per type, immutable.
#[derive(Clone, Default)]
pub struct ActorSystemSetup {
    entries: Vec<Entry>,
}

impl ActorSystemSetup {
    /// `ActorSystemSetup.empty`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `withSetup` (and `and`): a copy that also holds `setup`, replacing a setup of the same type.
    pub fn with_setup<T: Setup>(mut self, setup: T) -> Self {
        let entry = Entry {
            type_id: TypeId::of::<T>(),
            class_name: T::class_name(),
            value: Arc::new(setup),
        };
        match self.entries.iter_mut().find(|e| e.type_id == entry.type_id) {
            Some(existing) => *existing = entry,
            None => self.entries.push(entry),
        }
        self
    }

    /// `get[T]`: the setup of type `T`, if there is one.
    pub fn get<T: Setup>(&self) -> Option<&T> {
        self.entries
            .iter()
            .find(|e| e.type_id == TypeId::of::<T>())
            .and_then(|e| e.value.downcast_ref::<T>())
    }

    /// The number of setups.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there is no setup.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for ActorSystemSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.entries.iter().map(|e| e.class_name).collect();
        write!(f, "ActorSystemSettings({})", names.join(","))
    }
}

impl fmt::Debug for ActorSystemSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// `BootstrapSetup`: the core bootstrap settings of an actor system, immutable; every `with_*` returns a copy.
#[derive(Clone, Debug, Default)]
pub struct BootstrapSetup {
    config: Option<Config>,
    actor_ref_provider: Option<ProviderSelection>,
}

impl BootstrapSetup {
    /// `BootstrapSetup()`: the same as passing no `BootstrapSetup` at all.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BootstrapSetup(config)`.
    pub fn from_config(config: Config) -> Self {
        Self::default().with_config(config)
    }

    /// `withConfig`.
    pub fn with_config(self, config: Config) -> Self {
        Self {
            config: Some(config),
            ..self
        }
    }

    /// `withActorRefProvider`.
    pub fn with_actor_ref_provider(self, provider: ProviderSelection) -> Self {
        Self {
            actor_ref_provider: Some(provider),
            ..self
        }
    }

    /// `config`: the configuration to use instead of the default one, if any.
    pub fn config(&self) -> Option<&Config> {
        self.config.as_ref()
    }

    /// `actorRefProvider`: the provider that takes priority over `pekko.actor.provider`, if any.
    pub fn actor_ref_provider(&self) -> Option<&ProviderSelection> {
        self.actor_ref_provider.as_ref()
    }
}

impl Setup for BootstrapSetup {
    fn class_name() -> &'static str {
        "org.apache.pekko.actor.BootstrapSetup"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct DummySetup(String);
    #[derive(Debug, PartialEq, Eq)]
    struct DummySetup2(String);
    #[derive(Debug, PartialEq, Eq)]
    struct DummySetup3(String);

    impl Setup for DummySetup {
        fn class_name() -> &'static str {
            "DummySetup"
        }
    }
    impl Setup for DummySetup2 {
        fn class_name() -> &'static str {
            "DummySetup2"
        }
    }
    impl Setup for DummySetup3 {}

    fn dummy(name: &str) -> DummySetup {
        DummySetup(name.to_owned())
    }

    // ActorSystemSetupSpec.scala:31-37
    #[test]
    fn stores_and_retrieves_a_setup() {
        let setups = ActorSystemSetup::empty().with_setup(dummy("Al Dente"));
        assert_eq!(setups.get::<DummySetup>(), Some(&dummy("Al Dente")));
        assert_eq!(setups.get::<DummySetup2>(), None);
    }

    // ActorSystemSetupSpec.scala:39-45
    #[test]
    fn replaces_the_setup_of_the_same_type() {
        let setups = ActorSystemSetup::empty()
            .with_setup(dummy("Al Dente"))
            .with_setup(dummy("Earl E. Bird"));
        assert_eq!(setups.get::<DummySetup>(), Some(&dummy("Earl E. Bird")));
        assert_eq!(setups.len(), 1);
    }

    // ActorSystemSetupSpec.scala:47-55
    #[test]
    fn and_builds_a_setup_from_several_setups_and_the_last_of_a_type_wins() {
        let setups = dummy("Al Dente")
            .and(dummy("Earl E. Bird"))
            .with_setup(DummySetup2("Amanda Reckonwith".to_owned()));
        assert_eq!(setups.get::<DummySetup>(), Some(&dummy("Earl E. Bird")));
        assert_eq!(
            setups.get::<DummySetup2>(),
            Some(&DummySetup2("Amanda Reckonwith".to_owned()))
        );
    }

    // ActorSystemSetupSpec.scala:57-65
    #[test]
    fn is_created_with_a_set_of_setups() {
        let setups = ActorSystemSetup::empty()
            .with_setup(dummy("Manny Kin"))
            .with_setup(DummySetup2("Pepe Roni".to_owned()));
        assert!(setups.get::<DummySetup>().is_some());
        assert!(setups.get::<DummySetup2>().is_some());
        assert!(setups.get::<DummySetup3>().is_none());
    }

    #[test]
    fn with_setup_leaves_the_original_unchanged() {
        let original = ActorSystemSetup::empty();
        let changed = original.clone().with_setup(dummy("Al Dente"));
        assert!(original.is_empty());
        assert_eq!(changed.len(), 1);
    }

    // ActorSystemSetup.scala:94: the names in insertion order; a replaced setup keeps its position.
    #[test]
    fn display_lists_the_class_names_in_insertion_order() {
        assert_eq!(
            ActorSystemSetup::empty().to_string(),
            "ActorSystemSettings()"
        );
        let setups = ActorSystemSetup::empty()
            .with_setup(dummy("a"))
            .with_setup(DummySetup2("b".to_owned()))
            .with_setup(dummy("c"));
        assert_eq!(
            setups.to_string(),
            "ActorSystemSettings(DummySetup,DummySetup2)"
        );
        assert_eq!(format!("{setups:?}"), setups.to_string());
    }

    #[test]
    fn the_default_class_name_is_the_rust_type_name() {
        struct Anonymous;
        impl Setup for Anonymous {}
        let setups = ActorSystemSetup::empty().with_setup(Anonymous);
        assert!(setups.to_string().contains("Anonymous"));
    }

    // ActorSystem.scala:160-178
    #[test]
    fn bootstrap_setup_withers_return_copies_and_keep_the_other_fields() {
        let config = Config::parse_string("x = 1").unwrap();
        let original = BootstrapSetup::new();
        assert!(original.config().is_none());
        assert!(original.actor_ref_provider().is_none());
        let changed = original
            .clone()
            .with_config(config.clone())
            .with_actor_ref_provider(ProviderSelection::Remote);
        assert!(original.config().is_none());
        assert_eq!(
            changed.actor_ref_provider(),
            Some(&ProviderSelection::Remote)
        );
        assert_eq!(changed.config().map(|c| c.get_int("x").unwrap()), Some(1));
        let replaced = changed.with_config(Config::parse_string("x = 2").unwrap());
        assert_eq!(replaced.config().map(|c| c.get_int("x").unwrap()), Some(2));
        assert_eq!(
            replaced.actor_ref_provider(),
            Some(&ProviderSelection::Remote)
        );
        assert_eq!(
            BootstrapSetup::from_config(config)
                .config()
                .map(|c| c.get_int("x").unwrap()),
            Some(1)
        );
    }

    #[test]
    fn a_bootstrap_setup_is_found_in_an_actor_system_setup() {
        let bootstrap = BootstrapSetup::new().with_actor_ref_provider(ProviderSelection::Cluster);
        let setups = ActorSystemSetup::empty().with_setup(bootstrap);
        assert_eq!(
            setups
                .get::<BootstrapSetup>()
                .and_then(BootstrapSetup::actor_ref_provider),
            Some(&ProviderSelection::Cluster)
        );
        assert_eq!(
            setups.to_string(),
            "ActorSystemSettings(org.apache.pekko.actor.BootstrapSetup)"
        );
    }
}
