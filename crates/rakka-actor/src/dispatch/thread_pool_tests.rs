// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala (Apache-2.0).
//! The dispatcher's own pool: what any change of its queue has to keep. No actors: tasks only.

use crate::dispatch::dispatcher::{Executor, Rejected, RunQueue, ThreadPool};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(10);
const GONE: Result<(), RecvTimeoutError> = Err(RecvTimeoutError::Disconnected);

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
    let mut queue = RunQueue::new(2); // losing the last worker closes it and gives back its jobs
    assert_eq!(queue.push(1), Ok(false));
    assert!(queue.lose_worker().is_empty());
    assert!(!queue.closed);
    assert_eq!(queue.lose_worker(), [1]);
    assert_eq!(queue.push(2), Err(2));
}

thread_local!(static HELD: std::cell::RefCell<Option<Sender<()>>> = const { std::cell::RefCell::new(None) });

fn submit(pool: &ThreadPool, task: impl FnOnce() + Send + 'static) {
    pool.run(Box::new(task)).unwrap();
}

/// A pool whose workers each hold a clone of `alive` until they exit, and are all asleep.
fn sleeping_pool(workers: usize, alive: Sender<()>) -> ThreadPool {
    let pool = ThreadPool::start("sleepers", workers);
    let all_running = Arc::new(Barrier::new(workers + 1));
    for _ in 0..workers {
        let (all_running, alive) = (all_running.clone(), alive.clone());
        submit(&pool, move || {
            HELD.set(Some(alive));
            all_running.wait();
        });
    }
    all_running.wait();
    let deadline = Instant::now() + LIMIT;
    while pool.sleeping_workers() < workers {
        assert!(Instant::now() < deadline, "the workers never slept");
        std::thread::yield_now();
    }
    pool
}

#[test]
fn shutdown_runs_what_is_queued_rejects_later_tasks_and_ends_the_worker() {
    let pool = ThreadPool::start("drains", 1);
    let (alive, exited) = channel();
    let (release, released) = channel::<()>();
    let (ran, runs) = channel();
    submit(&pool, move || HELD.set(Some(alive)));
    submit(&pool, move || released.recv().unwrap());
    for i in 0..3 {
        let ran = ran.clone();
        submit(&pool, move || ran.send(i).unwrap());
    }
    pool.shutdown();
    assert_eq!(pool.run(Box::new(|| ())), Err(Rejected));
    release.send(()).unwrap();
    let order: Vec<i32> = (0..3).map(|_| runs.recv_timeout(LIMIT).unwrap()).collect();
    assert_eq!(order, [0, 1, 2]);
    assert_eq!(exited.recv_timeout(LIMIT), GONE);
}

#[test]
fn drop_and_shutdown_wake_every_sleeping_worker() {
    let (alive, exited) = channel();
    drop(sleeping_pool(3, alive));
    assert_eq!(exited.recv_timeout(LIMIT), GONE);
    let (alive, exited) = channel();
    let pool = sleeping_pool(3, alive);
    pool.shutdown();
    assert_eq!(exited.recv_timeout(LIMIT), GONE);
}

/// Only a panic that escapes the guard around a task ends a worker: here, the `Drop` of what the task panicked with.
#[test]
fn a_panicking_task_leaves_its_worker_and_a_pool_that_loses_it_rejects() {
    struct PanicsOnDrop;
    impl Drop for PanicsOnDrop {
        fn drop(&mut self) {
            panic!("the panic payload of a user");
        }
    }
    let pool = ThreadPool::start("lost", 1);
    let (release, released) = channel::<()>();
    let (queued, never_ran) = channel();
    let (ran, runs) = channel();
    submit(&pool, || panic!("a task of the user"));
    submit(&pool, move || ran.send(()).unwrap());
    assert_eq!(runs.recv_timeout(LIMIT), Ok(()));
    submit(&pool, move || {
        released.recv().unwrap();
        std::panic::panic_any(PanicsOnDrop)
    });
    submit(&pool, move || queued.send(()).unwrap());
    release.send(()).unwrap();
    assert_eq!(never_ran.recv_timeout(LIMIT), GONE);
    assert_eq!(pool.run(Box::new(|| ())), Err(Rejected));
}

/// A task that queues another one and blocks until it ran: no worker may keep for itself what it submits.
#[test]
fn a_task_queued_by_a_blocked_worker_runs_on_another_one() {
    let pool = Arc::new(ThreadPool::start("blocked", 2));
    for _ in 0..200 {
        let (done, finished) = channel();
        let inner = pool.clone();
        submit(&pool, move || {
            let (ran, awaited) = channel();
            submit(&inner, move || ran.send(()).unwrap());
            done.send(awaited.recv_timeout(LIMIT).is_ok()).unwrap();
        });
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
                        submit(&pool, move || ran.send(()).unwrap());
                    }
                });
            }
        });
        for _ in 0..4 * (round % 5 + 1) {
            runs.recv_timeout(LIMIT).expect("a task never ran");
        }
    }
}

/// A system that cannot start all its threads must not leave the started ones waiting for ever.
#[test]
fn a_failed_spawn_ends_the_workers_already_started() {
    let (ended, first_ended) = channel();
    let mut threads_left = 1;
    let started = catch_unwind(AssertUnwindSafe(|| {
        ThreadPool::start_spawning("partial", 2, |_thread, work| {
            if threads_left == 0 {
                return Err(std::io::Error::other("no more threads"));
            }
            threads_left -= 1;
            let ended = ended.clone();
            std::thread::spawn(move || {
                work();
                ended.send(()).unwrap();
            });
            Ok(())
        })
    }));
    assert!(started.is_err());
    assert_eq!(first_ended.recv_timeout(LIMIT), Ok(()));
}
