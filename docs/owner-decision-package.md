# Paquete de decisiones para el Owner

Preparado el 2026-10-31 a partir de `docs/decisions/register.tsv` (212 filas: 211 decisiones + el encabezado de presentación Q-X-9), `docs/inventory/contracts/*.decisions.tsv` y `docs/decisions.md`. Es un documento de lectura: **ninguna recomendación está aceptada**; sólo el Owner acepta. Las respuestas ya registradas viven únicamente en `docs/decisions/owner-decisions.tsv` y no se reabren.

## 1. Respuestas ya registradas (no reabiertas)

| ID | Estado | Alternativa vigente |
|---|---|---|
| Q-UTIL-1 | rechazada (2026-10-31) | Se conserva el dominio de Pekko: `i64` ns con signo, constructores falibles que rechazan lo que Pekko rechaza. No decide Q-UTIL-2, 3, 4 ni 5. |

## 2. Resumen ejecutivo

Estado: 211 decisiones (202 propuestas, 8 bloqueantes, 1 rechazada/respondida) + Q-X-9 (grupo).

| Clasificación | Nº | Qué exige del Owner |
|---|---|---|
| Representación que conserva el contrato | 60 | Aceptar o cambiar; sin pérdida funcional. |
| Arquitectura | 64 | Elegir; las transversales condicionan a las demás. |
| Divergencia funcional | 47 | **Aceptación explícita**; no es adaptación idiomática. |
| Exclusión de alcance | 27 | **Autorización explícita** (A-4/A-5). |
| Resuelta por investigación | 12 | Sin acción (sección 5); algunas piden registrar confirmación. |
| Licencia | 1 | Decisión legal (Q-PATH-6). |

Orden sugerido: (1) Q-X-4, Q-X-3, Q-X-8, Q-X-5, Q-X-2, Q-X-1, Q-X-7, Q-X-6, Q-X-10; (2) los 8 bloqueantes: Q-FSM-4, Q-RT-4, Q-SER-1, Q-T-BEH-1, Q-T-SUP-3, Q-TK-1, Q-UTIL-12, Q-UTIL-6; (3) las exclusiones de la sección 7; (4) el resto por unidad. Lo que está pendiente de las transversales no puede cerrarse aunque se responda antes.

Las 26 decisiones con ficha (sección 3: 10 transversales y 16 de unidad) cubren la mayor parte de las filas bloqueadas; la sección 4 trae las 186 restantes de unidad.

## 3. Fichas de las decisiones de mayor impacto

Alternativas, "(a)/(b)/(c)" y recomendaciones son las de las fichas `*.decisions.tsv`; las citas `path:line` son de `upstream/pekko` (verificadas salvo las marcadas "según la ficha").

### 3.1 Transversales

Las filas/unidades entre paréntesis son `propias → con dependientes` (ver sección 6). "Dependientes" son las decisiones por unidad cuya ficha cita la transversal; es un piso, porque el efecto real (p. ej. de Q-X-3 sobre todo el paso de mensajes Classic) es mayor.

#### Q-X-8 — Typed sobre Classic o runtime tipado nativo (260 → 265 filas, 23 → 24 unidades; 7 dependientes)
- **Pregunta:** ¿`rakka-actor-typed` replica las capas de Pekko (ActorAdapter/ActorContextAdapter/ActorSystemAdapter sobre Classic) o es un runtime tipado nativo con interop aparte?
- **Filas:** todas las AT-* (260), con efecto directo en AT-SYS-011, AT-ADP-*, AT-CTX-017, AT-SUP-020/021, AT-TK-003..005/011/012.
- **Evidencia Pekko:** `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/` (ActorAdapter, ActorContextAdapter, ActorSystemAdapter, EventStreamAdapter, GuardianStartupBehavior...); `ActorAdapter.scala:105` (adapt/mensajes registrados); `Supervision.scala:123` publica `Dropped` por el eventStream clásico.
- **Alternativas:** (a) replicar las capas; (b) runtime nativo + interop.
- **Consecuencias observables:** con (b) deben revalidarse las reglas del adaptador (despacho, resultado, hooks, supervisión de hijos: Q-T-ADP-2), el Stop por defecto de un typed lanzado desde classic (Q-T-ADP-3) y el `watch` sobre el ref adaptador (Q-T-CTX-1); sin capa clásica no hay hijos/decider clásicos (Q-T-CTX-4, Q-T-SUP-7).
- **Recomendación (no aceptada):** (a): cada contrato heredado se implementa y valida una sola vez; la interop sale casi gratis.
- **Bloquea:** `impl` de AT-TK-003.*/004.*/005.*/011/012 (Q-T-TK-4), AT-CTX-017, AT-SUP-020/021. **Puede avanzar:** todo Classic, los contratos AT-* y las filas que no construyen Behaviors (SIG, REF, STASH solo estructura).

#### Q-X-4 — Runtime y modelo de ejecución (5 → 79 filas, 3 → 13 unidades; 15 dependientes)
- **Pregunta:** ¿pools de hilos propios con interfaz de ejecutor, o port atado a tokio?
- **Filas:** AC-DSP-008, AC-MBX-007, AC-SCH-008 y, por dependientes, DSP, MBX, SCH, CELL, ASK, CS, FSM, EXT, TK, IO, UTIL-012/013, AT-DSP, AT-TK.
- **Evidencia Pekko:** `Mailbox.scala:560-562, 605-620, 705` (`BoundedMailbox` hace `queue.offer(handle, pushTimeOut)`: el emisor se bloquea); `Dispatchers.scala` y claves `fork-join-executor`; `CallingThreadDispatcher.scala:162`.
- **Alternativas:** (A) pools propios + futuros agnósticos; (B) atar el port a tokio.
- **Consecuencias observables:** (B) no permite el bloqueo del emisor con `push-timeout>0` ni `PinnedDispatcher` (Q-MBX-1 pasa a divergencia); entrega síncrona de la CTD (Q-TK-1) y claves de executor (Q-DSP-1) dependen de esto.
- **Recomendación:** (A); tokio como adaptador opcional posterior.
- **Bloquea:** `impl` de AC-MBX-002/007, AC-DSP-002/003/008, AC-FSM-003/004, AC-CS-004.*, AC-EXT-001.2, AC-TK-001.x. **Puede avanzar:** trait de ejecutor, lógica pura (aritmética de pools, colas no acotadas), contratos.

#### Q-X-3 — Tipo de los mensajes en Classic (5 → 31 filas, 3 → 10 unidades; 9 dependientes)
- **Pregunta:** `Box<dyn Any + Send>` con `receive` que hace downcast, o enum por actor.
- **Evidencia Pekko:** `Actor.scala:390` (`type Receive = PartialFunction[Any, Unit]`), `Actor.scala:656` (`unhandled`); `ActorRef.scala:43` (`noSender`).
- **Alternativas:** (A) `dyn Any` + Handled/Unhandled; (B) enum por actor.
- **Consecuencias:** `unhandled`, `become`, `UnhandledMessage`, routers, ask y mailbox de control dependen de poder rechazar un mensaje; (B) los pierde.
- **Recomendación:** (A); el tipado fuerte vive en `rakka-actor-typed`.
- **Bloquea:** `impl` de AC-REF-002, AC-CELL-001, AC-ASK-001, AC-RT-*, AC-SCH-011..013 (hasta Q-X-3), AC-MBX-012. **Avanza:** contratos, PATH/UTIL/CFG.

#### Q-X-5 — Qué es un fallo supervisable (4 → 13 filas, 3 → 4 unidades; 3 dependientes)
- **Pregunta:** ¿`Err` y `panic` son fallos supervisables tratados como `Exception`?
- **Evidencia Pekko:** `FaultHandling.scala:216-223` (`defaultDecider`: Initialization/Killed/DeathPact → Stop, `Exception` → Restart; "The error is escalated if it's a `Throwable`, i.e. `Error`"); `FaultHandling.scala:309, 372` (`escalateDefault`); `dungeon/FaultHandling.scala:353-359` (sólo `NonFatal` entra a supervisión); `ActorSystem.scala:862-880` (fatal: terminate o `System.exit`).
- **Alternativas:** (A) `Err` y `panic` = Exception; (B) `panic` escala; (C) no capturar panics.
- **Observación (ver 8):** la redacción de Q-X-5 (A) "tratados como Exception" contradice Q-SUP-1, que prevé `Panic → Escalate` (equivalente de Error). En Pekko un `Error` no fatal escala (no reinicia). El Owner debe decidir también ese mapeo, no sólo capturar o no.
- **Recomendación (no aceptada):** capturar panics (A o B); el mapeo Restart/Escalate queda a decisión. (C) rompe el aislamiento entre actores.
- **Bloquea:** `impl` de AC-SUP-001/004/012/017, AC-SYS-015, AT-SUP-002, deciders de AC-ASK-015. **Avanza:** ciclo de vida sin deciders, aritmética de AC-SUP-006.

#### Q-X-2 — Registro nombre → fábrica en vez de reflexión por FQCN (29 → 36 filas, 7 → 10 unidades; 5 dependientes)
- **Pregunta:** ¿`FactoryRegistry<Producto>` por punto de extensión, registro único con `Any` o macros?
- **Evidencia Pekko:** `ReflectiveDynamicAccess.scala:37-57` (`getClassFor`/`createInstanceFor` por FQCN); `ActorSystem.scala:958, 975, 1148` (logging-filter, provider y scheduler por reflexión); `DynamicAccessSpec.scala:40-72`.
- **Consecuencias:** la carga por nombre de dispatchers, mailboxes, schedulers, serializadores, extensiones y loggers sólo funciona para nombres registrados; error de configuración con el nombre si falta. Es **divergencia funcional parcial** (se pierde la reflexión pura), no mera adaptación.
- **Recomendación:** (A). Abierto: tipo del error (`ConfigurationException` o propio).
- **Bloquea:** carga por nombre de AC-CFG-016, AC-EXT-003, AC-MBX-004, AC-DSP-004/010, AC-SCH-010, AC-EVT-011, AC-SER-*. **Avanza:** registro programático.

#### Q-X-7 — Loggers como actores o backend directo (28 → 39 filas, 2 → 6 unidades; 4 dependientes)
- **Evidencia Pekko:** `Logging.scala:216` (`Await.result(actor ? InitializeLogger(this)...)`), `Logging.scala:966` (`InitializeLogger`); `LoggerSpec`/`EventFilter` dependen del handshake (AC-EVT-011).
- **Alternativas:** (A) loggers como actores con handshake; (B) backend directo a `tracing`.
- **Consecuencias:** (B) impide portar `LoggerSpec` y `EventFilter` sin cambios (AC-TK-*, AT-TK-006 LoggingTestKit).
- **Recomendación:** (A); `tracing` puede ser un logger más.
- **Bloquea:** AC-EVT-011, AC-CFG-018, AT-LOG-003, AT-BEH-018/019, AT-TK-006.*. **Avanza:** resto de EVT.

