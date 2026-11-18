#!/usr/bin/env python3
"""Token-sequence similarity between two Rust files (clean-room check, decision Q-X-18).

Strips comments and string contents, tokenises, and reports every maximal run of >= N identical consecutive tokens
(default 8) shared by the two files, with the lines of each file. Runs that are only signatures imposed by the
specification or numeric data generated from a public source must be explained by the reviewer.

  tools/legal/similarity.py OLD.rs NEW.rs [--min 8]
"""
import re, sys

TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|0[xXbBoO][0-9A-Fa-f_]+|[0-9][0-9_]*(?:\.[0-9]+)?|::|->|=>|&&|\|\||[<>=!+\-*/%&|^]=?|\S")


def tokens(path):
    text = open(path, encoding="utf-8").read()
    text = re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)
    out = []
    for n, line in enumerate(text.split("\n"), 1):
        line = re.sub(r"//.*$", "", line)
        line = re.sub(r'"(?:\\.|[^"\\])*"', '"S"', line)
        for m in TOKEN.finditer(line):
            out.append((m.group(0), n))
    return out


def shared_runs(a, b, minimum):
    index = {}
    for j in range(len(b)):
        index.setdefault(b[j][0], []).append(j)
    runs, covered = [], set()
    for i in range(len(a)):
        for j in index.get(a[i][0], []):
            if i > 0 and j > 0 and a[i - 1][0] == b[j - 1][0]:
                continue   # not the start of a maximal run
            k = 0
            while i + k < len(a) and j + k < len(b) and a[i + k][0] == b[j + k][0]:
                k += 1
            if k >= minimum:
                runs.append((k, a[i][1], a[i + k - 1][1], b[j][1], b[j + k - 1][1], " ".join(t for t, _ in a[i:i + min(k, 14)])))
    return sorted(runs, reverse=True)


def main():
    args = [x for x in sys.argv[1:] if not x.startswith("--")]
    minimum = int(sys.argv[sys.argv.index("--min") + 1]) if "--min" in sys.argv else 8
    if "--min" in sys.argv: args.remove(str(minimum))
    old, new = args
    a, b = tokens(old), tokens(new)
    runs = shared_runs(a, b, minimum)
    print(f"{old}: {len(a)} tokens; {new}: {len(b)} tokens; shared runs >= {minimum}: {len(runs)}; longest: {runs[0][0] if runs else 0}")
    for k, l1, l2, m1, m2, text in runs:
        print(f"  {k:4d} tokens  old:{l1}-{l2}  new:{m1}-{m2}   {text}{' ...' if k > 14 else ''}")


if __name__ == "__main__":
    main()
