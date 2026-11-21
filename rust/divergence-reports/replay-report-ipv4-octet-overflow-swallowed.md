Verdict path: fast-kill: case nfe-uri-ipv6-embedded-ip confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-14538-rtqx6ml7/target-ipv4-octet-overflow-swallowed/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### nfe-uri-ipv6-embedded-ip

Arguments: `path-from-string`

stdout:

```diff
- {"ok":false,"exception":"java.lang.NumberFormatException","message":"Error at index 9 in: \"99999999999\""}
+ {"ok":false,"exception":"java.net.MalformedURLException","message":"cannot parse as ActorPath: pekko://sys@[::1.2.3.99999999999]/x"}
```
