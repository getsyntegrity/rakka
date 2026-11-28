// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:133-165, actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorMiscSpec.scala:101-141 and actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:465-477 (Apache-2.0).
//! Child names (AC-REF-009.2) and child lookup (AC-REF-010).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, owned_strings, snapshot_of, wait_until};

use std::sync::{Arc, Barrier, Mutex};

use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::{ActorError, ActorRef, ErrorClass};
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};
use rakka_actor::dispatch::queue_semantics::ProducedQueue;
use rakka_actor::event::{Channel, ErrorEvent};

type Log = Arc<Mutex<Vec<String>>>;
type Behaviour = Arc<dyn Fn(&mut ActorContext, &AnyMessage) + Send + Sync>;

type OnStart = Arc<dyn Fn(&mut ActorContext) + Send + Sync>;

/// An actor that does what the test says, on start and on each message.
struct Script {
    start: OnStart,
    receive: Behaviour,
}

impl Actor for Script {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        (self.start)(ctx);
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        (self.receive)(ctx, msg);
        true
    }
}

fn script(start: OnStart, receive: Behaviour) -> Props {
    Props::new(move || Script {
        start: start.clone(),
        receive: receive.clone(),
    })
}

fn on_message(receive: impl Fn(&mut ActorContext, &AnyMessage) + Send + Sync + 'static) -> Props {
    script(Arc::new(|_| {}), Arc::new(receive))
}

fn idle() -> Props {
    on_message(|_, _| {})
}

fn not_unique(error: &ActorError, name: &str) {
    assert_eq!(error.class(), ErrorClass::InvalidActorName);
    assert_eq!(
        error.message(),
        format!("actor name [{name}] is not unique!")
    );
}

#[test]
fn only_one_of_four_concurrent_creations_with_a_name_succeeds() {
    let system = RunningSystem::create("same-name-concurrently");
    for round in 0..100 {
        let name = format!("new-actor{round}");
        let together = Barrier::new(4);
        let outcomes: Vec<Result<ActorRef, ActorError>> = std::thread::scope(|scope| {
            let attempts: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        together.wait();
                        system.actor_of_named(idle(), &name)
                    })
                })
                .collect();
            attempts.into_iter().map(|a| a.join().unwrap()).collect()
        });
        let refused: Vec<&ActorError> = outcomes.iter().filter_map(|o| o.as_ref().err()).collect();
        assert_eq!(refused.len(), 3, "round {round}");
        refused.iter().for_each(|error| not_unique(error, &name));
    }
}

#[test]
fn a_second_creation_in_one_message_fails_and_is_logged_once() {
    let system = RunningSystem::create("same-name-in-one-message");
    let errors = Log::default();
    let seen = errors.clone();
    let collector = system
        .actor_of(on_message(move |_, msg| {
            if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
                seen.lock().unwrap().push(error.cause.to_string());
            }
        }))
        .unwrap();
    system.event_stream().subscribe(&collector, Channel::Error);
    let supervisor = system
        .actor_of(on_message(|ctx, _| {
            for _ in 0..2 {
                if let Err(error) = ctx.actor_of_named(idle(), "duplicate") {
                    let class = "org.apache.pekko.actor.InvalidActorNameException";
                    throw(Throwable::new(class, Some(error.message())));
                }
            }
        }))
        .unwrap();
    supervisor.tell("", None);
    wait_until("the failure to be logged", || {
        !snapshot_of(&errors).is_empty()
    });
    common::settle_for_absence();
    assert_eq!(
        snapshot_of(&errors),
        ["org.apache.pekko.actor.InvalidActorNameException: actor name [duplicate] is not unique!"]
    );
}

#[test]
fn a_child_can_be_recreated_once_the_old_one_is_terminated() {
    let system = RunningSystem::create("recreate-after-terminated");
    let result = Log::default();
    let kid: Arc<Mutex<Option<ActorRef>>> = Arc::default();
    let (watched, remembered, seen) = (kid.clone(), kid.clone(), result.clone());
    let parent = script(
        Arc::new(move |ctx| {
            let child = ctx.actor_of_named(idle(), "foo").unwrap();
            ctx.watch(&child);
            *watched.lock().unwrap() = Some(child);
        }),
        Arc::new(move |ctx, msg| {
            let old = remembered.lock().unwrap().clone().expect("a kid");
            if msg.downcast_ref::<Terminated>().is_some() {
                let new = ctx.actor_of_named(idle(), "foo").unwrap();
                let verdict = if new == old {
                    "Failure: context.actorOf returned the same instance!"
                } else if !old.is_terminated() {
                    "Kid is zombie"
                } else if new.is_terminated() {
                    "newKid was stillborn"
                } else if old.path() != new.path() {
                    "The kids do not share the same path"
                } else {
                    "green"
                };
                seen.lock().unwrap().push(verdict.to_owned());
            } else {
                ctx.stop(&old);
            }
        }),
    );
    system.actor_of(parent).unwrap().tell("engage", None);
    wait_until("the verdict", || !snapshot_of(&result).is_empty());
    assert_eq!(snapshot_of(&result), ["green"]);
}

