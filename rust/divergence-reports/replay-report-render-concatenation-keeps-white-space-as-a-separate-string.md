Verdict path: fast-kill: case dp-0056 confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-34186-mdzc44sx/target-render-concatenation-keeps-white-space-as-a-separate-string/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### dp-0056

Arguments: `deploy-case`

stdout:

```diff
- TS-12 toString with an unresolved optional substitution and a concatenation -> ok Deploy(/a, Config(SimpleConfigObject({"a":${?b},"c":${d}" suffix","e":"x "${f}})), NoRouter, NoScopeGiven, , , Set())
+ TS-12 toString with an unresolved optional substitution and a concatenation -> ok Deploy(/a, Config(SimpleConfigObject({"a":${?b},"c":${d}"suffix","e":"x"""${f}})), NoRouter, NoScopeGiven, , , Set())
```
