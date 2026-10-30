# Spec 35 — AT-SUP (1/1)

**Estado:** bloqueada. **Filas:** 24. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 28 `AT-SIG-1` (`28-at-sig-1.md`), por las filas AT-SIG-001, AT-SIG-002, AT-SIG-005, AT-SIG-006, AT-SIG-007.
- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-004, AT-BEH-007, AT-BEH-010, AT-BEH-011, AT-BEH-013, AT-BEH-015, AT-BEH-024.
- Spec 33 `AT-TMR-1` (`33-at-tmr-1.md`), por las filas AT-TMR-001, AT-TMR-006.
- Spec 31 `AT-CTX-1` (`31-at-ctx-1.md`), por las filas AT-CTX-005, AT-CTX-006.
- Spec 34 `AT-STASH-1` (`34-at-stash-1.md`), por las filas AT-STASH-001, AT-STASH-006.
- Spec 32 `AT-SYS-1` (`32-at-sys-1.md`), por las filas AT-SYS-006, AT-SYS-008.
- Spec 37 `AT-EVT-1` (`37-at-evt-1.md`), por las filas AT-EVT-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 38 `AT-LOG-1` (`38-at-log-1.md`), por las filas AT-LOG-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 36 `AT-ADP-1` (`36-at-adp-1.md`), por las filas AT-ADP-002, AT-ADP-005. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: `Q-T-BEH-1`, `Q-T-SUP-3`.
- Propuestas aún sin decisión del Owner: `Q-T-BEH-5`, `Q-T-SIG-1`, `Q-T-STASH-1`, `Q-T-SUP-1`, `Q-T-SUP-2`, `Q-T-SUP-4`, `Q-T-SUP-5`, `Q-T-SUP-6`, `Q-T-SUP-7`, `Q-T-SUP-8`, `Q-T-TMR-4`, `Q-X-5`, `Q-X-6`, `Q-X-7`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Supervision por defecto: Stop (+5 filas)

Filas: AT-SUP-001, AT-SUP-002, AT-SUP-003, AT-SUP-004, AT-SUP-005, AT-SUP-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: SupervisorStrategy.restartWithBackoff(min, max, randomFactor): espera exponencial (+5 filas)

Filas: AT-SUP-007.1, AT-SUP-007.2, AT-SUP-007.3, AT-SUP-008, AT-SUP-009, AT-SUP-010. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Supervision anidada y orden de manejo (distintas excepciones) (+5 filas)

Filas: AT-SUP-011, AT-SUP-012, AT-SUP-013, AT-SUP-014, AT-SUP-015, AT-SUP-016. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Senales internas ScheduledRestart y ResetRestartCount (+5 filas)

