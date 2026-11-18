# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-py4t6s5v/target-index-put-duplicate-is-new/release/examples/classic-parity`

Passed: 1410/1417

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

### uc-0236

Arguments: `util-core-case`

stdout:

```diff
- IX-02 put reports whether the value was new -> ok t f t t t [1,2,3] [4] [1,2,3,4] [s1,s2]
+ IX-02 put reports whether the value was new -> ok t t t t t [1,2,3] [4] [1,2,3,4] [s1,s2]
```

### uc-0242

Arguments: `util-core-case`

stdout:

```diff
- IX-08 the comparator orders values, equal values collapse -> ok t t t f [3,2,1] some(3) some[3,2,1] []
+ IX-08 the comparator orders values, equal values collapse -> ok t t t t [3,2,1] some(3) some[3,2,1] []
```

### uc-0244

Arguments: `util-core-case`

stdout:

```diff
- IX-11 random sequence seed 11 -> ok f [] none f t none ok ok [2] none [] f t [2] t ok [4] t some[4] f [2] some[2] [] ok some[1] none f t none [2,4] [] some(2) some(4) [2,4] f some[2,4] ok [] f none none none none t t ok ok f f none [] t [1,5] [] [] some[1,5] some(2) ok none none t [] f f none none ok f f none ok t [] t some(4) ok f t ok some[2,5] ok t t t [] ok none f t none f [2] ok ok f none ok ok none ok f t f none ok [] [] none none [0] ok f t some(5) ok none f ok t none t t some[0] t t t ok f ok none none ok ok f f [1] f [] f t [0] ok ok t none t none t ok t f t ok t none [0,1] some[2,3,4] none [0,1] none some(4) some[1,4] [] none t none none t some[1] some[5] t [4] t some[0,1] f some(5) none ok ok none [] none ok [] f t ok none some(5) none some[3] t f [] none ok none t ok ok ok t some[1] t [] f [] f some(4) t t none none t some(1) none f none f ok [0,1,3] [0,1,3] none f [] f none some[0,1,3] [] none t t [] f t some(4) ok [5] f f t [] f some(5) f some(4) [] none f t none t f t some(4) none none t some[4,5] [] some(5) ok t f none ok t f some[1,5] f f f [] f ok [] none some[1] none t [1] none [1] ok none t ok f some[1] f ok none f none [] [] t t t some[2,4] none none none ok some(5) t t ok t [5] none f ok some[5] none t some[5] f [] t t f [2] t [1,3] none ok some[1,3] none f [2,3] [] t none [3] f f none f none [2] ok ok f none t some[2,3] t none some(2) [] some[3] [] ok t [] none ok ok none [] [5] t t f ok ok none ok [0,5] none none some[0] none ok t f ok t t some(5) some[0,2] ok t ok [] f none none some[2] [2] t [] t [] some[0] some[0,5] none [2,3] f f ok none f none [2,3] [] none f ok none ok f none t none ok ok none some[3] f ok ok none t none none none [] none ok none t some[1,3] t some(3) t some(3) t f f ok some[0] t none some(5) ok f ok t t [2] none ok t some[2,5] [] t none none ok [] t ok f ok [3] f some(3) some(3) f some[3] t f ok ok t [] none [] ok [3] none f [] t f ok f [] t some[3] none none none [3] f none ok ok {k1=[3] k2=[3]} [3] [k1,k2] f
+ IX-11 random sequence seed 11 -> ok f [] none f t none ok ok [2] none [] f t [2] t ok [4] t some[4] t [2] some[2] [] ok some[1] none f t none [2,4] [] some(2) some(4) [2,4] f some[2,4] ok [] f none none none none t t ok ok f t none [] t [1,5] [] [] some[1,5] some(2) ok none none t [] t f none none ok t f none ok t [] t some(4) ok f t ok some[2,5] ok t t t [] ok none f t none f [2] ok ok f none ok ok none ok f t f none ok [] [] none none [0] ok f t some(5) ok none f ok t none t t some[0] t t t ok f ok none none ok ok f f [1] f [] f t [0] ok ok t none t none t ok t f t ok t none [0,1] some[2,3,4] none [0,1] none some(4) some[1,4] [] none t none none t some[1] some[5] t [4] t some[0,1] f some(5) none ok ok none [] none ok [] f t ok none some(5) none some[3] t f [] none ok none t ok ok ok t some[1] t [] f [] f some(4) t t none none t some(1) none f none f ok [0,1,3] [0,1,3] none f [] f none some[0,1,3] [] none t t [] f t some(4) ok [5] f f t [] f some(5) f some(4) [] none f t none t f t some(4) none none t some[4,5] [] some(5) ok t f none ok t f some[1,5] f f f [] f ok [] none some[1] none t [1] none [1] ok none t ok f some[1] f ok none f none [] [] t t t some[2,4] none none none ok some(5) t t ok t [5] none t ok some[5] none t some[5] f [] t t f [2] t [1,3] none ok some[1,3] none f [2,3] [] t none [3] f f none f none [2] ok ok f none t some[2,3] t none some(2) [] some[3] [] ok t [] none ok ok none [] [5] t t t ok ok none ok [0,5] none none some[0] none ok t f ok t t some(5) some[0,2] ok t ok [] f none none some[2] [2] t [] t [] some[0] some[0,5] none [2,3] f f ok none f none [2,3] [] none f ok none ok f none t none ok ok none some[3] f ok ok none t none none none [] none ok none t some[1,3] t some(3) t some(3) t t f ok some[0] t none some(5) ok f ok t t [2] none ok t some[2,5] [] t none none ok [] t ok f ok [3] t some(3) some(3) t some[3] t f ok ok t [] none [] ok [3] none f [] t f ok f [] t some[3] none none none [3] f none ok ok {k1=[3] k2=[3]} [3] [k1,k2] f
```

