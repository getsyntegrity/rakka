# Spec 24 — AC-IO (2/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 5. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 23 `AC-IO-1` (`23-ac-io-1.md`), por las filas AC-IO-008.4, AC-IO-009.2.
- Spec 08 `AC-DSP-1` (`08-ac-dsp-1.md`), por las filas AC-DSP-011.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-IO-1`, `Q-IO-3`, `Q-IO-4`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: AsyncDnsManager y adaptación del protocolo antiguo Dns.Resolve a Dns.Resolved; GetCache;… (+4 filas)

Filas: AC-IO-009.8, AC-IO-010, AC-IO-011, AC-IO-012, AC-IO-013. Contratos y aserciones: `docs/inventory/contracts/classic-io-ser.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-IO-009.8 | AsyncDnsManager y adaptación del protocolo antiguo Dns.Resolve a Dns.Resolved;… | `AsyncDnsManager` actor | actor-tests/src/test/scala/org/apache/pekko/io/dns/internal/AsyncDnsManagerSpec.scala:23-… | Q-IO-4 |
| AC-IO-010 | BufferPool / DirectByteBufferPool: pool de buffers directos con límite | `BufferPool` con `Vec<BytesMut>` | derivada del código (ver bloque de contrato) | Q-IO-3 |
| AC-IO-011 | pekko.io.pinned-dispatcher y deployments /IO-DNS/inet-address,… | `pinned-dispatcher` config + deployments | derivada del código (ver bloque de contrato) | Q-IO-1 |
| AC-IO-012 | UnsynchronizedByteArrayInputStream (java io): InputStream sobre byte[] sin… | `std::io::Cursor<&[u8]>` | actor-tests/src/test/scala/org/apache/pekko/io/UnsynchronizedByteArrayInputStreamSpec.sca… | - |
| AC-IO-013 | ByteBufferCleaner (java io): liberación explícita de ByteBuffer directo… | n/a: `Drop` | actor-tests/src/test/scala/org/apache/pekko/io/ByteBufferCleanerSpec.scala:28-33 | Q-IO-3 |

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

Spec 25 `AC-JAPI-1` (`25-ac-japi-1.md`).
