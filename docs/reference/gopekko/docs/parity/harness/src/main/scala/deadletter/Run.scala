package deadletter {

import java.util.concurrent.{ ConcurrentHashMap, CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.Await
import scala.concurrent.duration._
import com.typesafe.config.ConfigFactory
import org.apache.pekko
import pekko.actor.{
  Actor, ActorRef, ActorSystem, AllDeadLetters, DeadLetter, DeadLetterSuppression, Kill, PoisonPill, Props,
  SuppressedDeadLetter, Terminated, WrappedMessage
}
import pekko.actor.{ StopHelper, TerminatedHelper }
import pekko.event.Logging

// Oracle for the dead-letter suppression of story #871 (ADR 0010 divergence 5a, ADR 0032 D3/D4/D6, ADR 0038 D7). One scenario per JVM:
// `Run DL1` ... `Run DL8`, plus DL1b. Every line is `[nnn +ms] text`; nnn is the order in which log() was called.
case object SuppressedMsg extends DeadLetterSuppression
case object PlainMsg
case object Control
final case class W(message: Any) extends WrappedMessage
final case class Fence(id: String)
final case class Block(entered: CountDownLatch, gate: CountDownLatch)
final case class BlockThenStop(entered: CountDownLatch, gate: CountDownLatch)

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
  // A direct tell from the publishing thread is queued behind every delivery that thread made before it (one mailbox,
  // FIFO per sender thread), so the fence proves "everything published before has arrived".
  def fence(a: ActorRef, id: String): Unit = {
    a ! Fence(id)
    require(latch(id).await(10, TimeUnit.SECONDS), s"fence $id timed out")
  }
}

// Logs every event it receives with class, message, sender and recipient (identity against deadLetters is shown too).
class Collector(name: String) extends Actor {
  def receive = {
    case Fence(id) => log(s"$name fence $id"); Fences.latch(id).countDown()
    case e =>
      log(s"$name recv ${show(e)}")
      e match { case d: AllDeadLetters if d.message.toString.startsWith("Terminate") => Fences.latch(s"term-$name").countDown(); case _ => }
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

// Collects the Info events (the lines the default DeadLetterListener publishes); signals when one contains `needle`.
class InfoCollector(needle: String, seen: CountDownLatch) extends Actor {
  def receive = {
    case i: Logging.Info =>
      log(s"info recv source=${i.logSource} message=${i.message}")
      if (i.message.toString.contains(needle)) seen.countDown()
    case Fence(id) => Fences.latch(id).countDown()
  }
}

// Tells `latch` when `target` is terminated: the mailbox has been detached (dead-letter mailbox installed) before Terminated is delivered.
class Watcher(target: ActorRef, latch: CountDownLatch) extends Actor {
  context.watch(target)
  def receive = { case Terminated(_) => latch.countDown() }
}

// Blocks inside receive on `Block` until released; other messages are consumed.
class Blocker extends Actor {
  def receive = {
    case Block(entered, gate) =>
      entered.countDown(); log("blocker is inside receive, blocked")
      gate.await(30, TimeUnit.SECONDS); log("blocker leaves receive")
    case BlockThenStop(entered, gate) =>
      entered.countDown(); log("blocker is inside receive, blocked (will context.stop(self))")
      gate.await(30, TimeUnit.SECONDS); log("blocker calls context.stop(self)"); context.stop(self)
    case m => log(s"blocker recv $m")
  }
}

object Run {
  def cfg(logging: Boolean) = ConfigFactory.parseString(s"""
    ${if (logging) "" else "pekko.loglevel = \"OFF\"\n    pekko.stdout-loglevel = \"OFF\""}
    pekko.actor.default-dispatcher.fork-join-executor.parallelism-max = 4
    bounded-mb {
      mailbox-type = "org.apache.pekko.dispatch.BoundedMailbox"
      mailbox-capacity = 1
      mailbox-push-timeout-time = 0s
    }
  """).withFallback(ConfigFactory.load())

  final class Subs(val dl: ActorRef, val sd: ActorRef, val all: ActorRef)

  def main(args: Array[String]): Unit = {
    val wd = new Thread(() => { Thread.sleep(60000); println("WATCHDOG"); Runtime.getRuntime.halt(3) })
    wd.setDaemon(true); wd.start()
    log(s"java.version=${System.getProperty("java.version")} java.home=${System.getProperty("java.home")} scenario=${args(0)}")
    val sys = ActorSystem("dls", cfg(logging = args(0) == "DL6"))
    var code = 0
    try {
      val subs = subscribe(sys)
      args(0) match {
        case "DL1"  => dl1(sys, subs, withSender = false)
        case "DL1b" => dl1(sys, subs, withSender = true)
        case "DL2"  => dl2(sys, subs)
        case "DL3"  => dl3(sys, subs)
        case "DL4"  => dl4(sys, subs)
        case "DL5"  => dl5(sys, subs)
        case "DL6"  => dl6(sys, subs)
        case "DL7"  => dl7(sys, subs)
        case "DL8"  => dl8(sys, subs, "internal")
        case "DL8b" => dl8(sys, subs, "sysstop")
        case "DL8c" => dl8(sys, subs, "self")
      }
    } catch { case t: Throwable => log(s"HARNESS FAILURE $t"); code = 1 }
    log("-- end")
    sys.terminate(); Await.result(sys.whenTerminated, 10.seconds)
    Runtime.getRuntime.halt(code)
  }

  // Three collectors on the event stream: DeadLetter, SuppressedDeadLetter and AllDeadLetters.
  def subscribe(sys: ActorSystem): Subs = {
    val dl = sys.actorOf(Props(new Collector("dl")), "dl")
    val sd = sys.actorOf(Props(new Collector("sd")), "sd")
    val all = sys.actorOf(Props(new Collector("all")), "all")
    sys.eventStream.subscribe(dl, classOf[DeadLetter])
    sys.eventStream.subscribe(sd, classOf[SuppressedDeadLetter])
    sys.eventStream.subscribe(all, classOf[AllDeadLetters])
    new Subs(dl, sd, all)
  }

  def fenceAll(s: Subs, id: String): Unit = {
    Fences.fence(s.dl, s"dl-$id"); Fences.fence(s.sd, s"sd-$id"); Fences.fence(s.all, s"all-$id")
  }

  def probe(sys: ActorSystem, name: String): ActorRef = sys.actorOf(Props(new Collector(name)), name)

  def stopAndWait(sys: ActorSystem, a: ActorRef, stopIt: Boolean = true): Unit = {
    val latch = new CountDownLatch(1)
    sys.actorOf(Props(new Watcher(a, latch)))
    if (stopIt) sys.stop(a)
    require(latch.await(10, TimeUnit.SECONDS), s"$a did not terminate")
  }

  def terminated(sys: ActorSystem, name: String): ActorRef = {
    val a = sys.actorOf(Props(new Blocker), name)
    stopAndWait(sys, a)
    log(s"-- $name terminated")
    a
  }

  def dl1(sys: ActorSystem, s: Subs, withSender: Boolean): Unit = {
    val v = terminated(sys, "victim")
    val snd = if (withSender) probe(sys, "probe") else ActorRef.noSender
    log(s"-- tell PoisonPill to victim, sender = ${if (withSender) "probe" else "ActorRef.noSender"}")
    v.tell(PoisonPill, snd)
    fenceAll(s, "1"); log("-- fenced")
  }

  def dl2(sys: ActorSystem, s: Subs): Unit = {
    val v = terminated(sys, "victim")
    log("-- tell SuppressedMsg to victim, sender = ActorRef.noSender")
    v.tell(SuppressedMsg, ActorRef.noSender)
    fenceAll(s, "1"); log("-- fenced")
  }

  def dl3(sys: ActorSystem, s: Subs): Unit = {
    val v = terminated(sys, "victim")
    val x = terminated(sys, "x")
    val t = TerminatedHelper.mk(x)
    log(s"-- tell $t to victim, sender = ActorRef.noSender")
    v.tell(t, ActorRef.noSender)
    fenceAll(s, "1"); log("-- fenced")
  }

  def dl4(sys: ActorSystem, s: Subs): Unit = {
    val v = terminated(sys, "victim")
    log("-- tell Kill to victim, sender = ActorRef.noSender")
    v.tell(Kill, ActorRef.noSender)
    fenceAll(s, "1"); log("-- fenced")
  }

  def dl5(sys: ActorSystem, s: Subs): Unit = {
    val p = probe(sys, "probe")
    val some = terminated(sys, "some")
    log("-- (a) deadLetters.tell(DeadLetter(SuppressedMsg, probe, some), noSender)")
    sys.deadLetters.tell(DeadLetter(SuppressedMsg, p, some), ActorRef.noSender)
    fenceAll(s, "a"); log("-- (a) fenced")
    log("-- (b) deadLetters.tell(PoisonPill, noSender)")
    sys.deadLetters.tell(PoisonPill, ActorRef.noSender)
    fenceAll(s, "b"); log("-- (b) fenced")
    log("-- (c) control: deadLetters.tell(DeadLetter(PlainMsg, probe, some), noSender)")
    sys.deadLetters.tell(DeadLetter(PlainMsg, p, some), ActorRef.noSender)
    fenceAll(s, "c"); log("-- (c) fenced")
  }

  // The default DeadLetterListener is on (default loglevel). W(SuppressedMsg) is told first, then W(PlainMsg) as a
  // positive control: the listener handles the DeadLetters in publication order, so once the Info of the control is seen
  // the decision about the first one has been taken.
  def dl6(sys: ActorSystem, s: Subs): Unit = {
    val seen = new CountDownLatch(1)
    val ic = sys.actorOf(Props(new InfoCollector("PlainMsg", seen)), "info")
    sys.eventStream.subscribe(ic, classOf[Logging.Info])
    val v = terminated(sys, "victim")
    log("-- tell W(SuppressedMsg) to victim, sender = ActorRef.noSender")
    v.tell(W(SuppressedMsg), ActorRef.noSender)
    fenceAll(s, "1"); log("-- (1) fenced")
    log("-- tell W(PlainMsg) to victim (control), sender = ActorRef.noSender")
    v.tell(W(PlainMsg), ActorRef.noSender)
    fenceAll(s, "2"); log("-- (2) fenced")
    log(s"-- Info for the control seen = ${seen.await(10, TimeUnit.SECONDS)}")
  }

  def dl7(sys: ActorSystem, s: Subs): Unit = {
    val a = sys.actorOf(Props(new Blocker).withMailbox("bounded-mb"), "victim")
    val entered = new CountDownLatch(1); val gate = new CountDownLatch(1)
    a.tell(Block(entered, gate), ActorRef.noSender)
    require(entered.await(10, TimeUnit.SECONDS)); log("-- blocker entered")
    log("-- tell m2 (fills the mailbox of capacity 1)")
    a.tell("m2", ActorRef.noSender)
    fenceAll(s, "0"); log("-- (0) fenced, nothing published so far if no recv above")
    log("-- tell PlainMsg (control) to the full mailbox, sender = ActorRef.noSender")
    a.tell(PlainMsg, ActorRef.noSender)
    fenceAll(s, "1"); log("-- (1) fenced")
    log("-- tell PoisonPill to the full mailbox, sender = ActorRef.noSender")
    a.tell(PoisonPill, ActorRef.noSender)
    fenceAll(s, "2"); log("-- (2) fenced")
    gate.countDown()
    stopAndWait(sys, a)
    fenceAll(s, "3"); log("-- (3) fenced after the stop")
  }

  // mode "internal": InternalActorRef.stop() from the test thread (Terminate goes straight to the victim's system queue);
  // "sysstop": ActorSystem.stop (an ordinary StopChild message to the guardian, asynchronous); "self": the victim calls context.stop(self).
  def dl8(sys: ActorSystem, s: Subs, mode: String): Unit = {
    val a = sys.actorOf(Props(new Blocker), "victim")
    val entered = new CountDownLatch(1); val gate = new CountDownLatch(1)
    a.tell(if (mode == "self") BlockThenStop(entered, gate) else Block(entered, gate), ActorRef.noSender)
    require(entered.await(10, TimeUnit.SECONDS)); log("-- blocker entered")
    log("-- tell m2, then PoisonPill, sender = ActorRef.noSender")
    a.tell("m2", ActorRef.noSender)
    a.tell(PoisonPill, ActorRef.noSender)
    fenceAll(s, "0"); log("-- (0) fenced, nothing published so far if no recv above")
    val latch = new CountDownLatch(1)
    sys.actorOf(Props(new Watcher(a, latch)))
    mode match {
      case "internal" => log("-- InternalActorRef.stop(victim) (Terminate system message), then release the gate"); StopHelper.stop(a)
      case "sysstop"  => log("-- sys.stop(victim) (StopChild to the guardian), then release the gate"); sys.stop(a)
      case _          => log("-- release the gate; the victim stops itself")
    }
    gate.countDown()
    require(latch.await(10, TimeUnit.SECONDS), "victim did not terminate")
    log("-- victim terminated")
    fenceAll(s, "1"); log("-- (1) fenced after the stop")
    if (mode == "sysstop") // the late Terminate() of the guardian's StopChild reaches the dead victim asynchronously: wait for it, bounded
      log(s"-- Terminate() dead letter seen at sd = ${Fences.latch("term-sd").await(5, TimeUnit.SECONDS)}")
  }
}

}

package org.apache.pekko.actor {
  // Terminated has a private[pekko] constructor; this helper lives in the package so the harness can build one.
  object TerminatedHelper {
    def mk(a: ActorRef): Terminated = Terminated(a)(existenceConfirmed = true, addressTerminated = false)
  }
  // InternalActorRef is private[pekko]; stop() sends the Terminate system message directly to the actor.
  object StopHelper {
    def stop(a: ActorRef): Unit = a.asInstanceOf[InternalActorRef].stop()
  }
}
