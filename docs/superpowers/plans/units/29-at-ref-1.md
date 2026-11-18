# Spec 29 — AT-REF (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 7. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 48 `AT-SER-1` (`48-at-ser-1.md`), por las filas AT-SER-001, AT-SER-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-022.
- Spec 42 `AT-ASK-1` (`42-at-ask-1.md`), por las filas AT-ASK-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 41 `AT-EXT-1` (`41-at-ext-1.md`), por las filas AT-EXT-001.1, AT-EXT-001.2, AT-EXT-001.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-BEH-4`, `Q-X-3`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ActorRef[-T]: tell / ! , contravariante, Comparable, Serializable (+5 filas)

Filas: AT-REF-001, AT-REF-002, AT-REF-003, AT-REF-004, AT-REF-005, AT-REF-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: ActorRefResolver (extension) y ActorRefResolverSetup

Filas: AT-REF-007. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-REF-001 | ActorRef[-T]: tell / ! , contravariante, Comparable, Serializable | struct ActorRef<M: Send + 'static> { inner: Arc<dyn RefInner> } con tell(&self, msg: M);… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-T-BEH-4 |
| AT-REF-002 | Rechazo de mensajes null | No aplica: Rust no tiene null; el tipo M lo garantiza. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:662-… | - |
| AT-REF-003 | narrow / unsafeUpcast | narrow: ActorRef<Super>::narrow::<Sub>() que envuelve con From<Sub> for Super (costo de… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:324… | Q-T-BEH-4, Q-X-3 |
| AT-REF-004 | RecipientRef[-T] | trait RecipientRef<M> { fn tell(&self, msg: M); } | sin spec propio en este modulo; se usa en… | - |
| AT-REF-005 | Igualdad, hash y path de ActorRef | impl PartialEq/Eq/Hash/Ord para ActorRef<M> delegando en path y uid. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:535-… | Q-T-BEH-4 |
| AT-REF-006 | ignoreRef | system.ignore_ref::<M>() -> ActorRef<M>. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefIgnoreSpec.scala:71… | - |
| AT-REF-007 | ActorRefResolver (extension) y ActorRefResolverSetup | Servicio ActorRefResolver con to_serialization_format/resolve_actor_ref. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorRefResolverSpec.scala:… | - |

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

Spec 30 `AT-TK-1` (`30-at-tk-1.md`).
