// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala (Apache-2.0).
//! The dispatcher's own pool: what any change of its queue has to keep. No actors: tasks only.

use crate::dispatch::dispatcher::{Executor, Rejected, RunQueue, ThreadPool};
use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(10);
const GONE: Result<(), RecvTimeoutError> = Err(RecvTimeoutError::Disconnected);

/// For the queue alone, where a `true` asks to notify a sleeping worker.
fn queue_with_sleepers(sleepers: usize) -> RunQueue<u32> {
    let mut queue = RunQueue::new(sleepers);
    (0..sleepers).for_each(|_| queue.sleep());
    queue
}

#[test]
fn no_second_worker_is_notified_while_one_is_waking() {
    assert_eq!(queue_with_sleepers(0).push(1), Ok(false));
    assert_eq!(queue_with_sleepers(1).push(1), Ok(true));
    let mut queue = queue_with_sleepers(3);
    assert_eq!(queue.push(1), Ok(true));
    assert_eq!(queue.push(2), Ok(false));
    assert_eq!(queue.push(3), Ok(false));
}

#[test]
fn the_woken_worker_notifies_the_next_one_while_jobs_remain() {
    let mut queue = queue_with_sleepers(3);
    let _ = (queue.push(1), queue.push(2));
    queue.wake_up();
    assert_eq!(queue.pop(), Some((1, true)));
    queue.wake_up();
    assert_eq!(queue.pop(), Some((2, false)));
}

#[test]
fn a_closed_queue_hands_a_new_job_back_and_still_gives_the_queued_ones_in_order() {
    let mut queue = queue_with_sleepers(0);
    let _ = (queue.push(1), queue.push(2));
    queue.closed = true;
    assert_eq!(queue.push(3), Err(3));
    assert_eq!(queue.pop(), Some((1, false)));
    assert_eq!(queue.pop(), Some((2, false)));
    assert_eq!(queue.pop(), None);
    assert!(queue.closed);
}

/// Were every thread to die, nothing may wait for ever in the queue.
#[test]
fn losing_the_last_worker_closes_the_queue_and_gives_back_its_jobs() {
    let mut queue = RunQueue::new(2);
    assert_eq!(queue.push(1), Ok(false));
    assert!(queue.lose_worker().is_empty());
    assert!(!queue.closed);
    assert_eq!(queue.lose_worker(), [1]);
    assert_eq!(queue.push(2), Err(2));
}

/// Submits a task that makes its worker hold `alive` until the thread exits.
fn hold_until_the_worker_exits(pool: &ThreadPool, alive: Sender<()>) {
    thread_local!(static HELD: std::cell::RefCell<Option<Sender<()>>> = const { std::cell::RefCell::new(None) });
    pool.run(Box::new(move || HELD.set(Some(alive)))).unwrap();
}

#[test]
fn shutdown_runs_what_is_queued_rejects_later_tasks_and_ends_the_worker() {
    let pool = ThreadPool::start("drains", 1);
    let (alive, exited) = channel();
    let (release, released) = channel::<()>();
    let (ran, runs) = channel();
    hold_until_the_worker_exits(&pool, alive);
    pool.run(Box::new(move || released.recv().unwrap()))
        .unwrap();
    for i in 0..3 {
        let ran = ran.clone();
        pool.run(Box::new(move || ran.send(i).unwrap())).unwrap();
    }
    pool.shutdown();
    assert_eq!(pool.run(Box::new(|| ())), Err(Rejected));
    release.send(()).unwrap();
    let order: Vec<i32> = (0..3).map(|_| runs.recv_timeout(LIMIT).unwrap()).collect();
    assert_eq!(order, [0, 1, 2]);
    assert_eq!(exited.recv_timeout(LIMIT), GONE);
}

#[test]
fn dropping_the_pool_ends_its_worker() {
    let pool = ThreadPool::start("dropped", 1);
    let (alive, exited) = channel();
    hold_until_the_worker_exits(&pool, alive);
    drop(pool);
    assert_eq!(exited.recv_timeout(LIMIT), GONE);
}

/// A task that queues another one and blocks until it ran: no worker may keep for itself what it submits.
#[test]
fn a_task_queued_by_a_blocked_worker_runs_on_another_one() {
    let pool = Arc::new(ThreadPool::start("blocked", 2));
    for _ in 0..200 {
        let (done, finished) = channel();
        let inner = pool.clone();
        pool.run(Box::new(move || {
            let (ran, awaited) = channel();
            inner.run(Box::new(move || ran.send(()).unwrap())).unwrap();
            done.send(awaited.recv_timeout(LIMIT).is_ok()).unwrap();
        }))
        .unwrap();
        assert_eq!(finished.recv_timeout(LIMIT), Ok(true));
    }
}

/// Bursts from several producers, with the workers going to sleep in between: no wake-up may be lost.
#[test]
fn every_task_of_many_producers_runs() {
    let pool = Arc::new(ThreadPool::start("stress", 8));
    for round in 0..500 {
        let (ran, runs) = channel();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..=round % 5 {
                        let ran = ran.clone();
                        pool.run(Box::new(move || ran.send(()).unwrap())).unwrap();
                    }
                });
            }
        });
        for _ in 0..4 * (round % 5 + 1) {
            runs.recv_timeout(LIMIT).expect("a task never ran");
        }
    }
}

#[test]
fn a_panicking_task_leaves_its_worker_running() {
    let pool = ThreadPool::start("panics", 1);
    let (ran, runs) = channel();
    pool.run(Box::new(|| panic!("a task of the user"))).unwrap();
    pool.run(Box::new(move || ran.send(()).unwrap())).unwrap();
    assert_eq!(runs.recv_timeout(LIMIT), Ok(()));
}
