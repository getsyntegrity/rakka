import com.typesafe.config.*;
import java.util.*;
public class H {
  public static void main(String[] a) {
    String[] cfgs = {"v=1","v=0","v=-1","v=2147483648","v=4294967296","v=1.0","v=1.5","v=0.1","v=-0.0","v=1e5","v=1e30","v=true","v=false","v=null","v=\"\"","v=a","v=\"é\"","v=[]","v=[1]","v=[1,2]","v=[2,1]","v={}","v={a=1}","v={a=1,b=2}","v={b=2,a=1}","v={a={b=1}}","v=[[1],{a=2}]","v=\"1\"","v=12345678901234567890","v=3000000000","v=1.0E10"};
    for (String c : cfgs) {
      Config cf = ConfigFactory.parseString(c);
      ConfigValue v = cf.root().get("v");
      System.out.println(c + " -> " + v.hashCode() + " " + v.getClass().getSimpleName() + " \"a\".hash=" + "a".hashCode());
    }
  }
}
