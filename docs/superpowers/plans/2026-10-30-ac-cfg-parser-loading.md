# Unidad AC-CFG, spec 1: parser y carga de configuración — plan

**Estado: preparada, implementación bloqueada por dos decisiones del Owner (Q-CFG-1, Q-CFG-2).** Este plan no elige parser ni mecanismo de registro: presenta la evidencia y una recomendación. Lo que no depende de ellas (oráculo, golden, trazabilidad) ya está hecho y es lo único que entra en este PR.

**Goal:** que `rakka-actor` pueda leer HOCON, combinar capas (`withFallback`), resolver sustituciones y leer valores tipados con el mismo comportamiento observable que Typesafe Config 1.4.9 en lo que Pekko 1.7.1 usa, y cargar el `reference.conf` oficial sin modificarlo.

**Fuente del contrato** (rutas relativas a `upstream/pekko/`): `actor/src/main/resources/reference.conf` (1443 líneas), `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:280-281` (`ConfigFactory.load`), `:853` (`withFallback(defaultReference)`), `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:172-177` (`ConfigOps`), `actor/src/main/scala/org/apache/pekko/actor/DynamicAccess.scala:33`, `ReflectiveDynamicAccess.scala:35`.

**Oráculo:** `tools/oracle/config-hocon.jsh` contra `config-1.4.9.jar` (la biblioteca HOCON de Pekko, SHA-1 verificado por `tools/oracle/fetch-jars.sh`). Golden: `crates/rakka-actor/tests/oracle/config-hocon.golden`, 161 casos, determinista (dos corridas idénticas). Typesafe Config no es código de Pekko: las afirmaciones sobre HOCON salen de este golden; las de Pekko, de sus specs (`path:line`).

## Alcance de esta spec

| Fila | Qué | Decisión |
|---|---|---|
| `AC-CFG-001` | carga: `reference.conf` como fallback, config explícita tal cual | **entra** |
| `AC-CFG-017` | formato HOCON usado por `reference.conf`: sustituciones, `+=`, duraciones, tamaños, `on/off`, `infinite`, `unlimited` | **entra** |
| `AC-CFG-016` | `DynamicAccess`: registro nombre → fábrica | **entra el contrato del registro** (tarea 5); la forma depende de Q-CFG-2 |
| `AC-UTIL-002` (parte `ConfigOps`) | `getMillisDuration`, `getNanosDuration` | **entra** (el plan de AC-UTIL la dejó para AC-CFG) |
| `AC-UTIL-002` (`renderWithRedactions`) | redacción en `toString` | fuera, spec 2 (`Settings.toString`, `ActorSystemSpec.scala:395-410`) |
| `AC-CFG-002..015` | `Settings`, claves por sección, `Deploy`, `Setup` | fuera, spec 2 y siguientes |
| `AC-SER-011` | `WireConfig` (parseo sin includes) | fuera, unidad SER |

## Evaluación de opciones HOCON

Criterio: no se elige un parser por poder leer `reference.conf`; se elige por cumplir los contratos de la tabla siguiente. Medido (no leído de un README) con un proyecto descartable contra el golden.

