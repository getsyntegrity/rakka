# Spec 50 — AT-JAPI (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 16. **Tareas:** 3 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-002, AT-BEH-003, AT-BEH-004, AT-BEH-005, AT-BEH-006, AT-BEH-007, AT-BEH-008.1, AT-BEH-008.3, AT-BEH-009, AT-BEH-010, AT-BEH-013, AT-BEH-015, AT-BEH-016, AT-BEH-017, AT-BEH-018, AT-BEH-019, AT-BEH-020, AT-BEH-021.
- Spec 36 `AT-SUP-1` (`36-at-sup-1.md`), por las filas AT-SUP-002, AT-SUP-013, AT-SUP-020, AT-SUP-021.
- Spec 28 `AT-SIG-1` (`28-at-sig-1.md`), por las filas AT-SIG-006, AT-SIG-007.
- Spec 39 `AT-LOG-1` (`39-at-log-1.md`), por las filas AT-LOG-001, AT-LOG-002.
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-001, AT-CTX-002, AT-CTX-003, AT-CTX-004, AT-CTX-005, AT-CTX-006, AT-CTX-007, AT-CTX-008, AT-CTX-009, AT-CTX-010, AT-CTX-011, AT-CTX-012, AT-CTX-013, AT-CTX-014, AT-CTX-015, AT-CTX-016, AT-CTX-017.
- Spec 42 `AT-ASK-1` (`42-at-ask-1.md`), por las filas AT-ASK-001, AT-ASK-002, AT-ASK-003, AT-ASK-004.
- Spec 34 `AT-TMR-1` (`34-at-tmr-1.md`), por las filas AT-TMR-001, AT-TMR-002, AT-TMR-003, AT-TMR-004, AT-TMR-005, AT-TMR-006, AT-TMR-007.1.
- Spec 35 `AT-STASH-1` (`35-at-stash-1.md`), por las filas AT-STASH-001, AT-STASH-002, AT-STASH-003, AT-STASH-004, AT-STASH-005, AT-STASH-006.
- Spec 44 `AT-RT-1` (`44-at-rt-1.md`), por las filas AT-RT-001, AT-RT-002.1, AT-RT-002.2, AT-RT-002.3, AT-RT-003, AT-RT-004, AT-RT-005, AT-RT-006.
- Spec 37 `AT-ADP-1` (`37-at-adp-1.md`), por las filas AT-ADP-001, AT-ADP-002, AT-ADP-003, AT-ADP-004, AT-ADP-005.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-BEH-6`, `Q-T-JAPI-1`, `Q-T-JAPI-2`, `Q-T-TMR-3`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: javadsl.Behaviors: fabricas delegadas sin logica propia (fachada de AT-BEH-*) (+5 filas)

Filas: AT-JAPI-001.1, AT-JAPI-001.2, AT-JAPI-001.3, AT-JAPI-002.1, AT-JAPI-002.2, AT-JAPI-003. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: javadsl.AbstractMatchingBehavior (OO Java sin builder, desde 1.3.0) (+5 filas)

Filas: AT-JAPI-004, AT-JAPI-005.1, AT-JAPI-005.2, AT-JAPI-005.3, AT-JAPI-005.4, AT-JAPI-006.1. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: javadsl.StashBuffer (vista Java de StashBufferImpl) (+3 filas)

Filas: AT-JAPI-006.2, AT-JAPI-006.3, AT-JAPI-006.4, AT-JAPI-006.5. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-JAPI-001.1 | javadsl.Behaviors: fabricas delegadas sin logica propia (fachada de AT-BEH-*) | Sin fachada separada (Q-T-JAPI-1): las fabricas son las de AT-BEH-*; se preserva toda la… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/InterceptTest.java:3… | Q-T-JAPI-1 |
| AT-JAPI-001.2 | javadsl.Behaviors.receive(onMessage, onSignal) y receiveSignal: funcion de… | Se preserva que el manejador de senales por defecto solo aplica cuando el usuario no da… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:54-82… | Q-T-JAPI-1, Q-T-JAPI-2 |
| AT-JAPI-001.3 | javadsl.Behaviors.withMdc: normalizacion de mapas null/vacios | Se preserva AT-BEH-019 (estatico + por mensaje, el del mensaje gana, limpieza tras cada… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.jav… | Q-T-BEH-6, Q-T-JAPI-1 |
| AT-JAPI-002.1 | javadsl.AbstractBehavior (estilo OO Java): createReceive diferido y chequeo de… | Una API unica (Q-T-JAPI-1): se preserva construccion diferida del receptor y fallo… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.j… | Q-T-JAPI-1, Q-T-JAPI-2 |
| AT-JAPI-002.2 | javadsl.Receive / ReceiveBuilder: builder mutable, primer caso coincidente,… | Se preserva orden de registro, unhandled por defecto, onAnyMessage que sombrea lo… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ReceiveBuilderTest.j… | Q-T-JAPI-1, Q-T-JAPI-2 |
| AT-JAPI-003 | javadsl.BehaviorBuilder: builder inmutable, sin senal por defecto para… | Se preserva primer-caso-coincidente, unhandled sin coincidencia, onAnyMessage que… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/BehaviorBuilderTest.… | Q-T-JAPI-1, Q-T-JAPI-2 |
| AT-JAPI-004 | javadsl.AbstractMatchingBehavior (OO Java sin builder, desde 1.3.0) | Una API unica (Q-T-JAPI-1); probablemente un trait Behavior con match sobre el… | actor-typed-tests/src/test/java/jdocs/org/apache/pekko/typed/OnMessageIntroTest.java:76-1… | Q-T-JAPI-1, Q-T-JAPI-2 |
| AT-JAPI-005.1 | javadsl.ActorContext: espejo directo (Optional, List copia, Duration) | Una API unica (Q-T-JAPI-1): Option, Vec snapshot y std::time::Duration; se preserva la… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorLoggingTest.jav… | Q-T-JAPI-1 |
| AT-JAPI-005.2 | javadsl.ActorContext.ask / askWithStatus: convencion (respuesta, excepcion) | ctx.ask(target, \\|reply_to\\| req, timeout, \\|result: Result<Res, AskError>\\| msg); se… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextAskTest.… | Q-T-JAPI-1 |
| AT-JAPI-005.3 | javadsl.ActorContext.pipeToSelf(CompletionStage) | ctx.pipe_to_self(future, \\|result\\| Option<T>): se preserva ejecucion en el actor,… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorContextPipeToSe… | Q-T-JAPI-1 |
| AT-JAPI-005.4 | javadsl.ActorContext.messageAdapter(Class, Function) | Adaptadores por TypeId; se preserva uno por tipo, reemplazo, mismo ref y orden inverso;… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:11… | Q-T-JAPI-1 |
| AT-JAPI-006.1 | javadsl.TimerScheduler: Duration Java y startPeriodicTimer | std::time::Duration en una API unica (Q-T-JAPI-1); se preserva AT-TMR-002..007.1; la… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/ActorCompile.java:13… | Q-T-JAPI-1, Q-T-TMR-3 |
| AT-JAPI-006.2 | javadsl.StashBuffer (vista Java de StashBufferImpl) | Un unico StashBuffer<T> (Q-T-JAPI-1); se preserva AT-STASH-*; dos traits y… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/StashBufferTest.java… | Q-T-JAPI-1 |
| AT-JAPI-006.3 | javadsl.Routers (fachada sobre los builders de router) | Una API unica (Q-T-JAPI-1); se preserva AT-RT-*; sobrecargas, Predicate/Function y el… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/RoutersTest.java:41-… | Q-T-JAPI-1 |
| AT-JAPI-006.4 | javadsl.AskPattern (ask/askWithStatus fuera de un actor) | ask(...) -> impl Future en una API unica (Q-T-JAPI-1); se preserva AT-ASK-*;… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/WatchTest.java:84-12… | Q-T-JAPI-1 |
| AT-JAPI-006.5 | javadsl.Adapter (equivalente Java de scaladsl.adapter) | Una API unica (Q-T-JAPI-1) sujeta a Q-X-8: si no hay interop clasica no hay equivalente;… | actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/javadsl/AdapterTest.java:260… | Q-T-JAPI-1, Q-X-8 |

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

Última spec del plan.
