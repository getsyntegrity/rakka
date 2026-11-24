# Cierre técnico — selección de mailbox por id (PR #339)

Base `375a763` (`feat/pekko-rust-pilot-migration`), upstream Apache Pekko `9fbd2283` (v2.0.0).

## 1. `git diff --check`

Origen: los reportes por mutante citan la salida de diff de Rust, que contiene líneas en blanco como `+ ` (espacio final), y `publish_mutant_report` las escribía tal cual.
Arreglo en el origen: `tools/parity/check_divergence.py` quita sólo el espacio final de las líneas de diff en blanco (`+ `, `- `) al publicar el reporte, de modo que una divergencia que sea de espacios sigue visible, con test previo
(`test_replay_isolation.py::test_a_published_report_has_no_trailing_whitespace_though_the_quoted_diff_has_blank_lines`, RED observado antes del cambio). Los reportes se regeneraron ejecutando de nuevo los mutantes. Sin exclusiones.

## 2. Flake `MBX-RJ-03`

Observado una vez: `ERR IllegalStateException [condition not reached]` dentro de `check_mailbox_oracle.py --runs 1` (`mailbox-all`, una JVM).
`condition not reached` es el techo de 10 s de `awaitUntil` del oráculo Java: lo ejecuta Pekko, no Rust.

| Medición | Base `375a763` | HEAD del PR |
|---|---|---|
| `RJ-03` aislado, 25 corridas, máquina libre | 0/25 | 0/25 |
| `RJ-03` aislado, 25 corridas, 8 bucles de CPU | 0/25 | 0/25 |
| `mailbox-all` completo, 8 corridas | 0/8 (62 casos) | 0/8 (65 casos) |

Código: `git diff` de `MailboxOracle.java` contra la base tiene 0 líneas eliminadas; los casos `MBX-ID-01..03` se registran después de `MBX-UT-09`, es decir después de `RJ-03`, y `IdEnv` es una clase aparte (no toca `Env`).

Clasificación: **no atribuible a este slice** (el código de `RJ-03` y el de su entorno no cambian y el slice no se ejecuta antes que él), y **no reproducido**: 0/116 fallos en base y HEAD. No se declara resuelto ni se afirma causa: es una intermitencia de espera con techo del oráculo, observada una sola vez, con causa no determinada. Seguimiento: si reaparece, registrar carga de la máquina y la salida completa de `mailbox-all`; candidato de endurecimiento, fuera de este PR: que `awaitUntil` informe qué condición esperaba.

## 3. Cola compartida entre actores

Ni el registro ni la resolución pueden compartir una cola entre actores; un tipo de usuario sí podría, igual que en Pekko.

- `Mailboxes` guarda `BTreeMap<String, Arc<dyn MailboxType>>` (`dispatch/mailboxes.rs:14`): una fábrica sin estado de cola.
- `MailboxType::create(&self, &ActorRef) -> Box<dyn MessageQueue>` (`dispatch/message_queue.rs:44`) devuelve una cola con dueño único (`Box`, sin `Clone`).
- Único sitio que crea la cola de usuario de un actor seleccionado: `ActorCell::build` (`actor/cell.rs`, `props.new_message_queue`), una vez por celda. Las otras dos llamadas a `UnboundedMailbox.create()` (la celda de reemplazo de una creación fallida y la limpieza) no usan selección.
- Por ownership, dos celdas no pueden poseer el mismo `Box`. Un `MailboxType` de usuario podría devolver colas que envuelvan estado compartido: es decisión del tipo.
- Cubierto de todos modos por `two_actors_with_the_same_id_each_get_their_own_queue` y por `MBX-ID-01` (actores `a` y `c`, mismo id).

## 4. Tamaño (regla de 400 líneas) y pila resultante

El PR #339 (referencia funcional, `c3858ba`) incumplía la regla. Se dividió en nueve PRs apilados; cada uno cabe en 400 líneas contando la evidencia generada:

| PR | Contenido | Propias / generadas / total |
|---|---|---|
| #340 | helpers de tests compartidos | 134 / 0 / 134 |
| #341 | herramienta de reportes | 13 / 0 / 13 |
| #342 | registro `Mailboxes` | 138 / 0 / 138 |
| #343 | `ActorError::Configuration` | 76 / 0 / 76 |
| #344 | `Props::with_mailbox_id` y resolución en `attach_child` | 400 / 0 / 400 |
| #345 | casos de oráculo `MBX-ID-01..03`, gemelo y grabaciones | 333 / 38 / 371 |
| #346 | cuatro mutantes de la ruta de creación y sus reportes | 25 / 198 / 223 |
| #347 | tres mutantes del registro y sus reportes | 18 / 174 / 192 |
| #348 | documentación y esta evidencia | ver el PR |

El árbol final equivale a `c3858ba` salvo: dos líneas de comentario de `tests/classic_mailbox_id_selection.rs` (para que #344 no pase de 400), `rust/app-cases/replay-report.md` y el reporte `selftest` (se dejan como en la rama de integración: el primero llevaba una ruta local y un caso ajeno) y los arreglos de revisión de #341.

## 5. Q-MBX-12

Texto completo en `docs/decisions/register.tsv`, `docs/inventory/contracts/classic-dispatch.decisions.tsv` y `docs/owner-batch-02.md`: ids soportados (`unbounded`, `pekko.actor.default-mailbox`, ids registrados), ids temporalmente no soportados (`bounded`, `bounded-capacity:N`, por Q-MBX-1), comportamiento de `pekko.actor.default-mailbox`, límites del error y del dispatcher, y cinco tareas futuras para eliminar las divergencias. Estado `proposed`; `AC-MBX-014` permanece `partial`. Ninguna decisión aceptada.

## 6. Procedencia de las grabaciones y límites conocidos

- Procedencia de `mb-0019..21`: `oracle/run.sh mailbox-case` (JDK 17, Pekko 2.0.0) invocado por `rust-anything app record . --inputs <las tres últimas líneas de cases/oracle-mailbox.jsonl> --no-traps --name classic-parity -- oracle/run.sh`; el recorder ejecuta el oráculo, nunca Rust. Las líneas coinciden con `cases/oracle-only/mailbox.golden`.
- Detección por caída: el harness calcula todas las líneas de mailbox a la vez, así que un mutante que hace entrar en pánico a `MBX-ID-01` o `-02` (selección ignorada, id inválido aceptado, nombre retenido) hace divergir los 21 casos de la familia y el reporte nombra `mb-0001`. Es detección real pero por caída del conjunto; sólo los mutantes del mensaje de error fallan con una divergencia concreta (`mb-0020`).
- El gemelo Rust de `MBX-ID-01..03` registra sólo los dead letters de tipo `String` y sirve `MBX-ID-03` con cuatro actores bloqueados a la vez (necesita `parallelism` ≥ 4, por defecto 8); el oráculo registra cualquier mensaje.
- El gemelo imprime la clase del error con la constante `java_name()` y la causa con una clase fija: la igualdad con Pekko la prueba la comparación byte a byte con el oráculo.
- Un tipo registrado bajo `pekko.actor.default-mailbox` sólo rige la selección explícita; Pekko lo aplica también a los actores sin selección (Q-MBX-12, tarea 2).
- Pendientes menores de #344: el comentario de `make_repointed` ("Nothing in Rakka can make this fail yet") no menciona que el mailbox falla antes; no hay test con nombre autogenerado ni con nombre duplicado e id desconocido (el nombre se reserva primero: `InvalidActorName`).
