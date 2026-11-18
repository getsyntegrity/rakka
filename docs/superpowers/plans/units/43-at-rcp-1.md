# Spec 43 — AT-RCP (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 10. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 41 `AT-EXT-1` (`41-at-ext-1.md`), por las filas AT-EXT-002.1, AT-EXT-002.2, AT-EXT-002.3, AT-EXT-003.
- Spec 48 `AT-SER-1` (`48-at-ser-1.md`), por las filas AT-SER-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-006.
- Spec 44 `AT-RT-1` (`44-at-rt-1.md`), por las filas AT-RT-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-RCP-1`, `Q-T-RCP-2`, `Q-T-RCP-3`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Receptionist (extension) y seleccion local/cluster (+5 filas)

Filas: AT-RCP-001, AT-RCP-002, AT-RCP-003, AT-RCP-004, AT-RCP-005.1, AT-RCP-005.2. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Subscribe / Listing y Find: Listing: accesores y chequeo de clave (+3 filas)

Filas: AT-RCP-005.3, AT-RCP-006, AT-RCP-007, AT-RCP-008. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-RCP-001 | Receptionist (extension) y seleccion local/cluster | trait ReceptionistBehaviorProvider{name(),behavior()}; struct LocalReceptionist… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | Q-T-RCP-2, Q-X-2 |
| AT-RCP-002 | ServiceKey[T](id) | struct ServiceKey<M>{id:String, type_name:&'static str, _m:PhantomData<fn(M)>};… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/receptionist/ReceptionistAp… | Q-T-RCP-1 |
| AT-RCP-003 | Register / Registered | enum… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | Q-T-RCP-3 |
| AT-RCP-004 | Deregister / Deregistered | ReceptionistCommand::Deregister{key,service,reply_to}; misma estructura de estado que… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | Q-T-RCP-3 |
| AT-RCP-005.1 | Subscribe / Listing y Find: Find | enum ReceptionistCommand::{Find,Subscribe}; struct… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | Q-T-RCP-1, Q-T-RCP-3 |
| AT-RCP-005.2 | Subscribe / Listing y Find: Subscribe y Listing inicial/incremental | enum ReceptionistCommand::{Find,Subscribe}; struct… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | Q-T-RCP-1, Q-T-RCP-3 |
| AT-RCP-005.3 | Subscribe / Listing y Find: Listing: accesores y chequeo de clave | enum ReceptionistCommand::{Find,Subscribe}; struct… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | Q-T-RCP-1 |
| AT-RCP-006 | Desregistro automatico al terminar | Observacion de terminacion via watch_with(RegisteredActorTerminated) sobre el estado del… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/receptionist/Local… | - |
| AT-RCP-007 | ReceptionistSetup (reemplazo para tests) | ReceptionistSetup::new(\|system\| Receptionist) pasado al builder del sistema. | Sin test dedicado (el mecanismo comun lo cubre ExtensionsSpec) | - |
| AT-RCP-008 | Extractores ServiceKey.Listing / ServiceKey.Registered | ServiceKey::match_listing(&Listing)->Option<Set<ActorRef<M>>> y match_registered. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/RoutersSpec.scala:… | - |

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

Spec 44 `AT-RT-1` (`44-at-rt-1.md`).
