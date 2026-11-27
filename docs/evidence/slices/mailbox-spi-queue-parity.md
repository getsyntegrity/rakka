# Paridad de los panics de un `MessageQueue` del usuario

`MBX-TQ-01..05` (`oracle/src/main/java/pilot/MailboxOracle.java`), grabados como `mb-0036..mb-0040` con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, nunca Rust) y presentes en `cases/oracle-only/mailbox.golden`.

`MBX-TQ-02/03` observan el informe de error no capturado, que pasa por un `LoggingAdapter`: la configuración del caso usa `loglevel = ERROR`, `use-slf4j = off` y ningún logger. Normalización: el número del hilo del pool se imprime como `N`.

Estabilidad: `python3.12 tools/parity/check_mailbox_oracle.py --runs 30` (ver la descripción del PR).
