# Evidencia del slice P10, PR 1: AC-CFG-013 (Deploy, Scope y withFallback), 2026-11-20

Fila: AC-CFG-013. **Resultado: `partial`, validación `oracle`** (antes `pending`/`pending`). AC-CFG-012.2 y AC-CFG-012.1 son los PR 2 y 3 de P10. AC-CFG-012.3/.4 (routers y resizers) se difieren a P37 (costura DEPLOY-S1).

## Alcance

`actor::{Deploy, Scope, RouterConfig}` (`actor/deploy.rs`), `Config::same_value` y `Config::render_concise` (`config/mod.rs`). Oráculo nuevo `DeployOracle.java` (familia `deploy`, prefijo de caso `dp`; `oracle/run.sh deploy-ids|deploy-all|deploy-case`).

## Contrato y citas de Pekko 2.0.0

`Deployer.scala:28-155` (`Deploy`, `withFallback` en `:104-112`, igualdad en `:134-147`), `:157-205` (`Scope`, `LocalScope`, `NoScopeGiven`), `RouterConfig.scala:94` y `:373` (`withFallback` por defecto y el de `NoRouter`). Specs: `DeployerSpec.scala:93-142` (el `Deploy` leído de la configuración; es del PR 3).

Dos hallazgos del contrato verificados en el código de 2.0.0: (1) `withFallback` no copia los `tags` (`Deployer.scala:105-112`; el contrato decía `:109-117`); (2) el `path` del resultado es el de `this`.

## Decisiones de diseño

- **`Scope`** solo tiene `Local` y `NoScopeGiven` y es `#[non_exhaustive]`. La propuesta del contrato (`Other(Box<dyn ..>)`) no se implementa: en el módulo actor nadie define otro scope (`RemoteScope` vive en otro módulo), así que un objeto de rasgo sería una abstracción sin consumidor. Añadir una variante después es compatible.
- **`RouterConfig::Opaque`** solo nombra el router (su clase). Limitado a lo que P10 necesita: `NoRouter.withFallback(x) = x` y cualquier otro router se queda consigo mismo (`RouterConfig.scala:94,373`). La fusión de dos pools reales (`overrideUnsetConfig`) y el texto de un router son de P37; los casos del oráculo imprimen un router solo por su clase y declaran ese límite.
- **Igualdad de `Deploy`:** el `PartialEq` derivado de `Config` incluye la línea de origen, y `Config.equals` de Java no (RED observado abajo). `Config::same_value` compara por valores; un número entero equivale al mismo número escrito como decimal (`1` y `1.0`, medido en DP-16).
- **`toString`:** no se implementa `Display` para `Deploy` (Q-CFG-35, `proposed`, sin decidir). Corrección a una versión anterior de este plan: no es cierto que `Config.toString` dependa del orden de un HashMap; medido en la JVM (JDK 17 y 27) las claves salen ordenadas. Lo no determinista es el orden de los `tags` con más de 4 elementos y el texto de los routers.

## Casos nuevos: 2669 → 2713 (44)

`dp-0001..dp-0044`, grabados con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, JDK 17, Q-X-16; nunca Rust). Ids: DP-01..21 (valores e igualdad), FB-01..15 (`Deploy.withFallback`), SC-01..04 (`Scope.withFallback`), RT-01..04 (`RouterConfig.withFallback` de `NoRouter`). Los 44 ids del oráculo los produce Rust, 0 sin producir. Determinismo: dos ejecuciones idénticas; con JDK 27 (`openjdk` 27.0.1) la salida es idéntica byte a byte; no se regrabó nada con JDK 27. `cases/coverage.tsv` los lista con AC-CFG-013.

## ROJO / VERDE

**ROJO unitario (real).** Con la API presente pero con cuerpos mínimos incorrectos (`with_fallback` devolvía `self.clone()`, `Deploy::local()` el valor por defecto, y la igualdad usaba el `PartialEq` derivado de `Config`), `cargo test -p rakka-actor --lib actor::deploy` dio **7 fallos y 2 aciertos** de 9 (salida guardada en la sesión):

```
test actor::deploy::tests::the_local_scope_keeps_itself_and_no_scope_given_takes_the_other ... FAILED
test actor::deploy::tests::the_local_deploy_is_the_default_with_the_local_scope ... FAILED
test actor::deploy::tests::the_dispatcher_and_mailbox_of_the_receiver_win_unless_they_are_empty ... FAILED
test actor::deploy::tests::no_router_takes_the_other_router_and_a_router_keeps_itself ... FAILED
test actor::deploy::tests::the_tags_are_not_carried_by_a_fallback ... FAILED
test actor::deploy::tests::two_deploys_with_the_same_content_are_equal_whatever_the_text_of_their_configuration ... FAILED
test actor::deploy::tests::the_configurations_are_merged_with_the_receiver_first ... FAILED
test result: FAILED. 2 passed; 7 failed; 0 ignored; 0 measured; 382 filtered out
```

