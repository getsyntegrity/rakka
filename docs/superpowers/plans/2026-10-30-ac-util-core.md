# Unidad AC-UTIL (parte del núcleo): `Timeout`, `Done`/`NotUsed`, excepciones base, `Helpers` — plan de implementación

**Goal:** Que `rakka-actor` tenga los valores y helpers de `org.apache.pekko` / `org.apache.pekko.util` que el núcleo necesita antes de CFG y del runtime, con el mismo comportamiento observable que Pekko 1.7.1.

**Architecture:** Módulos planos sin dependencias externas, con la misma partición de paquetes que Pekko: `crate::util` (`Timeout`, `TimeUnit`, `helpers`) y, en la raíz del crate, `Done`, `NotUsed`, `PekkoException`, `ConfigurationException` (paquete `org.apache.pekko`). Los errores reutilizan `IllegalArgumentError` de `actor::errors` (clase Java y mensaje exactos).

**Tech Stack:** Rust 1.96, edition 2024, sin dependencias. Cobertura: `cargo llvm-cov` 0.9.1. Oráculo: `jshell` con `tools/oracle/run.sh` (ver "Referencia").

**Fuente del contrato** (rutas relativas a `upstream/pekko/actor/src/main/scala/org/apache/pekko/`): `util/Timeout.scala:22-55`, `Done.scala:25-40`, `NotUsed.scala:22-37`, `PekkoException.scala:19-43`, `util/Helpers.scala:40` (`toRootLowerCase`), `:115-123` (`base64chars`, `base64`), `:125-168` (`Requiring`). Specs: `actor-tests/src/test/scala/org/apache/pekko/PekkoExceptionSpec.scala:26-42`. Filas: `AC-UTIL-001` a `004` de `docs/inventory/actor-classic.tsv`.

## Alcance: qué entra y qué se parte

`docs/implementation-order.md` (Fase 1, ítem 2) pide "`Timeout`, `Done`, excepciones base, helpers de duración" y dice que `ByteString` y lo periférico quedan para la fase 4. Cruzado con las filas del inventario:

| Fila | Qué | Decisión | Motivo |
|---|---|---|---|
| `AC-UTIL-001` | `Timeout` | **entra** | lo usan `ask` y `resolveOne` (AC-ASK); sin dependencias |
| `AC-UTIL-002` | `Helpers`: `base64`, `toRootLowerCase`, `Requiring` | **entra** | `base64` genera los nombres `$a`, `$b` y los temp paths (`Children.scala:126,130`, `ActorRefProvider.scala:440`); `toRootLowerCase` lo usan `Logging.scala:518` y `ActorSystem.scala:441`; `Requiring` valida settings (`LightArrayRevolverScheduler.scala:60`, `AffinityPool.scala:321`) |
| `AC-UTIL-002` | `ConfigOps` (`getMillisDuration`, `getNanosDuration`, `renderWithRedactions`) | **fuera** | necesita `Config`: depende de AC-CFG, que la tarea excluye |
| `AC-UTIL-003` | `Done`, `NotUsed` | **entra** | sin dependencias |
| `AC-UTIL-004` | `PekkoException`, `ConfigurationException`, `OnlyCauseStackTrace` | **entra** | base de los errores de configuración |
| `AC-UTIL-016` | `PrettyDuration` (único helper de duración propio de Pekko) | **fuera, siguiente spec** | no entra en cinco tareas junto con lo anterior; su primer consumidor es `AC-REF-014` (log de `DeadLetterListener`) y necesita decidir cómo modelar `Duration.Inf`/`Undefined` (ver "Qué queda") |
| `AC-UTIL-005` | `PekkoVersion`, `Version` | fuera | `listed`, sin consumidor en el núcleo |
| `AC-UTIL-006` en adelante | `ByteString`, colas, índices, etc. | fuera | fase 4 (instrucción explícita) |

