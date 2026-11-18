# Especificación: `crates/rakka-actor/src/actor/jstring.rs`

Rol: especificación (sucia). Esta página no contiene código, pseudocódigo traducido, nombres internos, estructura de ramas ni tablas copiadas del archivo actual ni de OpenJDK. Fuentes permitidas: documentación pública de la API de Java SE 17 (Javadoc de `String.compareTo`, `Integer.valueOf`/`parseInt`, `Character.digit`/`isDigit`), el estándar Unicode y los casos grabados contra Pekko 2.0.0 en JDK 17.

## 1. Interfaz pública (visibilidad `pub(crate)`, módulo `actor::jstring`)

```rust
pub(crate) fn compare_utf16(a: &str, b: &str) -> std::cmp::Ordering;
pub(crate) fn parse_int(s: &str) -> Result<i32, NumberFormatError>;
```

`NumberFormatError` ya existe en `actor/errors.rs` (`NumberFormatError::new(message)`, `message()`, `java_class()`); no se modifica. El archivo se declara en `actor/mod.rs` como `pub(crate) mod jstring;` y lo usan `address.rs` y `path.rs`.

## 2. Conducta

### 2.1 `compare_utf16`

Reproduce el orden de `java.lang.String.compareTo`: compara las dos cadenas **unidad de código UTF-16 por unidad de código UTF-16** como números sin signo; la primera diferencia decide; si una cadena es prefijo de la otra, la más corta es menor; dos cadenas iguales dan `Equal`.

Consecuencia que lo distingue del orden de `str` de Rust (que compara bytes UTF-8, es decir, puntos de código): un carácter fuera del plano básico (por ejemplo U+10000, que en UTF-16 empieza por el sustituto alto D800) es **menor** que un carácter del plano básico por encima de los sustitutos (por ejemplo U+FF61). En orden de puntos de código es al revés.

### 2.2 `parse_int`

Reproduce `Integer.valueOf(String)` en base 10:

- Un signo opcional, `-` o `+`, seguido de **al menos un dígito**. Una cadena vacía, un signo solo, o un signo seguido de otro signo no son números.
- Un dígito es cualquier unidad de código UTF-16 que `Character.digit(c, 10)` acepta: todos los caracteres del plano básico de categoría general Unicode **Nd** (dígito decimal), no solo `0`–`9` (por ejemplo los dígitos árabe-índicos U+0665 y los de ancho completo U+FF15). Cada unidad de código se examina por separado: un dígito decimal de un plano suplementario (par de sustitutos) **no** es dígito.
- Se permiten ceros a la izquierda; el valor es el de los dígitos como número decimal con el signo aplicado.
- El resultado debe caber en `i32` (de −2147483648 a 2147483647, ambos inclusive). Fuera de rango es error.
- No se admiten espacios ni otros caracteres: cualquier otro carácter es error.
- Error: `NumberFormatError` cuyo mensaje es `For input string: "<la cadena completa de entrada>"` (comillas dobles, sin escapar el contenido). Se observa igual para entrada vacía, signo solo, texto no numérico, basura al final, desbordamiento por arriba y por abajo, y cuando la entrada es una secuencia de escape `%31` sin decodificar.

### 2.3 Tabla de dígitos Unicode

La conducta de 2.2 depende de la lista de caracteres Nd del plano básico. Fuente pública para generarla: `UnicodeData.txt` de **Unicode 13.0.0** (la versión que documenta Java SE 17), https://www.unicode.org/Public/13.0.0/ucd/UnicodeData.txt: las líneas con categoría `Nd` y punto de código ≤ U+FFFF. Cada serie Nd son diez puntos de código consecutivos que valen 0 a 9. **No se copia ninguna tabla**: el implementador la genera de esa fuente (en el código basta con que figure de dónde sale y su versión). Puede comprobarla contra el oráculo (sección 4).

## 3. Casos que lo verifican (ids de `cases/`)

- Orden UTF-16: `CMPA-14` (`pa-0341`), `CMPP-13` (`pa-0356`), y en general `CMPA-*`/`CMPP-*` (`pa-0328`…`pa-0359`).
- `parse_int` (por el uid tras `#` de `ActorPath.fromString`): oráculo `FS-36`…`FS-56` (`pa-0101`…`pa-0121`), y de `cases/path-from-string.jsonl`: `uid-zero-is-undefined`, `uid-negative`, `uid-int-max`, `uid-int-min`, `uid-leading-zeros`, `uid-plus-sign`, `uid-arabic-indic-digits`, `uid-fullwidth-digits`, `uid-empty-nfe`, `uid-lone-sign-nfe`, `uid-not-a-number-nfe`, `uid-trailing-garbage-nfe`, `uid-overflow-nfe`, `uid-underflow-nfe`, `uid-huge-nfe`, `uid-percent-escaped-digi…`, `cov53`…`cov58`.
- Replay completo: `python3.12 tools/parity/replay.py` (hoy debe dar 1044/1044 aplicables + 1 no aplicable).

## 4. Cómo verificar más allá de los casos

El CLI del oráculo corre Pekko 2.0.0 en JDK 17: `oracle/run.sh path-from-string` con un literal JSON por stdin, por ejemplo `"pekko://sys/a#<dígito>"`. Sirve para contrastar cada serie Nd generada del punto 2.3.
