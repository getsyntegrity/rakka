# App replay

Build: `$TMPDIR/pekko-mutants-7u16pnpo/target-hash-map-order-reads-the-bits-from-the-top/release/examples/classic-parity`

Passed: 2838/2887

Fallback rate: not reported by this build

## Failing cases

### dr-0061

Arguments: `deployer-case`

stdout:

```diff
- DL-62 3 overlapping prefixes of pqzzz in a shuffled order -> ok ok | ok | ok | ok | some(/a/pqz*@d2) | some(/a/pqz*@d2)
+ DL-62 3 overlapping prefixes of pqzzz in a shuffled order -> ok ok | ok | ok | ok | some(/a/pqzz*@d4) | some(/a/pqzz*@d4)
```

### dr-0062

Arguments: `deployer-case`

stdout:

```diff
- DL-63 3 overlapping prefixes of pqzzz in the reverse order -> ok ok | ok | ok | ok | some(/a/pqz*@d4) | some(/a/pqz*@d4)
+ DL-63 3 overlapping prefixes of pqzzz in the reverse order -> ok ok | ok | ok | ok | some(/a/pqzz*@d2) | some(/a/pqzz*@d2)
```

### dr-0063

Arguments: `deployer-case`

stdout:

```diff
- DL-64 5 overlapping prefixes of pqzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/pqz*@d5) | some(/a/pqz*@d5)
+ DL-64 5 overlapping prefixes of pqzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/pqzz*@d2) | some(/a/pqzz*@d2)
```

### dr-0064

Arguments: `deployer-case`

stdout:

```diff
- DL-65 5 overlapping prefixes of pqzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/pqz*@d3) | some(/a/pqz*@d3)
+ DL-65 5 overlapping prefixes of pqzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/pqzz*@d6) | some(/a/pqzz*@d6)
```

### dr-0067

Arguments: `deployer-case`

stdout:

```diff
- DL-68 3 overlapping prefixes of abczzz in a shuffled order -> ok ok | ok | ok | ok | some(/a/abc*@d2) | some(/a/abc*@d2)
+ DL-68 3 overlapping prefixes of abczzz in a shuffled order -> ok ok | ok | ok | ok | some(/a/abczz*@d3) | some(/a/abczz*@d3)
```

### dr-0068

Arguments: `deployer-case`

stdout:

```diff
- DL-69 3 overlapping prefixes of abczzz in the reverse order -> ok ok | ok | ok | ok | some(/a/abc*@d4) | some(/a/abc*@d4)
+ DL-69 3 overlapping prefixes of abczzz in the reverse order -> ok ok | ok | ok | ok | some(/a/abczz*@d3) | some(/a/abczz*@d3)
```

### dr-0075

Arguments: `deployer-case`

stdout:

```diff
- DL-76 5 overlapping prefixes of userzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/u*@d6) | some(/a/u*@d6)
+ DL-76 5 overlapping prefixes of userzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/use*@d5) | some(/a/use*@d5)
```

### dr-0076

Arguments: `deployer-case`

stdout:

```diff
- DL-77 5 overlapping prefixes of userzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/u*@d2) | some(/a/u*@d2)
+ DL-77 5 overlapping prefixes of userzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/use*@d3) | some(/a/use*@d3)
```

### dr-0077

Arguments: `deployer-case`

stdout:

```diff
- DL-78 2 overlapping prefixes of workerzzz in a shuffled order -> ok ok | ok | ok | some(/a/wo*@d3) | some(/a/wo*@d3)
+ DL-78 2 overlapping prefixes of workerzzz in a shuffled order -> ok ok | ok | ok | some(/a/worker*@d2) | some(/a/worker*@d2)
```

### dr-0078

Arguments: `deployer-case`

stdout:

```diff
- DL-79 2 overlapping prefixes of workerzzz in the reverse order -> ok ok | ok | ok | some(/a/wo*@d2) | some(/a/wo*@d2)
+ DL-79 2 overlapping prefixes of workerzzz in the reverse order -> ok ok | ok | ok | some(/a/worker*@d3) | some(/a/worker*@d3)
```

### dr-0081

Arguments: `deployer-case`

stdout:

```diff
- DL-82 5 overlapping prefixes of workerzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/workerz*@d2) | some(/a/workerz*@d2)
+ DL-82 5 overlapping prefixes of workerzzz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/work*@d5) | some(/a/work*@d5)
```

### dr-0082

Arguments: `deployer-case`

stdout:

```diff
- DL-83 5 overlapping prefixes of workerzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/workerz*@d6) | some(/a/workerz*@d6)
+ DL-83 5 overlapping prefixes of workerzzz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/work*@d3) | some(/a/work*@d3)
```

### dr-0083

Arguments: `deployer-case`

stdout:

```diff
- DL-84 2 overlapping prefixes of stzzz in a shuffled order -> ok ok | ok | ok | some(/a/st*@d2) | some(/a/st*@d2)
+ DL-84 2 overlapping prefixes of stzzz in a shuffled order -> ok ok | ok | ok | some(/a/stz*@d3) | some(/a/stz*@d3)
```

### dr-0084

Arguments: `deployer-case`

stdout:

```diff
- DL-85 2 overlapping prefixes of stzzz in the reverse order -> ok ok | ok | ok | some(/a/st*@d3) | some(/a/st*@d3)
+ DL-85 2 overlapping prefixes of stzzz in the reverse order -> ok ok | ok | ok | some(/a/stz*@d2) | some(/a/stz*@d2)
```

### dr-0085

Arguments: `deployer-case`

stdout:

```diff
- DL-86 3 overlapping prefixes of stzzz in a shuffled order -> ok ok | ok | ok | ok | some(/a/s*@d3) | some(/a/s*@d3)
+ DL-86 3 overlapping prefixes of stzzz in a shuffled order -> ok ok | ok | ok | ok | some(/a/stzz*@d2) | some(/a/stzz*@d2)
```

### dr-0086

Arguments: `deployer-case`

stdout:

```diff
- DL-87 3 overlapping prefixes of stzzz in the reverse order -> ok ok | ok | ok | ok | some(/a/s*@d3) | some(/a/s*@d3)
+ DL-87 3 overlapping prefixes of stzzz in the reverse order -> ok ok | ok | ok | ok | some(/a/stzz*@d4) | some(/a/stzz*@d4)
```

### dr-0091

Arguments: `deployer-case`

stdout:

```diff
- DL-92 3 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | some(/a/m*@d3) | some(/a/m*@d3)
+ DL-92 3 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | some(/a/mnxyz*@d4) | some(/a/mnxyz*@d4)
```

### dr-0092

Arguments: `deployer-case`

stdout:

```diff
- DL-93 3 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | some(/a/m*@d3) | some(/a/m*@d3)
+ DL-93 3 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | some(/a/mnxyz*@d2) | some(/a/mnxyz*@d2)
```

### dr-0093

Arguments: `deployer-case`

stdout:

```diff
- DL-94 5 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnx*@d5) | some(/a/mnx*@d5)
+ DL-94 5 overlapping prefixes of mnxyz in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnxyz*@d3) | some(/a/mnxyz*@d3)
```

### dr-0094

Arguments: `deployer-case`

stdout:

```diff
- DL-95 5 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnx*@d3) | some(/a/mnx*@d3)
+ DL-95 5 overlapping prefixes of mnxyz in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/mnxyz*@d5) | some(/a/mnxyz*@d5)
```

### dr-0095

Arguments: `deployer-case`

stdout:

```diff
- DL-96 2 overlapping prefixes of service in a shuffled order -> ok ok | ok | ok | some(/a/se*@d2) | some(/a/se*@d2)
+ DL-96 2 overlapping prefixes of service in a shuffled order -> ok ok | ok | ok | some(/a/ser*@d3) | some(/a/ser*@d3)
```

### dr-0096

Arguments: `deployer-case`

stdout:

```diff
- DL-97 2 overlapping prefixes of service in the reverse order -> ok ok | ok | ok | some(/a/se*@d3) | some(/a/se*@d3)
+ DL-97 2 overlapping prefixes of service in the reverse order -> ok ok | ok | ok | some(/a/ser*@d2) | some(/a/ser*@d2)
```

### dr-0099

Arguments: `deployer-case`

stdout:

```diff
- DL-100 5 overlapping prefixes of service in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/se*@d6) | some(/a/se*@d6)
+ DL-100 5 overlapping prefixes of service in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/ser*@d5) | some(/a/ser*@d5)
```

### dr-0100

Arguments: `deployer-case`

stdout:

```diff
- DL-101 5 overlapping prefixes of service in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/se*@d2) | some(/a/se*@d2)
+ DL-101 5 overlapping prefixes of service in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/ser*@d3) | some(/a/ser*@d3)
```

### dr-0103

Arguments: `deployer-case`

stdout:

```diff
- DL-104 3 overlapping prefixes of dispatcherx in a shuffled order -> ok ok | ok | ok | ok | some(/a/di*@d4) | some(/a/di*@d4)
+ DL-104 3 overlapping prefixes of dispatcherx in a shuffled order -> ok ok | ok | ok | ok | some(/a/dispa*@d3) | some(/a/dispa*@d3)
```

