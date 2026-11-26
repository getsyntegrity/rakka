Verdict path: full replay (2976 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-34186-mdzc44sx/target-deploy-fallback-keeps-the-tags/release/examples/classic-parity`

Passed: 2974/2976

Fallback rate: not reported by this build

## Failing cases

### dp-0031

Arguments: `deploy-case`

stdout:

```diff
- FB-10 the tags are not carried -> ok path="/a" config={} router=NoRouter scope=NoScopeGiven dispatcher="" mailbox="" tags=[]
+ FB-10 the tags are not carried -> ok path="/a" config={} router=NoRouter scope=NoScopeGiven dispatcher="" mailbox="" tags=[t1]
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
