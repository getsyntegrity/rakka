# Propuesta de partición de P05 (UTIL-VARIOS)

**Estado (2026-11-01): reasignacion documental APLICADA; las alternativas semanticas siguen SIN aprobar.** El Owner autorizo reasignar filas y actualizar los documentos derivados; no aprobo alternativas semanticas, exclusiones ni respuestas a decisiones abiertas. Todo lo que sigue a esta seccion es el analisis de partida; donde difiere, manda esta seccion.

## Delta aplicado respecto de la propuesta original

- **Unidades finales:** P05 queda con AC-UTIL-016.6 (Glob). Nuevas: P65 HashCode (016.15), P66 PekkoVersion (005.1), P67 Version (005.2), P68 PrettyDuration (016.5), P69 UUID (016.14), P70 Reloj (016.16), P71 Registro de fabricas (016.7, 016.13), P72 Manifest (016.9), P73 JFR (016.10, 016.11), P74 Tipos de deserializacion (016.12), P75 Origen de closures (016.8), P76 Equivalencias (016.1, 016.2, 016.3, 016.18, 019), P77 Fronteras asincronas (018.1, 018.2, 018.3). Reasignadas: 004.3 -> P02; 018.4 y 018.5 -> P16; 016.17 -> P26.
- **Cambios respecto de la propuesta:** Glob y HashCode separados; UUID y WallClock separados (no comparten resultado funcional); 005.1 separada de 005.2 (segun su contrato `PekkoVersion.require` no depende de `util.Version`; cada una tiene una decision distinta: Q-CFG-5 y Q-UTIL-6). P76 y P77 se marcan como resolucion documental.
- **Dependencias de fila corregidas por contrato:** (1) Reflect consume BoxedType (`Reflect.scala:111`): `016.7 -> 016.13` y se retira `016.13 -> 016.7`. (2) `016.12` no depende de AC-SER-004 ni de AC-CFG-016: la consume AC-SER-013 (`Serializer.scala:358`); se retira esa arista y se anade `AC-SER-013 -> 016.12` (P40 declara P74). No se anadio `AC-CFG-016 -> 016.7` porque el contrato de AC-CFG-016 no cita Reflect.
- **Decisiones asociadas por contrato:** Q-UTIL-11 a 15 filas de P05 cuyo contrato la cita (la abreviatura `016.*` de la ficha no se expandia), Q-UTIL-8 a 016.15, Q-CFG-8 a 016.7, Q-CFG-11 a 016.12 (en `classic-foundation.decisions.tsv`); Q-X-2 en 016.12 y Q-X-4 en 018.4/018.5 (columna `notes`). Estado de ninguna decision modificado.
- **Costura de 016.9 (P72):** el contrato exige AC-UTIL-005.2, AC-EXT-001 (registro de la extension con AC-EXT-001.2) y AC-CFG-007, por lo que P72 declara P08, P30 y P67 y se ubica en la Ola 3. La consume AC-SYS-020 (P25, `ActorSystem.scala:1071`): la costura P05->P30 desaparece y aparece P25->P72 (hacia adelante, `partial` hasta integrar). Se preserva la dependencia funcional; `unit_plan.py --check` verifica ahora que toda dependencia de fila hacia atras queda cubierta por el grafo declarado de unidades (P25 declara P67, P46 declara P77, P06 declara P02, P40 declara P74). Se retiro la dependencia declarada de P05 en P07 y P14 porque ninguna de sus filas depende de las 26 filas originales; esa verificacion semantica queda pendiente.
- **P16:** pasa a 19 filas, heterogeneas; se registra en su ficha que **requiere subdivision** (no aplicada, no lista para implementar).
- **Mecanico:** el anexo de costuras del plan ahora se genera (148 aristas hacia adelante, antes 153); `unit_plan.py --check` comprueba ademas que la lista `Filas` de cada ficha coincide con `docs/unit-rows.tsv`.

## Dudas y contradicciones registradas (no resueltas)