`DurationSpec.scala:20-105` no es un contrato de Pekko: prueba `scala.concurrent.duration.Duration` (aritmética, infinitos, `fromNow`), que es de la biblioteca estándar de Scala. No se porta. `ActorTimeoutSpec.scala:34-47` prueba `ask` con `Timeout`: es de AC-ASK-001 y necesita un `ActorSystem`; no se porta aquí.

## Referencia: qué se usó como oráculo

Los escenarios diferenciales se corren con `ORACLE=jars tools/oracle/run.sh tools/oracle/util-core.jsh`, es decir contra los **jars oficiales** `pekko-actor_2.13-1.7.1.jar` y `scala-library-2.13.18.jar` (SHA-1 verificados por `tools/oracle/fetch-jars.sh`), con **JDK 11 Temurin** (el que hay en esta máquina; `AGENTS.md` pide Java 21). El modo por defecto (Pekko compilado desde `upstream/pekko`) no se pudo usar: el submódulo `upstream/pekko` no está inicializado en este checkout y `tools/oracle/sbt.sh` descarga un JDK 8 de Linux x64, que no corre en esta máquina (macOS arm64). Consecuencia: la fuente de verdad de los *contratos* son las fuentes de Pekko leídas en el commit fijado `9a54d129…` (copia local en el mismo SHA), y el golden es **evidencia complementaria de jars oficiales, no de fuentes compiladas**. Los jars son el artefacto publicado de la misma versión 1.7.1. El golden lo declara en su cabecera.

## Decisiones (con la evidencia y lo descartado)

**1. `Timeout` sobre `std::time::Duration`, `Copy`.** Es lo que fija el inventario (`rust_equivalent` de `AC-UTIL-001`). Se reproduce `Timeout.zero`, `Timeout(duration)` (`new`) y `Timeout(length, unit)` (`of`). La conversión implícita `durationToTimeout` (`Timeout.scala:54`) es `impl From<Duration> for Timeout`. `Timeout.create(java.time.Duration)` (`Timeout.scala:49-52`) es `new`: en Rust no hay otro tipo de duración. Tres diferencias con Pekko, todas con `Q:` (abajo): Q-UTIL-1 (negativos y rango), Q-UTIL-2 (`toString`).

**2. `Timeout::of(length: u64, unit)` reproduce el rango de `FiniteDuration`.** `Timeout(length, unit)` llama a `Duration(length, unit)`, que lanza `IllegalArgumentException("requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)")` si `length` excede `(2^63-1) / ns_por_unidad`. El oráculo fija los bordes: `106751 DAYS` ok, `106752 DAYS` error; `9223372036854 MILLISECONDS` ok, `…855` error. Los tipos sin signo dejan fuera los negativos (Q-UTIL-1).

**3. `Done` y `NotUsed` son structs unitarios propios**, no `()` (propuesta principal del inventario). Eso deja abierta la `Q:` que ya tiene la fila `AC-UTIL-003` ("¿usar `()`?, afecta firmas públicas"): la implementación toma la opción del inventario, pero la fila **no se cierra** hasta que el Owner decida. `getInstance()`/`done()`/`notUsed()` son fachada Java: en Rust el valor es el propio `Done`.

**4. Excepciones: tipos separados con causa compartida (`Arc`).** `PekkoException` y `ConfigurationException` exponen `message()`, `cause()` (`getCause`), `java_class()` y `Display` = mensaje, como los errores de `actor::errors`. La causa es `Arc<dyn Error + Send + Sync>` para poder clonar el error (Pekko comparte la excepción por referencia). No hay jerarquía `ConfigurationException <: PekkoException` en el sistema de tipos (Q-UTIL-4) ni `OnlyCauseStackTrace` (no hay stack traces: Q-UTIL-4).

**5. `base64(l: i64)`, no `u64`.** El inventario propone `u64`; el dominio de Pekko es `Long` con signo y `l >>> 6` (shift sin signo). Con `i64` el comportamiento para negativos (`base64(-1) = "$~~~~~~~~~~p"`) queda definido igual que en Pekko. Además de `base64(l)` (prefijo `$`) se expone `base64_append(l, &mut String)`, que es el parámetro `sb` que usan `Children.scala:126` y `ActorRefProvider.scala:440`.

