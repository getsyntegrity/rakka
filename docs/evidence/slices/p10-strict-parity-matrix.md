# P10 — matriz de divergencias y plan de paridad estricta (2026-11-20)

Instrucción del Owner (2026-11-20): Rakka reproduce el comportamiento observable de Apache Pekko 2.0.0; no se autorizan divergencias por conveniencia, simplicidad, rendimiento o idioma de Rust. Registrada como **Q-X-20 (`proposed`)**: `docs/owner-batch-02.md` no se sustituye ni se edita `owner-decisions.tsv` (solo lo edita el Owner). Q-CFG-35 (a), Q-CFG-36 (a) y Q-UTIL-21 (a) se conservan como historia, con la nota de sustitución propuesta.

## Hechos que corrigen la premisa del encargo

- **Q-UTIL-21 no está aceptada**: figura `proposed` en `register.tsv` y no aparece en `owner-decisions.tsv`. La divergencia actual (gana el primer `prefix*` insertado) viene de la lectura de Q-UTIL-8 (c) que el propio autor de #277 dejó marcada como ambigua (`p10-deployer.md`), no de una decisión explícita.
- Q-CFG-35 y Q-CFG-36 tampoco están aceptadas; solo Q-CFG-36 (a) tuvo una autorización "temporal" del Owner en la sesión de P10 3/3.
- Por tanto ninguna decisión *accepted* se contradice; Q-X-20 solo cambia la dirección de tres propuestas.

## Línea base (reproducible)

- Replay completo en `c0f25c5` (HEAD de integración con #278): **2810/2810 aplicables, 0 divergen, 1 no aplicable** (`python3.12 tools/parity/replay.py`, oráculo JDK 17).
- Q-UTIL-21, `docs/evidence/slices/p10-deployer-q-util-21-probe.java` contra los jars de 2.0.0: 10 conjuntos de `prefix*` solapados; el ganador de Pekko es el primer coincidente en el orden de iteración de un `immutable.HashMap` en **10/10**, independiente del orden de inserción; el prefijo más largo acierta 3/10; el primero insertado acierta en una de las dos órdenes de cada par.
- Q-CFG-36: `CO-04` y `CO-05` están en `PENDING` de `deployer_settings_cases.rs`: el gemelo Rust no los produce, y 53 de 99 conjuntos medidos difieren (`p10-from-settings-collisions-probe.java`; dependencia de los valores en `…-value-dependence-probe.java`).

## Matriz

| # | Diferencia | Pekko 2.0.0 | Rust hoy | Evidencia | Decisión | Clase | Corrección | Depende de | PR |
|---|---|---|---|---|---|---|---|---|---|
| A1 | Ganador entre claves de `deployment` que dan el mismo path (`/svc`, `/svc/`, `/svc//`; CO-04, CO-05) | Orden de `config.root.asScala`: `HashSet` de `SimpleImmutableEntry` (hash = hash(clave) XOR hash(valor), tamaño de tabla, orden de inserción por cubeta); gana la última entrada | `BTreeMap`: gana la última clave alfabética | 53/99 difieren; el ganador depende de los valores | Q-CFG-36 (a) `proposed` → Q-X-20 `proposed` | **FIX** bajo Q-X-20 | Emular el orden del `HashSet` y el `hashCode` de cada `ConfigValue`; fuzz diferencial | Q-X-20; cleanroom del orden de `HashMap`/`HashSet` de Java (Q-X-18: sin leer OpenJDK) | PR-D |
| A2 | Error informado con varias entradas inválidas | El de la primera entrada inválida en el mismo orden (`String: 4:` frente a `String: 2:`) | El del orden alfabético | revisor independiente de #278 | Q-CFG-36 | **FIX** con A1 | Mismo orden de A1 | A1 | PR-D |
| B1 | Precedencia de `prefix*` solapados (`p*`/`pq*`) | Primer coincidente al iterar `immutable.HashMap` (CHAMP) de Scala 2.13 | Primer insertado | probe 10/10 | Q-UTIL-21 (a) `proposed` → Q-X-20 | **FIX** bajo Q-X-20 | `String.hashCode` UTF-16, `improve` y recorrido CHAMP (cargas útiles, luego subnodos, colisiones en orden de inserción); fuzz diferencial | Q-X-20 | PR-B |
| B2 | Impacto en `Deployer` | Idéntico a B1 (usa `WildcardIndex`) | idem | `p10-deployer.md` | idem | se cierra con B1 | casos de `Deployer` con `prefix*` solapados | B1 | PR-B |
| C1 | `Deploy.toString` | `Deploy(path, config, router, scope, dispatcher, mailbox, tags)`; claves ordenadas, tags en orden de hash del `Set` de Scala si hay más de 4 | No hay `Display` (solo `Debug`) | JVM JDK 17 = 27 | Q-CFG-35 (a) `proposed` → Q-X-20 | **FIX** | `Display` exacto; orden de tags = orden del `Set` de Scala (comparte B1); medir el render de valores sin resolver | B1 (tags), P37 (texto de routers) | PR-C |
| C2 | `Config::render_concise` | Render de `ConfigObject.toString` | Existe; cobertura parcial | `Config::render_concise` y sus casos | Q-CFG-35 | **FIX** donde haya diferencia medida | Comparar con el oráculo (valores sin resolver, números, comillas) antes de afirmar equivalencia | — | PR-C |
| C3 | Texto del router y de `Scope` | Clase y parámetros del router real | `RouterConfig::Opaque` | — | Q-CFG-35 | **DEFERRED** | — | P37 | — |
| D1 | Routers desconocidos o inválidos, `type-mapping`, `nr-of-instances` inválido | `Deployer` lee `pekko.actor.router.type-mapping` y los routers lanzan al construirse | Se acepta cualquier router por nombre | `p10-from-settings.md` | — | **DEFERRED** | No se implementa P37 | P37 | — |
| E1 | `Deployer` público frente a `private[pekko]` | Visibilidad restringida | `pub` | F4 de #277 | — | **OWNER DECISION** (API) | Propuesta a redactar: `pub(crate)` más reexportación explícita | — | propuesta |
| E2 | `RouterConfig::Opaque` público | No existe | variante pública | riesgo de #276 | — | **OWNER DECISION** (API) | Propuesta: `#[non_exhaustive]` + constructor | P37 | propuesta |

## Estado de las filas (sin cambios hasta cerrar cada PR)

- AC-CFG-013 (`partial/oracle`): C1, C2, C3. AC-CFG-012.2 (`partial/oracle`): B1, B2. AC-CFG-012.1 (`partial/oracle`): A1, A2, D1.
- Ninguna fila pasa a `done` por pasar el replay; D1 y C3 mantienen la fila `partial` por una dependencia identificada, no por aceptación.

## Orden y alcance de los PR (cada uno < 400 líneas autoradas)

1. **PR-A (este)**: Q-X-20, matriz, regeneración del paquete. Solo documentación (sin mutantes).
2. **PR-B**: `WildcardIndex` con el orden CHAMP de Scala (B1/B2); mutantes nuevos; fuzz diferencial; sin tocar `Deployer`.
3. **PR-C**: `Display` de `Deploy` y revisión de `render_concise` (C1/C2).
4. **PR-D**: orden de `HashSet` de Java y `hashCode` de `ConfigValue` (A1/A2); CO-04 y CO-05 pasan de `PENDING` a casos grabados reales. Es el más grande y necesita una especificación de sala limpia previa (Q-X-18).
5. Propuestas de E1/E2 sin código.

No se hace merge de ninguno sin autorización del Owner.
