# Reporte profundo: cierre de 3b-2 y slices 3b-3, 3b-4 y 3b-5

Workspace `pekko-rust-pilot` · crates `pekko-actor` y `pekko-actor-typed` · referencia: Apache Pekko 2.0.0 en JDK 17 · sin commit, push ni PR.

## 0. Resumen ejecutivo

| Verificación | Resultado final |
|---|---|
| Replay contra grabaciones de Pekko | **566/566** (529 previos + 37 nuevos) |
| Regresión de mutantes | **34/34 detectados**, control limpio 566/566, 0 errores de parche/compilación/arnés/timeout |
| `cargo test --workspace` | **88 tests, 0 fallos** |
| Clippy `-D warnings` | limpio |
| Cobertura de producción de `pekko-actor` | **96,21 %** (gate ≥ 95 %: pasa) |
| Fuzzers (4 × 2 seeds) | **0 divergencias, 0 timeouts del oráculo** |

Lo que se entregó, en orden:

1. **3b-2 cerrado** con evidencia: 27/27 mutantes sobre 529 casos, lanzador endurecido y probado.
2. **3b-3 Escalate**: 17 escenarios, 11 tests, 3 mutantes.
3. **3b-4 Suspensión del subárbol** ante fallo propio: 6 escenarios, 5 tests, 2 mutantes. Rust divergía de Pekko y se corrigió.
4. **3b-5 AllForOneStrategy**: 14 escenarios, 6 tests, 2 mutantes.

Queda fuera, como se pidió: escalada hasta guardianes, `withinTimeRange`, ask y timers.

## 1. Cierre de 3b-2

### 1.1 Estado encontrado
Había un job activo con 13 mutantes más su control, y un job anterior de 14 mutantes (el 15.º se cortó por un `AssertionError`: un parche viejo ya no coincidía).
Verifiqué que los fuentes de producción no cambiaron entre ambas tandas (última modificación 09:14:46, anterior al primer job de las 09:15) y que el corpus era el de 529 casos.

### 1.2 Cifras que no cerraban
El resumen anterior hablaba de 509–528 casos aprobados para los primeros 14 y, a la vez, de 445/529 (sender) y 451/529 (mailbox LIFO).
Las cifras de 445 y 451 eran reales; el rango 509–528 solo describía a los demás. En la corrida final única: sender 445/529, LIFO 450/529 (varía por scheduler, es un mutante sensible a hilos).

### 1.3 Lanzador (`tools/check_divergence.py`)
Problema: contaba **cualquier** salida distinta de cero como «detectado», aunque fuese un fallo del arnés, y no tenía timeout.

Cambios:
- Clasifica cada mutante: `DETECTED`, `UNDETECTED`, `PATCH_ERROR`, `BUILD_ERROR`, `HARNESS_ERROR`, `TIMEOUT`.
- «Detectado» exige que el replay haya impreso `N/M cases passed`, con N < M y exit ≠ 0.
- Timeouts de 900 s (build y replay), `SUMMARY.md` generado, comprobación de que todos los mutantes ven el mismo tamaño de corpus.
- Códigos de salida: **0** todo detectado y control limpio; **1** mutante no detectado o control sucio; **2** experimento inválido.
- `--selftest`: corre un mutante equivalente (solo un comentario) que **no puede** detectarse. Resultado: `UNDETECTED`, exit **1** (`run-launcher-selftest.log`). Así queda probado que el lanzador propaga el fallo.

### 1.4 Resultado
**27/27 DETECTED, control 529/529**, en una única corrida con la versión final del lanzador (`run-3b2-27-mutants-529-cases.log`, `SUMMARY-3b2-27.md`).

## 2. Slice 3b-3: Escalate

### 2.1 Oráculo
Arbol de tres niveles: `g` (abuelo supervisor) → `p` (padre supervisor que escala) → `c` (hijo que falla en `receive`).
Se agregó al oráculo Java y al arnés Rust:
- la directiva `escalate` en el decider;
- `supervisor.log-decisions: true`: registra `{"ev":"decide","cause","directive"}` cada vez que corre un decider. Está desactivado por defecto, así las 529 grabaciones previas no cambian.

