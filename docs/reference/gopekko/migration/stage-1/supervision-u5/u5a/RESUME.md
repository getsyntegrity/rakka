# U5a resume point (informational, not part of the approval hashes)

State: the **design** (`../RULEBOOK.md`, decisions D1-D12, three deliveries) is approved by the Owner (commit `981a6ea`, PR #462). The **execution package of U5a** (RUNBOOK, manifest, permissions, launcher, implementer prompt) is prepared and waits for the Owner's final approval of its hashes. **Execution is NOT authorized yet**: no implementation agent has run and `pekko-actor/` is unchanged. U5b and U5c have no package and are not authorized.

Budget of this delivery: maximum 560,000 tokens, evaluation stop 450,000, 60 minutes of active work, no Haiku probe. Coordination is charged by rule: a fixed reserve of 90,000 plus 30,000 per extra correction round or repeated gate cycle (RUNBOOK section 3). The proposed aggregate of 1,680,000 for the whole of U5 is **not** authorized.

Hashes (sha256) to approve:
- `../RULEBOOK.md` 5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209
- `settings.template.json` 6f06ebbc499de8a3ab5ff4e0247b2cba72876bbfd4ddd33abbebe4e9aa01496b
- `run-agent.sh` 57b936d4738b43820862595354f3b9e2e876cea24d5200b1ef4e205b1cc57cdb
- `manifest.tsv` 9553023697bcd0cccf701f4ff9f9cca076ee8922b526a09fdd28c4e94443f736
- `implementer-prompt.md` d8dd53517e02af404ba9bb62a8768a19c0d8b1895663b0016822e397a25f7410
- `RUNBOOK.md` de2356971d093334fbb90bed345decd54e61c01adcb706c89008cf2542118add

To run, after the Owner approves: first merge PR #462 into `develop` (the provider takes the merge-base with `develop`), create `feat/actor-supervision-u5a` from it, follow RUNBOOK section 5 and run the command of section 10. Do not re-map or re-plan.

Not done and not claimed: the implementation, U5b, U5c, the mutation campaign, the judge campaign and tag, the repeated complete `-race`, the integral review, the S13/S14/S16 mirror (U8, #175), the Pekko surface inventory, `System.Shutdown`, `whenTerminated` and the ACTINV rows (the inventory is not in the repository). No parity is claimed and CURRENT stays stale.