| Contrato (caso del golden) | `hocon-parser` 1.14.2 (Apache-2.0) | `hocon-rs` 0.2.0 (MIT) | `hocon` 0.9.0 |
|---|---|---|---|
| Parsea `reference.conf` completo con `include "version"` | **sí** (con `base_dir`) | sí | no medido |
| `withFallback` por capas (FB-01..07) | **sí**, sólo si se parsea sin resolver (`with_resolve_substitutions(false)`); con el `parse` por defecto la sustitución se congela y el override de la aplicación no se ve (FB-05 da 1, Typesafe da 2) | **sin API de fallback** (no hay `with_fallback`/`merge` público) | no medido |
| `${?x} [d]` y `+=` entre capas (SUB-02/03, REF-02) | `${?x} [d]` **sí**. REF-02 (`+=` de la aplicación sobre el fallback con `${?x} [...]`) **no**: da `[my.Ext]`, Typesafe da `[SerializationExtension$, my.Ext]` | no medido | no medido |
| Duraciones (DUR-*) | sí salvo **negativas** (`-1s` falla; Typesafe da -1000 ms) | devuelve texto, `as_duration` propio | no medido |
| Tamaños (SIZE-*) | sí, mismas excepciones que Typesafe en `KB`/negativos | no medido | no medido |
| Booleanos (BOOL-*) | `"ON"` se acepta (Typesafe: `WrongType`) | no medido | no medido |
| `include classpath()` (INC-04) | **error de parseo** (Typesafe lo ignora si falta) | no medido | no medido |
| Mensajes de error (TYPE-*, SUB-*) | distintos de Typesafe | no medido | no medido |
| Mantenimiento | 1.14.2, 2026-10 | 0.2.0, 2026-08 | último release 2022-04 |

`hocon` 0.9.0 se descarta por falta de mantenimiento; `hocon-rs` por no ofrecer `withFallback`, que es el contrato central. `hocon-parser` es el único candidato viable pero tiene **una divergencia en el núcleo del algoritmo** (REF-02: la semántica de `+=`/autorreferencia entre capas), no sólo en bordes.

## Preguntas abiertas para el Owner

**Q-CFG-1 (AC-CFG-001, AC-CFG-017): ¿qué implementación de HOCON?**

- (A) **Implementación propia del subconjunto** en un crate del workspace sin dependencias (`crates/rakka-hocon`, o módulo `config` de `rakka-actor`), con el golden como suite de aceptación.
- (B) **`hocon-parser` detrás de una fachada**, parseando siempre sin resolver, y con una `Q:` explícita por cada divergencia: REF-02 (`library-extensions` con `+=`), duraciones negativas, `"ON"`, `include classpath()`, mensajes de error.
- (C) (B) más un parche o contribución upstream a `hocon-parser` para REF-02 y negativos, a la espera de que se publique.

**Recomendación: (A).** (1) La regla del proyecto es que ninguna diferencia semántica se acepta sin decisión; (B) abre por lo menos cinco `Q:` en una pieza que sostiene todo `Settings`, y REF-02 afecta a `pekko.library-extensions`, que el propio `reference.conf:78` usa. (2) La divergencia está en el algoritmo de merge/resolución, no detrás de la fachada: no se puede corregir sin tocar el crate. (3) El subconjunto que usa Pekko es acotado (el `reference.conf` tiene 2 sustituciones, 1 `+=`, 1 include) y el golden de 161 casos fija el comportamiento esperado. Costo: es la opción más cara; el riesgo es dejar fuera una parte de HOCON que no cubre el golden, que se mitiga ampliando el golden antes de implementar y no declarando paridad más allá de él. **Esto es una propuesta, no una decisión aceptada.** (C) es la alternativa si el Owner prefiere no mantener un parser; depende de un tercero.

**Q-CFG-2 (AC-CFG-016, AC-EXT-003): ¿cómo se reemplaza la reflexión por FQCN?** Propuesta ya presente en el inventario: registro explícito `nombre → fábrica` (`FactoryRegistry<T>`), con los nombres de `reference.conf` registrados de fábrica; un nombre no registrado devuelve un error de configuración que lleva el nombre. Recomendación concreta: un tipo `FactoryRegistry<Product>` por cada punto de extensión (mailbox, executor, logger, etc.) con `register(name, factory)` y `create(name, &Config) -> Result<Box<Product>, ConfigurationException>`; el mensaje del error contiene el nombre pedido. Preguntas dentro de la decisión: (i) el error es `ConfigurationException` o un tipo propio (Pekko lanza `ClassNotFoundException`, `DynamicAccessSpec.scala:50`); (ii) `classIsOnClasspath` pasa a `contains(name)` (`:72`); (iii) hay una sola jerarquía de registros o uno por tipo. Alternativa descartada: `inventory`/`linkme` como mecanismo principal (dependen del linker). **Propuesta, no decisión.**

