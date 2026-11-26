// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala, ActorRef.scala and actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala (Apache-2.0).
//! Actor selections with `*` and `?` patterns. Expected values were observed from Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{
    Actor, ActorContext, ActorIdentity, AnyMessage, Identify, PoisonPill, Props,
};
use rakka_actor::event::{Channel, DeadLetter};

type Log = Arc<Mutex<Vec<String>>>;

struct Run(Box<dyn Fn(&mut ActorContext) + Send + Sync>);

fn short(actor: &ActorRef) -> String {
    path_of(actor).replace("pekko://sel", "")
}

fn text(msg: &AnyMessage) -> &str {
    msg.downcast_ref::<&str>().copied().unwrap_or("?")
}

/// Records what it is told (strings, `ActorIdentity` replies, dead letters) and runs the steps it is told.
struct Node(Log);

impl Actor for Node {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(run) = msg.downcast_ref::<Run>() {
            (run.0)(ctx);
            return true;
        }
        let line = if let Some(dead) = msg.downcast_ref::<DeadLetter>() {
            let to = short(&dead.recipient);
            format!("dead {} to {to}", text(&dead.message))
        } else if let Some(identity) = msg.downcast_ref::<ActorIdentity>() {
            let found = identity.actor_ref.as_ref().map(short);
            let correlation = text(&identity.correlation_id);
            format!("{correlation} {}", found.as_deref().unwrap_or("-"))
        } else {
            let me = ctx.self_ref().path().name().to_owned();
            format!("{me}: {} from {}", text(msg), short(&ctx.sender()))
        };
        self.0.lock().unwrap().push(line);
        true
    }
}

/// The tree of ActorSelectionSpec "identify actors with wildcard selection correctly" (`/user/a/b1`, `/user/a/b2/c/d`) and a
/// probe `/user/r`, all recording into one log.
struct Tree {
    system: RunningSystem,
    log: Log,
    probe: ActorRef,
    /// a, b1, b2, c, d
    nodes: Vec<ActorRef>,
}

impl Tree {
    fn new() -> Self {
        let system = RunningSystem::create("sel");
        let log = Log::default();
        let recorded = log.clone();
        let props = Props::new(move || Node(recorded.clone()));
        let probe = system.actor_of_named(props.clone(), "r").unwrap();
        let a = system.actor_of_named(props.clone(), "a").unwrap();
        let mut nodes = vec![a];
        for (parent, name) in [(0, "b1"), (0, "b2"), (2, "c"), (3, "d")] {
            let (spawned, child_props) = (Arc::new(Mutex::new(None)), props.clone());
            let stored = spawned.clone();
            let spawn = move |ctx: &mut ActorContext| {
                let child = ctx.actor_of_named(child_props.clone(), name).unwrap();
                *stored.lock().unwrap() = Some(child);
            };
            nodes[parent].tell(Run(Box::new(spawn)), None);
            wait_until("child", || spawned.lock().unwrap().is_some());
            nodes.push(spawned.lock().unwrap().take().unwrap());
        }
        Tree {
            system,
            log,
            probe,
            nodes,
        }
    }

    /// `system.actorSelection(path)` with the probe as sender.
    fn tell<M: std::any::Any + Send + Sync>(&self, path: &str, message: M) {
        let selection = self.system.actor_selection(path);
        selection.tell(message, Some(&self.probe));
    }

    fn identify(&self, path: &str, correlation: &'static str) {
        let message_id = Arc::new(correlation);
        self.tell(path, Identify { message_id });
    }

    /// Waits for `count` lines, then sends a barrier through every node and the probe: nothing else may have been recorded.
    /// Lines are returned sorted, as different actors record in no fixed order.
    fn recorded(&self, count: usize) -> Vec<String> {
        wait_until("lines", || self.log.lock().unwrap().len() >= count);
        for (barriers, actor) in self.nodes.iter().chain([&self.probe]).enumerate() {
            actor.tell("barrier", None);
            wait_until("barrier", || {
                let log = self.log.lock().unwrap();
                log.iter().filter(|line| line.contains("barrier")).count() > barriers
            });
        }
        let mut lines = std::mem::take(&mut *self.log.lock().unwrap());
        lines.retain(|line| !line.contains("barrier"));
        lines.sort();
        lines
    }
}

#[test]
fn every_match_receives_the_message_with_the_sender() {
    let tree = Tree::new();
    for (path, message) in [
        ("/user/a/*", "children"),
        ("/user/a/**", "double"),
        ("/user/a/*1", "suffix"),
        ("user/*/b2/c", "inner"),
        ("pekko://sel/user/a/*/c/*", "uri"),
        // once per match
        ("/user/a/*/..", "twice"),
        // in a string path `?` starts the URI query, which is dropped
        ("/user/a/b1?x", "query"),
    ] {
        tree.tell(path, message);
    }
    // ActorSelectionSpec "send messages with correct sender" and "drop messages which cannot be delivered", from d
    let probe = tree.probe.clone();
    let from_d = move |ctx: &mut ActorContext| {
        let tell = |path, message| ctx.actor_selection(path).tell(message, Some(&probe));
        tell("../*", "own");
        tell("../../../*/c", "cousin");
    };
    tree.nodes[4].tell(Run(Box::new(from_d)), None);

    let expected = [
        "a: twice from /user/r",
        "a: twice from /user/r",
        "b1: children from /user/r",
        "b1: double from /user/r",
        "b1: query from /user/r",
        "b1: suffix from /user/r",
        "b2: children from /user/r",
        "b2: double from /user/r",
        "c: cousin from /user/r",
        "c: inner from /user/r",
        "d: own from /user/r",
        "d: uri from /user/r",
    ];
    assert_eq!(tree.recorded(expected.len()), expected);
}

