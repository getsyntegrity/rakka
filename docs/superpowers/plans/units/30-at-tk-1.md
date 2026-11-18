# Spec 30 — AT-TK (1/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-001.
- Spec 33 `AT-SYS-1` (`33-at-sys-1.md`), por las filas AT-SYS-001, AT-SYS-008. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 31 `AT-TK-2` (`31-at-tk-2.md`), por las filas AT-TK-010, AT-TK-011, AT-TK-012, AT-TK-013. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-002, AT-CTX-003, AT-CTX-004, AT-CTX-005, AT-CTX-006, AT-CTX-007, AT-CTX-008, AT-CTX-009, AT-CTX-011, AT-CTX-012, AT-CTX-013, AT-CTX-016, AT-CTX-018. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 42 `AT-ASK-1` (`42-at-ask-1.md`), por las filas AT-ASK-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 47 `AT-CS-1` (`47-at-cs-1.md`), por las filas AT-CS-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 39 `AT-LOG-1` (`39-at-log-1.md`), por las filas AT-LOG-001, AT-LOG-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 43 `AT-RCP-1` (`43-at-rcp-1.md`), por las filas AT-RCP-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 34 `AT-TMR-1` (`34-at-tmr-1.md`), por las filas AT-TMR-001, AT-TMR-002, AT-TMR-003, AT-TMR-004, AT-TMR-005, AT-TMR-007.1. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 29 `AT-REF-1` (`29-at-ref-1.md`), por las filas AT-REF-001.
- Spec 38 `AT-EVT-1` (`38-at-evt-1.md`), por las filas AT-EVT-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 12 `AC-TK-1` (`12-ac-tk-1.md`), por las filas AC-TK-001.5.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-CTX-1`, `Q-T-JAPI-1`, `Q-T-LOG-1`, `Q-T-TK-1`, `Q-T-TK-2`, `Q-T-TK-3`, `Q-T-TK-4`, `Q-T-TK-5`, `Q-T-TMR-1`, `Q-X-1`, `Q-X-7`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra pekko-actor-testkit-typed 2.0.0: agregar la dependencia a `oracle/pom.xml` es parte de la primera tarea. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ActorTestKit: construccion, nombre del sistema y configuracion (+5 filas)

Filas: AT-TK-001.1, AT-TK-001.2, AT-TK-001.3, AT-TK-002.1, AT-TK-002.2, AT-TK-002.3. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: TestProbe: awaitAssert (+5 filas)

Filas: AT-TK-002.4, AT-TK-002.5, AT-TK-002.6, AT-TK-003.1, AT-TK-003.2, AT-TK-003.3. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: BehaviorTestKit: hijos, selfInbox y childInbox/childTestKit (+5 filas)

Filas: AT-TK-003.4, AT-TK-003.5, AT-TK-003.6, AT-TK-003.7, AT-TK-004.1, AT-TK-004.2. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Effects: Watched, WatchedWith y Unwatched (+5 filas)

Filas: AT-TK-004.3, AT-TK-004.4, AT-TK-004.5, AT-TK-004.6, AT-TK-004.7, AT-TK-005.1. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: ReplyInbox (+5 filas)

Filas: AT-TK-005.2, AT-TK-005.3, AT-TK-006.1, AT-TK-006.2, AT-TK-006.3, AT-TK-006.4. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-TK-001.1 | ActorTestKit: construccion, nombre del sistema y configuracion | ActorTestKit::new()/with_name/with_config; nombre via macro/type_name. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTes… | Q-T-TK-3 |
| AT-TK-001.2 | ActorTestKit: spawn, stop y falla de spawn | spawn/stop asincronos o bloqueantes (Q-T-TK-1) sobre un guardian de testkit. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTes… | Q-T-TK-1 |
| AT-TK-001.3 | ActorTestKit: shutdown | shutdown(timeout, throw) sobre el sistema. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTes… | Q-T-TK-1 |
| AT-TK-002.1 | TestProbe: expectMessage, expectMessageType, expectNoMessage, receiveMessage(s) | TestProbe<M> con expect_message/expect_message_type/expect_no_message/receive_message(s)… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProb… | Q-T-TK-1 |
| AT-TK-002.2 | TestProbe: fishForMessage y FishingOutcome | fish_for_message(max, \|m\| FishingOutcome). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProb… | Q-T-TK-1 |
| AT-TK-002.3 | TestProbe: expectTerminated y stop | expect_terminated(&ActorRef, max); probe.stop(). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProb… | Q-T-TK-1 |
| AT-TK-002.4 | TestProbe: awaitAssert | await_assert(f, max, interval) con sleep async y reloj inyectable; panic/Err recuperable… | actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeT… | Q-T-TK-1 |
| AT-TK-002.5 | TestProbe: within, remaining, remainingOr, remainingOrDefault | within(min,max,f) y remaining*(); reloj monotono inyectable. | actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/TestProbeT… | Q-T-TK-1 |
| AT-TK-002.6 | TestProbe y TestKitSettings: tabla de dilatacion de plazos (timefactor) | TestKitSettings::dilated() y una llamada explicita por operacion; copiar la tabla… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProb… | Q-T-TK-1, Q-X-1 |
| AT-TK-003.1 | BehaviorTestKit: construccion, ruta, sistema y configuracion | BehaviorTestKit::new/with_name/with_config sobre interprete sin runtime (Q-T-TK-4). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4, Q-X-8 |
| AT-TK-003.2 | BehaviorTestKit: run, runOne, signal y estado del comportamiento | run/run_one/signal/current_behavior/returned_behavior/is_alive sobre interprete puro. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4, Q-X-8 |
| AT-TK-003.3 | BehaviorTestKit: cola de efectos (retrieveEffect, expectEffect*, hasEffects) | retrieve_effect/retrieve_all_effects/has_effects/expect_effect*; Effect enum sellado con… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-1, Q-T-TK-4 |
| AT-TK-003.4 | BehaviorTestKit: hijos, selfInbox y childInbox/childTestKit | arbol de BehaviorTestKit hijos en BTreeMap; child_inbox/child_test_kit/self_inbox. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-003.5 | BehaviorTestKit: logEntries y clearLog (CapturedLogEvent) | Vec<CapturedLogEvent> por testkit alimentado por la fachada de log (Q-T-LOG-1). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-LOG-1, Q-T-TK-4, Q-X-7 |
| AT-TK-003.6 | BehaviorTestKit: receptionistInbox | receptionist_inbox() como TestInbox compartido. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-003.7 | BehaviorTestKit: runAsk y runAskWithStatus | run_ask/run_ask_with_status. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.1 | Effects: AskInitiated (ask reificado) | Effect::AskInitiated<Req,Res,M> con respond_with/timeout/adapt_*. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.2 | Effects: Spawned, SpawnedAnonymous y Stopped | Effect::Spawned/SpawnedAnonymous/Stopped con PartialEq que ignora ref; igualdad de… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.3 | Effects: Watched, WatchedWith y Unwatched | variantes Watched/WatchedWith/Unwatched sin entrega automatica. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.4 | Effects: MessageAdapter, SpawnedAdapter y SpawnedAnonymousAdapter | Effect::MessageAdapter con type_id y funcion; adaptacion por tipo exacto (Q-T-CTX-1). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-CTX-1, Q-T-TK-4 |
| AT-TK-004.5 | Effects: ReceiveTimeoutSet, ReceiveTimeoutCancelled, Scheduled y NoEffects | variantes ReceiveTimeoutSet/ReceiveTimeoutCancelled/Scheduled/NoEffects. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.6 | Effects: TimerScheduled, TimerCancelled y simulacion de timers | Effect::TimerScheduled{.., send} y TimerMode de 5 variantes; claves segun Q-T-TMR-1. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4, Q-T-TK-5, Q-T-TMR-1 |
| AT-TK-004.7 | Effects: fabricas Effects (scaladsl/javadsl) y convenciones de Effect | constructores asociados de Effect; una sola API Rust cubre scaladsl y javadsl… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-JAPI-1, Q-T-TK-4 |
| AT-TK-005.1 | TestInbox | TestInbox<M> FIFO con ref local; errores y rechazo de ask. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-1, Q-T-TK-4 |
| AT-TK-005.2 | ReplyInbox | ReplyInbox<R> con Option<TestInbox<R>>. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-005.3 | StatusReplyInbox | StatusReplyInbox<R> sobre Result<R, StatusError>; expect_done solo para R=Done. | cluster-sharding-typed/src/test/scala/docs/org/apache/pekko/cluster/sharding/typed/Accoun… | Q-T-TK-4 |
| AT-TK-006.1 | LoggingTestKit: condiciones de coincidencia (matches) y LoggingEvent | builder inmutable sobre LogEvent y matches(&LogEvent); cause por downcast del error… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingE… | Q-T-LOG-1, Q-T-TK-2 |
| AT-TK-006.2 | LoggingTestKit: fabricas (error, warn, info, debug, trace, messageContains,… | LoggingTestKit::{empty, message_contains, trace.., dead_letters, custom}. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingE… | Q-T-LOG-1, Q-T-TK-2 |
| AT-TK-006.3 | LoggingTestKit: expect (ocurrencias, exceso, plazos y ausencia) | expect(sistema, \\|\\| ..) con guard RAII que retira el filtro; plazo filter-leeway y… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppe… | Q-T-LOG-1, Q-T-TK-2, Q-X-7 |
| AT-TK-006.4 | LoggingTestKit: captura de eventos (TestAppender) y logger name | tracing_subscriber::Layer de captura global (set_global_default perezoso) con registro… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppe… | Q-T-LOG-1, Q-T-TK-2, Q-X-7 |

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

Spec 31 `AT-TK-2` (`31-at-tk-2.md`).