#### Q-X-1 — Implementación de HOCON (8 → 15 filas, 1 → 4 unidades; 4 dependientes)
- **Evidencia:** `ActorSystem.scala:281` (`ConfigFactory.load`); plan `docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md`: golden de 161 casos contra Typesafe Config 1.4.9; `hocon-parser` 1.14.2 diverge en `+=`, duraciones negativas, `"ON"` e `include classpath()`.
- **Alternativas:** (A) subconjunto propio; (B) `hocon-parser` tras fachada con `Q:` por divergencia; (C) (B) + parche upstream. Consecuencia de (B)/(C): divergencias funcionales conocidas.
- **Recomendación:** (A) con el golden como suite de aceptación. **Bloquea:** AC-CFG-001, AC-CFG-017.*, checkValid (Q-CFG-3), AC-RT-004. **Avanza:** lectura tipada de claves, Duration (Q-UTIL-1 ya respondida).

#### Q-X-6 — Suscripción por jerarquía de tipos en EventStream (2 → 14 filas, 2 → 4 unidades)
- **Evidencia:** `EventStream.scala:28, 34, 44` (SubchannelClassification con `Subclassification[Class]`); `EventBus.scala:136-141`; `Serialization.scala:327-362` (también usa `isAssignableFrom`).
- **Alternativas:** (A) `TypeId` exacto + canales padre declarados; (B) enum cerrado. **Consecuencia:** un suscriptor a un supertipo sólo recibe subtipos si éstos los declaran.
- **Recomendación:** (A). **Bloquea:** AC-EVT-003, AC-UTIL-008, y reutilizable por Q-SER-1 y Q-JAPI-2. **Avanza:** el resto de EVT.

#### Q-X-10 — Límites del módulo (remote, cluster, streams, persistence, http) (2 → 3 filas)
- **Evidencia:** `ActorSystem.scala:112-115, 975` (providers `remote`/`cluster` por FQCN); ganchos AC-SYS-018, AC-DW-010/012, AC-SER-007.
- **Alternativas:** (A) fuera de alcance, `ActorRefProvider` abierto; (B) incluir alguno. **Es una exclusión de alcance** (A-6 no la cubre: el límite "no fue autorizado explícitamente").
- **Recomendación:** (A). **Bloquea:** cierre de AC-SYS-018/AC-CFG-008. **Avanza:** todo lo local.

#### Q-X-9 — Grupo de presentación Q-UTIL-1..5 (6 → 20 filas)
Se responde por miembro, nunca en bloque. **Q-UTIL-1 ya respondida** (sección 1). Pendientes (todas bloquean sólo el cierre de la fila, no su implementación):
- **Q-UTIL-2** (`toString` de Timeout conserva la unidad): recomendación (a), sólo logs; cierra AC-UTIL-001.
- **Q-UTIL-3** (`null`/mensajes diferidos en PekkoException y requiring, divergencia de lenguaje): (a) sin `null`, igual que Q-PATH-4; cierra AC-UTIL-002.3/004.1/004.2.
- **Q-UTIL-4** (jerarquía de errores + `OnlyCauseStackTrace`): (i)(a); (ii) excluir exige autorización (A-5); cierra AC-UTIL-004.2/.3.
- **Q-UTIL-5** (`Done` propio o `()`): el plan no registra recomendación; cierra AC-UTIL-003.1/.2.
Nota: Q-UTIL-1 elegida i64 ns con signo condiciona Q-ASK-1/Q-CS-7 (Display de Duration).

### 3.2 Por unidad con mayor impacto

#### Q-UTIL-11 — Elementos JVM/Scala sin equivalente natural (34 filas, UTIL)
- Propone equivalente (Option, atómicos, `pub(crate)`...) para AC-UTIL-014..022. **Recomendación:** (a) ahora; cualquier exclusión (b) requiere aprobación **fila por fila** (A-5). Bloquea: nada. Es la decisión con más filas tras Q-X-8: aceptar (a) sin más no excluye nada.

#### Q-T-DLV-1 — Alcance de delivery (22 filas, AT-DLV)
- **Evidencia:** `actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/` (ProducerController, ConsumerController, WorkPullingProducerController, DurableProducerQueue) e `internal/` (ChunkedMessage...). **Recomendación:** (a) variante local con cola en memoria + (b) cola durable de referencia (Q-T-DLV-2); **exclusión acotada**: no hay durabilidad real tras reiniciar el proceso. Bloquea: nada; avanza protocolo y reintentos verificables.

#### Q-JAPI-1 / Q-T-JAPI-1 — Fachadas Java (13 + 16 filas)
- **Evidencia:** `actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala:26` (`ReceiveBuilder`), `actor/src/main/java/` (30 archivos Java; AbstractActorRef, AbstractMailbox...). **Recomendación:** (a) excluir nombres/clases Java; la semántica la cubren las filas Rust equivalentes. **Exclusión de alcance explícita**: sin autorización las filas se implementan (b). Bloquea el cierre de 13 + 16 filas, nada de CELL/FSM/EVT/IO/SCH. Relacionadas: Q-JAPI-2/3/4, Q-T-JAPI-2.

#### Q-T-BEH-1 — Forma del tipo `Behavior` (6 filas directas, BLOQUEANTE)
- **Evidencia:** `Behavior.scala:48-49` (`abstract class Behavior[T]` `@DoNotInherit`); `BehaviorImpl.scala:38, 123` (`DeferredBehavior`: setup re-ejecutable). **Consecuencia:** sin una forma reutilizable/re-ejecutable no hay restart ni AT-BEH-023. **Recomendación previa (a), no aceptada.** Bloquea AT-BEH-001/004/007/011/023, AT-SUP-005/013 y (Q-T-RT-3) AT-RT-001/003/004/005. Avanza lo que no construye Behaviors.

#### Q-CELL-1 — Forma de `receive` y pila become/unbecome (2 filas, pero firma del trait `Actor`)
- **Evidencia:** `Actor.scala:390, 656`. **Recomendación:** (a) `Handled::{Handled,Unhandled}`; pila limpiada en restart. Bloquea `impl` de AC-CELL-001/005 y la firma de `Actor` para todo el crate. Depende de Q-X-3 (misma respuesta).

#### Q-UTIL-12 — ByteString/ByteIterator/ByteStringBuilder (9 filas, BLOQUEANTE)
- **Evidencia:** `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala`. Alternativas: (a) `bytes::Bytes`; propio; con `base64`/`encoding_rs`. Bloquea AC-UTIL-006.*/022 y por dependencia AC-IO-*/AC-SER-*. Decidir antes de IO/SER; no bloquea UTIL/CFG/PATH/EXT. La ficha contiene la comparación completa.

#### Q-SER-1 — Bindings y manifests sin jerarquía JVM (7 filas, BLOQUEANTE)
- **Evidencia:** `Serialization.scala:319-362` (`isAssignableFrom`, desempate y aviso "Multiple serializers found"). **Recomendación:** (a)+(c): cada tipo registrado declara nombre lógico y supertipos; FQCN de `reference.conf` como alias; reutiliza Q-X-6. Bloquea AC-SER-001..003.

#### Q-TK-1 — Entrega síncrona del `testActor` (5 filas, BLOQUEANTE)
- **Evidencia:** `CallingThreadDispatcher.scala:162`; `TestProbeSpec.scala:32-57`, `TestActorRefSpec.scala:224-230`. **Recomendación:** (a) CTD síncrono; sin él fallan 1:1 los specs con `expectMsg(0.millis)`. Bloquea AC-TK-001.1/002.1/006.1/006.2/007; avanza el resto con sondeo. Depende de Q-X-4. Relacionadas: Q-T-TK-1 (10 filas), Q-TK-4.

#### Q-IO-1 — Modelo de ejecución de `pekko.io` (14 filas)
- **Evidencia:** `io/SelectionHandler.scala:39, 313`. **Recomendación:** (c) protocolo público idéntico + costura `ChannelRegistry` + claves de selectores aceptadas. Depende de Q-X-4. Bloquea `impl` de AC-IO-001..004. Conexas: Q-IO-2 (6), Q-IO-3 (9, exclusión parcial), Q-IO-5 (defecto confirmado de 1.7.1).

#### Q-REF-2 — Props sin reflexión (4 filas)
- **Evidencia:** `Props.scala:57, 72, 87` (`classOf`, `ClassTag`). **Divergencia funcional parcial:** se pierden `Props(classOf[A], args)` y los mensajes de `findConstructor`/`validate`. **Recomendación:** (a) closures + (b) registro por nombre (Q-X-2); la fila sigue en alcance. Bloquea AC-REF-008 y la parte reflexiva de AC-REF-007.1/.2.

#### Q-MBX-1 — BoundedMailbox con push-timeout (2 filas)
- **Evidencia:** `Mailbox.scala:560-562, 605-620`. **Recomendación:** (c) timeout 0 no bloqueante; >0 bloquea sólo fuera de worker async (condicionada por Q-X-4). Bloquea AC-MBX-007/016 y variantes acotadas con timeout>0.

#### Cuatro bloqueantes puntuales
- **Q-FSM-4** (AC-FSM-006.1/.2): `FSM.scala:881-884` (`makeTransition` llama `terminate(...)` sin `context.stop(self)`: el actor sigue vivo). Divergencia si se detiene. Recomendación: (a) replicar, confirmando con oráculo antes de implementar. Bloquea AC-FSM-006.1 A1/A4.
- **Q-RT-4** (AC-RT-008/013.1/013.2/014): `SmallestMailbox.scala:79-84` usa `hasMessages`/`numberOfMessages`. Recomendación: (a) vista de mailbox; bloqueante si AC-MBX-019 no la ofrece. Bloquea integración de AC-RT-008/013.2/014.
- **Q-T-SUP-3** (AT-SUP-007.2/.3): `Supervision.scala:335` trata sólo `-1` como ilimitado. Parte resuelta por lectura (`resetBackoffAfter` = (min+max)/2); `maxRestarts<=0` se mide con el oráculo y entonces decide el Owner. Bloquea AT-SUP-007.3.
- **Q-UTIL-6** (AC-UTIL-005.2): **confirmado leyendo** `Version.scala:172-177`: el segundo `if` no es `else if`, por lo que `"" ` vs calificador cae al `rest.compareTo` y la comparación es asimétrica. Recomendación: replicar hasta decisión del Owner (defecto de Pekko; toda diferencia es regresión). Bloquea AC-UTIL-005.2 A5/A8 y AC-UTIL-016.9.

## 4. Tabla de las demás decisiones, por unidad

Columnas: "Filas/uds" = filas y unidades afectadas (expandidos hijos `.n` y comodines); "Bloquea" = lo que la ficha dice que impide cerrar (`impl` de la fila, sólo validación/cierre, o nada). Clase y recomendación en una línea; el detalle y las alternativas están en `docs/decisions.md` (sección 5) y en los `*.decisions.tsv`. "(resp.)" = respuesta del Owner registrada.

