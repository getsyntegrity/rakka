package pilot;

import org.apache.pekko.actor.*;
import java.util.*;
import java.util.concurrent.Callable;

/** Oracle cases ported from the deprecated jshell script (Q-X-12); registered in order, run by id. */
final class PathAddressOracle {
  private PathAddressOracle() {}

  static final java.util.Map<String, Callable<String>> CASES = new java.util.LinkedHashMap<>();

  static void run(String id, Callable<String> f) {
    if (CASES.put(esc(id), f) != null) throw new IllegalStateException("duplicate case id " + id);
  }

  /** Case ids in registration order (ASCII-escaped). */
  static java.util.List<String> ids() {
    return new java.util.ArrayList<>(CASES.keySet());
  }

  /** The oracle line of one case: {@code <id> -> ok <fields>} or {@code <id> -> ERR <Exception> [<message>]}. */
  static String line(String escapedId) {
    Callable<String> f = CASES.get(escapedId);
    if (f == null) throw new IllegalArgumentException("unknown case id " + escapedId);
    String r;
    try {
      r = "ok " + f.call();
    } catch (Throwable t) {
      r = "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]";
    }
    return esc(unesc(escapedId) + " -> " + r);
  }


  static String esc(String s) {
    StringBuilder b = new StringBuilder();
    for (char c : s.toCharArray()) {
      if (c < 0x20 || c > 0x7e) b.append(String.format("\\u%04X", (int) c));
      else b.append(c);
    }
    return b.toString();
  }

  static String unesc(String s) {
    StringBuilder b = new StringBuilder();
    for (int i = 0; i < s.length(); i++) {
      if (s.startsWith("\\u", i) && i + 6 <= s.length() && s.substring(i + 2, i + 6).matches("[0-9A-F]{4}")) {
        b.append((char) Integer.parseInt(s.substring(i + 2, i + 6), 16));
        i += 5;
      } else {
        b.append(s.charAt(i));
      }
    }
    return b.toString();
  }

  static String list(Iterable<String> it) {
    StringBuilder b = new StringBuilder("(");
    boolean first = true;
    for (String s : it) { if (!first) b.append(","); b.append("[").append(s).append("]"); first = false; }
    return b.append(")").toString();
  }

  static String opt(scala.Option<?> o) { return o.isDefined() ? "[" + o.get() + "]" : "none"; }

  static String addr(Address a) {
    return "str=[" + a + "] protocol=[" + a.protocol() + "] system=[" + a.system() + "] host=" + opt(a.host())
        + " port=" + opt(a.port()) + " local=" + a.hasLocalScope() + " global=" + a.hasGlobalScope()
        + " hostPort=[" + a.hostPort() + "]";
  }

  static String path(ActorPath p) {
    return p.getClass().getSimpleName() + " str=[" + p + "] ser=[" + p.toSerializationFormat() + "] noAddr=["
        + p.toStringWithoutAddress() + "] addr=[" + p.address() + "] name=[" + p.name() + "] uid=" + p.uid()
        + " elems=" + list(p.getElements());
  }

  static int sig(int x) { return Integer.signum(x); }

  static Address A(String p, String s) { return new Address(p, s); }

  static Address A(String p, String s, String h, int port) { return new Address(p, s, h, port); }

  static scala.Option<String> someS(String s) { return new scala.Some<String>(s); }

  static scala.Option<Object> someI(int i) { return new scala.Some<Object>(Integer.valueOf(i)); }

  static Address copyOf(Address a, scala.Option<String> h, scala.Option<Object> p) { return a.copy(a.protocol(), a.system(), h, p); }

  static ActorPath root(Address a) { return RootActorPath$.MODULE$.apply(a, "/"); }

  static ActorPath fs(String s) { return ActorPath$.MODULE$.fromString(s); }

  static int cmpA(Address a, Address b) { return sig(Address$.MODULE$.addressOrdering().compare(a, b)); }

  static String pairA(Address a, Address b) {
    boolean eq = a.equals(b);
    return "cmp=" + cmpA(a, b) + " cmpRev=" + cmpA(b, a) + " eq=" + eq + (eq ? " hashEq=" + (a.hashCode() == b.hashCode()) : "");
  }

  static String pairP(ActorPath a, ActorPath b) {
    boolean eq = a.equals(b);
    return "cmp=" + sig(a.compareTo(b)) + " cmpRev=" + sig(b.compareTo(a)) + " eq=" + eq + " eqRev=" + b.equals(a)
        + (eq ? " hashEq=" + (a.hashCode() == b.hashCode()) : "");
  }

  static void val(String id, String e, String full) {
    run(id, () -> {
      boolean valid = ActorPath$.MODULE$.isValidPathElement(e);
      try {
        if (full == null) ActorPath$.MODULE$.validatePathElement(e); else ActorPath$.MODULE$.validatePathElement(e, full);
        return "valid=" + valid + " validate=ok";
      } catch (InvalidActorNameException x) {
        return "valid=" + valid + " validate=ERR " + x.getClass().getSimpleName() + " [" + x.getMessage() + "]";
      }
    });
  }

