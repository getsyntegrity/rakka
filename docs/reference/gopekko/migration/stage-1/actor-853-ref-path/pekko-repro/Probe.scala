// Extra Classic scenario for goPekko #853 (Owner condition D-6): the path an ActorRef reports.
// This file is a probe for a DISPOSABLE COPY of Pekko v1.7.1 (see RUN.md). It is NOT part of
// upstream, it is NOT built in this repository, and it never runs inside upstream/pekko.
// Output lines are "ROW|<label>|<value>"; observed.txt holds them without the "ROW|" prefix.
// Element sequences are printed as n=<size> ["a","b"] because toString cannot tell List("") from Nil.
package org.apache.pekko.actor.probe853
import org.apache.pekko.actor._
import org.apache.pekko.pattern.gracefulStop
import java.util.concurrent.{ CountDownLatch, TimeUnit }
import scala.concurrent.Await
import scala.concurrent.duration._

class Leaf extends Actor { def receive = { case _ => } }

// Parent that owns a named child "b" and hands it out.
class Parent extends Actor {
  val kid: ActorRef = context.actorOf(Props[Leaf](), "b")
  val anon: ActorRef = context.actorOf(Props[Leaf]())
  def receive = {
    case "kids" => sender() ! ((kid, anon))
  }
}

object Counters {
  val postRestart = new CountDownLatch(1)
  @volatile var failerRef: ActorRef = _
}

// Fails on "boom"; the default supervisor strategy restarts it on an Exception.
class Failer extends Actor {
  def receive = {
    case "boom" => throw new IllegalStateException("boom")
    case "self" => sender() ! self
  }
  override def postRestart(reason: Throwable): Unit = Counters.postRestart.countDown()
}

class Boss extends Actor {
  val child: ActorRef = context.actorOf(Props[Failer](), "f")
  def receive = {
    case "child" => sender() ! child
    case "boom"  => child ! "boom"
  }
}

object Probe {
  def el(xs: Iterable[String]): String = "n=" + xs.size + " [" + xs.map("\"" + _ + "\"").mkString(",") + "]"
  def row(label: String, v: Any): Unit = println(s"ROW|$label|$v")
  def p(label: String, path: ActorPath): Unit = {
    row(label + " toString", path.toString)
    row(label + " elements", el(path.elements))
  }
  def main(args: Array[String]): Unit = {
    import org.apache.pekko.pattern.ask
    import org.apache.pekko.util.Timeout
    implicit val timeout: Timeout = Timeout(5.seconds)
    val sys = ActorSystem("sys")
    try {
      val prov = sys.asInstanceOf[ExtendedActorSystem].provider

      // N2 / P5: top-level actor
      val a = sys.actorOf(Props[Leaf](), "a")
      p("top-level a", a.path)
      row("top-level a path.parent", a.path.parent)

      // N2: nested actor a/b under a named parent; N4: anonymous child
      val par = sys.actorOf(Props[Parent](), "par")
      val (kid, anon) = Await.result((par ? "kids").mapTo[(ActorRef, ActorRef)], 5.seconds)
      p("nested par/b", kid.path)
      row("nested par/b parent == par.path", kid.path.parent == par.path)
      row("nested anonymous name starts with $", anon.path.name.startsWith("$"))
      row("nested anonymous elements size", anon.path.elements.size)
      row("nested anonymous parent == par.path", anon.path.parent == par.path)

      // P9: anonymous top-level actor
      val anonTop = sys.actorOf(Props[Leaf]())
      row("anonymous top-level name starts with $", anonTop.path.name.startsWith("$"))
      row("anonymous top-level elements size", anonTop.path.elements.size)
      row("anonymous top-level parent", anonTop.path.parent)

      // N3: restart keeps the path (and the ActorRef)
      val boss = sys.actorOf(Props[Boss](), "boss")
      val f = Await.result((boss ? "child").mapTo[ActorRef], 5.seconds)
      val before = f.path
      val uidBefore = f.path.uid
      boss ! "boom"
      row("restart postRestart reached", Counters.postRestart.await(5, TimeUnit.SECONDS))
      val f2 = Await.result((boss ? "child").mapTo[ActorRef], 5.seconds)
      row("restart path before", before)
      row("restart path after", f2.path)
      row("restart path equal", before == f2.path)
      row("restart ActorRef same instance", f eq f2)
      row("restart uid unchanged", uidBefore == f2.path.uid)

      // P7: a stopped actor still reports its path
      val s1 = sys.actorOf(Props[Leaf](), "s")
      val s1Path = s1.path
      val s1Uid = s1.path.uid
      row("stopped before stop isTerminated", s1.isTerminated)
      row("stopped gracefulStop", Await.result(gracefulStop(s1, 5.seconds), 6.seconds))
      row("stopped isTerminated", s1.isTerminated)
      row("stopped path", s1.path)
      row("stopped path equal to the path before", s1.path == s1Path)
      row("stopped uid unchanged", s1.path.uid == s1Uid)

      // N1: two incarnations at one path report equal paths and are not equal refs
      val s2 = sys.actorOf(Props[Leaf](), "s")
      row("incarnations path equal", s1.path == s2.path)
      row("incarnations uid differ", s1.path.uid != s2.path.uid)
      row("incarnations ref equal", s1 == s2)

      // P8: guardians
      p("rootGuardian", prov.rootGuardian.path)
      row("rootGuardian isRoot", prov.rootGuardian.path.isInstanceOf[RootActorPath])
      p("guardian (/user)", prov.guardian.path)
      p("systemGuardian (/system)", prov.systemGuardian.path)
      row("guardian parent == rootGuardian path", prov.guardian.path.parent == prov.rootGuardian.path)
      row("rootPath", prov.rootPath)
      row("rootPath == rootGuardian path", prov.rootPath == prov.rootGuardian.path)
      row("deadLetters path", prov.deadLetters.path)
    } finally { Await.result(sys.terminate(), 10.seconds) }
  }
}
