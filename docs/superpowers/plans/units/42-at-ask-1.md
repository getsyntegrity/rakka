# Spec 42 — AT-ASK (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 4. **Tareas:** 1 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 29 `AT-REF-1` (`29-at-ref-1.md`), por las filas AT-REF-004.
- Spec 38 `AT-EVT-1` (`38-at-evt-1.md`), por las filas AT-EVT-002.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-ASK-1`, `Q-T-ASK-2`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: AskPattern.ask / ? desde fuera de un actor (+3 filas)

Filas: AT-ASK-001, AT-ASK-002, AT-ASK-003, AT-ASK-004. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-ASK-001 | AskPattern.ask / ? desde fuera de un actor | async fn ask<Req,Res>(target:&dyn RecipientRef<Req>, make:impl… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:71-130,132-189 | Q-T-ASK-1, Q-X-9 |
| AT-ASK-002 | askWithStatus y StatusReply | `enum StatusReply<T> { Success(T), Error(StatusError) }`; `StatusError::Message(String)… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:267-272 ;… | Q-T-ASK-1 |
| AT-ASK-003 | Status.Failure clasico como respuesta falla el Future | Si se adopta Q-X-8(a): el ref temporal acepta el mensaje de fallo clasico; en (b) no… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala:191-216 | - |
| AT-ASK-004 | ask solo sobre InternalRecipientRef | trait RecipientRef<T> con ask() como metodo provisto: no hay restriccion; Q-T-ASK-2… | Sin test dedicado (todos los tests de AskSpec usan refs internos) | Q-T-ASK-2 |

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

Spec 43 `AT-RCP-1` (`43-at-rcp-1.md`).
