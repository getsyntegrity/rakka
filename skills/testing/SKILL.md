---
name: testing
description: Usar al crear, modificar, portar, clasificar o revisar pruebas en rakka, o al decidir si una fila del inventario puede pasar a `done` respecto de la validación por capas (unitario vs. integración).
---

# Skill: pruebas (unitarias e integración)

Procedimiento para aplicar la política de testing. **La política normativa está en [`docs/testing-policy.md`](../../docs/testing-policy.md)** (única fuente de las reglas); esta skill no la repite. Si ambas difieren, manda la política y se reporta la discrepancia al Owner.

Leer primero `docs/testing-policy.md` completa. Las rutas se resuelven desde la raíz del repositorio.

## Procedimiento

1. **Extraer las aserciones del contrato** de la fila del inventario (`docs/inventory/contracts/`) y del spec de Pekko (`path:line`). Conservar sus aserciones observables.
2. **Clasificar cada aserción** como unitaria o de integración con las preguntas de la política §1. La capa la fija el alcance y los componentes que se ejecutan, no la carpeta ni el nombre del archivo.
3. **Ubicar** cada prueba según la política §4 (unitarios junto al componente; integración de un crate en su `tests/`; integración transversal prevista en `rakka-testint`, que no se crea sin tarea explícita).
4. **Desarrollar con TDD**: RED real con evidencia (comando y salida) antes del código de producción, GREEN, refactor. Aplica a ambas capas.
5. **Agregar y ejecutar la integración** en cuanto estén disponibles los componentes reales; no se acumula para el final del proyecto. Debe existir antes de cerrar la unidad.
6. **Declarar la frontera sustituida** en cada integración que reemplace un colaborador ajeno al escenario.
7. **Verificar** la trazabilidad contrato → escenario → prueba → evidencia y la cobertura (cobertura de producción ≥95 % por crate, sin exclusiones, combinando ambas capas, según `AGENTS.md`, Q-X-14).
8. **Actualizar estados** (`impl`/`validation`) solo si se cumplen las reglas de `AGENTS.md`. Si falta una integración necesaria, la fila queda `partial` con el componente real y la prueba pendiente identificados.

## Señales de alerta

- Un spec de Pekko que arranca actores reales "portado" como unitario con el runtime reemplazado por mocks.
- Un test con dobles presentado como evidencia de que los componentes reales funcionan juntos.
- Un unitario con sleeps, red, archivos reales o ActorSystem.
- Un integración con sleeps arbitrarios, puertos fijos o recursos sin liberar ante fallos.
- Cobertura alta o tests verdes usados como prueba de paridad con Pekko.

Los ejemplos concretos (backoff, reinicio de un hijo, orden de mensajes, parser, carga desde archivos temporales) están en la política §5.