### dr-0104

Arguments: `deployer-case`

stdout:

```diff
- DL-105 3 overlapping prefixes of dispatcherx in the reverse order -> ok ok | ok | ok | ok | some(/a/di*@d2) | some(/a/di*@d2)
+ DL-105 3 overlapping prefixes of dispatcherx in the reverse order -> ok ok | ok | ok | ok | some(/a/dispa*@d3) | some(/a/dispa*@d3)
```

### dr-0105

Arguments: `deployer-case`

stdout:

```diff
- DL-106 5 overlapping prefixes of dispatcherx in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/dispatche*@d2) | some(/a/dispatche*@d2)
+ DL-106 5 overlapping prefixes of dispatcherx in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/dispatch*@d4) | some(/a/dispatch*@d4)
```

### dr-0106

Arguments: `deployer-case`

stdout:

```diff
- DL-107 5 overlapping prefixes of dispatcherx in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/dispatche*@d6) | some(/a/dispatche*@d6)
+ DL-107 5 overlapping prefixes of dispatcherx in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/dispatch*@d4) | some(/a/dispatch*@d4)
```

### dr-0107

Arguments: `deployer-case`

stdout:

```diff
- DL-108 2 overlapping prefixes of akkapekko in a shuffled order -> ok ok | ok | ok | some(/a/akka*@d3) | some(/a/akka*@d3)
+ DL-108 2 overlapping prefixes of akkapekko in a shuffled order -> ok ok | ok | ok | some(/a/akkapekko*@d2) | some(/a/akkapekko*@d2)
```

### dr-0108

Arguments: `deployer-case`

stdout:

```diff
- DL-109 2 overlapping prefixes of akkapekko in the reverse order -> ok ok | ok | ok | some(/a/akka*@d2) | some(/a/akka*@d2)
+ DL-109 2 overlapping prefixes of akkapekko in the reverse order -> ok ok | ok | ok | some(/a/akkapekko*@d3) | some(/a/akkapekko*@d3)
```

### dr-0111

Arguments: `deployer-case`

stdout:

```diff
- DL-112 5 overlapping prefixes of akkapekko in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/akk*@d6) | some(/a/akk*@d6)
+ DL-112 5 overlapping prefixes of akkapekko in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/akkapekko*@d3) | some(/a/akkapekko*@d3)
```

### dr-0112

Arguments: `deployer-case`

stdout:

```diff
- DL-113 5 overlapping prefixes of akkapekko in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/akk*@d2) | some(/a/akk*@d2)
+ DL-113 5 overlapping prefixes of akkapekko in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/akkapekko*@d5) | some(/a/akkapekko*@d5)
```

### dr-0113

Arguments: `deployer-case`

stdout:

```diff
- DL-114 2 overlapping prefixes of routees in a shuffled order -> ok ok | ok | ok | some(/a/routees*@d3) | some(/a/routees*@d3)
+ DL-114 2 overlapping prefixes of routees in a shuffled order -> ok ok | ok | ok | some(/a/route*@d2) | some(/a/route*@d2)
```

### dr-0114

Arguments: `deployer-case`

stdout:

```diff
- DL-115 2 overlapping prefixes of routees in the reverse order -> ok ok | ok | ok | some(/a/routees*@d2) | some(/a/routees*@d2)
+ DL-115 2 overlapping prefixes of routees in the reverse order -> ok ok | ok | ok | some(/a/route*@d3) | some(/a/route*@d3)
```

### dr-0117

Arguments: `deployer-case`

stdout:

```diff
- DL-118 5 overlapping prefixes of routees in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/routees*@d2) | some(/a/routees*@d2)
+ DL-118 5 overlapping prefixes of routees in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/route*@d4) | some(/a/route*@d4)
```

### dr-0118

Arguments: `deployer-case`

stdout:

```diff
- DL-119 5 overlapping prefixes of routees in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/routees*@d6) | some(/a/routees*@d6)
+ DL-119 5 overlapping prefixes of routees in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/route*@d4) | some(/a/route*@d4)
```

### dr-0119

Arguments: `deployer-case`

stdout:

```diff
- DL-120 2 overlapping prefixes of zeta in a shuffled order -> ok ok | ok | ok | some(/a/ze*@d3) | some(/a/ze*@d3)
+ DL-120 2 overlapping prefixes of zeta in a shuffled order -> ok ok | ok | ok | some(/a/zet*@d2) | some(/a/zet*@d2)
```

