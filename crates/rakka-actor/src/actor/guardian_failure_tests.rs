// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:322-381 and 485-529 (Apache-2.0).
//! A failed guardian. The root guardian stops it, which ends the system, and the guardians keep their children if they are ever restarted.
//! The expected lines are the ones the Pekko 2.0.0 JVM printed (`MBX-GRD-02..04` of `cases/oracle-only/mailbox.golden`).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::actor::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::fault::Throwable;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use crate::actor::system::ActorSystem;
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::{Channel, ErrorEvent};

const ORACLE_GOLDEN: &str = include_str!("../../../../cases/oracle-only/mailbox.golden");
const WAIT: Duration = Duration::from_secs(10);

fn recorded(case: &str) -> &'static str {
    ORACLE_GOLDEN
        .lines()
        .find(|line| line.starts_with(case))
        .and_then(|line| line.split_once(" -> "))
        .map(|(_, observed)| observed)
        .expect("the golden has the case")
}

fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + WAIT;
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Every `Error` published, as `logSource|logClass|cause class|message`. It is not an actor, so no suspended or stopped guardian hides
/// an error (the oracle uses a `FunctionRef` for the same reason).
pub(crate) struct ErrorRecorder {
    path: ActorPath,
    pub(crate) errors: Mutex<Vec<String>>,
}

impl InternalActorRef for ErrorRecorder {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
        if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
            self.errors.lock().unwrap().push(format!(
                "{}|{}|{}|{}",
                error.log_source,
                error.log_class,
                error.cause.class,
                error.message.as_deref().unwrap_or("null")
            ));
        }
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub(crate) struct Seen {
    started: AtomicUsize,
    restarted: AtomicUsize,
    pub(crate) stopped: Mutex<Vec<String>>,
}

/// The oracle's `Tree`: records its life; with a child name it creates that child when it starts.
struct Tree {
    seen: Arc<Seen>,
    child: Option<&'static str>,
}

impl Actor for Tree {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.seen.started.fetch_add(1, Ordering::SeqCst);
        if let Some(name) = self.child.filter(|name| ctx.child(name).is_none()) {
            ctx.actor_of_named(tree(&self.seen, None), name).unwrap();
        }
    }
    fn post_restart(&mut self, _ctx: &mut ActorContext, _reason: &Throwable) {
        self.seen.restarted.fetch_add(1, Ordering::SeqCst);
    }
    fn post_stop(&mut self, ctx: &mut ActorContext) {
        let name = ctx.self_ref().path().name().to_owned();
        self.seen.stopped.lock().unwrap().push(name);
    }
}

pub(crate) fn tree(seen: &Arc<Seen>, child: Option<&'static str>) -> Props {
    let seen = seen.clone();
    Props::new(move || Tree {
        seen: seen.clone(),
        child,
    })
}

pub(crate) fn record_errors(system: &ActorSystem) -> Arc<ErrorRecorder> {
    let path = ActorPath::from_string("pekko://mbx/system/recorder").unwrap();
    let recorder = Arc::new(ErrorRecorder {
        path,
        errors: Mutex::default(),
    });
    system
        .event_stream()
        .subscribe(&ActorRef(recorder.clone()), Channel::Error);
    recorder
}

/// `/user/a` with its child `c`, both started.
pub(crate) fn tree_under_user(system: &ActorSystem) -> Arc<Seen> {
    let seen = Arc::new(Seen::default());
    system.actor_of_named(tree(&seen, Some("c")), "a").unwrap();
    wait_until("the two actors to start", || {
        seen.started.load(Ordering::SeqCst) == 2
    });
    seen
}

/// `Kill` to the guardian at `path`, with `/user/a` and its child alive, until the system terminates.
fn line_after_killing(path: &str) -> String {
    let system = ActorSystem::create("mbx").unwrap();
    let recorder = record_errors(&system);
    let seen = tree_under_user(&system);
    system.actor_selection(path).tell(Kill, None);
    let terminated = system
        .when_terminated()
        .wait(WAIT)
        .expect("ended")
        .expect("terminated normally");
    format!(
        "errors=[{}] stopped=[{}] terminated={} existenceConfirmed={}",
        recorder.errors.lock().unwrap().join(", "),
        seen.stopped.lock().unwrap().join(", "),
        terminated.actor.path(),
        terminated.existence_confirmed
    )
}

#[test]
fn a_killed_user_guardian_is_logged_and_stopped_by_the_root_and_the_system_terminates() {
    assert_eq!(line_after_killing("/user"), recorded("MBX-GRD-02"));
}

#[test]
fn a_killed_system_guardian_is_logged_and_stopped_by_the_root_and_the_system_terminates() {
    assert_eq!(line_after_killing("/system"), recorded("MBX-GRD-04"));
}

#[test]
fn a_killed_root_guardian_is_logged_by_the_bubble_walker_and_the_system_never_reports_termination()
{
    let system = ActorSystem::create("mbx").unwrap();
    let recorder = record_errors(&system);
    let seen = tree_under_user(&system);
    system.actor_selection("/").tell(Kill, None);
    wait_until("the tree to stop", || {
        seen.stopped.lock().unwrap().len() == 2
    });
    std::thread::sleep(Duration::from_millis(100)); // a bounded wait for an absence, as the oracle does
    let observed = format!(
        "errors=[{}] stopped=[{}] whenTerminated={}",
        recorder.errors.lock().unwrap().join(", "),
        seen.stopped.lock().unwrap().join(", "),
        system.when_terminated().is_completed()
    );
    // the system never terminates by itself, so the test closes the dispatchers
    system
        .0
        .walker
        .get()
        .expect("a walker")
        .shut_down_dispatchers();
    assert_eq!(observed, recorded("MBX-GRD-03"));
}

/// `Guardian.preRestart` and `SystemGuardian.preRestart` are empty ("guardian MUST NOT lose its children during restart"). The root
/// never restarts them, so the `Recreate` is sent directly.
#[test]
fn a_restarted_user_or_system_guardian_keeps_its_children() {
    let system = ActorSystem::create("keeps").unwrap();
    for guardian in [&system.0.guardian, &system.0.system_guardian] {
        let guardian = guardian.get().expect("initialised");
        let seen = Arc::new(Seen::default());
        let child = guardian
            .attach_child(tree(&seen, None), Some("child"), true)
            .unwrap();
        wait_until("the child to start", || {
            seen.started.load(Ordering::SeqCst) == 1
        });
        guardian.self_ref().send_system(SystemMessage::Recreate {
            cause: Throwable::illegal_state("restart"),
        });
        wait_until("the child to be restarted or stopped", || {
            seen.restarted.load(Ordering::SeqCst) == 1 || child.is_terminated()
        });
        assert!(
            !child.is_terminated(),
            "{} lost its child",
            guardian.self_ref().path()
        );
    }
    system.terminate().wait(WAIT).expect("ended").expect("ok");
}
