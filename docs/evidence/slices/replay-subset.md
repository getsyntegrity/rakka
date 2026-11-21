# Replay por subconjunto: `tools/parity/replay_subset.py` (2026-11-21)

Primera mejora de proceso derivada de evaluar rust-anything 2.0.0 para Actor Classic y Typed. No cambia ningún gate, política ni código de `crates/`: es un bucle de desarrollo, **no evidencia de gate** (los gates siguen ejecutando el replay completo con `tools/gates/replay.sh`).

## Qué problema resuelve

`rust-anything app replay` sólo sabe replayar **todo** (≈81 s aquí, 2958 casos) o **un** caso por nombre exacto (`--case`, no admite repetición ni patrones; probado). El bucle TDD de una unidad ("¿siguen coincidiendo con Pekko todos los casos que citan AC-PATH-007?") quedaba entre ambos: se corría el replay completo o se enumeraban casos a mano. `cases/coverage.tsv` ya liga cada caso a las filas del inventario que cubre; faltaba usarlo para seleccionar.

## Uso

```
cargo build --release -p rakka-actor --example classic-parity     # o --build
python3.12 tools/parity/replay_subset.py --row AC-PATH-007 [--row ...] [--family oracle-util-core] [--match 'pa-*'] [--build] [binario]
```

- Los selectores se suman (unión). `--row AC-UTIL-002` incluye `AC-UTIL-002.1` y `.2`, nunca un id más largo que sólo empiece igual.
- Un selector que no coincide con nada es un error (exit 2): una selección vacía nunca parece un pass.
- Se copia sólo el subconjunto a un árbol temporal y se ejecuta el `rust-anything app replay` real sobre él: **no toca** `rust/app-cases/replay-report.md` versionado ni los locks del replay completo.
- Clasifica igual que `replay.py` (NOT_APPLICABLE Q-X-17, NA:Q-CFG-30), restringido a los casos seleccionados. Exit 0 todo coincide, 1 divergencias, 2 uso/harness. La línea final dice `SUBSET of N cases, not gate evidence`.
- `coverage.tsv` usa el nombre completo del caso y `rust/app-cases` lo trunca a 24 caracteres y además quita un `-` final (192 casos con nombre distinto); la herramienta lo traduce. Un caso de `coverage.tsv` sin grabación (hoy sólo `name-ok-28`) hace que el selector que lo toque se rechace con exit 2 en vez de ejecutar de menos. Desfase conocido y no resuelto aquí: `rust/app-cases` tiene una grabación que `coverage.tsv` no nombra (`nfe-uri-ipv6-embedded-ip-126`).
- La ruta del binario se resuelve a absoluta, y si el replay ejecuta un número de casos distinto del seleccionado es error de harness (exit 2).
- El binario se usa tal cual (sin la copia inmutable de `replay.py`): ejecutar `cargo build` o `--build` antes; es un bucle de desarrollo.

Para la siguiente unidad: tras añadir sus filas al inventario y grabar sus casos con el oráculo (flujo de siempre), regenerar `coverage.tsv` y usar `--row <fila>` en cada ciclo RED/GREEN; el replay completo queda para el gate.

## Medición (binario `classic-parity` release, 2958 casos)

| Selección | Casos | Tiempo | Antes |
|---|---|---|---|
| `--row AC-UTIL-001` | 48 | 4 s | 81 s (replay completo) |
| `--row AC-UTIL-002` | 351 | 6 s | 81 s |
| `--row AC-PATH-007` | 295 | 11 s | 81 s |
| `--family oracle-mailbox` | 13 | 1 s | 81 s |
| `--match 'a0*'` | 9 | <1 s | 9 llamadas a `--case` a mano (≈0,7 s c/u) |

Detección verificada: con un binario deliberadamente alterado (un byte de la salida), `--row AC-UTIL-001 --row AC-PATH-006` da 160/340, `DIVERGES` en 180 casos nombrados y exit 1.

## Qué se evaluó de rust-anything 2.0.0 y qué se descartó

| Comando | Resultado |
|---|---|
| `app record` / `app replay` | Ya son el harness del proyecto. `replay` no selecciona subconjuntos: de ahí esta herramienta, que lo usa como motor. |
| `analyze` | **No analiza Scala.** Lenguajes: ts, python, c, go, ruby, java, php, csharp. Sobre `upstream/pekko/actor` ve 28 archivos Java / 2765 LOC y 0 Scala (hay 203 archivos y 53 033 líneas Scala en `src/main`); sobre `actor-typed`, 1 archivo Java / 24 LOC frente a 16 835 líneas Scala. Sus "units" y dependencias describen el 5 % Java, no el núcleo. No sirve como mapa de dependencias de Actor/Typed. |
| `map` | Mapea crates puente (napi/wasm) y sus casos; rakka no tiene ninguno (`ports: []`) y sus "units" salen del Java del oráculo. Sin utilidad hasta que exista ese tipo de crate. |
| `bridge`, `refresh`, `record`, `build`, `diff`, `bench`, `shadow`, `gate`, `profile` | Pensados para exponer una unidad Rust a JS (napi-rs/wasm-bindgen) y compararla con el original TS/Python. Rakka es una librería Rust que se valida contra el oráculo JVM con el harness CLI; no hay frontera JS que puentear. Se revisó su `--help`; no se ejecutaron porque no aplican por construcción. |
| `mcp` | Expone los mismos pasos como herramientas MCP; sin capacidad adicional. |

## Verificación

- Tests: `python3.12 -m unittest test_replay_subset` desde `tools/parity` (28 tests: selección, staging, y extremo a extremo con el `rust-anything app replay` real y un harness de pruebas). RED observado antes de crear el módulo (`ModuleNotFoundError: replay_subset`).
- Cobertura del módulo nuevo (líneas y ramas, `coverage.py`): 98 %.
- `replay_lib.py` no se modifica (disparador global de mutantes): `replay_subset.py` sólo reutiliza sus funciones puras.
