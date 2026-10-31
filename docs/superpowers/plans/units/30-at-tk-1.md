# Spec 30 — AT-TK (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 19. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 31 `AT-CTX-1` (`31-at-ctx-1.md`), por las filas AT-CTX-002, AT-CTX-006. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 41 `AT-ASK-1` (`41-at-ask-1.md`), por las filas AT-ASK-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 46 `AT-CS-1` (`46-at-cs-1.md`), por las filas AT-CS-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 38 `AT-LOG-1` (`38-at-log-1.md`), por las filas AT-LOG-001, AT-LOG-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 37 `AT-EVT-1` (`37-at-evt-1.md`), por las filas AT-EVT-001, AT-EVT-002, AT-EVT-003, AT-EVT-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-LOG-1`, `Q-T-SER-1`, `Q-T-TK-1`, `Q-T-TK-2`, `Q-T-TK-3`, `Q-T-TK-4`, `Q-X-7`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

jars pekko-actor-typed; el jar `pekko-actor-testkit-typed` NO está en tools/oracle/fetch-jars.sh: agregarlo con su SHA-1 es parte de la primera tarea. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ActorTestKit: construccion, nombre del sistema y configuracion (+5 filas)

Filas: AT-TK-001.1, AT-TK-001.2, AT-TK-001.3, AT-TK-002.1, AT-TK-002.2, AT-TK-002.3. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: TestProbe: awaitAssert, within, remaining y dilatacion de tiempo (+5 filas)

Filas: AT-TK-002.4, AT-TK-003, AT-TK-004.1, AT-TK-004.2, AT-TK-005, AT-TK-006. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: ManualTime (scheduler disparado explicitamente) (+5 filas)

Filas: AT-TK-007, AT-TK-008, AT-TK-009, AT-TK-010, AT-TK-011, AT-TK-012. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: LogbackUtil y LogCapturingExtension: utilidades de captura de logs sobre Logback para…

Filas: AT-TK-013. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

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
| AT-TK-002.4 | TestProbe: awaitAssert, within, remaining y dilatacion de tiempo | await_assert(f, max, interval) y within(min,max,f). | Sin test leido de within; awaitAssert usado en… | Q-T-TK-1 |
| AT-TK-003 | BehaviorTestKit (test sincronico: run, signal, effects, selfInbox, childInbox,… | BehaviorTestKit<M> sincronico; requiere interprete de Behavior sin runtime (Q-T-TK-4). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.1 | Effects: AskInitiated (ask reificado) | Effect::AskInitiated<Req,Res,M> con respond_with/timeout/adapt_*. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-004.2 | Effects: Spawned, SpawnedAnonymous, Stopped, Watched, WatchedWith, Unwatched,… | enum Effect y EffectfulContext. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-005 | TestInbox / ReplyInbox / StatusReplyInbox | TestInbox<M>. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4 |
| AT-TK-006 | LoggingTestKit y LogCapturing (filtros por nivel, mensaje, causa, MDC,… | Layer de tracing que captura eventos con filtros (Q-T-LOG-1). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/LoggingE… | Q-T-LOG-1, Q-T-TK-2 |
| AT-TK-007 | ManualTime (scheduler disparado explicitamente) | Reloj inyectable (clock trait) en el scheduler; ManualTime::time_passes. | Sin test leido de ManualTime | - |
| AT-TK-008 | Probes de eventos del sistema: createUnhandledMessageProbe,… | ActorTestKit::create_unhandled_probe/dead_letter_probe/dropped_probe. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTes… | - |
| AT-TK-009 | Integraciones de framework (ScalaTestWithActorTestKit, JUnit… | Helper RAII/macro para #[tokio::test]; sin JUnit. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5Te… | Q-T-SER-1, Q-T-TK-3 |
| AT-TK-010 | TestKitSettings y pekko.actor.testkit.typed.* (timefactor, timeouts) | struct TestKitSettings con dilated(). | Uso indirecto en… | - |
| AT-TK-011 | ActorSystemStub: ActorSystem mínimo sin runtime real que usa BehaviorTestKit… | stub de sistema que cumple el trait de sistema tipado con operaciones no soportadas… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-X-8 |
| AT-TK-012 | ControlledExecutor y DebugRef: ejecutor manual (cola de tareas que se corre a… | ejecutor FIFO manual + ref que registra mensajes recibidos | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | - |
| AT-TK-013 | LogbackUtil y LogCapturingExtension: utilidades de captura de logs sobre… | captura de eventos de log por test sobre el backend elegido en Q-X-7 | actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/LogCapturi… | Q-X-7 |

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

Spec 31 `AT-CTX-1` (`31-at-ctx-1.md`).
