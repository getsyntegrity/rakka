# Carrera del helper `interleaving` de `dispatch::dispatcher` (test intermitente)

Test síntoma: `a_deadline_that_no_message_reaches_leaves_the_throughput_as_the_limit`; lo comparten `a_deadline_that_every_message_exceeds_…` y `without_a_deadline_the_throughput_alone_…` (mismo helper). Base de la investigación: `0c518a7`. Seguimiento de la deuda registrada en `docs/evidence/mailbox-requirements/closure.md` (punto 1).

## Causa raíz

El helper crea `x` e `y` con `actor_of_named` y enseguida ocupa el único hilo del dispatcher con un `Gate`. `actorOf` de un actor top-level es asíncrono (`RepointableActorRef`): el guardián `/user` procesa su `Supervise` en el dispatcher interno y recién entonces el actor se adjunta al dispatcher de un hilo. El helper da por hecho que `x` e `y` ya están adjuntos cuando libera el gate; si no lo están, el orden en que entran a la cola del hilo (y si `x` termina su turno antes de que `y` exista) depende de una carrera entre el hilo del test, el guardián y el hilo de trabajo. No es un defecto de producción: Pekko no ordena actores distintos, y su gemelo del oráculo (`IdEnv.spawn`) espera el arranque (`started`) antes de enviar.

Traza (instrumentación temporal, no incluida): en una corrida fallida `start pekko://turns/user/x` y `…/y` aparecen **después** de `gate` y de `run x1`; en una correcta, antes. Con la traza activa, en las cuatro series de 300 corridas (base y HEAD, dos veces) **todos** los fallos (40, 46, 41, 48) tuvieron la creación posterior al gate, y ninguno la tuvo anterior.

## Pruebas causales (controles)

| Experimento | Resultado |
|---|---|
| Sin arreglo + 20 ms de retraso diagnóstico en el `Supervise` de `y` (no se commitea) | **50/50** corridas fallan |
| Con arreglo + el mismo retraso | **0/50** |

## Bisección controlada (módulo `turn_limits_with_real_actors`, 1000 corridas por celda)

| Versión | Sin arreglo | Con arreglo |
|---|---|---|
| base `9b8ea66` (antes del Slice 3) | 413 | **0** |
| núcleo del Slice 3 (`a6f6398`) | 309 | **0** |
| HEAD integrado (`0c518a7`) | 256 | **0** |

Con el test aislado y 200 corridas alternadas, la tasa fue 7 % / 20 % / 12 % (base / núcleo / final); con el módulo entero y 1000 corridas es 41 % / 31 % / 26 %. El signo de la diferencia entre versiones cambia según la medición, así que **no se demuestra que el Slice 3 aumente la tasa**: la tasa depende de la carga y de los tiempos, y la condición que produce el fallo existe en todas las versiones. El arreglo la elimina en todas.

## Arreglo

El helper espera, por evento y sin sleeps ni reintentos, a que `x` e `y` ejecuten `pre_start` (`Works::pre_start` envía por un canal; `recv_timeout(10 s)` sólo evita un cuelgue, como el `wait_until` existente) antes de ocupar el hilo. Las aserciones no cambian.

RED: la serie de 50/50 con el retraso diagnóstico y la traza. GREEN: 0/50 con el retraso y 0/1000 por versión.
