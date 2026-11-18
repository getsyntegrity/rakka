# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-py4t6s5v/target-switch-no-revert-on-panic/release/examples/classic-parity`

Passed: 1411/1417

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

### uc-0316

Arguments: `util-core-case`

stdout:

```diff
- SC-03 an exception in the action reverts the switch -> ok threw:boom t [boom] t threw:boom t [boom] t threw:boom t
+ SC-03 an exception in the action reverts the switch -> ok threw:boom f [boom] f threw:boom f [boom] f threw:boom f
```

### uc-0322

Arguments: `util-core-case`

stdout:

```diff
- SC-09 failed actions leave no trace of success -> ok ok returned t [] threw:boom t [boom]
+ SC-09 failed actions leave no trace of success -> ok ok returned t [] threw:boom f [boom]
```

### uc-0323

Arguments: `util-core-case`

stdout:

```diff
- SC-71 random sequence seed 71 -> ok none t none t some(yes) f t on threw:boom some(yes) some(yes) on some(yes) t t locked locked t t returned some(yes) t t off some(yes) some(yes) some(yes) f none t f f f returned some(yes) f f off off off t some(yes) none t f inner=ttt f inner=ftt threw:boom threw:boom f locked some(yes) none t t threw:boom f t f some(yes) threw:boom f threw:boom f f locked inner=ftt some(yes) t some(yes) none returned some(yes) f f t t none locked f f t locked t t f some(yes) t inner=ftt t f f some(yes) t t f threw:boom f t locked on t some(yes) inner=ftt none t t some(yes) t t f t f t threw:boom some(yes) none some(yes) t returned t some(yes) t returned f f f t t f none f f t f inner=ttt t f f f some(yes) threw:boom t none f locked inner=ttt inner=ftt locked f f threw:boom t f f f t f f f f f locked t f f f f none none f f on f inner=ftt returned returned f returned t on f none some(yes) locked t f some(yes) threw:boom t t t some(yes) t t none some(yes) f off some(yes) returned t f t some(yes) locked t inner=ttt f some(yes) f inner=ftt t t t inner=ftt none some(yes) returned some(yes) none t f t none none some(yes) some(yes) t none t off t some(yes) f some(yes) f f t f none t t f off some(yes) some(yes) inner=ttt on t locked f inner=ftt returned inner=ftt t t f some(yes) some(yes) t threw:boom f t f t f some(yes) f t f t returned threw:boom t none threw:boom some(yes) f threw:boom f some(yes) some(yes) inner=ftt on threw:boom inner=ftt f f some(yes) on none f t returned f f t f t [y,on,y,boom,y,y,y,y,ifon,off,y,y,on,y,whoff,y,y,y,y,whoff,y,y,y,y,on,y,nested,nested,boom,boom,y,y,off,whoff,boom,y,boom,boom,y,nested,y,ifon,y,y,on,y,y,y,nested,ifon,y,ifon,boom,y,y,ifon,y,nested,off,on,y,ifoff,whoff,boom,y,y,ifoff,on,y,off,nested,off,y,boom,y,nested,nested,y,boom,ifon,y,ifon,y,nested,y,y,y,y,boom,on,off,y,off,y,y,y,ifon,y,y,off,nested,y,nested,whon,whon,nested,y,y,ifon,whon,y,y,whon,off,y,ifoff,y,y,ifoff,ifoff,y,y,y,nested,y,whon,y,nested,nested,off,y,y,ifoff,boom,on,whon,y,ifon,boom,boom,y,boom,y,y,nested,y,boom,nested,y,y,whon]
+ SC-71 random sequence seed 71 -> ok none t none t some(yes) f t on threw:boom none none off none f f locked locked t t returned some(yes) t t off some(yes) some(yes) some(yes) f none t f f f returned some(yes) f f off off off t some(yes) none t f inner=ttt f inner=ftt threw:boom returned t locked none some(yes) f t threw:boom t f f some(yes) threw:boom t threw:boom t f locked inner=ftt some(yes) t some(yes) none returned some(yes) f f t t none locked f f t locked t t f some(yes) t inner=ftt t f f some(yes) t t f threw:boom t t locked on t some(yes) inner=ftt none t t some(yes) t t f t f t threw:boom none some(yes) none f threw:boom t some(yes) t returned f f f t t f none f f t f inner=ttt t f f f some(yes) threw:boom f some(yes) t locked inner=ftt inner=ftt locked f f threw:boom f t t t f t f f f f locked t f f f f none none f f on f inner=ftt returned returned f returned t on f none some(yes) locked t f some(yes) threw:boom f t t some(yes) t t none some(yes) f off some(yes) returned t f t some(yes) locked t inner=ttt f some(yes) f inner=ftt t t t inner=ftt none some(yes) returned some(yes) none t f t none none some(yes) some(yes) t none t off t some(yes) f some(yes) f f t f none t t f off some(yes) some(yes) inner=ttt on t locked f inner=ftt returned inner=ftt t t f some(yes) some(yes) t threw:boom t t f t f some(yes) f t f t returned threw:boom f some(yes) returned none t returned t none none inner=ttt on threw:boom inner=ttt f f some(yes) on none f t returned f f t f t [y,on,y,boom,y,y,y,on,y,whoff,y,y,y,y,whoff,y,y,y,y,on,y,nested,nested,boom,ifoff,y,y,whoff,boom,whon,y,boom,on,boom,on,y,nested,y,ifon,y,y,on,y,y,y,nested,ifon,y,ifon,boom,y,y,ifon,y,nested,off,on,y,ifoff,whoff,boom,y,boom,on,y,off,nested,off,y,boom,y,whon,y,nested,nested,y,boom,ifoff,ifoff,ifoff,y,ifon,y,nested,y,y,y,y,boom,off,y,off,y,y,y,ifon,y,y,off,nested,y,nested,whon,whon,nested,y,y,ifon,whon,y,y,whon,off,y,ifoff,y,y,ifoff,ifoff,y,y,y,nested,y,whon,y,nested,nested,off,y,y,ifoff,boom,on,whon,y,ifon,boom,y,ifoff,nested,y,boom,nested,y,y,whon]
```

