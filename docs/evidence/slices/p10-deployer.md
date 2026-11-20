# Evidencia del slice P10, PR 2: AC-CFG-012.2 (Deployer.deploy y lookup), 2026-11-20

Fila: AC-CFG-012.2. **Resultado: `partial`, validación `oracle`** (antes `pending`/`pending`). Sigue `partial` porque falta AC-CFG-012.1 (leer `pekko.actor.deployment`, PR 3) y AC-UTIL-009 sigue `partial` por Q-UTIL-21 (qué `prefix*` gana cuando dos coinciden). Base: `9b6d5cc` (#276 mergeado); upstream `9fbd2283b1cf701c90a1223bf621942c191db61f`.

## Alcance

`actor::Deployer` (`actor/deployer.rs`): `deploy`, `lookup(elementos)` y `lookup_path(ActorPath)`. Reutiliza `Deploy`, `WildcardIndex` y `validate_path_element`. Oráculo nuevo `DeployerOracle.java` (familia `deployer`, prefijo `dr`; `oracle/run.sh deployer-ids|deployer-all|deployer-case`). Corrección del comentario de `Config::same_value`. Sin routers, sin `ActorSystem`, sin `from_settings` (P10 3/3 y P37).

## Contrato (Pekko 2.0.0)

`Deployer.scala:252-268`, `WildcardIndex.scala`. `deploy` hace `d.path.split("/").drop(1)`, rechaza un elemento vacío (`InvalidActorNameException: Actor name in deployment [<path>] must not be empty`), valida el resto con `validatePathElement(el, fullPath = d.path)` y recién entonces inserta con `compareAndSet`. `lookup(ActorPath)` descarta el primer elemento (el guardián).

Hallazgos medidos en el oráculo: `deploy a/b` guarda `b` (se descarta el primer elemento aunque no sea vacío); `/`, `""`, `//` y un solo elemento no son un despliegue (no hay error); un `**` como sufijo (`/c**`) es `IllegalArgumentException` y no cambia nada; un índice con solo `prefix*` no encuentra nada (quirk de `isEmpty`, ya en `WildcardIndex`); el primer elemento inválido o vacío en orden de path es el que se informa.

## Decisiones de diseño

- **Alta atómica:** el índice es una instantánea inmutable bajo un `Mutex`; se construye el índice nuevo antes de reemplazar, así un alta rechazada no deja nada. Serializar equivale al reintento del `compareAndSet` (no se pierde ninguna alta). Sin dependencia nueva.
- **`java_split_slash`** (privada): quita los vacíos finales. Java da `[""]` para el texto vacío; no se replica porque tras `drop(1)` es inobservable (un mutante que lo quitaba sobrevivió, por eso se eliminó la rama).
- **Fuera de dominio (Q-X-17):** un sustituto UTF-16 suelto en una ruta no cabe en `&str`; ningún guion lo usa (se quitó DL-41). La pareja de sustitutos sí (DL-40).
- **Oráculo:** el `Deployer` real de Pekko, con su campo privado `deployments` vaciado por reflexión: el constructor ya despliega las entradas `/IO-DNS/*` de `reference.conf`, que `Settings` exige, y leer esa sección es 012.1. `deploy` y `lookup` son los reales.
- `RouterConfig::Opaque` (riesgo de API pública de #276) **se conserva como pendiente de P37**; no se tocó.
- **Comentario de `Config::same_value`:** decía que dos valores sin resolver son iguales "solo cuando son idénticos, línea incluida". Era impreciso: la línea del valor sin resolver no se compara; solo cuentan las líneas de las piezas-valor dentro de una concatenación y el texto de los números. No pude leer el texto exacto del comentario de la revisión (no hay comentarios en #276 vía `gh`); esta es mi lectura de lo impreciso y conviene que el revisor la confirme.

## Casos nuevos: 2713 → 2773 (60)

`dr-0001..dr-0060`, grabados con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, JDK 17; nunca Rust). 58 guiones `DL-01..DL-59` (sin DL-41) en `crates/rakka-actor/examples/classic-parity/deployer.tsv`, que leen el oráculo y el gemelo Rust: comodines `*`, `prefix*`, `**` y precedencia; guardián; rutas inválidas, segmentos vacíos, barras finales, Unicode y percent-encoding; atomicidad tras un fallo; y 2 casos de concurrencia `DC-01/02` con `CyclicBarrier` (8 hilos; imprimen invariantes, sin sleeps). Dos ejecuciones del oráculo idénticas. Primera comparación gemelo vs oráculo: 60/60 idénticas. `cases/coverage.tsv` los lista con AC-CFG-012.2. Todos los ids los produce Rust (0 sin producir).

## ROJO / VERDE

**ROJO unitario (real).** Con la API presente y cuerpos mínimos incorrectos (`deploy` no hacía nada, `lookup` devolvía `None`, `split` sin quitar vacíos), `cargo test -p rakka-actor --lib actor::deployer` dio **16 fallos y 1 acierto** de 17 (salida: `17` líneas FAILED; `test result: FAILED. 1 passed; 16 failed`). Ejemplos: `split ... left: ["", "a", "", ""] right: ["", "a"]`; `the_guardian_of_an_actor_path_is_dropped_by_lookup ... left: None right: Some("/service1")`; el de concurrencia falló porque ninguna alta se guardó.

**VERDE:** 17/17 tras implementar. Un fallo posterior fue un esperado mío mal escrito (el `Display` de `ActorError` usa el nombre completo de la clase), corregido en el test.

**Sin ROJO diferencial:** el gemelo se escribió contra la API ya implementada y coincidió en la primera comparación; la detección la demuestran los mutantes.

## Replay externo completo (`python3.12 tools/parity/replay.py`)

**2772/2772 aplicables + 0 NA:Q-CFG-30 + 1 no aplicable, divergen 0** (2713 + 60 = 2773); repetido sobre el árbol final tras la revisión (control de los mutantes, CLEAN 2772/2772).

## Mutantes (10, `tools/parity/check_divergence.py`)

Tras la revisión independiente se reescribió `deploy` como el `compareAndSet` literal de Pekko (instantánea, índice nuevo, intercambio solo si la instantánea sigue vigente, si no se reintenta) y se relanzaron **todos** los mutantes de `deployer.rs` sobre el árbol final:

| Mutante | Resultado | Pasan |
|---|---|---|
| deployer-lookup-keeps-the-guardian | DETECTED | 2766/2772 |
| deployer-deploy-keeps-the-first-element | DETECTED | 2712/2772 |
| deployer-accepts-an-empty-element | DETECTED | 2766/2772 |
| deployer-skips-the-validation-of-the-elements | DETECTED | 2764/2772 |
| deployer-validation-message-omits-the-path | DETECTED | 2764/2772 |
| deployer-split-keeps-the-trailing-empty-parts | DETECTED | 2769/2772 |
| deployer-deploy-forgets-the-previous-deployments | DETECTED | 2757/2772 |
| deployer-deploy-stores-nothing | DETECTED | 2727/2772 |
| deployer-empty-element-message-is-wrong | DETECTED | 2766/2772 |
| deployer-deploy-swaps-a-stale-snapshot | DETECTED (test unitario) | - |

10/10 DETECTED, control CLEAN 2772/2772. Patrones: 185, 0 obsoletos. El mutante de **actualización perdida** (`swaps-a-stale-snapshot`) ya **no depende del scheduler**: lo mata el test unitario determinista `a_deployment_that_loses_the_swap_is_retried_on_the_newer_index_and_nothing_is_lost` (una sola hebra, sin sleeps), que usa el hook privado `deploy_between_read_and_swap` para hacer otra alta justo entre la lectura y el intercambio. En producción el hook es una clausura vacía. El harness (`UNIT_MUTANTS`) ejecuta ese test en lugar del replay para ese mutante; la salida del mutante fue `test result: FAILED`. DC-01/DC-02 quedan en el replay como comprobación del estado final, no como matador. (Un mutante anterior, equivalente, se eliminó en la primera versión.)

## Revisión independiente de #277 (agente nuevo, sin el contexto de la implementación), SHA `1a8e13f`: APPROVE WITH MINORS

Sin bloqueantes. Atendido en este SHA: **F1** (menor, real: mis `\uXXXX` de DL-36 y DL-51 se habían convertido en caracteres literales al escribir el archivo y el espacio se perdía al tokenizar; casos vacuos) corregido, el archivo es ahora ASCII puro y `dr-0036` se regrabó con el oráculo (valida ahora `/ ` y `b c`); **F3** (`lookup` esperaba al `Mutex` durante el `insert`) resuelto con la instantánea y el intercambio breve; **F7** (sin mutante del texto del elemento vacío ni de "no guarda") añadidos; **F8** (determinismo) resuelto arriba; **F9** y **F10** (encabezado y líneas en blanco); **F11** (el hueco DL-41 se menciona en la cabecera de `deployer.tsv`).

Documentado y **no resuelto** (decisión del Owner o de otros slices):
- **F2 / Q-UTIL-21 (hechos medidos por el revisor contra el `WildcardIndex` real de Pekko):** con dos `prefix*` solapados el ganador sigue el orden de hash del `HashMap` de Scala, no el de inserción: `p*`/`pq*` da `p*` y `s*`/`st*` da `st*` en las dos órdenes de inserción; `a*`/`ab*` da `a*`, `x*`/`xy*` da `x*`, `m*`/`mn*` da `m*`, `b*`/`bc*` da `bc*`. Rust elige el primero insertado, así que difiere de Pekko en una de las dos órdenes de cada par. El orden es determinista (`String.hashCode` en el trie CHAMP). Ningún caso grabado lo cubre; no se decidió nada (Q-UTIL-21 sigue propuesta).
- **F4:** `Deployer` es `private[pekko]` en Pekko y aquí es público (`pub use`): fija superficie semver; decidir `pub(crate)` o `#[doc(hidden)]` antes del primer release. `lookup` devuelve un `Deploy` clonado (Pekko devuelve una referencia compartida): un `Arc<Deploy>` en el índice lo abarataría.
- **F5 (para P10 3/3):** `from_settings` es compatible (`Deployer::new()` y `deploy` por entrada, abortando en el primer `Err`), pero Pekko itera `config.root` en el orden del `HashMap` de Typesafe Config: ese orden decide qué alta gana entre `/a` y `/a/` (ambas dan `["a"]`) y el caso de Q-UTIL-21. 012.1 debe replicarlo o registrar una `Q`.
- **F12 (nit):** desplegar `/`, `""` o un elemento clona el índice entero sin cambiar nada.

## Gates

- `cargo test --workspace` verde (410 unitarios de la lib + integración); `clippy --all-targets -D warnings`, `cargo fmt --all --check`, `cargo deny check licenses`: ok.
- Cobertura (`cargo llvm-cov --workspace --summary-only`): líneas **98,20 %**, regiones 98,35 %, funciones 98,35 %; `actor/deployer.rs` y `actor/deploy.rs` 0 líneas, 0 regiones y 0 funciones sin cubrir; sin líneas nuevas sin cubrir (regiones sin cubrir del workspace: 319 frente a 318 en la base de #276; la diferencia está fuera de `deployer.rs`, que tiene 0; no la investigué más).
- `python3.12 tools/package/check.py`: 0 fallos (tras `generate.py`).

## Etapas de Rust-Anything ejecutadas

CLI (el MCP no se usó): `rust-anything app record` (60 casos contra `oracle/run.sh`) y el replay (`tools/parity/replay.py`). No se ejecutaron `analyze`, `bridge`, `shadow` ni `gate`: el crate ya existe y el flujo del repositorio es oráculo Java + `app record/replay` (Q-X-12).

## Riesgos y pendientes

- AC-CFG-012.2 queda `partial/oracle`: 012.1 (PR 3) y Q-UTIL-21 abiertos.
- `RouterConfig::Opaque` en la API pública: pendiente de P37.
- Q-CFG-35 (`Display` de `Deploy`) sigue propuesta; no se decidió ninguna `Q-*`.
- Q-UTIL-21 (arriba) y el orden de iteración de 012.1 (F5): diferencias conocidas respecto de Pekko que no se han resuelto.