**6. `to_root_lower_case` es `str::to_lowercase`.** Rust y Java aplican el mapeo de minúsculas completo de Unicode con sigma final contextual (`ΑΣ` → `ας`) y `İ` → `i̇` (dos unidades). El oráculo lo comprueba sobre una muestra; no se afirma igualdad para versiones de Unicode distintas (JDK 11 = Unicode 10; Rust 1.96 es más nuevo), por eso la muestra se limita a caracteres anteriores a Unicode 10.

**7. `Requiring` es un trait con impl genérico**, importable como `util::helpers::Requiring` (en Scala es `import Helpers.Requiring`). `requiring(cond, msg)` y `requiring_with(|&a| cond, msg)` devuelven `Result<A, IllegalArgumentError>` con mensaje `requirement failed: <msg>`. `msg` es `impl Display`; la evaluación diferida (`=> Any`) no se reproduce más allá de que el `Display` se formatea sólo al fallar (Q-UTIL-3).

## Preguntas abiertas para el Owner

Todas anotadas con `Q:` en las filas del inventario.

- **Q-UTIL-1 (AC-UTIL-001): negativos y rango de `Timeout`.** `FiniteDuration` es un `Long` de nanosegundos con signo, entre `-(2^63-1)` y `2^63-1`; `Timeout(-5.millis)` es válido y `ask` lo rechaza después (AC-ASK-001 "≤0"). `std::time::Duration` no tiene negativos y llega a ~584 mil años. Alternativas: (a) quedarse con `std::time::Duration` y que `ask` reciba un `Timeout` siempre positivo, documentando que `Timeout(-5.millis)` no se puede construir (lo implementado, recomendada hasta que AC-ASK necesite el caso); (b) un tipo `FiniteDuration` propio con `i64` nanosegundos, exacto pero hay que reimplementar aritmética; (c) `Timeout::new` falible que rechace lo que Pekko no puede construir. El caso `Timeout.create(ofSeconds(Long.MAX_VALUE))` (`ArithmeticException: long overflow` en Pekko) tampoco se reproduce.
- **Q-UTIL-2 (AC-UTIL-001): `toString` conserva la unidad.** Pekko imprime `Timeout(100 milliseconds)` o `Timeout(0 days)`: el `FiniteDuration` recuerda la unidad con que se construyó, aunque `Timeout(1000.millis) == Timeout(1.second)`. `std::time::Duration` no la recuerda. Alternativas: (a) `Debug` derivado (`Timeout(100ms)`) y no prometer el formato de Pekko (lo implementado, recomendada: sólo aparece en logs); (b) guardar la unidad junto con el valor (rompe la igualdad por valor o exige un `PartialEq` manual).
- **Q-UTIL-3 (AC-UTIL-002, 004): `null` y mensajes diferidos.** `new PekkoException(null)` tiene `getMessage == null` y se imprime sin `": "`; `requiring(cond, null)` da `requirement failed: null`. Con `&str`/`impl Display` no se puede expresar `null`. Alternativa: aceptarlo como diferencia de lenguaje (recomendada), igual que Q-PATH-4.
- **Q-UTIL-4 (AC-UTIL-004): jerarquía y `OnlyCauseStackTrace`.** (i) `ConfigurationException` es subclase de `PekkoException`: un `catch PekkoException` la atrapa. Con tipos separados no. Alternativas: (a) tipos separados y un `From<ConfigurationException> for PekkoException` (pierde `java_class`) cuando algún consumidor lo pida (recomendada); (b) un único `enum PekkoError`; (c) un trait común. (ii) `OnlyCauseStackTrace` sólo cambia qué stack trace imprime la JVM; los errores de Rust no llevan stack trace. Propuesta: excluirlo y documentarlo.
- **Q-UTIL-5 (AC-UTIL-003): `Done` propio o `()`** (ya estaba en el inventario). Se implementó el struct; la fila queda `partial` hasta decidir.

