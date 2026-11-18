//! Slice 3b-3: `Escalate`. Three levels: `g` (grandparent) -> `p` (supervisor whose decider escalates) -> `c` (throws in `receive`).
//! Every expectation was recorded from Pekko 2.0.0 first (`cases/scenario-3b3.jsonl`, `rust/app-cases/`); the scenario name is in each test.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::dispatch::{Failed, Terminated};

type Log = Arc<Mutex<Vec<String>>>;
type Decide = Arc<dyn Fn(&Throwable) -> Directive + Send + Sync>;
type Slot = Arc<Mutex<Option<ActorRef>>>;

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

const ISE: &str = "decide java.lang.IllegalStateException: esc";

/// `c`: records `pre-start` / `post-stop` / each message; throws `IllegalStateException(msg)` on `esc` / `loc`, `ActorKilledException("Kill")` on `kill`.
struct Kid(Log);

impl Actor for Kid {
    fn pre_start(&mut self, _c: &mut ActorContext) {
        self.0.lock().unwrap().push("pre-start".into());
    }
    fn post_stop(&mut self, _c: &mut ActorContext) {
        self.0.lock().unwrap().push("post-stop".into());
    }
    fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(m) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.0.lock().unwrap().push(m.clone());
        match m.as_str() {
            "esc" | "loc" => throw(Throwable::illegal_state(m)),
            "kill" => throw(Throwable::new(
                "org.apache.pekko.actor.ActorKilledException",
                Some("Kill"),
            )),
            _ => true,
        }
    }
}

/// `p`: creates `c` (and `c2` when asked) once, escalates `esc`/`kill` and resumes `loc`; records what its decider saw and its restart hooks.
/// On `mk` it creates a child `d` (to fail again after a restart).
struct Middle {
    log: Log,
    logs: [Log; 3],
    slots: [Slot; 3],
    second: bool,
}

impl Actor for Middle {
    fn pre_start(&mut self, c: &mut ActorContext) {
        self.log.lock().unwrap().push("pre-start".into());
        for (i, name) in ["c", "c2"].into_iter().enumerate() {
            if (i == 0 || self.second) && self.slots[i].lock().unwrap().is_none() {
                let log = self.logs[i].clone();
                *self.slots[i].lock().unwrap() = Some(
                    c.actor_of_named(Props::new(move || Kid(log.clone())), name)
                        .unwrap(),
                );
            }
        }
    }
    fn pre_restart(&mut self, c: &mut ActorContext, reason: &Throwable, m: Option<&AnyMessage>) {
        let class = if m.is_some_and(|m| m.is::<Failed>()) {
            "Failed"
        } else {
            "other"
        };
        self.log
            .lock()
            .unwrap()
            .push(format!("pre-restart {reason} / {class}"));
        for k in c.children() {
            c.stop(&k);
        }
        self.post_stop(c);
    }
    fn post_restart(&mut self, c: &mut ActorContext, _r: &Throwable) {
        self.log.lock().unwrap().push("post-restart".into());
        self.pre_start(c);
    }
    fn post_stop(&mut self, _c: &mut ActorContext) {
        self.log.lock().unwrap().push("post-stop".into());
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let log = self.log.clone();
        OneForOneStrategy::new(-1, move |e: &Throwable| {
            let d = if e.message.as_deref() == Some("loc") {
                Directive::Resume
            } else {
                Directive::Escalate
            };
            log.lock().unwrap().push(format!("decide {e} -> {d:?}"));
            d
        })
        .into()
    }
    fn receive(&mut self, c: &mut ActorContext, m: &AnyMessage) -> bool {
        let Some(m) = m.downcast_ref::<String>() else {
            return false;
        };
        self.log.lock().unwrap().push(m.clone());
        if m == "mk" {
            let log = self.logs[2].clone();
            *self.slots[2].lock().unwrap() = Some(
                c.actor_of_named(Props::new(move || Kid(log.clone())), "d")
                    .unwrap(),
            );
        }
        true
    }
}

/// `g`: creates `p`; its decider records the cause it is given and applies `decide`; it records the `Terminated` of `p`.
struct Top {
    log: Log,
    decide: Decide,
    max: i32,
    mid_log: Log,
    logs: [Log; 3],
    slots: [Slot; 3],
    second: bool,
    mid: Slot,
}