1. **AC-UTIL-018.2:** la cabecera del contrato dice `impl=partial`/`validation=oracle`, el TSV `pending`/`pending`. La evidencia existente (`timeout.rs:54`, `util-core.golden` TO-NEG-04/05) cubre solo `JDuration.asScala` -> Timeout, no Optional*, FunctionConverters ni `Duration.asJava`. Estado no modificado.
2. **parasitic:** la regla de `reportFailure` de `ExecutionContext.parasitic` (biblioteca de Scala, fuera de `upstream/pekko`) esta sin medir. La direccion de dependencia entre 018.4 y AC-DSP-017.1 no la fija ningun contrato (`Future.scala:104` muestra que `ExecutionContexts.parasitic` usa `SameThreadExecutionContext()`); no se anadio arista.
3. **016.10 JavaVersion** no tiene comportamiento ejecutable propio en Rust; su agrupacion con 016.11 (P73) es discutible.
4. **016.9:** la funcion pura (A1-A2) no necesita la extension y podria validarse antes, pero exige dividir la fila; no se hizo.
5. **AC-UTIL-016.16 (WallClock):** existe trabajo de otros agentes en ramas `ao/ac-util-016.16-*`; no se evaluo ni se cambio su estado en el inventario.
6. `cargo semver-checks` (019) no esta verificado como herramienta del repositorio.
7. La atomicidad de las unidades nuevas y de P16 no esta acreditada.

## Decisiones que siguen abiertas

Ninguna decision cambio de estado. Siguen sin respuesta: Q-UTIL-6 (bloqueante, P67/P72), Q-CFG-5 (P66, P72), Q-CFG-7 (P73), Q-UTIL-8 (P05, P65), Q-UTIL-9 (P68), Q-UTIL-14 (P65), Q-UTIL-16 (P75), Q-UTIL-4/Q-X-9 (004.3 en P02), Q-X-2/Q-CFG-8 (P71), Q-X-2/Q-CFG-11 (P74), Q-X-4 (018.1, 018.4, 018.5), Q-UTIL-11 (P69, P71, P72, P73, P74, P75, P76, P77, 018.4/018.5 en P16). Q-UTIL-1 sigue respondida (rechazada, alternativa vigente).

---

# Analisis de partida (previo a la aplicacion)

Base: HEAD `6c4607d` más los cambios mecánicos sin commit. Alcance: las 26 filas que `docs/unit-rows.tsv` asigna a P05. Se leyó el bloque contractual completo de cada fila (`classic-foundation.md`), el de `AC-UTIL-018.4`/`AC-DSP-017.1`/`AC-DSP-016.2`, el estado en `register.tsv` y las fichas generadas en `docs/unit-decisions.md`. P55 no se revisó.

## 1. Resultado de la lectura, fila por fila

Convención: «Dep. real» es lo que dice el contrato (no solo la columna `depends_on`). «Decisión» indica estado en el registro: **B** bloqueante, **P** propuesta, **R** rechazada con alternativa vigente. «Cierre / inicio» se toma del texto de la ficha de la decisión cuando existe; si la ficha no lo dice, se indica «ficha no lo dice».

