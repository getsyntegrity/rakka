# Evidencia del slice P09, PR 1: AC-CFG-016 (FactoryRegistry), 2026-11-19

Fila: AC-CFG-016 (DynamicAccess / ReflectiveDynamicAccess). **Resultado: `partial`, validación `oracle`** (antes `pending`/`pending`). AC-CFG-008 sigue `pending` (PR 2) con una nota de Q-CFG-31; AC-CFG-015 es el PR 3. Decisión nueva: **Q-CFG-31 (`proposed`)**, que corrige la redacción de Q-X-2 y de Q-CFG-9 (a); no se tocó ninguna decisión aceptada ni `owner-decisions.tsv`.

## Alcance

`crates/rakka-actor/src/config/dynamic_access.rs`: `FactoryRegistry<F>` (`register`, `contains`, `lookup`, `create`, `names`), `ClassNotFoundError` (`java_class()` = `java.lang.ClassNotFoundException`, `message()` = el nombre pedido) y `CreateError::{ClassNotFound, Construction}`. Oráculo `DynamicAccessOracle.java` (familia `dynamic-access`, prefijo `da`, 20 casos `da-0001..da-0020`); arnés `examples/classic-parity/dynamic_access_cases.rs`; prueba `recorded_dynamic_access_cases_match_pekko`. Las ramas de consumidores (proveedor, Dispatchers, Mailboxes) no se implementan aquí: quedan anotadas en AC-CFG-008, AC-MBX-004, AC-DSP-004 y AC-DSP-010.

## Contrato y citas de Pekko 2.0.0

