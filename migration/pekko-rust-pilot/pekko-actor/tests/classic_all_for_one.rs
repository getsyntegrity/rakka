//! Slice 3b-5: `AllForOneStrategy`. A supervisor with two children, `c1` fails and `c2` is healthy.
//! Every expectation was recorded from Pekko 2.0.0 first (`cases/scenario-3b5.jsonl`, `rust/app-cases/`); the scenario names are in each test.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use pekko_actor::actor_ref::ActorRef;
use pekko_actor::fault::{Throwable, throw};
use pekko_actor::props::{Actor, ActorContext, AnyMessage, Props};
use pekko_actor::supervision::{AllForOneStrategy, Directive, SupervisorStrategy};
use pekko_actor::system::ActorSystem;

type Log = Arc<Mutex<Vec<String>>>;

fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn snap(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// records lifecycle, messages (`msg#n`, `n` per instance) and what `preRestart` was told; throws `IllegalStateException(msg)` on `b*`
struct Kid {
    log: Log,
    seen: usize,
}

impl Actor for Kid {
    fn pre_start(&mut self, _c: &mut ActorContext) {
        self.log.lock().unwrap().push("pre-start".into());
    }
    fn post_stop(&mut self, _c: &mut ActorContext) {
        self.log.lock().unwrap().push("post-stop".into());
    }
    fn pre_restart(&mut self, c: &mut ActorContext, reason: &Throwable, m: Option<&AnyMessage>) {
        let msg = m.and_then(|m| m.downcast_ref::<String>().cloned()).unwrap_or_else(|| "None".into());
        self.log.lock().unwrap().push(format!("pre-restart {} / {msg}", reason.message.clone().unwrap_or_default()));
        for k in c.children() {
            c.stop(&k);
        }
        self.post_stop(c);
    }
    fn post_restart(&mut self, c: &mut ActorContext, _r: &Throwable) {
        self.log.lock().unwrap().push("post-restart".into());
        self.pre_start(c);
    }
    fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(m) = msg.downcast_ref::<String>() else { return false };
        self.seen += 1;
        self.log.lock().unwrap().push(format!("{m}#{}", self.seen));
        if matches!(m.as_str(), "bres" | "brst" | "bstp" | "other") {
            throw(Throwable::illegal_state(m));
        }
        true
    }
}

/// `p`: an `AllForOneStrategy` over `c1` and `c2`; `bres`/`brst`/`bstp` map to the directives, anything else to the default decider.
struct Sup {
    logs: [Log; 2],
    refs: Arc<Mutex<Vec<ActorRef>>>,
    max: i32,
}

impl Actor for Sup {
    fn pre_start(&mut self, c: &mut ActorContext) {
        for (i, name) in ["c1", "c2"].into_iter().enumerate() {
            let log = self.logs[i].clone();
            let kid = c.actor_of_named(Props::new(move || Kid { log: log.clone(), seen: 0 }), name).unwrap();
            self.refs.lock().unwrap().push(kid);
        }
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        AllForOneStrategy::new(self.max, |e: &Throwable| match e.message.as_deref() {
            Some("bres") => Directive::Resume,
            Some("brst") => Directive::Restart,
            Some("bstp") => Directive::Stop,
            _ => pekko_actor::supervision::default_decider(e),
        })
        .into()
    }
}

struct Fixture {
    sys: ActorSystem,
    c1: ActorRef,
    c2: ActorRef,
    l1: Log,
    l2: Log,
}

fn fixture(name: &str, max: i32) -> Fixture {
    let sys = ActorSystem::create(name).unwrap();
    let (l1, l2) = (Log::default(), Log::default());
    let refs = Arc::new(Mutex::new(Vec::new()));
    let (logs, r) = ([l1.clone(), l2.clone()], refs.clone());
    sys.actor_of_named(Props::new(move || Sup { logs: logs.clone(), refs: r.clone(), max }), "p").unwrap();
    wait_until("children", || refs.lock().unwrap().len() == 2);
    let kids = refs.lock().unwrap().clone();
    let (c1, c2) = (kids[0].clone(), kids[1].clone());
    Fixture { sys, c1, c2, l1, l2 }
}

fn say(to: &ActorRef, msgs: &[&str]) {
    for m in msgs {
        to.tell(m.to_string(), None);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.sys.terminate().wait(Duration::from_secs(10)).expect("terminated");
    }
}

/// o01-resume-only-failed (characterization: it already agreed with OneForOne): Resume touches only the child that failed.
#[test]
fn resume_applies_to_the_failed_child_only() {
    let f = fixture("afo-resume", -1);
    say(&f.c2, &["a"]);
    wait_until("a", || snap(&f.l2).len() == 2);
    say(&f.c1, &["m1", "bres", "m2"]);
    say(&f.c2, &["b"]);
    wait_until("both done", || snap(&f.l1).len() == 4 && snap(&f.l2).len() == 3);
    assert_eq!(snap(&f.l1), strs(&["pre-start", "m1#1", "bres#2", "m2#3"]));
    assert_eq!(snap(&f.l2), strs(&["pre-start", "a#1", "b#2"]), "the healthy child saw nothing");
}

