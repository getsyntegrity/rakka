# MBX PR 7b: `NonBlockingBoundedMailbox` (AC-MBX-008)

Cola acotada que nunca bloquea al emisor y manda el excedente a `deadLetters` (`Mailbox.scala:421-452,707-724`,
`AbstractBoundedNodeQueue.java`). **No** es `BoundedMailbox` (bloqueante): Q-MBX-1 sigue `proposed` y nada de este PR la decide.

## Alcance

Implementado: `NonBlockingBoundedMailbox` (constructor con rechazo de capacidad negativa, `create`) y su cola
(`dispatch/bounded_node_queue.rs`), que implementa el trait `MessageQueue` existente sin cambiarlo.

No implementado (queda para otros PR): cableado en `ActorCell`/`Dispatcher`, `Mailboxes.lookup`, lectura de
`mailbox-capacity` desde `Config` (NB-CF-02, PR 10), el port 1:1 de `MailboxConfigSpec` con actores reales
(20 enqueue/10 dequeue ⇒ 10 `DeadLetter`), `BoundedMailbox` bloqueante (Q-MBX-1), capacidad 0 de `BoundedMailbox` (PROP-7A-1).

Capacidad 0 en **esta** cola no es una decisión: Pekko la acepta y la cola rechaza todo (`NB-CF-01`, `NB-QU-03`).

## Cambios de API

- Nuevo, público: `NonBlockingBoundedMailbox::{new(i32) -> Result<_, NegativeCapacity>, capacity, create(owner, dead_letters)}`, `NegativeCapacity`.
- Interno: `dead_letter_mailbox::tell_dead_letter` (extraído de `DeadLetterMailbox::enqueue`, sin cambio de conducta) y el doble de test
  `dispatch::test_support::Recorder` (movido desde `dead_letter_mailbox`).
