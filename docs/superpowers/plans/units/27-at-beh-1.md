# Spec 27 — AT-BEH (1/1)

**Estado:** bloqueada. **Filas:** 26. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 28 `AT-SIG-1` (`28-at-sig-1.md`), por las filas AT-SIG-001, AT-SIG-003, AT-SIG-005, AT-SIG-006, AT-SIG-007. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 37 `AT-EVT-1` (`37-at-evt-1.md`), por las filas AT-EVT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 35 `AT-SUP-1` (`35-at-sup-1.md`), por las filas AT-SUP-001, AT-SUP-003, AT-SUP-013, AT-SUP-022. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 38 `AT-LOG-1` (`38-at-log-1.md`), por las filas AT-LOG-001, AT-LOG-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 33 `AT-TMR-1` (`33-at-tmr-1.md`), por las filas AT-TMR-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 34 `AT-STASH-1` (`34-at-stash-1.md`), por las filas AT-STASH-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 29 `AT-REF-1` (`29-at-ref-1.md`), por las filas AT-REF-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: `Q-T-BEH-1`.
- Propuestas aún sin decisión del Owner: `Q-T-BEH-2`, `Q-T-BEH-3`, `Q-T-BEH-4`, `Q-T-BEH-5`, `Q-T-BEH-6`, `Q-T-CTX-2`, `Q-T-TMR-1`, `Q-X-3`, `Q-X-5`, `Q-X-6`, `Q-X-7`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Behavior[T]: tipo base sellado (@DoNotInherit) con tags internos y ExtensibleBehavior… (+5 filas)

Filas: AT-BEH-001, AT-BEH-002, AT-BEH-003, AT-BEH-004, AT-BEH-005, AT-BEH-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Behaviors.setup (DeferredBehavior) (+5 filas)

Filas: AT-BEH-007, AT-BEH-008.1, AT-BEH-008.2, AT-BEH-008.3, AT-BEH-009, AT-BEH-010. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Interprete de Behavior: start, canonicalize, interpretMessage, interpretSignal,… (+5 filas)

Filas: AT-BEH-011, AT-BEH-012, AT-BEH-013, AT-BEH-014, AT-BEH-015, AT-BEH-016. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Behaviors.monitor (+5 filas)

Filas: AT-BEH-017, AT-BEH-018, AT-BEH-019, AT-BEH-020, AT-BEH-021, AT-BEH-022. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: Reutilizacion de valores Behavior (+1 filas)