Filas: AT-SUP-017, AT-SUP-018, AT-SUP-019, AT-SUP-020, AT-SUP-021, AT-SUP-022. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-SUP-001 | Supervision por defecto: Stop | El runtime trata el error no supervisado como fin del actor (Stop) y notifica… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:129-1… | Q-X-5 |
| AT-SUP-002 | Behaviors.supervise(b).onFailure[Thr](strategy) y onFailure(clazz, strategy) | behaviors::supervise(b).on_failure::<E: Error + 'static>(strategy) usando downcast_ref… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:139-1… | Q-T-SUP-1, Q-X-5 |
| AT-SUP-003 | SupervisorStrategy.stop | SupervisorStrategy::Stop{logging}. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:397-4… | - |
| AT-SUP-004 | SupervisorStrategy.resume | SupervisorStrategy::Resume. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:163-1… | Q-T-SUP-5 |
| AT-SUP-005 | SupervisorStrategy.restart (ilimitado) | SupervisorStrategy::Restart{limit: Option<(u32, Duration)>, stop_children,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:149-1… | Q-T-BEH-1, Q-T-SUP-4 |
| AT-SUP-006 | RestartSupervisorStrategy.withLimit(maxNrOfRetries, withinTimeRange) | Restart{limit: Some((n, window))} con contador y deadline por Instant. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:224-2… | Q-T-SUP-7, Q-T-SUP-8, Q-T-TMR-4 |
| AT-SUP-007.1 | SupervisorStrategy.restartWithBackoff(min, max, randomFactor): espera… | Backoff{min, max, random_factor} con jitter y reloj inyectables. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:912-9… | Q-T-SUP-2, Q-T-SUP-8 |
| AT-SUP-007.2 | SupervisorStrategy.withResetBackoffAfter: reinicio del contador de backoff… | Backoff.reset_after (default (min+max)/2) y senal interna ResetRestartCount. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:991-1… | Q-T-SUP-3, Q-T-SUP-6 |
| AT-SUP-007.3 | BackoffSupervisorStrategy.withMaxRestarts: tope de reinicios fallidos seguidos | Backoff.max_restarts: Option<u32>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:960-9… | Q-T-SUP-3 |
| AT-SUP-008 | Hijos detenidos en restart (withStopChildren) | Opcion stop_children: bool; la espera de Terminated se modela como estado 'restarting'… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:575-7… | - |
| AT-SUP-009 | Stash durante restart/backoff (withStashCapacity) y Dropped al llenarse | Buffer acotado VecDeque en el estado del supervisor; Dropped por EventStream. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:882-9… | Q-T-STASH-1, Q-T-SUP-2, Q-X-6 |
| AT-SUP-010 | Logging de supervision (withLoggingEnabled, withLogLevel, withCriticalLogLevel) | Campos logging_enabled y level en la estrategia; tracing::event! con nivel dinamico. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1441-… | Q-X-7 |
| AT-SUP-011 | Supervision anidada y orden de manejo (distintas excepciones) | Pila de interceptores de supervision; is_same compara TypeId del error. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:176-2… | Q-T-SUP-1 |
| AT-SUP-012 | La supervision se mantiene al cambiar de Behavior (re-wrap) | El interceptor de supervision vive fuera del Behavior interno y sobrevive a los cambios. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1270-… | - |
| AT-SUP-013 | Rechazo de AbstractBehavior supervisado directamente | En Rust: supervise exige una fabrica (Fn) por tipo, haciendo imposible el caso. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1527-… | Q-T-BEH-1, Q-T-BEH-5 |
| AT-SUP-014 | DeathPactException supervisable | ActorError::DeathPact matcheable por on_failure. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1408-… | - |
| AT-SUP-015 | Supervision de fallas en funciones adaptadoras | Ver AT-SIG-007. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1575-… | Q-X-5 |
| AT-SUP-016 | Timers cancelados en restart y descarte de TimerMsg de encarnaciones previas | Generacion por timer + id de encarnacion. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:174-230,… | - |
| AT-SUP-017 | Senales internas ScheduledRestart y ResetRestartCount | Variantes internas de Signal con id de duenio. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1032-… | Q-T-SIG-1, Q-T-SUP-6 |
| AT-SUP-018 | Falla durante unstash (UnstashException) y supervision | El error de unstash lleva el Behavior en curso (ActorError::Unstash{cause, behavior}). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:47… | Q-T-STASH-1 |
| AT-SUP-019 | Supervision sobre Behavior que para (inicialmente, tras setup o tras restart) | Sin logica extra. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1651-… | - |
| AT-SUP-020 | Padre clasico supervisando hijo typed: default Stop | Solo aplica si hay capa clasica (ver Q de arquitectura en AT-SYS-011). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervis… | Q-T-SUP-7, Q-X-8 |
| AT-SUP-021 | Padre typed supervisando hijo clasico: decider clasico | Solo con capa clasica. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/TypedSupervisin… | Q-T-SUP-7 |
| AT-SUP-022 | rethrowTypedFailure: falla del guardian / systemActorOf vs hijo typed | Parametro interno del runtime al crear el actor (report_failure_to_parent: bool). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-163;… | Q-X-5, Q-X-8 |

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

Spec 36 `AT-ADP-1` (`36-at-adp-1.md`).
