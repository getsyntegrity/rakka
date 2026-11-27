# Plan final de unidades

Sustituye a las 50 specs preliminares de `docs/superpowers/plans/units/` (no se editan; este documento no es generado). Cubre las 783 filas de `docs/contract-map.tsv`, cada una en exactamente una unidad. Los ciclos que obligaban a los saltos hacia adelante y su resolucion estan en `docs/cycles.md`. **Nada de lo aqui descrito esta implementado**: el estado `impl`/`validation` de los TSV del inventario es la unica medida de avance; las filas ya `done`/`partial` (AC-UTIL y AC-PATH) conservan su estado y no se replanifican como pendientes.

## Estado de revision de este plan (2026-10-31)

**Aprobado por el Owner el 2026-11-18 (Q-X-13) como guia de orden y alcance:** las filas y las olas valen; los apartados *Tests/oraculo* de cada unidad se sustituyen por casos `rust-anything` contra Pekko 2.0.0 (Q-X-12) y el criterio de cobertura de *Cierre* pasa a >= 95 % de produccion por crate (Q-X-14, sustituye a >= 90 %). Lo que sigue describe la revision previa a esa aprobacion: **Antes de la aprobacion se decia: este plan no esta aprobado para implementar.** Revisado mecanicamente: asignacion fila -> unidad (`docs/unit-rows.tsv`, comprobada por `check.py`), dependencias declaradas (sin ciclos, ninguna hacia una ola posterior), orden de ejecucion y decisiones por unidad (generados), rutas y crates. **Sin revision semantica**, es decir, no contrastados con los contratos fila por fila y por tanto provisionales: *Resultado verificable*, *Exclusiones/costuras*, *Tests/oraculo* y *Cierre* de cada unidad (varios son genericos o repetidos), y la atomicidad de cada unidad. La particion de P05 se aplico el 2026-11-01 solo como reasignacion documental (ver "Particion de P05" abajo); las propuestas pendientes (P55, `AC-REF-003`, `AC-PATH-015`) no estan aplicadas, y P16 requiere subdivision.

### Particion de P05 (2026-11-01)

Reasignacion documental autorizada por el Owner; **no** aprueba alternativas semanticas, exclusiones ni respuestas a decisiones. Las 26 filas de P05 pasan a: P05 (016.6), P65 (016.15), P66 (005.1), P67 (005.2), P68 (016.5), P69 (016.14), P70 (016.16), P71 (016.7, 016.13), P72 (016.9), P73 (016.10, 016.11), P74 (016.12), P75 (016.8), P76 (016.1, 016.2, 016.3, 016.18, 019), P77 (018.1, 018.2, 018.3), P02 (004.3), P16 (018.4, 018.5) y P26 (016.17). P76 y P77 son **resolucion documental**. Detalle, dudas y preguntas abiertas: `docs/proposals/p05-partition.md`. Dependencias de fila corregidas por contrato: Reflect consume BoxedType; AC-SER-013 consume ClassLoaderObjectInputStream. La atomicidad de las unidades nuevas sigue sin acreditarse.

## Plantilla comun (aplica a todas las unidades; no se repite)

- TDD estricto por comportamiento: RED -> GREEN -> REFACTOR con comando y salida reales guardados en el PR; cobertura `cargo llvm-cov` >= 95 % (Q-X-14; antes >= 90 %) de las lineas de produccion nuevas o modificadas (comando, resultado y lineas no cubiertas con motivo); gates `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Todo archivo Rust que traduce un archivo de Pekko lleva la atribucion de `docs/upstream.md`. Nombres que expresan proposito; los IDs de fila van en comentarios, no en nombres.
- Una fila con decision abierta queda `partial` (nunca `done`) y se anota `Q:`. Validacion: specs de Pekko portados 1:1 y, cuando aplica, golden del oraculo (`tools/oracle/`). Un doble de prueba de una interfaz temporal NO cierra la fila que depende de el: esa fila queda `partial` hasta la prueba de integracion indicada en `docs/cycles.md`.
- Tamano: **sin estimar.** La estimacion anterior (~40 lineas de produccion+tests por fila, uniforme) se retiro: no estaba validada y no sirve para acreditar atomicidad ni numero de PR. La meta orientativa de `AGENTS.md` (~400 lineas de diff propio por PR) se evalua al implementar, leyendo los contratos de las filas; no se fragmenta un comportamiento para cumplirla. Este plan no fija numero de PR por unidad.
- "Decisiones": se listan solo las bloqueantes o rechazadas del registro (`docs/decisions/register.tsv`, `owner-decisions.tsv`) que tocan filas de la unidad, con su estado. Estado global al leer: 1 rechazada (Q-UTIL-1, vigente la alternativa del Owner), 8 bloqueantes sin respuesta, 202 propuestas sin respuesta. Las propuestas de cada fila estan en `docs/contract-map.tsv` (columna `decisions`); regla: la fila avanza en lo independiente y queda `partial`. `Q-X-8` (Typed sobre Classic) esta propuesta, no aceptada: toda la Ola 7-9 asume (a) y se rehace si el Owner elige (b).

## Resumen por ola

| Ola | Unidades | Filas |
|---|---|---|
| Ola 0 - valores sin runtime | P02..P06, P65..P71, P74..P77 (16) | 61 |
| Ola 1 - configuracion, eventos y scheduler | P01, P07..P12, P73 (8) | 70 |
| Ola 2 - nucleo del runtime Classic | P13..P24 (12) | 134 |
| Ola 3 - sistema, proveedor, referencias y extensiones | P25..P30, P72 (7) | 45 |
| Ola 4 - patrones, routers y logging | P31..P37 (7) | 81 |
| Ola 5 - apagado, serializacion e IO | P38..P43 (6) | 84 |
| Ola 6 - testkit Classic y superficie Java | P44..P47 (4) | 50 |
| Ola 7 - Typed: nucleo, adaptador y sistema | P48..P53 (6) | 105 |
| Ola 8 - Typed: contexto, supervision, receptionist y entrega fiable | P54..P60 (7) | 96 |
| Ola 9 - Typed: javadsl y testkits | P61..P64 (4) | 59 |

Total: 77 unidades, 785 filas.

## Orden de ejecución

<!-- BEGIN GENERATED EXECUTION ORDER -->

Orden **derivado** de las dependencias declaradas en las tablas de cada ola (esas dependencias son la fuente de verdad; este bloque lo genera `tools/package/unit_plan.py` y `check.py` lo verifica). Entre las unidades listas se elige la de menor ola y luego la de menor ID. Es un orden válido, no el único.

| # | Unidad | Ola | Depende de |
|---|---|---|---|
| 1 | P02 | 0 | - |
| 2 | P05 | 0 | - |
| 3 | P06 | 0 | P02, P05 |
| 4 | P65 | 0 | - |
| 5 | P66 | 0 | - |
| 6 | P67 | 0 | P65 |
| 7 | P68 | 0 | - |
| 8 | P69 | 0 | - |
| 9 | P70 | 0 | - |
| 10 | P71 | 0 | - |
| 11 | P74 | 0 | - |
| 12 | P75 | 0 | - |
| 13 | P76 | 0 | - |
| 14 | P03 | 0 | P02, P65, P76 |
| 15 | P04 | 0 | P02, P76 |
| 16 | P77 | 0 | P02 |
| 17 | P07 | 1 | P02 |
| 18 | P01 | 1 | P02, P07 |
| 19 | P08 | 1 | P02, P07 |
| 20 | P09 | 1 | P08 |
| 21 | P10 | 1 | P01, P03, P07 |
| 22 | P11 | 1 | P02, P03, P08 |
| 23 | P12 | 1 | P02, P06, P07, P09 |
| 24 | P73 | 1 | P08, P09 |
| 25 | P13 | 2 | P03, P08, P09 |
| 26 | P14 | 2 | P01, P10, P13 |
| 27 | P15 | 2 | P08, P13, P14 |
| 28 | P16 | 2 | P08, P09, P15 |
| 29 | P17 | 2 | P11, P14, P15 |
| 30 | P18 | 2 | P14 |
| 31 | P20 | 2 | P11, P14 |
| 32 | P19 | 2 | P08, P11, P12, P15, P16, P17, P18, P20 |
| 33 | P21 | 2 | P19, P20 |
| 34 | P22 | 2 | P17, P18, P21 |
| 35 | P23 | 2 | P11, P17, P20, P22 |
| 36 | P24 | 2 | P13, P22, P23 |
| 37 | P25 | 3 | P10, P16, P20, P22, P23, P67 |
| 38 | P26 | 3 | P09, P10, P16, P25 |
| 39 | P27 | 3 | P12, P22, P24 |
| 40 | P28 | 3 | P10, P13, P16, P24, P26 |
| 41 | P29 | 3 | P26 |
| 42 | P30 | 3 | P08, P09, P26 |
| 43 | P72 | 3 | P08, P30, P67 |
| 44 | P31 | 4 | P23, P26, P27, P29 |
| 45 | P32 | 4 | P22, P27, P30, P31 |
| 46 | P35 | 4 | P21, P23, P26, P31 |
| 47 | P33 | 4 | P22, P27, P35 |
| 48 | P34 | 4 | P23, P26, P30, P33 |
| 49 | P36 | 4 | P13, P15, P16, P28, P35 |
| 50 | P37 | 4 | P10, P28, P35, P36 |
| 51 | P38 | 5 | P08, P26, P27, P30 |
| 52 | P39 | 5 | P08, P25, P38 |
| 53 | P42 | 5 | P04, P08, P16, P26, P30 |
| 54 | P40 | 5 | P04, P08, P09, P42, P74 |
| 55 | P41 | 5 | P26, P29, P30, P40 |
| 56 | P43 | 5 | P08, P42 |
| 57 | P44 | 6 | P15, P22, P26, P28 |
| 58 | P46 | 6 | P20, P22, P28, P29, P33, P39, P40, P77 |
| 59 | P45 | 6 | P11, P23, P27, P34, P39, P44, P46 |
| 60 | P47 | 6 | P32, P34, P40, P42, P43, P46 |
| 61 | P48 | 7 | P22 |
| 62 | P49 | 7 | P23, P26, P30, P48 |
| 63 | P53 | 7 | P24, P27, P49 |
| 64 | P50 | 7 | P30, P39, P49, P53 |
| 65 | P52 | 7 | P34, P49, P50, P53 |
| 66 | P51 | 7 | P31, P50, P52 |
| 67 | P54 | 8 | P49, P51, P53 |
| 68 | P55 | 8 | P21, P49, P52, P53, P54 |
| 69 | P56 | 8 | P30, P41, P50, P54 |
| 70 | P57 | 8 | P54, P55, P56 |
| 71 | P58 | 8 | P53, P54, P56 |
| 72 | P59 | 8 | P58 |
| 73 | P60 | 8 | P56, P59 |
| 74 | P61 | 9 | P48, P49, P51, P53, P54, P55, P57 |
| 75 | P62 | 9 | P40, P44, P50, P52, P54, P56 |
| 76 | P63 | 9 | P53, P54, P55, P56, P62 |
| 77 | P64 | 9 | P45, P52, P62 |

<!-- END GENERATED EXECUTION ORDER -->

## Ola 0 - valores sin runtime

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P02 | UTIL-NUCLEO | 9 | - |
| P03 | UTIL-ESTRUCTURAS | 16 | P02, P65, P76 |
| P04 | UTIL-BYTES | 9 | P02, P76 |
| P05 | UTIL-VARIOS | 1 | - |
| P06 | UTIL-PLATAFORMA | 6 | P02, P05 |
| P65 | UTIL-HASHCODE | 1 | - |
| P66 | UTIL-PEKKOVERSION | 1 | - |
| P67 | UTIL-VERSION | 1 | P65 |
| P68 | UTIL-PRETTYDURATION | 1 | - |
| P69 | UTIL-UUID-ORDEN | 1 | - |
| P70 | UTIL-RELOJ | 1 | - |
| P71 | UTIL-REGISTRO-FABRICAS | 2 | - |
| P74 | UTIL-TIPOS-DESERIALIZACION | 1 | - |
| P75 | UTIL-ORIGEN-CLOSURES | 1 | - |
| P76 | UTIL-EQUIVALENCIAS | 5 | - |
| P77 | UTIL-FRONTERAS | 3 | P02 |

### P02 UTIL-NUCLEO

- **Resultado verificable:** Timeout, Done, NotUsed, excepciones base y Helpers puros cerrados contra los jars.
- **Filas (9; impl: partial 8, pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-001, AC-UTIL-002.1, AC-UTIL-002.2, AC-UTIL-002.3, AC-UTIL-003.1, AC-UTIL-003.2, AC-UTIL-004.1, AC-UTIL-004.2, AC-UTIL-004.3.
- **Exclusiones/costuras:** Q-UTIL-2..5 y Q-UTIL-7 siguen abiertas: filas partial hasta decision; no se replanifica lo ya implementado. Desde 2026-11-01 incluye AC-UTIL-004.3 (OnlyCauseStackTrace, Q-UTIL-4/Q-X-9), proveniente de P05; `impl` pending.
- **Decisiones:** 7 distintas en sus filas; respondidas: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P02). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/{done,not_used,pekko_exception}.rs` y `crates/rakka-actor/src/util/{timeout,time_unit,helpers}.rs` (existentes: implementados parcialmente); tests existentes en `crates/rakka-actor/tests/`.
- **Tests/oraculo:** unidad + oraculo jars.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P03 UTIL-ESTRUCTURAS

