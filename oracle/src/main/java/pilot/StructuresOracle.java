package pilot;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;
import java.util.concurrent.Callable;
import org.apache.pekko.util.*;

/**
 * Oracle cases of slice P03 (UTIL-ESTRUCTURAS): Index (AC-UTIL-007), SubclassifiedIndex (008), WildcardIndex (009),
 * MessageBuffer (010), TokenBucket (011), BoundedBlockingQueue (012), StablePriorityQueue (013), TypedMultiMap (014),
 * Switch (015.1), ImmutableIntMap (016.4), FrequencySketch (017.1), FrequencyList (017.2), RecencyList (017.3),
 * SegmentedRecencyList (017.4) and DoubleLinkedList (017.5) of Pekko 2.0.0.
 *
 * <p>Sequence cases are scripts shared with the Rust twin through {@code crates/rakka-actor/examples/classic-parity/structures.tsv}: one line per case,
 * {@code id<TAB>family<TAB>script}, a script being operations separated by ';' and words by spaces. Both sides run the
 * same interpreter and print one token per operation, so a divergence names the operation. A script operation
 * {@code rand SEED N} expands to N pseudo-random operations from the xorshift generator {@link Rng}, which both sides
 * implement identically. Concurrency cases are coded here and in the twin: they print invariants only (no sleeps, no
 * wall clock). Registered at the END of the util-core list by {@link UtilCoreOracle}.
 */
final class StructuresOracle {
  private StructuresOracle() {}

  private static void run(String id, Callable<String> f) { UtilCoreOracle.run(id, f); }

  // ---------------------------------------------------------------- shared helpers

  static String b(boolean x) { return x ? "t" : "f"; }

  static String err(Throwable t) {
    return "!" + t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
  }

  static <T> String list(Iterable<T> xs) {
    StringBuilder s = new StringBuilder("[");
    boolean first = true;
    for (T x : xs) {
      if (!first) s.append(',');
      s.append(x);
      first = false;
    }
    return s.append(']').toString();
  }

  static <T> List<T> toJava(scala.collection.Iterator<T> it) {
    List<T> out = new ArrayList<>();
    while (it.hasNext()) out.add(it.next());
    return out;
  }

  static <T> List<T> toJava(scala.collection.Iterable<T> xs) { return toJava(xs.iterator()); }

  static <T extends Comparable<T>> String sortedList(scala.collection.Iterable<T> xs) {
    List<T> l = toJava(xs);
    Collections.sort(l);
    return list(l);
  }

  /** xorshift64* generator, identical in the Rust twin. */
  static final class Rng {
    long s;
    Rng(long seed) { s = seed == 0 ? 0x9E3779B97F4A7C15L : seed; }
    long next() {
      s ^= s >>> 12;
      s ^= s << 25;
      s ^= s >>> 27;
      return s * 0x2545F4914F6CDD1DL;
    }
    int below(int bound) { return (int) ((next() >>> 33) % bound); }
  }

  interface Interp {
    String op(String[] t) throws Exception;
    /** One random operation for {@code rand}, as words. */
    String[] random(Rng r);
    /** Words that close a random run (summary reads of the state). */
    String[][] epilogue();
  }

  static String runScript(Interp in, String script) {
    List<String> out = new ArrayList<>();
    for (String raw : script.split(";")) {
      String text = raw.trim();
      if (text.isEmpty()) continue;
      String[] t = text.split(" ");
      if (t[0].equals("rand")) {
        Rng r = new Rng(Long.parseLong(t[1]));
        int n = Integer.parseInt(t[2]);
        for (int i = 0; i < n; i++) out.add(step(in, in.random(r)));
        for (String[] w : in.epilogue()) out.add(step(in, w));
      } else {
        out.add(step(in, t));
      }
    }
    return String.join(" ", out);
  }

  static String step(Interp in, String[] t) {
    try {
      return in.op(t);
    } catch (Throwable e) {
      return err(e);
    }
  }

  static List<String[]> scripts = null;