- `actor/src/main/scala/org/apache/pekko/actor/DynamicAccess.scala:32-71`: `createInstanceFor`, `getClassFor`, `classIsOnClasspath`, `getObjectFor`, `classLoader`.
- `actor/src/main/scala/org/apache/pekko/actor/ReflectiveDynamicAccess.scala:44`: `Class.forName(fqcn, false, classLoader)` (coincidencia exacta, sin plegar mayúsculas ni recortar); `:54` el error del constructor se relanza sin `InvocationTargetException`; `:69-72` `createInstanceFor(fqcn, ..)` = `getClassFor(fqcn).flatMap(..)`; `:74-79` `classIsOnClasspath`.
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:996-1011`: el proveedor se crea con `dynamicAccess.createInstanceFor[ActorRefProvider](ProviderClass, arguments).get`; el `catch` solo detiene el scheduler y relanza la misma excepción.
- Specs portados: `actor-tests/src/test/scala/org/apache/pekko/actor/DynamicAccessSpec.scala:40-48` (A1), `:50-57` y `:77-86` (A2), `:59-70` (argumentos), `:72-75` (A4), `:102-109` (error del constructor sin envolver).
- Sin equivalente en Rust (reflexión JVM; un registro, una firma, Q-X-2): A3 (constructores alternativos por firma), `:90-95` (*handles* de constructor), `:111-118` (campo `MODULE$`) y A5 (`getObjectFor`).

## Premisa verificada con una segunda fuente (Pekko 2.0.0 en la JVM)

Q-X-2 y Q-CFG-9 (a) decían `ConfigurationException`. La lectura de `ActorSystem.scala:996-1011` y la ejecución de un `ActorSystem` real con cada proveedor (JDK 17, classpath del oráculo) dan otra cosa:

```
foo.NonExisting -> java.lang.ClassNotFoundException: foo.NonExisting
remote -> java.lang.ClassNotFoundException: org.apache.pekko.remote.RemoteActorRefProvider
cluster -> java.lang.ClassNotFoundException: org.apache.pekko.cluster.ClusterActorRefProvider
local -> created ActorSystemImpl
```

Correspondencia con los casos grabados (se grabaron con el FQCN completo; los nombres cortos `remote` y `cluster` se graban en el PR 2, cuando exista `ProviderSelection`):

| Ejecución JVM | Caso grabado | Línea del oráculo |
|---|---|---|
| `foo.NonExisting` | `da-0014` (PRV-01) | `ERR ClassNotFoundException [foo.NonExisting]` |
| `remote` (`org.apache.pekko.remote.RemoteActorRefProvider`) | `da-0015` (PRV-02) | `ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]` |
| `cluster` (`org.apache.pekko.cluster.ClusterActorRefProvider`) | `da-0016` (PRV-03) | `ERR ClassNotFoundException [org.apache.pekko.cluster.ClusterActorRefProvider]` |
| `local` (`org.apache.pekko.actor.LocalActorRefProvider`) | `da-0017` (PRV-04) | `ok created` |

Los envoltorios de los consumidores: `Dispatchers.scala:274-281` usa `ConfigurationException` (MessageDispatcherConfigurator por FQCN); `Mailboxes.scala:86-96` usa `ConfigurationException` al cargar un tipo de requirement y `Mailboxes.scala:235-243` usa `IllegalArgumentException` al crear el `MailboxType`.

## Oráculo y casos

`oracle/run.sh dynamic-access-all` (JDK 17, openjdk@17 17.0.21; `oracle/run.sh` no cambia) imprime 20 líneas deterministas (dos ejecuciones idénticas, `cmp`); las 20 las produce Rust (`cases/oracle-ids/dynamic-access.txt`: 20 ids, 0 sin producir). Se grabaron con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, nunca Rust): `da-0001..da-0017` primero y `da-0018..da-0020` después (CNF-07, ON-07 y ON-08, añadidos para que mutaciones de mayúsculas y espacios tengan un caso que las mate). `cases/coverage.tsv` lista los 20 con sus filas. Los casos nuevos no dependen de la JDK: se grabaron solo con JDK 17 (Q-X-16); no se instaló JDK 21 (la excepción de #266 es solo para NUM-RENDER y NUM-SHORT).

```
CNF-01 createInstanceFor foo.NonExistingClass -> ERR ClassNotFoundException [foo.NonExistingClass]
CNF-02 createInstanceFor org.apache.pekko.actor.foo.NonExistingClass -> ERR ClassNotFoundException [org.apache.pekko.actor.foo.NonExistingClass]
CNF-03 createInstanceFor empty name -> ERR ClassNotFoundException []
CNF-04 createInstanceFor name differing only in case -> ERR ClassNotFoundException [org.apache.pekko.dispatch.unboundedmailbox]
CNF-05 createInstanceFor the provider org.apache.pekko.remote.RemoteActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]
CNF-06 createInstanceFor the provider org.apache.pekko.cluster.ClusterActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.cluster.ClusterActorRefProvider]
ON-01 classIsOnClasspath i.just.made.it.up.to.hurt.Myself -> ok false
ON-02 classIsOnClasspath org.apache.pekko.actor.Actor -> ok true
ON-03 classIsOnClasspath org.apache.pekko.remote.RemoteActorRefProvider -> ok false
ON-04 classIsOnClasspath org.apache.pekko.cluster.ClusterActorRefProvider -> ok false
ON-05 classIsOnClasspath org.apache.pekko.actor.LocalActorRefProvider -> ok true
ON-06 classIsOnClasspath empty name -> ok false
INST-01 createInstanceFor org.apache.pekko.dispatch.UnboundedMailbox -> ok created org.apache.pekko.dispatch.UnboundedMailbox
PRV-01 provider foo.NonExisting -> ERR ClassNotFoundException [foo.NonExisting]
PRV-02 provider org.apache.pekko.remote.RemoteActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]
PRV-03 provider org.apache.pekko.cluster.ClusterActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.cluster.ClusterActorRefProvider]
PRV-04 provider org.apache.pekko.actor.LocalActorRefProvider -> ok created
CNF-07 createInstanceFor name with surrounding spaces -> ERR ClassNotFoundException [ org.apache.pekko.dispatch.UnboundedMailbox ]
ON-07 classIsOnClasspath name differing only in case -> ok false
ON-08 classIsOnClasspath name with a trailing space -> ok false
```

Verificación independiente: `diff` entre `oracle/run.sh dynamic-access-all` y `classic-parity dynamic-access-all` vacío (20/20) antes de grabar.

## ROJO

Tests unitarios, con un stub de cuerpos incorrectos (`cargo test -p rakka-actor --lib dynamic_access`):

```
test config::dynamic_access::tests::the_empty_name_is_not_registered ... ok
test config::dynamic_access::tests::contains_tells_whether_a_name_is_registered ... FAILED
test config::dynamic_access::tests::a_failing_factory_error_is_returned_unwrapped ... FAILED
test config::dynamic_access::tests::passes_the_arguments_to_the_factory ... FAILED
test config::dynamic_access::tests::registering_a_name_again_replaces_the_factory_and_returns_the_previous_one ... FAILED
test config::dynamic_access::tests::names_match_exactly ... FAILED
test config::dynamic_access::tests::an_unregistered_name_is_a_class_not_found_error_carrying_the_name ... FAILED
test config::dynamic_access::tests::an_error_chain_reaches_the_class_not_found_error ... FAILED
test config::dynamic_access::tests::creates_an_instance_from_a_registered_name ... FAILED
test config::dynamic_access::tests::lookup_of_an_unregistered_name_is_the_same_error ... FAILED
test config::dynamic_access::tests::names_are_listed_in_ascending_order ... FAILED
test result: FAILED. 1 passed; 10 failed; 0 ignored; 0 measured; 348 filtered out; finished in 0.00s
```

10 de 11 fallaban. El que pasaba con el stub, `the_empty_name_is_not_registered`, no ejercitaba la implementación (el stub devolvía siempre un nombre vacío). Se reforzó como `the_empty_name_is_an_ordinary_name` y con el stub falla:

```
test config::dynamic_access::tests::the_empty_name_is_an_ordinary_name ... FAILED
assertion failed: registry.contains("")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 358 filtered out; finished in 0.00s
```

Casos grabados, antes de grabar (`cargo test -p rakka-actor --test recorded_pekko recorded_dynamic_access`):

```
test recorded_dynamic_access_cases_match_pekko ... FAILED
no recorded dynamic-access-case cases
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.39s
```

Tests añadidos para tres líneas de producción que la primera medición de cobertura dejó sin cubrir (`Display` de `ClassNotFound`, `source()` de `Construction`, `Default`); con esas tres líneas rotas a propósito fallan:

```
test config::dynamic_access::tests::the_default_registry_is_empty ... FAILED
test config::dynamic_access::tests::a_class_not_found_error_displays_as_its_name_through_create_error ... FAILED
test config::dynamic_access::tests::a_construction_error_is_the_source_of_the_create_error ... FAILED
test result: FAILED. 11 passed; 3 failed; 0 ignored; 0 measured; 348 filtered out; finished in 0.00s
```

## VERDE

```
test config::dynamic_access::tests::contains_tells_whether_a_name_is_registered ... ok
test config::dynamic_access::tests::names_match_exactly ... ok
test config::dynamic_access::tests::lookup_of_an_unregistered_name_is_the_same_error ... ok
test config::dynamic_access::tests::a_failing_factory_error_is_returned_unwrapped ... ok
test config::dynamic_access::tests::an_unregistered_name_is_a_class_not_found_error_carrying_the_name ... ok
test config::dynamic_access::tests::an_error_chain_reaches_the_class_not_found_error ... ok
test config::dynamic_access::tests::creates_an_instance_from_a_registered_name ... ok
test config::dynamic_access::tests::a_class_not_found_error_displays_as_its_name_through_create_error ... ok
test config::dynamic_access::tests::passes_the_arguments_to_the_factory ... ok
test config::dynamic_access::tests::a_construction_error_is_the_source_of_the_create_error ... ok
test config::dynamic_access::tests::the_default_registry_is_empty ... ok
test config::dynamic_access::tests::the_empty_name_is_an_ordinary_name ... ok
test config::dynamic_access::tests::registering_a_name_again_replaces_the_factory_and_returns_the_previous_one ... ok
test config::dynamic_access::tests::names_are_listed_in_ascending_order ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 348 filtered out; finished in 0.00s
```

```
test only_a_jdk17_double_text_difference_is_tolerated ... ok
test recorded_dynamic_access_cases_match_pekko ... ok
test recorded_path_from_string_cases_match_pekko ... ok
test recorded_config_hocon_cases_match_pekko ... ok
test recorded_util_core_cases_match_pekko ... ok
test recorded_path_address_cases_match_pekko ... ok
test recorded_config_settings_cases_match_pekko ... ok
test recorded_scenario_cases_match_pekko ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.24s
```

## Replay externo completo (`uv run tools/parity/replay.py`)

Referencia de la base (1584d4f): 2576 = 2575 pasan + 1 otro NA, 0 divergen. Ahora: `2595/2595 applicable + 0 NA:Q-CFG-30 + 1 not applicable (diverge 0)`; `breakdown: pass 2595 / NA:Q-CFG-30 0 / other NA 1 / diverge 0`. Total 2596 = 2576 + 20 casos `da-*`.

## Mutantes (`tools/parity/check_divergence.py`, 6 nuevos y control)

| mutante | resultado | pasan/total | casos que fallan |
|---|---|---|---|
| registry-lookup-ignores-case | DETECTED | 2594/2595 | da-0004 |
| registry-lookup-trims-the-name | DETECTED | 2594/2595 | da-0018 |
| registry-contains-ignores-case | DETECTED | 2594/2595 | da-0019 |
| registry-contains-trims-the-name | DETECTED | 2594/2595 | da-0020 |
| class-not-found-drops-the-name | DETECTED | 2586/2595 | da-0001, 0002, 0004, 0005, 0006, 0014, 0015, 0016, 0018 |
| class-not-found-is-another-java-class | DETECTED | 2585/2595 | da-0001..0006, 0014..0016, 0018 |
| control (unmodified) | CLEAN | 2595/2595 | - |

`detected 6/6`, UNDETECTED 0, PATCH_ERROR 0, BUILD_ERROR 0, TIMEOUT 0. `--check-patterns`: 145 patrones, 0 obsoletos. La regresión completa de mutantes no se corrió (la corre el Owner).

## Gates

- `cargo fmt --all --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok (0 avisos); `cargo deny check licenses` ok.
- `cargo test --workspace`: 520 passed, 0 failed (26 líneas `test result`).
- Cobertura (`cargo llvm-cov -p rakka-actor --lib --tests --fail-under-lines 95`, sin exclusiones): rakka-actor **98.26 % de líneas (12107/12321), 98.11 % de regiones (16290/16604), 98.29 % de funciones (1781/1812)**; rakka-actor-typed no tiene nada instrumentado. `config/dynamic_access.rs`: **99.49 % de líneas (196/197), 99.56 % de regiones (227/228)**. Sin cubrir: la línea 202, `panic!("expected ClassNotFound, got {error:?}");`, la rama de fallo de un `let ... else` dentro del módulo de tests (`#[cfg(test)]` en la línea 126): solo de test. Las demás regiones con conteo 0 de ese archivo son instanciaciones monomórficas duplicadas de funciones genéricas, no líneas sin cubrir.
- `uv run tools/package/check.py`: failures=0 (`rows=785 decisions=239 citation_problems=0`); `python3.12 -m unittest discover -s tools/package/tests`: 35 tests, OK.

