# Paridad de los panics de un `MessageQueue` del usuario

`MBX-TQ-01..03` (`oracle/src/main/java/pilot/MailboxOracle.java`), grabados como `mb-0036..mb-0038` con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, nunca Rust) y presentes en `cases/oracle-only/mailbox.golden`.

`MBX-TQ-02/03` observan el informe de error no capturado, que pasa por un `LoggingAdapter`: la configuración del caso usa `loglevel = ERROR`, `use-slf4j = off` y ningún logger. Normalización: el número del hilo del pool se imprime como `N`.

Estabilidad: `check_mailbox_oracle.py --runs 30` sobre la versión sin dividir (#448, 85 casos): `MBX-TQ-01..03` idénticos al golden en las 30 corridas. Las grabaciones son las mismas; tras integrar `8317bdf3` se reconstruyó el jar y `mb-0036..0038` salieron idénticos byte a byte (`cmp`, una corrida cada uno).

`MBX-TQ-06` (`mb-0041`): el orden que fija el test de #452 (informe no capturado y después el envío tragado del suscriptor roto), grabado de 2.0.0; `check_mailbox_oracle.py --runs 30` sobre este árbol (88 casos): `MBX-TQ-01..03` y `06` idénticos al golden en las 30 corridas; salió con código 1 sólo por el caso anterior y ajeno `MBX-RJ-03` (corrida 17).
