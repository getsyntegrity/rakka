# Inventario de alcance: Apache Pekko 1.7.1, actores tipados (`pekko-actor-typed`)

Fuente de verdad: el checkout fijado en `upstream/pekko` (tag `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`). Todas las citas de la planilla son `path:linea` relativas a `upstream/pekko`. La planilla con una fila por feature está en [`actor-typed.tsv`](actor-typed.tsv).

## Qué contiene el módulo

`pekko-actor-typed` es la API de actores con tipo de mensaje explícito. En vez de un `Actor` con un `receive` que acepta cualquier cosa, el usuario define un `Behavior[T]` (comportamiento): un valor que dice cómo reaccionar al próximo mensaje de tipo `T` o a una señal del sistema (`Signal`, por ejemplo `PostStop` o `Terminated`) y que devuelve el `Behavior` siguiente. El código vive en `actor-typed/src/main/scala/org/apache/pekko/actor/typed/` y se organiza en: el núcleo (`Behavior`, `ActorRef`, `ActorSystem`, señales, supervisión), dos DSL (lenguajes de dominio específico, uno para Scala en `scaladsl/` y otro para Java en `javadsl/`), y servicios construidos encima: temporizadores, stash (buffer de mensajes diferidos), ask, Receptionist (descubrimiento de servicios), routers, event stream, pub-sub local (`Topic`), `SpawnProtocol`, extensiones y entrega confiable (`delivery`). El módulo `actor-testkit-typed` aporta el kit de pruebas (`ActorTestKit`, `TestProbe`, `BehaviorTestKit`, `LoggingTestKit`, `ManualTime`), que entra en el inventario porque sin él no se pueden validar las filas.

Un ejemplo concreto del contrato: si un actor devuelve `Behaviors.unhandled` para un `Terminated`, Pekko no lo ignora; `Behavior.interpretSignal` lanza `DeathPactException` (`actor-typed/.../Behavior.scala:270-277`) y, sin supervisión, el actor se detiene. Ese tipo de regla, que no se ve en la firma de la API, es lo que la planilla intenta capturar fila por fila.

## Cómo está construido sobre el módulo clásico

En Pekko el mundo tipado no tiene runtime propio: es una capa sobre los actores clásicos (`pekko-actor`). Tres adaptadores internos hacen el trabajo. `ActorAdapter` (`internal/adapter/ActorAdapter.scala`) es un `Actor` clásico que guarda el `Behavior` actual, traduce los `Terminated` clásicos a `Terminated`/`ChildFailed`, aplica los message adapters y decide la supervisión de sus hijos. `ActorContextAdapter` implementa el `ActorContext` tipado delegando en el `ActorContext` clásico (spawn, watch, receive timeout, `FunctionRef` para adaptadores). `ActorSystemAdapter` envuelve un `ActorSystemImpl` clásico, y el guardian de usuario es un `ActorAdapter` creado por `GuardianStartupBehavior`.

Por eso cada fila del TSV declara en `depends_on` qué unidades clásicas necesita, con códigos `AC-<UNIDAD>` (CELL = celda del actor y ciclo de vida, DW = death watch, SUP = supervisión, MBX = mailbox, DSP = dispatcher, SCH = scheduler, EVT = event stream, CS = CoordinatedShutdown, EXT = extensiones, SER = serialización, CFG = configuración, PATH = rutas de actor, REF = referencias, SYS = sistema, ASK = patrón ask). La pregunta de arquitectura más importante del porte (Q1, abajo) es si en Rust se replica esta arquitectura en capas o se escribe un runtime tipado nativo.

## Cómo se contó

Una fila es una feature observable, no un método: por ejemplo `watchWith` es una fila, pero las sobrecargas Scala/Java de `watchWith` no. Las filas de las unidades núcleo (BEH, SIG, CTX, REF, SYS, SUP) más TMR están marcadas `extracted`: se leyó la implementación y se ubicaron los tests que afirman el comportamiento (por título de test y, en los casos dudosos, por el cuerpo). El resto está `listed`: el contrato sale de la Scaladoc y de los títulos de los specs, sin lectura completa del cuerpo. `impl` y `validation` están todos en `pending`.

