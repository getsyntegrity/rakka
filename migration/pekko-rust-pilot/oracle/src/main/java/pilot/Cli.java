package pilot;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

import org.apache.pekko.actor.ActorPath;
import org.apache.pekko.actor.ActorPaths;
import org.apache.pekko.actor.Address;

/**
 * Oracle CLI: runs the real Pekko 2.0.0 and prints what it observably does, as one line of JSON.
 *
 * <pre>
 *   path-from-string         stdin = one JSON string literal  -> one JSON result
 *   path-from-string-lines   stdin = one JSON string literal per line -> one JSON result per line
 *   scenario                 stdin = one scenario JSON (see Scenario) -> one JSON trace
 *   scenario-lines           stdin = one scenario JSON per line -> one JSON trace per line
 * </pre>
 *
 * The input travels as a JSON string literal on stdin (not argv) so that any UTF-16 sequence,
 * including lone surrogates, can be fed without depending on the platform's argv encoding.
 * The output is pure ASCII: every char outside 0x20..0x7e is written as a \\uXXXX escape.
 */
public final class Cli {
  private Cli() {}

  public static void main(String[] args) throws IOException {
    if (args.length != 1) usage();
    switch (args[0]) {
      case "path-from-string" -> {
        String in = new String(readAll(System.in), StandardCharsets.UTF_8);
        if (in.endsWith("\n")) in = in.substring(0, in.length() - 1);
        System.out.println(pathFromString(Json.parseString(in)));
      }
      case "path-from-string-lines" -> {
        String all = new String(readAll(System.in), StandardCharsets.UTF_8);
        StringBuilder out = new StringBuilder();
        for (String line : all.split("\n", -1)) {
          if (line.isEmpty()) continue;
          out.append(pathFromString(Json.parseString(line))).append('\n');
        }
        System.out.print(out);
      }
      case "scenario" -> {
        String in = new String(readAll(System.in), StandardCharsets.UTF_8);
        System.out.println(Scenario.run(in));
        System.exit(0); // the actor system's non-daemon threads must not keep the JVM alive
      }
      case "scenario-lines" -> {
        StringBuilder out = new StringBuilder();
        for (String line : new String(readAll(System.in), StandardCharsets.UTF_8).split("\n")) {
          if (!line.isEmpty()) out.append(Scenario.run(line)).append('\n');
        }
        System.out.print(out);
        System.out.flush();
        System.exit(0);
      }
      default -> usage();
    }
    System.out.flush();
  }

  private static void usage() {
    System.err.println("usage: Cli path-from-string | path-from-string-lines | scenario | scenario-lines   (JSON on stdin)");
    System.exit(2);
  }

  static String pathFromString(String s) {
    StringBuilder sb = new StringBuilder();
    try {
      ActorPath p = ActorPaths.fromString(s);
      Address a = p.address();
      sb.append("{\"ok\":true");
      sb.append(",\"toString\":").append(Json.str(p.toString()));
      sb.append(",\"name\":").append(Json.str(p.name()));
      sb.append(",\"elements\":[");
      List<String> els = new ArrayList<>();
      p.getElements().forEach(els::add);
      for (int i = 0; i < els.size(); i++) {
        if (i > 0) sb.append(',');
        sb.append(Json.str(els.get(i)));
      }
      sb.append(']');
      sb.append(",\"uid\":").append(p.uid());
      sb.append(",\"toStringWithoutAddress\":").append(Json.str(p.toStringWithoutAddress()));
      sb.append(",\"toSerializationFormat\":").append(Json.str(p.toSerializationFormat()));
      sb.append(",\"address\":{");
      sb.append("\"protocol\":").append(Json.str(a.protocol()));
      sb.append(",\"system\":").append(Json.str(a.system()));
      sb.append(",\"host\":").append(a.getHost().isPresent() ? Json.str(a.getHost().get()) : "null");
      sb.append(",\"port\":").append(a.getPort().isPresent() ? String.valueOf(a.getPort().get()) : "null");
      sb.append(",\"toString\":").append(Json.str(a.toString()));
      sb.append("}}");
    } catch (Exception e) {
      sb.setLength(0);
      sb.append("{\"ok\":false,\"exception\":").append(Json.str(e.getClass().getName()));
      sb.append(",\"message\":").append(e.getMessage() == null ? "null" : Json.str(e.getMessage()));
      sb.append('}');
    }
    return sb.toString();
  }

  private static byte[] readAll(InputStream in) throws IOException {
    ByteArrayOutputStream bo = new ByteArrayOutputStream();
    in.transferTo(bo);
    return bo.toByteArray();
  }

  /** Minimal JSON string reader/writer (ASCII-only output). */
  static final class Json {
    static String str(String s) {
      StringBuilder sb = new StringBuilder("\"");
      for (int i = 0; i < s.length(); i++) {
        char c = s.charAt(i);
        switch (c) {
          case '"' -> sb.append("\\\"");
          case '\\' -> sb.append("\\\\");
          default -> {
            if (c < 0x20 || c > 0x7e) sb.append(String.format("\\u%04x", (int) c));
            else sb.append(c);
          }
        }
      }
      return sb.append('"').toString();
    }

    static String parseString(String lit) {
      if (lit.length() < 2 || lit.charAt(0) != '"' || lit.charAt(lit.length() - 1) != '"')
        throw new IllegalArgumentException("stdin must be a JSON string literal, got: " + lit);
      StringBuilder sb = new StringBuilder();
      for (int i = 1; i < lit.length() - 1; i++) {
        char c = lit.charAt(i);
        if (c != '\\') { sb.append(c); continue; }
        char e = lit.charAt(++i);
        switch (e) {
          case '"' -> sb.append('"');
          case '\\' -> sb.append('\\');
          case '/' -> sb.append('/');
          case 'b' -> sb.append('\b');
          case 'f' -> sb.append('\f');
          case 'n' -> sb.append('\n');
          case 'r' -> sb.append('\r');
          case 't' -> sb.append('\t');
          case 'u' -> { sb.append((char) Integer.parseInt(lit.substring(i + 1, i + 5), 16)); i += 4; }
          default -> throw new IllegalArgumentException("bad escape \\" + e);
        }
      }
      return sb.toString();
    }
  }
}
