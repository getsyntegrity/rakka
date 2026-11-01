# Spec 32 — AT-CTX (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 18. **Tareas:** 3 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 29 `AT-REF-1` (`29-at-ref-1.md`), por las filas AT-REF-001.
- Spec 33 `AT-SYS-1` (`33-at-sys-1.md`), por las filas AT-SYS-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 40 `AT-DSP-1` (`40-at-dsp-1.md`), por las filas AT-DSP-001, AT-DSP-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 28 `AT-SIG-1` (`28-at-sig-1.md`), por las filas AT-SIG-004, AT-SIG-007.
- Spec 42 `AT-ASK-1` (`42-at-ask-1.md`), por las filas AT-ASK-001, AT-ASK-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-011.
- Spec 38 `AT-EVT-1` (`38-at-evt-1.md`), por las filas AT-EVT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 36 `AT-SUP-1` (`36-at-sup-1.md`), por las filas AT-SUP-016, AT-SUP-021. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 37 `AT-ADP-1` (`37-at-adp-1.md`), por las filas AT-ADP-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 50 `AT-JAPI-1` (`50-at-japi-1.md`), por las filas AT-JAPI-005.1, AT-JAPI-005.2, AT-JAPI-005.3, AT-JAPI-005.4. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 35 `AT-STASH-1` (`35-at-stash-1.md`), por las filas AT-STASH-005. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-BEH-5`, `Q-T-CTX-1`, `Q-T-CTX-2`, `Q-T-CTX-3`, `Q-T-CTX-4`, `Q-T-TMR-2`, `Q-X-3`, `Q-X-4`, `Q-X-6`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: self y system (accesibles desde otros hilos) (+5 filas)

Filas: AT-CTX-001, AT-CTX-002, AT-CTX-003, AT-CTX-004, AT-CTX-005, AT-CTX-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: unwatch(other) (+5 filas)

Filas: AT-CTX-007, AT-CTX-008, AT-CTX-009, AT-CTX-010, AT-CTX-011, AT-CTX-012. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: askWithStatus (StatusReply) (+5 filas)

Filas: AT-CTX-013, AT-CTX-014, AT-CTX-015, AT-CTX-016, AT-CTX-017, AT-CTX-018. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-CTX-001 | self y system (accesibles desde otros hilos) | ctx.self_ref() -> ActorRef<M> (Clone + Send); ctx.system() -> ActorSystemHandle. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:535-… | - |
| AT-CTX-002 | spawn / spawnAnonymous con Props | ctx.spawn(behavior, name, props) -> Result<ActorRef<U>, SpawnError>; spawn_anonymous… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:179-… | Q-T-CTX-3 |
| AT-CTX-003 | children / child(name) | ctx.children() -> impl Iterator<Item=ActorRef<Untyped>>; ctx.child(name) -> Option<..>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:550-… | Q-X-3 |
| AT-CTX-004 | stop(child) | ctx.stop(&child) -> Result<(), StopError::NotAChild>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:227-… | Q-T-CTX-3 |
| AT-CTX-005 | watch(other) | ctx.watch(&ref) -> Result<(), WatchError>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:375-… | Q-T-CTX-3 |
| AT-CTX-006 | watchWith(other, msg) | ctx.watch_with(&ref, msg: M) con M: Clone? (se guarda hasta la terminacion). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:234-308,… | Q-T-CTX-3 |
| AT-CTX-007 | unwatch(other) | ctx.unwatch(&ref). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:443-… | - |
| AT-CTX-008 | setReceiveTimeout / cancelReceiveTimeout | ctx.set_receive_timeout(Duration, msg: M) con M: Clone. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:578-… | Q-T-TMR-2 |
| AT-CTX-009 | scheduleOnce(delay, target, msg) | ctx.schedule_once(Duration, &target, msg) -> Cancellable. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:597-… | - |
| AT-CTX-010 | executionContext | ctx.executor() -> handle de runtime (p.ej. tokio::runtime::Handle) o spawner abstracto. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.sc… | Q-X-4 |
| AT-CTX-011 | messageAdapter[U](f) | ctx.message_adapter::<U>(\|u: U\| -> M) -> ActorRef<U>; clave por TypeId de U; el 'matchea… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec… | Q-T-CTX-1, Q-X-3 |
| AT-CTX-012 | ask (desde el actor) con mapResponse | ctx.ask(&target, \|reply_to\| Req, timeout, \|res: Result<Res, AskError>\| -> M). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpe… | Q-X-4, Q-X-9 |
| AT-CTX-013 | askWithStatus (StatusReply) | StatusReply<T> como Result<T, StatusError>; ctx.ask_with_status. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextAskSpe… | - |
| AT-CTX-014 | pipeToSelf(future)(mapResult) | ctx.pipe_to_self(future, \|Result<V, E>\| -> M): spawnea la future en el runtime y envia… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextPipeTo… | Q-X-4 |
| AT-CTX-015 | delegate(delegator, msg) | ctx.delegate(behavior, msg) -> Behavior<M>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelega… | Q-X-6 |
| AT-CTX-016 | Chequeo de hilo del ActorContext (no thread-safe) | En Rust el borrow checker lo resuelve en compilacion: ActorContext<'a> solo existe como… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.sc… | Q-T-BEH-5, Q-T-CTX-2 |
| AT-CTX-017 | TypedActorContext / asScala / asJava y ClassicActorContextProvider | Un unico ActorContext<M> (asScala/asJava desaparecen: eran la identidad). El puente… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-T-CTX-2, Q-T-CTX-4, Q-X-8 |
| AT-CTX-018 | spawnMessageAdapter (interno) y hooks internos… | Metodos pub(crate) del contexto. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:63… | Q-T-CTX-1 |

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

Spec 33 `AT-SYS-1` (`33-at-sys-1.md`).
