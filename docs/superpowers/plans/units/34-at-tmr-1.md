# Spec 34 — AT-TMR (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 9. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-016, AT-BEH-020.
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-008.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-TMR-1`, `Q-T-TMR-2`, `Q-T-TMR-3`, `Q-T-TMR-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: TimerScheduler ligado al actor (withTimers) (+5 filas)

Filas: AT-TMR-001, AT-TMR-002, AT-TMR-003, AT-TMR-004, AT-TMR-005, AT-TMR-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: cancel / cancelAll / isTimerActive (+2 filas)

Filas: AT-TMR-007.1, AT-TMR-007.2, AT-TMR-008. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-TMR-001 | TimerScheduler ligado al actor (withTimers) | struct TimerScheduler<M> con HashMap<TimerKey, Timer>; clave: Box<dyn Hash+Eq> o tipo… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:232-324 | Q-T-TMR-1 |
| AT-TMR-002 | startSingleTimer (con y sin clave) | timers.start_single_timer(key, msg, delay). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:98-111 | Q-T-TMR-1 |
| AT-TMR-003 | startTimerWithFixedDelay | tokio::time::interval con MissedTickBehavior::Delay (o loop con sleep). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:113-129 | Q-T-TMR-4 |
| AT-TMR-004 | startTimerAtFixedRate | interval con MissedTickBehavior::Burst. | sin test en actor-typed-tests (el inventario viejo citaba TimerSpec.scala:113 que usa… | Q-T-TMR-4 |
| AT-TMR-005 | startPeriodicTimer | No portar (o alias deprecado). | sin test dedicado | Q-T-TMR-3 |
| AT-TMR-006 | Reemplazo por clave con garantia de no entrega del timer previo (generaciones) | TimerMsg interno {key, generation, owner} filtrado antes del Behavior. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:131-149,… | - |
| AT-TMR-007.1 | cancel / cancelAll / isTimerActive | timers.cancel(key), timers.cancel_all(), timers.is_timer_active(key). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TimerSpec.scala:151-172,… | - |
| AT-TMR-007.2 | NotInfluenceReceiveTimeout en mensajes de timer | Marca explicita al iniciar el timer (no_influence_receive_timeout) o metodo del mensaje;… | actor-tests/src/test/scala/org/apache/pekko/actor/ReceiveTimeoutSpec.scala:180-327… | Q-T-TMR-2, Q-T-TMR-4 |
| AT-TMR-008 | Timers combinados con transformMessages y anidados | Sin logica extra si el TimerMsg se rutea antes del interceptor. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TransformMessagesSpec.scala… | - |

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

Spec 35 `AT-STASH-1` (`35-at-stash-1.md`).
