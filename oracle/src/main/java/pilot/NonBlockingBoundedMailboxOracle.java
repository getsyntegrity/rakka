package pilot;

import com.typesafe.config.ConfigFactory;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import org.apache.pekko.actor.*;
import org.apache.pekko.dispatch.*;

/**
 * Oracle for {@code NonBlockingBoundedMailbox} of Apache Pekko 2.0.0 (ADR-0001 PR 7b, AC-MBX-008): the lock-free bounded queue
 * ({@code BoundedNodeMessageQueue}) that never blocks the producer and sends the overflow to {@code deadLetters}. Output: one line per
 * case, {@code <ID> -> <observations>}, deterministic. Same determinism rules as {@link BoundedMailboxOracle}: no sleeps, concurrent
 * producers contribute totals, per-producer order and bounds only. The instruments are the public SPI (a real mailbox type, a real
 * {@code ActorSystem} for {@code deadLetters}); the queue is exercised alone, so every case is unit-level.
 */
final class NonBlockingBoundedMailboxOracle {
  private NonBlockingBoundedMailboxOracle() {}

  static final Map<String, Callable<String>> CASES = new LinkedHashMap<>();
  private static void run(String id, Callable<String> f) { if (CASES.put(id, f) != null) throw new IllegalStateException("dup " + id); }
  static List<String> ids() { register(); return new ArrayList<>(CASES.keySet()); }

  static String line(String id) {
    register();
    Callable<String> f = CASES.get(id);
    if (f == null) throw new IllegalArgumentException("unknown case id " + id);
    String r;
    try { r = f.call(); } catch (Throwable t) { r = "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]"; }
    return id + " -> " + r;
  }

  static MessageQueue newQueue(int capacity) { return new NonBlockingBoundedMailbox(capacity).create(scala.Option.empty(), scala.Option.empty()); }
  static String qs(MessageQueue q) { return "n=" + q.numberOfMessages() + " has=" + q.hasMessages(); }

  static String fromConfig(org.apache.pekko.actor.ActorSystem sys, String value) {
    try { return "capacity=" + new NonBlockingBoundedMailbox(sys.settings(), ConfigFactory.parseString("mailbox-capacity = " + value)).capacity(); }
    catch (Throwable t) { return BoundedMailboxOracle.err(t); }
  }

  static String missing(org.apache.pekko.actor.ActorSystem sys) {
    try { return "capacity=" + new NonBlockingBoundedMailbox(sys.settings(), ConfigFactory.empty()).capacity(); }
    catch (Throwable t) { return BoundedMailboxOracle.err(t); }
  }