## Filas y decisiones

| Fila | Antes | Ahora | Por qué |
|---|---|---|---|
| AC-CFG-016 | pending / pending | partial / oracle | Q-CFG-31 (propuesta) afecta al tipo del error; A3 y A5 sin equivalente; los FQCN de `reference.conf` se preinstalan cuando exista cada consumidor |
| AC-CFG-008 | pending / pending | pending / pending (nota Q-CFG-31) | `ProviderSelection` es el PR 2 |

Q-CFG-31 (`proposed`, `docs/owner-batch-02.md`, `classic-foundation.decisions.tsv`): (a) dos capas: el registro devuelve `ClassNotFound` con el nombre y cada consumidor lo transforma como Pekko; (b) mantener el texto literal de Q-X-2 (`ConfigurationException`). Recomendada (a). Se mantienen Q-X-2 (A) y Q-CFG-9 (c).

## Límites conocidos

- Un solo `FactoryRegistry` por tipo de factoría: la firma la elige el consumidor; no hay constructores alternativos por firma ni `getObjectFor`.
- Los nombres cortos `remote`/`cluster` y la prioridad de `BootstrapSetup.actorRefProvider` son AC-CFG-008/015.
- `cargo llvm-cov report --lcov` salió vacío tras `--json`; la línea sin cubrir sale del JSON.
