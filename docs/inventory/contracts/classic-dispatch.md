# Contratos del grupo classic-dispatch (MBX, DSP, SCH)

Fuente: Apache Pekko 1.7.1 (`upstream/pekko`, 9a54d129). Rutas relativas a `upstream/pekko/`. `actor/` = `actor/src/main/scala/org/apache/pekko/`, `tests/` = `actor-tests/src/test/scala/org/apache/pekko/`. Las filas marcadas `listed` en el TSV no tienen el cuerpo del spec leído: su bloque indica sólo lo leído en implementación.

---

# Unidad MBX (mailboxes y stash)

## AC-MBX-001 Estado del mailbox
- **Contrato observable**: interno. Estado primario Open(0)/Closed(1), bit Scheduled(2), contador de suspensión en unidades de 4. Suspender/reanudar anidado; Closed terminal.
- **Fuente**: `actor/dispatch/Mailbox.scala:35-57` (constantes), `:121-204` (operaciones). reference.conf: n/a.
- **Condiciones**: `suspend()` devuelve true si el contador previo era 0 (`:168`); `resume()` true si el contador llega a 0 (`:153`) y no baja de 0 (`:152`); en Closed ambas devuelven false y re-fijan Closed (`:149-150,165-166`); `setAsScheduled` sólo si estado (s & 3)==Open (`:193`); `canBeScheduledForExecution` (`:223-228`): Open|Scheduled -> hint o mensajes; Closed -> false; suspendido -> sólo mensajes de sistema.
- **Orden/concurrencia**: todas las transiciones por CAS sobre un entero (`:134-139`).
- **Dependencias**: ninguna.
- **Equivalente Rust**: `MailboxStatus` sobre `AtomicU32` con las mismas máscaras; métodos `suspend`, `resume`, `become_closed`, `set_as_scheduled`, `set_as_idle`. Sin diferencias observables.
- **Aserciones**: A1 suspend/resume anidados; A2 closed terminal; A3 sólo se procesa si no suspendido ni cerrado (`shouldProcessMessage`, `Mailbox.scala:124`). Todas *no afirmadas por spec* en esta revisión (derivadas del código `Mailbox.scala:121-204`); ActorModelSpec no fue leído.
- **Oráculo**: no aplica (interno).
- **Decisiones**: Q-MBX-5.

## AC-MBX-002 Ejecución del mailbox
- **Contrato observable**: interno. `run()`: si no cerrado, drena todos los mensajes de sistema y luego procesa hasta `max(throughput,1)` mensajes; tras cada mensaje re-procesa sistema; al final `setAsIdle` y `registerForExecution(this,false,false)`.
- **Fuente**: `Mailbox.scala:230-240` (run), `:263-280` (processMailbox), `:289-333` (processAllSystemMessages); reference.conf:591,594.
- **Condiciones**: continúa sólo si `left>1` y (sin deadline o `nanoTime-deadline<0`) (`:277`); mensajes de sistema pendientes tras cerrar se reenvían al deadLetterMailbox (`:308-326`) y un error al reenviar publica `Error(e, path, clase, "error while enqueuing <msg> to deadLetters: <msg>")` (`:320-325`). La interrupción de hilo relanza `InterruptedException("Interrupted while processing actor messages")` (`:275`) / `"...system messages"` (`:300`) — n/a en Rust.
- **Orden/concurrencia**: un mailbox nunca se ejecuta en dos hilos a la vez (bit Scheduled); nunca se ejecuta un mensaje normal con un mensaje de sistema pendiente (`:301`).
- **Dependencias**: AC-MBX-001, AC-DSP-002, AC-DSP-003.
- **Equivalente Rust**: `MailboxRunner::run_turn(&mut self)`; tarea por mailbox (Q-MBX-6, Q-X-4). Diferencia: interrupción de hilo no existe.
- **Aserciones**: A1 sistema antes que usuario; A2 máximo `throughput` mensajes por turno; A3 deadline corta el turno. *No afirmadas por spec* aquí (derivadas de `Mailbox.scala:230-280`); DispatcherActorSpec/ActorModelSpec no leídos.
- **Oráculo**: no aplica por ahora.
- **Decisiones**: Q-MBX-5, Q-MBX-6.

## AC-MBX-003 Mailbox cerrado y limpieza
- **Contrato observable**: al hacer `cleanUp` (detach) los mensajes de sistema pendientes pasan a `deadLetterMailbox.systemEnqueue` y los de usuario a `messageQueue.cleanUp(self, deadLetterMailbox.messageQueue)`.
- **Fuente**: `Mailbox.scala:340-354`; `Mailbox.scala:408-414,442-448,518-526` (cleanUp por tipo de cola); `AbstractDispatcher.scala:240-246` (no releído en esta revisión).
- **Condiciones**: `actor eq null` (deadLetterMailbox) => no hace nada (`:341`); `messageQueue ne null` necesario por CallingThreadDispatcher (`:352`).
- **Orden**: mensajes de sistema re-encolados "vírgenes" (`unlink`) (`:345-349`).
- **Dependencias**: AC-MBX-021, AC-DSP-001.
- **Equivalente Rust**: `Mailbox::clean_up(&self, dead_letters)`.
- **Aserciones**: A1 pendientes terminan en dead letters (afirmado por ActorWithBoundedStashSpec.scala:128-132,146 para el stash/mailbox acotado; el resto *no afirmada por spec* leída); ActorRefSpec no leído.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-5.

## AC-MBX-004 SPI MessageQueue / MailboxType / ProducesMessageQueue
- **Contrato observable**: `MessageQueue{enqueue, dequeue (null si vacía), numberOfMessages, hasMessages, cleanUp(owner, deadLetters)}`; `MailboxType.create(owner: Option[ActorRef], system: Option[ActorSystem])`; mailbox-type configurable por FQCN con constructor `(Settings, Config)`.
- **Fuente**: `Mailbox.scala:362-396,644-650`; `Mailboxes.scala:241-255`; reference.conf:631-635,686-733.
- **Condiciones**: constructor inexistente => `IllegalArgumentException("Cannot instantiate MailboxType [<fqcn>], defined in [<id>], make sure it has a public constructor with [org.apache.pekko.actor.ActorSystem.Settings, com.typesafe.config.Config] parameters", causa)` (`Mailboxes.scala:249-252`); `mailbox-type` vacío => `ConfigurationException("The setting mailbox-type, defined in [<id>] is empty")` (`:242`).
- **Orden**: múltiples productores, un consumidor como mínimo (`Mailbox.scala:360`).
- **Dependencias**: AC-CFG-016, Q-X-2.
- **Equivalente Rust**: `trait MessageQueue`, `trait MailboxType { fn create(&self, owner: Option<&ActorRef>, system: Option<&ActorSystem>) -> Box<dyn MessageQueue> }` y registro por nombre (Q-DSP-2).
- **Aserciones**: A1 mailbox custom por dispatcher se usa para el actor (queue.getClass == MyMailbox) — `tests/dispatch/MailboxConfigSpec.scala:257-265`; A2 `create` recibe owner (el MyMailboxType lanza "no mailbox owner given" si None) — `MailboxConfigSpec.scala:243-248`; A3 mensajes de error: derivados de `Mailboxes.scala:242,249-252` (*no afirmadas por spec*).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-2, Q-DSP-2.

