//! Slice 3b-2: `OneForOneStrategy` with a configurable decider (Resume / Restart / Stop) and its restart limit.
//! Every expectation was recorded from Pekko 2.0.0 first (`cases/scenario-3b2.jsonl`, `rust/app-cases/`).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, owned_strings, settle_for_absence, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use rakka_actor::event::{Channel, ErrorEvent, WarningEvent};

type Log = Arc<Mutex<Vec<String>>>;

/// Records its lifecycle and messages (`msg#n`, `n` counts per instance); throws `IllegalStateException(msg)` on `b*` messages
/// and `ActorKilledException` on `kill-me`.
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
    fn pre_restart(&mut self, c: &mut ActorContext, reason: &Throwable, _m: Option<&AnyMessage>) {
        self.log.lock().unwrap().push(format!(
            "pre-restart {}",
            reason.message().unwrap_or_default()
        ));
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
        let Some(m) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.seen += 1;
        self.log.lock().unwrap().push(format!("{m}#{}", self.seen));
        match m.as_str() {
            "bres" | "brst" | "bstp" | "other" => throw(Throwable::illegal_state(m)),
            "kill-me" => throw(Throwable::new(
                "org.apache.pekko.actor.ActorKilledException",
                Some("kill-me"),
            )),
            _ => true,
        }
    }
}

/// Supervisor with two children `c1` and `c2`; `bres`/`brst`/`bstp` map to the directives, `kill-me` to Restart, `max` is `maxNrOfRetries`.
struct Sup {
    logs: [Log; 2],
    refs: Arc<Mutex<Vec<ActorRef>>>,
    max: i32,
}

impl Actor for Sup {
    fn pre_start(&mut self, c: &mut ActorContext) {
        for (i, name) in ["c1", "c2"].into_iter().enumerate() {
            let log = self.logs[i].clone();
            let kid = c
                .actor_of_named(
                    Props::new(move || Kid {
                        log: log.clone(),
                        seen: 0,
                    }),
                    name,
                )
                .unwrap();
            self.refs.lock().unwrap().push(kid);
        }
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        OneForOneStrategy::new(self.max, |e: &Throwable| match e.message() {
            Some("bres") => Directive::Resume,
            Some("brst") | Some("kill-me") => Directive::Restart,
            Some("bstp") => Directive::Stop,
            _ => rakka_actor::actor::supervision::default_decider(e),
        })
        .into()
    }
}

struct Fixture {
    sys: RunningSystem,
    c1: ActorRef,
    c2: ActorRef,
    l1: Log,
    l2: Log,
}

fn fixture(name: &str, max: i32) -> Fixture {
    let sys = RunningSystem::create(name);
    let (l1, l2) = (Log::default(), Log::default());
    let refs = Arc::new(Mutex::new(Vec::new()));
    let (logs, r) = ([l1.clone(), l2.clone()], refs.clone());
    sys.actor_of_named(
        Props::new(move || Sup {
            logs: logs.clone(),
            refs: r.clone(),
            max,
        }),
        "p",
    )
    .unwrap();
    wait_until("children", || refs.lock().unwrap().len() == 2);
    let kids = refs.lock().unwrap().clone();
    let (c1, c2) = (kids[0].clone(), kids[1].clone());
    Fixture {
        sys,
        c1,
        c2,
        l1,
        l2,
    }
}

fn say(to: &ActorRef, msgs: &[&str]) {
    for m in msgs {
        to.tell(m.to_string(), None);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.sys
            .terminate()
            .wait(Duration::from_secs(10))
            .expect("terminated")
            .expect("terminated normally");
    }
}

