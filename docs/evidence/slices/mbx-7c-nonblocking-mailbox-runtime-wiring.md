# MBX PR 7c: cableado de `NonBlockingBoundedMailbox` en el runtime (AC-MBX-008) — **bloqueado, a la espera del Owner**

Base: `origin/feat/pekko-rust-pilot-migration` @ `162cab1`. Sólo documentación: no hay cambios de producción. Nada de lo que sigue es una
decisión; son propuestas (`proposed`) y no aprueban ni resuelven Q-MBX-1, Q-MBX-2, PROP-7A-1 ni PROP-7B-1.

## Resultado de la inspección

El cableado mínimo (una cola `BoundedNodeMessageQueue` detrás de un actor real) **no puede hacerse sin decidir** algo de la lista de bloqueos:

| # | Hallazgo (código en 162cab1) | Consecuencia |
|---|---|---|
| H1 | `ActorCell::build` (`actor/cell.rs:235-250`) crea la cola dentro de `Arc::new_cyclic` (`queue: UnboundedMailbox.create()`). Todo `ActorRef` local es `LocalActorRef(Arc<ActorCell>)`; `local_ref()` (`:308-312`) hace `self.me.upgrade().expect("cell alive")`, que **falla dentro de `new_cyclic`**. | Para una celda local (`new_local`: guardianes y `context.actorOf`) el `ActorRef` del dueño **no existe** cuando se crea la cola. `create(owner, dead_letters)` no puede recibirlo. |
| H2 | Para una celda creada por `system.actorOf` (`new_repointed`) el `outer` (`RepointableActorRef`) sí existe al construir. Pero el `RepointableActorRef` retiene la celda y la celda retiene `outer` (`finish_terminate` lo suelta al final, `:910`: "let go of it so the pair can be freed"). | Una cola que guarde `outer` como dueño añade un tercer dueño fuerte que **no se suelta al terminar**: ciclo `celda → cola → outer → celda`, fuga de la pareja. Con el dueño débil no hay fuga. |
| H3 | `Props` (`actor/props.rs:64`) sólo tiene `factory`: no hay forma de elegir mailbox. No existen `MailboxType` ni `Mailboxes` (ADR-0001 §6 paso 10, "tras Q-MBX-2", que sigue `proposed`). | Aunque H1/H2 se resolvieran, no hay punto de entrada para seleccionar la cola: añadirlo es API pública nueva (`Props::with_mailbox` o config `mailbox-type`) y es el contenido de Q-MBX-2. |
| H4 | `MessageQueue::enqueue(&self, envelope)` no recibe receptor (`dispatch/message_queue.rs`). El receptor de `DeadLetter(msg, sender, receiver)` sólo puede venir del dueño fijado en `create` (PROP-7B-1). | O dueño fijado tras `create`, o receptor por llamada. |
| H5 | `enqueue_unless_closed` + `register_for_execution` (`cell.rs:392-404`): una cola que rechaza **no** debe agendar un turno ni contarse como encolada; hoy `enqueue_user` agenda según `dead.is_empty()`, que describe sólo el cierre. | Con una cola que rechaza por llena hace falta que `enqueue` informe si almacenó (hoy devuelve `()`); es otro cambio del trait `MessageQueue`, o agendar un turno vacío (inocuo pero distinto de Pekko, `Dispatcher.dispatch` agenda siempre). |

Resto del ciclo de vida evaluado (no bloquea por sí mismo):

- **Reinicio:** la celda y su cola sobreviven (`finishRecreate` no reemplaza la cola); el dueño es estable. Sin impacto.
- **Parada (C4):** `finish_terminate` (`cell.rs:885-889`) cierra el estado y llama `queue.clean_up(&*leftovers)`; `BoundedNodeMessageQueue::clean_up`
  ya vacía en orden y libera los huecos (`NB-QU-05`). Sin impacto si el dueño no retiene la celda (H2).
- **Actor nunca iniciado:** los mensajes van a la cola provisional de `UnstartedCell` (`unstarted-push-timeout`, C3) y al apuntar pasan a la cola real con
  `enqueue`; el desbordamiento de esa transferencia se dead-letterea con el dueño. La capacidad 0 con `actorOf` (BoundedMailbox, PROP-7A-1) es otro camino: no se toca.