17 escenarios (`tools/make_scenarios_3b3.py`), todos deterministas en 12 corridas, nombres únicos en 24 caracteres. Se coordinan con barreras de eventos (`await-events`) y ida y vuelta de comandos; sin sleeps.

### 2.2 Qué hace Pekko (todo grabado)
- El decider del abuelo recibe **la misma excepción** que lanzó el hijo (clase y mensaje intactos). El decider del padre corre primero.
- La directiva se aplica al **padre**, no al hijo.
- El padre se suspende a sí mismo y a sus otros hijos mientras el abuelo decide; el hijo fallido no se suspende dos veces.
- **Resume** del padre: sin hooks, misma instancia y referencias; el hijo continúa (el mensaje fallido se pierde, los encolados se procesan); un hermano sano también se reanuda sin perder nada.
- **Restart** del padre: `preRestart(causa del hijo, mensaje = sysmsg.Failed)`, `postStop`, nueva instancia, `postRestart`, `preStart`. Los hijos se detienen por el `preRestart` por defecto y no se recrean; los mensajes encolados en el hijo son dead letters; la referencia al padre sigue válida y no hay `Terminated`.
- **Stop** del padre: se detienen padre e hijos; el abuelo recibe `Terminated`.
- El `maxNrOfRetries` del abuelo cuenta los reinicios del padre (0: la primera escalada lo detiene; 1: la segunda).
- Lo que el padre resuelve localmente nunca llega al abuelo; el límite propio del padre es irrelevante para una escalada.

### 2.3 Rojo → verde
- Rojo: replay **529/546** (los 17 casos nuevos divergían); el primer test no compilaba (`no variant Escalate`).
- Verde: implementación mínima y replay **546/546** de una vez.
- Como los otros 16 escenarios quedaron cubiertos de golpe por el mismo cambio, se agregaron sus tests de Rust uno por uno (11 en total en `tests/classic_escalate.rs`).

### 2.4 Implementación
`Directive::Escalate`; `escalate` en `actor/cell.rs` junto al resto del manejo de directivas; mensaje de sistema `Suspend`; `Resume.cause` pasa a `Option`; clase pública `Failed` (lo que recibe `preRestart`).

### 2.5 Mutantes
| Mutante | Qué rompe | Casos |
|---|---|---|
| `escalate-resolved-locally` | el padre resuelve localmente lo que debía escalar | 550/566 |
| `escalate-alters-cause` | la escalada reenvuelve la causa. **Sí es observable**: lo muestra el evento `decide` del abuelo | 550/566 |
| `escalate-never-resumes-children` | tras Resume, el hijo fallido nunca se reanuda | 564/566 |

## 3. Slice 3b-4: suspensión del subárbol ante fallo propio

### 3.1 Oráculo
`g` supervisa a `p`; `p` falla en `receive` y tiene un hijo activo `c` (también: dos hijos, un nieto). El decider de `g` registra su decisión y **espera una compuerta** (`supervisor.hold`) que el arnés libera con `open-hold`: una barrera, no un sleep.
Durante la pausa se envían mensajes a los hijos y `await-quiet` comprueba, con una espera acotada de 300 ms (el único paso con tiempo, necesario para probar una ausencia), que no se procesaron. La recuperación se espera solo por eventos.

6 escenarios, deterministas en 10 corridas.

### 3.2 Qué hace Pekko
**Todos los hijos del actor fallido están suspendidos, recursivamente (nietos incluidos), hasta que su supervisor decide.**
- Resume: se procesa en orden lo que llegó durante la pausa.
- Restart: los hijos se detienen y esos mensajes son dead letters, nunca procesados.
- Stop: se detiene todo el subárbol.

