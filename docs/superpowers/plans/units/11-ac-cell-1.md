# Spec 11 — AC-CELL (1/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 14 `AC-SUP-1` (`14-ac-sup-1.md`), por las filas AC-SUP-001, AC-SUP-004, AC-SUP-011, AC-SUP-015, AC-SUP-019. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 06 `AC-MBX-1` (`06-ac-mbx-1.md`), por las filas AC-MBX-002, AC-MBX-003.
- Spec 15 `AC-DW-1` (`15-ac-dw-1.md`), por las filas AC-DW-002, AC-DW-007, AC-DW-010. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 09 `AC-EVT-1` (`09-ac-evt-1.md`), por las filas AC-EVT-010.
- Spec 04 `AC-REF-1` (`04-ac-ref-1.md`), por las filas AC-REF-009.2, AC-REF-016, AC-REF-020.
- Spec 08 `AC-SCH-1` (`08-ac-sch-1.md`), por las filas AC-SCH-001.
- Spec 07 `AC-DSP-1` (`07-ac-dsp-1.md`), por las filas AC-DSP-016.1, AC-DSP-016.2.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CELL-1`, `Q-CELL-2`, `Q-CELL-3`, `Q-MBX-11`, `Q-REF-2`, `Q-REF-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Trait Actor: receive (PartialFunction), self, context; Receive; emptyBehavior;… (+5 filas)