Total: **260 filas** (cierre documental del 2026-10-31; el inventario inicial tenía 170, 209 tras la revisión del 2026-10-30): 207 `public`, 24 `internal-required`, 24 `api-may-change`, 4 `internal-accidental`, 1 `deprecated`; 260 `extracted` y 0 `listed`. Varias filas se descompusieron en hijos `<ID>.<n>` (ver `docs/inventory/contracts/typed-*.md`).

Dos cosas que podrían parecer features y no lo son: no existe `Behavior.widen` en Pekko 1.7.1 (lo reemplaza `transformMessages`), y `OrElseSpec` prueba una composición escrita en el propio test, no una API de `Behaviors`. Ninguna de las dos tiene fila.

## Unidades, cantidad de filas y orden de dependencias

El orden de la tabla es el orden sugerido de implementación: cada unidad necesita las anteriores. La columna "Clásico primero" dice qué unidades del inventario clásico tienen que existir antes (o su equivalente nativo, si se elige Q1-b).

| Orden | Unidad | Qué cubre | Filas | Análisis | Depende de (tipado) | Clásico primero |
|---|---|---|---|---|---|---|
| 1 | BEH | Behavior y fábricas de `Behaviors` (same, unhandled, stopped, empty, ignore, setup, receive*, interceptores, transformMessages, monitor, logMessages, withMdc) | 26 | extracted | — | AC-CELL |
| 2 | SIG | Señales: PreRestart, PostStop, Terminated, ChildFailed, MessageAdaptionFailure, DeathPact | 8 | extracted | BEH | AC-CELL, AC-DW |
| 3 | REF | `ActorRef[T]`, `RecipientRef`, narrow, ignoreRef, `ActorRefResolver` | 7 | extracted | — | AC-REF, AC-PATH |
| 4 | CTX | `ActorContext`: spawn, stop, watch, receive timeout, adapters, ask, pipeToSelf | 18 | extracted | BEH, SIG, REF | AC-CELL, AC-DW, AC-SCH |
| 5 | SYS | `ActorSystem`, guardian, terminate, settings, scheduler | 15 | extracted | BEH, CTX | AC-SYS, AC-CFG, AC-SCH |
| 6 | SUP | Supervisión: stop/resume/restart, withLimit, backoff, stash de restart, anidado | 24 | extracted | BEH, SIG, CTX, TMR, STASH | AC-SUP |
| 7 | TMR | `TimerScheduler` | 9 | extracted | BEH, CTX | AC-SCH |
| 8 | STASH | `StashBuffer` | 6 | extracted | BEH, CTX | — |
| 9 | EVT | Event stream tipado, DeadLetter, UnhandledMessage, Dropped | 4 | extracted | REF, SYS | AC-EVT |
| 10 | LOG | `context.log`, MDC (Mapped Diagnostic Context), integración SLF4J | 5 | extracted | CTX | AC-EVT |
| 11 | DSP | `Props`, `DispatcherSelector`, `MailboxSelector`, `ActorTags` | 5 | extracted | — | AC-DSP, AC-MBX, AC-CFG |
| 12 | EXT | Extensiones tipadas | 7 | extracted | SYS | AC-EXT |
| 13 | ASK | `AskPattern` desde fuera de un actor, `StatusReply` | 4 | extracted | REF, SYS | AC-ASK |
| 14 | CS | CoordinatedShutdown (reusa el clásico) | 1 | extracted | SYS | AC-CS |
| 15 | RCP | Receptionist y `ServiceKey` | 10 | extracted | CTX, EXT | AC-DW |
| 16 | RT | Routers pool y group | 8 | extracted | CTX, RCP, EVT | — |
| 17 | PS | `Topic` (pub-sub local) | 3 | extracted | RCP, EVT | — |
| 18 | SPV | `SpawnProtocol` | 1 | extracted | CTX, ASK | — |
| 19 | ADP | Interop clásico/tipado (`toTyped`, `toClassic`, `PropsAdapter`, `ActorAdapter`) | 13 | extracted | SYS, SUP | AC-CELL, AC-SUP, AC-SYS |
| 20 | SER | Serializadores de `ActorRef` y `ServiceKey` | 3 | extracted | REF, RCP | AC-SER |
| 21 | DLV | Entrega confiable local (`ProducerController`, `ConsumerController`, work pulling) | 25 | extracted | CTX, TMR, STASH, RCP | AC-SER (solo chunking) |
| 22 | JAPI | Fachadas `javadsl` | 16 | extracted | todas | — |
| 23 | TK | `actor-testkit-typed` | 42 | extracted | SYS, CTX, EVT | AC-SCH (ManualTime) |
| | | **Total** | **170** | | | |

