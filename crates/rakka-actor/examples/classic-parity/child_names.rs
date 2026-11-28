// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala and actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:133-153 (Apache-2.0).
//! `spawn-held`, `release-held` and `spawn-concurrently` (AC-REF-009.2), as `pilot.ChildNames` does them in the oracle.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::json::write_text;
use rakka_actor::actor::props::{Actor, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::actor::{ActorError, ActorRef};
use rakka_actor::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};
use rakka_actor::dispatch::queue_semantics::ProducedQueue;

const HELD_MAILBOX: &str = "held-mailbox";

struct Idle;
impl Actor for Idle {}

type Creation = JoinHandle<Result<ActorRef, ActorError>>;

/// The one creation `spawn-held` keeps between reserving its name and making its actor.
#[derive(Default)]
pub struct Held {
    entered: AtomicBool,
    released: AtomicBool,
    creation: Mutex<Option<Creation>>,
}

fn wait_for(flag: &AtomicBool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !flag.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    flag.load(Ordering::SeqCst)
}

/// An unbounded mailbox type whose first user waits for `release-held` while it is asked what it produces.
struct HeldMailbox(Arc<Held>);

impl MailboxType for HeldMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        UnboundedMailbox.create()
    }
    fn produced_queue(&self) -> ProducedQueue {
        if !self.0.entered.swap(true, Ordering::SeqCst) {
            wait_for(&self.0.released);
        }
        UnboundedMailbox.produced_queue()
    }
}

pub fn register(settings: &mut Settings, held: &Arc<Held>) {
    settings
        .mailboxes
        .register(HELD_MAILBOX, HeldMailbox(held.clone()));
}

/// `system.actorOf(props, name)` on another thread; returns once it is held, with the name reserved.
pub fn spawn_held(system: &ActorSystem, held: &Held, name: &str) -> bool {
    let (system, name) = (system.clone(), name.to_owned());
    let props = Props::new(|| Idle).with_mailbox_id(HELD_MAILBOX);
    let creation = std::thread::spawn(move || system.actor_of_named(props, &name));
    *held.creation.lock().unwrap() = Some(creation);
    wait_for(&held.entered)
}

pub fn release_held(held: &Held) -> Result<ActorRef, ActorError> {
    held.released.store(true, Ordering::SeqCst);
    let creation = held.creation.lock().unwrap().take();
    creation.expect("a held creation").join().expect("no panic")
}

/// What `threads` simultaneous `system.actorOf(props, name_of(round))` did, over all rounds.
pub struct Outcomes {
    pub winners: usize,
    pub one_winner_per_name: bool,
    /// `<class>: <message>` of the refusals, the name written `<n>`, with how many there were
    pub refused: BTreeMap<String, usize>,
    /// at least two of the calls were in progress at the same time
    pub overlapped: bool,
}

impl Outcomes {
    /// The result fields as the oracle prints them. Whether the calls overlapped is only certain over many rounds, so only those
    /// report it (`overlap`).
    pub fn json_fields(&self, rounds: usize, overlap: bool) -> String {
        let refused: Vec<String> = (self.refused.iter())
            .map(|(refusal, count)| {
                let mut entry = String::new();
                write_text(&mut entry, refusal);
                format!("{entry}:{count}")
            })
            .collect();
        let mut fields = format!(
            ",\"ok\":true,\"rounds\":{rounds},\"winners\":{},\"oneWinnerPerName\":{},\"refused\":{{{}}}",
            self.winners,
            self.one_winner_per_name,
            refused.join(",")
        );
        if overlap {
            fields.push_str(&format!(",\"overlapped\":{}", self.overlapped));
        }
        fields
    }
}

pub fn spawn_concurrently(
    system: &ActorSystem,
    name_of: impl Fn(usize) -> String,
    threads: usize,
    rounds: usize,
) -> Outcomes {
    let mut outcomes = Outcomes {
        winners: 0,
        one_winner_per_name: true,
        refused: BTreeMap::new(),
        overlapped: false,
    };
    let (running, most) = (AtomicUsize::new(0), AtomicUsize::new(0));
    for round in 0..rounds {
        let name = name_of(round);
        let together = Barrier::new(threads);
        let results: Vec<Result<ActorRef, ActorError>> = std::thread::scope(|scope| {
            let attempt = || {
                together.wait();
                most.fetch_max(running.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
                let result = system.actor_of_named(Props::new(|| Idle), &name);
                running.fetch_sub(1, Ordering::SeqCst);
                result
            };
            let attempts: Vec<_> = (0..threads).map(|_| scope.spawn(attempt)).collect();
            let joined = attempts.into_iter().map(|a| a.join().expect("no panic"));
            joined.collect()
        });
        let mut winners = 0;
        for result in results {
            match result {
                Ok(_) => winners += 1,
                Err(error) => {
                    let message = error.message().replace(&name, "<n>");
                    let refusal = format!("{}: {message}", error.class().java_name());
                    *outcomes.refused.entry(refusal).or_default() += 1;
                }
            }
        }
        outcomes.winners += winners;
        outcomes.one_winner_per_name &= winners == 1;
    }
    outcomes.overlapped = most.load(Ordering::SeqCst) > 1;
    outcomes
}
