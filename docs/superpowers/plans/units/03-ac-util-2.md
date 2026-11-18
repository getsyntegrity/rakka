# Spec 03 — AC-UTIL (2/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 19. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-UTIL-002.1, AC-UTIL-002.2, AC-UTIL-002.3, AC-UTIL-003.1, AC-UTIL-003.2, AC-UTIL-004.1, AC-UTIL-004.3, AC-UTIL-005.1, AC-UTIL-005.2, AC-UTIL-016.5, AC-UTIL-016.6, AC-UTIL-016.7, AC-UTIL-016.12, AC-UTIL-016.13, AC-UTIL-016.14, AC-UTIL-016.15, AC-UTIL-023, AC-UTIL-024, AC-UTIL-026, AC-UTIL-028.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-001, AC-UTIL-006.5, AC-UTIL-016.1.
- Spec 09 `AC-SCH-1` (`09-ac-sch-1.md`), por las filas AC-SCH-008.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-UTIL-18`, `Q-UTIL-19`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `util-core-case`) y `cases/oracle-util-core.jsonl`. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: WallClock / AlwaysIncreasingClock (reloj de pared estrictamente creciente) (+5 filas)

Filas: AC-UTIL-016.16, AC-UTIL-016.17, AC-UTIL-016.18, AC-UTIL-017.1, AC-UTIL-017.2, AC-UTIL-017.3. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: SegmentedRecencyList (LRU segmentado) (+5 filas)

Filas: AC-UTIL-017.4, AC-UTIL-017.5, AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3, AC-UTIL-018.4. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: ScalaBatchable (marca de ejecución por lotes en 2.12/2.13/3) (+5 filas)

Filas: AC-UTIL-018.5, AC-UTIL-019, AC-UTIL-020, AC-UTIL-021, AC-UTIL-022, AC-UTIL-025. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Clock / NanoClock / ScheduledClock (reloj interno basado en nanoTime)

Filas: AC-UTIL-027. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-UTIL-016.16 | WallClock / AlwaysIncreasingClock (reloj de pared estrictamente creciente) | trait `WallClock` + `AlwaysIncreasingClock` con `AtomicI64` | derivada del código (ver bloque de contrato) | - |
| AC-UTIL-016.17 | ErrorMessages.RemoteDeploymentConfigErrorPrefix | `const REMOTE_DEPLOYMENT_CONFIG_ERROR_PREFIX: &str` | derivada del código (ver bloque de contrato) | - |
| AC-UTIL-016.18 | Unused (marcador para silenciar warnings) | n/a: Rust avisa por `_` | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-017.1 | FrequencySketch (Count-Min Sketch de 4 bits) | no portar en este alcance (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/FrequencySketchSpec.scala:23-416 ;… | Q-UTIL-11, Q-UTIL-14, Q-UTIL-8 |
| AC-UTIL-017.2 | FrequencyList (lista de frecuencias LFU) | no portar en este alcance (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/FrequencyListSpec.scala:23-257 | Q-UTIL-11 |
| AC-UTIL-017.3 | RecencyList (LRU por reloj) | no portar en este alcance (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/RecencyListSpec.scala:21-120 | Q-UTIL-11 |
| AC-UTIL-017.4 | SegmentedRecencyList (LRU segmentado) | no portar en este alcance (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/SegmentedRecencyListSpec.scala:23-148 | Q-UTIL-11 |
| AC-UTIL-017.5 | DoubleLinkedList | no portar en este alcance (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/DoubleLinkedListSpec.scala:25-200 | Q-UTIL-11 |
| AC-UTIL-018.1 | FutureConverters / CompletionStage <-> Future | n/a: sin equivalente en Rust (propuesta; Q-UTIL-11) | actor-tests/src/test/java/org/apache/pekko/util/JavaConverterScala3InlineTest.java:34-46… | Q-UTIL-11, Q-X-4 |
| AC-UTIL-018.2 | OptionConverters / FunctionConverters / JavaDurationConverters | n/a: sin equivalente en Rust (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/Scala212CompatTest.scala:35-78 (1.7.1)… | Q-UTIL-1, Q-UTIL-11 |
| AC-UTIL-018.3 | ccompat (JavaConverters, compat.Future/PartialFunction) y shims de Scala 2.12 | n/a: sin equivalente en Rust (propuesta; Q-UTIL-11) | actor-tests/src/test/scala/org/apache/pekko/util/Scala212CompatTest.scala:35-78 (1.7.1) | Q-UTIL-11 |
| AC-UTIL-018.4 | SameThreadExecutionContext (ejecuta el callback inline, 'parasitic') | n/a salvo SameThreadExecutionContext (callback inline) | actor-tests/src/test/scala/org/apache/pekko/dispatch/SameThreadExecutionContextSpec.scala… | Q-UTIL-11, Q-X-4 |
| AC-UTIL-018.5 | ScalaBatchable (marca de ejecución por lotes en 2.12/2.13/3) | n/a: sin equivalente en Rust (propuesta; Q-UTIL-11) | derivada del código (ver bloque de contrato) | Q-UTIL-11, Q-X-4 |
| AC-UTIL-019 | Anotaciones ApiMayChange, DoNotInherit, InternalApi, InternalStableApi | `pub(crate)`, sealed traits (patrón `mod private { pub trait Sealed {} }`),… | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-020 | Acceso Unsafe y offsets de campos (Unsafe.java, AbstractActorRef,… | `std::sync::atomic` (`AtomicPtr`, `AtomicU32`) / `arc-swap` | actor-tests/src/test/scala/org/apache/pekko/util/AsciiStringCopySpec.scala:22-45 | Q-UTIL-11, Q-UTIL-19 |
| AC-UTIL-021 | Filtros MiMa (compatibilidad binaria) | N/A (en Rust: semver + `cargo-semver-checks`) | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
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
