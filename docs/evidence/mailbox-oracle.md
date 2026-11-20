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

Estabilidad: 25 corridas idénticas (8 JVM concurrentes incluidas) y 32 más, bajo carga, en la revisión independiente. Dos fuentes de no-determinismo reales detectadas y corregidas durante la construcción: (a) `Mailbox.isClosed` es verdadero **antes** de que termine `cleanUp` (se espera ahora la señal de fin de `cleanUp`); (b) el primer `dequeue` vacío ocurre en el turno de creación (el hook se ata a un mensaje ya entregado).

## Hallazgos frente al ADR-0001 (el upstream manda)

| # | ADR | Observado en 2.0.0 |
|---|---|---|
| 1 | §7b carrera 5: "o se procesa o va a dead letters; nunca ambos" | **CC-05/CC-05a** y el experimento natural: existe una tercera salida, **varado**, reproducible sin ganchos (ver «CC-05: ventana de cierre»). Clasificada como carrera sin garantía contractual; decisión **aceptada por el Owner el 2026-11-20** (Q-MBX-8: B + C). |
| 2 | §1 "Watch sobre cerrado" | **OR-05**: `Suspend`/`Resume` tras el cierre se publican como `DeadLetter`; un `Watch` **no**: lo contesta el `deadLetters` con `Terminated` al observador. |
| 3 | §7b `becomeClosed` borra `scheduled` y suspensión | Confirmado (ST-05: `raw=10 → 1`). `setAsIdle` sobre `Closed` devuelve `true` y deja `Closed` (ST-07). |
| 4 | §1 `canBeScheduled` | Confirmado (ST-08): suspendido sólo con mensajes de sistema; cerrado nunca. |
| 5 | §5 shim para instanciar `Mailbox` | Basta una subclase Java con `DefaultSystemMessageQueue.super.*`; no hace falta Scala ni reflexión. |

Otros hechos: `resume` en 0 devuelve `true` y no baja de 0 (ST-03); sistema y usuario: el `Suspend` encolado **después** de mensajes de usuario se procesa **antes** de ellos tras el turno en vuelo (OR-02); con `throughput = 2` dos actores en un hilo alternan en bloques de 2 (SC-01) y con deadline la ráfaga termina tras el primer mensaje lento (SC-02); `cleanUp` envía primero lo que el hook reenvió y luego lo pendiente (CC-04).

## CC-05: ventana de cierre (informe)

**Secuencia en Pekko 2.0.0.** Un emisor ejecuta `Dispatcher.dispatch` (Dispatcher.scala:80-84): lee `receiver.mailbox` (volátil), llama `mbox.enqueue` (sin comprobar cierre) y luego `registerForExecution`. En paralelo, `unregister` (AbstractDispatcher.scala:279-285) hace `swapMailbox(deadLetterMailbox)` → `becomeClosed()` → `cleanUp()`, y `cleanUp` drena la cola vieja a dead letters (Mailbox.scala:354-368). Un emisor que ya leyó la referencia vieja y cuyo `add` ocurre **después** del drenaje deja el mensaje en una cola cerrada y sin dueño: no se procesa (`canBeScheduledForExecution` es falso para `Closed`) ni llega a dead letters.

**Evidencia JVM.**
- Forzada y determinista (golden): **CC-05** (el `add` ocurre tras `cleanUp`: `received=[] deadLetters=[] queued=1`) y **CC-05a** (el mismo emisor, pero el `add` ocurre justo antes del drenaje: `deadLetters=[straggler]`). El único factor que cambia el resultado es el orden entre `q.add` y el drenaje.
- Natural, sin ganchos (no entra en el golden: sus conteos varían): `oracle/run.sh mailbox-stress <<< N`. Por ensayo: un actor, 4 emisores × 300 envíos liberados junto con `stop`. Ejecuciones con SHA upstream `9fbd2283…`, JDK 17, Mac de 16 núcleos; la corrida 1 se hizo sola y las corridas 2–4 como 3 JVM concurrentes:

| Corrida | Ensayos | Envíos | Recibidos | Dead letters | Varados | Ensayos con varado | No contabilizados |
|---|---|---|---|---|---|---|---|
| 1 | 3 000 | 3,6 M | 297 130 | 3 302 870 | 0 | 0 | 0 |
| 2 | 30 000 | 36 M | 3 677 991 | 32 322 008 | 1 | 1 | 0 |
| 3 | 30 000 | 36 M | 4 158 270 | 31 841 729 | 1 | 1 | 0 |
| 4 | 30 000 | 36 M | 3 839 681 | 32 160 317 | 2 | 2 | 0 |

  En los 93 000 ensayos (111,6 M de envíos): 4 mensajes varados (1 cada ~2,8·10^7 envíos), recibidos + dead letters + varados = enviados en todos los ensayos. El hallazgo **no es un artefacto de los ganchos**. Límites: la frecuencia depende de máquina y carga; el experimento cuenta los varados leyendo `numberOfMessages` de la cola vieja tras `cleanUp`, no distingue cuál emisor ni su hilo.

