# Spec 33 — AT-STASH (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 6. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 25 `AT-BEH-1` (`25-at-beh-1.md`), por las filas AT-BEH-021.
- Spec 36 `AT-EVT-1` (`36-at-evt-1.md`), por las filas AT-EVT-002, AT-EVT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 34 `AT-SUP-1` (`34-at-sup-1.md`), por las filas AT-SUP-018. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-STASH-1`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: StashBuffer: creacion con capacidad (withStash) y StashOverflowException (+5 filas)

Filas: AT-STASH-001, AT-STASH-002, AT-STASH-003, AT-STASH-004, AT-STASH-005, AT-STASH-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-STASH-001 | StashBuffer: creacion con capacidad (withStash) y StashOverflowException | struct StashBuffer<M> { buf: VecDeque<M>, capacity } con stash() -> Result<(),… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.sc… | Q-X-5 |
| AT-STASH-002 | unstashAll (orden, stash durante unstash) | stash.unstash_all(behavior) -> Behavior<M>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.sc… | - |
| AT-STASH-003 | unstash(behavior, n, wrap) | stash.unstash(behavior, n, \|m\| wrap(m)). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.sc… | - |
| AT-STASH-004 | Inspeccion: isEmpty, nonEmpty, size, capacity, isFull, head, foreach,… | Metodos sobre StashBuffer; head -> Option<&M>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashBufferSpec.sc… | - |
| AT-STASH-005 | Semantica del unstash: same, unhandled, stopped, Behavior inicial | Logica en unstash_all. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:32… | Q-X-6 |
| AT-STASH-006 | UnstashException (falla durante unstash) | ActorError::Unstash. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala:47… | Q-T-STASH-1 |

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

Spec 34 `AT-SUP-1` (`34-at-sup-1.md`).
