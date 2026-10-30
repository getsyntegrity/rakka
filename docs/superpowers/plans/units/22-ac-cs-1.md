# Spec 22 — AC-CS (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 15. **Tareas:** 3 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-001.
- Spec 10 `AC-SCH-1` (`10-ac-sch-1.md`), por las filas AC-SCH-001.
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-004.
- Spec 17 `AC-DW-1` (`17-ac-dw-1.md`), por las filas AC-DW-001.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CS-1`, `Q-CS-2`, `Q-CS-3`, `Q-CS-4`, `Q-CS-5`, `Q-CS-6`, `Q-X-1`, `Q-X-4`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Orden topologico de fases (DAG) y deteccion de ciclos (+5 filas)

Filas: AC-CS-001.1, AC-CS-001.2, AC-CS-002, AC-CS-003.1, AC-CS-003.2, AC-CS-004.1. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Timeout por fase y recover (continuar o abortar con TimeoutException) (+5 filas)

Filas: AC-CS-004.2, AC-CS-004.3, AC-CS-005, AC-CS-006.1, AC-CS-006.2, AC-CS-007.1. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Ejecucion de CoordinatedShutdown desde el hook de proceso (run-by-jvm-shutdown-hook) (+2 filas)

Filas: AC-CS-007.2, AC-CS-008, AC-CS-009. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-CS-001.1 | Orden topologico de fases (DAG) y deteccion de ciclos | `fn topological_sort(phases: &BTreeMap<String, Phase>) -> Result<Vec<String>, CycleError>` | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:37-65,71-… | Q-CS-1 |
| AC-CS-001.2 | phasesFromConfig: lectura de fases (depends-on, timeout, recover, enabled,… | `fn phases_from_config(&Config) -> Result<BTreeMap<String, Phase>>` | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:501-520 | Q-CS-1, Q-X-1 |
| AC-CS-002 | Fases predefinidas (incluye las de cluster, vacias en local) y constantes… | constantes `PHASE_*: &str` y config por defecto embebida | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:123-139 | Q-CS-3 |
| AC-CS-003.1 | addTask / addCancellableTask: validaciones y semantica de fase | `add_task(phase: &str, name: &str, task: impl FnOnce() -> BoxFuture<Done>) ->… | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:141-168 | Q-CS-1, Q-X-9 |
| AC-CS-003.2 | Cancelacion (Cancellable), registro duplicado, tareas anadidas durante el run… | registro `Vec<Arc<TaskDef>>` por fase con estado atomico {Pending, Cancelled,… | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:170-363,4… | Q-CS-2 |
| AC-CS-004.1 | run(reason, fromPhase): una sola vez, fases en orden, futuro compartido,… | `fn run(&self, reason: Reason, from_phase: Option<&str>) -> SharedFuture<Done>` | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:141-168,3… | Q-CS-2, Q-X-4 |
| AC-CS-004.2 | Timeout por fase y recover (continuar o abortar con TimeoutException) | tokio::time::timeout por fase + politica `recover` | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:402-462 | Q-CS-2, Q-X-5 |
| AC-CS-004.3 | Fases deshabilitadas (enabled = off) | campo `enabled` en Phase | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:464-483 | Q-CS-2 |
| AC-CS-005 | Reasons y reason-overrides (exit-code, terminate-actor-system, exit-jvm) | trait `Reason` con `fn config_key(&self) -> &str` (nombre estable como el de la clase JVM) | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:522-533 | Q-CS-4 |
| AC-CS-006.1 | Fase actor-system-terminate: terminate-actor-system, exit-jvm, exit-code | tarea interna registrada en `CoordinatedShutdown::init`; `std::process::exit` configurable | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:535-544 | Q-CS-5 |
| AC-CS-006.2 | Integracion con ActorSystem.terminate/close y combinacion de config invalida | Settings::validate + `ActorSystem::terminate`/`close` delegando en CoordinatedShutdown | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:546-582 | Q-CS-5 |
| AC-CS-007.1 | addJvmShutdownHook / addCancellableJvmShutdownHook (API de hooks de apagado… | trait `ProcessShutdownHooks { add_hook, remove_hook }` +… | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:584-659,7… | Q-CS-6 |
| AC-CS-007.2 | Ejecucion de CoordinatedShutdown desde el hook de proceso… | registro del hook de senal (SIGTERM/SIGINT) opcional | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:600-659 | Q-CS-6 |
| AC-CS-008 | addActorTerminationTask (mensaje de stop opcional, espera Terminated) y… | `add_actor_termination_task(phase, name, actor, stop: Option<Msg>)` + watcher interno | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:692-753 | Q-CS-2 |
| AC-CS-009 | timeout(phase), totalTimeout, default-phase-timeout, close-actor-system-timeout | `fn timeout(&self, phase) -> Result<Duration>` y `fn total_timeout(&self) -> Duration` | - (sin spec dedicado; usado en CoordinatedShutdownSpec:287-289 indirectamente) | Q-CS-2 |

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

Spec 23 `AC-SER-1` (`23-ac-ser-1.md`).
