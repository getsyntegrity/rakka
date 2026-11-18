package pilot;

import static pilot.StructuresOracle.*;

import java.util.ArrayList;
import java.util.List;
import org.apache.pekko.util.FastFrequencySketch;
import org.apache.pekko.util.FastFrequencySketch$;
import org.apache.pekko.util.FrequencySketch;
import org.apache.pekko.util.FrequencySketch$;
import scala.util.hashing.MurmurHash3$;

/** FrequencySketch and FastFrequencySketch cases (AC-UTIL-017.1), Q-UTIL-14 (a): exact hashes and counters. */
final class SketchOracle {
  private SketchOracle() {}

  static Interp interpreter(String family) {
    switch (family) {
      case "sketch": return new SketchInterp();
      default: return ListsOracle.interpreter(family);
    }
  }

  @SuppressWarnings("unchecked")
  static FrequencySketch.Hasher<String> hasher() {
    try {
      return (FrequencySketch.Hasher<String>) Class.forName("org.apache.pekko.util.FrequencySketch$Hasher$StringHasher").getConstructor(int.class).newInstance(135283237);
    } catch (ReflectiveOperationException e) {
      throw new ExceptionInInitializerError(e);
    }
  }

  /** Items: a word is a string, "#N" is the boxed Integer N (its hashCode is N). */
  static Object item(String word) {
    return word.startsWith("#") ? (Object) Integer.valueOf(word.substring(1)) : word;
  }

  static final class SketchInterp implements Interp {
    FrequencySketch<Object> generic;
    FastFrequencySketch<Object> fast;

    @SuppressWarnings({"unchecked", "rawtypes"})
    public String op(String[] t) {
      switch (t[0]) {
        case "murmur": return String.valueOf(MurmurHash3$.MODULE$.stringHash(t[1].equals("-") ? "" : t[1], 135283237));
        case "murmurseed": return String.valueOf(MurmurHash3$.MODULE$.stringHash(t[2], Integer.parseInt(t[1])));
        case "fs": {
          generic = (FrequencySketch) FrequencySketch$.MODULE$.apply(Integer.parseInt(t[1]), Integer.parseInt(t[2]), Double.parseDouble(t[3]), Integer.parseInt(t[4]), Integer.parseInt(t[5]), hasher());
          fast = null;
          return "ok";
        }
        case "fsraw": {
          generic = (FrequencySketch) new FrequencySketch<String>(Integer.parseInt(t[1]), Integer.parseInt(t[2]), Integer.parseInt(t[3]), Integer.parseInt(t[4]), hasher());
          fast = null;
          return "ok";
        }
        case "ffs": {
          fast = (FastFrequencySketch) FastFrequencySketch$.MODULE$.apply(Integer.parseInt(t[1]), Integer.parseInt(t[2]), Double.parseDouble(t[3]));
          generic = null;
          return "ok";
        }
        case "ffsraw": {
          fast = (FastFrequencySketch) new FastFrequencySketch<Object>(Integer.parseInt(t[1]), Integer.parseInt(t[2]));
          generic = null;
          return "ok";
        }
        case "inc": {
          for (int n = 0; n < (t.length > 2 ? Integer.parseInt(t[2]) : 1); n++) {
            if (generic != null) generic.increment(item(t[1])); else fast.increment(item(t[1]));
          }
          return "ok";
        }
        case "freq": return String.valueOf(generic != null ? generic.frequency(item(t[1])) : fast.frequency(item(t[1])));
        case "size": return String.valueOf(generic != null ? generic.size() : fast.size());
        case "dbg": return (generic != null ? generic.toDebugString() : fast.toDebugString());
        case "bits": {
          FrequencySketch.Bits$ bits = FrequencySketch.Bits$.MODULE$;
          int i = Integer.parseInt(t[1]);
          return "pow2=" + b(bits.isPowerOfTwo(i)) + " exp=" + bits.powerOfTwoExponent(i) + " ceil=" + bits.ceilingPowerOfTwo(i);
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      int pool = 40;
      int rank = r.below(r.below(pool) + 1);
      String word = (fast != null && r.below(5) == 0 ? "#" : "k") + rank;
      if (r.below(4) == 0) return new String[] {"freq", word};
      return new String[] {"inc", word};
    }

    public String[][] epilogue() {
      List<String[]> out = new ArrayList<>();
      out.add(new String[] {"size"});
      for (int i = 0; i < 8; i++) out.add(new String[] {"freq", "k" + i});
      if (fast != null) for (int i = 0; i < 4; i++) out.add(new String[] {"freq", "#" + i});
      return out.toArray(new String[0][]);
    }
  }

  static void register() {}
}
