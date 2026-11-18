# Evidencia del slice P65 (UTIL-HASHCODE), 2026-11-18

Fila: AC-UTIL-016.15. Decisiones: Q-UTIL-14 (a) FrequencySketch (otra unidad, AC-UTIL-017.1) y (b) para el resto; Q-PATH-3 aceptada (Address/ActorPath no reproducen el hash). HashCode.scala no tiene test upstream: la referencia es el oráculo 2.0.0 (casos HC-01..HC-14 en la familia util-core, añadidos al final).

## Determinismo del oráculo (12 corridas por caso, 1 salida distinta)

```
1 distinct output(s) in 12 runs: HC-01 boolean ... HC-14 SEED constant  (los 14 casos)
```

## ROJO (binario sin `util::hash_code` ni los casos en el arnés, casos ya grabados)

```
1084/1096 applicable + 1 not applicable
DIVERGES: uc-0122 ... DIVERGES: uc-0133   (12 casos)
```

## VERDE

```
1096/1096 applicable + 1 not applicable
```

HC-12 y HC-13 (null) están grabados en el oráculo pero no los produce Rust (PENDING del arnés: no hay null).

## Mutantes nuevos (solo ellos + control)

```
| hash-code-long-no-fold | DETECTED | 1090/1096 |
| hash-code-nan-payload-kept | DETECTED | 1095/1096 |
| hash-code-string-utf8-units | DETECTED | 1094/1096 |
| control (unmodified) | CLEAN | 1096/1096 |
51 mutant patterns checked, 0 stale
```

Cobertura de líneas: util/hash_code.rs 100 %, TOTAL 97.66 %.