#[test]
fn identify_is_answered_by_each_match_and_by_none_only_before_a_fan_out() {
    let tree = Tree::new();
    for (path, correlation) in [
        ("/user/a/*", "1"),
        ("/user/a/b1/*", "2"),
        ("/user/a/*/c", "3"),
        ("/user/a/b2/*/d", "4"),
        ("/user/a/*/*/d", "5"),
        ("/user/a/*/c/*", "6"),
        ("/user/a/b2/*/d/e", "7"),
        // silent: two matched at `*`
        ("/user/a/*/c/d/e", "8"),
        ("/user/a/x*", "9"),
        ("/user/a/*1/nothing", "10"),
        // silent even though the walk ends at `deadLetters`
        ("/*/../deadLetters/x", "11"),
        ("/*/../deadLetters", "12"),
    ] {
        tree.identify(path, correlation);
    }
    let expected = [
        "1 /user/a/b1",
        "1 /user/a/b2",
        "10 -",
        "12 -",
        "12 -",
        "2 -",
        "3 /user/a/b2/c",
        "4 /user/a/b2/c/d",
        "5 /user/a/b2/c/d",
        "6 /user/a/b2/c/d",
        "7 -",
        "9 -",
    ];
    assert_eq!(tree.recorded(expected.len()), expected);
}

#[test]
fn a_miss_is_a_dead_letter_below_the_single_match_and_silent_after_a_fan_out() {
    let tree = Tree::new();
    let events = tree.system.event_stream();
    events.subscribe(&tree.probe, Channel::DeadLetter);
    for (path, message) in [
        ("/user/a/x*", "none"),
        ("/user/a/x*/deeper", "none-deep"),
        ("/user/a/b1/*", "leafless"),
        ("/user/a/*1/nothing", "single-then-name"),
        ("/user/a/b*2/zz*/yy", "single-then-pattern"),
        ("/user/a/*1/../x*", "single-then-up"),
        ("/user/a/*/zz", "fan-name"),
        ("/user/a/*/zz*", "fan-pattern"),
        ("/user/a/*/c/zz", "fan-later"),
        // once for each of the two root children
        ("/*/../deadLetters/x", "fan-dead"),
    ] {
        tree.tell(path, message);
    }
    tree.tell("/user/a/x*", PoisonPill);
    tree.tell("/user/missing", "last");

    let expected = [
        "dead fan-dead to /deadLetters",
        "dead fan-dead to /deadLetters",
        "dead last to /user/missing",
        "dead leafless to /user/a/b1/*",
        "dead none to /user/a/x*",
        "dead none-deep to /user/a/x*/deeper",
        "dead single-then-name to /user/a/b1/nothing",
        "dead single-then-pattern to /user/a/b2/zz*/yy",
        "dead single-then-up to /user/a/b1/../x*",
    ];
    assert_eq!(tree.recorded(expected.len()), expected);
}

/// A stack overflow aborts the process, so the walk runs in a child: this test binary again, with `DEEP` set.
#[test]
fn very_many_wildcard_levels_do_not_exhaust_the_stack() {
    const DEEP: &str = "RAKKA_DEEP_SELECTION";
    if std::env::var_os(DEEP).is_none() {
        let test = "very_many_wildcard_levels_do_not_exhaust_the_stack";
        let mut child = Command::new(std::env::current_exe().unwrap());
        let output = child.args(["--exact", test]).env(DEEP, "1").output();
        let output = output.unwrap();
        let errors = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{}: {errors}", output.status);
        // (a filter that matched nothing would also succeed)
        return assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
    let walk = || {
        let tree = Tree::new();
        // `us*` matches only `user`: 20 000 levels, each walked from the match before
        tree.tell(&format!("/{}user/a", "us*/../".repeat(20_000)), "deep");
        assert_eq!(tree.recorded(1), ["a: deep from /user/r"]);
    };
    let thread = std::thread::Builder::new().stack_size(2 << 20);
    thread.spawn(walk).unwrap().join().unwrap();
}

/// ActorSelectionSpec "deliver a wildcard selection promptly even when the pattern would make a regex backtrack".
#[test]
fn a_pattern_that_would_backtrack_is_answered_promptly() {
    let tree = Tree::new();
    let name = "a".repeat(36);
    let child = Props::new(|| Node(Log::default()));
    tree.system.actor_of_named(child, &name).unwrap();
    let pattern = format!("/user/{}*b", "*a".repeat(12));
    let started = Instant::now();
    tree.identify(&pattern, "slow");

    assert_eq!(tree.recorded(1), ["slow -"]);
    assert!(started.elapsed() < Duration::from_secs(3));
}
