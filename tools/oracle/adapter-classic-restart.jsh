// Oracle for P55 review item C10: which path restarts a typed Behavior hosted by a
// classic ActorAdapter when its parent is a CLASSIC actor?
//   - typed restart:   Behaviors.supervise(...).onFailure(restart)  -> RestartSupervisor
//   - classic restart: the classic parent decides Restart for an exception that
//                      escapes the Behavior -> ActorAdapter.preRestart/postRestart
// For every PreRestart/PostStop signal the Behavior records the call stack markers
// (ActorAdapter.preRestart | ActorAdapter.postStop | RestartSupervisor) so the
// path is observed, not inferred. Causes: RuntimeException from the message handler
// (caught by typed supervision) and classic Kill (ActorKilledException, bypasses the
// Behavior and reaches the classic parent).
// Run: ORACLE=jars tools/oracle/run.sh tools/oracle/adapter-classic-restart.jsh
// (needs jshell >= 9; the repository asks for Java 21, see the P55 proposal for the
// JDK actually used). Output is one line per event: "<scenario> | <event>".
import org.apache.pekko.actor.typed.*;
import org.apache.pekko.actor.typed.javadsl.*;
import com.typesafe.config.ConfigFactory;
import java.time.Duration;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;

var events = new ConcurrentLinkedQueue<String>();
var current = new String[] { "?" };

void ev(String what) { events.add(current[0] + " | " + what); }

String via() {
  StringBuilder b = new StringBuilder();
  for (StackTraceElement e : Thread.currentThread().getStackTrace()) {
    String c = e.getClassName() + "." + e.getMethodName();
    if (c.endsWith("ActorAdapter.preRestart")) b.append("classic-hook:ActorAdapter.preRestart ");
    if (c.endsWith("ActorAdapter.postStop")) b.append("classic-hook:ActorAdapter.postStop ");
    if (e.getClassName().contains("internal.RestartSupervisor")) b.append("typed:RestartSupervisor ");
    if (e.getClassName().contains("ComposedStoppingBehavior")) b.append("ComposedStoppingBehavior ");
  }
  return b.toString().trim();
}

Behavior<String> child(AtomicInteger setups) {
  return Behaviors.setup(ctx -> {
    int n = setups.incrementAndGet();
    ev("setup#" + n);
    return Behaviors.withTimers(timers -> {
      timers.startTimerWithFixedDelay("T", "tick", Duration.ofSeconds(30));
      return Behaviors.<String>receive(String.class)
          .onMessageEquals("throw", () -> { throw new RuntimeException("boom"); })
          .onMessageEquals("tick", () -> Behaviors.same())
          .onSignal(Signal.class, s -> {
            ev("signal=" + s + " timerActive=" + timers.isTimerActive("T") + " via=[" + via() + "]");
            return Behaviors.same();
          })
          .build();
    });
  });
}

// Classic parent: spawns the typed child from its classic context (typed child wrapped
// in supervise(stop), rethrowTypedFailure=false) or with actorOf(Props adapter).
class Parent extends org.apache.pekko.actor.AbstractActor {
  final Behavior<?> behavior;
  final boolean viaProps;
  final org.apache.pekko.actor.SupervisorStrategy strategy;
  final CompletableFuture<org.apache.pekko.actor.ActorRef> out;
  Parent(Behavior<?> behavior, boolean viaProps, org.apache.pekko.actor.SupervisorStrategy strategy,
      CompletableFuture<org.apache.pekko.actor.ActorRef> out) {
    this.behavior = behavior; this.viaProps = viaProps; this.strategy = strategy; this.out = out;
  }
  @Override public org.apache.pekko.actor.SupervisorStrategy supervisorStrategy() { return strategy; }
  @Override public void preStart() {
    org.apache.pekko.actor.ActorRef c = viaProps
        ? getContext().actorOf(Adapter.props(() -> (Behavior<Object>) behavior), "kid")
        : Adapter.toClassic(Adapter.spawn(getContext(), (Behavior<Object>) behavior, "kid"));
    getContext().watch(c);
    out.complete(c);
  }
  @Override public Receive createReceive() {
    return receiveBuilder()
        .match(org.apache.pekko.actor.Terminated.class, t -> ev("parent saw Terminated(child)"))
        .build();
  }
}

org.apache.pekko.actor.SupervisorStrategy restartAll() {
  return new org.apache.pekko.actor.OneForOneStrategy(false,
      org.apache.pekko.japi.pf.DeciderBuilder.match(Throwable.class, e -> {
        ev("parent decider invoked, cause=" + e.getClass().getSimpleName() + " -> Restart");
        return org.apache.pekko.actor.SupervisorStrategy.restart();
      }).build());
}

var system = org.apache.pekko.actor.ActorSystem.create("c10",
    ConfigFactory.parseString("pekko.loglevel=OFF\npekko.stdout-loglevel=OFF"));

