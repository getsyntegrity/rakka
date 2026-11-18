# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-py4t6s5v/target-token-bucket-no-tick-at-exact-period/release/examples/classic-parity`

Passed: 1415/1417

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

### uc-0338

Arguments: `util-core-case`

stdout:

```diff
- TB-81 random sequence seed 81 -> ok ok 0 ok ok ok 5 ok 0 7 17 ok 7 27 28 48 ok 15 25 45 47 ok 29 49 69 ok ok 48 0 50 ok 69 0 74 0 79 84 0 85 87 ok 72 74 84 ok ok 0 1 3 ok 0 10 30 ok 48 ok ok 0 ok 15 16 26 ok 5 7 ok 25 ok 0 7 12 32 0 33 35 ok 40 42 47 ok 37 47 0 52 72 ok 54 74 76 ok 0 ok 76 0 ok 72 0 92 0 ok ok ok 89 94 0 99 ok 98 108 118 128 148 ok 149 159 161 ok 162 ok 5 ok 0 ok ok 5 0 25 ok 45 55 75 ok 72 ok ok 79 ok 69 79 80 85 ok 81 ok 5 6 ok 15 20 ok 20 0 ok ok ok 0 ok 0 ok 0 20 30 40 0 45 65 75 80 ok ok 5 ok 2 22 ok 42 47 ok ok ok 0 58 63 ok ok ok ok 0 0 0 0 16 ok 0 0 ok ok 15 ok 5 7 27 32 33 53 ok ok ok 29 49 0 69 70 80 ok 85 0 ok 0 ok 15 17 ok ok ok 15 0 16 ok 25 0 30 32 ok 22 23 28 ok 24 44 49 ok 46 66 ok 67 69 ok 0 0 0 ok 0 1 21 26 36 38 43 44 ok ok 0 ok ok ok 15 0 ok ok 11 ok ok ok 0 0 7 8 13 0 0 0 ok 5 ok 22 ok 21 ok ok ok 0 2 7 12 0 22 23 ok ok 21 23 25 0 ok 22 24 0 34 ok 34 44 49 ok 0 ok 50 0 52 ok 0 0 16 17 19 ok ok ok ok ok ok ok 15 35 ok 54 56 0 76 0 ok 84 94 95 ok 5 6 ok 14 16 ok ok 0 ok 2 ok 0 1 0 ok 10 ok 17 ok 17 22 42 43 ok ok ok ok ok 0 ok 45 55 ok ok 53 ok ok ok ok ok 5 ok ok 0 ok ok 0 ok ok 15 35 36 56 66 71 ok
+ TB-81 random sequence seed 81 -> ok ok 0 ok ok ok 5 ok 0 7 17 ok 7 27 28 48 ok 15 25 45 47 ok 29 49 69 ok ok 48 0 50 ok 69 0 74 0 79 84 0 85 87 ok 72 74 84 ok ok 0 1 3 ok 0 10 30 ok 48 ok ok 0 ok 15 16 26 ok 5 7 ok 25 ok 0 7 12 32 0 33 35 ok 40 42 47 ok 37 47 0 52 72 ok 54 74 76 ok 0 ok 76 0 ok 72 0 92 0 ok ok ok 89 94 0 99 ok 98 108 118 128 148 ok 149 159 161 ok 162 ok 5 ok 0 ok ok 5 0 25 ok 45 55 75 ok 72 ok ok 79 ok 69 79 80 85 ok 81 ok 5 6 ok 15 20 ok 20 0 ok ok ok 0 ok 0 ok 0 20 30 40 0 45 65 75 80 ok ok 5 ok 2 22 ok 42 47 ok ok ok 0 58 63 ok ok ok ok 0 0 0 0 16 ok 0 0 ok ok 15 ok 5 7 27 32 33 53 ok ok ok 29 49 0 69 70 80 ok 85 0 ok 0 ok 14 16 ok ok ok 15 0 16 ok 25 0 30 32 ok 22 23 28 ok 24 44 49 ok 46 66 ok 67 69 ok 0 0 0 ok 0 1 21 26 36 38 43 44 ok ok 0 ok ok ok 15 0 ok ok 11 ok ok ok 0 0 7 8 13 0 0 0 ok 5 ok 22 ok 21 ok ok ok 0 2 7 12 0 22 23 ok ok 21 23 25 0 ok 22 24 0 34 ok 34 44 49 ok 0 ok 50 0 52 ok 0 0 16 17 19 ok ok ok ok ok ok ok 15 35 ok 54 56 0 76 0 ok 84 94 95 ok 5 6 ok 14 16 ok ok 0 ok 2 ok 0 1 0 ok 10 ok 17 ok 17 22 42 43 ok ok ok ok ok 0 ok 45 55 ok ok 53 ok ok ok ok ok 5 ok ok 0 ok ok 0 ok ok 15 35 36 56 66 71 ok
```
