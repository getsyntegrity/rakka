# Contratos del grupo classic-dispatch (MBX, DSP, SCH)

Fuente: Apache Pekko 1.7.1 (`upstream/pekko`, 9a54d129). Las citas `path:line` de los bloques reescritos en el cierre documental (2026-10-31) usan la ruta completa desde la raíz del módulo (`actor/src/main/...`, `actor-tests/src/test/...`); en las listas de aserciones se abrevian a nombre de archivo cuando son únicos (`DispatchersSpec.scala`, `SchedulerSpec.scala`...) y dentro de bloques antiguos `tests/` = `actor-tests/src/test/scala/org/apache/pekko/`. Todas las filas del grupo están `extracted`: la implementación y los cuerpos de los specs citados se leyeron; cuando no existe spec la aserción se marca *no afirmada por spec* y se propone una caracterización (de rakka, no un test original de Pekko).

---

# Unidad MBX (mailboxes y stash)

## AC-MBX-001 Estado del mailbox

*unidad MBX · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Estado atómico de un mailbox: un entero con estado primario Open(0)/Closed(1), bit Scheduled(2) y un contador de suspensión en unidades de 4 (`suspendUnit`). Suspender/reanudar son anidados; Closed es terminal; sólo se procesan mensajes de usuario si el estado es Open (con o sin Scheduled) y sin suspensión.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:38-60` (constantes), `:121-204` (consultas y transiciones), `:223-228` (`canBeScheduledForExecution`).
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:316-332` (`suspend`/`resume` a nivel de dispatcher).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:312-330,360-382`.

**Precondiciones.** Estado inicial 0 (Open, sin Scheduled, sin suspensión). No hay configuración.

**Comportamiento.**
- `suspend()`: en Closed devuelve false y re-escribe Closed (`:164-167`); si no, CAS `s -> s + 4` y devuelve `s < 4` (true si el contador previo era 0).
- `resume()`: en Closed devuelve false (`:148-150`); si no, `next = if (s < 4) s else s - 4`, CAS, devuelve `next < 4`. Consecuencia: `resume()` sobre un mailbox NO suspendido devuelve true (el contador no baja de 0).
- `becomeClosed()`: si ya es Closed devuelve false; si no, CAS `s -> Closed` (descarta bit Scheduled y suspensión) y devuelve true (`:177-181`).
- `setAsScheduled()`: sólo si `(s & 3) == 0` (Open, sin Scheduled; un mailbox suspendido sí puede marcarse); CAS `s -> s | 2`; false si Closed o ya Scheduled (`:187-195`).
- `setAsIdle()`: CAS `s -> s & ~2`, siempre true; sobre Closed deja Closed (`:201-204`).
- Consultas: `shouldProcessMessage = (s & ~2) == 0`; `isSuspended = (s & ~3) != 0`; `suspendCount = s / 4`; `isClosed = (s == Closed)` (igualdad exacta); `isScheduled = (s & 2) != 0`.
- `canBeScheduledForExecution(hintMsg, hintSys)` (`:223-228`), por igualdad exacta del estado: 0 o 2 (Open / Open+Scheduled) -> `hintMsg || hintSys || hasSystemMessages || hasMessages`; 1 (Closed) -> false; cualquier otro (suspendido, con o sin Scheduled) -> `hintSys || hasSystemMessages` (los mensajes de usuario no alcanzan).
- A nivel de dispatcher: `suspend(actor)` sólo actúa si `mbox.actor eq actor && mbox.dispatcher eq this`; `resume(actor)` igual y, si `mbox.resume()` devuelve true, llama `registerForExecution(mbox, false, false)` (`AbstractDispatcher.scala:280-293`).

**Errores.** Ninguno: todas las operaciones son totales. Límite: el contador usa un `Int`; tras 2^29 suspensiones anidadas desborda (inalcanzable en la práctica).

**Orden / concurrencia.** Todas las transiciones son CAS en bucle (`@tailrec`) sobre un campo volátil (`Unsafe.compareAndSwapInt`); no hay locks. Un mensaje de usuario no se procesa mientras el contador de suspensión sea > 0; los mensajes de sistema sí (ver AC-MBX-002).

**Dependencias.** AC-DSP-002 (registerForExecution), AC-DSP-023 (suspend/resume del dispatcher), AC-CELL-013.1.

**Equivalente Rust propuesto.** `MailboxStatus` sobre `AtomicU32` con las mismas máscaras y `compare_exchange` en bucle; métodos `suspend`, `resume`, `become_closed`, `set_as_scheduled`, `set_as_idle`, `can_be_scheduled(hint_msg, hint_sys, has_msgs, has_sys)`. Sin diferencias observables; el desbordamiento del contador se define como pánico en debug / saturación en release (límite documentado).

**Aserciones.**
- A1 `ActorModelSpec.scala:359-381` ("not process messages for a suspended actor"): tras `a.suspend()` y `a ! CountDown(done)` el mensaje no se procesa durante 1000 ms (`assertNoCountDown`) con estadísticas `registers=1, msgsReceived=1, suspensions=1` (y `msgsProcessed=0`); tras `a.resume(causedByFailure = null)` se procesa en <=3 s con `resumes=1, msgsProcessed=1`; tras `system.stop(a)`, `unregisters=1`.
- A2 `ActorModelSpec.scala:311-329` ("process messages one at a time"): `a ! Wait(1000); a ! CountDown(oneAtATime)`; el segundo se procesa (si hubiera ejecución en paralelo `DispatcherActor.ack` lanzaría "isolation violated" y el actor se reiniciaría); `msgsReceived == msgsProcessed == 3`. Afirma que el bit Scheduled impide ejecución concurrente del mismo mailbox.
- A3 valores de retorno de suspend/resume/becomeClosed/setAsScheduled/setAsIdle, Closed terminal y `canBeScheduledForExecution`: *no afirmada por spec* (derivada de `Mailbox.scala:147-228`). Caracterización propuesta (de rakka, no test original): tabla estado x operación -> (estado', retorno) para {Open, Open+S, Susp1, Susp1+S, Susp2, Closed} x {suspend, resume, becomeClosed, setAsScheduled, setAsIdle} y la tabla de `canBeScheduledForExecution` con las 4 combinaciones de hints.
- A4 guardas de `MessageDispatcher.suspend/resume` (`mbox.actor eq actor`, `mbox.dispatcher eq this`, re-registro en resume): *no afirmada por spec*; caracterización propuesta (ver AC-DSP-023).

**Validación prevista.** A1 y A2 portados 1:1 (un `DispatcherActor` equivalente y un decorador de dispatcher que cuente suspensiones/registros). A3/A4 caracterización unitaria. Oráculo: no aplica (tipo `private[pekko]`; la tabla es trivialmente verificable por lectura).

**Decisiones.** Q-MBX-5 (el estado del mailbox y las colas son componentes separados; no cambia el contrato). Q-X-4 (modelo de ejecución).

## AC-MBX-002 Ejecución del mailbox

*unidad MBX · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Un turno de mailbox (`Mailbox.run`) procesa primero todos los mensajes de sistema pendientes y luego hasta `max(throughput, 1)` mensajes de usuario, deteniéndose antes si vence el `throughput-deadline-time`, si el mailbox se suspende/cierra, o si la cola queda vacía; después marca el mailbox ocioso y lo reprograma si quedan mensajes.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:262-273` (`run`, `exec`), `:260-280` (`processMailbox`), `:289-333` (`processAllSystemMessages`).
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:372` (`isThroughputDeadlineTimeDefined`), `Dispatcher.scala:136-159` (`registerForExecution`).
- `actor/src/main/resources/reference.conf:620-625`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatcherActorSpec.scala:84-142`; `ActorModelSpec.scala:311-347`.

**Precondiciones.** El mailbox fue marcado Scheduled por `registerForExecution` (AC-DSP-002) y tiene `actor` asignado. `throughput` y `throughputDeadlineTime` son los del dispatcher del actor (AC-DSP-003).

**Comportamiento.**
1. `run()`: si `!isClosed` -> `processAllSystemMessages()` y `processMailbox()`; en `finally`: `setAsIdle()` y `dispatcher.registerForExecution(this, false, false)` (`:230-240`), que sólo reprograma si `canBeScheduledForExecution` (AC-MBX-001).
2. `processMailbox(left = max(throughput,1), deadline = nanoTime + throughputDeadlineTime si el deadline está definido (toMillis > 0) o 0)`: mientras `shouldProcessMessage`: `dequeue()`; si es null el turno termina; `actor.invoke(next)`; `processAllSystemMessages()`; continúa sólo si `left > 1 && (!deadlineDefinido || nanoTime - deadline < 0)` (`:263-280`). Se procesa siempre al menos un mensaje si hay; el deadline se comprueba después de cada mensaje (un mensaje largo no se interrumpe).
3. `processAllSystemMessages()`: `systemDrain` a una lista EarliestFirst; mientras haya elementos y `!isClosed`: `unlink`, `actor.systemInvoke(msg)`; cuando la lista local se vacía y sigue abierto vuelve a drenar (`:302`), es decir, la cola de sistema se vacía por completo antes de cualquier mensaje de usuario ("don't ever execute normal message when system message present").
4. Si el mailbox se cerró procesando un mensaje (p. ej. Terminate), los mensajes de sistema restantes de la lista local se reenvían a `deadLetterMailbox.systemEnqueue(actor.self, msg)` (`:308-317`).
5. Interrupción de hilo: tras `invoke` -> `InterruptedException("Interrupted while processing actor messages")`; tras `systemInvoke` se acumula y se relanza al final como `InterruptedException("Interrupted while processing system messages")` limpiando el flag (`:274-276,299-300,328-332`). `exec()` captura `InterruptedException` (re-interrumpe) y para cualquier otro `Throwable` invoca el `UncaughtExceptionHandler` del hilo y relanza (`:244-258`). No existe equivalente en Rust.

**Errores.** Un `NonFatal` al reenviar un mensaje de sistema al deadLetterMailbox publica `Error(e, actor.self.path.toString, this.getClass, "error while enqueuing " + msg + " to deadLetters: " + e.getMessage)` (`:318-326`; el sufijo es el mensaje de la excepción, no el del mensaje de sistema). Los fallos del actor los absorbe `ActorCell.invoke` (AC-CELL).

**Orden / concurrencia.** Un mailbox nunca corre en dos hilos a la vez (bit Scheduled + CAS). Equidad entre actores = `throughput`: con throughput `n` el actor cede el hilo cada `n` mensajes (o al vencer el deadline). `throughput <= 0` equivale a 1 (Q-DSP-5).

**Dependencias.** AC-MBX-001, AC-DSP-002, AC-DSP-003, AC-CELL-013.1 (prioridad de mensajes de sistema).

**Equivalente Rust propuesto.** `MailboxRunner::run_turn(&mut self, clock: &dyn Clock)` ejecutado por el executor del dispatcher (un worker por turno); el reloj del deadline es inyectable para pruebas deterministas. Conserva: orden sistema-primero, límite de mensajes por turno, corte por deadline, reprogramación. Diferencias: no hay interrupción de hilo; con un executor async el "turno" es un poll que hace `yield` al terminar (Q-MBX-6).

**Aserciones.**
- A1 `DispatcherActorSpec.scala:84-110` ("respect the throughput setting"): dispatcher con `throughput = 101` y `thread-pool-executor.fixed-pool-size = 1`. `slowOne` responde "OK" a "hogexecutor" y bloquea en `start.await()`; se encolan 100 "ping" en `slowOne` y "sabotage" en `fastOne` (que pone `works=false`); se libera `start`. Se afirma `latch.getCount() == 0` (los 100 pings cuentan): con un solo hilo, `slowOne` procesa hogexecutor + 100 pings = 101 mensajes en un turno antes de que `fastOne` obtenga el hilo (101 es exactamente el límite).
- A2 `DispatcherActorSpec.scala:112-141` ("respect throughput deadline"): `throughput = 2`, `throughput-deadline-time = 100 ms`, un hilo. Se envían `slowOne ! hogexecutor`, `slowOne ! ping`, `fastOne ! ping`; tras `ready` se duerme 110 ms y se libera. Se afirma que `fastOne` llega a contar el latch (`works` sigue true): al vencer el deadline durante `hogexecutor` el turno de `slowOne` termina aunque `left > 1` y `fastOne` corre antes que el "ping" de `slowOne`.
- A3 `ActorModelSpec.scala:331-347` ("handle queueing from multiple threads"): 10 hilos x 20 `WaitAck(1)`; los 200 se procesan en <=3 s con `registers=1, msgsReceived=msgsProcessed=200` (sin pérdida, y el mailbox se reprograma mientras queden mensajes).
- A4 `ActorModelSpec.scala:311-329`: un mensaje a la vez (ver AC-MBX-001 A2).
- A5 sistema antes que usuario dentro del turno, re-chequeo de sistema tras cada mensaje, reenvío al deadLetterMailbox al cerrar y mensaje de error literal: *no afirmada por spec* aquí (AC-CELL-013.1 cubre el orden desde la celda). Caracterización propuesta: (a) encolar 3 mensajes de usuario y 1 de sistema, el sistema se procesa primero; (b) un mensaje de usuario que provoca un mensaje de sistema -> el de sistema corre antes del siguiente de usuario; (c) Terminate con mensajes de sistema posteriores en la misma ráfaga -> terminan en el deadLetterMailbox.

**Validación prevista.** A1-A4 portados 1:1 con un executor de 1 hilo; A1/A2 requieren pool de tamaño fijo 1 y (A2) reloj inyectable o un sleep real de 110 ms; A5 caracterización. Oráculo: no aplica (comportamiento temporal).

**Decisiones.** Q-DSP-5 (throughput <= 0), Q-MBX-5, Q-MBX-6.

## AC-MBX-003 Mailbox cerrado y limpieza

*unidad MBX · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Al desvincular un actor (`detach` -> `unregister`) su mailbox se reemplaza por el `deadLetterMailbox`, se cierra y se limpia: los mensajes de sistema pendientes pasan a `deadLetterMailbox.systemEnqueue` y los mensajes de usuario pendientes se mueven (vía `messageQueue.cleanUp`) al `deadLetterMailbox.messageQueue`, que los publica como `DeadLetter(msg, envelope.sender, receiver)`. Desde ese momento todo envío al actor va a dead letters.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:279-285` (`unregister`), `Mailbox.scala:340-354` (`cleanUp`), `:473-503` (`DefaultSystemMessageQueue`, centinela `NoMessage`), `:408-414,442-448,518-526` (`cleanUp` por tipo de cola).
- `actor/src/main/scala/org/apache/pekko/dispatch/BalancingDispatcher.scala:78-89` (SharingMailbox.cleanUp), `Mailboxes.scala:54-72` (deadLetterMailbox).
- `actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala:247-262` (orden de `finishTerminate`: `aroundPostStop` antes de `dispatcher.detach`).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithBoundedStashSpec.scala:122-149`; `DeathWatchSpec.scala:145-152`.

**Precondiciones.** El actor estaba registrado en el dispatcher; `detach` se invoca una sola vez (ver AC-DSP-001: dos `unregister` romperían el contador de inhabitants).

**Comportamiento.**
1. `unregister`: `addInhabitants(-1)`; `mailBox = actor.swapMailbox(deadLetterMailbox)`; `mailBox.becomeClosed()`; `mailBox.cleanUp()`.
2. `cleanUp()` (si `actor ne null`; el deadLetterMailbox tiene `actor == null` y no hace nada): `systemDrain(new LatestFirstSystemMessageList(NoMessage))` deja el centinela `NoMessage` como cabeza de la cola de sistema, de modo que cualquier `systemEnqueue` posterior sobre el mailbox viejo redirige a `deadLetterMailbox.systemEnqueue` (`:480-482`); cada mensaje drenado se desvincula (`unlink`, requisito para reencolar) y se envía a `dlm.systemEnqueue(actor.self, msg)`; después `messageQueue.cleanUp(actor.self, dlm.messageQueue)` (si `messageQueue ne null`, requisito de CallingThreadDispatcher).
3. `cleanUp` de las colas: las basadas en `Queue` y `NodeMessageQueue`/`BoundedNodeMessageQueue` desencolan todo y llaman `deadLetters.enqueue(owner, envelope)`; `LoggerMailbox` filtra (AC-MBX-020); el `SharingMailbox` del BalancingDispatcher sólo drena mensajes de sistema y NO limpia la cola compartida (`BalancingDispatcher.scala:79-90`).
4. Efecto del deadLetterMailbox (AC-MBX-021): usuario -> `DeadLetter(msg, sender, receiver)`; sistema -> `deadLetters ! DeadLetter(handle, receiver, receiver)`.

**Errores.** Si `inhabitants` queda < 0 al hacer `addInhabitants(-1)` se lanza `IllegalStateException` (AC-DSP-001).

**Orden / concurrencia.** `finishTerminate` ejecuta `aroundPostStop` (que en un `Stash` hace `unstashAll`, reencolando al frente del mailbox todavía vivo) ANTES de `detach`; por eso los mensajes estachados terminan como dead letters en su orden original. Los mensajes de usuario drenados salen en el orden de `dequeue` del mailbox (FIFO, con los desestachados al frente).

**Dependencias.** AC-MBX-021, AC-DSP-001, AC-CELL-008.2, AC-CELL-013.3, AC-CELL-014 (familia SystemMessage, fuera de este grupo).

**Equivalente Rust propuesto.** `Mailbox::clean_up(&self, dead_letters: &DeadLetterMailbox)`; centinela de cola de sistema cerrada como variante `SystemQueue::Closed`. Sin diferencias observables.

**Aserciones.**
- A1 `ActorWithBoundedStashSpec.scala:119-133` (`testDeadLetters`): mailbox `BoundedDequeBasedMailbox(10, 500 ms)` y `stash-capacity = 20`; 11 "hello" estachados, "world" fuerza `unstashAll` con violación de capacidad -> `DeadLetter("hello1", testActor, stasher)`; después `PoisonPill` -> los 10 restantes llegan como `DeadLetter("hello"+n, testActor, stasher)` para n = 2..11, en orden. Prueba el `cleanUp` del message queue acotado en `detach` y el orden.
- A2 `ActorWithBoundedStashSpec.scala:135-147` (`testStashOverflowException`): 20 estachados + 1 -> "STASHOVERFLOW", el actor se detiene y los 20 llegan como `DeadLetter` en orden 1..20.
- A3 `DeathWatchSpec.scala:145-152` ("notify with one Terminated message when an Actor is already dead"): vigilar un actor ya muerto produce un único `Terminated`; el `Watch` (mensaje de sistema) llega al deadLetterMailbox y `DeadLetterActorRef.specialHandle` lo convierte en `DeathWatchNotification` (`ActorRef.scala:745-750`).
- A4 centinela `NoMessage` y redirección de `systemEnqueue` tras la limpieza; no reencolado de la cola compartida del SharingMailbox: *no afirmada por spec*. Caracterización propuesta: encolar un mensaje de sistema después de `cleanUp` y comprobar que llega al deadLetterMailbox; limpiar un SharingMailbox y comprobar que la cola compartida conserva sus mensajes.
- Corrección del inventario: la fila citaba `ActorRefSpec.scala:422` ("stop when sent a poison pill", `:422-437`) como prueba de "10 mensajes + PoisonPill -> 10 DeadLetter". Ese test sólo hace `ask(5)`, `ask(0)`, `PoisonPill` y espera "five", "null" y `Terminated`; no afirma dead letters. El escenario de 10 dead letters es A1 (`ActorWithBoundedStashSpec`).

**Validación prevista.** A1-A3 portados 1:1 (A3 pertenece a la unidad DW; se porta allí y aquí se verifica la integración); A4 caracterización. Oráculo: no aplica.

**Decisiones.** Q-MBX-5.

## AC-MBX-004 SPI MessageQueue / MailboxType / ProducesMessageQueue

- **Contrato observable**: `MessageQueue{enqueue, dequeue (null si vacía), numberOfMessages, hasMessages, cleanUp(owner, deadLetters)}`; `MailboxType.create(owner: Option[ActorRef], system: Option[ActorSystem])`; mailbox-type configurable por FQCN con constructor `(Settings, Config)`.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:376-410,668-674`; `Mailboxes.scala:241-255`; `actor/src/main/resources/reference.conf:678-682,733-780`.
- **Condiciones**: constructor inexistente => `IllegalArgumentException("Cannot instantiate MailboxType [<fqcn>], defined in [<id>], make sure it has a public constructor with [org.apache.pekko.actor.ActorSystem.Settings, com.typesafe.config.Config] parameters", causa)` (`Mailboxes.scala:249-252`); `mailbox-type` vacío => `ConfigurationException("The setting mailbox-type, defined in [<id>] is empty")` (`:242`).
- **Orden**: múltiples productores, un consumidor como mínimo (`Mailbox.scala:360`).
- **Dependencias**: AC-CFG-016, Q-X-2.
- **Equivalente Rust**: `trait MessageQueue`, `trait MailboxType { fn create(&self, owner: Option<&ActorRef>, system: Option<&ActorSystem>) -> Box<dyn MessageQueue> }` y registro por nombre (Q-DSP-2).
- **Aserciones**: A1 mailbox custom por dispatcher se usa para el actor (`queue.getClass == classOf[MyMailbox]`) — `MailboxConfigSpec.scala:255-265` (la creación correcta con `owner = Some` es lo que se verifica; `MyMailboxType` lanza "no mailbox owner given" con `None`, `:243-248`, pero ningún test llama `create(None, ...)` sobre él); A2 mensajes de error: derivados de `Mailboxes.scala:242,249-252` (*no afirmada por spec*).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-2, Q-DSP-2.

## AC-MBX-005 UnboundedMailbox

- **Contrato observable**: FIFO sin límite, multi-consumidor (`ConcurrentLinkedQueue`), `remainingCapacity == Int.MaxValue`.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:557-562,677-689`; `reference.conf:635`.
- **Condiciones**: `dequeue()` en vacía => null; `numberOfMessages` consistente con el tamaño (`queue.size`, O(n)).
- **Orden**: FIFO por productor; con varios productores, orden de llegada.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: `UnboundedMailbox` sobre `crossbeam::queue::SegQueue`; "null" => `Option::None`.
- **Aserciones**: A1 estado inicial 0/false y `remainingCapacity == Int.MaxValue` — `MailboxConfigSpec.scala:42-46,127-140`; A2 tras 100 enqueue el tamaño crece de a 1 (o `-1`) y se desencola en el mismo orden hasta null — `MailboxConfigSpec.scala:107-125` (`ensureSingleConsumerEnqueueDequeue`, usado en `:48-50`); A3 10000 mensajes con 4 productores (step 500) y 4 consumidores concurrentes: sin duplicados ni pérdidas — `MailboxConfigSpec.scala:142-192` (usado en `:77-79`).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-006 SingleConsumerOnlyUnboundedMailbox

- **Contrato observable**: MPSC sin límite (`NodeMessageQueue` sobre `AbstractNodeQueue`, cola de Vyukov); sin `MultipleConsumerSemantics`, por lo que no es apta para BalancingDispatcher; `count()` es una estimación O(n) sobre una cola "en movimiento".
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:412-429,699-704`; `actor/src/main/java/org/apache/pekko/dispatch/AbstractNodeQueue.java:14-243` (leído: `add` = `getAndSet` del head + `setNext`; `poll` con spin cuando el productor aún no enlazó; `isEmpty` compara tail con head).
- **Condiciones**: `cleanUp` mueve todo a deadLetters (`:408-414`).
- **Orden**: FIFO; un solo consumidor.
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: canal MPSC (`crossbeam::channel` o cola intrusiva); `number_of_messages` puede ser exacto o estimado (permitido por AC-MBX-019). Licencia BSD (docs/upstream.md) si se porta el nodo.
- **Aserciones**: A1 las mismas pruebas de cola con 1 consumidor (`maxConsumers = 1`) — `MailboxConfigSpec.scala:269-277`; A2 ping-pong patológico de 2.000.000 mensajes (throughput 1) termina en 30 s, no acotado y acotado — `MailboxConfigSpec.scala:279-329`.
- **Oráculo**: no aplica (rendimiento).
- **Decisiones**: Q-MBX-6.

## AC-MBX-007 BoundedMailbox

- **Contrato observable**: cola acotada (`LinkedBlockingQueue`); si está llena `enqueue` espera `pushTimeOut`; vencido => `DeadLetter(msg, sender, receiver)` a `provider.deadLetters`; `pushTimeOut < 0` => `put` bloqueante infinito; `0` => `offer` inmediato.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:568-595,729-750`; `reference.conf:637-647` (`mailbox-capacity=1000`, `mailbox-push-timeout-time=10s`).
- **Condiciones**: `capacity<0` => `IllegalArgumentException("The capacity for BoundedMailbox can not be negative")` (`:713`); `pushTimeOut null` => `IllegalArgumentException("The push time-out for BoundedMailbox can not be null")` (`:714`). `capacity == 0` pasa esa validación pero `LinkedBlockingQueue(0)` lanza `IllegalArgumentException` de la JDK (sin mensaje propio) al crear la cola.
- **Orden/concurrencia**: el emisor se bloquea (hilo que ejecuta `tell`) mientras espera; cola `LinkedBlockingQueue`.
- **Dependencias**: AC-MBX-004, AC-MBX-022.
- **Equivalente Rust**: cola acotada con envío awaitable y variante no bloqueante (Q-MBX-1). Diferencia observable: bloqueo del emisor sólo con timeout>0.
- **Aserciones**: A1 estado inicial `remainingCapacity==capacity` — `MailboxConfigSpec.scala:127-140` (`:132`); A2 capacidad 10 llena + 1 enqueue (timeout 10 ms) => `DeadLetter(msg, system.deadLetters, testActor)` y tras `dequeue` quedan 9 — `MailboxConfigSpec.scala:56-75`; A3 timeout -1 ms (10000), 0 ms (capacidad 10, 20 enqueue secuenciales, 10 dequeue => 10 "received dead letter") y 100 ms (10000), sin duplicados — `MailboxConfigSpec.scala:81-92,142-192`; A4 `pushTimeOut == Duration.Zero` desde config — `ActorMailboxSpec.scala:338-344`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-1.

