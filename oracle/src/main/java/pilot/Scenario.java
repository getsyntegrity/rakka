package pilot;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

import com.typesafe.config.ConfigFactory;
import org.apache.pekko.actor.AbstractActor;
import org.apache.pekko.actor.ActorIdentity;
import org.apache.pekko.actor.Identify;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.DeadLetter;
import org.apache.pekko.actor.ExtendedActorSystem;
import org.apache.pekko.actor.InternalActorRef;
import org.apache.pekko.actor.Kill;
import org.apache.pekko.actor.AllForOneStrategy;
import org.apache.pekko.actor.OneForOneStrategy;
import org.apache.pekko.actor.SupervisorStrategy;
import org.apache.pekko.japi.pf.DeciderBuilder;
import org.apache.pekko.actor.PoisonPill;
import org.apache.pekko.actor.Props;
import org.apache.pekko.actor.Terminated;
import org.apache.pekko.actor.UnhandledMessage;

/**
 * Executes a scenario against real Pekko Classic and reports what it observed.
 *
 * <pre>
 * {"system": "name", "dispatcher": {"parallelism": 1, "throughput": 5}, "steps": [ {"op": "spawn", ...}, ... ]}
 * </pre>
 *
 * Every step awaits whatever it observes (events, termination), or reports only invariants for races,
 * so the trace is deterministic. Output is one JSON line. See tools/make_scenarios*.py for the ops.
 */
final class Scenario {
  private Scenario() {}

  /** How long an await-* step waits for its condition (override with SCENARIO_AWAIT_MS, e.g. when debugging). */
  private static final long AWAIT_MS = System.getenv("SCENARIO_AWAIT_MS") != null ? Long.parseLong(System.getenv("SCENARIO_AWAIT_MS")) : 10_000;

  /** Everything the actors record, shared with the harness thread. */
  static final class Ctx {
    final Map<String, CopyOnWriteArrayList<String>> events = new ConcurrentHashMap<>();
    final CopyOnWriteArrayList<String> order = new CopyOnWriteArrayList<>(); // "label:msg", global receive order
    final Map<String, ActorRef> refs = new ConcurrentHashMap<>();
    final Map<String, List<String>> sent = new HashMap<>(); // target label -> messages sent by the harness thread
    final CountDownLatch gate = new CountDownLatch(1);
    final Map<String, java.util.concurrent.atomic.AtomicInteger> strategyCalls = new ConcurrentHashMap<>(); // `supervisorStrategy` calls per label
    final Map<String, CountDownLatch> holds = new ConcurrentHashMap<>(); // named gates a supervisor's decider waits on (`supervisor.hold`)
    CountDownLatch hold(String name) { return holds.computeIfAbsent(name, k -> new CountDownLatch(1)); }
    final CopyOnWriteArrayList<String> life = new CopyOnWriteArrayList<>(); // "label:pre-start", "label:post-stop", "label:terminated:<is>"
    final Map<Long, String> results = new ConcurrentHashMap<>(); // command id -> result fields
    final java.util.Set<String> failedPostRestart = ConcurrentHashMap.newKeySet(); // labels that already threw (`throw-in-post-restart: "once"`)
    final java.util.Set<String> failedPreStart = ConcurrentHashMap.newKeySet(); // labels that already threw (`throw-in-prestart: "once"`)
    final java.util.concurrent.atomic.AtomicLong cmdIds = new java.util.concurrent.atomic.AtomicLong();
    ActorSystem system;

    /** The label (alphabetically first) of a registered ref equal to `r` (same path and uid), or null. */
    String labelOf(ActorRef r) {
      return new java.util.TreeMap<>(refs).entrySet().stream().filter(e -> e.getValue().equals(r)).map(Map.Entry::getKey).findFirst().orElse(null);
    }

    void add(String label, String json) { events.computeIfAbsent(label, k -> new CopyOnWriteArrayList<>()).add(json); }
    List<String> of(String label) { return events.computeIfAbsent(label, k -> new CopyOnWriteArrayList<>()); }
  }

  /** A user message type with value equality (`{"custom": name}`), for `watch-with`. */
  public record WatchMsg(String name) {}

  static String msgJson(Object m) {
    if (m instanceof WatchMsg) return "{\"custom\":" + Cli.Json.str(((WatchMsg) m).name()) + "}";
    if (m instanceof ActorIdentity) {
      ActorIdentity i = (ActorIdentity) m;
      return "{\"identity\":" + msgJson(i.correlationId()) + ",\"ref\":"
          + (i.ref().isDefined() ? Cli.Json.str(i.ref().get().path().toString()) : "null") + "}";
    }
    return m instanceof String ? Cli.Json.str((String) m) : "{\"class\":" + Cli.Json.str(m.getClass().getName()) + "}";
  }

  /** Config that lets {@code log.error} publish: level ERROR and no logger of its own besides the stdout one. */
  static final String PUBLISH_LOG_ERRORS = "pekko.loglevel = ERROR\npekko.loggers = [\"org.apache.pekko.event.Logging$StandardOutLogger\"]\n";

