# Selección de mailbox: default registrado, Deploy y configuración (Slice 2)

Base: `c4c6cbb` (rama de integración). Upstream: Apache Pekko 2.0.0, `9fbd2283`. `AC-MBX-014` sigue `partial`.

## Contrato verificado en el Scala fijado

- `Props.mailbox` devuelve `pekko.actor.default-mailbox` sólo como conveniencia (`Props.scala:168`); la selección real usa `deploy.mailbox` crudo (`Mailboxes.getMailboxType`, `Mailboxes.scala:178-199`).
- Orden de `getMailboxType`: (1) `deploy.mailbox != ""` → `lookup(deploy.mailbox)`; (2) dispatcher propio con `mailbox-type` (fuera de alcance); (3) requisito de cola del actor (fuera de alcance); (4) requisito del dispatcher (fuera de alcance); (5) `lookup("pekko.actor.default-mailbox")`.
- Antes, `ActorRefProvider.actorOf` (`:692-701`) sustituye el mailbox de Props por el del `Deploy` hallado con `deployer.lookup(path)` cuando éste no es vacío: **Deploy/configuración pisa a Props**. Un `Deploy` vacío deja a Props.
- El mensaje del error de `actorOf` usa `props2.mailbox`, es decir el id **efectivo** (`:724`).
- Los actores del sistema (guardianes, loggers) también pasan por `getMailboxType` y exigen `UnboundedMessageQueueSemantics` (`RequiresMessageQueue`, fuera de alcance): un default acotado impide arrancar el sistema (observado con el oráculo).

## PR 1 — default registrado para actores sin selección (tarea 1 de Q-MBX-12)

`Props::resolve_mailbox` resuelve `pekko.actor.default-mailbox` cuando no hay id ni tipo directo. `MBX-CFG-02` (oráculo `mb-0022`): el tipo configurado como default sirve a "sin selección", `""` y el id default; no a `unbounded` ni a otro id.

RED: `red-default-mailbox.txt` (`left: [a, b, c]`, `right: [c, b, a]`). GREEN: `green-default-mailbox.txt`.
