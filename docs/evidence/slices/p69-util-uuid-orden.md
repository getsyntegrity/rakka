# Evidencia del slice P69 (UTIL-UUID-ORDEN), 2026-11-18

Fila: AC-UTIL-016.14 (UUIDComparator). Decision: Q-UTIL-11 (no bloquea; se ofrecen el `Int` crudo y `Ordering`/`Ord`). El comportamiento de `java.util.UUID` (`version()`, `timestamp()`, disposicion de bits) sale del Javadoc publico y del oraculo (Q-X-18); no se leyo codigo del JDK. `UUIDComparator.scala` de 2.0.0 se leyo y se midio con el jar 2.0.0 (JDK 17). Casos UU-01..UU-21 en la familia util-core (al final), grabados como uc-0189..uc-0209.

## Determinismo del oraculo (12 corridas por caso)

```
UU-01 .. UU-21: distinct outputs in 12 runs = 1 (los 21)
```

Sin reloj ni sleeps: los barridos UU-19..UU-21 (3000 pares cada uno) usan xorshift64* con semilla fija, gemelo en Rust.

## ROJO (stub que devuelve 0 / timestamp 0)

```
1152/1172 applicable + 1 not applicable
DIVERGES: uc-0190 ... uc-0209   (20 casos; uc-0189 = version() ya correcto, caracterizacion)
cargo test -p rakka-actor --lib uuid_order -> 1 passed; 4 failed
```

## VERDE

```
1172/1172 applicable + 1 not applicable
cargo test -p rakka-actor --lib uuid_order -> 5 passed
```

## Hallazgos medidos (no estaban en el contrato)

- El `Int` crudo es la resta de mitades de 32 bits (`-15` entre version 0 y 15; `+-2147483647` entre mitades extremas del mismo signo) o `+-1` si difiere el signo.
- `timestamp()` de v1 = `time_hi(12 bits)<<48 | time_mid<<32 | time_low`; con otra version lanza `UnsupportedOperationException[Not a time-based UUID]`.
- Versiones 0, 2, 3, 5..15 se ordenan como la 4. Barridos: 0 violaciones de antisimetria y de transitividad.

## Mutantes nuevos (solo ellos + control)

```
| uuid-order-signed-halves | DETECTED | 1159/1172 |
| uuid-order-version-one-by-msb | DETECTED | 1165/1172 |
| uuid-order-timestamp-keeps-version | DETECTED | 1171/1172 |
| control (unmodified) | CLEAN | 1172/1172 |
```

## Limites

Sin parseo ni generacion de UUID (ningun consumidor del comparador los necesita; el unico consumidor, persistence-query Offset.scala, esta fuera de estos modulos). UU-17 (instancia compartida) es una constante en el gemelo Rust: no hay singleton.