Mientras Q-CFG-1 esté abierta no se escribe código de parser: cualquier implementación sería una reducción de contrato disfrazada de elección. Mientras Q-CFG-2 esté abierta no se fija la forma del registro; el contrato (nombre no registrado → error con el nombre) sí está definido por las aserciones de abajo.

## Trazabilidad contrato → aserción

Las aserciones de HOCON son casos del golden (`config-hocon.golden`, id entre corchetes); las de Pekko son specs. "RED" se registra cuando se implemente; hoy ningún contrato tiene implementación.

| # | Contrato (comportamiento) | Fuente Pekko | Aserción | Tarea |
|---|---|---|---|---|
| 1 | El valor del receptor gana sobre el del fallback, escalares y objetos se fusionan clave por clave | `ActorSystemSpec.scala:340-345` (`parseString(..).withFallback(testConf)`); `ConfigSpec.scala:198` | golden FB-01, FB-02, FB-03, FB-04, FB-07 | 3 |
| 2 | La sustitución se resuelve después de combinar las capas (el override de la aplicación se ve en lo derivado) | `ActorSystem.scala:853` | golden FB-05 (y FB-06 como contraejemplo) | 3 |
| 3 | `${?x}` ausente deja la clave sin definir; `${?x} [d]` concatena | `reference.conf:78` | golden SUB-01, SUB-02, SUB-03, REF-01 | 3 |
| 4 | `+=` de la aplicación sobre el valor por defecto con `${?x}` conserva el orden por defecto + aplicación | `reference.conf:78` | golden REF-02 | 3 |
| 5 | Sustitución requerida ausente, ciclo y getter sin resolver fallan | `ActorSystem.scala:396-400` (`checkValid`) | golden SUB-04, SUB-09, SUB-10, SUB-12 | 3 |
| 6 | Sustitución dentro de cadena entre comillas no se sustituye; autorreferencia; `+=` | `reference.conf:78,878` | golden SUB-05, SUB-06, SUB-07, SUB-11, SUB-14 | 2, 3 |
| 7 | Include inexistente se ignora; `required(...)` inexistente falla | `reference.conf:11` (`include "version"`) | golden INC-01..04 | 2 |
| 8 | Sintaxis de `reference.conf`: comentarios `#` y `//`, claves con guiones y con puntos entre comillas, triple comilla, claves duplicadas, listas con coma final | `reference.conf` completo | golden REF-06..10, LIST-01..06 | 2 |
| 9 | Duraciones: unidades `ns/us/ms/s/m/h/d` y nombres largos, número sin unidad = ms, fraccionarios, negativos, notación científica, truncado hacia cero, desborde | `ConfigSpec.scala:52,69,76,87,108,110,135,179`; `Helpers.scala:174,176` | golden DUR-MS/DUR-NS/DUR-ERR/DUR-SCI/DUR-ROUND/DUR-LARGE/REF-05 | 4 |
| 10 | `getMillisDuration` y `getNanosDuration` | `Helpers.scala:172-177`; `Dispatchers.scala:304-306` | golden DUR-MS y DUR-NS (misma tabla) | 4 |
| 11 | Tamaños: `KiB`, `K`, `kB`, `MB`, `g`, número sin unidad = bytes, negativos y fuera de rango fallan, `KB` falla | `reference.conf:966,1011` | golden SIZE-*, SIZE-ERR-*, REF-04 | 4 |
| 12 | Booleanos `on/off/yes/no/true/false` (con y sin comillas); `"ON"`, `1` y `"maybe"` fallan | `ConfigSpec.scala:53,59` | golden BOOL-* | 4 |
| 13 | Errores de getters tipados: clave ausente, tipo erróneo, `null`; coerción de número entre comillas y de entero a texto | `ConfigSpec.scala` (todas las lecturas tipadas) | golden TYPE-01..13 | 4 |
| 14 | Literales que Pekko interpreta por su cuenta (`infinite`, `unlimited`, `on/off` en `log-dead-letters`) llegan como texto/valor y el getter de duración/tamaño los rechaza | `reference.conf:64-67,863` | golden SPECIAL-01..08, REF-03 | 4 |
| 15 | `reference.conf` oficial se carga sin cambios y sirve de fallback; sin config explícita se usa `aplicación sobre reference`, con config explícita se usa tal cual | `ActorSystem.scala:280-281`, `:853`; `ConfigSpec.scala:30,190-201` | golden FB-08; test de carga del `reference.conf` embebido (valores de `ConfigSpec.scala:44-110`) | 5 |
| 16 | Fábrica por nombre: crea la instancia registrada | `DynamicAccessSpec.scala:40` | test de registro (Q-CFG-2) | 5 |
| 17 | Nombre no registrado → error que lleva el nombre | `DynamicAccessSpec.scala:50` | test de registro (Q-CFG-2); **fila `partial` hasta que el Owner decida el tipo de error** | 5 |
| 18 | Varias formas de construcción con reintento (`recoverWith`) | `DynamicAccessSpec.scala:59` | test de registro (Q-CFG-2) | 5 |
| 19 | `contains(name)` equivale a `classIsOnClasspath` | `DynamicAccessSpec.scala:72` | test de registro (Q-CFG-2) | 5 |