| Fila | Comportamiento observable | Aserciones y validación | Dep. real | Decisiones | Destino propuesto |
|---|---|---|---|---|---|
| 016.6 Glob | `?` un carácter, `*` cualquier secuencia, lineal con retroceso; `**`≡`*` | GlobSpec A1-A5 (exhaustivo 0-4 chars vs `makePattern`; rendimiento <3 s / <5 s) | consumidor P06 (AC-UTIL-023) | Q-UTIL-8 P: «bloquea nada si (a)/(c)» | A |
| 016.15 HashCode | `37*seed+contrib`, SEED=23; Long/Float/Double/arrays; `null` lanza IAE | sin spec; valores medidos con el jar; golden (extremos `Long.MIN`, `NaN`, `-0.0`) | consumidores 005.2 y P03 (017.1) | Q-UTIL-14 P: «bloquea AC-UTIL-017.1 (impl exacta)». Q-UTIL-8 (strings) según el contrato, **no figura** en `contract-map` para esta fila | A |
| 005.1 PekkoVersion.require | compara con regex propia `M/RC`; versiones no calzadas pasan; mensaje `UnsupportedPekkoVersion` | PekkoVersionSpec A1-A6; A7 (mensaje) derivada del código | ninguna fila; dato `Version.current` | Q-CFG-5 P: «bloquea AC-UTIL-005.1; puede avanzar el resto» | B |
| 005.2 util.Version | parseo a 4 números + resto, `Comparable`, dynver | VersionSpec A1-A7; A8 medido con el jar (defecto de asimetría) | 016.15 (HashCode) | **Q-UTIL-6 B**: bloquea A5/A8 y la elección de versión más alta en 016.9; puede avanzar A1-A4, A6-A7 | B |
| 016.5 PrettyDuration | unidad mayor entera, `%.4g` de Java, `∞`/`undefined`, `includeNanos` | PrettyDurationSpec A1-A4; A5 medido con el jar (`1.000e+04 d`) | consumidores REF-014 y logs | Q-UTIL-9 P: «bloquea nada; avanza con (a) y el golden por crear» | C |
| 016.14 UUIDComparator | orden por versión; v1 por `timestamp` sin signo; resto sin signo | sin spec; jar con valores medidos; `uuid-comparator.jsh` por crear | ninguna interna; único consumidor fuera de los módulos (persistence-query) | Q-UTIL-11 P (qué se devuelve: `Ordering` vs entero) | D |
| 016.16 WallClock | `currentTimeMillis` estrictamente creciente (`last+1`) | sin spec; A1 derivada del código | consumidores no identificados | ninguna | D |
| 016.7 Reflect | `findConstructor` (una sola coincidencia), `instantiate`, `findMarker`, class loader | ReflectSpec A1-A4 portable sobre registro de fábricas; A5 (mensajes, `findMarker`, loader) no portable | BoxedType; **consumidores** DynamicAccess (CFG-016) y Props | Q-X-2 P (ficha: «—»), Q-CFG-8 P, Q-UTIL-11 P | E |
| 016.13 BoxedType | primitivo → envoltorio | sin spec; medido con el jar | lo usa Reflect (ver duda 3) | Q-UTIL-11 P | E |
| 016.9 ManifestInfo | extensión que lee manifiestos; `checkSameVersion` con mensaje exacto | ManifestInfoSpec A1-A2 portables (función pura); A3 no afirmada | 005.2, EXT-001, CFG-007, SYS-020 (consumidor) | **Q-UTIL-6 B** (versión más alta), Q-CFG-5 P, Q-UTIL-11 P | F |
| 016.10 JavaVersion | major de `java.specification.version` | derivada del código | consumidor DSP-014 y 016.11 | Q-UTIL-11 P | G |
| 016.11 FlightRecorderLoader | carga JFR si JVM≥11 y clave `enabled`; si no, `fallback` | sin spec; caracterización propia | 016.10, CFG-007, CFG-016, CFG-002 | Q-CFG-7 P: «bloquea AC-CFG-007 y AC-UTIL-016.11 (impl); puede avanzar aceptar y validar las claves» | G |
| 016.12 ClassLoaderObjectInputStream | resuelve tipos primero contra el loader del sistema | sin spec; se cubre con AC-SER-003/004 | SER-004 (P40), CFG-016 (el contrato la cita; el TSV no) | Q-UTIL-11, Q-X-2, Q-CFG-11 P | H |
| 016.8 LineNumbers | `SourceFileLines` / `prettyName`; texto visible en `toString` de Typed | LineNumberSpec A1-A2 **no portables** (compilador de Scala); caracterización propia del formato Rust | consumidores Typed (BehaviorImpl, InterceptorImpl) | Q-UTIL-16 P: «bloquea AC-UTIL-016.8 (impl) y los tests de `toString` de Typed» | I |
| 016.1 OptionVal | `Option` con `null`=None; `get` lanza con texto | sin test; sin código propio en Rust | consumidores P03 (017.*) | Q-UTIL-11 P | J1 |
| 016.2 ConstantFun | identidad/constantes; misma instancia por `T` | A1/A2 no tienen sentido en Rust | japi | Q-UTIL-11 P | J1 |
| 016.3 Collections | `EmptyImmutableSeq`, `PartialImmutableValuesIterable`; mensajes | sin test; mensaje `next on empty iterator` lo observa ByteIterator (P04) | consumidor P04 | Q-UTIL-11 P | J1 |
| 016.18 Unused | marcador del compilador | clippy del repo | ninguna | Q-UTIL-11 P | J1 |
| 019 Anotaciones | frontera pública/interna, `@Target` | sin test; `cargo semver-checks` (herramienta no verificada en el repo) | todos los módulos | Q-UTIL-11 P | J1 |
| 018.1 FutureConverters | frontera `CompletionStage`↔`Future` | A1 es prueba de compilación | consumidores JAPI (P46) | Q-UTIL-11, Q-X-4 P | J2 |
| 018.2 Option/Function/JavaDuration converters | `Optional`↔`Option`; `JDuration`↔`Duration` con overflow | oráculo `util-core` TO-NEG-04/05 (en `util-core.golden` líneas 35-36); `TryFrom<Duration> for Timeout` en `timeout.rs:54` | AC-UTIL-001 (P02) | Q-UTIL-1 **R** (alternativa vigente), Q-UTIL-11 P | J2 |
| 018.3 ccompat / compat.Future | shims; `fold/reduce/find` delegan en `Future` | A1 compilación; A2 no afirmada | `compat.Future` solo se llama desde `Future.scala:184,208,219` (AC-DSP-017.2) | Q-UTIL-11 P; **Q-DSP-3 P** propone excluir 017.2 | J2 |
| 018.4 SameThreadExecutionContext | ejecuta inline en el hilo que envía | SameThreadExecutionContextSpec A2 (mismo hilo); A1 no portable | AC-DSP-* | Q-X-4, Q-UTIL-11 | **P16** (con DSP-017.1) |
| 018.5 ScalaBatchable | `Batchable.isBatchable` / `true` / `false` | sin test directo | el consumidor es `BatchingExecutor` (AC-DSP-016.2, P16) | Q-UTIL-11, Q-X-4 | **P16** (con DSP-016.2) |
| 004.3 OnlyCauseStackTrace | traza = la de la causa / vacía | A1 derivada del código | 004.1 (P02) | Q-UTIL-4 P: «bloquea AC-UTIL-004.2/.3 (cierre); avanza 004.1», Q-X-9 (grupo) | **P02** |
| 016.17 ErrorMessages | constante `"configuration requested remote deployment"` | A1 valor literal | consumidor AC-SYS-016 (P26) | ninguna | **P26** |

