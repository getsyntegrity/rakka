// A2 and A4 probes against the PINNED Apache Pekko Classic runtime (v1.7.1).
//  A2: what upstream does when a decider returns an invalid directive (the only out-of-domain value Scala can produce is null:
//      Directive is a sealed trait, FaultHandling.scala:118-161).
//  A4: the real ChildRestartStats.requestRestartPermission for a matrix of (maxNrOfRetries, withinTimeRange) cells, five calls in a
//      row each, so the matrix of the ADR is checked against the real class and not only against a transcription.
// Not part of the judge and not a test of goPekko.
import java.util.concurrent.{ CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.AtomicReference

import scala.concurrent.Await
import scala.concurrent.duration._

import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._

object A2A4Probe {

  // ---- A2 -------------------------------------------------------------------------------------------------------------
  final class Crasher extends Actor { def receive = { case _ => throw new RuntimeException("child-boom") } }

  final class Parent(seen: AtomicReference[String], restarted: CountDownLatch) extends Actor {
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() { case _ => null.asInstanceOf[Directive] }
    private val child = context.actorOf(Props(new Crasher), "crasher")
    override def preRestart(reason: Throwable, message: Option[Any]): Unit = {
      seen.set(s"parent.preRestart(reason=${reason.getClass.getName}: ${reason.getMessage}, message=${message.map(_.getClass.getName)})")
      restarted.countDown()
      super.preRestart(reason, message)
    }
    def receive = { case "go" => child ! "x" }
  }

  def probeInvalidDirective(): Unit = {
    println("--- A2: a decider that returns null (the only invalid Directive Scala can produce) ---")
    val system = ActorSystem("a2")
    val seen = new AtomicReference[String]("parent was not restarted within 3 s")
    val restarted = new CountDownLatch(1)
    try {
      val p = system.actorOf(Props(new Parent(seen, restarted)), "parent")
      p ! "go"
      restarted.await(3, TimeUnit.SECONDS)
      println(s"  ${seen.get}")
    } finally {
      system.terminate()
      Await.result(system.whenTerminated, 10.seconds)
    }
  }

  // ---- A4 -------------------------------------------------------------------------------------------------------------
  def decisions(system: ActorSystem, retries: Option[Int], window: Option[Int], calls: Int): String = {
    val stats = ChildRestartStats(system.deadLetters)
    (1 to calls).map(_ => if (stats.requestRestartPermission((retries, window))) "allow" else "deny").mkString(",")
  }

  def probeMatrix(): Unit = {
    println("--- A4: real ChildRestartStats.requestRestartPermission, 5 calls in a row per cell ---")
    val system = ActorSystem("a4")
    try {
      def stored(n: Int, w: Duration): (Option[Int], Option[Int]) = {
        val s = OneForOneStrategy(maxNrOfRetries = n, withinTimeRange = w)({ case _ => Restart })
        val f = classOf[OneForOneStrategy].getDeclaredField("retriesWindow"); f.setAccessible(true)
        f.get(s).asInstanceOf[(Option[Int], Option[Int])]
      }
      val windows: Seq[(String, Duration)] = Seq(
        ("Inf (no window)", Duration.Inf),
        ("-1ms (negative: no window)", FiniteDuration(-1, MILLISECONDS)),
        ("0 (zero-length window)", Duration.Zero),
        ("10s (finite)", 10.seconds),
        ("2^31 ms (wraps to a negative Int)", FiniteDuration(2147483648L, MILLISECONDS)),
        ("2^32 ms (wraps to 0)", FiniteDuration(4294967296L, MILLISECONDS)))
      val retries = Seq(-1, 0, 1, 3)
      for (r <- retries; (wl, w) <- windows) {
        val (ro, wo) = stored(r, w)
        println("retries=" + r.toString.reverse.padTo(2, ' ').reverse + " window=" + wl.padTo(36, ' ') +
          " stored=" + (ro, wo).toString.padTo(24, ' ') + " -> " + decisions(system, ro, wo, 5))
      }
    } finally {
      system.terminate()
      Await.result(system.whenTerminated, 10.seconds)
    }
  }

  def main(args: Array[String]): Unit = {
    println(s"Pekko ${ActorSystem.Version}")
    probeInvalidDirective()
    probeMatrix()
    System.exit(0)
  }
}
