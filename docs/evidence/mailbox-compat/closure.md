# Slice 4 (compatibilidad de mailbox): contratos, tarea 9 de Q-MBX-12 y deuda técnica

Base `92acd83` (rama de integración), Pekko 2.0.0 `9fbd2283`. PR en draft apilados: #362 (A) → #363 (B) → #364 (C) → #365 (D) → #366 (E) → #367 (F) → este (G). Sin merge.
`AC-MBX-013` y `AC-MBX-014` siguen `partial`; Q-MBX-13 `accepted`; Q-MBX-2 `proposed`; **Q-MBX-14 `proposed`** (API de capacidades, ver `docs/owner-batch-02.md`).

## Contratos implementados (cada uno con oráculo o sonda contra Pekko 2.0.0)

| Contrato | PR | Evidencia |
|---|---|---|
| `ProducesMessageQueue`: cola y semánticas que promete un `MailboxType`; el tipo sin marcador produce `MessageQueue` | A | `queue-capabilities.md` |
| `verifyRequirements` en `getMailboxType`: requisito del dispatcher y del actor, dispatcher primero, fallback al binding del dispatcher ante un fallo de verificación | B | `verify-requirements.md` |
| Mensaje y tipo del error (`IllegalArgumentException` envuelta en `ConfigurationException`), nombre liberado | C, D | MBX-CMP-01 (mb-0028), MBX-CMP-02 (mb-0029) idénticos al oráculo |
| `Dispatch.init`: la cola realmente creada se vuelve a comprobar si el tipo promete; fallo retrasado a `Create`; capacidades declaradas ≠ efectivas | E | `init-check.md`, MBX-CMP-03 (mb-0030) |
| Guardianes: root, `/user`, `/system` toman el default-mailbox directamente, requieren `Unbounded`, un guardián fallido detiene el sistema | F | `guardians.md`, MBX-GRD-01 (mb-0031) |

## Tarea 9 de Q-MBX-12 (guardianes y requisitos de cola)

Cumplida en lo ejecutable: selección directa del default, requisito por guardián (los tres `Unbounded`, probados uno a uno contra Pekko y Rakka), verificación en `Dispatch.init`, apagado del sistema. **No** reproducido: el error de `ActorSystem.create` (en la JVM viene del timeout de arranque de los loggers), `guardianProps`, `deadLetterListener` y `eventStreamUnsubscriber`.

## Deuda técnica restante

1. Tres anclas de mutantes obsoletas anteriores al Slice 3 (`config-push-timeout-not-read`, `deploy-fallback-keeps-the-tags`, `render-concatenation-keeps-white-space-as-a-separate-string`); 4 tests de `tools/gates` fallan por ellas. No son dependencia de la compatibilidad.
2. HOCON `mailbox-requirement` y `pekko.actor.mailbox.requirements`; `mailbox-type` propio del dispatcher; dispatcher elegido por `Props` (el requisito sólo existe para el dispatcher por defecto).
3. Filas de la tabla de requisitos sin mailbox (bounded/deque/control-aware/logger) y las semánticas Deque, ControlAware: el subtipado entre ellas se listará por tipo cuando existan (Q-MBX-14).
4. `Props::with_mailbox(tipo)` (sin equivalente en Pekko) no se verifica; los nombres de clase de actor y cola en los mensajes son rutas de tipo Rust (normalizadas sólo en los casos grabados).
5. Granularidad del harness: `mailbox-case` corre la familia en un proceso; 2 de los 3 mutantes de E se detectan por caída de familia (30 divergencias), no de forma específica.
6. La cobertura baja de 98,2 % a 96,3 % regiones con el arnés ampliado (sigue ≥95 %).