- **Resultado verificable:** Estructuras de datos concurrentes/inmutables (Index, WildcardIndex, SubclassifiedIndex, colas, LFU/LRU).
- **Filas (16; impl: pending 16):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-007, AC-UTIL-008, AC-UTIL-009, AC-UTIL-010, AC-UTIL-011, AC-UTIL-012, AC-UTIL-013, AC-UTIL-014, AC-UTIL-015.1, AC-UTIL-015.2, AC-UTIL-016.4, AC-UTIL-017.1, AC-UTIL-017.2, AC-UTIL-017.3, AC-UTIL-017.4, AC-UTIL-017.5.
- **Exclusiones/costuras:** UTIL-008/009/012/013 se prueban solo con sus propios specs; el uso por EVT/CFG/MBX es de las unidades consumidoras.
- **Decisiones:** 10 distintas en sus filas; respondidas: 9 (ver detalle); propuestas sin respuesta: Q-UTIL-21. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P03). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P65, P76.
- **Archivos previstos:** `crates/rakka-actor/src/util/{index,wildcard_index,queues,lfu}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs Pekko 1:1 (Index/WildcardIndex/queues).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P04 UTIL-BYTES

- **Resultado verificable:** ByteString, ByteStringBuilder/Iterator y adaptadores de flujo.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-006.1, AC-UTIL-006.2, AC-UTIL-006.3, AC-UTIL-006.4, AC-UTIL-006.5, AC-UTIL-006.8, AC-UTIL-006.7, AC-UTIL-006.6, AC-UTIL-022.
- **Exclusiones/costuras:** Q-UTIL-12 (bloqueante) fija el alcance: sin respuesta solo avanzan las filas que no dependen de la representacion.
- **Decisiones:** 11 distintas en sus filas; respondidas: Q-CFG-11, Q-UTIL-11, Q-UTIL-12, Q-UTIL-14, Q-UTIL-8, Q-X-3; propuestas sin respuesta: Q-UTIL-22, Q-UTIL-23, Q-UTIL-24, Q-UTIL-25, Q-UTIL-26. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P04). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P76.
- **Archivos previstos:** `crates/rakka-actor/src/util/bytes/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs ByteStringSpec portados + oraculo jars.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P05 UTIL-GLOB

- **Resultado verificable:** `glob_matches(patron, entrada)`: `?` = exactamente un caracter, `*` = cualquier secuencia (incluida vacia), el resto literal, `**` equivale a `*`; algoritmo lineal con retroceso al ultimo `*`, sin retroceso explosivo.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.6.
- **Exclusiones/costuras:** no reproduce `Helpers.makePattern` (AC-UTIL-023, otra fila, P06); la unidad de caracter (UTF-16 o punto de codigo) depende de Q-UTIL-8. Antes de la particion del 2026-11-01 esta unidad tenia 26 filas; sus otras 25 se repartieron en P02, P16, P26 y P65..P77 (`docs/proposals/p05-partition.md`).
- **Decisiones:** 1 distintas en sus filas; respondidas: Q-UTIL-8. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P05). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/glob.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** GlobSpec A1-A3 (literales, `?`, `*`), A4 (equivalencia exhaustiva con una regex de referencia sobre patrones de 0-4 caracteres en {a,b,*,?} y entradas de 0-4 en {a,b}), A5 (rendimiento: `(*a)x12*b` contra 36 `a` en < 3 s; 2000 caracteres en < 5 s). Sin oraculo: funcion pura.
- **Cierre:** A1-A5 verdes con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-8 siga sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P06 UTIL-PLATAFORMA

- **Resultado verificable:** Helpers de plataforma (makePattern, isWindows, timestamp, Unsafe).
- **Filas (8; impl: pending 8):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-020, AC-UTIL-021, AC-UTIL-023, AC-UTIL-024, AC-UTIL-025, AC-UTIL-026, AC-UTIL-027, AC-UTIL-028.
- **Exclusiones/costuras:** UTIL-020/021 son parte de Unsafe/MiMa: pueden quedar no aplicables solo con autorizacion del Owner (A-5); aqui se planifican. (2026-11-01) Declara P02 (AC-UTIL-026 depende de AC-UTIL-002.2): antes lo cubria P05 por cierre transitivo.
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-UTIL-10, Q-UTIL-11, Q-UTIL-13, Q-UTIL-8; propuestas sin respuesta: Q-UTIL-18, Q-UTIL-19. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P06). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P05.
- **Archivos previstos:** `crates/rakka-actor/src/util/platform.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** unidad + oraculo jars.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P65 UTIL-HASHCODE

- **Resultado verificable:** `hash(seed, valor)` = `37*seed + contribucion` con SEED=23 para bool/char/short/int/long/float/double (aritmetica `i32` con wrapping) y encadenado de arreglos; valores identicos a los medidos con el jar.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.15.
- **Exclusiones/costuras:** el caso `AnyRef` (`hashCode` generico) solo se reproduce para los tipos que lo necesiten (Q-UTIL-8 strings UTF-16, Q-UTIL-14); `hash(seed, null)` lanza `IllegalArgumentException` en Pekko y no se modela (diferencia a registrar). Lo consumen `Version.hashCode` (P67) y AC-UTIL-017.1 (P03).
- **Decisiones:** 3 distintas en sus filas; respondidas: Q-PATH-3, Q-UTIL-14, Q-UTIL-8. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P65). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/hash_code.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** No existe spec de Pekko. Golden con el jar (script por crear en `tools/oracle/`): `hash(23,5)=856`, `hash(23,true)=852`, `hash(23,5000000000L)=705033556`, `hash(23,"abc")=97205`, `hash(23,1.5)=1073218387`, `hash(23,int[]{1,2})=31526` y extremos `Long.MIN_VALUE`, `NaN`, `-0.0`.
- **Cierre:** valores medidos verdes con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-8 y Q-UTIL-14 sigan sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P66 UTIL-PEKKOVERSION

- **Resultado verificable:** `PekkoVersion::require(libreria, version_requerida)` con las reglas de PekkoVersionSpec (misma mayor/menor, parche, M/RC, SNAPSHOT y versiones incomprensibles no fallan) y `UnsupportedPekkoVersion` con el mensaje `Current version of Pekko is [<cur>], but <lib> requires version [<req>]`.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-005.1.
- **Exclusiones/costuras:** el valor de `Version.current` lo decide Q-CFG-5 (bloquea esta fila); `util.Version` (P67) queda fuera: segun el contrato `PekkoVersion.require` no depende de otra fila.
- **Decisiones:** 2 distintas en sus filas; respondidas: Q-CFG-5, Q-UTIL-17. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P66). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/pekko_version.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** PekkoVersionSpec A1-A6 portados; A7 (texto exacto del mensaje) derivada del codigo, comprobable con el jar 1.7.1 (`PekkoVersion.require` es publico; escenario por crear).
- **Cierre:** A1-A6 verdes con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-CFG-5 siga sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P67 UTIL-VERSION

- **Resultado verificable:** `Version`: parseo de 1 a 3 segmentos con calificador (`-rest`, `+dynver`), errores (`Empty version not supported.`, `Only 3 digits separated with '.' are supported. [<v>]`, `NumberFormatException`), igualdad por `compareTo == 0`, comparacion numerica y dynver, `hashCode` con `HashCode.SEED`.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-005.2.
- **Exclusiones/costuras:** `ManifestInfo` queda en P72. La comparacion con calificador contra un lado vacio (VersionSpec A5/A8; el jar mide `1.2.0` vs `1.2-M1` = -2 y el inverso = -1) depende de la decision **bloqueante** Q-UTIL-6: no se implementa una variante sin respuesta del Owner.
- **Decisiones:** 1 distintas en sus filas; respondidas: Q-UTIL-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P67). <!-- generated:unit-decisions -->
- **Dependencias:** P65.
- **Archivos previstos:** `crates/rakka-actor/src/util/version.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** VersionSpec A1-A4, A6, A7 portados (puede avanzar segun la ficha de Q-UTIL-6); A5 y A8 (antisimetria) solo como caracterizacion contra el jar, sin fijar el resultado hasta que responda el Owner.
- **Cierre:** A1-A4, A6-A7 verdes con evidencia RED->GREEN, cobertura >= 90 %; A5/A8 y el cierre de la fila siguen abiertos mientras Q-UTIL-6 (bloqueante) no tenga respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P68 UTIL-PRETTYDURATION

- **Resultado verificable:** `pretty(duracion, precision, include_nanos)`: unidad mas grande con valor entero >= 1, `%.{precision}g` de `java.util.Formatter` (incluida la notacion cientifica: `10000 days` -> `1.000e+04 d`), `∞ (infinity)`, `-∞ (minus infinity)`, `undefined`, `precision <= 0` falla con `requirement failed: precision must be > 0`.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.5.
- **Exclusiones/costuras:** no incluye otros formateos de logs; el `%g` de Java no esta en `upstream/pekko` (se mide con el jar, Q-UTIL-9).
- **Decisiones:** 1 distintas en sus filas; respondidas: Q-UTIL-9. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P68). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/pretty_duration.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** PrettyDurationSpec A1-A4 (10 casos de la tabla, `infinity`, `minus infinity`, `undefined`); A5 medido con el jar (`10000 days`, `99999 days`, `1 day`, `1500 microseconds`, `Duration.Inf`); golden por crear.
- **Cierre:** A1-A5 verdes con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-9 siga sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P69 UTIL-UUID-ORDEN

- **Resultado verificable:** `compare_uuid(a, b)`: orden por version; para ambos v1, por `timestamp()` como entero de 64 bits sin signo y, en empate, por los 64 bits menos significativos sin signo; para el resto, por los 64 bits mas y menos significativos sin signo.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.14.
- **Exclusiones/costuras:** el entero devuelto por Pekko (p. ej. `-3` = diferencia de versiones) no se reproduce salvo que Q-UTIL-11 lo decida (propuesta: `Ordering`); el unico consumidor esta fuera de estos modulos (persistence-query).
- **Decisiones:** 2 distintas en sus filas; respondidas: Q-UTIL-11, Q-X-18. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P69). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/uuid_ordering.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** No existe spec de Pekko. Diferencial con el jar (`tools/oracle/uuid-comparator.jsh`, por crear): v1 con timestamps extremos y v4 con bit alto (`new UUID(0x4000..., 0x8000...01)` > `new UUID(0x4000..., 1)`), diferencia de versiones.
- **Cierre:** casos del jar verdes con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-11 siga sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P70 UTIL-RELOJ

- **Resultado verificable:** `AlwaysIncreasingClock::current_time_millis()`: devuelve la hora de pared pero nunca repite ni retrocede (si no avanzo, `anterior + 1`), de forma atomica.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.16.
- **Exclusiones/costuras:** esta fila no tiene decision abierta. Existe trabajo de otros agentes sobre esta fila en ramas `ao/ac-util-016.16-*`; este plan no lo evalua ni cambia su estado: `impl`/`validation` siguen siendo los del inventario.
- **Decisiones:** ninguna asociada a sus filas en `contract-map.tsv`. <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/wall_clock.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** No existe spec de Pekko; A1 (monotonia estricta) derivada de `WallClock.scala:46-55`. Prueba con lecturas consecutivas y con hilos concurrentes. Sin oraculo.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P71 UTIL-REGISTRO-FABRICAS

- **Resultado verificable:** Resolucion por nombre registrado y firma de argumentos (`TypeId`) con los errores `no matching constructor found on <clazz> for arguments [...]` y `multiple matching constructors found on <clazz> for arguments [...]` (un argumento ausente se modela con `Option`); `BoxedType` se registra como trivial por diseno del lenguaje.
- **Filas (2; impl: pending 2):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.7, AC-UTIL-016.13.
- **Exclusiones/costuras:** `findClassLoader`, `getCallerClass`, `findMarker` e `instantiate` con argumentos incompatibles (ReflectSpec A5) no tienen equivalente: la exclusion exige aprobacion del Owner (Q-UTIL-11). Direccion de la dependencia: **Reflect consume BoxedType** (`Reflect.scala:111`), no al reves. La API definitiva del registro la fija Q-X-2; AC-CFG-016 (P09) la consumira, pero su contrato no cita Reflect (arista no registrada).
- **Decisiones:** 3 distintas en sus filas; respondidas: Q-CFG-8, Q-UTIL-11, Q-X-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P71). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/factory_registry.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** ReflectSpec A1-A4 portados sobre el registro (1 y 2 parametros, `null`, varios constructores, `null` ambiguo -> multiples coincidencias); `BoxedType` caracterizado con el jar (`int` -> `Integer`, `void` -> `BoxedUnit`), sin test de Pekko.
- **Cierre:** A1-A4 verdes con evidencia RED->GREEN, cobertura >= 90 %; las filas quedan `partial` mientras Q-X-2, Q-CFG-8 y Q-UTIL-11 sigan sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P74 UTIL-TIPOS-DESERIALIZACION

- **Resultado verificable:** Resolver un tipo por nombre contra el registro de tipos del sistema antes que uno global (efecto de `ClassLoaderObjectInputStream`), con error si ninguno lo conoce.
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.12.
- **Exclusiones/costuras:** no hay serializacion Java ni class loaders; la forma exacta del registro la fijan Q-X-2 y AC-SER-003. Direccion de la dependencia: **AC-SER-013 (JavaSerializer.fromBinary, `Serializer.scala:358`) consume esta fila** (P40 la declara); la arista inversa del inventario original se retiro.
- **Decisiones:** 3 distintas en sus filas; respondidas: Q-CFG-11, Q-UTIL-11, Q-X-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P74). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/type_resolution.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** No existe test de Pekko; caracterizacion propia del orden registro-del-sistema/global y del error. Se completa con las pruebas de AC-SER-003/013 cuando exista el registro.
- **Cierre:** caracterizacion verde con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-11, Q-X-2 y Q-CFG-11 sigan sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P75 UTIL-ORIGEN-CLOSURES

