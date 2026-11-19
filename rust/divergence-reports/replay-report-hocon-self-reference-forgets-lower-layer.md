# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-b87a6ub3/target-hocon-self-reference-forgets-lower-layer/release/examples/classic-parity`

Passed: 2004/2009

Fallback rate: not reported by this build

## Failing cases

### ch-0015

Arguments: `config-hocon-case`

stdout:

```diff
- SUB-05 self-reference concatenates strings -> ok foobar
+ SUB-05 self-reference concatenates strings -> ok bar
```

### ch-0016

Arguments: `config-hocon-case`

stdout:

```diff
- SUB-06 += appends to an existing list -> ok [1, 2]
+ SUB-06 += appends to an existing list -> ok [2]
```

### ch-0153

Arguments: `config-hocon-case`

stdout:

```diff
- REF-02 library-extensions with application value first -> ok [org.apache.pekko.serialization.SerializationExtension$, my.Ext]
+ REF-02 library-extensions with application value first -> ok [my.Ext]
```

### ch-0481

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.library-extensions -> ok LIST ["org.apache.pekko.serialization.SerializationExtension$","org.apache.pekko.actor.typed.internal.adapter.ActorSystemAdapter$LoadTypedExtensions"]
+ KEY actor-typed pekko.library-extensions -> ok LIST ["org.apache.pekko.actor.typed.internal.adapter.ActorSystemAdapter$LoadTypedExtensions"]
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
