# Evidencia del slice P08 (CFG-AJUSTES), 2026-11-19

Filas: AC-CFG-002, 004, 005, 006, 007, 009, 010, 011, 018 y 020 (10). **Resultado: las 10 quedan `partial`** (antes: 10 `pending`). Ninguna llega a `done`; el motivo de cada una está en su nota del inventario y en su bloque "Actualización P08" del contrato:

| Fila | Qué cubren los casos grabados | Por qué `partial` |
|---|---|---|
| AC-CFG-002 | A1, A2, A4..A8, A10 y los valores por defecto de A9 (`Settings::new` contra `new ActorSystem.Settings(..)`) | Q-CFG-15, Q-CFG-18, Q-CFG-19 (propuestas); `ActorSystem` aún no construye `Settings` (AC-SYS) |
| AC-CFG-004 | A1, A6 | A2..A5 (LoggerSpec) necesitan loggers como actores (AC-EVT-011/016); Q-CFG-15 |
| AC-CFG-005 | A1, A3 | A2 (DeadLetterSuspensionSpec) necesita AC-EVT-009; Q-CFG-15 |
| AC-CFG-006 | las listas que lee Pekko (`+=`, reemplazo) | A1..A5 necesitan el cargador de extensiones (AC-EXT-003) y el registro de fábricas; Q-CFG-15 |
| AC-CFG-007 | A1, A3 | A2 (hooks de JVM) es AC-CS-007; Q-CFG-15 |
| AC-CFG-009 | A1, A3 (⚠ `typed.timeout` no existe en 2.0.0) | A2 (guardián con StoppingSupervisorStrategy) necesita AC-SUP-014/AC-REF-020; Q-CFG-15 |
| AC-CFG-010 | las claves se leen; Q-CFG-11: `on` es error en Rust | A1..A6 necesitan AC-SER; Q-CFG-17 (propuesta); Q-CFG-15 |
| AC-CFG-011 | A1 | A2..A6 viven en los componentes que emiten los eventos; Q-CFG-15 |
| AC-CFG-018 | A1, A2, A3 (45 casos `AMEND`) | Q-CFG-15 (8 casos con `String: N`); `ActorSystem` aún no llama a la función |
| AC-CFG-020 | A1 y A2 con el texto exacto; ningún expected lleva descripción de origen | sólo la integración: A1 afirma que `ActorSystem(..)` lanza, y `ActorSystem` aún no construye `Settings` |

