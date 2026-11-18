# Evidencia del slice P03 (UTIL-ESTRUCTURAS), 2026-11-18

Filas: AC-UTIL-007, 008, 009, 010, 011, 012, 013, 014, 015.1, 015.2, 016.4, 017.1..017.5. Decisiones aplicadas: Q-UTIL-10 (a), Q-UTIL-11 (a, ReentrantGuard "eliminado en 2.0.0"), Q-UTIL-14 (a para FrequencySketch con la validacion de desbordamiento de 2.0.0, (b) para el resto), Q-UTIL-15 (b) (divergencia autorizada: `MessageBufferMap.drop` de un id ausente devuelve 0; es el caso que lanza NPE en Pekko: la decision cita MessageBufferMap, no ImmutableIntMap), Q-UTIL-8 (c), Q-X-4/5/6. Fuentes de 2.0.0 leidas en cada fila.

## Diseno del oraculo

Casos nuevos en la familia util-core, al final de la lista (uc-0235..uc-0453, ids intactos para los anteriores). Los de secuencia son guiones compartidos por el oraculo Java y su gemelo Rust (`crates/rakka-actor/examples/classic-parity/structures.tsv`), un interprete por estructura que imprime un token por operacion, mas operaciones `rand SEMILLA N` con un generador xorshift identico en ambos lados (sin reloj, sin sleeps). Los concurrentes (CC-01..03, BQ-C1..C8) esperan a que el hilo se aparque (`Thread.getState` en Java, evento del lock en Rust) e imprimen invariantes. MB-DROP-ABSENT (NPE) queda en PENDING del gemelo (Q-UTIL-15 (b)); RG-01 mide que `ReentrantGuard` no esta en el jar 2.0.0.

```
218 + 1 casos nuevos x 12 corridas: oraculo y gemelo, 0 casos con mas de una salida distinta (loop12.py sobre util-core-all)
```

## ROJO (arnes sin los casos del gemelo registrados)

```
1197/1415 applicable + 1 not applicable        (218 divergentes: uc-0235..uc-0452)
cargo test -p rakka-actor --test recorded_pekko:
  no Rust case for IX-01 take and return a value ... FAILED
```

## VERDE

```
1416/1416 applicable + 1 not applicable
cargo test --workspace: 413 passed, 0 failed (specs portados de IndexSpec, SwitchSpec, TokenBucketSpec, BoundedBlockingQueueSpec, FrequencySketchSpec, FrequencyListSpec, RecencyListSpec, SegmentedRecencyListSpec, DoubleLinkedListSpec, ImmutableIntMapSpec, MessageBufferSpec, WildcardIndexSpec, TypedMultiMapSpec; dos doctests compile_fail)
cargo clippy --workspace --all-targets -- -D warnings: limpio; cargo fmt --check: limpio; cargo deny check licenses: ok
cargo llvm-cov (coverage.sh): 98.64 % lineas del crate (modulos nuevos 96.5..100 %)
python3.12 tools/package/check.py: failures=0; unittest tools/package/tests: OK
```

El gemelo coincidio con el oraculo en todos los casos a la primera: la implementacion se escribio con los specs y el codigo de 2.0.0 delante; el rojo de arriba es el del arnes sin el gemelo, y los mutantes de abajo demuestran que los casos detectan los errores probables.

## Hallazgos medidos que el contrato no recogia

- `WildcardIndex.isEmpty` ignora los hijos `prefix*`: un indice con solo `a*` esta vacio y no encuentra nada (WI-15); reinsertar `prefix*` reemplaza su subarbol (WI-16). Replicado.
- `FrequencySketch.Bits.ceilingPowerOfTwo(2^30 + 1)` desborda a `Int.MinValue` (desplazamiento de Java); `FrequencySketch(capacity = 536870913)` falla con `requirement failed: computed width (4294967296) exceeds Int.MaxValue; reduce capacity or widthMultiplier` (FS-10, FS-11).
- Mensajes de `require` de TokenBucket con prefijo `requirement failed: `; el de `offer` con coste negativo no lo lleva (TB-12). El constructor de BoundedBlockingQueue falla con `requirement failed` sin texto (BQ-01). El spec de BoundedBlockingQueue tiene 62 bloques, no 79.
- `MessageBufferMap.drop` de un id ausente lanza NPE (MB-DROP-ABSENT).

## Mutantes nuevos (solo ellos + control)

```
| index-put-duplicate-is-new | DETECTED | 1410/1416 |
| token-bucket-no-tick-at-exact-period | DETECTED | 1415/1416 |
| switch-no-revert-on-panic | DETECTED | 1411/1416 |
| wildcard-skips-double-fallback | DETECTED | 1412/1416 |
| frequency-sketch-reset-keeps-neighbour-bits | DETECTED | 1413/1416 |
| murmur3-mix-last-rotation | DETECTED | 1408/1416 |
| stable-queue-ties-last-in-first-out | DETECTED | 1410/1416 |
| int-map-hash-starts-at-zero | DETECTED | 1410/1416 |
| control (unmodified) | CLEAN | 1416/1416 |
--check-patterns: 77 mutant patterns checked, 0 stale
```

## Pendiente

- AC-UTIL-008: A1/A2 pasan por el EventStream (AC-EVT-003).
- AC-UTIL-009: A7 (prefix* solapados) espera Q-UTIL-21 (docs/owner-batch-02.md).
- Sin legado GPL: MurmurHash3 y String.hashCode se derivan de su definicion publica y se validan con el jar; no hace falta cleanroom.