**Garantías públicas.** El doc de Pekko (`general/message-delivery-reliability.md:315-321`) dice que la entrega a dead letters es «best-effort; it may fail even within the local JVM (e.g. during actor termination)» y que sirve para depuración, no para entrega garantizada; la regla general es at-most-once (:41). Un mensaje varado no viola ninguna garantía publicada. Lo que Pekko sí garantiza (y el corpus ya valida con `await-accounted` en escenarios secuenciales): lo enviado con *happens-before* anterior al cierre se recibe o es dead letter, sin duplicar.

**Clasificación (evidencia, no decisión).** Carrera interna upstream, observable en la práctica, **sin garantía contractual** (ni un error del oráculo —reproducida con y sin ganchos— ni algo que Rakka deba reproducir por contrato). La reproducción exacta no es exigida por ningún contrato.

**Alternativas.**
- **A — replicar** (cola vieja sin comprobar cierre): reproduce el varado, pero ningún contrato lo exige y degrada la depuración con dead letters; el piloto Rust hoy tiene el comportamiento opuesto.
- **B — cerrar la ventana** (comprobar `closed` y encolar de forma atómica; el piloto lo hacía bajo el `Mutex` único de `cell.rs`; desde el PR 4a lo garantiza `dispatch::mailbox_close::enqueue_unless_closed`): Rust produce siempre dead letter para ese emisor. Observable que cambia: desaparece el resultado «varado», un subconjunto de lo que Pekko puede producir; no es una violación de paridad estricta porque Pekko no lo garantiza. Con el diseño atómico del ADR (PR 4) hay que conservarlo: comprobar el estado y encolar sin dejar un hueco.
- **C — aceptar el conjunto**: el oráculo admite {recibido, dead letter, varado} para el emisor en vuelo y exige las invariantes que sí valen (nunca procesar tras el cierre, nunca duplicar, nada se pierde si hay *happens-before*). Validación sin ocultar errores: contabilidad exacta `recibidos + dead letters + varados = enviados`, con el varado tolerado sólo en el lado Pekko.

**Estado en Rust (PR 4a, #303):** `dispatch::mailbox_close::enqueue_unless_closed` encola y vuelve a comprobar el cierre (el cierre marca `Closed` antes de vaciar la cola); la cola-doble que cierra a mitad de camino falla con la versión ingenua y pasa con la correcta (`mailbox_close`), y `tests/classic_mailbox_close.rs` (400 ensayos × 4 emisores × 300 envíos con actores reales) falló 3 de 3 veces con la versión ingenua y pasa con el protocolo.

**Recomendación:** implementar **B** y validar con **C** (`Q-MBX-8`, **aceptada por el Owner el 2026-11-20**: B + C, registrada en `docs/decisions/owner-decisions.tsv`). Consecuencia para Rust: el diseño del PR 4 no debe introducir un `enqueue` sin comprobación de cierre; una cola de usuario separada del estado (D4) exige un protocolo que cierre la ventana (p. ej. comprobar el estado tras encolar y reenviar a dead letters si quedó cerrado).

## Escenarios del encargo y cobertura

A (transiciones): ST-01…ST-08, CL-01. B (orden): OR-01…OR-05, SQ-01/02. C (carreras §7b): 1→CC-02, 2→CC-01, 3→CC-03, 4→CC-04, 5→CC-05. D (scheduling): CC-01, SC-01…SC-03, OR-04.

## Pendiente (no inventado)

- §7b carrera 6 (ejecución que entra en pánico): caso **nuevo de Rust**, sin oráculo (`MailboxTaskSubmissionSpec` sólo fija TK-01); se cubre en el PR 6.
- Rechazo del pool y reintento único de `registerForExecution` (D5, Dispatcher.scala:146-171): requiere un executor que rechace; PR 6.
- Exclusión mutua de turnos: CC-01 corre en un pool de un hilo, así que `maxInFlight=1` no prueba por sí solo el flag `Scheduled` (sólo ausencia de pérdida, orden por emisor y ejecución serial). Falta una variante en pool multi-hilo, no determinista en el hilo, y un punto de enganche en `setAsScheduled`. CC-03 ejerce el CAS de `systemEnqueue` desde dos hilos; `Mailbox.suspend` lo llama sólo el hilo del actor.
- Los esperas sin tope (`await` de latches propios) pueden colgar la corrida si Pekko regresara; el verificador las corta a los 300 s.
- Colas acotadas, prioridad, control-aware, deque/stash, `Mailboxes`, `UnstartedCell`: PR 7–10.
- Cobertura y mutantes de producción: no se repiten (no cambia código Rust de `crates/`).
