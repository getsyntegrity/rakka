# Registro único de decisiones del Owner

Generado desde `docs/inventory/contracts/*.decisions.tsv` y las decisiones transversales por `tools/` (ver `docs/inventory/README.md`). Fuente de la tabla machine-readable: `docs/decisions/register.tsv`.

**Estado: 190 decisiones: 8 bloqueantes, 181 propuestas, 0 aceptadas, 0 aceptadas con cambios, 1 rechazadas.** Las respuestas del Owner (con fecha y autor) figuran en la sección 6 y se leen de `docs/decisions/owner-decisions.tsv`; una respuesta no cierra por sí sola las filas afectadas: eso exige implementación y validación.

## 1. Ya autorizado (instrucciones explícitas del Owner)

| ID | Autorización | Alcance |
|---|---|---|
| A-1 | Apache Pekko 1.7.1 es la fuente de verdad, fijada en `upstream/pekko` @ `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (= `v1.7.1^{}`, verificado con `git ls-remote` el 2026-10-30). | todo |
| A-2 | `develop` es la rama de integración; `main` sólo recibe PR de release; sin push directo ni force push; las ramas salen de `origin/develop`. | proceso |
| A-3 | TDD estricto RED → GREEN → REFACTOR con evidencia real; cobertura mínima 90 % de las líneas de producción nuevas o modificadas; nombres que expresan propósito. | todas las unidades |
| A-4 | Ninguna diferencia semántica se acepta sin decisión explícita del Owner; una pregunta abierta impide cerrar el comportamiento afectado. | todas las filas con `Q-` |
| A-5 | Ningún comportamiento se elimina por ser difícil de expresar en Rust; las exclusiones requieren autorización explícita. | todo el inventario |
| A-6 | Classic y Typed completos, con interoperabilidad y testkits necesarios, están en alcance. | inventario |

## 2. Cómo responder

Responder con una de estas formas por decisión (o por bloque): `aceptar` (se adopta la recomendación tal cual), `aceptar con cambios: <texto>`, `rechazar: <alternativa>`. Una respuesta global válida es `aceptar todas las recomendaciones excepto: <lista>`. Cada respuesta se registra en una fila de `docs/decisions/owner-decisions.tsv` (`id`, `status` = `accepted` | `accepted_with_changes` | `rejected`, `decided_on` AAAA-MM-DD, `decided_by`, `decision`); es la única fuente editable y este documento, `register.tsv` y los conteos se regeneran desde ella sin perder respuestas. Hasta entonces no hay aprobación.

## 3. Bloqueos reales

Impiden cerrar el comportamiento afectado aunque el resto de la unidad avance.

### Q-FSM-4 — Estado inexistente: terminate sin context.stop(self)

- **Estado:** bloqueante
- **Evidencia:** FSM.scala:882-884 (terminate sin stop) vs 869-878 (applyState hace stop)
- **Alternativas:** (a) replicar: el actor sigue vivo tras "Next state X does not exist"; (b) detenerlo tambien
- **Recomendación:** (a) replicar (paridad) y marcarlo como Q; confirmar contra el jar oficial con un escenario de oraculo antes de implementar.
- **Filas afectadas:** AC-FSM-006.1;AC-FSM-006.2
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-RT-4 — Introspeccion de mailbox para SmallestMailbox y Resizer (pressure)

- **Estado:** bloqueante
- **Evidencia:** SmallestMailbox.scala:108-153; Resizer.scala:206-226
- **Alternativas:** (a) exponer una vista interna de solo lectura del mailbox (is_scheduled, has_messages, number_of_messages, is_suspended, current_message) en AC-MBX; (b) heuristica aproximada
- **Recomendación:** (a) necesaria para paridad; marcar bloqueante si AC-MBX-019 no la ofrece.
- **Filas afectadas:** AC-RT-008;AC-RT-013.1;AC-RT-013.2;AC-RT-014
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-SER-1 — Resolución de bindings y manifests sin jerarquía de clases JVM

- **Estado:** bloqueante
- **Evidencia:** Serialization.scala:326-390,507-520 (isAssignableFrom, orden subtipo-primero); SerializeSpec.scala:191-243 (8 casos de jerarquía/interfaz); reference.conf:817-831 (claves FQCN JVM como "[B", "java.io.Serializable", "scala.Long"); Serialization.scala:241-258 (manifest = nombre de clase cargado por reflexión)
- **Alternativas:** (a) Registro de tipos con relación de supertipos declarada (marker traits registrados) y orden por especificidad; (b) solo coincidencia exacta de TypeId (sin jerarquía) y bindings por enum cerrado; (c) bindings por nombre lógico de mensaje (string estable) en lugar de FQCN
- **Recomendación:** (a) con (c) para manifests: cada tipo registrado declara su nombre lógico y supertipos; las claves FQCN de reference.conf se aceptan como alias de nombre lógico; reutilizar la solución de Q-X-6 (jerarquía en EventStream). Conserva el desempate y el aviso "Multiple serializers found". El typo "synch" de un mensaje (Serialization.scala:277) queda a elección: copiar o corregir
- **Filas afectadas:** AC-SER-001.1; AC-SER-001.2; AC-SER-001.3; AC-SER-002; AC-SER-003.2; AC-SER-003.3; AC-SER-003.5
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-T-BEH-1 — Forma del tipo Behavior en Rust, setup re-ejecutable y reutilizacion del valor

- **Estado:** bloqueante
- **Evidencia:** Pekko reutiliza el mismo valor Behavior en varios spawns y re-ejecuta setup en cada restart (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:331-339; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:404; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:198-211); sin eso supervise/restart no puede recrear el Behavior. Requisitos observables que cualquier representacion debe cumplir (contrato en AT-BEH-007, R1-R3): reutilizar el mismo Behavior en actores independientes y volver a ejecutar la fabrica de setup en cada restart (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:404; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1145-1153, 1673-1691). El FnOnce propuesto antes en AT-BEH-001/AT-BEH-007 contradice R1 y R2.
- **Alternativas:** (a) enum Behavior<M> cerrado con Deferred(Arc<dyn Fn(&mut Ctx)->Behavior>) y Behavior: Clone via Arc; (b) Box<dyn FnOnce> y prohibir restart de esos Behaviors (rompe AT-BEH-023 y AT-SUP-005); (c) trait object Behavior con metodo clone_box.
- **Recomendación:** (a). Es el unico que conserva restart y reutilizacion. Es una recomendacion: el Owner no la acepto; hasta que responda AT-BEH-001, AT-BEH-004 y AT-BEH-007 no se cierran y ninguna alternativa se considera adoptada.
- **Filas afectadas:** AT-BEH-001; AT-BEH-007; AT-BEH-011; AT-BEH-023; AT-SUP-005; AT-SUP-013
- **Unidades afectadas:** BEH; SUP
- **Origen:** typed-core

### Q-T-SUP-3 — Backoff: maxRestarts 0/negativo y default de resetBackoffAfter segun doc Java

- **Estado:** bloqueante
- **Evidencia:** Scaladoc de withMaxRestarts: 'Zero or negative disables the upper limit' (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:329-333) vs codigo que solo trata -1 como ilimitado (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:145-152, 335); Scaladoc Java de withResetBackoffAfter dice 'same value as minBackoff' (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:322-326) vs codigo y spec (min+max)/2 (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:82-86; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1117-1120).
- **Alternativas:** (a) seguir el codigo: solo -1 ilimitado, 0 y otros negativos fallan en la primera excepcion; resetBackoffAfter por defecto (min+max)/2; (b) seguir el Scaladoc: <=0 desactiva el limite.
- **Recomendación:** (a) para el default de resetBackoffAfter (lo afirma el spec); para maxRestarts <=0 pedir decision: no hay spec que afirme ninguna de las dos lecturas.
- **Filas afectadas:** AT-SUP-007.2; AT-SUP-007.3
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-TK-1 — Entrega síncrona del testActor (CallingThreadDispatcher) en TestKit/TestProbe

- **Estado:** bloqueante
- **Evidencia:** TestKit.scala:189-193 crea el testActor con CallingThreadDispatcher; TestProbeSpec.scala:32-39 hace `expectMsg(0.millis, ..)` inmediatamente tras `ask`/tell y afirma `future.isCompleted == true` tras responder.
- **Alternativas:** (a) dispatcher inline obligatorio para TestActor (misma semántica que Pekko); (b) entrega asíncrona con expectMsg que sondea y cambiar los specs portados; (c) ambos, con inline por defecto
- **Recomendación:** (a): sin ella no se portan 1:1 los specs que usan expectMsg(0.millis)
- **Filas afectadas:** AC-TK-001.1 ; AC-TK-002.1 ; AC-TK-006.2
- **Unidades afectadas:** TK ; DSP
- **Origen:** classic-system

### Q-UTIL-12 — Alcance y representación de ByteString/ByteIterator/ByteStringBuilder

- **Estado:** bloqueante
- **Evidencia:** scala-2.13/util/ByteString.scala:31,867,1164,1254 y ByteIterator.scala:26-430; ByteStringSpec.scala:360 (140 tests según inventario). Lo consumen IO (AC-IO-*) y serialización (AC-SER-*) que están en alcance.
- **Alternativas:** (a) newtype sobre `bytes::Bytes` (dependencia externa); (b) rope propio sin dependencias (el crate no tiene dependencias hoy); (c) ByteString = `Bytes` + traits para el builder/iterator.
- **Recomendación:** Decidir antes de empezar IO/SER; no bloquea UTIL/CFG/PATH/EXT.
- **Filas afectadas:** AC-UTIL-006.*, AC-UTIL-022
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-6 — Version.compareTo es asimétrico cuando un lado no tiene calificador (rest vacío) y el otro sí

- **Estado:** bloqueante
- **Evidencia:** util/Version.scala:170-175: `if (rest=="" && other.rest!="") diff=1` seguido de un `if (other.rest=="" && rest!="") diff=-1 else diff=rest.compareTo(other.rest)` independiente; con rest="" y other.rest="M1" el `else` sobrescribe a 1 con "".compareTo("M1") < 0. VersionSpec.scala:56-64 sólo prueba `1.2-M1 < 1.2.0` (lado con calificador a la izquierda). Sin ejecutar con el jar: hipótesis a confirmar.
- **Alternativas:** (a) reproducir exactamente (bug-compatible) y fijarlo con un test del jar; (b) corregir para que sea antisimétrico (1.2.0 > 1.2-M1) y registrar la diferencia; (c) no portar Version.compareTo y usar semver.
- **Recomendación:** Confirmar primero con el jar (escenario nuevo en tools/oracle). Si se confirma, (a) hasta que el Owner decida: cualquier diferencia no autorizada es regresión (AGENTS.md).
- **Filas afectadas:** AC-UTIL-005.2
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

## 4. Decisiones transversales (afectan a varias unidades)

Decidirlas primero: desbloquean la mayor parte de las propuestas por unidad.

### Q-X-1 — Implementación de HOCON (parser y carga)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Plan docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md: golden de 161 casos contra Typesafe Config 1.4.9; `hocon-parser` 1.14.2 diverge en `+=` entre capas, duraciones negativas, `"ON"` e `include classpath()`; `hocon-rs` 0.2.0 no tiene `withFallback`; `hocon` 0.9.0 sin mantenimiento desde 2022.
- **Alternativas:** (A) subconjunto propio sin dependencias; (B) `hocon-parser` tras fachada con una `Q:` por divergencia; (C) (B) más parche upstream.
- **Recomendación:** (A), con el golden como suite de aceptación y ampliado antes de implementar. Alias: Q-CFG-1.
- **Filas afectadas:** AC-CFG-001; AC-CFG-017.*; todo lo que lee Config
- **Unidades afectadas:** AC-CFG, AC-SER (WireConfig), todas las que leen reference.conf
- **Origen:** transversal

### Q-X-2 — Registro nombre → fábrica en lugar de reflexión por FQCN

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AC-CFG-016, AC-EXT-003, DynamicAccessSpec.scala:40,50,59,72; el inventario ya propone el registro; `inventory`/`linkme` descartado como mecanismo principal (dependen del linker).
- **Alternativas:** (A) `FactoryRegistry<Producto>` por punto de extensión; (B) un registro único con `Any`; (C) macros de registro global.
- **Recomendación:** (A); un nombre no registrado devuelve un error de configuración que lleva el nombre. Abierto dentro de la decisión: tipo del error (`ConfigurationException` o propio). Alias: Q-CFG-2.
- **Filas afectadas:** AC-CFG-016; AC-EXT-003; AC-MBX-004; AC-DSP-010; AC-EVT-011; AC-SCH-010; AC-SER-*
- **Unidades afectadas:** AC-CFG, AC-EXT, AC-MBX, AC-DSP, AC-EVT, AC-SCH, AC-SER
- **Origen:** transversal

### Q-X-3 — Tipo de los mensajes en Classic

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-classic.md pregunta 2; Classic es dinámico (`Any` + `PartialFunction`); `unhandled`, `become` y `UnhandledMessage` dependen de poder rechazar un mensaje.
- **Alternativas:** (A) `Box<dyn Any + Send>` y `receive` que hace downcast y devuelve Handled/Unhandled; (B) enum por actor.
- **Recomendación:** (A); el tipado fuerte queda en `rakka-actor-typed`.
- **Filas afectadas:** AC-REF-002; AC-CELL-001; AC-ASK-001
- **Unidades afectadas:** AC-REF, AC-CELL, AC-ASK, AC-FSM, AC-RT
- **Origen:** transversal

### Q-X-4 — Runtime y modelo de ejecución

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-classic.md pregunta 3; `BoundedMailbox` bloquea al emisor (no encaja en async); claves de `fork-join-executor`; rueda de timers `LightArrayRevolverScheduler`.
- **Alternativas:** (A) pools de hilos propios con interfaz de ejecutor y futuros agnósticos del runtime; (B) atar el port a tokio.
- **Recomendación:** (A): preserva el bloqueo del emisor, `PinnedDispatcher` y las claves de configuración; tokio puede ser un adaptador opcional más adelante.
- **Filas afectadas:** AC-DSP-008; AC-MBX-007; AC-SCH-008
- **Unidades afectadas:** AC-DSP, AC-MBX, AC-SCH, AC-CELL, AC-ASK
- **Origen:** transversal

### Q-X-5 — Qué es un fallo supervisable

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-classic.md pregunta 1; Pekko distingue Exception (reinicia) de Error/Throwable (escala).
- **Alternativas:** (A) `Err` y `panic` son fallos supervisables, tratados como Exception; (B) `panic` escala; (C) no capturar panics.
- **Recomendación:** (A): capturar panics preserva el aislamiento entre actores; (C) lo rompe.
- **Filas afectadas:** AC-SUP-001; AC-SUP-004; AC-SYS-015; AT-SUP-002
- **Unidades afectadas:** AC-SUP, AC-CELL, AT-SUP
- **Origen:** transversal

### Q-X-6 — Suscripción por jerarquía de tipos en EventStream

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-classic.md pregunta 5; Rust no tiene herencia; AC-UTIL-008, AC-EVT-003.
- **Alternativas:** (A) `TypeId` exacto más canales padre declarados por el tipo de evento; (B) enum cerrado de eventos.
- **Recomendación:** (A).
- **Filas afectadas:** AC-UTIL-008; AC-EVT-003
- **Unidades afectadas:** AC-EVT, AC-UTIL, AT-EVT
- **Origen:** transversal

### Q-X-7 — Loggers como actores o backend directo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-classic.md pregunta 8; `LoggerSpec` y `EventFilter` dependen del handshake InitializeLogger/LoggerInitialized (AC-EVT-011).
- **Alternativas:** (A) loggers como actores con el handshake; (B) backend directo a `tracing`.
- **Recomendación:** (A): permite portar `LoggerSpec` y `EventFilter` sin cambios; `tracing` puede ser un logger más.
- **Filas afectadas:** AC-EVT-011; AC-TK-*
- **Unidades afectadas:** AC-EVT, AC-TK, AT-LOG
- **Origen:** transversal

### Q-X-8 — Typed sobre Classic o runtime tipado nativo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** docs/implementation-order.md (sección Decisión previa); `ActorAdapter`, `ActorContextAdapter`, `ActorSystemAdapter` en actor-typed/.../internal/adapter/.
- **Alternativas:** (a) replicar las capas de Pekko; (b) runtime tipado nativo con interop aparte.
- **Recomendación:** (a): cada contrato heredado se implementa y valida una sola vez y la interop sale casi gratis.
- **Filas afectadas:** AT-SYS-011; AT-ADP-*; todas las AT-*
- **Unidades afectadas:** todas las AT-*
- **Origen:** transversal

### Q-X-9 — Agrupación de presentación: Done, Timeout, null y unidades de duración (Q-UTIL-1..5)

- **Estado:** agrupación de presentación (se responde por miembro, nunca en bloque)
- **Miembros (decisiones independientes):** Q-UTIL-1, Q-UTIL-2, Q-UTIL-3, Q-UTIL-4, Q-UTIL-5
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md, sección Preguntas abiertas. Es sólo un encabezado: no tiene estado ni respuesta propios.
- **Alternativas:** No aplica: cada miembro tiene sus propias alternativas.
- **Recomendación:** Sin recomendación propia. Responder cada miembro por separado (Q-UTIL-1, Q-UTIL-2, Q-UTIL-3, Q-UTIL-4, Q-UTIL-5); una respuesta a Q-X-9 se rechaza y ninguna respuesta de un miembro aprueba ni rechaza a los demás.
- **Filas afectadas:** (las filas de cada miembro)
- **Unidades afectadas:** AC-UTIL
- **Origen:** transversal

### Q-X-10 — Límites del módulo: remote, cluster, streams, persistence, http

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-classic.md sección Límites del módulo; ninguno vive en `pekko-actor`/`pekko-actor-typed`; hay ganchos (AC-SYS-018, AC-DW-010/012, AC-SER-007).
- **Alternativas:** (A) fuera del alcance; el trait `ActorRefProvider` se deja abierto; (B) incluir alguno.
- **Recomendación:** (A). Es el límite con que se armó el inventario; no fue autorizado explícitamente.
- **Filas afectadas:** AC-SYS-018; AC-CFG-008
- **Unidades afectadas:** AC-SYS, AC-CFG, AC-DW, AC-SER
- **Origen:** transversal

Alias: `Q-CFG-1` = `Q-X-1`, `Q-CFG-2` = `Q-X-2`.

## 5. Propuestas por unidad

### Q-ASK-1 — Modelo de Future de ask/gracefulStop/after/retry y tipado del resultado

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AskSupport.scala:372-388 devuelve Future[Any]; mapTo en el llamador
- **Alternativas:** (a) async fn tipada con Result<R, AskError> sobre tokio; (b) Future<Output=Box<dyn Any>> sin tipado
- **Recomendación:** (a); mensajes de error identicos a Pekko donde el spec los fija (AskSpec:66-69,77,87). Depende de Q-X-3/Q-X-4/Q-X-9.
- **Filas afectadas:** AC-ASK-001.1;AC-ASK-001.2;AC-ASK-001.3;AC-ASK-003;AC-ASK-004;AC-ASK-007;AC-ASK-008;AC-ASK-010
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-2 — Ref temporal: path lazy, estados atomicos y texto de error de PromiseRef

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AskSupport.scala:515-700; PromiseRef.scala:166-172 (condicion length>0, texto "must not be negative")
- **Alternativas:** (a) replicar maquina de estados y registro lazy de path; (b) path eager; texto de error: replicar o corregir
- **Recomendación:** (a) replicar; mantener el texto exacto de Pekko (discrepancia condicion/texto documentada).
- **Filas afectadas:** AC-ASK-002;AC-ASK-010
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-3 — CircuitBreaker: mapeo de tiempos, ejecutor de listeners y semantica de saturacion de resetTimeout

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreaker.scala:723-775 (listeners via executor.execute); 1105-1118 (no satura a maxResetTimeout)
- **Alternativas:** (a) replicar exacto (incluida la no saturacion); (b) saturar a maxResetTimeout
- **Recomendación:** (a) replicar exacto; (b) solo con autorizacion del Owner.
- **Filas afectadas:** AC-ASK-011.1;AC-ASK-011.3;AC-ASK-011.4;AC-ASK-013
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-4 — CircuitBreaker: callTimeout=Duration.Zero ignora defineFailureFn; allowlist por FQCN de excepcion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreaker.scala:863-875 (rama sin defineFailureFn) vs 877-907; allowlist por getClass.getName (802-806)
- **Alternativas:** (a) replicar el comportamiento incluyendo la rama ignorando defineFailureFn; (b) aplicar defineFailureFn tambien con timeout 0. Allowlist: nombre de tipo registrado vs TypeId
- **Recomendación:** (a) replicar (paridad) y registrar la diferencia; allowlist por nombre de tipo en un registro explicito.
- **Filas afectadas:** AC-ASK-011.2;AC-ASK-012
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-5 — API sincrona de CircuitBreaker (withSyncCircuitBreaker bloquea con Await)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreaker.scala:436-458,466-480
- **Alternativas:** (a) exponer fn bloqueante (block_on) documentada; (b) proponer exclusion de la variante sync
- **Recomendación:** (a) exponer; prohibido excluir sin autorizacion; marcar la diferencia: bloquea el hilo del runtime.
- **Filas afectadas:** AC-ASK-011.4
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-6 — CircuitBreakerTelemetry: carga por reflexion (FQCN) y clave de config ausente de reference.conf

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreakerTelemetry.scala (provider); grep reference.conf sin `telemetry`
- **Alternativas:** (a) registro de fabricas por nombre (Q-X-2) leyendo la misma clave; (b) solo API programatica
- **Recomendación:** (a) misma clave `pekko.circuit-breaker.telemetry.implementations`; el trait es @InternalStableApi: exponerlo como API estable para extensores.
- **Filas afectadas:** AC-ASK-014
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-7 — BackoffSupervisor: composicion de strategy y directivas (Restart traducido a Stop)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** BackoffOnRestartSupervisor.scala:57-96; BackoffOnStopSupervisor.scala:57-70
- **Alternativas:** (a) replicar el decider compuesto sobre el modelo de supervision de rakka; (b) simplificar a estrategia fija
- **Recomendación:** (a); depende de Q-X-5 (que es un fallo) y del modelo de supervision (AC-SUP-001).
- **Filas afectadas:** AC-ASK-015.1;AC-ASK-015.2;AC-ASK-015.3;AC-ASK-015.4;AC-ASK-015.5
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-8 — API deprecada Backoff / BackoffOptions: propuesta de no portar

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Backoff.scala deprecated since Akka 2.5.22 (BackoffSupervisor.scala:175,220,236)
- **Alternativas:** (a) portarla; (b) proponer exclusion (solo el Owner puede autorizarla)
- **Recomendación:** (b) proponer exclusion con BackoffOpts como equivalente; la fila sigue en alcance con impl=pending hasta decision.
- **Filas afectadas:** AC-ASK-016
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-9 — Fachadas Java Patterns/PatternsCS: propuesta de no portar

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Patterns.scala:31-1333 (delegacion pura)
- **Alternativas:** (a) portar como modulo de fachada; (b) proponer exclusion
- **Recomendación:** (b) proponer exclusion (API Rust unica); la fila sigue en alcance con impl=pending hasta decision.
- **Filas afectadas:** AC-ASK-017
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-CELL-1 — Forma de receive (PartialFunction) y de la pila become/unbecome

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:384-410,545-556 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:602-615
- **Alternativas:** (a) fn(&mut self, &mut Context, msg) -> Handled y pila Vec<Box<dyn FnMut>> / (b) trait Behavior con enum de resultado / (c) macro receive!
- **Recomendación:** (a) con Handled::{Handled,Unhandled} para disparar unhandled(); la pila se limpia en restart y el receive original se re-evalúa tras unbecome con pila vacía
- **Filas afectadas:** AC-CELL-001 ; AC-CELL-005
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-2 — Interrupción de hilo (InterruptedException → ActorInterruptedException)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:226-234,353-359 ; actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:422-477
- **Alternativas:** (a) excluir; mapear la cancelación cooperativa de la tarea a un fallo normal no recuperable / (b) emular con un token de cancelación que convierte en ActorInterruptedError / (c) ignorar
- **Recomendación:** (a) como propuesta de exclusión; el Owner debe autorizarla explícitamente; los tests de interrupción no se portan
- **Filas afectadas:** AC-CELL-021
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-3 — Uso del contexto/ActorRef::self tras terminar el actor

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:193-230 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:247-276
- **Alternativas:** (a) las operaciones sobre el contexto capturado devuelven Err(Terminated) o se ignoran con dead letters / (b) prohibir por lifetime (no capturable)
- **Recomendación:** (b) donde el borrow checker lo garantice y (a) para refs clonadas; el contexto de Pekko no se anula tras terminar
- **Filas afectadas:** AC-CELL-019
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-4 — Texto de los eventos Debug que incluyen toString de actor/envelope/ref ("started (<actor>)", "received AutoReceiveMessage <envelope>")

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:569-570,655-656 ; actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:196-199,272-277
- **Alternativas:** (a) usar el nombre de tipo Rust y Debug del envelope / (b) formato Scala exacto "Envelope(msg,sender)" / (c) campo estructurado
- **Recomendación:** (b) para el prefijo fijo ("started (", "received AutoReceiveMessage Envelope(") y (a) para el resto; documentar la diferencia
- **Filas afectadas:** AC-CELL-023
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-5 — Modos de test serialize-messages y serialize-creators (ida y vuelta de mensajes y args de Props)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:171-225 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:279-306 ; actor/src/main/resources/reference.conf:129-140
- **Alternativas:** (a) implementar como hook opcional detrás de feature de test cuando exista la unidad de serialización / (b) omitir / (c) comprobación en tiempo de compilación (Serialize bound)
- **Recomendación:** (a), depende de AC-SER y de Q-X-3; hasta entonces las filas quedan en alcance con impl=pending
- **Filas afectadas:** AC-CELL-028 ; AC-CELL-029
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CFG-3 — checkValid contra reference.conf (Typesafe Config) con un parser propio

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:395-402: `cfg.checkValid(defaultReference.withoutPath("pekko.actor.internal-dispatcher"), "pekko")`. Las reglas (compatibilidad de tipos, listas, objetos, mensaje agregado `ValidationFailed`) pertenecen a Typesafe Config 1.4.9, que no está en upstream/pekko; ningún test de Pekko las afirma.
- **Alternativas:** (a) implementar checkValid con tabla de compatibilidad documentada y escenarios del jar; (b) omitirlo y fallar al leer cada clave; (c) reducirlo a presencia de claves.
- **Recomendación:** (a) si Q-X-1 elige parser propio; con (b)/(c) se pierde 'fail-fast con lista de errores'.
- **Filas afectadas:** AC-CFG-019, AC-CFG-002
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-4 — Fidelidad de errores y render de Config (tipos y textos de ConfigException, renderWithRedactions)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** DeployerSpec.scala:144-161 exige ConfigException.WrongType; el golden config-hocon.golden fija mensajes (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR); Helpers.scala:178-192 renderiza con ConfigRenderOptions.defaults (formato propio de Typesafe).
- **Alternativas:** (a) enum de errores con las mismas categorías y mensajes del golden; (b) mismas categorías, mensajes propios; (c) textos exactos sólo donde un test de Pekko los afirme.
- **Recomendación:** (a) para los mensajes del golden; render sin exigir igualdad textual.
- **Filas afectadas:** AC-CFG-002, 005, 012.1, 017.*, AC-UTIL-002.4, AC-UTIL-002.5
- **Unidades afectadas:** CFG, UTIL
- **Origen:** classic-foundation

### Q-CFG-5 — Valor de `pekko.version` y de `ActorSystem.Version` en rakka

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:10-11 (`include "version"`), project/VersionGenerator.scala:18-25 (genera version.conf y Version.scala), ActorSystem.scala:485-487 (error si difieren), ConfigSpec.scala:41-42.
- **Alternativas:** (a) la versión de Pekko portada ("1.7.1"); (b) la versión del crate (CARGO_PKG_VERSION); (c) ambas: `pekko.version` fijo en 1.7.1 y una constante aparte del crate.
- **Recomendación:** (a) mientras la fuente de verdad sea Pekko 1.7.1; así `PekkoVersion.require` de librerías se comporta igual.
- **Filas afectadas:** AC-CFG-003, AC-UTIL-005.1
- **Unidades afectadas:** CFG, UTIL
- **Origen:** classic-foundation

### Q-CFG-6 — Fuentes de la config de aplicación cuando no se pasa Config

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:281 y docs/src/main/paradox/general/configuration.md:18-53: `ConfigFactory.load(cl)` = application.conf/json/properties del classpath + system properties + `config.resource`; reference.conf de cada jar.
- **Alternativas:** (a) application.conf del directorio de trabajo + variables de entorno con prefijo + reference.conf embebidos de las crates; (b) sólo reference.conf embebido y config explícita; (c) además un equivalente a `-Dkey=value` por línea de comandos.
- **Recomendación:** (a) con una ruta configurable; documentar que no hay classpath scanning.
- **Filas afectadas:** AC-CFG-001
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-7 — Claves de JVM: qué semántica tienen en Rust (daemonic, jvm-exit-on-fatal-error, jvm-shutdown-hooks, fail-mixed-versions, java-flight-recorder.enabled, home)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:14-15,85-110; ActorSystem.scala:463-472; CoordinatedShutdownSpec.scala:632-645 (conteo de hooks).
- **Alternativas:** (a) aceptar todas; mapear daemonic a hilos del runtime, jvm-exit a abort/exit, jvm-shutdown-hooks a manejadores de señales; ignorar con aviso fail-mixed-versions y java-flight-recorder; (b) rechazar las claves sin sentido.
- **Recomendación:** (a); la clave siempre se acepta para no romper configs de Pekko.
- **Filas afectadas:** AC-CFG-007, AC-UTIL-016.11
- **Unidades afectadas:** CFG, UTIL
- **Origen:** classic-foundation

### Q-CFG-8 — Nombres estables para el registro de extensiones/fábricas (FQCN con `$` de objeto Scala)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:78 usa `org.apache.pekko.serialization.SerializationExtension$`; ReflectiveDynamicAccess.scala:69-87 resuelve `fqcn$` y luego `fqcn`; ExtensionSpec.scala:143 lista `InstanceCountingExtension` con y sin `$`.
- **Alternativas:** (a) registrar ambos nombres (con y sin `$`) para cada extensión; (b) normalizar quitando `$` al resolver; (c) claves propias de Rust y tabla de alias de los FQCN de Pekko.
- **Recomendación:** (b): mantiene el reference.conf sin cambios y los duplicados siguen creando una sola instancia.
- **Filas afectadas:** AC-CFG-006, AC-EXT-003
- **Unidades afectadas:** CFG, EXT
- **Origen:** classic-foundation

### Q-CFG-9 — Proveedor distinto de local (remote, cluster, FQCN) en un sistema solo-local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:107-145 y 410-420; remote/cluster están fuera del módulo; ProviderSelectionSpec.scala:38-67 sólo prueba la selección, no la carga de la clase.
- **Alternativas:** (a) `ProviderSelection` completo, pero crear el sistema con Remote/Cluster/Custom no registrado devuelve un error de configuración que nombra la clase; (b) rechazar `remote`/`cluster` al parsear Settings; (c) permitir proveedores registrados por el usuario (punto de extensión para remote).
- **Recomendación:** (a) + (c): deja la puerta abierta a remote sin implementarlo.
- **Filas afectadas:** AC-CFG-008, AC-SYS-018
- **Unidades afectadas:** CFG, SYS
- **Origen:** classic-foundation

### Q-CFG-10 — pekko.use-slf4j / amendSlf4jConfig

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:343-376, actor-typed reference.conf:68, ConfigSpec.scala:186-203. Depende de si los loggers son actores o un backend (Q-X-7).
- **Alternativas:** (a) no portar la función si Q-X-7 elige un backend de logging; (b) portarla con un 'logger' registrado como Slf4jLogger equivalente.
- **Recomendación:** Sigue a Q-X-7.
- **Filas afectadas:** AC-CFG-004, AC-CFG-018
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-11 — serialize-messages / serialize-creators / no-serialization-verification-needed-class-prefix

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatch.scala:171-223, Children.scala:283-307, SerializeSpec.scala:310-361, DispatchSpec.scala:29-41; en Rust los mensajes son `Box<dyn Any + Send>` (Q-X-3).
- **Alternativas:** (a) portar el chequeo sólo para tipos con serializador registrado; (b) no portar los dos modos y rechazar las claves en `on`; (c) portar con un trait `Verifiable` opt-in.
- **Recomendación:** (a) cuando exista AC-SER; mientras tanto las claves se aceptan en `off`.
- **Filas afectadas:** AC-CFG-010
- **Unidades afectadas:** CFG, SER
- **Origen:** classic-foundation

### Q-CFG-12 — Deploy.withFallback descarta los tags de `this`

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Deployer.scala:109-117 crea el Deploy resultante con `Deploy(path, config..., mailbox)` sin pasar `tags`; withTags existe aparte (línea 119-120). Ningún test lo afirma.
- **Alternativas:** (a) reproducir (los tags no sobreviven al merge); (b) conservar los tags de `this` y registrar la diferencia.
- **Recomendación:** (a) hasta que un test o consumidor de tags (cluster) diga lo contrario.
- **Filas afectadas:** AC-CFG-013
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-13 — ActorSystemSetup: identidad por clase concreta del Setup

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystemSetup.scala:48-49,76-87 indexa por `getClass`; ActorSystemSetupSpec.scala:39-55.
- **Alternativas:** (a) `HashMap<TypeId, Arc<dyn Any + Send + Sync>>` con `TypeId` del tipo concreto; (b) trait `Setup` con `fn key() -> &'static str`.
- **Recomendación:** (a).
- **Filas afectadas:** AC-CFG-015
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CS-1 — Fases: errores de config/validacion como Result y orden entre fases independientes

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:329-330,347-350,566-572; test solo comprueba tipo IllegalArgumentException
- **Alternativas:** (a) Result::Err con mismos mensajes; (b) panic. Orden de ramas independientes: arbitrario vs determinista
- **Recomendación:** (a) Result con mensajes de Pekko; orden determinista (BTreeSet) documentado como diferencia (Pekko usa Set sin orden).
- **Filas afectadas:** AC-CS-001.1;AC-CS-001.2;AC-CS-003.1
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-2 — Modelo de tareas de apagado: identidad por objeto, cancelacion, paralelismo y Future compartido

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:405-533,722-785
- **Alternativas:** (a) replicar el modelo (tarea = Arc con estado atomico, Future compartido); (b) simplificar a lista de tareas
- **Recomendación:** (a); depende de Q-X-4 (runtime) y Q-X-9 (Done).
- **Filas afectadas:** AC-CS-003.2;AC-CS-004.1;AC-CS-004.2;AC-CS-004.3;AC-CS-008;AC-CS-009
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-3 — Fases de cluster predefinidas sin cluster (vacias en local)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:1346-1397; CoordinatedShutdownSpec:123-139
- **Alternativas:** (a) mantener todas las fases (paridad de orden y nombres); (b) proponer quitar las de cluster (no autorizado)
- **Recomendación:** (a): el spec afirma la lista completa; cluster queda fuera del modulo pero las fases se mantienen.
- **Filas afectadas:** AC-CS-002
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-4 — Identificador estable de Reason y claves reason-overrides

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:1282-1291 (clave = nombre de clase JVM); CoordinatedShutdown.scala:206-213
- **Alternativas:** (a) cada Reason expone una cadena estable igual al nombre de clase JVM de Pekko; (b) nueva convencion de claves
- **Recomendación:** (a) para compatibilidad de configuracion HOCON (Q-X-1).
- **Filas afectadas:** AC-CS-005
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-5 — exit-jvm / System.exit en Rust y terminacion del ActorSystem desde la fase final

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:239-277; ActorSystem.scala:478-481,1086-1112
- **Alternativas:** (a) `std::process::exit(code)` mantenido bajo el nombre de clave `exit-jvm`; (b) clave nueva `exit-process`
- **Recomendación:** (a) mantener claves; documentar que "jvm" significa proceso.
- **Filas afectadas:** AC-CS-006.1;AC-CS-006.2
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-6 — Hook de apagado del proceso: senales y registro en el runtime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:279-297,844-889; reference.conf:1268-1271
- **Alternativas:** (a) tokio::signal (SIGTERM, y SIGINT como extension); (b) ctrlc/signal-hook; (c) solo API manual
- **Recomendación:** (a); SIGINT soportado es una diferencia observable respecto de Pekko (doc dice que SIGINT no funciona).
- **Filas afectadas:** AC-CS-007.1;AC-CS-007.2
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-DSP-1 — Qué claves de executor se honran: fork-join, thread-pool, affinity-pool, virtual-thread

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:371-585 (cada executor y sus claves); ForkJoinExecutorConfigurator.scala; ThreadPoolBuilder.scala; AffinityPool.scala
- **Alternativas:** (a) honrar todas las claves mapeándolas a un pool propio; (b) honrar parallelism-*, fixed-pool-size, thread-pool-*; mapear minimum-runnable, task-peeking-mode, maximum-pool-size, virtualize, affinity-* a "aceptadas y registradas, sin efecto, con warning"; (c) pools propios independientes del runtime
- **Recomendación:** (b) con equivalentes: virtual-thread => tareas async (cumple la intención), affinity-pool => alias a pool con colas por worker si Q-X-4 elige hilos propios; diferencia observable (nombres de hilo, work stealing) documentada. Sin exclusión: todas las claves se leen y validan.
- **Filas afectadas:** AC-DSP-008,AC-DSP-009,AC-DSP-013,AC-DSP-014,AC-DSP-019
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-2 — Registro de configuradores de dispatcher/executor/mailbox por FQCN (type, executor, mailbox-type)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatchers.scala:197-300,381-407; AbstractDispatcher.scala:353-410; Mailboxes.scala:241-255
- **Alternativas:** (a) registro por nombre; (b) nombres FQCN literales de Pekko como claves alias
- **Recomendación:** Ver Q-X-2; aceptar los nombres FQCN de Pekko como alias de los builtin
- **Filas afectadas:** AC-DSP-004,AC-DSP-010,AC-MBX-004
- **Unidades afectadas:** DSP,MBX
- **Origen:** classic-dispatch

