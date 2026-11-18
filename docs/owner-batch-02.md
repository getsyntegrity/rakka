# Paquete de decisiones para el Owner — lote 02 (Ola 0)

Nada de esto está aceptado: cada recomendación sigue `proposed` en `docs/decisions/register.tsv` hasta que el Owner responda.
Cuando responda se registra en `docs/decisions/owner-decisions.tsv`, se regenera el paquete y recién entonces se cierran las filas afectadas (hoy quedan `partial`).

### P06 UTIL-PLATAFORMA

| Decisión | Recomendada | Consecuencia observable | Evidencia contra Pekko 2.0.0 (⚠ si cambió) | Riesgo si resulta mal |
|---|---|---|---|---|
| Q-UTIL-18 — Zona horaria del sistema para Helpers.timestamp | (a) dependencia con base de zonas (candidatas `jiff`, o `chrono` + `iana-time-zone`; sujeta a `cargo deny`) | Sólo el texto de la hora en la salida estándar del logger (`MM/dd/yyyy HH:mm:ss.SSS`): con (b) (desplazamiento fijo, UTC por defecto) no coincide con Pekko fuera de UTC. El resto de `timestamp` ya coincide para un desplazamiento fijo. | Helpers.scala:108-113 (`ZoneId.systemDefault()` capturada al cargar), Logging.scala:1009; casos TS-LOCAL-01/02 (uc-0103, uc-0104) con la zona del oráculo fijada en GMT+05:30. Sin cambio funcional respecto de 1.7.1. | bajo: sólo texto de logs; (a) añade la primera dependencia externa del crate. |
| Q-UTIL-19 — fastHash y copyUSAsciiStrToBytes con texto no ASCII dependen del algoritmo de la JVM | (c) aceptar sólo texto ASCII (`fast_hash` devuelve `None` para el resto) | `fast_hash("é")` no devuelve un valor; con (a)/(b) devolvería el de la JVM sin opciones o el de la JVM con `--add-opens`, que discrepan entre sí. No hay consumidores en producción. | Unsafe.java:73-146; UN-03 medido con JDK 17 sin `--add-opens` (sin grabar): `é` y `€` -> 1727796305, `aéb` -> 967922898, U+1F600 -> ArrayIndexOutOfBoundsException; sin consumidores en actor/src/main ni actor-typed/src/main. Sin cambio respecto de 1.7.1. | bajo: la función no tiene usos; cambiarla después no rompe nada observable. |
