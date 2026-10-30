# Unidad AC-PATH: `Address` y `ActorPath` — plan de implementación

**Goal:** Que `pekko-actor` tenga los tipos `Address` y `ActorPath` con el mismo comportamiento observable que Pekko 1.7.1, comprobado contra Pekko compilado desde `upstream/pekko`.

**Architecture:** Dos tipos de valor inmutables en `crates/pekko-actor/src/actor/`. `ActorPath` es una lista enlazada hacia la raíz compartida con `Arc`, igual que en Pekko (cada hijo apunta a su padre). El parseo de cadenas reproduce la parte de `java.net.URI` que se ve a través de Pekko, en un módulo interno `uri.rs`, porque varios resultados observables (qué se acepta y qué no) salen de esa clase del JDK (Java Development Kit) y no de código de Pekko.

**Tech Stack:** Rust 1.96, edition 2024, sin dependencias externas. Oráculo: `jshell` (Java 21) con `tools/oracle/run.sh tools/oracle/path-address.jsh`.

**Fuente del contrato** (rutas relativas a `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`): `Address.scala:37-232` (`Address`, `addressOrdering`, `PathUtils.split`, `RelativeActorPath`, `AddressFromURIString`, `ActorPathExtractor`), `ActorPath.scala:29-151` (`ActorPaths`, `fromString`, validación) y `:279-495` (`RootActorPath`, `ChildActorPath`), `ActorCell.scala:390-404` (`undefinedUid`, `newUid`, `splitNameAndUid`), `ActorRef.scala:541-562` (`IgnoreActorRef`). Specs: ver la sección Trazabilidad. Filas: `AC-PATH-*` de `docs/inventory/actor-classic.tsv`.

## Qué resuelve y qué deja afuera

El problema: todas las demás unidades identifican actores por path, y el parseo de paths en Pekko tiene muchos detalles que salen del JDK (qué host acepta, qué puerto, cómo se lee el uid). Si el port los aproxima, las diferencias aparecen después mezcladas con concurrencia, donde cuesta mucho más encontrarlas. Esta unidad no depende de ninguna otra y se puede comparar entera contra Pekko sin levantar un `ActorSystem`.

Cierra `AC-PATH-001` a `008`, `010`, `012`, `013` y `014`, salvo las que tengan una `Q:` abierta al terminar (quedan `partial`). Tres filas quedan `partial` sí o sí, porque dependen de unidades futuras:

- `AC-PATH-009` (fachada Java `ActorPaths`): depende de la pregunta Q-PATH-5.
- `AC-PATH-011` (uid): este plan cubre el uid en `/`, `fromString`, `undefinedUid = 0` y `with_uid`. `newUid` (aleatorio ≠ 0) lo implementa AC-CELL, que es quien lo llama.
- `AC-PATH-015` (paths especiales): sólo `pekko://local/ignore`. `/user`, `/system`, `/deadLetters`, `/temp` los crea el proveedor local en AC-REF/AC-SYS.

Quedan fuera dos specs que tocan estos tipos pero prueban otra cosa: `ActorSelectionSpec.scala:64,169,352-357` (selección, unidad AC-SEL) y `actor-typed-tests/.../ActorRefResolverSpec.scala:35-51` (resolver typed). `LocalActorRefProviderSpec.scala:169-170` (`actorOf` con `null` y `""`) tampoco: esos mensajes salen de `dungeon/Children.scala:269-276` (AC-CELL), no de `validatePathElement`.

## Referencia: Pekko compilado desde la fuente

El golden se genera con `tools/oracle/run.sh tools/oracle/path-address.jsh`, que corre contra Pekko 1.7.1 compilado desde `upstream/pekko` (el cambio de `run.sh` lo hace otra tarea). Los jars publicados (`ORACLE=jars`) sólo sirven para comprobar que el script compila; **ningún golden se escribe desde los jars**. El golden actual `crates/pekko-actor/tests/oracle/path-address.golden` salió de los jars y de una versión vieja del script: no es referencia y se regenera en la Task 5.

El JDK también es parte de la referencia: `java.net.URI` y `Integer.valueOf` vienen de Java 21. Si se cambia de JDK, se regenera el golden y se revisan las diferencias.

## Decisiones (con la evidencia y lo descartado)

**1. Puerto `Option<i32>`, independiente del host.** El constructor de Pekko acepta cualquier `Int`: `Address.scala:44,132` reciben `port: Int` sin validar, y el oráculo lo confirma con `-5`, `-1`, `0`, `70000`, `Int.MinValue` y `Int.MaxValue` (casos `CT-03..08`). Además el host y el puerto son opcionales por separado: `copy` es público (`Address.scala:46-52`) y el constructor Java con `host = null` da `host = None, port = Some(1)` (`Address.scala:44`, caso `CT-09`, imprime `pekko://sys:1`). El parser es más estrecho: sólo dígitos decimales, ceros a la izquierda permitidos, hasta `2147483647`; con más dígitos `java.net.URI` no puede leer el puerto, cae a una autoridad "registry" sin host y Pekko lo rechaza (`AD-12..19`, `FS-34/35`). Por eso un `Address` con puerto negativo se puede construir pero no se puede volver a parsear desde su `to_string()` (`CT-20`). Se descartó `u32` (pierde los negativos del constructor) y `u16` (pierde `70000`, que el parser sí acepta).

