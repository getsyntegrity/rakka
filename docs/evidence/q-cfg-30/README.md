# Q-CFG-30: evidencia de la medición de `Double.toString`

Muestra de Fork D: `Gen.java` (semilla 42, 400 000 dobles, JDK 17) produce `java.txt` (`bits cadena`); 380 849 son no enteros (los enteros que caben en un long no pasan por `Double.toString`). `Conv.java` vuelve a convertir los mismos bits con el JDK en uso (se corrió en JDK 17 y en JDK 27).

```
javac Gen.java && java Gen > java17.txt          # JDK 17; idéntico byte a byte a la muestra original
javac Conv.java && java Conv < java17.txt        # JDK 27 (semántica de JDK 19+)
```

`divergent-362.tsv`: los 362 dobles donde el render anterior de rakka difería de JDK 17. Clasificación medida con aritmética exacta:

| clase | casos | qué es |
|---|---|---|
| `exact-tie` | 357 | el valor está a mitad de camino entre dos candidatos de la misma longitud; JDK 17 y JDK 19+ eligen la cifra par; el redondeo de la std de Rust (hacia arriba) era un fallo de rakka, corregido |
| `jdk17-bug` | 5 | JDK 17 no da la cadena más corta o la más cercana; JDK 19+ coincide con rakka |

Tras la corrección: rakka coincide con JDK 27 en los 380 849 dobles (0 diferencias) y difiere de JDK 17 en 5. Otros casos de JDK 17 con la misma firma: `2e23` (JDK 17: `1.9999999999999998E23`) y 2^-24 (JDK 17: `5.9604644775390625E-8`); los 7 son `NA:Q-CFG-30` en el replay (`tools/parity/na_q_cfg_30.py`). Decisión: `docs/decisions.md`, Q-CFG-30.
