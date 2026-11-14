# U3 resume point (informational, not part of the approval hashes)

State: package prepared and natively reviewed once (`f09c5a1`, high risk, approved); fixes and the 2026-10-15 policy in later commits, not reviewed. **Execution is NOT authorized** (weekly quota). The 560,000-token figure is a proposal, not an authorization.

Hashes (sha256, see RUNBOOK section 11):
- `RULEBOOK.md` c2f0a3f4498bd3f1b5fb2f67686b2b7216645313df8db56c123aaeb84613b0aa
- `settings.template.json` 1670e0c099fb07cf67dff1d6fe81f3d6feaf8283a9692b049698424ad52349c2
- `run-agent.sh` f7cec4a87ef684f5a5fb1cf5f907d13472b7a4a2d4a17f7c70a0eaf955587b3a
- `manifest.tsv` d588a8c3de3e783a796ae84a904f0e1aba62c1553d1dd09c12f3bb612a3ebe7e
- `RUNBOOK.md` 4071db587f2ea9e70670c94c890e18126ae3ab6482dc0848fa01cc080b620144

To resume: do not re-map or re-plan. Owner confirms quota and a cap; write the hashes outside the repository; start at RUNBOOK section 5 step 1 from branch `docs/u3-package`. One implementer session (Sonnet, production and tests), one Opus review (Goexit and routing), normal tests, one targeted `-race` over Goexit, 60 minutes of active work in total. No campaign, no tag, no merge without the Owner's word. No Haiku probe: scope is checked afterwards, isolation is not proven.

Also pending (module close): mutants M1-M15, judge campaign and tag, complete `-race`, S13/S14/S16 mirror (U8, #175), Pekko surface inventory; U2 carried gaps #173, K27, K28, K33, the n3 exception.
