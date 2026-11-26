import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import org.apache.pekko.actor.ActorSystem;

import java.util.concurrent.TimeUnit;

/** Focused probe: what can be observed of `whenTerminated` when the guardians get a default mailbox that cannot satisfy their Unbounded requirement.
 *  Run per configuration; every outcome is printed as one line, nothing is assumed. */
public class RootGuardianProbe {
    static String run(String label, String extra) {
        Config config = ConfigFactory.parseString(extra).withFallback(ConfigFactory.load());
        long start = System.nanoTime();
        ActorSystem system = null;
        try {
            system = ActorSystem.create("probe", config);
        } catch (Throwable t) {
            Throwable root = t;
            while (root.getCause() != null && root.getCause() != root) root = root.getCause();
            return label + ": ActorSystem.create THREW " + t.getClass().getName() + " after " + (System.nanoTime() - start) / 1_000_000 + " ms; message=" + String.valueOf(t.getMessage()).replace('\n', ' ') + "; root cause=" + root.getClass().getName() + ": " + String.valueOf(root.getMessage()).replace('\n', ' ');
        }
        StringBuilder out = new StringBuilder(label + ": ActorSystem.create RETURNED after " + (System.nanoTime() - start) / 1_000_000 + " ms; ");
        try {
            scala.concurrent.Future<?> terminated = system.whenTerminated();
            Thread.sleep(3000);
            out.append("whenTerminated.isCompleted after 3 s = ").append(terminated.isCompleted());
            if (terminated.isCompleted()) out.append(" value=").append(terminated.value());
        } catch (Throwable t) {
            out.append("observing whenTerminated THREW ").append(t);
        } finally {
            try { system.terminate(); system.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS); } catch (Throwable ignored) { }
        }
        return out.toString();
    }

    public static void main(String[] args) {
        String bounded = "pekko.actor.default-mailbox.mailbox-type = \"org.apache.pekko.dispatch.BoundedMailbox\"\n"
                + "pekko.actor.default-mailbox.mailbox-capacity = 1\npekko.actor.default-mailbox.mailbox-push-timeout-time = 1s\n";
        String noLoggers = "pekko.loggers = []\npekko.use-slf4j = off\npekko.stdout-loglevel = \"OFF\"\n";
        System.out.println(run("A baseline, loggers off", noLoggers));
        System.out.println(run("B bounded default mailbox, loggers off", bounded + noLoggers));
        
        System.exit(0);
    }
}