## AC-MBX-008 NonBlockingBoundedMailbox

- **Contrato observable**: cola acotada MPSC sin bloqueo (`BoundedNodeMessageQueue`); excedente => `DeadLetter(msg, sender, receiver)` inmediato; ignora push timeout (`pushTimeOut = Duration.Undefined`). Declara `MultipleConsumerSemantics` y `BoundedMessageQueueSemantics` aunque el comentario del mailbox diga que no admite varios consumidores: `poll` hace CAS sobre `deq` y reintenta ante consumidores concurrentes (`AbstractBoundedNodeQueue.java`, `pollNode`).
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:431-463,706-724`; `actor/src/main/java/org/apache/pekko/dispatch/AbstractBoundedNodeQueue.java:14-240` (leído: `add` comprueba `lastNode.count - deq.count < capacity`, `size()` es una aproximación "momentánea", capacidad 0 admitida).
- **Condiciones**: `capacity<0` => `IllegalArgumentException("The capacity for NonBlockingBoundedMailbox can not be negative")` (`Mailbox.scala:696`); a nivel de cola `capacity < 0` => `IllegalArgumentException("AbstractBoundedNodeQueue.capacity must be >= 0")`.
- **Orden**: FIFO; `size()` exacto sólo sin concurrencia (corrección del inventario: es una aproximación según el Javadoc).
- **Dependencias**: AC-MBX-004.
- **Equivalente Rust**: cola acotada lock-free con `try_push`; capacidad 0 admitida (todo va a dead letters).
- **Aserciones**: A1 20 enqueue / 10 dequeue (capacidad 10) produce 10 "received dead letter" (`EventFilter.warning(... occurrences = enqueueN - dequeueN)`) — `MailboxConfigSpec.scala:85-87,142-192` con `SingleConsumerOnlyMailboxSpec` (`:269-277`); A2 ping-pong con capacidad 1 y throughput 1 — `MailboxConfigSpec.scala:325-327`; A3 mensaje de capacidad negativa *no afirmada por spec* (`Mailbox.scala:696`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6.

## AC-MBX-009 UnboundedPriorityMailbox / BoundedPriorityMailbox / PriorityGenerator

*unidad MBX · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Cola de mensajes ordenada por un `Comparator[Envelope]` (habitualmente un `PriorityGenerator(Any => Int)`): `dequeue` devuelve el elemento mínimo según el comparador (menor valor = mayor prioridad). Entre elementos de igual prioridad el orden NO está garantizado (cola binaria).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:752-797` (mailboxes), `Dispatcher.scala:164-183` (`PriorityGenerator`: `compare = gen(a.message) - gen(b.message)`).
- `actor/src/main/scala/org/apache/pekko/util/BoundedBlockingQueue.scala:27-262` (cola acotada del caso acotado), `Mailbox.scala:557-571` (política de `enqueue` compartida).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/dispatch/PriorityDispatcherSpec.scala:25-100`; `MailboxConfigSpec.scala:204-213`.

**Precondiciones.** `Unbounded`: `initialCapacity` por defecto 11 (`:735`); un valor < 1 lo rechaza `PriorityBlockingQueue` con `IllegalArgumentException` de la JDK. `Bounded`: `capacity >= 0` y `pushTimeOut` no nulo.

**Comportamiento.**
- Unbounded: `PriorityBlockingQueue(initialCapacity, cmp)` + `UnboundedQueueBasedMessageQueue` (enqueue = `add`, sin límite).
- Bounded: `BoundedBlockingQueue(capacity, PriorityQueue(11, cmp))` + `BoundedQueueBasedMessageQueue.enqueue`: `pushTimeOut >= 0` -> `offer(handle, timeout)` y, si vence, `DeadLetter(msg, sender, receiver)`; `pushTimeOut < 0` -> `put` bloqueante (idéntico a AC-MBX-007).
- `PriorityGenerator.compare(a, b) = gen(a.message) - gen(b.message)` es una resta de `Int` con desbordamiento de complemento a dos (Q-MBX-7).
- La prioridad sólo ordena los mensajes presentes en la cola cuando se hace `dequeue`: no reordena un mensaje ya entregado al actor.

**Errores.** Capacidad negativa -> `IllegalArgumentException("The capacity for BoundedMailbox can not be negative")`; `pushTimeOut` nulo -> `IllegalArgumentException("The push time-out for BoundedMailbox can not be null")` (el mensaje nombra `BoundedMailbox` aunque la clase sea `BoundedPriorityMailbox`, `:760-761`).

**Orden / concurrencia.** Cola segura para varios consumidores (cumple `MultipleConsumerSemantics` por ser `QueueBased`). El orden de empate lo decide el heap, sin garantía (Q-MBX-4).

**Dependencias.** AC-MBX-004, AC-UTIL-012 (BoundedBlockingQueue).

**Equivalente Rust propuesto.** `PriorityMailbox<C: Fn(&Envelope, &Envelope) -> Ordering>` sobre `BinaryHeap` (mínimo primero) con `Mutex`; el caso acotado reutiliza el contador de capacidad y la política de timeout de AC-MBX-007. `PriorityGenerator` expone `compare` con resta `i32` envolvente (`wrapping_sub`) para paridad (Q-MBX-7). Diferencia: sin garantía de orden entre iguales (igual que Pekko).

**Aserciones.**
- A1 `PriorityDispatcherSpec.scala:58-66,69-98`: un actor hijo (no top-level, para que los mensajes no pasen por `UnstartedCell`) con `mailbox-type` = `Unbounded` o `Bounded(…, 1000, 10.seconds)` envía en su constructor los enteros 1..100 barajados y luego `Result` (prioridad `Int.MaxValue`); acumula en orden de recepción y responde `acc.toList`; se afirma que la lista es `(1 to 100)`, es decir, orden ascendente de prioridad con `Result` el último. Las prioridades son distintas: el spec NO ejerce empates.
- A2 `MailboxConfigSpec.scala:204-213` (+ pruebas genéricas `:40-92,107-192`): con comparador `_.##` el mailbox pasa las pruebas de estado inicial, FIFO de 100 idénticos, 4 consumidores concurrentes y el caso acotado (DeadLetter a capacidad llena).
- A3 orden entre iguales: *no afirmada por spec* (no caracterizar el orden; sólo que no se pierdan ni dupliquen, como A2).
- A4 desbordamiento de `gen(a) - gen(b)` y mensajes de validación: *no afirmada por spec*. Caracterización propuesta: prioridades `Int.MinValue` vs `Int.MaxValue` comparadas con la resta envolvente; oráculo diferencial con `jshell` y el jar oficial (`PriorityGenerator.apply(...).compare(...)`).

**Validación prevista.** A1 portado (enviar los mensajes en `pre_start` del hijo para que estén todos en la cola antes del primer `dequeue`) para no acotado y acotado; A2 portado; A4 caracterización + oráculo diferencial puntual (función pura).

**Decisiones.** Q-MBX-4 (empates), Q-MBX-7 (desbordamiento del comparador), Q-MBX-1 (bloqueo del caso acotado).

## AC-MBX-010 Stable priority mailboxes

