// Pekko-side probe for goPekko #1129 (clock, deadline, cancellation). Original goPekko code, not ported from Pekko.
// It drives the Classic `ask` pattern (AskSupport.scala:709-742) on ExplicitlyTriggeredScheduler
// (testkit/.../ExplicitlyTriggeredScheduler.scala:44-140) and prints one line per observation, "OBS <scenario> <key>=<value>".
// It is not part of any build and not run by CI. Run it in a DISPOSABLE copy of the pin (never upstream/pekko):
//   cp ClockRepro.scala <copy>/judge/src/main/scala/ClockRepro.scala   (judge/run-pekko.sh has already put judge.sbt in <copy>)
//   cd <copy> && sbt -batch "judge/runMain ClockRepro"
import java.util.concurrent.ConcurrentHashMap

import scala.concurrent.Await
import scala.concurrent.duration._
import scala.util.{ Failure, Success }

import com.typesafe.config.ConfigFactory
import org.apache.pekko.actor._
import org.apache.pekko.pattern.{ ask => pekkoAsk }
import org.apache.pekko.testkit.{ CallingThreadDispatcher, ExplicitlyTriggeredScheduler }
import org.apache.pekko.util.Timeout

final class Holder extends Actor {
  var kept: ActorRef = ActorRef.noSender
  def receive: Receive = {
    case "Q"    => kept = sender()
    case "R"    => kept ! "reply"
  }
}

object ClockRepro {
  private def obs(s: String, k: String, v: Any): Unit = println(s"OBS $s $k=$v")

  private def pending(sch: ExplicitlyTriggeredScheduler): Int = {
    // the private val is name-mangled by scalac, so the field is found by suffix
    val f = classOf[ExplicitlyTriggeredScheduler].getDeclaredFields.find(_.getName.endsWith("scheduled")).get
    f.setAccessible(true)
    f.get(sch).asInstanceOf[ConcurrentHashMap[_, _]].size
  }

  private def state(f: scala.concurrent.Future[Any]): String = f.value match {
    case None                  => "pending"
    case Some(Success(v))      => s"success:$v"
    case Some(Failure(e))      => "failure:" + e.getClass.getSimpleName
  }

  final class Driver(body: ActorContext => Unit) extends Actor {
    def receive: Receive = {
      case "warm" => sender() ! "ok"
      case "run"  => body(context)
    }
  }

  // Same pattern as judge/pekko-harness/Judge.scala (Driver, runOne): the scenario body runs inside the driver's handler on the
  // calling thread (CallingThreadDispatcher), after a warm-up round trip, so every send and every reply below is synchronous
  // and each observation is taken after the effects of the previous step. No sleep or polling is added by this file.
  private def scenarios(sys: ActorSystem, ctx: ActorContext): Unit = {
    val sch = sys.scheduler.asInstanceOf[ExplicitlyTriggeredScheduler]
    implicit val t: Timeout = Timeout(1.second)
    def holder(n: String) = ctx.actorOf(Props[Holder]().withDispatcher(CallingThreadDispatcher.Id), n)

    // P1 deadline + fires-on-advance: 1s ask, no reply; 999ms then 1ms.
    val f1 = pekkoAsk(holder("p1"), "Q")
    obs("P1", "armed.pending", pending(sch))
    obs("P1", "t0.state", state(f1))
    sch.timePasses(999.millis)
    obs("P1", "t999ms.state", state(f1))
    obs("P1", "t999ms.pending", pending(sch))
    sch.timePasses(1.milli)
    obs("P1", "t1000ms.state", state(f1))
    obs("P1", "t1000ms.pending", pending(sch))

    // P2 cancellation when the reply wins: the reply arrives before the deadline.
    val b = holder("p2")
    val f2 = pekkoAsk(b, "Q")
    obs("P2", "armed.pending", pending(sch))
    b ! "R"
    obs("P2", "afterReply.state", state(f2))
    obs("P2", "afterReply.pending", pending(sch))
    sch.timePasses(1.hour)
    obs("P2", "afterAdvance1h.state", state(f2))
    obs("P2", "afterAdvance1h.pending", pending(sch))

    // P3 non-positive timeout (informational: NOT a shared property, see the evidence matrix).
    val f3 = pekkoAsk(holder("p3"), "Q")(Timeout(0.millis))
    obs("P3", "state", state(f3))
    obs("P3", "pending", pending(sch))
  }

  def main(args: Array[String]): Unit = {
    val cfg = ConfigFactory
      .parseString("pekko.loglevel = OFF\npekko.stdout-loglevel = OFF\npekko.scheduler.implementation = \"org.apache.pekko.testkit.ExplicitlyTriggeredScheduler\"\n")
      .withFallback(ConfigFactory.load())
    val sys = ActorSystem("clock-repro", cfg)
    try {
      val driver = sys.actorOf(Props(new Driver(ctx => scenarios(sys, ctx))).withDispatcher(CallingThreadDispatcher.Id), "driver")
      Await.result(pekkoAsk(driver, "warm")(Timeout(5.seconds)), 5.seconds)
      driver ! "run"
    } finally {
      Await.ready(sys.terminate(), 10.seconds)
    }
  }
}