  private static synchronized void register() {
    if (!CASES.isEmpty()) return;

    run("NB-CF-01 the constructor rejects a negative capacity; zero and the maximum pass", () -> {
      StringBuilder sb = new StringBuilder();
      for (int c : new int[] {-1, Integer.MIN_VALUE, 0, 1, Integer.MAX_VALUE}) {
        sb.append(sb.length() == 0 ? "" : " | ").append(c).append(" => ");
        try { int capacity = new NonBlockingBoundedMailbox(c).capacity(); sb.append("ok capacity=").append(capacity); } catch (Throwable t) { sb.append(BoundedMailboxOracle.err(t)); }
      }
      return sb.toString();
    });
    run("NB-CF-02 mailbox-capacity from configuration: integers, truncation, bad values; the push timeout is not read", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        StringBuilder sb = new StringBuilder();
        for (String v : new String[] {"-4", "0", "5", "1.5", "abc"}) sb.append(sb.length() == 0 ? "" : " | ").append(v).append(" => ").append(fromConfig(e.sys, v));
        return sb.append(" | missing => ").append(missing(e.sys)).toString();
      }
    });
    run("NB-CF-03 create() accepts every capacity the constructor accepts, and the queue has no push timeout", () -> {
      StringBuilder sb = new StringBuilder();
      for (int c : new int[] {0, 1, 3, Integer.MAX_VALUE}) {
        MessageQueue qu = newQueue(c);
        sb.append(sb.length() == 0 ? "" : " | ").append("cap=").append(c).append(" => ").append(qu.getClass().getName()).append(" ").append(qs(qu))
            .append(" capacity=").append(((BoundedNodeMessageQueue) qu).capacity()).append(" pushTimeOut=").append(((BoundedNodeMessageQueue) qu).pushTimeOut());
      }
      return sb.toString();
    });

    run("NB-QU-01 a full queue refuses at once, the refused go to dead letters in order, the queue keeps the first", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"), snd = e.idle("snd"); MessageQueue qu = newQueue(3); StringBuilder sb = new StringBuilder("new: " + qs(qu));
        for (String m : new String[] {"a", "b", "c", "d", "e"}) { qu.enqueue(recv, e.env(m, snd)); sb.append(" | ").append(m).append(": ").append(qs(qu)); }
        sb.append(" | dead=").append(e.deadNow()).append(" | drain=").append(BoundedMailboxOracle.drain(qu)).append(" ").append(qs(qu));
        return sb.toString();
      }
    });
    run("NB-QU-02 dequeue frees a slot: the next enqueue is accepted and the order is still first in first out", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(2);
        qu.enqueue(recv, e.env("a", recv)); qu.enqueue(recv, e.env("b", recv)); qu.enqueue(recv, e.env("c", recv));
        String first = (String) qu.dequeue().message();
        qu.enqueue(recv, e.env("d", recv)); qu.enqueue(recv, e.env("e", recv));
        return "first=" + first + " rest=" + BoundedMailboxOracle.drain(qu) + " dead=" + e.deadNow() + " " + qs(qu);
      }
    });
    run("NB-QU-03 capacity 1 keeps one; capacity 0 is a valid queue that refuses everything", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"), other = e.idle("other"); MessageQueue one = newQueue(1), zero = newQueue(0);
        for (String m : new String[] {"a", "b", "c"}) one.enqueue(recv, e.env(m, recv));
        List<String> deadOfOne = e.deadNow();
        for (String m : new String[] {"x", "y"}) zero.enqueue(other, e.env(m, other));
        List<String> deadOfZero = new ArrayList<>(e.deadNow()); deadOfZero.removeAll(deadOfOne);
        return "one: " + qs(one) + " drain=" + BoundedMailboxOracle.drain(one) + " dead=" + deadOfOne + " | zero: " + qs(zero) + " drain=" + BoundedMailboxOracle.drain(zero) + " dead=" + deadOfZero;
      }
    });
    run("NB-QU-04 the dead letter carries the message, the envelope's sender and the receiver; a DeadLetter message is not dropped", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"), snd = e.idle("snd"); MessageQueue qu = newQueue(1);
        qu.enqueue(recv, e.env("kept", snd));
        qu.enqueue(recv, e.env("refused", snd));
        qu.enqueue(recv, e.env("noSender", ActorRef.noSender()));
        qu.enqueue(recv, e.env(new DeadLetter("inner", snd, recv), snd));
        return "dead=" + e.deadNow() + " queue=" + BoundedMailboxOracle.drain(qu);
      }
    });
    run("NB-QU-05 cleanUp moves a full queue to the target in order and frees every slot", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(3), target = new UnboundedMailbox().create(scala.Option.empty(), scala.Option.empty());
        for (String m : new String[] {"a", "b", "c", "d"}) qu.enqueue(recv, e.env(m, recv));
        qu.cleanUp(recv, target);
        String after = qs(qu);
        for (String m : new String[] {"e", "f", "g", "h"}) qu.enqueue(recv, e.env(m, recv));
        return "source=" + after + " target=" + qs(target) + " order=" + BoundedMailboxOracle.drain(target) + " | refilled: " + qs(qu) + " drain=" + BoundedMailboxOracle.drain(qu) + " dead=" + e.deadNow();
      }
    });

    run("NB-QC-01 four producers, nobody dequeues: exactly capacity accepted, every other message dead-lettered", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(10);
        BoundedMailboxOracle.joinAll(BoundedMailboxOracle.producers(4, 50, (p, m) -> qu.enqueue(recv, e.env(m, recv))));
        e.awaitDead(190); List<String> dead = e.deadNow();
        List<String> kept = BoundedMailboxOracle.drain(qu);
        List<String> all = new ArrayList<>(kept); all.addAll(BoundedMailboxOracle.deadMessages(dead));
        return "accepted=" + kept.size() + " dead=" + dead.size() + " acceptedOrdered=" + BoundedMailboxOracle.perProducerOrdered(kept)
            + " deadOrdered=" + BoundedMailboxOracle.perProducerOrdered(BoundedMailboxOracle.deadMessages(dead))
            + " everyMessageAccountedOnce=" + (new HashSet<>(all).size() == 200 && all.size() == 200);
      }
    });
    run("NB-QC-02 four producers and one consumer: every message is received or dead-lettered exactly once, per-producer order, never above capacity", () -> {
      try (BoundedMailboxOracle.BEnv e = BoundedMailboxOracle.BEnv.bounded(1, "0s")) {
        ActorRef recv = e.idle("recv"); MessageQueue qu = newQueue(5);
        List<String> got = Collections.synchronizedList(new ArrayList<>()); AtomicBoolean done = new AtomicBoolean(); AtomicInteger maxSeen = new AtomicInteger();
        Thread consumer = new Thread(() -> {
          for (boolean last = false; !last; ) {
            last = done.get();
            maxSeen.accumulateAndGet(qu.numberOfMessages(), Math::max);
            for (Envelope x; (x = qu.dequeue()) != null; ) got.add((String) x.message());
          }
        });
        consumer.start();
        BoundedMailboxOracle.joinAll(BoundedMailboxOracle.producers(4, 50, (p, m) -> qu.enqueue(recv, e.env(m, recv))));
        done.set(true); consumer.join(20_000);
        e.awaitDead(200 - got.size()); List<String> dead = e.deadNow();
        List<String> all = new ArrayList<>(got); all.addAll(BoundedMailboxOracle.deadMessages(dead));
        return "everyMessageAccountedOnce=" + (new HashSet<>(all).size() == 200 && all.size() == 200) + " receivedOrdered=" + BoundedMailboxOracle.perProducerOrdered(got)
            + " neverAboveCapacity=" + (maxSeen.get() <= 5) + " " + qs(qu);
      }
    });
  }
}
