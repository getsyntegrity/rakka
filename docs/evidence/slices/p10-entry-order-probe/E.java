import com.typesafe.config.*;
import java.nio.file.*; import java.util.*;
public class E { public static void main(String[] a) throws Exception {
  for (String line : Files.readAllLines(Paths.get(a[0]))) {
    Config c = ConfigFactory.parseString(line);
    StringBuilder sb = new StringBuilder();
    for (Map.Entry<String,ConfigValue> e : c.root().entrySet()) sb.append(e.getKey()).append(' ');
    System.out.println(sb.toString().trim());
  } } }
