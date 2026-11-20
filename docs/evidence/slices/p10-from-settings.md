# Evidencia del slice P10, PR 3: AC-CFG-012.1 (`Deployer::from_settings`), 2026-11-20

Fila: AC-CFG-012.1. **Resultado: `partial`, validación `oracle`** (antes `pending`/`pending`). P10 queda con AC-CFG-013 `partial/oracle` (Q-CFG-35), AC-CFG-012.2 `partial/oracle` (Q-UTIL-21) y AC-CFG-012.1 `partial/oracle` (Q-CFG-36, y el `ActorSystem`/provider de AC-SYS-016 y los routers de P37). Base: `b490881` (#277 mergeado); upstream `9fbd2283b1cf701c90a1223bf621942c191db61f`.

## Alcance

`Deployer::from_settings(&Settings)` y `DeployerError` (`actor/deployer.rs`), y `Config::object_entries` (`config/mod.rs`, crate interno). Oráculo nuevo `DeployerSettingsOracle.java` (familia `deployer-settings`, prefijo `ds`; `oracle/run.sh deployer-settings-ids|-all|-case`) con los guiones de `deployer_settings.tsv`. Sin type-mapping, pools ni validaciones de routers (P37), sin `ActorSystem`/provider, sin `Props.deploy`, sin `DynamicAccess`.

## Contrato (Pekko 2.0.0, `Deployer.scala:225-250` y `270-276`)

El constructor lee `pekko.actor.deployment`; `default` se descarta; cada entrada que es un objeto pasa por `parseConfig`: `deployment = entrada.withFallback(default)`, `dispatcher` y `mailbox` de esa configuración, `scope = NoScopeGiven`; las entradas que no son objeto se ignoran; después `foreach(deploy)`. El `flatMap` es estricto: se leen todas las entradas antes de desplegar la primera (DS-24: un `WrongType` gana a un path inválido, medido en el oráculo).

## Decisiones del Owner aplicadas (2026-11-20)

1. Claves en colisión: orden alfabético (`BTreeMap`), gana la última. **Temporal**, no es aceptación definitiva. Registrada como **Q-CFG-36 (`proposed`)** con la evidencia de abajo.
2. No se rechazan las colisiones (Pekko las acepta) ni se emula todavía el `HashSet` interno de Typesafe Config.
3. Routers: `RouterConfig::Opaque` conserva el nombre **configurado** (no la clase del type-mapping) cuando el router no es `from-code`. No se implementan type-mapping, pools ni validaciones; A5 (`nr-of-instances = boom`) y los errores de construcción de routers siguen pendientes de P37 (hoy una entrada con un router desconocido o con valores inválidos se acepta en Rust, y Pekko lanza `IllegalArgumentException`/`ConfigException`).

## Q-CFG-36: evidencia (y una corrección a lo informado antes)

- `javap` sobre `SimpleConfigObject` (config 1.4.9): `entrySet()` construye un `java.util.HashSet` de `AbstractMap.SimpleImmutableEntry`. Pekko itera ese conjunto, cuyo orden depende del hash de **clave y valor** de cada entrada y del tamaño de la tabla.
- 99 conjuntos de 2 a 4 claves en colisión (`p10-from-settings-collisions-probe.java`, con el dispatcher de cada entrada igual a su clave): el ganador no depende del orden de escritura (0 de 99) y el último en orden alfabético difiere del ganador de Pekko en **53 de 99**.
- **Corrección:** la primera medición presentada al Owner atribuía el orden solo a las claves. Es falso: con las mismas tres claves `/svc`, `/svc/`, `/svc//` y dispatchers `d1,d2,d3` gana `/svc`; con `aaa,bbb,ccc` gana `/svc//`; con `x,y,z` gana `/svc/` (`p10-from-settings-value-dependence-probe.java`). La cifra 53/99 vale para ese patrón de valores. Un modelo con `HashMap` de las claves acierta 47/99 (azar). Reproducir el orden exige emular el `hashCode` de los `ConfigValue` y el tamaño de la tabla: alternativa (b) de Q-CFG-36, más cara de lo estimado.
- Los casos `CO-01` a `CO-03` coinciden con Pekko por casualidad (se graban y pasan); `CO-04` y `CO-05` divergen y están **pendientes** en el gemelo Rust (`PENDING`, con su motivo) y listados en `cases/oracle-ids/deployer-settings.txt`. El orden decide además qué error se informa cuando varias entradas son inválidas (revisor independiente: `/a`,`/b`,`/c` con tipo incorrecto, Pekko `String: 4:`, Rust `String: 2:`; `/a b`,`/c d`,`/e f`, Pekko `[e f]`, Rust `[a b]`), sin casos grabados. La política de Rust la fija además el test unitario `of_several_keys_with_the_same_path_the_last_in_alphabetical_order_wins`.

## Casos nuevos: 2773 → 2811 (38)

`ds-0001..ds-0038`, grabados con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, JDK 17; nunca Rust). 40 ids en el oráculo: 35 `DS` de **paridad** (sección vacía con las entradas de `reference.conf`, entrada sin valores, dispatcher y mailbox propios y de `default`, routers por nombre, comodines, entradas que no son objeto, `default` no es un despliegue, errores de path y de tipo, el orden de fases, Unicode y pares de sustitutos, símbolos válidos, claves raíz/vacías/sin `/` inicial, `lookup(ActorPath)`) y 5 `CO` de **divergencia autorizada** (3 grabados, 2 pendientes). Cada línea imprime también la configuración efectiva (`render_concise`) del `Deploy`, así que se compara el fallback completo, no solo dos campos. Dos ejecuciones del oráculo idénticas; gemelo Rust vs oráculo: 38/38 en la primera comparación.