- **Resultado verificable:** Texto de origen de un closure para `Debug`/`Display` de comportamientos e interceptores (`<archivo>:<linea>` con `#[track_caller]`, o `type_name` sin ubicacion).
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.8.
- **Exclusiones/costuras:** no se reimplementa el parser de class files; los rangos `from-to` y los nombres de archivo de compilacion de Scala/Java no se reproducen. El formato lo fija Q-UTIL-16 (bloquea la implementacion segun su ficha) y condiciona los tests de `toString` de Typed.
- **Decisiones:** 2 distintas en sus filas; respondidas: Q-UTIL-11, Q-UTIL-16. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P75). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/util/closure_origin.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** LineNumberSpec A1-A2 no son portables (dependen del compilador de Scala); caracterizacion propia del formato elegido.
- **Cierre:** caracterizacion verde con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-16 y Q-UTIL-11 sigan sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P76 UTIL-EQUIVALENCIAS

- **Resultado verificable:** Disposicion documentada, fila por fila, de OptionVal (016.1), ConstantFun (016.2), Collections (016.3), Unused (016.18) y las anotaciones de API (019): equivalente Rust o exclusion aprobada por el Owner.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.1, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-016.18, AC-UTIL-019.
- **Tipo:** Tipo: **resolucion documental**. Una respuesta del Owner no acredita por si sola implementacion, validacion ni equivalencia: las filas conservan `impl`/`validation` en `pending` y sus decisiones abiertas hasta que exista evidencia en las unidades consumidoras.
- **Exclusiones/costuras:** no hay codigo propio previsto en esta unidad: `Option<T>`, closures, iteradores, prefijo `_` y visibilidad/`sealed` se materializan en las unidades consumidoras (P03 usa OptionVal; P04 usa Collections). `cargo semver-checks` (019) no esta verificado como herramienta del repositorio.
- **Decisiones:** 2 distintas en sus filas; respondidas: Q-UTIL-11; propuestas sin respuesta: Q-UTIL-20. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P76). <!-- generated:unit-decisions -->
- **Dependencias:** ninguna.
- **Archivos previstos:** sin archivo Rust previsto (resolucion documental).
- **Tests/oraculo:** Ninguno propio: A1/A2 de ConstantFun no tienen sentido en Rust; el resto no tiene test en Pekko. La evidencia vendra de las unidades consumidoras.
- **Cierre:** respuesta del Owner registrada para cada fila (Q-UTIL-11) y enlazada con el sitio consumidor; esto no cierra `impl` ni `validation`.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

### P77 UTIL-FRONTERAS

- **Resultado verificable:** Disposicion documentada de FutureConverters (018.1), OptionConverters/FunctionConverters/JavaDurationConverters (018.2) y ccompat/compat.Future (018.3): equivalente Rust o exclusion aprobada por el Owner.
- **Filas (3; impl: pending 3):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3.
- **Tipo:** Tipo: **resolucion documental**. Una respuesta del Owner no acredita por si sola implementacion, validacion ni equivalencia: las filas conservan `impl`/`validation` en `pending` y sus decisiones abiertas hasta que exista evidencia en las unidades consumidoras.
- **Exclusiones/costuras:** Sin archivo Rust nuevo previsto. Evidencia previa de 018.2 limitada a la conversion `JDuration.asScala` -> Timeout (`crates/rakka-actor/src/util/timeout.rs:54`, `TryFrom<std::time::Duration>`; oraculo `util-core.golden` TO-NEG-04/05): no cubre Optional*, FunctionConverters ni `Duration.asJava`. **Contradiccion sin resolver:** la cabecera del contrato de 018.2 dice `impl=partial`/`validation=oracle` y el TSV `pending`/`pending`; el estado no se modifico. Relacion con Q-DSP-3 (AC-DSP-017.2 usa `compat.Future.fold/reduce/find`): no es decision de estas filas.
- **Decisiones:** 5 distintas en sus filas; respondidas: Q-UTIL-1, Q-UTIL-11, Q-X-4; propuestas sin respuesta: Q-DSP-3, Q-UTIL-27. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P77). <!-- generated:unit-decisions -->
- **Dependencias:** P02.
- **Archivos previstos:** sin archivo Rust previsto (resolucion documental); evidencia previa de 018.2 en `crates/rakka-actor/src/util/timeout.rs`.
- **Tests/oraculo:** Oraculo `util-core` TO-NEG-04/05 (ya activo) solo para el overflow de `Timeout.create`; A1 de 018.1/018.2/018.3 son pruebas de compilacion Java/Scala sin aserciones de ejecucion.
- **Cierre:** respuesta del Owner registrada para cada fila (Q-UTIL-11, Q-X-4) y estado de 018.2 reconciliado; esto no cierra `impl` ni `validation`.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

## Ola 1 - configuracion, eventos y scheduler

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P01 | PATH | 16 | P02, P07 |
| P07 | CFG-HOCON | 12 | P02 |
| P08 | CFG-AJUSTES | 10 | P02, P07 |
| P09 | CFG-ENTORNO | 3 | P08 |
| P10 | CFG-DEPLOY | 3 | P01, P03, P07 |
| P11 | EVT-BUS | 11 | P02, P03, P08 |
| P12 | SCH-RUEDA | 13 | P02, P06, P07, P09 |
| P73 | UTIL-JFR | 2 | P08, P09 |

### P01 PATH

- **Resultado verificable:** Address/ActorPath completos (filas partial restantes) con los goldens de PATH.
- **Filas (16; impl: done 7, partial 8, pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-PATH-001, AC-PATH-002, AC-PATH-003, AC-PATH-004, AC-PATH-005, AC-PATH-006, AC-PATH-007, AC-PATH-008, AC-PATH-009, AC-PATH-010, AC-PATH-011, AC-PATH-012, AC-PATH-013, AC-PATH-014, AC-PATH-015, AC-PATH-016.
- **Exclusiones/costuras:** Las 7 filas done no se tocan; PATH-015 (paths especiales) usa solo constantes, el proveedor llega en P26.
- **Decisiones:** 8 distintas en sus filas; respondidas: 7 (ver detalle); propuestas sin respuesta: Q-PATH-7. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P01). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P07.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{address,path,uri,jstring,errors}.rs` (existentes: implementados parcialmente); tests existentes en `crates/rakka-actor/tests/`.
- **Tests/oraculo:** unidad + oraculo jars (golden existente).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P07 CFG-HOCON

- **Resultado verificable:** Cargador HOCON + semantica de duraciones/tamanos/literales + validacion contra reference.conf.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-002.4, AC-UTIL-002.5, AC-CFG-001, AC-CFG-003, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5, AC-CFG-017.6, AC-CFG-017.7, AC-CFG-019.
- **Exclusiones/costuras:** Detalle en docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md; Q-X-1 aceptada (condición cumplida en #243); la unidad queda partial por las propuestas Q-CFG-14..16. (2026-11-01) Se retira la dependencia declarada de P05: ninguna fila de P07 depende de las 26 filas que P05 tenia; verificacion semantica de la dependencia original pendiente.
- **Decisiones:** 12 distintas en sus filas; respondidas: 7 (ver detalle); propuestas sin respuesta: Q-CFG-14, Q-CFG-15, Q-CFG-16, Q-CFG-18, Q-CFG-19. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P07). <!-- generated:unit-decisions -->
- **Dependencias:** P02.
- **Archivos previstos:** `crates/rakka-actor/src/config/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** unidad + golden jars (Typesafe Config).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P08 CFG-AJUSTES

- **Resultado verificable:** ActorSystem.Settings fail-fast: todas las claves pekko.actor.* leidas y validadas.
- **Filas (10; impl: pending 10):** asignacion explicita en `docs/unit-rows.tsv`: AC-CFG-002, AC-CFG-004, AC-CFG-005, AC-CFG-006, AC-CFG-007, AC-CFG-009, AC-CFG-010, AC-CFG-011, AC-CFG-018, AC-CFG-020.
- **Exclusiones/costuras:** Costura SETTINGS-S1: struct Settings con los campos; los consumidores (EVT-011 niveles, EXT cargas, CS hooks, SER bindings) leen el campo, no lo implementan.
- **Decisiones:** 14 distintas en sus filas; respondidas: 10 (ver detalle); propuestas sin respuesta: Q-CFG-15, Q-CFG-17, Q-CFG-18, Q-CFG-19. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P08). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P07.
- **Archivos previstos:** `crates/rakka-actor/src/config/settings.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** unidad con configs sinteticas.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P09 CFG-ENTORNO

- **Resultado verificable:** ProviderSelection, Setup/BootstrapSetup y DynamicAccess (registro de fabricas).
- **Filas (3; impl: pending 3):** asignacion explicita en `docs/unit-rows.tsv`: AC-CFG-008, AC-CFG-015, AC-CFG-016.
- **Exclusiones/costuras:** Q-X-2 (registro de fabricas) abierta; CFG-008 queda partial hasta que el proveedor exista (P26).
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-CFG-13, Q-CFG-9, Q-X-10, Q-X-2; propuestas sin respuesta: Q-CFG-31, Q-CFG-32, Q-CFG-33, Q-CFG-34. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P09). <!-- generated:unit-decisions -->
- **Dependencias:** P08.
- **Archivos previstos:** `crates/rakka-actor/src/config/{setup,dynamic_access}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** unidad.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P10 CFG-DEPLOY

- **Resultado verificable:** Deployer: parseo pekko.actor.deployment, lookup con comodines, Deploy/Scope.
- **Filas (3; impl: pending 3):** asignacion explicita en `docs/unit-rows.tsv`: AC-CFG-012.1, AC-CFG-012.2, AC-CFG-013.
- **Exclusiones/costuras:** CFG-012.3/.4 (RouterConfig y resizers) se difieren a P37 (RT-CONFIG); aqui el campo router es un valor opaco (costura DEPLOY-S1).
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-CFG-12, Q-CFG-4, Q-UTIL-8, Q-X-2, Q-X-21; propuestas sin respuesta: Q-CFG-35, Q-CFG-36, Q-UTIL-21. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P10). <!-- generated:unit-decisions -->
- **Dependencias:** P01, P03, P07.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{deploy,deployer}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs DeployerSpec parciales + unidad.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P11 EVT-BUS

- **Resultado verificable:** EventBus, EventStream y tipos de log/dead letters/adaptadores de log sin logger real.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-EVT-001, AC-EVT-002, AC-EVT-003, AC-EVT-004, AC-EVT-006, AC-EVT-008, AC-EVT-012, AC-EVT-013.1, AC-EVT-013.2, AC-EVT-015, AC-EVT-016.
- **Exclusiones/costuras:** Costura EVT-S1: EventStream sin vigilancia (EVT-007 en P34) ni LoggingBus (EVT-011 en P34).
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-EVT-1, Q-EVT-3, Q-X-2, Q-X-3, Q-X-6; propuestas sin respuesta: Q-REF-12. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P11). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P03, P08.
- **Archivos previstos:** `crates/rakka-actor/src/event/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs EventStreamSpec/LoggingSpec parciales.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P12 SCH-RUEDA

- **Resultado verificable:** Scheduler (rueda LARS) con reloj y ticks verificables.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AC-SCH-001, AC-SCH-002, AC-SCH-003, AC-SCH-004, AC-SCH-005, AC-SCH-006, AC-SCH-008.1, AC-SCH-008.2, AC-SCH-008.3, AC-SCH-009, AC-SCH-010, AC-SCH-014, AC-SCH-015.
- **Exclusiones/costuras:** SCH-007/011/012/013 (actores y timers) a P27; la cancelacion de tarea a actor terminado usa solo la interfaz ActorRefLike.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-SCH-1, Q-SCH-2, Q-SCH-4, Q-X-2, Q-X-4, Q-X-5; propuestas sin respuesta: Q-SCH-5. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P12). <!-- generated:unit-decisions -->
- **Dependencias:** P02, P06, P07, P09.
- **Archivos previstos:** `crates/rakka-actor/src/actor/scheduler/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** LightArrayRevolverSchedulerSpec 1:1 con reloj manual.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P73 UTIL-JFR

- **Resultado verificable:** Clave `pekko.java-flight-recorder.enabled` aceptada y validada como booleana; `load` devuelve el `fallback` noop si esta desactivada o si la carga falla (con aviso). `JavaVersion` no tiene equivalente ejecutable.
- **Filas (2; impl: pending 2):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.10, AC-UTIL-016.11.
- **Exclusiones/costuras:** no hay backend JFR; `JavaVersion` (016.10) no aporta comportamiento verificable propio en Rust y solo es condicion de 016.11 (la constante de capacidad de hilos virtuales la define AC-DSP-014): su agrupacion con 016.11 es una duda registrada. Q-CFG-7 bloquea la implementacion de 016.11 segun su ficha.
- **Decisiones:** 2 distintas en sus filas; respondidas: Q-CFG-7, Q-UTIL-11. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P73). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P09.
- **Archivos previstos:** `crates/rakka-actor/src/util/flight_recorder.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** No existe test de Pekko; caracterizacion propia: `enabled=off` -> `fallback`; `on` con tipo no registrado -> `fallback` y aviso; clave ausente o no booleana -> error de configuracion.
- **Cierre:** caracterizacion verde con evidencia RED->GREEN, cobertura >= 90 %; las filas quedan `partial` mientras Q-CFG-7 y Q-UTIL-11 sigan sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

## Ola 2 - nucleo del runtime Classic

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P13 | MBX-COLAS | 12 | P03, P08, P09 |
| P14 | REF-BASE | 15 | P01, P10, P13 |
| P15 | MBX-NUCLEO | 7 | P08, P13, P14 |
| P16 | DSP-EJECUTORES | 19 | P08, P09, P15 |
| P17 | REF-MUERTOS | 4 | P11, P14, P15 |
| P18 | CELL-HIJOS | 6 | P14 |
| P19 | CELL-EJECUCION | 11 | P08, P11, P12, P15, P16, P17, P18, P20 |
| P20 | SUP-ESTRATEGIAS | 10 | P11, P14 |
| P21 | SUP-FALLO | 11 | P19, P20 |
| P22 | CELL-REINICIO-PARADA | 21 | P17, P18, P21 |
| P23 | DW-VIGILANCIA | 13 | P11, P17, P20, P22 |
| P24 | MBX-STASH | 5 | P13, P22, P23 |

### P13 MBX-COLAS

- **Resultado verificable:** Colas de mailbox (unbounded, bounded, prioridad, deque, control-aware) y vista de solo lectura.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AC-MBX-004, AC-MBX-005, AC-MBX-006, AC-MBX-007, AC-MBX-008, AC-MBX-009, AC-MBX-010, AC-MBX-011, AC-MBX-012, AC-MBX-015, AC-MBX-019, AC-MBX-022.
- **Exclusiones/costuras:** MBX-013/014/020 (seleccion) a P28; MBX-019 expone vista interna, Q-RT-4 (bloqueante) decide su forma.
- **Decisiones:** 13 distintas en sus filas; respondidas: Q-MBX-14, Q-MBX-15, Q-MBX-16, Q-MBX-6, Q-X-2, Q-X-4; propuestas sin respuesta: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P13). <!-- generated:unit-decisions -->
- **Dependencias:** P03, P08, P09.
- **Archivos previstos:** `crates/rakka-actor/src/dispatch/mailbox/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs *MailboxSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P14 REF-BASE

