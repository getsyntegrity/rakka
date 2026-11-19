// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! `ProviderSelection`: which actor-ref provider `pekko.actor.provider` names (AC-CFG-008).
//!
//! `local`, `remote` and `cluster` are the three short names; `remote` and `cluster` also have the class name Pekko used
//! before the short names existed. Any other string, whatever it looks like, is a custom provider class name
//! (`ActorSystem.scala:137-144`): no case folding and no trimming. Whether a name can be loaded is not decided here: a
//! provider that is not registered is the error of the factory registry (`FactoryRegistry`, Q-CFG-31).

/// The class of Pekko's local provider (`classOf[LocalActorRefProvider].getName`).
pub const LOCAL_ACTOR_REF_PROVIDER: &str = "org.apache.pekko.actor.LocalActorRefProvider";
/// The class of the remote provider (a module outside pekko-actor).
pub const REMOTE_ACTOR_REF_PROVIDER: &str = "org.apache.pekko.remote.RemoteActorRefProvider";
/// The class of the cluster provider (a module outside pekko-actor).
pub const CLUSTER_ACTOR_REF_PROVIDER: &str = "org.apache.pekko.cluster.ClusterActorRefProvider";

/// `ProviderSelection`: `Local`, `Remote`, `Cluster` or `Custom(class name)`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProviderSelection {
    /// `pekko.actor.provider = local`.
    Local,
    /// `remote` or the remote provider's class name.
    Remote,
    /// `cluster` or the cluster provider's class name.
    Cluster,
    /// Any other string, taken as the class name of the provider.
    Custom(String),
}

impl ProviderSelection {
    /// `ProviderSelection(providerClass)` (`ActorSystem.scala:137-144`).
    pub fn from_identifier(provider: &str) -> Self {
        match provider {
            "local" => ProviderSelection::Local,
            "remote" | REMOTE_ACTOR_REF_PROVIDER => ProviderSelection::Remote,
            "cluster" | CLUSTER_ACTOR_REF_PROVIDER => ProviderSelection::Cluster,
            class_name => ProviderSelection::Custom(class_name.to_owned()),
        }
    }

    /// The `identifier`: `local`, `remote`, `cluster` or `custom`.
    pub fn identifier(&self) -> &'static str {
        match self {
            ProviderSelection::Local => "local",
            ProviderSelection::Remote => "remote",
            ProviderSelection::Cluster => "cluster",
            ProviderSelection::Custom(_) => "custom",
        }
    }

    /// The `fqcn`: the class name of the selected provider.
    pub fn fqcn(&self) -> &str {
        match self {
            ProviderSelection::Local => LOCAL_ACTOR_REF_PROVIDER,
            ProviderSelection::Remote => REMOTE_ACTOR_REF_PROVIDER,
            ProviderSelection::Cluster => CLUSTER_ACTOR_REF_PROVIDER,
            ProviderSelection::Custom(class_name) => class_name,
        }
    }

    /// `hasCluster`: true only for `Cluster`.
    pub fn has_cluster(&self) -> bool {
        matches!(self, ProviderSelection::Cluster)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ProviderSelectionSpec.scala:38-43
    #[test]
    fn local_selects_the_local_provider_class_and_has_no_cluster() {
        let selection = ProviderSelection::from_identifier("local");
        assert_eq!(selection, ProviderSelection::Local);
        assert_eq!(selection.identifier(), "local");
        assert_eq!(
            selection.fqcn(),
            "org.apache.pekko.actor.LocalActorRefProvider"
        );
        assert!(!selection.has_cluster());
    }

    // ProviderSelectionSpec.scala:45-51
    #[test]
    fn remote_by_short_name_and_by_class_name_is_the_same_selection() {
        let selection = ProviderSelection::from_identifier("remote");
        assert_eq!(selection.identifier(), "remote");
        assert_eq!(selection.fqcn(), REMOTE_ACTOR_REF_PROVIDER);
        assert!(!selection.has_cluster());
        assert_eq!(
            selection,
            ProviderSelection::from_identifier(REMOTE_ACTOR_REF_PROVIDER)
        );
    }

    // ProviderSelectionSpec.scala:53-59
    #[test]
    fn cluster_by_short_name_and_by_class_name_is_the_same_selection_and_has_a_cluster() {
        let selection = ProviderSelection::from_identifier("cluster");
        assert_eq!(selection.identifier(), "cluster");
        assert_eq!(selection.fqcn(), CLUSTER_ACTOR_REF_PROVIDER);
        assert!(selection.has_cluster());
        assert_eq!(
            selection,
            ProviderSelection::from_identifier(CLUSTER_ACTOR_REF_PROVIDER)
        );
    }

    // ProviderSelectionSpec.scala:61-67
    #[test]
    fn any_other_string_is_a_custom_provider_with_that_class_name() {
        let selection = ProviderSelection::from_identifier("other.ActorRefProvider");
        assert_eq!(
            selection,
            ProviderSelection::Custom("other.ActorRefProvider".to_owned())
        );
        assert_eq!(selection.identifier(), "custom");
        assert_eq!(selection.fqcn(), "other.ActorRefProvider");
        assert!(!selection.has_cluster());
    }

    // ActorSystem.scala:137-144: the local class name is not special.
    #[test]
    fn the_local_class_name_is_a_custom_provider() {
        let selection = ProviderSelection::from_identifier(LOCAL_ACTOR_REF_PROVIDER);
        assert_eq!(
            selection,
            ProviderSelection::Custom(LOCAL_ACTOR_REF_PROVIDER.to_owned())
        );
        assert_eq!(selection.identifier(), "custom");
        assert_eq!(selection.fqcn(), LOCAL_ACTOR_REF_PROVIDER);
        assert_ne!(selection, ProviderSelection::Local);
        assert_ne!(selection, ProviderSelection::from_identifier("local"));
    }

    // ActorSystem.scala:137-144: exact match, no case folding and no trimming.
    #[test]
    fn short_names_match_exactly() {
        for text in ["LOCAL", "Remote", "CLUSTER", " local", "remote ", "", "5"] {
            let selection = ProviderSelection::from_identifier(text);
            assert_eq!(
                selection,
                ProviderSelection::Custom(text.to_owned()),
                "{text:?}"
            );
            assert_eq!(selection.fqcn(), text);
            assert!(!selection.has_cluster());
        }
    }

    #[test]
    fn two_custom_selections_are_equal_only_with_the_same_class_name() {
        assert_eq!(
            ProviderSelection::from_identifier("other.A"),
            ProviderSelection::from_identifier("other.A")
        );
        assert_ne!(
            ProviderSelection::from_identifier("other.A"),
            ProviderSelection::from_identifier("other.B")
        );
        assert_ne!(ProviderSelection::Remote, ProviderSelection::Cluster);
    }
}
