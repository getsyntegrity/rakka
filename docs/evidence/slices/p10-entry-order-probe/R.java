import com.typesafe.config.*; import java.util.*;
public class R { public static void main(String[] a){ ConfigObject o=ConfigFactory.defaultReference().getConfig("pekko.actor.deployment").root(); System.out.println(String.join("\t", o.keySet())); } }
