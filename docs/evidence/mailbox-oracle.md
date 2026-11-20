# Oráculo del mailbox de Pekko 2.0.0 (ADR-0001, PR 1/10)

Caracterización **oracle-only**: no hay `MailboxStatus` ni `MessageQueue` en Rust, de modo que estos casos **no** entran en `rust-anything app replay` (un caso grabado divergiría por construcción) y no declaran paridad Rust. Los consumirán los PR 2–6 de la secuencia.

- Fuente de verdad: Apache Pekko **2.0.0**, `9fbd2283b1cf701c90a1223bf621942c191db61f`, Scala 2.13.18, JDK 17 (`oracle/run.sh`).
- Comandos: `oracle/run.sh mailbox-ids | mailbox-all | mailbox-case` (código: `oracle/src/main/java/pilot/MailboxOracle.java`).
- Golden (lo que imprimió la JVM): `cases/oracle-only/mailbox.golden`; por caso: `cases/oracle-only/mailbox-cases.tsv` (id, capa, contratos del inventario, cita upstream, instrumento).
- Verificación: `python3.12 tools/parity/check_mailbox_oracle.py` (3 corridas idénticas entre sí y con el golden; `--record` regenera; `--selftest` perturba el golden y debe salir con 1). Tests del verificador: `python3.12 tools/parity/test_check_mailbox_oracle.py`.

## Instrumentos (sin tocar Pekko, sin reflexión)

1. **`Probe`**: un `Mailbox` real con `DefaultSystemMessageQueue` (`DefaultSystemMessageQueue.super.*`, como lo arma `MessageDispatcher.createMailbox`), con `actor == null`, que Pekko permite para un mailbox que no se ejecuta. Observa estado y cola de sistema. *Límite:* con `actor == null`, `systemEnqueue` tras el centinela `NoMessage` descarta en vez de reenviar al `deadLetterMailbox`; ese camino se observa con actores reales (OR-05, CC-04).
2. **`HookQueue`**: `MessageQueue` público (FIFO, igual a la cola por defecto) que ejecuta una función del caso en `enqueue`, `dequeue` y `cleanUp`. Fuerza cada interleaving en un punto exacto del protocolo (p. ej. el último `dequeue` vacío de un turno, antes de `setAsIdle`). No cambia ninguna decisión de Pekko.

Sincronización: sólo se espera una condición observable (evento, mailbox ocioso, `cleanUp` terminado) con techo de 10 s. La ausencia de procesamiento se prueba llegando a un mailbox ocioso cuya cola conserva el mensaje. Único uso de tiempo real: SC-02 (mensajes de 60 ms contra un deadline de 30 ms; el margen es el propio contrato de `throughput-deadline-time`).

## Condiciones de ejecución y normalización

Dispatcher `t-disp`: `thread-pool-executor` con `fixed-pool-size = 1` (FIFO, un hilo), `throughput` por caso (5 salvo SC-01 = 2 y SC-02 = 100 con deadline 30 ms); actores internos y listener en el dispatcher por defecto. **Normalización: ninguna.** El orden de mensajes, las pérdidas y los estados de cierre se imprimen tal cual. `raw=` es la codificación entera de Pekko (informativa; el contrato es la tabla de transiciones del ADR §7b).

Estabilidad: 25 corridas idénticas (8 JVM concurrentes incluidas). Dos fuentes de no-determinismo reales detectadas y corregidas durante la construcción: (a) `Mailbox.isClosed` es verdadero **antes** de que termine `cleanUp` (se espera ahora la señal de fin de `cleanUp`); (b) el primer `dequeue` vacío ocurre en el turno de creación (el hook se ata a un mensaje ya entregado).

## Hallazgos frente al ADR-0001 (el upstream manda)

| # | ADR | Observado en 2.0.0 |
|---|---|---|
| 1 | §7b carrera 5: "o se procesa o va a dead letters; nunca ambos" | **CC-05**: un productor que ya leyó el mailbox antes de `swapMailbox` y encola después de `cleanUp` deja el mensaje **varado** en la cola vieja cerrada: ni procesado ni dead letter (`dispatch` lee `receiver.mailbox` y luego encola, Dispatcher.scala:80-84; `unregister` hace `swapMailbox` → `becomeClosed` → `cleanUp`, AbstractDispatcher.scala:282-284; `enqueue` no comprueba el cierre). Existe una tercera salida; el diseño Rust debe decidir si la replica (paridad) o la cierra (decisión del Owner, no se cambia aquí). |
| 2 | §1 "Watch sobre cerrado" | **OR-05**: `Suspend`/`Resume` tras el cierre se publican como `DeadLetter`; un `Watch` **no**: lo contesta el `deadLetters` con `Terminated` al observador. |
| 3 | §7b `becomeClosed` borra `scheduled` y suspensión | Confirmado (ST-05: `raw=10 → 1`). `setAsIdle` sobre `Closed` devuelve `true` y deja `Closed` (ST-07). |
| 4 | §1 `canBeScheduled` | Confirmado (ST-08): suspendido sólo con mensajes de sistema; cerrado nunca. |
| 5 | §5 shim para instanciar `Mailbox` | Basta una subclase Java con `DefaultSystemMessageQueue.super.*`; no hace falta Scala ni reflexión. |

Otros hechos: `resume` en 0 devuelve `true` y no baja de 0 (ST-03); sistema y usuario: el `Suspend` encolado **después** de mensajes de usuario se procesa **antes** de ellos tras el turno en vuelo (OR-02); con `throughput = 2` dos actores en un hilo alternan en bloques de 2 (SC-01) y con deadline la ráfaga termina tras el primer mensaje lento (SC-02); `cleanUp` envía primero lo que el hook reenvió y luego lo pendiente (CC-04).

## Escenarios del encargo y cobertura

A (transiciones): ST-01…ST-08, CL-01. B (orden): OR-01…OR-05, SQ-01/02. C (carreras §7b): 1→CC-02, 2→CC-01, 3→CC-03, 4→CC-04, 5→CC-05. D (scheduling): CC-01, SC-01…SC-03, OR-04.

## Pendiente (no inventado)

- §7b carrera 6 (ejecución que entra en pánico): caso **nuevo de Rust**, sin oráculo (`MailboxTaskSubmissionSpec` sólo fija TK-01); se cubre en el PR 6.
- Rechazo del pool y reintento único de `registerForExecution` (D5, Dispatcher.scala:146-171): requiere un executor que rechace; PR 6.
- Doble agendado forzado en el instante exacto del CAS: sólo se observa el invariante (CC-01, `maxInFlight=1`); no hay punto de enganche público en `setAsScheduled`.
- Colas acotadas, prioridad, control-aware, deque/stash, `Mailboxes`, `UnstartedCell`: PR 7–10.
- Cobertura y mutantes de producción: no se repiten (no cambia código Rust de `crates/`).
