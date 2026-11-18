# Evidencia del slice P66 (UTIL-PEKKOVERSION), 2026-11-18

Fila: AC-UTIL-005.1. Casos PV-01..PV-14 (uc-0134..uc-0147). Decisión: Q-CFG-5 (valor "2.0.0"). Sin decisiones nuevas.

## Determinismo del oráculo (12 corridas por caso)

```
1 distinct output(s) in 12 runs: PV-01 current version
1 distinct output(s) in 12 runs: PV-02 spec: satisfied
1 distinct output(s) in 12 runs: PV-03 spec: unsatisfied
1 distinct output(s) in 12 runs: PV-04 spec: snapshot and nonsense
1 distinct output(s) in 12 runs: PV-05 messages
1 distinct output(s) in 12 runs: PV-06 pre-release matrix
1 distinct output(s) in 12 runs: PV-07 numbers compared as text then as ints
1 distinct output(s) in 12 runs: PV-08 numbers out of int range
1 distinct output(s) in 12 runs: PV-09 strings that are not a version
1 distinct output(s) in 12 runs: PV-10 digits outside ASCII
1 distinct output(s) in 12 runs: PV-11 equal strings are never checked
1 distinct output(s) in 12 runs: PV-12 public require against the current version
1 distinct output(s) in 12 runs: PV-13 the exception
1 distinct output(s) in 12 runs: PV-14 many required versions against 2.0.0
```

## ROJO (require_against como stub que siempre devuelve Ok)

```
1102/1110 applicable + 1 not applicable
DIVERGES: uc-0136
DIVERGES: uc-0138
DIVERGES: uc-0139
DIVERGES: uc-0140
DIVERGES: uc-0141
DIVERGES: uc-0145
DIVERGES: uc-0146
DIVERGES: uc-0147
---- recorded_util_core_cases_match_pekko stdout ----

thread 'recorded_util_core_cases_match_pekko' (80403498) panicked at crates/rakka-actor/tests/recorded_pekko.rs:152:9:
assertion `left == right` failed: case uc-0136 (PV-03 spec: unsatisfied)
  left: "PV-03 spec: unsatisfied -> ok [2.5.6]/[2.5.6-RC1]=ok [2.5.6-RC2]/[2.5.6-RC1]=ok [2.5.6]/[2.5.6-M1]=ok [2.5.7-M10]/[2.5.7-M9]=ok [2.5.7-RC1]/[2.5.7-M9]=ok [2.5.6]/[3.0.0]=ok [2.5.6]/[1.0.0]=ok [2.5.6]/[2.4.19]=ok [2.5.6]/[2.5.5]=ok\
 right: "PV-03 spec: unsatisfied -> ok [2.5.6]/[2.5.6-RC1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6-RC1], but lib requires version [2.5.6]] [2.5.6-RC2]/[2.5.6-RC1]=UnsupportedPekkoVersion[Current version of Pekko is [2.5.6
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    recorded_util_core_cases_match_pekko

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.33s

error: test failed, to rerun pass `-p rakka-actor --test recorded_pekko`
```

## VERDE

```
1110/1110 applicable + 1 not applicable

running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.34s

```

PV-01, PV-02, PV-04, PV-06 (parcial), PV-09..PV-11, PV-14 parcialmente pasaban con el stub (caracterización de los caminos que devuelven ok).

## Mutantes nuevos (solo ellos + control)

```

=== control (unmodified rakka-actor): exit=1  1110/1110 applicable + 1 not applicable

| mutant | outcome | passed/total |
|---|---|---|
| pekko-version-patch-compared-as-text | DETECTED | 1108/1110 |
| pekko-version-rc-accepts-milestone | DETECTED | 1107/1110 |
| pekko-version-final-accepts-pre-release | DETECTED | 1108/1110 |
| control (unmodified) | CLEAN | 1110/1110 |

corpus sizes seen: [1110]  outcomes: {'DETECTED': 3, 'UNDETECTED': 0, 'PATCH_ERROR': 0, 'BUILD_ERROR': 0, 'HARNESS_ERROR': 0, 'TIMEOUT': 0}  detected 3/3
54 mutant patterns checked, 0 stale
```

## Gates

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s
fmt-ok
licenses ok
pekko_version.rs                  183                 2    98.91%          16                 0   100.00%         147                 1    99.32%           0                 0         -
TOTAL                            4714               108    97.71%         516                14    97.29%        3916                93    97.63%           0                 0         -
```
