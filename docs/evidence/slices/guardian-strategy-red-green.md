# guardian-supervisor-strategy: evidencia RED y GREEN

Comando de cada paso: `cargo test -q -p rakka-actor --test classic_guardian_strategy`.

1. Estrategia stopping leída de la configuración. RED:
   `a_stopping_guardian_strategy_stops_the_failed_top_level_actor ... FAILED`,
   `left: Err(Timeout)`, `right: Ok((Actor[pekko://t-guardian-stopping/user/$a], true, false))`.
   GREEN (con una comparación provisional del nombre en `create_with`): `1 passed; 0 failed`.
2. Nombre sin registrar. RED:
   `an_unregistered_guardian_strategy_name_fails_the_creation_of_the_system ... FAILED`, `the system is not created`.
3. Configurador registrado que escala. RED (compilación):
   `error[E0609]: no field 'supervisor_strategies' on type 'rakka_actor::actor::system::Settings'`.
   GREEN de 2 y 3 con `SupervisorStrategies`: `3 passed; 0 failed`.

Caracterización (pasaron a la primera, no son RED): la prueba de los dos configuradores y las aserciones del `Error`
publicado añadidas en la revisión. También la prueba de `/system` y `guardian_failure_tests` con MBX-GRD-05.