*unidad MBX · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Igual que AC-MBX-009 pero los mensajes de igual prioridad salen en el orden en que fueron encolados (FIFO por número de secuencia asignado en `offer`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:799-846`.
- `actor/src/main/scala/org/apache/pekko/util/StablePriorityQueue.scala:14-87` (`PriorityQueueStabilizer`: cada `offer` envuelve el elemento con `seqNum.incrementAndGet`; `WrappedElementComparator` desempata con `signum(seqNum1 - seqNum2)`).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/dispatch/StablePriorityDispatcherSpec.scala:25-107`; `MailboxConfigSpec.scala:215-224`.

**Precondiciones.** Como AC-MBX-009 (capacidad >= 0, `pushTimeOut` no nulo con el mismo mensaje "BoundedMailbox", `:809-810`); el `capacity` de `StablePriorityQueue` debe ser > 0 (`PriorityQueue`).

**Comportamiento.** Orden total (prioridad, secuencia de encolado). El número de secuencia lo da un contador atómico por cola; con varios productores el orden FIFO es el de llegada a `offer`. El caso acotado usa `BoundedBlockingQueue` con `StablePriorityQueue` y la política de timeout de AC-MBX-007.

**Errores.** Como AC-MBX-009.

**Orden / concurrencia.** El desempate es determinista (secuencia), por lo que es el único orden de empate afirmado por spec en Pekko.

**Dependencias.** AC-MBX-004, AC-UTIL-013 (StablePriorityQueue).

**Equivalente Rust propuesto.** `BinaryHeap` de `(Envelope, u64)` con el comparador y la secuencia como segundo criterio; contador `AtomicU64`.

**Aserciones.**
- A1 `StablePriorityDispatcherSpec.scala:59-69,71-105`: 200 enteros barajados enviados por `self` en el constructor de un hijo (prioridad = el entero si <= 100, 101 para los mayores, `Int.MaxValue` para `Result`); el resultado afirmado es `lo ++ hi`, con `lo = 1..100` ordenado y `hi` = los enteros > 100 en el MISMO orden barajado en que se enviaron (FIFO entre iguales). Para cola no acotada y acotada (`1000, 10.seconds`).
- A2 `MailboxConfigSpec.scala:215-224` (+ genéricas): pasa las pruebas de cola con `_.##`.
- A3 mensajes de validación: *no afirmada por spec* (`Mailbox.scala:809-810`).

**Validación prevista.** A1 y A2 portados; A3 caracterización.

**Decisiones.** Q-MBX-4 (aquí no hay incertidumbre: el estable es FIFO), Q-MBX-7, Q-MBX-1.

## AC-MBX-011 Mailboxes deque-based

*unidad MBX · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `UnboundedDequeBasedMailbox` (`LinkedBlockingDeque`) y `BoundedDequeBasedMailbox(capacity, pushTimeOut)` añaden `enqueueFirst(receiver, envelope)`, que coloca el mensaje al frente (lo consume `Stash`). `enqueue` sigue añadiendo al final; `dequeue` toma el frente.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:597-653` (traits), `:824-868` (mailboxes), `reference.conf:686-733` (secciones `unbounded-deque-based`/`bounded-deque-based`).
- Consumidor: `actor/src/main/scala/org/apache/pekko/actor/Stash.scala:244-276`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:260-265,271-282,358-438`; `ActorWithStashSpec.scala:121-139`; `ActorWithBoundedStashSpec.scala:119-133`; `StashJavaAPI.java:41-54`.

**Precondiciones.** Bounded: `capacity >= 0` y `pushTimeOut` no nulo; `LinkedBlockingDeque(0)` falla en la JDK.

**Comportamiento.** Unbounded: `enqueue = addLast`, `enqueueFirst = addFirst`, `dequeue = pollFirst`. Bounded: `enqueue` y `enqueueFirst` usan `offer`/`offerFirst` con timeout (`pushTimeOut >= 0`) o `put`/`putFirst` (`< 0`); si vencen publican `DeadLetter(msg, sender, receiver)` (no lanzan excepción, pese a que el Scaladoc de `Stash.unstash` habla de `MessageQueueAppendFailedException`).

**Errores.** `IllegalArgumentException("The capacity for BoundedDequeBasedMailbox can not be negative")` (`:854`) y `("The push time-out for BoundedDequeBasedMailbox can not be null")` (`:855-856`).

**Orden / concurrencia.** `enqueueFirst` repetido invierte el orden (último en entrar = primero en salir); por eso `unstashAll` itera el stash al revés (AC-MBX-016). Cola multi-consumidor.

**Dependencias.** AC-MBX-004, AC-MBX-016, AC-UTIL-012.

**Equivalente Rust propuesto.** `DequeMailbox` sobre `Mutex<VecDeque<Envelope>>` + `Condvar` para el caso acotado; `push_front` para `enqueue_first`.

**Aserciones.**
- A1 `ActorMailboxSpec.scala:259-264,270-281,287-289`: `Props.withMailbox("...unbounded-deque-based")`, un actor con `Stash` y `deployment.mailbox` deque obtienen colas que cumplen `Deque/Unbounded/UnboundedDequeBased` semantics.
- A2 `ActorMailboxSpec.scala:357-362,374-381,399-404,420-425`: la cola es `BoundedDequeBased` cuando se configura y se requiere (4 formas).
- A3 `ActorWithStashSpec.scala:121-127`: "bye" estachado, "hello" -> `unstashAll`; "bye" se procesa DESPUÉS de "hello" y tras el `become` (`state.s == "bye"`): el mensaje desestachado se reencola al frente.
- A4 `ActorWithStashSpec.scala:129-139` (protocolo open/write/close): `unstashAll` reencola lo estachado por delante de lo posterior, en su orden original.
- A5 `ActorWithBoundedStashSpec.scala:119-133`: con mailbox `BoundedDequeBasedMailbox(10, 500.millis)`, `enqueueFirst` con la cola llena produce `DeadLetter` del mensaje que no cabe tras vencer el timeout.
- A6 `StashJavaAPI.java:41-54`: orden original preservado con las tres variantes de stash (ver AC-MBX-018).

**Validación prevista.** Portar A1-A6; el orden de `enqueueFirst` directo (`enqueue(a); enqueueFirst(b)` -> `dequeue = b, a`) es *no afirmada por spec* de forma aislada: se cubre con A3/A4; añadir caracterización unitaria.

**Decisiones.** Q-MBX-1 (bloqueo del caso acotado), Q-MBX-2 (semánticas de cola).

## AC-MBX-012 Mailboxes control-aware

*unidad MBX · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Los mensajes cuyo tipo implementa `ControlMessage` se desencolan antes que los demás; FIFO dentro de cada clase. Hay variante no acotada (`UnboundedControlAwareMailbox`) y acotada (`BoundedControlAwareMailbox(capacity, pushTimeOut)`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:894-1059`; `reference.conf:676-681,707-725`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/dispatch/ControlAwareDispatcherSpec.scala:20-73`; `MailboxConfigSpec.scala:226-234`; `ActorMailboxSpec.scala:291-314`.

**Precondiciones.** El mailbox bounded se crea por config con `mailbox-capacity` y `mailbox-push-timeout-time` (constructor `(Settings, Config)`). NO valida `capacity` ni `pushTimeOut` (a diferencia de los demás mailboxes acotados).

**Comportamiento.**
- `enqueue`: `Envelope(_: ControlMessage, _)` -> cola de control; cualquier otro -> cola normal (por el tipo dinámico del mensaje).
- `dequeue`: primero `controlQueue.poll()`, si no `queue.poll()`. `numberOfMessages` (no acotado) = suma de los dos `size()`; `hasMessages = !(queue.isEmpty && controlQueue.isEmpty)`.
- Acotado: contador atómico `size` COMPARTIDO por ambas colas (la capacidad es total, control incluido); `enqueueWithTimeout` toma `putLock`, espera mientras `size == capacity` con `awaitNanos(remaining)` hasta `stop = remaining <= 0`; si no se insertó -> `DeadLetter(msg, sender, receiver)`. Con `pushTimeOut = 0` el desborde es inmediato; con `pushTimeOut < 0` también descarta de inmediato (`awaitNanos` con valor negativo devuelve <= 0), a diferencia de `BoundedMailbox`/`BoundedDequeBasedMailbox`, donde un timeout negativo significa esperar indefinidamente (`put`). `dequeue` decrementa `size` por CAS antes de extraer y señala `notFull`.
- `capacity < 0` nunca satisface `size == capacity` (comportaría como no acotado).

**Errores.** Ninguno de validación (ver Precondiciones). Desborde -> DeadLetter.

**Orden / concurrencia.** El orden global es "control primero, luego normal"; no hay reordenamiento dentro de cada clase. Un mensaje de control no desplaza a uno normal que ya se entregó al actor.

**Dependencias.** AC-MBX-004; AC-MBX-015 (semánticas ControlAware).

**Equivalente Rust propuesto.** Dos colas FIFO bajo un `Mutex` (o dos `SegQueue`) y un `AtomicUsize` de tamaño para el caso acotado; la clasificación de control mediante un hook configurable (Q-MBX-3: registro de `TypeId` de control y/o predicado por mailbox), porque `dyn Any` no expone marcadores. Se conserva la semántica de timeout negativo propia de este mailbox (descarte inmediato) y la ausencia de validación.

**Aserciones.**
- A1 `ControlAwareDispatcherSpec.scala:37-47,48-72`: un hijo con `UnboundedControlAwareMailbox` o `BoundedControlAwareMailbox` envía en su constructor "test", "test2" e `ImportantMessage` (un `ControlMessage`) a sí mismo; `testActor` recibe `ImportantMessage`, "test", "test2" en ese orden.
- A2 `MailboxConfigSpec.scala:226-234` (+ genéricas `:40-92,107-192`): pasa las pruebas de cola; la variante acotada con capacidad 10 y timeout 10 ms entrega `DeadLetter` al 11.º (`:56-75`), y con timeout 0 (20 encolados, 10 desencolados) produce 10 dead letters (`:85-87`); los casos `-1 ms` y `100 ms` usan capacidad 10000 sin llegar a llenar, por lo que el descarte inmediato con timeout NEGATIVO no está ejercido.
- A3 `ActorMailboxSpec.scala:291-314`: elegido por `deployment.mailbox` y por `RequiresMessageQueue[...ControlAware...]`.
- A4 capacidad compartida y timeout negativo -> descarte: *no afirmada por spec* (derivada de `Mailbox.scala:1000-1033`). Caracterización propuesta: con capacidad 2, un control y un normal llenan la cola; el tercero (de cualquier clase) va a dead letters; con `pushTimeOut = -1ms` el envío a cola llena descarta de inmediato.

**Validación prevista.** A1-A3 portados; A4 caracterización unitaria (hilo único + reloj inyectable para el timeout).

**Decisiones.** Q-MBX-3 (detección de `ControlMessage`), Q-MBX-1 (semántica de bloqueo del caso acotado).

## AC-MBX-013 Precedencia de selección de mailbox

- **Contrato observable**: ver TSV (contrato corregido).
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:132-201`; `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:712-727`; `actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:91-103`.
- **Condiciones**: requisito del dispatcher `mailbox-requirement` ("" = sin requisito, `Mailboxes.scala:133-135`) y del actor (`RequiresMessageQueue[T]`, `:117-127`); `getRequiredType` con tipo comodín => `IllegalArgumentException("no wildcard type allowed in RequireMessageQueue argument (was [x])")` (`:123`). Cola producida que no cumple => IAE (textos en TSV) envuelta en `ConfigurationException` (`ActorRefProvider.scala:715-718`).
- **Orden**: la creación del mailbox ocurre al crear el actor, el fallo se retrasa a la creación (`Dispatch.scala:86-88`).
- **Dependencias**: AC-MBX-014, AC-MBX-015, AC-DSP-007.
- **Equivalente Rust**: `Mailboxes::mailbox_type_for(props, dispatcher_config) -> Result<MailboxType, ConfigurationError>`.
- **Aserciones** (todas `tests/actor/ActorMailboxSpec.scala`; el helper `checkMailboxQueue` `:244-251` crea el actor, envía "ping" y comprueba con `isInstance` que la cola cumple cada trait esperado): A1 por defecto cola no acotada `:255-257`; A2 Props.withMailbox deque `:259-264`; A3 RequiresMailbox acotado `:266-268`; A4 Stash => deque ilimitada (4 formas de Props) `:270-281`; A5 deployment.mailbox acotado `:283-285`, deque `:287-289`, control-aware `:291-300`; A6 `RequiresMessageQueue` control-aware `:302-314`; A7 deque configurado como mailbox con actor que requiere acotada => ConfigurationException `:316-319`; A8 mailbox en dispatcher `:321-323`, y conflicto con requisito del actor => ConfigurationException `:325-328`; A9 mailbox de deployment gana sobre dispatcher `:330-332,346-348`; A10 dispatcher acotado `:334-336`; A11 `Props.withMailbox` pierde frente a deployment `:350-355`; A12 combinaciones de `requiring-bounded-dispatcher`/stash `:357-437` (incluye fallos por cola no acotada o no configurada); A13 BalancingDispatcher (con y sin requirement, `requiring-balancing-bounded-dispatcher` vía la tabla `requirements`) `:439-458`. Los 33 tests verificados por conteo (`ActorMailboxSpec.scala:255-458`).
- **Oráculo**: no aplica (sólo Pekko compilado podría confirmar; no hay jar de oráculo para esto).
- **Decisiones**: Q-MBX-2.

## AC-MBX-014 Lookup de mailboxes

- **Contrato observable**: ver TSV.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:40-43,105-110,208-283`; `reference.conf:631-734`; typed `PropsAdapter.scala:45`.
- **Condiciones**: textos de error en TSV. Los ids se cachean en `ConcurrentHashMap` (`:74,272-277`): misma instancia por id.
- **Orden**: carrera benigna (`:224-226`).
- **Dependencias**: AC-CFG-001.
- **Equivalente Rust**: `Mailboxes::lookup(id)` con caché `DashMap`/`RwLock<HashMap>`.
- **Aserciones**: A1 requirement por tipo resuelve a mailbox correcto — `ActorMailboxSpec.scala:266-268,302-314` (bound/control-aware); A2 ids `unbounded`, `bounded` y `bounded-capacity:N`: *no afirmadas por spec* (derivadas de `Mailboxes.scala:229-235`); A3 mapeo de requirement de usuario (`MCBoundedMessageQueueSemantics` -> `mc-bounded-mailbox` vía `pekko.actor.mailbox.requirements`) — `ActorMailboxSpec.scala:176-179,453-458`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-1, Q-DSP-2.

## AC-MBX-015 Semánticas de cola y RequiresMessageQueue

- **Contrato observable**: marcadores: MultipleConsumer, Unbounded, Bounded (con `pushTimeOut`), DequeBased(+Unbounded/Bounded), ControlAware(+Unbounded/Bounded), Logger. `RequiresMessageQueue[T]` en el actor; `ProducesMessageQueue[T]` en el MailboxType.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:533-653,897-923,1062-1070,672`; `Mailboxes.scala:112-152`.
- **Condiciones**: un MailboxType que no declara `ProducesMessageQueue` produce `MessageQueue` genérico (`Mailboxes.scala:140`).
- **Orden**: n/a.
- **Dependencias**: AC-MBX-013.
- **Equivalente Rust**: `QueueSemantics` bitflags + `fn required_queue(&self) -> QueueSemantics` en el actor (Q-MBX-2).
- **Aserciones**: A1 cada mailbox satisface las semánticas esperadas (listas `UnboundedMailboxTypes`, `BoundedMailboxTypes`, `UnboundedDeqMailboxTypes`, `BoundedDeqMailboxTypes`, `Bounded/UnboundedControlAwareMailboxTypes`) — `ActorMailboxSpec.scala:202-222` y `:244-251`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-2.

## AC-MBX-016 Stash

- **Contrato observable**: ver TSV.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/actor/Stash.scala:125-277`; `Mailboxes.scala:295-331`; `reference.conf:649-653`; docs `docs/src/main/paradox/actors.md:1088-1146` (sección Stash: coincide con el código salvo la `MessageQueueAppendFailedException`, que la documentación afirma y el código no lanza).
- **Condiciones**: textos exactos en TSV. El mensaje estachado se elimina del stash aunque `unstash()` falle (`Stash.scala:210-217`). `stash()` compara por identidad (`currMsg eq theStash.last`): sólo detecta el doble stash del MISMO mensaje consecutivo (el último del stash). Corrección: el Scaladoc (`Stash.scala:203-206,221-224`) dice que un mailbox acotado desbordado lanza `MessageQueueAppendFailedException`; el código no lo hace: `enqueueFirst` del mailbox acotado publica `DeadLetter` y no lanza (A5).
- **Orden**: `unstashAll` recorre el stash al revés y hace `enqueueFirst` => orden original delante del resto (`:245-249`).
- **Dependencias**: AC-MBX-011, AC-MBX-022, AC-MBX-023.
- **Equivalente Rust**: `StashBuffer<Envelope>` en el contexto con `Vec` y `push_front` del mailbox deque.
- **Aserciones**: A1 stash + unstashAll + become procesa "bye" tras "hello" — `ActorWithStashSpec.scala:121-127`; A2 protocolo open/write/close con unstash — `:129-139`; A3 doble stash => IllegalStateException — `:141-147`; A4 Terminated estachado se re-recibe (2 veces) — `:186-192`; A5 desborde de mailbox al unstashAll => DeadLetter del primero — `ActorWithBoundedStashSpec.scala:119-128`; A6 stash-capacity=20: el 21 lanza StashOverflowException — `:134-146` (por dispatcher y por mailbox `:151-169`; alias `:171-174`).
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-1.

## AC-MBX-017 Stash en restart y stop

- **Contrato observable**: preRestart y postStop hacen `unstashAll`; en stop los mensajes terminan en deadLetters.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/actor/Stash.scala:81-102`.
- **Condiciones**: `unstashAll` se ejecuta en `try/finally` antes de `super.preRestart/postStop`.
- **Orden**: antes de que la nueva instancia procese; en stop, antes de `detach` (AC-MBX-003).
- **Dependencias**: AC-MBX-016.
- **Equivalente Rust**: hooks `pre_restart`/`post_stop` del trait Stash.
- **Aserciones**: A1 tras crash el nuevo actor recibe "hello" estachado — `ActorWithStashSpec.scala:149-184`; A2 tras PoisonPill los estachados llegan a deadLetters en orden — `ActorWithBoundedStashSpec.scala:119-146`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-018 UnboundedStash / UnrestrictedStash / StashFactory / clearStash

*unidad MBX · visibilidad public/internal · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `Stash` exige `DequeBasedMessageQueueSemantics`; `UnboundedStash` exige `UnboundedDequeBasedMessageQueueSemantics`; `UnrestrictedStash` no exige nada (el mailbox debe ser deque o falla la creación del stash). `StashFactory` crea stashes adicionales (aislados del stash de usuario) para actores internos; `clearStash()` devuelve y vacía; `unstashAll(filtro)` reencola sólo los que cumplen el filtro y vacía todo el stash.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Stash.scala:70-124` (traits y `StashFactory`), `:149-173` (capacidad y comprobación del mailbox), `:244-264` (`unstashAll(filtro)`, `clearStash`).
- `actor/src/main/scala/org/apache/pekko/actor/AbstractActor.scala` (`AbstractActorWithStash`, `AbstractActorWithUnboundedStash`, `AbstractActorWithUnrestrictedStash`).
- Pruebas: `actor-tests/src/test/java/org/apache/pekko/actor/StashJavaAPI.java:40-53`, `StashJavaAPITestActors.java:24-80`; `AbstractFSMWithStashActorTest.java`. Consumo interno (fuera de módulo): persistence/persistence-typed.

**Precondiciones.** `Stash`/`UnboundedStash` se resuelven por `RequiresMessageQueue` (AC-MBX-013); `UnrestrictedStash` requiere que el mailbox (p. ej. `Props.withMailbox("pekko.actor.mailbox.unbounded-deque-based")`) implemente `DequeBasedMessageQueueSemantics`.

**Comportamiento.** El stash es un `Vector[Envelope]` por instancia de actor con capacidad `stashCapacity(dispatcher, mailbox)` (AC-MBX-023). `unstashAll(filtro)` itera `theStash.reverseIterator.filter(...)`, hace `enqueueFirst` de cada uno y vacía el stash en `finally`. `clearStash()` devuelve el vector y vacía sin reencolar. Un `Terminated` reencolado llama `actorCell.terminatedQueuedFor(ref, None)` para poder volver a recibirse.

**Errores.** Mailbox sin deque con `UnrestrictedStash` -> `ActorInitializationException(self, "DequeBasedMailbox required, got: <clase>\nAn (unbounded) deque-based mailbox can be configured as follows:\n  my-custom-mailbox {\n    mailbox-type = \"org.apache.pekko.dispatch.UnboundedDequeBasedMailbox\"\n  }\n")` (`:164-171`). Cola incompatible con `UnboundedStash`/`Stash` -> `ConfigurationException` en la selección de mailbox (AC-MBX-013).

**Orden / concurrencia.** El stash es estado privado del actor (un solo hilo por turno).

**Dependencias.** AC-MBX-016, AC-MBX-013, AC-MBX-011.

**Equivalente Rust propuesto.** Variantes `StashRequirement { Deque, UnboundedDeque, None }` declaradas por el actor y verificadas en la creación; `StashSupport` como componente reutilizable (para actores internos y para el soporte de persistencia).

**Aserciones.**
- A1 `StashJavaAPI.java:41-54` (+ `StashJavaAPITestActors.java:24-41`): para `WithStash`, `WithUnboundedStash` y `WithUnrestrictedStash` (con `withMailbox(unbounded-deque-based)`) se envían "Hello", "Hello2", "Hello12"; los dos primeros se estachan (`count` 0->1->2), el tercero dispara `unstashAll()`; el probe espera `5` (= `"Hello".length`), la respuesta al PRIMER mensaje reencolado: el orden original se conserva con las tres variantes.
- A2 `AbstractFSMWithStashActorTest.java`: `AbstractFSMWithStash` compila y funciona (stash/unstash en transición de estado).
- A3 `clearStash`, `unstashAll(filtro)`, `StashFactory`, el mensaje de `ActorInitializationException` y el rechazo de `UnboundedStash` con mailbox acotado: *no afirmada por spec*. Caracterización propuesta: filtro que deja pasar sólo ciertos mensajes (y vacía el resto); `UnrestrictedStash` con mailbox no deque -> error literal; `UnboundedStash` con `requiring-bounded-dispatcher` -> `ConfigurationException`.

**Validación prevista.** A1/A2 portados (variantes como tipos de requisito); A3 caracterización. Oráculo: no aplica.

**Decisiones.** Q-MBX-2.

## AC-MBX-019 numberOfMessages aproximado

- **Contrato observable**: puede devolver -1/0 u otra estimación; usar `hasMessages` para presencia.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:118-123,388-393`.
- **Condiciones**: el test acepta -1.
- **Orden**: n/a.
- **Dependencias**: AC-RT-008.
- **Equivalente Rust**: `fn number_of_messages(&self) -> usize` aproximado.
- **Aserciones**: A1 `numberOfMessages` es -1 o exacto, y `hasMessages == (expected != 0)` — `MailboxConfigSpec.scala:99-105`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-020 Requirement LoggerMessageQueueSemantics

*unidad MBX · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** El requisito `org.apache.pekko.event.LoggerMessageQueueSemantics` se mapea (tabla `requirements`) a `pekko.actor.mailbox.logger-queue` con `mailbox-type = org.apache.pekko.event.LoggerMailboxType`. Es un mailbox no acotado cuyo `cleanUp` (al desvincular el actor, p. ej. al apagar el sistema) no envía los mensajes a dead letters sino que reenvía los eventos de log pendientes al `StandardOutLogger`.

**Fuente.**
- `actor/src/main/resources/reference.conf:729-730,775-780`.
- `actor/src/main/scala/org/apache/pekko/event/LoggerMailbox.scala:29-75`; consumidores: `actor/src/main/scala/org/apache/pekko/event/Logging.scala:1155` (`DefaultLogger`), `actor/src/main/scala/org/apache/pekko/event/jul/JavaLogger.scala:33 (1.7.1)`, `slf4j/src/main/scala/org/apache/pekko/event/slf4j/Slf4jLogger.scala`.

**Precondiciones.** `create` exige `owner` y `system` definidos.

**Comportamiento.** `LoggerMailbox` extiende `UnboundedMailbox.MessageQueue`. En `cleanUp(owner, deadLetters)`, si `hasMessages`: lee `system.eventStream.logLevel`; vacía la cola; por cada envelope, sólo si el nivel configurado está en `Logging.AllLogLevels` (no OFF) y el mensaje es un `LogEvent` con `e.level <= logLevel` (igual o más severo), lo envía con `StandardOutLogger.tell(message, sender)`; el resto se descarta SIN pasar por dead letters. Luego `super.cleanUp` (la cola ya está vacía).

**Errores.** `create` con `owner`/`system` ausentes -> `IllegalArgumentException("no mailbox owner or system given")`.

**Orden / concurrencia.** `cleanUp` se llama tras cambiar el mailbox por el deadLetterMailbox (AC-MBX-003), por eso el bucle sin límite termina; los eventos se emiten en orden FIFO.

**Dependencias.** AC-MBX-003, AC-MBX-014, AC-EVT-011 (loggers), Logging.StandardOutLogger (unidad EVT; frontera: este bloque define sólo el drenado).

**Equivalente Rust propuesto.** Un `LoggerMailbox` (cola no acotada con `clean_up` que filtra por nivel y escribe a un `StandardOutLogger`); si los loggers no son actores con mailbox (Q-X-7) el efecto observable (los eventos pendientes se emiten al apagar, filtrados por nivel) se implementa como vaciado final del canal de log.

**Aserciones.**
- A1 filtrado por nivel, descarte silencioso de lo que no es `LogEvent`, `IllegalArgumentException` del `create` y reenvío a stdout: *no afirmada por spec* (ningún test de actor-tests/testkit/slf4j referencia `LoggerMailbox`; verificado con búsqueda de `LoggerMailbox|LoggerMessageQueueSemantics`). Caracterización propuesta: encolar eventos de varios niveles con `loglevel = INFO`, desvincular el actor y comprobar qué llega al `StandardOutLogger` (stub) y que nada llega a dead letters.

**Validación prevista.** Caracterización unitaria + integración con la unidad EVT.

**Decisiones.** Q-X-7.

## AC-MBX-021 deadLetterMailbox

*unidad MBX · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Mailbox cerrado compartido por sistema. Usuario: `enqueue(receiver, envelope)` publica `DeadLetter(msg, envelope.sender, receiver)` en `deadLetters` salvo que `msg` ya sea un `DeadLetter` (se descarta, para evitar bucles con actores suscritos a `DeadLetter`). Sistema: `systemEnqueue(receiver, handle)` hace `deadLetters ! DeadLetter(handle, receiver, receiver)`; `systemDrain` es siempre vacío; `dequeue() = null`, `hasMessages = false`, `numberOfMessages = 0`, `cleanUp` no hace nada.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:54-72`.
- `actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:656-692,708-724` (cómo `EmptyLocalActorRef`/`DeadLetterActorRef` tratan el `DeadLetter` que envuelve un mensaje de sistema).

**Precondiciones.** El mailbox nace cerrado (`becomeClosed()` en su constructor) y con `actor == null`.

**Comportamiento.** Para mensajes de sistema el `DeadLetter(handle, ...)` recibido por `DeadLetterActorRef` pasa por `specialHandle(d.message, d.sender)`: un `Watch` cuyo watchee y watcher no son `deadLetters` produce `DeathWatchNotification(watchee, existenceConfirmed = false, addressTerminated = false)` al watcher; `Unwatch` se ignora; el resto se publica como `DeadLetter` en el `EventStream` (AC-REF/EVT). Para mensajes de usuario el `DeadLetter` se publica salvo `DeadLetterSuppression` (publicado como `SuppressedDeadLetter`).

**Errores.** Ninguno propio.

**Orden / concurrencia.** Sin estado mutable; cualquier hilo puede encolar.

**Dependencias.** AC-MBX-003, AC-REF-015, AC-EVT-008.

**Equivalente Rust propuesto.** `DeadLetterMailbox` singleton por sistema con un `Sender` hacia el actor `deadLetters`; la regla "no reenviar un `DeadLetter` a sí mismo" se conserva.

**Aserciones.**
- A1 mensajes de usuario terminan como `DeadLetter(msg, sender, receiver)` con el `receiver` del actor detenido — `ActorWithBoundedStashSpec.scala:119-146` (vía `cleanUp`, AC-MBX-003 A1/A2).
- A2 `Watch` sobre un actor muerto -> `Terminated` — `DeathWatchSpec.scala:145-152`.
- A3 descarte de `DeadLetter` ya envuelto, `systemDrain` vacío y los valores de `dequeue/hasMessages/numberOfMessages`: *no afirmada por spec*. Caracterización propuesta: encolar un `DeadLetter` y comprobar que no se republica; encolar un mensaje de sistema `Terminate` y comprobar el `DeadLetter(handle, receiver, receiver)`.

**Validación prevista.** A1/A2 portados (A2 en DW); A3 caracterización.

**Decisiones.** Q-MBX-5.

## AC-MBX-022 Validación y avisos de configuración de mailbox

- **Contrato observable**: textos en TSV.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:720,737-738,784-785,833-834,878-881`; `Mailboxes.scala:173-178,257-266`.
- **Condiciones**: warning sólo una vez por sistema para el pushTimeOut no cero (`mailboxNonZeroPushTimeoutWarningIssued`, `:131,257-267`; aplica a `ProducesPushTimeoutSemanticsMailbox` con `pushTimeOut.toNanos > 0`); warning de `mailbox-size` una vez (`:179`). Los mailboxes `BoundedControlAwareMailbox` y `NonBlockingBoundedMailbox` NO validan `pushTimeOut`; sólo este último valida `capacity`.
- **Orden**: publicados en EventStream, categoría `mailboxes`.
- **Dependencias**: AC-MBX-007, AC-MBX-014.
- **Equivalente Rust**: validación en constructores (`Result<_, IllegalArgument>`) y `EventStream::publish(Warning)`.
- **Aserciones**: *no afirmadas por spec* (derivadas del código). Caracterización propuesta: texto literal del warning `"Configured potentially-blocking mailbox [<id>] configured with non-zero pushTimeOut (<d>), which can lead to blocking behavior when sending messages to this mailbox. Avoid this by setting `<id>.mailbox-push-timeout-time` to `0`."`, emitido una sola vez por sistema.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-MBX-023 Capacidad del stash por dispatcher/mailbox

- **Contrato observable**: ver TSV.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:285-321`; `reference.conf:649-653`.
- **Condiciones**: precedencia: sección del mailbox, luego dispatcher, luego default-mailbox (`:326-329`); cache por `dispatcher + "-" + mailbox` (`:313`).
- **Orden**: n/a.
- **Dependencias**: AC-MBX-016, AC-DSP-004.
- **Equivalente Rust**: `Mailboxes::stash_capacity(dispatcher_id, mailbox_id) -> i32`.
- **Aserciones**: A1 vía dispatcher, vía mailbox y vía alias de dispatcher — `ActorWithBoundedStashSpec.scala:151-174` (`StashOverflowException` en el 21.º y `DeadLetter` según el caso).
- **Oráculo**: no aplica.
- **Decisiones**: Q-X-1.

---

# Unidad DSP (dispatchers y executors)

## AC-DSP-001 MessageDispatcher: attach/detach, inhabitants, apagado diferido

- **Contrato observable**: `attach` registra el actor (inhabitants+1) y agenda su mailbox con hint de sistema; `detach` desregistra (inhabitants-1), cambia el mailbox del actor por deadLetterMailbox, lo cierra y limpia; si inhabitants llega a 0 se agenda `shutdown()` tras `shutdown-timeout`; reutilizar el dispatcher recrea el executor (delegate lazy). Las tareas `execute(Runnable)` también cuentan como inhabitants (AC-DSP-016.1).
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:107-305`; `Dispatcher.scala:53-66,125-129`; `reference.conf:586-587`.
- **Condiciones**: inhabitants<0 => `IllegalStateException("ACTOR SYSTEM CORRUPTED!!! A dispatcher can't have less than 0 inhabitants!")` tras `reportFailure` (`:129-135`). Scheduler ya cerrado => shutdown inmediato (`:212-219`) y estado vuelto a UNSCHEDULED. Máquina UNSCHEDULED/SCHEDULED/RESCHEDULED (`:191-266`): llegar a 0 con UNSCHEDULED agenda el cierre (SCHEDULED); llegar a 0 con SCHEDULED pasa a RESCHEDULED (se reagenda al vencer); al vencer, `shutdown()` sólo si `inhabitants == 0` ("racy"), y el estado vuelve a UNSCHEDULED. `Dispatcher.shutdown()` reemplaza el delegate por una copia perezosa (`LazyExecutorServiceDelegate`) y apaga el viejo; el siguiente uso crea un executor nuevo.
- **Orden/concurrencia**: CAS sobre contador (`getAndAddLong`) y estado; `shutdown` "racy" (`:254`). El cierre usa `scheduler.scheduleOnce(shutdownTimeout, shutdownAction)` con un EC que ejecuta en el hilo del scheduler; `shutdownAction` es `TaskRunOnClose` (se ejecuta al cerrar el scheduler, AC-SCH-009).
- **Dependencias**: AC-MBX-002, AC-MBX-003, AC-SCH-001, AC-SCH-009.
- **Equivalente Rust**: `trait MessageDispatcher { attach, detach, inhabitants }` y apagado diferido con `Scheduler::schedule_once` (tarea `RunOnClose`). Diferencia: no hay hilos que apagar si el runtime es compartido (Q-X-4); el contador y la máquina de tres estados se conservan.
- **Aserciones**: A1 ciclo de vida `ActorModelSpec.scala:283-309` ("dynamically handle its own life cycle"): con un dispatcher interceptado, tras crear un actor `stops == 0`; tras `system.stop(a)` el dispatcher se apaga (`stops == 1`, esperando hasta `shutdownTimeout * 5`) con `registers == 1, unregisters == 1`; 10 `Future` ejecutados sobre el dispatcher lo reviven y lo vuelven a apagar (`stops == 2`); con un segundo actor vivo, otros 10 `Future` no provocan apagado (`stops` sigue en 2) y al detenerlo `stops == 3`. A2 oleadas `ActorModelSpec.scala:383-420`: 3 oleadas de 10000 actores hijos creados y detenidos con un `Future` que mantiene vivo al dispatcher; tras cada oleada `stops == run` (un único apagado por oleada). A3 no doble desregistro `ActorModelSpec.scala:499-506`: `DoubleStop` (dos `context.stop(self)`) -> `registers == 1` y `unregisters == 1`. A4 tras `system.close()` los hilos `default`/`internal` desaparecen en 3 s y el dispatcher sigue ejecutando un `Future` pospuesto — `DispatcherShutdownSpec.scala:32-51`. A5 el mensaje de `IllegalStateException`, el estado RESCHEDULED y el cierre inmediato con scheduler cerrado: *no afirmada por spec* (derivada de `AbstractDispatcher.scala:126-266`); caracterización propuesta con reloj/scheduler controlado.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6, Q-X-4.

## AC-DSP-002 Dispatcher event-based: registerForExecution

- **Contrato observable**: `dispatch` encola y registra con hint de mensaje; `systemDispatch` con hint de sistema; un mailbox se agenda a lo sumo una vez (CAS a Scheduled); los mailboxes de actores distintos se ejecutan en paralelo (un actor lento no bloquea a uno rápido en el mismo dispatcher).
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala:80-172`.
- **Condiciones**: rechazo del executor => 1 reintento; segundo rechazo => `setAsIdle`, publica `Error(e, clase, clase, "registerForExecution was rejected twice!")` y relanza (`:146-156`); `executeTask` análogo con `"executeTask was rejected twice!"` (`:95-107`). El rechazo sólo ocurre con un executor apagado (el `ThreadPoolExecutor` usa `SaneRejectedExecutionHandler`, AC-DSP-009), o con una cola acotada llena en el `AffinityPool` (AC-DSP-013); `ActorCell.sendMessage` captura ese fallo y lo publica como `Error` sin propagarlo al emisor (`actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:136-148`).
- **Orden**: `canBeScheduledForExecution` evaluado antes del CAS (`:140`).
- **Dependencias**: AC-MBX-001.
- **Equivalente Rust**: `Dispatcher::register_for_execution(mailbox, hints) -> Result<bool, Rejected>` sobre el executor elegido (Q-X-4); el rechazo sólo existe si el executor está cerrado o su cola llena.
- **Aserciones**: A1 `tell` funciona — `DispatcherActorSpec.scala:71-76`; A2 `ask`/reply funciona — `DispatcherActorSpec.scala:78-82`; A3 un actor lento (50 ms x 50 mensajes, `Thread.sleep`) no bloquea a un actor rápido (10 mensajes): `fFinished.await()` termina mientras `sFinished.getCount > 0` — `DispatcherActorsSpec.scala:44-69`; A4 un mailbox no corre en paralelo consigo mismo y los 200 mensajes de 10 productores se procesan — `ActorModelSpec.scala:311-347`; A5 reintento/doble rechazo *no afirmada por spec* (`Dispatcher.scala:136-158`): caracterización con un executor de prueba que rechaza 1 o 2 veces.
- **Oráculo**: no aplica.
- **Decisiones**: Q-MBX-6.

## AC-DSP-003 throughput y throughput-deadline-time

- **Contrato observable**: máximo de mensajes por turno y tiempo máximo del turno. CONTRADICCIÓN doc/código en `throughput<=0` (ver Q-DSP-5): el Scaladoc dice "vaciar el mailbox", el código usa `max(throughput,1)`; lo observable es el código.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:277-294`; `AbstractDispatcher.scala:333`; `reference.conf:589-594`.
- **Condiciones**: `max(throughput,1)`; deadline activo sólo si `toMillis>0`; la comprobación es `(nanoTime - deadline) < 0` después de cada mensaje.
- **Orden**: un actor lento no impide que otro obtenga turno cuando throughput es alto/bajo según config.
- **Dependencias**: AC-MBX-002.
- **Equivalente Rust**: `DispatcherSettings { throughput: i32, throughput_deadline: Duration }`.
- **Aserciones**: A1 con `throughput = 101` y un hilo, `slowOne` procesa `hogexecutor` + 100 pings (=101) en un turno antes de que `fastOne` obtenga el hilo; el latch de 100 pings llega a 0 — `DispatcherActorSpec.scala:84-110`; A2 con `throughput = 2` y deadline 100 ms, tras un `hogexecutor` que bloquea más que el deadline el turno de `slowOne` termina y `fastOne` corre antes que el segundo mensaje de `slowOne` (`latch.await(2 s) == true`) — `DispatcherActorSpec.scala:112-141`; A3 `throughput = 17` leído de config — `DispatchersSpec.scala:155-158`. El caso `throughput <= 0` y el deadline `<= 0 ms` (sin deadline): *no afirmada por spec*; caracterización propuesta (Q-DSP-5).
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

## AC-DSP-006 internal-dispatcher, default-blocking-io-dispatcher, ExecutionContext externo

*unidad DSP · visibilidad public/internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Existen tres ids de dispatcher fijos: `pekko.actor.default-dispatcher`, `pekko.actor.internal-dispatcher` (executor fork-join, `parallelism-min/factor/max = 4/1.0/64`, `throughput = 5`) y `pekko.actor.default-blocking-io-dispatcher` (`thread-pool-executor`, `fixed-pool-size = 16`, `throughput = 1`). Los guardianes y actores internos corren en `internal-dispatcher`; el guardián de usuario también, salvo que el sistema se cree con un `guardianProps` propio (entonces corre en `default-dispatcher`, o en el dispatcher de esos Props). `internal-dispatcher` puede redefinirse como alias de otro dispatcher. Un `ExecutionContext` pasado vía `BootstrapSetup.defaultExecutionContext` sustituye el executor de todo dispatcher con `executor = "default-executor"`.

**Fuente.**
- `actor/src/main/resources/reference.conf:371-400,638-676`.
- `actor/src/main/scala/org/apache/pekko/dispatch/Dispatchers.scala:24-56,58-77,129-136` (ids, prerrequisitos, `internalDispatcher` evaluado al construir `Dispatchers`).
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:154-177,281,394-402,1026` (BootstrapSetup, `checkValid` sin la ruta `internal-dispatcher` para permitir alias).
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:541-577` (dispatcher de los guardianes).
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:597-625` (`DefaultExecutorServiceConfigurator`).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:44-178`.

**Precondiciones.** El EC externo debe aceptar `execute(Runnable)`; no se le exige ser un pool propio.

**Comportamiento.**
- `DefaultExecutorServiceConfigurator`: con EC definido publica `Debug("DefaultExecutorServiceConfigurator", clase, "Using passed in ExecutionContext as default executor for this ActorSystem. If you want to use a different executor, please specify one in pekko.actor.default-dispatcher.default-executor.")` y usa un `ExecutorService` cuyo `execute` delega en `ec.execute`; `shutdown()` es no-op, `isShutdown`/`isTerminated`/`awaitTermination` devuelven false, `shutdownNow` devuelve lista vacía (el dispatcher nunca apaga el EC del usuario). Sin EC usa el `fallback` (`default-executor.fallback = "fork-join-executor"`).
- Un dispatcher con `executor` explícito (p. ej. `fork-join-executor`) NO usa el EC.
- El guardián de usuario: sin `guardianProps` -> `internalDispatcher`; con `guardianProps`, `Dispatchers.DefaultDispatcherId` si `props.deploy.dispatcher == DispatcherSameAsParent`, si no el dispatcher de los Props.
- Alias de `internal-dispatcher`: `pekko.actor.internal-dispatcher = pekko.actor.default-dispatcher` hace que los actores internos y el guardián de usuario usen `default-dispatcher` (y por tanto el EC externo).

**Errores.** Ninguno propio (ver AC-DSP-004 para ids inexistentes/bucles).

**Orden / concurrencia.** `internalDispatcher` es un `val` evaluado al construir `Dispatchers`; no hay carrera.

**Dependencias.** AC-CFG-015, AC-DSP-004, AC-DSP-010, AC-SYS (BootstrapSetup).

**Equivalente Rust propuesto.** `ActorSystemSetup::with_default_executor(Arc<dyn Executor>)` (o `BootstrapSetup::default_execution_context`) consumido por el configurador `default-executor`; tres ids fijos con la misma configuración por defecto; alias permitido para `internal-dispatcher`. Se preserva: un EC externo sólo reemplaza a `default-executor`, nunca se apaga desde el dispatcher, y se registra el `Debug`. Diferencia: nombres de hilo y dimensionado dependen del executor aportado.

**Aserciones.**
- A1 `ActorSystemDispatcherSpec.scala:53-75`: con un EC "snitching" pasado como `defaultExecutionContext`, un actor de usuario recibe "ping" y responde "pong" (<=1 s) y el EC recibe `execute` ("called").
- A2 `ActorSystemDispatcherSpec.scala:77-98`: con `default-dispatcher.executor = "fork-join-executor"` el EC no se usa (`ecProbe.expectNoMessage()`) y el eco funciona.
- A3 `ActorSystemDispatcherSpec.scala:105-119`: con `internal-dispatcher = default-dispatcher` el guardián de usuario corre en `pekko.actor.default-dispatcher`.
- A4 `ActorSystemDispatcherSpec.scala:121-150`: con ese alias y un EC pasado, un actor creado con `withDispatcher(InternalDispatcherId)` usa el EC ("called", "pong").
- A5 `ActorSystemDispatcherSpec.scala:152-154`: por defecto el guardián de usuario corre en `pekko.actor.internal-dispatcher`.
- A6 `ActorSystemDispatcherSpec.scala:156-168`: con `guardianProps = Some(Props.empty)` el guardián corre en `pekko.actor.default-dispatcher`.
- A7 bucle de alias -> `ConfigurationException`: `ActorSystemDispatcherSpec.scala:170-174` (AC-DSP-004).
- A8 valores por defecto de `internal-dispatcher` y `default-blocking-io-dispatcher`, el mensaje `Debug` y que `shutdown` del wrapper sea no-op: *no afirmada por spec*. Caracterización propuesta: leer la configuración por defecto y comparar con `reference.conf:607-629`; un executor de prueba que cuente `execute` y compruebe que `dispatcher.shutdown()` no lo detiene.

**Validación prevista.** A1-A7 portados con un `SnitchingExecutor`; A8 caracterización de configuración. Oráculo: no aplica.

**Decisiones.** Q-X-4 (runtime), Q-DSP-1 (claves de executor), Q-DSP-2 (alias/registro).

## AC-DSP-007 Selección del dispatcher de un actor

- **Contrato observable**: ver TSV.
- **Fuente**: `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:691-764`.
- **Condiciones**: textos en TSV.
- **Orden**: verificación en la creación, antes de arrancar.
- **Dependencias**: AC-CFG-012, AC-DSP-004, AC-MBX-013.
- **Equivalente Rust**: resolución en `ActorRefProvider::actor_of`.
- **Aserciones**: A1 dispatcher de deployment — `DispatchersSpec.scala:266-268`; A2 deployment pisa Props — `:270-273`; A3 pool-dispatcher del router (nombre de hilo `DispatchersSpec-pekko.actor.deployment./pool1.pool-dispatcher-N`) — `:275-284`; A4 `balancing-pool` con `pool-dispatcher.mailbox = myapp.mymailbox` (`OneShotMailboxType`, que sólo puede crearse una vez): una sola petición produce DOS respuestas con hilo `DispatchersSpec-BalancingPool-/balanced-N`, porque el `DoublingMailbox` duplica cada encolado: prueba que la cola compartida del BalancingPool se crea una vez a partir del mailbox configurado — `:286-295`; A5 dispatcher inexistente => ConfigurationException (actor, cabeza del router, routees) — `ActorConfigurationVerificationSpec.scala:95-110`; A6 router (RoundRobin/Broadcast/Random/SmallestMailbox/ScatterGather) con BalancingDispatcher => ConfigurationException y el actor sin router con el mismo dispatcher NO falla — `:58-88`; A7 pool con dispatcher no balanceador y router no falla — `:91-93`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-2.

## AC-DSP-008 ForkJoinExecutorConfigurator

- **Contrato observable**: ver TSV (contrato corregido). Efecto observable del `ForkJoinPool` de Pekko (modo `asyncMode` = cola FIFO por hilo cuando `task-peeking-mode = "FIFO"`, pila LIFO si `"LIFO"`; robo de trabajo entre hilos; tamaño `parallelism`; compensación de hilos bloqueados vía `blocking {}`/`Await` con `PekkoForkJoinWorkerThread` como `BlockContext` -> `ForkJoinPool.managedBlock`).
- **Fuente**: `actor/src/main/scala/org/apache/pekko/dispatch/ForkJoinExecutorConfigurator.scala:39-45,119-189`; `ThreadPoolBuilder.scala:49-50,216-240`; `reference.conf:469-520`.
- **Condiciones**: errores en TSV. `virtualize = on` sólo tiene efecto con JDK >= 21 (`isVirtualized = virtualize && JavaVersion.majorVersion >= 21`, `:105`): en JDK < 21 se ignora SIN error, a pesar de que `reference.conf:509-510` dice que lanzaría una excepción (contradicción registrada).
- **Orden**: n/a.
- **Dependencias**: AC-DSP-004.
- **Equivalente Rust**: pool work-stealing propio o runtime; claves aceptadas y validadas (Q-DSP-1). Ver tabla de executors en la revisión de exhaustividad.
- **Aserciones**: A1 `resolveMinimumRunnable(0,...)==0` en JDK 17 y 25 — `ForkJoinExecutorConfiguratorSpec.scala:76-79`; A2 valores positivos verbatim — `:81-85`; A3 -1 en JDK<25 => 1 — `:87-94`; A4 -1 en JDK 25: parallelism/2 acotado a [1,8] (1→1,2→1,4→2,8→4,16→8,64→8) — `:96-103`; A5 estrictamente mayor en JDK 25 que en 21 — `:105-115`; A6 nunca >8 — `:117-121`; A7 integración por dispatcher: explícito 0 y 7 respetados (`:145-151`), nunca <1 (`:170-175`), default no tocado = 1 en JDK<25 (`:164-168`), auto-escala en JDK 25+ (`:153-162`, `pending` en JDK < 25).
- **Oráculo**: A1-A5 son funciones puras: comparables con el jar oficial (`ForkJoinExecutorConfigurator.resolveMinimumRunnable`, `private[pekko]` accesible desde `jshell`).
- **Decisiones**: Q-DSP-1, Q-DSP-6 (el valor `-1` y el default efectivo).

## AC-DSP-009 ThreadPoolExecutorConfigurator y ThreadPoolConfig

- **Contrato observable / Fuente / Condiciones**: ver TSV (tamaños, colas y errores). Corrección/omisión: la política de rechazo por defecto es `SaneRejectedExecutionHandler` (`actor/src/main/scala/org/apache/pekko/dispatch/ThreadPoolBuilder.scala:49,312-321`): cuando el pool está saturado (cola acotada llena y `maxPoolSize` alcanzado) el `Runnable` se ejecuta en el hilo del llamante ("caller runs"), es decir, el emisor de `tell` ejecuta el turno del mailbox del receptor; si el executor está apagado lanza `RejectedExecutionException("Shutdown")`. Los valores de `ThreadPoolConfig()` sin configurar (usados por el fallback de `PinnedDispatcherConfigurator`) son `allowCorePoolTimeout=false`, `corePoolSize=16`, `maxPoolSize=128`, `threadTimeout=60 s`, cola enlazada sin límite (`:40-45,86-92`). `atFullThrottle() = activeCount >= poolSize`.
- **Orden**: n/a.
- **Dependencias**: AC-DSP-004.
- **Equivalente Rust**: pool de hilos con tamaño core/max, cola (array/linked, acotada o no), keep-alive y política "caller runs salvo apagado" (Q-DSP-1). El efecto de contrapresión (el emisor ejecuta el trabajo) es observable y se conserva; el nombre de hilo `<sistema>-<id>-N` también (AC-DSP-015).
- **Aserciones**: A1 nombre de hilo con `thread-pool-executor` — `DispatchersSpec.scala:234-240`; `fixed-pool-size = 1` se usa en `DispatcherActorSpec.scala:34-37,43-46` (un solo hilo); el cálculo de tamaños, los errores de `task-queue-type` y la política de rechazo *no afirmados por spec* (derivados de `ThreadPoolBuilder.scala:49-50,164-173` y `AbstractDispatcher.scala:467-507`); caracterización propuesta: `scaledPoolSize` (función pura, oráculo diferencial con `jshell`), cola acotada llena -> el emisor ejecuta la tarea, apagado -> `Shutdown`.
- **Oráculo**: `scaledPoolSize` comparable (función pura).
- **Decisiones**: Q-DSP-1.

## AC-DSP-010 ExecutorServiceConfigurator SPI, default-executor y fallback, executor por FQCN

*unidad DSP · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** La clave `executor` de un dispatcher elige la fábrica de `ExecutorService`: `"default-executor"` (usa el EC externo si existe, si no `default-executor.fallback`), `"fork-join-executor"` (también para `null` o cadena vacía), `"virtual-thread-executor"` (si el runtime no soporta hilos virtuales, la fábrica de `virtual-thread-executor.fallback`), `"thread-pool-executor"`, `"affinity-pool-executor"`, o el FQCN de un `ExecutorServiceConfigurator` con constructor `(Config, DispatcherPrerequisites)`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:392-450` (`ExecutorServiceConfigurator`, `MessageDispatcherConfigurator.configureExecutor`), `:513-541` (`DefaultExecutorServiceConfigurator`).
- `actor/src/main/resources/reference.conf:378-400,600-616`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemDispatcherSpec.scala:53-98`; `actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatchersSpec.scala:237-271`.

**Precondiciones.** La config del dispatcher incluye (por herencia de `default-dispatcher`) las secciones `fork-join-executor`, `thread-pool-executor`, `affinity-pool-executor`, `virtual-thread-executor` y `default-executor`.

**Comportamiento.** `configureExecutor` (recursión de cola `configurator(executor)`): `null | "" | "fork-join-executor"` -> `ForkJoinExecutorConfigurator(config.getConfig("fork-join-executor"))`; `"virtual-thread-executor"` -> `VirtualThreadExecutorConfigurator` si `VirtualThreadSupport.isSupported` (JDK >= 21) o, si no, `configurator(config.getString("virtual-thread-executor.fallback"))`; `"thread-pool-executor"` -> `ThreadPoolExecutorConfigurator`; `"affinity-pool-executor"` -> `AffinityPoolConfigurator`; cualquier otro valor se trata como FQCN (se instancia con `(config del dispatcher COMPLETO, prerequisites)`, no con la sección). `config.getString("executor")`: `"default-executor"` -> `DefaultExecutorServiceConfigurator(config.getConfig("default-executor"), prerequisites, configurator(default-executor.fallback))`, otro -> `configurator(other)`.

**Errores.** FQCN no instanciable -> `IllegalArgumentException("Cannot instantiate ExecutorServiceConfigurator (\"executor = [<fqcn>]\"), defined in [<id>],\n                make sure it has an accessible constructor with a [interface com.typesafe.config.Config,interface org.apache.pekko.dispatch.DispatcherPrerequisites] signature", causa)` (`:386-398`; el texto contiene un salto de línea y 16 espacios, y los `Class.toString` de la JVM). Un `fallback = "default-executor"` degenera en el caso FQCN (error anterior). Sección faltante -> `ConfigException.Missing` de la librería de config (sin envolver).

**Orden / concurrencia.** El executor se crea de forma perezosa al primer uso (`LazyExecutorServiceDelegate`), no al construir el dispatcher (`Dispatcher.scala:53-66`).

**Dependencias.** AC-CFG-016, AC-DSP-006, AC-DSP-008, AC-DSP-009, AC-DSP-013, AC-DSP-014.

**Equivalente Rust propuesto.** `trait ExecutorFactory { fn create(&self, id: &str, thread_factory: &ThreadFactory) -> Arc<dyn Executor>; fn is_virtualized(&self) -> bool }` y un registro nombre -> constructor (Q-DSP-2/Q-X-2) con los cinco nombres de Pekko; los FQCN de Pekko pueden registrarse como alias. El fallback de `virtual-thread-executor` se activa si el runtime no ofrece tareas ligeras (en Rust siempre se ofrece, ver AC-DSP-014).

**Aserciones.**
- A1 `ActorSystemDispatcherSpec.scala:53-98`: `default-executor` con EC pasado lo usa y, con `executor = "fork-join-executor"` explícito, no (AC-DSP-006 A1/A2).
- A2 `DispatchersSpec.scala:230-264`: el nombre de hilo refleja el tipo de executor elegido (fork-join, thread-pool, default, pinned, balancing).
- A3 resolución del `fallback` de `virtual-thread-executor`, de `null`/vacío y el error del FQCN: *no afirmada por spec*. Caracterización propuesta: `executor = ""` equivale a fork-join; FQCN/nombre desconocido -> error con el id del dispatcher y el nombre; `virtual-thread-executor` con soporte deshabilitado -> fallback.

**Validación prevista.** A1/A2 portados; A3 caracterización de configuración.

**Decisiones.** Q-DSP-1, Q-DSP-2 (registro por nombre).

## AC-DSP-011 PinnedDispatcher
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: un hilo; throughput infinito.
- **Dependencias**: AC-DSP-009.
- **Equivalente Rust**: `std::thread` dedicado + bucle del mailbox.
- **Aserciones**: A1 tell — `PinnedActorSpec.scala:49-56`; A2 ask — `:58-62`; A3 nombre de hilo contiene id — `DispatchersSpec.scala:250-256`; el error "Cannot register to anyone but" *no afirmado por spec*.
- **Oráculo**: no aplica.
- **Decisiones**: Q-DSP-1.

## AC-DSP-012 BalancingDispatcher

*unidad DSP · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Dispatcher de cola compartida ("work donating", deprecado, `private[pekko]`, base de `BalancingPool`): todos los actores del dispatcher usan UNA `messageQueue` común (`mailboxType.create(None, None)`) mediante un `SharingMailbox` por actor (con su propia cola de sistema); cualquier miembro libre puede procesar cualquier mensaje. Un mensaje enviado a un actor entra en la cola compartida y, si ese actor no pudo registrarse para ejecución (está ocupado), se intenta despertar a otros miembros ociosos (teamwork).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/BalancingDispatcher.scala:45-125`; `Dispatchers.scala:314-374` (configurador; ver AC-DSP-021).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/BalancingDispatcherSpec.scala:31-103`; `ActorModelSpec.scala:588-663`; `ActorConfigurationVerificationSpec.scala:57-88`; `DispatchersSpec.scala:258-264,286-295`.

**Precondiciones.** El `mailbox-requirement` del dispatcher implementa `MultipleConsumerSemantics` (AC-DSP-021). `attempt-teamwork` (por defecto `on`) activa el reparto activo.

**Comportamiento.**
- `createMailbox` ignora el `MailboxType` del actor y devuelve `SharingMailbox(system, messageQueue)` (`:93-94`).
- `register`: `super.register` + `team.add(actor)`; `unregister`: `team.remove` + `super.unregister` + `teamWork()` (`:96-105`). `team` es un `ConcurrentSkipListSet[ActorCell]` ordenado por `self.path` (con desempate por identidad).
- `dispatch(receiver, envelope)`: `messageQueue.enqueue(receiver.self, envelope)` y `if (!registerForExecution(receiver.mailbox, false, false)) teamWork()` (`:107-110`).
- `teamWork()` (si `attemptTeamWork`): recorre `team` en orden mientras `messageQueue.hasMessages` y quedan miembros y el executor no está al límite (si es `LoadMetrics`: `!atFullThrottle()`; si no lo es, true) y `!registerForExecution(i.next.mailbox, false, false)`; es decir, se detiene en el primer miembro cuyo mailbox se registró (`:112-125`).
- `SharingMailbox.cleanUp` sólo drena mensajes de sistema a deadLetterMailbox; la cola compartida NO se limpia (`:79-90`).
- `systemDispatch` es el de `Dispatcher` (cola de sistema propia de cada mailbox).
- Efecto observable: un mensaje enviado al actor A puede ser procesado por B; `hasMessages` es idéntico para todos los miembros; no hay orden por actor.

**Errores.** Ver AC-DSP-021 (validación del configurador) y AC-DSP-007 (router + BalancingDispatcher -> `ConfigurationException`).

**Orden / concurrencia.** Los mensajes salen de la cola compartida en el orden de ésta (FIFO por defecto); la asignación a actores es no determinista. El rechazo del executor en `registerForExecution` se maneja como en AC-DSP-002.

**Dependencias.** AC-DSP-021, AC-DSP-002, AC-MBX-003, AC-RT-012 (BalancingPool).

**Equivalente Rust propuesto.** `BalancingDispatcher` interno para `BalancingPool`: cola MPMC compartida + `SharingMailbox` + `team` ordenado por path + `team_work()`; `attempt_teamwork` y un indicador `at_full_throttle()` del executor (Q-DSP-1). Se conserva el efecto de reparto; no se exige igualdad de planificación de hilos.

**Aserciones.**
- A1 `BalancingDispatcherSpec.scala:63-102`: dispatcher `pooled-dispatcher` (`BalancingDispatcherConfigurator`, `throughput = 1`); actor `slow` (50 ms por mensaje) y `fast` (10 ms) reciben 100 mensajes (5 al `fast`, 95 al `slow`) y 10 más espaciados 150 ms; tras `finishedCounter.await(5 s)` (cuyo valor de retorno no se afirma) se afirma `hasMessages == false` en ambos mailboxes, `fast.invocationCount > sentToFast` (procesó trabajo ajeno) y `fast.invocationCount > slow.invocationCount`.
- A2 `ActorModelSpec.scala:588-663` (`BalancingDispatcherModelSpec`, `throughput = 1`): todas las pruebas genéricas del modelo (ciclo de vida, un mensaje a la vez, suspensión, oleadas, excepciones...) más "process messages in parallel" (un actor bloqueado en `Meet` no impide que otro procese, `registers = unregisters = 1`).
- A3 `DispatchersSpec.scala:258-264`: nombre de hilo `DispatchersSpec-myapp.balancing-dispatcher-N`; `:286-295` ver AC-DSP-007 A4.
- A4 `ActorConfigurationVerificationSpec.scala:58-85`: pools no balanceadores con BalancingDispatcher -> `ConfigurationException`.
- A5 `attempt-teamwork = off`, orden de `team`, parada en el primer miembro registrado y no limpieza de la cola compartida: *no afirmada por spec*. Caracterización propuesta: con executor de 1 hilo ocupado y teamwork on/off, comprobar quién procesa el mensaje; comprobar que tras `unregister` de un miembro sus mensajes siguen en la cola compartida.

**Validación prevista.** A1-A4 portados (A1 con retardos reales o ejecutor controlado; el assert es estadístico pero con holgura grande); A5 caracterización.

**Decisiones.** Q-MBX-6 (modelo de ejecución), Q-DSP-1 (`atFullThrottle` por executor).

## AC-DSP-013 AffinityPool

*unidad DSP · visibilidad internal-required (ApiMayChange) · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Executor `affinity-pool-executor`: un pool de `parallelism` hilos con una cola acotada MPSC por hilo; cada tarea se encola en la cola que elige un `QueueSelector` a partir del `hashCode` de la tarea (el `Mailbox`), de modo que el mismo mailbox corre siempre en el mismo hilo (afinidad). No hay robo de trabajo. Cola llena -> el `RejectionHandler` (por defecto lanza `RejectedExecutionException`). Los hilos ociosos aplican una estrategia de espera escalonada controlada por `idle-cpu-level`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/affinity/AffinityPool.scala:35-111` (estados y `IdleStrategy`), `:113-303` (pool), `:305-365` (configurador), `:367-444` (selector y rechazo).
- `actor/src/main/resources/reference.conf:402-472`; `actor/src/main/java/org/apache/pekko/dispatch/AbstractBoundedNodeQueue.java` (cola por hilo); `actor/src/main/java/org/apache/pekko/dispatch/affinity/OnSpinWait.java`.
- Sin tests en `actor-tests`/`testkit` (búsqueda de `AffinityPool|affinity-pool`: sólo `docs/src/test/.../DispatcherDocSpec.scala` y `bench-jmh`).

**Precondiciones / configuración.** `parallelism-min = 4`, `parallelism-factor = 0.8`, `parallelism-max = 64` (tamaño = `scaledPoolSize`: `min(max(ceil(cpus * factor), min), max)`); `task-queue-size = 512`; `rejection-handler = ...ThrowOnOverflowRejectionHandler`; `idle-cpu-level = 5` (1..10); `queue-selector = ...FairDistributionHashCache`; `fair-work-distribution.threshold = 128` (0..2048).

**Comportamiento.**
- Estados del pool: Uninitialized(0) -> Initializing(1) -> Running(2) -> ShuttingDown(3) -> ShutDown(4) -> Terminated(5). `start()` crea un worker por cola.
- `execute(cmd)`: `queue = workQueues(queueSelector.getQueue(cmd, parallelism))` (NPE si `cmd` es null); si `poolState >= ShuttingDown` o `!queue.add(cmd)` -> `rejectionHandler.reject(cmd, this)`; el manejador por defecto lanza `RejectedExecutionException("Task <cmd> rejected from <pool>")`. Para el dispatcher, `registerForExecution` reintenta una vez y luego publica `Error(..., "registerForExecution was rejected twice!")` y relanza (AC-DSP-002); `ActorCell.sendMessage` captura ese fallo y lo publica como `Error(..., "swallowing exception during message send")` (`actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:136-148`), de modo que el emisor NO recibe la excepción y el mensaje queda en el mailbox sin planificar hasta el siguiente envío.
- Worker: `poll` de su cola; si hay tarea la ejecuta y reinicia la estrategia de espera; si no, `idle()`. Estados de espera: Initial -> Spinning (`maxSpins = 1100*level - 1000` vueltas con `OnSpinWait`) -> Yielding (`maxYields = 5*level` `Thread.yield`) -> Parking (`parkNanos` desde 1 ns, duplicando hasta `maxPark = (250 - (80*(level-1))/3)` microsegundos, división entera).
- `shutdown()`: ShuttingDown, interrumpe sólo workers ociosos; los demás vacían su cola; `shutdownNow()`: ShutDown, interrumpe todos y descarta lo encolado (devuelve lista vacía). Un worker que termina por excepción de una tarea se sustituye por otro con la misma cola si el pool está Running.
- `FairDistributionHashCache`: con `threshold == 0`, `improve(hash) % queues`; si no, mapa inmutable hash -> índice: un hash visto reutiliza su índice; si el mapa ya tiene `> threshold` entradas, `improve(hash) % queues`; si no se le asigna `size % queues` (reparto round-robin por orden de llegada) y se guarda por CAS. `improve(h) = 0x7FFFFFFF & (reverseBytes(h * 0x9E3775CD) * 0x9E3775CD)` (aritmética `Int`).
- Nombres de hilo: como los demás executors (`<sistema>-<id>-N`, `AffinityPoolConfigurator.createExecutorServiceFactory`).

**Errores.** `parallelism <= 0` -> `IllegalArgumentException("Size of pool cannot be less or equal to 0")`; `idle-cpu-level` fuera de 1..10 -> `IllegalArgumentException("requirement failed: idle-cpu-level must be between 1 and 10")`; `fair-work-distribution.threshold` fuera de 0..2048 -> `IllegalArgumentException("requirement failed: fair-work-distribution.threshold must be between 0 and 2048")`; selector no instanciable -> `IllegalArgumentException("Cannot instantiate QueueSelectorFactory(queueSelector = <fqcn>), make sure it has an accessible constructor which accepts a Config parameter")`; rejection handler no instanciable -> `IllegalArgumentException("Cannot instantiate RejectionHandlerFactory(rejection-handler = <fqcn>), make sure it has an accessible empty constructor", causa)`; worker sin hilo -> `IllegalStateException("Was not able to allocate worker thread for <pool>")`.

**Orden / concurrencia.** FIFO por cola (por worker); las tareas de un mismo mailbox siempre pasan por la misma cola (si el `hashCode` es estable). Equidad: sin robo, un worker ocupado retrasa los demás mailboxes de su cola.

**Dependencias.** AC-DSP-010, AC-DSP-002, AC-DSP-015, AC-MBX-008 (misma cola acotada), AC-UTIL-002.3 (`requiring`).

**Equivalente Rust propuesto.** Pool de `std::thread` con una cola acotada por worker (`crossbeam::queue::ArrayQueue` o equivalente MPSC), trait `QueueSelector` con `FairDistributionHashCache` portado, `RejectionHandler` (`ThrowOnOverflow` = `Err(Rejected)`), estrategia de espera con `spin_loop`/`yield_now`/`park_timeout`. El selector usa un identificador estable por mailbox en vez de `identityHashCode` (la fase round-robin es reproducible; la fase por hash sólo preserva la distribución, no los valores). Viabilidad alta con hilos propios (Q-X-4 A); con un runtime async no tiene sentido (se documentaría como alias a un pool con colas por worker).

**Aserciones.** *No afirmada por spec* (ninguna prueba en upstream). Caracterización propuesta (de rakka): (a) selector round-robin hasta `threshold`, luego hash; (b) cola llena -> rechazo; (c) afinidad: el mismo mailbox corre siempre en el mismo hilo; (d) `shutdown` drena colas, `shutdownNow` las descarta; (e) validaciones con los textos literales. Oráculo diferencial viable para el selector: `jshell` con el jar oficial y `Runnable`s de `hashCode` controlado (`FairDistributionHashCache` es `private[pekko]` pero público en bytecode).

**Validación prevista.** Caracterización + oráculo diferencial del selector (función pura dada la secuencia de hashes).

**Decisiones.** Q-DSP-1, Q-X-4.

## AC-DSP-014 VirtualThreadExecutorConfigurator y virtualize

*unidad DSP · visibilidad public (JDK 21+) · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `executor = "virtual-thread-executor"` ejecuta cada tarea (cada turno de mailbox) en un hilo virtual NUEVO (`newThreadPerTaskExecutor`), con nombre `<sistema>-<idDispatcher>-virtual-thread-<n>` (`n` desde 0); si el runtime no soporta hilos virtuales se usa el `fallback`. `fork-join-executor.virtualize = on` (JDK >= 21) hace que el `ForkJoinPool` sirva de planificador (carrier) de hilos virtuales con nombre `<sistema>-<idDispatcher>-virtual-thread-<n>`. En ambos casos el dispatcher no agrupa tareas (`batchable = false`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:452-505` (configurador), `VirtualThreadSupport.scala:29-131` (reflexión sobre `Thread.ofVirtual`, `Executors.newThreadPerTaskExecutor`, planificador por defecto), `VirtualizedExecutorService.scala:25-117`, `ForkJoinExecutorConfigurator.scala:105,154-196`, `Dispatcher.scala:78-81`; `reference.conf:509-519,573-585`.
- Pruebas (sólo JDK 21): `actor-tests/src/test/scala-jdk21-only/org/apache/pekko/dispatch/VirtualThreadPoolDispatcherSpec.scala:48-56`, `ForkJoinPoolVirtualThreadSpec.scala:58-66`.

**Precondiciones.** `VirtualThreadSupport.isSupported = JavaVersion.majorVersion >= 21`; para hilos virtuales con `virtualize` hacen falta `--add-opens` (reflexión sobre `jdk.internal.misc`).

**Comportamiento.** `VirtualizedExecutorService` envuelve el ejecutor por-tarea: `execute`/`submit` crean un hilo virtual; `shutdown()` cierra el ejecutor y, con `cascadeShutdown`, también el pool subyacente (true para fork-join+virtualize, false para virtual-thread-executor, que no debe apagar el planificador global); `atFullThrottle()` = `pool.getActiveThreadCount >= pool.getParallelism` si se obtuvo el planificador por defecto, y `true` constante si no. Si no se pudo obtener el planificador por defecto se publica `Warning(e, "VirtualThreadExecutorConfigurator", clase, "\nFailed to get the default scheduler of virtual thread, so the `LoadMetrics` is not available when using it with `BalancingDispatcher`.\nAdd `--add-opens java.base/java.lang=ALL-UNNAMED` to the JVM options to help this.\n")`. El hilo virtual hereda `UncaughtExceptionHandler` y `contextClassLoader` de la `MonitorableThreadFactory`.

**Errores.** `UnsupportedOperationException("Failed to create virtual thread factory", causa)` / `("Failed to create newThreadPerTaskExecutor.", causa)` si la reflexión falla; `IllegalArgumentException` de `require(isSupported, "Virtual thread is not supported.")` en `newVirtualThreadFactory`. `virtualize = on` en JDK < 21 se ignora sin error (AC-DSP-008).

**Orden / concurrencia.** Sin cola ni robo propio: cada tarea es un hilo; el límite lo impone el planificador de JDK (`jdk.virtualThreadScheduler.*`).

**Dependencias.** AC-DSP-010, AC-DSP-008, AC-DSP-015 (nombres), AC-DSP-012 (LoadMetrics).

**Equivalente Rust propuesto.** Tareas async (`tokio::spawn`) por turno de mailbox o por tarea: es el equivalente natural. Se preserva la intención observable (bloquear/esperar dentro de un actor no consume un worker del pool y no hay límite fijo de concurrencia). No se preserva: nombres de hilo (`...-virtual-thread-N`; en Rust las tareas no tienen hilo propio, se pueden nombrar con `tracing`/`tokio::task::Builder::name`), `atFullThrottle` real (se aproxima con tareas activas vs workers del runtime) ni la reflexión/`add-opens`. Como en Rust la capacidad siempre existe, el `fallback` no se activa nunca.

**Aserciones.**
- A1 `VirtualThreadPoolDispatcherSpec.scala:48-56`: con `executor = virtual-thread-executor`, 1000 pings a un actor devuelven un nombre de hilo que incluye `VirtualThreadPoolDispatcherSpec-virtual-thread-dispatcher-virtual-thread-`.
- A2 `ForkJoinPoolVirtualThreadSpec.scala:58-66`: con `fork-join-executor.virtualize = on` (paralelismo 2, mailbox `SingleConsumerOnlyUnboundedMailbox`, throughput 5) 1000 pings devuelven un nombre que incluye `ForkJoinPoolVirtualThreadSpec-custom.task-dispatcher-virtual-thread-`.
- A3 `ForkJoinPoolStarvationSpec.scala:64-82` es `pending` desde JDK 17 (issue #31117): dos actores que se envían "tick" a sí mismos no deben matar de hambre a un tercero con 2 hilos; en Rust se retoma como prueba de equidad (AC-DSP-003).
- A4 el `Warning`, `cascadeShutdown` y `atFullThrottle`: *no afirmada por spec*. Caracterización: equivalente `atFullThrottle` documentado.

**Validación prevista.** A1/A2 se reformulan sin nombre de hilo (comprobar que cada turno corre en una tarea ligera distinta y que el nombre de tarea contiene el id); A3 como prueba de equidad con executor de 2 workers. Oráculo: no aplica.

**Decisiones.** Q-DSP-1.

## AC-DSP-015 MonitorableThreadFactory
- **Contrato observable / Fuente**: ver TSV; `ThreadPoolBuilder.scala:191-242`.
- **Condiciones**: nombre `<base>-<contador>`; el contador es por fábrica; base = `<sistema>-<id>` (`ForkJoinExecutorConfigurator.scala:201-205`, `ThreadPoolBuilder.scala:125-127`).
- **Orden/Dependencias**: AC-CFG-007.
- **Equivalente Rust**: `std::thread::Builder::name`.
- **Aserciones**: A1-A5 nombres de hilo con regex por tipo de dispatcher — `DispatchersSpec.scala:230-264`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-DSP-016 Dispatcher como ExecutionContextExecutor (descompuesta en AC-DSP-016.1 y AC-DSP-016.2)

Fila descompuesta: `Dispatcher.execute/TaskInvocation/reportFailure` es AC-DSP-016.1 y el agrupador `BatchingExecutor` es AC-DSP-016.2. Una referencia a AC-DSP-016 significa ambos hijos. Fuente conjunta: `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:55-67,206-225` y `BatchingExecutor.scala:23-147`.

## AC-DSP-016.1 Dispatcher.execute, TaskInvocation e inhabitants, reportFailure

*unidad DSP · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Un `MessageDispatcher` es un `ExecutionContextExecutor`: `execute(Runnable)` ejecuta la tarea en el executor del dispatcher, la cuenta como inhabitant mientras está pendiente o en ejecución (mantiene vivo al dispatcher), y publica en el EventStream cualquier excepción no fatal de la tarea; `reportFailure(t)` publica el fallo como evento.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:55-64` (`TaskInvocation`), `:172-189` (`unbatchedExecute`, `reportFailure`), `:223` (`taskCleanup`), `Dispatcher.scala:92-108` (`executeTask`).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:284-310,384-421`.

**Precondiciones.** El dispatcher está activo (si estaba apagado, `executorService` recrea el executor perezosamente).

**Comportamiento.**
- `unbatchedExecute(r)`: `TaskInvocation(eventStream, r, taskCleanup)`; `addInhabitants(+1)`; `executeTask(invocation)`; si lanza cualquier `Throwable`: `addInhabitants(-1)` y relanza.
- `executeTask`: `executorService.execute(invocation)`; ante `RejectedExecutionException` reintenta una vez; si vuelve a fallar publica `Error(e, getClass.getName, getClass, "executeTask was rejected twice!")` y lanza la segunda excepción.
- `TaskInvocation.run()`: `runnable.run()`; un `NonFatal(e)` se publica como `Error(e, "TaskInvocation", this.getClass, e.getMessage)`; en `finally` `cleanup()` (`taskCleanup`: `addInhabitants(-1) == 0` -> `ifSensibleToDoSoThenScheduleShutdown`).
- `reportFailure(t)`: `LogEventException` -> publica `e.event`; otro `Throwable` -> `Error(t, getClass.getName, getClass, t.getMessage)`.
- `batchable(r)`: false si el dispatcher está virtualizado (AC-DSP-014), si no ver AC-DSP-016.2.

**Errores.** Los de arriba; los `Fatal` no se capturan (`NonFatal`).

**Orden / concurrencia.** Seguro desde cualquier hilo; el inhabitant se libera siempre en `finally`.

**Dependencias.** AC-DSP-001, AC-DSP-002 (mismo patrón de reintento), AC-EVT-011 (Error en EventStream), AC-DSP-016.2.

**Equivalente Rust propuesto.** `Dispatcher::execute(Box<dyn FnOnce() + Send>) -> Result<(), Rejected>`; el cierre captura `panic` con `catch_unwind` y publica `Error` (Q-X-5); `report_failure(&dyn Error)`. Se conserva: conteo de inhabitants, reintento único y mensajes.

**Aserciones.**
- A1 `ActorModelSpec.scala:283-309`: 10 `Future { i }` ejecutados en el dispatcher interceptado vuelven a provocar su apagado (`stops == 2`) y con un actor vivo no lo provocan (`stops` sigue en 2): las tareas cuentan como inhabitants.
- A2 `ActorModelSpec.scala:383-420`: un `Future { keepAliveLatch.await }(dispatcher)` mantiene vivo al dispatcher durante una oleada de 10000 actores (`stops == run` al final de cada oleada).
- A3 publicación `Error(e, "TaskInvocation", ...)`, reversión del inhabitant si `executeTask` falla, doble rechazo y `reportFailure`: *no afirmada por spec*. Caracterización propuesta: tarea que entra en pánico -> evento `Error` con origen "TaskInvocation" y el dispatcher sigue operativo; executor que rechaza dos veces -> `Error` "executeTask was rejected twice!" y `inhabitants` vuelve a su valor.

**Validación prevista.** A1/A2 portados (con una tarea equivalente a `Future`); A3 caracterización.

**Decisiones.** Q-DSP-3 (alcance de los helpers), Q-X-5.

## AC-DSP-016.2 BatchingExecutor (agrupación de tareas anidadas y reenvío ante bloqueo)

*unidad DSP · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `BatchingExecutor` agrupa las tareas "batchable" enviadas desde dentro de otra tarea del mismo hilo en un lote que se ejecuta en serie en ese hilo, sin pasar por la cola del executor y sin ejecutarlas en línea (inline) dentro de quien las envía. Si una tarea del lote bloquea (`blocking {}`/`Await`), el resto del lote se reenvía al executor para evitar interbloqueos. `MessageDispatcher` lo usa con `resubmitOnBlock = true`; el contexto "mismo hilo" (`sameThreadExecutionContext`) con `false`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/BatchingExecutor.scala:24-152`; `actor/src/main/scala/org/apache/pekko/dispatch/internal/ScalaBatchable.scala:32-36` (y las variantes `scala-2.12`, `scala-3`); `AbstractDispatcher.scala:109,173-174`; `Dispatcher.scala:78-81`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:71-256`.

**Precondiciones.** El `unbatchedExecute` subyacente NO debe ejecutar el `Runnable` de forma síncrona en el llamante (Scaladoc `:60-61`).

**Comportamiento.** `execute(r)`: si `batchable(r)` (por defecto `ScalaBatchable.isBatchable`: `pekko.dispatch.Batchable.isBatchable` o `scala.concurrent.Batchable`, es decir, los callbacks de `Future`) -> con `_tasksLocal` nulo se crea un lote (`Batch` o, si `resubmitOnBlock`, `BlockableBatch`), se le añade la tarea y se envía el lote con `unbatchedExecute`; con `_tasksLocal` presente se añade al lote actual. Si no es batchable -> `unbatchedExecute(r)` directo. `Batch.run` instala el lote en el hilo, procesa con `pollFirst` (FIFO) hasta vaciar (o hasta que `_tasksLocal` cambie), y limpia. Ante una excepción: `resubmitUnbatched()` (el resto del lote se reenvía como una sola tarea) y se relanza. `BlockableBatch.blockOn(thunk)`: `resubmitUnbatched()` y delega el bloqueo en el `BlockContext` anterior.

**Errores.** `require(_tasksLocal.get eq null)` al arrancar un lote (violación de invariante). El reenvío tras una excepción no gestiona el fallo del reenvío (`TODO` en `:80`).

**Orden / concurrencia.** Las tareas de un lote se ejecutan en serie en un mismo hilo, nunca en paralelo ni dentro del `execute` de quien las envía. El orden entre lotes no está garantizado (el Scaladoc dice que cualquier orden, incluido LIFO, es posible).

**Dependencias.** AC-DSP-016.1.

**Equivalente Rust propuesto.** Optimización opcional: lote en `thread_local!` por executor; no hay `BlockContext` en Rust, de modo que el reenvío ante bloqueo no puede ser automático (Q-DSP-7). Se conserva: no-inline, ejecución en serie del lote, tareas no batchables directas.

**Aserciones.**
- A1 `ExecutionContextSpec.scala:55-83` ("be able to use Batching"): `system.dispatcher` es un `BatchingExecutor`; una tarea batchable lanza 100 tareas batchable anidadas; ninguna se ejecuta dentro de la tarea externa (`callingThreadLock` activo -> "Batch was executed inline!") ni en paralelo ("Executed batch in parallel!"); se completan las 100.
- A2 `ExecutionContextSpec.scala:85-108`: evitar inanición con `Await`: 100 tareas anidadas, cada una espera con `Await.ready` a otra tarea batchable que crea; el latch de 101 se completa (el reenvío ante bloqueo evita el interbloqueo).
- A3 `ExecutionContextSpec.scala:110-135`: tareas que usan `blocking {}` varias veces dentro de un callback de `Future` terminan (`Await.result(f, 3 s) == ()`).
- A4 `ExecutionContextSpec.scala:137-151`: `blocking` anidado 3 veces devuelve 42.
- A5 `ExecutionContextSpec.scala:153-166`: `ExecutionContexts.parasitic` + `blocking` ejecuta el cuerpo (x == 1) en el mismo hilo.
- A6 `ExecutionContextSpec.scala:168-185`: con `CallingThreadDispatcher`, `blocking` dentro de un actor que reenvía a otro entrega "hello" sin espera (`expectMsg(0.seconds)`).
- A7 `ExecutionContextSpec.scala:187-208`: `CallingThreadDispatcher` como executor, tarea batchable que encola otra batchable y luego llama `blocking`: la encolada se ejecuta (x == 1).
- A8 `ScalaBatchable` por tipo de runnable y el reenvío tras excepción: *no afirmada por spec*. Caracterización propuesta: una tarea del lote que entra en pánico -> las restantes se ejecutan igualmente.

**Validación prevista.** A1 portado (sin `Future` de Scala: tareas anidadas); A2-A4, A6, A7 dependen de `BlockContext` y no son portables 1:1 (Q-DSP-7); A5 portado para el ejecutor inline; A8 caracterización.

**Decisiones.** Q-DSP-7.

## AC-DSP-017 ExecutionContexts / Futures helpers (descompuesta en AC-DSP-017.1 y AC-DSP-017.2)

Fila descompuesta: los contextos de ejecución (`ExecutionContexts`, `SameThreadExecutionContext`) son AC-DSP-017.1 y los ayudantes de futuros/`CompletionStage` (`Futures`, `CompletionStages`, `japi`) son AC-DSP-017.2. Una referencia a AC-DSP-017 significa ambos hijos. Q-DSP-3 sigue proponiendo qué parte se porta.

## AC-DSP-017.1 ExecutionContexts y SameThreadExecutionContext (parasitic)

*unidad DSP · visibilidad public/internal · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `ExecutionContexts` (API Java) construye `ExecutionContextExecutor` a partir de un `Executor`/`ExecutorService` (opcionalmente con `errorReporter`) y expone `global()`; `ExecutionContexts.parasitic` (interno, estable) y el obsoleto `sameThreadExecutionContext` ejecutan la tarea en el hilo que la envía (en Scala 2.13+ es `scala.concurrent.ExecutionContext.parasitic`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Future.scala:25-81`; `actor/src/main/scala-2.13/org/apache/pekko/dispatch/internal/SameThreadExecutionContext.scala` (y variantes 2.12/3).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:52-69,204-214`; `SameThreadExecutionContextSpec.scala:26-70`.

**Precondiciones.** `fromExecutorService` exige un `ExecutorService` que el contexto devuelto puede apagar.

**Comportamiento.** `fromExecutor(ex)`/`fromExecutor(ex, reporter)`/`fromExecutorService(...)` delegan en `ExecutionContext.fromExecutor*` de Scala (el reportero por defecto imprime en `System.err` salvo `InterruptedException`/`ControlThrowable`). `parasitic` ejecuta de forma síncrona (con trampolín para tareas anidadas en Scala 2.13) y es la forma de ahorrar un salto al pool para acciones no bloqueantes y que no lanzan. `sameThreadExecutionContext` (obsoleto desde 2.6.4) es un `BatchingExecutor` con `resubmitOnBlock = false` cuyo `unbatchedExecute` delega en `parasitic`.

**Errores.** Sin errores propios.

**Orden / concurrencia.** Las continuaciones encadenadas con `parasitic` corren en el hilo del que completa el futuro anterior.

**Dependencias.** AC-DSP-016.2.

**Equivalente Rust propuesto.** `Executor` en línea (`InlineExecutor`) con trampolín opcional; `fromExecutor`/`global` se corresponden con el runtime elegido (Q-X-4). Si Q-DSP-3 excluye los helpers Java, queda sólo `InlineExecutor` y la conversión `Arc<dyn Executor>`.

**Aserciones.**
- A1 `ExecutionContextSpec.scala:36-53`: `fromExecutor`/`fromExecutorService` construyen contextos no nulos con un pool cacheado.
- A2 `SameThreadExecutionContextSpec.scala:30-38`: en 2.12 la clase empieza por `...SameThreadExecutionContext`; en 2.13+ es `scala.concurrent.ExecutionContext$parasitic$`.
- A3 `SameThreadExecutionContextSpec.scala:40-52` y `:54-66`: una continuación con `SameThreadExecutionContext()` corre en el MISMO hilo (mismo nombre) que el `map` previo hecho en `system.dispatcher` y en `ExecutionContext.global`.
- A4 `ExecutionContextSpec.scala:153-166`: `parasitic` + `blocking` (x == 1).

**Validación prevista.** A3/A4 portados para `InlineExecutor` (misma tarea/hilo); A1/A2 dependen de la decisión Q-DSP-3.

**Decisiones.** Q-DSP-3.

## AC-DSP-017.2 Futures / CompletionStages / japi (ayudantes Java de futuros)

*unidad DSP · visibilidad public (Java) · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Utilidades para Java sobre `scala.concurrent.Future` y `CompletionStage`: `Futures.asJava/future/promise/failed/successful` y los obsoletos `find/firstCompletedOf/fold/reduce/sequence/traverse/failedCompletionStage`; `CompletionStages.asScala/find/firstCompletedOf/fold/reduce/sequence/traverse`; clases puente `japi` (`OnSuccess`, `OnFailure`, `OnComplete`, `Recover`, `Filter`, `Foreach`, `Mapper`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/Future.scala:86-140`; `actor/src/main/scala/org/apache/pekko/dispatch/CompletionStages.scala:27-221`.
- Pruebas: `actor-tests/src/test/java/org/apache/pekko/dispatch/CompletionStagesTests.java:44-144`; `JavaFutureTests.java:40-404`.

**Precondiciones.** Los métodos con `executor` null (`sequence`/`traverse`) usan `thenCombine` síncrono; con executor usan `thenCombineAsync(executor)`.

**Comportamiento.**
- `find(stages, predicate)`: busca en secuencia (una etapa tras otra, `thenCompose`); devuelve `Optional.of(t)` para la primera que cumple, o `Optional.empty()` al agotar.
- `firstCompletedOf(stages)`: iterable vacío -> etapa que nunca completa; si no, completa con el primer resultado o excepción (un `AtomicReference` garantiza que sólo la primera completa).
- `fold(zero, stages, f)`: plegado secuencial en el orden del iterable; el primer fallo corta.
- `reduce(stages, f)`: como `fold` con la primera etapa como inicial; iterable vacío -> etapa fallida con `NoSuchElementException("reduce of an empty iterable of CompletionStages")`.
- `sequence`/`traverse`: lista en el orden de entrada; entrada vacía -> lista vacía.
- `asScala`/`asJava`: conversión sin cambio de valor ni de fallo.

**Errores.** Los de arriba; los demás propagan la excepción de la etapa.

**Orden / concurrencia.** `find`/`fold`/`reduce` son secuenciales (no evalúan en paralelo); `sequence`/`traverse` combinan en orden.

**Dependencias.** AC-DSP-017.1.

**Equivalente Rust propuesto.** `std::future`/`futures` (`join_all`, `select_ok`, `try_fold`) sin API propia: no se porta (propuesta de exclusión Q-DSP-3, la fila sigue en alcance hasta autorización del Owner). Se documenta la correspondencia para quien migre.

**Aserciones.**
- A1 `CompletionStagesTests.java:34-46`: `asScala` convierte una etapa completada (42) y una fallida (`RuntimeException`, "Simulated failure").
- A2 `CompletionStagesTests.java:47-58`: `find` encuentra el 3 entre 0..4 y devuelve `Optional.empty` para 42. A3 `:59-76`: `firstCompletedOf` devuelve 42 de la etapa 3 completada a los 200 ms. A4 `:77-86`: `fold(0, 1..5, sum) == 15`. A5 `:87-105`: `reduce == 15`; con lista vacía, SI `get` lanza, la causa es `NoSuchElementException` con el mensaje literal (si no lanza el test no falla: aserción débil). A6 `:106-140`: `sequence` y `traverse` devuelven [1..5] con executor nulo y con `ForkJoinPool.commonPool()`, y lista vacía para entrada vacía.
- A7 `JavaFutureTests.java:50-56` crear `CompletionStage` (`Futures.asJava`, 42); `:57-80` `map` con `Mapper` ("Hello World"); `:81-135` callbacks `OnComplete` para éxito, excepción (`NullPointerException`) y completado; `:137-153` `Foreach`; `:155-178` `flatMap` (1000); `:179-201` `filter` con `Filter.filterOf`; `:203-226` `Futures.sequence`; `:227-258` `fold`; `:259-289` `reduce`; `:290-319` `traverse`; `:320-346` `find`; `:348-403` `block`/`mapTo`/`recover`/`recoverWith` invocables desde Java. Todas con `Await.result(..., 5 s)` y latches de 5 s.

**Validación prevista.** Si Q-DSP-3 los excluye, no se portan; si se portan, portar A1-A6 con `futures`. Oráculo: no aplica.

**Decisiones.** Q-DSP-3.

## AC-DSP-018 SerializedSuspendableExecutionContext

*unidad DSP · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Envoltorio de un `ExecutionContext` que garantiza ejecución en serie de las tareas enviadas, hasta `throughput` tareas por barrido antes de ceder el hilo, y permite suspender y reanudar el procesamiento.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/SerializedSuspendableExecutionContext.scala:25-111`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/dispatch/ExecutionContextSpec.scala:259-341`.

**Precondiciones.** `throughput > 0`.

**Comportamiento.** Estado atómico con bits On(1) y Suspended(2) (Off = 0) y una cola MPSC. `execute(task)`: encola y `attach()` (en `finally`). `attach()`: si la cola no está vacía y `CAS(Off -> On)` -> `context.execute(this)`. `run()`: mientras `done < throughput && state == On` (igualdad exacta, no suspendido) hace `poll()`, ejecuta (un `NonFatal` va a `context.reportFailure`), y al final `remState(On)` (que vuelve a llamar `attach()`). `suspend()`: `state |= Suspended` (idempotente; no cede el hilo hasta el final del sweep en curso, pero el bucle sale porque `state != On`). `resume()`: `state &= ~Suspended` y `attach()`. `apply` desenvuelve otro `SerializedSuspendableExecutionContext` a su contexto subyacente. `size()` es O(n); `toString` = "Off" | "On" | "Off & Suspended" | "On & Suspended".

**Errores.** `throughput <= 0` -> `IllegalArgumentException("requirement failed: SerializedSuspendableExecutionContext.throughput must be greater than 0 but was <n>")`.

**Orden / concurrencia.** FIFO y exclusión mutua entre tareas (nunca dos a la vez); cede el hilo cada `throughput` tareas o al suspender.

**Dependencias.** AC-UTIL (`AbstractNodeQueue`), AC-DSP-016.

**Equivalente Rust propuesto.** `SerializedSuspendable { queue, state: AtomicU8, throughput, inner: Arc<dyn Executor> }` con los mismos estados; `size()` aproximado.

**Aserciones.**
- A1 `ExecutionContextSpec.scala:212-230`: `throughput 1`; `+1`, luego una tarea que suspende y duplica -> `counter == 2`; las dos siguientes (`+4`, `*2`) quedan encoladas (`size() == 2`, tras 500 ms `counter == 2`); `resume()` -> `counter == 12`; otra tarea `*2` -> 24 y `isEmpty`.
- A2 `ExecutionContextSpec.scala:232-253`: con `throughput = 25`, 1000 tareas encoladas en suspensión (`size() == 1000`); tras `resume()` se completan y el executor subyacente recibió exactamente `total/throughput = 40` envíos.
- A3 `ExecutionContextSpec.scala:255-266`: 10000 tareas que sólo incrementan si `c == i-1` se ejecutan en el orden de envío (`counter == 10000`).
- A4 `ExecutionContextSpec.scala:268-292`: `throughput = 25`, 13 tareas encoladas (una de ellas suspende dentro); tras `resume()` `counter == 2` (cede el hilo al suspenderse), tras otro `resume()` `counter == 10`, `isEmpty` y `submissions == 2`.

**Validación prevista.** A1-A4 portados 1:1 con un executor de prueba que cuente envíos. Oráculo: no aplica.

**Decisiones.** Q-DSP-3 (alcance; se trata como utilidad interna necesaria para persistencia/streams fuera de módulo).

## AC-DSP-019 Configuradores internos (PekkoForkJoinPool, PekkoJdk9ForkJoinPool, ForkJoinPoolConstants, CachingConfig)

*unidad DSP · visibilidad internal · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Piezas internas de los executors de JVM: (1) `PekkoForkJoinPool`/`PekkoJdk9ForkJoinPool`: `ForkJoinPool` que envuelve cada `Runnable` en una `PekkoForkJoinTask` y expone `atFullThrottle`; (2) `ForkJoinPoolConstants`: topes; (3) `CachingConfig`: `Config` con caché de `hasPath`/`getString`. Sin comportamiento propio observable fuera de rendimiento y de las salvaguardas de abajo.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/ForkJoinExecutorConfigurator.scala:49-98,119-129` (`PekkoForkJoinPool`, `PekkoForkJoinTask`, `validate`), `actor/src/main/scala-jdk-9/org/apache/pekko/dispatch/PekkoJdk9ForkJoinPool.scala:14-52 (1.7.1)`, `ForkJoinPoolConstants.scala:14-23`, `CachingConfig.scala:14-215`.
- Usos: `Dispatchers.scala:119`, `AbstractDispatcher.scala:361`.

**Precondiciones.** `ForkJoinExecutorConfigurator` exige una `ForkJoinPool.ForkJoinWorkerThreadFactory`.

**Comportamiento.**
- `execute(null)` -> `NullPointerException("Runnable was null")`; un `Runnable` que no es `ForkJoinTask` se envuelve en `PekkoForkJoinTask`.
- `PekkoForkJoinTask.exec()`: ejecuta; `InterruptedException` -> restablece el flag y devuelve false; otro `Throwable` -> invoca el `UncaughtExceptionHandler` del hilo y lo relanza. `getRawResult`/`setRawResult` son no-ops.
- `atFullThrottle() = getActiveThreadCount >= getParallelism`.
- `PekkoJdk9ForkJoinPool(parallelism, factory, maximumPoolSize, handler, asyncMode, minimumRunnable)`: construye `ForkJoinPool(parallelism, factory, handler, asyncMode, 0, maximumPoolSize, minimumRunnable, null, DefaultKeepAliveMillis = 60000 ms)`.
- `ForkJoinPoolConstants.MaxCap = 0x7FFF = 32767` (valor por defecto de `maximum-pool-size`) y `DefaultKeepAliveMillis = 60000`.
- `CachingConfig`: cachea por ruta `hasPath` y `getString` (el resto delega); comparte el mapa si envuelve otro `CachingConfig`; para rutas inválidas ejecuta el código real para obtener la excepción original; `withFallback`/`resolve`/`atPath`... devuelven nuevos `CachingConfig`. No cambia ningún resultado respecto de la `Config` envuelta.

**Errores.** `ForkJoinExecutorConfigurator.validate` con una fábrica de hilos inadecuada -> `IllegalStateException("The prerequisites for the ForkJoinExecutorConfigurator is a ForkJoinPool.ForkJoinWorkerThreadFactory!")`.

**Orden / concurrencia.** `CachingConfig` usa `ConcurrentHashMap.putIfAbsent`; las demás piezas son wrappers de la JDK.

**Dependencias.** AC-DSP-008, AC-DSP-004.

**Equivalente Rust propuesto.** Ninguno propio: el pool de Rust es el de AC-DSP-008; `execute(null)` no existe (el tipo no admite nulos); `CachingConfig` es una optimización que no se porta (detalle interno sin contrato independiente).

**Aserciones.** *No afirmada por spec* (sin pruebas directas en `actor-tests`: búsqueda de `CachingConfig|PekkoForkJoinPool|PekkoJdk9`). Caracterización propuesta: `maximum-pool-size` por defecto 32767; `atFullThrottle` coherente con el estado del pool de Rust.

**Validación prevista.** Ninguna específica más allá de la configuración por defecto (AC-DSP-008).

**Decisiones.** Q-DSP-1.

## AC-DSP-020 Envelope / mensaje nulo

*unidad DSP · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `Envelope(message, sender)` es el sobre inmutable de un mensaje de usuario. La fábrica `Envelope(message, sender, system)` rechaza el mensaje `null` y reemplaza un remitente `noSender` por `system.deadLetters`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:34-53`; uso en `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:329-335` (`sendMessage(message, sender)`), `:586`.
- Errores y excepción: `actor/src/main/scala/org/apache/pekko/actor/Actor.scala:273-275` (`InvalidMessageException`).

**Precondiciones.** Ninguna.

**Comportamiento.** `message == null` y `sender eq Actor.noSender` -> `InvalidMessageException("Message is null.")`; con remitente -> `InvalidMessageException(s"Message sent from [$sender] is null.")`; si no, `new Envelope(message, if (sender ne noSender) sender else system.deadLetters)`. `copy(message, sender)` reconstruye el sobre vía `Envelope(message, sender)` (marcado `noinline` para instrumentación).

**Errores.** `InvalidMessageException` (hereda de `PekkoException`, constructor `private[pekko]`). Se lanza al llamante de `tell` ANTES del `try` de `ActorCell.sendMessage(Envelope)` (la construcción del sobre está fuera del `catch handleException`). Nótese que `EmptyLocalActorRef.!`/`DeadLetterActorRef.!` lanzan `InvalidMessageException("Message is null")` (sin punto final, `ActorRef.scala:685,737`).

**Orden / concurrencia.** Inmutable; seguro entre hilos.

**Dependencias.** AC-REF (ActorRef `!`), AC-CELL (sendMessage), AC-MBX-004.

**Equivalente Rust propuesto.** `Envelope { message: Box<dyn Any + Send>, sender: ActorRef }` con `Envelope::new(message, sender, system)`; "null" no existe en Rust (Q-X-3), de modo que el error se convierte en una condición imposible por construcción; se conserva el reemplazo `no_sender -> dead_letters`.

**Aserciones.** *No afirmada por spec* (ninguna prueba de `actor-tests` referencia `InvalidMessageException`/"is null" para `Envelope`; las apariciones en `actor-typed-tests` son del módulo typed). Caracterización propuesta: sobre con remitente ausente -> `dead_letters`; con `Option<ActorRef>` vacío y mensaje inválido (si Q-X-3 admite un caso equivalente) -> error con el texto literal.

**Validación prevista.** Caracterización unitaria. Oráculo: no aplica.

**Decisiones.** Q-X-3.

## AC-DSP-021 BalancingDispatcherConfigurator

- **Contrato observable / Fuente / Condiciones**: `actor/src/main/scala/org/apache/pekko/dispatch/Dispatchers.scala:310-373`. El `mailbox-requirement` vacío se enmienda a `org.apache.pekko.dispatch.MultipleConsumerSemantics` (`:317-323`). Validaciones: (1) el requisito debe implementar `MultipleConsumerSemantics` -> `IllegalArgumentException("BalancingDispatcher must have 'mailbox-requirement' which implements org.apache.pekko.dispatch.MultipleConsumerSemantics; dispatcher [<id>] has [<requisito>]")` (`:338-341`); (2) con `mailbox` configurado, el tipo producido debe cumplir el requisito -> `IllegalArgumentException("BalancingDispatcher [<id>] has 'mailbox' [<clase>] which is incompatible with 'mailbox-requirement' [<requisito>]")` (`:343-348`); (3) idem con `mailbox-type` -> `"... has 'mailbox-type' [<clase>] which is incompatible with 'mailbox-requirement' [<requisito>]"` (`:349-354`); sin ninguno de los dos, `mailboxes.lookupByQueueType(requirement)`. La instancia se crea una vez (`dispatcher()` devuelve siempre la misma).
- **Orden**: validación en construcción del dispatcher (al primer `lookup`).
- **Dependencias**: AC-DSP-012, AC-MBX-013.
- **Equivalente Rust**: validación al construir (`Result<_, ConfigurationError>`).
- **Aserciones**: A1 dispatcher balanceador sin mailbox => cola ilimitada; con `mailbox-type` acotado => cola acotada; con `mailbox-requirement` mapeado => cola de la tabla `requirements` — `ActorMailboxSpec.scala:439-458`; A2 los tres mensajes de error: *no afirmada por spec* (`Dispatchers.scala:338-354`); caracterización propuesta con el texto literal.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-DSP-023 MessageDispatcher.suspend/resume por actor (omisión agregada)

*unidad DSP · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Punto de entrada del dispatcher para suspender y reanudar el procesamiento de mensajes de usuario de un actor (usado por la supervisión, `ActorCell.suspend/resume`): después de `suspend(actor)` el dispatcher no empieza a procesar mensajes de usuario de ese actor; `resume(actor)` los reanuda y vuelve a planificar el mailbox.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:316-332`; estado en `Mailbox.scala:141-170`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:360-382`.

**Precondiciones.** El mailbox del actor pertenece a este dispatcher y a este actor.

**Comportamiento.** `suspend(actor)`: `mbox = actor.mailbox`; sólo si `(mbox.actor eq actor) && (mbox.dispatcher eq this)` -> `mbox.suspend()`. `resume(actor)`: misma guarda; si además `mbox.resume()` devuelve true (el contador llegó a 0) -> `registerForExecution(mbox, false, false)`. Si la guarda falla (mailbox ya cambiado por el deadLetterMailbox o de otro dispatcher) no hace nada.

**Errores.** Ninguno.

**Orden / concurrencia.** La guarda y la operación no son atómicas entre sí, pero cada transición del mailbox es CAS (AC-MBX-001).

**Dependencias.** AC-MBX-001, AC-DSP-002, AC-CELL-007.1.

**Equivalente Rust propuesto.** `Dispatcher::suspend(&self, cell)` / `resume(&self, cell)` con la misma guarda (identidad de celda y de dispatcher).

**Aserciones.**
- A1 `ActorModelSpec.scala:359-381`: ver AC-MBX-001 A1 (suspensión, no procesado, reanudación, `suspensions = resumes = 1`).
- A2 las guardas de identidad y el reencolado tras `resume`: *no afirmada por spec*. Caracterización propuesta: suspender un actor ya detenido no tiene efecto; reanudar con contador 2 -> 1 no re-registra, 1 -> 0 sí.

**Validación prevista.** A1 portado (en AC-MBX-001); A2 caracterización.

**Decisiones.** Q-MBX-6.

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

- **Contrato observable / Fuente / Condiciones**: ver TSV (contrato refinado). Variantes: Runnable, closure, `(receiver, message)` con `sender` implícito (`actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala:355-419`); `maxFrequency` (`:499`).
- **Orden/concurrencia**: ejecuta en el `ExecutionContext` pasado; delay<=0 ejecuta ya en ese ec (devuelve un `Cancellable` no cancelable: `cancel()` y `isCancelled` son false, `LightArrayRevolverScheduler.scala:205-209,420-424`).
- **Dependencias**: AC-DSP-016.1.
- **Equivalente Rust**: `Scheduler::schedule_once(delay, task) -> Cancellable` con reloj inyectable (Q-SCH-1). Diferencia: `ExecutionContext` => `Executor`.
- **Aserciones**: A1 dos tareas a 300 ms (una a un actor y una closure, sobre un latch de 3) no corren antes de 100 ms (`getCount == 3`) y tras 2 s el latch queda en 1 — `SchedulerSpec.scala:97-116`; A2 cancelar 10 tareas de 1 s no entrega nada en 2 s — `:121-125`; A3 nunca dispara antes de tiempo: 300 mensajes a 20 ms, el actor lanza si `nanoTime < ts` — `:156-176`; A4 delay máximo tick*Int.MaxValue se acepta, +tick => IllegalArgumentException — `:477-484`; A5 un timeout múltiplo de la rueda (3200 ms) se ejecuta antes de 5 s — `:178-188`; A6 100000 tareas con retardos aleatorios <3 s se ejecutan todas (latencias medidas, sin aserción numérica) — `:190-212`.
- **Oráculo**: no aplica (timing); la función de redondeo `roundUp` sí es pura y comparable (AC-SCH-008.2).
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

## AC-SCH-004 schedule deprecado

*unidad SCH · visibilidad public (deprecado) · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `schedule(initialDelay, interval, runnable)(ec)` (deprecado desde Akka 2.6.0: "Use scheduleWithFixedDelay or scheduleAtFixedRate instead. This has the same semantics as scheduleAtFixedRate, but scheduleWithFixedDelay is often preferred.") es el método abstracto que cada scheduler implementa con semántica de tasa fija. Las demás sobrecargas (`java.time.Duration`, closure, `(receiver, message)`) delegan en él, y `scheduleAtFixedRate(initialDelay, interval)(runnable)` es simplemente `schedule(initialDelay, interval, runnable)`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala:223-225` (`scheduleAtFixedRate` -> `schedule`), `:343-420` (sobrecargas deprecadas; la variante con mensaje lanza `SchedulerException("timer active for terminated actor")` si el receptor terminó).
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:134-170` (implementación de tasa fija).
- `testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:53-55` (otra implementación).
- Pruebas: `testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:24-106`; indirectas `actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:83-94,219-448`.

**Precondiciones.** Las de AC-SCH-003 (intervalo > 0 y dentro del máximo de la rueda).

**Comportamiento.** Igual que AC-SCH-003: primera ejecución tras `initialDelay`, después una cada `interval` compensando la deriva (`max(delay - deriva, 1 ns)`), hasta cancelación, fallo de la tarea o cierre.

**Errores.** Los de AC-SCH-003; además, es el punto donde se lanza `IllegalArgumentException` por intervalo <= 0 (`checkPeriod`, `LightArrayRevolverScheduler.scala:134-137`).

**Orden / concurrencia.** Como AC-SCH-003. Cualquier implementación personalizada de `Scheduler` DEBE implementar este método (es el abstracto).

**Dependencias.** AC-SCH-003, AC-SCH-010.

**Equivalente Rust propuesto.** `Scheduler::schedule_at_fixed_rate` como método requerido del trait y un alias `#[deprecated] fn schedule(...)` que lo invoca (Q-SCH-4 (a)); el trait no necesita exponer dos nombres si el Owner elige (b).

**Aserciones.**
- A1 `ExplicitlyTriggeredSchedulerSpec.scala:24-30`: `scheduler.schedule(0 s, 5 s, task)` con `timePasses(12 s)` ejecuta 3 veces (t = 0, 5, 10). A2 `:32-38`: con retardo inicial 5 s y paso 12 s, 2 veces. A3 `:58-69`/`:71-84`/`:86-93`: cancelar detiene las repeticiones (devuelve true); cancelar una de tres tareas idénticas deja las otras dos; cancelar inmediatamente evita toda ejecución. A4 `:95-106`: avanzar el tiempo en tramos (7 s, 8 s, 18 s) da 1, 1, 3 ejecuciones. Estas pruebas ejercen `schedule` directamente, pero sobre el scheduler de testkit, no sobre la rueda.
- A5 la ruta `scheduleAtFixedRate -> schedule` en la rueda: indirecta, `ScheduleAtFixedRateAdapter` (`SchedulerSpec.scala:81-92`) usa `scheduleAtFixedRate`, que delega en `schedule`.
- A6 el atributo `@deprecated` y su mensaje: *no afirmada por spec*.

**Validación prevista.** A1-A4 portados contra el scheduler manual de rakka (AC-SCH-010); A5 cubierto por las pruebas de AC-SCH-003.

**Decisiones.** Q-SCH-4.

## AC-SCH-005 Cancellable
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: cancelación atómica (CAS).
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: `Cancellable(Arc<AtomicU8>)` o equivalente con `cancel() -> bool`, `is_cancelled()`, constante `ALREADY_CANCELLED`.
- **Aserciones**: A1 cancel antes de ejecutar: true, is_cancelled true, segunda llamada false — `SchedulerSpec.scala:127-133`; A2 tras ejecución: false/false/false/false — `:146-154`; A3 periódica: cancelar durante retardo inicial => 0 ticks — `:316-330`; A4 cancelar tras el retardo => 1 tick — `:332-346`.
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-SCH-006 SchedulerTask.cancelled()

- **Contrato observable / Fuente**: ver TSV; `actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala:429-447` (`SchedulerTask.cancelled()` es un callback vacío por defecto), `:101-104` (el envoltorio de `scheduleWithFixedDelay` lo reenvía), `LightArrayRevolverScheduler.scala:154-157,398-412` (el de tasa fija y `TaskHolder.cancel`).
- **Condiciones/Orden**: se invoca (en el `ExecutionContext` de la tarea) al cancelar antes de ejecutar; en repetidas se reenvía al `SchedulerTask` original; si el `execute` del callback falla, `reportFailure`.
- **Dependencias**: AC-SCH-005.
- **Equivalente Rust**: método opcional `on_cancel` en el trait de tarea.
- **Aserciones**: A1 cancelar un `scheduleOnce` de 100 ms antes de ejecutar invoca `cancelled()` (latch en <=100 ms) — `SchedulerSpec.scala:135-144`; A2 cancelar un `scheduleWithFixedDelay` DESPUÉS del retardo inicial invoca `cancelled()` del `SchedulerTask` original — `:348-363` (usa `system.scheduler.scheduleWithFixedDelay` directamente: el reenvío en la ruta de tasa fija NO está afirmado por spec; caracterización propuesta).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-SCH-007 Tarea periódica a actor

- **Contrato observable / Fuente**: ver TSV; `actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala:150-164,340`.
- **Condiciones**: texto `SchedulerException("timer active for terminated actor")`; la excepción de la tarea que no es `SchedulerException` (ni `IllegalStateException` con causa `SchedulerException`) NO se atrapa: la repetición se aborta porque no se reprograma, y el fallo se loguea por el `ExecutionContext`. En tasa fija, sólo `SchedulerException` se ignora.
- **Orden/Dependencias**: AC-SCH-002.
- **Equivalente Rust**: `is_terminated()` tras enviar.
- **Aserciones**: A1 tras `system.stop(actor)` no llegan más mensajes en 500 ms — `SchedulerSpec.scala:252-267`; A2 excepción de la tarea (RuntimeException "TEST" en la 3.ª ejecución): se reciben 1, 2, 3, no más en 200 ms y se loguea exactamente una vez (`EventFilter[Exception]("TEST", occurrences = 1)`) — `:269-283`; A3 `IllegalStateException` en la 1.ª ejecución (no se envía nada y se loguea "TEST-1") y en una posterior ("TEST-3" tras 1, 2, 3): también detiene la repetición — `:285-314`; A4 reinicio del actor retoma el schedule: un supervisor `AllForOne(3, 1 s)` reinicia al actor ante `Crash` (a los 1000 ms) y el schedule de `Ping` cada 500 ms continúa: tras `postRestart` se alcanzan 6 pings en <=5 s — `:368-397`. Todos se ejecutan para `scheduleWithFixedDelay` y `scheduleAtFixedRate` (`SchedulerSpec.scala:216-418`).
- **Oráculo**: no aplica.
- **Decisiones**: ninguna.

## AC-SCH-008 LightArrayRevolverScheduler (descompuesta en AC-SCH-008.1, AC-SCH-008.2 y AC-SCH-008.3)

Fila descompuesta: la validación de configuración es AC-SCH-008.1, el redondeo y los límites de retardo AC-SCH-008.2 y la mecánica de la rueda (buckets, vueltas, relojes que dan la vuelta) AC-SCH-008.3. Una referencia a AC-SCH-008 significa los tres hijos. Fuente conjunta: `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:50-450`. Decisión asociada: Q-SCH-1.

## AC-SCH-008.1 LARS: validación de configuración (tick-duration, ticks-per-wheel, shutdown-timeout, maxFrequency)

*unidad SCH · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** El scheduler por defecto valida y deriva su configuración al construirse: `ticks-per-wheel` potencia de 2, `tick-duration` con mínimo de 1 ms (10 ms en Windows) y `maxFrequency = 1 s / TickDuration`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:57-97,212`; `actor/src/main/resources/reference.conf:934-975`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:864-878`.

**Precondiciones.** Config con `pekko.scheduler.ticks-per-wheel` (512 por defecto), `tick-duration` (10 ms), `error-on-tick-duration-verification-failed` (on), `shutdown-timeout` (5 s).

**Comportamiento.**
- `WheelSize = ticks-per-wheel` con `requiring(ticks => (ticks & (ticks - 1)) == 0, "ticks-per-wheel must be a power of 2")`.
- `TickDuration`: se lee con `getMillisDuration` (AC-UTIL-002.4: trunca a milisegundos, por lo que `1ns` se convierte en 0 ms). Si `< 10 ms`: en Windows -> error o ajuste a 10 ms; si no es Windows y `< 1 ms` -> error o ajuste a 1 ms; si no, se acepta tal cual (entre 1 y 10 ms en Linux se acepta). Con `error-on-tick-duration-verification-failed = off` se ajusta y se emite `log.warning` con el mismo texto seguido de `, adjusted to <N>ms now.`
- `ShutdownTimeout = getMillisDuration("shutdown-timeout")`; `maxFrequency = 1.second / TickDuration` (Double).

**Errores.** `IllegalArgumentException("requirement failed: ticks-per-wheel must be a power of 2")`; `IllegalArgumentException("requirement failed: minimum supported pekko.scheduler.tick-duration on Windows is 10ms")` (Windows) o `("requirement failed: minimum supported pekko.scheduler.tick-duration is 1ms")`. Bordes de la validación: `ticks-per-wheel = 0` pasa `(t & (t-1)) == 0` y `Int.MinValue` también (el segundo falla después con `NegativeArraySizeException`); los negativos que no son `Int.MinValue` se rechazan.

**Orden / concurrencia.** Todo ocurre en el constructor, antes de arrancar el hilo del timer.

**Dependencias.** AC-UTIL-002.3 (`requiring`), AC-UTIL-002.4, AC-UTIL-026 (`isWindows`), AC-CFG-001.

**Equivalente Rust propuesto.** `SchedulerSettings::from_config(&Config) -> Result<_, ConfigError>` con los mismos mensajes y la rama de Windows detrás de `cfg!(windows)`; mismo truncamiento a milisegundos. Sin diferencias observables salvo el borde `Int.MinValue`, que en Rust sería un error de validación explícito (diferencia documentada).

**Aserciones.**
- A1 `SchedulerSpec.scala:859-873`: con `tick-duration = 1ns` y `error-on-tick-duration-verification-failed = off`, `TickDuration == 1.millis` (10 ms en Windows).
- A2 error con `error-on... = on`, texto del `IllegalArgumentException`, advertencia con el sufijo `adjusted to`, y el borde de `ticks-per-wheel`: *no afirmada por spec*. Caracterización propuesta: tabla de (tick-duration, plataforma, flag) -> (resultado, mensaje); `ticks-per-wheel` 3/0/-8/512.
- A3 `testConfRevolver` (`SchedulerSpec.scala:37-41`) fija `ticks-per-wheel = 32`: la rueda funciona con ese tamaño (todas las pruebas de `LightArrayRevolverSchedulerSpec`).

**Validación prevista.** A1 portado; A2 caracterización (la rama de Windows se prueba inyectando un indicador de plataforma). Oráculo diferencial opcional: lectura de la configuración con `jshell` (funciones puras tras construir el scheduler).

**Decisiones.** Q-SCH-1.

## AC-SCH-008.2 LARS: redondeo al tick y límites de retardo

*unidad SCH · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Los retardos se redondean HACIA ARRIBA a múltiplos del tick; los retardos <= 0 se ejecutan de inmediato en el `ExecutionContext` dado; se rechazan los retardos demasiado lejanos y los intervalos <= 0.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:93-97` (`roundUp`), `:127-168` (`scheduleWithFixedDelay`, `schedule`, `scheduleOnce`), `:205-235` (`schedule` privado, `checkPeriod`, `checkMaxDelay`).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:480-487,705-785`.

**Precondiciones.** Scheduler no parado para los casos que encolan.

**Comportamiento.**
- `roundUp(d)`: `dn = d.toNanos`; `r = ((dn - 1) / tickNanos + 1) * tickNanos`; devuelve `r.nanos` si `r != dn && r > 0 && dn > 0`, si no `d` (los no positivos y los desbordes quedan sin tocar).
- `scheduleOnce(delay, runnable)`: `schedule(ec, runnable, roundUp(delay))`; un `SchedulerException(msg)` se convierte en `IllegalStateException(msg, cause)`.
- `schedule(ec, r, delay)` (privado): `delay.length <= 0` -> si el scheduler está parado `SchedulerException("cannot enqueue after timer shutdown")`; si no `ec.execute(r)` en el acto y devuelve un `Cancellable` inerte; `delay > 0` -> parado => `SchedulerException(...)`; `checkMaxDelay(delayNanos)`; `ticks = (delayNanos / tickNanos).toInt`; encola un `TaskHolder`; si tras encolar el scheduler se paró y la tarea se pudo cancelar -> `SchedulerException(...)`.
- `scheduleWithFixedDelay` y `schedule` (tasa fija): `checkPeriod(delay)` y `checkMaxDelay(roundUp(delay).toNanos)`; la tasa fija programa la siguiente ejecución en `max(delay - deriva, 1 ns)` SIN `roundUp` (el redondeo ocurre al colocar la tarea en la rueda, AC-SCH-008.3).
- Una tarea de tasa fija sólo ignora `SchedulerException`; cualquier otra excepción termina la repetición. El constructor de `AtomicCancellable` convierte un `SchedulerException` del primer disparo en `IllegalStateException`.

**Errores.** `checkPeriod`: `IllegalArgumentException("Task scheduled with [<delay.toSeconds>] seconds delay, which means creating an infinite loop. The expected delay must be greater than 0.")`. `checkMaxDelay` (`delayNanos / tickNanos > Int.MaxValue`): `IllegalArgumentException("Task scheduled with [<s>] seconds delay, which is too far in future, maximum delay is [<m>] seconds")` con `s = delayNanos.nanos.toSeconds` y `m = (tickNanos * Int.MaxValue).nanos.toSeconds - 1`. Scheduler parado: `SchedulerException("cannot enqueue after timer shutdown")` (`IllegalStateException` con el mismo mensaje en `scheduleOnce`).

**Orden / concurrencia.** El estado `stopped` es un `AtomicReference`; el chequeo posterior a encolar evita aceptar tareas durante el cierre (AC-SCH-009).

**Dependencias.** AC-SCH-008.1, AC-SCH-009.

**Equivalente Rust propuesto.** Función pura `round_up(delay, tick) -> Duration`, `check_period`, `check_max_delay` con los mismos mensajes (los segundos con la misma división entera). Sin diferencias observables.

**Aserciones.**
- A1 `SchedulerSpec.scala:477-484`: `maxDelay = tickDuration * Int.MaxValue` se acepta; `maxDelay + tickDuration` -> `IllegalArgumentException`.
- A2 `SchedulerSpec.scala:701-718` (tasa fija) y `:721-738` (retardo fijo): intervalo cero o negativo -> `IllegalArgumentException`, y no se entrega ningún mensaje en 1 s.
- A3 `SchedulerSpec.scala:743-761`: un retardo inicial máximo se acepta y +tick se rechaza; un intervalo máximo (`tick * Int.MaxValue`) se acepta (el primer mensaje llega) y +tick se rechaza; para ambas variantes. A4 `:763-781`: cero y negativo también con el adaptador.
- A5 valores exactos de `roundUp` (incluido el desborde), los textos de los mensajes y el `IllegalStateException` de `scheduleOnce`: *no afirmada por spec*. Caracterización propuesta; oráculo diferencial con `jshell` y el jar oficial (se puede instanciar `LightArrayRevolverScheduler` y observar las excepciones y mensajes; `roundUp` es privado, se infiere por la hora de disparo).

**Validación prevista.** A1-A4 portados; A5 caracterización + oráculo diferencial de los mensajes de error.

**Decisiones.** Q-SCH-1.

## AC-SCH-008.3 LARS: mecánica de la rueda (buckets, vueltas, tick y reloj que dan la vuelta)

*unidad SCH · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Un hilo de timer avanza una rueda circular de `ticks-per-wheel` buckets un tick cada `tick-duration`; cada tick ejecuta (en el `ExecutionContext` de cada tarea) las tareas del bucket vencidas y reubica las que necesitan más vueltas. La ejecución se produce en el tick siguiente al vencimiento (precisión de un tick, nunca antes de tiempo) y es correcta cuando el contador de ticks (`Int`) o el reloj (`Long`) dan la vuelta.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:99-125` (ganchos `clock`, `startTick`, `getShutdownTimeout`, `waitNanos`), `:200-203,249-352` (hilo, `checkQueue`, `nextTick`), `:357-430` (`TaskHolder`, estados `ExecutedTask`/`CancelledTask`).
- `actor/src/main/java/org/apache/pekko/dispatch/AbstractNodeQueue.java` (cola de entrada multi-productor).
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:489-663,789-862`.

**Precondiciones.** El scheduler arrancó (el hilo está en `nextTick`).

**Comportamiento.**
- `nextTick`: `time = clock()`; `sleepTime = start + totalTick * tickNanos - time`; si `sleepTime > 0`: `checkQueue(time)` y `waitNanos(sleepTime)`; si no, vacía el bucket `tick & wheelMask`: por cada nodo no cancelado, si `ticks >= WheelSize` -> `ticks -= WheelSize` y vuelve al bucket (otra vuelta); si no, `executeTask()`; luego `tick += 1; totalTick += 1`. Tras cada iteración comprueba `stopped`.
- `checkQueue(time)` (tareas nuevas de otros hilos): `ticks == 0` -> se ejecuta ya; si no, `futureTick = ((time - start + ticks * tickNanos + tickNanos - 1) / tickNanos).toInt`, `offset = futureTick - tick`, `bucket = futureTick & wheelMask`, `node.ticks = offset`. `futureTick` es un `Int` que da la vuelta: sólo importa la diferencia.
- `TaskHolder`: la tarea se extrae por CAS a `ExecutedTask` (ejecutada) o `CancelledTask` (cancelada); `cancel()` devuelve true sólo la primera vez y sólo antes de ejecutarse, y notifica `SchedulerTask.cancelled()`; `isCancelled` = tarea `CancelledTask`. Un `TaskHolder` cancelado queda como cascarón hasta que la rueda lo recorre.
- `waitNanos(n)` por defecto duerme `ceil(n / 1 ms)` milisegundos (en Windows múltiplos de 10 ms) y re-interrumpe si lo despiertan. `clock()` por defecto `System.nanoTime`; `startTick` por defecto 0.
- `executeTask`: `executionContext.execute(task)`; un `InterruptedException` re-interrumpe el hilo; un `NonFatal` va a `executionContext.reportFailure`.

**Errores.** Ninguno propio (los fallos del timer: AC-SCH-015).

**Orden / concurrencia.** Un hilo de timer; las tareas del mismo bucket se ejecutan en el orden del bucket (FIFO de inserción en la cola de nodos); tareas de buckets distintos en orden de vencimiento. Cancelación atómica (CAS) respecto de la ejecución.

**Dependencias.** AC-SCH-008.1, AC-SCH-008.2, AC-SCH-005, AC-SCH-015, AC-DSP-016.1.

**Equivalente Rust propuesto.** `WheelScheduler<C: Clock, W: Waiter>` con la rueda portada (Vec de colas, `u32/i32` con aritmética envolvente para `tick`, `i64` para `time`) y un `Waiter` inyectable (el `waitNanos` de Pekko) para que las pruebas determinísticas (`withScheduler` + `Driver`) se porten 1:1; en producción el `Waiter` duerme el hilo. Se conserva el redondeo al tick y la ejecución "en el siguiente tick". Alternativa (Q-SCH-1 (b)): `tokio::time` con redondeo emulado, que NO es equivalente (los tiempos de disparo cambian) y no se recomienda.

**Aserciones.** Las pruebas determinísticas usan `withScheduler(start, startTick, config)` (`SchedulerSpec.scala:785-857`): reloj, `waitNanos` y `startTick` sobreescritos; `driver.wakeUp(d)` avanza el reloj y `expectWait(d)` afirma la espera que el timer pide a continuación; `step = TickDuration`; `localEC` ejecuta en el hilo del timer.
- A1 `:550-563` ("execute multiple jobs at once when expiring multiple buckets"): 4 tareas a `step/2 + step*i` (i = 0..3); no hay mensajes en `step`; `wakeUp(step)` y `expectWait(step)`; `wakeUp(step*4 + step/2)` y `expectWait(step/2)`; llegan los 4 "hello" en 0 s: un solo despertar procesa varios buckets y la espera siguiente se alinea al tick.
- A2 `:565-578` ("properly defer jobs even when the timer thread oversleeps"): tarea a `3 * step`; `wakeUp(5*step)` -> `expectWait(step)`; `wakeUp(2*step)` -> `expectWait(step)`; `wakeUp(step)` -> llega "hello" y `expectWait(step)`.
- A3 `:580-605` ("correctly wrap around wheel rounds"): `ticks-per-wheel = 2`, 4 tareas a `step/2 + step*i`; secuencia `wakeUp(step)/expectWait(step)` con un "hello" entregado en cada uno de los 4 despertares siguientes al primero (reubicación de tareas con `ticks >= WheelSize`).
- A4 `:607-632` ("correctly execute jobs when clock wraps around"): igual con `start = Long.MaxValue - 200000000L` (el reloj desborda).
- A5 `:634-659` ("correctly wrap around ticks"): `startTick = Int.MaxValue - 100`; `wakeUp(step * targetTicks)` con `targetTicks = Int.MaxValue - 40 + 20`; 40 eventos `hello-i` a `step/2 + step*i` se entregan en orden, uno por despertar.
- A6 `:486-518` ("survive being stressed with cancellation"): 1.000.000 de tareas con retardo aleatorio < 3 s; se cancelan en el instante ~500 ms; las no canceladas (N - canceladas) llegan todas en 10 s y no queda ninguna extra (1 s sin mensajes).
- A7 `:520-548` ("survive vicious enqueueing"): `ticks-per-wheel = 2`; un hilo programa `TaskRunOnClose` sin parar mientras el principal programa 1.000.000 con retardo `step` o `2*step`, y se cierra el scheduler: los intentos tardíos fallan, `> 10` rondas, y los 1.000.000 aceptados se ejecutan (`counter == N`).
- A8 reubicación por `futureTick`, estados de `TaskHolder` y `waitNanos` por defecto: derivadas del código, *no afirmada por spec* aparte de A1-A5.

**Validación prevista.** A1-A5 portados 1:1 (un reloj y un `Waiter` manuales son exactamente el patrón del spec) y A6/A7 como pruebas de estrés reducidas; oráculo diferencial puntual para `futureTick`/`bucket` (función pura de `time`, `start`, `ticks`, `tick`).

**Decisiones.** Q-SCH-1.

## AC-SCH-009 close()
- **Contrato observable / Fuente / Condiciones**: ver TSV.
- **Orden**: espera al hilo hasta shutdown-timeout; ejecuta TaskRunOnClose.
- **Dependencias**: AC-DSP-001, AC-SCH-014.
- **Equivalente Rust**: `Scheduler::close()` + marcador `RunOnClose`.
- **Aserciones**: A1 cierre ejecuta sólo TaskRunOnClose — `SchedulerSpec.scala:684-697`; A2 rechazo fiable durante el cierre — `:661-682`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-1.

## AC-SCH-010 scheduler.implementation pluggable

*unidad SCH · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** La clave `pekko.scheduler.implementation` (FQCN, por defecto `LightArrayRevolverScheduler`) elige el `Scheduler` del sistema; la clase se instancia por reflexión con un constructor `(Config, LoggingAdapter, ThreadFactory)`. Un scheduler propio debe: ejecutar todas las tareas `TaskRunOnClose` al cerrarse (si es `Closeable`), lanzar `IllegalStateException` si no puede agendar una tarea y, una vez agendada, ejecutarla (posiblemente antes de tiempo en el cierre). Se usa en pruebas para sustituir el tiempo real (`ExplicitlyTriggeredScheduler`).

**Fuente.**
- `actor/src/main/resources/reference.conf:954-969`; `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:467,996,1167-1196`; `actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala:34-58`.
- `testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:44-142`; `actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTime.scala:34-48`.
- Pruebas: `testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala:24-106`; `actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala:38-54` (selecciona la implementación por config).

**Precondiciones.** La clase existe y expone el constructor; el `ThreadFactory` entregado es el del sistema con el nombre `<sistema>-scheduler`.

**Comportamiento.** `ActorSystemImpl.scheduler = createScheduler()` instancia `settings.SchedulerClass` con `Seq(Config -> settings.config, LoggingAdapter -> log, ThreadFactory -> threadFactory.withName(threadFactory.name + "-scheduler"))`. Se registra `stopScheduler()` (que llama `close()` si el scheduler es `Closeable`) al terminar el sistema (`registerOnTermination`) y también si falla la inicialización del proveedor. `ExplicitlyTriggeredScheduler`: no mira el reloj; `timePasses(d)` (tras 100 ms de espera real "para que los dispatchers se vacíen") avanza el tiempo y ejecuta en el hilo llamante, en orden de vencimiento, todas las tareas vencidas (las que reprograma una tarea también); una tarea periódica que lanza no se reprograma; `maxFrequency = 42`; cancelar devuelve true si la tarea seguía pendiente; no implementa `Closeable` ni valida retardos.

**Errores.** Si la clase no existe o no tiene el constructor, `createInstanceFor(...).get` lanza la excepción subyacente sin envolver (p. ej. `ClassNotFoundException`/`NoSuchMethodException`), y el sistema no arranca (`Try(stopScheduler())` y relanzo en `:975-980`).

**Orden / concurrencia.** El scheduler se crea al construir el sistema, antes del proveedor y de los dispatchers (el apagado de un dispatcher depende de él, AC-DSP-001).

**Dependencias.** AC-CFG-016, AC-SCH-009, AC-DSP-001, AC-TK (ManualTime).

**Equivalente Rust propuesto.** `trait Scheduler` + `trait ClosableScheduler`, inyectable con el builder del sistema (`ActorSystemBuilder::scheduler(Arc<dyn Scheduler>)`) y/o un registro nombre -> constructor para la clave `scheduler.implementation` (Q-SCH-2/Q-X-2); un `ManualScheduler` de testkit con `time_passes`. La reflexión del FQCN no existe; el nombre `org.apache.pekko.actor.LightArrayRevolverScheduler` y `org.apache.pekko.testkit.ExplicitlyTriggeredScheduler` pueden registrarse como alias.

**Aserciones.**
- A1 `ExplicitlyTriggeredSchedulerSpec.scala:24-106` (8 pruebas): periódica con retardo 0 (3 ejecuciones en 12 s), con retardo inicial (2), una sola vez, 3 tareas idénticas (las 3 se ejecutan), cancelar (true y deja de ejecutar), cancelar una de varias, cancelar inmediatamente (0 ejecuciones) y avanzar el tiempo en tramos (1, 1, 3). Verifican el scheduler de prueba, no el mecanismo de carga.
- A2 `SchedulerSpec.scala:36-52`: la clase se elige por config (`pekko.scheduler.implementation = ...LightArrayRevolverScheduler` y `tick-duration`), y todo el spec corre contra ella; `ManualTime.scala:34-48` hace lo mismo con `ExplicitlyTriggeredScheduler`.
- A3 el contrato de `close()` ejecutando `TaskRunOnClose` para schedulers propios, el error por clase inexistente y la firma `(Config, LoggingAdapter, ThreadFactory)`: *no afirmada por spec* para implementaciones propias (se ejerce sólo con la rueda, AC-SCH-009). Caracterización propuesta: un scheduler de prueba que registra tareas `RunOnClose` y se cierra al terminar el sistema; nombre desconocido -> error con el nombre.

**Validación prevista.** A1 portado contra el scheduler manual de rakka; A2/A3 caracterización de carga por nombre/inyección.

**Decisiones.** Q-SCH-2 (y Q-X-2).

## AC-SCH-011 Timers / TimerScheduler

- **Contrato observable / Fuente / Condiciones**: ver TSV; `actor/src/main/scala/org/apache/pekko/actor/Timers.scala:31-382` (API pública), `actor/src/main/scala/org/apache/pekko/actor/dungeon/TimerSchedulerImpl.scala:28-161`. Detalles verificados: `startTimer` cancela el timer previo con la misma clave y crea uno con generación `++timerGen` (por instancia de `TimerSchedulerImpl`); modos `FixedRate(initialDelay)`, `FixedDelay(initialDelay)` y `Single`; `startTimerAtFixedRate(key, msg, interval)` usa `initialDelay = interval`; `startPeriodicTimer` = `startTimerAtFixedRate`; el mensaje que viaja es un `InfluenceReceiveTimeoutTimerMsg`/`NotInfluenceReceiveTimeoutTimerMsg(key, generation, owner)` (AC-SCH-012); `cancel(key)` y `cancelAll()` cancelan la tarea del scheduler y borran la entrada (los ya encolados se descartan por clave/generación).
- **Orden**: la intercepción ocurre en `aroundReceive` antes del receive y del stash; `aroundPreRestart` y `aroundPostStop` llaman `cancelAll()` ANTES de `preRestart`/`postStop` (por eso `isTimerActive` es false dentro de ellos).
- **Dependencias**: AC-SCH-001.
- **Equivalente Rust**: `TimerScheduler` en el contexto con mapa clave -> (generación, tarea).
- **Aserciones**: las pruebas viven en `AbstractTimerSpec` (`TimerSpec.scala:189-317`) y se ejecutan DOS veces: con `Timers` (`TimerSpec`, `:169-177`) y con los timers de `FSM` (`FsmTimerSpec`, `:179-187`, ver AC-FSM). A1 `:200-209` timer único: `Tock(1)` y nada más en 100 ms; tras `End`, `GotPostStop(false)`. A2 `:211-222` repetido: tres `Tock(1)` dentro de 4 intervalos - 100 ms. A3 `:224-237` reemplazo: con el actor bloqueado en `SlowThenBump`, el `Tick(1)` ya encolado se descarta al arrancar el timer nuevo y el siguiente mensaje es `Tock(2)`. A4 `:239-248` `cancel`: ninguna entrega en 1 intervalo + 100 ms. A5 `:250-258` reinicio: `GotPreRestart(false)`. A6 `:260-278` (alt 1): `Tick` de la instancia anterior encolado antes del reinicio se descarta; tras el reinicio no llega nada en medio intervalo y luego `Tock(2)` (el contador de arranque de la instancia nueva). A7 `:280-299` (alt 2): tras `Bump` y reinicio el estado vuelve a empezar (`Tock(1)`), descartando el `Tick(2)` viejo. A8 `:301-306` detención: `GotPostStop(false)`. A9 `isTimerActive` tras entregar un timer único, `startPeriodicTimer` como alias y los `Debug` literales de descarte: *no afirmada por spec* (derivadas de `TimerSchedulerImpl.scala:136-162`); caracterización propuesta.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-3.

## AC-SCH-012 Mensajes de timer: AutoReceivedMessage y NotInfluenceReceiveTimeout

*unidad SCH · visibilidad internal-required · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Dos mecanismos distintos sobre el mensaje de usuario programado: (1) si el mensaje es un `AutoReceivedMessage` (p. ej. `PoisonPill`, `Kill`, `Identify`) se entrega por la vía de auto-recibidos del actor (no pasa por el `receive` del usuario); (2) si el mensaje es `NotInfluenceReceiveTimeout`, el sobre interno del timer también lo es y por tanto la llegada del timer no reinicia el `receive-timeout`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Timers.scala:52-69` (`aroundReceive`), `actor/src/main/scala/org/apache/pekko/actor/dungeon/TimerSchedulerImpl.scala:33-40,92-97`, `actor/src/main/scala/org/apache/pekko/actor/dungeon/ReceiveTimeout.scala:52,86`; `Actor.scala:33,81,164`.
- Pruebas: `actor-tests/src/test/scala/org/apache/pekko/actor/TimerSpec.scala:309-315`; `ReceiveTimeoutSpec.scala:191-212`.

**Precondiciones.** El actor mezcla `Timers`.

**Comportamiento.** `startTimer`: si `msg.isInstanceOf[NotInfluenceReceiveTimeout]` el sobre es `NotInfluenceReceiveTimeoutTimerMsg(key, gen, this)` (que extiende `NotInfluenceReceiveTimeout`), si no `InfluenceReceiveTimeoutTimerMsg`. `Timers.aroundReceive` para un `TimerMsg`: `interceptTimerMsg` devuelve `Some(m: AutoReceivedMessage)` -> `autoReceiveMessage(Envelope(m, self, system))`; `Some(m)` -> si el actor es también `Stash`, `currentMessage = currentMessage.copy(message = m)` (porque el stash mira `currentMessage` directamente) y `super.aroundReceive(receive, m)`; `None` -> descarte. `ReceiveTimeout.scala:42,72`: el reinicio del temporizador de inactividad se omite cuando el mensaje del sobre es `NotInfluenceReceiveTimeout`.

**Errores.** Ninguno propio.

**Orden / concurrencia.** El sobre se evalúa en el momento de la recepción del mensaje del timer (turno normal del mailbox).

**Dependencias.** AC-SCH-011, AC-SCH-013, AC-CELL-017 (receive-timeout).

**Equivalente Rust propuesto.** Mensaje interno `TimerMsg { key, generation, owner, influences_receive_timeout }` y un predicado `is_auto_received(&dyn Any)` (Q-SCH-3, Q-X-3). Se conserva el efecto sobre el receive-timeout y la vía de auto-recibidos.

**Aserciones.**
- A1 `TimerSpec.scala:308-314` ("handle AutoReceivedMessages automatically"): con `startSingleTimer("A", PoisonPill, interval)` (para `Timers` y para FSM) el actor termina (`expectTerminated`).
- A2 `ReceiveTimeoutSpec.scala:191-212` ("get timeout while receiving NotInfluenceReceiveTimeout messages scheduled with Timers"): con timers cada 100 ms para `TransparentTick` (un `NotInfluenceReceiveTimeout`) e `Identify(None)` (auto-recibido y `NotInfluenceReceiveTimeout`) y `setReceiveTimeout(1 s)`, el `ReceiveTimeout` se produce igualmente y `count > 0` (los `TransparentTick` se entregaron).
- A3 la rama `Stash` (`currentMessage.copy`): cubierta por `TimerSpec.scala:344-353` (AC-SCH-013).

**Validación prevista.** A1/A2 portados; A3 en AC-SCH-013.

**Decisiones.** Q-SCH-3.

## AC-SCH-013 Timers combinados con Stash
- **Contrato observable / Fuente / Condiciones**: ver TSV; `Timers.scala:57-61`.
- **Orden**: el reemplazo del `currentMessage` ocurre antes de `super.aroundReceive`.
- **Dependencias**: AC-SCH-011, AC-MBX-016.
- **Equivalente Rust**: sustituir el mensaje actual antes de invocar el receive.
- **Aserciones**: A1 `TimerSpec.scala:344-355`.
- **Oráculo**: no aplica.
- **Decisiones**: Q-SCH-3.

## AC-SCH-014 scheduler.shutdown-timeout

*unidad SCH · visibilidad public · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** `close()` del scheduler por defecto espera como máximo `pekko.scheduler.shutdown-timeout` (5 s) a que el hilo del timer reconozca la parada y devuelva las tareas pendientes. Si vence, `close()` falla y las tareas aún encoladas no se ejecutan (el apagado del sistema continúa sin ellas).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:89,113,188-210,255-265,364-369`; `actor/src/main/resources/reference.conf:934-975`.

**Precondiciones.** Scheduler no cerrado (un segundo `close()` obtiene `Future.successful(Nil)` y no ejecuta nada).

**Comportamiento.** `close()`: `Await.result(stop(), getShutdownTimeout)`. `stop()`: `CAS(stopped, null -> Promise)`; devuelve el futuro (o `Future.successful(Nil)` si ya estaba parado). El hilo del timer comprueba `stopped` tras cada iteración (cada tick como máximo) y completa la promesa con las tareas pendientes (`clearAll`: rueda + cola de entrada). Luego `close()` ejecuta las `TaskRunOnClose` (directas o dentro de un `TaskHolder`) y no ejecuta el resto. Por eso un `tick-duration` alto alarga el cierre (comentario de `reference.conf`).

**Errores.** Si el hilo no responde dentro del timeout, `Await.result` lanza `java.util.concurrent.TimeoutException`; `close()` no la captura. Un `InterruptedException` en una tarea `RunOnClose` se relanza; un `SchedulerException` se ignora; otro `NonFatal` se loguea como `log.error(e, "exception while executing timer task")`.

**Orden / concurrencia.** El timeout aplica sólo a la espera del hilo; las tareas `RunOnClose` se ejecutan en el hilo que llama a `close()`.

**Dependencias.** AC-SCH-009, AC-SCH-008.3.

**Equivalente Rust propuesto.** Parámetro `shutdown_timeout` de `Scheduler::close()` que espera al hilo del timer con `join_timeout`/canal con timeout y devuelve `Err(Timeout)` si vence; el hilo comprueba la señal de parada en cada tick.

**Aserciones.**
- A1 el vencimiento del timeout: *no afirmada por spec* (`getShutdownTimeout` es sobreescribible: `SchedulerSpec.scala:813` lo fija en 10 s dilatados para las pruebas determinísticas, pero ningún test provoca el vencimiento). Caracterización propuesta: un `Waiter` que no devuelve y un timeout corto -> `close()` falla con `Timeout`.
- A2 comportamiento de `close()` con tareas `TaskRunOnClose`: ver AC-SCH-009 A1/A2.

**Validación prevista.** Caracterización con el `Waiter` manual de AC-SCH-008.3.

**Decisiones.** Q-SCH-1 (rueda propia).

## AC-SCH-015 Recuperación del hilo del timer (LARS)

*unidad SCH · visibilidad internal · analysis=extracted (cierre documental 2026-10-31)*

**Contrato observable.** Si el hilo del timer muere por un `Throwable` inesperado, el scheduler lo registra y, si no está parado, arranca otro hilo que continúa con la MISMA rueda; si no puede arrancar uno, se declara parado.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:307-329`.

**Precondiciones.** El hilo del timer ejecuta `nextTick()`.

**Comportamiento.** En `run()`: `catch t: Throwable` -> `log.error(t, "exception on LARS’ timer thread")` (apóstrofo tipográfico U+2019). Si `stopped.get == null`: nuevo hilo con el mismo `Runnable` (la rueda, `tick` y `totalTick` conservan su estado), `log.info("starting new LARS thread")`, `thread.start()`; si falla: `log.error(e, "LARS cannot start new thread, ship’s going down!")`, `stopped.set(Promise.successful(Nil))` y `clearAll()`; `timerThread = thread` en cualquier caso. Si ya había una parada pedida: `assert(stopped.compareAndSet(p, Promise.successful(Nil)), "Stop signal violated in LARS")` y `p.success(clearAll())`. Al final se relanza `t` (el hilo viejo muere). Un fallo no fatal de una tarea no llega aquí: `executeTask` lo captura y lo envía a `reportFailure`.

**Errores.** Los mensajes de log literales de arriba; `AssertionError("assertion failed: Stop signal violated in LARS")` si la parada se pierde.

**Orden / concurrencia.** Tras un fallo irrecuperable, `schedule` posterior lanza `SchedulerException("cannot enqueue after timer shutdown")` (y `IllegalStateException` en `scheduleOnce`).

**Dependencias.** AC-SCH-008.3, AC-SCH-014.

**Equivalente Rust propuesto.** El bucle del timer corre bajo `catch_unwind`; un pánico se registra con el mensaje equivalente y se reinicia el bucle con el mismo estado de la rueda, o se declara el scheduler parado si el reinicio falla (Q-X-5: qué es un fallo recuperable). Como `executeTask` captura los fallos de las tareas, el camino sólo se ejerce por errores internos o del propio `Executor.execute`.

**Aserciones.** *No afirmada por spec* (ninguna prueba provoca la muerte del hilo del timer). Caracterización propuesta: un `Executor` cuyo `execute` entra en pánico no mata al scheduler (se reporta) y un `Waiter` que entra en pánico provoca el reinicio con el estado de la rueda conservado y el mensaje de error esperado.

**Validación prevista.** Caracterización con ganchos de fallo inyectados (`Waiter` y `Clock`).

**Decisiones.** Q-X-5.

---

# Secciones finales

## Descomposiciones
- AC-DSP-016 -> AC-DSP-016.1 (`Dispatcher.execute`/`TaskInvocation`/inhabitants/`reportFailure`) y AC-DSP-016.2 (`BatchingExecutor`, `Batchable`, reenvío ante bloqueo). Motivo: comportamientos independientes con pruebas y decisiones distintas (Q-DSP-7 sólo afecta a .2).
- AC-DSP-017 -> AC-DSP-017.1 (`ExecutionContexts`, `SameThreadExecutionContext`/parasitic) y AC-DSP-017.2 (`Futures`, `CompletionStages`, puentes `japi`, API Java). Motivo: la primera es un `Executor` en línea (necesario), la segunda es API Java de futuros (candidata a exclusión, Q-DSP-3).
- AC-SCH-008 -> AC-SCH-008.1 (validación de configuración), AC-SCH-008.2 (redondeo y límites de retardo) y AC-SCH-008.3 (mecánica de la rueda y relojes que dan la vuelta). Motivo: tres grupos de aserciones y pruebas independientes (`SchedulerSpec` configuración, `roundUp`/rechazos, `withScheduler`/`Driver`).
- Los padres AC-DSP-016, AC-DSP-017 y AC-SCH-008 conservan un bloque que sólo remite a sus hijos (el coordinador decide si los reemplaza en el TSV). Una referencia a un padre significa todos sus hijos.
- Candidatas descartadas (se dejan como una fila): AC-MBX-016 (los cinco comportamientos de `Stash` comparten estado y pruebas), AC-DSP-004 (lookup, alias y `type` son una sola función `lookup`), AC-SCH-011 (las operaciones de `TimerScheduler` comparten las mismas pruebas `AbstractTimerSpec`), AC-DSP-008 (las claves de fork-join tienen una sola prueba pura para `minimum-runnable` y el resto es derivado del código).

## Omisiones agregadas
- AC-MBX-020: `reference.conf:682-683,728-733` y `event/LoggerMailbox.scala` (`LoggerMessageQueueSemantics`, `logger-queue`).
- AC-MBX-021: `Mailboxes.scala:54-72` (deadLetterMailbox).
- AC-MBX-022: `Mailbox.scala:713-714,760-761,809-810,854-857`; `Mailboxes.scala:173-178,257-266` (validaciones y warnings).
- AC-MBX-023: `Mailboxes.scala:295-331` (stash-capacity por dispatcher/mailbox).
- AC-DSP-020: `AbstractDispatcher.scala:34-53` (Envelope, InvalidMessageException).
- AC-DSP-021: `Dispatchers.scala:314-374` (BalancingDispatcherConfigurator).
- AC-DSP-022: `Dispatchers.scala:77-95` (getConfig / alias vacío).
- AC-DSP-023 (cierre 2026-10-31): `AbstractDispatcher.scala:277-293` (`suspend`/`resume` del dispatcher por actor).
- AC-SCH-014: `reference.conf:923-927` (`scheduler.shutdown-timeout`).
- AC-SCH-015: `LightArrayRevolverScheduler.scala:288-310` (recuperación del hilo).

## Contradicciones halladas y corregidas
- AC-MBX-013: el inventario decía `ActorInitializationException` por requisito incumplido; el test afirma `ConfigurationException` (`ActorMailboxSpec.scala:316-319`) y el código lanza `IllegalArgumentException` envuelta (`ActorRefProvider.scala:715-718`). `ActorInitializationException` "requires mailbox type" existe en otra rama (`actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:92-99`).
- AC-DSP-004: el inventario incluía `type=BalancingDispatcher` como válido; `Dispatchers.scala:268-273` lo rechaza salvo por FQCN.
- AC-DSP-003: Scaladoc (`Dispatcher.scala:37-40`) vs código (`Mailbox.scala:264`) para throughput<=0 (el código es lo observable; ver Q-DSP-5).
- AC-DSP-008: el comentario de `reference.conf:497` ("-1 (default)") contradice la línea efectiva `minimum-runnable = 1` (`:507`) y el comentario `:494-495` ("In Pekko 1.x, the default value is 1"); además `ForkJoinExecutorConfiguratorSpec.scala:153-162` ("auto-scale the default (minimum-runnable not set) on JDK 25+") lee la configuración con los valores por defecto de `reference.conf` (`system.dispatchers.config(id)`), por lo que con `minimum-runnable = 1` esperaría fallar en JDK 25+ (hoy `pending` en JDK < 25): inconsistencia no resoluble sin ejecutar con JDK 25 (Q-DSP-6).
- AC-DSP-008/014: `reference.conf:509-510` afirma que `virtualize = on` sin soporte de hilos virtuales lanza una excepción; el código lo ignora (`ForkJoinExecutorConfigurator.scala:105`, `virtualize && JavaVersion.majorVersion >= 21`).
- AC-DSP-011: el inventario omitía `throughput=Int.MaxValue` y el warning de ThreadPoolExecutor.
- AC-DSP-009 (omisión): la política de rechazo por defecto `SaneRejectedExecutionHandler` (el emisor ejecuta la tarea) no estaba en el contrato.
- AC-MBX-003: la prueba citada `ActorRefSpec.scala:422` no afirma dead letters; se sustituye por `ActorWithBoundedStashSpec` y `DeathWatchSpec.scala:145`.
- AC-MBX-002: el texto del `Error` al reenviar mensajes de sistema termina con `e.getMessage`, no con el mensaje de sistema (`Mailbox.scala:320-325`).
- AC-MBX-008: `size()` de la cola acotada es una aproximación (no exacta) y la cola declara `MultipleConsumerSemantics`.
- AC-MBX-018: el inventario decía "sin test directo en actor-tests"; `StashJavaAPI.java:41-54` y `AbstractFSMWithStashActorTest.java` sí ejercen las variantes.
- AC-MBX-012: `BoundedControlAwareMailbox` no valida argumentos y con `pushTimeOut <= 0` descarta de inmediato (los demás bounded esperan indefinidamente con timeout negativo).
- AC-MBX-016: el Scaladoc de `Stash` (`unstash`/`unstashAll`) menciona `MessageQueueAppendFailedException`, que el código nunca lanza (el mailbox acotado publica `DeadLetter`).
- AC-SCH-001: el texto "3 tareas a 300 ms" era "2 tareas a 300 ms sobre un latch de 3" (`SchedulerSpec.scala:97-116`).
- AC-SCH-004: el inventario decía "alias deprecado de fixed rate" y proponía excluir; en realidad es el método abstracto sobre el que se apoya `scheduleAtFixedRate`.
- AC-SCH-006: el reenvío de `cancelled()` sólo está afirmado para `scheduleWithFixedDelay` (`SchedulerSpec.scala:348-363`), no para tasa fija.
- AC-SCH-012: el inventario mezclaba `PoisonPill/Kill` con `NotInfluenceReceiveTimeout`; son dos mecanismos distintos (`Timers.scala:55-56` vs `TimerSchedulerImpl.scala:95-100`).
- Ficheros 'Q: excluir' del inventario viejo (AC-DSP-013/014/017, AC-SCH-004) se sustituyen por decisiones propuestas; las filas siguen en alcance.

## Filas que siguen listed (motivo)
Ninguna del grupo MBX/DSP/SCH tras el cierre documental del 2026-10-31: las 26 filas que estaban `listed` se pasan a `extracted` porque se leyeron la implementación y los cuerpos de los specs citados (o se verificó que no hay spec y se propone una caracterización marcada como tal). Las dependencias abiertas se recogen en las decisiones y bloquean la IMPLEMENTACIÓN (`impl`), no el contrato: Q-MBX-1 (AC-MBX-007/009/010/011/012/016 con timeout > 0), Q-DSP-1/Q-X-4 (executors), Q-DSP-7 (AC-DSP-016.2), Q-DSP-3 (AC-DSP-017.x), Q-SCH-1 (AC-SCH-008.x). Fronteras explícitas con otras unidades: Logging/StandardOutLogger (EVT) para AC-MBX-020, familia `SystemMessage` (AC-CELL-014) para AC-MBX-003, `FSM` timers (AC-FSM) para `FsmTimerSpec`, `CallingThreadDispatcher` (AC-TK-006.2) para la condición `messageQueue ne null` de AC-MBX-003.

## Cobertura
Archivos del árbol cubiertos por fila: `dispatch/Mailbox.scala` (AC-MBX-001..012,015,019,021,022), `dispatch/Mailboxes.scala` (AC-MBX-003,013,014,021,022,023), `actor/Stash.scala` (AC-MBX-016..018), `dispatch/AbstractDispatcher.scala` (AC-DSP-001,006,010,014,016.1,019,020,023), `dispatch/Dispatcher.scala` (AC-DSP-002,003; `PriorityGenerator` AC-MBX-009), `dispatch/Dispatchers.scala` (AC-DSP-004..007,011,021,022), `dispatch/PinnedDispatcher.scala` (AC-DSP-011), `dispatch/BalancingDispatcher.scala` (AC-DSP-012), `dispatch/ForkJoinExecutorConfigurator.scala` (AC-DSP-008,019), `dispatch/ThreadPoolBuilder.scala` (AC-DSP-009,015), `dispatch/VirtualThreadSupport.scala`/`VirtualizedExecutorService.scala` (AC-DSP-014), `dispatch/affinity/AffinityPool.scala` y `java/.../affinity/OnSpinWait.java` (AC-DSP-013), `dispatch/BatchingExecutor.scala` y `ScalaBatchable` (AC-DSP-016.2), `dispatch/Future.scala`/`CompletionStages.scala` (AC-DSP-017.x), `dispatch/CachingConfig.scala`/`ForkJoinPoolConstants.scala`/`PekkoJdk9ForkJoinPool.scala` (AC-DSP-019), `util/SerializedSuspendableExecutionContext.scala` (AC-DSP-018), `actor/Scheduler.scala` (AC-SCH-001..007), `actor/LightArrayRevolverScheduler.scala` (AC-SCH-008.x,009,014,015), `actor/Timers.scala` y `actor/dungeon/TimerSchedulerImpl.scala` (AC-SCH-011..013), `event/LoggerMailbox.scala` (AC-MBX-020), `java/.../AbstractNodeQueue.java` y `AbstractBoundedNodeQueue.java` (AC-MBX-006/008). Sin comportamiento propio: `java/.../AbstractMessageDispatcher.java` y `AbstractMailbox.java` (offsets `Unsafe` de los campos de `MessageDispatcher` y `Mailbox`). `reference.conf` leído: `default-dispatcher` (todas sus claves), `internal-dispatcher`, `default-blocking-io-dispatcher`, `default-mailbox`, `mailbox.*`, `scheduler.*`.

## Revisión de exhaustividad (cierre documental 2026-10-31)

Criterio: se contrastó el árbol de `dispatch/`, `dispatch/affinity/`, `event/LoggerMailbox.scala`, `actor/Scheduler.scala`, `LightArrayRevolverScheduler.scala`, `Timers.scala`, `actor/src/main/scala/org/apache/pekko/actor/dungeon/TimerSchedulerImpl.scala`, `util/SerializedSuspendableExecutionContext.scala`, las secciones de `reference.conf` y los specs de `actor-tests` (`actor/dispatch/*`, `dispatch/*`, `ActorMailboxSpec`, `ActorWithStashSpec`, `ActorWithBoundedStashSpec`, `ActorSystemDispatcherSpec`, `SchedulerSpec`, `TimerSpec`, `ReceiveTimeoutSpec`) con las filas del grupo.

### Clasificación

| Elemento | Clasificación | Filas / frontera |
|---|---|---|
| Estado, ejecución y limpieza del mailbox (`Mailbox`, `DefaultSystemMessageQueue`) | Cubierto por contratos concretos | AC-MBX-001..003, AC-DSP-023 |
| `MessageQueue`/`MailboxType`/semánticas/selección | Cubierto por contratos concretos | AC-MBX-004, 013..015 |
| Colas (unbounded, bounded, node, priority, stable, deque, control-aware) | Cubierto por contratos concretos | AC-MBX-005..012 |
| `Stash`, `UnboundedStash`, `UnrestrictedStash`, `StashFactory` | Cubierto por contratos concretos | AC-MBX-016..018, 023 |
| `LoggerMailbox`/`LoggerMailboxType` | Cubierto por contrato concreto; frontera con EVT para el destino `StandardOutLogger` | AC-MBX-020 |
| `deadLetterMailbox`, `NoMessage` (centinela) | Cubierto por contratos concretos | AC-MBX-003, 021 |
| `SystemMessage`, `LatestFirst/EarliestFirstSystemMessageList` | Fuera de estos módulos con frontera explícita (grupo CELL: AC-CELL-014); se usa sólo vía `systemEnqueue/systemDrain` | AC-MBX-003 |
| `CallingThreadDispatcher` (testkit) | Fuera de estos módulos con frontera explícita (AC-TK-006.2) | AC-MBX-003 (condición `messageQueue ne null`) |
| `MessageDispatcher` (attach/detach/shutdown/execute), `Dispatcher`, `PinnedDispatcher`, `BalancingDispatcher` | Cubierto por contratos concretos | AC-DSP-001..003, 011, 012, 016.1, 021, 023 |
| `Dispatchers` (lookup, alias, `getConfig`, `registerConfigurator`, `type`) | Cubierto por contratos concretos | AC-DSP-004..007, 022 |
| `Envelope`, `TaskInvocation` | Cubierto por contratos concretos | AC-DSP-020, 016.1 |
| `ExecutorServiceConfigurator`, `DefaultExecutorServiceConfigurator`, fork-join, thread-pool, affinity, virtual-thread | Cubierto por contratos concretos | AC-DSP-006, 008..010, 013, 014 |
| `MonitorableThreadFactory`, `MonitorableCarrierThreadFactory` | Cubierto (nombres de hilo) / detalle JVM (carrier) | AC-DSP-015, 014 |
| `PekkoForkJoinPool`/`PekkoForkJoinTask`/`PekkoJdk9ForkJoinPool`/`ForkJoinPoolConstants` | Detalle interno sin contrato independiente (envoltorios del `ForkJoinPool` de la JDK; los efectos observables están en AC-DSP-008) | AC-DSP-019 |
| `CachingConfig` | Detalle interno sin contrato independiente (sólo rendimiento; no altera resultados) | AC-DSP-019 |
| `VirtualThreadSupport` (reflexión sobre `Thread.ofVirtual`, `--add-opens`) | Detalle interno de la JVM; el efecto observable está en AC-DSP-014 | AC-DSP-014 |
| `ScalaBatchable` (tres copias por versión de Scala) | Duplicado o fachada equivalente: misma regla `Batchable` en 2.12/2.13/3 | AC-DSP-016.2 |
| `SameThreadExecutionContext` (tres copias) | Duplicado o fachada equivalente (en 2.13+ es `ExecutionContext.parasitic`) | AC-DSP-017.1 |
| `BatchingExecutor`, `Batchable` | Cubierto por contrato concreto | AC-DSP-016.2 |
| `ExecutionContexts`, `Futures`, `CompletionStages`, `japi` | Cubierto por contratos concretos (API Java, candidata a exclusión: Q-DSP-3) | AC-DSP-017.1/.2 |
| `SerializedSuspendableExecutionContext` | Cubierto por contrato concreto | AC-DSP-018 |
| `AbstractMessageDispatcher.java`, `AbstractMailbox.java` | Detalle interno sin contrato independiente (sólo offsets `Unsafe` de campos) | n/a |
| `AbstractNodeQueue.java`, `AbstractBoundedNodeQueue.java` | Cubierto por contratos concretos (colas MPSC) | AC-MBX-006, 008 |
| `Scheduler`, `Cancellable`, `SchedulerTask`, `TaskRunOnClose`, `AtomicCancellable` | Cubierto por contratos concretos | AC-SCH-001..007, 009 |
| `LightArrayRevolverScheduler` | Cubierto por contratos concretos | AC-SCH-008.1..3, 009, 014, 015 |
| `ExplicitlyTriggeredScheduler` (testkit) | Cubierto en la parte de scheduler pluggable; frontera con TK/ManualTime | AC-SCH-004, 010 |
| `Timers`, `TimerScheduler`, `TimerSchedulerImpl` | Cubierto por contratos concretos | AC-SCH-011..013 |
| Timers de `FSM` (`FsmTimerSpec`) | Fuera de estos módulos con frontera explícita (grupo FSM); comparte `AbstractTimerSpec` | AC-SCH-011 |
| `serialize-messages` en `ActorCell.sendMessage` (`serializeAndDeserialize`) | Fuera de estos módulos con frontera explícita (CELL/SER); no es parte del contrato de dispatch | n/a |
| Tests de rendimiento/JMH (`bench-jmh`), `DispatcherDocSpec` (docs) | Fuera de estos módulos (no son especificaciones de comportamiento) | n/a |

### Claves de configuración y su efecto documentado

| Clave | Efecto documentado en |
|---|---|
| `pekko.actor.default-dispatcher.type` | AC-DSP-004 |
| `...executor`, `default-executor.fallback` | AC-DSP-010, AC-DSP-006 |
| `...fork-join-executor.{parallelism-min,factor,max,task-peeking-mode,maximum-pool-size,minimum-runnable,virtualize}` | AC-DSP-008 (y AC-DSP-014 para `virtualize`) |
| `...thread-pool-executor.{keep-alive-time,fixed-pool-size,core-pool-size-*,max-pool-size-*,task-queue-size,task-queue-type,allow-core-timeout}` | AC-DSP-009 |
| `...affinity-pool-executor.{parallelism-*,task-queue-size,rejection-handler,idle-cpu-level,queue-selector,fair-work-distribution.threshold}` | AC-DSP-013 |
| `...virtual-thread-executor.fallback` | AC-DSP-010, AC-DSP-014 |
| `...shutdown-timeout` | AC-DSP-001 |
| `...throughput`, `...throughput-deadline-time` | AC-DSP-003, AC-MBX-002 |
| `...attempt-teamwork` | AC-DSP-012 |
| `...mailbox-requirement` | AC-MBX-013, AC-DSP-021 |
| `pekko.actor.internal-dispatcher`, `default-blocking-io-dispatcher` | AC-DSP-006 |
| `pekko.actor.default-mailbox.{mailbox-type,mailbox-capacity,mailbox-push-timeout-time,stash-capacity}` | AC-MBX-004, 007, 014, 022, 023 |
| `pekko.actor.mailbox.requirements` y secciones `unbounded-queue-based` ... `logger-queue` | AC-MBX-013..015, 020 |
| `pekko.scheduler.{tick-duration,error-on-tick-duration-verification-failed,ticks-per-wheel}` | AC-SCH-008.1 |
| `pekko.scheduler.implementation` | AC-SCH-010 |
| `pekko.scheduler.shutdown-timeout` | AC-SCH-014 |

### Mapa de executors de JVM: efecto observable y representación propuesta en Rust

| Executor JVM | Efecto observable que debe preservarse | Límites de configuración | Representación Rust propuesta | Diferencias |
|---|---|---|---|---|
| `ForkJoinPool` (`fork-join-executor`, FIFO/LIFO) | Paralelismo máximo `parallelism`; robo de trabajo entre hilos; modo `asyncMode` (cola FIFO por hilo) o LIFO; nombres `<sistema>-<id>-N`; compensación de hilos bloqueados vía `BlockContext` (`blocking {}`); `atFullThrottle` | `parallelism = min(max(ceil(cpus*factor), min), max)`; `maximum-pool-size`; `minimum-runnable` (0 = sin compensación) | Pool de `std::thread` con cola global + deques por worker con robo (p. ej. `crossbeam-deque`), FIFO por defecto; parámetros aceptados y validados (Q-DSP-1) | No hay `BlockContext`: la compensación automática ante bloqueo no existe (Q-DSP-7); `minimum-runnable`/`maximum-pool-size` se validan pero sin efecto o se documentan como tales |
| `ThreadPoolExecutor` (`thread-pool-executor`, `PinnedDispatcher`) | Tamaño core/max; cola enlazada/array acotada o no; keep-alive; "caller runs" si está saturado; `PinnedDispatcher` = un hilo por actor | `fixed-pool-size` o `scaledPoolSize`; `task-queue-size`/`task-queue-type`; `allow-core-timeout` | Pool de `std::thread` con cola `mpmc` (acotada o no) y política SaneRejected; hilo dedicado para Pinned | Ninguna relevante |
| `AffinityPool` | Afinidad mailbox->hilo, cola acotada por hilo, rechazo al llenarse, selector `FairDistributionHashCache`, espera escalonada | `parallelism-*`, `task-queue-size` 512, `idle-cpu-level` 1..10, `threshold` 0..2048 | Pool propio con cola acotada MPSC por worker y `QueueSelector` portado (AC-DSP-013) | `hashCode` de identidad sustituido por un id estable; distribución por hash no reproducible bit a bit |
| Hilos virtuales (`virtual-thread-executor`, `virtualize`) | Una tarea ligera por turno; bloquear no consume un worker; sin límite fijo | ninguna (solo propiedades del sistema de la JDK) | Tareas async del runtime (`tokio::spawn`) | Nombres `...-virtual-thread-N` y `atFullThrottle` real no se preservan |
| Executor externo (`ExecutionContext` de `BootstrapSetup`) | Reemplaza `default-executor`; nunca se apaga desde el dispatcher | n/a | `Arc<dyn Executor>` inyectable (AC-DSP-006) | Nombres de hilo y dimensionado los decide el usuario |
| `BatchingExecutor` | No-inline y serie de las tareas anidadas; reenvío ante bloqueo | n/a | Lote en `thread_local!` (opcional) | Reenvío ante bloqueo no automático (Q-DSP-7) |
