# Especificación: `crates/rakka-actor/src/actor/uri.rs`

Rol: especificación (sucia). Esta página no contiene código, pseudocódigo traducido, nombres internos, estructura de ramas ni tablas copiadas del archivo actual ni de OpenJDK. Fuentes permitidas: RFC 2396, RFC 2732, RFC 3986, RFC 3629, el estándar Unicode, la documentación pública de Java SE 17 (Javadoc de `java.net.URI`, `Character.isISOControl`, `Character.isSpaceChar`, `Integer.parseInt`) y los casos grabados contra Pekko 2.0.0 en JDK 17. **La conducta de detalle la fijan los casos grabados y el oráculo ejecutable**, no esta página: donde esta página es general, manda el oráculo (se puede consultar con `oracle/run.sh path-from-string`, y se puede comparar en masa con `tools/parity/fuzz_diff.py`).

## 1. Qué es y quién lo usa

Un analizador de cadenas que reproduce lo observable de `new java.net.URI(String)` para lo que Pekko consulta de ella: esquema, información de usuario (decodificada), host, puerto, ruta cruda y fragmento crudo. Lo usan `address.rs` (`AddressFromURIString`) y `path.rs` (`ActorPath.fromString` y `RelativeActorPath`).

## 2. Interfaz pública (visibilidad `pub(crate)`, módulo `actor::uri`)

```rust
pub(crate) enum UriError { Syntax, NumberFormat(NumberFormatError) }   // Debug, Clone, PartialEq, Eq

pub(crate) struct Uri {                       // Debug, Default
    pub(crate) scheme: Option<String>,        // tal como se escribió (conserva mayúsculas)
    raw_user_info: Option<String>,            // privado: la parte antes de '@' sin decodificar
    pub(crate) host: Option<String>,          // tal como se escribió; un IPv6 incluye los corchetes
    pub(crate) port: i32,                     // -1 si no hay puerto
    pub(crate) raw_path: Option<String>,      // sin decodificar; None en URI opacos
    pub(crate) raw_fragment: Option<String>,  // sin decodificar; Some("") si hay '#' vacío
}
impl Uri {
    pub(crate) fn user_info(&self) -> Option<String>;   // raw_user_info decodificada
    pub(crate) fn is_absolute(&self) -> bool;           // hay esquema
}
pub(crate) fn parse(s: &str) -> Result<Uri, UriError>;
```

`NumberFormatError` está en `actor/errors.rs` (`NumberFormatError::new(message)`). Los demás archivos del crate se mantienen: solo se implementa este.

## 3. Conducta general (fuente: Javadoc de `java.net.URI`, RFC 2396 con RFC 2732)

1. **Gramática.** `[esquema ":"] ( parte-jerárquica | parte-opaca ) ["#" fragmento]`. Con esquema, si lo que sigue a `:` empieza por `/` la URI es jerárquica; si no, es opaca (`raw_path = None`, y no se interpreta autoridad). Sin esquema la URI es relativa. La parte jerárquica es `["//" autoridad] ruta ["?" consulta]`; la consulta se reconoce y se descarta (no se expone), pero termina la ruta. El esquema es una letra seguida de letras, dígitos, `+`, `-` o `.`.
2. **Caracteres.** Se aplican las clases de caracteres de RFC 2396 por componente (sin reservados no permitidos, `%` solo seguido de dos dígitos hexadecimales en mayúscula o minúscula) más los caracteres «otros» de Java: cualquier carácter no ASCII que **no** sea de control ISO (`Character.isISOControl`) ni espacio Unicode (`Character.isSpaceChar`: categorías Zs, Zl, Zp). Los «otros» se admiten en ruta, consulta, fragmento, información de usuario y nombre de registro, no en el esquema, en nombres de host ni dentro de corchetes IPv6. Son ilegales (error de sintaxis) el espacio, los controles, `"`, `<`, `>`, `\`, `^`, `` ` ``, `{`, `|`, `}`. Los corchetes solo son legales alrededor de un IPv6. Un segundo `#` es ilegal.
3. **Autoridad.** Es lo que va tras `//` hasta el siguiente `/`, `?` o `#`. Java intenta primero leerla como *basada en servidor*: `[información-de-usuario "@"] host [":" puerto]`. Si no lo es pero todos sus caracteres son legales en un *nombre de registro*, se acepta como basada en registro: **host, información de usuario y puerto quedan indefinidos (`None`, `None`, `-1`)** y el resto de la URI se sigue analizando. Si no es ninguna de las dos, error de sintaxis. Una autoridad vacía seguida de una ruta, consulta o fragmento no vacíos se acepta con host indefinido; `//` sin nada más es error.
4. **Host.** Un nombre de dominio (etiquetas de letras, dígitos y guiones internos separadas por puntos; la última etiqueta empieza por letra; punto final admitido; sin guiones bajos), una dirección IPv4 de cuatro octetos decimales 0–255, o una referencia IPv6 entre corchetes (RFC 2732: grupos hexadecimales, compresión `::` una sola vez, hasta ocho grupos, IPv4 incrustada en los últimos 32 bits, y un identificador de ámbito tras `%`). Algo con forma de IPv4 con un octeto mayor que 255 no es una dirección IPv4.
5. **Puerto.** Solo dígitos ASCII; sin puerto (`host:` con nada detrás) el puerto es `-1`. Un puerto que no cabe en `i32` es error de sintaxis (ver casos).
6. **Ruta y fragmento.** Se devuelven tal como están escritos (con sus `%XX` sin decodificar). El fragmento es lo que sigue al primer `#`; vacío es `Some("")`.
7. **Información de usuario decodificada.** `user_info()` convierte cada secuencia de `%XX` en un octeto y decodifica los octetos consecutivos como UTF-8; las secuencias mal formadas se sustituyen por U+FFFD. **Cuántos U+FFFD aparecen por cada secuencia mal formada no es el criterio «máximo subparte» del estándar Unicode**: lo fijan los casos `UTF-AD-*` y `UTF-FS-*` (por ejemplo la codificación de un sustituto, `%ED%A0%80`, da un único U+FFFD). Hay que deducir la regla de esos casos y comprobarla con el oráculo y con el fuzzer. Los caracteres que no vienen escapados se copian tal cual.

