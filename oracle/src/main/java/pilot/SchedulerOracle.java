package pilot;

import com.typesafe.config.ConfigFactory;
import java.util.*;
import java.util.concurrent.*;
import org.apache.pekko.actor.*;
import scala.concurrent.duration.Duration;

/**
 * Oracle for {@code scheduleOnce} and {@code scheduleWithFixedDelay} of a message in the Scheduler of Apache Pekko 2.0.0
 * ({@code LightArrayRevolverScheduler} with its default settings) and for {@code Cancellable}. Output: one line per case, {@code <ID> -> <observations>}. No observation depends
 * on a race: presence is awaited, and absence is shown by a later marker that arrives first, with a margin of at least 500 ms for
 * the cancel or the termination that has to happen before the message that must not arrive.
 */
final class SchedulerOracle {
  private SchedulerOracle() {}

  static final Map<String, Callable<String>> CASES = new LinkedHashMap<>();
  private static void run(String id, Callable<String> f) { if (CASES.put(id, f) != null) throw new IllegalStateException("dup " + id); }
  static List<String> ids() { register(); return new ArrayList<>(CASES.keySet()); }

  static String line(String id) {
    register();
    Callable<String> f = CASES.get(id);
    if (f == null) throw new IllegalArgumentException("unknown case id " + id);
    String r;
    try { r = f.call(); } catch (Throwable t) { r = err(t); }
    return id + " -> " + r;
  }

  static String err(Throwable t) { return "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]"; }

  /** One system with an actor {@code target} that queues what it receives, and an idle actor {@code asker}. */
  static final class Env implements AutoCloseable {
    final ActorSystem sys;
    final BlockingQueue<String> seen = new LinkedBlockingQueue<>();
    final ActorRef target, asker;

    Env(String name) {
      sys = ActorSystem.create(name, ConfigFactory.parseString("pekko.loglevel = OFF\npekko.stdout-loglevel = OFF\npekko.log-dead-letters = off"));
      target = sys.actorOf(Props.create(Recorder.class, () -> new Recorder(seen)), "target");
      asker = sys.actorOf(Props.create(Recorder.class, () -> new Recorder(new LinkedBlockingQueue<>())), "asker");
    }

    Cancellable once(long nanos, ActorRef to, Object message, ActorRef sender) {
      return sys.scheduler().scheduleOnce(Duration.fromNanos(nanos), to, message, sys.dispatcher(), sender);
    }

    Cancellable repeat(long initialNanos, long nanos, ActorRef to, Object message, ActorRef sender) {
      return sys.scheduler().scheduleWithFixedDelay(Duration.fromNanos(initialNanos), Duration.fromNanos(nanos), to, message, sys.dispatcher(), sender);
    }

    String next() throws InterruptedException {
      String s = seen.poll(10, TimeUnit.SECONDS);
      return s == null ? "nothing" : s;
    }

