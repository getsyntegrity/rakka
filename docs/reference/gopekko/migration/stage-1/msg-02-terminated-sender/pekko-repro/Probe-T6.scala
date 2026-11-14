// Differential probe for goPekko #814 Part 2 (D5b), scenario T6: what sender() and the message read in preRestart
// (and postRestart) when the failure is a DeathPactException (Terminated received by an actor whose receive does
// not handle it) and the supervisor decides Restart.
// Run in a disposable copy of Pekko v1.7.1 (see RUN-T6.md), never in upstream/pekko.
// Output lines are "OBS T6 <where> ...".
import java.util.concurrent.{ CountDownLatch, TimeUnit }
import scala.concurrent.{ Await, Promise => SPromise }
import scala.concurrent.duration._
import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._

object ProbeT6 {
  case object Kill1
  case object Other

  class Victim extends Actor {
    def receive = { case Kill1 => context.stop(self) }
  }

  // Watches v, does NOT handle Terminated (only Other), so the default `unhandled` throws DeathPactException.
  class Pact(v: ActorRef, restarted: CountDownLatch) extends Actor {
    override def preStart(): Unit = context.watch(v)
    def receive = {
      case Other =>
    }
    override def preRestart(r: Throwable, m: Option[Any]): Unit = {
      println(s"OBS T6 preRestart sender=${sender().path.name}")
      println(s"OBS T6 preRestart cause=${r.getClass.getSimpleName} dead=${r match {
          case d: DeathPactException => d.dead.path.name
          case _                     => "n/a"
        }}")
      println(s"OBS T6 preRestart message-is-Terminated=${m.exists(_.isInstanceOf[Terminated])}")
    }
    override def postRestart(r: Throwable): Unit = {
      println(s"OBS T6 postRestart sender=${sender().path.name}")
      restarted.countDown()
    }
  }

  class Sup(mk: () => Actor, expose: SPromise[ActorRef]) extends Actor {
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() { case _ => Restart }
    expose.success(context.actorOf(Props(mk()), "watcher"))
    def receive = { case _ => }
  }

  def main(args: Array[String]): Unit = {
    val sys = ActorSystem("probe")
    try {
      val v = sys.actorOf(Props(new Victim), "victim-T6")
      val restarted = new CountDownLatch(1)
      val p = SPromise[ActorRef]()
      sys.actorOf(Props(new Sup(() => new Pact(v, restarted), p)), "sup-T6")
      Await.result(p.future, 10.seconds)
      v ! Kill1
      if (!restarted.await(10, TimeUnit.SECONDS)) println("OBS T6 TIMEOUT")
    } finally {
      sys.terminate()
      Await.result(sys.whenTerminated, 10.seconds)
    }
  }
}
