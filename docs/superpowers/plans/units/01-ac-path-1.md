# Spec 01 — AC-PATH (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 5. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-PATH-001, AC-PATH-003, AC-PATH-007, AC-PATH-008, AC-PATH-009, AC-PATH-010, AC-PATH-011, AC-PATH-012, AC-PATH-013, AC-PATH-014, AC-PATH-016.

## Prerrequisitos

- Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`), por las filas AC-UTIL-002.5. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-PATH-7`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `path-address-case`) y `cases/oracle-path-address.jsonl`. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Orden de Address (addressOrdering) (+4 filas)

Filas: AC-PATH-002, AC-PATH-004, AC-PATH-005, AC-PATH-006, AC-PATH-015. Contratos y aserciones: `docs/inventory/contracts/classic-foundation.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-PATH-002 | Orden de Address (addressOrdering) | `Address::pekko_compare(&self, &Address) -> Ordering` (sin `impl Ord`) | actor-tests/src/test/scala/org/apache/pekko/actor/AddressSpec.scala:26-37 | Q-PATH-1, Q-PATH-6 |
| AC-PATH-004 | AddressFromURIString / parse (semántica de java.net.URI) | parser propio del subconjunto RFC 2396 (`uri.rs`) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorPathSpec.scala:41-49,204 ;… | Q-PATH-4, Q-PATH-6, Q-PATH-7 |
| AC-PATH-005 | RelativeActorPath (extractor de paths relativos) | `fn relative_actor_path(&str) -> Option<Vec<String>>` | actor-tests/src/test/scala/org/apache/pekko/actor/RelativeActorPathSpec.scala:27-40 | Q-PATH-6, Q-PATH-7 |
| AC-PATH-006 | ActorPathExtractor (Address + elementos desde URI absoluta) | función de parseo | actor-tests/src/test/scala/org/apache/pekko/actor/ActorPathSpec.scala:26-49 | Q-PATH-6 |
| AC-PATH-015 | Paths especiales del proveedor local: "/", "/user", "/system", "/deadLetters",… | constantes en `provider::local` | actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:248 ;… | - |

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

Spec 02 `AC-UTIL-1` (`02-ac-util-1.md`).
