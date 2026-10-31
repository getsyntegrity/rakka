# Spec 16 — AC-SUP (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 21. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-002.2, AC-CELL-007.3, AC-CELL-007.4, AC-CELL-013.1.
- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-011.
- Spec 11 `AC-EVT-1` (`11-ac-evt-1.md`), por las filas AC-EVT-012.
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-001.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-013.
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-003.1, AC-SYS-003.2.
- Spec 17 `AC-DW-1` (`17-ac-dw-1.md`), por las filas AC-DW-008. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-SUP-1`, `Q-SUP-2`, `Q-X-2`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Propagación de fallos: excepción en receive → suspender actor e hijos → Failed(child,… (+5 filas)

Filas: AC-SUP-001, AC-SUP-002, AC-SUP-003, AC-SUP-004, AC-SUP-005, AC-SUP-006.1. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: OneForOneStrategy: ventana withinTimeRange (+5 filas)

Filas: AC-SUP-006.2, AC-SUP-007, AC-SUP-008, AC-SUP-009, AC-SUP-010, AC-SUP-011. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Helpers makeDecider (Seq de excepciones → Restart si no Escalate; lista CauseDirective;… (+5 filas)

Filas: AC-SUP-012, AC-SUP-013, AC-SUP-014, AC-SUP-015, AC-SUP-016, AC-SUP-017. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: logFailure: texto y destino del log de fallos de hijos (+2 filas)

Filas: AC-SUP-018, AC-SUP-019, AC-SUP-020. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-SUP-001 | Propagación de fallos: excepción en receive → suspender actor e hijos →… | handler devuelve `Err`/panic capturado con `catch_unwind` → `SystemMessage::Failed` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:832-859… | Q-X-5 |
| AC-SUP-002 | El padre sólo actúa ante Failed de un hijo actual con uid coincidente | comparar uid en `ChildStats` | derivada del código (ver bloque de contrato) | - |
| AC-SUP-003 | Directivas Resume/Restart/Stop/Escalate y su nivel de log | `enum Directive { Resume(LogLevel), Restart(LogLevel), Stop(LogLevel), Escalate }` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:815-830 | - |
| AC-SUP-004 | defaultDecider | función `default_decider(&Failure) -> Directive` | actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala:216-245 ;… | Q-SUP-1, Q-X-5 |
| AC-SUP-005 | defaultStrategy y stoppingStrategy | constantes/constructores | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:336-357 | - |
| AC-SUP-006.1 | OneForOneStrategy: aplicación al hijo que falló y límite de reintentos… | `ChildRestartStats { retries_count, window_start }`; `OneForOneStrategy { max_retries,… | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:232-290 ;… | Q-SUP-2 |
| AC-SUP-006.2 | OneForOneStrategy: ventana withinTimeRange | `RestartWindow { start: Option<Instant>, count }` con `Instant` monótono | actor-tests/src/test/scala/org/apache/pekko/actor/RestartStrategySpec.scala:43-89 ;… | Q-SUP-2 |
| AC-SUP-007 | AllForOneStrategy | igual que OneForOne sobre el conjunto | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:318-380 ;… | - |
| AC-SUP-008 | Escalate: el padre relanza la causa sin envolver hacia su propio supervisor | propagar `Failure` original | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:767-788… | - |
| AC-SUP-009 | Resume: el hijo conserva estado y se reanudan sus hijos | - | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:815-859 | - |
| AC-SUP-010 | Suspensión recursiva de hijos durante el fallo y contadores balanceados | contador de suspensión en mailbox (entero) + reanudar al llegar a 0 | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:832-859 | - |
| AC-SUP-011 | Fallo durante la creación: Recreate/Resume se convierten en Create | - | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:861-912 | - |
| AC-SUP-012 | Helpers makeDecider (Seq de excepciones → Restart si no Escalate; lista… | closures; matching por tipo de error con `downcast_ref` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:240-290 ;… | Q-SUP-1 |
| AC-SUP-013 | Puntos de extensión de SupervisorStrategy: decider, handleChildTerminated,… | trait `SupervisorStrategy` con métodos por defecto | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorMiscSpec.scala:144-180 ;… | - |
| AC-SUP-014 | Estrategias de guardianes: raíz para ante cualquier fallo; /user configurable… | estrategias fijas + registro de configuradores | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:328-375 ;… | Q-X-2 |
| AC-SUP-015 | Reinicio de un padre reinicia a sus hijos sobrevivientes | - | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorTreeSpec.scala:27-45 | - |
| AC-SUP-016 | Parada de emergencia ante excepción en el manejo del fallo | - | derivada del código (ver bloque de contrato) | Q-X-5 |
| AC-SUP-017 | Catálogo de excepciones del ciclo de vida: ActorInitializationException… | `enum ActorError` o structs con `source()` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:277-286 ;… | Q-SUP-1 |
| AC-SUP-018 | logFailure: texto y destino del log de fallos de hijos | `fn log_failure(&self, ctx, child, cause, directive)` | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala:815-830 | - |
| AC-SUP-019 | Fallo fatal en restart: Resume con causa se convierte en Restart y no se… | bandera `failed_fatally` en la celda | derivada del código (ver bloque de contrato) | - |
| AC-SUP-020 | Fallo en handleChildTerminated de la estrategia | - | actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorMiscSpec.scala:144-163 | - |

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

Spec 17 `AC-DW-1` (`17-ac-dw-1.md`).
