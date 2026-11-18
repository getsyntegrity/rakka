//! Slice 3b, part 1: failure handling with the default supervisor strategy, through the public API.
//! Every expectation was recorded from Pekko 2.0.0 first (`cases/scenario-3b1.jsonl`, `rust/app-cases/`).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::dispatch::Terminated;

type Log = Arc<Mutex<Vec<String>>>;

fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn settle() {
    std::thread::sleep(Duration::from_millis(150));
}

fn snapshot(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Records lifecycle hooks and string messages; throws `IllegalStateException(msg)` on `boom`.
struct Probe {
    log: Log,
    seen: usize,
    prestart_throws: bool,
}

impl Probe {
    fn new(log: &Log) -> Probe {
        Probe {
            log: log.clone(),
            seen: 0,
            prestart_throws: false,
        }
    }
}

impl Actor for Probe {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("pre-start".into());
        if self.prestart_throws {
            throw(Throwable::illegal_state("init"));
        }
    }
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("post-stop".into());
    }
    fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        reason: &Throwable,
        message: Option<&AnyMessage>,
    ) {
        let m = message
            .and_then(|m| m.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        self.log
            .lock()
            .unwrap()
            .push(format!("pre-restart {reason} / {m}"));
        for child in ctx.children() {
            ctx.stop(&child);
        }
        self.post_stop(ctx);
    }
    fn post_restart(&mut self, ctx: &mut ActorContext, reason: &Throwable) {
        self.log
            .lock()
            .unwrap()
            .push(format!("post-restart {reason}"));
        self.pre_start(ctx);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(m) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.seen += 1;
        self.log.lock().unwrap().push(format!("{m}#{}", self.seen));
        if m == "boom" {
            throw(Throwable::illegal_state("boom"));
        }
        true
    }
}

fn probe(sys: &ActorSystem, name: &str) -> (ActorRef, Log) {
    let log = Log::default();
    let l = log.clone();
    (
        sys.actor_of_named(Props::new(move || Probe::new(&l)), name)
            .unwrap(),
        log,
    )
}

fn shutdown(sys: &ActorSystem) {
    sys.terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated");
}

#[test]
fn an_exception_in_receive_restarts_the_actor_with_a_fresh_instance() {
    // oracle: r01-restart-lifecycle, r02-state-resets
    let sys = ActorSystem::create("f-restart").unwrap();
    let (a, log) = probe(&sys, "a");
    for m in ["hi", "boom", "after"] {
        a.tell(m.to_string(), None);
    }
    wait_until("restart and the next message", || snapshot(&log).len() == 8);
    assert_eq!(
        snapshot(&log),
        strs(&[
            "pre-start",
            "hi#1",
            "boom#2",
            "pre-restart java.lang.IllegalStateException: boom / boom",
            "post-stop",
            "post-restart java.lang.IllegalStateException: boom",
            "pre-start",
            "after#1", // the counter lives in the instance: a restart resets it
        ])
    );
    shutdown(&sys);
}

#[test]
fn the_failed_message_is_lost_and_the_rest_of_the_mailbox_is_kept_in_order() {
    // oracle: r03-failing-message-lost, r05-mailbox-kept-in-order
    let sys = ActorSystem::create("f-mailbox").unwrap();
    let (a, log) = probe(&sys, "a");
    for m in ["q0", "boom", "r0", "r1"] {
        a.tell(m.to_string(), None);
    }
    wait_until("r1", || snapshot(&log).iter().any(|e| e.starts_with("r1")));
    let got = snapshot(&log);
    assert_eq!(&got[got.len() - 2..], &strs(&["r0#1", "r1#2"]));
    shutdown(&sys);
}

#[test]
fn restarting_does_not_notify_watchers_and_keeps_the_reference_valid() {
    // oracle: r06-watcher-not-notified, r11-restart-keeps-identity
    let sys = ActorSystem::create("f-watch").unwrap();
    let (a, log) = probe(&sys, "a");
    let (w, wlog) = probe(&sys, "w");
    struct Watch(ActorRef, Log);
    impl Actor for Watch {
        fn pre_start(&mut self, ctx: &mut ActorContext) {
            ctx.watch(&self.0);
        }
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if msg.is::<Terminated>() {
                self.1.lock().unwrap().push("terminated".into());
            }
            true
        }
    }
    drop(w);
    let (a2, wl) = (a.clone(), wlog.clone());
    let _watcher = sys
        .actor_of_named(Props::new(move || Watch(a2.clone(), wl.clone())), "watcher")
        .unwrap();
    a.tell("boom".to_string(), None);
    a.tell("z".to_string(), None);
    wait_until("z", || snapshot(&log).iter().any(|e| e.starts_with("z")));
    settle();
    assert!(!snapshot(&wlog).contains(&"terminated".to_string()));
    assert!(!a.is_terminated());
    shutdown(&sys);
}

