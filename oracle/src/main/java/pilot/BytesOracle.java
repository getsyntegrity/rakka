package pilot;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.apache.pekko.io.UnsynchronizedByteArrayInputStream;
import org.apache.pekko.util.ByteIterator;
import org.apache.pekko.util.ByteString;
import org.apache.pekko.util.ByteStringBuilder;
import pilot.StructuresOracle.Interp;
import pilot.StructuresOracle.Rng;

/**
 * Oracle cases of slice P04 (UTIL-BYTES): ByteString (AC-UTIL-006.1, .2, .6, .7), ByteStringBuilder (006.3),
 * ByteIterator (006.4, 016.3), the java.io adapters (006.5) and UnsynchronizedByteArrayInputStream (022) of Pekko 2.0.0.
 *
 * <p>Sequence cases are scripts shared with the Rust twin through {@code crates/rakka-actor/examples/classic-parity/bytes.tsv}
 * (one line per case, {@code id<TAB>family<TAB>script}); both sides run the same interpreter and print one token per
 * operation, exactly as {@link StructuresOracle} does. Data arguments are {@code xHEX} words (an empty word {@code x} is
 * an empty array); single bytes are signed or unsigned decimals cast with {@code (byte)}. Registered at the END of the
 * util-core list by {@link UtilCoreOracle}, after {@link StructuresOracle}.
 */
public final class BytesOracle {
  private BytesOracle() {}

  static List<String[]> scripts = null;

