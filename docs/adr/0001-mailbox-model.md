# ADR-0001 (propuesta) — Modelo de mailbox de Rakka Classic frente a Pekko 2.0.0

- **Estado:** propuesta. Sin decisión del Owner. No modifica el runtime.
- **Issue:** getsyntegrity/rakka#282 (02/09). Resuelve en propuesta Q-MBX-6 y Q-MBX-1/2/3 (y toca Q-MBX-5).
- **Fuente de verdad:** Apache Pekko 2.0.0 (`9fbd2283b1cf701c90a1223bf621942c191db61f`). Citas `path:line` sobre `actor/src/main/scala/org/apache/pekko/dispatch/`.
- **Base del piloto:** `feat/pekko-rust-pilot-migration` @ `cb39023`; `crates/rakka-actor/src/actor/cell.rs`, `dispatch/dispatcher.rs`.
- **Verificación:** citas de Mailbox.scala/Mailboxes.scala y D1–D7 contrastadas contra el submódulo en `9fbd2283…` y el código del piloto (ver §8).

## 1. Contrato observable de Pekko 2.0.0

| Aspecto | Pekko | Cita |
|---|---|---|
| Estado | Un `Int`: primario `Open=0`/`Closed=1`, bit `Scheduled=2`, contador de suspensión en unidades de 4. Todo con CAS. | Mailbox.scala:40-56,125-237 |
| `Closed` es terminal | `suspend`/`resume`/`becomeClosed` sobre `Closed` no cambian nada (`resume` no baja de 0). | :160-211 |
| Agendado | `setAsScheduled` sólo si `(s & 3) == Open`: un mailbox **suspendido** sí puede agendarse (para mensajes de sistema); uno cerrado o ya agendado no. | :216-225 |
| `canBeScheduledForExecution` | Open/Scheduled: hay mensaje (hint o real) o de sistema; Closed: nunca; suspendido: sólo de sistema. | :255-260 |
| Turno (`run`) | Si no está cerrado: `processAllSystemMessages`, luego `processMailbox`. `finally`: `setAsIdle` y `registerForExecution` sin hints. | :262-272 |
| `processMailbox` | Hasta `max(throughput,1)` mensajes y corte por `throughput-deadline-time`; tras **cada** mensaje de usuario drena todo el sistema. Sólo procesa si no está suspendido ni cerrado. | :277-294 |
| Cola de sistema | Lista LIFO con CAS, drenada invertida (más antiguo primero). Tras `cleanUp` queda el centinela `NoMessage`: todo `systemEnqueue` posterior va al `deadLetterMailbox`. | :454-527 |
| Cierre | Los de sistema pendientes tras cerrar van al `deadLetterMailbox` (esencial para DeathWatch); `cleanUp` mueve los de usuario a dead letters. | :303-368 |
| `MessageQueue` (SPI) | `enqueue/dequeue/numberOfMessages/hasMessages/cleanUp`; N productores, 1 consumidor. | :376-410 |
| Acotado | `offer` con `pushTimeOut` (≥0) o `put` (<0, bloquea sin límite); vencido → `DeadLetter`. `NonBlockingBounded`: descarta ya. | :557-595,435-463 |
| Control-aware | Dos colas; `Envelope(_: ControlMessage, _)` va a la de control, que se desencola primero (FIFO por clase). Acotado: capacidad compartida, `awaitNanos`. | :897-1049 |
| Dispatcher | `dispatch` = encolar y `registerForExecution(hasMessageHint=true)`; `systemDispatch` igual con hint de sistema; reintento único ante `RejectedExecutionException`. | Dispatcher.scala:80-171 |
| `deadLetterMailbox` | `Mailbox` cerrado (`becomeClosed()`) compartido: `enqueue` de usuario reenvía `DeadLetter(msg, sender, receiver)` salvo que el mensaje ya sea `DeadLetter` (se descarta); `systemEnqueue` hace `deadLetters ! DeadLetter(handle, receiver, receiver)`; `systemDrain` siempre vacío; `dequeue()=null`. | Mailboxes.scala:54-72 |
| Cola por defecto | `UnboundedMailbox` = `ConcurrentLinkedQueue` (multi-consumidor); `SingleConsumerOnlyUnbounded` y `NonBlockingBounded` usan `NodeMessageQueue`/`BoundedNodeMessageQueue`. Las de prioridad acotadas usan `BoundedBlockingQueue` sobre `PriorityQueue`/`StablePriorityQueue`. Capacidad negativa → `IllegalArgumentException`. | Mailbox.scala:677-846 |
| Creación en dos pasos | Un actor de nivel superior puede recibir mensajes en una cola provisional (`UnstartedCell`) antes de que exista el mailbox real; se transfieren al apuntar (`point()`). Cualquier `MailboxType` custom debe tolerarlo. | Mailbox.scala:655-667; AC-REF-020 |
| 2.0.0 nuevo | `Mailbox` **no** debe ser una `ForkJoinTask` reenviada (se reenvía un objeto por toda la vida del actor); fijado por `MailboxTaskSubmissionSpec`. | Mailbox.scala:62-75 |

