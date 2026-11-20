package pilot;

import java.util.Arrays;
import org.apache.pekko.util.WildcardIndex;
import org.apache.pekko.util.WildcardIndex$;
import org.apache.pekko.util.WildcardTree$;

/**
 * Differential-fuzz oracle for Q-UTIL-21: which of several overlapping {@code prefix*} entries the real {@code WildcardIndex}
 * of Pekko 2.0.0 returns. One input line is {@code prefix,prefix,...|probe}: the prefixes are inserted in that order as
 * {@code [prefix + "*", "x"]} with the prefix as data, and the probe element is looked up as {@code [probe, "x"]}; the output
 * line is the prefix found or {@code none}. A leading {@code q} entry keeps the single wildcard tree non-empty.
 */
final class WildcardWinnersOracle {
  private WildcardWinnersOracle() {}

  static String line(String input) {
    int bar = input.lastIndexOf('|');
    String probe = input.substring(bar + 1);
    WildcardIndex<String> index = WildcardIndex$.MODULE$.apply(WildcardTree$.MODULE$.apply(), WildcardTree$.MODULE$.apply());
    index = index.insert(new String[] {"q"}, "anchor");
    for (String prefix : input.substring(0, bar).split(",")) {
      index = index.insert(new String[] {prefix + "*", "x"}, prefix);
    }
    scala.Option<String> found = index.find(scala.jdk.javaapi.CollectionConverters.asScala(Arrays.<String>asList(probe, "x")));
    return found.isDefined() ? found.get() : "none";
  }
}
