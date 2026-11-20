# Evidencia del slice P09, PR 3: AC-CFG-015 (ActorSystemSetup y BootstrapSetup), 2026-11-19

Fila: AC-CFG-015. **Resultado: `partial`, validación `oracle`** (antes `pending`/`pending`). Con este PR P09 queda completo: AC-CFG-016 (#267), AC-CFG-008 (#268) y AC-CFG-015.

## Alcance

`config::{Setup, ActorSystemSetup, BootstrapSetup}` (`config/setup.rs`) y `Settings::with_setup` / `Settings::setup` con la prioridad del proveedor del `BootstrapSetup` (`config/settings.rs`). Oráculo nuevo: `ActorSystemSetupOracle.java` (familia `actor-system-setup`, prefijo `as`, 43 ids en `cases/oracle-ids/actor-system-setup.txt`, 41 grabados y 2 pendientes (`PENDING` en `actor_system_setup_cases.rs`)).

## Contrato y citas de Pekko 2.0.0

`ActorSystemSetup.scala:30-90`, `ActorSystem.scala:48-179` (`BootstrapSetup`) y `ActorSystem.scala:118, 410-417` (`Custom` tiene identifier `"custom"` y `Settings` lee el proveedor del setup con `.map(_.identifier)`). Specs: `ActorSystemSetupSpec.scala:31-78`, `ActorSystemSetupTest.java:21-32`.

## Casos nuevos: 2628 → 2669 (41)

`as-0001..as-0041`, grabados con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, JDK 17, Q-X-16; nunca Rust). Cubren A1..A4 y A6; A5 por `Settings::setup` (PR-09). Los casos no dependen de la JDK: solo se grabaron con JDK 17. `cases/coverage.tsv` los lista con AC-CFG-015.

## ROJO / VERDE

No hay evidencia de ROJO guardada para este slice: el WIP se escribió en una sesión anterior y no conservó el comando ni la salida del fallo. No se declara un ROJO que no se observó. Lo que sí se midió en esta sesión es VERDE y la detección por mutantes (abajo).

## Replay externo completo (`uv run tools/parity/replay.py`)

**2668/2668 aplicables + 0 NA:Q-CFG-30 + 1 no aplicable, divergen 0** (total 2669 = 2628 de la base 3ced405 + 41).

## Mutantes (`tools/parity/check_divergence.py`, 10 nuevos y control)

Lanzados después del rebase, sobre el árbol `015a922` (hash de árbol `787a40bd1c30fb7e7b347a6c859e5282a37391d4`); control CLEAN **2668/2668 aplicables + 1 NA**.

| Mutante | Resultado | Pasan |
|---|---|---|
| setup-replacement-is-appended-as-a-second-entry | DETECTED | 2663/2668 |
| setup-replacement-moves-to-the-end | DETECTED | 2666/2668 |
| setup-display-sorts-the-names | DETECTED | 2666/2668 |
| setup-display-separator-has-a-space | DETECTED | 2664/2668 |
| setup-and-keeps-the-first-of-a-type | DETECTED | 2667/2668 |
| bootstrap-with-config-drops-the-provider | DETECTED | 2667/2668 |
| bootstrap-with-provider-drops-the-config | DETECTED | 2666/2668 |
| bootstrap-class-name-is-not-pekkos | DETECTED | 2667/2668 |
| settings-ignore-the-provider-of-the-setup | DETECTED | 2664/2668 |
| settings-custom-provider-of-the-setup-keeps-its-class | DETECTED | 2666/2668 |

10/10 DETECTED, 0 sobrevivientes. Después se añadió a `settings.rs` un comentario de 2 líneas (Q-CFG-34); no cambia el binario y no se repitieron los mutantes. La regresión completa la corre el Owner.

## Gates

- `cargo test --workspace`: 544/0. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` y `cargo deny check licenses`: ok.
- Cobertura (`cargo llvm-cov --workspace --summary-only`): líneas 98,29 % (217 sin cubrir), regiones 98,13 % (318 sin cubrir), funciones 98,29 %; las mismas 217 líneas y 318 regiones sin cubrir que la base, es decir, ninguna línea ni región nueva sin cubrir.
- `uv run tools/package/check.py`: 0 fallos tras `generate.py` y `unit_plan.py`; tests de las tools OK.

## Filas

| Fila | Antes | Después | Por qué |
|---|---|---|---|
| AC-CFG-015 | pending / pending | partial / oracle | Q-CFG-34, Q-CFG-32, Q-CFG-33 (propuestas) y AC-SYS-001 (`ActorSystem` aún no recibe el setup; falta que el sistema lo use y exponga `system.settings.setup`) |
| AC-CFG-008 | partial / oracle | partial / oracle | sigue por Q-CFG-31, Q-CFG-34, AC-SYS-018 y Q-CFG-9 (c); ya no depende de AC-CFG-015 |

## Decisiones propuestas

Q-CFG-32 (`BootstrapSetup` sin `classLoader` ni `defaultExecutionContext`), Q-CFG-33 (orden y nombres de `ActorSystemSetup.toString`), Q-CFG-34 (un `Custom` del setup acaba como la clase `custom`, bug de Pekko reproducido) y **Q-X-19 (fijar el oráculo en JDK 21 o 25 LTS y regrabar todo el corpus; `proposed`)**. Ninguna aceptada; `owner-decisions.tsv` sin tocar.

## Límites conocidos

Los 2 ids pendientes, BS-15 (`withClassloader`) y BS-16 (`withDefaultExecutionContext`), no tienen equivalente en Rust (Q-CFG-32); el orden de `toString` con más de cuatro setups queda en Q-CFG-33. El ROJO de este slice no se conserva (ver arriba).