## ROJO / VERDE

**ROJO unitario (real):** con `from_settings` devolviendo un `Deployer` vacío, `cargo test -p rakka-actor --lib actor::deployer` dio **11 fallos y 20 aciertos** de 31 (salida: `test result: FAILED. 20 passed; 11 failed`). Los 2 aciertos nuevos eran "no definido no se encuentra" y "`default` no es un despliegue" (triviales con un `Deployer` vacío). **VERDE:** 32/32 (31 + el test del `Display`). Dos fallos posteriores eran mis tests: comillas en claves con `*` y el valor resuelto de `dispatcher` en `reference.conf` (`pekko.actor.default-blocking-io-dispatcher`).

**Sin ROJO diferencial:** el gemelo se escribió con la implementación ya hecha y coincidió en la primera comparación; la detección la demuestran los mutantes.

## Replay y mutantes

Replay completo (control de los mutantes sobre el árbol sin mutar): **2810/2810 aplicables + 0 NA:Q-CFG-30 + 1 no aplicable, divergen 0** (2773 + 38 = 2811).

| Mutante (`deployer-…`) | Resultado | Pasan |
|---|---|---|
| entry-ignores-the-default-section | DETECTED | 2772/2810 |
| default-wins-over-the-entry | DETECTED | 2785/2810 |
| router-is-never-named | DETECTED | 2805/2810 |
| from-code-is-a-named-router | DETECTED | 2785/2810 |
| dispatcher-ignores-the-default-section | DETECTED | 2772/2810 |
| mailbox-ignores-the-default-section | DETECTED | 2772/2810 |
| dispatcher-and-mailbox-are-swapped | DETECTED | 2794/2810 |
| entry-scope-is-local | DETECTED | 2782/2810 |
| configuration-errors-are-swallowed | DETECTED | 2806/2810 |
| rejected-paths-are-swallowed | DETECTED | 2806/2810 |
| entries-go-in-reverse-alphabetical-order | DETECTED | 2807/2810 |

11/11 DETECTED, 0 supervivientes, control CLEAN. Patrones: 196, 0 obsoletos. No se repitió la batería completa ni los 10 mutantes de #277 (los patrones de `deploy`/`lookup` no cambiaron). Un mutante candidato (desplegar `default`) no se añadió: `default` da un path de un solo elemento, que `deploy` descarta, así que es equivalente. El mutante de actualización perdida de #277 sigue matándose con su test unitario determinista; ninguno de los nuevos depende de hilos.

## Gates

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check` (advisories, bans, licenses y sources ok) y `python3.12 tools/package/check.py` (0 fallos): ok.
- Cobertura (`cargo llvm-cov --workspace --summary-only`): líneas 98,39 % (216 sin cubrir; 217 en la base), regiones 98,24 %; `actor/deployer.rs` 0 líneas sin cubrir (2 regiones); sin líneas nuevas sin cubrir.

## Etapas de Rust-Anything ejecutadas

CLI: `rust-anything app record` (38 casos contra `oracle/run.sh`) y el replay (`tools/parity/replay.py`, vía el control de `check_divergence.py`). No se ejecutaron `analyze`, `bridge`, `shadow` ni `gate`: el crate ya existe y el flujo del repositorio es oráculo Java más `app record/replay` (Q-X-12).

## Divergencias y pendientes

- Pekko lee también `pekko.actor.router.type-mapping` en el constructor (Deployer.scala:229-236); Rust no, de acuerdo con la decisión de no implementar type-mapping: pendiente de P37.
- Sin mutante propio: la lectura estricta de todas las entradas antes de desplegar (la matan DS-24 y un test unitario) y el descarte de entradas que no son objeto (DS-14 y un test unitario). Ambas están cubiertas por casos y tests, pero no por el harness de mutantes.
- El oráculo solo usa claves entre comillas; los tests unitarios usan claves sin comillas (el revisor comprobó que coinciden). `Settings::new` rechaza algunas configuraciones inválidas antes de llegar a `from_settings`: no hay casos de eso aquí.
- Q-CFG-36 (propuesta): orden de claves en colisión; `CO-04` y `CO-05`.
- Routers (P37): un router desconocido o con valores inválidos se acepta; Pekko lanza. A5 sin cubrir. El `RouterConfig::Opaque` en la API pública sigue pendiente de P37 (riesgo de #276).
- `Deployer` público frente a `private[pekko]` en Pekko (F4 de #277), Q-UTIL-21 (propuesta) y Q-CFG-35 (propuesta) siguen abiertas.
- AC-CFG-012.1 sigue `partial`: falta la integración observable con `ActorSystem`/provider (AC-SYS-016) y los routers reales.
