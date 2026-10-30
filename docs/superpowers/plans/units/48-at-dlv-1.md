# Spec 48 — AT-DLV (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 7. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Ninguno entre specs de este plan.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-DLV-1`, `Q-T-DLV-2`, `Q-T-DLV-3`, `Q-T-SER-1`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ProducerController (punto a punto, RequestNext, RegisterConsumer, MessageWithConfirmation) (+5 filas)

Filas: AT-DLV-001, AT-DLV-002, AT-DLV-003, AT-DLV-004, AT-DLV-005, AT-DLV-006. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Flight recorder (JFR) de actor y delivery

Filas: AT-DLV-007. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-DLV-001 | ProducerController (punto a punto, RequestNext, RegisterConsumer,… | Behavior ProducerController<A> con mensajes… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1 |
| AT-DLV-002 | ConsumerController (Start, Delivery, Confirmed, ventana de flujo, deteccion de… | Behavior ConsumerController<A>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1 |
| AT-DLV-003 | WorkPullingProducerController (via Receptionist ServiceKey) | Behavior WorkPullingProducerController<A>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.sc… | Q-T-DLV-1 |
| AT-DLV-004 | DurableProducerQueue (protocolo) | trait DurableProducerQueue<A>; struct DurableState<A>; implementacion en memoria… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQue… | Q-T-DLV-2 |
| AT-DLV-005 | Chunking de mensajes grandes | Requiere serializacion de mensajes (Q-T-SER-1). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-SER-1 |
| AT-DLV-006 | Configuracion pekko.reliable-delivery.* y Settings | structs ProducerSettings/ConsumerSettings/WorkPullingSettings con Default y from_config. | Sin test leido que afirme los defaults | - |
| AT-DLV-007 | Flight recorder (JFR) de actor y delivery | Eventos tracing equivalentes (Q-T-DLV-3). | Sin test | Q-T-DLV-3 |

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

Spec 49 `AT-JAPI-1` (`49-at-japi-1.md`).
