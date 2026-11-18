# Spec 03 — AC-UTIL (2/3)

**Estado:** bloqueada. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-001, AC-UTIL-005.2.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-007, AC-CFG-016. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 18 `AC-EXT-1` (`18-ac-ext-1.md`), por las filas AC-EXT-001.1. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: `Q-UTIL-6`.
- Propuestas aún sin decisión del Owner: `Q-CFG-11`, `Q-CFG-5`, `Q-CFG-7`, `Q-CFG-8`, `Q-UTIL-11`, `Q-UTIL-14`, `Q-UTIL-16`, `Q-UTIL-8`, `Q-UTIL-9`, `Q-X-2`, `Q-X-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

jars oficiales pekko-actor + scala-library 1.7.1; ya existe tools/oracle/util-core.jsh. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ReentrantGuard (deprecado desde 1.2.1) (+5 filas)

Filas: AC-UTIL-015.2, AC-UTIL-016.1, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-016.4, AC-UTIL-016.5. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Glob (matching `?`/`*` sin regex) (+5 filas)

Filas: AC-UTIL-016.6, AC-UTIL-016.7, AC-UTIL-016.8, AC-UTIL-016.9, AC-UTIL-016.10, AC-UTIL-016.11. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: ClassLoaderObjectInputStream (+5 filas)

Filas: AC-UTIL-016.12, AC-UTIL-016.13, AC-UTIL-016.14, AC-UTIL-016.15, AC-UTIL-016.16, AC-UTIL-016.17. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Unused (marcador para silenciar warnings) (+5 filas)

Filas: AC-UTIL-016.18, AC-UTIL-017.1, AC-UTIL-017.2, AC-UTIL-017.3, AC-UTIL-017.4, AC-UTIL-017.5. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: FutureConverters / CompletionStage <-> Future (+5 filas)

Filas: AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3, AC-UTIL-018.4, AC-UTIL-018.5, AC-UTIL-019. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-UTIL-015.2 | ReentrantGuard (deprecado desde 1.2.1) | n/a: `std::sync::Mutex` ya cubre `withGuard` (propuesta de equivalente) | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-016.1 | OptionVal (Option sin allocación) | `Option<T>` de Rust (ya sin allocación) | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-016.2 | ConstantFun (funciones constantes/identidad Scala y Java) | closures `const`/`identity` donde se necesiten | actor-tests/src/test/java/org/apache/pekko/util/ConstantFunJavaTest.java:26-48 | Q-UTIL-11 |
| AC-UTIL-016.3 | Collections (EmptyImmutableSeq, PartialImmutableValuesIterable) | `Vec`/iteradores nativos | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-016.4 | ImmutableIntMap (mapa Int->Int inmutable) | `BTreeMap<i32,i32>` persistente | actor-tests/src/test/scala/org/apache/pekko/util/ImmutableIntMapSpec.scala:25-156 | Q-UTIL-11, Q-UTIL-14 |
| AC-UTIL-016.5 | PrettyDuration (formateo legible de duraciones) | `pretty(Duration, include_nanos, precision) -> String` | actor-tests/src/test/scala/org/apache/pekko/util/PrettyDurationSpec.scala:25-58 | Q-UTIL-9 |
| AC-UTIL-016.6 | Glob (matching `?`/`*` sin regex) | `glob_matches(pattern: &str, input: &str) -> bool` (greedy con backtracking de un solo… | actor-tests/src/test/scala/org/apache/pekko/util/GlobSpec.scala:33-91 | Q-UTIL-8 |
| AC-UTIL-016.7 | Reflect (findConstructor, findClassLoader, getCallerClass, instantiate) | n/a: sin reflexión; las fábricas las cubre AC-CFG-016 (Q-X-2) | actor-tests/src/test/scala/org/apache/pekko/util/ReflectSpec.scala:33-160 | Q-CFG-8, Q-UTIL-11, Q-X-2 |
| AC-UTIL-016.8 | LineNumbers (extracción de nombre de archivo/línea desde bytecode de lambdas) | n/a: depende de bytecode JVM; propuesta: `std::panic::Location` donde se use | actor-tests/src/test/scala-2/org/apache/pekko/util/LineNumberSpec.scala:22-66 ;… | Q-UTIL-11, Q-UTIL-16 |
| AC-UTIL-016.9 | ManifestInfo (versiones de módulos Pekko en el classpath) | constante de versión del crate; chequeo mixto n/a | actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:26-46 | Q-CFG-5, Q-UTIL-11, Q-UTIL-6 |
| AC-UTIL-016.10 | JavaVersion (major de java.specification.version) | n/a: no hay JVM (propuesta de exclusión en Q-UTIL-11) | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-016.11 | FlightRecorderLoader (carga de JFR vía reflexión) | n/a: `tracing` (Q-CFG-7) | derivada del código (ver bloque de contrato) | Q-CFG-7, Q-UTIL-11 |
| AC-UTIL-016.12 | ClassLoaderObjectInputStream | n/a: sin serialización Java | derivada del código (ver bloque de contrato) | Q-CFG-11, Q-UTIL-11, Q-X-2 |
| AC-UTIL-016.13 | BoxedType (primitivo -> clase envoltorio) | n/a: sin boxing | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-016.14 | UUIDComparator (orden de UUID de 128 bits) | `Ord` sobre `u128` con la regla de signo de Pekko | derivada del código (ver bloque de contrato) | Q-UTIL-11 |
| AC-UTIL-016.15 | HashCode (hash compuesto SEED=23, 37*) | n/a salvo que Version/Address exijan el mismo hash (Q-PATH-3) | derivada del código (ver bloque de contrato) | Q-UTIL-14, Q-UTIL-8 |
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

Spec 04 `AC-UTIL-3` (`04-ac-util-3.md`).
