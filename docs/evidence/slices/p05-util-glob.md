# Evidencia del slice P05 (UTIL-GLOB), 2026-11-18

Fila: AC-UTIL-016.6 (Glob). Decisión Q-UTIL-8 (c): unidades UTF-16; `Helpers.makePattern` no se reproduce bit a bit.
Casos nuevos en la familia `util-core`, añadidos al final: GL-01..GL-04, GL-U16-01..04, GL-EXH (longitudes 0-4), GL-PERF (2), GL-05 = uc-0084..uc-0099.

## Determinismo del oráculo (12 corridas por caso, salida idéntica)

```
1 distinct output(s) in 12 runs: GL-01 literals -> [abc]/[abc]=true [abc]/[abd]=false []/[
1 distinct output(s) in 12 runs: GL-02 question mark -> [a?c]/[abc]=true [a?c]/[ac]=false 
1 distinct output(s) in 12 runs: GL-03 star -> [*]/[]=true [*]/[anything]=true [a*]/[a]=tr
1 distinct output(s) in 12 runs: GL-04 star positions -> [*a]/[ba]=true [*a]/[ab]=false [a
1 distinct output(s) in 12 runs: GL-U16-01 ? vs astral -> [?]/[\uD83D\uDE00]=false [??]/[\
1 distinct output(s) in 12 runs: GL-U16-02 literal astral -> [\uD83D\uDE00]/[\uD83D\uDE00]
1 distinct output(s) in 12 runs: GL-U16-03 line breaks -> [a?b]/[a\u000Ab]=true [a*b]/[a\u
1 distinct output(s) in 12 runs: GL-U16-04 case and metachars -> [A]/[a]=false [a.c]/[abc]
1 distinct output(s) in 12 runs: GL-EXH patterns of length 0 -> []=TFFFFFFFFFFFFFFFFFFFFFF
1 distinct output(s) in 12 runs: GL-EXH patterns of length 1 -> [a]=FTFFFFFFFFFFFFFFFFFFFF
1 distinct output(s) in 12 runs: GL-EXH patterns of length 2 -> [aa]=FFFTFFFFFFFFFFFFFFFFF
1 distinct output(s) in 12 runs: GL-EXH patterns of length 3 -> [aaa]=FFFFFFFTFFFFFFFFFFFF
1 distinct output(s) in 12 runs: GL-EXH patterns of length 4 -> [aaaa]=FFFFFFFFFFFFFFFTFFF
1 distinct output(s) in 12 runs: GL-PERF 12 stars vs 36 a -> [*a*a*a*a*a*a*a*a*a*a*a*a*b]/
1 distinct output(s) in 12 runs: GL-PERF 20 stars vs 2000 a -> [42x]/[2000x]=false [42x]/[
  12 GL-05 wildcards in the input -> ok [*]/[*]=true [?]/[?]=true [*]/[?]=true [a*]/[a*]=true [a*b]/[a*b]=true
```

## ROJO

Replay con los 15 casos grabados y el binario sin Glob (después se grabó GL-05, uc-0099):

```
1046/1061 applicable + 1 not applicable
DIVERGES: uc-0084
...
DIVERGES: uc-0098
```

Test unitario (`crates/rakka-actor/tests/glob_spec.rs`) sin implementación:

```
error[E0432]: unresolved import `rakka_actor::util::glob_matches`
error: could not compile `rakka-actor` (test "glob_spec") due to 1 previous error
```

## VERDE

`cargo test -p rakka-actor --test glob_spec`: 7 passed. Primer replay con el arnés: `1061/1062 applicable` (uc-0098 divergía: el arnés Rust armaba mal el segundo patrón de GL-PERF 20; se corrigió el arnés, no `glob_matches`). Después:

```
1062/1062 applicable + 1 not applicable
```

Ningún caso es caracterización pura: antes del slice no existía `glob_matches`.

A4 sin regex (Q-UTIL-8 (c)): la prueba unitaria compara contra la definición recursiva ingenua; los casos GL-EXH graban el resultado de `Glob.matches` de Pekko para los 341 patrones x 31 entradas. Rendimiento (A5): GL-PERF graba el resultado; el límite de tiempo (< 3 s) se verifica en `chained_stars_do_not_backtrack_explosively`.

## Mutantes nuevos (solo ellos + control)

```
| mutant | outcome | passed/total |
|---|---|---|
| glob-trailing-star-unmatched | DETECTED | 1055/1062 |
| glob-question-rejects-surrogate-unit | DETECTED | 1061/1062 |
| glob-star-in-input-not-literal | DETECTED | 1061/1062 |
| control (unmodified) | CLEAN | 1062/1062 |
45 mutant patterns checked, 0 stale
```

## Gates

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check licenses`: limpios.
- `cargo test --workspace`: todo verde (incluye `glob_spec` 7 tests y `recorded_util_core_cases_match_pekko`).
- Cobertura de líneas del crate: 97.46 % regiones/ 97.37 % líneas; `util/glob.rs` 100 %.
- `check.py` sin fallos; `unittest discover -s tools/package/tests`: 35 tests OK.
