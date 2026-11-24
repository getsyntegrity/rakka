Verdict path: fast-kill: case mb-0001 confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-58352-2v0v0ota/target-mailbox-id-selection-ignored/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### mb-0001

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-01 open to scheduled -> first=true second=false open scheduled=true suspendCount=0 raw=2
```

stderr:

```diff
+
+ thread 'main' (11501149) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
