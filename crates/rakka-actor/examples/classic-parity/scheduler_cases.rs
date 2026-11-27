// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala and actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala (Apache-2.0).
//! Cases of the scheduler oracle (`SchedulerOracle.java`). Each script drives the Rust scheduler exactly as the oracle
//! drives Pekko's; see there for why no observation depends on a race.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::scheduler::Cancellable;
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::actor::{ActorError, ActorRef};
use rakka_actor::event::{Channel, DeadLetter};

/// Oracle cases Rust does not produce, with the reason.
#[allow(dead_code)] // read by tests/recorded_pekko.rs
pub const PENDING: &[(&str, &str)] = &[
    (
        "SCH-SO-04 a negative delay behaves like zero",
        "Q-SCH-5 (proposed): std::time::Duration cannot express a negative delay",
    ),
    (
        "SCH-FD-05 a negative delay is rejected like zero",
        "Q-SCH-5 (proposed): std::time::Duration cannot express a negative delay",
    ),
];

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
        let line = match msg.downcast_ref::<DeadLetter>() {
            Some(dead) => format!(
                "dead {} sender={} recipient={}",
                text(&dead.message),
                dead.sender.path().name(),
                dead.recipient.path().name()
            ),
            None => format!("{} from {}", text(msg), ctx.sender().path().name()),
        };
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

    fn repeat(
        &self,
        initial_delay: Duration,
        delay: Duration,
        to: &ActorRef,
        message: &'static str,
        sender: Option<&ActorRef>,
    ) -> Result<Cancellable, ActorError> {
        self.sys
            .scheduler()
            .schedule_with_fixed_delay(initial_delay, delay, to, message, sender)
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
    outcome(env.once(delay, &env.target, "far", None))
}

fn attempt_repeat(env: &Env, initial_delay: Duration, delay: Duration) -> String {
    outcome(env.repeat(initial_delay, delay, &env.target, "far", None))
}

