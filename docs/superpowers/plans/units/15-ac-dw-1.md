# Spec 15 — AC-DW (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 8. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-DW-001, AC-DW-003, AC-DW-004, AC-DW-008, AC-DW-009.

## Prerrequisitos

- Spec 04 `AC-REF-1` (`04-ac-ref-1.md`), por las filas AC-REF-015.
- Spec 11 `AC-CELL-1` (`11-ac-cell-1.md`), por las filas AC-CELL-004, AC-CELL-013.1, AC-CELL-014.
- Spec 14 `AC-SUP-1` (`14-ac-sup-1.md`), por las filas AC-SUP-004.
- Spec 16 `AC-EXT-1` (`16-ac-ext-1.md`), por las filas AC-EXT-001.1, AC-EXT-001.2. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 06 `AC-MBX-1` (`06-ac-mbx-1.md`), por las filas AC-MBX-016.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-DW-1`, `Q-DW-2`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: watch sobre un actor ya muerto → Terminated inmediato (+5 filas)

Filas: AC-DW-002, AC-DW-005, AC-DW-006, AC-DW-007, AC-DW-010, AC-DW-011. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: AddressTerminated y AddressTerminatedTopic (watch de refs remotos) (+1 filas)

Filas: AC-DW-012, AC-DW-013. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-DW-002 | watch sobre un actor ya muerto → Terminated inmediato | Watch a ref terminado responde DWN | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:144-151 | Q-DW-1 |
| AC-DW-005 | watchWith(ref, msg) con mensaje custom | `ctx.watch_with(&ActorRef, msg)` | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:292-315 | Q-DW-3, Q-DW-4 |
| AC-DW-006 | Mensaje Terminated (actor, existenceConfirmed, addressTerminated) | `struct Terminated { actor, existence_confirmed, address_terminated }` con `PartialEq`… | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:129-132 ;… | Q-DW-1 |
| AC-DW-007 | Death pact: Terminated no manejado → DeathPactException → Stop | - | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:215-245 | - |
| AC-DW-010 | Notificación a watchers al morir (remotos primero, luego locales; el padre se… | - | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:153-166 | Q-DW-2 |
| AC-DW-011 | Protocolo Watch/Unwatch de sistema (addWatcher/remWatcher) | - | actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:231-253 | - |
| AC-DW-012 | AddressTerminated y AddressTerminatedTopic (watch de refs remotos) | hook en el trait de ref (`is_local`) + topic opcional | actor-tests/src/test/scala/org/apache/pekko/event/AddressTerminatedTopicBenchSpec.scala:2… | Q-DW-2 |
| AC-DW-013 | Interacción con Stash: Terminated desestachado vuelve a ser entregable | - | actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithStashSpec.scala:83-98,186-190… | - |

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

Spec 16 `AC-EXT-1` (`16-ac-ext-1.md`).
