# Golden de HOCON ampliado con reference.conf de Pekko 2.0.0 (condición de Q-X-1, prerrequisito de P07)

Fecha: 2026-11-18. Rama `chore/hocon-golden-reference-keys`.

## Qué cambió
- `oracle/src/main/java/pilot/ConfigHoconOracle.java` (nuevo): los 161 casos del script jshell `config-hocon.jsh` portados al CLI del oráculo (`config-hocon-ids`, `config-hocon-all`, `config-hocon-case`), más un caso `KEY <archivo> <ruta>` por cada clave hoja de los dos `reference.conf`. Fuente: Typesafe Config 1.4.9 con Pekko 2.0.0 en JDK 17.
- `oracle/pom.xml`: nueva dependencia `pekko-actor-typed_2.13:2.0.0` (para leer su `reference.conf` desde el classpath del oráculo).
- Q-CFG-4 (re-grabar con el CLI): `crates/rakka-actor/tests/oracle/config-hocon.golden` se regeneró con `oracle/run.sh config-hocon-all`. Las 161 líneas de casos escritos a mano son **idénticas** a las del golden anterior (diff vacío); sólo cambia la cabecera (fuente y hash del oráculo) y se añaden las líneas `KEY`.
- `cases/oracle-ids/config-hocon.txt`: los 469 ids (161 + 308). No hay casos `rust-anything` grabados todavía: P07 los registra cuando Rust produzca cada id (`tools/parity/make_oracle_cases.py` exige que Rust produzca los ids que graba).

## Claves agregadas (308)
| Archivo (Pekko 2.0.0) | Claves hoja |
|---|---|
| `actor/src/main/resources/reference.conf` (`pekko-actor_2.13-2.0.0.jar`) | 281 |
| `actor-typed/src/main/resources/reference.conf` (`pekko-actor-typed_2.13-2.0.0.jar`) | 27 (las que añade o cambia sobre el de `actor`) |

La lista exacta, una por línea con su tipo y valor resuelto, está en las líneas `KEY ...` del golden (y los ids en `cases/oracle-ids/config-hocon.txt`). Una hoja es todo valor que no es un objeto: escalar, lista o `null`; las rutas con comillas conservan el formato `ConfigUtil.joinPath`. Quedan fuera `actor-testkit-typed` y `actor-tests` (testkit/tests, fuera del alcance de `rakka-actor` y `rakka-actor-typed`).

## Comprobaciones
- Los `reference.conf` de los jars son byte a byte los del submódulo `upstream/pekko` (v2.0.0, 9fbd2283).
- Recuento independiente con `Config.entrySet()`: 281 (actor) y 27 (typed añadidas o cambiadas), igual que los casos `KEY`.
- Dos corridas seguidas dan salida idéntica; con `LANG`/`TZ`/`-Dpekko.loglevel` distintos, también (no se usa `ConfigFactory.defaultReference()` porque superpone propiedades del sistema).
- Limitación: la parte de typed se calcula resolviendo con el de actor como fallback y quitando lo que no cambia (`Config#resolveWith` falla con el `library-extensions` autorreferente de typed).
