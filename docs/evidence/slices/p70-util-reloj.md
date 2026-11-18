# Evidencia del slice P70 (UTIL-RELOJ), 2026-11-18

Fila: AC-UTIL-016.16 (WallClock / AlwaysIncreasingClock). Sin decisiones asociadas. `WallClock.scala` de 2.0.0 leido y medido con el jar 2.0.0 (JDK 17). Casos WC-01..WC-14 en la familia util-core (al final), grabados como uc-0210..uc-0223.

## Determinismo sin reloj ni sleeps

`AlwaysIncreasingClock` lee `System.currentTimeMillis()` (no inyectable en Java) pero extiende `AtomicLong`: el oraculo fija el estado con `set` lejos en el futuro (1e18), donde cada resultado es exacto (`+1..+N`, tambien con 8 hilos: 1..4000 contiguos y distintos), o compara invariantes (estrictamente creciente; dentro de las lecturas de pared anterior y posterior; sin imprimir ningun valor de pared). El gemelo Rust usa el reloj real igual que el oraculo; las pruebas unitarias de `wall_clock.rs` usan la fuente de tiempo inyectable manejada a mano (estancada, hacia atras, envoltura).

```
WC-01 .. WC-14: distinct outputs in 12 runs = 1 (oraculo y gemelo Rust, los 14)
```

## ROJO (stub que devuelve 0)

```
1173/1186 applicable + 1 not applicable
DIVERGES: uc-0211 ... uc-0223   (13 casos; uc-0210 = estado inicial 0, caracterizacion)
```

## VERDE

```
1186/1186 applicable + 1 not applicable
cargo test -p rakka-actor --lib wall_clock -> 7 passed
```

## Hallazgos medidos (no estaban en el contrato)

- Estado inicial 0; la primera lectura es la hora de pared y el estado pasa a ser el resultado.
- Con el estado por delante, `+1` por lectura; en `Long.MAX_VALUE` envuelve a `Long.MIN_VALUE` y la lectura siguiente vuelve a la hora de pared.
- Concurrencia: 8 x 500 y 4 x 2000 lecturas con estado adelantado dan resultados distintos y contiguos; con reloj nuevo, distintos y crecientes por hilo.

## Mutantes nuevos (solo ellos + control)

```
| wall-clock-stall-repeats | DETECTED | 1176/1186 |
| wall-clock-follows-system-time | DETECTED | 1176/1186 |
| wall-clock-saturates | DETECTED | 1185/1186 |
| control (unmodified) | CLEAN | 1186/1186 |
--check-patterns: 66 mutant patterns checked, 0 stale
```

## Limites

El oraculo no puede inyectar la fuente de tiempo; el caso "ahora == ultimo" se cubre con pruebas unitarias en Rust (`an_equal_reading_is_not_repeated`).

## Compuertas

replay 1186/1186 + 1 no aplicable; `cargo test --workspace` verde; clippy `-D warnings` limpio; fmt, `cargo deny check licenses`, `tools/package/check.py` y unittest verdes; cobertura total 98.03 % (lineas), `util/wall_clock.rs` 98.35 %. La actualizacion atomica usa un bucle `compare_exchange_weak` (`fetch_update` esta deprecado en la toolchain).