- **Resultado verificable:** Identidad/entrega/Props de ActorRef sobre una interfaz de celda minima.
- **Filas (15; impl: pending 15):** asignacion explicita en `docs/unit-rows.tsv`: AC-REF-001.1, AC-REF-001.2, AC-REF-001.3, AC-REF-002, AC-REF-004, AC-REF-005, AC-REF-006, AC-REF-007.1, AC-REF-007.2, AC-REF-008, AC-REF-018, AC-REF-022, AC-REF-024, AC-REF-025, AC-DSP-020.
- **Exclusiones/costuras:** Costura CELL-S1: trait ActorRefLike {tell, send_system}; REF-002/018 se prueban con un buzon fake; la celda real llega en P15-P22. (2026-11-01) Se retira la dependencia declarada de P05: ninguna fila de P14 depende de las 26 filas que P05 tenia; verificacion semantica de la dependencia original pendiente.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-2, Q-X-3, Q-X-8; propuestas sin respuesta: Q-REF-1, Q-REF-2, Q-REF-4, Q-REF-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P14). <!-- generated:unit-decisions -->
- **Dependencias:** P01, P10, P13.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{actor_ref,props}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorRefSpec parcial + unidad.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P15 MBX-NUCLEO

- **Resultado verificable:** Estado atomico del mailbox, ejecucion con prioridad de sistema, throughput, attach/detach y suspend/resume de dispatcher.
- **Filas (7; impl: pending 7):** asignacion explicita en `docs/unit-rows.tsv`: AC-MBX-001, AC-MBX-002, AC-CELL-013.1, AC-DSP-001, AC-DSP-002, AC-DSP-003, AC-DSP-023.
- **Exclusiones/costuras:** Costura DISPATCH-S1: la celda se sustituye por trait CellInvoke {invoke, system_invoke}; DSP-023 usa suspend/resume sin restart real.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-MBX-10, Q-MBX-5, Q-MBX-6, Q-MBX-9, Q-X-4; propuestas sin respuesta: Q-CELL-2, Q-DSP-5, Q-MBX-11. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P15). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P13, P14.
- **Archivos previstos:** `crates/rakka-actor/src/dispatch/{mailbox,dispatcher}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** Dispatchers/Mailbox specs 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P16 DSP-EJECUTORES

- **Resultado verificable:** Registro de dispatchers, ejecutores (fork-join, pool, pinned, affinity, virtual), batching y contextos de ejecucion.
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-018.4, AC-UTIL-018.5, AC-DSP-004, AC-DSP-005, AC-DSP-006, AC-DSP-008, AC-DSP-009, AC-DSP-010, AC-DSP-011, AC-DSP-013, AC-DSP-014, AC-DSP-015, AC-DSP-016.1, AC-DSP-016.2, AC-DSP-017.1, AC-DSP-017.2, AC-DSP-018, AC-DSP-019, AC-DSP-022.
- **Exclusiones/costuras:** Q-DSP-1 decide los ejecutores; DSP-007 (seleccion por actor) a P28. Desde 2026-11-01 incluye AC-UTIL-018.4 y AC-UTIL-018.5 (proceden de P05). **Requiere subdivision (registrado, no aplicado):** 19 filas heterogeneas (configuracion y busqueda de dispatchers, pools de hilos, ejecucion/batching y contextos de ejecucion); atomicidad no acreditada y la unidad NO esta lista para implementar. 018.4 y AC-DSP-017.1 describen la misma pieza (SameThreadExecutionContextSpec:40-66); la direccion de su dependencia no la fija ningun contrato (`Future.scala:104` muestra que `ExecutionContexts.parasitic` usa `SameThreadExecutionContext()`); se implementa una sola vez. La regla de `reportFailure` de `parasitic` (biblioteca de Scala) esta sin medir.
- **Decisiones:** 13 distintas en sus filas; respondidas: Q-UTIL-11, Q-X-1, Q-X-2, Q-X-4, Q-X-5; propuestas sin respuesta: 8 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P16). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P09, P15.
- **Archivos previstos:** `crates/rakka-actor/src/dispatch/{dispatchers,executors}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** DispatchersSpec + unidad.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P17 REF-MUERTOS

- **Resultado verificable:** deadLetters, EmptyLocalActorRef y mailbox cerrado: el sumidero de cartas muertas.
- **Filas (4; impl: pending 4):** asignacion explicita en `docs/unit-rows.tsv`: AC-REF-015, AC-REF-016, AC-MBX-003, AC-MBX-021.
- **Exclusiones/costuras:** Watch->DWN de REF-015/016 usa la interfaz DeathWatchSink (DW-S1); la vigilancia real llega en P23.
- **Decisiones:** 4 distintas en sus filas; respondidas: Q-MBX-10, Q-MBX-5, Q-MBX-8, Q-MBX-9. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P17). <!-- generated:unit-decisions -->
- **Dependencias:** P11, P14, P15.
- **Archivos previstos:** `crates/rakka-actor/src/actor/dead_letters.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** unidad + oraculo de comportamiento.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P18 CELL-HIJOS

- **Resultado verificable:** Contenedor de hijos y actorOf: reserva atomica de nombres.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AC-CELL-015.1, AC-REF-009.1, AC-REF-009.2, AC-REF-009.3, AC-REF-010, AC-REF-019.
- **Exclusiones/costuras:** CELL-015.1 y REF-009.2 son un solo comportamiento (R28): se implementan juntos aqui.
- **Decisiones:** 2 distintas en sus filas; propuestas sin respuesta: Q-MBX-11, Q-REF-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P18). <!-- generated:unit-decisions -->
- **Dependencias:** P14.
- **Archivos previstos:** `crates/rakka-actor/src/actor/children.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorCreationSpec/ActorRefSpec parciales.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P19 CELL-EJECUCION

- **Resultado verificable:** Celda de actor: ciclo de mensajes, become, mensajes de sistema, ReceiveTimeout y API de contexto.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-CELL-001, AC-CELL-002.1, AC-CELL-003, AC-CELL-005, AC-CELL-013.2, AC-CELL-014, AC-CELL-017, AC-CELL-018, AC-CELL-019, AC-CELL-022, AC-CELL-031.
- **Exclusiones/costuras:** Los hooks de reinicio/parada (CELL-006/007/008) llegan en P21-P22; 013.3 en P23.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-CELL-1, Q-X-3, Q-X-4, Q-X-8; propuestas sin respuesta: Q-CELL-3, Q-CELL-8, Q-MBX-11, Q-REF-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P19). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P11, P12, P15, P16, P17, P18, P20.
- **Archivos previstos:** `crates/rakka-actor/src/actor/cell.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorSpec/ActorLifeCycleSpec parciales.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P20 SUP-ESTRATEGIAS

- **Resultado verificable:** SupervisorStrategy: directivas, deciders, One/AllForOne, ventanas de reintentos.
- **Filas (10; impl: pending 10):** asignacion explicita en `docs/unit-rows.tsv`: AC-SUP-003, AC-SUP-004, AC-SUP-005, AC-SUP-006.1, AC-SUP-006.2, AC-SUP-007, AC-SUP-012, AC-SUP-013, AC-SUP-017, AC-SUP-021.
- **Exclusiones/costuras:** Pura sobre tipos; no depende de la celda (SUP-004 solo lista excepciones de ciclo de vida).
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-SUP-1, Q-X-2, Q-X-5; propuestas sin respuesta: Q-SUP-2, Q-SUP-5, Q-SUP-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P20). <!-- generated:unit-decisions -->
- **Dependencias:** P11, P14.
- **Archivos previstos:** `crates/rakka-actor/src/actor/supervision.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** SupervisorHierarchySpec parcial.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P21 SUP-FALLO

- **Resultado verificable:** Propagacion de fallos: suspender, enviar Failed al padre, aplicar la directiva, hooks de ciclo de vida.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-SUP-001, AC-SUP-002, AC-SUP-008, AC-SUP-009, AC-SUP-010, AC-SUP-016, AC-SUP-018, AC-SUP-020, AC-CELL-006, AC-CELL-020, AC-CELL-021.
- **Exclusiones/costuras:** Reinicio/parada completos en P22.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-CELL-7, Q-SUP-1, Q-SUP-3, Q-X-4, Q-X-5; propuestas sin respuesta: Q-CELL-2, Q-SUP-4, Q-SUP-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P21). <!-- generated:unit-decisions -->
- **Dependencias:** P19, P20.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{fault,lifecycle}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** SupervisorSpec parcial.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P22 CELL-REINICIO-PARADA

- **Resultado verificable:** Restart, stop, terminacion y espera de hijos como una maquina de estados unica.
- **Filas (21; impl: pending 21):** asignacion explicita en `docs/unit-rows.tsv`: AC-CELL-002.2, AC-CELL-007.1, AC-CELL-007.2, AC-CELL-007.3, AC-CELL-007.4, AC-CELL-015.2, AC-CELL-008.1, AC-CELL-008.2, AC-CELL-009, AC-CELL-010, AC-CELL-011, AC-CELL-012, AC-CELL-023, AC-CELL-024, AC-CELL-030, AC-SUP-011, AC-SUP-015, AC-SUP-019, AC-DW-008, AC-DW-009, AC-DW-010.
- **Exclusiones/costuras:** Resuelve R4/R8/R25/R26/R27; DW-008/009 se implementan aqui con el hook de Terminated (DW-001 llega en P23).
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-CELL-6, Q-SUP-3, Q-X-5; propuestas sin respuesta: Q-CELL-4, Q-CELL-8, Q-DW-2, Q-REF-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P22). <!-- generated:unit-decisions -->
- **Dependencias:** P17, P18, P21.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{restart,terminate}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorLifeCycleSpec, SupervisorSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P23 DW-VIGILANCIA

- **Resultado verificable:** Watch/unwatch, Terminated, death pact, unhandled y entrega a celda terminada.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AC-DW-001, AC-DW-002, AC-DW-003, AC-DW-004, AC-DW-005, AC-DW-006, AC-DW-007, AC-DW-011, AC-DW-012, AC-CELL-004, AC-CELL-013.3, AC-CELL-025, AC-EVT-010.
- **Exclusiones/costuras:** Resuelve R1(a): DW-002 + CELL-013.3 juntas; CELL-004 + DW-007 juntas.
- **Decisiones:** 5 distintas en sus filas; respondidas: Q-DW-3, Q-DW-4, Q-EVT-1; propuestas sin respuesta: Q-DW-1, Q-DW-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P23). <!-- generated:unit-decisions -->
- **Dependencias:** P11, P17, P20, P22.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{deathwatch,unhandled}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** DeathWatchSpec 1:1 (oraculo por comportamiento).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P24 MBX-STASH

- **Resultado verificable:** Stash (stash/unstash/unstashAll) y su interaccion con restart/stop/Terminated.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-MBX-016, AC-MBX-017, AC-MBX-018, AC-MBX-023, AC-DW-013.
- **Exclusiones/costuras:** MBX-013 (tipo de mailbox exigido) se valida en P28.
- **Decisiones:** 3 distintas en sus filas; respondidas: Q-X-1; propuestas sin respuesta: Q-MBX-1, Q-MBX-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P24). <!-- generated:unit-decisions -->
- **Dependencias:** P13, P22, P23.
- **Archivos previstos:** `crates/rakka-actor/src/actor/stash.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** StashSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 3 - sistema, proveedor, referencias y extensiones

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P25 | SYS-GUARDIANES | 12 | P10, P16, P20, P22, P23, P67 |
| P26 | SYS-PROVEEDOR | 8 | P09, P10, P16, P25 |
| P27 | SCH-ACTORES | 4 | P12, P22, P24 |
| P28 | SELECCION-CREACION | 9 | P10, P13, P16, P24, P26 |
| P29 | REF-SELECCION | 6 | P26 |
| P30 | EXT | 5 | P08, P09, P26 |
| P72 | UTIL-MANIFEST | 1 | P08, P30, P67 |

### P25 SYS-GUARDIANES

- **Resultado verificable:** ActorSystem: constructores, arranque, jerarquia de guardianes y guardian de usuario.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-001.1, AC-SYS-001.2, AC-SYS-002, AC-SYS-003.1, AC-SYS-003.2, AC-SYS-007, AC-SYS-008, AC-SYS-009, AC-SYS-013, AC-SYS-020, AC-SYS-023, AC-SUP-014.
- **Exclusiones/costuras:** Costura SYS-S1: hooks de terminacion via trait TerminationHooks (P39); registro de extensiones via ExtensionRegistry (P30). (2026-11-01) Declara P67 (AC-SYS-020 consume util.Version, 005.2); AC-SYS-020 -> AC-UTIL-016.9 (P72) es una costura hacia adelante.
- **Decisiones:** 11 distintas en sus filas; respondidas: 7 (ver detalle); propuestas sin respuesta: Q-CFG-32, Q-SYS-3, Q-SYS-4, Q-SYS-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P25). <!-- generated:unit-decisions -->
- **Dependencias:** P10, P16, P20, P22, P23, P67.
- **Archivos previstos:** `crates/rakka-actor/src/system/{system,guardians}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorSystemSpec parcial + oraculo.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P26 SYS-PROVEEDOR

