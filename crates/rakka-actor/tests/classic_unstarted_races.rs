// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala (Apache-2.0).
//! Twin of oracle case `MBX-UT-08` through the public API: two threads send to an actor made by `actorOf` while its real cell is being
//! installed; nothing is lost or repeated and each sender's order is kept. (The race itself is probabilistic: this checks the invariant,
//! as the oracle case does.) Expected values are the oracle's golden line: `delivered=400 distinct=400 perSenderOrdered=true preStarts=1`.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, wait_until};

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};

const GOLDEN: &str = include_str!("../../../cases/oracle-only/mailbox.golden");
const MESSAGES_PER_SENDER: usize = 200;

struct Counts {
    received: Arc<Mutex<Vec<String>>>,
    pre_starts: Arc<AtomicUsize>,
}
impl Actor for Counts {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.pre_starts.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.received.lock().unwrap().push(text.clone());
        true
    }
}

#[test]
fn senders_racing_with_the_installation_of_the_real_cell_lose_nothing_repeat_nothing_and_keep_their_order()
 {
    let system = RunningSystem::create("t-unstarted-races");
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let pre_starts = Arc::new(AtomicUsize::new(0));
    let actor = system
        .actor_of_named(
            Props::new({
                let (received, pre_starts) = (received.clone(), pre_starts.clone());
                move || Counts {
                    received: received.clone(),
                    pre_starts: pre_starts.clone(),
                }
            }),
            "target",
        )
        .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let senders: Vec<_> = ["a", "b"]
        .into_iter()
        .map(|who| {
            let (actor, barrier) = (actor.clone(), barrier.clone());
            std::thread::spawn(move || {
                barrier.wait();
                for index in 0..MESSAGES_PER_SENDER {
                    actor.tell(format!("{who}:{index}"), None);
                }
            })
        })
        .collect();
    barrier.wait();
    for sender in senders {
        sender.join().unwrap();
    }
    wait_until("400 messages", || received.lock().unwrap().len() == 400);
    let received = received.lock().unwrap().clone();
    let mut next_index = HashMap::<String, usize>::new();
    let mut per_sender_ordered = true;
    for message in &received {
        let (who, index) = message.split_once(':').unwrap();
        let expected = next_index.entry(who.to_owned()).or_default();
        per_sender_ordered &= *expected == index.parse::<usize>().unwrap();
        *expected += 1;
    }
    let observed = format!(
        "delivered={} distinct={} perSenderOrdered={per_sender_ordered} preStarts={}",
        received.len(),
        received.iter().collect::<HashSet<_>>().len(),
        pre_starts.load(Ordering::SeqCst)
    );
    assert!(
        GOLDEN
            .lines()
            .find(|line| line.starts_with("MBX-UT-08"))
            .is_some_and(|line| line.contains(&format!("-> {observed} deadLetters=[]"))),
        "{observed}"
    );
}
