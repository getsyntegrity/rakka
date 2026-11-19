# Evidencia del slice P07 (CFG-HOCON), 2026-11-18

Filas: AC-UTIL-002.4, 002.5, AC-CFG-001, 003, 017.1..017.7 y 019 (12). **Reevaluación 2026-11-19 (Owner): Q-X-1 y Q-CFG-4 están aceptadas; lo abierto eran sus condiciones, cumplidas en #243. Resultado: 1 `done` (AC-UTIL-002.4) y 11 `partial`** (AC-CFG-017.5 vuelve a `partial` por Q-CFG-15: SPECIAL-08 lleva `String: 1:`) por las propuestas Q-CFG-14, 15 y 16 y por falta de caso grabado (AC-CFG-001 `Config::load`, AC-CFG-003). Cada fila explica su límite en su nota del inventario. Decisiones aplicadas: Q-X-1 (A) (parser propio, sin darla por resuelta), Q-CFG-3 (a) (`checkValid` propio), Q-CFG-4 (a) (categorías y textos de error medidos; el render sí se reproduce, no sólo se aproxima), Q-CFG-5 (a) (`pekko.version` = 2.0.0), Q-CFG-6 (a) (application.conf configurable sobre el reference embebido, sin classpath). Decisiones nuevas, `proposed` en `docs/owner-batch-02.md` (### P07): Q-CFG-14 (includes sin classpath), Q-CFG-15 (descripción de origen en errores y render), Q-CFG-16 (prefijo de variables de entorno que sobrescriben la config).

## Diseño

`crates/rakka-actor/src/config/`: `parse.rs` (lexer y parser HOCON propios), `model.rs` (árbol: cada clave guarda una *pila* de valores, el de mayor prioridad primero; fusionar con `withFallback` apila), `resolve.rs` (sustituciones tardías, `${?x}`, `+=`, autorreferencia que usa la capa inferior, concatenaciones, ciclos, variables de entorno), `units.rs` (duraciones y tamaños), `validate.rs` (`checkValid`, `renderWithRedactions`), `reference.rs` (los dos `reference.conf` de 2.0.0 embebidos, idénticos al submódulo), `error.rs` (categorías y mensajes). API: `Config::{parse_string, parse_string_with, with_fallback, resolve, default_reference, load, get_*, has_path*, check_valid, without_path, render_with_redactions, leaves}` y `check_pekko_version`. El parser no traduce código de Typesafe Config: se implementó desde el comportamiento observado con el oráculo y la especificación HOCON (Q-X-18); `tools/package/check.py` sigue sin mencionar OpenJDK/GPL.

## Oráculo y casos

`ConfigHoconOracle.java` (de #243) gana 27 casos: CV-01..21 (`checkValid`, incluido el `reference.conf` real de pekko-actor) y RW-01..06 (`renderWithRedactions`). La familia pasa de 469 a 496 ids (`cases/oracle-ids/config-hocon.txt`); el golden `crates/rakka-actor/tests/oracle/config-hocon.golden` se regeneró con el CLI (498 líneas con 2 de cabecera). Se registró la familia `config-hocon` (prefijo `ch`) en `tools/parity/make_oracle_cases.py` y se grabaron **496 casos** `ch-0001..ch-0496` con `rust-anything app record` (`cases/oracle-config-hocon.jsonl`, `rust/app-cases/ch-*`): **496 de 496 ids producidos por Rust, 0 sin producir**.

Determinismo (12 corridas de `config-hocon-all` en el oráculo y en el gemelo Rust, mismo hash md5 las 12 veces):

```
12 46773025647e447210571a8913001bca   (oráculo, 496 líneas)
12 46773025647e447210571a8913001bca   (gemelo Rust, mismo md5 que el oráculo)
```

## ROJO (arnés sin el gemelo registrado)

```
$ cargo test -p rakka-actor --test recorded_pekko recorded_config      (antes de grabar)
no recorded config-hocon-case cases
test recorded_config_hocon_cases_match_pekko ... FAILED
$ python3.12 tools/parity/replay.py        (con el arm `config-hocon-case` del CLI desactivado)
1512/2008 applicable + 1 not applicable        (496 DIVERGES: ch-0001..ch-0496)
```

## VERDE

```
$ python3.12 tools/parity/replay.py
2008/2008 applicable + 1 not applicable
$ cargo test -p rakka-actor --test recorded_pekko
test result: ok. 5 passed; 0 failed
```

El gemelo coincidió con el oráculo en las 496 líneas desde la primera comparación completa salvo una diferencia de render que el oráculo corrigió: un doble entero (`1.0`) se muestra sin fracción (`1`) en `render`.

## Mutantes (solo los de P07 + control)

```
| mutant | outcome | passed/total |
|---|---|---|
| hocon-self-reference-forgets-lower-layer | DETECTED | 2004/2008 |
| hocon-duration-floors-negative | DETECTED | 2007/2008 |
| hocon-size-accepts-uppercase-kb | DETECTED | 2007/2008 |
| hocon-boolean-on-ignores-case | DETECTED | 2007/2008 |
| hocon-check-valid-string-reference-needs-string | DETECTED | 2006/2008 |
| hocon-check-valid-list-accepts-number | DETECTED | 2007/2008 |
| hocon-render-shows-environment-values | DETECTED | 2007/2008 |
| control (unmodified) | CLEAN | 2008/2008 |
detected 7/7
```

`--selftest`: el mutante equivalente sale UNDETECTED (exit 1) con control CLEAN. `--check-patterns`: 90 patrones, 0 obsoletos.

## Cifras

- `cargo test --workspace`: 477 passed, 0 failed (25 tests unitarios nuevos en `config/tests.rs`).
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `cargo deny check licenses`: limpios.
- Cobertura de líneas (`cargo llvm-cov -p rakka-actor --lib --tests`): crate 98.18 %; `config/`: error 92.86, mod 95.70, model 95.97, parse 94.49, resolve 91.67, units 96.90, validate 95.21.
- `tools/package/check.py`: failures=0; `unittest discover -s tools/package/tests`: OK.


## Límites conocidos (por eso `partial`)

- Q-X-1 aceptada con condición cumplida (#243); P07 queda `partial` por Q-CFG-14, 15 y 16 (propuestas) y por AC-CFG-001/003 sin caso grabado completo.
- Medido sólo en el caso de dos sustituciones: el texto de un ciclo de sustituciones (`involving ${b}, ${a}, ${b}`).
- Sin medir: `required(file(..))`, `required(classpath(..))`, `url(..)` (Q-CFG-14); descripción de origen de archivos (Q-CFG-15); sobrescritura por variables con prefijo (Q-CFG-16); números enteros mayores que i128 y tamaños fraccionarios con más de 53 bits de precisión.
- `Config::check_valid` está medido contra el oráculo; su llamada desde `Settings` (con `internal-dispatcher` excluido, AC-CFG-019) llega con ActorSystem. AC-CFG-003 (`check_pekko_version`) y `Config::load` sólo tienen tests unitarios, sin caso grabado.
