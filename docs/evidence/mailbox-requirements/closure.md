# Cierre del Slice 3 (Mailbox Requirements) y deuda técnica

Integrado en `feat/pekko-rust-pilot-migration` (HEAD `e8c9aa3`, base `9b8ea66`, upstream Pekko 2.0.0 `9fbd2283`) por squash de #353–#358, con el árbol idéntico al de cada rama. `AC-MBX-013` y `AC-MBX-014` siguen `partial`. **Q-MBX-13 sigue `proposed`**; Q-MBX-2 no se aceptó (dossier: `q-mbx-13.md`).

## Verificación sobre el HEAD integrado

| Gate | Resultado |
|---|---|
| `cargo fmt --check`, `clippy -D warnings`, `cargo deny check licenses` | verdes |
| `cargo test --workspace` | 804 pasan, 0 fallan (788 al inicio) |
| Cobertura (`cargo llvm-cov`) | 98,21 % regiones, 98,25 % líneas |
| Replay completo contra Pekko 2.0.0 | 2971/2971, 0 divergencias |
| Mutantes | 24/24 DETECTED (13 del Slice 2 + 11 nuevos), control 2971/2971, 0 errores de infraestructura o compilación |
| Validación documental, tests de herramientas, `git diff --check` | verdes |

Tamaño de los PR (con evidencia generada): 353, 339, 199, 143, 288 y 284 líneas.

## Deuda técnica

1. ~~**Test intermitente `a_deadline_that_no_message_reaches_leaves_the_throughput_as_the_limit` (dispatcher)**~~ **Resuelto** (PR #361, `docs/evidence/dispatcher-turn-test-race/README.md`). Causa raíz: el helper `interleaving` ocupaba el único hilo del dispatcher antes de que `x` e `y`, creados con el `actorOf` asíncrono, estuvieran instalados; el orden en que entraban a la cola dependía de una carrera con el guardián (en las corridas fallidas trazadas la creación fue siempre posterior al gate). Corregido esperando por evento a que ambos procesen un mensaje; 0/1000 fallos por versión (antes 26–41 %), 0/50 con retrasos diagnósticos que antes daban 50/50. La medición previa (base 7 %, núcleo 20 %, final 12 % con el test aislado) no permite atribuir un aumento al Slice 3: con el módulo entero el signo se invierte (41 %, 31 %, 26 %); la causa existe en todas las versiones.
2. **Tres anclas de mutantes obsoletas desde antes del slice** (`tools/parity/check_divergence.py`): `config-push-timeout-not-read`, `deploy-fallback-keeps-the-tags`, `render-concatenation-keeps-white-space-as-a-separate-string`. Su texto `old` no aparece en el archivo indicado en `9b8ea66`; darían `PATCH_ERROR`. No se ejecutaron en este slice.
3. **Configuración HOCON**: `pekko.actor.default-dispatcher.mailbox-requirement` y `pekko.actor.mailbox.requirements` se ignoran (`Settings::from_config` no los lee); sólo rigen `Settings::mailbox_requirement` y `Mailboxes::bind_requirement`.
4. **Dispatcher**: falta el `mailbox-type` propio del dispatcher (paso 2 de `getMailboxType`) y que `Props` elija dispatcher; el requisito existe sólo para el dispatcher por defecto.
5. **Filas de la tabla de requisitos** sin enlazar: `bounded-queue-based`, `unbounded-deque-based`, `bounded-deque-based`, control-aware (acotado y no) y `logger-queue`, más las semánticas Deque, ControlAware y MultipleConsumer; necesitan sus mailboxes (Q-MBX-1 y relacionadas). Hoy sólo se enlaza `UnboundedMessageQueueSemantics`.
6. **Granularidad de detección del harness** (limitación 8): `rust_lines()` calcula todos los casos, así que 4 de los 11 mutantes nuevos (`mailbox-actor-requirement-ignored`, `-binding-ignored`, `-applied-to-every-actor`, `-without-binding-uses-the-default`) se detectan por caída de la familia mailbox (fast-kill en `mb-0001`) y no por el caso `MBX-REQ-*` que los motivó. Los otros 7 son detección específica.
7. **Slice 4** (no iniciado): `ProducesMessageQueue` y `verifyRequirements` en cada rama de `getMailboxType`; en guardianes, `lookup(default-mailbox)` y la verificación posterior a crear la cola (`ActorInitializationException`); ver `guardians.md`.
8. **Menores de API** (`q-mbx-13.md`): `Settings::mailbox_requirement` rompe literales de `Settings` sin `..Default`; `Debug` de `Mailboxes` no muestra los bindings; no hay `unbind_requirement`; `Display` de `MessageQueueSemantics` imprime siempre `interface`; `cargo semver-checks` entra en los gates desde el primer release.
