package restartcap {

import java.util.concurrent.{ CountDownLatch, Semaphore, TimeUnit }
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.Await
import scala.concurrent.duration._
import com.typesafe.config.ConfigFactory
import org.apache.pekko
import pekko.actor.{ Actor, ActorSystem, DeadLetter, Dropped, Props, SuppressedDeadLetter }
import pekko.actor.typed.{ ActorRef, Behavior, BehaviorInterceptor, PostStop, SupervisorStrategy, Terminated => TTerminated, TypedActorContext }
import pekko.actor.typed.scaladsl.Behaviors
import pekko.actor.typed.scaladsl.adapter._
import unstash.{ Fence, Fences, Log, Watcher }
import Log.log

// Oracle for the capacity of the restart stash of plain `SupervisorStrategy.restart` (scope 999-restart-stash-bound.md, unit O1).
// One scenario per JVM: `Run CP0` ... Same shape as restartstash.Run (RS*): A = supervise(setup {...}).onFailure[IllegalStateException](restart...),
// child c1 whose PostStop blocks on a latch, a watched actor X, a hook OUTSIDE the supervisor that logs and counts what the supervisor returned.
// Collectors on the event stream: `dl` DeadLetter, `sd` SuppressedDeadLetter, `dr` Dropped (message, reason, sender, recipient).
//
//   CP0 : withStashCapacity(0): m1, m2, then X stops (Terminated(X)), then c1 released
//   CP2 : withStashCapacity(2): m1..m5
//   CPN : withStashCapacity(-1) and pekko.actor.typed.restart-stash-capacity = 3: m1..m5
//   CPS : pekko.actor.typed.restart-stash-capacity = -1 (system default): what happens
//   CS1 : withStashCapacity(2): m1, m2 (full), then X stops (non-awaited Terminated(X) at a full stash), then c1 released
//   CF1 : withStashCapacity(3): m1, fail2, m3 retained; replay of fail2 throws; setup#2's child c2 blocks; n1, n2, n3 sent during the second wait
//   CN1 : nested supervisors, inner withStashCapacity(1), outer withStashCapacity(3): m1, m2, m3
object Run {
  val setups = new AtomicInteger()
  val c1Blocked = new CountDownLatch(1)
  val c1Latch = new CountDownLatch(1)
  val c2Blocked = new CountDownLatch(1)
  val c2Latch = new CountDownLatch(1)
  val handled = new Semaphore(0)

  def child(n: String, blocked: CountDownLatch, latch: CountDownLatch): Behavior[Any] =
    Behaviors.receiveMessage[Any](_ => Behaviors.same[Any]).receiveSignal {
      case (_, PostStop) =>
        log(s"$n PostStop begin"); blocked.countDown()
        latch.await(30, TimeUnit.SECONDS); log(s"$n PostStop end"); Behaviors.same[Any]
    }

  def inner(x: ActorRef[Any], first: CountDownLatch, spawnC2: Boolean): Behavior[Any] = Behaviors.setup[Any] { ctx =>
    val n = setups.incrementAndGet()
    log(s"A setup#$n children=${ctx.children.map(_.path.name).toList}")
    if (n == 1) { ctx.spawn(child("c1", c1Blocked, c1Latch), "c1"); ctx.watch(x); first.countDown() }
    if (n == 2 && spawnC2) ctx.spawn(child("c2", c2Blocked, c2Latch), "c2")
    Behaviors.receive[Any] { (_, m) =>
      m match {
        case "fail" | "fail2" => log(s"A msg $m: throws"); throw new IllegalStateException("boom")
        case "StopNow"        => log("A msg StopNow -> Behaviors.stopped"); Behaviors.stopped[Any](() => log("A stopped callback"))
        case other            => log(s"A msg $other"); Behaviors.same[Any]
      }
    }.receiveSignal {
      case (_, t: TTerminated) => log(s"A signal Terminated(${t.ref}) [${t.getClass.getName}]"); Behaviors.same[Any]
      case (_, sig)            => log(s"A signal $sig"); Behaviors.same[Any]
    }
  }

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

  class Coll(name: String) extends Actor {
    def receive = {
      case Fence(id) => log(s"$name fence $id"); Fences.latch(id).countDown()
      case d: Dropped =>
        log(s"$name recv Dropped(message=${d.message} [${d.message.getClass.getName}], reason=${d.reason}, sender=${d.sender}, recipient=${d.recipient})")
      case d: DeadLetter => log(s"$name recv DeadLetter(message=${d.message}, sender=${d.sender}, recipient=${d.recipient})")
      case e             => log(s"$name recv $e")
    }
  }

  def need(n: Int, what: String): Unit =
    require(handled.tryAcquire(n, 10, TimeUnit.SECONDS), s"timed out waiting for: $what")

  def cfg(extra: String) = ConfigFactory.parseString(extra).withFallback(unstash.Run.cfg)

  def main(args: Array[String]): Unit = {
    val wd = new Thread(() => { Thread.sleep(60000); println("WATCHDOG"); Runtime.getRuntime.halt(3) })
    wd.setDaemon(true); wd.start()
    log(s"java.version=${System.getProperty("java.version")} java.home=${System.getProperty("java.home")} scenario=${args(0)}")
    var code = 0
    val sys = ActorSystem("rcap", args(0) match {
      case "CPN" => cfg("pekko.actor.typed.restart-stash-capacity = 3")
      case "CPS" => cfg("pekko.actor.typed.restart-stash-capacity = -1")
      case _     => cfg("")
    })
    try run(sys, args(0)) catch { case t: Throwable => log(s"HARNESS FAILURE $t"); code = 1 }
    log("-- end")
    sys.terminate(); Await.result(sys.whenTerminated, 10.seconds)
    Runtime.getRuntime.halt(code)
  }

  def run(sys: ActorSystem, name: String): Unit = {
    val dl = sys.actorOf(Props(new Coll("dl")), "dl")
    val sd = sys.actorOf(Props(new Coll("sd")), "sd")
    val dr = sys.actorOf(Props(new Coll("dr")), "dr")
    sys.eventStream.subscribe(dl, classOf[DeadLetter])
    sys.eventStream.subscribe(sd, classOf[SuppressedDeadLetter])
    sys.eventStream.subscribe(dr, classOf[Dropped])
    val xTerminated = new CountDownLatch(1)
    val aTerminated = new CountDownLatch(1)
    val first = new CountDownLatch(1)
    val x = sys.spawn(Behaviors.receiveMessage[Any] { m => log(s"X got $m"); Behaviors.stopped[Any] }, "X")
    sys.actorOf(Props(new Watcher(x.toClassic, xTerminated)), "watcherX")
    val in = inner(x, first, spawnC2 = name == "CF1")
    def sup(b: Behavior[Any], cap: Int) =
      Behaviors.supervise(b).onFailure[IllegalStateException](SupervisorStrategy.restart.withStashCapacity(cap))
    val supervised: Behavior[Any] = name match {
      case "CP0" => sup(in, 0)
      case "CP2" | "CS1" => sup(in, 2)
      case "CPN" | "CPS" => sup(in, -1)
      case "CF1" => sup(in, 3)
      case "CN1" => sup(sup(in, 1), 3)
    }
    val a = try sys.spawn(Behaviors.intercept[Any, Any](() => hook)(supervised), "A")
    catch { case t: Throwable => log(s"spawn of A threw $t"); return }
    sys.actorOf(Props(new Watcher(a.toClassic, aTerminated)), "watcherA")
    require(first.await(10, TimeUnit.SECONDS), "A did not start"); log("-- A started")
    a ! "fail"
    need(1, "supervisor handled fail")
    require(c1Blocked.await(10, TimeUnit.SECONDS), "c1 PostStop did not begin"); log("-- restart is waiting for c1")
    def tellX(): Unit = {
      x ! "bye"
      require(xTerminated.await(10, TimeUnit.SECONDS), "X did not terminate")
      need(1, "supervisor handled the Terminated(X) signal")
    }
    def tell(m: Any): Unit = { a ! m; need(1, s"supervisor handled $m") }
    name match {
      case "CP0" => tell("m1"); tell("m2"); tellX()
      case "CP2" | "CPN" | "CN1" => (1 to (if (name == "CN1") 3 else 5)).foreach(i => tell(s"m$i"))
      case "CS1" => tell("m1"); tell("m2"); tellX()
      case "CF1" => tell("m1"); tell("fail2"); tell("m3")
      case "CPS" => ()
    }
    log("-- everything is sent: release c1")
    c1Latch.countDown()
    need(1, "the supervisor handled the Terminated(c1) signal and replayed the stash")
    if (name == "CF1") {
      require(c2Blocked.await(10, TimeUnit.SECONDS), "c2 PostStop did not begin"); log("-- second restart is waiting for c2")
      tell("n1"); tell("n2"); tell("n3")
      log("-- release c2")
      c2Latch.countDown()
      need(1, "the supervisor handled the Terminated(c2) signal and replayed the stash")
    }
    tell("StopNow")
    require(aTerminated.await(10, TimeUnit.SECONDS), "A did not terminate"); log("-- A terminated")
    Seq(dl, sd, dr).foreach(c => Fences.fence(c, s"${c.path.name}-1")); log("-- fenced")
  }
}

}
