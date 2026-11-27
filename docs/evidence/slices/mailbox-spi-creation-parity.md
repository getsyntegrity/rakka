# Paridad de los panics del SPI de mailbox al crear un actor

`MBX-TT-01/02` (`oracle/src/main/java/pilot/MailboxOracle.java`), grabados como `mb-0034`/`mb-0035` con `rust-anything app record` (el recorder ejecuta `oracle/run.sh`, nunca Rust) y presentes en `cases/oracle-only/mailbox.golden`.

Estabilidad: `python3.12 tools/parity/check_mailbox_oracle.py --runs 30` (80 casos): `MBX-TT-01/02` idénticos al golden en las 30 corridas. El comando terminó con código 1 por casos **anteriores y ajenos** a este cambio: `MBX-RJ-02`, `MBX-RJ-03` y `MBX-RJ-05` difirieron en las corridas 15, 20, 24 y 27 (máquina con carga: clon y tests a la vez).
