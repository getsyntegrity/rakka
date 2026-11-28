// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala (Apache-2.0).
//! The termination message of a `watchWith` (the `Some(msg)` of `watching: Map[ActorRef, Option[Any]]`).

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::actor::ActorRef;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::AnyMessage;

/// The message of a `watch_with`, with the equality of its concrete type (Q-DW-3 (a)).
#[derive(Clone)]
pub(crate) struct CustomTermination {
    pub(crate) message: AnyMessage,
    same_as: fn(&AnyMessage, &AnyMessage) -> bool,
    type_name: &'static str,
}

impl CustomTermination {
    pub(crate) fn new<M: Any + PartialEq + Send + Sync>(message: M) -> Self {
        CustomTermination {
            message: Arc::new(message),
            same_as: |previous, new| previous.downcast_ref::<M>() == new.downcast_ref::<M>(),
            type_name: std::any::type_name::<M>(),
        }
    }

    /// `Some(message)` in the exception text. Only strings are shown by value; any other type by its name (Q-DW-4).
    fn rendered(&self) -> String {
        let message = &self.message;
        let text = message
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| message.downcast_ref::<&str>().copied());
        format!("Some({})", text.unwrap_or(self.type_name))
    }
}

/// A panic of the user's `Drop` stops here (the panic hook has reported it): it never unwinds out of a death-watch transition.
impl Drop for CustomTermination {
    fn drop(&mut self) {
        let message = std::mem::replace(&mut self.message, Arc::new(()));
        let _ = catch_unwind(AssertUnwindSafe(|| drop(message)));
    }
}

/// `previous == newMessage`: both `watch`, or two `watch_with` with equal messages.
fn same_registration(
    previous: Option<&CustomTermination>,
    new: Option<&CustomTermination>,
) -> bool {
    match (previous, new) {
        (Some(previous), Some(new)) => (previous.same_as)(&previous.message, &new.message),
        (previous, new) => previous.is_none() && new.is_none(),
    }
}

/// `checkWatchingSame`: throws `IllegalStateException` unless `new` is the registration already made (both `watch`, or equal messages).
pub(crate) fn check_watching_same(
    watcher: &ActorRef,
    subject: &ActorRef,
    previous: Option<&CustomTermination>,
    new: Option<&CustomTermination>,
) {
    if same_registration(previous, new) {
        return;
    }
    let option = |c: Option<&CustomTermination>| c.map_or("None".to_owned(), |c| c.rendered());
    throw(Throwable::illegal_state(&format!(
        "Watch({watcher:?}, {subject:?}) termination message was not overwritten from [{}] to [{}]. \
         If this was intended, unwatch first before using `watch` / `watchWith` with another message.",
        option(previous),
        option(new),
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same<A, B>(previous: A, new: B) -> bool
    where
        A: Any + PartialEq + Send + Sync,
        B: Any + PartialEq + Send + Sync,
    {
        let (previous, new) = (
            CustomTermination::new(previous),
            CustomTermination::new(new),
        );
        same_registration(Some(&previous), Some(&new))
    }

    /// Scala accepts `1 == 1L` and has one string type: Rust compares concrete type and value (Q-DW-3 (a)).
    #[test]
    fn messages_are_equal_only_with_the_same_rust_type_and_an_equal_value() {
        assert!(same(1_i32, 1_i32));
        assert!(same(vec!["a".to_owned()], vec!["a".to_owned()]));
        assert!(!same(1_i32, 2_i32));
        assert!(!same(1_i32, 1_i64));
        assert!(!same("a", "a".to_owned()));
        assert!(!same(f64::NAN, f64::NAN));
    }

    #[test]
    fn a_watch_is_the_same_only_as_another_watch() {
        let custom = CustomTermination::new("a");
        assert!(same_registration(None, None));
        assert!(!same_registration(None, Some(&custom)));
        assert!(!same_registration(Some(&custom), None));
    }

    #[test]
    fn only_strings_are_rendered_by_value() {
        assert_eq!(CustomTermination::new("a").rendered(), "Some(a)");
        assert_eq!(CustomTermination::new("b".to_owned()).rendered(), "Some(b)");
        assert_eq!(CustomTermination::new(7_i32).rendered(), "Some(i32)");
    }
}
