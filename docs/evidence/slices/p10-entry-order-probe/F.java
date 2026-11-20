import com.typesafe.config.*;
import java.nio.file.*; import java.util.*;
public class F { public static void main(String[] a) throws Exception {
  Config ref = ConfigFactory.defaultReference();
  for (String line : Files.readAllLines(Paths.get(a[0]))) {
    Config c = ConfigFactory.parseString("pekko.actor.deployment { " + line + " }").withFallback(ref).resolve();
    ConfigObject d = c.getConfig("pekko.actor.deployment").root();
    StringBuilder sb = new StringBuilder();
    for (Map.Entry<String,ConfigValue> e : d.entrySet()) sb.append(e.getKey()).append('\t').append(e.getValue().hashCode()).append(';');
    System.out.println(sb);
    // doc-order keys
  } } }
