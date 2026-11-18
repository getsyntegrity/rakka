| mutant | outcome | passed/total |
|---|---|---|
| utf8-lossy-decoder | DETECTED | 544/546 |
| no-set-terminated-without-children | DETECTED | 542/546 |
| mailbox-is-lifo | DETECTED | 458/546 |
| receiver-never-sees-sender | DETECTED | 457/546 |
| forward-drops-sender | DETECTED | 543/546 |
| throughput-ignored | DETECTED | 536/546 |
| pill-not-suppressed-from-dead-letters | DETECTED | 545/546 |
| drop-uid-in-serialization-format | DETECTED | 531/546 |
| ref-identity-is-path | DETECTED | 526/546 |
| children-match-by-path | DETECTED | 545/546 |
| watch-self-check-by-path | DETECTED | 544/546 |
| unwatch-keeps-queued-terminated | DETECTED | 544/546 |
| unwatch-is-noop | DETECTED | 543/546 |
| watch-dead-confirms-existence | DETECTED | 537/546 |
| kill-restarts-instead-of-stopping | DETECTED | 538/546 |
| deathpact-restarts-instead-of-stopping | DETECTED | 542/546 |
| init-failure-restarts-instead-of-stopping | DETECTED | 544/546 |
| exception-stops-instead-of-restarting | DETECTED | 494/546 |
| post-restart-not-called | DETECTED | 510/546 |
| pre-restart-without-message | DETECTED | 510/546 |
| failure-does-not-suspend-mailbox | DETECTED | 510/546 |
| resume-confused-with-restart | DETECTED | 530/546 (rerun: run-3b3-rerun-2-stale-patches.log) |
| restart-confused-with-resume | DETECTED | 510/546 (rerun: run-3b3-rerun-2-stale-patches.log) |
| restart-limit-allows-one-more | DETECTED | 539/546 |
| restart-limit-allows-one-fewer | DETECTED | 538/546 |
| minus-one-is-not-unlimited | DETECTED | 519/546 |
| restart-counter-shared-by-siblings | DETECTED | 544/546 |
| escalate-resolved-locally | DETECTED | 530/546 |
| escalate-alters-cause | DETECTED | 530/546 |
| escalate-never-resumes-children | DETECTED | 540/546 |
| control (unmodified) | CLEAN | 546/546 |


Merged table: 28 rows from run-3b3-30-mutants-546-cases-run1.log (same tree, corpus 546) + 2 rows re-run after their patches went stale (run1 reported PATCH_ERROR / BUILD_ERROR, exit 2). 30/30 detected, control 546/546.