/// o02-restart-both, o03-restart-identity, o07-default-restarts-both: Restart replaces every child; the healthy one is told the
/// failed child's exception and no message; state restarts from zero in both; the references stay valid.
#[test]
fn restart_replaces_every_child() {
    let f = fixture("afo-restart", -1);
    let (r1, r2) = (f.c1.clone(), f.c2.clone());
    say(&f.c2, &["a"]);
    wait_until("a", || snap(&f.l2).len() == 2);
    say(&f.c1, &["m1", "brst"]);
    wait_until("both restarted", || snap(&f.l1).len() == 7 && snap(&f.l2).len() == 6);
    say(&f.c1, &["m2"]);
    say(&f.c2, &["b"]);
    wait_until("new instances work", || snap(&f.l1).len() == 8 && snap(&f.l2).len() == 7);
    assert_eq!(snap(&f.l1), strs(&["pre-start", "m1#1", "brst#2", "pre-restart brst / brst", "post-stop", "post-restart", "pre-start", "m2#1"]));
    assert_eq!(snap(&f.l2), strs(&["pre-start", "a#1", "pre-restart brst / None", "post-stop", "post-restart", "pre-start", "b#1"]));
    assert!(f.c1 == r1 && f.c2 == r2 && !f.c1.is_terminated() && !f.c2.is_terminated(), "same references, nobody terminated");
}

/// o05-restart-queued: what was queued behind the failing message goes to the new instance of that child.
#[test]
fn restart_keeps_the_messages_queued_behind_the_failure() {
    let f = fixture("afo-queued", -1);
    say(&f.c1, &["brst", "q0", "q1"]);
    wait_until("c1", || snap(&f.l1).len() == 8);
    assert_eq!(snap(&f.l1)[6..], strs(&["q0#1", "q1#2"])[..]);
}

/// o04-stop-both, o06-stop-queued-dead: Stop applies to every child, not only the failed one.
#[test]
fn stop_terminates_every_child() {
    let f = fixture("afo-stop", -1);
    say(&f.c2, &["a"]);
    wait_until("a", || snap(&f.l2).len() == 2);
    say(&f.c1, &["m1", "bstp", "q0"]);
    wait_until("both stopped", || snap(&f.l1).last().map(String::as_str) == Some("post-stop") && snap(&f.l2).last().map(String::as_str) == Some("post-stop"));
    assert_eq!(snap(&f.l1), strs(&["pre-start", "m1#1", "bstp#2", "post-stop"]));
    assert_eq!(snap(&f.l2), strs(&["pre-start", "a#1", "post-stop"]), "the healthy child is stopped too");
}

/// r01 / r02 / r03 / r04: the restart limit is shared in effect: when it is exhausted for any child, every child is stopped.
#[test]
fn an_exhausted_restart_limit_stops_both_children() {
    // max 0: the first restart is already refused
    let f = fixture("afo-max0", 0);
    say(&f.c1, &["brst"]);
    wait_until("both stopped", || snap(&f.l1).last().map(String::as_str) == Some("post-stop") && snap(&f.l2).last().map(String::as_str) == Some("post-stop"));
    assert_eq!(snap(&f.l1), strs(&["pre-start", "brst#1", "post-stop"]), "no restart hooks");
    assert_eq!(snap(&f.l2), strs(&["pre-start", "post-stop"]));

    // max 1: c1's failure restarts both; then the healthy c2 fails and its restart is refused: both stop
    let f = fixture("afo-max1", 1);
    say(&f.c1, &["brst"]);
    wait_until("both restarted", || snap(&f.l1).len() == 6 && snap(&f.l2).len() == 5);
    say(&f.c2, &["brst"]);
    wait_until("both stopped", || snap(&f.l1).last().map(String::as_str) == Some("post-stop") && snap(&f.l2).last().map(String::as_str) == Some("post-stop"));
    assert_eq!(snap(&f.l1), strs(&["pre-start", "brst#1", "pre-restart brst / brst", "post-stop", "post-restart", "pre-start", "post-stop"]));
    assert_eq!(snap(&f.l2), strs(&["pre-start", "pre-restart brst / None", "post-stop", "post-restart", "pre-start", "brst#1", "post-stop"]));
}

/// r03-resume-not-counted: Resume never consumes the limit; r05 (`-1`) is unlimited.
#[test]
fn resume_does_not_count_and_a_negative_limit_is_unlimited() {
    let f = fixture("afo-count", 1);
    say(&f.c1, &["bres", "bres", "brst"]);
    wait_until("restarted", || snap(&f.l1).len() == 8 && snap(&f.l2).len() == 5);
    say(&f.c2, &["brst"]);
    wait_until("stopped", || snap(&f.l1).last().map(String::as_str) == Some("post-stop"));

    let f = fixture("afo-unlimited", -1);
    say(&f.c1, &["brst", "brst"]);
    say(&f.c2, &["brst"]);
    wait_until("three restarts", || snap(&f.l1).len() >= 12 && snap(&f.l2).len() >= 12);
    assert!(!f.c1.is_terminated() && !f.c2.is_terminated());
}