  /** Discards what Pekko prints to standard output by itself (the log events still queued when /system stops, with their time stamps)
   *  until closed. Must be opened before the first Scala print, which keeps the stream it finds. */
  static AutoCloseable muteStandardOutput() {
    java.io.PrintStream real = System.out;
    System.setOut(new java.io.PrintStream(java.io.OutputStream.nullOutputStream()));
    return () -> System.setOut(real);
  }

  /** An actor that does nothing (slice 2a). */
  public static final class Noop extends AbstractActor {
    @Override public Receive createReceive() { return receiveBuilder().build(); }
  }

  /** A command the harness asks an actor to run inside its own context (`do` op). */
  static final class Cmd {
    final long id;
    final Map<String, Object> spec;
    Cmd(long id, Map<String, Object> spec) { this.id = id; this.spec = spec; }
  }

  /** A scripted actor: records lifecycle and received messages, then acts according to its behaviour config. */
  public static final class Scripted extends AbstractActor {
    private final String label;
    private final Map<String, Object> cfg;
    private final Ctx ctx;

    Scripted(String label, Map<String, Object> cfg, Ctx ctx) {
      this.label = label; this.cfg = cfg; this.ctx = ctx;
      // `constructor-fails-from: n`: the actor factory throws IllegalStateException("ctor") from the n-th instance of this behaviour on
      // (1 = always; 2 = the first instance works and the one made by a restart fails)
      Object from = cfg.get("constructor-fails-from");
      if (from != null) {
        int instance = ((java.util.concurrent.atomic.AtomicInteger) cfg.computeIfAbsent("_instances", k -> new java.util.concurrent.atomic.AtomicInteger())).incrementAndGet();
        if (instance >= ((Number) from).intValue()) throw new IllegalStateException("ctor");
      }
    }

    private long received = 0; // per instance: a restarted actor starts again at 0

    @Override public void preStart() throws Exception {
      ctx.add(label, "{\"ev\":\"pre-start\"}");
      ctx.life.add(label + ":pre-start");
      // `throw-in-prestart`: `true`, or the shape of the cause chain of what is thrown
      Object shape = cfg.get("throw-in-prestart");
      // `spawn-in-prestart: name`: a child made by `preStart`, before it can fail; `"once"` fails only the first `preStart` of the label
      if (cfg.get("spawn-in-prestart") != null) getContext().actorOf(Props.create(Noop.class), (String) cfg.get("spawn-in-prestart"));
      if ("once".equals(shape) && ctx.failedPreStart.add(label)) throw new IllegalStateException("init");
      if (Boolean.TRUE.equals(shape)) throw new IllegalStateException("init");
      if ("nested".equals(shape)) throw new IllegalStateException("init", new IllegalArgumentException("root"));
      if ("invocation".equals(shape)) throw new java.lang.reflect.InvocationTargetException(new IllegalStateException("inner"));
      if ("no-message".equals(shape)) throw new IllegalStateException();
    }

    /** `supervisor: {"log-calls": bool (records each call of this method), "throw-on-call": n (the n-th call of the label throws), "logging": bool (`loggingEnabled`), "decider-throws": bool, "strategy": "one-for-one"|"all-for-one", "hold": gate, "max": n, "log-decisions": bool, "decider": {"<exception message>": "resume"|"restart"|"stop"|"escalate"}}`: a OneForOneStrategy for this actor's children.
     *  Exceptions not in the map get Pekko's default decider. */
    @SuppressWarnings("unchecked")
    @Override public SupervisorStrategy supervisorStrategy() {
      Map<String, Object> sup = (Map<String, Object>) cfg.get("supervisor");
      if (sup == null) return super.supervisorStrategy();
      int call = ctx.strategyCalls.computeIfAbsent(label, k -> new java.util.concurrent.atomic.AtomicInteger()).incrementAndGet();
      if (Boolean.TRUE.equals(sup.get("log-calls"))) ctx.add(label, "{\"ev\":\"strategy\"}");
      if (sup.get("throw-on-call") != null && ((Long) sup.get("throw-on-call")).intValue() == call) throw new IllegalStateException("strategy");
      int max = sup.get("max") == null ? -1 : ((Long) sup.get("max")).intValue();
      Map<String, Object> dec = (Map<String, Object>) sup.get("decider");
      scala.PartialFunction<Throwable, SupervisorStrategy.Directive> decider =
          DeciderBuilder.match(Throwable.class, e -> {
            // `decider-throws`: the decider itself fails while deciding about a child's failure
            if (Boolean.TRUE.equals(sup.get("decider-throws"))) throw new IllegalStateException("decider");
            Object d = dec == null ? null : dec.get(e.getMessage());
            SupervisorStrategy.Directive chosen;
            if (d == null) chosen = SupervisorStrategy.defaultDecider().apply(e);
            else switch ((String) d) {
              case "resume": chosen = SupervisorStrategy.resume(); break;
              case "restart": chosen = SupervisorStrategy.restart(); break;
              case "stop": chosen = SupervisorStrategy.stop(); break;
              case "escalate": chosen = SupervisorStrategy.escalate(); break;
              default: throw new IllegalArgumentException("unknown directive " + d);
            }
            // `log-decisions`: record what the decider saw (the cause) and what it chose; off by default so older recordings stay valid
            if (Boolean.TRUE.equals(sup.get("log-decisions")))
              ctx.add(label, "{\"ev\":\"decide\",\"cause\":" + Cli.Json.str(reasonStr(e)) + ",\"directive\":" + Cli.Json.str(chosen.toString()) + "}");
            if (sup.get("hold") != null) { // the decision stays pending until the harness opens the hold
              ctx.life.add(label + ":holding");
              try { ctx.hold((String) sup.get("hold")).await(30, java.util.concurrent.TimeUnit.SECONDS); } catch (InterruptedException ie) { Thread.currentThread().interrupt(); }
            }
            return chosen;
          }).build();
      boolean logging = !Boolean.FALSE.equals(sup.get("logging"));
      return "all-for-one".equals(sup.get("strategy"))
          ? new AllForOneStrategy(max, scala.concurrent.duration.Duration.Inf(), logging, decider)
          : new OneForOneStrategy(max, scala.concurrent.duration.Duration.Inf(), logging, decider);
    }