## 2. Estado del piloto

`ActorCell.mb: Mutex<MbState { sys: VecDeque<SystemMessage>, user: VecDeque<Envelope>, suspend: u32, closed: bool, scheduled: bool }>` (cell.rs:29-35,83).

Lo que ya coincide en lo observable:
- Sistema antes que usuario y drenaje completo tras cada mensaje de usuario (cell.rs:245-288).
- Suspensión que impide el usuario pero no el sistema (`can_run`, cell.rs:190-192; `enqueue_user`, :212-229).
- `Closed` terminal; usuario sobre cerrado → dead letter (:212-218); `Watch` de sistema sobre cerrado → `DeathWatchNotification` (:231-241).
- Una única ejecución simultánea por actor; se agenda a lo sumo una vez (flag `scheduled` bajo lock).

Ya existen, **sin cablear al mailbox**, bloques de construcción en `src/util/`: `bounded_blocking_queue` y `stable_priority_queue` (insumo de D1, PR 7–8).

Divergencias y huecos (todas hoy invisibles porque el piloto sólo usa la cola FIFO no acotada):

| # | Hueco | Efecto | Fila |
|---|---|---|---|
| D1 | No hay `MessageQueue`/`MailboxType`/`Mailboxes`; `VecDeque<Envelope>` está cableada. | Imposible bounded, deque (stash), prioridad, control-aware, requisitos. | AC-MBX-004..015,018,020 |
| D2 | `throughput` es global (`system.settings.throughput`); no hay `throughput-deadline-time` ni throughput por dispatcher. | Equidad de `DispatcherActorSpec` no reproducible. | AC-MBX-002, AC-DSP-003 |
| D3 | No hay `deadLetterMailbox`; `dead_system_message` sólo conoce `Watch`. | Los demás de sistema sobre cerrado se descartan en silencio; Pekko los reenvía como `DeadLetter`. | AC-MBX-003/021 |
| D4 | Estado y cola bajo un único `Mutex`. | Con colas acotadas bloqueantes el emisor bloquearía bajo el lock del consumidor: **deadlock**. Pekko separa cola (propia) de estado (CAS). | AC-MBX-007 |
| D5 | `Dispatcher::execute` ignora el error de `send` (dispatcher.rs:44-48). | Tras `shutdown`, `scheduled` queda en `true` y el mailbox no vuelve a agendarse; Pekko reintenta y reporta. | AC-DSP-002 |
| D6 | Un único canal `mpsc` con `Mutex<Receiver>`; sin `PinnedDispatcher`, `internal-dispatcher` ni pools configurables. | Sin paridad de aislamiento; contención en un solo lock. | AC-DSP-005/008/011 |
| D7 | El lock de estado se toma ≥3 veces por mensaje (drenaje de sistema, chequeo de cierre, `pop`). | Sólo rendimiento; no observable. | — |