**2. Sin `Ord` ni `PartialOrd`; comparador explícito `pekko_compare`.** Decisión del Owner (2026-10-30): si el comparador de Pekko no cumple las leyes de `Ord` de Rust, no se implementa un `Ord` incoherente ni se agregan desempates silenciosos. Ninguno de los dos los cumple:

- `Address.addressOrdering` (`Address.scala:138-145`) usa `host.getOrElse("")` y `port.getOrElse(0)`, pero sólo después de ver que los campos difieren. Resultado: `host = None` contra `Some("")` da 0 sin ser iguales (`CMPA-08`), y la "igualdad" del orden no es transitiva: `a = (None, puerto 1)` ~ `b = (Some(""), 2)` (`CMPA-10`), `b` ~ `c = (None, 2)` (`CMPA-11`), pero `a < c` (`CMPA-12`). No es ni siquiera un orden débil estricto.
- `ActorPath.compareTo` compara nombres desde la hoja y, al llegar a una raíz, compara `RootActorPath.toString` (`ActorPath.scala:307-311,481-494`). Es un preorden total, pero no coincide con `equals`: dos raíces con el mismo texto y distinta `Address` dan 0 y no son iguales. Contraejemplos con API pública: `Address("p","s@h:1")` contra `Address("p","s","h",1)` (`CMPP-09`, también con hijos `CMPP-10`), y `RootActorPath(Address("p","s"), "/a")` contra `RootActorPath(Address("p","s/"), "a")` (`CMPP-11`).

Por eso: `Address` y `ActorPath` implementan `Clone`, `Debug`, `Display`, `PartialEq`, `Eq` y `Hash` (coherentes entre sí: la igualdad de `ActorPath` ignora el uid y la de la raíz es `address` + `name`, `ActorPath.scala:454-466`); no implementan `PartialOrd` ni `Ord`; exponen `pekko_compare(&self, &Self) -> std::cmp::Ordering`, documentado como "orden de Pekko, no total" para `Address` y "preorden total no coherente con `Eq`" para `ActorPath`. Ordenar listas con él queda como pregunta abierta (Q-PATH-1).

**3. Los textos se comparan por unidades UTF-16.** `String.compareTo` de Java compara unidades UTF-16; `str::cmp` de Rust compara bytes UTF-8 y da otro resultado entre U+FF61 y U+10000 (`CMPA-14`, `CMPP-13`). `pekko_compare` usa `encode_utf16()`. Por la misma razón, la posición que informa `validate_path_element` es un índice UTF-16 (`VAL-23`).

**4. Errores por operación, con la clase Java y el mensaje exacto.** Cada operación devuelve sólo los errores que Pekko puede lanzar en ella (tabla abajo). Cada error expone `java_class() -> &'static str` (nombre simple, el que imprime el oráculo) y `message() -> &str`; `Display` es el mensaje. Los mensajes reproducen las rarezas de Pekko byte a byte: el prefijo `requirement failed: ` de `require`, el `'Some(in_valid)'` de interpolar un `Option` (`Address.scala:115`), el `]` que falta en `[5 != 0` (`ActorPath.scala:323`), el espacio final del elemento vacío y el doble espacio cuando hay `fullPath` (`ActorPath.scala:98,106`).

**5. Se reproduce `Integer.valueOf` para el uid, incluidos `+` y los dígitos Unicode.** `splitNameAndUid` (`ActorCell.scala:400-404`) usa `Integer.valueOf`, que acepta `+5`, `-5` y dígitos de cualquier escritura (`#٥` y `#５` dan uid 5: `FS-52/53`). `i32::from_str` de Rust acepta los signos pero no los dígitos Unicode, así que el parseo del uid usa una tabla de dígitos decimales Unicode (categoría Nd, la de Java 21), sin dependencias. Cualquier fallo da `NumberFormatException` con `For input string: "<texto>"`. Ese error escapa de `fromString` sin convertirse en `MalformedURLException`, porque `ActorPathExtractor` sólo atrapa `URISyntaxException` (`Address.scala:229-230`) y la composición `RootActorPath(address) / elems` ocurre después (`ActorPath.scala:73`).

