# Spec 37 — AT-ADP (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 13. **Tareas:** 3 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 33 `AT-SYS-1` (`33-at-sys-1.md`), por las filas AT-SYS-003, AT-SYS-004, AT-SYS-011.
- Spec 36 `AT-SUP-1` (`36-at-sup-1.md`), por las filas AT-SUP-001, AT-SUP-005, AT-SUP-020, AT-SUP-021.
- Spec 40 `AT-DSP-1` (`40-at-dsp-1.md`), por las filas AT-DSP-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-005, AT-CTX-008, AT-CTX-011, AT-CTX-014.
- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-003, AT-BEH-004, AT-BEH-007, AT-BEH-011, AT-BEH-012, AT-BEH-024.
- Spec 38 `AT-EVT-1` (`38-at-evt-1.md`), por las filas AT-EVT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 41 `AT-EXT-1` (`41-at-ext-1.md`), por las filas AT-EXT-002.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 47 `AT-CS-1` (`47-at-cs-1.md`), por las filas AT-CS-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-ADP-2`, `Q-T-ADP-3`, `Q-T-SYS-3`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Sistema clasico a typed: toTyped, ActorSystem.wrap; typed a clasico: toClassic (+5 filas)

Filas: AT-ADP-001, AT-ADP-002, AT-ADP-003, AT-ADP-004, AT-ADP-005, AT-ADP-006.1. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: ActorAdapter: Resultado del Behavior (next): unhandled, failed, stopped (+5 filas)

Filas: AT-ADP-006.2, AT-ADP-006.3, AT-ADP-006.4, AT-ADP-007, AT-ADP-008, AT-ADP-009. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Aislamiento del guardian: user guardian detenido termina el sistema…

Filas: AT-ADP-010. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-ADP-001 | Sistema clasico a typed: toTyped, ActorSystem.wrap; typed a clasico: toClassic | trait ClassicInterop{to_typed(&ClassicSystem)->TypedSystem;… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-X-8 |
| AT-ADP-002 | spawn/spawnAnonymous de typed desde sistema o contexto clasico | ClassicCompat::spawn(behavior, props) con supervision Stop por defecto. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-T-ADP-3, Q-X-8 |
| AT-ADP-003 | PropsAdapter (typed como Props clasico) | fn to_classic_props(behavior, props) -> ClassicProps (solo en capa de compat). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/adpater/PropsAdapt… | Q-X-8 |
| AT-ADP-004 | Conversion de ActorRef, ActorContext y Scheduler (toClassic/toTyped) | ActorRef<T>::from_classic / to_classic; Scheduler::to_classic. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-X-8 |
| AT-ADP-005 | actorOf de hijos clasicos desde contexto typed; watch y stop cruzados | ClassicCompat::actor_of desde typed; watch cruzado. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-X-8 |
| AT-ADP-006.1 | ActorAdapter: Despacho de mensajes (aroundReceive) | bucle de actor propio con un enum de sobre interno `Envelope<M> { Msg(M), Signal(S),… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:100-128 ;… | Q-T-ADP-2, Q-T-ADP-3, Q-X-8 |
| AT-ADP-006.2 | ActorAdapter: Resultado del Behavior (next): unhandled, failed, stopped | `fn handle_next(Behavior) -> ActorAction { Continue, Unhandled, Fail(cause),… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorContextDelega… | Q-T-ADP-2, Q-T-ADP-3, Q-X-8 |
| AT-ADP-006.3 | ActorAdapter: Hooks de ciclo de vida: preStart, preRestart, postRestart,… | hooks `on_start`/`on_pre_restart`/`on_post_restart`/`on_stop` del ciclo de vida del… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervis… | Q-T-ADP-2, Q-T-ADP-3, Q-X-8 |
| AT-ADP-006.4 | ActorAdapter: Supervision de hijos y registro de fallos (ChildFailed) | tabla `ref_hijo -> causa` consumida al procesar la terminación del hijo; decisor Stop… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-T-ADP-2, Q-T-ADP-3, Q-X-8 |
| AT-ADP-007 | Despliegue remoto prohibido para actores typed | n/a bajo alcance sin remoting: Deploy::Local fijo. | Sin test dedicado | - |
| AT-ADP-008 | toTyped es idempotente: una sola ActorSystemAdapter por sistema clasico | Singleton por sistema. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:205-23… | Q-X-8 |
| AT-ADP-009 | ActorRefAdapter: mensajes de sistema y rechazo de null | Mapeo de mensajes de sistema de la capa de compat. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-X-8 |
| AT-ADP-010 | Aislamiento del guardian: user guardian detenido termina el sistema… | en Rust el sistema puede construirse completo antes de arrancar el guardián, haciendo… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-T-ADP-2, Q-T-SYS-3, Q-X-8 |

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

Spec 38 `AT-EVT-1` (`38-at-evt-1.md`).
