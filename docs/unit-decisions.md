# Decisiones por unidad

**Generado** por `tools/package/unit_plan.py` desde `docs/contract-map.tsv` (columna `decisions` de cada fila), `docs/unit-rows.tsv` y `docs/decisions/register.tsv`. No editar a mano. El estado es el del registro; este documento no registra ni infiere aprobaciones.

## Cómo leerlo

- **Agrupaciones:** una decisión de estado `group` (p. ej. Q-X-9) sólo rotula a sus miembros: no tiene respuesta propia, no se lista ni impide cerrar. Cada fila se asocia a los miembros que cita.
- **Estado:** `respondida` (el Owner contestó: se muestra su respuesta), `bloqueante` (registrada como bloqueante, sin respuesta), `propuesta` (recomendación sin respuesta).
- **Impide cerrar la unidad:** toda decisión no respondida cuyo ID figura en alguna fila de la unidad (regla de `AGENTS.md`: una pregunta abierta deja la fila `partial`, nunca `done`). Se cumple por construcción para las columnas `bloqueante` y `propuesta`.
- **Impide comenzar la unidad:** **no se deduce mecánicamente.** El registro no tiene un campo para eso. Las columnas *Ficha: bloquea* y *Ficha: puede avanzar* citan, tal cual y truncadas, lo que la propia ficha de la decisión afirma (cuando lo afirma; `—` = la ficha no lo dice y falta leerlo). Hasta que se lea el contrato de cada fila, una decisión sólo debe tratarse como impedimento para comenzar si la ficha lo dice explícitamente.
- Cada excerpt describe la decisión **completa**: puede nombrar filas de otras unidades. Para esta unidad sólo cuentan las filas de la columna 3.
- Los excerpts de la ficha no son aceptaciones: toda recomendación sigue sin aprobar.

## P01 PATH

16 filas; 8 decisiones distintas en sus filas: 7 respondidas, 0 bloqueantes, 1 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-PATH-1 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-PATH-002 | Bloquea: nada hasta que exista un consumidor (cluster); puede avanzar: AC-PATH-002 con pekko_compare. | — | `classic-foundation` |
| Q-PATH-2 | accepted (2026-11-18): aceptar lo implementado: message() muestra U+FFFD y message_utf16() es exacto (el sustituto suelto de Pekko). | 1: AC-PATH-008 | Bloquea: AC-PATH-008 (cierre; VAL-22/23 pendientes); puede avanzar: el resto. | — | `classic-foundation` |
| Q-PATH-3 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 2: AC-PATH-001, AC-PATH-014 | Bloquea: nada. | — | `classic-foundation` |
| Q-PATH-4 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 4: AC-PATH-001, AC-PATH-004, AC-PATH-007, AC-PATH-008 | Bloquea: nada; puede avanzar con (a). | — | `classic-foundation` |
| Q-PATH-5 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-PATH-009 | Bloquea: AC-PATH-009 (cierre); puede avanzar: AC-PATH-007/008. | — | `classic-foundation` |
| Q-PATH-6 | accepted_with_changes (2026-11-18): aceptar (b): reescritura en sala limpia de uri.rs y jstring.rs desde RFC 2396/3986/3629 y los casos grabados, por un agente que no lee los archivos actuales… | 7: AC-PATH-002, AC-PATH-004, AC-PATH-005, AC-PATH-006, AC-PATH-007, AC-PATH-011 … (+1) | Bloquea: publicación de uri.rs/jstring.rs; puede avanzar: el resto de PATH. | — | `classic-foundation` |
| Q-UTIL-14 | accepted (2026-11-18): aceptar (a) para FrequencySketch, incluida la validación de desbordamiento de 2.0.0, y (b) para el resto. | 2: AC-PATH-001, AC-PATH-014 | Bloquea: AC-UTIL-017.1 (impl exacta); puede avanzar: el resto de AC-UTIL-017.* y AC-UTIL-006.*. | — | `classic-foundation` |
| Q-PATH-7 | propuesta | 1: AC-PATH-004 | — | — | `classic-foundation` |

## P02 UTIL-NUCLEO

9 filas; 7 decisiones distintas en sus filas: 7 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-1 | rejected (2026-10-31): rechazar: la recomendación (a) (std::time::Duration, sólo valores positivos) NO se adopta. ALTERNATIVA ELEGIDA Y VIGENTE: preservar el dominio de Pekko, es… | 1: AC-UTIL-001 | — | — | `classic-foundation` |
| Q-UTIL-17 | accepted (2026-11-18): aceptar (a): Display de PekkoException y ConfigurationException muestra sólo el mensaje; java_class() queda aparte. | 2: AC-UTIL-004.1, AC-UTIL-004.2 | — | — | `classic-foundation` |
| Q-UTIL-2 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-001 | Bloquea: cierre de AC-UTIL-001 (TO-STR-*); puede avanzar: resto de AC-UTIL-001. | — | `classic-foundation` |
| Q-UTIL-3 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 3: AC-UTIL-002.3, AC-UTIL-004.1, AC-UTIL-004.2 | Bloquea: cierre de AC-UTIL-002.3/004.1/004.2; puede avanzar: el resto. | — | `classic-foundation` |
| Q-UTIL-4 | accepted (2026-11-18): aceptar (i-a). Autoriza excluir OnlyCauseStackTrace. | 2: AC-UTIL-004.2, AC-UTIL-004.3 | Bloquea: AC-UTIL-004.2/.3 (cierre); puede avanzar: AC-UTIL-004.1. | — | `classic-foundation` |
| Q-UTIL-5 | accepted (2026-11-18): aceptar (a): pub struct Done. | 2: AC-UTIL-003.1, AC-UTIL-003.2 | Bloquea: cierre de AC-UTIL-003.1/.2 (estado `partial`); puede avanzar: el resto. | — | `classic-foundation` |
| Q-UTIL-7 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-002.2 | Bloquea: nada; puede avanzar: AC-UTIL-002.2 con (a) y el golden limitado a Unicode <= 10 (como ahora). | — | `classic-foundation` |

## P03 UTIL-ESTRUCTURAS

16 filas; 10 decisiones distintas en sus filas: 8 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-10 | accepted (2026-11-18): aceptar (a). | 7: AC-UTIL-007, AC-UTIL-008, AC-UTIL-010, AC-UTIL-011, AC-UTIL-012, AC-UTIL-013 … (+1) | Bloquea: AC-UTIL-012 y AC-UTIL-013 (colas bloqueantes) hasta fijar Q-X-4; puede avanzar: AC-UTIL-010, 011, 017.*. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 10: AC-UTIL-012, AC-UTIL-014, AC-UTIL-015.1, AC-UTIL-015.2, AC-UTIL-016.4, AC-UTIL-017.1 … (+4) | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-UTIL-14 | accepted (2026-11-18): aceptar (a) para FrequencySketch, incluida la validación de desbordamiento de 2.0.0, y (b) para el resto. | 2: AC-UTIL-016.4, AC-UTIL-017.1 | Bloquea: AC-UTIL-017.1 (impl exacta); puede avanzar: el resto de AC-UTIL-017.* y AC-UTIL-006.*. | — | `classic-foundation` |
| Q-UTIL-15 | accepted (2026-11-18): aceptar (b): devolver 0. Autoriza la divergencia con Pekko; queda documentada. | 1: AC-UTIL-010 | Bloquea: nada (AC-UTIL-010 puede cerrarse con (b) documentado); puede avanzar: todo AC-UTIL-010. | — | `classic-foundation` |
| Q-UTIL-8 | accepted (2026-11-18): aceptar (c). Autoriza no reproducir makePattern bit a bit. | 2: AC-UTIL-009, AC-UTIL-017.1 | Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023). | — | `classic-foundation` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-UTIL-012 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-UTIL-015.1 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AC-UTIL-008 | — | — | `transversal` |
| Q-UTIL-21 | propuesta | 1: AC-UTIL-009 | Bloquea: AC-UTIL-009 A7; puede avanzar: el resto de la fila. Propuesta de sustitución por la instrucción del Owner del 2026-11-20: Q-X-21 (paridad observable estricta); esta ficha se conserva por trazabilidad. | — | `classic-foundation` |
| Q-X-21 | propuesta | 1: AC-UTIL-009 | Bloquea: el cierre de AC-CFG-013, AC-CFG-012.1, AC-CFG-012.2 y AC-UTIL-009 hasta que cada diferencia desaparezca o quede bloqueada por una dependencia. Reversible. | — | `classic-foundation` |

## P04 UTIL-BYTES

9 filas; 11 decisiones distintas en sus filas: 6 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-11 | accepted_with_changes (2026-11-18): aceptar las claves; con la opción en `on` el arranque falla con un error de configuración (no un no-op silencioso) hasta que exista AC-SER. | 1: AC-UTIL-006.8 | Bloquea: AC-CFG-010 (y AC-UTIL-006.8); puede avanzar: aceptar las claves en `off`. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 2: AC-UTIL-006.8, AC-UTIL-022 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-UTIL-12 | accepted (2026-11-18): aceptar (c): newtype sobre bytes::Bytes, con lista de fragmentos sólo si IO la necesita. | 9: AC-UTIL-006.1, AC-UTIL-006.2, AC-UTIL-006.3, AC-UTIL-006.4, AC-UTIL-006.5, AC-UTIL-006.6 … (+3) | Bloquea: AC-UTIL-006.* y AC-UTIL-022 (impl) y, por dependencia, AC-IO-*/AC-SER-*; puede avanzar: toda UTIL/CFG/PATH/EXT sin ByteString. | — | `classic-foundation` |
| Q-UTIL-14 | accepted (2026-11-18): aceptar (a) para FrequencySketch, incluida la validación de desbordamiento de 2.0.0, y (b) para el resto. | 1: AC-UTIL-006.1 | Bloquea: AC-UTIL-017.1 (impl exacta); puede avanzar: el resto de AC-UTIL-017.* y AC-UTIL-006.*. | — | `classic-foundation` |
| Q-UTIL-8 | accepted (2026-11-18): aceptar (c). Autoriza no reproducir makePattern bit a bit. | 1: AC-UTIL-006.1 | Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023). | — | `classic-foundation` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-UTIL-006.8 | — | — | `transversal` |
| Q-UTIL-22 | propuesta | 1: AC-UTIL-006.6 | Bloquea: nada observable de lo ya implementado; AC-UTIL-006.6 queda `partial` (charsets). | — | `classic-foundation` |
| Q-UTIL-23 | propuesta | 2: AC-UTIL-006.1, AC-UTIL-006.4 | Bloquea: AC-UTIL-006.1 (`partial`: mensaje de `apply`/`head`/`last` sobre un valor compacto y del vacío). | — | `classic-foundation` |
| Q-UTIL-24 | propuesta | 1: AC-UTIL-006.4 | — | — | `classic-foundation` |
| Q-UTIL-25 | propuesta | 1: AC-UTIL-022 | — | — | `classic-foundation` |
| Q-UTIL-26 | propuesta | 1: AC-UTIL-006.8 | — | — | `classic-foundation` |

## P05 UTIL-GLOB

1 filas; 1 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-8 | accepted (2026-11-18): aceptar (c). Autoriza no reproducir makePattern bit a bit. | 1: AC-UTIL-016.6 | Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023). | — | `classic-foundation` |

## P06 UTIL-PLATAFORMA

8 filas; 6 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-10 | accepted (2026-11-18): aceptar (a). | 2: AC-UTIL-024, AC-UTIL-025 | Bloquea: AC-UTIL-012 y AC-UTIL-013 (colas bloqueantes) hasta fijar Q-X-4; puede avanzar: AC-UTIL-010, 011, 017.*. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 4: AC-UTIL-020, AC-UTIL-021, AC-UTIL-027, AC-UTIL-028 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-UTIL-13 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-026 | Bloquea: nada; (a) es suficiente. | — | `classic-foundation` |
| Q-UTIL-8 | accepted (2026-11-18): aceptar (c). Autoriza no reproducir makePattern bit a bit. | 1: AC-UTIL-023 | Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023). | — | `classic-foundation` |
| Q-UTIL-18 | propuesta | 1: AC-UTIL-025 | — | — | `classic-foundation` |
| Q-UTIL-19 | propuesta | 1: AC-UTIL-020 | — | — | `classic-foundation` |

## P07 CFG-HOCON

12 filas; 12 decisiones distintas en sus filas: 7 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-3 | accepted (2026-11-18): aceptar (a): se implementa checkValid porque Q-X-1 es parser propio. | 1: AC-CFG-019 | Bloquea: AC-CFG-019 (y la parte fail-fast de AC-CFG-002). | Puede avanzar: lectura de cada clave en Settings. | `classic-foundation` |
| Q-CFG-30 | accepted (2026-11-19): aceptar (a) con la premisa corregida: 357 de las 362 diferencias eran empates exactos mal resueltos por rakko (se corrigen: cifra par) y 5 son bugs de JDK 17;… | 2: AC-CFG-017.1, AC-UTIL-002.5 | — | — | `classic-foundation` |
| Q-CFG-4 | accepted (2026-11-18): aceptar (a). Los mensajes de error de Config 1.4.9 son contrato. Condición: volver a grabar el golden de HOCON con el CLI del oráculo (jshell está deprecado). | 9: AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5, AC-CFG-017.6 … (+3) | Bloquea: AC-CFG-017.* (golden) y AC-UTIL-002.4/.5 (impl); puede avanzar: AC-CFG-002/005 con tipos de error propios. | — | `classic-foundation` |
| Q-CFG-5 | accepted_with_changes (2026-11-18): aceptar (a) con el valor "2.0.0" (la referencia pasó de 1.7.1 a 2.0.0, Q-X-11). | 1: AC-CFG-003 | Bloquea: AC-CFG-003, AC-UTIL-005.1; puede avanzar: el resto. | — | `classic-foundation` |
| Q-CFG-6 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-CFG-001 | Bloquea: AC-CFG-001 (carga por defecto); puede avanzar: `ActorSystem::new(config)` con config explícita. | — | `classic-foundation` |
| Q-UTIL-1 | rejected (2026-10-31): rechazar: la recomendación (a) (std::time::Duration, sólo valores positivos) NO se adopta. ALTERNATIVA ELEGIDA Y VIGENTE: preservar el dominio de Pekko, es… | 1: AC-UTIL-002.4 | — | — | `classic-foundation` |
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 9: AC-CFG-001, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5 … (+3) | — | — | `transversal` |
| Q-CFG-14 | propuesta | 2: AC-CFG-001, AC-CFG-017.6 | Bloquea: nada; las filas quedan `partial` hasta medir los mensajes de file/url. | — | `classic-foundation` |
| Q-CFG-15 | propuesta | 9: AC-CFG-001, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5 … (+3) | — | — | `classic-foundation` |
| Q-CFG-16 | propuesta | 1: AC-CFG-001 | — | — | `classic-foundation` |
| Q-CFG-18 | propuesta | 1: AC-CFG-019 | Bloquea: nada; las filas quedan `partial`. | — | `classic-foundation` |
| Q-CFG-19 | propuesta | 1: AC-CFG-019 | Bloquea: nada; reversible. | — | `classic-foundation` |

