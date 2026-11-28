// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala (Apache-2.0).
//! The actors of `SupervisorHierarchySpec` "survive being stressed" (AC-SUP-010): a random tree of 500 actors that is pinged and then
//! stopped from the leaves up (see `driver`). The random failures are not here yet. Integration: real actors and dispatcher.

mod driver;

use crate::actor::actor_ref::ActorRef;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use driver::{Event, Random};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

const BREADTH: usize = 6;

struct World {
    random: Mutex<Random>,
    events: Mutex<Sender<Event>>,
}

impl World {
    fn send(&self, event: Event) {
        let _ = self.events.lock().unwrap().send(event);
    }
}

struct PingOfDeath;
struct PongOfDeath;

struct Hierarchy {
    world: Arc<World>,
    size: usize,
    pongs_to_go: usize,
    reply_to: Option<ActorRef>,
}

fn hierarchy(world: &Arc<World>, size: usize) -> Props {
    let world = world.clone();
    Props::new(move || Hierarchy {
        world: world.clone(),
        size,
        pongs_to_go: 0,
        reply_to: None,
    })
}

impl Hierarchy {
    fn spawn(&self, ctx: &mut ActorContext, name: &str, size: usize) {
        let props = hierarchy(&self.world, size);
        let child = ctx.actor_of_named(props, name).unwrap();
        ctx.watch(&child);
    }

    fn stop_and_report(&self, ctx: &mut ActorContext) {
        ctx.stop(&ctx.self_ref());
        self.reply_to.as_ref().unwrap().tell(PongOfDeath, None);
    }
}

impl Actor for Hierarchy {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.world.send(Event::Ready(ctx.self_ref()));
        let rest = self.size - 1;
        if rest > 0 {
            let most = BREADTH.min(rest);
            let kids = self.world.random.lock().unwrap().below(most) + 1;
            for id in 1..=kids {
                let size = rest / kids + usize::from(id <= rest % kids);
                self.spawn(ctx, &id.to_string(), size);
            }
        }
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<&str>() == Some(&"ping") {
            self.world.send(Event::Pong(ctx.self_ref()));
        } else if msg.is::<PingOfDeath>() {
            self.reply_to = Some(ctx.sender());
            if self.size > 1 {
                let children = ctx.children();
                self.pongs_to_go = children.len();
                for child in &children {
                    child.tell(PingOfDeath, Some(&ctx.self_ref()));
                }
            } else {
                self.stop_and_report(ctx);
            }
        } else if msg.is::<PongOfDeath>() {
            self.pongs_to_go -= 1;
            if self.pongs_to_go == 0 {
                self.stop_and_report(ctx);
            }
        } else {
            // the `Terminated` of a child that stops during the `PingOfDeath`
            return msg.is::<crate::dispatch::Terminated>();
        }
        true
    }
}
