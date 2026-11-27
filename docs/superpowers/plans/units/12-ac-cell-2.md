# Spec 12 — AC-CELL (2/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 8. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-CELL-012.

## Prerrequisitos

- Spec 11 `AC-CELL-1` (`11-ac-cell-1.md`), por las filas AC-CELL-002.2, AC-CELL-004, AC-CELL-008.2, AC-CELL-013.3.
- Spec 09 `AC-EVT-1` (`09-ac-evt-1.md`), por las filas AC-EVT-010.
- Spec 06 `AC-MBX-1` (`06-ac-mbx-1.md`), por las filas AC-MBX-001, AC-MBX-016.
- Spec 04 `AC-REF-1` (`04-ac-ref-1.md`), por las filas AC-REF-002, AC-REF-018, AC-REF-019, AC-REF-020.
- Spec 21 `AC-SER-1` (`21-ac-ser-1.md`), por las filas AC-SER-001.1, AC-SER-001.2, AC-SER-001.3, AC-SER-008.1. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CELL-5`, `Q-CELL-9`, `Q-REF-5`, `Q-REF-6`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Limpieza de campos al terminar (props=terminatedProps, instancia nula, unstashAll) (+5 filas)

Filas: AC-CELL-024, AC-CELL-025, AC-CELL-026, AC-CELL-027, AC-CELL-028, AC-CELL-029. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Errores al enviar (suspend/resume/restart/stop/sendMessage/sendSystemMessage) se tragan… (+1 filas)

Filas: AC-CELL-030, AC-CELL-031. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-CELL-024 | Limpieza de campos al terminar (props=terminatedProps, instancia nula,… | la celda libera la fábrica y la instancia al terminar | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:108-128 | - |
| AC-CELL-025 | pekko.actor.debug.unhandled: reenviador de UnhandledMessage a Debug | suscriptor interno de `UnhandledMessage` que publica un Debug | actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala:137-153 | - |
| AC-CELL-026 | Validación del tipo de mailbox exigido por el actor en Create | comprobación de `RequiresMessageQueue` contra el tipo de cola del mailbox | derivada del código (ver bloque de contrato) | - |
| AC-CELL-027 | Fallo al crear la celda (initWithFailure) y punto de creación diferida | celda "fallida" que sólo entrega ActorInitializationError | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:510-533 | Q-CELL-9, Q-REF-5 |
| AC-CELL-028 | pekko.actor.serialize-messages: ida y vuelta de mensajes en modo test | hook de test que serializa y deserializa cada mensaje enviado (feature opcional) | actor-tests/src/test/scala/org/apache/pekko/actor/FunctionRefSpec.scala:108-120 ;… | Q-CELL-5 |
| AC-CELL-029 | pekko.actor.serialize-creators: verificación de argumentos de Props al crear… | hook de test que serializa los argumentos de Props (feature opcional) | actor-tests/src/test/scala/org/apache/pekko/actor/PropsCreationSpec.scala:44-73 ;… | Q-CELL-5 |
| AC-CELL-030 | Errores al enviar (suspend/resume/restart/stop/sendMessage/sendSystemMessage)… | `catch_unwind`/Result en el envío con log Error | actor-tests/src/test/scala/org/apache/pekko/actor/dungeon/DispatchSpec.scala:30-41 | Q-CELL-9 |
| AC-CELL-031 | Cell / ActorRefWithCell: interfaz interna de la celda (underlying, children,… | trait Cell pub(crate)… | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:847-862… | Q-REF-6 |

## Gates y criterio de cierre

```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --lcov --output-path target/lcov.info   # cobertura de líneas nuevas o modificadas >= 90 %; reportar comando, resultado y líneas no cubiertas con motivo
```

La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.

## Siguiente

Spec 13 `AC-SYS-1` (`13-ac-sys-1.md`).
