# Spec 44 — AT-RT (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 8. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-023.
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-002, AT-CTX-005.
- Spec 40 `AT-DSP-1` (`40-at-dsp-1.md`), por las filas AT-DSP-001, AT-DSP-002.
- Spec 43 `AT-RCP-1` (`43-at-rcp-1.md`), por las filas AT-RCP-005.2, AT-RCP-005.3.
- Spec 38 `AT-EVT-1` (`38-at-evt-1.md`), por las filas AT-EVT-004.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-EVT-1`, `Q-T-RT-1`, `Q-T-RT-2`, `Q-T-RT-3`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Routers.pool(poolSize)(behavior) (+5 filas)

Filas: AT-RT-001, AT-RT-002.1, AT-RT-002.2, AT-RT-002.3, AT-RT-003, AT-RT-004. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Group: preferLocalRoutees y manejo de inalcanzables (+1 filas)

Filas: AT-RT-005, AT-RT-006. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-RT-001 | Routers.pool(poolSize)(behavior) | fn pool<M>(size, behavior_factory) -> PoolRouter<M>; el Behavior de las rutas debe ser… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:… | Q-T-RT-3 |
| AT-RT-002.1 | Estrategia de ruteo: Round robin (por defecto de pool) | trait RoutingLogic<M>{select_routee(&M)->Option<ActorRef<M>>; routees_updated(set)};… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLog… | Q-T-RT-1, Q-T-RT-2 |
| AT-RT-002.2 | Estrategia de ruteo: Random (por defecto de group) | trait RoutingLogic<M>{select_routee(&M)->Option<ActorRef<M>>; routees_updated(set)};… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLog… | Q-T-RT-1, Q-T-RT-2 |
| AT-RT-002.3 | Estrategia de ruteo: Consistent hashing | trait RoutingLogic<M>{select_routee(&M)->Option<ActorRef<M>>; routees_updated(set)};… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLog… | Q-T-RT-1, Q-T-RT-2 |
| AT-RT-003 | Pool: withBroadcastPredicate, withPoolSize, withRouteeProps | PoolRouter::with_broadcast_predicate(Fn(&M)->bool), with_pool_size,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:… | Q-T-RT-3 |
| AT-RT-004 | Routers.group(serviceKey) | fn group<M>(key:ServiceKey<M>) -> GroupRouter<M>; internamente suscripcion a Listing y… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:… | Q-T-EVT-1, Q-T-RT-3 |
| AT-RT-005 | Group: preferLocalRoutees y manejo de inalcanzables | GroupRouter::prefer_local_routees(bool). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:… | Q-T-RT-3 |
| AT-RT-006 | Validacion e inmutabilidad de los builders de router | builders con consumo por valor (self -> Self); validar tamano en with_pool_size. | Sin test dedicado (los specs de routers no cubren poolSize invalido) | - |

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

Spec 45 `AT-PS-1` (`45-at-ps-1.md`).
