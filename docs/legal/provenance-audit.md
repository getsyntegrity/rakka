# Auditoría de procedencia

Decisión del Owner (2026-11-18, Q-X-18): todo código de rakka derivado de fuentes con licencia no compatible con Apache-2.0, o de procedencia dudosa, se reescribe en sala limpia. **El objetivo es eliminar la duda con un proceso documentado, no decidir si algo es legal**: quien audita no es abogado.

## Método (rol sucio)

- Se leyeron los encabezados y se buscó con `grep` (`OpenJDK`, `java.base`, `sun.`, `jdk.internal`, `ported`, `portado`, `translated`, `based on`, `GPL`, `Classpath`, `Copyright (c)`, `Oracle`) en `crates/`, `tools/`, `oracle/`, `cases/` y `docs/evidence/`.
- Se revisó el historial de git de los archivos marcados y los mensajes de commit.
- Clasificación: **OK** (desde cero, o derivado de Pekko Apache-2.0 con encabezado), **OK con aviso** (derivado o producido por código de terceros Apache-2.0/MIT/BSD que hay que avisar), **RUIDO** (derivado o «portado» de OpenJDK u otra fuente GPL/LGPL/sin licencia/desconocida, o con tablas o constantes copiadas, o con un comentario, commit o texto que lo diga).

## Resumen

| Clasificación | Archivos |
|---|---|
| OK (incluye 1 con observación) | 113 |
| OK con aviso | 1 |
| RUIDO | 4 |
| **Total auditado** | 118 |

Fuera de la tabla por archivo: `rust/app-cases/` (grabaciones de la salida de Pekko 2.0.0 en la JVM y de nuestro arnés: datos, OK), `rust/divergence-reports/` (informes generados por nuestras herramientas: OK), `upstream/pekko` (Pekko 2.0.0, Apache-2.0: solo lectura, no se modifica) y `target/`.

## Archivos RUIDO

### `crates/rakka-actor/src/actor/uri.rs` (815 líneas)

- **Funciones expuestas (`pub(crate)`):** `parse(&str) -> Result<Uri, UriError>`, tipo `Uri` (campos `scheme`, `host`, `port`, `raw_path`, `raw_fragment`; métodos `user_info()`, `is_absolute()`), enum `UriError { Syntax, NumberFormat(NumberFormatError) }`.
- **Usado por:** `address.rs` (`uri::parse`, `Uri`, `user_info()`), `path.rs` (`uri::parse`, `UriError`, `is_absolute()`).
- **Especificación:** `docs/legal/cleanroom/uri-spec.md`.

### `crates/rakka-actor/src/actor/jstring.rs` (109 líneas)

- **Funciones expuestas (`pub(crate)`):** `compare_utf16(&str, &str) -> Ordering`, `parse_int(&str) -> Result<i32, NumberFormatError>`.
- **Usado por:** `address.rs` (`compare_utf16`), `path.rs` (`compare_utf16`, `parse_int`).
- **Especificación:** `docs/legal/cleanroom/jstring-spec.md`.

### `docs/evidence/pilot/slice1.patch` y `docs/evidence/pilot/INVENTORY.md`

- Contienen o describen el código del piloto `jdk_uri.rs`/`jdk_int.rs`/`jdk_tables.rs`/`decode_utf8_jdk`, que ya no está en el árbol. No exponen funciones. Se borra el parche y se reescriben las dos líneas del INVENTORY.

## Estado de las reescrituras en sala limpia

| Archivo | Estado |
|---|---|
| `jstring.rs` | **Reescrito** (especificación `cleanroom/jstring-spec.md`, bitácora `jstring-log.md`, similitud `jstring-similarity.md`). El archivo anterior se borró del árbol; sigue en el historial de git. |
| `uri.rs` | **Reescrito** (especificación `cleanroom/uri-spec.md`, bitácora `uri-log.md`, similitud `uri-similarity.md`). El archivo anterior se borró del árbol; sigue en el historial de git. |

## Historial de git

- `6410d2a` (2026-10-30, «Initial import: pekko-rs scaffold, inventory, oracle and AC-PATH») y `2600f48` (renombrado a rakka): origen de `uri.rs` y `jstring.rs` con los encabezados citados.
- `60f586e` (2026-11-18, «import pekko-rust-pilot»): trajo `jdk_uri.rs`, `jdk_int.rs`, `jdk_tables.rs` y `GenTables.java` (borrados luego en `c9e91a9`).
- El historial **no se reescribe**: esos archivos siguen en commits ya publicados. Mitigación: no se publica ningún release hasta cerrar las reescrituras (Q-PATH-6; cerradas el 2026-11-18, pendiente la decisión del Owner sobre el historial) y la decisión de qué hacer con el historial queda para el Owner con asesoría legal.