## Global Constraints

- Sin dependencias en `crates/rakka-actor/Cargo.toml`.
- TDD por comportamiento (RED → GREEN → REFACTOR) con el comando y el resultado observados; esqueleto mínimo que compila para que el RED falle por aserción y no por infraestructura.
- Cobertura de líneas ≥ 90 % sobre las líneas de producción nuevas o modificadas, medida con `cargo llvm-cov`; sin exclusiones.
- `cargo fmt --all --check`, `cargo build --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets -- -D warnings` limpios.
- Cada archivo Rust lleva el encabezado de atribución de `docs/upstream.md`.

## Contracts

```rust
// crates/rakka-actor/src/util/timeout.rs  (+ time_unit.rs)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Timeout(Duration);
impl Timeout {
    pub const ZERO: Timeout;                                                     // Timeout.zero
    pub const fn new(duration: Duration) -> Timeout;                             // Timeout(duration) / create
    pub fn of(length: u64, unit: TimeUnit) -> Result<Timeout, IllegalArgumentError>; // Timeout(length, unit)
    pub const fn duration(&self) -> Duration;
}
impl From<Duration> for Timeout {}                                               // durationToTimeout
pub enum TimeUnit { Nanoseconds, Microseconds, Milliseconds, Seconds, Minutes, Hours, Days }

// crates/rakka-actor/src/done.rs, not_used.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub struct Done;     // Debug imprime "Done"
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub struct NotUsed;  // Debug imprime "NotUsed"

// crates/rakka-actor/src/pekko_exception.rs
pub struct PekkoException { /* message, cause: Option<Arc<dyn Error + Send + Sync>> */ }
pub struct ConfigurationException { /* igual */ }
// ambas: new(msg), with_cause(msg, cause), message(), cause(), java_class(); Display = message; Error::source = cause

// crates/rakka-actor/src/util/helpers.rs
pub fn base64(l: i64) -> String;                       // "$" + dígitos, menos significativo primero
pub fn base64_append(l: i64, sb: &mut String);         // base64(l, sb)
pub fn to_root_lower_case(s: &str) -> String;
pub trait Requiring: Sized {
    fn requiring(self, cond: bool, msg: impl Display) -> Result<Self, IllegalArgumentError>;
    fn requiring_with(self, cond: impl FnOnce(&Self) -> bool, msg: impl Display) -> Result<Self, IllegalArgumentError>;
}
```

## Trazabilidad: fila → escenarios

| Fila | Escenarios (ids del oráculo y tests) |
|---|---|
| `AC-UTIL-001` | `TO-*` (zero, unidades, bordes de rango, igualdad por valor) en `oracle_util_core.rs`; `timeout_spec.rs` |
| `AC-UTIL-002` | `B64-*` (alfabeto, orden, negativos, `sb` con prefijo), `LC-*` (sigma final, `İ`, vacío), `RQ-*` (mensaje, valor devuelto, forma con predicado) |
| `AC-UTIL-003` | `DN-*`, `done_spec.rs` |
| `AC-UTIL-004` | `EX-*` (mensaje, causa, sin causa), `pekko_exception_spec.rs` (traducción de `PekkoExceptionSpec.scala:29-36`: construcción con un único `String`) |

## Tareas (máximo 5)

### Task 1 — `Timeout` y `TimeUnit` (AC-UTIL-001)

**Files:** `crates/rakka-actor/src/util/{mod,timeout,time_unit}.rs`, `crates/rakka-actor/src/lib.rs`, `crates/rakka-actor/tests/timeout_spec.rs`.

Comportamientos, uno por ciclo RED→GREEN: `Timeout::ZERO` y `new`/`duration`; `From<Duration>`; `of` convierte cada unidad; `of` rechaza lo que excede el rango de `FiniteDuration` con el mensaje exacto; igualdad por valor (`1000 ms == 1 s`).

