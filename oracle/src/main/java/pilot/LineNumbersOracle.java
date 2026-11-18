package pilot;

import java.io.Serializable;
import org.apache.pekko.util.LineNumbers;
import org.apache.pekko.util.LineNumbers$;

/**
 * Oracle cases of slice P75 (UTIL-ORIGEN-CLOSURES): AC-UTIL-016.8 (util/LineNumbers.scala:30-87 of Pekko 2.0.0), the
 * source-origin description of closures used in the {@code toString} of Typed behaviours. Rust cannot read JVM
 * bytecode (Q-UTIL-16, accepted: a {@code #[track_caller]} location, one line number, in place of the line table), so
 * the cases fix (1) the text of each {@code Result} and of {@code prettyName}, which the Rust twin reproduces exactly,
 * and (2) the SHAPE of the result for closures compiled in this project (kind, file name without directory, line
 * numbers positive and ordered), never the compiler-specific numbers themselves. Registered at the END of the
 * util-core list by {@link UtilCoreOracle}.
 */
final class LineNumbersOracle {
  private LineNumbersOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  interface F extends Serializable { String doit(String arg); }

  interface G { String doit(String arg); }

  static F oneLine() { return (s) -> s; }

  static F manyLines() {
    return (s) -> {
      System.out.println(s);
      return s;
    };
  }

  static F anonymous() {
    return new F() {
      private static final long serialVersionUID = 1L;

      @Override
      public String doit(String arg) {
        return arg;
      }
    };
  }

  static G notSerializable() { return (s) -> s; }

  /** Java API {@code LineNumbers.for}: {@code for} is a Java keyword, so it is called reflectively. */
  static Object javaFor(Object o) throws Exception {
    return LineNumbers$.class.getMethod("for", Object.class).invoke(LineNumbers$.MODULE$, o);
  }

  static String kind(LineNumbers.Result r) { return r.getClass().getSimpleName().replace("$", ""); }

  /** Shape of a result for a closure of this project: no compiler numbers. */
  static String shape(LineNumbers.Result r) {
    if (r instanceof LineNumbers.SourceFileLines l) {
      return "kind=SourceFileLines fileIsBaseName=" + (!l.filename().contains("/") && l.filename().contains("."))
          + " linesPositive=" + (l.from() > 0 && l.to() > 0) + " fromLeTo=" + (l.from() <= l.to());
    }
    return "kind=" + kind(r);
  }

  static String pretty(Object o, LineNumbers.Result r) {
    String p = LineNumbers$.MODULE$.prettyName(o);
    String cn = o.getClass().getName();
    if (r instanceof LineNumbers.SourceFileLines) {
      return "prettyIsPackageSlashResult=" + p.equals(o.getClass().getPackage().getName() + "/" + r);
    }
    return "prettyIsClassName=" + p.equals(cn) + " prettyHasNoLineSuffix=" + !p.matches(".*:-?\\d+(-\\d+)?$");
  }

  static void register() {
    run("LN-01 one line shows file:line", () ->
        new LineNumbers.SourceFileLines("Foo.scala", 22, 22) + " " + new LineNumbers.SourceFileLines("a/B.java", 1, 1));
    run("LN-02 a range shows file:from-to", () ->
        new LineNumbers.SourceFileLines("Foo.scala", 29, 31) + " " + new LineNumbers.SourceFileLines("X.java", 40, 45));
    run("LN-03 unordered or non-positive bounds are printed as given", () ->
        new LineNumbers.SourceFileLines("F.scala", 9, 5) + " " + new LineNumbers.SourceFileLines("F.scala", 0, 0) + " "
            + new LineNumbers.SourceFileLines("F.scala", -1, 3) + " " + new LineNumbers.SourceFileLines("", 7, 7)
            + " " + new LineNumbers.SourceFileLines("F.scala", Integer.MIN_VALUE, Integer.MAX_VALUE));
    run("LN-04 SourceFile NoSourceInfo and UnknownSourceFormat text", () ->
        "[" + new LineNumbers.SourceFile("Foo.scala") + "] [" + new LineNumbers.SourceFile("") + "] ["
            + LineNumbers.NoSourceInfo$.MODULE$ + "] [" + new LineNumbers.UnknownSourceFormat("parse error: x") + "] ["
            + new LineNumbers.UnknownSourceFormat("") + "]");
    run("LN-05 equality and hash code of results", () -> {
      LineNumbers.Result a = new LineNumbers.SourceFileLines("F", 1, 2), b = new LineNumbers.SourceFileLines("F", 1, 2),
          c = new LineNumbers.SourceFileLines("F", 1, 3);
      return "eq=" + a.equals(b) + " ne=" + !a.equals(c) + " hash=" + (a.hashCode() == b.hashCode()) + " fileEq="
          + new LineNumbers.SourceFile("F").equals(new LineNumbers.SourceFile("F")) + " kindsDiffer="
          + !new LineNumbers.SourceFile("F").equals(new LineNumbers.UnknownSourceFormat("F"));
    });
    run("LN-06 a one-line serializable lambda has a source file and lines", () -> {
      Object o = oneLine();
      LineNumbers.Result r = LineNumbers$.MODULE$.apply(o);
      return shape(r) + " " + pretty(o, r) + " single=" + (((LineNumbers.SourceFileLines) r).from() == ((LineNumbers.SourceFileLines) r).to());
    });
    run("LN-07 a multi-line serializable lambda has a source file and lines", () -> {
      Object o = manyLines();
      LineNumbers.Result r = LineNumbers$.MODULE$.apply(o);
      return shape(r) + " " + pretty(o, r);
    });
    run("LN-08 an anonymous class has a source file and lines", () -> {
      Object o = anonymous();
      LineNumbers.Result r = LineNumbers$.MODULE$.apply(o);
      return shape(r) + " " + pretty(o, r);
    });
    run("LN-09 a non-serializable lambda has no source info", () -> {
      Object o = notSerializable();
      LineNumbers.Result r = LineNumbers$.MODULE$.apply(o);
      return shape(r) + " " + pretty(o, r);
    });
    run("LN-10 the Java API for is the same as apply", () -> {
      Object o = oneLine();
      return "same=" + LineNumbers$.MODULE$.apply(o).equals(javaFor(o)) + " again="
          + LineNumbers$.MODULE$.apply(o).equals(LineNumbers$.MODULE$.apply(o));
    });
    run("LN-11 two closures of one site report the same origin and two sites differ", () -> {
      Object a = oneLine(), b = oneLine(), c = manyLines();
      return "sameSite=" + LineNumbers$.MODULE$.apply(a).equals(LineNumbers$.MODULE$.apply(b)) + " otherSite="
          + !LineNumbers$.MODULE$.apply(a).equals(LineNumbers$.MODULE$.apply(c));
    });
  }
}
