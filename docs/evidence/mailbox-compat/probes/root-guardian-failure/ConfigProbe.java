import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;

public class ConfigProbe {
    public static void main(String[] args) {
        Config base = ConfigFactory.load();
        System.out.println("default pekko.loggers = " + base.getStringList("pekko.loggers") + " | origin: " + base.getValue("pekko.loggers").origin().description());
        Config overridden = ConfigFactory.parseString("pekko.loggers = []\n").withFallback(base);
        System.out.println("after the override    = " + overridden.getStringList("pekko.loggers"));
        System.out.println("logger-startup-timeout = " + overridden.getString("pekko.logger-startup-timeout") + " ; pekko.loglevel = " + overridden.getString("pekko.loglevel"));
        System.out.println("default-mailbox = " + overridden.getString("pekko.actor.default-mailbox.mailbox-type"));
    }
}
