// Differential probe for goPekko #1031 (MSG-09): Kill under the DEFAULT decider (Stop) and under a custom
// Restart decider. Run in a disposable copy of Pekko v1.7.1 (see RUN.md), never in upstream/pekko.
// Output lines are "OBS <scenario> <what> <value>".
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.Await
import scala.concurrent.duration._
import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._
import org.apache.pekko.pattern.ask
import org.apache.pekko.util.Timeout

object KillProbe {
  final class Counts {
    val made, postStops, preRestarts, postRestarts, terminated, receives = new AtomicInteger(0)
  }

  def obs(sc: String, what: String, v: Any): Unit = println(s"OBS $sc $what $v")

  class Child(sc: String, c: Counts) extends Actor {
    c.made.incrementAndGet()
    def receive = {
      case m =>
        c.receives.incrementAndGet(); obs(sc, "child-receive", m); sender() ! m
    }
    override def preRestart(r: Throwable, m: Option[Any]): Unit = {
      c.preRestarts.incrementAndGet(); obs(sc, "preRestart", s"cause=${r.getClass.getSimpleName}(${r.getMessage}) message=$m")
      super.preRestart(r, m)
    }
    override def postRestart(r: Throwable): Unit = { c.postRestarts.incrementAndGet(); super.postRestart(r) }
    override def postStop(): Unit = { c.postStops.incrementAndGet() }
  }

  // Parent without a strategy: the default decider decides.
  class DefaultParent(sc: String, c: Counts) extends Actor {
    private val child = context.actorOf(Props(new Child(sc, c)), "kid")
    def receive = { case "child" => sender() ! child }
  }

  // Parent with an explicit decider that restarts every cause.
  class RestartParent(sc: String, c: Counts) extends Actor {
    private val child = context.actorOf(Props(new Child(sc, c)), "kid")
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() {
      case e =>
        obs(sc, "decider-cause", s"${e.getClass.getSimpleName}(${e.getMessage}) isActorKilled=${e.isInstanceOf[ActorKilledException]}")
        Restart
    }
    def receive = { case "child" => sender() ! child }
  }

  class Watcher(sc: String, c: Counts, target: ActorRef) extends Actor {
    context.watch(target)
    def receive = {
      case Terminated(_) => c.terminated.incrementAndGet(); obs(sc, "terminated", "received")
      case "sync"        => sender() ! "sync"
    }
  }

  def main(args: Array[String]): Unit = {
    implicit val timeout: Timeout = Timeout(10.seconds)
    for ((sc, restart) <- Seq("A-default" -> false, "B-restart" -> true)) {
      val system = ActorSystem("probe-" + sc.take(1))
      val c = new Counts
      val parent =
        if (restart) system.actorOf(Props(new RestartParent(sc, c))) else system.actorOf(Props(new DefaultParent(sc, c)))
      val child = Await.result((parent ? "child").mapTo[ActorRef], 10.seconds)
      val watcher = system.actorOf(Props(new Watcher(sc, c, child)))
      Await.result(watcher ? "sync", 10.seconds)
      child ! Kill
      if (restart) obs(sc, "ping-reply-after-restart", Await.result(child ? "ping", 10.seconds))
      else {
        // the child stops: wait for its Terminated through the watcher, then settle with a sync
        while (c.terminated.get == 0) Thread.sleep(10)
      }
      Await.result(watcher ? "sync", 10.seconds)
      system.terminate(); Await.result(system.whenTerminated, 20.seconds)
      obs(sc, "final", s"made=${c.made.get} postStops=${c.postStops.get} preRestarts=${c.preRestarts.get} " +
        s"postRestarts=${c.postRestarts.get} terminated=${c.terminated.get} childReceives=${c.receives.get}")
    }
  }
}
