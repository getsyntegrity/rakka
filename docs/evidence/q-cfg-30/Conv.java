import java.io.*;
public class Conv { public static void main(String[] a) throws Exception {
  BufferedReader r = new BufferedReader(new InputStreamReader(System.in)); StringBuilder sb = new StringBuilder(); String l;
  while ((l = r.readLine()) != null) { String b = l.split(" ")[0]; double d = Double.longBitsToDouble(Long.parseLong(b)); sb.append(b).append(' ').append(Double.toString(d)).append('\n'); }
  System.out.print(sb); } }
