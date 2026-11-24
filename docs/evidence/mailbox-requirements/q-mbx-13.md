# Q-MBX-13: decisión del Owner (estado: `accepted`, 2026-11-24)

Aceptada con las limitaciones de este documento. No acepta Q-MBX-2 ni ninguna otra decisión; concreta sólo la parte de **selección** de Q-MBX-2. Registro: `docs/decisions/register.tsv`.

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

## Convenciones y riesgos que acompañan a la decisión

- **Nombres cualificados.** Un requisito personalizado se nombra con la ruta cualificada de su marcador, `crate.modulo.Marcador` (por ejemplo `miapp.audit.AuditTrailSemantics`), igual que Pekko usa el nombre completo de la clase. No se usan nombres sueltos como `Audit`.
- **Colisiones entre crates.** La identidad es el texto: dos crates que declaren el mismo nombre comparten requisito y binding sin aviso. La convención de nombres cualificados lo evita en la práctica; no hay detección en ejecución. La alternativa por `TypeId` (b) lo resolvería a costa del registro dinámico de tipos.
- **Compatibilidad de código fuente de `Settings`.** El campo público `mailbox_requirement` hace que un literal `Settings { .. }` sin `..Default::default()` deje de compilar. Es un cambio de código fuente, no de comportamiento; las construcciones con `Settings::default()` o `..Settings::default()` no se afectan. Cuenta para `cargo semver-checks` desde el primer release.
- **Selección y verificación separadas.** Este slice elige el mailbox; no comprueba que la cola producida cumpla el requisito (`ProducesMessageQueue`, `verifyRequirements`, `ActorInitializationException` tras crear la cola). Un binding incompatible se acepta aquí y Pekko lo rechaza: Slice 4. No se declara paridad de compatibilidad.
- **HOCON y tabla pendientes.** Las claves `pekko.actor.mailbox.requirements` y `pekko.actor.default-dispatcher.mailbox-requirement` se ignoran; sólo rigen `Mailboxes::bind_requirement` y `Settings::mailbox_requirement`. De la tabla por defecto sólo se enlaza `UnboundedMessageQueueSemantics`; las demás filas esperan sus mailboxes (Q-MBX-1 y relacionadas).
