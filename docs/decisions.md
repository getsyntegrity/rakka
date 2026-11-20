# Registro único de decisiones del Owner

Generado desde `docs/inventory/contracts/*.decisions.tsv` y las decisiones transversales por `tools/` (ver `docs/inventory/README.md`). Fuente de la tabla machine-readable: `docs/decisions/register.tsv`.

**Estado: 244 decisiones: 0 bloqueantes, 181 propuestas, 50 aceptadas, 10 aceptadas con cambios, 3 rechazadas.** Las respuestas del Owner (con fecha y autor) figuran en la sección 6 y se leen de `docs/decisions/owner-decisions.tsv`; una respuesta no cierra por sí sola las filas afectadas: eso exige implementación y validación.

## 1. Ya autorizado (instrucciones explícitas del Owner)

| ID | Autorización | Alcance |
|---|---|---|
| A-1 | Apache Pekko 2.0.0 es la fuente de verdad, fijada en `upstream/pekko` @ `9fbd2283b1cf701c90a1223bf621942c191db61f` (= `v2.0.0^{}`, verificado con `git ls-remote` el 2026-11-18). Sustituye a la referencia 1.7.1 (decisión Q-X-11). | todo |
| A-2 | `feat/pekko-rust-pilot-migration` es la rama de integración; `main` y `develop` están discontinuadas (referencia histórica hasta que el Owner decida archivarlas); sin push directo a la rama de integración salvo pedido explícito del Owner, sin force push ni borrado de ramas sin autorización; las ramas salen de `origin/feat/pekko-rust-pilot-migration`. | proceso |
| A-3 | TDD estricto RED → GREEN → REFACTOR con evidencia real; gates de la decisión Q-X-14 (cobertura de producción ≥95 % por crate, que sustituye al 90 % de líneas nuevas anterior); nombres que expresan propósito. | todas las unidades |
| A-4 | Ninguna diferencia semántica se acepta sin decisión explícita del Owner; una pregunta abierta impide cerrar el comportamiento afectado. | todas las filas con `Q-` |
| A-5 | Ningún comportamiento se elimina por ser difícil de expresar en Rust; las exclusiones requieren autorización explícita. | todo el inventario |
| A-6 | Classic y Typed completos, con interoperabilidad y testkits necesarios, están en alcance. | inventario |

## 2. Cómo responder

Responder con una de estas formas por decisión (o por bloque): `aceptar` (se adopta la recomendación tal cual), `aceptar con cambios: <texto>`, `rechazar: <alternativa>`. Una respuesta global válida es `aceptar todas las recomendaciones excepto: <lista>`. Cada respuesta se registra en una fila de `docs/decisions/owner-decisions.tsv` (`id`, `status` = `accepted` | `accepted_with_changes` | `rejected`, `decided_on` AAAA-MM-DD, `decided_by`, `decision`); es la única fuente editable y este documento, `register.tsv` y los conteos se regeneran desde ella sin perder respuestas. Hasta entonces no hay aprobación.

## 3. Bloqueos reales

Impiden cerrar el comportamiento afectado aunque el resto de la unidad avance.

## 4. Decisiones transversales (afectan a varias unidades)

Decidirlas primero: desbloquean la mayor parte de las propuestas por unidad.

### Q-X-1 — Implementación de HOCON (parser y carga)

- **Estado:** aceptada con cambios
- **Evidencia:** Plan docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md: golden de 161 casos contra Typesafe Config 1.4.9; `hocon-parser` 1.14.2 diverge en `+=` entre capas, duraciones negativas, `"ON"` e `include classpath()`; `hocon-rs` 0.2.0 no tiene `withFallback`; `hocon` 0.9.0 sin mantenimiento desde 2022.
- **Alternativas:** (A) subconjunto propio sin dependencias; (B) `hocon-parser` tras fachada con una `Q:` por divergencia; (C) (B) más parche upstream.
- **Recomendación:** (A), con el golden como suite de aceptación y ampliado antes de implementar. Alias: Q-CFG-1.
- **Filas afectadas:** AC-CFG-001; AC-CFG-017.*; todo lo que lee Config
- **Unidades afectadas:** AC-CFG, AC-SER (WireConfig), todas las que leen reference.conf
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-2 — Registro nombre → fábrica en lugar de reflexión por FQCN

- **Estado:** aceptada con cambios
- **Evidencia:** AC-CFG-016, AC-EXT-003, DynamicAccessSpec.scala:40,50,59,72; el inventario ya propone el registro; `inventory`/`linkme` descartado como mecanismo principal (dependen del linker).
- **Alternativas:** (A) `FactoryRegistry<Producto>` por punto de extensión; (B) un registro único con `Any`; (C) macros de registro global.
- **Recomendación:** (A); un nombre no registrado devuelve un error de configuración que lleva el nombre. Abierto dentro de la decisión: tipo del error (`ConfigurationException` o propio). Alias: Q-CFG-2.
- **Filas afectadas:** AC-CFG-016; AC-EXT-003; AC-MBX-004; AC-DSP-010; AC-EVT-011; AC-SCH-010; AC-SER-*
- **Unidades afectadas:** AC-CFG, AC-EXT, AC-MBX, AC-DSP, AC-EVT, AC-SCH, AC-SER
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-3 — Tipo de los mensajes en Classic

