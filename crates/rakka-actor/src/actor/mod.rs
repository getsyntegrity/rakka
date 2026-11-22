//! Classic actors (`org.apache.pekko.actor`): paths and addresses, then the runtime (system, cells with
//! mailbox and lifecycle, children registry, `Props`/context, supervision).

pub(crate) mod actor_ref;
mod address;
pub(crate) mod cell;
pub(crate) mod children;
mod deploy;
mod deployer;
mod errors;
pub mod fault;
pub(crate) mod jstring;
mod path;
pub mod props;
pub(crate) mod repointable;
pub mod supervision;
pub mod system;
pub(crate) mod uri;

pub use actor_ref::ActorRef;
pub use address::Address;
pub use deploy::{Deploy, NO_DISPATCHER_GIVEN, NO_MAILBOX_GIVEN, RouterConfig, Scope};
pub use deployer::{Deployer, DeployerError};
pub use errors::{
    ActorError, ChildError, ErrorClass, FromStringError, IllegalArgumentError, IllegalStateError,
    InvalidActorNameError, MalformedUrlError, NumberFormatError, UnsupportedOperationError,
};
pub use path::{
    ActorPath, is_valid_path_element, relative_actor_path_elements, validate_path_element,
};
