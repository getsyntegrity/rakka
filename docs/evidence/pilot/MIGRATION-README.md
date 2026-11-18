# pekko-rust-pilot: material de migración hacia rakka

Piloto previo, importado **tal cual** y aislado (tiene su propio `[workspace]`; no forma parte del workspace de `rakka`
ni toca `crates/`). Sirve como fuente de código, grabaciones y evidencia para la migración, que se arma en esta rama
antes de pasar a `develop`.

## Qué contiene
- `pekko-actor/`, `pekko-actor-typed/`: port a Rust de Classic (slices 1, 2a, 2b, 3a, 3b-1 a 3b-5) y Typed (vacío).
- `oracle/`: CLI Java contra **Apache Pekko 2.0.0** (JDK 17) que genera las grabaciones (`oracle/run.sh`).
- `cases/`, `rust/app-cases/`: 566 casos grabados de Pekko y su replay (`rust-anything app replay`).
- `tools/`: generadores de escenarios, fuzzers diferenciales, mutantes (`check_divergence.py`), cobertura.
- `INVENTORY.md`, `REPORT-3b.md`, `rust/divergence-reports/`: alcance, hallazgos del oráculo y evidencia (rojo/verde, mutantes, fuzzers).

## Estado verificado (antes de importar)
Replay 566/566 · 34/34 mutantes detectados · 88 tests · Clippy `-D warnings` limpio · cobertura 96,21 % · fuzzers 0 divergencias.

## Diferencias entre el piloto y rakka: RESUELTAS (2026-11-18)
Todas se cerraron con decisiones del Owner (`docs/decisions.md`, Q-X-11 a Q-X-17) y con la consolidación de la rama `chore/b-single-workspace`.

| Tema | Piloto | rakka | Resolución |
|---|---|---|---|
| Versión de Pekko de referencia | 2.0.0 | 1.7.1 | **2.0.0** (Q-X-11); `upstream/pekko` en `v2.0.0` |
| Nombres de crates | `pekko-actor`, `pekko-actor-typed` | `rakka-actor`, `rakka-actor-typed` | **`rakka-*`** (Q-X-15); el piloto vive en `crates/rakka-actor` |
| Oráculo | Java 17, CLI propio + `rust-anything` | Java 21, `jshell` + goldens | **El del piloto** (Q-X-12, JDK 17 fijo por Q-X-16); `tools/oracle` eliminado, sus escenarios de PATH y UTIL son casos grabados (`cases/oracle-*.jsonl`) |
| Cobertura mínima | 95 % de producción | 90 % de líneas nuevas | **≥95 % de producción por crate** (Q-X-14) |
| Capas de test | tests por slice + replay grabado | política `docs/testing-policy.md` | El replay grabado es una capa más; la reclasificación de los tests va con cada slice (bloque C) |
| Nombres en tests | ids de escenario | nombres que expresen propósito | Los ids de grabación pueden quedar; los tests de Rust se renombran en el slice de su unidad |
| ActorPath / Address / uri | `identity/` (UTF-16, solo `from_string`) | `actor/path.rs`, `address.rs`, `uri.rs` (API completa, `&str`) | **Base rakka** (API completa; arreglados con TDD los 2 casos `NumberFormatException` de JDK 17). El código `identity/` del piloto se borró tras pasar replay y mutantes. Entradas con sustitutos UTF-16 sueltos: fuera de dominio (Q-X-17) |

Estructura final: ver `AGENTS.md` (sección Estructura). Los comandos de reproducción están en `tools/parity/reproduce-all.sh`.
