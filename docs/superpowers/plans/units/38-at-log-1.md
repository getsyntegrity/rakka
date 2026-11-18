# Spec 38 — AT-LOG (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 5. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 26 `AT-BEH-1` (`26-at-beh-1.md`), por las filas AT-BEH-019.
- Spec 31 `AT-CTX-1` (`31-at-ctx-1.md`), por las filas AT-CTX-016.
- Spec 36 `AT-ADP-1` (`36-at-adp-1.md`), por las filas AT-ADP-006.1.
- Spec 39 `AT-DSP-1` (`39-at-dsp-1.md`), por las filas AT-DSP-004. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 29 `AT-TK-1` (`29-at-tk-1.md`), por las filas AT-TK-006.1, AT-TK-006.2, AT-TK-006.3, AT-TK-006.4.
- Spec 30 `AT-TK-2` (`30-at-tk-2.md`), por las filas AT-TK-006.5.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-LOG-1`, `Q-T-LOG-2`, `Q-T-LOG-3`, `Q-T-LOG-4`, `Q-T-TK-2`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: context.log (SLF4J Logger), nombre de logger detectado y setLoggerName (+4 filas)

Filas: AT-LOG-001, AT-LOG-002, AT-LOG-003, AT-LOG-004, AT-LOG-005. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-LOG-001 | context.log (SLF4J Logger), nombre de logger detectado y setLoggerName | trait ActorLog en contexto; nombre por defecto = type_name del comportamiento que loguea… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.s… | Q-T-LOG-1, Q-T-LOG-2, Q-X-7 |
| AT-LOG-002 | MDC automatico: pekkoSource, pekkoAddress, sourceActorSystem, pekkoTags | campos estructurados del span `tracing` abierto por mensaje: `pekko_source`,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.s… | Q-T-LOG-1, Q-T-TK-2 |
| AT-LOG-003 | Configuracion SLF4J: pekko.use-slf4j=on y Slf4jLogger agregado automaticamente | Config key aceptada; el backend por defecto se decide en Q-T-LOG-1/Q-X-7. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.s… | Q-T-LOG-3, Q-X-7 |
| AT-LOG-004 | LoggerOps (trace2/traceN... para Scala 2) y Markers | **subsumido por la macros de `tracing`** (`info!("template {} {}", a, b)` o campos… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/ActorLoggingSpec.s… | Q-T-LOG-1, Q-T-LOG-4 |
| AT-LOG-005 | Mensajes de log del LocalActorRefProvider en logger "serialization" | Log debug con target fijo al resolver refs por path. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/LocalActorRefProviderLogMes… | Q-T-LOG-1 |

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

Spec 39 `AT-DSP-1` (`39-at-dsp-1.md`).
