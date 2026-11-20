# Política de capas de test (unitario vs. integración)

Fuente única de verdad de la política de testing de rakka, válida para cualquier agente o persona (Claude, OpenCode, Codex o combinaciones). No depende de skills ni de configuraciones de una herramienta concreta. `AGENTS.md` contiene sólo las reglas esenciales y exige leer este documento, junto con `skills/testing/SKILL.md` (procedimiento), antes de crear, modificar, portar, clasificar o revisar pruebas. Si ambos difieren, no se resuelve por interpretación propia: se reporta al Owner.

Esta política no aprueba decisiones funcionales pendientes ni autoriza implementar módulos. Una diferencia semántica con Pekko sigue requiriendo decisión explícita del Owner (`Q:` en `notes`).

## 1. Elegir la capa

La capa la determina **el alcance y qué componentes ejecuta el test**, no la carpeta ni el nombre del archivo. Un archivo en `tests/` puede ser unitario (p. ej. specs portados de funciones puras como `actor_path_spec.rs`), y un módulo `#[cfg(test)]` podría ser integración si arranca el runtime.

Preguntas, en orden; la primera respuesta afirmativa fija la capa:

1. ¿Usa archivos reales, red, variables de entorno de la máquina o un servicio externo? → **Integración.**
2. ¿Necesita un ActorSystem o runtime de actores, o conecta dos o más componentes reales (actores, mailbox + dispatcher + celda, supervisión + hijos, timers + destinatario) para validar un proceso completo? → **Integración.** Esto incluye toda interacción entre actores reales, aunque sea en memoria y en el mismo proceso.
3. ¿El spec de Pekko que se porta arranca actores reales? → **Integración.** No se convierte en unitario sustituyendo el runtime por mocks.
4. ¿Valida un comportamiento concreto de una función, componente o transición de estado, aunque el componente sea el real (p. ej. un mailbox aislado, la decisión de una estrategia de supervisión, una transición de estado), con a lo sumo dobles pequeños para aislar un colaborador? → **Unitario.**

Usar un componente real no vuelve integración a un test: probar una implementación real de forma aislada sigue siendo unitario. La integración aparece al conectar componentes y validar el proceso del escenario.

Si una aserción del contrato exige ambos niveles (la lógica interna y el comportamiento observable en el sistema), se escriben las dos pruebas; una no sustituye a la otra.

## 2. Tests unitarios

Livianos, rápidos, aislados y atómicos: un comportamiento por caso (puede tener varias aserciones).

- Sin ActorSystem, runtime de actores, servicios externos ni contenedores.
- Sin red, archivos reales, variables de entorno de la máquina ni servicios instalados.
- Sin `sleep` para esperar resultados.
- Tiempo y azar bajo control (reloj y fuente aleatoria inyectables) cuando afecten al comportamiento.
- Mocks, stubs o fakes sólo para aislar un colaborador. Una función pura se prueba directamente, y un componente real probado de forma aislada puede tener tests unitarios: no es obligatorio mockear el componente bajo prueba. No hace falta una librería de mocks ni nuevas dependencias de testing si un doble pequeño alcanza.
- El resultado esperado sale del contrato (Pekko, golden, valores escritos a mano), nunca de reimplementar el algoritmo de producción en el test.
- Ubicación: junto al componente, con las convenciones de Rust (`#[cfg(test)] mod tests` en el mismo archivo) y las del repositorio.
- TDD estricto: RED real (falla por la razón correcta, con comando y salida guardados) antes del código de producción, luego GREEN y refactor.

## 3. Tests de integración

Validan el proceso completo de un escenario contractual con los componentes reales involucrados.

- Usan el runtime, mailbox, dispatcher, supervisión, timers y demás componentes reales que el escenario requiera.
- No se mockean los componentes cuya integración se valida. Pueden sustituirse colaboradores ajenos al escenario, pero **la frontera sustituida se declara explícitamente** (comentario en el test y nota en la evidencia).
- Verifican resultados observables: mensajes, orden contractual, fallos, reinicios, terminación, cancelación de timers y efectos.
- Sincronización explícita (probes, latches, canales, futuros) y esperas acotadas con timeout. Sin sleeps arbitrarios, sin puertos fijos, sin depender del orden de ejecución entre tests.
- Liberan actores, sistemas, archivos temporales y demás recursos incluso ante fallos (guardas `Drop`/RAII, no limpieza al final del cuerpo del test).
- Recursos externos: entornos aislados; Testcontainers cuando haga falta un servicio externo. No es requisito para integración local entre actores.
- Ubicación: en el directorio `tests/` del crate. Si cruzan Classic y Typed o varios crates, ver §4.

## 4. Organización

| Alcance | Dónde |
|---|---|
| Unitario | Junto al componente (`#[cfg(test)]` en el módulo) según las convenciones de Rust y del repositorio |
| Integración de un crate | `crates/<crate>/tests/` |
| Integración transversal Classic/Typed o de varios crates | Crate interno **`rakka-testint`** (ubicación **prevista**, aún no existe) |

