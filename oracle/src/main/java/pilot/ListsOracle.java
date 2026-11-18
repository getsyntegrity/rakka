package pilot;

import static pilot.StructuresOracle.*;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.apache.pekko.util.Clock;
import org.apache.pekko.util.DoubleLinkedList;
import org.apache.pekko.util.FrequencyList;
import org.apache.pekko.util.FrequencyList$;
import org.apache.pekko.util.RecencyList;
import org.apache.pekko.util.RecencyList$;
import org.apache.pekko.util.SegmentedRecencyList;
import org.apache.pekko.util.SegmentedRecencyList$;
import scala.concurrent.duration.FiniteDuration;

/** RecencyList (017.3), FrequencyList (017.2), SegmentedRecencyList (017.4) and DoubleLinkedList (017.5) cases. */
final class ListsOracle {
  private ListsOracle() {}

  static Interp interpreter(String family) {
    switch (family) {
      case "recency": return new RecencyInterp();
      case "frequency": return new FrequencyInterp();
      case "segmented": return new SegmentedInterp();
      case "double-list": return new DoubleListInterp();
      default: throw new IllegalArgumentException("unknown family " + family);
    }
  }

  /** The spec's TestClock: one tick = one second. */
  static final class TestClock implements Clock {
    long time = 0;
    void tick() { time += 1; }
    public long currentTime() { return time; }
    public long earlierTime(FiniteDuration duration) { return time - duration.toSeconds(); }
  }

  static FiniteDuration seconds(String s) { return FiniteDuration.apply(Long.parseLong(s), TimeUnit.SECONDS); }

  static List<String> csv(String s) {
    List<String> out = new ArrayList<>();
    if (!s.equals("-")) for (String x : s.split(",")) out.add(x);
    return out;
  }

  static String iter(scala.collection.Iterator<String> it) { return list(toJava(it)); }

  static String seq(scala.collection.immutable.Seq<String> xs) { return list(toJava(xs)); }

  static String opt(Object o) { return o == null ? "none" : "some(" + o + ")"; }

  static final String[] POOL = {"a", "b", "c", "d", "e", "f", "g", "h"};

  static String pick(Rng r) { return POOL[r.below(POOL.length)]; }

  // ------------------------------------------------------------ RecencyList

  static final class RecencyInterp implements Interp {
    final TestClock clock = new TestClock();
    final RecencyList<String> rl = RecencyList$.MODULE$.apply(clock);

    public String op(String[] t) {
      switch (t[0]) {
        case "tick": clock.tick(); return "t=" + clock.time;
        case "u": for (String x : csv(t[1])) rl.update(x); return "size=" + rl.size();
        case "rm": for (String x : csv(t[1])) rl.remove(x); return "size=" + rl.size();
        case "lr": return seq(rl.removeLeastRecent(Integer.parseInt(t[1])));
        case "lrs": return seq(rl.removeLeastRecent(Integer.parseInt(t[1]), Integer.parseInt(t[2])));
        case "lr1": return seq(rl.removeLeastRecent());
        case "mr": return seq(rl.removeMostRecent(Integer.parseInt(t[1])));
        case "mrs": return seq(rl.removeMostRecent(Integer.parseInt(t[1]), Integer.parseInt(t[2])));
        case "mr1": return seq(rl.removeMostRecent());
        case "lo": return seq(rl.removeLeastRecentOutside(seconds(t[1])));
        case "mw": return seq(rl.removeMostRecentWithin(seconds(t[1])));
        case "l": return iter(rl.leastToMostRecent());
        case "m": return iter(rl.mostToLeastRecent());
        case "has": return b(rl.contains(t[1]));
        case "size": return String.valueOf(rl.size());
        case "least": return opt(rl.leastRecent());
        case "most": return opt(rl.mostRecent());
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(16)) {
        case 0: return new String[] {"tick"};
        case 1: case 2: case 3: case 4: return new String[] {"u", pick(r)};
        case 5: return new String[] {"rm", pick(r)};
        case 6: return new String[] {"lr", String.valueOf(r.below(4))};
        case 7: return new String[] {"lrs", String.valueOf(r.below(3)), String.valueOf(r.below(3))};
        case 8: return new String[] {"mr", String.valueOf(r.below(4))};
        case 9: return new String[] {"mrs", String.valueOf(r.below(3)), String.valueOf(r.below(3))};
        case 10: return new String[] {"lo", String.valueOf(r.below(4))};
        case 11: return new String[] {"mw", String.valueOf(r.below(4))};
        case 12: return new String[] {"lr1"};
        case 13: return new String[] {"mr1"};
        case 14: return new String[] {"has", pick(r)};
        default: return new String[] {"l"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"l"}, {"m"}, {"size"}, {"least"}, {"most"}}; }
  }

  // ------------------------------------------------------------ FrequencyList

  static final class FrequencyInterp implements Interp {
    final TestClock clock = new TestClock();
    FrequencyList<String> fl = FrequencyList$.MODULE$.empty(false);
    boolean withClock = false;

