# Cierre técnico — selección de mailbox por id (PR #339)

Base `375a763` (`feat/pekko-rust-pilot-migration`), upstream Apache Pekko `9fbd2283` (v2.0.0).

## 1. `git diff --check`

Origen: los reportes por mutante citan la salida de diff de Rust, que contiene líneas en blanco como `+ ` (espacio final), y `publish_mutant_report` las escribía tal cual.
Arreglo en el origen: `tools/parity/check_divergence.py` quita los espacios finales al publicar el reporte, con test previo
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

## 3. Cola compartida entre actores: imposible por construcción

- `Mailboxes` guarda `BTreeMap<String, Arc<dyn MailboxType>>` (`dispatch/mailboxes.rs:14`): una fábrica sin estado de cola.
- `MailboxType::create(&self, &ActorRef) -> Box<dyn MessageQueue>` (`dispatch/message_queue.rs:44`) devuelve una cola con dueño único (`Box`, sin `Clone`).
- Único sitio de producción que crea una cola de usuario: `ActorCell::build` (`actor/cell.rs:242`), una vez por celda.
- Por ownership, dos celdas no pueden poseer el mismo `Box`. Un `MailboxType` de usuario podría devolver colas que envuelvan estado compartido, igual que en Pekko: es decisión del tipo, no del registro ni de la resolución.
- Cubierto de todos modos por `two_actors_with_the_same_id_each_get_their_own_queue` (2 colas creadas, mensajes separados) y por `MBX-ID-01` (actores `a` y `c`, mismo id).

## 4. Tamaño (regla de 400 líneas)

Diff contra la base, líneas añadidas + eliminadas, sin contar nada como excepción:

| Categoría | Líneas | Archivos |
|---|---|---|
| Producción (incluye tests unitarios en línea de `mailboxes.rs` y `errors.rs`) | 321 | 8 |
| Tests de integración (`tests/`) | 429 | 3 |
| Harness (oráculo Java, gemelo Rust, mutantes, herramientas) | ~440 | 5 |
| Documentación escrita | ~110 | 8 |
| Documentos generados (`progress`, `decisions`, plan de unidades) | 44 | 8 |
| Evidencia generada (grabaciones, reportes de mutantes, listas de casos) | ~2030 | 34 |

El PR **incumple** la regla de 400 líneas con cualquier lectura que incluya tests. Sólo la producción sin tests en línea (~180) cabe.

División mínima propuesta (sin fragmentar el comportamiento, apilada sobre la base):

1. **Refactor de tests** (~135 líneas, sin comportamiento): mover `LifoQueue`, `LifoMailbox` y `Gate` a `tests/common` y quitar la copia de `classic_mailbox_type_selection.rs`.
2. **Herramienta de reportes** (~60 líneas): quitar espacios finales al publicar reportes de mutantes, con su test.
3. **Núcleo** (~615 líneas escritas: ~320 de `src/` y ~295 del test de integración): `Mailboxes`, `Props::with_mailbox_id`, `ConfigurationError` y la resolución en `attach_child`. Es el mínimo atómico: sin el error tipado y sin la propagación en `actorOf`, un id inválido caería en silencio al default (viola MB-ID-08). **No baja de 400 sin separar los tests de la conducta que prueban**, lo que `docs/testing-policy.md` no permite; requiere decisión del Owner sobre cómo se mide la regla (¿se cuentan los tests?).
4. **Paridad** (~390 líneas escritas más grabaciones generadas): casos `MBX-ID-01..03` (Java y gemelo Rust), registro de casos y los 7 mutantes. Depende de 3.
5. **Documentación** (~40 líneas): Q-MBX-12 en `proposed`, nota de `AC-MBX-014` y evidencia; depende de 3 y 4.

## 5. Q-MBX-12

Texto completo en `docs/decisions/register.tsv`, `docs/inventory/contracts/classic-dispatch.decisions.tsv` y `docs/owner-batch-02.md`: ids soportados (`unbounded`, `pekko.actor.default-mailbox`, ids registrados), ids temporalmente no soportados (`bounded`, `bounded-capacity:N`, por Q-MBX-1), comportamiento de `pekko.actor.default-mailbox`, límites del error y del dispatcher, y cinco tareas futuras para eliminar las divergencias. Estado `proposed`; `AC-MBX-014` permanece `partial`. Ninguna decisión aceptada.
