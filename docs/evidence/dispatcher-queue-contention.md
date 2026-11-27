# Contención de `ThreadPool` (Q-MBX-16, opción (c) del Owner)
Carga: ping-pong de `MailboxConfigSpec.scala:279-329` (2.000.000 de mensajes, `throughput = 1`, 8 hilos). Apple M4 Max (12 P + 4 E), 64 GiB, Darwin 27.2.0, rustc 1.100.0, perfil `test`, máquina compartida. **Reproducir:** en la base y en la rama, `cargo test -p rakka-actor --test classic_single_consumer_mailbox --no-run` (instrumentado: `cargo llvm-cov --no-report -p rakka-actor --test classic_single_consumer_mailbox -- --list`); después `caffeinate -i python3.12 -I tools/perf/pingpong_stats.py --binary before=A --binary after=B --runs 30 --log runs.tsv` (gate: instrumentados y `--together`; contención: además `--background --burners 16`). Línea base previa (`2054d3bb`, n = 6, carga 10-26), mín/mediana/máx: sin límite 8,8/12,1/15,9 s; acotado 8,7/11,8/16,4 s.
## Mecanismo
`sample` (4 s): cada worker pasaba ~1100 de 1313 muestras en `__psynch_mutexwait` del `Mutex<Receiver>`; eso muestra dónde esperan los ociosos, no la causa. Barrido en una copia temporal instrumentada, test sin límite, dos pasadas (s):

| Variante \ hilos | 1 | 2 | 4 | 8 | 16 |
|---|---|---|---|---|---|
| antes (`mpsc` + `Mutex`) | 2,1/2,1 | 5,4/4,5 | 15,3/7,4 | 8,5/8,6 | 8,0/9,7 |
| `Condvar` simple | 2,0/2,0 | 3,0/3,2 | 6,5/9,2 | 9,4/11,9 | 11,8/7,9 |
| un despertar en vuelo (adoptada) | 2,8/2,9 | 4,8/3,2 | 5,3/3,7 | 5,5/4,1 | 4,9/6,1 |
| quien envía se queda el trabajo (descartada) | 1,9/2,5 | 2,1/1,8 | 2,6/1,7 | 2,5/1,8 | 2,0/1,8 |