El de igualdad fallaba porque `Config` derivado distinguía `line: 1` de `line: 3` para el mismo contenido.

**VERDE unitario:** los 9 tests pasan tras implementar (`test result: ok. 9 passed`).

**Sin ROJO diferencial:** el gemelo Rust de los casos se escribió contra la API ya implementada y coincidió con el oráculo en la primera comparación (44/44 líneas idénticas), así que no hay un replay en rojo previo; la detección de los casos la demuestran los mutantes. Los 2 tests unitarios de `Config::same_value` y `render_concise` se escribieron después de la implementación (sin ROJO propio; los cubre el RED de igualdad de `Deploy` y los mutantes `config-equality-*`).

## Replay externo completo (`uv run tools/parity/replay.py`)

**2712/2712 aplicables + 0 NA:Q-CFG-30 + 1 no aplicable, divergen 0** (total 2713 = 2669 de la base `c21bef3` + 44).

## Mutantes (`tools/parity/check_divergence.py`, 11 nuevos y control)

Lanzados sobre el árbol final, con el código de producción congelado; control CLEAN **2712/2712 aplicables + 1 NA**.

| Mutante | Resultado | Pasan |
|---|---|---|
| deploy-fallback-prefers-the-other-dispatcher | DETECTED | 2709/2712 |
| deploy-fallback-never-takes-the-other-mailbox | DETECTED | 2709/2712 |
| deploy-fallback-takes-the-path-of-the-other | DETECTED | 2698/2712 |
| deploy-fallback-keeps-the-tags | DETECTED | 2711/2712 |
| local-scope-lets-the-other-win | DETECTED | 2710/2712 |
| no-scope-given-keeps-itself | DETECTED | 2710/2712 |
| no-router-keeps-itself | DETECTED | 2710/2712 |
| router-takes-the-other-router | DETECTED | 2709/2712 |
| deploy-equality-compares-the-text-of-the-configuration | DETECTED | 2710/2712 |
| config-equality-compares-numbers-by-their-text | DETECTED | 2711/2712 |
| config-equality-ignores-the-extra-keys-of-the-other | DETECTED | 2711/2712 |

11/11 DETECTED, 0 sobrevivientes. Patrones de `MUTANTS`: 167, 0 obsoletos (cada texto original aparece exactamente una vez en su archivo, incluidos los 2 mutantes anteriores de `config/mod.rs`, que sí cambió). Se ejecutaron solo los mutantes nuevos: ningún otro archivo con mutantes existentes cambió (el único cambio en `settings.rs` es un comentario). La regresión completa la corre el Owner.

## Gates

- `cargo test --workspace`: 556/0. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` y `cargo deny check licenses`: ok.
- Cobertura (`cargo llvm-cov --workspace --summary-only`): líneas 98,32 %, regiones 98,16 %, funciones sin cambio relevante; `actor/deploy.rs` 0 líneas y 0 regiones sin cubrir. Totales sin cubrir: 217 líneas y 318 regiones, **los mismos que la base**: ninguna línea ni región nueva sin cubrir. (Una primera medición dio +3 y +3: eran dos ramas inalcanzables de `same_number` y `same_object`, que se eliminaron reestructurando el código antes de lanzar los mutantes.)
- `uv run tools/package/check.py`: 0 fallos; tests de las tools OK.

## Etapas de Rust-Anything efectivamente ejecutadas

El MCP no conectó en esta sesión; se usó el CLI. Ejecutado: `rust-anything app record` (44 casos, contra `oracle/run.sh`) y el replay (`tools/parity/replay.py`, que envuelve `rust-anything app replay`). No se ejecutaron `analyze`, `bridge`, `shadow` ni `gate`: el crate ya existe y el flujo del repositorio es el oráculo Java más `app record/replay` (Q-X-12).

## Filas

| Fila | Antes | Después | Por qué |
|---|---|---|---|
| AC-CFG-013 | pending / pending | partial / oracle | Q-CFG-35 (propuesta: `toString`); la fusión de pools reales y el texto del router son de P37; `Props.deploy` y la fusión de AC-SYS-016 (no hay sistema) |

## Decisiones propuestas

Q-CFG-35 (`proposed`): `toString` de `Deploy`. Ninguna decisión aceptada se modificó; `owner-decisions.tsv` sin tocar.

## Límites conocidos

Un router se compara solo por su clase. `Pool.withFallback(Pool)` (que fusiona resizer, estrategia de supervisión y dispatcher) no está cubierto: Rust devuelve el receptor y el oráculo se imprime por clase, así que esa fusión queda sin verificar hasta P37.
