package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import java.util.function.BooleanSupplier;
import org.apache.pekko.actor.*;
import org.apache.pekko.dispatch.*;
import java.util.concurrent.atomic.AtomicInteger;
import org.apache.pekko.dispatch.sysmsg.*;
import org.apache.pekko.event.Logging;
import org.apache.pekko.pattern.Patterns;

/**
 * Oracle for the mailbox state machine and processing order of Apache Pekko 2.0.0 (ADR-0001, PR 1/10): rows AC-MBX-001/002/003,
 * AC-CELL-003, AC-CELL-013.1 and AC-SUP-010. Output: one line per case, {@code <ID> -> <observations>}, deterministic.
 *
 * <p>Two instruments, neither of which changes Pekko's behaviour (no patching, no reflection):
 * <ul>
 * <li>{@link Probe}: a real {@code Mailbox} with {@code DefaultSystemMessageQueue}, built as {@code MessageDispatcher.createMailbox}
 *     builds it, but with {@code actor == null} (the case Pekko allows for a mailbox that is never run). It observes the status
 *     transitions and the system queue. Limitation: with no actor, {@code systemEnqueue} after the {@code NoMessage} sentinel drops
 *     the message instead of forwarding it to the {@code deadLetterMailbox}; that path is observed with real actors (OR-04, CC-04).</li>
 * <li>{@link HookQueue}: a public-SPI {@code MessageQueue} (FIFO, same as the default one) that runs a case-supplied callback at
 *     {@code enqueue}, {@code dequeue} and {@code cleanUp}. A callback fires at an exact point of the mailbox protocol (e.g. at the
 *     last empty {@code dequeue} of a turn, before {@code setAsIdle}), so each interleaving is forced, not hoped for.</li>
 * </ul>
 * Waiting is only ever for an observable condition (an event, an idle mailbox) with a 10 s ceiling; absence is proved by reaching
 * an idle mailbox whose queue still holds the message. The window "producer already read the mailbox, then the actor closes" is
 * forced with a latch inside {@code enqueue} (CC-05).
 */
final class MailboxOracle {
  private MailboxOracle() {}

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

  // ---- instrument 1: a bare Mailbox ------------------------------------------------------------------------------------------
  static final class Probe extends Mailbox implements DefaultSystemMessageQueue {
    Probe() { super(new HookQueue(null)); }
    @Override public void systemEnqueue(ActorRef r, SystemMessage m) { DefaultSystemMessageQueue.super.systemEnqueue(r, m); }
    @Override public SystemMessage systemDrain(SystemMessage n) { return DefaultSystemMessageQueue.super.systemDrain(n); }
    @Override public boolean hasSystemMessages() { return DefaultSystemMessageQueue.super.hasSystemMessages(); }
  }

  static String st(Mailbox m) {
    return (m.isClosed() ? "closed" : "open") + " scheduled=" + m.isScheduled() + " suspendCount=" + m.suspendCount() + " raw=" + m.currentStatus();
  }

  static MessageQueue defaultQueue() { return new UnboundedMailbox().create(scala.Option.empty(), scala.Option.empty()); }
  static String counts(MessageQueue q) { return "n=" + q.numberOfMessages() + " has=" + q.hasMessages(); }
  static List<String> drain(MessageQueue q) {
    List<String> out = new ArrayList<>();
    for (Envelope e; (e = q.dequeue()) != null; ) out.add((String) e.message());
    return out;
  }

  static void put(MessageQueue q, String m) { q.enqueue(Nobody$.MODULE$, Envelope.apply(m, Nobody$.MODULE$, null)); }
  static void putFirst(DequeBasedMessageQueueSemantics q, String m) { q.enqueueFirst(Nobody$.MODULE$, Envelope.apply(m, Nobody$.MODULE$, null)); }

  static String names(SystemMessage head) {
    List<String> l = new ArrayList<>();
    for (SystemMessage m = head; m != null; m = m.next()) l.add(m.getClass().getSimpleName());
    return l.toString();
  }

  // ---- instrument 2: a FIFO queue with hooks -----------------------------------------------------------------------------------
  static class Hooks {
    void onEnqueue(Object msg) throws Exception {}
    void onDequeue(Envelope result) {}
    void onCleanUp() {}
    volatile HookQueue queue; // the queue these hooks are attached to
    final CountDownLatch cleaned = new CountDownLatch(1); // counted down when cleanUp has moved everything to dead letters
  }
  static volatile Hooks nextHooks = new Hooks();

  static final class HookQueue implements MessageQueue {
    final ConcurrentLinkedQueue<Envelope> q = new ConcurrentLinkedQueue<>();
    final Hooks hooks;
    HookQueue(Hooks h) { hooks = h == null ? new Hooks() : h; hooks.queue = this; }
    public void enqueue(ActorRef r, Envelope e) {
      try { hooks.onEnqueue(e.message()); } catch (Exception x) { throw new RuntimeException(x); }
      q.add(e);
    }
    public Envelope dequeue() { Envelope e = q.poll(); hooks.onDequeue(e); return e; }
    public int numberOfMessages() { return q.size(); }
    public boolean hasMessages() { return !q.isEmpty(); }
    public void cleanUp(ActorRef owner, MessageQueue deadLetters) {
      hooks.onCleanUp();
      for (Envelope e; (e = q.poll()) != null; ) deadLetters.enqueue(owner, e);
      hooks.cleaned.countDown();
    }
  }

  public static final class HookType implements MailboxType, ProducesMessageQueue<HookQueue> {
    public HookType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { return new HookQueue(nextHooks); }
  }

  /** A queue that hands out the newest message first and loses none: tells a configured mailbox type apart from the unbounded FIFO one. */
  static final class LifoQueue implements MessageQueue, org.apache.pekko.dispatch.UnboundedMessageQueueSemantics {
    final java.util.concurrent.ConcurrentLinkedDeque<Envelope> q = new java.util.concurrent.ConcurrentLinkedDeque<>();
    public void enqueue(ActorRef r, Envelope e) { q.addFirst(e); }
    public Envelope dequeue() { return q.pollFirst(); }
    public int numberOfMessages() { return q.size(); }
    public boolean hasMessages() { return !q.isEmpty(); }
    public void cleanUp(ActorRef owner, MessageQueue deadLetters) { for (Envelope e; (e = q.pollFirst()) != null; ) deadLetters.enqueue(owner, e); }
  }

  public static final class LifoType implements MailboxType, ProducesMessageQueue<LifoQueue> {
    public LifoType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { return new LifoQueue(); }
  }

  /** A mailbox type whose {@code create} throws: the way a public SPI makes {@code newCell} of an asynchronously created actor fail. */
  public static final class FailingType implements MailboxType, ProducesMessageQueue<UnboundedMailbox.MessageQueue> {
    public FailingType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { throw new IllegalStateException("mailbox boom"); }
  }