impl Actor for Top {
    fn pre_start(&mut self, c: &mut ActorContext) {
        let (log, logs, slots, second) = (
            self.mid_log.clone(),
            self.logs.clone(),
            self.slots.clone(),
            self.second,
        );
        let p = c
            .actor_of_named(
                Props::new(move || Middle {
                    log: log.clone(),
                    logs: logs.clone(),
                    slots: slots.clone(),
                    second,
                }),
                "p",
            )
            .unwrap();
        c.watch(&p);
        *self.mid.lock().unwrap() = Some(p);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (log, decide) = (self.log.clone(), self.decide.clone());
        OneForOneStrategy::new(self.max, move |e: &Throwable| {
            let d = decide(e);
            log.lock().unwrap().push(format!("decide {e} -> {d:?}"));
            d
        })
        .into()
    }
    fn receive(&mut self, _c: &mut ActorContext, m: &AnyMessage) -> bool {
        if m.is::<Terminated>() {
            self.log.lock().unwrap().push("terminated p".into());
        }
        true
    }
}

struct Tree {
    _system: ActorSystem,
    g: Log,
    p: Log,
    c: Log,
    c2: Log,
    d: Log,
    mid: ActorRef,
    kid: ActorRef,
    slots: [Slot; 3],
}

fn tree_with(
    decide: impl Fn(&Throwable) -> Directive + Send + Sync + 'static,
    max: i32,
    second: bool,
) -> Tree {
    let system = ActorSystem::create("sys").unwrap();
    let (g, p) = (Log::default(), Log::default());
    let logs: [Log; 3] = Default::default();
    let slots: [Slot; 3] = Default::default();
    let mid: Slot = Default::default();
    let decide: Decide = Arc::new(decide);
    {
        let (g, p, logs, slots, mid) = (
            g.clone(),
            p.clone(),
            logs.clone(),
            slots.clone(),
            mid.clone(),
        );
        system
            .actor_of_named(
                Props::new(move || Top {
                    log: g.clone(),
                    decide: decide.clone(),
                    max,
                    mid_log: p.clone(),
                    logs: logs.clone(),
                    slots: slots.clone(),
                    second,
                    mid: mid.clone(),
                }),
                "g",
            )
            .unwrap();
    }
    wait_until("c started", || {
        snap(&logs[0]) == ["pre-start"] && (!second || snap(&logs[1]) == ["pre-start"])
    });
    let (mid, kid) = (
        mid.lock().unwrap().clone().unwrap(),
        slots[0].lock().unwrap().clone().unwrap(),
    );
    let [c, c2, d] = logs;
    Tree {
        _system: system,
        g,
        p,
        c,
        c2,
        d,
        mid,
        kid,
        slots,
    }
}

fn tree(decide: impl Fn(&Throwable) -> Directive + Send + Sync + 'static) -> Tree {
    tree_with(decide, -1, false)
}

fn tell(r: &ActorRef, m: &str) {
    r.tell(m.to_string(), None);
}

// ---- e01: the grandparent resumes the parent ----------------------------------------------------------------------------------------

/// e01-g-resumes-p
#[test]
fn escalated_failure_reaches_the_grandparent_which_resumes_the_parent() {
    let t = tree(|_| Directive::Resume);
    let (p_before, c_before) = (t.mid.clone(), t.kid.clone());
    for m in ["m1", "esc", "q-0", "q-1"] {
        tell(&t.kid, m);
    }
    wait_until("c processed the queued messages", || snap(&t.c).len() == 5);
    wait_until("g decided", || !snap(&t.g).is_empty());
    tell(&t.mid, "pm");
    wait_until("p processes messages again", || snap(&t.p).len() == 3);

    assert_eq!(
        snap(&t.g),
        strs(&["decide java.lang.IllegalStateException: esc -> Resume"]),
        "the grandparent's decider sees the child's exception"
    );
    assert_eq!(
        snap(&t.p),
        strs(&["pre-start", &format!("{ISE} -> Escalate"), "pm"]),
        "p: no restart hooks, no post-stop"
    );
    assert_eq!(
        snap(&t.c),
        strs(&["pre-start", "m1", "esc", "q-0", "q-1"]),
        "the failed message is lost, the queued ones are processed"
    );
    assert!(
        t.mid == p_before && t.kid == c_before,
        "p and c keep their identity (path and uid)"
    );
}

// ---- e02: the grandparent restarts the parent ---------------------------------------------------------------------------------------