### 3.3 Rojo → verde
Rust **divergía**.
- Rojo: replay **546/552** (los 6 nuevos) y el test vio que `c` procesó `x1`, `x2` durante la pausa: `["pre-start","m0","x1","x2"]`.
- Corrección en la ruta de fallo propio (`ActorCell::fail`, compartida con Escalate): suspender mailbox y todos los hijos; `Suspend` recursivo (`fault_suspend`); `fault_resume` reanuda a los hijos actuales (al escalado con la causa, a los demás sin ella).
- Verde: replay **552/552**, 5 tests en `tests/classic_suspension.rs`.

### 3.4 Mutantes
`failure-does-not-suspend-children` (560/566) y `suspension-not-recursive` (564/566).

### 3.5 Límite conocido
Pekko reinicia a los hijos que **sobreviven** a un `preRestart` que no los detiene. No está portado: el `preRestart` por defecto sí los detiene y ningún escenario grabado lo distingue.

## 4. Slice 3b-5: AllForOneStrategy

### 4.1 Oráculo
Supervisor `p` con `strategy: "all-for-one"`, dos hijos (`c1` falla, `c2` sano). 14 escenarios (9 de directivas `o*`, 5 de límite `r*`), deterministas en 20–40 corridas. El orden de dos `Terminated` es una carrera de Pekko, así que esos pasos comparan eventos ordenados (`await-events {"sorted": true}` / `events-sorted`).

### 4.2 Qué hace Pekko
| Caso | Comportamiento |
|---|---|
| Resume | solo el hijo que falló; el sano no ve nada |
| Restart | **ambos** reinician; estado desde cero en los dos; referencias y uids conservados, sin `Terminated`. El hijo sano ve `preRestart(causa del otro, None)`; el fallido, la causa y el mensaje que falló. Los mensajes encolados detrás del fallo van a su nueva instancia |
| Stop | **ambos** se detienen; mensajes posteriores son dead letters |
| Decisión retenida | el hermano sano **no** está suspendido todavía y sigue procesando; se suspende y reinicia al aplicarse la decisión |
| Límite | se comprueba con `forall` sobre todos los hijos: `max=0` detiene ambos al primer fallo; `max=1` reinicia ambos y, si falla el *otro* hijo, agota el límite y detiene ambos; Resume no cuenta; negativo es ilimitado |

### 4.3 Rojo → verde en pasos
- Rojo: replay **554/566** (12 de 14 divergían; `o01` y `o09`, Resume solo sobre el fallido, ya coincidían: **caracterización**) y 4 de 6 tests de Rust fallaban.
- Paso B: Restart sobre todos los hijos → 3/6 en verde, Stop y límite seguían rojos.
- Paso C: Stop y límite agotado sobre todos → 6/6.
- Replay final **566/566**.

### 4.4 API e implementación
`supervision::{AllForOneStrategy, SupervisorStrategy}`. `Actor::supervisor_strategy()` ahora devuelve `SupervisorStrategy`; `OneForOneStrategy` y `AllForOneStrategy` se convierten con `.into()`. Lógica en `ActorCell::process_failure` (equivalente a `processFailure` de Pekko), en el módulo de celda junto a las otras directivas; los hijos sanos se suspenden antes de su `Recreate`.

### 4.5 Mutantes
`all-for-one-degrades-to-one-for-one` (degrada AllForOne a OneForOne, pedido explícito): 554/566. `all-for-one-stop-only-failed`: 560/566.

## 5. Problemas encontrados en el camino