## 2. Solape AC-UTIL-018.4 / AC-DSP-017.1

Los dos contratos describen **la misma pieza**: el contexto de ejecución que corre la tarea en el hilo que la envía (`parasitic`). Los dos citan `SameThreadExecutionContextSpec.scala:40-66` (A2 de 018.4 = A3 de DSP-017.1). No son comportamientos distintos.

Responsabilidades propuestas (sin eliminar comportamiento, una sola implementación):

- **AC-UTIL-018.4** responde del *ejecutor inline*: ejecutar en el hilo actual, encolar si ya hay un ciclo en curso (trampolín), enviar la excepción del `Runnable` a `reportFailure`. Aserción: mismo hilo en una cadena `map`.
- **AC-DSP-017.1** responde de la *fachada*: `fromExecutor`/`fromExecutorService`/`global`, el reportero por defecto, `sameThreadExecutionContext` obsoleto (batching sobre `parasitic`), y `parasitic`+`blocking` (A4). Su dependencia pasa a incluir 018.4.
- La prueba A2/A3 se escribe **una vez** y se traza a ambas filas.

Razón para llevar 018.4 y 018.5 a P16 y no dejarlas en la ola 0: un ejecutor inline necesita el trait `Executor`, y `Batchable` solo tiene consumidor en `BatchingExecutor` (AC-DSP-016.2). Ambos pertenecen a P16 (que depende de P08 y P15). Implementarlas antes inventaría la abstracción de ejecutor fuera de su unidad.

Esto no se verificó: la regla exacta de `reportFailure` de `ExecutionContext.parasitic` de la biblioteca estándar de Scala (no está en `upstream/pekko`); hay que medirla con el jar (ver duda 6). Nota adicional: P16 ya tiene 17 filas y no se revisó su atomicidad; mover dos filas ahí no la acredita.

## 3. Las cuatro filas «posteriores a CFG/EXT/SER»: resultado de contrastarlas con los contratos

La propuesta inicial decía cuatro (016.9, .10, .11, .12). Contra los contratos son **tres** con dependencias reales hacia adelante:

- **016.9 ManifestInfo**: 005.2 (anterior), EXT-001.1 (P30), CFG-007 (P08). Correcto. Ojo: es consumida por AC-SYS-020 (P25); mover 016.9 después de P30 convierte la costura «P05→P30» en «P25→nueva unidad F», porque P30 depende de P26 y P26 de P25. Cambia el sentido de la costura documentada, no su número; debe pasar `cycles.py` antes de aplicarse.
- **016.11 FlightRecorderLoader**: 016.10, CFG-007 (P08), CFG-016 (P09), CFG-002. Correcto.
- **016.12 ClassLoaderObjectInputStream**: SER-004 (P40). Correcto; el contrato cita además CFG-016, que el TSV no lista.
- **016.10 JavaVersion**: **no** tiene dependencias hacia adelante (`depends_on` = `-`; es `n/a` en Rust). Solo lo consume 016.11 y DSP-014. Se agrupa con 016.11 porque su único resultado verificable es servir de condición a 016.11; no se mueve «por su nombre».

## 4. Partición propuesta (26 filas, sin pérdidas ni duplicados)

IDs provisionales; no sustituyen a P05 hasta aplicarse.

| Unidad | Filas | Resultado verificable | Exclusiones concretas | Depende de | Tipo |
|---|---|---|---|---|---|
| **A TEXTO-HASH** | 016.6, 016.15 | `glob_matches` y `hash_*` equivalentes a Pekko sobre las entradas aseveradas | `makePattern` bit a bit (lo cubre AC-UTIL-023); hash de tipos de otros módulos | P02 | implementación |
| **B VERSION** | 005.1, 005.2 | `Version` (parseo/igualdad/comparación numérica/dynver) y `PekkoVersion::require` con sus errores | `Version.current` (valor, Q-CFG-5); ManifestInfo | A (por HashCode), P02 | implementación |
| **C FORMATO-DURACION** | 016.5 | `pretty` idéntico a `PrettyDuration` incluida la notación `%g` | cualquier otro formateo de logs | P02 | implementación |
| **D API-AUTONOMAS** | 016.14, 016.16 | orden de UUID sin signo y por timestamp en v1; reloj estrictamente creciente | `persistence-query` | P02 | implementación |
| **E REGISTRO-FABRICAS** | 016.7, 016.13 | resolución por nombre+firma `TypeId` con los errores «ningún constructor»/«varios» | `findClassLoader`, `getCallerClass`, `findMarker` (sin equivalente; exclusión solo con Owner) | P02 | implementación |
| **F MANIFEST** | 016.9 | `check_same_version` con mensaje exacto + registro de versiones de crates | lectura de manifiestos | B, P08, P30 (costura con P25) | implementación |
| **G JFR** | 016.10, 016.11 | clave booleana validada, `fallback` noop, aviso si falla | backend JFR real | P08, P09 | implementación |
| **H TIPOS-DESERIALIZACION** | 016.12 | resolución de tipo contra el registro del sistema antes del global | Java serialization | P40 | implementación |
| **I ORIGEN-CLOSURES** | 016.8 | texto de origen `archivo:línea` / `type_name` | parser de class files; rangos `from-to` | ninguna interna (consumidores Typed) | implementación, con formato por decidir |
| **J1 EQUIVALENCIAS-INTERNAS** | 016.1, 016.2, 016.3, 016.18, 019 | tabla de equivalencias aceptada por el Owner: `Option<T>`, closures, iteradores, `_nombre`, visibilidad/`sealed`/semver | cualquier tipo propio nuevo | P02 | **disposición, no implementación** |
| **J2 FRONTERAS-ASINCRONAS** | 018.1, 018.2, 018.3 | `Future` único; `Option`; `TryFrom<Duration> for Timeout` (ya con oráculo); shims documentados | `compat.Future.fold/reduce/find` si Q-DSP-3 excluye 017.2 | P02 | **disposición con evidencia previa** |
| a **P02** | 004.3 | `Error::source()` encadenado, sin traza | — | 004.1 | reasignar |
| a **P16** | 018.4, 018.5 | ver §2 | — | P08, P15 | reasignar |
| a **P26** | 016.17 | constante literal | — | consumidor SYS-016 | reasignar |