**6. El valor de `hashCode` no se reproduce.** Pekko usa MurmurHash3 (`Address.scala:79`, `ActorPath.scala:469-479`, que además hashea el nombre de la hoja en cada nivel). Sólo se exige "iguales ⇒ mismo hash"; el oráculo imprime `hashEq` sólo para pares iguales. Es una diferencia, por eso tiene Q-PATH-3.

### Qué parte de `java.net.URI` se ve desde la API

No se porta `java.net.URI` entero: sólo lo que cambia el resultado de `ActorPath::from_string`, `Address::parse` y `relative_actor_path_elements`. Cada comportamiento tiene casos en el oráculo:

| Comportamiento del JDK | Efecto observable | Casos |
|---|---|---|
| Sin esquema la URI es relativa | `from_string`/`parse` fallan; `relative_actor_path_elements` acepta | `FS-32/33`, `AD-38..40`, `RP-01/02` |
| URI opaca (`pekko:sys`) tiene path nulo | falla | `FS-21/22`, `AD-33` |
| Autoridad vacía o ausente, o host vacío | falla | `FS-23`, `AD-34/35`, `SPEC-APS-53` |
| Host inválido (`_`, no ASCII, empieza con `-`, IPv4 fuera de rango) cae a autoridad "registry" sin host | falla, también `my_sys` sin `@` | `FS-18/19`, `AD-08`, `AD-24/25`, `AD-30`, `AD-32`, `AD-42`, `AD-44` |
| Host con punto final, mayúsculas | se acepta tal cual | `AD-36`, `AD-43` |
| IPv6 entre corchetes | host conserva los corchetes; `%25` de zona se mantiene | `FS-27`, `AD-20..22`, `AD-50..53` |
| Puerto: sólo dígitos, vacío = ausente, > `Int.MaxValue` = registry | ver decisión 1 | `AD-12..19`, `FS-34/35` |
| Userinfo: admite `:` y no ASCII, se decodifica `%XX` como UTF-8 (U+FFFD si es inválido), escape incompleto falla | `system` sale decodificado | `AD-27..29`, `AD-31`, `AD-48/49`, `FS-29/30` |
| Path crudo: no decodifica `%20`, valida escapes, rechaza espacios, admite no ASCII, no normaliza `.`/`..` | nombres tal cual | `FS-07`, `FS-09..14` |
| Query descartada | `?x` no aparece | `FS-15/16`, `AD-05`, `RP-07` |
| Fragmento = uid del último elemento; admite `?`, rechaza `#` o espacio | ver decisión 5 | `FS-17`, `FS-36..56` |
| Fragmento sin path o tras `/` final | se pierde (raíz) o crea un hijo de nombre vacío | `FS-48..50` |
| Espacio al final de la cadena | falla | `AD-45` |

## Errores por operación

| Operación Rust | Pekko | Clase Java | Mensaje exacto | Casos |
|---|---|---|---|---|
| `Address::parse` | `AddressFromURIString.apply/parse` (`Address.scala:207-215`) | `MalformedURLException` | la entrada tal cual | `AD-*` |
| `ActorPath::from_string` | `ActorPath.fromString` (`ActorPath.scala:72-75`) | `MalformedURLException` | `cannot parse as ActorPath: <entrada>` | `FS-*`, `SPEC-APS-51..55` |
| | uid del fragmento (`ActorCell.scala:403`) | `NumberFormatException` | `For input string: "<fragmento>"` | `FS-17,40,41,43,45,54..56` |
| `ActorPath::child`, `descendant` | `/` (`ActorPath.scala:203-204,290-293,339-342`) | `NumberFormatException` | `For input string: "<tras el primer #>"` | `CH-*`, `DS-07` |
| | `ChildActorPath` (`ActorPath.scala:331-332`) | `IllegalArgumentException` | `/ is a path separator and is not legal in ActorPath names: [<nombre>]` (sin prefijo) | `CH [a/b]`, `DS-06` |
| `ActorPath::root_named` | `RootActorPath` (`ActorPath.scala:280-284`) | `IllegalArgumentException` | `requirement failed: / may only exist at the beginning of the root actors name, it is a path separator and is not legal in ActorPath names: [<n>]` o `requirement failed: # is a fragment separator and is not legal in ActorPath names: [<n>]` | `RT-*`, `SPEC-APS-110` |
| `ActorPath::with_uid` (raíz, uid ≠ 0) | `ActorPath.scala:321-323` | `IllegalStateException` | `RootActorPath must have undefinedUid, [<uid> != 0` | `UID-02/03` |
| `validate_path_element` | `ActorPath.scala:97-115` | `InvalidActorNameException` | vacío: `Actor path element must not be empty <fp>`; inválido: `Invalid actor path element [<e>]<fp>, illegal character [<c>] at position: <i>. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';.` donde `<fp>` es vacío o ` (in path [<p>])` | `VAL-*`, `SPEC-LARP-*` |
| `Address::check_host_characters` | `Address.scala:114-115` | `IllegalArgumentException` | `requirement failed: Using invalid host characters 'Some(<host>)' in the Address is not allowed.` | `HC-*`, `SPEC-APS-133-143` |
| `relative_actor_path_elements` | `Address.scala:170-180` | — (devuelve `None`) | — | `RP-*` |

