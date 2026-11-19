# App replay

Build: `$TMPDIR/pekko-mutants-4l9e16mc/target-hocon-number-needs-fraction-digits/release/examples/classic-parity`

Passed: 1943/2304

Fallback rate: not reported by this build

## Failing cases

### ch-0001

Arguments: `config-hocon-case`

stdout:

```diff
- FB-01 receiver scalar wins over fallback scalar -> ok 1
+ FB-01 receiver scalar wins over fallback scalar -> ERR WrongType [String: 1: a has type STRING rather than NUMBER]
```

### ch-0002

Arguments: `config-hocon-case`

stdout:

```diff
- FB-02 objects merge key by key, receiver wins on conflict -> ok 1,2,3
+ FB-02 objects merge key by key, receiver wins on conflict -> ERR WrongType [String: 1: p.a has type STRING rather than NUMBER]
```

### ch-0003

Arguments: `config-hocon-case`

stdout:

```diff
- FB-03 scalar in receiver replaces object in fallback -> ok NUMBER
+ FB-03 scalar in receiver replaces object in fallback -> ok STRING
```

### ch-0005

Arguments: `config-hocon-case`

stdout:

```diff
- FB-05 substitution is late-bound: override of referenced key is seen -> ok 2
+ FB-05 substitution is late-bound: override of referenced key is seen -> ERR WrongType [String: 1: a.derived has type STRING rather than NUMBER]
```

### ch-0006

Arguments: `config-hocon-case`

stdout:

```diff
- FB-06 resolving the fallback first freezes the substitution -> ok 1
+ FB-06 resolving the fallback first freezes the substitution -> ERR WrongType [String: 1: a.derived has type STRING rather than NUMBER]
```

### ch-0007

Arguments: `config-hocon-case`

stdout:

```diff
- FB-07 chain of three: first receiver wins -> ok 1
+ FB-07 chain of three: first receiver wins -> ERR WrongType [String: 1: x has type STRING rather than NUMBER]
```

### ch-0016

Arguments: `config-hocon-case`

stdout:

```diff
- SUB-06 += appends to an existing list -> ok [1, 2]
+ SUB-06 += appends to an existing list -> ERR WrongType [String: 1: a has type list of STRING rather than list of NUMBER]
```

### ch-0017

Arguments: `config-hocon-case`

stdout:

```diff
- SUB-07 += on a missing key creates the list -> ok [2]
+ SUB-07 += on a missing key creates the list -> ERR WrongType [String: 1: a has type list of STRING rather than list of NUMBER]
```

### ch-0025

Arguments: `config-hocon-case`

stdout:

```diff
- INC-01 include of a missing file is ignored -> ok 1
+ INC-01 include of a missing file is ignored -> ERR WrongType [String: 2: x has type STRING rather than NUMBER]
```

### ch-0027

Arguments: `config-hocon-case`

stdout:

```diff
- INC-03 file() include of a missing file is ignored -> ok 1
+ INC-03 file() include of a missing file is ignored -> ERR WrongType [String: 2: x has type STRING rather than NUMBER]
```

### ch-0028

Arguments: `config-hocon-case`

stdout:

```diff
- INC-04 classpath() include of a missing resource is ignored -> ok 1
+ INC-04 classpath() include of a missing resource is ignored -> ERR WrongType [String: 2: x has type STRING rather than NUMBER]
```

### ch-0029

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 10 ms -> ok 10
+ DUR-MS 10 ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '10']
```

### ch-0030

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 10 ms -> ok 10000000
+ DUR-NS 10 ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '10']
```

### ch-0031

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 10ms -> ok 10
+ DUR-MS 10ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '10']
```

### ch-0032

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 10ms -> ok 10000000
+ DUR-NS 10ms -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '10']
```

### ch-0033

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 10 milliseconds -> ok 10
+ DUR-MS 10 milliseconds -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '10']
```

### ch-0034

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 10 milliseconds -> ok 10000000
+ DUR-NS 10 milliseconds -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '10']
```

### ch-0035

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1 us -> ok 0
+ DUR-MS 1 us -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0036

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1 us -> ok 1000
+ DUR-NS 1 us -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0037

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1us -> ok 0
+ DUR-MS 1us -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0038

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1us -> ok 1000
+ DUR-NS 1us -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0039

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 5 ns -> ok 0
+ DUR-MS 5 ns -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0040

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 5 ns -> ok 5
+ DUR-NS 5 ns -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0041

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1 d -> ok 86400000
+ DUR-MS 1 d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0042

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1 d -> ok 86400000000000
+ DUR-NS 1 d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0043

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1d -> ok 86400000
+ DUR-MS 1d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0044

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1d -> ok 86400000000000
+ DUR-NS 1d -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0045

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 2 minutes -> ok 120000
+ DUR-MS 2 minutes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '2']
```

### ch-0046

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 2 minutes -> ok 120000000000
+ DUR-NS 2 minutes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '2']
```

### ch-0047

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 5 minutes -> ok 300000
+ DUR-MS 5 minutes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0048

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 5 minutes -> ok 300000000000
+ DUR-NS 5 minutes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0049

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 5s -> ok 5000
+ DUR-MS 5s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0050

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 5s -> ok 5000000000
+ DUR-NS 5s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0051

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 5 s -> ok 5000
+ DUR-MS 5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0052

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 5 s -> ok 5000000000
+ DUR-NS 5 s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0053

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1 h -> ok 3600000
+ DUR-MS 1 h -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0054

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1 h -> ok 3600000000000
+ DUR-NS 1 h -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0055

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 100 -> ok 100
+ DUR-MS 100 -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '100']
```

### ch-0056

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 100 -> ok 100000000
+ DUR-NS 100 -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '100']
```

### ch-0061

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS -1s -> ok -1000
+ DUR-MS -1s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1']
```

### ch-0062

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS -1s -> ok -1000000000
+ DUR-NS -1s -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1']
```

### ch-0063

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 0 -> ok 0
+ DUR-MS 0 -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '0']
```

### ch-0064

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 0 -> ok 0
+ DUR-NS 0 -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '0']
```

### ch-0065

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1 hour -> ok 3600000
+ DUR-MS 1 hour -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0066

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1 hour -> ok 3600000000000
+ DUR-NS 1 hour -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0067

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-MS 1 day -> ok 86400000
+ DUR-MS 1 day -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0068

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NS 1 day -> ok 86400000000000
+ DUR-NS 1 day -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0075

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-SCI scientific notation is accepted -> ok 1000
+ DUR-SCI scientific notation is accepted -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1e3']
```

### ch-0076

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-UNQUOTED 5 minutes -> ok 300000
+ DUR-UNQUOTED 5 minutes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '5']
```

### ch-0077

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-NUMBER bare number is milliseconds -> ok 100
+ DUR-NUMBER bare number is milliseconds -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '100']
```

### ch-0078

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-ROUND 1 ns in millis truncates toward zero -> ok 0
+ DUR-ROUND 1 ns in millis truncates toward zero -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '1']
```

### ch-0079

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-ROUND 999999 ns in millis truncates toward zero -> ok 0
+ DUR-ROUND 999999 ns in millis truncates toward zero -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '999999']
```

### ch-0080

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-ROUND -1500 us in millis truncates toward zero -> ok -1
+ DUR-ROUND -1500 us in millis truncates toward zero -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '-1500']
```

### ch-0081

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-LARGE overflow -> ok 9223372036854775807
+ DUR-LARGE overflow -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '106752']
```

### ch-0082

Arguments: `config-hocon-case`

stdout:

```diff
- DUR-LARGE 292 years in ns still fits -> ok 9223286400000000000
+ DUR-LARGE 292 years in ns still fits -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse duration number '106751']
```

### ch-0083

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 128 KiB -> ok 131072
+ SIZE 128 KiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '128']
```