## 3. Opciones

**A — Conservar `Mutex<MbState>` y sólo añadir cola abstracta.** Mínimo cambio, pero D4 impide el bounded con timeout (bloqueo dentro del lock) y no hay CAS de estado; descartada salvo como paso intermedio.

**B — Separar estado, cola de sistema y cola de usuario (recomendada).**
- `MailboxStatus(AtomicU32)` con la **misma codificación** de Pekko y bucles CAS (sin `unsafe`).
- Cola de sistema: `Mutex<Vec<SystemMessage>>` invertida al drenar + centinela de cierre (Q-MBX-5 a; evita un Treiber con `unsafe`).
- `trait MessageQueue` (objeto `dyn`, `Send + Sync`) con las cinco operaciones de Pekko y `enqueue_first` en un sub-trait deque.
- `Dispatcher` con `register_for_execution(mbox, hints)` idéntico al de Pekko, throughput y deadline por dispatcher, reloj inyectable.
- Mantiene Q-X-4 (A): el turno es un trabajo del pool propio; `tell` bloqueante sólo vale en hilo de pool/externo.

**C — Mailbox como tarea async.** Contradice Q-X-4 (A) (aceptada por el Owner 2026-11-18); sólo como adaptador opcional futuro.

Riesgos de B: más superficie atómica (mitigada con tests de estado puros y diferenciales); el timeout bloqueante en un worker del pool puede agotar el pool (igual que Pekko; documentar); el mailbox **no** debe ser un tipo con semántica propia de tarea de pool (en Pekko, no `ForkJoinTask`; `MailboxTaskSubmissionSpec` sólo fija eso). Pekko reenvía el mismo objeto `Mailbox` durante toda la vida del actor; en Rust se reenvía un `Arc` del mailbox, sin estado de tarea propio.

## 4. Costuras (seams) propuestas
1. `MailboxStatus` — puro, sin actores: tabla de transiciones.
2. `MessageQueue`/`MailboxType` — sin dispatcher: cada cola probada aislada (unitario, política de testing).
3. `DeadLetterMailbox` — un `Mailbox` cerrado compartido.
4. `Dispatcher::register_for_execution` y `Clock` inyectable — fairness/deadline deterministas.
5. Integración: `ActorCell` con las anteriores; el piloto conserva su API (`LocalActorRef::tell/send_system`).

## 5. Escenarios diferenciales contra el oráculo JVM
Observables sin acceso privado a Pekko (el oráculo Java ya usa `ActorSystem`; `Mailbox` es `private[pekko]` pero público en bytecode):
1. Transiciones de estado: secuencias `suspend/resume/close/scheduled` y sus retornos (AC-MBX-001). *Riesgo:* `Mailbox` se instancia con un mixin Scala (`with DefaultSystemMessageQueue`); desde Java hace falta un shim o medir vía comportamiento.
2. Orden sistema/usuario: `Suspend`, mensajes, `Resume` → traza de recepción.
3. Fairness: `throughput=101` con 1 hilo y `throughput=2` con deadline 100 ms (`DispatcherActorSpec.scala:84-141`).
4. Cierre: mensajes pendientes y de sistema → dead letters (AC-MBX-003/021).
5. Bounded: capacidad llena con timeouts −1/0/100 ms (`MailboxConfigSpec.scala:56-92`).
6. Control-aware y prioridad/estable (`ControlAwareDispatcherSpec`, `PriorityDispatcherSpec`, `StablePriorityDispatcherSpec`).
7. Deque: `enqueue_first` vía Stash (`ActorWithBoundedStashSpec.scala:119-133`).

