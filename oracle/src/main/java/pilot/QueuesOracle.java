package pilot;

import static pilot.StructuresOracle.*;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.LinkedList;
import java.util.List;
import java.util.Queue;
import java.util.concurrent.Callable;
import java.util.concurrent.TimeUnit;
import org.apache.pekko.util.BoundedBlockingQueue;
import org.apache.pekko.util.StablePriorityBlockingQueue;
import org.apache.pekko.util.StablePriorityQueue;

/**
 * Queue cases of slice P03: StablePriorityQueue / StablePriorityBlockingQueue (AC-UTIL-013) and BoundedBlockingQueue
 * (AC-UTIL-012). Sequence cases are single-threaded scripts; the concurrent cases wait for the thread under test to
 * park (Thread.getState) instead of sleeping, and print invariants only.
 */
final class QueuesOracle {
  private QueuesOracle() {}

  private static void run(String id, Callable<String> f) { UtilCoreOracle.run(id, f); }

  static Interp interpreter(String family) {
    switch (family) {
      case "stable-queue": return new StableInterp(false);
      case "stable-blocking-queue": return new StableInterp(true);
      case "bounded-queue": return new BoundedInterp();
      default: return SketchOracle.interpreter(family);
    }
  }

  // ------------------------------------------------------------ StablePriorityQueue

  static final class Item {
    final int prio;
    final String label;
    Item(int prio, String label) { this.prio = prio; this.label = label; }
    @Override public String toString() { return label; }
  }

  static final class StableInterp implements Interp {
    final java.util.AbstractQueue<Item> queue;
    int sequence = 0;

    StableInterp(boolean blocking) {
      Comparator<Item> cmp = (a, c) -> Integer.compare(a.prio, c.prio);
      queue = blocking ? new StablePriorityBlockingQueue<>(11, cmp) : new StablePriorityQueue<>(11, cmp);
    }

    static String show(Item i) { return i == null ? "null" : i.label; }

    public String op(String[] t) {
      switch (t[0]) {
        case "offer": return b(queue.offer(new Item(Integer.parseInt(t[1]), t[2])));
        case "poll": return show(queue.poll());
        case "peek": return show(queue.peek());
        case "size": return String.valueOf(queue.size());
        case "drain": {
          List<String> out = new ArrayList<>();
          Item i;
          while ((i = queue.poll()) != null) out.add(i.label);
          return list(out);
        }
        case "iterall": {
          // The iteration order of the backing heap is not part of the contract: print it sorted.
          List<String> out = new ArrayList<>();
          for (Item i : queue) out.add(i.label);
          java.util.Collections.sort(out);
          return list(out);
        }
        case "iterrm": {
          java.util.Iterator<Item> it = queue.iterator();
          int skip = Integer.parseInt(t[1]);
          for (int n = 0; n <= skip; n++) it.next();
          it.remove();
          return "size=" + queue.size();
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(3)) {
        case 0: case 1: return new String[] {"offer", String.valueOf(r.below(5)), "e" + (sequence++)};
        default: return new String[] {"poll"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"size"}, {"drain"}}; }
  }

  // ------------------------------------------------------------ BoundedBlockingQueue

  static Queue<String> backing(String kind) {
    switch (kind) {
      case "linked": return new LinkedList<>();
      case "priority": return new StablePriorityQueue<>(11, Comparator.<String>naturalOrder());
      case "blocking2": return new java.util.concurrent.LinkedBlockingQueue<>(2);
      case "full": { LinkedList<String> l = new LinkedList<>(); l.add("Hello"); return l; }
      default: throw new IllegalArgumentException("unknown backing " + kind);
    }
  }

  static final class BoundedInterp implements Interp {
    BoundedBlockingQueue<String> queue;

    static List<String> csv(String s) { return s.equals("-") ? new ArrayList<>() : Arrays.asList(s.split(",")); }