  static Address mysys = A("pekko", "mysys");

  static Address aA = copyOf(mysys, someS("aaa"), someI(7355));

  static Address aB = copyOf(aA, someS("bb"), someI(7355));

  static Address aC = copyOf(aA, someS("cccc"), someI(7355));

  static String rel(String s) { scala.Option<scala.collection.immutable.Seq<String>> o = RelativeActorPath$.MODULE$.unapply(s); return o.isDefined() ? "some " + list(scala.jdk.javaapi.CollectionConverters.asJava(o.get())) : "none"; }

  static String[] larp = {"$hallo", "a%", "%3", "%xx", "%0G", "%gg", "%", "%1t", "a?", "\u00fc\u00df", "he\"llo", "$hello", "hell>o"};

  static String[] fsIn = {
    "pekko://mysys/user", "pekko://sys", "pekko://sys/", "pekko://sys/user/", "pekko://sys/user//x", "pekko://sys//a",
    "pekko://sys/./a/../b", "pekko://sys/$a", "pekko://sys/user/a%20b", "pekko://sys/a%2Fb", "pekko://sys/a%2",
    "pekko://sys/a%zz", "pekko://sys/a b", "pekko://sys/\u00fcser", "pekko://sys/user?x=1", "pekko://sys/user?x#5",
    "pekko://sys/user#5?x", "pekko://sys@in_valid:5001/user", "pekko://my_sys/user", "pekko://my_sys@host:1/user",
    "pekko:sys", "pekko:sys/user", "pekko:///user", "pekko://@host:1/user", "pekko://sys:1234/user",
    "pekko://sys@host/user", "pekko://sys@[::1]:2552/user", "pekko://sys@127.0.0.1:2552/user", "pekko://a%20b@h:1/u",
    "pekko://a:b@h:1/u", "PEKKO://Sys@HOST:1/User", "/user/a", "user/a", "pekko://sys@h:70000/u",
    "pekko://sys@h:99999999999/u",
    // uid in the fragment (ActorCell.scala:400-404 Integer.valueOf)
    "pekko://sys/a#5", "pekko://sys/a#+5", "pekko://sys/a#-5", "pekko://sys/a#0", "pekko://sys/a#abc",
    "pekko://sys/a#99999999999", "pekko://sys/a#2147483647", "pekko://sys/a#2147483648", "pekko://sys/a#-2147483648",
    "pekko://sys/a#", "pekko://sys/a#5#6", "pekko://sys/a# 5", "pekko://sys#5", "pekko://sys/#5", "pekko://sys/user/#5",
    "pekko://sys/a/b#7", "pekko://sys/a#\u0665", "pekko://sys/a#\uff15", "pekko://sys/a#+", "pekko://sys/a#-", "pekko://sys/a#+-5"};

  static String[] adIn = {
    "pekko://my_sys@host:1234", "pekko://sys", "pekko://sys/", "pekko://sys/user/a", "pekko://sys?x", "pekko://sys#5",
    "pekko://sys@h:1/user#5", "pekko://sys@in_valid:5001", "pekko://sys:1234", "pekko://@host:1", "pekko://sys@host",
    "pekko://sys@host:", "pekko://sys@host:0", "pekko://sys@host:00012", "pekko://sys@host:-1", "pekko://sys@host:70000",
    "pekko://sys@host:2147483647", "pekko://sys@host:2147483648", "pekko://sys@host:99999999999", "pekko://sys@[::1]:2552",
    "pekko://sys@[::1]", "pekko://sys@[::1", "pekko://sys@127.0.0.1:2552", "pekko://sys@256.1.1.1:1", "pekko://my_sys",
    "pekko://my_sys@host:1", "pekko://a:b@h:1", "pekko://a%20b@h:1", "pekko://a%2@h:1", "pekko://sys@h\u00f6:1",
    "pekko://s\u00ffs@h:1", "pekko://s\u00ffs", "pekko:sys", "pekko:///", "pekko://", "PEKKO://Sys@HOST:1", "pekko://s y",
    "/user", "sys@h:1", "1pekko://sys", "pekko://sys@h-1.example.com:1", "pekko://sys@-h:1", "pekko://sys@h.:1",
    "pekko://sys@h_1.example.com:1", "pekko://sys@host:1234 ", "", "://hallo", "pekko://a%C3%BC@h:1", "pekko://a%FF@h:1",
    "pekko://sys@[::FFFF:1.2.3.4]:1", "pekko://sys@[fe80::1%25eth0]:1", "pekko://sys@[1::2::3]:1", "pekko://sys@::1:1"};

