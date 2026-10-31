# Inventario de alcance: `pekko-actor` (actores clásicos) — resumen para el Owner

Este documento resume `docs/inventory/actor-classic.tsv`, que es la definición de "100%" del port de los actores clásicos de Apache Pekko 1.7.1 (commit `9a54d129…`, submódulo en `upstream/pekko`). El TSV (archivo de valores separados por tabulador) tiene una fila por funcionalidad observable; este resumen explica qué contiene, cómo se armó, qué queda afuera y qué decisiones necesitan al Owner.

## Qué contiene el módulo

`pekko-actor` es mucho más que "actores que se mandan mensajes". Además del núcleo (direcciones y paths, referencias, la celda que ejecuta cada actor, supervisión, DeathWatch, mailboxes y dispatchers) trae su propio sistema de configuración por claves, un bus de eventos con logging, un scheduler de timers, patrones (ask, pipe, circuit breaker, backoff), una máquina de estados finitos (FSM), routers, el apagado coordinado (CoordinatedShutdown), serialización, red no bloqueante (`pekko.io`: TCP, UDP y DNS) y una API Java paralela a la de Scala. El testkit clásico (`testkit/`) no es parte del módulo pero se incluye porque sin él no se pueden portar las specs que fijan el comportamiento.

En total el inventario tiene **507 filas** (tras la revisión de contratos del 2026-10-30; el inventario inicial tenía 322): 351 `public`, 106 `internal-required`, 38 `internal-accidental`, 12 `deprecated`. Varias filas del inventario inicial se descompusieron en hijos `<ID>.<n>` (el padre desaparece y una referencia a un padre significa todos sus hijos) y se agregaron las omisiones detectadas; la lista está en `docs/inventory/contracts/*.md`, secciones "Descomposiciones" y "Omisiones agregadas".

## Cómo se contó

Se recorrió `actor/src/main/` completo (los directorios `scala`, `scala-2.13`, `scala-3`, `java`, `boilerplate`, `resources` y `mima-filters`), cada sección de `reference.conf`, el testkit clásico y los títulos de todos los specs de `actor-tests` y `testkit/src/test`. La granularidad es de funcionalidad, no de método: "parar un actor para primero a sus hijos y después llama a postStop" es una fila, no cinco.

Cada fila cita la implementación con `path:línea` y, cuando existe, el spec que lo afirma. Todos los paths son relativos a `upstream/pekko` y un script verificó que cada archivo citado exista y que cada número de línea esté dentro del archivo (no verifica que la línea sea "la correcta"; los rangos de las unidades núcleo se leyeron a mano). El inventario previo de goPekko (`INV-A-actor.md`) se usó para no olvidar nada y para ahorrar búsquedas, pero ninguna cita sale de ahí sin haberse comprobado contra el código de Pekko.

La columna `analysis` distingue dos niveles. `extracted` significa que se leyó el código y el spec y el contrato de la fila sale de esa lectura. `listed` significa que la funcionalidad está enumerada con su evidencia, pero el contrato es un resumen y hay que extraerlo antes de implementar. Las unidades núcleo (PATH, CFG, REF, SYS, CELL, SUP, DW, MBX, DSP) están casi enteras en `extracted`; las pocas filas `listed` dentro de ellas son variantes secundarias (por ejemplo `AffinityPool` o los helpers de `Future` de Scala).

En la columna `contract`, el prefijo indica de dónde sale la afirmación: `[contractual]` y `[contractual Scaladoc]`/`[contractual docs]` vienen de la documentación; `[contractual por spec]` lo afirma un test; `[contractual interno]` es un invariante interno del que depende algo público; `[accidental]` es comportamiento observable que nadie promete y que no hace falta copiar; `[contractual JVM]` es contrato sólo en la JVM (Java Virtual Machine) y casi siempre lleva una pregunta abierta.

## Unidades, filas y orden de dependencias

El orden de la tabla es el orden sugerido de implementación. La idea es ir de las hojas del grafo (valores sin concurrencia) hacia lo que necesita un sistema de actores funcionando.