### uc-0324

Arguments: `util-core-case`

stdout:

```diff
- SC-72 random sequence seed 72 -> ok f f t threw:boom off some(yes) some(yes) t t some(yes) none f t t returned inner=ftt returned t locked f t locked returned f threw:boom f none t t f threw:boom inner=ttt none none t f f t threw:boom locked t f f inner=ftt on t t t f f none f threw:boom f t locked f inner=ttt t some(yes) f t f f f f some(yes) f t threw:boom f inner=ttt on f t t some(yes) on t t f t some(yes) some(yes) returned threw:boom some(yes) locked locked f on none t t t some(yes) t f t returned f t locked f f t f t inner=ttt none f f none t t inner=ttt some(yes) threw:boom f f f f t t t none t threw:boom none some(yes) f t off t t on t some(yes) on t f f f f none t f t some(yes) some(yes) on returned f none inner=ftt f t inner=ftt f t threw:boom none f some(yes) t f t locked none some(yes) some(yes) f f f t f locked t none threw:boom none returned some(yes) returned t none f some(yes) off none some(yes) t inner=ftt some(yes) t none none some(yes) f f f t inner=ftt locked t t none none none f f f t f t f f f f off returned some(yes) none off some(yes) t f f f inner=ttt t some(yes) t locked inner=ttt none f f t f some(yes) on t t none off f none some(yes) locked f f none f locked f f f t t t some(yes) f some(yes) t locked locked some(yes) f locked t t f t t t returned some(yes) f some(yes) t t threw:boom t t f t f f none f f f inner=ttt on f none returned t f t [whoff,boom,y,y,y,whoff,whoff,y,whon,nested,y,on,y,boom,ifoff,boom,nested,ifon,off,boom,y,nested,y,boom,off,y,nested,y,y,boom,nested,y,ifon,whon,y,y,ifon,whon,y,y,boom,y,y,y,y,y,y,whon,nested,nested,y,boom,ifoff,boom,y,ifoff,y,on,ifon,y,whon,y,y,off,whoff,y,y,y,nested,nested,off,boom,y,on,y,y,y,off,y,whoff,boom,y,ifoff,y,y,y,on,nested,y,whon,y,ifon,nested,y,off,whoff,ifoff,y,y,y,y,nested,ifon,y,off,y,nested,ifon,y,y,off,whoff,y,y,y,y,whoff,whoff,y,y,y,y,y,y,whoff,whoff,y,y,ifoff,boom,ifon,off,nested,y]
+ SC-72 random sequence seed 72 -> ok f f t threw:boom on none none f f none some(yes) t f t returned inner=ftt returned t locked f t locked returned f threw:boom t some(yes) f t f threw:boom inner=ftt none none t f f t threw:boom locked f f f inner=ftt on t t t f f none f threw:boom t f locked f inner=ttt t some(yes) f t f f f f some(yes) f t threw:boom t inner=ftt on f t t some(yes) on t t f t some(yes) some(yes) returned threw:boom none locked locked t off some(yes) f t t some(yes) t f t returned f t locked f f t f t inner=ttt none f f none t t inner=ttt some(yes) threw:boom t f f f t t t none t threw:boom some(yes) none t f on f t on t some(yes) on t f f f f none t f t some(yes) some(yes) on returned f none inner=ftt f t inner=ftt f t threw:boom some(yes) t none f f t locked none some(yes) some(yes) f f f t f locked t none threw:boom some(yes) threw:boom some(yes) returned t none f some(yes) off none some(yes) t inner=ftt some(yes) t none none some(yes) f f f t inner=ftt locked t t none none none f f f t f t f f f f off returned some(yes) none off some(yes) t f f f inner=ttt t some(yes) t locked inner=ttt none f f t f some(yes) on t t none off f none some(yes) locked f f none f locked f f f t t t some(yes) f some(yes) t locked locked some(yes) f locked t t f t t t returned some(yes) f some(yes) t t threw:boom f f t f f f none f f f inner=ttt on f none returned t f t [whoff,boom,y,y,whon,nested,y,on,y,boom,ifoff,y,ifoff,boom,nested,ifon,off,boom,y,nested,y,boom,y,nested,y,y,boom,nested,y,ifon,whon,y,y,ifon,whon,y,y,boom,y,y,ifoff,y,y,y,y,whon,nested,nested,y,boom,on,ifoff,boom,y,y,ifon,y,whon,y,y,off,whoff,y,y,y,nested,nested,off,boom,y,y,y,y,off,y,whoff,boom,y,boom,y,ifoff,y,y,y,on,nested,y,whon,y,ifon,nested,y,off,whoff,ifoff,y,y,y,y,nested,ifon,y,off,y,nested,ifon,y,y,off,whoff,y,y,y,y,whoff,whoff,y,y,y,y,y,y,whoff,whoff,y,y,ifoff,boom,whoff,nested,y]
```

