// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/LockUtil.scala (Apache-2.0).
//! `Switch` (AC-UTIL-015.1): an atomic on/off switch whose changes and `while*` reads run under a lock.
//!
//! `ReentrantGuard`, the other class of `LockUtil.scala` in 1.7.1, no longer exists in 2.0.0
//! (AC-UTIL-015.2): `std::sync::Mutex` is its replacement.
//!
//! Pekko synchronises on the switch itself, which is re-entrant: a thread holding the lock through
//! [`Switch::locked`] may call the switch again. [`Switch`] keeps that with a small re-entrant monitor
//! (Q-UTIL-10 (a)). An action that panics reverts the switch and the panic goes on (Throwable); with
//! `panic = "abort"` there is nothing to revert (Q-X-5, documented).

use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

/// A re-entrant lock: the monitor that `synchronized` takes in Scala.
#[derive(Default)]
struct Monitor {
    state: Mutex<MonitorState>,
    released: Condvar,
    /// Threads parked waiting for the monitor (a test seam: no sleeping to know a thread is blocked).
    waiting: AtomicUsize,
}

#[derive(Default)]
struct MonitorState {
    owner: Option<ThreadId>,
    depth: usize,
}

struct Held<'a>(&'a Monitor);

impl Monitor {
    fn state(&self) -> MutexGuard<'_, MonitorState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn enter(&self) -> Held<'_> {
        let me = thread::current().id();
        let mut state = self.state();
        while state.owner.is_some_and(|owner| owner != me) {
            self.waiting.fetch_add(1, Ordering::SeqCst);
            state = self
                .released
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
            self.waiting.fetch_sub(1, Ordering::SeqCst);
        }
        state.owner = Some(me);
        state.depth += 1;
        Held(self)
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.depth -= 1;
        if state.depth == 0 {
            state.owner = None;
            self.0.released.notify_one();
        }
    }
}

/// An atomic switch that can be either on or off.
pub struct Switch {
    switch: AtomicBool,
    monitor: Monitor,
}

impl Default for Switch {
    /// `new Switch()`: off.
    fn default() -> Self {
        Self::new(false)
    }
}

impl Switch {
    /// `new Switch(startAsOn)`.
    pub fn new(start_as_on: bool) -> Self {
        Self {
            switch: AtomicBool::new(start_as_on),
            monitor: Monitor::default(),
        }
    }

