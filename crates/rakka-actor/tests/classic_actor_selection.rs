// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala, ActorRefProvider.scala and actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala (Apache-2.0).
//! `actorSelection` from a string path (no wildcards), through the public API; misses are in `classic_actor_selection_miss.rs`.
//! Expected values were observed from Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};

/// One line per message heard: `name: message from sender`.
type Log = Arc<Mutex<Vec<String>>>;

/// A step run inside the actor's own context.
struct Run(Box<dyn Fn(&mut ActorContext) + Send + Sync>);

/// Records the strings it hears, or forwards them to `forward_to`; starts the children it is built with.
struct Node {
    log: Log,
    children: &'static [&'static str],
    forward_to: Option<&'static str>,
}

impl Actor for Node {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        for name in self.children {
            let props = node(&self.log, &[], None);
            ctx.actor_of_named(props, name).unwrap();
        }
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(run) = msg.downcast_ref::<Run>() {
            (run.0)(ctx);
        } else if let Some(path) = self.forward_to {
            ctx.actor_selection(path).forward(msg.clone(), ctx);
        } else {
            let heard = format!(
                "{}: {} from {}",
                ctx.self_ref().path().name(),
                msg.downcast_ref::<&str>().unwrap(),
                path_of(&ctx.sender()).replace("pekko://sel", "")
            );
            self.log.lock().unwrap().push(heard);
        }
        true
    }
}

fn node(log: &Log, children: &'static [&'static str], forward_to: Option<&'static str>) -> Props {
    let log = log.clone();
    Props::new(move || Node {
        log: log.clone(),
        children,
        forward_to,
    })
}

fn spawn(system: &RunningSystem, name: &str, props: Props) -> ActorRef {
    system.actor_of_named(props, name).unwrap()
}

/// The `count` children of `parent`, which exist once it has started.
fn children_of(parent: &ActorRef, count: usize) -> Vec<ActorRef> {
    let found: Arc<Mutex<Vec<ActorRef>>> = Arc::default();
    let stored = found.clone();
    let lookup = move |ctx: &mut ActorContext| *stored.lock().unwrap() = ctx.children();
    parent.tell(Run(Box::new(lookup)), None);
    wait_until("children", || found.lock().unwrap().len() == count);
    std::mem::take(&mut *found.lock().unwrap())
}

/// Sends `message` through `context.actorSelection(path)` of `actor`, with `actor` as sender.
fn send_from(actor: &ActorRef, path: &'static str, message: &'static str) {
    let step = move |ctx: &mut ActorContext| {
        let me = ctx.self_ref();
        ctx.actor_selection(path).tell(message, Some(&me));
    };
    actor.tell(Run(Box::new(step)), None);
}

/// Waits for `count` lines and returns them grouped by who heard them, each one's in the order it heard them.
fn lines(log: &Log, count: usize) -> Vec<String> {
    wait_until("recorded lines", || log.lock().unwrap().len() >= count);
    let mut lines = log.lock().unwrap().clone();
    lines.sort_by_key(|line| line.split(' ').next().map(str::to_owned));
    lines
}

#[test]
fn absolute_relative_and_full_uri_paths_deliver_with_the_sender() {
    let system = RunningSystem::create("sel");
    let log = Log::default();
    let probe = spawn(&system, "r", node(&log, &[], None));
    let parent = spawn(&system, "p", node(&log, &["c"], None));
    children_of(&parent, 1);

    for (path, message) in [
        ("/user/p/c", "absolute"),
        ("user/p/c", "relative"),
        ("pekko://sel/user/p/c", "uri"),
        ("/user//p//c/", "slashes"),
        ("/../user/p/c", "above-root"),
        ("/user/p/c/..", "up"),
    ] {
        system.actor_selection(path).tell(message, Some(&probe));
    }
    system.actor_selection("/user/p/c").tell("unsent", None);

    assert_eq!(
        lines(&log, 7),
        [
            "c: absolute from /user/r",
            "c: relative from /user/r",
            "c: uri from /user/r",
            "c: slashes from /user/r",
            "c: above-root from /user/r",
            "c: unsent from /deadLetters",
            "p: up from /user/r",
        ]
    );
}

#[test]
fn a_context_selection_is_anchored_at_the_actor() {
    let system = RunningSystem::create("sel");
    let log = Log::default();
    let parent = spawn(&system, "p", node(&log, &["c", "d"], None));
    let children = children_of(&parent, 2);
    let (c, d) = (&children[0], &children[1]);

    send_from(&parent, "c", "to-child");
    wait_until("to-child", || log.lock().unwrap().len() == 1);
    send_from(c, "../d", "to-sibling");
    send_from(c, "..", "to-parent");
    send_from(c, "../../p/d", "back-down");
    wait_until("c is done", || log.lock().unwrap().len() == 4);
    send_from(d, "/user/p/c", "from-root");

    assert_eq!(
        lines(&log, 5),
        [
            "c: to-child from /user/p",
            "c: from-root from /user/p/d",
            "d: to-sibling from /user/p/c",
            "d: back-down from /user/p/c",
            "p: to-parent from /user/p/c",
        ]
    );
}

#[test]
fn forward_keeps_the_original_sender() {
    let system = RunningSystem::create("sel");
    let log = Log::default();
    let probe = spawn(&system, "r", node(&log, &[], None));
    spawn(&system, "c", node(&log, &[], None));
    let forwarder = spawn(&system, "f", node(&log, &[], Some("../c")));

    forwarder.tell("kept", Some(&probe));
    forwarder.tell("nobody-sent", None);

    assert_eq!(
        lines(&log, 2),
        ["c: kept from /user/r", "c: nobody-sent from /deadLetters"]
    );
}
