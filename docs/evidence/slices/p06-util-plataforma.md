# Evidencia del slice P06 (UTIL-PLATAFORMA), 2026-11-18

Filas: AC-UTIL-020, 021, 023, 024, 025, 026, 027, 028. Casos nuevos al final de la familia `util-core`: WIN-01, TS-UTC-01/02, TS-LOCAL-01/02, ID-01/02, CK-01..05, SW-01..07, UN-01/02, MP-EXH = uc-0100..uc-0121 (UN-03 y MP-NL están en el oráculo y no se graban: Q-UTIL-19 y Q-UTIL-8 (c)).
El oráculo fija `user.timezone=GMT+05:30` antes de cargar Helpers, de modo que TS-LOCAL no depende del host (las corridas alternaron TZ=Asia/Tokyo y America/New_York).

## Determinismo del oráculo (12 corridas por caso: salida idéntica; cada línea = 12 corridas con un solo md5)

```
  12 97bf4a2a049118afc897862fb98b8c76  :: WIN-01 isWindows
  12 de48798365b8c39f27ad6093c09d721a  :: TS-UTC-01 currentTimeMillisToUTCString
  12 362dca189e6fe9ca6858d87963cd6464  :: TS-UTC-02 negative timestamps
  12 607bc4d4ca54b7e5965b9dfae30e6502  :: TS-LOCAL-01 timestamp small values
  12 84dbfa409e116d876d6ab57cce1b09ae  :: TS-LOCAL-02 timestamp calendar edges
  12 abca4babe24bf8b8452f7fc8fc6f1bae  :: ID-01 compareIdentityHash invariants
  12 84c67c135aa14ae551f1dc852c34f340  :: ID-02 identityHashComparator with tie-break
  12 67516f1184f7826a9a26721ec9828e64  :: CK-01 NanoClock
  12 4fe8613d3d27d47b0cde3b6fd495ac8c  :: CK-02 ScheduledClock increments by one until maxIncrement
  12 f5004b4552290785422a12abec918d5b  :: CK-03 ScheduledClock maxIncrement boundaries
  12 29b7584c445159e895de376d679f7136  :: CK-04 ScheduledClock update resumes the increments
  12 a5ad92cc9f124d8719fc69691fd16f5f  :: CK-05 Clock extension selection
  12 1f97684632edc4dac950c07c21e90453  :: SW-01 compilePattern for every byte
  12 90af36df538258be07fe9efeb2887d0a  :: SW-02 applyPattern
  12 6b87d68d239c156543693e57656a5bcb  :: SW-03 getIndex and getLastIndex
  12 55b3c7ef738d54f3ef155c3e46dc0d1b  :: SW-04 getLong getInt getShort
  12 c778d2ff88ee0bca3c90f4de8ff02584  :: SW-05 putLong putInt putShort
  12 9cb353dee6d8e7fabdc3a0b062b40451  :: SW-06 variants without MethodHandle
  12 b71ca4f2822abe16716b5093bd352ae4  :: SW-07 byte-wise getShort getInt getLong
  12 f7d25cb81e3fc84b40d8ece2ca2dc98a  :: UN-01 fastHash of ASCII text
  12 9e21f658f21807e7de2a5b316b41d26a  :: UN-02 copyUSAsciiStrToBytes of ASCII text
  12 bfb7aee49de80d0c0bd90104882c1f44  :: MP-EXH makePattern lengths 0-4
```

## ROJO

Replay con las grabaciones nuevas y el binario anterior (sin los casos en el arnés):

```
1062/1084 applicable + 1 not applicable
DIVERGES: uc-0100
DIVERGES: uc-0101
DIVERGES: uc-0102
... (22 casos divergentes: uc-0100..uc-0121)
```

El arnés con los casos y sin las funciones no compila:

```
   1 error: could not compile `rakka-actor` (example "classic-parity") due to 3 previous errors
   1 error: expected expression, found reserved keyword `gen`
   1 error: expected identifier, found reserved keyword `gen`
   1 error[E0432]: unresolved imports `rakka_actor::util::helpers::IS_WINDOWS`, `rakka_actor::util::helpers::compare_identity_hash`, `rakka_actor::util::helpers::current_time_millis_to_utc_string`, `rakka_actor::util::helpers::identity_hash_comparator`, `rakka_actor::util::helpers::timestamp_at_utc_offset`
```

## VERDE

```
1084/1084 applicable + 1 not applicable
```

Los 22 casos pasaron en cuanto compiló el código (que se escribió después de leer el contrato): son caracterización; el rojo era el arnés. `tests/platform_spec.rs` (28 tests) cubre lo que el oráculo no puede grabar: dirección como identidad, `ScheduledClock` con una fuente de tiempo manual (no retrocede, vuelta de Long.MaxValue), concurrencia y límites.

## Mutantes nuevos (sólo ellos + control)

```
| mutant | outcome | passed/total |
|---|---|---|
| utc-string-milliseconds-unpadded | DETECTED | 1082/1084 |
| scheduled-clock-increments-by-two | DETECTED | 1081/1084 |
| swar-last-index-counts-from-front | DETECTED | 1083/1084 |
| control (unmodified) | CLEAN | 1084/1084 |
```

## Gates

- `python3.12 tools/parity/replay.py`: 1084/1084 applicable + 1 not applicable
- `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `cargo deny check licenses`: sin fallos
- `cargo llvm-cov clean --workspace; tools/parity/coverage.sh`: líneas 97,52 % (92 sin cubrir de 3703), rc=0
- `python3.12 tools/package/check.py`: failures=0; `python3.12 -m unittest discover -s tools/package/tests`: 35 OK
- `check_divergence.py --check-patterns`: 48 patrones, 0 obsoletos
