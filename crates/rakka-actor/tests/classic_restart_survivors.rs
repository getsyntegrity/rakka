// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala and actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorTreeSpec.scala (Apache-2.0).
//! A restarted parent restarts the children its `preRestart` did not stop (`finishRecreate`, AC-SUP-015).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, owned_strings, settle_for_absence, snapshot_of, wait_until};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};

type Log = Arc<Mutex<Vec<String>>>;

/// A node of a supervision tree whose `preRestart` keeps the children. It restarts whatever child fails, records its restart hooks
/// and the text messages it receives, and creates the child a `Props` message asks for.
#[derive(Clone)]
struct Keeper {
    name: &'static str,
    log: Log,
    children: Arc<Mutex<Vec<ActorRef>>>,
    spawn_in_pre_start: bool,
    fail_first_post_restart: Arc<AtomicBool>,
}

impl Keeper {
    fn new(name: &'static str, log: &Log) -> Keeper {
        Keeper {
            name,
            log: log.clone(),
            children: Arc::default(),
            spawn_in_pre_start: false,
            fail_first_post_restart: Arc::default(),
        }
    }

    fn props(&self) -> Props {
        let node = self.clone();
        Props::new(move || node.clone())
    }

    fn record(&self, what: impl std::fmt::Display) {
        self.log
            .lock()
            .unwrap()
            .push(format!("{}:{what}", self.name));
    }
}

impl Actor for Keeper {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        if self.spawn_in_pre_start {
            let late = Keeper::new("late", &self.log);
            ctx.actor_of(late.props()).unwrap();
        }
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        OneForOneStrategy::new(3, |_: &Throwable| Directive::Restart).into()
    }
    fn pre_restart(&mut self, _: &mut ActorContext, cause: &Throwable, _: Option<&AnyMessage>) {
        self.record(format_args!("pre-restart {cause}"));
    }
    fn post_restart(&mut self, ctx: &mut ActorContext, _: &Throwable) {
        self.record("post-restart");
        if self.fail_first_post_restart.swap(false, Ordering::SeqCst) {
            throw(Throwable::illegal_state("post-restart"));
        }
        self.pre_start(ctx);
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(child) = msg.downcast_ref::<Keeper>() {
            let spawned = ctx.actor_of(child.props()).unwrap();
            self.children.lock().unwrap().push(spawned);
        } else if let Some(text) = msg.downcast_ref::<&str>() {
            if *text == "boom" {
                throw(Throwable::illegal_state("boom"));
            }
            self.record(text);
        }
        true
    }
}

/// Has `parent` create `child` and returns the new reference.
fn child_of(parent: &ActorRef, owner: &Keeper, child: &Keeper) -> ActorRef {
    let before = owner.children.lock().unwrap().len();
    parent.tell(child.clone(), None);
    wait_until("child created", || {
        owner.children.lock().unwrap().len() > before
    });
    owner.children.lock().unwrap()[before].clone()
}

const BOOM: &str = "java.lang.IllegalStateException: boom";

#[test]
fn survivors_are_restarted_after_the_parent_with_its_cause_and_keep_working() {
    let system = RunningSystem::create("t-survivors");
    let log = Log::default();
    let mut parent = Keeper::new("parent", &log);
    parent.spawn_in_pre_start = true;
    let parent_ref = system.actor_of(parent.props()).unwrap();
    let child = Keeper::new("child", &log);
    let child_ref = child_of(&parent_ref, &parent, &child);

    parent_ref.tell("boom", None);
    wait_until("child restarted", || snapshot_of(&log).len() >= 4);
    child_ref.tell("after", None);
    wait_until("child works", || snapshot_of(&log).len() >= 5);
    settle_for_absence();

    let entries = snapshot_of(&log);
    // the child the first `preStart` made survives and restarts too; the one the new instance makes does not
    let late: Vec<_> = entries.iter().filter(|e| e.starts_with("late:")).collect();
    assert_eq!(late.len(), 2, "{entries:?}");
    let rest: Vec<String> = entries
        .into_iter()
        .filter(|e| !e.starts_with("late:"))
        .collect();
    assert_eq!(
        rest,
        owned_strings(&[
            &format!("parent:pre-restart {BOOM}"),
            "parent:post-restart",
            &format!("child:pre-restart {BOOM}"),
            "child:post-restart",
            "child:after",
        ])
    );
}

/// `SupervisorTreeSpec`: "be able to kill the middle actor and see itself and its child restarted".
#[test]
fn killing_the_middle_of_three_levels_restarts_it_and_then_its_child() {
    let system = RunningSystem::create("t-tree");
    let log = Log::default();
    let (head, middle, last) = (
        Keeper::new("head", &log),
        Keeper::new("middle", &log),
        Keeper::new("last", &log),
    );
    let head_ref = system.actor_of(head.props()).unwrap();
    let middle_ref = child_of(&head_ref, &head, &middle);
    child_of(&middle_ref, &middle, &last);

    middle_ref.tell(Kill, None);
    wait_until("both restarted", || snapshot_of(&log).len() >= 4);
    settle_for_absence();

    let killed = "org.apache.pekko.actor.ActorKilledException: Kill";
    assert_eq!(
        snapshot_of(&log),
        owned_strings(&[
            &format!("middle:pre-restart {killed}"),
            "middle:post-restart",
            &format!("last:pre-restart {killed}"),
            "last:post-restart",
        ])
    );
}

/// `handleInvokeFailure(survivors, ..)`: a failing `postRestart` does not suspend the survivors a second time.
#[test]
fn a_survivor_is_not_left_suspended_when_the_first_post_restart_fails() {
    let system = RunningSystem::create("t-survivors-twice");
    let log = Log::default();
    let (top, parent, child) = (
        Keeper::new("top", &log),
        Keeper::new("parent", &log),
        Keeper::new("child", &log),
    );
    parent.fail_first_post_restart.store(true, Ordering::SeqCst);
    let top_ref = system.actor_of(top.props()).unwrap();
    let parent_ref = child_of(&top_ref, &top, &parent);
    let child_ref = child_of(&parent_ref, &parent, &child);

    parent_ref.tell("boom", None);
    wait_until("second restart of the parent", || {
        let restarts = snapshot_of(&log);
        restarts
            .iter()
            .filter(|e| *e == "parent:post-restart")
            .count()
            == 2
    });
    child_ref.tell("after", None);
    wait_until("child works", || {
        snapshot_of(&log).contains(&"child:after".to_string())
    });
}