Total: A2 + B2 + C1 + D2 + E2 + F1 + G2 + H1 + I1 + J1 5 + J2 3 + reasignadas 4 = **26**.

Orden: A → B → (C, D, E, I en cualquier orden, sin archivos compartidos conocidos) → J1, J2 → F, G, H cuando existan sus dependencias. Lo que consumen P03/P04/P06 (016.1, 016.15, 016.3, 016.6) queda en A y J1, ambas sin dependencias hacia adelante.

## 5. Decisiones: qué impide empezar y qué impide cerrar

Regla (la de `AGENTS.md`): una decisión no respondida de una fila impide **cerrarla** (queda `partial`). Lo que impide **empezar** solo se afirma cuando la ficha lo dice o el contrato lo exige; lo demás se marca «no derivable».

| Unidad | Impide empezar | Impide cerrar | Fundamento |
|---|---|---|---|
| A | nada | 016.15 (Q-UTIL-14: la ficha dice que bloquea la implementación exacta de 017.1), 016.6/016.15 si Q-UTIL-8 no se responde | fichas Q-UTIL-8, Q-UTIL-14 |
| B | solo la parte de 005.2 que usa el calificador (A5/A8) | 005.2 (Q-UTIL-6 B), 005.1 (Q-CFG-5) | ficha Q-UTIL-6: «puede avanzar parseo, igualdad, comparación numérica y dynver»; ficha Q-CFG-5 |
| C | nada | nada que la ficha bloquee (Q-UTIL-9 «bloquea nada») | ficha Q-UTIL-9 |
| D | no derivable | 016.14 hasta que Q-UTIL-11 decida `Ordering` vs entero | contrato 016.14 |
| E | no derivable (Q-X-2 con ficha «—»); los tests A1-A4 se escribirían contra una API propuesta | Q-X-2, Q-CFG-8, Q-UTIL-11 | contrato 016.7 |
| F | la parte que elige la versión más alta | Q-UTIL-6 B | ficha Q-UTIL-6 |
| G | aceptar/validar la clave puede avanzar; la implementación de 016.11 no | Q-CFG-7 | ficha Q-CFG-7 |
| H | no derivable | Q-UTIL-11, Q-X-2, Q-CFG-11 | contrato 016.12 |
| I | formato del texto sin definir | Q-UTIL-16 | ficha Q-UTIL-16: «bloquea AC-UTIL-016.8 (impl)» |
| J1 | nada (no hay qué empezar) | Q-UTIL-11 | contratos: «exclusión solo con aprobación» |
| J2 | nada | Q-UTIL-11, Q-X-4, Q-DSP-3 (018.3); Q-UTIL-1 está respondida | contratos 018.1-018.3 |

Ninguna decisión se marcó como aceptada.

## 6. Escenarios de aceptación por unidad (de las aserciones leídas)

- **A:** (1) GlobSpec A1-A3 literales/`?`/`*`; (2) A4 equivalencia exhaustiva con la regex de `makePattern` en 0-4 caracteres sobre `{a,b,*,?}`; (3) A5 sin retroceso explosivo y escalado; (4) `hash(23,5)=856`, `hash(23,true)=852`, `hash(23,5000000000L)=705033556`, `hash(23,"abc")=97205`, `hash(23,1.5)=1073218387`, `hash(23,int[]{1,2})=31526` y extremos con el jar; (5) `null` no se modela.
- **B:** (1) PekkoVersionSpec A1-A6; (2) VersionSpec A1-A4, A6-A7; (3) A5/A8 **solo después** de Q-UTIL-6: el jar mide `1.2.0` vs `1.2-M1` = -2 y el inverso = -1.
- **C:** (1) los 10 casos de PrettyDurationSpec; (2) `Inf`, `MinusInf`, `Undefined`; (3) `10000 days`→`1.000e+04 d`, `99999 days`→`1.000e+05 d`, `1 day`→`1.000 d`; (4) `precision<=0` falla.
- **D:** (1) UUID v4 con bit alto: `new UUID(0x4000…, 0x8000…01)` > `new UUID(0x4000…, 1)`; (2) diferencia de versiones; (3) v1 por timestamp con extremos; (4) reloj: dos lecturas consecutivas nunca iguales ni decrecientes, también con hilos.
- **E:** ReflectSpec A1-A4 (1 y 2 parámetros, `null`, varios constructores, `null` ambiguo → error de múltiples coincidencias) contra el registro.
- **F:** ManifestInfoSpec A1 (mensaje exacto con `[pekko-persistence, pekko-cluster]`) y A2 (dynver `1.0.2` vs `1.0.2+10-abababef`).
- **G:** `enabled=off` → `fallback`; `on` con tipo no registrado → `fallback` y aviso; clave ausente o no booleana → error de configuración.
- **H:** resolver un nombre presente solo en el registro del sistema; nombre solo global; nombre inexistente → error.
- **I:** el texto de origen identifica el closure; formato propio fijado por el Owner; sin ubicación → `type_name`.
- **J1/J2:** por fila, representación aprobada + sitio consumidor en una unidad Rust concreta; 018.2: oráculo TO-NEG-04/05 ya presente.

