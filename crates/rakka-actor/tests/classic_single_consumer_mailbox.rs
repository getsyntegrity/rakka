// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/dispatch/MailboxConfigSpec.scala:279-329 (Apache-2.0).
//! `SingleConsumerOnlyMailboxVerificationSpec`: two actors ping-pong 2,000,000 messages with `throughput = 1`, which makes the
//! mailbox be scheduled once per message, and both stop. Ported for the unbounded and the bounded single consumer mailbox.
//!
//! The spec's `within(30.seconds)` measures throughput, which here depends on the machine's load (about one cross-thread
//! hand-off per message, `docs/evidence/mailbox-single-consumer.md`). So the default tests assert the behaviour, and fail when
//! the ping pong stops making progress; the 30 s are asserted by the `#[ignore]`d measurements, run on a quiet machine with
//! `cargo test -p rakka-actor --test classic_single_consumer_mailbox -- --ignored --test-threads=1 --nocapture`.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::Settings;
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use rakka_actor::dispatch::single_consumer_mailbox::SingleConsumerOnlyUnboundedMailbox;

const TOTAL: usize = 2_000_000;

/// How often the delivery count is sampled; the test fails when a sample did not grow, so a hang is reported about one to two
/// periods after the last delivery. There is no upper bound on the whole run.
const NO_PROGRESS_LIMIT: Duration = Duration::from_secs(10);

struct Ping;

/// Answers every ping to its sender and stops after `TOTAL / 2` of them.
struct Player {
    remaining: usize,
    delivered: Arc<AtomicUsize>,
}

impl Actor for Player {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if !msg.is::<Ping>() {
            return false;
        }
        self.remaining -= 1;
        self.delivered.fetch_add(1, Ordering::Relaxed);
        ctx.sender().tell(Ping, Some(&ctx.self_ref()));
        if self.remaining == 0 {
            ctx.stop(&ctx.self_ref());
        }
        true
    }
}

/// Creates the two players and serves the first ping; stops once both have terminated, and reports its own stop.
struct Runner {
    players: Props,
    refs: Vec<ActorRef>,
    terminated: usize,
    stopped: Sender<()>,
}

impl Actor for Runner {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.is::<Ping>() && self.refs.is_empty() {
            for _ in 0..2 {
                let player = ctx.actor_of(self.players.clone()).expect("a player");
                ctx.watch(&player);
                self.refs.push(player);
            }
            self.refs[0].tell(Ping, Some(&self.refs[1]));
        } else if msg.is::<Terminated>() {
            self.terminated += 1;
            if self.terminated == 2 {
                ctx.stop(&ctx.self_ref());
            }
        }
        true
    }

    /// `expectTerminated(runner)` of the spec: the runner itself ends.
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        let _ = self.stopped.send(());
    }
}

/// Runs the spec's scenario until the runner stops, checks that every ping was delivered and returns how long it took.
fn pathological_ping_pong(name: &str, select_mailbox: impl Fn(Props) -> Props) -> Duration {
    let settings = Settings {
        throughput: 1,
        ..Settings::default()
    };
    let system = RunningSystem::create_with_settings(name, settings);
    let delivered = Arc::new(AtomicUsize::new(0));
    let players = select_mailbox(Props::new({
        let delivered = delivered.clone();
        move || Player {
            remaining: TOTAL / 2,
            delivered: delivered.clone(),
        }
    }));
    let (stopped, runner_stopped) = channel();
    let runner_props = Props::new(move || Runner {
        players: players.clone(),
        refs: Vec::new(),
        terminated: 0,
        stopped: stopped.clone(),
    });
    let runner = system.actor_of_named(runner_props, "runner").unwrap();
    let started = Instant::now();
    runner.tell(Ping, None);
    let mut seen = 0;
    while runner_stopped.recv_timeout(NO_PROGRESS_LIMIT).is_err() {
        let now = delivered.load(Ordering::Relaxed);
        assert!(now > seen, "ping pong hung after {now} of {TOTAL} pings");
        seen = now;
    }
    let elapsed = started.elapsed();
    assert_eq!(delivered.load(Ordering::Relaxed), TOTAL);
    eprintln!("{name}: {TOTAL} pings in {elapsed:?}");
    elapsed
}

fn unbounded_ping_pong(name: &str) -> Duration {
    pathological_ping_pong(name, |props| {
        props.with_mailbox(SingleConsumerOnlyUnboundedMailbox)
    })
}

fn bounded_ping_pong(name: &str) -> Duration {
    let bounded = NonBlockingBoundedMailbox::new(1).expect("a capacity of one");
    pathological_ping_pong(name, move |props| props.with_mailbox(bounded))
}

#[test]
fn pathological_ping_pong_for_the_unbounded_single_consumer_mailbox() {
    unbounded_ping_pong("mbxsc-unbounded");
}

#[test]
fn pathological_ping_pong_for_the_bounded_single_consumer_mailbox() {
    bounded_ping_pong("mbxsc-bounded");
}

/// `within(30.seconds)` of the spec.
const SPEC_TIME_LIMIT: Duration = Duration::from_secs(30);

#[test]
#[ignore = "performance measurement: depends on the machine's load, run it explicitly"]
fn throughput_of_the_unbounded_ping_pong_is_within_the_thirty_seconds_of_the_spec() {
    let elapsed = unbounded_ping_pong("mbxsc-unbounded-timed");
    assert!(elapsed < SPEC_TIME_LIMIT, "took {elapsed:?}");
}

#[test]
#[ignore = "performance measurement: depends on the machine's load, run it explicitly"]
fn throughput_of_the_bounded_ping_pong_is_within_the_thirty_seconds_of_the_spec() {
    let elapsed = bounded_ping_pong("mbxsc-bounded-timed");
    assert!(elapsed < SPEC_TIME_LIMIT, "took {elapsed:?}");
}