  static String[] utfIn = {"%ED%A0%80", "%ED%B0%80", "%ED%A0%80%ED%B0%80", "%ED%A0%41", "%ED%A0", "%C0%80", "%C1%BF", "%E0%80%80",
    "%E0%9F%BF", "%E0%80", "%F0%80%80%80", "%F0%8F%BF%BF", "%F4%90%80%80", "%F5%80%80%80", "%E2%82", "%E2%82%41",
    "%F0%9F%98", "%F0%9F%41", "%F0%90%80%41", "%80", "%BF%80", "%C3", "%C3%BCx%FF%E2%82%AC", "%E2%82%AC%ED%A0%E2%82%AC",
    "%F8%88%80%80%80", "%FC%84%80%80%80%80", "%FE", "%FF", "%C2%41", "%F0%9F%98%80%80"};

  static String[] utfFs = {"pekko://a%ED%A0%80@h:1/u%ED%A0%80/v%C0%80#5", "pekko://a%ED%A0%80/u", "pekko://a%E2%82@h%E2:1/u",
    "pekko://a%FF@h:1/u#%FF"};

  static String[] rpIn = {"", "/user/a", "pekko://sys/a", "foo:bar", "a%2", "a b", "foo?x", "foo#abc", "#5", "../a", "foo//bar",
    "foo/", "\u00fc", "a%20b", "foo#1#2"};

  static ActorPath c7 = root(mysys).$div("user").$div("foo#7");

  static Address other = A("pekko", "other");

  static ActorPath ign = IgnoreActorRef$.MODULE$.path();

  static Address hNoneP1 = copyOf(mysys, scala.Option.empty(), someI(1));

  static Address hEmptyP2 = copyOf(mysys, someS(""), someI(2));

  static Address hNoneP2 = copyOf(mysys, scala.Option.empty(), someI(2));

  static ActorPath r0 = root(mysys);

  static String[] valIn = {"", "abc", "ABCxyz019", "$abc", "$", "a$b", "a$", "a/b", "a#b", "a%2Fb", "%41", "A%aF", "a%2", "a%",
    "%", "%g1", "a%%41", "a b", " ", "\t", "\u00fc", "\ud83d\ude42", "x\ud83d\ude42", "-_.*$+:@&=,!~';", "-", ".", "..",
    "a\"b", "a?b", "a\\b", "a(b", "a[b"};

