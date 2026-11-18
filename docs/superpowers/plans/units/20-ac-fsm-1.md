# Spec 20 — AC-FSM (1/1)

**Estado:** bloqueada. **Filas:** 11. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-001, AC-CELL-008.1, AC-CELL-008.2.
- Spec 21 `AC-RT-1` (`21-ac-rt-1.md`), por las filas AC-RT-015. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 10 `AC-SCH-1` (`10-ac-sch-1.md`), por las filas AC-SCH-001, AC-SCH-011.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-011.

Decisiones:

- Bloqueantes: `Q-FSM-4`.
- Propuestas aún sin decisión del Owner: `Q-CS-7`, `Q-FSM-1`, `Q-FSM-2`, `Q-FSM-3`, `Q-RT-8`, `Q-X-3`, `Q-X-4`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: DSL de estado: State (goto/stay/stop/using/replying/forMax), Event(event, stateData),… (+5 filas)

Filas: AC-FSM-001.1, AC-FSM-001.2, AC-FSM-002, AC-FSM-003, AC-FSM-004, AC-FSM-005. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Terminacion: stop(), stop(reason), onTermination(StopEvent) y estado inexistente (+4 filas)

Filas: AC-FSM-006.1, AC-FSM-006.2, AC-FSM-007, AC-FSM-008, AC-FSM-009. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-FSM-001.1 | DSL de estado: State (goto/stay/stop/using/replying/forMax), Event(event,… | trait `Fsm<S, D>`; `struct FsmState<S, D> { name, data, timeout: Option<Duration>,… | actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:209-222,354-386 ;… | Q-FSM-1 |
| AC-FSM-001.2 | Registro de when(estado, stateTimeout), initialize/startWith, despacho de… | registro `HashMap<S, Vec<StateFn>>` y `fn process_event(&mut self, ev: Event<D>) ->… | actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:29-108,120-169 ;… | Q-FSM-1 |
| AC-FSM-002 | Transiciones: onTransition (goto incluido al mismo estado, no en stay),… | `on_transition(Fn(&S,&S,&TransitionCtx))` + `subscribe(ref)`; listeners en… | actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:83-203 ;… | Q-FSM-1, Q-RT-8, Q-X-3 |
| AC-FSM-003 | State timeout (StateTimeout; cancelado por cualquier mensaje; forMax;… | `state_timeout` en `FsmState` + generacion `u64`; el timer usa el scheduler del sistema | actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:35-75,115-129 ;… | Q-FSM-2, Q-X-4 |
| AC-FSM-004 | Timers con nombre (startSingleTimer, startTimerWithFixedDelay/AtFixedRate,… | `start_single_timer(name, msg, delay)` etc. sobre scheduler; `HashMap<String,… | actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:77-140 ;… | Q-CS-7, Q-FSM-2, Q-X-4 |
| AC-FSM-005 | whenUnhandled y manejo por defecto de eventos no manejados (warning + stay) | `when_unhandled(fn)`; defecto `log::warn!("unhandled event {:?} in state {:?}")` | actor-tests/src/test/scala/org/apache/pekko/actor/FSMTimingSpec.scala:142-160 ;… | Q-FSM-3 |
| AC-FSM-006.1 | Terminacion: stop(), stop(reason), onTermination(StopEvent) y estado inexistente | `stop(reason)`, `on_termination(Fn(StopEvent))`; razon `StopReason { Normal, Shutdown,… | actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:171-222 | Q-FSM-4, Q-X-5 |
| AC-FSM-006.2 | Terminacion externa: stop del actor -> StopEvent(Shutdown), timers cancelados… | implementacion en el hook post_stop del trait Fsm | actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:189-207,224-261,96-1… | Q-FSM-4 |
| AC-FSM-007 | LoggingFSM: log rotativo (logDepth), getLog; debug.fsm | `LoggingFsm` como extension del trait con `ring: Vec<Option<LogEntry>>` | actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:263-333 | Q-CS-7, Q-FSM-3 |
| AC-FSM-008 | transform(...) using | combinador `transform(f).using(g)` sobre `StateFn` | actor-tests/src/test/scala/org/apache/pekko/actor/FSMActorSpec.scala:335-352 | Q-FSM-1 |
| AC-FSM-009 | Acceso a stateName/stateData/nextStateData e initialize antes de startWith… | metodos del trait Fsm que devuelven Result / panic documentado segun Q-X-5 | actor-tests/src/test/scala/org/apache/pekko/actor/FSMTransitionSpec.scala:180-203 | Q-FSM-1, Q-X-5 |

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

Spec 21 `AC-RT-1` (`21-ac-rt-1.md`).
