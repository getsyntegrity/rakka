# Spec 49 — AT-DLV (1/1)

**Estado:** pendiente de decisiones propuestas. **Filas:** 25. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-typed.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

## Prerrequisitos

- Spec 32 `AT-CTX-1` (`32-at-ctx-1.md`), por las filas AT-CTX-002, AT-CTX-006, AT-CTX-011, AT-CTX-012.
- Spec 27 `AT-BEH-1` (`27-at-beh-1.md`), por las filas AT-BEH-021.
- Spec 43 `AT-RCP-1` (`43-at-rcp-1.md`), por las filas AT-RCP-003, AT-RCP-005.2.
- Spec 41 `AT-EXT-1` (`41-at-ext-1.md`), por las filas AT-EXT-001.1.

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-T-DLV-1`, `Q-T-DLV-2`, `Q-T-DLV-3`, `Q-T-DLV-4`, `Q-T-RCP-1`, `Q-T-SER-1`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: ProducerController: protocolo Start/RequestNext y demanda local uno a uno (+5 filas)

Filas: AT-DLV-001.1, AT-DLV-001.2, AT-DLV-001.3, AT-DLV-001.4, AT-DLV-001.5, AT-DLV-001.6. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: ConsumerController: arranque con Start, mensajes previos a Start y parada con el… (+5 filas)

Filas: AT-DLV-002.1, AT-DLV-002.2, AT-DLV-002.3, AT-DLV-002.4, AT-DLV-002.5, AT-DLV-002.6. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: ConsumerController: Ack opcional, Ack al detenerse y DeliverThenStop (+5 filas)

Filas: AT-DLV-002.7, AT-DLV-003.1, AT-DLV-003.2, AT-DLV-003.3, AT-DLV-003.4, AT-DLV-003.5. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: WorkPullingProducerController: cola duradera, preselección de worker y traspaso (+5 filas)

Filas: AT-DLV-003.6, AT-DLV-004, AT-DLV-005.1, AT-DLV-005.2, AT-DLV-005.3, AT-DLV-006. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: Flight recorder (JFR) de actor y delivery

Filas: AT-DLV-007. Contratos y aserciones: `docs/inventory/contracts/typed-extended.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AT-DLV-001.1 | ProducerController: protocolo Start/RequestNext y demanda local uno a uno | actor `ProducerController<A>` con `enum ProducerCommand<A> {… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1, Q-X-5, Q-X-8 |
| AT-DLV-001.2 | ProducerController: secuenciación, ventana de demanda y confirmación por… | `struct SequencedMessage<A> { producer_id, seq_nr: u64, message: MessageOrChunk<A>,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1, Q-T-DLV-2 |
| AT-DLV-001.3 | ProducerController: reenvío por Resend, Request vencida y reenvío del primero | timer `ResendFirst` con `tokio::time::interval`/`ManualTime` en tests; `unconfirmed:… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1, Q-T-DLV-4 |
| AT-DLV-001.4 | ProducerController: RegisterConsumer, reemplazo del ConsumerController y envío… | `send: Box<dyn Fn(SequencedMessage<A>)>` reemplazable en el estado; la variante con… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1, Q-X-10 |
| AT-DLV-001.5 | ProducerController: MessageWithConfirmation (respuesta de confirmación con el… | `reply_after_store: BTreeMap<u64, ActorRef<u64>>`; el productor usa `context.ask` con… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1, Q-T-DLV-2 |
| AT-DLV-001.6 | ProducerController: integración con DurableProducerQueue (carga,… | `Option<Behavior<DurableCommand<A>>>` + hijo `"durable"`; implementación en memoria… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerCon… | Q-T-DLV-1, Q-T-DLV-2, Q-X-10 |
| AT-DLV-002.1 | ConsumerController: arranque con Start, mensajes previos a Start y parada con… | `ConsumerController<A>` con `enum ConsumerCommand<A> { Start(ActorRef<Delivery<A>>),… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1 |
| AT-DLV-002.2 | ConsumerController: entrega uno a uno, Confirmed y cola mientras se espera… | `StashBuffer` acotado + estado `Waiting{seq_msg}`; `Delivery<A> { message, confirm_to:… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1 |
| AT-DLV-002.3 | ConsumerController: ventana de flujo, Request y reintentos con backoff | `RetryTimer { min, max, interval }` con `schedule_next`/`reset`/`start` y temporizador… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1, Q-T-DLV-4 |
| AT-DLV-002.4 | ConsumerController: detección de pérdidas, Resend y estado de reenvío | estados `Active`/`Resending`/`Waiting` como enum de estado interno del actor;… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1, Q-T-DLV-4 |
| AT-DLV-002.5 | ConsumerController: modo only-flow-control (sin reenvío de perdidos) | `ConsumerSettings { only_flow_control: bool }` y la rama `resend_lost` en el estado.… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1, Q-T-DLV-4 |
| AT-DLV-002.6 | ConsumerController: registro y cambio de ProducerController, deduplicación del… | campo `registering: Option<ActorRef<ProducerCommand<A>>>` en el estado. **Se preserva**:… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1, Q-T-DLV-4 |
| AT-DLV-002.7 | ConsumerController: Ack opcional, Ack al detenerse y DeliverThenStop | `DeliverThenStop` como comando; `on_stop` envía `Ack`. **Se preserva**: condición de… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1 |
| AT-DLV-003.1 | WorkPullingProducerController: workers registrados vía Receptionist y… | `Receptionist::subscribe(worker_key)` + mapa `out_key -> OutState`; hijos… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.sc… | Q-T-DLV-1, Q-T-RCP-1 |
| AT-DLV-003.2 | WorkPullingProducerController: demanda, selección aleatoria de worker y buffer | `rand::thread_rng()` para la selección; `StashBuffer` acotado a `buffer_size`; los… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.sc… | Q-T-DLV-1, Q-T-DLV-4 |
| AT-DLV-003.3 | WorkPullingProducerController: MessageWithConfirmation con Done, Ack del… | `ask` con timeout `internal_ask_timeout`; `Unconfirmed { total_seq_nr, out_seq_nr, msg,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.sc… | Q-T-DLV-1, Q-X-9 |
| AT-DLV-003.4 | WorkPullingProducerController: reasignación de no confirmados cuando un worker… | reenviar `Msg` al propio buzón con `was_stashed = true`. **Se preserva**: reasignación… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.sc… | Q-T-DLV-1 |
| AT-DLV-003.5 | WorkPullingProducerController: Start, reinicio del productor y estado inicial | estado `WaitingForStart` + `StashBuffer`. **Se preserva**: guardar todo hasta `Start`,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/WorkPullingSpec.sc… | Q-T-DLV-1 |
| AT-DLV-003.6 | WorkPullingProducerController: cola duradera, preselección de worker y traspaso | igual que AT-DLV-001.6 con `preselected_workers: HashMap<u64, Preselected>` y… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableWorkPulling… | Q-T-DLV-1, Q-T-DLV-2 |
| AT-DLV-004 | DurableProducerQueue (protocolo) | `trait`/protocolo `DurableCommand<A>`; `struct DurableState<A> { current_seq_nr,… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/DurableProducerQue… | Q-T-DLV-2 |
| AT-DLV-005.1 | Chunking: partición de mensajes grandes en el ProducerController | requiere un mecanismo de serialización de mensajes (Q-T-SER-1): `trait… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ProducerController… | Q-T-DLV-1, Q-T-SER-1 |
| AT-DLV-005.2 | Chunking: recolección y reensamblado en el ConsumerController | `collected_chunks: Vec<SequencedMessage>` y `MessageSerializer::deserialize`. **Se… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-T-DLV-1, Q-T-SER-1 |
| AT-DLV-005.3 | Chunking con DurableProducerQueue: almacenamiento por trozo, reintento… | igual que AT-DLV-001.6 más el algoritmo de re-almacenamiento completo. **Se preserva**:… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/TestDurableProduce… | Q-T-DLV-1, Q-T-DLV-2, Q-T-SER-1 |
| AT-DLV-006 | Configuracion pekko.reliable-delivery.* y Settings | structs `ProducerSettings`/`ConsumerSettings`/`WorkPullingSettings` con `Default`… | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/delivery/ConsumerController… | Q-UTIL-1, Q-X-1, Q-X-9 |
| AT-DLV-007 | Flight recorder (JFR) de actor y delivery | eventos `tracing` (nivel `trace`/`debug`, `target = "pekko::delivery"`) con los mismos… | derivada del código (ver bloque de contrato) | Q-T-DLV-3 |

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

Spec 50 `AT-JAPI-1` (`50-at-japi-1.md`).
