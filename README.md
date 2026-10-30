# rakka

Port a Rust de los actores de [Apache Pekko](https://pekko.apache.org/) 1.7.1: Classic (`pekko-actor`) y Typed (`pekko-actor-typed`), preservando sus contratos observables.

Estado: inventario de alcance y dos unidades de la Fase 1 implementadas, ambas con filas `partial` (preguntas abiertas para el Owner, anotadas con `Q:` en el inventario):

- **AC-PATH**: `Address`, `ActorPath` y validación de nombres, validados contra Pekko. Las filas `AC-PATH-001`, `002`, `007`, `008`, `009`, `011`, `014` y `015` siguen `partial`.
- **AC-UTIL (parte del núcleo)**: `Timeout`, `Done`/`NotUsed`, `PekkoException`/`ConfigurationException` y `Helpers` (`base64`, `toRootLowerCase`, `Requiring`). Las filas `AC-UTIL-001` a `004` siguen `partial`; faltan `ConfigOps` (depende de AC-CFG) y `PrettyDuration`.

Todavía no hay configuración (HOCON), actores, mailboxes ni sistema: el siguiente paso de la cadena es AC-CFG. El estado fila por fila está en `docs/inventory/`; "100%" se define allí.

- Alcance y estado por funcionalidad: `docs/inventory/`
- Orden de trabajo: `docs/implementation-order.md`
- Referencia upstream y licencia: `docs/upstream.md`, `LICENSE`, `NOTICE`
- Cómo trabajar en el repo: `AGENTS.md`

Licencia: Apache-2.0. Derivado de Apache Pekko (ver `NOTICE`).