### Q-DSP-3 — ExecutionContext/Future helpers (AC-DSP-017): portar sólo parasitic o todo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Future.scala; CompletionStages.scala (API Scala/Java específica)
- **Alternativas:** (a) portar sólo `parasitic`+`execute`; (b) sólo trait Executor; (c) todo
- **Recomendación:** (a); el resto se documenta como equivalente `std::future`/tokio sin API propia (propuesta de exclusión, la fila sigue en alcance hasta autorización)
- **Filas afectadas:** AC-DSP-017,AC-DSP-018
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-4 — Detección de ciclos de alias de dispatcher y profundidad máxima

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatchers.scala:171-190 (MaxDispatcherAliasDepth=20)
- **Alternativas:** (a) paridad con 20
- **Recomendación:** (a)
- **Filas afectadas:** AC-DSP-004
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-5 — throughput <=0: documentación (continúa hasta vaciar) vs código (equivale a 1)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatcher.scala:37-40 (Scaladoc: "Zero or negative means the dispatcher always continues until the mailbox is empty"); Mailbox.scala:264 (left = max(throughput,1)); reference.conf:589-591
- **Alternativas:** (a) seguir el código (0/neg = 1); (b) seguir la doc (0/neg = vaciar mailbox)
- **Recomendación:** (a): el código es lo observable y ningún spec afirma (b); registrar la diferencia con la doc
- **Filas afectadas:** AC-DSP-003,AC-MBX-002
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DW-1 — Igualdad del mensaje Terminated y existenceConfirmed para actores ya muertos

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:112-135 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:746-752 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:54-72 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:353
- **Alternativas:** (a) PartialEq sólo por actor (como la case class de Pekko con segunda lista de parámetros) y existenceConfirmed=false al vigilar un actor ya muerto / (b) PartialEq por los tres campos / (c) por actor y existenceConfirmed
- **Recomendación:** (a): los specs hacen expectMsg(Terminated(a)(true,false)) y esperan igualdad por actor; documentar que existenceConfirmed no entra en equals
- **Filas afectadas:** AC-DW-002 ; AC-DW-006
- **Unidades afectadas:** DW
- **Origen:** classic-cell