Aunque TK figura al final por dependencias de código, en la práctica `ActorTestKit` y `TestProbe` (AT-TK-001, AT-TK-002) se necesitan desde la primera historia de BEH para poder validar. Recomiendo tratarlos como infraestructura temprana.

## Límites del módulo

Quedan **fuera** de este inventario: `cluster-typed`, `cluster-sharding-typed`, `persistence-typed` y `stream-typed`. actor-typed los toca en cuatro puntos, y en cada uno la fila correspondiente anota el límite.

El Receptionist elige su implementación al arrancar: si la configuración del proveedor indica cluster, carga por reflexión `org.apache.pekko.cluster.typed.internal.receptionist.ClusterReceptionist`; si no, usa `LocalReceptionist` (`internal/receptionist/ReceptionistImpl.scala:27-48`). Solo el local entra en alcance, pero conviene dejar el punto de extensión. Lo mismo pasa con `Topic`, que se distribuye entre nodos "gratis" cuando el Receptionist es de cluster, y con la noción de routees inalcanzables del group router.

La entrega confiable define el protocolo `DurableProducerQueue`, pero la implementación durable (`EventSourcedProducerQueue`) vive en `persistence-typed` (`delivery/ProducerController.scala:66-71`); el `ShardingProducerController` vive en sharding. Entra solo la variante local con cola en memoria.

`PoisonPill` como señal interna (AT-SIG-008) existe para que sharding pueda pasivar entidades; se inventaría pero solo hace falta si más adelante se porta sharding. Los serializadores (unidad SER) solo importan con remoting, persistencia o el chunking de delivery.

## Dependencias externas esenciales

**SLF4J y MDC.** Todo el logging tipado usa `org.slf4j.Logger` y el MDC thread-local de SLF4J (`internal/ActorMdc.scala`, `WithMdcBehaviorInterceptor.scala`), y `reference.conf` activa `Slf4jLogger` por defecto. En Rust propongo `tracing`: un span por mensaje con los campos `pekkoSource`, `pekkoAddress`, `sourceActorSystem` y `pekkoTags` reemplaza al MDC, y funciona con async, donde un thread-local no sirve. La alternativa `log` + `kv` es más simple pero no propaga contexto entre `await`; la descarto salvo que el Owner prefiera la fachada mínima.

**HOCON (Human-Optimized Config Object Notation).** Las claves tipadas son pocas (`pekko.actor.typed.*`, `pekko.reliable-delivery.*`, `pekko.actor.testkit.typed.*`), pero dependen del mismo cargador de configuración que el módulo clásico. Esta decisión se toma en el inventario clásico (AC-CFG), no acá.

**Tipos función de Scala y `PartialFunction`.** La API usa funciones parciales (`receivePartial`, `receiveSignal`, `transformMessages`) y `ClassTag` para filtrar por clase en tiempo de ejecución. En Rust no hay funciones parciales ni herencia: la convención propuesta es un `match` que devuelve `Behavior::Unhandled` en el brazo `_`, y filtros por `TypeId` o enums de protocolo. Esto cambia contratos concretos (adaptadores que matchean subclases, suscripción a supertipos en el event stream, `ChildFailed` como subtipo de `Terminated`), y cada fila afectada lo anota.

**Runtime asíncrono y reloj.** Pekko usa `Future`/`ExecutionContext` y el scheduler clásico. El porte necesita un runtime (tokio es el candidato natural) y, para que `ManualTime` y los tests de backoff sean deterministas, un reloj inyectable desde el primer día.

