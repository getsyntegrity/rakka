# Paridad con Pekko 2.0.0 de Deploy y configuración (Slice 2, PR2b)

Oráculo `MBX-CFG-04` (`mb-0023`, JDK 17, estable en 3 corridas) y gemelo Rust; replay 2967/2967. Cada expectativa se derivó del Scala antes de ejecutar el oráculo (`ActorRefProvider.scala:692-724`, `Children.scala:316-336`, `Deployer.scala:252,274`, `Props.scala:190`).

| # | Contrato | Dónde se prueba |
|---|---|---|
| 1 | Deploy con mailbox no vacío prevalece sobre Props | `d-lifo` (Props `cap4`, Deploy `lifo`): cola LIFO |
| 2 | Deploy vacío conserva Props | `d-empty`: cola de capacidad 4 |
| 3 | Deploy válido prevalece sobre un id inválido de Props | `d-lifo-b` (Props `never-registered`): se crea, LIFO |
| 4 | Deploy inválido falla aunque Props sea válido | `d-bad`: `ConfigurationException` |
| 5 | Sin coincidencia se conserva Props | `other`: capacidad 4 |
| 6 | Path completo del hijo | `kid` bajo `/d-parent`: `/d-parent/kid` aplica, el señuelo `/kid` (id `typo`) no |
| 7 | Selección por `pekko.actor.deployment` | todo el caso; el gemelo la lee con `Settings::from_config` |
| 8 | El error nombra el mailbox efectivo | mensaje y causa con `[typo]`, no el de Props |
| 9 | Creación fallida y nombre | segundo `actorOf` de `d-bad` falla igual (no `InvalidActorName`): el nombre se liberó |
| 10 | Default registrado sin ninguna fuente | `MBX-CFG-02` (#350) |
| 11 | Configuración inválida | test `an_invalid_deployment_in_the_configuration_makes_the_runtime_settings_fail` (texto de `dp-*`) |

Mutantes (regresión de 11 + control 2967/2967, y los 2 de path de este PR; todos DETECTED):
- Específica, por el unitario esperado: `deployment-mailbox-ignored`, `deployment-empty-mailbox-resets-the-props`, `props-mailbox-beats-the-deployment`.
- Específica, por un caso: `mailbox-id-error-message-names-the-default-id` (`mb-0020`), `mailbox-registered-default-ignored-without-a-selection` (`mb-0022`).
- Por caída de la familia mailbox (limitación 8 del harness): los otros 6 de #346/#347 (`mb-0001`) y los 2 de este PR, `deployment-looked-up-by-the-path-of-the-parent` y `-by-the-name-only` (23 casos divergen).
- Manual, contrato 11: `Deployer::from_settings(config).unwrap_or_default()` en `Settings::from_config` hace fallar solo `an_invalid_deployment_in_the_configuration_makes_the_runtime_settings_fail` (1 de 16); revertido.

Con esta evidencia la tarea 2 de Q-MBX-12 queda resuelta para `Deploy.mailbox` y configuración; el id de dispatcher del error sigue fijo (tarea 4). `AC-MBX-014` sigue `partial`.
