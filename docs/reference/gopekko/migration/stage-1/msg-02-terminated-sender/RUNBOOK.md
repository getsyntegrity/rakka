# RUNBOOK: Sender of `Terminated` and `WatchWith` (#814 Part 2, unit msg-02-terminated-sender)

Status: **prepared; blocked on the Owner's approval of the RULEBOOK sha256.** No model or cost plan is claimed. Pekko pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

## 1. Writable files

Exactly the table in RULEBOOK section 2: `pekko-actor/actor/watch.go`, `pekko-actor/actor/sender_terminated_test.go`, `pekko-actor/README.md`, the B1 wording in `docs/adr/0032-local-sender-envelope.md`, then `EXECUTION-RECORD.md` and one `migration/cost-log.tsv` row. Nothing else.

## 2. Procedure

1. Check that the base contains ADR 0032 "D5a and D5b" (Part 1 merged); otherwise stop and report.
2. Observe RED: T1, T2, T3, T5, T6 fail on the base (sender absent); T4, T7 and T10 pass there (they pin existing behavior).
3. Implement B1 per RULEBOOK section 1; keep `deliverRuntime` at gocyclo 10 or less.
4. Run the mutation checks of RULEBOOK section 3 and the differential of existing watch/supervision tests against a base copy in a `mktemp -d` directory (stdout, stderr, exit code). Remove the scratch afterwards.
5. `go doc -all` before/after on `pekko-actor/actor`: no difference.
6. Gates of RULEBOOK section 6.
7. Optional, only with Owner authorization: extend `Probe.scala` for T3, T4, T5, T7 (RULEBOOK section 4) and record the observed lines.

## 3. Not recorded here

Agent counts, tokens and costs are not measured. The execution record holds the evidence bound to a SHA.
