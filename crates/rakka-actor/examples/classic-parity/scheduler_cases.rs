// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala and actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala (Apache-2.0).
//! Cases of the one-shot scheduler oracle (`SchedulerOracle.java`). Each script drives the Rust scheduler exactly as the oracle
//! drives Pekko's; see there for why no observation depends on a race.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::scheduler::Cancellable;
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::actor::{ActorError, ActorRef};

/// Oracle cases Rust does not produce, with the reason.
#[allow(dead_code)] // read by tests/recorded_pekko.rs
pub const PENDING: &[(&str, &str)] = &[(
    "SCH-SO-04 a negative delay behaves like zero",
    "Q-SCH-5 (proposed): std::time::Duration cannot express a negative delay",
)];

const WAIT: Duration = Duration::from_secs(10);
/// `Int.MaxValue` ticks of 10 ms.
const LARGEST: Duration = Duration::from_millis(i32::MAX as u64 * 10);

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

fn text(message: &AnyMessage) -> &str {
    message.downcast_ref::<&str>().copied().unwrap_or("?")
}

struct Recorder(Mutex<Sender<String>>);

impl Actor for Recorder {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let line = format!("{} from {}", text(msg), ctx.sender().path().name());
        self.0.lock().unwrap().send(line).ok();
        true
    }
}

fn recorder(sys: &ActorSystem, name: &str) -> (ActorRef, Receiver<String>) {
    let (tx, rx) = channel();
    let props = Props::new(move || Recorder(Mutex::new(tx.clone())));
    (sys.actor_of_named(props, name).unwrap(), rx)
}

struct Env {
    sys: ActorSystem,
    target: ActorRef,
    asker: ActorRef,
    seen: Receiver<String>,
}

impl Env {
    fn new(name: &str) -> Env {
        let sys = ActorSystem::create(name).unwrap();
        let (target, seen) = recorder(&sys, "target");
        let (asker, _) = recorder(&sys, "asker");
        Env {
            sys,
            target,
            asker,
            seen,
        }
    }

    fn once(
        &self,
        delay: Duration,
        to: &ActorRef,
        message: &'static str,
        sender: Option<&ActorRef>,
    ) -> Result<Cancellable, ActorError> {
        self.sys
            .scheduler()
            .schedule_once(delay, to, message, sender)
    }

    fn next(&self) -> String {
        self.seen
            .recv_timeout(WAIT)
            .unwrap_or_else(|_| "nothing".into())
    }

    fn close(&self) {
        self.sys.terminate().wait(WAIT);
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        self.close();
    }
}

fn state(task: &Cancellable) -> String {
    let (first, after, second) = (task.cancel(), task.is_cancelled(), task.cancel());
    format!(
        "cancel={first} isCancelled={after} cancel={second} isCancelled={}",
        task.is_cancelled()
    )
}

fn attempt(env: &Env, delay: Duration) -> String {
    match env.once(delay, &env.target, "far", None) {
        Ok(task) => state(&task),
        Err(error) => {
            let class = error.class().java_name().rsplit('.').next().unwrap();
            format!("ERR {class} [{}]", error.message())
        }
    }
}

fn delivered_once() -> String {
    let env = Env::new("sch");
    let start = Instant::now();
    let task = env
        .once(ms(50), &env.target, "tick", Some(&env.asker))
        .unwrap();
    let first = env.next();
    let early = start.elapsed() < ms(50);
    env.once(ms(50), &env.target, "marker", None).unwrap();
    let after = state(&task);
    format!("{first} | early={early} | {after} | next={}", env.next())
}

fn cancelled_before() -> String {
    let env = Env::new("sch");
    let task = env.once(ms(1000), &env.target, "cancelled", None).unwrap();
    let before = format!("isCancelled={} {}", task.is_cancelled(), state(&task));
    env.once(ms(1500), &env.target, "marker", None).unwrap();
    format!("{before} | next={}", env.next())
}

fn zero_delay() -> String {
    let env = Env::new("sch");
    let task = env.once(Duration::ZERO, &env.target, "now", None).unwrap();
    format!("{} | {}", env.next(), state(&task))
}

fn largest_delay() -> String {
    let env = Env::new("sch");
    let largest = attempt(&env, LARGEST);
    let beyond = attempt(&env, LARGEST + Duration::from_nanos(1));
    format!("largest => {largest} | largest+1ns => {beyond}")
}

type Case = (&'static str, fn() -> String);

const CASES: &[Case] = &[
    (
        "SCH-SO-01 the message arrives once, not before the delay, with the given sender; it cannot be cancelled afterwards",
        delivered_once,
    ),
    (
        "SCH-SO-02 a message cancelled before the delay is never sent; only the first cancel succeeds",
        cancelled_before,
    ),
    (
        "SCH-SO-03 a delay of zero sends at once and gives a task that is not cancellable",
        zero_delay,
    ),
    (
        "SCH-SO-05 the largest delay is Int.MaxValue ticks; the delay is rounded up to the tick before the check",
        largest_delay,
    ),
];

/// The lines of the cases `wanted` selects (each case takes up to a few seconds).
pub fn rust_lines_where(wanted: impl Fn(&str) -> bool) -> Vec<(String, String)> {
    CASES
        .iter()
        .filter(|(id, _)| wanted(id))
        .map(|(id, case)| (id.to_string(), format!("{id} -> {}", case())))
        .collect()
}

pub fn rust_ids() -> Vec<(String, String)> {
    CASES
        .iter()
        .map(|(id, _)| (id.to_string(), String::new()))
        .collect()
}
