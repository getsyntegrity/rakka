# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-i2z9eagu/target-drop-uid-in-serialization-format/release/examples/classic-parity`

Passed: 551/566

Fallback rate: not reported by this build

## Failing cases

### cov53-uid-plus-sign

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### cov54-uid-minus-sign

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-5,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### cov57-uid-min-int

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-2147483648","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-fragment-empty-path-e

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/","name":"","elements":["user",""],"uid":5,"toStringWithoutAddress":"/user/","toSerializationFormat":"pekko://sys/user/#5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/","name":"","elements":["user",""],"uid":5,"toStringWithoutAddress":"/user/","toSerializationFormat":"pekko://sys/user/","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-fragment-on-root-slas

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/","name":"","elements":[""],"uid":5,"toStringWithoutAddress":"/","toSerializationFormat":"pekko://sys/#5","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/","name":"","elements":[""],"uid":5,"toStringWithoutAddress":"/","toSerializationFormat":"pekko://sys/","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-local-uid

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#7","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-query-with-fragment

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":9,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#9","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":9,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### ok-remote-uid

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys@host:2552/user/a/b","name":"b","elements":["user","a","b"],"uid":42,"toStringWithoutAddress":"/user/a/b","toSerializationFormat":"pekko://sys@host:2552/user/a/b#42","address":{"protocol":"pekko","system":"sys","host":"host","port":2552,"toString":"pekko://sys@host:2552"}}
+ {"ok":true,"toString":"pekko://sys@host:2552/user/a/b","name":"b","elements":["user","a","b"],"uid":42,"toStringWithoutAddress":"/user/a/b","toSerializationFormat":"pekko://sys@host:2552/user/a/b","address":{"protocol":"pekko","system":"sys","host":"host","port":2552,"toString":"pekko://sys@host:2552"}}
```

### uid-arabic-indic-digits

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":34,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#34","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":34,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-fullwidth-digits

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":12,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#12","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":12,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-int-max

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":2147483647,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#2147483647","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":2147483647,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-int-min

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-2147483648","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-2147483648,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-leading-zeros

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#7","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":7,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-negative

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#-1","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":-1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```

### uid-plus-sign

Arguments: `path-from-string`

stdout:

```diff
- {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a#1","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
+ {"ok":true,"toString":"pekko://sys/user/a","name":"a","elements":["user","a"],"uid":1,"toStringWithoutAddress":"/user/a","toSerializationFormat":"pekko://sys/user/a","address":{"protocol":"pekko","system":"sys","host":null,"port":null,"toString":"pekko://sys"}}
```