## P08 CFG-AJUSTES

10 filas; 14 decisiones distintas en sus filas: 10 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-10 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 2: AC-CFG-004, AC-CFG-018 | Bloquea: AC-CFG-018; puede avanzar: AC-CFG-004 sin esa clave. | — | `classic-foundation` |
| Q-CFG-11 | accepted_with_changes (2026-11-18): aceptar las claves; con la opción en `on` el arranque falla con un error de configuración (no un no-op silencioso) hasta que exista AC-SER. | 1: AC-CFG-010 | Bloquea: AC-CFG-010 (y AC-UTIL-006.8); puede avanzar: aceptar las claves en `off`. | — | `classic-foundation` |
| Q-CFG-3 | accepted (2026-11-18): aceptar (a): se implementa checkValid porque Q-X-1 es parser propio. | 1: AC-CFG-002 | Bloquea: AC-CFG-019 (y la parte fail-fast de AC-CFG-002). | Puede avanzar: lectura de cada clave en Settings. | `classic-foundation` |
| Q-CFG-4 | accepted (2026-11-18): aceptar (a). Los mensajes de error de Config 1.4.9 son contrato. Condición: volver a grabar el golden de HOCON con el CLI del oráculo (jshell está deprecado). | 2: AC-CFG-002, AC-CFG-005 | Bloquea: AC-CFG-017.* (golden) y AC-UTIL-002.4/.5 (impl); puede avanzar: AC-CFG-002/005 con tipos de error propios. | — | `classic-foundation` |
| Q-CFG-7 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-CFG-007 | Bloquea: AC-CFG-007 y AC-UTIL-016.11 (impl); puede avanzar: aceptar y validar las claves. | — | `classic-foundation` |
| Q-CFG-8 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-CFG-006 | Bloquea: AC-CFG-006 y AC-EXT-003; puede avanzar: AC-EXT-001.*, AC-EXT-002. | — | `classic-foundation` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 4: AC-CFG-004, AC-CFG-006, AC-CFG-009, AC-CFG-018 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-CFG-010 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AC-CFG-011 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 3: AC-CFG-004, AC-CFG-011, AC-CFG-018 | — | — | `transversal` |
| Q-CFG-15 | propuesta | 9: AC-CFG-002, AC-CFG-004, AC-CFG-005, AC-CFG-006, AC-CFG-007, AC-CFG-009 … (+3) | — | — | `classic-foundation` |
| Q-CFG-17 | propuesta | 2: AC-CFG-002, AC-CFG-010 | Bloquea: nada; AC-CFG-010 queda `partial` hasta AC-SER. | — | `classic-foundation` |
| Q-CFG-18 | propuesta | 1: AC-CFG-002 | Bloquea: nada; las filas quedan `partial`. | — | `classic-foundation` |
| Q-CFG-19 | propuesta | 1: AC-CFG-002 | Bloquea: nada; reversible. | — | `classic-foundation` |

## P09 CFG-ENTORNO

3 filas; 8 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-13 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-CFG-015 | Bloquea: nada; (a) es suficiente. | — | `classic-foundation` |
| Q-CFG-9 | accepted (2026-11-18): aceptar (a)+(c). | 2: AC-CFG-008, AC-CFG-016 | Bloquea: AC-CFG-008; puede avanzar: provider `local`. | — | `classic-foundation` |
| Q-X-10 | accepted (2026-11-18): autoriza el alcance (A): sólo actor y actor-typed locales (sin remoting, cluster, persistence, streams ni HTTP). | 1: AC-CFG-008 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 2: AC-CFG-008, AC-CFG-016 | — | — | `transversal` |
| Q-CFG-31 | propuesta | 2: AC-CFG-008, AC-CFG-016 | Bloquea: AC-CFG-016 y AC-CFG-008 (cierre, quedan partial); puede avanzar: el resto. Reversible. | — | `classic-foundation` |
| Q-CFG-32 | propuesta | 1: AC-CFG-015 | Bloquea: el cierre de AC-CFG-015 (queda partial); puede avanzar: lo demás. Reversible. | — | `classic-foundation` |
| Q-CFG-33 | propuesta | 1: AC-CFG-015 | Bloquea: nada. Reversible. | — | `classic-foundation` |
| Q-CFG-34 | propuesta | 2: AC-CFG-008, AC-CFG-015 | Bloquea: el cierre de AC-CFG-015 y AC-CFG-008 (quedan partial); puede avanzar: el resto. Reversible. | — | `classic-foundation` |

## P10 CFG-DEPLOY

3 filas; 8 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-12 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-CFG-013 | Bloquea: nada en pekko-actor; puede avanzar: AC-CFG-013 con (a). | — | `classic-foundation` |
| Q-CFG-4 | accepted (2026-11-18): aceptar (a). Los mensajes de error de Config 1.4.9 son contrato. Condición: volver a grabar el golden de HOCON con el CLI del oráculo (jshell está deprecado). | 1: AC-CFG-012.1 | Bloquea: AC-CFG-017.* (golden) y AC-UTIL-002.4/.5 (impl); puede avanzar: AC-CFG-002/005 con tipos de error propios. | — | `classic-foundation` |
| Q-UTIL-8 | accepted (2026-11-18): aceptar (c). Autoriza no reproducir makePattern bit a bit. | 1: AC-CFG-012.2 | Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023). | — | `classic-foundation` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 2: AC-CFG-012.1, AC-CFG-013 | — | — | `transversal` |
| Q-CFG-35 | propuesta | 1: AC-CFG-013 | Bloquea: el cierre de AC-CFG-013; puede avanzar: todo el comportamiento de datos. Reversible. Propuesta de sustitución por la instrucción del Owner del 2026-11-20: Q-X-21 (paridad observable estricta); esta ficha se conserva por… | — | `classic-foundation` |
| Q-CFG-36 | propuesta | 1: AC-CFG-012.1 | Bloquea: el cierre de AC-CFG-012.1; puede avanzar: todo el resto. Reversible. Propuesta de sustitución por la instrucción del Owner del 2026-11-20: Q-X-21 (paridad observable estricta); esta ficha se conserva por trazabilidad. | — | `classic-foundation` |
| Q-UTIL-21 | propuesta | 1: AC-CFG-012.2 | Bloquea: AC-UTIL-009 A7; puede avanzar: el resto de la fila. Propuesta de sustitución por la instrucción del Owner del 2026-11-20: Q-X-21 (paridad observable estricta); esta ficha se conserva por trazabilidad. | — | `classic-foundation` |
| Q-X-21 | propuesta | 3: AC-CFG-012.1, AC-CFG-012.2, AC-CFG-013 | Bloquea: el cierre de AC-CFG-013, AC-CFG-012.1, AC-CFG-012.2 y AC-UTIL-009 hasta que cada diferencia desaparezca o quede bloqueada por una dependencia. Reversible. | — | `classic-foundation` |

## P11 EVT-BUS

11 filas; 5 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-EVT-1 | accepted (2026-11-18): aceptar, incluida la supresión de dead letters durante el shutdown terminante de 2.0.0. | 2: AC-EVT-006, AC-EVT-008 | Bloquea: la comparación textual exacta de AC-EVT-006/008/009/010/013.3/018/019; la lógica avanza con `message_type_name`. | — | `classic-system` |
| Q-EVT-3 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-EVT-013.1 | Bloquea: nada (puede decidirse al implementar AC-EVT-013.1). | — | `classic-system` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-EVT-016 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-EVT-008 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 2: AC-EVT-003, AC-EVT-006 | — | — | `transversal` |

## P12 SCH-RUEDA

13 filas; 6 decisiones distintas en sus filas: 6 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-SCH-1 | accepted (2026-11-18): aceptar (a): rueda propia con hilo propio. | 8: AC-SCH-001, AC-SCH-002, AC-SCH-003, AC-SCH-008.1, AC-SCH-008.2, AC-SCH-008.3 … (+2) | — | — | `classic-dispatch` |
| Q-SCH-2 | accepted (2026-11-18): aceptar (a). | 1: AC-SCH-010 | Bloquea sólo la carga por nombre. | — | `classic-dispatch` |
| Q-SCH-4 | rejected (2026-11-18): rechazar (a) (alias deprecado). Elegido (b): sólo schedule_at_fixed_rate, como en 2.0.0. Autoriza la exclusión del alias `schedule` deprecado. | 2: AC-SCH-003, AC-SCH-004 | — | — | `classic-dispatch` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-SCH-010 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 3: AC-SCH-008.1, AC-SCH-008.2, AC-SCH-008.3 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-SCH-015 | — | — | `transversal` |

## P13 MBX-COLAS

12 filas; 12 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 7 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-MBX-14 | accepted_with_changes (2026-11-26): aceptar con cambios: Acepto la alternativa (a), incluyendo ProducedQueue::class_name como API pública. Acepto la representación abierta de capacidades… | 1: AC-MBX-011 | — | — | `classic-dispatch` |
| Q-MBX-15 | accepted (2026-11-26): aceptar (a): MessageQueue::as_deque() -> Option<&dyn DequeBasedMessageQueue>, con None por defecto. Registrada en owner-decisions.tsv en representación del… | 1: AC-MBX-011 | — | — | `classic-dispatch` |
| Q-MBX-6 | accepted (2026-11-20): aceptar (b): pool de hilos propio, coherente con Q-X-4 (A); el turno de mailbox es un trabajo del pool; `throughput` y `throughput-deadline-time` son… | 2: AC-MBX-006, AC-MBX-008 | Bloquea la implementación de AC-MBX-002 y AC-DSP-002/003 hasta decidir Q-X-4; el diseño del runner con reloj inyectable puede avanzar. | — | `classic-dispatch` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-MBX-004 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-MBX-007 | — | — | `transversal` |
| Q-CFG-31 | propuesta | 1: AC-MBX-004 | Bloquea: AC-CFG-016 y AC-CFG-008 (cierre, quedan partial); puede avanzar: el resto. Reversible. | — | `classic-foundation` |
| Q-DSP-2 | propuesta | 1: AC-MBX-004 | Bloquea sólo la carga por nombre (AC-DSP-004/010, AC-MBX-004); el registro programático puede avanzar. | — | `classic-dispatch` |
| Q-MBX-1 | propuesta | 5: AC-MBX-007, AC-MBX-009, AC-MBX-010, AC-MBX-011, AC-MBX-012 | Bloquea la implementación de: AC-MBX-007 y las variantes acotadas de AC-MBX-009/010/011/012 con timeout>0 y AC-MBX-016 (overflow del stash en deque acotado). | Puede avanzar: todas las colas no acotadas, NonBlockingBoundedMailbox (AC-MBX-008) y las acotadas con timeout 0. Requiere decisión del Owner (Q-X-4). | `classic-dispatch` |
| Q-MBX-2 | propuesta | 2: AC-MBX-011, AC-MBX-015 | — | — | `classic-dispatch` |
| Q-MBX-3 | propuesta | 1: AC-MBX-012 | Bloquea la implementación de AC-MBX-012 (ambas variantes); el resto puede avanzar. | — | `classic-dispatch` |
| Q-MBX-4 | propuesta | 2: AC-MBX-009, AC-MBX-010 | — | — | `classic-dispatch` |
| Q-MBX-7 | propuesta | 2: AC-MBX-009, AC-MBX-010 | Bloquea sólo el cierre de AC-MBX-009/010 en el borde; el resto puede avanzar. | — | `classic-dispatch` |

## P14 REF-BASE

15 filas; 7 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 2: AC-REF-007.1, AC-REF-008 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 2: AC-DSP-020, AC-REF-002 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 1: AC-REF-025 | — | — | `transversal` |
| Q-REF-1 | propuesta | 3: AC-REF-002, AC-REF-005, AC-REF-024 | Bloquea: `impl` de AC-REF-002/005/024. | Puede avanzar: el resto de REF y todo CELL (la elección sólo afecta a la firma de tell/sender). | `classic-cell` |
| Q-REF-2 | propuesta | 3: AC-REF-007.1, AC-REF-007.2, AC-REF-008 | Bloquea: `impl` de AC-REF-008 y la parte reflexiva de AC-REF-007.1/.2; AC-CELL-002.2 sólo en el texto de InstantiationException. | Puede avanzar: Props con closure + trait ActorFactory, AC-CELL-002.1, AC-CELL-026. | `classic-cell` |
| Q-REF-4 | propuesta | 1: AC-REF-006 | Bloquea: sólo el cierre de AC-REF-006. | Puede avanzar: todo. | `classic-cell` |
| Q-REF-6 | propuesta | 2: AC-REF-001.1, AC-REF-001.2 | Bloquea: la validación (no la `impl`) de AC-REF-001.x, AC-REF-019 y AC-CELL-031. | Puede avanzar: toda la implementación. | `classic-cell` |

## P15 MBX-NUCLEO

