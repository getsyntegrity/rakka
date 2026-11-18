# Spec 19 — AC-ASK (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-001.
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-017.
- Spec 06 `AC-REF-1` (`06-ac-ref-1.md`), por las filas AC-REF-002, AC-REF-011.
- Spec 10 `AC-SCH-1` (`10-ac-sch-1.md`), por las filas AC-SCH-001.
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-001.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-001.
- Spec 18 `AC-EXT-1` (`18-ac-ext-1.md`), por las filas AC-EXT-001.1, AC-EXT-001.2.
- Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`), por las filas AC-SUP-001.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-ASK-1`, `Q-ASK-10`, `Q-ASK-2`, `Q-ASK-3`, `Q-ASK-4`, `Q-ASK-5`, `Q-ASK-6`, `Q-ASK-7`, `Q-ASK-8`, `Q-ASK-9`, `Q-CS-7`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ask / `?` sobre ActorRef: ref temporal PromiseActorRef como sender, Future completado… (+5 filas)

Filas: AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3, AC-ASK-002, AC-ASK-003, AC-ASK-004. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: StatusReply (Success/Error/Ack) y askWithStatus (+5 filas)

Filas: AC-ASK-005, AC-ASK-006, AC-ASK-007, AC-ASK-008, AC-ASK-009.1, AC-ASK-009.2. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: retry: predicado shouldRetry (por valor y por excepcion) (+5 filas)

Filas: AC-ASK-009.3, AC-ASK-010, AC-ASK-011.1, AC-ASK-011.2, AC-ASK-011.3, AC-ASK-011.4. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: CircuitBreaker: API Java (callWithCircuitBreakerCS, defineFailureFn con Optional) y… (+5 filas)

Filas: AC-ASK-011.5, AC-ASK-012, AC-ASK-013, AC-ASK-014, AC-ASK-015.1, AC-ASK-015.2. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: BackoffOpts.onFailure: reinicio del hijo tras fallo con backoff; traduccion de Restart a… (+5 filas)