  static List<String[]> scripts() {
    if (scripts != null) return scripts;
    try {
      Path classes = Paths.get(BytesOracle.class.getProtectionDomain().getCodeSource().getLocation().toURI());
      Path file = classes.resolve("../../../crates/rakka-actor/examples/classic-parity/bytes.tsv").normalize();
      List<String[]> out = new ArrayList<>();
      for (String line : Files.readAllLines(file)) {
        if (line.isBlank() || line.startsWith("#")) continue;
        String[] f = line.split("\t", 3);
        if (f.length != 3) throw new IllegalStateException("bad line in bytes.tsv: " + line);
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
      UtilCoreOracle.run(id, () -> StructuresOracle.runScript(interpreter(family), script));
    }
    // Measured, not reproduced (Q-UTIL-23): the index error of a compact (ByteString1C) value is the JVM's array error.
    UtilCoreOracle.run("BS-APPLY-1C apply out of range on a compact value", () -> {
      try {
        ByteString.fromArray(new byte[] {97, 98, 99}).apply(3);
        return "no error";
      } catch (Throwable t) {
        return t.getClass().getSimpleName();
      }
    });
    // Measured, not reproduced (Q-UTIL-24): `++` of two array iterators that are not contiguous keeps only the right side.
    UtilCoreOracle.run("IT-CAT ++ of iterators over different arrays", () ->
        StructuresOracle.runScript(new IteratorInterp(), "iter x0102;catx x0304;len;tobs;iterrope x01 x02;catx x0304;len;tobs;iter x0102;catx x;len;iter x;catx x0304;len;tobs"));
    UtilCoreOracle.run("BS-HEAD-EMPTY head of the empty ByteString", () -> {
      try {
        ByteString.emptyByteString().head();
        return "no error";
      } catch (Throwable t) {
        return t.getClass().getSimpleName();
      }
    });
  }

  /** Debugging aid: {@code java -cp ... pilot.BytesOracle <family> "<script>"} prints the line of one ad-hoc script. */
  public static void main(String[] args) {
    System.out.println(StructuresOracle.runScript(interpreter(args[0]), args[1]));
  }

  static Interp interpreter(String family) {
    switch (family) {
      case "bytestring": return new StringInterp();
      case "builder": return new BuilderInterp();
      case "iterator": return new IteratorInterp();
      case "stream": return new StreamInterp();
      default: throw new IllegalArgumentException("unknown family " + family);
    }
  }

  // ---------------------------------------------------------------- shared helpers

  static byte[] hex(String word) {
    if (word.charAt(0) != 'x') throw new IllegalArgumentException("not a hex word: " + word);
    byte[] out = new byte[(word.length() - 1) / 2];
    for (int i = 0; i < out.length; i++) out[i] = (byte) Integer.parseInt(word.substring(1 + 2 * i, 3 + 2 * i), 16);
    return out;
  }

  static String hexs(byte[] bytes) {
    StringBuilder s = new StringBuilder("<");
    for (byte x : bytes) s.append(String.format("%02x", x & 0xff));
    return s.append('>').toString();
  }

  static String hexs(ByteString bs) { return hexs(bs.toArray()); }

  static ByteOrder order(String w) {
    if (w.equals("BE")) return ByteOrder.BIG_ENDIAN;
    if (w.equals("LE")) return ByteOrder.LITTLE_ENDIAN;
    throw new IllegalArgumentException("order " + w);
  }

  static byte sb(String w) { return (byte) Integer.parseInt(w); }

  static String t(boolean x) { return x ? "t" : "f"; }

  static final scala.Function1<Object, Object> lessThan(int k) {
    return new scala.runtime.AbstractFunction1<Object, Object>() {
      public Object apply(Object v) { return ((Byte) v).byteValue() < k; }
    };
  }

  static ByteString rope(String[] t, int from) {
    ByteString out = ByteString.emptyByteString();
    for (int i = from; i < t.length; i++) out = out.concat(ByteString.fromArray(hex(t[i])));
    return out;
  }

  /** The charset words of the scripts are resolved by name, so an unknown one reaches the library as a String. */
  static Charset charset(String w) { return Charset.forName(w); }

  // ---------------------------------------------------------------- ByteString (AC-UTIL-006.1, .2, .6, .7)

  static final class StringInterp implements Interp {
    final Map<String, ByteString> vars = new HashMap<>();

    ByteString v(String n) {
      ByteString x = vars.get(n);
      if (x == null) throw new IllegalStateException("unbound " + n);
      return x;
    }

    String bind(String n, ByteString x) { vars.put(n, x); return hexs(x); }

    public String op(String[] t) throws Exception {
      switch (t[0]) {
        case "arr": return bind(t[1], ByteString.fromArray(hex(t[2])));
        case "view": return bind(t[1], ByteString.fromArray(hex(t[2])).slice(Integer.parseInt(t[3]), Integer.parseInt(t[4])));
        case "rope": {
          ByteString out = ByteString.emptyByteString();
          for (int i = 2; i < t.length; i++) out = out.concat(ByteString.fromArray(hex(t[i])));
          return bind(t[1], out);
        }
        case "ints": {
          int[] xs = new int[t.length - 2];
          for (int i = 0; i < xs.length; i++) xs[i] = Integer.parseInt(t[2 + i]);
          return bind(t[1], ByteString.fromInts(xs));
        }
        case "fromarr": return bind(t[1], ByteString.fromArray(hex(t[2]), Integer.parseInt(t[3]), Integer.parseInt(t[4])));
        case "str": {
          String s = new String(hex(t[2]), StandardCharsets.UTF_8);
          return bind(t[1], t.length > 3 ? ByteString.fromString(s, t[3]) : ByteString.fromString(s));
        }
        case "buf": {
          ByteBuffer bb = ByteBuffer.wrap(hex(t[2]));
          bb.position(Integer.parseInt(t[3]));
          String r = bind(t[1], ByteString.fromByteBuffer(bb));
          return r + " pos=" + bb.position();
        }
        case "rnd": {
          // N SEED COUNT POOL: COUNT bytes drawn with the shared generator from the POOL bytes (every byte value when POOL is empty).
          Rng r = new Rng(Long.parseLong(t[2]));
          byte[] pool = hex(t[4]);
          byte[] out = new byte[Integer.parseInt(t[3])];
          for (int i = 0; i < out.length; i++) out[i] = pool.length == 0 ? (byte) r.below(256) : pool[r.below(pool.length)];
          return bind(t[1], ByteString.fromArray(out));
        }
        case "unsafe": return bind(t[1], ByteString.fromArrayUnsafe(hex(t[2])));
        case "compact": return bind(t[2], v(t[1]).compact());
        case "show": return v(t[1]).toString();
        case "len": return String.valueOf(v(t[1]).length());
        case "empty": return t(v(t[1]).isEmpty());
        case "apply": return String.valueOf(v(t[1]).apply(Integer.parseInt(t[2])));
        case "head": return String.valueOf(v(t[1]).head());
        case "last": return String.valueOf(v(t[1]).last());
        case "eq": return t(v(t[1]).equals(v(t[2])));
        case "take": return bind(t[1], v(t[2]).take(Integer.parseInt(t[3])));
        case "drop": return bind(t[1], v(t[2]).drop(Integer.parseInt(t[3])));
        case "dropr": return bind(t[1], v(t[2]).dropRight(Integer.parseInt(t[3])));
        case "taker": return bind(t[1], v(t[2]).takeRight(Integer.parseInt(t[3])));
        case "sl": return bind(t[1], v(t[2]).slice(Integer.parseInt(t[3]), Integer.parseInt(t[4])));
        case "cat": return bind(t[1], v(t[2]).concat(v(t[3])));
        case "tail": return bind(t[1], v(t[2]).tail());
        case "init": return bind(t[1], v(t[2]).init());
        case "splitat": {
          scala.Tuple2<ByteString, ByteString> p = v(t[1]).splitAt(Integer.parseInt(t[2]));
          return hexs(p._1()) + hexs(p._2());
        }
        case "span": {
          scala.Tuple2<ByteString, ByteString> p = v(t[1]).span(lessThan(Integer.parseInt(t[2])));
          return hexs(p._1()) + hexs(p._2());
        }
        case "takew": return bind(t[1], v(t[2]).takeWhile(lessThan(Integer.parseInt(t[3]))));
        case "dropw": return bind(t[1], v(t[2]).dropWhile(lessThan(Integer.parseInt(t[3]))));
        case "grouped": {
          scala.collection.Iterator<ByteString> it = v(t[1]).grouped(Integer.parseInt(t[2]));
          StringBuilder s = new StringBuilder("[");
          while (it.hasNext()) s.append(hexs(it.next()));
          return s.append(']').toString();
        }
        case "mapi": {
          int k = Integer.parseInt(t[3]);
          return bind(t[1], v(t[2]).mapI(new scala.runtime.AbstractFunction1<Object, Object>() {
            public Object apply(Object x) { return ((Byte) x).byteValue() + k; }
          }));
        }
        case "sum": {
          int s = 0;
          for (byte x : v(t[1]).toArray()) s += x;
          return String.valueOf(s);
        }
        case "toarr": return hexs(v(t[1]).toArray());
        case "idx": {
          ByteString bs = v(t[1]);
          byte x = sb(t[2]);
          if (t.length == 3) return String.valueOf(bs.indexOf(x));
          if (t.length == 4) return String.valueOf(bs.indexOf(x, Integer.parseInt(t[3])));
          return String.valueOf(bs.indexOf(x, Integer.parseInt(t[3]), Integer.parseInt(t[4])));
        }
        case "lidx": {
          ByteString bs = v(t[1]);
          return String.valueOf(t.length == 3 ? bs.lastIndexOf(sb(t[2])) : bs.lastIndexOf(sb(t[2]), Integer.parseInt(t[3])));
        }
        case "has": return t(v(t[1]).contains(Byte.valueOf(sb(t[2]))));
        case "idxs": {
          ByteString bs = v(t[1]);
          return String.valueOf(t.length == 3 ? bs.indexOfSlice(hex(t[2])) : bs.indexOfSlice(hex(t[2]), Integer.parseInt(t[3])));
        }
        case "lidxs": {
          ByteString bs = v(t[1]);
          return String.valueOf(t.length == 3 ? bs.lastIndexOfSlice(hex(t[2])) : bs.lastIndexOfSlice(hex(t[2]), Integer.parseInt(t[3])));
        }
        case "hass": return t(v(t[1]).containsSlice(ByteString.fromArray(hex(t[2]))));
        case "starts": {
          ByteString bs = v(t[1]);
          return t(t.length == 3 ? bs.startsWith(hex(t[2])) : bs.startsWith(hex(t[2]), Integer.parseInt(t[3])));
        }
        case "ends": return t(v(t[1]).endsWith(hex(t[2])));
        case "utf8": return v(t[1]).utf8String();
        case "dec": return t.length == 3 ? v(t[1]).decodeString(t[2]) : v(t[1]).decodeString(charset(t[2]), Integer.parseInt(t[3]), Integer.parseInt(t[4]));
        case "b64e": return bind(t[1], v(t[2]).encodeBase64());
        case "b64d": return bind(t[1], v(t[2]).decodeBase64());
        case "copyarr": {
          byte[] dest = new byte[Integer.parseInt(t[2])];
          int n = v(t[1]).copyToArray(dest, Integer.parseInt(t[3]), Integer.parseInt(t[4]));
          return "n=" + n + " dest=" + hexs(dest);
        }
        case "copybuf": {
          ByteBuffer bb = ByteBuffer.allocate(Integer.parseInt(t[2]));
          int n = v(t[1]).copyToBuffer(bb);
          return "n=" + n + " pos=" + bb.position() + " dest=" + hexs(bb.array());
        }
        case "tobuf": {
          ByteBuffer bb = v(t[1]).toByteBuffer();
          byte[] out = new byte[bb.remaining()];
          bb.get(out);
          return hexs(out);
        }
        case "iter": return hexs(v(t[1]).iterator().toByteString());
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      String[] names = {"a", "b", "c"};
      String x = names[r.below(3)], y = names[r.below(3)], z = names[r.below(3)];
      int n = r.below(16) - 3, m = r.below(16) - 3;
      switch (r.below(9)) {
        case 0: return new String[] {"cat", x, y, z};
        case 1: return new String[] {"take", x, y, String.valueOf(n)};
        case 2: return new String[] {"drop", x, y, String.valueOf(n)};
        case 3: return new String[] {"sl", x, y, String.valueOf(n), String.valueOf(m)};
        case 4: return new String[] {"dropr", x, y, String.valueOf(n)};
        case 5: return new String[] {"taker", x, y, String.valueOf(n)};
        case 6: return new String[] {"splitat", y, String.valueOf(n)};
        case 7: return new String[] {"idx", y, String.valueOf(r.below(6)), String.valueOf(n), String.valueOf(m)};
        default: return new String[] {"idxs", y, "x0" + r.below(6) + "0" + r.below(6), String.valueOf(n)};
      }
    }

    public String[][] epilogue() {
      return new String[][] {{"show", "a"}, {"show", "b"}, {"show", "c"}, {"len", "a"}, {"eq", "a", "b"}, {"toarr", "c"}};
    }
  }

  // ---------------------------------------------------------------- ByteStringBuilder (AC-UTIL-006.3)

  static final class BuilderInterp implements Interp {
    ByteStringBuilder builder = ByteString.newBuilder();

    public String op(String[] t) throws Exception {
      switch (t[0]) {
        case "byte": builder.putByte(sb(t[1])); return "ok";
        case "plus": builder.addOne(sb(t[1])); return "ok";
        case "short": builder.putShort(Integer.parseInt(t[1]), order(t[2])); return "ok";
        case "int": builder.putInt(Integer.parseInt(t[1]), order(t[2])); return "ok";
        case "long": builder.putLong(Long.parseLong(t[1]), order(t[2])); return "ok";
        case "longpart": builder.putLongPart(Long.parseLong(t[1]), Integer.parseInt(t[2]), order(t[3])); return "ok";
        case "float": builder.putFloat(Float.intBitsToFloat((int) Long.parseLong(t[1], 16)), order(t[2])); return "ok";
        case "double": builder.putDouble(Double.longBitsToDouble(Long.parseUnsignedLong(t[1], 16)), order(t[2])); return "ok";
        case "bytes": builder.putBytes(hex(t[1])); return "ok";
        case "bytesr": builder.putBytes(hex(t[1]), Integer.parseInt(t[2]), Integer.parseInt(t[3])); return "ok";
        case "append": builder.append(rope(t, 1)); return "ok";
        case "addall": builder.addAll(ByteString.fromArray(hex(t[1]))); return "ok";
        case "shorts": {
          short[] xs = new short[t.length - 2];
          for (int i = 0; i < xs.length; i++) xs[i] = (short) Integer.parseInt(t[2 + i]);
          builder.putShorts(xs, order(t[1]));
          return "ok";
        }
        case "ints": {
          int[] xs = new int[t.length - 2];
          for (int i = 0; i < xs.length; i++) xs[i] = Integer.parseInt(t[2 + i]);
          builder.putInts(xs, order(t[1]));
          return "ok";
        }
        case "intsr": {
          int[] xs = new int[t.length - 4];
          for (int i = 0; i < xs.length; i++) xs[i] = Integer.parseInt(t[4 + i]);
          builder.putInts(xs, Integer.parseInt(t[2]), Integer.parseInt(t[3]), order(t[1]));
          return "ok";
        }
        case "longs": {
          long[] xs = new long[t.length - 2];
          for (int i = 0; i < xs.length; i++) xs[i] = Long.parseLong(t[2 + i]);
          builder.putLongs(xs, order(t[1]));
          return "ok";
        }
        case "floats": {
          float[] xs = new float[t.length - 2];
          for (int i = 0; i < xs.length; i++) xs[i] = Float.intBitsToFloat((int) Long.parseLong(t[2 + i], 16));
          builder.putFloats(xs, order(t[1]));
          return "ok";
        }
        case "doubles": {
          double[] xs = new double[t.length - 2];
          for (int i = 0; i < xs.length; i++) xs[i] = Double.longBitsToDouble(Long.parseUnsignedLong(t[2 + i], 16));
          builder.putDoubles(xs, order(t[1]));
          return "ok";
        }
        case "write": builder.asOutputStream().write(Integer.parseInt(t[1])); return "ok";
        case "writeb": builder.asOutputStream().write(hex(t[1]), Integer.parseInt(t[2]), Integer.parseInt(t[3])); return "ok";
        case "len": return String.valueOf(builder.length());
        case "isempty": return t(builder.isEmpty());
        case "nonempty": return t(builder.nonEmpty());
        case "clear": builder.clear(); return "ok";
        case "hint": builder.sizeHint(Integer.parseInt(t[1])); return "ok";
        case "result": {
          ByteString r = builder.result();
          return hexs(r) + " len=" + r.length();
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(7)) {
        case 0: return new String[] {"byte", String.valueOf(r.below(256) - 128)};
        case 1: return new String[] {"short", String.valueOf(r.below(70000) - 3000), r.below(2) == 0 ? "BE" : "LE"};
        case 2: return new String[] {"int", String.valueOf(r.next() >> 33), r.below(2) == 0 ? "BE" : "LE"};
        case 3: return new String[] {"longpart", String.valueOf(r.next() >> 10), String.valueOf(r.below(9)), r.below(2) == 0 ? "BE" : "LE"};
        case 4: return new String[] {"bytes", "x" + Integer.toHexString(0x100 + r.below(256)).substring(1) + Integer.toHexString(0x100 + r.below(256)).substring(1)};
        case 5: return new String[] {"result"};
        default: return new String[] {"len"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"result"}, {"len"}, {"isempty"}}; }
  }

  // ---------------------------------------------------------------- ByteIterator (AC-UTIL-006.4, 016.3)

  static final class IteratorInterp implements Interp {
    ByteIterator it, it2;
    InputStream in;

    ByteIterator pick(String[] t) {
      if (t[0].startsWith("2:")) {
        t[0] = t[0].substring(2);
        return it2;
      }
      return it;
    }

    static String floats(float[] xs) {
      StringBuilder s = new StringBuilder("[");
      for (float x : xs) s.append(Integer.toHexString(Float.floatToRawIntBits(x))).append(',');
      return s.append(']').toString();
    }

    static String doubles(double[] xs) {
      StringBuilder s = new StringBuilder("[");
      for (double x : xs) s.append(Long.toHexString(Double.doubleToRawLongBits(x))).append(',');
      return s.append(']').toString();
    }

    public String op(String[] t) throws Exception {
      switch (t[0]) {
        case "iter": it = ByteString.fromArray(hex(t[1])).iterator(); return "ok";
        case "iterview": it = ByteString.fromArray(hex(t[1])).slice(Integer.parseInt(t[2]), Integer.parseInt(t[3])).iterator(); return "ok";
        case "iterrope": it = rope(t, 1).iterator(); return "ok";
        case "clone": it2 = it.clone(); return "ok";
        case "dup": {
          scala.Tuple2<ByteIterator, ByteIterator> p = it.duplicate();
          it = p._1();
          it2 = p._2();
          return "ok";
        }
        case "catx": it = (ByteIterator) it.$plus$plus(ByteString.fromArray(hex(t[1])).iterator()); return "ok";
        default:
      }
      ByteIterator i = pick(t);
      switch (t[0]) {
        case "next": return String.valueOf(i.next());
        case "head": return String.valueOf(i.head());
        case "hasnext": return t(i.hasNext());
        case "len": return String.valueOf(i.len());
        case "size": return String.valueOf(i.size());
        case "take": i.take(Integer.parseInt(t[1])); return "len=" + i.len();
        case "drop": i.drop(Integer.parseInt(t[1])); return "len=" + i.len();
        case "slice": i.slice(Integer.parseInt(t[1]), Integer.parseInt(t[2])); return "len=" + i.len();
        case "getbyte": return String.valueOf(i.getByte());
        case "getshort": return String.valueOf(i.getShort(order(t[1])));
        case "getint": return String.valueOf(i.getInt(order(t[1])));
        case "getlong": return String.valueOf(i.getLong(order(t[1])));
        case "getlongpart": return String.valueOf(i.getLongPart(Integer.parseInt(t[1]), order(t[2])));
        case "getfloat": return Integer.toHexString(Float.floatToRawIntBits(i.getFloat(order(t[1]))));
        case "getdouble": return Long.toHexString(Double.doubleToRawLongBits(i.getDouble(order(t[1]))));
        case "getbytes": return hexs(i.getBytes(Integer.parseInt(t[1])));
        case "getbytesinto": {
          byte[] dest = new byte[Integer.parseInt(t[1])];
          i.getBytes(dest, Integer.parseInt(t[2]), Integer.parseInt(t[3]));
          return hexs(dest) + " len=" + i.len();
        }
        case "getbs": return hexs(i.getByteString(Integer.parseInt(t[1])));
        case "getshorts": {
          short[] xs = new short[Integer.parseInt(t[2])];
          i.getShorts(xs, order(t[1]));
          return java.util.Arrays.toString(xs);
        }
        case "getints": {
          int[] xs = new int[Integer.parseInt(t[2])];
          i.getInts(xs, order(t[1]));
          return java.util.Arrays.toString(xs);
        }
        case "getlongs": {
          long[] xs = new long[Integer.parseInt(t[2])];
          i.getLongs(xs, order(t[1]));
          return java.util.Arrays.toString(xs);
        }
        case "getfloats": {
          float[] xs = new float[Integer.parseInt(t[2])];
          i.getFloats(xs, order(t[1]));
          return floats(xs);
        }
        case "getdoubles": {
          double[] xs = new double[Integer.parseInt(t[2])];
          i.getDoubles(xs, order(t[1]));
          return doubles(xs);
        }
        case "copyarr": {
          byte[] dest = new byte[Integer.parseInt(t[1])];
          int n = i.copyToArray(dest, Integer.parseInt(t[2]), Integer.parseInt(t[3]));
          return "n=" + n + " dest=" + hexs(dest) + " len=" + i.len();
        }
        case "copybuf": {
          ByteBuffer bb = ByteBuffer.allocate(Integer.parseInt(t[1]));
          int n = i.copyToBuffer(bb);
          return "n=" + n + " pos=" + bb.position() + " dest=" + hexs(bb.array()) + " len=" + i.len();
        }
        case "tobs": return hexs(i.toByteString()) + " len=" + i.len();
        case "idxof": return String.valueOf(t.length == 2 ? i.indexOf(sb(t[1])) : i.indexOf(sb(t[1]), Integer.parseInt(t[2])));
        case "takew": i.takeWhile(lessThan(Integer.parseInt(t[1]))); return "len=" + i.len();
        case "dropw": i.dropWhile(lessThan(Integer.parseInt(t[1]))); return "len=" + i.len();
        case "spanw": {
          scala.Tuple2<ByteIterator, ByteIterator> p = i.span(lessThan(Integer.parseInt(t[1])));
          return hexs(p._1().toByteString()) + hexs(p._2().toByteString());
        }
        case "ins": in = i.asInputStream(); return "ok";
        case "rd": return String.valueOf(in.read());
        case "rdb": {
          byte[] dest = new byte[Integer.parseInt(t[1])];
          int n = in.read(dest, Integer.parseInt(t[2]), Integer.parseInt(t[3]));
          return "n=" + n + " dest=" + hexs(dest);
        }
        case "skip": return String.valueOf(in.skip(Long.parseLong(t[1])));
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(8)) {
        case 0: return new String[] {"next"};
        case 1: return new String[] {"take", String.valueOf(r.below(10) - 2)};
        case 2: return new String[] {"drop", String.valueOf(r.below(10) - 2)};
        case 3: return new String[] {"getshort", r.below(2) == 0 ? "BE" : "LE"};
        case 4: return new String[] {"getbs", String.valueOf(r.below(6))};
        case 5: return new String[] {"copyarr", "4", String.valueOf(r.below(5)), String.valueOf(r.below(6))};
        case 6: return new String[] {"len"};
        default: return new String[] {"hasnext"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"len"}, {"tobs"}, {"hasnext"}}; }
  }

  // ---------------------------------------------------------------- java.io adapters (AC-UTIL-006.5, 022)

  static final class StreamInterp implements Interp {
    InputStream in;

    public String op(String[] t) throws Exception {
      switch (t[0]) {
        case "bsin": in = ByteString.fromArray(hex(t[1])).asInputStream(); return "ok";
        case "bsview": in = ByteString.fromArray(hex(t[1])).slice(Integer.parseInt(t[2]), Integer.parseInt(t[3])).asInputStream(); return "ok";
        case "bsrope": in = rope(t, 1).asInputStream(); return "ok";
        case "uin": in = t.length == 2 ? new UnsynchronizedByteArrayInputStream(hex(t[1])) : new UnsynchronizedByteArrayInputStream(hex(t[1]), Integer.parseInt(t[2]), Integer.parseInt(t[3])); return "ok";
        case "read": return String.valueOf(in.read());
        case "readb": {
          byte[] dest = new byte[Integer.parseInt(t[1])];
          int n = in.read(dest, Integer.parseInt(t[2]), Integer.parseInt(t[3]));
          return "n=" + n + " dest=" + hexs(dest);
        }
        case "avail": return String.valueOf(in.available());
        case "skip": return String.valueOf(in.skip(Long.parseLong(t[1])));
        case "mark": in.mark(Integer.parseInt(t[1])); return "ok";
        case "reset": in.reset(); return "ok";
        case "marksup": return t(in.markSupported());
        case "readall": return hexs(in.readAllBytes());
        case "readn": return hexs(in.readNBytes(Integer.parseInt(t[1])));
        case "readnb": {
          byte[] dest = new byte[Integer.parseInt(t[1])];
          int n = in.readNBytes(dest, Integer.parseInt(t[2]), Integer.parseInt(t[3]));
          return "n=" + n + " dest=" + hexs(dest);
        }
        case "skipn": in.skipNBytes(Long.parseLong(t[1])); return "ok";
        case "transfer": {
          java.io.ByteArrayOutputStream sink = new java.io.ByteArrayOutputStream();
          long n = in.transferTo(sink);
          return "n=" + n + " out=" + hexs(sink.toByteArray());
        }
        default: throw new IllegalArgumentException("unknown op " + t[0]);
      }
    }

    public String[] random(Rng r) {
      switch (r.below(4)) {
        case 0: return new String[] {"read"};
        case 1: return new String[] {"readb", "5", String.valueOf(r.below(3)), String.valueOf(r.below(4))};
        case 2: return new String[] {"skip", String.valueOf(r.below(4))};
        default: return new String[] {"avail"};
      }
    }

    public String[][] epilogue() { return new String[][] {{"read"}, {"avail"}}; }
  }
}