void scenario(String name, Behavior<String> b, boolean viaProps, boolean parentRestartsAll, Object stimulus)
    throws Exception {
  current[0] = name;
  var out = new CompletableFuture<org.apache.pekko.actor.ActorRef>();
  var strat = parentRestartsAll ? restartAll() : org.apache.pekko.actor.SupervisorStrategy.defaultStrategy();
  system.actorOf(org.apache.pekko.actor.Props.create(Parent.class, () -> new Parent(b, viaProps, strat, out)));
  var kid = out.get(5, TimeUnit.SECONDS);
  Thread.sleep(300);
  if (stimulus instanceof String) kid.tell(stimulus, org.apache.pekko.actor.ActorRef.noSender());
  else kid.tell(org.apache.pekko.actor.Kill.getInstance(), org.apache.pekko.actor.ActorRef.noSender());
  Thread.sleep(700);
  ev("kid terminated=" + kid.isTerminated());
}

void dump() {
  String e;
  while ((e = events.poll()) != null) System.out.println(e);
}

// S-A: typed restart supervisor inside; cause = RuntimeException from the handler; parent restarts everything.
scenario("S-A typed-supervise(restart) cause=RuntimeException parent=restartAll",
    Behaviors.supervise(child(new AtomicInteger())).onFailure(SupervisorStrategy.restart()), false, true, "throw");
// S-B: default spawn wrapping (supervise(stop)); cause = RuntimeException; parent restarts everything.
scenario("S-B no-user-supervise cause=RuntimeException parent=restartAll",
    child(new AtomicInteger()), false, true, "throw");
// S-C: cause = classic Kill (ActorKilledException); parent restarts everything.
scenario("S-C no-user-supervise cause=Kill parent=restartAll",
    child(new AtomicInteger()), false, true, Boolean.TRUE);
// S-D: same cause, parent keeps the classic default strategy (control).
scenario("S-D no-user-supervise cause=Kill parent=defaultStrategy",
    child(new AtomicInteger()), false, false, Boolean.TRUE);
// S-E: as S-C but spawned with context.actorOf(Adapter.props(...)) (PropsAdapter-style Props).
scenario("S-E Props-spawn cause=Kill parent=restartAll",
    child(new AtomicInteger()), true, true, Boolean.TRUE);
// S-F: typed-supervise(restart) inside + classic Kill + parent restarts everything.
scenario("S-F typed-supervise(restart) cause=Kill parent=restartAll",
    Behaviors.supervise(child(new AtomicInteger())).onFailure(SupervisorStrategy.restart()), false, true, Boolean.TRUE);

// Stale TimerMsg across a CLASSIC restart (new ActorAdapter => new ActorContextAdapter => new TimerSchedulerImpl).
// "armBlock" starts a 50 ms single timer and blocks 150 ms; Kill is queued at ~20 ms, so the TimerMsg lands
// in the mailbox AFTER Kill and is delivered to the new incarnation. recreateTimer decides whether the new
// incarnation registers a timer under the same key (so ActorAdapter reaches interceptTimerMsg) or none.
Behavior<Object> timerChild(AtomicInteger setups, boolean recreateTimer) {
  return Behaviors.setup(ctx -> {
    int n = setups.incrementAndGet();
    ev("setup#" + n);
    if (n > 1 && !recreateTimer) {
      return Behaviors.<Object>receive((c, m) -> {
        ev("incarnation#" + n + " (no timer) received " + (m instanceof String ? m : m.getClass().getName()));
        return Behaviors.same();
      });
    }
    return Behaviors.<Object>withTimers(timers -> {
      if (n > 1) timers.startSingleTimer("T", "late", Duration.ofSeconds(30));
      return Behaviors.<Object>receive((c, m) -> {
        if ("armBlock".equals(m)) {
          timers.startSingleTimer("T", "tick", Duration.ofMillis(50));
          Thread.sleep(150);
        } else {
          ev("incarnation#" + n + " received " + (m instanceof String ? m : m.getClass().getName()));
        }
        return Behaviors.same();
      });
    });
  });
}

void staleTimer(String name, boolean recreateTimer) throws Exception {
  current[0] = name;
  var out = new CompletableFuture<org.apache.pekko.actor.ActorRef>();
  var b = timerChild(new AtomicInteger(), recreateTimer);
  system.actorOf(org.apache.pekko.actor.Props.create(Parent.class,
      () -> new Parent(b, false, restartAll(), out)));
  var kid = out.get(5, TimeUnit.SECONDS);
  Thread.sleep(300);
  kid.tell("armBlock", org.apache.pekko.actor.ActorRef.noSender());
  Thread.sleep(20);
  kid.tell(org.apache.pekko.actor.Kill.getInstance(), org.apache.pekko.actor.ActorRef.noSender());
  Thread.sleep(900);
  ev("kid terminated=" + kid.isTerminated());
}
staleTimer("S-G classic-restart stale-TimerMsg new-incarnation-recreates-timer-T", true);
staleTimer("S-H classic-restart stale-TimerMsg new-incarnation-has-no-timer", false);
dump();
system.terminate();
Thread.sleep(500);
/exit