| # | Unidad | Qué cubre | Filas | extracted / listed | Depende de |
|---|---|---|---|---|---|
| 1 | UTIL | Timeout, helpers, Done, excepciones base, ByteString, colas e índices de apoyo | 64 | 28 / 36 | — |
| 2 | CFG | Carga de config HOCON, `Settings`, cada sección de `reference.conf`, Deploy, reflexión por nombre | 29 | 29 / 0 | UTIL |
| 3 | PATH | `Address`, `ActorPath`, parseo y validación de nombres, paths especiales | 16 | 16 / 0 | UTIL |
| 4 | REF | `ActorRef`, `tell`/`forward`/`sender`, `Props`, `actorOf`, selección, deadLetters, refs internos | 32 | 26 / 6 | PATH, CFG |
| 5 | MBX | Mailboxes (estados, colas, acotadas, prioridad, control), selección de mailbox, Stash | 23 | 13 / 10 | REF, CFG |
| 6 | DSP | Dispatchers, executors, throughput, registro por id | 22 | 12 / 10 | MBX, CFG |
| 7 | SCH | Scheduler, Cancellable, Timers por actor | 15 | 9 / 6 | DSP |
| 8 | EVT | EventStream, buses, dead letters, logging | 24 | 19 / 5 | REF, DSP |
| 9 | CELL | Ciclo de vida del actor: creación, receive, become, hooks, restart, stop, ReceiveTimeout | 38 | 36 / 2 | MBX, DSP, SCH, EVT |
| 10 | SUP | Supervisión: directivas, estrategias, ventanas de reinicio, guardianes | 21 | 21 / 0 | CELL |
| 11 | DW | DeathWatch: watch, unwatch, watchWith, Terminated, death pact | 13 | 12 / 1 | CELL |
| 12 | SYS | `ActorSystem`: creación, guardianes, terminate, callbacks, provider local | 27 | 16 / 11 | todo lo anterior |
| 13 | EXT | Extensiones por sistema | 5 | 4 / 1 | SYS |
| 14 | TK | TestKit, TestProbe, EventFilter, scheduler manual | 25 | 18 / 7 | SYS, EVT |
| 15 | ASK | ask, pipe, gracefulStop, retry, CircuitBreaker, BackoffSupervisor | 28 | 25 / 3 | SYS, SCH |
| 16 | FSM | Máquina de estados finitos (FSM) | 11 | 11 / 0 | CELL, SCH |
| 17 | RT | Routers (pools, groups, resizers) | 22 | 21 / 1 | CELL, SUP |
| 18 | CS | CoordinatedShutdown | 15 | 15 / 0 | SYS, EXT |
| 19 | SER | Serialización (sólo la parte local) | 23 | 21 / 2 | EXT |
| 20 | IO | `pekko.io`: TCP, UDP, DNS | 35 | 34 / 1 | SYS |
| 21 | JAPI | API Java (`AbstractActor`, builders, interfaces funcionales, TypedActor) | 19 | 7 / 12 | — |
| | **Total** | | **507** | **393 / 114** | |

Hay un ciclo real que el orden tiene que romper: la celda (CELL) necesita el EventStream para publicar errores y el scheduler para ReceiveTimeout, y el `ActorSystem` (SYS) es quien crea ambos. Por eso SCH y EVT aparecen antes que CELL, pero sólo en su forma mínima (programar una tarea, publicar un evento). Lo mismo pasa con el testkit: las specs de CELL, SUP y DW usan `TestProbe` y `EventFilter`, que a su vez necesitan un sistema andando. La propuesta es construir un TK mínimo (probe con `expectMsg`, `expectTerminated` y captura de logs) junto con SYS, y validar CELL/SUP/DW después con las specs portadas. PATH, CFG y la parte de valores de REF se validan con tests comunes y con el oráculo (`tools/oracle`, los jars oficiales), sin sistema de actores.

## Límites del módulo: qué queda afuera y dónde lo toca `pekko-actor`

Quedan fuera de este inventario remote, cluster (y sharding, singleton, distributed data), streams, persistence y HTTP. Ninguno vive en `pekko-actor`, pero el módulo tiene ganchos hacia ellos que conviene conocer para no cerrar puertas sin querer.

El gancho principal es el proveedor de referencias. `pekko.actor.provider` acepta `local`, `remote` o `cluster`, y las dos últimas cargan por nombre `org.apache.pekko.remote.RemoteActorRefProvider` y `org.apache.pekko.cluster.ClusterActorRefProvider` (`actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:112-118`). Todo el resto del sistema habla con el trait `ActorRefProvider` (fila AC-SYS-018), así que si en Rust ese trait queda sellado y con una sola implementación, agregar remote más adelante implica abrirlo.

El segundo gancho es DeathWatch sobre refs no locales. Cada ref sabe si es local (`ActorRefScope.isLocal`); cuando un actor vigila a uno que no lo es, se suscribe a `AddressTerminatedTopic` para enterarse si se cae el nodo, y al morir notifica primero a los vigilantes remotos (AC-DW-010, AC-DW-012). En local nunca se ejercita, pero el campo `addressTerminated` de `Terminated` es público.

El tercero es la serialización de referencias: `Serialization.serializedActorPath`, la "información de transporte" y la serialización Java de `ActorRef` existen para que remote pueda mandar refs por la red (AC-SER-007, AC-REF-023). También `pekko.serialization.protobuf.allowed-classes` y `max-decompressed-size` están en `reference.conf` de `pekko-actor` pero los consumen remote y cluster.

