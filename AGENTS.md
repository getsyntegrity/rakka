# AGENTS.md — rakka

Port a Rust de los módulos de actores de Apache Pekko 1.7.1: `pekko-actor` (Classic) y `pekko-actor-typed` (Typed). Apache Pekko es la fuente de verdad; goPekko (`/home/pablog/workspace/getsyntegrity/gopekko`) sólo aporta aprendizajes.

## Estructura

- `upstream/pekko/` — submódulo fijado en v1.7.1 (`9a54d129…`). **Sólo lectura.** Ver `docs/upstream.md`.
- `crates/rakka-actor/`, `crates/rakka-actor-typed/` — código propio.
- `crates/*/tests/oracle/*.golden` — salidas del jar oficial de Pekko usadas por tests diferenciales.
- `tools/oracle/` — `fetch-jars.sh` (baja y verifica jars), `run.sh <script.jsh>` (corre un escenario con `jshell`).
- `docs/inventory/` — inventario de alcance (`actor-classic.tsv`, `actor-typed.tsv` + resúmenes `.md`) y `contracts/` (contrato, fuente `path:line`, aserciones y oráculo de cada fila). Es la definición de "100%". Convenciones en `docs/inventory/README.md`.
- `docs/decisions.md` (+ `docs/decisions/register.tsv`) — registro único de decisiones del Owner: autorizadas, propuestas y bloqueantes.
- `docs/implementation-order.md` — orden de ejecución; `docs/superpowers/plans/units/` — 49 specs preliminares generadas (agrupaciones de filas, máximo 5 tareas cada una; requieren revisión de atomicidad y dependencias antes de considerarse ejecutables); `docs/contract-map.tsv` — fila → spec → validación.
- `tools/package/generate.py` y `check.py` — regeneran y verifican el paquete documental desde los TSV (no editar a mano los archivos generados).
- `docs/superpowers/plans/*.md` — planes detallados previos (PATH, UTIL, detalle HOCON de CFG).
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

## Ramas y PR

- `develop` es la rama de integración. Las ramas de trabajo salen de `origin/develop` (nunca de otra rama de trabajo) y sus PR apuntan a `develop`.
- `main` recibe únicamente PR de release desde `develop`.
- No hay push directo a `main` ni a `develop`, ni force push, ni borrado de ramas. Los merges los decide el Owner.
- Antes de ramificar: verificar `git status`, preservar los cambios existentes y actualizar las referencias remotas (`git fetch --all --prune`).

## Flujo por unidad

1. Extraer el contrato desde la fuente y los specs de Pekko (`path:line`).
2. Escribir escenarios con aserciones concretas (en el plan de la unidad), con trazabilidad contrato → aserción.
3. Implementar en Rust idiomático con TDD estricto (ver "Reglas de calidad").
4. Validar: specs de Pekko portados 1:1 y, cuando aplique, golden del oráculo.
5. Actualizar `impl`/`validation` en el inventario.

Ninguna diferencia semántica con Pekko se acepta sin decisión explícita del Owner: se anota con `Q:` en la columna `notes` del inventario. Una pregunta abierta impide cerrar el comportamiento afectado: la fila queda `partial`, no `done`. Compilar, correr una demo o tener tests verdes no es paridad.

## Reglas de calidad

- **TDD estricto por comportamiento: RED → GREEN → REFACTOR.** Cada comportamiento empieza con un test que falla por la razón correcta; se guarda la evidencia real (comando y salida del fallo) antes de escribir el código de producción, después la del GREEN. No se inventa evidencia ni se declara RED sin haberlo observado.
- **Cobertura mínima del 90 %** de las líneas de producción nuevas o modificadas de cada unidad, medida con `cargo llvm-cov` y documentada en el plan o en el PR (comando, resultado y líneas no cubiertas con su motivo).
- **Nombres que expresan propósito.** Variables, parámetros, campos, funciones, tipos, módulos, archivos y tests dicen qué hacen. Se evitan nombres genéricos (`data`, `tmp`, `helper`, `util2`), abreviaturas ambiguas y los identificadores de specs (`AC-CFG-001`, `T3`) como sustituto del significado; el identificador de la fila va en comentarios o docs, no en el nombre.

## Reglas

- Trabajo local: no push, no PR, no publicar sin autorización del Owner para esa rama.
- Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de atribución de `docs/upstream.md`.
