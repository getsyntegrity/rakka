# Spec 11 — AC-EVT (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 24. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-008, AC-UTIL-009.
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-010. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 17 `AC-DW-1` (`17-ac-dw-1.md`), por las filas AC-DW-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 06 `AC-REF-1` (`06-ac-ref-1.md`), por las filas AC-REF-001.3, AC-REF-016.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-004, AC-CFG-005, AC-CFG-011.
- Spec 12 `AC-TK-1` (`12-ac-tk-1.md`), por las filas AC-TK-005.2. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-001, AC-CELL-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-015.
- Spec 20 `AC-FSM-1` (`20-ac-fsm-1.md`), por las filas AC-FSM-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-004.
- Spec 26 `AC-JAPI-1` (`26-ac-japi-1.md`), por las filas AC-JAPI-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-EVT-1`, `Q-EVT-2`, `Q-EVT-3`, `Q-EVT-4`, `Q-EVT-5`, `Q-EVT-6`, `Q-EVT-7`, `Q-X-2`, `Q-X-3`, `Q-X-4`, `Q-X-6`, `Q-X-7`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Contrato EventBus: subscribe/unsubscribe/publish con resultado booleano (+5 filas)

Filas: AC-EVT-001, AC-EVT-002, AC-EVT-003, AC-EVT-004, AC-EVT-005, AC-EVT-006. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: EventStreamUnsubscriber: quita suscriptores al terminar (+5 filas)

Filas: AC-EVT-007, AC-EVT-008, AC-EVT-009, AC-EVT-010, AC-EVT-011, AC-EVT-012. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: LoggingAdapter: formato de plantillas "{}" (hasta 4 args o Array), "WARNING arguments… (+5 filas)

Filas: AC-EVT-013.1, AC-EVT-013.2, AC-EVT-013.3, AC-EVT-014, AC-EVT-015, AC-EVT-016. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: DefaultLogger / StdOutLogger / StandardOutLogger: formato de salida, timestamps, MDC,… (+5 filas)

Filas: AC-EVT-017, AC-EVT-018, AC-EVT-019, AC-EVT-020, AC-EVT-021, AC-EVT-022. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-EVT-001 | Contrato EventBus: subscribe/unsubscribe/publish con resultado booleano | trait `EventBus { type Event; type Classifier; type Subscriber; }` | actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:61-159 | - |
| AC-EVT-002 | LookupClassification | `HashMap<Classifier, BTreeSet<Subscriber>>` | actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:331-360 | - |
| AC-EVT-003 | SubchannelClassification (jerarquía de clasificadores) | índice de subclasificación sobre `EventClass` (ver Q-X-6) | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:172-319 | Q-X-6 |
| AC-EVT-004 | ScanningClassification | `Vec`/`BTreeSet<(Classifier, Subscriber)>` + predicado | actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:299-329 | - |
| AC-EVT-005 | ManagedActorClassification (desuscripción automática al morir el suscriptor) y… | actor unsubscriber interno + mapa `ActorRef -> BTreeSet<ActorRef>` | actor-tests/src/test/scala/org/apache/pekko/event/EventBusSpec.scala:179-297 | Q-X-4 |
| AC-EVT-006 | EventStream: subscribe por clase, unsubscribe, publish; null →… | `EventStream` con suscripciones por `EventClass`/`TypeId` (Q-X-6) | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:108-135 | Q-EVT-1, Q-X-6 |
| AC-EVT-007 | EventStreamUnsubscriber: quita suscriptores al terminar | actor interno `EventStreamUnsubscriber` | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:320-445 | Q-X-4 |
| AC-EVT-008 | Tipos de dead letters: DeadLetter, SuppressedDeadLetter, Dropped,… | structs `DeadLetter`, `SuppressedDeadLetter`, `Dropped`; trait marker… | actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSupressionSpec.scala:41-106 | Q-EVT-1, Q-X-3 |
| AC-EVT-009 | DeadLetterListener: log de dead letters con límite (log-dead-letters),… | actor de sistema `DeadLetterListener` | actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSuspensionSpec.scala:70-103… | Q-EVT-1, Q-X-7 |
| AC-EVT-010 | UnhandledMessage (publicación y log con debug.unhandled) | `struct UnhandledMessage` + forwarder interno | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:137-153 | Q-EVT-1 |
| AC-EVT-011 | LoggingBus: niveles, setLogLevel, logger stdout de arranque,… | trait `Logger` + `LoggingBus`; Q-X-7 decide actor vs backend | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:155-170 ;… | Q-EVT-2, Q-X-2, Q-X-7 |
| AC-EVT-012 | Familia LogEvent (Error/Warning/Info/Debug y variantes con MDC/marker:… | `enum LogEvent { Error(..), Warning(..), Info(..), Debug(..) }` con `LogLevel` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:241-281,348-357 | - |
| AC-EVT-013.1 | LoggingAdapter: formato de plantillas "{}" (hasta 4 args o Array), "WARNING… | `LoggingAdapter::{error,warning,info,debug}(template, &[&dyn Display])` + macros | actor-tests/src/test/java/org/apache/pekko/event/LoggingAdapterTest.java:52-87 | Q-EVT-3 |
| AC-EVT-013.2 | BusLogging y NoLogging (publicación al bus y adaptador nulo) | `BusLogging` y `NoLogging` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:185-188 | - |
| AC-EVT-013.3 | LogSource y fábricas Logging.apply/getLogger/withMarker (obtención de… | trait `LogSource` + `Logging::new(system, &source)` | actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:75-95 ;… | Q-EVT-1, Q-EVT-6 |
| AC-EVT-014 | DiagnosticLoggingAdapter (MDC) y DiagnosticActorLogging | campos estructurados de `tracing` o `HashMap` en el adaptador | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:241-281 ;… | - |
| AC-EVT-015 | MarkerLoggingAdapter, LogMarker, ActorLogMarker | `struct LogMarker { name, properties }` y métodos con marker | actor-tests/src/test/scala/org/apache/pekko/event/MarkerLoggingSpec.scala:28-46 | - |
| AC-EVT-016 | LoggingFilter / DefaultLoggingFilter / LoggingFilterWithMarker | trait `LoggingFilter` + `DefaultLoggingFilter` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:210-216 ;… | Q-X-2 |
| AC-EVT-017 | DefaultLogger / StdOutLogger / StandardOutLogger: formato de salida,… | `StdOutFormatter` + `StandardOutLogger` | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:283-346 | Q-EVT-2, Q-X-7 |
| AC-EVT-018 | ActorLogging (log perezoso con logSource = path del actor) | `ctx.log()` | actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:36-43 (1.7.1)… | Q-EVT-1 |
| AC-EVT-019 | LoggingReceive (decora receive con log Debug; activable por debug.receive) | `logging_receive(ctx, receive)` / `with_label`; exige un Receive con `is_defined_at`… | actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:75-95 ;… | Q-EVT-1, Q-EVT-7, Q-X-3 |
| AC-EVT-020 | LoggerMailbox (vacía la cola de logs al cerrar) | mailbox del logger con drenaje final | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:190-206 | Q-X-7 |
| AC-EVT-021 | JavaLogger / JavaLoggingFilter (java.util.logging) | bridge a la fachada `log`/`tracing` (feature opcional) | actor-tests/src/test/scala/org/apache/pekko/event/jul/JavaLoggerSpec.scala:66-79 (1.7.1)… | Q-EVT-4 |
| AC-EVT-022 | API Java del EventBus (event/japi/EventBusJavaAPI.scala): clases abstractas… | mismas implementaciones; sin distinción Java/Scala (verificar paridad de puntos de… | docs/src/test/java/jdocs/event/EventBusDocTest.java:277-349 | Q-EVT-5 |

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

Spec 12 `AC-TK-1` (`12-ac-tk-1.md`).
