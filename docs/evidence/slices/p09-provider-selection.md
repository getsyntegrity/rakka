# Evidencia del slice P09, PR 2: AC-CFG-008 (ProviderSelection), 2026-11-19

Fila: AC-CFG-008 (`pekko.actor.provider` y `ProviderSelection`). **Resultado: `partial`, validación `oracle`** (antes `pending`/`pending`). AC-CFG-016 sigue `partial` (PR 1, #267); AC-CFG-015 es el PR 3. No se crea ninguna decisión nueva: se mantiene Q-CFG-31 (`proposed`, PR 1) y no se tocó ninguna decisión aceptada ni `owner-decisions.tsv`.

## Alcance

- `crates/rakka-actor/src/config/provider_selection.rs`: `ProviderSelection` (`Local`, `Remote`, `Cluster`, `Custom(String)`; `from_identifier`, `identifier`, `fqcn`, `has_cluster`) y las constantes `LOCAL_/REMOTE_/CLUSTER_ACTOR_REF_PROVIDER`.
- `config/settings.rs`: `Settings::new` lee `pekko.actor.provider` justo después de `pekko.version` (el orden del constructor Scala) y expone `provider_selection`, `provider_class()` y `has_cluster()`. La prioridad de `BootstrapSetup.actorRefProvider` es AC-CFG-015 (PR 3).
- Oráculo nuevo `ProviderSelectionOracle.java` (familia `provider-selection`, prefijo `ps`, 25 casos) y 6 casos nuevos `PRV-05..10` al final de `DynamicAccessOracle.java` (`da-0021..da-0026`). No se tocó ningún consumidor.

## Contrato y citas de Pekko 2.0.0

- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:102-144` (`ProviderSelection`: `Local`, `Remote`, `Cluster`, `Custom`; `apply` en `:137-144`: solo `local`, `remote`/clase de remote y `cluster`/clase de cluster; cualquier otra cadena es `Custom`).
- `ActorSystem.scala:411-421` (`Settings.ProviderSelectionType`, `ProviderClass`, `HasCluster`); `:410-414` la prioridad del `BootstrapSetup` (PR 3).
- `ActorSystem.scala:996-1011`: el proveedor se crea con `dynamicAccess.createInstanceFor[ActorRefProvider](ProviderClass, arguments).get` y el `catch` solo detiene el scheduler y relanza la excepción (casos PRV-05..10).
- Spec portado: `actor-tests/src/test/scala/org/apache/pekko/actor/ProviderSelectionSpec.scala:38-43` (A1), `:45-51` (A2), `:53-59` (A3), `:61-67` (A4).

## Casos nuevos: 2596 → 2627 (31)

| Familia | IDs | Casos | Aserción cubierta |
|---|---|---|---|
| provider-selection | ps-0001..0013 | 13 | A1 (`local`, SEL-01), A2 (`remote` y su clase, SEL-02/04), A3 (`cluster` y su clase con `hasCluster`, SEL-03/05), A4 (`other.ActorRefProvider`, SEL-06); coincidencia exacta: clase de Local, mayúsculas, espacios, vacío y un número (SEL-07..13) |
| provider-selection | ps-0014..0019 | 6 | `ProviderSelection("remote") == ProviderSelection(RemoteActorRefProvider)` (`:48`) y su par de cluster (`:56`); Local frente a su clase; dos `Custom` (EQ-01..06) |
| provider-selection | ps-0020..0025 | 6 | Un valor que no es texto: lista, lista vacía y objeto (`WrongType`), `null` (`ConfigException.Null`), booleano y decimal (SEL-14..19); cubren la rama de error del `?` de `Settings::new` |
| dynamic-access | da-0021..0026 | 6 | El proveedor que `ActorSystem` carga por nombre corto o por otra cadena: `remote` y `cluster` (`ClassNotFoundException` con el nombre de clase), `local` (se crea), `LOCAL`, `other.ActorRefProvider` y vacío (PRV-05..10) |

Los casos con los nombres cortos `remote` y `cluster` son SEL-02/03, EQ-01/02 y PRV-05/06. Se grabaron con el oráculo (JDK 17, openjdk@17 17.0.21, Q-X-16); no dependen de la JDK y no se instaló JDK 21.

```
SEL-01 provider = local -> ok identifier=local fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
SEL-02 provider = remote -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
SEL-03 provider = cluster -> ok identifier=cluster fqcn=org.apache.pekko.cluster.ClusterActorRefProvider hasCluster=true ProviderClass=org.apache.pekko.cluster.ClusterActorRefProvider HasCluster=true
SEL-04 provider = org.apache.pekko.remote.RemoteActorRefProvider -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
SEL-05 provider = org.apache.pekko.cluster.ClusterActorRefProvider -> ok identifier=cluster fqcn=org.apache.pekko.cluster.ClusterActorRefProvider hasCluster=true ProviderClass=org.apache.pekko.cluster.ClusterActorRefProvider HasCluster=true
SEL-06 provider = other.ActorRefProvider -> ok identifier=custom fqcn=other.ActorRefProvider hasCluster=false ProviderClass=other.ActorRefProvider HasCluster=false
SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=custom fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
SEL-08 provider = LOCAL (case differs) -> ok identifier=custom fqcn=LOCAL hasCluster=false ProviderClass=LOCAL HasCluster=false
SEL-09 provider = Remote (case differs) -> ok identifier=custom fqcn=Remote hasCluster=false ProviderClass=Remote HasCluster=false
SEL-10 provider = " local" (leading space) -> ok identifier=custom fqcn= local hasCluster=false ProviderClass= local HasCluster=false
SEL-11 provider = "cluster " (trailing space) -> ok identifier=custom fqcn=cluster  hasCluster=false ProviderClass=cluster  HasCluster=false
SEL-12 provider = empty string -> ok identifier=custom fqcn= hasCluster=false ProviderClass= HasCluster=false
SEL-13 provider = 5 (a number) -> ok identifier=custom ProviderClass=5
EQ-01 remote equals org.apache.pekko.remote.RemoteActorRefProvider -> ok true
EQ-02 cluster equals org.apache.pekko.cluster.ClusterActorRefProvider -> ok true
EQ-03 local equals org.apache.pekko.actor.LocalActorRefProvider -> ok false
EQ-04 two Custom with the same name -> ok true
EQ-05 two Custom with different names -> ok false
EQ-06 remote equals cluster -> ok false
SEL-14 provider = [local] (a list) -> ERR WrongType [String: 1: pekko.actor.provider has type LIST rather than STRING]
SEL-15 provider = [] (an empty list) -> ERR WrongType [String: 1: pekko.actor.provider has type LIST rather than STRING]
SEL-16 provider = {a=1} (an object) -> ERR WrongType [String: 1: pekko.actor.provider has type OBJECT rather than STRING]
SEL-17 provider = null -> ERR Null [String: 1: Configuration key 'pekko.actor.provider' is set to null but expected STRING]
SEL-18 provider = true (a boolean) -> ok identifier=custom ProviderClass=true
SEL-19 provider = 1.5 (a decimal number) -> ok identifier=custom ProviderClass=1.5
```

```
PRV-05 provider remote -> ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]
PRV-06 provider cluster -> ERR ClassNotFoundException [org.apache.pekko.cluster.ClusterActorRefProvider]
PRV-07 provider local -> ok created
PRV-08 provider LOCAL (case differs) -> ERR ClassNotFoundException [LOCAL]
PRV-09 provider other.ActorRefProvider -> ERR ClassNotFoundException [other.ActorRefProvider]
PRV-10 provider empty string -> ERR ClassNotFoundException []
```

Verificación independiente: dos ejecuciones de `oracle/run.sh provider-selection-all` idénticas (`cmp`) y `diff` vacío frente a `classic-parity provider-selection-all` (25/25); lo mismo para `dynamic-access-all` (26/26).

## Premisa verificada (valores que no son texto)

Antes de grabar SEL-14..19 se comprobó con una segunda fuente que la rama de error de `?` es alcanzable: `checkValid` acepta cualquier valor para una clave que es `STRING` en el reference, y el fallo llega desde `getString`. Pekko 2.0.0 y Rust dieron la misma salida byte a byte para `5`, `true`, `[local]`, `[]`, `{a=1}`, `null` y `1.5`. `provider = 5` no es un error: Pekko lo lee como la cadena `"5"` (SEL-13).

## ROJO

Tests unitarios con un stub de cuerpos incorrectos (`cargo test -p rakka-actor --lib provider`):

```
test config::provider_selection::tests::the_local_class_name_is_a_custom_provider ... ok
test config::provider_selection::tests::two_custom_selections_are_equal_only_with_the_same_class_name ... ok
test config::provider_selection::tests::any_other_string_is_a_custom_provider_with_that_class_name ... FAILED
test config::provider_selection::tests::remote_by_short_name_and_by_class_name_is_the_same_selection ... FAILED
test config::provider_selection::tests::cluster_by_short_name_and_by_class_name_is_the_same_selection_and_has_a_cluster ... FAILED
test config::provider_selection::tests::local_selects_the_local_provider_class_and_has_no_cluster ... FAILED
test config::provider_selection::tests::short_names_match_exactly ... FAILED
test config::settings::tests::the_provider_class_and_cluster_flag_follow_the_provider_key ... FAILED
test result: FAILED. 2 passed; 6 failed; 0 ignored; 0 measured; 362 filtered out; finished in 0.02s
```

6 de 8 fallaban. Los 2 que pasaban con el stub: `two_custom_selections_are_equal_only_with_the_same_class_name` comprueba la igualdad derivada de la enum, que el stub (todo `Custom`) cumplía por construcción (se mantiene: es una propiedad legítima de `PartialEq`); `the_local_class_name_is_a_custom_provider` no ejercitaba la implementación y se reforzó con las aserciones de `identifier()` y `fqcn()`, con lo que falla con el stub:

```
test config::provider_selection::tests::the_local_class_name_is_a_custom_provider ... FAILED
assertion `left == right` failed
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 369 filtered out; finished in 0.00s
```

Casos grabados, antes de grabar `ps-0020..0025`, el ROJO de la prueba grabada:

```
oracle case SEL-14 provider = [local] (a list) is neither recorded nor listed as not produced
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 1.02s
```

## VERDE

```
test config::provider_selection::tests::local_selects_the_local_provider_class_and_has_no_cluster ... ok
test config::provider_selection::tests::cluster_by_short_name_and_by_class_name_is_the_same_selection_and_has_a_cluster ... ok
test config::provider_selection::tests::remote_by_short_name_and_by_class_name_is_the_same_selection ... ok
test config::provider_selection::tests::any_other_string_is_a_custom_provider_with_that_class_name ... ok
test config::provider_selection::tests::two_custom_selections_are_equal_only_with_the_same_class_name ... ok
test config::provider_selection::tests::the_local_class_name_is_a_custom_provider ... ok
test config::provider_selection::tests::short_names_match_exactly ... ok
test config::settings::tests::the_provider_class_and_cluster_flag_follow_the_provider_key ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 362 filtered out; finished in 0.12s
```

```
test only_a_jdk17_double_text_difference_is_tolerated ... ok
test recorded_dynamic_access_cases_match_pekko ... ok
test recorded_path_from_string_cases_match_pekko ... ok
test recorded_config_hocon_cases_match_pekko ... ok
test recorded_provider_selection_cases_match_pekko ... ok
test recorded_util_core_cases_match_pekko ... ok
test recorded_path_address_cases_match_pekko ... ok
test recorded_config_settings_cases_match_pekko ... ok
test recorded_scenario_cases_match_pekko ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.82s
```

## Replay externo completo (`uv run tools/parity/replay.py`)

Referencia tras el PR 1: 2596 = 2595 + 1 NA. Ahora: `2626/2626 applicable + 0 NA:Q-CFG-30 + 1 not applicable (diverge 0)`; `breakdown: pass 2626 / NA:Q-CFG-30 0 / other NA 1 / diverge 0`. Total 2627 = 2596 + 31.

## Mutantes (`tools/parity/check_divergence.py`, 9 nuevos y control)

Los mutantes corrieron sobre **2621 casos** (corpus `2620` aplicables + 1 NA), antes de añadir `ps-0020..0025`; esos 6 casos solo añaden casos grabados y arnés, sin cambios de producción.

```
| mutant | outcome | passed/total |
| provider-remote-short-name-ignored | DETECTED | 2617/2620 |
| provider-cluster-class-name-ignored | DETECTED | 2618/2620 |
| provider-remote-has-cluster | DETECTED | 2618/2620 |
| provider-local-class-name-is-local | DETECTED | 2618/2620 |
| provider-selection-trims-the-value | DETECTED | 2618/2620 |
| provider-selection-ignores-case | DETECTED | 2610/2620 |
| provider-custom-identifier-changed | DETECTED | 2612/2620 |
| settings-provider-class-is-the-identifier | DETECTED | 2607/2620 |
| settings-has-cluster-always-false | DETECTED | 2618/2620 |
| control (unmodified) | CLEAN | 2620/2620 |
```

`detected 9/9`, UNDETECTED 0, PATCH_ERROR 0, BUILD_ERROR 0, TIMEOUT 0. `--check-patterns`: 154 patrones, 0 obsoletos. La regresión completa de mutantes no se corrió (la corre el Owner).

## Gates

- `cargo fmt --all --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok (0 avisos); `cargo deny check licenses` ok; `cargo test --workspace`: 529 passed, 0 failed (26 líneas `test result`).
- Cobertura (`cargo llvm-cov -p rakka-actor --lib --tests --fail-under-lines 95`, sin exclusiones): rakka-actor **98.28 % de líneas (12228/12442), 98.13 % de regiones (16450/16764), 98.30 % de funciones**; rakka-actor-typed no tiene nada instrumentado. `provider_selection.rs`: 100 % líneas y regiones; `dynamic_access.rs`: 99.49 % líneas, 99.56 % regiones (línea 202, test-only, sin cubrir); `settings.rs`: 98.83 % líneas, 94.58 % regiones.
- Rama de error del `?` de `pekko.actor.provider` (`settings.rs:175`, segmento `175:90`): **count 0 antes de grabar SEL-14..17, count 4 después** (regiones sin cubrir de `settings.rs`: 21 antes, 20 ahora).
- Lo que sigue sin cubrir en `settings.rs` son ramas de error de `?` del constructor que ya existían (P08, p. ej. la línea 187 `get_string_list("pekko.actor.no-serialization-verification-needed-class-prefix")?`) y de `amend_slf4j_config` (líneas 367, 371, 385); este PR no las toca. Ningún caso grabado llega a ellas. Se pueden cubrir como esta, con valores de tipo equivocado grabados con el oráculo.
- `uv run tools/package/check.py`: failures=0; tests de las tools: 35, OK.

## Filas

| Fila | Antes | Ahora | Por qué |
|---|---|---|---|
| AC-CFG-008 | pending / pending | partial / oracle | Q-CFG-31 (propuesta); AC-SYS-018 (P25: el sistema aún no crea el proveedor); AC-CFG-015 (PR 3: prioridad de `BootstrapSetup`); Q-CFG-9 (c) con el registro de fábricas del sistema |

## Límites conocidos

- `ProviderSelection` no carga la clase: resolver el nombre es del `FactoryRegistry` (PR 1) y de AC-SYS-018. La prueba de integración "crear el sistema con un proveedor no registrado falla" espera a P25.
- Los casos de `Settings` con `String: N` en el mensaje coinciden con Pekko para estos valores; los demás casos de origen siguen con Q-CFG-15.