- **Capacidad 0 de `NonBlockingBoundedMailbox`:** válida (NB-QU-03, golden de 10 casos); su prueba con actores reales necesita H3.

## Alternativas (impacto en paridad y arquitectura)

**A. Receptor por llamada: `MessageQueue::enqueue(&self, receiver: &ActorRef, envelope) -> bool`** (PROP-7B-1 con la firma de Pekko).
- Paridad: la más cercana a `Mailbox.scala:376-410`; resuelve H1, H2 y H4 (el dueño nunca se guarda) y H5 (el retorno dice si almacenó).
- Arquitectura: cambia el trait y sus cuatro implementaciones/llamadores (`UnboundedMessageQueue`, `BoundedNodeMessageQueue`, `mailbox_close::enqueue_unless_closed`,
  `DeadLetterMailbox`); la cola no retiene `ActorRef`, sin ciclos. `dead_letters` seguiría dado en `create` (o por el sistema). Mayor diff (~150-250 líneas), mecánico.
- Falta aparte: H3 (selección).

**B. Dueño fijado tras `create` con `OnceLock<ActorRef>`** (la alternativa prevista en la revisión de #331).
- Paridad: la misma conducta observable; no cambia el trait. H5 sigue abierto.
- Arquitectura: el dueño `ActorRef` retiene la celda → H2 obliga a que sea débil o a soltarlo en `finish_terminate` (más estado y un caso "owner no fijado" a cubrir).
  En celdas locales hay que fijarlo justo tras `new_cyclic`. Más pequeño que A, pero introduce estado mutable y una ventana sin dueño.

**C. Dueño débil sin `OnceLock`: referencia local auxiliar `WeakLocalActorRef { path, Weak<ActorCell> }` construida dentro de `new_cyclic`.**
- Paridad: `ActorRef` igual por `path`+`uid`, así que `DeadLetter.recipient == self_ref`; `tell` reenvía por `upgrade` (si la celda murió, se descarta).
- Arquitectura: un tercer tipo de `InternalActorRef`; no cambia el trait ni usa `OnceLock`, pero es un tipo de referencia nuevo y `is_terminated`/`stop` sobre él
  requieren definición. H5 y H3 siguen abiertos.

Recomendación: **A** (es la forma de Pekko, no retiene referencias y cierra H5), con H3 resuelto después por Q-MBX-2. Aun así elegirla es del Owner: toca la firma de
`MessageQueue::enqueue`, que el encargo reservó explícitamente.

## Independiente y ya cubierto (sin PR de producción)

- La cola, sus 10 casos contra el oráculo JVM, la ausencia de bloqueo, de pérdida y de duplicación y la capacidad 0 ya están en #331 (`bounded_node_queue.rs`,
  `cases/oracle-only/nonblocking-bounded-mailbox.golden`). Lo que falta es exactamente lo de H1-H5; no hay un slice funcional más pequeño que no lo requiera.
- No se agregaron pruebas de integración con actores reales: serían RED permanentes (no hay API para elegir la cola) o dobles que no acreditan la integración
  (`docs/testing-policy.md`: un doble no acredita la integración con el componente sustituido).

## Qué se necesita del Owner para desbloquear

1. Elegir A, B o C para el dueño/receptor (PROP-7B-1) y, si es A, aprobar el cambio de firma de `MessageQueue::enqueue` (con retorno `bool`).
2. Aprobar el punto de entrada de selección (Q-MBX-2): mínimo propuesto = `Props::with_mailbox(NonBlockingBoundedMailbox)` sólo para esta cola, sin `Mailboxes` ni config
   (NB-CF-02 queda para el PR 10).
3. Con eso, el slice tiene un tamaño de ~300 líneas autoradas: tests RED con actores reales (cola llena rechaza al instante, `DeadLetter(msg, emisor, receptor)`,
   sin pérdida ni duplicación, capacidad 0, parada con la cola llena), cableado, mutantes y replay.
