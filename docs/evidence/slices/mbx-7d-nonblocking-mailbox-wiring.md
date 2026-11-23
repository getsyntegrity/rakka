# MBX PR 7d: `NonBlockingBoundedMailbox` cableado en el runtime (AC-MBX-008)

Implementa la alternativa A de PROP-7B-1, autorizada por el Owner (continuación de `mbx-7c-nonblocking-mailbox-runtime-wiring.md`, PR #335). No implementa Q-MBX-1 ni Q-MBX-2 completos, ni lectura de `mailbox-capacity` desde `Config`.

## Cambios de API

- `MessageQueue::enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool` (antes `enqueue(&self, envelope)`): `true` si se almacenó. Una cola que rechaza ya envió el `DeadLetter` a `deadLetters` dirigido a `receiver`.
- `MessageQueue::clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue)` (Pekko `cleanUp(owner, deadLetters)`).
- `NonBlockingBoundedMailbox::create(dead_letters)` (antes `create(owner, dead_letters)`): la cola ya no guarda al dueño, así que no hay ciclo celda → cola → dueño.
- Nuevo, público: `Props::with_mailbox(NonBlockingBoundedMailbox)`; sólo acepta ese tipo hasta que Q-MBX-2 defina `Mailboxes`.
- Interno: `mailbox_close::Enqueued { awaits_consumption, dead }` reemplaza el `Vec<Envelope>` de `enqueue_unless_closed`.

## Punto de integración

`ActorCell::build` crea la cola con `Props::new_message_queue(&system.dead_letters)` (la celda de reemplazo de `new_with_failure` sigue ilimitada). `enqueue_user` pasa `self_ref()` como receptor y sólo llama a `register_for_execution` si `awaits_consumption`; `finish_terminate` limpia hacia el actor.

Diferencia consciente con Pekko: `Dispatcher.dispatch` registra un turno aunque la cola rechace; aquí no (contrato pedido por el Owner). Un turno vacío no es observable salvo por `scheduled`, y el estado `scheduled` sólo lo mira el oráculo de estado del mailbox, que no se ve afectado (replay sin divergencias).

## Pruebas

- RED (antes de cablear la celda): `cargo test --test classic_nonblocking_bounded_mailbox` → 3 fallas por `timed out waiting for ... dead-lettered` (la cola seguía ilimitada).
- GREEN: las 3 pasan; ver abajo.
- `tests/classic_nonblocking_bounded_mailbox.rs` (API pública, actores reales): cola llena rechaza al instante y el `DeadLetter` lleva mensaje, emisor y receptor; capacidad 0 rechaza todo; 4 productores concurrentes sin pérdida ni duplicación.
- `src/actor/nonblocking_mailbox_tests.rs` (interno): un rechazo no genera turno (ejecutor que cuenta), uno aceptado sí; `stop` con cola llena envía lo encolado a dead letters dirigido al actor; el reinicio conserva cola y cota; un actor terminado se libera (sin ciclo de `Arc`).
- La prueba de `stop` por la API pública se descartó: `system.stop` pasa por el guardián y no es determinista.

## Gates (sobre el HEAD de la rama)

| Gate | Resultado |
|---|---|
| `cargo test --workspace` | verde |
| `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` | limpios |
| `cargo deny check licenses` | ok |
| Replay completo | 2962/2962 aplicables + 1 no aplicable, 0 divergencias |
| Cobertura (`tools/parity/coverage.sh`) | 98,20 % líneas del crate; las líneas modificadas de producción están cubiertas (sin cubrir sólo dobles de prueba preexistentes) |
| Mutantes | 16/16 DETECTED, control CLEAN: 6 nuevos (`nonblocking-cell-*`, `nonblocking-queue-reports-a-refused-message-as-stored`, `enqueue-unless-closed-ignores-the-queue-answer`, `props-ignores-the-selected-mailbox`) y los 10 existentes que tocan estos archivos; 5 patrones actualizados a la nueva firma |
| `tools/package/check.py` | `failures=0` |

No se usó el runner `tools/gates/gates.py` porque necesita el submódulo `upstream/pekko` inicializado y el clon falló en este entorno; se ejecutó cada gate por separado con los mismos comandos. No se ejecutó la batería global de mutantes.

## Contratos JVM

Los 10 casos del oráculo (`cases/oracle-only/nonblocking-bounded-mailbox.golden`) siguen pasando con la nueva firma (`nb_*_matches_the_oracle`). No se modificó ningún golden ni oráculo. El cableado con actores reales no tiene gemelo grabado propio.

## Riesgos y pendientes

- `Props::with_mailbox` es transitoria (Q-MBX-2); `NB-CF-02` (config) queda para el PR 10.
- `BoundedMailbox` bloqueante (Q-MBX-1) y PROP-7A-1 no se tocan.
- Tamaño: ~110 líneas de producción; el resto son pruebas, mutantes y esta evidencia (~800 líneas en total), por encima del objetivo; el cambio de trait no se puede partir sin dejar la rama rota.

## Validación final (veredictos para el Owner)

### 1. Scheduling de un mensaje rechazado: **diferencia observable sólo con un ejecutor instrumentado; sin efecto semántico**

Pekko 2.0.0 (`Dispatcher.scala:80-84,146-171`, `Mailbox.scala:255-272`): `dispatch` hace `mbox.enqueue(receiver.self, invocation)` y luego
`registerForExecution(mbox, hasMessageHint = true, …)` **sin mirar el resultado de `enqueue`**. Con el mailbox abierto y ocioso,
`canBeScheduledForExecution` es `true` y se envía un turno al ejecutor; ese turno no halla mensajes (`processMailbox` no hace nada), hace
`setAsIdle` y no se re-registra. Con el mailbox suspendido (sólo cuentan los hints de sistema) o cerrado Pekko no envía turno; en Rakka el cerrado no agenda (`enqueue_unless_closed`) y el suspendido sólo se cubre con las pruebas de suspensión preexistentes, no con este diff.

Sonda JVM independiente (`mbx-7d-scheduling-probe.java`, Pekko 2.0.0, JDK 17, `NonBlockingBoundedMailbox` de capacidad 0, ejecutor de un hilo que cuenta `execute`;
3 corridas): tres `tell` a un actor ocioso envían **2 turnos** al ejecutor (el conteo es 1-3 según el instante; nunca 0), los tres mensajes van a dead letters y nada se recibe.
Con un ejecutor que rechaza dos veces, el `tell` de un mensaje que igualmente se dead-lettera consume ambas ocurrencias del rechazo (`rejectionsLeft=0`): Pekko
intenta ejecutar y, al fallar dos veces, publica el `Error` "registerForExecution was rejected twice!" aunque el mensaje no se almacenó.
Gemelo Rust: `a_refused_message_does_not_start_a_consumption_turn` (mismo escenario, cuenta los envíos al `Executor` de `rakka-actor`): **0 turnos**.

Efecto sobre la entrega de otros mensajes: ninguno. Dentro de un actor los turnos son seriales (`setAsScheduled` es la única puerta) y el turno vacío no consume mensajes,
no cambia el `throughput` ni la terminación (`Terminate` y `Resume` se registran con su propio hint de sistema) y, si llega un mensaje aceptado mientras corre,
el `registerForExecution` de fin de turno lo recoge porque `hasMessages` es verdadero. Lo que cambia es el estado `scheduled` durante ese instante, el uso del ejecutor y, si el ejecutor rechaza dos veces, el `Error` publicado:
`registerForExecution` hace `setAsIdle` y relanza `RejectedExecutionException` (`Dispatcher.scala:160-167`); en la sonda el `tell` externo **volvió** sin excepción
(el relanzamiento lo absorbe `sendMessage`), de modo que lo visible es el `Error` y la cuenta de `execute`, no una excepción al emisor.

Observabilidad en Rakka: el `Executor` es `pub(crate)`; ninguna API pública lo expone, de modo que la diferencia no es alcanzable por un usuario de Rakka. En la JVM
sí lo es mediante un `ExecutorServiceConfigurator` propio (como hace el oráculo `MBX-RJ`). Ningún caso grabado del oráculo la ejerce hoy (no hay caso con una cola que rechaza
y un ejecutor instrumentado), por lo que el replay no puede distinguirla.

**No se cambió producción.** El contrato "no se introduce un turno de consumo por un mensaje rechazado" lo pidió el Owner para esta tarea. Opciones:
(a) **mantener** y registrar la diferencia como decisión propuesta (recomendada: es interna, sin efecto de entrega); (b) **alinear con Pekko**:
`enqueue_user` registraría un turno siempre que el mailbox esté abierto (`awaits_consumption` pasaría a "no cerrado"), con el costo de un turno vacío por rechazo
y de que el rechazo del ejecutor pueda ocurrir tras un rechazo de la cola. Si el Owner elige (b) hay que añadir antes un caso JVM grabado equivalente a la sonda.
No se registró un `Q-*` (sólo el Owner decide si la diferencia se acepta).

### 2. `new_with_failure`: **correcto, igual que Pekko**

`RepointableActorRef.point(catchFailures)` (`RepointableActorRef.scala:112-121`) crea, si `newCell` falla, `new ActorCell(system, this, props, safeDispatcher, supervisor).initWithFailure(ex)`
con el dispatcher global por defecto, y `initWithFailure` (`Dispatch.scala:117-125`) hace `dispatcher.createMailbox(this, new UnboundedMailbox)`: el mailbox
seleccionado por los props **se descarta** en ese caso. Rakka hace lo mismo. Dead letters y terminación no cambian: la celda de reemplazo es la que recibe `Create(Some(ActorInitializationException))`
y el supervisor decide. En Rakka un `with_mailbox` no puede fallar al crear la celda (el constructor ya rechazó la capacidad negativa), así que hoy sólo se llega a ese camino por una
falla ajena a la cola. Test nuevo que lo fija: `the_replacement_of_a_cell_that_cannot_be_made_ignores_the_selected_bounded_mailbox`
(props con capacidad 0, un supervisor que reinicia: la línea resultante es igual a la del oráculo `MBX-IF-03`, es decir, el mensaje se recibe); mutante nuevo
`replacement-cell-honours-the-selected-mailbox` DETECTED, control CLEAN (2962/2962). Riesgo de regresión al usar `Props::with_mailbox`: ninguno.

### 3. Controles finales

Cambios posteriores a `446a64a`: sólo un test, un mutante, la sonda y este texto (sin producción). `cargo test --workspace`, fmt y `--check-patterns` limpios;
el mutante nuevo, DETECTED, y el control CLEAN 2962/2962. Q-MBX-1 y Q-MBX-2 siguen sin implementar (el diff no define `BoundedMailbox`, `MailboxType` ni `Mailboxes`).
La cola guarda sólo `capacity` y `dead_letters`; el receptor es un argumento prestado, y `a_terminated_actor_with_a_bounded_mailbox_is_released` comprueba que no queda
ninguna referencia fuerte al actor tras terminar.
