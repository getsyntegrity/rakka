# AGENTS.md — pekko-rs

Port a Rust de los módulos de actores de Apache Pekko 1.7.1: `pekko-actor` (Classic) y `pekko-actor-typed` (Typed). Apache Pekko es la fuente de verdad; goPekko (`/home/pablog/workspace/getsyntegrity/gopekko`) sólo aporta aprendizajes.

## Estructura

- `upstream/pekko/` — submódulo fijado en v1.7.1 (`9a54d129…`). **Sólo lectura.** Ver `docs/upstream.md`.
- `crates/pekko-actor/`, `crates/pekko-actor-typed/` — código propio.
- `crates/*/tests/oracle/*.golden` — salidas del jar oficial de Pekko usadas por tests diferenciales.
- `tools/oracle/` — `fetch-jars.sh` (baja y verifica jars), `run.sh <script.jsh>` (corre un escenario con `jshell`).
- `docs/inventory/` — inventario de alcance (`actor-classic.tsv`, `actor-typed.tsv` + resúmenes `.md`). Es la definición de "100%".
- `docs/implementation-order.md` — orden de unidades por dependencias.
- `docs/superpowers/plans/` — un plan por unidad, máximo 5 tareas.
- `docs/research/` — evaluaciones de herramientas (REA).

## Comandos

| Qué | Comando |
|---|---|
| Build | `cargo build --workspace` |
| Tests | `cargo test --workspace` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Formato | `cargo fmt --all --check` |
| Oráculo | `tools/oracle/run.sh tools/oracle/<escenario>.jsh` (requiere Java 21 en PATH) |

No usar `-race`/sanitizers ni correr workbench salvo pedido explícito.

## Flujo por unidad

1. Extraer el contrato desde la fuente y los specs de Pekko (`path:line`).
2. Escribir escenarios con aserciones concretas (en el plan de la unidad).
3. Implementar en Rust idiomático con TDD.
4. Validar: specs de Pekko portados 1:1 y, cuando aplique, golden del oráculo.
5. Actualizar `impl`/`validation` en el inventario.

Ninguna diferencia semántica con Pekko se acepta sin decisión del Owner: se anota con `Q:` en la columna `notes` del inventario. Compilar o correr una demo no es paridad.

## Reglas

- Trabajo local: no push, no PR, no publicar sin autorización.
- Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de atribución de `docs/upstream.md`.
