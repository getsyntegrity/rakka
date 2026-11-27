# AC-MBX-006: `SingleConsumerOnlyUnboundedMailbox`

Pekko 2.0.0: `Mailbox.scala:412-434,699-708`; spec `MailboxConfigSpec.scala:279-329`. Caso grabado MBX-SCQ-01 (`mb-0033`), esperado producido por la JVM.

- Cola ilimitada, varios productores y un consumidor, FIFO; `pathologicalPingPong` portado (sin límite y acotada). Declara lo que declara `NodeMessageQueue` (`Mailbox.scala:412`): `UnboundedMessageQueueSemantics`, sin `MultipleConsumerSemantics` ni `Bounded...`, clase `org.apache.pekko.dispatch.NodeMessageQueue` (Q-MBX-14, aceptada con cambios por el Owner el 2026-11-26).
- Límite: la cola de Pekko es la de Vyukov, sin bloqueos; ésta guarda un `Mutex` sólo durante el push o el pop (comportamiento observable, no velocidad); si un hilo hace panic con el cerrojo tomado, la cola queda inutilizable (`lock().unwrap()`), un modo que la de Pekko no tiene. AC-MBX-006 sigue `partial`.

## TDD

RED 1 (`cargo test -p rakka-actor --lib single_consumer`): `error[E0425]: cannot find value SingleConsumerOnlyUnboundedMailbox`, 7 errores de compilación. GREEN 1: 3 passed.

RED 2 (declaración de semánticas, antes de implementarla): `left: [None, None, None, None]`, `right: [Some(true), Some(true), Some(false), Some(false)]`. GREEN 2: 4 passed.

## Contra la JVM y verificado

- `mb-0033`: clase, `unbounded=true multipleConsumer=false bounded=false`, cola vacía, FIFO, conteos y `cleanUp`. Era `mb-0028`; renumerado al reapilar (la pila ocupa `mb-0028`..`mb-0032`) y regrabado con `rust-anything app record`; su `stdout` es idéntico, byte a byte, a la línea del golden de la JVM.
- Los dos ping-pong de 2.000.000 de mensajes pasan (`cargo test --release`, 9,4 s) y dentro de `cargo test --workspace`. Tres mutantes `single-consumer-*`: resultados en el PR.

## Intermitencia del ping-pong (los dos tests superaron 30 s bajo cobertura)

**Causa raíz:** el test usaba como gate el `within(30.seconds)` de Pekko, que mide rendimiento, sobre una carga cuyo coste es el relevo entre hilos de cada mensaje. No se observó ningún defecto de concurrencia funcional.

Mediciones (macOS, 16 núcleos, perfil `test`, copia instrumentada con contador de entregas y muestra cada 1 s, sin retirar la cota; carga ajena presente):

| Condición | Tiempo por test | Mensajes por segundo, mínimo | Pausas de 1 s |
|---|---|---|---|
| 1 hilo de dispatcher (ambos tests a la vez) | 2,0-2,7 s | 1,4-2,1 M | 0 |
| 2 hilos | 5,1-7,2 s | 450-708 k | 0 |
| 8 hilos (por defecto), 20 ejecuciones | 6,9-24,4 s | 146-570 k | 0 |
| `cargo llvm-cov`, carga 23-63 | 9,9-23,4 s | 100-370 k | 0 |
| `cargo llvm-cov` + `taskpolicy -c background`, carga 44-50 | 23,6 s y **30,1 / 30,5 s** | 45-58 k | 0 |
| tests nuevos, `llvm-cov` + background + 32 procesos de CPU | **140 s**, pasan | - | - |

- Se reprodujo el fallo (ambos tests por encima de 30 s) con las 4.000.000 de entregas completas y progreso continuo: es lentitud, no un bloqueo.
- Pérdida de wakeup / carrera del bit `Scheduled`: `register_for_execution` encola antes de intentar `set_as_scheduled`, y `run_mailbox` hace `set_as_idle` (SeqCst) antes de volver a consultar las colas; las colas usan `Mutex`. Es el protocolo de `Mailbox.scala`. La cola acotada de capacidad 1 nunca desbordó (se habría detenido el ping-pong).
- Coste del relevo: `sample` de un worker con 8 hilos: 520 de 619 muestras en `Mutex<Receiver<Job>>::lock` (`__psynch_mutexwait`). `ThreadPool` mantiene el cerrojo durante `recv`, así que cada trabajo despierta al hilo que espera en el canal y al que espera el cerrojo (5,6 M cambios de contexto involuntarios para 4 M de mensajes).
- **Mejora pendiente, no incluida:** un prototipo temporal (cola `Mutex` + `Condvar`, revertido) midió 5,6-10,5 s frente a 7,8-16,8 s del código actual (sólo el test sin límite, 6 ejecuciones cada uno, carga 15-36). Reduce el coste pero no la dependencia de la carga; cambiar el ejecutor es una unidad propia.

**Corrección (sólo tests, Q-MBX-16 propuesta):**

- Gate funcional: los dos tests afirman exactamente 2.000.000 entregas y la parada del runner; fallan si pasan 10 s sin ninguna entrega (un cuelgue), sin cota para la duración total y sin reintentos. El detector no se ha visto fallar: no hay un defecto real que lo dispare y no se inyectaron defectos.
- Medición: `throughput_of_the_*_ping_pong_is_within_the_thirty_seconds_of_the_spec` (`#[ignore]`) conserva los 30 s: `cargo test -p rakka-actor --test classic_single_consumer_mailbox -- --ignored --test-threads=1 --nocapture` dio 14,2 s y 17,2 s (carga ~17-26).
- Sin cambios de producción: no hay RED/GREEN de producción ni mutantes nuevos.
