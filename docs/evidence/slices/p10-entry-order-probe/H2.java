import com.typesafe.config.*;
public class H2 { public static void main(String[] a){
 String[] c={"v={}","v={a=1}","v={a=2}","v={a=3}","v={b=1}","v={c=1}","v={a=1,b=1}","v={a=1,b=2}","v={a=2,b=1}","v={a=1,b=1,c=1}","v={a=0}"};
 for(String s:c) System.out.println(s+" "+ConfigFactory.parseString(s).root().get("v").hashCode());}}