    public String op(String[] t) {
      switch (t[0]) {
        case "new": {
          boolean aging = t[1].contains("aging");
          withClock = t[1].contains("clock");
          fl = withClock ? FrequencyList.withOverallRecency$.MODULE$.empty(clock, aging) : FrequencyList$.MODULE$.empty(aging);
          return "ok";
        }
        case "tick": clock.tick(); return "t=" + clock.time;
        case "u": for (String x : csv(t[1])) fl.update(x); return "size=" + fl.size();
        case "rm": for (String x : csv(t[1])) fl.remove(x); return "size=" + fl.size();
        case "rlf": return seq(fl.removeLeastFrequent(Integer.parseInt(t[1])));
        case "rlf1": return seq(fl.removeLeastFrequent());
        case "rlfs": return seq(fl.removeLeastFrequent(Integer.parseInt(t[1]), t[2].equals("-") ? null : t[2]));
        case "rmf": return seq(fl.removeMostFrequent(Integer.parseInt(t[1])));
        case "rmf1": return seq(fl.removeMostFrequent());
        case "rmfs": return seq(fl.removeMostFrequent(Integer.parseInt(t[1]), t[2].equals("-") ? null : t[2]));
        case "lf": return iter(fl.leastToMostFrequent());
        case "mf": return iter(fl.mostToLeastFrequent());
        case "has": return b(fl.contains(t[1]));
        case "size": return String.valueOf(fl.size());
        case "least": return opt(fl.leastFrequent());
        case "most": return opt(fl.mostFrequent());
        case "olr": return seq(fl.removeOverallLeastRecent(Integer.parseInt(t[1])));
        case "omr": return seq(fl.removeOverallMostRecent(Integer.parseInt(t[1])));
        case "olo": return seq(fl.removeOverallLeastRecentOutside(seconds(t[1])));
        case "omw": return seq(fl.removeOverallMostRecentWithin(seconds(t[1])));
        case "oall": return iter(fl.overallLeastToMostRecent());
        case "omall": return iter(fl.overallMostToLeastRecent());
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(18)) {
        case 0: return new String[] {"tick"};
        case 1: case 2: case 3: case 4: case 5: return new String[] {"u", pick(r)};
        case 6: return new String[] {"rm", pick(r)};
        case 7: return new String[] {"rlf", String.valueOf(r.below(4))};
        case 8: return new String[] {"rlfs", String.valueOf(r.below(3)), r.below(2) == 0 ? "-" : pick(r)};
        case 9: return new String[] {"rmf", String.valueOf(r.below(4))};
        case 10: return new String[] {"rmfs", String.valueOf(r.below(3)), r.below(2) == 0 ? "-" : pick(r)};
        case 11: return new String[] {"rlf1"};
        case 12: return new String[] {"rmf1"};
        case 13: return new String[] {"olr", String.valueOf(r.below(3))};
        case 14: return new String[] {"omr", String.valueOf(r.below(3))};
        case 15: return new String[] {"olo", String.valueOf(r.below(4))};
        case 16: return new String[] {"omw", String.valueOf(r.below(4))};
        default: return new String[] {"lf"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"lf"}, {"mf"}, {"size"}, {"least"}, {"most"}, {"oall"}}; }
  }

  // ------------------------------------------------------------ SegmentedRecencyList

  static scala.collection.immutable.Seq<Object> limits(String csv) {
    List<Object> out = new ArrayList<>();
    for (String x : csv.split(",")) out.add(Integer.valueOf(x));
    return scala.jdk.javaapi.CollectionConverters.asScala(out).toSeq();
  }

  static final class SegmentedInterp implements Interp {
    final TestClock clock = new TestClock();
    SegmentedRecencyList<String> sl = SegmentedRecencyList$.MODULE$.empty(limits("2,3"));
    int levels = 2;

    public String op(String[] t) {
      switch (t[0]) {
        case "new": {
          levels = t[1].split(",").length;
          sl = t.length > 2 && t[2].equals("clock")
              ? SegmentedRecencyList.withOverallRecency$.MODULE$.empty(clock, limits(t[1]))
              : SegmentedRecencyList$.MODULE$.empty(limits(t[1]));
          return "ok";
        }
        case "tick": clock.tick(); return "t=" + clock.time;
        case "u": for (String x : csv(t[1])) sl.update(x); return "size=" + sl.size();
        case "rm": for (String x : csv(t[1])) sl.remove(x); return "size=" + sl.size();
        case "rol": return seq(sl.removeLeastRecentOverLimit());
        case "lr": return seq(sl.removeLeastRecent());
        case "oo": return seq(sl.removeOverallLeastRecentOutside(seconds(t[1])));
        case "limits": sl.updateLimits(limits(t[1])); levels = t[1].split(",").length; return "ok";
        case "lvl": return iter(sl.leastToMostRecentOf(Integer.parseInt(t[1])));
        case "sizeof": return String.valueOf(sl.sizeOf(Integer.parseInt(t[1])));
        case "levels": {
          StringBuilder s = new StringBuilder();
          for (int i = 0; i < levels; i++) s.append(iter(sl.leastToMostRecentOf(i))).append(i + 1 < levels ? "/" : "");
          return s.toString();
        }
        case "has": return b(sl.contains(t[1]));
        case "size": return String.valueOf(sl.size());
        case "least": return opt(sl.leastRecent());
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(10)) {
        case 0: return new String[] {"tick"};
        case 1: case 2: case 3: case 4: return new String[] {"u", pick(r)};
        case 5: return new String[] {"rm", pick(r)};
        case 6: return new String[] {"rol"};
        case 7: return new String[] {"lr"};
        case 8: return new String[] {"oo", String.valueOf(r.below(4))};
        default: return new String[] {"levels"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"levels"}, {"size"}, {"least"}, {"sizeof", "0"}}; }
  }

