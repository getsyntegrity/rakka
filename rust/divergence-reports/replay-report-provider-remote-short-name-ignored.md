# App replay

Build: `$TMPDIR/pekko-mutants-hrmb2f40/target-provider-remote-short-name-ignored/release/examples/classic-parity`

Passed: 2617/2621

Fallback rate: not reported by this build

## Failing cases

### da-0021

Arguments: `dynamic-access-case`

stdout:

```diff
- PRV-05 provider remote -> ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]
+ PRV-05 provider remote -> ERR ClassNotFoundException [remote]
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

### ps-0002

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-02 provider = remote -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
+ SEL-02 provider = remote -> ok identifier=custom fqcn=remote hasCluster=false ProviderClass=remote HasCluster=false
```

### ps-0014

Arguments: `provider-selection-case`

stdout:

```diff
- EQ-01 remote equals org.apache.pekko.remote.RemoteActorRefProvider -> ok true
+ EQ-01 remote equals org.apache.pekko.remote.RemoteActorRefProvider -> ok false
```
