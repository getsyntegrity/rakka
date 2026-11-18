# Spec 28 — AT-SIG (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 8. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 36 `AT-SUP-1` (`36-at-sup-1.md`), por las filas AT-SUP-001, AT-SUP-005, AT-SUP-014, AT-SUP-015. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-005, AT-CTX-011. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-014.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-SIG-1`, `Q-X-5`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Signal (trait) y entrega confiable de senales (+5 filas)

Filas: AT-SIG-001, AT-SIG-002, AT-SIG-003, AT-SIG-004, AT-SIG-005, AT-SIG-006. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: MessageAdaptionFailure (+1 filas)

Filas: AT-SIG-007, AT-SIG-008. Contratos y aserciones: `docs/inventory/contracts/typed-core.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-SIG-001 | Signal (trait) y entrega confiable de senales | enum Signal { PreRestart, PostStop, Terminated(ActorRefAny), ChildFailed{..},… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:205-239… | Q-T-SIG-1 |
| AT-SIG-002 | PreRestart | Signal::PreRestart. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala:151-1… | - |
| AT-SIG-003 | PostStop | Signal::PostStop. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/BehaviorSpec.scala:211-217;… | - |
| AT-SIG-004 | Terminated (watch) | Signal::Terminated(ActorRef<Untyped>) con PartialEq por path+uid. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala:375-… | Q-T-SIG-1 |
| AT-SIG-005 | ChildFailed | Signal::ChildFailed{ child, cause: Arc<dyn Error + Send + Sync> }; un helper… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/TerminatedSpec.scala:25-44;… | Q-T-SIG-1 |
| AT-SIG-006 | DeathPactException por Terminated no manejado | Error tipado ActorError::DeathPact(ActorRefAny). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/WatchSpec.scala:165-204;… | - |
| AT-SIG-007 | MessageAdaptionFailure | Signal::MessageAdaptionFailure(Arc<dyn Error>); las funciones de adaptacion devuelven… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/AdaptationFailureS… | Q-X-5 |
| AT-SIG-008 | PoisonPill (senal interna) y PoisonPillInterceptor | Variante interna del enum Signal; no exportar. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/InterceptSpec.scala:306-327… | Q-T-SIG-1 |

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

Spec 29 `AT-REF-1` (`29-at-ref-1.md`).
