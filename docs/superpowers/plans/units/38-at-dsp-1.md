# Spec 38 — AT-DSP (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 5. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 30 `AT-CTX-1` (`30-at-ctx-1.md`), por las filas AT-CTX-002.
- Spec 37 `AT-LOG-1` (`37-at-log-1.md`), por las filas AT-LOG-002.
- Spec 31 `AT-SYS-1` (`31-at-sys-1.md`), por las filas AT-SYS-007.4.
- Spec 29 `AT-TK-2` (`29-at-tk-2.md`), por las filas AT-TK-011.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-DSP-1`, `Q-T-DSP-2`, `Q-T-DSP-3`, `Q-T-LOG-1`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Props tipado (empty, encadenado, firstOrElse, allOf) (+4 filas)

Filas: AT-DSP-001, AT-DSP-002, AT-DSP-003, AT-DSP-004, AT-DSP-005. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-DSP-001 | Props tipado (empty, encadenado, firstOrElse, allOf) | struct Props{selectors: Vec<Selector>} con builder; first_of::<T>() y all_of::<T>(). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/PropsSpec.scala:21-32 | Q-T-DSP-1 |
| AT-DSP-002 | DispatcherSelector: default, blocking, fromConfig, sameAsParent | DispatcherSelector::{default,blocking,from_config(path),same_as_parent}; mapeo a pools… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelector… | Q-T-DSP-2, Q-X-4 |
| AT-DSP-003 | MailboxSelector: default (SingleConsumerOnlyUnboundedMailbox),… | MailboxSelector::{Default, Bounded(n), FromConfig(path)}. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/MailboxSelectorSpec.scala:3… | Q-T-DSP-3 |
| AT-DSP-004 | ActorTags | `ActorTags(Vec<String>)` como variante de `Props` con `ActorTags::new(impl… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.s… | Q-T-DSP-1, Q-T-LOG-1 |
| AT-DSP-005 | Dispatchers.lookup y ids (DefaultDispatcherId, InternalDispatcherId) | `Dispatchers::lookup(&DispatcherSelector) -> Executor` con la semántica anterior;… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/DispatcherSelector… | Q-T-DSP-2, Q-X-4 |

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

Spec 39 `AT-EXT-1` (`39-at-ext-1.md`).