| # | Hallazgo | Resolución |
|---|---|---|
| 1 | Lanzador de mutantes contaba errores de arnés como «detectado» | Clasificación de resultados y códigos de salida (§1.3) |
| 2 | Cinco mutantes viejos y `escalate-alters-cause` quedaron con patrón obsoleto tras refactorizar. Primero apareció como `PATCH_ERROR`/`BUILD_ERROR` (exit 2), sin ser un veredicto | Patrones re-apuntados y verificados con un pase `count == 1` antes de la corrida larga |
| 3 | Flake `o06`: dos `Terminated` en orden de carrera. Mi chequeo de 20 corridas tuvo suerte | Eventos ordenados; 40 corridas idénticas; re-grabado |
| 4 | Flake `r05`: dos hijos fallando a la vez, racy incluso en Pekko | Secuenciado con esperas por eventos; re-grabado |
| 5 | El replay externo no detectó esos flakes; los detectó el test en proceso (`recorded_pekko`) | 25 corridas seguidas sin fallos tras el arreglo |
| 6 | Clippy: tipos complejos en el arnés | Alias `Hold` y estructura `SupervisorCfg` |
| 7 | Primera tanda de fuzzers no corrió (zsh no separa `$var` en palabras) | Driver en Python con argumentos explícitos |
| 8 | Errores de conteo de eventos en mis escenarios | Corregidos desde las trazas de Pekko, nunca ajustando la grabación |

## 6. Cobertura de producción (`tools/coverage.sh`, sin exclusiones)

| Archivo | Líneas | Sin cubrir | Cobertura |
|---|---|---|---|
| `actor/cell.rs` | 496 | 22 | 95,56 % |
| `actor/children.rs` | 165 | 4 | 97,58 % |
| `actor/fault.rs` | 13 | 1 | 92,31 % |
| `actor/props.rs` | 54 | 0 | 100 % |
| `actor/supervision.rs` | 79 | 3 | 96,20 % |
| `actor/system.rs` | 269 | 23 | 91,45 % |
| `dispatch/dispatcher.rs` | 31 | 1 | 96,77 % |
| `error.rs` | 12 | 0 | 100 % |
| `event/mod.rs` | 30 | 1 | 96,67 % |
| `identity/actor_path.rs` | 166 | 0 | 100 % |
| `identity/actor_ref.rs` | 29 | 0 | 100 % |
| `identity/jdk_int.rs` | 104 | 1 | 99,04 % |
| `identity/jdk_uri.rs` | 560 | 23 | 95,89 % |
| `identity/names.rs` | 76 | 0 | 100 % |
| `lib.rs` | 3 | 0 | 100 % |
| **Total** | **2 087** | **79** | **96,21 %** |

`system.rs` (91,45 %) y `fault.rs` (92,31 %) están bajo 95 % individualmente; el gate se aplica al total. `pekko-actor-typed` no tiene implementación aún.

## 7. Tests (88)

| Binario | Tests |
|---|---|
| unitarios `pekko-actor` | 28 |
| `classic_api` | 7 |
| `classic_tell` | 5 |
| `classic_hierarchy` | 5 |
| `classic_failures` | 7 |
| `classic_supervision` | 12 |
| `classic_escalate` | 11 |
| `classic_suspension` | 5 |
| `classic_all_for_one` | 6 |
| `recorded_pekko` (replay en proceso de los 566 casos) | 2 |

## 8. Corpus de replay (566)

| Origen | Casos |
|---|---|
| `path-from-string` | 184 |
| `scenario-2a` | 125 |
| `scenario-2b` | 77 |
| `scenario-3a` | 88 |
| `scenario-3b1` | 27 |
| `scenario-3b2` | 29 |
| `scenario-3b3` (Escalate) | 17 |
| `scenario-3b4` (suspensión) | 6 |
| `scenario-3b5` (AllForOne) | 14 |

Evolución del replay: 529/529 → 529/546 (rojo 3b-3) → 546/546 → 546/552 (rojo 3b-4) → 552/552 → 554/566 (rojo 3b-5) → 566/566.

## 9. Mutantes (34)

