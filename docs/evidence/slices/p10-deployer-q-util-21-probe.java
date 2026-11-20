// Probe of Q-UTIL-21: run as Q21b.java (see docs/evidence/slices/p10-deployer.md).
import java.util.*;
import org.apache.pekko.util.WildcardIndex; import org.apache.pekko.util.WildcardIndex$; import org.apache.pekko.util.WildcardTree$;
import scala.collection.immutable.HashMap;

public class Q21b {
  static scala.collection.Iterable<String> it(String... a) { return scala.jdk.javaapi.CollectionConverters.asScala((java.lang.Iterable<String>) Arrays.asList(a)); }
  static String pekko(List<String> prefixesInOrder, String probe) {
    WildcardIndex<String> idx = WildcardIndex$.MODULE$.apply(WildcardTree$.MODULE$.apply(), WildcardTree$.MODULE$.apply());
    idx = idx.insert(new String[]{"q"}, "anchor"); // keep the single wildcard tree non-empty
    for (String p : prefixesInOrder) idx = idx.insert(new String[]{p + "*"}, p + "*");
    scala.Option<String> r = idx.find(it(probe));
    return r.isDefined() ? r.get() : "none";
  }
  // what iterating a Scala HashMap keyed by the prefixes gives: first prefix of the probe in iteration order
  @SuppressWarnings({"unchecked","rawtypes"})
  static String hashmapFirst(List<String> prefixes, String probe) {
    scala.collection.immutable.HashMap m = (scala.collection.immutable.HashMap) scala.collection.immutable.HashMap$.MODULE$.empty();
    for (String p : prefixes) m = (scala.collection.immutable.HashMap) m.updated(p, p + "*");
    scala.collection.Iterator iter = m.keysIterator();
    List<String> order = new ArrayList<>();
    String win = "none";
    while (iter.hasNext()) { String k = (String) iter.next(); order.add(k); if (win.equals("none") && probe.startsWith(k)) win = k + "*"; }
    return win + " (iteration " + order + ")";
  }
  public static void main(String[] a) {
    String[][] sets = {{"p","pq"},{"s","st"},{"a","ab"},{"x","xy"},{"m","mn"},{"b","bc"},{"ab","abc","a"},{"user","us","u"},{"worker","work","wo"},{"a","b","ab"}};
    int agree = 0, total = 0, longestWins = 0;
    for (String[] s : sets) {
      String probe;
      String longest = Arrays.stream(s).max(Comparator.comparingInt(String::length)).get();
      probe = longest + "zzz";
      List<String> fwd = Arrays.asList(s), rev = new ArrayList<>(fwd); java.util.Collections.reverse(rev);
      String r1 = pekko(fwd, probe), r2 = pekko(rev, probe);
      String h = hashmapFirst(fwd, probe);
      boolean eq = h.startsWith(r1 + " ") && r1.equals(r2);
      total++; if (eq) agree++;
      if (r1.equals(longest + "*")) longestWins++;
      System.out.println(fwd + " probe=" + probe + " | pekko fwd=" + r1 + " rev=" + r2 + " | HashMap first: " + h + " | orderIndependent=" + r1.equals(r2));
    }
    System.out.println("pekko result == first match in Scala HashMap iteration (and order independent): " + agree + "/" + total + "; the longest prefix is the winner in " + longestWins + "/" + total);
  }
}
