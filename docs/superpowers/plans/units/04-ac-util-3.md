# Spec 04 — AC-UTIL (3/3)

**Estado:** bloqueada. **Filas:** 7. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-002.2, AC-UTIL-006.5.
- Spec 03 `AC-UTIL-2` (`03-ac-util-2.md`), por las filas AC-UTIL-016.6.
- Spec 10 `AC-SCH-1` (`10-ac-sch-1.md`), por las filas AC-SCH-008.1, AC-SCH-008.2, AC-SCH-008.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 24 `AC-IO-1` (`24-ac-io-1.md`), por las filas AC-IO-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: `Q-UTIL-12`.
- Propuestas aún sin decisión del Owner: `Q-UTIL-10`, `Q-UTIL-11`, `Q-UTIL-13`, `Q-UTIL-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `util-core-case`) y `cases/oracle-util-core.jsonl`. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Acceso Unsafe y offsets de campos (Unsafe.java, AbstractActorRef, AbstractActorCell,… (+5 filas)

Filas: AC-UTIL-020, AC-UTIL-021, AC-UTIL-022, AC-UTIL-023, AC-UTIL-024, AC-UTIL-025. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Helpers.isWindows (detección de SO por os.name)

Filas: AC-UTIL-026. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-UTIL-020 | Acceso Unsafe y offsets de campos (Unsafe.java, AbstractActorRef,… | `std::sync::atomic` (`AtomicPtr`, `AtomicU32`) / `arc-swap` | actor-tests/src/test/scala/org/apache/pekko/util/AsciiStringCopySpec.scala:22-45 | Q-UTIL-11 |
| AC-UTIL-021 | Filtros MiMa (compatibilidad binaria) | N/A (en Rust: semver + `cargo-semver-checks`) | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-022 | UnsynchronizedByteArrayInputStream, ByteStringInputStream, ByteBufferCleaner | `std::io::Cursor<&[u8]>` | actor-tests/src/test/scala/org/apache/pekko/io/UnsynchronizedByteArrayInputStreamSpec.sca… | Q-UTIL-11, Q-UTIL-12 |
| AC-UTIL-023 | Helpers.makePattern (patrón glob `?`/`*` a java.util.regex.Pattern) | `glob_matches` cubre el contrato; sin regex | actor-tests/src/test/scala/org/apache/pekko/util/GlobSpec.scala:60-73 | Q-UTIL-8 |
| AC-UTIL-024 | Helpers.compareIdentityHash / identityHashComparator (orden por… | orden por dirección/ID único del objeto; en Rust no existe identityHashCode estable | actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:307 ;… | Q-UTIL-10 |
| AC-UTIL-025 | Helpers.currentTimeMillisToUTCString y Helpers.timestamp (formato de hora para… | `fn current_time_millis_to_utc_string(i64) -> String` y `fn timestamp(i64) -> String` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:283-317 | Q-UTIL-10 |
| AC-UTIL-026 | Helpers.isWindows (detección de SO por os.name) | `cfg!(windows)` | derivada del código (ver bloque de contrato) | Q-UTIL-13 |

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

Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`).
