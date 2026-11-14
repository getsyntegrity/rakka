# AC-PATH-016: validación del nombre en el constructor de `ChildActorPath`

Unidad: una fila, `AC-PATH-016` (P01 en `docs/unit-plan.md`). Contrato, criterios A1-A7 y evidencia: `docs/inventory/contracts/classic-foundation.md`, bloque `AC-PATH-016`. Registro de fuentes de goPekko: `docs/research/gopekko-sources.md`.

**Por qué esta unidad.** `AC-PATH-001..015` están implementados (7 `done`, 8 `partial`) y sus `partial` dependen de decisiones abiertas (`Q-PATH-1..6`) o de otras unidades (`AC-CELL`, `AC-REF`), no de código faltante. `AC-PATH-016` es la única fila de PATH sin implementar, no tiene dependencias sin resolver (`AC-PATH-010` done, `AC-PATH-011` partial sólo por `newUid`, que no interviene) ni decisión del Owner pendiente, y cabe en un PR chico.

**No es:** una unidad grande. El cambio de producción es de unas 15 líneas; el peso está en los tests y en el golden.

## Decisiones

- Bloqueantes: ninguna. `Q-PATH-2` (surrogate suelto) no aplica: un `&str` de Rust no contiene surrogates sueltos y el nombre se imprime tal cual en el mensaje.
- Rutinarias (decide el implementador): nombre de la función constructora, si `new_child` sigue privada o pasa a `pub(crate)` para el test.
- Prohibido: agregar validación de conjunto de caracteres, de nombre vacío o de `$` inicial al constructor (criterio A6; eso es `validate_path_element` y lo hace `Children`, no `ChildActorPath`).

## Prompt autocontenido para el implementador

> Implementá `AC-PATH-016` en el repo rakka, partiendo de `origin/develop` en una rama nueva. Leé primero `AGENTS.md`, `docs/testing-policy.md`, `skills/testing/SKILL.md` y el bloque `AC-PATH-016` de `docs/inventory/contracts/classic-foundation.md`.
>
> **Objetivo.** Trasladar a `ActorPath` los invariantes del constructor `ChildActorPath` de Pekko 1.7.1 (`upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:328-335`): un nombre con `/` o con `#` se rechaza con `IllegalArgumentException` y el mensaje exacto de Pekko, comprobando primero `/`.
>
> **Archivos.**
> - Producción: `crates/rakka-actor/src/actor/path.rs`. Hoy `ActorPath::child` (:206-215) divide nombre/uid, rechaza `/` y llama a `new_child` (:217-223), que no valida. Cambio mínimo: `new_child` devuelve `Result<ActorPath, IllegalArgumentError>`, contiene los dos chequeos en el orden de Pekko (`/` y después `#`), y `child` propaga su error (con `ChildError::from`, igual que hoy) en vez de duplicar el chequeo de `/`. Usá `IllegalArgumentError` y el tipo de error existentes; no crees tipos nuevos.
> - Tests unitarios: `#[cfg(test)] mod tests` de `path.rs` (junto a `child_splits_name_and_uid`, :766).
> - Oráculo: `tools/oracle/path-address.jsh` (sección CH, :262) y `crates/rakka-actor/tests/oracle/path-address.golden`; el consumidor es `crates/rakka-actor/tests/oracle_path_address.rs` (:786-790).
>
> **Cambios permitidos.** Los archivos de arriba y la fila `AC-PATH-016` en `docs/inventory/actor-classic.tsv` y en su bloque de contrato. Nada más: ni `Cargo.toml`, ni otros módulos, ni otras filas, ni decisiones.
>
> **Tareas (TDD estricto, RED real con comando y salida guardados antes de tocar producción).**
> 1. RED: tests unitarios A2, A3, A4, A5 y A7 de la tabla de criterios. A2/A3 llaman a `new_child` directamente (nombre con `#` y con `/` y `#`, uid 0); A4/A5 usan `child("a/b#5")` y `child("a#b/c")`. Mensajes, tal cual el bloque: `/ is a path separator and is not legal in ActorPath names: [<name>]` y `# is a fragment separator and is not legal in ActorPath names: [<name>]`, sin prefijo `requirement failed: `. Verificá que A2 y A3 fallen porque `new_child` acepta, y registrá qué pasa con A4/A5 (pueden pasar ya; si pasan, quedan como guardia, no como RED).
> 2. GREEN: el cambio mínimo de `new_child`/`child`. Los tests existentes (`child_splits_name_and_uid`, `descendant_skips_empty_names`, `deep_actor_path_spec`, `oracle_path_address`) pasan **sin editarse**.
> 3. Oráculo: agregar a la sección CH del `.jsh` los casos `CH-CTOR-HASH` (`new ChildActorPath(parent, "a#b", 0)`), `CH-CTOR-BOTH` (`"a/b#c"`), `CH java a/b#5` y `CH a#b/c` (`root.$div("a#b/c")`). Corré `tools/oracle/run.sh tools/oracle/path-address.jsh` con Java 21 y regenerá el golden. **Si `new ChildActorPath` no es accesible desde jshell, anotalo y limitá A2/A3 al unitario**; no inventes salida. Reflejá cada caso nuevo en `oracle_path_address.rs`.
> 4. Verificar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo llvm-cov` ≥ 90 % de las líneas nuevas o modificadas (comando, resultado y líneas sin cubrir con motivo en el PR).
> 5. Inventario: pasar `impl` a `done` y `validation` a `oracle+spec` de `AC-PATH-016` sólo si A1-A7 tienen evidencia; si A2/A3 quedaron sin oráculo, `validation=spec` o `partial` con la nota. Ejecutar `python3 -I tools/package/generate.py` y `python3 -I tools/package/check.py`.
>
> **Límites.** No modificar `upstream/`. Todo archivo que traduzca un archivo de Pekko lleva la atribución de `docs/upstream.md`. No push ni PR sin autorización del Owner. Si el oráculo contradice este bloque, gana el oráculo y se corrige el bloque. Cualquier diferencia observable nueva con Pekko se detiene y se registra con `Q:`; no se acepta sola.

## Cierre

Cada criterio A1-A7 cita un test y un comando con su salida. Las tres capas de la tabla (Pekko, goPekko, Rust) no se mezclan en el PR. `AC-PATH-001..015` no cambian de estado por este trabajo.