Decisiones aplicadas: Q-CFG-3 (a) (`checkValid` dentro de `Settings::new`), Q-CFG-4 (a), Q-CFG-7 (a) (claves de JVM aceptadas y guardadas sin efecto), Q-CFG-8 (b) (`$` de objeto Scala normalizado en `Settings::extension_names`), Q-CFG-10 + Q-X-7 (A) (`amend_slf4j_config` con el predicado de registro), Q-CFG-11 (error de configuración con `on`), Q-X-2 (nombres para el registro), Q-X-3, Q-X-6. Decisiones nuevas, `proposed` en `docs/owner-batch-02.md` (### P08 CFG-AJUSTES): Q-CFG-17 (texto del error de Q-CFG-11), Q-CFG-18 (orden de claves de `expecting: object with keys [..]` y origen de una clave ausente), Q-CFG-19 (reference contra el que valida `Settings::new`). Se añadieron las filas de P08 con expected de origen (`String: N`) a la lista de Q-CFG-15. No se aceptó ni respondió ninguna Q-*.

## Diseño

`crates/rakka-actor/src/config/settings.rs`: `Settings::new(name, Config) -> Result<Settings, SettingsError>` (`SettingsError::{Config(ConfigError), Configuration(ConfigurationException)}`) lee cada clave en el orden del constructor Scala (checkValid sin `internal-dispatcher` → `ConfigVersion` → timeouts → serialización → logging → dead letters → debug → home → scheduler → JVM → coordinated shutdown (+ combinación) → virtual nodes → versión) y devuelve el primer error; `amend_slf4j_config(&Config, is_registered)`; `Settings::extension_names`; `SuspendDuration`. `Config::default_reference_with_typed()` (oculta) da lo que `ConfigFactory.defaultReference()` con pekko-actor-typed. En `config/units.rs` se corrigió un defecto de P07 que halló la comparación: una duración mayor que los nanosegundos de un `Long` ahora satura en nanosegundos *antes* de convertir a la unidad pedida (`106752 d` en ms = 9223372036854, no un error) y un texto de sólo dígitos que no cabe en `Long` falla con `Could not parse duration number`. El test unitario de P07 `durations_millis_nanos_and_units` afirmaba lo contrario para `9223372036854775807 d` (la expectativa salía de Rust, no del oráculo): se corrigió con el valor del oráculo.

## Oráculo y casos

`ConfigSettingsOracle.java` (familia `config-settings`, prefijo de caso `cs`): construye `new ActorSystem.Settings(classLoader, config, name)` de Pekko 2.0.0 (el constructor es público y no necesita sistema en marcha) sobre la base typed-sobre-actor construida **desde el texto** de los dos `reference.conf` (así toda descripción de origen es `String: N` y ningún caso depende de la ruta del jar), e imprime sus 33 campos; y llama a `Settings.amendSlf4jConfig` con un `ReflectiveDynamicAccess` que ve o no ve las clases `org.apache.pekko.event.slf4j.*`. 216 ids en `cases/oracle-ids/config-settings.txt`; **213 grabados** `cs-0001..cs-0213` con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, nunca Rust); los 3 restantes (`SET-SER serialize-messages = on`, `serialize-creators = on`, y las dos) están en `PENDING` de `config_settings_cases.rs` con su motivo (Q-CFG-11: Pekko los acepta y Rust devuelve `ConfigurationException`; tests unitarios). `cases/coverage.tsv` lista los 213 casos con las filas que citan (familia `oracle-config-settings`).

Casos medidos y deliberadamente no grabados (dependen de la JVM, no del contrato): el orden de `expecting: object with keys [..]` (HashMap de Java) y el origen de una clave ausente (`String: 783`, `String: 3-112`, ruta del jar) → Q-CFG-18.

Verificación independiente del expected: (a) las 213 líneas grabadas son byte a byte iguales a las de una ejecución nueva de `oracle/run.sh config-settings-all`; (b) 25 casos al azar (semilla 8) vueltos a ejecutar uno a uno con `oracle/run.sh config-settings-case`, también byte a byte; (c) 12 ejecuciones de `config-settings-all` del oráculo dan el mismo hash md5 (0af3f7af4da9b945c75033558d22f519).

## ROJO

```
$ cargo test -p rakka-actor --test recorded_pekko recorded_config_settings     (antes de grabar)
no recorded config-settings-case cases
test recorded_config_settings_cases_match_pekko ... FAILED
$ diff <(oracle config-settings-all) <(classic-parity config-settings-all)      (primera comparación completa)
SET-LDSD log-dead-letters-suspend-duration = 106752 d: oracle ok 9223372036854ms, Rust ERR BadValue [.. Duration is limited to +-(2^63-1)ns ..]
SET-LDSD ... = 9223372036854775808 d: oracle ERR BadValue [Could not parse duration number '9223372036854775808'], Rust ok
(además: orden de claves de `expecting: object with keys` y origen de la clave ausente; esos casos se sacaron, Q-CFG-18)
```

## VERDE

```
$ cargo test -p rakka-actor --test recorded_pekko
test result: ok. 6 passed; 0 failed
$ python3.12 tools/parity/replay.py
2221/2221 applicable + 1 not applicable          (antes de P08: 2008/2008 + 1 not applicable; +213 casos cs-*)
```

El gemelo Rust coincide con el oráculo en las 213 líneas (la salida de `config-settings-all` de Rust es idéntica, hash md5, a la del oráculo sin las 3 líneas `PENDING`).