7 filas; 7 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-MBX-10 | accepted (2026-11-20): aceptar (a): la secuencia de 10 PR del ADR-0001 §6. Los PR 4-10 siguen condicionados a las decisiones Q-MBX pendientes que cada uno cita. | 2: AC-MBX-001, AC-MBX-002 | — | — | `classic-dispatch` |
| Q-MBX-5 | accepted (2026-11-20): aceptar (a) con `Mutex<Vec>` invertido al drenar: la cola de mensajes de sistema es FIFO observable (más antiguo primero), con centinela de cierre y sin… | 2: AC-MBX-001, AC-MBX-002 | — | — | `classic-dispatch` |
| Q-MBX-6 | accepted (2026-11-20): aceptar (b): pool de hilos propio, coherente con Q-X-4 (A); el turno de mailbox es un trabajo del pool; `throughput` y `throughput-deadline-time` son… | 5: AC-DSP-001, AC-DSP-002, AC-DSP-003, AC-DSP-023, AC-MBX-002 | Bloquea la implementación de AC-MBX-002 y AC-DSP-002/003 hasta decidir Q-X-4; el diseño del runner con reloj inyectable puede avanzar. | — | `classic-dispatch` |
| Q-MBX-9 | accepted (2026-11-20): aceptar (B): estado del mailbox separado de las colas de sistema y de usuario (ADR-0001). No aprueba Q-MBX-1, Q-MBX-2, Q-MBX-3, Q-MBX-5 ni Q-MBX-6. | 2: AC-MBX-001, AC-MBX-002 | — | — | `classic-dispatch` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 3: AC-CELL-013.1, AC-DSP-001, AC-MBX-001 | — | — | `transversal` |
| Q-DSP-5 | propuesta | 2: AC-DSP-003, AC-MBX-002 | — | — | `classic-dispatch` |
| Q-MBX-11 | propuesta | 2: AC-DSP-002, AC-MBX-002 | — | — | `classic-dispatch` |

## P16 DSP-EJECUTORES

19 filas; 13 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 8 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 2: AC-UTIL-018.4, AC-UTIL-018.5 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-DSP-004 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-DSP-010 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 5: AC-DSP-006, AC-DSP-008, AC-DSP-013, AC-UTIL-018.4, AC-UTIL-018.5 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-DSP-016.1 | — | — | `transversal` |
| Q-CFG-31 | propuesta | 2: AC-DSP-004, AC-DSP-010 | Bloquea: AC-CFG-016 y AC-CFG-008 (cierre, quedan partial); puede avanzar: el resto. Reversible. | — | `classic-foundation` |
| Q-CFG-32 | propuesta | 1: AC-DSP-006 | Bloquea: el cierre de AC-CFG-015 (queda partial); puede avanzar: lo demás. Reversible. | — | `classic-foundation` |
| Q-DSP-1 | propuesta | 9: AC-DSP-005, AC-DSP-006, AC-DSP-008, AC-DSP-009, AC-DSP-010, AC-DSP-011 … (+3) | Bloquea la implementación de AC-DSP-008/009/013/014/019; puede avanzar: lectura/validación de claves y las funciones puras `scaledPoolSize` y `resolveMinimumRunnable`. | — | `classic-dispatch` |
| Q-DSP-2 | propuesta | 3: AC-DSP-004, AC-DSP-006, AC-DSP-010 | Bloquea sólo la carga por nombre (AC-DSP-004/010, AC-MBX-004); el registro programático puede avanzar. | — | `classic-dispatch` |
| Q-DSP-3 | propuesta | 4: AC-DSP-016.1, AC-DSP-017.1, AC-DSP-017.2, AC-DSP-018 | Bloquea sólo AC-DSP-017.2. | — | `classic-dispatch` |
| Q-DSP-4 | propuesta | 2: AC-DSP-004, AC-DSP-022 | — | — | `classic-dispatch` |
| Q-DSP-6 | propuesta | 2: AC-DSP-008, AC-DSP-019 | — | — | `classic-dispatch` |
| Q-DSP-7 | propuesta | 1: AC-DSP-016.2 | Bloquea la implementación de AC-DSP-016.2; no bloquea nada más. | — | `classic-dispatch` |

## P17 REF-MUERTOS

4 filas; 4 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-MBX-10 | accepted (2026-11-20): aceptar (a): la secuencia de 10 PR del ADR-0001 §6. Los PR 4-10 siguen condicionados a las decisiones Q-MBX pendientes que cada uno cita. | 1: AC-MBX-003 | — | — | `classic-dispatch` |
| Q-MBX-5 | accepted (2026-11-20): aceptar (a) con `Mutex<Vec>` invertido al drenar: la cola de mensajes de sistema es FIFO observable (más antiguo primero), con centinela de cierre y sin… | 2: AC-MBX-003, AC-MBX-021 | — | — | `classic-dispatch` |
| Q-MBX-8 | accepted (2026-11-20): aceptar (b)+(c): Rust implementa un protocolo de cierre que impide el mensaje varado de CC-05 (el mensaje siempre es dead letter); el oráculo JVM acepta los… | 1: AC-MBX-003 | — | — | `classic-dispatch` |
| Q-MBX-9 | accepted (2026-11-20): aceptar (B): estado del mailbox separado de las colas de sistema y de usuario (ADR-0001). No aprueba Q-MBX-1, Q-MBX-2, Q-MBX-3, Q-MBX-5 ni Q-MBX-6. | 1: AC-MBX-003 | — | — | `classic-dispatch` |

## P18 CELL-HIJOS

6 filas; 2 decisiones distintas en sus filas: 0 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-MBX-11 | propuesta | 1: AC-CELL-015.1 | — | — | `classic-dispatch` |
| Q-REF-6 | propuesta | 1: AC-REF-019 | Bloquea: la validación (no la `impl`) de AC-REF-001.x, AC-REF-019 y AC-CELL-031. | Puede avanzar: toda la implementación. | `classic-cell` |

## P19 CELL-EJECUCION

11 filas; 7 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-CELL-001 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 3: AC-CELL-003, AC-CELL-017, AC-CELL-018 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 1: AC-CELL-018 | — | — | `transversal` |
| Q-CELL-1 | propuesta | 2: AC-CELL-001, AC-CELL-005 | Bloquea: `impl` de AC-CELL-001/005 (y la firma de Actor para todo el crate). | Puede avanzar: la documentación y los tests de ciclo de vida (se escriben contra el trait). | `classic-cell` |
| Q-CELL-3 | propuesta | 1: AC-CELL-019 | Bloquea: nada de la documentación; el cierre de `validation` de AC-CELL-019. | Puede avanzar: todo. | `classic-cell` |
| Q-MBX-11 | propuesta | 1: AC-CELL-003 | — | — | `classic-dispatch` |
| Q-REF-6 | propuesta | 1: AC-CELL-031 | Bloquea: la validación (no la `impl`) de AC-REF-001.x, AC-REF-019 y AC-CELL-031. | Puede avanzar: toda la implementación. | `classic-cell` |

## P20 SUP-ESTRATEGIAS

10 filas; 4 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-SUP-005 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-SUP-004 | — | — | `transversal` |
| Q-SUP-1 | propuesta | 4: AC-SUP-004, AC-SUP-012, AC-SUP-017, AC-SUP-021 | Bloquea: `impl` de AC-SUP-004/012/017 y, indirectamente, AC-SUP-001/003/008/013/018/021 y AC-CELL-002.2/007.4. | Puede avanzar: la planificación de ciclo de vida sin deciders, AC-SUP-006.x (aritmética) y DW. Decidir junto con Q-X-5. | `classic-cell` |
| Q-SUP-2 | propuesta | 3: AC-SUP-006.1, AC-SUP-006.2, AC-SUP-021 | Bloquea: `impl` de AC-SUP-006.2 (borde de ventana enorme). | Puede avanzar: AC-SUP-006.1 y todo lo demás (la aritmética normal es idéntica). | `classic-cell` |

## P21 SUP-FALLO

11 filas; 5 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 1 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CELL-7 | accepted (2026-11-26): aceptar (b): helper público que ejecute los pasos del preRestart por defecto (unwatch y stop de cada hijo, luego post_stop) para que un override pueda… | 1: AC-CELL-006 | — | — | `classic-cell` |
| Q-SUP-3 | accepted (2026-11-26): aceptar (a): Throwable lleva un campo `cause: Option<Box<Throwable>>`; se acepta que rompe la construcción por literal. logFailure publica el mensaje de la… | 1: AC-SUP-018 | — | — | `classic-cell` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-SUP-010 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 2: AC-SUP-001, AC-SUP-016 | — | — | `transversal` |
| Q-CELL-2 | propuesta | 1: AC-CELL-021 | Bloquea: sólo el cierre de AC-CELL-021 (requiere autorización explícita del Owner). | Puede avanzar: todo. | `classic-cell` |

## P22 CELL-REINICIO-PARADA

21 filas; 6 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CELL-6 | accepted (2026-11-26): aceptar (a): en el texto de PreRestartException un mensaje se nombra `class ?` salvo &str/String, Kill, ActorIdentity, Terminated y Failed, que llevan el… | 1: AC-CELL-007.1 | — | — | `classic-cell` |
| Q-SUP-3 | accepted (2026-11-26): aceptar (a): Throwable lleva un campo `cause: Option<Box<Throwable>>`; se acepta que rompe la construcción por literal. logFailure publica el mensaje de la… | 3: AC-CELL-002.2, AC-CELL-007.1, AC-CELL-007.4 | — | — | `classic-cell` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 4: AC-CELL-002.2, AC-CELL-007.1, AC-CELL-007.4, AC-CELL-009 | — | — | `transversal` |
| Q-CELL-4 | propuesta | 1: AC-CELL-023 | Bloquea: `validation` de AC-CELL-023 (impl puede avanzar). | Puede avanzar: el resto de CELL. | `classic-cell` |
| Q-DW-2 | propuesta | 1: AC-DW-010 | Bloquea: `impl` de AC-DW-012 y el orden remoto→local de AC-DW-010. | Puede avanzar: el resto de DW (el orden local es el único observable sin remote). | `classic-cell` |
| Q-REF-2 | propuesta | 1: AC-CELL-002.2 | Bloquea: `impl` de AC-REF-008 y la parte reflexiva de AC-REF-007.1/.2; AC-CELL-002.2 sólo en el texto de InstantiationException. | Puede avanzar: Props con closure + trait ActorFactory, AC-CELL-002.1, AC-CELL-026. | `classic-cell` |

## P23 DW-VIGILANCIA

13 filas; 5 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-DW-3 | accepted (2026-11-26): aceptar (a): igualdad por tipo concreto y valor mediante PartialEq: watch_with<M: Any + PartialEq + Send + Sync>(ref, msg). Se aceptan explícitamente las… | 1: AC-DW-005 | Bloquea AC-DW-005. | — | `classic-cell` |
| Q-DW-4 | accepted (2026-11-26): aceptar (a): en la IllegalStateException de watch / watchWith, String y &str se muestran por valor y cualquier otro tipo por su nombre Rust, sin cotas nuevas… | 1: AC-DW-005 | Bloquea el cierre de AC-DW-005. | — | `classic-cell` |
| Q-EVT-1 | accepted (2026-11-18): aceptar, incluida la supresión de dead letters durante el shutdown terminante de 2.0.0. | 1: AC-EVT-010 | Bloquea: la comparación textual exacta de AC-EVT-006/008/009/010/013.3/018/019; la lógica avanza con `message_type_name`. | — | `classic-system` |
| Q-DW-1 | propuesta | 2: AC-DW-002, AC-DW-006 | Bloquea: `impl` de AC-DW-002/006 (PartialEq). | Puede avanzar: el resto de DW. | `classic-cell` |
| Q-DW-2 | propuesta | 1: AC-DW-012 | Bloquea: `impl` de AC-DW-012 y el orden remoto→local de AC-DW-010. | Puede avanzar: el resto de DW (el orden local es el único observable sin remote). | `classic-cell` |

## P24 MBX-STASH

5 filas; 3 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-MBX-023 | — | — | `transversal` |
| Q-MBX-1 | propuesta | 1: AC-MBX-016 | Bloquea la implementación de: AC-MBX-007 y las variantes acotadas de AC-MBX-009/010/011/012 con timeout>0 y AC-MBX-016 (overflow del stash en deque acotado). | Puede avanzar: todas las colas no acotadas, NonBlockingBoundedMailbox (AC-MBX-008) y las acotadas con timeout 0. Requiere decisión del Owner (Q-X-4). | `classic-dispatch` |
| Q-MBX-2 | propuesta | 1: AC-MBX-018 | — | — | `classic-dispatch` |

## P25 SYS-GUARDIANES

12 filas; 11 decisiones distintas en sus filas: 7 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-SYS-001.2 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 3: AC-SUP-014, AC-SYS-002, AC-SYS-008 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-SYS-003.1 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 2: AC-SYS-001.2, AC-SYS-007 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 7: AC-SYS-001.1, AC-SYS-001.2, AC-SYS-002, AC-SYS-007, AC-SYS-008, AC-SYS-009 … (+1) | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 1: AC-SYS-002 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 1: AC-SYS-009 | — | — | `transversal` |
| Q-CFG-32 | propuesta | 1: AC-SYS-001.2 | Bloquea: el cierre de AC-CFG-015 (queda partial); puede avanzar: lo demás. Reversible. | — | `classic-foundation` |
| Q-SYS-3 | propuesta | 1: AC-SYS-020 | Bloquea: nada (AC-SYS-020/021 pueden implementarse con (a)); si el Owner exige paridad de manifests, queda partial. | — | `classic-system` |
| Q-SYS-4 | propuesta | 2: AC-SYS-002, AC-SYS-003.2 | Bloquea: sólo AC-SYS-003.2 (sin spec) y la rama de espera de AC-SYS-002; el resto avanza. | — | `classic-system` |
| Q-SYS-6 | propuesta | 2: AC-SYS-001.2, AC-SYS-013 | Bloquea: nada; AC-SYS-001.2/013/018 avanzan con (a). | — | `classic-system` |

## P26 SYS-PROVEEDOR