    @SuppressWarnings("unchecked")
    private boolean throwsOn(Object m) {
      Object t = cfg.get("throw-on");
      return t instanceof List ? ((List<Object>) t).contains(m) : m.equals(t);
    }

    /** `log-sender`: the lifecycle hooks also record `sender()`, which reads `currentMessage`. */
    private String senderIfLogged() {
      return Boolean.TRUE.equals(cfg.get("log-sender")) ? ",\"sender\":" + Cli.Json.str(getSender().path().toString()) : "";
    }

    private static String reasonStr(Throwable t) { return t.getClass().getName() + (t.getMessage() == null ? "" : ": " + t.getMessage()); }

    @Override public void preRestart(Throwable reason, java.util.Optional<Object> message) throws Exception {
      // `default-pre-restart`: Pekko's own preRestart with nothing recorded, as for an actor that does not override it
      if (Boolean.TRUE.equals(cfg.get("default-pre-restart"))) { super.preRestart(reason, message); return; }
      ctx.add(label, "{\"ev\":\"pre-restart\",\"reason\":" + Cli.Json.str(reasonStr(reason)) + ",\"msg\":" + (message.isPresent() ? msgJson(message.get()) : "null") + senderIfLogged() + "}");
      ctx.life.add(label + ":pre-restart");
      if (Boolean.TRUE.equals(cfg.get("keep-children"))) return; // `keep-children`: like SupervisorTreeSpec, neither stops the children nor runs postStop
      // `pre-restart-throws: <message or null>`: preRestart fails before it calls super, so nothing of the default runs
      if (cfg.containsKey("pre-restart-throws")) throw new IllegalStateException((String) cfg.get("pre-restart-throws"));
      // `stop-without-unwatch`: an override that stops its children and runs postStop itself instead of calling super
      if (Boolean.TRUE.equals(cfg.get("stop-without-unwatch"))) {
        for (ActorRef child : getContext().getChildren()) getContext().stop(child);
        postStop();
        return;
      }
      super.preRestart(reason, message); // default: unwatch and stop all children, then postStop
    }

    @Override public void postRestart(Throwable reason) throws Exception {
      ctx.add(label, "{\"ev\":\"post-restart\",\"reason\":" + Cli.Json.str(reasonStr(reason)) + senderIfLogged() + "}");
      ctx.life.add(label + ":post-restart");
      Object fails = cfg.get("throw-in-post-restart"); // `true`, or `"once"`: only the first postRestart of this label
      if (Boolean.TRUE.equals(fails) || ("once".equals(fails) && ctx.failedPostRestart.add(label))) throw new IllegalStateException("post-restart"); // `throw-in-post-restart`
      super.postRestart(reason); // default: preStart
    }
    @Override public void postStop() {
      ctx.add(label, "{\"ev\":\"post-stop\"" + senderIfLogged() + "}");
      ctx.life.add(label + ":post-stop");
      if (cfg.get("post-stop-spawn") != null) { // creating a child while terminating
        String r;
        try { r = "\"ok\":true," + refJson(getContext().actorOf(Props.create(Noop.class), (String) cfg.get("post-stop-spawn"))); }
        catch (Exception e) { r = err(e).substring(1, err(e).length() - 1); }
        ctx.add(label, "{\"ev\":\"post-stop-spawn\"," + r + "}");
      }
      if (cfg.containsKey("post-stop-throws")) throw new IllegalStateException((String) cfg.get("post-stop-throws"));
    }

    @Override public Receive createReceive() {
      @SuppressWarnings("unchecked") List<Object> handles = (List<Object>) cfg.get("handles");
      org.apache.pekko.japi.pf.ReceiveBuilder b = receiveBuilder().match(Cmd.class, this::command);
      if (!Boolean.TRUE.equals(cfg.get("ignore-terminated"))) b.match(Terminated.class, this::terminated);
      // with `ignore-terminated` a Terminated is left unhandled: Actor.unhandled throws DeathPactException
      if (handles == null) return b.matchAny(o -> { if (o instanceof Terminated) unhandled(o); else handle(o); }).build();
      for (Object h : handles) b.matchEquals(h, this::handle);
      return b.build();
    }