    public String op(String[] t) throws Exception {
      switch (t[0]) {
        case "new": queue = new BoundedBlockingQueue<>(Integer.parseInt(t[1]), backing(t[2])); return "ok";
        case "newnull": queue = new BoundedBlockingQueue<>(Integer.parseInt(t[1]), null); return "ok";
        case "offer": return b(queue.offer(t[1]));
        case "put": queue.put(t[1]); return "ok";
        case "poll": return String.valueOf(queue.poll());
        case "take": return queue.take();
        case "peek": return String.valueOf(queue.peek());
        case "rem": return b(queue.remove(t[1]));
        case "has": return b(queue.contains(t[1]));
        case "rc": return String.valueOf(queue.remainingCapacity());
        case "size": return String.valueOf(queue.size());
        case "empty": return b(queue.isEmpty());
        case "clear": queue.clear(); return "ok";
        case "drain": {
          List<String> sink = new ArrayList<>();
          int n = t[1].equals("all") ? queue.drainTo(sink) : queue.drainTo(sink, Integer.parseInt(t[1]));
          return n + list(sink);
        }
        case "containsAll": return b(queue.containsAll(csv(t[1])));
        case "removeAll": return b(queue.removeAll(csv(t[1])));
        case "retainAll": return b(queue.retainAll(csv(t[1])));
        case "array": return list(Arrays.asList(queue.toArray()));
        case "itall": { List<String> out = new ArrayList<>(); for (String s : queue) out.add(s); return list(out); }
        case "itrm": {
          java.util.Iterator<String> it = queue.iterator();
          for (int n = 0; n < Integer.parseInt(t[1]); n++) it.next();
          it.remove();
          return "removed";
        }
        case "itrm2": {
          java.util.Iterator<String> it = queue.iterator();
          it.next();
          it.remove();
          it.remove();
          return "removed twice";
        }
        case "itnext": {
          java.util.Iterator<String> it = queue.iterator();
          while (it.hasNext()) it.next();
          it.next();
          return "next";
        }
        case "offerT": return b(queue.offer(t[1], 1, TimeUnit.MILLISECONDS));
        case "pollT": return String.valueOf(queue.poll(1, TimeUnit.MILLISECONDS));
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    static final String[] ITEMS = {"a", "b", "c", "d", "e"};

    public String[] random(Rng r) {
      String x = ITEMS[r.below(ITEMS.length)];
      switch (r.below(14)) {
        case 0: case 1: case 2: return new String[] {"offer", x};
        case 3: case 4: return new String[] {"poll"};
        case 5: return new String[] {"peek"};
        case 6: return new String[] {"rem", x};
        case 7: return new String[] {"has", x};
        case 8: return new String[] {"drain", String.valueOf(1 + r.below(3))};
        case 9: return new String[] {"removeAll", x + "," + ITEMS[r.below(ITEMS.length)]};
        case 10: return new String[] {"retainAll", x + "," + ITEMS[r.below(ITEMS.length)] + "," + ITEMS[r.below(ITEMS.length)]};
        case 11: return new String[] {"offerT", x};
        case 12: return new String[] {"containsAll", x + "," + ITEMS[r.below(ITEMS.length)]};
        default: return new String[] {"rc"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"size"}, {"array"}, {"rc"}}; }
  }

  static void awaitParked(Thread t) {
    while (true) {
      Thread.State s = t.getState();
      if (s == Thread.State.WAITING || s == Thread.State.TIMED_WAITING) return;
      if (s == Thread.State.TERMINATED) throw new IllegalStateException("thread ended before parking");
      Thread.onSpinWait();
    }
  }

  static void register() {
    run("BQ-C1 one producer, one consumer through capacity 3 keep FIFO", () -> {
      BoundedBlockingQueue<Integer> q = new BoundedBlockingQueue<>(3, new LinkedList<>());
      int n = 2000;
      List<Integer> got = new ArrayList<>();
      boolean[] overfull = {false};
      Thread producer = new Thread(() -> {
        for (int i = 0; i < n; i++) q.put(i);
      });
      Thread consumer = new Thread(() -> {
                  for (int i = 0; i < n; i++) {
            if (q.remainingCapacity() < 0 || q.size() > 3) overfull[0] = true;
            got.add(q.take());
          }
      });
      producer.start(); consumer.start(); producer.join(); consumer.join();
      boolean fifo = true;
      for (int i = 0; i < n; i++) if (got.get(i) != i) fifo = false;
      return "received=" + got.size() + " fifo=" + b(fifo) + " neverOverfull=" + b(!overfull[0]) + " left=" + q.size();
    });

    run("BQ-C2 four producers and four consumers lose and duplicate nothing", () -> {
      BoundedBlockingQueue<Integer> q = new BoundedBlockingQueue<>(5, new LinkedList<>());
      int producers = 4, consumers = 4, each = 500;
      List<List<Integer>> got = new ArrayList<>();
      Thread[] ts = new Thread[producers + consumers];
      for (int p = 0; p < producers; p++) {
        final int base = p * 1_000_000;
        ts[p] = new Thread(() -> {
          for (int i = 0; i < each; i++) q.put(base + i);
        });
      }
      for (int c = 0; c < consumers; c++) {
        List<Integer> mine = new ArrayList<>();
        got.add(mine);
        ts[producers + c] = new Thread(() -> {
          for (int i = 0; i < each; i++) mine.add(q.take());
        });
      }
      for (Thread t : ts) t.start();
      for (Thread t : ts) t.join();
      java.util.Set<Integer> all = new java.util.HashSet<>();
      int total = 0;
      boolean perProducerOrder = true;
      for (List<Integer> mine : got) {
        int[] last = new int[producers];
        Arrays.fill(last, -1);
        for (int v : mine) {
          total++;
          all.add(v);
          int p = v / 1_000_000, i = v % 1_000_000;
          if (i <= last[p]) perProducerOrder = false;
          last[p] = i;
        }
      }
      return "total=" + total + " distinct=" + all.size() + " perProducerOrderPerConsumer=" + b(perProducerOrder) + " left=" + q.size();
    });

    run("BQ-C3 take blocks on an empty queue until an element arrives", () -> {
      BoundedBlockingQueue<String> q = new BoundedBlockingQueue<>(1, new LinkedList<>());
      String[] got = new String[1];
      Thread consumer = new Thread(() -> { got[0] = q.take(); });
      consumer.start();
      awaitParked(consumer);
      String before = String.valueOf(got[0]);
      q.put("x");
      consumer.join();
      return "before=" + before + " after=" + got[0] + " left=" + q.size();
    });

    run("BQ-C4 put blocks on a full queue until there is room", () -> {
      BoundedBlockingQueue<String> q = new BoundedBlockingQueue<>(1, new LinkedList<>());
      q.put("a");
      boolean[] done = {false};
      Thread producer = new Thread(() -> { q.put("b"); done[0] = true; });
      producer.start();
      awaitParked(producer);
      String during = "done=" + b(done[0]) + " size=" + q.size();
      String first = q.take();
      producer.join();
      return during + " first=" + first + " then=" + q.poll() + " done=" + b(done[0]);
    });

    run("BQ-C5 offer with a timeout gives up on a full queue and succeeds when room appears", () -> {
      BoundedBlockingQueue<String> q = new BoundedBlockingQueue<>(1, new LinkedList<>());
      q.put("a");
      boolean offeredOnFull = q.offer("b", 20, TimeUnit.MILLISECONDS);
      boolean[] result = new boolean[1];
      Thread producer = new Thread(() -> { result[0] = q.offer("c", 1000, TimeUnit.DAYS); });
      producer.start();
      awaitParked(producer);
      q.take();
      producer.join();
      return "gaveUp=" + b(!offeredOnFull) + " later=" + b(result[0]) + " content=" + list(Arrays.asList(q.toArray()));
    });

    run("BQ-C6 poll with a timeout returns null on an empty queue and the element when one arrives", () -> {
      BoundedBlockingQueue<String> q = new BoundedBlockingQueue<>(1, new LinkedList<>());
      String empty = String.valueOf(q.poll(20, TimeUnit.MILLISECONDS));
      String[] got = new String[1];
      Thread consumer = new Thread(() -> { got[0] = q.poll(1000, TimeUnit.DAYS); });
      consumer.start();
      awaitParked(consumer);
      q.put("z");
      consumer.join();
      return "empty=" + empty + " got=" + got[0];
    });

    run("BQ-C7 iterator remove drops the element it returned, by identity", () -> {
      BoundedBlockingQueue<String> q = new BoundedBlockingQueue<>(3, new LinkedList<>());
      String first = new String("a"), second = new String("a");
      q.put(first);
      q.put(second);
      java.util.Iterator<String> it = q.iterator();
      it.next();
      it.next();
      it.remove();
      Object[] left = q.toArray();
      return "size=" + q.size() + " keptFirst=" + b(left[0] == first) + " rc=" + q.remainingCapacity();
    });

    run("BQ-C8 the iterator is a snapshot", () -> {
      BoundedBlockingQueue<String> q = new BoundedBlockingQueue<>(4, new LinkedList<>());
      q.put("a");
      q.put("b");
      java.util.Iterator<String> it = q.iterator();
      q.put("c");
      q.poll();
      List<String> seen = new ArrayList<>();
      while (it.hasNext()) seen.add(it.next());
      return "seen=" + list(seen) + " now=" + list(Arrays.asList(q.toArray()));
    });
  }
}