Filas: AT-BEH-023, AT-BEH-024. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-BEH-001 | Behavior[T]: tipo base sellado (@DoNotInherit) con tags internos y… | enum Behavior<M> { Same, Unhandled, Stopped(Option<PostStopFn>), Empty, Ignore,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:204-339… | Q-T-BEH-1, Q-T-BEH-4 |
| AT-BEH-002 | Behaviors.same | Variante Behavior::Same; validate_as_initial devuelve Err(InvalidInitialBehavior). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:242-252;… | - |
| AT-BEH-003 | Behaviors.unhandled | Behavior::Unhandled; el runtime publica UnhandledMessage{msg, sender, recipient} en el… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:256-266;… | Q-X-6 |
| AT-BEH-004 | Behaviors.stopped y stopped(postStop) | Behavior::Stopped(Option<Box<dyn FnOnce(&mut Ctx<M>)>>). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:268-277;… | - |
| AT-BEH-005 | Behaviors.empty | Behavior::Empty. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureS… | - |
| AT-BEH-006 | Behaviors.ignore | Behavior::Ignore. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureS… | - |
| AT-BEH-007 | Behaviors.setup (DeferredBehavior) | Behavior::Deferred(Box<dyn FnOnce(&mut ActorContext<M>) -> Behavior<M> + Send>); para… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:50-175;… | Q-T-BEH-1 |
| AT-BEH-008.1 | Behaviors.receive / receiveMessage: Receive[T] con y sin contexto | behaviors::receive(|ctx, msg| ...) y behaviors::receive_message(|msg| ...) devuelven un… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:349-381,… | Q-T-BEH-2 |
| AT-BEH-008.2 | Behaviors.receivePartial / receiveMessagePartial: funciones parciales con… | No hay PartialFunction: se modela con receive cuyo match devuelve Behavior::Unhandled en… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ReceivePartialSpec… | Q-T-BEH-2 |
| AT-BEH-008.3 | Behaviors.receiveMessageWithSame (desde 1.1.0) | behaviors::receive_message_with_same(|msg| ...). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:383-397 | Q-T-BEH-2 |
| AT-BEH-009 | receiveSignal y Receive.receiveSignal (manejo de senales) | Metodo builder .on_signal(|ctx, sig| ...) sobre el Behavior devuelto por receive; Signal… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/OnSignalSpec.scala… | - |
| AT-BEH-010 | AbstractBehavior (scaladsl, estilo orientado a objetos) | trait ActorBehavior<M> { fn on_message(&mut self, ctx, msg) -> Next<M>; fn on_signal(..)… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.sc… | Q-T-BEH-5, Q-T-CTX-2 |
| AT-BEH-011 | Interprete de Behavior: start, canonicalize, interpretMessage,… | Funcion interna interpret(behavior, ctx, input) -> Result<Behavior<M>, ActorError> en el… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:91-1… | Q-T-BEH-1 |
| AT-BEH-012 | Validacion de Behavior inicial (validateAsInitial, isAlive, isUnhandled,… | Funciones publicas en el modulo behavior: is_alive, is_unhandled; validate_as_initial… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/DeferredSpec.scala:147-158;… | - |
| AT-BEH-013 | BehaviorInterceptor + Behaviors.intercept… | trait BehaviorInterceptor<O, I> { fn around_start; fn around_receive(&mut self, ctx,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:107-166… | Q-T-BEH-3 |
| AT-BEH-014 | BehaviorSignalInterceptor | Implementacion por defecto de around_receive que delega. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:306-327… | Q-T-BEH-3 |
| AT-BEH-015 | Deduplicacion de interceptores (isSame) | is_same por identidad de tipo o comparacion explicita; el runtime recorre la pila antes… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:126-207… | Q-T-BEH-3 |
| AT-BEH-016 | transformMessages (Behavior.BehaviorDecorators) / narrow | Combinador behavior.transform_messages(|outer: O| -> Option<I>) que produce Behavior<O>;… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala… | Q-T-BEH-2, Q-T-BEH-3, Q-X-3 |
| AT-BEH-017 | Behaviors.monitor | behaviors::monitor(monitor_ref: ActorRef<M>, behavior) con M: Clone. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MonitorSpec.scala:28-71;… | Q-T-BEH-3 |
| AT-BEH-018 | Behaviors.logMessages + LogOptions | Interceptor que emite tracing::event! con nivel configurable. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LogMessagesSpec.scala:35-147 | Q-T-BEH-3, Q-T-BEH-6, Q-X-7 |
| AT-BEH-019 | Behaviors.withMdc (estatico, por mensaje, combinado) | Interceptor que abre un tracing::Span con campos (o usa log::kv) durante around_receive. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.s… | Q-T-BEH-3, Q-T-BEH-6, Q-X-7 |
| AT-BEH-020 | Behaviors.withTimers (fabrica de TimerScheduler) | behaviors::with_timers(|timers| ...). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:96-370 | Q-T-TMR-1 |
| AT-BEH-021 | Behaviors.withStash (fabrica de StashBuffer) | behaviors::with_stash(capacity, |stash| ...). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:22… | - |
| AT-BEH-022 | Behavior.narrow (contravarianza simulada) | Sin subtipado en Rust: narrow pasa a ser una conversion explicita via From/Into entre… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1497-… | Q-T-BEH-4 |
| AT-BEH-023 | Reutilizacion de valores Behavior | Behavior<M>: Clone (via Arc internamente) o fabricas Fn. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:331-339… | Q-T-BEH-1 |
| AT-BEH-024 | Behavior fallido (FailedBehavior) y su tratamiento en el adaptador | Behavior::Failed(ActorError) interno (no exportado); el runtime lo convierte en… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:387-4… | Q-X-5 |

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

Spec 28 `AT-SIG-1` (`28-at-sig-1.md`).
