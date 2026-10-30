# Spec 37 — AT-EVT (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 4. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-003.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-EVT-1`, `Q-X-6`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: EventStream tipado: Publish / Subscribe / Unsubscribe (+3 filas)

Filas: AT-EVT-001, AT-EVT-002, AT-EVT-003, AT-EVT-004. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-EVT-001 | EventStream tipado: Publish / Subscribe / Unsubscribe | trait EventBus{publish<E:Any>(E); subscribe<E>(ActorRef<E>); unsubscribe}; Q-X-6 define… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec… | Q-X-6 |
| AT-EVT-002 | Dead letters (DeadLetter) y suscripcion via message adapter | Suscripcion de un actor a DeadLetter mediante adaptador. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/eventstream/EventStreamSpec… | - |
| AT-EVT-003 | UnhandledMessage publicado al quedar un mensaje sin manejar | Evento UnhandledMessage{message,sender,recipient}. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelega… | Q-T-EVT-1 |
| AT-EVT-004 | Dropped (stash de supervision lleno, group router sin routees, Topic sin… | fn publish_dropped(system, msg, reason, sender) con un unico camino; la variante Topic… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:882-9… | Q-T-EVT-1 |

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

Spec 38 `AT-LOG-1` (`38-at-log-1.md`).
