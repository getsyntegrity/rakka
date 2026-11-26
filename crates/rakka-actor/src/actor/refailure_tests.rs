// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! `handleInvokeFailure` has no `isFailed` guard (AC-SUP-001). The public API cannot get there (a failed cell is suspended), so the
//! tests send it the `Resume(null)` of a recursive resume; both traces were observed on Pekko 2.0.0 with a throwaway JVM probe.

use crate::actor::actor_ref::ActorRef;
use crate::actor::async_creation_tests::wait_until;
use crate::actor::cell::FailedInfo;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::ActorSystem;
use crate::dispatch::sysmsg::SystemMessage;
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Clone, Default)]
struct Records {
    born: Arc<Mutex<Vec<(&'static str, ActorRef)>>>,
    decided: Arc<Mutex<Vec<String>>>,
    gate: Arc<Mutex<()>>,
    restarted_with: Arc<Mutex<Vec<&'static str>>>,
}

/// Creates its children and throws on any text. Unless it escalates, its decider records the cause, waits for the gate and resumes.
#[derive(Clone)]
struct Node(&'static str, Directive, Vec<Node>, Records);

impl Actor for Node {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.3.born.lock().unwrap().push((self.0, ctx.self_ref()));
        for child in self.2.clone() {
            ctx.actor_of(Props::new(move || child.clone())).unwrap();
        }
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (records, answer) = (self.3.clone(), self.1);
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            if answer != Directive::Escalate {
                records.decided.lock().unwrap().push(cause.to_string());
                drop(records.gate.lock().unwrap_or_else(PoisonError::into_inner));
            }
            answer
        })
        .into()
    }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, msg: Option<&AnyMessage>) {
        let text = msg.and_then(|msg| msg.downcast_ref::<&str>().copied());
        self.3.restarted_with.lock().unwrap().extend(text);
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = msg.downcast_ref::<&str>().unwrap();
        throw(Throwable::illegal_state(text));
    }
}

/// Starts `root` and returns the references of `names`, once all of them exist.
fn start<const N: usize>(root: Node, names: [&str; N]) -> (ActorSystem, [ActorRef; N]) {
    let records = root.3.clone();
    let system = ActorSystem::create("t-refailure").unwrap();
    system.actor_of(Props::new(move || root.clone())).unwrap();
    let find = |name: &str| {
        let born = records.born.lock().unwrap();
        born.iter().find(|b| b.0 == name).map(|b| b.1.clone())
    };
    wait_until("tree", || names.iter().all(|name| find(name).is_some()));
    (system, names.map(|name| find(name).unwrap()))
}

/// Waits until the mailboxes of `actors` have those suspend counts.
fn wait_suspended(actors: &[(&ActorRef, u8)]) {
    wait_until("suspend counts", || {
        actors.iter().all(|(actor, times)| {
            let count = format!("suspendCount={times} ");
            actor.mailbox_view().contains(&count)
        })
    });
}

#[test]
fn a_failed_actor_that_fails_again_is_suspended_with_its_children_and_reported_again() {
    let records = Records::default();
    let node = |name, children| Node(name, Directive::Resume, children, records.clone());
    let root = node("top", vec![node("child", vec![node("leaf", vec![])])]);
    let (_system, [child, leaf]) = start(root, ["child", "leaf"]);
    let deciding = records.gate.lock().unwrap();
    child.tell("first", None);
    wait_suspended(&[(&child, 1), (&leaf, 1)]);
    child.0.send_system(SystemMessage::Resume { cause: None });
    wait_suspended(&[(&child, 0), (&leaf, 0)]);
    assert!(child.is_failed());
    child.tell("second", None);
    wait_suspended(&[(&child, 1), (&leaf, 1)]);
    let cause = Throwable::illegal_state("recreate");
    child.0.send_system(SystemMessage::Recreate { cause });
    let restarted = || records.restarted_with.lock().unwrap().clone();
    wait_until("restart", || !restarted().is_empty());
    assert_eq!(restarted(), ["second"]);
    drop(deciding);
    wait_until("both reports", || {
        records.decided.lock().unwrap().len() == 2
    });
    assert_eq!(
        *records.decided.lock().unwrap(),
        ["first", "second"].map(|m| format!("java.lang.IllegalStateException: {m}"))
    );
}

#[test]
fn escalating_while_already_failed_suspends_that_child_too_and_keeps_the_first_perpetrator() {
    let records = Records::default();
    let node = |name, answer, children| Node(name, answer, children, records.clone());
    let leaves = ["a", "b"].map(|name| node(name, Directive::Resume, vec![]));
    let parent = node("parent", Directive::Escalate, leaves.to_vec());
    let root = node("top", Directive::Resume, vec![parent]);
    let (_system, [parent, a, b]) = start(root, ["parent", "a", "b"]);
    let deciding = records.gate.lock().unwrap();
    a.tell("a", None);
    wait_suspended(&[(&parent, 1), (&a, 1), (&b, 1)]);
    parent.0.send_system(SystemMessage::Resume { cause: None });
    wait_suspended(&[(&parent, 0), (&a, 0), (&b, 0)]);
    b.tell("b", None);
    wait_suspended(&[(&parent, 1), (&a, 1), (&b, 2)]);
    drop(deciding);
    // two `Resume(cause)` from the top: only the first perpetrator, `a`, gets the cause and forgets its failure
    wait_until("both reports", || {
        records.decided.lock().unwrap().len() == 2
    });
    wait_suspended(&[(&parent, 0), (&a, 0), (&b, 0)]);
    wait_until("a forgot", || !a.is_failed() && !parent.is_failed());
    assert!(b.is_failed());
}

#[test]
fn failed_fatally_is_neither_overwritten_nor_cleared_and_does_not_count_as_failed() {
    let mut failed = FailedInfo::FailedFatally;
    failed.set_failed(None);
    failed.clear_failed();
    assert!(matches!(failed, FailedInfo::FailedFatally));
    assert!(!failed.is_failed());
}