## AC-MBX-005 UnboundedMailbox
- **Contrato observable**: FIFO sin límite, multi-consumidor (ConcurrentLinkedQueue), `remainingCapacity==Int.MaxValue`.
- **Fuente**: `Mailbox.scala:535-538,653-665`; reference.conf:635.
- **Condiciones**: `dequeue()` en vacía => null; `numberOfMessages` consistente con el tamaño.
- **Orden**: FIFO por productor; con varios productores, orden de llegada.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: `UnboundedMailbox` sobre `crossbeam::queue::SegQueue`; "null" => `Option::None`.
- **Aserciones**: A1 estado inicial 0/false — `MailboxConfigSpec.scala:127-140`; A2 tras 100 enqueue el tamaño crece de a 1 y se desencola en orden — `MailboxConfigSpec.scala:100-125` (`ensureSingleConsumerEnqueueDequeue`, usado en `:48-50`); A3 10000 mensajes con 4 consumidores concurrentes: sin duplicados ni pérdidas — `MailboxConfigSpec.scala:142-192` (usado en `:77-79`).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-006 SingleConsumerOnlyUnboundedMailbox
- **Contrato observable**: MPSC sin límite; no apta para BalancingDispatcher; `maxConsumers=1`.
- **Fuente**: `Mailbox.scala:398-415,675-680`; `actor/src/main/java/org/apache/pekko/dispatch/AbstractNodeQueue.java` (no releído).
- **Condiciones**: cleanUp mueve todo a deadLetters (`:408-414`).
- **Orden**: FIFO; un solo consumidor.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: canal MPSC (`crossbeam::channel` o cola intrusiva); diferencia: `numberOfMessages` exacto. Licencia BSD (docs/upstream.md) si se porta el nodo.
- **Aserciones**: A1 las mismas pruebas de cola con 1 consumidor — `MailboxConfigSpec.scala:269-277`; A2 ping-pong patológico de 2.000.000 mensajes termina en 30 s — `MailboxConfigSpec.scala:297-318,321-323`.
- **Oráculo**: no aplica (rendimiento).
- **Decisiones**: Q-MBX-6.

## AC-MBX-007 BoundedMailbox
- **Contrato observable**: cola acotada; si está llena `enqueue` espera `pushTimeOut`; vencido => `DeadLetter(msg, sender, receiver)` a `provider.deadLetters`; `pushTimeOut<0` => `put` bloqueante infinito; `0` => `offer` inmediato.
- **Fuente**: `Mailbox.scala:557-571,705-726`; reference.conf:637-647 (`mailbox-capacity=1000`, `mailbox-push-timeout-time=10s`).
- **Condiciones**: `capacity<0` => `IllegalArgumentException("The capacity for BoundedMailbox can not be negative")` (`:713`); `pushTimeOut null` => `IllegalArgumentException("The push time-out for BoundedMailbox can not be null")` (`:714`).
- **Orden/concurrencia**: el emisor se bloquea; cola `LinkedBlockingQueue`.
- **Dependencias**: AC-MBX-004, AC-MBX-022.
- **Equivalente Rust**: cola acotada con envío awaitable y variante no bloqueante (Q-MBX-1). Diferencia observable: bloqueo del emisor sólo con timeout>0.
- **Aserciones**: A1 estado inicial `remainingCapacity==capacity` — `MailboxConfigSpec.scala:132`; A2 capacidad 10 llena + 1 enqueue => DeadLetter(msg, deadLetters, testActor) y tras dequeue quedan 9 — `MailboxConfigSpec.scala:56-75`; A3 timeout -1 ms y 0 ms y 100 ms: 10000 (o 20/10 con timeout 0) sin duplicados — `MailboxConfigSpec.scala:81-92`; A4 `pushTimeOut == Duration.Zero` desde config — `tests/actor/ActorMailboxSpec.scala:338-344`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-1.