### uc-0325

Arguments: `util-core-case`

stdout:

```diff
- SC-73 random sequence seed 73 -> ok some(yes) none f t t f t t inner=ttt some(yes) f none f f t some(yes) returned none t t f locked returned none on some(yes) f t returned t some(yes) f t t inner=ttt some(yes) f f t f none some(yes) inner=ftt inner=ftt f f locked f f returned some(yes) returned t t threw:boom f some(yes) t t t f f none locked t f f t some(yes) t t t returned some(yes) f t f none f none t off t some(yes) none t some(yes) inner=ftt f f f f returned f f some(yes) locked some(yes) inner=ftt f t f t f t none f off t f f t returned locked threw:boom some(yes) on t f none f none locked none some(yes) f some(yes) f t some(yes) f t some(yes) some(yes) some(yes) f none none inner=ttt t t f inner=ttt t t some(yes) some(yes) some(yes) threw:boom t t f t locked some(yes) some(yes) returned off none t threw:boom f none none f returned f threw:boom some(yes) off t returned some(yes) f returned f some(yes) off t f f some(yes) t threw:boom t f t threw:boom some(yes) t off off inner=ttt on t f t f t none f t f t f t none t none on none none t none returned t none f t f f returned f t returned some(yes) f t t returned t f threw:boom some(yes) f none t t inner=ftt t returned f t t locked inner=ttt locked threw:boom f inner=ftt t inner=ftt t t t some(yes) t some(yes) t f f locked t t f some(yes) some(yes) locked locked t f f f threw:boom some(yes) t locked t threw:boom some(yes) inner=ftt some(yes) t f some(yes) f t none f some(yes) t t f t returned inner=ttt f f t f t [y,whoff,whoff,nested,y,y,whoff,y,y,y,off,whoff,y,whoff,nested,y,whon,y,nested,nested,y,y,whon,boom,y,on,off,y,y,on,y,off,y,on,y,whon,y,nested,y,y,y,nested,ifon,off,ifoff,y,on,ifon,y,boom,y,y,whon,y,y,y,off,y,ifoff,y,y,y,nested,ifoff,nested,ifon,y,y,y,boom,ifon,whoff,y,y,y,y,boom,boom,y,y,ifoff,y,y,y,y,boom,off,boom,y,whoff,y,y,nested,y,ifoff,y,whon,whon,y,off,whoff,boom,y,nested,y,nested,y,boom,nested,nested,whon,whon,whon,y,whon,y,y,on,ifon,y,y,y,y,boom,y,y,on,boom,y,nested,y,y,on,y,ifoff,whoff,nested,ifon]
+ SC-73 random sequence seed 73 -> ok some(yes) none f t t f t t inner=ttt some(yes) f none f f t some(yes) returned none t t f locked returned none on some(yes) f t returned t some(yes) f t t inner=ttt some(yes) f f t f none some(yes) inner=ftt inner=ftt f f locked f f returned some(yes) returned t t threw:boom t none f t t f f none locked t f f t some(yes) t t t returned some(yes) f t f none f none t off t some(yes) none t some(yes) inner=ftt f f f f returned f f some(yes) locked some(yes) inner=ftt f t f t f t none f off t f f t returned locked threw:boom none off f t some(yes) t none locked none some(yes) f some(yes) f t some(yes) f t some(yes) some(yes) some(yes) f none none inner=ttt t t f inner=ttt t t some(yes) some(yes) some(yes) threw:boom f f f t locked some(yes) some(yes) returned off none t threw:boom t some(yes) some(yes) t threw:boom f threw:boom none on f threw:boom some(yes) f returned f some(yes) off t f f some(yes) t threw:boom f f t threw:boom none f on on inner=ftt on t f t f t none f t f t f t none t none on none none t none returned t none f t f f returned f t returned some(yes) f t t returned t f threw:boom none t some(yes) f f inner=ftt t returned f t t locked inner=ttt locked threw:boom t inner=ftt t inner=ftt t t t some(yes) t some(yes) t f f locked t t f some(yes) some(yes) locked locked t f f f threw:boom none f locked f threw:boom none inner=ttt some(yes) t f some(yes) f t none f some(yes) t t f t returned inner=ttt f f t f t [y,whoff,whoff,nested,y,y,whoff,y,y,y,off,whoff,y,whoff,nested,y,whon,y,nested,nested,y,y,whon,boom,ifoff,on,off,y,y,on,y,off,y,on,y,whon,y,nested,y,y,y,nested,ifon,off,ifoff,y,on,ifon,y,boom,y,whoff,y,on,y,y,y,off,y,ifoff,y,y,y,nested,ifoff,nested,ifon,y,y,y,boom,whoff,y,y,y,y,boom,y,y,ifon,boom,boom,y,boom,y,y,y,y,boom,off,boom,y,y,nested,y,ifoff,y,whon,whon,y,off,whoff,boom,y,nested,y,nested,y,boom,on,nested,nested,whon,whon,whon,y,whon,y,y,on,ifon,y,y,y,y,boom,y,boom,nested,y,y,on,y,ifoff,whoff,nested,ifon]
```
