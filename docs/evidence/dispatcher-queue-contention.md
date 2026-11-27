# Contención de `ThreadPool` (Q-MBX-16, opción (c) del Owner, 2026-11-27)

Carga: el ping-pong de `MailboxConfigSpec.scala:279-329` (2.000.000 de mensajes, `throughput = 1`, 8 hilos), `tests/classic_single_consumer_mailbox.rs`.

## Entorno y línea base

Apple M4 Max (12 núcleos P + 4 E), 64 GiB, macOS (Darwin 27.2.0), rustc 1.100.0 `aarch64-apple-darwin`, perfil `test` (sin optimizar). Máquina compartida con otros agentes: la carga se anota por ejecución. Base: `origin/feat/pekko-rust-pilot-migration` @ `2054d3bb`.

```
cargo test -p rakka-actor --test classic_single_consumer_mailbox --no-run          # binario normal
cargo llvm-cov --no-report -p rakka-actor --test classic_single_consumer_mailbox -- --list   # instrumentado
# copiar cada binario (antes y después del cambio) y comparar, intercalados:
python3.12 tools/perf/pingpong_stats.py --binary before=A --binary after=B --runs 30 --log runs.tsv
#   gate:       binarios llvm-cov, añadir --together
#   contención: binarios llvm-cov, añadir --together --background --burners 16
```

Línea base antes de tocar el ejecutor (6 ejecuciones por test, carga 10-26): sin límite 8,8 / 12,1 / 15,9 s (mín/mediana/máx); acotado 8,7 / 11,8 / 16,4 s.

## Mecanismo

**Observación** (`sample`, 4 s a 1 ms): cada uno de los 8 workers está ~1100 de 1313 muestras en `__psynch_mutexwait` (el `Mutex<Receiver>`), 72-96 aparcado dentro de `recv` y 30-56 en `__psynch_mutexdrop`. Esto dice dónde esperan los workers ociosos, no por qué es lento.

**Barrido** (copia temporal fuera del repositorio, instrumentada con contadores; test sin límite; dos pasadas, segundos; carga 10-18):

| Hilos | `mpsc` + `Mutex` (antes) | `Condvar` simple | `Condvar`, un solo despertar en vuelo (adoptado) | cada worker se queda lo que envía (descartado) |
|---|---|---|---|---|
| 1 | 2,1 / 2,1 | 2,0 / 2,0 | 2,8 / 2,9 | 1,9 / 2,5 |
| 2 | 5,4 / 4,5 | 3,0 / 3,2 | 4,8 / 3,2 | 2,1 / 1,8 |
| 4 | 15,3 / 7,4 | 6,5 / 9,2 | 5,3 / 3,7 | 2,6 / 1,7 |
| 8 | 8,5 / 8,6 | 9,4 / 11,9 | 5,5 / 4,1 | 2,5 / 1,8 |
| 16 | 8,0 / 9,7 | 11,8 / 7,9 | 4,9 / 6,1 | 2,0 / 1,8 |

Contadores con 8 hilos (primera pasada, ~2,0 M de trabajos):

| Variante | veces que un worker se duerme | el trabajo cambia de worker | cambios de contexto involuntarios | `sys` |
|---|---|---|---|---|
| antes | 743 k (+776 k esperas del cerrojo) | 925 k | 2,49 M | 10,5 s |
| `Condvar` simple | 1,43 M | 972 k | 1,74 M | 14,1 s |
| un despertar en vuelo | 457 k | 212 k | 0,64 M | 5,4 s |
| se queda lo que envía | 18 | 8 | 2 k | 0,03 s |

**Conclusión.** El coste es el relevo entre hilos: cada envío despierta a un worker dormido aunque el emisor vaya a quedar libre enseguida, y el trabajo cambia de núcleo. Sin relevos (1 hilo, o la última columna) el test tarda ~2 s, así que el bit `Scheduled`, la cola y `throughput = 1` en sí (~1 µs por mensaje) no explican el resto. El cerrojo retenido durante `recv` **no es la causa por sí solo**: duplica los despertares (receptor y siguiente en el cerrojo), pero quitarlo (`Condvar` simple) no mejora con 4 o más hilos. El prototipo de #443 (5,6-10,5 s) era ruido o carga.

Alternativas: la última columna es la más rápida y **se descarta**: un turno que se bloquea retendría el trabajo que envió (test `a_task_queued_by_a_blocked_worker_runs_on_another_one`); hacerlo seguro exige colas por worker con robo, que es una unidad propia. Girar antes de dormir gasta CPU justo bajo contención (no medido). Dependencia nueva: ninguna.

Carga paralela (4 y 64 parejas a la vez, 3 pasadas, copia temporal), para descartar una regresión: 4 parejas 5,6-5,9 s antes frente a 4,3-5,0 s; 64 parejas 3,7-4,0 s frente a 3,1-3,6 s.

