# Evidencia del slice P77 (UTIL-FRONTERAS), 2026-11-18

Filas: AC-UTIL-018.1, 018.2, 018.3. Todas quedan `partial` (Q-UTIL-27 propuesta, docs/owner-batch-02.md).

Releido contra 2.0.0 (upstream/pekko, v2.0.0): FutureConverters, OptionConverters, FunctionConverters, JavaDurationConverters, ccompat y compat.Future ya no existen (actor/src/main/mima-filters/2.0.x.backwards.excludes/{future-converters,javaapi-option,function-converters,java-duration-converters,remove-scala-2.12-support}.excludes); queda scala.jdk.* (OptionalUtil.java:21,38). Eliminado en 2.0.0 no es exclusion. Unico comportamiento propio observable: `Timeout.create(java.time.Duration)` (DurationConverters.toScala). No se leyo codigo de OpenJDK (Q-X-18): la conducta sale del oraculo.

## Determinismo del oraculo (12 corridas por caso, salida identica; sin sleeps ni reloj)

```
12 identical line(s) in 12 runs: FR-DUR-01 create(Duration.ZERO) -> ok nanos=0
12 identical line(s) in 12 runs: FR-DUR-02 create(ofNanos(1)) -> ok nanos=1
12 identical line(s) in 12 runs: FR-DUR-03 create(ofSeconds(0, 999999999)) -> ok nanos=999999999
12 identical line(s) in 12 runs: FR-DUR-04 create(ofSeconds(1)) -> ok nanos=1000000000
12 identical line(s) in 12 runs: FR-DUR-05 create(ofSeconds(86400, 5)) -> ok nanos=86400000000005
12 identical line(s) in 12 runs: FR-DUR-06 create(ofSeconds(9223372036)) -> ok nanos=9223372036000000000
12 identical line(s) in 12 runs: FR-DUR-07 create(ofSeconds(9223372037)) -> ERR IllegalArgumentException [requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)]
12 identical line(s) in 12 runs: FR-DUR-08 create(ofSeconds(9223372035, 999999999)) -> ok nanos=9223372035999999999
12 identical line(s) in 12 runs: FR-DUR-09 create(ofSeconds(9223372037, 1)) -> ERR IllegalArgumentException [Java duration PT2562047H47M17.000000001S cannot be expressed as a Scala duration]
12 identical line(s) in 12 runs: FR-DUR-10 create(ofSeconds(3600, 500000000)) -> ok nanos=3600500000000
12 identical line(s) in 12 runs: FR-DUR-11 create(ofSeconds(Long.MAX_VALUE, 999999999)) -> ERR IllegalArgumentException [Java duration PT2562047788015215H30M7.999999999S cannot be expressed as a Scala duration]
12 identical line(s) in 12 runs: FR-DUR-12 create(ofSeconds(61, 1)) -> ok nanos=61000000001
12 identical line(s) in 12 runs: FR-DUR-13 create(ofSeconds(9223372036, 854775807)) eq Timeout(Long.MaxValue ns) -> ok eq=true nanos=9223372036854775807
```

## ROJO (grabaciones de Pekko sin los casos en el arnes Rust)

```
DIVERGES: uc-0544
DIVERGES: uc-0545
DIVERGES: uc-0546
DIVERGES: uc-0547
DIVERGES: uc-0548
DIVERGES: uc-0549
thread 'recorded_util_core_cases_match_pekko' (82487665) panicked at crates/rakka-actor/tests/recorded_pekko.rs:155:32:
no Rust case for FR-DUR-01 create(Duration.ZERO)
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.52s
```

## VERDE (twin `fronteras_cases.rs`, sin cambios en codigo de produccion: caracterizacion)

```
1512/1512 applicable + 1 not applicable
```

Los 13 casos (uc-0537..uc-0549) ya eran correctos en `TryFrom<std::time::Duration> for Timeout`: el rojo era el arnes sin el caso. Registrados como caracterizacion.

## Mutantes nuevos (solo ellos + control)

```
| mutant | outcome | passed/total |
|---|---|---|
| duration-nanos-sum-wraps | DETECTED | 1511/1512 |
| duration-seconds-multiply-wraps | DETECTED | 1510/1512 |
| duration-text-minutes-unreduced | DETECTED | 1509/1512 |
| control (unmodified) | CLEAN | 1512/1512 |
corpus sizes seen: [1512]  outcomes: {'DETECTED': 3, 'UNDETECTED': 0, 'PATCH_ERROR': 0, 'BUILD_ERROR': 0, 'HARNESS_ERROR': 0, 'TIMEOUT': 0}  detected 3/3
```

`--check-patterns`: 83 mutant patterns checked, 0 stale.

## Gates

- replay: 1512/1512 applicable + 1 not applicable
- `cargo test --workspace`: todo ok; `cargo clippy --workspace --all-targets -- -D warnings`: limpio; `cargo fmt --all --check`: ok; `cargo deny check licenses`: ok
- cobertura (`tools/parity/coverage.sh`): 98.74 % de lineas (>= 95 %)
- `tools/package/check.py`: failures=0; `unittest discover -s tools/package/tests`: 35 OK