- **Resultado verificable:** Proveedor local: actorOf con Deploy, systemActorOf, resolveActorRef y SPI.
- **Filas (8; impl: pending 8):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.17, AC-REF-017, AC-SYS-010, AC-SYS-012, AC-SYS-016, AC-SYS-018, AC-SYS-019, AC-SYS-024.
- **Exclusiones/costuras:** SYS-016 parte router->P37 (CFG-012.3), seleccion de dispatcher->P28. Desde 2026-11-01 incluye AC-UTIL-016.17 (constante `RemoteDeploymentConfigErrorPrefix`, consumida por AC-SYS-016), proveniente de P05.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-CFG-9, Q-EVT-1, Q-X-10, Q-X-2, Q-X-4, Q-X-5; propuestas sin respuesta: Q-SYS-5, Q-SYS-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P26). <!-- generated:unit-decisions -->
- **Dependencias:** P09, P10, P16, P25.
- **Archivos previstos:** `crates/rakka-actor/src/system/provider.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** LocalActorRefProviderSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P27 SCH-ACTORES

- **Resultado verificable:** Timers y mensajes de scheduler dirigidos a actores.
- **Filas (4; impl: pending 4):** asignacion explicita en `docs/unit-rows.tsv`: AC-SCH-007, AC-SCH-011, AC-SCH-012, AC-SCH-013.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 1 distintas en sus filas; propuestas sin respuesta: Q-SCH-3. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P27). <!-- generated:unit-decisions -->
- **Dependencias:** P12, P22, P24.
- **Archivos previstos:** `crates/rakka-actor/src/actor/timers.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** TimerSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P28 SELECCION-CREACION

- **Resultado verificable:** Seleccion de dispatcher/mailbox por actor, arranque diferido (RepointableActorRef) y arranque de hijo.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-DSP-007, AC-MBX-013, AC-MBX-014, AC-MBX-020, AC-CELL-016, AC-CELL-026, AC-CELL-027, AC-REF-020, AC-REF-003.
- **Exclusiones/costuras:** CFG-012.3/.4 (router) ->P37: aqui Deploy.router opaco.
- **Decisiones:** 10 distintas en sus filas; respondidas: Q-MBX-12, Q-MBX-13, Q-MBX-14, Q-X-1, Q-X-7; propuestas sin respuesta: Q-DSP-2, Q-MBX-1, Q-MBX-11, Q-MBX-2, Q-REF-5. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P28). <!-- generated:unit-decisions -->
- **Dependencias:** P10, P13, P16, P24, P26.
- **Archivos previstos:** `crates/rakka-actor/src/actor/{repointable,selection}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorCreationSpec, RepointableActorRefSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P29 REF-SELECCION

- **Resultado verificable:** ActorSelection, Identify/ActorIdentity y entrega con comodines.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AC-REF-011, AC-REF-012, AC-REF-013, AC-REF-014, AC-REF-021, AC-REF-026.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-X-3; propuestas sin respuesta: Q-REF-10, Q-REF-11, Q-REF-7, Q-REF-8, Q-REF-9. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P29). <!-- generated:unit-decisions -->
- **Dependencias:** P26.
- **Archivos previstos:** `crates/rakka-actor/src/actor/selection.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorSelectionSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P30 EXT

- **Resultado verificable:** Extensiones: registro, carga desde config, extensiones provistas.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-EXT-001.1, AC-EXT-001.2, AC-EXT-002, AC-EXT-003, AC-EXT-004.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-CFG-8, Q-X-2, Q-X-4; propuestas sin respuesta: Q-EXT-1, Q-EXT-2, Q-EXT-3. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P30). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P09, P26.
- **Archivos previstos:** `crates/rakka-actor/src/system/extension.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ExtensionSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P72 UTIL-MANIFEST

- **Resultado verificable:** `check_same_version(producto, dependencias, versiones) -> Option<String>` con el mensaje exacto de `ManifestInfo`, mas la extension con registro de versiones de crates (`CARGO_PKG_VERSION`), `log_warning` y `throw_exception` (`pekko.fail-mixed-versions`).
- **Filas (1; impl: pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-016.9.
- **Exclusiones/costuras:** no se leen manifiestos (ManifestInfoSpec A3 no portable). **Costura:** el contrato de 016.9 depende de AC-UTIL-005.2 (P67), AC-EXT-001 (la extension se registra con el mecanismo de AC-EXT-001.2, P30) y AC-CFG-007 (P08), por eso la unidad va despues de P30; la consume AC-SYS-020 (P25, `ActorSystem.scala:1071`), que pasa a ser dependencia hacia adelante P25 -> P72 y queda `partial` hasta integrarla. La funcion pura (A1-A2) no necesita la extension y podria validarse antes, pero eso exige dividir la fila 016.9: no se hizo (duda registrada). Q-UTIL-6 (bloqueante) afecta a la eleccion de la version mas alta.
- **Decisiones:** 3 distintas en sus filas; respondidas: Q-CFG-5, Q-UTIL-11, Q-UTIL-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P72). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P30, P67.
- **Archivos previstos:** `crates/rakka-actor/src/util/manifest_info.rs` (nuevo; nombre orientativo).
- **Tests/oraculo:** ManifestInfoSpec A1 (mensaje exacto con `[pekko-persistence, pekko-cluster]` y `(1.0.1, [pekko-cluster, pekko-persistence]), (1.0.2, [pekko-actor])`) y A2 (dynver `1.0.2` vs `1.0.2+10-abababef`); A3 no afirmada por spec.
- **Cierre:** A1-A2 verdes con evidencia RED->GREEN, cobertura >= 90 %; la fila queda `partial` mientras Q-UTIL-6, Q-CFG-5 y Q-UTIL-11 sigan sin respuesta.
- **Revision:** Contratos de las filas leidos en la particion de P05 (2026-11-01); atomicidad no acreditada; unidad sin aprobar por el Owner.

## Ola 4 - patrones, routers y logging

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P31 | ASK | 17 | P23, P26, P27, P29 |
| P32 | CIRCUIT-BREAKER-BACKOFF | 15 | P22, P27, P30, P31 |
| P33 | FSM | 11 | P22, P27, P35 |
| P34 | EVT-LOGGING | 11 | P23, P26, P30, P33 |
| P35 | RT-NUCLEO | 9 | P21, P23, P26, P31 |
| P36 | RT-ESTRATEGIAS | 9 | P13, P15, P16, P28, P35 |
| P37 | RT-CONFIG | 9 | P10, P28, P35, P36 |

### P31 ASK

- **Resultado verificable:** ask, PromiseActorRef, actores temporales, StatusReply, pipeTo, gracefulStop, after/retry.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-017, AC-SYS-025, AC-REF-027, AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3, AC-ASK-002, AC-ASK-003, AC-ASK-004, AC-ASK-005, AC-ASK-006, AC-ASK-007, AC-ASK-008, AC-ASK-009.1, AC-ASK-009.2, AC-ASK-009.3, AC-ASK-010.
- **Exclusiones/costuras:** Q-ASK-1 abierta fija la forma del Future.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-3, Q-X-4, Q-X-5; propuestas sin respuesta: Q-ASK-1, Q-ASK-10, Q-ASK-2, Q-CS-7. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P31). <!-- generated:unit-decisions -->
- **Dependencias:** P23, P26, P27, P29.
- **Archivos previstos:** `crates/rakka-actor/src/pattern/{ask,promise_ref,temp}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** AskSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P32 CIRCUIT-BREAKER-BACKOFF

- **Resultado verificable:** CircuitBreaker y BackoffSupervisor.
- **Filas (15; impl: pending 15):** asignacion explicita en `docs/unit-rows.tsv`: AC-ASK-011.1, AC-ASK-011.2, AC-ASK-011.3, AC-ASK-011.4, AC-ASK-011.5, AC-ASK-012, AC-ASK-013, AC-ASK-014, AC-ASK-015.1, AC-ASK-015.2, AC-ASK-015.3, AC-ASK-015.4, AC-ASK-015.5, AC-ASK-015.6, AC-ASK-016.
- **Exclusiones/costuras:** ASK-011.5 (API Java) a P47; ASK-013/014 consumen EXT.
- **Decisiones:** 13 distintas en sus filas; respondidas: Q-X-1, Q-X-2, Q-X-3, Q-X-4, Q-X-5; propuestas sin respuesta: 8 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P32). <!-- generated:unit-decisions -->
- **Dependencias:** P22, P27, P30, P31.
- **Archivos previstos:** `crates/rakka-actor/src/pattern/{circuit_breaker,backoff}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** CircuitBreakerSpec, BackoffSupervisorSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P33 FSM

- **Resultado verificable:** FSM: DSL, transiciones, timers, terminacion y LoggingFSM.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-FSM-001.1, AC-FSM-001.2, AC-FSM-002, AC-FSM-003, AC-FSM-004, AC-FSM-005, AC-FSM-006.1, AC-FSM-006.2, AC-FSM-007, AC-FSM-008, AC-FSM-009.
- **Exclusiones/costuras:** Q-FSM-4 bloquea FSM-006.1/.2; el resto avanza.
- **Decisiones:** 9 distintas en sus filas; respondidas: Q-FSM-4, Q-X-3, Q-X-4, Q-X-5; propuestas sin respuesta: Q-CS-7, Q-FSM-1, Q-FSM-2, Q-FSM-3, Q-RT-8. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P33). <!-- generated:unit-decisions -->
- **Dependencias:** P22, P27, P35.
- **Archivos previstos:** `crates/rakka-actor/src/pattern/fsm.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** FSMActorSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P34 EVT-LOGGING

- **Resultado verificable:** Logging completo: LoggingBus, loggers, DeadLetterListener, EventStreamUnsubscriber, LoggingReceive.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-EVT-005, AC-EVT-007, AC-EVT-009, AC-EVT-011, AC-EVT-013.3, AC-EVT-014, AC-EVT-017, AC-EVT-018, AC-EVT-019, AC-EVT-020, AC-EVT-021.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 9 distintas en sus filas; respondidas: Q-EVT-1, Q-X-2, Q-X-3, Q-X-4, Q-X-7; propuestas sin respuesta: Q-EVT-2, Q-EVT-4, Q-EVT-6, Q-EVT-7. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P34). <!-- generated:unit-decisions -->
- **Dependencias:** P23, P26, P30, P33.
- **Archivos previstos:** `crates/rakka-actor/src/event/{logging,listeners}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** LoggingSpec, EventStreamSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P35 RT-NUCLEO

- **Resultado verificable:** Router, RoutedActorCell, Pool/Group, RoundRobin/Random/Broadcast, listeners.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-RT-001, AC-RT-002, AC-RT-003.1, AC-RT-003.2, AC-RT-003.3, AC-RT-005, AC-RT-006, AC-RT-007, AC-RT-015.
- **Exclusiones/costuras:** Costura RT-S1 para ask: ROUTER-GATHER usa trait Asker hasta P31.
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-X-3, Q-X-4, Q-X-5; propuestas sin respuesta: Q-RT-1, Q-RT-2, Q-RT-8. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P35). <!-- generated:unit-decisions -->
- **Dependencias:** P21, P23, P26, P31.
- **Archivos previstos:** `crates/rakka-actor/src/routing/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** RoutingSpec parcial.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P36 RT-ESTRATEGIAS

- **Resultado verificable:** SmallestMailbox, ConsistentHashing, ScatterGather, TailChopping, BalancingPool/Dispatcher.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-RT-008, AC-RT-009.1, AC-RT-009.2, AC-RT-009.3, AC-RT-010, AC-RT-011, AC-RT-012, AC-DSP-012, AC-DSP-021.
- **Exclusiones/costuras:** Q-RT-4 (bloqueante) afecta RT-008.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-MBX-6, Q-RT-4, Q-X-3; propuestas sin respuesta: Q-ASK-1, Q-DSP-1, Q-RT-5, Q-RT-6, Q-RT-7. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P36). <!-- generated:unit-decisions -->
- **Dependencias:** P13, P15, P16, P28, P35.
- **Archivos previstos:** `crates/rakka-actor/src/routing/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** RoutingSpec, ConsistentHashingRouterSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P37 RT-CONFIG

- **Resultado verificable:** FromConfig/NoRouter, resizers, router.type-mapping y secciones de resizer en deployment.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-RT-004, AC-RT-013.1, AC-RT-013.2, AC-RT-013.3, AC-RT-014, AC-RT-016, AC-CFG-012.3, AC-CFG-012.4, AC-CFG-014.
- **Exclusiones/costuras:** Q-RT-4 afecta RT-013.*/014.
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-RT-4, Q-X-1, Q-X-2, Q-X-5; propuestas sin respuesta: Q-RT-3, Q-RT-9. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P37). <!-- generated:unit-decisions -->
- **Dependencias:** P10, P28, P35, P36.
- **Archivos previstos:** `crates/rakka-actor/src/routing/{config,resizer}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ResizerSpec, RouterConfigSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 5 - apagado, serializacion e IO

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P38 | CS-NUCLEO | 13 | P08, P26, P27, P30 |
| P39 | TERMINACION | 10 | P08, P25, P38 |
| P40 | SER-NUCLEO | 18 | P04, P08, P09, P42, P74 |
| P41 | SER-SISTEMA | 8 | P26, P29, P30, P40 |
| P42 | IO-TCP | 16 | P04, P08, P16, P26, P30 |
| P43 | IO-UDP-DNS | 19 | P08, P42 |

