# Comparación de similitud: `uri.rs` (antes y después de la sala limpia)

Herramienta: `tools/legal/similarity.py OLD NEW --min 8` (tokens sin comentarios ni contenido de cadenas). Resultado:

```
uri-old.rs: 5927 tokens; uri.rs: 4314 tokens; shared runs >= 8: 329; longest: 105
```

329 bloques compartidos de ≥ 8 tokens; el más largo, 105. Clasificación de los bloques de 10 o más tokens que no son aserciones de tests (23 bloques):

- **105 y 35 tokens:** la definición de `UriError`, `Uri` (campos, derive) y las importaciones: es la **interfaz impuesta por la especificación** (sección 2 de `uri-spec.md`).
- **24 y 14 tokens:** las firmas `is_absolute(&self) -> bool` y `parse(s: &str) -> ...`: impuestas.
- **22, 19, 18, 17, 14, 13… tokens:** código de **tests** (`assert_eq!(u.raw_path.as_deref(), Some("…"))`, `#[test] fn …`, `mod tests { use super::*; }`) escrito contra la misma interfaz; las cadenas se abstraen en la medición. Ambos archivos tienen un test llamado `syntax_errors` (nombre natural).
- **20 tokens:** `digits.parse::<i32>().map_err(|_| …)`: idioma de Rust para convertir el puerto.
- **15 tokens:** `Uri { port: -1, ..Uri::default() }`: construcción del valor por defecto que fija la interfaz (`Default` + puerto −1).
- **14 tokens:** `(bytes: &[u8], out: &mut String) { let`: forma de una función auxiliar de decodificación.

No hay bloques largos idénticos de lógica de análisis (esquema, autoridad, host, IPv6, ruta, fragmento, decodificación) que no sean interfaz, firmas, tests o idiomas de Rust. La estructura de ramas del análisis es distinta (el archivo nuevo tiene 4 314 tokens frente a 5 927).
