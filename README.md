# rakka

Port a Rust de los actores de [Apache Pekko](https://pekko.apache.org/) 1.7.1: Classic (`pekko-actor`) y Typed (`pekko-actor-typed`), preservando sus contratos observables.

Estado: la **documentación de migración** (inventario, contratos, decisiones y specs ejecutables) está completa hasta donde permiten las decisiones abiertas del Owner; el código implementado es poco. Documentación completa no es implementación ni paridad.

- **Inventario:** 716 filas (507 Classic, 209 Typed) revisadas contra el código, los cuerpos de los specs y `reference.conf` de Pekko 1.7.1; 570 `extracted` y 146 `listed` (las `listed` no se pueden ejecutar hasta extraer su contrato).
- **Implementado:** AC-PATH y la parte del núcleo de AC-UTIL (`Timeout`, `Done`/`NotUsed`, excepciones base, `Helpers`), con filas `partial` por decisiones abiertas. Nada más: no hay HOCON, actores, mailboxes ni sistema.
- **Decisiones:** `docs/decisions.md` (186 decisiones: 8 bloqueantes, 178 propuestas, 0 aceptadas).

- Alcance y estado por funcionalidad: `docs/inventory/`
- Orden de trabajo y specs ejecutables: `docs/implementation-order.md`, `docs/superpowers/plans/units/`
- Decisiones pendientes del Owner: `docs/decisions.md`
- Referencia upstream y licencia: `docs/upstream.md`, `LICENSE`, `NOTICE`
- Cómo trabajar en el repo: `AGENTS.md`

Licencia: Apache-2.0. Derivado de Apache Pekko (ver `NOTICE`).
