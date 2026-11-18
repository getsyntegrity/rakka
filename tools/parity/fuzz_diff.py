#!/usr/bin/env python3
"""Differential fuzzer: Pekko 2.0.0 oracle (JVM) vs the Rust port, over ActorPath.fromString.

Both sides expose `path-from-string-lines` (one JSON string literal per stdin line -> one JSON
result per stdout line), so tens of thousands of inputs run in one JVM and one Rust process.

  tools/fuzz_diff.py [--count N] [--seed S] [--show K]

Exit status: 0 = identical on every input, 1 = at least one divergence.
"""
import argparse, json, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ORACLE = [os.path.join(ROOT, "oracle", "run.sh"), "path-from-string-lines"]
RUST = [os.environ.get("CLASSIC_PARITY_BIN", os.path.join(ROOT, "target", "release", "examples", "classic-parity")),
        "path-from-string-lines"]

TOKENS = [
    "pekko", "akka", "a1", "sys", "my_sys", "host", "h-1", "h-", "-", "_", ".", "..", ":", "::", "@", "/", "//", "///",
    "#", "##", "?", "%", "%41", "%zz", "%4", "%C3%A9", "%C0%80", "%ED%A0%80", "%F0%9F%98%80", "%FF", "%E2%82", "%F4%90%80%80",
    "%C3", "%80", "%E0%80%80", "[", "]", "[::1]", "[::1", "::1", "1.2.3.4", "256", "2552", "65536", "99999999999", "0", "-5", "+5",
    "user", "system", "a", "b", "c#7", "\u00e9", "\u0660", "\u0663", "\uff13", "\ud800", "\udc00", "\ud83d\ude00", "\u00a0",
    "\u0085", " ", "\t", "\\", "<", ">", "{", "}", "|", "^", "`", '"', "!", "~", "*", "'", "(", ")", ";", "=", "+", "$", ",",
    "&", "fe80::1%eth0", "[fe80::1%eth0]", "[::ffff:1.2.3.4]", "[1:2:3:4:5:6:7:8]", "[1:2:3:4:5:6:7:8:9]", "[::1.2.3.4]", "[1::2::3]",
    "%25", "1.2.3", "1.2.3.4.5", "1.2.3.256", "0x1", "12345678901234567890", "2147483647", "2147483648", "-2147483648",
    "-2147483649", "\u2028", "\u200b", "\u3000", "\u1680", "\u2000", "\u0000", "\u007f", "\u0080",
    "pekko://", "pekko://sys", "pekko://sys@host:2552", "pekko://sys@host:2552/user", "/user/", "/user/a/b", "/temp/", "/deadLetters",
    "pekko.tcp://sys@127.0.0.1:2552", "pekko://sys@[::1]:2552", "pekko://s@h:1/x#1", "x#0", "x#-1", "x#+1", "x#", "x#abc",
]
PREFIXES = ["pekko://", "pekko://sys@host:2552", "pekko://sys", "pekko.tcp://sys@h:1", "a:", "x://", "", "//", "pekko:/"]
ASCII = [chr(c) for c in range(0, 128)]
EXTRA = ["\u00e9", "\u0660", "\u00a0", "\u0085", "\ud800", "\u2028", "\uff13", "\ud83d\ude00"]


SCHEMES = ["pekko", "akka", "pekko.tcp", "a+b-c.d", "PEKKO", "1a", "a_b", ""]
USERS = ["sys", "s", "my-sys", "my_sys", "S%41", "%C3%A9", "%ED%A0%80", "%F0%9F%98%80", "\u00e9", "a:b", "a;b", "a=b&c$d,e+f", "", "a@b",
         "%", "%zz", "\ud800", "\u00a0", "a b", "a\u0660", "%E2%82%AC", "%C3", "%FF%FE", "%F4%90%80%80", "%C0%AF"]
HOSTS = ["host", "h-1", "h.example.com", "a.b.c", "1.2.3.4", "1.2.3", "1.2.3.4.5", "256.1.1.1", "01.02.03.04", "[::1]", "[1::]", "[::]",
         "[fe80::1%eth0]", "[fe80::1%]", "[::1.2.3.4]", "[::1.2.3.99999999999]", "[1:2:3:4:5:6:7:8]", "[1:2:3:4:5:6:1.2.3.4]", "[::ffff:1.2.3.4]",
         "[1::2::3]", "[g::1]", "[", "[]", "a_b", "a_b.c", "_", "-h", "h-", "h..x", "1h", "a.1", "a.b1", "a.1b", "h.", "", "\u00e9", "h%41"]