- **El trait `MessageQueue` no cambia.** Pekko pasa el receptor en cada `enqueue(receiver, handle)` y llega a `deadLetters` por su proveedor.
  Aquí el dueño y `deadLetters` se dan al crear la cola (el equivalente de `create(owner, system)`); el receptor de la cola de un mailbox es
  siempre su dueño. **Propuesta `PROP-7B-1`** (sin decisión del Owner; no se registró un `Q-*` para no chocar con las propuestas paralelas de
  #329/#330): si el Dispatcher necesita pasar el receptor por llamada, se añade el parámetro al trait en el PR que lo cablee.
- La cola usa un `Mutex<VecDeque>` en lugar de la cola sin bloqueos de Pekko; sólo se mantiene durante el push o el pop y el excedente se manda a
  `deadLetters` después de soltarlo (probado con un `deadLetters` que reentra en la cola).

## Contratos JVM comprobados (oráculo Pekko 2.0.0, JDK 17)

Oráculo nuevo `NonBlockingBoundedMailboxOracle.java` (3 `case` nuevos en `Cli.java`; no toca otros oráculos ni goldens):
`oracle/run.sh nonblocking-bounded-mailbox-all` → `cases/oracle-only/nonblocking-bounded-mailbox.golden` (10 casos, manifiesto
`nonblocking-bounded-mailbox-cases.tsv`). 6 corridas consecutivas con salida idéntica byte a byte (md5 único).

| Caso | Gemelo Rust |
|---|---|
| NB-CF-01 constructor: −1 y `Int.MIN` ⇒ IAE con el texto de Pekko; 0, 1 y `Int.MAX` pasan | `nb_cf_01_matches_the_oracle` (línea a línea con el golden) |
| NB-QU-01 llena: rechaza al instante, conserva las primeras, los rechazados a dead letters en orden | `nb_qu_01_…` |
| NB-QU-02 `dequeue` libera hueco, sigue FIFO | `nb_qu_02_…` |
| NB-QU-03 capacidad 1 conserva una; capacidad 0 rechaza todo | `nb_qu_03_…` |
| NB-QU-04 el `DeadLetter` lleva mensaje, emisor del sobre (o `deadLetters`) y receptor; un `DeadLetter` rebosado **no** se descarta | `nb_qu_04_…` |
| NB-QU-05 `cleanUp` mueve en orden a la cola destino y libera los huecos | `nb_qu_05_…` |
| NB-QC-01 4 productores sin consumidor: aceptados = capacidad, el resto dead letters, sin pérdida ni duplicados | `concurrent_producers_nobody_consuming_…` (propiedad: totales y orden por productor) |
| NB-QC-02 4 productores y un consumidor: cada mensaje exactamente una vez, orden por productor, nunca sobre la capacidad | `concurrent_producers_with_a_consumer_…` |

Sin gemelo (con motivo): **NB-CF-02** (`getInt` de `mailbox-capacity`, truncado y errores de Config) es del dominio `Mailboxes`/Config, PR 10;
**NB-CF-03** (nombre de clase JVM y `pushTimeOut = Duration.Undefined`) son hechos de la JVM (la parte de capacidad la cubre NB-CF-01).
Todos los casos son unitarios (cola sola); no hay caso de integración hasta que el Dispatcher use esta cola.

## TDD

RED real (stub que aceptaba todo, antes del código de producción): `cargo test -p rakka-actor --lib bounded_node_queue` →
`test result: FAILED. 2 passed; 16 failed` (p. ej. `nb_qu_01`: `left: "…d: n=4 … dead=[] | drain=[a, b, c, d, e]"` contra
`right: "…d: n=3 … dead=[d<-snd->recv, e<-snd->recv] | drain=[a, b, c]"`; `concurrent_producers_nobody_consuming…`: `left: (200, 0) right: (10, 190)`).
GREEN: `test result: ok. 18 passed; 0 failed`. Refactor (extraer `tell_dead_letter`, probar la reentrada con un hilo y plazo para que el fallo no cuelgue)
sin cambiar conducta: 571 pruebas de la librería y los de integración del workspace en verde.

## Gates

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`: verdes. `cargo deny check licenses`: ok.
- `uv run tools/package/generate.py` + `check.py`: `failures=0`.
- Cobertura (`cargo llvm-cov --lib -p rakka-actor`): todas las líneas de producción de `bounded_node_queue.rs` cubiertas (100 %); las únicas sin cubrir
  del archivo son los métodos de relleno del doble de test `CallsBack` (`path`, `send_system`, `stop`, `is_terminated`).
- Mutantes nuevos (`tools/parity/check_divergence.py`, solo ellos + control): 7/7 DETECTED, control CLEAN.

```
nonblocking-queue-keeps-one-over-capacity | nonblocking-queue-drops-the-overflow-silently | nonblocking-queue-dead-letters-to-the-wrong-receiver
nonblocking-queue-holds-its-lock-while-dead-lettering (falla a los 2 s, no cuelga) | nonblocking-queue-dequeues-the-newest
nonblocking-queue-accepts-a-negative-capacity | nonblocking-queue-clean-up-moves-only-the-first
control (unmodified) CLEAN 2960/2960 applicable + 0 NA:Q-CFG-30 + 1 not applicable (diverge 0)
```

`--check-patterns`: 261 patrones, 2 stale preexistentes (`deploy-fallback-keeps-the-tags`, `render-concatenation-keeps-white-space-as-a-separate-string`), ninguno de este PR.
No se ejecutó la batería global de mutantes.

## Revisión independiente (HEAD 539927b)

Veredicto: aprobado, sin bloqueantes (paridad de capacidad incl. 0, contenido del `DeadLetter`, sustituto de emisor, `DeadLetter` rebosado no descartado,
`cleanUp`, ausencia de bloqueo por reentrada, ausencia de pérdida y duplicación). Notas aceptadas como riesgos de integración:

- **Dueño fijado en `create`.** Si el cableado futuro de `ActorCell`/`Dispatcher` construye la cola antes de que exista el `ActorRef` del dueño, el dueño
  habrá de fijarse después de `create` (p. ej. `OnceLock`) o pasar el receptor por llamada (PROP-7B-1). Un reinicio no afecta (el dueño es estable) y
  Pekko excluye esta cola en mailboxes compartidos (`BalancingPool`).
- **NB-CF-02 y NB-CF-03 sin gemelo Rust:** quedan para el PR 10 (`Mailboxes`/Config) y para el cableado del Dispatcher, respectivamente.
- `lock().unwrap()` sólo entra en pánico con un lock envenenado, inalcanzable porque no se ejecuta código ajeno bajo el lock.
