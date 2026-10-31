# Spec 18 — AC-EXT (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 5. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-012.
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-006, AC-CFG-009, AC-CFG-016.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CFG-8`, `Q-EXT-1`, `Q-EXT-2`, `Q-EXT-3`, `Q-X-2`, `Q-X-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Extension / ExtensionId / AbstractExtensionId / ExtensionIdProvider: identidad por… (+4 filas)

Filas: AC-EXT-001.1, AC-EXT-001.2, AC-EXT-002, AC-EXT-003, AC-EXT-004. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-EXT-001.1 | Extension / ExtensionId / AbstractExtensionId / ExtensionIdProvider: identidad… | `trait ExtensionId { type Ext; fn create(&self, &ExtendedActorSystem) -> Self::Ext }`… | actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:28-80 ;… | Q-EXT-1 |
| AC-EXT-001.2 | ActorSystemImpl.registerExtension: una instancia por sistema, registro en… | mapa `ExtensionKey -> OnceLock<Result<Arc<dyn Any>, Arc<Error>>>` con espera acotada por… | actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:82-97 | Q-EXT-2, Q-X-4 |
| AC-EXT-002 | extension() y hasExtension() | `fn extension<E>() -> Option<Arc<E>>` | - (JavaExtension.java:83-88 y ExtensionSpec.scala:70-80 lo ejercitan) | Q-EXT-1 |
| AC-EXT-003 | Carga automática desde config (library-extensions / extensions) | registro nombre→ExtensionId | actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala:99,111,122,135,153,… | Q-CFG-8, Q-X-2 |
| AC-EXT-004 | Extensiones provistas por pekko-actor (CoordinatedShutdown,… | - | derivada del código (ver bloque de contrato) | Q-EXT-3 |

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

Spec 19 `AC-ASK-1` (`19-ac-ask-1.md`).
