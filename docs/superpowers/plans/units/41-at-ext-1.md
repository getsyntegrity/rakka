# Spec 41 — AT-EXT (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 7. **Tareas:** 2 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 37 `AT-ADP-1` (`37-at-adp-1.md`), por las filas AT-ADP-008.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-EXT-1`, `Q-T-EXT-2`, `Q-T-EXT-3`, `Q-X-2`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Extension / ExtensionId: identidad, una instancia por sistema y concurrencia (+5 filas)

Filas: AT-EXT-001.1, AT-EXT-001.2, AT-EXT-001.3, AT-EXT-002.1, AT-EXT-002.2, AT-EXT-002.3. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: ExtensionSetup / AbstractExtensionSetup (override en tests)

Filas: AT-EXT-003. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-EXT-001.1 | Extension / ExtensionId: identidad, una instancia por sistema y concurrencia | ExtensionId<T> como valor estatico/Arc con identidad por puntero; registry… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:89-120… | Q-T-EXT-2 |
| AT-EXT-001.2 | Extension: fallo de creacion cacheado | Registro con Result<Arc<T>,Arc<Error>> cacheado y clonado en cada acceso. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:136-153 | Q-T-EXT-3 |
| AT-EXT-001.3 | Extension: extension() sin registro y hasExtension | Result<Arc<T>,ExtensionError::NotRegistered> y has_extension(). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:121-13… | Q-T-EXT-2 |
| AT-EXT-002.1 | Carga desde configuracion: pekko.actor.typed.library-extensions | Lista de nombres sustituida por registro explicito (Q-X-2):… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:165-193 | Q-X-2 |
| AT-EXT-002.2 | Carga desde configuracion: pekko.actor.typed.extensions | Registro explicito con politica "skip" solo para errores de resolucion de nombre. | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:121-15… | Q-T-EXT-1 |
| AT-EXT-002.3 | Carga de extensiones tipadas desde el sistema clasico (LoadTypedExtensions) | n/a si no hay sistema clasico (Q-X-8 b). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:204-234 | Q-X-8 |
| AT-EXT-003 | ExtensionSetup / AbstractExtensionSetup (override en tests) | Builder del sistema acepta ExtensionOverride(id, factory). | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ExtensionsSpec.scala:37-40… | - |

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

Spec 42 `AT-ASK-1` (`42-at-ask-1.md`).
