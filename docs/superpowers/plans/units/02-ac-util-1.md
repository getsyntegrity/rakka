# Spec 02 — AC-UTIL (1/3)

**Estado:** bloqueada. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-UTIL-002.1.

## Prerrequisitos

- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-012.1, AC-CFG-012.2, AC-CFG-012.3, AC-CFG-012.4, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5, AC-CFG-017.6, AC-CFG-017.7. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 03 `AC-UTIL-2` (`03-ac-util-2.md`), por las filas AC-UTIL-016.15, AC-UTIL-016.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 04 `AC-UTIL-3` (`04-ac-util-3.md`), por las filas AC-UTIL-022. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 23 `AC-SER-1` (`23-ac-ser-1.md`), por las filas AC-SER-004.1, AC-SER-004.2. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 11 `AC-EVT-1` (`11-ac-evt-1.md`), por las filas AC-EVT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-009, AC-MBX-010. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: `Q-UTIL-12`, `Q-UTIL-6`.
- Propuestas aún sin decisión del Owner: `Q-CFG-11`, `Q-CFG-4`, `Q-CFG-5`, `Q-UTIL-10`, `Q-UTIL-11`, `Q-UTIL-14`, `Q-UTIL-15`, `Q-UTIL-2`, `Q-UTIL-3`, `Q-UTIL-4`, `Q-UTIL-5`, `Q-UTIL-7`, `Q-UTIL-8`, `Q-X-1`, `Q-X-3`, `Q-X-4`, `Q-X-5`, `Q-X-6`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `util-core-case`) y `cases/oracle-util-core.jsonl`. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Timeout (duración para ask/resolveOne) (+5 filas)

Filas: AC-UTIL-001, AC-UTIL-002.2, AC-UTIL-002.3, AC-UTIL-002.4, AC-UTIL-002.5, AC-UTIL-003.1. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: NotUsed (marcador de valor sin importancia) (+5 filas)

