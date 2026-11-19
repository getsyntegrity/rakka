// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/DynamicAccess.scala and actor/src/main/scala/org/apache/pekko/actor/ReflectiveDynamicAccess.scala (Apache-2.0).
//! By-name factory lookup, the Rust counterpart of Pekko's `DynamicAccess` (AC-CFG-016, Q-X-2).
//!
//! Pekko creates mailboxes, dispatchers, loggers, extensions and the actor-ref provider from the class names found in the
//! configuration, through JVM reflection. Rust has no reflection, so a [`FactoryRegistry`] maps those names to factories
//! that the crate (or its user, Q-CFG-9 (c)) registers. What a caller can observe is kept: a name that is not registered
//! is a [`ClassNotFoundError`] that carries the name, exactly like `Class.forName` in `ReflectiveDynamicAccess.getClassFor`
//! (`ReflectiveDynamicAccess.scala:38`). The registry never wraps that error in a `ConfigurationException`: each consumer
//! transforms it the way Pekko 2.0.0 does (the provider loader propagates it, `ActorSystem.scala:996-1011`; Q-CFG-31).

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// The name is not registered: `java.lang.ClassNotFoundException`, whose message is the requested class name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassNotFoundError {
    name: String,
}

impl ClassNotFoundError {
    /// The name that was looked up.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The Java class of the exception Pekko throws for this failure.
    pub fn java_class(&self) -> &'static str {
        "java.lang.ClassNotFoundException"
    }

    /// `getMessage()`: `Class.forName` reports the requested name and nothing else.
    pub fn message(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ClassNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl Error for ClassNotFoundError {}

/// Why `FactoryRegistry::create` failed.
#[derive(Debug, PartialEq, Eq)]
pub enum CreateError<E> {
    /// No factory is registered under the name.
    ClassNotFound(ClassNotFoundError),
    /// The factory ran and failed; its error is returned as is, like the target exception of a failing constructor
    /// (`ReflectiveDynamicAccess.scala:54`, no `InvocationTargetException`).
    Construction(E),
}

impl<E: fmt::Display> fmt::Display for CreateError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreateError::ClassNotFound(error) => error.fmt(f),
            CreateError::Construction(error) => error.fmt(f),
        }
    }
}

impl<E: Error + 'static> Error for CreateError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CreateError::ClassNotFound(error) => Some(error),
            CreateError::Construction(error) => Some(error),
        }
    }
}

/// Factories by name. The factory type is chosen by the consumer (a mailbox factory is not a logger factory).
#[derive(Debug)]
pub struct FactoryRegistry<F> {
    factories: BTreeMap<String, F>,
}

impl<F> Default for FactoryRegistry<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> FactoryRegistry<F> {
    /// An empty registry.
    pub fn new() -> Self {
        Self {
            factories: BTreeMap::new(),
        }
    }

    /// Registers `factory` under `name`; a factory already registered under that name is replaced and returned.
    pub fn register(&mut self, name: impl Into<String>, factory: F) -> Option<F> {
        self.factories.insert(name.into(), factory)
    }

    /// `classIsOnClasspath`: whether a factory is registered under exactly `name`.
    pub fn contains(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }

    /// The factory registered under exactly `name` (no case folding, no trimming).
    pub fn lookup(&self, name: &str) -> Result<&F, ClassNotFoundError> {
        self.factories.get(name).ok_or_else(|| ClassNotFoundError {
            name: name.to_owned(),
        })
    }