    private void terminated(Terminated t) {
      String is = ctx.labelOf(t.getActor());
      ctx.add(label, "{\"ev\":\"terminated\",\"actor\":" + Cli.Json.str(t.getActor().path().toString())
          + ",\"is\":" + (is == null ? "null" : Cli.Json.str(is))
          + ",\"existenceConfirmed\":" + t.getExistenceConfirmed() + ",\"addressTerminated\":" + t.getAddressTerminated()
          + ",\"sender\":" + Cli.Json.str(getSender().path().toString()) + "}");
      ctx.life.add(label + ":terminated:" + is);
    }

    private String isJson(ActorRef r) { String is = ctx.labelOf(r); return "\"is\":" + (is == null ? "null" : Cli.Json.str(is)); }

    @SuppressWarnings("unchecked")
    private void command(Cmd c) {
      String res;
      try {
        Map<String, Object> sp = c.spec;
        switch ((String) sp.get("c")) {
          case "spawn-child": {
            Map<String, Object> beh = (Map<String, Object>) sp.get("behavior");
            String lb = (String) sp.get("as");
            Props props = beh == null ? Props.create(Noop.class) : Props.create(Scripted.class, () -> new Scripted(lb == null ? "_" : lb, beh, ctx));
            ActorRef r = sp.containsKey("name") ? getContext().actorOf(props, (String) sp.get("name")) : getContext().actorOf(props);
            if (lb != null) ctx.refs.put(lb, r);
            res = "\"ok\":true," + refJson(r);
            break;
          }
          case "child": {
            scala.Option<ActorRef> o = getContext().child((String) sp.get("name"));
            res = "\"ok\":true,\"found\":" + o.isDefined() + (o.isDefined() ? "," + refJson(o.get()) + "," + isJson(o.get()) : "");
            break;
          }
          case "children": {
            List<String> ps = new ArrayList<>();
            for (ActorRef k : getContext().getChildren()) ps.add(Cli.Json.str(k.path().toString()));
            java.util.Collections.sort(ps);
            res = "\"ok\":true,\"children\":[" + String.join(",", ps) + "]";
            break;
          }
          case "watch": getContext().watch(ctx.refs.get((String) sp.get("target"))); res = "\"ok\":true"; break;
          case "watch-with": getContext().watchWith(ctx.refs.get((String) sp.get("target")), message(sp.get("msg"))); res = "\"ok\":true"; break;
          case "unwatch": getContext().unwatch(ctx.refs.get((String) sp.get("target"))); res = "\"ok\":true"; break;
          case "stop": getContext().stop(ctx.refs.get((String) sp.get("target"))); res = "\"ok\":true"; break;
          case "select": getContext().actorSelection((String) sp.get("path")).tell(message(sp.get("msg")), getSelf()); res = "\"ok\":true"; break;
          default: throw new IllegalArgumentException("unknown command " + sp.get("c"));
        }
      } catch (Exception e) {
        // uids are random; `no-text`: Java renders the values of a termination message in ways Rust cannot (Q-DW-4)
        res = err(e).substring(1, err(e).length() - 1).replaceAll("#-?\\d+\\]", "#uid]");
        if (Boolean.TRUE.equals(c.spec.get("no-text"))) res = res.substring(0, res.indexOf(",\"message\""));
      }
      ctx.results.put(c.id, res);
    }

    private void handle(Object m) {
      ctx.add(label, "{\"ev\":\"received\",\"msg\":" + msgJson(m) + ",\"sender\":" + Cli.Json.str(getSender().path().toString())
          + (Boolean.TRUE.equals(cfg.get("numbered")) ? ",\"n\":" + (++received) : "") + "}");
      ctx.order.add(label + ":" + m);
      if (throwsOn(m)) throw new IllegalStateException((String) m);
      String kind = (String) cfg.getOrDefault("kind", "log");
      switch (kind) {
        case "echo": getSender().tell(m, getSelf()); break;
        case "forward": ctx.refs.get((String) cfg.get("to")).forward(m, getContext()); break;
        case "select-forward": getContext().actorSelection((String) cfg.get("path")).forward(m, getContext()); break;
        case "stop-on": if (m.equals(cfg.get("msg"))) getContext().stop(getSelf()); break;
        case "blast": {
          ActorRef to = ctx.refs.get((String) cfg.get("to"));
          for (long i = 0; i < (Long) cfg.get("count"); i++) to.tell(cfg.get("prefix") + "-" + i, getSelf());
          break;
        }
        case "gate": if (m.equals("block")) { try { ctx.gate.await(); } catch (InterruptedException e) { throw new RuntimeException(e); } } break;
        default: break;
      }
    }
  }

  /** `,"strategy":<class>` when a supervisor strategy is the `logClass` of an event (`logFailure`), nothing for any other reporter. */
  private static String strategyOf(Class<?> logClass) {
    return SupervisorStrategy.class.isAssignableFrom(logClass) ? ",\"strategy\":" + Cli.Json.str(logClass.getName()) : "";
  }

