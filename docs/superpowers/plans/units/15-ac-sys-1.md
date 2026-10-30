# Spec 15 — AC-SYS (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 27. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-005, AC-CFG-007, AC-CFG-008, AC-CFG-015.
- Spec 11 `AC-EVT-1` (`11-ac-evt-1.md`), por las filas AC-EVT-007, AC-EVT-009, AC-EVT-010, AC-EVT-011, AC-EVT-015.
- Spec 18 `AC-EXT-1` (`18-ac-ext-1.md`), por las filas AC-EXT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`), por las filas AC-SUP-014. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-004, AC-CELL-015.1, AC-CELL-015.2.
- Spec 17 `AC-DW-1` (`17-ac-dw-1.md`), por las filas AC-DW-001. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 22 `AC-CS-1` (`22-ac-cs-1.md`), por las filas AC-CS-004.1, AC-CS-004.2, AC-CS-004.3, AC-CS-006.1, AC-CS-006.2, AC-CS-009. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-003.1, AC-UTIL-003.2, AC-UTIL-009.
- Spec 09 `AC-DSP-1` (`09-ac-dsp-1.md`), por las filas AC-DSP-006, AC-DSP-007.
- Spec 21 `AC-RT-1` (`21-ac-rt-1.md`), por las filas AC-RT-003.1, AC-RT-003.2, AC-RT-003.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-013.
- Spec 19 `AC-ASK-1` (`19-ac-ask-1.md`), por las filas AC-ASK-002. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-007.
- Spec 06 `AC-REF-1` (`06-ac-ref-1.md`), por las filas AC-REF-010, AC-REF-012.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CFG-9`, `Q-SYS-1`, `Q-SYS-2`, `Q-SYS-3`, `Q-SYS-4`, `Q-SYS-5`, `Q-X-10`, `Q-X-2`, `Q-X-4`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Validación del nombre de ActorSystem (regex y mensaje) (+5 filas)

Filas: AC-SYS-001.1, AC-SYS-001.2, AC-SYS-002, AC-SYS-003.1, AC-SYS-003.2, AC-SYS-004. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: whenTerminated / getWhenTerminated (+5 filas)

Filas: AC-SYS-005, AC-SYS-006, AC-SYS-007, AC-SYS-008, AC-SYS-009, AC-SYS-010. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: abort(): apagado forzado sin CoordinatedShutdown (+5 filas)

Filas: AC-SYS-011, AC-SYS-012, AC-SYS-013, AC-SYS-014, AC-SYS-015, AC-SYS-016. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: Actores temporales: tempPath, tempContainer, registerTempActor/unregisterTempActor (+5 filas)

Filas: AC-SYS-017, AC-SYS-018, AC-SYS-019, AC-SYS-020, AC-SYS-021, AC-SYS-022. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: assertInitialized: guardia contra uso del sistema durante la inicialización (+2 filas)