fn outcome(scheduled: Result<Cancellable, ActorError>) -> String {
    match scheduled {
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

fn stopped_target() -> String {
    let env = Env::new("sch");
    let (listener, letters) = recorder(&env.sys, "listener");
    env.sys
        .event_stream()
        .subscribe(&listener, Channel::DeadLetter);
    env.once(ms(1000), &env.target, "late", Some(&env.asker))
        .unwrap();
    env.sys.stop(&env.target);
    if !await_terminated(&env.target) {
        return "target did not stop".into();
    }
    letters.recv_timeout(WAIT).unwrap_or_else(|_| "null".into())
}

fn termination() -> String {
    let other = Env::new("other");
    let closing = Env::new("closing");
    let pending = closing
        .once(ms(1000), &other.target, "pending", None)
        .unwrap();
    closing.close();
    other.once(ms(1500), &other.target, "marker", None).unwrap();
    let next = other.next();
    format!(
        "next={next} | pending: {} | zero => {} | 1s => {} | too far => {}",
        state(&pending),
        attempt(&closing, Duration::ZERO),
        attempt(&closing, ms(1000)),
        attempt(&closing, LARGEST + Duration::from_nanos(1))
    )
}

fn await_terminated(actor: &ActorRef) -> bool {
    let deadline = Instant::now() + ms(5000);
    while !actor.is_terminated() {
        if Instant::now() > deadline {
            return false;
        }
        std::thread::sleep(ms(2));
    }
    true
}

fn repeats_until_cancelled() -> String {
    let env = Env::new("sch");
    let start = Instant::now();
    let task = env
        .repeat(ms(50), ms(600), &env.target, "tick", Some(&env.asker))
        .unwrap();
    let seen = format!("{} | {} | {}", env.next(), env.next(), env.next());
    let early = start.elapsed() < ms(1250);
    let after = state(&task);
    env.once(ms(900), &env.target, "marker", None).unwrap();
    format!("{seen} | early={early} | {after} | next={}", env.next())
}

fn repetition_cancelled_before() -> String {
    let env = Env::new("sch");
    let task = env
        .repeat(ms(1000), ms(100), &env.target, "cancelled", None)
        .unwrap();
    let before = format!("isCancelled={} {}", task.is_cancelled(), state(&task));
    env.once(ms(1500), &env.target, "marker", None).unwrap();
    format!("{before} | next={}", env.next())
}

fn repetition_from_zero() -> String {
    let env = Env::new("sch");
    let task = env
        .repeat(Duration::ZERO, ms(1000), &env.target, "now", None)
        .unwrap();
    let (first, after) = (env.next(), state(&task));
    env.once(ms(1500), &env.target, "marker", None).unwrap();
    format!("{first} | {after} | next={}", env.next())
}

fn repetition_validation() -> String {
    let env = Env::new("sch");
    let beyond = LARGEST + Duration::from_nanos(1);
    let report = format!(
        "zero => {} | zero, initial too far => {} | delay too far => {} | initial too far => {} | largest => {}",
        attempt_repeat(&env, Duration::ZERO, Duration::ZERO),
        attempt_repeat(&env, beyond, Duration::ZERO),
        attempt_repeat(&env, Duration::ZERO, beyond),
        attempt_repeat(&env, beyond, ms(1000)),
        attempt_repeat(&env, LARGEST, LARGEST)
    );
    env.once(Duration::ZERO, &env.target, "marker", None)
        .unwrap();
    format!("{report} | next={}", env.next())
}

/// A listener of dead letters.
fn dead_letter_listener(env: &Env) -> (ActorRef, Receiver<String>) {
    let (listener, letters) = recorder(&env.sys, "listener");
    env.sys
        .event_stream()
        .subscribe(&listener, Channel::DeadLetter);
    (listener, letters)
}

/// The first dead letter, what the listener gets next and the state of the repetition.
fn ended(
    env: &Env,
    listener: &ActorRef,
    letters: &Receiver<String>,
    task: &Cancellable,
    margin: Duration,
) -> String {
    let next = || letters.recv_timeout(WAIT).unwrap_or_else(|_| "null".into());
    let letter = next();
    env.once(margin, listener, "marker", None).unwrap();
    let after = next();
    format!(
        "{letter} | next={after} | isCancelled={} {}",
        task.is_cancelled(),
        state(task)
    )
}

fn repetition_to_a_target_that_stops() -> String {
    let env = Env::new("sch");
    let (listener, letters) = dead_letter_listener(&env);
    let task = env
        .repeat(
            Duration::ZERO,
            ms(600),
            &env.target,
            "late",
            Some(&env.asker),
        )
        .unwrap();
    let first = env.next();
    env.sys.stop(&env.target);
    if !await_terminated(&env.target) {
        return "target did not stop".into();
    }
    format!(
        "{first} | {}",
        ended(&env, &listener, &letters, &task, ms(1500))
    )
}

fn repetition_to_dead_letters() -> String {
    let env = Env::new("sch");
    let (listener, letters) = dead_letter_listener(&env);
    let dead = env.sys.dead_letters();
    let terminated = dead.is_terminated();
    let task = env
        .repeat(ms(100), ms(100), &dead, "gone", Some(&env.asker))
        .unwrap();
    format!(
        "isTerminated={terminated} | {}",
        ended(&env, &listener, &letters, &task, ms(600))
    )
}

fn repetition_at_termination() -> String {
    let other = Env::new("other");
    let closing = Env::new("closing");
    let pending = closing
        .repeat(ms(1000), ms(1000), &other.target, "pending", None)
        .unwrap();
    closing.close();
    other.once(ms(1500), &other.target, "marker", None).unwrap();
    let next = other.next();
    format!(
        "next={next} | pending: isCancelled={} {} | initial zero => {} | initial 1s => {} | initial too far => {} | delay zero => {}",
        pending.is_cancelled(),
        state(&pending),
        attempt_repeat(&closing, Duration::ZERO, ms(1000)),
        attempt_repeat(&closing, ms(1000), ms(1000)),
        attempt_repeat(&closing, LARGEST + Duration::from_nanos(1), ms(1000)),
        attempt_repeat(&closing, ms(1000), Duration::ZERO)
    )
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
    (
        "SCH-SO-06 a target that stopped before the delay elapsed gets nothing: the message is a dead letter",
        stopped_target,
    ),
    (
        "SCH-TE-01 termination discards what is pending without sending it, and the scheduler then rejects every delay",
        termination,
    ),
    (
        "SCH-FD-01 the message repeats with the given sender, never before the initial delay and the delays; cancel stops it",
        repeats_until_cancelled,
    ),
    (
        "SCH-FD-02 a repetition cancelled during the initial delay sends nothing",
        repetition_cancelled_before,
    ),
    (
        "SCH-FD-03 an initial delay of zero sends at once, and the repetition can still be cancelled",
        repetition_from_zero,
    ),
    (
        "SCH-FD-04 the delay must be positive and within the reach of the wheel, and is checked before the initial delay",
        repetition_validation,
    ),
    (
        "SCH-FD-06 a target that stops between two sends gets one more, a dead letter, and the repetition ends uncancelled",
        repetition_to_a_target_that_stops,
    ),
    (
        "SCH-FD-07 deadLetters counts as terminated: it gets the message once and the repetition ends",
        repetition_to_dead_letters,
    ),
    (
        "SCH-FD-08 termination discards a pending repetition, and the scheduler then rejects new ones",
        repetition_at_termination,
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