### P38 CS-NUCLEO

- **Resultado verificable:** CoordinatedShutdown: fases, tareas, run, timeouts, reasons y hooks de JVM.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AC-CS-001.1, AC-CS-001.2, AC-CS-002, AC-CS-003.1, AC-CS-003.2, AC-CS-004.1, AC-CS-004.2, AC-CS-004.3, AC-CS-005, AC-CS-007.1, AC-CS-007.2, AC-CS-008, AC-CS-009.
- **Exclusiones/costuras:** CS-006.* en P39.
- **Decisiones:** 10 distintas en sus filas; respondidas: Q-X-1, Q-X-4, Q-X-5; propuestas sin respuesta: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P38). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P26, P27, P30.
- **Archivos previstos:** `crates/rakka-actor/src/system/coordinated_shutdown.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** CoordinatedShutdownSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P39 TERMINACION

- **Resultado verificable:** terminate/close/whenTerminated/abort y la fase actor-system-terminate.
- **Filas (10; impl: pending 10):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-004, AC-SYS-005, AC-SYS-006, AC-SYS-011, AC-SYS-014, AC-SYS-015, AC-SYS-021, AC-SYS-022, AC-CS-006.1, AC-CS-006.2.
- **Exclusiones/costuras:** Resuelve R16/R49 (SYS-004/005/006 + CS-006).
- **Decisiones:** 9 distintas en sus filas; respondidas: Q-X-1, Q-X-2, Q-X-4, Q-X-5; propuestas sin respuesta: Q-CS-5, Q-SYS-1, Q-SYS-2, Q-SYS-3, Q-SYS-5. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P39). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P25, P38.
- **Archivos previstos:** `crates/rakka-actor/src/system/termination.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorSystemSpec, CoordinatedShutdownSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P40 SER-NUCLEO

- **Resultado verificable:** Serialization: resolucion, serializers por defecto y limites.
- **Filas (18; impl: pending 18):** asignacion explicita en `docs/unit-rows.tsv`: AC-SER-001.1, AC-SER-001.2, AC-SER-001.3, AC-SER-002, AC-SER-003.1, AC-SER-003.2, AC-SER-003.3, AC-SER-003.4, AC-SER-003.5, AC-SER-004.1, AC-SER-004.2, AC-SER-005, AC-SER-006, AC-SER-009, AC-SER-010, AC-SER-011, AC-SER-012, AC-SER-013.
- **Exclusiones/costuras:** Q-SER-1 (bloqueante): sin respuesta solo SER-002/003.1/003.4/010 pueden avanzar.
- **Decisiones:** 11 distintas en sus filas; respondidas: 7 (ver detalle); propuestas sin respuesta: Q-SER-2, Q-SER-3, Q-SER-5, Q-SER-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P40). <!-- generated:unit-decisions -->
- **Dependencias:** P04, P08, P09, P42, P74.
- **Archivos previstos:** `crates/rakka-actor/src/serialization/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** SerializeSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P41 SER-SISTEMA

- **Resultado verificable:** SerializationExtension, serialized paths, verificacion de mensajes/creadores.
- **Filas (8; impl: pending 8):** asignacion explicita en `docs/unit-rows.tsv`: AC-SER-007.1, AC-SER-007.2, AC-SER-008.1, AC-SER-008.2, AC-SER-014, AC-REF-023, AC-CELL-028, AC-CELL-029.
- **Exclusiones/costuras:** Resuelve R3 (SER-001.3 + SER-007.2 + SYS-018).
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-X-4; propuestas sin respuesta: Q-CELL-5, Q-REF-3, Q-SER-3, Q-SER-4, Q-SER-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P41). <!-- generated:unit-decisions -->
- **Dependencias:** P26, P29, P30, P40.
- **Archivos previstos:** `crates/rakka-actor/src/serialization/ext.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** SerializationSetupSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P42 IO-TCP

- **Resultado verificable:** IO: extension, Tcp y gestores de selector.
- **Filas (16; impl: pending 16):** asignacion explicita en `docs/unit-rows.tsv`: AC-IO-001, AC-IO-002.1, AC-IO-002.2, AC-IO-002.3, AC-IO-002.4, AC-IO-002.5, AC-IO-002.6, AC-IO-002.7, AC-IO-003, AC-IO-004.1, AC-IO-004.2, AC-IO-004.3, AC-IO-004.4, AC-IO-010, AC-IO-012, AC-IO-013.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-1, Q-X-3, Q-X-5; propuestas sin respuesta: Q-IO-1, Q-IO-2, Q-IO-3, Q-IO-4. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P42). <!-- generated:unit-decisions -->
- **Dependencias:** P04, P08, P16, P26, P30.
- **Archivos previstos:** `crates/rakka-actor/src/io/tcp/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** TcpIntegrationSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P43 IO-UDP-DNS

- **Resultado verificable:** IO: Udp, UdpConnected, DNS (inet y async).
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AC-IO-005, AC-IO-006.1, AC-IO-006.2, AC-IO-006.3, AC-IO-007, AC-IO-008.1, AC-IO-008.2, AC-IO-008.3, AC-IO-008.4, AC-IO-008.5, AC-IO-009.1, AC-IO-009.2, AC-IO-009.3, AC-IO-009.4, AC-IO-009.5, AC-IO-009.6, AC-IO-009.7, AC-IO-009.8, AC-IO-011.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-1, Q-X-2; propuestas sin respuesta: Q-IO-1, Q-IO-3, Q-IO-4, Q-IO-5, Q-IO-6. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P43). <!-- generated:unit-decisions -->
- **Dependencias:** P08, P42.
- **Archivos previstos:** `crates/rakka-actor/src/io/{udp,dns}/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** UdpIntegrationSpec, AsyncDnsResolverSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 6 - testkit Classic y superficie Java

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P44 | TK-DISPATCHER-REF | 5 | P15, P22, P26, P28 |
| P45 | TK-API | 22 | P11, P23, P27, P34, P39, P44, P46 |
| P46 | JAPI-ACTORES | 16 | P20, P22, P28, P29, P33, P39, P40, P77 |
| P47 | JAPI-EXTRA | 7 | P32, P34, P40, P42, P43, P46 |

### P44 TK-DISPATCHER-REF

- **Resultado verificable:** TestActorRef y CallingThreadDispatcher.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-TK-001.1, AC-TK-006.1, AC-TK-006.2, AC-TK-008, AC-TK-016.
- **Exclusiones/costuras:** Q-TK-1 (bloqueante).
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-TK-1, Q-X-3, Q-X-4, Q-X-5; propuestas sin respuesta: Q-TK-2, Q-TK-4, Q-TK-7, Q-TK-8. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P44). <!-- generated:unit-decisions -->
- **Dependencias:** P15, P22, P26, P28.
- **Archivos previstos:** `testkit/src/{test_actor_ref,calling_thread}.rs` relativo a un crate de testkit **que no existe y cuya ubicacion no esta decidida** (hoy solo existen `rakka-actor` y `rakka-actor-typed`; ver propuesta pendiente de ubicacion de los testkits).
- **Tests/oraculo:** CallingThreadDispatcherSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P45 TK-API

- **Resultado verificable:** TestKit/TestProbe, EventFilter, TestFSMRef, utilidades y SocketUtil.
- **Filas (22; impl: pending 22):** asignacion explicita en `docs/unit-rows.tsv`: AC-TK-001.2, AC-TK-001.3, AC-TK-001.4, AC-TK-001.5, AC-TK-002.1, AC-TK-002.2, AC-TK-002.3, AC-TK-003, AC-TK-004, AC-TK-005.1, AC-TK-005.2, AC-TK-005.3, AC-TK-007, AC-TK-009, AC-TK-010, AC-TK-011, AC-TK-012.1, AC-TK-012.2, AC-TK-013, AC-TK-014, AC-TK-015, AC-TK-017.
- **Exclusiones/costuras:** TK-011 (javadsl) tras P46.
- **Decisiones:** 14 distintas en sus filas; respondidas: Q-TK-1, Q-X-2, Q-X-3, Q-X-5, Q-X-7; propuestas sin respuesta: 9 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P45). <!-- generated:unit-decisions -->
- **Dependencias:** P11, P23, P27, P34, P39, P44, P46.
- **Archivos previstos:** `testkit/src/*.rs` relativo a un crate de testkit **que no existe y cuya ubicacion no esta decidida** (hoy solo existen `rakka-actor` y `rakka-actor-typed`; ver propuesta pendiente de ubicacion de los testkits).
- **Tests/oraculo:** TestKitSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P46 JAPI-ACTORES

- **Resultado verificable:** Superficie Java-facing de actores: AbstractActor, AbstractFSM, japi.pf/function, Props Java.
- **Filas (16; impl: pending 16):** asignacion explicita en `docs/unit-rows.tsv`: AC-JAPI-001, AC-JAPI-002, AC-JAPI-003, AC-JAPI-004.1, AC-JAPI-004.2, AC-JAPI-004.3, AC-JAPI-004.4, AC-JAPI-005.1, AC-JAPI-005.2, AC-JAPI-005.3, AC-JAPI-005.4, AC-JAPI-013, AC-JAPI-006, AC-JAPI-008, AC-JAPI-011, AC-JAPI-012.
- **Exclusiones/costuras:** Q-JAPI-1..4 deciden si se porta. (2026-11-01) Declara P77 (AC-JAPI-001/008 consumen 018.1/018.2): antes la cubria P05 por cierre transitivo.
- **Decisiones:** 4 distintas en sus filas; respondidas: Q-X-5; propuestas sin respuesta: Q-JAPI-1, Q-JAPI-2, Q-JAPI-4. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P46). <!-- generated:unit-decisions -->
- **Dependencias:** P20, P22, P28, P29, P33, P39, P40, P77.
- **Archivos previstos:** `crates/rakka-actor/src/japi/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs Java de Pekko.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P47 JAPI-EXTRA

- **Resultado verificable:** EventBus Java, TypedActor clasico, fabricas IO Java, API Java de patrones.
- **Filas (7; impl: pending 7):** asignacion explicita en `docs/unit-rows.tsv`: AC-JAPI-007, AC-JAPI-009.1, AC-JAPI-009.2, AC-JAPI-009.3, AC-JAPI-010, AC-EVT-022, AC-ASK-017.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-X-5; propuestas sin respuesta: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P47). <!-- generated:unit-decisions -->
- **Dependencias:** P32, P34, P40, P42, P43, P46.
- **Archivos previstos:** `crates/rakka-actor/src/japi/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs Java.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 7 - Typed: nucleo, adaptador y sistema

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P48 | T-COMPORTAMIENTO-NUCLEO | 21 | P22 |
| P49 | T-ADAPTADOR-CLASICO | 19 | P23, P26, P30, P48 |
| P50 | T-SISTEMA-TIPADO | 25 | P30, P39, P49, P53 |
| P51 | T-ASK-TIPADO | 5 | P31, P50, P52 |
| P52 | T-OBSERVABILIDAD | 14 | P34, P49, P50, P53 |
| P53 | T-INTERCEPTORES-TIMERS-STASH | 21 | P24, P27, P49 |

### P48 T-COMPORTAMIENTO-NUCLEO

- **Resultado verificable:** Typed: Behavior, interprete, receive/setup, ActorRef tipado, senales basicas.
- **Filas (21; impl: pending 21):** asignacion explicita en `docs/unit-rows.tsv`: AT-BEH-001, AT-BEH-002, AT-BEH-004, AT-BEH-005, AT-BEH-006, AT-BEH-007, AT-BEH-008.1, AT-BEH-008.2, AT-BEH-008.3, AT-BEH-009, AT-BEH-011, AT-BEH-012, AT-BEH-022, AT-BEH-023, AT-SIG-001, AT-SIG-003, AT-REF-001, AT-REF-002, AT-REF-003, AT-REF-005, AT-REF-006.
- **Exclusiones/costuras:** Q-T-BEH-1 (bloqueante) fija la forma; Costura BEH-S1: Behavior sin supervision/timers/stash, que llegan en P53/P55.
- **Decisiones:** 6 distintas en sus filas; respondidas: Q-T-BEH-1, Q-X-3, Q-X-5; propuestas sin respuesta: Q-T-BEH-2, Q-T-BEH-4, Q-T-SIG-1. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P48). <!-- generated:unit-decisions -->
- **Dependencias:** P22.
- **Archivos previstos:** `crates/rakka-actor-typed/src/{behavior,interpreter,actor_ref}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** BehaviorSpec 1:1.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P49 T-ADAPTADOR-CLASICO

- **Resultado verificable:** Adaptador typed sobre celda clasica (ActorAdapter, ActorSystemAdapter) y contexto basico.
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AT-ADP-001, AT-ADP-003, AT-ADP-004, AT-ADP-006.1, AT-ADP-006.2, AT-ADP-006.3, AT-ADP-006.4, AT-ADP-007, AT-ADP-008, AT-ADP-009, AT-ADP-010, AT-EXT-002.3, AT-SYS-011, AT-CTX-001, AT-CTX-002, AT-CTX-003, AT-CTX-004, AT-CTX-016, AT-CTX-018.
- **Exclusiones/costuras:** Q-X-8 (proposed): asume runtime sobre Classic; si el Owner elige nativo, esta unidad y P50 se rehacen.
- **Decisiones:** 9 distintas en sus filas; respondidas: Q-X-3, Q-X-8; propuestas sin respuesta: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P49). <!-- generated:unit-decisions -->
- **Dependencias:** P23, P26, P30, P48.
- **Archivos previstos:** `crates/rakka-actor-typed/src/adapter/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorContextSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P50 T-SISTEMA-TIPADO

