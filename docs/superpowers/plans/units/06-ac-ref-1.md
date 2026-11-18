# Spec 06 — AC-REF (1/2)

**Estado:** pendiente de decisiones propuestas. **Filas:** 30. **Tareas:** 5 (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.

## Alcance y exclusiones

Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-classic.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.

Filas ya `done` en esta unidad (no se reabren): AC-REF-009.3.

## Prerrequisitos

- Spec 01 `AC-PATH-1` (`01-ac-path-1.md`), por las filas AC-PATH-008, AC-PATH-014, AC-PATH-015.
- Spec 08 `AC-MBX-1` (`08-ac-mbx-1.md`), por las filas AC-MBX-001, AC-MBX-002, AC-MBX-003, AC-MBX-005. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 13 `AC-CELL-1` (`13-ac-cell-1.md`), por las filas AC-CELL-002.1, AC-CELL-002.2, AC-CELL-003, AC-CELL-008.2, AC-CELL-013.1, AC-CELL-013.3, AC-CELL-015.1, AC-CELL-016. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 05 `AC-CFG-1` (`05-ac-cfg-1.md`), por las filas AC-CFG-013.
- Spec 19 `AC-ASK-1` (`19-ac-ask-1.md`), por las filas AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 11 `AC-EVT-1` (`11-ac-evt-1.md`), por las filas AC-EVT-008. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 15 `AC-SYS-1` (`15-ac-sys-1.md`), por las filas AC-SYS-012, AC-SYS-018. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 14 `AC-CELL-2` (`14-ac-cell-2.md`), por las filas AC-CELL-027. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**
- Spec 23 `AC-SER-1` (`23-ac-ser-1.md`), por las filas AC-SER-007.1, AC-SER-007.2. **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**

Decisiones:

- Bloqueantes: ninguna.
- Propuestas aún sin decisión del Owner: `Q-REF-1`, `Q-REF-2`, `Q-REF-3`, `Q-REF-4`, `Q-REF-5`, `Q-REF-6`, `Q-X-2`, `Q-X-3`, `Q-X-8`.
- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.

## Estrategia de oráculo y procedencia

casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.

## Tareas

### Tarea 1: Identidad de ActorRef: equals y hashCode (+5 filas)

Filas: AC-REF-001.1, AC-REF-001.2, AC-REF-001.3, AC-REF-002, AC-REF-003, AC-REF-004. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 2: sender() (+5 filas)

Filas: AC-REF-005, AC-REF-006, AC-REF-007.1, AC-REF-007.2, AC-REF-008, AC-REF-009.1. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 3: actorOf: unicidad y reserva atómica del nombre (+5 filas)

Filas: AC-REF-009.2, AC-REF-010, AC-REF-011, AC-REF-012, AC-REF-013, AC-REF-014. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 4: EmptyLocalActorRef (ref a actor inexistente) (+5 filas)

Filas: AC-REF-015, AC-REF-016, AC-REF-017, AC-REF-018, AC-REF-019, AC-REF-020. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

### Tarea 5: FunctionRef (ref respaldado por una función, hijo virtual del actor) (+5 filas)

Filas: AC-REF-021, AC-REF-022, AC-REF-023, AC-REF-024, AC-REF-025, AC-REF-026. Contratos y aserciones: `docs/inventory/contracts/classic-cell.md` (bloque de cada fila; sección **Aserciones**).

1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.
2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.
3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.
4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.

## Trazabilidad contrato → aserción → validación

| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |
|---|---|---|---|---|
| AC-REF-001.1 | Identidad de ActorRef: equals y hashCode | `ActorRef` con `PartialEq/Eq/Hash` por (path, uid); `ActorIncarnationUid` no nulo para… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:124-148 | Q-REF-6 |
| AC-REF-001.2 | Identidad de ActorRef: compareTo | `impl Ord for ActorRef`: orden por `ActorPath` y luego `uid` | derivada del código (ver bloque de contrato) | Q-REF-6 |
| AC-REF-001.3 | Identidad de ActorRef: toString | `impl Display for ActorRef` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:139-143,346-34… | - |
| AC-REF-002 | tell / `!` (fire-and-forget) y Envelope | `fn tell(&self, msg: Message, sender: Option<&ActorRef>)`; el Envelope guarda `sender:… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:390-413 ;… | Q-REF-1, Q-X-3 |
| AC-REF-003 | Garantías de entrega: at-most-once y orden por par emisor–receptor | cola MPSC FIFO por actor | actor-tests/src/test/scala/org/apache/pekko/actor/ConsistencySpec.scala:41-81 ;… | - |
| AC-REF-004 | forward (preserva el sender original) | `fn forward(&self, msg, ctx: &Context)` | actor-tests/src/test/scala/org/apache/pekko/actor/ForwardActorSpec.scala:45-60 ;… | - |
| AC-REF-005 | sender() | `ctx.sender() -> ActorRef` (deadLetters por defecto) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:390-413 ;… | Q-REF-1 |
| AC-REF-006 | Los actores sólo se crean vía actorOf | la restricción es estructural: `Actor` no tiene constructor con acceso a `Context`;… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:148-175 | Q-REF-4 |
| AC-REF-007.1 | Props: formas de creación, valores por defecto y validación | `Props<A>` con fábrica `Arc<dyn Fn() -> A + Send + Sync>`; `Props::empty()`,… | actor-tests/src/test/scala/org/apache/pekko/actor/PropsCreationSpec.scala:52-73 | Q-REF-2, Q-X-2 |
| AC-REF-007.2 | Props: modificadores… | `Props::with_dispatcher(&self, id) -> Props` (copia; Arc compartido) | derivada del código (ver bloque de contrato) | Q-REF-2 |
| AC-REF-008 | IndirectActorProducer y constructores por reflexión | trait `ActorFactory` / closure | actor-tests/src/test/scala/org/apache/pekko/actor/PropsCreationSpec.scala:41-75 ;… | Q-REF-2, Q-X-2 |
| AC-REF-009.1 | actorOf: validación del nombre | `ctx.actor_of(props, name) -> Result<ActorRef, InvalidActorName>` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:167-197 | - |
| AC-REF-009.2 | actorOf: unicidad y reserva atómica del nombre | reserva atómica del nombre en el contenedor de hijos (CAS o lock) | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:133-165… | - |
| AC-REF-010 | children, child(name), getChild, findChild | `ctx.children() -> impl Iterator<Item=ActorRef>`, `ctx.child(&str) -> Option<ActorRef>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:465-477 | - |
| AC-REF-011 | ActorSelection: actorSelection(path\|ActorPath), relativo, `..`, comodines * y… | `ActorSelection { anchor, elements: Vec<SelectionElement> }` con… | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:100-189,193-27… | Q-X-3 |
| AC-REF-012 | resolveOne(timeout) y ActorNotFound | `async fn resolve_one(&self, timeout) -> Result<ActorRef, ActorNotFound>` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:319-335 ;… | - |
| AC-REF-013 | Identify / ActorIdentity | mensaje manejado por la celda antes del receive de usuario | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:70-85,179-189,… | - |
| AC-REF-014 | Entrega de selección con comodines (ActorSelectionMessage,… | matcher glob propio (sin regex) | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:361-369 ;… | - |
| AC-REF-015 | EmptyLocalActorRef (ref a actor inexistente) | variante `RefInner::EmptyLocal` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:341-365 ;… | - |
| AC-REF-016 | deadLetters (DeadLetterActorRef) | ref especial que publica en EventStream | actor-tests/src/test/scala/org/apache/pekko/actor/DeadLetterSupressionSpec.scala:74-106… | - |
| AC-REF-017 | resolveActorRef(path) del proveedor | `provider.resolve_actor_ref(&ActorPath) -> ActorRef` | actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala:48-64… | - |
| AC-REF-018 | Jerarquía interna InternalActorRef / MinimalActorRef / ActorRefWithCell /… | trait sellado `RefInner` con variantes; métodos internos `pub(crate)` | derivada del código (ver bloque de contrato) | - |
| AC-REF-019 | LocalActorRef: getChild por nombres (incluye FunctionRefs) e isTerminated… | `AtomicBool` de mailbox cerrado | actor-tests/src/test/scala/org/apache/pekko/actor/FunctionRefSpec.scala:89-93 ;… | Q-REF-6 |
| AC-REF-020 | RepointableActorRef / UnstartedCell (creación asíncrona) | cola previa en el ref hasta que la celda está lista | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:303-329 ;… | Q-REF-5 |
| AC-REF-021 | FunctionRef (ref respaldado por una función, hijo virtual del actor) | `FunctionRef` con closure `Fn(ActorRef sender, Msg)` | actor-tests/src/test/scala/org/apache/pekko/actor/FunctionRefSpec.scala:58-123 | - |
| AC-REF-022 | IgnoreActorRef | variante `RefInner::Ignore` | derivada del código (ver bloque de contrato) | - |
| AC-REF-023 | Serialización Java de ActorRef (SerializedActorRef, writeReplace/readResolve) | `serialized_actor_path(&ActorRef) -> String` + `provider.resolve_actor_ref` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala:288-365 | Q-REF-3 |
| AC-REF-024 | noSender (null) y sender implícito | `Option<ActorRef>` explícito | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:281-285 ;… | Q-REF-1 |
| AC-REF-025 | ClassicActorSystemProvider / ClassicActorContextProvider | traits `AsClassicSystem`, `AsClassicContext` | actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/MailboxSelectorSpe… | Q-X-8 |
| AC-REF-026 | ActorSelection: equals, hashCode, toString, toSerializationFormat, anchorPath,… | `impl Eq/Hash/Display for ActorSelection`; `to_serialization_format()` | actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala:337-359 | - |

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

Spec 07 `AC-REF-2` (`07-ac-ref-2.md`).
