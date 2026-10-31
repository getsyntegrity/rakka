# Spec 05 — AC-CFG (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 29. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 11 `AC-EVT-1` (`11-ac-evt-1.md`), por las filas AC-EVT-009, AC-EVT-011, AC-EVT-019. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 18 `AC-EXT-1` (`18-ac-ext-1.md`), por las filas AC-EXT-003. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-015, AC-SYS-018. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 22 `AC-CS-1` (`22-ac-cs-1.md`), por las filas AC-CS-006.1, AC-CS-006.2, AC-CS-007.1, AC-CS-007.2. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 06 `AC-REF-1` (`06-ac-ref-1.md`), por las filas AC-REF-020. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 16 `AC-SUP-1` (`16-ac-sup-1.md`), por las filas AC-SUP-014. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 23 `AC-SER-1` (`23-ac-ser-1.md`), por las filas AC-SER-001.1, AC-SER-001.2, AC-SER-001.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-004.2, AC-UTIL-009.
- Spec 21 `AC-RT-1` (`21-ac-rt-1.md`), por las filas AC-RT-004, AC-RT-013.1, AC-RT-013.2, AC-RT-013.3, AC-RT-014. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CFG-10`, `Q-CFG-11`, `Q-CFG-12`, `Q-CFG-13`, `Q-CFG-3`, `Q-CFG-4`, `Q-CFG-5`, `Q-CFG-6`, `Q-CFG-7`, `Q-CFG-8`, `Q-CFG-9`, `Q-UTIL-8`, `Q-X-1`, `Q-X-10`, `Q-X-2`, `Q-X-3`, `Q-X-6`, `Q-X-7`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

config-1.4.9.jar (Typesafe Config, no es código de Pekko); existe tools/oracle/config-hocon.jsh. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Carga de configuración: ConfigFactory.load + reference.conf como fallback; config… (+5 filas)

Filas: AC-CFG-001, AC-CFG-002, AC-CFG-003, AC-CFG-004, AC-CFG-005, AC-CFG-006. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: Claves JVM: daemonic, jvm-exit-on-fatal-error, jvm-shutdown-hooks, fail-mixed-versions,… (+5 filas)

Filas: AC-CFG-007, AC-CFG-008, AC-CFG-009, AC-CFG-010, AC-CFG-011, AC-CFG-012.1. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Deployer.lookup/deploy: búsqueda por path con comodines (`*`, `prefix*`, `**` terminal) (+5 filas)

Filas: AC-CFG-012.2, AC-CFG-012.3, AC-CFG-012.4, AC-CFG-013, AC-CFG-014, AC-CFG-015. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: DynamicAccess / ReflectiveDynamicAccess (instanciar clases por FQCN desde config) (+5 filas)

Filas: AC-CFG-016, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: Includes: `include "version"` inexistente se ignora; `required(...)` inexistente falla (+4 filas)

Filas: AC-CFG-017.6, AC-CFG-017.7, AC-CFG-018, AC-CFG-019, AC-CFG-020. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-CFG-001 | Carga de configuración: ConfigFactory.load + reference.conf como fallback;… | crate de HOCON (p. ej. `hocon` o parser propio) + `reference.conf` embebido con… | actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:30-205 ;… | Q-CFG-6, Q-X-1 |
| AC-CFG-002 | ActorSystem.Settings: lectura fail-fast de todas las claves pekko.* al crear… | `struct Settings` construido con `TryFrom<&Config>` | actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:33-183 | Q-CFG-3, Q-CFG-4 |
| AC-CFG-003 | Chequeo pekko.version vs versión del binario | comparar con `env!("CARGO_PKG_VERSION")` o constante "1.7.1" | actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:41-42 | Q-CFG-5 |
| AC-CFG-004 | Claves de logging: loggers, logging-filter, loggers-dispatcher,… | campos de Settings | actor-tests/src/test/scala/org/apache/pekko/event/LoggerSpec.scala:184,211,219 | Q-CFG-10, Q-X-7 |
| AC-CFG-005 | Claves de dead letters: log-dead-letters (on/off/N),… | `enum LogDeadLetters { Off, Count(u32), Always }` | actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSuspensionSpec.scala:70:46-103 | Q-CFG-4 |
| AC-CFG-006 | Claves de extensiones: library-extensions y extensions | registro de extensiones por nombre (sin reflexión) | actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:99,111,122,153,164 | Q-CFG-8, Q-X-2 |
| AC-CFG-007 | Claves JVM: daemonic, jvm-exit-on-fatal-error, jvm-shutdown-hooks,… | mapear: daemonic→hilos detachados;… | actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala:632:632-6… | Q-CFG-7 |
| AC-CFG-008 | pekko.actor.provider y ProviderSelection (local \| remote \| cluster \| FQCN) | `enum ProviderSelection { Local, Remote, Cluster, Custom(String) }`; sólo Local… | actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:38,45,53,61… | Q-CFG-9, Q-X-10, Q-X-2 |
| AC-CFG-009 | guardian-supervisor-strategy, creation-timeout, unstarted-push-timeout,… | Settings + registro de configuradores por nombre | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:336:336-357 | Q-X-2 |
| AC-CFG-010 | serialize-messages, serialize-creators,… | chequeo opcional en `tell` si hay serializador registrado para el tipo | actor-tests/src/test/scala/org/apache/pekko/actor/dungeon/DispatchSpec.scala:36 ;… | Q-CFG-11, Q-X-3 |
| AC-CFG-011 | pekko.actor.debug: receive, autoreceive, lifecycle, fsm, event-stream,… | campos de Settings + chequeos en ActorCell | actor-tests/src/test/scala/org/apache/pekko/event/LoggingReceiveSpec.scala:96,186,206,230… | Q-X-6, Q-X-7 |
| AC-CFG-012.1 | Parseo de pekko.actor.deployment: Config por entrada con fallback a `default`,… | `Deployer::from_settings(&Settings)` que construye `Deploy` por entrada | actor-tests/src/test/scala/org/apache/pekko/actor/DeployerSpec.scala:93-142 | Q-X-2 |
| AC-CFG-012.2 | Deployer.lookup/deploy: búsqueda por path con comodines (`*`, `prefix*`, `**`… | `Deployer::lookup(&[&str]) -> Option<Deploy>` sobre `WildcardIndex` | actor-tests/src/test/scala/org/apache/pekko/actor/DeployerSpec.scala:216-229 | Q-UTIL-8 |
| AC-CFG-012.3 | Creación del RouterConfig: type-mapping, FQCN, resizer legado, reintento con… | registro `name -> fn(&Config) -> Box<dyn RouterConfig>` (Q-X-2) | actor-tests/src/test/scala/org/apache/pekko/actor/DeployerSpec.scala:184-214 | Q-X-2 |
| AC-CFG-012.4 | Secciones resizer y optimal-size-exploring-resizer: claves y defaults de… | structs `DefaultResizerConfig` / `OptimalSizeExploringResizerConfig` leídos de Config | derivada del código (ver bloque de contrato) | - |
| AC-CFG-013 | Deploy, Scope, LocalScope, NoScopeGiven (valores y withFallback) | `struct Deploy` + `enum Scope { Local, NoScopeGiven, Other(Box<dyn ...>) }` | actor-tests/src/test/scala/org/apache/pekko/actor/DeployerSpec.scala:93,108:93-142 | Q-CFG-12, Q-X-2 |
| AC-CFG-014 | pekko.actor.router.type-mapping (alias de routers) | tabla estática nombre→constructor | actor-tests/src/test/scala/org/apache/pekko/actor/DeployerSpec.scala:231:231-246 | Q-X-2 |
| AC-CFG-015 | BootstrapSetup / ActorSystemSetup / Setup (config programática: classLoader,… | builder `ActorSystem::builder().config(..).setup(T)` con `TypeMap` | actor-tests/src/test/scala/org/apache/pekko/actor/setup/ActorSystemSetupSpec.scala:31,39,… | Q-CFG-13 |
| AC-CFG-016 | DynamicAccess / ReflectiveDynamicAccess (instanciar clases por FQCN desde… | registro estático `name -> fn(&Settings,&Config) -> Box<dyn T>` poblado por el usuario o… | actor-tests/src/test/scala/org/apache/pekko/actor/DynamicAccessSpec.scala:40,50,59,72… | Q-X-2 |
| AC-CFG-017.1 | Combinación y sustitución: withFallback, resolve tardío, `${?x}` opcional,… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden FB-01..08, SUB-01..14, REF-01..02 | Q-X-1 |
| AC-CFG-017.2 | Duraciones: unidades, nombres largos, sin unidad = ms, fracciones, negativos,… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden DUR-MS-*, DUR-NS-*, DUR-ERR-*, DUR-SCI-*, DUR-ROUND-*, DUR-LARGE-*, DUR-NUMBER,… | Q-X-1 |
| AC-CFG-017.3 | Tamaños en memoria: `128 KiB`, `K`, `kB`, `MB`, número sin unidad = bytes;… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden SIZE-*, SIZE-ERR-*, SIZE-MEMORY, SIZE-NUMBER | Q-X-1 |
| AC-CFG-017.4 | Booleanos `on/off/yes/no/true/false` con y sin comillas; `ON`, `1`, `maybe`… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden BOOL-*; ConfigSpec.scala:44-97 | Q-X-1 |
| AC-CFG-017.5 | Literales que Pekko interpreta por su cuenta: `infinite`, `unlimited`,… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden SPECIAL-*, REF-03 | Q-X-1 |
| AC-CFG-017.6 | Includes: `include "version"` inexistente se ignora; `required(...)`… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden INC-01..04 | Q-X-1 |
| AC-CFG-017.7 | Sintaxis del archivo: comentarios `#` y `//`, claves con guiones y puntos,… | parser HOCON + helpers `get_duration_or_infinite`, `get_size_or_unlimited` | golden REF-04..10, LIST-*, TYPE-* | Q-X-1 |
| AC-CFG-018 | Settings.amendSlf4jConfig: `pekko.use-slf4j` agrega… | función `amend_logging_backend(&Config, &FactoryRegistry) -> Config` (Q-X-7) | actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:186-204 | Q-CFG-10, Q-X-7 |
| AC-CFG-019 | Validación de la config contra reference.conf (`checkValid` con… | `Config::check_valid(&reference, &["pekko"]) -> Result<(), ConfigError>` | actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:30-205 (config de… | Q-CFG-3 |
| AC-CFG-020 | Combinación no soportada coordinated-shutdown:… | validación en `Settings::from_config` | derivada del código (ver bloque de contrato) | - |

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

Spec 06 `AC-REF-1` (`06-ac-ref-1.md`).
