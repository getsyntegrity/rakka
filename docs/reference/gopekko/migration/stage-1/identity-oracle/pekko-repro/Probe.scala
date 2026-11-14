// Differential probe for the goPekko identity units (#852, #812, #1089, #853, #894, #1092):
// ActorPath navigation and parsing, AddressFromURIString, ActorPathExtractor,
// RelativeActorPath, isValidPathElement and stale-uid resolution.
// This file is a probe for a DISPOSABLE COPY of Pekko v1.7.1 (see RUN.md). It is NOT part of
// upstream, it is NOT built in this repository, and it never runs inside upstream/pekko.
// Output lines are "ROW|<label>|<value>"; observed.txt holds them without the "ROW|" prefix.
package org.apache.pekko.actor.probe
import org.apache.pekko.actor._
import scala.concurrent.Await
import scala.concurrent.duration._
import scala.util.Try

class A extends Actor { def receive = { case _ => } }
class Blocker(started: java.util.concurrent.CountDownLatch, gate: java.util.concurrent.CountDownLatch) extends Actor {
  def receive = { case _ => started.countDown(); gate.await() }
}

object Probe {
  // Unambiguous rendering: toString cannot tell List("") from Nil (both print "List()").
  def el(xs: Iterable[String]): String = "n=" + xs.size + " [" + xs.map("\"" + _ + "\"").mkString(",") + "]"
  def fmt(v: Any): String = v match {
    case xs: Iterable[_] => el(xs.map(String.valueOf))
    case Some(x)    => "Some(" + fmt(x) + ")"
    case (a, b)     => "(" + fmt(a) + "," + fmt(b) + ")"
    case other      => String.valueOf(other)
  }
  def r(label: String)(f: => Any): Unit = {
    val v = try { fmt(f) } catch { case t: Throwable => "THROWS " + t.getClass.getName + ": " + t.getMessage }
    println(s"ROW|$label|$v")
  }
  def main(args: Array[String]): Unit = {
    val root = RootActorPath(Address("pekko", "sys"))
    r("root.parent==root")(root.parent == root)
    r("root.parent")(root.parent)
    val ua = root / "user" / "a"
    r("(root/user/a).parent")(ua.parent)
    r("(root/user/a).parent.parent")(ua.parent.parent)
    r("root.descendant(user,'',a)")(Try(root.descendant(java.util.Arrays.asList("user", "", "a"))).fold(t => "THROWS " + t, p => p.toString + " elements=" + el(p.elements)))
    r("root./(Seq(user,'',a))")(Try(root / Seq("user", "", "a")).fold(t => "THROWS " + t, p => p.toString + " elements=" + el(p.elements)))
    r("root.descendant(Seq(''))")(Try(root.descendant(java.util.Arrays.asList(""))).fold(t => "THROWS " + t, p => p.toString + " elements=" + el(p.elements)))
    r("root.descendant(new java.util.ArrayList[String]())")(Try(root.descendant(new java.util.ArrayList[String]())).fold(t => "THROWS " + t, p => p.toString + " same=" + (p eq root) + " elements=" + el(p.elements)))
    r("root./(\"\")")(Try(root / "").fold(t => "THROWS " + t, p => p.toString + " elements=" + el(p.elements)))
    r("root.elements")(root.elements)
    r("ua.elements")(ua.elements)
    r("ua.name/root.name")(ua.name + "|" + root.name)
    r("ua.toString")(ua.toString)
    r("ua.toStringWithoutAddress")(ua.toStringWithoutAddress)
    r("ua.toSerializationFormat")(ua.toSerializationFormat)
    r("root.toString")(root.toString)
    r("root.toSerializationFormat")(root.toSerializationFormat)
    r("root==RootActorPath(same addr)")(root == RootActorPath(Address("pekko", "sys")))
    r("ua==root/user/a")(ua == (RootActorPath(Address("pekko", "sys")) / "user" / "a"))
    r("ua vs different system")(ua == (RootActorPath(Address("pekko", "sys2")) / "user" / "a"))
    r("ua.withUid(5)==ua")(ua.withUid(5) == ua)
    r("ua.withUid(5).toString")(ua.withUid(5).toString)
    r("ua.withUid(5).toSerializationFormat")(ua.withUid(5).toSerializationFormat)
    r("ua.uid")(ua.uid)
    r("ua.compareTo(root)")(ua.compareTo(root))
    r("ua.hashCode==other.hashCode")(ua.hashCode == (RootActorPath(Address("pekko", "sys")) / "user" / "a").hashCode)

    val fs = Seq("pekko://sys/user/a", "pekko://sys/", "pekko://sys", "pekko://sys//a", "pekko://sys/a/", "pekko://sys/a%41", "pekko://sys/a%4",
      "pekko://sys/a#5", "pekko://sys/user/a?q", "pekko://my_sys/user/a", "pekko://s y/a", "", "://x", "pekko://sys@h:1/a", "/user/a", "pekko://sys/a#0", "pekko://sys/a#x", "pekko://sys/a#-1", "pekko://sys/a#", "pekko://sys/a#5#6", "pekko://sys@h:1/user/a#7")
    fs.foreach { s =>
      r(s"ActorPath.fromString('$s')")(ActorPath.fromString(s) match { case p => s"toString=${p.toString} ser=${p.toSerializationFormat} uid=${p.uid} elements=${el(p.elements)} addr=${p.address} isRoot=${p.isInstanceOf[RootActorPath]}" })
      r(s"ActorPath.fromString-apply-via-Try('$s')")(Try(ActorPath.fromString(s)).isSuccess)
      r(s"AddressFromURIString.apply('$s')")(AddressFromURIString(s))
      r(s"AddressFromURIString.parse('$s')")(AddressFromURIString.parse(s))
      r(s"AddressFromURIString.unapply('$s')")(AddressFromURIString.unapply(s))
      r(s"ActorPathExtractor.unapply('$s')")(ActorPathExtractor.unapply(s))
    }
    val rels = Seq("foo/bar", "", "foo//bar", "../sibling", "foo?x=1", "a:b/c", "foo/%4", "foo/bar#1234", "foo/bar/", "/foo", "..", "a/./b")
    rels.foreach { s => r(s"RelativeActorPath.unapply('$s')")(RelativeActorPath.unapply(s)) }
    Seq("a", "a b", "$a", "a%41", "a%4", "", "-", "_a", "a#1", "a?", "a/b", "ä", "$$", "a@b", ".", "..", "a*", "a:b").foreach { s =>
      r(s"isValidPathElement('$s')")(ActorPath.isValidPathElement(s))
    }

    // stale uid
    val sys = ActorSystem("sys")
    try {
      val a1 = sys.actorOf(Props[A](), "a")
      val old = a1.path.toSerializationFormat
      val oldUid = a1.path.uid
      println(s"ROW|stale.old ser|$old")
      sys.stop(a1)
      Thread.sleep(500)
      val a2 = sys.actorOf(Props[A](), "a")
      println(s"ROW|stale.new ser|${a2.path.toSerializationFormat}")
      println(s"ROW|stale.uids differ|${oldUid != a2.path.uid}")
      val prov = sys.asInstanceOf[ExtendedActorSystem].provider
      def res(lbl: String, s: String): Unit = r(lbl) { val x = prov.resolveActorRef(s); s"class=${x.getClass.getSimpleName} eqNew=${x == a2} eqOld=${x == a1} path=${x.path.toSerializationFormat} isTerminated=${x.isInstanceOf[InternalActorRef] && x.asInstanceOf[InternalActorRef].isTerminated}" }
      res("stale.resolve old string with #uid", old)
      res("stale.resolve no-uid string", old.takeWhile(_ != '#'))
      res("stale.resolve #0", old.takeWhile(_ != '#') + "#0")
      res("stale.resolve #x", old.takeWhile(_ != '#') + "#x")
      res("stale.resolve current uid", a2.path.toSerializationFormat)
      res("stale.resolve nonexistent", "pekko://sys/user/zzz")
      res("stale.resolve other system", "pekko://other/user/a")
      res("stale.resolve remote addr", "pekko://sys@h:1/user/a")
      res("stale.resolve relative 'foo'", "foo")
      res("stale.resolve garbage ''", "")
      r("stale.deadLetters path")(sys.deadLetters.path.toSerializationFormat)
    } finally { Await.result(sys.terminate(), 10.seconds) }

    // Revision 3 (rows 197 on; rows 1-196 are unchanged): Owner decisions of 2026-11-04 (G2 of #1286 and #1289).
    // Relative text rows 11-13 of the #1089 scope plus %2F, %41, a lone '#', '#x', 'foo#', a lone '?', a non-ASCII text.
    Seq("foo", "pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355", "pekko://sys/user/a", "foo bar", "foo/a b", "%", "foo/%ZZ",
      "%2F", "%41", "#", "#x", "foo#", "?", "\u00e4").foreach { s => r(s"rev3.RelativeActorPath.unapply('$s')")(RelativeActorPath.unapply(s)) }
    // Uid forms and the stop window of K3, in a second system.
    val sys3 = ActorSystem("sys3")
    try {
      val a = sys3.actorOf(Props[A](), "a")
      val base = a.path.toStringWithoutAddress
      val prov3 = sys3.asInstanceOf[ExtendedActorSystem].provider
      def res3(lbl: String, str: String): Unit = r(lbl) { val x = prov3.resolveActorRef(str); s"class=${x.getClass.getSimpleName} eqLive=${x == a} path=${x.path.toSerializationFormat}" }
      val abs = "pekko://sys3" + base
      res3("rev3.resolve 'path#' (empty uid)", abs + "#")
      res3("rev3.resolve '#' alone", "#")
      res3("rev3.resolve uid 2147483647", abs + "#2147483647")
      res3("rev3.resolve uid 2147483648", abs + "#2147483648")
      res3("rev3.resolve uid 4294967296", abs + "#4294967296")
      res3("rev3.resolve uid -1", abs + "#-1")
      res3("rev3.resolve uid -5", abs + "#-5")
      res3("rev3.resolve live uid (negative or positive Int)", a.path.toSerializationFormat)
      println(s"ROW|rev3.live uid|${a.path.uid}")
      val parent = sys3.actorOf(Props(new Actor { val c = context.actorOf(Props[A](), "c"); def receive = { case "uid" => sender() ! c.path.uid } }), "p")
      val cuid = Await.result(org.apache.pekko.pattern.ask(parent, "uid")(3.seconds).mapTo[Int], 5.seconds)
      val pp = parent.path.toStringWithoutAddress
      res3("rev3.resolve uid on the intermediate element /p#<wrong>/c", "pekko://sys3" + pp + "#12345/c")
      res3("rev3.resolve uid on the intermediate element /p#<live>/c", "pekko://sys3" + pp + "#" + parent.path.uid + "/c")
      res3("rev3.resolve uid on both /p#<live>/c#<childuid>", "pekko://sys3" + pp + "#" + parent.path.uid + "/c#" + cuid)
      // K3 stop window: the stop is a system message processed after the current message, so the child is still in the container.
      val started = new java.util.concurrent.CountDownLatch(1)
      val gate = new java.util.concurrent.CountDownLatch(1)
      val b = sys3.actorOf(Props(new Blocker(started, gate)), "b")
      b ! "go"; started.await()
      sys3.stop(b)
      def resB(lbl: String, str: String): Unit = r(lbl) { val x = prov3.resolveActorRef(str); s"class=${x.getClass.getSimpleName} eqB=${x == b} isTerminated=${x.isInstanceOf[InternalActorRef] && x.asInstanceOf[InternalActorRef].isTerminated}" }
      resB("rev3.stop window: resolve with live uid, stop told, not processed", b.path.toSerializationFormat)
      resB("rev3.stop window: resolve without uid", "pekko://sys3" + b.path.toStringWithoutAddress)
      gate.countDown(); Thread.sleep(500)
      resB("rev3.after stop processed: resolve with the old uid", b.path.toSerializationFormat)
    } finally { Await.result(sys3.terminate(), 10.seconds) }
  }
}