### dr-0120

Arguments: `deployer-case`

stdout:

```diff
- DL-121 2 overlapping prefixes of zeta in the reverse order -> ok ok | ok | ok | some(/a/ze*@d2) | some(/a/ze*@d2)
+ DL-121 2 overlapping prefixes of zeta in the reverse order -> ok ok | ok | ok | some(/a/zet*@d3) | some(/a/zet*@d3)
```

### dr-0121

Arguments: `deployer-case`

stdout:

```diff
- DL-122 3 overlapping prefixes of zeta in a shuffled order -> ok ok | ok | ok | ok | some(/a/zeta*@d3) | some(/a/zeta*@d3)
+ DL-122 3 overlapping prefixes of zeta in a shuffled order -> ok ok | ok | ok | ok | some(/a/zet*@d2) | some(/a/zet*@d2)
```

### dr-0122

Arguments: `deployer-case`

stdout:

```diff
- DL-123 3 overlapping prefixes of zeta in the reverse order -> ok ok | ok | ok | ok | some(/a/zeta*@d3) | some(/a/zeta*@d3)
+ DL-123 3 overlapping prefixes of zeta in the reverse order -> ok ok | ok | ok | ok | some(/a/zet*@d4) | some(/a/zet*@d4)
```

### dr-0123

Arguments: `deployer-case`

stdout:

```diff
- DL-124 4 overlapping prefixes of zeta in a shuffled order -> ok ok | ok | ok | ok | ok | some(/a/ze*@d5) | some(/a/ze*@d5)
+ DL-124 4 overlapping prefixes of zeta in a shuffled order -> ok ok | ok | ok | ok | ok | some(/a/zet*@d3) | some(/a/zet*@d3)
```

### dr-0124

Arguments: `deployer-case`

stdout:

```diff
- DL-125 4 overlapping prefixes of zeta in the reverse order -> ok ok | ok | ok | ok | ok | some(/a/ze*@d2) | some(/a/ze*@d2)
+ DL-125 4 overlapping prefixes of zeta in the reverse order -> ok ok | ok | ok | ok | ok | some(/a/zet*@d4) | some(/a/zet*@d4)
```

### dr-0125

Arguments: `deployer-case`

stdout:

```diff
- DL-126 2 overlapping prefixes of orders in a shuffled order -> ok ok | ok | ok | some(/a/order*@d2) | some(/a/order*@d2)
+ DL-126 2 overlapping prefixes of orders in a shuffled order -> ok ok | ok | ok | some(/a/orde*@d3) | some(/a/orde*@d3)
```

### dr-0126

Arguments: `deployer-case`

stdout:

```diff
- DL-127 2 overlapping prefixes of orders in the reverse order -> ok ok | ok | ok | some(/a/order*@d3) | some(/a/order*@d3)
+ DL-127 2 overlapping prefixes of orders in the reverse order -> ok ok | ok | ok | some(/a/orde*@d2) | some(/a/orde*@d2)
```

### dr-0127

Arguments: `deployer-case`

stdout:

```diff
- DL-128 3 overlapping prefixes of orders in a shuffled order -> ok ok | ok | ok | ok | some(/a/or*@d2) | some(/a/or*@d2)
+ DL-128 3 overlapping prefixes of orders in a shuffled order -> ok ok | ok | ok | ok | some(/a/orde*@d4) | some(/a/orde*@d4)
```

### dr-0128

Arguments: `deployer-case`

stdout:

```diff
- DL-129 3 overlapping prefixes of orders in the reverse order -> ok ok | ok | ok | ok | some(/a/or*@d4) | some(/a/or*@d4)
+ DL-129 3 overlapping prefixes of orders in the reverse order -> ok ok | ok | ok | ok | some(/a/orde*@d2) | some(/a/orde*@d2)
```

### dr-0129

Arguments: `deployer-case`

stdout:

```diff
- DL-130 5 overlapping prefixes of orders in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/or*@d5) | some(/a/or*@d5)
+ DL-130 5 overlapping prefixes of orders in a shuffled order -> ok ok | ok | ok | ok | ok | ok | some(/a/ord*@d3) | some(/a/ord*@d3)
```

### dr-0130

Arguments: `deployer-case`

stdout:

```diff
- DL-131 5 overlapping prefixes of orders in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/or*@d3) | some(/a/or*@d3)
+ DL-131 5 overlapping prefixes of orders in the reverse order -> ok ok | ok | ok | ok | ok | ok | some(/a/ord*@d5) | some(/a/ord*@d5)
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