  /** Creates a child whose cell cannot be made, inside its own turn, and restarts it when it fails to initialise. */
  public static final class RestartingParent extends AbstractActor {
    final Env env; final AtomicReference<ActorRef> child; final boolean resume;
    RestartingParent(Env env, AtomicReference<ActorRef> child) { this(env, child, false); }
    RestartingParent(Env env, AtomicReference<ActorRef> child, boolean resume) { this.env = env; this.child = child; this.resume = resume; }
    @Override public SupervisorStrategy supervisorStrategy() {
      return new OneForOneStrategy(10, java.time.Duration.ofMinutes(1), t -> resume ? SupervisorStrategy.resume() : SupervisorStrategy.restart());
    }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g ->
          child.set(((ActorCell) getContext()).attachChild(Props.create(Worker.class, env).withMailbox("failing-mailbox"), "c", false))).build();
    }
  }

  /** Creates `k` in preStart and throws on the first start: a restart has to wait for `k` to die, or the second `k` would clash. */
  public static final class Rebuilder extends AbstractActor {
    final Env env; final AtomicInteger starts;
    Rebuilder(Env env, AtomicInteger starts) { this.env = env; this.starts = starts; }
    @Override public void preStart() {
      int n = starts.incrementAndGet();
      getContext().actorOf(Props.create(Worker.class, env), "k");
      env.log.add("preStart:" + n);
      if (n == 1) throw new IllegalStateException("first start fails");
    }
    @Override public Receive createReceive() { return receiveBuilder().matchAny(m -> env.log.add("r:" + m)).build(); }
  }

  /** Restarts whatever fails, creating `c` from `props` when told "go". */
  public static final class RestartingSupervisor extends AbstractActor {
    final Props props; final AtomicReference<ActorRef> child;
    RestartingSupervisor(Props props, AtomicReference<ActorRef> child) { this.props = props; this.child = child; }
    @Override public SupervisorStrategy supervisorStrategy() {
      return new OneForOneStrategy(10, java.time.Duration.ofMinutes(1), t -> SupervisorStrategy.restart());
    }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> child.set(((ActorCell) getContext()).attachChild(props, "c", false))).build();
    }
  }

  /** The mailbox operation that throws in a {@link FaultyQueue} (the {@code MBX-TQ} cases): the public SPI is user code and may fail. */
  enum Fault { ENQUEUE, DEQUEUE, HAS_MESSAGES, ENQUEUE_OF_UNCAUGHT_REPORT }
  static volatile Fault nextFault;

  /** A FIFO queue with one operation that throws: {@code enqueue}/{@code dequeue} of "bad", or the {@code hasMessages} that follows the dequeue of "bad". */
  static final class FaultyQueue implements MessageQueue {
    final ConcurrentLinkedQueue<Envelope> q = new ConcurrentLinkedQueue<>();
    final Fault fault = nextFault;
    final AtomicBoolean armed = new AtomicBoolean();
    public void enqueue(ActorRef r, Envelope e) {
      if (fault == Fault.ENQUEUE && "bad".equals(e.message())) throw new IllegalStateException("enqueue boom");
      if (fault == Fault.ENQUEUE_OF_UNCAUGHT_REPORT && e.message() instanceof Logging.Error error && String.valueOf(error.message()).startsWith("Uncaught")) throw new IllegalStateException("enqueue boom");
      q.add(e);
    }
    public Envelope dequeue() {
      Envelope e = q.poll();
      if (e != null && "bad".equals(e.message())) {
        if (fault == Fault.DEQUEUE) throw new IllegalStateException("dequeue boom");
        if (fault == Fault.HAS_MESSAGES) armed.set(true);
      }
      return e;
    }
    public int numberOfMessages() { return q.size(); }
    public boolean hasMessages() {
      if (armed.compareAndSet(true, false)) throw new IllegalStateException("hasMessages boom");
      return !q.isEmpty();
    }
    public void cleanUp(ActorRef owner, MessageQueue deadLetters) {
      for (Envelope e; (e = q.poll()) != null; ) deadLetters.enqueue(owner, e);
    }
  }

  public static final class FaultyType implements MailboxType {
    public FaultyType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { return new FaultyQueue(); }
  }

  /** Configuration appended to the next {@link Env}. The cases run one after another. */
  static String extraConfig = "";

  /** An {@link Env} that publishes what a {@code LoggingAdapter} logs as errors (the pool's uncaught-error report does), printing nothing. */
  static Env errorLoggingEnv() {
    // with SLF4J on the classpath Pekko swaps in its filter, which has no binding here: `use-slf4j = off` keeps the default one
    // naming the standard output logger as the only logger starts none (`startDefaultLoggers` skips it)
    extraConfig = "pekko.loglevel = ERROR\n pekko.use-slf4j = off\n pekko.loggers = [\"org.apache.pekko.event.Logging$StandardOutLogger\"]\n";
    try { return new Env(5, "0ms"); } finally { extraConfig = ""; }
  }

  /** The actor {@code a} on a mailbox whose queue has {@code fault}. */
  static ActorRef faulty(Env e, Fault fault) throws Exception {
    nextFault = fault;
    ActorRef a = e.sys.actorOf(Props.create(Worker.class, e).withMailbox("faulty-mailbox"), "a");
    awaitUntil(() -> e.started.get() == 1);
    return a;
  }

  /** The uncaught-error report names the pool thread, whose number depends on scheduling. */
  static List<String> withoutThreadNumber(Collection<String> errors) {
    return errors.stream().map(x -> x.replaceAll("default-dispatcher-\\d+", "default-dispatcher-N")).collect(java.util.stream.Collectors.toList());
  }

  /** Creates the child {@code kid} with a mailbox type that throws, catches what {@code actorOf} throws, and reuses the name. */
  public static final class CatchingParent extends AbstractActor {
    final Env env; CatchingParent(Env env) { this.env = env; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> {
        try { getContext().actorOf(Props.create(Worker.class, env).withMailbox("failing-mailbox"), "kid"); env.note.set("created"); }
        catch (Throwable t) { env.note.set("caught=" + t.getClass().getName() + ": " + t.getMessage() + " cause=" + t.getCause()); }
        int registered = 0;
        for (ActorRef child : getContext().getChildren()) registered++;
        env.log.add("children=" + registered);
        getContext().actorOf(Props.create(Worker.class, env), "kid").tell("m", getSelf());
      }).build();
    }
  }

  // ---- real actors ---------------------------------------------------------------------------------------------------------------
  static final class Block { final CountDownLatch entered = new CountDownLatch(1), release = new CountDownLatch(1); }

  public static final class Worker extends AbstractActor {
    final Env env; Worker(Env env) { this.env = env; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(Block.class, b -> { enter(); b.entered.countDown(); b.release.await(); env.log.add("r:block"); leave(); })
          .match(String.class, s -> { enter(); if (s.startsWith("sleep")) Thread.sleep(60); env.log.add("r:" + s); leave(); })
          .match(Terminated.class, t -> env.log.add("terminated")).build();
    }
    void enter() { env.max.accumulateAndGet(env.inFlight.incrementAndGet(), Math::max); }
    void leave() { env.inFlight.decrementAndGet(); }
    @Override public void preStart() { env.started.incrementAndGet(); }
  }

  /** A worker of the MBX-ID cases: logs {@code label:message}; "block" holds its turn until the environment releases it. */
  public static class IdWorker extends AbstractActor {
    final String label; final IdEnv env;
    IdWorker(String label, IdEnv env) { this.label = label; this.env = env; }
    @Override public void preStart() { env.started.incrementAndGet(); }
    @Override public Receive createReceive() {
      return receiveBuilder().match(String.class, s -> {
        if (s.equals("block")) { env.entered.countDown(); env.release.await(); }
        env.log.add(label + ":" + s);
      }).build();
    }
  }

  /** Application-defined queue semantics ({@code RequiresMessageQueue[T]} with a type of the user): one that is bound in the requirements, one that is not. */
  public interface AuditTrail extends MessageQueue {}
  public interface Unmapped extends MessageQueue {}

  /** An unbounded FIFO queue that has the {@link AuditTrail} semantics. */
  static final class AuditTrailQueue implements MessageQueue, AuditTrail, UnboundedMessageQueueSemantics {
    final ConcurrentLinkedQueue<Envelope> q = new ConcurrentLinkedQueue<>();
    public void enqueue(ActorRef r, Envelope e) { q.add(e); }
    public Envelope dequeue() { return q.poll(); }
    public int numberOfMessages() { return q.size(); }
    public boolean hasMessages() { return !q.isEmpty(); }
    public void cleanUp(ActorRef owner, MessageQueue deadLetters) { for (Envelope e; (e = q.poll()) != null; ) deadLetters.enqueue(owner, e); }
  }

  public static final class AuditTrailType implements MailboxType, ProducesMessageQueue<AuditTrailQueue> {
    public AuditTrailType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { return new AuditTrailQueue(); }
  }

  /** Creates a queue that has the {@link AuditTrail} semantics but does not promise it ({@code ProducesMessageQueue} is absent). */
  public static final class PlainType implements MailboxType {
    public PlainType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { return new AuditTrailQueue(); }
  }

  /** Promises {@link AuditTrailQueue} but creates a {@link LifoQueue}, which has no {@link AuditTrail} semantics. */
  public static final class LiarType implements MailboxType, ProducesMessageQueue<AuditTrailQueue> {
    public LiarType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) { return new LifoQueue(); }
  }

  /** Creates {@code kid} from each {@link Props} it is sent and logs what it sees of it: the failure its supervision gets, its death. */
  public static final class InitParent extends AbstractActor {
    final IdEnv env; int attempt;
    InitParent(IdEnv env) { this.env = env; }
    @Override public SupervisorStrategy supervisorStrategy() {
      return new OneForOneStrategy(10, java.time.Duration.ofMinutes(1), t -> {
        env.log.add("failure:" + t.getClass().getName() + ": " + t.getMessage().replaceAll("#-?\\d+", ""));
        return SupervisorStrategy.stop();
      });
    }
    @Override public Receive createReceive() {
      return receiveBuilder().match(Props.class, p -> {
        String label = attempt++ == 0 ? "created" : "again";
        try { getContext().watch(getContext().actorOf(p, "kid")); env.log.add(label); }
        catch (Throwable t) { env.log.add(label + ":" + t.getClass().getName() + ": " + t.getMessage()); }
      }).match(Terminated.class, t -> env.log.add("terminated")).build();
    }
  }

  static final AtomicInteger DEFAULT_QUEUES = new AtomicInteger();

  /** A default mailbox that counts the queues of the three guardians (the system also makes `deadLetterListener` and `eventStreamUnsubscriber`, which Rakka has not); it promises the unbounded {@link LifoQueue}. */
  public static final class CountingDefaultType implements MailboxType, ProducesMessageQueue<LifoQueue> {
    public CountingDefaultType(ActorSystem.Settings s, Config c) {}
    public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) {
      String path = owner.isDefined() ? owner.get().path().toString() : "";
      if (path.equals("pekko://mbx/") || path.equals("pekko://mbx/user") || path.equals("pekko://mbx/system")) DEFAULT_QUEUES.incrementAndGet();
      return new LifoQueue();
    }
  }

  /** The workers of the {@code MBX-REQ} cases: the same worker, each declaring {@code RequiresMessageQueue[T]} with a different {@code T}. */
  public static final class UnboundedWorker extends IdWorker implements RequiresMessageQueue<UnboundedMessageQueueSemantics> {
    UnboundedWorker(String label, IdEnv env) { super(label, env); }
  }
  public static final class BoundedWorker extends IdWorker implements RequiresMessageQueue<BoundedMessageQueueSemantics> {
    BoundedWorker(String label, IdEnv env) { super(label, env); }
  }
  public static final class AuditTrailWorker extends IdWorker implements RequiresMessageQueue<AuditTrail> {
    AuditTrailWorker(String label, IdEnv env) { super(label, env); }
  }
  public static final class AnyQueueWorker extends IdWorker implements RequiresMessageQueue<MessageQueue> {
    AnyQueueWorker(String label, IdEnv env) { super(label, env); }
  }
  public static final class UnmappedWorker extends IdWorker implements RequiresMessageQueue<Unmapped> {
    UnmappedWorker(String label, IdEnv env) { super(label, env); }
  }

  /** Creates the child {@code kid} (props with the capacity-4 id) when told {@code spawn}. */
  public static final class IdParent extends AbstractActor {
    final IdEnv env; IdParent(IdEnv env) { this.env = env; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("spawn", s -> env.kid.set(getContext().actorOf(env.props("kid", "cap4-mailbox"), "kid"))).build();
    }
  }

  public static final class IdDeadListener extends AbstractActor {
    final IdEnv env; IdDeadListener(IdEnv env) { this.env = env; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(DeadLetter.class, d -> env.dead.add(d.recipient().path().name() + ":" + d.message())).build();
    }
  }

  /** A system with two NonBlockingBoundedMailbox ids of capacity 2 and 4; three actors may be blocked at once. */
  static final class IdEnv implements AutoCloseable {
    final ConcurrentLinkedQueue<String> log = new ConcurrentLinkedQueue<>(), dead = new ConcurrentLinkedQueue<>();
    final CountDownLatch entered, release = new CountDownLatch(1);
    final AtomicInteger started = new AtomicInteger();
    final AtomicReference<ActorRef> kid = new AtomicReference<>();
    final ActorSystem sys;
    IdEnv(int blockedWorkers) {
      entered = new CountDownLatch(blockedWorkers);
      sys = ActorSystem.create("mbx", ConfigFactory.parseString("pekko { loglevel = OFF\n stdout-loglevel = OFF\n log-dead-letters = off }\n"
          + "cap2-mailbox { mailbox-type = \"org.apache.pekko.dispatch.NonBlockingBoundedMailbox\"\n mailbox-capacity = 2 }\n"
          + "cap4-mailbox { mailbox-type = \"org.apache.pekko.dispatch.NonBlockingBoundedMailbox\"\n mailbox-capacity = 4 }\n")
          .withFallback(ConfigFactory.load()));
      sys.eventStream().subscribe(sys.actorOf(Props.create(IdDeadListener.class, this)), DeadLetter.class);
    }
    /** A system whose configuration is {@code hocon} over the base one; the listener has an unbounded mailbox of its own. */
    IdEnv(int blockedWorkers, String hocon) {
      entered = new CountDownLatch(blockedWorkers);
      sys = ActorSystem.create("mbx", ConfigFactory.parseString("pekko { loglevel = OFF\n stdout-loglevel = OFF\n log-dead-letters = off }\n"
          + "cap4-mailbox { mailbox-type = \"org.apache.pekko.dispatch.NonBlockingBoundedMailbox\"\n mailbox-capacity = 4 }\n" + hocon)
          .withFallback(ConfigFactory.load()));
      sys.eventStream().subscribe(sys.actorOf(Props.create(IdDeadListener.class, this).withMailbox("unbounded")), DeadLetter.class);
    }
    Props props(String label, String mailboxId) {
      Props p = Props.create(IdWorker.class, label, this);
      return mailboxId == null ? p : p.withMailbox(mailboxId);
    }
    Props requiring(Class<? extends IdWorker> worker, String label) { return Props.create(worker, label, this); }
    ActorRef spawn(String name, String mailboxId) throws Exception { return spawnProps(name, props(name, mailboxId)); }
    ActorRef spawnProps(String name, Props props) throws Exception {
      int before = started.get();
      ActorRef r = sys.actorOf(props, name);
      awaitUntil(() -> started.get() > before);
      return r;
    }
    /** What each actor received, in order, and what was dead-lettered to it, in order. */
    String perActor(String... labels) {
      StringBuilder out = new StringBuilder();
      for (String l : labels) {
        List<String> got = new ArrayList<>(), lost = new ArrayList<>();
        for (String s : log) if (s.startsWith(l + ":")) got.add(s.substring(l.length() + 1));
        for (String s : dead) if (s.startsWith(l + ":")) lost.add(s.substring(l.length() + 1));
        out.append(l).append(" received=").append(got).append(" deadLettered=").append(lost).append("; ");
      }
      return out.toString().trim();
    }
    @Override public void close() { release.countDown(); sys.terminate(); }
  }

  static final class Env implements AutoCloseable {
    final ConcurrentLinkedQueue<String> log = new ConcurrentLinkedQueue<>(), dead = new ConcurrentLinkedQueue<>(), errors = new ConcurrentLinkedQueue<>();
    final AtomicInteger inFlight = new AtomicInteger(), max = new AtomicInteger();
    final AtomicInteger started = new AtomicInteger();
    final AtomicReference<String> note = new AtomicReference<>();
    final CountDownLatch release = new CountDownLatch(1); // what a LifeActor with `hold` waits for in postStop
    final ActorSystem sys;
    final ConcurrentLinkedQueue<String> warnings = new ConcurrentLinkedQueue<>();
    Env(int throughput, String deadline) { this(throughput, deadline, null); }
    Env(int throughput, String deadline, String pushTimeout) {
      sys = ActorSystem.create("mbx", ConfigFactory.parseString("pekko { loglevel = OFF\n stdout-loglevel = OFF\n log-dead-letters = off\n"
          + (pushTimeout == null ? "" : " actor.unstarted-push-timeout = " + pushTimeout + "\n") + " }\n"
          + "t-disp { type = Dispatcher\n executor = thread-pool-executor\n thread-pool-executor.fixed-pool-size = 1\n throughput = " + throughput
          + "\n throughput-deadline-time = " + deadline + " }\n hook-mailbox.mailbox-type = \"pilot.MailboxOracle$HookType\"\n"
          + "failing-mailbox.mailbox-type = \"pilot.MailboxOracle$FailingType\"\n"
          + "faulty-mailbox.mailbox-type = \"pilot.MailboxOracle$FaultyType\"\n"
          + "rej-disp { type = Dispatcher\n executor = \"pilot.MailboxOracle$RejectingConfigurator\"\n throughput = " + throughput + " }\n"
          + extraConfig)
          .withFallback(ConfigFactory.load()));
      ActorRef listener = sys.actorOf(Props.create(Listener.class, dead));
      sys.eventStream().subscribe(listener, DeadLetter.class);
      sys.eventStream().subscribe(sys.actorOf(Props.create(ErrorListener.class, errors)), Logging.Error.class);
      if (pushTimeout != null) sys.eventStream().subscribe(sys.actorOf(Props.create(WarningListener.class, warnings)), Logging.Warning.class);
    }
    ActorRef spawn(String name, Hooks hooks) throws Exception { return spawnOn("t-disp", name, hooks); }
    ActorRef spawnOn(String dispatcher, String name) throws Exception { return spawnOn(dispatcher, name, null); }
    ActorRef spawnOn(String dispatcher, String name, Hooks hooks) throws Exception {
      int before = started.get();
      nextHooks = hooks == null ? new Hooks() : hooks;
      ActorRef r = sys.actorOf(Props.create(Worker.class, this).withDispatcher(dispatcher).withMailbox("hook-mailbox"), name);
      awaitUntil(() -> started.get() > before);
      return r;
    }
    @Override public void close() { release.countDown(); sys.terminate(); }
  }

  public static final class Listener extends AbstractActor {
    final Collection<String> dead; Listener(Collection<String> dead) { this.dead = dead; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(DeadLetter.class, d -> dead.add(d.message() instanceof String ? (String) d.message() : d.message().getClass().getSimpleName())).build();
    }
  }

  /** Records `logSource|cause class|message` of every {@code Logging.Error} published on the event stream. */
  public static final class ErrorListener extends AbstractActor {
    final Collection<String> errors; ErrorListener(Collection<String> errors) { this.errors = errors; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(Logging.Error.class, e -> errors.add(e.logSource() + "|" + e.cause().getClass().getName() + "|" + e.message())).build();
    }
  }

  /** The number of upcoming {@code execute} calls of {@code rej-disp}'s executor that throw {@code RejectedExecutionException}. */
  static final AtomicInteger REJECT_NEXT = new AtomicInteger();
  // Static state shared by every case that uses rej-disp: valid only because the cases run sequentially (never run two in one JVM concurrently).
  /** When set, the next task {@code rej-disp}'s executor accepts is run, and its submitter then waits for this latch (`SUBMITTED_AND_HELD` says it does). */
  static final AtomicReference<CountDownLatch> HOLD_SUBMITTER_AFTER_NEXT_EXECUTE = new AtomicReference<>();
  static volatile CountDownLatch SUBMITTED_AND_HELD = new CountDownLatch(1);

  /** A public-SPI executor (one thread) that rejects the next {@link #REJECT_NEXT} tasks, so the rejection path of
   *  {@code Dispatcher.registerForExecution} is exercised without patching Pekko. */
  public static final class RejectingConfigurator extends ExecutorServiceConfigurator {
    public RejectingConfigurator(Config config, DispatcherPrerequisites prerequisites) { super(config, prerequisites); }
    @Override public ExecutorServiceFactory createExecutorServiceFactory(String id, ThreadFactory threadFactory) {
      return new ExecutorServiceFactory() {
        @Override public ExecutorService createExecutorService() {
          return new ThreadPoolExecutor(1, 1, 0L, TimeUnit.SECONDS, new LinkedBlockingQueue<>(), threadFactory) {
            @Override public void execute(Runnable task) {
              if (REJECT_NEXT.getAndUpdate(n -> n > 0 ? n - 1 : 0) > 0) throw new RejectedExecutionException("injected");
              super.execute(task);
              // the next submission is accepted and then holds the submitting thread until released: the task runs, its submitter does not move on
              CountDownLatch hold = HOLD_SUBMITTER_AFTER_NEXT_EXECUTE.getAndSet(null);
              if (hold != null) {
                SUBMITTED_AND_HELD.countDown();
                try { hold.await(); } catch (InterruptedException x) { Thread.currentThread().interrupt(); }
              }
            }
          };
        }
      };
    }
  }

  /** Creates a child inside its own turn with {@code attachChild}, the asynchronous path (what {@code system.actorOf} uses): the parent
   *  attaches the real cell when it processes {@code Supervise}, which happens after this handler returns, so everything done here
   *  reaches the child's {@code UnstartedCell}. (A plain {@code context.actorOf} is synchronous and would race the child's first turn.) */
  public static final class Spawner extends AbstractActor {
    final Env env; final ActorRef watcher; final String script; final AtomicReference<ActorRef> child;
    Spawner(Env env, ActorRef watcher, String script, AtomicReference<ActorRef> child) { this.env = env; this.watcher = watcher; this.script = script; this.child = child; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(Terminated.class, t -> env.log.add("terminated")).matchEquals("go", g -> {
        nextHooks = new Hooks();
        // `context.actorOf` is synchronous in Pekko (a LocalActorRef is attached at once); `attachChild` is the asynchronous path that
        // `system.actorOf` takes: the new cell is attached when this actor processes Supervise, after this handler returns
        ActorRef c = ((ActorCell) getContext()).attachChild(Props.create(Worker.class, env).withMailbox("hook-mailbox"), "c", false);
        switch (script) {
          case "queued": c.tell("m1", ActorRef.noSender()); c.tell("m2", ActorRef.noSender()); in(c).suspend(); break;
          case "watched": getContext().watch(c); in(c).stop(); break;
          case "unwatched": getContext().watch(c); getContext().unwatch(c); in(c).stop(); break;
          case "visibility": {
            String dup;
            try { getContext().actorOf(Props.create(Worker.class, env).withMailbox("hook-mailbox"), "c"); dup = "none"; }
            catch (InvalidActorNameException x) { dup = x.toString(); }
            env.note.set("startedInsideTurn=" + env.started.get() + " childFound=" + getContext().findChild("c").isPresent()
                + " sameRef=" + getContext().findChild("c").map(r -> r.equals(c)).orElse(false) + " path=" + c.path() + " duplicate=" + dup);
            break;
          }
          default: throw new IllegalStateException(script);
        }
        child.set(c);
      }).build();
    }
  }

  /** Creates a child with the synchronous {@code context.actorOf} (a {@code LocalActorRef}, {@code makeChild(async = false)}) on the rejecting
   *  dispatcher. {@code caught}: the actor handles what {@code actorOf} throws; otherwise it escapes and fails the actor. */
  public static final class SyncSpawner extends AbstractActor {
    final Env env; final boolean caught; SyncSpawner(Env env, boolean caught) { this.env = env; this.caught = caught; }
    @Override public void preStart() { env.started.incrementAndGet(); }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> {
        try { getContext().actorOf(Props.create(Listener.class, env.dead).withDispatcher("rej-disp"), "c"); env.note.set("thrown=none"); }
        catch (Throwable t) { env.note.set("thrown=" + t.getClass().getName()); if (!caught) throw t; }
      }).matchEquals("again", a -> {
        try { getContext().actorOf(Props.create(Listener.class, env.dead), "c"); env.note.set("again=ok"); }
        catch (Throwable t) { env.note.set("again=" + t.getClass().getName()); }
      }).matchEquals("who", w -> env.note.set("children=" + getContext().findChild("c").isPresent())).build();
    }
  }

  /** Records `logSource|logClass|message` of every {@code Logging.Warning} published on the event stream. */
  public static final class WarningListener extends AbstractActor {
    final Collection<String> warnings; WarningListener(Collection<String> warnings) { this.warnings = warnings; }
    @Override public Receive createReceive() {
      return receiveBuilder().match(Logging.Warning.class, w -> warnings.add(w.logSource() + "|" + w.logClass().getName() + "|" + w.message())).build();
    }
  }

  /** Blocks the hand-over of "m1" into the real mailbox: {@code UnstartedCell.replaceWith} keeps its lock while the hook waits. */
  static final class BlockingHooks extends Hooks {
    final CountDownLatch entered = new CountDownLatch(1), release = new CountDownLatch(1);
    @Override void onEnqueue(Object msg) throws Exception { if ("m1".equals(msg)) { entered.countDown(); release.await(); } }
  }

  /** While "m1" is handed over it sends "m2" to the same reference, on the same thread: the lock of the unstarted cell is reentrant. */
  static final class ReentrantHooks extends Hooks {
    final AtomicReference<ActorRef> child; ReentrantHooks(AtomicReference<ActorRef> child) { this.child = child; }
    @Override void onEnqueue(Object msg) { if ("m1".equals(msg)) child.get().tell("m2", ActorRef.noSender()); }
  }

  /** Creates a child with {@code attachChild} (the asynchronous path) and sends it "m1" while it is unstarted; the hand-over of "m1"
   *  then blocks in {@link BlockingHooks} while this actor processes {@code Supervise}. */
  public static final class PushSpawner extends AbstractActor {
    final Env env; final Hooks hooks; final AtomicReference<ActorRef> child;
    PushSpawner(Env env, Hooks hooks, AtomicReference<ActorRef> child) { this.env = env; this.hooks = hooks; this.child = child; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> {
        nextHooks = hooks;
        ActorRef c = ((ActorCell) getContext()).attachChild(Props.create(Worker.class, env).withMailbox("hook-mailbox"), "c", false);
        c.tell("m1", ActorRef.noSender());
        child.set(c);
      }).build();
    }
  }

  static ActorRef pushChild(Env e, BlockingHooks h) {
    AtomicReference<ActorRef> child = new AtomicReference<>();
    e.sys.actorOf(Props.create(PushSpawner.class, e, h, child), "spawner").tell("go", ActorRef.noSender());
    awaitUntil(() -> h.entered.getCount() == 0); // the hand-over is in progress and holds the lock
    return child.get();
  }

  static void pause(long ms) { try { Thread.sleep(ms); } catch (InterruptedException x) { throw new RuntimeException(x); } } // a bounded wait for an absence

  /** The errors published by cells themselves: the supervisor's {@code logFailure} lines carry the failure's own class and are left out. */
  static List<String> cellErrors(Env e) {
    List<String> l = new ArrayList<>();
    for (String x : e.errors) if (!x.contains("|org.apache.pekko.actor.ActorInitializationException|")) l.add(x);
    return l;
  }

  static void awaitChildStarted(AtomicReference<ActorRef> child) { awaitUntil(() -> child.get() != null); box(child.get()); }

  static void awaitUntil(BooleanSupplier c) {
    long end = System.nanoTime() + 10_000_000_000L;
    while (!c.getAsBoolean()) { if (System.nanoTime() > end) throw new IllegalStateException("condition not reached"); Thread.onSpinWait(); }
  }

  /** The mailbox of the started actor: a RepointableActorRef answers with its UnstartedCell until the real ActorCell is attached. */
  static Mailbox box(ActorRef r) {
    awaitUntil(() -> ((ActorRefWithCell) r).underlying() instanceof ActorCell);
    return ((ActorCell) ((ActorRefWithCell) r).underlying()).mailbox();
  }
  static InternalActorRef in(ActorRef r) { return (InternalActorRef) r; }
  static String view(Mailbox m) { return st(m).replace(" raw=" + m.currentStatus(), "") + " queued=" + m.numberOfMessages(); }
  static void idle(ActorRef r) { awaitUntil(() -> !box(r).isScheduled()); }
  static void sendAll(ActorRef r, String... ms) { for (String m : ms) r.tell(m, ActorRef.noSender()); }


  // ---- Q-MBX-11 slice C4: termination, restart and lookup of actors that have not started -----------------------------------------
  /** What a {@link UnstartedParent} does, and what the harness waits for. The parent blocks inside its own turn (`go`) before it creates `c`
   *  (so the harness can first send it system messages, which then precede the `Supervise` of `c`) and, optionally, after (so the harness
   *  can look at `c` while it is still unstarted: the parent processes `Supervise` only when this turn ends). */
  static final class Plan {
    final CountDownLatch entered = new CountDownLatch(1), proceed = new CountDownLatch(1), created = new CountDownLatch(1), finish = new CountDownLatch(1);
    final AtomicReference<ActorRef> child = new AtomicReference<>();
    final AtomicInteger instances = new AtomicInteger();
    volatile Hooks hooks;
    volatile List<String> messages = List.of();
    volatile boolean childMakesChild, parentHoldsChild, holdAfterCreate;
    volatile String childDispatcher = "pekko.actor.default-dispatcher";
  }

  /** `label` records its lifecycle in the environment's log; with {@code makesChild} it creates `k` in preStart; with {@code hold} its
   *  postStop waits for {@code Env.release} (a started child that keeps its parent waiting for it to die). */
  public static final class LifeActor extends AbstractActor {
    final Env env; final String label; final boolean makesChild, hold;
    LifeActor(Env env, String label, boolean makesChild, boolean hold) { this.env = env; this.label = label; this.makesChild = makesChild; this.hold = hold; }
    @Override public void preStart() {
      env.log.add(label + ":preStart");
      if (makesChild) { getContext().actorOf(Props.create(Scenario.Noop.class), "k"); env.log.add(label + ":made-k"); }
    }
    @Override public void postStop() throws Exception {
      env.log.add(label + ":postStop");
      if (hold) { env.release.await(); env.log.add(label + ":postStop-released"); }
    }
    @Override public Receive createReceive() { return receiveBuilder().matchAny(m -> env.log.add(label + ":r:" + m)).build(); }
  }

  /** Creates `c` with `attachChild` (the path of `system.actorOf`) inside its turn, so `c` stays unstarted until the turn ends. */
  public static final class UnstartedParent extends AbstractActor {
    final Env env; final Plan plan;
    UnstartedParent(Env env, Plan plan) { this.env = env; this.plan = plan; }
    @Override public void preStart() {
      int n = plan.instances.incrementAndGet();
      env.log.add("P:preStart");
      if (n == 1 && plan.parentHoldsChild) getContext().actorOf(Props.create(LifeActor.class, env, "k1", false, true), "k1");
    }
    @Override public void preRestart(Throwable reason, java.util.Optional<Object> message) throws Exception { env.log.add("P:preRestart"); super.preRestart(reason, message); }
    @Override public void postStop() { env.log.add("P:postStop"); }
    @Override public Receive createReceive() {
      return receiveBuilder().matchEquals("go", g -> {
        plan.entered.countDown();
        plan.proceed.await();
        nextHooks = plan.hooks != null ? plan.hooks : new Hooks();
        ActorRef c = ((ActorCell) getContext()).attachChild(Props.create(LifeActor.class, env, "c", plan.childMakesChild, false).withDispatcher(plan.childDispatcher).withMailbox("hook-mailbox"), "c", false);
        plan.child.set(c);
        for (String m : plan.messages) c.tell(m, ActorRef.noSender());
        plan.created.countDown();
        if (plan.holdAfterCreate) plan.finish.await();
      }).build();
    }
  }

  static String path(ActorRef r) { return r.path().toString(); }

  /** What can be asked of an actor reference, without taking the unstarted cell's lock (which a hand-over in progress holds). */
  static String lookupView(ActorRef c) {
    ActorRefWithCell rc = (ActorRefWithCell) c;
    List<String> kids = new ArrayList<>();
    for (ActorRef k : scala.jdk.javaapi.CollectionConverters.asJava(rc.children())) kids.add(path(k));
    return "isStarted=" + ((RepointableRef) c).isStarted() + " underlying=" + rc.underlying().getClass().getSimpleName() + " children=" + kids
        + " singleChild(k)=" + path(rc.getSingleChild("k")) + " viaPath(k)=" + path(in(c).getChild(scala.jdk.javaapi.CollectionConverters.asScala(List.of("k").iterator())))
        + " viaPath(none)=" + path(in(c).getChild(scala.jdk.javaapi.CollectionConverters.asScala(List.of("none").iterator())));
  }

  /** What an unstarted reference answers, taking its lock (do not call while a hand-over is blocked). */
  static String stateView(ActorRef c) {
    ActorRefWithCell rc = (ActorRefWithCell) c;
    return "isStarted=" + ((RepointableRef) c).isStarted() + " underlying=" + rc.underlying().getClass().getSimpleName() + " isTerminated=" + in(c).isTerminated()
        + " hasMessages=" + rc.underlying().hasMessages() + " queued=" + rc.underlying().numberOfMessages();
  }

  static List<String> ignoring(Collection<String> log, String prefix) { List<String> l = new ArrayList<>(); for (String x : log) if (!x.startsWith(prefix)) l.add(x); return l; }
  static List<String> only(Collection<String> log, String prefix) { List<String> l = new ArrayList<>(); for (String x : log) if (x.startsWith(prefix)) l.add(x); return l; }

  static void absence() { try { Thread.sleep(500); } catch (InterruptedException x) { throw new RuntimeException(x); } } // a bounded wait for an absence

  /** The parent `p` of a {@link Plan}, started, with its turn `go` entered and blocked before `c` is created. */
  static ActorRef parentInGo(Env e, Plan plan) throws Exception {
    ActorRef p = e.sys.actorOf(Props.create(UnstartedParent.class, e, plan), "p");
    awaitUntil(() -> e.log.contains("P:preStart"));
    p.tell("go", ActorRef.noSender());
    plan.entered.await();
    return p;
  }

  // ---- cases ---------------------------------------------------------------------------------------------------------------------
  private static boolean registered;
  private static synchronized void register() {
    if (registered) return;
    registered = true;

    run("MBX-ST-01 open to scheduled", () -> { Probe m = new Probe(); return "first=" + m.setAsScheduled() + " second=" + m.setAsScheduled() + " " + st(m); });
    run("MBX-ST-02 scheduled to idle", () -> {
      Probe m = new Probe(); m.setAsScheduled();
      return "idle=" + m.setAsIdle() + " " + st(m) + " | again=" + m.setAsIdle() + " " + st(m) + " | reschedule=" + m.setAsScheduled();
    });
    run("MBX-ST-03 open to suspended and resumed", () -> {
      Probe m = new Probe();
      return "suspend1=" + m.suspend() + " suspend2=" + m.suspend() + " " + st(m) + " | resume1=" + m.resume() + " " + st(m) + " resume2=" + m.resume() + " " + st(m)
          + " | resumeAtZero=" + m.resume() + " " + st(m);
    });
    run("MBX-ST-04 suspended and scheduled", () -> {
      Probe m = new Probe(); m.suspend();
      String s = "schedule=" + m.setAsScheduled() + " " + st(m) + " canBe(f,f)=" + m.canBeScheduledForExecution(false, false) + " canBe(f,sys)=" + m.canBeScheduledForExecution(false, true);
      return s + " | resume=" + m.resume() + " " + st(m);
    });
    run("MBX-ST-05 open to closed", () -> {
      Probe m = new Probe(); m.setAsScheduled(); m.suspend(); m.suspend();
      return "before " + st(m) + " | close1=" + m.becomeClosed() + " " + st(m) + " close2=" + m.becomeClosed() + " " + st(m);
    });
    run("MBX-ST-06 closed ignores resume and suspend", () -> {
      Probe m = new Probe(); m.becomeClosed();
      return "resume=" + m.resume() + " suspend=" + m.suspend() + " " + st(m);
    });
    run("MBX-ST-07 closed refuses scheduling", () -> {
      Probe m = new Probe(); m.becomeClosed();
      return "schedule=" + m.setAsScheduled() + " idle=" + m.setAsIdle() + " " + st(m) + " canBe=" + m.canBeScheduledForExecution(true, true);
    });
    run("MBX-ST-08 canBeScheduledForExecution matrix", () -> {
      StringBuilder sb = new StringBuilder();
      String[] names = {"open", "scheduled", "suspended", "suspended+scheduled", "closed"};
      for (int k = 0; k < names.length; k++) {
        Probe m = new Probe();
        if (k == 1 || k == 3) m.setAsScheduled();
        if (k == 2 || k == 3) m.suspend();
        if (k == 4) m.becomeClosed();
        sb.append(names[k]).append(": ");
        for (boolean hm : new boolean[] {false, true}) for (boolean hs : new boolean[] {false, true}) sb.append(m.canBeScheduledForExecution(hm, hs) ? 'T' : 'F');
        m.systemEnqueue(null, new Suspend());
        sb.append(" withSystemMessage=").append(m.canBeScheduledForExecution(false, false)).append("; ");
      }
      return sb.toString().trim();
    });
    run("MBX-MQ-01 default queue is FIFO and counts", () -> {
      MessageQueue q = defaultQueue(); StringBuilder sb = new StringBuilder("new: " + counts(q) + " | enqueue");
      for (String m : new String[] {"a", "b", "c"}) { q.enqueue(Nobody$.MODULE$, Envelope.apply(m, Nobody$.MODULE$, null)); sb.append(' ').append(counts(q)); }
      return sb + " | dequeue " + drain(q) + " then " + (q.dequeue() == null ? "null" : "message") + " | after " + counts(q);
    });
    run("MBX-MQ-02 default queue stays FIFO when enqueue and dequeue interleave", () -> {
      MessageQueue q = defaultQueue();
      q.enqueue(Nobody$.MODULE$, Envelope.apply("a", Nobody$.MODULE$, null)); q.enqueue(Nobody$.MODULE$, Envelope.apply("b", Nobody$.MODULE$, null));
      String first = (String) q.dequeue().message();
      q.enqueue(Nobody$.MODULE$, Envelope.apply("c", Nobody$.MODULE$, null));
      return "first=" + first + " rest=" + drain(q);
    });
    run("MBX-MQ-03 cleanUp moves everything to the dead-letters queue in order", () -> {
      MessageQueue q = defaultQueue(), dead = defaultQueue();
      for (String m : new String[] {"a", "b", "c"}) q.enqueue(Nobody$.MODULE$, Envelope.apply(m, Nobody$.MODULE$, null));
      q.cleanUp(Nobody$.MODULE$, dead);
      return "source " + counts(q) + " | target " + counts(dead) + " order " + drain(dead);
    });
    run("MBX-SQ-01 system queue drains earliest first", () -> {
      Probe m = new Probe();
      m.systemEnqueue(null, new Suspend()); m.systemEnqueue(null, new Resume(null)); m.systemEnqueue(null, new Terminate());
      boolean has = m.hasSystemMessages();
      String first = names(m.systemDrain(null));
      return "has=" + has + " drain=" + first + " | after has=" + m.hasSystemMessages() + " drain=" + names(m.systemDrain(null));
    });
    run("MBX-SQ-02 sentinel NoMessage closes the system queue", () -> {
      Probe m = new Probe(); m.systemEnqueue(null, new Suspend());
      String d = names(m.systemDrain(NoMessage$.MODULE$));
      m.systemEnqueue(null, new Resume(null));
      return "drain=" + d + " | afterSentinel has=" + m.hasSystemMessages() + " drain=" + names(m.systemDrain(null)) + " (actor==null: dropped, not forwarded)";
    });

    run("MBX-TK-01 Mailbox is a plain Runnable, not a ForkJoinTask", () ->
        "forkJoinTask=" + java.util.concurrent.ForkJoinTask.class.isAssignableFrom(Mailbox.class) + " runnable=" + Runnable.class.isAssignableFrom(Mailbox.class));
    run("MBX-OR-01 user messages are FIFO", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null);
        sendAll(a, "1", "2", "3", "4", "5", "6", "7", "8");
        awaitUntil(() -> e.log.size() == 8);
        return e.log.toString();
      }
    });
    run("MBX-OR-02 system message queued behind user messages runs before them, drained after the turn's message", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null); Block b = new Block();
        a.tell(b, ActorRef.noSender()); b.entered.await();
        String turn = view(box(a));
        sendAll(a, "u2", "u3"); in(a).suspend();
        String queued = view(box(a));
        b.release.countDown(); idle(a);
        String afterTurn = view(box(a)) + " log=" + e.log;
        in(a).resume(null); awaitUntil(() -> e.log.size() == 3); idle(a);
        return "inTurn: " + turn + " | queued: " + queued + " | afterTurn: " + afterTurn + " | afterResume: " + view(box(a)) + " log=" + e.log;
      }
    });
    run("MBX-OR-03 suspension keeps user messages pending; nested suspend needs balanced resume", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null);
        in(a).suspend(); in(a).suspend(); sendAll(a, "u1", "u2");
        awaitUntil(() -> box(a).suspendCount() == 2); idle(a);
        String suspended = view(box(a)) + " log=" + e.log;
        in(a).resume(null); awaitUntil(() -> box(a).suspendCount() == 1); idle(a);
        String half = view(box(a)) + " log=" + e.log;
        in(a).resume(null); awaitUntil(() -> e.log.size() == 2); idle(a);
        return "suspended: " + suspended + " | oneResume: " + half + " | twoResumes: " + view(box(a)) + " log=" + e.log;
      }
    });
    run("MBX-OR-04 messages sent while terminating and after termination go to dead letters", () -> {
      try (Env e = new Env(5, "0ms")) {
        Hooks h = new Hooks(); ActorRef a = e.spawn("w", h); Block b = new Block(); Mailbox old = box(a);
        a.tell(b, ActorRef.noSender()); b.entered.await();
        sendAll(a, "u2", "u3"); in(a).stop();
        sendAll(a, "u4");
        b.release.countDown(); h.cleaned.await(); awaitUntil(() -> in(a).isTerminated());
        sendAll(a, "after");
        awaitUntil(() -> e.dead.size() == 4);
        return "received=" + e.log + " deadLetters=" + e.dead + " oldMailbox=" + view(old);
      }
    });
    run("MBX-OR-05 system messages after close go to dead letters (Watch is answered)", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null), watcher = e.sys.actorOf(Props.create(Worker.class, e));
        Mailbox old = box(a); in(a).stop(); awaitUntil(() -> in(a).isTerminated() && old.isClosed());
        in(a).sendSystemMessage(new Suspend()); in(a).sendSystemMessage(new Resume(null));
        in(watcher).sendSystemMessage(new Watch(in(a), in(watcher)));
        awaitUntil(() -> e.dead.size() == 2 && e.log.contains("terminated"));
        return "deadLetters=" + e.dead + " watcherLog=" + e.log;
      }
    });
    run("MBX-DL-01 watching deadLetters is not answered, watching a stopped actor is", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null), watcher = e.sys.actorOf(Props.create(Worker.class, e));
        in(a).stop(); awaitUntil(() -> in(a).isTerminated());
        // the watcher watches deadLetters first (DeadLetterActorRef.specialHandle ignores a Watch whose watchee is itself), then the stopped actor
        in(watcher).sendSystemMessage(new Watch(in(e.sys.deadLetters()), in(watcher)));
        in(watcher).sendSystemMessage(new Watch(in(a), in(watcher)));
        awaitUntil(() -> e.log.size() >= 1); idle(watcher);
        return "watcherLog=" + e.log;
      }
    });
    run("MBX-RJ-01 one rejection of the executor is retried once and the message is delivered", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawnOn("rej-disp", "w"); idle(a); // the start-up turn is over: the next send must go through the executor
        REJECT_NEXT.set(1); a.tell("m", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 1); idle(a);
        return "log=" + e.log + " errors=" + e.errors + " rejectionsLeft=" + REJECT_NEXT.get() + " " + view(box(a));
      }
    });
    run("MBX-RJ-02 two rejections: Error published, mailbox idle, message kept and delivered by the next send", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawnOn("rej-disp", "w"); idle(a); // the start-up turn is over: the next send must go through the executor
        REJECT_NEXT.set(2); a.tell("m", ActorRef.noSender());
        awaitUntil(() -> e.errors.size() == 2);
        String afterRejection = "errors=" + e.errors + " " + view(box(a)) + " log=" + e.log + " rejectionsLeft=" + REJECT_NEXT.get();
        a.tell("n", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 2); idle(a);
        return afterRejection + " | next send: log=" + e.log + " errors=" + e.errors.size() + " " + view(box(a));
      }
    });
    run("MBX-RJ-03 a system message whose scheduling is rejected twice stays queued and runs with the next send", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawnOn("rej-disp", "w"); idle(a); // the start-up turn is over: the next send must go through the executor
        REJECT_NEXT.set(2); in(a).sendSystemMessage(new Suspend());
        awaitUntil(() -> e.errors.size() == 2);
        String afterRejection = "errors=" + e.errors + " " + view(box(a)) + " systemPending=" + box(a).hasSystemMessages();
        a.tell("n", ActorRef.noSender());
        awaitUntil(() -> box(a).suspendCount() == 1); idle(a);
        return afterRejection + " | next send: " + view(box(a)) + " systemPending=" + box(a).hasSystemMessages() + " log=" + e.log;
      }
    });
    run("MBX-RJ-04 a Terminate whose scheduling is rejected twice terminates the actor with the next send; the message is dead-lettered", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawnOn("rej-disp", "w"); idle(a); // the start-up turn is over: the next send must go through the executor
        REJECT_NEXT.set(2); e.sys.stop(a);
        awaitUntil(() -> e.errors.size() == 2);
        String afterRejection = "errors=" + e.errors + " terminated=" + in(a).isTerminated() + " systemPending=" + box(a).hasSystemMessages();
        a.tell("m", ActorRef.noSender());
        awaitUntil(() -> in(a).isTerminated() && e.dead.size() == 1);
        return afterRejection + " | next send: terminated=" + in(a).isTerminated() + " deadLetters=" + e.dead + " log=" + e.log;
      }
    });
    run("MBX-RJ-05 an actor whose attach is rejected twice: actorOf returns, the supervisor fails and its children stop, the new actor never starts", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef sibling = e.spawn("sib", null);
        REJECT_NEXT.set(2);
        // actorOf is asynchronous: the supervisor attaches the new cell while handling Supervise (RepointableActorRef.point)
        ActorRef a = e.sys.actorOf(Props.create(Worker.class, e).withDispatcher("rej-disp").withMailbox("hook-mailbox"), "a");
        awaitUntil(() -> e.errors.size() >= 1 && in(sibling).isTerminated());
        try { Thread.sleep(500); } catch (InterruptedException x) { throw new RuntimeException(x); } // a bounded wait for an absence
        return "errors=" + e.errors + " refReturned=" + (a != null) + " newActorTerminated=" + in(a).isTerminated()
            + " siblingTerminated=" + in(sibling).isTerminated() + " startedActors=" + e.started.get() + " rejectionsLeft=" + REJECT_NEXT.get();
      }
    });
    run("MBX-RJ-06 a synchronous context.actorOf whose attach is rejected twice: the caller gets the exception and the name stays taken", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef p = e.sys.actorOf(Props.create(SyncSpawner.class, e, true), "p");
        awaitUntil(() -> e.started.get() == 1);
        REJECT_NEXT.set(2);
        p.tell("go", ActorRef.noSender());
        awaitUntil(() -> e.note.get() != null);
        String thrown = e.note.get();
        try { Thread.sleep(500); } catch (InterruptedException x) { throw new RuntimeException(x); } // a bounded wait for an absence
        String errors = e.errors.toString();
        e.note.set(null); p.tell("again", ActorRef.noSender()); awaitUntil(() -> e.note.get() != null);
        return thrown + " errors=" + errors + " rejectionsLeft=" + REJECT_NEXT.get() + " | " + e.note.get() + " startedActors=" + e.started.get() + " parentTerminated=" + in(p).isTerminated();
      }
    });
    run("MBX-RJ-07 a synchronous context.actorOf whose attach is rejected twice and whose exception escapes: the parent fails and restarts", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef p = e.sys.actorOf(Props.create(SyncSpawner.class, e, false), "p");
        awaitUntil(() -> e.started.get() == 1);
        REJECT_NEXT.set(2);
        p.tell("go", ActorRef.noSender());
        awaitUntil(() -> e.started.get() == 2);
        try { Thread.sleep(500); } catch (InterruptedException x) { throw new RuntimeException(x); } // a bounded wait for an absence
        e.note.set(null); p.tell("again", ActorRef.noSender()); awaitUntil(() -> e.note.get() != null);
        // the supervisor's logFailure line (AC-SUP-018, another contract) is left out: only what the dispatcher publishes
        List<String> dispatcherErrors = new ArrayList<>();
        for (String x : e.errors) if (x.startsWith("org.apache.pekko.dispatch.Dispatcher|")) dispatcherErrors.add(x);
        return e.note.get() + " dispatcherErrors=" + dispatcherErrors + " rejectionsLeft=" + REJECT_NEXT.get() + " startedActors=" + e.started.get() + " parentTerminated=" + in(p).isTerminated();
      }
    });
    run("MBX-PT-01 a send that waits for the unstarted cell's lock longer than unstarted-push-timeout is a dead letter with a Warning", () -> {
      try (Env e = new Env(5, "0ms", "300ms")) {
        BlockingHooks h = new BlockingHooks(); ActorRef c = pushChild(e, h);
        long t0 = System.nanoTime(); c.tell("m2", ActorRef.noSender()); long ms = (System.nanoTime() - t0) / 1_000_000;
        awaitUntil(() -> !e.dead.isEmpty() && !e.warnings.isEmpty());
        h.release.countDown(); awaitUntil(() -> e.log.size() == 1); pause(300);
        return "waitedTheTimeout=" + (ms >= 300) + " returnedWithinBound=" + (ms < 5000) + " deadLetters=" + e.dead + " warnings=" + e.warnings + " log=" + e.log;
      }
    });
    run("MBX-PT-02 a send that waits for the lock while the hand-over ends before the timeout is delivered after the queued messages", () -> {
      try (Env e = new Env(5, "0ms", "10s")) {
        BlockingHooks h = new BlockingHooks(); ActorRef c = pushChild(e, h);
        Thread t = new Thread(() -> c.tell("m2", ActorRef.noSender())); t.start();
        awaitUntil(() -> t.getState() == Thread.State.TIMED_WAITING);
        h.release.countDown(); try { t.join(); } catch (InterruptedException x) { throw new RuntimeException(x); }
        awaitUntil(() -> e.log.size() == 2); pause(300);
        return "log=" + e.log + " deadLetters=" + e.dead + " warnings=" + e.warnings;
      }
    });
    run("MBX-PT-03 a system message waits for the lock without a timeout and is delivered once the hand-over ends", () -> {
      try (Env e = new Env(5, "0ms", "300ms")) {
        BlockingHooks h = new BlockingHooks(); ActorRef c = pushChild(e, h);
        Thread t = new Thread(() -> in(c).suspend()); t.start();
        awaitUntil(() -> t.getState() == Thread.State.WAITING);
        pause(1000); // three timeouts
        boolean stillBlocked = t.getState() == Thread.State.WAITING;
        h.release.countDown(); try { t.join(); } catch (InterruptedException x) { throw new RuntimeException(x); }
        awaitUntil(() -> box(c).suspendCount() == 1);
        return "stillBlockedAfterThreeTimeouts=" + stillBlocked + " suspendCount=" + box(c).suspendCount() + " deadLetters=" + e.dead + " warnings=" + e.warnings;
      }
    });
    run("MBX-PT-06 the thread that hands the queue over can send to the same reference: its message joins the queue, not the timeout", () -> {
      try (Env e = new Env(5, "0ms", "300ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(PushSpawner.class, e, new ReentrantHooks(child), child), "spawner").tell("go", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 2); pause(300);
        return "log=" + e.log + " deadLetters=" + e.dead + " warnings=" + e.warnings;
      }
    });
    for (String timeout : new String[] { "0s", "-1s" }) {
      run("MBX-PT-0" + (timeout.equals("0s") ? "4" : "5") + " unstarted-push-timeout = " + timeout + ": a send that finds the lock held is dropped at once", () -> {
        try (Env e = new Env(5, "0ms", timeout)) {
          BlockingHooks h = new BlockingHooks(); ActorRef c = pushChild(e, h);
          long t0 = System.nanoTime(); c.tell("m2", ActorRef.noSender()); long ms = (System.nanoTime() - t0) / 1_000_000;
          awaitUntil(() -> !e.dead.isEmpty() && !e.warnings.isEmpty());
          h.release.countDown(); awaitUntil(() -> e.log.size() == 1); pause(300);
          return "returnedWithinBound=" + (ms < 5000) + " deadLetters=" + e.dead + " warnings=" + e.warnings + " log=" + e.log;
        }
      });
    }
    run("MBX-UC-01 messages sent to an unstarted actor: the system message goes first, then the user messages in order", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(Spawner.class, e, null, "queued", child)).tell("go", ActorRef.noSender());
        awaitChildStarted(child); ActorRef c = child.get();
        awaitUntil(() -> box(c).suspendCount() == 1 && !box(c).isScheduled());
        String afterStart = view(box(c)) + " log=" + e.log;
        in(c).resume(null);
        awaitUntil(() -> e.log.size() == 2); idle(c);
        return "afterStart: " + afterStart + " | afterResume: " + view(box(c)) + " log=" + e.log;
      }
    });
    run("MBX-UC-02 a Watch and a Terminate sent to an unstarted actor are applied once it starts: the watcher gets Terminated", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(Spawner.class, e, null, "watched", child)).tell("go", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 1);
        awaitUntil(() -> child.get() != null && in(child.get()).isTerminated());
        return "watcherLog=" + e.log + " childTerminated=" + in(child.get()).isTerminated();
      }
    });
    run("MBX-UC-03 a Watch undone by an Unwatch before the actor starts: the watcher is not told", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(Spawner.class, e, null, "unwatched", child)).tell("go", ActorRef.noSender());
        awaitUntil(() -> child.get() != null && in(child.get()).isTerminated());
        try { Thread.sleep(300); } catch (InterruptedException x) { throw new RuntimeException(x); } // a bounded wait for an absence
        return "watcherLog=" + e.log + " childTerminated=" + in(child.get()).isTerminated();
      }
    });
    run("MBX-UC-04 a child created under a suspended parent inherits the suspension", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef parent = e.spawn("p", null); idle(parent);
        in(parent).suspend(); awaitUntil(() -> box(parent).suspendCount() == 1);
        nextHooks = new Hooks();
        ActorRef c = ((ActorCell) ((ActorRefWithCell) parent).underlying()).attachChild(Props.create(Worker.class, e).withMailbox("hook-mailbox"), "c", false);
        awaitUntil(() -> box(c).suspendCount() == 1 && !box(c).isScheduled());
        c.tell("u", ActorRef.noSender()); idle(c);
        String suspended = view(box(c)) + " log=" + e.log;
        in(c).resume(null);
        awaitUntil(() -> e.log.size() == 1); idle(c);
        return "childUnderSuspendedParent: " + suspended + " | afterResume: " + view(box(c)) + " log=" + e.log;
      }
    });
    run("MBX-IF-01 the cell of an asynchronously created actor cannot be made: actorOf returns, the supervisor stops the actor, which never starts", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.sys.actorOf(Props.create(Worker.class, e).withMailbox("failing-mailbox"), "a");
        awaitUntil(() -> in(a).isTerminated());
        AtomicReference<String> again = new AtomicReference<>();
        awaitUntil(() -> { try { e.sys.actorOf(Props.create(Worker.class, e), "a"); again.set("created"); return true; } catch (InvalidActorNameException x) { return false; } });
        return "refReturned=" + (a != null) + " started=" + e.started.get() + " terminated=" + in(a).isTerminated() + " nameReusable=" + again.get();
      }
    });
    run("MBX-IF-02 messages sent to an actor whose cell cannot be made, before and after it is stopped, are dead letters", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.sys.actorOf(Props.create(Worker.class, e).withMailbox("failing-mailbox"), "a");
        a.tell("before", ActorRef.noSender());
        awaitUntil(() -> in(a).isTerminated());
        a.tell("after", ActorRef.noSender());
        awaitUntil(() -> e.dead.size() == 2);
        return "dead=" + new TreeSet<>(e.dead) + " log=" + e.log + " started=" + e.started.get();
      }
    });
    run("MBX-IF-03 a supervisor that restarts on ActorInitializationException: the failed creation becomes a Create and the actor starts", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(RestartingParent.class, e, child), "parent").tell("go", ActorRef.noSender());
        awaitUntil(() -> e.started.get() == 1);
        ActorRef c = child.get();
        c.tell("m", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 1);
        // the supervisor's own log of the failure (logFailure, AC-SUP-018) is another contract: only the cell's Error is observed here
        return "errors=" + cellErrors(e) + " started=" + e.started.get() + " log=" + e.log + " terminated=" + in(c).isTerminated();
      }
    });
    run("MBX-IF-04 an actor that fails to start after creating a child is restarted as a Create: its children are stopped first", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        AtomicInteger starts = new AtomicInteger();
        e.sys.actorOf(Props.create(RestartingSupervisor.class, Props.create(Rebuilder.class, e, starts), child), "sup").tell("go", ActorRef.noSender());
        awaitUntil(() -> e.log.contains("preStart:2"));
        awaitUntil(() -> e.started.get() == 2);
        child.get().tell("m", ActorRef.noSender());
        awaitUntil(() -> e.log.contains("r:m"));
        return "errors=" + cellErrors(e) + " starts=" + starts.get() + " childStarts=" + e.started.get() + " log=" + e.log + " terminated=" + in(child.get()).isTerminated();
      }
    });
    run("MBX-IF-05 a supervisor that resumes on ActorInitializationException: the failed creation becomes a Create and the actor starts", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(RestartingParent.class, e, child, true), "parent").tell("go", ActorRef.noSender());
        awaitUntil(() -> e.started.get() == 1);
        ActorRef c = child.get();
        c.tell("m", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 1);
        return "errors=" + cellErrors(e) + " started=" + e.started.get() + " log=" + e.log + " terminated=" + in(c).isTerminated();
      }
    });
    run("MBX-IF-06 the supervisor's log of a cell that cannot be made publishes the message of what made it fail", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(RestartingParent.class, e, child), "parent").tell("go", ActorRef.noSender());
        String logged = "|org.apache.pekko.actor.ActorInitializationException|";
        awaitUntil(() -> e.errors.stream().anyMatch(x -> x.contains(logged)));
        awaitUntil(() -> e.started.get() == 1);
        return "errors=" + e.errors.stream().filter(x -> x.contains(logged)).collect(java.util.stream.Collectors.toList());
      }
    });
    run("MBX-AC-01 actorOf returns before the child starts: it is found by name and its name is taken at once", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> child = new AtomicReference<>();
        e.sys.actorOf(Props.create(Spawner.class, e, null, "visibility", child)).tell("go", ActorRef.noSender());
        awaitUntil(() -> e.note.get() != null);
        awaitUntil(() -> e.started.get() == 1);
        return "insideTurn: " + e.note.get() + " | afterTurn: startedActors=" + e.started.get();
      }
    });
    run("MBX-CL-01 closed mailbox of a stopped actor", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null); Mailbox old = box(a); in(a).stop();
        awaitUntil(() -> in(a).isTerminated() && old.isClosed());
        return "old: " + st(old) + " resume=" + old.resume() + " suspend=" + old.suspend() + " schedule=" + old.setAsScheduled() + " " + st(old)
            + " | cellNow closed=" + box(a).isClosed() + " same=" + (box(a) == old);
      }
    });

    run("MBX-CC-01 eight producers on a one-thread pool: no loss, per-sender order, serial execution", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null); CyclicBarrier go = new CyclicBarrier(8); List<Thread> ts = new ArrayList<>();
        for (int p = 0; p < 8; p++) { int id = p; Thread t = new Thread(() -> { try { go.await(); } catch (Exception x) { throw new RuntimeException(x); }
            for (int i = 0; i < 50; i++) a.tell(id + ":" + i, ActorRef.noSender()); }); ts.add(t); t.start(); }
        awaitUntil(() -> e.log.size() == 400);
        Map<String, Integer> next = new HashMap<>(); boolean ordered = true;
        for (String s : e.log) { String[] kv = s.substring(2).split(":"); int n = next.merge(kv[0], 1, Integer::sum) - 1; ordered &= n == Integer.parseInt(kv[1]); }
        return "received=" + e.log.size() + " maxInFlight=" + e.max.get() + " perSenderOrdered=" + ordered;
      }
    });
    run("MBX-CC-02 message enqueued at the last empty dequeue of a turn is not stranded", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> ref = new AtomicReference<>(); AtomicBoolean fired = new AtomicBoolean(), delivered = new AtomicBoolean();
        ActorRef a = e.spawn("w", new Hooks() { @Override void onDequeue(Envelope r) { if (r != null) delivered.set(true); else if (delivered.get() && fired.compareAndSet(false, true)) ref.get().tell("late", ActorRef.noSender()); } });
        ref.set(a); a.tell("first", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 2); idle(a);
        return "log=" + e.log + " " + view(box(a));
      }
    });
    run("MBX-CC-03 concurrent Suspend/Resume system messages lose no increments", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null); Block b = new Block(); a.tell(b, ActorRef.noSender()); b.entered.await(); sendAll(a, "u");
        hammer(() -> in(a).suspend());
        b.release.countDown(); awaitUntil(() -> box(a).suspendCount() == 100); idle(a);
        String suspended = view(box(a)) + " log=" + e.log;
        hammer(() -> in(a).resume(null));
        awaitUntil(() -> e.log.size() == 2); idle(a);
        return "afterSuspends: " + suspended + " | afterResumes: " + view(box(a)) + " log=" + e.log;
      }
    });
    run("MBX-CC-04 dead-letter forwarding during cleanUp", () -> {
      try (Env e = new Env(5, "0ms")) {
        AtomicReference<ActorRef> ref = new AtomicReference<>();
        ActorRef a = e.spawn("w", new Hooks() { @Override void onCleanUp() { ref.get().tell("in-cleanup", ActorRef.noSender()); in(ref.get()).sendSystemMessage(new Suspend()); } });
        ref.set(a); Block b = new Block(); a.tell(b, ActorRef.noSender()); b.entered.await(); sendAll(a, "pending"); in(a).stop();
        b.release.countDown(); awaitUntil(() -> e.dead.size() == 3);
        return "deadLetters=" + e.dead + " received=" + e.log;
      }
    });
    run("MBX-CC-05 producer that read the mailbox before it closed strands its message", () -> {
      try (Env e = new Env(5, "0ms")) {
        CountDownLatch inEnqueue = new CountDownLatch(1), proceed = new CountDownLatch(1);
        Hooks h = new Hooks() { @Override void onEnqueue(Object m) throws Exception { if ("straggler".equals(m)) { inEnqueue.countDown(); proceed.await(); } } };
        ActorRef a = e.spawn("w", h);
        Mailbox old = box(a); Thread t = new Thread(() -> a.tell("straggler", ActorRef.noSender())); t.start(); inEnqueue.await();
        in(a).stop(); h.cleaned.await(); awaitUntil(() -> in(a).isTerminated());
        proceed.countDown(); t.join();
        a.tell("marker", ActorRef.noSender()); awaitUntil(() -> e.dead.contains("marker")); // drains the asynchronous listener before reading
        e.dead.remove("marker");
        return "received=" + e.log + " deadLetters=" + e.dead + " oldMailbox=" + view(old) + " (neither processed nor dead-lettered)";
      }
    });

    run("MBX-CC-05a producer that enqueues before cleanUp drains the old queue is dead-lettered", () -> {
      try (Env e = new Env(5, "0ms")) {
        CountDownLatch inEnqueue = new CountDownLatch(1), proceed = new CountDownLatch(1);
        Hooks h = new Hooks() {
          @Override void onEnqueue(Object m) throws Exception { if ("straggler".equals(m)) { inEnqueue.countDown(); proceed.await(); } }
          @Override void onCleanUp() { proceed.countDown(); awaitUntil(() -> queue.hasMessages()); } // the straggler's add lands just before the drain
        };
        ActorRef a = e.spawn("w", h);
        Mailbox old = box(a); Thread t = new Thread(() -> a.tell("straggler", ActorRef.noSender())); t.start(); inEnqueue.await();
        in(a).stop(); h.cleaned.await(); t.join(); awaitUntil(() -> in(a).isTerminated());
        a.tell("marker", ActorRef.noSender()); awaitUntil(() -> e.dead.contains("marker"));
        e.dead.remove("marker");
        return "received=" + e.log + " deadLetters=" + e.dead + " oldMailbox=" + view(old);
      }
    });
    run("MBX-SC-01 throughput 2 on one thread interleaves two actors in blocks of two", () -> {
      try (Env e = new Env(2, "0ms")) {
        ActorRef g = e.spawn("g", null), x = e.spawn("x", null), y = e.spawn("y", null); Block b = new Block(); g.tell(b, ActorRef.noSender()); b.entered.await();
        for (int i = 1; i <= 6; i++) x.tell("x" + i, ActorRef.noSender());
        for (int i = 1; i <= 6; i++) y.tell("y" + i, ActorRef.noSender());
        b.release.countDown(); awaitUntil(() -> e.log.size() == 13);
        return e.log.toString();
      }
    });
    run("MBX-SC-02 throughput-deadline-time ends the turn after a slow message", () -> {
      try (Env e = new Env(100, "30ms")) {
        ActorRef g = e.spawn("g", null), x = e.spawn("x", null), y = e.spawn("y", null); Block b = new Block(); g.tell(b, ActorRef.noSender()); b.entered.await();
        for (int i = 1; i <= 3; i++) x.tell("sleep-x" + i, ActorRef.noSender());
        for (int i = 1; i <= 3; i++) y.tell("sleep-y" + i, ActorRef.noSender());
        b.release.countDown(); awaitUntil(() -> e.log.size() == 7);
        return e.log.toString();
      }
    });
    run("MBX-SC-03 actor suspended with queued work does not reschedule itself", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = e.spawn("w", null); Block b = new Block(); a.tell(b, ActorRef.noSender()); b.entered.await();
        sendAll(a, "u2", "u3", "u4"); in(a).suspend(); b.release.countDown(); idle(a);
        String s = view(box(a)) + " log=" + e.log; in(a).stop(); awaitUntil(() -> in(a).isTerminated()); awaitUntil(() -> e.dead.size() == 3);
        return "suspendedWithWork: " + s + " | afterStop deadLetters=" + e.dead;
      }
    });

    run("MBX-UT-01 swapLookup: before the real cell is installed the reference is still unstarted, but a child the actor created in preStart already resolves through the real cell", () -> {
      try (Env e = new Env(5, "0ms")) {
        CountDownLatch release = new CountDownLatch(1);
        Plan plan = new Plan(); plan.messages = List.of("m1"); plan.childMakesChild = true; plan.childDispatcher = "rej-disp";
        parentInGo(e, plan);
        // point() = swapLookup(cell); cell.start(); u.replaceWith(cell). The executor accepts the task cell.start() submits, then holds the submitting
        // thread: the actor runs (preStart creates `k`) while replaceWith has not begun.
        SUBMITTED_AND_HELD = new CountDownLatch(1); HOLD_SUBMITTER_AFTER_NEXT_EXECUTE.set(release);
        plan.proceed.countDown();
        SUBMITTED_AND_HELD.await();
        awaitUntil(() -> e.log.contains("c:made-k"));
        ActorRef c = plan.child.get();
        String during = lookupView(c);
        release.countDown();
        awaitUntil(() -> e.log.contains("c:r:m1")); awaitUntil(() -> ((ActorRefWithCell) c).underlying() instanceof ActorCell);
        return "beforeReplaceWith: " + during + " | afterReplaceWith: " + lookupView(c) + " log=" + e.log;
      }
    });
    run("MBX-UT-02 an unstarted reference: not started, not terminated, counts its queued user messages, queues Suspend and Resume and hands everything over in order", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan(); plan.messages = List.of("m1", "m2"); plan.holdAfterCreate = true;
        parentInGo(e, plan); plan.proceed.countDown(); plan.created.await();
        ActorRef c = plan.child.get();
        String twoQueued = stateView(c) + " lookup: " + lookupView(c) + " parent=" + path(in(c).getParent()) + " isLocal=" + in(c).isLocal() + " path=" + path(c);
        in(c).suspend(); in(c).resume(null);
        String systemQueued = stateView(c);
        c.tell("m3", ActorRef.noSender());
        String threeQueued = stateView(c);
        plan.finish.countDown();
        awaitUntil(() -> e.log.contains("c:r:m3"));
        return "unstarted: " + twoQueued + " | suspend+resume queued: " + systemQueued + " | m3: " + threeQueued + " | handedOver: log=" + e.log + " started=" + ((RepointableRef) c).isStarted();
      }
    });
    run("MBX-UT-03 a parent that terminates before the Supervise of its child: Supervise is dead-lettered, the child never starts and keeps its queue", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan(); plan.messages = List.of("m1", "m2");
        ActorRef p = parentInGo(e, plan);
        in(p).stop(); // Terminate enters p's system queue before the Supervise that attachChild sends
        plan.proceed.countDown();
        awaitUntil(() -> in(p).isTerminated()); awaitUntil(() -> e.dead.contains("Supervise"));
        absence();
        ActorRef c = plan.child.get();
        String afterTerminate = "parentTerminated=" + in(p).isTerminated() + " deadLetters=" + e.dead + " | child: " + stateView(c);
        c.tell("m3", ActorRef.noSender());
        absence();
        return afterTerminate + " | after a third message: " + stateView(c) + " deadLetters=" + e.dead + " log=" + e.log;
      }
    });
    run("MBX-UT-04 control: a parent stopped after its child started waits for it, and the child dies first", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan(); plan.messages = List.of("m1");
        ActorRef p = parentInGo(e, plan); plan.proceed.countDown();
        awaitUntil(() -> e.log.contains("c:r:m1"));
        in(p).stop();
        awaitUntil(() -> in(p).isTerminated());
        return "parentTerminated=" + in(p).isTerminated() + " childTerminated=" + in(plan.child.get()).isTerminated() + " deadLetters=" + e.dead + " log=" + e.log;
      }
    });
    run("MBX-UT-05 a parent already waiting for a started child to die ignores the Supervise of a new child: the child is never pointed and the parent still terminates", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan(); plan.messages = List.of("m1"); plan.parentHoldsChild = true;
        ActorRef p = parentInGo(e, plan);
        in(p).stop();
        plan.proceed.countDown();
        awaitUntil(() -> e.log.contains("k1:postStop")); // the parent is now Terminating, waiting for k1 (whose postStop is held)
        absence();
        ActorRef c = plan.child.get();
        String whileWaiting = "parentTerminated=" + in(p).isTerminated() + " log=" + e.log + " deadLetters=" + e.dead + " | child: " + stateView(c);
        e.release.countDown();
        awaitUntil(() -> in(p).isTerminated());
        absence();
        return "whileWaiting: " + whileWaiting + " | afterRelease: parentTerminated=" + in(p).isTerminated() + " log=" + e.log + " deadLetters=" + e.dead + " | child: " + stateView(c);
      }
    });
    run("MBX-UT-06 a restart with only an unstarted child does not wait for it: the new instance starts first, then the child starts, is terminated", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan(); plan.messages = List.of("m1");
        ActorRef p = parentInGo(e, plan);
        in(p).suspend(); in(p).restart(new IllegalStateException("restart"));
        plan.proceed.countDown();
        awaitUntil(() -> e.log.contains("c:postStop"));
        awaitUntil(() -> in(plan.child.get()).isTerminated());
        return "log=" + e.log + " parentInstances=" + plan.instances.get() + " childTerminated=" + in(plan.child.get()).isTerminated() + " parentTerminated=" + in(p).isTerminated();
      }
    });
    run("MBX-UT-07 a restart waiting for a started child: the Supervise of an unstarted child is processed meanwhile, the child starts and terminates, the restart completes when the started child is gone", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan(); plan.messages = List.of("m1"); plan.parentHoldsChild = true;
        ActorRef p = parentInGo(e, plan);
        in(p).suspend(); in(p).restart(new IllegalStateException("restart"));
        plan.proceed.countDown();
        awaitUntil(() -> e.log.contains("c:postStop")); awaitUntil(() -> e.log.contains("k1:postStop"));
        awaitUntil(() -> in(plan.child.get()).isTerminated());
        absence();
        // k1 runs on its own thread: where its lines fall between the parent's and c's is not deterministic, so they are reported apart
        String whileWaiting = "parentAndChild=" + ignoring(e.log, "k1:") + " k1=" + only(e.log, "k1:") + " parentInstances=" + plan.instances.get();
        e.release.countDown();
        awaitUntil(() -> plan.instances.get() == 2); awaitUntil(() -> e.log.contains("k1:postStop-released"));
        List<String> all = new ArrayList<>(e.log);
        return "whileWaiting: " + whileWaiting + " | afterRelease: parentAndChild=" + ignoring(all, "k1:") + " k1=" + only(all, "k1:") + " parentInstances=" + plan.instances.get()
            + " restartAfterK1Gone=" + (all.lastIndexOf("P:preStart") > all.indexOf("k1:postStop-released")) + " parentTerminated=" + in(p).isTerminated();
      }
    });
    run("MBX-UT-08 senders racing with the installation of the real cell: nothing lost, nothing repeated, per-sender order kept", () -> {
      try (Env e = new Env(5, "0ms")) {
        Plan plan = new Plan();
        parentInGo(e, plan); plan.proceed.countDown(); plan.created.await();
        ActorRef c = plan.child.get(); CyclicBarrier go = new CyclicBarrier(3); List<Thread> ts = new ArrayList<>();
        for (String who : new String[] {"a", "b"}) {
          Thread t = new Thread(() -> { try { go.await(); } catch (Exception x) { throw new RuntimeException(x); } for (int i = 0; i < 200; i++) c.tell(who + ":" + i, ActorRef.noSender()); });
          ts.add(t); t.start();
        }
        go.await();
        for (Thread t : ts) t.join();
        awaitUntil(() -> e.log.stream().filter(x -> x.startsWith("c:r:")).count() == 400);
        absence();
        Map<String, Integer> next = new HashMap<>(); boolean ordered = true; Set<String> distinct = new HashSet<>(); long delivered = 0;
        for (String s : e.log) {
          if (!s.startsWith("c:r:")) continue;
          delivered++; distinct.add(s); String[] kv = s.substring(4).split(":"); ordered &= next.merge(kv[0], 1, Integer::sum) - 1 == Integer.parseInt(kv[1]);
        }
        return "delivered=" + delivered + " distinct=" + distinct.size() + " perSenderOrdered=" + ordered + " preStarts=" + e.log.stream().filter("c:preStart"::equals).count() + " deadLetters=" + e.dead;
      }
    });
    run("MBX-UT-09 a child created in preStart while the hand-over is in progress waits for the hand-over to end: the reference's lock is held across replaceWith", () -> {
      try (Env e = new Env(5, "0ms")) {
        CountDownLatch inHandOver = new CountDownLatch(1), letGo = new CountDownLatch(1);
        Plan plan = new Plan(); plan.messages = List.of("m1"); plan.childMakesChild = true;
        // the first user message queued while unstarted blocks replaceWith, which holds the unstarted cell's lock; the real cell is already running
        plan.hooks = new Hooks() { @Override void onEnqueue(Object m) throws Exception { if ("m1".equals(m)) { inHandOver.countDown(); letGo.await(); } } };
        parentInGo(e, plan); plan.proceed.countDown();
        inHandOver.await();
        awaitUntil(() -> e.log.contains("c:preStart"));
        absence();
        String during = "isStarted=" + ((RepointableRef) plan.child.get()).isStarted() + " log=" + e.log;
        letGo.countDown();
        awaitUntil(() -> e.log.contains("c:r:m1"));
        return "duringHandOver: " + during + " | afterHandOver: isStarted=" + ((RepointableRef) plan.child.get()).isStarted() + " log=" + e.log;
      }
    });
    run("MBX-ID-01 two ids select their own queue and two actors with the same id keep separate queues", () -> {
      try (IdEnv e = new IdEnv(3)) {
        ActorRef a = e.spawn("a", "cap2-mailbox"), b = e.spawn("b", "cap4-mailbox"), c = e.spawn("c", "cap2-mailbox");
        for (ActorRef r : List.of(a, b, c)) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : List.of(a, b, c)) for (int i = 1; i <= 6; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 3 + 8 && e.dead.size() == 10);
        return e.perActor("a", "b", "c");
      }
    });
    run("MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again", () -> {
      try (IdEnv e = new IdEnv(0)) {
        String outcome;
        try { e.sys.actorOf(e.props("w", "typo"), "w"); outcome = "created"; }
        catch (Throwable t) { outcome = t.getClass().getName() + ": " + t.getMessage() + " | cause " + t.getCause(); }
        e.spawn("w", null);
        return "actorOf: " + outcome + " | instancesStartedAfterRetryWithSameName=" + e.started.get();
      }
    });
    run("MBX-ID-03 no selection, the built-in ids and the empty id all give the unbounded FIFO queue", () -> {
      try (IdEnv e = new IdEnv(4)) {
        String[] ids = { null, "unbounded", "pekko.actor.default-mailbox", "" };
        List<ActorRef> refs = new ArrayList<>();
        for (int i = 0; i < ids.length; i++) refs.add(e.spawn("w" + i, ids[i]));
        for (ActorRef r : refs) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : refs) for (int i = 1; i <= 10; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 4 * 11);
        return e.perActor("w0", "w1", "w2", "w3");
      }
    });
    run("MBX-CFG-02 a mailbox configured as the default serves the actors without a selection, not the ids that name another", () -> {
      try (IdEnv e = new IdEnv(5, "pekko.actor.default-mailbox.mailbox-type = \"pilot.MailboxOracle$LifoType\"\n")) {
        String[] ids = { null, "", "pekko.actor.default-mailbox", "unbounded", "cap4-mailbox" };
        List<ActorRef> refs = new ArrayList<>();
        for (int i = 0; i < ids.length; i++) refs.add(e.spawn("w" + i, ids[i]));
        for (ActorRef r : refs) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : refs) for (int i = 1; i <= 6; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 3 * 7 + 7 + 5 && e.dead.size() == 2);
        return e.perActor("w0", "w1", "w2", "w3", "w4");
      }
    });
    run("MBX-CFG-04 a deployment of the actor path decides the mailbox over the props, by the full path of a child too, and its unknown id fails actorOf", () -> {
      try (IdEnv e = new IdEnv(5, "lifo-mailbox.mailbox-type = \"pilot.MailboxOracle$LifoType\"\n"
          + "pekko.actor.deployment { \"/d-lifo\" { mailbox = lifo-mailbox }\n \"/d-lifo-b\" { mailbox = lifo-mailbox }\n \"/d-empty\" { mailbox = \"\" }\n"
          + " \"/d-bad\" { mailbox = typo }\n \"/kid\" { mailbox = typo }\n \"/d-parent/kid\" { mailbox = lifo-mailbox } }\n")) {
        ActorRef lifo = e.spawn("d-lifo", "cap4-mailbox"), lifoB = e.spawn("d-lifo-b", "never-registered"), empty = e.spawn("d-empty", "cap4-mailbox"),
            other = e.spawn("other", "cap4-mailbox");
        ActorRef parent = e.sys.actorOf(Props.create(IdParent.class, e), "d-parent");
        parent.tell("spawn", ActorRef.noSender());
        awaitUntil(() -> e.started.get() == 5 && e.kid.get() != null);
        String bad = "";
        for (int attempt = 0; attempt < 2; attempt++) {
          try { e.sys.actorOf(e.props("d-bad", "cap4-mailbox"), "d-bad"); bad += " | created"; }
          catch (Throwable t) { bad += " | " + t.getClass().getName() + ": " + t.getMessage() + " cause " + t.getCause(); }
        }
        List<ActorRef> refs = new ArrayList<>(List.of(lifo, lifoB, empty, other, e.kid.get()));
        for (ActorRef r : refs) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : refs) for (int i = 1; i <= 6; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 3 * 7 + 2 * 5 && e.dead.size() == 4);
        return e.perActor("d-lifo", "d-lifo-b", "d-empty", "other", "kid") + bad;
      }
    });
    run("MBX-REQ-01 an actor that requires a queue semantics gets the mailbox bound to it, not the default, and the actors that do not require it keep the default", () -> {
      try (IdEnv e = new IdEnv(5, "pekko.actor.default-mailbox.mailbox-type = \"pilot.MailboxOracle$LifoType\"\n"
          + "audit-mailbox.mailbox-type = \"pilot.MailboxOracle$AuditTrailType\"\n"
          + "pekko.actor.mailbox.requirements { \"org.apache.pekko.dispatch.BoundedMessageQueueSemantics\" = cap4-mailbox\n"
          + " \"pilot.MailboxOracle$AuditTrail\" = audit-mailbox }\n")) {
        ActorRef plain = e.spawn("plain", null), unbounded = e.spawnProps("unbounded", e.requiring(UnboundedWorker.class, "unbounded")),
            bounded = e.spawnProps("bounded", e.requiring(BoundedWorker.class, "bounded")), audit = e.spawnProps("audit", e.requiring(AuditTrailWorker.class, "audit")),
            plainAfter = e.spawn("plainAfter", null);
        List<ActorRef> refs = List.of(plain, unbounded, bounded, audit, plainAfter);
        for (ActorRef r : refs) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : refs) for (int i = 1; i <= 6; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 4 * 7 + 5 && e.dead.size() == 2);
        return e.perActor("plain", "unbounded", "bounded", "audit", "plainAfter");
      }
    });
    run("MBX-REQ-02 an actor that requires a queue semantics with no mailbox bound to it fails actorOf at the call and its name is free again", () -> {
      try (IdEnv e = new IdEnv(0)) {
        String outcome;
        try { e.sys.actorOf(e.requiring(UnmappedWorker.class, "w"), "w"); outcome = "created"; }
        catch (Throwable t) { outcome = t.getClass().getName() + ": " + t.getMessage() + " | cause " + t.getCause(); }
        e.spawn("w", (String) null);
        return "actorOf: " + outcome + " | instancesStartedAfterRetryWithSameName=" + e.started.get();
      }
    });
    run("MBX-REQ-03 an id selected by the props or by a deployment decides the mailbox over the queue semantics the actor requires; an empty id leaves the requirement", () -> {
      try (IdEnv e = new IdEnv(6, "lifo-mailbox.mailbox-type = \"pilot.MailboxOracle$LifoType\"\n"
          + "cap2-mailbox { mailbox-type = \"org.apache.pekko.dispatch.NonBlockingBoundedMailbox\"\n mailbox-capacity = 2 }\n"
          + "pekko.actor.mailbox.requirements { \"org.apache.pekko.dispatch.BoundedMessageQueueSemantics\" = cap4-mailbox }\n"
          + "pekko.actor.deployment { \"/deployed\" { mailbox = lifo-mailbox }\n \"/emptyDeploy\" { mailbox = \"\" } }\n")) {
        ActorRef byProps = e.spawnProps("byProps", e.requiring(UnboundedWorker.class, "byProps").withMailbox("lifo-mailbox")),
            byPropsBounded = e.spawnProps("byPropsBounded", e.requiring(BoundedWorker.class, "byPropsBounded").withMailbox("cap2-mailbox")),
            deployed = e.spawnProps("deployed", e.requiring(UnboundedWorker.class, "deployed")),
            emptyDeploy = e.spawnProps("emptyDeploy", e.requiring(UnboundedWorker.class, "emptyDeploy")),
            emptyProps = e.spawnProps("emptyProps", e.requiring(UnboundedWorker.class, "emptyProps").withMailbox("")),
            required = e.spawnProps("required", e.requiring(BoundedWorker.class, "required"));
        List<ActorRef> refs = List.of(byProps, byPropsBounded, deployed, emptyDeploy, emptyProps, required);
        for (ActorRef r : refs) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : refs) for (int i = 1; i <= 6; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 4 * 7 + 3 + 5 && e.dead.size() == 4 + 2);
        return e.perActor("byProps", "byPropsBounded", "deployed", "emptyDeploy", "emptyProps", "required");
      }
    });
    run("MBX-REQ-04 the queue semantics the dispatcher requires selects the mailbox of the actors that require nothing, and of those whose own requirement has no binding; the actor's own binding and a selected id come first", () -> {
      try (IdEnv e = new IdEnv(4, "pekko.actor.default-dispatcher.mailbox-requirement = \"org.apache.pekko.dispatch.UnboundedMessageQueueSemantics\"\n"
          + "lifo-mailbox.mailbox-type = \"pilot.MailboxOracle$LifoType\"\n audit-mailbox.mailbox-type = \"pilot.MailboxOracle$AuditTrailType\"\n"
          + "pekko.actor.mailbox.requirements { \"org.apache.pekko.dispatch.UnboundedMessageQueueSemantics\" = lifo-mailbox\n"
          + " \"pilot.MailboxOracle$AuditTrail\" = audit-mailbox }\n")) {
        ActorRef plain = e.spawn("plain", null), anyQueue = e.spawnProps("anyQueue", e.requiring(AnyQueueWorker.class, "anyQueue")),
            audit = e.spawnProps("audit", e.requiring(AuditTrailWorker.class, "audit")), selected = e.spawn("selected", "unbounded");
        List<ActorRef> refs = List.of(plain, anyQueue, audit, selected);
        for (ActorRef r : refs) r.tell("block", ActorRef.noSender());
        e.entered.await();
        for (ActorRef r : refs) for (int i = 1; i <= 6; i++) r.tell("m" + i, ActorRef.noSender());
        e.release.countDown();
        awaitUntil(() -> e.log.size() == 4 * 7);
        return e.perActor("plain", "anyQueue", "audit", "selected");
      }
    });
    run("MBX-CMP-01 an actor whose requirement the queue promised by the selected mailbox does not fulfill fails actorOf at the call; one it fulfills, or an actor that requires nothing, is created", () -> {
      try (IdEnv e = new IdEnv(0, "plain-mailbox.mailbox-type = \"pilot.MailboxOracle$PlainType\"\n audit-mailbox.mailbox-type = \"pilot.MailboxOracle$AuditTrailType\"\n")) {
        return attempts(e, new Object[][] {
            {"unboundedOnPlain", UnboundedWorker.class, "plain-mailbox"}, {"auditOnUnbounded", AuditTrailWorker.class, "unbounded"},
            {"boundedOnUnbounded", BoundedWorker.class, "unbounded"}, {"unboundedOnAudit", UnboundedWorker.class, "audit-mailbox"},
            {"auditOnAudit", AuditTrailWorker.class, "audit-mailbox"}, {"anyOnPlain", AnyQueueWorker.class, "plain-mailbox"},
            {"nothingOnPlain", IdWorker.class, "plain-mailbox"}});
      }
    });
    run("MBX-CMP-02 the queue of the selected mailbox must fulfill the requirement of the dispatcher and that of the actor, the dispatcher's reported first; an actor binding that does not fit gives way to the dispatcher's binding", () -> {
      try (IdEnv e = new IdEnv(0, "pekko.actor.default-dispatcher.mailbox-requirement = \"org.apache.pekko.dispatch.UnboundedMessageQueueSemantics\"\n"
          + "audit-mailbox.mailbox-type = \"pilot.MailboxOracle$AuditTrailType\"\n"
          + "pekko.actor.mailbox.requirements { \"org.apache.pekko.dispatch.BoundedMessageQueueSemantics\" = cap4-mailbox\n \"pilot.MailboxOracle$AuditTrail\" = audit-mailbox }\n")) {
        return attempts(e, new Object[][] {
            {"dispatcherUnmetBySelected", IdWorker.class, "cap4-mailbox"}, {"dispatcherMetBySelected", IdWorker.class, "audit-mailbox"},
            {"boundedBindingFallsBack", BoundedWorker.class, null}, {"bothMetByAudit", AuditTrailWorker.class, null},
            {"actorUnmetOnly", BoundedWorker.class, "unbounded"}, {"bothUnmet", BoundedWorker.class, "cap4-mailbox"}, {"plainOnDefault", IdWorker.class, null}});
      }
    });
    run("MBX-CMP-03 a type that promises its queue is checked against the queue it creates when the actor is made: actorOf returns, the actor fails to initialise and stops, and its name is free", () -> {
      try (IdEnv e = new IdEnv(0, "liar-mailbox.mailbox-type = \"pilot.MailboxOracle$LiarType\"\n")) {
        ActorRef parent = e.sys.actorOf(Props.create(InitParent.class, e).withMailbox("unbounded"), "parent");
        parent.tell(e.requiring(AuditTrailWorker.class, "liar").withMailbox("liar-mailbox"), ActorRef.noSender());
        awaitUntil(() -> e.log.contains("terminated"));
        parent.tell(e.props("again", null), ActorRef.noSender());
        awaitUntil(() -> e.log.stream().anyMatch(l -> l.startsWith("again")) && e.started.get() == 1);
        return String.join(" | ", e.log) + " | started=" + e.started.get();
      }
    });
    run("MBX-GRD-01 the root, user and system guardians take the registered default mailbox directly: three queues of it exist before any actor is created", () -> {
      DEFAULT_QUEUES.set(0);
      try (IdEnv e = new IdEnv(0, "pekko.actor.default-mailbox.mailbox-type = \"pilot.MailboxOracle$CountingDefaultType\"\n")) {
        return "guardianQueues=" + DEFAULT_QUEUES.get();
      }
    });
    run("MBX-GRD-02 Kill to /user: the strategy of the root guardian logs and stops it, and the system terminates", () -> {
      try (GuardianEnv e = new GuardianEnv()) {
        e.treeUnderUser();
        e.sys.actorSelection("/user").tell(Kill.getInstance(), ActorRef.noSender());
        return e.afterTermination();
      }
    });
    run("MBX-GRD-03 Kill to the root guardian: the bubble walker logs, /user stops, and whenTerminated stays pending", () -> {
      try (GuardianEnv e = new GuardianEnv()) {
        e.treeUnderUser();
        e.sys.actorSelection("/").tell(Kill.getInstance(), ActorRef.noSender());
        awaitUntil(() -> e.stopped.size() == 2);
        try { Thread.sleep(500); } catch (InterruptedException x) { throw new RuntimeException(x); } // a bounded wait for an absence
        return "errors=" + e.errors + " stopped=" + e.stopped + " whenTerminated=" + e.sys.getWhenTerminated().toCompletableFuture().isDone();
      }
    });
    run("MBX-GRD-04 Kill to /system: the strategy of the root guardian logs and stops it, and the system terminates", () -> {
      try (GuardianEnv e = new GuardianEnv()) {
        e.treeUnderUser();
        e.sys.actorSelection("/system").tell(Kill.getInstance(), ActorRef.noSender());
        return e.afterTermination();
      }
    });
    run("MBX-RJ-08 an attach rejected twice while /user supervises: the root guardian stops /user and the system terminates", () -> {
      try (GuardianEnv e = new GuardianEnv()) {
        e.treeUnderUser();
        REJECT_NEXT.set(2);
        ActorRef rejected = e.sys.actorOf(Props.create(Tree.class, e.stopped, (Object) null).withDispatcher("rej-disp"), "r");
        return e.afterTermination() + " refReturned=" + (rejected != null) + " rejectionsLeft=" + REJECT_NEXT.get();
      }
    });
    run("MBX-DQ-01 the unbounded deque based queue takes an envelope enqueued first at its head, counts it and cleans up from the head", () -> {
      MessageQueue q = new UnboundedDequeBasedMailbox().create(scala.Option.empty(), scala.Option.empty()), dead = defaultQueue();
      DequeBasedMessageQueueSemantics deque = (DequeBasedMessageQueueSemantics) q;
      String type = "class=" + q.getClass().getName() + " unbounded=" + (q instanceof UnboundedMessageQueueSemantics)
          + " multipleConsumer=" + (q instanceof MultipleConsumerSemantics) + " bounded=" + (q instanceof BoundedMessageQueueSemantics);
      put(q, "a"); put(q, "b"); putFirst(deque, "c"); putFirst(deque, "d");
      String mixed = counts(q) + " " + drain(q) + " then " + (q.dequeue() == null ? "null" : "message") + " " + counts(q);
      putFirst(deque, "x"); putFirst(deque, "y");
      String onlyFirst = drain(q).toString();
      put(q, "a"); putFirst(deque, "c"); put(q, "b");
      q.cleanUp(Nobody$.MODULE$, dead);
      return type + " | mixed " + mixed + " | onlyFirst " + onlyFirst + " | cleanUp source " + counts(q) + " target " + drain(dead);
    });
    run("MBX-SCQ-01 the single consumer only unbounded queue is first in first out, counts and cleans up in order, and declares the semantics of Pekko's NodeMessageQueue", () -> {
      MessageQueue q = new SingleConsumerOnlyUnboundedMailbox().create(scala.Option.empty(), scala.Option.empty()), dead = defaultQueue();
      String type = "class=" + q.getClass().getName() + " unbounded=" + (q instanceof UnboundedMessageQueueSemantics)
          + " multipleConsumer=" + (q instanceof MultipleConsumerSemantics) + " bounded=" + (q instanceof BoundedMessageQueueSemantics);
      String empty = counts(q) + " " + (q.dequeue() == null ? "null" : "message");
      put(q, "a"); put(q, "b");
      String first = (String) q.dequeue().message();
      put(q, "c");
      String mixed = counts(q) + " first=" + first + " rest=" + drain(q) + " " + counts(q);
      put(q, "x"); put(q, "y");
      q.cleanUp(Nobody$.MODULE$, dead);
      return type + " | new " + empty + " | " + mixed + " | cleanUp source " + counts(q) + " target " + drain(dead);
    });
    run("MBX-TT-01 context.actorOf with a mailbox type that throws fails at the call with a ConfigurationException: no child is registered and the name is free", () -> {
      try (Env e = new Env(5, "0ms")) {
        e.sys.actorOf(Props.create(CatchingParent.class, e), "parent").tell("go", ActorRef.noSender());
        awaitUntil(() -> e.log.contains("r:m"));
        return e.note.get() + " log=" + e.log + " started=" + e.started.get() + " errors=" + e.errors;
      }
    });
    run("MBX-TT-02 system.actorOf with a mailbox type that throws returns: the user guardian stops the actor, keeps serving and the system terminates", () -> {
      Env e = new Env(5, "0ms");
      ActorRef a = e.sys.actorOf(Props.create(Worker.class, e).withMailbox("failing-mailbox"), "a");
      awaitUntil(() -> in(a).isTerminated() && !e.errors.isEmpty());
      e.sys.actorOf(Props.create(Worker.class, e), "b").tell("m", ActorRef.noSender());
      awaitUntil(() -> e.log.contains("r:m"));
      scala.concurrent.Await.result(e.sys.terminate(), scala.concurrent.duration.Duration.create(10, TimeUnit.SECONDS));
      return "terminated=true log=" + e.log + " started=" + e.started.get() + " errors=" + e.errors + " systemTerminated=true";
    });
    run("MBX-TQ-01 an enqueue that throws does not reach the caller of tell: an Error is published and the actor goes on", () -> {
      try (Env e = new Env(5, "0ms")) {
        ActorRef a = faulty(e, Fault.ENQUEUE);
        a.tell("a", ActorRef.noSender());
        String told;
        try { a.tell("bad", ActorRef.noSender()); told = "returned"; } catch (Throwable t) { told = "threw " + t; }
        a.tell("c", ActorRef.noSender());
        awaitUntil(() -> e.log.size() == 2 && !e.errors.isEmpty());
        return "tell=" + told + " log=" + e.log + " dead=" + e.dead + " errors=" + e.errors + " terminated=" + in(a).isTerminated();
      }
    });
    run("MBX-TQ-02 a dequeue that throws ends the turn with an uncaught-error report: the thread survives and the queued messages are processed", () -> {
      try (Env e = errorLoggingEnv()) {
        ActorRef a = faulty(e, Fault.DEQUEUE);
        Block b = new Block(); a.tell(b, ActorRef.noSender()); b.entered.await(); sendAll(a, "a", "bad", "c"); b.release.countDown();
        awaitUntil(() -> e.log.contains("r:c") && !e.errors.isEmpty());
        a.tell("d", ActorRef.noSender());
        awaitUntil(() -> e.log.contains("r:d"));
        return "log=" + e.log + " dead=" + e.dead + " errors=" + withoutThreadNumber(e.errors) + " terminated=" + in(a).isTerminated();
      }
    });
    run("MBX-TQ-03 a hasMessages that throws at the end of a turn is reported as uncaught: the next send schedules the actor again", () -> {
      try (Env e = errorLoggingEnv()) {
        ActorRef a = faulty(e, Fault.HAS_MESSAGES);
        a.tell("bad", ActorRef.noSender());
        awaitUntil(() -> !e.errors.isEmpty());
        a.tell("c", ActorRef.noSender());
        awaitUntil(() -> e.log.contains("r:c"));
        return "log=" + e.log + " errors=" + withoutThreadNumber(e.errors) + " terminated=" + in(a).isTerminated();
      }
    });
    run("MBX-TQ-06 a subscriber whose enqueue throws on the uncaught-error report: the others get both that report and the swallowed send, in no guaranteed order", () -> {
      try (Env e = errorLoggingEnv()) {
        ActorRef a = faulty(e, Fault.DEQUEUE);
        nextFault = Fault.ENQUEUE_OF_UNCAUGHT_REPORT;
        ActorRef broken = e.sys.actorOf(Props.create(Worker.class, e).withMailbox("faulty-mailbox"), "broken");
        awaitUntil(() -> e.started.get() == 2);
        e.sys.eventStream().subscribe(broken, Logging.Error.class); // after the listener of the environment
        a.tell("bad", ActorRef.noSender());
        awaitUntil(() -> e.errors.size() == 2);
        // an event stream gives no order guarantee between the two reports (AC-EVT-001): print them sorted
        return "errors=" + withoutThreadNumber(e.errors).stream().sorted().collect(java.util.stream.Collectors.toList());
      }
    });
  }

  /** Records its name when it stops; with a `child` name it first creates that child. */
  public static final class Tree extends AbstractActor {
    final Collection<String> stopped; final String child;
    public Tree(Collection<String> stopped, String child) { this.stopped = stopped; this.child = child; }
    @Override public void preStart() { if (child != null) getContext().actorOf(Props.create(Tree.class, stopped, (Object) null), child); }
    @Override public void postStop() { stopped.add(getSelf().path().name()); }
    @Override public Receive createReceive() { return receiveBuilder().matchAny(m -> getSender().tell(m, getSelf())).build(); }
  }

  /** A system whose {@code log.error} publishes ({@link Scenario#PUBLISH_LOG_ERRORS}) and a recorder of every {@code Logging.Error} that is not an
   *  actor: a FunctionRef runs on the publishing thread, so no suspended or stopped guardian can hide an error. */
  static final class GuardianEnv implements AutoCloseable {
    final ConcurrentLinkedQueue<String> errors = new ConcurrentLinkedQueue<>(), stopped = new ConcurrentLinkedQueue<>();
    final AutoCloseable mute = Scenario.muteStandardOutput();
    final ActorSystem sys = ActorSystem.create("mbx", ConfigFactory.parseString(Scenario.PUBLISH_LOG_ERRORS + "pekko.stdout-loglevel = OFF\n"
        + "rej-disp { type = Dispatcher\n executor = \"pilot.MailboxOracle$RejectingConfigurator\" }\n").withFallback(ConfigFactory.load()));
    GuardianEnv() throws Exception {
      ActorRef host = ((ExtendedActorSystem) sys).systemActorOf(Props.create(Tree.class, new ConcurrentLinkedQueue<String>(), (Object) null), "recorder");
      Patterns.ask(host, "started?", java.time.Duration.ofSeconds(10)).toCompletableFuture().get(); // until then its cell is unstarted
      FunctionRef recorder = ((ActorCell) ((ActorRefWithCell) host).underlying()).addFunctionRef((sender, message) -> {
        Logging.Error e = (Logging.Error) message;
        String text = String.valueOf(e.message()); // the text of the injected rejection is the test executor's own, not Pekko's
        errors.add(e.logSource() + "|" + e.logClass().getName() + "|" + e.cause().getClass().getName() + "|" + (text.equals("injected") ? "<rejection>" : text));
        return scala.runtime.BoxedUnit.UNIT;
      }, "");
      sys.eventStream().subscribe(recorder, Logging.Error.class);
    }
    /** {@code /user/a} with its child {@code c}, both started. */
    void treeUnderUser() throws Exception {
      ActorRef a = sys.actorOf(Props.create(Tree.class, stopped, "c"), "a");
      Patterns.ask(a, "started?", java.time.Duration.ofSeconds(10)).toCompletableFuture().get();
      Patterns.ask(sys.actorSelection("/user/a/c"), "started?", java.time.Duration.ofSeconds(10)).toCompletableFuture().get();
    }
    String afterTermination() throws Exception {
      Terminated t = sys.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS);
      return "errors=" + errors + " stopped=" + stopped + " terminated=" + t.getActor().path() + " existenceConfirmed=" + t.getExistenceConfirmed();
    }
    @Override public void close() throws Exception { sys.terminate(); mute.close(); }
  }

  /** Each attempt {label, worker, mailbox id or null}: {@code actorOf} with that mailbox selected; what it throws, or "created". */
  static String attempts(IdEnv e, Object[][] attempts) throws Exception {
    StringBuilder out = new StringBuilder();
    for (Object[] a : attempts) {
      String label = (String) a[0], outcome;
      @SuppressWarnings("unchecked") Class<? extends IdWorker> worker = (Class<? extends IdWorker>) a[1];
      Props props = e.requiring(worker, label);
      try { e.spawnProps(label, a[2] == null ? props : props.withMailbox((String) a[2])); outcome = "created"; }
      catch (Throwable t) { outcome = t.getClass().getName() + ": " + t.getMessage() + " | cause " + t.getCause(); }
      out.append(label).append(": ").append(outcome).append("; ");
    }
    return out.toString().trim();
  }

  /** Not a golden case (its counts vary run to run): `n` actors, each hit by 4 producers (300 sends each) released together with
   *  `stop`. Every send is either processed, dead-lettered, or left in the old closed queue (CC-05's outcome); the three must add up. */
  static String stress(int n) throws Exception {
    long received = 0, dead = 0, stranded = 0, sent = 0, trialsWithStranded = 0, unaccounted = 0;
    try (Env e = new Env(5, "0ms")) {
      for (int trial = 0; trial < n; trial++) {
        e.log.clear(); e.dead.clear();
        Hooks h = new Hooks(); ActorRef a = e.spawn("s" + trial, h); Mailbox old = box(a);
        CyclicBarrier go = new CyclicBarrier(5); List<Thread> ts = new ArrayList<>();
        for (int p = 0; p < 4; p++) { Thread th = new Thread(() -> { try { go.await(); } catch (Exception x) { throw new RuntimeException(x); }
            for (int i = 0; i < 300; i++) a.tell("m", ActorRef.noSender()); }); ts.add(th); th.start(); }
        go.await(); in(a).stop();
        for (Thread th : ts) th.join();
        h.cleaned.await(); awaitUntil(() -> in(a).isTerminated());
        a.tell("marker", ActorRef.noSender()); awaitUntil(() -> e.dead.contains("marker")); e.dead.remove("marker");
        long r = e.log.size(), d = e.dead.size(), st = old.numberOfMessages();
        received += r; dead += d; stranded += st; sent += 1200; if (st > 0) trialsWithStranded++; if (r + d + st != 1200) unaccounted++;
      }
    }
    return "trials=" + n + " sent=" + sent + " received=" + received + " deadLettered=" + dead + " strandedInOldQueue=" + stranded
        + " trialsWithStranded=" + trialsWithStranded + " trialsNotAccountedFor=" + unaccounted;
  }

  /** Two threads released together by a barrier, 50 operations each. */
  static void hammer(Runnable op) throws InterruptedException {
    CyclicBarrier go = new CyclicBarrier(2); List<Thread> ts = new ArrayList<>();
    for (int t = 0; t < 2; t++) {
      Thread th = new Thread(() -> { try { go.await(); } catch (Exception x) { throw new RuntimeException(x); } for (int i = 0; i < 50; i++) op.run(); });
      ts.add(th); th.start();
    }
    for (Thread th : ts) th.join();
  }
}