/// e02-g-restarts-p
#[test]
fn grandparent_restart_replaces_the_parent_and_stops_its_child() {
    let t = tree(|_| Directive::Restart);
    let p_before = t.mid.clone();
    for m in ["m1", "esc", "q-0", "q-1"] {
        tell(&t.kid, m);
    }
    wait_until("p restarted", || snap(&t.p).len() == 6);
    tell(&t.mid, "pm");
    wait_until("the new instance receives", || snap(&t.p).len() == 7);

    assert_eq!(
        snap(&t.p),
        strs(&[
            "pre-start",
            &format!("{ISE} -> Escalate"),
            "pre-restart java.lang.IllegalStateException: esc / Failed", // the message preRestart sees is the escalated `Failed`
            "post-stop",
            "post-restart",
            "pre-start",
            "pm",
        ])
    );
    assert_eq!(
        snap(&t.c),
        strs(&["pre-start", "m1", "esc", "post-stop"]),
        "c is stopped by p's preRestart; the queued messages are dead letters"
    );
    assert!(
        t.mid == p_before,
        "the reference to p stays valid (no Terminated, same uid)"
    );
    assert_eq!(
        snap(&t.g),
        strs(&["decide java.lang.IllegalStateException: esc -> Restart"]),
        "no Terminated for a restart"
    );
}

// ---- e03: the grandparent stops the parent ------------------------------------------------------------------------------------------

/// e03-g-stops-p
#[test]
fn grandparent_stop_terminates_the_parent_and_its_child() {
    let t = tree(|_| Directive::Stop);
    for m in ["m1", "esc", "q-0", "q-1"] {
        tell(&t.kid, m);
    }
    wait_until("g saw p terminate", || snap(&t.g).len() == 2);
    wait_until("c stopped", || {
        snap(&t.c).last().map(String::as_str) == Some("post-stop")
    });
    assert_eq!(
        snap(&t.g),
        strs(&[
            "decide java.lang.IllegalStateException: esc -> Stop",
            "terminated p"
        ])
    );
    assert_eq!(
        snap(&t.p),
        strs(&["pre-start", &format!("{ISE} -> Escalate"), "post-stop"])
    );
    assert_eq!(snap(&t.c), strs(&["pre-start", "m1", "esc", "post-stop"]));
}

// ---- the cause the supervisor above sees is the child's own ---------------------------------------------------------------------------

/// e04-cause-is-kill-resume: the class and message survive the escalation untouched.
#[test]
fn the_escalated_cause_keeps_its_class_and_message() {
    let t = tree(|_| Directive::Resume);
    tell(&t.kid, "kill");
    wait_until("g decided", || !snap(&t.g).is_empty());
    assert_eq!(
        snap(&t.g),
        strs(&["decide org.apache.pekko.actor.ActorKilledException: Kill -> Resume"])
    );
    assert_eq!(
        snap(&t.p),
        strs(&[
            "pre-start",
            "decide org.apache.pekko.actor.ActorKilledException: Kill -> Escalate"
        ])
    );
}

/// e06 / e07: with no mapping the grandparent's default decider applies (IllegalStateException restarts, ActorKilledException stops).
#[test]
fn the_default_decider_of_the_grandparent_applies_to_the_escalated_cause() {
    let t = tree(rakka_actor::actor::supervision::default_decider);
    tell(&t.kid, "esc");
    wait_until("p restarted", || snap(&t.p).len() == 6);
    assert_eq!(
        snap(&t.g),
        strs(&["decide java.lang.IllegalStateException: esc -> Restart"])
    );

    let t = tree(rakka_actor::actor::supervision::default_decider);
    tell(&t.kid, "kill");
    wait_until("p stopped", || snap(&t.g).len() == 2);
    assert_eq!(
        snap(&t.g),
        strs(&[
            "decide org.apache.pekko.actor.ActorKilledException: Kill -> Stop",
            "terminated p"
        ])
    );
}

// ---- e08: only what the parent cannot handle reaches the grandparent -------------------------------------------------------------------

