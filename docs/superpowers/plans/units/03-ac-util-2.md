# Spec 03 — AC-UTIL (2/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 3. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-UTIL-002.1, AC-UTIL-002.2, AC-UTIL-002.3, AC-UTIL-003.1, AC-UTIL-003.2, AC-UTIL-004.1, AC-UTIL-004.3, AC-UTIL-005.1, AC-UTIL-005.2, AC-UTIL-007, AC-UTIL-010, AC-UTIL-011, AC-UTIL-012, AC-UTIL-013, AC-UTIL-014, AC-UTIL-015.1, AC-UTIL-015.2, AC-UTIL-016.4, AC-UTIL-016.5, AC-UTIL-016.6, AC-UTIL-016.7, AC-UTIL-016.8, AC-UTIL-016.12, AC-UTIL-016.13, AC-UTIL-016.14, AC-UTIL-016.15, AC-UTIL-016.16, AC-UTIL-017.1, AC-UTIL-017.2, AC-UTIL-017.3, AC-UTIL-017.4, AC-UTIL-017.5, AC-UTIL-023, AC-UTIL-024, AC-UTIL-026, AC-UTIL-028.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-006.5.
- Spec 09 `AC-SCH-1` (`09-ac-sch-1.md`), por las filas AC-SCH-008.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-UTIL-18`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `util-core-case`) y `cases/oracle-util-core.jsonl`. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: UnsynchronizedByteArrayInputStream, ByteStringInputStream, ByteBufferCleaner (+2 filas)

Filas: AC-UTIL-022, AC-UTIL-025, AC-UTIL-027. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-UTIL-022 | UnsynchronizedByteArrayInputStream, ByteStringInputStream, ByteBufferCleaner | `std::io::Cursor<&[u8]>` | actor-tests/src/test/scala/org/apache/pekko/io/UnsynchronizedByteArrayInputStreamSpec.sca… | Q-UTIL-11, Q-UTIL-12 |
| AC-UTIL-025 | Helpers.currentTimeMillisToUTCString y Helpers.timestamp (formato de hora para… | `fn current_time_millis_to_utc_string(i64) -> String` y `fn timestamp(i64) -> String` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:283-317 | Q-UTIL-10, Q-UTIL-18 |
| AC-UTIL-027 | Clock / NanoClock / ScheduledClock (reloj interno basado en nanoTime) | `util::clock`: trait `Clock` (`current_time`, `earlier_time(i64 ns)`), `NanoClock`… | - (sin spec en upstream: PlatformOracle CK-*; tests/platform_spec.rs) | Q-UTIL-11 |

## Gates y criterio de cierre

```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --lcov --output-path target/lcov.info   # cobertura de líneas nuevas o modificadas >= 90 %; reportar comando, resultado y líneas no cubiertas con motivo
```

La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.

## Siguiente

Spec 04 `AC-CFG-1` (`04-ac-cfg-1.md`).