8 filas; 8 decisiones distintas en sus filas: 6 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-9 | accepted (2026-11-18): aceptar (a)+(c). | 1: AC-SYS-018 | Bloquea: AC-CFG-008; puede avanzar: provider `local`. | — | `classic-foundation` |
| Q-EVT-1 | accepted (2026-11-18): aceptar, incluida la supresión de dead letters durante el shutdown terminante de 2.0.0. | 1: AC-SYS-012 | Bloquea: la comparación textual exacta de AC-EVT-006/008/009/010/013.3/018/019; la lógica avanza con `message_type_name`. | — | `classic-system` |
| Q-X-10 | accepted (2026-11-18): autoriza el alcance (A): sólo actor y actor-typed locales (sin remoting, cluster, persistence, streams ni HTTP). | 1: AC-SYS-018 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 2: AC-SYS-016, AC-SYS-018 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-SYS-012 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-SYS-016 | — | — | `transversal` |
| Q-SYS-5 | propuesta | 1: AC-SYS-012 | Bloquea: nada; AC-SYS-022 y AC-SYS-012 avanzan con (a). | — | `classic-system` |
| Q-SYS-6 | propuesta | 1: AC-SYS-018 | Bloquea: nada; AC-SYS-001.2/013/018 avanzan con (a). | — | `classic-system` |

## P27 SCH-ACTORES

4 filas; 1 decisiones distintas en sus filas: 0 respondidas, 0 bloqueantes, 1 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-SCH-3 | propuesta | 3: AC-SCH-011, AC-SCH-012, AC-SCH-013 | Bloquea AC-SCH-011/012/013 hasta Q-X-3; no bloquea el resto del grupo. | — | `classic-dispatch` |

## P28 SELECCION-CREACION