  /** Lines of the structures.tsv next to the Rust twin: id, family, script. */
  static List<String[]> scripts() {
    if (scripts != null) return scripts;
    try {
      Path classes = Paths.get(StructuresOracle.class.getProtectionDomain().getCodeSource().getLocation().toURI());
      Path file = classes.resolve("../../../crates/rakka-actor/examples/classic-parity/structures.tsv").normalize();
      List<String[]> out = new ArrayList<>();
      for (String line : Files.readAllLines(file)) {
        if (line.isBlank() || line.startsWith("#")) continue;
        String[] f = line.split("\t", 3);
        if (f.length != 3) throw new IllegalStateException("bad line in structures.tsv: " + line);
        out.add(f);
      }
      scripts = out;
      return out;
    } catch (IOException | java.net.URISyntaxException e) {
      throw new ExceptionInInitializerError(e);
    }
  }

  static void register() {
    for (String[] c : scripts()) {
      String id = c[0], family = c[1], script = c[2];
      run(id, () -> runScript(interpreter(family), script));
    }
    run("MB-DROP-ABSENT drop of an id that was never added", () -> {
      // Pekko 2.0.0 throws NullPointerException here (Q-UTIL-15 (b): the Rust port returns 0, an authorised divergence).
      MessageBufferMap<String> map = new MessageBufferMap<>();
      return "dropped=" + map.drop("x", "r", new NamedRef("deadLetters"));
    });
    concurrentCases();
    QueuesOracle.register();
    SketchOracle.register();
    // Last, so that the recorded case names of the cases above do not move.
    run("RG-01 ReentrantGuard is absent from the 2.0.0 jar", () -> {
      try {
        Class.forName("org.apache.pekko.util.ReentrantGuard");
        return "class=present";
      } catch (ClassNotFoundException e) {
        return "class=absent";
      }
    });
  }

  static Interp interpreter(String family) {
    switch (family) {
      case "index": return new IndexInterp(false);
      case "index-reversed": return new IndexInterp(true);
      case "int-map": return new IntMapInterp();
      case "typed-multimap": return new TypedMultiMapInterp();
      case "wildcard": return new WildcardInterp();
      case "subclassified": return new SubclassifiedInterp();
      case "message-buffer": return new MessageBufferInterp();
      case "switch": return new SwitchInterp();
      case "token-bucket": return new TokenBucketInterp();
      default: return QueuesOracle.interpreter(family);
    }
  }

  // ---------------------------------------------------------------- Index (AC-UTIL-007)

  static final class IndexInterp implements Interp {
    final Index<String, Integer> index;
    IndexInterp(boolean reversed) {
      Comparator<Integer> cmp = reversed ? (a, c) -> Integer.compare(c, a) : (a, c) -> Integer.compare(a, c);
      index = new Index<>(100, cmp);
    }

    static String values(scala.collection.Iterator<Integer> it) { return list(toJava(it)); }

