Verdict path: full replay (2967 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-91650-9f9saoy8/target-mailbox-registered-default-ignored-without-a-selection/release/examples/classic-parity`

Passed: 2965/2967

Fallback rate: not reported by this build

## Failing cases

### mb-0022

Arguments: `mailbox-case`

stdout:

```diff
- MBX-CFG-02 a mailbox configured as the default serves the actors without a selection, not the ids that name another -> w0 received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; w1 received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; w2 received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; w3 received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; w4 received=[block, m1, m2, m3, m4] deadLettered=[m5, m6];
+ MBX-CFG-02 a mailbox configured as the default serves the actors without a selection, not the ids that name another -> w0 received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; w1 received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; w2 received=[block, m6, m5, m4, m3, m2, m1] deadLettered=[]; w3 received=[block, m1, m2, m3, m4, m5, m6] deadLettered=[]; w4 received=[block, m1, m2, m3, m4] deadLettered=[m5, m6];
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