#[test]
fn resume_keeps_the_instance_and_state_and_drops_only_the_failed_message() {
    // oracle: a01-resume-keeps-state, a02-resume-next-queued, a03-resume-message-lost, a04-resume-ref-alive
    let f = fixture("s-resume", -1);
    say(&f.c1, &["q0", "q1", "bres", "r0", "r1"]);
    wait_until("r1", || snapshot_of(&f.l1).len() == 6);
    assert_eq!(
        snapshot_of(&f.l1),
        owned_strings(&["pre-start", "q0#1", "q1#2", "bres#3", "r0#4", "r1#5"]),
        "no restart hooks, counter keeps counting"
    );
    assert!(!f.c1.is_terminated());
}

#[test]
fn restart_replaces_the_instance_and_continues_with_the_queued_messages() {
    // oracle: a05-restart-resets-state, a06-restart-next-queued, a07-restart-message-lost, a08-restart-ref-alive
    let f = fixture("s-restart", -1);
    say(&f.c1, &["q0", "brst", "r0", "r1"]);
    wait_until("r1", || snapshot_of(&f.l1).len() == 9);
    assert_eq!(
        snapshot_of(&f.l1),
        owned_strings(&[
            "pre-start",
            "q0#1",
            "brst#2",
            "pre-restart brst",
            "post-stop",
            "post-restart",
            "pre-start",
            "r0#1",
            "r1#2"
        ])
    );
    assert!(!f.c1.is_terminated());
}

#[test]
fn a_decider_can_restart_what_the_default_would_stop() {
    // oracle: the decider is consulted first (a12 uses the default for unmapped exceptions)
    let f = fixture("s-override", -1);
    say(&f.c1, &["kill-me", "after"]);
    wait_until("after", || {
        snapshot_of(&f.l1).iter().any(|e| e.starts_with("after"))
    });
    assert!(
        snapshot_of(&f.l1).contains(&"post-restart".to_string()),
        "restarted although ActorKilledException stops by default"
    );
    assert!(!f.c1.is_terminated());
}

#[test]
fn unmapped_exceptions_use_the_default_decider() {
    // oracle: a12-unmapped-default-restart
    let f = fixture("s-default", -1);
    say(&f.c1, &["m1", "other", "m2"]);
    wait_until("m2", || {
        snapshot_of(&f.l1).iter().any(|e| e.starts_with("m2"))
    });
    assert_eq!(snapshot_of(&f.l1).last().unwrap(), "m2#1");
}

#[test]
fn stop_stops_the_child_and_dead_letters_what_was_queued() {
    // oracle: a09-stop-directive, a10-stop-queued-dead, a11-stop-then-late-letter
    let f = fixture("s-stop", -1);
    say(&f.c1, &["m1", "bstp", "q0", "q1"]);
    wait_until("stopped", || f.c1.is_terminated());
    settle_for_absence();
    assert_eq!(
        snapshot_of(&f.l1),
        owned_strings(&["pre-start", "m1#1", "bstp#2", "post-stop"])
    );
}

#[test]
fn the_healthy_sibling_is_not_affected_by_any_directive() {
    // oracle: w-resume-healthy-sibling, w-restart-healthy-sibling, w-stop-healthy-sibling, w-both-fail-*
    for (tag, bad) in [("resume", "bres"), ("restart", "brst"), ("stop", "bstp")] {
        let f = fixture(&format!("s-sib-{tag}"), -1);
        say(&f.c2, &["a"]);
        say(&f.c1, &["m1", bad, "m2"]);
        say(&f.c2, &["b", "c"]);
        wait_until("sibling", || snapshot_of(&f.l2).len() == 4);
        settle_for_absence();
        assert_eq!(
            snapshot_of(&f.l2),
            owned_strings(&["pre-start", "a#1", "b#2", "c#3"]),
            "{tag}: sibling saw nothing but its own messages"
        );
        assert!(!f.c2.is_terminated());
    }
}

