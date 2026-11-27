//! Classic actors (`org.apache.pekko.actor`): paths and addresses, then the runtime (system, cells with
//! mailbox and lifecycle, children registry, `Props`/context, supervision).

pub(crate) mod actor_ref;
mod address;
#[cfg(test)]
mod async_creation_tests;
pub(crate) mod cell;
pub(crate) mod children;
mod deploy;
mod deployer;
mod errors;
pub mod fault;
#[cfg(test)]
pub(crate) mod guardian_failure_tests;
#[cfg(test)]
mod guardian_termination_tests;
#[cfg(test)]
mod init_failure_tests;
pub(crate) mod jstring;
#[cfg(test)]
mod nonblocking_mailbox_tests;
mod path;
pub mod props;
#[cfg(test)]
mod refailure_tests;
pub(crate) mod repointable;
pub mod scheduler;
#[cfg(test)]
mod scheduler_tests;
pub mod selection;
pub mod supervision;
pub mod system;
mod timer_wheel;
#[cfg(test)]
mod unstarted_push_timeout_tests;
#[cfg(test)]
mod unstarted_termination_tests;
pub(crate) mod uri;
pub(crate) mod watching;

pub use actor_ref::ActorRef;
pub use address::Address;
pub use deploy::{Deploy, NO_DISPATCHER_GIVEN, NO_MAILBOX_GIVEN, RouterConfig, Scope};
pub use deployer::{Deployer, DeployerError};
pub use errors::{
    ActorError, ChildError, ConfigurationError, ErrorClass, FromStringError, IllegalArgumentError,
    IllegalStateError, InvalidActorNameError, MalformedUrlError, NumberFormatError,
    UnsupportedOperationError,
};
pub use path::{
    ActorPath, is_valid_path_element, relative_actor_path_elements, validate_path_element,
};