  /** `errors` channel: the published `Error`s (`logs` channel: also the `Warning`s) whose message starts with `prefix`.
   *  `watch`: the collector also watches that actor, so its mailbox shows the order of the errors and the `Terminated`. */
  public static final class ErrorCollector extends AbstractActor {
    private final String label, prefix;
    private final Ctx ctx;
    private final String watch;
    private final boolean causes;
    ErrorCollector(String label, String prefix, String watch, boolean causes, Ctx ctx) { this.label = label; this.prefix = prefix; this.watch = watch; this.causes = causes; this.ctx = ctx; }
    /** `causes`: the `getCause` chain below the published cause. */
    private String causedBy(Throwable cause) {
      if (!causes) return "";
      List<String> chain = new ArrayList<>();
      for (Throwable c = cause.getCause(); c != null; c = c.getCause()) chain.add(Cli.Json.str(c.toString()));
      return ",\"causedBy\":[" + String.join(",", chain) + "]";
    }
    @Override public void preStart() { if (watch != null) getContext().watch(ctx.refs.get(watch)); }
    @Override public Receive createReceive() {
      return receiveBuilder().match(org.apache.pekko.event.Logging.Error.class, e -> {
        if (String.valueOf(e.message()).startsWith(prefix))
          ctx.add(label, "{\"ev\":\"error\",\"source\":" + Cli.Json.str(e.logSource())
              + ",\"message\":" + (e.message() == null ? "null" : Cli.Json.str(String.valueOf(e.message())))
              + ",\"ofActorClass\":" + (e.logClass() == Scripted.class) + ",\"cause\":" + Cli.Json.str(e.cause().getClass().getName())
              + (e.cause().getClass().getName().endsWith("NoCause$") ? "" : ",\"causeText\":" + Cli.Json.str(e.cause().toString())) + causedBy(e.cause()) + strategyOf(e.logClass()) + "}");
      }).match(org.apache.pekko.event.Logging.Warning.class, w -> {
        if (String.valueOf(w.message()).startsWith(prefix))
          ctx.add(label, "{\"ev\":\"warning\",\"source\":" + Cli.Json.str(w.logSource()) + ",\"message\":" + (w.message() == null ? "null" : Cli.Json.str(String.valueOf(w.message())))
              + strategyOf(w.logClass()) + "}");
      }).match(Terminated.class, t -> ctx.add(label, "{\"ev\":\"watched-terminated\",\"actor\":" + Cli.Json.str(t.getActor().path().toString()) + "}"))
          .build();
    }
  }

  /** Subscribed to the event stream; records dead letters and unhandled messages. */
  public static final class Collector extends AbstractActor {
    private final String label;
    private final Ctx ctx;

    Collector(String label, Ctx ctx) { this.label = label; this.ctx = ctx; }

    @Override public Receive createReceive() {
      return receiveBuilder()
          .match(DeadLetter.class, d -> ctx.add(label, "{\"ev\":\"dead-letter\",\"msg\":" + msgJson(d.message())
              + ",\"sender\":" + Cli.Json.str(d.sender().path().toString())
              + ",\"recipient\":" + Cli.Json.str(d.recipient().path().toString()) + "}"))
          .match(UnhandledMessage.class, u -> ctx.add(label, "{\"ev\":\"unhandled\",\"msg\":" + msgJson(u.message())
              + ",\"sender\":" + Cli.Json.str(u.sender().path().toString())
              + ",\"recipient\":" + Cli.Json.str(u.recipient().path().toString()) + "}"))
          .build();
    }
  }

