# Q-MBX-13 para revisión del Owner (estado: `proposed`)

No acepta Q-MBX-2 ni ninguna otra decisión; concreta sólo la parte de **selección** de Q-MBX-2. Registro: `docs/decisions/register.tsv`.

## Problema

Pekko deja que un actor declare qué cola necesita (`RequiresMessageQueue[T]`) y elige su mailbox con la tabla `pekko.actor.mailbox.requirements` (`Mailboxes.scala:157-201`). Rakka sólo seleccionaba por id. Sin esto, `Stash` (deque), los actores de `io` y cualquier actor con requisito reciben el mailbox por defecto.

## Alternativas

| | Representación | A favor | En contra |
|---|---|---|---|
| (a) elegida | `MessageQueueSemantics`: conjunto abierto identificado por un nombre de clase `&'static str` | sirve a marcadores de usuario (`ActorMailboxSpec.scala:224-234`); reproduce el texto del error de Pekko; sin registro dinámico de tipos | dos crates pueden elegir el mismo nombre |
| (b) | tipo marcador con `TypeId` | sin colisiones | la tabla exige registrar tipos en ejecución; el error no tiene nombre JVM |
| (c) | enum cerrado | mínimo | no admite marcadores de usuario: incumple el spec de Pekko |

## Representación Rust elegida

- `Actor::required_message_queue() -> Option<MessageQueueSemantics>`: función asociada al tipo (en Pekko el requisito es de la clase); `Props::new` la captura sin instanciar el actor.
- `Mailboxes::bind_requirement(semantics, mailbox_id)` y `lookup_by_queue_type`: tabla programática (Q-MBX-12, sin HOCON) que parte de la fila unbounded de `reference.conf`.
- Constantes `UNBOUNDED`, `BOUNDED` y `MESSAGE_QUEUE`; `Mailboxes::UNBOUNDED_QUEUE_BASED_ID`.
- `Settings::mailbox_requirement: Option<MessageQueueSemantics>`: `pekko.actor.default-dispatcher.mailbox-requirement` (`None` = cadena vacía). Todos los actores corren en ese dispatcher hasta que `Props` pueda elegir otro.

## Justificación semántica

Oráculo `MBX-REQ-01..03` y sonda de 11 escenarios contra Pekko 2.0.0: el requisito gana al default y pierde ante un id del Props o del Deploy (el id vacío no cuenta); sin binding falla `actorOf` con `ConfigurationException "Mailbox Mapping for [interface T] not configured"`; el requisito de un actor no afecta a otro.

## Extensibilidad

Un mailbox personalizado se registra con `Mailboxes::register` y se enlaza con `bind_requirement(MessageQueueSemantics::new("app.Mi"), id)`. Un actor lo pide con `required_message_queue`. No hace falta tocar el crate.

## Limitaciones

- No hay `ProducesMessageQueue` ni `verifyRequirements`: un binding a una cola que no cumple el requisito se acepta (Pekko lo rechaza). Slice 4.
- Sólo la fila unbounded de la tabla por defecto; bounded, deque y control-aware esperan sus mailboxes (Q-MBX-1).
- Sin tabla por HOCON ni FQCN.
- Guardianes: no pasan por la selección (`guardians.md`); Slice 4.

## Consecuencias sobre la API pública

Nuevos: `Settings::mailbox_requirement` (campo público: quien construye `Settings` con un literal sin `..Default::default()` deja de compilar), `dispatch::queue_semantics::MessageQueueSemantics`, `Actor::required_message_queue` (método con valor por defecto: no rompe a los implementadores), `Mailboxes::{bind_requirement, lookup_by_queue_type, UNBOUNDED_QUEUE_BASED_ID}`. `Mailboxes` deja de derivar `Default` (lo implementa a mano, con la tabla inicial). Si el Owner prefiere (b), cambia `MessageQueueSemantics` y `Actor::required_message_queue`; la selección y los casos del oráculo no cambian.
