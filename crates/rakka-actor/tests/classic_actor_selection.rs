// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala, ActorRefProvider.scala and actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala (Apache-2.0).
//! `actorSelection` from a string path (no wildcards), through the public API. Expected values were observed from Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};

type Log = Arc<Mutex<Vec<String>>>;

struct Run(Box<dyn Fn(&mut ActorContext) + Send + Sync>);

/// Records `name: message from sender` and runs the steps it is told.
struct Node(Log);

impl Actor for Node {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(run) = msg.downcast_ref::<Run>() {
            (run.0)(ctx);
        } else {
            let heard = format!(
                "{}: {} from {}",
                ctx.self_ref().path().name(),
                msg.downcast_ref::<&str>().unwrap(),
                path_of(&ctx.sender()).replace("pekko://sel", "")
            );
            self.0.lock().unwrap().push(heard);
        }
        true
    }
}

fn node(log: &Log) -> Props {
    let log = log.clone();
    Props::new(move || Node(log.clone()))
}

fn spawn(system: &RunningSystem, name: &str, props: Props) -> ActorRef {
    system.actor_of_named(props, name).unwrap()
}

/// Spawns the children `names` of `parent`, each recording into `log`.
fn children(parent: &ActorRef, log: &Log, names: &'static [&'static str]) -> Vec<ActorRef> {
    let spawned: Arc<Mutex<Vec<ActorRef>>> = Arc::default();
    let (stored, log) = (spawned.clone(), log.clone());
    let spawn = move |ctx: &mut ActorContext| {
        let child = |name: &&str| ctx.actor_of_named(node(&log), name).unwrap();
        stored.lock().unwrap().extend(names.iter().map(child));
    };
    parent.tell(Run(Box::new(spawn)), None);
    wait_until("children", || spawned.lock().unwrap().len() == names.len());
    std::mem::take(&mut *spawned.lock().unwrap())
}

/// Sends `message` through `context.actorSelection(path)` of `actor`, with `actor` as sender.
fn send_from(actor: &ActorRef, path: &'static str, message: &'static str) {
    let send = move |ctx: &mut ActorContext| {
        let me = ctx.self_ref();
        ctx.actor_selection(path).tell(message, Some(&me));
    };
    actor.tell(Run(Box::new(send)), None);
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
    let probe = spawn(&system, "r", node(&log));
    let parent = spawn(&system, "p", node(&log));
    let uid = children(&parent, &log, &["c"])[0].path().uid();

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

    // `#uid`: another incarnation gets nothing before the plain path does; the live uid and 0 deliver
    log.lock().unwrap().clear();
    let tell = |path: &str, message| system.actor_selection(path).tell(message, None);
    tell("/user/p/c#999999999", "stale-absolute");
    tell("user/p/c#999999999", "stale-relative");
    tell("pekko://sel/user/p/c#999999999", "stale-uri");
    tell("/user/p/c", "plain");
    assert_eq!(lines(&log, 1), ["c: plain from /deadLetters"]);
    log.lock().unwrap().clear();
    tell(&format!("/user/p/c#{uid}"), "absolute");
    tell(&format!("user/p/c#{uid}"), "relative");
    tell(&format!("pekko://sel/user/p/c#{uid}"), "uri");
    tell("/user/p/c#0", "any");
    let heard = ["absolute", "relative", "uri", "any"];
    let expected = heard.map(|message| format!("c: {message} from /deadLetters"));
    assert_eq!(lines(&log, 4), expected);
}

#[test]
fn a_context_selection_is_anchored_at_the_actor() {
    let system = RunningSystem::create("sel");
    let log = Log::default();
    let parent = spawn(&system, "p", node(&log));
    let spawned = children(&parent, &log, &["c", "d"]);
    let (c, d) = (&spawned[0], &spawned[1]);

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

    // `forward` takes the sender of the message being processed: here the step's own
    log.lock().unwrap().clear();
    for (message, sender) in [("kept", Some(d)), ("nobody-sent", None)] {
        let forward = move |ctx: &mut ActorContext| {
            ctx.actor_selection("../c").forward(Arc::new(message), ctx);
        };
        d.tell(Run(Box::new(forward)), sender);
    }
    assert_eq!(
        lines(&log, 2),
        ["c: kept from /user/p/d", "c: nobody-sent from /deadLetters"]
    );
}
