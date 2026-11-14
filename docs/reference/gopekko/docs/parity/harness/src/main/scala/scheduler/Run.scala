package scheduler {

import java.util.concurrent.{ CountDownLatch, Executors, Semaphore, ThreadFactory, TimeUnit }
import java.util.concurrent.atomic.{ AtomicInteger, AtomicReference }
import scala.concurrent.{ Await, ExecutionContext }
import scala.concurrent.duration._
import com.typesafe.config.ConfigFactory
import org.apache.pekko
import pekko.actor.{ Access, Actor, ActorRef, ActorSystem, Cancellable, DeadLetter, Props, Terminated }
import pekko.event.Logging

// Oracle for the scheduler lifecycle (ADR 0039, issue #776). One scenario per JVM: `Run SC1` ... (names in main).
// Every line is `[nnn +ms] text`; nnn is the order in which log() was called.
object Log {
  private val t0 = System.nanoTime()
  private val seq = new AtomicInteger()
  def log(s: String): Unit = synchronized {
    println(f"[${seq.incrementAndGet()}%03d +${(System.nanoTime() - t0) / 1000000}%5dms] $s")
  }
}
import Log.log

// Listens to Logging.Error and DeadLetter on the event stream.
class Listener(errs: Semaphore, dls: Semaphore, errCount: AtomicInteger, dlCount: AtomicInteger) extends Actor {
  def receive = {
    case e: Logging.Error =>
      errCount.incrementAndGet()
      log(s"eventstream Error cause=${e.cause} logSource=${e.logSource} logClass=${e.logClass.getName} message=${e.message}")
      errs.release()
    case d: DeadLetter =>
      dlCount.incrementAndGet()
      log(s"eventstream DeadLetter message=${d.message} sender=${d.sender} recipient=${d.recipient}")
      dls.release()
  }
}

// Counts what it receives; releases `got` per message.
class Victim(got: Semaphore, n: AtomicInteger) extends Actor {
  def receive = { case m => log(s"victim recv $m on ${Thread.currentThread.getName}"); n.incrementAndGet(); got.release() }
}
class Watcher(target: ActorRef, done: CountDownLatch) extends Actor {
  context.watch(target)
  def receive = { case Terminated(_) => log("watcher saw Terminated"); done.countDown() }
}

object Run {
  val cfg = ConfigFactory.parseString("""
    pekko.loglevel = "INFO"
    pekko.stdout-loglevel = "INFO"
    pekko.actor.default-dispatcher.fork-join-executor.parallelism-max = 4
  """).withFallback(ConfigFactory.load())

  def r(f: => Unit): Runnable = new Runnable { def run(): Unit = f }
  def th: String = Thread.currentThread.getName
  def desc(c: Cancellable): String = s"${c.getClass.getName} isCancelled=${c.isCancelled}"
  def ms(d: FiniteDuration): String = d.toString

  var sys: ActorSystem = _
  var disp: ExecutionContext = _
  val errs = new Semaphore(0); val dls = new Semaphore(0)
  val errCount = new AtomicInteger; val dlCount = new AtomicInteger

  // The callers thread runs the task: shows that the scheduler calls ec.execute inside the schedule call.
  val sameThread: ExecutionContext = new ExecutionContext {
    def execute(run: Runnable): Unit = run.run()
    def reportFailure(t: Throwable): Unit = log(s"sameThread reportFailure $t")
  }
  // A plain executor-backed context with a reporter, to see who reports a task failure.
  lazy val plain: ExecutionContext = {
    val tf = new ThreadFactory {
      def newThread(x: Runnable): Thread = { val t = new Thread(x, "plain-ec-thread"); t.setDaemon(true); t }
    }
    ExecutionContext.fromExecutor(Executors.newCachedThreadPool(tf), t => log(s"plain reporter reportFailure on $th: $t"))
  }

  def attempt[T](label: String)(f: => T): Option[T] =
    try { val v = f; log(s"$label -> returned ${v match { case c: Cancellable => desc(c); case o => o }}"); Some(v) }
    catch { case t: Throwable => log(s"$label -> threw ${t.getClass.getName}: ${t.getMessage}"); None }

  // The bounded fence: a one-shot task on the scheduler under test; used to observe "nothing else happened" for `w` ms.
  def window(w: Long): Unit = {
    val l = new CountDownLatch(1)
    sys.scheduler.scheduleOnce(w.millis, r(l.countDown()))(disp)
    require(l.await(10, TimeUnit.SECONDS), "window timed out")
    log(s"-- window ${w}ms elapsed")
  }

  def periodic(kind: String)(init: FiniteDuration, iv: FiniteDuration)(run: Runnable)(implicit ec: ExecutionContext): Cancellable =
    if (kind == "fd") sys.scheduler.scheduleWithFixedDelay(init, iv)(run)(ec)
    else sys.scheduler.scheduleAtFixedRate(init, iv)(run)(ec)

  def main(args: Array[String]): Unit = {
    val wd = new Thread(() => { Thread.sleep(60000); println("WATCHDOG"); Runtime.getRuntime.halt(3) })
    wd.setDaemon(true); wd.start()
    Thread.setDefaultUncaughtExceptionHandler((t: Thread, e: Throwable) => log(s"uncaught exception handler: thread=${t.getName} $e"))
    log(s"java.version=${System.getProperty("java.version")} scenario=${args(0)} caller thread=$th")
    sys = ActorSystem("sch", cfg)
    disp = sys.dispatcher
    sys.eventStream.subscribe(sys.actorOf(Props(new Listener(errs, dls, errCount, dlCount)), "listener"), classOf[Logging.Error])
    sys.eventStream.subscribe(sys.actorOf(Props(new Listener(errs, dls, errCount, dlCount)), "listener-dl"), classOf[DeadLetter])
    var code = 0
    try {
      val a = args(0)
      a match {
        case "SC1"                 => sc1()
        case "SC2fd"               => sc2("fd")
        case "SC2fr"               => sc2("fr")
        case "SC3fd"               => sc3p("fd")
        case "SC3fr"               => sc3p("fr")
        case "SC3o"                => sc3o()
        case _ if a.startsWith("SC4") =>
          // SC4{fd|fr}-{disp|global|plain} : RuntimeException; SC4s{fd|fr}: SchedulerException; SC4i{fd|fr}: ISE caused by SchedulerException
          val (what, rest) = if (a.startsWith("SC4s")) ("s", a.drop(4)) else if (a.startsWith("SC4i")) ("i", a.drop(4)) else ("r", a.drop(3))
          val kind = rest.take(2)
          val ec = rest.drop(3) match { case "global" => ExecutionContext.global; case "plain" => plain; case _ => disp }
          sc4(kind, what, ec, if (rest.length > 2) rest.drop(3) else "disp")
        case "SC5once"             => sc5(None, live = false)
        case "SC5fd"               => sc5(Some("fd"), live = false)
        case "SC5fr"               => sc5(Some("fr"), live = false)
        case "SC5fdlive"           => sc5(Some("fd"), live = true)
        case "SC5frlive"           => sc5(Some("fr"), live = true)
      }
    } catch { case t: Throwable => log(s"HARNESS FAILURE $t"); code = 1 }
    log("-- end")
    sys.terminate(); Await.result(sys.whenTerminated, 10.seconds)
    Runtime.getRuntime.halt(code)
  }

  // SC1: one-shot with delay zero and negative.
  def sc1(): Unit = {
    val sch = sys.scheduler
    for ((lbl, d) <- Seq[(String, FiniteDuration)]("Zero" -> Duration.Zero, "-1s" -> (-1).second, "-1ns" -> Duration(-1, NANOSECONDS), "+1ns(control)" -> Duration(1, NANOSECONDS))) {
      val l = new CountDownLatch(1); val t0 = System.nanoTime(); @volatile var ranOn = ""
      val h = attempt(s"[$lbl] scheduleOnce(runnable) on dispatcher") {
        sch.scheduleOnce(d, r { ranOn = th; log(s"[$lbl] task ran on $th after ${(System.nanoTime() - t0) / 1000} us"); l.countDown() })(disp)
      }
      require(l.await(5, TimeUnit.SECONDS), "task did not run")
      h.foreach { c => log(s"[$lbl] after run: cancel()=${c.cancel()} isCancelled=${c.isCancelled} cancel() again=${c.cancel()}") }
      // the task runs on the callers thread, inside the schedule call
      val l2 = new CountDownLatch(1)
      log(s"[$lbl] before scheduleOnce on sameThread EC (caller=$th)")
      val h2 = attempt(s"[$lbl] scheduleOnce(runnable) on sameThread EC") {
        sch.scheduleOnce(d, r { log(s"[$lbl] sameThread task ran on $th"); l2.countDown() })(sameThread)
      }
      log(s"[$lbl] sameThread latch count after return = ${l2.getCount}")
      h2.foreach { c => log(s"[$lbl] sameThread handle: cancel()=${c.cancel()} isCancelled=${c.isCancelled}") }
      // message form
      val got = new Semaphore(0); val n = new AtomicInteger
      val v = sys.actorOf(Props(new Victim(got, n)))
      val h3 = attempt(s"[$lbl] scheduleOnce(receiver, msg)") { sch.scheduleOnce(d, v, s"m-$lbl")(disp, Actor.noSender) }
      require(got.tryAcquire(5, TimeUnit.SECONDS), "message not received")
      h3.foreach { c => log(s"[$lbl] message handle after delivery: cancel()=${c.cancel()} isCancelled=${c.isCancelled}") }
    }
  }

  // SC2: periodic, zero and negative interval, zero and negative initial delay.
  def sc2(kind: String): Unit = {
    val rejected = new AtomicInteger
    for ((lbl, iv) <- Seq[(String, FiniteDuration)]("Zero" -> Duration.Zero, "-1s" -> (-1).second, "-1ns" -> Duration(-1, NANOSECONDS))) {
      attempt(s"$kind runnable init=100ms interval=$lbl") { periodic(kind)(100.millis, iv)(r(rejected.incrementAndGet()))(disp) }
      val v = sys.actorOf(Props(new Victim(new Semaphore(0), new AtomicInteger)))
      attempt(s"$kind message init=100ms interval=$lbl") {
        if (kind == "fd") sys.scheduler.scheduleWithFixedDelay(100.millis, iv, v, "x")(disp, Actor.noSender)
        else sys.scheduler.scheduleAtFixedRate(100.millis, iv, v, "x")(disp, Actor.noSender)
      }
      attempt(s"$kind runnable init=Zero interval=$lbl") { periodic(kind)(Duration.Zero, iv)(r(rejected.incrementAndGet()))(disp) }
    }
    window(400)
    log(s"runs of the rejected schedules after the window = ${rejected.get}")
    // accepted forms: initial delay zero / negative / 1ns interval
    for ((lbl, init, iv) <- Seq[(String, FiniteDuration, FiniteDuration)](
        ("init=Zero interval=100ms", Duration.Zero, 100.millis),
        ("init=-1s interval=100ms", (-1).second, 100.millis),
        ("init=100ms interval=1ns", 100.millis, Duration(1, NANOSECONDS)))) {
      val n = new AtomicInteger; val l = new CountDownLatch(3); val t0 = System.nanoTime(); val tsRef = new AtomicReference(Vector.empty[Long])
      val h = attempt(s"$kind $lbl") {
        periodic(kind)(init, iv)(r {
          val k = n.incrementAndGet()
          log(s"$kind $lbl run $k on $th at +${(System.nanoTime() - t0) / 1000} us since the call")
          l.countDown()
        })(disp)
      }
      require(l.await(5, TimeUnit.SECONDS), s"$lbl: fewer than 3 runs")
      h.foreach { c => log(s"$kind $lbl: cancel()=${c.cancel()} isCancelled=${c.isCancelled}") }
    }
  }

  // SC3 periodic: Cancellable of a periodic task.
  def sc3p(kind: String): Unit = {
    // (a) before the first run
    val runsA = new AtomicInteger
    val ha = periodic(kind)(500.millis, 100.millis)(r(runsA.incrementAndGet()))(disp)
    log(s"(a) created: ${desc(ha)}")
    log(s"(a) cancel()=${ha.cancel()} isCancelled=${ha.isCancelled}")
    log(s"(a) cancel() again=${ha.cancel()} isCancelled=${ha.isCancelled}")
    window(800)
    log(s"(a) runs after window = ${runsA.get}")
    // (b) after some runs, while the task is idle
    val runsB = new AtomicInteger; val l2 = new CountDownLatch(2)
    val hb = periodic(kind)(100.millis, 400.millis)(r { val k = runsB.incrementAndGet(); log(s"(b) run $k"); l2.countDown() })(disp)
    require(l2.await(5, TimeUnit.SECONDS)); Thread.sleep(100) // bounded sleep: the interval is 400ms, so the task is idle (run 3 not due)
    log(s"(b) idle: runs=${runsB.get} isCancelled=${hb.isCancelled}")
    log(s"(b) cancel()=${hb.cancel()} isCancelled=${hb.isCancelled}")
    log(s"(b) cancel() again=${hb.cancel()} isCancelled=${hb.isCancelled}")
    window(800)
    log(s"(b) runs after window = ${runsB.get}")
    // (c) cancel from another thread while the task is running (blocked in the callback)
    val runsC = new AtomicInteger; val entered = new CountDownLatch(1); val gate = new CountDownLatch(1)
    val hc = periodic(kind)(50.millis, 100.millis)(r {
      val k = runsC.incrementAndGet(); log(s"(c) run $k begins")
      if (k == 1) { entered.countDown(); gate.await(10, TimeUnit.SECONDS) }; log(s"(c) run $k ends")
    })(disp)
    require(entered.await(5, TimeUnit.SECONDS))
    log(s"(c) during the run: cancel()=${hc.cancel()} isCancelled=${hc.isCancelled}")
    log(s"(c) cancel() again=${hc.cancel()}")
    gate.countDown(); window(500)
    log(s"(c) runs after window = ${runsC.get}")
    // (e) cancel from inside the callback (self-cancel on run 2)
    val ref = new AtomicReference[Cancellable]; val runsE = new AtomicInteger; val selfDone = new CountDownLatch(1)
    ref.set(periodic(kind)(100.millis, 100.millis)(r {
      val k = runsE.incrementAndGet()
      if (k == 2) { val c = ref.get.cancel(); log(s"(e) run 2: self cancel()=$c isCancelled=${ref.get.isCancelled}"); selfDone.countDown() } else log(s"(e) run $k")
    })(disp))
    require(selfDone.await(5, TimeUnit.SECONDS)); window(500)
    log(s"(e) runs after window = ${runsE.get}; cancel() again=${ref.get.cancel()} isCancelled=${ref.get.isCancelled}")
    // (d) after the repetition was ended by an exception in the callback (run 1 throws)
    val runsD = new AtomicInteger; val ld = new CountDownLatch(1)
    val hd = periodic(kind)(100.millis, 100.millis)(r { val k = runsD.incrementAndGet(); log(s"(d) run $k throws"); ld.countDown(); throw new RuntimeException("boom-d") })(disp)
    require(ld.await(5, TimeUnit.SECONDS)); require(errs.tryAcquire(5, TimeUnit.SECONDS), "no Error event")
    log(s"(d) after the abort: runs=${runsD.get} isCancelled=${hd.isCancelled}")
    log(s"(d) cancel()=${hd.cancel()} isCancelled=${hd.isCancelled}")
    log(s"(d) cancel() again=${hd.cancel()} isCancelled=${hd.isCancelled}")
    window(400)
    log(s"(d) runs after window = ${runsD.get}")
  }

  // SC3 one-shot.
  def sc3o(): Unit = {
    val sch = sys.scheduler
    val runsA = new AtomicInteger
    val ha = sch.scheduleOnce(500.millis, r(runsA.incrementAndGet()))(disp)
    log(s"(a) created: ${desc(ha)}")
    log(s"(a) cancel()=${ha.cancel()} isCancelled=${ha.isCancelled}")
    log(s"(a) cancel() again=${ha.cancel()} isCancelled=${ha.isCancelled}")
    window(800); log(s"(a) runs after window = ${runsA.get}")
    val l = new CountDownLatch(1)
    val hb = sch.scheduleOnce(50.millis, r { log("(b) task ran"); l.countDown() })(disp)
    require(l.await(5, TimeUnit.SECONDS))
    log(s"(b) after run: isCancelled=${hb.isCancelled}")
    log(s"(b) cancel()=${hb.cancel()} isCancelled=${hb.isCancelled}")
    log(s"(b) cancel() again=${hb.cancel()} isCancelled=${hb.isCancelled}")
    val entered = new CountDownLatch(1); val gate = new CountDownLatch(1)
    val hc = sch.scheduleOnce(50.millis, r { log("(c) task begins"); entered.countDown(); gate.await(10, TimeUnit.SECONDS); log("(c) task ends") })(disp)
    require(entered.await(5, TimeUnit.SECONDS))
    log(s"(c) during the run: cancel()=${hc.cancel()} isCancelled=${hc.isCancelled}")
    gate.countDown(); window(100)
  }

  // SC4: a periodic task throws on run 2. what: r = RuntimeException, s = SchedulerException, i = ISE caused by SchedulerException.
  def sc4(kind: String, what: String, ec: ExecutionContext, ecName: String): Unit = {
    val runs = new AtomicInteger; val l2 = new CountDownLatch(1); val t0 = System.nanoTime()
    log(s"SC4 kind=$kind throws=$what ec=$ecName")
    val h = periodic(kind)(100.millis, 100.millis)(r {
      val k = runs.incrementAndGet()
      log(s"run $k on $th at +${(System.nanoTime() - t0) / 1000000} ms")
      if (k == 2) {
        log("run 2 throws")
        l2.countDown()
        what match {
          case "s" => throw Access.schedEx("task-scheduler-exception")
          case "i" => throw new IllegalStateException("task-ise", Access.schedEx("cause-scheduler-exception"))
          case _   => throw new RuntimeException("boom-2")
        }
      }
    })(ec)
    require(l2.await(5, TimeUnit.SECONDS))
    window(500) // 5x the interval after the throw
    log(s"runs after the window = ${runs.get} (a 3rd run would have started within 100ms of the throw)")
    log(s"Error events on the event stream = ${errCount.get}")
    log(s"handle: isCancelled=${h.isCancelled}")
    log(s"handle: cancel()=${h.cancel()} isCancelled=${h.isCancelled}")
  }

  // SC5: scheduled messages to a terminated actor (live=false) or to an actor stopped mid-repetition (live=true).
  def sc5(kind: Option[String], live: Boolean): Unit = {
    val got = new Semaphore(0); val n = new AtomicInteger
    val v = sys.actorOf(Props(new Victim(got, n)), "victim")
    def stopIt(): Unit = {
      val done = new CountDownLatch(1)
      sys.actorOf(Props(new Watcher(v, done)))
      sys.stop(v)
      require(done.await(5, TimeUnit.SECONDS))
      log(s"victim stopped: isTerminated=${Access.isTerminated(v)}")
    }
    var h: Option[Cancellable] = None
    if (!live) stopIt()
    kind match {
      case None =>
        h = attempt("scheduleOnce(50ms, victim, m1)") { sys.scheduler.scheduleOnce(50.millis, v, "m1")(disp, Actor.noSender) }
      case Some("fd") =>
        h = attempt(s"scheduleWithFixedDelay(Zero, 100ms, victim, p) live=$live") { sys.scheduler.scheduleWithFixedDelay(Duration.Zero, 100.millis, v, "p")(disp, Actor.noSender) }
      case Some(_) =>
        h = attempt(s"scheduleAtFixedRate(Zero, 100ms, victim, p) live=$live") { sys.scheduler.scheduleAtFixedRate(Duration.Zero, 100.millis, v, "p")(disp, Actor.noSender) }
    }
    if (live) {
      require(got.tryAcquire(2, 5, TimeUnit.SECONDS), "victim did not get 2 messages")
      log(s"victim received ${n.get} messages so far; stopping it")
      stopIt()
    } else require(dls.tryAcquire(5, TimeUnit.SECONDS), "no dead letter")
    window(600)
    log(s"summary: victim received=${n.get} deadLetters=${dlCount.get} errorEvents=${errCount.get}")
    h.foreach { c => log(s"handle: isCancelled=${c.isCancelled}"); log(s"handle: cancel()=${c.cancel()} isCancelled=${c.isCancelled}") }
  }
}
}

// SchedulerException and ActorRef.isTerminated are private[pekko]; this object is the harness door to them.
package org.apache.pekko.actor {
  object Access {
    def schedEx(msg: String): Throwable = SchedulerException(msg)
    def isTerminated(r: ActorRef): Boolean = r.isTerminated
  }
}
