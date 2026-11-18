# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-nknvl_e7/target-version-difference-saturates/release/examples/classic-parity`

Passed: 1131/1133

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

### uc-0164

Arguments: `util-core-case`

stdout:

```diff
- VR-17 negative and wrapping numbers -> ok [-1.2.3]/[1.2.3]=-2 eq=false [1.2.3]/[-1.2.3]=2 eq=false [-1.2.3]/[-1.2.3]=0 eq=true [-1.2.3]/[-2.2.3]=1 eq=false [2147483647.0.0]/[-1.0.0]=-2147483648 eq=false [-1.0.0]/[2147483647.0.0]=-2147483648 eq=false [-2147483648.0.0]/[2147483647.0.0]=1 eq=false [2147483647.0.0]/[-2147483648.0.0]=-1 eq=false [1.2147483647.0]/[1.-1.0]=-2147483648 eq=false [1.-1.0]/[1.2147483647.0]=-2147483648 eq=false [1.2.-3]/[1.2.3]=NumberFormatException[For input string: ""] eq=- [1.2.3]/[1.2.-3]=NumberFormatException[For input string: ""] eq=- [1.2.-2147483648]/[1.2.2147483647]=NumberFormatException[For input string: ""] eq=- [1.2.2147483647]/[1.2.-2147483648]=NumberFormatException[For input string: ""] eq=-
+ VR-17 negative and wrapping numbers -> ok [-1.2.3]/[1.2.3]=-2 eq=false [1.2.3]/[-1.2.3]=2 eq=false [-1.2.3]/[-1.2.3]=0 eq=true [-1.2.3]/[-2.2.3]=1 eq=false [2147483647.0.0]/[-1.0.0]=2147483647 eq=false [-1.0.0]/[2147483647.0.0]=-2147483648 eq=false [-2147483648.0.0]/[2147483647.0.0]=-2147483648 eq=false [2147483647.0.0]/[-2147483648.0.0]=2147483647 eq=false [1.2147483647.0]/[1.-1.0]=2147483647 eq=false [1.-1.0]/[1.2147483647.0]=-2147483648 eq=false [1.2.-3]/[1.2.3]=NumberFormatException[For input string: ""] eq=- [1.2.3]/[1.2.-3]=NumberFormatException[For input string: ""] eq=- [1.2.-2147483648]/[1.2.2147483647]=NumberFormatException[For input string: ""] eq=- [1.2.2147483647]/[1.2.-2147483648]=NumberFormatException[For input string: ""] eq=-
```