PORTS = ["", ":", ":2552", ":0", ":65536", ":99999", ":2147483647", ":2147483648", ":99999999999", ":+1", ":-1", ":a", ":1a", ":\u0663", ":1:2", ":00012"]
SEGS = ["", "user", "a", "b", "$a", "a%20b", "a%zz", "temp", "x;y", "x:y", "x@y", "x?y", "x\u00e9", "x y", "..", ".", "a#1", "a#", "a#-1", "a#+1", "a#007",
        "a#2147483648", "a#-2147483648", "a#-2147483649", "a#1x", "a#\u0663", "a#%31", "#5", "a#1#2", "a%", "a[b]", "a{b}"]


def structured(rng):
    s = rng.choice(SCHEMES) + "://" if rng.random() < 0.93 else rng.choice(SCHEMES) + ":"
    if rng.random() < 0.85:
        s += rng.choice(USERS) + "@"
    s += rng.choice(HOSTS) + rng.choice(PORTS)
    for _ in range(rng.choice([0, 0, 1, 2, 3, 5])):
        s += "/" + rng.choice(SEGS)
    if rng.random() < 0.12:
        s += rng.choice(["?q", "?", "?a=b#c", "#"])
    return s


BYTES = [0x00, 0x41, 0x7F, 0x80, 0xBF, 0xC0, 0xC1, 0xC2, 0xDF, 0xE0, 0xE1, 0xED, 0xEF, 0xF0, 0xF1, 0xF4, 0xF5, 0xF8, 0xFF,
         0x9F, 0xA0, 0x8F, 0x90, 0xBE, 0x82, 0xAC]


def utf8_user(rng):
    """Random %XX byte soup in the user info, whose decoding is the JDK's UTF-8 decoder."""
    esc = "".join("%%%02X" % rng.choice(BYTES) if rng.random() < 0.8 else rng.choice("aZ-") for _ in range(rng.randint(1, 7)))
    return "pekko://" + esc + "@h:1" + rng.choice(["", "/x", "/x#1"])


def gen(rng):
    kind = rng.random()
    if kind < 0.12:
        return utf8_user(rng)
    kind = rng.random()
    if kind < 0.45:
        return structured(rng)
    if kind < 0.70:
        return "".join(rng.choice(TOKENS) for _ in range(rng.randint(1, 9)))
    if kind < 0.88:
        s = rng.choice(PREFIXES)
        for _ in range(rng.randint(0, 6)):
            s += rng.choice(TOKENS)
        return s
    n = rng.randint(1, 14)
    return "".join(rng.choice(ASCII + EXTRA) for _ in range(n))


def has_lone_surrogate(s):
    """An input the Rust API cannot represent (public inputs are &str; decision Q-X-17): NOT_APPLICABLE."""
    try:
        s.encode("utf-8")
        return False
    except UnicodeEncodeError:
        return True


def run(cmd, data):
    p = subprocess.run(cmd, input=data.encode("ascii"), capture_output=True)
    if p.returncode != 0:
        sys.exit(f"{cmd[0]} failed ({p.returncode}): {p.stderr.decode(errors='replace')[:500]}")
    return p.stdout.decode("ascii").split("\n")[:-1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=50000)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--show", type=int, default=15)
    a = ap.parse_args()
    rng = random.Random(a.seed)
    generated = [gen(rng) for _ in range(a.count)]
    inputs = [s for s in generated if not has_lone_surrogate(s)]
    not_applicable = len(generated) - len(inputs)
    data = "".join(json.dumps(s, ensure_ascii=True) + "\n" for s in inputs)
    want, got = run(ORACLE, data), run(RUST, data)
    assert len(want) == len(got) == len(inputs), (len(want), len(got), len(inputs))
    bad = [(s, w, g) for s, w, g in zip(inputs, want, got) if w != g]
    oks = sum(1 for w in want if w.startswith('{"ok":true'))
    print(f"seed={a.seed} cases={len(inputs)} not_applicable={not_applicable} oracle_ok={oks} oracle_err={len(inputs) - oks} divergences={len(bad)}")
    for s, w, g in bad[: a.show]:
        print("INPUT :", json.dumps(s))
        print("ORACLE:", w)
        print("RUST  :", g)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
