// Differential probe for goPekko #814 (MSG-02): what sender() reads in the parent's decider,
// in preRestart/postStop after an escalation, and in Terminated / WatchWith deliveries.
// Run in a disposable copy of Pekko v1.7.1 (see RUN.md), never in upstream/pekko.
// Output lines are "OBS <scenario> <where> sender=<path|deadLetters>".
import java.util.concurrent.{ CountDownLatch, TimeUnit }
import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._

object Probe {
  case object Boom
  case object Go
  case object Kill1
  final case class Wm(tag: String)

  def obs(sc: String, where: String, s: ActorRef): Unit =
    println(s"OBS $sc $where sender=${s.path.name}")

  // Failing child: throws on Boom (sent by the named probe actor "x-sender").
  class Child extends Actor {
    def receive = { case Boom => throw new RuntimeException("boom") }
  }

  // Parent: decider prints sender() and escalates; preRestart/postStop print sender().
  class Parent(sc: String, latch: CountDownLatch, expose: Promise[ActorRef]) extends Actor {
    val child = context.actorOf(Props(new Child), "child-c")
    expose.success(child)
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() {
      case _ => obs(sc, "decider", sender()); Escalate
    }
    def receive = { case _ => }
    override def preRestart(r: Throwable, m: Option[Any]): Unit = {
      obs(sc, "preRestart", sender()); println(s"OBS $sc preRestart-message=$m"); latch.countDown()
    }
    override def postStop(): Unit = { obs(sc, "postStop", sender()); latch.countDown() }
  }

  class Grand(sc: String, dec: Directive, latch: CountDownLatch, expose: Promise[ActorRef]) extends Actor {
    val parent = context.actorOf(Props(new Parent(sc, latch, expose)), "parent-p")
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() { case _ => dec }
    def receive = { case _ => }
  }

  class XSender(target: ActorRef) extends Actor {
    def receive = { case Go => target.tell(Boom, self) }
  }

  import scala.concurrent.{ Await, Promise => SPromise }
  import scala.concurrent.duration._
  type Promise[A] = SPromise[A]

  def failureScenario(sys: ActorSystem, sc: String, dec: Directive): Unit = {
    val latch = new CountDownLatch(if (dec == Restart) 1 else 1)
    val p = SPromise[ActorRef]()
    sys.actorOf(Props(new Grand(sc, dec, latch, p)), s"grand-$sc")
    val child = Await.result(p.future, 10.seconds)
    val x = sys.actorOf(Props(new XSender(child)), "x-sender-" + sc)
    x ! Go
    if (!latch.await(10, TimeUnit.SECONDS)) println(s"OBS $sc TIMEOUT")
  }

  // Watcher scenarios: who observes Terminated / WatchWith / death pact.
  class Victim extends Actor { def receive = { case Kill1 => context.stop(self) } }

  class WatcherTerm(latch: CountDownLatch, v: ActorRef) extends Actor {
    context.watch(v)
    def receive = { case _: Terminated => obs("T1", "Terminated", sender()); latch.countDown() }
  }
  class WatcherWith(latch: CountDownLatch, v: ActorRef) extends Actor {
    context.watchWith(v, Wm("wm"))
    def receive = { case _: Wm => obs("T2", "WatchWith", sender()); latch.countDown() }
  }
  class WatcherPact(latch: CountDownLatch, v: ActorRef) extends Actor {
    context.watch(v)
    def receive = { case "never" => }
    override def postStop(): Unit = { obs("T3", "postStop-after-deathpact", sender()); latch.countDown() }
  }

  def watchScenario(sys: ActorSystem, f: (CountDownLatch, ActorRef) => Actor, name: String): Unit = {
    val latch = new CountDownLatch(1)
    val v = sys.actorOf(Props(new Victim), "victim-" + name)
    sys.actorOf(Props(f(latch, v)), "watcher-" + name)
    v ! Kill1
    if (!latch.await(10, TimeUnit.SECONDS)) println(s"OBS $name TIMEOUT")
  }

  def main(args: Array[String]): Unit = {
    val sys = ActorSystem("probe")
    try {
      failureScenario(sys, "G4", Restart) // escalate, grandparent Restart: decider + preRestart
      failureScenario(sys, "G6", Stop) // escalate, grandparent Stop: postStop
      watchScenario(sys, (l, v) => new WatcherTerm(l, v), "T1")
      watchScenario(sys, (l, v) => new WatcherWith(l, v), "T2")
      watchScenario(sys, (l, v) => new WatcherPact(l, v), "T3")
    } finally {
      sys.terminate()
      Await.result(sys.whenTerminated, 10.seconds)
    }
  }
}