El chequeo de `#` de `ChildActorPath` (`ActorPath.scala:333-335`) no se puede alcanzar desde la API pública: `/` corta en el primer `#` y lo que sigue, si tiene otro `#`, falla antes como número (`CH [foo#1#2]`). No se porta como variante.

## Preguntas abiertas para el Owner

Cada una cambiaría comportamiento público; todas están anotadas con `Q:` en las filas del inventario.

- **Q-PATH-1 (AC-PATH-002): cómo ordenar listas de `Address`.** Con `slice::sort_by(Address::pekko_compare)` Rust ≥ 1.81 puede entrar en pánico si detecta que el comparador no es un orden total, y aunque no lo haga el resultado puede diferir de `List.sort` de Java para entradas como `CMPA-17`. Alternativas: (a) exponer sólo `pekko_compare` y documentar que no sirve para ordenar mezclas de `None`/`Some("")`/`Some(0)` (recomendada hasta que aparezca un consumidor, por ejemplo cluster); (b) agregar una función que reproduzca el `List.sort` de Java (TimSort) para dar el mismo orden que Pekko; (c) divergir con un desempate `None < Some` y un `Ord` (lo que proponía la versión anterior de este plan; contradice la decisión 2).
- **Q-PATH-2 (AC-PATH-008): surrogate suelto en el mensaje.** Para `x🙂` Pekko informa `illegal character [\uD83D] at position: 1` (`VAL-23`): la mitad de un par UTF-16, que un `String` de Rust no puede contener. Alternativas: (a) poner U+FFFD, la conversión con pérdida estándar de Rust (`String::from_utf16_lossy`; recomendada); (b) poner `?`, que es lo que escribe Java al imprimir ese mensaje en UTF-8; (c) poner el carácter completo `🙂`; (d) guardar el mensaje como UTF-16. La posición sigue siendo el índice UTF-16 en todos los casos.
- **Q-PATH-3 (AC-PATH-001, 014): valor de `hashCode`.** Pekko lo expone como `int`; Rust sólo da `Hash`. Alternativas: (a) no reproducir el valor (recomendada); (b) exponer `pekko_hash_code() -> i32` idéntico, sólo si alguna unidad posterior lo necesita.
- **Q-PATH-4 (AC-PATH-001, 007, 008): `null` de Java.** `fromString(null)` y `validatePathElement(null)` lanzan `NullPointerException`; `new Address(null, "s")` imprime `null://s`. Con `&str` no se puede expresar. Alternativa: aceptarlo como diferencia de lenguaje (recomendada). El caso `host = null` del constructor Java sí se representa: `Address::new(.., None, Some(port))`.
- **Q-PATH-5 (AC-PATH-009): fachada Java `ActorPaths`.** Es la pregunta 7 de `docs/inventory/actor-classic.md`. En Rust las funciones libres cubren el contrato; falta decidir si la fila se excluye, se fusiona con 007/008 o se porta.
- **Q-PATH-6 (AC-PATH-002, 004..007, 011, 014): licencia de `uri.rs` y `jstring.rs`.** `crates/pekko-actor/src/actor/uri.rs` porta el comportamiento del parser y de `decode` de OpenJDK 21 `java/net/URI.java` y las longitudes de entrada malformada de `sun/nio/cs/UTF_8.java`; `jstring.rs` reproduce `String.compareTo`, `Integer.valueOf` y la tabla de `Character.digit` de Java 21. OpenJDK es GPL-2.0 WITH Classpath-exception-2.0, no Apache-2.0. Alternativas: (a) aceptar el código como reimplementación del comportamiento observable y declarar la referencia en el encabezado (estado actual, sin decidir); (b) reescribirlo en sala limpia desde RFC 2396/3629 y los oráculos; (c) depender de un crate externo con licencia compatible y adaptar las desviaciones del JDK. Es una pregunta de licencia, no de semántica: no cambia el comportamiento ni bloquea `impl=done` de esas filas.

## Global Constraints

- Sin dependencias en `crates/pekko-actor/Cargo.toml` para esta unidad.
- Los tests portados de specs comparan como el spec (`include` → `contains`); el oráculo compara la línea completa.
- `cargo test -p pekko-actor` sin `-race` ni equivalentes; `cargo clippy --workspace --all-targets -- -D warnings` limpio.
- Cada archivo Rust lleva el encabezado de atribución de `docs/upstream.md`.

## Contracts