- [ ] Commit `feat(util): Timeout and TimeUnit (AC-UTIL-001)`.

### Task 2 — `Done` y `NotUsed` (AC-UTIL-003)

**Files:** `crates/rakka-actor/src/{done,not_used}.rs`, `crates/rakka-actor/src/lib.rs`, `crates/rakka-actor/tests/done_spec.rs`.

Comportamientos: `Done` es un valor único (copias iguales, un solo elemento en un `HashSet`, `Debug` y `Display` = `"Done"`); `NotUsed` ídem; son tipos distintos.

- [ ] Commit `feat: Done and NotUsed (AC-UTIL-003)`.

### Task 3 — `PekkoException` y `ConfigurationException` (AC-UTIL-004)

**Files:** `crates/rakka-actor/src/pekko_exception.rs`, `crates/rakka-actor/src/lib.rs`, `crates/rakka-actor/tests/pekko_exception_spec.rs`.

Comportamientos: constructor con un único `String` (el que verifica `PekkoExceptionSpec`); mensaje; con y sin causa; `Display`/`Error::source`; `Clone` comparte la causa; `java_class`.

- [ ] Commit `feat: PekkoException and ConfigurationException (AC-UTIL-004)`.

### Task 4 — `util::helpers` (AC-UTIL-002, sin `ConfigOps`)

**Files:** `crates/rakka-actor/src/util/helpers.rs`, `crates/rakka-actor/tests/helpers_spec.rs`.

Comportamientos: `base64` (alfabeto, orden, un dígito vs. varios, negativos, prefijo propio); `to_root_lower_case`; `requiring` (éxito devuelve el valor, fallo con mensaje exacto); `requiring_with`.

- [ ] Commit `feat(util): Helpers base64, toRootLowerCase, Requiring (AC-UTIL-002)`.

### Task 5 — Oráculo, gates, inventario y documentación

**Files:** `tools/oracle/util-core.jsh`, `crates/rakka-actor/tests/oracle/util-core.golden`, `crates/rakka-actor/tests/oracle_util_core.rs`, `docs/inventory/actor-classic.tsv` (filas `AC-UTIL-001` a `004`), `README.md`, `docs/implementation-order.md`, este plan (sección Evidencia) y la última línea del plan AC-PATH.

- [ ] Escribir los casos del oráculo, regenerar el golden con los jars y comparar línea por línea con el render de Rust (los casos que dependen de una `Q:` abierta existen en el golden y Rust no los produce a propósito, como en `oracle_path_address.rs`).
- [ ] Correr los gates y `cargo llvm-cov`; registrar líneas cubiertas/medibles.
- [ ] Inventario: `impl`/`validation` sólo donde la evidencia lo justifica (ver "Cierre").
- [ ] Commit `test: differential oracle for AC-UTIL core`, `docs: inventory and README for AC-UTIL core`.

## Cierre esperado de las filas

`AC-UTIL-001`, `003` y `004` quedan `partial` mientras Q-UTIL-1, 2, 4 y 5 estén sin decidir. `AC-UTIL-002` queda `partial` por `ConfigOps` (AC-CFG) y Q-UTIL-3. Ninguna fila pasa a `done` en esta unidad salvo que la evidencia lo permita sin `Q:` abierta.

## Qué queda de AC-UTIL tras esta spec

- **Siguiente spec de la cadena:** `AC-CFG` (parser y carga), según `docs/implementation-order.md`.
- **`ConfigOps`** (`AC-UTIL-002`) se agrega apenas exista `Config` (AC-CFG).
- **`PrettyDuration`** (`AC-UTIL-016`) tiene que estar antes de `AC-REF-014`; hay que decidir cómo modelar `Duration.Inf`, `MinusInf` y `Undefined` (el formato `%.4g` de Java es el de `java.util.Formatter`, no el de C).
- **`AC-UTIL-005`** (`Version`) y el resto de la fila 006 en adelante: fase 4.

## Evidencia TDD

(Se completa durante la ejecución, con el comando y el resultado observados en cada etapa.)
