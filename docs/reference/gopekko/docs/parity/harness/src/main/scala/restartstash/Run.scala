package restartstash {

import java.util.concurrent.{ CountDownLatch, Semaphore, TimeUnit }
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.Await
import scala.concurrent.duration._
import org.apache.pekko
import pekko.actor.{ ActorSystem, DeadLetterSuppression, Props }
import pekko.actor.typed.{ ActorRef, Behavior, BehaviorInterceptor, PostStop, SupervisorStrategy, Terminated => TTerminated, TypedActorContext }
import pekko.actor.typed.scaladsl.Behaviors
import pekko.actor.typed.scaladsl.adapter._
import unstash.{ Log, Watcher, Run => U }
import Log.log

// Oracle for what the restart stash of a Typed supervisor does with what it retained when the rebuilt behavior stops
// (#1182 slice B; QA of PR #1545). One scenario per JVM: `Run RS0` ... `Run RS3`. Reuses Log, Collector, Fences and Watcher of the
// `unstash` package. Every line is `[nnn +ms] text`.
//
// Shape (the same as W3 of typed-restart-child-wait-2026-11-06.md): A is `supervise(setup { ... }).onFailure[IllegalStateException](restart)`,
// has one child `c1` whose PostStop blocks on a latch and watches an actor X. "fail" makes A restart; the restart stops c1 and waits.
// What the main thread sends during the wait is retained in the supervisor's StashBuffer (Supervision.scala:265-286); the latch is then
// released, the restart completes and `stashBuffer.unstashAll(newBehavior)` replays it (Supervision.scala:405-407).
//
// A hook OUTSIDE the supervisor (Behaviors.intercept around supervise) logs and counts every message and signal after the supervisor
// returned from it, so "the supervisor has retained it" is proved by a log line and a counter, not by a sleep. The hook only delegates
// to the supervisor; it is declared here as part of the harness.
object Run {
  case object SuppMsg extends DeadLetterSuppression
  val setups = new AtomicInteger()
  val childBlocked = new CountDownLatch(1)
  val childLatch = new CountDownLatch(1)
  val handled = new Semaphore(0)

  def child: Behavior[Any] = Behaviors.receiveMessage[Any](_ => Behaviors.same[Any]).receiveSignal {
    case (_, PostStop) =>
      log("c1 PostStop begin"); childBlocked.countDown()
      childLatch.await(30, TimeUnit.SECONDS); log("c1 PostStop end"); Behaviors.same[Any]
  }

  // The inner behavior (what the supervisor restarts).
  def inner(x: ActorRef[Any], first: CountDownLatch): Behavior[Any] = Behaviors.setup[Any] { ctx =>
    val n = setups.incrementAndGet()
    log(s"A setup#$n children=${ctx.children.map(_.path.name).toList}")
    if (n == 1) { ctx.spawn(child, "c1"); ctx.watch(x); first.countDown() }
    Behaviors.receive[Any] { (_, m) =>
      m match {
        case "fail" => log("A msg fail: throws"); throw new IllegalStateException("boom")
        case "StopNow" => log("A msg StopNow -> Behaviors.stopped"); Behaviors.stopped[Any](() => log("A stopped callback"))
        case other => log(s"A msg $other"); Behaviors.same[Any]
      }
    }.receiveSignal {
      case (_, t: TTerminated) => log(s"A signal Terminated(${t.ref}) [${t.getClass.getName}]"); Behaviors.same[Any]
      case (_, sig)            => log(s"A signal $sig"); Behaviors.same[Any]
    }
  }

  // Outside the supervisor: logs and counts after the supervisor returned.
  def hook: BehaviorInterceptor[Any, Any] = new BehaviorInterceptor[Any, Any] {
    override def aroundReceive(ctx: TypedActorContext[Any], msg: Any, target: BehaviorInterceptor.ReceiveTarget[Any]): Behavior[Any] = {
      val r = target(ctx, msg)
      log(s"hook: supervisor returned for message $msg"); handled.release(); r
    }
    override def aroundSignal(ctx: TypedActorContext[Any], signal: pekko.actor.typed.Signal, target: BehaviorInterceptor.SignalTarget[Any]): Behavior[Any] = {
      val r = target(ctx, signal)
      log(s"hook: supervisor returned for signal $signal"); handled.release(); r
    }
  }

  def need(n: Int, what: String): Unit =
    require(handled.tryAcquire(n, 10, TimeUnit.SECONDS), s"timed out waiting for: $what")

  def main(args: Array[String]): Unit = {
    val wd = new Thread(() => { Thread.sleep(60000); println("WATCHDOG"); Runtime.getRuntime.halt(3) })
    wd.setDaemon(true); wd.start()
    log(s"java.version=${System.getProperty("java.version")} java.home=${System.getProperty("java.home")} scenario=${args(0)}")
    val sys = ActorSystem("rst", U.cfg)
    var code = 0
    try {
      val subs = U.subscribe(sys)
      run(sys, subs, args(0))
    } catch { case t: Throwable => log(s"HARNESS FAILURE $t"); code = 1 }
    log("-- end")
    sys.terminate(); Await.result(sys.whenTerminated, 10.seconds)
    Runtime.getRuntime.halt(code)
  }

  // RS0 control : m1, then X stops, no StopNow  -> the new behavior gets m1 and the Terminated(X) signal in order.
  // RS1 (a)     : m1, StopNow, m-behind-1, m-behind-2 retained -> the two behind are dead-lettered (sender, recipient).
  // RS2 (b)     : StopNow, then X stops (Terminated(X) retained behind StopNow), then m-behind-1.
  // RS3         : StopNow, SuppMsg (a DeadLetterSuppression user message), m-behind-1.
  def run(sys: ActorSystem, s: U.Subs, name: String): Unit = {
    val xTerminated = new CountDownLatch(1)
    val aTerminated = new CountDownLatch(1)
    val first = new CountDownLatch(1)
    val x = sys.spawn(Behaviors.receiveMessage[Any] { m => log(s"X got $m"); Behaviors.stopped[Any] }, "X")
    sys.actorOf(Props(new Watcher(x.toClassic, xTerminated)), "watcherX")
    val a = sys.spawn(
      Behaviors.intercept[Any, Any](() => hook)(
        Behaviors.supervise(inner(x, first)).onFailure[IllegalStateException](SupervisorStrategy.restart)),
      "A")
    sys.actorOf(Props(new Watcher(a.toClassic, aTerminated)), "watcherA")
    require(first.await(10, TimeUnit.SECONDS), "A did not start"); log("-- A started")
    a ! "fail"
    need(1, "supervisor handled fail")
    require(childBlocked.await(10, TimeUnit.SECONDS), "c1 PostStop did not begin"); log("-- restart is waiting for c1")
    def tellX(): Unit = {
      x ! "bye"
      require(xTerminated.await(10, TimeUnit.SECONDS), "X did not terminate")
      need(1, "supervisor handled the Terminated(X) signal")
    }
    def tell(m: Any): Unit = { a ! m; need(1, s"supervisor handled $m") }
    name match {
      case "RS0" => tell("m1"); tellX()
      case "RS1" => tell("m1"); tell("StopNow"); tell("m-behind-1"); tell("m-behind-2")
      case "RS2" => tell("StopNow"); tellX(); tell("m-behind-1")
      case "RS3" => tell("StopNow"); tell(SuppMsg); tell("m-behind-1")
    }
    log("-- everything is retained: release c1")
    childLatch.countDown()
    if (name == "RS0") {
      // control: the restart completes when the Terminated(c1) signal is handled; then A is running and the harness stops it.
      need(1, "the supervisor handled the Terminated(c1) signal and replayed the stash")
      tell("StopNow")
    }
    require(aTerminated.await(10, TimeUnit.SECONDS), "A did not terminate"); log("-- A terminated")
    U.fenceAll(s, "1"); log("-- fenced")
  }
}

}
