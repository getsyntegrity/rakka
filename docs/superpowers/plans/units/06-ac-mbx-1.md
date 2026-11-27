# Spec 06 — AC-MBX (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 23. **Tareas:** 4 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 11 `AC-CELL-1` (`11-ac-cell-1.md`), por las filas AC-CELL-008.2, AC-CELL-013.1, AC-CELL-013.3, AC-CELL-014. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 07 `AC-DSP-1` (`07-ac-dsp-1.md`), por las filas AC-DSP-001, AC-DSP-002, AC-DSP-003, AC-DSP-004, AC-DSP-023. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 09 `AC-EVT-1` (`09-ac-evt-1.md`), por las filas AC-EVT-008. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 03 `AC-CFG-1` (`03-ac-cfg-1.md`), por las filas AC-CFG-001, AC-CFG-016.
- Spec 19 `AC-RT-1` (`19-ac-rt-1.md`), por las filas AC-RT-008. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 04 `AC-REF-1` (`04-ac-ref-1.md`), por las filas AC-REF-015.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-CFG-31`, `Q-DSP-2`, `Q-DSP-5`, `Q-MBX-1`, `Q-MBX-11`, `Q-MBX-2`, `Q-MBX-3`, `Q-MBX-4`, `Q-MBX-7`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Estado del mailbox: Open/Closed/Scheduled + contador de suspensión (+5 filas)

Filas: AC-MBX-001, AC-MBX-002, AC-MBX-003, AC-MBX-004, AC-MBX-005, AC-MBX-006. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: BoundedMailbox (capacidad + mailbox-push-timeout-time) (+5 filas)

Filas: AC-MBX-007, AC-MBX-008, AC-MBX-009, AC-MBX-010, AC-MBX-011, AC-MBX-012. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: Precedencia de selección de mailbox y verificación de requisitos (+5 filas)

Filas: AC-MBX-013, AC-MBX-014, AC-MBX-015, AC-MBX-016, AC-MBX-017, AC-MBX-018. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: numberOfMessages/hasMessages son aproximados (+4 filas)

Filas: AC-MBX-019, AC-MBX-020, AC-MBX-021, AC-MBX-022, AC-MBX-023. Contratos y aserciones: `docs/inventory/contracts/classic-dispatch.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-MBX-001 | Estado del mailbox: Open/Closed/Scheduled + contador de suspensión | `AtomicU32` con los mismos bits o enum+contador bajo lock | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/ActorModelSpec.scala:312-330… | Q-MBX-10, Q-MBX-5, Q-MBX-9, Q-X-4 |
| AC-MBX-002 | Ejecución del mailbox: sistema primero, luego hasta `throughput` mensajes (o… | tarea async/hilo que procesa N y cede (yield) | actor-tests/src/test/scala/org/apache/pekko/actor/dispatch/DispatcherActorSpec.scala:84,1… | Q-DSP-5, Q-MBX-10, Q-MBX-11, Q-MBX-5, Q-MBX-6, Q-MBX-9 |
| AC-MBX-003 | Mailbox cerrado: mensajes pendientes a deadLetters; sistema pendiente al… | drenar la cola a deadLetters al cerrar | actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithBoundedStashSpec.scala:122-149… | Q-MBX-10, Q-MBX-5, Q-MBX-8, Q-MBX-9 |
| AC-MBX-004 | SPI MessageQueue / MailboxType / ProducesMessageQueue y mailbox-type custom… | traits `MessageQueue`, `MailboxType` + registro por nombre | actor-tests/src/test/scala/org/apache/pekko/dispatch/MailboxConfigSpec.scala:259 | Q-CFG-31, Q-DSP-2, Q-X-2 |
| AC-MBX-005 | UnboundedMailbox (default) | cola MPMC lock-free (`crossbeam::queue::SegQueue`) o `Mutex<VecDeque>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:256 ;… | - |
| AC-MBX-006 | SingleConsumerOnlyUnboundedMailbox (MPSC de Vyukov) | canal MPSC (`std::sync::mpsc`/`crossbeam-channel`/`tokio::mpsc`) | actor-tests/src/test/scala/org/apache/pekko/dispatch/MailboxConfigSpec.scala:323 | Q-MBX-16, Q-MBX-6 |
| AC-MBX-007 | BoundedMailbox (capacidad + mailbox-push-timeout-time) | canal acotado; bloqueo vs `async send` | actor-tests/src/test/scala/org/apache/pekko/dispatch/MailboxConfigSpec.scala:53,57,86,90… | Q-MBX-1, Q-X-4 |
| AC-MBX-008 | NonBlockingBoundedMailbox | cola acotada lock-free con `try_push` | actor-tests/src/test/scala/org/apache/pekko/dispatch/MailboxConfigSpec.scala:327 | Q-MBX-6 |
| AC-MBX-009 | UnboundedPriorityMailbox / BoundedPriorityMailbox y PriorityGenerator | `BinaryHeap` con comparador | actor-tests/src/test/scala/org/apache/pekko/dispatch/PriorityDispatcherSpec.scala:58,63… | Q-MBX-1, Q-MBX-4, Q-MBX-7 |
| AC-MBX-010 | UnboundedStablePriorityMailbox / BoundedStablePriorityMailbox | heap con secuencia | actor-tests/src/test/scala/org/apache/pekko/dispatch/StablePriorityDispatcherSpec.scala:6… | Q-MBX-1, Q-MBX-4, Q-MBX-7 |
| AC-MBX-011 | Mailboxes basados en deque (Unbounded/BoundedDequeBasedMailbox, enqueueFirst) | `Mutex<VecDeque>` con push_front | actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:260,288,358,375,… | Q-MBX-1, Q-MBX-14, Q-MBX-15, Q-MBX-2 |
| AC-MBX-012 | Mailboxes control-aware (ControlMessage primero) | dos colas; marker trait `ControlMessage` | actor-tests/src/test/scala/org/apache/pekko/dispatch/ControlAwareDispatcherSpec.scala:37,… | Q-MBX-1, Q-MBX-3 |
| AC-MBX-013 | Precedencia de selección de mailbox y verificación de requisitos | resolución en `Mailboxes::mailbox_type_for(props, dispatcher_cfg)` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:256-459 | Q-MBX-13, Q-MBX-14, Q-MBX-2 |
| AC-MBX-014 | Lookup de mailboxes por id, `bounded-capacity:N`, lookupByQueueType,… | registro + parseo de prefijo | actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:267,303,310 | Q-DSP-2, Q-MBX-1, Q-MBX-12, Q-MBX-13, Q-MBX-14, Q-MBX-2, Q-X-1 |
| AC-MBX-015 | Traits de semántica de cola… | `enum QueueSemantics` + método `required_queue()` en el actor | actor-tests/src/test/scala/org/apache/pekko/actor/ActorMailboxSpec.scala:267 | Q-MBX-2 |
| AC-MBX-016 | Stash: stash(), unstash, unstashAll (orden original), capacidad,… | `Stash` en el contexto (Vec<Envelope>) + push_front | actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithStashSpec.scala:121-127,129-13… | Q-MBX-1 |
| AC-MBX-017 | Stash en restart y stop: preRestart y postStop hacen unstashAll | - | actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithStashSpec.scala:149-184 ;… | - |
| AC-MBX-018 | Variantes UnboundedStash / UnrestrictedStash, StashFactory, clearStash,… | - | actor-tests/src/test/java/org/apache/pekko/actor/StashJavaAPI.java:40-53 ;… | Q-MBX-2 |
| AC-MBX-019 | numberOfMessages/hasMessages son aproximados | - | derivada del código (ver bloque de contrato) | - |
| AC-MBX-020 | Mapeo de requirement LoggerMessageQueueSemantics a mailbox logger-queue… | tipo de cola de logger drenable | derivada del código (ver bloque de contrato) | Q-X-7 |
| AC-MBX-021 | deadLetterMailbox: mailbox cerrado compartido que reenvía a deadLetters | mailbox singleton cerrado con cola que publica DeadLetter | actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithBoundedStashSpec.scala:122-149… | Q-MBX-5 |
| AC-MBX-022 | Validación y avisos de configuración de mailbox (capacidad negativa,… | validación en constructores de MailboxType | derivada del código (ver bloque de contrato) | - |
| AC-MBX-023 | Capacidad del stash por dispatcher/mailbox (stash-capacity, caché, alias de… | resolución de config con fallback encadenado | actor-tests/src/test/scala/org/apache/pekko/actor/ActorWithBoundedStashSpec.scala:154-178 | Q-X-1 |

## Gates y criterio de cierre

```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --lcov --output-path target/lcov.info   # cobertura de líneas nuevas o modificadas >= 90 %; reportar comando, resultado y líneas no cubiertas con motivo
```

La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.

## Siguiente

Spec 07 `AC-DSP-1` (`07-ac-dsp-1.md`).
