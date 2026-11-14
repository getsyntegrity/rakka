// Extended differential probe for goPekko #814 Part 2 (D5b): what sender() reads around Terminated deliveries.
// Scenarios: T3 no leak of a sender retained by a resumed failure, T4 success clears, T5 failure keeps until restart,
// T7 next message replaces, T9 incarnation identity (stop and respawn under the same name).
// Run in a disposable copy of Pekko v1.7.1 (see RUN-T.md), never in upstream/pekko.
// Output lines are "OBS <scenario> <where> sender=<name>[ ...]".
import java.util.concurrent.{ CountDownLatch, TimeUnit }
import scala.concurrent.{ Await, Promise => SPromise }
import scala.concurrent.duration._
import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._

object ProbeT {
  case object Boom
  case object Kill1
  final case class Msg(tag: String)

  def obs(sc: String, where: String, s: ActorRef): Unit =
    println(s"OBS $sc $where sender=${s.path.name}")

  class Victim extends Actor {
    def receive = { case Kill1 => context.stop(self) }
  }

  // Runs `mk` as its only child under the given strategy and exposes the child.
  class Sup(strat: SupervisorStrategy, mk: () => Actor, expose: SPromise[ActorRef]) extends Actor {
    override val supervisorStrategy: SupervisorStrategy = strat
    expose.success(context.actorOf(Props(mk()), "watcher"))
    def receive = { case _ => }
  }

  class XSender(target: ActorRef) extends Actor {
    def receive = { case Boom => target.tell(Boom, self) }
  }

  // T3: Receive of Boom (told by x-sender-T3) fails and is Resumed, retaining that sender; then Terminated arrives.
  class WatcherLeak(v: ActorRef, threw: CountDownLatch, done: CountDownLatch) extends Actor {
    override def preStart(): Unit = context.watch(v)
    def receive = {
      case Boom => threw.countDown(); throw new RuntimeException("boom")
      case _: Terminated => obs("T3", "Terminated-after-resumed-failure", sender()); done.countDown()
    }
  }

  // T4: Terminated is handled successfully; then the actor is stopped from outside and postStop reads sender().
  class WatcherOk(v: ActorRef, seen: CountDownLatch, stopped: CountDownLatch) extends Actor {
    override def preStart(): Unit = context.watch(v)
    def receive = {
      case _: Terminated => obs("T4", "Terminated", sender()); seen.countDown()
    }
    override def postStop(): Unit = { obs("T4", "postStop-after-success", sender()); stopped.countDown() }
  }

  // T5: Terminated handling fails; the parent Restarts; preRestart and postRestart read sender().
  class WatcherFail(v: ActorRef, restarted: CountDownLatch) extends Actor {
    override def preStart(): Unit = context.watch(v)
    def receive = {
      case _: Terminated => obs("T5", "Terminated", sender()); throw new RuntimeException("boom")
    }
    override def preRestart(r: Throwable, m: Option[Any]): Unit = {
      obs("T5", "preRestart", sender())
      println(s"OBS T5 preRestart-message-is-Terminated=${m.exists(_.isInstanceOf[Terminated])}")
    }
    override def postRestart(r: Throwable): Unit = { obs("T5", "postRestart", sender()); restarted.countDown() }
  }

  // T7: after a Terminated whose handling failed and was Resumed, a plain tell and a tell from z read their own sender.
  class WatcherNext(v: ActorRef, term: CountDownLatch, done: CountDownLatch) extends Actor {
    override def preStart(): Unit = context.watch(v)
    def receive = {
      case _: Terminated => obs("T7", "Terminated", sender()); term.countDown(); throw new RuntimeException("boom")
      case Msg(tag) => obs("T7", "next-" + tag, sender()); if (tag == "from-z") done.countDown()
    }
  }

  // T9: parent spawns child "inc", watches it, stops it; on Terminated it respawns "inc" (a new incarnation) and
  // compares the delivered sender with the old and the new reference.
  class Incarnation(done: CountDownLatch) extends Actor {
    var old: ActorRef = _
    override def preStart(): Unit = {
      old = context.actorOf(Props(new Victim), "inc")
      context.watch(old)
      context.stop(old)
    }
    def receive = {
      case t: Terminated =>
        val s = sender()
        val fresh = context.actorOf(Props(new Victim), "inc")
        println(s"OBS T9 sender=${s.path.name} oldHash=${old.hashCode} newHash=${fresh.hashCode} senderHash=${s.hashCode} (ActorRef.hashCode is derived from path.uid, which is private[pekko])")
        println(s"OBS T9 sender==old=${s == old} sender==new=${s == fresh} terminated.actor==sender=${t.actor == s}")
        done.countDown()
    }
  }

  def spawnUnder(sys: ActorSystem, strat: SupervisorStrategy, name: String, mk: () => Actor): ActorRef = {
    val p = SPromise[ActorRef]()
    sys.actorOf(Props(new Sup(strat, mk, p)), "sup-" + name)
    Await.result(p.future, 10.seconds)
  }

  def await(l: CountDownLatch, sc: String): Unit =
    if (!l.await(10, TimeUnit.SECONDS)) println(s"OBS $sc TIMEOUT")

  val resume: SupervisorStrategy = OneForOneStrategy() { case _ => Resume }
  val restart: SupervisorStrategy = OneForOneStrategy() { case _ => Restart }

  def main(args: Array[String]): Unit = {
    val sys = ActorSystem("probe")
    try {
      // T3
      val v3 = sys.actorOf(Props(new Victim), "victim-T3")
      val threw3, done3 = new CountDownLatch(1)
      val w3 = spawnUnder(sys, resume, "T3", () => new WatcherLeak(v3, threw3, done3))
      val x3 = sys.actorOf(Props(new XSender(w3)), "x-sender-T3")
      x3 ! Boom
      await(threw3, "T3")
      v3 ! Kill1
      await(done3, "T3")
      // T4
      val v4 = sys.actorOf(Props(new Victim), "victim-T4")
      val seen4, stopped4 = new CountDownLatch(1)
      val w4 = sys.actorOf(Props(new WatcherOk(v4, seen4, stopped4)), "watcher-T4")
      v4 ! Kill1
      await(seen4, "T4")
      sys.stop(w4)
      await(stopped4, "T4")
      // T5
      val v5 = sys.actorOf(Props(new Victim), "victim-T5")
      val restarted5 = new CountDownLatch(1)
      spawnUnder(sys, restart, "T5", () => new WatcherFail(v5, restarted5))
      v5 ! Kill1
      await(restarted5, "T5")
      // T7
      val v7 = sys.actorOf(Props(new Victim), "victim-T7")
      val term7, done7 = new CountDownLatch(1)
      val w7 = spawnUnder(sys, resume, "T7", () => new WatcherNext(v7, term7, done7))
      val z7 = sys.actorOf(Props(new Victim), "z-T7")
      v7 ! Kill1
      await(term7, "T7")
      w7 ! Msg("tell")
      w7.tell(Msg("from-z"), z7)
      await(done7, "T7")
      // T9
      val done9 = new CountDownLatch(1)
      sys.actorOf(Props(new Incarnation(done9)), "incarnation-T9")
      await(done9, "T9")
    } finally {
      sys.terminate()
      Await.result(sys.whenTerminated, 10.seconds)
    }
  }
}
