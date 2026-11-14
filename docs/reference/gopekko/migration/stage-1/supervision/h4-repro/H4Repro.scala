// Controlled reproductions against the PINNED Apache Pekko Classic runtime (v1.7.1,
// 9a54d1294c3ac7821bbed3c940769c3d5027d1ff). Not part of the judge and not a test of goPekko: it records what the
// original runtime does so that the Go port is defined by observed behavior, not by documentation.
//
// H4: a restart whose children have a RETAINED termination (their postStop blocks on a latch), followed by a Stop of the
//     restarting actor. Hooks, order and number of calls are recorded.
// H6: the restart window and retry count as the strategy really stores them (read by reflection from the real object).
//
// Run (disposable export of the tag, never upstream/pekko): see README.md in this directory.
import java.util.concurrent.{ ConcurrentLinkedQueue, CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.{ AtomicInteger, AtomicLong }

import scala.concurrent.Await
import scala.concurrent.duration._

import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._
import org.apache.pekko.pattern.ask
import org.apache.pekko.util.Timeout

object H4Repro {

  final class Log {
    private val seq = new AtomicLong
    private val q = new ConcurrentLinkedQueue[(Long, String)]
    def add(s: String): Unit = q.add((seq.incrementAndGet(), s))
    def lines: Vector[String] = {
      val it = q.iterator()
      var b = Vector.empty[(Long, String)]
      while (it.hasNext) b = b :+ it.next()
      b.sortBy(_._1).map { case (i, s) => f"$i%03d $s" }
    }
    def count(sub: String): Int = lines.count(_.contains(sub))
    def has(sub: String): Boolean = count(sub) > 0
  }

  def waitFor(cond: => Boolean, ms: Long): Boolean = {
    val end = System.nanoTime + ms * 1000000L
    while (!cond && System.nanoTime < end) Thread.sleep(10)
    cond
  }

  /** A grandchild whose termination is retained: postStop blocks until the driver releases the latch. */
  final class G(name: String, log: Log, hold: CountDownLatch) extends Actor {
    override def preStart(): Unit = log.add(s"G($name).preStart")
    override def postStop(): Unit = {
      log.add(s"G($name).postStop-begin")
      hold.await(20, TimeUnit.SECONDS)
      log.add(s"G($name).postStop-end")
    }
    def receive = { case _ => }
  }

  /** The actor under test. mode: "default-hooks" (Actor.preRestart default), "manual-children-stop" (children stopped by
    * hand, postStop NOT called by preRestart), "plain-stop" (no restart: control of the termination rule). */
  final class C(mode: String, log: Log, hold: CountDownLatch, inst: AtomicInteger) extends Actor {
    val n = inst.incrementAndGet()
    log.add(s"C#$n.constructor")
    override def preStart(): Unit = log.add(s"C#$n.preStart")
    override def preRestart(reason: Throwable, message: Option[Any]): Unit = {
      log.add(s"C#$n.preRestart(message=$message)")
      mode match {
        case "default-hooks"        => super.preRestart(reason, message) // unwatch + stop children, then postStop()
        case "manual-children-stop" => context.children.foreach(context.stop)
        case _                      => ()
      }
      log.add(s"C#$n.preRestart-returned")
    }
    override def postRestart(reason: Throwable): Unit = {
      log.add(s"C#$n.postRestart")
      super.postRestart(reason) // default: preStart()
    }
    override def postStop(): Unit = log.add(s"C#$n.postStop")
    def receive = {
      case "spawn" =>
        context.actorOf(Props(new G("g1", log, hold)), "g1")
        context.actorOf(Props(new G("g2", log, hold)), "g2")
        sender() ! "spawned"
      case "boom" => throw new RuntimeException("boom")
    }
  }

  final case class StopChild(c: ActorRef)

  final class P(mode: String, log: Log, hold: CountDownLatch, inst: AtomicInteger) extends Actor {
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() { case _ => Restart }
    def receive = {
      case "mkC"           => sender() ! context.actorOf(Props(new C(mode, log, hold, inst)), "c")
      case StopChild(c)    => context.stop(c); sender() ! "stop-requested"
    }
  }

  final class W(target: ActorRef, log: Log, done: CountDownLatch) extends Actor {
    context.watch(target)
    def receive = { case Terminated(t) => log.add(s"W.Terminated(${t.path.name})"); done.countDown() }
  }

  def runOnce(mode: String, label: String): Unit = {
    implicit val timeout: Timeout = Timeout(5.seconds)
    val system = ActorSystem("h4")
    val log = new Log
    val hold = new CountDownLatch(1)
    val inst = new AtomicInteger
    val done = new CountDownLatch(1)
    try {
      val p = system.actorOf(Props(new P(mode, log, hold, inst)), "p")
      val c = Await.result((p ? "mkC").mapTo[ActorRef], 5.seconds)
      Await.result(c ? "spawn", 5.seconds)
      system.actorOf(Props(new W(c, log, done)), "w")
      if (mode != "plain-stop") {
        c ! "boom"
        waitFor(log.has("C#1.preRestart-returned"), 5000)
        // the children are stopping and held: the restart is waiting for them (Recreation)
        waitFor(log.has("G(g1).postStop-begin") && log.has("G(g2).postStop-begin"), 5000)
        log.add("DRIVER: restart waiting for retained children; requesting Stop of C")
      } else {
        log.add("DRIVER: requesting Stop of C (no restart)")
      }
      Await.result(p ? StopChild(c), 5.seconds)
      if (mode == "plain-stop") waitFor(log.has("G(g1).postStop-begin") && log.has("G(g2).postStop-begin"), 5000)
      Thread.sleep(1000) // let the Terminate system message be processed while the children are still held
      log.add("DRIVER: releasing the retained children")
      hold.countDown()
      val terminated = done.await(10, TimeUnit.SECONDS)
      Thread.sleep(700) // catch late duplicates
      println(s"--- $label mode=$mode ---")
      log.lines.foreach(println)
      println(
        s"SUMMARY $label mode=$mode: terminatedDelivered=$terminated " +
        s"constructors=${log.count(".constructor")} C.preStart=${log.count("C#1.preStart") + log.count("C#2.preStart")} " +
        s"preRestart=${log.count(".preRestart(")} postRestart=${log.count(".postRestart")} " +
        s"C.postStop=${log.count("C#1.postStop") + log.count("C#2.postStop")} " +
        s"C#1.postStop=${log.count("C#1.postStop")} C#2.postStop=${log.count("C#2.postStop")} " +
        s"W.Terminated=${log.count("W.Terminated")}")
    } finally {
      hold.countDown()
      system.terminate()
      Await.result(system.whenTerminated, 15.seconds)
    }
  }

  /** H6: what the real strategy stores for (maxNrOfRetries, withinTimeRange). */
  def probeWindow(): Unit = {
    println(s"--- H6 retriesWindow as stored by the real OneForOneStrategy (Pekko ${ActorSystem.Version}) ---")
    val field = classOf[OneForOneStrategy].getDeclaredField("retriesWindow")
    field.setAccessible(true)
    def stored(n: Int, w: Duration): String = {
      val s = OneForOneStrategy(maxNrOfRetries = n, withinTimeRange = w)({ case _ => Restart })
      field.get(s).toString
    }
    val MS = 1000000L
    val cases: Seq[(String, Int, Duration)] = Seq(
      ("n=-1 Inf", -1, Duration.Inf),
      ("n=0  Inf", 0, Duration.Inf),
      ("n=3  Inf", 3, Duration.Inf),
      ("n=-1 10s", -1, 10.seconds),
      ("n=3  10s", 3, 10.seconds),
      ("n=3  0", 3, Duration.Zero),
      ("n=3  -1ms", 3, FiniteDuration(-1, MILLISECONDS)),
      ("n=3  1ns", 3, FiniteDuration(1, NANOSECONDS)),
      ("n=3  999999ns", 3, FiniteDuration(999999, NANOSECONDS)),
      ("n=3  1000000ns (1ms)", 3, FiniteDuration(MS, NANOSECONDS)),
      ("n=3  1999999ns", 3, FiniteDuration(1999999, NANOSECONDS)),
      ("n=3  2147483647ms (Int.Max)", 3, FiniteDuration(2147483647L, MILLISECONDS)),
      ("n=3  2147483648ms (2^31)", 3, FiniteDuration(2147483648L, MILLISECONDS)),
      ("n=3  4294967295ms (2^32-1)", 3, FiniteDuration(4294967295L, MILLISECONDS)),
      ("n=3  4294967296ms (2^32)", 3, FiniteDuration(4294967296L, MILLISECONDS)),
      ("n=3  4294967301ms (2^32+5)", 3, FiniteDuration(4294967301L, MILLISECONDS)),
      ("n=3  Long.Max ns", 3, FiniteDuration(Long.MaxValue, NANOSECONDS)))
    cases.foreach { case (label, n, w) => println(f"$label%-34s -> ${stored(n, w)}") }
    println(s"TimeUnit.MILLISECONDS.toNanos(-2147483648) = ${TimeUnit.MILLISECONDS.toNanos(-2147483648)}")
  }

  def main(args: Array[String]): Unit = {
    probeWindow()
    for (mode <- Seq("plain-stop", "default-hooks", "manual-children-stop"); i <- 1 to 3)
      runOnce(mode, s"run$i")
    System.exit(0)
  }
}