    /// The registered names in ascending order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.factories.keys().map(String::as_str)
    }

    /// `createInstanceFor(name, args)`: looks the factory up and runs it with `arguments`.
    pub fn create<A, T, E>(&self, name: &str, arguments: A) -> Result<T, CreateError<E>>
    where
        F: Fn(A) -> Result<T, E>,
    {
        let factory = self.lookup(name).map_err(CreateError::ClassNotFound)?;
        factory(arguments).map_err(CreateError::Construction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in for the instances the registry creates (`TestClassWithDefaultConstructor` and friends).
    #[derive(Debug, PartialEq, Eq)]
    struct Named(String);

    /// The error of `TestClassWithThrowingConstructor` (`IllegalArgumentException("user-bug")`).
    #[derive(Debug, PartialEq, Eq)]
    struct UserBug;

    impl fmt::Display for UserBug {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("user-bug")
        }
    }

    impl Error for UserBug {}

    type Factory = Box<dyn Fn(Option<String>) -> Result<Named, UserBug>>;

    fn default_constructor() -> Factory {
        Box::new(|_| Ok(Named("default".to_owned())))
    }

    fn string_constructor() -> Factory {
        Box::new(|argument| Ok(Named(argument.unwrap_or_default())))
    }

    fn throwing_constructor() -> Factory {
        Box::new(|_| Err(UserBug))
    }

    fn registry() -> FactoryRegistry<Factory> {
        let mut registry = FactoryRegistry::new();
        registry.register(
            "org.apache.pekko.actor.TestClassWithDefaultConstructor",
            default_constructor(),
        );
        registry.register(
            "org.apache.pekko.actor.TestClassWithStringConstructor",
            string_constructor(),
        );
        registry.register(
            "org.apache.pekko.actor.TestClassWithThrowingConstructor",
            throwing_constructor(),
        );
        registry
    }

    // DynamicAccessSpec.scala:40-48
    #[test]
    fn creates_an_instance_from_a_registered_name() {
        let created = registry().create(
            "org.apache.pekko.actor.TestClassWithDefaultConstructor",
            None,
        );
        assert_eq!(created, Ok(Named("default".to_owned())));
    }

    // DynamicAccessSpec.scala:59-70: the arguments reach the factory.
    #[test]
    fn passes_the_arguments_to_the_factory() {
        let created = registry().create(
            "org.apache.pekko.actor.TestClassWithStringConstructor",
            Some("string ctor argument".to_owned()),
        );
        assert_eq!(created, Ok(Named("string ctor argument".to_owned())));
    }

    // DynamicAccessSpec.scala:50-57 and :77-86
    #[test]
    fn an_unregistered_name_is_a_class_not_found_error_carrying_the_name() {
        let error = registry().create("foo.NonExistingClass", None).err();
        let Some(CreateError::ClassNotFound(not_found)) = error else {
            panic!("expected ClassNotFound, got {error:?}");
        };
        assert_eq!(not_found.name(), "foo.NonExistingClass");
        assert_eq!(not_found.message(), "foo.NonExistingClass");
        assert_eq!(not_found.java_class(), "java.lang.ClassNotFoundException");
        assert_eq!(not_found.to_string(), "foo.NonExistingClass");
    }

    #[test]
    fn lookup_of_an_unregistered_name_is_the_same_error() {
        let registry = registry();
        let error = registry
            .lookup("org.apache.pekko.actor.foo.NonExistingClass")
            .err();
        assert_eq!(
            error.map(|e| e.message().to_owned()),
            Some("org.apache.pekko.actor.foo.NonExistingClass".to_owned())
        );
    }

    // Class.forName("") is a ClassNotFoundException with an empty message; the empty name is an ordinary name for the
    // registry: it is unknown until a factory is registered under it.
    #[test]
    fn the_empty_name_is_an_ordinary_name() {
        let mut registry = registry();
        let message = registry.lookup("").err().map(|e| e.message().to_owned());
        assert_eq!(message, Some(String::new()));
        assert!(!registry.contains(""));
        registry.register("", default_constructor());
        assert!(registry.contains(""));
        assert_eq!(registry.create("", None), Ok(Named("default".to_owned())));
    }

    // Class.forName is case sensitive and does not trim.
    #[test]
    fn names_match_exactly() {
        let registry = registry();
        assert!(registry.contains("org.apache.pekko.actor.TestClassWithDefaultConstructor"));
        assert!(!registry.contains("org.apache.pekko.actor.testclasswithdefaultconstructor"));
        assert!(!registry.contains(" org.apache.pekko.actor.TestClassWithDefaultConstructor"));
        assert!(!registry.contains("org.apache.pekko.actor.TestClassWithDefaultConstructor "));
    }

    // DynamicAccessSpec.scala:72-75
    #[test]
    fn contains_tells_whether_a_name_is_registered() {
        let registry = registry();
        assert!(!registry.contains("i.just.made.it.up.to.hurt.Myself"));
        assert!(registry.contains("org.apache.pekko.actor.TestClassWithStringConstructor"));
    }

    // DynamicAccessSpec.scala:102-109: the exception of a failing constructor is not wrapped.
    #[test]
    fn a_failing_factory_error_is_returned_unwrapped() {
        let error = registry()
            .create(
                "org.apache.pekko.actor.TestClassWithThrowingConstructor",
                None,
            )
            .err();
        assert_eq!(error, Some(CreateError::Construction(UserBug)));
        assert_eq!(error.map(|e| e.to_string()), Some("user-bug".to_owned()));
    }

    #[test]
    fn registering_a_name_again_replaces_the_factory_and_returns_the_previous_one() {
        let mut registry = FactoryRegistry::new();
        assert!(registry.register("a.B", default_constructor()).is_none());
        let previous = registry.register("a.B", string_constructor());
        assert!(previous.is_some());
        assert_eq!(
            registry.create("a.B", Some("replacement".to_owned())),
            Ok(Named("replacement".to_owned()))
        );
        assert_eq!(registry.names().count(), 1);
    }

    #[test]
    fn names_are_listed_in_ascending_order() {
        let mut registry = FactoryRegistry::new();
        registry.register("b.Second", default_constructor());
        registry.register("a.First", default_constructor());
        registry.register("c.Third", default_constructor());
        assert_eq!(
            registry.names().collect::<Vec<_>>(),
            ["a.First", "b.Second", "c.Third"]
        );
    }

    #[test]
    fn an_error_chain_reaches_the_class_not_found_error() {
        let error = registry().create("x.Y", None).err();
        let source = error
            .as_ref()
            .and_then(|e| Error::source(e))
            .map(ToString::to_string);
        assert_eq!(source, Some("x.Y".to_owned()));
    }

    #[test]
    fn a_class_not_found_error_displays_as_its_name_through_create_error() {
        let error = registry().create("x.Y", None).err();
        assert_eq!(error.map(|e| e.to_string()), Some("x.Y".to_owned()));
    }

    #[test]
    fn a_construction_error_is_the_source_of_the_create_error() {
        let error = registry()
            .create(
                "org.apache.pekko.actor.TestClassWithThrowingConstructor",
                None,
            )
            .err();
        let source = error
            .as_ref()
            .and_then(|e| Error::source(e))
            .map(ToString::to_string);
        assert_eq!(source, Some("user-bug".to_owned()));
    }

    #[test]
    fn the_default_registry_is_empty() {
        let registry = FactoryRegistry::<Factory>::default();
        assert_eq!(registry.names().count(), 0);
        assert!(!registry.contains("org.apache.pekko.actor.Actor"));
    }
}
