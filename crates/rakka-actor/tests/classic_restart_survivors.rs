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

/// A node of a supervision tree whose `preRestart` keeps the children unless told otherwise. It restarts whatever child fails, records its restart hooks
/// and the text messages it receives, and creates the child a `Props` message asks for.
#[derive(Clone)]
struct Keeper {
    name: &'static str,
    log: Log,
    children: Arc<Mutex<Vec<ActorRef>>>,
    spawn_in_pre_start: bool,
    stops_children: bool,
    fail_first_post_restart: Arc<AtomicBool>,
    /// what this node answers to a child that failed in `postRestart` (any other failure: restart)
    on_post_restart_failure: Directive,
    /// the message "block" waits until the test lets go of this lock
    gate: Arc<Mutex<()>>,
}

impl Keeper {
    fn new(name: &'static str, log: &Log) -> Keeper {
        Keeper {
            name,
            log: log.clone(),
            children: Arc::default(),
            spawn_in_pre_start: false,
            stops_children: false,
            fail_first_post_restart: Arc::default(),
            on_post_restart_failure: Directive::Restart,
            gate: Arc::default(),
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
        let answer = self.on_post_restart_failure;
        OneForOneStrategy::new(3, move |cause: &Throwable| {
            let fatal = cause.to_string().contains("PostRestartException");
            if fatal { answer } else { Directive::Restart }
        })
        .into()
    }
    fn pre_restart(&mut self, ctx: &mut ActorContext, cause: &Throwable, _: Option<&AnyMessage>) {
        if self.stops_children {
            ctx.children().iter().for_each(|child| ctx.stop(child));
        }
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
            if *text == "block" {
                drop(self.gate.lock().unwrap());
            }
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

/// `Suspend` and `Recreate` are `StashWhenWaitingForChildren`: a survivor halfway through its own restart takes them afterwards.
#[test]
fn a_survivor_waiting_for_its_children_to_die_restarts_for_its_parent_afterwards() {
    let system = RunningSystem::create("t-survivor-waiting");
    let log = Log::default();
    let (top, parent, mut child) = (
        Keeper::new("top", &log),
        Keeper::new("parent", &log),
        Keeper::new("child", &log),
    );
    child.stops_children = true;
    let slow = Keeper::new("slow", &log);
    let top_ref = system.actor_of(top.props()).unwrap();
    let parent_ref = child_of(&top_ref, &top, &parent);
    let child_ref = child_of(&parent_ref, &parent, &child);
    let slow_ref = child_of(&child_ref, &child, &slow);
    let seen = |entry: &str| snapshot_of(&log).iter().any(|e| e.starts_with(entry));

    let held = slow.gate.lock().unwrap();
    slow_ref.tell("block", None);
    wait_until("slow child busy", || seen("slow:block"));
    child_ref.tell("boom", None);
    wait_until("child restarting", || seen("child:pre-restart"));
    parent_ref.tell(Kill, None);
    parent_ref.tell("done", None);
    wait_until("parent restarted", || seen("parent:done"));
    drop(held);
    child_ref.tell("after", None);
    wait_until("child works", || seen("child:after"));

    let mut entries = snapshot_of(&log);
    entries.retain(|e| e.starts_with("child:"));
    let killed = "org.apache.pekko.actor.ActorKilledException: Kill";
    assert_eq!(
        entries,
        owned_strings(&[
            &format!("child:pre-restart {BOOM}"),
            "child:post-restart",
            &format!("child:pre-restart {killed}"),
            "child:post-restart",
            "child:after",
        ])
    );
}

/// The parent fails its first `postRestart`: the next restart skips `preRestart` (`setFailedFatally`) and the survivor, which
/// `handleInvokeFailure(survivors, ..)` did not suspend again, restarts once and works.
fn restart_again_after_a_failed_post_restart(system_name: &str, top_answers: Directive) {
    let system = RunningSystem::create(system_name);
    let log = Log::default();
    let (mut top, parent, child) = (
        Keeper::new("top", &log),
        Keeper::new("parent", &log),
        Keeper::new("child", &log),
    );
    top.on_post_restart_failure = top_answers;
    parent.fail_first_post_restart.store(true, Ordering::SeqCst);
    let top_ref = system.actor_of(top.props()).unwrap();
    let parent_ref = child_of(&top_ref, &top, &parent);
    let child_ref = child_of(&parent_ref, &parent, &child);
    let entries_of = |who: &str| {
        let mut entries = snapshot_of(&log);
        entries.retain(|e| e.starts_with(who));
        entries
    };

    parent_ref.tell("boom", None);
    parent_ref.tell("alive", None);
    wait_until("parent works again", || entries_of("parent:").len() >= 4);
    child_ref.tell("after", None);
    wait_until("child works", || entries_of("child:").len() >= 3);

    assert_eq!(
        entries_of("parent:"),
        owned_strings(&[
            &format!("parent:pre-restart {BOOM}"),
            "parent:post-restart",
            "parent:post-restart",
            "parent:alive",
        ])
    );
    assert_eq!(
        entries_of("child:")[1..],
        ["child:post-restart", "child:after"]
    );
}

#[test]
fn a_restart_after_a_failed_post_restart_skips_pre_restart_and_keeps_the_survivor_working() {
    restart_again_after_a_failed_post_restart("t-fatal-restart", Directive::Restart);
}

#[test]
fn resuming_an_actor_that_failed_fatally_restarts_it() {
    restart_again_after_a_failed_post_restart("t-fatal-resume", Directive::Resume);
}