### Q-DW-2 — Hooks de remote en DeathWatch: AddressTerminated, AddressTerminatedTopic y orden remoto→local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:133-163,217-264 ; actor/src/main/scala/org/apache/pekko/event/AddressTerminatedTopic.scala:30-72
- **Alternativas:** (a) conservar el hook is_local en el trait de ref y el topic sin uso local / (b) omitir el topic y el orden / (c) implementarlo completo con un provider simulado
- **Recomendación:** (a) mínimo: el trait de ref expone is_local y la celda mantiene los conjuntos watching/watchedBy con la lógica de suscripción, aunque sin remote nunca se suscriba
- **Filas afectadas:** AC-DW-010 ; AC-DW-012
- **Unidades afectadas:** DW
- **Origen:** classic-cell

### Q-EVT-1 — Nombres de clase JVM en mensajes observables (dead letters, canales Debug, `class`/`interface`)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** DeadLetterListener.scala:123-137 (`Message [java.lang.Integer]`); EventStreamSpec.scala:395-423 (`channel class ...A`/`interface ...T`); toString de ActorRef `Actor[pekko://sys/path#uid]`.
- **Alternativas:** (a) usar `std::any::type_name` y declarar la diferencia; (b) tabla de nombres Pekko para tipos estándar (Integer, String) y type_name para el resto; (c) nombre configurable por tipo
- **Recomendación:** (b) para tipos primitivos/estándar que los specs afirman (`java.lang.Integer`) y (a) para el resto; el texto exacto de specs portados se ajusta con una función `message_type_name`
- **Filas afectadas:** AC-EVT-006 ; AC-EVT-008 ; AC-EVT-009 ; AC-EVT-010
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-2 — Fidelidad byte a byte de la salida stdout de los loggers por defecto

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Logging.scala:993-1105 (formatos), Helpers.scala:108-113 (timestamp MM/dd/yyyy HH:mm:ss.SSS en zona del sistema), LoggerSpec.scala:319-345.
- **Alternativas:** (a) byte-compatible con Pekko (thread = nombre de hilo del runtime); (b) formato propio con `tracing`; (c) ambos: StandardOutLogger fiel y puente opcional a `tracing`
- **Recomendación:** (c), con (a) como default de DefaultLogger/StandardOutLogger
- **Filas afectadas:** AC-EVT-011 ; AC-EVT-017
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-3 — Plantillas `{}` y varargs en LoggingAdapter

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Logging.scala:1520-1557; LoggingAdapterTest.java:55-90: hasta 4 argumentos o Array expandido; sufijo ` WARNING arguments left: N`.
- **Alternativas:** (a) métodos con `&[&dyn Display]` + macros `log_info!`; (b) sólo macros (no verificable con tests dinámicos); (c) `format!` nativo sin `{}` de Pekko
- **Recomendación:** (a): conserva el formato textual `{}` y el sufijo de argumentos sobrantes, con macros como azúcar
- **Filas afectadas:** AC-EVT-013.1
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-4 — JavaLogger (java.util.logging) en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** JavaLogger.scala:32-120 (deprecado, JVM).
- **Alternativas:** (a) puente opcional a la fachada `log`; (b) sin equivalente (requiere exclusión explícita)
- **Recomendación:** (a)
- **Filas afectadas:** AC-EVT-021
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-5 — API Java (EventBusJavaAPI, javadsl TestKit/EventFilter)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** event/japi/EventBusJavaAPI.scala; testkit javadsl/*.
- **Alternativas:** (a) la misma API única en Rust (no existe distinción Java); (b) módulo de compatibilidad con nombres Java
- **Recomendación:** (a); la fila queda cubierta por AC-EVT-002..005 y AC-TK-001/005
- **Filas afectadas:** AC-EVT-022 ; AC-TK-011
- **Unidades afectadas:** EVT ; TK
- **Origen:** classic-system

### Q-EXT-1 — Identidad del ExtensionId

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Extension.scala:123-124 (hashCode = identityHashCode, equals = eq, ambos final); ExtensionSpec.scala:43-53 (singletons).
- **Alternativas:** (a) `TypeId` del tipo que implementa ExtensionId (singleton por tipo); (b) dirección de un `static`; (c) clave explícita `&'static str`.
- **Recomendación:** (a) con una nota sobre extensiones parametrizadas.
- **Filas afectadas:** AC-EXT-001.1, AC-EXT-002
- **Unidades afectadas:** EXT
- **Origen:** classic-foundation

### Q-EXT-2 — Semántica de registerExtension: bloqueo, reentrada, caché de fallo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:1165-1221; ExtensionSpec.scala:82-97 (misma excepción en el 2.º intento, efectos laterales no revertidos); mensaje de timeout con `creation-timeout`.
- **Alternativas:** (a) mismo diseño con condvar y `creation-timeout`; (b) creación una sola vez con `OnceLock` y panic/Err cacheado; (c) async con `tokio::sync::OnceCell`.
- **Recomendación:** Depende de Q-X-4; (a) para runtime con hilos, (c) para async.
- **Filas afectadas:** AC-EXT-001.2
- **Unidades afectadas:** EXT
- **Origen:** classic-foundation

### Q-EXT-3 — AC-EXT-004 como fila índice de extensiones provistas

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Las extensiones están en actor/CoordinatedShutdown.scala:42, serialization/SerializationExtension.scala:24, io/Tcp.scala:50, pattern/CircuitBreakersRegistry.scala:33, event/AddressTerminatedTopic.scala:38; cada una pertenece a otra unidad.
- **Alternativas:** (a) mantener la fila como índice y cerrarla cuando se cierren las filas hijas; (b) repartir cada extensión en su unidad.
- **Recomendación:** (a).
- **Filas afectadas:** AC-EXT-004
- **Unidades afectadas:** EXT
- **Origen:** classic-foundation

### Q-FSM-1 — FSM en Rust: funciones parciales (when/whenUnhandled/transform), tipos S y D, respuestas y mensajes dinamicos

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** FSM.scala:439-441 (PartialFunction[Event, State]); 774-792 (orElse); 306-308 (replying Any)
- **Alternativas:** (a) trait Fsm con tablas de closures Option-returning y `Box<dyn Any>` para mensajes (Q-X-3); (b) enum cerrado de eventos por FSM (solo tipado)
- **Recomendación:** (a) para paridad con classic; (b) corresponde a Typed (Q-X-8).
- **Filas afectadas:** AC-FSM-001.1;AC-FSM-001.2;AC-FSM-002;AC-FSM-008;AC-FSM-009
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-FSM-2 — Timers de FSM y state timeout: sobre el scheduler y generacion de cancelacion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** FSM.scala:105-160,814-829,896-907
- **Alternativas:** (a) replicar generacion (TimeoutMarker/Timer con gen) sobre el scheduler de AC-SCH; (b) timers locales de tokio
- **Recomendación:** (a) (paridad de cancelacion de mensajes ya encolados); depende de Q-X-4.
- **Filas afectadas:** AC-FSM-003;AC-FSM-004
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-FSM-3 — Textos de log exactos de FSM y debug.fsm

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** FSM.scala:616-617,631-632,787-790,981-1004 ; FSMActorSpec:262-312
- **Alternativas:** (a) emitir exactamente los textos de Pekko por el logging de rakka (Q-X-7); (b) estructurar los campos
- **Recomendación:** (a): son aserciones de spec (prefijos y textos completos en :296,:297,:304).
- **Filas afectadas:** AC-FSM-005;AC-FSM-007
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-IO-1 — Modelo de ejecución de pekko.io: selectores + pinned-dispatcher + actores de canal vs tokio/mio con actores-conexión

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** SelectionHandler.scala:84-387 (bucle select, interest ops); reference.conf:930-1138 (nr-of-selectors, selector-dispatcher, worker-dispatcher, select-timeout); TcpConnectionSpec.scala:883-1142 (ChannelRegistry simulado); Q-X-4
- **Alternativas:** (a) Reproducir selectores sobre mio con un hilo por selector y actores de canal que reciben ChannelReadable/Writable; (b) actores-conexión sobre tokio TcpStream/UdpSocket manteniendo el protocolo de mensajes público y descartando interest ops; (c) híbrido: costura ChannelRegistry para portar los specs de conexión y tokio detrás
- **Recomendación:** (c): protocolo público idéntico (Bind/Connect/Register/Write...), costura `ChannelRegistry` interna para poder portar TcpConnectionSpec/TcpListenerSpec (Q-IO-2), y claves nr-of-selectors/selector-dispatcher/worker-dispatcher/select-timeout aceptadas con efecto documentado (probable no-op). Depende de Q-X-4
- **Filas afectadas:** AC-IO-001; AC-IO-002.2; AC-IO-002.3; AC-IO-002.4; AC-IO-002.7; AC-IO-003; AC-IO-004.1; AC-IO-004.2; AC-IO-004.4; AC-IO-006.1; AC-IO-006.2; AC-IO-007; AC-IO-008.5; AC-IO-011
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-2 — Estrategia de port de specs de conexión que simulan el selector

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TcpConnectionSpec.scala:96-1142 (36 tests con selector.send(ChannelReadable/Connectable)); TcpListenerSpec.scala:36-142; TcpIntegrationSpec.scala:41-205 (caja negra)
- **Alternativas:** (a) Portar solo los specs de integración (caja negra); (b) portar también los de conexión sobre un ChannelRegistry falso inyectable; (c) reescribir los de conexión como integración en loopback
- **Recomendación:** (b) si Q-IO-1 adopta la costura ChannelRegistry; si no, (c) manteniendo las aserciones de secuencia de mensajes. Los tests pending/ignore (TcpConnectionSpec.scala:109,316-320) no son aserciones
- **Filas afectadas:** AC-IO-002.2; AC-IO-002.3; AC-IO-002.4; AC-IO-002.5; AC-IO-002.6; AC-IO-004.4
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-3 — Piezas NIO/JVM/SO específicas: buffers directos y limpieza, workaround de Windows, finish-connect-retries, transferTo, DatagramChannelCreator, textos de error del SO

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** DirectByteBufferPool.scala:38-106; ByteBufferCleaner.java:21-100; reference.conf:964-1025; TcpConnection.scala:224-250,533-549; Inet.scala:86-101; TcpConnectionSpec.scala:97-98 (mensajes por plataforma)
- **Alternativas:** (a) Portar todo con equivalentes (pool de BytesMut, socket2, sendfile, factoría de sockets); (b) aceptar las claves y no implementar ByteBufferCleaner, windows-workaround, finish-connect-retries; (c) excluir WritePath/WriteFile
- **Recomendación:** (a) para pool de buffers, opciones de socket (socket2), WritePath (tokio::fs + sendfile) y DatagramSocketFactory; (b) para ByteBufferCleaner (Drop), windows-connection-abort-workaround y finish-connect-retries, cuya exclusión se propone al Owner; el texto de ErrorClosed/CommandFailed.cause será el del SO (std::io::Error) y los tests lo comparan contra constantes por plataforma como hace Pekko
- **Filas afectadas:** AC-IO-002.1; AC-IO-002.3; AC-IO-002.5; AC-IO-002.6; AC-IO-003; AC-IO-005; AC-IO-006.3; AC-IO-010; AC-IO-013
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-4 — Resolver inet-address: caché y propiedades JVM, preferencia IPv6, API deprecada y clave de caché Ip()

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** InetAddressDnsResolver.scala:49-174; InetAddressDnsResolverSpec.scala:31-149; Dns.scala:76-160,257-267 (preferIPv6Addresses); SimpleDnsCacheSpec.scala:77 (TODO)
- **Alternativas:** (a) "default" = 30 s positivo / Never negativo, sin propiedades de entorno, IPv4 primero; (b) leer variables de entorno análogas; (c) portar también la API deprecada Dns.Resolve/Resolved y reproducir la clave de caché Ip() inconsistente
- **Recomendación:** (a) para TTL y IPv6 (las propiedades networkaddress.cache.* y java.net.preferIPv6Addresses no existen); no portar la API deprecada salvo que el Owner la pida; la clave Ip() inconsistente se corrige (clave por flags reales) y se anota la diferencia
- **Filas afectadas:** AC-IO-002.3; AC-IO-008.1; AC-IO-008.3; AC-IO-009.8
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-5 — Nombres de host en UDP y UdpConnected: Send con miss de caché y doConnect que ignora la dirección resuelta

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** WithUdpSend.scala:58-78 (miss -> CommandFailed inmediato); UdpConnection.scala:55-63,82-92 (doConnect(address) usa remoteAddress); UdpIntegrationSpec.scala:62-73; UdpConnectedIntegrationSpec.scala:61-77
- **Alternativas:** (a) Replicar exactamente (UdpConnected a host por nombre siempre falla; el primer Send a nombre falla); (b) corregir: conectar a la dirección resuelta y esperar la resolución en Send; (c) corregir solo UdpConnected
- **Recomendación:** (b) con diferencia documentada: es un defecto latente de 1.7.1 (derivado del código, NO verificado contra el jar). Antes de decidir, correr el escenario de oráculo "UdpConnected.Connect a localhost por nombre" para confirmar el comportamiento real
- **Filas afectadas:** AC-IO-006.2; AC-IO-007
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-6 — Nameservers por defecto del async-dns y alcance de /etc/resolv.conf

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** DnsSettings.scala:62-80,110-127,195-253 (JNDI y sun.net.dns.ResolverConfiguration); ResolvConfParser.scala:42-76 (solo search/domain/ndots); DnsSettingsSpec.scala:42-48 (solo "no lanza")
- **Alternativas:** (a) Leer líneas nameserver de /etc/resolv.conf (Unix) y APIs del SO en Windows/macOS; (b) exigir nameservers explícitos (error IllegalState con el mismo texto); (c) crate externo (hickory/resolv-conf)
- **Recomendación:** (a) en Unix con (b) como error cuando la lista queda vacía, con el mismo texto de IllegalStateException; Windows/macOS: pendiente de decisión del Owner. Pekko no lee "nameserver" del archivo, por lo que es una ampliación observable
- **Filas afectadas:** AC-IO-009.1; AC-IO-009.2; AC-IO-009.6
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-JAPI-1 — Superficie Java-facing (AbstractActor y familia, AbstractFSM, japi.function, EventBus Java, sobrecargas, fábricas IO, Props.create con Creator/lambda)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AbstractActor.scala:32-427; AbstractFSM.scala:26-586; AbstractProps.scala:28-138; EventBusJavaAPI.scala:26-230; JavaAPI.java:66-161; Tcp.scala:702-; Udp.scala:257-363
- **Alternativas:** (a) Excluir la superficie Java (la API Rust es una sola) manteniendo en alcance el comportamiento observable; (b) reproducir clases base con traits y builders; (c) portar solo las validaciones de Props
- **Recomendación:** (a) propuesta de exclusión de los nombres/clases Java; las filas siguen en alcance con impl=pending y su semántica queda cubierta por las filas Scala/Rust equivalentes (CELL, FSM, EVT, SCH, IO). Si el Owner no autoriza la exclusión se implementa (b)
- **Filas afectadas:** AC-JAPI-001; AC-JAPI-002; AC-JAPI-003; AC-JAPI-004.3; AC-JAPI-005.1; AC-JAPI-005.2; AC-JAPI-005.4; AC-JAPI-006; AC-JAPI-007; AC-JAPI-008; AC-JAPI-010; AC-JAPI-011
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-JAPI-2 — ReceiveBuilder/PFBuilder/Match para receive dinámico: igualdad de tipo exacto vs isInstance

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PFBuilder.java:43-148; Match.java:131-133; MatchBuilderTest.java:28-58 (Integer antes que Number); Q-X-3, Q-X-6
- **Alternativas:** (a) Builder por TypeId exacto con match_any/match_equals y MatchError como Err; (b) emular isInstance con la relación de supertipos de Q-X-6; (c) no ofrecer builder (solo match de Rust)
- **Recomendación:** (a) como utilidad idiomática de los actores classic con Box<dyn Any> (Q-X-3), con (b) si Q-X-6 define supertipos; la diferencia con isInstance se documenta
- **Filas afectadas:** AC-JAPI-004.1; AC-JAPI-004.2; AC-JAPI-004.4
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-JAPI-3 — TypedActor clásico deprecado (proxies JDK dinámicos)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TypedActor.scala:43-524; TypedActorSpec.scala:290-588; Akka 2.6 deprecación (@deprecated)
- **Alternativas:** (a) Excluir; (b) macro `#[typed_actor]` que genere un proxy desde un trait con las cuatro formas de retorno; (c) portar solo MethodCall y el actor interno sin proxy
- **Recomendación:** (a) exclusión propuesta (deprecado y sin equivalente dinámico); la fila sigue en alcance. Si el Owner prefiere paridad, (b) con la semántica de invoke de TypedActor.scala:466-499 (void unidireccional, Future sin bloqueo, Option con timeout -> none, valor con Await)
- **Filas afectadas:** AC-JAPI-009.1; AC-JAPI-009.2; AC-JAPI-009.3
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-MBX-1 — Semántica de BoundedMailbox: enqueue bloqueante con push-timeout vs. no bloqueante

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:560-568 (offer con timeout; put si timeout negativo; DeadLetter si vence); reference.conf:644-647; Mailboxes.scala:257-266 (Pekko advierte que un timeout no cero bloquea al emisor y recomienda 0)
- **Alternativas:** (a) `tell` síncrono que bloquea el hilo emisor hasta pushTimeOut (paridad); (b) `tell` nunca bloquea: si está lleno, DeadLetter inmediato (equivale a push-timeout=0); (c) paridad en el modo 0 y `try_tell`/`tell_async` awaitable para timeout>0
- **Recomendación:** Implementar (c): `mailbox-push-timeout-time=0` y timeouts >0 en cola con envío awaitable; `tell` sincrónico con timeout>0 bloquea el hilo (paridad) sólo fuera de un worker async; diferencia observable documentada. Requiere decisión del Owner (Q-X-4).
- **Filas afectadas:** AC-MBX-007,AC-MBX-016
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-2 — Identidad de tipo de cola / semántica: reflexión de marker traits vs. descriptor explícito

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailboxes.scala:117-152 (Reflect.findMarker sobre RequiresMessageQueue[T] y ProducesMessageQueue[T]); Mailbox.scala:509-648
- **Alternativas:** (a) `enum/bitflags QueueSemantics` declarados por MailboxType y por el actor; (b) TypeId + subtyping manual
- **Recomendación:** (a) con jerarquía de implicancia codificada (p.ej. UnboundedDequeBased implica Deque y Unbounded y MultipleConsumer); mapeo `requirements` por nombre de semántica
- **Filas afectadas:** AC-MBX-013,AC-MBX-014,AC-MBX-015
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-3 — ControlMessage: detección de mensajes de control en mensajes `dyn Any`

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:877-880 (match por tipo `ControlMessage`)
- **Alternativas:** (a) trait/método opcional en el envoltorio Envelope; (b) registro de TypeId de control; (c) predicado en el MailboxType
- **Recomendación:** (b) registro por TypeId a nivel sistema + hook (c) para custom; depende de Q-X-3
- **Filas afectadas:** AC-MBX-012
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-4 — Comparator de prioridad sobre Envelope: orden estable bajo empate para UnboundedPriorityMailbox

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PriorityDispatcherSpec (orden entre iguales no especificado, Mailbox.scala:741-745 usa PriorityBlockingQueue sin garantía); StablePriorityDispatcherSpec afirma FIFO
- **Alternativas:** (a) usar siempre heap estable también en el no estable (más estricto que Pekko); (b) BinaryHeap sin secuencia
- **Recomendación:** (b) para el no estable (no afirmar orden entre iguales); (a) rechazado por diferenciar solo hacia más estricto
- **Filas afectadas:** AC-MBX-009,AC-MBX-010
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-5 — Aislamiento de tipos de mensajes de sistema vs. usuario en el mailbox (lista de sistema CAS + deadLetterMailbox)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:340-354,473-503; Mailboxes.scala:54-72
- **Alternativas:** (a) cola de sistema separada Treiber-stack o Mutex<Vec> invertido al drenar; (b) canal único con prioridad
- **Recomendación:** (a) conservar dos colas; sistema siempre antes que usuario
- **Filas afectadas:** AC-MBX-001,AC-MBX-002,AC-MBX-003
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-6 — Modelo de ejecución del mailbox (hilo + ForkJoinTask vs. tarea async): tareas por turno, fairness y `throughput`

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:230-280; Dispatcher.scala:42-160
- **Alternativas:** (a) tarea tokio por mailbox que procesa N mensajes y hace yield_now; (b) pool propio work-stealing; ver Q-X-4
- **Recomendación:** (a) bajo Q-X-4 (runtime); cada actor es una tarea que se re-spawnea; throughput = nº máx de mensajes por poll
- **Filas afectadas:** AC-MBX-002,AC-DSP-002,AC-DSP-003
- **Unidades afectadas:** MBX,DSP
- **Origen:** classic-dispatch

### Q-PATH-1 — Cómo ordenar listas de Address con un comparador que no es orden total

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Address.scala:135-145; plan AC-PATH: slice::sort_by puede entrar en pánico en Rust >= 1.81 si detecta un orden no total y puede diferir de List.sort de Java (CMPA-17).
- **Alternativas:** (a) sólo `pekko_compare` y documentar el límite; (b) función que reproduce el TimSort de Java; (c) desempate `None < Some` con `Ord` (contradice la decisión del Owner).
- **Recomendación:** (a) hasta que haya un consumidor (cluster).
- **Filas afectadas:** AC-PATH-002
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-PATH-2 — Surrogate suelto en el mensaje de validación de nombres

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorPath.scala:107-113: para `x🙂` Pekko informa `illegal character [\uD83D] at position: 1` (VAL-23); un String de Rust no puede contener un surrogate suelto.
- **Alternativas:** (a) U+FFFD (`from_utf16_lossy`); (b) `?`; (c) el carácter completo; (d) guardar el mensaje en UTF-16.
- **Recomendación:** (a); la posición sigue siendo el índice UTF-16.
- **Filas afectadas:** AC-PATH-008
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-PATH-3 — Valor de hashCode de Address y ActorPath

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Address.scala:77-79 (MurmurHash3.productHash), ActorPath.scala:469-479 (hash por niveles). Rust sólo expone `Hash`.
- **Alternativas:** (a) no reproducir el valor; (b) `pekko_hash_code() -> i32` idéntico.
- **Recomendación:** (a); (b) si alguna unidad lo necesita.
- **Filas afectadas:** AC-PATH-001, AC-PATH-014
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-PATH-4 — `null` de Java en Address/ActorPath

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** fromString(null) y validatePathElement(null) lanzan NullPointerException; `new Address(null, "s")` imprime `null://s`.
- **Alternativas:** (a) aceptarlo como diferencia de lenguaje; (b) tipos `Option<&str>` en la API.
- **Recomendación:** (a); `host = null` sí se representa como None.
- **Filas afectadas:** AC-PATH-001, 004, 007, 008
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-PATH-5 — Fachada Java ActorPaths

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorPath.scala:29-65 delega 1:1 en ActorPath; es la pregunta 7 de docs/inventory/actor-classic.md.
- **Alternativas:** (a) excluir; (b) fusionar con AC-PATH-007/008; (c) portar como funciones libres.
- **Recomendación:** (b)/(c): las funciones libres ya cubren el contrato; la exclusión requiere decisión explícita.
- **Filas afectadas:** AC-PATH-009
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-PATH-6 — Licencia de uri.rs y jstring.rs (comportamiento portado de OpenJDK 21, GPL-2.0 WITH Classpath-exception-2.0)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** crates/rakka-actor/src/actor/uri.rs y jstring.rs reproducen java.net.URI, sun.nio.cs.UTF_8, String.compareTo, Integer.valueOf, Character.digit; el proyecto es Apache-2.0 (docs/upstream.md).
- **Alternativas:** (a) aceptar como reimplementación del comportamiento y declarar la referencia en el encabezado (estado actual); (b) sala limpia desde RFC 2396/3629 y el oráculo; (c) crate externo compatible.
- **Recomendación:** Es una pregunta de licencia, no de semántica; recomendar (b) o (a) con revisión legal.
- **Filas afectadas:** AC-PATH-002, 004, 005, 006, 007, 011, 014
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-REF-1 — Representación de "sin sender" y valor de sender() fuera de un mensaje

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:416 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:44-52 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:596-600 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:157,736
- **Alternativas:** (a) tell(msg, Option<&ActorRef>) y sender() devuelve siempre un ActorRef (deadLetters por defecto) / (b) sender() devuelve Option<ActorRef> / (c) ActorRef con variante NoSender
- **Recomendación:** (a): iguala el observable de Pekko (el receptor siempre ve deadLetters); DeadLetterActorRef usa sí mismo como sender por defecto (ActorRef.scala:736) y debe replicarse
- **Filas afectadas:** AC-REF-002 ; AC-REF-005 ; AC-REF-024
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-REF-2 — Props sin reflexión: formas classOf + args, IndirectActorProducer y InstantiationException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Props.scala:32-124 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:622-640,667-674
- **Alternativas:** (a) sólo closures y constructores tipados (Props::new(|| A)) / (b) además un registro de fábricas por nombre (Q-X-2) para clases por nombre / (c) macros de derive
- **Recomendación:** (a) más (b) para configuración por nombre; las formas reflexivas puras (classOf con args null, mensaje de InstantiationException, validación de firma de constructor) se marcan como diferencia observable aceptada; la fila sigue en alcance hasta que el Owner decida
- **Filas afectadas:** AC-REF-007.1 ; AC-REF-007.2 ; AC-REF-008 ; AC-CELL-002.2
- **Unidades afectadas:** REF ; CELL
- **Origen:** classic-cell

### Q-REF-3 — Serialización Java de ActorRef (writeReplace / readResolve / SerializedActorRef)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:471-496 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:288-365
- **Alternativas:** (a) conservar sólo el contrato path→ref con serialized_actor_path y resolve_actor_ref, sin formato Java / (b) implementar un serializador propio equivalente a SerializedActorRef (string de path) / (c) no soportar
- **Recomendación:** (a)+(b): conservar el mensaje IllegalStateException "Trying to deserialize a serialized ActorRef without an ActorSystem in scope…" como error del resolver cuando no hay sistema en el contexto de deserialización
- **Filas afectadas:** AC-REF-023
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-REF-4 — Restricción "los actores sólo se crean vía actorOf" (ActorInitializationException al usar new)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:495-506 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:148-175
- **Alternativas:** (a) estructural: el contexto es un parámetro y el error es imposible; el caso se marca no aplicable / (b) emular con un guard de runtime
- **Recomendación:** (a) con justificación escrita; el test upstream no se porta
- **Filas afectadas:** AC-REF-006
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-REF-5 — Creación asíncrona de top-level (system.actorOf vía RepointableActorRef/UnstartedCell) frente a síncrona (context.actorOf)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:659-719 ; actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:95-135,204-326 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:58-64
- **Alternativas:** (a) crear siempre la celda de forma síncrona preservando observables / (b) reproducir UnstartedCell con cola previa y lock con unstarted-push-timeout / (c) cola previa sin lock
- **Recomendación:** (a) si se prueba que los observables coinciden: mensajes enviados justo tras actorOf se entregan en orden, los mensajes de sistema encolados antes se entregan primero, y Supervise llega al guardián antes de la celda real; si no se puede, (b). Impacta pekko.actor.unstarted-push-timeout (reference.conf:145)
- **Filas afectadas:** AC-REF-020 ; AC-CELL-016 ; AC-CELL-027
- **Unidades afectadas:** REF ; CELL
- **Origen:** classic-cell

### Q-REF-6 — Visibilidad de accesores internos usados por tests y testkit (ActorRef.isTerminated, underlying, children)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:173,309,389 ; actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:210
- **Alternativas:** (a) pub(crate) más un módulo testkit con reexport / (b) pub con doc hidden / (c) no exponer
- **Recomendación:** (a): los specs portados necesitan isTerminated y el estado de mailbox
- **Filas afectadas:** AC-REF-001.1 ; AC-REF-001.2 ; AC-REF-019
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-RT-1 — Routing en el hilo del emisor y Router inmutable: mensajes dinamicos y estado atomico

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** RoutedActorCell.scala:147-152; Router.scala:110-185
- **Alternativas:** (a) Router en ArcSwap con ruteo en el tell del emisor (paridad); (b) actor-router con cola propia (mas simple, cambia el hilo y el orden)
- **Recomendación:** (a) paridad observable; mensajes Box<dyn Any> (Q-X-3).
- **Filas afectadas:** AC-RT-001;AC-RT-002;AC-RT-005;AC-RT-006;AC-RT-007
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-2 — Mensajes de gestion del router y ciclo de vida del router actor

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** RoutedActorCell.scala:65-114,159-218
- **Alternativas:** (a) replicar GetRoutees/AddRoutee/RemoveRoutee/AdjustPoolSize y Terminated->evict; (b) API tipada separada
- **Recomendación:** (a), con los mismos nombres de mensaje.
- **Filas afectadas:** AC-RT-003.1;AC-RT-003.2;AC-RT-003.3
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-3 — Router config via HOCON: type-mapping, FQCN custom, fromConfig por router

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:155-170; Deployer.scala:233-290; Resizer.scala:69-80
- **Alternativas:** (a) tabla de fabricas por nombre (Q-X-2) leyendo las mismas claves; (b) solo API programatica
- **Recomendación:** (a) misma sintaxis HOCON (Q-X-1) y mismas claves.
- **Filas afectadas:** AC-RT-004;AC-RT-013.3;AC-RT-016
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-5 — ConsistentHashing: serializacion de claves arbitrarias y MurmurHash exacto

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ConsistentHashing.scala:175-260 (AnyRef -> SerializationExtension); MurmurHash.scala:127-141
- **Alternativas:** (a) claves String/bytes con hash identico; resto via trait `ConsistentHashKey` propio (difiere de Pekko); (b) replicar serializacion Java (imposible)
- **Recomendación:** (a); documentar diferencia observable para claves no String/bytes.
- **Filas afectadas:** AC-RT-009.1;AC-RT-009.2;AC-RT-009.3
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-6 — Discrepancias doc/codigo en ScatterGather y TailChopping (excepcion y texto)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ScatterGatherFirstCompleted.scala:41-42 (doc: AskTimeoutException) vs :62 (TimeoutException "Timeout due to no routees"); TailChopping.scala:105 ("[$within.toMillis} ms]")
- **Alternativas:** (a) replicar el codigo exactamente (tipo y texto, incluido el defecto de formato); (b) corregir al comportamiento documentado
- **Recomendación:** (a) replicar el codigo (spec afirma TimeoutException en ScatterGatherFirstCompletedSpec:113); el texto de TailChopping como diferencia a decidir por el Owner.
- **Filas afectadas:** AC-RT-010;AC-RT-011
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-7 — BalancingPool: dispatcher de mailbox compartido (BalancingDispatcher) y specs pending upstream

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Balancing.scala:116-144; BalancingSpec.scala:133-165 (pending)
- **Alternativas:** (a) cola multiconsumidor compartida; (b) work-stealing entre mailboxes
- **Recomendación:** (a); depende de AC-DSP-012; los tests de reparto upstream estan pending: validar con aserciones propias.
- **Filas afectadas:** AC-RT-012
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-8 — Listeners requiere ActorRef ordenable (TreeSet) y semantica de duplicados

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Listeners.scala:37-47
- **Alternativas:** (a) BTreeSet<ActorRef> con Ord por path+uid; (b) Vec con orden de insercion
- **Recomendación:** (a) paridad con TreeSet (orden e idempotencia).
- **Filas afectadas:** AC-RT-015;AC-FSM-002
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-SCH-1 — Rueda de ticks (LightArrayRevolver) propia vs. timers del runtime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** LightArrayRevolverScheduler.scala:50-357; reference.conf:886-927; SchedulerSpec
- **Alternativas:** (a) portar la rueda (precisión por tick, semántica de redondeo y wrap); (b) tokio::time y emular redondeo a tick
- **Recomendación:** (a) port de la rueda sobre un hilo propio o tarea de tokio con tick (la rueda es observable: redondeo al tick, ticks-per-wheel, rechazo de delays); testable con reloj controlado
- **Filas afectadas:** AC-SCH-001,AC-SCH-002,AC-SCH-003,AC-SCH-008,AC-SCH-009
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch

### Q-SCH-2 — Scheduler pluggable (scheduler.implementation) sin reflexión

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:914-921; ActorSystem.scala:1136-1160
- **Alternativas:** (a) registro por nombre + inyección en builder; (b) sólo inyección
- **Recomendación:** (a) ver Q-X-2
- **Filas afectadas:** AC-SCH-010
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch

### Q-SCH-3 — Mensajes de timer: tipo de mensaje interno y entrega al actor

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TimerSchedulerImpl.scala:28-164
- **Alternativas:** (a) Envelope especial de timer con generación interceptado antes de receive; (b) mensaje normal
- **Recomendación:** (a) paridad
- **Filas afectadas:** AC-SCH-011,AC-SCH-012,AC-SCH-013
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch

### Q-SCH-4 — schedule(...) deprecado: mantener como alias de scheduleAtFixedRate

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Scheduler.scala:229-231,346-396 (scheduleAtFixedRate delega en schedule); LightArrayRevolverScheduler.scala:134 (implementación del tipo fixed-rate vive en schedule)
- **Alternativas:** (a) alias deprecado; (b) omitir y exponer sólo schedule_at_fixed_rate (propuesta de exclusión, requiere autorización)
- **Recomendación:** (a): costo nulo, mismo código interno
- **Filas afectadas:** AC-SCH-004,AC-SCH-003
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch

### Q-SER-2 — Java serialization (JavaSerializer, DisabledJavaSerializer, java.io.Serializable, JavaSerializer.currentSystem, serialización Java de ActorRef)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Serializer.scala:306-418; reference.conf:761-778,799-831; SerializeSpec.scala:372-374,402-511; SerializationSetupSpec.scala:170-212; DisabledJavaSerializerWarningSpec.scala:42-68; ActorRef.scala:462-496
- **Alternativas:** (a) Excluir toda la serialización Java incluyendo el id 1 y los avisos; (b) mantener solo DisabledJavaSerializer (id 1 reservado, warnings Security y excepciones con texto exacto) y la lógica de avisos, sin JavaSerializer habilitado; (c) emular JavaSerializer con un formato propio
- **Recomendación:** (b): allow-java-serialization siempre efectivo en off; "enabled" propuesto como exclusión (sin equivalente razonable en Rust); la fila AC-SER-013 sigue en alcance con impl=pending hasta que el Owner decida. La tabla por defecto conserva el binding "Serializable -> java" como entrada vacía documentada
- **Filas afectadas:** AC-SER-003.5; AC-SER-004.1; AC-SER-004.2; AC-SER-013
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-3 — Información de transporte (thread-local) y serialización de ActorRef con address externa

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Serialization.scala:46-141,170-177,282-295; ActorRefProvider.scala:764-780; ActorRef.scala:471-486 (el límite de remote queda fuera del módulo)
- **Alternativas:** (a) task_local/thread_local con TransportInformation; (b) parámetro explícito SerializationContext en Serializer::to_binary/from_binary; (c) solo address por defecto del sistema local (sin Information)
- **Recomendación:** (b) para serializadores de usuario nuevos más (a) interno para compatibilidad con serializadores que consultan la información; sin remote solo hace falta el caso local. Documentar que el valor se pierde al cambiar de hilo
- **Filas afectadas:** AC-SER-001.3; AC-SER-007.1; AC-SER-007.2
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-4 — Verificación serialize-messages/serialize-creators con mensajes dinámicos y Props como closures

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatch.scala:171-223; Children.scala:280-309; SerializeSpec.scala:308-362; reference.conf:129-140
- **Alternativas:** (a) verificar solo tipos registrados en el registry y fallar el resto como Pekko falla lo no enlazado; (b) verificación opt-in por tipo con un marker NoSerializationVerificationNeeded invertido; (c) no portar serialize-creators (Props de Rust son closures sin args reflectivos) pero sí serialize-messages
- **Recomendación:** (a) para serialize-messages y (c) para serialize-creators, con `Props::with_serializable_args` opcional si se quiere paridad; la clave se acepta siempre. Los tests que dependen de serialize-messages=on (varios specs de Pekko) funcionarán con mensajes registrados
- **Filas afectadas:** AC-SER-008.1; AC-SER-008.2
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-5 — Serializadores asíncronos y camino síncrono bloqueante con warning

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AsyncSerializer.scala:52-71 (Await.result Duration.Inf + warning); AsyncSerializeSpec.scala:111-137; Q-X-4
- **Alternativas:** (a) Futures como async fn y camino síncrono con block_in_place/hilo dedicado + warning idéntico; (b) camino síncrono devuelve Err("Async serializer called synchronously") en vez de bloquear; (c) sin camino síncrono
- **Recomendación:** (a): preserva el contrato observable (warning de 1 línea y resultado) y documenta que bloquear dentro del runtime requiere runtime multihilo; CompletionStage no se porta
- **Filas afectadas:** AC-SER-006
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-6 — Alcance de SER en esta fase: serialización local, claves solo-remote y detección de ciclos de extensión

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** inventario AC-SER-001 nota "Q: alcance"; reference.conf:852-880 (max-decompressed-size y protobuf allowed-classes los consumen remote/cluster); ActorSystem.scala:1167-1192 (espera creation-timeout); grep sin consumidores de allowed-classes en pekko-actor
- **Alternativas:** (a) Portar toda la serialización local ahora (la necesitan serialize-messages y Persistence/Remote luego); (b) diferir SER hasta que exista un consumidor; (c) portar solo el SPI y los primitivos
- **Recomendación:** (a) ordenada tras EXT; las claves protobuf se aceptan sin efecto; detectar ciclo de inicialización de extensión de inmediato (error con el mismo texto) en vez de esperar creation-timeout, anotando la diferencia de latencia
- **Filas afectadas:** AC-SER-010; AC-SER-012; AC-SER-014
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SUP-1 — Modelo de errores para deciders: jerarquía Exception/Error/Throwable y tipos de fallos del ciclo de vida

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:210-223,246-301 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:166-295
- **Alternativas:** (a) enum Failure { Init, Killed, DeathPact, PreRestart, PostRestart, User(Box<dyn Error>), Panic } con downcast para makeDecider / (b) Box<dyn Error> con downcast_ref / (c) códigos
- **Recomendación:** (a); Q-X-5 decide si Panic se mapea a Escalate (equivalente de Error/Throwable) o a Restart; el decider por defecto debe reproducir Stop para Init/Killed/DeathPact, Restart para User y Escalate para Panic si se acepta
- **Filas afectadas:** AC-SUP-004 ; AC-SUP-012 ; AC-SUP-017
- **Unidades afectadas:** SUP
- **Origen:** classic-cell

### Q-SUP-2 — Aritmética de la ventana de reinicios (withinTimeRange.toMillis.toInt y System.nanoTime)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:58-90,303-309,550
- **Alternativas:** (a) usar u64 de milisegundos con Instant monótono, saturando en lugar de truncar a Int / (b) replicar el truncamiento a i32 / (c) Duration nativa
- **Recomendación:** (a): diferencia observable sólo para ventanas mayores a ~24.8 días (Int overflow); la ventana empieza en la primera solicitud, no se desliza, y la comparación es "<= window"
- **Filas afectadas:** AC-SUP-006.1 ; AC-SUP-006.2
- **Unidades afectadas:** SUP
- **Origen:** classic-cell

### Q-SYS-1 — ¿Drop del último handle de ActorSystem termina el sistema? (close/AutoCloseable)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Pekko: sólo terminate()/close() explícitos o Using (ActorSystem.scala:1106-1110; ActorSystemScala213PlusSpec.scala:29-35); Drop no existe en JVM.
- **Alternativas:** (a) sin Drop, `close_blocking`/`terminate` explícitos; (b) Drop termina de forma no bloqueante; (c) guarda opcional `SystemGuard`
- **Recomendación:** (a) más guarda opcional (c) para tests; ninguna semántica implícita de Drop
- **Filas afectadas:** AC-SYS-014
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-2 — Mapeo de error fatal JVM (OOM/StackOverflow/Error) a Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:858-906: handler distingue NonFatal de fatal; jvm-exit-on-fatal-error=on (reference.conf:90) hace System.exit(-1). En Rust abort/OOM no es capturable y un panic en tarea de actor es un fallo normal (Q-X-5).
- **Alternativas:** (a) fatal = panic fuera de actores + señal; (b) fatal = sólo `std::process::abort` (nada que manejar); (c) política configurable `FatalErrorPolicy`
- **Recomendación:** (c) con default ExitProcess para panics en hilos propios del sistema; documentar que OOM/stack overflow no son capturables
- **Filas afectadas:** AC-SYS-015
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-3 — Filas JVM-específicas: ManifestInfo (mixed versions) y Main

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ManifestInfo.scala + ActorSystem.scala:1069; Main.scala:32-60 (deprecada). No hay MANIFEST ni classpath en Rust.
- **Alternativas:** (a) equivalente: comprobación de versión en compilación + constante `VERSION`; (b) sin equivalente (requiere exclusión explícita del Owner); Main: función utilitaria tras feature `deprecated`
- **Recomendación:** (a); la exclusión sólo si el Owner la autoriza
- **Filas afectadas:** AC-SYS-020 ; AC-SYS-021
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-4 — Protocolo RegisterTerminationHook sin remote/cluster

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Sólo lo usa RemoteActorRefProvider (fuera de alcance) y no hay test en pekko-actor (grep RegisterTerminationHook). ActorRefProvider.scala:295-374.
- **Alternativas:** (a) implementar el protocolo completo con tests propios derivados del código; (b) dejar el hook sin consumidores y sin test; (c) marcar como excluido (requiere autorización)
- **Recomendación:** (a): es parte del comportamiento de /system y lo necesitarán extensiones; sin autorización de exclusión
- **Filas afectadas:** AC-SYS-003.2 ; AC-SYS-002
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-5 — Redacción de datos sensibles en Settings.toString

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:488-491; Helpers.scala:180-192: sin valores de variables de entorno y reemplazo de `user.name` por `<username>`; ActorSystemSpec.scala:395-411 prueba con `${user.home}` (propiedad Java).
- **Alternativas:** (a) igual: no mostrar valores de entorno y redactar el usuario del proceso; (b) redactar además el home
- **Recomendación:** (a); el spec porteado usará `${HOME}`/`USER` como equivalente de las propiedades Java
- **Filas afectadas:** AC-SYS-022
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-T-ADP-2 — Que reglas de ActorAdapter sobreviven bajo runtime tipado nativo (Q-X-8 b)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:87-362
- **Alternativas:** (a) mantener las 4 familias (despacho, resultado, hooks, supervision de hijos); (b) re-especificarlas en el modelo nativo y validar solo contra goldens
- **Recomendación:** (a) como lista de reglas verificables
- **Filas afectadas:** AT-ADP-006.1;AT-ADP-006.2;AT-ADP-006.3;AT-ADP-006.4
- **Unidades afectadas:** ADP
- **Origen:** typed-extended

### Q-T-ADP-3 — Semantica de rethrowTypedFailure y supervision Stop por defecto del tipado spawneado desde clasico

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:168-172,236-261; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:57-77
- **Alternativas:** (a) conservar ambos (Stop por defecto, FailedBehavior detiene en silencio); (b) unificar con el default restart del clasico
- **Recomendación:** (a): afirmado por ClassicSupervisingTypedSpec
- **Filas afectadas:** AT-ADP-002;AT-ADP-006.2;AT-ADP-006.4
- **Unidades afectadas:** ADP
- **Origen:** typed-extended

### Q-T-ASK-1 — Taxonomia de errores de ask (TimeoutException vs error del destino) y conservacion del texto

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:148-157; actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:729-735; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:80-81,105-106
- **Alternativas:** (a) enum AskError con variantes y mensajes textuales identicos a Pekko; (b) un unico Timeout(String); (c) std::io::Error
- **Recomendación:** (a): conserva los prefijos "Ask timed out on" y "had already been terminated." que los tests afirman
- **Filas afectadas:** AT-ASK-001;AT-ASK-002
- **Unidades afectadas:** ASK
- **Origen:** typed-extended

### Q-T-ASK-2 — Conservar o no la restriccion de ask a InternalRecipientRef

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:120-127
- **Alternativas:** (a) permitir ask sobre cualquier RecipientRef; (b) replicar IllegalStateException con el mismo texto
- **Recomendación:** (a): la restriccion es transitoria segun su propio mensaje; sin test que la afirme
- **Filas afectadas:** AT-ASK-004
- **Unidades afectadas:** ASK
- **Origen:** typed-extended

### Q-T-BEH-2 — Colapsar receivePartial/receiveMessagePartial/transformMessages en cierres con Unhandled

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PartialFunction no existe en Rust; Scala usa applyOrElse a Behaviors.unhandled (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:157-171) y actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:238-243 para transformMessages.
- **Alternativas:** (a) un solo receive con match que devuelve Behavior::Unhandled en el brazo _; transform_messages con Option<I>; (b) mantener variantes *_partial como alias de closures que devuelven Option<Behavior>.
- **Recomendación:** (a) y documentar el alias (b) solo si el Owner quiere paridad de nombres.
- **Filas afectadas:** AT-BEH-008.1; AT-BEH-008.2; AT-BEH-008.3; AT-BEH-016
- **Unidades afectadas:** BEH
- **Origen:** typed-core

### Q-T-BEH-3 — Filtro por clase de interceptores (interceptMessageClass) y isSame

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** InterceptorImpl.receive usa interceptMessageClass.isAssignableFrom(msg.getClass) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:90-99); monitor, logMessages, withMdc y transformMessages usan ClassTag (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:206, 214, 284); TransformMessagesInterceptor pasa mensajes de otra clase al interno con cast sin chequeo (InterceptorImpl.scala:97).
- **Alternativas:** (a) funcion de proyeccion fn(&Outer)->Option<Inner> mas TypeId para isSame; (b) enums de protocolo exterior con variante para mensajes ajenos; (c) mantener Box<dyn Any> y downcast (alinea con Q-X-3).
- **Recomendación:** (a) con isSame por TypeId y valor; los mensajes ajenos solo existen si el protocolo exterior es un enum (b); monitor exige M: Clone.
- **Filas afectadas:** AT-BEH-013; AT-BEH-014; AT-BEH-015; AT-BEH-016; AT-BEH-017; AT-BEH-018; AT-BEH-019
- **Unidades afectadas:** BEH
- **Origen:** typed-core

### Q-T-BEH-4 — narrow de Behavior y de ActorRef (contravarianza) y su identidad

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:56 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:29-31: narrow devuelve this, por lo que el ref estrechado es == al original; en Rust requiere envolver con From.
- **Alternativas:** (a) narrow como adaptador explicito (closure From) cuyo path/uid y Eq coinciden con el original; (b) mismo objeto con tipo borrado (Box<dyn Any>) y narrow sin costo (alinea con Q-X-3); (c) no portar narrow y pedir enums de protocolo.
- **Recomendación:** (a) con Eq/Hash/Ord por path+uid independientes del tipo de mensaje.
- **Filas afectadas:** AT-BEH-001; AT-BEH-022; AT-REF-001; AT-REF-003; AT-REF-005
- **Unidades afectadas:** BEH; REF
- **Origen:** typed-core

### Q-T-BEH-5 — AbstractBehavior en Rust: contexto por parametro y desaparicion de la clase de error 'wrong ActorContext'

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:48-51, 82-88; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:188-277; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:46-51.
- **Alternativas:** (a) trait ActorBehavior<M> con on_message(&mut self, ctx, msg) y fabrica Fn por supervise, sin guardar el contexto; (b) replicar el contexto en el struct y los errores en runtime.
- **Recomendación:** (a). Divergencia de forma aceptable porque elimina IllegalStateException de contexto equivocado y el rechazo de supervise(AbstractBehavior).
- **Filas afectadas:** AT-BEH-010; AT-SUP-013; AT-CTX-016
- **Unidades afectadas:** BEH; SUP; CTX
- **Origen:** typed-core

### Q-T-BEH-6 — MDC y logMessages sobre backend de logging (tracing) en vez de MDC thread-local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/WithMdcBehaviorInterceptor.scala:44-117 usa org.slf4j.MDC (thread-local); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:340-481.
- **Alternativas:** (a) span de tracing por mensaje con los campos del MDC y clave pekkoSource/pekkoAddress/sourceActorSystem/pekkoTags; (b) task-local propio; (c) no portar MDC.
- **Recomendación:** (a). Coordinar con el grupo LOG (AT-LOG-002) y Q-X-7.
- **Filas afectadas:** AT-BEH-018; AT-BEH-019
- **Unidades afectadas:** BEH
- **Origen:** typed-core

### Q-T-CTX-1 — messageAdapter: coincidencia por tipo exacto y comportamiento de watch/ask sobre el ref adaptador (FunctionRef)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:306-331 (isAssignableFrom y orden inverso de registro; ver tambien actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:182-196); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:117-209.
- **Alternativas:** (a) coincidencia por TypeId exacto, uno por TypeId, mas reciente primero; (b) emular subtipado con una funcion de coincidencia fn(&dyn Any)->bool; en ambos: decidir si el ref adaptador es watchable.
- **Recomendación:** (a); documentar que el match por subclase no existe; el watch sobre el adaptador replica el comportamiento de FunctionRef solo si Q-X-8 elige la opcion (a) (capa clasica).
- **Filas afectadas:** AT-CTX-011; AT-CTX-018
- **Unidades afectadas:** CTX
- **Origen:** typed-core

### Q-T-CTX-2 — Chequeo de hilo del ActorContext: reemplazar ActorThreadSpec por garantias de compilacion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:338-376; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:50-277.
- **Alternativas:** (a) ActorContext<'a> solo existe como &mut en el handler, no es Send/'static y tests trybuild; (b) mantener chequeo en runtime con el mismo texto de error.
- **Recomendación:** (a). Mantener en el handoff una tabla de los 9 casos de ActorThreadSpec y como cada uno se vuelve compile-fail o no aplica.
- **Filas afectadas:** AT-CTX-016; AT-BEH-010
- **Unidades afectadas:** CTX; BEH
- **Origen:** typed-core

### Q-T-CTX-3 — Errores de uso del contexto (IllegalArgument/IllegalState): Result vs panic

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:77-104 (stop de no-hijo/self), actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:125-131 (watch/watchWith), actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:27 (despliegue remoto).
- **Alternativas:** (a) Result<_, ContextError> con variantes NotAChild, StopSelf, WatchConflict{previous,new}, SpawnError; (b) panic con los mismos textos; (c) hibrido: Result para spawn/stop, panic para violaciones de contrato.
- **Recomendación:** (a) conservando los textos de error como Display (ver Contratos AT-CTX-004/005).
- **Filas afectadas:** AT-CTX-002; AT-CTX-004; AT-CTX-005; AT-CTX-006
- **Unidades afectadas:** CTX
- **Origen:** typed-core

### Q-T-DLV-1 — Alcance de delivery y descomposicion antes de implementar

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:66-71; DS (specs 2673 lineas sin leer)
- **Alternativas:** (a) solo variante local con cola en memoria (como indica el inventario) y descomponer DLV-001/002/003 tras leer implementacion; (b) ademas DurableProducerQueue en memoria
- **Recomendación:** (a)+(b) opcional; fila sigue listed hasta leer las implementaciones
- **Filas afectadas:** AT-DLV-001;AT-DLV-002;AT-DLV-003
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DLV-2 — Implementacion durable del DurableProducerQueue

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:66-71; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueue.scala:41-80
- **Alternativas:** (a) solo el trait; (b) cola en memoria de referencia (para tests); (c) persistir con storage propio
- **Recomendación:** (b): permite validar el protocolo; la durable real es persistence-typed (OUT)
- **Filas afectadas:** AT-DLV-004
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DLV-3 — Equivalente del flight recorder JFR

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorFlightRecorder.scala; scala-jdk-9/jfr/Events.scala
- **Alternativas:** (a) eventos tracing con los mismos nombres; (b) no portar (propuesta de exclusion, requiere autorizacion)
- **Recomendación:** (a)
- **Filas afectadas:** AT-DLV-007
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DSP-1 — Orden de resolucion de Props: ultimo agregado gana

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:75-113; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/PropsSpec.scala:21-30
- **Alternativas:** (a) conservar lista con anteposicion; (b) builder con primer-gana
- **Recomendación:** (a): lo afirma el spec
- **Filas afectadas:** AT-DSP-001;AT-DSP-004
- **Unidades afectadas:** DSP
- **Origen:** typed-extended

### Q-T-DSP-2 — Mapeo de DispatcherSelector a ejecutores de Rust (hilos vs tareas tokio)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:67-135
- **Alternativas:** (a) nombre de runtime/pool; (b) hilo dedicado por actor
- **Recomendación:** depende de Q-X-4; aqui solo se fija que fromConfig/sameAsParent/blocking existan
- **Filas afectadas:** AT-DSP-002
- **Unidades afectadas:** DSP
- **Origen:** typed-extended

### Q-T-DSP-3 — Semantica de mailbox acotado lleno (dead letters) y default single-consumer

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:75-95
- **Alternativas:** (a) dead letters con evento; (b) backpressure
- **Recomendación:** (a): paridad con Pekko
- **Filas afectadas:** AT-DSP-003
- **Unidades afectadas:** DSP
- **Origen:** typed-extended

### Q-T-EVT-1 — Tres caminos distintos para publicar Dropped (classic eventStream, typed EventStream.Publish, deadLetters)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:120-124; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:99-102; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:96; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:736-743
- **Alternativas:** (a) un unico publish_dropped al event stream (Topic cambia de DeadLetter(Dropped) a Dropped); (b) preservar los tres caminos; (c) unificar y pedir al oraculo confirmar el efecto en Topic
- **Recomendación:** (c): validar con jar oficial y decidir; observable para suscriptores de DeadLetter/Dropped
- **Filas afectadas:** AT-EVT-004;AT-PS-001.2;AT-RT-004
- **Unidades afectadas:** EVT;PS;RT
- **Origen:** typed-extended

### Q-T-EXT-1 — Politica "skip" en extensions cuando la clase no resuelve

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ExtensionsSpec (no afirma el skip)
- **Alternativas:** (a) conservar log+skip para nombres no resolubles; (b) fallar siempre
- **Recomendación:** (a): paridad con el codigo; sin spec que lo afirme
- **Filas afectadas:** AT-EXT-002.2
- **Unidades afectadas:** EXT
- **Origen:** typed-extended

### Q-T-EXT-2 — Identidad de ExtensionId por referencia en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:131; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:155-163
- **Alternativas:** (a) puntero (Arc ptr_eq) / direccion de static; (b) TypeId + id numerico explicito
- **Recomendación:** (a)
- **Filas afectadas:** AT-EXT-001.1;AT-EXT-001.3
- **Unidades afectadas:** EXT
- **Origen:** typed-extended

### Q-T-EXT-3 — Relanzar la misma excepcion en cada acceso tras un fallo de creacion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:110-114,134
- **Alternativas:** (a) error cacheado clonable; (b) reintentar en cada acceso
- **Recomendación:** (a): lo dicen el codigo y el Scaladoc; el test solo comprueba que el sistema no arranca
- **Filas afectadas:** AT-EXT-001.2
- **Unidades afectadas:** EXT
- **Origen:** typed-extended

### Q-T-JAPI-1 — Fachada javadsl en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/*.scala (2353 lineas); actor-typed-tests/src/test/java
- **Alternativas:** (a) no portar: Rust tiene una API; las aserciones de scaladsl cubren la semantica (propuesta de exclusion, requiere autorizacion); (b) portar helpers equivalentes para usuarios que vengan de Java
- **Recomendación:** (a) con autorizacion explicita; la fila sigue en alcance hasta entonces
- **Filas afectadas:** AT-JAPI-001;AT-JAPI-002;AT-JAPI-003;AT-JAPI-004;AT-JAPI-005;AT-JAPI-006
- **Unidades afectadas:** JAPI
- **Origen:** typed-extended

### Q-T-LOG-1 — Fachada de logging: tracing vs log+kv, y nombres de clave MDC

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:24-27; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:160-188; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:76-129
- **Alternativas:** (a) tracing con span por mensaje y campos pekko_*; (b) log + kv; (c) trait propio con backend enchufable
- **Recomendación:** (a); mantener los nombres camelCase de Pekko como claves opcionales para compatibilidad de dashboards
- **Filas afectadas:** AT-LOG-001;AT-LOG-002;AT-LOG-005;AT-TK-006
- **Unidades afectadas:** LOG;TK
- **Origen:** typed-extended

### Q-T-LOG-2 — Deteccion del nombre del logger por stack (JVM-only)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/LoggerClass.scala:24-60; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:131-173
- **Alternativas:** (a) nombre = type_name del Behavior/AbstractBehavior mas externo; (b) obligatorio set_logger_name; (c) module_path! del call site
- **Recomendación:** (a)+(c): conserva el efecto observable "nombre = donde se definio el comportamiento"
- **Filas afectadas:** AT-LOG-001
- **Unidades afectadas:** LOG
- **Origen:** typed-extended

### Q-T-LOG-3 — pekko.use-slf4j / Slf4jLogger como configuracion en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/resources/reference.conf:56-68; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:335-370
- **Alternativas:** (a) aceptar la clave y mapearla a "activar subscriber de tracing"; (b) ignorarla; (c) excluir (propuesta de exclusion, requiere autorizacion del Owner)
- **Recomendación:** (a)
- **Filas afectadas:** AT-LOG-003
- **Unidades afectadas:** LOG
- **Origen:** typed-extended

### Q-T-LOG-4 — LoggerOps (Scala 2) subsumido por macros de tracing

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/package.scala:30-120
- **Alternativas:** (a) no portar: las macros de tracing resuelven el problema de varargs; (b) portar helpers equivalentes
- **Recomendación:** (a): propuesta de que no haga falta equivalente; la fila sigue en alcance hasta que el Owner decida
- **Filas afectadas:** AT-LOG-004
- **Unidades afectadas:** LOG
- **Origen:** typed-extended

### Q-T-RCP-1 — Igualdad de ServiceKey: (id, nombre de clase JVM) vs (id, TypeId de Rust)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKey.scala:41-43; actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:41-48
- **Alternativas:** (a) (id,TypeId) de Rust: distingue Vec<i32> de Vec<String>; (b) (id,type_name) imitando typeName; (c) solo id con chequeo en runtime
- **Recomendación:** (a): mas seguro; difiere de Pekko en claves con genericos del mismo borrado (no afirmado por tests)
- **Filas afectadas:** AT-RCP-002;AT-RCP-005.3;AT-SER-002
- **Unidades afectadas:** RCP;SER
- **Origen:** typed-extended

### Q-T-RCP-2 — Punto de extension del receptionist de cluster

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistImpl.scala:27-50; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:28-36
- **Alternativas:** (a) trait ReceptionistBehaviorProvider y registro explicito; (b) sin extension point (solo local)
- **Recomendación:** (a): costo bajo; cluster-typed queda fuera del modulo
- **Filas afectadas:** AT-RCP-001
- **Unidades afectadas:** RCP
- **Origen:** typed-extended

### Q-T-RCP-3 — Deregister de clave no registrada con actor que tiene otras claves notifica Listing sin cambios y responde Deregistered

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:195-218 (sin spec)
- **Alternativas:** (a) conservar; (b) no notificar cuando el estado no cambia
- **Recomendación:** (a) por paridad; tambien Register repetido notifica de nuevo
- **Filas afectadas:** AT-RCP-003;AT-RCP-004;AT-RCP-005.2
- **Unidades afectadas:** RCP
- **Origen:** typed-extended

### Q-T-RT-1 — Hashing consistente byte-compatible con Pekko

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:101-122 (usa pekko.routing.ConsistentHash de classic)
- **Alternativas:** (a) portar ConsistentHash identico (MurmurHash/rehash) para igualar nodos elegidos; (b) algoritmo propio con las mismas garantias (estabilidad), sin compatibilidad
- **Recomendación:** (b) salvo que se requiera interoperar con cluster Pekko (fuera del modulo)
- **Filas afectadas:** AT-RT-002.3
- **Unidades afectadas:** RT
- **Origen:** typed-extended

### Q-T-RT-2 — Comportamiento de selectRoutee sin routees (NPE/AIOOBE/IllegalArgument/IllegalState segun logica)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:54-62,91-93,109
- **Alternativas:** (a) Option/Result uniforme; (b) panic con el mismo texto solo para hashing
- **Recomendación:** (a): solo el caso de hashing esta afirmado por spec
- **Filas afectadas:** AT-RT-002.1;AT-RT-002.2;AT-RT-002.3
- **Unidades afectadas:** RT
- **Origen:** typed-extended

### Q-T-RT-3 — Reuso del Behavior de routees y orden de entrega del broadcast

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:86-90,99-101
- **Alternativas:** (a) Behavior Clone/Arc<dyn Fn> (AT-BEH-023, decision del grupo BEH); (b) factory explicito por routee
- **Recomendación:** (a), consistente con la decision de AT-BEH-023
- **Filas afectadas:** AT-RT-001;AT-RT-003;AT-RT-004;AT-RT-005
- **Unidades afectadas:** RT
- **Origen:** typed-extended

### Q-T-SER-1 — Estrategia de serializacion de mensajes y refs en un runtime local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/MiscMessageSerializer.scala:28-55; actor-typed/src/main/resources/reference.conf:38-53
- **Alternativas:** (a) trait Serializer con registro por tipo y los dos serializadores internos; (b) solo serde sin registro; (c) no portar mientras no haya remoting/chunking
- **Recomendación:** (a) minimo, necesario para chunking de delivery y SerializationTestKit; depende de AC-SER
- **Filas afectadas:** AT-SER-001;AT-SER-002;AT-DLV-005;AT-TK-009
- **Unidades afectadas:** SER;DLV;TK
- **Origen:** typed-extended

### Q-T-SER-2 — Formato de ServiceKey en el cable (manifest = nombre de clase JVM)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKeySerializer.scala:30-43
- **Alternativas:** (a) conservar manifest=type_name Rust (no interoperable con JVM); (b) formato propio
- **Recomendación:** (a)
- **Filas afectadas:** AT-SER-002
- **Unidades afectadas:** SER
- **Origen:** typed-extended

### Q-T-SER-3 — Serializacion Java de ActorRef (JVM-only)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:60-112
- **Alternativas:** (a) no portar (propuesta de exclusion, requiere autorizacion); (b) equivalente serde con resolver global
- **Recomendación:** (b) si Q-T-SER-1 elige (a)
- **Filas afectadas:** AT-SER-003
- **Unidades afectadas:** SER
- **Origen:** typed-extended

### Q-T-SIG-1 — Forma de Signal: enum no exhaustivo, ChildFailed como Terminated, igualdad asimetrica y senales internas

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:35-125: Terminated(r) == ChildFailed(r,c) es true y a la inversa false (lineas 90-93 vs 115-118); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:198-203 y PoisonPill.scala:30-37 extienden Signal.
- **Alternativas:** (a) enum Signal #[non_exhaustive] con Terminated, ChildFailed{cause}, y variantes internas privadas; as_terminated() para tratar ambos igual; igualdad simetrica por ref; (b) reproducir la asimetria; (c) Signal como trait object abierto.
- **Recomendación:** (a) con igualdad simetrica documentada como divergencia; las senales internas no se exportan.
- **Filas afectadas:** AT-SIG-001; AT-SIG-004; AT-SIG-005; AT-SIG-008; AT-SUP-017
- **Unidades afectadas:** SIG; SUP
- **Origen:** typed-core

### Q-T-SPV-1 — Spawn[T] existencial y borrado de tipo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:54-85
- **Alternativas:** (a) mensaje Spawn con Box<dyn Any> + downcast; (b) macro/closure generica con canal tipado por llamada; (c) SpawnProtocol especifico por tipo de hijo
- **Recomendación:** (b) si el API publico se mantiene; difiere de Pekko en que el ref devuelto queda tipado por llamada
- **Filas afectadas:** AT-SPV-001
- **Unidades afectadas:** SPV
- **Origen:** typed-extended

### Q-T-STASH-1 — Buffer interno de supervision que guarda mensajes y senales; UnstashException con Behavior en curso

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:210, 365-375 (StashBuffer[Any] con mensajes y senales); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:187-244, 286-287.
- **Alternativas:** (a) tipo interno Input<M> = Msg(M) | Sig(Signal) para el StashBuffer del supervisor y error Unstash{cause, behavior}; (b) dos buffers separados.
- **Recomendación:** (a) preservando el orden relativo de mensajes y senales.
- **Filas afectadas:** AT-SUP-009; AT-SUP-018; AT-STASH-006
- **Unidades afectadas:** SUP; STASH
- **Origen:** typed-core

### Q-T-SUP-1 — Selector de excepciones en supervise().on_failure: por tipo o por predicado

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:70-83, 146; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:82-94 (ClassTag.isAssignableFrom); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:139-147, 448-490. El eje Err/panic lo decide Q-X-5.
- **Alternativas:** (a) on_failure::<E: Error + 'static>(strategy) con downcast_ref sobre el error dinamico; (b) on_failure_if(|e| bool, strategy); (c) ambos.
- **Recomendación:** (c). 'Throwable' por defecto equivale a un selector que acepta todo.
- **Filas afectadas:** AT-SUP-002; AT-SUP-011
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-2 — Discrepancia doc/codigo en backoff: 'los mensajes se descartan' vs stash con Dropped

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Scaladoc dice 'During the back-off incoming messages are dropped' (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:64, 100); el codigo guarda en stash hasta stashCapacity y publica Dropped al llenarse (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:278-292); el spec lo afirma (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:882-910, 912-936).
- **Alternativas:** (a) seguir el codigo y el spec (stash + Dropped), registrar la discrepancia; (b) seguir el Scaladoc (descartar siempre).
- **Recomendación:** (a).
- **Filas afectadas:** AT-SUP-007.1; AT-SUP-009
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-4 — Setup que falla durante un restart ilimitado (bucle o StackOverflowError)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:294-311 (no reinicia si falla el primer arranque con restarts ilimitados), 392-419 (restartCompleted captura con handleException y puede reintentar); antecedente goPekko S-3 (solo contexto, no vinculante).
- **Alternativas:** (a) reproducir el comportamiento de Pekko (reintento sin limite si el setup falla en un restart); (b) divergir: limitar reintentos de arranque tras restart o tratar la falla de arranque como fatal.
- **Recomendación:** (b) como divergencia marcada solo si el Owner la acepta; hasta entonces (a).
- **Filas afectadas:** AT-SUP-005
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-5 — Resume con falla de arranque: envoltura accidental ActorInitializationException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:169-178 (ResumeSupervisor devuelve same) + actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:240-243 (validateAsInitial rechaza same); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1107-1115 (se loguean TestException y ActorInitializationException). Antecedente goPekko S-5.
- **Alternativas:** (a) reproducir (el actor para y el padre ve ActorInitializationException con IllegalArgumentException como causa); (b) tipar la falla como InitializationFailed{cause} directamente.
- **Recomendación:** (b) conservando el texto y la parada del actor (la envoltura exacta es accidental).
- **Filas afectadas:** AT-SUP-004
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-6 — ScheduledRestart y ResetRestartCount como senales opacas que llegan al Behavior de usuario

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:198-203, 220-250; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1032-1076 (el spec afirma que la senal llega). Antecedente goPekko S-4.
- **Alternativas:** (a) seguir Pekko: variante opaca de Signal con id de duenio que se propaga cuando no coincide; (b) consumirlas siempre en el supervisor duenio (no visibles al usuario).
- **Recomendación:** (a) porque el spec lo afirma; ocultar el tipo concreto.
- **Filas afectadas:** AT-SUP-017; AT-SUP-007.2
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-7 — Padre/hijo clasico: decider clasico y excepcion agotada que sube al padre

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:55-58, 232-266; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/TypedSupervisingClassicSpec.scala:61-72; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:335-338 (throw t al agotar withLimit). Depende de Q-X-8. Antecedentes goPekko S-2 y S-6.
- **Alternativas:** (a) si Q-X-8 = capa clasica: portar defaultDecider (ActorInitializationException/ActorKilled/DeathPact paran, resto reinicia) y log ERROR; (b) si Q-X-8 = nativo: solo semantica Stop/ChildFailed y sin hijos clasicos.
- **Recomendación:** Seguir Q-X-8; (a) tal cual si hay capa clasica.
- **Filas afectadas:** AT-SUP-006; AT-SUP-020; AT-SUP-021
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-8 — Aleatoriedad e instante inyectables en backoff

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:185-196 (ThreadLocalRandom), 430-441 (Deadline.now); tests con tiempo real (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:912-958).
- **Alternativas:** (a) trait BackoffJitter y Clock inyectables con implementaciones reales y de test; (b) tokio::time::pause y semilla fija.
- **Recomendación:** (a) alineado con el reloj inyectable de Q-T-TMR-4.
- **Filas afectadas:** AT-SUP-007.1; AT-SUP-006
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SYS-1 — Accesores JVM-only del sistema: threadFactory, dynamicAccess, printTree, logConfiguration

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:56-109, 179-185; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:155-165. Los inventarios viejos proponian excluir; la exclusion no esta autorizada.
- **Alternativas:** (a) ThreadFactory -> std::thread::Builder/hook de creacion de hilos, dynamicAccess -> registro de fabricas (Q-X-2), printTree -> arbol textual propio, logConfiguration -> log de config redactada; (b) proponer exclusion formal al Owner.
- **Recomendación:** (a); mantener la fila en alcance con la diferencia observable documentada.
- **Filas afectadas:** AT-SYS-007.3; AT-SYS-007.4
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-SYS-2 — Redaccion de configuracion en Settings.toString y logConfiguration

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:343-346; actor/src/main/scala/org/apache/pekko/util/Helpers.scala:183-192; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorSystemSpec.scala:30-46 (no filtrar user.name/user.home; Pekko PR 771).
- **Alternativas:** (a) misma regla: ocultar valores de variables de entorno y reemplazar user.name por <username>; (b) ocultar ademas claves con nombres sensibles (password, secret); (c) no redactar.
- **Recomendación:** (a) como minimo; (b) opcional como mejora marcada.
- **Filas afectadas:** AT-SYS-012; AT-SYS-006
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-SYS-3 — Buffer de arranque del guardian: limite de 1000 y StashOverflowException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:35, 49-51; ActorSystem.scala:319-322 envia Start antes de devolver el sistema.
- **Alternativas:** (a) construir el sistema completo antes de arrancar el guardian y omitir el buffer; (b) reproducir el limite de 1000 y el error; (c) buffer sin limite.
- **Recomendación:** (a) manteniendo el contrato: ningun mensaje temprano se pierde; si se conserva buffer, reproducir (b).
- **Filas afectadas:** AT-SYS-003
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-SYS-4 — close() bloqueante y Drop del ActorSystem

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:148-166 (close desde 1.3.0 bloquea hasta terminar o TimeoutException); actor-typed-tests/src/test/scala-2.13+/org/apache/pekko/actor/typed/ActorSystemScala213PlusSpec.scala:28-34; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:46-53.
- **Alternativas:** (a) close() bloqueante explicito y Drop que dispara terminate sin bloquear; (b) Drop bloqueante con runtime.block_on (riesgo de deadlock en async); (c) solo terminate().await.
- **Recomendación:** (a). Documentar que try-with-resources/Using equivale a close() explicito.
- **Filas afectadas:** AT-SYS-005
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-TK-1 — API de TestProbe/ActorTestKit: bloqueante vs async

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:160-252; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:238-272
- **Alternativas:** (a) async fn sobre tokio con timeouts dilatados; (b) bloqueante sobre runtime multi-hilo; (c) ambas
- **Recomendación:** (a) o (c); depende de Q-X-4; los mensajes de AssertionError se conservan
- **Filas afectadas:** AT-TK-001.2;AT-TK-001.3;AT-TK-002.1;AT-TK-002.2;AT-TK-002.3;AT-TK-002.4
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-2 — Captura de logs (LoggingTestKit) sin Logback

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestAppender.scala
- **Alternativas:** (a) tracing Layer de captura; (b) log crate con appender de prueba
- **Recomendación:** (a); depende de Q-T-LOG-1
- **Filas afectadas:** AT-TK-006
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-3 — Integraciones JUnit/ScalaTest y nombre del sistema por stack

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ScalaTestWithActorTestKit.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:82-91
- **Alternativas:** (a) helper/macro propio de Rust; (b) no portar JUnit (propuesta de exclusion, requiere autorizacion)
- **Recomendación:** (a)
- **Filas afectadas:** AT-TK-001.1;AT-TK-009
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-4 — BehaviorTestKit requiere interprete de Behavior sin runtime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala
- **Alternativas:** (a) interprete puro de Behavior reutilizado por el runtime (necesario con Q-X-8 b); (b) montar sobre el runtime con un ejecutor controlado
- **Recomendación:** (a)
- **Filas afectadas:** AT-TK-003;AT-TK-004.1;AT-TK-004.2;AT-TK-005
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TMR-1 — Tipo de la clave del timer (Any en Scala)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:50-258 (key: Any); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:98 (Map[Any, Timer]).
- **Alternativas:** (a) TimerScheduler<M, K: Hash+Eq+Clone+Send>; (b) clave Box<dyn Any> con TypeId+Eq dinamico; (c) enum de claves por actor.
- **Recomendación:** (a); las variantes sin clave usan el mensaje como clave (M: Hash+Eq) o un id generado.
- **Filas afectadas:** AT-TMR-001; AT-TMR-002; AT-BEH-020
- **Unidades afectadas:** TMR; BEH
- **Origen:** typed-core

### Q-T-TMR-2 — NotInfluenceReceiveTimeout: marker de clase vs flag explicito

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:126-130 (instanceOf); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:118-129.
- **Alternativas:** (a) flag no_influence_receive_timeout al iniciar el timer o trait NotInfluenceReceiveTimeout en M; (b) ignorar el marker.
- **Recomendación:** (a) flag explicito (el trait solo si M es un enum con metodo).
- **Filas afectadas:** AT-TMR-007.2; AT-CTX-008
- **Unidades afectadas:** TMR; CTX
- **Origen:** typed-core

### Q-T-TMR-3 — startPeriodicTimer deprecado: alias y semantica (la sobrecarga Java difiere de la Scala)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:81-84 (Java: FixedDelay) vs 113-114 (Scala: FixedRate); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:224-229 (doc: semantica de AtFixedRate). No se puede excluir sin autorizacion.
- **Alternativas:** (a) alias deprecado con semantica fixed rate (Scaladoc y Scala); (b) alias con la semantica de la sobrecarga Java (fixed delay); (c) proponer exclusion formal.
- **Recomendación:** (a) y documentar la divergencia Java como bug conocido de Pekko.
- **Filas afectadas:** AT-TMR-005
- **Unidades afectadas:** TMR
- **Origen:** typed-core

### Q-T-TMR-4 — Reloj y scheduler inyectables (ManualTime) para timers, backoff, limites y uptime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AGENTS.md/docs/inventory/actor-typed.md: necesidad de reloj inyectable; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala (TimingTest); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:430-441; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:70-78.
- **Alternativas:** (a) trait Clock/Scheduler en el sistema con implementacion tokio y ManualTime; (b) tokio::time::pause().
- **Recomendación:** (a) porque tambien sirve al grupo testkit.
- **Filas afectadas:** AT-TMR-003; AT-TMR-004; AT-SYS-009; AT-SYS-007.2; AT-SUP-006
- **Unidades afectadas:** TMR; SYS; SUP
- **Origen:** typed-core

### Q-TK-2 — TestActorRef/TestFSMRef y acceso a `underlyingActor` frente al modelo de propiedad de Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestActorRef.scala:34-263 y TestFSMRef.scala:44-147 (sin leer).
- **Alternativas:** (a) `TestActorRef<A>` que devuelve un `MutexGuard<A>` sobre el actor con dispatcher inline; (b) sólo `receive` síncrono sin acceso; (c) sin equivalente (exclusión requiere autorización)
- **Recomendación:** (a) tras leer TestActorRefSpec
- **Filas afectadas:** AC-TK-006.1 ; AC-TK-007
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-3 — EventFilter[T] por tipo de causa en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestEventListener.scala:324-336 (ErrorFilter usa throwable.isInstance(cause)); TestEventListenerSpec.scala:36-64.
- **Alternativas:** (a) filtrar por `TypeId` del error con downcast; (b) filtrar por predicado `Fn(&dyn Error) -> bool`; (c) ambos
- **Recomendación:** (c); EventFilter[NoCause] = cualquier Error
- **Filas afectadas:** AC-TK-005.1
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-4 — API bloqueante vs asíncrona de TestKit (awaitCond, within, expectMsg, timePasses)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestKit.scala:298-443 usa Thread.sleep y polling; ExplicitlyTriggeredScheduler.scala:67-81 duerme 100 ms reales.
- **Alternativas:** (a) API bloqueante (std::thread) igual que Pekko; (b) sólo async; (c) ambas, async como núcleo y bloqueante como envoltorio
- **Recomendación:** (c); relacionada con Q-X-4
- **Filas afectadas:** AC-TK-001.1 ; AC-TK-001.2 ; AC-TK-001.5 ; AC-TK-005.2 ; AC-TK-008 ; AC-TK-013
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-5 — Origen del timefactor en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestKitExtension.scala:35-37 sólo lee `pekko.test.timefactor`; no hay variable de entorno (corrige el inventario).
- **Alternativas:** (a) sólo clave de config; (b) además una variable de entorno `RAKKA_TEST_TIMEFACTOR` como extensión sin equivalente en Pekko
- **Recomendación:** (a) por defecto; (b) sólo si el Owner la autoriza como diferencia documentada
- **Filas afectadas:** AC-TK-004
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-6 — Harness de tests para sustituir PekkoSpec/Coroner/ScalaTest

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PekkoSpec.scala:38-148; TestKitUtils.scala:20-70; Coroner.scala sin leer.
- **Alternativas:** (a) crate `rakka-testkit` con un `TestSystem` por test; (b) macros `#[rakka::test]`; (c) helpers en `tests/common`
- **Recomendación:** (a) con (b) opcional; definir nombres de sistema desde el nombre del test
- **Filas afectadas:** AC-TK-012.1 ; AC-TK-012.2 ; AC-TK-014
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-UTIL-2 — toString de Timeout conserva la unidad

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-2): Pekko imprime `Timeout(100 milliseconds)` o `Timeout(0 days)`: el `FiniteDuration` recuerda la unidad con que se construyó, aunque `Timeout(1000.millis) == Timeout(1.second)`. `std::time::Duration` no la recuerda.
- **Alternativas:** (a) `Debug` derivado (`Timeout(100ms)`) sin prometer el formato de Pekko (lo implementado); (b) guardar la unidad junto con el valor (rompe la igualdad por valor o exige un `PartialEq` manual).
- **Recomendación:** (a): sólo aparece en logs. Sin respuesta del Owner la fila AC-UTIL-001 queda `partial`.
- **Filas afectadas:** AC-UTIL-001
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-3 — null y mensajes diferidos en PekkoException y requiring

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-3): `new PekkoException(null)` tiene `getMessage == null` y se imprime sin `: `; `requiring(cond, null)` da `requirement failed: null`. Con `&str`/`impl Display` no se puede expresar `null`. Igual que Q-PATH-4.
- **Alternativas:** (a) aceptarlo como diferencia de lenguaje (única alternativa registrada en el plan).
- **Recomendación:** (a), igual que Q-PATH-4. Sin respuesta del Owner las filas afectadas quedan `partial`.
- **Filas afectadas:** AC-UTIL-002.3; AC-UTIL-004.1; AC-UTIL-004.2
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-4 — Jerarquía de excepciones y OnlyCauseStackTrace

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-4): (i) `ConfigurationException` es subclase de `PekkoException`: un `catch PekkoException` la atrapa; con tipos separados no. (ii) `OnlyCauseStackTrace` sólo cambia qué stack trace imprime la JVM; los errores de Rust no llevan stack trace.
- **Alternativas:** (a) tipos separados y un `From<ConfigurationException> for PekkoException` (pierde `java_class`) cuando algún consumidor lo pida; (b) un único `enum PekkoError`; (c) un trait común. Para (ii) el plan propone excluir `OnlyCauseStackTrace` y documentarlo.
- **Recomendación:** (i) (a); (ii) excluirlo y documentarlo. La exclusión de `OnlyCauseStackTrace` requiere autorización explícita del Owner (A-5). Sin respuesta las filas quedan `partial`.
- **Filas afectadas:** AC-UTIL-004.2; AC-UTIL-004.3
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-5 — Done propio o ()

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-5), inventario AC-UTIL-003.1: se implementó `pub struct Done;`; la fila queda `partial` hasta decidir. Java expone `getInstance()` y `done()`.
- **Alternativas:** (a) `pub struct Done;` (lo implementado); (b) `()`.
- **Recomendación:** Sin recomendación registrada en el plan: lo implementado (a) es un estado, no una decisión aceptada. La fila AC-UTIL-003.1 queda `partial`.
- **Filas afectadas:** AC-UTIL-003.1
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-7 — toRootLowerCase: tabla de minúsculas de Unicode distinta entre JDK y Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** util/Helpers.scala:40 usa String.toLowerCase(Locale.ROOT); el oráculo corre con JDK 11 (Unicode 10) y Rust 1.96 usa Unicode más nuevo (plan AC-UTIL, decisión 6).
- **Alternativas:** (a) usar `str::to_lowercase` y declarar la diferencia para caracteres posteriores a Unicode 10; (b) tabla propia congelada a Unicode 10; (c) tabla del JDK 21 (referencia del port de URI).
- **Recomendación:** (a): los consumidores (log-dead-letters, niveles de log) sólo reciben ASCII en la práctica.
- **Filas afectadas:** AC-UTIL-002.2
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-8 — Semántica de cadenas por unidades UTF-16 (Glob, makePattern, WildcardIndex)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** util/Glob.scala:43-72 recorre `charAt` (unidades UTF-16): `?` consume una unidad, no un punto de código; Helpers.makePattern usa `.` de java.util.regex que no cruza terminadores de línea (Helpers.scala:44-45) mientras Glob.matches sí; WildcardIndex.scala:87-89 elige entre `prefix*` solapados según el orden de iteración de un HashMap de Scala. GlobSpec.scala:60-73 sólo compara sobre {a,b,*,?}.
- **Alternativas:** (a) operar sobre `&[u16]` (idéntico a Java, incluidos pares sustitutos); (b) sobre `char` de Rust (distinto para caracteres fuera del BMP); (c) igual que (a) y fijar el orden de `prefix*` por inserción; documentar la diferencia de saltos de línea.
- **Recomendación:** (c): los nombres válidos de actor son ASCII/%XX (ActorPath.scala:127-148) y los patrones de selección vienen del usuario; la diferencia de saltos de línea debe decidirla el Owner.
- **Filas afectadas:** AC-UTIL-009, AC-UTIL-016.6, AC-UTIL-023, AC-CFG-012.2
- **Unidades afectadas:** UTIL, CFG
- **Origen:** classic-foundation

### Q-UTIL-9 — PrettyDuration: `%.4g` de java.util.Formatter y representación de infinitos/indefinido

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** util/PrettyDuration.scala:51-55,57-59: `"%.{p}g"` con Locale.ROOT; el %g de Java usa notación científica cuando el valor redondeado >= 10^precisión (p. ej. 10000 d) y difiere del %g de C/Rust; Duration.Inf/MinusInf/Undefined son valores de scala.concurrent.duration.Duration, que `std::time::Duration` no tiene. PrettyDurationSpec.scala:25-57 fija 10 casos + 3 sin números.
- **Alternativas:** (a) implementar el %g de Java a mano y un enum `PekkoDuration { Finite, Inf, MinusInf, Undefined }`; (b) limitar a los casos del spec y documentar el resto; (c) delegar a un crate de formateo.
- **Recomendación:** (a), validado contra un escenario del oráculo con valores grandes y pequeños.
- **Filas afectadas:** AC-UTIL-016.5
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-10 — Estructuras concurrentes e identidad (Index, Switch, TokenBucket, colas, identityHashCode, timestamp)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Index.scala:23-199 (ConcurrentSkipListSet por clave, iteración débilmente consistente, isEmpty con falsos negativos), LockUtil.scala:29-155 (synchronized), TokenBucket.scala:19-91 (aritmética con wrap-around), Helpers.scala:47-69 (identityHashCode), Helpers.scala:108-113 (zona horaria del sistema).
- **Alternativas:** (a) Mutex + estructuras estándar, iteración sobre instantánea; (b) mapas concurrentes lock-free; (c) fijar una API mínima por consumidor real y diferir el resto.
- **Recomendación:** (a) con tests portados 1:1 (IndexSpec, SwitchSpec, TokenBucketSpec); id de identidad = dirección del Arc o contador.
- **Filas afectadas:** AC-UTIL-007, 008, 010, 011, 012, 013, 015.1, 024, 025
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-11 — Elementos propios de la JVM o de Scala sin equivalente natural en Rust (propuesta de equivalente; no exclusión)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** util/LockUtil.scala:19-27 (ReentrantGuard, deprecado), util/TypedMultiMap.scala, util/JavaVersion.scala, util/Reflect.scala, util/LineNumbers.scala, util/BoxedType.scala, java/org/apache/pekko/annotation/*.java, java/org/apache/pekko/util/Unsafe.java, mima-filters, shims de Scala 2.12/2.13/3 (scala-2.13/util/*Converters.scala), cachés de cluster (FrequencySketch...).
- **Alternativas:** (a) cada fila se mantiene en alcance con un equivalente Rust documentado (atómicos, `pub(crate)`, traits sellados, semver) y la diferencia observable anotada; (b) proponer al Owner excluirlas explícitamente fila por fila.
- **Recomendación:** (a) ahora; (b) sólo con aprobación explícita del Owner (nada se excluye sin autorización).
- **Filas afectadas:** AC-UTIL-014, 015.2, 016.*, 017.*, 018.*, 019, 020, 021, 022
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-13 — Helpers.isWindows deriva de `os.name` contiene "win"

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** util/Helpers.scala:42; consumidores: LightArrayRevolverScheduler.scala:67,120, io/Tcp.scala:640, io/dns/DnsSettings.scala:113.
- **Alternativas:** (a) `cfg!(windows)` (comportamiento esperado); (b) reproducir la heurística de subcadena sobre `std::env::consts::OS`.
- **Recomendación:** (a).
- **Filas afectadas:** AC-UTIL-026
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

## 6. Respuestas del Owner por unidad

Decisiones por unidad que el Owner ya resolvió (las transversales figuran en la sección 4).

### Q-UTIL-1 — Negativos y rango de Timeout

- **Estado:** rechazada
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Preguntas abiertas, Q-UTIL-1): `FiniteDuration` es un `Long` de nanosegundos con signo entre -(2^63-1) y 2^63-1; `Timeout(-5.millis)` es válido y `ask` lo rechaza después (AC-ASK-001, ≤0). `std::time::Duration` no tiene negativos y llega a ~584 mil años. `Timeout.create(ofSeconds(Long.MAX_VALUE))` lanza `ArithmeticException: long overflow` en Pekko y tampoco se reproduce.
- **Alternativas:** (a) quedarse con `std::time::Duration` y que `ask` reciba un `Timeout` siempre positivo, documentando que `Timeout(-5.millis)` no se puede construir (lo implementado); (b) tipo `FiniteDuration` propio con `i64` nanosegundos, exacto pero hay que reimplementar la aritmética; (c) `Timeout::new` falible que rechace lo que Pekko no puede construir.
- **Recomendación:** (a) hasta que AC-ASK necesite el caso. Sin respuesta del Owner la fila AC-UTIL-001 queda `partial`.
- **Filas afectadas:** AC-UTIL-001
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** rechazar la recomendación (a): conservar el comportamiento de Pekko; Timeout admite duraciones negativas, cero y positivas dentro del rango de FiniteDuration [-(2^63-1), 2^63-1] ns, con representación i64 y constructores falibles; la restricción a valores positivos no se acepta como adaptación al lenguaje. No decide Q-UTIL-2, Q-UTIL-3, Q-UTIL-4 ni Q-UTIL-5 (Owner (instrucción en la sesión), 2026-10-31)

