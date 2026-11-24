Verdict path: fast-kill: case mb-0027 confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-57562-cmt7pz01/target-mailbox-dispatcher-requirement-ignored/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### mb-0027

Arguments: `mailbox-case`

stdout:

```diff
- MBX-REQ-04 the queue semantics the dispatcher requires selects the mailbox of the actors that require nothing, and of those whose own requirement has no binding; the actor's own binding and a selected id come first -> plain received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; anyQueue received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; audit received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; selected received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[];
+ MBX-REQ-04 the queue semantics the dispatcher requires selects the mailbox of the actors that require nothing, and of those whose own requirement has no binding; the actor's own binding and a selected id come first -> plain received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; anyQueue received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; audit received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; selected received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[];
```
