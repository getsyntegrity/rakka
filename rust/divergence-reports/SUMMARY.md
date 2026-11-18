| mutant | outcome | passed/total |
|---|---|---|
| utf8-lossy-decoder | DETECTED | 1035/1044 |
| no-set-terminated-without-children | DETECTED | 1040/1044 |
| mailbox-is-lifo | DETECTED | 948/1044 |
| receiver-never-sees-sender | DETECTED | 948/1044 |
| forward-drops-sender | DETECTED | 1041/1044 |
| throughput-ignored | DETECTED | 1034/1044 |
| pill-not-suppressed-from-dead-letters | DETECTED | 1043/1044 |
| drop-uid-in-serialization-format | DETECTED | 1002/1044 |
| ref-identity-is-path | DETECTED | 1024/1044 |
| children-match-by-path | DETECTED | 1043/1044 |
| watch-self-check-by-path | DETECTED | 1042/1044 |
| unwatch-keeps-queued-terminated | DETECTED | 1042/1044 |
| unwatch-is-noop | DETECTED | 1041/1044 |
| watch-dead-confirms-existence | DETECTED | 1035/1044 |
| kill-restarts-instead-of-stopping | DETECTED | 1036/1044 |
| deathpact-restarts-instead-of-stopping | DETECTED | 1040/1044 |
| init-failure-restarts-instead-of-stopping | DETECTED | 1042/1044 |
| exception-stops-instead-of-restarting | DETECTED | 976/1044 |
| post-restart-not-called | DETECTED | 997/1044 |
| pre-restart-without-message | DETECTED | 997/1044 |
| failure-does-not-suspend-mailbox | DETECTED | 1005/1044 |
| resume-confused-with-restart | DETECTED | 1022/1044 |
| restart-confused-with-resume | DETECTED | 997/1044 |
| restart-limit-allows-one-more | DETECTED | 1034/1044 |
| restart-limit-allows-one-fewer | DETECTED | 1033/1044 |
| minus-one-is-not-unlimited | DETECTED | 1009/1044 |
| restart-counter-shared-by-siblings | DETECTED | 1039/1044 |
| escalate-resolved-locally | DETECTED | 1028/1044 |
| escalate-alters-cause | DETECTED | 1028/1044 |
| escalate-never-resumes-children | DETECTED | 1042/1044 |
| failure-does-not-suspend-children | DETECTED | 1038/1044 |
| suspension-not-recursive | DETECTED | 1042/1044 |
| all-for-one-degrades-to-one-for-one | DETECTED | 1032/1044 |
| all-for-one-stop-only-failed | DETECTED | 1038/1044 |
| lossy-stdin-decoder | DETECTED | 1044/1044 |
| ipv4-octet-overflow-swallowed | DETECTED | 1042/1044 |
| control (unmodified) | CLEAN | 1044/1044 |

outcomes: {'DETECTED': 36, 'UNDETECTED': 0, 'PATCH_ERROR': 0, 'BUILD_ERROR': 0, 'HARNESS_ERROR': 0, 'TIMEOUT': 0}