```rust
// crates/pekko-actor/src/actor/address.rs
#[derive(Clone, Debug, PartialEq, Eq, Hash)]   // sin PartialOrd/Ord (decisión 2)
pub struct Address { /* protocol: String, system: String, host: Option<String>, port: Option<i32> */ }

impl Address {
    pub fn local(protocol: impl Into<String>, system: impl Into<String>) -> Address;            // new Address(p, s)
    pub fn remote(protocol: impl Into<String>, system: impl Into<String>,
                  host: impl Into<String>, port: i32) -> Address;                                // Address(p, s, h, port)
    pub fn new(protocol: impl Into<String>, system: impl Into<String>,
               host: Option<String>, port: Option<i32>) -> Address;                              // copy(...)
    pub fn protocol(&self) -> &str;  pub fn system(&self) -> &str;
    pub fn host(&self) -> Option<&str>;  pub fn port(&self) -> Option<i32>;
    pub fn has_local_scope(&self) -> bool;   // host.is_none()
    pub fn has_global_scope(&self) -> bool;  // host.is_some()
    pub fn host_port(&self) -> String;       // to_string() sin "protocol://"
    pub fn parse(s: &str) -> Result<Address, MalformedUrlError>;          // AddressFromURIString
    pub fn pekko_compare(&self, other: &Address) -> std::cmp::Ordering;   // addressOrdering, no total
    #[doc(hidden)] pub fn has_invalid_host_characters(&self) -> bool;     // "_[^.]*$": hay un '_' después del último '.'
    #[doc(hidden)] pub fn check_host_characters(&self) -> Result<(), IllegalArgumentError>;
}
impl std::fmt::Display for Address {} // "p://s", "p://s@h", "p://s:1", "p://s@h:1"

// crates/pekko-actor/src/actor/path.rs
#[derive(Clone)]
pub struct ActorPath(/* Arc<Node> */);   // PartialEq, Eq, Hash (sin uid), Debug, Display; sin Ord

impl ActorPath {
    pub fn root(address: Address) -> ActorPath;                                              // nombre "/"
    pub fn root_named(address: Address, name: &str) -> Result<ActorPath, IllegalArgumentError>;
    pub fn child(&self, name: &str) -> Result<ActorPath, ChildError>;                       // "/" de Pekko
    pub fn descendant<'a>(&self, names: impl IntoIterator<Item = &'a str>) -> Result<ActorPath, ChildError>;
    pub fn from_string(s: &str) -> Result<ActorPath, FromStringError>;
    pub fn address(&self) -> &Address;  pub fn name(&self) -> &str;
    pub fn parent(&self) -> ActorPath;  pub fn root_path(&self) -> ActorPath;  pub fn is_root(&self) -> bool;
    pub fn elements(&self) -> Vec<&str>;   // raíz → [""] (ActorPath.scala:150,295)
    pub fn to_string_without_address(&self) -> String;
    pub fn to_string_with_address(&self, address: &Address) -> String;
    pub fn to_serialization_format(&self) -> String;
    pub fn to_serialization_format_with_address(&self, address: &Address) -> String;
    pub fn pekko_compare(&self, other: &ActorPath) -> std::cmp::Ordering;   // compareTo
    #[doc(hidden)] pub fn uid(&self) -> i32;                                  // 0 = indefinido
    #[doc(hidden)] pub fn with_uid(&self, uid: i32) -> Result<ActorPath, IllegalStateError>;
}
pub fn validate_path_element(element: &str, full_path: Option<&str>) -> Result<(), InvalidActorNameError>;
pub fn is_valid_path_element(element: &str) -> bool;
pub fn relative_actor_path_elements(s: &str) -> Option<Vec<String>>;   // RelativeActorPath.unapply

// crates/pekko-actor/src/actor/errors.rs — todos con java_class() y message(); Display = message
pub struct MalformedUrlError;  pub struct NumberFormatError;  pub struct IllegalArgumentError;
pub struct IllegalStateError;  pub struct InvalidActorNameError;
pub enum FromStringError { MalformedUrl(MalformedUrlError), NumberFormat(NumberFormatError) }
pub enum ChildError { NumberFormat(NumberFormatError), IllegalArgument(IllegalArgumentError) }

// crates/pekko-actor/src/actor/uri.rs (pub(crate)) — lo observable de java.net.URI (tabla de arriba)
pub(crate) struct Uri { /* scheme, user_info (decodificado), host, port: i32 (-1 = ausente), raw_path: Option, raw_fragment */ }
pub(crate) fn parse(s: &str) -> Result<Uri, UriSyntaxError>;
```

## Trazabilidad

**Specs → tests Rust.** Una fila por aserción (o por grupo de aserciones iguales en forma). Rutas relativas a `upstream/pekko/actor-tests/src/test/`. Las columnas son: línea del spec, qué afirma, test Rust a escribir y caso del oráculo.

`scala/org/apache/pekko/actor/ActorPathSpec.scala` (15 tests) → `crates/pekko-actor/tests/actor_path_spec.rs`:

| Línea | Aserción | Test Rust | Oráculo |
|---|---|---|---|
| :27 | `fromString(path.toString) == path` | `actor_path_spec::support_parsing_its_string_rep` | `SPEC-APS-27` |
| :32 | remoto pekko ida y vuelta | `actor_path_spec::support_parsing_remote_paths` | `SPEC-APS-32` |
| :37 | remoto akka ida y vuelta | `actor_path_spec::support_parsing_remote_akka_paths` | `SPEC-APS-37` |
| :42 | `AddressFromURIString` pekko = `Address(pekko,my_sys,Some(host),Some(1234))` | `actor_path_spec::support_parsing_address_from_uri_string` | `SPEC-APS-42` |
| :47 | ídem akka | `actor_path_spec::support_parsing_akka_address_from_uri_string` | `SPEC-APS-47` |
| :51-55 | `""`, `://hallo`, `s://dd@:12`, `s://dd@h:hd`, `a://l:1/b` → `MalformedURLException` | `actor_path_spec::throw_exception_upon_malformed_paths` | `SPEC-APS-51..55` |
| :60-63 | `toString` de raíz, `/user`, `/user/foo`, `/user/foo/bar` | `actor_path_spec::create_correct_to_string` | `SPEC-APS-60..63` |
| :67 | `elements == [user, foo, bar]` | `actor_path_spec::have_correct_path_elements` | `SPEC-APS-67` |
| :73-76 | `toStringWithoutAddress` `/`, `/user`, `/user/foo`, `/user/foo/bar` | `actor_path_spec::create_correct_to_string_without_address` | `SPEC-APS-73..76` |
| :80 | `validatePathElement("")` → mensaje contiene `must not be empty` | `actor_path_spec::validate_path_elements` | `SPEC-APS-80` |
| :90-92 | raíz local con dirección `aaa`: `/`, `/user`, `/user/foo` | `actor_path_spec::create_correct_to_string_with_address` | `SPEC-APS-90..92` |
| :94 | (comentada en el spec) raíz con `bb` | sin test; sólo oráculo | `SPEC-APS-94` |
| :95-96, :98-100 | ídem con `bb` y `cccc` | `actor_path_spec::create_correct_to_string_with_address` | `SPEC-APS-95,96,98..100` |
| :103-105 | raíz remota no cambia de dirección | `actor_path_spec::create_correct_to_string_with_address` | `SPEC-APS-103..105` |
| :110-111 | `RootActorPath(a, "/user/boom/*")` → `IllegalArgumentException` contiene `is a path separator` | `actor_path_spec::not_allow_path_separators_in_root_actor_paths_name` | `SPEC-APS-110` |
| :114 | `fromString("pekko://mysys/user/boom/*")` funciona | ídem | `SPEC-APS-114` |
| :118-124 | `hasInvalidHostCharacters` falso para `valid`, `is_valid.org`, `fu.is_valid.org`; verdadero para `in_valid`, `invalid._org` | `actor_path_spec::detect_valid_and_invalid_chars_in_host_names` | `SPEC-APS-119-124[*]` |
| :126 | `AddressFromURIString("pekko://sys@in_valid:5001")` → `MalformedURLException` | ídem | `SPEC-APS-126` |
| :130-133 | `checkHostCharacters` no falla con hosts válidos | `actor_path_spec::not_fail_fast_if_the_check_is_called_on_valid_chars_in_host_names` | `SPEC-APS-133-143[localhost/is_valid.org/fu.is_valid.org]` |
| :137-143 | ídem, y falla con `IllegalArgumentException` para `in_valid`, `invalid._org` | `actor_path_spec::fail_fast_if_the_check_is_called_when_invalid_chars_are_in_host_names` | `SPEC-APS-133-143[in_valid/invalid._org]` |

`scala/org/apache/pekko/actor/AddressSpec.scala:34` (orden `[akka h1, akka h2, pekko h1, pekko h2]`) → `address_spec::work_correctly_with_multiple_mixed_protocols`, oráculo `SPEC-ADS-34`. El test ordena con `sort_by(Address::pekko_compare)`; esas cuatro direcciones no tocan el caso de Q-PATH-1.

`scala/org/apache/pekko/actor/RelativeActorPathSpec.scala` → `crates/pekko-actor/tests/relative_actor_path_spec.rs`: `:29` → `match_single_name` (`SPEC-RAP-29`); `:32` → `match_path_separated_names` (`SPEC-RAP-32`); `:35-36` → `match_url_encoded_name` (`SPEC-RAP-35`); `:39` → `match_path_with_uid_fragment` (`SPEC-RAP-39`).

`scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:171-197` (sólo los nombres que llegan a `validatePathElement`) → `crates/pekko-actor/tests/local_actor_ref_provider_spec.rs::throw_suitable_exceptions_for_malformed_actor_names`, llamando a `validate_path_element` en lugar de `actorOf`: `$hallo` contiene ``not start with `$` `` (:171, `SPEC-LARP-01`); `a%`, `%3`, `%xx`, `%0G`, `%gg`, `%`, `%1t`, `a?` contienen `Invalid actor path element` (:173-188, `SPEC-LARP-02..09`); `üß` contiene `include only ASCII` (:189, `SPEC-LARP-10`); `he"llo` contiene `["] at position: 2` (:192, `SPEC-LARP-11`); `$hello` contiene `[$] at position: 0` (:194, `SPEC-LARP-12`); `hell>o` contiene `[>] at position: 4` (:196, `SPEC-LARP-13`). Las líneas :169-170 son de AC-CELL.

`java/org/apache/pekko/actor/JavaAPI.java:62` (`ActorPaths.fromString` dentro de `mustCompile`, que Pekko compila pero no ejecuta) → sin test Rust; oráculo `SPEC-JAPI-62`.

**Casos del oráculo → contrato.** `tools/oracle/path-address.jsh` imprime una línea por caso: `<ID> -> ok <campos>` o `<ID> -> ERR <Clase> [<mensaje>]`, con todo lo que no sea ASCII imprimible escrito como `\uXXXX` (unidad UTF-16) para que los espacios finales y los surrogates queden visibles. El test `crates/pekko-actor/tests/oracle_path_address.rs` reconstruye cada caso con la API Rust y compara la línea.

| Prefijo | Cantidad | Contrato | Filas |
|---|---|---|---|
| `SPEC-APS`, `SPEC-ADS`, `SPEC-RAP`, `SPEC-LARP`, `SPEC-JAPI` | 65 | aserciones de los specs (tabla de arriba) | 001-008, 012-014 |
| `FS` | 56 | `from_string`: parser, query, escapes, no ASCII, IPv6, opacas, `_`, userinfo, uid en fragmento | 004, 006, 007, 011 |
| `AD` | 55 | `Address::parse`: dominio del parser, puerto, hosts, userinfo | 004 |
| `RP` | 15 | `relative_actor_path_elements` | 005 |
| `CT` | 21 | dominio del constructor (puertos negativos, `null` como host, `copy`), `Display`, `host_port`, scopes | 001 |
| `HC` | 11 | `has_invalid_host_characters`, `check_host_characters` | 003 |
| `RT` | 11 | `root_named` y sus dos mensajes | 012 |
| `CH`, `DS` | 25 + 8 | `child` (uid, nombre vacío, `/`) y `descendant` (salta vacíos) | 010, 011 |
| `UID` | 7 | `with_uid` en raíz e hijo | 011 |
| `NAV` | 4 | `name`, `parent`, `root_path`, `elements`, `address` | 010 |
| `STR` | 18 | `to_string*`, `to_serialization_format*`, raíz con nombre, `IgnoreActorRef` | 013, 015 |
| `CMPA` | 17 | `Address::pekko_compare`, `Eq`, `Hash` | 002, 001 |
| `CMPP` | 16 | `ActorPath::pekko_compare`, `Eq`, `Hash` | 014 |
| `VAL` | 39 | `validate_path_element`, `is_valid_path_element`, `fullPath`, fachada Java | 008, 009 |

## Tasks

### Task 1: `Address` y errores

**Files:** Create `crates/pekko-actor/src/actor/{mod.rs,address.rs,errors.rs}`; Modify `crates/pekko-actor/src/lib.rs`; Test `crates/pekko-actor/tests/address_spec.rs`, `crates/pekko-actor/tests/actor_path_spec.rs` (sólo los tres tests de host).

- [ ] **Step 1: tests que fallan** — `address_spec::work_correctly_with_multiple_mixed_protocols`; los tres tests de host de `actor_path_spec` (:117-144); unitarios con los valores de `CT-01..19`, `HC-*` y `CMPA-01..16` (incluidos los mensajes exactos y `pekko_compare` por UTF-16).
- [ ] **Step 2:** `cargo test -p pekko-actor` → no compila.
- [ ] **Step 3:** implementar `Address`, `pekko_compare` y los tipos de error (decisiones 1, 2, 3, 4).
- [ ] **Step 4:** el mismo comando pasa.
- [ ] **Step 5:** commit `feat(actor): Address value type (AC-PATH)`.

### Task 2: lo observable de `java.net.URI`, `Address::parse` y rutas relativas

**Files:** Create `crates/pekko-actor/src/actor/uri.rs` (con `#[cfg(test)] mod tests`); Modify `address.rs`, `path.rs` (sólo `relative_actor_path_elements`); Test `crates/pekko-actor/tests/relative_actor_path_spec.rs`, `actor_path_spec.rs` (:40-48).