#[test]
fn a_sequence_of_directives_on_one_child() {
    // oracle: a13-sequence-all-three
    let f = fixture("s-seq", -1);
    say(&f.c1, &["m1", "bres", "m2", "brst", "m3", "bstp"]);
    wait_until("stopped", || f.c1.is_terminated());
    wait_until("log", || snapshot_of(&f.l1).len() == 12);
    assert_eq!(
        snapshot_of(&f.l1),
        owned_strings(&[
            "pre-start",
            "m1#1",
            "bres#2",
            "m2#3",
            "brst#4",
            "pre-restart brst",
            "post-stop",
            "post-restart",
            "pre-start",
            "m3#1",
            "bstp#2",
            "post-stop"
        ])
    );
}

// ---- maxNrOfRetries -------------------------------------------------------------------------------------------------------------

fn restarts_in(log: &Log) -> usize {
    snapshot_of(log)
        .iter()
        .filter(|e| e.starts_with("pre-restart"))
        .count()
}

#[test]
fn zero_retries_stop_the_child_at_its_first_failure() {
    // oracle: l01-max-zero-first-stops
    let f = fixture("s-max0", 0);
    say(&f.c1, &["brst"]);
    wait_until("stopped", || f.c1.is_terminated());
    assert_eq!(
        snapshot_of(&f.l1),
        owned_strings(&["pre-start", "brst#1", "post-stop"]),
        "stopped without any restart hook"
    );
}

#[test]
fn the_last_allowed_restart_runs_and_the_first_beyond_the_limit_stops() {
    // oracle: l02-max-one-second-stops, l03-max-two-third-stops, l04-max-three-fourth-stops
    for (max, expected_events) in [(1, 8), (2, 13), (3, 18)] {
        let f = fixture(&format!("s-max{max}"), max);
        say(&f.c1, &vec!["brst"; max as usize + 1]);
        wait_until("stopped", || f.c1.is_terminated());
        wait_until("events", || snapshot_of(&f.l1).len() == expected_events);
        assert_eq!(
            restarts_in(&f.l1),
            max as usize,
            "max {max}: exactly {max} restarts, then a stop"
        );
        assert_eq!(snapshot_of(&f.l1).last().unwrap(), "post-stop");
    }
}

#[test]
fn resume_and_stop_directives_do_not_consume_the_limit() {
    // oracle: l05-resume-does-not-count, l10-stop-and-resume-then-restart-ok
    let f = fixture("s-nocount", 1);
    say(&f.c1, &["bres", "bres", "bres", "brst", "ok"]);
    say(&f.c2, &["bstp"]);
    wait_until("ok", || {
        snapshot_of(&f.l1).iter().any(|e| e.starts_with("ok"))
    });
    assert!(
        !f.c1.is_terminated(),
        "three resumes then one restart is within a limit of 1"
    );
    assert_eq!(restarts_in(&f.l1), 1);
}

#[test]
fn the_limit_is_counted_per_child() {
    // oracle: l06-counter-per-child
    let f = fixture("s-perchild", 1);
    say(&f.c1, &["brst"]);
    wait_until("c1 restarted", || restarts_in(&f.l1) == 1);
    say(&f.c2, &["brst"]);
    wait_until("c2 restarted", || {
        restarts_in(&f.l2) == 1 && snapshot_of(&f.l2).len() == 6
    });
    say(&f.c1, &["brst"]);
    wait_until("c1 stopped", || f.c1.is_terminated());
    assert!(
        !f.c2.is_terminated(),
        "c1 exhausting its limit leaves c2 with its own"
    );
    say(&f.c2, &["brst"]);
    wait_until("c2 stopped", || f.c2.is_terminated());
}

#[test]
fn a_negative_limit_never_stops_the_child() {
    // oracle: l07-unlimited-minus-one, l08-negative-is-unlimited
    for max in [-1, -5] {
        let f = fixture(&format!("s-unl{}", -max), max);
        say(&f.c1, &["brst", "brst", "brst", "brst", "brst", "ok"]);
        wait_until("ok", || {
            snapshot_of(&f.l1).iter().any(|e| e.starts_with("ok"))
        });
        assert_eq!(restarts_in(&f.l1), 5);
        assert!(!f.c1.is_terminated());
    }
}