### uc-0245

Arguments: `util-core-case`

stdout:

```diff
- IX-12 random sequence seed 12 -> ok none t [] [] [] ok none f f none ok [] f some(5) ok none t ok some(5) t t f t some[4,5] none t ok some(5) none ok ok t t none none none f [4,5] [] t t t some[4,5] [3] none ok [] f t t some[4] none some[0] f none f none none f some(3) [3] none none t ok none t t t [0,1] [] f f f ok ok [] t f some[4] f [0] [] ok none ok [] none f none t none f some[3] ok none none none none t f f f none f none ok none ok f none none f t none ok f ok t ok some[1,4] none t none t some[5] [] some(4) ok some(4) f some[3] [] [] none ok none [] [] ok none f t t some[3] ok t none none f ok f none none f f [] f f none [0,4] [] t f [] [0,4] t ok some[4] t t ok ok ok [] t ok f [] t [] none some[0] none none none f t t some(3) [3,4] none [] ok f ok t t ok f t f [3,4] ok [4] ok t [3,4] [] none t t t ok t none none some(0) t f [4] t some(4) some[4,5] some(4) none none [] ok f f none [] none none ok ok some(3) none ok f none ok none none [] [] ok none ok [] none f [] none none ok t none f [] ok f none f [] ok f none f some[3] [] f none f f [] ok [] f ok none [] t f [] none none t t none [] ok ok ok none none none [] f none none t t none t [] some[2,4] none none t none f t [3,5] f none some(5) t some(3) ok f some(5) some[2] ok none none t t ok [5] f some(5) some(5) some(5) [5] ok f ok none [] t f t ok none [] [0,2] none none f f [0,2] t none t none none none some(5) ok none some(3) t [0,2,5] some[0,2,5] [] [3] f [] none ok f [] t [] [] [] none f ok f [] ok [3,5] ok [] none ok f some[3] none [] f ok none none none none none none none f ok f t none none some(0) f none ok f f none f [] f t [0,3] some[0,3] none [] [] t [1] some(1) [1] ok f t f [] none [] [] f [] none [] none ok none none none f none none none f t [] [] t [] f ok f some[4] t [0] none ok ok f none [] [] t none ok [0] some[0,1] f ok [] [] t [] t none some[0] some[5] none none f ok t t [] f t t some(3) f none f ok none some(4) {k0=[5] k2=[4]} [4,5] [k0,k2] f
+ IX-12 random sequence seed 12 -> ok none t [] [] [] ok none f f none ok [] f some(5) ok none t ok some(5) t t t t some[4,5] none t ok some(5) none ok ok t t none none none f [4,5] [] t t t some[4,5] [3] none ok [] f t t some[4] none some[0] f none f none none f some(3) [3] none none t ok none t t t [0,1] [] f f f ok ok [] t f some[4] f [0] [] ok none ok [] none f none t none t some[3] ok none none none none t f f f none f none ok none ok f none none f t none ok t ok t ok some[1,4] none t none t some[5] [] some(4) ok some(4) t some[3] [] [] none ok none [] [] ok none f t t some[3] ok t none none f ok f none none f f [] f f none [0,4] [] t f [] [0,4] t ok some[4] t t ok ok ok [] t ok f [] t [] none some[0] none none none f t t some(3) [3,4] none [] ok f ok t t ok f t f [3,4] ok [4] ok t [3,4] [] none t t t ok t none none some(0) t t [4] t some(4) some[4,5] some(4) none none [] ok t t none [] none none ok ok some(3) none ok f none ok none none [] [] ok none ok [] none f [] none none ok t none f [] ok f none f [] ok f none f some[3] [] f none f f [] ok [] f ok none [] t f [] none none t t none [] ok ok ok none none none [] f none none t t none t [] some[2,4] none none t none f t [3,5] f none some(5) t some(3) ok f some(5) some[2] ok none none t t ok [5] f some(5) some(5) some(5) [5] ok f ok none [] t f t ok none [] [0,2] none none t f [0,2] t none t none none none some(5) ok none some(3) t [0,2,5] some[0,2,5] [] [3] f [] none ok f [] t [] [] [] none f ok f [] ok [3,5] ok [] none ok f some[3] none [] f ok none none none none none none none f ok f t none none some(0) f none ok f f none f [] f t [0,3] some[0,3] none [] [] t [1] some(1) [1] ok f t f [] none [] [] f [] none [] none ok none none none f none none none f t [] [] t [] f ok f some[4] t [0] none ok ok f none [] [] t none ok [0] some[0,1] f ok [] [] t [] t none some[0] some[5] none none f ok t t [] f t t some(3) f none f ok none some(4) {k0=[5] k2=[4]} [4,5] [k0,k2] f
```

