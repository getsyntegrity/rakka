# Contradicciones, brechas y evidencia pendiente de la transferencia

Registro de lo que se encontró al transferir (SHA goPekko `8ad83a6`, Rakka `develop` `fb7a744`). Es un registro de **esta transferencia**, no una auditoría nueva de Rakka ni de goPekko: las auditorías importadas siguen siendo las de goPekko, con su fecha y SHA.

## Versión de Pekko

| # | Hallazgo | Evidencia | Tratamiento |
|---|---|---|---|
| V1 | goPekko y Rakka fijan el mismo commit de Pekko (`v1.7.1`, `9a54d129`) | `docs/parity/actor-inventory.md` y ADR 0031 (goPekko); `docs/upstream.md` (Rakka) | Se acepta como pin común; se verifica por documento |
| V2 | **Un oráculo de goPekko se ejecutó contra otra rama de Pekko:** `apache/pekko` rama `1.8.x`, HEAD `59d8040885ea17675ad63aebc9f746e687306934` (`v1.8.0-M0-1-g59d8040`) | `docs/parity/oracles/d-a-restart-setup-unbounded-2026-11-04.md:261` | No se asume equivalencia con 1.7.1. Su resultado no sirve como golden de Rakka hasta repetirse con los jars de 1.7.1 |
| V3 | Los ADR 0026/0027 citan `pekko-projection v1.1.0` (repositorio aparte) | ADR 0026:77, 0027:65 | Fuera de alcance (lectura/proyecciones); excluidos |
| V4 | El oráculo de Rakka usa jars publicados + JDK 11 o Pekko compilado de `upstream/pekko` (`tools/oracle/run.sh`); el de goPekko usa copias desechables o JDK propio | `docs/upstream.md`; manifests de goPekko | Cada observación se rotula con su origen; sin equivalencia supuesta |

## Estados y fechas que no deben leerse como estado de Rakka

| # | Hallazgo | Tratamiento |
|---|---|---|
| E1 | `docs/judge.md` y `docs/parity/actor-inventory.md` se declaran *proposed* (2026-10-08 y 2026-10-24) aunque gobiernan el trabajo de goPekko | Copiados con su estado; el índice lo expone en `declared_status` |
| E2 | Las columnas «Go state» y los niveles V del inventario de goPekko valen sólo en los SHAs que el propio documento nombra (`a3aa35d`, `9abfb40`, `537fd19`) y mientras su verificador informe «complete» | No se importan como estados Rakka |
| E3 | ADR 0031 de goPekko dice «No code implements it yet» y lo reconcilia en una nota posterior; sus adendas #1186/#1187 son *PROPOSAL* no aplicadas y las enmiendas del 2026-11-04 las revisan | Conservadas tal cual; precedencia documentada en `docs/research/gopekko-sources.md` |
| E4 | ADR 0033 define «100 %» como capacidades **locales** del inventario, excluyendo cluster y remoting; Rakka define «100 %» como todas las filas de los dos TSV con `impl=done` o exclusión del Owner | Alcances distintos: el de goPekko **no** se importa |
| E5 | goPekko usa ids propios (`C-01`, `IDN-03`, `#812`) y unidades por historia; Rakka usa `AC-*`/`AT-*` y `P01..P77` | No hay mapeo validado; `rakka_unit_hint` es una pista por tema |

## Referencias incompletas

| # | Hallazgo | Detalle |
|---|---|---|
| R1 | 666 enlaces relativos en los `.md` copiados; 164 (en 68 documentos) no resuelven a un archivo del árbol de origen | La mayoría son falsos positivos de la expresión de búsqueda (fragmentos de código Go como `](ctx,`); **no se depuraron uno a uno**. Los destinos reales que no se copiaron se resuelven con `blob/<SHA>/<ruta>` |
| R2 | Documentos que citan líneas de código Go (`pekko-actor/actor/*.go:NN`) | Válidas sólo en el SHA citado por cada documento, no en `8ad83a6` |
| R3 | goPekko referencia el submódulo `migration-kit` y `upstream/pekko` | No se copian: `upstream/` ya está en Rakka; el kit es proceso de goPekko |

## Propuestas pendientes de goPekko (no integradas, no copiadas)

PR abiertos en `getsyntegrity/gopekko` al 2026-11-14, todos draft. Los que tocan Typed (restart con stash, supervisión) son relevantes y se pueden transferir cuando se integren; hasta entonces **no son fuente**.

| PR | HEAD | Título |
|---|---|---|
| #1582 | `14c013c1a88c414b6cd8c78324f33cd679578a71` | test(typed): keep the restart stash through a failing Setup (U4a of #999) |
| #1581 | `c18125b5dc6fa1cbc72b1c1b6c3f046927055980` | docs(parity): O2e oracle of restart stash gaps, B2 and conclusions |
| #1580 | `ac6d9bc0e140f4a070669327c8e0abc884c18bbc` | docs(parity): O2d oracle … error during the replay (B1, B3) |
| #1579 | `4232d036af30608953248f239145753bdd212070` | docs(parity): O2c oracle … stop with capacity 0 (C0, C4) |
| #1578 | `73b0a640dd92498d2a9fcfd98a499d992a6198d7` | docs(parity): O2b oracle … SF1 and SF2 |
| #1577 | `ef7ca42102d4fb799bdee7be52d6f8d95a1d82b4` | docs(parity): O2a oracle … the mount of SupervisionSpec 1232 (SF0) |
| #1576 | `d0123d2700aceb466b02a254b4c948a8c0dc0629` | docs(parity): O2d oracle … B2 and conclusions |
| #1575 | `0f6c8730f116b492bfdbe4eb1ba3d71332bba8ff` | docs(parity): O2c oracle … error during the replay (B1, B3) |
| #1574 | `c47ad57e6b2e4eb74c1ef1e83391d30f957a3cc8` | docs(parity): O2b oracle … SF2 and stop with capacity 0 (C0, C4) |
| #1573 | `0ba48c5eaad6152d7e3fe5af6913b10ece0a413f` | docs(parity): O2a oracle … Setup failure (SF1) |
| #1572 | `fe70e45dafb8c14817fc195cbf345f67c620ddac` | docs(typed): audit #999 restart stash and reconcile S7 manifest rows |

(#1573-#1576 y #1577-#1581 son dos series, ramas `evidence/999-o2*` y `evidence/999-o2v2*`, con títulos solapados; no se resolvió cuál supera a cuál.)

## Lo no leído

Se copiaron 571 archivos verificados byte a byte, pero quien transfirió **no leyó** su contenido salvo: ADR 0031 y 0037 completos; cabeceras y tablas de decisión de los ADR en alcance; cabecera de `docs/judge.md` y del inventario `actor-inventory.md`; `docs/scopes/852-*`; el manifest de 852. No se leyeron: auditorías, cierres y auditorías de diseño Typed, escenarios del juez, registros de migración, `pekko-repro` ni resúmenes de oráculo (salvo la línea de V2). Cualquier afirmación de esos documentos es afirmación de goPekko sin verificar.

## Cobertura Rust

Ninguna aserción Rust se atribuye por el nombre de un test. Esta transferencia no cambia `impl` ni `validation` de ninguna fila. Las únicas aserciones Rust revisadas son las de AC-PATH-016 (PR #10).
