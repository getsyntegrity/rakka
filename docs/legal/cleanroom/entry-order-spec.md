# Especificación (sucia): orden de iteración de `config.root` en `Deployer` (Q-CFG-36, Q-X-21)

Rol: especificación. **No contiene código de OpenJDK ni pseudocódigo traducido.** Fuentes permitidas: Javadoc público de `java.util.Map.Entry`, `HashSet`, `HashMap` y `String`; el código de Typesafe Config 1.4.9 (Apache-2.0, `config-1.4.9-sources.jar` de Maven Central; no es OpenJDK); y el oráculo (Pekko 2.0.0 sobre JDK 17). Q-X-18 prohíbe leer código de OpenJDK: el comportamiento de `HashMap`/`HashSet` que no está en el Javadoc se toma **solo de mediciones**. La implementación la hace una sesión que lee esta especificación y no el código de OpenJDK.

## 1. Qué decide el orden

`Deployer` itera `config.root.asScala` (Deployer.scala:236-241) y llama a `deploy` por entrada; si varias claves dan el mismo path, gana la última desplegada (y con varias entradas inválidas se informa la primera en ese orden). `SimpleConfigObject.entrySet()` (Typesafe Config 1.4.9, `SimpleConfigObject.java:618-630`) crea un `HashSet` de `SimpleImmutableEntry(clave, valor)` recorriendo el `HashMap` interno de la configuración.

## 2. Hechos documentados (Javadoc)

- `Map.Entry.hashCode()` = `hashCode(clave) XOR hashCode(valor)` (Javadoc de `Map.Entry`).
- `String.hashCode()` = polinomio de 31 sobre unidades UTF-16 (ya implementado, `util::scala_hash_order::string_hash_code`).

## 3. Hechos medidos del `hashCode` de `ConfigValue` (`H.java`, `H2.java`; JDK 17, config 1.4.9)

- Entero o número entero representable como `long` (`1`, `1.0`, `1e5`, `-0.0`, `3000000000`): hash del `long` como `Long.hashCode` (`(int)(v ^ v>>>32)`); `-1` da 0, `4294967296` da 1, `1.0E10` da 1410065410.
- Número no entero (`1.5`, `0.1`, `1e30`): `Double.hashCode` de sus bits (`1.5` da 1073217536).
- `true` 1231, `false` 1237, `null` 0.
- Cadena (entre comillas o no): `String.hashCode` (`""` 0, `a` 97, `é` 233). Un número que no cabe en `long` ni en `double` exacto se lee como cadena (`12345678901234567890` da -1902336138).
- Lista: `List.hashCode` de los hashes de sus elementos (`[]` 1, `[1]` 32, `[1,2]` 994, `[2,1]` 1024).
- Objeto: `41 * (41 + hash(lista de claves ordenadas)) + suma de los hashes de los valores`, con aritmética de 32 bits (`{}` 1722, `{a=1}` 6930, `{b=1}` 6971, `{a=1,b=1}` 168389, `{a=1,b=2}`/`{a=2,b=1}` 168390; el orden de escritura no importa). Comprobado en 11 objetos.

## 4. Hechos medidos del orden (`E.java`, `F.java`)

Modelo candidato: el `HashSet` final se llena en el orden de iteración del mapa interno; la tabla empieza en 16 posiciones y se duplica cuando hay más de 3/4 de ocupación; el cubo de un elemento es `(h XOR (h >>> 16)) AND (tamaño - 1)` con `h` el hash de la entrada; la iteración recorre los cubos de menor a mayor y, dentro de un cubo, en orden de inserción.

- `parseString` de 400 objetos de 2 a 14 claves con valores enteros (`E.java`, semilla 3): el modelo reproduce el orden **400 de 400**, tomando como orden del mapa interno el que da el mismo modelo con el hash de la clave sola (orden de lectura en los desempates).
- `pekko.actor.deployment` tras `withFallback(defaultReference()).resolve()` con 1 a 12 entradas de dispatcher (`F.java`, 300 casos): el orden de cubos del conjunto final no decrece en **300 de 300**; el orden exacto con el modelo anterior acierta **279 de 300**. Los 21 fallos son desempates dentro de un mismo cubo: ahí manda el orden del mapa interno, que depende de cómo se construyó (lectura, `withFallback`, `resolve`; Typesafe crea nuevos `HashMap` en `mergedWithObject` y `resolve`).

## 5. Modelo completo (medido 300/300)

Lectura de `SimpleConfigObject.mergedWithObject` y `modifyMayThrow` (Typesafe Config 1.4.9, Apache-2.0, líneas 255-330): la fusión copia las claves de ambos objetos a un `HashSet` nuevo (primero las del receptor, luego las del fallback), y recorre ese conjunto para llenar un `HashMap` nuevo; `resolve` rehace el mapa recorriéndolo en su orden. El orden final de un objeto se obtiene así, con `orden_por_cubos(elementos, hash)` = ordenar por `(h XOR (h >>> 16)) AND (tabla - 1)` y, a igual cubo, por el orden en que entraron; la tabla es 16 y se duplica mientras haya más de 3/4 de ocupación:

1. Objeto del usuario: `orden_por_cubos(claves en orden de lectura, hash de la clave)`.
2. `allKeys`: `orden_por_cubos(claves del usuario en el orden 1, seguidas de las claves del fallback que faltan en el orden de su propio objeto, hash de la clave)`.
3. Mapa interno fusionado: `orden_por_cubos(allKeys, hash de la clave)` (no cambia el orden).
4. `entrySet()`: `orden_por_cubos(mapa interno, hash de la clave XOR hash del valor)`.

Comprobación: 300 configuraciones de 1 a 12 entradas (`F.java`, `R.java`; claves `/svc`, `/svc/`, `/svc//`, `/w1`...) con el orden de las claves del fallback de `reference.conf` `default, /IO-DNS/async-dns, /IO-DNS/async-dns/*, /IO-DNS/inet-address/*, /IO-DNS/inet-address`: orden exacto en **300 de 300**; objetos leídos con `parseString` (paso 1 y 4): **400 de 400**.

## 5b. Lo que sigue sin medir

1. Cubos con 8 o más entradas (tabla de 64 o más): el Javadoc de `HashMap` dice que los cubos grandes pueden pasar a árboles; medir si cambia el orden.
2. Objetos de la configuración que pasan por más operaciones (varios `withFallback` encadenados, `include`, sustituciones): cada una rehace el mapa con la misma regla, pero no está medido.
3. Valores sin resolver no aplican (`Settings` entrega una configuración resuelta).

## 6. Interfaz propuesta para la implementación (Rust)

`Config::object_entries_in_pekko_order()` en `config/`, usando `hash_code_of_value(&Val) -> i32` y una función pura `entry_set_order(&[(key_hash, value_hash)], underlying_order) -> Vec<usize>`; `Deployer::from_settings` y `DeployerSettingsOracle` la usan en vez del orden alfabético. Casos: CO-01..CO-05, los 99 conjuntos de `p10-from-settings-collisions-probe.java`, y un fuzz diferencial nuevo con claves en colisión y valores de varios tipos.

## 7. Registro de proceso (Q-X-18)

La parte sucia (esta especificación y las sondas) se escribe sin leer OpenJDK. La implementación (PR-D1) solo lee esta especificación; la similitud se comprueba como en `uri-similarity.md`. Pendiente: log y similitud al cerrar PR-D1.