    fn compare_and_set(&self, from: bool, to: bool) -> bool {
        self.switch
            .compare_exchange(from, to, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Under the lock, switches from `from` to `!from` and runs `action`; reverts and re-raises if
    /// the action panics.
    fn transcend(&self, from: bool, action: impl FnOnce()) -> bool {
        let _held = self.monitor.enter();
        if !self.compare_and_set(from, !from) {
            return false;
        }
        if let Err(panic_payload) = panic::catch_unwind(AssertUnwindSafe(action)) {
            self.compare_and_set(!from, from); // revert status
            panic::resume_unwind(panic_payload);
        }
        true
    }

    /// `switchOff(action)`: if the switch is on, switches it off and runs `action`, under the lock; if
    /// `action` panics the switch goes back on. Returns whether the switch changed.
    pub fn switch_off_with(&self, action: impl FnOnce()) -> bool {
        self.transcend(true, action)
    }

    /// `switchOn(action)`: if the switch is off, switches it on and runs `action`, under the lock; if
    /// `action` panics the switch goes back off. Returns whether the switch changed.
    pub fn switch_on_with(&self, action: impl FnOnce()) -> bool {
        self.transcend(false, action)
    }

    /// `switchOff`: switches the switch off (if on), using the lock.
    pub fn switch_off(&self) -> bool {
        let _held = self.monitor.enter();
        self.compare_and_set(true, false)
    }

    /// `switchOn`: switches the switch on (if off), using the lock.
    pub fn switch_on(&self) -> bool {
        let _held = self.monitor.enter();
        self.compare_and_set(false, true)
    }

    /// `ifOnYield(action)`: runs `action` and returns its value if the switch is IMMEDIATELY on (no lock).
    pub fn if_on_yield<T>(&self, action: impl FnOnce() -> T) -> Option<T> {
        self.is_on().then(action)
    }

    /// `ifOffYield(action)`: as [`Switch::if_on_yield`] when the switch is off.
    pub fn if_off_yield<T>(&self, action: impl FnOnce() -> T) -> Option<T> {
        self.is_off().then(action)
    }

    /// `ifOn(action)`: runs `action` if the switch is IMMEDIATELY on (no lock); whether it ran.
    pub fn if_on(&self, action: impl FnOnce()) -> bool {
        self.if_on_yield(action).is_some()
    }

    /// `ifOff(action)`: runs `action` if the switch is IMMEDIATELY off (no lock); whether it ran.
    pub fn if_off(&self, action: impl FnOnce()) -> bool {
        self.if_off_yield(action).is_some()
    }

    /// `whileOnYield(action)`: runs `action` and returns its value if the switch is on, after any
    /// pending change (locking). Long or blocking actions can deadlock or slow other threads.
    pub fn while_on_yield<T>(&self, action: impl FnOnce() -> T) -> Option<T> {
        let _held = self.monitor.enter();
        self.is_on().then(action)
    }

    /// `whileOffYield(action)`: as [`Switch::while_on_yield`] when the switch is off.
    pub fn while_off_yield<T>(&self, action: impl FnOnce() -> T) -> Option<T> {
        let _held = self.monitor.enter();
        self.is_off().then(action)
    }

    /// `whileOn(action)`: whether `action` ran, the switch being on, under the lock.
    pub fn while_on(&self, action: impl FnOnce()) -> bool {
        self.while_on_yield(action).is_some()
    }

    /// `whileOff(action)`: whether `action` ran, the switch being off, under the lock.
    pub fn while_off(&self, action: impl FnOnce()) -> bool {
        self.while_off_yield(action).is_some()
    }

    /// `fold(on)(off)`: one callback or the other depending on the state, under the lock.
    pub fn fold<T>(&self, on: impl FnOnce() -> T, off: impl FnOnce() -> T) -> T {
        let _held = self.monitor.enter();
        if self.is_on() { on() } else { off() }
    }

    /// `locked(code)`: runs `code` holding the switch's lock, protected from concurrent changes of the
    /// status. The lock is re-entrant.
    pub fn locked<T>(&self, code: impl FnOnce() -> T) -> T {
        let _held = self.monitor.enter();
        code()
    }

    /// `isOn`: whether the switch is IMMEDIATELY on (no locking).
    pub fn is_on(&self) -> bool {
        self.switch.load(Ordering::SeqCst)
    }

    /// `isOff`.
    pub fn is_off(&self) -> bool {
        !self.is_on()
    }
}

#[cfg(test)]
mod tests {
    //! `SwitchSpec` of Pekko 2.0.0 (actor-tests/.../util/SwitchSpec.scala).
    use super::*;
    use std::sync::Arc;
    use std::sync::mpsc;

    #[test]
    fn on_and_off() {
        let s = Switch::new(false);
        assert!(s.is_off() && !s.is_on());

        assert!(s.switch_on_with(|| {}));
        assert!(s.is_on() && !s.is_off());
        assert!(!s.switch_on_with(|| {}));
        assert!(s.is_on() && !s.is_off());

        assert!(s.switch_off_with(|| {}));
        assert!(s.is_off() && !s.is_on());
        assert!(!s.switch_off_with(|| {}));
        assert!(s.is_off() && !s.is_on());
    }

    #[test]
    fn revert_when_exception() {
        let s = Switch::new(false);
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            s.switch_on_with(|| panic!("boom"));
        }));
        assert!(outcome.is_err());
        assert!(s.is_off());
        s.switch_on();
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            s.switch_off_with(|| panic!("boom"));
        }));
        assert!(outcome.is_err());
        assert!(s.is_on());
    }

    #[test]
    fn run_action_without_locking() {
        let s = Switch::new(false);
        assert_eq!(s.if_off_yield(|| "yes"), Some("yes"));
        assert_eq!(s.if_on_yield(|| "no"), None);
        assert!(s.if_off(|| {}));
        assert!(!s.if_on(|| {}));

        s.switch_on_with(|| {});
        assert_eq!(s.if_on_yield(|| "yes"), Some("yes"));
        assert_eq!(s.if_off_yield(|| "no"), None);
        assert!(s.if_on(|| {}));
        assert!(!s.if_off(|| {}));
    }

    #[test]
    fn run_action_with_locking() {
        let s = Switch::new(false);
        assert_eq!(s.while_off_yield(|| "yes"), Some("yes"));
        assert_eq!(s.while_on_yield(|| "no"), None);
        assert!(s.while_off(|| {}));
        assert!(!s.while_on(|| {}));

        s.switch_on_with(|| {});
        assert_eq!(s.while_on_yield(|| "yes"), Some("yes"));
        assert_eq!(s.while_off_yield(|| "no"), None);
        assert!(s.while_on(|| {}));
        assert!(!s.while_off(|| {}));
    }

    #[test]
    fn run_first_or_second_action_depending_on_state() {
        let s = Switch::new(false);
        assert_eq!(s.fold(|| "on", || "off"), "off");
        s.switch_on_with(|| {});
        assert_eq!(s.fold(|| "on", || "off"), "on");
    }

    #[test]
    fn do_proper_locking() {
        let s = Arc::new(Switch::new(false));
        let (finished, wait_for_finish) = mpsc::channel();
        let during_lock = s.locked(|| {
            s.switch_on(); // re-entrant: the thread holding the lock may use the switch
            assert!(s.is_on());
            let other = s.clone();
            thread::spawn(move || {
                other.switch_off();
                finished.send(()).expect("the test is listening");
            });
            // The other thread parks on the monitor: no sleeping, wait until it is blocked.
            while s.monitor.waiting.load(Ordering::SeqCst) == 0 {
                thread::yield_now();
            }
            (s.is_on(), wait_for_finish.try_recv().is_ok())
        });
        assert_eq!(during_lock, (true, false), "switchOff must wait for locked");
        wait_for_finish
            .recv()
            .expect("switchOff finished after locked");
        assert!(s.is_off());
    }

    #[test]
    fn only_one_of_several_racing_threads_switches_on() {
        let s = Arc::new(Switch::new(false));
        let go = Arc::new(std::sync::Barrier::new(8));
        let winners: usize = (0..8)
            .map(|_| {
                let (s, go) = (s.clone(), go.clone());
                thread::spawn(move || {
                    go.wait();
                    s.switch_on()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|t| usize::from(t.join().expect("no panic")))
            .sum();
        assert_eq!(winners, 1);
        assert!(s.is_on());
    }

    #[test]
    fn locking_calls_nest_and_default_is_off() {
        let s = Switch::default();
        assert!(s.is_off());
        let inner = s.locked(|| s.locked(|| (s.switch_on(), s.while_on(|| {}), s.is_on())));
        assert_eq!(inner, (true, true, true));
        assert!(s.switch_off());
        assert!(!s.switch_off());
    }
}