Filas: AC-CELL-001, AC-CELL-002.1, AC-CELL-002.2, AC-CELL-003, AC-CELL-004, AC-CELL-005. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Hooks de ciclo de vida: preStart, postStop, preRestart (default: unwatch+stop hijos y… (+5 filas)

Filas: AC-CELL-006, AC-CELL-007.1, AC-CELL-007.2, AC-CELL-007.3, AC-CELL-007.4, AC-CELL-008.1. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Stop 2/2: finishTerminate — postStop, detach y notificaciones (+5 filas)

Filas: AC-CELL-008.2, AC-CELL-009, AC-CELL-010, AC-CELL-011, AC-CELL-012, AC-CELL-013.1. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Diferimiento (stash) de mensajes de sistema según el estado de la celda (+5 filas)

Filas: AC-CELL-013.2, AC-CELL-013.3, AC-CELL-014, AC-CELL-015.1, AC-CELL-015.2, AC-CELL-016. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: ReceiveTimeout (+5 filas)

Filas: AC-CELL-017, AC-CELL-018, AC-CELL-019, AC-CELL-020, AC-CELL-021, AC-CELL-022. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-CELL-001 | Trait Actor: receive (PartialFunction), self, context; Receive; emptyBehavior;… | `trait Actor: Send + 'static { fn receive(&mut self, ctx: &mut Context, msg: AnyMsg) ->… | actor-tests/src/test/scala/org/apache/pekko/actor/HotSwapSpec.scala:27-69 ;… | Q-CELL-1, Q-X-3 |
| AC-CELL-002.1 | Creación: Create como primer mensaje de sistema, instanciación y preStart | fábrica de Props ejecutada en la tarea del actor durante Create; `pre_start` tras… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:49-82 ;… | - |
| AC-CELL-002.2 | Creación: fallos del constructor o de preStart → ActorInitializationException | panic/Err en la fábrica o en pre_start → `ActorInitializationError { actor, message,… | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:548-572 ;… | Q-REF-2, Q-X-5 |
| AC-CELL-003 | Procesamiento de mensajes uno a la vez; AutoReceivedMessage antes del receive… | tarea por actor que drena el mailbox; `&mut self` garantiza exclusión | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:312-345… | Q-MBX-11, Q-X-4 |
| AC-CELL-004 | unhandled(): Terminated → DeathPactException; resto → UnhandledMessage en… | resultado `Unhandled` → la celda llama `actor.unhandled(msg)` | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:215-244 ;… | - |
| AC-CELL-005 | become / unbecome (pila de conductas) | `Vec<Box<dyn Behavior>>` en la celda | actor-tests/src/test/scala/org/apache/pekko/actor/HotSwapSpec.scala:27-133 ;… | Q-CELL-1 |
| AC-CELL-006 | Hooks de ciclo de vida: preStart, postStop, preRestart (default: unwatch+stop… | métodos por defecto del trait Actor | actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:49-130 ;… | - |
| AC-CELL-007.1 | Restart 1/4: Recreate, suspensión y preRestart de la instancia vieja | estado de celda `Restarting{cause}`; `fault_recreate(cause)` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:49-82 ;… | Q-X-5 |
| AC-CELL-007.2 | Restart 2/4: espera a la parada de los hijos (Recreation) | contenedor de hijos `Terminating{ reason: Recreation(cause) }` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorMiscSpec.scala:86-99 ;… | - |
| AC-CELL-007.3 | Restart 3/4: nueva instancia, postRestart y reinicio de hijos sobrevivientes | `finish_recreate(cause)` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorTreeSpec.scala:27-45 ;… | - |
| AC-CELL-007.4 | Restart 4/4: fallo en postRestart o constructor → PostRestartException | `PostRestartError { actor, cause, original_cause }` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:421-460 | Q-X-5 |
| AC-CELL-008.1 | Stop 1/2: terminate() — cancelación, unwatch y parada de hijos | estado `Terminating` + conjunto de hijos pendientes | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:422-440 ;… | - |
| AC-CELL-008.2 | Stop 2/2: finishTerminate — postStop, detach y notificaciones | `finish_terminate()` con orden fijo | actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:132-140 ;… | - |
| AC-CELL-009 | Fallo en postStop se loguea y no impide la terminación | capturar panic/Err del hook y publicar Error | actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:132-140 ;… | Q-X-5 |
| AC-CELL-010 | stop(ref): context.stop y system.stop (asíncrono) | `ctx.stop(&ActorRef)`, `system.stop(&ActorRef)` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:422-440 ;… | - |
| AC-CELL-011 | PoisonPill | mensaje de control interceptado por la celda | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:422-440 ;… | - |
| AC-CELL-012 | Kill | ídem PoisonPill | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:442-463 ;… | - |
| AC-CELL-013.1 | Prioridad de mensajes de sistema sobre los de usuario (bucle del mailbox) | dos colas por actor (sistema, usuario) y bucle: drenar sistema → un mensaje de usuario →… | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:462-508 ;… | Q-X-4 |
| AC-CELL-013.2 | Diferimiento (stash) de mensajes de sistema según el estado de la celda | `enum CellState { Default, Suspended, WaitingForChildren }` + cola de sistema diferida | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:863-914 | - |
| AC-CELL-013.3 | Mensajes de sistema a una celda terminada → deadLetters | mailbox cerrado sustituido por un mailbox "dead letter" que reenvía a deadLetters | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:144-151 | - |
| AC-CELL-014 | Familia SystemMessage (Create, Recreate, Suspend, Resume, Terminate,… | `enum SystemMessage` + `VecDeque` | actor-tests/src/test/scala/org/apache/pekko/dispatch/sysmsg/SystemMessageListSpec.scala:2… | - |
| AC-CELL-015.1 | Contenedor de hijos: estados Normal/Terminating y reserva de nombres | `enum ChildrenState` + `BTreeMap<String, ChildStats>` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorMiscSpec.scala:101-141 ;… | - |
| AC-CELL-015.2 | Contenedor de hijos: razones de espera y transición al terminar el último hijo | `ChildrenState::Terminating { to_die, reason }` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:863-914… | - |
| AC-CELL-016 | Arranque de un hijo: reserva, uid nuevo, hereda suspensiones del padre,… | `make_child` con reserva atómica y suspensión heredada | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorMiscSpec.scala:86-99 ;… | Q-MBX-11, Q-REF-5 |
| AC-CELL-017 | ReceiveTimeout | timer por actor en la celda (tarea cancelable) con `ReceiveTimeoutDeadline` | actor-tests/src/test/scala/org/apache/pekko/actor/ReceiveTimeoutSpec.scala:90-350 | Q-X-4 |
| AC-CELL-018 | API de ActorContext: self, props, system, dispatcher, parent, getters Java | `Context` con métodos equivalentes | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:367-388 ;… | Q-X-4, Q-X-8 |
| AC-CELL-019 | El contexto sólo es válido dentro del procesamiento del actor | `&mut Context` prestado sólo durante el handler | actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:193-230 ;… | Q-CELL-3 |
| AC-CELL-020 | El mensaje que causó el fallo no se reprocesa tras restart | - | actor-tests/src/test/scala/org/apache/pekko/actor/Ticket669Spec.scala:36-47 | - |
| AC-CELL-021 | Interrupción de hilo: InterruptedException → ActorInterruptedException al padre | N/A (Rust no interrumpe hilos) | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:423-478 | Q-CELL-2 |
| AC-CELL-022 | Hooks de instrumentación… | trait `Interceptor` o métodos `pub(crate)` con implementación por defecto | derivada del código (ver bloque de contrato) | - |

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

Spec 12 `AC-CELL-2` (`12-ac-cell-2.md`).