`rakka-testint` es un crate interno del workspace, dedicado a pruebas, **no publicable** (`publish = false`). No es el testkit público que usarán los usuarios de Rakka para probar sus propios actores (`AC-TK-*`); son cosas distintas y no deben confundirse ni compartir API.

Mientras `rakka-testint` no exista, no se crea ni se modifica `Cargo.toml` por iniciativa de un agente: crear el crate es una tarea explícita aparte. Hasta entonces, una integración transversal necesaria se deja identificada como dependencia pendiente (ver §6).

## 5. Ejemplos

| Caso | Capa | Por qué |
|---|---|---|
| Cálculo de backoff (`min`, `max`, `randomFactor`, nº de reinicio → retardo) | Unitario | Función pura; el azar se inyecta o se fija con factor 0. Se verifica el valor esperado escrito a mano desde el contrato, sin recalcular con la fórmula de producción. |
| Un hijo falla y se reinicia por completo (preRestart, nueva instancia, postRestart, mensajes posteriores) | Integración | Requiere supervisión, celda, mailbox y dispatcher reales; se observa el ciclo de vida completo. |
| Orden de mensajes entre dos actores reales | Integración | Interacción entre actores reales sobre mailbox y dispatcher reales; con un doble no se demuestra. El orden FIFO de un mailbox probado de forma aislada, sin actores, es unitario. |
| Parser (HOCON, path, address) sobre una cadena en memoria | Unitario | Sin I/O; entrada y salida son valores. |
| Carga de configuración real desde archivos temporales | Integración | Usa el sistema de archivos real; el directorio temporal es único por test y se elimina aunque el test falle. |
| Transición de estado de un componente (p. ej. estado de un `Terminated`/watch aislado) | Unitario | Un comportamiento, sin runtime. |
| Cancelación de un timer programado y no entrega posterior del mensaje al actor destinatario | Integración | Conecta scheduler real y actor receptor; con tiempo controlable si el scheduler lo admite, sin sleeps arbitrarios. Cancelar una tarea en el scheduler aislado, con reloj controlado y sin destinatario real, es unitario. |

No se exige una integración artificial a funciones puras que no tienen una frontera de integración relevante.

## 6. Secuencia por unidad y cierre de filas

Por cada unidad:

1. Al extraer el contrato, marcar cada aserción como **unitaria** o **de integración** (con la capa justificada según §1).
2. Desarrollar con TDD unitario.
3. Agregar y ejecutar la integración correspondiente **en cuanto los componentes reales estén disponibles**, y en todo caso antes de cerrar la unidad. No acumular toda la integración para el final del proyecto.
4. Verificar la trazabilidad **contrato → escenario → prueba → evidencia** (la fila del inventario, la aserción, el test que la cubre y el comando/salida que lo prueba).
5. Actualizar `impl`/`validation` sólo cuando se cumplan las reglas vigentes de `AGENTS.md`.

Costuras temporales: un doble que cumpla las aserciones del contrato (ver `docs/implementation-order.md`) permite avanzar con unitarios antes de que exista el componente real, pero **un doble no acredita la integración con el componente sustituido**. Mientras falte esa integración necesaria, la fila permanece `partial` y se identifica el componente real y la prueba pendiente.

Reglas que se mantienen sin cambios:

- Cobertura de producción ≥95 % por crate, sin exclusiones (`AGENTS.md`, Q-X-14), con `cargo llvm-cov` (la cobertura combinada de ambas capas cuenta; no hay un porcentaje obligatorio separado por capa; se documentan comando, resultado y líneas sin cubrir). La cobertura de integración no sustituye los unitarios necesarios, ni demuestra paridad, ni permite omitir escenarios contractuales.
- Una pregunta abierta (`Q:`) impide cerrar el comportamiento afectado: `partial`, no `done`, aunque ambas capas estén verdes.

## 7. Paridad con Pekko

- Al clasificar y portar un spec, se conservan sus aserciones observables.
- Los tests diferenciales contra los jars oficiales (`tools/oracle/`, `*.golden`) son validación de paridad y complementan tanto unitarios como integración; **no** eximen de escribir las aserciones necesarias en cada capa.
- Tests verdes y cobertura alta no demuestran paridad por sí solos.
- Una diferencia semántica con Pekko requiere decisión explícita del Owner.

## 8. Lista de verificación rápida

- [ ] Cada aserción del contrato tiene capa asignada y justificada por alcance, no por carpeta.
- [ ] Los unitarios no arrancan sistema, no tocan I/O/entorno, no duermen y no recalculan el resultado con el algoritmo de producción.
- [ ] Las integraciones usan componentes reales, declaran la frontera sustituida, esperan con límite y liberan recursos ante fallos.
- [ ] Los specs de Pekko con actores reales están clasificados como integración.
- [ ] Hay RED real guardado antes del código y GREEN después.
- [ ] La trazabilidad contrato → escenario → prueba → evidencia está completa, o la fila queda `partial` con la dependencia pendiente anotada.
- [ ] Cobertura de producción ≥ 95 % por crate documentada (comando, resultado y líneas sin cubrir).
