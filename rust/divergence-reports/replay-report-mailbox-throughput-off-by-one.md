Verdict path: fast-kill: case tput-2-actors-thr1 confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-99457-336p785y/target-mailbox-throughput-off-by-one/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### tput-2-actors-thr1

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","b:b-0","a:a-1","b:b-1","a:a-2","b:b-2","a:a-3","b:b-3","a:a-4","b:b-4","a:a-5","b:b-5","a:a-6","b:b-6","a:a-7","b:b-7","a:a-8","b:b-8","a:a-9","b:b-9","a:a-10","b:b-10","a:a-11","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","b:b-0","b:b-1","a:a-2","a:a-3","b:b-2","b:b-3","a:a-4","a:a-5","b:b-4","b:b-5","a:a-6","a:a-7","b:b-6","b:b-7","a:a-8","a:a-9","b:b-8","b:b-9","a:a-10","a:a-11","b:b-10","b:b-11"]}]}
```