Filas: AC-SYS-023, AC-SYS-024, AC-SYS-025. Contratos y aserciones: `docs/inventory/contracts/classic-system.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-SYS-001.1 | Validación del nombre de ActorSystem (regex y mensaje) | `ActorSystemName::parse(&str) -> Result<ActorSystemName, InvalidSystemName>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:135-162 | Q-X-5 |
| AC-SYS-001.2 | Constructores de ActorSystem: apply/create y sus sobrecargas, nombre por… | `ActorSystem::builder(name).config(..).execution_context(..).build()` y… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:250-261 ;… | Q-X-4 |
| AC-SYS-002 | Secuencia de arranque (start): provider.init (guardián raíz, cadena de Watch… | `ActorSystemBuilder::build` ejecuta la secuencia y devuelve `Err` tras terminar el sistema | actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:99,153 | Q-SYS-4 |
| AC-SYS-003.1 | Jerarquía de guardianes "/", "/user", "/system" y su comportamiento ante… | `RootGuardian`, `UserGuardian`, `SystemGuardian` como actores internos | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:69-107 | - |
| AC-SYS-003.2 | Protocolo de hooks de terminación del guardián de sistema… | `SystemGuardian` con `HashSet<ActorRef>` de hooks; mensajes internos… | derivada del código (ver bloque de contrato) | Q-SYS-4 |
| AC-SYS-004 | terminate(): idempotente; corre CoordinatedShutdown si… | `async fn terminate(&self) -> Terminated` y `fn terminate_nowait(&self)` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:212-286 | - |
| AC-SYS-005 | whenTerminated / getWhenTerminated | `system.when_terminated() -> impl Future<Output=Terminated> + Clone` (Shared) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:234-261 | Q-X-4 |
| AC-SYS-006 | registerOnTermination: callbacks en orden inverso al registro (encadenados);… | Pila de closures `Mutex<Option<Vec<Callback>>>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:212-232,263-286 | - |
| AC-SYS-007 | Creación de actores durante el apagado se rechaza | `Err(SpawnError::SystemTerminating)` desde `actor_of` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:288-326 | Q-X-5 |
| AC-SYS-008 | Fallo o escalamiento de /user apaga el sistema; guardian-supervisor-strategy… | `GuardianSupervisorStrategy` cargado por nombre (Q-X-2) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:328-375 | Q-X-2 |
| AC-SYS-009 | Guardián de usuario custom (guardianProps; lo usa actor-typed) | parámetro interno `guardian_props: Option<Props>` del builder | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:377-394 ;… | - |
| AC-SYS-010 | systemActorOf (actores bajo /system) | `system.system_actor_of(props, name) -> Result<ActorRef>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:162,245 | - |
| AC-SYS-011 | abort(): apagado forzado sin CoordinatedShutdown | `fn abort(&self)` | derivada del código (ver bloque de contrato) | - |
| AC-SYS-012 | Accesores: name, settings, startTime, uptime (s), eventStream, log,… | métodos del handle `ActorSystem` (Clone, Arc interno) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:395-411 | - |
| AC-SYS-013 | ExtendedActorSystem: provider, guardian, systemGuardian, systemActorOf,… | trait o métodos `pub` marcados unstable | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:316-320 | - |
| AC-SYS-014 | close() (AutoCloseable): terminate + espera hasta close-actor-system-timeout | método explícito `close_blocking(&self)`; sin `Drop` que termine (Q-SYS-1) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:263-270 ;… | Q-SYS-1 |
| AC-SYS-015 | Errores fatales no capturados: jvm-exit-on-fatal-error → System.exit(-1); si… | hook de panic en hilos propios + política configurable | derivada del código (ver bloque de contrato) | Q-SYS-2, Q-X-5 |
| AC-SYS-016 | LocalActorRefProvider.actorOf: merge de Deploy, ruta router vs no-router,… | fábrica en `provider::local::spawn` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:132-198… | - |
| AC-SYS-017 | Actores temporales: tempPath, tempContainer,… | mapa concurrente nombre→ref con contador atómico | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:248,268 | - |
| AC-SYS-018 | SPI ActorRefProvider (@DoNotInherit): rootGuardian, guardian, systemGuardian,… | trait sellado `ActorRefProvider` con impl local | actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:38 | Q-CFG-9, Q-X-10 |
| AC-SYS-019 | terminationFuture del proveedor y 'bubble-walker'… | oneshot interno + ref mínimo | derivada del código (ver bloque de contrato) | - |
| AC-SYS-020 | ManifestInfo: chequeo de versiones mezcladas de módulos Pekko | verificación en tiempo de compilación (cargo resuelve una versión por crate) + `Version`… | actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:19 | Q-SYS-3 |
| AC-SYS-021 | Main (arranca un sistema con un actor y termina al morir) | `rakka::Main::run::<A>()` (feature `deprecated`) | derivada del código (ver bloque de contrato) | Q-SYS-3 |
| AC-SYS-022 | Settings.toString con redacción de datos sensibles (renderWithRedactions) | `Settings::to_redacted_string()` y `Display` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:395-411 | Q-SYS-5 |
| AC-SYS-023 | assertInitialized: guardia contra uso del sistema durante la inicialización | `system.assert_initialized() -> Result<(), InitializationError>` | derivada del código (ver bloque de contrato) | - |
| AC-SYS-024 | LocalActorRefProvider.resolveActorRef (String y ActorPath): paths ajenos o… | `provider.resolve_actor_ref(path) -> ActorRef` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:49-65 | - |
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

Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`).