/// The name stays taken in the turn that stops the child, also once the child is already terminated: only the parent's
/// processing of its termination releases it. The stopping child is still among `children`.
#[test]
fn a_child_cannot_be_recreated_while_its_termination_is_not_processed() {
    let system = RunningSystem::create("recreate-while-stopping");
    let result = Log::default();
    let seen = result.clone();
    let parent = on_message(move |ctx, _| {
        let kid = ctx.actor_of_named(idle(), "foo").unwrap();
        ctx.stop(&kid);
        not_unique(&ctx.actor_of_named(idle(), "foo").unwrap_err(), "foo");
        wait_until("the kid to be terminated", || kid.is_terminated());
        not_unique(&ctx.actor_of_named(idle(), "foo").unwrap_err(), "foo");
        assert!(ctx.child("foo") == Some(kid.clone()));
        assert!(ctx.children() == [kid]);
        seen.lock().unwrap().push("green".to_owned());
    });
    system.actor_of(parent).unwrap().tell("engage", None);
    wait_until("the verdict", || !snapshot_of(&result).is_empty());
}

/// Holds the thread that creates an actor at the point where its mailbox type is asked what it produces: after the name is
/// reserved, before the actor exists.
struct HeldMailbox(Arc<Gate>);

impl MailboxType for HeldMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        UnboundedMailbox.create()
    }
    fn produced_queue(&self) -> ProducedQueue {
        self.0.hold_until_released();
        UnboundedMailbox.produced_queue()
    }
}

#[test]
fn a_name_is_taken_while_its_actor_is_still_being_created() {
    let system = RunningSystem::create("reserved-name");
    let gate: Arc<Gate> = Arc::default();
    std::thread::scope(|scope| {
        let held = idle().with_mailbox(HeldMailbox(gate.clone()));
        let creation = scope.spawn(|| system.actor_of_named(held, "kid"));
        wait_until("the creation to be held", || gate.entered());
        let second = system.actor_of_named(idle(), "kid");
        gate.release(); // before asserting: a failed assertion must not leave the first creation blocked
        not_unique(&second.unwrap_err(), "kid");
        assert!(creation.join().unwrap().is_ok());
    });
}

#[test]
fn an_actor_can_check_for_the_existence_of_children() {
    let system = RunningSystem::create("existence-of-children");
    let found = Log::default();
    let seen = found.clone();
    let parent = script(
        Arc::new(|ctx| {
            ctx.actor_of_named(idle(), "child").unwrap();
        }),
        Arc::new(move |ctx, msg| {
            let name = msg.downcast_ref::<&str>().expect("a name");
            let defined = ctx.child(name).is_some();
            seen.lock().unwrap().push(format!("{name}={defined}"));
        }),
    );
    let parent = system.actor_of_named(parent, "parent").unwrap();
    parent.tell("child", None);
    parent.tell("whatnot", None);
    wait_until("both answers", || snapshot_of(&found).len() == 2);
    assert_eq!(snapshot_of(&found), ["child=true", "whatnot=false"]);
}

/// `ChildrenContainer` is a `TreeMap` by name: `children` is in name order, whatever the order of creation.
#[test]
fn children_are_listed_in_name_order() {
    let system = RunningSystem::create("children-order");
    let listed = Log::default();
    let seen = listed.clone();
    let parent = on_message(move |ctx, _| {
        for name in ["b", "a", "10", "9", "a-b", "B"] {
            ctx.actor_of_named(idle(), name).unwrap();
        }
        ctx.actor_of(idle()).unwrap();
        let names = ctx.children();
        let mut listed = seen.lock().unwrap();
        listed.extend(names.iter().map(|c| c.path().name().to_owned()));
    });
    system.actor_of(parent).unwrap().tell("list", None);
    wait_until("the list", || !snapshot_of(&listed).is_empty());
    assert_eq!(
        snapshot_of(&listed),
        owned_strings(&["$a", "10", "9", "B", "a", "a-b", "b"])
    );
}
