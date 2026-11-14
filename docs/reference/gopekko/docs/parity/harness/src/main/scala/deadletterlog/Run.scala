package deadletterlog {

import java.util.concurrent.{ ConcurrentHashMap, CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.locks.LockSupport
import scala.concurrent.Await
import scala.concurrent.duration._
import com.typesafe.config.ConfigFactory
import org.apache.pekko
import pekko.actor.{
  Actor, ActorRef, ActorSystem, AllDeadLetters, DeadLetter, DeadLetterSuppression, Dropped, DlProbe, ExtendedActorSystem, PoisonPill,
  Props, Terminated, WrappedMessage
}
import pekko.event.Logging
import pekko.event.Logging.{ InitializeLogger, LogEventWithMarker, LoggerInitialized }

// Oracle for the INFO log of the dead letters (#1479, DeadLetterListener). One scenario per JVM: `Run LG1` ... `Run LG9`.
// Every line is `[nnn +ms] text`; nnn is the order in which log() was called. `info recv` lines are the Info events that the
// listener published on the event stream, as seen by a collector actor; the stdout `[INFO]` lines are the default logger's.
case object SuppMsg extends DeadLetterSuppression
case object PlainMsg
final case class W(message: Any) extends WrappedMessage
final case class Fence(id: String)

object Log {
  private val t0 = System.nanoTime()
  private val seq = new AtomicInteger()
  def log(s: String): Unit = synchronized { println(f"[${seq.incrementAndGet()}%03d +${(System.nanoTime() - t0) / 1000000}%5dms] $s") }
}
import Log.log

object Fences {
  private val m = new ConcurrentHashMap[String, CountDownLatch]()
  def latch(id: String): CountDownLatch = m.computeIfAbsent(id, _ => new CountDownLatch(1))
  // A direct tell from the publishing thread is queued behind every delivery that thread made before it (one mailbox).
  def fence(a: ActorRef, id: String): Unit = { a ! Fence(id); require(latch(id).await(10, TimeUnit.SECONDS), s"fence $id timed out") }
}

// Logs every dead letter it receives (DeadLetter, Dropped, UnhandledMessage...) with class, message, sender and recipient.
class Collector(name: String, count: AtomicInteger) extends Actor {
  def receive = {
    case Fence(id) => log(s"$name fence $id"); Fences.latch(id).countDown()
    case d: AllDeadLetters =>
      count.incrementAndGet()
      log(s"$name recv ${d.getClass.getSimpleName}(message=${d.message} [${d.message.getClass.getName}], sender=${d.sender}, recipient=${d.recipient})")
  }
}

// Logs the Info events (the lines the DeadLetterListener publishes): source, class, marker and message, verbatim.
class InfoCollector(name: String) extends Actor {
  def receive = {
    case Fence(id) => log(s"$name fence $id"); Fences.latch(id).countDown()
    case i: Logging.Info =>
      val mk = i match { case m: LogEventWithMarker => s" marker=${m.marker.name} props=${m.marker.properties}"; case _ => " marker=none" }
      log(s"$name recv source=${i.logSource} class=${i.logClass.getName} mdc=${i.mdc}$mk message=${i.message}")
  }
}

class Quiet extends Actor { def receive = { case _ => } }
class Watcher(target: ActorRef, latch: CountDownLatch) extends Actor {
  context.watch(target)
  def receive = { case Terminated(_) => latch.countDown() }
}
// The actors of DeadLetterSuspensionSpec.scala:25-40: one publishes Dropped, the other calls unhandled.
class Publisher(dropped: Boolean, done: AtomicInteger) extends Actor {
  def receive = { case n: Int => if (dropped) context.system.eventStream.publish(Dropped(n, "Don't like numbers", self)) else unhandled(n); done.incrementAndGet() }
}
// A /user actor that, when the system terminates, tells a dead actor from its PostStop (a dead letter during the shutdown).
class LateSender(victim: ActorRef, listener: ActorRef, off: Boolean, dl: ActorRef, info: ActorRef) extends Actor {
  def receive = { case _ => }
  override def postStop(): Unit = {
    log("late: PostStop of a /user actor tells 'late' to the dead victim"); victim ! "late"
    if (off) Run.poll("the listener is terminated", DlProbe.terminated(listener))
    else Run.poll("the listener has handled what was published", DlProbe.idle(listener))
    Fences.fence(dl, "late-dl"); Fences.fence(info, "late-info")
    log("late: PostStop done")
  }
}
// A logger (an actor, as in Pekko) that answers every dead-letter Info by telling a dead actor: a dead letter produced from a log entry.
class ReentrantLogger extends Actor {
  def receive = {
    case InitializeLogger(_) => sender() ! LoggerInitialized
    case i: Logging.Info if i.message.toString.contains("was not delivered") =>
      val n = Run.fromLogger.incrementAndGet()
      log(s"logger enter #$n"); Run.victim ! s"from-logger-$n"; log(s"logger exit #$n")
    case _ =>
  }
}

object Run {
  @volatile var victim: ActorRef = _
  val fromLogger = new AtomicInteger()
  final class Subs(val dl: ActorRef, val all: ActorRef, val info: ActorRef, val dlCount: AtomicInteger)

  def cfg(extra: String) = ConfigFactory.parseString(s"""
    pekko.loglevel = INFO
    pekko.use-slf4j = off
    pekko.actor.default-dispatcher.fork-join-executor.parallelism-max = 4
    $extra""").withFallback(ConfigFactory.load())

  def main(args: Array[String]): Unit = {
    val wd = new Thread(() => { Thread.sleep(60000); println("WATCHDOG"); Runtime.getRuntime.halt(3) })
    wd.setDaemon(true); wd.start()
    val sc = args(0)
    log(s"java.version=${System.getProperty("java.version")} java.home=${System.getProperty("java.home")} scenario=$sc")
    val inf = "pekko.log-dead-letters-suspend-duration = infinite"
    val extra = sc match {
      case "LG2" => "pekko.log-dead-letters = on"
      case "LG3" => "pekko.log-dead-letters = off"
      case "LG4" => s"pekko.log-dead-letters = 3\n$inf"
      case "LG5" => "pekko.log-dead-letters = 4\npekko.log-dead-letters-suspend-duration = 2s"
      case "LG8" => "pekko.log-dead-letters-during-shutdown = on"
      case "LG9" => s"""pekko.log-dead-letters = 3\n$inf\npekko.loggers = ["org.apache.pekko.event.Logging$$DefaultLogger", "deadletterlog.ReentrantLogger"]"""
      case _     => ""
    }
    val sys = ActorSystem("dlg", cfg(extra))
    val st = sys.settings
    log(s"settings LogDeadLetters=${st.LogDeadLetters} SuspendDuration=${st.LogDeadLettersSuspendDuration} DuringShutdown=${st.LogDeadLettersDuringShutdown} Loggers=${st.Loggers}")
    var code = 0
    try {
      val s = subscribe(sys, system = sc == "LG7" || sc == "LG8")
      sc match {
        case "LG1" => lg1(sys, s, 11)
        case "LG2" => lg1(sys, s, 12)
        case "LG3" => lg1(sys, s, 1)
        case "LG4" => lg4(sys, s)
        case "LG5" => lg5(sys, s)
        case "LG6" => lg6(sys, s)
        case "LG7" | "LG8" => lg7(sys, s, off = sc == "LG7")
        case "LG9" => lg9(sys, s)
      }
    } catch { case t: Throwable => log(s"HARNESS FAILURE $t"); code = 1 }
    log("-- end")
    sys.terminate(); Await.result(sys.whenTerminated, 10.seconds)
    Runtime.getRuntime.halt(code)
  }

  // Collectors on the event stream: DeadLetter, AllDeadLetters and the Info events (/system actors for the shutdown scenarios).
  def subscribe(sys: ActorSystem, system: Boolean): Subs = {
    val n = new AtomicInteger()
    def mk(p: Props, name: String): ActorRef = if (system) sys.asInstanceOf[ExtendedActorSystem].systemActorOf(p, name) else sys.actorOf(p, name)
    val dl = mk(Props(new Collector("dl", n)), "dl")
    val all = mk(Props(new Collector("all", new AtomicInteger())), "all")
    val info = mk(Props(new InfoCollector("info")), "info")
    sys.eventStream.subscribe(dl, classOf[DeadLetter]); sys.eventStream.subscribe(all, classOf[AllDeadLetters]); sys.eventStream.subscribe(info, classOf[Logging.Info])
    new Subs(dl, all, info, n)
  }

  def poll(what: String, cond: => Boolean): Unit = {
    val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10)
    while (!cond) { require(System.nanoTime() < deadline, s"timed out waiting for: $what"); LockSupport.parkNanos(100000L) }
    log(s"-- observed: $what")
  }

  def listener(sys: ActorSystem): ActorRef = {
    val p = sys.asInstanceOf[ExtendedActorSystem].provider
    p.resolveActorRef(p.rootPath.toString + "/system/deadLetterListener")
  }

  // Everything told by this thread has reached the listener (its mailbox is empty and not scheduled), and then the collectors.
  def barrier(sys: ActorSystem, s: Subs, id: String): Unit = {
    val l = listener(sys)
    if (!DlProbe.terminated(l)) poll(s"the listener is idle ($id)", DlProbe.idle(l)) else log(s"-- ($id) no live listener: terminated=true")
    Fences.fence(s.dl, s"dl-$id"); Fences.fence(s.all, s"all-$id"); Fences.fence(s.info, s"info-$id")
    log(s"-- ($id) fenced")
  }

  def dead(sys: ActorSystem, name: String): ActorRef = {
    val a = sys.actorOf(Props(new Quiet), name)
    val latch = new CountDownLatch(1)
    sys.actorOf(Props(new Watcher(a, latch))); sys.stop(a)
    require(latch.await(10, TimeUnit.SECONDS), s"$name did not terminate")
    log(s"-- $name terminated"); a
  }

  def say(s: String): Unit = log(s"-- $s")

  // LG1 (defaults), LG2 (log-dead-letters = on), LG3 (off): n integers to a dead actor; the first has a real sender (snd), the third
  // has deadLetters as sender, the others none.
  def lg1(sys: ActorSystem, s: Subs, n: Int): Unit = {
    val l = listener(sys); say(s"listener = $l terminated=${DlProbe.terminated(l)}")
    val v = dead(sys, "victim"); val snd = sys.actorOf(Props(new Quiet), "snd")
    (1 to n).foreach { i =>
      say(s"tell $i"); v.tell(i, if (i == 1) snd else if (i == 3) sys.deadLetters else ActorRef.noSender)
      if (i <= 3) barrier(sys, s, i.toString)
    }
    barrier(sys, s, "end")
  }

  // LG4 (log-dead-letters = 3, suspend-duration = infinite): the max count variant; the listener stops itself at the third.
  def lg4(sys: ActorSystem, s: Subs): Unit = {
    val l = listener(sys); say(s"listener = $l terminated=${DlProbe.terminated(l)}")
    val gone = new CountDownLatch(1); sys.actorOf(Props(new Watcher(l, gone)), "w")
    val v = dead(sys, "victim")
    say("tell 1, 2, 3"); (1 to 3).foreach(i => v.tell(i, ActorRef.noSender))
    say(s"the listener stopped itself = ${gone.await(10, TimeUnit.SECONDS)}"); barrier(sys, s, "3")
    say("tell 4"); v.tell(4, ActorRef.noSender); barrier(sys, s, "4")
  }

  // LG5 (log-dead-letters = 4, suspend-duration = 2s): the scenario of DeadLetterSuspensionSpec.scala:70-103, observed instead of read.
  def lg5(sys: ActorSystem, s: Subs): Unit = {
    val v = dead(sys, "deadActor"); val snd = sys.actorOf(Props(new Quiet), "snd")
    val d = new AtomicInteger(); val u = new AtomicInteger()
    val dropping = sys.actorOf(Props(new Publisher(true, d)), "droppingActor")
    val unh = sys.actorOf(Props(new Publisher(false, u)), "unhandledActor")
    say("1: deadActor ! 1"); v.tell(1, snd); barrier(sys, s, "1")
    say("2: droppingActor ! 2 (publishes Dropped)"); dropping.tell(2, snd); poll("Dropped published", d.get == 1); barrier(sys, s, "2")
    say("3: unhandledActor ! 3 (unhandled)"); unh.tell(3, snd); poll("unhandled called", u.get == 1); barrier(sys, s, "3")
    say("4: deadActor ! 4 (the limit is reached)"); v.tell(4, snd); val t4 = System.nanoTime(); barrier(sys, s, "4")
    say("5: deadActor ! 5, droppingActor ! 6 (suspended)"); v.tell(5, snd); dropping.tell(6, snd); poll("Dropped 6 published", d.get == 2); barrier(sys, s, "5")
    // The suspension is a time deadline (Deadline.now + 2s, checked only when a dead letter arrives): wait until 2.3 s after message 4.
    while (System.nanoTime() < t4 + TimeUnit.MILLISECONDS.toNanos(2300)) LockSupport.parkNanos(10000000L)
    say(s"the suspension elapsed (${(System.nanoTime() - t4) / 1000000} ms after message 4)")
    say("7: deadActor ! 7"); v.tell(7, snd); barrier(sys, s, "7")
    say("8: deadActor ! 8 (reset)"); v.tell(8, snd); barrier(sys, s, "8")
  }

  // LG6: suppressible messages are not logged and do not count; the control is [1].
  def lg6(sys: ActorSystem, s: Subs): Unit = {
    val v = dead(sys, "victim")
    say("tell SuppMsg, W(SuppMsg), PoisonPill"); v.tell(SuppMsg, ActorRef.noSender); v.tell(W(SuppMsg), ActorRef.noSender); v.tell(PoisonPill, ActorRef.noSender)
    barrier(sys, s, "1"); say("control: tell PlainMsg"); v.tell(PlainMsg, ActorRef.noSender); barrier(sys, s, "2")
  }

  // LG7 (during-shutdown off, the default) and LG8 (on): a dead letter produced while the system terminates.
  def lg7(sys: ActorSystem, s: Subs, off: Boolean): Unit = {
    val v = dead(sys, "victim")
    say("early: tell 'early' (before terminate)"); v.tell("early", ActorRef.noSender); barrier(sys, s, "early")
    sys.actorOf(Props(new LateSender(v, listener(sys), off, s.dl, s.info)), "late")
    say("terminate()"); sys.terminate(); Await.result(sys.whenTerminated, 10.seconds); say("terminated")
  }

  // LG9 (log-dead-letters = 3, infinite): a logger that answers every dead-letter Info with a dead letter.
  def lg9(sys: ActorSystem, s: Subs): Unit = {
    val v = dead(sys, "victim"); victim = v
    say("tell 'first'"); v.tell("first", ActorRef.noSender)
    poll("4 dead letters (1 + 3 from the logger)", s.dlCount.get == 4); barrier(sys, s, "1")
    say(s"the logger answered ${fromLogger.get} times")
  }
}

}

package org.apache.pekko.actor {
  // Mailbox and ref internals are private[pekko]; the harness reaches them from this package (polling instead of sleeping).
  object DlProbe {
    private def cell(a: ActorRef): ActorCell = a.asInstanceOf[ActorRefWithCell].underlying.asInstanceOf[ActorCell]
    def idle(a: ActorRef): Boolean = { val m = cell(a).mailbox; m.numberOfMessages == 0 && !m.isScheduled }
    def terminated(a: ActorRef): Boolean = a.asInstanceOf[InternalActorRef].isTerminated
  }
}