## AC-MBX-008 NonBlockingBoundedMailbox
- **Contrato observable**: cola acotada MPSC sin bloqueo; excedente => DeadLetter; ignora push timeout.
- **Fuente**: `Mailbox.scala:421-448,690-700`; `AbstractBoundedNodeQueue.java` (no releído).
- **Condiciones**: `capacity<0` => `IllegalArgumentException("The capacity for NonBlockingBoundedMailbox can not be negative")` (`:696`). `pushTimeOut=Duration.Undefined` (`:426`).
- **Orden**: un consumidor; `size()` exacto.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: cola acotada lock-free con `try_push`.
- **Aserciones**: A1 20 enqueue/10 dequeue con timeout 0 produce 10 DeadLetter (`EventFilter.warning(... occurrences = enqueueN - dequeueN)`) — `MailboxConfigSpec.scala:85-87,150`; A2 ping-pong con capacidad 1 y throughput 1 — `MailboxConfigSpec.scala:325-327`; A3 mensaje de capacidad negativa *no afirmada por spec* (`Mailbox.scala:696`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6.

## AC-MBX-009 UnboundedPriorityMailbox / BoundedPriorityMailbox / PriorityGenerator  (listed)
- **Contrato observable**: cola ordenada por `Comparator[Envelope]`; el no estable no garantiza FIFO entre iguales.
- **Fuente**: `Mailbox.scala:732-773`; `dispatch/Dispatcher.scala:164-183` (PriorityGenerator, no releído).
- **Condiciones**: Bounded: capacidad<0 y pushTimeOut null como BoundedMailbox (`:760-761`); cola `BoundedBlockingQueue(capacity, PriorityQueue(11,cmp))` (`:769`); Unbounded `initialCapacity` por defecto 11 (`:735`).
- **Orden**: por prioridad.
- **Dependencias**: AC-MBX-004, AC-UTIL-012.
- **Equivalente Rust**: `BinaryHeap` con comparador.
- **Aserciones**: A1 cola de prioridad pasa las pruebas genéricas de cola con comparador `_.##` — `MailboxConfigSpec.scala:204-213` (cuerpo de MailboxSpec leído); el orden por prioridad *no afirmado* en esta revisión (PriorityDispatcherSpec no leído).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-4.

## AC-MBX-010 Stable priority mailboxes  (listed)
- **Contrato observable**: como prioridad pero FIFO entre iguales.
- **Fuente**: `Mailbox.scala:780-822`.
- **Condiciones**: igual que AC-MBX-009 (`:809-810`).
- **Orden**: prioridad, luego FIFO.
- **Dependencias**: AC-MBX-004, AC-UTIL-013.
- **Equivalente Rust**: heap con número de secuencia.
- **Aserciones**: A1 pasa pruebas genéricas — `MailboxConfigSpec.scala:215-224`; orden estable *no afirmado* en esta revisión.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-4.

## AC-MBX-011 Mailboxes deque-based  (listed)
- **Contrato observable**: `enqueueFirst` pone el mensaje al frente (requisito de Stash); Bounded con push timeout igual que BoundedMailbox.
- **Fuente**: `Mailbox.scala:576-629,827-868`.
- **Condiciones**: Bounded: `offerFirst` con timeout o `putFirst` si timeout<0, desbordado => DeadLetter (`:618-626`); capacidad<0 => `IllegalArgumentException("The capacity for BoundedDequeBasedMailbox can not be negative")` (`:854`) y null pushTimeOut (`:855-856`).
- **Orden**: LIFO sólo para `enqueueFirst`.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: `VecDeque` bajo `Mutex` con `push_front`.
- **Aserciones**: A1 el actor con Stash obtiene cola deque — `ActorMailboxSpec.scala:270-281`; A2 `bounded-deque-based` acotado — `ActorMailboxSpec.scala:357-362`. El orden de `enqueueFirst` *no afirmado* directamente (derivado de `Mailbox.scala:596`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-1.

## AC-MBX-012 Mailboxes control-aware  (listed)
- **Contrato observable**: `ControlMessage` se desencola antes que los demás; FIFO por clase.
- **Fuente**: `Mailbox.scala:873-960`.
- **Condiciones**: Bounded: contador atómico `size`, bloqueo en `putLock` con `awaitNanos(pushTimeOut)`; vencido => DeadLetter (`:1000-1033`). `numberOfMessages` = control + normal (`:889`).
- **Orden**: control primero.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: dos colas + marcador (Q-MBX-3).
- **Aserciones**: A1 pasa pruebas genéricas — `MailboxConfigSpec.scala:226-234`; A2 mailbox control-aware elegido por config y por requirement — `ActorMailboxSpec.scala:291-314`. La prioridad de control *no afirmada* (ControlAwareDispatcherSpec no leído).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-3.

## AC-MBX-013 Precedencia de selección de mailbox
- **Contrato observable**: ver TSV (contrato corregido).
- **Fuente**: `Mailboxes.scala:133-210`; `actor/ActorRefProvider.scala:705-720`; `actor/dungeon/Dispatch.scala:92-104`.
- **Condiciones**: requisito del dispatcher `mailbox-requirement` ("" = sin requisito, `Mailboxes.scala:133-135`) y del actor (`RequiresMessageQueue[T]`, `:117-127`); `getRequiredType` con tipo comodín => `IllegalArgumentException("no wildcard type allowed in RequireMessageQueue argument (was [x])")` (`:123`). Cola producida que no cumple => IAE (textos en TSV) envuelta en `ConfigurationException` (`ActorRefProvider.scala:715-718`).
- **Orden**: la creación del mailbox ocurre al crear el actor, el fallo se retrasa a la creación (`Dispatch.scala:86-88`).
- **Dependencias**: AC-MBX-014, AC-MBX-015, AC-DSP-007.
- **Equivalente Rust**: `Mailboxes::mailbox_type_for(props, dispatcher_config) -> Result<MailboxType, ConfigurationError>`.
- **Aserciones** (todas `tests/actor/ActorMailboxSpec.scala`): A1 por defecto cola no acotada `:255-257`; A2 Props.withMailbox deque `:259-264`; A3 RequiresMailbox acotado `:266-268`; A4 Stash => deque ilimitada (4 formas de Props) `:270-281`; A5 deployment.mailbox acotado `:283-285`, deque `:287-289`, control-aware `:291-300`; A6 `RequiresMessageQueue` control-aware `:302-314`; A7 deque configurado como mailbox con actor que requiere acotada => ConfigurationException `:316-319`; A8 mailbox en dispatcher `:321-323`, y conflicto con requisito del actor => ConfigurationException `:325-328`; A9 mailbox de deployment gana sobre dispatcher `:330-332,346-348`; A10 dispatcher acotado `:334-336`; A11 Props.withMailbox pierde frente a deployment `:350-355`; A12 combinaciones de `requiring-bounded-dispatcher`/stash `:357-437`; A13 BalancingDispatcher (con y sin requirement) `:439-458`.
- **Oráculo**: no aplica (sólo Pekko compilado podría confirmar; no hay jar de oráculo para esto).
- **Decisiones**: Q-MBX-2.

## AC-MBX-014 Lookup de mailboxes
- **Contrato observable**: ver TSV.
- **Fuente**: `Mailboxes.scala:40-43,105-110,217-293`; reference.conf:631-734; typed `PropsAdapter.scala:45`.
- **Condiciones**: textos de error en TSV. Los ids se cachean en `ConcurrentHashMap` (`:74,272-277`): misma instancia por id.
- **Orden**: carrera benigna (`:224-226`).
- **Dependencias**: AC-CFG-001.
- **Equivalente Rust**: `Mailboxes::lookup(id)` con caché `DashMap`/`RwLock<HashMap>`.
- **Aserciones**: A1 requirement por tipo resuelve a mailbox correcto — `ActorMailboxSpec.scala:266-268,302-314` (bound/control-aware); A2 ids `unbounded`, `bounded` y `bounded-capacity:N`: *no afirmadas por spec* (derivadas de `Mailboxes.scala:229-235`); A3 mapeo de requirement de usuario (`MCBoundedMessageQueueSemantics` -> `mc-bounded-mailbox`) — `ActorMailboxSpec.scala:176-179,453-458`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-1, Q-DSP-2.

## AC-MBX-015 Semánticas de cola y RequiresMessageQueue
- **Contrato observable**: marcadores: MultipleConsumer, Unbounded, Bounded (con `pushTimeOut`), DequeBased(+Unbounded/Bounded), ControlAware(+Unbounded/Bounded), Logger. `RequiresMessageQueue[T]` en el actor; `ProducesMessageQueue[T]` en el MailboxType.
- **Fuente**: `Mailbox.scala:509-629,873-899,1038-1046,648`; `Mailboxes.scala:112-152`.
- **Condiciones**: un MailboxType que no declara `ProducesMessageQueue` produce `MessageQueue` genérico (`Mailboxes.scala:140`).
- **Orden**: n/a.
- **Dependencias**: AC-MBX-013.
- **Equivalente Rust**: `QueueSemantics` bitflags + `fn required_queue(&self) -> QueueSemantics` en el actor (Q-MBX-2).
- **Aserciones**: A1 cada mailbox satisface las semánticas esperadas — `ActorMailboxSpec.scala:202-222` y `:244-251`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-2.

## AC-MBX-016 Stash
- **Contrato observable**: ver TSV.
- **Fuente**: `actor/Stash.scala:71-277`; `Mailboxes.scala:295-331`; reference.conf:649-653; docs `docs/src/main/paradox/actors.md` (sección Stash, no releída).
- **Condiciones**: textos exactos en TSV. El mensaje estachado se elimina del stash aunque `unstash()` falle (`Stash.scala:210-217`).
- **Orden**: `unstashAll` recorre el stash al revés y hace `enqueueFirst` => orden original delante del resto (`:245-249`).
- **Dependencias**: AC-MBX-011, AC-MBX-022, AC-MBX-023.
- **Equivalente Rust**: `StashBuffer<Envelope>` en el contexto con `Vec` y `push_front` del mailbox deque.
- **Aserciones**: A1 stash + unstashAll + become procesa "bye" tras "hello" — `ActorWithStashSpec.scala:121-127`; A2 protocolo open/write/close con unstash — `:129-139`; A3 doble stash => IllegalStateException — `:141-147`; A4 Terminated estachado se re-recibe (2 veces) — `:186-192`; A5 desborde de mailbox al unstashAll => DeadLetter del primero — `ActorWithBoundedStashSpec.scala:125-128`; A6 stash-capacity=20: el 21 lanza StashOverflowException — `:137-146`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-1.

## AC-MBX-017 Stash en restart y stop
- **Contrato observable**: preRestart y postStop hacen `unstashAll`; en stop los mensajes terminan en deadLetters.
- **Fuente**: `Stash.scala:82-102`.
- **Condiciones**: `unstashAll` se ejecuta en `try/finally` antes de `super.preRestart/postStop`.
- **Orden**: antes de que la nueva instancia procese.
- **Dependencias**: AC-MBX-016.
- **Equivalente Rust**: hooks `pre_restart`/`post_stop` del trait Stash.
- **Aserciones**: A1 tras crash el nuevo actor recibe "hello" estachado — `ActorWithStashSpec.scala:149-184`; A2 tras PoisonPill los estachados llegan a deadLetters en orden — `ActorWithBoundedStashSpec.scala:128-132`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-018 UnboundedStash / UnrestrictedStash / StashFactory / clearStash  (listed)
- **Contrato observable**: `Stash` exige `DequeBasedMessageQueueSemantics`; `UnboundedStash` exige `UnboundedDequeBasedMessageQueueSemantics`; `UnrestrictedStash` no exige nada (el mailbox debe ser deque o falla `ActorInitializationException`).
- **Fuente**: `Stash.scala:71-125,160-173,254-264`.
- **Condiciones**: ver AC-MBX-016. `clearStash` devuelve y vacía; `unstashAll(filtro)` filtra y vacía todo.
- **Orden**: como AC-MBX-016.
- **Dependencias**: AC-MBX-016.
- **Equivalente Rust**: variantes de `StashRequirement` en el actor.
- **Aserciones**: sin test en actor-tests (*no afirmada por spec*).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-2.

## AC-MBX-019 numberOfMessages aproximado
- **Contrato observable**: puede devolver -1/0 u otra estimación; usar `hasMessages` para presencia.
- **Fuente**: `Mailbox.scala:108-113,374-379`.
- **Condiciones**: el test acepta -1.
- **Orden**: n/a.
- **Dependencias**: AC-RT-008.
- **Equivalente Rust**: `fn number_of_messages(&self) -> usize` aproximado.
- **Aserciones**: A1 `numberOfMessages` es -1 o exacto — `MailboxConfigSpec.scala:99-105`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-020 Requirement LoggerMessageQueueSemantics  (listed, omisión agregada)
- **Contrato observable**: reference.conf mapea `org.apache.pekko.event.LoggerMessageQueueSemantics` a `pekko.actor.mailbox.logger-queue` (`mailbox-type = LoggerMailboxType`).
- **Fuente**: `actor/src/main/resources/reference.conf:682-683,728-733`; implementación en `event/Logging.scala` (no leída).
- **Condiciones/Orden/Aserciones**: n/a: no leído; *no afirmada por spec* aquí.
- **Dependencias**: AC-MBX-014, EVT.
- **Equivalente Rust**: cola de logger drenable (decidir en unidad EVT, Q-X-7).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-7.

## AC-MBX-021 deadLetterMailbox  (listed, omisión agregada)
- **Contrato observable**: mailbox cerrado compartido. Usuario: `DeadLetter(msg, envelope.sender, receiver)` salvo si ya es DeadLetter (se descarta). Sistema: `deadLetters ! DeadLetter(handle, receiver, receiver)`.
- **Fuente**: `Mailboxes.scala:54-72`.
- **Condiciones**: `dequeue()=null`, `hasMessages=false`, `numberOfMessages=0`, `systemDrain` vacío.
- **Orden**: n/a.
- **Dependencias**: AC-MBX-003.
- **Equivalente Rust**: `DeadLetterMailbox` singleton.
- **Aserciones**: *no afirmada por spec* leída (derivada de `Mailboxes.scala:54-72`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-5.

## AC-MBX-022 Validación y avisos de configuración de mailbox (omisión agregada)
- **Contrato observable**: textos en TSV.
- **Fuente**: `Mailbox.scala:713-714,760-761,809-810,854-857`; `Mailboxes.scala:173-178,257-266`.
- **Condiciones**: warning sólo una vez por sistema para el pushTimeOut no cero (`mailboxNonZeroPushTimeoutWarningIssued`, `:264`); warning de `mailbox-size` una vez (`:179`).
- **Orden**: publicados en EventStream, categoría `mailboxes`.
- **Dependencias**: AC-MBX-007, AC-MBX-014.
- **Equivalente Rust**: validación en constructores (`Result<_, IllegalArgument>`) y `EventStream::publish(Warning)`.
- **Aserciones**: *no afirmadas por spec* (derivadas del código).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-023 Capacidad del stash por dispatcher/mailbox (omisión agregada)
- **Contrato observable**: ver TSV.
- **Fuente**: `Mailboxes.scala:295-331`; reference.conf:649-653.
- **Condiciones**: precedencia: sección del mailbox, luego dispatcher, luego default-mailbox (`:326-329`); cache por `dispatcher + "-" + mailbox` (`:313`).
- **Orden**: n/a.
- **Dependencias**: AC-MBX-016, AC-DSP-004.
- **Equivalente Rust**: `Mailboxes::stash_capacity(dispatcher_id, mailbox_id) -> i32`.
- **Aserciones**: A1 vía dispatcher y vía mailbox y vía alias — `ActorWithBoundedStashSpec.scala:151-175`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-1.


---

# Unidad DSP (dispatchers y executors)

## AC-DSP-001 MessageDispatcher: attach/detach, inhabitants, apagado diferido
- **Contrato observable**: `attach` registra el actor (inhabitants+1) y agenda su mailbox con hint de sistema; `detach` desregistra (inhabitants-1), cambia el mailbox del actor por deadLetterMailbox, lo cierra y limpia; si inhabitants llega a 0 se agenda `shutdown()` tras `shutdown-timeout`; reutilizar el dispatcher recrea el executor (delegate lazy).
- **Fuente**: `actor/dispatch/AbstractDispatcher.scala:75-266`; `Dispatcher.scala:53-66,125-129`; reference.conf:586-587.
- **Condiciones**: inhabitants<0 => `IllegalStateException("ACTOR SYSTEM CORRUPTED!!! A dispatcher can't have less than 0 inhabitants!")` tras `reportFailure` (`:129-135`). Scheduler ya cerrado => shutdown inmediato (`:212-219`). Máquina UNSCHEDULED/SCHEDULED/RESCHEDULED (`:191-266`).
- **Orden/concurrencia**: CAS sobre contador y estado; `shutdown` "racy" (`:254`).
- **Dependencias**: AC-MBX-002, AC-MBX-003, AC-SCH-001.
- **Equivalente Rust**: `trait MessageDispatcher { attach, detach, inhabitants }` y apagado diferido con `Scheduler::schedule_once`. Diferencia: no hay hilos que apagar si el runtime es compartido (Q-X-4).
- **Aserciones**: A1 tras `system.close()` los hilos `default`/`internal` desaparecen en 3 s y el dispatcher sigue ejecutando `Future` pospuesto — `tests/../dispatch/DispatcherShutdownSpec.scala:32-51`; A2 inhabitants/estados *no afirmados por spec* (derivados de `AbstractDispatcher.scala:126-266`); ActorModelSpec no leído.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6, Q-X-4.

## AC-DSP-002 Dispatcher event-based: registerForExecution
- **Contrato observable**: `dispatch` encola y registra con hint de mensaje; `systemDispatch` con hint de sistema; un mailbox se agenda a lo sumo una vez (CAS a Scheduled).
- **Fuente**: `Dispatcher.scala:72-160`.
- **Condiciones**: rechazo del executor => 1 reintento; segundo rechazo => `setAsIdle`, publica `Error(e, clase, clase, "registerForExecution was rejected twice!")` y relanza (`:146-156`); `executeTask` análogo con `"executeTask was rejected twice!"` (`:95-107`).
- **Orden**: `canBeScheduledForExecution` evaluado antes del CAS (`:140`).
- **Dependencias**: AC-MBX-001.
- **Equivalente Rust**: `Dispatcher::register_for_execution(mailbox, hints) -> bool` sobre `tokio::spawn`; el rechazo sólo existe si el runtime está cerrado (Result).
- **Aserciones**: A1 tell y ask/reply funcionan — `tests/actor/dispatch/DispatcherActorSpec.scala:71-82`; A2 reintento/doble rechazo *no afirmado por spec* (`Dispatcher.scala:136-158`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6.

## AC-DSP-003 throughput y throughput-deadline-time
- **Contrato observable**: máximo de mensajes por turno y tiempo máximo del turno. CONTRADICCIÓN doc/código en `throughput<=0` (ver Q-DSP-5).
- **Fuente**: `Mailbox.scala:263-280`; `AbstractDispatcher.scala:333`; reference.conf:589-594.
- **Condiciones**: `max(throughput,1)`; deadline activo sólo si `toMillis>0`; la comprobación es `(nanoTime - deadline) < 0` después de cada mensaje.
- **Orden**: un actor lento no impide que otro obtenga turno cuando throughput es alto/bajo según config.
- **Dependencias**: AC-MBX-002.
- **Equivalente Rust**: `DispatcherSettings { throughput: i32, throughput_deadline: Duration }`.
- **Aserciones**: A1 con throughput=101 y un hilo, un actor que hoga el executor no impide procesar 100 pings tras liberarlo — `DispatcherActorSpec.scala:84-110`; A2 con throughput=2 y deadline 100 ms el segundo actor corre tras el deadline — `DispatcherActorSpec.scala:112-141`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-5.

## AC-DSP-004 Registro Dispatchers
- **Contrato observable**: ver TSV (contrato corregido).
- **Fuente**: `Dispatchers.scala:56-96,136-289`.
- **Condiciones**: textos exactos en TSV.
- **Orden**: caché `ConcurrentHashMap` con `putIfAbsent`; un alias se guarda también bajo el id real (`:173-175`).
- **Dependencias**: AC-CFG-001, AC-DSP-010.
- **Equivalente Rust**: `Dispatchers` con `lookup(id) -> Result<Arc<dyn MessageDispatcher>, ConfigurationError>`, `has_dispatcher`, `register_configurator`.
- **Aserciones**: A1 propiedades definidas `throughput==17` — `DispatchersSpec.scala:155-158`; A2 id — `:160-163`; A3 config faltante => ConfigurationException — `:165-169`; A4 un solo default y `system.dispatcher` es el mismo — `:171-175`; A5 type inexistente => ConfigurationException — `:177-185`; A6 tipos correctos Pinned/Dispatcher — `:187-190`; A7 lookup idempotente — `:192-196`; A8 alias devuelve la misma instancia — `:198-202`; A9 alias faltante => ConfigurationException — `:204-208`; A10 bucle de alias => ConfigurationException — `ActorSystemDispatcherSpec.scala:170-174`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-2, Q-DSP-4, Q-X-1.

## AC-DSP-005 default-dispatcher
- **Contrato observable**: valores por defecto en TSV; instancia única.
- **Fuente**: `reference.conf:366-605`; `Dispatchers.scala:62,121-127`.
- **Condiciones/Orden**: `defaultGlobalDispatcher == lookup(DefaultDispatcherId) == system.dispatcher`.
- **Dependencias**: AC-DSP-008.
- **Equivalente Rust**: `DispatcherSettings::default()` con los números de reference.conf.
- **Aserciones**: A1 identidad — `DispatchersSpec.scala:171-175`; los valores numéricos *no afirmados por spec* (derivados de `reference.conf:471-605`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-1.

## AC-DSP-006 internal-dispatcher, default-blocking-io-dispatcher, ExecutionContext externo  (listed)
- **Contrato observable**: `internal-dispatcher` (fork-join, parallelism 4/1.0/64, throughput 5) para guardianes y actores internos; `default-blocking-io-dispatcher` (thread-pool fixed 16, throughput 1); EC pasado por BootstrapSetup reemplaza al executor `default-executor`.
- **Fuente**: `reference.conf:607-629`; `Dispatchers.scala:64-75,134`; `AbstractDispatcher.scala:513-541` (con EC: `Debug("DefaultExecutorServiceConfigurator", ..., "Using passed in ExecutionContext as default executor for this ActorSystem. If you want to use a different executor, please specify one in pekko.actor.default-dispatcher.default-executor.")`).
- **Condiciones**: sin EC => `fallback`.
- **Orden**: n/a.
- **Dependencias**: AC-CFG-015.
- **Equivalente Rust**: `Executor` inyectable en `ActorSystemSetup`.
- **Aserciones**: guardián de usuario por defecto usa internal-dispatcher — `ActorSystemDispatcherSpec.scala:152-154`; con guardián provisto usa default — `:156-167`; el resto de ese spec no se leyó completo.
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-4.

## AC-DSP-007 Selección del dispatcher de un actor
- **Contrato observable**: ver TSV.
- **Fuente**: `ActorRefProvider.scala:684-757`.
- **Condiciones**: textos en TSV.
- **Orden**: verificación en la creación, antes de arrancar.
- **Dependencias**: AC-CFG-012, AC-DSP-004, AC-MBX-013.
- **Equivalente Rust**: resolución en `ActorRefProvider::actor_of`.
- **Aserciones**: A1 dispatcher de deployment — `DispatchersSpec.scala:266-268`; A2 deployment pisa Props — `:270-273`; A3 pool-dispatcher del router — `:275-284`; A4 balancing-pool con mailbox especial — `:286-295`; A5 dispatcher inexistente => ConfigurationException (actor, cabeza del router, routees) — `ActorConfigurationVerificationSpec.scala:95-110`; A6 router con BalancingDispatcher => ConfigurationException — `:58-85`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-2.

## AC-DSP-008 ForkJoinExecutorConfigurator
- **Contrato observable**: ver TSV (contrato corregido).
- **Fuente**: `ForkJoinExecutorConfigurator.scala:43-49,120-234`; `ThreadPoolBuilder.scala:49-50`; reference.conf:469-520.
- **Condiciones**: errores en TSV.
- **Orden**: n/a.
- **Dependencias**: AC-DSP-004.
- **Equivalente Rust**: pool work-stealing propio o runtime; claves aceptadas y validadas (Q-DSP-1).
- **Aserciones**: A1 `resolveMinimumRunnable(0,...)==0` en JDK 17 y 25 — `ForkJoinExecutorConfiguratorSpec.scala:76-79`; A2 valores positivos verbatim — `:81-85`; A3 -1 en JDK<25 => 1 — `:87-94`; A4 -1 en JDK 25: parallelism/2 acotado a [1,8] (1→1,2→1,4→2,8→4,16→8,64→8) — `:96-103`; A5 nunca >8 — `:117-120`; A6 integración por dispatcher (0, 7, auto, nunca <1) — `:145-174`.
- **Oráculo**: A1-A5 son funciones puras: comparables con el Pekko compilado de `upstream/pekko`.
- **Decisiones**: Q-DSP-1.

## AC-DSP-009 ThreadPoolExecutorConfigurator
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: n/a.
- **Dependencias**: AC-DSP-004.
- **Equivalente Rust**: pool de hilos con tamaño core/max, cola, keep-alive (Q-DSP-1).
- **Aserciones**: A1 nombre de hilo — `DispatchersSpec.scala:234-240`; el cálculo de tamaños *no afirmado por spec* (derivado de `ThreadPoolBuilder.scala:49-50,164-173`).
- **Oráculo**: scaledPoolSize comparable (función pura).
- **Decisiones**: Q-DSP-1.

## AC-DSP-010 ExecutorServiceConfigurator SPI  (listed)
- **Contrato observable**: `executor` ∈ {default-executor, fork-join-executor, virtual-thread-executor (fallback si JDK no soporta), thread-pool-executor, affinity-pool-executor, FQCN}; vacío o null => fork-join.
- **Fuente**: `AbstractDispatcher.scala:353-410,513-541`; reference.conf:373-395.
- **Condiciones**: FQCN inválido => `IllegalArgumentException("Cannot instantiate ExecutorServiceConfigurator (\"executor = [x]\"), defined in [id], make sure it has an accessible constructor with a [Config,DispatcherPrerequisites] signature")` (`:392-396`).
- **Orden**: n/a.
- **Dependencias**: AC-CFG-016.
- **Equivalente Rust**: `trait ExecutorFactory` + registro (Q-DSP-2).
- **Aserciones**: *no afirmadas por spec* leída; ActorSystemDispatcherSpec parcial.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-1, Q-DSP-2.

## AC-DSP-011 PinnedDispatcher
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: un hilo; throughput infinito.
- **Dependencias**: AC-DSP-009.
- **Equivalente Rust**: `std::thread` dedicado + bucle del mailbox.
- **Aserciones**: A1 tell — `PinnedActorSpec.scala:49-56`; A2 ask — `:58-62`; A3 nombre de hilo contiene id — `DispatchersSpec.scala:250-256`; el error "Cannot register to anyone but" *no afirmado por spec*.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-1.

## AC-DSP-012 BalancingDispatcher  (listed)
- **Contrato observable**: cola compartida (`messageQueue` única creada con `mailboxType.create(None,None)`); cada actor tiene un `SharingMailbox` que usa esa cola; `dispatch` encola en la cola compartida y, si no pudo registrar el mailbox del receptor, hace `teamWork` (agenda otros miembros del equipo ociosos) si `attempt-teamwork=on` y el executor no está al límite.
- **Fuente**: `BalancingDispatcher.scala:60-126`.
- **Condiciones**: cleanUp del SharingMailbox sólo drena mensajes de sistema a deadLetters (la cola compartida no se limpia).
- **Orden**: `team` ordenado por path.
- **Dependencias**: AC-DSP-021, AC-RT (BalancingPool).
- **Equivalente Rust**: `BalancingDispatcher` interno para `BalancingPool`.
- **Aserciones**: A1 el actor rápido procesa más mensajes que el lento y ambos mailboxes quedan vacíos — `BalancingDispatcherSpec.scala:63-100` (parcial, no leído hasta el final); A2 ConfigurationException con pools no Balancing — `ActorConfigurationVerificationSpec.scala:58-85`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6.

## AC-DSP-013 AffinityPool  (listed)
- **Contrato observable**: claves leídas de reference.conf:400-467 (`parallelism-min=4`, `factor=0.8`, `max=64`, `task-queue-size=512`, `rejection-handler=ThrowOnOverflowRejectionHandler`, `idle-cpu-level=5` en [1,10], `queue-selector=FairDistributionHashCache`, `fair-work-distribution.threshold=128` en [0,2048]). Implementación (AffinityPool.scala) **no leída**.
- **Fuente**: reference.conf:397-467; AffinityPool.scala (pendiente).
- **Condiciones/Orden**: pendiente.
- **Dependencias**: AC-DSP-010.
- **Equivalente Rust**: pool con una cola MPSC acotada por worker (Q-DSP-1).
- **Aserciones**: sin tests en actor-tests (*no afirmada por spec*).
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-1.

## AC-DSP-014 VirtualThreadExecutorConfigurator  (listed)
- **Contrato observable**: `virtual-thread-executor` ejecuta cada tarea en un hilo virtual; si la JVM no lo soporta usa `fallback`; con `fork-join-executor.virtualize=on` los actores corren en hilos virtuales sobre el pool como scheduler.
- **Fuente**: `AbstractDispatcher.scala:372-380,412-465`; reference.conf:509-519,573-585.
- **Condiciones**: fallo al obtener el scheduler por defecto => `Warning(e, "VirtualThreadExecutorConfigurator", ..., "Failed to get the default scheduler of virtual thread, so the `LoadMetrics` is not available when using it with `BalancingDispatcher`. ...")` (`:440-445`).
- **Orden**: n/a.
- **Dependencias**: ninguna.
- **Equivalente Rust**: tareas async (equivalente natural; diferencia: sin hilo por tarea).
- **Aserciones**: spec JDK21 no leído (*no afirmada*).
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-1.

## AC-DSP-015 MonitorableThreadFactory
- **Contrato observable / Fuente**: ver TSV; `ThreadPoolBuilder.scala:191-242`.
- **Condiciones**: nombre `<base>-<contador>`; el contador es por fábrica; base = `<sistema>-<id>` (`ForkJoinExecutorConfigurator.scala:201-205`, `ThreadPoolBuilder.scala:125-127`).
- **Orden/Dependencias**: AC-CFG-007.
- **Equivalente Rust**: `std::thread::Builder::name`.
- **Aserciones**: A1-A5 nombres de hilo con regex por tipo de dispatcher — `DispatchersSpec.scala:230-264`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-DSP-016 Dispatcher como ExecutionContextExecutor  (listed)
- **Contrato observable**: `execute(Runnable)` crea `TaskInvocation`, cuenta inhabitant (+1 y -1 al terminar), errores no fatales publican `Error(e, "TaskInvocation", clase, msg)`; `reportFailure(LogEventException)` publica el evento, otra publica `Error(t, clase, clase, msg)`.
- **Fuente**: `AbstractDispatcher.scala:55-63,174-189`. BatchingExecutor.scala **no leído**.
- **Condiciones**: si `executeTask` falla se revierte el inhabitant (`:180-182`).
- **Orden/Dependencias**: AC-EVT-011.
- **Equivalente Rust**: `Executor::execute(Box<dyn FnOnce() + Send>)`.
- **Aserciones**: sin leer ExecutionContextSpec (sólo títulos): *no afirmadas* aquí.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-3.

## AC-DSP-017 ExecutionContexts / Futures helpers  (listed)
- n/a: no se leyó `Future.scala`, `CompletionStages.scala` ni sus specs. **Contrato**: pendiente. **Fuente**: solo rutas del inventario. **Equivalente Rust**: `std::future` / tokio, sin API propia (Q-DSP-3). Aserciones: ninguna verificada. Oráculo: no aplica.

## AC-DSP-018 SerializedSuspendableExecutionContext  (listed)
- n/a: implementación no leída. Del grep de `ExecutionContextSpec.scala:212-291` solo se vio: suspendible/reanudable (size 2→counter 2), ejecuta `throughput` tareas por barrido (`submissions == total/throughput`), ejecuta en serie, cede el hilo al suspenderse (13 tareas, 2 submissions). Sin lectura del cuerpo: sin aserciones registradas. Q-DSP-3.

## AC-DSP-019 Configuradores internos  (listed)
- n/a: `PekkoJdk9ForkJoinPool`, `ForkJoinPoolConstants`, `CachingConfig` no leídos. `CachingConfig` envuelve la config de los configuradores (`AbstractDispatcher.scala:361`) y de `Dispatchers` (`Dispatchers.scala:119`). Boilerplate de JVM salvo CachingConfig (comportamiento: cachea paths; sin efecto observable salvo rendimiento).

## AC-DSP-020 Envelope / mensaje nulo  (listed, omisión agregada)
- **Contrato observable**: ver TSV. **Fuente**: `AbstractDispatcher.scala:46-52` (líneas 40-45 no releídas). **Equivalente Rust**: `Envelope::new(message, sender)`; "null" no existe (Q-X-3). Aserciones: *no afirmadas por spec* leída.

## AC-DSP-021 BalancingDispatcherConfigurator (omisión agregada)
- **Contrato observable / Fuente / Condiciones**: ver TSV; `Dispatchers.scala:309-379`.
- **Orden**: validación en construcción del dispatcher.
- **Dependencias**: AC-DSP-012, AC-MBX-013.
- **Equivalente Rust**: validación al construir.
- **Aserciones**: A1 dispatcher balanceador sin mailbox => cola ilimitada; con `mailbox-type` acotado => cola acotada — `ActorMailboxSpec.scala:439-458`; A2 mensajes de error *no afirmados por spec* (`Dispatchers.scala:331-348`).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-DSP-022 Dispatchers.getConfig (omisión agregada)
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: recursivo con profundidad.
- **Dependencias**: AC-DSP-004.
- **Equivalente Rust**: `fn dispatcher_config(config, id) -> Config`.
- **Aserciones**: A1 directo — `DispatchersSpec.scala:210-213`; A2 alias — `:215-218`; A3 faltante => vacía — `:220-223`; A4 alias faltante => vacía — `:225-228`.
- **Oráculo**: función pura; comparable con Pekko compilado.
- **Decisiones**: Q-DSP-4.

---

# Unidad SCH (scheduler y timers)

## AC-SCH-001 scheduleOnce
- **Contrato observable / Fuente / Condiciones**: ver TSV (contrato refinado). Variantes: Runnable, closure, `(receiver, message)` con `sender` implícito (`Scheduler.scala:428-495`); `maxFrequency` (`Scheduler.scala` final).
- **Orden/concurrencia**: ejecuta en el `ExecutionContext` pasado; delay<=0 ejecuta ya en ese ec.
- **Dependencias**: AC-DSP-016.
- **Equivalente Rust**: `Scheduler::schedule_once(delay, task) -> Cancellable` con reloj inyectable (Q-SCH-1). Diferencia: `ExecutionContext` => `Executor`.
- **Aserciones**: A1 no corre antes de 100 ms y a los 2 s quedan 1 de 3 pendientes — `SchedulerSpec.scala:97-117`; A2 cancelar 10 tareas de 1 s no entrega nada en 2 s — `:121-125`; A3 nunca dispara antes de tiempo — `:156-176` (cuerpo no leído completo); A4 delay máximo tick*Int.MaxValue se acepta, +tick => IllegalArgumentException — `:477-484`.
- **Oráculo**: no aplica (timing); la función de redondeo `roundUp` sí es pura y comparable.
- **Decisiones**: Q-SCH-1.

## AC-SCH-002 scheduleWithFixedDelay
- **Contrato observable / Condiciones**: ver TSV.
- **Fuente**: `Scheduler.scala:83-106,153-169`; `LightArrayRevolverScheduler.scala:127-132,224-228`.
- **Orden**: el siguiente disparo se agenda tras terminar el anterior.
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: bucle async con sleep tras la tarea.
- **Aserciones**: A1 12 ejecuciones con delay 125 ms y tarea de 100 ms dan 4.4 ±0.5/s — `SchedulerSpec.scala:450-461`; A2 intervalo cero o negativo => IllegalArgumentException (mensaje en TSV, *mensaje no afirmado por spec*) — `:721-737`; A3 retardo inicial y frecuencia distintos: 1750 ms ±250 — `:399-415`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-1.

## AC-SCH-003 scheduleAtFixedRate
- **Contrato observable / Fuente / Condiciones**: ver TSV; `Scheduler.scala:228-231`, `LightArrayRevolverScheduler.scala:134-161`.
- **Orden**: compensa deriva con `max(delay - deriva, 1ns)`.
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: `tokio::time::interval` con `MissedTickBehavior::Burst` o implementación propia (Q-SCH-1).
- **Aserciones**: A1 40.0 ±4 ejecuciones/s a 25 ms — `SchedulerSpec.scala:422-432`; A2 4.4 ±0.5/s con tarea de 100 ms e intervalo 225 ms — `:434-445`; A3 cero/negativo => IllegalArgumentException — `:701-717`; A4 intervalo/retardo máximo aceptado y +tick rechazado — `:743-761`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-1.

## AC-SCH-004 schedule deprecado  (listed)
- **Contrato observable**: método deprecado desde Akka 2.6.0, misma semántica que scheduleAtFixedRate; es el abstracto que implementan los schedulers.
- **Fuente**: `Scheduler.scala:343-420`; `LightArrayRevolverScheduler.scala:134`.
- **Condiciones/Orden**: como AC-SCH-003.
- **Dependencias**: AC-SCH-003.
- **Equivalente Rust**: alias deprecado (Q-SCH-4).
- **Aserciones**: sin tests directos (*no afirmada por spec*).
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-4.

## AC-SCH-005 Cancellable
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: cancelación atómica (CAS).
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: `Cancellable(Arc<AtomicU8>)` o equivalente con `cancel() -> bool`, `is_cancelled()`, constante `ALREADY_CANCELLED`.
- **Aserciones**: A1 cancel antes de ejecutar: true, is_cancelled true, segunda llamada false — `SchedulerSpec.scala:127-133`; A2 tras ejecución: false/false/false/false — `:146-154`; A3 periódica: cancelar durante retardo inicial => 0 ticks — `:316-330`; A4 cancelar tras el retardo => 1 tick — `:332-346`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-SCH-006 SchedulerTask.cancelled()
- **Contrato observable / Fuente**: ver TSV; `Scheduler.scala:505-523` (no releído).
- **Condiciones/Orden**: se invoca al cancelar antes de ejecutar; el wrapper de repetición lo reenvía.
- **Dependencias**: AC-SCH-005.
- **Equivalente Rust**: método opcional `on_cancel` en el trait de tarea.
- **Aserciones**: A1 `SchedulerSpec.scala:135-144`; A2 periódica `:348-366`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-SCH-007 Tarea periódica a actor
- **Contrato observable / Fuente**: ver TSV; `Scheduler.scala:153-169,351-362`.
- **Condiciones**: texto `SchedulerException("timer active for terminated actor")`; excepciones de la tarea abortan la repetición y se loguean (EventFilter).
- **Orden/Dependencias**: AC-SCH-002.
- **Equivalente Rust**: `is_terminated()` tras enviar.
- **Aserciones**: A1 `SchedulerSpec.scala:252-267`; A2 excepción — `:269-283`; A3 IllegalStateException — `:285-314`; A4 reinicio del actor retoma el schedule — `:368-397` (título; cuerpo no leído).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-SCH-008 LightArrayRevolverScheduler  (listed)
- **Contrato observable**: ver TSV. Estructura: rueda de `ticks-per-wheel` cubos; `TaskHolder` con `ticks`; cada tick ejecuta el cubo; tareas con `ticks >= WheelSize` se reubican; al cancelar solo queda un cascarón.
- **Fuente**: `LightArrayRevolverScheduler.scala:50-430`; reference.conf:886-927.
- **Condiciones**: validaciones de tick-duration y ticks-per-wheel en TSV; `roundUp` (`:93-97`).
- **Orden/concurrencia**: un hilo de timer; `clock()`, `startTick`, `waitNanos` y `getShutdownTimeout` sobreescribibles para tests (`:99-125`).
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: `WheelScheduler` con `Clock` inyectable (Q-SCH-1).
- **Aserciones**: A1 `TickDuration == 1ms` con config de test — `SchedulerSpec.scala:864-870`; A2 rechazo por lejanía — `:477-484`; el resto (wrap-around, oversleep) con título solamente — *no leídas*.
- **Oráculo**: `roundUp` y cálculo de buckets puros: comparables con el Pekko compilado.
- **Decisiones**: Q-SCH-1.

## AC-SCH-009 close()
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: espera al hilo hasta shutdown-timeout; ejecuta TaskRunOnClose.
- **Dependencias**: AC-DSP-001, AC-SCH-014.
- **Equivalente Rust**: `Scheduler::close()` + marcador `RunOnClose`.
- **Aserciones**: A1 cierre ejecuta sólo TaskRunOnClose — `SchedulerSpec.scala:684-697`; A2 rechazo fiable durante el cierre — `:661-682`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-1.

## AC-SCH-010 scheduler.implementation pluggable  (listed)
- **Contrato observable**: `scheduler.implementation` = clase con constructor `(Config, LoggingAdapter, ThreadFactory)`; debe ejecutar los TaskRunOnClose al cerrar y lanzar IllegalStateException si no puede agendar (`Scheduler.scala:47-53`).
- **Fuente**: reference.conf:914-921; `Scheduler.scala:47-60`; ActorSystem.scala:1136-1160 (no leído).
- **Condiciones/Orden**: pendiente. **Dependencias**: AC-CFG-016.
- **Equivalente Rust**: trait `Scheduler` inyectable en el builder (Q-SCH-2).
- **Aserciones**: ninguna verificada.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-2.

## AC-SCH-011 Timers / TimerScheduler
- **Contrato observable / Fuente / Condiciones**: ver TSV; `Timers.scala:31-370` (API pública, doc no releída línea a línea), `dungeon/TimerSchedulerImpl.scala:28-164`.
- **Orden**: la intercepción ocurre en `aroundReceive` antes del receive y del stash.
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: `TimerScheduler` en el contexto con mapa clave -> (generación, tarea).
- **Aserciones**: A1-A8 las de `TimerSpec.scala:200-306` listadas en TSV.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-3.

## AC-SCH-012 Mensajes de timer  (listed)
- **Contrato observable**: timer cuyo mensaje es AutoReceivedMessage (p. ej. PoisonPill) se maneja como auto-recibido; un mensaje NotInfluenceReceiveTimeout produce un wrapper que no reinicia receive timeout.
- **Fuente**: `Timers.scala:52-65`; `TimerSchedulerImpl.scala:95-100`.
- **Condiciones/Orden**: wrapper `InfluenceReceiveTimeoutTimerMsg` vs `NotInfluenceReceiveTimeoutTimerMsg`.
- **Dependencias**: AC-SCH-011, AC-CELL-017.
- **Equivalente Rust**: dos variantes de mensaje interno.
- **Aserciones**: A1 timer con AutoReceive termina al actor — `TimerSpec.scala:308-314`; A2 no reiniciar receive timeout: *no leída* (`ReceiveTimeoutSpec.scala:191`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-3.

## AC-SCH-013 Timers combinados con Stash
- **Contrato observable / Fuente / Condiciones**: ver TSV; `Timers.scala:57-61`.
- **Orden**: el reemplazo del `currentMessage` ocurre antes de `super.aroundReceive`.
- **Dependencias**: AC-SCH-011, AC-MBX-016.
- **Equivalente Rust**: sustituir el mensaje actual antes de invocar el receive.
- **Aserciones**: A1 `TimerSpec.scala:344-355`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-3.

## AC-SCH-014 scheduler.shutdown-timeout  (listed, omisión agregada)
- **Contrato observable / Fuente**: ver TSV. **Condiciones**: `Await.result(stop(), getShutdownTimeout)` (`:181`). **Orden**: tras el timeout el apagado sigue sin ejecutar tareas encoladas (comentario reference.conf:923-926). **Dependencias**: AC-SCH-009. **Equivalente Rust**: parámetro de `Scheduler::close`. **Aserciones**: *no afirmada por spec*. **Oráculo**: no aplica. **Decisiones**: Q-SCH-1.

## AC-SCH-015 Recuperación del hilo del timer  (listed, omisión agregada)
- **Contrato observable / Fuente**: ver TSV. **Condiciones**: mensajes de log literales en TSV. **Orden**: se relanza `throw t` tras reiniciar (`:309`). **Dependencias**: AC-SCH-008. **Equivalente Rust**: supervisión de la tarea del reloj (Q-X-5). **Aserciones**: *no afirmada por spec*. **Oráculo**: no aplica. **Decisiones**: Q-X-5.

---

# Secciones finales

## Descomposiciones
Ninguna: no se descompuso ninguna fila en esta revisión (no se alcanzó el análisis de granularidad para las filas `listed`; ver "Filas que siguen listed"). Candidatas a descomponer por agrupar comportamientos distintos: AC-MBX-016 (stash/unstash/overflow/doble stash), AC-DSP-004 (lookup, alias, registerConfigurator, types), AC-SCH-011 (cada operación de TimerScheduler), AC-DSP-008 (cada clave de fork-join).

## Omisiones agregadas
- AC-MBX-020: reference.conf:682-683,728-733 (`LoggerMessageQueueSemantics`, `logger-queue`).
- AC-MBX-021: `Mailboxes.scala:54-72` (deadLetterMailbox).
- AC-MBX-022: `Mailbox.scala:713-714,760-761,809-810,854-857`; `Mailboxes.scala:173-178,257-266` (validaciones y warnings).
- AC-MBX-023: `Mailboxes.scala:295-331` (stash-capacity por dispatcher/mailbox).
- AC-DSP-020: `AbstractDispatcher.scala:40-52` (Envelope, InvalidMessageException).
- AC-DSP-021: `Dispatchers.scala:309-379` (BalancingDispatcherConfigurator).
- AC-DSP-022: `Dispatchers.scala:77-95` (getConfig / alias vacío).
- AC-SCH-014: reference.conf:923-927 (`scheduler.shutdown-timeout`).
- AC-SCH-015: `LightArrayRevolverScheduler.scala:288-310` (recuperación del hilo).

## Contradicciones halladas y corregidas
- AC-MBX-013: inventario decía ActorInitializationException por requisito incumplido; el test afirma ConfigurationException (`ActorMailboxSpec.scala:316-319`) y el código lanza IllegalArgumentException envuelta (`ActorRefProvider.scala:715-718`). ActorInitializationException "requires mailbox type" existe en otra rama (`dungeon/Dispatch.scala:93-100`).
- AC-DSP-004: inventario incluía `type=BalancingDispatcher` como válido; `Dispatchers.scala:268-273` lo rechaza salvo por FQCN.
- AC-DSP-003: Scaladoc (`Dispatcher.scala:37-40`) vs código (`Mailbox.scala:264`) para throughput<=0 (abierto, Q-DSP-5).
- AC-DSP-008: comentario de reference.conf:497 (default -1) vs línea efectiva `minimum-runnable = 1` (`:507`).
- AC-DSP-011: inventario omitía throughput=Int.MaxValue y el warning de ThreadPoolExecutor.
- AC-SCH-004: inventario decía "alias deprecado de fixed rate" y proponía excluir; en realidad es el método abstracto sobre el que se apoya scheduleAtFixedRate.
- AC-SCH-012: inventario decía que `PoisonPill/Kill` por timer es "auto-received" vía NotInfluenceReceiveTimeout; son dos mecanismos distintos (`Timers.scala:55-56` vs `TimerSchedulerImpl.scala:95-100`).
- Ficheros 'Q: excluir' del inventario viejo (AC-DSP-013/014/017, AC-SCH-004) se sustituyen por decisiones propuestas; las filas siguen en alcance.

## Filas que siguen listed (motivo)
AC-MBX-001/002/003/009/010/011/012 (no se leyeron ActorModelSpec, DispatcherActorSpec parcial, ActorRefSpec, Priority/StablePriority/ControlAware specs), AC-MBX-018 (sin test), AC-MBX-020/021 (omisiones sin spec leído), AC-DSP-006, 010, 012, 013, 014, 016, 017, 018, 019, 020 (specs o implementaciones no leídas), AC-SCH-004, 008, 010, 012, 014, 015.

## Cobertura
Archivos del árbol cubiertos por fila: `dispatch/Mailbox.scala` (AC-MBX-001..012,015,019,022), `dispatch/Mailboxes.scala` (AC-MBX-003,013,014,021,022,023), `actor/Stash.scala` (AC-MBX-016..018), `dispatch/AbstractDispatcher.scala` (AC-DSP-001,010,014,016,020), `dispatch/Dispatcher.scala` (AC-DSP-002,003), `dispatch/Dispatchers.scala` (AC-DSP-004..007,011,021,022), `dispatch/PinnedDispatcher.scala` (AC-DSP-011), `dispatch/BalancingDispatcher.scala` (AC-DSP-012), `dispatch/ForkJoinExecutorConfigurator.scala` (AC-DSP-008), `dispatch/ThreadPoolBuilder.scala` (AC-DSP-009,015), `dispatch/VirtualThreadSupport.scala`/`VirtualizedExecutorService.scala` (AC-DSP-014, no leídos), `dispatch/affinity/AffinityPool.scala` (AC-DSP-013, no leído), `dispatch/BatchingExecutor.scala` (AC-DSP-016, no leído), `dispatch/Future.scala`/`CompletionStages.scala` (AC-DSP-017, no leídos), `dispatch/CachingConfig.scala`/`ForkJoinPoolConstants.scala` (AC-DSP-019, no leídos), `actor/Scheduler.scala` (AC-SCH-001..007), `actor/LightArrayRevolverScheduler.scala` (AC-SCH-001,008,009,014,015), `actor/Timers.scala` y `actor/dungeon/TimerSchedulerImpl.scala` (AC-SCH-011..013). Sin comportamiento observable propio: `actor/src/main/scala-2.12/.../ScalaBatchable.scala`, `scala-2.13/...`, `scala-3/...` (adaptación de compilación de `isBatchable`, usado por `TaskInvocation`, AbstractDispatcher.scala:56) y `java/.../AbstractMessageDispatcher.java` (offsets Unsafe de los campos de `MessageDispatcher`, sin contrato observable). `AbstractNodeQueue.java`/`AbstractBoundedNodeQueue.java` (colas MPSC de AC-MBX-006/008) y `scala-jdk-9/PekkoJdk9ForkJoinPool.scala` no leídos; `reference.conf` leído: secciones default-dispatcher (todas sus claves), internal-dispatcher, default-blocking-io-dispatcher, default-mailbox, mailbox.*, scheduler.*.
