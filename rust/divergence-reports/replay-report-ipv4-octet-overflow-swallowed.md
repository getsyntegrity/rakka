# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-c5f9c6on/target-ipv4-octet-overflow-swallowed/release/examples/classic-parity`

Passed: 1042/1045

Fallback rate: not reported by this build

## Failing cases

### nfe-uri-ipv6-embedded-ip-126

Arguments: `path-from-string`

stdout:

```diff
- {"ok":false,"exception":"java.lang.NumberFormatException","message":"Error at index 9 in: \"2147483648\""}
+ {"ok":false,"exception":"java.net.MalformedURLException","message":"cannot parse as ActorPath: pekko://sys@[1:2:3:4:5:6:1.2.3.2147483648]:1/x"}
```

### nfe-uri-ipv6-embedded-ip

Arguments: `path-from-string`

stdout:

```diff
- {"ok":false,"exception":"java.lang.NumberFormatException","message":"Error at index 9 in: \"99999999999\""}
+ {"ok":false,"exception":"java.net.MalformedURLException","message":"cannot parse as ActorPath: pekko://sys@[::1.2.3.99999999999]/x"}
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
