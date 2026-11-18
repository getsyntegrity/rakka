package pilot;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** Minimal JSON reader: objects -> LinkedHashMap, arrays -> ArrayList, numbers -> Long/Double. */
final class Jv {
  private final String s;
  private int i;

  private Jv(String s) { this.s = s; }

  static Object parse(String text) {
    Jv p = new Jv(text);
    p.ws();
    Object v = p.value();
    p.ws();
    if (p.i != text.length()) throw new IllegalArgumentException("trailing JSON at " + p.i);
    return v;
  }

  private void ws() { while (i < s.length() && " \t\r\n".indexOf(s.charAt(i)) >= 0) i++; }

  private Object value() {
    char c = s.charAt(i);
    switch (c) {
      case '{': return object();
      case '[': return array();
      case '"': return string();
      case 't': i += 4; return Boolean.TRUE;
      case 'f': i += 5; return Boolean.FALSE;
      case 'n': i += 4; return null;
      default: return number();
    }
  }

  private Map<String, Object> object() {
    Map<String, Object> m = new LinkedHashMap<>();
    i++; ws();
    if (s.charAt(i) == '}') { i++; return m; }
    while (true) {
      ws(); String k = string(); ws(); i++; /* : */ ws();
      m.put(k, value()); ws();
      char c = s.charAt(i++);
      if (c == '}') return m;
    }
  }

  private List<Object> array() {
    List<Object> l = new ArrayList<>();
    i++; ws();
    if (s.charAt(i) == ']') { i++; return l; }
    while (true) {
      ws(); l.add(value()); ws();
      char c = s.charAt(i++);
      if (c == ']') return l;
    }
  }

  private String string() {
    int start = i;
    i++;
    while (s.charAt(i) != '"') { if (s.charAt(i) == '\\') i++; i++; }
    i++;
    return Cli.Json.parseString(s.substring(start, i));
  }

  private Object number() {
    int st = i;
    while (i < s.length() && "+-0123456789.eE".indexOf(s.charAt(i)) >= 0) i++;
    String t = s.substring(st, i);
    return (t.contains(".") || t.contains("e") || t.contains("E")) ? (Object) Double.valueOf(t) : (Object) Long.valueOf(t);
  }
}
