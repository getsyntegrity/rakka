import com.typesafe.config.*;
import java.util.*;
import org.apache.pekko.actor.*;

public class Q36ValueDependence {
  static scala.collection.Iterable<String> it(String... a) { return scala.jdk.javaapi.CollectionConverters.asScala((java.lang.Iterable<String>) Arrays.asList(a)); }
  static String run(String[] keys, String[] values) {
    StringBuilder b = new StringBuilder("pekko.actor.deployment {\n");
    for (int i = 0; i < keys.length; i++) b.append("\"" + keys[i] + "\" { dispatcher = " + values[i] + " }\n");
    Config c = ConfigFactory.parseString(b + "}").withFallback(ConfigFactory.defaultReference()).resolve();
    ClassLoader l = Q36ValueDependence.class.getClassLoader();
    Deployer d = new Deployer(new ActorSystem.Settings(l, c, "x"), new ReflectiveDynamicAccess(l));
    return d.lookup(it("svc")).get().path();
  }
  public static void main(String[] a) {
    String[] keys = {"/svc", "/svc/", "/svc//"};
    String[][] perms = {{"d1","d2","d3"},{"d1","d3","d2"},{"d2","d1","d3"},{"d2","d3","d1"},{"d3","d1","d2"},{"d3","d2","d1"},{"aaa","bbb","ccc"},{"x","y","z"}};
    for (String[] p : perms) System.out.println(Arrays.toString(p) + " -> winner " + run(keys, p));
  }
}
