# Spec 21 — AC-RT (1/1)

**Estado:** bloqueada. **Filas:** 22. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 06 `AC-REF-1` (`06-ac-ref-1.md`), por las filas AC-REF-002.
- Spec 19 `AC-ASK-1` (`19-ac-ask-1.md`), por las filas AC-ASK-001.1, AC-ASK-006.
- Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`), por las filas AC-SUP-001.
- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-001.
- Spec 17 `AC-DW-1` (`17-ac-dw-1.md`), por las filas AC-DW-001.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-012.1, AC-CFG-012.2, AC-CFG-012.3, AC-CFG-012.4, AC-CFG-014.
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-019.
- Spec 23 `AC-SER-1` (`23-ac-ser-1.md`), por las filas AC-SER-001.1, AC-SER-001.2, AC-SER-001.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 10 `AC-SCH-1` (`10-ac-sch-1.md`), por las filas AC-SCH-001.
- Spec 09 `AC-DSP-1` (`09-ac-dsp-1.md`), por las filas AC-DSP-012.

Decisiones:

- Bloqueantes: `Q-RT-4`.
- Propuestas aún sin decisión del Owner: `Q-RT-1`, `Q-RT-2`, `Q-RT-3`, `Q-RT-5`, `Q-RT-6`, `Q-RT-7`, `Q-RT-8`, `Q-RT-9`, `Q-X-1`, `Q-X-2`, `Q-X-3`, `Q-X-4`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Router (valor inmutable), RoutingLogic, Routee (ActorRefRoutee, ActorSelectionRoutee,… (+5 filas)

Filas: AC-RT-001, AC-RT-002, AC-RT-003.1, AC-RT-003.2, AC-RT-003.3, AC-RT-004. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: RoundRobin (logic, pool, group) (+5 filas)

Filas: AC-RT-005, AC-RT-006, AC-RT-007, AC-RT-008, AC-RT-009.1, AC-RT-009.2. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: MurmurHash (variante Pekko): startHash/extendHash/finalizeHash, arrayHash, stringHash,… (+5 filas)

Filas: AC-RT-009.3, AC-RT-010, AC-RT-011, AC-RT-012, AC-RT-013.1, AC-RT-013.2. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Resizer.fromConfig: seleccion DefaultResizer vs OptimalSizeExploringResizer, exclusion… (+3 filas)

Filas: AC-RT-013.3, AC-RT-014, AC-RT-015, AC-RT-016. Contratos y aserciones: `docs/inventory/contracts/classic-patterns.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-RT-001 | Router (valor inmutable), RoutingLogic, Routee (ActorRefRoutee,… | `struct Router { logic: Arc<dyn RoutingLogic>, routees: Vec<Routee> }`, `enum Routee {… | actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:163-210 ;… | Q-RT-1 |
| AC-RT-002 | RouterConfig / Pool / Group / CustomRouterConfig; estrategia por defecto del… | trait `RouterConfig`, `trait Pool: RouterConfig`, `trait Group: RouterConfig`, `fn… | actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:147-204 ;… | Q-RT-1, Q-X-5 |
| AC-RT-003.1 | Actor router (RoutedActorCell/RoutedActorRef): ruteo en el hilo del emisor,… | `RoutedActorCell`: `ArcSwap<Router>` + enrutamiento en `tell` sin pasar por mailbox | actor-tests/src/test/scala/org/apache/pekko/routing/RouteeCreationSpec.scala:29-59 ;… | Q-RT-2, Q-X-4 |
| AC-RT-003.2 | Mensajes de gestion del router: GetRoutees, AddRoutee, RemoveRoutee,… | enum `RouterManagement { GetRoutees, AddRoutee(Routee), RemoveRoutee(Routee),… | actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:111-127 ;… | Q-RT-2 |
| AC-RT-003.3 | Routees terminados: evictarlos y detener el router al quedar vacio (salvo con… | RouterActor recibe Terminated(ref) -> remove_routee y `stop_if_all_removed` | actor-tests/src/test/scala/org/apache/pekko/routing/RoutingSpec.scala:66-110 | Q-RT-2 |
| AC-RT-004 | FromConfig / NoRouter y precedencia deployment vs codigo | `FromConfig`, `NoRouter` como RouterConfig; `Deployer::lookup(path)` +… | actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:141-… | Q-RT-3, Q-X-1 |
| AC-RT-005 | RoundRobin (logic, pool, group) | `RoundRobinRoutingLogic { next: AtomicU64 }` + `RoundRobinPool`/`RoundRobinGroup` | actor-tests/src/test/scala/org/apache/pekko/routing/RoundRobinSpec.scala:34-210 | Q-RT-1 |
| AC-RT-006 | Random (logic, pool, group) | `RandomRoutingLogic` con `rand::thread_rng()` (rng inyectable para tests) | actor-tests/src/test/scala/org/apache/pekko/routing/RandomSpec.scala:28-112 | Q-RT-1 |
| AC-RT-007 | Broadcast (pool, group) | `BroadcastRoutingLogic`; pool y group | actor-tests/src/test/scala/org/apache/pekko/routing/BroadcastSpec.scala:33-95 | Q-RT-1 |
| AC-RT-008 | SmallestMailbox (pool) | `SmallestMailboxRoutingLogic` que usa `ActorCellView { is_processing, has_messages,… | actor-tests/src/test/scala/org/apache/pekko/routing/SmallestMailboxSpec.scala:26-75 | Q-RT-4 |
| AC-RT-009.1 | ConsistentHashing: seleccion por clave (hashMapping, ConsistentHashable,… | `ConsistentHashingRoutingLogic` con clave `HashKey { Bytes(Vec<u8>), Str(String),… | actor-tests/src/test/scala/org/apache/pekko/routing/ConsistentHashingRouterSpec.scala:76-… | Q-RT-5, Q-X-3 |
| AC-RT-009.2 | ConsistentHash (anillo con nodos virtuales) | `ConsistentHash<T>` con `BTreeMap<i32, T>` y `nodeFor(&[u8]\|&str)` | - (sin spec dedicado; cubierto indirectamente por… | Q-RT-5 |
| AC-RT-009.3 | MurmurHash (variante Pekko): startHash/extendHash/finalizeHash, arrayHash,… | modulo `murmur_hash` con las mismas constantes y aritmetica `wrapping` i32 | - (sin spec dedicado en actor-tests) | Q-RT-5 |
| AC-RT-010 | ScatterGatherFirstCompleted (pool, group, within) | `ScatterGatherFirstCompletedRoutingLogic { within }`; pool/group | actor-tests/src/test/scala/org/apache/pekko/routing/ScatterGatherFirstCompletedSpec.scala… | Q-RT-6 |
| AC-RT-011 | TailChopping (pool, group, interval, within) | `TailChoppingRoutingLogic { within, interval }` con scheduler | actor-tests/src/test/scala/org/apache/pekko/routing/TailChoppingSpec.scala:58-135 | Q-RT-6 |
| AC-RT-012 | BalancingPool | `BalancingPool` + dispatcher de cola compartida (work-stealing/mailbox comun) | actor-tests/src/test/scala/org/apache/pekko/routing/BalancingSpec.scala:133-178… | Q-RT-7 |
| AC-RT-013.1 | DefaultResizer: capacidad (rampup/backoff), cotas y validaciones | `DefaultResizer { lower_bound, upper_bound, pressure_threshold, rampup_rate,… | actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:104-135 | Q-RT-4 |
| AC-RT-013.2 | DefaultResizer: pressure (routees ocupados) y mecanica de resize del pool… | `ResizablePoolCell` con `AtomicU64` contador y `AtomicBool` resize_in_progress | actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:138-254 | Q-RT-4 |
| AC-RT-013.3 | Resizer.fromConfig: seleccion DefaultResizer vs OptimalSizeExploringResizer,… | `Resizer::from_config(&Config) -> Result<Option<Resizer>, ResizerInitError>` | actor-tests/src/test/scala/org/apache/pekko/routing/ResizerSpec.scala:61-102 | Q-RT-3, Q-X-1 |
| AC-RT-014 | OptimalSizeExploringResizer | OptimalSizeExploringResizer con traits inyectables RouteeLoad, Clock y Rng;… | actor-tests/src/test/scala/org/apache/pekko/routing/MetricsBasedResizerSpec.scala:89-111,… | Q-RT-4, Q-RT-9, Q-X-1 |
| AC-RT-015 | Listeners / Listen / Deafen / WithListeners | trait `Listeners { fn listeners(&mut self) -> &mut BTreeSet<ActorRef>; fn gossip(&self,… | actor-tests/src/test/scala/org/apache/pekko/actor/routing/ListenerSpec.scala:29-70 | Q-RT-8 |
| AC-RT-016 | Configuracion de deployment de routers (router.type-mapping, nr-of-instances,… | registro de fabricas por nombre (`router.type-mapping`) y `fn from_config(&Config)` por… | actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:139-… | Q-RT-3, Q-X-2 |

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

Spec 22 `AC-CS-1` (`22-ac-cs-1.md`).