  // ------------------------------------------------------------ DoubleLinkedList

  static final class ListNode {
    final String value;
    Object less, more;
    ListNode(String value) { this.value = value; }
    @Override public String toString() { return value; }
  }

  static final class DoubleListInterp implements Interp {
    @SuppressWarnings({"rawtypes", "unchecked"})
    final DoubleLinkedList<ListNode> list = new DoubleLinkedList(
        new scala.runtime.AbstractFunction1<ListNode, Object>() { public Object apply(ListNode n) { return n.less; } },
        new scala.runtime.AbstractFunction1<ListNode, Object>() { public Object apply(ListNode n) { return n.more; } },
        new scala.runtime.AbstractFunction2<ListNode, Object, scala.runtime.BoxedUnit>() {
          public scala.runtime.BoxedUnit apply(ListNode n, Object v) { n.less = v; return scala.runtime.BoxedUnit.UNIT; }
        },
        new scala.runtime.AbstractFunction2<ListNode, Object, scala.runtime.BoxedUnit>() {
          public scala.runtime.BoxedUnit apply(ListNode n, Object v) { n.more = v; return scala.runtime.BoxedUnit.UNIT; }
        });
    final Map<String, ListNode> nodes = new HashMap<>();

    ListNode node(String name) { return nodes.computeIfAbsent(name, ListNode::new); }

    static scala.runtime.AbstractFunction1<ListNode, Object> startsWith(String prefix) {
      return new scala.runtime.AbstractFunction1<ListNode, Object>() { public Object apply(ListNode n) { return n.value.startsWith(prefix); } };
    }

    scala.runtime.AbstractFunction0<ListNode> fresh(String name) {
      return new scala.runtime.AbstractFunction0<ListNode>() { public ListNode apply() { return node(name); } };
    }

    static String forward(DoubleLinkedList<ListNode> l) { return list(toJava(l.forwardIterator())); }

    public String op(String[] t) {
      switch (t[0]) {
        case "pre": return String.valueOf(list.prepend(node(t[1])));
        case "app": return String.valueOf(list.append(node(t[1])));
        case "rm": list.remove(node(t[1])); return "ok";
        case "front": return String.valueOf(list.moveToFront(node(t[1])));
        case "back": return String.valueOf(list.moveToBack(node(t[1])));
        case "gfp": return String.valueOf(list.getFirstOrElsePrepend(startsWith(t[1]), fresh(t[2])));
        case "glp": return String.valueOf(list.getLastOrElseAppend(startsWith(t[1]), fresh(t[2])));
        case "gno": return String.valueOf(list.getNextOrElseInsert(node(t[1]), startsWith(t[2]), fresh(t[3])));
        case "gpo": return String.valueOf(list.getPreviousOrElseInsert(node(t[1]), startsWith(t[2]), fresh(t[3])));
        case "fno": return String.valueOf(list.findNextOrElseInsert(node(t[1]), startsWith(t[2]), startsWith(t[3]), fresh(t[4])));
        case "fpo": return String.valueOf(list.findPreviousOrElseInsert(node(t[1]), startsWith(t[2]), startsWith(t[3]), fresh(t[4])));
        case "check": {
          List<String> back = toJava(list.backwardIterator()).stream().map(ListNode::toString).collect(java.util.stream.Collectors.toList());
          java.util.Collections.reverse(back);
          return "empty=" + b(list.isEmpty()) + " first=" + list.getFirst() + " last=" + list.getLast() + " fwd=" + forward(list) + " backReversed=" + list(back);
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    // Random operations keep to the contract: nodes are linked at most once, and only linked nodes are moved/removed.
    final List<String> linked = new ArrayList<>();
    int created = 0;

    public String[] random(Rng r) {
      int choice = r.below(8);
      if (linked.isEmpty() || choice < 2) {
        String name = "n" + (created++);
        linked.add(name);
        return new String[] {r.below(2) == 0 ? "pre" : "app", name};
      }
      String existing = linked.get(r.below(linked.size()));
      switch (choice) {
        case 2: linked.remove(existing); return new String[] {"rm", existing};
        case 3: return new String[] {"front", existing};
        case 4: return new String[] {"back", existing};
        default: return new String[] {"check"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"check"}}; }
  }
}
