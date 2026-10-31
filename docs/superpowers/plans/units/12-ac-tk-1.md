# Spec 12 — AC-TK (1/1)

**Estado:** bloqueada. **Filas:** 27. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 17 `AC-DW-1` (`17-ac-dw-1.md`), por las filas AC-DW-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 09 `AC-DSP-1` (`09-ac-dsp-1.md`), por las filas AC-DSP-001, AC-DSP-002.
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-001.1, AC-SYS-004, AC-SYS-010, AC-SYS-013. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`), por las filas AC-SUP-003, AC-SUP-014. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-001.
- Spec 18 `AC-EXT-1` (`18-ac-ext-1.md`), por las filas AC-EXT-001.1, AC-EXT-001.2. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 11 `AC-EVT-1` (`11-ac-evt-1.md`), por las filas AC-EVT-008, AC-EVT-011, AC-EVT-012, AC-EVT-017.
- Spec 06 `AC-REF-1` (`06-ac-ref-1.md`), por las filas AC-REF-002, AC-REF-009.2.
- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-001, AC-CELL-004, AC-CELL-010. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-013.
- Spec 20 `AC-FSM-1` (`20-ac-fsm-1.md`), por las filas AC-FSM-001.1, AC-FSM-002, AC-FSM-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 10 `AC-SCH-1` (`10-ac-sch-1.md`), por las filas AC-SCH-010.
- Spec 26 `AC-JAPI-1` (`26-ac-japi-1.md`), por las filas AC-JAPI-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 23 `AC-SER-1` (`23-ac-ser-1.md`), por las filas AC-SER-002, AC-SER-003.5, AC-SER-004.1, AC-SER-004.2, AC-SER-013. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 24 `AC-IO-1` (`24-ac-io-1.md`), por las filas AC-IO-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: `Q-TK-1`.
- Propuestas aún sin decisión del Owner: `Q-EVT-5`, `Q-TK-10`, `Q-TK-2`, `Q-TK-3`, `Q-TK-4`, `Q-TK-5`, `Q-TK-6`, `Q-TK-7`, `Q-TK-8`, `Q-TK-9`, `Q-X-2`, `Q-X-3`, `Q-X-4`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: TestKitBase/TestKit: testActor (cola interna, actor de sistema sobre… (+5 filas)

Filas: AC-TK-001.1, AC-TK-001.2, AC-TK-001.3, AC-TK-001.4, AC-TK-001.5, AC-TK-002.1. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: AutoPilot (TestActor.AutoPilot, NoAutoPilot, KeepRunning) y ejecución antes de encolar (+5 filas)

Filas: AC-TK-002.2, AC-TK-002.3, AC-TK-003, AC-TK-004, AC-TK-005.1, AC-TK-005.2. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: TestEventListener: Mute/UnMute, conversión de DeadLetter/UnhandledMessage/Dropped en… (+5 filas)

Filas: AC-TK-005.3, AC-TK-006.1, AC-TK-006.2, AC-TK-007, AC-TK-008, AC-TK-009. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: TestKit.shutdownActorSystem / shutdown (verifySystemShutdown) (+5 filas)

Filas: AC-TK-010, AC-TK-011, AC-TK-012.1, AC-TK-012.2, AC-TK-013, AC-TK-014. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: JavaSerializable y TestJavaSerializer (serializador `java-test`, id 23, binding del… (+2 filas)

Filas: AC-TK-015, AC-TK-016, AC-TK-017. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-TK-001.1 | TestKitBase/TestKit: testActor (cola interna, actor de sistema sobre… | `TestKit::new(&system)` con canal propio; `test_actor() -> ActorRef` | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:32-57,193-198,200-211 | Q-TK-1, Q-TK-4 |
| AC-TK-001.2 | TestKit expect*: expectMsg, expectMsgPF, expectMsgType/Class,… | `expect_msg(&v)`, `expect_msg_type::<T>()`, `expect_msg_pf(f)`,… | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:32-57,107-124,151-160 | Q-TK-4 |
| AC-TK-001.3 | TestKit expectNoMessage / expectNoMsg | `expect_no_message()` y `expect_no_message_for(dur)` | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:223-233 | - |
| AC-TK-001.4 | TestKit receiveWhile, receiveN, receiveOne, fishForMessage,… | `receive_n(n)`, `receive_while(f)`, `fish_for_message(f)`, `fish_for_specific_message(f)` | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:162-191 | - |
| AC-TK-001.5 | TestKit within / remaining / remainingOr / remainingOrDefault / awaitCond /… | `within(min, max, \|\| ..)`, `remaining()`, `await_cond(..)`, `await_assert(..)` | testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:22-49 | Q-TK-4 |
| AC-TK-002.1 | TestProbe: ref, send, forward, sender, reply, nombres (testProbe-N / nombre de… | `TestProbe::new(&system)`, `TestProbe::named(&system, name)` | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:32-57,213-221 | Q-TK-1 |
| AC-TK-002.2 | AutoPilot (TestActor.AutoPilot, NoAutoPilot, KeepRunning) y ejecución antes de… | trait `AutoPilot { fn run(&mut self, sender, msg) -> AutoPilotNext }` | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:126-149 | - |
| AC-TK-002.3 | childActorOf (Spawn) y DelegatingSupervisorStrategy del testActor | `probe.child_actor_of(props, name, strategy)` | testkit/src/test/scala/org/apache/pekko/testkit/TestProbeSpec.scala:59-91 | Q-X-3, Q-X-5 |
| AC-TK-003 | ImplicitSender y DefaultTimeout | `TestKit::tell_from_probe`/sender explícito; `default_timeout()` | testkit/src/test/scala/org/apache/pekko/testkit/ImplicitSenderSpec.scala:28-32 ;… | Q-X-3 |
| AC-TK-004 | Dilatación de tiempo y settings (pekko.test.timefactor, single-expect-default,… | `TestKitSettings` leído de config + `Dilate` trait | testkit/src/test/scala/org/apache/pekko/testkit/TestTimeSpec.scala:22-34 | Q-TK-5 |
| AC-TK-005.1 | EventFilter: fábricas error/warning/info/debug/custom/apply[E],… | `EventFilter::error::<E>()`, `::warning()`, `::info()`, `::debug()`, `::custom(f)` | testkit/src/test/scala/org/apache/pekko/testkit/TestEventListenerSpec.scala:23-83 | Q-TK-3 |
| AC-TK-005.2 | EventFilter.intercept, filterEvents, filterException, awaitDone/assertDone y… | `filter.intercept(&system, \|\| ..)`, `filter_events(&system, [..], \|\| ..)` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:164-210 ;… | Q-TK-4 |
| AC-TK-005.3 | TestEventListener: Mute/UnMute, conversión de… | logger de test que captura `LogEvent` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:164-210 ;… | - |
| AC-TK-006.1 | TestActorRef (underlyingActor, receive síncrono) | `TestActorRef<A>` con candado reentrante compartido con la CTD: `with_actor(cierre)` y… | testkit/src/test/scala/org/apache/pekko/testkit/TestActorRefSpec.scala:130-154 ;… | Q-TK-1, Q-TK-2, Q-TK-8, Q-X-5 |
| AC-TK-006.2 | CallingThreadDispatcher (ejecución síncrona en el hilo del emisor;… | `CallingThreadDispatcher` con `ReentrantMutex` por actor, colas `HashMap<ThreadId,… | actor-tests/src/test/scala/org/apache/pekko/testkit/CallingThreadDispatcherModelSpec.scal… | Q-TK-1, Q-TK-7, Q-TK-8, Q-X-4, Q-X-5 |
| AC-TK-007 | TestFSMRef | `TestFsmRef<S, D, A: Fsm<S, D>>` sobre `TestActorRef<A>`: `state_name()`/`state_data()`… | testkit/src/test/scala/org/apache/pekko/testkit/TestFSMRefSpec.scala:24-50 ;… | Q-TK-1, Q-TK-2, Q-TK-8 |
| AC-TK-008 | ExplicitlyTriggeredScheduler (tiempo manual) | scheduler manual inyectable (`ManualScheduler`) | testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:22… | Q-TK-4 |
| AC-TK-009 | Utilidades de sincronización de test: TestLatch, TestBarrier y TestException | `testkit::util::{TestLatch, TestBarrier, TestException}` sobre Mutex/Condvar con timeout… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:212-232 ;… | Q-TK-4, Q-TK-5 |
| AC-TK-010 | TestKit.shutdownActorSystem / shutdown (verifySystemShutdown) | `shutdown_actor_system(&system, timeout, verify)` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:189-205 | - |
| AC-TK-011 | javadsl TestKit / EventFilter | N/A (misma API; sin distinción Java) | testkit/src/test/java/org/apache/pekko/testkit/javadsl/TestKitApiTest.java:27-182 ;… | Q-EVT-5, Q-TK-3 |
| AC-TK-012.1 | PekkoSpec: configuración base de specs (testConf), nombre del sistema por la… | `tests/common::TestSystem` (harness de tests de rakka) | testkit/src/test/scala/org/apache/pekko/testkit/PekkoSpecSpec.scala | Q-TK-6 |
| AC-TK-012.2 | Infra restante de specs: Coroner, TestTags, WithLogCapturing, AbstractSpec,… | helpers de `tests/common` (sólo los que un spec portado use) | testkit/src/test/scala/org/apache/pekko/testkit/CoronerSpec.scala:39-47 ;… | Q-TK-6, Q-TK-9 |
| AC-TK-013 | Funciones de paquete: filterEvents(Iterable\|varargs), filterException[T],… | funciones libres `filter_events`, `filter_exception`, trait `Dilate` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:77-105 | Q-TK-4 |
| AC-TK-014 | TestKitUtils.testNameFromCallStack / scrubActorSystemName | función de harness que usa `std::any::type_name`/nombre de test | derivada del código (ver bloque de contrato) | Q-TK-6 |
| AC-TK-015 | JavaSerializable y TestJavaSerializer (serializador `java-test`, id 23,… | serializador de pruebas id 23 / nombre java-test registrado por nombre (Q-X-2) con… | derivada del código (ver bloque de contrato) | Q-TK-10, Q-X-2 |
| AC-TK-016 | TestActors: EchoActor, BlackholeActor y ForwardActor (echoActorProps,… | `testkit::actors::{echo_props, blackhole_props, forward_props}` | testkit/src/test/scala/org/apache/pekko/testkit/TestActorsSpec.scala:16-41 | Q-X-3 |
| AC-TK-017 | SocketUtil: direcciones y puertos libres para tests (TCP/UDP, loopback… | `testkit::net::*` con std::net::{TcpListener, UdpSocket} | derivada del código (ver bloque de contrato) | Q-X-5 |

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

Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`).