    @Override public void close() throws Exception {
      sys.terminate();
      sys.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS);
    }
  }

  static final class Recorder extends AbstractActor {
    private final BlockingQueue<String> seen;
    Recorder(BlockingQueue<String> seen) { this.seen = seen; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchAny(m -> {
        if (m instanceof DeadLetter d) seen.add("dead " + d.message() + " sender=" + d.sender().path().name() + " recipient=" + d.recipient().path().name());
        else seen.add(m + " from " + getSender().path().name());
      }).build();
    }
  }

  static String state(Cancellable c) {
    boolean first = c.cancel(), after = c.isCancelled(), second = c.cancel();
    return "cancel=" + first + " isCancelled=" + after + " cancel=" + second + " isCancelled=" + c.isCancelled();
  }

  static final long MS = 1_000_000L;
  /** {@code Int.MaxValue} ticks of 10 ms. */
  static final long LARGEST = Integer.MAX_VALUE * 10L * MS;

  static String attempt(Env e, long nanos) {
    try { return state(e.once(nanos, e.target, "far", ActorRef.noSender())); } catch (Throwable t) { return err(t); }
  }

  static String attemptRepeat(Env e, long initialNanos, long nanos) {
    try { return state(e.repeat(initialNanos, nanos, e.target, "far", ActorRef.noSender())); } catch (Throwable t) { return err(t); }
  }

  static boolean awaitTerminated(ActorRef ref) throws InterruptedException {
    long deadline = System.nanoTime() + 5_000 * MS;
    while (!((InternalActorRef) ref).isTerminated()) {
      if (System.nanoTime() > deadline) return false;
      Thread.sleep(2);
    }
    return true;
  }

  private static void registerFixedDelay() {
    run("SCH-FD-01 the message repeats with the given sender, never before the initial delay and the delays; cancel stops it", () -> {
      try (Env e = new Env("sch")) {
        long start = System.nanoTime();
        Cancellable c = e.repeat(50 * MS, 600 * MS, e.target, "tick", e.asker);
        String seen = e.next() + " | " + e.next() + " | " + e.next();
        boolean early = System.nanoTime() - start < 1250 * MS;
        String state = state(c);
        e.once(900 * MS, e.target, "marker", ActorRef.noSender());
        return seen + " | early=" + early + " | " + state + " | next=" + e.next();
      }
    });
    run("SCH-FD-02 a repetition cancelled during the initial delay sends nothing", () -> {
      try (Env e = new Env("sch")) {
        Cancellable c = e.repeat(1000 * MS, 100 * MS, e.target, "cancelled", ActorRef.noSender());
        String before = "isCancelled=" + c.isCancelled() + " " + state(c);
        e.once(1500 * MS, e.target, "marker", ActorRef.noSender());
        return before + " | next=" + e.next();
      }
    });
    run("SCH-FD-03 an initial delay of zero sends at once, and the repetition can still be cancelled", () -> {
      try (Env e = new Env("sch")) {
        Cancellable c = e.repeat(0, 1000 * MS, e.target, "now", ActorRef.noSender());
        String first = e.next(), state = state(c);
        e.once(1500 * MS, e.target, "marker", ActorRef.noSender());
        return first + " | " + state + " | next=" + e.next();
      }
    });
    run("SCH-FD-04 the delay must be positive and within the reach of the wheel, and is checked before the initial delay", () -> {
      try (Env e = new Env("sch")) {
        StringBuilder sb = new StringBuilder("zero => " + attemptRepeat(e, 0, 0));
        sb.append(" | zero, initial too far => ").append(attemptRepeat(e, LARGEST + 1, 0));
        sb.append(" | delay too far => ").append(attemptRepeat(e, 0, LARGEST + 1));
        sb.append(" | initial too far => ").append(attemptRepeat(e, LARGEST + 1, 1000 * MS));
        sb.append(" | largest => ").append(attemptRepeat(e, LARGEST, LARGEST));
        e.once(0, e.target, "marker", ActorRef.noSender());
        return sb.append(" | next=").append(e.next()).toString();
      }
    });
    run("SCH-FD-05 a negative delay is rejected like zero", () -> {
      try (Env e = new Env("sch")) {
        return attemptRepeat(e, 0, -5000 * MS);
      }
    });
    run("SCH-FD-06 a target that stops between two sends gets one more, a dead letter, and the repetition ends uncancelled", () -> {
      try (Env e = new Env("sch")) {
        BlockingQueue<String> letters = new LinkedBlockingQueue<>();
        ActorRef listener = e.sys.actorOf(Props.create(Recorder.class, () -> new Recorder(letters)), "listener");
        e.sys.eventStream().subscribe(listener, DeadLetter.class);
        Cancellable c = e.repeat(0, 600 * MS, e.target, "late", e.asker);
        String first = e.next();
        e.sys.stop(e.target);
        if (!awaitTerminated(e.target)) return "target did not stop";
        String letter = String.valueOf(letters.poll(10, TimeUnit.SECONDS));
        e.once(1500 * MS, listener, "marker", ActorRef.noSender());
        return first + " | " + letter + " | next=" + letters.poll(10, TimeUnit.SECONDS) + " | isCancelled=" + c.isCancelled() + " " + state(c);
      }
    });
    run("SCH-FD-07 deadLetters counts as terminated: it gets the message once and the repetition ends", () -> {
      try (Env e = new Env("sch")) {
        BlockingQueue<String> letters = new LinkedBlockingQueue<>();
        ActorRef listener = e.sys.actorOf(Props.create(Recorder.class, () -> new Recorder(letters)), "listener");
        e.sys.eventStream().subscribe(listener, DeadLetter.class);
        boolean terminated = ((InternalActorRef) e.sys.deadLetters()).isTerminated();
        Cancellable c = e.repeat(100 * MS, 100 * MS, e.sys.deadLetters(), "gone", e.asker);
        String letter = String.valueOf(letters.poll(10, TimeUnit.SECONDS));
        e.once(600 * MS, listener, "marker", ActorRef.noSender());
        return "isTerminated=" + terminated + " | " + letter + " | next=" + letters.poll(10, TimeUnit.SECONDS) + " | isCancelled=" + c.isCancelled() + " " + state(c);
      }
    });
    run("SCH-FD-08 termination discards a pending repetition, and the scheduler then rejects new ones", () -> {
      try (Env other = new Env("other")) {
        Env closing = new Env("closing");
        Cancellable pending = closing.repeat(1000 * MS, 1000 * MS, other.target, "pending", ActorRef.noSender());
        closing.close();
        other.once(1500 * MS, other.target, "marker", ActorRef.noSender());
        StringBuilder sb = new StringBuilder("next=" + other.next() + " | pending: isCancelled=" + pending.isCancelled() + " " + state(pending));
        sb.append(" | initial zero => ").append(attemptRepeat(closing, 0, 1000 * MS));
        sb.append(" | initial 1s => ").append(attemptRepeat(closing, 1000 * MS, 1000 * MS));
        sb.append(" | initial too far => ").append(attemptRepeat(closing, LARGEST + 1, 1000 * MS));
        sb.append(" | delay zero => ").append(attemptRepeat(closing, 1000 * MS, 0));
        return sb.toString();
      }
    });
  }

  private static synchronized void register() {
    if (!CASES.isEmpty()) return;

    run("SCH-SO-01 the message arrives once, not before the delay, with the given sender; it cannot be cancelled afterwards", () -> {
      try (Env e = new Env("sch")) {
        long start = System.nanoTime();
        Cancellable c = e.once(50 * MS, e.target, "tick", e.asker);
        String first = e.next();
        boolean early = System.nanoTime() - start < 50 * MS;
        e.once(50 * MS, e.target, "marker", ActorRef.noSender());
        return first + " | early=" + early + " | " + state(c) + " | next=" + e.next();
      }
    });
    run("SCH-SO-02 a message cancelled before the delay is never sent; only the first cancel succeeds", () -> {
      try (Env e = new Env("sch")) {
        Cancellable c = e.once(1000 * MS, e.target, "cancelled", ActorRef.noSender());
        String before = "isCancelled=" + c.isCancelled() + " " + state(c);
        e.once(1500 * MS, e.target, "marker", ActorRef.noSender());
        return before + " | next=" + e.next();
      }
    });
    run("SCH-SO-03 a delay of zero sends at once and gives a task that is not cancellable", () -> {
      try (Env e = new Env("sch")) {
        Cancellable c = e.once(0, e.target, "now", ActorRef.noSender());
        return e.next() + " | " + state(c);
      }
    });
    run("SCH-SO-04 a negative delay behaves like zero", () -> {
      try (Env e = new Env("sch")) {
        Cancellable c = e.once(-5000 * MS, e.target, "now", ActorRef.noSender());
        return e.next() + " | " + state(c);
      }
    });
    run("SCH-SO-05 the largest delay is Int.MaxValue ticks; the delay is rounded up to the tick before the check", () -> {
      try (Env e = new Env("sch")) {
        return "largest => " + attempt(e, LARGEST) + " | largest+1ns => " + attempt(e, LARGEST + 1);
      }
    });
    run("SCH-SO-06 a target that stopped before the delay elapsed gets nothing: the message is a dead letter", () -> {
      try (Env e = new Env("sch")) {
        BlockingQueue<String> letters = new LinkedBlockingQueue<>();
        ActorRef listener = e.sys.actorOf(Props.create(Recorder.class, () -> new Recorder(letters)), "listener");
        e.sys.eventStream().subscribe(listener, DeadLetter.class);
        e.once(1000 * MS, e.target, "late", e.asker);
        e.sys.stop(e.target);
        long deadline = System.nanoTime() + 5_000 * MS;
        while (!((InternalActorRef) e.target).isTerminated()) {
          if (System.nanoTime() > deadline) return "target did not stop";
          Thread.sleep(2);
        }
        return String.valueOf(letters.poll(10, TimeUnit.SECONDS));
      }
    });
    run("SCH-TE-01 termination discards what is pending without sending it, and the scheduler then rejects every delay", () -> {
      try (Env other = new Env("other")) {
        Env closing = new Env("closing");
        Cancellable pending = closing.once(1000 * MS, other.target, "pending", ActorRef.noSender());
        closing.close();
        other.once(1500 * MS, other.target, "marker", ActorRef.noSender());
        StringBuilder sb = new StringBuilder("next=" + other.next() + " | pending: " + state(pending));
        sb.append(" | zero => ").append(attempt(closing, 0));
        sb.append(" | 1s => ").append(attempt(closing, 1000 * MS));
        sb.append(" | too far => ").append(attempt(closing, LARGEST + 1));
        return sb.toString();
      }
    });
    registerFixedDelay();
  }
}