/// e08-local-and-escalated, e09-only-local-no-escalation
#[test]
fn the_grandparent_sees_only_the_failures_the_parent_escalates() {
    let t = tree(|_| Directive::Resume);
    for m in ["m1", "loc", "m2", "esc", "m3"] {
        tell(&t.kid, m);
    }
    wait_until("c done", || snap(&t.c).len() == 6);
    wait_until("g decided", || !snap(&t.g).is_empty());
    assert_eq!(snap(&t.g).len(), 1, "one escalated failure only");
    assert_eq!(
        snap(&t.p),
        strs(&[
            "pre-start",
            "decide java.lang.IllegalStateException: loc -> Resume",
            &format!("{ISE} -> Escalate")
        ])
    );

    let t = tree(|_| Directive::Restart);
    for m in ["m1", "loc", "m2"] {
        tell(&t.kid, m);
    }
    wait_until("c done", || snap(&t.c).len() == 4);
    assert!(
        snap(&t.g).is_empty(),
        "nothing escalated: the grandparent never decides"
    );
    assert_eq!(
        snap(&t.p),
        strs(&[
            "pre-start",
            "decide java.lang.IllegalStateException: loc -> Resume"
        ])
    );
}

/// e11-resume-twice / e10-escalate-ignores-p-max: every escalation reaches the grandparent; the parent's own retry limit is not involved.
#[test]
fn every_escalation_is_decided_again_by_the_grandparent() {
    let t = tree(|_| Directive::Resume);
    for m in ["esc", "m1", "esc", "m2"] {
        tell(&t.kid, m);
    }
    wait_until("c done", || snap(&t.c).len() == 5);
    wait_until("g decided twice", || snap(&t.g).len() == 2);
    assert_eq!(snap(&t.p).len(), 3);
}

// ---- f-*: the other children of the escalating parent ----------------------------------------------------------------------------------

/// f-resume-healthy-sibling: the healthy sibling is suspended with its parent and resumed with it; nothing is lost.
#[test]
fn a_healthy_sibling_is_resumed_with_its_parent() {
    let t = tree_with(|_| Directive::Resume, -1, true);
    let c2 = t.slots[1].lock().unwrap().clone().unwrap();
    tell(&c2, "a");
    wait_until("a", || snap(&t.c2).len() == 2);
    for m in ["m1", "esc", "q-0", "q-1"] {
        tell(&t.kid, m);
    }
    tell(&c2, "b");
    wait_until("c and c2 done", || {
        snap(&t.c).len() == 5 && snap(&t.c2).len() == 3
    });
    assert_eq!(snap(&t.c2), strs(&["pre-start", "a", "b"]));
}

/// f-restart-healthy-sibling / f-stop-healthy-sibling: the sibling dies with its parent.
#[test]
fn a_healthy_sibling_dies_with_its_restarted_or_stopped_parent() {
    for directive in [Directive::Restart, Directive::Stop] {
        let t = tree_with(move |_| directive, -1, true);
        let c2 = t.slots[1].lock().unwrap().clone().unwrap();
        tell(&c2, "a");
        wait_until("a", || snap(&t.c2).len() == 2);
        tell(&t.kid, "esc");
        wait_until("both children stopped", || {
            snap(&t.c).last().map(String::as_str) == Some("post-stop")
                && snap(&t.c2).last().map(String::as_str) == Some("post-stop")
        });
        assert_eq!(
            snap(&t.c2),
            strs(&["pre-start", "a", "post-stop"]),
            "{directive:?}"
        );
    }
}

// ---- e12, g01, g02: after a restart, and the grandparent's retry limit ------------------------------------------------------------------

/// e12-escalate-after-restart: the new instance of p escalates again; g02-g-max-one-second-stops: the grandparent's limit counts p's restarts.
#[test]
fn the_grandparents_retry_limit_counts_restarts_of_the_parent() {
    let t = tree_with(|_| Directive::Restart, 1, false);
    tell(&t.kid, "esc");
    wait_until("p restarted", || snap(&t.p).len() == 6);
    tell(&t.mid, "mk");
    wait_until("d created", || t.slots[2].lock().unwrap().is_some());
    let d = t.slots[2].lock().unwrap().clone().unwrap();
    wait_until("d started", || snap(&t.d) == ["pre-start"]);
    tell(&d, "esc");
    wait_until("p stopped by the limit", || snap(&t.g).len() == 3);
    assert_eq!(snap(&t.g)[2], "terminated p");
    assert_eq!(snap(&t.d), strs(&["pre-start", "esc", "post-stop"]));
}

/// g01-g-max-zero-stops-p
#[test]
fn a_zero_limit_at_the_grandparent_stops_the_parent_at_the_first_escalation() {
    let t = tree_with(|_| Directive::Restart, 0, false);
    tell(&t.kid, "esc");
    wait_until("p stopped", || snap(&t.g).len() == 2);
    assert_eq!(
        snap(&t.p),
        strs(&["pre-start", &format!("{ISE} -> Escalate"), "post-stop"]),
        "no restart hooks"
    );
}
