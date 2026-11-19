//! Slice 3b, part 1: failure handling with the default supervisor strategy, through the public API.
//! Every expectation was recorded from Pekko 2.0.0 first (`cases/scenario-3b1.jsonl`, `rust/app-cases/`).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, owned_strings, settle_for_absence, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::dispatch::Terminated;

type Log = Arc<Mutex<Vec<String>>>;

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

#[test]
fn an_exception_in_receive_restarts_the_actor_with_a_fresh_instance() {
    // oracle: r01-restart-lifecycle, r02-state-resets
    let sys = RunningSystem::create("f-restart");
    let (a, log) = probe(&sys, "a");
    for m in ["hi", "boom", "after"] {
        a.tell(m.to_string(), None);
    }
    wait_until("restart and the next message", || {
        snapshot_of(&log).len() == 8
    });
    assert_eq!(
        snapshot_of(&log),
        owned_strings(&[
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
}

#[test]
fn the_failed_message_is_lost_and_the_rest_of_the_mailbox_is_kept_in_order() {
    // oracle: r03-failing-message-lost, r05-mailbox-kept-in-order
    let sys = RunningSystem::create("f-mailbox");
    let (a, log) = probe(&sys, "a");
    for m in ["q0", "boom", "r0", "r1"] {
        a.tell(m.to_string(), None);
    }
    wait_until("r1", || {
        snapshot_of(&log).iter().any(|e| e.starts_with("r1"))
    });
    let got = snapshot_of(&log);
    assert_eq!(&got[got.len() - 2..], &owned_strings(&["r0#1", "r1#2"]));
}

#[test]
fn restarting_does_not_notify_watchers_and_keeps_the_reference_valid() {
    // oracle: r06-watcher-not-notified, r11-restart-keeps-identity
    let sys = RunningSystem::create("f-watch");
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
    wait_until("z", || snapshot_of(&log).iter().any(|e| e.starts_with("z")));
    settle_for_absence();
    assert!(!snapshot_of(&wlog).contains(&"terminated".to_string()));
    assert!(!a.is_terminated());
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
    let sys = RunningSystem::create("f-kids");
    let log = Log::default();
    let l = log.clone();
    let p = sys
        .actor_of_named(Props::new(move || Parent(l.clone())), "p")
        .unwrap();
    p.tell("spawn".to_string(), None);
    wait_until("kid started", || {
        snapshot_of(&log) == owned_strings(&["pre-start"])
    });
    p.tell("boom".to_string(), None);
    wait_until("kid stopped", || {
        snapshot_of(&log) == owned_strings(&["pre-start", "post-stop"])
    });
    p.tell("count".to_string(), None);
    wait_until("count", || {
        snapshot_of(&log).iter().any(|e| e.starts_with("children="))
    });
    assert_eq!(snapshot_of(&log).last().unwrap(), "children=0");
}

#[test]
fn kill_stops_the_actor_without_restart_hooks_and_notifies_watchers() {
    // oracle: k01-kill-stops, k02-kill-no-restart-hooks, k03-after-kill-dead-letters
    let sys = RunningSystem::create("f-kill");
    let (a, log) = probe(&sys, "a");
    a.tell(Kill, None);
    wait_until("stopped", || a.is_terminated());
    assert_eq!(
        snapshot_of(&log),
        owned_strings(&["pre-start", "post-stop"])
    );
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
    let sys = RunningSystem::create("f-pact");
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
        snapshot_of(&lw),
        owned_strings(&["post-stop"]),
        "stopped, not restarted"
    );
    settle_for_absence();
    assert!(!handling.is_terminated());
    assert!(snapshot_of(&lh).is_empty());
}

#[test]
fn a_failing_pre_start_stops_the_actor_and_post_stop_is_not_called() {
    // oracle: i01-prestart-failure-stops, i03-prestart-fail-dead-letters
    let sys = RunningSystem::create("f-init");
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
    assert_eq!(snapshot_of(&log), owned_strings(&["pre-start"]));
}
