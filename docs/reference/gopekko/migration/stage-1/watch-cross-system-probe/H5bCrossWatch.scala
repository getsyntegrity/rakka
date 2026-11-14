// H5b probe against the PINNED Apache Pekko Classic runtime (v1.7.1): watch, watchWith and unwatch across TWO
// ActorSystems of the same JVM, for the cases H5Probe does not cover (see README.md in this directory).
// Sibling of migration/stage-1/supervision/h4-repro/H5Probe.scala. Not part of the judge.
// STATUS: written, NOT compiled and NOT executed (sbt is denied by migration/settings.go.json in loops).
// The program prints observed events; it asserts nothing. The "expected" lines in README.md are hypotheses read from
// the Pekko source, not results.
import java.util.concurrent.{ ConcurrentLinkedQueue, CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.AtomicReference

import scala.concurrent.Await
import scala.concurrent.duration._

import org.apache.pekko.actor._

object H5bCrossWatch {

  final case class Do(op: String, subject: ActorRef)
  final case class Ping(latch: CountDownLatch)
  final case class Custom(name: String)

  final class Target extends Actor { def receive = { case _ => } }

  /** Records what the watcher saw. The operations run inside the actor, as user code would call them. */
  final class Watcher(log: ConcurrentLinkedQueue[String]) extends Actor {
    def receive = {
      case Do("watch", s)     => context.watch(s)
      case Do("watchWith", s) => context.watchWith(s, Custom(s.path.name))
      case Do("unwatch", s)   => context.unwatch(s)
      case Ping(l)            => l.countDown()
      case t: Terminated =>
        log.add(s"Terminated(${t.actor.path.name}) existenceConfirmed=${t.existenceConfirmed}")
      case Custom(n) => log.add(s"Custom($n)")
    }
  }

  /** Parent that records any failure of the watcher and stops it, so a failure is visible instead of a silent restart. */
  final class Parent(log: ConcurrentLinkedQueue[String], out: AtomicReference[ActorRef], ready: CountDownLatch)
      extends Actor {
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() {
      case t: Throwable =>
        log.add(s"FAILURE ${t.getClass.getName}: ${t.getMessage}")
        SupervisorStrategy.Stop
    }
    out.set(context.actorOf(Props(new Watcher(log)), "watcher"))
    ready.countDown()
    def receive = { case _ => }
  }

  /** Counts down when `subject` terminates; used to know that the subject is really gone before the next step. */
  final class Witness(subject: ActorRef, done: CountDownLatch) extends Actor {
    override def preStart(): Unit = context.watch(subject)
    def receive = { case _: Terminated => done.countDown() }
  }

  def newWatcher(a: ActorSystem, name: String, log: ConcurrentLinkedQueue[String]): ActorRef = {
    val out = new AtomicReference[ActorRef]()
    val ready = new CountDownLatch(1)
    a.actorOf(Props(new Parent(log, out, ready)), s"p-$name")
    ready.await(5, TimeUnit.SECONDS)
    out.get
  }

  def awaitTerminated(sys: ActorSystem, subject: ActorRef, name: String): Boolean = {
    val done = new CountDownLatch(1)
    sys.actorOf(Props(new Witness(subject, done)), name)
    done.await(5, TimeUnit.SECONDS)
  }

  def roundTrip(w: ActorRef): Unit = {
    val l = new CountDownLatch(1)
    w ! Ping(l)
    l.await(3, TimeUnit.SECONDS)
  }

  /** A mailbox round trip, a bounded wait for in-flight notifications, and another round trip. A negative observation
   *  ("nothing arrived") holds only within this window; it is not a proof. */
  def settle(w: ActorRef): Unit = { roundTrip(w); Thread.sleep(300); roundTrip(w) }

  def dump(label: String, log: ConcurrentLinkedQueue[String]): Unit = {
    println(s"--- $label")
    val it = log.iterator()
    if (!it.hasNext) println("  (no events)")
    while (it.hasNext) println("  " + it.next())
  }

  def main(args: Array[String]): Unit = {
    println(s"Pekko ${ActorSystem.Version}")
    val a = ActorSystem("A") // watcher side
    val b = ActorSystem("B") // subject side
    try {
      // (a) live actor of B watched from A, then stopped in B
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "a", log)
        val x = b.actorOf(Props(new Target), "a-x")
        w ! Do("watch", x); roundTrip(w)
        b.stop(x); println(s"  a: subject terminated in B: ${awaitTerminated(b, x, "a-wit")}")
        settle(w); dump("a: watch, live subject then stopped", log)
      }
      // (b) actor of B already terminated before the watch
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "b", log)
        val x = b.actorOf(Props(new Target), "b-x")
        b.stop(x); println(s"  b: subject terminated in B: ${awaitTerminated(b, x, "b-wit")}")
        w ! Do("watch", x); settle(w); dump("b: watch, subject already terminated", log)
      }
      // (c1) watchWith, live subject then stopped
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "c1", log)
        val x = b.actorOf(Props(new Target), "c1-x")
        w ! Do("watchWith", x); roundTrip(w)
        b.stop(x); println(s"  c1: subject terminated in B: ${awaitTerminated(b, x, "c1-wit")}")
        settle(w); dump("c1: watchWith, live subject then stopped", log)
      }
      // (c2) watchWith, subject already terminated
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "c2", log)
        val x = b.actorOf(Props(new Target), "c2-x")
        b.stop(x); println(s"  c2: subject terminated in B: ${awaitTerminated(b, x, "c2-wit")}")
        w ! Do("watchWith", x); settle(w); dump("c2: watchWith, subject already terminated", log)
      }
      // (d1) unwatch after the termination notification was delivered
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "d1", log)
        val x = b.actorOf(Props(new Target), "d1-x")
        w ! Do("watch", x); roundTrip(w)
        b.stop(x); println(s"  d1: subject terminated in B: ${awaitTerminated(b, x, "d1-wit")}")
        settle(w)
        w ! Do("unwatch", x); settle(w); dump("d1: unwatch after the notification was delivered", log)
      }
      // (d2) unwatch of a terminated subject that was never watched
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "d2", log)
        val x = b.actorOf(Props(new Target), "d2-x")
        b.stop(x); println(s"  d2: subject terminated in B: ${awaitTerminated(b, x, "d2-wit")}")
        w ! Do("unwatch", x); settle(w); dump("d2: unwatch of a terminated, never-watched subject", log)
      }
      // (e) the watcher stops first; the subject is stopped afterwards. Only "no delivery, no error" is observable;
      // the release of the link inside B is not visible through the public API.
      locally {
        val log = new ConcurrentLinkedQueue[String]()
        val w = newWatcher(a, "e", log)
        val x = b.actorOf(Props(new Target), "e-x")
        w ! Do("watch", x); roundTrip(w)
        w ! PoisonPill; println(s"  e: watcher terminated in A: ${awaitTerminated(a, w, "e-wit-w")}")
        b.stop(x); println(s"  e: subject terminated in B: ${awaitTerminated(b, x, "e-wit-x")}")
        Thread.sleep(300); dump("e: watcher stopped first, then the subject", log)
      }
    } finally {
      a.terminate(); b.terminate()
      Await.result(a.whenTerminated, 10.seconds); Await.result(b.whenTerminated, 10.seconds)
    }
    System.exit(0)
  }
}