9 filas; 10 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-MBX-12 | accepted (2026-11-24): aceptar (a) exclusivamente como LIMITACIÓN TEMPORAL de la migración, no como paridad completa: registro programático en `Settings.mailboxes` (sin FQCN ni… | 1: AC-MBX-014 | — | — | `classic-dispatch` |
| Q-MBX-13 | accepted (2026-11-24): aceptar (a) con las limitaciones documentadas en `docs/evidence/mailbox-requirements/q-mbx-13.md`: `MessageQueueSemantics` como conjunto abierto identificado… | 2: AC-MBX-013, AC-MBX-014 | — | — | `classic-dispatch` |
| Q-MBX-14 | accepted_with_changes (2026-11-26): aceptar con cambios: Acepto la alternativa (a), incluyendo ProducedQueue::class_name como API pública. Acepto la representación abierta de capacidades… | 2: AC-MBX-013, AC-MBX-014 | — | — | `classic-dispatch` |
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-MBX-014 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 1: AC-MBX-020 | — | — | `transversal` |
| Q-DSP-2 | propuesta | 2: AC-DSP-007, AC-MBX-014 | Bloquea sólo la carga por nombre (AC-DSP-004/010, AC-MBX-004); el registro programático puede avanzar. | — | `classic-dispatch` |
| Q-MBX-1 | propuesta | 1: AC-MBX-014 | Bloquea la implementación de: AC-MBX-007 y las variantes acotadas de AC-MBX-009/010/011/012 con timeout>0 y AC-MBX-016 (overflow del stash en deque acotado). | Puede avanzar: todas las colas no acotadas, NonBlockingBoundedMailbox (AC-MBX-008) y las acotadas con timeout 0. Requiere decisión del Owner (Q-X-4). | `classic-dispatch` |
| Q-MBX-11 | propuesta | 2: AC-CELL-016, AC-REF-020 | — | — | `classic-dispatch` |
| Q-MBX-2 | propuesta | 2: AC-MBX-013, AC-MBX-014 | — | — | `classic-dispatch` |
| Q-REF-5 | propuesta | 3: AC-CELL-016, AC-CELL-027, AC-REF-020 | Bloquea: `impl` de AC-REF-020, AC-CELL-016 y AC-CELL-027. | Puede avanzar: AC-REF-003, AC-CELL-002.x, 007.x, 008.x, SUP y DW (no dependen de esta elección). | `classic-cell` |

## P29 REF-SELECCION

6 filas; 6 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-REF-011 | — | — | `transversal` |
| Q-REF-10 | propuesta | 1: AC-REF-014 | Bloquea el cierre de AC-REF-014. | — | `classic-cell` |
| Q-REF-11 | propuesta | 1: AC-REF-014 | Bloquea el cierre de AC-REF-014. | — | `classic-cell` |
| Q-REF-7 | propuesta | 1: AC-REF-013 | Bloquea el cierre de AC-REF-013. | — | `classic-cell` |
| Q-REF-8 | propuesta | 1: AC-REF-011 | Bloquea el cierre de AC-REF-011. | — | `classic-cell` |
| Q-REF-9 | propuesta | 1: AC-REF-014 | Bloquea el cierre de AC-REF-014 y AC-REF-013. | — | `classic-cell` |

## P30 EXT

5 filas; 6 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-8 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-EXT-003 | Bloquea: AC-CFG-006 y AC-EXT-003; puede avanzar: AC-EXT-001.*, AC-EXT-002. | — | `classic-foundation` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-EXT-003 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-EXT-001.2 | — | — | `transversal` |
| Q-EXT-1 | propuesta | 3: AC-EXT-001.1, AC-EXT-002, AC-EXT-004 | Bloquea: AC-EXT-001.1, AC-EXT-002 y, por el índice, AC-EXT-004; puede avanzar: AC-EXT-003. | — | `classic-foundation` |
| Q-EXT-2 | propuesta | 1: AC-EXT-001.2 | Bloquea: AC-EXT-001.2; puede avanzar: AC-EXT-002. | — | `classic-foundation` |
| Q-EXT-3 | propuesta | 1: AC-EXT-004 | Bloquea: el cierre de impl de AC-EXT-004; puede avanzar: todo lo demás. | — | `classic-foundation` |

## P31 ASK

17 filas; 7 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 4: AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3, AC-ASK-003 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 4: AC-ASK-001.1, AC-ASK-001.3, AC-ASK-006, AC-ASK-009.2 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 3: AC-ASK-006, AC-ASK-009.1, AC-ASK-009.3 | — | — | `transversal` |
| Q-ASK-1 | propuesta | 9: AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3, AC-ASK-003, AC-ASK-004, AC-ASK-005 … (+3) | Bloquea el diseno de la firma publica de ask/gracefulStop/after/retry (AC-ASK-001.*, 003, 004, 007, 008, 010); puede avanzar la validacion de entradas (AC-ASK-001.2 A3-A5) y los textos con tests de unidad sin runtime. | — | `classic-patterns` |
| Q-ASK-10 | propuesta | 2: AC-ASK-009.1, AC-ASK-009.2 | — | — | `classic-patterns` |
| Q-ASK-2 | propuesta | 2: AC-ASK-002, AC-ASK-010 | Bloquea solo AC-ASK-010 A5; AC-ASK-002 puede avanzar. | — | `classic-patterns` |
| Q-CS-7 | propuesta | 1: AC-ASK-008 | Bloquea AC-CS-004.2 A1, AC-FSM-004/007 textos; puede avanzar todo lo demas. | — | `classic-patterns` |

## P32 CIRCUIT-BREAKER-BACKOFF

15 filas; 13 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 8 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-ASK-013 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 2: AC-ASK-013, AC-ASK-014 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-ASK-015.1 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-ASK-011.1 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 5: AC-ASK-012, AC-ASK-015.1, AC-ASK-015.2, AC-ASK-015.3, AC-ASK-015.6 | — | — | `transversal` |
| Q-ASK-10 | propuesta | 3: AC-ASK-011.3, AC-ASK-015.1, AC-ASK-016 | — | — | `classic-patterns` |
| Q-ASK-3 | propuesta | 5: AC-ASK-011.1, AC-ASK-011.3, AC-ASK-011.4, AC-ASK-011.5, AC-ASK-013 | Bloquea AC-ASK-011.3 A3 y AC-ASK-013 (defaults de config); puede avanzar 011.1, 011.2 y los callbacks (011.4) con maxResetTimeout alto. | — | `classic-patterns` |
| Q-ASK-4 | propuesta | 3: AC-ASK-011.2, AC-ASK-011.5, AC-ASK-012 | Bloquea AC-ASK-011.2 A5 y AC-ASK-012 A6; el resto puede avanzar. | — | `classic-patterns` |
| Q-ASK-5 | propuesta | 1: AC-ASK-011.4 | — | — | `classic-patterns` |
| Q-ASK-6 | propuesta | 1: AC-ASK-014 | — | — | `classic-patterns` |
| Q-ASK-7 | propuesta | 6: AC-ASK-015.1, AC-ASK-015.2, AC-ASK-015.3, AC-ASK-015.4, AC-ASK-015.5, AC-ASK-015.6 | Bloquea AC-ASK-015.2, .3 y .6 (necesitan directiva por fallo del hijo de AC-SUP-001); pueden avanzar AC-ASK-015.1 (opciones y validaciones), .4 (mensajes mientras detenido, parcialmente) y .5 (finalStopMessage) con un hijo que… | — | `classic-patterns` |
| Q-ASK-8 | propuesta | 1: AC-ASK-016 | Bloquea solo la implementacion de AC-ASK-016; si se acepta la exclusion la fila se cierra sin impl. Nada mas depende de esta decision. | — | `classic-patterns` |
| Q-ASK-9 | propuesta | 1: AC-ASK-011.5 | Bloquea solo la implementacion de AC-ASK-017. | — | `classic-patterns` |

## P33 FSM

11 filas; 9 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-FSM-4 | accepted_with_changes (2026-11-18): aceptar (a) replicar. Condición: antes, grabar un escenario del oráculo contra 2.0.0 que lo confirme; si el oráculo contradice la lectura del código, gana el… | 2: AC-FSM-006.1, AC-FSM-006.2 | Bloquea AC-FSM-006.1 A1/A4 y AC-FSM-006.2 (A1 depende del orden de terminate); puede avanzar el resto de FSM. Solo hace falta el Owner para elegir replicar (recomendado) o detener el actor tambien. | — | `classic-patterns` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-FSM-002 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 2: AC-FSM-003, AC-FSM-004 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 2: AC-FSM-006.1, AC-FSM-009 | — | — | `transversal` |
| Q-CS-7 | propuesta | 2: AC-FSM-004, AC-FSM-007 | Bloquea AC-CS-004.2 A1, AC-FSM-004/007 textos; puede avanzar todo lo demas. | — | `classic-patterns` |
| Q-FSM-1 | propuesta | 5: AC-FSM-001.1, AC-FSM-001.2, AC-FSM-002, AC-FSM-008, AC-FSM-009 | Bloquea el diseno de AC-FSM-001.1/.2, 002, 008, 009; la logica de State inmutable y de timers puede prototiparse sin ella. | — | `classic-patterns` |
| Q-FSM-2 | propuesta | 2: AC-FSM-003, AC-FSM-004 | Bloquea AC-FSM-003 y AC-FSM-004; no bloquea el resto de FSM. | — | `classic-patterns` |
| Q-FSM-3 | propuesta | 2: AC-FSM-005, AC-FSM-007 | — | — | `classic-patterns` |
| Q-RT-8 | propuesta | 1: AC-FSM-002 | Bloquea AC-RT-015 y AC-FSM-002 A10 solo en el orden. | — | `classic-patterns` |

## P34 EVT-LOGGING

11 filas; 9 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-EVT-1 | accepted (2026-11-18): aceptar, incluida la supresión de dead letters durante el shutdown terminante de 2.0.0. | 4: AC-EVT-009, AC-EVT-013.3, AC-EVT-018, AC-EVT-019 | Bloquea: la comparación textual exacta de AC-EVT-006/008/009/010/013.3/018/019; la lógica avanza con `message_type_name`. | — | `classic-system` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-EVT-011 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-EVT-019 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 2: AC-EVT-005, AC-EVT-007 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 4: AC-EVT-009, AC-EVT-011, AC-EVT-017, AC-EVT-020 | — | — | `transversal` |
| Q-EVT-2 | propuesta | 2: AC-EVT-011, AC-EVT-017 | Bloquea: AC-EVT-011/017 en su validación diferencial; el resto avanza. | — | `classic-system` |
| Q-EVT-4 | propuesta | 1: AC-EVT-021 | Bloquea: nada; AC-EVT-021 avanza detrás de la feature `log-bridge`. | — | `classic-system` |
| Q-EVT-6 | propuesta | 1: AC-EVT-013.3 | Bloquea: nada. | — | `classic-system` |
| Q-EVT-7 | propuesta | 1: AC-EVT-019 | Bloquea: AC-EVT-019 (y el log de FSM con debug.fsm); relacionada con Q-X-3. | — | `classic-system` |

## P35 RT-NUCLEO

9 filas; 6 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-RT-001 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-RT-003.1 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-RT-002 | — | — | `transversal` |
| Q-RT-1 | propuesta | 5: AC-RT-001, AC-RT-002, AC-RT-005, AC-RT-006, AC-RT-007 | Bloquea AC-RT-001, 002, 005, 006, 007; la logica pura (RoundRobin/Random/Broadcast select) puede avanzar. | — | `classic-patterns` |
| Q-RT-2 | propuesta | 3: AC-RT-003.1, AC-RT-003.2, AC-RT-003.3 | Bloquea AC-RT-003.1-.3. | — | `classic-patterns` |
| Q-RT-8 | propuesta | 1: AC-RT-015 | Bloquea AC-RT-015 y AC-FSM-002 A10 solo en el orden. | — | `classic-patterns` |

## P36 RT-ESTRATEGIAS

9 filas; 8 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-MBX-6 | accepted (2026-11-20): aceptar (b): pool de hilos propio, coherente con Q-X-4 (A); el turno de mailbox es un trabajo del pool; `throughput` y `throughput-deadline-time` son… | 1: AC-DSP-012 | Bloquea la implementación de AC-MBX-002 y AC-DSP-002/003 hasta decidir Q-X-4; el diseño del runner con reloj inyectable puede avanzar. | — | `classic-dispatch` |
| Q-RT-4 | accepted (2026-11-18): aceptar (a), como vista interna (no API pública estable) en AC-MBX-019. | 1: AC-RT-008 | Bloquea la integracion de AC-RT-008, 013.2 y 014 (reportMessageCount); NO bloquea la logica pura de AC-RT-013.1 (capacity/rampup/backoff), de AC-RT-014 (resize/optimize/explore con carga inyectada) ni los constructores. Es el… | — | `classic-patterns` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-RT-009.1 | — | — | `transversal` |
| Q-ASK-1 | propuesta | 1: AC-RT-010 | Bloquea el diseno de la firma publica de ask/gracefulStop/after/retry (AC-ASK-001.*, 003, 004, 007, 008, 010); puede avanzar la validacion de entradas (AC-ASK-001.2 A3-A5) y los textos con tests de unidad sin runtime. | — | `classic-patterns` |
| Q-DSP-1 | propuesta | 1: AC-DSP-012 | Bloquea la implementación de AC-DSP-008/009/013/014/019; puede avanzar: lectura/validación de claves y las funciones puras `scaledPoolSize` y `resolveMinimumRunnable`. | — | `classic-dispatch` |
| Q-RT-5 | propuesta | 3: AC-RT-009.1, AC-RT-009.2, AC-RT-009.3 | Bloquea la paridad exacta de AC-RT-009.1 A2; AC-RT-009.2/.3 (puras) pueden avanzar y validarse contra el oraculo. | — | `classic-patterns` |
| Q-RT-6 | propuesta | 2: AC-RT-010, AC-RT-011 | Bloquea solo AC-RT-011 A5; el resto de RT-010/011 puede avanzar. | — | `classic-patterns` |
| Q-RT-7 | propuesta | 1: AC-RT-012 | Bloquea AC-RT-012; sin AC-DSP-012 no hay forma de validar el reparto, si de los nombres de dispatcher. | — | `classic-patterns` |

## P37 RT-CONFIG

9 filas; 6 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-RT-4 | accepted (2026-11-18): aceptar (a), como vista interna (no API pública estable) en AC-MBX-019. | 3: AC-RT-013.1, AC-RT-013.2, AC-RT-014 | Bloquea la integracion de AC-RT-008, 013.2 y 014 (reportMessageCount); NO bloquea la logica pura de AC-RT-013.1 (capacity/rampup/backoff), de AC-RT-014 (resize/optimize/explore con carga inyectada) ni los constructores. Es el… | — | `classic-patterns` |
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 4: AC-RT-004, AC-RT-013.3, AC-RT-014, AC-RT-016 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 3: AC-CFG-012.3, AC-CFG-014, AC-RT-016 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-RT-014 | — | — | `transversal` |
| Q-RT-3 | propuesta | 3: AC-RT-004, AC-RT-013.3, AC-RT-016 | Bloquea AC-RT-004, 013.3, 016 y la creacion de AC-RT-014 desde configuracion; los constructores programaticos pueden avanzar. | — | `classic-patterns` |
| Q-RT-9 | propuesta | 1: AC-RT-014 | Bloquea solo la validacion diferencial de empates (AC-RT-014 A9); no bloquea nada mas. | — | `classic-patterns` |

## P38 CS-NUCLEO

13 filas; 10 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 7 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-CS-001.2 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-CS-004.1 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-CS-004.2 | — | — | `transversal` |
| Q-ASK-10 | propuesta | 1: AC-CS-003.1 | — | — | `classic-patterns` |
| Q-CS-1 | propuesta | 3: AC-CS-001.1, AC-CS-001.2, AC-CS-003.1 | Bloquea AC-CS-001.1/.2, 003.1; puede avanzar el resto. | — | `classic-patterns` |
| Q-CS-2 | propuesta | 6: AC-CS-003.2, AC-CS-004.1, AC-CS-004.2, AC-CS-004.3, AC-CS-008, AC-CS-009 | Bloquea AC-CS-003.2, 004.1-.3, 008, 009. | — | `classic-patterns` |
| Q-CS-3 | propuesta | 1: AC-CS-002 | Bloquea AC-CS-002. | — | `classic-patterns` |
| Q-CS-4 | propuesta | 1: AC-CS-005 | Bloquea AC-CS-005 A3. | — | `classic-patterns` |
| Q-CS-6 | propuesta | 2: AC-CS-007.1, AC-CS-007.2 | Bloquea AC-CS-007.1/.2; el resto de CS no depende. | — | `classic-patterns` |
| Q-CS-7 | propuesta | 2: AC-CS-004.2, AC-CS-008 | Bloquea AC-CS-004.2 A1, AC-FSM-004/007 textos; puede avanzar todo lo demas. | — | `classic-patterns` |

## P39 TERMINACION

10 filas; 9 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-SYS-022 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-SYS-021 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 3: AC-SYS-004, AC-SYS-005, AC-SYS-006 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-SYS-015 | — | — | `transversal` |
| Q-CS-5 | propuesta | 2: AC-CS-006.1, AC-CS-006.2 | Bloquea AC-CS-006.1 A2/A3; AC-CS-006.2 puede avanzar. | — | `classic-patterns` |
| Q-SYS-1 | propuesta | 1: AC-SYS-014 | Bloquea: sólo AC-SYS-014 (política de Drop); puede avanzar todo lo demás con close()/terminate() explícitos. | — | `classic-system` |
| Q-SYS-2 | propuesta | 1: AC-SYS-015 | Bloquea: AC-SYS-015; el resto avanza. | — | `classic-system` |
| Q-SYS-3 | propuesta | 1: AC-SYS-021 | Bloquea: nada (AC-SYS-020/021 pueden implementarse con (a)); si el Owner exige paridad de manifests, queda partial. | — | `classic-system` |
| Q-SYS-5 | propuesta | 1: AC-SYS-022 | Bloquea: nada; AC-SYS-022 y AC-SYS-012 avanzan con (a). | — | `classic-system` |

## P40 SER-NUCLEO

18 filas; 11 decisiones distintas en sus filas: 7 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-SER-1 | accepted (2026-11-18): aceptar (a)+(c), incluido el comportamiento de 2.0.0 con includeManifest=false. El typo "synch" se copia por compatibilidad de configuración. | 7: AC-SER-001.1, AC-SER-001.2, AC-SER-001.3, AC-SER-002, AC-SER-003.2, AC-SER-003.3 … (+1) | — | — | `classic-io-ser` |
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 2: AC-SER-010, AC-SER-011 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 3: AC-SER-001.1, AC-SER-001.2, AC-SER-005 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 2: AC-SER-002, AC-SER-003.3 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-SER-006 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AC-SER-001.1 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 1: AC-SER-004.2 | — | — | `transversal` |
| Q-SER-2 | propuesta | 4: AC-SER-003.5, AC-SER-004.1, AC-SER-004.2, AC-SER-013 | — | — | `classic-io-ser` |
| Q-SER-3 | propuesta | 1: AC-SER-001.3 | — | — | `classic-io-ser` |
| Q-SER-5 | propuesta | 1: AC-SER-006 | — | — | `classic-io-ser` |
| Q-SER-6 | propuesta | 2: AC-SER-010, AC-SER-012 | — | — | `classic-io-ser` |

## P41 SER-SISTEMA

8 filas; 6 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-SER-014 | — | — | `transversal` |
| Q-CELL-5 | propuesta | 2: AC-CELL-028, AC-CELL-029 | Bloquea: `impl` de AC-CELL-028/029 hasta que exista SER. | Puede avanzar: todo CELL. | `classic-cell` |
| Q-REF-3 | propuesta | 1: AC-REF-023 | Bloquea: `impl` de AC-REF-023 (A1–A3 se portan a serialized_actor_path/resolve_actor_ref). | Puede avanzar: todo lo demás; la serialización de ActorRef por la unidad SER es independiente. | `classic-cell` |
| Q-SER-3 | propuesta | 2: AC-SER-007.1, AC-SER-007.2 | — | — | `classic-io-ser` |
| Q-SER-4 | propuesta | 2: AC-SER-008.1, AC-SER-008.2 | — | — | `classic-io-ser` |
| Q-SER-6 | propuesta | 1: AC-SER-014 | — | — | `classic-io-ser` |

## P42 IO-TCP

16 filas; 7 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-IO-003 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-IO-002.1 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AC-IO-004.3 | — | — | `transversal` |
| Q-IO-1 | propuesta | 10: AC-IO-001, AC-IO-002.2, AC-IO-002.3, AC-IO-002.4, AC-IO-002.6, AC-IO-002.7 … (+4) | — | — | `classic-io-ser` |
| Q-IO-2 | propuesta | 6: AC-IO-002.2, AC-IO-002.3, AC-IO-002.4, AC-IO-002.5, AC-IO-002.6, AC-IO-004.4 | — | — | `classic-io-ser` |
| Q-IO-3 | propuesta | 7: AC-IO-002.1, AC-IO-002.3, AC-IO-002.5, AC-IO-002.6, AC-IO-003, AC-IO-010 … (+1) | — | — | `classic-io-ser` |
| Q-IO-4 | propuesta | 1: AC-IO-002.3 | — | — | `classic-io-ser` |

## P43 IO-UDP-DNS

19 filas; 7 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AC-IO-009.1 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 2: AC-IO-008.1, AC-IO-008.5 | — | — | `transversal` |
| Q-IO-1 | propuesta | 5: AC-IO-006.1, AC-IO-006.2, AC-IO-007, AC-IO-008.5, AC-IO-011 | — | — | `classic-io-ser` |
| Q-IO-3 | propuesta | 2: AC-IO-005, AC-IO-006.3 | — | — | `classic-io-ser` |
| Q-IO-4 | propuesta | 3: AC-IO-008.1, AC-IO-008.3, AC-IO-009.8 | — | — | `classic-io-ser` |
| Q-IO-5 | propuesta | 2: AC-IO-006.2, AC-IO-007 | — | — | `classic-io-ser` |
| Q-IO-6 | propuesta | 3: AC-IO-009.1, AC-IO-009.2, AC-IO-009.6 | — | — | `classic-io-ser` |

## P44 TK-DISPATCHER-REF

5 filas; 8 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-TK-1 | accepted (2026-11-18): aceptar (a): dispatcher inline obligatorio para el testActor. | 3: AC-TK-001.1, AC-TK-006.1, AC-TK-006.2 | Bloquea: AC-TK-001.1, AC-TK-002.1, AC-TK-006.1, AC-TK-006.2 y el port de los specs que usen testActor/TestProbe con 0 ms. | Puede avanzar: el resto de AC-SYS/AC-EVT/AC-TK y los specs con esperas no nulas (con sondeo, opción b). | `classic-system` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AC-TK-016 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-TK-006.2 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 2: AC-TK-006.1, AC-TK-006.2 | — | — | `transversal` |
| Q-TK-2 | propuesta | 1: AC-TK-006.1 | Bloquea: AC-TK-006.1 y AC-TK-007 en impl; los contratos están extraídos. | Puede avanzar: AC-TK-006.2 (la CTD no depende de esta forma de acceso) y el resto del grupo. | `classic-system` |
| Q-TK-4 | propuesta | 2: AC-TK-001.1, AC-TK-008 | Bloquea: la API async de AC-TK-001.x hasta Q-X-4. | — | `classic-system` |
| Q-TK-7 | propuesta | 1: AC-TK-006.2 | Bloquea: nada para el resto de AC-TK-006.2. | — | `classic-system` |
| Q-TK-8 | propuesta | 2: AC-TK-006.1, AC-TK-006.2 | Bloquea: AC-TK-006.1, AC-TK-006.2, AC-TK-007 en impl, hasta definir el trait Receive y el runtime. | — | `classic-system` |

## P45 TK-API

22 filas; 14 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 9 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-TK-1 | accepted (2026-11-18): aceptar (a): dispatcher inline obligatorio para el testActor. | 2: AC-TK-002.1, AC-TK-007 | Bloquea: AC-TK-001.1, AC-TK-002.1, AC-TK-006.1, AC-TK-006.2 y el port de los specs que usen testActor/TestProbe con 0 ms. | Puede avanzar: el resto de AC-SYS/AC-EVT/AC-TK y los specs con esperas no nulas (con sondeo, opción b). | `classic-system` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-TK-015 | — | — | `transversal` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 2: AC-TK-002.3, AC-TK-003 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 3: AC-TK-001.2, AC-TK-002.3, AC-TK-017 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 2: AC-TK-005.2, AC-TK-005.3 | — | — | `transversal` |
| Q-EVT-5 | propuesta | 1: AC-TK-011 | Bloquea: nada. | — | `classic-system` |
| Q-TK-10 | propuesta | 1: AC-TK-015 | Bloquea: nada; depende de la política de serialización de AC-SER. | — | `classic-system` |
| Q-TK-2 | propuesta | 1: AC-TK-007 | Bloquea: AC-TK-006.1 y AC-TK-007 en impl; los contratos están extraídos. | Puede avanzar: AC-TK-006.2 (la CTD no depende de esta forma de acceso) y el resto del grupo. | `classic-system` |
| Q-TK-3 | propuesta | 2: AC-TK-005.1, AC-TK-011 | Bloquea: nada. | — | `classic-system` |
| Q-TK-4 | propuesta | 5: AC-TK-001.2, AC-TK-001.5, AC-TK-005.2, AC-TK-009, AC-TK-013 | Bloquea: la API async de AC-TK-001.x hasta Q-X-4. | — | `classic-system` |
| Q-TK-5 | propuesta | 2: AC-TK-004, AC-TK-009 | Bloquea: nada. | — | `classic-system` |
| Q-TK-6 | propuesta | 3: AC-TK-012.1, AC-TK-012.2, AC-TK-014 | Bloquea: nada. | — | `classic-system` |
| Q-TK-8 | propuesta | 1: AC-TK-007 | Bloquea: AC-TK-006.1, AC-TK-006.2, AC-TK-007 en impl, hasta definir el trait Receive y el runtime. | — | `classic-system` |
| Q-TK-9 | propuesta | 1: AC-TK-012.2 | Bloquea: nada. | — | `classic-system` |

## P46 JAPI-ACTORES

16 filas; 4 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 2: AC-JAPI-004.4, AC-JAPI-005.3 | — | — | `transversal` |
| Q-JAPI-1 | propuesta | 13: AC-JAPI-001, AC-JAPI-002, AC-JAPI-003, AC-JAPI-004.2, AC-JAPI-004.3, AC-JAPI-005.1 … (+7) | — | — | `classic-io-ser` |
| Q-JAPI-2 | propuesta | 5: AC-JAPI-003, AC-JAPI-004.1, AC-JAPI-004.2, AC-JAPI-004.4, AC-JAPI-012 | — | — | `classic-io-ser` |
| Q-JAPI-4 | propuesta | 3: AC-JAPI-001, AC-JAPI-008, AC-JAPI-012 | — | — | `classic-io-ser` |

## P47 JAPI-EXTRA

7 filas; 8 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 7 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 2: AC-ASK-017, AC-JAPI-009.1 | — | — | `transversal` |
| Q-ASK-1 | propuesta | 1: AC-ASK-017 | Bloquea el diseno de la firma publica de ask/gracefulStop/after/retry (AC-ASK-001.*, 003, 004, 007, 008, 010); puede avanzar la validacion de entradas (AC-ASK-001.2 A3-A5) y los textos con tests de unidad sin runtime. | — | `classic-patterns` |
| Q-ASK-10 | propuesta | 1: AC-ASK-017 | — | — | `classic-patterns` |
| Q-ASK-9 | propuesta | 1: AC-ASK-017 | Bloquea solo la implementacion de AC-ASK-017. | — | `classic-patterns` |
| Q-EVT-5 | propuesta | 2: AC-EVT-022, AC-JAPI-007 | Bloquea: nada. | — | `classic-system` |
| Q-JAPI-1 | propuesta | 2: AC-JAPI-007, AC-JAPI-010 | — | — | `classic-io-ser` |
| Q-JAPI-3 | propuesta | 3: AC-JAPI-009.1, AC-JAPI-009.2, AC-JAPI-009.3 | — | — | `classic-io-ser` |
| Q-JAPI-4 | propuesta | 1: AC-JAPI-010 | — | — | `classic-io-ser` |

## P48 T-COMPORTAMIENTO-NUCLEO

21 filas; 6 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-T-BEH-1 | accepted (2026-11-18): aceptar (a). La caché de Props de 2.0.0 no se replica: es una optimización interna sin efecto observable. | 6: AT-BEH-001, AT-BEH-002, AT-BEH-004, AT-BEH-007, AT-BEH-011, AT-BEH-023 | Bloquea AT-BEH-001, 004, 007, 011, 023 y AT-SUP-005/013; puede avanzar todo lo que no construye Behaviors (SIG, REF, STASH solo-estructura). | — | `typed-core` |
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AT-REF-003 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AT-BEH-007 | — | — | `transversal` |
| Q-T-BEH-2 | propuesta | 3: AT-BEH-008.1, AT-BEH-008.2, AT-BEH-008.3 | — | — | `typed-core` |
| Q-T-BEH-4 | propuesta | 5: AT-BEH-001, AT-BEH-022, AT-REF-001, AT-REF-003, AT-REF-005 | Bloquea la firma de narrow; no bloquea tell/igualdad. | — | `typed-core` |
| Q-T-SIG-1 | propuesta | 1: AT-SIG-001 | Bloquea la API publica de Signal (AT-SIG-001/004/005); el resto avanza. | — | `typed-core` |

## P49 T-ADAPTADOR-CLASICO

19 filas; 9 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 7 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AT-CTX-003 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 12: AT-ADP-001, AT-ADP-003, AT-ADP-004, AT-ADP-006.1, AT-ADP-006.2, AT-ADP-006.3 … (+6) | — | — | `transversal` |
| Q-T-ADP-2 | propuesta | 5: AT-ADP-006.1, AT-ADP-006.2, AT-ADP-006.3, AT-ADP-006.4, AT-ADP-010 | Bloquea: nada si Q-X-8 elige (b) (las reglas pasan al interprete nativo); con Q-X-8 (a) se implementan en la capa de compatibilidad. | Puede avanzar: AT-ADP-006.* como especificacion. | `typed-extended` |
| Q-T-ADP-3 | propuesta | 4: AT-ADP-006.1, AT-ADP-006.2, AT-ADP-006.3, AT-ADP-006.4 | Bloquea: AT-ADP-002/006.2/006.4 solo si Q-X-8 elige (a). | Puede avanzar: el resto. | `typed-extended` |
| Q-T-BEH-5 | propuesta | 1: AT-CTX-016 | Bloquea AT-BEH-010 y AT-SUP-013; el resto avanza. | — | `typed-core` |
| Q-T-CTX-1 | propuesta | 1: AT-CTX-018 | Bloquea AT-CTX-011; no bloquea ask/pipeToSelf. | — | `typed-core` |
| Q-T-CTX-2 | propuesta | 1: AT-CTX-016 | — | — | `typed-core` |
| Q-T-CTX-3 | propuesta | 2: AT-CTX-002, AT-CTX-004 | Bloquea la firma de spawn/stop/watch/watchWith (AT-CTX-002/004/005/006); la logica avanza. | — | `typed-core` |
| Q-T-SYS-3 | propuesta | 1: AT-ADP-010 | Bloquea solo la rama de overflow de AT-SYS-003. | — | `typed-core` |

## P50 T-SISTEMA-TIPADO

25 filas; 16 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 11 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 2: AT-SYS-001, AT-SYS-012 | — | — | `transversal` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 3: AT-EXT-001.1, AT-EXT-002.1, AT-EXT-002.2 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 4: AT-CS-001, AT-DSP-002, AT-DSP-005, AT-SYS-007.4 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AT-SYS-010 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 1: AT-SYS-001 | — | — | `transversal` |
| Q-T-DSP-1 | propuesta | 1: AT-DSP-001 | Bloquea: nada. | Puede avanzar: AT-DSP-001/002/003/004. | `typed-extended` |
| Q-T-DSP-2 | propuesta | 2: AT-DSP-002, AT-DSP-005 | Bloquea: AT-DSP-002/005 impl hasta Q-X-4. | Puede avanzar: contratos. | `typed-extended` |
| Q-T-DSP-3 | propuesta | 1: AT-DSP-003 | Bloquea: AT-DSP-003 impl hasta Q-X-4 (modelo de mailbox). | Puede avanzar: contrato. | `typed-extended` |
| Q-T-EXT-1 | propuesta | 1: AT-EXT-002.2 | Bloquea: nada. | Puede avanzar: AT-EXT-002.1/002.2 con caracterizacion del skip. | `typed-extended` |
| Q-T-EXT-2 | propuesta | 2: AT-EXT-001.1, AT-EXT-001.3 | Bloquea: nada. | Puede avanzar: AT-EXT-001.*. | `typed-extended` |
| Q-T-EXT-3 | propuesta | 1: AT-EXT-001.2 | Bloquea: nada. | Puede avanzar: AT-EXT-001.2. | `typed-extended` |
| Q-T-SYS-1 | propuesta | 2: AT-SYS-007.3, AT-SYS-007.4 | Bloquea AT-SYS-007.4; AT-SYS-007.3 tiene contrato cerrado y su implementacion puede avanzar con formato propio. | — | `typed-core` |
| Q-T-SYS-2 | propuesta | 3: AT-SYS-006, AT-SYS-007.3, AT-SYS-012 | — | — | `typed-core` |
| Q-T-SYS-3 | propuesta | 1: AT-SYS-003 | Bloquea solo la rama de overflow de AT-SYS-003. | — | `typed-core` |
| Q-T-SYS-4 | propuesta | 1: AT-SYS-005 | Bloquea AT-SYS-005 (parte close); terminate/whenTerminated avanzan. | — | `typed-core` |
| Q-T-TMR-4 | propuesta | 2: AT-SYS-007.2, AT-SYS-009 | Bloquea la validacion determinista, no la implementacion. | — | `typed-core` |

## P51 T-ASK-TIPADO

5 filas; 4 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AT-ASK-003 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 1: AT-ASK-003 | — | — | `transversal` |
| Q-T-ASK-1 | propuesta | 2: AT-ASK-001, AT-ASK-002 | Bloquea: cierre de impl de AT-ASK-001 (mensajes). | Puede avanzar: tipo AskError, ask vivo y respuesta tardia como DeadLetter. | `typed-extended` |
| Q-T-ASK-2 | propuesta | 1: AT-ASK-004 | Bloquea: nada (la fila AT-ASK-004 es internal-accidental). | Puede avanzar: toda AT-ASK-*. | `typed-extended` |

## P52 T-OBSERVABILIDAD

14 filas; 11 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 9 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 3: AT-BEH-003, AT-EVT-001, AT-EVT-002 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 4: AT-BEH-018, AT-BEH-019, AT-LOG-001, AT-LOG-003 | — | — | `transversal` |
| Q-T-BEH-3 | propuesta | 3: AT-BEH-017, AT-BEH-018, AT-BEH-019 | Bloquea el cierre de AT-BEH-013..019 en cuanto a firma; la semantica de aroundReceive/aroundSignal y la deduplicacion isSame pueden avanzar. | — | `typed-core` |
| Q-T-BEH-6 | propuesta | 2: AT-BEH-018, AT-BEH-019 | Bloquea AT-BEH-018/019 (dependen tambien de Q-X-7); no bloquea el resto de BEH. | — | `typed-core` |
| Q-T-DSP-1 | propuesta | 1: AT-DSP-004 | Bloquea: nada. | Puede avanzar: AT-DSP-001/002/003/004. | `typed-extended` |
| Q-T-EVT-1 | propuesta | 3: AT-EVT-001, AT-EVT-003, AT-EVT-004 | Bloquea: cierre de AT-EVT-004, AT-PS-001.2, AT-RT-004. | Puede avanzar: AT-EVT-001..003 y el resto de AT-PS. | `typed-extended` |
| Q-T-LOG-1 | propuesta | 5: AT-DSP-004, AT-LOG-001, AT-LOG-002, AT-LOG-004, AT-LOG-005 | Bloquea: cierre de impl de AT-LOG-001/002, AT-DSP-004 y la captura de AT-TK-006. | Puede avanzar: contratos (ya extraidos). | `typed-extended` |
| Q-T-LOG-2 | propuesta | 1: AT-LOG-001 | Bloquea: nada. | Puede avanzar: AT-LOG-001. | `typed-extended` |
| Q-T-LOG-3 | propuesta | 1: AT-LOG-003 | Bloquea: AT-LOG-003 (depende de Q-X-7). | Puede avanzar: el resto de LOG. | `typed-extended` |
| Q-T-LOG-4 | propuesta | 1: AT-LOG-004 | Bloquea: nada. | Puede avanzar: AT-LOG-004 como verificacion del mensaje renderizado. La exclusion requiere autorizacion explicita del Owner. | `typed-extended` |
| Q-T-TK-2 | propuesta | 1: AT-LOG-002 | Bloquea: AT-TK-006.*, AT-TK-013 (impl). | Puede avanzar: contrato. | `typed-extended` |

## P53 T-INTERCEPTORES-TIMERS-STASH

21 filas; 10 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 7 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AT-BEH-016 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AT-STASH-001 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AT-STASH-005 | — | — | `transversal` |
| Q-T-BEH-2 | propuesta | 1: AT-BEH-016 | — | — | `typed-core` |
| Q-T-BEH-3 | propuesta | 4: AT-BEH-013, AT-BEH-014, AT-BEH-015, AT-BEH-016 | Bloquea el cierre de AT-BEH-013..019 en cuanto a firma; la semantica de aroundReceive/aroundSignal y la deduplicacion isSame pueden avanzar. | — | `typed-core` |
| Q-T-SIG-1 | propuesta | 1: AT-SIG-008 | Bloquea la API publica de Signal (AT-SIG-001/004/005); el resto avanza. | — | `typed-core` |
| Q-T-TMR-1 | propuesta | 3: AT-BEH-020, AT-TMR-001, AT-TMR-002 | Bloquea la firma de TimerScheduler (AT-TMR-001/002); la logica de generaciones avanza. | — | `typed-core` |
| Q-T-TMR-2 | propuesta | 1: AT-TMR-007.2 | — | — | `typed-core` |
| Q-T-TMR-3 | propuesta | 1: AT-TMR-005 | — | — | `typed-core` |
| Q-T-TMR-4 | propuesta | 3: AT-TMR-003, AT-TMR-004, AT-TMR-007.2 | Bloquea la validacion determinista, no la implementacion. | — | `typed-core` |

## P54 T-CONTEXTO-SENALES

14 filas; 8 decisiones distintas en sus filas: 4 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-3 | accepted (2026-11-18): aceptar (A). | 1: AT-CTX-011 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 3: AT-CTX-010, AT-CTX-012, AT-CTX-014 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 2: AT-SIG-007, AT-SUP-015 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AT-CTX-015 | — | — | `transversal` |
| Q-T-CTX-1 | propuesta | 1: AT-CTX-011 | Bloquea AT-CTX-011; no bloquea ask/pipeToSelf. | — | `typed-core` |
| Q-T-CTX-3 | propuesta | 2: AT-CTX-005, AT-CTX-006 | Bloquea la firma de spawn/stop/watch/watchWith (AT-CTX-002/004/005/006); la logica avanza. | — | `typed-core` |
| Q-T-SIG-1 | propuesta | 1: AT-SIG-004 | Bloquea la API publica de Signal (AT-SIG-001/004/005); el resto avanza. | — | `typed-core` |
| Q-T-TMR-2 | propuesta | 1: AT-CTX-008 | — | — | `typed-core` |

## P55 T-SUPERVISION

31 filas; 20 decisiones distintas en sus filas: 6 respondidas, 0 bloqueantes, 14 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-T-BEH-1 | accepted (2026-11-18): aceptar (a). La caché de Props de 2.0.0 no se replica: es una optimización interna sin efecto observable. | 2: AT-SUP-005, AT-SUP-013 | Bloquea AT-BEH-001, 004, 007, 011, 023 y AT-SUP-005/013; puede avanzar todo lo que no construye Behaviors (SIG, REF, STASH solo-estructura). | — | `typed-core` |
| Q-T-SUP-3 | accepted_with_changes (2026-11-18): aceptar (a) para el default (min+max)/2. Para maxRestarts <= 0 manda el código, no el Scaladoc; condición: confirmarlo con el oráculo antes de implementar… | 2: AT-SUP-007.2, AT-SUP-007.3 | Bloquea AT-SUP-007.3; AT-SUP-007.2 puede avanzar. | — | `typed-core` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 4: AT-BEH-024, AT-SUP-001, AT-SUP-002, AT-SUP-022 | — | — | `transversal` |
| Q-X-6 | accepted (2026-11-18): aceptar (A). | 1: AT-SUP-009 | — | — | `transversal` |
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 1: AT-SUP-010 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 4: AT-ADP-002, AT-ADP-005, AT-SUP-020, AT-SUP-022 | — | — | `transversal` |
| Q-T-ADP-3 | propuesta | 1: AT-ADP-002 | Bloquea: AT-ADP-002/006.2/006.4 solo si Q-X-8 elige (a). | Puede avanzar: el resto. | `typed-extended` |
| Q-T-BEH-5 | propuesta | 2: AT-BEH-010, AT-SUP-013 | Bloquea AT-BEH-010 y AT-SUP-013; el resto avanza. | — | `typed-core` |
| Q-T-CTX-2 | propuesta | 1: AT-BEH-010 | — | — | `typed-core` |
| Q-T-CTX-4 | propuesta | 1: AT-SUP-020 | Bloquea la ejecutabilidad de AT-CTX-017 y AT-SUP-020/021; el contrato esta cerrado y el resto de CTX avanza. | — | `typed-core` |
| Q-T-SIG-1 | propuesta | 2: AT-SIG-005, AT-SUP-017 | Bloquea la API publica de Signal (AT-SIG-001/004/005); el resto avanza. | — | `typed-core` |
| Q-T-STASH-1 | propuesta | 3: AT-STASH-006, AT-SUP-009, AT-SUP-018 | Bloquea AT-SUP-009/018 y AT-STASH-006 en el tipo interno; el stash publico avanza. | — | `typed-core` |
| Q-T-SUP-1 | propuesta | 2: AT-SUP-002, AT-SUP-011 | Bloquea AT-SUP-002 y AT-SUP-011. | — | `typed-core` |
| Q-T-SUP-2 | propuesta | 2: AT-SUP-007.1, AT-SUP-009 | — | — | `typed-core` |
| Q-T-SUP-4 | propuesta | 2: AT-SUP-005, AT-SUP-008 | Bloquea solo la rama ilimitada de AT-SUP-005. | — | `typed-core` |
| Q-T-SUP-5 | propuesta | 1: AT-SUP-004 | Bloquea AT-SUP-004 (forma del error); la parada y el log avanzan. | — | `typed-core` |
| Q-T-SUP-6 | propuesta | 2: AT-SUP-007.2, AT-SUP-017 | Bloquea AT-SUP-017; el backoff avanza. | — | `typed-core` |
| Q-T-SUP-7 | propuesta | 3: AT-SUP-006, AT-SUP-020, AT-SUP-021 | Bloquea AT-SUP-021 (y la rama clasica de AT-SUP-020/006); lo demas de SUP avanza. | — | `typed-core` |
| Q-T-SUP-8 | propuesta | 2: AT-SUP-006, AT-SUP-007.1 | — | — | `typed-core` |
| Q-T-TMR-4 | propuesta | 1: AT-SUP-006 | Bloquea la validacion determinista, no la implementacion. | — | `typed-core` |

## P56 T-RECEPCIONISTA-SERIALIZACION

14 filas; 7 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 6 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AT-RCP-001 | — | — | `transversal` |
| Q-T-RCP-1 | propuesta | 5: AT-RCP-002, AT-RCP-005.1, AT-RCP-005.2, AT-RCP-005.3, AT-SER-002 | Bloquea: cierre de AT-RCP-002/005.3 y AT-SER-002. | Puede avanzar: Register/Find/Subscribe con claves de tipo no generico. | `typed-extended` |
| Q-T-RCP-2 | propuesta | 1: AT-RCP-001 | Bloquea: nada. | Puede avanzar: AT-RCP-003..008 con LocalReceptionist. | `typed-extended` |
| Q-T-RCP-3 | propuesta | 4: AT-RCP-003, AT-RCP-004, AT-RCP-005.1, AT-RCP-005.2 | Bloquea: nada. | Puede avanzar: AT-RCP-003/004/005.2 con caracterizacion de estos casos. | `typed-extended` |
| Q-T-SER-1 | propuesta | 2: AT-SER-001, AT-SER-002 | Bloquea: AT-DLV-005.1-3 (impl), AT-SER-001/002 (impl) y AT-TK-009 (SerializationTestKit). | Puede avanzar: el resto de DLV y la especificacion de 005.*. | `typed-extended` |
| Q-T-SER-2 | propuesta | 1: AT-SER-002 | Bloquea: AT-SER-002 impl. | Puede avanzar: contrato. | `typed-extended` |
| Q-T-SER-3 | propuesta | 1: AT-SER-003 | Bloquea: AT-SER-003 impl. | Puede avanzar: contrato. | `typed-extended` |

## P57 T-ROUTERS-PUBSUB-SPAWN

12 filas; 5 decisiones distintas en sus filas: 0 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-T-EVT-1 | propuesta | 2: AT-PS-001.2, AT-RT-004 | Bloquea: cierre de AT-EVT-004, AT-PS-001.2, AT-RT-004. | Puede avanzar: AT-EVT-001..003 y el resto de AT-PS. | `typed-extended` |
| Q-T-RT-1 | propuesta | 3: AT-RT-002.1, AT-RT-002.2, AT-RT-002.3 | Bloquea: nada para impl; el oraculo no podra comparar nodos elegidos si se elige (b). | Puede avanzar: AT-RT-002.3 con las aserciones A1-A4. | `typed-extended` |
| Q-T-RT-2 | propuesta | 3: AT-RT-002.1, AT-RT-002.2, AT-RT-002.3 | Bloquea: nada. | Puede avanzar: AT-RT-002.1-3. | `typed-extended` |
| Q-T-RT-3 | propuesta | 4: AT-RT-001, AT-RT-003, AT-RT-004, AT-RT-005 | Bloquea: AT-RT-001/003/004/005 hasta cerrar Q-T-BEH-1 (grupo BEH). | Puede avanzar: RoutingLogic puro (AT-RT-002.*). | `typed-extended` |
| Q-T-SPV-1 | propuesta | 1: AT-SPV-001 | Bloquea: nada para contratos. | Puede avanzar: AT-SPV-001 en cuanto se fije el tipo de Spawn. | `typed-extended` |

## P58 T-ENTREGA-FIABLE-BASE

13 filas; 8 decisiones distintas en sus filas: 5 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-1 | rejected (2026-10-31): rechazar: la recomendación (a) (std::time::Duration, sólo valores positivos) NO se adopta. ALTERNATIVA ELEGIDA Y VIGENTE: preservar el dominio de Pekko, es… | 1: AT-DLV-006 | — | — | `classic-foundation` |
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 1: AT-DLV-006 | — | — | `transversal` |
| Q-X-10 | accepted (2026-11-18): autoriza el alcance (A): sólo actor y actor-typed locales (sin remoting, cluster, persistence, streams ni HTTP). | 1: AT-DLV-001.4 | — | — | `transversal` |
| Q-X-5 | accepted_with_changes (2026-11-18): aceptar (A). Documentar que con panic=abort no hay supervisión de panics. | 1: AT-DLV-001.1 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 1: AT-DLV-001.1 | — | — | `transversal` |
| Q-T-DLV-1 | propuesta | 12: AT-DLV-001.1, AT-DLV-001.2, AT-DLV-001.3, AT-DLV-001.4, AT-DLV-001.5, AT-DLV-002.1 … (+6) | Bloquea: nada para la especificacion (todas las filas DLV salvo 005.* estan extraidas); AT-DLV-005.* y 001.6/003.6 dependen de Q-T-SER-1/Q-T-DLV-2 para impl. | Puede avanzar: AT-DLV-001.1-5, 002.1-7, 003.1-5. | `typed-extended` |
| Q-T-DLV-2 | propuesta | 2: AT-DLV-001.2, AT-DLV-001.5 | Bloquea: AT-DLV-001.6, 003.6, 005.3 y 004 (implementacion) si el Owner exige persistencia real. | Puede avanzar: AT-DLV-004 (State puro) y toda la parte sin durable. | `typed-extended` |
| Q-T-DLV-4 | propuesta | 5: AT-DLV-001.3, AT-DLV-002.3, AT-DLV-002.4, AT-DLV-002.5, AT-DLV-002.6 | Bloquea: nada; condiciona la validacion de AT-DLV-001.3, 002.3, 002.4, 002.6. | Puede avanzar: todo lo demas. | `typed-extended` |

## P59 T-ENTREGA-FIABLE-DURABLE

6 filas; 5 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-10 | accepted (2026-11-18): autoriza el alcance (A): sólo actor y actor-typed locales (sin remoting, cluster, persistence, streams ni HTTP). | 1: AT-DLV-001.6 | — | — | `transversal` |
| Q-T-DLV-1 | propuesta | 4: AT-DLV-001.6, AT-DLV-005.1, AT-DLV-005.2, AT-DLV-005.3 | Bloquea: nada para la especificacion (todas las filas DLV salvo 005.* estan extraidas); AT-DLV-005.* y 001.6/003.6 dependen de Q-T-SER-1/Q-T-DLV-2 para impl. | Puede avanzar: AT-DLV-001.1-5, 002.1-7, 003.1-5. | `typed-extended` |
| Q-T-DLV-2 | propuesta | 3: AT-DLV-001.6, AT-DLV-004, AT-DLV-005.3 | Bloquea: AT-DLV-001.6, 003.6, 005.3 y 004 (implementacion) si el Owner exige persistencia real. | Puede avanzar: AT-DLV-004 (State puro) y toda la parte sin durable. | `typed-extended` |
| Q-T-DLV-3 | propuesta | 1: AT-DLV-007 | Bloquea: nada. | Puede avanzar: toda DLV (el recorder es pasivo). | `typed-extended` |
| Q-T-SER-1 | propuesta | 3: AT-DLV-005.1, AT-DLV-005.2, AT-DLV-005.3 | Bloquea: AT-DLV-005.1-3 (impl), AT-SER-001/002 (impl) y AT-TK-009 (SerializationTestKit). | Puede avanzar: el resto de DLV y la especificacion de 005.*. | `typed-extended` |

## P60 T-ENTREGA-FIABLE-WORKPULLING

6 filas; 4 decisiones distintas en sus filas: 0 respondidas, 0 bloqueantes, 4 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-T-DLV-1 | propuesta | 6: AT-DLV-003.1, AT-DLV-003.2, AT-DLV-003.3, AT-DLV-003.4, AT-DLV-003.5, AT-DLV-003.6 | Bloquea: nada para la especificacion (todas las filas DLV salvo 005.* estan extraidas); AT-DLV-005.* y 001.6/003.6 dependen de Q-T-SER-1/Q-T-DLV-2 para impl. | Puede avanzar: AT-DLV-001.1-5, 002.1-7, 003.1-5. | `typed-extended` |
| Q-T-DLV-2 | propuesta | 1: AT-DLV-003.6 | Bloquea: AT-DLV-001.6, 003.6, 005.3 y 004 (implementacion) si el Owner exige persistencia real. | Puede avanzar: AT-DLV-004 (State puro) y toda la parte sin durable. | `typed-extended` |
| Q-T-DLV-4 | propuesta | 1: AT-DLV-003.2 | Bloquea: nada; condiciona la validacion de AT-DLV-001.3, 002.3, 002.4, 002.6. | Puede avanzar: todo lo demas. | `typed-extended` |
| Q-T-RCP-1 | propuesta | 1: AT-DLV-003.1 | Bloquea: cierre de AT-RCP-002/005.3 y AT-SER-002. | Puede avanzar: Register/Find/Subscribe con claves de tipo no generico. | `typed-extended` |

## P61 T-JAVADSL

17 filas; 7 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 6 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 2: AT-CTX-017, AT-JAPI-006.5 | — | — | `transversal` |
| Q-T-BEH-6 | propuesta | 1: AT-JAPI-001.3 | Bloquea AT-BEH-018/019 (dependen tambien de Q-X-7); no bloquea el resto de BEH. | — | `typed-core` |
| Q-T-CTX-2 | propuesta | 1: AT-CTX-017 | — | — | `typed-core` |
| Q-T-CTX-4 | propuesta | 1: AT-CTX-017 | Bloquea la ejecutabilidad de AT-CTX-017 y AT-SUP-020/021; el contrato esta cerrado y el resto de CTX avanza. | — | `typed-core` |
| Q-T-JAPI-1 | propuesta | 16: AT-JAPI-001.1, AT-JAPI-001.2, AT-JAPI-001.3, AT-JAPI-002.1, AT-JAPI-002.2, AT-JAPI-003 … (+10) | Bloquea: impl de AT-JAPI-001.2, 002.1, 002.2, 003 y 004 (dependen de Q-T-JAPI-2). | Puede avanzar: AT-JAPI-001.1, 005.1, 006.2-006.5 (alias de filas AT-*). | `typed-extended` |
| Q-T-JAPI-2 | propuesta | 5: AT-JAPI-001.2, AT-JAPI-002.1, AT-JAPI-002.2, AT-JAPI-003, AT-JAPI-004 | Bloquea: impl de AT-JAPI-001.2, 002.1, 002.2, 003 y 004. | Puede avanzar: el resto de JAPI y toda la API Rust de scaladsl. | `typed-extended` |
| Q-T-TMR-3 | propuesta | 1: AT-JAPI-006.1 | — | — | `typed-core` |

## P62 T-TK-SONDA

17 filas; 7 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 5 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-1 | accepted_with_changes (2026-11-18): aceptar (A) (parser HOCON propio). Condición: antes de implementar P07, ampliar el golden con las claves del reference.conf de 2.0.0. | 2: AT-TK-002.6, AT-TK-010 | — | — | `transversal` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AT-TK-007 | — | — | `transversal` |
| Q-T-EVT-1 | propuesta | 1: AT-TK-008 | Bloquea: cierre de AT-EVT-004, AT-PS-001.2, AT-RT-004. | Puede avanzar: AT-EVT-001..003 y el resto de AT-PS. | `typed-extended` |
| Q-T-SER-1 | propuesta | 4: AT-TK-009.1, AT-TK-009.2, AT-TK-009.3, AT-TK-009.4 | Bloquea: AT-DLV-005.1-3 (impl), AT-SER-001/002 (impl) y AT-TK-009 (SerializationTestKit). | Puede avanzar: el resto de DLV y la especificacion de 005.*. | `typed-extended` |
| Q-T-TK-1 | propuesta | 9: AT-TK-001.2, AT-TK-001.3, AT-TK-002.1, AT-TK-002.2, AT-TK-002.3, AT-TK-002.4 … (+3) | Bloquea: nombrar la API de TestProbe e impl de AT-TK-001.*/002.* hasta Q-X-4. | Puede avanzar: AT-TK-004.*, 005.*, 006.1/006.2 y los contratos. | `typed-extended` |
| Q-T-TK-3 | propuesta | 5: AT-TK-001.1, AT-TK-009.1, AT-TK-009.2, AT-TK-009.3, AT-TK-009.4 | Bloquea: AT-TK-009.1-009.3 (impl). | Puede avanzar: contratos y 009.4 (SerializationTestKit, ver Q-T-SER-1). | `typed-extended` |
| Q-T-TMR-4 | propuesta | 1: AT-TK-007 | Bloquea la validacion determinista, no la implementacion. | — | `typed-core` |

## P63 T-TK-COMPORTAMIENTO

19 filas; 9 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 7 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 2: AT-TK-003.5, AT-TK-011 | — | — | `transversal` |
| Q-X-8 | accepted (2026-11-18): aceptar (a): Typed sobre Classic. | 4: AT-TK-003.1, AT-TK-003.2, AT-TK-011, AT-TK-012 | — | — | `transversal` |
| Q-T-CTX-1 | propuesta | 1: AT-TK-004.4 | Bloquea AT-CTX-011; no bloquea ask/pipeToSelf. | — | `typed-core` |
| Q-T-JAPI-1 | propuesta | 1: AT-TK-004.7 | Bloquea: impl de AT-JAPI-001.2, 002.1, 002.2, 003 y 004 (dependen de Q-T-JAPI-2). | Puede avanzar: AT-JAPI-001.1, 005.1, 006.2-006.5 (alias de filas AT-*). | `typed-extended` |
| Q-T-LOG-1 | propuesta | 1: AT-TK-003.5 | Bloquea: cierre de impl de AT-LOG-001/002, AT-DSP-004 y la captura de AT-TK-006. | Puede avanzar: contratos (ya extraidos). | `typed-extended` |
| Q-T-TK-1 | propuesta | 2: AT-TK-003.3, AT-TK-005.1 | Bloquea: nombrar la API de TestProbe e impl de AT-TK-001.*/002.* hasta Q-X-4. | Puede avanzar: AT-TK-004.*, 005.*, 006.1/006.2 y los contratos. | `typed-extended` |
| Q-T-TK-4 | propuesta | 19: AT-TK-003.1, AT-TK-003.2, AT-TK-003.3, AT-TK-003.4, AT-TK-003.5, AT-TK-003.6 … (+13) | Bloquea: AT-TK-003.*, 004.*, 005.*, 011, 012 (impl) hasta Q-X-8. | Puede avanzar: contrato. | `typed-extended` |
| Q-T-TK-5 | propuesta | 1: AT-TK-004.6 | Bloquea: nada. | Puede avanzar: AT-TK-004.6 con la caracterizacion; cualquier correccion es divergencia y requiere al Owner. | `typed-extended` |
| Q-T-TMR-1 | propuesta | 1: AT-TK-004.6 | Bloquea la firma de TimerScheduler (AT-TMR-001/002); la logica de generaciones avanza. | — | `typed-core` |

## P64 T-TK-LOGGING

6 filas; 4 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 3 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-X-7 | accepted (2026-11-18): aceptar (A). | 4: AT-TK-006.3, AT-TK-006.4, AT-TK-006.5, AT-TK-013 | — | — | `transversal` |
| Q-T-LOG-1 | propuesta | 6: AT-TK-006.1, AT-TK-006.2, AT-TK-006.3, AT-TK-006.4, AT-TK-006.5, AT-TK-013 | Bloquea: cierre de impl de AT-LOG-001/002, AT-DSP-004 y la captura de AT-TK-006. | Puede avanzar: contratos (ya extraidos). | `typed-extended` |
| Q-T-TK-2 | propuesta | 6: AT-TK-006.1, AT-TK-006.2, AT-TK-006.3, AT-TK-006.4, AT-TK-006.5, AT-TK-013 | Bloquea: AT-TK-006.*, AT-TK-013 (impl). | Puede avanzar: contrato. | `typed-extended` |
| Q-T-TK-3 | propuesta | 1: AT-TK-013 | Bloquea: AT-TK-009.1-009.3 (impl). | Puede avanzar: contratos y 009.4 (SerializationTestKit, ver Q-T-SER-1). | `typed-extended` |

## P65 UTIL-HASHCODE

1 filas; 3 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-PATH-3 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-016.15 | Bloquea: nada. | — | `classic-foundation` |
| Q-UTIL-14 | accepted (2026-11-18): aceptar (a) para FrequencySketch, incluida la validación de desbordamiento de 2.0.0, y (b) para el resto. | 1: AC-UTIL-016.15 | Bloquea: AC-UTIL-017.1 (impl exacta); puede avanzar: el resto de AC-UTIL-017.* y AC-UTIL-006.*. | — | `classic-foundation` |
| Q-UTIL-8 | accepted (2026-11-18): aceptar (c). Autoriza no reproducir makePattern bit a bit. | 1: AC-UTIL-016.15 | Bloquea: nada si se adopta (a) o (c) con Glob como referencia; el Owner sólo debe autorizar que makePattern NO se reproduzca bit a bit (AC-UTIL-023). | — | `classic-foundation` |

## P66 UTIL-PEKKOVERSION

1 filas; 2 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-5 | accepted_with_changes (2026-11-18): aceptar (a) con el valor "2.0.0" (la referencia pasó de 1.7.1 a 2.0.0, Q-X-11). | 1: AC-UTIL-005.1 | Bloquea: AC-CFG-003, AC-UTIL-005.1; puede avanzar: el resto. | — | `classic-foundation` |
| Q-UTIL-17 | accepted (2026-11-18): aceptar (a): Display de PekkoException y ConfigurationException muestra sólo el mensaje; java_class() queda aparte. | 1: AC-UTIL-005.1 | — | — | `classic-foundation` |

## P67 UTIL-VERSION

1 filas; 1 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-6 | rejected (2026-11-18): rechazar (a) bug-compatible de 1.7.1. Elegido: seguir el comportamiento corregido de 2.0.0 (1.2.0 > 1.2.0-M1). Condición: medirlo con el jar 2.0.0 antes de… | 1: AC-UTIL-005.2 | Bloquea: la implementación de la comparación con calificador (AC-UTIL-005.2 A5/A8) y la elección de la versión más alta en AC-UTIL-016.9; puede avanzar: parseo, igualdad, comparación numérica y dynver (A1-A4, A6-A7). | — | `classic-foundation` |

## P68 UTIL-PRETTYDURATION

1 filas; 1 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-9 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-016.5 | Bloquea: nada; puede avanzar: AC-UTIL-016.5 con (a) y el golden por crear. | — | `classic-foundation` |

## P69 UTIL-UUID-ORDEN

1 filas; 2 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 1: AC-UTIL-016.14 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-X-18 | accepted (2026-11-18): aceptar: todo el código de rakka derivado de fuentes con licencia no compatible con Apache-2.0, o de procedencia dudosa, se reescribe en sala limpia… | 1: AC-UTIL-016.14 | — | — | `transversal` |

## P70 UTIL-RELOJ

1 filas; 0 decisiones distintas en sus filas: 0 respondidas, 0 bloqueantes, 0 propuestas.

Sin decisiones asociadas a sus filas en `contract-map.tsv`.

## P71 UTIL-REGISTRO-FABRICAS

2 filas; 3 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-8 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-016.7 | Bloquea: AC-CFG-006 y AC-EXT-003; puede avanzar: AC-EXT-001.*, AC-EXT-002. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 2: AC-UTIL-016.13, AC-UTIL-016.7 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-UTIL-016.7 | — | — | `transversal` |

## P72 UTIL-MANIFEST

1 filas; 3 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-5 | accepted_with_changes (2026-11-18): aceptar (a) con el valor "2.0.0" (la referencia pasó de 1.7.1 a 2.0.0, Q-X-11). | 1: AC-UTIL-016.9 | Bloquea: AC-CFG-003, AC-UTIL-005.1; puede avanzar: el resto. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 1: AC-UTIL-016.9 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-UTIL-6 | rejected (2026-11-18): rechazar (a) bug-compatible de 1.7.1. Elegido: seguir el comportamiento corregido de 2.0.0 (1.2.0 > 1.2.0-M1). Condición: medirlo con el jar 2.0.0 antes de… | 1: AC-UTIL-016.9 | Bloquea: la implementación de la comparación con calificador (AC-UTIL-005.2 A5/A8) y la elección de la versión más alta en AC-UTIL-016.9; puede avanzar: parseo, igualdad, comparación numérica y dynver (A1-A4, A6-A7). | — | `classic-foundation` |

## P73 UTIL-JFR

2 filas; 2 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-7 | accepted (2026-11-18): aceptar tal cual la recomendación (lista ACEPTAR EN BLOQUE del lote 01). Criterio general del Owner: paridad con Pekko 2.0.0; donde 2.0.0 cambió respecto de… | 1: AC-UTIL-016.11 | Bloquea: AC-CFG-007 y AC-UTIL-016.11 (impl); puede avanzar: aceptar y validar las claves. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 2: AC-UTIL-016.10, AC-UTIL-016.11 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |

## P74 UTIL-TIPOS-DESERIALIZACION

1 filas; 3 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-CFG-11 | accepted_with_changes (2026-11-18): aceptar las claves; con la opción en `on` el arranque falla con un error de configuración (no un no-op silencioso) hasta que exista AC-SER. | 1: AC-UTIL-016.12 | Bloquea: AC-CFG-010 (y AC-UTIL-006.8); puede avanzar: aceptar las claves en `off`. | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 1: AC-UTIL-016.12 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-X-2 | accepted_with_changes (2026-11-18): aceptar (A). El error de nombre no registrado es ConfigurationException del crate, como en Pekko. | 1: AC-UTIL-016.12 | — | — | `transversal` |

## P75 UTIL-ORIGEN-CLOSURES

1 filas; 2 decisiones distintas en sus filas: 2 respondidas, 0 bloqueantes, 0 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 1: AC-UTIL-016.8 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-UTIL-16 | accepted (2026-11-18): aceptar. | 1: AC-UTIL-016.8 | Bloquea: AC-UTIL-016.8 (impl) y los tests de toString de Typed; puede avanzar: el resto. | — | `classic-foundation` |

## P76 UTIL-EQUIVALENCIAS

5 filas; 2 decisiones distintas en sus filas: 1 respondidas, 0 bloqueantes, 1 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 5: AC-UTIL-016.1, AC-UTIL-016.18, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-019 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-UTIL-20 | propuesta | 5: AC-UTIL-016.1, AC-UTIL-016.18, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-019 | — | — | `classic-foundation` |

## P77 UTIL-FRONTERAS

3 filas; 5 decisiones distintas en sus filas: 3 respondidas, 0 bloqueantes, 2 propuestas.

| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |
|---|---|---|---|---|---|
| Q-UTIL-1 | rejected (2026-10-31): rechazar: la recomendación (a) (std::time::Duration, sólo valores positivos) NO se adopta. ALTERNATIVA ELEGIDA Y VIGENTE: preservar el dominio de Pekko, es… | 1: AC-UTIL-018.2 | — | — | `classic-foundation` |
| Q-UTIL-11 | accepted_with_changes (2026-11-18): aceptar (a) con la lista actualizada a 2.0.0. ReentrantGuard y ccompat se marcan como "eliminado en 2.0.0", no como exclusión. Crear filas para Clock y… | 3: AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3 | Bloquea: nada; puede avanzar: el resto con (a). | — | `classic-foundation` |
| Q-X-4 | accepted_with_changes (2026-11-18): aceptar (A). virtualize y virtual-thread-start-number se aceptan y se ignoran con un warning. minimum-runnable sigue el default de 2.0.0 (-1). | 1: AC-UTIL-018.1 | — | — | `transversal` |
| Q-DSP-3 | propuesta | 1: AC-UTIL-018.3 | Bloquea sólo AC-DSP-017.2. | — | `classic-dispatch` |
| Q-UTIL-27 | propuesta | 3: AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3 | — | — | `classic-foundation` |

## Filas que citan una agrupación sin miembros concretos (pendientes de revisión)

Una agrupación (estado `group`) no admite respuesta ni impide cerrar. Estas filas la citan pero ninguna de sus decisiones individuales es un miembro. **No se les asigna ningún miembro automáticamente** y haberlas desligado de la agrupación no prueba que no necesiten decisiones individuales: su revisión semántica está pendiente.

- Q-X-9 (Q-UTIL-1, Q-UTIL-2, Q-UTIL-3, Q-UTIL-4, Q-UTIL-5): 8 filas: AC-ASK-001.1, AC-ASK-005, AC-CS-003.1, AC-UTIL-005.2, AT-ASK-001, AT-ASK-002, AT-CTX-012, AT-DLV-003.3

## Totales

Total de pares unidad–decisión: 524 (253 respondidas, 0 bloqueantes, 271 propuestas).
