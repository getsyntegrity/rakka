# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-3pr9tpx_/target-throughput-ignored/release/examples/classic-parity`

Passed: 1034/1045

Fallback rate: not reported by this build

## Failing cases

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```

### tput-2-actors-thr1

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","b:b-0","a:a-1","b:b-1","a:a-2","b:b-2","a:a-3","b:b-3","a:a-4","b:b-4","a:a-5","b:b-5","a:a-6","b:b-6","a:a-7","b:b-7","a:a-8","b:b-8","a:a-9","b:b-9","a:a-10","b:b-10","a:a-11","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
```

### tput-2-actors-thr10

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","a:a-10","a:a-11","b:b-10","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
```

### tput-2-actors-thr2

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","b:b-0","b:b-1","a:a-2","a:a-3","b:b-2","b:b-3","a:a-4","a:a-5","b:b-4","b:b-5","a:a-6","a:a-7","b:b-6","b:b-7","a:a-8","a:a-9","b:b-8","b:b-9","a:a-10","a:a-11","b:b-10","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
```

### tput-2-actors-thr3

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","b:b-0","b:b-1","b:b-2","a:a-3","a:a-4","a:a-5","b:b-3","b:b-4","b:b-5","a:a-6","a:a-7","a:a-8","b:b-6","b:b-7","b:b-8","a:a-9","a:a-10","a:a-11","b:b-9","b:b-10","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
```

### tput-2-actors-thr5

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","a:a-10","a:a-11","b:b-10","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
```

### tput-2-actors-thr7

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","b:b-8","b:b-9","b:b-10","b:b-11"]}]}
```

### tput-3-uneven-thr3

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/c","name":"c","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","b:b-0","b:b-1","b:b-2","c:c-0","c:c-1","c:c-2","a:a-3","a:a-4","a:a-5","c:c-3","c:c-4","c:c-5","a:a-6","a:a-7","a:a-8","c:c-6","a:a-9","a:a-10","a:a-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/c","name":"c","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","c:c-0","c:c-1","c:c-2","c:c-3","c:c-4","c:c-5","c:c-6"]}]}
```

### tput-3-uneven-thr5

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/c","name":"c","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","b:b-0","b:b-1","b:b-2","c:c-0","c:c-1","c:c-2","c:c-3","c:c-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","c:c-5","c:c-6","a:a-10","a:a-11"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/c","name":"c","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7","a:a-8","a:a-9","a:a-10","a:a-11","b:b-0","b:b-1","b:b-2","c:c-0","c:c-1","c:c-2","c:c-3","c:c-4","c:c-5","c:c-6"]}]}
```

### tput-fill-order-b-first

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","b:b-0","b:b-1","b:b-2","a:a-0","a:a-1","a:a-2","b:b-3","b:b-4","b:b-5","a:a-3","a:a-4","a:a-5","b:b-6","b:b-7","a:a-6","a:a-7"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell-seq","ok":true},{"op":"tell-seq","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5","b:b-6","b:b-7","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","a:a-6","a:a-7"]}]}
```

### tput-interleaved-enqueue

Arguments: `scenario`

stdout:

```diff
- {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","b:b-0","b:b-1","b:b-2","b:b-3","a:a-4","a:a-5","b:b-4","b:b-5"]}]}
+ {"system":{"ok":true,"name":"sys","toString":"pekko://sys","root":"pekko://sys/","user":"pekko://sys/user","systemGuardian":"pekko://sys/system","deadLetters":"pekko://sys/deadLetters"},"steps":[{"op":"spawn","ok":true,"path":"pekko://sys/user/gate","name":"gate","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/a","name":"a","parent":"pekko://sys/user","hasUid":true},{"op":"spawn","ok":true,"path":"pekko://sys/user/b","name":"b","parent":"pekko://sys/user","hasUid":true},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"await-events","ok":true,"events":[{"ev":"pre-start"}]},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"tell","ok":true},{"op":"release","ok":true},{"op":"await-order","ok":true,"order":["gate:block","a:a-0","a:a-1","a:a-2","a:a-3","a:a-4","a:a-5","b:b-0","b:b-1","b:b-2","b:b-3","b:b-4","b:b-5"]}]}
```
