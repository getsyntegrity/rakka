# Spec 25 — AC-JAPI (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 21. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-020.
- Spec 12 `AC-CELL-1` (`12-ac-cell-1.md`), por las filas AC-CELL-001.
- Spec 19 `AC-FSM-1` (`19-ac-fsm-1.md`), por las filas AC-FSM-001.1, AC-FSM-001.2.
- Spec 15 `AC-SUP-1` (`15-ac-sup-1.md`), por las filas AC-SUP-001.
- Spec 05 `AC-REF-1` (`05-ac-ref-1.md`), por las filas AC-REF-007.1, AC-REF-007.2, AC-REF-011, AC-REF-013.
- Spec 10 `AC-EVT-1` (`10-ac-evt-1.md`), por las filas AC-EVT-001.
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-001.
- Spec 14 `AC-SYS-1` (`14-ac-sys-1.md`), por las filas AC-SYS-004.
- Spec 09 `AC-SCH-1` (`09-ac-sch-1.md`), por las filas AC-SCH-001, AC-SCH-002, AC-SCH-003, AC-SCH-004, AC-SCH-010.
- Spec 18 `AC-ASK-1` (`18-ac-ask-1.md`), por las filas AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3.
- Spec 22 `AC-SER-1` (`22-ac-ser-1.md`), por las filas AC-SER-002, AC-SER-013.
- Spec 23 `AC-IO-1` (`23-ac-io-1.md`), por las filas AC-IO-002.1, AC-IO-005, AC-IO-006.2, AC-IO-007.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-EVT-5`, `Q-JAPI-1`, `Q-JAPI-2`, `Q-JAPI-3`, `Q-JAPI-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: AbstractActor + createReceive y AbstractActor.ActorContext (getters Java,… (+5 filas)

Filas: AC-JAPI-001, AC-JAPI-002, AC-JAPI-003, AC-JAPI-004.1, AC-JAPI-004.2, AC-JAPI-004.3. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: japi.pf DeciderBuilder: decider de supervisión por tipo de Throwable (+5 filas)

Filas: AC-JAPI-004.4, AC-JAPI-005.1, AC-JAPI-005.2, AC-JAPI-005.3, AC-JAPI-005.4, AC-JAPI-013. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Props Java (Props.create con clase/args o Creator) y AbstractProps: validaciones y… (+5 filas)

Filas: AC-JAPI-006, AC-JAPI-007, AC-JAPI-008, AC-JAPI-009.1, AC-JAPI-009.2, AC-JAPI-009.3. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Fábricas Java de mensajes IO: TcpMessage, UdpMessage, UdpConnectedMessage, TcpSO, UdpSO… (+2 filas)

Filas: AC-JAPI-010, AC-JAPI-011, AC-JAPI-012. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-JAPI-001 | AbstractActor + createReceive y AbstractActor.ActorContext (getters Java,… | Equivalente: el trait `Actor` de Rust con `receive`; ver Q-JAPI-1/Q-JAPI-2 | actor-tests/src/test/java/org/apache/pekko/actor/ActorCreationTest.java:80-98 ;… | Q-JAPI-1, Q-JAPI-4 |
| AC-JAPI-002 | UntypedAbstractActor, AbstractLoggingActor,… | Traits `Stash`, `Timers`, logging por contexto en Rust | actor-tests/src/test/java/org/apache/pekko/actor/StashJavaAPI.java:30-54 ;… | Q-JAPI-1 |
| AC-JAPI-003 | AbstractFSM / AbstractLoggingFSM / AbstractFSMWithStash y builders FSM… | Equivalente: FSM de AC-FSM (una sola API) | actor-tests/src/test/java/org/apache/pekko/actor/AbstractFSMActorTest.java:27-62 ;… | Q-JAPI-1, Q-JAPI-2 |
| AC-JAPI-004.1 | japi.pf Match/PFBuilder/AbstractMatch/AbstractPFBuilder/CaseStatement:… | `ReceiveBuilder`/`TypeMatcher` (Q-JAPI-2) | actor-tests/src/test/java/org/apache/pekko/japi/MatchBuilderTest.java:26-104 ;… | Q-JAPI-2 |
| AC-JAPI-004.2 | japi.pf UnitMatch/UnitPFBuilder (variante con efecto, sin valor de retorno) | `ReceiveBuilder<()>` (Q-JAPI-2) | derivada del código (ver bloque de contrato) | Q-JAPI-1, Q-JAPI-2 |
| AC-JAPI-004.3 | japi.pf FI: interfaces funcionales (Apply, UnitApply, Predicate,… | Closures `Fn`/`FnMut` de Rust | derivada del código (ver bloque de contrato) | Q-JAPI-1 |
| AC-JAPI-004.4 | japi.pf DeciderBuilder: decider de supervisión por tipo de Throwable | `Decider` por tipo de error (Q-X-5) | derivada del código (ver bloque de contrato) | Q-JAPI-2, Q-X-5 |
| AC-JAPI-005.1 | japi.function: Function, Function2, Procedure, Effect, Predicate (+ identity),… | Closures `Fn`; `Option<T>`; tuplas | actor-tests/src/test/java/org/apache/pekko/japi/JavaAPITestBase.java:23-25 ;… | Q-JAPI-1 |
| AC-JAPI-005.2 | Funciones y tuplas generadas (japi.function.Functions, japi.tuple.Tuple1..22)… | tuplas nativas | actor-tests/src/test/java/org/apache/pekko/actor/JavaAPI.java:141-145 | Q-JAPI-1 |
| AC-JAPI-005.3 | japi.Throwables: isFatal / isNonFatal con la semántica de… | `is_fatal(&panic_payload)` / clasificación de fallos | actor-tests/src/test/java/org/apache/pekko/japi/ThrowablesTest.java:21-42 | Q-X-5 |
| AC-JAPI-005.4 | japi.JAPI (deprecado) y utilidades Java de colecciones/conversión | n/a | derivada del código (ver bloque de contrato) | Q-JAPI-1 |
| AC-JAPI-013 | JavaPartialFunction: función parcial pura para Java (noMatch, isCheck,… | Fn(&A) -> Option<B> (las dos fases isCheck se sustituyen por Option) | derivada del código (ver bloque de contrato) | Q-JAPI-1 |
| AC-JAPI-006 | Props Java (Props.create con clase/args o Creator) y AbstractProps:… | `Props::new(\|\| Actor)` (closure) y `Props::with_args` | actor-tests/src/test/scala/org/apache/pekko/actor/PropsCreationSpec.scala:51-75 ;… | Q-JAPI-1 |
| AC-JAPI-007 | EventBus Java (EventBus, LookupEventBus, SubchannelEventBus, ScanningEventBus,… | Traits de bus de eventos de AC-EVT | derivada del código (ver bloque de contrato) | Q-EVT-5, Q-JAPI-1 |
| AC-JAPI-008 | Sobrecargas con java.time.Duration, Optional y CompletionStage en Scheduler,… | `std::time::Duration`, `Option<T>`, `Future` | actor-tests/src/test/java/org/apache/pekko/actor/ActorSelectionTest.java:33-44 ;… | Q-JAPI-1, Q-JAPI-4 |
| AC-JAPI-009.1 | TypedActor (clásico, deprecado): proxies JDK; semántica de invocación (void,… | Q-JAPI-3: exclusión propuesta; alternativa macro `#[typed_actor]` | actor-tests/src/test/scala/org/apache/pekko/actor/TypedActorSpec.scala:340-441 (1.7.1) | Q-JAPI-3, Q-X-5 |
| AC-JAPI-009.2 | TypedActor: self/context, hooks de ciclo de vida (PreStart, PostStop,… | Q-JAPI-3 | actor-tests/src/test/scala/org/apache/pekko/actor/TypedActorSpec.scala:292-338,443-588… | Q-JAPI-3 |
| AC-JAPI-009.3 | TypedProps, TypedActorExtension y createActorRefProxy (creación de proxies,… | Q-JAPI-3 | actor-tests/src/test/scala/org/apache/pekko/actor/TypedActorSpec.scala:290-305 (1.7.1) ;… | Q-JAPI-3 |
| AC-JAPI-010 | Fábricas Java de mensajes IO: TcpMessage, UdpMessage, UdpConnectedMessage,… | n/a: se construyen los enums directamente | derivada del código (ver bloque de contrato) | Q-JAPI-1, Q-JAPI-4 |
| AC-JAPI-011 | Clases base Java del núcleo: AbstractScheduler, JSerializer, offsets Unsafe… | n/a | derivada del código (ver bloque de contrato) | Q-JAPI-1 |
| AC-JAPI-012 | ReceiveBuilder (receiveBuilder()): casos… | match nativo sobre enum de mensajes; ReceiveBuilder opcional con… | actor-tests/src/test/java/org/apache/pekko/japi/pf/ReceiveBuilderTest.java:60-258 | Q-JAPI-1, Q-JAPI-2, Q-JAPI-4 |

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

Spec 26 `AT-BEH-1` (`26-at-beh-1.md`).
