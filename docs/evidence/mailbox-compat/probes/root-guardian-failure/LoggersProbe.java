import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import org.apache.pekko.actor.ActorSystem;

public class LoggersProbe {
    static void show(String label, String extra) {
        Config config = ConfigFactory.parseString(extra).withFallback(ConfigFactory.load());
        ActorSystem system = ActorSystem.create("loggers-probe", config);
        System.out.println(label + ": the system's own pekko.loggers = " + system.settings().config().getStringList("pekko.loggers")
                + " ; Loggers() = " + system.settings().Loggers() + " ; whenTerminated completed = " + system.whenTerminated().isCompleted());
        system.terminate();
    }

    public static void main(String[] args) throws Exception {
        show("baseline", "pekko.stdout-loglevel = \"OFF\"\n");
        show("pekko.loggers = []", "pekko.loggers = []\npekko.stdout-loglevel = \"OFF\"\n");
        System.exit(0);
    }
}