- [ ] **Step 1: tests que fallan** — los 4 de `relative_actor_path_spec`; `support_parsing_address_from_uri_string` y la versión akka; unitarios de `Address::parse` con los resultados de `AD-*` y de `relative_actor_path_elements` con `RP-*`.
- [ ] **Step 2:** `cargo test -p pekko-actor` → falla.
- [ ] **Step 3:** parser según la tabla "Qué parte de `java.net.URI` se ve desde la API": autoridad "server" con caída a "registry", puerto `i32`, userinfo decodificado, path y fragmento crudos.
- [ ] **Step 4:** pasa.
- [ ] **Step 5:** commit `feat(actor): JDK-compatible URI subset and Address parsing (AC-PATH)`.

### Task 3: validación de elementos de path

**Files:** Modify `crates/pekko-actor/src/actor/path.rs` (funciones libres); Test `crates/pekko-actor/tests/actor_path_spec.rs` (`validate_path_elements`), `crates/pekko-actor/tests/local_actor_ref_provider_spec.rs`.

- [ ] **Step 1: tests que fallan** — `actor_path_spec::validate_path_elements`, `local_actor_ref_provider_spec::throw_suitable_exceptions_for_malformed_actor_names` (13 nombres), y unitarios con el mensaje exacto de `VAL-*` (espacio final, doble espacio con `fullPath`, posición UTF-16). `VAL-22/23` según lo que decida Q-PATH-2; mientras tanto se marcan `#[ignore = "Q-PATH-2"]`.
- [ ] **Step 2:** falla. **Step 3:** implementar (`ValidSymbols = "-_.*$+:@&=,!~';"`, `%` seguido de dos hex, `$` sólo prohibido en la posición 0). **Step 4:** pasa.
- [ ] **Step 5:** commit `feat(actor): actor path element validation (AC-PATH)`.

### Task 4: `ActorPath`

**Files:** Modify `crates/pekko-actor/src/actor/path.rs`; Test `crates/pekko-actor/tests/actor_path_spec.rs`.

- [ ] **Step 1: tests que fallan** — los tests restantes de `actor_path_spec` (tabla de trazabilidad) más unitarios con los valores de `FS-*`, `CH-*`, `DS-*`, `RT-*`, `UID-*`, `NAV-*`, `STR-*` y `CMPP-*`. Puntos a no olvidar: `NumberFormatException` sale de `from_string` sin convertirse (decisión 5); `#٥` da uid 5; `pekko://sys/user/#5` crea un hijo de nombre vacío; la raíz con nombre `/x` concatena sin separador (`STR-09`); `IgnoreActorRef` se detecta por igualdad e ignora el uid incluso en `to_serialization_format_with_address` (`STR-15`).
- [ ] **Step 2:** falla. **Step 3:** implementar; `from_string` = `ActorPathExtractor` (`split` de `Address.scala:149-160`, descarta el primer elemento, `descendant` ignora vacíos). **Step 4:** pasa.
- [ ] **Step 5:** commit `feat(actor): ActorPath (AC-PATH)`.

### Task 5: validación diferencial contra Pekko compilado desde la fuente

**Files:** Regenerate `crates/pekko-actor/tests/oracle/path-address.golden`; Create `crates/pekko-actor/tests/oracle_path_address.rs`; Modify `docs/inventory/actor-classic.tsv` (filas `AC-PATH-*`).

- [ ] **Step 1:** `tools/oracle/run.sh tools/oracle/path-address.jsh > crates/pekko-actor/tests/oracle/path-address.golden` con el runner que compila desde `upstream/pekko` (no con `ORACLE=jars`). Revisar que tenga 368 líneas y ninguna fuera del formato.
- [ ] **Step 2:** `oracle_path_address.rs` reconstruye cada caso con la API Rust, imprime la misma línea (mismo escape `\uXXXX`, `java_class()` y `message()` del error) y compara con el golden. `cargo test -p pekko-actor --test oracle_path_address`.
- [ ] **Step 3:** corregir en Tasks 1-4 lo que difiera. Una diferencia que no se corrige va a `notes` del inventario con `Q:`; el test la marca explícitamente con el id de la pregunta, nunca la silencia.
- [ ] **Step 4:** `cargo test --workspace` y `cargo clippy --workspace --all-targets -- -D warnings` limpios.
- [ ] **Step 5:** inventario: `validation=oracle+spec` en las filas cubiertas; `impl=done` sólo en filas sin `Q:` abierta y sin dependencia de unidades futuras. `AC-PATH-009`, `011` y `015` quedan `partial` siempre (ver arriba), igual que cualquier fila con Q-PATH-1..5 sin resolver. Commit `test(actor): differential oracle for AC-PATH`.

**Siguiente spec en la cadena:** `AC-CFG` (configuración HOCON y `reference.conf`), ver `docs/implementation-order.md`.
