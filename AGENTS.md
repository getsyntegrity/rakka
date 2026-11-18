# AGENTS.md — rakka

Port a Rust de los módulos de actores de Apache Pekko 1.7.1: `pekko-actor` (Classic) y `pekko-actor-typed` (Typed). Apache Pekko es la fuente de verdad; goPekko (`/home/pablog/workspace/getsyntegrity/gopekko`) sólo aporta aprendizajes.

## Estructura

- `migration/pekko-rust-pilot/` — piloto previo (port contra Pekko 2.0.0) importado tal cual y aislado: tiene su propio workspace y no forma parte del workspace raíz. Es fuente de código, grabaciones y evidencia para la migración; leer su `MIGRATION-README.md` (estado verificado y diferencias a resolver).
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
- `docs/testing-policy.md` — política de testing (única fuente normativa; ver "Capas de test").
- `skills/` — skills compartidas, agnósticas del agente (ver "Índice de skills" y `skills/README.md`). `.claude/skills/testing/` es sólo un adaptador mínimo para Claude Code que remite a `skills/testing/SKILL.md`.

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

- `feat/pekko-rust-pilot-migration` es la rama de integración y la rama por defecto del repositorio. Las ramas de trabajo salen de `origin/feat/pekko-rust-pilot-migration` (nunca de otra rama de trabajo) y sus PR apuntan a ella.
- `main` y `develop` están discontinuadas: no se ramifica desde ellas ni se apunta ningún PR a ellas. Quedan como referencia histórica hasta que el Owner decida archivarlas.
- No hay push directo a `feat/pekko-rust-pilot-migration` salvo pedido explícito del Owner, ni force push, ni borrado de ramas sin autorización. Los merges los decide el Owner.
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

## Índice de skills

Las skills son archivos del repositorio y se leen directamente por ruta desde la raíz; no se presupone soporte nativo de skills ni descubrimiento automático. Estructura y acceso por herramienta: [`skills/README.md`](skills/README.md).

| Acción | Lectura obligatoria |
|---|---|
| Crear, modificar, portar, clasificar o revisar pruebas | [`skills/testing/SKILL.md`](skills/testing/SKILL.md) y [`docs/testing-policy.md`](docs/testing-policy.md) |

## Capas de test

La lectura obligatoria está en el "Índice de skills". Reglas esenciales (el detalle, los criterios de elección y los ejemplos están en [`docs/testing-policy.md`](docs/testing-policy.md)):

- **Dos capas, definidas por alcance y no por carpeta.** Unitario: un comportamiento aislado y atómico de una función, componente o transición de estado; sin ActorSystem ni runtime de actores, sin red/archivos reales/entorno/servicios, sin sleeps, con tiempo y azar controlados, y sin reproducir el algoritmo de producción para calcular lo esperado. Integración: un escenario contractual completo con los componentes reales involucrados; toda interacción entre actores reales es integración aunque sea en memoria. Probar un componente real de forma aislada (mailbox, decisión de supervisión) sigue siendo unitario; la integración aparece al conectar componentes y validar el proceso. Los dobles sólo aíslan colaboradores ajenos al escenario y la frontera sustituida se declara.
- **Ubicación.** Unitarios junto al componente; integración de un crate en su `tests/`; integración transversal Classic/Typed o multi-crate en el crate interno `rakka-testint` (**previsto, aún no existe**; no publicable y distinto del testkit público). No crear ese crate ni tocar `Cargo.toml` sin tarea explícita.
- **TDD y cobertura se mantienen en ambas capas:** RED real antes del código, GREEN, refactor; cobertura mínima del 90 % según "Reglas de calidad", que puede sumar ambas capas (no hay un 90 % obligatorio por capa) pero no sustituye los unitarios necesarios, ni demuestra paridad, ni permite omitir escenarios contractuales.
- **Por unidad:** identificar aserciones unitarias y de integración; TDD unitario; agregar y ejecutar la integración en cuanto existan los componentes reales y antes de cerrar la unidad (no acumularla al final); verificar contrato → escenario → prueba → evidencia. Un doble no acredita la integración con el componente sustituido: mientras falte esa integración necesaria, la fila permanece `partial` y se identifica el componente real y la prueba pendiente.
- **Paridad.** Se conservan las aserciones observables de los specs de Pekko; un spec que arranca actores reales es integración. Los diferenciales contra el jar oficial complementan ambas capas y no reemplazan aserciones. Tests verdes y cobertura no prueban paridad.

## Reglas

- Trabajo local: no push, no PR, no publicar sin autorización del Owner para esa rama.
- Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de atribución de `docs/upstream.md`.