### ch-0084

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 128KiB -> ok 131072
+ SIZE 128KiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '128']
```

### ch-0085

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 128 K -> ok 131072
+ SIZE 128 K -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '128']
```

### ch-0086

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 128 k -> ok 131072
+ SIZE 128 k -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '128']
```

### ch-0087

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1 MB -> ok 1000000
+ SIZE 1 MB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0088

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1MB -> ok 1000000
+ SIZE 1MB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0089

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1 kB -> ok 1000
+ SIZE 1 kB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0091

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 512 -> ok 512
+ SIZE 512 -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '512']
```

### ch-0092

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 512 B -> ok 512
+ SIZE 512 B -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '512']
```

### ch-0093

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 512 bytes -> ok 512
+ SIZE 512 bytes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '512']
```

### ch-0095

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1 GiB -> ok 1073741824
+ SIZE 1 GiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0096

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1 MiB -> ok 1048576
+ SIZE 1 MiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0097

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1 g -> ok 1073741824
+ SIZE 1 g -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0098

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 0 -> ok 0
+ SIZE 0 -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '0']
```

### ch-0099

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE -1 KiB -> ERR BadValue [String: 1: Invalid value at 'x': Attempt to construct memory size with negative number: -1024]
+ SIZE -1 KiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '-1']
```

### ch-0100

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 1 TiB -> ok 1099511627776
+ SIZE 1 TiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '1']
```

### ch-0101

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE 256 MiB -> ok 268435456
+ SIZE 256 MiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '256']
```

### ch-0107

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE-ERR 8 EiB -> ERR BadValue [String: 1: Invalid value at 'x': size-in-bytes value is out of range for a 64-bit long: '9223372036854775808']
+ SIZE-ERR 8 EiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '8']
```

### ch-0108

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE-ERR 8388608 TiB -> ERR BadValue [String: 1: Invalid value at 'x': size-in-bytes value is out of range for a 64-bit long: '9223372036854775808']
+ SIZE-ERR 8388608 TiB -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '8388608']
```