## Tareas (máximo cinco)

Cada comportamiento sigue RED → GREEN → REFACTOR (`AGENTS.md`); la cobertura mínima es 90 % de las líneas de producción nuevas, medida con `cargo llvm-cov` y registrada en el PR de implementación.

1. **Oráculo y golden de HOCON.** `tools/oracle/config-hocon.jsh`, `tests/oracle/config-hocon.golden`. **Hecha** (no tiene código de producción; evidencia: el golden se generó dos veces con salida idéntica). *No depende de decisión.*
2. **Modelo de valores y parser** (comentarios, claves con puntos, comillas triples, listas, objetos, includes). Contratos 6-8. *Bloqueada por Q-CFG-1.*
3. **Combinación y resolución** (`withFallback`, sustituciones tardías, `${?x}`, `+=`, autorreferencia, ciclos). Contratos 1-6. *Bloqueada por Q-CFG-1.*
4. **Lectores tipados** (duraciones, tamaños, booleanos, errores, `getMillisDuration`/`getNanosDuration`). Contratos 9-14. *Bloqueada por Q-CFG-1.*
5. **Carga y registro de fábricas.** `reference.conf` embebido con `include_str!` sin cambios, regla de carga, `FactoryRegistry`. Contratos 15-19. *Carga bloqueada por Q-CFG-1; registro por Q-CFG-2.*

## Diferencias pendientes (no se declara paridad)

- Los mensajes de error de Typesafe (`TYPE-*`, `SUB-*`, `DUR-ERR`, `SIZE-ERR`) son parte del golden; reproducirlos textualmente o no es una decisión a incluir en Q-CFG-1 (los specs de Pekko no los afirman, las afirmaciones están en el golden).
- No hay oráculo contra Pekko compilado desde `upstream/pekko`: este golden es de la biblioteca Typesafe Config, no de Pekko. La carga (`ConfigFactory.load` con classpath, propiedades del sistema) no tiene equivalente en Rust y se define en la tarea 5; las propiedades del sistema (`-Dpekko...`) y los recursos de classpath **no** se modelan hasta que el Owner lo pida.
- El golden se generó con Temurin 11.0.32 (`AGENTS.md` pide Java 21 para el oráculo); el comportamiento de `config-1.4.9.jar` no depende de la versión, pero no se verificó en 21.
