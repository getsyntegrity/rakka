# Contratos verificables: classic-foundation (UTIL, CFG, PATH, EXT)

Grupo `classic-foundation` del inventario de `pekko-actor` (Apache Pekko 1.7.1, commit `9a54d129…`). Cada bloque sigue el orden de `docs/inventory/rows/classic-foundation.tsv`. Todas las rutas `path:line` son relativas a `upstream/pekko/` salvo las que empiezan por `crates/`, `tools/` o `docs/superpowers/`. Las decisiones `Q-X-*` son transversales del orquestador (HOCON Q-X-1, registro de fábricas Q-X-2, mensajes dinámicos Q-X-3, runtime Q-X-4, fallo Q-X-5, jerarquía EventStream Q-X-6, loggers Q-X-7, Done/Timeout/null Q-X-9); las específicas del grupo están en `classic-foundation.decisions.tsv`.

**Procedencia de los oráculos.** `util-core.golden` y `path-address.golden` se generaron con los *jars oficiales* de Pekko 1.7.1 (pekko-actor, scala-library 2.13.18) y JDK 11 vía `tools/oracle/run.sh`; `config-hocon.golden` con el jar `config-1.4.9`. Ninguno es Pekko compilado desde `upstream/pekko`. Cuando discrepen de la fuente, gana la fuente (docs/upstream.md). `scala-library`, `java.net.URI` y Typesafe Config no están en `upstream/pekko`: su comportamiento sólo se conoce por esos oráculos.

**Nota del cierre documental (2026-10-31).** Los valores `medido con el jar` de este archivo se obtuvieron con `ORACLE=jars tools/oracle/run.sh <script.jsh>` (jars publicados de Pekko 1.7.1, JDK 11 Temurin), con scripts ad hoc que aún no están persistidos en `tools/oracle/`. Ojo: la cabecera actual de `tools/oracle/run.sh` indica que el modo por defecto (`ORACLE=source`) compila Pekko desde `upstream/pekko`, lo cual no coincide con la «Procedencia» de arriba (golden generados con jars); si el golden se regenera, esa procedencia debe actualizarse.

**Convención de aserciones.** `A<n>` con `path:line` = el cuerpo del test de Pekko lo afirma (leído). *no afirmada por spec* = derivada del código o del oráculo. Las filas `listed` no tienen aserciones extraídas.


---

## AC-UTIL-001 — Timeout (duración para ask/resolveOne)

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** `Timeout` envuelve un `FiniteDuration` (case class, igualdad por valor). `Timeout.zero` = `Timeout(Duration.Zero)`; `Timeout(length, unit)` = `Timeout(Duration(length, unit))`; `Timeout.create(java.time.Duration)` convierte con `JavaDurationConverters.asScala`; la conversión implícita `FiniteDuration => Timeout` permite pasar una duración donde se pide un `Timeout` (ask, resolveOne). Un `Timeout` demasiado corto hace fallar `ask` con `AskTimeoutException`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Timeout.scala:22-29 (case class + constructor (Long, TimeUnit))`
- `actor/src/main/scala/org/apache/pekko/util/Timeout.scala:34-55 (zero, apply, create, durationToTimeout)`
- `actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala:25-35 (1.7.1)` (`asScala = Duration.fromNanos(self.toNanos)`; `Duration.asJava = JDuration.ofNanos(toNanos)`; el desborde de `toNanos` es el `ArithmeticException` de TO-NEG-04/05; leído en el cierre documental 2026-10-31)
- `reference.conf: n/a (Timeout no lee claves; los consumidores `creation-timeout`, `unstarted-push-timeout`, `logger-startup-timeout` lo construyen en ActorSystem.scala:423-424,439)`

**Condiciones.**
- Normal: `Timeout(100.millis).duration == 100.millis`; `Timeout(1000.millis) == Timeout(1.second)` (igualdad numérica del FiniteDuration).
- Límites: `Duration(length, unit)` lanza `IllegalArgumentException("requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)")` si excede el rango; 106751 DAYS ok, 106752 DAYS falla (oráculo TO-*). El rango de `FiniteDuration` es [-(2^63-1), 2^63-1] ns, con negativos, cero y positivos: `Timeout(-5.millis)` es válido (TO-NEG-01) y `ask` lo rechaza después (AC-ASK-001). `Long.MinValue` ns está fuera: `Timeout(Long.MinValue, NANOSECONDS)` y `create(ofNanos(Long.MIN_VALUE))` lanzan `IllegalArgumentException` con el mismo mensaje (TO-NEG-02; medido en los jars con `Timeout.create`). `-(2^63-1)` ns es válido en cada unidad y `-max-1` falla (medido en los jars).
- Errores: `Timeout.create(ofSeconds(9223372036, 854775808))` y `create(ofSeconds(Long.MAX_VALUE))` lanzan `ArithmeticException: long overflow` (TO-NEG-04, TO-NEG-05); en Rust son el `Err(ArithmeticError)` de `TryFrom<std::time::Duration>`, sin saturar ni truncar.
- toString conserva la unidad: `Timeout(100 milliseconds)`, `Timeout(0 days)` (TO-STR-01..03).
- Nota de procedencia: scala-library (scala.concurrent.duration) no está en upstream/pekko: su comportamiento sólo se conoce por el oráculo de jars.

**Orden / concurrencia / ciclo de vida.** n/a: valor inmutable, sin hilos ni hooks.

**Dependencias.** Ninguna fila. Externas: `scala.concurrent.duration.{Duration, FiniteDuration}`, `java.util.concurrent.TimeUnit`. Consumidores: AC-ASK-001, AC-CFG-002 (CreationTimeout, UnstartedPushTimeout, LoggerStartTimeout).

**Equivalente Rust propuesto y diferencias observables.** `pub struct Timeout(i64)` (`Copy`, `PartialEq`, `Eq`, `Hash`; campo privado; invariante único |nanos| <= 2^63-1) con `Timeout::ZERO`, `Timeout::new(nanos: i64) -> Result<Timeout, IllegalArgumentError>`, `Timeout::of(length: i64, TimeUnit) -> Result<Timeout, IllegalArgumentError>`, `impl TryFrom<std::time::Duration> for Timeout` (`Error = ArithmeticError`) y `as_nanos() -> i64`. Implementado en crates/rakka-actor/src/util/timeout.rs (decisión Q-UTIL-1 del Owner, 2026-10-31). Diferencias observables que quedan: `Debug` no conserva la unidad (Q-UTIL-2, abierta); `std::time::Duration` no puede ser negativa, así que los negativos se construyen con `new`/`of`, no con `try_from`.

**Aserciones.**
- A1 `Timeout(testTimeout)` implícito en `echo ? "hallo"` a un actor que no responde falla con `AskTimeoutException` dentro de testTimeout+leeway: ActorTimeoutSpec.scala:34-39 (la aserción es de ask, AC-ASK-001).
- A2 el mismo resultado con timeout explícito `echo.?("hallo")(testTimeout)`: ActorTimeoutSpec.scala:41-45.
- A3 `Timeout.zero.duration == Duration.Zero`: derivada del código Timeout.scala:39; oráculo TO-* (*no afirmada por spec*).
- A4 `Timeout(length, unit)` fuera de rango lanza IllegalArgumentException con el mensaje exacto de arriba: oráculo TO-* (*no afirmada por spec*).
- A5 `Timeout(1000.millis) == Timeout(1.second)` (también para negativos: `Timeout(-1000.millis) == Timeout(-1.second)` y mismo hash, medido en los jars), `Timeout(-5.millis)` válido, `toString` con unidad: oráculo TO-*, TO-NEG-*, TO-STR-* (*no afirmadas por spec*). Activos en Rust: TO-* y TO-NEG-01..05; TO-STR-* siguen pendientes (Q-UTIL-2).

**Oráculo.** Sí, con jars oficiales: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko. Los jars son el artefacto publicado de 1.7.1, no el commit fijado compilado; si discrepan gana la fuente (docs/upstream.md).

**Decisiones.** Q-UTIL-1 (negativos/rango): resuelta por el Owner el 2026-10-31. Q-UTIL-2 (toString con unidad): abierta; la fila sigue `partial`.


---

**Actualización P02 (2026-11-18, Pekko 2.0.0 en JDK 17).** `Timeout.create(java.time.Duration)` ya no pasa por `JavaDurationConverters.asScala` sino por `scala.jdk.DurationConverters.toScala`: `create(ofSeconds(9223372036, 854775808))` lanza `IllegalArgumentException: Java duration PT2562047H47M16.854775808S cannot be expressed as a Scala duration` y `create(ofSeconds(Long.MAX_VALUE))` lanza `IllegalArgumentException: requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)` (casos uc-0033 y uc-0034); el texto «ArithmeticException» de este bloque corresponde a 1.7.1. Q-UTIL-2 respondida (a): el `Debug` de Rust no promete el formato de Pekko (TO-STR-* no se reproducen).

## AC-UTIL-002.1 — Helpers.base64 / base64chars (nombres generados `$a`, temp paths)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** `Helpers.base64(l, sb)` agrega al builder (por defecto `"$"`) un carácter por cada 6 bits de `l`, empezando por los menos significativos, hasta que el resto sea 0; con `l = 0` agrega un solo carácter.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:117-125`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:441-449 (tempPath usa base64 con prefijo)`
- `Children.scala:126,130 (nombres anónimos `$a`)`

**Condiciones.**
- Normal: `base64(0) == "$a"`, `base64(63) == "$~"`, `base64(64) == "$ab"` (oráculo B64-*).
- Negativos: `l.toInt & 63` y `l >>> 6` sin signo, por lo que `base64(-1)` produce 11 dígitos tras el `$` (oráculo B64-*).
- Con `sb` dado, agrega sin escribir el prefijo `$` (el prefijo lo pone el valor por defecto del parámetro).
- Errores: ninguno posible.

**Orden / concurrencia / ciclo de vida.** Pura; el contador de temp paths (`AtomicLong`) vive en LocalActorRefProvider (AC-PATH-015).

**Dependencias.** Ninguna. Consumidores: AC-PATH-015, AC-CELL (nombres anónimos).

**Equivalente Rust propuesto y diferencias observables.** `base64(l: i64)` y `base64_append(l, &mut String)` (implementado). Sin diferencias observables.

**Aserciones.**
- A1 alfabeto y orden de dígitos: oráculo B64-* (*no afirmada por spec*; ningún test de upstream llama a `base64` directamente).
- A2 negativos con `>>>`: derivada del código Helpers.scala:119-122 y oráculo B64-*.
- A3 prefijo propio con `sb` dado: derivada del código Helpers.scala:118; oráculo B64-* (*no afirmada por spec*).

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko.

**Decisiones.** Ninguna.


---

## AC-UTIL-002.2 — Helpers.toRootLowerCase

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Convierte a minúsculas con el locale raíz (sin reglas del locale del sistema, p. ej. turco).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:42`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:440,448 (log-dead-letters y suspend-duration)`

**Condiciones.**
- Normal: ASCII y Unicode usan el mapeo completo de minúsculas (`İ` -> `i` + U+0307, sigma final contextual).
- Errores: ninguno.

**Orden / concurrencia / ciclo de vida.** n/a: pura.

**Dependencias.** Consumidores: AC-CFG-005, AC-EVT (Logging).

**Equivalente Rust propuesto y diferencias observables.** `str::to_lowercase`; diferencia posible por versión de Unicode (Q-UTIL-7).

**Aserciones.**
- A1 mapeo completo y sigma final: oráculo LC-* (*no afirmada por spec*).
- A2 medido con el jar (JDK 11, Unicode 10): `toRootLowerCase("\u1C90")` (GEORGIAN MTAVRULI CAPITAL LETTER AN, añadida en Unicode 11) devuelve el mismo carácter, mientras `char::to_lowercase` de Rust (tablas de Unicode >= 11) lo pasa a U+10D0. La salida de Pekko depende de la JDK que lo ejecute (Unicode 10 en JDK 11, 13 en JDK 17, 15 en JDK 21), no de Pekko; ver Q-UTIL-7.

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko (limitado a Unicode ≤ 10).

**Decisiones.** Q-UTIL-7.


---

**Actualización P02 (2026-11-18, JDK 17).** La JDK 17 implementa Unicode 13: `toRootLowerCase("U+1C90")` da U+10D0 (caso uc-0083), igual que Rust; la divergencia de A2 (medida con JDK 11) sólo subsiste para caracteres añadidos después de Unicode 13, p. ej. U+A7C0, que JDK 17 deja igual y Rust pasa a U+A7C1 (Q-UTIL-7 (a), aceptada).

## AC-UTIL-002.3 — Helpers.Requiring (requiring con IllegalArgumentException)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Extensión de cualquier valor para validar argumentos con `IllegalArgumentException` en lugar de `AssertionError`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:145-170 (usa `require` de Predef)`
- `actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala:60 y dispatch/affinity/AffinityPool.scala:321 (consumidores)`

**Condiciones.**
- Normal: cond verdadera -> devuelve `value` sin copiar.
- Error: `IllegalArgumentException` con mensaje `requirement failed: <msg>` (Predef.require); `msg` null -> `requirement failed: null`.
- Efectos: `msg` se evalúa sólo si falla.

**Orden / concurrencia / ciclo de vida.** n/a: síncrona y pura.

**Dependencias.** Consumidores: AC-SCH-008 (tick-duration), AC-DSP-013.

**Equivalente Rust propuesto y diferencias observables.** Trait con impl genérico (implementado); mensaje `impl Display` formateado sólo al fallar.

**Aserciones.**
- A1 devuelve el valor / lanza con el prefijo exacto: oráculo RQ-* (*no afirmada por spec*).
- A2 forma con predicado `A => Boolean`: derivada del código Helpers.scala:164-167.

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko.

**Decisiones.** Q-X-9 (Q-UTIL-3).


---

## AC-UTIL-002.4 — ConfigOps.getMillisDuration / getNanosDuration

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Lectores de duración usados por Settings y por los configuradores (`creation-timeout`, `unstarted-push-timeout`, `logger-startup-timeout`, `log-dead-letters-suspend-duration`, Dispatchers.scala:304-306).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:175-178,196-197`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:424-425,438,450`
- `Dispatchers.scala:304-306 (consumidor; fuera de mi grupo)`

**Condiciones.**
- Normal: unidades `ns us ms s m h d` y nombres largos; sin unidad = ms.
- Error: clave ausente -> ConfigException.Missing; tipo erróneo -> WrongType; valor mal formado -> BadValue (mensajes en golden DUR-ERR-*).
- Negativos válidos en Typesafe Config (REF-05/DUR-*).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-CFG-017 (formato de duraciones), AC-CFG-001.

**Equivalente Rust propuesto y diferencias observables.** Métodos del tipo `Config` propio (Q-X-1). Diferencia: `std::time::Duration` sin negativos (Q-X-9).

**Aserciones.**
- A1 `getDuration` en ms para `5s` = 5000, `5 minutes` = 300000: ConfigSpec.scala:52,69,76,87.
- A2 tabla de unidades, truncado y desborde: golden DUR-MS-*/DUR-NS-*/DUR-ROUND-*/DUR-LARGE-* (jar Typesafe Config 1.4.9).

**Oráculo.** Sí: golden de HOCON (jar config-1.4.9, tools/oracle/config-hocon.jsh), no Pekko compilado.

**Decisiones.** Q-X-1, Q-X-9.


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `Config::get_millis_duration` / `get_nanos_duration` (`config/mod.rs`) sobre un parser HOCON propio; casos grabados ch-* (DUR-MS/NS/ERR/ROUND/LARGE/UNQUOTED/NUMBER). Pasa a `done` (reevaluación 2026-11-19): A1 y A2 cubiertas por los casos grabados DUR-* (ch-*); Q-X-1 y Q-CFG-4 están aceptadas y sus condiciones se cumplieron en #243; Q-X-9 es un grupo cuyos miembros Q-UTIL-1..5 ya están respondidos.

## AC-UTIL-002.5 — ConfigOps.renderWithRedactions (toString de Settings con datos sensibles ocultos)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `Settings.toString` = config renderizada con redacciones (PR apache/pekko#771, 1.0.2).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:180-194`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:491`

**Condiciones.**
- Normal: render con `ConfigRenderOptions.defaults()` (comentarios, JSON-like, formateado) pero sin valores de entorno.
- Edge: `username` vacío o null -> `String.replace(null,...)` lanzaría NPE (no hay test); derivada del código.
- Efecto: sólo lectura.

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-CFG-001, AC-CFG-002.

**Equivalente Rust propuesto y diferencias observables.** Función de render del `Config` propio; no se exige igualdad textual con Typesafe (Q-CFG-4).

**Aserciones.**
- A1 el texto no contiene valores de entorno: derivada del código Helpers.scala:190 (*no afirmada por spec*; no hay test en upstream, grep sobre actor-tests sin resultados para `renderWithRedactions`).

**Oráculo.** Parcial: se puede comparar con el jar config-1.4.9 el efecto de `setShowEnvVariableValues(false)`; no existe escenario aún.

**Decisiones.** Q-CFG-4.


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `Config::render_with_redactions`; casos grabados RW-01..06 (formato con `# String: N`, `# env variables`, `<env variable>`, `<username>`). Queda `partial`: depende de Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida (RW-01..06 cubren el texto y el entorno).

**Actualización 2026-11-28 (Q-CFG-15 (a), aceptada con cambios): reparto de la verificación de archivos entre esta fila y AC-CFG-001, sin eliminar criterios.** Esta fila es dueña del **render** por origen; la asignación del origen al cargar y los textos de error son de AC-CFG-001 (A6).
- A2 el render antepone el origen de cada valor de texto (`# String: <línea>`) y de variable de entorno (`# env variables`, valor oculto): RW-01..06 (cubierta; A1 sigue vigente).
- A3 el render de un valor leído de un archivo antepone `# <ruta>: <línea>`, con el texto que dé Typesafe Config 1.4.9 para un archivo: **pendiente** (sin implementar ni medir). Depende de AC-CFG-001 A6.
- **DEFECTO ABIERTO: fuga de redacción (A1 incumplida).** Visto por lectura, sin ejecutar: `fold_pieces` (`crates/rakka-actor/src/config/resolve.rs`) crea la concatenación con el origen de su primera pieza; en `"prefijo-"${?SECRETO}` la primera pieza es texto, el valor pierde el origen de entorno (línea 0) y `render_with_redactions` lo imprime en claro bajo `# String: N`. RW-02 sólo cubre el valor de entorno suelto. Pendiente: test que lo reproduzca (RED), medir con el oráculo el texto de Typesafe y corregir. La fila sigue `partial`.

## AC-UTIL-003.1 — Done (singleton de finalización sin valor)

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Valor único `Done` que se usa como resultado de futuros que completan sin valor.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/Done.scala:21-41`

**Condiciones.**
- Normal: una sola instancia; `toString` = "Done" (case object).
- Serializable (marcador JVM).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** Consumidores: AC-SYS (whenTerminated?), AC-CS, AC-ASK (gracefulStop).

**Equivalente Rust propuesto y diferencias observables.** `struct Done;` implementado; `getInstance`/`done` son fachada Java sin equivalente (DN-03).

**Aserciones.**
- A1 `Done.toString == "Done"`: oráculo DN-01 (*no afirmada por spec*).
- A2 singleton: derivada del código Done.scala:27.

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko.

**Decisiones.** Q-X-9 (Q-UTIL-5).


---

## AC-UTIL-003.2 — NotUsed (marcador de valor sin importancia)

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Valor `NotUsed` para tipos genéricos donde el valor materializado no importa (streams, fuera de alcance, pero el tipo está en pekko-actor).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/NotUsed.scala:20-42`

**Condiciones.**
- Normal: singleton, `toString` = "NotUsed".
- Diferencia con Done: no extiende Serializable.

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** Ninguna.

**Equivalente Rust propuesto y diferencias observables.** `struct NotUsed;` implementado.

**Aserciones.**
- A1 `toString == "NotUsed"`: oráculo DN-02 (*no afirmada por spec*).

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko.

**Decisiones.** Q-X-9 (Q-UTIL-5).


---

## AC-UTIL-004.1 — PekkoException (excepción base)

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=oracle+spec*

**Contrato observable.** Excepción base de Pekko: `PekkoException(message, cause)` y `PekkoException(msg)` (causa null).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/PekkoException.scala:17-22`

**Condiciones.**
- Normal: `getMessage`, `getCause`; `toString` = `org.apache.pekko.PekkoException: <msg>` (JVM).
- Edge: `new PekkoException(null)` tiene `getMessage == null` y se imprime sin `: ` (EX-NULL).
- Constructor (String) obligatorio: lo exige PekkoExceptionSpec.

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** Base de ActorKilledException (verify en el spec) y de ConfigurationException.

**Equivalente Rust propuesto y diferencias observables.** Tipo con `message()`, `cause()`, `java_class()`, `Display`, `Error::source` (implementado). Sin jerarquía de subtipos (Q-X-9).

**Aserciones.**
- A1 existe un constructor que acepta un único String en PekkoException: PekkoExceptionSpec.scala:29-33,39-41 (comprobación por reflexión JVM; en Rust se traduce a "se construye con un único mensaje").
- A2 lo mismo para ActorKilledException: PekkoExceptionSpec.scala:35 (fila de AC-SUP/AC-CELL; aquí sólo se registra).
- A3 mensaje/causa/`Display`: oráculo EX-01..06 (*no afirmada por spec*).

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko.

**Decisiones.** Q-X-9 (Q-UTIL-3).


---

## AC-UTIL-004.2 — ConfigurationException

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Excepción para configuración inválida; un `catch PekkoException` también la atrapa.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/PekkoException.scala:38-43`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:478-486 (usos)`

**Condiciones.**
- Mensajes fijados por el código: `pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.` (ActorSystem.scala:479-481) y `Pekko JAR version [<v>] does not match the provided config version [<c>]` (ActorSystem.scala:486-487).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-UTIL-004.1.

**Equivalente Rust propuesto y diferencias observables.** Tipo separado (implementado); subtipado perdido (Q-X-9).

**Aserciones.**
- A1 mensaje de versión: derivada del código ActorSystem.scala:485-487 (*no afirmada por spec*).
- A2 pertenencia a la jerarquía PekkoException: oráculo EX-SUB.

**Oráculo.** Sí: jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/util-core.jsh, golden crates/rakka-actor/tests/oracle/util-core.golden); NO es Pekko compilado de upstream/pekko.

**Decisiones.** Q-X-9 (Q-UTIL-4).


---

## AC-UTIL-004.3 — OnlyCauseStackTrace (trait que suprime el stack trace propio y conserva el de la causa)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Marca que ciertos errores muestren sólo la traza de la causa (p. ej. `ActorInitializationException`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/PekkoException.scala:24-36`

**Condiciones.**
- Con causa: stack trace = el de la causa. Sin causa: arreglo vacío.
- Errores: ninguno.

**Orden / concurrencia / ciclo de vida.** `fillInStackTrace` corre al construir la excepción (JVM).

**Dependencias.** Consumidores: excepciones de CELL/SUP (filas ajenas).

**Equivalente Rust propuesto y diferencias observables.** No hay equivalente directo; se propone `Error::source()` encadenado y backtrace opcional; diferencia observable: no hay traza (Q-X-9).

**Aserciones.**
- A1 traza vacía sin causa / igual a la causa con causa: derivada del código PekkoException.scala:29-35 (*no afirmada por spec*).

**Oráculo.** No aplica: depende del runtime JVM (stack traces).

**Decisiones.** Q-X-9 (Q-UTIL-4).


---

## AC-UTIL-005.1 — PekkoVersion.require y UnsupportedPekkoVersion

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Los módulos externos llaman `PekkoVersion.require("mi-lib", "1.0.4")` para abortar si el Pekko cargado es más viejo/incompatible.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/PekkoVersion.scala:19-83`
- `actor/src/main/scala/org/apache/pekko/util/Version.scala:19-46 (Version.current vía Version.scala generado)`

**Condiciones.**
- Normal: igual -> ok (línea 41). Misma mayor y menor: parche actual >= requerido; menor actual > requerido -> ok; mayor distinta -> falla (línea 73).
- Pre-release (patrón `(\d+)\.(\d+)\.(\d+)(?:-(M|RC)(\d+))?`): requerido final y actual M/RC -> falla; M(req) vs RC(act) ok; RC(req) vs M(act) falla; mismo tipo compara el número; requerido M/RC con actual final -> ok.
- Error: `UnsupportedPekkoVersion("Current version of Pekko is [<cur>], but <lib> requires version [<req>]")` (líneas 52-54), clase `final`, extiende RuntimeException, constructor `private[pekko]`.
- Versiones que no calzan con el patrón (SNAPSHOT, timestamped, 'nonsense') -> no falla (líneas 74,77).

**Orden / concurrencia / ciclo de vida.** n/a: pura.

**Dependencias.** Ninguna fila; dato externo: `Version.current` generado en build (project/VersionGenerator.scala:18-25).

**Equivalente Rust propuesto y diferencias observables.** Función `require` + tipo de error; `Version.current` = constante de rakka (Q-CFG-5).

**Aserciones.**
- A1 ok: (2.5.6,2.5.6), (2.5.6,2.5.7), (2.5.6,2.6.0): PekkoVersionSpec.scala:23-27.
- A2 RC ok: (2.5.6, 2.5.7-RC10), (2.6.0-RC1, 2.6.0-RC1/RC2/2.7.0): PekkoVersionSpec.scala:29-34.
- A3 RC falla: (2.5.6, 2.5.6-RC1) y (2.5.6-RC2, 2.5.6-RC1): PekkoVersionSpec.scala:36-43.
- A4 milestone ok: PekkoVersionSpec.scala:45-51; falla: (2.5.6,2.5.6-M1), (2.5.7-M10,2.5.7-M9), (2.5.7-RC1,2.5.7-M9): PekkoVersionSpec.scala:53-63.
- A5 mayor distinta falla (3.0.0, 1.0.0): :65-73; menor baja (2.4.19): :75-79; parche bajo (2.5.5): :81-85.
- A6 SNAPSHOT actual/requerido y timestamped pasan: :87-101; versión incomprensible pasa: :103-106.
- A7 el texto exacto del mensaje no lo afirma ningún test: derivada del código PekkoVersion.scala:52-54.

**Oráculo.** Posible: jar pekko-actor 1.7.1 (`PekkoVersion.require` es público); no hay escenario aún.

**Decisiones.** Q-CFG-5.

**Actualización P66 (2026-11-18, Pekko 2.0.0).** `Version.current` en 2.0.0 es "2.0.0" (Q-CFG-5; `version.conf` generado por `project/VersionGenerator.scala:20-40`, PV-01), no 1.7.1. El oráculo 2.0.0 confirma A1..A7 (PV-02..PV-06, PV-12, PV-13) y añade: comparación primero como texto y luego como Int (`2.5.6` contra `2.05.5` pasa y no mira el parche), `NumberFormatException: For input string: "<dígitos>"` para un número fuera de Int cuando hay que convertirlo (PV-07, PV-08, PV-11), patrón de dígitos ASCII con coincidencia total (PV-09, PV-10). En Rust: `rakka_actor::pekko_version::{CURRENT, require, require_against, UnsupportedPekkoVersion, RequireError}`.


---

## AC-UTIL-005.2 — util.Version (versión comparable con M/RC/dynver)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `Version(String)` convierte una versión en 4 números + resto y es `Comparable`; se usa para chequear versiones de módulos (ManifestInfo) y por Typed/Cluster.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Version.scala:19-202`

**Condiciones.**
- Parseo: 1 segmento -> número simple (o alfanumérico si no es numérico) con los otros 3 números = Undefined(0); 2 segmentos -> `a.b[-rest|+dynver]`; 3 segmentos `a.b.c[...]`; más de 3 -> IllegalArgumentException("Only 3 digits separated with '.' are supported. [<v>]") (línea 146); vacío -> IllegalArgumentException("Empty version not supported.") (línea 114).
- Segmento no numérico donde se espera número -> NumberFormatException (via toInt; VersionSpec.scala:67-74).
- Comparación: si `version` string es igual -> 0 sin parsear (línea 156); si no compara numbers(0..3) restando; luego `rest` (líneas 170-175, ver hallazgo); `equals` = compareTo==0 (1.2 == 1.2.0); hashCode con HashCode.SEED sobre números y rest.
- dynver: `+N-hash` o `-N-hash`: N se interpreta como número de commits (3 < 10); `HEAD+20140707-1030` compara por rest (VersionSpec.scala:77-92).

**Orden / concurrencia / ciclo de vida.** `parse()` es perezoso (campo `numbers` volatile) y se invoca desde compareTo/hashCode/apply.

**Dependencias.** AC-UTIL-016.15 (HashCode).

**Equivalente Rust propuesto y diferencias observables.** Struct inmutable con parseo en la construcción; `pekko_compare` explícito (no `Ord` si no es orden total, como AC-PATH-002).

**Aserciones.**
- A1 3 dígitos: `1.2.3 == 1.2.3`, `!= 1.2.4`, `1.2.4 > 1.2.3`, `3.2.1 > 1.2.3`, `3.2.1 < 3.3.1`, `3.2.0 < 3.2.1`: VersionSpec.scala:23-30.
- A2 más de 3 dígitos -> IllegalArgumentException: :32-34.
- A3 2 dígitos: `1.2 == 1.2.0`, `1.2 != 1.3`, `1.2.1 > 1.2`, `2.4 > 2.3`, `3.2 < 3.2.7`: :36-43.
- A4 1 dígito: `1 == 1.0 == 1.0.0`, `3 > 2`, `2b > 2a`, `2020-09-07 > 2020-08-30`: :45-54.
- A5 extra: `1.2.3-M1 == 1.2.3-M1`, `1.2.0-M1 == 1.2-M1`, `1.2-M1 < 1.2.0`, `1.2.3-M2 > 1.2.3-M1`: :56-64.
- A6 no dígitos -> NumberFormatException para `1.x.3`, `1.2x.3`, `1.2.x`, `1.2.3x`, `x.3`, `1.x`, `1.2x`: :66-75.
- A7 dynver: :77-92. Sin dígitos: `foo == foo`, `foo != bar`, `foo < 1.2.3`, `foo > bar`, `1-foo != 01-foo`, `1-foo > 02-foo`: :94-102.
- A8 antisimetría de compareTo con rest vacío en un lado: NO afirmada por spec; **medido con el jar 1.7.1 (JDK 11)**: `Version("1.2.0").compareTo(Version("1.2-M1")) = -2` **y** `Version("1.2-M1").compareTo(Version("1.2.0")) = -1` (ambos negativos: el `if (rest == "" && other.rest != "") diff = 1` de Version.scala:170-171 queda sobrescrito por el `else` de la línea 172-175 en la comparación `"".compareTo(rest)`); además `1.2.0` vs `1.2.0-RC1` = -3 y `1.2.0-RC1` vs `1.2.0` = -1, `1.2-M1` vs `1.2-M2` = -1. VersionSpec.scala:56-64 sólo prueba `1.2-M1 < 1.2.0` (compatible con el defecto). Confirma Q-UTIL-6.

**Actualización P67 (2026-11-18, Pekko 2.0.0).** El defecto de A8 (medido con 1.7.1) está CORREGIDO en 2.0.0 (`Version.scala:169-175`: `if (rest == "" && other.rest != "") 1 else if (other.rest == "" && rest != "") -1 else rest.compareTo(other.rest)`); el Owner rechazó la alternativa bug-compatible (Q-UTIL-6) y pidió medirlo con el jar 2.0.0: `Version("1.2.0").compareTo(Version("1.2-M1")) = 1`, `Version("1.2-M1").compareTo(Version("1.2.0")) = -1`, 0 pares no antisimétricos sobre 19 versiones (VR-05, VR-10, VR-11), y `hashCode` coincide con `equals` (VR-14). La medición coincide con la descripción del Owner y con la lectura de la fuente. Además fija: `split('.')` descarta vacíos finales (`1.` y `1.2.3.` válidas; `.`, `..` -> "Only 3 digits"; `.1`, `1..2` -> `NumberFormatException: For input string: ""`), diferencias de `Int` con wrap (`2147483647.0.0` frente a `-1.0.0` da `-2147483648`), calificador vacío (`1.2.3-`) igual a sin calificador, dígitos Nd aceptados (`٢.٥.٥` == `2.5.5`), resto comparado con `String.compareTo` bruto (`M9` frente a `M10` = 8), y que `new Version(s)` parsea de forma perezosa (VR-21, sin equivalente en Rust). En Rust: `rakka_actor::util::{Version, VersionError}`.

**Oráculo.** Casos VR-01..VR-23 (`VersionOracle.java`, uc-0148..uc-0169); VR-21 queda como PENDING (fachada Java).

**Decisiones.** Q-UTIL-6 (rechazada: 2.0.0 corregido), Q-X-9.


---

## AC-UTIL-006.1 — ByteString inmutable: estructura (ByteString1C, ByteString1, ByteStrings), operaciones de secuencia, igualdad y toString

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ByteString` es una secuencia inmutable de bytes (`IndexedSeq[Byte]`, sellada) con tres representaciones internas: `ByteString1C` (un array completo, compacta), `ByteString1` (un array con `startIndex`/`length`) y `ByteStrings` (rope de 2 o más `ByteString1` no vacíos). Lo observable no es la representación sino: (a) los valores de `length`, `apply`, `head/tail/last/init`, `take/drop/dropRight/takeRight/slice/splitAt/span/takeWhile/dropWhile/grouped/map/foreach/foldLeft` son los de un `Vector[Byte]`; (b) los argumentos fuera de rango en `take/drop/dropRight/slice` se saturan y nunca fallan; (c) igualdad por contenido entre las tres representaciones y `hashCode` consistente con ella; (d) `++` y los recortes no copian el contenido; (e) `toString` y `ByteString.empty`. Las factorías y conversiones están en AC-UTIL-006.6, la búsqueda en AC-UTIL-006.7, la serialización en AC-UTIL-006.8, la compactación en AC-UTIL-006.2.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:168-307 (1.7.1)` (ByteString1C), `:312-507` (ByteString1), `:509-826` (ByteStrings y su compañero, `compare`), `:867-1162` (clase base: `head/tail/last/init`, `takeRight`, `span`, `splitAt`, `grouped`, `toString`, `map/mapI`)
- `actor/src/main/scala-3/org/apache/pekko/util/ByteString.scala` (espejo para Scala 3; el módulo se compila contra 2.13 y 3)
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/ByteStringSpec.scala:372-441` (ByteString1), `:430-493` (ByteString1C), `:494-645` (ByteStrings drop/dropRight/slice/take), `:1112-1372` (propiedades de ByteString), `ByteStringInitializationSpec.scala:25-64`

**Precondiciones.** Ninguna para las operaciones de recorte (todo `Int` es aceptado, incluidos `Int.MinValue` e `Int.MaxValue`). `ByteStrings` exige `bytestrings` no vacío y primer fragmento no vacío (`ByteString.scala:576-577`), pero sólo se construye por `++`/builder, que ya garantizan la invariante.

**Comportamiento.**
- `take(n)`: `n <= 0` -> vacío; `n >= length` -> el mismo objeto (`ByteString.scala:223-226,374-379,653-656`); `drop(n)`: `n <= 0` -> el mismo objeto; `n >= length` -> vacío; `dropRight(n)` análogo; `slice(from, until)` con `from <= 0 && until >= length` -> el mismo objeto, `from >= length || until <= 0 || from >= until` -> vacío (`:273-276,381-382,699-702`).
- `++`: si un lado es vacío devuelve el otro (mismo objeto); `ByteString1 ++ ByteString1` sobre el mismo array contiguo (`startIndex + length == b.startIndex`) produce un único `ByteString1` sin copiar (`:437-449`); en otro caso, un `ByteStrings` (concatenación de vectores de fragmentos, longitud = suma).
- `apply(idx)`: `ByteString1` y `ByteStrings` lanzan `IndexOutOfBoundsException(idx.toString)` fuera de `[0, length)`; `ByteString1C` indexa el array y lanza `ArrayIndexOutOfBoundsException` (subclase) con el texto de la JVM (medido: `apply(3)` sobre `"abc"` compacto -> `Index 3 out of bounds for length 3`; sobre un rope -> mensaje `"3"`).
- `head`/`last` sobre vacío: `apply(0)`/`apply(length-1)` y por tanto la excepción de `apply` (medido: `ArrayIndexOutOfBoundsException` sobre `ByteString.empty`); `tail`/`init` = `drop(1)`/`dropRight(1)` (vacío -> vacío).
- `grouped(size)`: `size <= 0` -> `IllegalArgumentException("size=<n> must be positive")` (`:994-1000`, medido para 0); en otro caso fragmentos de `take(size)` hasta agotar.
- `toString`: `"ByteString(b1, b2, ...)"` con los bytes con signo en decimal; vacío -> `"ByteString()"`; si `size > 100` -> `take(100).toString + "... and [<size-100>] more"` (`:1002-1008`; medido: 101 bytes -> `ByteString(0, 0, ..., 0)... and [1] more`).
- Igualdad y `hashCode`: los de `Seq[Byte]` de Scala (cualquier `ByteString` es igual a otra con los mismos bytes con independencia de la representación; no es igual a un `java.util.List` aunque tenga los mismos bytes: medido `false`). Valores medidos con el jar: `hashCode` de `ByteString("abc")` = -1431565476 y el de `ByteString.empty` = 473519988; el `hashCode` del rope `"a" ++ "bc"` es el mismo que el compacto.
- `ByteString.empty` es un singleton (`ByteString1C.empty`); `take(length)` devuelve el mismo objeto (`eq`).

**Errores.** `IndexOutOfBoundsException` en `apply`; `IllegalArgumentException` en `grouped(<=0)` y en la construcción inválida de `ByteStrings`; las operaciones `iterator`/`take`/`slice`/`drop`/`dropRight`/`copyToArray` de la base abstracta lanzan `UnsupportedOperationException` si una subclase no las sobreescribe (inalcanzable: la jerarquía está sellada).

**Orden / concurrencia / ciclo de vida.** Valor inmutable y seguro entre hilos (el contrato lo declara: "thread safe way of working with bytes", `:861-863`). No hay estado mutable salvo la caché interna de ninguna clase. `ByteStringInitializationSpec` fija que inicializar `CompactByteString` antes que `ByteString` no falla (problema de inicialización de clases de la JVM, sin equivalente en Rust).

**Dependencias.** AC-UTIL-006.2 (compactación), AC-UTIL-006.4 (iterador devuelto por `iterator`), AC-UTIL-016.4/AC-UTIL-016.15 no. Consumidores: AC-IO-*, AC-SER-*.

**Equivalente Rust propuesto y diferencias observables.** `pub struct ByteString` con la misma semántica de valor. Viable con: (a) newtype sobre `bytes::Bytes` (recorte y `clone` baratos, un solo fragmento; `++` copia o usa una cadena de `Bytes`); o (b) rope propio sin dependencias (`Vec<Bytes>`-like). Preserva (a)-(e). Diferencias: las saturaciones se expresan con `usize`/`isize` (los argumentos negativos de Pekko se aceptan sólo con firmas `i32`/`i64`; propuesta: aceptar `i64` en `take/drop/slice` y saturar para no perder el contrato); `apply` fuera de rango será `Option`/panic con mensaje propio (el texto de la JVM no se reproduce); la identidad por referencia (`eq`) de `take(length)` y la fusión de fragmentos contiguos no son observables en Rust salvo con `Bytes::as_ptr`; `hashCode` de Scala no se reproduce salvo que se necesite (depende de Q-UTIL-14). Decisión de representación: Q-UTIL-12.

**Aserciones.**
- A1 `drop`/`dropRight`/`take` sobre `ByteString1` y `ByteString1C` con negativos, 0, 1, mayor que la longitud y encadenados: ByteStringSpec.scala:361-407 (ByteString1), :431-471 (ByteString1C); `ByteString1.empty.drop(-1) == ByteString("")`, `fromString("abc").drop(1) == "bc"`, `fromString("0123456789").drop(5).take(4).drop(1).take(2) == "67"`.
- A2 `ByteStrings.drop/dropRight` con `Int.MinValue`, `-1`, `0`, `n > length`, `Int.MaxValue`, fragmentos vacíos y `ByteString("0123456789").drop(5).take(2) == "56"`: ByteStringSpec.scala:495-536, :537-582.
- A3 `slice` con 6 patrones de borde (`(-10,10)`, `(-10,0)`, `(1,-2)`, `(1,0)`, `(-100,-10)`...) sobre `ByteStrings` y `ByteString1`, "get an empty if from is greater than until": ByteStringSpec.scala:583-631.
- A4 `take` sobre ByteStrings (negativos, `take(100)` = todo): :632-645.
- A5 tamaños y secuencialidad (propiedades de 1000 casos): `(a++b).size == a.size+b.size`, `(a++b).take(a.size) == a`, `(a++b).drop(a.size) == b`, `takeRight`, `(b++a++b).drop(b.size).take(a.size) == a`: :1113-1127.
- A6 reconstrucción: `(a ++ b ++ c) == xs` tras `splitAt` con índices extremos (`-2147483648`, `2147483647`, `Int.MaxValue`) incluidos casos borde de `ByteString1.drop1`: :1129-1163.
- A7 `b.take(b.length) eq b`: :1172-1176; `toString.startsWith("ByteString(")`: :1223-1227; `ByteString(iterator) == a`: :1186-1191.
- A8 comportarse como `Vector` en concatenación, `apply`, `head`, `tail`, `last`, `init`, `length`, `span`, `takeWhile`, `dropWhile`, `indexWhere`, `foreach`, `foldLeft`, `toArray`, `slice`, `take/drop`, `grouped`, `copyToArray`: :1229-1372 (propiedad `likeVector`).
- A9 *no afirmada por spec*: valores de `hashCode`, `toString` de más de 100 bytes, texto de las excepciones, `grouped(0)`; caracterización propuesta con jar (valores arriba) y golden `bytestring.golden` por crear.

**Oráculo / validación prevista.** Test portado 1:1 (A1-A8; las propiedades `check` se portan a `proptest` con las mismas generaciones: fragmentos 1..N, `from/until` en rango) más un diferencial con el jar oficial (medido ad hoc con `ORACLE=jars tools/oracle/run.sh`, JDK 11; falta persistir `tools/oracle/bytestring.jsh` y su golden) para A9.

**Decisiones.** Q-UTIL-12 (representación; bloquea la implementación, no la extracción del contrato), Q-UTIL-14 (reproducir el hash de Scala), Q-UTIL-8 no aplica.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** En 2.0.0 `ByteString.scala` es un único archivo `actor/src/main/scala/org/apache/pekko/util/ByteString.scala` (ya no hay `scala-2.13`/`scala-3`) y añade una cuarta representación, `ByteString2` (dos fragmentos simples, identidad de serialización 3), que no se observa. Medido con el jar 2.0.0 en JDK 17: `takeRight(Int.MinValue)` devuelve todo (`length - n` desborda), `grouped(0)` falla con `size=0 must be positive`, `apply` fuera de rango sobre ByteString1/ByteStrings lanza `IndexOutOfBoundsException(<idx>)` y sobre un valor compacto `ArrayIndexOutOfBoundsException` con el texto de la JVM (BS-APPLY-1C, Q-UTIL-23 propuesta). `hashCode` no se reproduce (Q-UTIL-14 (b)). Rust: `rakka_actor::util::byte_string::ByteString` (Q-UTIL-12 (c)); casos uc-0454..uc-0467 y BS-R* (uc-0487..uc-0494); fila `partial` por Q-UTIL-23.

---

## AC-UTIL-006.2 — CompactByteString (compactación a ByteString1C) y factorías compactas

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `CompactByteString` es un `ByteString` contiguo en memoria y sin holgura (`isCompact == true`, `compact` devuelve `this`). `ByteString.compact` devuelve un `CompactByteString` con el mismo contenido: O(1) si `isCompact`, copia en caso contrario. `isCompact` es `true` sólo si el valor es un `ByteString1C`, un `ByteString1` cuyo array tiene exactamente su longitud, o un `ByteStrings` con un único fragmento compacto. Las factorías `CompactByteString.apply/fromArray` copian siempre y devuelven `CompactByteString.empty` (singleton) para entrada vacía.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:1164-1257 (1.7.1)` (compañero y clase), `:351,396-397` (ByteString1.isCompact/compact), `:610,620-631` (ByteStrings.isCompact/compact)
- tests: `ByteStringSpec.scala:1129-1134,1193-1202`, `ByteStringInitializationSpec.scala:25-64`

**Precondiciones.** Ninguna.

**Comportamiento.**
- `compact` de un `ByteStrings` con varios fragmentos aloja un array de `length` bytes y copia cada fragmento (`:620-631`); la mutación posterior del array de origen no afecta (los arrays de las factorías públicas se clonan).
- `CompactByteString(bytes: Array[Byte])` clona; vacío -> `empty`. `CompactByteString(bytes: Byte*)`, `(IterableOnce[Byte])`, `(Integral*)` (convierte con `toInt.toByte`, truncando), `(ByteBuffer)` (consume `remaining` bytes y avanza la posición), `(String)` UTF-8, `(String, charset)` (nombre o `Charset`), `fromArray(array, offset, length)`: `offset` negativo se lleva a 0 y `length` se acota a `array.length - offset` y a >= 0; si queda 0 -> vacío (`:1234-1243`; medido: `fromArray([1,2,3,4], 2, 10)` -> 2 bytes; `fromArray(.., -2, 3)` -> 3 bytes).
- Propiedad `compact`: `b = a.compact; (!a.isCompact || b eq a) && b == a && b.isCompact && (b.compact eq b)`.

**Errores.** Charset desconocido -> `UnsupportedEncodingException` (medido `decodeString("nope")`, aplica igual a `apply(string, "nope")`); `ByteBuffer` nulo -> NPE.

**Orden / concurrencia / ciclo de vida.** Inmutable. La construcción de los singletons `empty` es parte de la inicialización de clases (`ByteStringInitializationSpec`); en Rust equivale a una `const`/`static`.

**Dependencias.** AC-UTIL-006.1, AC-UTIL-006.6 (factorías de `ByteString` que delegan en las compactas).

**Equivalente Rust propuesto y diferencias observables.** `ByteString::compact(&self) -> ByteString` y `is_compact(&self) -> bool`. Con `bytes::Bytes` (un solo fragmento) todo valor es compacto salvo los recortes que retienen el búfer original (`isCompact` de `ByteString1` compara `length == bytes.length`); para conservar el contrato observable `is_compact` debe ser `slice_len == backing_len` o `fragments == 1 && ...`; si el equipo acepta que `is_compact` pase a ser `true` también para recortes, es una diferencia a registrar (Q-UTIL-12). `CompactByteString` como tipo aparte sólo hace falta si la API pública lo exige; propuesta: alias de tipo o nuevo tipo con `Deref<Target=ByteString>`.

**Aserciones.**
- A1 `compact` conserva contenido y devuelve compacto: `xs.compact == xs && ys.isCompact` (ByteStringSpec.scala:1130-1134); `(!wasCompact || (b eq a)) && b == a && b.isCompact && (b.compact eq b)` (:1193-1202).
- A2 `CompactByteString.empty ne null` y `ByteString.empty ne null` con carga de clases en ese orden (ByteStringInitializationSpec.scala:56-64, clase `ByteStringInitTest`); no aplicable a Rust.
- A3 *no afirmada por spec*: saturación de `offset/length` en `CompactByteString.fromArray`, conversión truncante de `Integral`, consumo de la posición del `ByteBuffer`; caracterización propuesta con el jar (valores medidos arriba).

**Oráculo / validación prevista.** Test portado de A1 y diferencial con jar para A3 (ver AC-UTIL-006.1).

**Decisiones.** Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** Q-UTIL-12 (c): `is_compact` es siempre `true` y `compact` copia a un tamaño exacto; `CompactByteString` no tiene tipo aparte. Casos uc-0468..uc-0470 y uc-0478. A2 no aplica.

---

## AC-UTIL-006.3 — ByteStringBuilder (append de primitivos con byte order, putBytes, result)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ByteStringBuilder` es un builder mutable no sincronizado que acumula bytes y produce un `ByteString` con `result()`. Escribe primitivos con el orden de bytes indicado (`BIG_ENDIAN`/`LITTLE_ENDIAN`), arreglos de primitivos y rangos de bytes, y se adapta a `java.io.OutputStream`. El resultado no se compacta automáticamente.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:1259-1595 (1.7.1)` (builder completo)
- tests: `ByteStringSpec.scala:301-358` (helpers `test*Encoding`), `:1681-1801`; `ByteStringBuilderSpec.scala:22-38`; `actor-tests/src/test/scala-2.13+/org/apache/pekko/util/ByteStringBuilderScala213PlusSpec.scala:26-40`, `actor-tests/src/test/java/org/apache/pekko/util/ByteStringTest.java:20-36`

**Precondiciones.** `byteOrder` debe ser uno de los dos valores de `java.nio.ByteOrder`; `putBytes(array, start, len)` exige un rango válido del arreglo.

**Comportamiento.**
- `putByte`/`+=`/`addOne`, `putShort(x: Int)`, `putInt`, `putLong`, `putLongPart(x, n)` (los `n` bytes menos significativos de `x`, en el orden pedido), `putFloat`/`putDouble` (bits crudos `floatToRawIntBits`/`doubleToRawLongBits`), `putBytes`, `putShorts/Ints/Longs/Floats/Doubles(array[, start, len])`. Medido: `putInt(0x01020304, BIG)` -> `(1,2,3,4)`, `LITTLE` -> `(4,3,2,1)`; `putLongPart(0x0102030405, 3, BIG)` -> `(3,4,5)`, `LITTLE` -> `(5,4,3)`; `putShort(0x0102, LITTLE)` -> `(2,1)`; `putFloat(1.0f, BIG)` -> `(63,-128,0,0)`.
- `++=`/`addAll`/`append(bs)`: un `ByteString` vacío no cambia nada; un `ByteString` se agrega por fragmentos sin copiar contenido; `Array`/`IndexedSeq`/`IterableOnce` se copian. `append` de vacío devuelve el builder.
- `length`/`isEmpty`/`nonEmpty` reflejan los bytes acumulados; `clear()` vacía; `result()` con 0 bytes -> `ByteString.empty`, con un fragmento -> ese `ByteString1`, con varios -> `ByteStrings`. `result()` consolida el temporal y no invalida el builder: se puede seguir escribiendo y llamar de nuevo (medido: `result()` tras 1 byte -> longitud 1; tras escribir otro byte, el primer resultado sigue con longitud 1 y el nuevo tiene 2).
- `sizeHint(len)` ajusta la capacidad del temporal; el temporal crece duplicando desde 16.
- `asOutputStream`: `write(b: Int)` agrega `b.toByte` (medido: `write(300)` -> 44), `write(b, off, len)` -> `putBytes`.

**Errores.** `IllegalArgumentException("Unknown byte order " + byteOrder)` para un orden distinto de los dos estándar (medido con `null`: `Unknown byte order null`); `putBytes`/`write(b, off, len)` con rango inválido -> `ArrayIndexOutOfBoundsException` de `System.arraycopy` (medido: `last source index 7 out of bounds for byte[3]`) después de haber reservado espacio (el builder queda con `length` incrementado: efecto lateral no afirmado).

**Orden / concurrencia / ciclo de vida.** No thread-safe. Los bytes de un mismo `put*` son contiguos y en el orden pedido; no hay flush.

**Dependencias.** AC-UTIL-006.1 (resultado), AC-UTIL-006.4 (lectura simétrica).

**Equivalente Rust propuesto y diferencias observables.** `ByteStringBuilder` sobre `Vec<u8>` + lista de fragmentos (o `BytesMut`), con `put_short_be/le`, `put_int_be/le`... o un parámetro `ByteOrder` propio; `impl std::io::Write` para el adaptador. Preserva el resultado de cada `put*`. Diferencias: el error de orden desconocido desaparece si `ByteOrder` es un `enum` de dos variantes (diferencia de tipos, no de comportamiento); `putLongPart` con `n` fuera de `0..=8` hay que definirlo (Pekko no lo valida: `n > 8` desplaza `x >>> 8*i` con desbordes de Java, `n < 0` no escribe) -> *no afirmada*, se propone `assert!(n <= 8)`; los valores de rango inválido serán `panic`/`Result`.

**Aserciones.**
- A1 `++=` (y `addAll`, en Scala 2.13+) de `List[Byte]`, `Vector[Byte]` y `ByteString` produce `ByteString("a")`: ByteStringBuilderSpec.scala:24-37; ByteStringBuilderScala213PlusSpec.scala:28-39.
- A1b API Java: `createBuilder` + tres `append` + `result()` == `ByteString.fromString("Hello World")`; `fromString("")` y `fromInts(1,2,3)` no fallan: ByteStringTest.java:24-36.
- A2 mezcla de `+=`/`++=` de arrays, `ByteString`s y `IndexedSeq` equivale a un `VectorBuilder` (propiedad): ByteStringSpec.scala:1683-1697.
- A3 `putByte`/`putBytes(data, from, to-from)` equivalen a la secuencia original: :1700-1710; con `asOutputStream.write` mezclando `write(Int)` y `write(array, off, len)`: :1712-1722.
- A4 codificación de Short, Int, Long, LongPart, Float y Double en BIG y LITTLE endian frente a una referencia hecha con `ByteBuffer`: :1726-1785 (helpers :301-358, incluida la variante arreglo `putShorts/putInts/...` con rango).
- A5 `isEmpty` de un builder nuevo y `nonEmpty` tras `putByte(1)`: :1788-1799.
- A6 *no afirmada por spec*: errores de orden desconocido y de rango, reutilización tras `result()`, `write(300) == 44`; caracterización con jar.

**Oráculo / validación prevista.** Test portado A1-A5 con `proptest`; A6 con diferencial (medido ad hoc).

**Decisiones.** Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** Medido: `putLongPart(x, n, order)` con n > 8 desplaza módulo 64 (n=9 BE de 0x0102030405060708 escribe `08 01..08`); con n < 0 el builder queda corrupto (`result()` falla con `AssertionError: empty ByteStrings length must be 0, was -1`), que Rust no reproduce (escribe 0 bytes). Errores de `putBytes` fuera de rango: texto del arraycopy de la JVM, no reproducido. Rust: `ByteStringBuilder`; casos uc-0495..uc-0505.

---

## AC-UTIL-006.4 — ByteIterator (lectura de primitivos con byte order, getBytes, take/drop)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ByteIterator` es un `BufferedIterator[Byte]` mutable sobre un `ByteString` (implementado por `ByteArrayIterator` y `MultiByteArrayIterator`). Además de la iteración, lee primitivos con orden de bytes (`getShort/Int/Long/LongPart/Float/Double`), copia a arreglos (`getBytes/getShorts/getInts/getLongs/getFloats/getDoubles`), a `ByteBuffer` (`copyToBuffer`), expone `len`, `clone`, `duplicate`, `toByteString`, `take/drop/slice/takeWhile/dropWhile/span/indexWhere/indexOf` y se adapta a `java.io.InputStream`. Las operaciones de recorte mutan el iterador y devuelven `this`.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteIterator.scala:28-219` (ByteArrayIterator), `:197-426` (MultiByteArrayIterator), `:428-712` (clase base, `getXxx`)
- tests: `ByteStringSpec.scala:1415-1679` (iterador), `ByteIteratorSpec.scala:22-47`

**Precondiciones.** `byteOrder` estándar; los arreglos destino con espacio para `n` elementos desde `offset`.

**Comportamiento.**
- `next()` sobre iterador agotado lanza `NoSuchElementException("next on empty iterator")` (medido). `head` no avanza. `len` es el número de bytes restantes; `size` **consume** el iterador (devuelve `len` y lo vacía: medido `size == 3` y luego `hasNext == false`).
- `take(n)`: `n <= 0` vacía; `n >= len` no cambia. `drop(n)`: `n <= 0` no cambia; `n >= len` vacía. `slice(from, until)` = `drop(from).take(until - from)` (con saturaciones). `takeWhile`/`dropWhile` mutan.
- `clone` es independiente (medido: tras `it.next()` el original tiene 2 y el clon 3); `duplicate = (this, clone)`.
- `getByte`, `getShort/Int/Long(order)` componen big/little endian con `next()` (un iterador corto lanza `NoSuchElementException` a mitad de la lectura, tras haber consumido los bytes disponibles: medido `getInt` sobre 2 bytes). `getLongPart(n)`: n bytes -> `Long` (medido `getLongPart(3, BIG)` sobre `(1,2,3)` = 66051). `getFloat/getDouble` por `intBitsToFloat/longBitsToDouble`. Orden desconocido -> `IllegalArgumentException("Unknown byte order <o>")`.
- `getBytes(xs, offset, n)`: si `n > len` -> `NoSuchElementException("next on empty iterator")` (medido) **sin copiar**; devuelve `this`. `getBytes(n)` crea el arreglo. **`getByteString(n)` con `n > len` no falla**: devuelve lo que queda (medido: `getByteString(5)` sobre `"abc"` -> `ByteString(97,98,99)`) y vacía el iterador.
- `copyToArray(xs, start, len)`: copia `max(0, min(xs.length - start, this.len, len))` bytes y avanza ese número; nunca entra en bucle con `len` más allá del destino (regresión fijada por test).
- `toByteString`: `ByteString1C` si abarca el array completo, `ByteString1` en otro caso; consume el iterador (queda vacío).
- `++` (otro `ByteIterator`): fusiona sin copiar si es el mismo array contiguo; si no, `MultiByteArrayIterator`.
- `asInputStream`: `read()` -> byte sin signo o `-1`; `read(b, off, len)` -> `IndexOutOfBounds` si `off < 0 || len < 0 || off + len > b.length`, `0` si `len == 0`, `-1` si agotado, si no `min(available, len)` bytes; `skip(n)` -> `min(len, n.toInt)` (medido: `skip(2)` -> 2, luego `read` -> 99 y `-1`).

**Errores.** `NoSuchElementException("next on empty iterator")`, `IllegalArgumentException` (orden), `IndexOutOfBoundsException` (mensaje `null` medido) en `asInputStream.read`.

**Orden / concurrencia / ciclo de vida.** Mutable, no thread-safe; avanza de forma estrictamente secuencial; el `ByteString` original no cambia.

**Dependencias.** AC-UTIL-006.1; AC-UTIL-016.1 no (el iterador no usa `OptionVal`); `Collections.EmptyImmutableSeq` (AC-UTIL-016.3) genera el `NoSuchElementException` de `next`.

**Equivalente Rust propuesto y diferencias observables.** `struct ByteIterator` (cursor sobre uno o varios fragmentos) con `get_u16/u32/u64/f32/f64` en ambos órdenes (el crate `bytes::Buf` ya lo hace) e `impl Iterator<Item=u8>` y `impl std::io::Read`. Diferencias: `Buf::get_*` hace `panic` si faltan bytes y no consume parcialmente; el contrato de Pekko consume lo disponible y luego falla: se propone `Result<_, NoSuchElement>` y un test que fije el consumo parcial (o declarar la diferencia: Q-UTIL-12). `getByteString(n>len)` devuelve el resto (a preservar).

**Aserciones.**
- A1 se comporta como `BufferedIterator` de `Vector`: `++`, `head`, `next`, `hasNext`, `length`, `duplicate`, `span`, `takeWhile`, `dropWhile`, `indexWhere`, `indexOf`, `toSeq`, `foreach`, `foldLeft`, `toArray`, `slice`, `take/drop`, `copyToArray`: ByteStringSpec.scala:1416-1537.
- A2 `getByte`+`getBytes` mezclados reconstruyen la secuencia y dejan el iterador vacío: :1540-1551; `getBytes(n)`: :1553-1559; `getByteString(n)`: :1561-1567.
- A3 `asInputStream` con `skip`, `read(buf, off, len)` y `read()`, y `-1` al final: :1569-1595.
- A4 `copyToBuffer` equivale a copiar y leer: :1597-1607; copia por trozos "123"+"456": :1609-1616; `copyToArray` con `len` pasando el final del destino (bucle infinito previo): :1618-1624 (`Seq(0,0,1)`).
- A5 decodificación de Short/Int/Long/Float/Double en BIG y LITTLE endian: :1628-1677 (helpers `test*Decoding`).
- A6 *no afirmada por spec*: `getByteString(n > len)`, `size` que consume, consumo parcial en lecturas cortas, texto de excepciones; caracterización con jar.
- A7 `indexOf(elem)` y `indexOf(elem, from)` sobre un `ByteArrayIterator` de `[0x20,0x20,0x10,0x20,0x20,0x10]` devuelven 0, 2, 1, 2, 5 (la sobrecarga `indexOf(Byte, Int)` incluida), con un iterador nuevo por prueba porque `indexOf` **invalida** (consume) el iterador: ByteIteratorSpec.scala:24-45.

**Oráculo / validación prevista.** Test portado A1-A5; diferencial A6.

**Decisiones.** Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** El código está ahora en `scala-2.13/org/apache/pekko/util/ByteIterator.scala`: `getShort/getInt/getLong` del iterador de un array comprueban `len` y lanzan `NoSuchElementException("next on empty iterator")` SIN consumir (el multi-fragmento consume lo disponible; el contrato de 1.7.1 decía que siempre consumía), `getLongPart(n, LITTLE_ENDIAN)` desplaza un Int (módulo 32, extensión de signo), `indexOf` agota el iterador, `asInputStream.skip(-2)` devuelve -2, y `++` entre iteradores de arrays distintos pierde el lado izquierdo (IT-CAT, Q-UTIL-24 propuesta; no implementado). Rust: `ByteIterator`; casos uc-0506..uc-0528; fila `partial`.

---

## AC-UTIL-006.5 — Adaptadores java.io: ByteString.asInputStream, ByteIterator.asInputStream y ByteStringBuilder.asOutputStream

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ByteString.asInputStream` (desde 1.1.0) expone los bytes como un `InputStream`: `UnsynchronizedByteArrayInputStream` sobre el array (o la franja) para `ByteString1C`/`ByteString1`, y la secuencia de los fragmentos para `ByteStrings` (`SequenceInputStream`). `ByteIterator.asInputStream` (AC-UTIL-006.4) y `ByteStringBuilder.asOutputStream` (AC-UTIL-006.3) completan los adaptadores. La clase `UnsynchronizedByteArrayInputStream` está en AC-UTIL-022.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:308,505-506,637-638 (1.7.1)` (`asInputStream`), `:1581-1587` (`asOutputStream`)
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/ByteStringInputStreamSpec.scala:28-80`

**Precondiciones.** Ninguna.

**Comportamiento.** Lee el contenido completo del `ByteString` en orden; vacío -> `read()` = `-1`, `read(Array(1))` = `-1`; después de agotar, `-1` (medido: lecturas `97,98,99,-1` sobre un rope de 3 bytes). `read(buf, 0, 10)` devuelve los bytes disponibles (medido 3, luego -1). `ByteString.fromArray(bytes, 1, 3).asInputStream` lee solo `"bcd"` (el recorte, no el array).

**Errores.** Los de `UnsynchronizedByteArrayInputStream` (AC-UTIL-022); `read` fuera de rango -> `IndexOutOfBoundsException`.

**Orden / concurrencia / ciclo de vida.** El stream no es thread-safe ni cierra nada (`close` es no-op por herencia de `InputStream`).

**Dependencias.** AC-UTIL-006.1, AC-UTIL-022.

**Equivalente Rust propuesto y diferencias observables.** `impl std::io::Read for ByteStringReader` (y `std::io::Write for ByteStringBuilder`); `read` devuelve `Ok(0)` en lugar de `-1` al terminar (diferencia de convención de `std::io`, no de contenido). `mark/reset/skip` se mapea a `Seek`/`BufRead` solo si algún consumidor lo requiere.

**Aserciones.**
- A1 `ByteString1.empty.asInputStream.read() == -1`, `read(Array(1)) == -1`, contenido `""`/`"abc"`: ByteStringInputStreamSpec.scala:29-35.
- A2 `ByteString1C` vacío/`"abc"` y `ByteString.fromArray(bytes, 1, 3)` -> `"bcd"`: :37-44.
- A3 `ByteStrings` vacío y `"a"+"bc"` -> `"abc"`: :46-53.
- A4 *no afirmada por spec*: `read(buf, off, len)` agotado, `-1` repetido.

**Oráculo / validación prevista.** Test portado A1-A3 adaptando `-1` a `Ok(0)`.

**Decisiones.** Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** `ByteString.asInputStream` sobre un ByteString1C/ByteString1 es `UnsynchronizedByteArrayInputStream` y sobre un rope `SequenceInputStream` (entrega por fragmento); con `Bytes` hay un fragmento. Rust: `ByteString::as_input_stream`, `ByteIterator::into_input_stream`, `ByteStringBuilder: io::Write`; casos uc-0529..uc-0536, IT-17 y BU-08.

---

## AC-UTIL-006.6 — ByteString: factorías y conversiones (String, Array, ByteBuffer, Base64, charset)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Construcción y conversión de `ByteString` desde/hacia tipos externos: `ByteString.apply` (arreglo, `Byte*`, `IterableOnce[Byte]`, `Integral*`, `ByteBuffer`, `String` con/sin charset), `fromArray`, `fromArrayUnsafe`, `fromInts`, `fromString`, `fromByteBuffer`, `UTF_8`, `empty`/`emptyByteString`, y salidas `utf8String`, `decodeString(charset)`, `encodeBase64`/`decodeBase64`, `asByteBuffer`/`asByteBuffers`/`getByteBuffers`, `toByteBuffer`, `toArray`/`toArrayUnsafe`, `copyToArray`, `copyToBuffer`, `mapI`.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:31-161 (1.7.1)` (compañero), `:201-215,281-308` (ByteString1C), `:384-435,489-506` (ByteString1), `:612-646,808-823` (ByteStrings), `:1010-1160`
- tests: `ByteStringSpec.scala:408-428,472-492,1004-1025` (copyToArray), `:1166-1184,1204-1221,1402-1412`

**Precondiciones.** Un `Array` mutado después de `fromArrayUnsafe`/`toArrayUnsafe` rompe la inmutabilidad (contrato "Unsafe API" documentado: `:79-118,1035-1052`).

**Comportamiento.**
- Las factorías seguras copian; las `*Unsafe` envuelven sin copiar (`ByteString1C(array)` / `ByteString1(array, offset, length)`). `fromArrayUnsafe(array, offset, length)` no valida el rango (medido: `([1,2,3],1,2)` -> longitud 2).
- `ByteString(string)` = UTF-8; `ByteString(string, "nombre")`/`Charset`. `fromInts(255, 256, -1)` -> `(-1, 0, -1)` (truncamiento `toByte`, medido). `fromByteBuffer`/`apply(ByteBuffer)` **consume** el buffer (medido: tras `get()`, resultado `(2,3)` y `position == 3`).
- `decodeString(charset)`: vacío -> `""`; `ByteStrings` compacta primero. `utf8String` = `decodeString(UTF_8)` con el reemplazo estándar de `new String` para secuencias inválidas (medido: `[0xff, 0xfe]` -> 2 caracteres U+FFFD).
- `encodeBase64`/`decodeBase64`: `java.util.Base64` básico; vacío -> `this`; base64 inválido -> `IllegalArgumentException` (medido: `Illegal base64 character 21` para `"a!"`). Propiedad: `encodeBase64 == ByteString(Base64.getEncoder.encode(a.toArray))` y `decodeBase64` invierte.
- `asByteBuffer`: `ByteBuffer` de solo lectura sobre el array (con `slice` si es una franja); `ByteStrings.asByteBuffer = compact.asByteBuffer`; `asByteBuffers` -> uno por fragmento (medido: rope de 2 -> 2; compacto -> 1 e igual a `asByteBuffer`); todos son `isReadOnly`; `getByteBuffers()` es la vista Java equivalente. `toByteBuffer` copia.
- `toArray` copia; `toArrayUnsafe()` devuelve el array subyacente sin copiar si es un único fragmento completo (`theSameInstanceAs`), y copia (nuevo array) para ropes (medido: longitud 3 para el rope).
- `copyToArray(xs, start, len)`: copia `min(len, length, xs.length - start)` bytes (>= 0) y devuelve ese número (tabla de casos en el test); `copyToArray(xs, start)` usa `min(size, xs.length)`.
- `copyToBuffer(buffer)`: copia `min(buffer.remaining, length)` y devuelve el número (medido: buffer de 2 y `"abc"` -> 2).
- `mapI(f)`: `map(f andThen toByte)`.

**Errores.** `UnsupportedEncodingException` (charset por nombre inexistente), `IllegalArgumentException` (Base64), NPE con `null`.

**Orden / concurrencia / ciclo de vida.** Inmutable; el `ByteBuffer` de entrada se consume en el instante de la llamada.

**Dependencias.** AC-UTIL-006.1/.2, AC-UTIL-022 (`asInputStream`).

**Equivalente Rust propuesto y diferencias observables.** `From<&[u8]>`, `From<Vec<u8>>` (sin copia, equivale a `fromArrayUnsafe` seguro por propiedad de Rust), `ByteString::from_utf8_lossy`/`decode_string(encoding)`, `base64` (`std` no lo trae; dependencia externa o implementación propia pequeña), `as_slice`/`chunks()` en lugar de `asByteBuffers`. Las variantes `Unsafe` desaparecen o se vuelven seguras (Rust garantiza la inmutabilidad por propiedad): se registra la diferencia. Charsets distintos de UTF-8: `encoding_rs` o limitar a UTF-8/ASCII/Latin-1 (decisión Q-UTIL-12 / Owner).

**Aserciones.**
- A1 `ByteString(s, UTF_8).decodeString(UTF_8) == s` y round-trip Base64 `encoded == ByteString(Base64.getEncoder.encode(a.toArray)) && encoded.decodeBase64 == a`: ByteStringSpec.scala:1166-1184.
- A2 `asByteBuffers`: un buffer si es compacto e igual a `asByteBuffer`; al reunirlos se recupera el contenido; todos de solo lectura; coinciden con `getByteBuffers`: :1204-1221.
- A3 `fromArrayUnsafe`/`toArrayUnsafe` conservan la misma instancia de array; sobre `bs ++ bs` el resultado es la concatenación: :1402-1412.
- A4 `copyToArray` sobre ByteString1 (franja `startIndex=1,length=3`), ByteString1C y ByteStrings con las 12 combinaciones `(start, len)`: :408-428, :472-492, :1004-1025.
- A5 `calling copyToBuffer`: :1597-1607 (iterador) y propiedad de `Vector` en :1360-1371.
- A6 *no afirmada por spec*: saturaciones de `fromArray`, `fromInts`, consumo del `ByteBuffer`, errores de Base64/charset, `utf8String` con inválidos; caracterización con jar (valores medidos arriba).

**Oráculo / validación prevista.** Test portado A1-A5; diferencial con jar para A6.

**Decisiones.** Q-UTIL-12 (representación y dependencias `bytes`/`base64`/`encoding_rs`).


**Actualización P04 (2026-11-18, Pekko 2.0.0).** Nuevo en 2.0.0: `decodeString(charset, from, until)` (rango saturado; BS-20). Medido: el reemplazo U+FFFD de la JVM trata un sustituto codificado `ED A0..BF xx` como una sola unidad (BS-19, BS-31); `decodeBase64` acepta falta de relleno y falla con `Illegal base64 character <byte con signo en hexadecimal>`, `Input byte array has wrong 4-byte ending unit`, `... incorrect ending byte at <i>`, `Last unit does not have enough valid bits` o `Input byte[] should at least have 2 bytes for base64 bytes` (BS-22, BS-32); los alias de charset (`latin1`, `utf8`, `ascii`, ...) se resuelven como en `Charset.forName` (BS-33). Q-UTIL-22 propuesta: charsets fuera de UTF-8/US-ASCII/ISO-8859-1 y dependencias. Casos uc-0469..uc-0479, uc-0484..uc-0486; fila `partial`.

---

## AC-UTIL-006.7 — ByteString: búsqueda (indexOf, lastIndexOf, contains, indexOfSlice, containsSlice, startsWith)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Familia de búsqueda sobre bytes con semántica de `Seq`: `indexOf(elem)`, `indexOf(elem, from)`, `indexOf(elem, from, to)` (`to` exclusivo, desde 1.3.0), `lastIndexOf(elem[, end])`, `contains`, `indexOfSlice(slice[, from])`, `containsSlice`, `startsWith(prefix[, offset])`, con variantes especializadas para `Byte` (sin boxing) y para `Array[Byte]`.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:236-271 (1.7.1)` (ByteString1C), `:451-487` (ByteString1), `:730-806` (ByteStrings), `:927-992` (base: `indexOf` sobrecargas, `indexOfSlice`, `contains`)
- tests: `ByteStringSpec.scala:646-1003,1026-1108`
- mima: `actor/src/main/mima-filters/1.2.x.backwards.excludes/bytestring-indexOf-overload.excludes`

**Precondiciones.** Ninguna; los índices fuera de rango se saturan.

**Comportamiento.**
- `indexOf(elem, from)`: devuelve el primer índice `>= max(from, 0)` y `< length` con ese byte, o `-1`; `from >= length` -> `-1`. `indexOf(elem, from, to)`: restringe además a `< to`; `to <= from` o `from >= length` -> `-1`; `to` se acota a `length`. Idéntico resultado entre `ByteString1`, `ByteString1C`, rope y su `compact`.
- `lastIndexOf` y `indexOf` genérico (`B >: Byte`) son los de `IndexedSeq`.
- `indexOfSlice(slice, from)`: longitud 0 -> 0; longitud 1 -> `indexOf(head, from)`; en otro caso usa `indexOf(head, from, length - slice.length + 1)` y verifica el resto (`:966-990`). `containsSlice` y `startsWith(prefix[, offset])` son los de `Seq` (los especializados con `Array[Byte]` se afirman en tests).
- `contains(elem)` = `indexOf(elem, 0) != -1`.

**Errores.** Ninguno (devuelve `-1`/`false`).

**Orden / concurrencia / ciclo de vida.** Puro e inmutable; búsqueda secuencial.

**Dependencias.** AC-UTIL-006.1.

**Equivalente Rust propuesto y diferencias observables.** `fn index_of(&self, byte: u8, from: usize, to: usize) -> Option<usize>` (Rust usa `Option` en lugar de `-1`; a mapear 1:1 en los tests portados), `memchr` o iteradores para la búsqueda de byte, `windows`/`memmem` para subcadenas, `starts_with`. El "from negativo" de Pekko se modela con `isize`/saturación o se documenta que Rust sólo acepta no negativos (diferencia aceptable si se registra).

**Aserciones.**
- A1 `indexOf` en vacío, ByteString1, ByteStrings (`"abc"+"efg"`), compacto y largo: ByteStringSpec.scala:647-674; `lastIndexOf`: :676-702, :770-822.
- A2 `indexOf(elem, from)` con `from` negativo/dentro/fuera, incluyendo franjas (`ByteString1(array, 3, 3)`) y ropes: :704-768 (por ejemplo `byteStrings.indexOf('c', 3) == -1`, `byteStrings.indexOf('g', 5) == 5`); especializada con `Byte`: :825-922.
- A3 `contains` genérico y especializado: :924-948.
- A4 `indexOf(elem, from, to)`: `byteString1.indexOf('a', 0, 0) == -1`, `indexOf('a', -1, 1) == 0`, ropes `indexOf('c', 2, 3) == 2`, `indexOf('c', 2, 2) == -1`, `indexOf('g', 0, 5) == -1`: :950-1003.
- A5 `containsSlice`: :1027-1039; `indexOfSlice` (ByteString y Array, con `from`, franja con offset): :1041-1087; `startsWith` especializado: :1089-1108.

**Oráculo / validación prevista.** Test portado A1-A5 (los casos son deterministas); no se necesita oráculo.

**Decisiones.** Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** 2.0.0 añade `lastIndexOf(elem[, end])`, `lastIndexOfSlice`, `endsWith(Array)` e `indexOfSlice/startsWith` con `Array[Byte]` (`@since 2.0.0`), y busca con `SWARUtil` (AC-UTIL-028); `matchesAt`/`compareBytesTo` son `private[pekko]`. Casos uc-0480..uc-0483 y BS-R*; los resultados son los del contrato.

---

## AC-UTIL-006.8 — ByteString: serialización Java (SerializationProxy, identidades 0/1/2)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Un `ByteString` serializado con `java.io.ObjectOutputStream` se escribe vía `writeReplace` -> `SerializationProxy`: un byte de identidad de representación seguido del contenido (`ByteString1` = 0, `ByteString1C` = 1, `ByteStrings` = 2) y se reconstruye en `readResolve`, de modo que `deserialize(serialize(x)) == x` para cualquier representación. El formato de `ByteString1C` se mantiene compatible con Akka (comprobado con un hex fijo en Scala 2.12).

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:173-180,278-279,320-324,346-349,498,550-567,648-651,825-856 (1.7.1)`
- tests: `ByteStringSpec.scala:1374-1400`

**Precondiciones.** Sólo aplica con serialización Java activa (`allow-java-serialization`, AC-SER-004).

**Comportamiento.** `ByteString1`: `writeInt(length)` + bytes; `ByteString1C`: igual (a través de `toByteString1`); `ByteStrings`: `writeInt(n fragmentos)` + cada fragmento. Lectura: identidad inválida -> `IllegalArgumentException("Invalid serialization id <n>")` (`:848`); `ByteString1.readFromInputStream` lee como `ByteString1C` y lo convierte; `ByteStrings` suma longitudes.

**Errores.** `IllegalArgumentException` por identidad inválida; `EOFException` de la lectura de `readFully` si falta contenido.

**Orden / concurrencia / ciclo de vida.** Sin estado compartido.

**Dependencias.** AC-SER-004 (serialización Java), AC-UTIL-006.1.

**Equivalente Rust propuesto y diferencias observables.** La serialización Java no tiene equivalente en Rust y depende de Q-X-3/AC-SER-004. Efecto observable a preservar si el proyecto implementa un serializador de ByteString para compatibilidad de red: el cuerpo `(longitud i32 BE, bytes)` por fragmento y las identidades 0/1/2; si se omite la serialización Java, la fila queda como diferencia explícita (propuesta, no exclusión). Rust no puede leer el stream de `ObjectOutputStream` sin parser de ese formato (cabecera `aced0005`, descriptores de clase).

**Aserciones.**
- A1 (sólo Scala 2.12) el hex serializado de `ByteString("teststring", "UTF8")` es el de Akka (`aced00057372002d...`): ByteStringSpec.scala:1377-1386; **en el árbol fijado (2.13) la condición `versionNumberString.startsWith("2.12")` es falsa y la aserción no se ejecuta**.
- A2 `deserialize(serialize(bs)) == bs` para cualquier `ByteString` generada: :1388-1392.
- A3 con una concatenación grande (`1000` x1 ++ `1000` x2): :1394-1399 (regresión #20901).
- A4 *no afirmada por spec*: error de identidad inválida (derivada del código `:848`).

**Oráculo / validación prevista.** No se porta (serialización Java fuera de Rust). Si se decide un formato propio: test de ida y vuelta con las propiedades A2/A3.

**Decisiones.** Q-X-3 (mensajes dinámicos), Q-CFG-11 (serialización) y el módulo AC-SER; Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** 2.0.0 añade la identidad 3 (`ByteString2`) a 0/1/2 (`ByteString.scala:1071`, `SerializationProxy` :2146-2170, `Companion` :2164-2175 con `Invalid serialization id <n>`). Sin equivalente grabable; Q-UTIL-26 propuesta; fila `partial` hasta AC-SER.

---

## AC-UTIL-007 — Index / ConcurrentMultiMap

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Multimapa concurrente clave -> conjunto ordenado de valores (ordenado por `valueComparator`, sin duplicados). Altas/bajas se serializan por clave; las lecturas no bloquean.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Index.scala:22-198`
- `actor/src/main/scala/org/apache/pekko/util/Index.scala:200-206 (ConcurrentMultiMap)`

**Condiciones.**
- `put(k, v)` -> true si el valor no estaba asociado a la clave, false si ya estaba (IndexSpec.scala:56-60).
- `remove(k, v)` -> true si lo desasoció (y elimina la clave si queda vacío); false si no estaba (IndexSpec.scala:66-76). `remove(k)` -> Some(copia de los valores) o None si la clave no existía (IndexSpec.scala:77-82).
- `removeValue(v)` lo quita de todas las claves y elimina las claves que quedan vacías (IndexSpec.scala:84-97).
- `valueIterator(k)` iterador vacío si la clave no existe; `findValue(k)(f)` primer valor (en orden del comparador) que cumple `f`; `foreach(fun)` aplica a cada par; `values` = unión de conjuntos; `keys`; `isEmpty` (puede dar falsos negativos mientras corre el último remove: Index.scala:181-184); `clear()`.
- Concurrencia: `put` reintenta (spin) si encuentra un conjunto vacío (marcado como removido por `remove(k)`).

**Orden / concurrencia / ciclo de vida.** Escrituras sincronizadas sobre el `ConcurrentSkipListSet` de la clave (`set.synchronized`); lecturas sin lock (iteradores débilmente consistentes). `remove(k)` vacía el set original para avisar a los `put` en curso.

**Dependencias.** Dependencias: AC-EVT (EventStream/ managed channels?) no usa Index directamente según lo leído; consumidores no verificados.

**Equivalente Rust propuesto y diferencias observables.** `Index<K, V>` con `Mutex<HashMap<K, BTreeSet<V>>>` o mapa concurrente sharded; comparador de valores explícito. Diferencias observables: iteración sobre instantánea (copia) en lugar de débilmente consistente; `isEmpty` sin falsos negativos (Q-UTIL-10).

**Aserciones.**
- A1 `put` + `valueIterator` devuelve el valor: IndexSpec.scala:51-54.
- A2 `put` repetido devuelve true y luego false; varios valores/claves: IndexSpec.scala:56-64.
- A3 `remove(k,v)` true/false y `remove(k)` Some(set)/None, luego iterador vacío: IndexSpec.scala:66-82.
- A4 `removeValue(1)` deja {2,3}, {2}, {2}: IndexSpec.scala:84-97.
- A5 `foreach` visita 6 pares y `findValue` encuentra cada valor: IndexSpec.scala:98-106.
- A6 `clear` -> `isEmpty` true: IndexSpec.scala:108-112.
- A7 10 000 tareas paralelas de put/remove/lectura sin excepciones y las claves ≥ 5 siempre con valores: IndexSpec.scala:114-155.

**Oráculo.** No aplica: sin oráculo de jar necesario (estructura pura); se prueba portando IndexSpec 1:1.

**Decisiones.** Q-UTIL-10.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `Index.scala` re-leído contra 2.0.0: sin cambios de comportamiento. Equivalente (Q-UTIL-10 (a)): `Mutex` sobre `HashMap<K, Vec<V>>` con iteración sobre instantánea. A1-A7 cubiertas (IndexSpec portado, casos IX-*, CC-01).

---

## AC-UTIL-008 — SubclassifiedIndex / Subclassification

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Índice mutable que respeta relaciones de subclase entre claves: al agregar una clave hereda los valores de su superclase más específica; agregar/quitar un valor aplica a todas las subclases. Devuelve los cambios (clave -> conjunto de valores) para actualizar una caché externa.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/SubclassifiedIndex.scala:17-30 (Subclassification)`
- `actor/src/main/scala/org/apache/pekko/util/SubclassifiedIndex.scala:32-73 (Nonroot)`
- `actor/src/main/scala/org/apache/pekko/util/SubclassifiedIndex.scala:76-231 (addKey, addValue, removeValue, findValues, integrate, mergeChangesByKey)`

**Condiciones.**
- `addKey(k)`: si ninguna clave conocida es igual ni superclase, crea un nodo con los valores de la raíz; si hay una superclase, delega en ella; devuelve el diff para la caché (líneas 111-127).
- `addValue(k, v)`: aplica `v` a todas las claves que son subclase de `k` (incluida `k`); si `k` es desconocida la inserta primero (líneas 135-150). `removeValue(k, v)` análogo y devuelve para cada clave afectada el conjunto completo restante (`findValues`) porque pueden haberse creado nodos nuevos (líneas 157-163).
- `removeValue(v)` lo quita de todas las claves (línea 185). `integrate` reubica nodos existentes que son subclase del nuevo y compensa herencia múltiple con `findSubKeysExcept` (líneas 222-228).
- Contrato de `Subclassification`: `isSubclass` debe considerar iguales las clases iguales; la clase no valida el contrato (línea 31).
- Errores: ninguno explícito; el resultado de `mergeChangesByKey` no tiene orden definido (Map -> Seq).

**Orden / concurrencia / ciclo de vida.** No es thread-safe: el llamador (EventStream) sincroniza; la raíz y los nodos usan `var` sin sincronización.

**Dependencias.** AC-EVT-003 (EventStream/SubchannelClassification), AC-EVT-001..; Q-X-6.

**Equivalente Rust propuesto y diferencias observables.** `SubclassifiedIndex<K, V>` genérico sobre un trait `Subclassification<K>`; en Rust la relación de subclase no existe: el trait lo implementa quien declare la jerarquía de canales (Q-X-6).

**Aserciones.**
- A1 suscribir a B3 y luego a A (superclase) entrega a B3 sólo b3 y luego a A todo incluido c y b2; tras `unsubscribe(B2)` ya no llegan b2/c pero sí b3 y a: EventStreamSpec.scala:150-174 (a través de EventStream, no del índice).
- A2 `subscribe/unsubscribe` con clases y traits actualiza la caché en subscribe, unsubscribe, unsubscribe-all y publish: EventStreamSpec.scala:176-279.
- A3 los diffs exactos devueltos por addKey/addValue/removeValue no están afirmados por ningún spec: derivada del código SubclassifiedIndex.scala:111-236.

**Oráculo.** No aplica: estructura interna sin API pública observable; la procedencia es el código y EventStreamSpec.

**Decisiones.** Q-X-6, Q-UTIL-10.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `SubclassifiedIndex.scala` re-leído contra 2.0.0 (líneas 19-231). Lo observable es lo de A3: diffs de addKey/addValue/removeValue y findValues, medidos con el jar en una jerarquía con herencia múltiple (casos SI-*). Quedan fuera A1/A2 (EventStream, AC-EVT-003).

---

## AC-UTIL-009 — WildcardIndex (matching de paths de deployment con * y **)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Índice inmutable de patrones de path con comodines: `*` (un elemento), `elemento*` (prefijo) y `**` terminal (todos los elementos restantes). Lo usa el Deployer para encontrar el Deploy de un path.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala:19-47 (WildcardIndex)`
- `actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala:49-114 (WildcardTree)`

**Condiciones.**
- `insert(elems, d)`: si el último elemento es `**` va al árbol de dobles comodines; si no, al de simples; `elems` vacío no inserta nada, pero `Array.empty` devuelve un índice (WildcardIndexSpec.scala:22-26).
- Error: elemento que termina en `**` sin ser `**` -> IllegalArgumentException("double wildcard can't be used as a suffix (e.g. /user/actor**), only as a full subPath element (e.g. /user/actor/**)") (WildcardIndex.scala:66-68; WildcardIndexSpec.scala:72-74).
- `find(elems)`: busca primero con el árbol de simples (`findWithSingleWildcard`: coincidencia exacta del elemento, luego `*`, luego sufijo-comodín `prefix*` por `startsWith`, la primera que aparezca en el HashMap), si no hay resultado usa el de dobles (`findWithTerminalDoubleWildcard`, que recuerda el último `**` visto como alternativa).
- `**` no terminal se ignora: `a/**/c` no encuentra nada (WildcardIndexSpec.scala:94-98). `**` terminal también cubre el path más corto sólo si hay al menos un elemento tras el prefijo: `/double` no coincide con `/double/**` (DeployerSpec.scala:228).
- Sin elementos -> `find` devuelve None; índice vacío nunca encuentra nada (WildcardIndexSpec.scala:76-80).

**Orden / concurrencia / ciclo de vida.** Inmutable: cada insert devuelve una copia (el Deployer usa `AtomicReference` + CAS, Deployer.scala:256-265). El orden de resolución entre varios `prefix*` aplicables depende del orden de iteración del `HashMap` (derivada del código: WildcardIndex.scala:87-89).

**Dependencias.** Consumidor: AC-CFG-012 (Deployer). Q-UTIL-8.

**Equivalente Rust propuesto y diferencias observables.** `WildcardIndex<T>` inmutable (persistent map o clon por insert) con trie de `String`. Diferencia: la elección entre varios sufijos-comodín solapados depende del orden de iteración del mapa de Scala (no determinista entre implementaciones): se propone orden de inserción documentado (Q-UTIL-8).

**Aserciones.**
- A1 insertar con arrays de strings y encontrar `a/b`, `a`; `x` y `[]` -> None: WildcardIndexSpec.scala:22-34.
- A2 comodín simple `a/*` y `a/*/c`: WildcardIndexSpec.scala:36-49.
- A3 sufijo `a*`, `b/c*`, `a/b*`, `b/c*/d`: WildcardIndexSpec.scala:51-70.
- A4 `a**` falla con IllegalArgumentException: WildcardIndexSpec.scala:72-74.
- A5 `**` terminal: `a/**` cubre `a/b`, `a/x/y`; `**` solo cubre todo: WildcardIndexSpec.scala:82-92.
- A6 `a/**/c` no se comporta como comodín: WildcardIndexSpec.scala:94-98.
- A7 elegir entre `prefix*` solapados: NO afirmada por spec; hallazgo del código WildcardIndex.scala:87-89.

**Oráculo.** No aplica: estructura interna; se porta WildcardIndexSpec 1:1.

**Decisiones.** Q-UTIL-8.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `WildcardIndex.scala` re-leído contra 2.0.0. Hallazgos medidos (WI-15, WI-16): `isEmpty` ignora los hijos `prefix*` y reinsertar `prefix*` reemplaza el subárbol; la elección entre `prefix*` solapados (A7) depende del orden de un HashMap de Scala: Q-UTIL-21 (propuesta entonces; aceptada (c) el 2026-11-27).


**Actualización P10 PR-B (2026-11-20, instrucción de paridad estricta del Owner, propuesta en #279).** A7 (elegir entre `prefix*` solapados) deja de ser una divergencia: `WildcardTree` mantiene los hijos `prefix*` en el orden de iteración de un `immutable.HashMap` de Scala 2.13 (`util/scala_hash_order.rs`: `String.hashCode` UTF-16, `improve`, trie de cinco bits por nivel con las entradas solas antes que los nodos hijo, y colisiones completas en orden de inserción). Medido contra el oráculo (DL-60..DL-135, `dr-0059..dr-0136`; fuzz `tools/parity/fuzz_wildcard_order.py`, resultado histórico del 2026-11-20, no repetido: 3 x 20 000 conjuntos sin diferencias; DL-132/133: con dos prefijos de igual `hashCode` gana el primero insertado). La fila siguió `partial` a la espera de Q-UTIL-21 y de la propuesta de #279; ambas aceptadas (Q-UTIL-21 el 2026-11-27).

---

## AC-UTIL-010 — MessageBuffer / MessageBufferMap

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `MessageBuffer` es una cola FIFO mutable, no thread-safe, de pares `(mensaje, ActorRef)` para almacenar mensajes dentro de un actor. `MessageBufferMap[I]` es un `HashMap[I, MessageBuffer]` con operaciones por identificador, incluida `drop`, que publica un `Dropped` en deadLetters por cada mensaje descartado.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/MessageBuffer.scala:24-176` (MessageBuffer y nodo), `:184-316` (MessageBufferMap)
- consumidores (fuera del grupo): `cluster-sharding/.../Shard.scala`, `cluster-tools/.../ClusterSingletonProxy.scala`, `PerGroupingBuffer.scala`, `ClusterClient.scala`, `persistence/.../RecoveryPermitter.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/MessageBufferSpec.scala:28-192`

**Precondiciones.** Ninguna; `message` y `ref` pueden ser `null` (un buffer vacío devuelve `(null, null)` en `head`).

**Comportamiento (MessageBuffer).**
- `append(message, ref)` agrega al final y devuelve el mismo buffer; `size` cuenta; `isEmpty`/`nonEmpty`.
- `head()` devuelve el primer par sin quitarlo; en un buffer vacío `(null, null)` (medido: `(null,null)`); `getHead()` es la versión Java con `Pair`. `dropHead()` quita el primero y es no-op si está vacío; al vaciarse la cola queda sin cola (`_tail = null`).
- `foreach(f)`/`forEach(Procedure2)` recorren de cabeza a cola en orden de inserción.
- `filterNot(p)` (interno): conserva los pares para los que `p` es falso, en el mismo orden, y recalcula `size`, cabeza y cola (casos: quitar el primero, el último, uno intermedio, varios, todos).

**Comportamiento (MessageBufferMap).**
- `add(id)` crea un buffer vacío si no existe; `append(id, m, ref)` lo crea si falta; `remove(id)`; `contains(id)`; `getOrEmpty(id)` devuelve el buffer o uno vacío **no registrado**; `size` = número de ids; `totalSize` = suma de tamaños; `foreach(f)` itera sobre las entradas del `HashMap` (orden de la tabla hash, no de inserción).
- `drop(id, reason, deadLetters)`: por cada par del buffer envía `Dropped(msg, reason, ref, noSender)` a `deadLetters` en orden FIFO, luego quita la entrada y devuelve el número de mensajes. **Con un `id` ausente lanza `NullPointerException`** (`bufferMap.get(id)` es `null` y se llama `entries.nonEmpty`; medido con el jar), antes de tocar `deadLetters`.

**Errores.** NPE en `drop` de id inexistente (defecto de Pekko; ningún test lo cubre); ninguna otra.

**Orden / concurrencia / ciclo de vida.** No thread-safe (documentado en la clase). FIFO estricto en `MessageBuffer`; `MessageBufferMap.foreach` no tiene orden contractual.

**Dependencias.** `Dropped` y `ActorRef.noSender` (AC-EVT / AC-REF); AC-UTIL-016.1 no.

**Equivalente Rust propuesto y diferencias observables.** `MessageBuffer<M, R>` sobre `VecDeque<(M, R)>` con `append`, `head() -> Option<&(M, R)>`, `drop_head`, `for_each`, `retain` (para `filterNot`); `MessageBufferMap<I: Hash+Eq, M, R>` sobre `HashMap<I, MessageBuffer>`. Diferencias: `head()` devuelve `Option` en lugar de `(null,null)` (diferencia de tipos); el `drop` de un id ausente debe decidirse: propuesta `0` mensajes sin enviar nada y registrar la diferencia con el NPE de Pekko (Q-UTIL-15); el orden de `foreach` del mapa no es contractual, por lo que se usa `HashMap` y los tests no deben fijar orden.

**Aserciones.**
- A1 `isEmpty`/`nonEmpty` antes y después de `append`: MessageBufferSpec.scala:27-36.
- A2 `append` y `dropHead` mantienen `size`, `head()` devuelve `("m1","s1")` y luego `("m2","s2")`, y `dropHead()` sobre vacío no cambia `size`: :38-59.
- A3 `foreach` respeta el orden `m1->s1:m2->s2:m3->s3:` y tras `dropHead` `m2->s2:m3->s3:`: :61-73.
- A4 `filterNot` de primero, de último, de un elemento entre seis (con `dropHead` y `append` intercalados) y de varios: :75-157.
- A5 `MessageBufferMap`: `contains/add/append/remove/getOrEmpty/totalSize`: :159-176; varios buffers y orden por buffer: :178-190.
- A6 *no afirmada por spec*: `drop` (ni con id presente ni ausente), `foreach` del mapa, `getHead`/`forEach` Java, `size` del mapa; caracterización: `drop` publica `Dropped` FIFO y devuelve el número (derivada del código `:265-272`), NPE ausente (medido).

**Oráculo / validación prevista.** Test portado A1-A5; `drop` con un `DeadLetters` de prueba (integración con AC-EVT) y diferencial con jar para el NPE.

**Decisiones.** Q-UTIL-10 (estructuras concurrentes; aquí no thread-safe), Q-UTIL-15 (nuevo: `drop` de id ausente).


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `MessageBuffer.scala` re-leído contra 2.0.0: sin cambios de comportamiento; `drop` de un id ausente lanza NullPointerException (MB-DROP-ABSENT). Q-UTIL-15 (b) aceptada: Rust devuelve 0 (divergencia autorizada y documentada).

---

## AC-UTIL-011 — TokenBucket / NanoTimeTokenBucket

*unidad UTIL · visibilidad internal-accidental · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Token bucket de capacidad `capacity` y un token cada `nanosBetweenTokens`; `offer(cost)` devuelve cuántos ns hay que demorar el elemento (0 si sale ya). No protege contra llamadas antes de tiempo.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/TokenBucket.scala:19-91 (TokenBucket)`
- `actor/src/main/scala/org/apache/pekko/util/TokenBucket.scala:93-98 (NanoTimeTokenBucket)`

**Condiciones.**
- Precondiciones: `capacity >= 0` ("Capacity must be non-negative.") y `nanosBetweenTokens > 0` ("Time between tokens must be larger than zero nanoseconds.") vía `require` (TokenBucket.scala:20-21); `offer(cost < 0)` -> IllegalArgumentException("Cost must be non-negative") (línea 55).
- `init()` debe llamarse antes de usar: `availableTokens = capacity`, `lastUpdate = currentTime`.
- `offer`: tokens llegados = floor((now-lastUpdate)/period) (rama rápida para 1 tick), `lastUpdate` avanza en múltiplos exactos de `period`; disponible = min(disponible + llegados, capacity); si alcanza resta y devuelve 0; si no, `delay = (cost - disponible) * period - (now - lastUpdate)`, deja 0 tokens y `lastUpdate = now + delay` (líneas 54-88).
- Overflow: si el delay no cabe en un Long positivo el resultado es indefinido (Scaladoc líneas 44-46).
- `NanoTimeTokenBucket` usa `System.nanoTime()` (monótono, puede dar la vuelta).

**Orden / concurrencia / ciclo de vida.** No thread-safe; `currentTime` es abstracto (los tests lo fijan a mano).

**Dependencias.** Ninguna fila. Consumidores en Pekko Streams (fuera).

**Equivalente Rust propuesto y diferencias observables.** `TokenBucket` con trait `Clock` (nanos i64) y aritmética `wrapping_*`; `NanoTimeTokenBucket` sobre `Instant`. Sin diferencias previstas.

**Aserciones.**
- A1 empieza lleno: offer(1) x3, offer(7) = 0, offer(3) = 3 (cap 10, período 1): TokenBucketSpec.scala:28-38.
- A2 distintas tasas: (10,2): 0,0,10; (8,3): 0,6 y tras t=6 offer(3)=9: :40-55.
- A3 mayor que capacidad: (10,2) offer(5)=0, offer(20)=30, t=30 offer(1)=2, t=34: 0 y 2: :57-70.
- A4 capacidad 0: offer(10)=20 y t=40 offer(10)=20: :72-80.
- A5 tasa de llegada menor que producción: sin demora: :82-91.
- A6 capacidad máxima mantenida: t=100000 offer(20)=10: :93-100.
- A7 tiempo negativo y wrap-around de Long: :102-126.
- A8 mantener intervalos regulares (15, 2, 5; 0, 2): :128-168.
- A9 costo 0, tasas muy lentas (T = Long.MaxValue >> 10): :170-201.
- A10 equivalencia con el bucket ideal para 4 capacidades x 3 períodos x 3 llegadas x 4 tiempos iniciales (incl. Long.MinValue/MaxValue) x 3 costos: :203-259 (usa Random: fijar semilla al portar).
- A11 mensajes de `require`: derivada del código TokenBucket.scala:20-21,55 (*no afirmada por spec*).

**Oráculo.** No aplica: se porta TokenBucketSpec 1:1 con reloj inyectado.

**Decisiones.** Q-UTIL-10.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `TokenBucket.scala` re-leído contra 2.0.0 y medido: los mensajes de `require` incluyen el prefijo `requirement failed: ` y la excepción de `offer` con coste negativo no (TB-12). Toda la aritmética del tiempo hace wrapping de Long.

---

## AC-UTIL-012 — BoundedBlockingQueue

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `BoundedBlockingQueue[E](maxCapacity, backing)` convierte cualquier `java.util.Queue` vacía en una `BlockingQueue` de capacidad limitada con un único `ReentrantLock` (no justo) y dos condiciones (`notEmpty`, `notFull`). `put`/`take` bloquean; `offer`/`poll` son inmediatas; `offer`/`poll` con timeout esperan como máximo ese tiempo. Es la base de `BoundedMailbox`/`BoundedDequeBasedMailbox` (AC-MBX-007/009).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/BoundedBlockingQueue.scala:27-299`
- consumidor: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/BoundedBlockingQueueSpec.scala:49-916` (79 casos; 18 marcados `TimingTest`)

**Precondiciones (constructor).** `backing` no nulo (`IllegalArgumentException("Backing Queue may not be null")`); `maxCapacity > 0`; `backing.size() == 0`; si `backing` es `BlockingQueue`, `backing.remainingCapacity >= maxCapacity` (los `require` lanzan `IllegalArgumentException("requirement failed")`).

**Comportamiento.**
- `put(e)`: NPE si `e == null`; toma el lock interrumpible; mientras `backing.size >= maxCapacity` espera `notFull`; luego `backing.offer(e)` (exigido con `require`) y `notEmpty.signal()`.
- `take()`: espera `notEmpty` mientras `backing` esté vacío; `poll`, `require(e ne null)`, `notFull.signal()`.
- `offer(e)`: NPE si nulo; si `backing.size == maxCapacity` devuelve `false` sin bloquear; si no inserta, `notEmpty.signal()` y `true`. `offer(e, timeout, unit)`: reintenta tras `awaitNanos` mientras queden nanosegundos; `false` al vencer.
- `poll()`: `null` si vacío, si no `notFull.signal()`. `poll(timeout, unit)`: `null` al vencer.
- `remove(e)`/`contains(e)`: NPE si nulo; `remove` señala `notFull`. `clear()`: `notFull.signalAll()`. `remainingCapacity() = maxCapacity - backing.size`. `size`, `peek`, `isEmpty`, `toArray` delegan bajo lock.
- `drainTo(c[, max])`: NPE si `c` nulo; `IllegalArgumentException` si `c eq this` o `c eq backing`; `max <= 0` -> 0; extrae hasta `max` y señala `notFull.signalAll()` si extrajo.
- `containsAll`, `removeAll`, `retainAll` bajo lock; si cambian el contenido, señalan `notFull` (si queda hueco) y `notEmpty` (si queda algún elemento).
- `iterator()`: instantánea (copia de `backing.toArray`) bajo lock; `remove()` sobre el iterador quita **ese objeto por identidad** (`eq`) del `backing`; `remove()` antes de `next()` o dos veces seguidas -> `IllegalStateException`; `next()` agotado -> `NoSuchElementException`.

**Errores.** `NullPointerException`, `IllegalArgumentException`, `IllegalStateException`, `NoSuchElementException`, `InterruptedException` (en los métodos con `lockInterruptibly`: `put`, `take`, `offer`/`poll` con timeout).

**Orden / concurrencia / ciclo de vida.** Seguridad entre hilos con un solo lock; las esperas re-comprueban la condición tras despertarse (los tests con señales espurias fijan que `put` y `take` no avanzan hasta que hay hueco/elemento). Sin justicia entre productores y consumidores (`new ReentrantLock(false)`).

**Dependencias.** `BoundedMailbox` (AC-MBX-*); `java.util.Queue` abstracta (en Rust: un trait propio).

**Equivalente Rust propuesto y diferencias observables.** `BoundedBlockingQueue<E, Q: Queue<E>>` con `Mutex` + dos `Condvar` (modelo exacto) o `tokio::sync::mpsc::channel` si la cola sólo se usa desde código async; `std::sync::mpsc::sync_channel` no ofrece `remove`, `iterator`, `drainTo` ni `retainAll`. Preserva los contratos de capacidad, bloqueo y señales. Diferencias: no hay `InterruptedException` (la cancelación de hilos en Rust es cooperativa: propuesta, `wait_timeout` + token de cancelación o ignorar); `E <: AnyRef` y `null` desaparecen (`Option`); los `require` sobre `backing` son invariantes de tipo.

**Aserciones.**
- A1 constructor: `null` rechazado, cola no vacía rechazada, vacía aceptada, `maxCapacity = 0` rechazado, `BlockingQueue` de menor capacidad rechazada: BoundedBlockingQueueSpec.scala:48-76.
- A2 `remainingCapacity` consulta el tamaño del backing: :81-87.
- A3 `put`: rechaza nulos, llama a `offer` del backing, señala `notEmpty`, bloquea con la cola llena durante 100 ms, bloquea hasta que hay hueco y comprueba el tamaño tras una señal espuria: :91-164.
- A4 `take`: llama a `poll`, señala `notFull`, bloquea en vacío, bloquea hasta que hay elemento, comprueba tamaño tras señal espuria: :168-241.
- A5 `offer`: rechaza nulos, no bloquea con cola llena (`true` luego `false`), llama al backing, señala `notEmpty`: :245-271.
- A6 `offer` con timeout (no bloquea si hay hueco, señala, bloquea al menos el timeout, devuelve `false` al vencer, termina antes si se libera hueco, comprueba el tamaño): :274-364.
- A7 `poll`, `poll` con timeout, `remove`, `contains`, `clear`, `peek`: :367-533.
- A8 `drainTo`: rechaza `null`, la propia cola y el backing; vacía y agrega al destino: :535-572.
- A9 `containsAll`, `removeAll`, `retainAll` (devuelven `false` si no cambian nada): :575-642.
- A10 `iterator`: vacío sin elementos; `remove()` antes de `next()` -> `IllegalStateException`; dos `remove()` seguidos -> `IllegalStateException`: :645-683.
- A11 *no afirmada por spec*: orden de señales en `removeAll/retainAll`, `InterruptedException`, mensaje de los `require`.

**Oráculo / validación prevista.** Test portado A1-A10 con un `Queue` de prueba que registre eventos (`QueueTestEvents` del spec) y tiempos con `TimingTest`; el oráculo no aporta (es pura concurrencia). Cobertura de los 79 casos exige puerto individual.

**Decisiones.** Q-UTIL-10, Q-UTIL-11 (cancelación/interrupciones), Q-X-4 (runtime).


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `BoundedBlockingQueue.scala` re-leído contra 2.0.0: sin cambios de comportamiento. El spec tiene 62 bloques `in` (no 79): los ~8 que rechazan `null` no tienen contraparte en Rust.

---

## AC-UTIL-013 — StablePriorityQueue / StablePriorityBlockingQueue / PriorityQueueStabilizer

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Colas de prioridad que respetan FIFO entre elementos de igual prioridad: `PriorityQueueStabilizer` envuelve cada elemento con un número de secuencia (`AtomicLong`, incrementa antes de usar) y compara primero con el comparador del usuario y, si empata, por `signum(seq1 - seq2)`. `StablePriorityQueue(capacity, cmp)` usa `java.util.PriorityQueue`; `StablePriorityBlockingQueue` usa `PriorityBlockingQueue`. Las usan `UnboundedStablePriorityMailbox`/`BoundedStablePriorityMailbox` (AC-MBX).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/StablePriorityQueue.scala:23-87`
- consumidor: `actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/dispatch/StablePriorityDispatcherSpec.scala:27-106`, `MailboxConfigSpec.scala`

**Precondiciones.** `capacity >= 1` (lo exige `PriorityQueue`); el comparador debe ser consistente; `E <: AnyRef`.

**Comportamiento.** `offer(e)`: NPE si `e == null`; envuelve y delega. `peek`/`poll` devuelven el elemento (o `null`). `size` delega. `iterator()` itera el backing (orden de heap, no de prioridad) y su `remove()` delega. Dos elementos de igual prioridad salen en orden de inserción aunque se inserten tras elementos de otras prioridades.

**Errores.** NPE por elemento nulo; `capacity <= 0` -> `IllegalArgumentException` de `PriorityQueue`.

**Orden / concurrencia / ciclo de vida.** `StablePriorityQueue` no es thread-safe; `StablePriorityBlockingQueue` lo es (el contador es atómico; el empate en el comparador usa `signum` sobre diferencia de `Long`, correcto salvo desbordes tras 2^63 inserciones). La estabilidad entre hilos depende del orden en que `incrementAndGet` se ejecuta, no del orden de llegada al backing.

**Dependencias.** AC-MBX-* (mailboxes con prioridad estable).

**Equivalente Rust propuesto y diferencias observables.** `BinaryHeap<Reverse<(priority, seq)>>`/clave compuesta `(prio, seq)` con `seq` en un `AtomicU64`/contador. Preserva el orden. Diferencias: `BinaryHeap` es max-heap (invertir), `iterator()` no tiene orden contractual; el comparador de Pekko entero con signo se mapea a `Ord`.

**Aserciones.**
- A1 con mailbox no acotado y acotado, 200 enteros barajados enviados a un actor hijo (no raíz, para evitar que `UnstartedCell` reordene) salen ordenados por prioridad conservando el orden FIFO de los de igual prioridad (todos los >100 comparten prioridad 101 y deben conservar el orden de envío): StablePriorityDispatcherSpec.scala:50-71 y :71-105 (`lo ++ hi`: los 100 primeros por prioridad ascendente y los restantes en el orden en que se enviaron, tras barajar).
- A2 *no afirmada por spec*: iterador, NPE por nulo, `StablePriorityBlockingQueue` en aislamiento.

**Oráculo / validación prevista.** Test portado A1 sobre la cola (sin mailbox) y, con integración AC-MBX, sobre el mailbox.

**Decisiones.** Q-UTIL-10.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `StablePriorityQueue.scala` y `StablePriorityDispatcherSpec` re-leídos contra 2.0.0: sin cambios de comportamiento.

---

## AC-UTIL-014 — TypedMultiMap

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Multimapa **inmutable** `TypedMultiMap[T <: AnyRef, K[_ <: T]]` donde el tipo de los valores asociados a una clave depende (a nivel de tipos) de la clave. En tiempo de ejecución es un `Map[T, Set[Any]]`. La usan el Receptionist de Typed (`LocalReceptionist`, `ServiceKey`) y el de cluster; está en alcance por esos consumidores.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/TypedMultiMap.scala:41-125`
- consumidores: `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/receptionist/LocalReceptionist.scala`, `ServiceKey.scala`
- tests: `actor-tests/src/test/scala-2/org/apache/pekko/util/TypedMultiMapSpec.scala:33-77`

**Precondiciones.** Coherencia de tipos clave/valor garantizada por el compilador (los tests `shouldNot compile` fijan que `inserted(Key(1))(MyValue(42L))` no compila).

**Comportamiento.**
- `empty` -> mapa sin claves; `inserted(key)(value)` agrega `value` al conjunto de `key`; `get(key)` -> conjunto (vacío si no hay clave); `keySet` -> claves con conjunto en `map`.
- `removed(key)(value)`: si el conjunto no contiene `value` devuelve `this`; si queda vacío **elimina la clave**.
- `valueRemoved(value)`: quita `value` de todos los conjuntos; **una clave cuyo conjunto era exactamente `{value}` desaparece**; conserva las demás (`map.collect { case (k, set) if set != Set(value) => (k, set - value) }`).
- `keyRemoved(key)` quita la clave.
- `setAll(key)(values)` **reemplaza** el conjunto y no elimina la clave si `values` está vacío (la clave sigue en `keySet` con conjunto vacío: contradice el comentario "claves con conjuntos no vacíos").
- `++ other` suma mapas **sobrescribiendo** las claves comunes (no mezcla; hay un `FIXME` en el código). `equals`/`hashCode` por el `Map` interno; `toString = "TypedMultiMap(<map>)"`.

**Errores.** Ninguno en tiempo de ejecución.

**Orden / concurrencia / ciclo de vida.** Inmutable; el orden de iteración del `Set` de Scala no es contractual.

**Dependencias.** Q-T-* del Receptionist (Typed); `AC-UTIL` no tiene dependencias.

**Equivalente Rust propuesto y diferencias observables.** Propuesta: `TypedMultiMap<K: Hash+Eq, V: Hash+Eq>` con una sola función de tipo (el sistema de tipos de Scala con tipos dependientes `K[key.type]` no tiene equivalente directo); para la dependencia clave -> valor del Receptionist se puede usar un trait asociado (`trait Key { type Value }`) con `HashMap<Box<dyn AnyKey>, HashSet<Box<dyn Any>>>` y chequeo en construcción. Preserva el comportamiento en tiempo de ejecución (`inserted`, `removed`, `valueRemoved`, `keyRemoved`, `setAll`, `++`); la garantía "no compila" se sustituye por la del trait. Rust requiere una estructura persistente o copia (O(n)) para la inmutabilidad: propuesta `im`/clonación y registrar el coste.

**Aserciones.**
- A1 `inserted`/`removed` en la misma clave: `Set(MyValue(42))`, tras `removed` vacío, con dos valores `Set(42,43)` y quitar uno deja `Set(43)`: TypedMultiMapSpec.scala:35-42.
- A2 varias claves se mantienen independientes: :44-51.
- A3 `valueRemoved(MyValue(42))` quita el valor de todas las claves y **la clave que quedó vacía desaparece de `keySet`** (`Set(Key(2))`): :53-59.
- A4 `keyRemoved(Key(1))` elimina todos los valores y deja `keySet == Set(Key(2))`: :61-67.
- A5 `inserted` y `removed` con valor de tipo incorrecto no compilan: :69-75 (no portable a Rust sin `trybuild`/`compile_fail`).
- A6 *no afirmada por spec*: `setAll` con conjunto vacío, `++` que sobrescribe, `toString`, `equals`.

**Oráculo / validación prevista.** Test portado A1-A4 (más `compile_fail` para A5 si se mantiene el tipado fuerte); A6 se caracteriza leyendo el código.

**Decisiones.** Q-UTIL-11 (tipos dependientes sin equivalente); sin posibilidad de oráculo útil (el comportamiento depende del tipado).


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `TypedMultiMap.scala` re-leído contra 2.0.0: sin cambios de comportamiento. Equivalente: `TypedKey` con tipo asociado.

---

## AC-UTIL-015.1 — Switch (interruptor atómico con acciones bajo lock)

*unidad UTIL · visibilidad deprecated · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `Switch(startAsOn=false)`; `switchOn`/`switchOff` devuelven true si cambió de estado; las variantes con acción la ejecutan bajo lock sólo si cambia y revierten el estado si la acción lanza.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/LockUtil.scala:18-144`

**Condiciones.**
- `switchOn(action)`: si estaba off -> on, ejecuta la acción, true; si ya estaba on -> false sin ejecutar. Si la acción lanza (Throwable) revierte el estado y relanza (líneas 35-45).
- `ifOnYield/ifOffYield/ifOn/ifOff` leen el estado sin lock; `whileOn/whileOff/whileOnYield/whileOffYield/fold/locked` esperan cambios pendientes (synchronized).
- `isOn`/`isOff` sin lock.

**Orden / concurrencia / ciclo de vida.** `transcend` y las variantes `while*` usan `synchronized` sobre el Switch; `switch` es AtomicBoolean (FIXME en el código: pasar a AQS).

**Dependencias.** Ninguna.

**Equivalente Rust propuesto y diferencias observables.** Struct con `AtomicBool` y `Mutex`; las acciones como `FnOnce`; el revertir ante panic requiere `catch_unwind` (Q-X-5).

**Aserciones.**
- A1 on/off: switchOn true, switchOn false, switchOff true, false; isOn/isOff coherentes: SwitchSpec.scala:26-44.
- A2 excepción en la acción revierte: SwitchSpec.scala:46-52.
- A3 `ifOffYield("yes") == Some`, `ifOnYield == None`, `ifOff/ifOn` boolean: SwitchSpec.scala:54-66.
- A4 mismas con `while*`: :68-80.
- A5 `fold("on")("off")` según estado: :82-87.
- A6 `locked` retiene el lock: otro hilo que hace `switchOff` espera a que termine (500 ms): :89-108.

**Oráculo.** No aplica.

**Decisiones.** Q-UTIL-10, Q-UTIL-11, Q-X-5.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `LockUtil.scala` de 2.0.0 sólo define `Switch` (144 líneas). El lock es el monitor del propio objeto (re-entrante).

---

## AC-UTIL-015.2 — ReentrantGuard (deprecado desde 1.2.1)

*unidad UTIL · visibilidad deprecated · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ReentrantGuard.withGuard(body)` ejecuta `body` bajo el lock reentrante y lo libera siempre.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/LockUtil.scala:16`

**Condiciones.**
- Libera el lock aunque `body` lance. Sin tests propios (SwitchSpec no la usa).

**Orden / concurrencia / ciclo de vida.** Lock reentrante del JDK.

**Dependencias.** Ninguna.

**Equivalente Rust propuesto y diferencias observables.** Sin equivalente (Mutex no reentrante); diferencia observable: reentrada en el mismo hilo bloquearía (Q-UTIL-11).

**Aserciones.**
- A1 `withGuard` libera en finally: derivada del código LockUtil.scala:23-26 (*no afirmada por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-UTIL-11.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** ELIMINADO EN 2.0.0 (no es una exclusión, Q-UTIL-11): `ReentrantGuard` ya no existe en `LockUtil.scala` ni en el jar (caso RG-01).

---

## AC-UTIL-016.1 — OptionVal (Option sin asignación)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=documented*

**Contrato observable.** `OptionVal[+A]` es un *value class* (`extends AnyVal`) sobre `x: A` con `null` como ausencia, usado en todo el código interno (~cientos de usos, incluyendo Typed) para evitar la asignación de un `Some`. API: `apply`, `OptionVal.Some(x)` (extractor por nombre), `none`/`OptionVal.None`, `isEmpty`, `isDefined`, `getOrElse`, `toOption`, `contains`, `orNull`, `get`, `toString`. Efecto observable a preservar: la semántica de `Option` con `null` como `None`, `toString` y los errores de `get`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/OptionVal.scala:23-94`

**Precondiciones.** El valor no debe ser `null` si se quiere `Some`: `OptionVal(null)` **es** `None`.

**Comportamiento.** `isEmpty = x == null`; `getOrElse(d)`; `toOption = Option(x)`; `contains(it) = x != null && x == it`; `orNull` requiere `Null <:< A1`; `get` lanza `NoSuchElementException("OptionVal.None.get")` si vacío; `toString` = `"None"` o `"Some(<x>)"`.

**Errores.** `NoSuchElementException("OptionVal.None.get")`.

**Orden / concurrencia / ciclo de vida.** Valor inmutable.

**Dependencias.** Ninguna. Consumidores: `DoubleLinkedList`, `RecencyList`, `FrequencyList`, `SegmentedRecencyList`, `BehaviorImpl`, etc.

**Equivalente Rust propuesto y diferencias observables.** `Option<T>`: Rust ya optimiza la ausencia (niche) sin asignar para `Box`/`Arc`/`NonNull`/referencias. Preserva todo salvo `OptionVal(null) == None` (en Rust no existe `null`; en `Option<Option<T>>`/valores `Option` anidados Pekko no distingue `Some(null)` de `None`) y el texto de `get` (`expect("OptionVal.None.get")` si algún test lo observa). No se necesita un tipo propio; documentar que cada uso de `OptionVal` se traduce a `Option`.

**Aserciones.**
- A1 *no afirmada por spec*: no existe test de `OptionVal` en `actor-tests` (grep de `OptionVal` en el árbol de tests sólo aparece como uso). Caracterización propuesta: `OptionVal(x).get == x`; `OptionVal.None.get` lanza con el texto citado; `toString` `None`/`Some(5)`; `contains` falsa en vacío; `getOrElse` devuelve el valor por defecto en vacío (derivada del código `:49-93`).

**Oráculo / validación prevista.** Test unitario de caracterización en Rust (trivial) sólo si se implementa un tipo propio; en caso contrario, ninguno (se prueba por los consumidores).

**Decisiones.** Q-UTIL-11 (representación de un tipo de optimización de la JVM: `Option<T>`).


**Actualización P76 (2026-11-18, Pekko 2.0.0).** `OptionVal.scala` no cambió (`get` lanza `NoSuchElementException("OptionVal.None.get")`, `toString` `None`/`Some(x)`). Equivalente aceptado por Q-UTIL-11 (a): `Option<T>`; sin caso grabado (A1 no afirmada). Cierre pendiente de Q-UTIL-20.

**Cierre 2026-11-28 (Q-UTIL-20 (a), aceptada): sin implementación propia, equivalencia documentada.** Sin tipo en Rust. Correspondencia (A1, caracterización leída de `:23-94`):

| Pekko | Rust | Diferencia |
|---|---|---|
| `OptionVal(x)`, `OptionVal.Some(x)`, patrón `OptionVal.Some` | `Some(x)`, patrón `Some(x)` | `OptionVal(null)` es `None`; Rust no tiene `null`: la ausencia se escribe `None` |
| `OptionVal.None`, `none` | `None` | - |
| `isEmpty`, `isDefined` | `is_none()`, `is_some()` | - |
| `getOrElse(d)` | `unwrap_or(d)` | - |
| `toOption` | el propio valor | - |
| `contains(x)` (falso si vacío) | `opt.as_ref() == Some(&x)` | - |
| `orNull` | el propio `Option` | no hay `null` |
| `get` (`NoSuchElementException("OptionVal.None.get")`) | `expect`/`unwrap` | panic en vez de excepción; el texto no se reproduce |
| `toString` (`None`, `Some(x)`) | `{:?}` | `Debug` entrecomilla las cadenas |

---

## AC-UTIL-016.2 — ConstantFun (funciones constantes/identidad Scala y Java)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=documented*

**Contrato observable.** Objeto interno con funciones reutilizables para evitar allocaciones: identidad Java/Scala, constantes (`zeroLong`, `oneLong`, `oneInt`, `anyToTrue`, `anyToFalse`, `unitToUnit`, `nullFun`), `any -> None`, `any -> unit`, `(a,b) -> None/unit/true`, `(a,b,c) -> unit/false/third`, `scalaAnyTwoEquals`, `javaAnyTwoEquals`, `javaCreatePairFunction`. Observable: el resultado de cada función y que `javaIdentityFunction` devuelve la **misma instancia** para cualquier `T`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/ConstantFun.scala:26-91`
- tests: `actor-tests/src/test/java/org/apache/pekko/util/ConstantFunJavaTest.java:26-48`

**Precondiciones.** Ninguna.

**Comportamiento.** `javaIdentityFunction[T]` devuelve el argumento (`fun.apply(value) == value`); es un único objeto con un cast a `JFun[T, T]` (`:28-38`). `scalaIdentityFunction` = `conforms`. `scalaAnyToNone` y `javaAnyToNone` -> `None`. `scalaAnyTwoToTrue` -> `true`; `scalaAnyThreeToThird` -> tercer argumento. `javaAnyTwoEquals` compara con `==` (igualdad estructural de Scala; en Java boxea a `java.lang.Boolean`). `JavaPairFunction` crea `Pair(p1, p2)`.

**Errores.** `javaIdentityFunction.apply` declara `throws Exception` pero no lanza.

**Orden / concurrencia / ciclo de vida.** Singletons inmutables.

**Dependencias.** `japi.function.{Function, Function2}`, `japi.Pair` (AC-JAPI-*).

**Equivalente Rust propuesto y diferencias observables.** `std::convert::identity`, closures constantes (`|_| 0u64`, `|_| true`), `Option::None`. No hay que portar nada salvo los pocos sitios que usan estas funciones como valor por defecto de un parámetro (se sustituyen por `Default`/closure en el sitio). La identidad por instancia (`funObj.equals(funStr)`) no es observable en Rust.

**Aserciones.**
- A1 `fun.apply(value) == value` (identidad por referencia): ConstantFunJavaTest.java:26-34.
- A2 `javaIdentityFunction<Object,Object>().equals(javaIdentityFunction<String,String>())` es `true` (misma instancia): :36-45.
- A3 *no afirmada por spec*: el resto de constantes (derivadas del código `:42-89`).

**Oráculo / validación prevista.** Ninguna (A1/A2 no tienen sentido en Rust); se verifican por los consumidores.

**Decisiones.** Q-UTIL-11.


**Actualización P76 (2026-11-18, Pekko 2.0.0).** `ConstantFun.scala` y `ConstantFunJavaTest` sin cambios de comportamiento. Equivalente aceptado por Q-UTIL-11 (a): `std::convert::identity` y closures; A1/A2 inobservables en Rust. Cierre pendiente de Q-UTIL-20.

**Cierre 2026-11-28 (Q-UTIL-20 (a), aceptada): sin implementación propia, equivalencia documentada.** Sin módulo en Rust; cada uso se escribe en su sitio. Correspondencia (A3, leída de `:26-91`):

| Pekko | Rust |
|---|---|
| `javaIdentityFunction`, `scalaIdentityFunction` | `std::convert::identity` (A1: sobre una referencia devuelve la misma referencia) |
| `JavaPairFunction`, `javaCreatePairFunction` | closure `\|a, b\| (a, b)` |
| `scalaAnyToNone`, `javaAnyToNone`, `scalaAnyTwoToNone` | closure que devuelve `None` |
| `scalaAnyToUnit`, `scalaAnyTwoToUnit`, `scalaAnyThreeToUnit`, `unitToUnit` | closure que devuelve `()` |
| `scalaAnyTwoToTrue`, `anyToTrue`, `anyToFalse`, `scalaAnyThreeToFalse` | closure que devuelve `true`/`false` |
| `scalaAnyThreeToThird` | closure `\|_, _, c\| c` |
| `scalaAnyTwoEquals`, `javaAnyTwoEquals` | `PartialEq::eq` |
| `zeroLong`, `oneLong`, `oneInt` | closure que devuelve `0_i64`, `1_i64`, `1_i32` |
| `nullFun` | closure que devuelve `None` (no hay `null`) |

A1 (`fun.apply(value) == value` por referencia, ConstantFunJavaTest) **es observable** en Rust: `std::ptr::eq(std::convert::identity(&value), &value)`. Se cumple por la firma `fn identity<T>(x: T) -> T`, que sólo puede devolver su argumento; no se porta como test porque rakka no tiene código propio aquí y el test probaría la biblioteca estándar. A2 (la misma instancia de función para cualquier `T`) es identidad de objeto en la JVM y no tiene correspondencia: no se reproduce.

---

## AC-UTIL-016.3 — Collections (EmptyImmutableSeq, PartialImmutableValuesIterable)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** (1) `EmptyImmutableSeq` es una `immutable.Seq[Nothing]` vacía cuyo `apply(idx)` lanza `IndexOutOfBoundsException(idx.toString)` y cuyo `iterator.next()` lanza `NoSuchElementException("next on empty iterator")`. (2) `PartialImmutableValuesIterable[From, To]` es una vista inmutable que, dado un `valuesIterator: Iterator[From]`, una función parcial `isDefinedAt`/`apply`, itera sólo los valores para los que la función está definida; `size` y `foreach` se derivan del iterador.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Collections.scala:22-94`
- consumidores: `ByteIterator.scala:49` (`EmptyImmutableSeq.iterator.next()`), `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:33,519` (argumentos vacíos de `createInstanceFor`), `actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala:20,63-86,135` (`PartialImmutableValuesIterable` para `children`/`stats`), `actor/src/main/scala/org/apache/pekko/japi/JavaAPI.scala:23,137`.

**Precondiciones.** `valuesIterator` debe poder recorrerse una vez por llamada a `iterator`.

**Comportamiento.** `iterator` es perezoso con una mirada adelantada (`hasNext` busca el siguiente valor definido y lo guarda; `next` lo entrega y marca consumido; sin siguiente -> `NoSuchElementException("next")`). `size` es `lazy val` (se calcula una vez recorriendo un nuevo iterador). `foreach(f)` recorre un iterador nuevo.

**Errores.** `IndexOutOfBoundsException`, `NoSuchElementException("next")`.

**Orden / concurrencia / ciclo de vida.** Cada llamada a `iterator` crea un cursor independiente; la vista no es thread-safe si `valuesIterator` no lo es.

**Dependencias.** AC-UTIL-006.4 (consumidor).

**Equivalente Rust propuesto y diferencias observables.** `std::iter::empty()` y `Vec::new()`; `filter_map` sobre un iterador (`values.filter_map(|from| if defined(from) { Some(apply(from)) } else { None })`). Sin tipo propio salvo que se quiera conservar el nombre; el único texto observable es el mensaje `"next on empty iterator"` de `ByteIterator` (ver AC-UTIL-006.4).

**Aserciones.**
- A1 *no afirmada por spec*: no existe test de `Collections` en `actor-tests`; caracterización a partir del código (`:24-66`) y del uso en `ByteIterator` (medido `NoSuchElementException: next on empty iterator`).

**Oráculo / validación prevista.** Caracterización con jar para los mensajes (`EmptyImmutableSeq.iterator.next()`), el resto por tests unitarios propios.

**Decisiones.** Q-UTIL-11.


**Actualización P76 (2026-11-18, Pekko 2.0.0).** `EmptyImmutableSeq.iterator` es ahora `Iterator.empty` (`next on empty iterator` lo produce la biblioteca de Scala, no Pekko) y `apply(idx)` sigue lanzando `IndexOutOfBoundsException(idx.toString)`; `PartialImmutableValuesIterable` sigue igual (`NoSuchElementException("next")`); nuevo `Collections.IterableOps.collectToImmutableSeq` (`java.lang.Iterable` + función parcial -> `Seq`, equivalente `filter_map`; sin consumidores en main). Cierre pendiente de Q-UTIL-20 y de P04 (ByteIterator).

**Actualización P04 (2026-11-18, Pekko 2.0.0).** El mensaje `next on empty iterator` que `ByteIterator` hereda de `Collections.EmptyImmutableSeq.iterator` (el `Iterator.empty` de Scala) queda medido y comparado: IT-01 (uc-0506), IT-09 (uc-0514), IT-11 (uc-0516), IT-R* (uc-0523..uc-0528). Rust lo da con `ByteIterator::next_byte`/`get_*` (`ByteStringError::NoSuchElement`). La fila sigue `partial` hasta que el Owner responda Q-UTIL-20.

**2026-11-28 (Q-UTIL-20 (a), aceptada): equivalencia documentada; la fila sigue `partial`.** Falta: (1) la ficha pide cerrar `Collections` junto con ByteIterator (P04) y AC-UTIL-006.4 sigue `partial`; (2) los tests unitarios propios previstos arriba. Correspondencia (A1, caracterización):

| Pekko | Rust | Diferencia |
|---|---|---|
| `EmptyImmutableSeq` (longitud 0) | slice vacío, `Vec::new()`, `std::iter::empty()` | - |
| `EmptyImmutableSeq.apply(idx)` (`IndexOutOfBoundsException(idx)`) | `get(idx)` da `None` (indexar entra en panic) | sin excepción ni su texto |
| `EmptyImmutableSeq.iterator.next()` | `Iterator::next` da `None` | donde llega a un usuario (`ByteIterator`) se reproduce con `ByteStringError::NoSuchElement`, texto `next on empty iterator` (uc-0506, uc-0514, uc-0516, uc-0523..uc-0528) |
| `PartialImmutableValuesIterable.iterator` (perezoso, un cursor por llamada) | `filter_map` sobre un iterador nuevo de los valores | `next` agotado da `None` en vez de `NoSuchElementException("next")` |
| `size` (calculado una vez), `foreach` | `count()`, `for_each` | `count()` no memoriza; sin efecto sobre una vista inmutable |
| `collectToImmutableSeq(pf)` | `filter_map(..).collect::<Vec<_>>()` | - |

AC-UTIL-006.4 sigue `partial` por Q-UTIL-24 (`++` entre iteradores).

---

## AC-UTIL-016.4 — ImmutableIntMap (mapa Int->Int inmutable)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Mapa `Int -> Int` inmutable de copia en escritura, almacenado como un `Array[Int]` ordenado por clave con pares consecutivos, pensado para menos de ~1000 elementos. `get` devuelve `Int.MinValue` si la clave no existe. API: `empty`, `size`, `get`, `contains`, `updated`, `updateIfAbsent`, `remove`, `keysIterator`, `toString`, `equals`, `hashCode`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/ImmutableIntMap.scala:26-167`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/ImmutableIntMapSpec.scala:25-156`

**Precondiciones.** No almacenar `Int.MinValue` como valor (indistinguible de ausente).

**Comportamiento.** `updated(k, v)`: inserta en posición ordenada (búsqueda binaria sobre `kvs`); si la clave existe y el valor cambia copia el array con el nuevo valor; si no cambia devuelve `this`. `updateIfAbsent(k, v)` (`v` por nombre, sólo se evalúa si falta): devuelve `this` si existe. `remove(k)`: copia sin el par; si era el único devuelve `empty`; si no existe, `this`. `keysIterator`: claves en orden ascendente. `toString = "ImmutableIntMap(k -> v, ...)"`, vacío `"ImmutableIntMap()"`. `equals` y `hashCode` comparan/derivan de `Arrays.equals/hashCode(kvs)`: medido `hashCode` de `{10->10}` = 1281.

**Errores.** Ninguno.

**Orden / concurrencia / ciclo de vida.** Inmutable; seguro entre hilos.

**Dependencias.** Único consumidor en el árbol fijado: `actor/src/main/scala/org/apache/pekko/dispatch/affinity/AffinityPool.scala:29,437-448` (caché de selección de cola por hash; AC-DSP-*).

**Equivalente Rust propuesto y diferencias observables.** `BTreeMap<i32, i32>` (clon en escritura) o un `Vec<(i32, i32)>` ordenado con `binary_search`; `get -> Option<i32>` (el centinela `Int.MinValue` desaparece, diferencia de tipos que se registra; si un test porta `get(5) == Int.MinValue` hay que mantener un `get_or_min`). `toString` y `hashCode` de Pekko sólo se reproducen si algún consumidor los observa (`hashCode` = `Arrays.hashCode`: `31 * h + e`).

**Aserciones.**
- A1 vacío: `size == 0`, `keysIterator == Nil`: ImmutableIntMapSpec.scala:27-31.
- A2 `updated` mantiene el orden: `{10}`, `{10,20}`, `{5,10}`, `{5,10,20}`, `{5,10,15,20}`: :33-52.
- A3 reemplazo de valores: `10->11`, `20->21`, `30->31`: :54-60.
- A4 `updateIfAbsent` no cambia una clave presente y agrega una nueva: :62-65.
- A5 `toString`: `"ImmutableIntMap()"`, `"ImmutableIntMap(10 -> 10)"`, `"ImmutableIntMap(10 -> 10, 20 -> 20)"`: :67-71.
- A6 `equals`/`hashCode` iguales para mapas iguales y distintos si cambia una clave o un valor: :73-93.
- A7 `remove` de la primera, intermedia y última clave, de clave ausente (`m1.remove(5) == m1`) y hasta vaciar (`== empty`): :95-109.
- A8 `get` de clave ausente = `Int.MinValue`: :111-117; `contains`: :119-126.
- A9 1000 operaciones aleatorias (`updated`/`remove`, claves 0..99) coinciden con un `Map` de referencia: :128-154.

**Oráculo / validación prevista.** Test portado A1-A9 (el aleatorio con `proptest`); diferencial con jar para `hashCode`.

**Decisiones.** Q-UTIL-11, Q-UTIL-14 (hash).


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `ImmutableIntMap.scala` re-leído contra 2.0.0 (líneas 26-167): sin cambios de comportamiento; `hashCode` es `Arrays.hashCode` del array (1281 para {10 -> 10}).

---

## AC-UTIL-016.5 — PrettyDuration (formateo legible de duraciones)

*unidad UTIL · visibilidad internal-accidental · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `PrettyDuration`: elige la unidad más grande con valor entero >= 1 (días, horas, minutos, segundos, ms, μs, ns) y formatea `%.{precision}g <unidad>[ (<n> ns)]` con Locale.ROOT; infinitos: `-∞ (minus infinity)`, `∞ (infinity)`, indefinido: `undefined`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/PrettyDuration.scala:21-86`

**Condiciones.**
- Normal: `9.nanos` -> `9.000 ns`; `95.nanos` -> `95.00 ns`; `999.nanos` -> `999.0 ns`; `1000.nanos` -> `1.000 μs`; `95.seconds` -> `1.583 min`; `95.hours` -> `3.958 d` (4 cifras significativas).
- `precision <= 0` -> IllegalArgumentException("requirement failed: precision must be > 0") (PrettyDuration.scala:43).
- `includeNanos` agrega ` (<nanos> ns)` tras la abreviatura. Abreviaturas: ns, μs (U+03BC), ms, s, min, h, d (líneas 75-83).
- Derivada del código (no afirmada): `%g` de Java usa notación científica si el valor redondeado >= 10^precision (p. ej. `10000 d` -> `1.000e+04 d`) y no es igual al `%g` de C.

**Orden / concurrencia / ciclo de vida.** n/a: pura.

**Dependencias.** Consumidores: AC-REF-014 (DeadLetterListener), mensajes de log.

**Equivalente Rust propuesto y diferencias observables.** Función con formateo propio de `%g` de Java (Q-UTIL-9); `Duration` Rust necesita enum para infinitos.

**Aserciones.**
- A1 los 10 casos de la tabla: PrettyDurationSpec.scala:25-45.
- A2 `Duration.Inf.pretty` contiene `infinity`: PrettyDurationSpec.scala:47-49.
- A3 `MinusInf` contiene `minus infinity`: :51-53.
- A4 `Undefined` contiene `undefined`: :55-57.
- A5 notación científica y redondeo de %g: NO afirmada por spec; medido con el jar (JDK 11): `10000 days` -> `1.000e+04 d`, `99999 days` -> `1.000e+05 d`, `1 day` -> `1.000 d`, `1500 microseconds` -> `1.500 ms`, `Duration.Inf` -> `∞ (infinity)`. La semántica de `%g` es la de `java.util.Formatter` (no está en upstream/pekko); Q-UTIL-9 queda resuelta por medición (ver decisiones).

**Oráculo.** Posible con jar pekko-actor, pero la clase es private[pekko] (accesible vía su API Java `format`); sin escenario aún.

**Actualización P68 (2026-11-18, Pekko 2.0.0).** Q-UTIL-9 (a) aceptada. Medido con el jar 2.0.0 (JDK 17) por `PrettyDurationOracle` (casos PD-01..PD-19 -> uc-0170..uc-0188), no con JDK 11: A1..A5 se confirman sin cambios (`10000 days` -> `1.000e+04 d`, `99999 days` -> `1.000e+05 d`, `Inf` -> `∞ (infinity)`, `MinusInf` -> `-∞ (minus infinity)`, `Undefined` -> `undefined`). Añade lo medido que el contrato no afirmaba: `precision <= 0` falla antes de mirar el valor (también con `Inf`/`MinusInf`/`Undefined`); una duración <= 0 se muestra en ns (`-95.seconds` -> `-9.500e+10 ns`) y cero es `0.000 ns` (`0 ns` con precisión 1); `%g` redondea HALF_UP sobre los dígitos decimales más cortos del `double` (no su expansión binaria); en un entero `double` >= 2^53 (duración negativa en ns) imprime los dígitos del entero exacto menos k dígitos bajos redondeados (k = 0 hasta 2^57, 1 en 2^58..2^60, 2 en 2^61..2^62, 3 en 2^63). Rust: `PekkoDuration::pretty()` y `PekkoDuration::pretty_with(include_nanos, precision) -> Result<String, IllegalArgumentError>` (`crates/rakka-actor/src/util/pretty_duration.rs`).

**Decisiones.** Q-UTIL-9


---

## AC-UTIL-016.6 — Glob (matching `?`/`*` sin regex)

*unidad UTIL · visibilidad internal-accidental · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `Glob.matches(pattern, input)`: `?` = exactamente un carácter, `*` = cualquier secuencia (incluida vacía), el resto literal; algoritmo lineal con retroceso al último `*` (no regex).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Glob.scala:38-73`
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:46-47 (makePattern, comportamiento que reemplaza)`

**Condiciones.**
- Normal: ver aserciones; `**` equivale a `*`.
- Rendimiento: patrón `(*a)x12*b` contra 36 `a` debe responder en < 3 s (regex tardaba ~47 s); 2000 caracteres en < 5 s.
- Semántica sobre `Char` de UTF-16: `?` consume una unidad de código, no un punto de código (derivada del código Glob.scala:51-52).

**Orden / concurrencia / ciclo de vida.** n/a: pura.

**Dependencias.** Consumidor: SelectChildPattern (actor selection; AC-REF-014/AC-REF-0xx).

**Equivalente Rust propuesto y diferencias observables.** `glob_matches` sobre `&[u16]` o `&str` con decisión explícita (Q-UTIL-8).

**Aserciones.**
- A1 literales: GlobSpec.scala:35-41.
- A2 `?`: :43-48.
- A3 `*`: :50-58.
- A4 coincide con `Helpers.makePattern` para todos los patrones de 0-4 caracteres sobre {a,b,*,?} y entradas de 0-4 sobre {a,b}: :60-73.
- A5 no hay retroceso explosivo: :75-83; escala con la entrada: :85-90.

**Oráculo.** No aplica: pura; la equivalencia con la regex se prueba exhaustivamente como en el spec.

**Decisiones.** Q-UTIL-8



**Actualización P05 (2026-11-18, Pekko 2.0.0).** Q-UTIL-8 respondida (c): la referencia es `Glob.matches` (`Glob.scala:38-73`), medido con el oráculo (casos uc-0084..uc-0099): `?` y `*` aceptan saltos de línea; `?` consume una unidad UTF-16 (`"??"` coincide con U+1F600, `"?"` no); la comparación literal precede a la de comodín, de modo que un `*` de la entrada coincide con un `*` del patrón (`"*a"` contra `"**a"` es falso). `Helpers.makePattern` no se reproduce bit a bit, así que A4 se cubre sin regex: el resultado de Glob para todos los patrones de 0-4 caracteres contra todas las entradas de 0-4 queda grabado (GL-EXH). En Rust: `rakka_actor::util::glob_matches(&str, &str)`.

---

## AC-UTIL-016.7 — Reflect (findConstructor, instantiate, findClassLoader, getCallerClass, findMarker)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Utilidades de reflexión de la JVM: `findConstructor(clazz, args)` (resolución primitiva de sobrecarga), `instantiate` (constructor por defecto, con argumentos o con un `Constructor`), `instantiator`, `findMarker(root, marker)` (tipo genérico de un ancestro), `findClassLoader()` y `getCallerClass`. Las usan `DynamicAccess`/`Props` (creación por clase y argumentos: AC-CFG-016, AC-PROPS).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Reflect.scala:33-293`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/ReflectSpec.scala:33-160`

**Precondiciones.** Clases con constructores accesibles (`setAccessible(true)` se usa en `instantiate`).

**Comportamiento.**
- `findConstructor`: sin argumentos busca el constructor sin parámetros (`getDeclaredConstructor()`); con argumentos filtra los constructores con el mismo número de parámetros donde cada argumento satisface `found.isInstance(arg) || BoxedType(found).isInstance(arg) || (arg == null && !found.isPrimitive)`; **exactamente uno**.
- `instantiate(constructor, args)`: `setAccessible(true)` y `newInstance`; `IllegalArgumentException` de la JVM se re-lanza como `IllegalArgumentException("constructor <c> is incompatible with arguments [<clases>]", causa)`. `instantiate(clazz)` reintenta con `setAccessible` ante `IllegalAccessException`.
- `findMarker` sube por superclases/interfaces hasta encontrar el `marker` (con `ParameterizedType`); si no existe lanza `IllegalArgumentException("cannot find [<marker>] in ancestors of [<root>]")`.
- `findClassLoader()`: el `contextClassLoader` del hilo; si no hay, el del llamador (vía `getCallerClass` saltando marcos de `ActorSystem`, `scala.Option`, `Iterator`, `Reflect`); si no, el de `Reflect`. `getCallerClass` usa `sun.reflect.Reflection.getCallerClass` o `None`.

**Errores.** `IllegalArgumentException("no matching constructor found on <clazz> for arguments [<clases>]")` y `IllegalArgumentException("multiple matching constructors found on <clazz> for arguments [<clases>]")` (los `null` se muestran como `Object`); `NoSuchMethodException` del constructor por defecto es capturado por `Try` y se convierte en el error "no matching constructor".

**Orden / concurrencia / ciclo de vida.** Sin estado (los `getCallerClass` se evalúan al cargar el objeto).

**Dependencias.** AC-UTIL-016.13 (BoxedType; `Reflect.scala:111` lo consume: Reflect depende de BoxedType). Consumidores citados por este contrato: AC-CFG-016 (DynamicAccess), AC-PROPS (creación de actores); el contrato de AC-CFG-016 no cita Reflect, por lo que esa arista no está registrada.

**Equivalente Rust propuesto y diferencias observables.** Rust no tiene reflexión; el efecto observable preservado es la **creación de actores por nombre/fábrica registrada con argumentos** (Q-X-2). `findConstructor` -> un `FactoryRegistry` que resuelve por nombre y por firma de argumentos (`TypeId` de cada argumento), con los mismos errores "ningún constructor coincide" / "varios coinciden" (el caso `null` se modela con `Option`). `findClassLoader`/`getCallerClass`/`findMarker` no tienen equivalente (los class loaders no existen): propuesta de exclusión documentada para aprobación del Owner (Q-UTIL-11), sin eliminar la fila.

**Aserciones.**
- A1 un constructor con 1 parámetro coincide: ReflectSpec.scala:32-33; con 2 parámetros: :35-37.
- A2 con un argumento `null` en uno de los 2 parámetros sigue habiendo un único candidato: :38-41; con `null` y 1 parámetro, `newInstance(null)` funciona: :42-45.
- A3 con varios constructores (`A`, `B`, `(A, B)`) se selecciona el correcto según el tipo del argumento: :46-50.
- A4 con `null` y dos constructores de 1 parámetro posibles -> `IllegalArgumentException` (múltiples coincidencias): :51-55.
- A5 *no afirmada por spec*: textos de los mensajes (derivados del código `:96-123`), `findMarker`, `findClassLoader`, `getCallerClass`, `instantiate` con args incompatibles.

**Oráculo / validación prevista.** Test portado A1-A4 sobre el registro de fábricas (no hay clases JVM); A5 no portable (class loaders). Integración con AC-CFG-016.

**Decisiones.** Q-X-2, Q-CFG-8, Q-UTIL-11.


---

## AC-UTIL-016.8 — LineNumbers (nombre de archivo y líneas de una lambda/clase desde el bytecode)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `LineNumbers(obj)` (Scala), ``LineNumbers.`for`(obj)`` (Java) y `prettyName(obj)` devuelven información de origen de la clase que define el objeto (normalmente una lambda): `NoSourceInfo`, `UnknownSourceFormat(explanation)`, `SourceFile(filename)` o `SourceFileLines(filename, from, to)` (`toString`: `"<f>:<from>-<to>"` o `"<f>:<n>"` si `from == to`). Se obtiene leyendo el bytecode de la clase del recurso (puede bloquear por E/S) e interpretando `SourceFile` y `LineNumberTable`. El efecto visible en otros módulos es el texto de `toString` de los comportamientos de Typed (`Receive(<f>:<l>)`, `Deferred(...)`, `TransformMessages(...)`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/LineNumbers.scala:30-87` (API pública y `prettyName`), `:78-373` (parser de class files; constantes, `LineNumberTable`, Java 8 lambdas)
- consumidores: `actor-typed/.../internal/BehaviorImpl.scala:117,131,149,171`, `InterceptorImpl.scala:234,245`
- tests: `actor-tests/src/test/scala-2/org/apache/pekko/util/LineNumberSpec.scala:22-66` (y `scala-3/`), `LineNumberSpecCodeForScala.scala`, `java/.../LineNumberSpecCodeForJava.java`

**Precondiciones.** El objeto debe ser una lambda serializable, una clase anónima o un objeto cuyo `.class` esté disponible como recurso (`getResourceAsStream`); no funciona con lambdas de Java no serializables.

**Comportamiento.** `prettyName`: `NoSourceInfo` -> `<nombreClase>`; `UnknownSourceFormat(m)` -> `<nombreClase>(<m>)`; `SourceFile(f)` -> `<nombreClase>(<f>)`; `SourceFileLines` -> `<paquete>/<f>:<from>-<to>`. Resuelve `NoSourceInfo` si no hay recurso de la clase o no tiene información de depuración (`:162,181`); un fallo de parseo produce `UnknownSourceFormat("parse error: <mensaje>")` (`:189`). Con `debug = false` no imprime nada.

**Errores.** No lanza: las excepciones no fatales de lectura/parseo se convierten en `UnknownSourceFormat`.

**Orden / concurrencia / ciclo de vida.** Lee el recurso en cada llamada (hay un `FIXME` de memoización LRU); sin estado compartido.

**Dependencias.** Consumido por Typed (`BehaviorImpl`, `InterceptorImpl`).

**Equivalente Rust propuesto y diferencias observables.** Rust no tiene bytecode ni lambdas con metadatos de línea de ejecución. Efecto observable a preservar: que los `Debug`/`Display` de comportamientos y interceptores permitan identificar dónde se definió el closure. Propuesta: capturar la ubicación con `#[track_caller]`/`std::panic::Location::caller()` en los constructores (`Behaviors::receive(...)`) y mostrar `"<archivo>:<línea>"` (un solo número, no un rango), y para objetos sin ubicación `std::any::type_name::<F>()`. **Diferencia observable**: los rangos `from-to` y los nombres de archivos de compilación de Scala/Java no se reproducen; los tests portados que comparan texto de `toString` deben fijar el formato propio. No se reimplementa el parser de class files.

**Aserciones.**
- A1 Scala: función de una línea -> `SourceFileLines("LineNumberSpecCodeForScala.scala", 22, 22)`; de dos líneas -> `(…, 24, 24)`; función parcial -> `(…, 29, 31)`; `def` -> `(…, 35, 36)`: LineNumberSpec.scala:27-43 (versión Scala 2).
- A2 Java: lambda pequeña -> `("LineNumberSpecCodeForJava.java", 29, 29)`, grande -> `(34, 35)`, clase anónima -> `(40, 45)`: :47-62.
- A3 *no afirmada por spec*: `prettyName`, `NoSourceInfo`, `UnknownSourceFormat`, `toString` de `SourceFileLines` (derivadas del código `:30-76`); la rama Scala 3 tiene su propio spec con números distintos (`actor-tests/src/test/scala-3/.../LineNumberSpec.scala`, no leído: el módulo se porta contra el compilador de Scala 2.13).

**Oráculo / validación prevista.** No aplica (depende del compilador de Scala). Test propio de caracterización para el formato de Rust.

**Decisiones.** Q-UTIL-11 (propuesta de equivalente por `track_caller`); la forma del texto debe confirmarla el Owner si algún test de Typed la fija.

**Actualización 2026-11-18 (P75, Pekko 2.0.0).** Q-UTIL-16 aceptada por el Owner: se sigue la opción (a) con retroceso a (b) (un solo número de línea con `#[track_caller]`; `type_name` sin origen). En 2.0.0 `LineNumbers` se reescribió con `MethodHandles` (`writeReplace` por `ClassValue`) y con soporte de constantes Dynamic/Module/Package, pero el texto visible no cambió (medido con el jar 2.0.0: `Raw.java:6`, `Raw.java:8-9`, `NoSourceInfo`; `prettyName` = `<paquete>/<archivo>:<líneas>`). `LineNumbers(obj)` lanza `NullPointerException` para una clase sin classloader (p. ej. `new Object()`): el contrato de "no lanza" no es cierto en ese caso. Implementado: `rakka_actor::util::line_numbers`; oráculo LN-01..LN-11 (uc-0224..uc-0234). Las aserciones A1/A2 (números del compilador) quedan excluidas por la decisión.


---

## AC-UTIL-016.9 — ManifestInfo (versiones de módulos Pekko en el classpath) y chequeo de versiones mezcladas

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ManifestInfo` es una extensión (`ExtensionId` + `ExtensionIdProvider`) que lee `META-INF/MANIFEST.MF` de todos los jars del classpath, conserva los de proveedor conocido (`org.apache.pekko`) en un `Map[título, Version]`, y ofrece `checkSameVersion(productName, dependencies, logWarning[, throwException])`. `ManifestInfo.Version` envuelve `util.Version` (comparable, `equals`, `hashCode`, `toString`). `ManifestInfo.checkSameVersion(product, dependencies, versions)` (interno) construye el mensaje cuando los módulos tienen versiones distintas.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/ManifestInfo.scala:33-189`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/ManifestInfoSpec.scala:26-46`
- claves: `pekko.fail-mixed-versions` (`reference.conf:99-110`, AC-CFG-007); cruza con AC-SYS-020 y AC-CFG-003.

**Precondiciones.** `system.dynamicAccess.classLoader` con recursos; atributos `Implementation-Title|Bundle-Name`, `Implementation-Version|Bundle-Version`, `Implementation-Vendor-Id|Bundle-Vendor`.

**Comportamiento.**
- `versions`: por cada manifiesto con título, versión y proveedor `org.apache.pekko`, `manifests.updated(title, Version(version))`. Un `IOException` se registra como warning ("Could not read manifest information. {}") y se devuelve lo leído.
- Interno `checkSameVersion`: filtra por `dependencies`; si hay más de un valor distinto, `highest = values.max` y devuelve `Some(mensaje)`: `"You are using version <highest> of <product>, but it appears you (perhaps indirectly) also depend on older versions of related artifacts. You can solve this by adding an explicit dependency on version <highest> of the [<k1, k2>] artifacts to your project. Here's a complete collection of detected artifacts: (<v1>, [<a, b>]), (<v2>, [<c>]). See also: https://pekko.apache.org/docs/pekko/current/common/binary-compatibility-rules.html#mixed-versioning-is-not-allowed"`; los grupos van ordenados por versión y los nombres por orden alfabético; `None` si todas coinciden. `toBeUpdated` conserva el orden del mapa filtrado (no ordenado).
- Público `checkSameVersion`: con `logWarning` escribe el mensaje como warning; con `throwException` (por defecto `pekko.fail-mixed-versions`) lanza `IllegalStateException(message)`; devuelve `false` si hubo mensaje y no se lanzó, `true` si coinciden.

**Errores.** `IllegalStateException(message)`.

**Orden / concurrencia / ciclo de vida.** `versions` se calcula una vez al crear la extensión (por sistema, `val`). La extensión se registra con el mecanismo de AC-EXT-001.2.

**Dependencias.** AC-UTIL-005.2 (`Version`), AC-EXT-001, AC-CFG-007, AC-SYS-020.

**Equivalente Rust propuesto y diferencias observables.** No hay jars ni manifiestos en Rust. Efecto observable a preservar: la función pura `check_same_version(product, deps, versions) -> Option<String>` con el mensaje exacto y la API de la extensión. El origen de `versions` pasa a un **registro de versiones de crates** (por ejemplo `CARGO_PKG_VERSION` de rakka-actor, rakka-actor-typed...) que cada crate registra; con un único crate o versiones coherentes, siempre `true`. La diferencia (no leer manifiestos) debe aprobarla el Owner (Q-UTIL-11, Q-CFG-5).

**Aserciones.**
- A1 dado `{actor:1.0.2, persistence:1.0.1, cluster:1.0.1, unrelated:1.0.1}` y módulos `[actor, persistence, cluster]`, el mensaje es exactamente el del contrato con `[pekko-persistence, pekko-cluster]` y `(1.0.1, [pekko-cluster, pekko-persistence]), (1.0.2, [pekko-actor])`: ManifestInfoSpec.scala:27-37.
- A2 *dynver*: `1.0.2` frente a `1.0.2+10-abababef` -> la más alta es la `+10-...` y el mensaje pide actualizar `[pekko-actor]`: :39-45.
- A3 *no afirmada por spec*: la lectura de manifiestos, el filtrado por proveedor, la rama de `IOException`, `throwException`.

**Oráculo / validación prevista.** Test portado A1-A2 de la función pura (golden del mensaje); la lectura de manifiestos no se porta. Depende de `util.Version` y de su defecto de comparación (Q-UTIL-6).

**Decisiones.** Q-UTIL-6, Q-UTIL-11, Q-CFG-5.


---

## AC-UTIL-016.10 — JavaVersion (major de java.specification.version)

*unidad UTIL · visibilidad internal-accidental · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `JavaVersion.majorVersion`: versión mayor del JDK ('1.8' -> 8, '11' -> 11); sin hilos.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/JavaVersion.scala:22-26`

**Condiciones.**
- Lee la propiedad de sistema `java.specification.version`; si falta -> NullPointerException; si no es numérica -> NumberFormatException (derivada del código).

**Orden / concurrencia / ciclo de vida.** Se evalúa al cargar el objeto.

**Dependencias.** AC-DSP-014 (virtualize).

**Equivalente Rust propuesto y diferencias observables.** n/a: no hay JVM; si se necesita una constante de capacidad (hilos virtuales), la define AC-DSP-014.

**Aserciones.**
- A1 derivada del código JavaVersion.scala:23-34 (*no afirmada por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-UTIL-11


---

## AC-UTIL-016.11 — FlightRecorderLoader (carga de JFR vía reflexión)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `FlightRecorderLoader.load[T](casp, fqcn, fallback)` instancia dinámicamente un *flight recorder* de Java Flight Recorder sólo si la JVM es >= 11 y `pekko.java-flight-recorder.enabled` es `true`; si la carga falla devuelve `fallback` y registra un warning; en otro caso devuelve `fallback`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/FlightRecorderLoader.scala:27-42`
- consumidor: `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorFlightRecorder.scala:29`; clave en `reference.conf` (AC-CFG-007)

**Precondiciones.** `casp.classicSystem` es un `ExtendedActorSystem`; la clave booleana existe.

**Comportamiento.** `JavaVersion.majorVersion >= 11 && config.getBoolean("pekko.java-flight-recorder.enabled")` -> `dynamicAccess.createInstanceFor[T](fqcn, Nil)`: `Success(jfr)` -> `jfr`; `Failure(ex)` -> `log.warning("Failed to load JFR flight recorder, falling back to noop. Exception: {}", ex.toString)` y `fallback`. Si la condición es falsa: `fallback`.

**Errores.** Una clave ausente o no booleana lanza `ConfigException` (desde `getBoolean`).

**Orden / concurrencia / ciclo de vida.** Una vez, durante la creación de la extensión que lo usa.

**Dependencias.** AC-UTIL-016.10 (JavaVersion), AC-CFG-007, AC-CFG-016 (DynamicAccess), AC-CFG-002.

**Equivalente Rust propuesto y diferencias observables.** JFR no existe en Rust. Efecto observable a preservar: la clave de configuración se acepta y valida (booleana) y existe un punto de extensión para un recorder de eventos (p. ej. trait `ActorFlightRecorder` con implementación *noop* por defecto); `enabled = on` en Rust devuelve el *noop* y registra el warning de "no disponible" o activa un backend de `tracing` si el Owner lo decide (Q-CFG-7). El texto del warning de fallo no es contractual en Rust.

**Aserciones.**
- A1 *no afirmada por spec*: no existe test de `FlightRecorderLoader` en `actor-tests` (los tests de Typed usan `ActorFlightRecorder` pero no su carga); caracterización: con `enabled=off` devuelve `fallback`; con `on` y clase inexistente devuelve `fallback` y emite el warning (derivada del código `:27-41`).

**Oráculo / validación prevista.** Test de caracterización propio; con `enabled = off` por defecto (`reference.conf`) no hay riesgo.

**Decisiones.** Q-CFG-7 (claves de JVM), Q-UTIL-11.


---

## AC-UTIL-016.12 — ClassLoaderObjectInputStream

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ClassLoaderObjectInputStream(classLoader, is)` es un `ObjectInputStream` que resuelve primero cada clase con `Class.forName(nombre, false, classLoader)` y, si no la encuentra (`ClassNotFoundException`), recurre al comportamiento por defecto de `ObjectInputStream.resolveClass`. Sirve para deserializar con Java serialization en un sistema con class loader propio.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/ClassLoaderObjectInputStream.scala:25-31`
- consumidor: `JavaSerializer.fromBinary` (AC-SER-013), `Serializer.scala:358`

**Precondiciones.** `is` es un stream con formato de serialización Java.

**Comportamiento.** Ver contrato; no inicializa la clase (`false`).

**Errores.** `ClassNotFoundException` del `super.resolveClass` si tampoco se encuentra allí; `IOException` por stream inválido.

**Orden / concurrencia / ciclo de vida.** Una instancia por deserialización; no thread-safe.

**Dependencias.** Ninguna fila. Consumidor: AC-SER-013 (`JavaSerializer.fromBinary`), que le pasa `system.dynamicAccess.classLoader` (AC-CFG-016); la clase no depende de esa fila.

**Equivalente Rust propuesto y diferencias observables.** Sin equivalente: ni la serialización Java ni los class loaders existen en Rust. Efecto observable a preservar: **resolver un tipo por nombre durante la deserialización con un registro de tipos específico del sistema** (el `TypeRegistry` del `ActorSystem`) antes de uno global. Se propone integrarlo en el registro de fábricas/serializadores (Q-X-2, AC-SER-003). Exclusión explícita de la clase sólo con aprobación del Owner (Q-UTIL-11).

**Aserciones.**
- A1 *no afirmada por spec*: no hay test directo en `actor-tests` (sólo se usa en rutas de serialización); caracterización derivada del código `:26-30`.

**Oráculo / validación prevista.** Ninguna (sin equivalente); se cubre con las pruebas de AC-SER-003/004 si se porta el registro de tipos.

**Decisiones.** Q-UTIL-11, Q-X-2, Q-CFG-11.


---

## AC-UTIL-016.13 — BoxedType (primitivo -> clase envoltorio)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `BoxedType(c: Class[_])` devuelve la clase envoltorio de un tipo primitivo (`boolean->Boolean`, `byte->Byte`, `char->Character`, `short->Short`, `int->Integer`, `long->Long`, `float->Float`, `double->Double`, `Unit->BoxedUnit`) y la propia clase si no es primitiva. Lo usa `Reflect.findConstructor` para aceptar argumentos boxeados donde el constructor declara un primitivo.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/BoxedType.scala:16-31`
- consumidor: `Reflect.scala:111`

**Precondiciones.** Ninguna.

**Comportamiento.** `if (c.isPrimitive) toBoxed(c) else c`. Medido con el jar: `BoxedType(int.class) = java.lang.Integer`, `BoxedType(String.class) = String`, `BoxedType(void.class) = scala.runtime.BoxedUnit`.

**Errores.** Un primitivo que no está en el mapa lanzaría `NoSuchElementException` (no existe otro primitivo en la JVM: el mapa es exhaustivo).

**Orden / concurrencia / ciclo de vida.** Mapa inmutable.

**Dependencias.** Ninguna. Consumidor: AC-UTIL-016.7 (`Reflect.scala:111`); BoxedType no depende de Reflect.

**Equivalente Rust propuesto y diferencias observables.** Sin equivalente: Rust no distingue primitivos de envoltorios; las coincidencias de constructor por `TypeId` (AC-UTIL-016.7) ya son exactas. El efecto observable (aceptar un argumento `Integer` para un parámetro `int`) se vuelve trivial. Se registra como "n/a por diseño del lenguaje, con el efecto preservado".

**Aserciones.**
- A1 *no afirmada por spec*: no hay test directo; los casos de ReflectSpec.scala:32-50 usan clases no primitivas. Caracterización medida (arriba).

**Oráculo / validación prevista.** Ninguna.

**Decisiones.** Q-UTIL-11.


---

## AC-UTIL-016.14 — UUIDComparator (orden de UUID de 128 bits)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `UUIDComparator` (`Comparator[UUID]`, adaptado de java-uuid-generator, Apache 2.0) ordena UUIDs: primero por versión (`u1.version - u2.version`); si ambos son versión 1 (basada en tiempo), por `timestamp()` como entero sin signo de 64 bits y, en empate, por los 64 bits menos significativos sin signo; para las demás versiones, por los 64 bits más significativos y luego los menos significativos, **sin signo** ("java.util.UUID compara con extensión de signo, lo cual es erróneo"). Singleton `UUIDComparator.comparator`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/UUIDComparator.scala:24-75`

**Precondiciones.** Ninguna; `timestamp()` sólo se invoca para versión 1.

**Comportamiento.** `compareULongs` compara los 32 bits altos y luego los bajos con `compareUInts` (resta cuando ambos tienen el mismo signo, `±1` si el signo difiere). El resultado puede ser cualquier entero con el signo correcto (no sólo -1/0/1): medido `compare(v4 ...01, v4 ...02) = -1`, `compare(v1, v4) = -3` (diferencia de versiones), `compare(new UUID(0x4000..., 0x8000...01), new UUID(0x4000..., 1)) = 1` (sin signo).

**Errores.** Ninguno.

**Orden / concurrencia / ciclo de vida.** Sin estado.

**Dependencias.** Ninguna interna. Único consumidor en el árbol: `persistence-query/src/main/scala/org/apache/pekko/persistence/query/Offset.scala:21,62` (`TimeBasedUUID.compare`; fuera de estos módulos); `LICENSE`/`NOTICE` declaran que el archivo procede de java-uuid-generator (Apache 2.0).

**Equivalente Rust propuesto y diferencias observables.** `impl Ord for ... ` o `fn compare_uuid(a: Uuid, b: Uuid) -> Ordering` usando `u128`/`(u64, u64)` y `Ord` de enteros sin signo; para la versión 1 la ordenación por `timestamp()` del JDK (60 bits en 100 ns desde 1582, reordenado desde los campos `time_low/mid/hi`) hay que reconstruirla. Diferencia observable: el valor concreto devuelto (`-3`) no se reproduce, sólo su signo (propuesta: devolver `Ordering`); si algún consumidor guarda el entero, hay que decidirlo.

**Aserciones.**
- A1 *no afirmada por spec*: no existe test de `UUIDComparator` en `actor-tests`. Caracterización con el jar (valores medidos arriba): orden por versión, sin signo en los 64 bits, y para v1 por `timestamp`.

**Oráculo / validación prevista.** Diferencial con jar (scripts `tools/oracle/uuid-comparator.jsh` por crear) con UUIDs v1 con timestamps extremos y v4 con bit alto; no depende de la versión de la JVM.

**Decisiones.** Q-UTIL-11 (qué se devuelve: `Ordering` vs entero).

**Actualización P69 (2026-11-18, Pekko 2.0.0).** Medido con el jar 2.0.0 (JDK 17) por `UuidOrderOracle` (casos UU-01..UU-21 -> uc-0189..uc-0209): `UUIDComparator.scala` no cambió respecto de 1.7.1 y A1 se confirma. Añade lo medido que el contrato no afirmaba: el resultado crudo es la resta de las mitades de 32 bits o `±1` (`-15` entre versión 0 y 15, `±2147483647` entre mitades extremas del mismo signo); `timestamp()` de v1 es `time_hi(12 bits) << 48 | time_mid << 32 | time_low` (sin los 4 bits de versión) y lanza `UnsupportedOperationException[Not a time-based UUID]` con otra versión; las versiones 0, 2, 3, 5..15 se ordenan como la 4 (msb y lsb sin signo). Rust: `JavaUuid`, `pekko_compare_uuid` (Int crudo), `compare_uuid` y `Ord` (signo) en `crates/rakka-actor/src/util/uuid_order.rs`; sin parseo ni generación de UUID.


---

## AC-UTIL-016.15 — HashCode (hash compuesto SEED=23, PRIME=37)

*unidad UTIL · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `HashCode.hash(seed, value)` combina un valor en un hash: `37 * seed + contribución`, con `SEED = 23`. Contribución: `Boolean` -> 1/0; `Char`/`Short`/`Int`/`Byte` -> el valor entero; `Long` -> `(v ^ (v >>> 32)).toInt`; `Float` -> `floatToIntBits`; `Double` -> `doubleToLongBits` (como `Long`); `AnyRef` -> `value.hashCode()`; arreglos -> se encadena el hash de cada elemento.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/HashCode.scala:33-126`
- consumidor citado por el inventario: `Version.hashCode` (Version.scala:189-198)

**Precondiciones.** Ninguna.

**Comportamiento.** Valores medidos con el jar: `hash(23, 5) = 856`; `hash(23, true) = 852`; `hash(23, 5000000000L) = 705033556`; `hash(23, "abc") = 97205` (= 37*23 + `"abc".hashCode`=96354); `hash(23, 1.5) = 1073218387`; `hash(23, int[]{1,2}) = 31526`. Para un arreglo se llama `hash(result, JArray.get(value, id))` por elemento, de modo que los elementos se encadenan comenzando en `seed`.

**Errores.** **`hash(seed, null: Any)` lanza `IllegalArgumentException("Unexpected hash parameter: null")`** (medido): la rama `if (value eq null)` (`:48`) es inalcanzable porque el patrón `case value: AnyRef` no casa con `null`.

**Orden / concurrencia / ciclo de vida.** Funciones puras.

**Dependencias.** Consumidor: `Version.hashCode` (AC-UTIL-005.2).

**Equivalente Rust propuesto y diferencias observables.** Funciones `hash_bool/char/short/int/long/float/double` con aritmética `i32` con *wrapping* (`37i32.wrapping_mul(seed).wrapping_add(x)`); el caso `AnyRef` depende del `hashCode` de cada tipo (String de Java: polinomio 31 sobre unidades UTF-16 (ver Q-UTIL-8); en Rust no hay `hashCode` universal) -> sólo se reproduce para los tipos que lo requieran (Version, enteros, strings). Diferencia: el error de `null` desaparece.

**Aserciones.**
- A1 *no afirmada por spec*: no existe test de `HashCode` en `actor-tests`; la caracterización son los valores medidos con el jar (arriba).

**Oráculo / validación prevista.** Diferencial con jar (golden de los valores medidos y de los extremos `Long.MIN_VALUE`, `NaN`, `-0.0`).

**Decisiones.** Q-UTIL-8 (strings UTF-16), Q-UTIL-14 (hash).

**Actualización P65 (2026-11-18, Pekko 2.0.0).** Los valores medidos arriba se confirman con el oráculo 2.0.0 (casos HC-01..HC-11, uc-0122..uc-0132) y se amplían: `Float`/`Double` usan `floatToIntBits`/`doubleToLongBits` (todo NaN colapsa al NaN canónico), `Short`/`Byte` se ensanchan a `Int`, un `Object[]` encadena sus elementos recursivamente (arreglos anidados incluidos) y un `Integer[]` da lo mismo que un `int[]`. `hash(seed, null)` y un `null` dentro de un `Object[]` lanzan `IllegalArgumentException: Unexpected hash parameter: null` (HC-12, HC-13, sin counterpart en Rust). En Rust: `rakka_actor::util::hash_code::{SEED, HashContribution, hash}`; el hash de otros `AnyRef` (colecciones, Option, tuplas de Scala) no se reproduce (Q-UTIL-14 (b), Q-PATH-3).


---

## AC-UTIL-016.16 — WallClock / AlwaysIncreasingClock (reloj de pared estrictamente creciente)

*unidad UTIL · visibilidad internal-accidental · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `WallClock.AlwaysIncreasingClock.currentTimeMillis()`: devuelve `System.currentTimeMillis()` pero nunca repite ni retrocede (si el reloj no avanzó devuelve `anterior + 1`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/WallClock.scala:27-57`

**Condiciones.**
- Atómico: `updateAndGet` con función `if (now <= last) last + 1 else now`.
- Sin errores.

**Orden / concurrencia / ciclo de vida.** Thread-safe por AtomicLong.

**Dependencias.** Consumidores no identificados (public @ApiMayChange).

**Equivalente Rust propuesto y diferencias observables.** `AtomicI64` con `fetch_update`.

**Aserciones.**
- A1 monotonicidad estricta: derivada del código WallClock.scala:46-55 (*no afirmada por spec*).

**Oráculo.** No aplica.

**Decisiones.** Ninguna.

**Actualización P70 (2026-11-18, Pekko 2.0.0).** Medido con el jar 2.0.0 (JDK 17) por `WallClockOracle` (casos WC-01..WC-14 -> uc-0210..uc-0223): `WallClock.scala` no cambió de comportamiento y A1 se confirma. Añade lo medido que el contrato no afirmaba: el estado inicial es 0 (`AtomicLong`), la primera lectura es la hora de pared; con el estado por delante de la hora de pared cada lectura suma 1 (también con 8 hilos concurrentes: resultados distintos y contiguos); el `+1` envuelve de `Long.MAX_VALUE` a `Long.MIN_VALUE` y la lectura siguiente vuelve a la hora de pared; relojes distintos son independientes y `WallClock.AlwaysIncreasingClock` es una instancia compartida. Los casos no dependen del reloj real: el oráculo fija el estado con `set` o compara invariantes. Rust: `crates/rakka-actor/src/util/wall_clock.rs` con fuente de tiempo inyectable.


---

## AC-UTIL-016.17 — ErrorMessages.RemoteDeploymentConfigErrorPrefix

*unidad UTIL · visibilidad internal-accidental · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Constante `RemoteDeploymentConfigErrorPrefix = "configuration requested remote deployment"` usada al rechazar despliegue remoto en LocalActorRefProvider.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/ErrorMessages.scala:19-21`

**Condiciones.**
- Valor fijo; el mensaje completo lo arma el consumidor (AC-SYS-016).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-SYS-016.

**Equivalente Rust propuesto y diferencias observables.** `const &str`.

**Aserciones.**
- A1 valor literal: ErrorMessages.scala:20.

**Oráculo.** No aplica.

**Decisiones.** Ninguna.


---

## AC-UTIL-016.18 — Unused (marcador `@unused` para parámetros no usados)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=documented*

**Contrato observable.** `private[pekko] class unused extends deprecated("unused", "")` (con `@nowarn("msg=deprecated")`): anotación interna que suprime el aviso de parámetro no usado cuando el parámetro es necesario por compatibilidad binaria u otro motivo (`-Ywarn-unused`) y avisa si se **usa** un parámetro marcado. No tiene efecto en tiempo de ejecución.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Unused.scala:36-37 (1.7.1)`
- usos: `ByteString.scala:1084` (`copyToBuffer(@unused buffer)`), `StablePriorityDispatcherSpec.scala:29,33`, etc.

**Precondiciones.** Ninguna. **Comportamiento.** Ninguno en runtime. **Errores.** Ninguno. **Orden / concurrencia / ciclo de vida.** n/a (anotación del compilador).

**Dependencias.** Ninguna.

**Equivalente Rust propuesto y diferencias observables.** Efecto a preservar: "el parámetro no usado no produce aviso". Equivalente: prefijo `_` en el nombre (`_buffer`) o `#[allow(unused_variables)]`. Sin tipo. No requiere test.

**Aserciones.**
- A1 *no afirmada por spec*: sin test (marcador de compilador).

**Oráculo / validación prevista.** `cargo clippy -- -D warnings` (del repositorio) cubre el efecto observable.

**Decisiones.** Q-UTIL-11.


**Actualización P76 (2026-11-18, Pekko 2.0.0).** `Unused.scala` está eliminado en 2.0.0: eliminado en 2.0.0 (no es exclusión). El efecto lo cubre `_nombre` y `cargo clippy -- -D warnings`. Cierre pendiente de Q-UTIL-20.

**Cierre 2026-11-28 (Q-UTIL-20 (a), aceptada): ELIMINADO EN 2.0.0, sin implementación propia.** `@unused` sobre un parámetro corresponde al prefijo `_` (o `#[allow(unused_variables)]`): no hay aviso por no usarlo y no hay efecto en ejecución. Diferencia: la anotación de Pekko, al extender `deprecated`, avisa si el parámetro marcado se usa; Rust no avisa por defecto al usar un `_nombre`.

---

## AC-UTIL-017.1 — FrequencySketch y FastFrequencySketch (Count-Min Sketch con reinicio TinyLFU)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Dos estructuras no thread-safe que estiman la frecuencia de elementos para la política de admisión TinyLFU: `FrequencySketch[A](depth, width, counterBits, resetSize, hasher)` (genérica: profundidad y tamaño de contador configurables) y `FastFrequencySketch[A](width, resetSize)` (profundidad fija 4, contadores de 4 bits, usa `value.hashCode`). Operaciones: `increment(value)`, `frequency(value)` (mínimo de los contadores de las filas), `size` (número de contadores incrementados desde el último reinicio), `toDebugString`. Cuando `size` llega a `resetSize`, **todos los contadores se dividen por la mitad** (reinicio TinyLFU) y `size` se recalcula.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/FrequencySketch.scala:45-94` (fábrica, `Hasher`, `Bits`), `:126-242` (genérica), `:248-377` (rápida), `:383-425` (`debugString`)
- consumidores (fuera de estos módulos): `cluster-sharding/.../internal/EntityPassivationStrategy.scala`, `ClusterShardingSettings.scala`; benchmarks `bench-jmh/.../FrequencySketchBenchmark.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/FrequencySketchSpec.scala:23-266` (genérica), `:269-374` (rápida), `ZipfianGenerator.scala:1-57`

**Precondiciones.** `width` potencia de dos (`require(..., "width must be a power of two")`); `counterBits ∈ {2,4,8,16,32,64}` (`require(..., "counterBits must be 2, 4, 8, 16, 32, or 64 bits")`). La fábrica `apply(capacity, widthMultiplier=4, resetMultiplier=10, depth=4, counterBits=4)` usa `width = widthMultiplier * ceilingPowerOfTwo(capacity)` y `resetSize = (resetMultiplier * capacity).toInt`.

**Comportamiento.**
- Matriz de `depth` filas de `width/slots` `Long` (`slots = 64/counterBits`); contador `c` de la fila `r` para el hash `h`: columna `(h & (width-1)) >>> slotShift`, ranura `(h & slotMask) << counterShift`. `increment` suma 1 a cada contador de cada fila que no esté saturado (máximo `2^bits - 1`, `Long.MaxValue` para 64 bits) y cuenta un incremento de `size` **si al menos una fila cambió**; saturado en todas las filas -> `size` no cambia (`15` incrementos con contadores de 4 bits y 20 llamadas).
- Hashes de fila: `h_i = h1 + i*h2`, con `h1 = hasher.hash(value)` y `h2 = MurmurHash3.finalizeHash(MurmurHash3.mixLast(h1, h1), 2)`; el `Hasher[String]` por defecto es `MurmurHash3.stringHash(value, 135283237)`. `FastFrequencySketch` usa `rehash(value.hashCode)` (Hash Function Prospector: `x = ((x>>>15)^x)*0xD168AAAD; x = ((x>>>15)^x)*0xAF723597; (x>>>15)^x`) y `index(hash, seed) = ((hash + seed)*seed + ((hash+seed)*seed >>> 32)).toInt & (width-1)` con semillas `0xC3A5C85C97CB3127`, `0xB492B66FBE98F273`, `0x9AE16A3B2F90404F`, `0xCBF29CE484222325`.
- Reinicio: por fila, `odd` = contadores impares (`bitCount(row & oddMask)`); cada palabra `w` pasa a `(w >>> 1) & resetMask`; `rowSizes(row) = (rowSizes(row) - odd) >>> 1`; `size = max(rowSizes)`.
- `toDebugString`: tabla Unicode (`╔═╦`...) de todos los contadores, con anchos calculados a partir de `rowWidth` y del máximo del contador.

**Errores.** `IllegalArgumentException("requirement failed: width must be a power of two")` y `("requirement failed: counterBits must be 2, 4, 8, 16, 32, or 64 bits")`.

**Orden / concurrencia / ciclo de vida.** No thread-safe; los incrementos son secuenciales y el reinicio ocurre dentro del `increment` que alcanza `resetSize`.

**Dependencias.** `scala.util.hashing.MurmurHash3` (no está en upstream/pekko: se conoce sólo por la biblioteca de Scala), `String.hashCode`/`hashCode` de Java para `FastFrequencySketch`. Q-UTIL-14.

**Equivalente Rust propuesto y diferencias observables.** Estructura con `Vec<Vec<u64>>`, mismas máscaras y misma aritmética de 32/64 bits con *wrapping*. El resultado numérico sólo coincide con Pekko si se reproducen **bit a bit** `MurmurHash3` de Scala (y, para `FastFrequencySketch`, el `hashCode` del tipo: `String.hashCode` de Java usa unidades UTF-16, Q-UTIL-8). Los tests de Pekko fijan valores exactos (`size shouldBe 500` tras un reinicio, `frequency("foo") shouldBe 1`), que sólo son estables con el hash exacto o con ausencia de colisiones; con otro hash habría que relajarlos a las cotas estadísticas que el propio spec usa en otros casos.

**Aserciones.**
- A1 incrementos y `size`: `foo`=1, `bar`=2, `baz`=3, `size == 6`: FrequencySketchSpec.scala:25-37; saturación a 15: `frequency("foo") == 15`, `size == 15` tras 20 incrementos: :39-45.
- A2 reinicio: tras 999 incrementos `size == 999`; el milésimo dispara el reinicio y `size == 500` con colisiones nulas; ≥ 495 de 499 frecuencias quedan a la mitad; y en la segunda fase `size` entre 300 y 500: :47-90.
- A3 precisión relativa: las frecuencias de elementos populares preservan el orden con ruido (`frequency(i) >= frequency(i+1)`) y accuracy > 0.95 con una distribución Zipf: :92-127.
- A4 contadores de 2 bits (`resetMultiplier=1`): máximo 3, `size == 6 + 2` tras 10 incrementos extra de `foo`, `size == 99` antes del reinicio y `size <= 50` tras él: :129-154; de 8 bits: máximo 255, `size == 6 + 254`, `size == 999` y `size <= 500` tras el reinicio: :156-181; de 16, 32 y 64 bits con el mismo esquema (:183-256, estructura análoga; leer los límites exactos al portar); profundidad 1 y ancho 1 (colisión total): `frequency("foo") == 2`, `frequency("bar") == 2`: :258-266.
- A5 `FastFrequencySketch`: mismos escenarios 1-3 con `Depth=4`, 4 bits: :269-374.
- A6 *no afirmada por spec*: errores de `require`, formato de `toDebugString`, valores concretos de hash.

**Oráculo / validación prevista.** Test portado A1-A5 + diferencial con el jar para fijar `MurmurHash3.stringHash`, `rehash` e `index` (golden de hashes) antes de portar los valores exactos; `toDebugString` se prueba contra un golden del jar para una matriz pequeña.

**Decisiones.** Q-UTIL-14 (hash de Scala), Q-UTIL-8 (UTF-16), Q-UTIL-11. Fila propuesta como equivalente directo (no exclusión).


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `FrequencySketch.scala` de 2.0.0 añade `require(widthLong <= Int.MaxValue, "computed width (...) exceeds Int.MaxValue; reduce capacity or widthMultiplier")` (el ancho se calcula en Long) y limita el tamaño de reinicio a Int.MaxValue; no estaba en 1.7.1. Q-UTIL-14 (a): MurmurHash3 y String.hashCode exactos.

---

## AC-UTIL-017.2 — FrequencyList (lista de frecuencias LFU, con recencia global y envejecimiento dinámico opcionales)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Estructura mutable no thread-safe para seguir la frecuencia de elementos (políticas LFU) con operaciones en tiempo constante: lista doblemente enlazada de nodos de frecuencia, cada uno con una lista de elementos en orden de actualización, más un mapa de búsqueda. `FrequencyList.empty[A](dynamicAging = false)` y `FrequencyList.withOverallRecency.empty[A](dynamicAging = false)` (con reloj `NanoClock`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/FrequencyList.scala:26-271`
- consumidor: `cluster-sharding/.../EntityPassivationStrategy.scala` (fuera de estos módulos)
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/FrequencyListSpec.scala:39-257`

**Precondiciones.** `clock` definido para las operaciones `removeOverall*` (si no: `UnsupportedOperationException("Overall recency is not enabled for this FrequencyList")`).

**Comportamiento.**
- `update(value)`: si existe, incrementa su frecuencia (nodo siguiente o nuevo) y se coloca **al final** de su nueva lista; si es nuevo, entra con frecuencia 1 (sin envejecimiento) o `1 + age` (con envejecimiento) al final de la lista de esa frecuencia. Con reloj, actualiza `timestamp` y mueve a la cola de la recencia global.
- `remove(value)`: quita el elemento si existe. `contains`, `size`, `leastFrequent`/`mostFrequent` (`OptionVal`), `leastToMostFrequent`/`mostToLeastFrequent` (iteradores; dentro de una frecuencia, orden de actualización).
- `removeLeastFrequent()`, `(n)`, `(n, skip)`; `removeMostFrequent()`, `(n)`, `(n, skip)`: devuelven los valores eliminados en el orden de eliminación; `skip` omite un valor concreto; con `n == 1` hay un camino optimizado. `removeLeastFrequent*` con envejecimiento dinámico fija `age = prioridad del último eliminado` (`removeMostFrequent` no cambia `age`).
- Con recencia global: `overallLeastToMostRecent`, `overallMostToLeastRecent`, `removeOverallLeastRecent(n)`, `removeOverallMostRecent(n)`, `removeOverallLeastRecentOutside(duration)` (elementos con `timestamp < clock.earlierTime(duration)`), `removeOverallMostRecentWithin(duration)` (`timestamp > earlierTime`).
- Envejecimiento dinámico: la prioridad de un nodo es `accesses + age` en el momento de la inserción/ascenso (las prioridades antiguas se conservan), de modo que los elementos nuevos no quedan permanentemente por debajo de los antiguos.

**Errores.** `UnsupportedOperationException` (recencia global no habilitada).

**Orden / concurrencia / ciclo de vida.** No thread-safe; todas las operaciones modifican en el sitio y devuelven `this` (o la secuencia eliminada).

**Dependencias.** AC-UTIL-017.5 (DoubleLinkedList), AC-UTIL-017.3 (`RecencyList.Clock`), AC-UTIL-016.1 (`OptionVal`).

**Equivalente Rust propuesto y diferencias observables.** `BTreeMap<u64 prioridad, VecDeque/lista intrusiva>` o una lista doblemente enlazada sobre una arena (`slotmap`/`Vec` con índices) más `HashMap<A, NodeId>`; `Clock` como trait con `NanoClock` sobre `Instant`. Preserva el orden dentro de cada frecuencia y las operaciones de eliminación. Diferencia: listas intrusivas de punteros no se reproducen (observable sólo en rendimiento); `OptionVal` -> `Option`; `UnsupportedOperationException` -> `Result`/`panic`.

**Aserciones.**
- A1 seguimiento de frecuencia: orden `a,b,c` (freq 1), tras `update("a").update("c")` -> `b | a,c`, etc.; `removeLeastFrequent(3) == List("h","i","a")`; `removeLeastFrequent(skip = Some("j")) == List("k")`; `removeLeastFrequent(2, skip = Some("l")) == List("j","m")`; `removeMostFrequent(3) == List("e","c","l")`; `removeMostFrequent(skip = Some("o")) == List("n")`; `removeMostFrequent(2, skip = Some("p")) == List("o","r")`: FrequencyListSpec.scala:50-102.
- A2 recencia global con reloj de prueba (tick por segundo): `removeOverallLeastRecent() == List("a")`, `removeOverallMostRecent() == List("k")`, `removeOverallLeastRecentOutside(3.seconds) == List("c","e","h")`, `removeOverallMostRecentWithin(3.seconds) == List("m","l")`: :104-179.
- A3 envejecimiento dinámico frente a LFU regular: tras `removeLeastFrequent` `age` pasa a 1, 3, 7, 9, 10 y los nuevos elementos entran por encima de los antiguos cuando corresponde (`aging` termina `removeLeastFrequent(3) == List("a","b","c")`): :181-265.
- A4 *no afirmada por spec*: `leastFrequent`/`mostFrequent` (`OptionVal`), excepción de recencia no habilitada, `removeOverallMostRecent(n>1)`.

**Oráculo / validación prevista.** Test portado A1-A3 con el reloj de prueba; no hace falta oráculo (determinista).

**Decisiones.** Q-UTIL-11 (equivalente propuesto).


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `FrequencyList.scala` re-leído contra 2.0.0: sin cambios de comportamiento.

---

## AC-UTIL-017.3 — RecencyList (LRU por reloj)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Lista de recencia mutable no thread-safe (doblemente enlazada + mapa): `update(value)` mueve/agrega al **final** (más reciente) y anota el tiempo del `Clock`; permite quitar los menos/más recientes por número (con `skip`) o por ventana de tiempo. `RecencyList.empty[A]` usa `NanoClock` (`System.nanoTime`); `Clock` es un trait con `currentTime()` y `earlierTime(duration)`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/RecencyList.scala:26-137`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/RecencyListSpec.scala:21-120`

**Precondiciones.** Ninguna.

**Comportamiento.** `update`, `remove` (no-op si ausente), `contains`, `size`, `leastRecent`/`mostRecent` (`OptionVal`), `leastToMostRecent`/`mostToLeastRecent`, `removeLeastRecent()` / `(n)` / `(n, skip)`, `removeMostRecent()` / `(n)` / `(n, skip)` (`skip` descarta los primeros `skip` y quita los `n` siguientes), `removeLeastRecentOutside(d)` (todos con `timestamp < earlierTime(d)`), `removeMostRecentWithin(d)` (`timestamp > earlierTime(d)`). Las operaciones de eliminación devuelven los valores en orden de eliminación; pedir más de los existentes devuelve los que haya.

**Errores.** Ninguno.

**Orden / concurrencia / ciclo de vida.** No thread-safe.

**Dependencias.** AC-UTIL-017.5, AC-UTIL-016.1; consumidores AC-UTIL-017.2/.4.

**Equivalente Rust propuesto y diferencias observables.** `RecencyList<A: Hash+Eq+Clone>` con `Clock` como trait (reloj inyectable para tests); lista enlazada en arena o `LinkedHashMap`-like (`indexmap` con movimiento al final O(n); propuesta: arena). Preserva el orden y las ventanas. Diferencia: `NanoClock` -> `Instant`/`SystemTime` monótono.

**Aserciones.**
- A1 secuencia completa con reloj de prueba: orden tras `update`, `remove`, `removeLeastRecent(3) == List("a","c","e")`, `removeMostRecent(3) == List("i","m","l")`, `removeLeastRecentOutside(3.seconds) == List("h","j")`, `removeMostRecentWithin(3.seconds) == List("r","k","q","p")`, `removeLeastRecent(3, skip = 3) == List("t","u","v")`, `removeMostRecent(3, skip = 2) == List("x","w","s")`, `removeLeastRecent(10)` sobre 4 elementos devuelve los 4, `removeMostRecent(10) == List("c","b","a")`: RecencyListSpec.scala:43-128.
- A2 *no afirmada por spec*: `leastRecent`/`mostRecent`, `NanoClock`.

**Oráculo / validación prevista.** Test portado A1.

**Decisiones.** Q-UTIL-11.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `RecencyList.scala` re-leído contra 2.0.0: sin cambios de comportamiento.

---

## AC-UTIL-017.4 — SegmentedRecencyList (LRU segmentado con niveles protegidos)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Variación de `RecencyList` para LRU segmentado (SLRU): `limits` por nivel; el nivel más bajo (0) es el "probatorio" y los superiores están protegidos. `update` inserta nuevos en el nivel 0 y **promociona** los existentes un nivel (el más alto los mueve al final de su lista). `removeLeastRecentOverLimit()` elimina del nivel 0 el exceso sobre la suma de límites, tras demover el exceso de cada nivel protegido al nivel inferior. `updateLimits(newLimits)` cambia los límites en caliente. Con reloj (`withOverallRecency`) soporta `removeOverallLeastRecentOutside(duration)`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/SegmentedRecencyList.scala:26-188`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/SegmentedRecencyListSpec.scala:23-148`

**Precondiciones.** `limits` no vacío; `levels = limits.size`.

**Comportamiento.** `update(value)`: nuevo -> `appendTo(0)`; existente -> `promote`: nivel + 1 (`removeFromCurrentLevel`, `appendTo`, `adjust(newLevel)`) o, si ya está en el más alto, `moveToBack`. `adjust(level)`: si `sizes(level) > limits(level)`, los `excess` primeros del nivel se **demueven** (se anexan al final del nivel inferior). `removeLeastRecentOverLimit()`: si `size > totalLimit` ejecuta `adjustProtectedLevels()` (de arriba abajo) y quita `size - totalLimit` elementos desde el inicio del nivel 0, devolviéndolos; si no, `Nil`. `removeLeastRecent()`, `leastRecent`, `leastToMostRecentOf(level)`, `sizeOf(level)`, `contains`, `remove`. `removeOverallLeastRecentOutside(d)` exige reloj (`UnsupportedOperationException("Overall recency is not enabled")`).

**Errores.** `UnsupportedOperationException`; `IndexOutOfBounds` si `level` fuera de rango (derivado de la `IndexedSeq`, no afirmado).

**Orden / concurrencia / ciclo de vida.** No thread-safe.

**Dependencias.** AC-UTIL-017.3 (`RecencyList.Clock`), AC-UTIL-017.5, AC-UTIL-016.1.

**Equivalente Rust propuesto y diferencias observables.** `Vec<RecencyListSegment>` + `HashMap<A, (level, node)>`; misma lógica de `promote/demote/adjust`. Preserva el estado observable (`leastToMostRecentOf(level)`).

**Aserciones.**
- A1 2 segmentos `limits = (2, 3)`: tras seis `update` todo en el nivel 0, `removeLeastRecentOverLimit() == List("a")`; después de promociones el estado `[e,f,a] / [b,c,d]` y `removeLeastRecentOverLimit() == List("e")`, etc.: SegmentedRecencyListSpec.scala:46-74.
- A2 4 segmentos `limits = (4,3,2,1)`: `removeLeastRecentOverLimit() == Nil`, luego `List("a","b")`, `List("c","e")`, `List("g","b","d")` con los estados por nivel indicados: :76-107.
- A3 `updateLimits(List(1,4))` tras `limits=(2,8)`: `removeLeastRecentOverLimit() == List("c","d","e","f","g")` y estado `[h] / [i,j,k,l]`: :109-129.
- A4 recencia global: `removeOverallLeastRecentOutside(2.seconds) == List("a","b","c","d","e","f","g")`: :131-156.
- A5 *no afirmada por spec*: excepción de recencia no habilitada, niveles fuera de rango.

**Oráculo / validación prevista.** Test portado A1-A4.

**Decisiones.** Q-UTIL-11.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `SegmentedRecencyList.scala` re-leído contra 2.0.0: sin cambios de comportamiento.

---

## AC-UTIL-017.5 — DoubleLinkedList (lista intrusiva con operaciones "obtener u insertar")

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Lista doblemente enlazada **intrusiva** parametrizada por accesores `getPrevious/getNext/setPrevious/setNext` sobre el nodo. Operaciones: `isEmpty`, `getFirst`/`getLast` (`OptionVal`), `prepend`, `append`, `remove`, `moveToFront`, `moveToBack`, `getFirstOrElsePrepend`, `getLastOrElseAppend`, `getNextOrElseInsert`, `getPreviousOrElseInsert`, `findNextOrElseInsert`, `findPreviousOrElseInsert`, `forwardIterator`, `backwardIterator`. Es la base de `RecencyList`, `FrequencyList` y `SegmentedRecencyList`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/DoubleLinkedList.scala:27-150`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/DoubleLinkedListSpec.scala:25-200`

**Precondiciones.** Un nodo sólo puede pertenecer a una lista a la vez; `remove`/`moveTo*` suponen que el nodo está en la lista (sin comprobación).

**Comportamiento.** `remove(node)` desenlaza y deja los enlaces del nodo en `None`. `moveToFront/Back`: no-op si ya está en ese extremo. `getFirstOrElsePrepend(check, newNode)`: devuelve el primero si cumple `check`, si no antepone `newNode` (evaluado por nombre) y lo devuelve; `getNextOrElseInsert(node, check, newNode)` análogo con el siguiente. `findNextOrElseInsert(node, isBefore, check, newNode)`: avanza mientras `isBefore(siguiente)` y luego `getNextOrElseInsert`. Los iteradores recorren de extremo a extremo y toleran la eliminación del nodo ya entregado.

**Errores.** Ninguno (los accesores nulos no se validan).

**Orden / concurrencia / ciclo de vida.** No thread-safe.

**Dependencias.** AC-UTIL-016.1 (`OptionVal`).

**Equivalente Rust propuesto y diferencias observables.** Lista enlazada en arena (índices en vez de punteros, p. ej. `slab`/`slotmap`) o `Vec`+`VecDeque` si basta con las operaciones que se usan; la intrusividad por accesores no es observable. Registrar que la API pública de rakka no necesita exponerla (`pub(crate)`).

**Aserciones.**
- A1 `start empty`, `prepend` (c,b,a -> a,b,c), `append`, `remove` de intermedio/último/primero hasta vacía: DoubleLinkedListSpec.scala:40-73.
- A2 `moveToFront`/`moveToBack`: orden resultante en cada paso (`c,b,a` -> `b,c,a` -> `a,b,c`): :75-101.
- A3 `getFirstOrElsePrepend`, `getLastOrElseAppend`, `getNextOrElseInsert`, `getPreviousOrElseInsert`, `findNextOrElseInsert`, `findPreviousOrElseInsert`: :103-200 (leer cada caso al portar).
- A4 *no afirmada por spec*: eliminación de un nodo ya desenlazado, nodo en dos listas.

**Oráculo / validación prevista.** Test portado A1-A3.

**Decisiones.** Q-UTIL-11.


**Actualización P03 (2026-11-18, Pekko 2.0.0).** `DoubleLinkedList.scala` re-leído contra 2.0.0: sin cambios de comportamiento.

---

## AC-UTIL-018.1 — FutureConverters (CompletionStage <-> Future)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=documented*

**Contrato observable.** `FutureConverters.asJava(Future[T]): CompletionStage[T]` y `asScala(CompletionStage[T]): Future[T]` (más extensiones `FutureOps.asJava` y `CompletionStageOps.asScala`). En Scala 2.13/3 delegan en `scala.jdk.javaapi.FutureConverters`; en 2.12 existe una implementación propia (`scala-2.12/.../FutureConverters.scala`, no relevante para el árbol 2.13). Son la frontera Java/Scala de `ask`, `Patterns` y `CoordinatedShutdown` (Java API).

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/FutureConverters.scala:30-45 (1.7.1)`; `scala-3/.../FutureConverters.scala` (espejo); `scala-2.12/...` (excluido)
- tests: `actor-tests/src/test/java/org/apache/pekko/util/JavaConverterScala3InlineTest.java:34-46 (1.7.1)`

**Precondiciones.** Ninguna.

**Comportamiento.** Semántica de la biblioteca estándar (`asJava` produce un `CompletionStage` que completa con el mismo resultado/excepción; `asScala` de un `CompletableFuture` completo devuelve un `Future` equivalente). Los métodos no están marcados `inline` para poder invocarse desde Java (regresión de Scala 3 #19346).

**Errores.** Los de la biblioteca estándar.

**Orden / concurrencia / ciclo de vida.** Sin estado propio.

**Dependencias.** `ask` (AC-ASK), `CoordinatedShutdown` (AC-CS, Java API).

**Equivalente Rust propuesto y diferencias observables.** `async fn`/`Future` ya es el tipo de resultado en Rust; no hay conversión Java<->Scala. Efecto preservado: las API que en Pekko devuelven `CompletionStage` (versión Java) tienen en Rust un único `Future`. Propuesta: no portar el tipo; documentar la equivalencia en cada fila que usa `CompletionStage` (Q-X-4, runtime). Exclusión explícita requiere aprobación del Owner (Q-UTIL-11).

**Aserciones.**
- A1 compila desde Java: `FutureConverters.asJava(Future.successful(""))` y `asScala(CompletableFuture.completedFuture(""))`: JavaConverterScala3InlineTest.java:42-43 (prueba de compilación, sin aserción de ejecución).
- A2 *no afirmada por spec*: comportamiento de `asJava/asScala` (delegado a la biblioteca estándar).

**Oráculo / validación prevista.** Ninguna.

**Decisiones.** Q-UTIL-11, Q-X-4.

**Actualización P77 (2026-11-18, Pekko 2.0.0).** `util/FutureConverters.scala` y `JavaConverterScala3InlineTest.java` están eliminados en 2.0.0 (MiMa `future-converters.excludes`: se usa `scala.jdk.javaapi.FutureConverters`): eliminado en 2.0.0, no es exclusión. Equivalente aceptado por Q-UTIL-11 y Q-X-4 (A): un único `Future` de Rust; sin tipo ni caso grabado (A1 es una prueba de compilación, A2 delega en la biblioteca). Cierre pendiente de Q-UTIL-27.

**Cierre 2026-11-28 (Q-UTIL-27 (a), aceptada): ELIMINADO EN 2.0.0, no implementado.** `asJava`/`asScala` no tienen contraparte: una API que en Pekko devuelve `CompletionStage` o `Future` devuelve en rakka el único `Future` de Rust.


---

## AC-UTIL-018.2 — OptionConverters / FunctionConverters / JavaDurationConverters

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle*

**Contrato observable.** (a) `OptionConverters`: `toScala(Optional|OptionalDouble|OptionalInt|OptionalLong)` y `toJava(Option)`, más `RichOptional/RichOption/RichOptionalDouble/Int/Long` (`toScala`, `toJava`, `toJavaPrimitive`, `toJavaGeneric`); en 2.13+ delegan en `scala.jdk`. (b) `FunctionConverters` hereda de `Priority0FunctionExtensions` (2.13+). (c) `JavaDurationConverters`: `asFiniteDuration(JDuration)`, `JDuration.asScala = Duration.fromNanos(self.toNanos)` y `Duration.asJava = JDuration.ofNanos(self.toNanos)`; es lo que usa `Timeout.create` (AC-UTIL-001).

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/util/OptionConverters.scala:30-80 (1.7.1)`; `scala-2.13+/.../FunctionConverters.scala:29-30`; `scala-2/.../JavaDurationConverters.scala:24-35`; `scala-3/.../*Converters.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/Scala212CompatTest.scala:35-78 (1.7.1)` (compilación), `JavaConverterScala3InlineTest.java:36-45`

**Precondiciones.** `JDuration.asScala`: `toNanos` no debe desbordar (`ArithmeticException: long overflow`, medido por Timeout TO-NEG-04/05); `Duration.asJava`: duración finita (`toNanos` sobre `Inf` lanza `IllegalArgumentException`).

**Comportamiento.** `Optional.empty <-> None`, `Optional.of(x) <-> Some(x)`; `OptionalInt/Long/Double` <-> `Option[Int|Long|Double]`; `FunctionConverters`: conversiones entre lambdas Java/Scala. `asScala` de duración: los nanosegundos exactos; `asJava`: `ofNanos(toNanos)`.

**Errores.** `ArithmeticException("long overflow")` al convertir duraciones fuera de `Long` (TO-NEG-04/05 del golden `util-core.golden`), `IllegalArgumentException` para infinitas.

**Orden / concurrencia / ciclo de vida.** Sin estado.

**Dependencias.** AC-UTIL-001 (Timeout, `create`).

**Equivalente Rust propuesto y diferencias observables.** `Option` ya cubre `Optional`/`Option`; `TryFrom<std::time::Duration> for Timeout` (ya implementado, error `ArithmeticError`) es la conversión de duración; `FunctionConverters` no aplica (los closures de Rust no se convierten). Efecto observable preservado: rango y error de overflow de `Timeout.create` (cubierto por el oráculo `util-core`).

**Aserciones.**
- A1 compila en Java y Scala el uso de todas las conversiones (`toJavaPrimitive`, `toScala`, `toJavaGeneric`, `Iterable.single`): Scala212CompatTest.scala:35-77, JavaConverterScala3InlineTest.java:36-45; ninguna aserción de ejecución.
- A2 `Duration`: `JavaDuration.java` ejercita `scala.concurrent.duration.Duration` (no los convertidores): :24-32 (no es contrato de Pekko).
- A3 desbordes de `asScala`: oráculo TO-NEG-04/05 (*no afirmada por spec de Pekko*).

**Oráculo / validación prevista.** Oráculo `util-core.golden` (TO-NEG-*) ya activo en `oracle_util_core`.

**Decisiones.** Q-UTIL-1 (resuelta por el Owner), Q-UTIL-11.

**Actualización P77 (2026-11-18, Pekko 2.0.0).** `OptionConverters`, `FunctionConverters` y `JavaDurationConverters` están eliminados en 2.0.0 (MiMa `javaapi-option`, `function-converters`, `java-duration-converters`); `Timeout.create(java.time.Duration)` usa `scala.jdk.DurationConverters.toScala` (`FiniteDuration(seconds, SECONDS)` si no hay nanos; `seconds*1e9+nanos` si caben en un `Long`, si no `IllegalArgumentException("Java duration PT...S cannot be expressed as a Scala duration")`). Medido en los casos grabados uc-0537..uc-0549 (FR-DUR-01..13) y TO-NEG-04/05; `TryFrom<std::time::Duration> for Timeout` ya los reproducía (caracterización). Sin cubrir: Timeout -> `java.time.Duration` (`toJava`, usada por Actor.scala:158, AbstractActor.scala:173, CachePolicy.scala:33, OptimalSizeExploringResizer.scala:273: filas de otras unidades) e `Optional*`/`FunctionConverters` (equivalente `Option<T>`/closures, sin caso). Estado reconciliado: impl=partial, validation=oracle. Cierre pendiente de Q-UTIL-27.

**Cierre 2026-11-28 (Q-UTIL-27 (a), aceptada): conversores ELIMINADOS EN 2.0.0, no implementados.** `Optional*`/`Option` corresponde a `Option<T>`; las conversiones de funciones no aplican a closures. `Timeout.create(java.time.Duration)` (A3) es `TryFrom<std::time::Duration> for Timeout`, que pertenece a AC-UTIL-001 y queda en sus casos grabados. **Corrección sobre `toJava`:** no hay conversión Timeout -> duración Java. `util/Timeout.scala` 2.0.0 no tiene `toJava`, y `Actor.scala:158`, `AbstractActor.scala:173`, `io/dns/CachePolicy.scala:33` y `OptimalSizeExploringResizer.scala:273` (que el párrafo P77 anterior daba como usos desde `Timeout`) convierten un `FiniteDuration`. Esta fila no deja nada pendiente a otras filas. La ficha de Q-UTIL-27 repite esa premisa errónea; la decisión del Owner no se edita.


---

## AC-UTIL-018.3 — ccompat (JavaConverters, Factory) y compat.Future/PartialFunction (shims de Scala 2.12/2.13)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=documented*

**Contrato observable.** Shims para compilar el mismo código contra Scala 2.12, 2.13 y 3: `util.ccompat` (alias `Factory`, objeto `JavaConverters` con `AsJavaExtensions`/`AsScalaExtensions`, y la anotación `ccompatUsedUntil213`), `compat.Future` (`fold`, `reduce`, `find` sobre `IterableOnce[Future[T]]`/`immutable.Iterable[Future[T]]`, delegando en `Future.foldLeft/reduceLeft/find`) y `compat.PartialFunction` (sólo en 2.12). Sin comportamiento propio observable.

**Fuente.**
- `actor/src/main/scala-2.13+/org/apache/pekko/util/ccompat/package.scala:31-40`, `.../ccompatUsedUntil213.scala`; `scala-2.13+/org/apache/pekko/compat/Future.scala:29-59`; `scala-2.12/.../ccompat/*`, `scala-2.12/.../compat/{Future,PartialFunction}.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/Scala212CompatTest.scala:35-78 (1.7.1)`

**Precondiciones.** Ninguna.

**Comportamiento.** `compat.Future.fold(futures)(zero)(op)` = `Future.foldLeft` sobre una copia en `Iterable` (la sobrecarga `IterableOnce` copia los elementos: "performance implications"); `reduce` = `reduceLeft`; `find` = `Future.find`. `JavaConverters` es sólo un conjunto de extensiones `asJava/asScala`.

**Errores.** Los de `Future`.

**Orden / concurrencia / ciclo de vida.** Sin estado.

**Dependencias.** AC-UTIL-018.1.

**Equivalente Rust propuesto y diferencias observables.** No hay versiones de Scala que reconciliar. Efecto observable a preservar: `fold`/`reduce`/`find` sobre colecciones de futuros (`futures::future::join_all` + `fold`/`try_join_all`, `select_ok`): las combinaciones de `Future.fold`/`reduce`/`find` aparecen como API de `pattern`/`Future` en otras unidades y deben tener sus propios contratos (AC-DSP/ASK). Propuesta: sin tipo propio; la fila queda como shim documentado, **sin exclusión** hasta aprobación del Owner (Q-UTIL-11).

**Aserciones.**
- A1 compilación cruzada de extensiones de `Optional*` y `Iterable.single`: Scala212CompatTest.scala:35-77.
- A2 *no afirmada por spec*: semántica de `compat.Future.fold/reduce/find`.

**Oráculo / validación prevista.** Ninguna para el shim; los consumidores de `fold/reduce/find` se validan en sus filas.

**Decisiones.** Q-UTIL-11.

**Actualización P77 (2026-11-18, Pekko 2.0.0).** `util/ccompat/*` y `compat/Future` están eliminados en 2.0.0 (MiMa `remove-scala-2.12-support.excludes`): eliminado en 2.0.0, no es exclusión (respuesta del Owner a Q-UTIL-11, 2026-11-18). Sin tipo ni caso; `fold/reduce/find` sobre futuros se valida en sus consumidores (AC-DSP-017.2). Cierre pendiente de Q-UTIL-27.

**Cierre 2026-11-28 (Q-UTIL-27 (a), aceptada): ELIMINADO EN 2.0.0, no implementado.** Sin tipos ni funciones en rakka.


---

## AC-UTIL-018.4 — SameThreadExecutionContext (ejecuta el callback en el hilo que completa: 'parasitic')

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `SameThreadExecutionContext()` devuelve un `ExecutionContext` que ejecuta cada tarea **inline en el hilo que la envía** (en Scala 2.13+: `ExecutionContext.parasitic`; en 2.12 una implementación propia con `BatchingExecutor`). Sólo debe crearse desde `ExecutionContexts#parasitic`; los `map`/`flatMap` hechos con él corren en el hilo del `Future` previo.

**Fuente.**
- `actor/src/main/scala-2.13/org/apache/pekko/dispatch/internal/SameThreadExecutionContext.scala:27-29 (1.7.1)`; `scala-3/.../SameThreadExecutionContext.scala`; `scala-2.12/.../SameThreadExecutionContext.scala`
- consumidor: `actor/src/main/scala/org/apache/pekko/dispatch/Future.scala` (`ExecutionContexts.parasitic`), `AbstractDispatcher.scala`, `BatchingExecutor.scala`
- tests: `actor-tests/src/test/scala/org/apache/pekko/dispatch/SameThreadExecutionContextSpec.scala:26-68 (1.7.1)`

**Precondiciones.** Ninguna.

**Comportamiento.** `execute(runnable)` corre `runnable.run()` en el hilo actual; si ya hay un ciclo de ejecución en curso (parasitic de Scala), se encola y se ejecuta al terminar el actual para evitar desbordar la pila.

**Errores.** Una excepción en el `Runnable` se informa al `reportFailure` del contexto.

**Orden / concurrencia / ciclo de vida.** Sin hilos propios; el orden depende de quién completa el `Promise`.

**Dependencias.** AC-DSP-* (`ExecutionContexts.parasitic`).

**Equivalente Rust propuesto y diferencias observables.** Un ejecutor "inline" para los `then` de una cadena de futuros (`futures::executor::block_on` no aplica); en el runtime elegido (Q-X-4) es la continuación del `await`: el `.await` de un `Future` que otro hilo completa corre en el hilo del `waker` sólo si se usa un ejecutor "same-thread"; propuesta: `ExecutionContext::parasitic()` como trait/handle con una cola local por hilo. Diferencia a registrar: en Tokio las continuaciones no corren inline en el hilo del completador.

**Aserciones.**
- A1 en 2.13 la clase es `scala.concurrent.ExecutionContext$parasitic$`: SameThreadExecutionContextSpec.scala:30-38 (no portable).
- A2 una cadena `.map(...)(system.dispatcher).map(...)(SameThreadExecutionContext())` ejecuta ambos pasos en el mismo hilo (`threadName1 == threadName2`): :40-52; lo mismo con `ExecutionContext.global`: :54-66.
- A3 *no afirmada por spec*: relación con `BatchingExecutor`.

**Oráculo / validación prevista.** Test portado de A2 sobre el ejecutor elegido; A1 no se porta.

**Decisiones.** Q-X-4 (runtime), Q-UTIL-11.


---

## AC-UTIL-018.5 — ScalaBatchable (marca de ejecución por lotes)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ScalaBatchable.isBatchable(runnable)` devuelve `true` si el `Runnable` es `pekko.dispatch.Batchable` con `isBatchable == true` o es un `scala.concurrent.Batchable`; `false` en otro caso. Lo usa `BatchingExecutor` para decidir si agrupa tareas. Tres versiones (2.12, 2.13, 3).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/dispatch/internal/ScalaBatchable.scala:23-36`; `scala-3/...`; `scala-2.12/...`
- consumidor: `actor/src/main/scala/org/apache/pekko/dispatch/BatchingExecutor.scala`

**Precondiciones.** Ninguna.

**Comportamiento.** Coincidencia de patrón en este orden: `pekko.dispatch.Batchable` -> `b.isBatchable`; `scala.concurrent.Batchable` -> `true`; otro -> `false`.

**Errores.** Ninguno.

**Orden / concurrencia / ciclo de vida.** Función pura.

**Dependencias.** AC-DSP-* (`BatchingExecutor`, `Batchable`).

**Equivalente Rust propuesto y diferencias observables.** Un trait `Batchable { fn is_batchable(&self) -> bool }` implementado por las tareas internas del despachador; el caso `scala.concurrent.Batchable` desaparece (no hay ejecutor de Scala). Sin test de Pekko.

**Aserciones.**
- A1 *no afirmada por spec*: no hay test directo (`BatchingExecutor` se ejerce en los specs de AC-DSP); caracterización por el patrón citado.

**Oráculo / validación prevista.** Test unitario propio al implementar `BatchingExecutor` (AC-DSP).

**Decisiones.** Q-UTIL-11, Q-X-4.


---

## AC-UTIL-019 — Anotaciones de API: ApiMayChange, DoNotInherit, InternalApi, InternalStableApi

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=pending*

**Contrato observable.** Cuatro anotaciones Java con retención `CLASS` (legibles por MiMa, no en tiempo de ejecución) que documentan la estabilidad de la API: `@ApiMayChange(issue = "")` (evoluciona hacia estable; puede cambiar en una versión de parche sin aviso), `@DoNotInherit` (sólo para tipos; no extender fuera de Pekko), `@InternalApi` (interno, puede cambiar sin aviso; el scaladoc debe empezar por `INTERNAL API`), `@InternalStableApi` (interno pero usado entre módulos de Pekko, no cambiar sin considerar usos externos). Efecto observable: ninguno en tiempo de ejecución; la compatibilidad binaria que protegen la comprueba MiMa (AC-UTIL-021).

**Fuente.**
- `actor/src/main/java/org/apache/pekko/annotation/ApiMayChange.java:18-49`, `DoNotInherit.java:18-36`, `InternalApi.java:18-40`, `InternalStableApi.java:22-42`

**Precondiciones.** Ninguna.

**Comportamiento.** `@Target`: `ApiMayChange`, `InternalApi`, `InternalStableApi` -> método, constructor, campo, tipo, paquete; `DoNotInherit` -> sólo tipo. `@Documented`, `@Retention(CLASS)`.

**Errores.** Ninguno.

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** Ninguna; las usan todos los módulos (miles de usos, p. ej. `@InternalApi` en `ConstantFun`, `Reflect`, `FrequencySketch`).

**Equivalente Rust propuesto y diferencias observables.** Efecto a preservar: **la frontera pública/interna es explícita y se ve en la documentación**. Representación: visibilidad real (`pub(crate)` para `InternalApi`/`InternalStableApi` entre crates de rakka: `#[doc(hidden)] pub` si el uso es entre crates), `#[sealed]`/trait sellado para `DoNotInherit`, y un atributo de documentación (`#[doc = "UNSTABLE: ..."]` o feature `unstable`) para `ApiMayChange`; la compatibilidad se verifica con `cargo semver-checks` en lugar de MiMa. Diferencia: no existe retención por clase; los marcadores no tienen forma de leerse en tiempo de ejecución (tampoco en Pekko).

**Aserciones.**
- A1 *no afirmada por spec*: no existe test; caracterización de los `@Target`/`@Retention` leída de las fuentes.

**Oráculo / validación prevista.** Ninguna; revisión de la API pública (`cargo doc`, `cargo semver-checks`) por el Owner al cerrar cada crate.

**Decisiones.** Q-UTIL-11.


**Actualización P76 (2026-11-18, Pekko 2.0.0).** Las cuatro anotaciones no cambiaron en 2.0.0 (`@Documented`, `@Retention(CLASS)`; `@Target` método/constructor/campo/tipo/paquete salvo `DoNotInherit` sólo tipo). Equivalente aceptado por Q-UTIL-11 (a); `cargo-semver-checks` entra en los gates desde el primer release (AGENTS.md). Cierre pendiente de Q-UTIL-20.

**2026-11-28 (Q-UTIL-20 (a), aceptada): correspondencia documentada; la fila sigue `partial`.** Es una convención para el código de rakka, no un tipo:

| Anotación de Pekko | En rakka | Diferencia |
|---|---|---|
| `@InternalApi` | `pub(crate)`; entre crates, `#[doc(hidden)] pub` | el elemento no aparece en la documentación pública (Pekko lo documenta como `INTERNAL API`) |
| `@InternalStableApi` | `#[doc(hidden)] pub` | la estabilidad entre módulos no la comprueba el compilador |
| `@DoNotInherit` (sólo tipos) | trait sellado (`mod private { pub trait Sealed {} }`) | lo impide el compilador, no sólo lo documenta |
| `@ApiMayChange` | nota en el rustdoc del elemento (o feature `unstable`) | sin marcador legible por herramientas |

La retención `CLASS` (A1: la lee MiMa, no el programa) corresponde a `cargo semver-checks`, que entra en los gates desde el primer release (AC-UTIL-021 sigue abierta hasta entonces).

Falta para cerrar: (1) `util::frequency_sketch`, `util::recency_list` y `util::token_bucket` son módulos públicos sin `#[doc(hidden)]` aunque en Pekko son INTERNAL API (lista no exhaustiva: falta la revisión completa); (2) la revisión de la API pública (`cargo doc`) por el Owner; (3) `cargo semver-checks` en los gates.

---

## AC-UTIL-020 — Acceso Unsafe y offsets de campos (Unsafe.java y clases Abstract*)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** `Unsafe` obtiene la instancia de `sun.misc.Unsafe` por reflexión y la usan 9 clases Java para CAS/lectura-escritura volátil de campos (offsets de `_xxxDoNotCallMeDirectly`) sin `AtomicReferenceFieldUpdater`: `AbstractActorRef` (campos de `RepointableActorRef`: `_cellDoNotCallMeDirectly`, `_lookupDoNotCallMeDirectly`), `actor/dungeon/AbstractActorCell` (cuatro campos de `ActorCell`: `_mailbox`, `_childrenRefs`, `_nextName`, `_functionRefs`, con los nombres mangled `org$apache$pekko$actor$dungeon$Dispatch$$_mailboxDoNotCallMeDirectly` y análogos de `Children`), `AbstractMailbox` (`Mailbox._statusDoNotCallMeDirectly`, `_systemQueueDoNotCallMeDirectly`), `AbstractMessageDispatcher` (`_inhabitantsDoNotCallMeDirectly`, `_shutdownScheduleDoNotCallMeDirectly`), `AbstractNodeQueue` (`_tailDoNotCallMeDirectly`, `Node._nextDoNotCallMeDirectly`), `AbstractBoundedNodeQueue` (`_enqDoNotCallMeDirectly`, `_deqDoNotCallMeDirectly`, `Node._nextDoNotCallMeDirectly`), `AbstractPromiseActorRef` (`PromiseActorRef._stateDoNotCallMeDirectly`, `_watchedByDoNotCallMeDirectly`), `AbstractCircuitBreaker` (`CircuitBreaker._currentStateDoNotCallMeDirectly`, `_currentResetTimeoutDoNotCallMeDirectly`). Además `Unsafe` contiene dos utilidades públicas: `copyUSAsciiStrToBytes(str, bytes)` (copia ASCII con tres algoritmos según la JVM) y `fastHash(str)` (xorshift128+ sobre los caracteres).

**Fuente.**
- `actor/src/main/java/org/apache/pekko/util/Unsafe.java:17-146`
- `actor/src/main/java/org/apache/pekko/actor/AbstractActorRef.java:26-29`, `actor/src/main/java/org/apache/pekko/actor/dungeon/AbstractActorCell.java:30-42`, `dispatch/AbstractMailbox.java:24`, `dispatch/AbstractMessageDispatcher.java:24`, `dispatch/AbstractNodeQueue.java:57-220`, `dispatch/AbstractBoundedNodeQueue.java:48-190`, `pattern/AbstractPromiseActorRef.java:25-28`, `pattern/AbstractCircuitBreaker.java:25-28`
- tests: `actor-tests/src/test/scala/org/apache/pekko/util/AsciiStringCopySpec.scala:22-45`

**Precondiciones.** Una JVM con `sun.misc.Unsafe` (se lanza `ExceptionInInitializerError` si falla, `IllegalStateException("Can't find instance of sun.misc.Unsafe")` si no se localiza).

**Comportamiento observable.**
- `copyUSAsciiStrToBytes(str, bytes)` copia los `str.length()` caracteres de `str` a `bytes[0..]` como bytes ASCII; mismo resultado con cualquiera de los 3 algoritmos para texto ASCII; `AsciiStringCopySpec` fija que el algoritmo 0 funciona siempre, el 1 sólo en JDK >= 9 (en la prueba) y el 2 sólo en JDK 8 (`testUSAsciiStrToBytesAlgorithm2("abc") == false` en 9+).
- `fastHash(str)`: `s0 = 391408`, `s1 = 601258`; por cada carácter `x = s0 ^ (long) ch; y = s1; s0 = y; x ^= x << 23; y ^= y >>> 26; x ^= x >>> 17; s1 = x ^ y`; resultado `(int)(s0 + s1)`. Medido (JDK 11): `fastHash("abc") = 1177643766`, `fastHash("") = 992666`; **para caracteres no ASCII el resultado depende del algoritmo seleccionado** (medido `fastHash("é") = 1914060472`: el algoritmo 1 extiende el signo del byte Latin-1, el 0 usaría `?`).
- En el árbol fijado, **ni `fastHash` ni `copyUSAsciiStrToBytes` tienen consumidores en producción** (búsqueda en `actor/src/main` y `actor-typed/src/main`); sólo el spec.
- Las clases `Abstract*` ofrecen operaciones atómicas sobre campos: CAS, `getObjectVolatile`, `putOrderedObject`, `putObjectVolatile`.

**Errores.** `ExceptionInInitializerError`/`IllegalStateException` de arranque.

**Orden / concurrencia / ciclo de vida.** Los offsets se obtienen en un bloque estático; los accesos tienen semántica volátil/ordenada (release/acquire).

**Dependencias.** AC-CELL-*, AC-REF-*, AC-MBX-*, AC-DSP-*, AC-ASK-*(circuit breaker), AC-SCH-*: cada fila dueña de su estructura define qué atómico necesita.

**Equivalente Rust propuesto y diferencias observables.** `std::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, AtomicBool}` y `arc-swap`/`crossbeam` para punteros compartidos; ningún `unsafe` de offsets. Efecto preservado: la atomicidad (CAS y orden de memoria) de cada campo. `fastHash` si algún día se necesita: portable con aritmética `wrapping`, sólo sobre ASCII (la dependencia del algoritmo para no ASCII se declara como diferencia). `copyUSAsciiStrToBytes` -> `str.as_bytes()` (UTF-8 = ASCII).

**Aserciones.**
- A1 algoritmo 0 funciona siempre; algoritmos 1/2 según versión de Java: AsciiStringCopySpec.scala:25-37 (no portable).
- A2 `copyUSAsciiStrToBytes("abcdefghijklmnopqrstuvxyz")` produce los mismos bytes en US-ASCII: :39-45.
- A3 `fastHash("abcdefghijklmnopqrstuvxyz")` no lanza (no afirma valor): :47-52.
- A4 *no afirmada por spec*: valores de `fastHash` (medidos arriba), semántica de los atómicos (se prueba por los consumidores).

**Oráculo / validación prevista.** Diferencial con jar para `fastHash` (sólo ASCII); lo demás se valida en las unidades dueñas.

**Decisiones.** Q-UTIL-11 (representación por atómicos de Rust; no exclusión).


---

**Actualización P06 (2026-11-18, Pekko 2.0.0).** Oráculo (la clase `Unsafe` es pública, se llama directo): `fastHash` de texto ASCII y `copyUSAsciiStrToBytes` de texto ASCII coinciden (UN-01 uc-0119, UN-02 uc-0120). Sin `--add-opens` la JVM elige el algoritmo 0: para texto no ASCII `fastHash` usa `String.getBytes(US_ASCII)` (`é` y `€` dan el mismo valor, `a\u00e9b` otro) y un carácter fuera del BMP lanza `ArrayIndexOutOfBoundsException` porque la cadena de bytes es más corta que `length()` (UN-03, medido, sin grabar). Con el algoritmo 1 (JDK con acceso a `String.value`) los bytes Latin-1 se extienden con signo. Rust: `fast_hash(&str) -> Option<i32>` sólo para ASCII; el resto es la propuesta Q-UTIL-19. Los atómicos de las clases `Abstract*` no se portan aquí: cada fila dueña usa `std::sync::atomic` (Q-UTIL-11 (a)).

## AC-UTIL-021 — Filtros MiMa (compatibilidad binaria)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `actor/src/main/mima-filters/<versión>.backwards.excludes/*.excludes` listan excepciones a las comprobaciones de compatibilidad binaria de MiMa en el build de Pekko. En el árbol fijado existen 3 archivos: `1.0.x.../bytestring-inputstream.excludes` (`ReversedMissingMethodProblem` para `ByteString.asInputStream`), `1.2.x.../bytestring-indexOf-overload.excludes` (`ReversedMissingMethodProblem` para `ByteString.indexOf`), `1.3.x.../actorsystem-close.excludes` (cierre de `ActorSystem`). Sin efecto en tiempo de ejecución.

**Fuente.**
- `actor/src/main/mima-filters/1.0.x.backwards.excludes/bytestring-inputstream.excludes`, `1.2.x.backwards.excludes/bytestring-indexOf-overload.excludes`, `1.3.x.backwards.excludes/actorsystem-close.excludes`

**Precondiciones.** Ninguna. **Comportamiento.** Ninguno en runtime. **Errores.** Ninguno. **Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-UTIL-006.5 (`asInputStream`), AC-UTIL-006.7 (`indexOf` con `from`/`to`), AC-SYS-* (`ActorSystem.close`).

**Equivalente Rust propuesto y diferencias observables.** Efecto a preservar: **la API pública no rompe compatibilidad sin que se note**. Representación: `cargo semver-checks` en CI (y `#[non_exhaustive]` en enums/estructuras públicas), no archivos de filtros. Lo que sí es contrato son los **métodos añadidos** que los filtros documentan (`ByteString.asInputStream`, `indexOf(elem, from, to)`, `ActorSystem.close`), que ya están contenidos en sus filas.

**Aserciones.**
- A1 *no afirmada por spec*: no hay test; las 3 entradas son de configuración de build.

**Oráculo / validación prevista.** Ninguna (fase de CI).

**Decisiones.** Q-UTIL-11.


---

## AC-UTIL-022 — UnsynchronizedByteArrayInputStream, ByteStringInputStream y ByteBufferCleaner

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `UnsynchronizedByteArrayInputStream` (copiada de commons-io): `InputStream` sobre un array con `mark/reset` y sin sincronización. `ByteBufferCleaner`: libera explícitamente `ByteBuffer`s directos (`clean`, `isSupported`). Los adaptadores de `ByteString` están en AC-UTIL-006.5.

**Fuente.**
- `actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java:17-212`
- `actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java:17-123`
- tests: `actor-tests/src/test/scala/org/apache/pekko/io/UnsynchronizedByteArrayInputStreamSpec.scala:28-255`, `ByteBufferCleanerSpec.scala:25-34`; mima: `actor/src/main/mima-filters/1.0.x.backwards.excludes/bytestring-inputstream.excludes`

**Precondiciones.** `data` no nulo (NPE con el nombre `"data"`); `offset` y `length` no negativos (`IllegalArgumentException("<nombre> cannot be negative")`).

**Comportamiento (`UnsynchronizedByteArrayInputStream`).**
- Constructor `(data, offset, length)`: la posición inicial es `min(offset, data.length)` (si `data` vacío, `min(offset, offset)`), el fin `min(posInicial + length, data.length)` (medido: `([1,2,3], 5, 2)` -> `available 0`, `read -1`; `([1,2,3], 1, 5)` -> `available 2`).
- `read()` -> byte sin signo o `-1`; `read(dest, off, len)`: valida `0 <= off`, `0 <= len`, `off + len <= dest.length` con `IndexOutOfBoundsException("Range [off, off + len) out of bounds for length <n>")` (medido), `len == 0` -> 0, agotado -> `-1`, si no copia `min(len, restante)`.
- `available() = max(0, eod - offset)`. `markSupported() == true`; `mark(n)` guarda la posición (el límite se ignora); `reset()` vuelve a la marca.
- `skip(n)`: `n < 0` -> `IllegalArgumentException("Skipping backward is not supported")`; devuelve `min(n, restante)` pero **avanza `offset` en `n` completo** (`Math.addExact(offset, Math.toIntExact(n))`): medido `skip(50)` sobre 3 bytes devuelve 3, `available` 0 y `read -1`; `skip(2^40)` lanza `ArithmeticException: integer overflow`.

**Comportamiento (`ByteBufferCleaner`).** Al inicializar elige `Java8Cleaner` (`sun.misc.Cleaner`) o `Java9Cleaner` (`Unsafe.invokeCleaner`), prueba el cleaner con un buffer directo de 1 byte y, si falla, lo desactiva imprimiendo un aviso en `stderr`. `isSupported` = cleaner disponible. `clean(buffer)` libera el buffer; si falla lanza `IllegalStateException("Failed to clean direct buffer.", causa)`.

**Errores.** Ver arriba.

**Orden / concurrencia / ciclo de vida.** No thread-safe (la clase lo declara). `ByteBufferCleaner` se inicializa una vez por carga de clase.

**Dependencias.** AC-UTIL-006.5; consumidores de `ByteBufferCleaner`: AC-IO (buffer pools de TCP/UDP, fuera de este grupo).

**Equivalente Rust propuesto y diferencias observables.** `Cursor<&[u8]>` cubre `read/seek/mark-reset` (usando `position`/`set_position`), pero hay que reproducir `skip` que avanza más allá del fin y devuelve el recorte; propuesta: tipo propio pequeño. `ByteBufferCleaner` no tiene equivalente: en Rust los buffers se liberan por `Drop` de forma determinista (efecto observable preservado: el buffer se libera; no hay llamada explícita). `isSupported` = siempre `true`. Documentar como "n/a por diseño del lenguaje" requiere aprobación del Owner (Q-UTIL-11).

**Aserciones.**
- A1 `markSupported == true`; `read() == 'a'`; tras `mark(1)` y leer `'b'`, `reset()` permite releer `'b'` (el límite de `mark` se ignora): UnsynchronizedByteArrayInputStreamSpec.scala:25-35.
- A2 `skip(1) == 1` y luego `read() == 'b'`: :37-42; `skip(50) == 3` y `read() == -1`: :44-49.
- A3 `ByteBufferCleaner.isSupported == true` en un runtime que lo soporta y `clean(buffer directo)` no lanza: ByteBufferCleanerSpec.scala:28-35.
- A4 *no afirmada por spec*: `skip` negativo, desbordes, ctor con `offset > length`, texto de `IndexOutOfBounds`; caracterización medida (arriba).

**Oráculo / validación prevista.** Test portado de A1/A2; diferencial con jar para A4 (medido ad hoc); A3 no aplica.

**Decisiones.** Q-UTIL-11, Q-UTIL-12.


**Actualización P04 (2026-11-18, Pekko 2.0.0).** `UnsynchronizedByteArrayInputStream.skip(n)` ya no avanza el `n` completo: avanza `min(n, restante)` y devuelve ese número (`skip(2^40)` no lanza `ArithmeticException`; ST-03), `available` nunca es negativo, y el constructor con rango usa aritmética long; nuevos `readAllBytes`, `readNBytes`, `skipNBytes` (EOFException) y `transferTo` (ST-06). Texto de `read` con rango inválido: `Range [off, off + len) out of bounds for length n`. `ByteBufferCleaner` es privada del paquete y no se mide; Q-UTIL-25 propuesta; fila `partial`. Casos uc-0530..uc-0536.

---

## AC-UTIL-023 — Helpers.makePattern (patrón glob `?`/`*` a java.util.regex.Pattern)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle*

**Contrato observable.** Convierte un patrón glob en regex anclada; es la definición histórica de `?`/`*` en actor selection (hoy reemplazada por Glob).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:46-47`
- `actor/src/main/scala/org/apache/pekko/util/Glob.scala:25-37 (documenta la equivalencia)`

**Condiciones.**
- `?` -> `.` (un carácter excepto terminadores de línea), `*` -> `.*` (cualquier secuencia sin terminadores de línea), el resto entre \Q y \E.
- Hallazgo: `.` en Java no coincide con `\n`, `\r`, U+0085, U+2028, U+2029; Glob.matches sí: con entrada que contiene saltos de línea ambos difieren (GlobSpec sólo compara con {a,b}).

**Orden / concurrencia / ciclo de vida.** n/a: pura.

**Dependencias.** AC-UTIL-016.6.

**Equivalente Rust propuesto y diferencias observables.** No se porta `Pattern`; si se exige la misma semántica de saltos de línea, `glob_matches` debe replicar o documentar (Q-UTIL-8).

**Aserciones.**
- A1 equivalencia con Glob para patrones/entradas cortos sobre {a,b,*,?}: GlobSpec.scala:60-73.
- A2 comportamiento con saltos de línea: NO afirmada por spec; medido con el jar: `makePattern("a?b")` rechaza `"a\nb"` y `"a\u0085b"`, `makePattern("a*b")` rechaza `"a\nb"`, mientras `Glob.matches("a?b", "a\nb")` y `("a*b", "a\nb")` dan `true`; con un par sustituto `"a\uD83D\uDE42b"` `makePattern("a?b")` da `true` (el regex trabaja por punto de código) y `Glob.matches("a?b", ...)` da `false` (`Glob.matches("a??b", ...)` da `true`).

**Oráculo.** Posible con jar pekko-actor (Helpers.makePattern es público).

**Decisiones.** Q-UTIL-8


---

**Actualización P06 (2026-11-18, Pekko 2.0.0).** Q-UTIL-8 (c) autoriza no reproducir `makePattern` bit a bit. A1: MP-EXH (uc-0121) graba el resultado de `makePattern` para los 341 patrones de 0-4 caracteres sobre {a,b,*,?} contra las 31 entradas de 0-4 sobre {a,b}; Rust responde con `glob_matches` y coincide (igual que GL-EXH). A2 medida con el oráculo 2.0.0 (MP-NL, sin grabar): `makePattern("a?b")` rechaza `a\nb`, `a\u0085b` y `a\u2028b` mientras Glob los acepta; `makePattern("a?b")` acepta `a` + U+1F642 + `b` (punto de código) y Glob no (`"a??b"` sí); `makePattern("?")` rechaza `\r` y `makePattern("*")` rechaza `\r\n`. Es la diferencia autorizada: la referencia es Glob.matches (ActorSelection.scala:369 decide).

## AC-UTIL-024 — Helpers.compareIdentityHash / identityHashComparator (orden por identityHashCode sin overflow)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle*

**Contrato observable.** Ordena objetos por identidad de forma consistente (sin overflow) para usar en estructuras ordenadas (BalancingDispatcher).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:49-71`
- `actor/src/main/scala/org/apache/pekko/dispatch/BalancingDispatcher.scala:67` (único uso en el árbol: `new ConcurrentSkipListSet[ActorCell](Helpers.identityHashComparator(...))`, leído en el cierre documental)

**Condiciones.**
- `diff = (hash(a) & 0xFFFFFFFFL) - (hash(b) & 0xFFFFFFFFL)`; resultado 1, -1 o 0.
- `identityHashComparator(comp)`: si compareIdentityHash == 0 y `a != b` -> `comp.compare(a,b)`; si no el resultado de identidad.

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-DSP (BalancingDispatcher).

**Equivalente Rust propuesto y diferencias observables.** Función que compara por id/dirección; el orden concreto no es observable salvo consistencia (Q-UTIL-10).

**Aserciones.**
- A1 orden consistente sin ciclos: derivada del comentario y código Helpers.scala:48-54 (*no afirmada por spec*). **Corrección del cierre documental**: el contrato decía "sin test en upstream"; `EventBusSpec.scala:309,345` usan `Helpers.compareIdentityHash(a, b)` como `compareSubscribers` de `ScanningClassification`, pero esos tests afirman el comportamiento del bus (publicar/suscribir), no el signo ni el valor de la comparación.

**Oráculo.** No aplica (identityHashCode no es reproducible).

**Decisiones.** Q-UTIL-10


---

**Actualización P06 (2026-11-18, Pekko 2.0.0).** Q-UTIL-10 (a): la identidad es la dirección del referente. Rust: `util::helpers::compare_identity_hash(&T, &T) -> Ordering` (dirección sin signo) e `identity_hash_comparator(comp)` (rama `case 0 if a != b`: sólo cuando son iguales por identidad y no por `==`, p. ej. NaN). Oráculo 2.0.0: invariantes (ID-01 uc-0105, ID-02 uc-0106). El valor concreto del orden no es observable.

## AC-UTIL-025 — Helpers.currentTimeMillisToUTCString y Helpers.timestamp (formato de hora para logs)

*unidad UTIL · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Dos formateadores de marcas de tiempo públicos de Helpers.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:73-115`

**Condiciones.**
- `currentTimeMillisToUTCString`: `timeOfDay = ts % 86400000`; horas, minutos y segundos con cero a la izquierda (`if (x < 10) append(0)`), ms con 3 dígitos; sufijo `UTC` (líneas 80-105).
- Timestamp negativo: el módulo es negativo y el formato incluye `-` (medido con el jar: `currentTimeMillisToUTCString(-1) = "00:00:00.00-1UTC"`; `(0) = "00:00:00.000UTC"`; `(86399999) = "23:59:59.999UTC"`; sin test en upstream).
- `timestamp`: patrón `MM/dd/yyyy HH:mm:ss.SSS` con `ZoneId.systemDefault()` capturado al cargar el objeto (líneas 108-113).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** Ninguna.

**Equivalente Rust propuesto y diferencias observables.** Funciones libres; `timestamp` usa zona local del proceso. Diferencia: la zona de Rust sin dependencias requiere API del SO (Q-UTIL-10).

**Aserciones.**
- A1 `currentTimeMillisToUTCString(System.currentTimeMillis)` es igual a `"%02d:%02d:%02d.%03dUTC"` de la hora UTC calculada con un `GregorianCalendar`: LoggerSpec.scala:283-293 ("Ticket 3080").
- A2 relleno con ceros: `2019-05-05T00:00:00.000Z` -> `"00:00:00.000UTC"` y `...T23:59:59.999Z` -> `"23:59:59.999UTC"`: LoggerSpec.scala:295-316. **Corrección del cierre documental**: el contrato decía "sin tests en upstream"; sí los hay, y existe un consumidor en `actor/src/main/scala/org/apache/pekko/event/Logging.scala:1009` (`StdOutLogger.timestamp`, que usa `Helpers.timestamp`).
- A3 `timestamp` patrón y zona: derivada del código Helpers.scala:108-113 (*no afirmada por spec*).

**Oráculo.** Posible con jar pekko-actor (comparar salida con zona UTC fijada).

**Decisiones.** Q-UTIL-10


---

**Actualización P06 (2026-11-18, Pekko 2.0.0).** `current_time_millis_to_utc_string(i64)` coincide con el oráculo (TS-UTC-01/02, uc-0101/0102), incluido el `-` por campo para negativos (`-1` -> `00:00:00.00-1UTC`, `-1000` -> `00:00:0-1.000UTC`, `-86399999` -> `0-23:0-59:0-59.00-999UTC`) y `Long.MaxValue` -> `07:12:55.807UTC`. `timestamp`: el oráculo fija la zona `GMT+05:30` (propiedad `user.timezone` antes de cargar Helpers, que la captura) para que el caso no dependa del host; `timestamp_at_utc_offset(ms, segundos)` coincide (TS-LOCAL-01/02, uc-0103/0104). El año es el `yyyy` de Java (año de la era: el año 0 sale `0001`, y más de 9999 lleva `+`: `01/01/+10000`, `08/17/+292278994` para Long.MAX). Pendiente: la zona por defecto del sistema (Q-UTIL-18).

## AC-UTIL-026 — Helpers.isWindows (detección de SO por os.name)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle*

**Contrato observable.** Booleano de plataforma usado para ajustar el scheduler y IO.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:44`
- `LightArrayRevolverScheduler.scala:67,120`
- `Tcp.scala:640`
- `DnsSettings.scala:113`

**Condiciones.**
- `os.name` en minúsculas con 'win' como subcadena: 'Windows 10' sí; 'Mac OS X' no; 'Darwin' (valor de os.name en algunos JVM de macOS) sí.
- Efectos: `tick-duration` mínimo 10 ms (Windows) frente a 1 ms; sleep redondeado a 10 ms (LightArrayRevolverScheduler.scala:120).

**Orden / concurrencia / ciclo de vida.** Se evalúa una vez al cargar Helpers.

**Dependencias.** AC-SCH-008, AC-IO-003, AC-IO-008, AC-UTIL-002.2.

**Equivalente Rust propuesto y diferencias observables.** `const IS_WINDOWS: bool = cfg!(windows)`; diferencia sólo en JVM que informen un os.name con 'win' no Windows (Q-UTIL-13).

**Aserciones.**
- A1 derivada del código Helpers.scala:42 (*no afirmada por spec*); medido con el jar en macOS (JDK 11): `os.name = "Mac OS X"` y `Helpers.isWindows == false`. El caso `Darwin` sigue siendo teórico (HotSpot no lo informa).

**Oráculo.** Posible con jar pekko-actor (leer Helpers.isWindows en jshell).

**Decisiones.** Q-UTIL-13


---

**Actualización P06 (2026-11-18, Pekko 2.0.0).** Q-UTIL-13 (a): `util::helpers::IS_WINDOWS = cfg!(windows)`. WIN-01 (uc-0100) en macOS: `isWindows=false`.

## AC-UTIL-027 — Clock / NanoClock / ScheduledClock (reloj interno basado en nanoTime)

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Extensión `Clock` (`private[pekko]`, pública en el bytecode) con `currentTime(): Long` y `earlierTime(duration: FiniteDuration): Long`. `Clock.createExtension(system)` lee `pekko.scheduled-clock-interval` (reference.conf:980, por defecto `1 s`; `0` significa llamar a `System.nanoTime` en cada llamada): si el intervalo es > 0 crea un `ScheduledClock(interval, system.scheduler, system.dispatcher)`, si no un `NanoClock`. Los valores tienen el origen arbitrario y la vuelta de `System.nanoTime`: sólo las diferencias significan algo.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/Clock.scala:36-67` (objeto, trait y `NanoClock`), `:77-125` (`ScheduledClock`)
- `actor/src/main/resources/reference.conf:978-980`
- tests: ninguno en upstream (`grep Clock actor-tests` sólo da usos de otros tipos).

**Condiciones.**
- `NanoClock.currentTime = System.nanoTime()`; `earlierTime(d) = currentTime() - d.toNanos` (aritmética con vuelta; `d` puede ser negativo).
- `ScheduledClock`: `time` (AtomicLong) y `updatedTime` parten de `System.nanoTime()`; `maxIncrement = max(updateInterval.toNanos - 100000, 0)`; en construcción llama `scheduler.scheduleWithFixedDelay(updateInterval, updateInterval){ update() }`.
- `update()`: `now = nanoTime`; `newTime = if (now - current >= 0) now else current` (la diferencia cubre la vuelta de Long.MaxValue); CAS sobre `time` (si una llamada concurrente a `currentTime` lo cambió, reintenta); al lograrlo `updatedTime = newTime` y `maxIncrementReached = false`.
- `currentTime()`: si `maxIncrementReached` devuelve `time.get()`; si no `now = time.incrementAndGet()` y, si `now - updatedTime >= maxIncrement`, `maxIncrementReached = true`; devuelve `now`.
- `earlierTime(d) = time.get() - d.toNanos` (no incrementa).

**Orden / concurrencia / ciclo de vida.** `time` es atómico; `updatedTime` y `maxIncrementReached` son `@volatile`. La tarea del scheduler y los llamantes concurrentes nunca hacen retroceder `time`.

**Dependencias.** AC-EXT (Extension, ExtensionId), AC-SCH-008.3 (`scheduleWithFixedDelay`), AC-CFG (clave de configuración).

**Equivalente Rust propuesto y diferencias observables.** `util::clock`: trait `Clock`, `NanoClock`, `ScheduledClock::new(intervalo, &dyn FixedDelayScheduler)` (la tarea de Pekko pasa a ser un cierre que el scheduler real programará) y `SelectedClock::for_interval` (la lógica de `createExtension`). El tiempo es `i64` con `wrapping_*`; `earlier_time` recibe nanosegundos con signo como un `FiniteDuration`. El origen de `NanoClock` es el primer uso en el proceso.

**Aserciones.**
- A1 `NanoClock.currentTime` no retrocede y `earlierTime(d) + d` cae entre dos lecturas (también con `d` negativo o enorme): CK-01 (uc-0107).
- A2 `ScheduledClock`: con intervalo 100003 ns, `currentTime` da +1, +2, +3 y se queda en +3 con `maxIncrementReached = true`; `earlierTime(0) = +3`, `earlierTime(5) = -2`; la tarea se programa con retardo inicial igual al intervalo: CK-02 (uc-0108).
- A3 bordes de `maxIncrement` (intervalos 1, 100000, 100001, 100002, 100004 ns; con 1 ms, 900000 incrementos y luego se queda): CK-03 (uc-0109).
- A4 tras `update()` el tiempo no retrocede, `maxIncrementReached` vuelve a `false` y el siguiente `currentTime` es +1: CK-04 (uc-0110).
- A5 selección: sin clave y con `1 ms`/`1 s` -> `ScheduledClock`; `0`/`0 s` -> `NanoClock` (ActorSystem real): CK-05 (uc-0111).
- A6 *no afirmada por spec ni observable con el reloj real*: `update` no retrocede si `nanoTime` quedó detrás de los incrementos, y vuelta a negativo: `tests/platform_spec.rs` con una fuente de tiempo manual.

**Oráculo.** `PlatformOracle` (familia util-core): `NanoClock` y `ScheduledClock` se construyen directo (públicos en el bytecode) con una `Scheduler` falsa que captura la tarea, y CK-05 usa un `ActorSystem` con el log apagado.

**Decisiones.** Q-UTIL-11.

**Actualización P06 (2026-11-18).** Falta para cerrar: registrar `Clock` como extensión (AC-EXT), leer la clave (AC-CFG) y un `FixedDelayScheduler` sobre el scheduler real, con cancelación (AC-SCH-008.3).

## AC-UTIL-028 — SWARUtil (lectura/escritura multibyte y patrones SWAR sobre byte[])

*unidad UTIL · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle*

**Contrato observable.** Utilidades de bajo nivel de `ByteString` (`private[pekko]`, públicas en el bytecode), copiadas del proyecto Netty (Apache-2.0, `Copyright 2024 The Netty Project`): `compilePattern(byte)` repite el byte en los 8 carriles de un `Long` (`(b & 0xFF) * 0x0101010101010101`); `applyPattern(word, pattern)` marca con el bit alto cada byte de `word` igual al de `pattern` (`input = word ^ pattern; tmp = (input & 0x7F..) + 0x7F..; ~(tmp | input | 0x7F..)`); `getIndex(word)` = `numberOfLeadingZeros(word) >>> 3` (8 si no hay marca; sólo big endian); `getLastIndex(word)` = -1 si `word == 0`, si no `(63 - numberOfTrailingZeros(word)) >>> 3`; `getLong/getInt/getShort(array, index, byteOrder)` y `putInt/putShort/putLong(array, index, value, byteOrder)` (`putShort` escribe los 16 bits bajos de un `Int`); variantes `...WithoutMethodHandle` por desplazamientos y `getShortBE/LE`, `getIntBE/LE`, `getLongBE/LE` por bytes sueltos.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/util/SWARUtil.scala:64-318` (API) y `:319-457` (variantes de respaldo)
- tests: ninguno en upstream.

**Condiciones.**
- Las lecturas y escrituras usan `MethodHandles.byteArrayViewVarHandle`; si la JVM no lo ofrece, las variantes por desplazamientos. No comprueban límites (lo hace la JVM con una excepción de índice).
- Cada lectura reconstruye el valor con signo del tipo (Long, Int, Short).

**Orden / concurrencia / ciclo de vida.** Funciones puras sobre el arreglo del llamador.

**Dependencias.** AC-UTIL-022 (ByteString los usa).

**Equivalente Rust propuesto y diferencias observables.** `util::swar`: `i64`/`i32`/`i16` como los tipos de la JVM, `u8` por `Byte`, `ByteOrder::{BigEndian, LittleEndian}`; una sola implementación (`from_be_bytes`...), porque las dos rutas de Pekko dan lo mismo. Un índice fuera del arreglo entra en pánico (Pekko: `IndexOutOfBoundsException`; el contrato lo deja a cargo del llamador).

**Aserciones.**
- A1 `compilePattern` de los 256 bytes: SW-01 (uc-0112).
- A2 `applyPattern`: 35 pares fijos (palabras con bytes 00/01/7F/80/FF) y 400 aleatorios con el byte plantado en un carril: SW-02 (uc-0113); además `tests/platform_spec.rs` comprueba con 20000 palabras que marca exactamente los bytes iguales.
- A3 `getIndex`/`getLastIndex` de las 256 combinaciones de marcas y de palabras con otros bits: SW-03 (uc-0114).
- A4 `getLong/getInt/getShort` en todos los desplazamientos y ambos órdenes: SW-04 (uc-0115).
- A5 `putLong/putInt/putShort` (incluido el recorte a 16 bits) en ambos órdenes: SW-05 (uc-0116).
- A6 variantes sin MethodHandle iguales a las de VarHandle: SW-06 (uc-0117).
- A7 variantes por bytes sueltos: SW-07 (uc-0118).
- A8 *no afirmada*: índices fuera de rango (ver arriba).

**Oráculo.** `PlatformOracle` llama `SWARUtil$.MODULE$` directo.

**Decisiones.** Q-UTIL-11. Aviso de Netty: encabezado de `crates/rakka-actor/src/util/swar.rs` y NOTICE (ya incluye a Netty).

## AC-CFG-001 — Carga de configuración: ConfigFactory.load + reference.conf como fallback; config explícita por ActorSystem(name, config)

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Un `ActorSystem` sin config explícita usa `ConfigFactory.load(classLoader)` (application.conf/json/properties + system properties + reference.conf); con `config` explícita usa ésa tal cual. En ambos casos las Settings leen `appConfig.withFallback(ConfigFactory.defaultReference(classLoader))` (reference.conf de todos los jars). Los defaults nunca viven en código: sólo en reference.conf.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:277-284 (apply(name, setup): config explícita o ConfigFactory.load(cl))`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:322-327`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:847-859 (ActorSystemImpl.settings: applicationConfig.withFallback(defaultReference))`
- `docs/src/main/paradox/general/configuration.md:18-53`
- `actor/src/main/resources/reference.conf:1-1495 (el archivo completo, ver Cobertura)`

**Condiciones.**
- Sin config: `ConfigFactory.load(cl)`; precedencia: system properties > application.conf > reference.conf (configuration.md:39-43; `config.resource` redefine el nombre).
- Con config: la config dada es la capa de aplicación; reference.conf actúa como fallback.
- Config incompleta (sin `pekko.*` requerido) no falla en la mezcla, falla en `checkValid` (AC-CFG-019) o al leer una clave ausente (ConfigException.Missing).
- Classloader: `ActorSystem.findClassLoader()` = `Reflect.findClassLoader()` (AC-UTIL-016.7).

**Orden / concurrencia / ciclo de vida.** Resolución de sustituciones después de combinar capas: un override de la aplicación se ve en los valores derivados (golden FB-05); la carga ocurre una vez, en el constructor de `ActorSystemImpl`, antes de crear Settings.

**Dependencias.** AC-UTIL-002.4 (lectores de duración), AC-CFG-017 (formato HOCON), AC-CFG-019 (checkValid). Externas: Typesafe Config 1.4.9 (project/Dependencies.scala:66).

**Equivalente Rust propuesto y diferencias observables.** `Config` inmutable con `with_fallback`/`resolve`; `ConfigLoader::load()` (fuentes: Q-CFG-6) + `reference.conf` embebido con `include_str!` + `version.conf` generado en build. Diferencia: sin system properties de JVM ni classpath scanning (Q-CFG-6).

**Aserciones.**
- A1 `ConfigSpec` usa `ConfigFactory.defaultReference(classLoader)` como config del sistema y todas las claves de abajo tienen su default: ConfigSpec.scala:30-110.
- A2 `parseString(...).withFallback(testConf)` da prioridad al string: ActorSystemSpec.scala:336-345.
- A3 la config derivada se resuelve tras combinar capas: golden FB-05 y contraejemplo FB-06; fallback de reference FB-08.
- A4 `pekko.library-extensions += "X"` en la app conserva el default y agrega: ExtensionSpec.scala:157-160 (REF-02 del golden).
- A5 la regla 'sin config explícita -> load' no la afirma un test aislado: derivada del código ActorSystem.scala:281.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-6.


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `Config::parse_string`, `with_fallback`, `resolve`, `default_reference` (reference.conf embebido) y `Config::load(ruta)` (Q-CFG-6 (a): application.conf configurable sobre el reference embebido). Casos grabados KEY (308 claves de los reference.conf de actor y actor-typed) y SUB/FB/REF. Queda `partial`: Q-CFG-14 (includes con classpath) y Q-CFG-16 (variables con prefijo) propuestas; Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida; y `Config::load` (A5) no tiene caso grabado contra el oráculo.

**Actualización 2026-11-28 (Q-CFG-15 (a), aceptada con cambios): reparto de la verificación de archivos entre esta fila y AC-UTIL-002.5, sin eliminar criterios.** Esta fila es dueña del origen de archivo **al cargar y en los errores**; el render es de AC-UTIL-002.5 (A3).
- A6 un valor leído de un archivo de aplicación conserva su origen, y los errores sobre él (getters, resolución, `check_valid`) lo dicen como Typesafe Config 1.4.9 (`<ruta>: <línea>`): **pendiente** (hoy `String: <línea>`; sin medir). Cubre la parte de archivo que las notas de AC-CFG-017.* y AC-CFG-019 describen como "la ruta de archivo no está medida"; esas filas la siguen exigiendo.
- El origen de texto está en casos grabados; el de variable de entorno en errores sigue pendiente en las filas de getters (AC-CFG-017.2-.5, 017.7 y 019).

## AC-CFG-002 — ActorSystem.Settings: lectura fail-fast de todas las claves pekko.* al crear el sistema

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ActorSystem.Settings` lee de una vez, al construirse, todas las claves `pekko.*` que usa el núcleo y las expone como campos (`final val`). Cualquier clave ausente/ilegible lanza `ConfigException` y aborta la creación del sistema.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:387-493 (clase Settings)`
- `actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:34-184`
- `actor/src/main/resources/reference.conf:13-146`

**Condiciones.**
- Campos y defaults verificados: ver aserciones. Campos extra sin test: ProviderSelectionType/ProviderClass/HasCluster (AC-CFG-008), SupervisorStrategyClass, CreationTimeout (20s), LoggersDispatcher, LogConfigOnStart, Home, SchedulerClass.
- `setup` (ActorSystemSetup) es accesible: `settings.setup`.
- Errores: `ConfigException.Missing`/`WrongType`/`BadValue` de Typesafe según la clave (golden TYPE-*).
- Efecto lateral: ninguno salvo las excepciones de AC-CFG-003 y AC-CFG-020.

**Orden / concurrencia / ciclo de vida.** Los `final val` se evalúan en orden textual dentro del constructor: primero `config` (checkValid), luego `ConfigVersion`, provider, timeouts, serialización, logging, debug, scheduler, JVM, coordinated shutdown, `DefaultVirtualNodesFactor`, y al final el chequeo de versión (líneas 395-487).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4, AC-UTIL-002.2 (toRootLowerCase), AC-UTIL-001 (Timeout).

**Equivalente Rust propuesto y diferencias observables.** `struct Settings` con `Settings::from_config(&Config) -> Result<Settings, ConfigError>`; campos con tipos de dominio (`LogDeadLetters`, `Timeout`, `Duration` o `Infinite`).

**Aserciones.**
- A1 `pekko.version` == `ActorSystem.Version` y `ConfigVersion`: ConfigSpec.scala:41-42.
- A2 `daemonic=false`; `serialize-messages=false`; `NoSerializationVerificationNeededClassPrefix == Set("org.apache.pekko.")`: ConfigSpec.scala:44-49,56-57.
- A3 scheduler: `ticks-per-wheel=512`, `tick-duration=10 ms`, `error-on-tick-duration-verification-failed=true`, `implementation=org.apache.pekko.actor.LightArrayRevolverScheduler`: :51-54.
- A4 `jvm-exit-on-fatal-error=true`, `jvm-shutdown-hooks=true`, `fail-mixed-versions=true`: :59-64.
- A5 `virtual-nodes-factor=10`, `unstarted-push-timeout=10s`: :66-70.
- A6 loggers: un elemento `org.apache.pekko.event.Logging$DefaultLogger`; `logger-startup-timeout=5s`; `logging-filter=org.apache.pekko.event.DefaultLoggingFilter`: :72-79.
- A7 `log-dead-letters=10`, `log-dead-letters-during-shutdown=false`, `log-dead-letters-suspend-duration=5 min`: :81-88.
- A8 `coordinated-shutdown.terminate-actor-system=true`, `run-by-actor-system-terminate=true`; `allow-java-serialization=false`: :90-97.
- A9 default-dispatcher (type, executor, shutdown-timeout=1s, throughput=5, throughput-deadline-time=0ms, attempt-teamwork=true), default-executor.fallback, fork-join-executor, thread-pool-executor y default-mailbox: :100-182 (pertenecen a AC-DSP/AC-MBX; se verifican con el mismo test).
- A10 sección debug: las 7 claves `false` y los campos AddLoggingReceive, DebugAutoReceive, DebugLifecycle, FsmDebugEvent, DebugEventStream, DebugUnhandledMessage, DebugRouterMisconfiguration: :144-168.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-CFG-3, Q-CFG-4.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `config::Settings::new(name, Config)` (`config/settings.rs`): `check_valid` contra `Config::default_reference()` sin `pekko.actor.internal-dispatcher` (AC-CFG-019), lectura de las claves en el orden del constructor Scala y primer error; el resultado de `Settings::new` coincide con `new ActorSystem.Settings(..)` de Pekko 2.0.0 en 213 casos grabados `cs-*` (SET-DEFAULT, SET-A9, SET-PROVIDER, SET-SCHEDULER, SET-VNF, SET-VERSION, SET-INVALID): A1, A2, A4..A8 y A10 y los valores por defecto de A9; los de A3 (scheduler) los dan los casos KEY de P07. Queda `partial`: Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error con `String: N` y a `toString`; Q-CFG-18 y Q-CFG-19 (propuestas): texto de `expecting: object with keys [..]` y origen de una clave ausente sin grabar, y reference contra el que se valida; `ActorSystem::create_with_settings` aún no recibe estos `Settings` (AC-SYS); `ProviderSelection` es AC-CFG-008; A9 pertenece a AC-DSP/AC-MBX.

## AC-CFG-003 — Chequeo pekko.version vs versión del binario

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Si `pekko.version` de la config difiere de la versión del binario (`ActorSystem.Version`, generada en build), el sistema no arranca.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:409,484-486`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:183 (Version = pekko.Version.current, archivo generado)`
- `actor/src/main/resources/reference.conf:10-11 (include "version")`
- `project/VersionGenerator.scala:18-25 (genera version.conf y Version.scala)`

**Condiciones.**
- Error: `ConfigurationException("Pekko JAR version [<Version>] does not match the provided config version [<ConfigVersion>]")` (ActorSystem.scala:485-487).
- Una config de aplicación con otro `pekko.version` provoca el error (no hay test; derivada del código).
- Es la última verificación del constructor de Settings.

**Orden / concurrencia / ciclo de vida.** Se evalúa tras leer todas las claves (líneas 485-487).

**Dependencias.** AC-CFG-002, AC-UTIL-004.2; AC-SYS-020 (chequeo de módulos mezclados con ManifestInfo).

**Equivalente Rust propuesto y diferencias observables.** Constante `VERSION` del crate y `pekko.version` en el `reference.conf` embebido (Q-CFG-5).

**Aserciones.**
- A1 `getString("pekko.version") == ActorSystem.Version` en la config por defecto: ConfigSpec.scala:41-42.
- A2 el mensaje de error y la condición de fallo: derivadas del código ActorSystem.scala:485-487 (*no afirmadas por spec*).

**Oráculo.** No aplica: la versión es del build.

**Decisiones.** Q-CFG-5.


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `config::check_pekko_version` (mensaje `Pekko JAR version [..] does not match the provided config version [..]`; `pekko.version` = 2.0.0 por Q-CFG-5 (a)) probado con tests unitarios; sin caso grabado (el oráculo no crea un ActorSystem aquí). Queda `partial`: sin caso grabado contra el oráculo (la condición del build no es observable en él); sólo tests unitarios.

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::new` comprueba la versión como último paso (`check_pekko_version`, después de la combinación de coordinated-shutdown y de `virtual-nodes-factor`); casos grabados cs-* SET-VERSION (cinco con `ConfigurationException` o la versión correcta, sin descripción de origen) y SET-DEFAULT (A1) contra `new ActorSystem.Settings(..)` de Pekko 2.0.0: A1 y A2 cubiertas. No cambia de estado (`partial`): `ActorSystem::create_with_settings` aún no construye estos `Settings` (AC-SYS).

## AC-CFG-004 — Claves de logging: loggers, logging-filter, loggers-dispatcher, logger-startup-timeout, loglevel, stdout-loglevel, log-config-on-start

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Claves de logging que el ActorSystem lee al arrancar: `loggers`, `logging-filter`, `loggers-dispatcher`, `logger-startup-timeout`, `loglevel`, `stdout-loglevel`, `log-config-on-start`.

**Fuente.**
- `actor/src/main/resources/reference.conf:17-50`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:433-439`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:342-377 (amendSlf4jConfig; AC-CFG-018)`

**Condiciones.**
- Defaults: loggers=["org.apache.pekko.event.Logging$DefaultLogger"], logging-filter="org.apache.pekko.event.DefaultLoggingFilter", loggers-dispatcher="pekko.actor.default-dispatcher", logger-startup-timeout=5s, loglevel="INFO", stdout-loglevel="WARNING", log-config-on-start=off.
- Niveles válidos: OFF, ERROR, WARNING, INFO, DEBUG (reference.conf:40,45); el parseo del nombre lo hace Logging (AC-EVT).
- Con loglevel=OFF/`loggers=[]` no se escribe nada a stdout (LoggerSpec.scala:209-215).
- Los loggers se crean sincrónicamente en el arranque; `logger-startup-timeout` acota la espera (reference.conf:33-36).

**Orden / concurrencia / ciclo de vida.** Durante el arranque se usa `stdout-loglevel` hasta que los loggers configurados arrancan (reference.conf:38-46); al terminar el sistema la cola de logs se drena (LoggerSpec.scala:189-205).

**Dependencias.** AC-EVT-011 (loggers), AC-EVT-016 (LoggingFilter), AC-CFG-018, AC-UTIL-001.

**Equivalente Rust propuesto y diferencias observables.** Campos de Settings; los FQCN de logger/filter se resuelven por registro (Q-X-2, Q-X-7).

**Aserciones.**
- A1 defaults de loggers, timeout y filtro: ConfigSpec.scala:72-79.
- A2 sistema con configuración normal escribe a stdout: LoggerSpec.scala:184-187.
- A3 `system.terminate` drena la cola de logs en orden (msg1..msg3): LoggerSpec.scala:189-205.
- A4 sistema con logging apagado no escribe: LoggerSpec.scala:211-214.
- A5 varios loggers reciben el mismo evento: LoggerSpec.scala:219-235 (el spec continúa hasta ~240; leído hasta 235).
- A6 `stdout-loglevel` y `log-config-on-start` sin test directo: derivadas del código ActorSystem.scala:435,440.

**Oráculo.** No aplica: salida de logs; se porta LoggerSpec.

**Decisiones.** Q-X-2, Q-X-7, Q-CFG-10.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: campos `log_level`, `stdout_log_level`, `loggers`, `loggers_dispatcher`, `logging_filter`, `logger_start_timeout`, `log_config_on_start` de `Settings`; casos grabados cs-* SET-DEFAULT, SET-LOG, SET-TO logger-startup-timeout (A1, A6). Queda `partial`: A2..A5 (LoggerSpec) necesitan los loggers como actores (AC-EVT-011/016, Q-X-7), sin implementar; Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error grabados (`String: N`).

## AC-CFG-005 — Claves de dead letters: log-dead-letters (on/off/N), log-dead-letters-during-shutdown, log-dead-letters-suspend-duration

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `log-dead-letters` (on|off|true|false|N), `log-dead-letters-during-shutdown` y `log-dead-letters-suspend-duration` (infinite o duración) controlan cuántos dead letters se loguean y cuándo se reanuda el log.

**Fuente.**
- `actor/src/main/resources/reference.conf:52-68`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:440-452`

**Condiciones.**
- `LogDeadLetters`: `toRootLowerCase(getString)`: `off`/`false` -> 0; `on`/`true` -> Int.MaxValue; otro -> `getInt` (un valor no numérico lanza ConfigException; ActorSystem.scala:441-445). Default 10.
- `LogDeadLettersSuspendDuration`: `infinite` (minúsculas) -> `Duration.Inf`; otro -> `getMillisDuration`. Default 5 minutes (ActorSystem.scala:447-453).
- Mensajes observables (consumidor AC-EVT-009): `Message [<tipo>] from <remitente> to <destino> was not delivered. [<n>] dead letters encountered` (DeadLetterSuspensionSpec.scala:61-62), `... was dropped. <razón>. [<n>] dead letters encountered` (:64-65), `... was unhandled. [<n>] dead letters encountered` (:67-68); al llegar al tope: sufijo `, no more dead letters will be logged in next` (:81); al reanudar: `, of which 2 were not logged` y el conteo se reinicia (:93,99).

**Orden / concurrencia / ciclo de vida.** La suspensión dura `suspend-duration`; tras reanudar, el siguiente log lleva el conteo de no registrados y luego el contador vuelve a 1 (DeadLetterSuspensionSpec.scala:88-101).

**Dependencias.** AC-EVT-009, AC-UTIL-002.2, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** `enum LogDeadLetters { Off, Count(u32), Always }` y `enum SuspendDuration { Infinite, After(Duration) }`.

**Aserciones.**
- A1 default `log-dead-letters=10`, durante shutdown `false`, suspend `5 min`: ConfigSpec.scala:81-88.
- A2 con `log-dead-letters=4` y suspend 2s: el 1.º, 2.º (Dropped) y 3.º (unhandled) se loguean con conteo; el 4.º lleva `, no more dead letters will be logged in next`; 5.º y 6.º no se loguean; tras 2s el 7.º se loguea con `, of which 2 were not logged`; el siguiente reinicia en 1: DeadLetterSuspensionSpec.scala:70-103.
- A3 `on/off/true/false` e `infinite` (insensible a mayúsculas): derivadas del código ActorSystem.scala:441-451 (*no afirmadas por spec*).

**Oráculo.** No aplica (logs); se porta DeadLetterSuspensionSpec.

**Decisiones.** Q-CFG-4.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::log_dead_letters` (i32 de Pekko: `off`/`false` 0, `on`/`true` `i32::MAX`, otro `getInt`), `log_dead_letters_during_shutdown` y `SuspendDuration::{Infinite, Finite}`; casos grabados cs-* SET-LDL (23), SET-LDSD (25), SET-LDDS (8): A1 y A3. Hallazgo: `log-dead-letters = true`/`false` sin comillas no pasa `checkValid` (el reference tiene un número: `ValidationFailed`); `on`/`off` y `"true"` sí. Queda `partial`: A2 (DeadLetterSuspensionSpec) necesita el log de dead letters (AC-EVT-009); Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error grabados.

## AC-CFG-006 — Claves de extensiones: library-extensions y extensions

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `library-extensions` (extensiones de librería, acumulables con `+=`) y `extensions` (de usuario) listan FQCN de extensiones que se cargan al arrancar.

**Fuente.**
- `actor/src/main/resources/reference.conf:70-83`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1261-1291`

**Condiciones.**
- Defaults: library-extensions = `${?pekko.library-extensions} ["org.apache.pekko.serialization.SerializationExtension$"]`; extensions = [].
- Las de `library-extensions` fallan el arranque si no cargan (throwOnLoadFail=true); las de `extensions` sólo se loguean (AC-EXT-003).

**Orden / concurrencia / ciclo de vida.** Primero library-extensions, luego extensions, en orden de lista (ActorSystem.scala:1258-1259).

**Dependencias.** AC-EXT-003, AC-CFG-016, AC-CFG-017 (sustitución `${?x}`).

**Equivalente Rust propuesto y diferencias observables.** Listas de nombres resueltas por el registro de extensiones.

**Aserciones.**
- A1 `extensions` carga TestExtension al arrancar (`hasExtension`): ExtensionSpec.scala:70-77.
- A2 la lista de library-extensions contiene `InstanceCountingExtension` una vez y se crea una vez: ExtensionSpec.scala:122-133.
- A3 duplicados en la lista (3 entradas) -> 1 sola instancia: ExtensionSpec.scala:135-151.
- A4 una extensión que falla en `extensions` no tumba el sistema si no se encuentra (loguea): ExtensionSpec.scala:111-120; si falla al iniciar, el sistema falla: :99-109.
- A5 falla de library-extension (al iniciar o inexistente) aborta el sistema: ExtensionSpec.scala:153-172.

**Oráculo.** No aplica.

**Decisiones.** Q-X-2, Q-CFG-8.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::extension_names` (biblioteca primero, luego las del usuario, sin el `$` de objeto Scala y sin repetidos, Q-CFG-8 (b)) con tests unitarios; casos grabados cs-* SET-EXT (las listas que Pekko lee de `pekko.library-extensions` y `pekko.extensions`, incluido `+=`). Queda `partial`: A1..A5 (ExtensionSpec) necesitan el cargador de extensiones (AC-EXT-003) y el registro de fábricas (Q-X-2), sin implementar; Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a un texto de error grabado.

## AC-CFG-007 — Claves JVM: daemonic, jvm-exit-on-fatal-error, jvm-shutdown-hooks, fail-mixed-versions, java-flight-recorder.enabled, home

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Claves ligadas a la JVM: `home`, `daemonic`, `jvm-exit-on-fatal-error`, `jvm-shutdown-hooks`, `fail-mixed-versions`, `java-flight-recorder.enabled`.

**Fuente.**
- `actor/src/main/resources/reference.conf:14-15,85-110`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:462-471`

**Condiciones.**
- Defaults: home="" (-> `Home = None`), daemonic=off, jvm-exit-on-fatal-error=on, jvm-shutdown-hooks=on, fail-mixed-versions=on, java-flight-recorder.enabled=true.
- `jvm-shutdown-hooks=off` desactiva los hooks de JVM que instala Pekko (CoordinatedShutdown), no los de usuario (`addCancellableJvmShutdownHook`) (reference.conf:92-98).
- Test: con `jvm-shutdown-hooks=off` y `run-by-jvm-shutdown-hook=on` sólo queda 1 hook (el del usuario); con ambos on quedan 2: CoordinatedShutdownSpec.scala:620-645.

**Orden / concurrencia / ciclo de vida.** Los hooks se registran al crear la extensión CoordinatedShutdown (AC-CS-007).

**Dependencias.** AC-SYS-015, AC-CS-007, AC-UTIL-016.11 (FlightRecorderLoader), AC-UTIL-016.9 (ManifestInfo para fail-mixed-versions).

**Equivalente Rust propuesto y diferencias observables.** Settings: `daemonic` (hilos del runtime), `jvm_exit_on_fatal_error` (-> `process::abort`/exit en panic), `jvm_shutdown_hooks` (-> manejadores de SIGTERM/Ctrl-C), `fail_mixed_versions`/`java_flight_recorder` aceptadas e ignoradas con warning (Q-CFG-7).

**Aserciones.**
- A1 `daemonic=false`, `jvm-exit-on-fatal-error=true`, `jvm-shutdown-hooks=true`, `fail-mixed-versions=true`: ConfigSpec.scala:44,56-64.
- A2 hooks de JVM con `jvm-shutdown-hooks=off`: CoordinatedShutdownSpec.scala:632-645 (el conteo 1 vs 2).
- A3 `home` y `java-flight-recorder`: derivadas del código ActorSystem.scala:463-466 (*no afirmadas por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-CFG-7.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::{daemonicity, jvm_exit_on_fatal_error, jvm_shutdown_hooks, fail_mixed_versions, home}` aceptadas y guardadas sin efecto (Q-CFG-7 (a)); casos grabados cs-* SET-JVM y SET-HOME: A1 y A3 (`home`). `java-flight-recorder.enabled` no lo lee `Settings` en Pekko (lo lee FlightRecorderLoader, AC-UTIL-016.11). Queda `partial`: A2 (hooks de JVM, CoordinatedShutdownSpec) pertenece a AC-CS-007; Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error grabados.

## AC-CFG-008 — pekko.actor.provider y ProviderSelection (local | remote | cluster | FQCN)

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `pekko.actor.provider` (local | remote | cluster | FQCN) o `BootstrapSetup.withActorRefProvider` selecciona el proveedor de referencias; `ProviderSelection` lo representa.

**Fuente.**
- `actor/src/main/resources/reference.conf:112-118`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:102-144 (ProviderSelection)`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:411-421 (Settings.ProviderSelectionType, ProviderClass, HasCluster)`

**Condiciones.**
- `ProviderSelection("local")` = Local (fqcn = LocalActorRefProvider, hasCluster=false); `"remote"`/`org.apache.pekko.remote.RemoteActorRefProvider` = Remote (hasCluster=false); `"cluster"`/`org.apache.pekko.cluster.ClusterActorRefProvider` = Cluster (hasCluster=true); otra cadena = Custom(fqcn) (hasCluster=false) (ActorSystem.scala:137-144).
- Un `BootstrapSetup.actorRefProvider` tiene prioridad sobre la config (ActorSystem.scala:410-414).
- Remote/Cluster requieren clases que no están en pekko-actor: si faltan, falla la carga por reflexión (ClassNotFoundException); los Custom se validan con dynamicAccess.

**Orden / concurrencia / ciclo de vida.** La selección se resuelve en Settings, antes de crear el proveedor.

**Dependencias.** AC-SYS-018, AC-CFG-016, AC-CFG-015.

**Equivalente Rust propuesto y diferencias observables.** `enum ProviderSelection { Local, Remote, Cluster, Custom(String) }` (sólo Local ejecutable); Remote/Cluster/Custom no resueltos -> error de configuración explícito (Q-CFG-9).

**Aserciones.**
- A1 Local: fqcn == `LocalActorRefProvider`, hasCluster false, Settings.ProviderClass igual: ProviderSelectionSpec.scala:38-43.
- A2 Remote: fqcn == RemoteActorRefProvider; `ProviderSelection("remote") == ProviderSelection(RemoteActorRefProvider)`: :45-51.
- A3 Cluster: hasCluster true; equivalencia nombre corto/FQCN: :53-59.
- A4 Custom("other.ActorRefProvider"): fqcn sin cambios y hasCluster false: :61-67.

**Oráculo.** `oracle/run.sh provider-selection-all|provider-selection-case|provider-selection-ids` (`ProviderSelectionOracle.java`, familia `provider-selection`, casos `ps-*`, 25): construye `new ActorSystem.Settings(..)` de Pekko 2.0.0 con cada valor de `pekko.actor.provider` y lee `ProviderSelectionType`, `ProviderClass` y `HasCluster`. SEL-01..13 (A1..A4: `local`, `remote`, `cluster`, los nombres de clase de remote y cluster, `other.ActorRefProvider`, el nombre de clase de Local, mayúsculas, espacios, cadena vacía y un número), EQ-01..06 (`ProviderSelection("remote") == ProviderSelection(RemoteActorRefProvider)` y su par de cluster, A2 y A3) y SEL-14..19 (un valor que no es texto: una lista, una lista vacía y un objeto fallan con `WrongType` desde `getString`, `null` con `ConfigException.Null`, y un booleano o un decimal se leen como su texto). Además, `oracle/run.sh dynamic-access-case` PRV-05..10 (familia `dynamic-access`, casos `da-0021..da-0026`): el proveedor que `ActorSystem` carga por nombre corto o por otra cadena (`ActorSystem.scala:996-1011`).

**Decisiones.** Q-CFG-9, Q-X-2.


---

**Actualización P09 (2026-11-19, Pekko 2.0.0).** Rust: `config::ProviderSelection` (`Local`, `Remote`, `Cluster`, `Custom(String)`; `from_identifier`, `identifier`, `fqcn`, `has_cluster`; `config/provider_selection.rs`) y `Settings::provider_selection`, `provider_class()` y `has_cluster()` (`Settings::new` lee `pekko.actor.provider` justo después de `pekko.version`, como el constructor Scala). 25 casos `ps-0001..ps-0025` y 6 casos `da-0021..da-0026` grabados contra Pekko 2.0.0: A1, A2, A3 y A4 cubiertas; un nombre que no es exactamente `local`, `remote` o `cluster` (o la clase de remote o cluster) es `Custom`, también el nombre de clase de Local, sin plegar mayúsculas ni recortar. Queda `partial` por (1) Q-CFG-31 (propuesta): el error de un proveedor no registrado es `ClassNotFoundException` (lo da `FactoryRegistry`, AC-CFG-016) y no `ConfigurationException`, como dicen Q-X-2 y Q-CFG-9 (a); (2) AC-SYS-018 (P25): el sistema aún no crea el proveedor, así que "crear el sistema con Remote/Cluster/Custom no registrado falla" solo se verifica con el registro; Q-CFG-9 (c) (proveedores registrados por el usuario) llega con el registro de fábricas del sistema; (3) AC-CFG-015 (PR 3): la prioridad de `BootstrapSetup.actorRefProvider` sobre la configuración (`ActorSystem.scala:410-414`).

**Actualización P09 PR 3 (2026-11-19, Pekko 2.0.0).** La prioridad de `BootstrapSetup.actorRefProvider` sobre la configuración (`ActorSystem.scala:410-414`) queda cubierta por los casos `as-0032..as-0041` (PR-01..10, AC-CFG-015): `Settings::with_setup` toma el `identifier` de la selección del setup. AC-CFG-008 sigue `partial` por Q-CFG-31, Q-CFG-34 (propuesta: un `Custom` del setup acaba como la clase `custom`), AC-SYS-018 y Q-CFG-9 (c); ya no depende de AC-CFG-015.

## AC-CFG-009 — guardian-supervisor-strategy, creation-timeout, unstarted-push-timeout, typed.timeout

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `guardian-supervisor-strategy` (FQCN de un SupervisorStrategyConfigurator), `creation-timeout`, `unstarted-push-timeout` y `typed.timeout`.

**Fuente.**
- `actor/src/main/resources/reference.conf:120-146`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:423-425`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:519 (instancia el configurador)`
- `actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala:222 (UnstartedPushTimeout)`
- `actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:715 (1.7.1) (typed.timeout)`

**Condiciones.**
- Defaults: guardian-supervisor-strategy="org.apache.pekko.actor.DefaultSupervisorStrategy", creation-timeout=20s, unstarted-push-timeout=10s, typed.timeout=5s.
- Con `StoppingSupervisorStrategy` el guardián de usuario para al hijo que falla (ActorSystemSpec.scala:336-357); el configurador se crea con `createInstanceFor[SupervisorStrategyConfigurator](SupervisorStrategyClass, EmptyImmutableSeq)`.
- `creation-timeout` acota la espera de extensiones en construcción (mensaje de AC-EXT-001).

**Orden / concurrencia / ciclo de vida.** El configurador se instancia al crear el guardián de usuario.

**Dependencias.** AC-SUP-014, AC-REF-020, AC-EXT-001, AC-CFG-016.

**Equivalente Rust propuesto y diferencias observables.** Nombre del configurador -> fábrica en el registro (Q-X-2).

**Aserciones.**
- A1 `unstarted-push-timeout=10s`: ConfigSpec.scala:69-70.
- A2 con `guardian-supervisor-strategy=...StoppingSupervisorStrategy`, `a ! "die"` termina al hijo `a` (watch recibe `Terminated(a)(existenceConfirmed=true, addressTerminated=false)`): ActorSystemSpec.scala:336-357.
- A3 `creation-timeout=20s` y `typed.timeout=5s`: sin test; derivadas del reference.conf:127,151.

**Oráculo.** No aplica.

**Decisiones.** Q-X-2.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::{supervisor_strategy_class, creation_timeout, unstarted_push_timeout}` (el nombre del configurador se resuelve por registro, Q-X-2); casos grabados cs-* SET-GUARDIAN y SET-TO (A1, A3). ⚠ `pekko.actor.typed.timeout` no existe en 2.0.0 (TypedActor se eliminó: ni `reference.conf` ni los casos KEY de P07 la tienen), la parte `typed.timeout` del contrato no aplica. Queda `partial`: A2 (StoppingSupervisorStrategy en el guardián, ActorSystemSpec) necesita el sistema con guardianes configurables (AC-SUP-014, AC-REF-020); Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error grabados.

## AC-CFG-010 — serialize-messages, serialize-creators, no-serialization-verification-needed-class-prefix

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Modos de prueba: `serialize-messages` (serializar y deserializar cada mensaje no primitivo al enviarlo), `serialize-creators` (comprobar que los argumentos de `Props` son serializables) y el prefijo `no-serialization-verification-needed-class-prefix`.

**Fuente.**
- `actor/src/main/resources/reference.conf:129-140`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:428-431`
- `actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:172-224`
- `actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala:283-307`

**Condiciones.**
- Defaults: ambas `off`; prefijo ["org.apache.pekko."].
- Mensajes: se saltan los `NoSerializationVerificationNeeded` y las clases con el prefijo; un `DeadLetter` se verifica su mensaje interior; fallo -> `SerializationCheckFailedException(msg, e)`; si el serializador es DisabledJavaSerializer y no hay que avisar, se omite (Dispatch.scala:180-222).
- Creators: sólo si `serialize-creators` y el Props no es de sistema ni `scope == LocalScope`; cada argumento no null/no marcado se serializa y deserializa; fallo -> `IllegalArgumentException("pre-creation serialization check failed at [<path>/<name>]", cause)` (Children.scala:286-306).
- Mensaje observado: `Failed to serialize and deserialize message of type java.lang.Object` (SerializeSpec.scala:343-344).

**Orden / concurrencia / ciclo de vida.** Mensajes: se verifica en `sendMessage` antes de despachar; creators: antes de crear el hijo.

**Dependencias.** AC-SER-001, AC-SER-003, AC-REF-0xx (Props.args).

**Equivalente Rust propuesto y diferencias observables.** Chequeo opcional en `tell` si hay serializador registrado para el tipo; en Rust la mayoría de mensajes `Box<dyn Any + Send>` no tendrán serializador (Q-X-3, Q-CFG-11).

**Aserciones.**
- A1 con `serialize-messages=on` y prefijo vacío, un mensaje no serializable provoca una excepción con `NoSerializationVerificationNeeded` en el texto: DispatchSpec.scala:29-41.
- A2 `SerializeAllCreators` y `SerializeAllMessages` activos en el sistema de prueba: SerializeSpec.scala:311-314.
- A3 creators: actores con args serializables se crean; `Props(classOf[NonSerializableActor], new AnyRef)` -> IllegalArgumentException: SerializeSpec.scala:316-327.
- A4 args con prefijo `org.apache.pekko.` no se verifican: SerializeSpec.scala:329-337.
- A5 mensajes: `pigdog` pasa; `new AnyRef` -> SerializationCheckFailedException con texto `Failed to serialize and deserialize message of type java.lang.Object`: SerializeSpec.scala:339-349.
- A6 mensajes con el prefijo del sistema no se verifican: SerializeSpec.scala:351-360.

**Oráculo.** No aplica.

**Decisiones.** Q-CFG-11, Q-X-3.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::{allow_java_serialization, serialize_all_messages, serialize_all_creators, no_serialization_verification_needed_class_prefix}`; casos grabados cs-* SET-SER. Q-CFG-11 (aceptada con cambios): con `serialize-messages` o `serialize-creators` en `on`, `Settings::new` falla con `ConfigurationException` (Pekko las acepta: los tres casos SET-SER `on` del oráculo no se comparan, quedan en `PENDING` con su motivo; tests unitarios); el texto del error es Q-CFG-17 (propuesta). Queda `partial`: A1..A6 (DispatchSpec, SerializeSpec) necesitan AC-SER; Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error grabados.

## AC-CFG-011 — pekko.actor.debug: receive, autoreceive, lifecycle, fsm, event-stream, unhandled, router-misconfiguration

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `pekko.actor.debug.*` (siete banderas, todas off) activan eventos Debug: `receive` (LoggingReceive), `autoreceive`, `lifecycle`, `fsm`, `event-stream`, `unhandled`, `router-misconfiguration`.

**Fuente.**
- `actor/src/main/resources/reference.conf:783-806`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:454-460`
- `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:588,674,718`
- `actor/src/main/scala/org/apache/pekko/event/LoggingReceive.scala:66`
- `actor/src/main/scala/org/apache/pekko/event/Logging.scala:164`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:677`
- `actor/src/main/scala/org/apache/pekko/actor/FSM.scala:969`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:984 (EventStream(this, DebugEventStream))`

**Condiciones.**
- `receive=on`: `LoggingReceive` envuelve `receive` y emite `Logging.Debug(path, clase, "received handled message <m> from <sender>")` o `received unhandled message ...` (LoggingReceiveSpec.scala:96-119, :90-92).
- `autoreceive=on`: `received AutoReceiveMessage Envelope(PoisonPill...` (LoggingReceiveSpec.scala:186-202).
- `lifecycle=on`: `started (<clase>...)`, `now supervising <ref>`, `now watched by`, `no longer watched by`, `restarting`, `stopping`, `restarted` (LoggingReceiveSpec.scala:206-277; FaultHandling.scala:99,206,270,290; DeathWatch.scala:189,206).
- `fsm`, `event-stream`, `unhandled` (UnhandledMessage a Debug en Logging.scala:154), `router-misconfiguration` (ActorRefProvider.scala:670): sin test leído.

**Orden / concurrencia / ciclo de vida.** Cada bandera se lee una vez en Settings; los eventos se publican en el EventStream.

**Dependencias.** AC-EVT-019, AC-CELL-*, AC-DW-*, AC-FSM-*, AC-RT-*.

**Equivalente Rust propuesto y diferencias observables.** Campos booleanos de Settings; los chequeos viven en cada componente (`tracing::debug!` o eventos del EventStream según Q-X-6).

**Aserciones.**
- A1 siete banderas `false`: ConfigSpec.scala:144-168.
- A2 `LoggingReceive` agrega el log de recepción cuando se pide: LoggingReceiveSpec.scala:96-128.
- A3 `AutoReceiveMessage`: LoggingReceiveSpec.scala:186-204.
- A4 lifecycle: `now supervising`, `started`: LoggingReceiveSpec.scala:206-228,254-277.
- A5 DeathWatch: `now watched by`/`no longer watched by`: LoggingReceiveSpec.scala:230-252.
- A6 `fsm`, `event-stream`, `router-misconfiguration`: no leídas (marcar al portar sus filas).

**Oráculo.** No aplica.

**Decisiones.** Q-X-6, Q-X-7.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: los siete campos de depuración de `Settings` (`add_logging_receive`, `debug_auto_receive`, `debug_lifecycle`, `fsm_debug_event`, `debug_event_stream`, `debug_unhandled_message`, `debug_router_misconfiguration`); casos grabados cs-* SET-DEBUG (una bandera por caso y las siete): A1. Queda `partial`: A2..A5 (LoggingReceiveSpec) y A6 viven en los componentes que emiten los eventos (AC-EVT-019, AC-CELL, AC-DW, AC-FSM), sin implementar; Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a un texto de error grabado.

## AC-CFG-012.1 — Parseo de pekko.actor.deployment: Config por entrada con fallback a `default`, dispatcher y mailbox

*unidad CFG · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Cada subsección de `pekko.actor.deployment` (menos `default`) se convierte en un `Deploy` con `config = entrada.withFallback(default)`, `dispatcher`/`mailbox` leídos de la config con default "" y `scope = NoScopeGiven`.

**Fuente.**
- `actor/src/main/resources/reference.conf:166-226`
- `actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:229-250,270-276`

**Condiciones.**
- Claves y defaults de `default`: dispatcher="", mailbox="", router="from-code", nr-of-instances=1, within=5 seconds, virtual-nodes-factor=10, tail-chopping-router.interval=10 milliseconds, routees.paths=[].
- Entradas por defecto del archivo: `/IO-DNS/inet-address` (mailbox=unbounded, router=consistent-hashing-pool, nr-of-instances=4), `/IO-DNS/inet-address/*` (dispatcher=default-blocking-io-dispatcher), `/IO-DNS/async-dns` (mailbox=unbounded, router=round-robin-pool, nr-of-instances=1) (reference.conf:349-363).
- Error: `nr-of-instances = boom` -> `ConfigException.WrongType` al crear el sistema (DeployerSpec.scala:144-161).

**Orden / concurrencia / ciclo de vida.** Se parsea una vez al construir el Deployer; en el arranque los Deploy se insertan por `deploy(d)`.

**Dependencias.** AC-CFG-013, AC-CFG-014.

**Equivalente Rust propuesto y diferencias observables.** `Deployer` con tabla `WildcardIndex<Deploy>`; `Deploy.config` es un `Config`.

**Aserciones.**
- A1 `/service1` vacío -> Deploy("/service1", config, NoRouter, NoScopeGiven, NoDispatcherGiven, NoMailboxGiven): DeployerSpec.scala:93-106.
- A2 `/undefined` -> None: :108-112.
- A3 dispatcher `my-dispatcher`: :114-127. A4 mailbox `my-mailbox`: :129-142.
- A5 `nr-of-instances=boom` -> WrongType: :144-161.

**Oráculo.** Sí: `oracle/run.sh deployer-settings-ids|deployer-settings-all|deployer-settings-case` (`DeployerSettingsOracle.java`, 40 ids: 35 `DS` de paridad y 5 `CO` de claves en colisión; el Rust produce 38). Para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-CFG-4, Q-CFG-36 (propuesta).

**Actualización P10 PR 3 (2026-11-20).** Rust: `Deployer::from_settings(&Settings)` y `DeployerError` (`actor/deployer.rs`). Se leen todas las entradas (objetos distintos de `default`) antes de desplegar la primera, como el `flatMap` estricto de Pekko: un error de configuración gana a un path inválido (DS-24). Cada entrada es `entrada.withFallback(default)`; `dispatcher` y `mailbox` salen de esa configuración; el scope es `NoScopeGiven`; una entrada que no es objeto se ignora. Un router distinto de `from-code` conserva solo el nombre configurado (`RouterConfig::Opaque`): type-mapping, pools, validaciones (incluida A5, `nr-of-instances = boom`) y los errores de construcción de routers son de AC-CFG-012.3/P37. Claves que dan el mismo path: orden alfabético, gana la última, política **temporal** autorizada por el Owner (Q-CFG-36, propuesta): Pekko usa el orden de un `HashSet` de entradas de Java que depende de clave y valor; CO-04 y CO-05 divergen y están pendientes en el gemelo Rust. Sigue `partial`: falta la integración con `ActorSystem`/provider (AC-SYS-016) y los routers reales (P37).


---

## AC-CFG-012.2 — Deployer.lookup/deploy: búsqueda por path con comodines (`*`, `prefix*`, `**` terminal)

*unidad CFG · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** `lookup(path)` descarta el primer elemento (el guardián), busca en el `WildcardIndex` y devuelve el Deploy cuyo `path` es el patrón insertado; `deploy(d)` inserta con CAS.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:252-268`
- `actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala:19-114`

**Condiciones.**
- `deploy(d)` separa `d.path.split("/").drop(1)`; cada elemento no vacío se valida con `ActorPath.validatePathElement(el, fullPath = d.path)`; elemento vacío -> `InvalidActorNameException("Actor name in deployment [<path>] must not be empty")` (Deployer.scala:260).
- Error de caracteres: `/gul/ubåt` -> InvalidActorNameException cuyo mensaje incluye `[ubåt]` y `[/gul/ubåt]` (DeployerSpec.scala:163-182).
- Comodines `*` (un elemento) y `**` terminal (DeployerSpec.scala:216-229); `/double` no coincide con `/double/**`.

**Orden / concurrencia / ciclo de vida.** `deployments` es un `AtomicReference[WildcardIndex]`; las altas reintentan ante carrera (compareAndSet).

**Dependencias.** AC-UTIL-009, AC-PATH-008.

**Equivalente Rust propuesto y diferencias observables.** Mismo diseño con `ArcSwap`/`Mutex`; errores `InvalidActorName`.

**Aserciones.**
- A1 `/some/*` y `/*/some`: DeployerSpec.scala:216-222.
- A2 `/double/**` y el más específico `/double/more/**`; `/double` sin match: :224-229.
- A3 path inválido (`ubåt`) lanza InvalidActorNameException con ambos fragmentos: :163-182.
- A4 mensaje de elemento vacío: derivada del código Deployer.scala:260 (*no afirmada por spec*).

**Oráculo.** Sí: `oracle/run.sh deployer-ids|deployer-all|deployer-case` (`DeployerOracle.java`, 60 ids: 58 guiones `DL` de `deployer.tsv` y 2 de concurrencia `DC` con barrera). Ejecuta el `Deployer` real de Pekko 2.0.0 con su índice vaciado por reflexión (el constructor ya despliega las entradas `/IO-DNS/*` de `reference.conf`, que es AC-CFG-012.1). Un sustituto UTF-16 suelto en una ruta está fuera del dominio `&str` (Q-X-17): ningún guion lo usa (la pareja de sustitutos sí, DL-40).

**Actualización P10 PR 2 (2026-11-20).** Rust: `actor::Deployer` (`actor/deployer.rs`) con `deploy`, `lookup` y `lookup_path`. `deploy` divide con la semántica de `String.split("/")` de Java (se conserva el vacío inicial, se pierden los finales, `"/"` no da ningún elemento), descarta el primero, rechaza un elemento vacío y valida los demás antes de tocar el índice; el índice es una instantánea inmutable y el alta es el `compareAndSet` literal de Pekko (se construye el índice nuevo desde la instantánea y se intercambia solo si sigue vigente; si no, se reintenta), con un `Mutex` que guarda solo el intercambio: no se pierde ninguna alta y `lookup` no espera. Sigue `partial`: falta AC-CFG-012.1 (leer `pekko.actor.deployment`) y AC-UTIL-009 sigue `partial` (Q-UTIL-21, orden entre dos `prefix*` que coinciden, aceptada (c) el 2026-11-27).

**Decisiones.** Q-UTIL-8, Q-UTIL-21 (aceptada).


---

## AC-CFG-012.3 — Creación del RouterConfig: type-mapping, FQCN, resizer legado, reintento con DynamicAccess

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `router = from-code` -> `NoRouter`; otro valor se traduce con `router.type-mapping` o se toma como FQCN y se instancia con `(Config)` y, si no hay constructor, con `(Config, DynamicAccess)`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:285-321`
- `actor/src/main/resources/reference.conf:148-164`

**Condiciones.**
- Si la entrada define `resizer` y `resizer.enabled` no es true, se antepone `resizer.enabled=on` (compatibilidad hacia atrás; Deployer.scala:289-292).
- Errores: `IllegalArgumentException`/`ConfigException` del constructor se relanzan; `NoSuchMethodException` con 1 arg reintenta con 2; cualquier otro fallo -> `IllegalArgumentException("Cannot instantiate router [<fqn>], defined in [<key>], make sure it extends [class org.apache.pekko.routing.RouterConfig] and has constructor with [com.typesafe.config.Config] and optional [org.apache.pekko.actor.DynamicAccess] parameter", cause)` (Deployer.scala:296-301).
- `nr-of-instances` del default es 1; con resizer el pool queda con `nrOfInstances = 0` (DeployerSpec.scala:211-214).

**Orden / concurrencia / ciclo de vida.** Se resuelve al construir el Deployer (todas las entradas).

**Dependencias.** AC-CFG-014, AC-CFG-016, AC-RT-004, AC-RT-013.

**Equivalente Rust propuesto y diferencias observables.** Registro de fábricas con las claves de router.type-mapping preinstaladas; custom por nombre registrado.

**Aserciones.**
- A1 `from-code`, `service-direct` y `service-direct2` (nr-of-instances ignorado) -> NoRouter: DeployerSpec.scala:184-190.
- A2 round-robin, random, scatter-gather (within 2 s), consistent-hashing -> pools del tipo esperado, con el mismo resizer: :192-209.
- A3 resizer → `RoundRobinPool(nrOfInstances = 0, resizer = Some(DefaultResizer()))`: :211-214.
- A4 el mensaje de error de instanciación: derivada del código Deployer.scala:296-301 (*no afirmada por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-X-2.


---

## AC-CFG-012.4 — Secciones resizer y optimal-size-exploring-resizer: claves y defaults de deployment

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Claves de `deployment.default.resizer` y `deployment.default.optimal-size-exploring-resizer` con sus defaults.

**Fuente.**
- `actor/src/main/resources/reference.conf:237-347`

**Condiciones.**
- resizer: enabled=off, lower-bound=1, upper-bound=10, pressure-threshold=1, rampup-rate=0.2, backoff-threshold=0.3, backoff-rate=0.1, messages-per-resize=10.
- optimal-size-exploring-resizer: enabled=off, lower-bound=1, upper-bound=10, chance-of-ramping-down-when-full=0.2, action-interval=5s, downsize-after-underutilized-for=72h, explore-step-size=0.1, chance-of-exploration=0.4, downsize-ratio=0.8, optimization-range=16, weight-of-latest-metric=0.5.
- `pool-dispatcher` inline: sólo documentado en comentario (reference.conf:235-242); lo consume AC-RT.

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-RT-013, AC-RT-014.

**Equivalente Rust propuesto y diferencias observables.** Structs con `Default` coherente con el archivo.

**Aserciones.**
- A1 `DefaultResizer()` por defecto coincide con los valores de reference.conf: DeployerSpec.scala:211-214 compara `resizer` con `DefaultResizer()` construido sin argumentos.
- A2 defaults del optimal-size-exploring-resizer: derivados del archivo; sin test en mi grupo (los cubre AC-RT-014).

**Oráculo.** No aplica.

**Decisiones.** Ninguna.


---

## AC-CFG-013 — Deploy, Scope, LocalScope, NoScopeGiven (valores y withFallback)

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `Deploy(path, config, routerConfig, scope, dispatcher, mailbox, tags)` es la descripción final y combinable de un despliegue; `Scope` (LocalScope, NoScopeGiven, extensible) decide dónde se crea.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:28-220`

**Condiciones.**
- Defaults: path="", config vacía, routerConfig=NoRouter, scope=NoScopeGiven, dispatcher="", mailbox="", tags vacío; `Deploy.local = Deploy(scope = LocalScope)`; `DispatcherSameAsParent = ".."` (interno).
- `withFallback(other)`: `this` manda; `config`, `routerConfig` y `scope` se combinan con `X.withFallback(other.X)`; dispatcher/mailbox: los de `this` salvo que sean "" (entonces los de `other`); el `path` de `other` y los `tags` no se tienen en cuenta (el nuevo Deploy se crea sin tags: Deployer.scala:110-116).
- `LocalScope.withFallback(_) = LocalScope`; `NoScopeGiven.withFallback(other) = other`.
- Igualdad estructural sobre los 7 campos; `toString = Deploy(path, config, routerConfig, scope, dispatcher, mailbox, tags)`.

**Orden / concurrencia / ciclo de vida.** Inmutable.

**Dependencias.** AC-CFG-012, AC-REF-0xx (Props.deploy), AC-SYS-016 (merge de Deploy).

**Equivalente Rust propuesto y diferencias observables.** `struct Deploy` + `trait Scope { fn with_fallback(&self, other: &dyn Scope) -> Box<dyn Scope> }` o `enum Scope { Local, NoScopeGiven, Other(Arc<dyn ScopeImpl>) }`.

**Aserciones.**
- A1 el Deploy leído de config tiene `NoScopeGiven`, `NoRouter` y strings vacíos por defecto: DeployerSpec.scala:93-106.
- A2 dispatcher/mailbox sólo se rellenan cuando la config los define: DeployerSpec.scala:114-142.
- A3 `withFallback` pierde los tags: NO afirmada por spec; hallazgo del código Deployer.scala:109-117 (los tags de `this` no se copian al resultado).

**Oráculo.** Sí: `oracle/run.sh deploy-ids|deploy-all|deploy-case` (`DeployOracle.java`, 44 ids `DP`/`FB`/`SC`/`RT`): valores por defecto, igualdad de los siete campos, `Deploy.withFallback`, `Scope.withFallback` y `RouterConfig.withFallback` de `NoRouter`. Un router se imprime solo por su clase; `toString` no se graba (Q-CFG-35, propuesta).

**Decisiones.** Q-CFG-12, Q-CFG-35 (propuesta)



**Actualización P10 PR 1 (2026-11-20, Pekko 2.0.0).** Rust: `actor::{Deploy, Scope, RouterConfig}` (`actor/deploy.rs`) y `Config::same_value` / `Config::render_concise`. `Scope` solo tiene `Local` y `NoScopeGiven` (`#[non_exhaustive]`): ningún consumidor define otro scope en el módulo actor, así que no hay objeto de rasgo; se añadirá cuando exista el consumidor (scope remoto). `RouterConfig` tiene `NoRouter` y un router solo nombrado (`Opaque`); la fusión de dos pools reales (`overrideUnsetConfig`) y su `toString` son del módulo de routing (P37). La igualdad de `Deploy` compara la configuración por valores (`Config.equals` ignora el origen y el texto, y `1` es igual a `1.0`), no con `PartialEq` derivado, que incluye la línea de origen. 44 casos `dp-0001..dp-0044` grabados contra Pekko 2.0.0 y 9 + 2 tests unitarios: A1, A2 y A3 cubiertas. Queda `partial`: Q-CFG-35 (propuesta: `toString`); la fusión de pools reales y el `toString` del router (P37); y la integración con `Props.deploy` y el merge de `AC-SYS-016` (el sistema aún no existe).
---

## AC-CFG-014 — pekko.actor.router.type-mapping (alias de routers)

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `pekko.actor.router.type-mapping` traduce nombres cortos de router a clases.

**Fuente.**
- `actor/src/main/resources/reference.conf:147-164`
- `actor/src/main/scala/org/apache/pekko/actor/Deployer.scala:233-242`

**Condiciones.**
- 15 entradas: from-code=NoRouter, round-robin-pool/group, random-pool/group, balancing-pool, smallest-mailbox-pool, broadcast-pool/group, scatter-gather-pool/group (ScatterGatherFirstCompleted*), tail-chopping-pool/group, consistent-hashing-pool/group (todas `org.apache.pekko.routing.*`).
- Sólo valores String se incluyen en `routerTypeMapping` (collect de `String`).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-CFG-012.3, AC-RT-004.

**Equivalente Rust propuesto y diferencias observables.** Tabla estática nombre -> constructor (Q-X-2).

**Aserciones.**
- A1 mapeos de from-code, round-robin, random, balancing, smallest-mailbox, broadcast, scatter-gather, consistent-hashing (pool y group): DeployerSpec.scala:231-246 (el spec lista 13; tail-chopping no se afirma: derivado del reference.conf:167-168).

**Oráculo.** No aplica.

**Decisiones.** Q-X-2.


---

## AC-CFG-015 — BootstrapSetup / ActorSystemSetup / Setup (config programática: classLoader, config, defaultExecutionContext, provider)

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Configuración programática: `Setup` (marcador), `ActorSystemSetup` (mapa clase concreta -> Setup) y `BootstrapSetup` (classLoader, config, defaultExecutionContext, actorRefProvider).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/setup/ActorSystemSetup.scala:25-97`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:48-179`

**Condiciones.**
- `ActorSystemSetup(setups*)` indexa por `s.getClass` (clase concreta); un segundo Setup de la misma clase reemplaza al primero; `withSetup`/`and` equivalentes; `get[T]` devuelve Option; `get(Class)` Optional (ActorSystemSetup.scala:48-96).
- `BootstrapSetup()`/`apply(config)`/`create(...)`; `withClassloader`, `withConfig`, `withDefaultExecutionContext`, `withActorRefProvider` devuelven copias.
- Pasar un `BootstrapSetup` por defecto equivale a no pasar ninguno (ActorSystem.scala:93-96).
- `toString` = `ActorSystemSettings(<clases separadas por coma>)`.

**Orden / concurrencia / ciclo de vida.** Inmutable.

**Dependencias.** AC-CFG-001, AC-CFG-008, AC-SYS-001.

**Equivalente Rust propuesto y diferencias observables.** `ActorSystemSetup` con `HashMap<TypeId, Arc<dyn Any + Send + Sync>>`; `BootstrapSetup` builder (Q-CFG-13).

**Aserciones.**
- A1 guardar y recuperar un Setup: ActorSystemSetupSpec.scala:31-37.
- A2 el segundo Setup del mismo tipo reemplaza: :39-45.
- A3 `a and b and c`: :47-55.
- A4 crear con varios: :57-65.
- A5 el setup está en `system.settings.setup`: :67-78.
- A6 uso desde Java: `ActorSystemSetup.create().withSetup(new JavaSetup("Jasmine Rice")).get(JavaSetup.class)` devuelve un `Optional` presente e igual al original: ActorSystemSetupTest.java:23-34 (leído en el cierre documental).

**Oráculo.** `oracle/run.sh actor-system-setup-all|actor-system-setup-case|actor-system-setup-ids` (`ActorSystemSetupOracle.java`, familia `actor-system-setup`, casos `as-*`, 43 ids y 41 casos grabados): AS-01..10 (A1..A4 y A6: guardar y recuperar, reemplazar, `a and b and c`, `create` con varios, la API de Java con `get(Class)`, inmutabilidad y `and` frente a `withSetup`), TS-01..07 (`toString`: vacío, un setup, dos en orden de inserción, un setup reemplazado conserva su posición, cuatro, `create` con una clase repetida y `BootstrapSetup`), BS-01..14 (los factories `BootstrapSetup()`, `create()`, `apply(config)` y `create(config)`, `withConfig`, `withActorRefProvider` con Remote, Cluster, Local y Custom, que los withers conservan los demás campos y devuelven copias, y que un `BootstrapSetup` se recupera de un `ActorSystemSetup`) y PR-01..10 (la prioridad de `BootstrapSetup.actorRefProvider` sobre `pekko.actor.provider` en `new ActorSystem.Settings(classLoader, config, name, setup)`, `ActorSystem.scala:410-421`, y A5: el setup está en `settings.setup`). Los setups del spec son clases de Java del oráculo (DummySetup, DummySetup2, DummySetup3 y JavaSetup). BS-15 (`withClassloader`) y BS-16 (`withDefaultExecutionContext`) están en el oráculo y no los produce Rust (Q-CFG-32).

**Decisiones.** Q-CFG-13.


---

**Actualización P09 (2026-11-19, Pekko 2.0.0).** Rust: `config::Setup` (marcador con `class_name()` y `and`), `config::ActorSystemSetup` (`empty`, `with_setup` que reemplaza conservando la posición, `get::<T>()`, `len`, `is_empty`, `Display` = `toString`) y `config::BootstrapSetup` (`new`, `from_config`, `with_config`, `with_actor_ref_provider`, `config()`, `actor_ref_provider()`; cada `with_*` devuelve una copia), en `config/setup.rs`; `Settings::with_setup(name, config, setup)` (`Settings::new` = setup vacío), `Settings::setup` y la prioridad del proveedor del `BootstrapSetup` (`config/settings.rs`). Una selección del setup se lee por su `identifier` (como `ActorSystem.scala:410-414`): un `Custom(clase)` pasado por el `BootstrapSetup` acaba como el nombre de clase `custom`, tal como lo hace Pekko 2.0.0 (PR-06; es un bug de Pekko que se reproduce a propósito, Q-CFG-34, propuesta). 41 casos `as-0001..as-0041` grabados y 9 + 3 tests unitarios. A1..A4 y A6 cubiertas; A5 por `Settings::setup` (PR-09). Queda `partial`: Q-CFG-34 (propuesta): reproducir o corregir el bug del `Custom` del setup; Q-CFG-32 (propuesta): `BootstrapSetup` no tiene `classLoader` ni `defaultExecutionContext` (BS-15, BS-16); Q-CFG-33 (propuesta): orden de `toString` con más de cuatro setups y nombres de clase; AC-SYS-001: `ActorSystem` aún no recibe el setup ni usa `BootstrapSetup.config`, así que "el setup está en `system.settings.setup`" se verifica solo con `Settings`. Q-CFG-13 (aceptada) no deja nada abierto.

## AC-CFG-016 — DynamicAccess / ReflectiveDynamicAccess (instanciar clases por FQCN desde config)

*unidad CFG · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `DynamicAccess` carga y crea instancias por FQCN (mailboxes, dispatchers, loggers, serializadores, extensiones, scheduler, routers, estrategia de guardianes); `ReflectiveDynamicAccess` lo implementa con reflexión JVM.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/DynamicAccess.scala:32-71`
- `actor/src/main/scala/org/apache/pekko/actor/ReflectiveDynamicAccess.scala:35-150`

**Condiciones.**
- `createInstanceFor(fqcn, args)`: clase inexistente -> Failure(ClassNotFoundException); constructor no encontrado -> Failure(NoSuchMethodException); tipo incompatible -> `ClassCastException("<T> is not assignable from <c>")`/`"<clase> is not a subtype of <T>"` (ReflectiveDynamicAccess.scala:41,53); excepciones del constructor se relanzan sin envoltorio InvocationTargetException (línea 54).
- `getObjectFor(fqcn)`: busca `fqcn$` (módulo Scala) y si no `fqcn`; lee el campo `MODULE$`; null -> NPE; tipo incorrecto -> ClassCastException (líneas 69-87).
- `classIsOnClasspath(fqcn)`: false sólo ante ClassNotFoundException/NoClassDefFoundError (líneas 61-67).
- `classLoader` expuesto para casos especiales.

**Orden / concurrencia / ciclo de vida.** Síncrono; `Class.forName(fqcn, false, classLoader)` no inicializa la clase.

**Dependencias.** Consumido por AC-CFG-006/008/009/012, AC-EXT-003, AC-MBX-014, AC-DSP-010, AC-SCH-010, AC-EVT, AC-SER.

**Equivalente Rust propuesto y diferencias observables.** `FactoryRegistry<T>`: `register(name, fn(&Config, &Settings) -> Result<Box<T>, E>)`, `create(name, args) -> Result<Box<T>, FactoryError>`, `contains(name)`; los FQCN de reference.conf se preinstalan como claves (Q-X-2). Diferencia: no hay constructores alternativos por firma: el registro recibe una firma única.

**Aserciones.**
- A1 instanciar con constructor por defecto -> Success(no null): DynamicAccessSpec.scala:40-48.
- A2 clase inexistente -> Failure(ClassNotFoundException): :50-57.
- A3 probar constructor sin args y si no existe (NoSuchMethodException) el de (String): TestClassWithStringConstructor da "string ctor argument", TestClassWithDefaultConstructor "default", inexistente -> ClassNotFoundException: :59-70,77-86.
- A4 `classIsOnClasspath` falso/verdadero: :72-75.
- A5 `getObjectFor` y el cast: derivadas del código ReflectiveDynamicAccess.scala:69-87 (*no afirmadas por spec*).

**Oráculo.** `oracle/run.sh dynamic-access-all|dynamic-access-case|dynamic-access-ids` (`DynamicAccessOracle.java`, familia `dynamic-access`, casos `da-*`, 20): CNF-01..07 (`createInstanceFor` de un nombre inexistente, vacío, con otra capitalización o con espacios: A2, `ClassNotFoundException` con el nombre), ON-01..08 (`classIsOnClasspath`: A4), INST-01 (`createInstanceFor` de una clase con constructor por defecto: A1) y PRV-01..04 (`pekko.actor.provider` con `foo.NonExisting`, Remote, Cluster y Local al crear un `ActorSystem`: `ActorSystem.scala:996-1011` relanza la excepción sin envolver). Sin oráculo, por ser reflexión JVM sin equivalente en Rust (Q-X-2): A3 (constructores alternativos por firma), los *handles* de constructor y de campo `MODULE$` y A5 (`getObjectFor`); DynamicAccessSpec.scala:40-48, :50-57, :59-70, :72-75 y :102-109 se portan como tests unitarios (`config/dynamic_access.rs`).

**Decisiones.** Q-X-2.


---

**Actualización P09 (2026-11-19, Pekko 2.0.0).** Rust: `config::FactoryRegistry<F>` (`config/dynamic_access.rs`): `register` (reemplaza y devuelve la fábrica anterior), `contains` (= `classIsOnClasspath`), `lookup`, `create(nombre, argumentos)` y `names`; `ClassNotFoundError` (clase `java.lang.ClassNotFoundException`, mensaje = el nombre pedido, coincidencia exacta sin plegar mayúsculas ni recortar) y `CreateError::{ClassNotFound, Construction}` (el error de la fábrica se devuelve sin envolver). 20 casos grabados `da-0001..da-0020` y 11 tests unitarios. Queda `partial`: Q-CFG-31 (propuesta) corrige la redacción de Q-X-2 y de Q-CFG-9 (a): Pekko lanza `ClassNotFoundException`, no `ConfigurationException`; A3 y A5 no tienen equivalente; los FQCN de `reference.conf` se preinstalan cuando exista cada consumidor (AC-MBX-004, AC-DSP-004/010, AC-SCH-010, AC-EVT-016, AC-EXT-003, AC-SER-001.2), que transforma el error como Pekko.

## AC-CFG-017.1 — Combinación y sustitución: withFallback, resolve tardío, `${?x}` opcional, `+=`, autorreferencia, ciclos

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Combinación y sustitución: withFallback, resolve tardío, `${?x}` opcional, `+=`, autorreferencia, ciclos. Config.withFallback: gana el receptor; objetos se fusionan clave por clave; las sustituciones se resuelven tras combinar capas; `${?x}` ausente omite la clave; `${?x} [d]` concatena listas; `+=` agrega al valor anterior; ciclo o sustitución requerida ausente fallan.

**Fuente.**
- `actor/src/main/resources/reference.conf:11,78,926`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- Config.withFallback: gana el receptor; objetos se fusionan clave por clave; las sustituciones se resuelven tras combinar capas; `${?x}` ausente omite la clave; `${?x} [d]` concatena listas; `+=` agrega al valor anterior; ciclo o sustitución requerida ausente fallan.
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden FB-*, SUB-*, REF-01, REF-02 se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `config/{parse,resolve}.rs`; casos grabados FB-01..10, SUB-01..14, REF-01..02 pasan (parser propio). Queda `partial`: afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida. Sus aserciones A1 y A2 están cubiertas por casos grabados (FB, SUB, REF-01..02, KEY).

## AC-CFG-017.2 — Duraciones: unidades, nombres largos, sin unidad = ms, fracciones, negativos, notación científica, truncado, desborde

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Duraciones: unidades, nombres largos, sin unidad = ms, fracciones, negativos, notación científica, truncado, desborde. Unidades ns/us/ms/s/m/h/d y nombres largos; número sin unidad = ms; trunca hacia cero; admite negativos; errores BadValue con mensajes del jar.

**Fuente.**
- `actor/src/main/resources/reference.conf:36,68,145,211,218,312,316`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- Unidades ns/us/ms/s/m/h/d y nombres largos; número sin unidad = ms; trunca hacia cero; admite negativos; errores BadValue con mensajes del jar.
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden DUR-* se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `config/units.rs` (`get_duration`); casos grabados DUR-* pasan. Queda `partial`: afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida. Sus aserciones A1 y A2 están cubiertas por casos grabados (DUR-*).

## AC-CFG-017.3 — Tamaños en memoria: `128 KiB`, `K`, `kB`, `MB`, número sin unidad = bytes; `KB` falla

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Tamaños en memoria: `128 KiB`, `K`, `kB`, `MB`, número sin unidad = bytes; `KB` falla. getBytes/getMemorySize con prefijos binarios y decimales; negativos/fuera de rango fallan.

**Fuente.**
- `actor/src/main/resources/reference.conf:1018,1063,1112,1168`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- getBytes/getMemorySize con prefijos binarios y decimales; negativos/fuera de rango fallan.
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden SIZE-* se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `config/units.rs` (`get_bytes`, `get_memory_size`); casos grabados SIZE-* pasan (los tamaños fraccionarios usan f64, los enteros i128). Queda `partial`: afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida. Sus aserciones A1 y A2 están cubiertas por casos grabados (SIZE-*).

## AC-CFG-017.4 — Booleanos `on/off/yes/no/true/false` con y sin comillas; `ON`, `1`, `maybe` fallan

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Booleanos `on/off/yes/no/true/false` con y sin comillas; `ON`, `1`, `maybe` fallan. getBoolean acepta los seis literales (también entre comillas); distingue mayúsculas.

**Fuente.**
- `actor/src/main/resources/reference.conf:50,63,86,90,98,103,241`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- getBoolean acepta los seis literales (también entre comillas); distingue mayúsculas.
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden BOOL-* se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `Config::get_boolean`; casos grabados BOOL-* pasan. Queda `partial`: afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida. Sus aserciones A1 y A2 están cubiertas por casos grabados (BOOL-*).

## AC-CFG-017.5 — Literales que Pekko interpreta por su cuenta: `infinite`, `unlimited`, `forever`, `never`, `default`, `on/off` en log-dead-letters

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Literales que Pekko interpreta por su cuenta: `infinite`, `unlimited`, `forever`, `never`, `default`, `on/off` en log-dead-letters. El parser entrega texto; getDuration/getBytes rechazan `infinite`/`unlimited`; Pekko los interpreta antes (ActorSystem.scala:441-453) o con helpers (DnsSettings, Tcp).

**Fuente.**
- `actor/src/main/resources/reference.conf:64-68,911,1098,1216,1234,1244,1250`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- El parser entrega texto; getDuration/getBytes rechazan `infinite`/`unlimited`; Pekko los interpreta antes (ActorSystem.scala:441-453) o con helpers (DnsSettings, Tcp).
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden SPECIAL-*, REF-03 se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: literales como `infinite`/`unlimited` son cadenas (`get_value`, `get_string`); casos grabados SPECIAL-*, REF-03 pasan. Queda `partial` (corregido 2026-11-19): afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): el expected de SPECIAL-08 es `ERR BadValue [String: 1: Invalid value at 'x': ...]` y `String: 1:` es la descripción de origen que decide Q-CFG-15. Sus aserciones A1 y A2 están cubiertas por casos grabados (SPECIAL-*, REF-03, KEY).

## AC-CFG-017.6 — Includes: `include "version"` inexistente se ignora; `required(...)` inexistente falla

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Includes: `include "version"` inexistente se ignora; `required(...)` inexistente falla. Un include no encontrado se ignora salvo `required(...)`; reference.conf incluye el generado `version`.

**Fuente.**
- `actor/src/main/resources/reference.conf:11`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- Un include no encontrado se ignora salvo `required(...)`; reference.conf incluye el generado `version`.
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden INC-* se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: includes (`include "x"`, `required(..)`, `file(..)`, `classpath(..)`); casos grabados INC-01..04 pasan; los mensajes de `required(file/url)` no están medidos (Q-CFG-14 propuesta). Queda `partial`: Q-CFG-14 (propuesta) sobre includes sin classpath, `url()` y los mensajes de `required(file/url)` sin medir. INC-01..04 están grabados.

## AC-CFG-017.7 — Sintaxis del archivo: comentarios `#` y `//`, claves con guiones y puntos, comillas triples, claves duplicadas, listas con coma final, errores de getters tipados

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Sintaxis del archivo: comentarios `#` y `//`, claves con guiones y puntos, comillas triples, claves duplicadas, listas con coma final, errores de getters tipados. El parser debe aceptar reference.conf completo sin cambios; getters tipados: clave ausente (Missing), tipo erróneo (WrongType), null.

**Fuente.**
- `actor/src/main/resources/reference.conf:1-1495`
- tools/oracle/config-hocon.jsh
- crates/rakka-actor/tests/oracle/config-hocon.golden

**Condiciones.**
- El parser debe aceptar reference.conf completo sin cambios; getters tipados: clave ausente (Missing), tipo erróneo (WrongType), null.
- Los mensajes de error de Typesafe (TYPE-*, SUB-*, DUR-ERR, SIZE-ERR) forman parte del golden; reproducirlos textualmente es parte de Q-X-1/Q-CFG-4.

**Orden / concurrencia / ciclo de vida.** La resolución es una pasada posterior a la combinación (FB-05).

**Dependencias.** AC-CFG-001, AC-UTIL-002.4.

**Equivalente Rust propuesto y diferencias observables.** Parser/resolver HOCON propio del subconjunto (recomendación del plan: opción A) con el golden como suite de aceptación; helpers `get_duration`, `get_bytes`, `get_boolean`.

**Aserciones.**
- A1 los casos del golden REF-*, LIST-*, TYPE-* se reproducen con el parser Rust (cada id del golden es una aserción; *no afirmadas por spec de Pekko*, afirmadas por el jar config-1.4.9).
- A2 `reference.conf` oficial se carga sin cambios: derivado del plan; ConfigSpec.scala:30.

**Oráculo.** Sí para el formato HOCON y los lectores tipados: golden de 161 casos con el jar config-1.4.9 (tools/oracle/config-hocon.jsh). Para los defaults de reference.conf: el propio archivo es la fuente (se embebe sin cambios).

**Decisiones.** Q-X-1, Q-CFG-4


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: sintaxis (comentarios, claves con comillas y guiones, comillas triples, claves duplicadas, listas), getters tipados y sus errores; casos grabados TYPE-*, LIST-*, REF-04..10 pasan. Queda `partial`: afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida. Sus aserciones A1 y A2 están cubiertas por casos grabados (REF-04..10, LIST-*, TYPE-*).

## AC-CFG-018 — Settings.amendSlf4jConfig: `pekko.use-slf4j` agrega Slf4jLogger/Slf4jLoggingFilter a loggers

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Adaptación de la config para que Typed use SLF4J por defecto: sólo si `pekko.use-slf4j` es true y la clase del logger existe.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:332-378`
- `actor-typed/src/main/resources/reference.conf:68 (pekko.use-slf4j = on)`
- `actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala:187-204`

**Condiciones.**
- Si `pekko.loggers` ya contiene Slf4jLogger: config sin cambios, salvo que el filtro sea el default -> se fuerza `Slf4jLoggingFilter` (líneas 357-362).
- Si `use-slf4j` existe y es true y la clase está en el classpath: `loggers = [Slf4jLogger] + loggers sin DefaultLogger`; el filtro se cambia sólo si todavía es DefaultLoggingFilter (líneas 364-372).
- En otro caso la config no cambia (línea 374).

**Orden / concurrencia / ciclo de vida.** Se aplica antes de construir Settings (ActorSystem.scala:850-855).

**Dependencias.** AC-CFG-004, AC-CFG-016, AC-EVT-011, Q-X-7.

**Equivalente Rust propuesto y diferencias observables.** Si Q-X-7 elige backend `tracing`/`log`, esta función desaparece; si elige actores-logger, se registra Slf4jLogger como fábrica opcional.

**Aserciones.**
- A1 con la config de referencia no se amenda: 1 logger = DefaultLogger y filtro = DefaultLoggingFilter: ConfigSpec.scala:187-193.
- A2 `use-slf4j=on` sin pekko-slf4j en el classpath no amenda: ConfigSpec.scala:195-203.
- A3 el caso con pekko-slf4j disponible no está afirmado en mi grupo; derivado del código ActorSystem.scala:364-372.

**Oráculo.** No aplica.

**Decisiones.** Q-X-7, Q-CFG-10


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `config::amend_slf4j_config(&Config, is_registered)` (`is_registered` es `dynamicAccess.classIsOnClasspath`: si hay una fábrica registrada con ese nombre, Q-X-2/Q-X-7 (A)); 45 casos grabados cs-* AMEND (22 configuraciones por 2 estados del classpath, más una doble aplicación) contra `Settings.amendSlf4jConfig` de Pekko 2.0.0: A1, A2 y A3. Queda `partial`: Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos de error grabados; `ActorSystem::create_with_settings` aún no llama a esta función (AC-SYS) y el registro de fábricas (AC-CFG-016) no existe: el predicado es un parámetro.

## AC-CFG-019 — Validación de la config contra reference.conf (`checkValid` con `internal-dispatcher` excluido)

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Validación temprana de toda la config de `pekko.*` contra los defaults.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:396-403`
- `actor/src/main/scala/org/apache/pekko/dispatch/Dispatchers.scala:77,136` (`InternalDispatcherId = "pekko.actor.internal-dispatcher"`; `internalDispatcher = lookup(InternalDispatcherId)`; leído en el cierre documental)

**Condiciones.**
- Se valida el path `pekko` completo; la excepción es `ConfigException.ValidationFailed` agregada (comportamiento de Typesafe, no está en upstream/pekko).
- `pekko.actor.internal-dispatcher` se excluye porque en configs antiguas es un String (nombre de dispatcher) y en la referencia un objeto.

**Orden / concurrencia / ciclo de vida.** Primer paso del constructor de Settings.

**Dependencias.** AC-CFG-001, AC-CFG-017.

**Equivalente Rust propuesto y diferencias observables.** `check_valid` implementado en el parser propio con reglas de compatibilidad documentadas (Q-CFG-3).

**Aserciones.**
- A1 la config por defecto pasa la validación: ConfigSpec.scala:30-110 (se crea el sistema con `defaultReference`).
- A2 comportamiento ante claves ausentes/de tipo erróneo: NO afirmada por spec; derivada de la semántica de Typesafe (jar config-1.4.9), sin escenario en el golden.

**Oráculo.** Posible con el jar config-1.4.9 (checkValid sobre configs de ejemplo); no hay escenario en config-hocon.golden.

**Decisiones.** Q-CFG-3


---

**Actualización P07 (2026-11-18, Pekko 2.0.0).** Rust: `Config::check_valid` y `without_path`; casos grabados CV-01..21 (incluida la validación del reference.conf real de pekko-actor) pasan. La llamada desde `Settings` (con `internal-dispatcher` excluido) llega con ActorSystem. Queda `partial`: afectada por Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente): descripción de origen en los mensajes de error y en el render; la ruta de archivo no está medida. Sus aserciones A1 y A2 están cubiertas por CV-01..21 grabados.

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::new` llama a `Config::check_valid` con el reference de `Config::default_reference()` sin `pekko.actor.internal-dispatcher`; casos grabados cs-* SET-INVALID (`internal-dispatcher` como nombre de dispatcher permitido, tipos erróneos, varios problemas a la vez, config sin resolver). No cambia de estado (`partial`): Q-CFG-15 (aceptada con cambios 2026-11-28, antes propuesta; origen de entorno y de archivo pendiente) afecta a los textos con `String: N`; Q-CFG-18 y Q-CFG-19 (propuestas): orden de claves de `expecting: object with keys [..]`, origen de una clave ausente y reference contra el que se valida.

## AC-CFG-020 — Combinación no soportada coordinated-shutdown: run-by-actor-system-terminate=on con terminate-actor-system=off

*unidad CFG · visibilidad internal-required · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Rechazo de una combinación de banderas de CoordinatedShutdown incoherente al crear el sistema.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:473-480`
- `actor/src/main/resources/reference.conf:1297-1328`

**Condiciones.**
- Error: `ConfigurationException("pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.")` (ActorSystem.scala:479-481).

**Orden / concurrencia / ciclo de vida.** Se evalúa tras leer las dos banderas, antes del chequeo de versión.

**Dependencias.** AC-CS-006, AC-UTIL-004.2.

**Equivalente Rust propuesto y diferencias observables.** Validación en el constructor de Settings.

**Aserciones.**
- A1 la combinación `terminate-actor-system = off` (con `run-by-actor-system-terminate` por defecto `on`) hace que `ActorSystem(...)` lance `ConfigurationException`: CoordinatedShutdownSpec.scala:572-582 (afirma sólo el **tipo**; **corrección del cierre documental**: el contrato decía "sin test").
- A2 el texto exacto del mensaje no lo afirma ningún test: derivado del código ActorSystem.scala:478-481 (*no afirmada por spec*).
- A3 con `run-by-actor-system-terminate = off`, `terminate()` no dispara CoordinatedShutdown (`shutdownReason() == None`): CoordinatedShutdownSpec.scala:557-570 (pertenece a AC-CS-006).

**Oráculo.** No aplica.

**Decisiones.** Ninguna.


---

**Actualización P08 (2026-11-19, Pekko 2.0.0).** Rust: `Settings::new` falla con `ConfigurationException` con el texto exacto de Pekko cuando `run-by-actor-system-terminate=on` y `terminate-actor-system=off`; casos grabados cs-* SET-CS (cuatro combinaciones, la clave sola con el otro valor por defecto, el orden respecto del chequeo de versión y de `virtual-nodes-factor`): A1 y A2. Sin decisión propuesta que afecte a sus expected (ninguno lleva `String: N`). Queda `partial` sólo por la integración: A1 afirma que `ActorSystem(..)` lanza la excepción y `ActorSystem::create_with_settings` aún no construye estos `Settings` (AC-SYS); A3 pertenece a AC-CS-006.

## AC-PATH-001 — Address: protocol, system, host?, port?; alcance local vs global; toString y hostPort

*unidad PATH · visibilidad public · analysis=extracted · impl=partial · validation=oracle+spec*

**Contrato observable.** `Address(protocol, system, host?, port?)`: dirección de un sistema de actores; con host+puerto tiene alcance global, sin ellos alcance local. `toString` = `protocol://system[@host][:port]`; `hostPort` = `system[@host][:port]`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:38-102`
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:120-134 (companion, InvalidHostRegex, apply)`
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:102 (hostPort)`

**Condiciones.**
- Constructores: `Address(protocol, system)` (host y puerto None) y `Address(protocol, system, host, port)` (Some/Some; constructor Java con `Option(host)`: host null -> None). `copy` re-crea con los campos dados.
- `hasLocalScope = host.isEmpty`; `hasGlobalScope = host.isDefined`.
- `toString` agrega `@host` si hay host y `:port` si hay puerto (puede haber puerto sin host: `pekko://s:5`), `hostPort = toString.substring(protocol.length + 3)`.
- Java API: `getHost(): Optional<String>`, `getPort(): Optional<Integer>`.
- `hashCode` = MurmurHash3.productHash (Q-PATH-3). El puerto es `Option[Int]`: acepta cualquier Int, incluso negativo (oráculo CT-04..08).

**Orden / concurrencia / ciclo de vida.** Valor inmutable (case class); `hashCode` y `toString` son `lazy val` transitorios.

**Dependencias.** Ninguna fila (base de AC-PATH-*).

**Equivalente Rust propuesto y diferencias observables.** `struct Address { protocol, system, host: Option<String>, port: Option<i32> }` con `Display`/`Eq`/`Hash` y sin `Ord` (ya implementado en crates/rakka-actor/src/actor/address.rs). Diferencias: hash distinto (Q-PATH-3), sin `null` (Q-PATH-4).

**Aserciones.**
- A1 `new Address("pekko", "MySystem", "localhost", 2525)`: `getPort == Optional.of(2525)` y `getHost == Optional.of("localhost")`: AddressTest.java:26-30.
- A2 `Address("pekko","my_sys",Some(host),Some(1234))` es igual al parseado de `pekko://my_sys@host:1234`: ActorPathSpec.scala:40-48 (y SPEC-APS-42/47 del oráculo: local=false global=true hostPort=my_sys@host:1234).
- A3 `toString`/`hostPort`/alcance para las combinaciones con y sin host/puerto: oráculo CT-*, AD-* (*no afirmada por spec* salvo A2).
- A4 puerto negativo aceptado por el constructor, rechazado por el parser (AD-13..19): oráculo (*no afirmada por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-3, Q-PATH-4


---

## AC-PATH-002 — Orden de Address (addressOrdering)

*unidad PATH · visibilidad public · analysis=extracted · impl=partial · validation=oracle+spec*

**Contrato observable.** `Address.addressOrdering` ordena por protocolo, luego sistema, luego host (`None` = "") y luego puerto (`None` = 0), con comparación de Strings por unidades UTF-16.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:136-146`

**Condiciones.**
- `a eq b` -> no menor; si protocolos distintos -> `compareTo` < 0; igual con sistema; luego `a.host != b.host` compara `getOrElse("")`; luego puertos (`getOrElse(0)`) (Address.scala:139-144).
- No es un orden total: host None vs Some("") y port None vs Some(0) dan 'no menor' en ambos sentidos sin ser iguales, y la incomparabilidad no es transitiva (oráculo CMPA-08..13).
- El comparador es `Ordering.fromLessThan`: el resultado de `sorted` depende del algoritmo (TimSort de Java) en entradas inconsistentes (CMPA-17).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-001.

**Equivalente Rust propuesto y diferencias observables.** `Address::pekko_compare(&self, &Address) -> Ordering` sin `impl Ord` (decisión del Owner 2026-10-30, ya aplicada); ordenar listas: Q-PATH-1.

**Aserciones.**
- A1 `[pekko h1, pekko h2, akka h1, akka h2].sorted == [akka h1, akka h2, pekko h1, pekko h2]`: AddressSpec.scala:25-36 (oráculo SPEC-ADS-34).
- A2 casos de orden parcial y UTF-16: oráculo CMPA-01..17 (*no afirmados por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-1, Q-PATH-6


---

## AC-PATH-003 — Validación de caracteres de host (InvalidHostRegex "_[^.]*$")

*unidad PATH · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** `hasInvalidHostCharacters` detecta hosts con `_` sin punto posterior; `checkHostCharacters()` lanza `IllegalArgumentException` si los hay. `AddressFromURIString` rechaza esos hosts porque `java.net.URI` no los reconoce como host.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:104-116`
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:122-123 (InvalidHostRegex = "_[^.]*$")`

**Condiciones.**
- Regex `_[^.]*$` buscada en el host: `valid`, `is_valid.org`, `fu.is_valid.org` son válidos; `in_valid`, `invalid._org` son inválidos (el `_` no va seguido de punto hasta el final).
- `checkHostCharacters()`: `require(!hasInvalidHostCharacters, s"Using invalid host characters '$host' in the Address is not allowed.")` -> IllegalArgumentException con prefijo `requirement failed: ` y el Option impreso `Some(in_valid)` (oráculo HC-*).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-001, AC-UTIL-002.3 (prefijo `requirement failed:`).

**Equivalente Rust propuesto y diferencias observables.** `has_invalid_host_characters(&self) -> bool` y `check_host_characters(&self) -> Result<(), IllegalArgumentError>` (implementado).

**Aserciones.**
- A1 hosts válidos `valid`, `is_valid.org`, `fu.is_valid.org` -> false; `in_valid`, `invalid._org` -> true: ActorPathSpec.scala:117-124.
- A2 `AddressFromURIString("pekko://sys@in_valid:5001")` lanza MalformedURLException: ActorPathSpec.scala:126.
- A3 `checkHostCharacters` no falla con `localhost`, `is_valid.org`, `fu.is_valid.org`: ActorPathSpec.scala:129-134.
- A4 falla con IllegalArgumentException para `in_valid` y `invalid._org`: ActorPathSpec.scala:136-144 (el mensaje exacto: oráculo HC-*, *no afirmado por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Ninguna.


---

## AC-PATH-004 — AddressFromURIString / parse (semántica de java.net.URI)

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** `AddressFromURIString(s)` / `.parse` / `.unapply` construye un Address a partir de `protocol://system[@host:port]` usando `java.net.URI`; falla con `MalformedURLException(s)` si no es una dirección válida.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:186-217`

**Condiciones.**
- Rechaza (None / MalformedURLException(addr)) si: URI mal formada (URISyntaxException), sin esquema, sin userInfo ni host, `pekko://system:5` (con puerto pero sin userInfo), userInfo sin host o sin puerto (Address.scala:190-202).
- Con `pekko://system` (userInfo null, host=system) crea `Address(scheme, host)`; con `pekko://sys@host:1234` crea `Address(scheme, userInfo, host, port)`.
- Dependencia de java.net.URI: hosts con `_` o no ASCII caen en autoridad 'registry' y se rechazan; sólo el userInfo (nombre del sistema) se decodifica (%XX como UTF-8 con U+FFFD por secuencia malformada); host, path y fragmento quedan crudos; IPv6 conserva corchetes; puerto > Int.MaxValue se rechaza (oráculo AD-*, UTF-AD-*, UTF-FS-*).
- Nota: java.net.URI del JDK no está en upstream/pekko: su comportamiento se conoce sólo por el oráculo (JDK 11 en el golden; el port mira OpenJDK 21: ver Q-PATH-6).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-001, java.net.URI.

**Equivalente Rust propuesto y diferencias observables.** Parser propio del subconjunto RFC 2396 (`uri.rs`, implementado) + `address_from_uri_string(&str) -> Result<Address, MalformedUrl>`. Diferencia a decidir: licencia del código portado de OpenJDK (Q-PATH-6).

**Aserciones.**
- A1 `AddressFromURIString("pekko://my_sys@host:1234") == Address("pekko","my_sys",Some("host"),Some(1234))`: ActorPathSpec.scala:40-43.
- A2 lo mismo con protocolo `akka`: ActorPathSpec.scala:45-48.
- A3 host con `_` inválido -> MalformedURLException: ActorPathSpec.scala:126.
- A4 forma sin `@` y casos límite (puerto, IPv6, UTF-8 del userInfo): oráculo AD-*, UTF-AD-01..30 (*no afirmados por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-6, Q-PATH-4


---

## AC-PATH-005 — RelativeActorPath (extractor de paths relativos)

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** `RelativeActorPath.unapply(addr)` devuelve los elementos de un path relativo (`grand/child`, `/user/hello/world`) y None si la cadena es una URI absoluta o inválida.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:149-181 (PathUtils.split + RelativeActorPath)`

**Condiciones.**
- `new URI(addr)`: si `isAbsolute` -> None; si URISyntaxException -> None; si no `split(uri.getRawPath, uri.getRawFragment)`.
- `split`: parte por `/` desde la derecha; si hay fragmento se concatena `#fragmento` al último elemento (`foo/bar/baz#1234` -> `baz#1234`); un path que empieza con `/` produce un primer elemento vacío.
- No decodifica (usa las versiones 'raw'); la query se descarta (oráculo RP-*).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-004 (java.net.URI).

**Equivalente Rust propuesto y diferencias observables.** `relative_actor_path(&str) -> Option<Vec<String>>` (implementado).

**Aserciones.**
- A1 `foo` -> [foo]: RelativeActorPathSpec.scala:28-30.
- A2 `foo/bar/baz` -> [foo, bar, baz]: :31-33.
- A3 un nombre URL-encodeado (`pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355`) queda como un solo elemento: :34-37.
- A4 `foo/bar/baz#1234` -> [foo, bar, baz#1234]: :38-40.

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-6


---

## AC-PATH-006 — ActorPathExtractor (Address + elementos desde URI absoluta)

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** `ActorPathExtractor.unapply(addr)` extrae `(Address, elementos)` de una URI absoluta de actor.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Address.scala:219-233`

**Condiciones.**
- `new URI(addr)`; si `getRawPath` es null o hay URISyntaxException -> None; si no, `AddressFromURIString.unapply(uri)` y `split(path, rawFragment).drop(1)` (se descarta el primer elemento vacío).
- Con fragmento, el último elemento lleva `#uid`.

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-004, AC-PATH-005 (PathUtils).

**Equivalente Rust propuesto y diferencias observables.** Función de parseo usada por `ActorPath::from_string` (implementada).

**Aserciones.**
- A1 `pekko://my_sys@host:1234/some/ref` -> Address + [some, ref]: observado vía `fromString(...).toString`: ActorPathSpec.scala:30-33 (oráculo SPEC-APS-32).
- A2 lo mismo con `akka://`: ActorPathSpec.scala:35-38 (SPEC-APS-37).
- A3 casos límite: oráculo FS-* (*no afirmados por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-6


---

## AC-PATH-007 — ActorPath.fromString

*unidad PATH · visibilidad public · analysis=extracted · impl=partial · validation=oracle+spec*

**Contrato observable.** `ActorPath.fromString(s)` parsea una cadena de path absoluto; `MalformedURLException("cannot parse as ActorPath: " + s)` si no es parseable.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:68-76`

**Condiciones.**
- `RootActorPath(address) / elems`: cada elemento pasa por `/` (que interpreta `nombre#uid`, AC-PATH-011).
- Errores: `""`, `"://hallo"`, `"s://dd@:12"`, `"s://dd@h:hd"`, `"a://l:1/b"` -> MalformedURLException con mensaje `cannot parse as ActorPath: <entrada>` (oráculo SPEC-APS-51..55).
- Un uid no numérico (`a#x`) lanza `NumberFormatException` desde `Integer.valueOf`, que NO se captura (ActorCell.scala:400-404; oráculo FS-36..56).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-006, AC-PATH-010, AC-PATH-011.

**Equivalente Rust propuesto y diferencias observables.** `ActorPath::from_string(&str) -> Result<ActorPath, ParseActorPathError>` con dos variantes: `MalformedUrl` y `NumberFormat` (hoy `MalformedUrl` según el inventario; ver Q-PATH-4).

**Aserciones.**
- A1 `fromString(path.toString) == path` para `pekko://mysys/user`: ActorPathSpec.scala:25-28.
- A2 `pekko://my_sys@host:1234/some/ref` y la variante `akka://` conservan su toString: :30-38.
- A3 las cinco entradas malformadas lanzan MalformedURLException: :50-56.
- A4 NumberFormatException por uid inválido: oráculo FS-* (*no afirmada por spec*; derivada del código ActorCell.scala:400-404).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-4, Q-PATH-6


---

## AC-PATH-008 — Validación de elementos de path (nombres de actor)

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** Un elemento de path (nombre de actor) debe ser no vacío, no empezar con `$` e incluir sólo letras ASCII, dígitos, los símbolos `-_.*$+:@&=,!~';` y secuencias `%XX` (dos dígitos hex).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:78-174`
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:34-65 (ActorPaths: fachada)`

**Condiciones.**
- Vacío -> `InvalidActorNameException("Actor path element must not be empty " + fullPathMsg)` (nótese el espacio antes de fullPathMsg, que es " (in path [<fullPath>])" o "") (ActorPath.scala:105-106).
- Otro carácter inválido en la posición `n` (índice UTF-16; `$` en la posición 0 cuenta) -> `InvalidActorNameException("Invalid actor path element [<e>]<fullPathMsg>, illegal character [<c>] at position: <n>. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';.")` (líneas 107-113).
- `%` válido sólo si lo siguen dos dígitos hex (`pos + 2 < len`): `a%`, `%3`, `%xx`, `%0G`, `%gg`, `%`, `%1t` son inválidos (LocalActorRefProviderSpec.scala:173-186).
- `isValidPathElement(s)` es la versión booleana.

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-UTIL-002 (no), AC-PATH-009 (fachada).

**Equivalente Rust propuesto y diferencias observables.** `validate_path_element(&str) -> Result<(), InvalidActorName>` y `is_valid_path_element(&str) -> bool` (implementado). Surrogate suelto en el mensaje: Q-PATH-2; `null` no representable: Q-PATH-4.

**Aserciones.**
- A1 elemento vacío -> InvalidActorNameException cuyo mensaje incluye `must not be empty`: ActorPathSpec.scala:79-82.
- A2 `system.actorOf(Props.empty, null)` -> mensaje contiene `null`; `""` -> `empty`; `"$hallo"` -> `not start with `$``: LocalActorRefProviderSpec.scala:169-172.
- A3 `a%`, `%3`, `%xx`, `%0G`, `%gg`, `%`, `%1t`, `a?` -> `Invalid actor path element`; `üß` -> `include only ASCII`: :173-190.
- A4 posición y carácter: `he"llo` -> `["] at position: 2`; `$hello` -> `[$] at position: 0`; `hell>o` -> `[>] at position: 4`: :192-197.
- A5 mensaje completo con `fullPath` y doble espacio: oráculo VAL-* (*no afirmada por spec*).
- A6 son válidos `a`, `actor-1`, todas las letras y dígitos ASCII, todos los símbolos, `a$b`, `%20`, `%ff%FF%0a`, `a%20b` y `x` repetida 10000 veces: ActorPathSpec.scala:85-101.
- A7 cada nombre inválido se rechaza informando la posición UTF-16 (`$a` 0, `a%2g` 1, `café` 3, `中文` 0, par sustituto en 6, NUL y DEL en 1, `x`*100+`é` en 100, y el resto de la tabla): ActorPathSpec.scala:103-134.
- A8 para cada uno de los 65.536 `char` c: `c` es válido si es del alfabeto y no `$`; `a`+c si es del alfabeto; `%`+c+`0` y `%0`+c si c es hex: ActorPathSpec.scala:136-160. Los 2.048 sustitutos no caben en `&str` (Q-X-17) y se comprueban sobre el recorrido UTF-16; el caso grabado `VAL-A8` compara los rangos aceptados (el oráculo evalúa los 65.536 y no acepta ninguno de ellos).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-2, Q-PATH-4


---

## AC-PATH-009 — ActorPaths (fachada Java: fromString, validatePathElement, isValidPathElement)

*unidad PATH · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** `ActorPaths` es una fachada estática Java sobre `object ActorPath`: `fromString`, `validatePathElement` (con y sin `fullPath`) e `isValidPathElement`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:30-66`

**Condiciones.**
- Sin lógica propia: delega 1:1 en ActorPath.
- Existe porque `trait ActorPath` no podía pasar a clase abstracta con compatibilidad binaria (comentario en ActorPath.scala:30-31).

**Orden / concurrencia / ciclo de vida.** n/a.

**Dependencias.** AC-PATH-007, AC-PATH-008.

**Equivalente Rust propuesto y diferencias observables.** Las mismas funciones libres (sin fachada); la fila queda `partial` hasta decidir Q-PATH-5.

**Aserciones.**
- A1 equivalencia con ActorPath.*: oráculo VAL-JAPI-*, SPEC-JAPI (*no afirmada por spec de Pekko*; `ActorSelectionTest.java:35-45` sólo prueba `selection.resolveOne(timeout)` sobre `user/ref1` y no llama a `ActorPaths`, leído en el cierre documental).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-5


---

## AC-PATH-010 — Estructura de ActorPath: RootActorPath / ChildActorPath, address, name, parent, root, `/`, child, descendant, elements

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** Jerarquía `RootActorPath` / `ChildActorPath` con `address`, `name`, `parent`, `root`, `/`, `child`, `descendant`, `elements`, `toStringWithoutAddress`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:200-242`
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:307-328`
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:356-388`

**Condiciones.**
- `root / "user"` crea un ChildActorPath; `/` con `Iterable` ignora elementos vacíos (`if (elem.isEmpty) path else path / elem`) (ActorPath.scala:203-204).
- `elements` de la raíz es `List("")` (`emptyActorPath`) pero `ChildActorPath.elements` excluye la raíz (rec hasta RootActorPath); `RootActorPath.parent = this`; `root` camina hacia arriba.
- ChildActorPath: `name` con `/` -> IllegalArgumentException (AC-PATH-016).
- Profundidad: la implementación es iterativa (@tailrec); un path muy profundo no desborda la pila.

**Orden / concurrencia / ciclo de vida.** Inmutable; el padre se comparte.

**Dependencias.** AC-PATH-001, AC-PATH-016.

**Equivalente Rust propuesto y diferencias observables.** `ActorPath(Arc<Node>)` con nodo padre compartido; render/igualdad/hash/compare iterativos (un path de 1 000 000 niveles se construye, compara y libera: oráculo DEEP-01).

**Aserciones.**
- A1 `(root / user / foo / bar).elements.toSeq == [user, foo, bar]`: ActorPathSpec.scala:66-69.
- A2 toString de cada nivel: ActorPathSpec.scala:58-64.
- A3 `child`, `descendant` y NAV-*: oráculo (*no afirmados por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Ninguna.


---

## AC-PATH-011 — uid en paths: "#uid" en `/`, undefinedUid=0, newUid aleatorio ≠0, withUid; RootActorPath siempre uid 0

*unidad PATH · visibilidad internal-required · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** Un path puede llevar el uid de la incarnación del actor como fragmento `#uid`; `/` acepta `nombre#uid`; `withUid`; `undefinedUid = 0`; `newUid` aleatorio distinto de 0; el uid se ve en `toSerializationFormat*`, no en `toString`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:292-298,344-351,364-370,393-395`
- `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:389-403 (undefinedUid, newUid, splitNameAndUid)`

**Condiciones.**
- `splitNameAndUid(name)`: si hay `#`, parte el nombre y hace `Integer.valueOf(resto)` (acepta `+`/`-` y dígitos Unicode; vacío o desborde -> NumberFormatException) (ActorCell.scala:400-404).
- `RootActorPath.uid = 0` y `withUid(uid != 0)` lanza `IllegalStateException("RootActorPath must have undefinedUid, [<uid> != 0")` (nótese el corchete sin cierre) (ActorPath.scala:321-323).
- `ChildActorPath.withUid(uid)` devuelve `this` si es el mismo uid, si no un hijo nuevo.
- `newUid()`: `ThreadLocalRandom.nextInt` reintentando si da 0; el mismo uid se usa como hashCode de ActorRef (ActorCell.scala:392-398).

**Orden / concurrencia / ciclo de vida.** `newUid` se llama al crear la celda (AC-CELL); aquí sólo el contrato del path.

**Dependencias.** AC-PATH-010, AC-CELL (newUid).

**Equivalente Rust propuesto y diferencias observables.** `uid: i32` en el nodo hijo; `new_uid()` con RNG local (se implementa en AC-CELL, la fila queda `partial`).

**Aserciones.**
- A1 `fromString(path.toString) == path` (el toString no lleva uid): ActorPathSpec.scala:25-28.
- A2 `nombre#uid`, signos, desborde y vacío en el uid: oráculo FS-36..56, CH-*, UID-* (*no afirmados por spec*).
- A3 el mensaje de IllegalStateException de RootActorPath: derivada del código ActorPath.scala:322-323 (*no afirmada por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-6


---

## AC-PATH-012 — RootActorPath: nombre sin separadores

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** `RootActorPath(address, name = "/")` no admite separadores de path ni fragmentos en el nombre (salvo el `/` inicial).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:307-313`

**Condiciones.**
- `require(name.length == 1 || name.indexOf('/', 1) == -1, "/ may only exist at the beginning of the root actors name, it is a path separator and is not legal in ActorPath names: [%s]")` -> IllegalArgumentException con prefijo `requirement failed: `.
- `require(name.indexOf('#') == -1, "# is a fragment separator and is not legal in ActorPath names: [%s]")`.
- Pero `ActorPath.fromString("pekko://mysys/user/boom/*")` sigue funcionando (el `*` es válido).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-010.

**Equivalente Rust propuesto y diferencias observables.** `ActorPath::root_named(address, name) -> Result<_, IllegalArgumentError>` (implementado; RT-* del oráculo).

**Aserciones.**
- A1 `RootActorPath(Address("pekko","mysys"), "/user/boom/*")` lanza IllegalArgumentException con `is a path separator`: ActorPathSpec.scala:108-111.
- A2 crear el path equivalente por `fromString` funciona: ActorPathSpec.scala:113-114.
- A3 mensaje de `#`: oráculo RT-* (*no afirmada por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Ninguna.


---

## AC-PATH-013 — Representaciones de texto: toString, toStringWithoutAddress, toStringWithAddress, toSerializationFormat(WithAddress)

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle+spec*

**Contrato observable.** Representaciones de texto: `toString`, `toStringWithoutAddress`, `toStringWithAddress(addr)`, `toSerializationFormat` y `toSerializationFormatWithAddress(addr)`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:261-285,325-333,397-442,477-480`

**Condiciones.**
- `toString`: `address.toString + name` para la raíz; `/`-unión para hijos. `toStringWithoutAddress = elements.mkString("/", "/", "")` (la raíz da `/`).
- `toStringWithAddress(addr)` reemplaza la dirección de la raíz por `addr` salvo que la dirección propia tenga host (alcance global): entonces conserva la suya. El path `IgnoreActorRef.path` (`pekko://local/ignore`) NUNCA cambia de dirección (ChildActorPath.toStringWithAddress, ActorPath.scala:388-390).
- `toSerializationFormat*` = igual más `#uid` si `uid != 0` (ActorPath.scala:449-452).
- Raíz con nombre distinto de `/` concatena el nombre sin separador (oráculo STR-09).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-011, AC-PATH-015.

**Equivalente Rust propuesto y diferencias observables.** `Display` + métodos explícitos (implementado).

**Aserciones.**
- A1 toString: `pekko://mysys/`, `pekko://mysys/user`, `.../user/foo`, `.../user/foo/bar`: ActorPathSpec.scala:58-64.
- A2 toStringWithoutAddress: `/`, `/user`, `/user/foo`, `/user/foo/bar`: ActorPathSpec.scala:71-77.
- A3 toStringWithAddress con direcciones locales y globales (la propia con host se conserva): ActorPathSpec.scala:84-106 (la línea 94 está comentada en el spec; el oráculo SPEC-APS-94 muestra `pekko://mysys@bb:7355/`).
- A4 IgnoreActorRef y uid en serialización: oráculo STR-11..18 (*no afirmados por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Ninguna.


---

## AC-PATH-014 — Igualdad, hash y orden de ActorPath (ignoran uid)

*unidad PATH · visibilidad public · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Dos `ActorPath` son iguales si tienen mismos nombres y mismas raíces (address + name); el uid no cuenta. `compareTo` ordena por elementos de derecha a izquierda; `RootActorPath > ChildActorPath`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:179-198 (Scaladoc)`
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:335-340`
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:482-522`

**Condiciones.**
- `ChildActorPath.equals` compara nombres hacia arriba hasta la raíz (iterativo) y después `RootActorPath.equals` (case class: address + name).
- `hashCode`: MurmurHash sobre los nombres de CADA nivel (usa `name` del hijo en cada paso; ActorPath.scala:473-476), no se reproduce el valor (Q-PATH-3).
- `compareTo`: compara nombre por nombre de derecha a izquierda; raíz vs hijo: raíz es mayor (1); raíces: `toString.compareTo` (FIXME en el código), por lo que dos raíces con el mismo toString y distinta Address comparan igual sin ser iguales (oráculo CMPP-09..11).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-010.

**Equivalente Rust propuesto y diferencias observables.** `impl PartialEq/Eq/Hash` sin uid; sin `Ord`; `ActorPath::pekko_compare` (decisión del Owner 2026-10-30, ya aplicada).

**Aserciones.**
- A1 igualdad ignorando el uid: oráculo CMPP-*, EQ (*no afirmada por spec*; hallazgo del código ActorPath.scala:454-466).
- A2 orden por elementos de derecha a izquierda y raíz mayor: derivada del Scaladoc ActorPath.scala:157-161 y oráculo CMPP-*.
- A3 inconsistencia compareTo/equals con raíces: oráculo CMPP-09..11 (*no afirmada por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Q-PATH-3, Q-PATH-6


---

## AC-PATH-015 — Paths especiales del proveedor local: "/", "/user", "/system", "/deadLetters", "/temp/$x", "/bubble-walker", ignore ("pekko://local/ignore")

*unidad PATH · visibilidad internal-required · analysis=extracted · impl=partial · validation=oracle*

**Contrato observable.** Paths especiales del proveedor local: `/` (raíz), `/user`, `/system`, `/deadLetters`, `/temp/$x`, `/bubble-walker` e `ignore` (`pekko://local/ignore`).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:409 (rootPath = RootActorPath(Address("pekko", systemName)))`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:421-424 (deadLetters = rootPath / "deadLetters")`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:435-449 (tempNode = rootPath / "temp"; tempPath(prefix) = prefix + "$" + base64(contador))`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:458 (bubble-walker)`
- `actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:513-534 (IgnoreActorRef.path = RootActorPath(Address("pekko", "local")) / "ignore"; isIgnoreRefPath)`

**Condiciones.**
- `tempPath(prefix)` = `/temp/<prefix>$<base64(n)>` con n incremental (AtomicLong) (AC-UTIL-002.1).
- El nombre del actor del destino se codifica en el nombre temporal de `ask`: `promiseActorRef.path.name` empieza con `myName` (AskSpec.scala:248-262).
- Mensajes a `/` o `/user` o `/system` que los guardianes no manejan se convierten en `UnhandledMessage` con el aviso `unhandled message from Actor[<deadLetters path>]: <mensaje>` (LocalActorRefProviderSpec.scala:70-100).

**Orden / concurrencia / ciclo de vida.** El contador de temp paths es atómico; /ignore es único.

**Dependencias.** AC-UTIL-002.1, AC-REF-*, AC-SYS-*.

**Equivalente Rust propuesto y diferencias observables.** Constantes/funciones en `provider::local`; AC-PATH sólo cubre `pekko://local/ignore`: los demás nacen en AC-REF/AC-SYS (fila `partial`).

**Aserciones.**
- A1 `ask` a `myName` crea un path temporal cuyo nombre empieza con `myName`: AskSpec.scala:248-262.
- A2 el guardián raíz/usuario/sistema avisa mensajes no manejados con el texto indicado: LocalActorRefProviderSpec.scala:70-100 (se leyó hasta la línea 99).
- A3 `isIgnoreRefPath`: oráculo STR-11..18 (*no afirmada por spec*).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec.

**Decisiones.** Ninguna.


---

## AC-PATH-016 — ChildActorPath: validación del nombre en el constructor (`/` y `#` prohibidos)

*unidad PATH · visibilidad public · analysis=extracted · impl=done · validation=oracle*

**Contrato observable.** Invariantes del nombre de un ChildActorPath.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:356-363`
- `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:318-321,367-370 (`/` ya separó nombre y uid)`

**Condiciones.**
- Errores con mensajes exactos sin prefijo `requirement failed: ` (usa `throw new IllegalArgumentException`).
- Vía `/`, un nombre con `#` se divide en nombre y uid antes de construir; con `/` en el nombre, sólo es posible construyendo directamente (interno).

**Orden / concurrencia / ciclo de vida.** Pura.

**Dependencias.** AC-PATH-010, AC-PATH-011.

**Equivalente Rust propuesto y diferencias observables.** Validación en `ActorPath::child_with_uid`, que es `pub` con `#[doc(hidden)]` (como `uid`/`with_uid`), no forma parte de la API documentada. El path de una selección sin coincidencia no pasa por él: Q-REF-8 (propuesta; ver AC-REF-014).

**Aserciones.**
- A1 mensajes y condiciones: derivadas del código ActorPath.scala:356-363 (*no afirmadas por spec*; ActorPathSpec sólo prueba el RootActorPath, :108-115).

**Oráculo.** Sí, con jars oficiales Pekko 1.7.1 + JDK 11 (tools/oracle/path-address.jsh, golden crates/rakka-actor/tests/oracle/path-address.golden, 405 líneas); NO es Pekko compilado de upstream/pekko. Los ids SPEC-APS-<n> corresponden a la línea <n> de ActorPathSpec. Escenario CH-* del golden cubre construcciones de hijos.

**Decisiones.** Ninguna.


---

## AC-EXT-001.1 — Extension / ExtensionId / AbstractExtensionId / ExtensionIdProvider: identidad por objeto y acceso `apply`/`get`

*unidad EXT · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `ExtensionId[T].apply(system)` devuelve la instancia de la extensión de ese sistema (la crea la primera vez); `get` es la versión Java; `ExtensionIdProvider.lookup` da el `ExtensionId` canónico para cargas por config. La búsqueda es por IDENTIDAD del ExtensionId.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/Extension.scala:74-143`
- docs/src/main/paradox/extending-pekko.md

**Condiciones.**
- `apply(null)` -> NullPointerException (`requireNonNull(system, "system must not be null!")`, Extension.scala:86-88).
- `hashCode = System.identityHashCode(this)`, `equals = (this eq other)`, ambos `final` (líneas 123-124): dos instancias del mismo ExtensionId son extensiones distintas.
- `apply(ClassicActorSystemProvider)` delega en `apply(system.classicSystem)`.
- Java: `AbstractExtensionId` es la base; `get(system)` puede sobreescribirse para el tipo de retorno.

**Orden / concurrencia / ciclo de vida.** La creación ocurre en `registerExtension` (AC-EXT-001.2).

**Dependencias.** AC-SYS-012, AC-EXT-001.2.

**Equivalente Rust propuesto y diferencias observables.** `trait ExtensionId` con `fn apply(&self, system) -> Arc<Self::Ext>`; identidad por `TypeId` del ExtensionId singleton o por dirección de un `static` (Q-EXT-1).

**Aserciones.**
- A1 `system.hasExtension(TestExtension)`, `TestExtension(system).system == system`, `system.extension(TestExtension).system == system` cuando está en `pekko.extensions`: ExtensionSpec.scala:70-77.
- A2 desde Java: `hasExtension`, `extension(...).system` y `apply(system).system` son el mismo sistema: JavaExtension.java:83-88.
- A3 una extensión ad-hoc (no listada en config) se crea al pedirla: JavaExtension.java:90-93.
- A4 identidad por objeto: derivada del código Extension.scala:123-124 (*no afirmada por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-EXT-1


---

## AC-EXT-001.2 — ActorSystemImpl.registerExtension: una instancia por sistema, registro en curso, excepción cacheada, timeout de creación

*unidad EXT · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `registerExtension(ext)` devuelve la instancia existente o la crea una sola vez por sistema aun con llamadas concurrentes; si la creación falla, la excepción queda registrada y se relanza igual en cada intento posterior.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1196-1252`

**Condiciones.**
- Estado por ExtensionId: `CountDownLatch` (registro en curso), `Throwable` (falló) o la extensión (líneas 1167-1171).
- `findExtension` con latch: espera `creation-timeout`; si no termina: `IllegalStateException("Initialization of [<ext>] took more than [<ms> ms]. " + (if SerializationExtension "A serializer must not access the SerializationExtension from its constructor. Use lazy init." else "Could be deadlock due to cyclic initialization of extensions."))` (líneas 1177-1189).
- `createExtension` devuelve null -> `IllegalStateException("Extension instance created as 'null' for extension [<ext>]")` (líneas 1202-1205).
- Si la creación lanza: `extensions.replace(ext, latch, t)` y se relanza `t`; los demás intentos relanzan `t` (línea 1186: `case t: Throwable => throw t`).
- Los efectos laterales de la creación fallida NO se revierten (el actor `uniqueName` queda creado: comentario en ExtensionSpec.scala:57-59).

**Orden / concurrencia / ciclo de vida.** El latch se decrementa siempre en `finally`; los reintentos de quien llega durante el registro esperan y releen.

**Dependencias.** AC-CFG-009 (creation-timeout), AC-EXT-001.1, AC-SYS-012.

**Equivalente Rust propuesto y diferencias observables.** Estado `Pending | Failed(Arc<Error>) | Ready(Arc<dyn Any>)` con condvar o `OnceLock`; el fallo debe cachearse (Q-EXT-2).

**Aserciones.**
- A1 primer intento de `FailingTestExtension(system)` lanza TestException; el segundo lanza la misma excepción: ExtensionSpec.scala:82-97.
- A2 una instancia por sistema con múltiples nombres en config (duplicados): ExtensionSpec.scala:135-151 (cuenta == 1).
- A3 mensajes de timeout/null: derivados del código ActorSystem.scala:1182-1187,1203-1205 (*no afirmados por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-EXT-2, Q-X-4


---

## AC-EXT-002 — extension() y hasExtension()

*unidad EXT · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** `extension(ext)` devuelve la extensión ya registrada o lanza; `hasExtension(ext)` indica si está registrada (esperando si está en curso).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1254-1259`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:727-741 (firmas en ExtendedActorSystem/ActorSystem)`

**Condiciones.**
- `extension(ext)` no registrada -> `IllegalArgumentException("Trying to get non-registered extension [<ext>]")` (línea 1224).
- `hasExtension(ext)` = `findExtension(ext) != null` (puede bloquear hasta `creation-timeout` si hay registro en curso y lanza la excepción cacheada si falló).
- No crea la extensión.

**Orden / concurrencia / ciclo de vida.** Como `findExtension` (AC-EXT-001.2).

**Dependencias.** AC-EXT-001.2.

**Equivalente Rust propuesto y diferencias observables.** `fn extension<E>() -> Option<Arc<E>>` o `Result`; `has_extension`.

**Aserciones.**
- A1 `hasExtension` true y `extension(...)` devuelve la instancia con `.system == system`: ExtensionSpec.scala:75-77; JavaExtension.java:85-86.
- A2 `extension` de una no registrada lanza IllegalArgumentException con el texto indicado: derivada del código ActorSystem.scala:1223-1226 (*no afirmada por spec*).

**Oráculo.** No aplica.

**Decisiones.** Q-EXT-1


---

## AC-EXT-003 — Carga automática desde config (library-extensions / extensions)

*unidad EXT · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Al arrancar el sistema se cargan las extensiones listadas en `pekko.library-extensions` (todas, y un fallo aborta el arranque) y `pekko.extensions` (un fallo se loguea y se sigue).

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1261-1291`
- `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1104 (loadExtensions en el arranque)`
- `actor/src/main/resources/reference.conf:70-83`

**Condiciones.**
- Cada FQCN se resuelve con `getObjectFor` (objeto Scala `Name$` o clase) y, si falla, con `createInstanceFor(fqcn, Nil)` (constructor sin args), y si falla ambas se informa el primer problema (líneas 1238-1242).
- Si el resultado es `ExtensionIdProvider` -> `registerExtension(p.lookup)`; si es `ExtensionId` -> `registerExtension(p)`; otra cosa -> en library-extensions `RuntimeException("[<fqcn>] is not an 'ExtensionIdProvider' or 'ExtensionId'")`, en extensions `log.error("[{}] is not an 'ExtensionIdProvider' or 'ExtensionId', skipping...")` (líneas 1243-1250).
- Si no se pudo cargar: library -> `RuntimeException("While trying to load extension [<fqcn>]", problem)`; extensions -> `log.error(problem, "While trying to load extension [{}], skipping...", fqcn)` (líneas 1251-1253); texto probado: `While trying to load extension [org.apache.pekko.actor.MissingExtension], skipping.` (ExtensionSpec.scala:117-118).
- Una extensión de `pekko.extensions` que SÍ se encuentra pero falla al inicializarse hace fallar el arranque (la excepción de `registerExtension` no se captura; ExtensionSpec.scala:99-109); en cambio una que no se encuentra sólo se loguea. El comentario del código ('when the extension cannot be found at all we throw regardless of this setting', ActorSystem.scala:1232-1235) contradice el código y el test ExtensionSpec.scala:111-120: gana el código.

**Orden / concurrencia / ciclo de vida.** library-extensions primero, luego extensions; en orden de lista; antes de `start()` devuelva el sistema.

**Dependencias.** AC-CFG-006, AC-CFG-016, AC-EXT-001.2.

**Equivalente Rust propuesto y diferencias observables.** Registro nombre -> `ExtensionId` (fábrica); las claves de reference.conf (`...SerializationExtension$`) preinstaladas (Q-X-2, Q-CFG-8).

**Aserciones.**
- A1 extensión listada en `pekko.extensions` queda registrada: ExtensionSpec.scala:70-77.
- A2 si falla al iniciar -> el sistema no arranca (RuntimeException): ExtensionSpec.scala:99-109.
- A3 inexistente en `extensions` -> log de error `While trying to load extension [...MissingExtension], skipping.` y sigue: ExtensionSpec.scala:111-120.
- A4 library-extensions de reference.conf se cargan una sola vez (InstanceCountingExtension) y duplicados no crean instancias extra: ExtensionSpec.scala:122-151.
- A5 library-extension que falla (`TestException`) o no existe aborta: ExtensionSpec.scala:153-172.
- A6 Java: `pekko.extensions = ["...JavaExtension$TestExtensionId"]` carga una clase con constructor sin args: JavaExtension.java:73-88.

**Oráculo.** No aplica.

**Decisiones.** Q-X-2, Q-CFG-8


---

## AC-EXT-004 — Extensiones provistas por pekko-actor (índice de ExtensionId concretos)

*unidad EXT · visibilidad public · analysis=extracted · impl=pending · validation=pending*

**Contrato observable.** Índice de las 11 extensiones que define el módulo `actor` y de su propiedad común: cada una es un `object` que extiende `ExtensionId[T]` (y, salvo `LogExt`, `ExtensionIdProvider` con `lookup` = el propio objeto), con `createExtension(system)` y, salvo `LogExt`/`AddressTerminatedTopic` en parte, sobrecargas `get(ActorSystem)`/`get(ClassicActorSystemProvider)` para Java. La instancia es única por `ActorSystem` y por identidad del `ExtensionId` (AC-EXT-001.1/.2). Sólo `SerializationExtension` se carga al arrancar (`pekko.library-extensions`, `reference.conf:78`); el resto se crea de forma perezosa en el primer `apply`/`get`/`extension`.

**Fuente.**
- `actor/src/main/scala/org/apache/pekko/actor/CoordinatedShutdown.scala:41,190-202` (`createExtension` lee `pekko.coordinated-shutdown`, construye las fases y llama a `init`)
- `actor/src/main/scala/org/apache/pekko/serialization/SerializationExtension.scala:24-29` (`new Serialization(system)`)
- `actor/src/main/scala/org/apache/pekko/io/Tcp.scala:49-61`, `Udp.scala:44-54`, `UdpConnected.scala:43-53`, `Dns.scala:76,151-159` (cada una `new XExt(system)`)
- `actor/src/main/scala/org/apache/pekko/pattern/CircuitBreakersRegistry.scala:35-60`
- `actor/src/main/scala/org/apache/pekko/event/AddressTerminatedTopic.scala:38-46` (`private[pekko]`; `new AddressTerminatedTopic`)
- `actor/src/main/scala/org/apache/pekko/actor/TypedActor.scala:119-125 (1.7.1)` (deprecada desde Akka 2.6.0)
- `actor/src/main/scala/org/apache/pekko/util/ManifestInfo.scala:33-49`
- `actor/src/main/scala/org/apache/pekko/event/Logging.scala:480-490` (`LogExt`, `private[pekko]`, sin `ExtensionIdProvider`; `id()` incrementa un `AtomicInteger`)
- carga: `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:1261-1290`
- tests: `actor-tests/src/test/scala/org/apache/pekko/actor/ExtensionSpec.scala` (sólo extensiones de prueba)

**Precondiciones.** Las de cada extensión (por ejemplo, `Dns` y los `*Ext` de IO necesitan los gestores del sistema).

**Comportamiento.**

| Extensión (objeto) | Tipo creado | Cuándo se crea | Fila dueña del comportamiento |
|---|---|---|---|
| `CoordinatedShutdown` | `CoordinatedShutdown` | perezosa en su primer `apply`; `ActorSystemImpl` la consulta al terminar (`ActorSystem.scala:1099,1120`) | AC-CS-* |
| `SerializationExtension` | `Serialization` | al arrancar (library-extension) | AC-SER-014 |
| `Tcp` / `Udp` / `UdpConnected` | `TcpExt` / `UdpExt` / `UdpConnectedExt` | perezosa | AC-IO-002..007 |
| `Dns` | `DnsExt` | perezosa | AC-IO-008, AC-IO-009 |
| `CircuitBreakersRegistry` | `CircuitBreakersRegistry` | perezosa | AC-ASK-013 |
| `AddressTerminatedTopic` | `AddressTerminatedTopic` | perezosa (watch de refs remotos) | AC-DW-012 |
| `TypedActor` | `TypedActorExtension` | perezosa | AC-JAPI-009.3 |
| `ManifestInfo` | `ManifestInfo` | perezosa (la usan los chequeos de versión) | AC-UTIL-016.9 |
| `Logging.LogExt` | `LogExt` | perezosa (ids de loggers) | AC-EVT-* |

- Reglas comunes (de `loadExtensions`, `ActorSystem.scala:1237-1256`): para `pekko.library-extensions` un fallo al cargar/instanciar lanza `RuntimeException(s"While trying to load extension [fqcn]", causa)` o `RuntimeException(s"[fqcn] is not an 'ExtensionIdProvider' or 'ExtensionId'")`; para `pekko.extensions` el fallo se registra con `log.error` y se omite.

**Errores.** Los de cada extensión; los de carga por configuración arriba.

**Orden / concurrencia / ciclo de vida.** Registro bajo el lock de `registerExtension` (AC-EXT-001.2); `SerializationExtension` se carga antes de que arranque cualquier actor del usuario.

**Dependencias.** AC-EXT-001.1, AC-EXT-001.2, AC-EXT-002, AC-EXT-003, AC-CFG-006, y las filas dueñas de la tabla.

**Equivalente Rust propuesto y diferencias observables.** Cada extensión como un tipo con un `ExtensionId` único (decisión Q-EXT-1) y una función `create(system)`; la tabla es el índice de qué crates/módulos las aportan. Como la identidad del `ExtensionId` es por tipo (Q-EXT-1 a), `ExtensionId` genérico por tipo cubre las 11. Q-EXT-3 decide si esta fila se conserva como índice o se reparte: esta fila **no define comportamiento propio**; se cierra cuando se cierren las filas dueñas (propuesta: (a), mantenerla como índice con `impl` derivado de las filas hijas).

**Aserciones.**
- A1 *no afirmada por spec* (índice): ExtensionSpec sólo prueba extensiones de prueba (ver AC-EXT-001.2/AC-EXT-003); la existencia y la forma de cada `ExtensionId` es la de las fuentes citadas (derivada del código).
- A2 que `SerializationExtension` esté registrada por defecto en `library-extensions` lo cubren AC-EXT-003 y AC-CFG-006.

**Oráculo / validación prevista.** Ninguna propia; cada extensión se valida en su fila dueña. Prueba de integración propuesta cuando existan ≥ 2 extensiones: crear un sistema y comprobar que `Serialization` ya está creada y que las demás no lo están hasta el primer `get`.

**Decisiones.** Q-EXT-3 (índice vs reparto), Q-EXT-1.


---

## Descomposiciones

El padre desaparece del TSV; los hijos heredan `impl`/`validation` sólo cuando los tests existentes lo demuestran (UTIL-002/003/004 con helpers_spec, done_spec, pekko_exception_spec y el oráculo `util-core`); el resto queda `pending`.

- `AC-UTIL-002` (Helpers: base64 para nombres generados, ConfigOps (getMillisDuration/getNanosDuration), Re) -> `AC-UTIL-002.1`, `AC-UTIL-002.2`, `AC-UTIL-002.3`, `AC-UTIL-002.4`, `AC-UTIL-002.5`
- `AC-UTIL-003` (Done y NotUsed) -> `AC-UTIL-003.1`, `AC-UTIL-003.2`
- `AC-UTIL-004` (PekkoException, ConfigurationException, OnlyCauseStackTrace) -> `AC-UTIL-004.1`, `AC-UTIL-004.2`, `AC-UTIL-004.3`
- `AC-UTIL-005` (PekkoVersion.require, UnsupportedPekkoVersion, util.Version) -> `AC-UTIL-005.1`, `AC-UTIL-005.2`
- `AC-UTIL-006` (ByteString / CompactByteString / ByteStringBuilder / ByteIterator) -> `AC-UTIL-006.1`, `AC-UTIL-006.2`, `AC-UTIL-006.3`, `AC-UTIL-006.4`, `AC-UTIL-006.5`
- Cierre documental 2026-10-31: `AC-UTIL-006` gana tres hijos nuevos por comportamientos independientes: `AC-UTIL-006.6` (factorías y conversiones), `AC-UTIL-006.7` (búsqueda) y `AC-UTIL-006.8` (serialización Java); `AC-UTIL-006.1` queda como estructura, recortes, igualdad y `toString`. La referencia a `AC-UTIL-006` en `depends_on` sigue significando todos sus hijos.
- `AC-UTIL-015` (Switch / ReentrantGuard (LockUtil)) -> `AC-UTIL-015.1`, `AC-UTIL-015.2`
- `AC-UTIL-016` (Helpers internos: OptionVal, ConstantFun, Collections, ImmutableIntMap, PrettyDuration, Gl) -> `AC-UTIL-016.1`, `AC-UTIL-016.2`, `AC-UTIL-016.3`, `AC-UTIL-016.4`, `AC-UTIL-016.5`, `AC-UTIL-016.6`, `AC-UTIL-016.7`, `AC-UTIL-016.8`, `AC-UTIL-016.9`, `AC-UTIL-016.10`, `AC-UTIL-016.11`, `AC-UTIL-016.12`, `AC-UTIL-016.13`, `AC-UTIL-016.14`, `AC-UTIL-016.15`, `AC-UTIL-016.16`, `AC-UTIL-016.17`, `AC-UTIL-016.18`
- `AC-UTIL-017` (Estructuras de cache internas: FrequencySketch, FrequencyList, RecencyList, SegmentedRecen) -> `AC-UTIL-017.1`, `AC-UTIL-017.2`, `AC-UTIL-017.3`, `AC-UTIL-017.4`, `AC-UTIL-017.5`
- `AC-UTIL-018` (Shims de versión de Scala y conversores Java: FutureConverters, OptionConverters, Function) -> `AC-UTIL-018.1`, `AC-UTIL-018.2`, `AC-UTIL-018.3`, `AC-UTIL-018.4`, `AC-UTIL-018.5`
- `AC-CFG-012` (pekko.actor.deployment: config por path con comodines (dispatcher, mailbox, router, nr-of-) -> `AC-CFG-012.1`, `AC-CFG-012.2`, `AC-CFG-012.3`, `AC-CFG-012.4`
- `AC-CFG-017` (Formato HOCON usado por reference.conf: sustituciones opcionales, duraciones, tamaños, val) -> `AC-CFG-017.1`, `AC-CFG-017.2`, `AC-CFG-017.3`, `AC-CFG-017.4`, `AC-CFG-017.5`, `AC-CFG-017.6`, `AC-CFG-017.7`
- `AC-EXT-001` (Extension / ExtensionId / AbstractExtensionId / ExtensionIdProvider: una instancia por sis) -> `AC-EXT-001.1`, `AC-EXT-001.2`

## Omisiones agregadas

- `AC-UTIL-023` — Helpers.makePattern (patrón glob `?`/`*` a java.util.regex.Pattern). Evidencia: `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:46-47`.
- `AC-UTIL-024` — Helpers.compareIdentityHash / identityHashComparator (orden por identityHashCode sin overflow). Evidencia: `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:49-71`.
- `AC-UTIL-025` — Helpers.currentTimeMillisToUTCString y Helpers.timestamp (formato de hora para logs). Evidencia: `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:73-115`.
- `AC-UTIL-026` — Helpers.isWindows (detección de SO por os.name). Evidencia: `actor/src/main/scala/org/apache/pekko/util/Helpers.scala:44`.
- `AC-CFG-018` — Settings.amendSlf4jConfig: `pekko.use-slf4j` agrega Slf4jLogger/Slf4jLoggingFilter a loggers. Evidencia: `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:332-378`.
- `AC-CFG-019` — Validación de la config contra reference.conf (`checkValid` con `internal-dispatcher` excluido). Evidencia: `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:396-403`.
- `AC-CFG-020` — Combinación no soportada coordinated-shutdown: run-by-actor-system-terminate=on con terminate-actor-system=off. Evidencia: `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala:473-480`.
- `AC-PATH-016` — ChildActorPath: validación del nombre en el constructor (`/` y `#` prohibidos). Evidencia: `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala:356-363`.
- `AC-UTIL-006.6`, `AC-UTIL-006.7`, `AC-UTIL-006.8` (cierre documental 2026-10-31) — ByteString: factorías/conversiones, búsqueda y serialización Java. Evidencia: `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala:31-161,236-271,451-487,730-806,825-856 (1.7.1)`.
- Hijos nuevos por archivos de `util/` que ninguna fila citaba por ruta (ya listados dentro de AC-UTIL-016 sólo por nombre de clase): `AC-UTIL-016.2` (ConstantFun), `.3` (Collections), `.4` (ImmutableIntMap), `.10` (JavaVersion), `.12` (ClassLoaderObjectInputStream), `.13` (BoxedType), `.14` (UUIDComparator), `.15` (HashCode), `.17` (ErrorMessages), `.18` (Unused) y `AC-UTIL-017.2/.4` (FrequencyList, SegmentedRecencyList): estos archivos aparecían en `uncited-classic-main.txt`.
- Tests sin cita previa asignados: ByteStringInitializationSpec, AsciiStringCopySpec, ByteStringTest.java, ByteStringBuilderScala213PlusSpec -> AC-UTIL-006.*; ConstantFunJavaTest.java -> AC-UTIL-016.2; LineNumberSpec* -> AC-UTIL-016.8; ImmutableIntMapSpec -> AC-UTIL-016.4; DoubleLinkedListSpec, FrequencyListSpec, SegmentedRecencyListSpec, ZipfianGenerator -> AC-UTIL-017.*; AddressTest.java -> AC-PATH-001; JavaExtension.java -> AC-EXT-001.1/AC-EXT-003; ActorSystemSetupTest.java y Scala212CompatTest -> AC-CFG-015 / AC-UTIL-018.3 (leídos en el cierre documental 2026-10-31); JavaConverterScala3InlineTest.java -> AC-UTIL-018.1/.2; `JavaDuration.java` prueba `scala.concurrent.duration.Duration` (biblioteca de Scala) y no es contrato de Pekko.

## Contradicciones halladas y corregidas

- AC-UTIL-015 proponía excluir `Switch` por deprecado: `LockUtil.scala:19-32` marca como `@deprecated` sólo `ReentrantGuard`; `Switch` (líneas 29-155) no lo está. Dividido en `.1` (Switch) y `.2` (ReentrantGuard).
- AC-UTIL-002 enumeraba base64/ConfigOps/Requiring/toRootLowerCase pero `Helpers.scala` también define `makePattern`, `compareIdentityHash`, `identityHashComparator`, `currentTimeMillisToUTCString`, `timestamp` e `isWindows` (omisiones AC-UTIL-023..026).
- AC-UTIL-005 `Version` tiene un defecto de comparación (`Version.scala:170-175`) que el inventario no registraba; VersionSpec no lo cubre (Q-UTIL-6).
- AC-PATH-014 citaba `ActorSelectionSpec.scala:124` como test de igualdad de paths: ese test prueba igualdad de `ActorRef` (incarnation/uid), no de `ActorPath`; ningún spec afirma 'igualdad de paths ignora el uid'. AC-PATH-011 igual (comparte la cita).
- AC-EXT-003 / ActorSystem.scala:1232-1235: el comentario dice que una extensión no encontrada lanza siempre; el código y ExtensionSpec.scala:111-120 muestran que en `pekko.extensions` sólo se loguea. Gana el código.
- AC-CFG-012 agrupaba cuatro comportamientos (parseo, búsqueda con comodines, creación de router, claves de resizers): dividido en `.1`..`.4`.
- AC-CFG-017 agrupaba siete familias de comportamiento del golden de HOCON: dividido en `.1`..`.7` con su familia de ids del golden.
- AC-CFG-013 no registraba que `Deploy.withFallback` descarta los `tags` (Deployer.scala:109-117): Q-CFG-12.
- AC-CFG-001 omitía la regla de carga por defecto `ConfigFactory.load(cl)` (ActorSystem.scala:281) y la mezcla `app.withFallback(defaultReference)` (ActorSystem.scala:853): incorporadas al contrato.
- AC-UTIL-008 citaba EventStreamSpec.scala:150-281 como spec del índice: es un spec de EventStream; no existe spec directo de SubclassifiedIndex, por lo que los diffs devueltos no están afirmados.

- (cierre 2026-10-31) `AC-UTIL-025` decía «sin tests en upstream/pekko ni consumidores en actor/src/main»: `LoggerSpec.scala:282-316` prueba `currentTimeMillisToUTCString` y `Logging.scala:997` usa `Helpers.timestamp`.
- (cierre 2026-10-31) `AC-UTIL-024` decía «sin test en upstream (grep sin resultados)»: `EventBusSpec.scala:309,345` usan `Helpers.compareIdentityHash` (aunque no afirman su valor).
- (cierre 2026-10-31) `AC-CFG-020` decía «sin test»: `CoordinatedShutdownSpec.scala:572-582` afirma el `ConfigurationException` (sólo el tipo, no el texto).
- (cierre 2026-10-31) `AC-UTIL-023`: la nota «Glob.matches acepta saltos de línea» era una hipótesis; se midió con el jar (también con pares sustitutos) y se confirma.
- (cierre 2026-10-31) `AC-UTIL-005.2` / Q-UTIL-6: el defecto de `Version.compareTo` era una hipótesis a confirmar; se midió y se confirma (ambos sentidos negativos).
- (cierre 2026-10-31) `AC-PATH-009`: `ActorSelectionTest.java` no llama a `ActorPaths` (sólo prueba `resolveOne`); la fila se valida únicamente con el oráculo.
- (cierre 2026-10-31) `AC-UTIL-006.8`: la aserción de compatibilidad con Akka (hex de `ByteString1C`) sólo se ejecuta en Scala 2.12; en el árbol fijado (2.13) no se ejecuta.
- (cierre 2026-10-31) `AC-UTIL-010`: `MessageBufferMap.drop` con id ausente lanza `NullPointerException` (medido); el contrato anterior no lo registraba.
- (cierre 2026-10-31) `AC-UTIL-016.15`: `HashCode.hash(seed, null: Any)` lanza `IllegalArgumentException` (la rama `null` es inalcanzable).
- (cierre 2026-10-31) `AC-UTIL-020`: `Unsafe.fastHash` y `copyUSAsciiStrToBytes` no tienen consumidores de producción en el árbol fijado; sólo los usa `AsciiStringCopySpec`.

## Filas que siguen listed

Ninguna. Las 37 filas que estaban `listed` (AC-UTIL-006.1-.5, 010, 012, 013, 014, 016.1/.2/.3/.4/.7/.8/.9/.11/.12/.13/.14/.15/.18, 017.1-.5, 018.1-.5, 019, 020, 021, 022 y AC-EXT-004) pasaron a `extracted` en el cierre documental de 2026-10-31 tras leer la implementación y los cuerpos de los tests citados (y medir con el jar donde el spec no afirma). `extracted` no implica decidido: cada bloque indica qué decisiones del Owner (Q-UTIL-6, -12, -14, -15, -16, Q-CFG-*, Q-EXT-*) bloquean la implementación; ninguna bloquea la extracción del contrato.

## Cobertura

### A. Archivos de `actor/src/main` de mi área y filas que los cubren

| Archivo | Filas |
|---|---|
| `actor/src/main/scala/org/apache/pekko/util/BoundedBlockingQueue.scala` | AC-UTIL-012 |
| `actor/src/main/scala/org/apache/pekko/util/BoxedType.scala` | AC-UTIL-016.13 |
| `actor/src/main/scala/org/apache/pekko/util/ClassLoaderObjectInputStream.scala` | AC-UTIL-016.12 |
| `actor/src/main/scala/org/apache/pekko/util/Collections.scala` | AC-UTIL-016.3 |
| `actor/src/main/scala/org/apache/pekko/util/ConstantFun.scala` | AC-UTIL-016.2 |
| `actor/src/main/scala/org/apache/pekko/util/DoubleLinkedList.scala` | AC-UTIL-017.5 |
| `actor/src/main/scala/org/apache/pekko/util/ErrorMessages.scala` | AC-UTIL-016.17 |
| `actor/src/main/scala/org/apache/pekko/util/FlightRecorderLoader.scala` | AC-UTIL-016.11 |
| `actor/src/main/scala/org/apache/pekko/util/FrequencyList.scala` | AC-UTIL-017.2 |
| `actor/src/main/scala/org/apache/pekko/util/FrequencySketch.scala` | AC-UTIL-017.1 |
| `actor/src/main/scala/org/apache/pekko/util/Glob.scala` | AC-UTIL-016.6 |
| `actor/src/main/scala/org/apache/pekko/util/HashCode.scala` | AC-UTIL-016.15, AC-UTIL-024 |
| `actor/src/main/scala/org/apache/pekko/util/Helpers.scala` | AC-UTIL-002.1, AC-UTIL-002.2, AC-UTIL-002.3, AC-UTIL-002.4, AC-UTIL-002.5, AC-UTIL-023 … |
| `actor/src/main/scala/org/apache/pekko/util/ImmutableIntMap.scala` | AC-UTIL-016.4 |
| `actor/src/main/scala/org/apache/pekko/util/Index.scala` | AC-UTIL-007, AC-UTIL-008, AC-UTIL-009, AC-CFG-012.2 |
| `actor/src/main/scala/org/apache/pekko/util/JavaVersion.scala` | AC-UTIL-016.10 |
| `actor/src/main/scala/org/apache/pekko/util/LineNumbers.scala` | AC-UTIL-016.8 |
| `actor/src/main/scala/org/apache/pekko/util/LockUtil.scala` | AC-UTIL-015.1, AC-UTIL-015.2 |
| `actor/src/main/scala/org/apache/pekko/util/ManifestInfo.scala` | AC-UTIL-016.9 |
| `actor/src/main/scala/org/apache/pekko/util/MessageBuffer.scala` | AC-UTIL-010 |
| `actor/src/main/scala/org/apache/pekko/util/OptionVal.scala` | AC-UTIL-016.1 |
| `actor/src/main/scala/org/apache/pekko/util/PrettyDuration.scala` | AC-UTIL-016.5 |
| `actor/src/main/scala/org/apache/pekko/util/RecencyList.scala` | AC-UTIL-017.3, AC-UTIL-017.4 |
| `actor/src/main/scala/org/apache/pekko/util/Reflect.scala` | AC-UTIL-016.7, AC-CFG-016 |
| `actor/src/main/scala/org/apache/pekko/util/SegmentedRecencyList.scala` | AC-UTIL-017.4 |
| `actor/src/main/scala/org/apache/pekko/util/SerializedSuspendableExecutionContext.scala` | AC-DSP-018 (otra unidad; fuera de mi grupo) |
| `actor/src/main/scala/org/apache/pekko/util/StablePriorityQueue.scala` | AC-UTIL-013 |
| `actor/src/main/scala/org/apache/pekko/util/SubclassifiedIndex.scala` | AC-UTIL-008 |
| `actor/src/main/scala/org/apache/pekko/util/Timeout.scala` | AC-UTIL-001 |
| `actor/src/main/scala/org/apache/pekko/util/TokenBucket.scala` | AC-UTIL-011 |
| `actor/src/main/scala/org/apache/pekko/util/TypedMultiMap.scala` | AC-UTIL-014 |
| `actor/src/main/scala/org/apache/pekko/util/UUIDComparator.scala` | AC-UTIL-016.14 |
| `actor/src/main/scala/org/apache/pekko/util/Unused.scala` | AC-UTIL-016.18 |
| `actor/src/main/scala/org/apache/pekko/util/Version.scala` | AC-UTIL-005.1, AC-UTIL-005.2, AC-UTIL-016.10 |
| `actor/src/main/scala/org/apache/pekko/util/WallClock.scala` | AC-UTIL-016.16 |
| `actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala` | AC-UTIL-009, AC-CFG-012.2 |
| `actor/src/main/scala-2.13/org/apache/pekko/util/ByteIterator.scala` | AC-UTIL-006.4 |
| `actor/src/main/scala-2.13/org/apache/pekko/util/ByteString.scala` | AC-UTIL-006.1, AC-UTIL-006.2, AC-UTIL-006.3, AC-UTIL-006.5, AC-UTIL-022 |
| `actor/src/main/scala-2.13/org/apache/pekko/util/FutureConverters.scala` | AC-UTIL-018.1 |
| `actor/src/main/scala-2.13/org/apache/pekko/util/OptionConverters.scala` | AC-UTIL-018.2 |
| `actor/src/main/scala-2/org/apache/pekko/util/JavaDurationConverters.scala` | AC-UTIL-001, AC-UTIL-018.2 |
| `actor/src/main/scala-3/org/apache/pekko/util/ByteIterator.scala` | AC-UTIL-006.4 |
| `actor/src/main/scala-3/org/apache/pekko/util/ByteString.scala` | AC-UTIL-006.1, AC-UTIL-006.2, AC-UTIL-006.3, AC-UTIL-006.5, AC-UTIL-022 |
| `actor/src/main/scala-3/org/apache/pekko/util/FutureConverters.scala` | AC-UTIL-018.1 |
| `actor/src/main/scala-3/org/apache/pekko/util/JavaDurationConverters.scala` | AC-UTIL-001, AC-UTIL-018.2 |
| `actor/src/main/scala-3/org/apache/pekko/util/OptionConverters.scala` | AC-UTIL-018.2 |
| `actor/src/main/java/org/apache/pekko/util/Unsafe.java` | AC-UTIL-020 |
| `actor/src/main/java/org/apache/pekko/annotation/ApiMayChange.java` | AC-UTIL-019 |
| `actor/src/main/java/org/apache/pekko/annotation/DoNotInherit.java` | AC-UTIL-019 |
| `actor/src/main/java/org/apache/pekko/annotation/InternalApi.java` | AC-UTIL-019 |
| `actor/src/main/java/org/apache/pekko/annotation/InternalStableApi.java` | AC-UTIL-019 |
| `actor/src/main/java/org/apache/pekko/io/ByteBufferCleaner.java` | AC-UTIL-022 |
| `actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java` | AC-UTIL-022 |
| `actor/src/main/scala/org/apache/pekko/Done.scala` | AC-UTIL-003.1 |
| `actor/src/main/scala/org/apache/pekko/Main.scala` | AC-SYS-021 (otra unidad; fuera de mi grupo) |
| `actor/src/main/scala/org/apache/pekko/NotUsed.scala` | AC-UTIL-003.2 |
| `actor/src/main/scala/org/apache/pekko/PekkoException.scala` | AC-UTIL-004.1, AC-UTIL-004.2, AC-UTIL-004.3 |
| `actor/src/main/scala/org/apache/pekko/PekkoVersion.scala` | AC-UTIL-005.1 |
| `actor/src/main/scala/org/apache/pekko/actor/Address.scala` | AC-PATH-001, AC-PATH-002, AC-PATH-003, AC-PATH-004, AC-PATH-005, AC-PATH-006 … |
| `actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala` | AC-PATH-005, AC-PATH-006, AC-PATH-007, AC-PATH-008, AC-PATH-009, AC-PATH-010 … |
| `actor/src/main/scala/org/apache/pekko/actor/Extension.scala` | AC-EXT-001.1, AC-EXT-001.2, AC-EXT-002, AC-EXT-004 |
| `actor/src/main/scala/org/apache/pekko/actor/Deployer.scala` | AC-CFG-012.1, AC-CFG-012.2, AC-CFG-012.3, AC-CFG-013 |
| `actor/src/main/scala/org/apache/pekko/actor/DynamicAccess.scala` | AC-CFG-012.3, AC-CFG-016 |
| `actor/src/main/scala/org/apache/pekko/actor/ReflectiveDynamicAccess.scala` | AC-CFG-016 |
| `actor/src/main/scala/org/apache/pekko/actor/setup/ActorSystemSetup.scala` | AC-CFG-015 |
| `actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (BootstrapSetup, ProviderSelection, Settings, extensiones)` | AC-CFG-001..011, AC-CFG-015, AC-CFG-018..020, AC-EXT-001.2, AC-EXT-002, AC-EXT-003 |
| `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:389-403 (undefinedUid, newUid, splitNameAndUid)` | AC-PATH-011 |
| `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:403-462 (paths especiales)` | AC-PATH-015 |
| `actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:513-534 (IgnoreActorRef)` | AC-PATH-015 |
| `actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:172-224, Children.scala:283-307` | AC-CFG-010 |
| `actor/src/main/boilerplate/org/apache/pekko/{function,tuple}/** (plantillas de interfaces funcionales y tuplas Java)` | AC-JAPI-* (otra unidad; fuera de mi grupo, no leído) |
| `actor/src/main/scala-2.12/**, actor/src/main/scala-2.13+/**` | AC-UTIL-018.* (shims de versión de Scala; sin comportamiento observable para Rust) |
| `actor/src/main/mima-filters/**` | AC-UTIL-021 (compatibilidad binaria; sin comportamiento de runtime) |
| `actor/src/main/resources/reference.conf (1443 líneas)` | ver sección B |

### B. `reference.conf`: cada sección/clave y su unidad dueña

Leído completo (1443 líneas). Las claves marcadas con unidad ajena no tienen fila en mi grupo; el dueño es la fila indicada, que debe cubrirlas.

| Líneas | Sección / clave | Unidad dueña | Fila(s) |
|---|---|---|---|
| 10-11 | `include "version"` (pekko.version generado) | CFG | AC-CFG-003, AC-CFG-017.6 |
| 13-15 | `pekko.home` | CFG | AC-CFG-007 |
| 17-19 | `pekko.loggers` | CFG | AC-CFG-004, AC-CFG-018 |
| 21-28 | `pekko.logging-filter` | CFG | AC-CFG-004 (comportamiento del filtro: AC-EVT-016) |
| 30-31 | `pekko.loggers-dispatcher` | CFG | AC-CFG-004 (dispatcher: AC-DSP-006) |
| 33-36 | `pekko.logger-startup-timeout` | CFG | AC-CFG-004 |
| 38-46 | `pekko.loglevel`, `pekko.stdout-loglevel` | CFG | AC-CFG-004 |
| 48-50 | `pekko.log-config-on-start` | CFG | AC-CFG-004 |
| 52-68 | `pekko.log-dead-letters`, `-during-shutdown`, `-suspend-duration` | CFG | AC-CFG-005 (EVT-009) |
| 70-83 | `pekko.library-extensions`, `pekko.extensions` | CFG | AC-CFG-006, AC-EXT-003 |
| 85-110 | `daemonic`, `jvm-exit-on-fatal-error`, `jvm-shutdown-hooks`, `fail-mixed-versions`, `java-flight-recorder.enabled` | CFG | AC-CFG-007 (SYS-015, CS-007) |
| 112-118 | `pekko.actor.provider` | CFG | AC-CFG-008 |
| 120-123 | `guardian-supervisor-strategy` | CFG | AC-CFG-009 (SUP-014) |
| 125-127 | `creation-timeout` | CFG | AC-CFG-009, AC-EXT-001.2 |
| 129-140 | `serialize-messages`, `serialize-creators`, `no-serialization-verification-needed-class-prefix` | CFG | AC-CFG-010 |
| 142-145 | `unstarted-push-timeout` | CFG | AC-CFG-009 (REF-020) |
| 147-152 | `pekko.actor.typed.timeout` | CFG | AC-CFG-009 |
| 154-171 | `pekko.actor.router.type-mapping` | CFG | AC-CFG-014 |
| 173-233 | `deployment.default` (dispatcher, mailbox, router, nr-of-instances, within, virtual-nodes-factor, tail-chopping-router.interval, routees.paths) | CFG | AC-CFG-012.1, AC-CFG-012.3 |
| 235-242 | `pool-dispatcher` (sólo comentario) | CFG/RT | AC-CFG-012.4 (consumidor: AC-RT) |
| 244-290 | `deployment.default.resizer` | CFG/RT | AC-CFG-012.4, AC-RT-013 |
| 291-346 | `deployment.default.optimal-size-exploring-resizer` | CFG/RT | AC-CFG-012.4, AC-RT-014 |
| 349-363 | `deployment` `/IO-DNS/*` | IO | AC-IO-011 (AC-CFG-012.1 para el parseo) |
| 366-608 | `pekko.actor.default-dispatcher` y sub-secciones (executors, throughput, shutdown-timeout, attempt-teamwork, mailbox-requirement) | DSP | AC-DSP-001, 003, 005, 008, 009, 010, 013, 014 |
| 610-630 | `internal-dispatcher`, `default-blocking-io-dispatcher` | DSP | AC-DSP-006 |
| 631-654 | `default-mailbox` (mailbox-type, capacity, push-timeout, stash-capacity) | MBX | AC-MBX-005, 007, 016 |
| 656-735 | `pekko.actor.mailbox` (requirements, unbounded/bounded/deque/control-aware, logger-queue) | MBX/EVT | AC-MBX-014, AC-EVT-020 |
| 736-759 | `pekko.actor.debug.*` | CFG | AC-CFG-011 |
| 760-797 | `allow-java-serialization`, `serialization-max-nesting-depth`, `warn-about-java-serializer-usage`, `warn-on-no-serialization-verification` | SER | AC-SER-004, AC-SER-009 |
| 800-850 | `serializers`, `serialization-bindings`, `serialization-identifiers` | SER | AC-SER-003 |
| 851-863 | `pekko.serialization.max-decompressed-size` | SER | AC-SER-010 (formato `unlimited`: AC-CFG-017.5) |
| 865-880 | `pekko.serialization.protobuf` (whitelist-class, allowed-classes) | SER | AC-SER-012 (sustitución `${...}`: AC-CFG-017.1) |
| 886-927 | `pekko.scheduler` | SCH | AC-SCH-008, AC-SCH-010 |
| 930-938 | `pekko.io.pinned-dispatcher` | IO | AC-IO-011 |
| 940-1026 | `pekko.io.tcp` | IO | AC-IO-003 (tamaños y `unlimited`: AC-CFG-017.3/.5) |
| 1028-1082 | `pekko.io.udp` | IO | AC-IO-006 (`select-timeout = infinite`: AC-CFG-017.5) |
| 1084-1138 | `pekko.io.udp-connected` | IO | AC-IO-007 |
| 1140-1243 | `pekko.io.dns` (inet-address, async-dns) | IO | AC-IO-008, AC-IO-009 (`forever`, `never`, `default`: AC-CFG-017.5) |
| 1245-1400 | `pekko.coordinated-shutdown` (flags, reason-overrides, phases) | CS | AC-CS-001, 002, 005, 006, 009; AC-CFG-020 |
| 1408-1440 | `pekko.circuit-breaker.default` | ASK | AC-ASK-011, 012, 013 |

No existen claves de `reference.conf` sin dueño; `pekko.use-slf4j` no está en este archivo (la define `actor-typed/src/main/resources/reference.conf:68`) y se cubre en AC-CFG-018.

### C. Tests de mi área y filas que los usan

| Test | Filas |
|---|---|
| `actor-tests/.../actor/ActorTimeoutSpec.scala` | AC-UTIL-001 (pertenece a AC-ASK-001) |
| `actor-tests/.../PekkoExceptionSpec.scala` | AC-UTIL-004.1 |
| `actor-tests/.../PekkoVersionSpec.scala` | AC-UTIL-005.1 |
| `actor-tests/.../util/VersionSpec.scala` | AC-UTIL-005.2 |
| `actor-tests/.../util/IndexSpec.scala` | AC-UTIL-007 |
| `actor-tests/.../event/EventStreamSpec.scala:150-281` | AC-UTIL-008 (indirecto) |
| `actor-tests/.../util/WildcardIndexSpec.scala` | AC-UTIL-009, AC-CFG-012.2 |
| `actor-tests/.../util/MessageBufferSpec.scala` | AC-UTIL-010 (listed) |
| `actor-tests/.../util/TokenBucketSpec.scala` | AC-UTIL-011 |
| `actor-tests/.../util/BoundedBlockingQueueSpec.scala` | AC-UTIL-012 (listed) |
| `actor-tests/.../util/SwitchSpec.scala` | AC-UTIL-015.1 |
| `actor-tests/.../util/GlobSpec.scala` | AC-UTIL-016.6, AC-UTIL-023 |
| `actor-tests/.../util/PrettyDurationSpec.scala` | AC-UTIL-016.5 |
| `actor-tests/.../util/DurationSpec.scala` | n/a: prueba `scala.concurrent.duration.Duration` (biblioteca estándar de Scala), no un contrato de Pekko; no se porta (plan AC-UTIL) |
| `actor-tests/.../config/ConfigSpec.scala` | AC-CFG-001..005, 007, 011, 018 |
| `actor-tests/.../actor/DeployerSpec.scala` | AC-CFG-012.*, 013, 014 |
| `actor-tests/.../actor/ProviderSelectionSpec.scala` | AC-CFG-008 |
| `actor-tests/.../actor/setup/ActorSystemSetupSpec.scala` | AC-CFG-015 |
| `actor-tests/.../actor/DynamicAccessSpec.scala` | AC-CFG-016 |
| `actor-tests/.../actor/ExtensionSpec.scala, java/.../JavaExtension.java` | AC-EXT-001.*, 002, 003, AC-CFG-006 |
| `actor-tests/.../actor/ActorPathSpec.scala` | AC-PATH-001, 003..008, 010..013 |
| `actor-tests/.../actor/AddressSpec.scala, java/.../AddressTest.java` | AC-PATH-001, 002 |
| `actor-tests/.../actor/RelativeActorPathSpec.scala` | AC-PATH-005 |
| `actor-tests/.../actor/LocalActorRefProviderSpec.scala:70-100,160-200` | AC-PATH-008, 015 |
| `actor-tests/.../pattern/AskSpec.scala:248-262` | AC-PATH-015 |
| `actor-tests/.../serialization/SerializeSpec.scala:310-361, actor/dungeon/DispatchSpec.scala` | AC-CFG-010 |
| `actor-tests/.../event/LoggerSpec.scala, LoggingReceiveSpec.scala, actor/DeadLetterSuspensionSpec.scala, actor/CoordinatedShutdownSpec.scala:623-663, actor/ActorSystemSpec.scala:328-381` | AC-CFG-004, 005, 007, 009, 011 |
| `ByteStringSpec, ByteStringBuilderSpec, ByteStringBuilderScala213PlusSpec, ByteIteratorSpec, ByteStringInputStreamSpec, ByteStringInitializationSpec, ByteStringTest.java, AsciiStringCopySpec, ImmutableIntMapSpec, FrequencySketchSpec, FrequencyListSpec, RecencyListSpec, SegmentedRecencyListSpec, DoubleLinkedListSpec, ReflectSpec, ManifestInfoSpec, LineNumberSpec, ConstantFunJavaTest, SameThreadExecutionContextSpec, ByteBufferCleanerSpec, UnsynchronizedByteArrayInputStreamSpec, TypedMultiMapSpec, BoundedBlockingQueueSpec, MessageBufferSpec, StablePriorityDispatcherSpec, Scala212CompatTest, JavaConverterScala3InlineTest` | AC-UTIL-006.*, 016.*, 017.*, 018.*, 022 (extracted en el cierre documental 2026-10-31) |

### D. Archivos que no definen comportamiento observable

- `actor/src/main/java/org/apache/pekko/annotation/*.java` (AC-UTIL-019): anotaciones de estabilidad de API, sin efecto en runtime.
- `actor/src/main/mima-filters/**` (AC-UTIL-021): filtros de compatibilidad binaria del build.
- `util/Unused.scala`, `util/BoxedType.scala`, `util/OptionVal.scala` (AC-UTIL-016.18/.13/.1): optimizaciones o supresión de avisos del compilador de Scala; su efecto observable y su representación Rust están en los bloques de esas filas (leídos en el cierre documental 2026-10-31); ninguna se excluye.
- `actor/src/main/scala-2.12/**` y `scala-2.13+/ccompat`: shims para versiones de Scala que Rust no tiene (AC-UTIL-018.3).

*Totales del grupo (cierre documental 2026-10-31):* 117 filas (extracted 117, listed 0); 36 decisiones en `classic-foundation.decisions.tsv` (3 nuevas: Q-UTIL-14, Q-UTIL-15, Q-UTIL-16).


## Revisión de exhaustividad (cierre documental 2026-10-31)

Criterio: se contrastó el árbol de fuentes de este grupo (`actor/src/main/{scala,scala-2,scala-2.12,scala-2.13,scala-2.13+,scala-3,java,mima-filters,resources}` en lo que toca a UTIL, CFG, PATH y EXT; `actor-tests/src/test/{scala,scala-2,scala-2.13+,java}` bajo `util/`, `config/`, y los specs de `actor/` de path, address, extensión, deployer, dynamic access y setup) con las 117 filas del grupo. Las clasificaciones usan las categorías del cierre; «Pendiente de análisis» **no** se convierte en «fuera de alcance».

| Elemento revisado | Clasificación | Fila(s) / justificación |
|---|---|---|
| `util/{Timeout,Helpers,Version,PekkoVersion,Done,NotUsed,PekkoException}` y sus specs | Cubierto por contratos concretos | AC-UTIL-001..005, 023..026 (filas `extracted` previas; auditadas sólo las que citaban «no leído/derivada», ver «Contradicciones halladas») |
| `util/ByteString.scala` (2.13 y 3), `ByteIterator.scala`, `Unsynchronized…`, `ByteBufferCleaner` | Cubierto por contratos concretos | AC-UTIL-006.1-.8, 022; `ByteString.Companion`/`SerializationProxy` van en 006.8, `ByteArrayIterator`/`MultiByteArrayIterator` en 006.4 |
| `util/{MessageBuffer,BoundedBlockingQueue,StablePriorityQueue,TypedMultiMap}` | Cubierto por contratos concretos | AC-UTIL-010, 012, 013, 014 |
| `util/{OptionVal,ConstantFun,Collections,ImmutableIntMap,Reflect,LineNumbers,ManifestInfo,FlightRecorderLoader,ClassLoaderObjectInputStream,BoxedType,UUIDComparator,HashCode,Unused}` | Cubierto por contratos concretos | AC-UTIL-016.1-.4, .7-.9, .11-.15, .18 (los tipos sin equivalente JVM tienen efecto observable y representación Rust; ninguno se excluye) |
| `util/{FrequencySketch,FrequencyList,RecencyList,SegmentedRecencyList,DoubleLinkedList}` | Cubierto por contratos concretos | AC-UTIL-017.1-.5 |
| `scala-2.12/**`, `scala-2.13+/**`, `scala-2.13/{Future,Option}Converters`, `scala-2/JavaDurationConverters`, `dispatch/internal/{SameThreadExecutionContext,ScalaBatchable}` | Cubierto por contratos concretos | AC-UTIL-018.1-.5 (shims de versión; efecto observable: `Timeout.create` y `parasitic`) |
| `annotation/*.java`, `mima-filters/**`, `util/Unsafe.java` y las 9 clases `Abstract*` que lo usan | Cubierto por contratos concretos | AC-UTIL-019, 021, 020 (las estructuras que usan los atómicos se contratan en AC-CELL/AC-REF/AC-MBX/AC-DSP/AC-ASK) |
| `util/{Index,SubclassifiedIndex,WildcardIndex,TokenBucket,LockUtil,PrettyDuration,Glob,JavaVersion,WallClock,ErrorMessages}` | Cubierto por contratos concretos | AC-UTIL-007..009, 011, 015.1/.2, 016.5/.6/.10/.16/.17 (filas previas) |
| `util/SerializedSuspendableExecutionContext.scala` | Cubierto por contratos concretos (otro grupo) | AC-DSP-018 |
| `actor/{ActorPath,Address}.scala`, `ActorCell.scala:390-404`, `IgnoreActorRef` | Cubierto por contratos concretos | AC-PATH-001..016 (filas previas; sin re-verificar línea a línea en este cierre) |
| `actor/{ActorSystem (Settings, BootstrapSetup, ProviderSelection, extensiones), Deployer, DynamicAccess, ReflectiveDynamicAccess, setup/ActorSystemSetup}.scala` y `reference.conf` | Cubierto por contratos concretos | AC-CFG-001..020, AC-EXT-001..003 (sección B de «Cobertura» asigna cada clave; sin re-verificar línea a línea) |
| `actor/Extension.scala` y las 11 extensiones concretas del módulo | Cubierto por contratos concretos | AC-EXT-001.1/.2/002/003 y AC-EXT-004 (índice) |
| `ActorSystem.Settings.EnableAdditionalSerializationBindings` (`ActorSystem.scala:427-428`) | Detalle interno sin contrato independiente | constante `true` deprecada desde Akka 2.6.0, sin lectura de configuración; ningún comportamiento que probar. `AllowJavaSerialization` es la lectura de `pekko.actor.allow-java-serialization`, que cubre AC-SER-004.2 |
| `Helpers.compareIdentityHash` vs `identityHashComparator`; `ByteStringBuilder.append` vs `++=`; `ByteString.concat` vs `++`; `getByteBuffers` vs `asByteBuffers`; `utf8String` vs `decodeString(UTF_8)`; `ExtensionId.get` vs `apply`; `ActorSystemSetup.create/apply` | Duplicado o fachada equivalente | fachadas Java/alias del mismo comportamiento; se verifican con la fila del comportamiento y no necesitan fila propia |
| `Helpers.makePattern` frente a `Glob` | Duplicado o fachada equivalente (con diferencias medidas) | AC-UTIL-023 y 016.6; en producción decide `Glob.matches` (ActorSelection.scala:369); Q-UTIL-8 |
| `ByteString1C/ByteString1/ByteStrings` como tres clases, `toByteString1`, `appendToBuilder`, `writeToBuffer`, `take0/dropRight0/drop0`, `Companion` | Detalle interno sin contrato independiente | representación interna; el comportamiento observable (recortes saturados, `++` sin copia, `isCompact`) está en AC-UTIL-006.1/.2 |
| `DoubleLinkedList` por accesores y los nodos de `FrequencyList`/`RecencyList` | Detalle interno sin contrato independiente | la intrusividad no es observable; la lista tiene fila propia (017.5) por sus operaciones «obtener o insertar» |
| `actor-tests/…/util/DurationSpec.scala` y `JavaDuration.java` | Fuera de estos módulos con frontera explícita | prueban `scala.concurrent.duration.Duration` (scala-library), no un contrato de Pekko; la frontera es `Timeout.create`/`JavaDurationConverters` (AC-UTIL-001, 018.2) |
| Consumidores de `FrequencySketch`/`FrequencyList`/`RecencyList`/`SegmentedRecencyList` (cluster-sharding), `MessageBuffer` (cluster-tools, persistence), `UUIDComparator` (persistence-query), `Deploy.tags` (remote) | Fuera de estos módulos con frontera explícita | el contrato de la estructura sí está en estas filas (la estructura vive en `pekko-actor`); el consumidor no. Cada fila lo menciona en «Dependencias» |
| `actor/src/main/boilerplate/**` (interfaces funcionales y tuplas Java) | Cubierto por contratos concretos (otro grupo) | AC-JAPI-005.2 |
| `ClassicActorSystemProvider`, `ActorLogMarker`, `ExtendedActorSystem` (`registerOnTermination`, `uptime`, …) | Cubierto por contratos concretos (otro grupo) | classic-cell / classic-system (no re-verificado aquí) |
| Cuerpos de `FrequencySketchSpec.scala:183-256` (contadores de 16/32/64 bits), `BoundedBlockingQueueSpec.scala:684-914` (soporte del spec), `LineNumberSpec` de Scala 3 | Pendiente de análisis | se leyó el patrón de los casos equivalentes (2/8 bits, `TestContext`); al portar hay que leer cada caso. Queda registrado en las aserciones A4/A10 de AC-UTIL-017.1/012 y no se asume que sean idénticos |
| Re-verificación línea a línea de los bloques `extracted` previos de AC-CFG-*, AC-PATH-*, AC-EXT-001..003 y AC-UTIL-001..009, 011, 015 | Pendiente de análisis | en este cierre sólo se auditaron y corrigieron los bloques con las marcas «no leído / derivada / sin verificar / confirmar» y los que contradecían una búsqueda directa de tests (ver «Contradicciones halladas»); el resto conserva el estado del cierre anterior |
| Scripts del oráculo para los valores medidos (`bytestring.jsh`, `uuid-comparator.jsh`, `hash-code.jsh`, `message-buffer.jsh`, `version-compare.jsh`) | Pendiente de análisis | los valores citados como «medido con el jar» se obtuvieron con scripts ad hoc; hay que persistirlos en `tools/oracle/` con su golden antes de que los tests diferenciales dependan de ellos |

Filas nuevas por omisión real: `AC-UTIL-006.6`, `AC-UTIL-006.7`, `AC-UTIL-006.8` (ver «Omisiones agregadas»). Decisiones nuevas: `Q-UTIL-14` (hash de Scala), `Q-UTIL-15` (`MessageBufferMap.drop` con id ausente), `Q-UTIL-16` (texto de origen de closures).
