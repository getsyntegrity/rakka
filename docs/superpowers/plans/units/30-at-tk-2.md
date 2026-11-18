# Spec 30 — AT-TK (2/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 12. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 29 `AT-TK-1` (`29-at-tk-1.md`), por las filas AT-TK-001.1, AT-TK-001.2, AT-TK-001.3, AT-TK-002.1, AT-TK-003.1, AT-TK-003.2, AT-TK-006.4.
- Spec 09 `AC-SCH-1` (`09-ac-sch-1.md`), por las filas AC-SCH-010.
- Spec 11 `AC-TK-1` (`11-ac-tk-1.md`), por las filas AC-TK-008.
- Spec 32 `AT-SYS-1` (`32-at-sys-1.md`), por las filas AT-SYS-001, AT-SYS-005, AT-SYS-009. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 33 `AT-TMR-1` (`33-at-tmr-1.md`), por las filas AT-TMR-002, AT-TMR-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 37 `AT-EVT-1` (`37-at-evt-1.md`), por las filas AT-EVT-001, AT-EVT-002, AT-EVT-003, AT-EVT-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 22 `AC-SER-1` (`22-ac-ser-1.md`), por las filas AC-SER-001.1, AC-SER-001.3, AC-SER-002.
- Spec 47 `AT-SER-1` (`47-at-ser-1.md`), por las filas AT-SER-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-001, AC-UTIL-002.4.
- Spec 04 `AC-CFG-1` (`04-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-002.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-EVT-1`, `Q-T-LOG-1`, `Q-T-SER-1`, `Q-T-TK-1`, `Q-T-TK-2`, `Q-T-TK-3`, `Q-T-TK-4`, `Q-T-TMR-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra pekko-actor-testkit-typed 2.0.0: agregar la dependencia a `oracle/pom.xml` es parte de la primera tarea. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: LogCapturing (ScalaTest, JUnit4) y CapturingAppender (+5 filas)

Filas: AT-TK-006.5, AT-TK-007, AT-TK-008, AT-TK-009.1, AT-TK-009.2, AT-TK-009.3. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: SerializationTestKit (verifySerialization) (+5 filas)

Filas: AT-TK-009.4, AT-TK-010, AT-TK-011, AT-TK-012, AT-TK-013, AT-TK-014. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-TK-006.5 | LogCapturing (ScalaTest, JUnit4) y CapturingAppender | libtest oculta stdout de tests que pasan (with_test_writer) + guard RAII que vuelca el… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppe… | Q-T-LOG-1, Q-T-TK-2, Q-X-7 |
| AT-TK-007 | ManualTime (scheduler disparado explicitamente) | Reloj inyectable (clock trait) en el scheduler; ManualTime::time_passes. | actor-testkit-typed/src/test/scala/docs/org/apache/pekko/actor/testkit/typed/scaladsl/Man… | Q-T-TMR-4, Q-X-4 |
| AT-TK-008 | Probes de eventos del sistema: createUnhandledMessageProbe,… | ActorTestKit::create_unhandled_probe/dead_letter_probe/dropped_probe. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTes… | Q-T-EVT-1 |
| AT-TK-009.1 | ScalaTestWithActorTestKit, ScalaTestWithActorTestKitBase y ActorTestKitBase | fixture RAII/macro sobre #[tokio::test]: un testkit por suite, apagado en Drop, nombre… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ActorTes… | Q-T-SER-1, Q-T-TK-3 |
| AT-TK-009.2 | TestKitJunitResource (JUnit 4) | mismo fixture que AT-TK-009.1 (sin JUnit). | actor-testkit-typed/src/test/java/org/apache/pekko/actor/testkit/typed/javadsl/ActorTestK… | Q-T-SER-1, Q-T-TK-3 |
| AT-TK-009.3 | TestKitJUnit5Extension, @JUnit5TestKit y JUnit5TestKitBuilder (JUnit 5) | ActorTestKitBuilder con la misma precedencia; la extension JUnit se sustituye por el… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/JUnit5Te… | Q-T-SER-1, Q-T-TK-3 |
| AT-TK-009.4 | SerializationTestKit (verifySerialization) | verify_serialization(&obj) sobre el registro de serializadores (Q-T-SER-1); requiere… | persistence-typed-tests/src/test/scala/org/apache/pekko/persistence/typed/jackson/Replica… | Q-T-SER-1, Q-T-TK-3 |
| AT-TK-010 | TestKitSettings y pekko.actor.testkit.typed.* (timefactor, timeouts) | struct TestKitSettings con dilated(). | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestProb… | Q-T-TK-1, Q-X-1 |
| AT-TK-011 | ActorSystemStub: ActorSystem mínimo sin runtime real que usa BehaviorTestKit… | ActorSystemStub que implementa el trait de sistema tipado con las operaciones no… | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4, Q-X-7, Q-X-8 |
| AT-TK-012 | ControlledExecutor y DebugRef: ejecutor manual (cola de tareas que se corre a… | ejecutor FIFO manual + ref que registra mensajes recibidos | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/Behavior… | Q-T-TK-4, Q-X-8 |
| AT-TK-013 | LogbackUtil y LogCapturingExtension: utilidades de captura de logs sobre… | captura de eventos de log por test sobre el backend elegido en Q-X-7 | actor-testkit-typed/src/test/java/jdocs/org/apache/pekko/actor/testkit/typed/javadsl/LogC… | Q-T-LOG-1, Q-T-TK-2, Q-T-TK-3, Q-X-7 |
| AT-TK-014 | TestException (excepcion de prueba sin traza) | struct TestException{message} que implementa Error + Display sin backtrace. | actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/TestAppe… | - |

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

Spec 31 `AT-CTX-1` (`31-at-ctx-1.md`).
