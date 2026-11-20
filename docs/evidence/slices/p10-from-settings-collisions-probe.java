import com.typesafe.config.*;
import java.util.*;
import org.apache.pekko.actor.*;

public class Q36Collisions {
  static scala.collection.Iterable<String> it(String... a) { return scala.jdk.javaapi.CollectionConverters.asScala((java.lang.Iterable<String>) Arrays.asList(a)); }
  public static void main(String[] x) throws Exception {
    String[] names = {"a","svc","worker","x1","long-name-here","k","name","q","p"};
    String[] suffixes = {"","/","//","///"};
    int total=0, differ=0, textDependent=0;
    List<String> examples = new ArrayList<>();
    for (String n : names) {
      for (int mask = 3; mask < 16; mask++) {
        List<String> keys = new ArrayList<>();
        for (int i = 0; i < 4; i++) if ((mask & (1<<i)) != 0) keys.add("/" + n + suffixes[i]);
        if (keys.size() < 2) continue;
        String[] res = new String[2];
        for (int rev = 0; rev < 2; rev++) {
          List<String> order = new ArrayList<>(keys); if (rev == 1) Collections.reverse(order);
          StringBuilder b = new StringBuilder("pekko.actor.deployment {\n");
          for (String k : order) b.append("\"" + k + "\" { dispatcher = \"" + k + "\" }\n");
          b.append("}\n");
          Config c = ConfigFactory.parseString(b.toString()).withFallback(ConfigFactory.defaultReference()).resolve();
          ClassLoader l = Q36Collisions.class.getClassLoader();
          Deployer d = new Deployer(new ActorSystem.Settings(l, c, "x"), new ReflectiveDynamicAccess(l));
          res[rev] = d.lookup(it(n)).get().path();
        }
        List<String> sorted = new ArrayList<>(keys); Collections.sort(sorted);
        String rustSortedWinner = sorted.get(sorted.size()-1);
        total++;
        if (!res[0].equals(res[1])) textDependent++;
        if (!res[0].equals(rustSortedWinner)) { differ++; if (examples.size() < 6) examples.add(keys + " java=" + res[0] + " sortedLast=" + rustSortedWinner); }
      }
    }
    System.out.println("conflict sets=" + total + " java winner depends on text order=" + textDependent + " java winner != last in sorted order=" + differ);
    examples.forEach(System.out::println);
  }
}