- **Estado:** aceptada
- **Evidencia:** actor-classic.md pregunta 2; Classic es dinámico (`Any` + `PartialFunction`); `unhandled`, `become` y `UnhandledMessage` dependen de poder rechazar un mensaje.
- **Alternativas:** (A) `Box<dyn Any + Send>` y `receive` que hace downcast y devuelve Handled/Unhandled; (B) enum por actor.
- **Recomendación:** (A); el tipado fuerte queda en `rakka-actor-typed`.
- **Filas afectadas:** AC-REF-002; AC-CELL-001; AC-ASK-001
- **Unidades afectadas:** AC-REF, AC-CELL, AC-ASK, AC-FSM, AC-RT
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A). (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-4 — Runtime y modelo de ejecución

- **Estado:** aceptada con cambios
- **Evidencia:** actor-classic.md pregunta 3; `BoundedMailbox` bloquea al emisor (no encaja en async); claves de `fork-join-executor`; rueda de timers `LightArrayRevolverScheduler`.
- **Alternativas:** (A) pools de hilos propios con interfaz de ejecutor y futuros agnósticos del runtime; (B) atar el port a tokio.
- **Recomendación:** (A): preserva el bloqueo del emisor, `PinnedDispatcher` y las claves de configuración; tokio puede ser un adaptador opcional más adelante.
- **Filas afectadas:** AC-DSP-008; AC-MBX-007; AC-SCH-008
- **Unidades afectadas:** AC-DSP, AC-MBX, AC-SCH, AC-CELL, AC-ASK
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-5 — Qué es un fallo supervisable

- **Estado:** aceptada con cambios
- **Evidencia:** actor-classic.md pregunta 1; Pekko distingue Exception (reinicia) de Error/Throwable (escala).
- **Alternativas:** (A) `Err` y `panic` son fallos supervisables, tratados como Exception; (B) `panic` escala; (C) no capturar panics.
- **Recomendación:** (A): capturar panics preserva el aislamiento entre actores; (C) lo rompe.
- **Filas afectadas:** AC-SUP-001; AC-SUP-004; AC-SYS-015; AT-SUP-002
- **Unidades afectadas:** AC-SUP, AC-CELL, AT-SUP
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A). Documentar que con panic=abort no hay supervisión de panics. (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-6 — Suscripción por jerarquía de tipos en EventStream

- **Estado:** aceptada
- **Evidencia:** actor-classic.md pregunta 5; Rust no tiene herencia; AC-UTIL-008, AC-EVT-003.
- **Alternativas:** (A) `TypeId` exacto más canales padre declarados por el tipo de evento; (B) enum cerrado de eventos.
- **Recomendación:** (A).
- **Filas afectadas:** AC-UTIL-008; AC-EVT-003
- **Unidades afectadas:** AC-EVT, AC-UTIL, AT-EVT
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A). (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-7 — Loggers como actores o backend directo

- **Estado:** aceptada
- **Evidencia:** actor-classic.md pregunta 8; `LoggerSpec` y `EventFilter` dependen del handshake InitializeLogger/LoggerInitialized (AC-EVT-011).
- **Alternativas:** (A) loggers como actores con el handshake; (B) backend directo a `tracing`.
- **Recomendación:** (A): permite portar `LoggerSpec` y `EventFilter` sin cambios; `tracing` puede ser un logger más.
- **Filas afectadas:** AC-EVT-011; AC-TK-*
- **Unidades afectadas:** AC-EVT, AC-TK, AT-LOG
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (A). (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-8 — Typed sobre Classic o runtime tipado nativo

- **Estado:** aceptada
- **Evidencia:** docs/implementation-order.md (sección Decisión previa); `ActorAdapter`, `ActorContextAdapter`, `ActorSystemAdapter` en actor-typed/.../internal/adapter/.
- **Alternativas:** (a) replicar las capas de Pekko; (b) runtime tipado nativo con interop aparte.
- **Recomendación:** (a): cada contrato heredado se implementa y valida una sola vez y la interop sale casi gratis.
- **Filas afectadas:** AT-SYS-011; AT-ADP-*; todas las AT-*
- **Unidades afectadas:** todas las AT-*
- **Origen:** transversal
- **Respuesta del Owner:** aceptar (a): Typed sobre Classic. (Owner (respuesta al lote 01), 2026-11-18)

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

- **Estado:** aceptada
- **Evidencia:** actor-classic.md sección Límites del módulo; ninguno vive en `pekko-actor`/`pekko-actor-typed`; hay ganchos (AC-SYS-018, AC-DW-010/012, AC-SER-007).
- **Alternativas:** (A) fuera del alcance; el trait `ActorRefProvider` se deja abierto; (B) incluir alguno.
- **Recomendación:** (A). Es el límite con que se armó el inventario; no fue autorizado explícitamente.
- **Filas afectadas:** AC-SYS-018; AC-CFG-008
- **Unidades afectadas:** AC-SYS, AC-CFG, AC-DW, AC-SER
- **Origen:** transversal
- **Respuesta del Owner:** autoriza el alcance (A): sólo actor y actor-typed locales (sin remoting, cluster, persistence, streams ni HTTP). (Owner (respuesta al lote 01), 2026-11-18)

### Q-X-11 — Versión de referencia: Apache Pekko 2.0.0

- **Estado:** aceptada
- **Evidencia:** upstream/pekko @ v2.0.0 (commit 9fbd2283b1cf701c90a1223bf621942c191db61f); docs/upstream.md. Sustituye a A-1 (1.7.1).
- **Alternativas:** (a) 2.0.0; (b) mantener 1.7.1.
- **Recomendación:** (a).
- **Filas afectadas:** todas las filas
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: la referencia es Apache Pekko 2.0.0 (upstream/pekko @ v2.0.0). Sustituye a A-1 (1.7.1). (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-12 — Oráculo oficial: CLI Java del piloto con rust-anything

- **Estado:** aceptada
- **Evidencia:** oracle/ (Java 17), `rust-anything app record/replay`. El oráculo jshell + goldens de tools/oracle queda deprecado y sus escenarios se convierten a casos rust-anything.
- **Alternativas:** (a) CLI del piloto + rust-anything; (b) jshell + goldens; (c) ambos.
- **Recomendación:** (a).
- **Filas afectadas:** todas las filas
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: el oráculo oficial es el CLI Java del piloto (Java 17) con rust-anything app record/replay. El oráculo jshell + goldens (tools/oracle) queda deprecado y sus escenarios se convierten a casos rust-anything. (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-13 — docs/unit-plan.md aprobado como guía de orden y alcance

- **Estado:** aceptada
- **Evidencia:** docs/unit-plan.md: 77 unidades, 783 filas, 10 olas. Las filas y las olas valen; los apartados Tests/oráculo de cada unidad se sustituyen por casos rust-anything (Q-X-12). Levanta la nota Este plan no está aprobado para implementar.
- **Alternativas:** (a) aprobar con ese reemplazo; (b) seguir sin aprobar.
- **Recomendación:** (a).
- **Filas afectadas:** todas las filas
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: docs/unit-plan.md queda aprobado como guía de orden y alcance; las filas y las olas valen y los Tests/oráculo genéricos de cada unidad se reemplazan por casos rust-anything. (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-14 — Gates de calidad por slice e hito

- **Estado:** aceptada
- **Evidencia:** Replay completo, tests, Clippy -D warnings, fmt, cobertura de producción >=95 % por crate, mutantes nuevos en cada slice, y regresión completa de mutantes y fuzzers en cada hito. Sustituye a A-3 (cobertura 90 % de líneas nuevas).
- **Alternativas:** (a) estos gates; (b) mantener el 90 % de líneas nuevas.
- **Recomendación:** (a).
- **Filas afectadas:** todas las filas
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: gates = replay completo, tests, Clippy -D warnings, fmt, cobertura de producción >=95 % por crate, mutantes nuevos en cada slice y regresión completa de mutantes y fuzzers en cada hito. Sustituye a A-3 (90 % de líneas nuevas). (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-15 — Nombres de los crates: rakka-actor y rakka-actor-typed

- **Estado:** aceptada
- **Evidencia:** crates/rakka-actor, crates/rakka-actor-typed; el piloto usaba pekko-actor y pekko-actor-typed.
- **Alternativas:** (a) rakka-*; (b) pekko-*.
- **Recomendación:** (a).
- **Filas afectadas:** todas las filas
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: los crates son rakka-actor y rakka-actor-typed. (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-16 — JDK del oráculo fijo en 17

- **Estado:** aceptada
- **Evidencia:** oracle/ (CLI Java, JDK 17), project/JdkOptions.scala:26 (targetJavaVersion 17). Los mensajes de excepción del JDK forman parte de lo observable y cambian entre versiones: p. ej. URI.scanByte lanza NumberFormatException con `Error at index N in: "…"` en JDK 17.
- **Alternativas:** (a) JDK 17 fijo; (b) JDK 21; (c) ambos.
- **Recomendación:** (a); las grabaciones de JDK 21 de los goldens deprecados no cuentan.
- **Filas afectadas:** AC-PATH-*; AC-UTIL-*; todas las filas con mensajes de excepción del JDK
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: el JDK del oráculo queda fijo en 17, porque los mensajes de excepción del JDK forman parte de lo observable y cambian entre versiones. Las grabaciones de JDK 21 de los goldens deprecados no cuentan. (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-17 — Dominio de entrada de la API pública: &str

- **Estado:** aceptada
- **Evidencia:** Q-PATH-2; la entrada con un sustituto UTF-16 suelto (p. ej. `pekko://sys\ud800@host:2552/user`) es representable en un String de Java y no en un &str de Rust. Caso grabado: ok-user-info-lone-surrogate.
- **Alternativas:** (a) &str y las entradas con sustitutos sueltos quedan fuera de dominio (NOT_APPLICABLE explícito y rechazo explícito en el arnés); (b) UTF-16 en la API.
- **Recomendación:** (a).
- **Filas afectadas:** AC-PATH-*; AC-UTIL-*
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: el dominio de entrada de la API pública es &str; las entradas con sustitutos UTF-16 sueltos (representables en Java, no en Rust) quedan fuera de dominio. En el replay se listan como NOT_APPLICABLE en cases/not-applicable.tsv (con motivo y esta decisión), se informan como "N/N aplicables + K no aplicables" y el arnés las rechaza con un error explícito, nunca con reemplazo lossy. Sustituye a "las cifras del piloto tienen que mantenerse" sólo para ese caso. (Owner (instrucción en la sesión), 2026-11-18)

### Q-X-18 — Procedencia del código: reescritura en sala limpia de lo derivado de fuentes no compatibles o dudosas

- **Estado:** aceptada
- **Evidencia:** Auditoría de procedencia (docs/legal/provenance-audit.md); uri.rs y jstring.rs declaran estar portados de OpenJDK (GPL-2.0 con Classpath exception), ver Q-PATH-6.
- **Alternativas:** (a) sala limpia documentada; (b) revisión legal del código existente.
- **Recomendación:** (a).
- **Filas afectadas:** AC-PATH-*; todo archivo de procedencia dudosa
- **Unidades afectadas:** todas
- **Origen:** transversal
- **Respuesta del Owner:** aceptar: todo el código de rakka derivado de fuentes con licencia no compatible con Apache-2.0, o de procedencia dudosa, se reescribe en sala limpia (auditoría de procedencia, especificación sin código, implementación por un agente que no lee el original ni código GPL, verificación con replay, mutantes y comparación de similitud). Prohibido leer o portar código de OpenJDK u otras fuentes GPL: la conducta del JDK se toma sólo de su documentación pública y del oráculo. No se publica ningún release hasta cerrar todas las reescrituras. (Owner (respuesta al lote 01), 2026-11-18)

Alias: `Q-CFG-1` = `Q-X-1`, `Q-CFG-2` = `Q-X-2`.

## 5. Propuestas por unidad

### Q-ASK-1 — Modelo de Future de ask/gracefulStop/after/retry y tipado del resultado

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AskSupport.scala:372-388 devuelve Future[Any]; mapTo en el llamador | Cierre 2026-10-31: Clasificacion: arquitectura (modelo async de Rust) con representacion que conserva el contrato (tipos de error y textos). Resuelto por lectura: AskSpec fija tipos y textos exactos solo para entradas invalidas (AskSpec:61-92) y contiene (no afirma el texto) el de AskTimeoutException (AskSpec:94-126).
- **Alternativas:** (a) async fn tipada con Result<R, AskError> sobre tokio; (b) Future<Output=Box<dyn Any>> sin tipado
- **Recomendación:** (a); mensajes de error identicos a Pekko donde el spec los fija (AskSpec:66-69,77,87). Depende de Q-X-3/Q-X-4/Q-X-9. Consecuencias observables: tipo y texto de errores de ask (A3-A5 de AC-ASK-001.2). Bloquea el diseno de la firma publica de ask/gracefulStop/after/retry (AC-ASK-001.*, 003, 004, 007, 008, 010); puede avanzar la validacion de entradas (AC-ASK-001.2 A3-A5) y los textos con tests de unidad sin runtime.
- **Filas afectadas:** AC-ASK-001.1;AC-ASK-001.2;AC-ASK-001.3;AC-ASK-003;AC-ASK-004;AC-ASK-007;AC-ASK-008;AC-ASK-010
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-2 — Ref temporal: path lazy, estados atomicos y texto de error de PromiseRef

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AskSupport.scala:515-700; PromiseRef.scala:166-172 (condicion length>0, texto "must not be negative") | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato salvo el texto de error (divergencia funcional si se corrige). Resuelto por lectura: PromiseRef.scala:166-172 usa condicion `length > 0` y texto "must not be negative" (0 tambien falla); ningun test lo ejercita (PromiseRefSpec no cubre timeout invalido).
- **Alternativas:** (a) replicar maquina de estados y registro lazy de path; (b) path eager; texto de error: replicar o corregir
- **Recomendación:** (a) replicar; mantener el texto exacto de Pekko (discrepancia condicion/texto documentada). Consecuencias: solo el mensaje de AskPromiseRef con timeout 0/negativo (nunca visto por usuarios de ask, que valida antes). Bloquea solo AC-ASK-010 A5; AC-ASK-002 puede avanzar.
- **Filas afectadas:** AC-ASK-002;AC-ASK-010
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-3 — CircuitBreaker: mapeo de tiempos, ejecutor de listeners y semantica de saturacion de resetTimeout

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreaker.scala:723-775 (listeners via executor.execute); 1105-1118 (no satura a maxResetTimeout) | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (replicar) / divergencia funcional solo si se satura. Resuelto por lectura: CircuitBreaker.scala:1105-1118 (la rama `if (nextResetTimeout < maxResetTimeout)` NO actualiza cuando el valor calculado alcanza o supera el maximo, de modo que el resetTimeout se queda en el ultimo valor menor que el maximo y nunca llega a maxResetTimeout).
- **Alternativas:** (a) replicar exacto (incluida la no saturacion); (b) saturar a maxResetTimeout
- **Recomendación:** (a) replicar exacto; (b) solo con autorizacion del Owner. Consecuencias: secuencia de resetTimeout observable por onHalfOpen/onOpen. Bloquea AC-ASK-011.3 A3 y AC-ASK-013 (defaults de config); puede avanzar 011.1, 011.2 y los callbacks (011.4) con maxResetTimeout alto.
- **Filas afectadas:** AC-ASK-011.1;AC-ASK-011.3;AC-ASK-011.4;AC-ASK-013
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-4 — CircuitBreaker: callTimeout=Duration.Zero ignora defineFailureFn; allowlist por FQCN de excepcion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreaker.scala:863-875 (rama sin defineFailureFn) vs 877-907; allowlist por getClass.getName (802-806) | Cierre 2026-10-31: Clasificacion: representacion (allowlist por nombre de tipo) + divergencia funcional si se corrige la rama callTimeout=0. Resuelto por lectura: CircuitBreaker.scala:863-875 (callTimeout == Duration.Zero: todo Failure cuenta, defineFailureFn ignorada) frente a 877-907; ningun test con callTimeout=0 y defineFailureFn.
- **Alternativas:** (a) replicar el comportamiento incluyendo la rama ignorando defineFailureFn; (b) aplicar defineFailureFn tambien con timeout 0. Allowlist: nombre de tipo registrado vs TypeId
- **Recomendación:** (a) replicar (paridad) y registrar la diferencia; allowlist por nombre de tipo en un registro explicito. Consecuencias: breakers con callTimeout=0 ignoran defineFailureFn y allowlist. Bloquea AC-ASK-011.2 A5 y AC-ASK-012 A6; el resto puede avanzar.
- **Filas afectadas:** AC-ASK-011.2;AC-ASK-012
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-5 — API sincrona de CircuitBreaker (withSyncCircuitBreaker bloquea con Await)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreaker.scala:436-458,466-480 | Cierre 2026-10-31: Clasificacion: exclusion de alcance prohibida sin autorizacion; representacion (bloquea el hilo). Resuelto por lectura: CircuitBreaker.scala:436-458,466-480.
- **Alternativas:** (a) exponer fn bloqueante (block_on) documentada; (b) proponer exclusion de la variante sync
- **Recomendación:** (a) exponer; prohibido excluir sin autorizacion; marcar la diferencia: bloquea el hilo del runtime. Consecuencias: solo AC-ASK-011.4 (A7); no bloquea nada mas.
- **Filas afectadas:** AC-ASK-011.4
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-6 — CircuitBreakerTelemetry: carga por reflexion (FQCN) y clave de config ausente de reference.conf

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CircuitBreakerTelemetry.scala (provider); grep reference.conf sin `telemetry` | Cierre 2026-10-31: Clasificacion: arquitectura (registro de fabricas, Q-X-2) + representacion (misma clave). Resuelto por lectura: la clave pekko.circuit-breaker.telemetry.implementations no existe en reference.conf (CircuitBreakerTelemetry.scala la lee con getStringList y hasPath).
- **Alternativas:** (a) registro de fabricas por nombre (Q-X-2) leyendo la misma clave; (b) solo API programatica
- **Recomendación:** (a) misma clave `pekko.circuit-breaker.telemetry.implementations`; el trait es @InternalStableApi: exponerlo como API estable para extensores. Consecuencias: AC-ASK-014 solo; sin telemetria el breaker usa el noop. No bloquea el breaker.
- **Filas afectadas:** AC-ASK-014
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-7 — BackoffSupervisor: composicion de strategy y directivas (Restart traducido a Stop)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** BackoffOnRestartSupervisor.scala:57-96; BackoffOnStopSupervisor.scala:57-70 | Cierre 2026-10-31: Clasificacion: arquitectura (modelo de supervision de rakka, AC-SUP-001) con representacion que conserva el contrato. Evidencia nueva: composicion del decider en BackoffOnStopSupervisor.scala:57-70 y BackoffOnRestartSupervisor.scala:56-93 (detallada en AC-ASK-015.6); en onStop un Restart del decider es un reinicio normal inmediato (no hay backoff); en onFailure todo Restart se traduce a Stop+backoff; un `Terminated` de un actor que no es el hijo se reenvia al hijo en onStop (BackoffOnStopSupervisor.scala:72-73) y detiene el supervisor en onFailure (BackoffOnRestartSupervisor.scala:106-110).
- **Alternativas:** (a) replicar el decider compuesto sobre el modelo de supervision de rakka; (b) simplificar a estrategia fija
- **Recomendación:** (a); depende de Q-X-5 (que es un fallo) y del modelo de supervision (AC-SUP-001). Consecuencias observables: que excepcion provoca backoff y cual reinicio inmediato; reenvio de Terminated ajeno. Bloquea AC-ASK-015.2, .3 y .6 (necesitan directiva por fallo del hijo de AC-SUP-001); pueden avanzar AC-ASK-015.1 (opciones y validaciones), .4 (mensajes mientras detenido, parcialmente) y .5 (finalStopMessage) con un hijo que se detiene por su cuenta.
- **Filas afectadas:** AC-ASK-015.1;AC-ASK-015.2;AC-ASK-015.3;AC-ASK-015.4;AC-ASK-015.5
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-8 — API deprecada Backoff / BackoffOptions: propuesta de no portar

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Backoff.scala deprecated since Akka 2.5.22 (BackoffSupervisor.scala:175,220,236) | Cierre 2026-10-31: Clasificacion: exclusion de alcance (solo el Owner puede autorizarla). Resuelto por lectura: Backoff.scala completo (643 lineas) y BackoffSupervisor.scala:45-237,332-395: ninguna semantica propia salvo 5 diferencias de superficie (finalStopMessage ignorado en onFailure, sin withHandlerWhileStopped, propsWithSupervisorStrategy acepta cualquier SupervisorStrategy, constructor publico de BackoffSupervisor, replyWhileStopped Option); no hay un solo test de la API vieja en actor-tests.
- **Alternativas:** (a) portarla; (b) proponer exclusion (solo el Owner puede autorizarla)
- **Recomendación:** (b) proponer exclusion con BackoffOpts como equivalente; la fila sigue en alcance con impl=pending hasta decision. Consecuencias: ninguna para otras filas (AC-ASK-016 es hoja). Bloquea solo la implementacion de AC-ASK-016; si se acepta la exclusion la fila se cierra sin impl. Nada mas depende de esta decision.
- **Filas afectadas:** AC-ASK-016
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-9 — Fachadas Java Patterns/PatternsCS: propuesta de no portar

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Patterns.scala:31-1333 (delegacion pura) | Cierre 2026-10-31: Clasificacion: exclusion de alcance (solo el Owner puede autorizarla). Resuelto por lectura: Patterns.scala completo (1333 lineas) y PatternsTest.java completo: delegacion pura salvo los require de nulos (incluido el defecto de copia "Parameter minBackoff should not be null." tambien para maxBackoff) y el formato "Timeout of PT0.2S expired" de la variante CompletionStage.
- **Alternativas:** (a) portar como modulo de fachada; (b) proponer exclusion
- **Recomendación:** (b) proponer exclusion (API Rust unica); la fila sigue en alcance con impl=pending hasta decision. Consecuencias: ninguna para otras filas (AC-ASK-017 es hoja; AC-ASK-011.5 solo cita la fachada). Bloquea solo la implementacion de AC-ASK-017.
- **Filas afectadas:** AC-ASK-017
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-ASK-10 — Prefijo "requirement failed: " en los mensajes de require (BackoffOpts, CircuitBreaker, RetrySupport, CoordinatedShutdown.addTask, Patterns.retry Java)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Scala `require(cond, msg)` lanza IllegalArgumentException("requirement failed: " + msg) (Predef); BackoffOptions.scala:381-386, CircuitBreaker.scala:163-164, RetrySupport.scala:179-183,377-380, CoordinatedShutdown.scala:566-572,606-614, Patterns.scala:515-882. Los `throw new IllegalArgumentException(msg)` directos (AskSupport, PromiseRef, Resizer, OptimalSizeExploringResizer, CoordinatedShutdown.phasesFromConfig/topologicalSort/timeout) no llevan prefijo. Ningun test de actor-tests fija el texto completo de un `require` de este grupo (grep sobre "minBackoff must", "Unknown phase").
- **Alternativas:** (a) replicar el texto con prefijo; (b) texto base sin prefijo; (c) tipo de error propio con el texto base y sin prefijo
- **Recomendación:** (c) con el texto base contractual de cada fila (sin prefijo); el prefijo es un artefacto de Predef, no de Pekko. Consecuencias observables: solo el Display del error de validacion; nada bloquea y todo puede avanzar. Si el Owner quiere paridad textual estricta de los require, elegir (a).
- **Filas afectadas:** AC-ASK-009.1;AC-ASK-009.2;AC-ASK-011.3;AC-ASK-015.1;AC-ASK-016;AC-ASK-017;AC-CS-003.1
- **Unidades afectadas:** ASK
- **Origen:** classic-patterns

### Q-CELL-1 — Forma de receive (PartialFunction) y de la pila become/unbecome

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:384-410,545-556 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:602-615
- **Alternativas:** (a) fn(&mut self, &mut Context, msg) -> Handled y pila Vec<Box<dyn FnMut>> / (b) trait Behavior con enum de resultado / (c) macro receive!
- **Recomendación:** (a) con Handled::{Handled,Unhandled} para disparar unhandled(); la pila se limpia en restart y el receive original se re-evalúa tras unbecome con pila vacía [2026-10-31 cierre documental] Clasificación: arquitectura. Consecuencia observable: `Handled::Unhandled` dispara `unhandled()` (DeathPactException para Terminated; UnhandledMessage para el resto) igual que `Actor.aroundReceive`; emptyBehavior = siempre Unhandled; la pila se vacía en restart y `unbecome` con pila de 0/1 recupera `receive`. Bloquea: `impl` de AC-CELL-001/005 (y la firma de Actor para todo el crate). Puede avanzar: la documentación y los tests de ciclo de vida (se escriben contra el trait).
- **Filas afectadas:** AC-CELL-001 ; AC-CELL-005
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-2 — Interrupción de hilo (InterruptedException → ActorInterruptedException)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:226-234,353-359 ; actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:422-477
- **Alternativas:** (a) excluir; mapear la cancelación cooperativa de la tarea a un fallo normal no recuperable / (b) emular con un token de cancelación que convierte en ActorInterruptedError / (c) ignorar
- **Recomendación:** (a) como propuesta de exclusión; el Owner debe autorizarla explícitamente; los tests de interrupción no se portan [2026-10-31 cierre documental] Clasificación: exclusión de alcance (Thread.interrupt no existe en Rust). Consecuencia: AC-CELL-021 y ActorInterruptedException no se implementan; los tests ActorModelSpec.scala:422-477 no se portan. Bloquea: sólo el cierre de AC-CELL-021 (requiere autorización explícita del Owner). Puede avanzar: todo.
- **Filas afectadas:** AC-CELL-021
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-3 — Uso del contexto/ActorRef::self tras terminar el actor

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:193-230 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:247-276 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:240-244 ; actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:178-189
- **Alternativas:** (a) las operaciones sobre el contexto capturado devuelven Err(Terminated) o se ignoran con dead letters / (b) prohibir por lifetime (no capturable)
- **Recomendación:** (b) donde el borrow checker lo garantice y (a) para refs clonadas; el contexto de Pekko no se anula tras terminar [2026-10-31 cierre documental] Clasificación: representación. Incertidumbre resuelta leyendo: Pekko NO anula el contexto tras terminar (ActorLifeCycleSpec.scala:210-213) y no es serializable (ActorCell.scala:243-244); la propuesta (b) (no capturable por lifetime) más (a) para refs clonadas es compatible. Consecuencia: capturar `ActorRef` clonado tras la parada manda a dead letters, como Pekko. Queda sólo la aprobación del Owner. Bloquea: nada de la documentación; el cierre de `validation` de AC-CELL-019. Puede avanzar: todo.
- **Filas afectadas:** AC-CELL-019
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-4 — Texto de los eventos Debug que incluyen toString de actor/envelope/ref ("started (<actor>)", "received AutoReceiveMessage <envelope>")

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:569-570,655-656 ; actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:196-199,272-277
- **Alternativas:** (a) usar el nombre de tipo Rust y Debug del envelope / (b) formato Scala exacto "Envelope(msg,sender)" / (c) campo estructurado
- **Recomendación:** (b) para el prefijo fijo ("started (", "received AutoReceiveMessage Envelope(") y (a) para el resto; documentar la diferencia [2026-10-31 cierre documental] Clasificación: representación. Consecuencia: los specs de LoggingReceiveSpec comparan por prefijo ("received AutoReceiveMessage Envelope(PoisonPill", "now supervising TestActor[…]"); basta fijar prefijos y formato `Envelope(<msg>,<sender>)`. Bloquea: `validation` de AC-CELL-023 (impl puede avanzar). Puede avanzar: el resto de CELL.
- **Filas afectadas:** AC-CELL-023
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CELL-5 — Modos de test serialize-messages y serialize-creators (ida y vuelta de mensajes y args de Props)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:171-225 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:279-306 ; actor/src/main/resources/reference.conf:129-140 ; actor-tests/src/test/scala/org/apache/pekko/serialization/SerializeSpec.scala:308-361 ; actor-tests/src/test/scala/org/apache/pekko/actor/dungeon/DispatchSpec.scala:30-41 ; docs/inventory/contracts/classic-io-ser.md (AC-SER-008.1/008.2)
- **Alternativas:** (a) implementar como hook opcional detrás de feature de test cuando exista la unidad de serialización / (b) omitir / (c) comprobación en tiempo de compilación (Serialize bound)
- **Recomendación:** (a), depende de AC-SER y de Q-X-3; hasta entonces las filas quedan en alcance con impl=pending [2026-10-31 cierre documental] Clasificación: arquitectura (modo de pruebas opcional) y exclusión parcial. Resuelto leyendo: el algoritmo vive en AC-SER-008.1/008.2 (classic-io-ser); AC-CELL-028/029 son sólo los puntos de integración (sendMessage / makeChild, antes de reservar el nombre) y se definen como fachadas. Consecuencia: sin SER no hay hook; con Box<dyn Any> sólo se verifican tipos registrados (ver Q-SER-4). Bloquea: `impl` de AC-CELL-028/029 hasta que exista SER. Puede avanzar: todo CELL.
- **Filas afectadas:** AC-CELL-028 ; AC-CELL-029
- **Unidades afectadas:** CELL
- **Origen:** classic-cell

### Q-CFG-14 — Includes de HOCON sin classpath: `include "x"`, `classpath(...)`, `url(...)`, `file(...)` y `required(...)` en rakka

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:11 (`include "version"`); casos grabados INC-01..04 (include ausente se ignora; `required("x")` ausente falla con `IO` y la lista de tres intentos `.conf/.json/.properties` con el texto `resource not found on classpath`); `Config.parseString` de Typesafe Config 1.4.9 con Pekko 2.0.0 (JDK 17). Sin medir: los mensajes de `required(file(..))`, `required(classpath(..))` y `url(..)`.
- **Alternativas:** (a) `include "x"` y `classpath("x")` buscan en una tabla de recursos embebidos que pasa quien llama (`parse_string_with`); `file("p")` lee del sistema de archivos; `url(..)` no se admite; un `required` ausente falla con el texto medido (el de `file` es una suposición sin medir). (b) además un directorio de búsqueda configurable para los nombres sin envoltorio. (c) sólo ignorar todos los includes.
- **Recomendación:** (a). Clasificación: arquitectura (no hay classpath en Rust; Q-CFG-6 (a) lo descarta). Consecuencias observables: un `include` que en la JVM resolvería por classpath se ignora o falla según `required`. Bloquea: nada; las filas quedan `partial` hasta medir los mensajes de file/url.
- **Filas afectadas:** AC-CFG-001, AC-CFG-017.6
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-15 — Descripción de origen de los valores de configuración en errores y en el render (`String: N`, archivo, `env variables`)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Casos grabados: todos los mensajes de error llevan el prefijo `String: <línea>: ` (texto) y `renderWithRedactions` imprime `# String: <línea>` antes de cada valor, `# env variables` y `"<env variable>"` para variables de entorno (RW-01..06); un archivo sería `<ruta>: <línea>` (no medido: el oráculo sólo carga texto y los dos reference.conf desde jar).
- **Alternativas:** (a) reproducir el formato de Typesafe: `String: N` para texto, `<ruta>: N` para archivos, `env variables`; el render con comentarios de origen. (b) mensajes y render sin descripción de origen (más cortos, distintos de los medidos). (c) (a) para texto y entorno y, para archivos, el formato `<ruta>: N` sin medir.
- **Recomendación:** (c). Clasificación: representación. Consecuencias observables: los textos de `ConfigError::message()` y de `Settings.toString`; con (b) cambian todos. P07 implementa (a) para texto y entorno; la ruta de archivo queda sin implementar (los valores de `Config::load` dicen `String: N`).
- **Filas afectadas:** AC-UTIL-002.5, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5, AC-CFG-017.7, AC-CFG-019, AC-CFG-002, AC-CFG-004, AC-CFG-005, AC-CFG-006, AC-CFG-007, AC-CFG-009, AC-CFG-010, AC-CFG-011, AC-CFG-018
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-16 — Sobrescritura de la configuración por variables de entorno con prefijo (completa Q-CFG-6 (a), que no fija el prefijo)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Q-CFG-6 (a): `application.conf` del directorio de trabajo + variables de entorno con prefijo + reference.conf embebido. Typesafe Config no tiene prefijo propio: sólo `${VAR}` / `${?VAR}` (casos SUB-08, SUB-09) y, con la propiedad de sistema `config.override_with_env_vars`, las variables `CONFIG_FORCE_*`.
- **Alternativas:** (a) sólo sustituciones `${VAR}` y `${?VAR}` (hecho en P07) y ninguna sobrescritura automática. (b) la convención de Typesafe: con un indicador, `CONFIG_FORCE_a_b__c=1` sobrescribe `a.b-c`. (c) un prefijo propio (`RAKKA_`) con `__` como separador de ruta.
- **Recomendación:** (a). Clasificación: arquitectura. Consecuencias observables: con (a) un despliegue no puede cambiar claves sin editar el archivo. P07 implementa (a); `Config::load` no lee variables con prefijo.
- **Filas afectadas:** AC-CFG-001
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-17 — Texto, tipo y lugar del error de `serialize-messages=on` / `serialize-creators=on` (concreta Q-CFG-11 (a), aceptada con cambios: error de configuración hasta que exista AC-SER)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Q-CFG-11: con la opción en `on` el arranque falla con un error de configuración. Pekko 2.0.0 acepta ambas claves en `on` (casos del oráculo `SET-SER serialize-messages = on`, `serialize-creators = on` y las dos: `ok SerializeAllMessages=true ..`), de modo que no hay texto de Pekko que copiar; la decisión no fijó el texto ni el lugar del error entre los demás.
- **Alternativas:** (a) `ConfigurationException` `pekko.actor.serialize-messages=on is not supported yet: the serialization checks need the serialization module (AC-SER).` (la clave de `serialize-creators` si sólo ésta está en `on`), devuelta por `Settings::new` después de todos los demás errores, también del chequeo de versión. (b) el mismo error antes de leer el resto de las claves. (c) un tipo de error propio (`SettingsError::Unsupported`) en lugar de `ConfigurationException`.
- **Recomendación:** (a). Clasificación: representación. Consecuencias observables: sólo el texto de un arranque con la opción en `on`; los tres casos del oráculo con `on` no se comparan (`PENDING` en `config_settings_cases.rs`) y se prueban con tests unitarios. Bloquea: nada; AC-CFG-010 queda `partial` hasta AC-SER.
- **Filas afectadas:** AC-CFG-010, AC-CFG-002
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-18 — Mensajes de `checkValid` que dependen del orden de un HashMap de la JVM y del origen de una clave ausente

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Medido con `new ActorSystem.Settings(..)` y no grabado: `expecting: object with keys [lifecycle, unhandled, receive, fsm, router-misconfiguration, autoreceive, event-stream]` (orden de iteración del HashMap de Typesafe Config 1.4.9 para `pekko.actor.debug`) y `String: 783: pekko.actor.debug.receive: No setting at 'pekko.actor.debug.receive', expecting: string` (origen = el del objeto padre en la config comprobada: una línea, un rango como `String: 3-112` al fusionar dos archivos, o la ruta del jar para `version.conf`). Rust: claves en orden alfabético y `String: 1-132` (origen de la raíz). Q-CFG-4 (aceptada) pide los mensajes de Config 1.4.9 como contrato.
- **Alternativas:** (a) conservar el texto de Rust (claves ordenadas, origen de la raíz) y documentarlo como divergencia de texto, no de categoría ni de clave citada. (b) reproducir el orden de iteración del HashMap de Java (`String.hashCode`, capacidad de la tabla) y el origen del objeto padre. (c) no listar las claves esperadas.
- **Recomendación:** (a). Clasificación: representación. Consecuencias observables: sólo el texto de `ValidationFailed` cuando un objeto de la config es de otro tipo o falta una clave; con (b) el texto coincide con la JVM a cambio de simular el HashMap. Bloquea: nada; las filas quedan `partial`.
- **Filas afectadas:** AC-CFG-002, AC-CFG-019
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-19 — Reference contra el que `Settings::new` valida la configuración (¿incluye el de pekko-actor-typed?)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Pekko valida contra `ConfigFactory.defaultReference(classLoader)`: todos los `reference.conf` del classpath. Con `pekko-actor-typed` presente, una configuración sin `pekko.use-slf4j` ni las claves de typed falla con `ValidationFailed` (caso del oráculo: la base de todos los casos `SET-*` es typed sobre actor). `Settings::new` valida contra `Config::default_reference()` (sólo `pekko-actor`) porque `rakka-actor-typed` aún no existe.
- **Alternativas:** (a) validar sólo contra el reference de `pekko-actor` mientras `rakka-actor-typed` no aporte el suyo (las claves extra de typed se aceptan). (b) `Settings::new_with_reference(name, config, reference)` y que Typed pase el fusionado. (c) validar siempre contra typed sobre actor.
- **Recomendación:** (a). Clasificación: arquitectura. Consecuencias observables: una configuración de Typed sin las claves de typed no se rechaza hasta que Typed pase su reference (b). Bloquea: nada; reversible.
- **Filas afectadas:** AC-CFG-002, AC-CFG-019
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-31 — Error de un nombre de clase no registrado: ClassNotFoundException que lleva el nombre (corrige la redacción de Q-X-2 y de Q-CFG-9 (a))

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Q-X-2 (aceptada con cambios, 2026-11-18) dice "el error de nombre no registrado es ConfigurationException del crate, como en Pekko" y Q-CFG-9 (a) "un error de configuración que nombra la clase"; en Pekko 2.0.0 no es así para DynamicAccess ni para el proveedor. ReflectiveDynamicAccess.getClassFor usa Class.forName (ReflectiveDynamicAccess.scala:44) y devuelve Failure(ClassNotFoundException) (DynamicAccessSpec.scala:50-57). ActorSystemImpl carga el proveedor con dynamicAccess.createInstanceFor[ActorRefProvider](ProviderClass, arguments).get y su catch solo detiene el scheduler y relanza la misma excepción (ActorSystem.scala:996-1011). Medido con el oráculo (Pekko 2.0.0, JDK 17, casos da-0014..da-0017 de la familia dynamic-access): pekko.actor.provider = foo.NonExisting, org.apache.pekko.remote.RemoteActorRefProvider y org.apache.pekko.cluster.ClusterActorRefProvider lanzan ClassNotFoundException con el FQCN como mensaje; LocalActorRefProvider crea el sistema. Los consumidores que envuelven el fallo de carga lo hacen con tipos distintos: Dispatchers lo envuelve en ConfigurationException (Dispatchers.scala:274-281, MessageDispatcherConfigurator); Mailboxes, al cargar un tipo de requirement, en ConfigurationException (Mailboxes.scala:86-96) y, al crear el MailboxType, en IllegalArgumentException (Mailboxes.scala:235-243).
- **Alternativas:** (a) Dos capas. Capa 1: FactoryRegistry::lookup (y create) devuelve un error ClassNotFound, equivalente a ClassNotFoundException, que lleva el nombre; el registro no usa ConfigurationException. Capa 2: cada consumidor transforma el error como Pekko 2.0.0: la resolución del proveedor lo propaga sin envolver; Dispatchers (y el requirement de Mailboxes) lo envuelven en ConfigurationException y la creación del MailboxType en IllegalArgumentException, cuando se implementen (otras unidades; queda anotado en sus filas). Se mantienen Q-X-2 (A) y Q-CFG-9 (c). (b) Mantener el texto literal de Q-X-2 y de Q-CFG-9 (a): el registro y la resolución del proveedor devuelven ConfigurationException (diferencia deliberada con Pekko para el proveedor, que el oráculo no confirma).
- **Recomendación:** (a). Clasificación: comportamiento observable (tipo de error). Consecuencia: un proveedor desconocido, remote o cluster falla con el tipo y el mensaje de Pekko; Cada consumidor que Pekko envuelve conserva su tipo (ConfigurationException en Dispatchers; IllegalArgumentException al crear un MailboxType). Bloquea: AC-CFG-016 y AC-CFG-008 (cierre, quedan partial); puede avanzar: el resto. Reversible.
- **Filas afectadas:** AC-CFG-008;AC-CFG-016
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-32 — BootstrapSetup sin classLoader ni defaultExecutionContext (campos de la JVM sin equivalente en Rust)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** BootstrapSetup tiene cuatro campos (ActorSystem.scala:160-178): classLoader, config, defaultExecutionContext y actorRefProvider, con sus withers. ActorSystem.apply usa classLoader (findClassLoader() por defecto) y defaultExecutionContext (ActorSystem.scala:278-281). En Rust no hay ClassLoader (la carga por nombre es el registro de Q-X-2 y FactoryRegistry; Q-SYS-6, propuesta, recomienda no tener sobrecargas con ClassLoader) y el tipo del ejecutor lo define AC-DSP-006 (Q-X-4). Medido con el oráculo (casos BS-15 y BS-16, no producidos por Rust): withClassloader y withDefaultExecutionContext devuelven una copia que guarda el valor.
- **Alternativas:** (a) BootstrapSetup tiene por ahora solo `config` y `actorRefProvider`; `classLoader` no existe (registro de fábricas) y `defaultExecutionContext` se añade con AC-DSP-006 cuando exista el tipo del ejecutor. (b) Tipos opacos sin efecto (`ClassLoaderHandle`, `ExecutorHandle`) que se guardan y se devuelven. (c) Excluir los dos campos (requiere autorización explícita del Owner).
- **Recomendación:** (a). Clasificación: arquitectura y exclusión de alcance parcial. Consecuencia observable: no existen `withClassloader` ni `withDefaultExecutionContext`; el resto de BootstrapSetup se comporta como Pekko. Bloquea: el cierre de AC-CFG-015 (queda partial); puede avanzar: lo demás. Reversible.
- **Filas afectadas:** AC-CFG-015;AC-SYS-001.2;AC-DSP-006
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-33 — Orden y nombres de ActorSystemSetup.toString (Display)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystemSetup.toString lista `setups.keys.map(_.getName).mkString(",")` (ActorSystemSetup.scala:94). `setups` es un Map inmutable de Scala: con hasta cuatro setups conserva el orden de inserción (un setup reemplazado conserva su posición; casos TS-03..TS-06 del oráculo), y desde cinco es un HashMap ordenado por `Class.hashCode`, que depende de la identidad del objeto Class y cambia entre ejecuciones de la JVM: no es reproducible. Los nombres son nombres de clase de Java; en Rust cada tipo declara el suyo con `Setup::class_name()` (BootstrapSetup devuelve `org.apache.pekko.actor.BootstrapSetup`) y por defecto es `std::any::type_name`.
- **Alternativas:** (a) Orden de inserción siempre, con `class_name()` por tipo. (b) Orden alfabético de los nombres. (c) No ofrecer `Display` (solo `Debug` sin contrato).
- **Recomendación:** (a). Clasificación: comportamiento observable menor (solo diagnóstico). Consecuencia: con más de cuatro setups el orden de Pekko no es reproducible y Rust usa el de inserción; con cuatro o menos coincide con Pekko. Bloquea: nada. Reversible.
- **Filas afectadas:** AC-CFG-015
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-34 — Un `Custom` de `BootstrapSetup.withActorRefProvider` se lee por su `identifier` y queda como el nombre de clase `custom` (bug de Pekko 2.0.0)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Custom(fqcn) tiene identifier "custom" (ActorSystem.scala:118) y Settings lee el proveedor del setup con `setup.get[BootstrapSetup].flatMap(_.actorRefProvider).map(_.identifier).getOrElse(getString("pekko.actor.provider"))` (ActorSystem.scala:411-414) y lo pasa por `ProviderSelection(providerSelectionSetup)` (ActorSystem.scala:417): "custom" no es ninguno de los nombres conocidos y vuelve a ser `Custom("custom")`, así que se pierde la clase. Caso grabado as-0037 (PR-06): `BootstrapSetup().withActorRefProvider(Custom("other.ActorRefProvider"))` da `identifier=custom ProviderClass=custom HasCluster=false`; con Local, Remote o Cluster (as-0034..as-0036, PR-03..05) la selección sí se conserva, y por texto (`pekko.actor.provider = other.ActorRefProvider`, ps-0006) tampoco se pierde. Parece un bug de Pekko 2.0.0: nada documenta esa pérdida.
- **Alternativas:** (a) Reproducir el comportamiento de Pekko 2.0.0 (Q-X-11): un `Custom` del setup acaba como la clase `custom`; el código lleva un comentario que lo marca como paridad deliberada. (b) Conservar el FQCN del `Custom` del setup (corrige el bug; es una diferencia deliberada con Pekko que el oráculo no confirma).
- **Recomendación:** (a). Clasificación: comportamiento observable (bug de Pekko reproducido). Consecuencia: un proveedor personalizado solo es utilizable por configuración de texto, no por `BootstrapSetup`, como en Pekko 2.0.0. Bloquea: el cierre de AC-CFG-015 y AC-CFG-008 (quedan partial); puede avanzar: el resto. Reversible.
- **Filas afectadas:** AC-CFG-015;AC-CFG-008
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CFG-35 — `toString` de `Deploy`: formato de `Config.toString`, orden de los tags y texto del router

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** `Deploy.toString = s"Deploy($path, $config, $routerConfig, $scope, $dispatcher, $mailbox, $tags)"` (Deployer.scala:150). Medido en la JVM (sonda fuera del corpus, JDK 17 y JDK 27 idénticos): `Deploy(/a, Config(SimpleConfigObject({"a":5,"apple":2,"b":4,"mango":3,"zebra":1})), NoRouter, NoScopeGiven, , , Set())`; las claves de la configuración salen ordenadas (no dependen de un HashMap), con la forma `Config(SimpleConfigObject(` + render conciso + `))`. Lo que sí no es determinista o no es de este crate: (1) los `tags` son un `Set` de Scala, que con más de 4 elementos sale en orden de hash; (2) el texto de un router real (`RoundRobinPool(...)`) lo define el módulo de routing (P37); (3) el texto de los scopes. Hoy `Deploy` solo tiene `Debug`; ningún caso graba `toString`. Una versión anterior de este plan atribuyó el problema al orden de claves de la JVM: no es cierto, lo desmiente la medición.
- **Alternativas:** (a) `Display` con el formato exacto de Pekko, claves ordenadas, tags en orden de inserción hasta 4 y ordenados desde 5 (divergencia documentada), router por su nombre de clase hasta P37. (b) No implementar `Display`: `toString` queda fuera del contrato de Rust y se documenta. (c) `Display` propio y legible, sin pretender el formato de Pekko.
- **Recomendación:** (a). Clasificación: texto de diagnóstico, no comportamiento de datos. Consecuencia: `toString` solo se compara en los casos con config, router y tags deterministas; AC-CFG-013 queda `partial` hasta que el Owner decida. Bloquea: el cierre de AC-CFG-013; puede avanzar: todo el comportamiento de datos. Reversible.
- **Filas afectadas:** AC-CFG-013
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation

### Q-CS-1 — Fases: errores de config/validacion como Result y orden entre fases independientes

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:329-330,347-350,566-572; test solo comprueba tipo IllegalArgumentException | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (Result) + divergencia funcional menor (orden determinista entre ramas independientes). Resuelto por lectura: CoordinatedShutdown.scala:337-364 (DFS sobre `Set`, `result.reverse`); CoordinatedShutdownSpec:72-119 compara solo restricciones.
- **Alternativas:** (a) Result::Err con mismos mensajes; (b) panic. Orden de ramas independientes: arbitrario vs determinista
- **Recomendación:** (a) Result con mensajes de Pekko; orden determinista (BTreeSet) documentado como diferencia (Pekko usa Set sin orden). Consecuencias: orden entre fases independientes (no afirmado) y texto de errores de configuracion. Bloquea AC-CS-001.1/.2, 003.1; puede avanzar el resto.
- **Filas afectadas:** AC-CS-001.1;AC-CS-001.2;AC-CS-003.1
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-2 — Modelo de tareas de apagado: identidad por objeto, cancelacion, paralelismo y Future compartido

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:405-533,722-785 | Cierre 2026-10-31: Clasificacion: arquitectura (modelo de tareas con estado atomico y Future compartido). Resuelto por lectura: CoordinatedShutdown.scala:405-533,722-785.
- **Alternativas:** (a) replicar el modelo (tarea = Arc con estado atomico, Future compartido); (b) simplificar a lista de tareas
- **Recomendación:** (a); depende de Q-X-4 (runtime) y Q-X-9 (Done). Consecuencias: cancelacion, duplicados y concurrencia (CoordinatedShutdownSpec:170-363). Bloquea AC-CS-003.2, 004.1-.3, 008, 009.
- **Filas afectadas:** AC-CS-003.2;AC-CS-004.1;AC-CS-004.2;AC-CS-004.3;AC-CS-008;AC-CS-009
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-3 — Fases de cluster predefinidas sin cluster (vacias en local)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:1346-1397; CoordinatedShutdownSpec:123-139 | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (fases de cluster vacias). Resuelto por lectura: reference.conf y CoordinatedShutdownSpec:123-139 (afirma la lista completa).
- **Alternativas:** (a) mantener todas las fases (paridad de orden y nombres); (b) proponer quitar las de cluster (no autorizado)
- **Recomendación:** (a): el spec afirma la lista completa; cluster queda fuera del modulo pero las fases se mantienen. Consecuencias: nombres y orden de fases. Bloquea AC-CS-002.
- **Filas afectadas:** AC-CS-002
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-4 — Identificador estable de Reason y claves reason-overrides

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:1282-1291 (clave = nombre de clase JVM); CoordinatedShutdown.scala:206-213 | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (identificador estable). Resuelto por lectura: CoordinatedShutdown.scala:206-213 (`r.getClass.getName`, p. ej. "org.apache.pekko.actor.CoordinatedShutdown$ClusterDowningReason$"); CoordinatedShutdownSpec:528-533 solo prueba ClusterDowningReason.
- **Alternativas:** (a) cada Reason expone una cadena estable igual al nombre de clase JVM de Pekko; (b) nueva convencion de claves
- **Recomendación:** (a) para compatibilidad de configuracion HOCON (Q-X-1). Consecuencias: claves de reason-overrides en config. Bloquea AC-CS-005 A3.
- **Filas afectadas:** AC-CS-005
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-5 — exit-jvm / System.exit en Rust y terminacion del ActorSystem desde la fase final

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:239-277; ActorSystem.scala:478-481,1086-1112 | Cierre 2026-10-31: Clasificacion: arquitectura (proceso vs JVM) con representacion que conserva claves. Resuelto por lectura: CoordinatedShutdown.scala:239-277 y ActorSystem.scala:474-482,1086-1112; CoordinatedShutdownSpec no prueba la salida del proceso.
- **Alternativas:** (a) `std::process::exit(code)` mantenido bajo el nombre de clave `exit-jvm`; (b) clave nueva `exit-process`
- **Recomendación:** (a) mantener claves; documentar que "jvm" significa proceso. Consecuencias: exit-jvm en Rust cierra el proceso. Bloquea AC-CS-006.1 A2/A3; AC-CS-006.2 puede avanzar.
- **Filas afectadas:** AC-CS-006.1;AC-CS-006.2
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-6 — Hook de apagado del proceso: senales y registro en el runtime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdown.scala:279-297,844-889; reference.conf:1268-1271 | Cierre 2026-10-31: Clasificacion: arquitectura (senales del proceso) + divergencia funcional (SIGINT). Resuelto por lectura: CoordinatedShutdown.scala:279-297,830-913 y reference.conf; CoordinatedShutdownSpec:584-659,757-803 prueba el recuento de hooks con un runtime simulado (JVMShutdownHooks) — el equivalente Rust debe abstraer el registro de hooks para poder probarlo igual.
- **Alternativas:** (a) tokio::signal (SIGTERM, y SIGINT como extension); (b) ctrlc/signal-hook; (c) solo API manual
- **Recomendación:** (a); SIGINT soportado es una diferencia observable respecto de Pekko (doc dice que SIGINT no funciona). Consecuencias: registro de hooks y ejecucion desde senal. Bloquea AC-CS-007.1/.2; el resto de CS no depende.
- **Filas afectadas:** AC-CS-007.1;AC-CS-007.2
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-CS-7 — Formato textual de Duration y colecciones en mensajes de log/error de CoordinatedShutdown y FSM

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CoordinatedShutdownSpec:402-440 afirma el warning "Coordinated shutdown phase [b] timed out after 100 milliseconds" (toString de FiniteDuration de Scala: "100 milliseconds"); FSMActorSpec:296 afirma "setting timer 't'/1500 milliseconds: Shutdown"; el cycle error incluye `Map(...)`/`Set(...)` de Scala (CoordinatedShutdown.scala:347-350,568); la variante CompletionStage de timeout usa Duration.toString de Java ("PT0.2S", PatternsTest.java:552-562).
- **Alternativas:** (a) implementar un Display de Duration con el formato de Scala ("100 milliseconds", "1 second", "1500 milliseconds"); (b) formato propio de Rust y adaptar los tests portados
- **Recomendación:** (a) para los textos que los specs afirman (timeouts de fase de CS, timers de FSM) mediante un unico Display compartido para Duration (tambien util a Q-FSM-3); el formato de colecciones del mensaje de ciclo y el ISO-8601 de la API Java quedan fuera de paridad (no afirmados). Consecuencias: tests portados de CS-004.2 y FSM-007 exigen el formato. Bloquea AC-CS-004.2 A1, AC-FSM-004/007 textos; puede avanzar todo lo demas.
- **Filas afectadas:** AC-CS-004.2;AC-CS-008;AC-FSM-004;AC-FSM-007;AC-ASK-008
- **Unidades afectadas:** CS
- **Origen:** classic-patterns

### Q-DSP-1 — Qué claves de executor se honran: fork-join, thread-pool, affinity-pool, virtual-thread

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:371-585; ForkJoinExecutorConfigurator.scala:43-49,105,120-234; ThreadPoolBuilder.scala:44-64,86-132,293-302; AffinityPool.scala:35-444; AbstractDispatcher.scala:370-465; VirtualizedExecutorService.scala:25-117
- **Alternativas:** (a) honrar todas las claves mapeándolas a un pool propio; (b) honrar parallelism-*, fixed-pool-size, thread-pool-*, affinity-*; mapear minimum-runnable, maximum-pool-size, virtualize a "aceptadas y validadas, con efecto documentado o sin efecto, con warning"; (c) pools propios independientes del runtime
- **Recomendación:** [Clasificación: arquitectura con exclusión parcial de efecto] (b) con equivalentes: fork-join -> pool work-stealing propio (FIFO por defecto; LIFO si se configura); thread-pool -> pool con cola acotada/no acotada y política "caller runs salvo apagado"; affinity-pool -> pool con cola acotada por worker y selector portado (sólo con hilos propios, Q-X-4 A); virtual-thread / virtualize -> tareas async (cumple la intención, sin nombres de hilo); `minimum-runnable` y `maximum-pool-size` se validan sin efecto (ver Q-DSP-6). Diferencias observables documentadas: nombres de hilo, compensación ante bloqueo, `atFullThrottle`. Sin exclusión: todas las claves se leen y validan. Bloquea la implementación de AC-DSP-008/009/013/014/019; puede avanzar: lectura/validación de claves y las funciones puras `scaledPoolSize` y `resolveMinimumRunnable`.
- **Filas afectadas:** AC-DSP-006,AC-DSP-008,AC-DSP-009,AC-DSP-010,AC-DSP-013,AC-DSP-014,AC-DSP-019
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-2 — Registro de configuradores de dispatcher/executor/mailbox por FQCN (type, executor, mailbox-type)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatchers.scala:197-300,381-407; AbstractDispatcher.scala:353-410; Mailboxes.scala:241-255; DispatchersSpec.scala:177-185; ActorSystemDispatcherSpec.scala:170-174
- **Alternativas:** (a) registro por nombre; (b) nombres FQCN literales de Pekko como claves alias
- **Recomendación:** [Clasificación: arquitectura; subordinada a Q-X-2] Ver Q-X-2; aceptar los nombres FQCN de Pekko como alias de los builtin (`Dispatcher`, `PinnedDispatcher`, `org.apache.pekko.dispatch.BalancingDispatcherConfigurator`, los cinco `executor`, los mailbox-type). Consecuencia: un nombre no registrado devuelve un error de configuración con el nombre. Bloquea sólo la carga por nombre (AC-DSP-004/010, AC-MBX-004); el registro programático puede avanzar.
- **Filas afectadas:** AC-DSP-004,AC-DSP-010,AC-MBX-004
- **Unidades afectadas:** DSP,MBX
- **Origen:** classic-dispatch

### Q-DSP-3 — ExecutionContext/Future helpers (AC-DSP-017): portar sólo parasitic o todo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Future.scala:36-455 y CompletionStages.scala:28-220 (API Java sobre scala.concurrent.Future/CompletionStage); SameThreadExecutionContext (3 copias); SerializedSuspendableExecutionContext.scala:24-110; ExecutionContextSpec.scala:36-53,153-166,211-293; CompletionStagesTests.java; JavaFutureTests.java
- **Alternativas:** (a) portar `InlineExecutor` (parasitic) y `SerializedSuspendable`; excluir Futures/CompletionStages/japi; (b) sólo el trait Executor; (c) todo
- **Recomendación:** [Clasificación: exclusión de alcance (propuesta) para AC-DSP-017.2; el resto, necesario] (a): AC-DSP-017.1 (InlineExecutor) y AC-DSP-018 (SerializedSuspendable, utilidad interna con comportamiento afirmado) se portan; AC-DSP-017.2 (API Java de futuros: Futures, CompletionStages, japi) se documenta como equivalente `std::future`/`futures` sin API propia. La fila AC-DSP-017.2 sigue en alcance hasta autorización del Owner. Consecuencia: las pruebas Java (CompletionStagesTests, JavaFutureTests) no se portan. Bloquea sólo AC-DSP-017.2.
- **Filas afectadas:** AC-DSP-016.1,AC-DSP-017.1,AC-DSP-017.2,AC-DSP-018
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-4 — Detección de ciclos de alias de dispatcher y profundidad máxima

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatchers.scala:77-95 (getConfig: profundidad > 20 -> Config vacía), :157-161 (lookup: ConfigurationException tras 20 alias); ActorSystemDispatcherSpec.scala:170-174 (bucle de alias -> ConfigurationException); DispatchersSpec.scala:198-228
- **Alternativas:** (a) paridad con 20
- **Recomendación:** [RESUELTA POR LECTURA 2026-10-31; Clasificación: representación que conserva el contrato] (a) paridad con MaxDispatcherAliasDepth = 20 y los textos literales (AC-DSP-004, AC-DSP-022). No requiere decisión del Owner. No bloquea.
- **Filas afectadas:** AC-DSP-004,AC-DSP-022
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-5 — throughput <=0: documentación (continúa hasta vaciar) vs código (equivale a 1)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatcher.scala:37-40 (Scaladoc: "Zero or negative means the dispatcher always continues until the mailbox is empty"); Mailbox.scala:263-280 (`left = max(throughput,1)`, el bucle sólo continúa si `left > 1`); reference.conf:589-591; DispatcherActorSpec.scala:84-141 (sólo ejerce throughput 101 y 2)
- **Alternativas:** (a) seguir el código (0/neg = 1); (b) seguir la doc (0/neg = vaciar mailbox)
- **Recomendación:** [RESUELTA POR LECTURA 2026-10-31; Clasificación: representación que conserva el contrato] (a): el código es lo observable (throughput <= 0 equivale a 1 mensaje por turno) y ningún spec afirma (b); registrar la diferencia con el Scaladoc. No requiere decisión del Owner salvo que se desee seguir la documentación. No bloquea.
- **Filas afectadas:** AC-DSP-003,AC-MBX-002
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-6 — minimum-runnable = -1 y valor por defecto efectivo en Rust (sin JDK)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ForkJoinExecutorConfigurator.scala:33-49 (resolveMinimumRunnable: configurado >= 0 verbatim; -1 con JDK >= 25 -> min(8, max(1, parallelism/2)); si no 1); reference.conf:491-507 (comentario dice "-1 (default)" pero la línea efectiva es `minimum-runnable = 1`); ForkJoinExecutorConfiguratorSpec.scala:76-121,145-175 (el caso "auto-scale the default on JDK 25+", :153-162, lee los defaults de reference.conf y espera >= 4: incoherente con el valor 1 efectivo; hoy `pending` en JDK < 25, no resoluble sin ejecutar con JDK 25)
- **Alternativas:** (a) default efectivo 1 y -1 equivale a 1 (comportamiento "JDK < 25"); (b) portar `resolveMinimumRunnable(configured, parallelism, jdk_major)` como función pura parametrizada y usar jdk_major = "legacy" en Rust; (c) aplicar siempre la política de JDK >= 25
- **Recomendación:** [Clasificación: divergencia funcional (no hay JDK en Rust)] (b): la función pura se porta y se prueba con el oráculo (A1-A6 de AC-DSP-008) y en tiempo de ejecución el valor efectivo por defecto es 1; -1 se comporta como en JDK < 25. Consecuencia observable: ninguna en Rust (el parámetro no tiene efecto con un pool propio salvo que se implemente compensación). No bloquea nada; la incoherencia de reference.conf/spec queda registrada como contradicción.
- **Filas afectadas:** AC-DSP-008,AC-DSP-019
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DSP-7 — BatchingExecutor y reenvío ante bloqueo (BlockContext) en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** BatchingExecutor.scala:23-147; AbstractDispatcher.scala:109,173-174 (resubmitOnBlock = true); ThreadPoolBuilder.scala:212-240 (PekkoForkJoinWorkerThread como BlockContext); ExecutionContextSpec.scala:55-208 (no-inline y serie, evitar inanición con Await/blocking, CallingThreadDispatcher como executor)
- **Alternativas:** (a) lote en thread_local como optimización + hook explícito `blocking(|| ..)` que reenvía el resto del lote (sin detección automática); (b) no portar el lote ni el reenvío; (c) lote sin reenvío y documentar que el código bloqueante debe ir a un dispatcher bloqueante
- **Recomendación:** [Clasificación: divergencia funcional] (a) mínimo: lote opcional con las garantías no-inline y ejecución en serie (A1 de AC-DSP-016.2) y un hook explícito de bloqueo; las pruebas A2-A4, A6, A7 de AC-DSP-016.2 (que dependen de `scala.concurrent.blocking`/`Await`) se adaptan al hook y se anotan como adaptadas. Consecuencia observable: no hay compensación automática de hilos bloqueados (ver Q-DSP-1). Bloquea la implementación de AC-DSP-016.2; no bloquea nada más.
- **Filas afectadas:** AC-DSP-016.2
- **Unidades afectadas:** DSP
- **Origen:** classic-dispatch

### Q-DW-1 — Igualdad del mensaje Terminated y existenceConfirmed para actores ya muertos

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:112-135 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:746-752 ; actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:54-72 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:353 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:99-115 ; actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:76,87 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:250-261
- **Alternativas:** (a) PartialEq sólo por actor (como la case class de Pekko con segunda lista de parámetros) y existenceConfirmed=false al vigilar un actor ya muerto / (b) PartialEq por los tres campos / (c) por actor y existenceConfirmed
- **Recomendación:** (a): los specs hacen expectMsg(Terminated(a)(true,false)) y esperan igualdad por actor; documentar que existenceConfirmed no entra en equals [2026-10-31 cierre documental] Clasificación: representación que conserva el contrato. Resuelto leyendo: Terminated(actor)(existenceConfirmed, addressTerminated) compara y hashea sólo `actor` (case class con segunda lista de parámetros, Actor.scala:123-125); existenceConfirmed=false al vigilar un actor ya muerto porque su mailbox es el deadLetterMailbox (Mailboxes.scala:62-63 → DeadLetterActorRef.specialHandle, ActorRef.scala:746-752); es true cuando el Terminated llega por el camino normal (RoutingSpec.scala:76,87; ActorSystemSpec.scala:353). Consecuencia: `assert_eq!(Terminated{a,true,false}, Terminated{a,false,true})` debe cumplirse. Bloquea: `impl` de AC-DW-002/006 (PartialEq). Puede avanzar: el resto de DW.
- **Filas afectadas:** AC-DW-002 ; AC-DW-006
- **Unidades afectadas:** DW
- **Origen:** classic-cell

### Q-DW-2 — Hooks de remote en DeathWatch: AddressTerminated, AddressTerminatedTopic y orden remoto→local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:133-163,217-264 ; actor/src/main/scala/org/apache/pekko/event/AddressTerminatedTopic.scala:30-72 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:35-79,241-263 ; actor-tests/src/test/scala/org/apache/pekko/event/AddressTerminatedTopicBenchSpec.scala:25-62
- **Alternativas:** (a) conservar el hook is_local en el trait de ref y el topic sin uso local / (b) omitir el topic y el orden / (c) implementarlo completo con un provider simulado
- **Recomendación:** (a) mínimo: el trait de ref expone is_local y la celda mantiene los conjuntos watching/watchedBy con la lógica de suscripción, aunque sin remote nunca se suscriba [2026-10-31 cierre documental] Clasificación: arquitectura / exclusión de alcance (remote). Resuelto leyendo: sin proveedor remoto `hasNonLocalAddress` es siempre falso, no hay suscripción y `AddressTerminated` no se publica; el hook mínimo (is_local + conjuntos watching/watchedBy + suscripción) permite un remote futuro sin cambiar la celda. Consecuencia: ningún test funcional upstream (sólo el benchmark). Bloquea: `impl` de AC-DW-012 y el orden remoto→local de AC-DW-010. Puede avanzar: el resto de DW (el orden local es el único observable sin remote).
- **Filas afectadas:** AC-DW-010 ; AC-DW-012
- **Unidades afectadas:** DW
- **Origen:** classic-cell

### Q-EVT-2 — Fidelidad byte a byte de la salida stdout de los loggers por defecto

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Logging.scala:993-1105 (formatos), Helpers.scala:108-113 (timestamp MM/dd/yyyy HH:mm:ss.SSS en zona del sistema), LoggerSpec.scala:319-345.
- **Alternativas:** (a) byte-compatible con Pekko (thread = nombre de hilo del runtime); (b) formato propio con `tracing`; (c) ambos: StandardOutLogger fiel y puente opcional a `tracing`
- **Recomendación:** [Clasificación: arquitectura (backend de logging)] (c), con (a) como default de DefaultLogger/StandardOutLogger. Consecuencias observables: formato byte a byte de stdout, hilo y marca de tiempo. Bloquea: AC-EVT-011/017 en su validación diferencial; el resto avanza.
- **Filas afectadas:** AC-EVT-011 ; AC-EVT-017
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-4 — JavaLogger (java.util.logging) en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** JavaLogger.scala:32-120 (deprecado, JVM). Cierre 2026-10-31 (leídos JavaLogger.scala:29-121 y JavaLoggerSpec.scala:26-97): mapa de niveles Info->INFO, Debug->CONFIG, Warning->WARNING, Error->SEVERE, otros->FINE; los campos observables son nivel, mensaje, thrown, threadID, sourceClassName y sourceMethodName=null.
- **Alternativas:** (a) puente opcional a la fachada `log`; (b) sin equivalente (requiere exclusión explícita)
- **Recomendación:** [Clasificación: representación que conserva el contrato (con exclusión de los tipos JUL)] (a): puente opcional a la fachada `log` con un RecordingLogger de pruebas que captura nivel, mensaje, causa, hilo y clase origen; CONFIG pasa a Debug y FINE a Trace. Consecuencias observables: sin LogRecord.getMillis ni getSourceMethodName. Bloquea: nada; AC-EVT-021 avanza detrás de la feature `log-bridge`.
- **Filas afectadas:** AC-EVT-021
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-5 — API Java (EventBusJavaAPI, javadsl TestKit/EventFilter)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** event/japi/EventBusJavaAPI.scala; testkit javadsl/*. Cierre 2026-10-31: EventBusJavaAPI.scala son cuatro fachadas 1:1 sin lógica (delegan en traits Scala); javadsl.TestKit delega en TestProbe (TestKit.scala:56) y javadsl.EventFilter construye ErrorFilter/WarningFilter/InfoFilter/DebugFilter (EventFilter.scala:47-61); no hay specs ejecutables propios (TestKitApiTest sólo compila).
- **Alternativas:** (a) la misma API única en Rust (no existe distinción Java); (b) módulo de compatibilidad con nombres Java
- **Recomendación:** [Clasificación: representación que conserva el contrato] (a); la fila queda cubierta por AC-EVT-002..005 y AC-TK-001/005. Consecuencias observables: los nombres alternativos Java (getRef, expectMsgEquals, expectNoMsg) no existen salvo alias. Bloquea: nada.
- **Filas afectadas:** AC-EVT-022 ; AC-TK-011
- **Unidades afectadas:** EVT ; TK
- **Origen:** classic-system

### Q-EVT-6 — Asimetría de LogSource.fromAnyRef con sistema (getLogger vs Logging.apply)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Logging.scala:395-402: fromAnyRef(o, system) llama apply(c|a|s) sin sistema para Class/Actor/ActorRef/String, de modo que getLogger(system, "x") usa logSource "x" y Logging(system, "x") usa "x(pekko://sys)" (Logging.scala:318-322, 573-576, 679-682). Ningún spec afirma la diferencia.
- **Alternativas:** (a) replicar la asimetría; (b) unificar (siempre incluir la dirección); (c) unificar sin dirección
- **Recomendación:** [Clasificación: incertidumbre resuelta por lectura; replicar es sin coste] (a): conserva el comportamiento de Pekko; si el Owner prefiere unificar es una divergencia a registrar en `notes`. Consecuencias observables: sólo el texto de logSource de getLogger. Bloquea: nada.
- **Filas afectadas:** AC-EVT-013.3
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EVT-7 — LoggingReceive exige is_defined_at antes de ejecutar el Receive

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** LoggingReceive.scala:87-105 registra en isDefinedAt (antes de apply) con el texto handled/unhandled; LoggingReceiveSpec.scala:96-128 afirma el orden Debug -> respuesta "x" y que un orElse que atiende antes el mensaje no registra.
- **Alternativas:** (a) Receive basado en tabla de casos tipados con `is_defined_at(&msg)` separado de `apply` (también útil para unhandled y become); (b) closures opacas y registrar tras ejecutar (rompe el orden observable); (c) macro `receive!` que genera ambos métodos
- **Recomendación:** [Clasificación: arquitectura (modelo de Receive de classic)] (a) o (c). Consecuencias observables: con (b) el Debug llega después de los efectos del handler. Bloquea: AC-EVT-019 (y el log de FSM con debug.fsm); relacionada con Q-X-3.
- **Filas afectadas:** AC-EVT-019
- **Unidades afectadas:** EVT
- **Origen:** classic-system

### Q-EXT-1 — Identidad del ExtensionId

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Extension.scala:123-124 (hashCode = identityHashCode, equals = eq, ambos final); ExtensionSpec.scala:43-53 (singletons).
- **Alternativas:** (a) `TypeId` del tipo que implementa ExtensionId (singleton por tipo); (b) dirección de un `static`; (c) clave explícita `&'static str`.
- **Recomendación:** (a) con una nota sobre extensiones parametrizadas. [Cierre documental 2026-10-31] Clasificación: representación (identidad del ExtensionId). Consecuencias observables: equals/hashCode por identidad (Extension.scala:123-124): dos instancias del mismo ExtensionId son extensiones distintas. Con TypeId esa propiedad desaparece para ExtensionIds del mismo tipo con distintos parámetros (nota). Bloquea: AC-EXT-001.1, AC-EXT-002 y, por el índice, AC-EXT-004; puede avanzar: AC-EXT-003.
- **Filas afectadas:** AC-EXT-001.1, AC-EXT-002
- **Unidades afectadas:** EXT
- **Origen:** classic-foundation

### Q-EXT-2 — Semántica de registerExtension: bloqueo, reentrada, caché de fallo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:1165-1221; ExtensionSpec.scala:82-97 (misma excepción en el 2.º intento, efectos laterales no revertidos); mensaje de timeout con `creation-timeout`.
- **Alternativas:** (a) mismo diseño con condvar y `creation-timeout`; (b) creación una sola vez con `OnceLock` y panic/Err cacheado; (c) async con `tokio::sync::OnceCell`.
- **Recomendación:** Depende de Q-X-4; (a) para runtime con hilos, (c) para async. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA (depende de Q-X-4). Consecuencias observables: reentrada, caché del fallo y mensaje de timeout con `creation-timeout`. Bloquea: AC-EXT-001.2; puede avanzar: AC-EXT-002.
- **Filas afectadas:** AC-EXT-001.2
- **Unidades afectadas:** EXT
- **Origen:** classic-foundation

### Q-EXT-3 — AC-EXT-004 como fila índice de extensiones provistas

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Las extensiones están en actor/CoordinatedShutdown.scala:42, serialization/SerializationExtension.scala:24, io/Tcp.scala:50, pattern/CircuitBreakersRegistry.scala:33, event/AddressTerminatedTopic.scala:38; cada una pertenece a otra unidad.
- **Alternativas:** (a) mantener la fila como índice y cerrarla cuando se cierren las filas hijas; (b) repartir cada extensión en su unidad.
- **Recomendación:** (a). [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA documental. Estado tras el cierre: AC-EXT-004 ya está extraída como índice de 11 extensiones con su fila dueña; la decisión sólo afecta a cómo se cierra la fila (índice con impl derivado de las hijas, recomendación (a)). Bloquea: el cierre de impl de AC-EXT-004; puede avanzar: todo lo demás.
- **Filas afectadas:** AC-EXT-004
- **Unidades afectadas:** EXT
- **Origen:** classic-foundation

### Q-FSM-1 — FSM en Rust: funciones parciales (when/whenUnhandled/transform), tipos S y D, respuestas y mensajes dinamicos

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** FSM.scala:439-441 (PartialFunction[Event, State]); 774-792 (orElse); 306-308 (replying Any) | Cierre 2026-10-31: Clasificacion: arquitectura (tipado) con representacion que conserva el contrato. Resuelto por lectura de FSM.scala y de todos los specs (FSMActorSpec, FSMTimingSpec, FSMTransitionSpec): todos los casos usan estados/datos como valores comparables por igualdad y mensajes `Any`; requiere `S: Eq+Hash` (stateFunctions es un mutable.Map[S, ...]).
- **Alternativas:** (a) trait Fsm con tablas de closures Option-returning y `Box<dyn Any>` para mensajes (Q-X-3); (b) enum cerrado de eventos por FSM (solo tipado)
- **Recomendación:** (a) para paridad con classic; (b) corresponde a Typed (Q-X-8). Consecuencias: forma de la API publica del trait Fsm. Bloquea el diseno de AC-FSM-001.1/.2, 002, 008, 009; la logica de State inmutable y de timers puede prototiparse sin ella.
- **Filas afectadas:** AC-FSM-001.1;AC-FSM-001.2;AC-FSM-002;AC-FSM-008;AC-FSM-009
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-FSM-2 — Timers de FSM y state timeout: sobre el scheduler y generacion de cancelacion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** FSM.scala:105-160,814-829,896-907 | Cierre 2026-10-31: Clasificacion: arquitectura (scheduler) con representacion que conserva el contrato. Resuelto por lectura: FSM.scala:97-155,814-829,896-907 (generacion Int/Long, TimeoutMarker, Timer con owner).
- **Alternativas:** (a) replicar generacion (TimeoutMarker/Timer con gen) sobre el scheduler de AC-SCH; (b) timers locales de tokio
- **Recomendación:** (a) (paridad de cancelacion de mensajes ya encolados); depende de Q-X-4. Consecuencias: semantica "at most once tras cancelar" (FSMTimingSpec:100-127). Bloquea AC-FSM-003 y AC-FSM-004; no bloquea el resto de FSM.
- **Filas afectadas:** AC-FSM-003;AC-FSM-004
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-FSM-3 — Textos de log exactos de FSM y debug.fsm

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** FSM.scala:616-617,631-632,787-790,981-1004 ; FSMActorSpec:262-312 | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (textos de log). Resuelto por lectura: FSMActorSpec:262-312 fija los textos "setting timer 't'/1500 milliseconds: Shutdown", "transition 1 -> 2", "canceling timer 't'" y los prefijos "processing Event(go,null) from Actor[" (nota: el formato de Duration es el toString de Scala, ver Q-CS-7).
- **Alternativas:** (a) emitir exactamente los textos de Pekko por el logging de rakka (Q-X-7); (b) estructurar los campos
- **Recomendación:** (a): son aserciones de spec (prefijos y textos completos en :296,:297,:304). Consecuencias: solo logs de depuracion y warning "unhandled event ..."; bloquea AC-FSM-005, AC-FSM-007 A1/A3; el log rotativo (getLog) puede avanzar.
- **Filas afectadas:** AC-FSM-005;AC-FSM-007
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns

### Q-IO-1 — Modelo de ejecución de pekko.io: selectores + pinned-dispatcher + actores de canal vs tokio/mio con actores-conexión

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** SelectionHandler.scala:84-387 (bucle select, interest ops); reference.conf:930-1138 (nr-of-selectors, selector-dispatcher, worker-dispatcher, select-timeout); TcpConnectionSpec.scala:883-1142 (ChannelRegistry simulado); Q-X-4
- **Alternativas:** (a) Reproducir selectores sobre mio con un hilo por selector y actores de canal que reciben ChannelReadable/Writable; (b) actores-conexión sobre tokio TcpStream/UdpSocket manteniendo el protocolo de mensajes público y descartando interest ops; (c) híbrido: costura ChannelRegistry para portar los specs de conexión y tokio detrás
- **Recomendación:** (c): protocolo público idéntico (Bind/Connect/Register/Write...), costura `ChannelRegistry` interna para poder portar TcpConnectionSpec/TcpListenerSpec (Q-IO-2), y claves nr-of-selectors/selector-dispatcher/worker-dispatcher/select-timeout aceptadas con efecto documentado (probable no-op). Depende de Q-X-4 | Clasificación (cierre documental 2026-10-31): arquitectura.
- **Filas afectadas:** AC-IO-001; AC-IO-002.2; AC-IO-002.3; AC-IO-002.4; AC-IO-002.7; AC-IO-003; AC-IO-004.1; AC-IO-004.2; AC-IO-004.4; AC-IO-006.1; AC-IO-006.2; AC-IO-007; AC-IO-008.5; AC-IO-011
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-2 — Estrategia de port de specs de conexión que simulan el selector

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TcpConnectionSpec.scala:96-1142 (36 tests con selector.send(ChannelReadable/Connectable)); TcpListenerSpec.scala:36-142; TcpIntegrationSpec.scala:41-205 (caja negra)
- **Alternativas:** (a) Portar solo los specs de integración (caja negra); (b) portar también los de conexión sobre un ChannelRegistry falso inyectable; (c) reescribir los de conexión como integración en loopback
- **Recomendación:** (b) si Q-IO-1 adopta la costura ChannelRegistry; si no, (c) manteniendo las aserciones de secuencia de mensajes. Los tests pending/ignore (TcpConnectionSpec.scala:109,316-320) no son aserciones | Clasificación (cierre documental 2026-10-31): arquitectura (estrategia de validación, no de comportamiento).
- **Filas afectadas:** AC-IO-002.2; AC-IO-002.3; AC-IO-002.4; AC-IO-002.5; AC-IO-002.6; AC-IO-004.4
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-3 — Piezas NIO/JVM/SO específicas: buffers directos y limpieza, workaround de Windows, finish-connect-retries, transferTo, DatagramChannelCreator, textos de error del SO

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** DirectByteBufferPool.scala:38-106; ByteBufferCleaner.java:21-100; reference.conf:964-1025; TcpConnection.scala:224-250,533-549; Inet.scala:86-101; TcpConnectionSpec.scala:97-98 (mensajes por plataforma)
- **Alternativas:** (a) Portar todo con equivalentes (pool de BytesMut, socket2, sendfile, factoría de sockets); (b) aceptar las claves y no implementar ByteBufferCleaner, windows-workaround, finish-connect-retries; (c) excluir WritePath/WriteFile
- **Recomendación:** (a) para pool de buffers, opciones de socket (socket2), WritePath (tokio::fs + sendfile) y DatagramSocketFactory; (b) para ByteBufferCleaner (Drop), windows-connection-abort-workaround y finish-connect-retries, cuya exclusión se propone al Owner; el texto de ErrorClosed/CommandFailed.cause será el del SO (std::io::Error) y los tests lo comparan contra constantes por plataforma como hace Pekko | Clasificación (cierre documental 2026-10-31): exclusión de alcance parcial y representación.
- **Filas afectadas:** AC-IO-002.1; AC-IO-002.3; AC-IO-002.5; AC-IO-002.6; AC-IO-003; AC-IO-005; AC-IO-006.3; AC-IO-010; AC-IO-013
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-4 — Resolver inet-address: caché y propiedades JVM, preferencia IPv6, API deprecada y clave de caché Ip()

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** InetAddressDnsResolver.scala:49-174; InetAddressDnsResolverSpec.scala:31-149; Dns.scala:76-160,257-267 (preferIPv6Addresses); SimpleDnsCacheSpec.scala:77 (TODO)
- **Alternativas:** (a) "default" = 30 s positivo / Never negativo, sin propiedades de entorno, IPv4 primero; (b) leer variables de entorno análogas; (c) portar también la API deprecada Dns.Resolve/Resolved y reproducir la clave de caché Ip() inconsistente
- **Recomendación:** (a) para TTL y IPv6 (las propiedades networkaddress.cache.* y java.net.preferIPv6Addresses no existen); no portar la API deprecada salvo que el Owner la pida; la clave Ip() inconsistente se corrige (clave por flags reales) y se anota la diferencia | Clasificación (cierre documental 2026-10-31): divergencia funcional (propiedades JVM inexistentes en Rust).
- **Filas afectadas:** AC-IO-002.3; AC-IO-008.1; AC-IO-008.3; AC-IO-009.8
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-5 — Nombres de host en UDP y UdpConnected: Send con miss de caché y doConnect que ignora la dirección resuelta

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** WithUdpSend.scala:58-78 (miss -> CommandFailed inmediato); UdpConnection.scala:55-63,82-92 (doConnect(address) usa remoteAddress); UdpIntegrationSpec.scala:62-73; UdpConnectedIntegrationSpec.scala:61-77; MEDIDO 2026-10-31 con el jar 1.7.1 (ORACLE=jars, jshell local): UdpConnected.Connect a InetSocketAddress.createUnresolved("localhost",40999) -> CommandFailed; a una dirección ya resuelta -> Connected; a "doesnotexist.local" -> CommandFailed. UdpConnection.scala:88 channel.connect(remoteAddress) usa la dirección sin resolver.
- **Alternativas:** (a) Replicar exactamente (UdpConnected a host por nombre siempre falla; el primer Send a nombre falla); (b) corregir: conectar a la dirección resuelta y esperar la resolución en Send; (c) corregir solo UdpConnected
- **Recomendación:** (b) con diferencia documentada: es un defecto de 1.7.1 CONFIRMADO con el jar (ya no hace falta correr más oráculo). Consecuencia observable: Connect por nombre siempre falla en Pekko; si se corrige, un programa que dependía del CommandFailed pasa a conectar. Qué bloquea: sólo el cierre de impl de AC-IO-007 (Connect por nombre) y AC-IO-006.2 (Send con nombre); puede avanzar el resto de UDP | Clasificación (cierre documental 2026-10-31): incertidumbre RESUELTA por medición; queda una divergencia funcional (corregir un defecto de 1.7.1) que sólo decide el Owner.
- **Filas afectadas:** AC-IO-006.2; AC-IO-007
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-IO-6 — Nameservers por defecto del async-dns y alcance de /etc/resolv.conf

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** DnsSettings.scala:62-80,110-127,195-253 (JNDI y sun.net.dns.ResolverConfiguration); ResolvConfParser.scala:42-76 (solo search/domain/ndots); DnsSettingsSpec.scala:42-48 (solo "no lanza")
- **Alternativas:** (a) Leer líneas nameserver de /etc/resolv.conf (Unix) y APIs del SO en Windows/macOS; (b) exigir nameservers explícitos (error IllegalState con el mismo texto); (c) crate externo (hickory/resolv-conf)
- **Recomendación:** (a) en Unix con (b) como error cuando la lista queda vacía, con el mismo texto de IllegalStateException; Windows/macOS: pendiente de decisión del Owner. Pekko no lee "nameserver" del archivo, por lo que es una ampliación observable | Clasificación (cierre documental 2026-10-31): divergencia funcional (ampliación observable: leer nameserver de /etc/resolv.conf).
- **Filas afectadas:** AC-IO-009.1; AC-IO-009.2; AC-IO-009.6
- **Unidades afectadas:** IO
- **Origen:** classic-io-ser

### Q-JAPI-1 — Superficie Java-facing (AbstractActor y familia, AbstractFSM, japi.function, EventBus Java, sobrecargas, fábricas IO, Props.create con Creator/lambda)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AbstractActor.scala:32-427; AbstractFSM.scala:26-586; AbstractProps.scala:28-138; EventBusJavaAPI.scala:26-230; JavaAPI.java:66-161; Tcp.scala:702-; Udp.scala:257-363; FACHADA DEMOSTRADA: ActorCell implementa AbstractActor.ActorContext (ActorCell.scala:426) y getContext es un cast (AbstractActor.scala:236); AbstractFSM reenvía a FSM (AbstractFSM.scala:86-204); EventBusJavaAPI contiene un bus Scala anónimo y reenvía 1:1 (EventBusJavaAPI.scala:57-230); TcpMessage/UdpMessage/UdpConnectedMessage construyen case classes (Tcp.scala:702-933); sobrecargas Duration/Optional/CompletionStage convierten y delegan (AC-JAPI-008). COMPORTAMIENTO DIFERENCIAL propio: ReceiveBuilder (AC-JAPI-012), Match/PFBuilder (AC-JAPI-004.x), builders FSM (AC-JAPI-003), validaciones de Props.create (ActorCreationTest.java:186-341, AC-JAPI-006), JavaPartialFunction (AC-JAPI-013), japi.Option (AC-JAPI-005.1), getReceiveTimeout (AC-JAPI-001). Deprecaciones: FI.* y japi.{Function,Procedure,Effect,Predicate,Creator,Option} deprecadas, retiro anunciado en Pekko 2.0.0 (FI.java:28-36, JavaAPI.scala:34-35,200); TypedActor deprecado desde Akka 2.6.0. Los únicos tests de EventBus Java y fábricas IO viven en el módulo docs
- **Alternativas:** (a) Excluir la superficie Java (la API Rust es una sola) manteniendo en alcance el comportamiento observable; (b) reproducir clases base con traits y builders; (c) portar solo las validaciones de Props
- **Recomendación:** (a) propuesta de exclusión de los nombres/clases Java; las filas siguen en alcance con impl=pending y su semántica queda cubierta por las filas Scala/Rust equivalentes (CELL, FSM, EVT, SCH, IO). Si el Owner no autoriza la exclusión se implementa (b) | Clasificación (cierre documental 2026-10-31): exclusión de alcance (superficie Java duplicada). | Consecuencias observables: sin fachada ningún programa Rust observa diferencia en el comportamiento de actores, FSM, buses, IO, scheduler; sólo desaparecen nombres Java (getX, Optional, CompletionStage). Qué bloquea: cierre de impl de las 13 filas JAPI; no bloquea ninguna fila CELL/FSM/EVT/IO/SCH. Qué avanza: todo el resto; si se acepta (a), las filas JAPI se cierran por equivalencia con evidencia de fachada y se conservan sólo los comportamientos diferenciales listados (ver Q-JAPI-2 y Q-JAPI-4).
- **Filas afectadas:** AC-JAPI-001; AC-JAPI-002; AC-JAPI-003; AC-JAPI-004.3; AC-JAPI-005.1; AC-JAPI-005.2; AC-JAPI-005.4; AC-JAPI-006; AC-JAPI-007; AC-JAPI-008; AC-JAPI-010; AC-JAPI-011; AC-JAPI-013
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-JAPI-2 — ReceiveBuilder/PFBuilder/Match para receive dinámico: igualdad de tipo exacto vs isInstance

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PFBuilder.java:43-148; Match.java:131-133; MatchBuilderTest.java:28-58 (Integer antes que Number); Q-X-3, Q-X-6; ReceiveBuilderTest.java:89-287 (A1-A11: orden de alta, isInstance con interfaz Msg, matchEquals por equals, matchUnchecked con ClassCastException); PFBuilder.build() resetea y ReceiveBuilder.build() no (medido con el jar 1.7.1)
- **Alternativas:** (a) Builder por TypeId exacto con match_any/match_equals y MatchError como Err; (b) emular isInstance con la relación de supertipos de Q-X-6; (c) no ofrecer builder (solo match de Rust)
- **Recomendación:** (a) como utilidad idiomática de los actores classic con Box<dyn Any> (Q-X-3), con (b) si Q-X-6 define supertipos; la diferencia con isInstance se documenta | Clasificación (cierre documental 2026-10-31): divergencia funcional (isInstance vs TypeId exacto). | Consecuencias observables: con TypeId exacto un caso por trait/interfaz no atrapa implementadores (ReceiveBuilderTest.java:108-121 no portable literalmente); con supertipos declarados (Q-X-6) sí. Qué bloquea: la fila AC-JAPI-012 y los builders de AC-JAPI-003/004.x; qué avanza: el resto.
- **Filas afectadas:** AC-JAPI-003; AC-JAPI-004.1; AC-JAPI-004.2; AC-JAPI-004.4; AC-JAPI-012
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-JAPI-3 — TypedActor clásico deprecado (proxies JDK dinámicos)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TypedActor.scala:43-524; TypedActorSpec.scala:290-588; Akka 2.6 deprecación (@deprecated); TypedActorFactory.typedActorOf aplica el dispatcher de TypedProps (medido con el jar 1.7.1: withDispatcher("my-disp") ejecuta el método en el hilo my-disp); DefaultReturnTimeout=5000 ms (pekko.actor.typed.timeout, reference.conf:147-152)
- **Alternativas:** (a) Excluir; (b) macro `#[typed_actor]` que genere un proxy desde un trait con las cuatro formas de retorno; (c) portar solo MethodCall y el actor interno sin proxy
- **Recomendación:** (a) exclusión propuesta (deprecado y sin equivalente dinámico); la fila sigue en alcance. Si el Owner prefiere paridad, (b) con la semántica de invoke de TypedActor.scala:466-499 (void unidireccional, Future sin bloqueo, Option con timeout -> none, valor con Await) | Clasificación (cierre documental 2026-10-31): exclusión de alcance (API deprecada). | Consecuencias observables: sin portar, ningún programa Rust puede llamar a un actor como si fuera un objeto con interfaz; la alternativa nativa es el enum de mensajes + ask. Qué bloquea: AC-JAPI-009.1-.3; nada más depende de TypedActor.
- **Filas afectadas:** AC-JAPI-009.1; AC-JAPI-009.2; AC-JAPI-009.3
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-JAPI-4 — Divergencias de la fachada Java si se porta: quirks no replicados

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AbstractActor.getReceiveTimeout() sin timeout lanza IllegalArgumentException("toNanos not allowed on infinite Durations") (medido con el jar 1.7.1; AbstractActor.scala:168-171); ReceiveBuilder.build() no resetea pese al Javadoc (ReceiveBuilder.java:57-69; medido); UdpConnectedMessage.connect(..., null, ...) produce Some(null) mientras TcpMessage.connect usa Option(null) (UdpConnected.scala:208 vs Tcp.scala:724); matchEquals(null) lanza NullPointerException al evaluar (medido); Duration de Java que no cabe en nanos de Long lanza ArithmeticException (medido)
- **Alternativas:** (a) No replicar: getReceiveTimeout devuelve Option, build consume el builder, local_address es Option, matchEquals valida al registrar; (b) replicar cada quirk con su excepción; (c) no portar la fachada (Q-JAPI-1 a) y descartar la pregunta
- **Recomendación:** (a) si se porta la fachada; (c) si se acepta Q-JAPI-1 (a). Consecuencias observables: sólo para programas que dependieran de la excepción o de la reutilización del builder; en ambos casos son defectos/artefactos de la fachada Java, no contrato de actores. Qué bloquea: la implementación de AC-JAPI-001 (getReceiveTimeout), AC-JAPI-012 (build) y AC-JAPI-010 (UdpConnectedMessage.connect) sólo si Q-JAPI-1 elige portar. Clasificación: divergencia funcional condicionada a Q-JAPI-1
- **Filas afectadas:** AC-JAPI-001; AC-JAPI-008; AC-JAPI-010; AC-JAPI-012
- **Unidades afectadas:** JAPI
- **Origen:** classic-io-ser

### Q-MBX-1 — Semántica de BoundedMailbox: enqueue bloqueante con push-timeout vs. no bloqueante

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:557-571 (BoundedQueueBasedMessageQueue: offer con timeout si pushTimeOut>=0, put si <0, DeadLetter si vence); :604-629 (deque acotado, igual con offerFirst/putFirst); :1000-1033 (BoundedControlAwareMailbox: awaitNanos, timeout <=0 descarta de inmediato); util/BoundedBlockingQueue.scala:99-113; reference.conf:644-647; Mailboxes.scala:257-266 (warning por pushTimeOut no cero; Pekko recomienda 0); MailboxConfigSpec.scala:56-92 (DeadLetter con capacidad llena; timeouts -1/0/100 ms); ActorWithBoundedStashSpec.scala:119-133 (offerFirst con 500 ms).
- **Alternativas:** (a) `tell` síncrono que bloquea el hilo emisor hasta pushTimeOut (paridad); (b) `tell` nunca bloquea: si está lleno, DeadLetter inmediato (equivale a push-timeout=0); (c) paridad en el modo 0 y `try_tell`/`tell_async` awaitable para timeout>0, con `tell` bloqueante sólo fuera de un worker async
- **Recomendación:** [Clasificación: divergencia funcional condicionada por Q-X-4] Implementar (c): `mailbox-push-timeout-time=0` y timeouts >0 en cola con envío awaitable; `tell` sincrónico con timeout>0 bloquea el hilo (paridad) sólo fuera de un worker async; diferencia observable documentada. Consecuencias observables: con la cola llena y timeout>0 el emisor ve una latencia de hasta pushTimeOut y luego un DeadLetter; con 0 el descarte es inmediato; con <0 espera sin límite (riesgo de interbloqueo, como en Pekko); BoundedControlAwareMailbox con timeout <=0 descarta de inmediato. Bloquea la implementación de: AC-MBX-007 y las variantes acotadas de AC-MBX-009/010/011/012 con timeout>0 y AC-MBX-016 (overflow del stash en deque acotado). Puede avanzar: todas las colas no acotadas, NonBlockingBoundedMailbox (AC-MBX-008) y las acotadas con timeout 0. Requiere decisión del Owner (Q-X-4).
- **Filas afectadas:** AC-MBX-007,AC-MBX-016
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-2 — Identidad de tipo de cola / semántica: reflexión de marker traits vs. descriptor explícito

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailboxes.scala:112-152 (Reflect.findMarker sobre RequiresMessageQueue[T] y ProducesMessageQueue[T]); Mailbox.scala:509-629,873-899; ActorMailboxSpec.scala:202-251 (listas de semánticas esperadas con isInstance) y :224-234,453-458 (un marcador DEFINIDO POR EL USUARIO, MCBoundedMessageQueueSemantics, mapeado vía pekko.actor.mailbox.requirements)
- **Alternativas:** (a) `enum/bitflags QueueSemantics` cerrado declarado por MailboxType y por el actor; (b) TypeId + subtyping manual; (a') conjunto ABIERTO de semánticas (nombre/TypeId registrado) con tabla de implicancia extensible
- **Recomendación:** [Clasificación: representación que conserva el contrato] (a'): un conjunto abierto de semánticas con implicancia codificada (UnboundedDequeBased implica Deque, Unbounded y MultipleConsumer; BoundedDequeBased implica Deque, Bounded y MultipleConsumer; ControlAware(Un)Bounded implican ControlAware, (Un)Bounded y MultipleConsumer; Logger implica Unbounded) y mapeo `requirements` por nombre de semántica. Un enum cerrado NO basta: el spec usa un marcador de usuario (MCBoundedMessageQueueSemantics, ActorMailboxSpec.scala:224-234). Consecuencia: desbloquea AC-MBX-013/014/015/018 sin afectar a las colas. No bloquea a otras filas.
- **Filas afectadas:** AC-MBX-013,AC-MBX-014,AC-MBX-015
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-3 — ControlMessage: detección de mensajes de control en mensajes `dyn Any`

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:877-880,957-960 (match por tipo dinámico `Envelope(_: ControlMessage, _)`); ControlAwareDispatcherSpec.scala:30,48-72 (`case object ImportantMessage extends ControlMessage`)
- **Alternativas:** (a) trait/método opcional en el envoltorio Envelope; (b) registro de TypeId de control; (c) predicado en el MailboxType
- **Recomendación:** [Clasificación: representación; depende de Q-X-3] (b) registro por TypeId a nivel sistema + hook (c) predicado en el MailboxType para colas custom. Consecuencia observable: sólo los mensajes registrados se tratan como control (orden: control primero, FIFO por clase). Bloquea la implementación de AC-MBX-012 (ambas variantes); el resto puede avanzar.
- **Filas afectadas:** AC-MBX-012
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-4 — Comparator de prioridad sobre Envelope: orden estable bajo empate para UnboundedPriorityMailbox

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:741-745,789-793 (PriorityBlockingQueue sin garantía entre iguales; StablePriorityBlockingQueue con número de secuencia); util/StablePriorityQueue.scala:14-87; PriorityDispatcherSpec.scala:58-98 (prioridades DISTINTAS: no ejerce empates); StablePriorityDispatcherSpec.scala:100-104 (afirma FIFO entre iguales)
- **Alternativas:** (a) usar siempre heap estable también en el no estable (más estricto que Pekko); (b) BinaryHeap sin secuencia
- **Recomendación:** [RESUELTA POR LECTURA 2026-10-31; Clasificación: representación que conserva el contrato] El spec del mailbox no estable nunca ejerce empates y la implementación no los especifica, de modo que Rust no debe afirmar ningún orden entre iguales (b: BinaryHeap sin secuencia); el estable sí lleva secuencia (FIFO afirmado). No requiere decisión del Owner salvo que se quiera endurecer el contrato (a, rechazado por diferir sólo hacia más estricto). No bloquea nada.
- **Filas afectadas:** AC-MBX-009,AC-MBX-010
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-5 — Aislamiento de tipos de mensajes de sistema vs. usuario en el mailbox (lista de sistema CAS + deadLetterMailbox)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:340-354,454-503 (cola de sistema = lista enlazada LIFO con CAS; `systemDrain` la invierte a EarliestFirst; centinela NoMessage tras cleanUp); Mailboxes.scala:54-72; Mailbox.scala:289-333 (sistema siempre antes que usuario)
- **Alternativas:** (a) cola de sistema separada Treiber-stack o Mutex<Vec> invertido al drenar; (b) canal único con prioridad
- **Recomendación:** [Clasificación: arquitectura] (a) conservar dos colas (sistema y usuario); la de sistema es un stack Treiber o Mutex<Vec> invertido al drenar; el sistema siempre se drena entero antes de cada mensaje de usuario y tras cada mensaje. Consecuencia observable: ningún mensaje de usuario se procesa con un mensaje de sistema pendiente. No bloquea; puede avanzar AC-MBX-001/002/003/021.
- **Filas afectadas:** AC-MBX-001,AC-MBX-002,AC-MBX-003
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-MBX-6 — Modelo de ejecución del mailbox (hilo + ForkJoinTask vs. tarea async): tareas por turno, fairness y `throughput`

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Mailbox.scala:230-280; Dispatcher.scala:42-160; DispatcherActorSpec.scala:84-141 (equidad: throughput=101 con 1 hilo; throughput=2 con deadline 100 ms); DispatcherActorsSpec.scala:44-69
- **Alternativas:** (a) tarea tokio por mailbox que procesa N mensajes y hace yield_now; (b) pool propio work-stealing; ver Q-X-4
- **Recomendación:** [Clasificación: arquitectura, subordinada a Q-X-4] Si Q-X-4 elige (A) pools de hilos propios: el turno de mailbox es un trabajo del pool, igual que en Pekko (paridad total de throughput/equidad). Si se elige un runtime async: (a) tarea por mailbox que procesa N mensajes y hace yield_now (el throughput pasa a ser el número máximo de mensajes por poll). Consecuencia observable: en ambos casos se conserva "máx max(throughput,1) mensajes por turno y corte por deadline". Bloquea la implementación de AC-MBX-002 y AC-DSP-002/003 hasta decidir Q-X-4; el diseño del runner con reloj inyectable puede avanzar.
- **Filas afectadas:** AC-MBX-002,AC-DSP-002,AC-DSP-003
- **Unidades afectadas:** MBX,DSP
- **Origen:** classic-dispatch

### Q-MBX-7 — PriorityGenerator.compare: resta de Int con desbordamiento

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatcher.scala:181-182 (`compare = gen(a.message) - gen(b.message)` con Int de complemento a dos); Mailbox.scala:732-822 (todas las colas de prioridad usan ese comparador); PriorityDispatcherSpec.scala:38-50 y StablePriorityDispatcherSpec.scala:37-51 (prioridades acotadas, no ejercen el desborde)
- **Alternativas:** (a) paridad: `gen(a).wrapping_sub(gen(b))`; (b) `i32::cmp` (corrige el desborde pero invierte el orden cuando |a-b| > 2^31-1); (c) restar en i64
- **Recomendación:** [Clasificación: divergencia funcional menor] (a) paridad con el desborde envolvente y documentarlo; un usuario que necesite prioridades extremas debe usar un comparador propio. Consecuencia observable: con prioridades como Int.MinValue frente a Int.MaxValue el orden es el de Pekko (incorrecto). Si el Owner prefiere (b) se anota Q: en la columna notes. Bloquea sólo el cierre de AC-MBX-009/010 en el borde; el resto puede avanzar.
- **Filas afectadas:** AC-MBX-009,AC-MBX-010
- **Unidades afectadas:** MBX
- **Origen:** classic-dispatch

### Q-PATH-7 — Address.parse / AddressFromURIString.apply con un octeto IPv4 embebido fuera de rango en un literal IPv6: NumberFormatException de Pekko vs MalformedURLException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Address.scala:186-188 captura sólo URISyntaxException; medido con el oráculo 2.0.0 (AD-NFE-01 `pekko://sys@[::1.2.3.99999999999]:1`, AD-NFE-02 `pekko://sys@[1:2:3:4:5:6:1.2.3.2147483648]:1`): `java.lang.NumberFormatException: Error at index 9 in: "99999999999"` / `"2147483648"`. `ActorPath::from_string` ya reproduce la NFE (casos nfe-uri-ipv6-*); `Address::parse` (address.rs) la convierte en MalformedUrlError con `uri::parse(s).ok()`. Rust hoy: MalformedURLException.
- **Alternativas:** (a) `Address::parse` devuelve un error propio que distingue NumberFormat de MalformedUrl (cambia la firma pública `Result<Address, MalformedUrlError>`); (b) mantener MalformedUrlError y documentar la diferencia; (c) añadir la variante sólo en un enum de error `#[non_exhaustive]` nuevo.
- **Recomendación:** (a): paridad con Pekko 2.0.0; es una diferencia observable y cambia la API pública, por lo que requiere decisión del Owner. Hasta entonces AD-NFE-01/02 están en `PENDING` del arnés y AC-PATH-004 queda `partial`.
- **Filas afectadas:** AC-PATH-004
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation

### Q-REF-1 — Representación de "sin sender" y valor de sender() fuera de un mensaje

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:416 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:44-52 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:596-600 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:157,736 ; actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:51 (Envelope.apply sustituye noSender por deadLetters) ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:281-285
- **Alternativas:** (a) tell(msg, Option<&ActorRef>) y sender() devuelve siempre un ActorRef (deadLetters por defecto) / (b) sender() devuelve Option<ActorRef> / (c) ActorRef con variante NoSender
- **Recomendación:** (a): iguala el observable de Pekko (el receptor siempre ve deadLetters); DeadLetterActorRef usa sí mismo como sender por defecto (ActorRef.scala:736) y debe replicarse [2026-10-31 cierre documental] Clasificación: representación que conserva el contrato. Resuelto leyendo: el receptor SIEMPRE ve un ActorRef (deadLetters si no hay emisor: Envelope.apply y ActorCell.sender); sólo DeadLetterActorRef usa `this` como sender implícito. Consecuencia observable: no existe sender ausente en el receptor. Bloquea: `impl` de AC-REF-002/005/024. Puede avanzar: el resto de REF y todo CELL (la elección sólo afecta a la firma de tell/sender).
- **Filas afectadas:** AC-REF-002 ; AC-REF-005 ; AC-REF-024
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-REF-2 — Props sin reflexión: formas classOf + args, IndirectActorProducer y InstantiationException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Props.scala:32-124 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:622-640,667-674 ; actor/src/main/scala/org/apache/pekko/actor/IndirectActorProducer.scala:51-77 ; actor/src/main/scala/org/apache/pekko/actor/AbstractProps.scala:33-41 ; actor/src/main/scala/org/apache/pekko/util/Reflect.scala:95-124 ; actor-tests/src/test/scala/org/apache/pekko/util/ReflectSpec.scala:38-62
- **Alternativas:** (a) sólo closures y constructores tipados (Props::new(|| A)) / (b) además un registro de fábricas por nombre (Q-X-2) para clases por nombre / (c) macros de derive
- **Recomendación:** (a) más (b) para configuración por nombre; las formas reflexivas puras (classOf con args null, mensaje de InstantiationException, validación de firma de constructor) se marcan como diferencia observable aceptada; la fila sigue en alcance hasta que el Owner decida [2026-10-31 cierre documental] Clasificación: divergencia funcional parcial (exclusión de reflexión). Consecuencias observables enumeradas al contrastar AC-REF-008: se pierden `Props(classOf[A], args…)` con resolución de constructor por tipos, los mensajes de IllegalArgumentException de `Reflect.findConstructor`/`Props.validate` ("no matching constructor found…", "multiple matching constructors found…", "must not be abstract", "must be subClass of …") y "unknown actor creator"; el error de configuración temprano (en `Props(...)`) pasa a ser error de compilación. Se conservan la fábrica por closure, una fábrica nueva por incarnation y `actorClass` para elegir mailbox. Bloquea: `impl` de AC-REF-008 y la parte reflexiva de AC-REF-007.1/.2; AC-CELL-002.2 sólo en el texto de InstantiationException. Puede avanzar: Props con closure + trait ActorFactory, AC-CELL-002.1, AC-CELL-026.
- **Filas afectadas:** AC-REF-007.1 ; AC-REF-007.2 ; AC-REF-008 ; AC-CELL-002.2
- **Unidades afectadas:** REF ; CELL
- **Origen:** classic-cell

### Q-REF-3 — Serialización Java de ActorRef (writeReplace / readResolve / SerializedActorRef)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:471-496 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:288-365
- **Alternativas:** (a) conservar sólo el contrato path→ref con serialized_actor_path y resolve_actor_ref, sin formato Java / (b) implementar un serializador propio equivalente a SerializedActorRef (string de path) / (c) no soportar
- **Recomendación:** (a)+(b): conservar el mensaje IllegalStateException "Trying to deserialize a serialized ActorRef without an ActorSystem in scope…" como error del resolver cuando no hay sistema en el contexto de deserialización [2026-10-31 cierre documental] Clasificación: exclusión de alcance (serialización Java) + representación (contrato path→ref). Consecuencia: no hay writeReplace/readResolve ni SerializedActorRef; se conserva el error de resolución sin sistema en contexto. Bloquea: `impl` de AC-REF-023 (A1–A3 se portan a serialized_actor_path/resolve_actor_ref). Puede avanzar: todo lo demás; la serialización de ActorRef por la unidad SER es independiente.
- **Filas afectadas:** AC-REF-023
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-REF-4 — Restricción "los actores sólo se crean vía actorOf" (ActorInitializationException al usar new)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/Actor.scala:495-506 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:148-175
- **Alternativas:** (a) estructural: el contexto es un parámetro y el error es imposible; el caso se marca no aplicable / (b) emular con un guard de runtime
- **Recomendación:** (a) con justificación escrita; el test upstream no se porta [2026-10-31 cierre documental] Clasificación: representación (la restricción pasa a ser estructural). Resuelto leyendo: el único mecanismo es el thread-local `contextStack` (ActorCell.scala:622-640); en Rust el contexto es parámetro, por lo que el error es inalcanzable. Consecuencia: AC-REF-006 se marca no aplicable (requiere confirmación del Owner para cerrarla como no aplicable en `validation`). Bloquea: sólo el cierre de AC-REF-006. Puede avanzar: todo.
- **Filas afectadas:** AC-REF-006
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-REF-5 — Creación asíncrona de top-level (system.actorOf vía RepointableActorRef/UnstartedCell) frente a síncrona (context.actorOf)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:659-719 ; actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:95-135,204-326 ; actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:58-64 ; actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:112-135,225-251,268-304 ; actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:300-326 ; actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:206-216 ; actor-tests/src/test/scala/org/apache/pekko/dispatch/PriorityDispatcherSpec.scala:72-75
- **Alternativas:** (a) crear siempre la celda de forma síncrona preservando observables / (b) reproducir UnstartedCell con cola previa y lock con unstarted-push-timeout / (c) cola previa sin lock
- **Recomendación:** (a) si se prueba que los observables coinciden: mensajes enviados justo tras actorOf se entregan en orden, los mensajes de sistema encolados antes se entregan primero, y Supervise llega al guardián antes de la celda real; si no se puede, (b). Impacta pekko.actor.unstarted-push-timeout (reference.conf:145) [2026-10-31 cierre documental] Clasificación: arquitectura. Incertidumbre resuelta leyendo y enumerando observables (AC-REF-020): una creación síncrona CONSERVA orden por emisor, sistema antes que usuario, `Create` primero y `Supervise` antes de procesar mensajes; PIERDE (1) `pekko.actor.unstarted-push-timeout` (la clave quedaría aceptada y sin efecto), (2) el descarte con Warning por lock/cola llena de la `UnstartedCell`, (3) la ventana en que un top-level con mailbox de prioridad procesa en orden de llegada (con creación síncrona SÍ ordena por prioridad desde el primer mensaje), (4) `isStarted`/`UnstartedCell` observables. Recomendación: (a) aceptando explícitamente (1)–(4) como diferencias, o (b) si el Owner exige paridad completa. Bloquea: `impl` de AC-REF-020, AC-CELL-016 y AC-CELL-027. Puede avanzar: AC-REF-003, AC-CELL-002.x, 007.x, 008.x, SUP y DW (no dependen de esta elección).
- **Filas afectadas:** AC-REF-020 ; AC-CELL-016 ; AC-CELL-027
- **Unidades afectadas:** REF ; CELL
- **Origen:** classic-cell

### Q-REF-6 — Visibilidad de accesores internos usados por tests y testkit (ActorRef.isTerminated, underlying, children)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:173,309,389 ; actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:210 ; actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:249-367 ; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:316-322
- **Alternativas:** (a) pub(crate) más un módulo testkit con reexport / (b) pub con doc hidden / (c) no exponer
- **Recomendación:** (a): los specs portados necesitan isTerminated y el estado de mailbox [2026-10-31 cierre documental] Clasificación: arquitectura (visibilidad). Se necesitan accesores internos para portar los specs (isTerminated, underlying.mailbox.isSuspended, children, props). Consecuencia: tests de integración dependen de un módulo `testkit` con reexport. Bloquea: la validación (no la `impl`) de AC-REF-001.x, AC-REF-019 y AC-CELL-031. Puede avanzar: toda la implementación.
- **Filas afectadas:** AC-REF-001.1 ; AC-REF-001.2 ; AC-REF-019 ; AC-CELL-031
- **Unidades afectadas:** REF
- **Origen:** classic-cell

### Q-RT-1 — Routing en el hilo del emisor y Router inmutable: mensajes dinamicos y estado atomico

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** RoutedActorCell.scala:147-152; Router.scala:110-185 | Cierre 2026-10-31: Clasificacion: arquitectura (ruteo en el hilo del emisor) con representacion que conserva el contrato. Resuelto por lectura: RoutedActorCell.scala:147-152 (ruteo en el emisor, mailbox solo para mensajes de gestion) y RoundRobinSpec:163-210.
- **Alternativas:** (a) Router en ArcSwap con ruteo en el tell del emisor (paridad); (b) actor-router con cola propia (mas simple, cambia el hilo y el orden)
- **Recomendación:** (a) paridad observable; mensajes Box<dyn Any> (Q-X-3). Consecuencias: orden entre emisores y atomicidad del Router (copy-on-write). Bloquea AC-RT-001, 002, 005, 006, 007; la logica pura (RoundRobin/Random/Broadcast select) puede avanzar.
- **Filas afectadas:** AC-RT-001;AC-RT-002;AC-RT-005;AC-RT-006;AC-RT-007
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-2 — Mensajes de gestion del router y ciclo de vida del router actor

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** RoutedActorCell.scala:65-114,159-218 | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (nombres de mensajes de gestion). Resuelto por lectura: RoutedActorCell.scala:159-218 (detalle en AC-RT-003.2 A5/A6 y 003.3).
- **Alternativas:** (a) replicar GetRoutees/AddRoutee/RemoveRoutee/AdjustPoolSize y Terminated->evict; (b) API tipada separada
- **Recomendación:** (a), con los mismos nombres de mensaje. Consecuencias: AdjustPoolSize no detiene un pool vacio; GetRoutees/Terminated. Bloquea AC-RT-003.1-.3.
- **Filas afectadas:** AC-RT-003.1;AC-RT-003.2;AC-RT-003.3
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-3 — Router config via HOCON: type-mapping, FQCN custom, fromConfig por router

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** reference.conf:155-170; Deployer.scala:233-290; Resizer.scala:69-80 | Cierre 2026-10-31: Clasificacion: arquitectura (registro de fabricas por nombre, Q-X-2) + HOCON (Q-X-1). Resuelto por lectura: Deployer.scala:233-290 y reference.conf:155-171.
- **Alternativas:** (a) tabla de fabricas por nombre (Q-X-2) leyendo las mismas claves; (b) solo API programatica
- **Recomendación:** (a) misma sintaxis HOCON (Q-X-1) y mismas claves. Consecuencias: configuracion de routers y resizers por HOCON. Bloquea AC-RT-004, 013.3, 016 y la creacion de AC-RT-014 desde configuracion; los constructores programaticos pueden avanzar.
- **Filas afectadas:** AC-RT-004;AC-RT-013.3;AC-RT-016
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-5 — ConsistentHashing: serializacion de claves arbitrarias y MurmurHash exacto

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ConsistentHashing.scala:175-260 (AnyRef -> SerializationExtension); MurmurHash.scala:127-141 | Cierre 2026-10-31: Clasificacion: divergencia funcional acotada (claves no String/bytes) + representacion exacta para String/bytes. Resuelto por lectura: ConsistentHashing.scala:226-247 y MurmurHash.scala:127-141; ConsistentHashingRouterSpec:81-96 usa claves Int y case class (serializadas con Java).
- **Alternativas:** (a) claves String/bytes con hash identico; resto via trait `ConsistentHashKey` propio (difiere de Pekko); (b) replicar serializacion Java (imposible)
- **Recomendación:** (a); documentar diferencia observable para claves no String/bytes. Consecuencias: solo el reparto de claves que no son String/Array[Byte]. Bloquea la paridad exacta de AC-RT-009.1 A2; AC-RT-009.2/.3 (puras) pueden avanzar y validarse contra el oraculo.
- **Filas afectadas:** AC-RT-009.1;AC-RT-009.2;AC-RT-009.3
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-6 — Discrepancias doc/codigo en ScatterGather y TailChopping (excepcion y texto)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ScatterGatherFirstCompleted.scala:41-42 (doc: AskTimeoutException) vs :62 (TimeoutException "Timeout due to no routees"); TailChopping.scala:105 ("[$within.toMillis} ms]") | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato (tipo de excepcion) / divergencia funcional solo para el texto con defecto de formato de TailChopping. Resuelto por lectura: ScatterGatherFirstCompleted.scala:62 usa TimeoutException (spec lo afirma, ScatterGatherFirstCompletedSpec:108-114); TailChopping.scala:104-105 interpola `$within.toMillis}` (el literal "[<within>.toMillis} ms]", p. ej. "[300 milliseconds.toMillis} ms]").
- **Alternativas:** (a) replicar el codigo exactamente (tipo y texto, incluido el defecto de formato); (b) corregir al comportamiento documentado
- **Recomendación:** (a) replicar el codigo (spec afirma TimeoutException en ScatterGatherFirstCompletedSpec:113); el texto de TailChopping como diferencia a decidir por el Owner. Consecuencias: texto del AskTimeoutException de TailChopping (no afirmado por TailChoppingSpec:105-121). Bloquea solo AC-RT-011 A5; el resto de RT-010/011 puede avanzar.
- **Filas afectadas:** AC-RT-010;AC-RT-011
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-7 — BalancingPool: dispatcher de mailbox compartido (BalancingDispatcher) y specs pending upstream

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Balancing.scala:116-144; BalancingSpec.scala:133-165 (pending) | Cierre 2026-10-31: Clasificacion: arquitectura (dispatcher de cola compartida, AC-DSP-012). Resuelto por lectura: BalancingSpec:133-165 pending upstream (akka/akka#30860); solo los tests de nombres (:167-178) estan vigentes.
- **Alternativas:** (a) cola multiconsumidor compartida; (b) work-stealing entre mailboxes
- **Recomendación:** (a); depende de AC-DSP-012; los tests de reparto upstream estan pending: validar con aserciones propias. Consecuencias: reparto de trabajo en BalancingPool. Bloquea AC-RT-012; sin AC-DSP-012 no hay forma de validar el reparto, si de los nombres de dispatcher.
- **Filas afectadas:** AC-RT-012
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-8 — Listeners requiere ActorRef ordenable (TreeSet) y semantica de duplicados

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Listeners.scala:37-47 | Cierre 2026-10-31: Clasificacion: representacion que conserva el contrato. Resuelto por lectura: Listeners.scala:37-58 (TreeSet[ActorRef], idempotente).
- **Alternativas:** (a) BTreeSet<ActorRef> con Ord por path+uid; (b) Vec con orden de insercion
- **Recomendación:** (a) paridad con TreeSet (orden e idempotencia). Consecuencias: orden de entrega de gossip. Bloquea AC-RT-015 y AC-FSM-002 A10 solo en el orden.
- **Filas afectadas:** AC-RT-015;AC-FSM-002
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-RT-9 — Resizer optimo: reloj y aleatoriedad inyectables, desempate en optimize y defecto de upper-bound entre constructor y reference.conf

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** OptimalSizeExploringResizer.scala:148-166 (estado mutable, ThreadLocalRandom, LocalDateTime/System.nanoTime), :299 (`minBy` sobre un Map inmutable: en empate de duraciones el resultado depende del orden de iteracion de Scala), :129-139 (upperBound = 30) frente a reference.conf:304 (upper-bound = 10). MetricsBasedResizerSpec:348-364 no incluye empates.
- **Alternativas:** (a) Clock y Rng inyectables, BTreeMap y desempate por menor tamano, documentado como diferencia no observable en specs; (b) replicar el orden de iteracion de Scala (imposible de forma estable para Map de mas de 4 entradas)
- **Recomendación:** (a). Consecuencias: la secuencia aleatoria y el desempate no son reproducibles contra Pekko; los valores por defecto del constructor (30) y de la configuracion (10) se mantienen distintos como en Pekko. Bloquea solo la validacion diferencial de empates (AC-RT-014 A9); no bloquea nada mas.
- **Filas afectadas:** AC-RT-014
- **Unidades afectadas:** RT
- **Origen:** classic-patterns

### Q-SCH-3 — Mensajes de timer: tipo de mensaje interno y entrega al actor

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TimerSchedulerImpl.scala:28-164; Timers.scala:52-70; TimerSpec.scala:189-317 (se ejecuta con Timers y con FSM); ReceiveTimeoutSpec.scala:191-212
- **Alternativas:** (a) Envelope especial de timer con generación interceptado antes de receive; (b) mensaje normal
- **Recomendación:** [Clasificación: representación que conserva el contrato; depende de Q-X-3] (a) sobre interno de timer (key, generación, propietario, influencia en receive-timeout) interceptado antes de receive y del stash; auto-recibidos por la vía de autoReceive. Bloquea AC-SCH-011/012/013 hasta Q-X-3; no bloquea el resto del grupo.
- **Filas afectadas:** AC-SCH-011,AC-SCH-012,AC-SCH-013
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch

### Q-SER-2 — Java serialization (JavaSerializer, DisabledJavaSerializer, java.io.Serializable, JavaSerializer.currentSystem, serialización Java de ActorRef)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Serializer.scala:306-418; reference.conf:761-778,799-831; SerializeSpec.scala:372-374,402-511; SerializationSetupSpec.scala:170-212; DisabledJavaSerializerWarningSpec.scala:42-68; ActorRef.scala:462-496
- **Alternativas:** (a) Excluir toda la serialización Java incluyendo el id 1 y los avisos; (b) mantener solo DisabledJavaSerializer (id 1 reservado, warnings Security y excepciones con texto exacto) y la lógica de avisos, sin JavaSerializer habilitado; (c) emular JavaSerializer con un formato propio
- **Recomendación:** (b): allow-java-serialization siempre efectivo en off; "enabled" propuesto como exclusión (sin equivalente razonable en Rust); la fila AC-SER-013 sigue en alcance con impl=pending hasta que el Owner decida. La tabla por defecto conserva el binding "Serializable -> java" como entrada vacía documentada | Clasificación (cierre documental 2026-10-31): exclusión de alcance (Java serialization) con divergencia funcional declarada.
- **Filas afectadas:** AC-SER-003.5; AC-SER-004.1; AC-SER-004.2; AC-SER-013
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-3 — Información de transporte (thread-local) y serialización de ActorRef con address externa

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Serialization.scala:46-141,170-177,282-295; ActorRefProvider.scala:764-780; ActorRef.scala:471-486 (el límite de remote queda fuera del módulo)
- **Alternativas:** (a) task_local/thread_local con TransportInformation; (b) parámetro explícito SerializationContext en Serializer::to_binary/from_binary; (c) solo address por defecto del sistema local (sin Information)
- **Recomendación:** (b) para serializadores de usuario nuevos más (a) interno para compatibilidad con serializadores que consultan la información; sin remote solo hace falta el caso local. Documentar que el valor se pierde al cambiar de hilo | Clasificación (cierre documental 2026-10-31): representación que conserva el contrato.
- **Filas afectadas:** AC-SER-001.3; AC-SER-007.1; AC-SER-007.2
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-4 — Verificación serialize-messages/serialize-creators con mensajes dinámicos y Props como closures

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Dispatch.scala:171-223; Children.scala:280-309; SerializeSpec.scala:308-362; reference.conf:129-140
- **Alternativas:** (a) verificar solo tipos registrados en el registry y fallar el resto como Pekko falla lo no enlazado; (b) verificación opt-in por tipo con un marker NoSerializationVerificationNeeded invertido; (c) no portar serialize-creators (Props de Rust son closures sin args reflectivos) pero sí serialize-messages
- **Recomendación:** (a) para serialize-messages y (c) para serialize-creators, con `Props::with_serializable_args` opcional si se quiere paridad; la clave se acepta siempre. Los tests que dependen de serialize-messages=on (varios specs de Pekko) funcionarán con mensajes registrados | Clasificación (cierre documental 2026-10-31): divergencia funcional parcial (serialize-creators) y representación (serialize-messages).
- **Filas afectadas:** AC-SER-008.1; AC-SER-008.2
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-5 — Serializadores asíncronos y camino síncrono bloqueante con warning

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AsyncSerializer.scala:52-71 (Await.result Duration.Inf + warning); AsyncSerializeSpec.scala:111-137; Q-X-4
- **Alternativas:** (a) Futures como async fn y camino síncrono con block_in_place/hilo dedicado + warning idéntico; (b) camino síncrono devuelve Err("Async serializer called synchronously") en vez de bloquear; (c) sin camino síncrono
- **Recomendación:** (a): preserva el contrato observable (warning de 1 línea y resultado) y documenta que bloquear dentro del runtime requiere runtime multihilo; CompletionStage no se porta | Clasificación (cierre documental 2026-10-31): representación que conserva el contrato.
- **Filas afectadas:** AC-SER-006
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SER-6 — Alcance de SER en esta fase: serialización local, claves solo-remote y detección de ciclos de extensión

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** inventario AC-SER-001 nota "Q: alcance"; reference.conf:852-880 (max-decompressed-size y protobuf allowed-classes los consumen remote/cluster); ActorSystem.scala:1167-1192 (espera creation-timeout); grep sin consumidores de allowed-classes en pekko-actor
- **Alternativas:** (a) Portar toda la serialización local ahora (la necesitan serialize-messages y Persistence/Remote luego); (b) diferir SER hasta que exista un consumidor; (c) portar solo el SPI y los primitivos
- **Recomendación:** (a) ordenada tras EXT; las claves protobuf se aceptan sin efecto; detectar ciclo de inicialización de extensión de inmediato (error con el mismo texto) en vez de esperar creation-timeout, anotando la diferencia de latencia | Clasificación (cierre documental 2026-10-31): exclusión de alcance temporal (secuencia de entrega).
- **Filas afectadas:** AC-SER-010; AC-SER-012; AC-SER-014
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser

### Q-SUP-1 — Modelo de errores para deciders: jerarquía Exception/Error/Throwable y tipos de fallos del ciclo de vida

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:210-223,246-301 ; actor/src/main/scala/org/apache/pekko/actor/Actor.scala:166-295 ; actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:372 (decider.applyOrElse(cause, escalateDefault)) ; actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:216-245
- **Alternativas:** (a) enum Failure { Init, Killed, DeathPact, PreRestart, PostRestart, User(Box<dyn Error>), Panic } con downcast para makeDecider / (b) Box<dyn Error> con downcast_ref / (c) códigos
- **Recomendación:** (a); Q-X-5 decide si Panic se mapea a Escalate (equivalente de Error/Throwable) o a Restart; el decider por defecto debe reproducir Stop para Init/Killed/DeathPact, Restart para User y Escalate para Panic si se acepta [2026-10-31 cierre documental] Clasificación: arquitectura y divergencia funcional (Error/Throwable vs panic). Es la decisión con más filas dependientes: define el tipo de causa que ven todos los deciders, `Failed`, logFailure y las excepciones del ciclo de vida. Consecuencia observable: en Pekko un Throwable que no es Exception ESCALA con el decider por defecto; si en Rust un `panic` se mapea a Escalate hay que garantizar que el supervisor raíz lo detenga; si se mapea a Restart se pierde esa propiedad. Bloquea: `impl` de AC-SUP-004/012/017 y, indirectamente, AC-SUP-001/003/008/013/018/021 y AC-CELL-002.2/007.4. Puede avanzar: la planificación de ciclo de vida sin deciders, AC-SUP-006.x (aritmética) y DW. Decidir junto con Q-X-5.
- **Filas afectadas:** AC-SUP-004 ; AC-SUP-012 ; AC-SUP-017 ; AC-SUP-021
- **Unidades afectadas:** SUP
- **Origen:** classic-cell

### Q-SUP-2 — Aritmética de la ventana de reinicios (withinTimeRange.toMillis.toInt y System.nanoTime)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:58-90,303-309,550 ; actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala:58-88,549-550,660-662 ; actor-tests/src/test/scala/org/apache/pekko/actor/RestartStrategySpec.scala:43-277 ; actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:572-612
- **Alternativas:** (a) usar u64 de milisegundos con Instant monótono, saturando en lugar de truncar a Int / (b) replicar el truncamiento a i32 / (c) Duration nativa
- **Recomendación:** (a): diferencia observable sólo para ventanas mayores a ~24.8 días (Int overflow); la ventana empieza en la primera solicitud, no se desliza, y la comparación es "<= window" [2026-10-31 cierre documental] Clasificación: representación con una divergencia funcional menor. Incertidumbre resuelta midiendo/leyendo: `withinTimeRange.toMillis.toInt` es un cast que ENVUELVE (no satura): 25 días = 2_160_000_000 ms → Int −2_134_967_296; con ventana negativa `(now − windowStart) <= toNanos(window)` es falso, así que la ventana nunca contiene el fallo (cuenta = 1 y reinicio siempre permitido: equivale a "sin límite"). Para ventanas ≤ 24,8 días el comportamiento es el normal. Opciones: (a) saturar a u64 (diferencia solo para > 24,8 días: la ventana funcionaría como ventana real), (b) replicar el cast i32 (paridad exacta, incluido el comportamiento anómalo). Bloquea: `impl` de AC-SUP-006.2 (borde de ventana enorme). Puede avanzar: AC-SUP-006.1 y todo lo demás (la aritmética normal es idéntica).
- **Filas afectadas:** AC-SUP-006.1 ; AC-SUP-006.2 ; AC-SUP-021
- **Unidades afectadas:** SUP
- **Origen:** classic-cell

### Q-SYS-1 — ¿Drop del último handle de ActorSystem termina el sistema? (close/AutoCloseable)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Pekko: sólo terminate()/close() explícitos o Using (ActorSystem.scala:1106-1110; ActorSystemScala213PlusSpec.scala:29-35); Drop no existe en JVM. Cierre 2026-10-31: close() hace terminate() + Await.result(whenTerminated, close-actor-system-timeout) y lanza TimeoutException si vence (ActorSystem.scala:1106-1110, Scaladoc 683-701); en AC-SYS-004 A4 terminate() no depende de ningún Drop.
- **Alternativas:** (a) sin Drop, `close_blocking`/`terminate` explícitos; (b) Drop termina de forma no bloqueante; (c) guarda opcional `SystemGuard`
- **Recomendación:** [Clasificación: arquitectura (ciclo de vida del handle)] (a) más guarda opcional (c) para tests; ninguna semántica implícita de Drop. Consecuencias observables: un test que olvide terminar el sistema deja tareas vivas (igual que en Pekko). Bloquea: sólo AC-SYS-014 (política de Drop); puede avanzar todo lo demás con close()/terminate() explícitos.
- **Filas afectadas:** AC-SYS-014
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-2 — Mapeo de error fatal JVM (OOM/StackOverflow/Error) a Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:858-906: handler distingue NonFatal de fatal; jvm-exit-on-fatal-error=on (reference.conf:90) hace System.exit(-1). En Rust abort/OOM no es capturable y un panic en tarea de actor es un fallo normal (Q-X-5). Cierre 2026-10-31: el handler distingue NonFatal|InterruptedException|NotImplementedError|ControlThrowable (sólo loguea) de lo demás (stderr + log con marker SECURITY + System.exit(-1) o terminate()) (ActorSystem.scala:858-907).
- **Alternativas:** (a) fatal = panic fuera de actores + señal; (b) fatal = sólo `std::process::abort` (nada que manejar); (c) política configurable `FatalErrorPolicy`
- **Recomendación:** [Clasificación: arquitectura + divergencia funcional (OOM/stack overflow no capturables)] (c) con default ExitProcess para panics en hilos propios del sistema; documentar que OOM/stack overflow no son capturables. Consecuencias observables: un pánico en un hilo propio del sistema termina el proceso (o el sistema) en vez de la JVM; los errores no capturables (abort por OOM) no pasan por el handler. Bloquea: AC-SYS-015; el resto avanza.
- **Filas afectadas:** AC-SYS-015
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-3 — Filas JVM-específicas: ManifestInfo (mixed versions) y Main

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ManifestInfo.scala + ActorSystem.scala:1069; Main.scala:32-60 (deprecada). No hay MANIFEST ni classpath en Rust. Cierre 2026-10-31 (leídos ManifestInfo.scala y ManifestInfoSpec): la parte pura checkSameVersion(product, deps, versions) es portable y está afirmada por spec (ManifestInfoSpec.scala:20-44); sólo la lectura de META-INF/MANIFEST.MF (ManifestInfo.scala:116-155) es JVM. Main.scala:27-62 es trivial y está deprecado desde Akka 2.6.0.
- **Alternativas:** (a) implementar checkSameVersion puro + registro de versiones de crates en compilación; Main tras feature `deprecated`; (b) excluir la lectura de MANIFEST (única parte sin equivalente) y mantener el resto; (c) excluir ambas filas (requiere autorización explícita)
- **Recomendación:** [Clasificación: exclusión de alcance parcial (lectura de MANIFEST) + representación que conserva el contrato] Recomendación (b)+(a): conservar la función pura y el mensaje exactos, excluir sólo la lectura de manifests. Consecuencias observables: en un workspace de versión única el chequeo nunca falla salvo con mapa inyectado. Bloquea: nada (AC-SYS-020/021 pueden implementarse con (a)); si el Owner exige paridad de manifests, queda partial.
- **Filas afectadas:** AC-SYS-020 ; AC-SYS-021
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-4 — Protocolo RegisterTerminationHook sin remote/cluster

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Sólo lo usa RemoteActorRefProvider (fuera de alcance) y no hay test en pekko-actor (grep RegisterTerminationHook). ActorRefProvider.scala:295-374. Cierre 2026-10-31: grep sobre todo upstream: RegisterTerminationHook sólo aparece en ActorRefProvider.scala y remote/RemoteActorRefProvider.scala; el protocolo es private[pekko] (ActorRefProvider.scala:295-308) y SystemGuardian lo implementa en 331-374 (espera a que todos los hooks respondan TerminationHookDone antes de detener /system y parar los loggers).
- **Alternativas:** (a) implementar el protocolo completo con tests propios derivados del código; (b) dejar el hook sin consumidores y sin test; (c) marcar como excluido (requiere autorización)
- **Recomendación:** [Clasificación: exclusión de alcance vs. representación; decide el Owner] (a): es parte del comportamiento de /system y lo necesitarán extensiones; sin autorización de exclusión. Consecuencias observables: sin hooks, /system se detiene en cuanto termina /user (igual que Pekko sin consumidores). Bloquea: sólo AC-SYS-003.2 (sin spec) y la rama de espera de AC-SYS-002; el resto avanza.
- **Filas afectadas:** AC-SYS-003.2 ; AC-SYS-002
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-5 — Redacción de datos sensibles en Settings.toString

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:488-491; Helpers.scala:180-192: sin valores de variables de entorno y reemplazo de `user.name` por `<username>`; ActorSystemSpec.scala:395-411 prueba con `${user.home}` (propiedad Java). Cierre 2026-10-31: renderWithRedactions = config.root.render(setShowEnvVariableValues(false)).replace(user.name, "<username>") (Helpers.scala:188-192); el spec (ActorSystemSpec.scala:395-411) afirma que ni el nombre de usuario ni user.home aparecen y que sí aparece <username>; la presencia de user.name en el texto proviene de las propiedades del sistema que ConfigFactory.defaultReference mezcla en la config.
- **Alternativas:** (a) igual: no mostrar valores de entorno y redactar el usuario del proceso; (b) redactar además el home
- **Recomendación:** [Clasificación: representación que conserva el contrato; incertidumbre resuelta por lectura] (a); el spec porteado usará `${HOME}`/`USER` como equivalente de las propiedades Java. Consecuencias: la redacción depende de si la config de rakka incluye variables de entorno/usuario (Q-X-1). Bloquea: nada; AC-SYS-022 y AC-SYS-012 avanzan con (a).
- **Filas afectadas:** AC-SYS-022
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-SYS-6 — Superficie JVM de ActorSystem: sobrecargas con ClassLoader, DynamicAccess y ThreadFactory

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorSystem.scala:193-328 (sobrecargas con ClassLoader), 909-918 (threadFactory, dynamicAccess), 849-856; ProviderSelection y proveedores se instancian por FQCN con DynamicAccess (ActorSystem.scala:967-975). Ningún spec depende de la firma con ClassLoader (ActorSystemDispatcherSpec.scala:57-85 usa name/config/defaultExecutionContext).
- **Alternativas:** (a) sin sobrecargas con ClassLoader ni accesores dynamicAccess/threadFactory; registro de fábricas (Q-X-2) y runtime (Q-X-4); (b) tipo opaco `ClassLoaderHandle` sin efecto; (c) exclusión explícita de estas APIs
- **Recomendación:** [Clasificación: exclusión de alcance parcial + arquitectura] (a). Consecuencias observables: las firmas con ClassLoader no existen; el comportamiento "carga por nombre" pasa a Q-X-2. Bloquea: nada; AC-SYS-001.2/013/018 avanzan con (a).
- **Filas afectadas:** AC-SYS-001.2 ; AC-SYS-013 ; AC-SYS-018
- **Unidades afectadas:** SYS
- **Origen:** classic-system

### Q-T-ADP-2 — Que reglas de ActorAdapter sobreviven bajo runtime tipado nativo (Q-X-8 b)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:87-362
- **Alternativas:** (a) mantener las 4 familias (despacho, resultado, hooks, supervision de hijos); (b) re-especificarlas en el modelo nativo y validar solo contra goldens
- **Recomendación:** [Arquitectura; depende de Q-X-8] (a) las 4 familias como lista de reglas verificables (despacho AT-ADP-006.1, resultado 006.2, hooks 006.3, supervision de hijos 006.4); sus aserciones A# ya no dependen del adaptador clasico. Consecuencias observables: ninguna distinta si las reglas se conservan en el interprete. Bloquea: nada si Q-X-8 elige (b) (las reglas pasan al interprete nativo); con Q-X-8 (a) se implementan en la capa de compatibilidad. Puede avanzar: AT-ADP-006.* como especificacion. [Recomendacion previa: (a) como lista de reglas verificables]
- **Filas afectadas:** AT-ADP-006.1;AT-ADP-006.2;AT-ADP-006.3;AT-ADP-006.4;AT-ADP-010
- **Unidades afectadas:** ADP
- **Origen:** typed-extended

### Q-T-ADP-3 — Semantica de rethrowTypedFailure y supervision Stop por defecto del tipado spawneado desde clasico

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:168-172,236-261; actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:57-77 | Cierre documental 2026-10-31: ClassicSupervisingTypedSpec:74-143 afirma Stop por defecto y Restart con supervise(restart); la rama rethrowTypedFailure no tiene spec directo (ActorAdapter.scala:168-172).
- **Alternativas:** (a) conservar ambos (Stop por defecto, FailedBehavior detiene en silencio); (b) unificar con el default restart del clasico
- **Recomendación:** [Divergencia funcional evitada: se conserva] (a). Consecuencias observables: un tipado spawneado desde clasico se detiene (no reinicia) ante un fallo no supervisado; FailedBehavior detiene en silencio salvo rethrow. Bloquea: AT-ADP-002/006.2/006.4 solo si Q-X-8 elige (a). Puede avanzar: el resto. [Recomendacion previa: (a): afirmado por ClassicSupervisingTypedSpec]
- **Filas afectadas:** AT-ADP-002;AT-ADP-006.2;AT-ADP-006.4
- **Unidades afectadas:** ADP
- **Origen:** typed-extended

### Q-T-ASK-1 — Taxonomia de errores de ask (TimeoutException vs error del destino) y conservacion del texto

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:148-157; actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:729-735; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:80-81,105-106 | Cierre documental 2026-10-31: AskSpec:71-82 y 96-107 afirman los prefijos "had already been terminated." y "Ask timed out on"; el texto largo y el IllegalArgumentException de timeout<=0 no los afirma ningun spec (AskPattern.scala:152-157; AskSupport.scala:729-735).
- **Alternativas:** (a) enum AskError con variantes y mensajes textuales identicos a Pekko; (b) un unico Timeout(String); (c) std::io::Error
- **Recomendación:** [Representacion que conserva el contrato] (a). Consecuencias observables: el texto del error de timeout y de destino terminado es identico (prefijos afirmados por spec; el resto, por oraculo). Bloquea: cierre de impl de AT-ASK-001 (mensajes). Puede avanzar: tipo AskError, ask vivo y respuesta tardia como DeadLetter. [Recomendacion previa: (a): conserva los prefijos "Ask timed out on" y "had already been terminated." que los tests afirman]
- **Filas afectadas:** AT-ASK-001;AT-ASK-002
- **Unidades afectadas:** ASK
- **Origen:** typed-extended

### Q-T-ASK-2 — Conservar o no la restriccion de ask a InternalRecipientRef

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AskPattern.scala:120-127 | Cierre documental 2026-10-31: AskPattern.scala:117-127 es una restriccion transitoria declarada ("until native system is implemented"); ningun test la afirma (AT-ASK-004 A1).
- **Alternativas:** (a) permitir ask sobre cualquier RecipientRef; (b) replicar IllegalStateException con el mismo texto
- **Recomendación:** [Divergencia funcional minima, resuelta por lectura: la restriccion es accidental] (a) permitir ask sobre cualquier RecipientRef. Consecuencias observables: en Pekko un RecipientRef propio lanza IllegalStateException sincronico; en Rust no existiria ese error. Bloquea: nada (la fila AT-ASK-004 es internal-accidental). Puede avanzar: toda AT-ASK-*. [Recomendacion previa: (a): la restriccion es transitoria segun su propio mensaje; sin test que la afirme]
- **Filas afectadas:** AT-ASK-004
- **Unidades afectadas:** ASK
- **Origen:** typed-extended

### Q-T-BEH-2 — Colapsar receivePartial/receiveMessagePartial/transformMessages en cierres con Unhandled

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PartialFunction no existe en Rust; Scala usa applyOrElse a Behaviors.unhandled (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:157-171) y actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:238-243 para transformMessages.
- **Alternativas:** (a) un solo receive con match que devuelve Behavior::Unhandled en el brazo _; transform_messages con Option<I>; (b) mantener variantes *_partial como alias de closures que devuelven Option<Behavior>.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Consecuencia observable: ninguna si el brazo no manejado devuelve Unhandled (se publica UnhandledMessage). No bloquea; puede avanzar con (a). | Recomendacion previa: (a) y documentar el alias (b) solo si el Owner quiere paridad de nombres.
- **Filas afectadas:** AT-BEH-008.1; AT-BEH-008.2; AT-BEH-008.3; AT-BEH-016
- **Unidades afectadas:** BEH
- **Origen:** typed-core

### Q-T-BEH-3 — Filtro por clase de interceptores (interceptMessageClass) y isSame

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** InterceptorImpl.receive usa interceptMessageClass.isAssignableFrom(msg.getClass) (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala:90-99); monitor, logMessages, withMdc y transformMessages usan ClassTag (actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/Behaviors.scala:206, 214, 284); TransformMessagesInterceptor pasa mensajes de otra clase al interno con cast sin chequeo (InterceptorImpl.scala:97).
- **Alternativas:** (a) funcion de proyeccion fn(&Outer)->Option<Inner> mas TypeId para isSame; (b) enums de protocolo exterior con variante para mensajes ajenos; (c) mantener Box<dyn Any> y downcast (alinea con Q-X-3).
- **Recomendación:** Clasificacion: representacion que conserva el contrato (con limite). Consecuencia observable: el filtro por clase solo es observable si el protocolo exterior es una jerarquia; en Rust equivale a la proyeccion. Bloquea el cierre de AT-BEH-013..019 en cuanto a firma; la semantica de aroundReceive/aroundSignal y la deduplicacion isSame pueden avanzar. | Recomendacion previa: (a) con isSame por TypeId y valor; los mensajes ajenos solo existen si el protocolo exterior es un enum (b); monitor exige M: Clone.
- **Filas afectadas:** AT-BEH-013; AT-BEH-014; AT-BEH-015; AT-BEH-016; AT-BEH-017; AT-BEH-018; AT-BEH-019
- **Unidades afectadas:** BEH
- **Origen:** typed-core

### Q-T-BEH-4 — narrow de Behavior y de ActorRef (contravarianza) y su identidad

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:56 y actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorRefImpl.scala:29-31: narrow devuelve this, por lo que el ref estrechado es == al original; en Rust requiere envolver con From.
- **Alternativas:** (a) narrow como adaptador explicito (closure From) cuyo path/uid y Eq coinciden con el original; (b) mismo objeto con tipo borrado (Box<dyn Any>) y narrow sin costo (alinea con Q-X-3); (c) no portar narrow y pedir enums de protocolo.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Consecuencia observable: Eq/Hash/Ord por path+uid deben coincidir con el original (ActorRefImpl.scala:29-40). Bloquea la firma de narrow; no bloquea tell/igualdad. | Recomendacion previa: (a) con Eq/Hash/Ord por path+uid independientes del tipo de mensaje.
- **Filas afectadas:** AT-BEH-001; AT-BEH-022; AT-REF-001; AT-REF-003; AT-REF-005
- **Unidades afectadas:** BEH; REF
- **Origen:** typed-core

### Q-T-BEH-5 — AbstractBehavior en Rust: contexto por parametro y desaparicion de la clase de error 'wrong ActorContext'

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/AbstractBehavior.scala:48-51, 82-88; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:188-277; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:46-51.
- **Alternativas:** (a) trait ActorBehavior<M> con on_message(&mut self, ctx, msg) y fabrica Fn por supervise, sin guardar el contexto; (b) replicar el contexto en el struct y los errores en runtime.
- **Recomendación:** Clasificacion: divergencia funcional (aceptable por diseño). Consecuencia observable: desaparece IllegalStateException de contexto equivocado y el rechazo de supervise(AbstractBehavior). Bloquea AT-BEH-010 y AT-SUP-013; el resto avanza. | Recomendacion previa: (a). Divergencia de forma aceptable porque elimina IllegalStateException de contexto equivocado y el rechazo de supervise(AbstractBehavior).
- **Filas afectadas:** AT-BEH-010; AT-SUP-013; AT-CTX-016
- **Unidades afectadas:** BEH; SUP; CTX
- **Origen:** typed-core

### Q-T-BEH-6 — MDC y logMessages sobre backend de logging (tracing) en vez de MDC thread-local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/WithMdcBehaviorInterceptor.scala:44-117 usa org.slf4j.MDC (thread-local); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:340-481.
- **Alternativas:** (a) span de tracing por mensaje con los campos del MDC y clave pekkoSource/pekkoAddress/sourceActorSystem/pekkoTags; (b) task-local propio; (c) no portar MDC.
- **Recomendación:** Clasificacion: arquitectura. Consecuencia observable: MDC visible solo via tracing; claves pekkoSource/pekkoAddress/sourceActorSystem/pekkoTags a conservar. Bloquea AT-BEH-018/019 (dependen tambien de Q-X-7); no bloquea el resto de BEH. | Recomendacion previa: (a). Coordinar con el grupo LOG (AT-LOG-002) y Q-X-7.
- **Filas afectadas:** AT-BEH-018; AT-BEH-019
- **Unidades afectadas:** BEH
- **Origen:** typed-core

### Q-T-CTX-1 — messageAdapter: coincidencia por tipo exacto y comportamiento de watch/ask sobre el ref adaptador (FunctionRef)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:306-331 (isAssignableFrom y orden inverso de registro; ver tambien actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:182-196); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec.scala:117-209.
- **Alternativas:** (a) coincidencia por TypeId exacto, uno por TypeId, mas reciente primero; (b) emular subtipado con una funcion de coincidencia fn(&dyn Any)->bool; en ambos: decidir si el ref adaptador es watchable.
- **Recomendación:** Clasificacion: representacion que conserva el contrato (con divergencia en subtipado). Consecuencia observable: el match por subclase desaparece; un adaptador registrado para un supertipo no recibe subtipos. Bloquea AT-CTX-011; no bloquea ask/pipeToSelf. | Recomendacion previa: (a); documentar que el match por subclase no existe; el watch sobre el adaptador replica el comportamiento de FunctionRef solo si Q-X-8 elige la opcion (a) (capa clasica).
- **Filas afectadas:** AT-CTX-011; AT-CTX-018
- **Unidades afectadas:** CTX
- **Origen:** typed-core

### Q-T-CTX-2 — Chequeo de hilo del ActorContext: reemplazar ActorThreadSpec por garantias de compilacion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:338-376; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorThreadSpec.scala:50-277.
- **Alternativas:** (a) ActorContext<'a> solo existe como &mut en el handler, no es Send/'static y tests trybuild; (b) mantener chequeo en runtime con el mismo texto de error.
- **Recomendación:** Clasificacion: arquitectura. Consecuencia observable: los 9 casos de ActorThreadSpec dejan de ser errores en runtime y pasan a no compilar. No bloquea semantica del contexto. | Recomendacion previa: (a). Mantener en el handoff una tabla de los 9 casos de ActorThreadSpec y como cada uno se vuelve compile-fail o no aplica.
- **Filas afectadas:** AT-CTX-016; AT-BEH-010
- **Unidades afectadas:** CTX; BEH
- **Origen:** typed-core

### Q-T-CTX-3 — Errores de uso del contexto (IllegalArgument/IllegalState): Result vs panic

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:77-104 (stop de no-hijo/self), actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala:125-131 (watch/watchWith), actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorRefFactoryAdapter.scala:27 (despliegue remoto).
- **Alternativas:** (a) Result<_, ContextError> con variantes NotAChild, StopSelf, WatchConflict{previous,new}, SpawnError; (b) panic con los mismos textos; (c) hibrido: Result para spawn/stop, panic para violaciones de contrato.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Consecuencia observable: Result frente a excepcion; el texto del error se conserva. Bloquea la firma de spawn/stop/watch/watchWith (AT-CTX-002/004/005/006); la logica avanza. | Recomendacion previa: (a) conservando los textos de error como Display (ver Contratos AT-CTX-004/005).
- **Filas afectadas:** AT-CTX-002; AT-CTX-004; AT-CTX-005; AT-CTX-006
- **Unidades afectadas:** CTX
- **Origen:** typed-core

### Q-T-CTX-4 — Puente al contexto clasico (toClassic/actorOf) y su error cuando el contexto no es adaptado

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:26-42 (UnsupportedOperationException); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/adapter/package.scala:142-152; contrato AT-CTX-017.
- **Alternativas:** (a) con capa clasica (Q-X-8): to_classic() -> Result/Option y actor_of; (b) diseño nativo: no exponer el puente y omitir la fila; (c) puente solo en un modulo de compatibilidad opcional.
- **Recomendación:** Clasificacion: arquitectura (depende de Q-X-8). Recomendacion: seguir Q-X-8; (a) si hay capa clasica, devolviendo Result en lugar de lanzar. Consecuencia observable: sin capa clasica el actor typed no puede crear hijos clasicos. Bloquea la ejecutabilidad de AT-CTX-017 y AT-SUP-020/021; el contrato esta cerrado y el resto de CTX avanza.
- **Filas afectadas:** AT-CTX-017; AT-SUP-020
- **Unidades afectadas:** CTX; SUP
- **Origen:** typed-core

### Q-T-DLV-1 — Alcance de delivery y descomposicion antes de implementar

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:66-71; DS (specs 2673 lineas sin leer) | Cierre documental 2026-10-31: Resuelta la parte de alcance por lectura de ProducerControllerImpl (855 lineas), ConsumerControllerImpl (743), WorkPullingProducerControllerImpl (704) y de los 10 specs de delivery (ProducerControllerSpec, ConsumerControllerSpec, DurableProducerControllerSpec, WorkPullingSpec, DurableWorkPullingSpec, ReliableDeliverySpec, ReliableDeliveryRandomSpec, DurableProducerQueueSpec y los Test*): AT-DLV-001/002/003/005 se descomponen en AT-DLV-001.1-6, 002.1-7, 003.1-6 y 005.1-3, todas con aserciones citadas. Los variantes con `send` propio (Cluster Sharding) quedan fuera de alcance (Q-X-10).
- **Alternativas:** (a) solo variante local con cola en memoria (como indica el inventario) y descomponer DLV-001/002/003 tras leer implementacion; (b) ademas DurableProducerQueue en memoria
- **Recomendación:** [Exclusion de alcance acotada + arquitectura] (a) variante local con cola en memoria + (b) cola durable en memoria de referencia (Q-T-DLV-2). Consecuencias observables: ProducerController/ConsumerController/WorkPulling locales con identica secuencia de mensajes internos (Request/Resend/Ack/SequencedMessage); sin Sharding ni persistencia real. Bloquea: nada para la especificacion (todas las filas DLV salvo 005.* estan extraidas); AT-DLV-005.* y 001.6/003.6 dependen de Q-T-SER-1/Q-T-DLV-2 para impl. Puede avanzar: AT-DLV-001.1-5, 002.1-7, 003.1-5. [Recomendacion previa: (a)+(b) opcional; fila sigue listed hasta leer las implementaciones]
- **Filas afectadas:** AT-DLV-001;AT-DLV-002;AT-DLV-003;AT-DLV-005
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DLV-2 — Implementacion durable del DurableProducerQueue

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/ProducerController.scala:66-71; actor-typed/src/main/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQueue.scala:41-80 | Cierre documental 2026-10-31: DurableProducerQueue.scala:41-246 (leido completo) define el protocolo y las funciones puras de State; la cola real (EventSourcedProducerQueue) esta en pekko-persistence-typed, fuera de estos modulos (Q-X-10); TestDurableProducerQueue.scala:25-111 es la referencia en memoria usada por los specs y encarna el contrato que el ProducerController espera (carga con limpieza de chunks, re-ack ante duplicado, silencio ante seqNr inesperado).
- **Alternativas:** (a) solo el trait; (b) cola en memoria de referencia (para tests); (c) persistir con storage propio
- **Recomendación:** [Arquitectura / exclusion acotada] (b) cola en memoria de referencia equivalente a TestDurableProducerQueue, mas el protocolo. Consecuencias observables: no hay durabilidad real tras reinicio del proceso; el protocolo y los reintentos son verificables. Bloquea: AT-DLV-001.6, 003.6, 005.3 y 004 (implementacion) si el Owner exige persistencia real. Puede avanzar: AT-DLV-004 (State puro) y toda la parte sin durable. [Recomendacion previa: (b): permite validar el protocolo; la durable real es persistence-typed (OUT)]
- **Filas afectadas:** AT-DLV-004;AT-DLV-001.6;AT-DLV-003.6;AT-DLV-005.3
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DLV-3 — Equivalente del flight recorder JFR

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorFlightRecorder.scala; scala-jdk-9/jfr/Events.scala | Cierre documental 2026-10-31: Leidos ActorFlightRecorder.scala (111), Events.scala (21 eventos), JFRActorFlightRecorder.scala y FlightRecorderLoader.scala; ningun test lo referencia (AT-DLV-007 A1).
- **Alternativas:** (a) eventos tracing con los mismos nombres; (b) no portar (propuesta de exclusion, requiere autorizacion)
- **Recomendación:** [Representacion / exclusion de alcance si (b)] (a) eventos tracing con los mismos nombres tras una feature opcional. Consecuencias observables: telemetria equivalente en tracing, sin JFR. Bloquea: nada. Puede avanzar: toda DLV (el recorder es pasivo). [Recomendacion previa: (a)]
- **Filas afectadas:** AT-DLV-007
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DLV-4 — Tiempo y semillas en la validacion de delivery (reloj real y aleatoriedad del original)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Los specs de delivery dependen de tiempo real (resend-first 1 s, expectNoMessage(1.1 s), resend-interval-min 1 s, ConsumerControllerSpec:185-217, ProducerControllerSpec:47-72) y de semillas `System.currentTimeMillis()` (ReliableDeliveryRandomSpec:155-169, 171-201, 203-216); WorkPulling elige workers con ThreadLocalRandom (WorkPullingProducerControllerImpl.scala:409).
- **Alternativas:** (a) portar las pruebas con tiempo controlado (ManualTime de AT-TK-007) y semilla fija registrada; (b) portarlas con tiempo real y tolerancias; (c) solo tiempo real en el diferencial con el jar
- **Recomendación:** [Arquitectura de validacion] (a): determinismo sin cambiar el contrato (intervalos y factores son configuracion). Consecuencias observables: ninguna en el producto; cambia como se valida. Bloquea: nada; condiciona la validacion de AT-DLV-001.3, 002.3, 002.4, 002.6. Puede avanzar: todo lo demas.
- **Filas afectadas:** AT-DLV-001.3;AT-DLV-002.3;AT-DLV-002.4;AT-DLV-002.5;AT-DLV-002.6;AT-DLV-003.2
- **Unidades afectadas:** DLV
- **Origen:** typed-extended

### Q-T-DSP-1 — Orden de resolucion de Props: ultimo agregado gana

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala:75-113; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/PropsSpec.scala:21-30 | Cierre documental 2026-10-31: Props.scala:75-113 (`withNext` antepone) y PropsSpec:21-30 afirman que gana el ultimo agregado; ActorTags aplica solo el primero de la cadena (PropsAdapter.scala:53).
- **Alternativas:** (a) conservar lista con anteposicion; (b) builder con primer-gana
- **Recomendación:** [Representacion que conserva el contrato] (a). Consecuencias observables: orden de resolucion de Props identico. Bloquea: nada. Puede avanzar: AT-DSP-001/002/003/004. [Recomendacion previa: (a): lo afirma el spec]
- **Filas afectadas:** AT-DSP-001;AT-DSP-004
- **Unidades afectadas:** DSP
- **Origen:** typed-extended

### Q-T-DSP-2 — Mapeo de DispatcherSelector a ejecutores de Rust (hilos vs tareas tokio)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelectorSpec.scala:67-135
- **Alternativas:** (a) nombre de runtime/pool; (b) hilo dedicado por actor
- **Recomendación:** [Arquitectura; depende de Q-X-4] El unico contrato independiente del runtime es la existencia de fromConfig/sameAsParent/blocking y el nombre de los hilos del dispatcher (afirmado por DispatcherSelectorSpec:67-134 como prefijo "DispatcherSelectorSpec-ping-pong-dispatcher"). Consecuencias observables: los nombres de hilo/pool pueden diferir; se conserva "el actor corre en el ejecutor elegido". Bloquea: AT-DSP-002/005 impl hasta Q-X-4. Puede avanzar: contratos. [Recomendacion previa: depende de Q-X-4; aqui solo se fija que fromConfig/sameAsParent/blocking existan]
- **Filas afectadas:** AT-DSP-002
- **Unidades afectadas:** DSP
- **Origen:** typed-extended

### Q-T-DSP-3 — Semantica de mailbox acotado lleno (dead letters) y default single-consumer

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:75-95
- **Alternativas:** (a) dead letters con evento; (b) backpressure
- **Recomendación:** [Divergencia funcional evitada: paridad] (a). Consecuencias observables: mailbox acotado lleno envia a dead letters (MailboxSelectorSpec:75-95). Bloquea: AT-DSP-003 impl hasta Q-X-4 (modelo de mailbox). Puede avanzar: contrato. [Recomendacion previa: (a): paridad con Pekko]
- **Filas afectadas:** AT-DSP-003
- **Unidades afectadas:** DSP
- **Origen:** typed-extended

### Q-T-EVT-1 — Tres caminos distintos para publicar Dropped (classic eventStream, typed EventStream.Publish, deadLetters)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:120-124; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/GroupRouterImpl.scala:99-102; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/pubsub/TopicImpl.scala:96; actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:736-743 | Cierre documental 2026-10-31: Resuelta por lectura la parte factual: Supervision.scala:120-124 publica Dropped en el eventStream CLASICO; GroupRouterImpl.scala:99-102 publica EventStream.Publish(Dropped) TIPADO; TopicImpl.scala:96 envia `deadLetters ! Dropped(..)`, y DeadLetterActorRef.! (ActorRef.scala:736-743) envuelve cualquier mensaje que no sea DeadLetter como DeadLetter(msg, ..) y lo publica: los suscriptores de Dropped (p. ej. createDroppedMessageProbe, ActorTestKit.scala:306-308) NO ven el descarte de Topic; los de DeadLetter ven DeadLetter(Dropped(..)). Sin spec que lo afirme para Topic.
- **Alternativas:** (a) un unico publish_dropped al event stream (Topic cambia de DeadLetter(Dropped) a Dropped); (b) preservar los tres caminos; (c) unificar y pedir al oraculo confirmar el efecto en Topic
- **Recomendación:** [Incertidumbre resuelta leyendo la fuente; queda una divergencia a decidir] La recomendacion ya no es "(c) validar con jar": el efecto esta determinado. Opciones: (b) preservar los tres caminos (paridad exacta, DeadLetter(Dropped) para Topic) o (a) unificar (suscriptores de Dropped veran tambien los descartes de Topic). Recomendacion: (b) por paridad; (a) solo si el Owner prefiere coherencia. Consecuencias observables: quien se suscriba a Dropped o a DeadLetter ve cosas distintas segun el origen. Bloquea: cierre de AT-EVT-004, AT-PS-001.2, AT-RT-004. Puede avanzar: AT-EVT-001..003 y el resto de AT-PS. [Recomendacion previa: (c): validar con jar oficial y decidir; observable para suscriptores de DeadLetter/Dropped]
- **Filas afectadas:** AT-EVT-004;AT-PS-001.2;AT-RT-004
- **Unidades afectadas:** EVT;PS;RT
- **Origen:** typed-extended

### Q-T-EXT-1 — Politica "skip" en extensions cuando la clase no resuelve

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:56-65; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ExtensionsSpec (no afirma el skip) | Cierre documental 2026-10-31: Resuelta por lectura (ExtensionsImpl.scala:39-66): `loadExtensionsFor(key, throwOnLoadFail)`: en `pekko.actor.typed.library-extensions` (throwOnLoadFail = true) una clase que no carga o no es ExtensionId lanza RuntimeException (`[<fqcn>] is not an ExtensionId` / `While trying to load extension [<fqcn>]`); en `pekko.actor.typed.extensions` (false) se registra error y se omite ("skipping..."); un fallo de CREACION de la extension hace fallar el arranque en ambos. ExtensionsSpec:136-153 afirma el fallo de creacion, no el skip.
- **Alternativas:** (a) conservar log+skip para nombres no resolubles; (b) fallar siempre
- **Recomendación:** [Incertidumbre resuelta leyendo la fuente + representacion] (a) conservar la politica por clave: `extensions` = log+skip y `library-extensions` = fallo (registro de fabricas por nombre, Q-X-2). Consecuencias observables: un nombre no resoluble en `extensions` no impide arrancar; en `library-extensions` si. Bloquea: nada. Puede avanzar: AT-EXT-002.1/002.2 con caracterizacion del skip. [Recomendacion previa: (a): paridad con el codigo; sin spec que lo afirme]
- **Filas afectadas:** AT-EXT-002.2
- **Unidades afectadas:** EXT
- **Origen:** typed-extended

### Q-T-EXT-2 — Identidad de ExtensionId por referencia en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/Extensions.scala:131; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:155-163
- **Alternativas:** (a) puntero (Arc ptr_eq) / direccion de static; (b) TypeId + id numerico explicito
- **Recomendación:** [Representacion] (a) identidad por puntero/direccion de static. Consecuencias observables: dos ExtensionId distintos con igual tipo son extensiones distintas (como en Pekko, ExtensionsSpec:155-163). Bloquea: nada. Puede avanzar: AT-EXT-001.*. [Recomendacion previa: (a)]
- **Filas afectadas:** AT-EXT-001.1;AT-EXT-001.3
- **Unidades afectadas:** EXT
- **Origen:** typed-extended

### Q-T-EXT-3 — Relanzar la misma excepcion en cada acceso tras un fallo de creacion

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ExtensionsImpl.scala:110-114,134 | Cierre documental 2026-10-31: Resuelta por lectura (ExtensionsImpl.scala:110-114,134): el fallo de creacion se guarda en el mapa de instancias y se relanza la MISMA excepcion en cada acceso posterior; el Scaladoc lo documenta.
- **Alternativas:** (a) error cacheado clonable; (b) reintentar en cada acceso
- **Recomendación:** [Incertidumbre resuelta leyendo la fuente] (a) error cacheado clonable; no requiere al Owner salvo que se quiera reintentar (b). Consecuencias observables: segundo acceso falla con el mismo error. Bloquea: nada. Puede avanzar: AT-EXT-001.2. [Recomendacion previa: (a): lo dicen el codigo y el Scaladoc; el test solo comprueba que el sistema no arranca]
- **Filas afectadas:** AT-EXT-001.2
- **Unidades afectadas:** EXT
- **Origen:** typed-extended

### Q-T-JAPI-1 — Fachada javadsl en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/javadsl/*.scala (2353 lineas); actor-typed-tests/src/test/java | Cierre documental 2026-10-31: Leida la fachada javadsl (12 archivos) y los 13 tests Java: la mayor parte delega en scaladsl/BehaviorImpl (AT-JAPI-001.1, 005.1, 006.2-006.5, evidencia de delegacion con lineas); comportamiento propio de la fachada: senal total de receive(m,s), normalizacion de null en withMdc, cache de createReceive en AbstractBehavior, ReceiveBuilder mutable frente a BehaviorBuilder inmutable, convencion (res, ex) de ask/pipeToSelf, boxing de messageAdapter(Class) y startPeriodicTimer = fixed delay. Varios tests Java no afirman nada (ActorLoggingTest, ReceiveBuilderTest.testMutableCounter): ver hallazgos en AT-JAPI-001.1/001.3/002.2.
- **Alternativas:** (a) no portar: Rust tiene una API; las aserciones de scaladsl cubren la semantica (propuesta de exclusion, requiere autorizacion); (b) portar helpers equivalentes para usuarios que vengan de Java
- **Recomendación:** [Exclusion de alcance parcial + divergencia funcional a decidir] (a) no portar la ergonomia Java (Class/ClassTag, japi.function, Optional/CompletionStage/Duration java.time): la API Rust unica reutiliza las aserciones de scaladsl; el comportamiento propio de la fachada (defaults de senales, createReceive diferido, convenciones (res, ex), startPeriodicTimer) se decide en Q-T-JAPI-2 y se verifica como propiedad de la API Rust. Consecuencias observables: sin AbstractBehavior/ReceiveBuilder/BehaviorBuilder Java; las 18 filas JAPI siguen en alcance hasta que el Owner autorice la exclusion. Bloquea: impl de AT-JAPI-001.2, 002.1, 002.2, 003 y 004 (dependen de Q-T-JAPI-2). Puede avanzar: AT-JAPI-001.1, 005.1, 006.2-006.5 (alias de filas AT-*). [Recomendacion previa: (a) con autorizacion explicita; la fila sigue en alcance hasta entonces]
- **Filas afectadas:** AT-JAPI-001;AT-JAPI-002;AT-JAPI-003;AT-JAPI-004;AT-JAPI-005;AT-JAPI-006
- **Unidades afectadas:** JAPI
- **Origen:** typed-extended

### Q-T-JAPI-2 — Default ante MessageAdaptionFailure y otras senales no manejadas en los builders de la API Rust (tres defaults distintos en la fachada Java)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ActorAdapter.scala:220-229 ignora senales `unhandled`; ReceiveBuilder/AbstractBehavior (javadsl) relanzan la excepcion del adaptador (BehaviorImpl.unhandledSignal), BehaviorBuilder y AbstractMatchingBehavior no tienen manejador por defecto (el actor sobrevive) y Behaviors.receive(m, s) depende de la funcion de senales suministrada (AT-JAPI-001.2 A3, 002.2, 003, 004). Sin spec Java que lo afirme; el equivalente scaladsl esta afirmado por AdaptationFailureSpec:75-129 (AT-ADP-006.1 A5).
- **Alternativas:** (a) un unico default (relanzar la excepcion del adaptador, como scaladsl y ReceiveBuilder) en todos los builders Rust; (b) reproducir los tres defaults de la fachada Java; (c) default configurable por builder
- **Recomendación:** [Divergencia funcional] (a): coherente con scaladsl, que es lo que afirman los specs. Consecuencias observables: con (a) un actor construido con el equivalente de BehaviorBuilder/AbstractMatchingBehavior que reciba una MessageAdaptionFailure falla en lugar de sobrevivir (difiere de Pekko Java). Bloquea: impl de AT-JAPI-001.2, 002.1, 002.2, 003 y 004. Puede avanzar: el resto de JAPI y toda la API Rust de scaladsl.
- **Filas afectadas:** AT-JAPI-001.2;AT-JAPI-002.1;AT-JAPI-002.2;AT-JAPI-003;AT-JAPI-004
- **Unidades afectadas:** JAPI
- **Origen:** typed-extended

### Q-T-LOG-1 — Fachada de logging: tracing vs log+kv, y nombres de clave MDC

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorMdc.scala:24-27; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala:160-188; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:76-129 | Cierre documental 2026-10-31: ActorMdc.scala:24-27; ActorContextImpl.scala:44-90,173-201; ActorLoggingSpec:270-289,338-529 afirman las claves exactas pekkoSource/sourceActorSystem/pekkoAddress/pekkoTags y el aislamiento por mensaje.
- **Alternativas:** (a) tracing con span por mensaje y campos pekko_*; (b) log + kv; (c) trait propio con backend enchufable
- **Recomendación:** [Representacion que conserva el contrato] (a) tracing con span por mensaje y campos pekko_*; mantener los nombres camelCase como alias para comparaciones diferenciales. Consecuencias observables: los consumidores ven los mismos valores bajo campos de span; el MDC thread-local no se hereda por tareas lanzadas desde el actor (diferencia aceptada). Bloquea: cierre de impl de AT-LOG-001/002, AT-DSP-004 y la captura de AT-TK-006. Puede avanzar: contratos (ya extraidos). [Recomendacion previa: (a); mantener los nombres camelCase de Pekko como claves opcionales para compatibilidad de dashboards]
- **Filas afectadas:** AT-LOG-001;AT-LOG-002;AT-LOG-005;AT-DSP-004;AT-TK-006
- **Unidades afectadas:** LOG;DSP;TK
- **Origen:** typed-extended

### Q-T-LOG-2 — Deteccion del nombre del logger por stack (JVM-only)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/LoggerClass.scala:24-60; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.scala:131-173 | Cierre documental 2026-10-31: LoggerClass.scala:24-60 recorre el stack para hallar la primera clase fuera de Behavior/Behaviors; ActorLoggingSpec:131-173 afirma el nombre de la clase que llama.
- **Alternativas:** (a) nombre = type_name del Behavior/AbstractBehavior mas externo; (b) obligatorio set_logger_name; (c) module_path! del call site
- **Recomendación:** [Representacion] (a)+(c): type_name del Behavior mas externo o module_path del call site. Consecuencias observables: el nombre del logger por defecto es el del modulo/tipo donde se definio el comportamiento (mismo efecto observable, otro formato); la detección por stack no se porta. Bloquea: nada. Puede avanzar: AT-LOG-001. [Recomendacion previa: (a)+(c): conserva el efecto observable "nombre = donde se definio el comportamiento"]
- **Filas afectadas:** AT-LOG-001
- **Unidades afectadas:** LOG
- **Origen:** typed-extended

### Q-T-LOG-3 — pekko.use-slf4j / Slf4jLogger como configuracion en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/resources/reference.conf:56-68; actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:335-370 | Cierre documental 2026-10-31: Settings.amendSlf4jConfig (ActorSystem.scala:335-370) y ActorLoggingSpec:298-336 afirman la reescritura de pekko.loggers/logging-filter.
- **Alternativas:** (a) aceptar la clave y mapearla a "activar subscriber de tracing"; (b) ignorarla; (c) excluir (propuesta de exclusion, requiere autorizacion del Owner)
- **Recomendación:** [Exclusion de alcance parcial / representacion] (a) aceptar la clave y mapearla a "activar subscriber de tracing". Consecuencias observables: la clave se acepta; no hay Slf4jLogger. Bloquea: AT-LOG-003 (depende de Q-X-7). Puede avanzar: el resto de LOG. [Recomendacion previa: (a)]
- **Filas afectadas:** AT-LOG-003
- **Unidades afectadas:** LOG
- **Origen:** typed-extended

### Q-T-LOG-4 — LoggerOps (Scala 2) subsumido por macros de tracing

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/package.scala:30-120 | Cierre documental 2026-10-31: Leidos package.scala:30-397 (20 metodos, todos delegan en SLF4J) y LoggerOpsSpec:37-86 (solo info2/infoN, afirma el mensaje renderizado).
- **Alternativas:** (a) no portar: las macros de tracing resuelven el problema de varargs; (b) portar helpers equivalentes
- **Recomendación:** [Exclusion de alcance con subsuncion] (a) no portar: las macros de tracing resuelven la ambiguedad. Consecuencias observables: se pierde la API simbolica (info2/infoN...), el texto emitido es el mismo. Bloquea: nada. Puede avanzar: AT-LOG-004 como verificacion del mensaje renderizado. La exclusion requiere autorizacion explicita del Owner. [Recomendacion previa: (a): propuesta de que no haga falta equivalente; la fila sigue en alcance hasta que el Owner decida]
- **Filas afectadas:** AT-LOG-004
- **Unidades afectadas:** LOG
- **Origen:** typed-extended

### Q-T-RCP-1 — Igualdad de ServiceKey: (id, nombre de clase JVM) vs (id, TypeId de Rust)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKey.scala:41-43; actor-typed/src/main/scala/org/apache/pekko/actor/typed/receptionist/Receptionist.scala:41-48 | Cierre documental 2026-10-31: ServiceKey.scala:41-43 compara (id, typeName JVM) y el serializador usa el nombre de clase (ServiceKeySerializer.scala:30-43). Sin spec que ejercite claves genericas con igual borrado.
- **Alternativas:** (a) (id,TypeId) de Rust: distingue Vec<i32> de Vec<String>; (b) (id,type_name) imitando typeName; (c) solo id con chequeo en runtime
- **Recomendación:** [Divergencia funcional controlada] (a) (id, TypeId de Rust). Consecuencias observables: dos claves con el mismo id y tipos distintos nunca colisionan (en Pekko podian colisionar si el borrado de tipo coincidia) y el manifest de ServiceKey en el cable difiere (ver Q-T-SER-2). Bloquea: cierre de AT-RCP-002/005.3 y AT-SER-002. Puede avanzar: Register/Find/Subscribe con claves de tipo no generico. [Recomendacion previa: (a): mas seguro; difiere de Pekko en claves con genericos del mismo borrado (no afirmado por tests)]
- **Filas afectadas:** AT-RCP-002;AT-RCP-005.3;AT-SER-002
- **Unidades afectadas:** RCP;SER
- **Origen:** typed-extended

### Q-T-RCP-2 — Punto de extension del receptionist de cluster

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ReceptionistImpl.scala:27-50; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:28-36
- **Alternativas:** (a) trait ReceptionistBehaviorProvider y registro explicito; (b) sin extension point (solo local)
- **Recomendación:** [Arquitectura] (a) trait ReceptionistBehaviorProvider con registro explicito. Consecuencias observables: ninguna con el receptionist local; el de cluster queda fuera (Q-X-10). Bloquea: nada. Puede avanzar: AT-RCP-003..008 con LocalReceptionist. [Recomendacion previa: (a): costo bajo; cluster-typed queda fuera del modulo]
- **Filas afectadas:** AT-RCP-001
- **Unidades afectadas:** RCP
- **Origen:** typed-extended

### Q-T-RCP-3 — Deregister de clave no registrada con actor que tiene otras claves notifica Listing sin cambios y responde Deregistered

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala:195-218 (sin spec) | Cierre documental 2026-10-31: Resuelta por lectura (LocalReceptionist.scala:195-218, leido completo): Deregister de un actor sin ningun servicio registrado no responde ni notifica (197-199); Deregister de una clave no registrada de un actor que tiene otras claves SI responde Deregistered y notifica un Listing sin cambios (205-216); Register repetido tambien notifica de nuevo (185-193). Sin spec que lo afirme (LocalReceptionistSpec:91-143 solo cubre desregistros efectivos).
- **Alternativas:** (a) conservar; (b) no notificar cuando el estado no cambia
- **Recomendación:** [Incertidumbre resuelta leyendo la fuente: el comportamiento es el descrito] Se conserva por paridad ((a)); solo requiere al Owner si se quisiera divergir de Pekko con (b) "no notificar cuando no cambia". Consecuencias observables: los suscriptores reciben Listings repetidos identicos. Bloquea: nada. Puede avanzar: AT-RCP-003/004/005.2 con caracterizacion de estos casos. [Recomendacion previa: (a) por paridad; tambien Register repetido notifica de nuevo]
- **Filas afectadas:** AT-RCP-003;AT-RCP-004;AT-RCP-005.2
- **Unidades afectadas:** RCP
- **Origen:** typed-extended

### Q-T-RT-1 — Hashing consistente byte-compatible con Pekko

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:101-122 (usa pekko.routing.ConsistentHash de classic) | Cierre documental 2026-10-31: RoutingLogic.scala:101-122 usa pekko.routing.ConsistentHash (clasico, AC): RoutingLogicSpec:163-234 solo afirma consistencia (mismo routee para la misma clave, estable al cambiar el conjunto), no los nodos concretos elegidos.
- **Alternativas:** (a) portar ConsistentHash identico (MurmurHash/rehash) para igualar nodos elegidos; (b) algoritmo propio con las mismas garantias (estabilidad), sin compatibilidad
- **Recomendación:** [Divergencia funcional controlada] (b) algoritmo propio con las mismas garantias. Consecuencias observables: el routee concreto elegido para una clave puede diferir del de Pekko (los specs no lo afirman); ninguna perdida de garantia afirmada. Bloquea: nada para impl; el oraculo no podra comparar nodos elegidos si se elige (b). Puede avanzar: AT-RT-002.3 con las aserciones A1-A4. [Recomendacion previa: (b) salvo que se requiera interoperar con cluster Pekko (fuera del modulo)]
- **Filas afectadas:** AT-RT-002.3
- **Unidades afectadas:** RT
- **Origen:** typed-extended

### Q-T-RT-2 — Comportamiento de selectRoutee sin routees (NPE/AIOOBE/IllegalArgument/IllegalState segun logica)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:54-62,91-93,109 | Cierre documental 2026-10-31: Resuelta por lectura (RoutingLogic.scala:50-122): round robin sin routeesUpdated previo falla por NPE (currentRoutees == null) y con arreglo vacio por ArrayIndexOutOfBounds; random con arreglo vacio lanza IllegalArgumentException (nextInt(0)); consistent hashing lanza IllegalStateException "Can't get node for [k] from an empty node ring" (afirmado, RoutingLogicSpec:170-178). Los routers nunca llaman selectRoutee sin routees (GroupRouterImpl:54-63 y PoolRouterImpl:54-60 lo evitan), por lo que los dos primeros casos son inalcanzables desde la API publica.
- **Alternativas:** (a) Option/Result uniforme; (b) panic con el mismo texto solo para hashing
- **Recomendación:** [Incertidumbre resuelta leyendo la fuente + representacion] (a) Option/Result uniforme en la logica; el unico error contractual es el de hashing (IllegalStateException con texto fijo). Consecuencias observables: ninguna por la API publica. Bloquea: nada. Puede avanzar: AT-RT-002.1-3. [Recomendacion previa: (a): solo el caso de hashing esta afirmado por spec]
- **Filas afectadas:** AT-RT-002.1;AT-RT-002.2;AT-RT-002.3
- **Unidades afectadas:** RT
- **Origen:** typed-extended

### Q-T-RT-3 — Reuso del Behavior de routees y orden de entrega del broadcast

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/PoolRouterImpl.scala:86-90,99-101
- **Alternativas:** (a) Behavior Clone/Arc<dyn Fn> (AT-BEH-023, decision del grupo BEH); (b) factory explicito por routee
- **Recomendación:** [Arquitectura; depende de AT-BEH-023] (a) Behavior: Clone. Consecuencias observables: los routees comparten la definicion del Behavior; el broadcast sigue el orden de ctx.children (no contractual). Bloquea: AT-RT-001/003/004/005 hasta cerrar Q-T-BEH-1 (grupo BEH). Puede avanzar: RoutingLogic puro (AT-RT-002.*). [Recomendacion previa: (a), consistente con la decision de AT-BEH-023]
- **Filas afectadas:** AT-RT-001;AT-RT-003;AT-RT-004;AT-RT-005
- **Unidades afectadas:** RT
- **Origen:** typed-extended

### Q-T-SER-1 — Estrategia de serializacion de mensajes y refs en un runtime local

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/MiscMessageSerializer.scala:28-55; actor-typed/src/main/resources/reference.conf:38-53 | Cierre documental 2026-10-31: Necesidad concreta leida: chunking (ProducerControllerImpl.scala:352-375 serializa con findSerializerFor/manifest; ConsumerControllerImpl.scala:535-549 deserializa) y SerializationTestKit; los specs usan un TestSerializer propio (TestConsumer.scala del spec, id 787878).
- **Alternativas:** (a) trait Serializer con registro por tipo y los dos serializadores internos; (b) solo serde sin registro; (c) no portar mientras no haya remoting/chunking
- **Recomendación:** [Arquitectura / divergencia funcional] (a) minimo: trait de serializacion registrado por tipo con id y manifest. Consecuencias observables: con Rust local el chunking solo tiene sentido con un serializador registrado; sin el, chunk-large-messages > 0 no puede funcionar. Bloquea: AT-DLV-005.1-3 (impl), AT-SER-001/002 (impl) y AT-TK-009 (SerializationTestKit). Puede avanzar: el resto de DLV y la especificacion de 005.*. [Recomendacion previa: (a) minimo, necesario para chunking de delivery y SerializationTestKit; depende de AC-SER]
- **Filas afectadas:** AT-SER-001;AT-SER-002;AT-DLV-005;AT-TK-009
- **Unidades afectadas:** SER;DLV;TK
- **Origen:** typed-extended

### Q-T-SER-2 — Formato de ServiceKey en el cable (manifest = nombre de clase JVM)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/ServiceKeySerializer.scala:30-43
- **Alternativas:** (a) conservar manifest=type_name Rust (no interoperable con JVM); (b) formato propio
- **Recomendación:** [Divergencia funcional controlada] (a) manifest = nombre de tipo Rust, no interoperable con JVM. Consecuencias observables: ServiceKey serializada por un nodo Rust no se lee en Pekko JVM. Bloquea: AT-SER-002 impl. Puede avanzar: contrato. [Recomendacion previa: (a)]
- **Filas afectadas:** AT-SER-002
- **Unidades afectadas:** SER
- **Origen:** typed-extended

### Q-T-SER-3 — Serializacion Java de ActorRef (JVM-only)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:60-112
- **Alternativas:** (a) no portar (propuesta de exclusion, requiere autorizacion); (b) equivalente serde con resolver global
- **Recomendación:** [Exclusion de alcance o divergencia; requiere Owner si (a)] (b) si Q-T-SER-1 elige (a). Consecuencias observables: una ActorRef (de)serializada fuera de contexto necesita un resolver global (currentSystem). Bloquea: AT-SER-003 impl. Puede avanzar: contrato. [Recomendacion previa: (b) si Q-T-SER-1 elige (a)]
- **Filas afectadas:** AT-SER-003
- **Unidades afectadas:** SER
- **Origen:** typed-extended

### Q-T-SIG-1 — Forma de Signal: enum no exhaustivo, ChildFailed como Terminated, igualdad asimetrica y senales internas

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/MessageAndSignals.scala:35-125: Terminated(r) == ChildFailed(r,c) es true y a la inversa false (lineas 90-93 vs 115-118); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:198-203 y PoisonPill.scala:30-37 extienden Signal.
- **Alternativas:** (a) enum Signal #[non_exhaustive] con Terminated, ChildFailed{cause}, y variantes internas privadas; as_terminated() para tratar ambos igual; igualdad simetrica por ref; (b) reproducir la asimetria; (c) Signal como trait object abierto.
- **Recomendación:** Clasificacion: representacion que conserva el contrato (con divergencia en la igualdad). Consecuencia observable: si la igualdad deja de ser asimetrica, un patron `Terminated(r)` hecho con == coincide tambien con ChildFailed (en Pekko el match por tipo coincide con ambos solo porque ChildFailed extiende Terminated). Bloquea la API publica de Signal (AT-SIG-001/004/005); el resto avanza. | Recomendacion previa: (a) con igualdad simetrica documentada como divergencia; las senales internas no se exportan.
- **Filas afectadas:** AT-SIG-001; AT-SIG-004; AT-SIG-005; AT-SIG-008; AT-SUP-017
- **Unidades afectadas:** SIG; SUP
- **Origen:** typed-core

### Q-T-SPV-1 — Spawn[T] existencial y borrado de tipo

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/SpawnProtocol.scala:54-85
- **Alternativas:** (a) mensaje Spawn con Box<dyn Any> + downcast; (b) macro/closure generica con canal tipado por llamada; (c) SpawnProtocol especifico por tipo de hijo
- **Recomendación:** [Arquitectura / representacion] (b) si el API publico se mantiene. Consecuencias observables: el ref devuelto queda tipado por llamada; el nombre unico con sufijo "-1","-2" se conserva (SpawnProtocolSpec:89-111). Bloquea: nada para contratos. Puede avanzar: AT-SPV-001 en cuanto se fije el tipo de Spawn. [Recomendacion previa: (b) si el API publico se mantiene; difiere de Pekko en que el ref devuelto queda tipado por llamada]
- **Filas afectadas:** AT-SPV-001
- **Unidades afectadas:** SPV
- **Origen:** typed-extended

### Q-T-STASH-1 — Buffer interno de supervision que guarda mensajes y senales; UnstashException con Behavior en curso

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:210, 365-375 (StashBuffer[Any] con mensajes y senales); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:187-244, 286-287.
- **Alternativas:** (a) tipo interno Input<M> = Msg(M) | Sig(Signal) para el StashBuffer del supervisor y error Unstash{cause, behavior}; (b) dos buffers separados.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Consecuencia observable: orden relativo de mensajes y senales preservado en el stash de restart. Bloquea AT-SUP-009/018 y AT-STASH-006 en el tipo interno; el stash publico avanza. | Recomendacion previa: (a) preservando el orden relativo de mensajes y senales.
- **Filas afectadas:** AT-SUP-009; AT-SUP-018; AT-STASH-006
- **Unidades afectadas:** SUP; STASH
- **Origen:** typed-core

### Q-T-SUP-1 — Selector de excepciones en supervise().on_failure: por tipo o por predicado

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:70-83, 146; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:82-94 (ClassTag.isAssignableFrom); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:139-147, 448-490. El eje Err/panic lo decide Q-X-5.
- **Alternativas:** (a) on_failure::<E: Error + 'static>(strategy) con downcast_ref sobre el error dinamico; (b) on_failure_if(|e| bool, strategy); (c) ambos.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Consecuencia observable: seleccion de excepcion por tipo o predicado; depende de Q-X-5 (Err vs panic). Bloquea AT-SUP-002 y AT-SUP-011. | Recomendacion previa: (c). 'Throwable' por defecto equivale a un selector que acepta todo.
- **Filas afectadas:** AT-SUP-002; AT-SUP-011
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-2 — Discrepancia doc/codigo en backoff: 'los mensajes se descartan' vs stash con Dropped

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Scaladoc dice 'During the back-off incoming messages are dropped' (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:64, 100); el codigo guarda en stash hasta stashCapacity y publica Dropped al llenarse (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:278-292); el spec lo afirma (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:882-910, 912-936).
- **Alternativas:** (a) seguir el codigo y el spec (stash + Dropped), registrar la discrepancia; (b) seguir el Scaladoc (descartar siempre).
- **Recomendación:** Clasificacion: incertidumbre resuelta leyendo Pekko. Resuelto por lectura y spec: el codigo guarda en stash hasta stashCapacity y publica Dropped al llenarse (Supervision.scala:278-292) y SupervisionSpec.scala:882-936 lo afirma; el Scaladoc (SupervisorStrategy.scala:64, 100) es incorrecto. Seguir codigo y spec; no requiere al Owner (queda proposed solo para que lo ratifique como registro de la discrepancia). | Recomendacion previa: (a).
- **Filas afectadas:** AT-SUP-007.1; AT-SUP-009
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-4 — Setup que falla durante un restart ilimitado (bucle o StackOverflowError)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:294-311 (no reinicia si falla el primer arranque con restarts ilimitados), 392-419 (restartCompleted captura con handleException y puede reintentar); antecedente goPekko S-3 (solo contexto, no vinculante).
- **Alternativas:** (a) reproducir el comportamiento de Pekko (reintento sin limite si el setup falla en un restart); (b) divergir: limitar reintentos de arranque tras restart o tratar la falla de arranque como fatal.
- **Recomendación:** Clasificacion: divergencia funcional posible. Evidencia nueva: SupervisionSpec.scala:779-832 afirma que un setup que falla durante un restart se reintenta (hijos de la encarnacion fallida se detienen) hasta que tiene exito, dentro del limite; con restart ilimitado el reintento no tiene tope (Supervision.scala:414-418). Seguir (a) salvo que el Owner acepte (b). Bloquea solo la rama ilimitada de AT-SUP-005. | Recomendacion previa: (b) como divergencia marcada solo si el Owner la acepta; hasta entonces (a).
- **Filas afectadas:** AT-SUP-005
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-5 — Resume con falla de arranque: envoltura accidental ActorInitializationException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:169-178 (ResumeSupervisor devuelve same) + actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:240-243 (validateAsInitial rechaza same); actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1107-1115 (se loguean TestException y ActorInitializationException). Antecedente goPekko S-5.
- **Alternativas:** (a) reproducir (el actor para y el padre ve ActorInitializationException con IllegalArgumentException como causa); (b) tipar la falla como InitializationFailed{cause} directamente.
- **Recomendación:** Clasificacion: divergencia funcional posible (forma del error). Consecuencia observable: el padre ve ActorInitializationException con IllegalArgumentException como causa; el actor termina. Bloquea AT-SUP-004 (forma del error); la parada y el log avanzan. | Recomendacion previa: (b) conservando el texto y la parada del actor (la envoltura exacta es accidental).
- **Filas afectadas:** AT-SUP-004
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-6 — ScheduledRestart y ResetRestartCount como senales opacas que llegan al Behavior de usuario

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:198-203, 220-250; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1032-1076 (el spec afirma que la senal llega). Antecedente goPekko S-4.
- **Alternativas:** (a) seguir Pekko: variante opaca de Signal con id de duenio que se propaga cuando no coincide; (b) consumirlas siempre en el supervisor duenio (no visibles al usuario).
- **Recomendación:** Clasificacion: arquitectura. Consecuencia observable: el spec afirma que ScheduledRestart llega al Behavior del usuario como senal (SupervisionSpec.scala:1032-1076). Bloquea AT-SUP-017; el backoff avanza. | Recomendacion previa: (a) porque el spec lo afirma; ocultar el tipo concreto.
- **Filas afectadas:** AT-SUP-017; AT-SUP-007.2
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-7 — Padre/hijo clasico: decider clasico y excepcion agotada que sube al padre

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala:55-58, 232-266; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/TypedSupervisingClassicSpec.scala:61-72; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:335-338 (throw t al agotar withLimit). Depende de Q-X-8. Antecedentes goPekko S-2 y S-6.
- **Alternativas:** (a) si Q-X-8 = capa clasica: portar defaultDecider (ActorInitializationException/ActorKilled/DeathPact paran, resto reinicia) y log ERROR; (b) si Q-X-8 = nativo: solo semantica Stop/ChildFailed y sin hijos clasicos.
- **Recomendación:** Clasificacion: arquitectura (depende de Q-X-8). Consecuencia observable: sin capa clasica no hay hijos clasicos ni decider clasico. Bloquea AT-SUP-021 (y la rama clasica de AT-SUP-020/006); lo demas de SUP avanza. | Recomendacion previa: Seguir Q-X-8; (a) tal cual si hay capa clasica.
- **Filas afectadas:** AT-SUP-006; AT-SUP-020; AT-SUP-021
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SUP-8 — Aleatoriedad e instante inyectables en backoff

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:185-196 (ThreadLocalRandom), 430-441 (Deadline.now); tests con tiempo real (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:912-958).
- **Alternativas:** (a) trait BackoffJitter y Clock inyectables con implementaciones reales y de test; (b) tokio::time::pause y semilla fija.
- **Recomendación:** Clasificacion: arquitectura. Consecuencia observable: ninguna con implementaciones reales; permite tests deterministas. No bloquea; coordinar con Q-T-TMR-4. | Recomendacion previa: (a) alineado con el reloj inyectable de Q-T-TMR-4.
- **Filas afectadas:** AT-SUP-007.1; AT-SUP-006
- **Unidades afectadas:** SUP
- **Origen:** typed-core

### Q-T-SYS-1 — Accesores JVM-only del sistema: threadFactory, dynamicAccess, printTree, logConfiguration

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:56-109, 179-185; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.scala:155-165. Los inventarios viejos proponian excluir; la exclusion no esta autorizada.
- **Alternativas:** (a) ThreadFactory -> std::thread::Builder/hook de creacion de hilos, dynamicAccess -> registro de fabricas (Q-X-2), printTree -> arbol textual propio, logConfiguration -> log de config redactada; (b) proponer exclusion formal al Owner.
- **Recomendación:** Clasificacion: divergencia funcional posible / exclusion de alcance no autorizada. Resuelto por lectura: logConfiguration y printTree delegan en el sistema clasico (ActorSystemAdapter.scala:108, 115; ActorSystem.scala clasico:920, 1264-1301) y printTree esta documentado como formato no estable; threadFactory/dynamicAccess siguen pendientes de la decision de runtime (Q-X-4, Q-X-2). Consecuencia observable: ninguna para log/logConfiguration; printTree y dynamicAccess cambian de forma. Bloquea AT-SYS-007.4; AT-SYS-007.3 tiene contrato cerrado y su implementacion puede avanzar con formato propio. | Recomendacion previa: (a); mantener la fila en alcance con la diferencia observable documentada.
- **Filas afectadas:** AT-SYS-007.3; AT-SYS-007.4
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-SYS-2 — Redaccion de configuracion en Settings.toString y logConfiguration

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:343-346; actor/src/main/scala/org/apache/pekko/util/Helpers.scala:183-192; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorSystemSpec.scala:30-46 (no filtrar user.name/user.home; Pekko PR 771).
- **Alternativas:** (a) misma regla: ocultar valores de variables de entorno y reemplazar user.name por <username>; (b) ocultar ademas claves con nombres sensibles (password, secret); (c) no redactar.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Resuelto por lectura: la regla es ocultar variables de entorno y reemplazar user.name (Helpers.scala:183-192); con user.name nulo/vacio replace(null) lanzaria NPE en la JVM (caso no afirmado). Consecuencia observable: los valores sensibles no aparecen en logs. No bloquea. | Recomendacion previa: (a) como minimo; (b) opcional como mejora marcada.
- **Filas afectadas:** AT-SYS-012; AT-SYS-006
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-SYS-3 — Buffer de arranque del guardian: limite de 1000 y StashOverflowException

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/GuardianStartupBehavior.scala:35, 49-51; ActorSystem.scala:319-322 envia Start antes de devolver el sistema.
- **Alternativas:** (a) construir el sistema completo antes de arrancar el guardian y omitir el buffer; (b) reproducir el limite de 1000 y el error; (c) buffer sin limite.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Resuelto por lectura: ActorSystem.scala:319-322 envia Start antes de devolver el sistema, por lo que el buffer de 1000 solo se usa si otro emisor encola antes de Start (por ejemplo via el sistema clasico); en un diseño nativo (a) el buffer se vuelve innecesario sin cambiar lo observable salvo StashOverflowException con >1000 mensajes tempranos. Bloquea solo la rama de overflow de AT-SYS-003. | Recomendacion previa: (a) manteniendo el contrato: ningun mensaje temprano se pierde; si se conserva buffer, reproducir (b).
- **Filas afectadas:** AT-SYS-003
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-SYS-4 — close() bloqueante y Drop del ActorSystem

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:148-166 (close desde 1.3.0 bloquea hasta terminar o TimeoutException); actor-typed-tests/src/test/scala-2.13+/org/apache/pekko/actor/typed/ActorSystemScala213PlusSpec.scala:28-34; actor-typed-tests/src/test/java/org/apache/pekko/actor/typed/ActorSystemTest.java:46-53.
- **Alternativas:** (a) close() bloqueante explicito y Drop que dispara terminate sin bloquear; (b) Drop bloqueante con runtime.block_on (riesgo de deadlock en async); (c) solo terminate().await.
- **Recomendación:** Clasificacion: divergencia funcional (Drop). Consecuencia observable: close() bloquea y puede lanzar TimeoutException; Drop no puede bloquear en async. Bloquea AT-SYS-005 (parte close); terminate/whenTerminated avanzan. | Recomendacion previa: (a). Documentar que try-with-resources/Using equivale a close() explicito.
- **Filas afectadas:** AT-SYS-005
- **Unidades afectadas:** SYS
- **Origen:** typed-core

### Q-T-TK-1 — API de TestProbe/ActorTestKit: bloqueante vs async

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestProbeImpl.scala:160-252; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTestKit.scala:238-272 | Cierre documental 2026-10-31: Leidos TestProbeImpl.scala y ActorTestKit.scala: los mensajes producidos con Predef.assert llevan el prefijo "assertion failed: " (stop de ActorTestKit, expectTerminated, within, TestInbox.expectMessage, expectEffect, childInbox) y los de assertFail/AssertionError no (expectMessage, receiveMessage, fish, ReplyInbox); expectNoMessage(max), el min de within, el intervalo de awaitAssert y ManualTime.timePasses no se dilatan con timefactor (AT-TK-002.6).
- **Alternativas:** (a) async fn sobre tokio con timeouts dilatados; (b) bloqueante sobre runtime multi-hilo; (c) ambas; ademas, para los mensajes de error: (i) conservar el prefijo "assertion failed: " donde Pekko lo produce, (ii) normalizar sin prefijo
- **Recomendación:** [Arquitectura; depende de Q-X-4 + representacion de mensajes] (a) o (c); mensajes de error conservados con (i): el texto es el contrato observable que fijan los specs. Consecuencias observables: API de TestProbe no bloqueante; texto de fallos identico. Bloquea: nombrar la API de TestProbe e impl de AT-TK-001.*/002.* hasta Q-X-4. Puede avanzar: AT-TK-004.*, 005.*, 006.1/006.2 y los contratos. [Recomendacion previa: (a) o (c); depende de Q-X-4; los mensajes de AssertionError se conservan]
- **Filas afectadas:** AT-TK-001.2;AT-TK-001.3;AT-TK-002.1;AT-TK-002.2;AT-TK-002.3;AT-TK-002.4;AT-TK-002.5;AT-TK-002.6;AT-TK-003.3;AT-TK-005.1
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-2 — Captura de logs (LoggingTestKit) sin Logback

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/LoggingTestKitImpl.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestAppender.scala | Cierre documental 2026-10-31: Efecto observable fijado en AT-TK-006.4: un appender global captura eventos con nivel, mensaje, logger, MDC, causa, marcador y thread; los filtros se registran por sistema y un evento solo cuenta para el filtro mas reciente que coincide (filters.exists). Limites: causa por tipo exacto en vez de subclase y jerarquia de nombres de logger por "::".
- **Alternativas:** (a) tracing Layer de captura; (b) log crate con appender de prueba
- **Recomendación:** [Representacion que conserva el contrato, con limites] (a) tracing Layer global + registro de filtros (ver AT-TK-006.4). Consecuencias observables: mismos criterios de coincidencia; sin Logback no hay appender XML. Los limites (causa por tipo exacto, jerarquia de nombres) requieren al Owner solo si se aceptan como diferencia. Depende de Q-X-7 y Q-T-LOG-1. Bloquea: AT-TK-006.*, AT-TK-013 (impl). Puede avanzar: contrato. [Recomendacion previa: (a); depende de Q-T-LOG-1]
- **Filas afectadas:** AT-TK-006.1;AT-TK-006.2;AT-TK-006.3;AT-TK-006.4;AT-TK-006.5;AT-TK-013
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-3 — Integraciones JUnit/ScalaTest y nombre del sistema por stack

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ScalaTestWithActorTestKit.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/TestKitUtils.scala:82-91 | Cierre documental 2026-10-31: Resuelta por lectura la parte del nombre del sistema: TestKitUtils.scala:82-91 usa testNameFromCallStack (pila JVM); en Rust se usa nombre explicito o module_path!/type_name (los hilos de libtest llevan el nombre del test).
- **Alternativas:** (a) helper/macro propio de Rust; (b) no portar JUnit (propuesta de exclusion, requiere autorizacion)
- **Recomendación:** [Exclusion de alcance con sustituto + incertidumbre resuelta] (a) helper/macro/fixture RAII propios de Rust; JUnit4/JUnit5/ScalaTest no se portan (exclusion que requiere autorizacion del Owner, no concedida). Consecuencias observables: sin TestKitJunitResource ni extension JUnit5; el sistema se crea y apaga por fixture. Bloquea: AT-TK-009.1-009.3 (impl). Puede avanzar: contratos y 009.4 (SerializationTestKit, ver Q-T-SER-1). [Recomendacion previa: (a)]
- **Filas afectadas:** AT-TK-001.1;AT-TK-009.1;AT-TK-009.2;AT-TK-009.3;AT-TK-009.4;AT-TK-013
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-4 — BehaviorTestKit requiere interprete de Behavior sin runtime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/BehaviorTestKitImpl.scala; actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/internal/StubbedActorContext.scala | Cierre documental 2026-10-31: BehaviorTestKitImpl.scala:170-185 usa Behavior.interpretMessage/interpretSignal/canonicalize directamente: el interprete puro es lo que necesita el testkit, no hace falta runtime. La igualdad de Behavior (efecto Spawned) requiere identidad en Rust.
- **Alternativas:** (a) interprete puro de Behavior reutilizado por el runtime (necesario con Q-X-8 b); (b) montar sobre el runtime con un ejecutor controlado
- **Recomendación:** [Arquitectura; depende de Q-X-8] (a) interprete puro de Behavior reutilizado por el runtime. Bloquea: AT-TK-003.*, 004.*, 005.*, 011, 012 (impl) hasta Q-X-8. Puede avanzar: contrato. [Recomendacion previa: (a)]
- **Filas afectadas:** AT-TK-003.1;AT-TK-003.2;AT-TK-003.3;AT-TK-003.4;AT-TK-003.5;AT-TK-003.6;AT-TK-003.7;AT-TK-004.1;AT-TK-004.2;AT-TK-004.3;AT-TK-004.4;AT-TK-004.5;AT-TK-004.6;AT-TK-004.7;AT-TK-005.1;AT-TK-005.2;AT-TK-005.3;AT-TK-011;AT-TK-012
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TK-5 — cancelAll del TimerScheduler de BehaviorTestKit es un no-op (paridad bug-compatible o correccion)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** EffectfulActorContext.scala:191 (`override def cancelAll(): Unit = activeTimers.foreach(cancel)`): al ser `activeTimers` un Map[Any, TimerScheduled], `foreach` pasa la tupla (clave, efecto) a `cancel(key: Any)`; `activeTimers.keySet(tupla)` es falso (185), por lo que no cancela nada ni registra TimerCancelled. Ningun test lo ejercita (AT-TK-004.6). Incertidumbre resuelta por lectura de la semantica de Scala; falta confirmar con el jar oficial (no esta en tools/oracle/fetch-jars.sh).
- **Alternativas:** (a) paridad: cancelAll no cancela ni registra efectos; (b) corregirlo para que cancele todos los timers activos y registre TimerCancelled por cada uno
- **Recomendación:** [Divergencia funcional si (b)] (a): paridad documentada. Consecuencias observables: un test de BehaviorTestKit que use cancelAll no ve TimerCancelled ni deja de ver el timer activo. Bloquea: nada. Puede avanzar: AT-TK-004.6 con la caracterizacion; cualquier correccion es divergencia y requiere al Owner.
- **Filas afectadas:** AT-TK-004.6
- **Unidades afectadas:** TK
- **Origen:** typed-extended

### Q-T-TMR-1 — Tipo de la clave del timer (Any en Scala)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:50-258 (key: Any); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:98 (Map[Any, Timer]).
- **Alternativas:** (a) TimerScheduler<M, K: Hash+Eq+Clone+Send>; (b) clave Box<dyn Any> con TypeId+Eq dinamico; (c) enum de claves por actor.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Consecuencia observable: claves tipadas en lugar de Any. Bloquea la firma de TimerScheduler (AT-TMR-001/002); la logica de generaciones avanza. | Recomendacion previa: (a); las variantes sin clave usan el mensaje como clave (M: Hash+Eq) o un id generado.
- **Filas afectadas:** AT-TMR-001; AT-TMR-002; AT-BEH-020
- **Unidades afectadas:** TMR; BEH
- **Origen:** typed-core

### Q-T-TMR-2 — NotInfluenceReceiveTimeout: marker de clase vs flag explicito

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:126-130 (instanceOf); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorContextAdapter.scala:118-129.
- **Alternativas:** (a) flag no_influence_receive_timeout al iniciar el timer o trait NotInfluenceReceiveTimeout en M; (b) ignorar el marker.
- **Recomendación:** Clasificacion: representacion que conserva el contrato. Resuelto por lectura: la regla de no-reinicio la implementa el ActorCell clasico sobre cualquier mensaje con el marker (ReceiveTimeout.scala:40-42, 71-76; ReceiveTimeoutSpec.scala:151-297) y TimerSchedulerImpl solo propaga el marker del msg al TimerMsg (:126-130). Conserva el contrato un flag explicito o un metodo del mensaje. No bloquea: AT-TMR-007.2 ya tiene contrato cerrado; solo la firma depende de la respuesta. | Recomendacion previa: (a) flag explicito (el trait solo si M es un enum con metodo).
- **Filas afectadas:** AT-TMR-007.2; AT-CTX-008
- **Unidades afectadas:** TMR; CTX
- **Origen:** typed-core

### Q-T-TMR-3 — startPeriodicTimer deprecado: alias y semantica (la sobrecarga Java difiere de la Scala)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/TimerSchedulerImpl.scala:81-84 (Java: FixedDelay) vs 113-114 (Scala: FixedRate); actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/TimerScheduler.scala:224-229 (doc: semantica de AtFixedRate). No se puede excluir sin autorizacion.
- **Alternativas:** (a) alias deprecado con semantica fixed rate (Scaladoc y Scala); (b) alias con la semantica de la sobrecarga Java (fixed delay); (c) proponer exclusion formal.
- **Recomendación:** Clasificacion: divergencia funcional posible (bug de Pekko). Resuelto por lectura: Scala FixedRate (TimerSchedulerImpl.scala:113-114), Java FixedDelay (:81-84); el Scaladoc de TimerScheduler.scala:224-229 describe fixed rate. No bloquea AT-TMR-001..004; bloquea solo AT-TMR-005. | Recomendacion previa: (a) y documentar la divergencia Java como bug conocido de Pekko.
- **Filas afectadas:** AT-TMR-005
- **Unidades afectadas:** TMR
- **Origen:** typed-core

### Q-T-TMR-4 — Reloj y scheduler inyectables (ManualTime) para timers, backoff, limites y uptime

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** AGENTS.md/docs/inventory/actor-typed.md: necesidad de reloj inyectable; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala (TimingTest); actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:430-441; actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorSystem.scala:70-78.
- **Alternativas:** (a) trait Clock/Scheduler en el sistema con implementacion tokio y ManualTime; (b) tokio::time::pause().
- **Recomendación:** Clasificacion: arquitectura. Consecuencia observable: sin reloj inyectable los tests TimingTest (TimerSpec, ReceiveTimeoutSpec, ClassicSupervisingTypedSpec.expectNoMessage) son de tiempo real. Bloquea la validacion determinista, no la implementacion. | Recomendacion previa: (a) porque tambien sirve al grupo testkit.
- **Filas afectadas:** AT-TMR-003; AT-TMR-004; AT-SYS-009; AT-SYS-007.2; AT-SUP-006
- **Unidades afectadas:** TMR; SYS; SUP
- **Origen:** typed-core

### Q-TK-2 — TestActorRef/TestFSMRef y acceso a `underlyingActor` frente al modelo de propiedad de Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestActorRef.scala:28-263 y TestFSMRef.scala:22-147, TestActorRefSpec.scala:30-358 y TestFSMRefSpec.scala:20-105 leídos completos (cierre 2026-10-31). underlyingActor devuelve en Pekko una referencia compartida y mutable al actor (TestActorRef.scala:125-134), que los specs leen mientras la CTD ejecuta receive en otro hilo sin sincronización (TcpListenerSpec.scala:177, TcpConnectionSpec.scala:104, FSMActorSpec.scala:288). El acceso por referencia es el único modo de uso observado en specs (sólo lectura de campos); setState y los proxies de timers se ejecutan fuera del receive.
- **Alternativas:** (a) `with_actor(|a| ..)` por cierre y `underlying_actor()` como guarda `!Send` sobre el candado reentrante de la CTD (estado en celda protegida, acceso serializado); (b) sólo `receive` síncrono sin acceso al actor (no porta TcpListenerSpec/TcpConnectionSpec/FSMActorSpec/TestActorRefSpec A7); (c) `Arc<Mutex<A>>` expuesto sin candado reentrante (interbloqueo al enviar mensajes con la guarda viva); (d) sin equivalente (exclusión: requiere autorización)
- **Recomendación:** [Clasificación: arquitectura] (a). Consecuencias observables: el acceso espera si otro hilo está ejecutando el actor (más estricto que Pekko); los tests deben soltar la guarda antes de enviar mensajes si quieren ver el estado modificado por ese mensaje; `state_name()/state_data()` devuelven clones. Bloquea: AC-TK-006.1 y AC-TK-007 en impl; los contratos están extraídos. Puede avanzar: AC-TK-006.2 (la CTD no depende de esta forma de acceso) y el resto del grupo.
- **Filas afectadas:** AC-TK-006.1 ; AC-TK-007
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-3 — EventFilter[T] por tipo de causa en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestEventListener.scala:324-336 (ErrorFilter usa throwable.isInstance(cause)); TestEventListenerSpec.scala:36-64.
- **Alternativas:** (a) filtrar por `TypeId` del error con downcast; (b) filtrar por predicado `Fn(&dyn Error) -> bool`; (c) ambos
- **Recomendación:** [Clasificación: representación que conserva el contrato] (c); EventFilter[NoCause] = cualquier Error. Consecuencias: EventFilter por TypeId requiere downcast del error; javadsl.EventFilter(clazz) comparte el mismo mecanismo (AC-TK-011). Bloquea: nada.
- **Filas afectadas:** AC-TK-005.1 ; AC-TK-011
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-4 — API bloqueante vs asíncrona de TestKit (awaitCond, within, expectMsg, timePasses)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestKit.scala:298-443 usa Thread.sleep y polling; ExplicitlyTriggeredScheduler.scala:67-81 duerme 100 ms reales.
- **Alternativas:** (a) API bloqueante (std::thread) igual que Pekko; (b) sólo async; (c) ambas, async como núcleo y bloqueante como envoltorio
- **Recomendación:** [Clasificación: arquitectura (modelo de ejecución de tests)] (c); relacionada con Q-X-4. Consecuencias: TestLatch/TestBarrier necesitan variante bloqueante y asíncrona (AC-TK-009). Bloquea: la API async de AC-TK-001.x hasta Q-X-4.
- **Filas afectadas:** AC-TK-001.1 ; AC-TK-001.2 ; AC-TK-001.5 ; AC-TK-005.2 ; AC-TK-008 ; AC-TK-013 ; AC-TK-009
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-5 — Origen del timefactor en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestKitExtension.scala:35-37 sólo lee `pekko.test.timefactor`; no hay variable de entorno (corrige el inventario). Cierre 2026-10-31: confirmado por lectura que TestKitExtension sólo lee la clave `pekko.test.timefactor` (testkit/src/main/resources/reference.conf:14); TestLatch/TestBarrier dilatan con TestTimeFactor (TestLatch.scala:51, TestBarrier.scala:44).
- **Alternativas:** (a) sólo clave de config; (b) además una variable de entorno `RAKKA_TEST_TIMEFACTOR` como extensión sin equivalente en Pekko
- **Recomendación:** [Clasificación: incertidumbre resuelta por lectura; la variable de entorno es una extensión opcional sin equivalente en Pekko] (a): sólo la clave de configuración. Una variable de entorno sería una extensión de rakka que requiere autorización del Owner. Bloquea: nada.
- **Filas afectadas:** AC-TK-004 ; AC-TK-009
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-6 — Harness de tests para sustituir PekkoSpec/Coroner/ScalaTest

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** PekkoSpec.scala:38-148; TestKitUtils.scala:20-70; Coroner.scala sin leer. Cierre 2026-10-31 (leídos Coroner.scala, CoronerSpec, WithLogCapturing, PekkoJUnitActorSystemResource, PekkoSpec): Coroner usa ThreadMXBean/JMX (volcado de hilos y detección de deadlocks de sincronizadores); WithLogCapturing imprime el log del test sólo si falla; PekkoJUnitActorSystemResource crea/apaga el sistema por test.
- **Alternativas:** (a) crate `rakka-testkit` con un `TestSystem` por test; (b) macros `#[rakka::test]`; (c) helpers en `tests/common`
- **Recomendación:** [Clasificación: arquitectura (harness de tests)] (a) con (b) opcional; definir nombres de sistema desde el nombre del test. Consecuencias: Coroner sin pilas de otros hilos (limitación de std; deadlocks con parking_lot opcional). Bloquea: nada.
- **Filas afectadas:** AC-TK-012.1 ; AC-TK-012.2 ; AC-TK-014 ; AC-TK-012.2
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-7 — Interrupciones de hilo en la CallingThreadDispatcher (Thread.interrupt) no tienen equivalente en Rust

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** CallingThreadDispatcher.scala:258-338 captura, loguea y relanza InterruptedException; ActorModelSpec.scala:422-455 y 457-477 lo ejercen (ActorInterruptedException con causa "Ping!"; interrupción sin excepción).
- **Alternativas:** (a) excluir esos dos casos con justificación (JVM-específico) y portar el resto; (b) emular con un indicador de cancelación cooperativo por hilo (`CancelToken`) y adaptar los tests; (c) sin CTD multihilo
- **Recomendación:** [Clasificación: exclusión de alcance parcial] (a), salvo que el Owner pida (b). Consecuencias: dos casos de ActorModelSpec no portados 1:1. Bloquea: nada para el resto de AC-TK-006.2.
- **Filas afectadas:** AC-TK-006.2
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-8 — Manejadores asíncronos ejecutados en el hilo del emisor (CallingThreadDispatcher)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** En Pekko receive es síncrono, de modo que ejecutarlo en el hilo del emisor es trivial (CallingThreadDispatcher.scala:283-291). Si el manejador de rakka es `async fn`, ejecutarlo "en el hilo del emisor" exige un block_on sin reactor o un executor local.
- **Alternativas:** (a) CTD sólo para manejadores síncronos o async sin reactor (block_on mínimo); error explícito si el manejador necesita el reactor; (b) CTD con un runtime current_thread privado por llamada; (c) no ofrecer CTD y sustituirla por esperas
- **Recomendación:** [Clasificación: arquitectura (depende de Q-X-3/Q-X-4)] (a). Consecuencias: actores de prueba que hagan I/O de Tokio dentro del receive no pueden correr sobre la CTD. Bloquea: AC-TK-006.1, AC-TK-006.2, AC-TK-007 en impl, hasta definir el trait Receive y el runtime.
- **Filas afectadas:** AC-TK-006.1 ; AC-TK-006.2 ; AC-TK-007
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-9 — Límites del Coroner y exclusión de metrics/* del testkit

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Coroner.scala:37-254 usa ManagementFactory (ThreadMXBean) para volcar hilos y detectar deadlocks; metrics/* (MetricsKit, HdrHistogram…) es private[pekko], su Scaladoc lo descarta como micro-benchmarking (MetricsKit.scala:25-32) y sólo lo consume ActorCreationPerfSpec (actor-tests/.../ActorCreationPerfSpec.scala:131).
- **Alternativas:** (a) Coroner con informe reducido (hilos vivos y deadlocks vía parking_lot) y sin metrics/* (benchmarks con criterion fuera del contrato); (b) portar metrics/* con crates equivalentes; (c) sin Coroner
- **Recomendación:** [Clasificación: exclusión de alcance parcial] (a); la exclusión de metrics/* y de la parte JMX del informe del Coroner requiere confirmación del Owner. Bloquea: nada.
- **Filas afectadas:** AC-TK-012.2
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-TK-10 — Serializador de pruebas java-test (id 23) sin serialización Java

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** TestJavaSerializer.scala:30-52 usa ObjectOutputStream/ObjectInputStream; reference.conf del testkit:36-49 registra java-test, id 23 y binding JavaSerializable. Sin spec propio.
- **Alternativas:** (a) serializador de pruebas con el mismo id/nombre/binding por marcador y formato binario estándar (bincode/postcard); (b) no registrar nada; (c) exclusión
- **Recomendación:** [Clasificación: divergencia funcional (formato de bytes)] (a). Consecuencias: bytes no interoperables con Pekko JVM; contrato de selección por binding conservado. Bloquea: nada; depende de la política de serialización de AC-SER.
- **Filas afectadas:** AC-TK-015
- **Unidades afectadas:** TK
- **Origen:** classic-system

### Q-UTIL-18 — Zona horaria del sistema para Helpers.timestamp

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Helpers.scala:108-113: `timestamp(ms)` formatea con `ZoneId.systemDefault()` capturada al cargar Helpers; consumidor: StdOutLogger.timestamp (Logging.scala:1009). La biblioteca estándar de Rust no conoce zonas horarias (ni sus reglas de horario de verano). Medido en 2.0.0 (TS-LOCAL-01/02, zona fijada en GMT+05:30): el patrón y el cálculo de fechas ya coinciden para un desplazamiento fijo (`timestamp_at_utc_offset`).
- **Alternativas:** (a) añadir una dependencia con base de zonas (candidatas: `jiff`, o `chrono` con `iana-time-zone`; a verificar con `cargo deny check licenses`); (b) sin dependencia: `timestamp` recibe el desplazamiento respecto de UTC y quien lo llama lo decide (el logger usa UTC por defecto); (c) FFI a `localtime_r`/`GetTimeZoneInformation` (código `unsafe` por plataforma).
- **Recomendación:** (a) una dependencia mantenida y con licencia compatible; con (b) la salida de log no coincide con la de Pekko fuera de UTC. Consecuencia observable: sólo el texto de la hora en la salida estándar del logger.
- **Filas afectadas:** AC-UTIL-025
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-19 — fastHash y copyUSAsciiStrToBytes con texto no ASCII dependen del algoritmo que elija la JVM

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Unsafe.java:73-146: el algoritmo 1 (acceso reflexivo a `String.value`, que exige `--add-opens java.base/java.lang`) usa los bytes Latin-1 con extensión de signo; el algoritmo 0 usa `getBytes(US_ASCII)` (`?` por cada carácter no ASCII, y un par sustituto da un solo `?`, de modo que `fastHash` lanza ArrayIndexOutOfBoundsException). Medido en 2.0.0 con JDK 17 sin `--add-opens` (UN-03, sin grabar): `é` y `€` -> 1727796305, `aéb` -> 967922898, U+1F600 -> ArrayIndexOutOfBoundsException. Ningún consumidor de producción en actor/src/main ni actor-typed/src/main (sólo AsciiStringCopySpec).
- **Alternativas:** (a) reproducir el algoritmo 0 (el que corre sin opciones de JVM); (b) reproducir el algoritmo 1 (Latin-1 con signo; los caracteres > U+00FF dependen de la representación interna); (c) aceptar sólo texto ASCII y devolver `None` para el resto (lo implementado).
- **Recomendación:** (c): la función no tiene consumidores y los dos algoritmos discrepan entre sí.
- **Filas afectadas:** AC-UTIL-020
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-20 — Cierre de las filas de equivalencia (OptionVal, ConstantFun, Collections, Unused, anotaciones de API) sin tipo ni oraculo propios

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** util/OptionVal.scala:23-94, util/ConstantFun.scala:26-91, util/Collections.scala:22-94 y las cuatro anotaciones de java/org/apache/pekko/annotation: en 2.0.0 siguen sin test propio salvo ConstantFunJavaTest (A1/A2: identidad por referencia, inobservable en Rust); util/Unused.scala no existe en 2.0.0; Collections.EmptyImmutableSeq.iterator ya es Iterator.empty de Scala (el mensaje 'next on empty iterator' es de la biblioteca de Scala y lo observa ByteIterator, P04) y 2.0.0 anade IterableOps.collectToImmutableSeq (sin consumidores en main).
- **Alternativas:** (a) cerrar OptionVal, ConstantFun, Unused y las anotaciones como `done` con equivalente documentado (Option<T>, closures/std::convert::identity, prefijo `_`, visibilidad/traits sellados/cargo-semver-checks) y sin oraculo, y cerrar Collections junto con ByteIterator (P04); (b) escribir tipos propios (OptionVal, constantes de ConstantFun, EmptyImmutableSeq) y caracterizarlos con oraculo aunque no tengan consumidor; (c) excluir alguna fila (requiere autorizacion explicita).
- **Recomendación:** (a): Q-UTIL-11 (a) ya acepta el equivalente documentado; falta confirmar que ello basta para cerrar `impl`/`validation` sin casos grabados porque las aserciones A# son 'no afirmadas por spec' o inobservables en Rust.
- **Filas afectadas:** AC-UTIL-016.1, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-016.18, AC-UTIL-019
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-21 — Selección entre entradas `prefix*` solapadas de WildcardIndex (orden del HashMap de Scala)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** WildcardIndex.scala:87-89 (`wildcardSuffixChildren.collectFirst { case (key, branch) if nextElement.startsWith(key) => branch }` sobre un `HashMap[String, WildcardTree]`): con dos prefijos que ambos son prefijo del elemento (p. ej. `a*` y `ab*` para `abc`) gana el que el HashMap de Scala itera primero, que depende de `String.hashCode` y de la estructura del trie; ningún spec lo afirma (AC-UTIL-009 A7).
- **Alternativas:** (a) gana el prefijo más largo (el más específico), y a igual longitud el insertado primero: determinista; (b) gana el primero insertado (lo que hay hoy); (c) reproducir el orden de iteración del HashMap de Scala (String.hashCode y orden del CHAMP trie).
- **Recomendación:** (a): es el único criterio que un usuario puede predecir y coincide con "la entrada más específica" que el contrato de AC-UTIL-009 promete; sólo difiere de Pekko en configuraciones solapadas, donde Pekko depende del hash. Clasificación: representación (diferencia observable sólo en solapamientos). Bloquea: AC-UTIL-009 A7; puede avanzar: el resto de la fila.
- **Filas afectadas:** AC-UTIL-009
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-22 — Charsets y Base64 de ByteString sin dependencias externas

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ByteString.scala:207-215,630-640 y companion (`fromString(s, charset)`, `decodeString(charset)`, `encodeBase64/decodeBase64`); el oráculo (BS-18..BS-22 y BS-31..BS-33: uc-0471..uc-0475, uc-0484..uc-0486) acepta cualquier charset de la JVM por nombre (UTF-16, windows-1252, ...) y alias (`latin1`, `utf8`, `ascii`...). Q-UTIL-12 (c) dejó abierta la lista de charsets y la dependencia `base64`/`encoding_rs`.
- **Alternativas:** (a) UTF-8, US-ASCII e ISO-8859-1 con los alias medidos y Base64 propio (std no lo trae): sin dependencias nuevas; el resto de nombres falla con UnsupportedEncodingException; (b) añadir `encoding_rs` (y su tabla de alias) para el resto de charsets; (c) además `base64` en lugar del decodificador propio.
- **Recomendación:** (a): cubre todo lo que usa Pekko en main (UTF-8) y los tests; un charset exótico (UTF-16, windows-1252) falla con `UnsupportedEncodingException` hasta que un consumidor (IO/SER) lo pida. El Base64 propio reproduce los mensajes de error de la JVM (BS-22 y BS-32: 60 sopas de texto grabadas, más una corrida diferencial ad hoc de 900 entradas sin divergencias). Clasificación: dependencia externa. Bloquea: nada observable de lo ya implementado; AC-UTIL-006.6 queda `partial` (charsets).
- **Filas afectadas:** AC-UTIL-006.6
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-23 — Excepción de índice de ByteString/ByteIterator que depende de la representación

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ByteString.scala:1.7.1 168-307 / 2.0.0 ByteString1C.apply: `apply(idx)` sobre un ByteString1C indexa el array y lanza ArrayIndexOutOfBoundsException con el texto de la JVM (`Index 3 out of bounds for length 3`, caso BS-APPLY-1C); sobre ByteString1/ByteStrings lanza IndexOutOfBoundsException(idx) (BS-08: `3`, `-1`). `head`/`last` del vacío (ByteString1C.empty) lanzan la primera (BS-HEAD-EMPTY); `ByteIterator.head` agotado lanza `Index n out of bounds for length n` (IT-01 medido). Con `bytes::Bytes` no hay representación que elegir.
- **Alternativas:** (a) `apply`/`head`/`last` fallan siempre con `IndexOutOfBoundsException` cuyo mensaje es el índice (el texto de ByteString1/ByteStrings); `ByteIterator::head` devuelve `Option`; (b) reproducir el texto de la JVM del array compacto cuando el valor es un único fragmento completo; (c) devolver `Option<u8>` sin excepción.
- **Recomendación:** (a): la clase y el texto de ByteString1/ByteStrings son los que un programa ve con valores recortados o concatenados; el texto de la JVM del array compacto depende del tamaño del array subyacente, que `Bytes` no expone. Clasificación: representación. Bloquea: AC-UTIL-006.1 (`partial`: mensaje de `apply`/`head`/`last` sobre un valor compacto y del vacío).
- **Filas afectadas:** AC-UTIL-006.1
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-24 — ByteIterator: `++` entre arrays no contiguos y lecturas cortas

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ByteIterator.scala:61-80 (ByteArrayIterator.++): para dos ByteArrayIterator sobre arrays distintos construye `MultiByteArrayIterator(List(this, bai))` y después `this.clear()`: el iterador resultante conserva sólo el lado derecho (medido IT-CAT: `iter 0102 ++ 0304` -> len 2, `<0304>`; con ByteArrayIterator contiguos del mismo array sí concatena). Lecturas cortas: `getShort/getInt/getLong` sobre ByteArrayIterator no consumen nada y lanzan `next on empty iterator` (IT-09: len sigue en 2), pero MultiByteArrayIterator consume lo disponible antes de fallar (medido `iterrope x01 x02; getint` -> len 0); `getLongPart` consume siempre. 2.0.0 añade `getShort/Int/Long` por SWARUtil (ByteIterator.scala:136-160); en 1.7.1 todos consumían (contrato 006.4 desactualizado).
- **Alternativas:** (a) `++` concatena correctamente (sin pérdida del lado izquierdo) y las lecturas de primitivas cortas no consumen; (b) reproducir la pérdida del lado izquierdo y la diferencia según la representación; (c) no exponer `++` entre iteradores.
- **Recomendación:** (a) para las lecturas (ya implementado: atómicas, como ByteArrayIterator) y para `++`: pérdida de datos es un defecto de Pekko 2.0.0, no un contrato. Hasta que el Owner responda no se implementa `++` entre iteradores (AC-UTIL-006.4 queda `partial`). Clasificación: defecto de Pekko + representación.
- **Filas afectadas:** AC-UTIL-006.4
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-25 — ByteBufferCleaner (liberación explícita de buffers directos)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:17-123 (clase privada del paquete: `clean`, `isSupported`); sin oráculo posible desde fuera del paquete; ByteBufferCleanerSpec.scala:25-34. Consumidores: AC-IO (buffer pools de TCP/UDP), aún sin implementar. En Rust un `Bytes`/`Vec` se libera por `Drop`.
- **Alternativas:** (a) no portar la clase: la liberación determinista de `Drop` cumple el contrato observable (el buffer se libera), `isSupported` sería siempre `true`; requiere que el Owner autorice la exclusión de esta parte de la fila; (b) portar una API `clean`/`is_supported` vacía; (c) esperar a AC-IO.
- **Recomendación:** (a): ninguna API nueva; Q-UTIL-11 no autorizó esta exclusión (sólo 016.7, 016.12, 016.13). AC-UTIL-022 queda `partial` (UnsynchronizedByteArrayInputStream hecho y comparado con el oráculo; falta resolver el cleaner). Clasificación: lenguaje (Drop).
- **Filas afectadas:** AC-UTIL-022
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-26 — Serialización de ByteString (proxy 0/1/2 y la identidad 3 nueva en 2.0.0)

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** ByteString.scala (2.0.0) `SerializationProxy` :2146-2170 y `Companion` :2164-2175: identidades ByteString1=0, ByteString1C=1, ByteStrings=2 y, nueva en 2.0.0, ByteString2=3 (ByteString.scala:1071); `writeObject` escribe el byte de identidad y `writeToOutputStream` (i32 BE de longitud + bytes por fragmento); `readFromInputStream` lanza `Invalid serialization id <n>` para otro valor. Q-X-3 (A) y Q-CFG-11 aceptadas: la serialización Java no existe hasta AC-SER. Ningún caso grabable: ObjectOutputStream no tiene equivalente.
- **Alternativas:** (a) no implementar nada hasta AC-SER; cuando exista, un formato propio (identidad `u8` 0..3, cuerpo i32 BE + bytes por fragmento) con los mismos tests de ida y vuelta; (b) implementar ya un `to_wire`/`from_wire` con ese formato; (c) excluir la fila (la serialización Java no tiene equivalente).
- **Recomendación:** (a): ningún efecto observable hoy; el formato de red de los mensajes lo decide AC-SER. AC-UTIL-006.8 queda `partial`. Clasificación: arquitectura (depende de AC-SER).
- **Filas afectadas:** AC-UTIL-006.8
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-UTIL-27 — Cierre de las filas de frontera Java/Scala (FutureConverters, OptionConverters/FunctionConverters/JavaDurationConverters, ccompat) eliminadas en 2.0.0

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** 2.0.0 elimina util/FutureConverters, util/OptionConverters, util/FunctionConverters, util/JavaDurationConverters, util/ccompat y compat/Future (actor/src/main/mima-filters/2.0.x.backwards.excludes: future-converters, javaapi-option, function-converters, java-duration-converters, remove-scala-2.12-support); lo que queda es scala.jdk.* de la biblioteca de Scala (actor/src/main/java/org/apache/pekko/util/OptionalUtil.java:21,38). Único comportamiento propio observable: Timeout.create(java.time.Duration), medido por los casos grabados uc-0537..uc-0549 y TO-NEG-04/05, ya correcto en Rust. Timeout -> java.time.Duration (toJava) lo usan Actor.scala:158, AbstractActor.scala:173, CachePolicy.scala:33 y OptimalSizeExploringResizer.scala:273 (otras unidades).
- **Alternativas:** (a) cerrar 018.1, 018.2 y 018.3 como `done` con el equivalente documentado (un único `Future`, `Option<T>`, closures, `TryFrom<std::time::Duration> for Timeout`) y marcadas `eliminado en 2.0.0`, sin tipo propio; la conversión inversa Timeout -> duración Java se resuelve en las filas consumidoras (JAPI/ACT/DNS/RTG); (b) escribir tipos propios (conversores) y caracterizarlos con el oráculo aunque Pekko 2.0.0 ya no los tenga; (c) excluir alguna fila (requiere autorización explícita).
- **Recomendación:** (a): Q-UTIL-11 (a) y Q-X-4 (A) ya aceptan el equivalente y el Owner marcó ccompat como `eliminado en 2.0.0`; falta confirmar que ello cierra `impl`/`validation` de las tres filas sin conversores propios (las aserciones A1/A2 son pruebas de compilación o delegan en la biblioteca de Scala).
- **Filas afectadas:** AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation

### Q-X-19 — Fijar el oráculo en JDK 21 (o 25 LTS) y regrabar todo el corpus, para dejar de mezclar JDK 17 y 21

- **Estado:** propuesta (sin decisión del Owner)
- **Evidencia:** Q-X-16 (aceptada) fija el oráculo en JDK 17, pero #266 regrabó con JDK 21.0.13.1 los 77 casos NUM-RENDER (ch-0497..0528) y NUM-SHORT (ch-0778..0822): 70 salieron idénticos y 7 distintos (los antiguos NA:Q-CFG-30), así que el corpus mezcla dos JDK. Q-CFG-30: la representación más corta de un double cambió en JDK 19+; Rakka coincide con JDK 19+ (0 diferencias sobre 380 849 dobles con JDK 27, 5 con JDK 17) y el clasificador NA:Q-CFG-30 existe solo para esa diferencia. Hoy `oracle/run.sh` usa JDK 17 y los casos nuevos de P09 no dependen de la JDK (uc-0553 da el mismo stdout y stderr con JDK 17 y JDK 27). JDK 21 no está instalado en la máquina donde se midió; la disponibilidad de JDK 25 no se ha comprobado.
- **Alternativas:** (a) Seguir en JDK 17 (Q-X-16) con excepciones por familia: hoy NUM-RENDER y NUM-SHORT en JDK 21. (b) Pasar el oráculo y todo el corpus a JDK 21 y regrabarlo. (c) Pasar a JDK 25 LTS y regrabarlo.
- **Recomendación:** (b), sin regrabar nada hasta que el Owner responda. Clasificación: infraestructura del oráculo; no cambia el contrato de Pekko 2.0.0, pero puede cambiar los esperados de los casos que dependen de la JDK (p. ej. el texto de excepciones del JDK y el render de doubles). Bloquea: nada hoy; deja de mezclar JDK 17 y 21 y elimina la excepción de #266 y el clasificador NA:Q-CFG-30 si se elige (b) o (c). Reversible.
- **Filas afectadas:** todas las filas con casos grabados
- **Unidades afectadas:** todas
- **Origen:** classic-foundation

## 6. Respuestas del Owner por unidad

Decisiones por unidad que el Owner ya resolvió (las transversales figuran en la sección 4).

### Q-CFG-3 — checkValid contra reference.conf (Typesafe Config) con un parser propio

- **Estado:** aceptada
- **Evidencia:** ActorSystem.scala:395-402: `cfg.checkValid(defaultReference.withoutPath("pekko.actor.internal-dispatcher"), "pekko")`. Las reglas (compatibilidad de tipos, listas, objetos, mensaje agregado `ValidationFailed`) pertenecen a Typesafe Config 1.4.9, que no está en upstream/pekko; ningún test de Pekko las afirma.
- **Alternativas:** (a) implementar checkValid con tabla de compatibilidad documentada y escenarios del jar; (b) omitirlo y fallar al leer cada clave; (c) reducirlo a presencia de claves.
- **Recomendación:** (a) si Q-X-1 elige parser propio; con (b)/(c) se pierde 'fail-fast con lista de errores'. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA (depende de Q-X-1). Consecuencias observables: sin checkValid un sistema arranca con configuración mal tipada y falla más tarde, clave a clave, sin la lista agregada `ValidationFailed`. Bloquea: AC-CFG-019 (y la parte fail-fast de AC-CFG-002). Puede avanzar: lectura de cada clave en Settings.
- **Filas afectadas:** AC-CFG-019, AC-CFG-002
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a): se implementa checkValid porque Q-X-1 es parser propio. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-4 — Fidelidad de errores y render de Config (tipos y textos de ConfigException, renderWithRedactions)

- **Estado:** aceptada
- **Evidencia:** DeployerSpec.scala:144-161 exige ConfigException.WrongType; el golden config-hocon.golden fija mensajes (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR); Helpers.scala:178-192 renderiza con ConfigRenderOptions.defaults (formato propio de Typesafe).
- **Alternativas:** (a) enum de errores con las mismas categorías y mensajes del golden; (b) mismas categorías, mensajes propios; (c) textos exactos sólo donde un test de Pekko los afirme.
- **Recomendación:** (a) para los mensajes del golden; render sin exigir igualdad textual. [Cierre documental 2026-10-31] Clasificación: representación (errores) con riesgo de divergencia de mensajes. Consecuencias observables: `ConfigException.WrongType` exigido por DeployerSpec.scala:144-161; mensajes del golden. Bloquea: AC-CFG-017.* (golden) y AC-UTIL-002.4/.5 (impl); puede avanzar: AC-CFG-002/005 con tipos de error propios.
- **Filas afectadas:** AC-CFG-002, 005, 012.1, 017.*, AC-UTIL-002.4, AC-UTIL-002.5
- **Unidades afectadas:** CFG, UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a). Los mensajes de error de Config 1.4.9 son contrato. Condición: volver a grabar el golden de HOCON con el CLI del oráculo (jshell está deprecado). (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-5 — Valor de `pekko.version` y de `ActorSystem.Version` en rakka

- **Estado:** aceptada con cambios
- **Evidencia:** reference.conf:10-11 (`include "version"`), project/VersionGenerator.scala:18-25 (genera version.conf y Version.scala), ActorSystem.scala:485-487 (error si difieren), ConfigSpec.scala:41-42.
- **Alternativas:** (a) la versión de Pekko portada ("1.7.1"); (b) la versión del crate (CARGO_PKG_VERSION); (c) ambas: `pekko.version` fijo en 1.7.1 y una constante aparte del crate.
- **Recomendación:** (a) mientras la fuente de verdad sea Pekko 1.7.1; así `PekkoVersion.require` de librerías se comporta igual. [Cierre documental 2026-10-31] Clasificación: representación; incertidumbre NO resoluble con las fuentes (es una decisión de producto). Consecuencias observables: `ActorSystem.Version`, `pekko.version` y `PekkoVersion.require`. Bloquea: AC-CFG-003, AC-UTIL-005.1; puede avanzar: el resto.
- **Filas afectadas:** AC-CFG-003, AC-UTIL-005.1
- **Unidades afectadas:** CFG, UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a) con el valor "2.0.0" (la referencia pasó de 1.7.1 a 2.0.0, Q-X-11). (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-6 — Fuentes de la config de aplicación cuando no se pasa Config

- **Estado:** aceptada
- **Evidencia:** ActorSystem.scala:281 y docs/src/main/paradox/general/configuration.md:18-53: `ConfigFactory.load(cl)` = application.conf/json/properties del classpath + system properties + `config.resource`; reference.conf de cada jar.
- **Alternativas:** (a) application.conf del directorio de trabajo + variables de entorno con prefijo + reference.conf embebidos de las crates; (b) sólo reference.conf embebido y config explícita; (c) además un equivalente a `-Dkey=value` por línea de comandos.
- **Recomendación:** (a) con una ruta configurable; documentar que no hay classpath scanning. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA (fuentes de configuración). Consecuencias observables: cómo se localiza application.conf/variables; no cambia el contrato de merge (golden FB-*). Bloquea: AC-CFG-001 (carga por defecto); puede avanzar: `ActorSystem::new(config)` con config explícita.
- **Filas afectadas:** AC-CFG-001
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-7 — Claves de JVM: qué semántica tienen en Rust (daemonic, jvm-exit-on-fatal-error, jvm-shutdown-hooks, fail-mixed-versions, java-flight-recorder.enabled, home)

- **Estado:** aceptada
- **Evidencia:** reference.conf:14-15,85-110; ActorSystem.scala:463-472; CoordinatedShutdownSpec.scala:632-645 (conteo de hooks).
- **Alternativas:** (a) aceptar todas; mapear daemonic a hilos del runtime, jvm-exit a abort/exit, jvm-shutdown-hooks a manejadores de señales; ignorar con aviso fail-mixed-versions y java-flight-recorder; (b) rechazar las claves sin sentido.
- **Recomendación:** (a); la clave siempre se acepta para no romper configs de Pekko. [Cierre documental 2026-10-31] Clasificación: EXCLUSIÓN PARCIAL de comportamiento JVM con aceptación de claves (daemonic, jvm-exit-on-fatal-error, jvm-shutdown-hooks, fail-mixed-versions, java-flight-recorder.enabled, home). Evidencia añadida: la clave jvm-shutdown-hooks cuenta hooks en CoordinatedShutdownSpec.scala:632-645. Consecuencias observables: sólo el comportamiento al terminar el proceso. Bloquea: AC-CFG-007 y AC-UTIL-016.11 (impl); puede avanzar: aceptar y validar las claves.
- **Filas afectadas:** AC-CFG-007, AC-UTIL-016.11
- **Unidades afectadas:** CFG, UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-8 — Nombres estables para el registro de extensiones/fábricas (FQCN con `$` de objeto Scala)

- **Estado:** aceptada
- **Evidencia:** reference.conf:78 usa `org.apache.pekko.serialization.SerializationExtension$`; ReflectiveDynamicAccess.scala:69-87 resuelve `fqcn$` y luego `fqcn`; ExtensionSpec.scala:143 lista `InstanceCountingExtension` con y sin `$`.
- **Alternativas:** (a) registrar ambos nombres (con y sin `$`) para cada extensión; (b) normalizar quitando `$` al resolver; (c) claves propias de Rust y tabla de alias de los FQCN de Pekko.
- **Recomendación:** (b): mantiene el reference.conf sin cambios y los duplicados siguen creando una sola instancia. [Cierre documental 2026-10-31] Clasificación: representación (nombres estables). Consecuencias observables: reference.conf con `...SerializationExtension$` debe cargar la misma extensión una sola vez. Bloquea: AC-CFG-006 y AC-EXT-003; puede avanzar: AC-EXT-001.*, AC-EXT-002.
- **Filas afectadas:** AC-CFG-006, AC-EXT-003 [asociacion explicita por contrato, cierre P05: AC-UTIL-016.7]
- **Unidades afectadas:** CFG, EXT, UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-9 — Proveedor distinto de local (remote, cluster, FQCN) en un sistema solo-local

- **Estado:** aceptada
- **Evidencia:** ActorSystem.scala:107-145 y 410-420; remote/cluster están fuera del módulo; ProviderSelectionSpec.scala:38-67 sólo prueba la selección, no la carga de la clase.
- **Alternativas:** (a) `ProviderSelection` completo, pero crear el sistema con Remote/Cluster/Custom no registrado devuelve un error de configuración que nombra la clase; (b) rechazar `remote`/`cluster` al parsear Settings; (c) permitir proveedores registrados por el usuario (punto de extensión para remote).
- **Recomendación:** (a) + (c): deja la puerta abierta a remote sin implementarlo. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA (alcance de remote/cluster). Consecuencias observables: mensaje de error al pedir `remote`/`cluster`/FQCN en un sistema solo local. Bloquea: AC-CFG-008; puede avanzar: provider `local`.
- **Filas afectadas:** AC-CFG-008, AC-SYS-018
- **Unidades afectadas:** CFG, SYS
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a)+(c). (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-10 — pekko.use-slf4j / amendSlf4jConfig

- **Estado:** aceptada
- **Evidencia:** ActorSystem.scala:343-376, actor-typed reference.conf:68, ConfigSpec.scala:186-203. Depende de si los loggers son actores o un backend (Q-X-7).
- **Alternativas:** (a) no portar la función si Q-X-7 elige un backend de logging; (b) portarla con un 'logger' registrado como Slf4jLogger equivalente.
- **Recomendación:** Sigue a Q-X-7. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA (depende de Q-X-7). Consecuencias observables: `pekko.use-slf4j` agrega Slf4jLogger/Slf4jLoggingFilter. Bloquea: AC-CFG-018; puede avanzar: AC-CFG-004 sin esa clave.
- **Filas afectadas:** AC-CFG-004, AC-CFG-018
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-11 — serialize-messages / serialize-creators / no-serialization-verification-needed-class-prefix

- **Estado:** aceptada con cambios
- **Evidencia:** Dispatch.scala:171-223, Children.scala:283-307, SerializeSpec.scala:310-361, DispatchSpec.scala:29-41; en Rust los mensajes son `Box<dyn Any + Send>` (Q-X-3).
- **Alternativas:** (a) portar el chequeo sólo para tipos con serializador registrado; (b) no portar los dos modos y rechazar las claves en `on`; (c) portar con un trait `Verifiable` opt-in.
- **Recomendación:** (a) cuando exista AC-SER; mientras tanto las claves se aceptan en `off`. [Cierre documental 2026-10-31] Clasificación: DIVERGENCIA FUNCIONAL potencial (los mensajes en Rust son `Box<dyn Any + Send>`, Q-X-3) y dependencia de AC-SER. Consecuencias observables: verificación de serialización en `serialize-messages`/`serialize-creators` y el prefijo `no-serialization-verification-needed-class-prefix`. Bloquea: AC-CFG-010 (y AC-UTIL-006.8); puede avanzar: aceptar las claves en `off`.
- **Filas afectadas:** AC-CFG-010 [asociacion explicita por contrato, cierre P05: AC-UTIL-016.12]
- **Unidades afectadas:** CFG, SER, UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar las claves; con la opción en `on` el arranque falla con un error de configuración (no un no-op silencioso) hasta que exista AC-SER. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-12 — Deploy.withFallback descarta los tags de `this`

- **Estado:** aceptada
- **Evidencia:** Deployer.scala:109-117 crea el Deploy resultante con `Deploy(path, config..., mailbox)` sin pasar `tags`; withTags existe aparte (línea 119-120). Ningún test lo afirma.
- **Alternativas:** (a) reproducir (los tags no sobreviven al merge); (b) conservar los tags de `this` y registrar la diferencia.
- **Recomendación:** (a) hasta que un test o consumidor de tags (cluster) diga lo contrario. [Cierre documental 2026-10-31] Clasificación: incertidumbre RESUELTA parcialmente por lectura: Deploy.withFallback (Deployer.scala:109-117) no copia `tags`; en el árbol fijado los únicos usos de `tags` son `Props.withTags` (Props.scala:217) y `DaemonMsgCreateSerializer.scala:78-79,170` (remote, fuera de estos módulos); ningún test de actor-tests los toca. No se puede decidir con las fuentes del módulo. Consecuencias observables: tags de un Props que se mezcla con un Deploy de configuración. Bloquea: nada en pekko-actor; puede avanzar: AC-CFG-013 con (a).
- **Filas afectadas:** AC-CFG-013
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-13 — ActorSystemSetup: identidad por clase concreta del Setup

- **Estado:** aceptada
- **Evidencia:** ActorSystemSetup.scala:48-49,76-87 indexa por `getClass`; ActorSystemSetupSpec.scala:39-55.
- **Alternativas:** (a) `HashMap<TypeId, Arc<dyn Any + Send + Sync>>` con `TypeId` del tipo concreto; (b) trait `Setup` con `fn key() -> &'static str`.
- **Recomendación:** (a). [Cierre documental 2026-10-31] Clasificación: representación (identidad por clase concreta del Setup). Consecuencias observables: dos Setup del mismo tipo se reemplazan (ActorSystemSetupSpec.scala:39-45). Bloquea: nada; (a) es suficiente.
- **Filas afectadas:** AC-CFG-015
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-CFG-30 — Dígitos de Double.toString en el render de números (JDK 17 no siempre da la cadena más corta)

- **Estado:** aceptada
- **Evidencia:** D-BUG-01 (refinement-01): el render de un número muestra su valor (Typesafe: un double entero que cabe en long se muestra como long; el resto con Double.toString de Java). Rust usa los dígitos más cortos que identifican el double, con la regla de 2 cifras de Java para 1 dígito. Medido con 380 849 doubles aleatorios no enteros (semilla 42) contra JDK 17: 362 difieren. Medido después contra JDK 27 (semántica de JDK 19+, Javadoc de Double.toString): 357 son EMPATES EXACTOS (el valor está a mitad de camino entre dos candidatos de la misma longitud; JDK 17 y JDK 19+ eligen la última cifra par, `Display` de Rust la mayor) y eran un fallo de rakko, no de JDK; 5 son bugs de JDK 17 (JDK 19+ coincide con rakko; JDK-4511638), a los que se suma `2e23` (JDK 17 imprime 1.9999999999999998E23). Nota sobre D-BUG-01: esos 357 empates se atribuyeron mal a JDK en la medición de D-BUG-01; eran un fallo del render de rakko (desempate por redondeo hacia arriba de la std de Rust), corregido en Q-CFG-30.
- **Alternativas:** (a) rakko sigue JDK 19+: dígitos más cortos, el más cercano al valor exacto y la cifra par en un empate exacto (el empate se detecta con aritmética entera: el valor exacto es mantisa·2^exp); el replay clasifica como NA:Q-CFG-30 solo los casos con la firma del bug de JDK 17. (b) reproducir el algoritmo FloatingDecimal de JDK 17: exige leer código de OpenJDK, prohibido por Q-X-18. (c) aceptar los 357 empates como diferencia: descartada, es un fallo de rakko.
- **Recomendación:** (a). Clasificación: comportamiento observable menor. Consecuencia: el render de un double sigue la semántica de JDK 19+ (cadena más corta); difiere de JDK ≤ 18 en unos 5 de 380 849 dobles aleatorios (≈0,0013 %) y en valores como 2e23, solo en la última cifra; no afecta a getters ni a reference.conf.
- **Filas afectadas:** AC-CFG-017.1;AC-UTIL-002.5
- **Unidades afectadas:** CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a) con la premisa corregida: 357 de las 362 diferencias eran empates exactos mal resueltos por rakko (se corrigen: cifra par) y 5 son bugs de JDK 17; el replay las clasifica como NA:Q-CFG-30 por firma, sin listas de casos. (Owner (respuesta en el hilo de la tarea Q-CFG-30), 2026-11-19)

### Q-EVT-1 — Nombres de clase JVM en mensajes observables (dead letters, canales Debug, `class`/`interface`)

- **Estado:** aceptada
- **Evidencia:** DeadLetterListener.scala:123-137 (`Message [java.lang.Integer]`); EventStreamSpec.scala:395-423 (`channel class ...A`/`interface ...T`); toString de ActorRef `Actor[pekko://sys/path#uid]`.
- **Alternativas:** (a) usar `std::any::type_name` y declarar la diferencia; (b) tabla de nombres Pekko para tipos estándar (Integer, String) y type_name para el resto; (c) nombre configurable por tipo
- **Recomendación:** [Clasificación: divergencia funcional (texto observable)] (b) para tipos primitivos/estándar que los specs afirman (`java.lang.Integer`) y (a) para el resto; el texto exacto de specs portados se ajusta con una función `message_type_name`. Consecuencias observables: mensajes de dead letters, canales Debug y logSource/logClass de ActorLogging, LoggingReceive, system.log y LogSource usan el nombre de tipo elegido. Bloquea: la comparación textual exacta de AC-EVT-006/008/009/010/013.3/018/019; la lógica avanza con `message_type_name`.
- **Filas afectadas:** AC-EVT-006 ; AC-EVT-008 ; AC-EVT-009 ; AC-EVT-010 ; AC-EVT-013.3 ; AC-EVT-018 ; AC-EVT-019 ; AC-SYS-012
- **Unidades afectadas:** EVT
- **Origen:** classic-system
- **Respuesta del Owner:** aceptar, incluida la supresión de dead letters durante el shutdown terminante de 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-EVT-3 — Plantillas `{}` y varargs en LoggingAdapter

- **Estado:** aceptada
- **Evidencia:** Logging.scala:1520-1557; LoggingAdapterTest.java:55-90: hasta 4 argumentos o Array expandido; sufijo ` WARNING arguments left: N`.
- **Alternativas:** (a) métodos con `&[&dyn Display]` + macros `log_info!`; (b) sólo macros (no verificable con tests dinámicos); (c) `format!` nativo sin `{}` de Pekko
- **Recomendación:** [Clasificación: representación que conserva el contrato] (a): conserva el formato textual `{}` y el sufijo de argumentos sobrantes, con macros como azúcar. Consecuencias observables: el sufijo ` WARNING arguments left: N` y el límite de 4 argumentos. Bloquea: nada (puede decidirse al implementar AC-EVT-013.1).
- **Filas afectadas:** AC-EVT-013.1
- **Unidades afectadas:** EVT
- **Origen:** classic-system
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-FSM-4 — Estado inexistente: terminate sin context.stop(self)

- **Estado:** aceptada con cambios
- **Evidencia:** FSM.scala:882-884 (terminate sin stop) vs 869-878 (applyState hace stop) | Cierre 2026-10-31: Clasificacion: divergencia funcional (replicar un comportamiento que parece defecto) — la parte de incertidumbre sobre el comportamiento queda RESUELTA por lectura: FSM.scala:881-884 (makeTransition) llama `terminate(stay().withStopReason(Failure("Next state %s does not exist")))` sin `context.stop(self)`; `terminate` cancela timers, fija currentState al stay() con stopReason, ejecuta onTermination una vez y loguea el error; el actor sigue vivo procesando en el estado anterior, y el terminate de postStop ya no re-ejecuta onTermination. FSMActorSpec:170-186 solo afirma el log de error. No se ha ejecutado el oraculo (Java disponible es 11 y AGENTS.md pide 21); queda como confirmacion opcional.
- **Alternativas:** (a) replicar: el actor sigue vivo tras "Next state X does not exist"; (b) detenerlo tambien
- **Recomendación:** (a) replicar (paridad) y marcarlo como Q; confirmar contra el jar oficial con un escenario de oraculo antes de implementar. Consecuencias observables: (a) un FSM con estado destino inexistente sigue contestando mensajes tras el error; (b) onTermination se invoca con Failure y no se vuelve a invocar al parar. Bloquea AC-FSM-006.1 A1/A4 y AC-FSM-006.2 (A1 depende del orden de terminate); puede avanzar el resto de FSM. Solo hace falta el Owner para elegir replicar (recomendado) o detener el actor tambien.
- **Filas afectadas:** AC-FSM-006.1;AC-FSM-006.2
- **Unidades afectadas:** FSM
- **Origen:** classic-patterns
- **Respuesta del Owner:** aceptar (a) replicar. Condición: antes, grabar un escenario del oráculo contra 2.0.0 que lo confirme; si el oráculo contradice la lectura del código, gana el oráculo. (Owner (respuesta al lote 01), 2026-11-18)

### Q-PATH-1 — Cómo ordenar listas de Address con un comparador que no es orden total

- **Estado:** aceptada
- **Evidencia:** Address.scala:135-145; plan AC-PATH: slice::sort_by puede entrar en pánico en Rust >= 1.81 si detecta un orden no total y puede diferir de List.sort de Java (CMPA-17).
- **Alternativas:** (a) sólo `pekko_compare` y documentar el límite; (b) función que reproduce el TimSort de Java; (c) desempate `None < Some` con `Ord` (contradice la decisión del Owner).
- **Recomendación:** (a) hasta que haya un consumidor (cluster). [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA/divergencia funcional acotada (orden no total). Consecuencias observables: ordenar listas de Address con el comparador de Pekko puede diferir de List.sort. Bloquea: nada hasta que exista un consumidor (cluster); puede avanzar: AC-PATH-002 con pekko_compare.
- **Filas afectadas:** AC-PATH-002
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-PATH-2 — Surrogate suelto en el mensaje de validación de nombres

- **Estado:** aceptada
- **Evidencia:** ActorPath.scala:107-113: para `x🙂` Pekko informa `illegal character [\uD83D] at position: 1` (VAL-23); un String de Rust no puede contener un surrogate suelto.
- **Alternativas:** (a) U+FFFD (`from_utf16_lossy`); (b) `?`; (c) el carácter completo; (d) guardar el mensaje en UTF-16.
- **Recomendación:** (a); la posición sigue siendo el índice UTF-16. [Cierre documental 2026-10-31] Clasificación: DIVERGENCIA FUNCIONAL menor inevitable (String de Rust no contiene surrogates sueltos). Consecuencias observables: sólo el texto del mensaje de error de nombres con surrogates sueltos. Bloquea: AC-PATH-008 (cierre; VAL-22/23 pendientes); puede avanzar: el resto.
- **Filas afectadas:** AC-PATH-008
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar lo implementado: message() muestra U+FFFD y message_utf16() es exacto (el sustituto suelto de Pekko). (Owner (respuesta al lote 01), 2026-11-18)

### Q-PATH-3 — Valor de hashCode de Address y ActorPath

- **Estado:** aceptada
- **Evidencia:** Address.scala:77-79 (MurmurHash3.productHash), ActorPath.scala:469-479 (hash por niveles). Rust sólo expone `Hash`.
- **Alternativas:** (a) no reproducir el valor; (b) `pekko_hash_code() -> i32` idéntico.
- **Recomendación:** (a); (b) si alguna unidad lo necesita. [Cierre documental 2026-10-31] Clasificación: representación; depende del hash de Scala (ver Q-UTIL-14). Bloquea: nada.
- **Filas afectadas:** AC-PATH-001, AC-PATH-014
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-PATH-4 — `null` de Java en Address/ActorPath

- **Estado:** aceptada
- **Evidencia:** fromString(null) y validatePathElement(null) lanzan NullPointerException; `new Address(null, "s")` imprime `null://s`.
- **Alternativas:** (a) aceptarlo como diferencia de lenguaje; (b) tipos `Option<&str>` en la API.
- **Recomendación:** (a); `host = null` sí se representa como None. [Cierre documental 2026-10-31] Clasificación: DIVERGENCIA DE LENGUAJE (null). Bloquea: nada; puede avanzar con (a).
- **Filas afectadas:** AC-PATH-001, 004, 007, 008
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-PATH-5 — Fachada Java ActorPaths

- **Estado:** aceptada
- **Evidencia:** ActorPath.scala:29-65 delega 1:1 en ActorPath; es la pregunta 7 de docs/inventory/actor-classic.md.
- **Alternativas:** (a) excluir; (b) fusionar con AC-PATH-007/008; (c) portar como funciones libres.
- **Recomendación:** (b)/(c): las funciones libres ya cubren el contrato; la exclusión requiere decisión explícita. [Cierre documental 2026-10-31] Clasificación: EXCLUSIÓN DE ALCANCE si se elige (a) (fachada Java); (b)/(c) conservan el contrato. Bloquea: AC-PATH-009 (cierre); puede avanzar: AC-PATH-007/008.
- **Filas afectadas:** AC-PATH-009
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-PATH-6 — Licencia de uri.rs y jstring.rs (comportamiento portado de OpenJDK 21, GPL-2.0 WITH Classpath-exception-2.0)

- **Estado:** aceptada con cambios
- **Evidencia:** crates/rakka-actor/src/actor/uri.rs y jstring.rs reproducen java.net.URI, sun.nio.cs.UTF_8, String.compareTo, Integer.valueOf, Character.digit; el proyecto es Apache-2.0 (docs/upstream.md).
- **Alternativas:** (a) aceptar como reimplementación del comportamiento y declarar la referencia en el encabezado (estado actual); (b) sala limpia desde RFC 2396/3629 y el oráculo; (c) crate externo compatible.
- **Recomendación:** Es una pregunta de licencia, no de semántica; recomendar (b) o (a) con revisión legal. [Cierre documental 2026-10-31] Clasificación: pregunta de LICENCIA (no semántica) que el Owner debe resolver; las fuentes del repositorio no la resuelven. Consecuencias observables: ninguna en comportamiento; el riesgo es legal. Bloquea: publicación de uri.rs/jstring.rs; puede avanzar: el resto de PATH.
- **Filas afectadas:** AC-PATH-002, 004, 005, 006, 007, 011, 014
- **Unidades afectadas:** PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (b): reescritura en sala limpia de uri.rs y jstring.rs desde RFC 2396/3986/3629 y los casos grabados, por un agente que no lee los archivos actuales ni código de OpenJDK. No se publica ningún release hasta cerrarla. NOTICE se corrige a 2.0.0. AMPLIACIÓN (Owner, misma fecha): todo código de rakka derivado de fuentes con licencia no compatible con Apache-2.0, o de procedencia dudosa, se reescribe en sala limpia con proceso documentado (ver Q-X-18); el objetivo es eliminar la duda con un proceso, no decidir si algo es legal. (Owner (respuesta al lote 01), 2026-11-18)

### Q-RT-4 — Introspeccion de mailbox para SmallestMailbox y Resizer (pressure)

- **Estado:** aceptada
- **Evidencia:** SmallestMailbox.scala:108-153; Resizer.scala:206-226 | Cierre 2026-10-31: Clasificacion: arquitectura (vista interna del mailbox en AC-MBX-019). Evidencia ampliada: SmallestMailbox.scala:96-153 usa is_scheduled/currentMessage/hasMessages/numberOfMessages/isSuspended; Resizer.scala:206-226 (pressure) usa isScheduled, hasMessages, currentMessage, numberOfMessages; OptimalSizeExploringResizer.scala:217-228 usa numberOfMessages + currentMessage != null.
- **Alternativas:** (a) exponer una vista interna de solo lectura del mailbox (is_scheduled, has_messages, number_of_messages, is_suspended, current_message) en AC-MBX; (b) heuristica aproximada
- **Recomendación:** (a) necesaria para paridad; marcar bloqueante si AC-MBX-019 no la ofrece. Consecuencias: sin la vista, SmallestMailbox degrada a heuristica (cambia la eleccion de routee) y los resizers no pueden medir presion/carga. Bloquea la integracion de AC-RT-008, 013.2 y 014 (reportMessageCount); NO bloquea la logica pura de AC-RT-013.1 (capacity/rampup/backoff), de AC-RT-014 (resize/optimize/explore con carga inyectada) ni los constructores. Es el unico Q bloqueante de RT: priorizarlo en AC-MBX.
- **Filas afectadas:** AC-RT-008;AC-RT-013.1;AC-RT-013.2;AC-RT-014
- **Unidades afectadas:** RT
- **Origen:** classic-patterns
- **Respuesta del Owner:** aceptar (a), como vista interna (no API pública estable) en AC-MBX-019. (Owner (respuesta al lote 01), 2026-11-18)

### Q-SCH-1 — Rueda de ticks (LightArrayRevolver) propia vs. timers del runtime

- **Estado:** aceptada
- **Evidencia:** LightArrayRevolverScheduler.scala:50-357; reference.conf:886-927; SchedulerSpec.scala:467-857 (redondeo al tick, ticks-per-wheel, wrap, relojes que dan la vuelta, rechazo de retardos, y un Driver determinista con reloj, waitNanos y startTick sobreescribibles)
- **Alternativas:** (a) portar la rueda (precisión por tick, semántica de redondeo y wrap); (b) tokio::time y emular redondeo a tick
- **Recomendación:** [Clasificación: arquitectura] (a) port de la rueda con `Clock` y `Waiter` inyectables sobre un hilo propio (la rueda es observable: redondeo al tick, ticks-per-wheel, rechazo de retardos); es exactamente el patrón del spec y permite portar 1:1 las pruebas determinísticas. (b) tokio::time con redondeo emulado no es equivalente y no se recomienda. Consecuencias: bloquea la implementación de AC-SCH-008.1/.2/.3, 009, 014, 015; AC-SCH-001..003 pueden probarse antes con el scheduler manual (AC-SCH-010).
- **Filas afectadas:** AC-SCH-001,AC-SCH-002,AC-SCH-003,AC-SCH-008.1,AC-SCH-008.2,AC-SCH-008.3,AC-SCH-009,AC-SCH-014
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch
- **Respuesta del Owner:** aceptar (a): rueda propia con hilo propio. (Owner (respuesta al lote 01), 2026-11-18)

### Q-SCH-2 — Scheduler pluggable (scheduler.implementation) sin reflexión

- **Estado:** aceptada
- **Evidencia:** reference.conf:906-921; ActorSystem.scala:468,965,1136-1165; ExplicitlyTriggeredScheduler.scala:44-141; ManualTime.scala:34-48
- **Alternativas:** (a) registro por nombre + inyección en builder; (b) sólo inyección
- **Recomendación:** [Clasificación: arquitectura; subordinada a Q-X-2] (a) registro por nombre + inyección en el builder del sistema; el scheduler manual de testkit se registra como `org.apache.pekko.testkit.ExplicitlyTriggeredScheduler`. Consecuencia: el testkit determinista (ManualTime) depende de esto. Bloquea sólo la carga por nombre.
- **Filas afectadas:** AC-SCH-010
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch
- **Respuesta del Owner:** aceptar (a). (Owner (respuesta al lote 01), 2026-11-18)

### Q-SCH-4 — schedule(...) deprecado: mantener como alias de scheduleAtFixedRate

- **Estado:** rechazada
- **Evidencia:** Scheduler.scala:229-231 (scheduleAtFixedRate delega en schedule), :343-420 (sobrecargas deprecadas desde Akka 2.6.0), :403 (método abstracto); LightArrayRevolverScheduler.scala:134-161; ExplicitlyTriggeredScheduler.scala:52-54; ExplicitlyTriggeredSchedulerSpec.scala:24-106 (ejerce schedule directamente)
- **Alternativas:** (a) alias deprecado; (b) omitir y exponer sólo schedule_at_fixed_rate (propuesta de exclusión, requiere autorización)
- **Recomendación:** [Clasificación: exclusión de alcance (propuesta) o alias] (a): alias deprecado, costo nulo, mismo código interno; (b) exponer sólo `schedule_at_fixed_rate` queda como propuesta de exclusión que requiere autorización. La fila sigue en alcance. No bloquea.
- **Filas afectadas:** AC-SCH-004,AC-SCH-003
- **Unidades afectadas:** SCH
- **Origen:** classic-dispatch
- **Respuesta del Owner:** rechazar (a) (alias deprecado). Elegido (b): sólo schedule_at_fixed_rate, como en 2.0.0. Autoriza la exclusión del alias `schedule` deprecado. (Owner (respuesta al lote 01), 2026-11-18)

### Q-SER-1 — Resolución de bindings y manifests sin jerarquía de clases JVM

- **Estado:** aceptada
- **Evidencia:** Serialization.scala:326-390,507-520 (isAssignableFrom, orden subtipo-primero); SerializeSpec.scala:191-243 (8 casos de jerarquía/interfaz); reference.conf:817-831 (claves FQCN JVM como "[B", "java.io.Serializable", "scala.Long"); Serialization.scala:241-258 (manifest = nombre de clase cargado por reflexión)
- **Alternativas:** (a) Registro de tipos con relación de supertipos declarada (marker traits registrados) y orden por especificidad; (b) solo coincidencia exacta de TypeId (sin jerarquía) y bindings por enum cerrado; (c) bindings por nombre lógico de mensaje (string estable) en lugar de FQCN
- **Recomendación:** (a) con (c) para manifests: cada tipo registrado declara su nombre lógico y supertipos; las claves FQCN de reference.conf se aceptan como alias de nombre lógico; reutilizar la solución de Q-X-6 (jerarquía en EventStream). Conserva el desempate y el aviso "Multiple serializers found". El typo "synch" de un mensaje (Serialization.scala:277) queda a elección: copiar o corregir | Clasificación (cierre documental 2026-10-31): arquitectura (representación de tipos y jerarquías JVM).
- **Filas afectadas:** AC-SER-001.1; AC-SER-001.2; AC-SER-001.3; AC-SER-002; AC-SER-003.2; AC-SER-003.3; AC-SER-003.5
- **Unidades afectadas:** SER
- **Origen:** classic-io-ser
- **Respuesta del Owner:** aceptar (a)+(c), incluido el comportamiento de 2.0.0 con includeManifest=false. El typo "synch" se copia por compatibilidad de configuración. (Owner (respuesta al lote 01), 2026-11-18)

### Q-T-BEH-1 — Forma del tipo Behavior en Rust, setup re-ejecutable y reutilizacion del valor

- **Estado:** aceptada
- **Evidencia:** Pekko reutiliza el mismo valor Behavior en varios spawns y re-ejecuta setup en cada restart (actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:331-339; actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:404; actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala:198-211); sin eso supervise/restart no puede recrear el Behavior. Requisitos observables que cualquier representacion debe cumplir (contrato en AT-BEH-007, R1-R3): reutilizar el mismo Behavior en actores independientes y volver a ejecutar la fabrica de setup en cada restart (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:404; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1145-1153, 1673-1691). El FnOnce propuesto antes en AT-BEH-001/AT-BEH-007 contradice R1 y R2.
- **Alternativas:** (a) enum Behavior<M> cerrado con Deferred(Arc<dyn Fn(&mut Ctx)->Behavior>) y Behavior: Clone via Arc; (b) Box<dyn FnOnce> y prohibir restart de esos Behaviors (rompe AT-BEH-023 y AT-SUP-005); (c) trait object Behavior con metodo clone_box.
- **Recomendación:** Clasificacion: arquitectura. Consecuencia observable: sin (a) un Behavior no puede reutilizarse ni reiniciarse (se rompen restart y AT-BEH-023). Bloquea AT-BEH-001, 004, 007, 011, 023 y AT-SUP-005/013; puede avanzar todo lo que no construye Behaviors (SIG, REF, STASH solo-estructura). | Recomendacion previa: (a). Es el unico que conserva restart y reutilizacion. Es una recomendacion: el Owner no la acepto; hasta que responda AT-BEH-001, AT-BEH-004 y AT-BEH-007 no se cierran y ninguna alternativa se considera adoptada.
- **Filas afectadas:** AT-BEH-001; AT-BEH-007; AT-BEH-011; AT-BEH-023; AT-SUP-005; AT-SUP-013
- **Unidades afectadas:** BEH; SUP
- **Origen:** typed-core
- **Respuesta del Owner:** aceptar (a). La caché de Props de 2.0.0 no se replica: es una optimización interna sin efecto observable. (Owner (respuesta al lote 01), 2026-11-18)

### Q-T-SUP-3 — Backoff: maxRestarts 0/negativo y default de resetBackoffAfter segun doc Java

- **Estado:** aceptada con cambios
- **Evidencia:** Scaladoc de withMaxRestarts: 'Zero or negative disables the upper limit' (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:329-333) vs codigo que solo trata -1 como ilimitado (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala:145-152, 335); Scaladoc Java de withResetBackoffAfter dice 'same value as minBackoff' (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:322-326) vs codigo y spec (min+max)/2 (actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala:82-86; actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:1117-1120).
- **Alternativas:** (a) seguir el codigo: solo -1 ilimitado, 0 y otros negativos fallan en la primera excepcion; resetBackoffAfter por defecto (min+max)/2; (b) seguir el Scaladoc: <=0 desactiva el limite.
- **Recomendación:** Clasificacion: incertidumbre parcialmente resuelta; el caso maxRestarts<=0 requiere medicion con el oraculo y luego decision. Resuelto por lectura: el default de resetBackoffAfter es (min+max)/2 (spec :1117-1120; el Scaladoc Java es erroneo). Para maxRestarts: el codigo trata solo -1 como ilimitado (Supervision.scala:145-152, 335) y ningun spec afirma 0 ni negativos; la medicion con el oraculo (withMaxRestarts(0)) decidira la lectura de Pekko, y solo si el oraculo contradice al codigo leido hace falta al Owner. Bloquea AT-SUP-007.3; AT-SUP-007.2 puede avanzar. | Recomendacion previa: (a) para el default de resetBackoffAfter (lo afirma el spec); para maxRestarts <=0 pedir decision: no hay spec que afirme ninguna de las dos lecturas.
- **Filas afectadas:** AT-SUP-007.2; AT-SUP-007.3
- **Unidades afectadas:** SUP
- **Origen:** typed-core
- **Respuesta del Owner:** aceptar (a) para el default (min+max)/2. Para maxRestarts <= 0 manda el código, no el Scaladoc; condición: confirmarlo con el oráculo antes de implementar AT-SUP-007.3. (Owner (respuesta al lote 01), 2026-11-18)

### Q-TK-1 — Entrega síncrona del testActor (CallingThreadDispatcher) en TestKit/TestProbe

- **Estado:** aceptada
- **Evidencia:** TestKit.scala:189-193 crea el testActor con CallingThreadDispatcher; TestProbeSpec.scala:32-39 hace `expectMsg(0.millis, ..)` inmediatamente tras `ask`/tell y afirma `future.isCompleted == true` tras responder. Cierre 2026-10-31 (leídos CallingThreadDispatcher.scala completo, CallingThreadDispatcherModelSpec y ActorModelSpec:278-508): el testActor se crea con CallingThreadDispatcher (TestKit.scala:189-193); la entrega síncrona se apoya en register->runQueue, dispatch->runQueue y un candado reentrante por actor con colas por hilo (CallingThreadDispatcher.scala:183-338). Es implementable en Rust (ver AC-TK-006.2, Equivalente Rust).
- **Alternativas:** (a) dispatcher inline obligatorio para TestActor (misma semántica que Pekko); (b) entrega asíncrona con expectMsg que sondea y cambiar los specs portados; (c) ambos, con inline por defecto; (d) TestProbe como ref especial que empuja directamente a su cola en el hilo del emisor (misma entrega observable, sin path /system/testActor ni ciclo de vida de actor)
- **Recomendación:** [Clasificación: arquitectura] (a): sin ella no se portan 1:1 los specs que usan expectMsg(0.millis). Consecuencias observables: sin entrega síncrona fallan 1:1 los specs con expectMsg(0.millis, ..) y `future.isCompleted` tras responder (TestProbeSpec.scala:32-57, TestActorRefSpec.scala:224-230, ExecutionContextSpec.scala:168-185). Bloquea: AC-TK-001.1, AC-TK-002.1, AC-TK-006.1, AC-TK-006.2 y el port de los specs que usen testActor/TestProbe con 0 ms. Puede avanzar: el resto de AC-SYS/AC-EVT/AC-TK y los specs con esperas no nulas (con sondeo, opción b).
- **Filas afectadas:** AC-TK-001.1 ; AC-TK-002.1 ; AC-TK-006.2 ; AC-TK-006.1 ; AC-TK-007
- **Unidades afectadas:** TK ; DSP
- **Origen:** classic-system
- **Respuesta del Owner:** aceptar (a): dispatcher inline obligatorio para el testActor. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-1 — Negativos y rango de Timeout

- **Estado:** rechazada
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Preguntas abiertas, Q-UTIL-1): `FiniteDuration` es un `Long` de nanosegundos con signo entre -(2^63-1) y 2^63-1; `Timeout(-5.millis)` es válido y `ask` lo rechaza después (AC-ASK-001, ≤0). `std::time::Duration` no tiene negativos y llega a ~584 mil años. `Timeout.create(ofSeconds(Long.MAX_VALUE))` lanza `ArithmeticException: long overflow` en Pekko y tampoco se reproduce.
- **Alternativas:** (a) quedarse con `std::time::Duration` y que `ask` reciba un `Timeout` siempre positivo, documentando que `Timeout(-5.millis)` no se puede construir (lo implementado); (b) tipo `FiniteDuration` propio con `i64` nanosegundos, exacto pero hay que reimplementar la aritmética; (c) `Timeout::new` falible que rechace lo que Pekko no puede construir.
- **Recomendación:** (a) hasta que AC-ASK necesite el caso. Sin respuesta del Owner la fila AC-UTIL-001 queda `partial`.
- **Filas afectadas:** AC-UTIL-001
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** rechazar: la recomendación (a) (std::time::Duration, sólo valores positivos) NO se adopta. ALTERNATIVA ELEGIDA Y VIGENTE: preservar el dominio de Pekko, es decir (b) en su variante mínima (i64 nanosegundos con signo, sin biblioteca de aritmética) con (c) constructores falibles que rechazan lo que Pekko rechaza; admite duraciones negativas, cero y positivas en [-(2^63-1), 2^63-1] ns (Long.MinValue fuera). La restricción a positivos no se acepta como adaptación al lenguaje. No decide Q-UTIL-2, Q-UTIL-3, Q-UTIL-4 ni Q-UTIL-5 (Owner (instrucción en la sesión), 2026-10-31)

### Q-UTIL-2 — toString de Timeout conserva la unidad

- **Estado:** aceptada
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-2): Pekko imprime `Timeout(100 milliseconds)` o `Timeout(0 days)`: el `FiniteDuration` recuerda la unidad con que se construyó, aunque `Timeout(1000.millis) == Timeout(1.second)`. `std::time::Duration` no la recuerda.
- **Alternativas:** (a) `Debug` derivado (`Timeout(100ms)`) sin prometer el formato de Pekko (lo implementado); (b) guardar la unidad junto con el valor (rompe la igualdad por valor o exige un `PartialEq` manual).
- **Recomendación:** (a): sólo aparece en logs. Sin respuesta del Owner la fila AC-UTIL-001 queda `partial`. [Cierre documental 2026-10-31] Clasificación: representación (toString). Bloquea: cierre de AC-UTIL-001 (TO-STR-*); puede avanzar: resto de AC-UTIL-001.
- **Filas afectadas:** AC-UTIL-001
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-3 — null y mensajes diferidos en PekkoException y requiring

- **Estado:** aceptada
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-3): `new PekkoException(null)` tiene `getMessage == null` y se imprime sin `: `; `requiring(cond, null)` da `requirement failed: null`. Con `&str`/`impl Display` no se puede expresar `null`. Igual que Q-PATH-4.
- **Alternativas:** (a) aceptarlo como diferencia de lenguaje (única alternativa registrada en el plan).
- **Recomendación:** (a), igual que Q-PATH-4. Sin respuesta del Owner las filas afectadas quedan `partial`. [Cierre documental 2026-10-31] Clasificación: DIVERGENCIA DE LENGUAJE (null). Bloquea: cierre de AC-UTIL-002.3/004.1/004.2; puede avanzar: el resto.
- **Filas afectadas:** AC-UTIL-002.3; AC-UTIL-004.1; AC-UTIL-004.2
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-4 — Jerarquía de excepciones y OnlyCauseStackTrace

- **Estado:** aceptada
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-4): (i) `ConfigurationException` es subclase de `PekkoException`: un `catch PekkoException` la atrapa; con tipos separados no. (ii) `OnlyCauseStackTrace` sólo cambia qué stack trace imprime la JVM; los errores de Rust no llevan stack trace.
- **Alternativas:** (a) tipos separados y un `From<ConfigurationException> for PekkoException` (pierde `java_class`) cuando algún consumidor lo pida; (b) un único `enum PekkoError`; (c) un trait común. Para (ii) el plan propone excluir `OnlyCauseStackTrace` y documentarlo.
- **Recomendación:** (i) (a); (ii) excluirlo y documentarlo. La exclusión de `OnlyCauseStackTrace` requiere autorización explícita del Owner (A-5). Sin respuesta las filas quedan `partial`. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA (jerarquía de errores) + EXCLUSIÓN propuesta (OnlyCauseStackTrace). Bloquea: AC-UTIL-004.2/.3 (cierre); puede avanzar: AC-UTIL-004.1.
- **Filas afectadas:** AC-UTIL-004.2; AC-UTIL-004.3
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (i-a). Autoriza excluir OnlyCauseStackTrace. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-5 — Done propio o ()

- **Estado:** aceptada
- **Evidencia:** docs/superpowers/plans/2026-10-30-ac-util-core.md (Q-UTIL-5), inventario AC-UTIL-003.1: se implementó `pub struct Done;`; la fila queda `partial` hasta decidir. Java expone `getInstance()` y `done()`.
- **Alternativas:** (a) `pub struct Done;` (lo implementado); (b) `()`.
- **Recomendación:** Sin recomendación registrada en el plan: lo implementado (a) es un estado, no una decisión aceptada. La fila AC-UTIL-003.1 queda `partial`. [Cierre documental 2026-10-31] Clasificación: representación (Done). Bloquea: cierre de AC-UTIL-003.1/.2 (estado `partial`); puede avanzar: el resto.
- **Filas afectadas:** AC-UTIL-003.1
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a): pub struct Done. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-6 — Version.compareTo es asimétrico cuando un lado no tiene calificador (rest vacío) y el otro sí

- **Estado:** rechazada
- **Evidencia:** util/Version.scala:170-175: `if (rest=="" && other.rest!="") diff=1` seguido de un `if (other.rest=="" && rest!="") diff=-1 else diff=rest.compareTo(other.rest)` independiente; con rest="" y other.rest="M1" el `else` sobrescribe a 1 con "".compareTo("M1") < 0. VersionSpec.scala:56-64 sólo prueba `1.2-M1 < 1.2.0` (lado con calificador a la izquierda). Sin ejecutar con el jar: hipótesis a confirmar. [Cierre documental 2026-10-31] CONFIRMADO midiendo con el jar 1.7.1 (JDK 11, ORACLE=jars tools/oracle/run.sh): Version("1.2.0").compareTo(Version("1.2-M1")) = -2 y Version("1.2-M1").compareTo(Version("1.2.0")) = -1 (ambos negativos); 1.2.0 vs 1.2.0-RC1 = -3 y 1.2.0-RC1 vs 1.2.0 = -1.
- **Alternativas:** (a) reproducir exactamente (bug-compatible) y fijarlo con un test del jar; (b) corregir para que sea antisimétrico (1.2.0 > 1.2-M1) y registrar la diferencia; (c) no portar Version.compareTo y usar semver.
- **Recomendación:** Confirmar primero con el jar (escenario nuevo en tools/oracle). Si se confirma, (a) hasta que el Owner decida: cualquier diferencia no autorizada es regresión (AGENTS.md). [Cierre documental 2026-10-31] Clasificación: la incertidumbre se resolvió leyendo y midiendo (defecto confirmado); lo que queda es una DIVERGENCIA FUNCIONAL candidata que sólo el Owner puede autorizar (bug-compatible vs corregir). Consecuencias observables: con (a) una versión final 1.2.0 compara MENOR que 1.2-M1 (ManifestInfo.checkSameVersion y PekkoVersion eligen la 'más alta' con max); con (b) 1.2.0 > 1.2-M1. Bloquea: la implementación de la comparación con calificador (AC-UTIL-005.2 A5/A8) y la elección de la versión más alta en AC-UTIL-016.9; puede avanzar: parseo, igualdad, comparación numérica y dynver (A1-A4, A6-A7).
- **Filas afectadas:** AC-UTIL-005.2
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** rechazar (a) bug-compatible de 1.7.1. Elegido: seguir el comportamiento corregido de 2.0.0 (1.2.0 > 1.2.0-M1). Condición: medirlo con el jar 2.0.0 antes de implementarlo. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-7 — toRootLowerCase: tabla de minúsculas de Unicode distinta entre JDK y Rust

- **Estado:** aceptada
- **Evidencia:** util/Helpers.scala:40 usa String.toLowerCase(Locale.ROOT); el oráculo corre con JDK 11 (Unicode 10) y Rust 1.96 usa Unicode más nuevo (plan AC-UTIL, decisión 6). [Cierre documental 2026-10-31] Medido con el jar (JDK 11): toRootLowerCase("\u1C90") devuelve U+1C90 (sin cambio; el carácter es de Unicode 11). La salida de Pekko depende de la JDK que lo ejecute (JDK 17 = Unicode 13, JDK 21 = Unicode 15), de modo que no existe una única referencia 'Unicode de Pekko'.
- **Alternativas:** (a) usar `str::to_lowercase` y declarar la diferencia para caracteres posteriores a Unicode 10; (b) tabla propia congelada a Unicode 10; (c) tabla del JDK 21 (referencia del port de URI).
- **Recomendación:** (a): los consumidores (log-dead-letters, niveles de log) sólo reciben ASCII en la práctica. [Cierre documental 2026-10-31] Clasificación: representación que conserva el contrato en la práctica (los consumidores son ASCII) con una divergencia menor y acotada; la opción (c) es inviable como 'referencia' porque Pekko no fija la versión de Unicode. Consecuencias observables: sólo para caracteres añadidos después de la JDK usada (p. ej. U+1C90 -> U+10D0 en Rust). Bloquea: nada; puede avanzar: AC-UTIL-002.2 con (a) y el golden limitado a Unicode <= 10 (como ahora).
- **Filas afectadas:** AC-UTIL-002.2
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-8 — Semántica de cadenas por unidades UTF-16 (Glob, makePattern, WildcardIndex)

- **Estado:** aceptada
- **Evidencia:** util/Glob.scala:43-72 recorre `charAt` (unidades UTF-16): `?` consume una unidad, no un punto de código; Helpers.makePattern usa `.` de java.util.regex que no cruza terminadores de línea (Helpers.scala:44-45) mientras Glob.matches sí; WildcardIndex.scala:87-89 elige entre `prefix*` solapados según el orden de iteración de un HashMap de Scala. GlobSpec.scala:60-73 sólo compara sobre {a,b,*,?}. [Cierre documental 2026-10-31] Medido con el jar: Glob.matches("a?b","a\nb") = true, Glob.matches("a*b","a\nb") = true, Glob.matches("a?b","a\u0085b") = true, pero makePattern("a?b") rechaza "a\nb" y "a\u0085b" y makePattern("a*b") rechaza "a\nb". Par sustituto: makePattern("a?b") acepta "a\uD83D\uDE42b" (un punto de código) y Glob.matches("a?b", ...) = false (Glob.matches("a??b", ...) = true). Uso en producción: ActorSelection.scala:364 (`pattern = Helpers.makePattern`, usado por SelectChildPattern) y :369 (`Glob.matches`, el que decide).
- **Alternativas:** (a) operar sobre `&[u16]` (idéntico a Java, incluidos pares sustitutos); (b) sobre `char` de Rust (distinto para caracteres fuera del BMP); (c) igual que (a) y fijar el orden de `prefix*` por inserción; documentar la diferencia de saltos de línea.
- **Recomendación:** (c): los nombres válidos de actor son ASCII/%XX (ActorPath.scala:127-148) y los patrones de selección vienen del usuario; la diferencia de saltos de línea debe decidirla el Owner. [Cierre documental 2026-10-31] Clasificación: representación (UTF-16 vs char) + divergencia funcional ya existente DENTRO de Pekko (Glob y makePattern difieren en saltos de línea y pares sustitutos). Resuelto por medición: la semántica contractual en producción es la de Glob.matches (UTF-16, sin terminadores de línea especiales); makePattern sólo se conserva como definición histórica. Consecuencias observables: nombres de actor con caracteres fuera del BMP o con saltos de línea (los nombres válidos son ASCII/%XX, ActorPath.scala:127-148, así que sólo afecta a patrones de selección arbitrarios). Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023).
- **Filas afectadas:** AC-UTIL-009, AC-UTIL-016.6, AC-UTIL-023, AC-CFG-012.2 [asociacion explicita por contrato, cierre P05: AC-UTIL-016.15]
- **Unidades afectadas:** UTIL, CFG
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (c). Autoriza no reproducir makePattern bit a bit. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-9 — PrettyDuration: `%.4g` de java.util.Formatter y representación de infinitos/indefinido

- **Estado:** aceptada
- **Evidencia:** util/PrettyDuration.scala:51-55,57-59: `"%.{p}g"` con Locale.ROOT; el %g de Java usa notación científica cuando el valor redondeado >= 10^precisión (p. ej. 10000 d) y difiere del %g de C/Rust; Duration.Inf/MinusInf/Undefined son valores de scala.concurrent.duration.Duration, que `std::time::Duration` no tiene. PrettyDurationSpec.scala:25-57 fija 10 casos + 3 sin números. [Cierre documental 2026-10-31] Medido con el jar: 10000 days -> `1.000e+04 d`, 99999 days -> `1.000e+05 d`, 1 day -> `1.000 d`, 1500 microseconds -> `1.500 ms`, Duration.Inf -> `∞ (infinity)`. Consumidores: AsyncDnsResolver.scala:37,146 y DeadLetterListener.scala:31,99 (sólo mensajes de log).
- **Alternativas:** (a) implementar el %g de Java a mano y un enum `PekkoDuration { Finite, Inf, MinusInf, Undefined }`; (b) limitar a los casos del spec y documentar el resto; (c) delegar a un crate de formateo.
- **Recomendación:** (a), validado contra un escenario del oráculo con valores grandes y pequeños. [Cierre documental 2026-10-31] Clasificación: representación (reimplementar %.4g de Java; la regla observable es: notación científica con exponente de dos dígitos cuando el valor redondeado a 4 cifras significativas es >= 10^4). Resuelto por medición; no requiere al Owner salvo que se prefiera (b) (limitar a los casos del spec). Consecuencias observables: sólo texto de logs. Bloquea: nada; puede avanzar: AC-UTIL-016.5 con (a) y el golden por crear.
- **Filas afectadas:** AC-UTIL-016.5
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-10 — Estructuras concurrentes e identidad (Index, Switch, TokenBucket, colas, identityHashCode, timestamp)

- **Estado:** aceptada
- **Evidencia:** Index.scala:23-199 (ConcurrentSkipListSet por clave, iteración débilmente consistente, isEmpty con falsos negativos), LockUtil.scala:29-155 (synchronized), TokenBucket.scala:19-91 (aritmética con wrap-around), Helpers.scala:47-69 (identityHashCode), Helpers.scala:108-113 (zona horaria del sistema).
- **Alternativas:** (a) Mutex + estructuras estándar, iteración sobre instantánea; (b) mapas concurrentes lock-free; (c) fijar una API mínima por consumidor real y diferir el resto.
- **Recomendación:** (a) con tests portados 1:1 (IndexSpec, SwitchSpec, TokenBucketSpec); id de identidad = dirección del Arc o contador. [Cierre documental 2026-10-31] Clasificación: arquitectura (modelo de concurrencia de las estructuras). Consecuencias observables: iteración débilmente consistente de Index y semántica de Switch/BoundedBlockingQueue (AC-UTIL-012: 79 casos con eventos de señal). Hallazgos del cierre: MessageBuffer/MessageBufferMap y las colas LRU/LFU (AC-UTIL-010, 017.*) NO son thread-safe por contrato (no aplican). Bloquea: AC-UTIL-012 y AC-UTIL-013 (colas bloqueantes) hasta fijar Q-X-4; puede avanzar: AC-UTIL-010, 011, 017.*.
- **Filas afectadas:** AC-UTIL-007, 008, 010, 011, 012, 013, 015.1, 024, 025
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a). (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-11 — Elementos propios de la JVM o de Scala sin equivalente natural en Rust (propuesta de equivalente; no exclusión)

- **Estado:** aceptada con cambios
- **Evidencia:** util/LockUtil.scala:19-27 (ReentrantGuard, deprecado), util/TypedMultiMap.scala, util/JavaVersion.scala, util/Reflect.scala, util/LineNumbers.scala, util/BoxedType.scala, java/org/apache/pekko/annotation/*.java, java/org/apache/pekko/util/Unsafe.java, mima-filters, shims de Scala 2.12/2.13/3 (scala-2.13/util/*Converters.scala), cachés de cluster (FrequencySketch...).
- **Alternativas:** (a) cada fila se mantiene en alcance con un equivalente Rust documentado (atómicos, `pub(crate)`, traits sellados, semver) y la diferencia observable anotada; (b) proponer al Owner excluirlas explícitamente fila por fila.
- **Recomendación:** (a) ahora; (b) sólo con aprobación explícita del Owner (nada se excluye sin autorización). [Cierre documental 2026-10-31] Clasificación: mezcla de representación (Option<T>, atómicos, pub(crate), semver-checks) y EXCLUSIÓN DE ALCANCE (sólo si el Owner la aprueba fila por fila). Cierre documental: cada fila afectada (AC-UTIL-014, 015.2, 016.1-.4, .7, .8, .9, .11-.15, .18, 017.*, 018.*, 019, 020, 021, 022) tiene ahora un efecto observable identificado y una representación Rust; ninguna se excluye. Candidatas a exclusión explícita (requieren aprobación): AC-UTIL-016.7 (findMarker/findClassLoader/getCallerClass), 016.12 (ClassLoaderObjectInputStream), 016.13 (BoxedType), 018.3 (ccompat), 021 (MiMa, sustituible por cargo-semver-checks). Bloquea: nada; puede avanzar: el resto con (a).
- **Filas afectadas:** AC-UTIL-014, 015.2, 016.*, 017.*, 018.*, 019, 020, 021, 022 [asociacion explicita por contrato, cierre P05: AC-UTIL-016.1, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-016.8, AC-UTIL-016.9, AC-UTIL-016.10, AC-UTIL-016.11, AC-UTIL-016.13, AC-UTIL-016.14, AC-UTIL-016.18, AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.4, AC-UTIL-018.5, AC-UTIL-019]
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y SWARUtil. Las exclusiones candidatas 016.7, 016.12, 016.13 y 021 NO quedan autorizadas: el Owner decide aparte tras una línea por cada una con lo que se pierde. RESPUESTA POSTERIOR (2026-11-18): se autoriza excluir AC-UTIL-016.7 (cubierto por el registro de Q-X-2), AC-UTIL-016.12 y AC-UTIL-016.13. AC-UTIL-021 NO es exclusión: su equivalente es cargo-semver-checks, que entra en los gates a partir del primer release (anotado en la fila y en AGENTS.md). Las filas excluidas no cuentan como done: progress.md las muestra como excluded y calcula los porcentajes sobre las filas en alcance. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-12 — Alcance y representación de ByteString/ByteIterator/ByteStringBuilder

- **Estado:** aceptada
- **Evidencia:** scala-2.13/util/ByteString.scala:31,867,1164,1254 y ByteIterator.scala:26-430; ByteStringSpec.scala:360 (140 tests según inventario). Lo consumen IO (AC-IO-*) y serialización (AC-SER-*) que están en alcance. [Cierre documental 2026-10-31] Contrato extraído (AC-UTIL-006.1-.8, 022): ver bloques. Operaciones que condicionan la representación: recortes sin copia, `++` sin copia, `compact`/`isCompact`, `asByteBuffers` por fragmento, `toArrayUnsafe` sin copia en un fragmento, igualdad y `hashCode` por contenido, `toString` con tope de 100 bytes, serialización Java (identidades 0/1/2), builder con temporal y fragmentos, iterador mutable con `clone`.
- **Alternativas:** (a) newtype sobre `bytes::Bytes` (dependencia externa); (b) rope propio sin dependencias (el crate no tiene dependencias hoy); (c) ByteString = `Bytes` + traits para el builder/iterator.
- **Recomendación:** Decidir antes de empezar IO/SER; no bloquea UTIL/CFG/PATH/EXT. [Cierre documental 2026-10-31] Clasificación: ARQUITECTURA/dependencia externa (bytes, base64, encoding_rs). Con la extracción completada, la comparación es posible: (a) bytes::Bytes cubre recortes y clone baratos, un solo fragmento (isCompact deja de distinguir ropes; 'asByteBuffers' devolvería un solo trozo); (b) rope propio conserva fragmentos (isCompact, asByteBuffers y la fusión de ByteString1 contiguos son reproducibles) sin dependencias. Recomendación: (c) = Bytes + traits, con una lista de fragmentos sólo si IO la necesita (writev). Consecuencias observables: asByteBuffers/isCompact; alcance de charsets distintos de UTF-8. Bloquea: AC-UTIL-006.* y AC-UTIL-022 (impl) y, por dependencia, AC-IO-*/AC-SER-*; puede avanzar: toda UTIL/CFG/PATH/EXT sin ByteString.
- **Filas afectadas:** AC-UTIL-006.*, AC-UTIL-022
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (c): newtype sobre bytes::Bytes, con lista de fragmentos sólo si IO la necesita. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-13 — Helpers.isWindows deriva de `os.name` contiene "win"

- **Estado:** aceptada
- **Evidencia:** util/Helpers.scala:42; consumidores: LightArrayRevolverScheduler.scala:67,120, io/Tcp.scala:640, io/dns/DnsSettings.scala:113. [Cierre documental 2026-10-31] Medido con el jar en macOS (JDK 11): os.name = "Mac OS X" y Helpers.isWindows = false.
- **Alternativas:** (a) `cfg!(windows)` (comportamiento esperado); (b) reproducir la heurística de subcadena sobre `std::env::consts::OS`.
- **Recomendación:** (a). [Cierre documental 2026-10-31] Clasificación: representación (plataforma real). Consecuencias observables: sólo si un JVM informara os.name con 'win' sin ser Windows (ninguno conocido). Bloquea: nada; (a) es suficiente.
- **Filas afectadas:** AC-UTIL-026
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de 1.7.1, se sigue 2.0.0. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-14 — Reproducir scala.util.hashing.MurmurHash3 y los hashCode de Scala/Java donde un valor observable depende de ellos

- **Estado:** aceptada
- **Evidencia:** FrequencySketch.scala:79,201 usan MurmurHash3.stringHash(value, 135283237), mixLast y finalizeHash; FrequencySketchSpec.scala:25-90 afirma valores exactos (size == 6, size == 500 tras el reinicio) que sólo son estables con el hash exacto o sin colisiones; ByteString.hashCode es el de Seq[Byte] (medido: `ByteString("abc")` = -1431565476, vacío = 473519988); ImmutableIntMap.hashCode = Arrays.hashCode (medido 1281 para {10->10}); HashCode.scala (medido 856 para (23, 5)); Address/ActorPath hashCode (Q-PATH-3); FastFrequencySketch usa `value.hashCode` (String.hashCode de Java, UTF-16). scala-library no está en upstream/pekko: el comportamiento se conoce sólo por el jar.
- **Alternativas:** (a) portar MurmurHash3 (stringHash, mixLast, finalizeHash, seqHash) y String.hashCode de Java exactos y validarlos con un golden del jar; (b) usar cualquier hash y relajar los tests exactos a las cotas estadísticas que el spec ya usa en otros casos; (c) crate de hash externo sin garantía de igualdad con Scala.
- **Recomendación:** (a) para FrequencySketch/FastFrequencySketch (los tests de Pekko afirman valores exactos); (b) para ByteString/ImmutableIntMap/Address/ActorPath mientras ningún consumidor guarde o compare el valor numérico (coincide con Q-PATH-3 (a)). Consecuencias observables: sólo en estructuras de caché de cluster-sharding y en hashCode explícitos; con (b) los valores exactos del spec dejan de coincidir. Bloquea: AC-UTIL-017.1 (impl exacta); puede avanzar: el resto de AC-UTIL-017.* y AC-UTIL-006.*.
- **Filas afectadas:** AC-UTIL-017.1, AC-UTIL-006.1, AC-UTIL-016.4, AC-UTIL-016.15, AC-PATH-001, AC-PATH-014
- **Unidades afectadas:** UTIL, PATH
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a) para FrequencySketch, incluida la validación de desbordamiento de 2.0.0, y (b) para el resto. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-15 — MessageBufferMap.drop con un id ausente: NullPointerException de Pekko vs 0 mensajes

- **Estado:** aceptada
- **Evidencia:** MessageBuffer.scala:265-272: `val entries = bufferMap.get(id); if (entries.nonEmpty) ...` sin comprobar null; medido con el jar: `new MessageBufferMap[String].drop("x", "r", null)` lanza NullPointerException (antes de tocar deadLetters). Ningún test de MessageBufferSpec cubre `drop`. Consumidor: cluster-sharding Shard.scala (fuera de estos módulos).
- **Alternativas:** (a) reproducir el NPE (panic en Rust); (b) devolver 0 sin enviar nada y registrar la diferencia; (c) devolver Option/Result.
- **Recomendación:** (b): un NPE por id ausente es un defecto evidente (el resto de operaciones del mapa toleran ids ausentes: getOrEmpty/remove/contains) y ningún test lo afirma; pero cualquier diferencia con Pekko requiere autorización. Consecuencias observables: sólo cuando un consumidor llama a drop con un id que nunca tuvo mensajes. Bloquea: nada (AC-UTIL-010 puede cerrarse con (b) documentado); puede avanzar: todo AC-UTIL-010.
- **Filas afectadas:** AC-UTIL-010
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (b): devolver 0. Autoriza la divergencia con Pekko; queda documentada. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-16 — Representación del texto de origen de closures de Typed (equivalente de LineNumbers)

- **Estado:** aceptada
- **Evidencia:** LineNumbers.scala:28-76 (API) y su uso en BehaviorImpl.scala:117,131,149,171 e InterceptorImpl.scala:234,245 (toString `Receive(<archivo:líneas>)`); LineNumberSpec fija rangos (22-22, 24-24, 29-31, 35-36; Java 29-29, 34-35, 40-45) que dependen del compilador de Scala/Java; Rust no tiene bytecode.
- **Alternativas:** (a) #[track_caller] / Location::caller() en los constructores para mostrar `archivo:línea` (un solo número); (b) std::any::type_name::<F>() del closure; (c) excluir el texto (Debug opaco).
- **Recomendación:** (a) con fallback a (b). Consecuencias observables: el texto de Debug/Display de Behavior e interceptores deja de coincidir con Pekko (rangos y nombres de archivo de compilación); los tests portados de Typed que comparen ese texto deben fijar el formato propio. Bloquea: AC-UTIL-016.8 (impl) y los tests de toString de Typed; puede avanzar: el resto.
- **Filas afectadas:** AC-UTIL-016.8
- **Unidades afectadas:** UTIL
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar. (Owner (respuesta al lote 01), 2026-11-18)

### Q-UTIL-17 — toString / Display de PekkoException y ConfigurationException: con o sin nombre de clase

- **Estado:** aceptada
- **Evidencia:** Pekko 2.0.0 (oráculo EX-TS-01..03, casos medidos el 2026-11-18): `new PekkoException("boom").toString` = `org.apache.pekko.PekkoException: boom`; `ConfigurationException` = `org.apache.pekko.ConfigurationException: bad`; con mensaje vacío `org.apache.pekko.PekkoException: `. Hoy el `Display` de Rust imprime sólo el mensaje (`java_class()` da el nombre simple).
- **Alternativas:** (a) `Display` = sólo el mensaje, como es convención en Rust, y `java_class()`/`java_name()` para quien quiera el nombre; (b) `Display` = `<nombre calificado JVM>: <mensaje>`, como `Throwable.toString`.
- **Recomendación:** (a): lo que consumen los usuarios de un crate Rust es el mensaje; el nombre de clase está disponible aparte. Es una diferencia observable con Pekko: requiere autorización del Owner (A-4); sin respuesta AC-UTIL-004.1 queda `partial`.
- **Filas afectadas:** AC-UTIL-004.1; AC-UTIL-004.2
- **Unidades afectadas:** P02
- **Origen:** classic-foundation
- **Respuesta del Owner:** aceptar (a): Display de PekkoException y ConfigurationException muestra sólo el mensaje; java_class() queda aparte. (Owner (respuesta tras P02), 2026-11-18)