Por último, `reference.conf` declara las fases de CoordinatedShutdown de cluster (`cluster-leave`, `cluster-exiting`, etc.) y razones de apagado como `ClusterDowningReason`. En local esas fases quedan vacías pero existen, y el orden topológico las incluye (AC-CS-002).

## Dependencias externas y su contraparte propuesta en Rust

Pekko se apoya en piezas de la JVM y del ecosistema Scala que en Rust no existen tal cual. Cada una es una decisión de diseño que atraviesa varias unidades.

**Typesafe Config / HOCON** (Human-Optimized Config Object Notation, versión 1.4.9). Todo `Settings` y todo `reference.conf` dependen de él: sustituciones `${?x}`, includes, duraciones como `"5 s"`, tamaños como `"128 KiB"` y literales especiales (`off`, `infinite`, `unlimited`). La contraparte puede ser un crate de HOCON existente o un parser propio del subconjunto que usa Pekko; la condición es que `reference.conf` se pueda embeber sin cambios y que `ConfigSpec` (que verifica cada default) pase igual.

**Reflexión de la JVM** (`DynamicAccess`). Mailboxes, executors, loggers, serializadores, extensiones, el scheduler y el configurador de la estrategia de `/user` se instancian por FQCN (nombre de clase completo) leído de la config. Rust no tiene reflexión, así que la propuesta es un registro explícito nombre → fábrica, con los nombres de `reference.conf` registrados de fábrica. Se rechazó usar macros de registro global (`inventory`/`linkme`) como mecanismo principal porque dependen del linker y complican los tests; pueden sumarse después como comodidad.

**Concurrencia del JDK** (Java Development Kit): `ForkJoinPool` para el default-dispatcher, `ThreadPoolExecutor`, `ConcurrentLinkedQueue` y la cola MPSC (multi-productor, un consumidor) de Vyukov para los mailboxes, `Unsafe` para operaciones atómicas sobre campos. En Rust: un runtime con work-stealing (tokio u otro pool propio) para el dispatcher por defecto, hilos dedicados para `PinnedDispatcher`, colas lock-free (`crossbeam` o propias) y `std::sync::atomic`. Si se porta la cola de Vyukov hay que copiar su licencia BSD (ver `docs/upstream.md`).

**Futures de Scala y `CompletionStage` de Java**. `ask`, `whenTerminated`, `resolveOne`, `gracefulStop` y CoordinatedShutdown devuelven futures. En Rust se mapean a `async fn`/`Future`, lo que obliga a elegir un runtime (o ser agnóstico) desde el principio.

**Biblioteca estándar de Scala**: `PartialFunction` (el `receive`), `Duration`/`FiniteDuration` (incluido `Duration.Inf` y `Duration.Undefined` con significado propio), `Option`, `Try`. Se mapean a una función que devuelve "manejado / no manejado", `std::time::Duration` más un enum para infinito/indefinido, `Option` y `Result`.

**java.net.URI**. El parseo de `Address` y `ActorPath` delega en esa clase y hereda sus rarezas (por ejemplo, rechaza hosts con guión bajo). El plan de AC-PATH ya decidió escribir un parser propio del subconjunto RFC 2396 y validarlo contra el oráculo.

**java.nio** (selectores, canales, buffers directos) para `pekko.io`, y **java.util.logging**/SLF4J como destinos de log. Contraparte: `mio` o sockets de tokio para IO; `log`/`tracing` como backend de logging.

## Preguntas abiertas para el Owner

Hay 313 filas de Classic con una decisión `Q-` en la columna `notes` (los hijos de una fila descompuesta cuentan por separado). El registro único, con evidencia, alternativas y recomendación de cada una, es `docs/decisions.md`. Las agrupo por tema; las cinco primeras bloquean diseño en las unidades núcleo y conviene decidirlas antes de empezar CELL.