  @SuppressWarnings("unchecked")
  static String run(String text) {
    Map<String, Object> sc = (Map<String, Object>) Jv.parse(text);
    StringBuilder out = new StringBuilder("{\"system\":");
    String hocon = "pekko.loglevel = OFF\npekko.stdout-loglevel = OFF\n";
    Map<String, Object> disp = (Map<String, Object>) sc.get("dispatcher");
    if (disp != null) {
      hocon += "pekko.actor.default-dispatcher.executor = \"thread-pool-executor\"\n"
          + "pekko.actor.default-dispatcher.thread-pool-executor.fixed-pool-size = " + disp.get("parallelism") + "\n"
          + "pekko.actor.default-dispatcher.throughput = " + disp.get("throughput") + "\n";
    }
    ActorSystem system;
    try {
      system = ActorSystem.create((String) sc.get("system"), ConfigFactory.parseString(hocon));
    } catch (Exception e) {
      return out.append(err(e)).append(",\"steps\":[]}").toString();
    }
    Ctx ctx = new Ctx();
    ctx.system = system;
    try {
      ExtendedActorSystem ext = (ExtendedActorSystem) system;
      out.append("{\"ok\":true,\"name\":").append(Cli.Json.str(system.name()))
          .append(",\"toString\":").append(Cli.Json.str(system.toString()))
          .append(",\"root\":").append(Cli.Json.str(ext.provider().rootPath().toString()))
          .append(",\"user\":").append(Cli.Json.str(ext.guardian().path().toString()))
          .append(",\"systemGuardian\":").append(Cli.Json.str(ext.systemGuardian().path().toString()))
          .append(",\"deadLetters\":").append(Cli.Json.str(system.deadLetters().path().toString()))
          .append("},\"steps\":[");
      boolean first = true;
      for (Object o : (List<Object>) sc.get("steps")) {
        if (!first) out.append(',');
        first = false;
        out.append(step(ctx, (Map<String, Object>) o));
      }
      out.append("]}");
      return out.toString();
    } finally {
      ctx.gate.countDown();
      try {
        system.terminate();
        system.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS);
      } catch (Exception ignored) { }
    }
  }

  private static String err(Throwable e) {
    return "{\"ok\":false,\"exception\":" + Cli.Json.str(e.getClass().getName())
        + ",\"message\":" + (e.getMessage() == null ? "null" : Cli.Json.str(e.getMessage())) + "}";
  }

  private static String refJson(ActorRef r) {
    return "\"path\":" + Cli.Json.str(r.path().toString()) + ",\"name\":" + Cli.Json.str(r.path().name())
        + ",\"parent\":" + Cli.Json.str(r.path().parent().toString())
        + ",\"hasUid\":" + (r.path().uid() != 0);
  }

  private static ActorRef ref(Ctx ctx, String label) {
    return "deadLetters".equals(label) ? ctx.system.deadLetters() : ctx.refs.get(label);
  }

  /** Waits until `cond` holds or the timeout elapses; returns whether it holds. */
  private static boolean await(java.util.function.BooleanSupplier cond) throws InterruptedException { return await(cond, AWAIT_MS); }

  private static boolean await(java.util.function.BooleanSupplier cond, long ms) throws InterruptedException {
    long end = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(ms);
    while (!cond.getAsBoolean() && System.nanoTime() < end) Thread.sleep(2);
    return cond.getAsBoolean();
  }

  private static Object message(Object raw) {
    if (raw instanceof Map && ((Map<?, ?>) raw).get("custom") != null) return new WatchMsg((String) ((Map<?, ?>) raw).get("custom"));
    if (raw instanceof Map && "Identify".equals(((Map<?, ?>) raw).get("special"))) return new Identify(((Map<?, ?>) raw).get("id"));
    if (raw instanceof Map && "PoisonPill".equals(((Map<?, ?>) raw).get("special"))) return PoisonPill.getInstance();
    if (raw instanceof Map && "Kill".equals(((Map<?, ?>) raw).get("special"))) return Kill.getInstance();
    return raw;
  }

  @SuppressWarnings("unchecked")
  private static String step(Ctx ctx, Map<String, Object> st) {
    ActorSystem system = ctx.system;
    String op = (String) st.get("op");
    String head = "{\"op\":" + Cli.Json.str(op);
    try {
      switch (op) {
        case "spawn": {
          Map<String, Object> beh = (Map<String, Object>) st.get("behavior");
          String label = (String) st.get("as");
          Props props = beh == null ? Props.create(Noop.class)
              : Props.create(Scripted.class, () -> new Scripted(label == null ? "_" : label, beh, ctx));
          ActorRef r = st.containsKey("name") ? system.actorOf(props, (String) st.get("name")) : system.actorOf(props);
          if (label != null) ctx.refs.put(label, r);
          return head + ",\"ok\":true," + refJson(r) + "}";
        }
        case "subscribe": {
          String label = (String) st.get("as");
          boolean logs = "logs".equals(st.get("channel"));
          boolean errors = logs || "errors".equals(st.get("channel"));
          ActorRef c = system.actorOf(errors
              ? Props.create(ErrorCollector.class, () -> new ErrorCollector(label, (String) st.get("prefix"), (String) st.get("watch"), Boolean.TRUE.equals(st.get("causes")), ctx))
              : Props.create(Collector.class, () -> new Collector(label, ctx)));
          ctx.refs.put(label, c);
          system.eventStream().subscribe(c, errors ? org.apache.pekko.event.Logging.Error.class
              : "dead-letters".equals(st.get("channel")) ? DeadLetter.class : UnhandledMessage.class);
          if (logs) system.eventStream().subscribe(c, org.apache.pekko.event.Logging.Warning.class);
          return head + ",\"ok\":true}";
        }
        case "tell": {
          String target = (String) st.get("ref");
          Object msg = message(st.get("msg"));
          ActorRef from = st.get("from") == null ? ActorRef.noSender() : ctx.refs.get((String) st.get("from"));
          ctx.sent.computeIfAbsent(target, k -> new ArrayList<>());
          if (msg instanceof String) ctx.sent.get(target).add((String) msg);
          ref(ctx, target).tell(msg, from);
          return head + ",\"ok\":true}";
        }
        case "select": { // system.actorSelection(path).tell(msg, from)
          ActorRef from = st.get("from") == null ? ActorRef.noSender() : ctx.refs.get((String) st.get("from"));
          String path = (String) st.get("path"); // `uid-of` appends the uid of that actor, after a `#` in the path
          if (st.get("uid-of") != null) path += ctx.refs.get((String) st.get("uid-of")).path().uid();
          system.actorSelection(path).tell(message(st.get("msg")), from);
          return head + ",\"ok\":true}";
        }
        case "tell-seq": {
          String target = (String) st.get("ref");
          ActorRef from = st.get("from") == null ? ActorRef.noSender() : ctx.refs.get((String) st.get("from"));
          ctx.sent.computeIfAbsent(target, k -> new ArrayList<>());
          for (long i = 0; i < (Long) st.get("count"); i++) {
            String m = st.get("prefix") + "-" + i;
            ctx.sent.get(target).add(m);
            ref(ctx, target).tell(m, from);
          }
          return head + ",\"ok\":true}";
        }
        case "release": {
          ctx.gate.countDown();
          return head + ",\"ok\":true}";
        }
        case "stop": {
          system.stop(ctx.refs.get((String) st.get("ref")));
          return head + ",\"ok\":true}";
        }
        case "await-stopped": {
          InternalActorRef r = (InternalActorRef) ctx.refs.get((String) st.get("ref"));
          await(r::isTerminated);
          return head + ",\"ok\":true,\"terminated\":" + r.isTerminated() + "}";
        }
        case "terminate": {
          system.terminate();
          return head + ",\"ok\":true}";
        }
        case "await-system-terminated": {
          Terminated t = system.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS);
          return head + ",\"ok\":true,\"actor\":" + Cli.Json.str(t.getActor().path().toString())
              + ",\"existenceConfirmed\":" + t.getExistenceConfirmed()
              + ",\"addressTerminated\":" + t.getAddressTerminated() + "}";
        }
        case "await-events": { // the first `count` events of one actor, in order
          List<String> ev = ctx.of((String) st.get("target"));
          int n = ((Long) st.get("count")).intValue();
          boolean ok = await(() -> ev.size() >= n);
          List<String> shown = new ArrayList<>(ev.subList(0, Math.min(n, ev.size())));
          if (Boolean.TRUE.equals(st.get("sorted"))) java.util.Collections.sort(shown); // events whose order is a race (two Terminated)
          return head + ",\"ok\":" + ok + ",\"events\":[" + String.join(",", shown) + "]}";
        }
        case "open-hold": { // release a supervisor decider that waits on `supervisor.hold`
          ctx.life.add("hold-open:" + st.get("hold"));
          ctx.hold((String) st.get("hold")).countDown();
          return head + ",\"ok\":true}";
        }
        case "await-quiet": { // nothing beyond the first `count` events of an actor arrives within `ms` (a bounded wait for an absence)
          List<String> ev = ctx.of((String) st.get("target"));
          int n = ((Long) st.get("count")).intValue();
          boolean more = await(() -> ev.size() > n, ((Long) st.get("ms")));
          return head + ",\"ok\":true,\"quiet\":" + !more + ",\"events\":[" + String.join(",", ev.subList(0, Math.min(n, ev.size()))) + "]}";
        }
        case "await-order": { // the first `count` receives across all actors, in global order
          int n = ((Long) st.get("count")).intValue();
          boolean ok = await(() -> ctx.order.size() >= n);
          StringBuilder sb = new StringBuilder();
          for (int i = 0; i < Math.min(n, ctx.order.size()); i++) sb.append(i > 0 ? "," : "").append(Cli.Json.str(ctx.order.get(i)));
          return head + ",\"ok\":" + ok + ",\"order\":[" + sb + "]}";
        }
        case "await-received": { // invariants of the first `total` receives: per-sender count and in-order
          String target = (String) st.get("target");
          int total = ((Long) st.get("total")).intValue();
          boolean ok = await(() -> received(ctx, target).size() >= total);
          List<String[]> rcv = received(ctx, target);
          rcv = rcv.subList(0, Math.min(total, rcv.size()));
          Map<String, int[]> per = new TreeMap<>(); // sender -> {count, inOrder(1/0)}
          java.util.Set<String> seen = new java.util.HashSet<>();
          boolean dup = false;
          for (String[] r : rcv) {
            dup |= !seen.add(r[0] + "|" + r[1]);
            int[] p = per.computeIfAbsent(r[0], k -> new int[] {0, 1});
            long idx = Long.parseLong(r[1].substring(r[1].lastIndexOf('-') + 1));
            if (idx != p[0]) p[1] = 0;
            p[0]++;
          }
          StringBuilder sb = new StringBuilder();
          for (Map.Entry<String, int[]> e : per.entrySet())
            sb.append(sb.length() > 0 ? "," : "").append(Cli.Json.str(e.getKey())).append(":{\"count\":").append(e.getValue()[0])
                .append(",\"inOrder\":").append(e.getValue()[1] == 1).append("}");
          return head + ",\"ok\":" + ok + ",\"total\":" + rcv.size() + ",\"duplicates\":" + dup + ",\"senders\":{" + sb + "}}";
        }
        case "await-accounted": { // tell vs stop: every sent message was either received or dead-lettered, once
          String target = (String) st.get("target");
          String dl = (String) st.get("dl");
          int total = ((Long) st.get("total")).intValue();
          String recipient = ctx.refs.get(target).path().toString();
          boolean ok = await(() -> received(ctx, target).size() + deadLettersFor(ctx, dl, recipient).size() >= total);
          List<String> rcv = new ArrayList<>();
          for (String[] r : received(ctx, target)) rcv.add(r[1]);
          List<String> dls = deadLettersFor(ctx, dl, recipient);
          List<String> sent = ctx.sent.get(target);
          boolean prefix = rcv.size() <= sent.size() && rcv.equals(sent.subList(0, rcv.size()));
          List<String> rest = new ArrayList<>(sent.subList(Math.min(rcv.size(), sent.size()), sent.size()));
          java.util.Collections.sort(rest);
          List<String> dsorted = new ArrayList<>(dls);
          java.util.Collections.sort(dsorted);
          String counts = Boolean.TRUE.equals(st.get("counts"))
              ? ",\"receivedCount\":" + rcv.size() + ",\"deadLetterCount\":" + dls.size() : "";
          return head + ",\"ok\":" + ok + ",\"receivedIsPrefixOfSent\":" + prefix
              + ",\"deadLettersAreTheRest\":" + rest.equals(dsorted)
              + ",\"accounted\":" + (rcv.size() + dls.size() == total) + counts + "}";
        }
        case "do": { // run a command inside an actor and report its result
          long id = ctx.cmdIds.incrementAndGet();
          ctx.refs.get((String) st.get("ref")).tell(new Cmd(id, (Map<String, Object>) st.get("cmd")), ActorRef.noSender());
          if (Boolean.TRUE.equals(st.get("async"))) return head + ",\"ok\":true,\"queued\":true}"; // result is never read
          boolean ok = await(() -> ctx.results.containsKey(id));
          return head + (ok ? "," + ctx.results.get(id) : ",\"ok\":false,\"timeout\":true") + "}";
        }
        case "await-child-gone": { // poll `context.child(name)` of an actor until it is released
          String target = (String) st.get("ref");
          Map<String, Object> spec = new HashMap<>();
          spec.put("c", "child");
          spec.put("name", st.get("name"));
          boolean gone = await(() -> {
            long id = ctx.cmdIds.incrementAndGet();
            ctx.refs.get(target).tell(new Cmd(id, spec), ActorRef.noSender());
            try { await(() -> ctx.results.containsKey(id)); } catch (InterruptedException e) { throw new RuntimeException(e); }
            return "\"ok\":true,\"found\":false".equals(ctx.results.get(id));
          });
          return head + ",\"ok\":true,\"gone\":" + gone + "}";
        }
        case "events-now": { // everything one actor recorded so far (use after a barrier to prove nothing else arrived)
          List<String> ev = ctx.of((String) st.get("target"));
          return head + ",\"ok\":true,\"events\":[" + String.join(",", ev) + "]}";
        }
        case "events-sorted": { // the same as events-now, as a set (the order of Terminated from concurrent deaths is not guaranteed)
          List<String> ev = new ArrayList<>(ctx.of((String) st.get("target")));
          java.util.Collections.sort(ev);
          return head + ",\"ok\":true,\"events\":[" + String.join(",", ev) + "]}";
        }
        case "compare-refs": {
          ActorRef a = ctx.refs.get((String) st.get("a")), b = ctx.refs.get((String) st.get("b"));
          return head + ",\"ok\":true,\"pathEqual\":" + a.path().equals(b.path()) + ",\"refEqual\":" + a.equals(b)
              + ",\"uidEqual\":" + (a.path().uid() == b.path().uid()) + ",\"pathStringEqual\":" + a.path().toString().equals(b.path().toString())
              + ",\"aHasUid\":" + (a.path().uid() != 0) + ",\"bHasUid\":" + (b.path().uid() != 0) + "}";
        }
        case "await-life": { // lifecycle/termination events of all scripted actors; order only through the `before` constraints
          int n = ((Long) st.get("count")).intValue();
          boolean ok = await(() -> ctx.life.size() >= n);
          List<String> snap = new ArrayList<>(ctx.life);
          List<String> sorted = new ArrayList<>(snap);
          java.util.Collections.sort(sorted);
          StringBuilder ev = new StringBuilder(), cs = new StringBuilder();
          for (String e : sorted) ev.append(ev.length() > 0 ? "," : "").append(Cli.Json.str(e));
          List<Object> before = st.get("before") == null ? new ArrayList<>() : (List<Object>) st.get("before");
          for (Object o : before) {
            List<Object> pr = (List<Object>) o;
            int i = snap.indexOf(pr.get(0)), j = snap.indexOf(pr.get(1));
            cs.append(cs.length() > 0 ? "," : "").append(i >= 0 && j >= 0 && i < j);
          }
          return head + ",\"ok\":" + ok + ",\"events\":[" + ev + "],\"before\":[" + cs + "]}";
        }
        default:
          throw new IllegalArgumentException("unknown op " + op);
      }
    } catch (Exception e) {
      return head + "," + err(e).substring(1);
    }
  }

  /** (sender, msg) pairs of the `received` events of an actor, in order. */
  private static List<String[]> received(Ctx ctx, String label) {
    List<String[]> out = new ArrayList<>();
    for (String ev : ctx.of(label)) {
      if (!ev.startsWith("{\"ev\":\"received\"")) continue;
      Map<String, Object> m = (Map<String, Object>) Jv.parse(ev);
      Object msg = m.get("msg");
      if (msg instanceof String) out.add(new String[] {(String) m.get("sender"), (String) msg});
    }
    return out;
  }

  private static List<String> deadLettersFor(Ctx ctx, String dl, String recipient) {
    List<String> out = new ArrayList<>();
    for (String ev : ctx.of(dl)) {
      if (!ev.startsWith("{\"ev\":\"dead-letter\"")) continue;
      Map<String, Object> m = (Map<String, Object>) Jv.parse(ev);
      if (recipient.equals(m.get("recipient")) && m.get("msg") instanceof String) out.add((String) m.get("msg"));
    }
    return out;
  }
}
