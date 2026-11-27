# rakka

Port a Rust de los actores de [Apache Pekko](https://pekko.apache.org/) 2.0.0: Classic (`rakka-actor`) y Typed (`rakka-actor-typed`), preservando sus contratos observables.

Estado: la **documentación de migración** (inventario, contratos, decisiones y 48 specs preliminares, aún pendientes de revisión de atomicidad y dependencias) está completa hasta donde permiten las decisiones abiertas del Owner. El código cubre una fracción del alcance (cifras vigentes en `docs/progress.md`). Documentación completa no es implementación ni paridad.

- **Inventario:** 785 filas (525 Classic, 260 Typed) revisadas contra el código, los cuerpos de los specs y `reference.conf` de Pekko 1.7.1 y reancladas a 2.0.0 (547 filas marcadas "revisar contra 2.0.0", ver `docs/inventory/pekko-2.0.0-reanchor.md`); 785 `extracted` y 0 `listed` (AC-UTIL-027 y 028, creadas por Q-UTIL-11, se extrajeron en P06) (cierre documental 2026-10-31; `extracted` significa contrato leído y aserciones citadas, no implementación).
- **Implementado:** AC-PATH; el núcleo de AC-UTIL; HOCON, ajustes, `Deploy`/`Deployer`, selección de proveedor y `ActorSystemSetup` (AC-CFG, parcial); y un runtime Classic del piloto: `ActorSystem` local con guardianes, celdas, hijos, `tell`/sender/dead letters, supervisión (`OneForOne`/`AllForOne`, Resume/Restart/Stop/Escalate), DeathWatch, `Identify`/`ActorIdentity` (sin `ActorSelection`) y despachador de un pool de hilos. Ese runtime no está integrado con `Config` (usa un `Settings` propio) y no tiene mailboxes especiales, `become`, `ActorSelection`, scheduler ni `Props` completo. Qué filas están validadas contra Pekko y cuáles no: `docs/evidence/slices/p01-reconciliacion-piloto-classic.md`.
- **Decisiones:** `docs/decisions.md` (276 decisiones: 0 bloqueantes, 190 propuestas, 66 aceptadas, 17 aceptadas con cambios, 3 rechazadas).

- Alcance y estado por funcionalidad: `docs/inventory/`
- Cierre documental 2026-10-31: plan final de 77 unidades (P05 particionada el 2026-11-01) `docs/unit-plan.md` (asignación fila→unidad en `docs/unit-rows.tsv`), ciclos y costuras `docs/cycles.md`, paquete de decisiones para el Owner `docs/owner-decision-package.md`.
- Orden de trabajo y specs preliminares (no ejecutables hasta su revisión de atomicidad y dependencias): `docs/implementation-order.md`, `docs/superpowers/plans/units/`
- Decisiones pendientes del Owner: `docs/decisions.md`
- Referencia upstream y licencia: `docs/upstream.md`, `LICENSE`, `NOTICE`
- Cómo trabajar en el repo: `AGENTS.md`

Licencia: Apache-2.0. Derivado de Apache Pekko (ver `NOTICE`).

## Compatibilidad

- **Render de `double` (Q-CFG-30).** El render de un `double` sigue la semántica de JDK 19+ (cadena más corta; entre las más cortas, la más cercana al valor y, en un empate exacto, la cifra par); difiere de JDK ≤ 18 en ~0,1 % de valores, solo en la última cifra. Ver Q-CFG-30 en `docs/decisions.md`. El oráculo corre en JDK 17: el replay clasifica esas diferencias como `NA:Q-CFG-30` por su firma, nunca por número de caso.