## Preguntas abiertas para el Owner

Las filas con una decisión `Q-` en `notes` son 122; el registro único es `docs/decisions.md` (las decisiones de Typed llevan el prefijo `Q-T-`). Estas son las que más condicionan el resto, en orden de impacto.

1. **Arquitectura en capas o runtime nativo** (AT-SYS-011, AT-ADP-006). (a) Replicar Pekko: actores tipados sobre una celda clásica portada, con `ActorAdapter`. Mantiene la interoperabilidad y el comportamiento idéntico, pero obliga a portar primero buena parte del módulo clásico y arrastra su tipado dinámico. (b) Runtime tipado nativo, con interop clásico como capa opcional. Es más idiomático y simple en Rust, pero cada contrato que hoy "hereda" del clásico (death watch, orden de `PostStop`, receive timeout) hay que reimplementarlo y validarlo por separado. Recomiendo (b) si el objetivo principal es la API tipada, y (a) si la coexistencia con actores clásicos es requisito.
2. **Forma de `setup` y reutilización de `Behavior`** (AT-BEH-007, AT-BEH-023). Pekko permite reutilizar el mismo valor `Behavior` en varios spawns y re-ejecuta `setup` en cada restart. Con `Box<dyn FnOnce>` eso es imposible; hace falta `Arc<dyn Fn>` o `Behavior: Clone`. Recomiendo `Arc<dyn Fn>`. La alternativa de permitir `FnOnce` y prohibir el restart de esos Behaviors rompe el contrato de supervisión.
3. **Qué se supervisa: errores, panics o ambos** (AT-SUP-002, AT-SIG-007). Pekko captura solo `NonFatal` y filtra por clase de excepción. En Rust hay que decidir si `on_failure` actúa sobre `Err` devueltos por los handlers, sobre panics capturados con `catch_unwind`, o sobre ambos, y si el filtro es por tipo (`downcast_ref`) o por predicado.
4. **Contravarianza y `narrow`** (AT-BEH-022, AT-REF-003). En Scala estrechar el tipo es un cast gratis y el ref estrechado es igual al original. En Rust requiere envolver con una conversión `From`, lo que tiene costo y puede cambiar la identidad o igualdad del ref. Hay que decidir si un `ActorRef<Sub>` obtenido por `narrow` debe ser `==` al original.
5. **Discrepancia doc/código en backoff** (AT-SUP-007). La Scaladoc dice que durante el backoff los mensajes se descartan (`SupervisorStrategy.scala:64` y `:100`), pero el código los guarda en stash hasta `stashCapacity` y solo publica `Dropped` cuando se llena (`internal/Supervision.scala:278-286`), y hay un spec que lo afirma (`SupervisionSpec.scala:882`). Recomiendo seguir el código.

Otras preguntas registradas en el TSV, de menor alcance: si los tests de hilo (`ActorThreadSpec`) se reemplazan por tests de no-compilación (AT-CTX-016); si los adaptadores matchean solo el tipo exacto (AT-CTX-011); cómo se modela "suscribirse a subtipos" en el event stream (AT-EVT-001); si las extensiones se registran explícitamente en vez de cargarse por nombre de clase (AT-EXT-002); si `SpawnProtocol` acepta borrado de tipo (AT-SPV-001); si se excluyen las piezas JVM-only (`dynamicAccess`, `threadFactory`, JFR, integración SLF4J/Logback, serialización Java, `LoggerOps`, `startPeriodicTimer` deprecado, fachadas `javadsl`); y si `Topic` debe publicar `Dropped` al event stream en lugar de a dead letters (AT-EVT-004).

## Lo que no se hizo

No se leyó el cuerpo de cada test de las unidades `listed`; sus contratos salen de la Scaladoc y de los títulos de los specs. Los tests Java (`actor-typed-tests/src/test/java`) y los specs de documentación (`docs/`, `jdocs/`) se citan por nombre de archivo, no por aserción. La implementación interna de delivery (`delivery/internal/*Impl.scala`) se inventarió a nivel de protocolo público. Estos son los candidatos a pasar a `extracted` cuando se planifiquen esas unidades.