## 6. Secuencia de PR (≤400 líneas autoradas cada uno; ninguno rompe el piloto)
1. Oráculo + casos grabados de transiciones de estado y orden sistema/usuario (sin runtime).
2. `MailboxStatus` puro con tests unitarios (RED→GREEN) y replay.
3. `trait MessageQueue` + `UnboundedMailbox` por defecto; `ActorCell` la usa (refactor sin cambio observable; D1 base).
4. Estado atómico en `ActorCell` + cola de sistema separada con centinela (cierra D4 parcial).
5. `deadLetterMailbox` (D3, AC-MBX-003/021).
6. `register_for_execution` con reintento + throughput/deadline por dispatcher + reloj inyectable (D2/D5; AC-DSP-002/003).
7. Colas acotadas (`BoundedMailbox`, `NonBlockingBounded`) — tras Q-MBX-1.
8. Deque + stash; prioridad (+ estable).
9. Control-aware — tras Q-MBX-3.
10. `Mailboxes` (registro, `requirements`, lookup) — tras Q-MBX-2.

## 7. Preguntas exactas para el Owner (propuestas; se registrarán como `proposed`)

Las letras (a, a′, b, c) remiten a las opciones de cada `Q-MBX-n` en `docs/inventory/contracts/classic-dispatch.decisions.tsv`.

1. **Q-MBX-6.** Dado Q-X-4 (A), ¿se confirma que el turno de mailbox es un trabajo del pool propio, con `throughput` y `throughput-deadline-time` **por dispatcher** y reloj inyectable? (sí/no)
2. **Q-MBX-1.** ¿Se aprueba (c)? Concretamente: (i) nombre de la API awaitable (`try_tell` o `tell_async`); (ii) `tell` con `push-timeout>0` bloquea el hilo del emisor (paridad) también dentro de un worker del pool, con aviso de agotamiento del pool; (iii) `<0` espera sin límite como Pekko.
3. **Q-MBX-2.** ¿Se aprueba (a′) (conjunto abierto de semánticas con implicancia codificada)?
4. **Q-MBX-3.** ¿Se aprueba (b)+(c) (registro por `TypeId` en el sistema + predicado en el `MailboxType`)? ¿Cómo declara el usuario un mensaje de control: por registro de tipo o por un trait opcional en el sobre?
5. **Q-MBX-5.** ¿Se acepta `Mutex<Vec>` invertido (sin `unsafe`) para la cola de sistema, aun con peor contención que el Treiber de Pekko?
6. **Secuencia.** ¿Se aprueba la opción B y el orden de PR de la sección 6 (con el PR 1 antes de tocar runtime)?
7. **Oráculo.** ¿Se autoriza un shim Scala/Java mínimo en `oracle/` para instanciar `Mailbox` y medir transiciones de estado, o se prefiere medir sólo por comportamiento?

Decisiones nuevas se anotan como `Q-MBX-n` con estado `proposed` en `docs/owner-batch-02.md` y su `*.decisions.tsv`; `docs/decisions/owner-decisions.tsv` no se toca.

## 7b. Diseño de `MailboxStatus`: transiciones observables y carreras

La codificación de Pekko es un detalle; lo que Rust debe preservar son las transiciones observables. Sea `S = (primario ∈ {Open, Closed}, scheduled: bool, suspend: u32)`.

| Operación | Efecto | Retorno | Cita |
|---|---|---|---|
| `suspend` | `suspend += 1`; sobre `Closed`: sin cambio | `true` si estaba en 0 y no cerrado | Mailbox.scala:181-193 |
| `resume` | `suspend -= 1` con piso en 0; sobre `Closed`: sin cambio | `true` si queda en 0 y no cerrado | :160-173 |
| `becomeClosed` | estado ← `Closed` exacto (`updateStatus(s, Closed)`): terminal y **borra** el bit `scheduled` y el contador de suspensión (`isClosed` es `currentStatus == Closed`) | `true` sólo la primera vez | :199-211 |
| `setAsScheduled` | `scheduled ← true` **sólo si** no cerrado y no agendado (suspendido sí) | `true` si lo cambió | :216-225 |
| `setAsIdle` | `scheduled ← false` (incondicional) | `true` | :230-237 |
| `canBeScheduled(msgHint, sysHint)` | ver §1 | bool | :255-260 |

