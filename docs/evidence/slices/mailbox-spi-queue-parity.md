# Paridad de los panics de un `MessageQueue` del usuario

`MBX-TQ-01..03` (`oracle/src/main/java/pilot/MailboxOracle.java`), grabados como `mb-0036..mb-0038` con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, nunca Rust) y presentes en `cases/oracle-only/mailbox.golden`.

`MBX-TQ-02/03` observan el informe de error no capturado, que pasa por un `LoggingAdapter`: la configuración del caso usa `loglevel = ERROR`, `use-slf4j = off` y ningún logger. Normalización: el número del hilo del pool (sólo sus dígitos) se imprime como `N`.

Estabilidad: `check_mailbox_oracle.py --runs 30` sobre la versión sin dividir (#448, 85 casos): `MBX-TQ-01..03` idénticos al golden en las 30 corridas. Las grabaciones son las mismas; tras integrar `8317bdf3` se reconstruyó el jar y `mb-0036..0038` salieron idénticos byte a byte (`cmp`, una corrida cada uno).

`MBX-TQ-06` (`mb-0041`): un suscriptor de `Error` cuya cola lanza al recibir el informe no capturado; lo contractual es que el oyente sano recibe los dos errores (el informe y el envío tragado del suscriptor roto). El orden entre ambos no está garantizado (AC-EVT-001; el observado en 2.0.0 sale de la iteración de un `Set` pequeño, accidental según Q-X-21): el caso imprime `errors` ordenado en los dos gemelos y el test de `classic_queue_spi_failure.rs` no depende del orden. Tras ese cambio y con #453 integrado: `check_mailbox_oracle.py --runs 30` (jar reconstruido, 88 casos) terminó con código 0, las 30 corridas iguales al golden. `mb-0041` se volvió a grabar: sólo cambió el título; los dos errores quedaron byte a byte como antes.
