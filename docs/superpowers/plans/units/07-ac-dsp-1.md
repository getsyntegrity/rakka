# Spec 07 — AC-DSP (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 25. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 06 `AC-MBX-1` (`06-ac-mbx-1.md`), por las filas AC-MBX-001, AC-MBX-002, AC-MBX-004, AC-MBX-013.
- Spec 03 `AC-CFG-1` (`03-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-007, AC-CFG-012.1, AC-CFG-012.2, AC-CFG-012.3, AC-CFG-012.4, AC-CFG-015, AC-CFG-016.
- Spec 19 `AC-RT-1` (`19-ac-rt-1.md`), por las filas AC-RT-012. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 09 `AC-EVT-1` (`09-ac-evt-1.md`), por las filas AC-EVT-011. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 11 `AC-CELL-1` (`11-ac-cell-1.md`), por las filas AC-CELL-007.1. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CFG-31`, `Q-CFG-32`, `Q-DSP-1`, `Q-DSP-2`, `Q-DSP-3`, `Q-DSP-4`, `Q-DSP-5`, `Q-DSP-6`, `Q-DSP-7`, `Q-MBX-11`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: MessageDispatcher: attach/detach, inhabitants y apagado diferido (shutdown-timeout) (+5 filas)

Filas: AC-DSP-001, AC-DSP-002, AC-DSP-003, AC-DSP-004, AC-DSP-005, AC-DSP-006. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Selección del dispatcher de un actor (Props, deployment, "same-as-parent") y… (+5 filas)

Filas: AC-DSP-007, AC-DSP-008, AC-DSP-009, AC-DSP-010, AC-DSP-011, AC-DSP-012. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: AffinityPool executor (affinity-pool-executor) (+5 filas)

Filas: AC-DSP-013, AC-DSP-014, AC-DSP-015, AC-DSP-016.1, AC-DSP-016.2, AC-DSP-017.1. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Futures / CompletionStages / japi (ayudantes Java de futuros) (+5 filas)

Filas: AC-DSP-017.2, AC-DSP-018, AC-DSP-019, AC-DSP-020, AC-DSP-021, AC-DSP-022. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: MessageDispatcher.suspend/resume por actor (guardas de identidad y reprogramación)

Filas: AC-DSP-023. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-DSP-001 | MessageDispatcher: attach/detach, inhabitants y apagado diferido… | `trait Dispatcher` con contador de habitantes; pool de hilos o runtime async | actor-tests/src/test/scala/org/apache/pekko/dispatch/DispatcherShutdownSpec.scala:32 ;… | Q-MBX-6, Q-X-4 |
| AC-DSP-002 | Dispatcher (event-based): registerForExecution con CAS a Scheduled y reintento… | `spawn` de tarea por mailbox con flag atómico | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatcherActorSpec.scala:71,7… | Q-MBX-11, Q-MBX-6 |
| AC-DSP-003 | throughput y throughput-deadline-time | parámetros del loop de mailbox | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatcherActorSpec.scala:84,1… | Q-DSP-5, Q-MBX-6 |
| AC-DSP-004 | Registro Dispatchers: lookup por id, hasDispatcher, alias (string) con… | `Dispatchers` con `DashMap<String, Arc<dyn Dispatcher>>` | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:162-235… | Q-CFG-31, Q-DSP-2, Q-DSP-4, Q-X-1 |
| AC-DSP-005 | default-dispatcher (fork-join-executor por defecto,… | dispatcher sobre pool work-stealing | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:178-182 | Q-DSP-1 |
| AC-DSP-006 | internal-dispatcher, default-blocking-io-dispatcher y ExecutionContext externo… | ids fijos + `Executor` provisto por el usuario (p. ej. handle de tokio) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:53,77,1… | Q-CFG-32, Q-DSP-1, Q-DSP-2, Q-X-4 |
| AC-DSP-007 | Selección del dispatcher de un actor (Props, deployment, "same-as-parent") y… | resolución en el provider | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:273-302… | Q-DSP-2 |
| AC-DSP-008 | ForkJoinExecutorConfigurator (PekkoForkJoinPool, parallelism,… | runtime multi-hilo work-stealing (tokio o pool propio) con N=parallelism | actor-tests/src/test/scala/org/apache/pekko/dispatch/ForkJoinExecutorConfiguratorSpec.sca… | Q-DSP-1, Q-DSP-6, Q-X-4 |
| AC-DSP-009 | ThreadPoolExecutorConfigurator y ThreadPoolConfig | pool de hilos propio (sin work stealing) | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:241-247 | Q-DSP-1 |
| AC-DSP-010 | ExecutorServiceConfigurator SPI, default-executor y fallback, executor por FQCN | trait `ExecutorFactory` + registro | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:53,77… | Q-CFG-31, Q-DSP-1, Q-DSP-2, Q-X-2 |
| AC-DSP-011 | PinnedDispatcher (un hilo dedicado por actor) | `std::thread` dedicado por actor | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/PinnedActorSpec.scala:49-62 ;… | Q-DSP-1 |
| AC-DSP-012 | BalancingDispatcher (cola compartida, work stealing; deprecado como tipo de… | cola MPMC compartida | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/BalancingDispatcherSpec.scala:… | Q-DSP-1, Q-MBX-6 |
| AC-DSP-013 | AffinityPool executor (affinity-pool-executor) | pool propio con colas por hilo | derivada del código (ver bloque de contrato) | Q-DSP-1, Q-X-4 |
| AC-DSP-014 | VirtualThreadExecutorConfigurator y virtualize (JDK 21) | N/A (equivalente natural: tareas async) | actor-tests/src/test/scala-jdk21-only/org/apache/pekko/dispatch/VirtualThreadPoolDispatch… | Q-DSP-1 |
| AC-DSP-015 | MonitorableThreadFactory: nombres "<system>-<dispatcherId>-N", daemonic,… | `std::thread::Builder::name` | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:237-271 | - |
| AC-DSP-016.1 | Dispatcher.execute: TaskInvocation, conteo de inhabitants y reportFailure | `Dispatcher::execute(Box<dyn FnOnce + Send>)` con catch_unwind y publicación de Error | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:284-310… | Q-DSP-3, Q-X-5 |
| AC-DSP-016.2 | BatchingExecutor: lotes de tareas anidadas, no-inline y reenvío ante bloqueo | lote opcional en thread_local con hook explícito de bloqueo (Q-DSP-7) | actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:71-97 ;… | Q-DSP-7 |
| AC-DSP-017.1 | ExecutionContexts (fromExecutor, global) y SameThreadExecutionContext… | `InlineExecutor` (ejecución en línea con trampolín) | actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:52-69 ;… | Q-DSP-3 |
| AC-DSP-017.2 | Futures / CompletionStages / japi (ayudantes Java de futuros) | `std::future`/`futures` sin API propia (propuesta de exclusión Q-DSP-3) | actor-tests/src/test/java/org/apache/pekko/dispatch/CompletionStagesTests.java:38-144 ;… | Q-DSP-3 |
| AC-DSP-018 | SerializedSuspendableExecutionContext | - | actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:260,280,3… | Q-DSP-3 |
| AC-DSP-019 | Configuradores internos PekkoJdk9ForkJoinPool / ForkJoinPoolConstants /… | - | derivada del código (ver bloque de contrato) | Q-DSP-1, Q-DSP-6 |
| AC-DSP-020 | Envelope y validación de mensaje nulo (InvalidMessageException) | tipo Envelope { message, sender } con validación | derivada del código (ver bloque de contrato) | Q-X-3 |
| AC-DSP-021 | BalancingDispatcherConfigurator: requisito MultipleConsumerSemantics y… | validación en la construcción del dispatcher | actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:440-459 | - |
| AC-DSP-022 | Dispatchers.getConfig: resolución de alias devolviendo config vacía | función pura de resolución sobre Config | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:217-235 | Q-DSP-4 |
| AC-DSP-023 | MessageDispatcher.suspend/resume por actor (guardas de identidad y… | `Dispatcher::suspend(cell)` / `resume(cell)` con la misma guarda | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:360-382 | Q-MBX-6 |

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

Spec 08 `AC-SCH-1` (`08-ac-sch-1.md`).