Invariantes a probar: (I1) `Closed` absorbe todo; (I2) a lo sumo un turno en vuelo por mailbox (`setAsScheduled` es la única puerta); (I3) `suspend`/`resume` concurrentes no pierden incrementos (CAS); (I4) tras `run`, `setAsIdle` seguido de `registerForExecution` reprograma si quedó trabajo, de modo que un mensaje encolado durante el cierre del turno no queda varado.

Carreras que los escenarios deben forzar (con inyección de pausas/barreras, no con sleeps):
1. **Encolar vs. fin de turno:** productor encola entre el último `dequeue` vacío y `setAsIdle` → debe re-agendarse (I4).
2. **Doble agendado:** N productores llaman `registerForExecution` a la vez → un solo turno (I2).
3. **`suspend` durante un turno y `resume` concurrente:** el contador termina balanceado; el actor reanuda sólo al llegar a 0.
4. **`becomeClosed` vs. `systemEnqueue`:** el mensaje de sistema llega al `deadLetterMailbox`, nunca se pierde (centinela `NoMessage`).
5. **`becomeClosed` vs. `enqueue` de usuario:** o se procesa o va a dead letters; nunca ambos. Que no haya ventana en la que se pierda depende del orden `swapMailbox`/`cleanUp` de Pekko: el escenario debe **observarlo** en el oráculo, no suponerlo.
6. **Ejecución que entra en pánico:** el mailbox vuelve a ser agendable (caso **nuevo** de Rust; `MailboxTaskSubmissionSpec` no lo prueba: su reenvío tras fallo usa un `Runnable` simple, no `Mailbox`). `run` limpia `scheduled` en `finally`.

Con `AtomicU32` el diseño usa la misma máscara que Pekko por simplicidad, pero el contrato probado es la tabla anterior, no los valores numéricos.

## 7c. Propuestas por decisión (recomendaciones, **no** decisiones aprobadas)

| Q | Recomendación | Garantía/riesgo clave |
|---|---|---|
| Q-MBX-6 | Turno de mailbox = trabajo del pool propio (Q-X-4 A, ya aceptada); `throughput`/`deadline` por dispatcher, `Clock` inyectable. | Equidad medible con 1 hilo; riesgo: pool agotado por emisores bloqueados. |
| Q-MBX-1 | Paridad bloqueante de Pekko en `tell`; añadir API awaitable **adicional** (no sustituye). El bloqueo ocurre **fuera** de todo lock de estado. | Evita deadlock: la cola acotada tiene su propia sincronización; el estado es atómico (D4). Un `put` sin timeout (<0) puede bloquear para siempre, como en Pekko. |
| Q-MBX-2 | Conjunto abierto de semánticas con implicancia codificada (a′). | Necesario por el marcador de usuario de `ActorMailboxSpec`. |
| Q-MBX-3 | Registro explícito de tipos de control + predicado en `MailboxType`; sujeto a validar contra `ControlAwareDispatcherSpec`. | Sólo los tipos registrados son control. |
| Q-MBX-5 | `Mutex<Vec>` invertido al drenar, con centinela de cierre, sin `unsafe`; se acepta si preserva el orden más-antiguo-primero y el desvío al `deadLetterMailbox`. | Más contención que el Treiber; se mide antes de cambiar. |

Conservación del piloto: cada PR mantiene `LocalActorRef::tell/send_system` y el replay verde; el `Mutex<MbState>` actual sigue siendo la implementación hasta el PR 4, y sólo se reemplaza tras tener los diferenciales (PR 1) y `MailboxStatus` (PR 2).