Con 8 hilos y ~2,0 M de trabajos, antes → adoptada → descartada: sueños de un worker 743 k → 457 k → 18; cambios de worker 925 k → 212 k → 8; cambios de contexto involuntarios 2,49 M → 0,64 M → 2 k; `sys` 10,5 → 5,4 → 0,03 s (`Condvar` simple: 1,43 M, 972 k, 1,74 M, 14,1 s). **El coste es el relevo entre hilos:** cada envío despierta a un worker dormido aunque el emisor vaya a quedar libre. Sin relevos el test tarda ~2 s: ni el bit `Scheduled`, ni la cola, ni `throughput = 1` en sí explican el resto. El cerrojo retenido en `recv` **no es la causa por sí solo** (`Condvar` simple no mejora desde 4 hilos; el prototipo de #443 era ruido). La descartada es la más rápida, pero un turno bloqueado retendría lo que envió; haría falta robo de trabajo (unidad propia). Girar antes de dormir no se midió. Carga paralela (3 pasadas): 4 parejas 5,6-5,9 s antes y 4,3-5,0 s después; 64 parejas 3,7-4,0 y 3,1-3,6 s.
## Corrección y razonamiento
`RunQueue` (estado bajo un `Mutex`) + `Condvar`, sin dependencias nuevas. Se despierta sólo si hay trabajo, alguien duerme y **no hay otro despertar en vuelo**.
- Sin wakeups perdidos: al soltar el cerrojo, o la cola está vacía, o nadie duerme, o hay un despertar en vuelo. Sólo se duerme con la cola vacía; quien despierta borra la marca, mira la cola y despierta al siguiente **antes** de ejecutar su trabajo (las ráfagas despiertan en cadena). Un despertar espurio sólo añade un `notify`. Todo el estado va bajo el mutex (sin atómicos) y ningún cerrojo se mantiene al ejecutar o soltar un trabajo.
- Apagado: `closed` + `notify_all`; lo encolado se ejecuta y lo posterior se rechaza; `Drop` equivale a `shutdown`. Una sola cola FIFO, como antes. Panics (Q-MBX-17 sigue abierta, sin cambios): el turno se atrapa en `cell.rs`, las tareas en el pool; si aun así muriesen todos los hilos, el pool rechaza y suelta lo encolado, como antes.
## TDD
Antes del cambio, 5 tests de guarda del pool en verde. `cargo test -p rakka-actor --lib thread_pool_tests` (base `2054d3bb`):
- RED 1: `error[E0432]: unresolved import crate::dispatch::dispatcher::RunQueue`.
- RED 2 (despertar si alguien duerme): `no_second_worker_is_notified_while_one_is_waking`, `left: Ok(true)`, `right: Ok(false)`; 9 passed, 1 failed.
- RED 3 (sin `Drop`): `dropping_the_pool_ends_its_worker`, `left: Err(Timeout)`, `right: Err(Disconnected)`.
- GREEN: 10 passed (hoy 9: dos unidos); `cargo test --workspace` en verde sobre `c4338dd7`. Sin tests de tiempo. Mutante `pool-notifies-while-a-worker-is-already-waking` registrado, **no ejecutado**.
## Antes y después
Antes = `c4338dd7`, después = esta rama; intercaladas, `caffeinate -i`, ningún otro build, **con batería** (sin modo de bajo consumo). Excluidas por suspensión (cierre de la tapa, 16:50:01) 3 muestras de la última condición. Escenarios **concurrentes** (planificación no determinista): una ejecución no prueba nada. n = 30 (p95 por rango = 29.º valor) en idle y gate; n = 20 (19.º) bajo contención; sin afirmaciones sobre p99. Segundos: mín / mediana / p95 / máx. En idle se excluyeron además dos muestras «antes» (~9,8 s) que corrieron solapadas entre dos tandas mías y se añadió una ronda.

| Condición (carga 1 min) | n | antes | después | fallos de 30 s |
|---|---|---|---|---|
| idle, test sin límite (3,5-5,8) | 30 / 31 | 7,60 / 7,69 / 7,81 / 7,88 | 2,98 / 3,03 / 3,09 / 3,10 | 0 |
| idle, test acotado (3,5-5,8) | 30 / 31 | 7,63 / 7,72 / 7,83 / 7,92 | 2,98 / 3,01 / 3,08 / 3,09 | 0 |
| gate: `llvm-cov`, ambos tests a la vez (4-7) | 30 | 11,70 / 11,84 / 12,00 / 12,32 | 6,63 / 6,71 / 6,79 / 6,83 | 0 |
| gate + 16 quemadores de CPU (23-31) | 20 | 9,33 / 10,07 / 10,82 / 10,95 | 6,97 / 7,50 / 7,80 / 7,89 | 0 |
| gate + 16 quemadores + QoS background (6-78); censurado: el test aborta a los 30 s | 19 / 18 | 30,03 / 30,11 / 32,15 / 32,15 | 30,07 / 30,14 / 31,02 / 31,02 | todas: 19/19 y 18/18 |

## Re-evaluación de la opción (b) (30 s como gate)
- Con la corrección el peor caso medido sin inanición deja un margen de ~3,8x (gate ~4,4x, idle ~10x). Antes el margen en gate era 2,4x y, con carga ajena, #443 llegó a superar los 30 s.
- Con QoS background y la CPU sobresuscrita **ambas versiones fallan siempre**: ahí el límite mide al planificador del SO, no a rakka; la corrección no lo arregla.
- **Recomendación (decide el Owner): (b)**, mantener el `within(30.seconds)` original en el gate habitual (hoy sigue en `c4338dd7`; no fusionar su exclusión de #443), sin subirlo ni reintentar, con el gate nunca en QoS background. Sin medir: Linux y hardware de CI.
## Política propuesta si hay mediciones `#[ignore]`, como las de #443 (decide el Owner)
- **Cuándo:** PR que toque `dispatch/` o la planificación de `actor/cell.rs`, cada gate MILESTONE y antes de un release. **Quién:** el autor del PR o quien ejecute el gate; el Owner revisa. **Comando:** carga a 1 min < 8, `caffeinate -i cargo test -p rakka-actor --test classic_single_consumer_mailbox -- --ignored --test-threads=1 --nocapture`; si cambia el ejecutor, además el script antes/después.
- **Registro:** fila aquí (fecha, SHA, máquina, carga, toolchain, perfil, tiempos, resultado), citada en el PR. Un fallo se informa; no se reintenta hasta que pase. **Límites de esta unidad:** sólo macOS en Apple Silicon; sigue dependiendo de la carga ajena; queda ~2x sobre el coste sin relevos.
