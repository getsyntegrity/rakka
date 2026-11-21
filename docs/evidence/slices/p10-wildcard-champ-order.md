# P10 PR-B — orden de los `prefix*` solapados (Q-UTIL-21, Q-X-21)

Filas: AC-UTIL-009 (A7), AC-CFG-012.2 (B1/B2 de `p10-strict-parity-matrix.md`). **Resultado: `partial`, validación `oracle`** (sin cambio de estado: Q-UTIL-21 y Q-X-21 siguen `proposed`).

## RED (observado)

`oracle/run.sh deployer-all` frente a `classic-parity deployer-all` con 72 guiones nuevos (`DL-60..DL-131`, orden barajado y orden inverso de 2, 3 y 5 prefijos de 12 elementos): **45 de 72 difieren** (p. ej. `DL-60`: Pekko `/a/pqzz*@d3`, Rust `/a/pq*@d2`, el primero insertado). Test unitario `keys_iterate_in_the_order_measured_in_the_jvm_whatever_the_insertion_order` fallando con `left: ["pq", "p"] right: ["p", "pq"]`.

## GREEN

- `util/scala_hash_order.rs` (nuevo): `string_hash_code`, `improve`, recorrido por nodos. El comportamiento sale de las mediciones del `HashMap` real y no de ninguna fuente; el primer candidato de `improve` acertó contra el oráculo.
- `WildcardTree` mantiene `wildcard_suffix_children` en ese orden tras cada alta (`sort_suffix_children_in_hash_map_order`); reinsertar un prefijo conserva su posición.
- Replay: **2886/2886 aplicables, 0 diverge, 1 no aplicable** (antes 2810; +76 casos `dr-0061..dr-0136`; `dr-0059/0060` pasan a ser DL-60/61 y los casos de concurrencia DC-01/02 son ahora `dr-0135/0136`).
- Colisión completa: `tools/parity/find_prefix_hash_collision.py` encontró dos prefijos de un mismo elemento con igual `hashCode` (263630388). Pekko, con ambos órdenes de inserción, elige el primero insertado (DL-132/133); Rust igual.
- Fuzz diferencial nuevo (`wildcard-winners` en el oráculo y el gemelo; `fuzz_wildcard_order.py`): seeds 1, 2 y 3, 20 000 conjuntos cada una, **0 diferencias**.

## Mutantes nuevos (5): todos DETECTED, control CLEAN

`tools/parity/check_divergence.py <nombre>` (un único proceso; el nombre repetido en la línea de comandos se ejecutó una sola vez, sin duplicados):

| Mutante | Resultado | Casos que divergen |
|---|---|---|
| wildcard-suffix-children-keep-insertion-order | DETECTED | 45 |
| hash-map-order-skips-the-improve-scrambling | DETECTED | 54 |
| hash-map-order-reads-the-bits-from-the-top | DETECTED | 48 |
| hash-map-order-puts-child-nodes-before-lone-entries | DETECTED | 10 |
| hash-map-order-reverses-colliding-keys | DETECTED | 2 (DL-132/133) |
| control sin modificar | CLEAN | 0 de 2886 |

Ninguno es equivalente. Reportes en `rust/divergence-reports/`. Cobertura (`tools/parity/coverage.sh`): 98,27 % del crate; `scala_hash_order.rs` 100 %, `wildcard_index.rs` 99,6 %.

## Límites

- Solo claves `&str` (Q-X-17). Las claves de `wildcard_suffix_children` se comparan por `String.hashCode` sobre UTF-16 como en Pekko.
- Depende de un detalle interno de Scala 2.13 (`immutable.HashMap`); el fuzz diferencial lo vigila.
- `Deployer::from_settings` sigue con el orden alfabético de claves en colisión (A1/A2, PR-D).
