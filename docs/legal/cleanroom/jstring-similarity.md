# Comparación de similitud: `jstring.rs` (antes y después de la sala limpia)

Herramienta: `tools/legal/similarity.py OLD NEW --min 8` (tokens sin comentarios ni contenido de cadenas). Resultado de la corrida:

```
/private/tmp/claude-501/-Users-pablogore-Projects-pekko-rust-pilot/18dc031d-7a9c-4868-bcc9-2aa4e561a03d/scratchpad/jstring-old.rs: 743 tokens; crates/rakka-actor/src/actor/jstring.rs: 1275 tokens; shared runs >= 8: 59; longest: 85
```

## Lectura de los bloques compartidos

- **Mayor bloque (85 tokens):** la tabla de las 37 series de dígitos decimales (categoría Nd, BMP). Son **datos factuales de Unicode 13.0.0**, generados de `UnicodeData.txt` por ambas versiones; no hay otra forma de escribirlos.
- **70 tokens:** `use std::cmp::Ordering; use super::errors::NumberFormatError;`: importaciones impuestas por la especificación.
- **Bloques de cadenas/tests (61, 48… tokens, todos «S, S, S»):** listas de entradas de prueba que salen de los casos grabados y de la especificación (las cadenas se abstraen en la medición).
- **Idiomas de Rust (≤ 47 tokens):** el cierre `let fail = || NumberFormatError::new(format!("For input string: ..."))` (mensaje fijado por los casos), y el recorrido por la tabla `iter().find(|&&start| (start..start + 10).contains(&unit)).map(...)` (15 tokens). Esta última coincidencia se reporta al Owner: es una forma idiomática obvia de buscar una unidad en series de diez, y el agente limpio declara no haber leído el archivo anterior.
- **Firmas:** `compare_utf16` y `parse_int` son las impuestas por la especificación.

No hay bloques largos idénticos que no sean datos, importaciones, firmas o entradas de prueba impuestas por la especificación y los casos grabados.
