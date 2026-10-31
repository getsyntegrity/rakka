//! Classic actor types (`org.apache.pekko.actor`).

mod address;
mod errors;
pub(crate) mod jstring;

pub use address::Address;
pub use errors::{
    ArithmeticError, ChildError, FromStringError, IllegalArgumentError, IllegalStateError,
    InvalidActorNameError, MalformedUrlError, NumberFormatError,
};
mod path;
pub(crate) mod uri;

pub use path::{
    ActorPath, is_valid_path_element, relative_actor_path_elements, validate_path_element,
};
