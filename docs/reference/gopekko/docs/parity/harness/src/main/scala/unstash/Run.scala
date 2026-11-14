package unstash {

import java.util.concurrent.{ ConcurrentHashMap, CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.Await
import scala.concurrent.duration._
import com.typesafe.config.ConfigFactory
import org.apache.pekko
import pekko.actor.{ Actor, ActorRef, ActorSystem, AllDeadLetters, DeadLetter, DeadLetterSuppression, Props, SuppressedDeadLetter }
import pekko.actor.typed.{ Behavior, Terminated => TTerminated }
import pekko.actor.typed.scaladsl.Behaviors
import pekko.actor.typed.scaladsl.adapter._
import pekko.actor.MailboxProbe

// Oracle for the dead letters of a Typed stash and of a retained Terminated (scopes of #1182 and #927, Owner decision 2026-11-10 on #1524).
// One scenario per JVM: `Run US1` ... `Run US4` and `Run US6`. Every line is `[nnn +ms] text`; nnn is the order in which log() was called.
case object SuppMsg extends DeadLetterSuppression
final case class Fence(id: String)

object Log {
  private val t0 = System.nanoTime()
  private val seq = new AtomicInteger()
  def log(s: String): Unit = synchronized {
    println(f"[${seq.incrementAndGet()}%03d +${(System.nanoTime() - t0) / 1000000}%5dms] $s")
  }
}
import Log.log

object Fences {
  private val m = new ConcurrentHashMap[String, CountDownLatch]()
  def latch(id: String): CountDownLatch = m.computeIfAbsent(id, _ => new CountDownLatch(1))
  // A direct tell from the publishing thread is queued behind every delivery that thread made before it, so the fence
  // proves "everything published before has arrived" (the eventStream publish is synchronous to the subscriber mailbox).
  def fence(a: ActorRef, id: String): Unit = {
    a ! Fence(id)
    require(latch(id).await(10, TimeUnit.SECONDS), s"fence $id timed out")
  }
}

// Logs every event with class, message, sender and recipient (identity against deadLetters is shown too).
class Collector(name: String) extends Actor {
  def receive = {
    case Fence(id) => log(s"$name fence $id"); Fences.latch(id).countDown()
    case e =>
      log(s"$name recv ${show(e)}")
  }
  def show(e: Any): String = {
    val dls = context.system.deadLetters
    e match {
      case d: AllDeadLetters =>
        s"${d.getClass.getName}(message=${d.message} [${d.message.getClass.getName}], sender=${d.sender}, recipient=${d.recipient}) " +
        s"[sender eq deadLetters = ${d.sender eq dls}, recipient eq deadLetters = ${d.recipient eq dls}]"
      case o => o.toString
    }
  }
}

// Tells `latch` when `target` is terminated (a classic watcher): the mailbox has been detached by then.
class Watcher(target: ActorRef, latch: CountDownLatch) extends Actor {
  context.watch(target)
  def receive = { case pekko.actor.Terminated(_) => latch.countDown() }
}

object Run {
  def cfg = ConfigFactory.parseString("""
    pekko.loglevel = "OFF"
    pekko.stdout-loglevel = "OFF"
    pekko.actor.default-dispatcher.fork-join-executor.parallelism-max = 4
  """).withFallback(ConfigFactory.load())

  final class Subs(val dl: ActorRef, val sd: ActorRef, val all: ActorRef)

  def main(args: Array[String]): Unit = {
    val wd = new Thread(() => { Thread.sleep(60000); println("WATCHDOG"); Runtime.getRuntime.halt(3) })
    wd.setDaemon(true); wd.start()
    log(s"java.version=${System.getProperty("java.version")} java.home=${System.getProperty("java.home")} scenario=${args(0)}")
    val sys = ActorSystem("uns", cfg)
    var code = 0
    try {
      val subs = subscribe(sys)
      args(0) match {
        case "US1" => us(sys, subs, "US1", unstashFull = true, wrap = false, stopAt = "m2", withSupp = false)
        case "US2" => us(sys, subs, "US2", unstashFull = false, wrap = true, stopAt = "m1", withSupp = true)
        case "US3" => us(sys, subs, "US3", unstashFull = true, wrap = false, stopAt = "<stopped initially>", withSupp = false)
        case "US4" => us(sys, subs, "US4", unstashFull = true, wrap = false, stopAt = "m1", withSupp = true)
        case "US6" => us6(sys, subs)
      }
    } catch { case t: Throwable => log(s"HARNESS FAILURE $t"); code = 1 }
    log("-- end")
    sys.terminate(); Await.result(sys.whenTerminated, 10.seconds)
    Runtime.getRuntime.halt(code)
  }

  def subscribe(sys: ActorSystem): Subs = {
    val dl = sys.actorOf(Props(new Collector("dl")), "dl")
    val sd = sys.actorOf(Props(new Collector("sd")), "sd")
    val all = sys.actorOf(Props(new Collector("all")), "all")
    sys.eventStream.subscribe(dl, classOf[DeadLetter])
    sys.eventStream.subscribe(sd, classOf[SuppressedDeadLetter])
    sys.eventStream.subscribe(all, classOf[pekko.actor.AllDeadLetters])
    new Subs(dl, sd, all)
  }

  def fenceAll(s: Subs, id: String): Unit = {
    Fences.fence(s.dl, s"dl-$id"); Fences.fence(s.sd, s"sd-$id"); Fences.fence(s.all, s"all-$id")
  }

  // US1..US5: a typed actor stashes m1, m2, m3, m4 (and SuppMsg when withSupp, after m3), then "Go" replays them into a behavior that
  // stops on `stopAt` (US3: the behavior is Behaviors.stopped from the start).
  //   US1: unstashAll, replay stops on m2           -> m3, m4 are the rest
  //   US2: unstash(b, 3, wrap = "w:" + _), stops on the first replayed message, SuppMsg in the stash -> rest of the 3 taken
  //   US3: unstashAll(Behaviors.stopped)            -> m1..m4 all go to dead letters (the "initial behavior is not alive" branch)
  //   US4: unstashAll, stops on m1, SuppMsg in the stash (identity wrap) -> a DeadLetterSuppression message among the rest
  def us(sys: ActorSystem, s: Subs, name: String, unstashFull: Boolean, wrap: Boolean, stopAt: String, withSupp: Boolean): Unit = {
    val done = new CountDownLatch(1)
    val terminated = new CountDownLatch(1)
    val msgs: List[Any] = List("m1", "m2") ++ (if (withSupp) List(SuppMsg) else Nil) ++ List("m3", "m4")
    val replay: Behavior[Any] =
      if (stopAt == "<stopped initially>") Behaviors.stopped[Any]
      else Behaviors.receiveMessage[Any] { m =>
        log(s"replay behavior got $m")
        if (m.toString == stopAt || m.toString == s"w:$stopAt") Behaviors.stopped[Any] else Behaviors.same[Any]
      }
    val a = sys.spawn(Behaviors.withStash[Any](10) { buf =>
      Behaviors.receive[Any] { (ctx, m) =>
        m match {
          case "Go" =>
            log(s"actor ${ctx.self.path} Go: stash size ${buf.size}")
            val b = if (unstashFull) buf.unstashAll(replay) else buf.unstash(replay, 3, (x: Any) => "w:" + x)
            log(s"unstash returned (stash size now ${buf.size})")
            done.countDown()
            b
          case other => buf.stash(other); Behaviors.same[Any]
        }
      }
    }, s"stasher-$name")
    val tw = sys.actorOf(Props(new Watcher(a.toClassic, terminated)), "watcher")
    msgs.foreach(m => a ! m)
    log(s"-- told ${msgs.mkString(", ")} then Go (typed tell: no sender)")
    a ! "Go"
    require(done.await(10, TimeUnit.SECONDS), "unstash did not return")
    require(terminated.await(10, TimeUnit.SECONDS), "actor did not terminate")
    log("-- actor terminated")
    fenceAll(s, "1"); log("-- fenced")
  }

  // US6: a Terminated retained in the user queue when a typed actor stops.
  //  A watches B. A is blocked inside "go". B stops; the death-watch notification reaches A's system queue (polled with
  //  MailboxProbe, no sleep). "stop" is queued behind "go". The gate opens: after "go" the mailbox processes the system queue
  //  first, which enqueues Terminated(B) in A's user queue behind "stop" (polled again: 2 messages); A handles "stop" and returns
  //  Behaviors.stopped, with Terminated(B) still queued.
  def us6(sys: ActorSystem, s: Subs): Unit = {
    val entered = new CountDownLatch(1)
    val gate = new CountDownLatch(1)
    val watching = new CountDownLatch(1)
    val aTerminated = new CountDownLatch(1)
    val b = sys.spawn(Behaviors.receiveMessage[Any] { m => log(s"B got $m"); Behaviors.stopped[Any] }, "B")
    val a = sys.spawn(Behaviors.receive[Any] { (ctx, m) =>
      m match {
        case "watch" => ctx.watch(b); log("A watches B"); watching.countDown(); Behaviors.same[Any]
        case "go" =>
          log("A is inside go, blocked"); entered.countDown()
          gate.await(30, TimeUnit.SECONDS); log("A leaves go"); Behaviors.same[Any]
        case "stop" =>
          // The system queue was processed after "go": Terminated(B) is now in the user queue, behind "stop".
          poll("A has 1 queued user message (Terminated(B)) while handling stop", MailboxProbe.numberOfMessages(ctx.self.toClassic) == 1)
          log("A handles stop: Behaviors.stopped"); Behaviors.stopped[Any]
        case other => log(s"A got $other"); Behaviors.same[Any]
      }
    }.receiveSignal {
      case (_, TTerminated(ref)) => log(s"A signal Terminated($ref)"); Behaviors.same[Any]
      case (_, sig)               => log(s"A signal $sig"); Behaviors.same[Any]
    }, "A")
    sys.actorOf(Props(new Watcher(a.toClassic, aTerminated)), "watcherA")
    a ! "watch"
    require(watching.await(10, TimeUnit.SECONDS)); log("-- A watches B")
    a ! "go"
    require(entered.await(10, TimeUnit.SECONDS)); log("-- A blocked")
    b ! "bye"
    poll("A has a system message", MailboxProbe.hasSystemMessages(a.toClassic))
    a ! "stop"
    log("-- release the gate")
    gate.countDown()
    require(aTerminated.await(10, TimeUnit.SECONDS), "A did not terminate")
    log("-- A terminated")
    fenceAll(s, "1"); log("-- fenced")
  }

  def poll(what: String, cond: => Boolean): Unit = {
    val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10)
    while (!cond) {
      require(System.nanoTime() < deadline, s"timed out waiting for: $what")
      java.util.concurrent.locks.LockSupport.parkNanos(100000L)
    }
    log(s"-- observed: $what")
  }
}

}

package org.apache.pekko.actor {
  // ActorCell.mailbox is private[pekko]: reach the queue sizes of a (typed adapter) actor, to prove a state before acting.
  object MailboxProbe {
    private def cell(a: ActorRef): ActorCell = a.asInstanceOf[ActorRefWithCell].underlying.asInstanceOf[ActorCell]
    def hasSystemMessages(a: ActorRef): Boolean = cell(a).mailbox.hasSystemMessages
    def numberOfMessages(a: ActorRef): Int = cell(a).mailbox.numberOfMessages
  }
}
