Verdict path: full replay (2959 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-79342-7ewgt65e/target-dead-letters-answers-a-watch-of-itself/release/examples/classic-parity`

Passed: 2957/2959

Fallback rate: not reported by this build

## Failing cases

### mb-0014

Arguments: `mailbox-case`

stdout:

```diff
- MBX-DL-01 watching deadLetters is not answered, watching a stopped actor is -> watcherLog=[terminated]
+ MBX-DL-01 watching deadLetters is not answered, watching a stopped actor is -> watcherLog=[terminated, terminated]
```

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
