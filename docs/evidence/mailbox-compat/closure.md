# Slice 4 (compatibilidad de mailbox): contratos, tarea 9 de Q-MBX-12 y deuda técnica

Base `92acd83` (rama de integración), Pekko 2.0.0 `9fbd2283`. PR en draft apilados: #362 (A) → #363 (B) → #364 (C) → #365 (D) → #366 (E) → #369 (H, arreglo de `satisfies`) → #367 (F) → #370 (F2, whenTerminated pendiente) → #371 (F3, otros caminos del walker) → este (G). Sin merge.
`AC-MBX-013` y `AC-MBX-014` siguen `partial`; Q-MBX-13 `accepted`; Q-MBX-2 `proposed`; **Q-MBX-14 `accepted`** (Owner, 2026-11-25, API de capacidades). Clasificación de la evidencia: **SOURCE_VERIFIED** contra el Scala fijado de Pekko 2.0.0; **RUST_VERIFIED** con tests de regresión, mutantes y replay; **la paridad de comportamiento en la JVM NO está verificada** salvo los casos MBX-CMP-01..03 y MBX-GRD-01 medidos con el oráculo (no para `whenTerminated` tras el fallo de un guardián ni para `ActorSystem.create`).

## Contratos implementados (cada uno con oráculo o sonda contra Pekko 2.0.0)

| Contrato | PR | Evidencia |
|---|---|---|
| `ProducesMessageQueue`: cola y semánticas que promete un `MailboxType`; el tipo sin marcador produce `MessageQueue` | A | `queue-capabilities.md` |
| `verifyRequirements` en `getMailboxType`: requisito del dispatcher y del actor, dispatcher primero, fallback al binding del dispatcher ante un fallo de verificación | B | `verify-requirements.md` |
| Mensaje y tipo del error (`IllegalArgumentException` envuelta en `ConfigurationException`), nombre liberado | C, D | MBX-CMP-01 (mb-0028), MBX-CMP-02 (mb-0029) idénticos al oráculo |
| `Dispatch.init`: la cola realmente creada se vuelve a comprobar si el tipo promete; fallo retrasado a `Create`; capacidades declaradas ≠ efectivas | E | `init-check.md`, MBX-CMP-03 (mb-0030) |
| Guardianes: root, `/user`, `/system` toman el default-mailbox directamente, requieren `Unbounded`, los guardianes `/user` y `/system` fallidos terminan el sistema (si falla el raíz, `whenTerminated` queda pendiente) | F | `guardians.md`, MBX-GRD-01 (mb-0031) |

## Tarea 9 de Q-MBX-12 (guardianes y requisitos de cola)

Cumplida en lo ejecutable: selección directa del default, requisito por guardián (los tres `Unbounded`, probados uno a uno contra Pekko y Rakka), verificación en `Dispatch.init`, apagado del sistema (en `/user` y `/system`; el raíz deja `whenTerminated` pendiente). **No** reproducido: el error de `ActorSystem.create` (en la JVM viene del timeout de arranque de los loggers), `guardianProps`, `deadLetterListener` y `eventStreamUnsubscriber`.

## Deuda técnica restante

1. ~~Tres anclas de mutantes obsoletas~~ **Reparadas** (cierre): `config-push-timeout-not-read` (el código ahora también lee el `deployer` entre las dos líneas), `deploy-fallback-keeps-the-tags` (`tags` es un `Vec`, ya no un `BTreeSet`) y `render-concatenation-keeps-white-space-as-a-separate-string` (`plain_text` devuelve `String`): mismo modelo de fallo, 3/3 DETECTED con control CLEAN 2975/2975. El test de `tools/gates` que fijaba 205 mutantes ahora fija 333 (exacto). `tools/gates`: 159 tests OK.
2. HOCON `mailbox-requirement` y `pekko.actor.mailbox.requirements`; `mailbox-type` propio del dispatcher; dispatcher elegido por `Props` (el requisito sólo existe para el dispatcher por defecto).
3. Filas de la tabla de requisitos sin mailbox (bounded/deque/control-aware/logger) y las semánticas Deque, ControlAware: el subtipado entre ellas se listará por tipo cuando existan (Q-MBX-14).
4. `Props::with_mailbox(tipo)` (sin equivalente en Pekko) no se verifica; los nombres de clase de actor y cola en los mensajes son rutas de tipo Rust (normalizadas sólo en los casos grabados).
5. Granularidad del harness: `mailbox-case` corre la familia en un proceso; 2 de los 3 mutantes de E se detectan por caída de familia (30 divergencias), no de forma específica.
6. La cobertura baja de 98,2 % a 96,3 % regiones con el arnés ampliado (sigue ≥95 %).

## Revisión independiente (sobre `92acd83..e64a671`, agente con lectura efectiva de Rust y Scala; sin defectos altos)

Dos revisores especializados se negaron por no tener herramientas de lectura y no abrieron nada: no cuentan. Hallazgos del revisor que sí leyó el código, pendientes (no corregidos para no cambiar los SHA revisados):

1. **Medio, corregido en #369 (H).** `MessageQueue::satisfies` devuelve `Option<bool>`: `None` (por defecto) vale lo que prometió el tipo; `Some` se verifica. Descripción original: Lo prometido (`produced_queue`) y lo efectivo (`MessageQueue::satisfies`) tienen dos fuentes: un `MailboxType` de terceros que promete `UNBOUNDED` pero cuya cola no sobrescribe `satisfies` hace fallar `Dispatch.init` aunque la promesa sea honesta (en Pekko la jerarquía de clases es la única fuente).
2. Bajo. `MultipleConsumerSemantics` (que las colas de `UnboundedMailbox` y `BoundedNodeMessageQueue` sí tienen en Pekko) no está listada ni tiene constante.
3. Bajo. `unwrap_or_else` de `lookup(default)` en `system.rs` es código muerto que enmascararía un cambio futuro.
4. Bajo. `ProducedQueue::PLAIN` como centinela confunde "sin promesa" con `ProducesMessageQueue[MessageQueue]` (sin efecto hoy).
5. Bajo. El id del dispatcher en el mensaje está fijo en el por defecto.
6. Bajo (tests). `a_default_mailbox_that_promises_nothing...` afirma "no terminado" justo tras crear; los sistemas del test de fallo no se terminan.

## Revisiones finales (HEAD d5bb0b9, acceso a Scala verificado: Mailbox.scala, Mailboxes.scala, Dispatch.scala, ActorRefProvider.scala, reference.conf)

Sin defectos bloqueantes en esa revision (HEAD d5bb0b9). El hallazgo medio de satisfies (H) y el de MultipleConsumerSemantics (H, 7642c88) estan corregidos y verificados.

Actualizacion (F2 #370 y F3 #371): `whenTerminated` queda PENDIENTE cuando falla el guardian raiz y llega despues su notificacion de muerte, como Pekko 2.0.0 segun la lectura de ActorRefProvider.scala:428-490 (el walker registra la causa en su propio causeOfTermination, deja de caminar y descarta la notificacion, de modo que stop() no enlaza el terminationPromise publico). Si despues se llama directamente a stop() del walker, el futuro publico FALLA con la causa (completeWith del futuro interno fallido, :464-468); /user, /system y la terminacion solicitada terminan con Ok. El futuro es un unico Result<Terminated, Throwable> de un solo valor, sin accessor failure(). Leido del Scala, NO probado con una sonda contra la JVM (alli ActorSystem.create ya falla por el timeout de los loggers). Detalle en guardians.md.
