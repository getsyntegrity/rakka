# Guardianes y requisitos de cola (Slice 3, tarea 9 de Q-MBX-12)

Base `9b8ea66`, Pekko 2.0.0 `9fbd2283`. Sonda ejecutada contra el jar del oráculo (sistema real, JDK 17); no es un caso grabado porque Rakka aún no puede reproducirla (ver "Qué falta").

## Contrato verificado

- Declaran `RequiresMessageQueue[UnboundedMessageQueueSemantics]`: `LocalActorRefProvider.Guardian` (raíz y `/user`) y `SystemGuardian` (`ActorRefProvider.scala:322-340`), además de actores de `io` (`TcpConnection`, `UdpSender`, ...) y `DefaultLogger` (`LoggerMessageQueueSemantics`).
- **Los guardianes no pasan por `Mailboxes.getMailboxType`.** `LocalActorRefProvider` les da `defaultMailbox = mailboxes.lookup(DefaultMailboxId)` (`ActorRefProvider.scala:543,550,583-584,600`); sólo `/user` con `guardianProps` usa `lookup(props.mailbox)`. Por eso el requisito no participa en la **selección**.
- El requisito se verifica **después de crear** el mailbox, en `Dispatch.init` (`dungeon/Dispatch.scala:91-103`): sólo si el `MailboxType` es `ProducesMessageQueue` y la cola real no cumple, `Create(Some(ActorInitializationException("Actor [..] requires mailbox type [req] got [got]")))`. El fallo llega al actor como excepción de inicialización, no al llamar a `actorOf`.

| Sonda | Resultado en Pekko 2.0.0 |
|---|---|
| G1: `default-mailbox` = cola LIFO que cumple unbounded | los tres guardianes usan la cola LIFO: el default sirve a los guardianes |
| G3: igual, y `UnboundedMessageQueueSemantics` enlazado a un mailbox acotado | los tres guardianes siguen con la cola LIFO: el binding no los afecta |
| G2: `default-mailbox` acotado (`NonBlockingBoundedMailbox`) | el sistema no arranca (los guardianes fallan al iniciar y los loggers no responden: `LoggerInitializationException ... [TIMEOUT]`) |

Corrección: la nota del Slice 2 ("los actores del sistema, guardianes incluidos, pasan por `getMailboxType`") es imprecisa para los guardianes; sí vale para los actores creados con `actorOf`/`systemActorOf` (loggers).

## Qué hace Rakka y qué falta

- Los guardianes de Rakka no consultan el default registrado (usan `UnboundedMailbox`) ni verifican el requisito. No se añade una declaración suelta: no tendría efecto sin la verificación de `Dispatch.init`.
- Slice 4: que los guardianes usen `lookup(default-mailbox)`, declaren `UNBOUNDED` y se verifique el requisito tras crear la cola (`ActorInitializationException`), junto con `verifyRequirements` de `getMailboxType`.

## Slice 4 (implementado)

Los guardianes de Rakka usan ahora `lookup(default-mailbox)`, declaran `UNBOUNDED`, y `Dispatch.init` verifica la cola creada; un guardián que falla detiene el sistema. Cada uno de los tres se probó por separado contra Pekko y contra Rakka. Ver `docs/evidence/mailbox-compat/guardians.md`. Sigue sin reproducirse el error de `ActorSystem.create` (timeout de los loggers) ni `guardianProps`.