## Corrección y su razonamiento

`RunQueue` (estado puro bajo un `Mutex`) + `Condvar`. Un envío o un `pop` piden despertar sólo si hay trabajo, hay un dormido y **no hay otro despertar en vuelo**.

- **Sin wakeups perdidos.** Al soltar el cerrojo se cumple: cola vacía, o nadie duerme, o hay un despertar en vuelo. Un worker sólo se duerme con la cola vacía, contándose bajo el mismo cerrojo que `wait` libera. Cada dormido contado vuelve por `wake_up`, que borra la marca, y mira la cola; si quedan trabajos despierta al siguiente **antes** de ejecutar el suyo. Un despertar espurio sólo puede causar un `notify` de más.
- **Orden de memoria:** todo el estado se lee y escribe bajo el mutex; no hay atómicos nuevos. `notify` va fuera del cerrojo.
- **Sin interbloqueo:** ningún cerrojo se mantiene al ejecutar o al soltar un trabajo (los rechazados y los abandonados se sueltan fuera).
- **Apagado:** `close` + `notify_all`; lo encolado se ejecuta, lo posterior se rechaza, idempotente; `Drop` del pool equivale a `shutdown` (antes lo hacía soltar el `Sender`).
- **Hilos muertos:** como antes, si mueren todos los workers el pool rechaza y suelta lo encolado.
- **Justicia:** una sola cola FIFO, como antes; ningún trabajo se queda en un worker.
- **Cambio de planificación:** una ráfaga despierta a los workers en cadena (cada uno al siguiente) en lugar de uno por envío.

## TDD

Antes del cambio, 5 tests de guarda del pool (`dispatch/thread_pool_tests.rs`) en verde: `test result: ok. 5 passed`. `cargo test -p rakka-actor --lib thread_pool_tests`:

- RED 1: `error[E0432]: unresolved import crate::dispatch::dispatcher::RunQueue`.
- RED 2 (cola que despierta si hay dormidos): `no_second_worker_is_notified_while_one_is_waking`, `left: Ok(true)`, `right: Ok(false)`; 9 passed, 1 failed.
- RED 3 (pool sobre la cola nueva, sin `Drop`): `dropping_the_pool_ends_its_worker`, `left: Err(Timeout)`, `right: Err(Disconnected)`.
- GREEN: 10 passed; `cargo test --workspace` en verde.

No hay test de tiempo. Mutante registrado y **no ejecutado**: `pool-notifies-while-a-worker-is-already-waking`.

## Antes y después

**PARCIAL (pausado por el Owner, 2026-11-27); no sirve para decidir.** Sólo la condición «idle» (inicio de ronda con carga < 8), 9 de 30 rondas; binarios normales, intercalados; datos en `dispatcher-queue-contention-idle-partial.tsv` (versión, test, ronda, segundos, pasó, carga 1 min: 6,8-16,9). Con n = 9 el p95 coincide con el máximo.

| Versión, test | n | mín | mediana | máx | fallos |
|---|---|---|---|---|---|
| antes, sin límite | 9 | 7,63 | 8,84 | 15,21 | 0 |
| después, sin límite | 9 | 2,90 | 3,16 | 4,19 | 0 |
| antes, acotado | 9 | 7,81 | 8,79 | 18,78 | 0 |
| después, acotado | 9 | 3,03 | 4,12 | 5,89 | 0 |

Pendiente: idle completo, gate (`llvm-cov`, ambos tests en un proceso) y contención (16 quemadores + QoS background).

## Re-evaluación de la opción (b) (30 s como gate)

Pendiente de las mediciones de gate y contención.

## Política de ejecución propuesta para las mediciones `#[ignore]` (las de #443)

Propuesta; decide el Owner. No cambia `AGENTS.md`.

- **Cuándo:** en todo PR que toque `crates/rakka-actor/src/dispatch/` o la planificación de `actor/cell.rs`; en cada gate MILESTONE; antes de un release.
- **Quién:** el autor del PR (persona o agente); en MILESTONE y release, quien ejecute el gate. El Owner revisa.
- **Comando:** con carga a 1 min < 8, `cargo test -p rakka-actor --test classic_single_consumer_mailbox -- --ignored --test-threads=1 --nocapture`; si el PR cambia el ejecutor, además `tools/perf/pingpong_stats.py` antes/después.
- **Dónde:** una fila (fecha, SHA, máquina, carga, toolchain, perfil, tiempos, resultado) en este documento, citada en el PR. Un fallo se anota y se informa; no se reintenta hasta que pase.

## Límites

Sólo macOS en Apple Silicon (Linux sin medir); máquina compartida; el tiempo sigue creciendo con la carga ajena; queda ~2x sobre el coste sin relevos.