## 7. Diferencias respecto de los cinco grupos iniciales

1. **Version**: sale HashCode (016.15). Razón: lo necesitan P03 y P04 antes y Version arrastra la decisión bloqueante Q-UTIL-6; se agrupa con Glob por compartir la semántica de cadenas (Q-UTIL-8).
2. **Valores puros**: deja de ser una unidad. 004.3 pasa a P02 (mismo archivo `pekko_exception.rs`, misma decisión Q-UTIL-4 que 004.2), 016.17 a P26 (su único consumidor), 016.5, 016.14 y 016.16 se separan porque no comparten resultado ni decisión.
3. **Representación JVM**: se parte en lo que genera código (E, H, I) y lo que es una disposición (J1). Hay seis filas sin código propio posible y sin test que Pekko afirme; no deben presentarse como una unidad de TDD.
4. **Conversores asíncronos**: 018.4/018.5 salen a P16; 018.1-018.3 son una disposición (J2) y Q-X-4 solo afecta a su cierre.
5. **Posteriores a CFG/EXT/SER**: tres (F, G, H) en vez de cuatro; 016.10 deja de ser «posterior» pero viaja con 016.11.

## 8. Dudas pendientes y preguntas que requieren decisión

**Dudas verificables (no resueltas; no las corregí):**
1. `AC-UTIL-018.2`: el encabezado del contrato dice `impl=partial · validation=oracle`; el TSV dice `pending/pending`. El código (`timeout.rs:54`) y el golden (`util-core.golden:35-36`) existen. Hay que reconciliar estados.
2. `contract-map.tsv` cita Q-UTIL-11 para 3 de las filas P05 que la mencionan en su contrato, y no cita Q-UTIL-8 para 016.15, pese a que el contrato lo hace.
3. Dirección de la arista 016.13→016.7: el contrato dice que `Reflect` usa `BoxedType`; el TSV tiene la inversa.
4. Faltan aristas de consumidor: CFG-016 y Props deberían depender de 016.7; 016.12 debería listar CFG-016.
5. 016.16 WallClock: el historial de la sesión indica trabajo de otro agente fuera de esta rama; no se evaluó su estado.
6. Regla de `reportFailure` de `parasitic` (biblioteca de Scala): medir con el jar.
7. `cargo semver-checks` no está verificado como herramienta del repo (afecta a 019).
8. Mover 018.4/018.5 a P16 no acredita la atomicidad de P16.

**Preguntas para el Owner:**
1. ¿Se acepta que J1 y J2 sean unidades de disposición cuyo cierre es la respuesta a Q-UTIL-11 (y Q-X-4/Q-DSP-3), sin código propio?
2. ¿Se aprueba reasignar 004.3→P02, 016.17→P26, 018.4/018.5→P16?
3. 016.14: ¿`Ordering` o el entero de Pekko? (Q-UTIL-11).
4. 016.9: ¿se acepta cambiar la costura P05→P30 por P25→F (verificando con `cycles.py`)?
5. B: ¿una sola unidad con 005.1 y 005.2 (005.2 quedará `partial` hasta Q-UTIL-6) o dos?
6. D agrupa dos filas sin tema común; ¿se prefiere una por unidad?
