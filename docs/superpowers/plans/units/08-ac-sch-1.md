# Spec 08 — AC-SCH (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 17. **Tareas:** 3 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 07 `AC-DSP-1` (`07-ac-dsp-1.md`), por las filas AC-DSP-001, AC-DSP-016.1, AC-DSP-016.2.
- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-002.4.
- Spec 03 `AC-CFG-1` (`03-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-016.
- Spec 11 `AC-CELL-1` (`11-ac-cell-1.md`), por las filas AC-CELL-017. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 06 `AC-MBX-1` (`06-ac-mbx-1.md`), por las filas AC-MBX-016.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-SCH-3`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Scheduler.scheduleOnce (Runnable, closure, o mensaje a ActorRef) → Cancellable (+5 filas)

Filas: AC-SCH-001, AC-SCH-002, AC-SCH-003, AC-SCH-004, AC-SCH-005, AC-SCH-006. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Tarea periódica a actor se detiene si el receptor terminó o si la tarea lanza (+5 filas)

Filas: AC-SCH-007, AC-SCH-008.1, AC-SCH-008.2, AC-SCH-008.3, AC-SCH-009, AC-SCH-010. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Timers / TimerScheduler: startSingleTimer, startTimerWithFixedDelay,… (+4 filas)

Filas: AC-SCH-011, AC-SCH-012, AC-SCH-013, AC-SCH-014, AC-SCH-015. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-SCH-001 | Scheduler.scheduleOnce (Runnable, closure, o mensaje a ActorRef) → Cancellable | `fn schedule_once(&self, delay, f) -> Cancellable` sobre runtime de timers | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:98-121,158-178,478-… | Q-SCH-1 |
| AC-SCH-002 | scheduleWithFixedDelay | loop async con sleep tras la tarea | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:220-251,453-475,724… | Q-SCH-1 |
| AC-SCH-003 | scheduleAtFixedRate | `tokio::time::interval` con `MissedTickBehavior::Burst` | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:423-449,704-722 | Q-SCH-1, Q-SCH-4 |
| AC-SCH-004 | schedule(...) deprecado (= fixed rate) | - | testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:24… | Q-SCH-4 |
| AC-SCH-005 | Cancellable: cancel() true sólo si esta llamada canceló; isCancelled;… | `struct Cancellable(Arc<AtomicU8>)` | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:123-156,319-349 | - |
| AC-SCH-006 | SchedulerTask.cancelled() callback | trait opcional `on_cancel` | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:137-146,351-369 | - |
| AC-SCH-007 | Tarea periódica a actor se detiene si el receptor terminó o si la tarea lanza | chequear `is_terminated()` antes de reenviar | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:255-317 | - |
| AC-SCH-008.1 | LARS: validación de tick-duration, ticks-per-wheel, shutdown-timeout y… | `SchedulerSettings::from_config` con los mismos mensajes y cfg!(windows) | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:864-878 | Q-SCH-1, Q-X-4 |
| AC-SCH-008.2 | LARS: redondeo al tick (roundUp), retardos <= 0, límite de retardo y de… | funciones puras round_up, check_period, check_max_delay con los mismos mensajes | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:480-487 ;… | Q-SCH-1, Q-X-4 |
| AC-SCH-008.3 | LARS: mecánica de la rueda (buckets, vueltas, tick y reloj que dan la vuelta,… | `WheelScheduler<Clock, Waiter>` con rueda portada y reloj/espera inyectables | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:489-663 ;… | Q-SCH-1, Q-X-4 |
| AC-SCH-009 | close() del scheduler: ejecuta TaskRunOnClose, rechaza nuevas tareas con… | - | actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:665-701 | Q-SCH-1 |
| AC-SCH-010 | scheduler.implementation pluggable (constructor Config, LoggingAdapter,… | trait `Scheduler` inyectable en el builder | testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:24… | Q-SCH-2, Q-X-2 |
| AC-SCH-011 | Timers / TimerScheduler: startSingleTimer, startTimerWithFixedDelay,… | `ctx.timers()` con `HashMap<Key, (gen, Cancellable)>` | actor-tests/src/test/scala/org/apache/pekko/actor/TimerSpec.scala:201-307 | Q-SCH-3 |
| AC-SCH-012 | Mensajes de timer: AutoReceivedMessage y NotInfluenceReceiveTimeout | - | actor-tests/src/test/scala/org/apache/pekko/actor/TimerSpec.scala:309-315 ;… | Q-SCH-3 |
| AC-SCH-013 | Timers combinados con Stash | - | actor-tests/src/test/scala/org/apache/pekko/actor/TimerSpec.scala:423-452 | Q-SCH-3 |
| AC-SCH-014 | scheduler.shutdown-timeout (espera de parada del hilo del scheduler) | parámetro de cierre del scheduler | derivada del código (ver bloque de contrato) | Q-SCH-1 |
| AC-SCH-015 | Recuperación del hilo del timer ante excepción (LARS) | supervisión del hilo/tarea del reloj | derivada del código (ver bloque de contrato) | Q-X-5 |

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

Spec 09 `AC-EVT-1` (`09-ac-evt-1.md`).