- **Resultado verificable:** ActorSystem tipado, guardian, Props/Dispatcher/Mailbox selectors, extensiones y CS.
- **Filas (25; impl: pending 25):** asignacion explicita en `docs/unit-rows.tsv`: AT-SYS-001, AT-SYS-002, AT-SYS-003, AT-SYS-004, AT-SYS-005, AT-SYS-006, AT-SYS-007.1, AT-SYS-007.2, AT-SYS-007.3, AT-SYS-007.4, AT-SYS-008, AT-SYS-009, AT-SYS-010, AT-SYS-012, AT-CS-001, AT-DSP-001, AT-DSP-002, AT-DSP-003, AT-DSP-005, AT-EXT-001.1, AT-EXT-001.2, AT-EXT-001.3, AT-EXT-002.1, AT-EXT-002.2, AT-EXT-003.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 16 distintas en sus filas; respondidas: Q-X-1, Q-X-2, Q-X-4, Q-X-6, Q-X-8; propuestas sin respuesta: 11 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P50). <!-- generated:unit-decisions -->
- **Dependencias:** P30, P39, P49, P53.
- **Archivos previstos:** `crates/rakka-actor-typed/src/{system,props,extension}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorSystemSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P51 T-ASK-TIPADO

- **Resultado verificable:** ask tipado, StatusReply, RecipientRef.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AT-ASK-001, AT-ASK-002, AT-ASK-003, AT-ASK-004, AT-REF-004.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 4 distintas en sus filas; respondidas: Q-X-3, Q-X-8; propuestas sin respuesta: Q-T-ASK-1, Q-T-ASK-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P51). <!-- generated:unit-decisions -->
- **Dependencias:** P31, P50, P52.
- **Archivos previstos:** `crates/rakka-actor-typed/src/ask.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** AskSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P52 T-OBSERVABILIDAD

- **Resultado verificable:** EventStream tipado, logging (MDC), withMdc/monitor/logMessages, unhandled.
- **Filas (14; impl: pending 14):** asignacion explicita en `docs/unit-rows.tsv`: AT-EVT-001, AT-EVT-002, AT-EVT-003, AT-EVT-004, AT-LOG-001, AT-LOG-002, AT-LOG-003, AT-LOG-004, AT-LOG-005, AT-BEH-003, AT-BEH-017, AT-BEH-018, AT-BEH-019, AT-DSP-004.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 11 distintas en sus filas; respondidas: Q-X-6, Q-X-7; propuestas sin respuesta: 9 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P52). <!-- generated:unit-decisions -->
- **Dependencias:** P34, P49, P50, P53.
- **Archivos previstos:** `crates/rakka-actor-typed/src/{eventstream,logging}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** LoggingSpec, MDCSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P53 T-INTERCEPTORES-TIMERS-STASH

- **Resultado verificable:** Interceptores, timers y stash tipados.
- **Filas (21; impl: pending 21):** asignacion explicita en `docs/unit-rows.tsv`: AT-BEH-013, AT-BEH-014, AT-BEH-015, AT-BEH-016, AT-BEH-020, AT-BEH-021, AT-SIG-008, AT-TMR-001, AT-TMR-002, AT-TMR-003, AT-TMR-004, AT-TMR-005, AT-TMR-006, AT-TMR-007.1, AT-TMR-007.2, AT-TMR-008, AT-STASH-001, AT-STASH-002, AT-STASH-003, AT-STASH-004, AT-STASH-005.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 10 distintas en sus filas; respondidas: Q-X-3, Q-X-5, Q-X-6; propuestas sin respuesta: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P53). <!-- generated:unit-decisions -->
- **Dependencias:** P24, P27, P49.
- **Archivos previstos:** `crates/rakka-actor-typed/src/{intercept,timers,stash}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** InterceptSpec, TimerSpec, StashSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 8 - Typed: contexto, supervision, receptionist y entrega fiable

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P54 | T-CONTEXTO-SENALES | 14 | P49, P51, P53 |
| P55 | T-SUPERVISION | 31 | P21, P49, P52, P53, P54 |
| P56 | T-RECEPCIONISTA-SERIALIZACION | 14 | P30, P41, P50, P54 |
| P57 | T-ROUTERS-PUBSUB-SPAWN | 12 | P54, P55, P56 |
| P58 | T-ENTREGA-FIABLE-BASE | 13 | P53, P54, P56 |
| P59 | T-ENTREGA-FIABLE-DURABLE | 6 | P58 |
| P60 | T-ENTREGA-FIABLE-WORKPULLING | 6 | P56, P59 |

### P54 T-CONTEXTO-SENALES

- **Resultado verificable:** Contexto tipado completo (watch, receive timeout, adapters, pipeToSelf) y senales ligadas.
- **Filas (14; impl: pending 14):** asignacion explicita en `docs/unit-rows.tsv`: AT-CTX-005, AT-CTX-006, AT-CTX-007, AT-CTX-008, AT-CTX-009, AT-CTX-010, AT-CTX-011, AT-CTX-012, AT-CTX-013, AT-CTX-014, AT-CTX-015, AT-SIG-004, AT-SIG-007, AT-SUP-015.
- **Exclusiones/costuras:** Resuelve R19/R64.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-X-3, Q-X-4, Q-X-5, Q-X-6; propuestas sin respuesta: Q-T-CTX-1, Q-T-CTX-3, Q-T-SIG-1, Q-T-TMR-2. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P54). <!-- generated:unit-decisions -->
- **Dependencias:** P49, P51, P53.
- **Archivos previstos:** `crates/rakka-actor-typed/src/context.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ActorContextSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P55 T-SUPERVISION

- **Resultado verificable:** Supervision tipada: estrategias, backoff, stash de restart, interop clasica.
- **Filas (31; impl: pending 31):** asignacion explicita en `docs/unit-rows.tsv`: AT-SUP-001, AT-SUP-002, AT-SUP-003, AT-SUP-004, AT-SUP-005, AT-SUP-006, AT-SUP-007.1, AT-SUP-007.2, AT-SUP-007.3, AT-SUP-008, AT-SUP-009, AT-SUP-010, AT-SUP-011, AT-SUP-012, AT-SUP-013, AT-SUP-014, AT-SUP-016, AT-SUP-017, AT-SUP-018, AT-SUP-019, AT-SUP-020, AT-SUP-021, AT-SUP-022, AT-SIG-002, AT-SIG-005, AT-SIG-006, AT-STASH-006, AT-BEH-010, AT-BEH-024, AT-ADP-002, AT-ADP-005.
- **Exclusiones/costuras:** Q-T-SUP-3 (bloqueante) afecta SUP-007.2/.3.
- **Decisiones:** 20 distintas en sus filas; respondidas: Q-T-BEH-1, Q-T-SUP-3, Q-X-5, Q-X-6, Q-X-7, Q-X-8; propuestas sin respuesta: 14 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P55). <!-- generated:unit-decisions -->
- **Dependencias:** P21, P49, P52, P53, P54.
- **Archivos previstos:** `crates/rakka-actor-typed/src/supervision.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** SupervisionSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P56 T-RECEPCIONISTA-SERIALIZACION

- **Resultado verificable:** Receptionist y serializadores tipados.
- **Filas (14; impl: pending 14):** asignacion explicita en `docs/unit-rows.tsv`: AT-RCP-001, AT-RCP-002, AT-RCP-003, AT-RCP-004, AT-RCP-005.1, AT-RCP-005.2, AT-RCP-005.3, AT-RCP-006, AT-RCP-007, AT-RCP-008, AT-SER-001, AT-SER-002, AT-SER-003, AT-REF-007.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-2; propuestas sin respuesta: Q-T-RCP-1, Q-T-RCP-2, Q-T-RCP-3, Q-T-SER-1, Q-T-SER-2, Q-T-SER-3. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P56). <!-- generated:unit-decisions -->
- **Dependencias:** P30, P41, P50, P54.
- **Archivos previstos:** `crates/rakka-actor-typed/src/{receptionist,serializer}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ReceptionistSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P57 T-ROUTERS-PUBSUB-SPAWN

- **Resultado verificable:** Routers tipados, pub-sub local y SpawnProtocol.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AT-RT-001, AT-RT-002.1, AT-RT-002.2, AT-RT-002.3, AT-RT-003, AT-RT-004, AT-RT-005, AT-RT-006, AT-PS-001.1, AT-PS-001.2, AT-PS-001.3, AT-SPV-001.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 5 distintas en sus filas; propuestas sin respuesta: Q-T-EVT-1, Q-T-RT-1, Q-T-RT-2, Q-T-RT-3, Q-T-SPV-1. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P57). <!-- generated:unit-decisions -->
- **Dependencias:** P54, P55, P56.
- **Archivos previstos:** `crates/rakka-actor-typed/src/{routing,pubsub,spawn_protocol}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** RoutersSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P58 T-ENTREGA-FIABLE-BASE

- **Resultado verificable:** Reliable delivery: Producer/ConsumerController, control de flujo y reenvio.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AT-DLV-006, AT-DLV-001.1, AT-DLV-001.2, AT-DLV-001.3, AT-DLV-001.4, AT-DLV-001.5, AT-DLV-002.1, AT-DLV-002.2, AT-DLV-002.3, AT-DLV-002.4, AT-DLV-002.5, AT-DLV-002.6, AT-DLV-002.7.
- **Exclusiones/costuras:** Resuelve R2 parte 1: DLV-001.6 (durable) se difiere a P59 con interfaz DurableQueue.
- **Decisiones:** 8 distintas en sus filas; respondidas: Q-UTIL-1, Q-X-1, Q-X-10, Q-X-5, Q-X-8; propuestas sin respuesta: Q-T-DLV-1, Q-T-DLV-2, Q-T-DLV-4. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P58). <!-- generated:unit-decisions -->
- **Dependencias:** P53, P54, P56.
- **Archivos previstos:** `crates/rakka-actor-typed/src/delivery/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** ReliableDeliverySpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P59 T-ENTREGA-FIABLE-DURABLE

- **Resultado verificable:** Cola duradera y chunking.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AT-DLV-001.6, AT-DLV-004, AT-DLV-005.1, AT-DLV-005.2, AT-DLV-005.3, AT-DLV-007.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 5 distintas en sus filas; respondidas: Q-X-10; propuestas sin respuesta: Q-T-DLV-1, Q-T-DLV-2, Q-T-DLV-3, Q-T-SER-1. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P59). <!-- generated:unit-decisions -->
- **Dependencias:** P58.
- **Archivos previstos:** `crates/rakka-actor-typed/src/delivery/{durable,chunking}.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** DurableProducerQueueSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P60 T-ENTREGA-FIABLE-WORKPULLING