**ASK**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-ASK-1 | Modelo de Future de ask/gracefulStop/after/retry y tipado del resultado | 8/1 | Arquitectura | impl | (a) Future propio; errores idénticos donde el spec los fija (AskSpec:66-69,77,87). |
| Q-ASK-2 | Ref temporal: path lazy, estados atomicos y texto de error de PromiseRef | 2/1 | Representación | validación/cierre | (a) replicar PromiseRef con el texto exacto de Pekko. |
| Q-ASK-3 | CircuitBreaker: mapeo de tiempos, ejecutor de listeners y semantica de saturacio | 4/1 | Representación | impl + validación | (a) replicar la secuencia de resetTimeout; (b) sólo con autorización. |
| Q-ASK-4 | CircuitBreaker: callTimeout=Duration.Zero ignora defineFailureFn; allowlist por  | 2/1 | Representación | impl + validación | (a) callTimeout = 0 ignora defineFailureFn; allowlist por nombre en registro explícito. |
| Q-ASK-5 | API sincrona de CircuitBreaker (withSyncCircuitBreaker bloquea con Await) | 1/1 | Representación | impl | (a) exponer withSyncCircuitBreaker (bloquea el hilo); no excluir sin autorización. |
| Q-ASK-6 | CircuitBreakerTelemetry: carga por reflexion (FQCN) y clave de config ausente de | 1/1 | Arquitectura | impl | (a) misma clave `telemetry.implementations`; carga por registro de fábricas. |
| Q-ASK-7 | BackoffSupervisor: composicion de strategy y directivas (Restart traducido a Sto | 5/1 | Arquitectura | impl | (a); depende de Q-X-5 y del modelo de supervisión (AC-SUP-001). |
| Q-ASK-8 | API deprecada Backoff / BackoffOptions: propuesta de no portar | 1/1 | Exclusión de alcance | impl | (b) excluir Backoff/BackoffOptions deprecados; BackoffOpts es el equivalente. |
| Q-ASK-9 | Fachadas Java Patterns/PatternsCS: propuesta de no portar | 1/1 | Exclusión de alcance | impl | (b) excluir las fachadas Java Patterns/PatternsCS. |
| Q-ASK-10 | Prefijo "requirement failed: " en los mensajes de require (BackoffOpts, CircuitB | 7/2 | Divergencia funcional | nada | (c) mensajes de require sin el prefijo "requirement failed: "; (a) si se quiere paridad textual. |

**CELL**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-CELL-2 | Interrupción de hilo (InterruptedException → ActorInterruptedException) | 1/1 | Exclusión de alcance | validación/cierre | (a) excluir Thread.interrupt/ActorInterruptedException; exige autorización explícita. |
| Q-CELL-3 | Uso del contexto/ActorRef::self tras terminar el actor | 1/1 | Representación | validación/cierre | (b) contexto no capturable por lifetime + (a) refs clonadas; Pekko no anula el contexto. |
| Q-CELL-4 | Texto de los eventos Debug que incluyen toString de actor/envelope/ref ("started | 1/1 | Representación | validación/cierre | (b) prefijo fijo del texto Debug; (a) para el resto con toString propio. |
| Q-CELL-5 | Modos de test serialize-messages y serialize-creators (ida y vuelta de mensajes  | 2/1 | Exclusión de alcance | impl | (a) modos serialize-* tras AC-SER; entre tanto `impl=pending`. |

**CFG**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-CFG-3 | checkValid contra reference.conf (Typesafe Config) con un parser propio | 2/1 | Arquitectura | impl | (a) checkValid propio si Q-X-1 elige (A); con parser ajeno se pierde el fail-fast con lista de errores. |
| Q-CFG-4 | Fidelidad de errores y render de Config (tipos y textos de ConfigException, rend | 12/2 | Representación | impl + validación | (a) tipos y textos de ConfigException del golden; render sin exigir igualdad textual. |
| Q-CFG-5 | Valor de `pekko.version` y de `ActorSystem.Version` en rakka | 2/2 | Representación | impl + validación | (a) `pekko.version` = 1.7.1 mientras Pekko sea la fuente de verdad (PekkoVersion.require). |
| Q-CFG-6 | Fuentes de la config de aplicación cuando no se pasa Config | 1/1 | Arquitectura | impl | (a) ruta configurable para application.conf; sin classpath scanning. |
| Q-CFG-7 | Claves de JVM: qué semántica tienen en Rust (daemonic, jvm-exit-on-fatal-error,  | 2/2 | Exclusión de alcance | impl | (a) aceptar siempre las claves JVM (daemonic, exit-on-fatal, shutdown-hooks...) sin su efecto JVM. |
| Q-CFG-8 | Nombres estables para el registro de extensiones/fábricas (FQCN con `$` de objet | 2/2 | Representación | impl | (b) los FQCN de reference.conf son alias estables del registro de fábricas. |
| Q-CFG-9 | Proveedor distinto de local (remote, cluster, FQCN) en un sistema solo-local | 2/2 | Arquitectura | impl | (a)+(c) error explícito ante remote/cluster/FQCN en sistema local; puerta abierta a remote. |
| Q-CFG-10 | pekko.use-slf4j / amendSlf4jConfig | 2/1 | Arquitectura | impl | Sigue a Q-X-7: `pekko.use-slf4j` activa Slf4jLogger/filtro; bloquea sólo AC-CFG-018. |
| Q-CFG-11 | serialize-messages / serialize-creators / no-serialization-verification-needed-c | 1/1 | Divergencia funcional | impl | (a) aceptar las claves en `off` hasta que exista AC-SER; modos serialize-* diferidos (ver Q-SER-4). |
| Q-CFG-12 | Deploy.withFallback descarta los tags de `this` | 1/1 | Resuelta por investigación | nada | (a) replicar: `Deploy.withFallback` no copia `tags` (Deployer.scala:109-117); registrar confirmación en `notes`. |
| Q-CFG-13 | ActorSystemSetup: identidad por clase concreta del Setup | 1/1 | Representación | nada | (a) identidad por clase concreta del Setup (ActorSystemSetupSpec.scala:39-45). |

**CS**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-CS-1 | Fases: errores de config/validacion como Result y orden entre fases independient | 3/1 | Representación | impl | (a) Result con mensajes de Pekko; orden determinista (BTreeSet) documentado. |
| Q-CS-2 | Modelo de tareas de apagado: identidad por objeto, cancelacion, paralelismo y Fu | 6/1 | Arquitectura | impl | (a); depende de Q-X-4 y Q-X-9 (Done). |
| Q-CS-3 | Fases de cluster predefinidas sin cluster (vacias en local) | 1/1 | Representación | impl | (a) mantener las fases de cluster (vacías en local). |
| Q-CS-4 | Identificador estable de Reason y claves reason-overrides | 1/1 | Representación | impl | (a) ids estables de Reason para `reason-overrides` (HOCON). |
| Q-CS-5 | exit-jvm / System.exit en Rust y terminacion del ActorSystem desde la fase final | 2/1 | Representación | impl | (a) mantener `exit-jvm`; "jvm" significa proceso. |
| Q-CS-6 | Hook de apagado del proceso: senales y registro en el runtime | 2/1 | Divergencia funcional | impl | (a) hook de señales; SIGINT soportado difiere de Pekko. |
| Q-CS-7 | Formato textual de Duration y colecciones en mensajes de log/error de Coordinate | 5/3 | Divergencia funcional | validación/cierre | (a) un Display de Duration para los textos que afirman los specs; ISO-8601 Java fuera. |

**DSP**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-DSP-1 | Qué claves de executor se honran: fork-join, thread-pool, affinity-pool, virtual | 7/1 | Exclusión de alcance | impl | (b) equivalentes propios de fork-join/thread-pool/affinity-pool; virtual-thread sin efecto. |
| Q-DSP-2 | Registro de configuradores de dispatcher/executor/mailbox por FQCN (type, execut | 3/2 | Arquitectura | impl | Subordinada a Q-X-2: aceptar los FQCN de Pekko como alias de builtins. |
| Q-DSP-3 | ExecutionContext/Future helpers (AC-DSP-017): portar sólo parasitic o todo | 4/1 | Exclusión de alcance | validación/cierre | (a) portar InlineExecutor y SerializedSuspendable; excluir AC-DSP-017.2 (futuros Java). |
| Q-DSP-4 | Detección de ciclos de alias de dispatcher y profundidad máxima | 2/1 | Resuelta por investigación | nada | (a) MaxDispatcherAliasDepth = 20 y textos literales (Dispatchers.scala:77-87). |
| Q-DSP-5 | throughput <=0: documentación (continúa hasta vaciar) vs código (equivale a 1) | 2/2 | Resuelta por investigación | nada | (a) throughput <= 0 equivale a 1 (Mailbox.scala:264); anotar la diferencia con el Scaladoc. |
| Q-DSP-6 | minimum-runnable = -1 y valor por defecto efectivo en Rust (sin JDK) | 2/1 | Divergencia funcional | nada | (b) `minimum-runnable = -1` se comporta como JDK < 25; valor efectivo 1. |
| Q-DSP-7 | BatchingExecutor y reenvío ante bloqueo (BlockContext) en Rust | 1/1 | Divergencia funcional | impl | (a) BatchingExecutor mínimo; A2-A4/A6/A7 de AC-DSP-016.2 dependen de blocking/Await. |

**DW**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-DW-1 | Igualdad del mensaje Terminated y existenceConfirmed para actores ya muertos | 2/1 | Representación | impl | (a) Terminated con igualdad por actor; existenceConfirmed fuera de equals. |
| Q-DW-2 | Hooks de remote en DeathWatch: AddressTerminated, AddressTerminatedTopic y orden | 2/1 | Exclusión de alcance | impl | (a) mínimo: `is_local` y conjuntos watching/watchedBy sin remote. |

**EVT**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-EVT-1 | Nombres de clase JVM en mensajes observables (dead letters, canales Debug, `clas | 8/2 | Divergencia funcional | validación/cierre | (b) nombres de tipos estándar que afirman los specs; (a) resto; función `message_type_name`. |
| Q-EVT-2 | Fidelidad byte a byte de la salida stdout de los loggers por defecto | 2/1 | Arquitectura | validación/cierre | (c) con (a) como default; stdout no byte a byte. |
| Q-EVT-3 | Plantillas `{}` y varargs en LoggingAdapter | 1/1 | Representación | nada | (a) plantillas `{}` y sufijo "arguments left" con macros. |
| Q-EVT-4 | JavaLogger (java.util.logging) en Rust | 1/1 | Representación | nada | (a) JavaLogger como puente opcional a `log` (feature). |
| Q-EVT-5 | API Java (EventBusJavaAPI, javadsl TestKit/EventFilter) | 2/2 | Representación | nada | (a) sin API Java; la cubren las filas EVT/TK. |
| Q-EVT-6 | Asimetría de LogSource.fromAnyRef con sistema (getLogger vs Logging.apply) | 1/1 | Resuelta por investigación | nada | (a) replicar la asimetría de LogSource.fromAnyRef; unificar sería divergencia. |
| Q-EVT-7 | LoggingReceive exige is_defined_at antes de ejecutar el Receive | 1/1 | Arquitectura | validación/cierre | (a) o (c): is_defined_at antes de ejecutar el Receive (Q-X-3). |

**EXT**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-EXT-1 | Identidad del ExtensionId | 2/1 | Representación | impl | (a) identidad por instancia del ExtensionId (Extension.scala:123-124). |
| Q-EXT-2 | Semántica de registerExtension: bloqueo, reentrada, caché de fallo | 1/1 | Arquitectura | impl | Depende de Q-X-4: (a) bloqueante con hilos, (c) con async. |
| Q-EXT-3 | AC-EXT-004 como fila índice de extensiones provistas | 1/1 | Arquitectura | validación/cierre | (a) AC-EXT-004 es fila índice; su impl se deriva de las 11 filas dueñas. |

**FSM**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-FSM-1 | FSM en Rust: funciones parciales (when/whenUnhandled/transform), tipos S y D, re | 5/1 | Arquitectura | impl | (a) paridad con classic (funciones parciales); (b) corresponde a Typed. |
| Q-FSM-2 | Timers de FSM y state timeout: sobre el scheduler y generacion de cancelacion | 2/1 | Arquitectura | impl | (a) paridad de cancelación de timers ya encolados; depende de Q-X-4. |
| Q-FSM-3 | Textos de log exactos de FSM y debug.fsm | 2/1 | Representación | impl + validación | (a) textos de log exactos (son aserciones de spec). |

**IO**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-IO-2 | Estrategia de port de specs de conexión que simulan el selector | 6/1 | Arquitectura | validación/cierre | (b) si Q-IO-1 adopta ChannelRegistry; si no, (c) aserciones de secuencia de mensajes. |
| Q-IO-3 | Piezas NIO/JVM/SO específicas: buffers directos y limpieza, workaround de Window | 9/1 | Exclusión de alcance | impl | (a) buffers/socket2/WritePath/DatagramSocketFactory; (b) excluir ByteBufferCleaner, workaround Windows, finish-connect-retries. |
| Q-IO-4 | Resolver inet-address: caché y propiedades JVM, preferencia IPv6, API deprecada  | 4/1 | Divergencia funcional | impl | (a) TTL e IPv6 sin propiedades JVM; la clave Ip() se corrige y se anota. |
| Q-IO-5 | Nombres de host en UDP y UdpConnected: Send con miss de caché y doConnect que ig | 2/1 | Divergencia funcional | impl | (b) corregir un defecto de 1.7.1 confirmado con el jar (Connect por nombre siempre falla). |
| Q-IO-6 | Nameservers por defecto del async-dns y alcance de /etc/resolv.conf | 3/1 | Divergencia funcional | impl | (a) en Unix leer nameserver de /etc/resolv.conf (ampliación); Windows/macOS pendiente. |

**JAPI**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-JAPI-2 | ReceiveBuilder/PFBuilder/Match para receive dinámico: igualdad de tipo exacto vs | 5/1 | Divergencia funcional | impl | (a) TypeId exacto; (b) con supertipos si Q-X-6 los define; isInstance se documenta. |
| Q-JAPI-3 | TypedActor clásico deprecado (proxies JDK dinámicos) | 3/1 | Exclusión de alcance | impl | (a) excluir TypedActor clásico deprecado (proxies JDK); (b) paridad con invoke. |
| Q-JAPI-4 | Divergencias de la fachada Java si se porta: quirks no replicados | 4/1 | Divergencia funcional | nada | (a) si se porta la fachada; (c) si se acepta Q-JAPI-1 (a): quirks de la fachada no replicados. |

**MBX**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-MBX-2 | Identidad de tipo de cola / semántica: reflexión de marker traits vs. descriptor | 3/1 | Representación | nada | (a') semánticas abiertas con implicancias codificadas, en vez de marker traits. |
| Q-MBX-3 | ControlMessage: detección de mensajes de control en mensajes `dyn Any` | 1/1 | Representación | impl | (b)+(c) registro por TypeId de mensajes de control + predicado en MailboxType (Q-X-3). |
| Q-MBX-4 | Comparator de prioridad sobre Envelope: orden estable bajo empate para Unbounded | 2/1 | Resuelta por investigación | nada | (b) sin orden entre iguales en el mailbox de prioridad; el estable lleva secuencia. |
| Q-MBX-5 | Aislamiento de tipos de mensajes de sistema vs. usuario en el mailbox (lista de  | 3/1 | Arquitectura | nada | (a) dos colas (sistema y usuario); la de sistema se drena entera primero. |
| Q-MBX-6 | Modelo de ejecución del mailbox (hilo + ForkJoinTask vs. tarea async): tareas po | 3/2 | Arquitectura | impl | Subordinada a Q-X-4: turno de mailbox = trabajo del pool (paridad) o tarea async. |
| Q-MBX-7 | PriorityGenerator.compare: resta de Int con desbordamiento | 2/1 | Divergencia funcional | validación/cierre | (a) resta de Int con desborde envolvente como Pekko; documentarlo. |

**PATH**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-PATH-1 | Cómo ordenar listas de Address con un comparador que no es orden total | 1/1 | Divergencia funcional | nada | (a) comparador `pekko_compare`; no confiar en List.sort hasta tener consumidor (cluster). |
| Q-PATH-2 | Surrogate suelto en el mensaje de validación de nombres | 1/1 | Divergencia funcional | validación/cierre | (a) posición UTF-16 en el mensaje; el surrogate suelto no existe en `String`. |
| Q-PATH-3 | Valor de hashCode de Address y ActorPath | 2/1 | Representación | nada | (a) hashCode propio; (b) sólo si una unidad lo necesita. |
| Q-PATH-4 | `null` de Java en Address/ActorPath | 4/1 | Divergencia funcional | nada | (a) sin `null`; `host = null` se modela como `None`. |
| Q-PATH-5 | Fachada Java ActorPaths | 1/1 | Representación | validación/cierre | (b)/(c) las funciones libres cubren ActorPaths; excluir la fachada exigiría autorización. |
| Q-PATH-6 | Licencia de uri.rs y jstring.rs (comportamiento portado de OpenJDK 21, GPL-2.0 W | 7/1 | Licencia | validación/cierre | (b), o (a) con revisión legal: licencia de uri.rs/jstring.rs (OpenJDK GPLv2+CPE). |

**REF**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-REF-1 | Representación de "sin sender" y valor de sender() fuera de un mensaje | 3/1 | Representación | impl | (a) `tell(msg, Option<&ActorRef>)` y `sender()` siempre ActorRef (deadLetters). |
| Q-REF-3 | Serialización Java de ActorRef (writeReplace / readResolve / SerializedActorRef) | 1/1 | Exclusión de alcance | impl | (a)+(b) sin Java serialization; error del resolver si no hay sistema en contexto. |
| Q-REF-4 | Restricción "los actores sólo se crean vía actorOf" (ActorInitializationExceptio | 1/1 | Representación | validación/cierre | (a) la restricción de `actorOf` pasa a ser estructural; no se porta el test upstream. |
| Q-REF-5 | Creación asíncrona de top-level (system.actorOf vía RepointableActorRef/Unstarte | 3/2 | Arquitectura | impl | (a) actorOf síncrono si se prueba equivalencia de orden observable; si no, (b). |
| Q-REF-6 | Visibilidad de accesores internos usados por tests y testkit (ActorRef.isTermina | 4/2 | Arquitectura | validación/cierre | (a) accesores internos visibles para specs y testkit. |

**RT**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-RT-1 | Routing en el hilo del emisor y Router inmutable: mensajes dinamicos y estado at | 5/1 | Arquitectura | impl | (a) routing en el hilo del emisor y Router inmutable; mensajes Box<dyn Any>. |
| Q-RT-2 | Mensajes de gestion del router y ciclo de vida del router actor | 3/1 | Representación | impl | (a) mismos nombres de mensajes de gestión. |
| Q-RT-3 | Router config via HOCON: type-mapping, FQCN custom, fromConfig por router | 3/1 | Arquitectura | impl | (a) misma sintaxis HOCON y claves (Q-X-1). |
| Q-RT-5 | ConsistentHashing: serializacion de claves arbitrarias y MurmurHash exacto | 3/1 | Divergencia funcional | validación/cierre | (a) MurmurHash; diferencia observable sólo para claves no String/bytes. |
| Q-RT-6 | Discrepancias doc/codigo en ScatterGather y TailChopping (excepcion y texto) | 2/1 | Divergencia funcional | validación/cierre | (a) replicar el código; el texto de TailChopping lo decide el Owner. |
| Q-RT-7 | BalancingPool: dispatcher de mailbox compartido (BalancingDispatcher) y specs pe | 1/1 | Arquitectura | impl | (a); depende de AC-DSP-012; validar con aserciones propias (specs upstream pending). |
| Q-RT-8 | Listeners requiere ActorRef ordenable (TreeSet) y semantica de duplicados | 2/2 | Representación | impl + validación | (a) paridad con TreeSet (orden e idempotencia). |
| Q-RT-9 | Resizer optimo: reloj y aleatoriedad inyectables, desempate en optimize y defect | 1/1 | Arquitectura | validación/cierre | (a) reloj y azar inyectables; desempates no reproducibles contra Pekko. |

**SCH**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-SCH-1 | Rueda de ticks (LightArrayRevolver) propia vs. timers del runtime | 8/1 | Arquitectura | nada | (a) rueda propia con Clock/Waiter inyectables (el redondeo al tick es observable). |
| Q-SCH-2 | Scheduler pluggable (scheduler.implementation) sin reflexión | 1/1 | Arquitectura | impl | Subordinada a Q-X-2: scheduler pluggable por nombre. |
| Q-SCH-3 | Mensajes de timer: tipo de mensaje interno y entrega al actor | 3/1 | Representación | impl | (a) sobre interno de timer interceptado antes de receive; depende de Q-X-3. |
| Q-SCH-4 | schedule(...) deprecado: mantener como alias de scheduleAtFixedRate | 2/1 | Representación | nada | (a) `schedule` deprecado como alias; (b) excluirlo exigiría autorización. |

**SER**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-SER-2 | Java serialization (JavaSerializer, DisabledJavaSerializer, java.io.Serializable | 4/1 | Exclusión de alcance | impl | (b) allow-java-serialization siempre off; `enabled` propuesto como exclusión. |
| Q-SER-3 | Información de transporte (thread-local) y serialización de ActorRef con address | 3/1 | Representación | impl | (b) info de transporte para serializadores nuevos + (a) interno por compatibilidad. |
| Q-SER-4 | Verificación serialize-messages/serialize-creators con mensajes dinámicos y Prop | 2/1 | Divergencia funcional | impl | (a) serialize-messages; (c) serialize-creators con `Props::with_serializable_args` opcional. |
| Q-SER-5 | Serializadores asíncronos y camino síncrono bloqueante con warning | 1/1 | Representación | nada | (a) camino síncrono bloqueante con warning; CompletionStage no se porta. |
| Q-SER-6 | Alcance de SER en esta fase: serialización local, claves solo-remote y detección | 3/1 | Exclusión de alcance | impl | (a) SER tras EXT; claves protobuf aceptadas sin efecto; ciclo de extensión detectado de inmediato. |

**SUP**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-SUP-1 | Modelo de errores para deciders: jerarquía Exception/Error/Throwable y tipos de  | 4/1 | Arquitectura | impl | (a); Q-X-5 decide si Panic mapea a Escalate o Restart (hay componente D). |
| Q-SUP-2 | Aritmética de la ventana de reinicios (withinTimeRange.toMillis.toInt y System.n | 3/1 | Divergencia funcional | impl | (a) saturar la ventana; (b) replicar el cast i32 (ventanas > 24,8 días envuelven). |

**SYS**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-SYS-1 | ¿Drop del último handle de ActorSystem termina el sistema? (close/AutoCloseable) | 1/1 | Arquitectura | validación/cierre | (a) terminate/close explícito; sin Drop implícito (+ guarda opcional en tests). |
| Q-SYS-2 | Mapeo de error fatal JVM (OOM/StackOverflow/Error) a Rust | 1/1 | Divergencia funcional | impl | (c) panic en hilos propios → ExitProcess; OOM/stack overflow no capturables. |
| Q-SYS-3 | Filas JVM-específicas: ManifestInfo (mixed versions) y Main | 2/1 | Exclusión de alcance | nada | (b)+(a) excluir sólo la lectura de MANIFEST; conservar función y mensaje. |
| Q-SYS-4 | Protocolo RegisterTerminationHook sin remote/cluster | 2/1 | Representación | impl + validación | (a) mantener RegisterTerminationHook; no excluir. |
| Q-SYS-5 | Redacción de datos sensibles en Settings.toString | 1/1 | Representación | nada | (a) redacción por variables de entorno/usuario. |
| Q-SYS-6 | Superficie JVM de ActorSystem: sobrecargas con ClassLoader, DynamicAccess y Thre | 3/1 | Exclusión de alcance | nada | (a) sin sobrecargas ClassLoader/DynamicAccess/ThreadFactory. |

**TK**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-TK-2 | TestActorRef/TestFSMRef y acceso a `underlyingActor` frente al modelo de propied | 2/1 | Arquitectura | impl | (a) guarda de acceso a `underlyingActor`. |
| Q-TK-3 | EventFilter[T] por tipo de causa en Rust | 2/1 | Representación | nada | (c) EventFilter por TypeId; NoCause = cualquier Error. |
| Q-TK-4 | API bloqueante vs asíncrona de TestKit (awaitCond, within, expectMsg, timePasses | 7/1 | Arquitectura | impl | (c) API de TestKit bloqueante y asíncrona; relacionada con Q-X-4. |
| Q-TK-5 | Origen del timefactor en Rust | 2/1 | Resuelta por investigación | nada | (a) timefactor sólo por clave de configuración. |
| Q-TK-6 | Harness de tests para sustituir PekkoSpec/Coroner/ScalaTest | 3/1 | Arquitectura | nada | (a) harness propio + (b) opcional. |
| Q-TK-7 | Interrupciones de hilo en la CallingThreadDispatcher (Thread.interrupt) no tiene | 1/1 | Exclusión de alcance | nada | (a) no portar 2 casos de Thread.interrupt de ActorModelSpec. |
| Q-TK-8 | Manejadores asíncronos ejecutados en el hilo del emisor (CallingThreadDispatcher | 3/1 | Arquitectura | impl | (a) manejadores async no corren sobre la CTD. |
| Q-TK-9 | Límites del Coroner y exclusión de metrics/* del testkit | 1/1 | Exclusión de alcance | nada | (a) excluir metrics/* y la parte JMX del Coroner (confirmación del Owner). |
| Q-TK-10 | Serializador de pruebas java-test (id 23) sin serialización Java | 1/1 | Divergencia funcional | nada | (a) serializador de prueba con bytes no interoperables con la JVM. |

**UTIL**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-UTIL-1 (resp.) | Negativos y rango de Timeout | 1/1 | Representación | nada | RESPONDIDA (rechazada): se conserva el dominio de Pekko (i64 ns con signo); ver sección 1. |
| Q-UTIL-2 | toString de Timeout conserva la unidad | 1/1 | Representación | validación/cierre | Ver grupo Q-X-9: (a) toString de Timeout (sólo logs); cierre de AC-UTIL-001. |
| Q-UTIL-3 | null y mensajes diferidos en PekkoException y requiring | 3/1 | Divergencia funcional | validación/cierre | Ver grupo Q-X-9: (a) sin `null` ni mensajes diferidos, igual que Q-PATH-4. |
| Q-UTIL-4 | Jerarquía de excepciones y OnlyCauseStackTrace | 2/1 | Exclusión de alcance | validación/cierre | Ver grupo Q-X-9: (i)(a) jerarquía de errores; (ii) excluir OnlyCauseStackTrace (requiere autorización, A-5). |
| Q-UTIL-5 | Done propio o () | 1/1 | Representación | validación/cierre | Ver grupo Q-X-9: `Done` propio vs `()`; el plan no registra recomendación. |
| Q-UTIL-7 | toRootLowerCase: tabla de minúsculas de Unicode distinta entre JDK y Rust | 1/1 | Divergencia funcional | nada | (a) tabla de minúsculas de Rust; los consumidores sólo reciben ASCII. |
| Q-UTIL-8 | Semántica de cadenas por unidades UTF-16 (Glob, makePattern, WildcardIndex) | 4/2 | Divergencia funcional | nada | (c) `char` de Rust con Glob como referencia; el Owner autoriza no reproducir makePattern bit a bit. |
| Q-UTIL-9 | PrettyDuration: `%.4g` de java.util.Formatter y representación de infinitos/inde | 1/1 | Resuelta por investigación | nada | (a) reimplementar `%.4g` de Java; resuelto por medición con oráculo. |
| Q-UTIL-10 | Estructuras concurrentes e identidad (Index, Switch, TokenBucket, colas, identit | 9/1 | Arquitectura | impl | (a) estructuras propias con tests 1:1; las colas bloqueantes (UTIL-012/013) esperan a Q-X-4. |
| Q-UTIL-13 | Helpers.isWindows deriva de `os.name` contiene "win" | 1/1 | Representación | nada | (a) isWindows por `os.name` como Pekko. |
| Q-UTIL-14 | Reproducir scala.util.hashing.MurmurHash3 y los hashCode de Scala/Java donde un  | 6/2 | Divergencia funcional | impl | (a) Murmur exacto sólo en FrequencySketch; (b) hash propio en el resto. |
| Q-UTIL-15 | MessageBufferMap.drop con un id ausente: NullPointerException de Pekko vs 0 mens | 1/1 | Divergencia funcional | nada | (b) `drop` de id ausente devuelve 0 en vez de NPE (defecto de Pekko; requiere autorización). |
| Q-UTIL-16 | Representación del texto de origen de closures de Typed (equivalente de LineNumb | 1/1 | Divergencia funcional | impl | (a) texto de origen de closures propio con fallback (b); el Debug de Behavior deja de coincidir. |

**T-ADP**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-ADP-2 | Que reglas de ActorAdapter sobreviven bajo runtime tipado nativo (Q-X-8 b) | 5/1 | Arquitectura | nada | (a) las 4 familias de reglas de ActorAdapter como lista verificable bajo Q-X-8 (b). |
| Q-T-ADP-3 | Semantica de rethrowTypedFailure y supervision Stop por defecto del tipado spawn | 3/1 | Representación | impl | (a) conservar Stop por defecto y rethrowTypedFailure. |

**T-ASK**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-ASK-1 | Taxonomia de errores de ask (TimeoutException vs error del destino) y conservaci | 2/1 | Representación | impl | (a) errores de ask idénticos (timeout y destino terminado). |
| Q-T-ASK-2 | Conservar o no la restriccion de ask a InternalRecipientRef | 1/1 | Divergencia funcional | nada | (a) permitir ask sobre cualquier RecipientRef (la restricción es accidental). |

**T-BEH**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-BEH-2 | Colapsar receivePartial/receiveMessagePartial/transformMessages en cierres con U | 4/1 | Representación | nada | (a) receivePartial/transformMessages colapsados en cierres con Unhandled. |
| Q-T-BEH-3 | Filtro por clase de interceptores (interceptMessageClass) y isSame | 7/1 | Representación | impl + validación | (a) isSame por TypeId y valor; monitor exige M: Clone. |
| Q-T-BEH-4 | narrow de Behavior y de ActorRef (contravarianza) y su identidad | 5/2 | Representación | impl | (a) Eq/Hash/Ord por path+uid en narrow (ActorRefImpl.scala:29-40). |
| Q-T-BEH-5 | AbstractBehavior en Rust: contexto por parametro y desaparicion de la clase de e | 3/3 | Divergencia funcional | impl | (a) AbstractBehavior con contexto por parámetro; desaparece IllegalStateException. |
| Q-T-BEH-6 | MDC y logMessages sobre backend de logging (tracing) en vez de MDC thread-local | 2/1 | Arquitectura | impl | (a) MDC vía tracing; claves pekkoSource... conservadas (junto con Q-X-7). |

**T-CTX**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-CTX-1 | messageAdapter: coincidencia por tipo exacto y comportamiento de watch/ask sobre | 2/1 | Divergencia funcional | impl | (a) messageAdapter por tipo exacto; sin match por subclase. |
| Q-T-CTX-2 | Chequeo de hilo del ActorContext: reemplazar ActorThreadSpec por garantias de co | 2/2 | Arquitectura | validación/cierre | (a) ActorThreadSpec pasa a garantías de compilación; tabla de 9 casos. |
| Q-T-CTX-3 | Errores de uso del contexto (IllegalArgument/IllegalState): Result vs panic | 4/1 | Representación | impl | (a) Result con los textos de error como Display. |
| Q-T-CTX-4 | Puente al contexto clasico (toClassic/actorOf) y su error cuando el contexto no  | 2/2 | Arquitectura | impl | Sigue a Q-X-8; (a) con capa clásica, devolviendo Result. |

**T-DLV**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-DLV-2 | Implementacion durable del DurableProducerQueue | 4/1 | Exclusión de alcance | impl | (b) cola en memoria equivalente a TestDurableProducerQueue; sin durabilidad real. |
| Q-T-DLV-3 | Equivalente del flight recorder JFR | 1/1 | Representación | nada | (a) eventos tracing en lugar de JFR (feature opcional). |
| Q-T-DLV-4 | Tiempo y semillas en la validacion de delivery (reloj real y aleatoriedad del or | 6/1 | Arquitectura | validación/cierre | (a) reloj y semillas inyectables. |

**T-DSP**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-DSP-1 | Orden de resolucion de Props: ultimo agregado gana | 2/1 | Representación | nada | (a) el último Props agregado gana. |
| Q-T-DSP-2 | Mapeo de DispatcherSelector a ejecutores de Rust (hilos vs tareas tokio) | 1/1 | Arquitectura | impl | Depende de Q-X-4: DispatcherSelector a ejecutores de Rust. |
| Q-T-DSP-3 | Semantica de mailbox acotado lleno (dead letters) y default single-consumer | 1/1 | Representación | impl | (a) mailbox acotado lleno envía a dead letters. |

**T-EVT**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-EVT-1 | Tres caminos distintos para publicar Dropped (classic eventStream, typed EventSt | 3/3 | Divergencia funcional | validación/cierre | (b) preservar los tres caminos de Dropped, o (a) unificar. |

**T-EXT**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-EXT-1 | Politica "skip" en extensions cuando la clase no resuelve | 1/1 | Resuelta por investigación | nada | (a) `extensions` = log+skip; `library-extensions` = fallo. |
| Q-T-EXT-2 | Identidad de ExtensionId por referencia en Rust | 2/1 | Representación | nada | (a) identidad del ExtensionId por puntero. |
| Q-T-EXT-3 | Relanzar la misma excepcion en cada acceso tras un fallo de creacion | 1/1 | Resuelta por investigación | nada | (a) error de creación cacheado y clonable. |

**T-JAPI**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-JAPI-2 | Default ante MessageAdaptionFailure y otras senales no manejadas en los builders | 5/1 | Divergencia funcional | impl | (a) MessageAdaptionFailure no manejada falla (coherente con scaladsl). |

**T-LOG**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-LOG-1 | Fachada de logging: tracing vs log+kv, y nombres de clave MDC | 9/3 | Representación | impl | (a) tracing con span por mensaje y campos pekko_*. |
| Q-T-LOG-2 | Deteccion del nombre del logger por stack (JVM-only) | 1/1 | Representación | nada | (a)+(c) nombre del logger por type_name/module_path. |
| Q-T-LOG-3 | pekko.use-slf4j / Slf4jLogger como configuracion en Rust | 1/1 | Exclusión de alcance | impl | (a) aceptar la clave use-slf4j sin Slf4jLogger; depende de Q-X-7. |
| Q-T-LOG-4 | LoggerOps (Scala 2) subsumido por macros de tracing | 1/1 | Exclusión de alcance | nada | (a) no portar LoggerOps (lo subsumen las macros de tracing). |

**T-RCP**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-RCP-1 | Igualdad de ServiceKey: (id, nombre de clase JVM) vs (id, TypeId de Rust) | 3/2 | Divergencia funcional | validación/cierre | (a) ServiceKey = (id, TypeId); el manifest del cable difiere. |
| Q-T-RCP-2 | Punto de extension del receptionist de cluster | 1/1 | Arquitectura | nada | (a) trait ReceptionistBehaviorProvider; cluster fuera (Q-X-10). |
| Q-T-RCP-3 | Deregister de clave no registrada con actor que tiene otras claves notifica List | 3/1 | Resuelta por investigación | nada | (a) Deregister/Register repetidos notifican (paridad). |

**T-RT**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-RT-1 | Hashing consistente byte-compatible con Pekko | 1/1 | Divergencia funcional | nada | (b) hashing consistente propio; el routee puede diferir del de Pekko. |
| Q-T-RT-2 | Comportamiento de selectRoutee sin routees (NPE/AIOOBE/IllegalArgument/IllegalSt | 3/1 | Resuelta por investigación | nada | (a) Option/Result uniforme; el único error contractual es el de hashing. |
| Q-T-RT-3 | Reuso del Behavior de routees y orden de entrega del broadcast | 4/1 | Arquitectura | impl | (a) Behavior: Clone en routees; depende de Q-T-BEH-1. |

**T-SER**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-SER-1 | Estrategia de serializacion de mensajes y refs en un runtime local | 9/3 | Arquitectura | impl + validación | (a) trait mínimo de serialización por tipo con id y manifest. |
| Q-T-SER-2 | Formato de ServiceKey en el cable (manifest = nombre de clase JVM) | 1/1 | Divergencia funcional | impl | (a) manifest = nombre de tipo Rust; no interoperable con la JVM. |
| Q-T-SER-3 | Serializacion Java de ActorRef (JVM-only) | 1/1 | Exclusión de alcance | impl | (b) si Q-T-SER-1 elige (a); Java serialization de ActorRef excluida. |

**T-SIG**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-SIG-1 | Forma de Signal: enum no exhaustivo, ChildFailed como Terminated, igualdad asime | 5/2 | Divergencia funcional | impl | (a) igualdad simétrica de Signal documentada como divergencia. |

**T-SPV**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-SPV-1 | Spawn[T] existencial y borrado de tipo | 1/1 | Arquitectura | nada | (b) Spawn tipado por llamada. |

**T-STASH**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-STASH-1 | Buffer interno de supervision que guarda mensajes y senales; UnstashException co | 3/2 | Representación | impl | (a) buffer de supervisión conserva el orden relativo de mensajes y señales. |

**T-SUP**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-SUP-1 | Selector de excepciones en supervise().on_failure: por tipo o por predicado | 2/1 | Representación | impl | (c) selector por tipo o predicado; depende de Q-X-5. |
| Q-T-SUP-2 | Discrepancia doc/codigo en backoff: 'los mensajes se descartan' vs stash con Dro | 2/1 | Resuelta por investigación | nada | Seguir código y spec (Supervision.scala:123,278-292); el Scaladoc es erróneo. |
| Q-T-SUP-4 | Setup que falla durante un restart ilimitado (bucle o StackOverflowError) | 1/1 | Divergencia funcional | impl | (a) reintento de setup sin tope en restart ilimitado; (b) sólo si el Owner lo acepta. |
| Q-T-SUP-5 | Resume con falla de arranque: envoltura accidental ActorInitializationException | 1/1 | Divergencia funcional | validación/cierre | (b) conservar texto y parada; la envoltura exacta es accidental. |
| Q-T-SUP-6 | ScheduledRestart y ResetRestartCount como senales opacas que llegan al Behavior  | 2/1 | Arquitectura | impl | (a) ScheduledRestart llega al Behavior como señal opaca (lo afirma el spec). |
| Q-T-SUP-7 | Padre/hijo clasico: decider clasico y excepcion agotada que sube al padre | 3/1 | Arquitectura | impl | Sigue a Q-X-8: decider clásico sólo con capa clásica. |
| Q-T-SUP-8 | Aleatoriedad e instante inyectables en backoff | 2/1 | Arquitectura | nada | (a) aleatoriedad e instante inyectables (con Q-T-TMR-4). |

**T-SYS**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-SYS-1 | Accesores JVM-only del sistema: threadFactory, dynamicAccess, printTree, logConf | 2/1 | Divergencia funcional | impl | (a) mantener en alcance con diferencia documentada (printTree/dynamicAccess/threadFactory). |
| Q-T-SYS-2 | Redaccion de configuracion en Settings.toString y logConfiguration | 2/1 | Representación | nada | (a) como mínimo; (b) opcional. |
| Q-T-SYS-3 | Buffer de arranque del guardian: limite de 1000 y StashOverflowException | 1/1 | Representación | validación/cierre | (a) buffer innecesario en diseño nativo (salvo StashOverflowException con > 1000). |
| Q-T-SYS-4 | close() bloqueante y Drop del ActorSystem | 1/1 | Divergencia funcional | impl | (a) close() explícito bloqueante; Drop no puede bloquear. |

**T-TK**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-TK-1 | API de TestProbe/ActorTestKit: bloqueante vs async | 10/1 | Arquitectura | impl + validación | (a) o (c): TestProbe no bloqueante; textos de fallo idénticos. |
| Q-T-TK-2 | Captura de logs (LoggingTestKit) sin Logback | 6/1 | Representación | impl | (a) Layer global de tracing + registro de filtros. |
| Q-T-TK-3 | Integraciones JUnit/ScalaTest y nombre del sistema por stack | 6/1 | Exclusión de alcance | nada | (a) fixtures RAII propios; JUnit/ScalaTest no se portan (exclusión no concedida). |
| Q-T-TK-4 | BehaviorTestKit requiere interprete de Behavior sin runtime | 19/1 | Arquitectura | impl | (a) intérprete puro de Behavior; depende de Q-X-8. |
| Q-T-TK-5 | cancelAll del TimerScheduler de BehaviorTestKit es un no-op (paridad bug-compati | 1/1 | Representación | nada | (a) cancelAll no-op por paridad documentada. |

**T-TMR**

| ID | Decisión | Filas/uds | Clase | Bloquea | Recomendación |
|---|---|---|---|---|---|
| Q-T-TMR-1 | Tipo de la clave del timer (Any en Scala) | 3/2 | Representación | impl | (a) claves de timer tipadas. |
| Q-T-TMR-2 | NotInfluenceReceiveTimeout: marker de clase vs flag explicito | 2/2 | Representación | nada | (a) flag explícito NotInfluenceReceiveTimeout. |
| Q-T-TMR-3 | startPeriodicTimer deprecado: alias y semantica (la sobrecarga Java difiere de l | 1/1 | Divergencia funcional | impl | (a) startPeriodicTimer FixedRate; la sobrecarga Java FixedDelay es un bug de Pekko. |
| Q-T-TMR-4 | Reloj y scheduler inyectables (ManualTime) para timers, backoff, limites y uptim | 5/3 | Arquitectura | validación/cierre | (a) reloj y scheduler inyectables (ManualTime). |

## 5. Resueltas por investigación, sin acción del Owner

Evidencia leída en `upstream/pekko` o medida con el oráculo según su ficha. Ninguna requiere decisión; las marcadas **registrar** piden dejar constancia (en `notes` del inventario o aceptando el registro) para que la diferencia con la documentación de Pekko no se pierda.

| ID | Resultado y evidencia | Filas |
|---|---|---|
| Q-CFG-12 | (a) replicar: `Deploy.withFallback` no copia `tags` (Deployer.scala:109-117); registrar confirmación en `notes`. | 1 |
| Q-DSP-4 | (a) MaxDispatcherAliasDepth = 20 y textos literales (Dispatchers.scala:77-87). | 2 |
| Q-DSP-5 | (a) throughput <= 0 equivale a 1 (Mailbox.scala:264); anotar la diferencia con el Scaladoc. | 2 |
| Q-EVT-6 | (a) replicar la asimetría de LogSource.fromAnyRef; unificar sería divergencia. | 1 |
| Q-MBX-4 | (b) sin orden entre iguales en el mailbox de prioridad; el estable lleva secuencia. | 2 |
| Q-T-EXT-1 | (a) `extensions` = log+skip; `library-extensions` = fallo. | 1 |
| Q-T-EXT-3 | (a) error de creación cacheado y clonable. | 1 |
| Q-T-RCP-3 | (a) Deregister/Register repetidos notifican (paridad). | 3 |
| Q-T-RT-2 | (a) Option/Result uniforme; el único error contractual es el de hashing. | 3 |
| Q-T-SUP-2 | Seguir código y spec (Supervision.scala:123,278-292); el Scaladoc es erróneo. | 2 |
| Q-TK-5 | (a) timefactor sólo por clave de configuración. | 2 |
| Q-UTIL-9 | (a) reimplementar `%.4g` de Java; resuelto por medición con oráculo. | 1 |

Constancia a registrar: Q-DSP-5 (diferencia con el Scaladoc, `Mailbox.scala:264`), Q-T-SUP-2 (la ficha pide que el Owner "ratifique" el registro; el Scaladoc `SupervisorStrategy.scala:64,100` es incorrecto), Q-CFG-12 (resuelta sólo parcialmente: si aparece un consumidor de `tags` hay que reabrirla), Q-EVT-6 y Q-TK-5 (si el Owner quisiera unificar o añadir una variable de entorno, pasaría a divergencia/extensión). No se presentaron al Owner como decisión: Q-DSP-4, Q-MBX-4, Q-T-EXT-1, Q-T-EXT-3, Q-T-RCP-3, Q-T-RT-2, Q-UTIL-9. Quedan abiertas pese a haber sido investigadas: Q-IO-5 (el defecto de 1.7.1 está confirmado; falta autorizar corregirlo), Q-UTIL-6 (asimetría confirmada; falta autorizar), Q-T-SUP-3 (`maxRestarts<=0` pendiente de oráculo), Q-SUP-2 (medido: el cast i32 envuelve; falta elegir a/b), Q-CELL-3 (sólo falta aprobación).

## 6. Método de medición (script de sólo lectura)

`python3 -I` desde la raíz del repositorio; no escribe nada. Expande `affected_rows` (listas separadas por `;`/`,`, herencia de prefijo `AC-UTIL-014, 015.2`, comodines `AC-SUP-*`, hijos `.n` de un padre descompuesto) contra los 783 ids de `actor-classic.tsv` + `actor-typed.tsv`; cuenta unidades por prefijo (`AC-SYS` ≠ `AT-SYS`); suma, para cada `Q-X-n`, las filas de las decisiones cuya ficha lo cita ("con dependientes"; es un piso porque sólo cuenta citas textuales); Q-X-9 es la unión de Q-UTIL-1..5. Ejecutado: 212 filas procesadas. Las columnas "Bloquea" y "Clase" no salen del script: son lectura de cada ficha (`Bloquea:`/`Clasificación:`).

```python
import csv, re
csv.field_size_limit(10**7)
def rd(p): return list(csv.DictReader(open(p, encoding='utf-8'), delimiter='\t', quoting=csv.QUOTE_NONE))
inv = rd('docs/inventory/actor-classic.tsv') + rd('docs/inventory/actor-typed.tsv')
ids = sorted(r['id'] for r in inv)
unit = lambda i: '-'.join(i.split('-')[:2])
reg = rd('docs/decisions/register.tsv')
answered = {r['id']: r['status'] for r in rd('docs/decisions/owner-decisions.tsv')}
def expand(spec):
    out, prefix = set(), ''
    for t in (x.strip() for x in re.split(r'[;,]', spec)):
        if t in ('(las filas de cada miembro)', 'todo lo que lee Config', ''): continue
        if t == 'todas las AT-*': t = 'AT-*'
        if re.fullmatch(r'\d+(\.\d+|\.\*)?', t): t = prefix + t
        m = re.match(r'(A[CT]-[A-Z]+-)', t)
        if m: prefix = m.group(1)
        if t.endswith('*'): out |= {i for i in ids if i.startswith(t[:-1])}
        else: out |= {i for i in ids if i == t or i.startswith(t + '.')}
    return out
own = {r['id']: expand(r['affected_rows']) for r in reg}
own['Q-X-9'] = set().union(*(own[f'Q-UTIL-{n}'] for n in range(1, 6)))
deps = {}
for r in reg:
    for n in set(re.findall(r'Q-X-(\d+)|Q-CFG-([12])\b', r['recommendation'] + ' ' + r['title'])):
        k = 'Q-X-' + (n[0] or n[1])
        if r['id'] != k and not r['id'].startswith('Q-X-'): deps.setdefault(k, set()).add(r['id'])
for r in sorted(reg, key=lambda r: (-len(own[r['id']]), r['id'])):
    d = deps.get(r['id'], set()); rows = set(own[r['id']])
    for q in d: rows |= own[q]
    print(r['id'], answered.get(r['id'], ''), len(own[r['id']]), len({unit(i) for i in own[r['id']]}),
          len(rows), len({unit(i) for i in rows}), len(d))
```

Top por impacto (filas propias → con dependientes): Q-X-8 260 → 265; Q-UTIL-11 34; Q-X-2 29 → 36; Q-X-7 28 → 39; Q-T-DLV-1 22; Q-T-TK-4 19; Q-T-JAPI-1 16; Q-IO-1 14; Q-JAPI-1 13; Q-CFG-4 12; Q-X-4 5 → 79; Q-X-3 5 → 31.

## 7. Exclusiones de alcance y divergencias funcionales (aceptación explícita)

Ninguna puede registrarse como "adaptación idiomática". Si no se autoriza una exclusión, la fila se implementa (AGENTS.md, A-4/A-5). Casos mixtos con componente funcional además de su clase principal: Q-SUP-1 y Q-X-5 (Error/Throwable vs panic), Q-T-SER-1, Q-UTIL-11 (la exclusión sólo si se elige (b)), Q-X-2 (pérdida de reflexión), Q-PATH-5, Q-SYS-4 y Q-SCH-4 (excluir sería la alternativa no recomendada).

| ID | Clase | Qué se pierde o cambia | Filas | Autorización |
|---|---|---|---|---|
| Q-ASK-10 | Divergencia funcional | (c) mensajes de require sin el prefijo "requirement failed: "; (a) si se quiere paridad textual. | 7 | explícita |
| Q-CFG-11 | Divergencia funcional | (a) aceptar las claves en `off` hasta que exista AC-SER; modos serialize-* diferidos (ver Q-SER-4). | 1 | explícita |
| Q-CS-6 | Divergencia funcional | (a) hook de señales; SIGINT soportado difiere de Pekko. | 2 | explícita |
| Q-CS-7 | Divergencia funcional | (a) un Display de Duration para los textos que afirman los specs; ISO-8601 Java fuera. | 5 | explícita |
| Q-DSP-6 | Divergencia funcional | (b) `minimum-runnable = -1` se comporta como JDK < 25; valor efectivo 1. | 2 | explícita |
| Q-DSP-7 | Divergencia funcional | (a) BatchingExecutor mínimo; A2-A4/A6/A7 de AC-DSP-016.2 dependen de blocking/Await. | 1 | explícita |
| Q-EVT-1 | Divergencia funcional | (b) nombres de tipos estándar que afirman los specs; (a) resto; función `message_type_name`. | 8 | explícita |
| Q-FSM-4 | Divergencia funcional | Bloqueante: (a) replicar terminate sin stop (FSM.scala:881-884); confirmar con oráculo. | 2 | explícita |
| Q-IO-4 | Divergencia funcional | (a) TTL e IPv6 sin propiedades JVM; la clave Ip() se corrige y se anota. | 4 | explícita |
| Q-IO-5 | Divergencia funcional | (b) corregir un defecto de 1.7.1 confirmado con el jar (Connect por nombre siempre falla). | 2 | explícita |
| Q-IO-6 | Divergencia funcional | (a) en Unix leer nameserver de /etc/resolv.conf (ampliación); Windows/macOS pendiente. | 3 | explícita |
| Q-JAPI-2 | Divergencia funcional | (a) TypeId exacto; (b) con supertipos si Q-X-6 los define; isInstance se documenta. | 5 | explícita |
| Q-JAPI-4 | Divergencia funcional | (a) si se porta la fachada; (c) si se acepta Q-JAPI-1 (a): quirks de la fachada no replicados. | 4 | explícita |
| Q-MBX-1 | Divergencia funcional | (c) push-timeout bloqueante sólo fuera de worker async; condicionada por Q-X-4. | 2 | explícita |
| Q-MBX-7 | Divergencia funcional | (a) resta de Int con desborde envolvente como Pekko; documentarlo. | 2 | explícita |
| Q-PATH-1 | Divergencia funcional | (a) comparador `pekko_compare`; no confiar en List.sort hasta tener consumidor (cluster). | 1 | explícita |
| Q-PATH-2 | Divergencia funcional | (a) posición UTF-16 en el mensaje; el surrogate suelto no existe en `String`. | 1 | explícita |
| Q-PATH-4 | Divergencia funcional | (a) sin `null`; `host = null` se modela como `None`. | 4 | explícita |
| Q-REF-2 | Divergencia funcional | (a)+(b) closures y registro por nombre; las formas reflexivas puras quedan como diferencia aceptada. | 4 | explícita |
| Q-RT-5 | Divergencia funcional | (a) MurmurHash; diferencia observable sólo para claves no String/bytes. | 3 | explícita |
| Q-RT-6 | Divergencia funcional | (a) replicar el código; el texto de TailChopping lo decide el Owner. | 2 | explícita |
| Q-SER-4 | Divergencia funcional | (a) serialize-messages; (c) serialize-creators con `Props::with_serializable_args` opcional. | 2 | explícita |
| Q-SUP-2 | Divergencia funcional | (a) saturar la ventana; (b) replicar el cast i32 (ventanas > 24,8 días envuelven). | 3 | explícita |
| Q-SYS-2 | Divergencia funcional | (c) panic en hilos propios → ExitProcess; OOM/stack overflow no capturables. | 1 | explícita |
| Q-T-ASK-2 | Divergencia funcional | (a) permitir ask sobre cualquier RecipientRef (la restricción es accidental). | 1 | explícita |
| Q-T-BEH-5 | Divergencia funcional | (a) AbstractBehavior con contexto por parámetro; desaparece IllegalStateException. | 3 | explícita |
| Q-T-CTX-1 | Divergencia funcional | (a) messageAdapter por tipo exacto; sin match por subclase. | 2 | explícita |
| Q-T-EVT-1 | Divergencia funcional | (b) preservar los tres caminos de Dropped, o (a) unificar. | 3 | explícita |
| Q-T-JAPI-2 | Divergencia funcional | (a) MessageAdaptionFailure no manejada falla (coherente con scaladsl). | 5 | explícita |
| Q-T-RCP-1 | Divergencia funcional | (a) ServiceKey = (id, TypeId); el manifest del cable difiere. | 3 | explícita |
| Q-T-RT-1 | Divergencia funcional | (b) hashing consistente propio; el routee puede diferir del de Pekko. | 1 | explícita |
| Q-T-SER-2 | Divergencia funcional | (a) manifest = nombre de tipo Rust; no interoperable con la JVM. | 1 | explícita |
| Q-T-SIG-1 | Divergencia funcional | (a) igualdad simétrica de Signal documentada como divergencia. | 5 | explícita |
| Q-T-SUP-3 | Divergencia funcional | Bloqueante: (a) default resetBackoffAfter = (min+max)/2; maxRestarts <= 0 se mide con el oráculo. | 2 | explícita |
| Q-T-SUP-4 | Divergencia funcional | (a) reintento de setup sin tope en restart ilimitado; (b) sólo si el Owner lo acepta. | 1 | explícita |
| Q-T-SUP-5 | Divergencia funcional | (b) conservar texto y parada; la envoltura exacta es accidental. | 1 | explícita |
| Q-T-SYS-1 | Divergencia funcional | (a) mantener en alcance con diferencia documentada (printTree/dynamicAccess/threadFactory). | 2 | explícita |
| Q-T-SYS-4 | Divergencia funcional | (a) close() explícito bloqueante; Drop no puede bloquear. | 1 | explícita |
| Q-T-TMR-3 | Divergencia funcional | (a) startPeriodicTimer FixedRate; la sobrecarga Java FixedDelay es un bug de Pekko. | 1 | explícita |
| Q-TK-10 | Divergencia funcional | (a) serializador de prueba con bytes no interoperables con la JVM. | 1 | explícita |
| Q-UTIL-14 | Divergencia funcional | (a) Murmur exacto sólo en FrequencySketch; (b) hash propio en el resto. | 6 | explícita |
| Q-UTIL-15 | Divergencia funcional | (b) `drop` de id ausente devuelve 0 en vez de NPE (defecto de Pekko; requiere autorización). | 1 | explícita |
| Q-UTIL-16 | Divergencia funcional | (a) texto de origen de closures propio con fallback (b); el Debug de Behavior deja de coincidir. | 1 | explícita |
| Q-UTIL-3 | Divergencia funcional | Ver grupo Q-X-9: (a) sin `null` ni mensajes diferidos, igual que Q-PATH-4. | 3 | explícita |
| Q-UTIL-6 | Divergencia funcional | Replicar la asimetría de Version.compareTo (Version.scala:172-177) hasta que decida el Owner. | 1 | explícita |
| Q-UTIL-7 | Divergencia funcional | (a) tabla de minúsculas de Rust; los consumidores sólo reciben ASCII. | 1 | explícita |
| Q-UTIL-8 | Divergencia funcional | (c) `char` de Rust con Glob como referencia; el Owner autoriza no reproducir makePattern bit a bit. | 4 | explícita |
| Q-ASK-8 | Exclusión de alcance | (b) excluir Backoff/BackoffOptions deprecados; BackoffOpts es el equivalente. | 1 | explícita (A-5) |
| Q-ASK-9 | Exclusión de alcance | (b) excluir las fachadas Java Patterns/PatternsCS. | 1 | explícita (A-5) |
| Q-CELL-2 | Exclusión de alcance | (a) excluir Thread.interrupt/ActorInterruptedException; exige autorización explícita. | 1 | explícita (A-5) |
| Q-CELL-5 | Exclusión de alcance | (a) modos serialize-* tras AC-SER; entre tanto `impl=pending`. | 2 | explícita (A-5) |
| Q-CFG-7 | Exclusión de alcance | (a) aceptar siempre las claves JVM (daemonic, exit-on-fatal, shutdown-hooks...) sin su efecto JVM. | 2 | explícita (A-5) |
| Q-DSP-1 | Exclusión de alcance | (b) equivalentes propios de fork-join/thread-pool/affinity-pool; virtual-thread sin efecto. | 7 | explícita (A-5) |
| Q-DSP-3 | Exclusión de alcance | (a) portar InlineExecutor y SerializedSuspendable; excluir AC-DSP-017.2 (futuros Java). | 4 | explícita (A-5) |
| Q-DW-2 | Exclusión de alcance | (a) mínimo: `is_local` y conjuntos watching/watchedBy sin remote. | 2 | explícita (A-5) |
| Q-IO-3 | Exclusión de alcance | (a) buffers/socket2/WritePath/DatagramSocketFactory; (b) excluir ByteBufferCleaner, workaround Windows, finish-connect-retries. | 9 | explícita (A-5) |
| Q-JAPI-1 | Exclusión de alcance | (a) excluir nombres/clases Java; la semántica la cubren las filas Rust equivalentes. | 13 | explícita (A-5) |
| Q-JAPI-3 | Exclusión de alcance | (a) excluir TypedActor clásico deprecado (proxies JDK); (b) paridad con invoke. | 3 | explícita (A-5) |
| Q-REF-3 | Exclusión de alcance | (a)+(b) sin Java serialization; error del resolver si no hay sistema en contexto. | 1 | explícita (A-5) |
| Q-SER-2 | Exclusión de alcance | (b) allow-java-serialization siempre off; `enabled` propuesto como exclusión. | 4 | explícita (A-5) |
| Q-SER-6 | Exclusión de alcance | (a) SER tras EXT; claves protobuf aceptadas sin efecto; ciclo de extensión detectado de inmediato. | 3 | explícita (A-5) |
| Q-SYS-3 | Exclusión de alcance | (b)+(a) excluir sólo la lectura de MANIFEST; conservar función y mensaje. | 2 | explícita (A-5) |
| Q-SYS-6 | Exclusión de alcance | (a) sin sobrecargas ClassLoader/DynamicAccess/ThreadFactory. | 3 | explícita (A-5) |
| Q-T-DLV-1 | Exclusión de alcance | (a) delivery local con cola en memoria + (b) cola durable de referencia. | 22 | explícita (A-5) |
| Q-T-DLV-2 | Exclusión de alcance | (b) cola en memoria equivalente a TestDurableProducerQueue; sin durabilidad real. | 4 | explícita (A-5) |
| Q-T-JAPI-1 | Exclusión de alcance | (a) no portar javadsl; API Rust única con las aserciones de scaladsl. | 16 | explícita (A-5) |
| Q-T-LOG-3 | Exclusión de alcance | (a) aceptar la clave use-slf4j sin Slf4jLogger; depende de Q-X-7. | 1 | explícita (A-5) |
| Q-T-LOG-4 | Exclusión de alcance | (a) no portar LoggerOps (lo subsumen las macros de tracing). | 1 | explícita (A-5) |
| Q-T-SER-3 | Exclusión de alcance | (b) si Q-T-SER-1 elige (a); Java serialization de ActorRef excluida. | 1 | explícita (A-5) |
| Q-T-TK-3 | Exclusión de alcance | (a) fixtures RAII propios; JUnit/ScalaTest no se portan (exclusión no concedida). | 6 | explícita (A-5) |
| Q-TK-7 | Exclusión de alcance | (a) no portar 2 casos de Thread.interrupt de ActorModelSpec. | 1 | explícita (A-5) |
| Q-TK-9 | Exclusión de alcance | (a) excluir metrics/* y la parte JMX del Coroner (confirmación del Owner). | 1 | explícita (A-5) |
| Q-UTIL-4 | Exclusión de alcance | Ver grupo Q-X-9: (i)(a) jerarquía de errores; (ii) excluir OnlyCauseStackTrace (requiere autorización, A-5). | 2 | explícita (A-5) |
| Q-X-10 | Exclusión de alcance | Límites del módulo: remote, cluster, streams, persistence, http | 2 | explícita (A-5) |

Las filas de clase "Exclusión de alcance" de las fichas por unidad (Q-CELL-2, Q-ASK-8/9, Q-JAPI-1/3, Q-T-JAPI-1, Q-T-TK-3, Q-TK-7/9, Q-SER-2, Q-IO-3, Q-X-10...) siguen en alcance con `impl=pending` hasta que el Owner decida.

## 8. Formato de respuesta esperado (`docs/decisions.md`, sección 2)

Por decisión o por bloque: `aceptar` (se adopta la recomendación tal cual), `aceptar con cambios: <texto>`, `rechazar: <alternativa>`. Respuesta global válida: `aceptar todas las recomendaciones excepto: <lista>`. Q-X-9 no admite respuesta propia: contestar Q-UTIL-2..5 por separado. Cada respuesta se registra en una fila de `docs/decisions/owner-decisions.tsv` (`id`, `status` = `accepted` | `accepted_with_changes` | `rejected`, `decided_on`, `decided_by`, `decision`); luego `python3 -I tools/package/generate.py` y `check.py`. Una respuesta no cierra las filas: faltan implementación y validación. Recomendación de mínimo: responder primero el bloque 1 del orden sugerido en la sección 2; con esas respuestas se desbloquean (por dependencia, con solapes) 265 filas AT (Q-X-8), 79 (Q-X-4), 36 (Q-X-2) y 39 (Q-X-7).

## 9. Observaciones (contradicciones o carencias; no se editaron los `.decisions.tsv`)

1. **Q-X-5 vs Q-SUP-1:** Q-X-5 (A) dice "tratados como Exception" (Restart); Q-SUP-1 prevé `Panic → Escalate` y Pekko escala los `Error` no fatales (`FaultHandling.scala:216-223, 309, 372`). Hay que decidir el mapeo, no sólo capturar. Cambia el contenido de la decisión.
2. **Cobertura de las transversales:** las fichas Q-X-* no traen "Bloquea/Puede avanzar" ni afectan filas completas (Q-X-3 lista 3 filas; Q-X-5 4); aquí se complementaron con dependientes por cita, un piso.
3. **Q-UTIL-1** conserva en `register.tsv` la recomendación antigua (a) pese a estar rechazada; vigente es la alternativa de la sección 1. Q-ASK-1, Q-CS-7 y Q-UTIL-9 deben releerse con Duration `i64` con signo.
4. **Fichas sin clasificación explícita** (clasificadas aquí por lectura): los 30 de classic-patterns (Q-ASK-*, Q-CS-*, Q-FSM-*, Q-RT-*), typed-extended en su mayoría y Q-UTIL-1/14/15/16. Las cifras de la sección 2 dependen de esa lectura.
5. **Q-T-SUP-3, Q-IO-5 y Q-UTIL-6** están resueltas en parte; la cita `Supervision.scala:278-292` de Q-T-SUP-2 no coincide con el archivo fijado (el `aroundReceive` está en 278-290 y `dropped` en 123): el sentido de la decisión no cambia.
6. El conteo de `docs/decisions.md` (211) coincide con este paquete; `register.tsv` tiene 212 filas porque incluye Q-X-9.