## Mutantes (los de P08, el de P07 cuyo patrón cambió y control)

```
| mutant | outcome | passed/total |
|---|---|---|
| hocon-duration-floors-negative (patrón actualizado) | DETECTED | 2219/2221 |
| settings-log-dead-letters-on-is-zero | DETECTED | 2217/2221 |
| settings-log-dead-letters-case-sensitive | DETECTED | 2216/2221 |
| settings-suspend-infinite-case-sensitive | DETECTED | 2219/2221 |
| settings-home-empty-is-some | DETECTED | 2201/2221 |
| settings-coordinated-shutdown-combination-accepted | DETECTED | 2217/2221 |
| settings-version-check-skipped | DETECTED | 2217/2221 |
| settings-check-valid-skipped | DETECTED | 2213/2221 |
| settings-debug-lifecycle-reads-fsm | DETECTED | 2218/2221 |
| amend-ignores-the-registry | DETECTED | 2211/2221 |
| amend-keeps-the-default-logger | DETECTED | 2212/2221 |
| amend-always-replaces-the-filter | DETECTED | 2218/2221 |
| duration-does-not-saturate-before-converting | DETECTED | 2218/2221 |
| control (unmodified) | CLEAN | 2221/2221 |
detected 13/13
```

`--selftest`: el mutante equivalente sale UNDETECTED (exit 1) con control CLEAN (2221/2221). `--check-patterns`: 102 patrones, 0 obsoletos. El patrón de `hocon-duration-floors-negative` (P07) se actualizó porque el código que mutaba cambió (saturación previa a la conversión). Los informes por mutante están en `rust/divergence-reports/`.

## Cifras

- `cargo test --workspace`: 487 passed, 0 failed (antes: 477; +9 tests unitarios de `config/settings.rs`, +1 `recorded_config_settings_cases_match_pekko`).
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `cargo deny check licenses`: limpios.
- Cobertura de líneas (`cargo llvm-cov -p rakka-actor --lib --tests --fail-under-lines 95`, Q-X-14): crate **98.16 %** (antes 98.18 %); `config/settings.rs` 98.71 %, `config/units.rs` 97.04 % (líneas sin cubrir: 144-146, el desborde de tamaños, ya existente en P07), el resto de `config/` sin cambios.
- `tools/package/check.py`: failures=0 (`rows=785 decisions=236 citation_problems=0`); `python3.12 -m unittest discover -s tools/package/tests`: OK (35 tests).
- Casos grabados: 2010 → 2223 en `cases/coverage.tsv` (+213, familia `oracle-config-settings`); 216 ids del oráculo, 213 producidos por Rust y 3 en `PENDING`.
- Filas: Classic 47 done / 93 partial / 381 pending → 47 / 103 / 371 (las 10 de P08 pasan de `pending` a `partial`; AC-CFG-003 y AC-CFG-019, de P07, siguen `partial` con nota nueva y AC-CFG-003 pasa de validación `pending` a `oracle`).

## Límites conocidos

- Sin medir ni reproducir: el orden de `expecting: object with keys [..]` y el origen de una clave ausente (Q-CFG-18); las claves ausentes se prueban con un test unitario (categoría `ValidationFailed` y clave citada).
- `Settings::new` valida contra el reference de pekko-actor, no el fusionado con typed (Q-CFG-19).
- `ProviderSelection` (`pekko.actor.provider`, AC-CFG-008) y `java-flight-recorder.enabled` (AC-UTIL-016.11) no se leen aquí.
- `pekko.actor.typed.timeout` no existe en 2.0.0 (TypedActor eliminado): no hay campo ni caso.
- `ActorSystem::create_with_settings` sigue recibiendo su `Settings` propio de dispatcher; conectar `config::Settings` con el sistema es AC-SYS (por eso AC-CFG-018 y AC-CFG-020 no llegan a `done`).
- Un entero de sólo dígitos en una duración que no cabe en `Long` ahora falla como en Typesafe Config; un número con punto o exponente que desborda satura como `(long) double`.
