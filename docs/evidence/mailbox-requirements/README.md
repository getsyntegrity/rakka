# Requisitos de cola del actor y selección de mailbox (Slice 3)

Base `9b8ea66` (rama de integración). Upstream: Apache Pekko 2.0.0, `9fbd2283`. `AC-MBX-014` sigue `partial`; `AC-MBX-013` sigue `partial` (falta la validación, Slice 4).

## Contrato verificado (Scala + oráculo)

`Mailboxes.getMailboxType` (`Mailboxes.scala:157-201`) elige, en este orden: (1) `deploy.mailbox`; (2) dispatcher propio con `mailbox-type`; (3) requisito del actor (`RequiresMessageQueue[T]`) → `lookupByQueueType(T)`, con el requisito del dispatcher como alternativa si ese falla; (4) requisito del dispatcher; (5) `default-mailbox`. `lookupByQueueType` consulta la tabla `pekko.actor.mailbox.requirements` (`:78-100,110,208-212`); sin binding: `ConfigurationException "Mailbox Mapping for [interface T] not configured"`, que `actorOf` envuelve en "configuration problem while creating …" (`ActorRefProvider.scala:714-724`).

Antes de fijar expectativas se ejecutó una sonda contra el jar 2.0.0 (11 escenarios). Resultados que sostienen este PR:

| Hipótesis | Observado en Pekko 2.0.0 | Dónde se prueba |
|---|---|---|
| Actor sin requisito | recibe el default registrado | `plain`, `plainAfter` (`MBX-REQ-01`, `mb-0024`) |
| Requisito unbounded, tabla de `reference.conf` | `UnboundedMailbox`, no el default (aquí LIFO) | `unbounded` |
| Requisito bounded con binding configurado | la cola del id enlazado (capacidad 4) | `bounded` |
| Requisito definido por la aplicación | la cola del id enlazado | `audit` |
| Requisito sin binding | `ConfigurationException` en `actorOf`; el nombre queda libre | `MBX-REQ-02` (`mb-0025`) |
| Aislamiento entre actores | un actor sin requisito creado después conserva el default | `plainAfter` |
| Id seleccionado/Deploy frente a requisito | gana el id (compatible) | unitario y `MBX-REQ-03` (`mb-0026`) |

## Decisiones de diseño

- `MessageQueueSemantics`: conjunto abierto identificado por el nombre de clase de su marcador (Q-MBX-2 (a'), aún `proposed`). Un enum cerrado no sirve: el spec de Pekko usa marcadores de usuario (`ActorMailboxSpec.scala:224-234`). Constantes `UNBOUNDED`, `BOUNDED` y `MESSAGE_QUEUE` ("cualquier cola", que para un dispatcher significa sin requisito); las demás se añaden con sus mailboxes.
- Declaración: `Actor::required_message_queue()` asociada al tipo (como `RequiresMessageQueue` es del tipo del actor); `Props::new` la captura, sin instanciar el actor.
- Resolución: `Mailboxes::bind_requirement` / `lookup_by_queue_type`; la tabla parte de `reference.conf` (`UnboundedMessageQueueSemantics → pekko.actor.mailbox.unbounded-queue-based`, que es `UnboundedMailbox` salvo que se registre otro tipo).
- Limitación temporal, igual que Q-MBX-12: `bounded-queue-based` y el resto de la tabla por defecto no se enlazan hasta que existan sus mailboxes (Q-MBX-1); un actor con requisito bounded necesita un binding explícito.
- Fuera de este PR (Slice 4): `ProducesMessageQueue` y `verifyRequirements`. Aquí un binding a una cola que no cumple el requisito se acepta; Pekko lo rechaza (`produced message queue type … does not fulfill requirement`). No se declara paridad de compatibilidad.

## RED / GREEN

`red-actor-requirement.txt`: con la API declarada y `resolve_mailbox` sin consultar el requisito, 6 de los 15 tests de `dispatch::mailboxes` fallan por comportamiento (p. ej. `left: 3, right: 100`: el actor con requisito recibía el default). `green-actor-requirement.txt`: implementado, 104/104 en `dispatch::`. Los casos de integración `MBX-REQ-01/02` se escribieron con la unidad ya en GREEN; su RED es el mutante `mailbox-actor-requirement-ignored`, que hace divergir `mb-0024`.
