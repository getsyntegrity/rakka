import java.util.*;
public class Gen { public static void main(String[] a){ Random r=new Random(42); StringBuilder sb=new StringBuilder();
 for(int i=0;i<400000;i++){ double d; int m=i%4;
  if(m==0) d=Double.longBitsToDouble(r.nextLong()); else if(m==1) d=r.nextDouble()*Math.pow(10,r.nextInt(40)-20); else if(m==2) d=Math.round(r.nextDouble()*1e6)/Math.pow(10,r.nextInt(12)); else d=Double.longBitsToDouble(r.nextLong()&0x800fffffffffffffL|((long)(r.nextInt(30))<<52));
  if(Double.isNaN(d)||Double.isInfinite(d)) continue; sb.append(Double.doubleToLongBits(d)).append(' ').append(Double.toString(d)).append('\n'); }
 System.out.print(sb);}}
