# Skills compartidas de rakka

Las skills de este directorio son procedimientos de trabajo del repositorio, independientes del agente (Claude, OpenCode, Codex o combinaciones). La fuente pertenece al repo, no a la carpeta de una herramienta.

## Estructura

- `AGENTS.md` (raíz): punto de entrada. Contiene las reglas generales, el índice de skills y cuándo es obligatorio leer cada una.
- `skills/<nombre>/SKILL.md`: procedimiento de una tarea. Enlaza los documentos normativos; no los copia.
- `docs/`: reglas normativas (p. ej. `docs/testing-policy.md`).

| Skill | Cuándo | Política que aplica |
|---|---|---|
| [`testing`](testing/SKILL.md) | Crear, modificar, portar, clasificar o revisar pruebas | [`docs/testing-policy.md`](../docs/testing-policy.md) |

## Cómo leerlas manualmente (cualquier agente)

Abrir el archivo por su ruta desde la raíz del repositorio. No hace falta que la herramienta soporte skills ni interprete el frontmatter. La condición de lectura obligatoria está en la tabla de `AGENTS.md`.

## Acceso por herramienta

| Herramienta | Acceso manual (AGENTS.md + rutas) | Descubrimiento automático |
|---|---|---|
| Claude Code | Sí. `AGENTS.md` fue cargado como instrucciones del proyecto en la sesión donde se armó este cambio. | Solo para el adaptador `.claude/skills/testing/SKILL.md`: una skill creada en `.claude/skills/` apareció en la lista de skills de esa sesión. Ese adaptador solo remite a `skills/testing/SKILL.md`; el directorio `skills/` por sí mismo no se descubre automáticamente. |
| OpenCode | Por lectura directa de `AGENTS.md` y de las rutas. | No verificado. |
| Codex | Por lectura directa de `AGENTS.md` y de las rutas. | No verificado. |

"No verificado" significa que no se comprobó el comportamiento; no implica que no funcione. Hasta que se compruebe, la vía soportada para esas herramientas es la lectura manual indicada arriba.

## Adaptadores

Un adaptador es una referencia mínima a la skill compartida; no puede contener una segunda versión de las reglas. Hoy existe uno, para Claude Code (`.claude/skills/testing/SKILL.md`). No se crean adaptadores para otras herramientas sin necesidad comprobada, ni un sistema de sincronización.
