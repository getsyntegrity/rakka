# P10 PR-C — `Deploy.toString` y `render_concise` (Q-CFG-35, Q-X-21)

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

## Divergencias abiertas halladas por la revisión independiente (HEAD 4d62b3f)

El corpus grabado (79 casos) coincide, pero el revisor encontró, con unas 160 entradas propias contra el `Deploy.toString` real, diferencias que **bloquean el merge**. Siguen sin corregir y no están aceptadas:

- **D1** Claves de igual valor numérico (`"7"` y `"007"`): Pekko desempata por el orden de iteración de su `HashMap`, Rust alfabéticamente. Depende del mismo emulador de `HashMap` de Java que PR-D.
- **D2** Concatenación de escalares no string con espacios (`a = 1.50 ${x}`, `1e2`, `0x10`, `true  `, `null`): Pekko convierte el escalar a su texto original y lo une al espacio (`"1.50 "${x}`); Rust imprime el valor tipado y un `" "` aparte.
- **D3** Listas u objetos adyacentes en una concatenación (`[1] [2] ${x}`): Pekko los fusiona y descarta el espacio (`[1,2]${x}`); Rust conserva las piezas.
- **D4** Cita de los elementos de una ruta de sustitución (`${1}`, `${a.007}`, `${a.-1}`, `${true}`): Pekko no cita lo que empieza por dígito o `-`; Rust sí (`Subst::display`).
- **D5** (parser) `${"a\"b"}` da error en Rust; `a=[1] x` da error en Rust y `{"a":[1]}` en Pekko; `a=1 [2] ${x}` da WrongType en Pekko y `ok` en Rust.
- **D6** (Q-X-17) Un surrogate suelto en un valor (no solo en una clave) se lee como U+FFFD.
- Sin mutante: el manejo de dígitos Unicode en `compare_render_keys` y el desempate D1.
- `with_tags` con un `HashSet` da orden no determinista para 2 a 4 tags; en Pekko el orden depende de cómo se construyó el `Set`.
- Cobertura: el criterio es por crate (98,38 %); `config/parse.rs` aislado está en 93,6 %.

Por tanto este PR **no** afirma que `render_concise` sea equivalente a Typesafe Config 1.4.9; solo que lo es en los 79 casos del corpus y en las entradas que el revisor comprobó igual. Las correcciones de D2 a D4 y el nuevo corpus van en commits siguientes de este mismo PR.