#[test]
fn restart_stops_the_children_and_does_not_recreate_them() {
    // oracle: r07-restart-stops-children, r08-new-child-after-restart
    struct Parent(Log);
    impl Actor for Parent {
        fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            match msg.downcast_ref::<String>().map(String::as_str) {
                Some("spawn") => {
                    let l = self.0.clone();
                    ctx.actor_of_named(Props::new(move || Probe::new(&l)), "kid")
                        .unwrap();
                    true
                }
                Some("boom") => throw(Throwable::illegal_state("boom")),
                Some("count") => {
                    self.0
                        .lock()
                        .unwrap()
                        .push(format!("children={}", ctx.children().len()));
                    true
                }
                _ => false,
            }
        }
    }
    let sys = ActorSystem::create("f-kids").unwrap();
    let log = Log::default();
    let l = log.clone();
    let p = sys
        .actor_of_named(Props::new(move || Parent(l.clone())), "p")
        .unwrap();
    p.tell("spawn".to_string(), None);
    wait_until("kid started", || snapshot(&log) == strs(&["pre-start"]));
    p.tell("boom".to_string(), None);
    wait_until("kid stopped", || {
        snapshot(&log) == strs(&["pre-start", "post-stop"])
    });
    p.tell("count".to_string(), None);
    wait_until("count", || {
        snapshot(&log).iter().any(|e| e.starts_with("children="))
    });
    assert_eq!(snapshot(&log).last().unwrap(), "children=0");
    shutdown(&sys);
}

#[test]
fn kill_stops_the_actor_without_restart_hooks_and_notifies_watchers() {
    // oracle: k01-kill-stops, k02-kill-no-restart-hooks, k03-after-kill-dead-letters
    let sys = ActorSystem::create("f-kill").unwrap();
    let (a, log) = probe(&sys, "a");
    a.tell(Kill, None);
    wait_until("stopped", || a.is_terminated());
    assert_eq!(snapshot(&log), strs(&["pre-start", "post-stop"]));
    shutdown(&sys);
}

#[test]
fn an_unhandled_terminated_stops_the_watcher_with_death_pact() {
    // oracle: d01-unhandled-terminated-stops, d02-handled-terminated-survives, d03-no-restart-on-deathpact
    struct Watcher(ActorRef, Log, bool);
    impl Actor for Watcher {
        fn pre_start(&mut self, ctx: &mut ActorContext) {
            ctx.watch(&self.0);
        }
        fn post_stop(&mut self, _ctx: &mut ActorContext) {
            self.1.lock().unwrap().push("post-stop".into());
        }
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            msg.is::<Terminated>() && self.2
        }
    }
    let sys = ActorSystem::create("f-pact").unwrap();
    let (x, _) = probe(&sys, "x");
    let (y, _) = probe(&sys, "y");
    let (lw, lh) = (Log::default(), Log::default());
    let (xr, yr, l1, l2) = (x.clone(), y.clone(), lw.clone(), lh.clone());
    let ignoring = sys
        .actor_of_named(
            Props::new(move || Watcher(xr.clone(), l1.clone(), false)),
            "ignoring",
        )
        .unwrap();
    let handling = sys
        .actor_of_named(
            Props::new(move || Watcher(yr.clone(), l2.clone(), true)),
            "handling",
        )
        .unwrap();
    sys.stop(&x);
    sys.stop(&y);
    wait_until("DeathPact stop", || ignoring.is_terminated());
    assert_eq!(
        snapshot(&lw),
        strs(&["post-stop"]),
        "stopped, not restarted"
    );
    settle();
    assert!(!handling.is_terminated());
    assert!(snapshot(&lh).is_empty());
    shutdown(&sys);
}

#[test]
fn a_failing_pre_start_stops_the_actor_and_post_stop_is_not_called() {
    // oracle: i01-prestart-failure-stops, i03-prestart-fail-dead-letters
    let sys = ActorSystem::create("f-init").unwrap();
    let log = Log::default();
    let l = log.clone();
    let a = sys
        .actor_of_named(
            Props::new(move || Probe {
                prestart_throws: true,
                ..Probe::new(&l)
            }),
            "a",
        )
        .unwrap();
    wait_until("stopped", || a.is_terminated());
    assert_eq!(snapshot(&log), strs(&["pre-start"]));
    shutdown(&sys);
}
