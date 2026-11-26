// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala and actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala (Apache-2.0).
//! A suspended supervisor defers the `Failed` of its child until it is resumed or restarted, and drops it when it stops
//! (`StashWhenFailed`, AC-CELL-013.2).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, owned_strings, settle_for_absence, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};

type Log = Arc<Mutex<Vec<String>>>;

/// Throws on the message "fail", keeps its children across a restart and answers every failed child with `answer`, once the test
/// lets go of `deciding`.
#[derive(Clone)]
struct Node {
    name: &'static str,
    log: Log,
    answer: Directive,
    deciding: Arc<Mutex<()>>,
    children: Arc<Mutex<Vec<ActorRef>>>,
}

impl Node {
    fn new(name: &'static str, log: &Log, answer: Directive) -> Node {
        Node {
            name,
            log: log.clone(),
            answer,
            deciding: Arc::default(),
            children: Arc::default(),
        }
    }

    fn record(&self, what: &str) {
        self.log
            .lock()
            .unwrap()
            .push(format!("{}:{what}", self.name));
    }
}

impl Actor for Node {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let node = self.clone();
        OneForOneStrategy::new(3, move |_: &Throwable| {
            node.record("decides");
            drop(node.deciding.lock().unwrap());
            node.answer
        })
        .into()
    }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {}
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.record("post-restart");
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        self.record("post-stop");
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(child) = msg.downcast_ref::<Node>() {
            let node = child.clone();
            let spawned = ctx.actor_of(Props::new(move || node.clone())).unwrap();
            self.children.lock().unwrap().push(spawned);
        } else if msg.downcast_ref::<&str>() == Some(&"fail") {
            throw(Throwable::illegal_state("fail"));
        }
        true
    }
}

/// Builds the chain, one child under the other, below an actor of `chain[0]`.
fn spawn_chain(system: &RunningSystem, chain: &[&Node]) -> Vec<ActorRef> {
    let top = chain[0].clone();
    let mut refs = vec![system.actor_of(Props::new(move || top.clone())).unwrap()];
    for pair in chain.windows(2) {
        refs.last().unwrap().tell(pair[1].clone(), None);
        wait_until("child created", || {
            !pair[0].children.lock().unwrap().is_empty()
        });
        let child = pair[0].children.lock().unwrap()[0].clone();
        refs.push(child);
    }
    refs
}

/// `parent` > `child` > `grandchild` hang from the actor under `above` that fails. The grandchild fails first and `child` is held
/// deciding about it; only once the failing actor's supervisor decides (so `parent` is suspended) does `child` escalate.
fn escalate_to_a_suspended_parent(
    system_name: &str,
    above: &[&Node],
    parent: Node,
    failing_depth: usize,
    expected_before: &[&str],
    expected_of_parent: &[&str],
) {
    let system = RunningSystem::create(system_name);
    let log = parent.log.clone();
    let child = Node::new("child", &log, Directive::Escalate);
    let grandchild = Node::new("grandchild", &log, Directive::Resume);
    let chain: Vec<&Node> = above
        .iter()
        .copied()
        .chain([&parent, &child, &grandchild])
        .collect();
    let refs = spawn_chain(&system, &chain);
    let supervisor = chain[failing_depth - 1];
    let child_deciding = child.deciding.lock().unwrap();
    let supervisor_deciding = supervisor.deciding.lock().unwrap();
    let seen = |entry: &str| snapshot_of(&log).iter().any(|e| e == entry);

    refs.last().unwrap().tell("fail", None);
    wait_until("child deciding", || seen("child:decides"));
    refs[failing_depth].tell("fail", None);
    wait_until("supervisor deciding", || snapshot_of(&log).len() >= 2);
    drop(child_deciding);
    settle_for_absence();
    assert_eq!(snapshot_of(&log), owned_strings(expected_before));

    drop(supervisor_deciding);
    // what the others record meanwhile (their own restart or stop) runs on other threads: only `parent` is ordered
    let of_parent = || {
        let mut entries = snapshot_of(&log);
        entries.retain(|e| e.starts_with("parent:"));
        entries
    };
    wait_until("parent", || of_parent().len() >= expected_of_parent.len());
    settle_for_absence();
    assert_eq!(of_parent(), owned_strings(expected_of_parent));
}

fn failed_parent(system_name: &str, top_answers: Directive, expected_of_parent: &[&str]) {
    let log = Log::default();
    let top = Node::new("top", &log, top_answers);
    let parent = Node::new("parent", &log, Directive::Resume);
    let before = ["child:decides", "top:decides"];
    escalate_to_a_suspended_parent(system_name, &[&top], parent, 1, &before, expected_of_parent);
}

#[test]
fn a_failed_parent_decides_about_its_child_only_once_it_is_resumed() {
    failed_parent("t-deferred-resume", Directive::Resume, &["parent:decides"]);
}

#[test]
fn a_restarted_parent_decides_about_its_surviving_child_after_post_restart() {
    let after = ["parent:post-restart", "parent:decides"];
    failed_parent("t-deferred-restart", Directive::Restart, &after);
}

#[test]
fn a_stopped_parent_never_decides_about_the_deferred_failure() {
    let after = ["parent:post-stop"];
    failed_parent("t-deferred-stop", Directive::Stop, &after);
}

#[test]
fn a_parent_suspended_with_its_failed_supervisor_defers_too() {
    let log = Log::default();
    let [top, middle, parent] =
        ["top", "middle", "parent"].map(|name| Node::new(name, &log, Directive::Resume));
    let (before, after) = (["child:decides", "top:decides"], ["parent:decides"]);
    let above = [&top, &middle];
    escalate_to_a_suspended_parent("t-deferred-suspended", &above, parent, 1, &before, &after);
}