### uc-0246

Arguments: `util-core-case`

stdout:

```diff
- IX-13 random sequence seed 13 -> ok f none t f none ok some[0] ok ok none f ok none f t f none t none ok ok t [5] [] some[4] ok ok [] none some[5] t [] ok ok none ok ok some(0) none f ok none f f f [0] f t t ok t none none ok none [0] f t t some[0,3] ok ok t f some[0] t ok [] ok none t ok [] none f t some[1] f f t [3] [2] some(3) some[3] t some(5) ok t f some(5) some[4] f some[5] ok t ok ok t some(5) [2,5] none ok none [] t some(5) ok [] none ok f some[2] [1] [1] some[1] none [] none f none [] none f ok none none f f f ok none f f f t f none t none none t ok t t t f t ok ok ok some[1,5] ok f [] none none none t none none [] t f none ok f [] ok [] f ok none none ok t [] t [] ok ok [] [] t [4] ok t some(4) f none some[4,5] none f none ok none [4] none none none [] t none t t some[2] some[0] t some[5] none f ok [] f f ok t t ok ok none ok [] f ok t t t [4,5] ok [1,2] some[4] none f [] none f none [] none t f some[1,2] none t f some[2] none [] t [] f [1] some(4) f f f t f f [4] none f t f [1] f t f ok ok some[0,5] f t ok t t some[0,4] t ok ok [] none none f none ok [] none none none f [] none [] ok f none ok f [] [] none t [] f none [0] f ok none t f none [] [] some[1] f ok [] t t ok some(5) t [] none [] t t none some[1,5] t f some[5] [0] [0] ok ok none t ok f f none ok none f none none none [0] none none f some[0] ok none ok ok f ok [] [] ok none none t t f [0,5] t some[2] ok f t some[3] [5] ok t ok [1] ok ok f none [] none none none none t some[1] f f f f ok [] none f none ok [] none [] [] t none none none ok t [] none f ok ok ok none none none none [] [] ok f t [4] [3] f ok some[3] t [4] none [4] f ok none t [3,5] none some(4) [4] t [] ok ok none f ok none ok [3,5] some[3,5] f none f none t [] f f none f [] ok f ok t some[0,1] [] [] ok none t [] [] ok t ok [] none t some[1] t [] f none none ok t none some[4,5] none ok t f [] t {k0=[4] k1=[2] k3=[4]} [2,4] [k0,k1,k3] f
+ IX-13 random sequence seed 13 -> ok f none t f none ok some[0] ok ok none f ok none f t f none t none ok ok t [5] [] some[4] ok ok [] none some[5] t [] ok ok none ok ok some(0) none f ok none f f f [0] f t t ok t none none ok none [0] t t t some[0,3] ok ok t t some[0] t ok [] ok none t ok [] none f t some[1] f f t [3] [2] some(3) some[3] t some(5) ok t f some(5) some[4] f some[5] ok t ok ok t some(5) [2,5] none ok none [] t some(5) ok [] none ok f some[2] [1] [1] some[1] none [] none f none [] none f ok none none f f f ok none f f f t f none t none none t ok t t t f t ok ok ok some[1,5] ok f [] none none none t none none [] t f none ok f [] ok [] f ok none none ok t [] t [] ok ok [] [] t [4] ok t some(4) f none some[4,5] none f none ok none [4] none none none [] t none t t some[2] some[0] t some[5] none f ok [] f f ok t t ok ok none ok [] f ok t t t [4,5] ok [1,2] some[4] none f [] none f none [] none t f some[1,2] none t f some[2] none [] t [] f [1] some(4) f f f t f f [4] none f t f [1] f t f ok ok some[0,5] f t ok t t some[0,4] t ok ok [] none none f none ok [] none none none f [] none [] ok f none ok f [] [] none t [] f none [0] f ok none t f none [] [] some[1] f ok [] t t ok some(5) t [] none [] t t none some[1,5] t f some[5] [0] [0] ok ok none t ok f f none ok none f none none none [0] none none f some[0] ok none ok ok f ok [] [] ok none none t t f [0,5] t some[2] ok f t some[3] [5] ok t ok [1] ok ok f none [] none none none none t some[1] f f f f ok [] none f none ok [] none [] [] t none none none ok t [] none f ok ok ok none none none none [] [] ok f t [4] [3] f ok some[3] t [4] none [4] f ok none t [3,5] none some(4) [4] t [] ok ok none f ok none ok [3,5] some[3,5] t none f none t [] f t none f [] ok f ok t some[0,1] [] [] ok none t [] [] ok t ok [] none t some[1] t [] f none none ok t none some[4,5] none ok t t [] t {k0=[4] k1=[2] k3=[4]} [2,4] [k0,k1,k3] f
```

