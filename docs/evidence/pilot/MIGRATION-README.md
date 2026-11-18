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

## Diferencias a resolver al migrar (decisiones del Owner)
| Tema | Piloto | rakka |
|---|---|---|
| Versión de Pekko de referencia | 2.0.0 | 1.7.1 (submódulo `upstream/pekko`) |
| Nombres de crates | `pekko-actor`, `pekko-actor-typed` | `rakka-actor`, `rakka-actor-typed` |
| Oráculo | Java 17, CLI propio + `rust-anything` | Java 21, `jshell` + goldens |
| Cobertura mínima | 95 % de producción | 90 % de líneas nuevas/modificadas |
| Capas de test | tests por slice + replay grabado | política `docs/testing-policy.md` (unitario vs integración) |
| Nombres en tests | ids de escenario (`o02-restart-both`) | `AGENTS.md` pide nombres que expresen propósito |

El piloto no incluye `target/` (se reconstruye con `cargo build`; el oráculo con `mvn -f oracle/pom.xml package`).
