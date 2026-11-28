# ActorSystem desde una Config (AC-CFG-003, AC-CFG-020): evidencia RED y GREEN

Comando: `cargo test -p rakka-actor --test classic_system_from_config` (las nueve pruebas son de integración: arrancan un
sistema real o comprueban que no arranca).

## RED

Con un `create_from_config` provisional que ignoraba la `Config` (`Self::create(name)`) y un `config_settings` que
devolvía `None`:

```
test the_name_is_checked_before_the_settings_are_read ... ok
test coordinated_shutdown_cannot_run_on_terminate_without_terminating_the_system ... FAILED
test the_slf4j_config_is_amended_before_the_settings_are_read ... FAILED
test a_system_created_without_a_config_has_no_config_settings ... ok
test the_system_exposes_the_settings_read_from_the_config_over_the_reference ... FAILED
test a_config_of_another_pekko_version_does_not_start_the_system ... FAILED
test the_runtime_settings_and_the_deployments_come_from_the_config ... FAILED
test a_rejected_deployment_path_does_not_start_the_system ... FAILED
test an_error_amending_the_slf4j_config_is_a_settings_error ... FAILED
test result: FAILED. 2 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out
```

Las dos que pasaban caracterizan lo que el `create` anterior ya hacía (el nombre inválido y el sistema sin `Config`).
Ejemplo del motivo del fallo: `a system was created from: pekko.version = "0.0.1"`; en la de los ajustes del runtime,
`left: 10s`, `right: 3s`.

## GREEN

`test result: ok. 9 passed; 0 failed`.

## Caracterización tras integrar la estrategia del guardián

`an_unregistered_guardian_strategy_name_does_not_start_the_system` pasó a la primera (el rechazo ya lo hace
`create_with_settings`): es caracterización, no un RED. `test result: ok. 10 passed; 0 failed`.
