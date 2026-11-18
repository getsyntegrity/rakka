# Spec 32 — AT-SYS (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 15. **Tareas:** 3 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 36 `AT-ADP-1` (`36-at-adp-1.md`), por las filas AT-ADP-006.1, AT-ADP-006.2, AT-ADP-006.3, AT-ADP-006.4. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 28 `AT-REF-1` (`28-at-ref-1.md`), por las filas AT-REF-001.
- Spec 34 `AT-STASH-1` (`34-at-stash-1.md`), por las filas AT-STASH-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 46 `AT-CS-1` (`46-at-cs-1.md`), por las filas AT-CS-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 38 `AT-LOG-1` (`38-at-log-1.md`), por las filas AT-LOG-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 37 `AT-EVT-1` (`37-at-evt-1.md`), por las filas AT-EVT-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-SYS-1`, `Q-T-SYS-2`, `Q-T-SYS-3`, `Q-T-SYS-4`, `Q-T-TMR-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ActorSystem.apply/create (guardian, nombre, config, guardianProps, setups) (+5 filas)

Filas: AT-SYS-001, AT-SYS-002, AT-SYS-003, AT-SYS-004, AT-SYS-005, AT-SYS-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: ActorSystem.name y address (+5 filas)

Filas: AT-SYS-007.1, AT-SYS-007.2, AT-SYS-007.3, AT-SYS-007.4, AT-SYS-008, AT-SYS-009. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: deadLetters (+2 filas)

Filas: AT-SYS-010, AT-SYS-011, AT-SYS-012. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-SYS-001 | ActorSystem.apply/create (guardian, nombre, config, guardianProps, setups) | ActorSystem::<M>::new(guardian: Behavior<M>, name, config: Config) ->… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | Q-X-1, Q-X-8 |
| AT-SYS-002 | ActorSystem como ActorRef[T] del guardian | impl Deref<Target=ActorRef<M>> o metodo system.tell(msg). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | - |
| AT-SYS-003 | Arranque del guardian (GuardianStartupBehavior) | En Rust el sistema puede construirse completo antes de arrancar el guardian, haciendo… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/GuardianSt… | Q-T-SYS-3 |
| AT-SYS-004 | Parada del guardian termina el sistema (GuardianStopInterceptor) | Interceptor interno sobre el guardian que dispara terminate. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | - |
| AT-SYS-005 | terminate / whenTerminated / getWhenTerminated / close | system.terminate(); system.when_terminated() -> impl Future<Output=()>; Drop o close()… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | Q-T-SYS-4 |
| AT-SYS-006 | Settings tipados y restart-stash-capacity | struct TypedSettings { restart_stash_capacity: usize, .. } desde la config. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:888-9… | Q-T-SYS-2 |
| AT-SYS-007.1 | ActorSystem.name y address | system.name() -> &str; system.address() -> Address. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | - |
| AT-SYS-007.2 | ActorSystem.startTime y uptime | system.start_time() (ms desde epoch) y system.uptime() (segundos enteros) con reloj… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | Q-T-TMR-4 |
| AT-SYS-007.3 | ActorSystem.log, logConfiguration y printTree | system.log() (target tracing sin span de actor), system.log_configuration() (INFO con… | No hay spec que afirme el comportamiento. Usos de diagnostico:… | Q-T-SYS-1, Q-T-SYS-2 |
| AT-SYS-007.4 | ActorSystem.threadFactory, dynamicAccess y executionContext | thread_factory/dynamic_access segun Q-T-SYS-1; system.executor() -> handle del runtime. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/internal/ActorSystemSpec.sc… | Q-T-SYS-1, Q-X-4 |
| AT-SYS-008 | systemActorOf | system.system_actor_of(behavior, name, props). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/coexistence/ClassicSupervis… | - |
| AT-SYS-009 | Scheduler tipado (scheduleOnce, scheduleWithFixedDelay, scheduleAtFixedRate) | Trait Scheduler sobre el timer del runtime. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SchedulerSpec.scala:19-27… | Q-T-TMR-4 |
| AT-SYS-010 | deadLetters | system.dead_letters::<M>(). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MessageAdapterSpec… | Q-X-6 |
| AT-SYS-011 | ActorSystemAdapter / AdapterExtension (sistema tipado sobre el clasico) | En Rust: decidir si existe un runtime clasico debajo (porte en capas) o un runtime… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/adapter/AdapterSpe… | Q-X-8 |
| AT-SYS-012 | Settings.toString redacta valores sensibles | impl Display para Settings que aplica una funcion redact(config). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorSystemSpec.scala:29-48 | Q-T-SYS-2, Q-X-1 |

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

Spec 33 `AT-TMR-1` (`33-at-tmr-1.md`).