Filas: AC-ASK-015.3, AC-ASK-015.4, AC-ASK-015.5, AC-ASK-015.6, AC-ASK-016, AC-ASK-017. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-ASK-001.1 | ask / `?` sobre ActorRef: ref temporal PromiseActorRef como sender, Future… | `async fn ask<M, R>(target: &ActorRef, msg: M, timeout: Duration) -> Result<R,… | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:34-39 | Q-ASK-1, Q-X-3, Q-X-4, Q-X-9 |
| AC-ASK-001.2 | ask: entradas invalidas (timeout <= 0, destinatario no soportado, destinatario… | enum `AskError { Timeout{..}, InvalidTimeout{..}, UnsupportedRecipient{..},… | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:41-92 | Q-ASK-1, Q-X-3 |
| AC-ASK-001.3 | ask: AskTimeoutException al vencer el timeout (target, timeout, sender, clase… | AskError::Timeout { target: String, timeout: Duration, message_type: &str, sender:… | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:94-126 | Q-ASK-1, Q-X-3, Q-X-4 |
| AC-ASK-002 | PromiseActorRef: ciclo de vida, vigilable, path /temp que codifica el nombre… | TemporaryReplyRef { state: AtomicU8/enum {Fresh, Registering, Registered(path), Stopped,… | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:227-289 | Q-ASK-2 |
| AC-ASK-003 | ask sobre ActorSelection y respuestas dirigidas via selection | ask_selection(selection, msg, timeout) reutilizando el mismo ReplyRef que ask; prefijo =… | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:128-225 | Q-ASK-1, Q-X-3 |
| AC-ASK-004 | Ask explicito (fabrica de mensaje que recibe replyTo) | `ask_with(target, \|reply_to: ActorRef\| Msg { reply_to }, timeout)` | actor-tests/src/test/scala/org/apache/pekko/pattern/extended/ExplicitAskSpec.scala:33-59 | Q-ASK-1 |
| AC-ASK-005 | StatusReply (Success/Error/Ack) y askWithStatus | `enum StatusReply<T> { Success(T), Error(Box<dyn Error + Send + Sync>) }` +… | actor-tests/src/test/scala/org/apache/pekko/pattern/StatusReplySpec.scala:31,39,46,53,60,… | Q-ASK-1, Q-X-9 |
| AC-ASK-006 | pipeTo / pipeToSelection (Future y CompletionStage) y Status.Failure | `fn pipe_to<T>(fut: impl Future<Output=Result<T,E>>, target: ActorRef, sender:… | actor-tests/src/test/scala/org/apache/pekko/pattern/PipeToSpec.scala:29,35,41,49,55,66,73… | Q-X-4, Q-X-5 |
| AC-ASK-007 | gracefulStop | `async fn graceful_stop(target: &ActorRef, timeout: Duration, stop: impl Into<Msg>) ->… | actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:39-57 | Q-ASK-1 |
| AC-ASK-008 | after / timeout (FutureTimeoutSupport) | `async fn after<T>(d, fut)`/`timeout(d, fut)` sobre tokio::time | actor-tests/src/test/scala/org/apache/pekko/pattern/PatternSpec.scala:60-94 | Q-ASK-1, Q-CS-7 |
| AC-ASK-009.1 | retry: conteo de intentos, excepciones lanzadas como fallos, agotamiento | `async fn retry<T,E>(attempt: impl FnMut() -> Fut, attempts: u32, policy) ->… | actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:30-95,142-187 | Q-ASK-10, Q-X-5 |
| AC-ASK-009.2 | retry: demora fija, funcion de demora y backoff exponencial (calculateDelay) | enum `RetryDelay { None, Fixed(Duration), Func(Box<dyn Fn(u32)->Option<Duration>>),… | actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:97-140 ;… | Q-ASK-10, Q-X-4 |
| AC-ASK-009.3 | retry: predicado shouldRetry (por valor y por excepcion) | parametro `should_retry: Fn(&Result<T,E>) -> bool` con defecto `result.is_err()` | actor-tests/src/test/scala/org/apache/pekko/pattern/RetrySpec.scala:189-218 | Q-X-5 |
| AC-ASK-010 | PromiseRef / FutureRef | `PromiseRef<T> { reference: ActorRef, future: impl Future }` y `FutureRef<T>` | actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala:37-82 | Q-ASK-1, Q-ASK-2 |
| AC-ASK-011.1 | CircuitBreaker: maquina de estados Closed/Open/HalfOpen, conteo de fallos,… | `struct CircuitBreaker` con `enum State { Closed(AtomicU32), HalfOpen(AtomicBool),… | actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:137-194,197-… | Q-ASK-3, Q-X-4 |
| AC-ASK-011.2 | CircuitBreaker: callTimeout (TimeoutException y fallo contado) | tokio::time::timeout alrededor del futuro de la llamada; error… | actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:286-298,405-… | Q-ASK-4 |
| AC-ASK-011.3 | CircuitBreaker: constructores, validaciones, backoff exponencial de… | `CircuitBreakerConfig { max_failures, call_timeout, reset_timeout, max_reset_timeout,… | actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:81-83,481-501 | Q-ASK-10, Q-ASK-3 |
| AC-ASK-011.4 | CircuitBreaker: callbacks… | trait/closures `on_open(Fn())`... y `with_circuit_breaker(fut)`; la variante sync =… | actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:38-76,171-19… | Q-ASK-3, Q-ASK-5 |
| AC-ASK-011.5 | CircuitBreaker: API Java (callWithCircuitBreakerCS, defineFailureFn con… | La API Java no se porta (Q-ASK-9); la regresion #17415 se porta como test de… | actor-tests/src/test/java/org/apache/pekko/pattern/CircuitBreakerTest.java:40-74 ;… | Q-ASK-3, Q-ASK-4, Q-ASK-9 |
| AC-ASK-012 | CircuitBreaker: defineFailureFn y exception-allowlist | closure `is_failure: Fn(&Result<T,E>) -> bool` + `allowlist: HashSet<&str>` por… | actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:84-92,206-21… | Q-ASK-4, Q-X-5 |
| AC-ASK-013 | CircuitBreakersRegistry (breakers por id desde config) | `CircuitBreakerRegistry` como extension: `HashMap<String, Arc<CircuitBreaker>>` bajo Mutex | actor-tests/src/test/scala/org/apache/pekko/pattern/CircuitBreakerSpec.scala:95-106,762-8… | Q-ASK-3, Q-X-1, Q-X-2 |
| AC-ASK-014 | CircuitBreakerTelemetry (SPI) | trait `CircuitBreakerTelemetry` + registro de fabricas por nombre (Q-X-2) | - (sin specs upstream) | Q-ASK-6, Q-X-2 |
| AC-ASK-015.1 | BackoffSupervisor.props / BackoffOpts: parametros, validaciones y mensajes… | Builder consumidor BackoffOpts::on_stop/on_failure ->… | actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:60-66,69-… | Q-ASK-10, Q-ASK-7, Q-X-3, Q-X-5 |
| AC-ASK-015.2 | BackoffOpts.onStop: reinicio del hijo tras su detencion con backoff exponencial | actor `BackoffOnStopSupervisor` con timer interno y `child: Option<ActorRef>` | actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:92-139,29… | Q-ASK-7, Q-X-5 |
| AC-ASK-015.3 | BackoffOpts.onFailure: reinicio del hijo tras fallo con backoff; traduccion de… | actor `BackoffOnFailureSupervisor` con supervisor_strategy propia | actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffOnRestartSupervisorSpec.scala:… | Q-ASK-7, Q-X-5 |
| AC-ASK-015.4 | BackoffSupervisor: manejo de mensajes mientras no hay hijo (replyWhileStopped,… | enum `WhileStopped { DeadLetters, ReplyWith(Msg), ForwardTo(ActorRef) }`; `enum… | actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:141-186,1… | Q-ASK-7 |
| AC-ASK-015.5 | BackoffOpts.onStop: finalStopMessage | campo `final_stop_message: Option<Box<dyn Fn(&Msg)->bool>>` | actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:394-431 | Q-ASK-7 |
| AC-ASK-015.6 | BackoffSupervisor: composicion de la estrategia de supervision del hijo… | Estrategia del hijo como decide(&Failure) -> Directive compuesto en el constructor… | actor-tests/src/test/scala/org/apache/pekko/pattern/BackoffSupervisorSpec.scala:92-117,11… | Q-ASK-7, Q-X-5 |
| AC-ASK-016 | Backoff (API vieja) / BackoffOptions deprecada | Q-ASK-8 propone no portar la API deprecada (equivalente: BackoffOpts) | sin tests de la API deprecada en Pekko 1.7.1 (grep en actor-tests, actor-typed*,… | Q-ASK-10, Q-ASK-8 |
| AC-ASK-017 | Patterns / PatternsCS (fachadas Java) | Q-ASK-9 propone no portar las fachadas Java (el API Rust unico las cubre) | actor-tests/src/test/java/org/apache/pekko/pattern/PatternsTest.java:81-112,114-126,128-2… | Q-ASK-1, Q-ASK-10, Q-ASK-9, Q-X-5 |

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

Spec 20 `AC-FSM-1` (`20-ac-fsm-1.md`).