/// Records `level source class message` of the events published.
struct Collector(Log);

impl Actor for Collector {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let line = if let Some(e) = msg.downcast_ref::<ErrorEvent>() {
            format!("error {} {} {:?}", e.log_source, e.log_class, e.message)
        } else if let Some(w) = msg.downcast_ref::<WarningEvent>() {
            format!("warning {} {} {:?}", w.log_source, w.log_class, w.message)
        } else {
            return false;
        };
        self.0.lock().unwrap().push(line);
        true
    }
}

/// AC-SUP-018: `Resume` is logged as a `Warning`, `Restart` and `Stop` as an `Error` (`FaultHandling.scala:394-425`).
#[test]
fn each_directive_publishes_the_failed_child_at_its_level() {
    let f = fixture("s-log", -1);
    let events = Log::default();
    let log = events.clone();
    let collector = f.sys.actor_of(Props::new(move || Collector(log.clone())));
    for channel in [Channel::Warning, Channel::Error] {
        let collector = collector.as_ref().unwrap();
        f.sys.event_stream().subscribe(collector, channel);
    }
    say(&f.c1, &["bres", "brst", "bstp"]);
    wait_until("three events", || snapshot_of(&events).len() == 3);
    let of_c1 = "pekko://s-log/user/p/c1 org.apache.pekko.actor.OneForOneStrategy";
    let line = |level: &str, message: &str| format!("{level} {of_c1} Some({message:?})");
    assert_eq!(
        snapshot_of(&events),
        [
            line("warning", "bres"),
            line("error", "brst"),
            line("error", "bstp")
        ]
    );
}

/// Over one child `c`, which gets every message: a strategy answering `directive`, with `loggingEnabled` set to `logging`.
struct Decides {
    directive: Directive,
    logging: bool,
    log: Log,
}

impl Actor for Decides {
    fn pre_start(&mut self, c: &mut ActorContext) {
        let log = self.log.clone();
        let child = Props::new(move || Kid {
            log: log.clone(),
            seen: 0,
        });
        c.actor_of_named(child, "c").unwrap();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let directive = self.directive;
        let strategy = OneForOneStrategy::new(-1, move |_| directive);
        strategy.with_logging_enabled(self.logging).into()
    }
    fn receive(&mut self, c: &mut ActorContext, msg: &AnyMessage) -> bool {
        c.children()[0].tell_any(msg.clone(), None);
        true
    }
}

/// AC-SUP-018: nothing is published with `loggingEnabled` off, nor by the supervisor that escalates (`FaultHandling.scala:394-427`):
/// the only event is the one of the escalating supervisor's own supervisor, the guardian.
#[test]
fn disabled_logging_and_escalate_publish_nothing_for_the_child() {
    let system = RunningSystem::create("s-silent");
    let events = Log::default();
    let log = events.clone();
    let collector = system.actor_of(Props::new(move || Collector(log.clone())));
    for channel in [Channel::Warning, Channel::Error] {
        let collector = collector.as_ref().unwrap();
        system.event_stream().subscribe(collector, channel);
    }
    let parent = |name: &str, directive, logging| {
        let log = Log::default();
        let seen = log.clone();
        let props = Props::new(move || Decides {
            directive,
            logging,
            log: log.clone(),
        });
        (system.actor_of_named(props, name).unwrap(), seen)
    };
    let (quiet, seen) = parent("quiet", Directive::Restart, false);
    say(&quiet, &["brst", "after"]);
    wait_until("the restarted child", || {
        snapshot_of(&seen).iter().any(|e| e == "after#1")
    });
    let (escalates, _) = parent("esc", Directive::Escalate, true);
    say(&escalates, &["brst"]);
    wait_until("an event", || !snapshot_of(&events).is_empty());
    let guardian = "pekko://s-silent/user/esc org.apache.pekko.actor.OneForOneStrategy";
    assert_eq!(
        snapshot_of(&events),
        [format!("error {guardian} Some(\"brst\")")]
    );
}
