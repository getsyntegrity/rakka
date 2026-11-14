// Upstream observation for goPekko #770 (MSG-06): PoisonPill order, stop, and the undelivered case. Disposable Pekko v1.7.1 copy.
import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.Await
import scala.concurrent.duration._
import org.apache.pekko.actor._
import org.apache.pekko.pattern.ask
import org.apache.pekko.util.Timeout

object PoisonProbe {
  def obs(what: String, v: Any): Unit = println(s"OBS $what $v")
  val gate, gate2 = new CountDownLatch(1)
  val started, started2 = new CountDownLatch(1)
  val postStops, preRestarts, terminated = new AtomicInteger(0)
  class Child(started: CountDownLatch, gate: CountDownLatch) extends Actor {
    def receive = {
      case 1 => started.countDown(); gate.await(); obs("child-receive", 1)
      case m => obs("child-receive", m)
    }
    override def preRestart(r: Throwable, m: Option[Any]): Unit = { preRestarts.incrementAndGet(); super.preRestart(r, m) }
    override def postStop(): Unit = { postStops.incrementAndGet() }
  }
  class Parent(started: CountDownLatch, gate: CountDownLatch) extends Actor {
    private val child = context.actorOf(Props(new Child(started, gate)), "kid")
    def receive = { case "child" => sender() ! child; case "stop" => context.stop(child) }
  }
  class Watcher(target: ActorRef) extends Actor {
    context.watch(target)
    def receive = { case Terminated(_) => terminated.incrementAndGet(); case "sync" => sender() ! "sync" }
  }
  class Listener extends Actor {
    def receive = { case d: DeadLetter => obs("DeadLetter", d.message); case s: SuppressedDeadLetter => obs("SuppressedDeadLetter", s.message) }
  }
  def main(args: Array[String]): Unit = {
    implicit val timeout: Timeout = Timeout(10.seconds)
    val system = ActorSystem("poison")
    val listener = system.actorOf(Props(new Listener))
    system.eventStream.subscribe(listener, classOf[DeadLetter]); system.eventStream.subscribe(listener, classOf[SuppressedDeadLetter])
    val parent = system.actorOf(Props(new Parent(started, gate)))
    val child = Await.result((parent ? "child").mapTo[ActorRef], 10.seconds)
    val watcher = system.actorOf(Props(new Watcher(child)))
    Await.result(watcher ? "sync", 10.seconds)
    child ! 1; started.await(); child ! 2; child ! PoisonPill; child ! 3
    gate.countDown()
    while (terminated.get == 0) Thread.sleep(10)
    Await.result(watcher ? "sync", 10.seconds)
    child ! PoisonPill // to the stopped actor
    Await.result(watcher ? "sync", 10.seconds)
    system.terminate(); Await.result(system.whenTerminated, 20.seconds)
    // Scenario 5c: Kill and PoisonPill queued between user messages when a direct stop is accepted; the order of the dead letters.
    val sys2 = ActorSystem("poison2")
    val l2 = sys2.actorOf(Props(new Listener))
    sys2.eventStream.subscribe(l2, classOf[DeadLetter]); sys2.eventStream.subscribe(l2, classOf[SuppressedDeadLetter])
    val p2 = sys2.actorOf(Props(new Parent(started2, gate2)))
    val c2 = Await.result((p2 ? "child").mapTo[ActorRef], 10.seconds)
    val w2 = sys2.actorOf(Props(new Watcher(c2)))
    Await.result(w2 ? "sync", 10.seconds)
    val before = terminated.get
    c2 ! 1; started2.await(); c2 ! 2; c2 ! Kill; c2 ! 3; c2 ! PoisonPill; c2 ! 4
    p2 ! "stop"; Thread.sleep(200) // the stop is a system message: it is accepted behind the held Receive(1) once the parent has sent it
    gate2.countDown()
    while (terminated.get == before) Thread.sleep(10)
    Await.result(w2 ? "sync", 10.seconds)
    sys2.terminate(); Await.result(sys2.whenTerminated, 20.seconds)
    obs("final", s"postStops=${postStops.get} preRestarts=${preRestarts.get} terminated=${terminated.get}")
  }
}