### ch-0109

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE-NUMBER bare number is bytes -> ok 4096
+ SIZE-NUMBER bare number is bytes -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '4096']
```

### ch-0110

Arguments: `config-hocon-case`

stdout:

```diff
- SIZE-MEMORY getMemorySize -> ok 4194304
+ SIZE-MEMORY getMemorySize -> ERR BadValue [String: 1: Invalid value at 'x': Could not parse size-in-bytes number '4']
```

### ch-0122

Arguments: `config-hocon-case`

stdout:

```diff
- BOOL 1 -> ERR WrongType [String: 1: x has type NUMBER rather than BOOLEAN]
+ BOOL 1 -> ERR WrongType [String: 1: x has type STRING rather than BOOLEAN]
```

### ch-0126

Arguments: `config-hocon-case`

stdout:

```diff
- TYPE-03 quoted number as int -> ok 7
+ TYPE-03 quoted number as int -> ERR WrongType [String: 1: a has type STRING rather than NUMBER]
```

### ch-0131

Arguments: `config-hocon-case`

stdout:

```diff
- TYPE-08 long beyond int -> ERR WrongType [String: 1: a has type out-of-range value 3000000000 rather than 32-bit integer]
+ TYPE-08 long beyond int -> ERR WrongType [String: 1: a has type STRING rather than NUMBER]
```

### ch-0134

Arguments: `config-hocon-case`

stdout:

```diff
- TYPE-11 getConfig on scalar -> ERR WrongType [String: 1: a has type NUMBER rather than OBJECT]
+ TYPE-11 getConfig on scalar -> ERR WrongType [String: 1: a has type STRING rather than OBJECT]
```

### ch-0136

Arguments: `config-hocon-case`

stdout:

```diff
- TYPE-13 getDouble of int -> ok 3.0
+ TYPE-13 getDouble of int -> ERR WrongType [String: 1: a has type STRING rather than NUMBER]
```

### ch-0138

Arguments: `config-hocon-case`

stdout:

```diff
- SPECIAL-02 log-dead-letters = 10 is a number -> ok NUMBER
+ SPECIAL-02 log-dead-letters = 10 is a number -> ok STRING
```

### ch-0150

Arguments: `config-hocon-case`

stdout:

```diff
- LIST-06 trailing comma allowed -> ok [1, 2]
+ LIST-06 trailing comma allowed -> ERR WrongType [String: 1: l has type list of STRING rather than list of NUMBER]
```

### ch-0155

Arguments: `config-hocon-case`

stdout:

```diff
- REF-04 direct-buffer-size = 128 KiB (line 966) -> ok 131072
+ REF-04 direct-buffer-size = 128 KiB (line 966) -> ERR BadValue [String: 1: Invalid value at 'pekko.io.tcp.direct-buffer-size': Could not parse size-in-bytes number '128']
```

### ch-0156

Arguments: `config-hocon-case`

stdout:

```diff
- REF-05 log-dead-letters-suspend-duration = 5 minutes (line 68) -> ok 300000
+ REF-05 log-dead-letters-suspend-duration = 5 minutes (line 68) -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': Could not parse duration number '5']
```

### ch-0158

Arguments: `config-hocon-case`

stdout:

```diff
- REF-07 unquoted key with hyphens -> ok 8
+ REF-07 unquoted key with hyphens -> ERR WrongType [String: 1: fork-join-executor.parallelism-min has type STRING rather than NUMBER]
```

### ch-0159

Arguments: `config-hocon-case`

stdout:

```diff
- REF-08 comment styles # and // -> ok 3
+ REF-08 comment styles # and // -> ERR WrongType [String: 3: a has type STRING rather than NUMBER]
```

### ch-0161

Arguments: `config-hocon-case`

stdout:

```diff
- REF-10 duplicate keys: later object merges, later scalar wins -> ok 1,2,2
+ REF-10 duplicate keys: later object merges, later scalar wins -> ERR WrongType [String: 1: a.x has type STRING rather than NUMBER]
```

### ch-0165

Arguments: `config-hocon-case`

stdout:

```diff
- CV-04 object where a number is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: number but got: object with keys [x]]
+ CV-04 object where a number is expected -> ok valid
```

### ch-0166

Arguments: `config-hocon-case`

stdout:

```diff
- CV-05 number where an object is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: object with keys [x] but got: number]
+ CV-05 number where an object is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: object with keys [x] but got: string]
```

### ch-0167

Arguments: `config-hocon-case`

stdout:

```diff
- CV-06 number where a list is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: list but got: number]
+ CV-06 number where a list is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: list but got: string]
```

### ch-0170

Arguments: `config-hocon-case`

stdout:

```diff
- CV-09 boolean where a number is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: number but got: boolean]
+ CV-09 boolean where a number is expected -> ok valid
```

### ch-0171

Arguments: `config-hocon-case`

stdout:

```diff
- CV-10 list where a number is expected -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: number but got: list]
+ CV-10 list where a number is expected -> ok valid
```

### ch-0172

Arguments: `config-hocon-case`

stdout:

```diff
- CV-11 several problems are reported together -> ERR ValidationFailed [String: 1: pekko.a: Wrong value type at 'pekko.a', expecting: number but got: object with keys [x], String: 1: pekko.b: No setting at 'pekko.b', expecting: number, String: 1: pekko.c: Wrong value type at 'pekko.c', expecting: object with keys [d] but got: number]
+ CV-11 several problems are reported together -> ERR ValidationFailed [String: 1: pekko.b: No setting at 'pekko.b', expecting: string, String: 1: pekko.c: Wrong value type at 'pekko.c', expecting: object with keys [d] but got: string]
```

### ch-0183

Arguments: `config-hocon-case`

stdout:

```diff
- RW-01 render of scalars, nested objects, lists, null and empty values -> ok {\u000A    # String: 1\u000A    "a" : 1,\u000A    # String: 2\u000A    "b" : "hello",\u000A    # String: 8\u000A    "bool" : true,\u000A    # String: 3\u000A    "c" : {\u000A        # String: 3\u000A        "d" : [\u000A            # String: 3\u000A            1,\u000A            # String: 3\u000A            2\u000A        ],\u000A        # String: 3\u000A        "e" : "x y"\u000A    },\u000A    # String: 7\u000A    "f" : 1.5,\u000A    # String: 4\u000A    "l" : [],\u000A    # String: 6\u000A    "n" : null,\u000A    # String: 5\u000A    "o" : {},\u000A    # String: 9\u000A    "w" : 1\u000A}\u000A
+ RW-01 render of scalars, nested objects, lists, null and empty values -> ok {\u000A    # String: 1\u000A    "a" : "1",\u000A    # String: 2\u000A    "b" : "hello",\u000A    # String: 8\u000A    "bool" : true,\u000A    # String: 3\u000A    "c" : {\u000A        # String: 3\u000A        "d" : [\u000A            # String: 3\u000A            "1",\u000A            # String: 3\u000A            "2"\u000A        ],\u000A        # String: 3\u000A        "e" : "x y"\u000A    },\u000A    # String: 7\u000A    "f" : 1.5,\u000A    # String: 4\u000A    "l" : [],\u000A    # String: 6\u000A    "n" : null,\u000A    # String: 5\u000A    "o" : {},\u000A    # String: 9\u000A    "w" : 1\u000A}\u000A
```

### ch-0186

Arguments: `config-hocon-case`

stdout:

```diff
- RW-04 render of an unresolved config -> ok {\u000A    # String: 1\u000A    "a" : 1,\u000A    # String: 2\u000A    "b" : ${a}\u000A}\u000A
+ RW-04 render of an unresolved config -> ok {\u000A    # String: 1\u000A    "a" : "1",\u000A    # String: 2\u000A    "b" : ${a}\u000A}\u000A
```

### ch-0187

Arguments: `config-hocon-case`

stdout:

```diff
- RW-05 render of a merge of two configs -> ok {\u000A    # String: 1\u000A    "a" : {\u000A        # String: 1\u000A        "x" : 1,\u000A        # String: 1\u000A        "y" : 2\u000A    },\u000A    # String: 2\u000A    "b" : 3\u000A}\u000A
+ RW-05 render of a merge of two configs -> ok {\u000A    # String: 1\u000A    "a" : {\u000A        # String: 1\u000A        "x" : "1",\u000A        # String: 1\u000A        "y" : "2"\u000A    },\u000A    # String: 2\u000A    "b" : "3"\u000A}\u000A
```

### ch-0188

Arguments: `config-hocon-case`

stdout:

```diff
- RW-06 keys that need quotes and escapes in strings -> ok {\u000A    # String: 1\u000A    "a.b" : 1,\u000A    # String: 2\u000A    "c d" : "q\"uote\nline"\u000A}\u000A
+ RW-06 keys that need quotes and escapes in strings -> ok {\u000A    # String: 1\u000A    "a.b" : "1",\u000A    # String: 2\u000A    "c d" : "q\"uote\nline"\u000A}\u000A
```

### ch-0199

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-blocking-io-dispatcher.thread-pool-executor.fixed-pool-size -> ok NUMBER 16
+ KEY actor pekko.actor.default-blocking-io-dispatcher.thread-pool-executor.fixed-pool-size -> ok STRING "16"
```

### ch-0200

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-blocking-io-dispatcher.thread-pool-executor.virtual-thread-start-number -> ok NUMBER -1
+ KEY actor pekko.actor.default-blocking-io-dispatcher.thread-pool-executor.virtual-thread-start-number -> ok STRING "-1"
```

### ch-0202

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-blocking-io-dispatcher.throughput -> ok NUMBER 1
+ KEY actor pekko.actor.default-blocking-io-dispatcher.throughput -> ok STRING "1"
```

### ch-0204

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.fair-work-distribution.threshold -> ok NUMBER 128
+ KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.fair-work-distribution.threshold -> ok STRING "128"
```

### ch-0205

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.idle-cpu-level -> ok NUMBER 5
+ KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.idle-cpu-level -> ok STRING "5"
```

### ch-0207

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.parallelism-max -> ok NUMBER 64
+ KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.parallelism-max -> ok STRING "64"
```

### ch-0208

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.parallelism-min -> ok NUMBER 4
+ KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.parallelism-min -> ok STRING "4"
```

### ch-0211

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.task-queue-size -> ok NUMBER 512
+ KEY actor pekko.actor.default-dispatcher.affinity-pool-executor.task-queue-size -> ok STRING "512"
```

### ch-0215

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.fork-join-executor.maximum-pool-size -> ok NUMBER 32767
+ KEY actor pekko.actor.default-dispatcher.fork-join-executor.maximum-pool-size -> ok STRING "32767"
```

### ch-0216

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.fork-join-executor.minimum-runnable -> ok NUMBER -1
+ KEY actor pekko.actor.default-dispatcher.fork-join-executor.minimum-runnable -> ok STRING "-1"
```

### ch-0218

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.fork-join-executor.parallelism-max -> ok NUMBER 64
+ KEY actor pekko.actor.default-dispatcher.fork-join-executor.parallelism-max -> ok STRING "64"
```

### ch-0219

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.fork-join-executor.parallelism-min -> ok NUMBER 8
+ KEY actor pekko.actor.default-dispatcher.fork-join-executor.parallelism-min -> ok STRING "8"
```

### ch-0221

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.fork-join-executor.virtual-thread-start-number -> ok NUMBER -1
+ KEY actor pekko.actor.default-dispatcher.fork-join-executor.virtual-thread-start-number -> ok STRING "-1"
```

### ch-0227

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.core-pool-size-max -> ok NUMBER 64
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.core-pool-size-max -> ok STRING "64"
```

### ch-0228

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.core-pool-size-min -> ok NUMBER 8
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.core-pool-size-min -> ok STRING "8"
```

### ch-0232

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.max-pool-size-max -> ok NUMBER 64
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.max-pool-size-max -> ok STRING "64"
```

### ch-0233

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.max-pool-size-min -> ok NUMBER 8
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.max-pool-size-min -> ok STRING "8"
```

### ch-0234

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.task-queue-size -> ok NUMBER -1
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.task-queue-size -> ok STRING "-1"
```

### ch-0236

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.thread-pool-executor.virtual-thread-start-number -> ok NUMBER -1
+ KEY actor pekko.actor.default-dispatcher.thread-pool-executor.virtual-thread-start-number -> ok STRING "-1"
```

### ch-0238

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.throughput -> ok NUMBER 5
+ KEY actor pekko.actor.default-dispatcher.throughput -> ok STRING "5"
```

### ch-0242

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-dispatcher.virtual-thread-executor.virtual-thread-start-number -> ok NUMBER -1
+ KEY actor pekko.actor.default-dispatcher.virtual-thread-executor.virtual-thread-start-number -> ok STRING "-1"
```

### ch-0243

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-mailbox.mailbox-capacity -> ok NUMBER 1000
+ KEY actor pekko.actor.default-mailbox.mailbox-capacity -> ok STRING "1000"
```

### ch-0246

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.default-mailbox.stash-capacity -> ok NUMBER -1
+ KEY actor pekko.actor.default-mailbox.stash-capacity -> ok STRING "-1"
```

### ch-0248

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment."/IO-DNS/async-dns".nr-of-instances -> ok NUMBER 1
+ KEY actor pekko.actor.deployment."/IO-DNS/async-dns".nr-of-instances -> ok STRING "1"
```

### ch-0252

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment."/IO-DNS/inet-address".nr-of-instances -> ok NUMBER 4
+ KEY actor pekko.actor.deployment."/IO-DNS/inet-address".nr-of-instances -> ok STRING "4"
```

### ch-0257

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.nr-of-instances -> ok NUMBER 1
+ KEY actor pekko.actor.deployment.default.nr-of-instances -> ok STRING "1"
```

### ch-0265

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.optimal-size-exploring-resizer.lower-bound -> ok NUMBER 1
+ KEY actor pekko.actor.deployment.default.optimal-size-exploring-resizer.lower-bound -> ok STRING "1"
```

### ch-0266

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.optimal-size-exploring-resizer.optimization-range -> ok NUMBER 16
+ KEY actor pekko.actor.deployment.default.optimal-size-exploring-resizer.optimization-range -> ok STRING "16"
```

### ch-0267

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.optimal-size-exploring-resizer.upper-bound -> ok NUMBER 10
+ KEY actor pekko.actor.deployment.default.optimal-size-exploring-resizer.upper-bound -> ok STRING "10"
```

### ch-0272

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.resizer.lower-bound -> ok NUMBER 1
+ KEY actor pekko.actor.deployment.default.resizer.lower-bound -> ok STRING "1"
```

### ch-0273

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.resizer.messages-per-resize -> ok NUMBER 10
+ KEY actor pekko.actor.deployment.default.resizer.messages-per-resize -> ok STRING "10"
```

### ch-0274

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.resizer.pressure-threshold -> ok NUMBER 1
+ KEY actor pekko.actor.deployment.default.resizer.pressure-threshold -> ok STRING "1"
```

### ch-0276

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.resizer.upper-bound -> ok NUMBER 10
+ KEY actor pekko.actor.deployment.default.resizer.upper-bound -> ok STRING "10"
```

### ch-0280

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.deployment.default.virtual-nodes-factor -> ok NUMBER 10
+ KEY actor pekko.actor.deployment.default.virtual-nodes-factor -> ok STRING "10"
```

### ch-0284

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.internal-dispatcher.fork-join-executor.minimum-runnable -> ok NUMBER -1
+ KEY actor pekko.actor.internal-dispatcher.fork-join-executor.minimum-runnable -> ok STRING "-1"
```

### ch-0286

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.internal-dispatcher.fork-join-executor.parallelism-max -> ok NUMBER 64
+ KEY actor pekko.actor.internal-dispatcher.fork-join-executor.parallelism-max -> ok STRING "64"
```

### ch-0287

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.internal-dispatcher.fork-join-executor.parallelism-min -> ok NUMBER 4
+ KEY actor pekko.actor.internal-dispatcher.fork-join-executor.parallelism-min -> ok STRING "4"
```

### ch-0288

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.internal-dispatcher.throughput -> ok NUMBER 5
+ KEY actor pekko.actor.internal-dispatcher.throughput -> ok STRING "5"
```

### ch-0337

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers."org.apache.pekko.serialization.ByteArraySerializer" -> ok NUMBER 4
+ KEY actor pekko.actor.serialization-identifiers."org.apache.pekko.serialization.ByteArraySerializer" -> ok STRING "4"
```

### ch-0338

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers."org.apache.pekko.serialization.JavaSerializer" -> ok NUMBER 1
+ KEY actor pekko.actor.serialization-identifiers."org.apache.pekko.serialization.JavaSerializer" -> ok STRING "1"
```

### ch-0339

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers.primitive-boolean -> ok NUMBER 35
+ KEY actor pekko.actor.serialization-identifiers.primitive-boolean -> ok STRING "35"
```

### ch-0340

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers.primitive-bytestring -> ok NUMBER 21
+ KEY actor pekko.actor.serialization-identifiers.primitive-bytestring -> ok STRING "21"
```

### ch-0341

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers.primitive-int -> ok NUMBER 19
+ KEY actor pekko.actor.serialization-identifiers.primitive-int -> ok STRING "19"
```

### ch-0342

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers.primitive-long -> ok NUMBER 18
+ KEY actor pekko.actor.serialization-identifiers.primitive-long -> ok STRING "18"
```

### ch-0343

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-identifiers.primitive-string -> ok NUMBER 20
+ KEY actor pekko.actor.serialization-identifiers.primitive-string -> ok STRING "20"
```

### ch-0344

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.actor.serialization-max-nesting-depth -> ok NUMBER 32
+ KEY actor pekko.actor.serialization-max-nesting-depth -> ok STRING "32"
```

### ch-0360

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.circuit-breaker.default.max-failures -> ok NUMBER 10
+ KEY actor pekko.circuit-breaker.default.max-failures -> ok STRING "10"
```

### ch-0366

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.coordinated-shutdown.exit-code -> ok NUMBER 0
+ KEY actor pekko.coordinated-shutdown.exit-code -> ok STRING "0"
```

### ch-0382

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.coordinated-shutdown.reason-overrides."org.apache.pekko.actor.CoordinatedShutdown$ClusterDowningReason$".exit-code -> ok NUMBER -1
+ KEY actor pekko.coordinated-shutdown.reason-overrides."org.apache.pekko.actor.CoordinatedShutdown$ClusterDowningReason$".exit-code -> ok STRING "-1"
```

### ch-0383

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.coordinated-shutdown.reason-overrides."org.apache.pekko.actor.CoordinatedShutdown$ClusterJoinUnsuccessfulReason$".exit-code -> ok NUMBER -1
+ KEY actor pekko.coordinated-shutdown.reason-overrides."org.apache.pekko.actor.CoordinatedShutdown$ClusterJoinUnsuccessfulReason$".exit-code -> ok STRING "-1"
```

### ch-0409

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.tcp.batch-accept-limit -> ok NUMBER 10
+ KEY actor pekko.io.tcp.batch-accept-limit -> ok STRING "10"
```

### ch-0410

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.tcp.direct-buffer-pool-limit -> ok NUMBER 1000
+ KEY actor pekko.io.tcp.direct-buffer-pool-limit -> ok STRING "1000"
```

### ch-0414

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.tcp.finish-connect-retries -> ok NUMBER 5
+ KEY actor pekko.io.tcp.finish-connect-retries -> ok STRING "5"
```

### ch-0416

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.tcp.max-channels -> ok NUMBER 256000
+ KEY actor pekko.io.tcp.max-channels -> ok STRING "256000"
```

### ch-0418

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.tcp.nr-of-selectors -> ok NUMBER 1
+ KEY actor pekko.io.tcp.nr-of-selectors -> ok STRING "1"
```

### ch-0420

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.tcp.selector-association-retries -> ok NUMBER 10
+ KEY actor pekko.io.tcp.selector-association-retries -> ok STRING "10"
```

### ch-0425

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp-connected.direct-buffer-pool-limit -> ok NUMBER 1000
+ KEY actor pekko.io.udp-connected.direct-buffer-pool-limit -> ok STRING "1000"
```

### ch-0428

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp-connected.max-channels -> ok NUMBER 4096
+ KEY actor pekko.io.udp-connected.max-channels -> ok STRING "4096"
```

### ch-0429

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp-connected.nr-of-selectors -> ok NUMBER 1
+ KEY actor pekko.io.udp-connected.nr-of-selectors -> ok STRING "1"
```

### ch-0430

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp-connected.receive-throughput -> ok NUMBER 3
+ KEY actor pekko.io.udp-connected.receive-throughput -> ok STRING "3"
```

### ch-0432

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp-connected.selector-association-retries -> ok NUMBER 10
+ KEY actor pekko.io.udp-connected.selector-association-retries -> ok STRING "10"
```

### ch-0436

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp.direct-buffer-pool-limit -> ok NUMBER 1000
+ KEY actor pekko.io.udp.direct-buffer-pool-limit -> ok STRING "1000"
```

### ch-0439

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp.max-channels -> ok NUMBER 4096
+ KEY actor pekko.io.udp.max-channels -> ok STRING "4096"
```

### ch-0440

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp.nr-of-selectors -> ok NUMBER 1
+ KEY actor pekko.io.udp.nr-of-selectors -> ok STRING "1"
```

### ch-0441

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp.receive-throughput -> ok NUMBER 3
+ KEY actor pekko.io.udp.receive-throughput -> ok STRING "3"
```

### ch-0443

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.io.udp.selector-association-retries -> ok NUMBER 10
+ KEY actor pekko.io.udp.selector-association-retries -> ok STRING "10"
```

### ch-0452

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.log-dead-letters -> ok NUMBER 10
+ KEY actor pekko.log-dead-letters -> ok STRING "10"
```

### ch-0465

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor pekko.scheduler.ticks-per-wheel -> ok NUMBER 512
+ KEY actor pekko.scheduler.ticks-per-wheel -> ok STRING "512"
```

### ch-0473

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.actor.serialization-identifiers."org.apache.pekko.actor.typed.internal.MiscMessageSerializer" -> ok NUMBER 24
+ KEY actor-typed pekko.actor.serialization-identifiers."org.apache.pekko.actor.typed.internal.MiscMessageSerializer" -> ok STRING "24"
```

### ch-0474

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.actor.serialization-identifiers."org.apache.pekko.actor.typed.internal.receptionist.ServiceKeySerializer" -> ok NUMBER 26
+ KEY actor-typed pekko.actor.serialization-identifiers."org.apache.pekko.actor.typed.internal.receptionist.ServiceKeySerializer" -> ok STRING "26"
```

### ch-0480

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.actor.typed.restart-stash-capacity -> ok NUMBER 1000
+ KEY actor-typed pekko.actor.typed.restart-stash-capacity -> ok STRING "1000"
```

### ch-0482

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.reliable-delivery.consumer-controller.flow-control-window -> ok NUMBER 50
+ KEY actor-typed pekko.reliable-delivery.consumer-controller.flow-control-window -> ok STRING "50"
```

### ch-0489

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.reliable-delivery.producer-controller.durable-queue.retry-attempts -> ok NUMBER 10
+ KEY actor-typed pekko.reliable-delivery.producer-controller.durable-queue.retry-attempts -> ok STRING "10"
```

### ch-0490

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.reliable-delivery.work-pulling.producer-controller.buffer-size -> ok NUMBER 1000
+ KEY actor-typed pekko.reliable-delivery.work-pulling.producer-controller.buffer-size -> ok STRING "1000"
```

### ch-0494

Arguments: `config-hocon-case`

stdout:

```diff
- KEY actor-typed pekko.reliable-delivery.work-pulling.producer-controller.durable-queue.retry-attempts -> ok NUMBER 10
+ KEY actor-typed pekko.reliable-delivery.work-pulling.producer-controller.durable-queue.retry-attempts -> ok STRING "10"
```

### ch-0498

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e-7 -> ok 1.0E-7
+ NUM-RENDER 1e-7 -> ok "1e-7"
```

### ch-0500

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -0 -> ok 0
+ NUM-RENDER -0 -> ok "-0"
```

### ch-0501

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 010 -> ok 10
+ NUM-RENDER 010 -> ok "010"
```

### ch-0502

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e400 -> ok Infinity
+ NUM-RENDER 1e400 -> ok "1e400"
```

### ch-0504

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 5e18 -> ok 5000000000000000000
+ NUM-RENDER 5e18 -> ok "5e18"
```

### ch-0505

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e18 -> ok 1000000000000000000
+ NUM-RENDER 1e18 -> ok "1e18"
```

### ch-0509

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1E5 -> ok 100000
+ NUM-RENDER 1E5 -> ok "1E5"
```

### ch-0510

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e2 -> ok 100
+ NUM-RENDER 1e2 -> ok "1e2"
```

### ch-0511

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 123456789012345678 -> ok 123456789012345678
+ NUM-RENDER 123456789012345678 -> ok "123456789012345678"
```

### ch-0514

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e+2 -> ok 100
+ NUM-RENDER 1e+2 -> ERR Parse [String: 1: Reserved character '+' is not allowed outside quotes]
```

### ch-0516

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 9223372036854775807 -> ok 9223372036854775807
+ NUM-RENDER 9223372036854775807 -> ok "9223372036854775807"
```

### ch-0517

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER 1e19 -> ok 1.0E19
+ NUM-RENDER 1e19 -> ok "1e19"
```

### ch-0518

Arguments: `config-hocon-case`

stdout:

```diff
- NUM-RENDER -1e19 -> ok -1.0E19
+ NUM-RENDER -1e19 -> ok "-1e19"
```

### ch-0529

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1. -> ok NUMBER 1 1.
+ NUMLEX 1. -> ok STRING "1." 1.
```

### ch-0531

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX -.5 -> ok NUMBER -0.5 -.5
+ NUMLEX -.5 -> ok STRING "-.5" -.5
```

### ch-0532

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1.e5 -> ok NUMBER 100000 1.e5
+ NUMLEX 1.e5 -> ok STRING "1.e5" 1.e5
```

### ch-0533

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1e5 -> ok NUMBER 100000 1e5
+ NUMLEX 1e5 -> ok STRING "1e5" 1e5
```

### ch-0534

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1E5 -> ok NUMBER 100000 1E5
+ NUMLEX 1E5 -> ok STRING "1E5" 1E5
```

### ch-0537

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1e+5 -> ok NUMBER 100000 1e+5
+ NUMLEX 1e+5 -> ok error Parse
```

### ch-0538

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX -1 -> ok NUMBER -1 -1
+ NUMLEX -1 -> ok STRING "-1" -1
```

### ch-0541

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 00 -> ok NUMBER 0 00
+ NUMLEX 00 -> ok STRING "00" 00
```

### ch-0542

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 007 -> ok NUMBER 7 007
+ NUMLEX 007 -> ok STRING "007" 007
```

### ch-0549

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 9223372036854775807 -> ok NUMBER 9223372036854775807 9223372036854775807
+ NUMLEX 9223372036854775807 -> ok STRING "9223372036854775807" 9223372036854775807
```

### ch-0551

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX -9223372036854775808 -> ok NUMBER -9223372036854775808 -9223372036854775808
+ NUMLEX -9223372036854775808 -> ok STRING "-9223372036854775808" -9223372036854775808
```

### ch-0561

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1e400 -> ok NUMBER Infinity 1e400
+ NUMLEX 1e400 -> ok STRING "1e400" 1e400
```

### ch-0562

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1e-400 -> ok NUMBER 0 1e-400
+ NUMLEX 1e-400 -> ok STRING "1e-400" 1e-400
```

### ch-0567

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX 1E+5 -> ok NUMBER 100000 1E+5
+ NUMLEX 1E+5 -> ok error Parse
```

### ch-0568

Arguments: `config-hocon-case`

stdout:

```diff
- NUMLEX -0 -> ok NUMBER 0 -0
+ NUMLEX -0 -> ok STRING "-0" -0
```

### cs-0001

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEFAULT the reference configuration -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEFAULT the reference configuration -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0002

Arguments: `config-settings-case`

stdout:

```diff
- SET-A9 dispatcher and mailbox keys of ConfigSpec -> ok type=Dispatcher executor=default-executor shutdown-timeout=1000 throughput=5 throughput-deadline-time=0 attempt-teamwork=true fallback=fork-join-executor fj-parallelism-min=8 fj-parallelism-factor=1.0 fj-parallelism-max=64 fj-task-peeking-mode=FIFO tp-keep-alive-time=60000 tp-core-pool-size-factor=3.0 tp-max-pool-size-factor=3.0 tp-task-queue-size=-1 tp-task-queue-type=linked tp-allow-core-timeout=true tp-fixed-pool-size=off mailbox-capacity=1000 mailbox-push-timeout-time=10000 mailbox-type=org.apache.pekko.dispatch.UnboundedMailbox
+ SET-A9 dispatcher and mailbox keys of ConfigSpec -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0003

Arguments: `config-settings-case`

stdout:

```diff
- SET-EXT default library-extensions and extensions -> ok library-extensions=[org.apache.pekko.serialization.SerializationExtension$, org.apache.pekko.actor.typed.internal.adapter.ActorSystemAdapter$LoadTypedExtensions] extensions=[]
+ SET-EXT default library-extensions and extensions -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0004

Arguments: `config-settings-case`

stdout:

```diff
- SET-EXT library-extensions += and extensions set by the application -> ok library-extensions=[org.apache.pekko.serialization.SerializationExtension$, org.apache.pekko.actor.typed.internal.adapter.ActorSystemAdapter$LoadTypedExtensions, my.Ext] extensions=[a.B, c.D$]
+ SET-EXT library-extensions += and extensions set by the application -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0005

Arguments: `config-settings-case`

stdout:

```diff
- SET-EXT library-extensions replaced by the application -> ok library-extensions=[only.One]
+ SET-EXT library-extensions replaced by the application -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0006

Arguments: `config-settings-case`

stdout:

```diff
- SET-EXT extensions of a wrong type -> ERR ValidationFailed [String: 1: pekko.extensions: Wrong value type at 'pekko.extensions', expecting: list but got: number]
+ SET-EXT extensions of a wrong type -> ERR ValidationFailed [String: 1: pekko.extensions: Wrong value type at 'pekko.extensions', expecting: list but got: string]
```

### cs-0007

Arguments: `config-settings-case`

stdout:

```diff
- SET-PROVIDER default provider key -> ok local
+ SET-PROVIDER default provider key -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0008

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = on -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0009

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = off -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0010

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = true -> ERR ValidationFailed [String: 1: pekko.log-dead-letters: Wrong value type at 'pekko.log-dead-letters', expecting: number but got: boolean]
+ SET-LDL log-dead-letters = true -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0011

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = false -> ERR ValidationFailed [String: 1: pekko.log-dead-letters: Wrong value type at 'pekko.log-dead-letters', expecting: number but got: boolean]
+ SET-LDL log-dead-letters = false -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0012

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = ON -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = ON -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0013

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = Off -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = Off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0014

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = TRUE -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = TRUE -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0015

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = False -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = False -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0016

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = "ON" -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = "ON" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0017

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = "off" -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = "off" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0018

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = 10 -> ok LogDeadLetters=10
+ SET-LDL log-dead-letters = 10 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0019

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = 0 -> ok LogDeadLetters=0
+ SET-LDL log-dead-letters = 0 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0020

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = 5 -> ok LogDeadLetters=5
+ SET-LDL log-dead-letters = 5 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0021

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = -1 -> ok LogDeadLetters=-1
+ SET-LDL log-dead-letters = -1 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0022

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = 2147483647 -> ok LogDeadLetters=2147483647
+ SET-LDL log-dead-letters = 2147483647 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0023

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = "5" -> ok LogDeadLetters=5
+ SET-LDL log-dead-letters = "5" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0024

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = yes -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
+ SET-LDL log-dead-letters = yes -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0025

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = no -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
+ SET-LDL log-dead-letters = no -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0026

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = 1.5 -> ok LogDeadLetters=1
+ SET-LDL log-dead-letters = 1.5 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0027

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = abc -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
+ SET-LDL log-dead-letters = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0028

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = 3000000000 -> ERR WrongType [String: 1: pekko.log-dead-letters has type out-of-range value 3000000000 rather than 32-bit integer]
+ SET-LDL log-dead-letters = 3000000000 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0029

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = "" -> ERR WrongType [String: 1: pekko.log-dead-letters has type STRING rather than NUMBER]
+ SET-LDL log-dead-letters = "" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0030

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDL log-dead-letters = [1] -> ERR ValidationFailed [String: 1: pekko.log-dead-letters: Wrong value type at 'pekko.log-dead-letters', expecting: number but got: list]
+ SET-LDL log-dead-letters = [1] -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0031

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = infinite -> ok LogDeadLettersSuspendDuration=Inf
+ SET-LDSD log-dead-letters-suspend-duration = infinite -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0032

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = Infinite -> ok LogDeadLettersSuspendDuration=Inf
+ SET-LDSD log-dead-letters-suspend-duration = Infinite -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0033

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = INFINITE -> ok LogDeadLettersSuspendDuration=Inf
+ SET-LDSD log-dead-letters-suspend-duration = INFINITE -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0034

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = inf -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': No number in duration value 'inf']
+ SET-LDSD log-dead-letters-suspend-duration = inf -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0035

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 5s -> ok LogDeadLettersSuspendDuration=5000ms
+ SET-LDSD log-dead-letters-suspend-duration = 5s -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0036

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 1 minute -> ok LogDeadLettersSuspendDuration=60000ms
+ SET-LDSD log-dead-letters-suspend-duration = 1 minute -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0037

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 0 -> ok LogDeadLettersSuspendDuration=0ms
+ SET-LDSD log-dead-letters-suspend-duration = 0 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0038

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 100 -> ok LogDeadLettersSuspendDuration=100ms
+ SET-LDSD log-dead-letters-suspend-duration = 100 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0039

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = -1s -> ok LogDeadLettersSuspendDuration=-1000ms
+ SET-LDSD log-dead-letters-suspend-duration = -1s -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0040

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = off -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': No number in duration value 'off']
+ SET-LDSD log-dead-letters-suspend-duration = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0041

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = unlimited -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': No number in duration value 'unlimited']
+ SET-LDSD log-dead-letters-suspend-duration = unlimited -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0042

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 1.5s -> ok LogDeadLettersSuspendDuration=1500ms
+ SET-LDSD log-dead-letters-suspend-duration = 1.5s -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0043

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 5 foo -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': Could not parse time unit 'foo' (try ns, us, ms, s, m, h, d)]
+ SET-LDSD log-dead-letters-suspend-duration = 5 foo -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0044

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 106752 d -> ok LogDeadLettersSuspendDuration=9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = 106752 d -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0045

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 106751 d -> ok LogDeadLettersSuspendDuration=9223286400000ms
+ SET-LDSD log-dead-letters-suspend-duration = 106751 d -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0046

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 9223372036854775807 d -> ok LogDeadLettersSuspendDuration=9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = 9223372036854775807 d -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0047

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 9223372036854775808 d -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': Could not parse duration number '9223372036854775808']
+ SET-LDSD log-dead-letters-suspend-duration = 9223372036854775808 d -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0048

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = -9223372036854775808 d -> ok LogDeadLettersSuspendDuration=-9223372036854ms
+ SET-LDSD log-dead-letters-suspend-duration = -9223372036854775808 d -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0049

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 1ns -> ok LogDeadLettersSuspendDuration=0ms
+ SET-LDSD log-dead-letters-suspend-duration = 1ns -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0050

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 999999ns -> ok LogDeadLettersSuspendDuration=0ms
+ SET-LDSD log-dead-letters-suspend-duration = 999999ns -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0051

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = "infinite" -> ok LogDeadLettersSuspendDuration=Inf
+ SET-LDSD log-dead-letters-suspend-duration = "infinite" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0052

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = "" -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': No number in duration value '']
+ SET-LDSD log-dead-letters-suspend-duration = "" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0053

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = [1] -> ERR WrongType [String: 1: pekko.log-dead-letters-suspend-duration has type LIST rather than STRING]
+ SET-LDSD log-dead-letters-suspend-duration = [1] -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0054

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = on -> ERR BadValue [String: 1: Invalid value at 'pekko.log-dead-letters-suspend-duration': No number in duration value 'on']
+ SET-LDSD log-dead-letters-suspend-duration = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0055

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDSD log-dead-letters-suspend-duration = 2562047 h -> ok LogDeadLettersSuspendDuration=9223369200000ms
+ SET-LDSD log-dead-letters-suspend-duration = 2562047 h -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0056

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = on -> ok LogDeadLettersDuringShutdown=true
+ SET-LDDS log-dead-letters-during-shutdown = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0057

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = off -> ok LogDeadLettersDuringShutdown=false
+ SET-LDDS log-dead-letters-during-shutdown = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0058

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = true -> ok LogDeadLettersDuringShutdown=true
+ SET-LDDS log-dead-letters-during-shutdown = true -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0059

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = false -> ok LogDeadLettersDuringShutdown=false
+ SET-LDDS log-dead-letters-during-shutdown = false -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0060

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = yes -> ok LogDeadLettersDuringShutdown=true
+ SET-LDDS log-dead-letters-during-shutdown = yes -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0061

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = abc -> ERR WrongType [String: 1: pekko.log-dead-letters-during-shutdown has type STRING rather than BOOLEAN]
+ SET-LDDS log-dead-letters-during-shutdown = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0062

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = 1 -> ERR WrongType [String: 1: pekko.log-dead-letters-during-shutdown has type NUMBER rather than BOOLEAN]
+ SET-LDDS log-dead-letters-during-shutdown = 1 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0063

Arguments: `config-settings-case`

stdout:

```diff
- SET-LDDS log-dead-letters-during-shutdown = "on" -> ok LogDeadLettersDuringShutdown=true
+ SET-LDDS log-dead-letters-during-shutdown = "on" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0064

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG receive = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=true DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG receive = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0065

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG autoreceive = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=true DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG autoreceive = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0066

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG lifecycle = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG lifecycle = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0067

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG fsm = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=true DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG fsm = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0068

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG event-stream = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=true DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG event-stream = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0069

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG unhandled = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=true DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG unhandled = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0070

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG router-misconfiguration = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=true Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG router-misconfiguration = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0071

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG all seven on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=true DebugAutoReceive=true DebugLifecycle=true FsmDebugEvent=true DebugEventStream=true DebugUnhandledMessage=true DebugRouterMisconfiguration=true Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG all seven on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0072

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG receive of a wrong type -> ERR WrongType [String: 1: pekko.actor.debug.receive has type STRING rather than BOOLEAN]
+ SET-DEBUG receive of a wrong type -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0073

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG lifecycle quoted -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG lifecycle quoted -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0074

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM daemonic = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=true JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM daemonic = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0075

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM jvm-exit-on-fatal-error = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=false JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM jvm-exit-on-fatal-error = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0076

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM jvm-shutdown-hooks = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=false FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM jvm-shutdown-hooks = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0077

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM fail-mixed-versions = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=false CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM fail-mixed-versions = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0078

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM daemonic = abc -> ERR WrongType [String: 1: pekko.daemonic has type STRING rather than BOOLEAN]
+ SET-JVM daemonic = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0079

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM jvm-exit-on-fatal-error = 3 -> ERR WrongType [String: 1: pekko.jvm-exit-on-fatal-error has type NUMBER rather than BOOLEAN]
+ SET-JVM jvm-exit-on-fatal-error = 3 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0080

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM all flipped -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=true JvmExitOnFatalError=false JvmShutdownHooks=false FailMixedVersions=false CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM all flipped -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0081

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = "/opt/pekko" -> ok Home=Some(/opt/pekko)
+ SET-HOME home = "/opt/pekko" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0082

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = "" -> ok Home=None
+ SET-HOME home = "" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0083

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = " " -> ok Home=Some( )
+ SET-HOME home = " " -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0084

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = x -> ok Home=Some(x)
+ SET-HOME home = x -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0085

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = [1] -> ERR WrongType [String: 1: pekko.home has type LIST rather than STRING]
+ SET-HOME home = [1] -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0086

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = 5 -> ok Home=Some(5)
+ SET-HOME home = 5 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0087

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = null -> ERR Null [String: 1: Configuration key 'pekko.home' is set to null but expected STRING]
+ SET-HOME home = null -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0088

Arguments: `config-settings-case`

stdout:

```diff
- SET-SCHEDULER implementation = my.Sched -> ok SchedulerClass=my.Sched
+ SET-SCHEDULER implementation = my.Sched -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0089

Arguments: `config-settings-case`

stdout:

```diff
- SET-SCHEDULER implementation = [1] -> ERR WrongType [String: 1: pekko.scheduler.implementation has type LIST rather than STRING]
+ SET-SCHEDULER implementation = [1] -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0090

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = 1s -> ok CreationTimeout=1000ms
+ SET-TO creation-timeout = 1s -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '1']
```

### cs-0091

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = 2500ms -> ok CreationTimeout=2500ms
+ SET-TO creation-timeout = 2500ms -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '2500']
```

### cs-0092

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = 0 -> ok CreationTimeout=0ms
+ SET-TO creation-timeout = 0 -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '0']
```

### cs-0093

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = -1s -> ok CreationTimeout=-1000ms
+ SET-TO creation-timeout = -1s -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '-1']
```

### cs-0095

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = 1.5 minutes -> ok CreationTimeout=90000ms
+ SET-TO creation-timeout = 1.5 minutes -> ERR BadValue [String: 145: Invalid value at 'pekko.actor.unstarted-push-timeout': Could not parse duration number '10']
```

### cs-0097

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = 1 d -> ok CreationTimeout=86400000ms
+ SET-TO creation-timeout = 1 d -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '1']
```

### cs-0098

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO creation-timeout = 100 -> ok CreationTimeout=100ms
+ SET-TO creation-timeout = 100 -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '100']
```

### cs-0099

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO unstarted-push-timeout = 500ms -> ok UnstartedPushTimeout=500ms
+ SET-TO unstarted-push-timeout = 500ms -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0100

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO unstarted-push-timeout = 1 minute -> ok UnstartedPushTimeout=60000ms
+ SET-TO unstarted-push-timeout = 1 minute -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0101

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO unstarted-push-timeout = -5ms -> ok UnstartedPushTimeout=-5ms
+ SET-TO unstarted-push-timeout = -5ms -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0102

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO unstarted-push-timeout = x -> ERR BadValue [String: 1: Invalid value at 'pekko.actor.unstarted-push-timeout': No number in duration value 'x']
+ SET-TO unstarted-push-timeout = x -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0103

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO logger-startup-timeout = 7s -> ok LoggerStartTimeout=7000ms
+ SET-TO logger-startup-timeout = 7s -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0104

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO logger-startup-timeout = 1ms -> ok LoggerStartTimeout=1ms
+ SET-TO logger-startup-timeout = 1ms -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0105

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO logger-startup-timeout = 0 -> ok LoggerStartTimeout=0ms
+ SET-TO logger-startup-timeout = 0 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0106

Arguments: `config-settings-case`

stdout:

```diff
- SET-TO logger-startup-timeout = never -> ERR BadValue [String: 1: Invalid value at 'pekko.logger-startup-timeout': No number in duration value 'never']
+ SET-TO logger-startup-timeout = never -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0107

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loggers = [] -> ok Loggers=[]
+ SET-LOG loggers = [] -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0108

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loggers = two names -> ok Loggers=[a.B, c.D$E]
+ SET-LOG loggers = two names -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0109

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loggers with a repeated name -> ok Loggers=[a.B, a.B]
+ SET-LOG loggers with a repeated name -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0111

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loggers = numbers -> ok Loggers=[1, 2]
+ SET-LOG loggers = numbers -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0112

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loggers = list of objects -> ERR WrongType [String: 1: pekko.loggers has type list of OBJECT rather than list of STRING]
+ SET-LOG loggers = list of objects -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0113

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG logging-filter = x.Filter -> ok LoggingFilter=x.Filter
+ SET-LOG logging-filter = x.Filter -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0114

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG logging-filter = a number -> ok LoggingFilter=5
+ SET-LOG logging-filter = a number -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0115

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loggers-dispatcher = my-dispatcher -> ok LoggersDispatcher=my-dispatcher
+ SET-LOG loggers-dispatcher = my-dispatcher -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0116

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loglevel = DEBUG -> ok LogLevel=DEBUG
+ SET-LOG loglevel = DEBUG -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0117

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG stdout-loglevel = DEBUG -> ok StdoutLogLevel=DEBUG
+ SET-LOG stdout-loglevel = DEBUG -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0118

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loglevel = OFF -> ok LogLevel=OFF
+ SET-LOG loglevel = OFF -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0119

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG stdout-loglevel = OFF -> ok StdoutLogLevel=OFF
+ SET-LOG stdout-loglevel = OFF -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0120

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loglevel = debug -> ok LogLevel=debug
+ SET-LOG loglevel = debug -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0121

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG stdout-loglevel = debug -> ok StdoutLogLevel=debug
+ SET-LOG stdout-loglevel = debug -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0122

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loglevel = "WARNING" -> ok LogLevel=WARNING
+ SET-LOG loglevel = "WARNING" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0123

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG stdout-loglevel = "WARNING" -> ok StdoutLogLevel=WARNING
+ SET-LOG stdout-loglevel = "WARNING" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0124

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loglevel = 5 -> ok LogLevel=5
+ SET-LOG loglevel = 5 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0125

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG stdout-loglevel = 5 -> ok StdoutLogLevel=5
+ SET-LOG stdout-loglevel = 5 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0126

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG loglevel = off -> ok LogLevel=off
+ SET-LOG loglevel = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0127

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG stdout-loglevel = off -> ok StdoutLogLevel=off
+ SET-LOG stdout-loglevel = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0128

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG log-config-on-start = on -> ok LogConfigOnStart=true
+ SET-LOG log-config-on-start = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0129

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG log-config-on-start = off -> ok LogConfigOnStart=false
+ SET-LOG log-config-on-start = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0130

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG log-config-on-start = true -> ok LogConfigOnStart=true
+ SET-LOG log-config-on-start = true -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0131

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG log-config-on-start = abc -> ERR WrongType [String: 1: pekko.log-config-on-start has type STRING rather than BOOLEAN]
+ SET-LOG log-config-on-start = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0132

Arguments: `config-settings-case`

stdout:

```diff
- SET-LOG log-config-on-start = 1 -> ERR WrongType [String: 1: pekko.log-config-on-start has type NUMBER rather than BOOLEAN]
+ SET-LOG log-config-on-start = 1 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0133

Arguments: `config-settings-case`

stdout:

```diff
- SET-GUARDIAN guardian-supervisor-strategy = StoppingSupervisorStrategy -> ok SupervisorStrategyClass=org.apache.pekko.actor.StoppingSupervisorStrategy
+ SET-GUARDIAN guardian-supervisor-strategy = StoppingSupervisorStrategy -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0135

Arguments: `config-settings-case`

stdout:

```diff
- SET-SER allow-java-serialization = on -> ok AllowJavaSerialization=true
+ SET-SER allow-java-serialization = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0136

Arguments: `config-settings-case`

stdout:

```diff
- SET-SER serialize-messages = off with prefix [] -> ok SerializeAllMessages=false prefix=[]
+ SET-SER serialize-messages = off with prefix [] -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0137

Arguments: `config-settings-case`

stdout:

```diff
- SET-SER prefix with a repeated entry -> ok prefix=[a., b.]
+ SET-SER prefix with a repeated entry -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0139

Arguments: `config-settings-case`

stdout:

```diff
- SET-SER serialize-messages = abc -> ERR WrongType [String: 1: pekko.actor.serialize-messages has type STRING rather than BOOLEAN]
+ SET-SER serialize-messages = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0140

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = on run-by-actor-system-terminate = on -> ok CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true
+ SET-CS terminate-actor-system = on run-by-actor-system-terminate = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0141

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = on run-by-actor-system-terminate = off -> ok CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=false
+ SET-CS terminate-actor-system = on run-by-actor-system-terminate = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0142

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = off run-by-actor-system-terminate = on -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS terminate-actor-system = off run-by-actor-system-terminate = on -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0143

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = off run-by-actor-system-terminate = off -> ok CoordinatedShutdownTerminateActorSystem=false CoordinatedShutdownRunByActorSystemTerminate=false
+ SET-CS terminate-actor-system = off run-by-actor-system-terminate = off -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0144

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = off with the default run-by-actor-system-terminate -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS terminate-actor-system = off with the default run-by-actor-system-terminate -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0145

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS run-by-actor-system-terminate = off alone -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=false DefaultVirtualNodesFactor=10
+ SET-CS run-by-actor-system-terminate = off alone -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0146

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = abc -> ERR WrongType [String: 1: pekko.coordinated-shutdown.terminate-actor-system has type STRING rather than BOOLEAN]
+ SET-CS terminate-actor-system = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0147

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS the unsupported combination is reported before the version check -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS the unsupported combination is reported before the version check -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0148

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS the unsupported combination is reported before a bad virtual-nodes-factor -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS the unsupported combination is reported before a bad virtual-nodes-factor -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0149

Arguments: `config-settings-case`

stdout:

```diff
- SET-VNF virtual-nodes-factor = 3 -> ok DefaultVirtualNodesFactor=3
+ SET-VNF virtual-nodes-factor = 3 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0150

Arguments: `config-settings-case`

stdout:

```diff
- SET-VNF virtual-nodes-factor = 0 -> ok DefaultVirtualNodesFactor=0
+ SET-VNF virtual-nodes-factor = 0 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0151

Arguments: `config-settings-case`

stdout:

```diff
- SET-VNF virtual-nodes-factor = -2 -> ok DefaultVirtualNodesFactor=-2
+ SET-VNF virtual-nodes-factor = -2 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0152

Arguments: `config-settings-case`

stdout:

```diff
- SET-VNF virtual-nodes-factor = abc -> ERR WrongType [String: 1: pekko.actor.deployment.default.virtual-nodes-factor has type STRING rather than NUMBER]
+ SET-VNF virtual-nodes-factor = abc -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0153

Arguments: `config-settings-case`

stdout:

```diff
- SET-VNF virtual-nodes-factor = 1.5 -> ok DefaultVirtualNodesFactor=1
+ SET-VNF virtual-nodes-factor = 1.5 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0154

Arguments: `config-settings-case`

stdout:

```diff
- SET-VNF virtual-nodes-factor = "7" -> ok DefaultVirtualNodesFactor=7
+ SET-VNF virtual-nodes-factor = "7" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0155

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "2.0.0" -> ok ConfigVersion=2.0.0
+ SET-VERSION pekko.version = "2.0.0" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0156

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "1.7.1" -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [1.7.1]]
+ SET-VERSION pekko.version = "1.7.1" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0157

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "" -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version []]
+ SET-VERSION pekko.version = "" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0158

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = "2.0.0-M1" -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [2.0.0-M1]]
+ SET-VERSION pekko.version = "2.0.0-M1" -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0159

Arguments: `config-settings-case`

stdout:

```diff
- SET-VERSION pekko.version = 2 -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [2]]
+ SET-VERSION pekko.version = 2 -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0161

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID a key of the reference is an object in the application -> ERR WrongType [String: 1: pekko.loglevel has type OBJECT rather than STRING]
+ SET-INVALID a key of the reference is an object in the application -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0164

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID several problems are reported together -> ERR ValidationFailed [String: 2: pekko.loggers: Wrong value type at 'pekko.loggers', expecting: list but got: number]
+ SET-INVALID several problems are reported together -> ERR ValidationFailed [String: 2: pekko.loggers: Wrong value type at 'pekko.loggers', expecting: list but got: string]
```

### cs-0165

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID internal-dispatcher as a dispatcher name is allowed -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-INVALID internal-dispatcher as a dispatcher name is allowed -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0166

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID internal-dispatcher removed from the application -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-INVALID internal-dispatcher removed from the application -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
```

### cs-0168

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID the application adds unknown keys -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-INVALID the application adds unknown keys -> ERR BadValue [String: 127: Invalid value at 'pekko.actor.creation-timeout': Could not parse duration number '20']
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