Oráculo: probablemente haga falta un shim mínimo (clase Scala/Java en `oracle/`) para instanciar `Mailbox` con `DefaultSystemMessageQueue` y observar transiciones; alternativa sin shim: medir sólo por comportamiento de actores (menos preciso para I1–I4). Decisión del Owner (pregunta 7).

Alcance de este PR: sólo documentación (`docs/adr/0001-mailbox-model.md`). `scripts/wt.sh` **no** forma parte de la decisión y debe retirarse de este PR (ver issue aparte).

## 8. Verificaciones

Confirmado (salida de shell del Owner, 2026-11-20):
- **Cierre/`detach`** (cell.rs:668-686, `finish_terminate`): bajo el lock se marca `closed = true` y se vacían `sys` y `user`; cada mensaje de sistema pasa por `dead_system_message` (sólo `Watch` produce efecto; el resto se descarta) y cada mensaje de usuario va a `publish_dead_letter`. Confirma D3: no hay `deadLetterMailbox` ni reenvío de los demás mensajes de sistema.
- **Dispatchers** (system.rs:266-268): se crean exactamente dos, `Dispatcher::start(name, settings.parallelism)` e `internal-dispatcher` con 4 hilos fijos. `throughput` es un único `usize` en `Settings` (system.rs:65, default 5); no hay throughput por dispatcher ni deadline (confirma D2, D6).
- **`MailboxTaskSubmissionSpec`**: fija sólo que `Mailbox` no es un `ForkJoinTask` (:73-76) y que el pool conserva la cancelación/finalización de un `ForkJoinTask` enviado como `Runnable`; su caso de reenvío tras fallo usa un `Runnable` de control, no `Mailbox`. Para Rust: una ejecución que entra en pánico debe dejar el mailbox re-agendable; es un caso nuevo a cubrir en el PR 6, no derivado de ese spec.
- **Ubicación del ADR:** `docs/adr/` existe sólo con este archivo; `docs/` no tiene otra convención de ADR (hay `proposals/`, `research/`, `evidence/`). Se mantiene `docs/adr/` salvo indicación del Owner.

Verificado contra el código (2026-11-20):
- **D1–D7 confirmados** en `cell.rs` (MbState :29-35, `can_run` :190, `enqueue_user` :212, `dead_system_message` :232, `finish_terminate` :666) y `dispatcher.rs`: `execute` hace `let _ = tx.send(..)` y tras `shutdown` (`tx.take()`) no agenda ni reporta, dejando `scheduled = true` (D5).
- **Corrección respecto de la redacción previa:** `becomeClosed` fija `Closed` exacto y por tanto borra `scheduled` y la cuenta de suspensión (§7b).
- **Cobertura real del piloto** (`crates/rakka-actor/tests/`): throughput por turno (`classic_tell.rs::single_thread_dispatcher_runs_throughput_messages_per_turn`), usuario → dead letters tras cierre (`classic_tell.rs`, `classic_supervision.rs::stop_stops_the_child_and_dead_letters_what_was_queued`), suspensión de hijos (`classic_suspension.rs`), `Watch` sobre muerto (`classic_hierarchy.rs::watching_a_dead_actor_notifies_each_watch_without_confirming_existence`), y, en `src/`, `dispatcher.rs::shutdown_is_idempotent`. **Sin tests** de: reenvío de mensajes de sistema distintos de `Watch` a dead letters (D3), `throughput-deadline-time`, bounded/prioridad/control-aware/deque en el mailbox, carreras de estado, ni reintento ante rechazo del pool.
- Gates: el PR sólo cambia documentación. `tools/package/check.py` sobre el árbol final: `rows=785 citation_problems=0 failures=0`; `cargo fmt --all --check`: limpio. Sin cambios de código: replay, clippy, deny, cobertura y mutantes no se ven afectados.

Pendiente: cuerpos de `ActorModelSpec` ("one at a time", "suspended", "queueing from multiple threads") para fijar los escenarios del PR 1.
