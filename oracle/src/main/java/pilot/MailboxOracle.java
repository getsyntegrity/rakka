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

  static final class Env implements AutoCloseable {
    final ConcurrentLinkedQueue<String> log = new ConcurrentLinkedQueue<>(), dead = new ConcurrentLinkedQueue<>(), errors = new ConcurrentLinkedQueue<>();
    final AtomicInteger inFlight = new AtomicInteger(), max = new AtomicInteger();
    final AtomicInteger started = new AtomicInteger();
    final AtomicReference<String> note = new AtomicReference<>();
    final ActorSystem sys;
    Env(int throughput, String deadline) {
      sys = ActorSystem.create("mbx", ConfigFactory.parseString("pekko { loglevel = OFF\n stdout-loglevel = OFF\n log-dead-letters = off\n }\n"
          + "t-disp { type = Dispatcher\n executor = thread-pool-executor\n thread-pool-executor.fixed-pool-size = 1\n throughput = " + throughput
          + "\n throughput-deadline-time = " + deadline + " }\n hook-mailbox.mailbox-type = \"pilot.MailboxOracle$HookType\"\n"
          + "failing-mailbox.mailbox-type = \"pilot.MailboxOracle$FailingType\"\n"
          + "rej-disp { type = Dispatcher\n executor = \"pilot.MailboxOracle$RejectingConfigurator\"\n throughput = " + throughput + " }\n")
          .withFallback(ConfigFactory.load()));
      ActorRef listener = sys.actorOf(Props.create(Listener.class, dead));
      sys.eventStream().subscribe(listener, DeadLetter.class);
      sys.eventStream().subscribe(sys.actorOf(Props.create(ErrorListener.class, errors)), Logging.Error.class);
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
    @Override public void close() { sys.terminate(); }
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
        try { getContext().actorOf(Props.create(Worker.class, env).withDispatcher("rej-disp"), "c"); env.note.set("thrown=none"); }
        catch (Throwable t) { env.note.set("thrown=" + t.getClass().getName()); if (!caught) throw t; }
      }).matchEquals("again", a -> {
        try { getContext().actorOf(Props.create(Worker.class, env), "c"); env.note.set("again=ok"); }
        catch (Throwable t) { env.note.set("again=" + t.getClass().getName()); }
      }).matchEquals("who", w -> env.note.set("children=" + getContext().findChild("c").isPresent())).build();
    }
  }

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
