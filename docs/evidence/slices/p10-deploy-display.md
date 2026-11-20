# P10 PR-C — `Deploy.toString` y `render_concise` (Q-CFG-35, Q-X-20)

Filas: AC-CFG-013. **Resultado: `partial`, validación `oracle`** (Q-CFG-35 sigue `proposed`; el texto de los routers reales espera a P37). Apilado sobre PR-B (#280): usa `scala_hash_order` para el orden de 5 o más tags.

## RED
`oracle/run.sh deploy-all` frente al gemelo con un `Display` provisional: **22 de 22** casos TS difieren. Después, con el `Display` real, la comparación de TS-23..TS-35 encontró cuatro divergencias de verdad: claves con dígitos Unicode (TS-24), pares de escapes `😀` leídos como dos U+FFFD (TS-25, bug del parser), y las capas de una clave sin resolver (TS-27, 29..35).

## GREEN
- `Display` de `Deploy` y `Scope`; tags en orden de inserción (hasta 4) o de `HashSet` (5 o más); `render_concise` con el orden de claves de Typesafe Config (dígitos primero por valor, resto UTF-16), sustituciones y concatenaciones sin resolver, y todas las capas de una clave (la más antigua primero).
- `Deploy::tags()` pasa de `&BTreeSet<String>` a `&[String]` (cambio de API interno al crate; sin consumidores fuera del arnés).
- Casos grabados `dp-0045..dp-0079` (TS-01..TS-35) con el oráculo. Replay **2921/2921, 0 diverge, 1 NA**.
- Cobertura 98,38 % del crate (`deploy.rs` 100 %, `config/mod.rs` 95,6 %, `config/parse.rs` 93,6 %: el archivo ya estaba bajo 95 % por sí solo; el criterio es por crate).

## Mutantes nuevos (7): todos DETECTED, control CLEAN (2921/2921)
render-keys-sort-as-plain-strings, render-numeric-keys-compare-as-text, render-shows-only-the-top-layer-of-a-key, render-concatenation-keeps-white-space-as-a-separate-string, tags-are-listed-sorted, five-tags-print-as-a-small-set, surrogate-escape-pair-becomes-two-replacement-characters. Reportes en `rust/divergence-reports/`.

## Límites (no equivalentes a Pekko aún)
- Router: se imprime el nombre de clase dado; Pekko imprime el `toString` del router real (P37). Solo hay casos con `NoRouter`.
- `Scope` solo tiene `Local` y `NoScopeGiven`. Un surrogate suelto en una clave queda fuera del dominio `&str` (Q-X-17).
- El orden de `HashSet` de los tags asume el mismo trie que `HashMap` (medido: TS-16..18 coinciden).
