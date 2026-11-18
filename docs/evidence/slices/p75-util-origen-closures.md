# Evidencia del slice P75 (UTIL-ORIGEN-CLOSURES), 2026-11-18

Fila: AC-UTIL-016.8 (LineNumbers). Decision: Q-UTIL-16 aceptada por el Owner ("aceptar": opcion (a) `#[track_caller]` con retroceso a (b) `type_name`; Rust no reproduce las tablas de lineas del bytecode). Q-UTIL-11 (respondida) no afecta la fila. `LineNumbers.scala` de 2.0.0 leido y medido con el jar 2.0.0 (JDK 17). Casos LN-01..LN-11 en la familia util-core (al final), grabados como uc-0224..uc-0234.

## 2.0.0 frente al contrato (1.7.1)

`LineNumbers.scala` se reescribio con `MethodHandles` (`writeReplace` por `ClassValue`, `privateLookupIn`) y el parser acepta las constantes Dynamic/Module/Package (class file >= 53/55), pero el texto visible no cambio: `SourceFileLines.toString` (`f:n` o `f:from-to`), `SourceFile.toString` (el nombre), `NoSourceInfo`, `UnknownSourceFormat(<m>)` y `prettyName` (`<clase>`, `<clase>(<m>)`, `<clase>(<f>)`, `<paquete>/<f:lineas>`).

Medido con el jar (closures compilados en este proyecto):

```
lambda serializable de una linea    -> Raw.java:6        | /Raw.java:6      (paquete vacio)
lambda serializable de dos lineas   -> Raw.java:8-9      | /Raw.java:8-9
clase anonima                       -> Raw.java:12
lambda NO serializable              -> NoSourceInfo      | Raw$$Lambda$3/0x...  (nombre de clase)
LineNumbers(new Object())           -> NullPointerException (Class.getClassLoader() es null: el contrato "no lanza" no vale)
```

## Traduccion de la decision al oraculo

Los numeros de linea dependen del compilador de Java/Scala y Rust no puede reproducirlos. Los casos fijan (1) el texto exacto de cada `Result` y de `prettyName` (LN-01..LN-05, que el gemelo Rust reproduce bit a bit) y (2) la FORMA del resultado para closures del proyecto del oraculo (LN-06..LN-11: tipo `SourceFileLines`, archivo sin directorio con extension, lineas positivas, `from <= to`, `prettyName = <paquete>/<resultado>`; lambda no serializable = `NoSourceInfo` y nombre de clase; mismo sitio = mismo origen, sitios distintos = origenes distintos). Nunca imprimen los numeros del compilador. A1/A2 (22-22, 24-24, 29-31, 35-36; Java 29-29, 34-35, 40-45) quedan excluidas por la decision.

Determinismo: sin hilos, sin reloj, sin sleeps.

```
LN-01 .. LN-11: distinct outputs in 12 runs = 1 (oraculo, 12 corridas; gemelo Rust, 12 corridas)
```

## ROJO (stub: Display vacio, closure_origin = NoSourceInfo)

```
1188/1197 applicable + 1 not applicable
DIVERGES: uc-0224 uc-0225 uc-0226 uc-0227 uc-0229 uc-0230 uc-0231 uc-0232 uc-0234   (9; uc-0228 igualdad y uc-0233 son caracterizacion: pasan con el stub)
```

Un primer VERDE dejo uc-0232 divergente: `type_name` de Rust contiene `::`, el nombre de clase de Java no. La abstraccion del caso se cambio de "sin ':'" a "sin sufijo `:<n>[-<n>]`" (diferencia real y aceptada por Q-UTIL-16 (b)); caso regrabado.

## VERDE

```
1197/1197 applicable + 1 not applicable
cargo test -p rakka-actor --lib line_numbers -> 3 passed
```

## Mutantes nuevos (solo ellos + control)

```
| line-numbers-range-always | DETECTED | 1195/1197 |
| line-numbers-keeps-directory | DETECTED | 1194/1197 |
| line-numbers-pretty-without-package | DETECTED | 1194/1197 |
| control (unmodified) | CLEAN | 1197/1197 |
--check-patterns: 69 mutant patterns checked, 0 stale
```

## Limites

El constructor tipado (`Behaviors::receive`, interceptores) no existe aun en `rakka-actor-typed`: cuando llegue debe llamar `closure_origin()` desde un constructor `#[track_caller]` y sus tests de `Debug` fijar este formato (consecuencia ya anunciada en Q-UTIL-16). `Location` solo da el sitio de la llamada, no el rango de lineas del cuerpo del closure.