Filas: AC-UTIL-003.2, AC-UTIL-004.1, AC-UTIL-004.2, AC-UTIL-004.3, AC-UTIL-005.1, AC-UTIL-005.2. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: ByteString inmutable (ByteString1C, ByteString1, ByteStrings; factories, slicing,… (+5 filas)

Filas: AC-UTIL-006.1, AC-UTIL-006.2, AC-UTIL-006.3, AC-UTIL-006.4, AC-UTIL-006.5, AC-UTIL-006.8. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: ByteString: búsqueda (indexOf con from/to, lastIndexOf, contains, indexOfSlice,… (+5 filas)

Filas: AC-UTIL-006.7, AC-UTIL-006.6, AC-UTIL-007, AC-UTIL-008, AC-UTIL-009, AC-UTIL-010. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: TokenBucket / NanoTimeTokenBucket (+5 filas)

Filas: AC-UTIL-011, AC-UTIL-012, AC-UTIL-013, AC-UTIL-014, AC-UTIL-015.1, AC-UTIL-015.2. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-UTIL-001 | Timeout (duración para ask/resolveOne) | `pub struct Timeout(i64)` Copy: nanosegundos con signo, invariante único \|nanos\| <=… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorTimeoutSpec.scala:34-45… | Q-UTIL-1, Q-UTIL-2 |
| AC-UTIL-002.2 | Helpers.toRootLowerCase | `to_root_lower_case(&str) -> String` (= `str::to_lowercase`) | derivada del código (ver bloque de contrato) | Q-UTIL-7 |
| AC-UTIL-002.3 | Helpers.Requiring (requiring con IllegalArgumentException) | trait `util::helpers::Requiring` con `requiring` y `requiring_with` -> `Result<A,… | derivada del código (ver bloque de contrato) | Q-UTIL-3, Q-X-9 |
| AC-UTIL-002.4 | ConfigOps.getMillisDuration / getNanosDuration | `Config::get_millis_duration(path) -> Result<Duration, ConfigError>` y… | derivada del código (ver bloque de contrato) | Q-CFG-4, Q-X-1, Q-X-9 |
| AC-UTIL-002.5 | ConfigOps.renderWithRedactions (toString de Settings con datos sensibles… | `Config::render_with_redactions(&self) -> String` | derivada del código (ver bloque de contrato) | Q-CFG-4 |
| AC-UTIL-003.1 | Done (singleton de finalización sin valor) | `pub struct Done;` (Q-X-9: o `()`) | derivada del código (ver bloque de contrato) | Q-UTIL-5, Q-X-9 |
| AC-UTIL-003.2 | NotUsed (marcador de valor sin importancia) | `pub struct NotUsed;` (Q-X-9: o `()`) | derivada del código (ver bloque de contrato) | Q-UTIL-5, Q-X-9 |
| AC-UTIL-004.1 | PekkoException (excepción base) | `struct PekkoException { message, cause: Option<Arc<dyn Error+Send+Sync>> }` | actor-tests/src/test/scala/org/apache/pekko/PekkoExceptionSpec.scala:32-44 | Q-UTIL-3, Q-X-9 |
| AC-UTIL-004.2 | ConfigurationException | `struct ConfigurationException` (tipo separado) con `From` a PekkoException si hace falta | derivada del código (ver bloque de contrato) | Q-UTIL-3, Q-UTIL-4, Q-X-9 |
| AC-UTIL-004.3 | OnlyCauseStackTrace (trait que suprime el stack trace propio y conserva el de… | n/a: los errores de Rust no llevan stack trace (propuesta de exclusión en Q-X-9) | derivada del código (ver bloque de contrato) | Q-UTIL-4, Q-X-9 |
| AC-UTIL-005.1 | PekkoVersion.require y UnsupportedPekkoVersion | `fn require(library: &str, required: &str, current: &str) -> Result<(),… | actor-tests/src/test/scala/org/apache/pekko/PekkoVersionSpec.scala:23-106 | Q-CFG-5 |
| AC-UTIL-005.2 | util.Version (versión comparable con M/RC/dynver) | `struct Version` con `pekko_compare`/`PartialEq`, `Version::parse -> Result<_,… | actor-tests/src/test/scala/org/apache/pekko/util/VersionSpec.scala:19-105 | Q-UTIL-6, Q-X-9 |
| AC-UTIL-006.1 | ByteString inmutable (ByteString1C, ByteString1, ByteStrings; factories,… | `ByteString` sobre `bytes::Bytes` o rope propio con recortes saturados y sin copia;… | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringSpec.scala:372-696 ;… | Q-UTIL-12, Q-UTIL-14, Q-UTIL-8 |
| AC-UTIL-006.2 | CompactByteString (compactación a ByteString1C) | variante compacta del mismo tipo | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringSpec.scala:2186-2191 ;… | Q-UTIL-12 |
| AC-UTIL-006.3 | ByteStringBuilder (append de primitivos con byte order, putBytes, result) | builder con `ByteOrder` explícito | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringBuilderSpec.scala:22-38 ;… | Q-UTIL-12 |
| AC-UTIL-006.4 | ByteIterator (lectura de primitivos con byte order, getBytes, take/drop) | iterador con cursor y `ByteOrder` | actor-tests/src/test/scala/org/apache/pekko/util/ByteIteratorSpec.scala:24-93 ;… | Q-UTIL-12 |
| AC-UTIL-006.5 | ByteStringInputStream / ByteStringOutputStream (adaptadores java.io) | `impl Read` / `impl Write` sobre ByteString | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringInputStreamSpec.scala:26-53 | Q-UTIL-12 |
| AC-UTIL-006.8 | ByteString: serialización Java (SerializationProxy, identidades 0/1/2) | sin equivalente directo (serialización Java); depende de Q-X-3/Q-CFG-11 y del módulo… | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringSpec.scala:2461-2527 | Q-CFG-11, Q-UTIL-11, Q-UTIL-12, Q-X-3 |
| AC-UTIL-006.7 | ByteString: búsqueda (indexOf con from/to, lastIndexOf, contains,… | `index_of(byte, from, to) -> Option<usize>`, memchr/memmem y `starts_with`; Q-UTIL-12 | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringSpec.scala:697-2110 | Q-UTIL-12 |
| AC-UTIL-006.6 | ByteString: factorías y conversiones (String, Array, ByteBuffer, Base64,… | `From<&[u8]>`/`From<Vec<u8>>`, `decode_string`, base64 y `as_slice`/`chunks()`; las… | actor-tests/src/test/scala/org/apache/pekko/util/ByteStringSpec.scala:420-440 ;… | Q-UTIL-12 |
| AC-UTIL-007 | Index / ConcurrentMultiMap | `DashMap`-like o `Mutex<BTreeMap<K, BTreeSet<V>>>` | actor-tests/src/test/scala/org/apache/pekko/util/IndexSpec.scala:49:49-155 | Q-UTIL-10 |
| AC-UTIL-008 | SubclassifiedIndex / Subclassification | Rust no tiene herencia: clasificador por `TypeId` + jerarquía declarada (trait de "canal… | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:172-303 | Q-UTIL-10, Q-X-6 |
| AC-UTIL-009 | WildcardIndex (matching de paths de deployment con * y **) | trie por elementos de path | actor-tests/src/test/scala/org/apache/pekko/util/WildcardIndexSpec.scala:21:21-99 | Q-UTIL-8 |
| AC-UTIL-010 | MessageBuffer / MessageBufferMap | `VecDeque<(M, R)>` y `HashMap<I, MessageBuffer>`; `drop` de id ausente según Q-UTIL-15 | actor-tests/src/test/scala/org/apache/pekko/util/MessageBufferSpec.scala:26-192 | Q-UTIL-10, Q-UTIL-15 |
| AC-UTIL-011 | TokenBucket / NanoTimeTokenBucket | struct simple con `Instant` | actor-tests/src/test/scala/org/apache/pekko/util/TokenBucketSpec.scala:27:26-263 | Q-UTIL-10 |
| AC-UTIL-012 | BoundedBlockingQueue | `Mutex<BinaryHeap>` + `Condvar`, o canal acotado | actor-tests/src/test/scala/org/apache/pekko/util/BoundedBlockingQueueSpec.scala:49-685 | Q-UTIL-10, Q-UTIL-11, Q-X-4 |
| AC-UTIL-013 | StablePriorityQueue / PriorityQueueStabilizer | `BinaryHeap<(prio, seq, T)>` | actor-tests/src/test/scala/org/apache/pekko/dispatch/StablePriorityDispatcherSpec.scala:2… | Q-UTIL-10 |
| AC-UTIL-014 | TypedMultiMap | `TypedMultiMap<K, V>` con trait asociado para la dependencia clave->valor y estructura… | actor-tests/src/test/scala-2/org/apache/pekko/util/TypedMultiMapSpec.scala:22-77 | Q-UTIL-11 |
| AC-UTIL-015.1 | Switch (interruptor atómico con acciones bajo lock) | `Switch` con `AtomicBool` + `Mutex<()>`; acciones como closures | actor-tests/src/test/scala/org/apache/pekko/util/SwitchSpec.scala:24-110 | Q-UTIL-10, Q-UTIL-11, Q-X-5 |
| AC-UTIL-015.2 | ReentrantGuard (deprecado desde 1.2.1) | n/a: `std::sync::Mutex` ya cubre `withGuard` (propuesta de equivalente) | derivada del código (ver bloque de contrato) | Q-UTIL-11 |

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

Spec 03 `AC-UTIL-2` (`03-ac-util-2.md`).
