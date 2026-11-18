| mutant | outcome | passed/total |
|---|---|---|
| utf8-lossy-decoder | DETECTED | 527/529 |
| no-set-terminated-without-children | DETECTED | 525/529 |
| mailbox-is-lifo | DETECTED | 450/529 |
| receiver-never-sees-sender | DETECTED | 445/529 |
| forward-drops-sender | DETECTED | 526/529 |
| throughput-ignored | DETECTED | 519/529 |
| pill-not-suppressed-from-dead-letters | DETECTED | 528/529 |
| drop-uid-in-serialization-format | DETECTED | 514/529 |
| ref-identity-is-path | DETECTED | 509/529 |
| children-match-by-path | DETECTED | 528/529 |
| watch-self-check-by-path | DETECTED | 527/529 |
| unwatch-keeps-queued-terminated | DETECTED | 527/529 |
| unwatch-is-noop | DETECTED | 526/529 |
| watch-dead-confirms-existence | DETECTED | 520/529 |
| kill-restarts-instead-of-stopping | DETECTED | 522/529 |
| deathpact-restarts-instead-of-stopping | DETECTED | 525/529 |
| init-failure-restarts-instead-of-stopping | DETECTED | 527/529 |
| exception-stops-instead-of-restarting | DETECTED | 494/529 |
| post-restart-not-called | DETECTED | 499/529 |
| pre-restart-without-message | DETECTED | 499/529 |
| failure-does-not-suspend-mailbox | DETECTED | 500/529 |
| resume-confused-with-restart | DETECTED | 520/529 |
| restart-confused-with-resume | DETECTED | 499/529 |
| restart-limit-allows-one-more | DETECTED | 523/529 |
| restart-limit-allows-one-fewer | DETECTED | 522/529 |
| minus-one-is-not-unlimited | DETECTED | 507/529 |
| restart-counter-shared-by-siblings | DETECTED | 527/529 |
| control (unmodified) | CLEAN | 529/529 |

outcomes: {'DETECTED': 27, 'UNDETECTED': 0, 'PATCH_ERROR': 0, 'BUILD_ERROR': 0, 'HARNESS_ERROR': 0, 'TIMEOUT': 0}
