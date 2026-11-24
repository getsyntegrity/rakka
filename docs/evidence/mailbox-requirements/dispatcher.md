# Requisito del dispatcher y selección de mailbox (Slice 3)

Base: la rama `slice/mailbox-requirements-docs` (cuarto PR de la pila). Pekko 2.0.0 `9fbd2283`. `AC-MBX-013/014` siguen `partial`.

## Contrato (Scala, `Mailboxes.scala:132-201`; oráculo `MBX-REQ-04`; sonda S8/S9)

| Pregunta | Respuesta verificada |
|---|---|
| Declaración | clave `mailbox-requirement` de la configuración del dispatcher: nombre de clase; `""` (por defecto, `reference.conf:632-635`) = sin requisito (`getMailboxRequirement` devuelve `MessageQueue`) |
| Mapeo a mailbox | la misma tabla `pekko.actor.mailbox.requirements` (`lookupByQueueType`) que el requisito del actor |
| Precedencia | `deploy.mailbox` > requisito del actor > **requisito del dispatcher** > `default-mailbox`; además el del dispatcher es la alternativa cuando el mapeo del actor falla (cualquier `NonFatal`: sin binding o id sin registrar) |
| Props y Deploy | un id seleccionado (Props o Deploy no vacío) gana al dispatcher; el id vacío no cuenta |
| Sin mapeo | `ConfigurationException "Mailbox Mapping for [interface T] not configured"` con `T` el requisito del dispatcher, envuelta por `actorOf`; si el actor también tenía requisito sin binding, el mensaje nombra el del dispatcher |
| Dispatcher con `MessageQueue` | `hasMailboxRequirement` es falso (`Mailboxes.scala:165`): no hay requisito; Rakka normaliza `Some(MESSAGE_QUEUE)` a `None` (hallazgo de la revisión independiente; test `a_dispatcher_requiring_any_queue_has_no_requirement`) |
| Selección vs validación | seleccionar = elegir el mailbox (este PR). `verifyRequirements` (en cada rama) comprueba que la cola producida cumple el requisito del dispatcher y el del actor: sonda S8 `#2`, un actor `Unbounded` bajo un dispatcher bounded falla con "does not fulfill requirement for actor class". Es validación: Slice 4 |

La clave `pekko.actor.default-dispatcher.mailbox-requirement` de una configuración HOCON **se ignora** (`Settings::from_config` no la lee; sin HOCON, Q-MBX-12): sólo rige `Settings::mailbox_requirement`. La alternativa del dispatcher cuando el actor no tiene binding o su binding apunta a un id sin registrar sólo tiene tests unitarios; el oráculo `MBX-REQ-04` la cubre con un actor que requiere `MessageQueue` (sin binding).

No se extrapoló la precedencia del actor: el dispatcher ocupa el paso 4 y sólo actúa como alternativa del paso 3.

## Rust

`Settings::mailbox_requirement` (el único dispatcher que existe) y un parámetro de `Props::resolve_mailbox`. `MessageQueueSemantics::MESSAGE_QUEUE` permite a un actor requerir "cualquier cola": sin binding propio, usa el del dispatcher o falla (sonda S1 `#3`). No se implementó un sistema de dispatchers: `Props` aún no elige dispatcher, `mailbox-type` propio del dispatcher (paso 2) y dispatchers con id quedan fuera.

## Mutantes sobre el tip (11 en total + control; 0 errores de infraestructura o compilación)

Repetición de los 6 de `parity.md` (el refactor de `resolve_mailbox` cambió el texto de dos) y 5 nuevos. Control sin mutaciones: **2971/2971**. Resultado: 11 DETECTED, 0 UNDETECTED.

| Mutante | Detección |
|---|---|
| `mailbox-dispatcher-requirement-ignored` | **específica**: fast-kill en `mb-0027` (`MBX-REQ-04`), no en `mb-0001` |
| `mailbox-dispatcher-any-queue-counts-as-a-requirement` | **específica**: unitario `a_dispatcher_requiring_any_queue_has_no_requirement` (1 de 22) |
| `mailbox-dispatcher-requirement-beats-the-actor` | **específica**: unitario `the_requirement_of_the_actor_decides_before_…` (1 de 21) |
| `mailbox-dispatcher-requirement-is-not-the-fallback` | **específica**: 3 unitarios fallan (`…serves_an_actor_whose_requirement_has_no_binding`, …) |
| `mailbox-dispatcher-requirement-beats-the-selected-id` | **específica**: unitario `a_selected_id_decides_before_the_requirement_of_the_dispatcher` |
| `mailbox-requirement-error-names-the-class-without-its-kind` | **específica**: fast-kill en `mb-0025` |
| `mailbox-requirement-beats-the-selected-id` | **específica**: unitario (1 de 21) |
| `mailbox-actor-requirement-ignored`, `-binding-ignored`, `-applied-to-every-actor`, `-without-binding-uses-the-default` | **caída de la familia mailbox**: fast-kill en `mb-0001` (limitación 8 del harness: `rust_lines()` calcula todos los casos) |

## RED / GREEN

`red-dispatcher-any-queue.txt` (hallazgo de la revisión independiente): un dispatcher con `MessageQueue` hacía fallar a todo actor sin binding; RED, luego `Some(MESSAGE_QUEUE)` se normaliza a `None`. `red-dispatcher-requirement.txt`: con la API declarada y el requisito sin consultar, 4 de 6 tests nuevos fallan por comportamiento (`left: 5, right: 2`) y 2 de precedencia pasan como guardas. GREEN: `dispatch::mailboxes` 22/22. Oráculo `MBX-REQ-04` (`mb-0027`) coincide con Rakka byte a byte.
