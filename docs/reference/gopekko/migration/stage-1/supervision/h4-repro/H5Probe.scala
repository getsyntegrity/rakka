// H5 probe against the PINNED Apache Pekko Classic runtime (v1.7.1): what happens when an actor watches, watchWith-es or
// unwatches (1) a null reference and (2) a reference that belongs to ANOTHER ActorSystem of the same JVM. The result
// decides whether the approved Go contract (ADR 0010, watch increment: a nil handle or a handle of another System is
// rejected with ErrInvalidHandle) has an upstream equivalent. Not part of the judge.
import java.util.concurrent.{ CountDownLatch, TimeUnit }
import java.util.concurrent.atomic.AtomicReference

import scala.concurrent.Await
import scala.concurrent.duration._

import org.apache.pekko.actor._
import org.apache.pekko.actor.SupervisorStrategy._

object H5Probe {

  final class Target extends Actor { def receive = { case _ => } }

  /** Watches `subject` from preStart and reports what it saw; Terminated ends the probe. */
  final class Watcher(op: String, subject: ActorRef, terminated: CountDownLatch) extends Actor {
    override def preStart(): Unit = op match {
      case "watch"     => context.watch(subject)
      case "watchWith" => context.watchWith(subject, "custom-termination")
      case "unwatch"   => context.unwatch(subject)
    }
    def receive = {
      case t: Terminated         => println(s"  Terminated delivered for ${t.actor.path} existenceConfirmed=${t.existenceConfirmed}"); terminated.countDown()
      case "custom-termination" => println("  custom WatchWith message delivered"); terminated.countDown()
    }
  }

  /** A parent whose decider records the class and message of the failure and the child, then stops the child. */
  final class Parent(op: String, subject: ActorRef, seen: AtomicReference[String], terminated: CountDownLatch) extends Actor {
    override val supervisorStrategy: SupervisorStrategy = OneForOneStrategy() {
      case t: Throwable =>
        seen.set(s"${t.getClass.getName}: ${t.getMessage} (is Exception=${t.isInstanceOf[Exception]}) default decider would be: " +
          SupervisorStrategy.defaultDecider.applyOrElse(t, (_: Throwable) => Escalate))
        Stop
    }
    context.actorOf(Props(new Watcher(op, subject, terminated)), "watcher")
    def receive = { case _ => }
  }

  def run(label: String, op: String, systemA: ActorSystem, subject: ActorRef, stopSubject: () => Unit): Unit = {
    println(s"--- $label ($op) ---")
    val seen = new AtomicReference[String]("no failure reported to the parent")
    val terminated = new CountDownLatch(1)
    systemA.actorOf(Props(new Parent(op, subject, seen, terminated)), s"p-$label-$op")
    Thread.sleep(500)
    println(s"  parent saw: ${seen.get}")
    if (seen.get.startsWith("no failure")) {
      stopSubject()
      val got = terminated.await(3, TimeUnit.SECONDS)
      println(s"  termination observed after stopping the subject: $got")
    }
  }

  def main(args: Array[String]): Unit = {
    println(s"Pekko ${ActorSystem.Version}")
    val a = ActorSystem("A")
    val b = ActorSystem("B")
    try {
      for (op <- Seq("watch", "watchWith", "unwatch")) run("null", op, a, null, () => ())
      for (op <- Seq("watch", "watchWith")) {
        val x = b.actorOf(Props(new Target), s"x-$op")
        run("other-system", op, a, x, () => b.stop(x))
      }
      // unwatch of a live ref of another system, never watched: allowed?
      val y = b.actorOf(Props(new Target), "y")
      run("other-system", "unwatch", a, y, () => ())
    } finally {
      a.terminate(); b.terminate()
      Await.result(a.whenTerminated, 10.seconds); Await.result(b.whenTerminated, 10.seconds)
    }
    System.exit(0)
  }
}