    public String op(String[] t) {
      switch (t[0]) {
        case "put": return b(index.put(t[1], Integer.valueOf(t[2])));
        case "rm": return b(index.remove(t[1], Integer.valueOf(t[2])));
        case "rmk": {
          scala.Option<scala.collection.Iterable<Integer>> r = index.remove(t[1]);
          return r.isDefined() ? "some" + list(toJava(r.get())) : "none";
        }
        case "rmv": index.removeValue(Integer.valueOf(t[1])); return "ok";
        case "vals": return values(index.valueIterator(t[1]));
        case "find": {
          int n = Integer.parseInt(t[2]);
          scala.Option<Integer> r = index.findValue(t[1], new scala.runtime.AbstractFunction1<Integer, Object>() {
            public Object apply(Integer v) { return v >= n; }
          });
          return r.isDefined() ? "some(" + r.get() + ")" : "none";
        }
        case "allv": return sortedList(index.values());
        case "keys": return sortedList(index.keys());
        case "empty": return b(index.isEmpty());
        case "clear": index.clear(); return "ok";
        case "each": {
          TreeMap<String, List<Integer>> seen = new TreeMap<>();
          index.foreach(new scala.runtime.AbstractFunction2<String, Integer, scala.runtime.BoxedUnit>() {
            public scala.runtime.BoxedUnit apply(String k, Integer v) {
              seen.computeIfAbsent(k, x -> new ArrayList<>()).add(v);
              return scala.runtime.BoxedUnit.UNIT;
            }
          });
          StringBuilder s = new StringBuilder();
          for (var e : seen.entrySet()) s.append(e.getKey()).append('=').append(list(e.getValue())).append(' ');
          return "{" + s.toString().trim() + "}";
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      String key = "k" + r.below(4);
      String value = String.valueOf(r.below(6));
      switch (r.below(6)) {
        case 0: return new String[] {"put", key, value};
        case 1: return new String[] {"rm", key, value};
        case 2: return new String[] {"rmk", key};
        case 3: return new String[] {"rmv", value};
        case 4: return new String[] {"vals", key};
        default: return new String[] {"find", key, value};
      }
    }

    public String[][] epilogue() {
      return new String[][] {{"each"}, {"allv"}, {"keys"}, {"empty"}};
    }
  }

  // ---------------------------------------------------------------- ImmutableIntMap (AC-UTIL-016.4)

  static final int[] INT_KEYS = {Integer.MIN_VALUE, -5, -1, 0, 1, 2, 7, 100, Integer.MAX_VALUE};

  static final class IntMapInterp implements Interp {
    ImmutableIntMap m = ImmutableIntMap$.MODULE$.empty();
    final ImmutableIntMap[] marks = new ImmutableIntMap[4];

    public String op(String[] t) {
      switch (t[0]) {
        case "u": m = m.updated(Integer.parseInt(t[1]), Integer.parseInt(t[2])); return "ok";
        case "ua": {
          boolean[] evaluated = {false};
          int value = Integer.parseInt(t[2]);
          ImmutableIntMap next = m.updateIfAbsent(Integer.parseInt(t[1]), new scala.runtime.AbstractFunction0<Object>() {
            public Object apply() { evaluated[0] = true; return value; }
          });
          boolean same = next.equals(m);
          m = next;
          return (evaluated[0] ? "evaluated" : "lazy") + "/" + (same ? "same" : "new");
        }
        case "rm": {
          ImmutableIntMap next = m.remove(Integer.parseInt(t[1]));
          boolean same = next.equals(m);
          m = next;
          return same ? "same" : "new";
        }
        case "get": return String.valueOf(m.get(Integer.parseInt(t[1])));
        case "has": return b(m.contains(Integer.parseInt(t[1])));
        case "keys": return list(toJava(m.keysIterator()));
        case "str": return m.toString();
        case "hash": return String.valueOf(m.hashCode());
        case "size": return String.valueOf(m.size());
        case "mark": marks[Integer.parseInt(t[1])] = m; return "ok";
        case "eqm": {
          ImmutableIntMap o = marks[Integer.parseInt(t[1])];
          return "eq=" + b(m.equals(o)) + " hashEq=" + b(m.hashCode() == o.hashCode());
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      String key = String.valueOf(INT_KEYS[r.below(INT_KEYS.length)]);
      String value = String.valueOf(r.below(5) - 2);
      switch (r.below(7)) {
        case 0: case 1: return new String[] {"u", key, value};
        case 2: return new String[] {"ua", key, value};
        case 3: return new String[] {"rm", key};
        case 4: return new String[] {"get", key};
        case 5: return new String[] {"has", key};
        default: return new String[] {"hash"};
      }
    }

    public String[][] epilogue() {
      return new String[][] {{"keys"}, {"str"}, {"size"}, {"hash"}};
    }
  }

  // ---------------------------------------------------------------- TypedMultiMap (AC-UTIL-014)

  static scala.collection.immutable.Set<Object> scalaSet(List<Integer> xs) {
    scala.collection.immutable.Set<Object> s = scala.collection.immutable.Set$.MODULE$.empty();
    for (Integer x : xs) s = s.incl(x);
    return s;
  }

  static final class TypedMultiMapInterp implements Interp {
    @SuppressWarnings({"rawtypes", "unchecked"})
    TypedMultiMap<Object, Object> m = TypedMultiMap$.MODULE$.empty();
    TypedMultiMap<Object, Object> mark = m;

    static List<Integer> ints(String csv) {
      List<Integer> out = new ArrayList<>();
      if (!csv.equals("-")) for (String s : csv.split(",")) out.add(Integer.valueOf(s));
      return out;
    }

    @SuppressWarnings({"rawtypes", "unchecked"})
    public String op(String[] t) {
      switch (t[0]) {
        case "ins": m = m.inserted(t[1], Integer.valueOf(t[2])); return "ok";
        case "rm": m = m.removed(t[1], Integer.valueOf(t[2])); return "ok";
        case "vrm": m = m.valueRemoved(Integer.valueOf(t[1])); return "ok";
        case "krm": m = m.keyRemoved(t[1]); return "ok";
        case "setall": m = m.setAll(t[1], (scala.collection.immutable.Set) scalaSet(ints(t[2]))); return "ok";
        case "get": return sortedList((scala.collection.immutable.Set<Integer>) (scala.collection.immutable.Set) m.get(t[1]));
        case "keys": return sortedList((scala.collection.immutable.Set<String>) (scala.collection.immutable.Set) m.keySet());
        case "mark": mark = m; return "ok";
        case "plus": m = m.$plus$plus(mark); return "ok";
        case "eqm": return "eq=" + b(m.equals(mark)) + " hashEq=" + b(m.hashCode() == mark.hashCode());
        case "reset": m = TypedMultiMap$.MODULE$.empty(); return "ok";
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      String key = "k" + r.below(3);
      String value = String.valueOf(r.below(4));
      switch (r.below(9)) {
        case 0: case 1: case 2: return new String[] {"ins", key, value};
        case 3: return new String[] {"rm", key, value};
        case 4: return new String[] {"vrm", value};
        case 5: return new String[] {"krm", key};
        case 6: return new String[] {"setall", key, r.below(2) == 0 ? "-" : value + "," + (r.below(4))};
        case 7: return new String[] {"get", key};
        default: return new String[] {r.below(3) == 0 ? "mark" : "plus"};
      }
    }

    public String[][] epilogue() {
      return new String[][] {{"keys"}, {"get", "k0"}, {"get", "k1"}, {"get", "k2"}, {"eqm"}};
    }
  }

  // ---------------------------------------------------------------- WildcardIndex (AC-UTIL-009)

  static String[] path(String text) { return text.equals("-") ? new String[0] : text.split("/", -1); }

  static final class WildcardInterp implements Interp {
    WildcardIndex<Integer> index = WildcardIndex$.MODULE$.apply(WildcardTree$.MODULE$.apply(), WildcardTree$.MODULE$.apply());

    public String op(String[] t) {
      switch (t[0]) {
        case "ins": index = index.insert(path(t[1]), Integer.valueOf(t[2])); return "ok";
        case "find": {
          scala.Option<Integer> r = index.find(scala.jdk.javaapi.CollectionConverters.asScala(java.util.Arrays.asList(path(t[1]))));
          return r.isDefined() ? "some(" + r.get() + ")" : "none";
        }
        case "empty": return b(index.isEmpty());
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    static final String[] ELEMENTS = {"a", "ab", "b", "c", "x", "*", "**", "a*", "b*", "c*"};

    String insertedPath(Rng r) {
      int len = r.below(4);
      if (len == 0) return "-";
      StringBuilder s = new StringBuilder();
      for (int i = 0; i < len; i++) s.append(i == 0 ? "" : "/").append(ELEMENTS[r.below(ELEMENTS.length)]);
      return s.toString();
    }

    public String[] random(Rng r) {
      if (r.below(3) == 0) return new String[] {"ins", insertedPath(r), String.valueOf(r.below(9))};
      int len = r.below(4);
      if (len == 0) return new String[] {"find", "-"};
      String[] concrete = {"a", "ab", "abc", "b", "bc", "c", "cd", "x", "y"};
      StringBuilder s = new StringBuilder();
      for (int i = 0; i < len; i++) s.append(i == 0 ? "" : "/").append(concrete[r.below(concrete.length)]);
      return new String[] {"find", s.toString()};
    }

    public String[][] epilogue() { return new String[][] {{"empty"}}; }
  }

  // ---------------------------------------------------------------- SubclassifiedIndex (AC-UTIL-008)

  static final class DivisibilityClassification implements Subclassification<Integer> {
    public boolean isEqual(Integer x, Integer y) { return x.equals(y); }
    public boolean isSubclass(Integer x, Integer y) { return x % y == 0; }
  }

  static String changes(scala.collection.immutable.Seq<scala.Tuple2<Integer, scala.collection.immutable.Set<String>>> cs) {
    TreeMap<Integer, String> sorted = new TreeMap<>();
    for (scala.Tuple2<Integer, scala.collection.immutable.Set<String>> c : toJava(cs)) {
      if (sorted.containsKey(c._1())) throw new IllegalStateException("duplicate key in changes " + c._1());
      sorted.put(c._1(), sortedList(c._2()));
    }
    StringBuilder s = new StringBuilder();
    for (var e : sorted.entrySet()) s.append(e.getKey()).append(':').append(e.getValue()).append(' ');
    return "{" + s.toString().trim() + "}";
  }

  static final class SubclassifiedInterp implements Interp {
    final SubclassifiedIndex<Integer, String> index = new SubclassifiedIndex<>(new DivisibilityClassification());

    public String op(String[] t) {
      switch (t[0]) {
        case "addKey": return changes(index.addKey(Integer.valueOf(t[1])));
        case "addValue": return changes(index.addValue(Integer.valueOf(t[1]), t[2]));
        case "rmValue": return changes(index.removeValue(Integer.valueOf(t[1]), t[2]));
        case "rmAll": return changes(index.removeValue(t[1]));
        case "find": return sortedList(index.findValues(Integer.valueOf(t[1])));
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    static final int[] KEYS = {1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 18, 24, 30, 36};

    public String[] random(Rng r) {
      String key = String.valueOf(KEYS[r.below(KEYS.length)]);
      String value = "v" + r.below(3);
      switch (r.below(7)) {
        case 0: return new String[] {"addKey", key};
        case 1: case 2: case 3: return new String[] {"addValue", key, value};
        case 4: return new String[] {"rmValue", key, value};
        case 5: return new String[] {"rmAll", value};
        default: return new String[] {"find", key};
      }
    }

    public String[][] epilogue() {
      return new String[][] {{"find", "1"}, {"find", "6"}, {"find", "36"}, {"find", "5"}};
    }
  }

  // ---------------------------------------------------------------- MessageBuffer (AC-UTIL-010)

  /** An ActorRef that only knows its name, like the spec's DummyActorRef, and records what it is sent. */
  static final class NamedRef extends org.apache.pekko.actor.InternalActorRef {
    final String id;
    final List<String> received = new ArrayList<>();
    NamedRef(String id) { this.id = id; }
    @Override public String toString() { return id; }
    @Override public org.apache.pekko.actor.ActorRefProvider provider() { throw new UnsupportedOperationException(); }
    @Override public org.apache.pekko.actor.ActorPath path() { throw new UnsupportedOperationException(); }
    @Override public boolean isTerminated() { return false; }
    @Override public boolean isLocal() { return true; }
    @Override public void start() {}
    @Override public void resume(Throwable causedByFailure) {}
    @Override public void suspend() {}
    @Override public void restart(Throwable cause) {}
    @Override public void stop() {}
    @Override public void sendSystemMessage(org.apache.pekko.dispatch.sysmsg.SystemMessage message) {}
    @Override public org.apache.pekko.actor.InternalActorRef getParent() { return null; }
    @Override public org.apache.pekko.actor.InternalActorRef getChild(scala.collection.Iterator<String> names) { return null; }
    @Override public void $bang(Object message, org.apache.pekko.actor.ActorRef sender) {
      if (message instanceof org.apache.pekko.actor.Dropped d) {
        received.add(d.message() + "/" + d.reason() + "/" + d.sender() + "/" + (d.recipient() == org.apache.pekko.actor.Actor.noSender() ? "noSender" : "other"));
      } else received.add(String.valueOf(message));
    }
  }

  static final class MessageBufferInterp implements Interp {
    MessageBuffer buffer = MessageBuffer$.MODULE$.empty();
    final MessageBufferMap<String> map = new MessageBufferMap<>();
    final NamedRef deadLetters = new NamedRef("deadLetters");

    static String render(MessageBuffer buf) {
      List<String> items = new ArrayList<>();
      buf.foreach(new scala.runtime.AbstractFunction2<Object, org.apache.pekko.actor.ActorRef, scala.runtime.BoxedUnit>() {
        public scala.runtime.BoxedUnit apply(Object m, org.apache.pekko.actor.ActorRef s) {
          items.add(m + "->" + s);
          return scala.runtime.BoxedUnit.UNIT;
        }
      });
      return list(items);
    }

    public String op(String[] t) {
      switch (t[0]) {
        case "app": buffer.append(t[1], new NamedRef(t[2])); return "size=" + buffer.size();
        case "dh": buffer.dropHead(); return "size=" + buffer.size();
        case "head": {
          scala.Tuple2<Object, org.apache.pekko.actor.ActorRef> h = buffer.head();
          return "(" + h._1() + "," + h._2() + ")";
        }
        case "gethead": {
          org.apache.pekko.japi.Pair<Object, org.apache.pekko.actor.ActorRef> h = buffer.getHead();
          return "(" + h.first() + "," + h.second() + ")";
        }
        case "each": return render(buffer);
        case "fn": {
          List<String> drop = java.util.Arrays.asList(t[1].split(","));
          buffer.filterNot(new scala.runtime.AbstractFunction2<Object, org.apache.pekko.actor.ActorRef, Object>() {
            public Object apply(Object m, org.apache.pekko.actor.ActorRef s) { return drop.contains(m); }
          });
          return "size=" + buffer.size();
        }
        case "size": return String.valueOf(buffer.size());
        case "empty": return b(buffer.isEmpty()) + b(buffer.nonEmpty());
        case "ma": map.add(t[1]); return "ok";
        case "mp": map.append(t[1], t[2], new NamedRef(t[3])); return "ok";
        case "mr": map.remove(t[1]); return "ok";
        case "mc": return b(map.contains(t[1]));
        case "mg": return render(map.getOrEmpty(t[1]));
        case "mt": return String.valueOf(map.totalSize());
        case "ms": return map.size() + b(map.isEmpty()) + b(map.nonEmpty());
        case "md": {
          int n = map.drop(t[1], t[2], deadLetters);
          String got = list(new ArrayList<>(deadLetters.received));
          deadLetters.received.clear();
          return "dropped=" + n + " sent=" + got;
        }
        case "me": {
          TreeMap<String, String> seen = new TreeMap<>();
          map.foreach(new scala.runtime.AbstractFunction2<String, MessageBuffer, scala.runtime.BoxedUnit>() {
            public scala.runtime.BoxedUnit apply(String id, MessageBuffer buf) {
              seen.put(id, render(buf));
              return scala.runtime.BoxedUnit.UNIT;
            }
          });
          return seen.toString();
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      String id = "id" + r.below(3);
      String m = "m" + r.below(6);
      String s = "s" + r.below(3);
      switch (r.below(11)) {
        case 0: case 1: return new String[] {"app", m, s};
        case 2: return new String[] {"dh"};
        case 3: return new String[] {"fn", m + ",m" + r.below(6)};
        case 4: case 5: return new String[] {"mp", id, m, s};
        case 6: return new String[] {"mr", id};
        case 7: return new String[] {"ma", id};
        case 8: return new String[] {"mg", id};
        case 9: return map.contains(id) ? new String[] {"md", id, "why"} : new String[] {"mg", id};
        default: return new String[] {"head"};
      }
    }

    public String[][] epilogue() {
      return new String[][] {{"each"}, {"size"}, {"mt"}, {"ms"}, {"me"}};
    }
  }

  // ---------------------------------------------------------------- Switch (AC-UTIL-015.1)

  static final class SwitchInterp implements Interp {
    Switch sw = new Switch(false);
    final List<String> log = new ArrayList<>();

    scala.runtime.AbstractFunction0<scala.runtime.BoxedUnit> action(String name) {
      return new scala.runtime.AbstractFunction0<scala.runtime.BoxedUnit>() {
        public scala.runtime.BoxedUnit apply() { log.add(name); return scala.runtime.BoxedUnit.UNIT; }
      };
    }

    scala.runtime.AbstractFunction0<scala.runtime.BoxedUnit> thrower() {
      return new scala.runtime.AbstractFunction0<scala.runtime.BoxedUnit>() {
        public scala.runtime.BoxedUnit apply() { log.add("boom"); throw new RuntimeException("boom"); }
      };
    }

    scala.runtime.AbstractFunction0<Object> yielding(String v) {
      return new scala.runtime.AbstractFunction0<Object>() {
        public Object apply() { log.add("y"); return v; }
      };
    }

    static String opt(scala.Option<?> o) { return o.isDefined() ? "some(" + o.get() + ")" : "none"; }

    public String op(String[] t) {
      switch (t[0]) {
        case "start": sw = new Switch(t[1].equals("1")); return "ok";
        case "on": return b(sw.switchOn());
        case "off": return b(sw.switchOff());
        case "onA": return b(sw.switchOn(action("on")));
        case "offA": return b(sw.switchOff(action("off")));
        case "onX": try { sw.switchOn(thrower()); return "returned"; } catch (RuntimeException e) { return "threw:" + e.getMessage(); }
        case "offX": try { sw.switchOff(thrower()); return "returned"; } catch (RuntimeException e) { return "threw:" + e.getMessage(); }
        case "ifon": return b(sw.ifOn(action("ifon")));
        case "ifoff": return b(sw.ifOff(action("ifoff")));
        case "ifonY": return opt(sw.ifOnYield(yielding("yes")));
        case "ifoffY": return opt(sw.ifOffYield(yielding("yes")));
        case "whon": return b(sw.whileOn(action("whon")));
        case "whoff": return b(sw.whileOff(action("whoff")));
        case "whonY": return opt(sw.whileOnYield(yielding("yes")));
        case "whoffY": return opt(sw.whileOffYield(yielding("yes")));
        case "fold": return String.valueOf(sw.fold(yielding("on"), yielding("off")));
        case "locked": return String.valueOf(sw.locked(yielding("locked")));
        // Re-entrancy: the lock is the monitor of the switch, so a thread holding it can use the switch again.
        case "lockedOn": return String.valueOf(sw.locked(new scala.runtime.AbstractFunction0<Object>() {
          public Object apply() { return "inner=" + b(sw.switchOn()) + b(sw.whileOn(action("nested"))) + b(sw.isOn()); }
        }));
        case "ison": return b(sw.isOn());
        case "isoff": return b(sw.isOff());
        case "log": { String s = list(new ArrayList<>(log)); log.clear(); return s; }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    static final String[] OPS = {"on", "off", "onA", "offA", "onX", "offX", "ifon", "ifoff", "ifonY", "ifoffY", "whon", "whoff", "whonY", "whoffY", "fold", "locked", "lockedOn", "ison", "isoff"};

    public String[] random(Rng r) { return new String[] {OPS[r.below(OPS.length)]}; }

    public String[][] epilogue() { return new String[][] {{"ison"}, {"log"}}; }
  }

  // ---------------------------------------------------------------- TokenBucket (AC-UTIL-011)

  static final class ClockedBucket extends TokenBucket {
    long now;
    ClockedBucket(long capacity, long period) { super(capacity, period); }
    @Override public long currentTime() { return now; }
  }

  static final class TokenBucketInterp implements Interp {
    ClockedBucket bucket;
    long pendingTime = 0;

    public String op(String[] t) {
      switch (t[0]) {
        case "new": bucket = new ClockedBucket(Long.parseLong(t[1]), Long.parseLong(t[2])); bucket.now = pendingTime; return "ok";
        case "time": pendingTime = Long.parseLong(t[1]); if (bucket != null) bucket.now = pendingTime; return "ok";
        case "init": bucket.init(); return "ok";
        case "setup": {
          bucket = new ClockedBucket(Long.parseLong(t[1]), Long.parseLong(t[2]));
          bucket.now = Long.parseLong(t[3]);
          bucket.init();
          return "ok";
        }
        case "add": bucket.now += Long.parseLong(t[1]); return "ok";
        case "o": return String.valueOf(bucket.offer(Long.parseLong(t[1])));
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    static final long[] CAPACITIES = {0, 1, 2, 5, 10, 100};
    static final long[] PERIODS = {1, 2, 3, 7, Long.MAX_VALUE >> 10};
    static final long[] STARTS = {0, 1, -100, Long.MIN_VALUE, Long.MAX_VALUE, Long.MAX_VALUE - 5, 123456789L};
    static final long[] STEPS = {0, 1, 2, 3, 5, 20, 100, 1_000_000};
    static final long[] COSTS = {0, 1, 2, 5, 10, 20};
    boolean started = false;

    public String[] random(Rng r) {
      if (!started) {
        started = true;
        return new String[] {"setup", String.valueOf(CAPACITIES[r.below(CAPACITIES.length)]), String.valueOf(PERIODS[r.below(PERIODS.length)]), String.valueOf(STARTS[r.below(STARTS.length)])};
      }
      if (r.below(3) == 0) return new String[] {"add", String.valueOf(STEPS[r.below(STEPS.length)])};
      return new String[] {"o", String.valueOf(COSTS[r.below(COSTS.length)])};
    }

    public String[][] epilogue() { return new String[0][]; }
  }

  // ---------------------------------------------------------------- concurrent cases (invariants only)

  static void concurrentCases() {
    run("CC-01 Index in parallel: the upper half is never lost", () -> {
      Comparator<Integer> natural = (a, c) -> Integer.compare(a, c);
      Index<Integer, Integer> index = new Index<>(100, natural);
      int keys = 10, values = 10, threads = 8, tasksPerThread = 1500;
      for (int k = 0; k < keys; k++) for (int v = 0; v < values; v++) index.put(k, v);
      java.util.concurrent.CountDownLatch go = new java.util.concurrent.CountDownLatch(1);
      java.util.concurrent.atomic.AtomicBoolean upperAlwaysFilled = new java.util.concurrent.atomic.AtomicBoolean(true);
      Thread[] ts = new Thread[threads];
      for (int i = 0; i < threads; i++) {
        final long seed = 1000 + i;
        ts[i] = new Thread(() -> {
          Rng r = new Rng(seed);
          try { go.await(); } catch (InterruptedException e) { throw new RuntimeException(e); }
          for (int n = 0; n < tasksPerThread; n++) {
            switch (r.below(4)) {
              case 0: index.put(r.below(keys), r.below(values)); break;
              case 1: index.remove(r.below(keys / 2), r.below(values)); break;
              case 2: index.remove(r.below(keys / 2)); break;
              default: {
                int key = r.below(keys);
                if (key >= keys / 2 && !index.valueIterator(key).hasNext()) upperAlwaysFilled.set(false);
              }
            }
          }
        });
        ts[i].start();
      }
      go.countDown();
      for (Thread t : ts) t.join();
      StringBuilder upper = new StringBuilder();
      for (int k = keys / 2; k < keys; k++) upper.append(list(toJava(index.valueIterator(k)))).append(' ');
      boolean lowerWithinRange = true;
      for (int k = 0; k < keys / 2; k++)
        for (Integer v : toJava(index.valueIterator(k))) if (v < 0 || v >= values) lowerWithinRange = false;
      return "upperAlwaysFilled=" + upperAlwaysFilled.get() + " upper=" + upper.toString().trim() + " lowerWithinRange=" + lowerWithinRange;
    });

    run("CC-02 Switch: exactly one of 8 racing switchOn wins", () -> {
      Switch sw = new Switch(false);
      int threads = 8;
      java.util.concurrent.CountDownLatch go = new java.util.concurrent.CountDownLatch(1);
      java.util.concurrent.atomic.AtomicInteger winners = new java.util.concurrent.atomic.AtomicInteger();
      Thread[] ts = new Thread[threads];
      for (int i = 0; i < threads; i++) {
        ts[i] = new Thread(() -> {
          try { go.await(); } catch (InterruptedException e) { throw new RuntimeException(e); }
          if (sw.switchOn()) winners.incrementAndGet();
        });
        ts[i].start();
      }
      go.countDown();
      for (Thread t : ts) t.join();
      return "winners=" + winners.get() + " on=" + b(sw.isOn());
    });

    run("CC-03 Switch: switchOff waits for locked", () -> {
      Switch sw = new Switch(true);
      java.util.concurrent.atomic.AtomicBoolean offDone = new java.util.concurrent.atomic.AtomicBoolean(false);
      String[] whileLocked = new String[1];
      Thread[] other = new Thread[1];
      sw.locked(new scala.runtime.AbstractFunction0<Object>() {
        public Object apply() {
          other[0] = new Thread(() -> { sw.switchOff(); offDone.set(true); });
          other[0].start();
          // The other thread is parked on the monitor of the switch: it is BLOCKED, not finished.
          while (other[0].getState() != Thread.State.BLOCKED) Thread.onSpinWait();
          whileLocked[0] = "on=" + b(sw.isOn()) + " offDone=" + b(offDone.get());
          return null;
        }
      });
      other[0].join();
      return "whileLocked[" + whileLocked[0] + "] after: on=" + b(sw.isOn()) + " offDone=" + b(offDone.get());
    });
  }
}