1. **Qué es un "fallo" y cómo se clasifica** (AC-SUP-001, AC-SUP-004, AC-SYS-015). Pekko distingue `Exception` (el decider por defecto reinicia) de `Error`/`Throwable` (escala), y trata aparte los errores fatales de la JVM. En Rust hay `Err` devuelto por el handler y `panic`. Propuesta: ambos son fallos supervisables; `Err` se trata como `Exception` y `panic` también, salvo que el Owner prefiera que `panic` escale. Alternativa rechazada: no capturar panics, porque rompería el aislamiento entre actores que es el punto de la supervisión.
2. **Tipado de mensajes en classic** (AC-REF-002, AC-CELL-001, AC-ASK-001). El classic de Pekko es dinámico (`Any` + `PartialFunction`). Propuesta: mensajes `Box<dyn Any + Send>` y un `receive` que hace downcast y devuelve `Handled`/`Unhandled`, para que `unhandled`, `become` y `UnhandledMessage` funcionen igual; el tipado fuerte queda para `pekko-actor-typed`. Alternativa: enums por actor, más idiomático pero no permite reproducir `unhandled` ni `ActorSelection` genérico sin trucos.
3. **Runtime async y modelo de ejecución** (AC-DSP-008, AC-MBX-007, AC-SCH-008). ¿Se ata el port a tokio, o los dispatchers son pools propios con hilos? Afecta qué claves de `fork-join-executor` se honran, cómo se implementa `BoundedMailbox` (que bloquea al emisor, algo que no encaja en async) y si se porta la rueda de timers `LightArrayRevolverScheduler` tal cual o se usan los timers del runtime adaptando sus specs de precisión.
4. **Reemplazo de la reflexión** (AC-CFG-016, AC-CFG-006, AC-EXT-003). Confirmar el registro explícito nombre → fábrica descrito arriba, y qué error se da cuando la config nombra algo no registrado (Pekko lanza `ClassNotFoundException`; proponemos un error de configuración con el nombre).
5. **Jerarquía de tipos en el EventStream** (AC-UTIL-008, AC-EVT-003). Suscribirse a una clase recibe también sus subclases. Rust no tiene herencia. Propuesta: suscripción por `TypeId` exacto más "canales padre" declarados por el tipo de evento (por ejemplo, todo `LogEvent` publica también en el canal `LogEvent`). Bloquea logging y dead letters.
6. **Alcance de piezas grandes y periféricas**: `pekko.io` completo (AC-IO-001), serialización (AC-SER-001), `ByteString` (AC-UTIL-006), CircuitBreaker y Backoff (AC-ASK-011, AC-ASK-015). Ninguna la usa el núcleo. Propuesta: dejarlas en el inventario como unidades posteriores y no bloquear el "100% núcleo" por ellas.
7. **Exclusiones propuestas por ser JVM, Java o deprecadas**: API Java (AC-JAPI-*), `TypedActor`, `Main`, `Switch`, `Backoff` viejo, `schedule` deprecado, `JavaLogger`, serialización Java, interrupción de hilos (AC-CELL-021), hilos virtuales (AC-DSP-014), `AffinityPool`, `ManifestInfo`, claves JVM de config (AC-CFG-007), caches de cluster-sharding (AC-UTIL-017). Propuesta: marcarlas como excluidas con justificación en vez de borrarlas, para que el conteo de "100%" sea explícito. **Hasta que el Owner decida, siguen dentro del alcance** (`impl=pending`).
8. **Logging**: ¿loggers como actores con el handshake `InitializeLogger`/`LoggerInitialized` (fiel a Pekko, AC-EVT-011) o un backend directo a `tracing`? Lo fiel permite portar `LoggerSpec` y `EventFilter` sin cambios.
9. **`TestActorRef` y `CallingThreadDispatcher`** (AC-TK-006): dan acceso síncrono al estado interno del actor, lo que choca con la propiedad en Rust. Propuesta: no portarlos y adaptar las specs que los usan.
10. **Detalles menores**: si `Done` es un tipo o `()` (AC-UTIL-003); qué versión reporta el chequeo de `pekko.version` (AC-CFG-003); si soltar el último handle de `ActorSystem` lo termina (AC-SYS-014, Pekko no lo hace); qué hacer con `provider = remote` (AC-CFG-008); si `serialize-messages` tiene sentido en Rust (AC-CFG-010); hook de señales del sistema operativo para CoordinatedShutdown (AC-CS-007).

## Qué no se terminó

Tras la revisión de contratos, 393 de 507 filas están en `extracted` (se leyó implementación y cuerpos de specs) y 114 siguen en `listed`; el motivo de cada una está en su `notes` y en la sección "Filas que siguen listed" del archivo de contratos de su grupo. `listed` significa que el contrato todavía no es verificable y la fila no puede ejecutarse. Las citas de specs indican la línea del título del test y, para las filas `extracted`, el bloque de contrato cita además las aserciones. No se leyeron los filtros MiMa ni todos los internos de `ByteString`.


Las columnas `impl` y `validation` avanzan unidad por unidad. Estado: 484 filas `pending`, 16 `partial` y 7 `done` en `impl` (el recuento exacto sale de `docs/inventory/actor-classic.tsv`). Lo implementado es AC-PATH y la parte del núcleo de AC-UTIL; los `partial` son pendientes reales (decisiones `Q-` abiertas del Owner o piezas diferidas) y no se cierran por tener tests verdes. El orden de implementación está en `docs/implementation-order.md`, las specs ejecutables en `docs/superpowers/plans/units/` y las reglas de trabajo en `AGENTS.md`.
