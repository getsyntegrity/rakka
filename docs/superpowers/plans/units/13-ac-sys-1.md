# Spec 13 — AC-SYS (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 25. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-SYS-001.1, AC-SYS-007.

## Prerrequisitos

- Spec 03 `AC-CFG-1` (`03-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-005, AC-CFG-007, AC-CFG-008, AC-CFG-015.
- Spec 16 `AC-EXT-1` (`16-ac-ext-1.md`), por las filas AC-EXT-001.2, AC-EXT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 09 `AC-EVT-1` (`09-ac-evt-1.md`), por las filas AC-EVT-006, AC-EVT-007, AC-EVT-009, AC-EVT-010, AC-EVT-011, AC-EVT-015.
- Spec 11 `AC-CELL-1` (`11-ac-cell-1.md`), por las filas AC-CELL-004.
- Spec 14 `AC-SUP-1` (`14-ac-sup-1.md`), por las filas AC-SUP-014. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 20 `AC-CS-1` (`20-ac-cs-1.md`), por las filas AC-CS-004.1, AC-CS-004.2, AC-CS-004.3, AC-CS-006.1, AC-CS-006.2, AC-CS-009. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 07 `AC-DSP-1` (`07-ac-dsp-1.md`), por las filas AC-DSP-001, AC-DSP-006, AC-DSP-007, AC-DSP-015.
- Spec 04 `AC-REF-1` (`04-ac-ref-1.md`), por las filas AC-REF-009.1, AC-REF-009.2, AC-REF-010, AC-REF-011, AC-REF-012.
- Spec 10 `AC-TK-1` (`10-ac-tk-1.md`), por las filas AC-TK-001.1.
- Spec 15 `AC-DW-1` (`15-ac-dw-1.md`), por las filas AC-DW-012. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 08 `AC-SCH-1` (`08-ac-sch-1.md`), por las filas AC-SCH-001.
- Spec 06 `AC-MBX-1` (`06-ac-mbx-1.md`), por las filas AC-MBX-013.
- Spec 19 `AC-RT-1` (`19-ac-rt-1.md`), por las filas AC-RT-003.1. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-015.
- Spec 17 `AC-ASK-1` (`17-ac-ask-1.md`), por las filas AC-ASK-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 21 `AC-SER-1` (`21-ac-ser-1.md`), por las filas AC-SER-001.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-016.9.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CFG-32`, `Q-SYS-1`, `Q-SYS-2`, `Q-SYS-3`, `Q-SYS-4`, `Q-SYS-5`, `Q-SYS-6`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Constructores de ActorSystem: apply/create y sus sobrecargas, nombre por defecto "default" (+5 filas)

Filas: AC-SYS-001.2, AC-SYS-002, AC-SYS-003.1, AC-SYS-003.2, AC-SYS-004, AC-SYS-005. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: registerOnTermination: callbacks en orden inverso al registro (encadenados); tras… (+5 filas)

Filas: AC-SYS-006, AC-SYS-008, AC-SYS-009, AC-SYS-010, AC-SYS-011, AC-SYS-012. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: ExtendedActorSystem: provider, guardian, systemGuardian, systemActorOf, threadFactory,… (+5 filas)

Filas: AC-SYS-013, AC-SYS-014, AC-SYS-015, AC-SYS-016, AC-SYS-017, AC-SYS-018. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: terminationFuture del proveedor y 'bubble-walker' (theOneWhoWalksTheBubblesOfSpaceTime) (+5 filas)

Filas: AC-SYS-019, AC-SYS-020, AC-SYS-021, AC-SYS-022, AC-SYS-023, AC-SYS-024. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: Hijos especiales del guardián raíz: temp, deadLetters y extraNames (registerExtraNames)

Filas: AC-SYS-025. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-SYS-001.2 | Constructores de ActorSystem: apply/create y sus sobrecargas, nombre por… | `ActorSystem::builder(name).config(..).execution_context(..).provider(..).build()` y… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:253-264 ;… | Q-CFG-32, Q-SYS-6, Q-SYS-7, Q-X-1, Q-X-4, Q-X-5 |
| AC-SYS-002 | Secuencia de arranque (start): provider.init (guardián raíz, cadena de Watch… | `ActorSystemBuilder::build` ejecuta la secuencia y devuelve `Err` tras terminar el sistema | actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:100-110 ;… | Q-SYS-4, Q-X-2, Q-X-5, Q-X-7 |
| AC-SYS-003.1 | Jerarquía de guardianes "/", "/user", "/system" y su comportamiento ante… | `RootGuardian`, `UserGuardian`, `SystemGuardian` como actores internos | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:68-106 | Q-X-3 |
| AC-SYS-003.2 | Protocolo de hooks de terminación del guardián de sistema… | `SystemGuardian` con `HashSet<ActorRef>` de hooks; mensajes internos… | derivada del código (ver bloque de contrato) | Q-SYS-4 |
| AC-SYS-004 | terminate(): idempotente; corre CoordinatedShutdown si… | `async fn terminate(&self) -> Terminated` y `fn terminate_nowait(&self)` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:215-289 ;… | Q-X-4 |
| AC-SYS-005 | whenTerminated / getWhenTerminated | `system.when_terminated() -> impl Future<Output=Terminated> + Clone` (Shared) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:237-264 | Q-X-4 |
| AC-SYS-006 | registerOnTermination: callbacks en orden inverso al registro (encadenados);… | Pila de closures `Mutex<Option<Vec<Callback>>>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:215-235,266-289 | Q-X-4 |
| AC-SYS-008 | Fallo o escalamiento de /user apaga el sistema; guardian-supervisor-strategy… | `GuardianSupervisorStrategy` cargado por nombre (Q-X-2) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:331-392 | Q-X-2, Q-X-5 |
| AC-SYS-009 | Guardián de usuario custom (guardianProps; lo usa actor-typed) | parámetro interno `guardian_props: Option<Props>` del builder | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:394-411 ;… | Q-X-5, Q-X-8 |
| AC-SYS-010 | systemActorOf (actores bajo /system) | `system.system_actor_of(props, name) -> Result<ActorRef>` | actor-tests/src/test/scala/org/apache/pekko/routing/ConfiguredLocalRoutingSpec.scala:196-… | - |
| AC-SYS-011 | abort(): apagado forzado sin CoordinatedShutdown | `fn abort(&self)` | derivada del código (ver bloque de contrato) | - |
| AC-SYS-012 | Accesores: name, settings, startTime, uptime (s), eventStream, log,… | métodos del handle `ActorSystem` (Clone, Arc interno) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319 ;… | Q-EVT-1, Q-SYS-5, Q-X-4 |
| AC-SYS-013 | ExtendedActorSystem: provider, guardian, systemGuardian, systemActorOf,… | trait o métodos `pub` marcados unstable | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:319-323 ;… | Q-SYS-6 |
| AC-SYS-014 | close() (AutoCloseable): terminate + espera hasta close-actor-system-timeout | método explícito `close_blocking(&self)`; sin `Drop` que termine (Q-SYS-1) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:266-273 ;… | Q-SYS-1 |
| AC-SYS-015 | Errores fatales no capturados: jvm-exit-on-fatal-error → System.exit(-1); si… | hook de panic en hilos propios + política configurable | derivada del código (ver bloque de contrato) | Q-SYS-2, Q-X-5 |
| AC-SYS-016 | LocalActorRefProvider.actorOf: merge de Deploy, ruta router vs no-router,… | fábrica en `provider::local::spawn` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:131-197… | Q-X-2, Q-X-5 |
| AC-SYS-017 | Actores temporales: tempPath, tempContainer,… | mapa concurrente nombre→ref con contador atómico | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:248-266 ;… | - |
| AC-SYS-018 | SPI ActorRefProvider (@DoNotInherit): rootGuardian, guardian, systemGuardian,… | trait sellado `ActorRefProvider` con impl local | actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:38-43 ;… | Q-CFG-9, Q-SYS-6, Q-X-10, Q-X-2 |
| AC-SYS-019 | terminationFuture del proveedor y 'bubble-walker'… | oneshot interno + ref mínimo | derivada del código (ver bloque de contrato) | - |
| AC-SYS-020 | ManifestInfo: chequeo de versiones mezcladas de módulos Pekko | verificación en tiempo de compilación (cargo resuelve una versión por crate) + `Version`… | actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:21-33 ;… | Q-SYS-3 |
| AC-SYS-021 | Main (arranca un sistema con un actor y termina al morir) | `rakka::Main::run::<A>()` (feature `deprecated`) | derivada del código (ver bloque de contrato) | Q-SYS-3, Q-X-2 |
| AC-SYS-022 | Settings.toString con redacción de datos sensibles (renderWithRedactions) | `Settings::to_redacted_string()` y `Display` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:412-428 | Q-SYS-5, Q-X-1 |
| AC-SYS-023 | assertInitialized: guardia contra uso del sistema durante la inicialización | `system.assert_initialized() -> Result<(), InitializationError>` | derivada del código (ver bloque de contrato) | Q-X-5 |
| AC-SYS-024 | LocalActorRefProvider.resolveActorRef (String y ActorPath): paths ajenos o… | `provider.resolve_actor_ref(path) -> ActorRef` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:48-64 | - |
| AC-SYS-025 | Hijos especiales del guardián raíz: temp, deadLetters y extraNames… | `RootGuardian::child(name)` con tabla de nombres especiales | derivada del código (ver bloque de contrato) | - |

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

Spec 14 `AC-SUP-1` (`14-ac-sup-1.md`).