  static {
    // Oracle for unit AC-PATH: prints how Pekko 2.0.0 parses, builds, renders,
    // compares and validates Address / ActorPath values.
    // Run: oracle/run.sh path-address-all
    //
    // Output contract (one deterministic line per case):
    //   <CASE-ID> -> ok <fields>
    //   <CASE-ID> -> ERR <ExceptionSimpleName> [<message verbatim>]
    // Every char outside printable ASCII is written as \\uXXXX (UTF-16 code unit),
    // so trailing spaces stay visible inside [ ] and the file is pure ASCII.
    // No identity hashes, no timestamps; hashCode values are never printed, only
    // "hashEq" for pairs that Pekko considers equal (equal => same hash is the
    // contract; the hash value itself is not).
    // Case-id prefixes: SPEC-* (assertion of a Pekko spec, file:line in the id
    // comment), FS (ActorPath.fromString), AD (AddressFromURIString), RP
    // (RelativeActorPath), CT (Address constructors/copy), HC (host characters),
    // RT (RootActorPath constructor), CH (child "/"), DS (descendant), UID
    // (withUid), NAV (name/parent/root/elements), STR (toString variants), VAL
    // (validatePathElement/isValidPathElement), CMPA/CMPP (Address ordering,
    // ActorPath compareTo + equals/hash), UTF-AD/UTF-FS (malformed UTF-8
    // escapes in the user-info), DEEP (1,000,000-level path).
    
    
    
    // ---------------------------------------------------------------- SPEC: ActorPathSpec.scala (actor-tests/.../actor/ActorPathSpec.scala)
    // :25-28 support parsing its String rep
    run("SPEC-APS-27", () -> { ActorPath p = root(mysys).$div("user"); return "eq=" + fs(p.toString()).equals(p) + " " + path(fs(p.toString())); });
    // :30-38 remote pekko / akka paths
    run("SPEC-APS-32", () -> path(fs("pekko://my_sys@host:1234/some/ref")));
    run("SPEC-APS-37", () -> path(fs("akka://my_sys@host:1234/some/ref")));
    // :40-48 AddressFromURIString
    run("SPEC-APS-42", () -> addr(AddressFromURIString$.MODULE$.apply("pekko://my_sys@host:1234")) + " eqExpected=" + AddressFromURIString$.MODULE$.apply("pekko://my_sys@host:1234").equals(A("pekko", "my_sys", "host", 1234)));
    run("SPEC-APS-47", () -> addr(AddressFromURIString$.MODULE$.apply("akka://my_sys@host:1234")) + " eqExpected=" + AddressFromURIString$.MODULE$.apply("akka://my_sys@host:1234").equals(A("akka", "my_sys", "host", 1234)));
    // :50-56 malformed paths
    run("SPEC-APS-51", () -> path(fs("")));
    run("SPEC-APS-52", () -> path(fs("://hallo")));
    run("SPEC-APS-53", () -> path(fs("s://dd@:12")));
    run("SPEC-APS-54", () -> path(fs("s://dd@h:hd")));
    run("SPEC-APS-55", () -> path(fs("a://l:1/b")));
    // :58-64 toString
    run("SPEC-APS-60", () -> root(mysys).toString());
    run("SPEC-APS-61", () -> root(mysys).$div("user").toString());
    run("SPEC-APS-62", () -> root(mysys).$div("user").$div("foo").toString());
    run("SPEC-APS-63", () -> root(mysys).$div("user").$div("foo").$div("bar").toString());
    // :66-69 elements
    run("SPEC-APS-67", () -> list(root(mysys).$div("user").$div("foo").$div("bar").getElements()));
    // :71-77 toStringWithoutAddress
    run("SPEC-APS-73", () -> root(mysys).toStringWithoutAddress());
    run("SPEC-APS-74", () -> root(mysys).$div("user").toStringWithoutAddress());
    run("SPEC-APS-75", () -> root(mysys).$div("user").$div("foo").toStringWithoutAddress());
    run("SPEC-APS-76", () -> root(mysys).$div("user").$div("foo").$div("bar").toStringWithoutAddress());
    // :79-82 validate path elements
    val("SPEC-APS-80", "", null);
    // :84-106 toStringWithAddress
    run("SPEC-APS-86", () -> addr(aA) + " | " + addr(aB) + " | " + addr(aC));
    run("SPEC-APS-90", () -> root(mysys).toStringWithAddress(aA));
    run("SPEC-APS-91", () -> root(mysys).$div("user").toStringWithAddress(aA));
    run("SPEC-APS-92", () -> root(mysys).$div("user").$div("foo").toStringWithAddress(aA));
    run("SPEC-APS-94", () -> root(mysys).toStringWithAddress(aB)); // commented-out assertion in the spec; printed for completeness
    run("SPEC-APS-95", () -> root(mysys).$div("user").toStringWithAddress(aB));
    run("SPEC-APS-96", () -> root(mysys).$div("user").$div("foo").toStringWithAddress(aB));
    run("SPEC-APS-98", () -> root(mysys).toStringWithAddress(aC));
    run("SPEC-APS-99", () -> root(mysys).$div("user").toStringWithAddress(aC));
    run("SPEC-APS-100", () -> root(mysys).$div("user").$div("foo").toStringWithAddress(aC));
    run("SPEC-APS-103", () -> root(aA).toStringWithAddress(aB));
    run("SPEC-APS-104", () -> root(aA).$div("user").toStringWithAddress(aB));
    run("SPEC-APS-105", () -> root(aA).$div("user").$div("foo").toStringWithAddress(aB));
    // :108-115 no path separators in RootActorPath name
    run("SPEC-APS-110", () -> path(RootActorPath$.MODULE$.apply(mysys, "/user/boom/*")));
    run("SPEC-APS-114", () -> path(fs("pekko://mysys/user/boom/*")));
    // :117-127 hasInvalidHostCharacters
    for (String h : new String[] {"valid", "is_valid.org", "fu.is_valid.org", "in_valid", "invalid._org"})
      run("SPEC-APS-119-124[" + h + "]", () -> "invalid=" + A("pekko", "sys", h, 0).hasInvalidHostCharacters());
    run("SPEC-APS-126", () -> addr(AddressFromURIString$.MODULE$.apply("pekko://sys@in_valid:5001")));
    // :129-144 checkHostCharacters
    for (String h : new String[] {"localhost", "is_valid.org", "fu.is_valid.org", "in_valid", "invalid._org"})
      run("SPEC-APS-133-143[" + h + "]", () -> { A("pekko", "sys", h, 0).checkHostCharacters(); return "checked"; });
    
    // ---------------------------------------------------------------- SPEC: AddressSpec.scala:25-35
    run("SPEC-ADS-34", () -> {
      List<Address> l = new ArrayList<>(List.of(A("pekko", "system", "host1", 1000), A("pekko", "system", "host2", 1000),
          A("akka", "system", "host1", 1000), A("akka", "system", "host2", 1000)));
      l.sort(Address$.MODULE$.addressOrdering());
      return l.toString();
    });
    
    // ---------------------------------------------------------------- SPEC: RelativeActorPathSpec.scala:28-40
    run("SPEC-RAP-29", () -> rel("foo"));
    run("SPEC-RAP-32", () -> rel("foo/bar/baz"));
    run("SPEC-RAP-35", () -> rel(java.net.URLEncoder.encode("pekko://ClusterSystem@127.0.0.1:7355", "UTF-8")));
    run("SPEC-RAP-39", () -> rel("foo/bar/baz#1234"));
    
    // ---------------------------------------------------------------- SPEC: LocalActorRefProviderSpec.scala:168-197
    // actorOf(null|"") messages come from dungeon/Children.scala:269-276 (AC-CELL), not from AC-PATH; only the
    // names that reach ActorPath.validatePathElement are covered here.
    for (int i = 0; i < larp.length; i++) val("SPEC-LARP-" + String.format("%02d", i + 1), larp[i], null);
    
    // ---------------------------------------------------------------- SPEC: JavaAPI.java:62 (ActorPaths facade)
    run("SPEC-JAPI-62", () -> path(ActorPaths.fromString("pekko://Sys@localhost:1234/user/abc")));
    
    // ---------------------------------------------------------------- FS: ActorPath.fromString (parser domain)
    for (int i = 0; i < fsIn.length; i++) { String s = fsIn[i]; run(String.format("FS-%02d [", i + 1) + s + "]", () -> path(fs(s))); }
    
    // ---------------------------------------------------------------- AD: AddressFromURIString.apply / parse (parser domain)
    for (int i = 0; i < adIn.length; i++) { String s = adIn[i]; run(String.format("AD-%02d [", i + 1) + s + "]", () -> addr(AddressFromURIString$.MODULE$.apply(s))); }
    run("AD-PARSE [pekko://sys@h:1]", () -> addr(AddressFromURIString$.MODULE$.parse("pekko://sys@h:1")));
    run("AD-PARSE [pekko://sys:1]", () -> addr(AddressFromURIString$.MODULE$.parse("pekko://sys:1")));
    
    // ---------------------------------------------------------------- UTF: malformed UTF-8 escapes. Only the user-info (system
    // name) is decoded (URI.getUserInfo -> URI.decode, CharsetDecoder UTF-8 with REPLACE); host, path and fragment stay raw.
    for (int i = 0; i < utfIn.length; i++) { String s = "pekko://a" + utfIn[i] + "b@h:1"; run(String.format("UTF-AD-%02d [", i + 1) + s + "]", () -> addr(AddressFromURIString$.MODULE$.apply(s))); }
    for (int i = 0; i < utfFs.length; i++) { String s = utfFs[i]; run(String.format("UTF-FS-%02d [", i + 1) + s + "]", () -> path(fs(s))); }
    
    // ---------------------------------------------------------------- DEEP: 1,000,000-level path (no recursion on build/render/eq/drop)
    run("DEEP-01 fromString 1e6 levels vs built", () -> {
      ActorPath f = fs("pekko://s/" + "a/".repeat(1_000_000));
      ActorPath b = root(A("pekko", "s"));
      for (int i = 0; i < 1_000_000; i++) b = b.$div("a");
      return "elems=" + f.elements().size() + " strLen=" + f.toString().length() + " eq=" + f.equals(b)
          + " hashEq=" + (f.hashCode() == b.hashCode()) + " cmp=" + sig(f.compareTo(b)) + " vsParent=" + sig(f.compareTo(f.parent()))
          + " eqParent=" + f.equals(f.parent());
    });
    
    // ---------------------------------------------------------------- RP: RelativeActorPath.unapply
    for (int i = 0; i < rpIn.length; i++) { String s = rpIn[i]; run(String.format("RP-%02d [", i + 1) + s + "]", () -> rel(s)); }
    
    // ---------------------------------------------------------------- CT: Address constructor domain (any Int, null host)
    run("CT-01 new(pekko,sys)", () -> addr(A("pekko", "sys")));
    run("CT-02 new(pekko,sys,h,1234)", () -> addr(A("pekko", "sys", "h", 1234)));
    run("CT-03 new(pekko,sys,h,0)", () -> addr(A("pekko", "sys", "h", 0)));
    run("CT-04 new(pekko,sys,h,-5)", () -> addr(A("pekko", "sys", "h", -5)));
    run("CT-05 new(pekko,sys,h,-1)", () -> addr(A("pekko", "sys", "h", -1)));
    run("CT-06 new(pekko,sys,h,70000)", () -> addr(A("pekko", "sys", "h", 70000)));
    run("CT-07 new(pekko,sys,h,MAX)", () -> addr(A("pekko", "sys", "h", Integer.MAX_VALUE)));
    run("CT-08 new(pekko,sys,h,MIN)", () -> addr(A("pekko", "sys", "h", Integer.MIN_VALUE)));
    run("CT-09 new(pekko,sys,null,1)", () -> addr(A("pekko", "sys", null, 1)));
    run("CT-10 new(pekko,sys,'',1)", () -> addr(A("pekko", "sys", "", 1)));
    run("CT-11 copy(host=Some(h),port=None)", () -> addr(copyOf(mysys, someS("h"), scala.Option.empty())));
    run("CT-12 copy(host=Some(''),port=None)", () -> addr(copyOf(mysys, someS(""), scala.Option.empty())));
    run("CT-13 copy(host=None,port=Some(7))", () -> addr(copyOf(mysys, scala.Option.empty(), someI(7))));
    run("CT-14 new('',s)", () -> addr(A("", "s")));
    run("CT-15 new(p,'')", () -> addr(A("p", "")));
    run("CT-16 new(p,'s@h:1')", () -> addr(A("p", "s@h:1")));
    run("CT-17 new(p,s,'h:1',2)", () -> addr(A("p", "s", "h:1", 2)));
    run("CT-18 apply(pekko,sys,h,1) eq new", () -> pairA(Address$.MODULE$.apply("pekko", "sys", "h", 1), A("pekko", "sys", "h", 1)));
    run("CT-19 parsed eq constructed", () -> pairA(AddressFromURIString$.MODULE$.apply("pekko://sys@h:1"), A("pekko", "sys", "h", 1)));
    run("CT-20 roundtrip parse(new(p,s,h,-5).toString)", () -> addr(AddressFromURIString$.MODULE$.apply(A("pekko", "sys", "h", -5).toString())));
    run("CT-21 roundtrip parse(new(p,s,in_valid,1).toString)", () -> addr(AddressFromURIString$.MODULE$.apply(A("pekko", "sys", "in_valid", 1).toString())));
    
    // ---------------------------------------------------------------- HC: host characters ("_[^.]*$")
    for (String h : new String[] {"_", "a_", "_a", "a_b.c", "a.b_c", "a_b.c_d", "a_.b", "a._", "\u00fc_x"})
      run("HC [" + h + "]", () -> { Address a = A("pekko", "sys", h, 1); String inv = "invalid=" + a.hasInvalidHostCharacters(); a.checkHostCharacters(); return inv + " checked"; });
    run("HC host=None", () -> { Address a = A("pekko", "sys"); a.checkHostCharacters(); return "invalid=" + a.hasInvalidHostCharacters() + " checked"; });
    run("HC host=Some(a_b) port=None", () -> { Address a = copyOf(mysys, someS("a_b"), scala.Option.empty()); a.checkHostCharacters(); return "checked"; });
    
    // ---------------------------------------------------------------- RT: RootActorPath(address, name)
    for (String n : new String[] {"/", "", "user", "/a", "/a/", "//", "a/b", "#", "/a#1", "a#b"})
      run("RT [" + n + "]", () -> path(RootActorPath$.MODULE$.apply(mysys, n)));
    run("RT new RootActorPath(addr) default name", () -> path(new RootActorPath(mysys, RootActorPath$.MODULE$.apply$default$2())));
    
    // ---------------------------------------------------------------- CH: child "/" (ActorCell.splitNameAndUid + ChildActorPath checks)
    for (String n : new String[] {"foo", "foo#7", "foo#+7", "foo#-7", "foo#0", "foo#abc", "foo#", "foo#99999999999", "foo#1#2",
        "#5", "", "foo#\u0665", "foo#+", "a/b", "/", "a b", "\u00fc", "$a", ".", "..", "a?b", "a%2"})
      run("CH [" + n + "]", () -> path(root(mysys).$div("user").$div(n)));
    run("CH root/'foo#7'", () -> path(root(mysys).$div("foo#7")));
    run("CH root/'a/b'", () -> path(root(mysys).$div("a/b")));
    run("CH java child('x#3')", () -> path(root(mysys).child("x#3")));
    
    // ---------------------------------------------------------------- DS: descendant "/(Iterable)" (skips empty elements)
    run("DS-01 [a,,b]", () -> path(root(mysys).descendant(List.of("a", "", "b"))));
    run("DS-02 []", () -> path(root(mysys).descendant(List.of())));
    run("DS-03 [,]", () -> path(root(mysys).descendant(List.of("", ""))));
    run("DS-04 [a,b#5]", () -> path(root(mysys).descendant(List.of("a", "b#5"))));
    run("DS-05 [a#5,b]", () -> path(root(mysys).descendant(List.of("a#5", "b"))));
    run("DS-06 [a/b]", () -> path(root(mysys).descendant(List.of("a/b"))));
    run("DS-07 [a#x]", () -> path(root(mysys).descendant(List.of("a#x"))));
    run("DS-08 child.descendant[c,d]", () -> path(root(mysys).$div("a").descendant(List.of("c", "d"))));
    
    // ---------------------------------------------------------------- UID: withUid
    run("UID-01 root.withUid(0)", () -> { ActorPath r = root(mysys); ActorPath w = r.withUid(0); return "same=" + (w == r) + " " + path(w); });
    run("UID-02 root.withUid(5)", () -> path(root(mysys).withUid(5)));
    run("UID-03 root.withUid(-1)", () -> path(root(mysys).withUid(-1)));
    run("UID-04 child.withUid(9)", () -> path(root(mysys).$div("a#3").withUid(9)));
    run("UID-05 child#3.withUid(0)", () -> path(root(mysys).$div("a#3").withUid(0)));
    run("UID-06 child#3.withUid(3) same", () -> { ActorPath c = root(mysys).$div("a#3"); return "same=" + (c.withUid(3) == c); });
    run("UID-07 child.withUid(MIN)", () -> path(root(mysys).$div("a").withUid(Integer.MIN_VALUE)));
    
    // ---------------------------------------------------------------- NAV: name, parent, root, elements, address
    run("NAV-01 root", () -> { ActorPath r = root(mysys); return path(r) + " parentIsSelf=" + (r.parent() == r) + " rootIsSelf=" + (r.root() == r); });
    run("NAV-02 child", () -> { ActorPath r = root(mysys); ActorPath c = r.$div("a").$div("b#4"); return path(c) + " parent=[" + c.parent() + "] parentUid=" + c.parent().uid() + " rootIsSame=" + (c.root() == r) + " root=[" + c.root() + "]"; });
    run("NAV-03 remote child address", () -> path(root(aA).$div("x")));
    run("NAV-04 root named elements", () -> path(RootActorPath$.MODULE$.apply(mysys, "/x")));
    
    // ---------------------------------------------------------------- STR: toString / toSerializationFormat / *WithAddress
    run("STR-01 child#7", () -> path(c7));
    run("STR-02 child#7 strWithAddr(remote)", () -> c7.toStringWithAddress(aA));
    run("STR-03 child#7 serWithAddr(remote)", () -> c7.toSerializationFormatWithAddress(aA));
    run("STR-04 child#7 strWithAddr(local other)", () -> c7.toStringWithAddress(other));
    run("STR-05 child#7 serWithAddr(local other)", () -> c7.toSerializationFormatWithAddress(other));
    run("STR-06 remote child#7 serWithAddr(other remote)", () -> root(aA).$div("foo#7").toSerializationFormatWithAddress(aB));
    run("STR-07 root serWithAddr(remote)", () -> root(mysys).toSerializationFormatWithAddress(aA));
    run("STR-08 root named '/x' strWithAddr(remote)", () -> RootActorPath$.MODULE$.apply(mysys, "/x").toStringWithAddress(aA));
    run("STR-09 root named '/x' child strWithAddr(remote)", () -> RootActorPath$.MODULE$.apply(mysys, "/x").$div("y").toStringWithAddress(aA));
    run("STR-10 child#-3 ser", () -> root(mysys).$div("a#-3").toSerializationFormat());
    // IgnoreActorRef (ActorRef.scala:543-560): equality-based, so any equal path keeps its address
    run("STR-11 ignore", () -> path(ign));
    run("STR-12 ignore strWithAddr(remote)", () -> ign.toStringWithAddress(aA));
    run("STR-13 ignore serWithAddr(remote)", () -> ign.toSerializationFormatWithAddress(aA));
    run("STR-14 user-built pekko://local/ignore#5 strWithAddr", () -> root(A("pekko", "local")).$div("ignore#5").toStringWithAddress(aA));
    run("STR-15 user-built pekko://local/ignore#5 serWithAddr", () -> root(A("pekko", "local")).$div("ignore#5").toSerializationFormatWithAddress(aA));
    run("STR-16 pekko://local/ignore/x strWithAddr", () -> root(A("pekko", "local")).$div("ignore").$div("x").toStringWithAddress(aA));
    run("STR-17 root pekko://local/ strWithAddr", () -> root(A("pekko", "local")).toStringWithAddress(aA));
    run("STR-18 akka://local/ignore strWithAddr", () -> root(A("akka", "local")).$div("ignore").toStringWithAddress(aA));
    
    // ---------------------------------------------------------------- CMPA: Address ordering (Address.scala:138-145), equals, hash
    run("CMPA-01 same fields", () -> pairA(A("pekko", "s", "h", 1), A("pekko", "s", "h", 1)));
    run("CMPA-02 protocol akka<pekko", () -> pairA(A("akka", "s", "h", 1), A("pekko", "s", "h", 1)));
    run("CMPA-03 system", () -> pairA(A("pekko", "a"), A("pekko", "b")));
    run("CMPA-04 host", () -> pairA(A("pekko", "s", "h1", 9), A("pekko", "s", "h2", 1)));
    run("CMPA-05 port", () -> pairA(A("pekko", "s", "h", 2), A("pekko", "s", "h", 10)));
    run("CMPA-06 negative port", () -> pairA(A("pekko", "s", "h", -5), A("pekko", "s", "h", 0)));
    run("CMPA-07 local vs remote", () -> pairA(A("pekko", "s"), A("pekko", "s", "h", 1)));
    run("CMPA-08 host None vs Some('') (same port None)", () -> pairA(mysys, copyOf(mysys, someS(""), scala.Option.empty())));
    run("CMPA-09 port None vs Some(0) (same host)", () -> pairA(copyOf(mysys, someS("h"), scala.Option.empty()), copyOf(mysys, someS("h"), someI(0))));
    run("CMPA-10 a=(None,1) b=(Some(''),2)", () -> pairA(hNoneP1, hEmptyP2));
    run("CMPA-11 b=(Some(''),2) c=(None,2)", () -> pairA(hEmptyP2, hNoneP2));
    run("CMPA-12 a=(None,1) c=(None,2)", () -> pairA(hNoneP1, hNoneP2));
    run("CMPA-13 local(None,None) vs (Some(''),Some(5))", () -> pairA(mysys, copyOf(mysys, someS(""), someI(5))));
    run("CMPA-14 UTF-16 order system U+FF61 vs U+10000", () -> pairA(A("pekko", "\uff61"), A("pekko", "\ud800\udc00")));
    run("CMPA-15 uppercase vs lowercase", () -> pairA(A("pekko", "Sys"), A("pekko", "sys")));
    run("CMPA-16 same toString different fields", () -> pairA(A("p", "s@h:1"), A("p", "s", "h", 1)));
    run("CMPA-17 sort [pekko://a, akka://a@h:2, akka://a@h:1, pekko://a@:0, pekko://a:3]", () -> {
      List<Address> l = new ArrayList<>(List.of(A("pekko", "a"), A("akka", "a", "h", 2), A("akka", "a", "h", 1),
          copyOf(A("pekko", "a"), someS(""), someI(0)), copyOf(A("pekko", "a"), scala.Option.empty(), someI(3))));
      l.sort(Address$.MODULE$.addressOrdering());
      return l.toString();
    });
    
    // ---------------------------------------------------------------- CMPP: ActorPath compareTo (ActorPath.scala:307-311, 481-494), equals, hash
    run("CMPP-01 root vs equal root", () -> pairP(r0, root(A("pekko", "mysys"))));
    run("CMPP-02 root vs child", () -> pairP(r0, r0.$div("a")));
    run("CMPP-03 /a vs /b", () -> pairP(r0.$div("a"), r0.$div("b")));
    run("CMPP-04 /a/b vs /b", () -> pairP(r0.$div("a").$div("b"), r0.$div("b")));
    run("CMPP-05 /b/a vs /a/b", () -> pairP(r0.$div("b").$div("a"), r0.$div("a").$div("b")));
    run("CMPP-06 /a#1 vs /a#2 (uid ignored)", () -> pairP(r0.$div("a#1"), r0.$div("a#2")));
    run("CMPP-07 /a vs /a on other system", () -> pairP(r0.$div("a"), root(other).$div("a")));
    run("CMPP-08 /a local vs /a remote", () -> pairP(r0.$div("a"), root(aA).$div("a")));
    run("CMPP-09 roots same toString, different Address", () -> pairP(root(A("p", "s@h:1")), root(A("p", "s", "h", 1))));
    run("CMPP-10 children of roots with same toString", () -> pairP(root(A("p", "s@h:1")).$div("x"), root(A("p", "s", "h", 1)).$div("x")));
    run("CMPP-11 root(s,'/a') vs root('s/','a')", () -> pairP(RootActorPath$.MODULE$.apply(A("p", "s"), "/a"), RootActorPath$.MODULE$.apply(A("p", "s/"), "a")));
    run("CMPP-12 root(s,'/a') vs child s/a", () -> pairP(RootActorPath$.MODULE$.apply(A("p", "s"), "/a"), root(A("p", "s")).$div("a")));
    run("CMPP-13 UTF-16 child names U+FF61 vs U+10000", () -> pairP(r0.$div("\uff61"), r0.$div("\ud800\udc00")));
    run("CMPP-14 fromString vs built", () -> pairP(fs("pekko://mysys/user/a#9"), r0.$div("user").$div("a")));
    run("CMPP-15 ignore vs user-built", () -> pairP(ign, root(A("pekko", "local")).$div("ignore#5")));
    run("CMPP-16 empty-name child vs root", () -> pairP(r0.$div(""), r0));
    
    // ---------------------------------------------------------------- VAL: validatePathElement / isValidPathElement (ActorPath.scala:97-148)
    for (int i = 0; i < valIn.length; i++) val(String.format("VAL-%02d [", i + 1) + valIn[i] + "]", valIn[i], null);
    val("VAL-FP-01 ['' in /a/b]", "", "/a/b");
    val("VAL-FP-02 [$a in /user/$a]", "$a", "/user/$a");
    val("VAL-FP-03 [ok in /x]", "ok", "/x");
    val("VAL-FP-04 [a b in '']", "a b", "");
    run("VAL-JAPI-01 ActorPaths.validatePathElement($x)", () -> { ActorPaths.validatePathElement("$x"); return "ok"; });
    run("VAL-JAPI-02 ActorPaths.validatePathElement(x, /p)", () -> { ActorPaths.validatePathElement("x y", "/p"); return "ok"; });
    run("VAL-JAPI-03 ActorPaths.isValidPathElement(a)", () -> "valid=" + ActorPaths.isValidPathElement("a"));
    
  }
}
