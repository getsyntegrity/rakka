# Evidencia del slice P68 (UTIL-PRETTYDURATION), 2026-11-18

Fila: AC-UTIL-016.5. Decisión: Q-UTIL-9 (a) aceptada por el Owner (reimplementar el `%g` de `java.util.Formatter` + enum `PekkoDuration`; criterio general: paridad con 2.0.0). `PrettyDuration.scala` de 2.0.0 se leyó y se midió con el jar 2.0.0 (JDK 17) por sus forwarders Java públicos `PrettyDuration.format`. Casos PD-01..PD-19 en la familia util-core (añadidos al final), grabados como uc-0170..uc-0188. Duración finita en Rust: `Timeout` (nanosegundos con signo, Q-UTIL-1).

## Determinismo del oráculo (12 corridas por caso, 1 salida distinta)

```
PD-01 .. PD-18 distinct outputs in 12 runs: 1   (los 18 primeros; PD-19 también: 1 salida distinta en 12 corridas)
```

Sin reloj ni sleeps: los barridos PD-16/PD-17 (3000 valores cada uno) usan un xorshift64* con semilla fija, gemelo en Rust.

## ROJO (casos grabados, binario sin `PekkoDuration` ni los casos en el arnés)

```
1132/1150 applicable + 1 not applicable
DIVERGES: uc-0170 ... DIVERGES: uc-0187   (18 casos)
```

Y en Rust, con `pretty_with` en `todo!()`: `cargo test -p rakka-actor --lib pretty_duration` -> `test result: FAILED. 0 passed; 8 failed`.

## VERDE

```
1151/1151 applicable + 1 not applicable      (uc-0188 = PD-19, añadido tras el hallazgo de abajo)
cargo test -p rakka-actor --lib pretty_duration -> 9 passed
```

## Hallazgo medido (no estaba en el contrato)

Un fuzzer diferencial puntual (400000 valores, `nanos` de 1 a 2^63 con signo y precisión 1..20, JVM vs Rust; script de un solo uso, no se versiona) dio 3269 diferencias con la primera versión (dígitos más cortos del double): todas con duración negativa de 2^53 ns o más, que se muestra en ns y cuyo valor es un `double` entero. Java no imprime los dígitos más cortos de ese `double` sino los del entero exacto, redondeando HALF_UP los k dígitos bajos (k = 0 con exponente binario <= 57, 1 con 58..60, 2 con 61..62, 3 con 63). Tabla obtenida sólo de la salida del oráculo (sin leer código del JDK). Tras el ajuste: 0 diferencias en 400000 valores (la mitad con magnitudes de 2^52 a 2^63), y PD-19 queda grabado como caso.

## Mutantes nuevos (solo ellos + control)

```
| pretty-duration-rounds-half-down | DETECTED | 1141/1151 |
| pretty-duration-no-scientific-at-power | DETECTED | 1144/1151 |
| pretty-duration-shortest-digits-of-big-integers | DETECTED | 1150/1151 |
| control (unmodified) | CLEAN | 1151/1151 |
60 mutant patterns checked, 0 stale
```

## Cobertura de líneas

`util/pretty_duration.rs` 100 %, TOTAL 97.95 %.

## Fuera de la unidad / límites

- Los consumidores (AsyncDnsResolver.scala:146, DeadLetterListener.scala:99) son sólo logs y pertenecen a otras unidades.
- `precision` es `i32` como en Scala; un valor enorme pide memoria enorme (Java también); no se acota.