### uc-0247

Arguments: `util-core-case`

stdout:

```diff
- IX-14 random sequence, reversed comparator -> ok none f f none ok f none [] ok none [] f none none t ok none t none t t f some(4) t f [] none some[4] [] t t [4] none none none f f some[4] none none [] none [] none [] t some[2] ok f ok ok [] [] none ok none none none ok none ok [] [] [] t [] none f none none f ok none [0] ok [] some[0] [] [] f none t ok [] none t [] [] none [5] t t some[0] some(2) none [2] t some[5] f [] none [2] f [] [] none ok t t ok f ok none ok ok t f none [] some(5) ok f ok t none ok [] none none ok some[2] t [] t some[3] none t ok ok t some(2) f none ok f ok none [2] [] [] ok [2] ok [] f none f none none ok none ok f none [] [] ok f t [] none some[2] f none some[1] f t none t ok f f ok t some(1) t f ok some[5] none some(5) some(5) none f ok none none f [] t [] [] none none none none none ok ok t t [1] none none none [1] none ok none t f some(1) none [1] f [] none ok [] f some(3) [] ok [] none ok none f ok f some(3) [1] [3,2,1] f f none f none some[1] t some(3) [3,2,1,0] f f [] ok t none [] [] none some(5) t none ok t ok [5] ok some[5] [] ok [] none none none f t some[5,1] none ok f ok [] f ok none t f [] ok some(4) none f t [4] none f some[3] ok t f f none [4,0] [] [] [] ok none ok none none [] f none t [4] some[4] [4,0] t t none none some[1] t none some[4,0] [0] ok t t [3,2] none ok t some[1] f t t t f t [] ok ok ok ok f f some[4] none ok t t ok t t t some(5) some(5) ok ok t t some(5) none none f ok t ok ok ok none ok f t [3] ok [] t some(3) some[3] none [] f none f some[3] none none none t t ok t some(4) f f some(5) some[4] t t some[5,1,0] some[5] [] f t none none ok f ok ok t ok t t f none some(0) none ok none f t none f t [0] some[1] ok none ok t t ok some[5] ok ok [] none [] none t none f [] none none f f some(2) f none none none ok f none ok [] ok none some(2) f t t none f ok ok some[5] ok [] t t none none t [5,4] ok f [] none some[5] f ok ok none none f f ok none [5] none ok some(5) {k3=[5]} [5] [k3] f
+ IX-14 random sequence, reversed comparator -> ok none f f none ok f none [] ok none [] f none none t ok none t none t t f some(4) t t [] none some[4] [] t t [4] none none none f f some[4] none none [] none [] none [] t some[2] ok f ok ok [] [] none ok none none none ok none ok [] [] [] t [] none f none none f ok none [0] ok [] some[0] [] [] f none t ok [] none t [] [] none [5] t t some[0] some(2) none [2] t some[5] f [] none [2] f [] [] none ok t t ok f ok none ok ok t f none [] some(5) ok f ok t none ok [] none none ok some[2] t [] t some[3] none t ok ok t some(2) t none ok t ok none [2] [] [] ok [2] ok [] f none f none none ok none ok f none [] [] ok f t [] none some[2] f none some[1] f t none t ok f f ok t some(1) t f ok some[5] none some(5) some(5) none f ok none none f [] t [] [] none none none none none ok ok t t [1] none none none [1] none ok none t f some(1) none [1] f [] none ok [] f some(3) [] ok [] none ok none f ok f some(3) [1] [3,2,1] f f none f none some[1] t some(3) [3,2,1,0] f f [] ok t none [] [] none some(5) t none ok t ok [5] ok some[5] [] ok [] none none none f t some[5,1] none ok f ok [] f ok none t f [] ok some(4) none f t [4] none f some[3] ok t f f none [4,0] [] [] [] ok none ok none none [] f none t [4] some[4] [4,0] t t none none some[1] t none some[4,0] [0] ok t t [3,2] none ok t some[1] t t t t f t [] ok ok ok ok f f some[4] none ok t t ok t t t some(5) some(5) ok ok t t some(5) none none f ok t ok ok ok none ok f t [3] ok [] t some(3) some[3] none [] f none f some[3] none none none t t ok t some(4) t f some(5) some[4] t t some[5,1,0] some[5] [] f t none none ok f ok ok t ok t t f none some(0) none ok none f t none f t [0] some[1] ok none ok t t ok some[5] ok ok [] none [] none t none f [] none none f f some(2) f none none none ok t none ok [] ok none some(2) f t t none f ok ok some[5] ok [] t t none none t [5,4] ok f [] none some[5] f ok ok none none f f ok none [5] none ok some(5) {k3=[5]} [5] [k3] f
```
