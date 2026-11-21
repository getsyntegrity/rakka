package pilot;

import com.typesafe.config.ConfigFactory;
import org.apache.pekko.actor.Deploy;
import org.apache.pekko.actor.NoScopeGiven$;
import org.apache.pekko.routing.NoRouter$;

/**
 * Differential-fuzz oracle for the concise render of an unresolved configuration (D2-D4 of PR-C): one input line is the text
 * of a configuration; the output line is {@code Deploy.toString} of a deployment holding it (non-ASCII escaped), or
 * {@code ERR <simple class name>} when Typesafe Config rejects it.
 */
final class ConfigRenderOracle {
  private ConfigRenderOracle() {}

  static String line(String text) {
    try {
      Deploy deploy = new Deploy("/a", ConfigFactory.parseString(text), NoRouter$.MODULE$, NoScopeGiven$.MODULE$, "", "",
          scala.collection.immutable.Set$.MODULE$.<String>empty());
      return UtilCoreOracle.esc(deploy.toString());
    } catch (Throwable t) {
      return "ERR " + t.getClass().getSimpleName();
    }
  }
}
