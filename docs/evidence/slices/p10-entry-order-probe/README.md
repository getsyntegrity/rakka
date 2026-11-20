# Sondas del orden de `SimpleConfigObject.entrySet()` (P10 PR-D, Q-CFG-36, Q-X-20)

Se ejecutan con el classpath del oráculo (`oracle/target/lib/*`, JDK 17). No forman parte del corpus; su resultado está resumido en `docs/legal/cleanroom/entry-order-spec.md`.

- `H.java`, `H2.java`: `hashCode()` de cada tipo de `ConfigValue` (escalares, listas, objetos).
- `E.java`: orden de `root().entrySet()` de un objeto leído con `parseString`; entrada: una configuración por línea.
- `F.java`: igual para `pekko.actor.deployment` tras `withFallback(defaultReference()).resolve()`; imprime clave y `hashCode` del valor.