| Grupo | Mutantes | Casos aprobados (de 566) |
|---|---|---|
| 1–2b | utf8-lossy-decoder 564 · no-set-terminated-without-children 562 · mailbox-is-lifo 467 · receiver-never-sees-sender 470 · forward-drops-sender 563 · throughput-ignored 556 · pill-not-suppressed-from-dead-letters 565 · drop-uid-in-serialization-format 551 | 467–565 |
| 3a | ref-identity-is-path 546 · children-match-by-path 565 · watch-self-check-by-path 564 · unwatch-keeps-queued-terminated 564 · unwatch-is-noop 563 · watch-dead-confirms-existence 557 | 546–565 |
| 3b-1 | kill-restarts-instead-of-stopping 558 · deathpact-restarts-instead-of-stopping 562 · init-failure-restarts-instead-of-stopping 564 · exception-stops-instead-of-restarting 498 · post-restart-not-called 519 · pre-restart-without-message 519 · failure-does-not-suspend-mailbox 527 | 498–564 |
| 3b-2 | resume-confused-with-restart 544 · restart-confused-with-resume 519 · restart-limit-allows-one-more 556 · restart-limit-allows-one-fewer 555 · minus-one-is-not-unlimited 531 · restart-counter-shared-by-siblings 561 | 519–561 |
| 3b-3 | escalate-resolved-locally 550 · escalate-alters-cause 550 · escalate-never-resumes-children 564 | 550–564 |
| 3b-4 | failure-does-not-suspend-children 560 · suspension-not-recursive 564 | 560–564 |
| 3b-5 | all-for-one-degrades-to-one-for-one 554 · all-for-one-stop-only-failed 560 | 554–560 |

Control sin modificar: 566/566. Nota: `replay-report-terminated-delivered-while-stopping.md` es un resto de un mutante equivalente descartado en 3a; no pertenece a los 34.

## 10. Fuzzers (árbol final, seeds 1 y 2)

| Fuzzer | Seed 1 | Seed 2 | Divergencias |
|---|---|---|---|
| `fuzz_diff` | 20 000 casos | 20 000 casos | 0 |
| `fuzz_scenarios` | 1 500 esc., 19 009 pasos | 1 500 esc., 18 619 pasos | 0 |
| `fuzz_tell` | 1 000 esc., 36 712 pasos | 1 000 esc., 37 855 pasos | 0 |
| `fuzz_hier` | 1 000 esc., 29 445 pasos | 1 000 esc., 29 584 pasos | 0 |

Alcance: estos fuzzers no generan supervisión. Son regresión de los slices 1–3a; la supervisión (3b) la cubren los escenarios grabados y los mutantes. Un fuzzer diferencial de supervisión **no existe aún** y sería un buen siguiente paso.

## 11. Evidencia guardada

- `rust/divergence-reports/SUMMARY.md` (34 mutantes), `SUMMARY-3b2-27.md`, `run-*.log` (corridas completas), `run-launcher-selftest.log`, un `replay-report-<mutante>.md` por mutante.
- `rust/divergence-reports/tdd/`: salidas del **rojo y del verde** de cada slice (replay y tests de Rust), cobertura final, replay final.
- `rust/app-cases/`: las 566 grabaciones de Pekko; `cases/*.jsonl`: entradas; `tools/make_scenarios_3b3|3b4|3b5.py`: generadores.
- `INVENTORY.md`: notas de cada slice, hallazgos del oráculo y verificación final.

## 12. Cambios en el arnés (oráculo Java + CLI Rust)

`supervisor.decider` acepta `escalate`; `supervisor.strategy` (`one-for-one`/`all-for-one`); `supervisor.log-decisions`; `supervisor.hold` + op `open-hold`; op `await-quiet`; `await-events` con `sorted`. Todo opcional y desactivado por defecto: las grabaciones anteriores no se tocaron.

## 13. Pendiente y límites

- Fuera de este tramo, como se pidió: escalada hasta guardianes, `withinTimeRange`, ask, timers.
- No portado: reinicio de hijos que sobreviven a un `preRestart` que no los detiene.
- Pendiente sugerido: fuzzer diferencial de supervisión; subir cobertura de `system.rs` y `fault.rs` por encima de 95 % individual.
- Sin commit, push ni PR (el directorio no es un repositorio git).
