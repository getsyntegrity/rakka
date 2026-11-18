| mutant | outcome | passed/total |
|---|---|---|
| utf8-lossy-decoder | DETECTED | 564/566 |
| no-set-terminated-without-children | DETECTED | 562/566 |
| mailbox-is-lifo | DETECTED | 467/566 |
| receiver-never-sees-sender | DETECTED | 470/566 |
| forward-drops-sender | DETECTED | 563/566 |
| throughput-ignored | DETECTED | 556/566 |
| pill-not-suppressed-from-dead-letters | DETECTED | 565/566 |
| drop-uid-in-serialization-format | DETECTED | 551/566 |
| ref-identity-is-path | DETECTED | 546/566 |
| children-match-by-path | DETECTED | 565/566 |
| watch-self-check-by-path | DETECTED | 564/566 |
| unwatch-keeps-queued-terminated | DETECTED | 564/566 |
| unwatch-is-noop | DETECTED | 563/566 |
| watch-dead-confirms-existence | DETECTED | 557/566 |
| kill-restarts-instead-of-stopping | DETECTED | 558/566 |
| deathpact-restarts-instead-of-stopping | DETECTED | 562/566 |
| init-failure-restarts-instead-of-stopping | DETECTED | 564/566 |
| exception-stops-instead-of-restarting | DETECTED | 498/566 |
| post-restart-not-called | DETECTED | 519/566 |
| pre-restart-without-message | DETECTED | 519/566 |
| failure-does-not-suspend-mailbox | DETECTED | 527/566 |
| resume-confused-with-restart | DETECTED | 544/566 |
| restart-confused-with-resume | DETECTED | 519/566 |
| restart-limit-allows-one-more | DETECTED | 556/566 |
| restart-limit-allows-one-fewer | DETECTED | 555/566 |
| minus-one-is-not-unlimited | DETECTED | 531/566 |
| restart-counter-shared-by-siblings | DETECTED | 561/566 |
| escalate-resolved-locally | DETECTED | 550/566 |
| escalate-alters-cause | DETECTED | 550/566 |
| escalate-never-resumes-children | DETECTED | 564/566 |
| failure-does-not-suspend-children | DETECTED | 560/566 |
| suspension-not-recursive | DETECTED | 564/566 |
| all-for-one-degrades-to-one-for-one | DETECTED | 554/566 |
| all-for-one-stop-only-failed | DETECTED | 560/566 |
| control (unmodified) | CLEAN | 566/566 |

outcomes: {'DETECTED': 34, 'UNDETECTED': 0, 'PATCH_ERROR': 0, 'BUILD_ERROR': 0, 'HARNESS_ERROR': 0, 'TIMEOUT': 0}