- **Resultado verificable:** WorkPullingProducerController.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AT-DLV-003.1, AT-DLV-003.2, AT-DLV-003.3, AT-DLV-003.4, AT-DLV-003.5, AT-DLV-003.6.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 4 distintas en sus filas; propuestas sin respuesta: Q-T-DLV-1, Q-T-DLV-2, Q-T-DLV-4, Q-T-RCP-1. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P60). <!-- generated:unit-decisions -->
- **Dependencias:** P56, P59.
- **Archivos previstos:** `crates/rakka-actor-typed/src/delivery/work_pulling.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** WorkPullingProducerControllerSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 9 - Typed: javadsl y testkits

| ID | Unidad | Filas | Depende de |
|---|---|---|---|
| P61 | T-JAVADSL | 17 | P48, P49, P51, P53, P54, P55, P57 |
| P62 | T-TK-SONDA | 17 | P40, P44, P50, P52, P54, P56 |
| P63 | T-TK-COMPORTAMIENTO | 19 | P53, P54, P55, P56, P62 |
| P64 | T-TK-LOGGING | 6 | P45, P52, P62 |

### P61 T-JAVADSL

- **Resultado verificable:** javadsl tipado.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AT-JAPI-001.1, AT-JAPI-001.2, AT-JAPI-001.3, AT-JAPI-002.1, AT-JAPI-002.2, AT-JAPI-003, AT-JAPI-004, AT-JAPI-005.1, AT-JAPI-005.2, AT-JAPI-005.3, AT-JAPI-005.4, AT-JAPI-006.1, AT-JAPI-006.2, AT-JAPI-006.3, AT-JAPI-006.4, AT-JAPI-006.5, AT-CTX-017.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-8; propuestas sin respuesta: Q-T-BEH-6, Q-T-CTX-2, Q-T-CTX-4, Q-T-JAPI-1, Q-T-JAPI-2, Q-T-TMR-3. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P61). <!-- generated:unit-decisions -->
- **Dependencias:** P48, P49, P51, P53, P54, P55, P57.
- **Archivos previstos:** `crates/rakka-actor-typed/src/javadsl/*.rs` (nuevos; nombres orientativos).
- **Tests/oraculo:** specs Java.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P62 T-TK-SONDA

- **Resultado verificable:** ActorTestKit, TestProbe, ManualTime, adaptadores de test.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AT-TK-001.1, AT-TK-001.2, AT-TK-001.3, AT-TK-002.1, AT-TK-002.2, AT-TK-002.3, AT-TK-002.4, AT-TK-002.5, AT-TK-002.6, AT-TK-007, AT-TK-008, AT-TK-009.1, AT-TK-009.2, AT-TK-009.3, AT-TK-009.4, AT-TK-010, AT-TK-014.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** 7 distintas en sus filas; respondidas: Q-X-1, Q-X-4; propuestas sin respuesta: Q-T-EVT-1, Q-T-SER-1, Q-T-TK-1, Q-T-TK-3, Q-T-TMR-4. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P62). <!-- generated:unit-decisions -->
- **Dependencias:** P40, P44, P50, P52, P54, P56.
- **Archivos previstos:** `testkit-typed/src/*.rs` relativo a un crate de testkit **que no existe y cuya ubicacion no esta decidida** (hoy solo existen `rakka-actor` y `rakka-actor-typed`; ver propuesta pendiente de ubicacion de los testkits).
- **Tests/oraculo:** ActorTestKitSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P63 T-TK-COMPORTAMIENTO

- **Resultado verificable:** BehaviorTestKit, Effects, inboxes, ActorSystemStub.
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AT-TK-003.1, AT-TK-003.2, AT-TK-003.3, AT-TK-003.4, AT-TK-003.5, AT-TK-003.6, AT-TK-003.7, AT-TK-004.1, AT-TK-004.2, AT-TK-004.3, AT-TK-004.4, AT-TK-004.5, AT-TK-004.6, AT-TK-004.7, AT-TK-005.1, AT-TK-005.2, AT-TK-005.3, AT-TK-011, AT-TK-012.
- **Exclusiones/costuras:** Resuelve R5/R7/R24 como una sola unidad.
- **Decisiones:** 9 distintas en sus filas; respondidas: Q-X-7, Q-X-8; propuestas sin respuesta: 7 (ver detalle). Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P63). <!-- generated:unit-decisions -->
- **Dependencias:** P53, P54, P55, P56, P62.
- **Archivos previstos:** `testkit-typed/src/behavior_testkit/*.rs` relativo a un crate de testkit **que no existe y cuya ubicacion no esta decidida** (hoy solo existen `rakka-actor` y `rakka-actor-typed`; ver propuesta pendiente de ubicacion de los testkits).
- **Tests/oraculo:** BehaviorTestKitSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P64 T-TK-LOGGING

- **Resultado verificable:** LoggingTestKit y captura de logs.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AT-TK-006.1, AT-TK-006.2, AT-TK-006.3, AT-TK-006.4, AT-TK-006.5, AT-TK-013.
- **Exclusiones/costuras:** Resuelve R12/R14.
- **Decisiones:** 4 distintas en sus filas; respondidas: Q-X-7; propuestas sin respuesta: Q-T-LOG-1, Q-T-TK-2, Q-T-TK-3. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección P64). <!-- generated:unit-decisions -->
- **Dependencias:** P45, P52, P62.
- **Archivos previstos:** `testkit-typed/src/logging.rs` relativo a un crate de testkit **que no existe y cuya ubicacion no esta decidida** (hoy solo existen `rakka-actor` y `rakka-actor-typed`; ver propuesta pendiente de ubicacion de los testkits).
- **Tests/oraculo:** LoggingTestKitSpec.
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Verificacion de cobertura de filas

Script de solo lectura (`python3 -I`), descrito y ejecutado: lee `docs/unit-rows.tsv` y comprueba que cada `row_id` aparece en exactamente una unidad.

```python
import csv,collections
rows=[r[0] for r in list(csv.reader(open("docs/contract-map.tsv"),delimiter="\t"))[1:]]
# unidades: lista (id, filas) tal como aparece en este documento, con rangos A..B segun el orden del TSV
c=collections.Counter(r for u in UNIDADES for r in expandir(u))
assert [r for r in rows if c[r]!=1]==[] and len(c)==len(rows)==783
```

Resultado de la ejecucion al redactar: 783 filas del mapa, 783 cubiertas, 0 sin unidad, 0 en mas de una unidad. La asignacion es explicita (sin rangos) en `docs/unit-rows.tsv`; `python3 -I tools/package/check.py` verifica que cada `row_id` aparece una vez, que el recuento por unidad coincide con la ficha y que no hay IDs inexistentes.

## Anexo: dependencias de fila que quedan hacia adelante (costuras)

<!-- BEGIN GENERATED SEAMS -->

Despues de ordenar las unidades (bloque de orden de ejecucion), 150 aristas `depends_on` fila->fila apuntan a una unidad posterior. Ninguna crea un ciclo entre unidades (el grafo de unidades es aciclico, verificado). Cada par se resuelve segun `docs/cycles.md` (causa: referencia documental/consumidor, dependencia de validacion o interfaz temporal). Bloque **generado** por `tools/package/unit_plan.py` desde `depends_on` de los TSV del inventario y `docs/unit-rows.tsv`; pares unidad->unidad con hasta 3 ejemplos:

| Unidad | Necesita (posterior) | n | Ejemplos |
|---|---|---|---|
| P06 | P04 | 1 | AC-UTIL-028>AC-UTIL-022 |
| P06 | P12 | 4 | AC-UTIL-026>AC-SCH-008.1, AC-UTIL-026>AC-SCH-008.2, AC-UTIL-026>AC-SCH-008.3 |
| P06 | P42 | 1 | AC-UTIL-026>AC-IO-003 |
| P03 | P10 | 2 | AC-UTIL-009>AC-CFG-012.1, AC-UTIL-009>AC-CFG-012.2 |
| P03 | P11 | 1 | AC-UTIL-008>AC-EVT-003 |
| P03 | P13 | 2 | AC-UTIL-012>AC-MBX-009, AC-UTIL-013>AC-MBX-010 |
| P03 | P37 | 2 | AC-UTIL-009>AC-CFG-012.3, AC-UTIL-009>AC-CFG-012.4 |
| P04 | P40 | 2 | AC-UTIL-006.8>AC-SER-004.1, AC-UTIL-006.8>AC-SER-004.2 |
| P07 | P08 | 2 | AC-CFG-003>AC-CFG-002, AC-CFG-019>AC-CFG-002 |
| P08 | P09 | 1 | AC-CFG-018>AC-CFG-016 |
| P08 | P25 | 1 | AC-CFG-009>AC-SUP-014 |
| P08 | P28 | 1 | AC-CFG-009>AC-REF-020 |
| P08 | P30 | 1 | AC-CFG-006>AC-EXT-003 |
| P08 | P34 | 3 | AC-CFG-004>AC-EVT-011, AC-CFG-005>AC-EVT-009, AC-CFG-011>AC-EVT-019 |
| P08 | P38 | 2 | AC-CFG-007>AC-CS-007.1, AC-CFG-007>AC-CS-007.2 |
| P08 | P39 | 3 | AC-CFG-007>AC-SYS-015, AC-CFG-020>AC-CS-006.1, AC-CFG-020>AC-CS-006.2 |
| P08 | P40 | 3 | AC-CFG-010>AC-SER-001.1, AC-CFG-010>AC-SER-001.2, AC-CFG-010>AC-SER-001.3 |
| P09 | P26 | 1 | AC-CFG-008>AC-SYS-018 |
| P10 | P37 | 2 | AC-CFG-013>AC-CFG-012.3, AC-CFG-013>AC-CFG-012.4 |
| P11 | P17 | 1 | AC-EVT-008>AC-REF-016 |
| P11 | P34 | 2 | AC-EVT-006>AC-EVT-007, AC-EVT-012>AC-EVT-011 |
| P12 | P15 | 1 | AC-SCH-009>AC-DSP-001 |
| P12 | P16 | 2 | AC-SCH-001>AC-DSP-016.1, AC-SCH-001>AC-DSP-016.2 |
| P13 | P28 | 2 | AC-MBX-015>AC-MBX-013, AC-MBX-022>AC-MBX-014 |
| P13 | P36 | 1 | AC-MBX-019>AC-RT-008 |
| P14 | P15 | 2 | AC-REF-002>AC-MBX-001, AC-REF-018>AC-CELL-013.1 |
| P14 | P19 | 2 | AC-REF-005>AC-CELL-003, AC-REF-006>AC-CELL-002.1 |
| P14 | P22 | 1 | AC-REF-006>AC-CELL-002.2 |
| P14 | P26 | 1 | AC-REF-025>AC-SYS-012 |
| P15 | P22 | 1 | AC-DSP-023>AC-CELL-007.1 |
| P16 | P34 | 1 | AC-DSP-016.1>AC-EVT-011 |
| P17 | P19 | 1 | AC-MBX-003>AC-CELL-014 |
| P17 | P22 | 1 | AC-MBX-003>AC-CELL-008.2 |
| P17 | P23 | 1 | AC-MBX-003>AC-CELL-013.3 |
| P18 | P22 | 1 | AC-REF-019>AC-CELL-008.2 |
| P19 | P21 | 1 | AC-CELL-022>AC-CELL-006 |
| P19 | P22 | 3 | AC-CELL-013.2>AC-CELL-007.2, AC-CELL-013.2>AC-CELL-008.1, AC-CELL-031>AC-CELL-030 |
| P19 | P23 | 1 | AC-CELL-031>AC-CELL-013.3 |
| P19 | P28 | 1 | AC-CELL-031>AC-REF-020 |
| P21 | P22 | 3 | AC-CELL-006>AC-CELL-007.1, AC-CELL-020>AC-CELL-007.1, AC-SUP-020>AC-DW-008 |
| P22 | P23 | 1 | AC-DW-010>AC-DW-001 |
| P23 | P30 | 2 | AC-DW-012>AC-EXT-001.1, AC-DW-012>AC-EXT-001.2 |
| P24 | P28 | 1 | AC-MBX-018>AC-MBX-013 |
| P25 | P26 | 4 | AC-SYS-001.2>AC-SYS-018, AC-SYS-002>AC-SYS-010, AC-SYS-013>AC-SYS-010 |
| P25 | P30 | 4 | AC-SYS-002>AC-EXT-003, AC-SYS-013>AC-EXT-001.2, AC-SYS-020>AC-EXT-001.2 |
| P25 | P72 | 1 | AC-SYS-020>AC-UTIL-016.9 |
| P25 | P34 | 4 | AC-SYS-002>AC-EVT-007, AC-SYS-002>AC-EVT-009, AC-SYS-002>AC-EVT-011 |
| P25 | P39 | 1 | AC-SYS-007>AC-SYS-004 |
| P26 | P28 | 2 | AC-SYS-016>AC-DSP-007, AC-SYS-016>AC-MBX-013 |
| P26 | P29 | 1 | AC-SYS-024>AC-REF-012 |
| P26 | P31 | 2 | AC-SYS-018>AC-SYS-017, AC-SYS-018>AC-SYS-025 |
| P26 | P35 | 1 | AC-SYS-016>AC-RT-003.1 |
| P26 | P39 | 2 | AC-SYS-012>AC-SYS-022, AC-SYS-019>AC-SYS-005 |
| P26 | P40 | 1 | AC-SYS-018>AC-SER-001.3 |
| P26 | P44 | 1 | AC-SYS-010>AC-TK-001.1 |
| P28 | P37 | 2 | AC-DSP-007>AC-CFG-012.3, AC-DSP-007>AC-CFG-012.4 |
| P29 | P31 | 3 | AC-REF-012>AC-ASK-001.1, AC-REF-012>AC-ASK-001.2, AC-REF-012>AC-ASK-001.3 |
| P32 | P47 | 1 | AC-ASK-011.5>AC-ASK-017 |
| P34 | P45 | 1 | AC-EVT-009>AC-TK-005.2 |
| P36 | P40 | 3 | AC-RT-009.1>AC-SER-001.1, AC-RT-009.1>AC-SER-001.2, AC-RT-009.1>AC-SER-001.3 |
| P42 | P43 | 1 | AC-IO-002.3>AC-IO-008.1 |
| P40 | P41 | 3 | AC-SER-001.2>AC-SER-014, AC-SER-001.3>AC-SER-007.2, AC-SER-013>AC-REF-023 |
| P48 | P52 | 2 | AT-BEH-005>AT-BEH-003, AT-BEH-008.2>AT-BEH-003 |
| P48 | P54 | 1 | AT-BEH-008.1>AT-SIG-007 |
| P48 | P55 | 1 | AT-BEH-007>AT-SUP-001 |
| P48 | P56 | 2 | AT-REF-001>AT-SER-001, AT-REF-001>AT-SER-003 |
| P49 | P53 | 1 | AT-CTX-018>AT-STASH-005 |
| P49 | P50 | 6 | AT-ADP-003>AT-DSP-003, AT-ADP-010>AT-CS-001, AT-ADP-010>AT-SYS-003 |
| P49 | P52 | 2 | AT-ADP-006.2>AT-BEH-003, AT-ADP-006.2>AT-EVT-003 |
| P49 | P54 | 4 | AT-ADP-006.1>AT-CTX-008, AT-ADP-006.1>AT-CTX-011, AT-ADP-006.1>AT-CTX-014 |
| P49 | P55 | 4 | AT-ADP-006.2>AT-BEH-024, AT-ADP-006.3>AT-SUP-005, AT-ADP-006.4>AT-SUP-001 |
| P53 | P52 | 2 | AT-STASH-005>AT-EVT-002, AT-STASH-005>AT-EVT-003 |
| P53 | P54 | 1 | AT-TMR-007.2>AT-CTX-008 |
| P50 | P52 | 2 | AT-SYS-007.3>AT-LOG-001, AT-SYS-010>AT-EVT-002 |
| P50 | P63 | 1 | AT-DSP-005>AT-TK-011 |
| P52 | P55 | 1 | AT-BEH-003>AT-SIG-006 |
| P52 | P64 | 5 | AT-LOG-004>AT-TK-006.1, AT-LOG-004>AT-TK-006.2, AT-LOG-004>AT-TK-006.3 |
| P56 | P57 | 1 | AT-RCP-008>AT-RT-004 |
| P58 | P59 | 5 | AT-DLV-001.1>AT-DLV-001.6, AT-DLV-001.2>AT-DLV-001.6, AT-DLV-001.5>AT-DLV-001.6 |
| P58 | P60 | 1 | AT-DLV-006>AT-DLV-003.2 |
| P59 | P60 | 1 | AT-DLV-004>AT-DLV-003.6 |

<!-- END GENERATED SEAMS -->
