// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/dispatch/MailboxConfigSpec.scala:279-329 (Apache-2.0).
//! `SingleConsumerOnlyMailboxVerificationSpec`: two actors ping-pong 2,000,000 messages with `throughput = 1`, which makes the
//! mailbox be scheduled once per message, and both stop. Ported for the unbounded and the bounded single consumer mailbox.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, snapshot_of};

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::Settings;
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use rakka_actor::dispatch::single_consumer_mailbox::SingleConsumerOnlyUnboundedMailbox;

const TOTAL: usize = 2_000_000;

struct Ping;

/// Answers every ping to its sender and stops after `TOTAL / 2` of them.
struct Player {
    remaining: usize,
}

impl Actor for Player {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if !msg.is::<Ping>() {
            return false;
        }
        self.remaining -= 1;
        ctx.sender().tell(Ping, Some(&ctx.self_ref()));
        if self.remaining == 0 {
            ctx.stop(&ctx.self_ref());
        }
        true
    }
}

/// Creates the two players and serves the first ping; stops once both have terminated, and records its own stop.
struct Runner {
    players: Props,
    refs: Vec<ActorRef>,
    terminated: usize,
    done: Arc<Mutex<Vec<()>>>,
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
        self.done.lock().unwrap().push(());
    }
}

fn pathological_ping_pong(name: &str, select_mailbox: impl Fn(Props) -> Props) {
    let settings = Settings {
        throughput: 1,
        ..Settings::default()
    };
    let system = RunningSystem::create_with_settings(name, settings);
    let players = select_mailbox(Props::new(|| Player {
        remaining: TOTAL / 2,
    }));
    let done = Arc::new(Mutex::new(Vec::new()));
    let runner_props = Props::new({
        let done = done.clone();
        move || Runner {
            players: players.clone(),
            refs: Vec::new(),
            terminated: 0,
            done: done.clone(),
        }
    });
    let runner = system.actor_of_named(runner_props, "runner").unwrap();
    let started = Instant::now();
    runner.tell(Ping, None);
    // `within(30.seconds)` of the spec
    while snapshot_of(&done).is_empty() {
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "ping pong did not finish"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn pathological_ping_pong_for_the_unbounded_single_consumer_mailbox() {
    pathological_ping_pong("mbxsc-unbounded", |props| {
        props.with_mailbox(SingleConsumerOnlyUnboundedMailbox)
    });
}

#[test]
fn pathological_ping_pong_for_the_bounded_single_consumer_mailbox() {
    let bounded = NonBlockingBoundedMailbox::new(1).expect("a capacity of one");
    pathological_ping_pong("mbxsc-bounded", move |props| props.with_mailbox(bounded));
}
