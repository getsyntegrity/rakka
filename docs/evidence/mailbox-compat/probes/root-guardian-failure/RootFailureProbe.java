import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import org.apache.pekko.actor.ActorSystem;

import java.util.concurrent.TimeUnit;

public class RootFailureProbe {
    public static void main(String[] args) throws Exception {
        String extra = "pekko.loggers = []\npekko.use-slf4j = off\npekko.stdout-loglevel = \"OFF\"\npekko.actor.default-mailbox.mailbox-type = \"FirstBoundedMailbox\"\n";
        Config config = ConfigFactory.parseString(extra).withFallback(ConfigFactory.load());
        long start = System.nanoTime();
        final ActorSystem[] holder = new ActorSystem[1];
        final Throwable[] failure = new Throwable[1];
        Thread creator = new Thread(() -> {
            try { holder[0] = ActorSystem.create("root-failure-probe", config); } catch (Throwable t) { failure[0] = t; }
        });
        creator.setDaemon(true);
        creator.start();
        creator.join(20_000);
        long millis = (System.nanoTime() - start) / 1_000_000;
        if (creator.isAlive()) {
            System.out.println("RESULT: ActorSystem.create is STILL RUNNING after " + millis + " ms (it did not return and did not throw)");
        } else if (failure[0] != null) {
            Throwable root = failure[0];
            while (root.getCause() != null && root.getCause() != root) root = root.getCause();
            System.out.println("RESULT: ActorSystem.create THREW after " + millis + " ms: " + failure[0].getClass().getName() + ": " + String.valueOf(failure[0].getMessage()).replace('\n', ' '));
            System.out.println("        root cause: " + root.getClass().getName() + ": " + String.valueOf(root.getMessage()).replace('\n', ' '));
            ActorSystem captured = FirstBoundedMailbox.CAPTURED;
            if (captured == null) {
                System.out.println("OBSERVATION: no ActorSystem handle was captured");
            } else {
                for (int seconds : new int[] {1, 3, 6}) {
                    Thread.sleep(seconds == 1 ? 1000 : seconds == 3 ? 2000 : 3000);
                    scala.concurrent.Future<?> terminated = captured.whenTerminated();
                    System.out.println("OBSERVATION after ~" + seconds + " s: captured system whenTerminated.isCompleted = " + terminated.isCompleted() + (terminated.isCompleted() ? " value=" + terminated.value() : ""));
                }
                System.out.println("OBSERVATION: captured system name=" + captured.name() + " ; guardian children created before the failure=" + FirstBoundedMailbox.CREATED.get());
            }
        } else {
            ActorSystem system = holder[0];
            Thread.sleep(3000);
            System.out.println("RESULT: ActorSystem.create RETURNED after " + millis + " ms; whenTerminated.isCompleted after 3 s = " + system.whenTerminated().isCompleted()
                    + (system.whenTerminated().isCompleted() ? " value=" + system.whenTerminated().value() : ""));
            try { system.terminate(); system.getWhenTerminated().toCompletableFuture().get(5, TimeUnit.SECONDS); System.out.println("        after terminate(): completed"); }
            catch (Throwable t) { System.out.println("        after terminate(): " + t.getClass().getSimpleName() + " (whenTerminated did not complete in 5 s)"); }
        }
        System.exit(0);
    }
}
