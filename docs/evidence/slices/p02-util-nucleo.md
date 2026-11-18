# Evidencia del slice P02 (UTIL-NUCLEO), 2026-11-18

## Determinismo del oráculo (12 corridas por caso, salida idéntica)

```
1 distinct output(s) in 12 runs: TO-06N eq Timeout(-1000 ms) Timeout(-1 s) -> TO-06N eq Timeout(-1000 ms) Timeout(-1 s) -> ok eq=true hashEq=true
1 distinct output(s) in 12 runs: LC \u1C90 -> LC \u1C90 -> ok [\u10D0]
1 distinct output(s) in 12 runs: LC \uA7C0 -> LC \uA7C0 -> ok [\uA7C0]
1 distinct output(s) in 12 runs: EX-TS-01 PekkoException(boom).toString -> EX-TS-01 PekkoException(boom).toString -> ok str=[org.apache.pekko.PekkoException: boom]
1 distinct output(s) in 12 runs: EX-TS-02 ConfigurationException(bad, cause).toString -> EX-TS-02 ConfigurationException(bad, cause).toString -> ok str=[org.apache.pekko.ConfigurationException: bad]
1 distinct output(s) in 12 runs: EX-TS-03 PekkoException(empty).toString -> EX-TS-03 PekkoException(empty).toString -> ok str=[org.apache.pekko.PekkoException: ]
```

## ROJO (binario sin los casos en el arnés)

```
1044/1046 applicable + 1 not applicable
DIVERGES: uc-0082
DIVERGES: uc-0083
thread 'recorded_util_core_cases_match_pekko' (79904294) panicked at crates/rakka-actor/tests/recorded_pekko.rs:151:32:
no Rust case for TO-06N eq Timeout(-1000 ms) Timeout(-1 s)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.02s
```

## VERDE

```
1046/1046 applicable + 1 not applicable
```

Los dos casos nuevos (uc-0082 TO-06N, uc-0083 LC U+1C90) ya eran correctos en la implementación: son caracterización; el rojo era el arnés sin el caso.

## Mutantes nuevos (solo ellos + control)

```
| mutant | outcome | passed/total |
|---|---|---|
| lower-case-ascii-only | DETECTED | 1038/1046 |
| timeout-max-rejected | DETECTED | 1045/1046 |
| requirement-prefix-missing | DETECTED | 1042/1046 |
| control (unmodified) | CLEAN | 1046/1046 |
corpus sizes seen: [1046]  outcomes: {'DETECTED': 3, 'UNDETECTED': 0, 'PATCH_ERROR': 0, 'BUILD_ERROR': 0, 'HARNESS_ERROR': 0, 'TIMEOUT': 0}  detected 3/3
```
