package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import java.util.function.BooleanSupplier;
import org.apache.pekko.actor.*;
import org.apache.pekko.dispatch.*;
import org.apache.pekko.event.Logging;
import scala.concurrent.duration.FiniteDuration;

/**
 * Oracle for {@code BoundedMailbox} of Apache Pekko 2.0.0 (ADR-0001 PR 7a): capacity, the blocking push with
 * {@code mailbox-push-timeout-time}, dead-lettering on timeout, concurrent producers, backpressure, the interaction with the real
 * {@code Dispatcher}/{@code ActorCell}, and the separation of user and system messages. Output: one line per case,
 * {@code <ID> -> <observations>}, deterministic.
 *
 * <p>Determinism rules (a case that cannot follow them is not in the golden; see docs/research/pr-7a-bounded-mailbox-oracle.md):
 * waits are only for an observable condition ({@link Thread#getState} of the thread under test, a counter, an idle mailbox) with a
 * 10 s ceiling, never a sleep; the clock is observed only as the boolean "waited at least the push timeout"; concurrent producers
 * contribute totals, per-producer order and bounds, never interleavings. Consumers that must not run are held in a {@code Block}
 * message, so what the queue accepted is a function of the capacity alone.
 *
 * <p>No patching, no reflection: the instruments are the public SPI (a real {@code BoundedMailbox}, real actors).
 */
final class BoundedMailboxOracle {
  private BoundedMailboxOracle() {}

  static final Map<String, Callable<String>> CASES = new LinkedHashMap<>();
  private static void run(String id, Callable<String> f) { if (CASES.put(id, f) != null) throw new IllegalStateException("dup " + id); }
  static List<String> ids() { register(); return new ArrayList<>(CASES.keySet()); }

  static String line(String id) {
    register();
    Callable<String> f = CASES.get(id);
    if (f == null) throw new IllegalArgumentException("unknown case id " + id);
    String r;
    try { r = f.call(); } catch (Throwable t) { r = "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]"; }
    return id + " -> " + r;
  }

  // ---- helpers -----------------------------------------------------------------------------------------------------------
  static void awaitUntil(BooleanSupplier c) { MailboxOracle.awaitUntil(c); }

  static FiniteDuration ms(long n) { return new FiniteDuration(n, TimeUnit.MILLISECONDS); }

  static String err(Throwable t) {
    // the text of a ConfigException (origin, line, jar paths) belongs to the HOCON layer and is machine dependent: only its class is observed
    if (t instanceof com.typesafe.config.ConfigException) return "ERR " + t.getClass().getSimpleName() + " [config message elided]";
    return "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]";
  }

  static String dur(scala.concurrent.duration.Duration d) { return d.toNanos() + "ns"; }
  static String show(MailboxType m) { return m instanceof BoundedMailbox ? "BoundedMailbox(capacity=" + ((BoundedMailbox) m).capacity() + " pushTimeOut=" + dur(((BoundedMailbox) m).pushTimeOut()) + ")" : String.valueOf(m); }

  static void awaitState(Thread t, Thread.State s) { awaitUntil(() -> t.getState() == s); }


  static final class Block { final CountDownLatch entered = new CountDownLatch(1), release = new CountDownLatch(1); }

  /** Receives {@link Block} (holds the turn), strings (logged in arrival order) and records the mailbox size seen inside every turn. */
  public static final class Consumer extends AbstractActor {
    final BEnv env; Consumer(BEnv env) { this.env = env; }
    @Override public Receive createReceive() {
      return receiveBuilder()
          .match(Block.class, b -> { enter(); b.entered.countDown(); b.release.await(); env.log.add("block"); leave(); })
          .match(String.class, s -> { enter(); env.log.add(s); env.maxQueued.accumulateAndGet(((ActorCell) getContext()).mailbox().numberOfMessages(), Math::max); leave(); })
          .build();
    }
    void enter() { env.max.accumulateAndGet(env.inFlight.incrementAndGet(), Math::max); }
    void leave() { env.inFlight.decrementAndGet(); }
    @Override public void preStart() { env.started.incrementAndGet(); }
  }

  /** Records {@code message<-sender->recipient} of every {@link DeadLetter}, and {@code "fence"} strings. */
  public static final class DeadRecorder extends AbstractActor {
    final Collection<String> dead; DeadRecorder(Collection<String> dead) { this.dead = dead; }
    @Override public Receive createReceive() {
      return receiveBuilder()
          .match(DeadLetter.class, d -> dead.add(show(d.message()) + "<-" + d.sender().path().name() + "->" + d.recipient().path().name()))
          .matchEquals("fence", f -> dead.add("fence")).build();
    }
    static String show(Object m) { return m instanceof DeadLetter ? "DeadLetter(" + show(((DeadLetter) m).message()) + ")" : String.valueOf(m); }
  }

  /** Records {@code source|message} of every {@code Logging.Warning}. */
  public static final class WarnRecorder extends AbstractActor {
    final Collection<String> out; WarnRecorder(Collection<String> out) { this.out = out; }
    @Override public Receive createReceive() { return receiveBuilder().match(Logging.Warning.class, w -> out.add(w.logSource() + "|" + w.message())).build(); }
  }

  public static final class ErrRecorder extends AbstractActor {
    final Collection<String> out; ErrRecorder(Collection<String> out) { this.out = out; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(Logging.Error.class, e -> out.add(e.cause().getClass().getSimpleName() + "|" + e.message())).build();
    }
  }

  /** Does nothing: a named actor to be the sender or the receiver of a message. */
  public static final class Idle extends AbstractActor {
    @Override public Receive createReceive() { return receiveBuilder().build(); }
  }

  static final class BEnv implements AutoCloseable {
    final ConcurrentLinkedQueue<String> log = new ConcurrentLinkedQueue<>(), dead = new ConcurrentLinkedQueue<>(),
        warnings = new ConcurrentLinkedQueue<>(), errors = new ConcurrentLinkedQueue<>();
    final AtomicInteger inFlight = new AtomicInteger(), max = new AtomicInteger(), started = new AtomicInteger(), maxQueued = new AtomicInteger();
    final ActorSystem sys;
    final ActorRef deadRecorder;