## 4. Errores

- `UriError::Syntax`: la cadena no es una URI según lo anterior.
- `UriError::NumberFormat(e)`: un octeto IPv4 **incrustado en un IPv6** cuya tira de dígitos no cabe en un entero de 32 bits con signo no es un error de sintaxis: escapa como `NumberFormatException` de JDK 17. Mensaje exacto: `Error at index N in: "D"`, donde `D` es la tira de dígitos del octeto y `N` es la posición (desde 0, dentro de `D`) del primer dígito que, al acumularlo, lleva el valor por encima de 2147483647. Ejemplos grabados: `Error at index 9 in: "99999999999"` y `Error at index 9 in: "2147483648"`. Un octeto que cabe en el entero pero es mayor que 255 es simplemente «no es IPv4» (sintaxis).

## 5. Casos que lo verifican (ids de `cases/`)

- `cases/path-from-string.jsonl` completo (184 casos: `ok-*`, `bad-*`, `uid-*`, `nfe-*`, `cov*`), en particular `nfe-uri-ipv6-embedded-ip`, `nfe-uri-ipv6-embedded-ip-126`, `bad-port-*`, `bad-host-*`, `bad-ipv6-*`, `bad-illegal-*`, `ok-user-info-*` (el de sustituto suelto, `ok-user-info-lone-surrog`, está fuera de dominio: la API es `&str`).
- Oráculo de AC-PATH (`cases/oracle-path-address.jsonl`, ids en `cases/oracle-case-ids.tsv`): `FS-*` (fromString), `AD-*` (AddressFromURIString), `UTF-AD-*` y `UTF-FS-*` (escapes UTF-8 mal formados), `RP-*` (RelativeActorPath), `SPEC-APS-*`, `SPEC-RAP-*`, `SPEC-LARP-*`, `HC-*`, `DEEP-01` (una ruta de 1 000 000 de elementos: el análisis no debe desbordar la pila ni ser cuadrático).
- Escenarios del arnés que pasan por rutas (`cases/scenario-*.jsonl`): reutilizan `ActorPath`.
- Replay completo: `python3.12 tools/parity/replay.py` (debe dar 1044/1044 aplicables + 1 no aplicable).
- Diferencial en masa contra la JVM: `python3.12 tools/parity/fuzz_diff.py --count 20000 --seed 1` (y semilla 2): 0 divergencias.

## 6. Notas de implementación (no son código)

- El recorrido es sobre caracteres Unicode (`char`) de `&str`; Java trabaja en unidades UTF-16, pero ninguna regla distingue las dos mitades de un par sustituto, así que el resultado coincide.
- Los datos de clases Unicode (control ISO, espacios) se generan de la base de datos Unicode 13.0.0 (la versión de Java SE 17): https://www.unicode.org/Public/13.0.0/ucd/UnicodeData.txt. No se copia ninguna tabla.