## Dependencias

`deny.toml` permite solo Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0 y Zlib. El resultado de `cargo deny check licenses` está al final.

## Dudas que no son RUIDO pero se dejan escritas

- `util/timeout.rs::java_duration_to_string`: formato ISO-8601 documentado en el Javadoc de Java SE; escrito por quien conoce el JDK. Se verificará contra el Javadoc y los casos grabados.
- `crates/rakka-actor/tests/oracle/config-hocon.golden`: salida de Typesafe Config 1.4.9 (Apache-2.0).

## Tabla por archivo

| Archivo | Clasificación | Evidencia | Acción |
|---|---|---|---|
| `Cargo.toml` | OK | Documento o manifiesto propio. | Ninguna |
| `cases/coverage.tsv` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/not-applicable.tsv` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/oracle-case-ids.tsv` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/oracle-ids/path-address.txt` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/oracle-ids/util-core.txt` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/oracle-path-address.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/oracle-util-core.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/path-from-string.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-2a.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-2b.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-3a.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-3b1.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-3b2.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-3b3.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-3b4.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `cases/scenario-3b5.jsonl` | OK | Entradas de casos e ids escritos por el equipo; las salidas están en rust/app-cases. | Ninguna |
| `crates/rakka-actor-typed/Cargo.toml` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor-typed/src/lib.rs` | OK | Escrito desde cero (agrupa módulos o runtime del piloto); sin fuente externa citada. | Ninguna |
| `crates/rakka-actor/Cargo.toml` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/examples/classic-parity/json.rs` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/examples/classic-parity/main.rs` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/examples/classic-parity/path_address_cases.rs` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/examples/classic-parity/path_case.rs` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/examples/classic-parity/scenario.rs` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/examples/classic-parity/util_core_cases.rs` | OK | Escrito desde cero (arnés de paridad o manifiesto). | Ninguna |
| `crates/rakka-actor/src/actor/actor_ref.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/address.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/cell.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/children.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/errors.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/fault.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/jstring.rs` | RUIDO | El encabezado dice «Reproduces the observable behavior of OpenJDK 21 java.lang.String.compareTo, Integer.valueOf … Character.digit (GPL-2.0 WITH Classpath-exception-2.0); the BMP digit table lists the values Character.digit reports on Java 21» y deja abierta si deriva de OpenJDK (Q-PATH-6). Contiene una tabla BMP de dígitos. | Reescritura en sala limpia (PR propio) desde docs/legal/cleanroom/jstring-spec.md; la tabla se genera de la base de datos Unicode (Nd), no se copia |
| `crates/rakka-actor/src/actor/mod.rs` | OK | Escrito desde cero (agrupa módulos o runtime del piloto); sin fuente externa citada. | Ninguna |
| `crates/rakka-actor/src/actor/path.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/props.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/supervision.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/system.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/actor/uri.rs` | RUIDO | El encabezado dice «Parsing behavior ported from OpenJDK 21 java.base/java/net/URI.java (Parser, decode) and … sun/nio/cs/UTF_8.java (Decoder) (GPL-2.0 WITH Classpath-exception-2.0)» y «licensing pending Q-PATH-6». Además `parse_int_range` (añadida el 2026-11-18) se escribió conociendo la implementación de `Integer.parseInt` del JDK. | Reescritura en sala limpia (PR propio) desde docs/legal/cleanroom/uri-spec.md |
| `crates/rakka-actor/src/dispatch/dispatcher.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/dispatch/mod.rs` | OK | Escrito desde cero (agrupa módulos o runtime del piloto); sin fuente externa citada. | Ninguna |
| `crates/rakka-actor/src/dispatch/sysmsg.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/done.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/event/mod.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/lib.rs` | OK | Escrito desde cero (agrupa módulos o runtime del piloto); sin fuente externa citada. | Ninguna |
| `crates/rakka-actor/src/not_used.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/pekko_exception.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/util/helpers.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/util/mod.rs` | OK | Escrito desde cero (agrupa módulos o runtime del piloto); sin fuente externa citada. | Ninguna |
| `crates/rakka-actor/src/util/time_unit.rs` | OK | Derivado de Pekko 2.0.0 (Apache-2.0) con encabezado de atribución. | Ninguna |
| `crates/rakka-actor/src/util/timeout.rs` | OK (observación) | Derivado de Timeout.scala y scala.jdk.DurationConverters (Apache-2.0). `java_duration_to_string` imita el formato ISO-8601 documentado en el Javadoc de `java.time.Duration.toString`; lo escribió un asistente que conoce la implementación del JDK, por eso se deja constancia. | Verificar contra el Javadoc y los casos TO-NEG-04/05 en la revisión limpia; sin reescritura prevista |
| `crates/rakka-actor/tests/actor_path_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/address_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/address_test.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/classic_all_for_one.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_api.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_escalate.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_failures.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_hierarchy.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_supervision.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_suspension.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/classic_tell.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/deep_actor_path_spec.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/done_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/fixtures/pekko-2.0.0-identity-cases.txt` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/helpers_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/local_actor_ref_provider_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/oracle/config-hocon.golden` | OK con aviso | Salida grabada de com.typesafe:config 1.4.9 (Apache-2.0; dependencia de Pekko, no incluida). No se copió su código. | Ninguna; regrabado con el CLI del oráculo (Q-CFG-4, 2026-11-18); incluye los valores de `reference.conf` de 2.0.0 (Apache-2.0), no el texto del archivo |
| `crates/rakka-actor/tests/pekko_exception_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/recorded_pekko.rs` | OK | Escrito desde cero (casos propios o comparación con los casos grabados). | Ninguna |
| `crates/rakka-actor/tests/relative_actor_path_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `crates/rakka-actor/tests/timeout_spec.rs` | OK | Casos de un spec de Pekko (Apache-2.0) con encabezado. | Ninguna |
| `docs/evidence/pilot/INVENTORY.md` | RUIDO (texto) | Líneas 28 y 30: «the JDK 17.0.21 parser is ported as-is (jdk_uri.rs)» y «malformed-length rules are ported in decode_utf8_jdk». Afirmación de procedencia, no código. | Reescribir esas líneas como conducta observada, sin «portado»; el archivo aludido ya no existe |
| `docs/evidence/pilot/MIGRATION-README.md` | OK | Documento o manifiesto propio. | Ninguna |
| `docs/evidence/pilot/REPORT-3b.md` | OK | Documento o manifiesto propio. | Ninguna |
| `docs/evidence/pilot/slice1.patch` | RUIDO | Parche del piloto que contiene `jdk_uri.rs`, `jdk_int.rs` y `jdk_tables.rs`; el INVENTORY del piloto dice que el parser del JDK 17.0.21 está «ported as-is». | Borrar el archivo (la evidencia del slice 1 no depende de él) |
| `oracle/pom.xml` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `oracle/run.sh` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `oracle/src/main/java/pilot/Cli.java` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `oracle/src/main/java/pilot/Jv.java` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `oracle/src/main/java/pilot/PathAddressOracle.java` | OK | Escrito desde cero: formula preguntas a la API pública de Pekko; convertido de un script jshell propio. | Ninguna |
| `oracle/src/main/java/pilot/Scenario.java` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `oracle/src/main/java/pilot/UtilCoreOracle.java` | OK | Escrito desde cero: formula preguntas a la API pública de Pekko; convertido de un script jshell propio. | Ninguna |
| `tools/package/check.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/contract_decisions.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/cycles.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/generate.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/reanchor.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/package_sandbox.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_contract_location.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_decision_persistence.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_decision_traceability.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_deferred_contract.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_independent_decisions.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_markdown_tables.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_unit_plan.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/tests/test_unit_rows.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/package/unit_plan.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/apply_comparison.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/check_divergence.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/coverage.sh` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/fuzz_diff.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/fuzz_hier.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/fuzz_scenarios.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/fuzz_tell.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_cases.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_oracle_cases.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_2b.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_3a.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_3b1.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_3b2.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_3b3.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_3b4.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/make_scenarios_3b5.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/replay.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/replay_lib.py` | OK | Escrito desde cero (herramienta propia). | Ninguna |
| `tools/parity/reproduce-all.sh` | OK | Escrito desde cero (herramienta propia). | Ninguna |

## Resultado de `cargo deny check licenses` (cargo-deny 0.21.0)

```
licenses ok
```

(avisos «license was not encountered»: ninguna dependencia usa BSD, ISC, Unicode-3.0 ni Zlib todavía). El árbol de dependencias de `rakka` no tiene dependencias externas: solo los crates del workspace, Apache-2.0.