    /** {@code mailboxes}: extra HOCON; the id {@code bm} is conventionally the bounded mailbox under test. */
    BEnv(String extra) {
      sys = ActorSystem.create("bm", ConfigFactory.parseString("pekko { loglevel = OFF\n stdout-loglevel = OFF\n log-dead-letters = off\n }\n"
          + "t-disp { type = Dispatcher\n executor = thread-pool-executor\n thread-pool-executor.fixed-pool-size = 1\n throughput = 5\n }\n" + extra)
          .withFallback(ConfigFactory.load()));
      deadRecorder = sys.actorOf(Props.create(DeadRecorder.class, dead));
      sys.eventStream().subscribe(deadRecorder, DeadLetter.class);
      sys.eventStream().subscribe(sys.actorOf(Props.create(WarnRecorder.class, warnings)), Logging.Warning.class);
      sys.eventStream().subscribe(sys.actorOf(Props.create(ErrRecorder.class, errors)), Logging.Error.class);
    }

    static BEnv bounded(int capacity, String pushTimeout) {
      return new BEnv("bm { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = " + capacity
          + "\n mailbox-push-timeout-time = " + pushTimeout + "\n }\n");
    }

    ActorRef spawn(String name) throws Exception { return spawn(name, "t-disp", "bm"); }
    ActorRef spawn(String name, String dispatcher, String mailbox) throws Exception {
      int before = started.get();
      ActorRef r = sys.actorOf(Props.create(Consumer.class, this).withDispatcher(dispatcher).withMailbox(mailbox), name);
      awaitUntil(() -> started.get() > before);
      return r;
    }
    Envelope env(Object msg, ActorRef sender) { return Envelope.apply(msg, sender, sys); }
    ActorRef idle(String name) { return sys.actorOf(Props.create(Idle.class), name); }

    /** Dead letters published so far, after every earlier publication has reached the recorder (fence), without the fence itself. */
    List<String> deadNow() {
      int n = (int) dead.stream().filter("fence"::equals).count();
      deadRecorder.tell("fence", ActorRef.noSender());
      awaitUntil(() -> dead.stream().filter("fence"::equals).count() == n + 1);
      List<String> l = new ArrayList<>(dead); l.removeIf("fence"::equals); return l;
    }
    void awaitDead(int n) { awaitUntil(() -> dead.stream().filter(s -> !s.equals("fence")).count() >= n); }

    @Override public void close() { sys.terminate(); }
  }

  static Mailbox box(ActorRef r) { return MailboxOracle.box(r); }
  static InternalActorRef in(ActorRef r) { return MailboxOracle.in(r); }
  static void idleMailbox(ActorRef r) { MailboxOracle.idle(r); }

  static MessageQueue newQueue(int capacity, FiniteDuration timeout) { return new BoundedMailbox(capacity, timeout).create(scala.Option.empty(), scala.Option.empty()); }
  static int remaining(MessageQueue q) { return ((BlockingQueue<?>) q).remainingCapacity(); }
  static String qs(MessageQueue q) { return "n=" + q.numberOfMessages() + " has=" + q.hasMessages() + " remaining=" + remaining(q); }
  static List<String> drain(MessageQueue q) {
    List<String> out = new ArrayList<>();
    for (Envelope e; (e = q.dequeue()) != null; ) out.add((String) e.message());
    return out;
  }
  static String q(MessageQueue qu) { return qu.getClass().getName(); }

  /** Each producer's sequence {@code p:i} must be in increasing order inside {@code seen}; the sequences may not be contiguous. */
  static boolean perProducerOrdered(Collection<String> seen) {
    Map<String, Integer> last = new HashMap<>();
    for (String s : seen) {
      String[] kv = s.split(":"); if (kv.length != 2) continue;
      int i = Integer.parseInt(kv[1]); Integer prev = last.put(kv[0], i);
      if (prev != null && prev >= i) return false;
    }
    return true;
  }

  static List<Thread> producers(int count, int each, java.util.function.BiConsumer<Integer, String> send) {
    CyclicBarrier go = new CyclicBarrier(count); List<Thread> ts = new ArrayList<>();
    for (int p = 0; p < count; p++) {
      int id = p;
      Thread t = new Thread(() -> { try { go.await(); } catch (Exception x) { throw new RuntimeException(x); } for (int i = 0; i < each; i++) send.accept(id, id + ":" + i); });
      ts.add(t); t.start();
    }
    return ts;
  }
  static void joinAll(List<Thread> ts) throws InterruptedException { for (Thread t : ts) t.join(10_000); }

  // ---- config ----------------------------------------------------------------------------------------------------------------------
  /** {@code new BoundedMailbox(settings, config)} from HOCON text: the constructor Mailboxes uses for a configured mailbox. */
  static String fromConfig(ActorSystem sys, String hocon) {
    try {
      BoundedMailbox m = new BoundedMailbox(sys.settings(), ConfigFactory.parseString(hocon));
      return "capacity=" + m.capacity() + " pushTimeOut=" + dur(m.pushTimeOut());
    } catch (Throwable t) { return err(t); }
  }
  static String table(ActorSystem sys, String keyFormat, String... values) {
    StringBuilder sb = new StringBuilder();
    for (String v : values) sb.append(sb.length() == 0 ? "" : " | ").append(v).append(" => ").append(fromConfig(sys, String.format(keyFormat, v)));
    return sb.toString();
  }

  private static boolean registered;
  private static synchronized void register() {
    if (registered) return;
    registered = true;

    // =============================================================== configuration: BoundedMailbox(settings, config)
    run("BM-CF-01 the default-mailbox keys of reference.conf build a 1000 slot, 10 s mailbox", () -> {
      try (BEnv e = new BEnv("")) {
        Config c = e.sys.settings().config().getConfig("pekko.actor.default-mailbox");
        BoundedMailbox m = new BoundedMailbox(e.sys.settings(), c);
        return "capacity=" + m.capacity() + " pushTimeOut=" + dur(m.pushTimeOut()) + " mailbox-type=" + c.getString("mailbox-type");
      }
    });
    run("BM-CF-02 push timeout values: units, negative, none, nanosecond precision, bad values", () -> {
      try (BEnv e = new BEnv("")) {
        return table(e.sys, "mailbox-capacity = 1\nmailbox-push-timeout-time = %s", "0", "0s", "10s", "250ms", "5", "1ns", "2h", "-1", "-1ms", "-2s", "infinite", "abc", "\"10 parsecs\"", "\"\"", "1.5s", "null");
      }
    });
    run("BM-CF-03 capacity values: zero and one pass the constructor, negative and non-integers do not", () -> {
      try (BEnv e = new BEnv("")) {
        return table(e.sys, "mailbox-push-timeout-time = 1s\nmailbox-capacity = %s", "0", "1", "1000", "2147483647", "2147483648", "-1", "-2147483648", "1.5", "\"7\"", "abc", "null");
      }
    });
    run("BM-CF-04 a missing key is a ConfigException of that key", () -> {
      try (BEnv e = new BEnv("")) {
        return "noCapacity => " + fromConfig(e.sys, "mailbox-push-timeout-time = 1s") + " | noTimeout => " + fromConfig(e.sys, "mailbox-capacity = 1") + " | none => " + fromConfig(e.sys, "x = 1");
      }
    });
    run("BM-CF-05 the case class constructor validates: negative capacity, null timeout; zero capacity is accepted", () -> {
      StringBuilder sb = new StringBuilder();
      try { new BoundedMailbox(-1, ms(1)); sb.append("negative=ok"); } catch (Throwable t) { sb.append("negative=").append(err(t)); }
      try { new BoundedMailbox(1, null); sb.append(" | nullTimeout=ok"); } catch (Throwable t) { sb.append(" | nullTimeout=").append(err(t)); }
      try { BoundedMailbox z = new BoundedMailbox(0, ms(1)); sb.append(" | zero=ok capacity=").append(z.capacity()); } catch (Throwable t) { sb.append(" | zero=").append(err(t)); }
      try { BoundedMailbox z = new BoundedMailbox(Integer.MAX_VALUE, ms(1)); sb.append(" | max=ok capacity=").append(z.capacity()); } catch (Throwable t) { sb.append(" | max=").append(err(t)); }
      return sb.toString();
    });
    run("BM-CF-06 create(): capacity 0 is rejected by the queue it builds, not by the mailbox type", () -> {
      StringBuilder sb = new StringBuilder();
      for (int cap : new int[] {0, 1, 3, Integer.MAX_VALUE}) {
        try { MessageQueue qu = newQueue(cap, ms(1)); sb.append(sb.length() == 0 ? "" : " | ").append("cap=").append(cap).append(" => ").append(q(qu)).append(" ").append(qs(qu)); }
        catch (Throwable t) { sb.append(sb.length() == 0 ? "" : " | ").append("cap=").append(cap).append(" => ").append(err(t)); }
      }
      return sb.toString();
    });
    run("BM-CF-07 the queue it makes is a BlockingQueue exposing its push timeout", () -> {
      MessageQueue qu = newQueue(5, ms(7));
      BoundedMailbox.MessageQueue bq = (BoundedMailbox.MessageQueue) qu;
      return "blocking=" + (qu instanceof BlockingQueue) + " bounded=" + (qu instanceof BoundedMessageQueueSemantics) + " unboundedSemantics=" + (qu instanceof UnboundedMessageQueueSemantics)
          + " pushTimeOut=" + dur(bq.pushTimeOut()) + " " + qs(qu);
    });

    // =============================================================== Mailboxes: looking a bounded mailbox up by id
    run("BM-MB-01 a configured BoundedMailbox with a non-zero timeout warns once per system, whatever the id", () -> {
      try (BEnv e = new BEnv("a { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 3\n mailbox-push-timeout-time = 10s\n }\n"
          + "b { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 4\n mailbox-push-timeout-time = 1ms\n }\n")) {
        Mailboxes mbs = e.sys.mailboxes();
        MailboxType a = mbs.lookup("a"); fenceWarn(e); String afterA = String.valueOf(e.warnings);
        MailboxType b = mbs.lookup("b"); fenceWarn(e);
        return "a=" + show(a) + " warnings after a=" + afterA + " | b=" + show(b) + " warnings after b=" + e.warnings + " | again a same=" + (mbs.lookup("a") == a);
      }
    });
    run("BM-MB-02 a zero timeout does not warn and does not use up the single warning", () -> {
      try (BEnv e = new BEnv("z { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 3\n mailbox-push-timeout-time = 0s\n }\n"
          + "n { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 3\n mailbox-push-timeout-time = -1s\n }\n"
          + "p { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 3\n mailbox-push-timeout-time = 5s\n }\n")) {
        Mailboxes mbs = e.sys.mailboxes();
        mbs.lookup("z"); fenceWarn(e); String zero = String.valueOf(e.warnings);
        mbs.lookup("n"); fenceWarn(e); String negative = String.valueOf(e.warnings);
        mbs.lookup("p"); fenceWarn(e);
        return "afterZero=" + zero + " afterNegative=" + negative + " afterPositive=" + e.warnings;
      }
    });
    run("BM-MB-03 bounded-capacity:N builds BoundedMailbox(N, 0) and never warns", () -> {
      try (BEnv e = new BEnv("")) {
        Mailboxes mbs = e.sys.mailboxes(); StringBuilder sb = new StringBuilder();
        for (String id : new String[] {"bounded-capacity:5", "bounded-capacity:1", "bounded-capacity:0", "bounded-capacity:2147483647", "bounded-capacity:-1", "bounded-capacity:", "bounded-capacity:x", "bounded-capacity:5:6", "bounded-capacity: 7"}) {
          sb.append(sb.length() == 0 ? "" : " | ").append(id).append(" => ");
          try { MailboxType m = mbs.lookup(id); sb.append(show(m)); } catch (Throwable t) { sb.append(err(t)); }
        }
        fenceWarn(e);
        return sb + " | warnings=" + e.warnings;
      }
    });
    run("BM-MB-04 lookup errors: unknown id, the literal id bounded, a class that is not a mailbox, a negative capacity in config", () -> {
      try (BEnv e = new BEnv("bad-class { mailbox-type = \"java.lang.String\" }\nneg { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = -4\n mailbox-push-timeout-time = 1s\n }\n"
          + "empty { mailbox-type = \"\" }\n")) {
        Mailboxes mbs = e.sys.mailboxes(); StringBuilder sb = new StringBuilder();
        for (String id : new String[] {"nope", "bounded", "bad-class", "neg", "empty"}) {
          sb.append(sb.length() == 0 ? "" : " | ").append(id).append(" => ");
          try { sb.append(show(mbs.lookup(id))); } catch (Throwable t) { sb.append(err(t)); if (t.getCause() != null) sb.append(" cause=").append(err(t.getCause())); }
        }
        return sb.toString();
      }
    });
    run("BM-MB-05 an actor that requires BoundedMessageQueueSemantics gets the configured BoundedMailbox (1000, 10 s)", () -> {
      try (BEnv e = new BEnv("")) {
        ActorRef a = e.sys.actorOf(Props.create(RequiresBounded.class).withDispatcher("t-disp"), "r");
        MessageQueue qu = box(a).messageQueue();
        Mailboxes mbs = e.sys.mailboxes();
        return "queue=" + q(qu) + " " + qs(qu) + " | lookupByQueueType=" + show(mbs.lookupByQueueType(BoundedMessageQueueSemantics.class));
      }
    });
    run("BM-MB-06 BoundedMailbox produces a BoundedMailbox.MessageQueue, which fulfils BoundedMessageQueueSemantics and not the unbounded one", () -> {
      try (BEnv e = new BEnv("")) {
        Mailboxes mbs = e.sys.mailboxes(); BoundedMailbox m = new BoundedMailbox(3, ms(1));
        Class<?> produced = mbs.getProducedMessageQueueType(m);
        return "produced=" + produced.getName() + " bounded=" + BoundedMessageQueueSemantics.class.isAssignableFrom(produced)
            + " unbounded=" + UnboundedMessageQueueSemantics.class.isAssignableFrom(produced) + " blocking=" + BlockingQueue.class.isAssignableFrom(produced);
      }
    });

    // =============================================================== the queue alone: capacity and the push timeout
    run("BM-QU-01 timeout 0: a full queue refuses at once, the refused go to dead letters in order, the queue keeps the first", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"), snd = e.idle("snd"); MessageQueue qu = newQueue(3, ms(0)); StringBuilder sb = new StringBuilder("new: " + qs(qu));
        for (String m : new String[] {"a", "b", "c", "d", "e"}) { qu.enqueue(recv, e.env(m, snd)); sb.append(" | ").append(m).append(": ").append(qs(qu)); }
        sb.append(" | dead=").append(e.deadNow()).append(" | drain=").append(drain(qu)).append(" ").append(qs(qu));
        return sb.toString();
      }
    });
    run("BM-QU-02 dequeue frees a slot: the next enqueue is accepted and the order is still first in first out", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(2, ms(0));
        qu.enqueue(recv, e.env("a", recv)); qu.enqueue(recv, e.env("b", recv)); qu.enqueue(recv, e.env("c", recv));
        String first = (String) qu.dequeue().message();
        qu.enqueue(recv, e.env("d", recv)); qu.enqueue(recv, e.env("e", recv));
        return "first=" + first + " rest=" + drain(qu) + " dead=" + e.deadNow() + " " + qs(qu);
      }
    });
    run("BM-QU-03 capacity 1: one accepted, the rest refused", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(1, ms(0));
        for (String m : new String[] {"a", "b", "c"}) qu.enqueue(recv, e.env(m, recv));
        return qs(qu) + " drain=" + drain(qu) + " dead=" + e.deadNow();
      }
    });
    run("BM-QU-04 the dead letter carries the message, the envelope's sender and the receiver; a DeadLetter message is not dropped", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"), snd = e.idle("snd"); MessageQueue qu = newQueue(1, ms(0));
        qu.enqueue(recv, e.env("kept", snd));
        qu.enqueue(recv, e.env("refused", snd));
        qu.enqueue(recv, e.env("noSender", ActorRef.noSender()));
        qu.enqueue(recv, e.env(new DeadLetter("inner", snd, recv), snd));
        return "dead=" + e.deadNow() + " queue=" + drain(qu);
      }
    });
    run("BM-QU-05 timeout > 0: the producer parks for at least the timeout, then the message is a dead letter", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(1, ms(100));
        qu.enqueue(recv, e.env("kept", recv));
        long t0 = System.nanoTime(); qu.enqueue(recv, e.env("late", recv)); long waited = System.nanoTime() - t0;
        return "waitedAtLeastTimeout=" + (waited >= 100_000_000L) + " waitedUnderTenSeconds=" + (waited < 10_000_000_000L) + " dead=" + e.deadNow() + " queue=" + drain(qu);
      }
    });
    run("BM-QU-06 timeout > 0: a dequeue during the wait lets the parked producer through, nothing is dead-lettered", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(1, new FiniteDuration(60, TimeUnit.SECONDS));
        qu.enqueue(recv, e.env("a", recv));
        Thread p = new Thread(() -> qu.enqueue(recv, e.env("b", recv))); p.start();
        awaitState(p, Thread.State.TIMED_WAITING);
        String parked = "state=" + p.getState() + " " + qs(qu);
        String first = (String) qu.dequeue().message(); p.join(10_000);
        return "parked: " + parked + " | first=" + first + " producerAlive=" + p.isAlive() + " rest=" + drain(qu) + " dead=" + e.deadNow();
      }
    });
    run("BM-QU-07 timeout < 0: the producer waits without a deadline until space appears", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(1, ms(-1));
        qu.enqueue(recv, e.env("a", recv));
        Thread p = new Thread(() -> qu.enqueue(recv, e.env("b", recv))); p.start();
        awaitState(p, Thread.State.WAITING);
        String parked = "state=" + p.getState() + " " + qs(qu);
        String first = (String) qu.dequeue().message(); p.join(10_000);
        return "parked: " + parked + " | first=" + first + " producerAlive=" + p.isAlive() + " rest=" + drain(qu) + " dead=" + e.deadNow();
      }
    });
    run("BM-QU-08 an interrupted blocked producer fails with InterruptedException: nothing queued, nothing dead-lettered", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); StringBuilder sb = new StringBuilder();
        for (long timeoutMs : new long[] {60_000, -1}) {
          MessageQueue qu = newQueue(1, ms(timeoutMs)); qu.enqueue(recv, e.env("a", recv));
          AtomicReference<String> outcome = new AtomicReference<>("none");
          Thread p = new Thread(() -> { try { qu.enqueue(recv, e.env("b", recv)); outcome.set("returned"); } catch (Throwable t) { outcome.set(err(t)); } });
          p.start(); awaitState(p, timeoutMs > 0 ? Thread.State.TIMED_WAITING : Thread.State.WAITING); p.interrupt(); p.join(10_000);
          sb.append(sb.length() == 0 ? "" : " | ").append("timeout=").append(timeoutMs > 0 ? "positive" : "negative").append(": ").append(outcome.get()).append(" ").append(qs(qu)).append(" queue=").append(drain(qu));
        }
        return sb + " | dead=" + e.deadNow();
      }
    });
    run("BM-QU-09 cleanUp moves a full queue to the target in order and frees every slot", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(3, ms(0)), target = MailboxOracle.defaultQueue();
        for (String m : new String[] {"a", "b", "c"}) qu.enqueue(recv, e.env(m, recv));
        qu.cleanUp(recv, target);
        qu.enqueue(recv, e.env("after", recv));
        return "source " + qs(qu) + " | target n=" + target.numberOfMessages() + " order=" + drain(target) + " | sourceAfterNewEnqueue=" + drain(qu);
      }
    });
    run("BM-QU-10 a producer parked on a full queue is let through by cleanUp and its message stays in the cleaned queue or goes to the target, never lost", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(2, ms(-1)), target = MailboxOracle.defaultQueue();
        qu.enqueue(recv, e.env("a", recv)); qu.enqueue(recv, e.env("b", recv));
        Thread p = new Thread(() -> qu.enqueue(recv, e.env("c", recv))); p.start(); awaitState(p, Thread.State.WAITING);
        qu.cleanUp(recv, target); p.join(10_000);
        int moved = target.numberOfMessages(), stranded = qu.numberOfMessages();
        return "producerDone=" + !p.isAlive() + " movedPlusStranded=" + (moved + stranded) + " dead=" + e.deadNow() + " firstTwoMovedInOrder=" + drainFirst(target, 2);
      }
    });

    // =============================================================== the queue alone: concurrent producers
    run("BM-QC-01 four producers, timeout 0, nobody dequeues: exactly capacity accepted, every other message dead-lettered", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(10, ms(0));
        joinAll(producers(4, 50, (p, m) -> qu.enqueue(recv, e.env(m, recv))));
        e.awaitDead(190); List<String> dead = e.deadNow();
        List<String> kept = drain(qu);
        List<String> all = new ArrayList<>(kept); for (String d : dead) all.add(d.substring(0, d.indexOf("<-")));
        return "accepted=" + kept.size() + " dead=" + dead.size() + " acceptedOrdered=" + perProducerOrdered(kept) + " deadOrdered=" + perProducerOrdered(deadMessages(dead))
            + " everyMessageAccountedOnce=" + (new HashSet<>(all).size() == 200 && all.size() == 200);
      }
    });
    run("BM-QC-02 four producers and one consumer, long timeout: nothing lost, per-producer order, the queue never above capacity", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(5, new FiniteDuration(60, TimeUnit.SECONDS));
        List<String> got = new ArrayList<>(); AtomicBoolean stop = new AtomicBoolean(); AtomicInteger maxSeen = new AtomicInteger();
        Thread consumer = new Thread(() -> {
          while (got.size() < 200) {
            maxSeen.accumulateAndGet(qu.numberOfMessages(), Math::max);
            Envelope x = qu.dequeue(); if (x != null) got.add((String) x.message());
          }
        });
        consumer.start();
        joinAll(producers(4, 50, (p, m) -> qu.enqueue(recv, e.env(m, recv))));
        consumer.join(20_000);
        return "received=" + got.size() + " distinct=" + new HashSet<>(got).size() + " perProducerOrdered=" + perProducerOrdered(got) + " neverAboveCapacity=" + (maxSeen.get() <= 5)
            + " dead=" + e.deadNow().size() + " " + qs(qu);
      }
    });

    // =============================================================== real actors: Dispatcher, ActorCell, backpressure
    run("BM-AC-01 an actor with a full bounded mailbox: the first `capacity` messages are processed, the rest are dead letters in order", () -> {
      try (BEnv e = BEnv.bounded(3, "0s")) {
        ActorRef a = e.spawn("w"), snd = e.idle("snd"); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        for (String m : new String[] {"m1", "m2", "m3", "m4", "m5", "m6"}) a.tell(m, snd);
        String blocked = "queued=" + box(a).numberOfMessages() + " dead=" + e.deadNow();
        b.release.countDown(); awaitUntil(() -> e.log.size() == 4); idleMailbox(a);
        return "inTurn: " + blocked + " | afterRelease: log=" + e.log + " queued=" + box(a).numberOfMessages() + " dead=" + e.deadNow();
      }
    });
    run("BM-AC-02 the message being processed does not occupy a slot: capacity messages queue behind it", () -> {
      try (BEnv e = BEnv.bounded(1, "0s")) {
        ActorRef a = e.spawn("w"); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        String empty = "queued=" + box(a).numberOfMessages();
        a.tell("m1", ActorRef.noSender()); a.tell("m2", ActorRef.noSender());
        String full = "queued=" + box(a).numberOfMessages() + " dead=" + e.deadNow();
        b.release.countDown(); awaitUntil(() -> e.log.size() == 2); idleMailbox(a);
        return "inTurn before: " + empty + " | after two sends: " + full + " | log=" + e.log;
      }
    });
    run("BM-AC-03 backpressure: with a long timeout the sender thread parks while the mailbox is full and everything arrives in order", () -> {
      try (BEnv e = BEnv.bounded(2, "60s")) {
        ActorRef a = e.spawn("w"); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        Thread sender = new Thread(() -> { for (int i = 1; i <= 20; i++) a.tell("m" + i, ActorRef.noSender()); }); sender.start();
        awaitState(sender, Thread.State.TIMED_WAITING);
        String parked = "senderState=" + sender.getState() + " queued=" + box(a).numberOfMessages();
        b.release.countDown(); sender.join(10_000); awaitUntil(() -> e.log.size() == 21); idleMailbox(a);
        List<String> expected = new ArrayList<>(List.of("block")); for (int i = 1; i <= 20; i++) expected.add("m" + i);
        return parked + " | received=" + e.log.size() + " inOrder=" + expected.equals(new ArrayList<>(e.log)) + " dead=" + e.deadNow() + " senderDone=" + !sender.isAlive();
      }
    });
    run("BM-AC-04 four producers, timeout 0, consumer held: exactly capacity accepted, per-producer order, the rest dead", () -> {
      try (BEnv e = BEnv.bounded(5, "0s")) {
        ActorRef a = e.spawn("w"); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        joinAll(producers(4, 25, (p, m) -> a.tell(m, ActorRef.noSender())));
        e.awaitDead(95); String blocked = "queued=" + box(a).numberOfMessages() + " dead=" + e.deadNow().size();
        b.release.countDown(); awaitUntil(() -> e.log.size() == 6); idleMailbox(a);
        List<String> received = new ArrayList<>(e.log); received.remove("block");
        List<String> dead = deadMessages(e.deadNow());
        Set<String> all = new HashSet<>(received); all.addAll(dead);
        return "inTurn: " + blocked + " | received=" + received.size() + " perProducerOrdered=" + perProducerOrdered(received) + " dead=" + dead.size()
            + " deadOrdered=" + perProducerOrdered(dead) + " everyMessageAccountedOnce=" + (all.size() == 100 && received.size() + dead.size() == 100);
      }
    });
    run("BM-AC-05 four producers, long timeout, consumer running: nothing lost, per-producer order, mailbox never above capacity, one turn at a time", () -> {
      try (BEnv e = BEnv.bounded(4, "60s")) {
        ActorRef a = e.spawn("w");
        joinAll(producers(4, 50, (p, m) -> a.tell(m, ActorRef.noSender())));
        awaitUntil(() -> e.log.size() == 200); idleMailbox(a);
        return "received=" + e.log.size() + " distinct=" + new HashSet<>(e.log).size() + " perProducerOrdered=" + perProducerOrdered(e.log)
            + " mailboxNeverAboveCapacity=" + (e.maxQueued.get() <= 4) + " maxInFlight=" + e.max.get() + " dead=" + e.deadNow().size();
      }
    });
    run("BM-AC-06 system messages are not bounded: a full mailbox still takes Suspend/Resume and they run before the queued user messages", () -> {
      try (BEnv e = BEnv.bounded(2, "0s")) {
        ActorRef a = e.spawn("w"); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        a.tell("m1", ActorRef.noSender()); a.tell("m2", ActorRef.noSender());
        String full = "queued=" + box(a).numberOfMessages();
        in(a).suspend(); in(a).suspend(); in(a).resume(null);
        String sys = "queued=" + box(a).numberOfMessages() + " hasSystemMessages=" + box(a).hasSystemMessages() + " dead=" + e.deadNow();
        b.release.countDown(); awaitUntil(() -> box(a).suspendCount() == 1 && !box(a).hasSystemMessages()); idleMailbox(a);
        String suspended = "suspendCount=" + box(a).suspendCount() + " queued=" + box(a).numberOfMessages() + " log=" + e.log;
        in(a).resume(null); awaitUntil(() -> e.log.size() == 3); idleMailbox(a);
        return "full: " + full + " | systemMessagesSent: " + sys + " | afterTurn: " + suspended + " | afterResume: log=" + e.log + " suspendCount=" + box(a).suspendCount();
      }
    });
    run("BM-AC-07 a suspended actor with a full mailbox stays unscheduled, refuses further user messages, and resumes with the queued ones", () -> {
      try (BEnv e = BEnv.bounded(2, "0s")) {
        ActorRef a = e.spawn("w");
        in(a).suspend(); awaitUntil(() -> box(a).suspendCount() == 1); idleMailbox(a);
        for (String m : new String[] {"m1", "m2", "m3", "m4"}) a.tell(m, ActorRef.noSender());
        String suspended = "queued=" + box(a).numberOfMessages() + " scheduled=" + box(a).isScheduled() + " dead=" + e.deadNow() + " log=" + e.log;
        in(a).resume(null); awaitUntil(() -> e.log.size() == 2); idleMailbox(a);
        a.tell("m5", ActorRef.noSender()); awaitUntil(() -> e.log.size() == 3); idleMailbox(a);
        return "suspended: " + suspended + " | afterResume: log=" + e.log + " dead=" + e.deadNow();
      }
    });
    run("BM-AC-08 a sender parked on a full mailbox is a suspended actor's problem too: with a timeout it ends in a dead letter", () -> {
      try (BEnv e = BEnv.bounded(1, "100ms")) {
        ActorRef a = e.spawn("w");
        in(a).suspend(); awaitUntil(() -> box(a).suspendCount() == 1); idleMailbox(a);
        a.tell("m1", ActorRef.noSender());
        long t0 = System.nanoTime(); a.tell("m2", ActorRef.noSender()); long waited = System.nanoTime() - t0;
        return "waitedAtLeastTimeout=" + (waited >= 100_000_000L) + " dead=" + e.deadNow() + " queued=" + box(a).numberOfMessages();
      }
    });
    run("BM-AC-09 stop while the mailbox is full: Terminate overtakes the queued user messages, which become dead letters in order", () -> {
      try (BEnv e = BEnv.bounded(3, "0s")) {
        ActorRef a = e.spawn("w"); Block b = new Block(); Mailbox mb;
        a.tell(b, ActorRef.noSender()); b.entered.await(); mb = box(a);
        for (String m : new String[] {"m1", "m2", "m3", "m4"}) a.tell(m, ActorRef.noSender());
        in(a).stop(); awaitUntil(mb::hasSystemMessages);
        b.release.countDown(); awaitUntil(() -> mb.isClosed()); e.awaitDead(4);
        return "log=" + e.log + " dead=" + e.deadNow() + " oldMailbox: closed=" + mb.isClosed() + " queued=" + mb.numberOfMessages();
      }
    });
    run("BM-AC-10 a stopped actor does not block its senders, whatever the timeout: the closed mailbox is the dead-letter mailbox", () -> {
      try (BEnv e = BEnv.bounded(1, "-1ms")) {
        ActorRef a = e.spawn("w"), probe = e.idle("probe");
        in(a).stop(); e.sys.eventStream().subscribe(probe, DeadLetter.class); // keeps the system quiet; the recorder is the observer
        awaitUntil(() -> box(a).isClosed());
        Thread t = new Thread(() -> { a.tell("x", ActorRef.noSender()); a.tell("y", ActorRef.noSender()); }); t.start(); t.join(10_000);
        e.awaitDead(2);
        return "senderDone=" + !t.isAlive() + " dead=" + e.deadNow();
      }
    });
    run("BM-AC-11 a sender parked on a full mailbox when the actor stops: every message is dead-lettered or stranded in the old queue, none is processed", () -> {
      try (BEnv e = BEnv.bounded(2, "-1ms")) {
        ActorRef a = e.spawn("w"); Block b = new Block(); Mailbox mb;
        a.tell(b, ActorRef.noSender()); b.entered.await(); mb = box(a);
        a.tell("m1", ActorRef.noSender()); a.tell("m2", ActorRef.noSender());
        Thread sender = new Thread(() -> a.tell("m3", ActorRef.noSender())); sender.start(); awaitState(sender, Thread.State.WAITING);
        in(a).stop(); awaitUntil(mb::hasSystemMessages);
        b.release.countDown(); sender.join(10_000); awaitUntil(mb::isClosed);
        // terminal state: m1 and m2 are always drained to dead letters by cleanUp (m3 is a dead letter or stranded), so wait until both are published
        awaitUntil(() -> e.deadNow().size() >= 2 && e.deadNow().size() + mb.messageQueue().numberOfMessages() == 3);
        List<String> dead = deadMessages(e.deadNow());
        return "senderDone=" + !sender.isAlive() + " processed=" + e.log + " deadPlusStranded=" + (dead.size() + mb.messageQueue().numberOfMessages()) + " firstTwoDeadInOrder=" + dead.subList(0, Math.min(2, dead.size()));
      }
    });
    run("BM-AC-12 a sender on the actor's own single-thread pool starves it: the push timeout is what ends the wait", () -> {
      try (BEnv e = new BEnv("bm { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 1\n mailbox-push-timeout-time = 150ms\n }\n")) {
        ActorRef target = e.spawn("target");
        AtomicLong waited = new AtomicLong(); CountDownLatch done = new CountDownLatch(1);
        ActorRef sender = e.sys.actorOf(Props.create(PoolSender.class, target, waited, done).withDispatcher("t-disp"), "sender");
        sender.tell("go", ActorRef.noSender()); done.await(10, TimeUnit.SECONDS);
        awaitUntil(() -> e.log.size() == 1); idleMailbox(target);
        return "senderTurnWaitedAtLeastTimeout=" + (waited.get() >= 150_000_000L) + " received=" + e.log + " dead=" + e.deadNow();
      }
    });
    run("BM-AC-13 Props.withMailbox with capacity 0: accepted by the lookup, the actor's cell cannot be made", () -> {
      try (BEnv e = new BEnv("bm { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 0\n mailbox-push-timeout-time = 0s\n }\n")) {
        String lookup = show(e.sys.mailboxes().lookup("bm"));
        String outcome;
        try {
          ActorRef a = e.sys.actorOf(Props.create(Consumer.class, e).withDispatcher("t-disp").withMailbox("bm"), "w");
          awaitUntil(() -> ((ActorRefWithCell) a).isTerminated() || e.started.get() > 0);
          outcome = "actorOf returned started=" + e.started.get() + " terminated=" + a.isTerminated();
        } catch (Throwable t) { outcome = "actorOf threw " + err(t); }
        return "lookup=" + lookup + " | " + outcome;
      }
    });
    run("BM-AC-14 Props.withMailbox with a negative capacity in config: the lookup fails when the actor is created", () -> {
      try (BEnv e = new BEnv("bm { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = -3\n mailbox-push-timeout-time = 0s\n }\n")) {
        try {
          ActorRef a = e.sys.actorOf(Props.create(Consumer.class, e).withDispatcher("t-disp").withMailbox("bm"), "w");
          return "actorOf returned started=" + e.started.get();
        } catch (Throwable t) { return "actorOf threw " + err(t) + (t.getCause() != null ? " cause=" + err(t.getCause()) : ""); }
      }
    });
    run("BM-AC-15 the dispatcher's own mailbox-type key selects the bounded mailbox for its actors", () -> {
      try (BEnv e = new BEnv("bm-disp { type = Dispatcher\n executor = thread-pool-executor\n thread-pool-executor.fixed-pool-size = 1\n throughput = 5\n"
          + " mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 2\n mailbox-push-timeout-time = 0s\n }\n")) {
        ActorRef a = e.sys.actorOf(Props.create(Consumer.class, e).withDispatcher("bm-disp"), "w");
        awaitUntil(() -> e.started.get() == 1);
        MessageQueue qu = box(a).messageQueue();
        return "queue=" + q(qu) + " remaining=" + remaining(qu);
      }
    });

    run("BM-AC-16 a sender interrupted while parked on a full mailbox: ActorCell swallows the InterruptedException, publishes an Error and restores the interrupt flag; the message is neither queued nor dead-lettered", () -> {
      try (BEnv e = BEnv.bounded(1, "-1ms")) {
        ActorRef a = e.spawn("w"); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        a.tell("m1", ActorRef.noSender());
        AtomicReference<String> outcome = new AtomicReference<>("none");
        Thread sender = new Thread(() -> { try { a.tell("m2", ActorRef.noSender()); outcome.set("returned interruptedFlag=" + Thread.currentThread().isInterrupted()); } catch (Throwable t) { outcome.set(err(t)); } });
        sender.start(); awaitState(sender, Thread.State.WAITING); sender.interrupt(); sender.join(10_000);
        awaitUntil(() -> !e.errors.isEmpty());
        String during = "queued=" + box(a).numberOfMessages();
        b.release.countDown(); awaitUntil(() -> e.log.size() == 2); idleMailbox(a);
        return "sender: " + outcome.get() + " errors=" + e.errors + " " + during + " | log=" + e.log + " dead=" + e.deadNow();
      }
    });

    // =============================================================== an unstarted top-level actor with a bounded mailbox (UnstartedCell drains into it)
    run("BM-UC-01 messages held by the UnstartedCell are delivered into the bounded mailbox when it starts: the first `capacity` are kept, the overflow is dead-lettered in order", () -> {
      try (BEnv e = BEnv.bounded(2, "0s")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        ActorRef spawner = e.sys.actorOf(Props.create(UnstartedSpawner.class, e, child, new String[] {"m1", "m2", "m3", "m4"}).withDispatcher("t-disp"), "spawner");
        spawner.tell("go", ActorRef.noSender());
        awaitUntil(() -> e.started.get() == 1); awaitUntil(() -> e.log.size() == 2); idleMailbox(child.get());
        return "log=" + e.log + " dead=" + e.deadNow();
      }
    });
    run("BM-UC-02 the drain runs on the thread of the actor that attaches the child: with a one-thread pool nobody consumes meanwhile, so each overflowing message is dead-lettered once its push timeout expires", () -> {
      try (BEnv e = BEnv.bounded(1, "100ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        ActorRef spawner = e.sys.actorOf(Props.create(UnstartedSpawner.class, e, child, new String[] {"m1", "m2", "m3"}).withDispatcher("t-disp"), "spawner");
        spawner.tell("go", ActorRef.noSender());
        awaitUntil(() -> e.started.get() == 1); awaitUntil(() -> e.log.size() == 1); idleMailbox(child.get());
        return "log=" + e.log + " dead=" + e.deadNow();
      }
    });
    run("BM-UC-03 a sender that meets the UnstartedCell while it drains into a full bounded mailbox waits unstarted-push-timeout for the lock, then its message is a dead letter with a warning", () -> {
      try (BEnv e = new BEnv("pekko.actor.unstarted-push-timeout = 150ms\nbm { mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n mailbox-capacity = 1\n mailbox-push-timeout-time = 2s\n }\n")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        ActorRef spawner = e.sys.actorOf(Props.create(UnstartedSpawner.class, e, child, new String[] {"m1", "m2"}).withDispatcher("t-disp"), "spawner");
        spawner.tell("go", ActorRef.noSender());
        awaitUntil(() -> child.get() != null && parkedInDrain());
        long t0 = System.nanoTime(); child.get().tell("late", ActorRef.noSender()); long waited = System.nanoTime() - t0;
        awaitUntil(() -> e.started.get() == 1); awaitUntil(() -> e.log.size() == 1); idleMailbox(child.get());
        List<String> w = new ArrayList<>(); for (String x : e.warnings) if (x.contains("due to lock timeout")) w.add(x);
        return "senderWaitedAtLeastLockTimeout=" + (waited >= 150_000_000L) + " senderWaitedLessThanPushTimeout=" + (waited < 1_500_000_000L) + " log=" + e.log + " dead=" + e.deadNow() + " lockWarnings=" + w;
      }
    });
  }

  static void fenceWarn(BEnv e) {
    // warnings reach WarnRecorder asynchronously: fence through the same actor by publishing a marker warning
    String marker = "fence-" + System.nanoTime();
    e.sys.eventStream().publish(new Logging.Warning("fence", BoundedMailboxOracle.class, marker));
    awaitUntil(() -> e.warnings.stream().anyMatch(s -> s.endsWith(marker)));
    e.warnings.removeIf(s -> s.endsWith(marker));
  }


  /** True while some thread is parked inside {@code UnstartedCell.replaceWith}: the drain is blocked in the bounded enqueue, holding the lock. */
  static boolean parkedInDrain() {
    for (Map.Entry<Thread, StackTraceElement[]> t : Thread.getAllStackTraces().entrySet()) {
      if (t.getKey().getState() != Thread.State.TIMED_WAITING) continue;
      boolean replace = false, offer = false;
      for (StackTraceElement f : t.getValue()) { replace |= f.getMethodName().equals("replaceWith"); offer |= f.getClassName().contains("BoundedQueueBasedMessageQueue"); }
      if (replace && offer) return true;
    }
    return false;
  }

  /** Creates a child with the asynchronous {@code attachChild} (what {@code system.actorOf} uses) and sends it messages in the same turn: they reach its UnstartedCell. */
  public static final class UnstartedSpawner extends AbstractActor {
    final BEnv env; final AtomicReference<ActorRef> child; final String[] messages;
    UnstartedSpawner(BEnv env, AtomicReference<ActorRef> child, String[] messages) { this.env = env; this.child = child; this.messages = messages; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> {
        ActorRef c = ((ActorCell) getContext()).attachChild(Props.create(Consumer.class, env).withDispatcher("t-disp").withMailbox("bm"), "c", false);
        for (String m : messages) c.tell(m, self());
        child.set(c);
      }).build();
    }
  }

  static List<String> deadMessages(List<String> dead) {
    List<String> l = new ArrayList<>(); for (String d : dead) l.add(d.substring(0, d.indexOf("<-"))); return l;
  }

  static List<String> drainFirst(MessageQueue qu, int n) {
    List<String> l = new ArrayList<>(); for (int i = 0; i < n; i++) { Envelope x = qu.dequeue(); if (x != null) l.add((String) x.message()); } return l;
  }

  /** An actor that, in its only turn, sends two messages to a bounded target running on the same one-thread dispatcher. */
  public static final class PoolSender extends AbstractActor {
    final ActorRef target; final AtomicLong waited; final CountDownLatch done;
    PoolSender(ActorRef target, AtomicLong waited, CountDownLatch done) { this.target = target; this.waited = waited; this.done = done; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> {
        target.tell("first", self()); long t0 = System.nanoTime(); target.tell("second", self()); waited.set(System.nanoTime() - t0); done.countDown();
      }).build();
    }
  }

  public static final class RequiresBounded extends AbstractActor implements RequiresMessageQueue<BoundedMessageQueueSemantics> {
    @Override public Receive createReceive() { return receiveBuilder().build(); }
  }
}
