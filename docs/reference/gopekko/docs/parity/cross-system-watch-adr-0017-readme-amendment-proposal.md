# Amendment proposal for ADR 0017 and pekko-actor/README.md: watching an actor of another local System

**Status: ACCEPTED AS DESIGN by the Owner on 2026-10-25 (work thread). APPLIED to docs/adr in the PR that carries this commit (#1206); the README change is NOT applied (it belongs to the delivery of DTH-01). NOT implemented.** The ADR 0017 markers are applied in [`docs/adr/0017-supervision.md`](../adr/0017-supervision.md); [`pekko-actor/README.md`](../../pekko-actor/README.md) is not touched and changes the status of no ADR. Nothing in it says that cross-System watch is implemented, executed or at parity with Pekko. The Owner authorized the ADR edit on 2026-10-25; the README edit is authorized only for the delivery of DTH-01.
Provenance: option (b) of GOV-15 ([#1065](https://github.com/getsyntegrity/gopekko/issues/1065)) is recorded in #1065. The Owner decisions of 2026-10-25 were given in the work thread, **not in a GitHub comment or an ADR**; the exact text below was approved as design on 2026-10-25 (see Status). See the [design document](cross-system-watch-design.md), Provenance, and the companion proposals for [ADR 0010](cross-system-watch-adr-0010-amendment-proposal.md), [ADR 0015](cross-system-watch-adr-0015-amendment-proposal.md) and [ADR 0024](cross-system-watch-adr-0024-amendment-proposal.md). The Ask, Done and Stop contracts stay out of this change.
**Blocked:** DTH-01 ([#785](https://github.com/getsyntegrity/gopekko/issues/785)) and DTH-02 ([#827](https://github.com/getsyntegrity/gopekko/issues/827)) until the documents are merged by the Owner and their own readiness is checked ([#1200](https://github.com/getsyntegrity/gopekko/issues/1200), [story](cross-system-watch-ownership-story.md), "ADR impact list (ADR 0033 D5)", items I5-I10, and the amendments are approved as design).

## Status of ADR 0017 and what the Owner accepted (`docs/adr/0017-supervision.md` lines 3-15)

Status line: "accepted on 2026-10-14 by the owner, at commit `d6c74c2fb442ad68d075e90bf6719c630d1a1a6e`: the four hashes below and the A-list (A1-A13, including A9a, A9b and A9c) as written there." Lines 4-5: "**it does not show that any behavior is implemented or equivalent**"; "The pending capabilities and divergences (#443 and #450-#452) stay inside the migration target and are not excluded from the final surface." The hashes are those of the content at `d6c74c2`; the file already differs from that content "by design" (lines 13-15), and any edit adds a further difference. So: **the Owner accepted on 2026-10-14 the whole file as hashed at `d6c74c2` plus the A-list, so all of lines 69, 493, 501-509, 601-602 and 620 are inside accepted content; A9c is additionally named in the A-list. The line numbers in this proposal are those of the current file; at `d6c74c2` the same text is at lines 61, 487, 495, 595 and 615.** The statements are marked: "**[Run]**" for the Pekko behavior (`H5Probe`, a run of Pekko v1.7.1, unchanged by this proposal) and "**Not approved**" for H5b.

## How the proposal records the decision (the choice and why)

Every row keeps its original text, character for character. The proposal **adds** one marker sentence and erases nothing. The five variants share the stem "Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then", so a grep for the stem finds all five places. The row variant (I5, I6, I9) is:

> Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then this row describes the current state.

The paragraph variant (I7) replaces "this row" with "this paragraph" and the clause variant (I8) is a literal sentence, both given in their sections.

Why this wording: (1) "once DTH-01 is implemented" avoids claiming that anything is implemented now; (2) "superseded ... by the ADR 0010 amendment" names the one place where the contract lives; (3) "until then this row describes the current state" keeps the rows true today, because goPekko still rejects the foreign handle (the divergence of #443 is closed by the implementation, not by this PR); (4) "for `Watch`, `WatchWith` and `Unwatch`" keeps `Done`, `Ask` and `Stop` out; (5) the history of the divergence and its issue number remain readable. The marker is written only after the Owner approves it, and the sentence is rewritten (for example "Superseded ... by ADR 0010; implemented in ...") only in the implementation story.

## I5. H5b, line 69 (original text, whole table row)

> | H5b | **Not approved**: `ErrInvalidHandle` is **not** turned into an ordinary supervised failure as a substitute for compatibility between ActorSystems. The rejection of refs of another System is a **divergence pending correction**, with an issue and tests to be derived; it is separate from `nil` and from Go handles without an equivalent; it is **not parity** | S12, #443 |

Proposed: the same row with the row-variant marker sentence inserted at the end of the second cell, after `**not parity**` and a period, before ` | S12, #443 |`. Necessary because: today the row is true; under option (b) the "divergence pending correction" is corrected by DTH-01 by accepting the handle, which is the correction the row foresees. It does **not** contradict the Owner's H5b decision: `ErrInvalidHandle` is still not turned into a supervised failure, and `nil` and non-actor handles are untouched.

## I6. Case 3 row, line 493 (original text, whole table row)

> | `ErrInvalidHandle` — case 3: a handle of **another System** | **none**: a reference of another ActorSystem of the same JVM is *not* rejected; `watch` and `watchWith` work and `Terminated` arrives **[Run]**; no upstream test observes it | **not classified** | **Divergence pending correction (#443)**. **Interim unchanged** (as case 2). Not parity, and **not** an ordinary supervised failure (owner, H5b) |

Proposed: the same row with the row-variant marker sentence appended at the end of the last cell, after "(owner, H5b)" and a period. The Pekko column and its **[Run]** mark are unchanged: they describe Pekko, not goPekko. Under option (b) goPekko is intended to accept the handle for the three calls, as Pekko does for `watch` and `watchWith` (**[Run]**); `Unwatch` is not **[Run]** (probe `H5bCrossWatch`, not executed). The row is a description of the divergence, not an Owner-accepted rule, so it gains the marker and is not rewritten.

## I7. Paragraph "H5b: the three `ErrInvalidHandle` cases are kept apart", lines 501-509 (original text, whole paragraph)

> **H5b: the three `ErrInvalidHandle` cases are kept apart.** The owner did **not** approve turning `ErrInvalidHandle` into an ordinary
> supervised failure as a substitute for compatibility between ActorSystems. Therefore: case 1 (`nil`) is supervised because Pekko has the
> equivalent and treats it as an `Exception`; cases 2 and 3 keep **today's approved Go behavior** (the watcher stops with `ErrInvalidHandle`
> and the decider is not consulted), which is an *interim exception to supervision recorded as a divergence*, not a classification. Case 3
> is a **divergence pending correction**: Pekko accepts the reference (**[Run]**, `H5Probe`), no upstream test exists for it (the specs that
> create more than one `ActorSystem` do not watch across systems), so its tests will be new: a Pekko-side judge scenario and its Go mirror
> (#443). The restriction is **not marked as parity** anywhere. The approved Watch texts that list the three cases together
> (`0010:170`, `pekko-actor/README.md:41`) are edited by the unit that makes case 1 supervised, and only for case 1; they keep stating cases 2
> and 3 as they are, now labelled as a divergence.

Proposed: the paragraph unchanged, followed by one new paragraph, literal text: "Note (proposed): Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then this paragraph describes the current state. Case 2 and case 1 are not affected." Necessary because "cases 2 and 3 keep today's approved Go behavior" stops being true for case 3 when DTH-01 lands; case 2 (zero ID, non-actor `Handle`) stays rejected. The pointer `pekko-actor/README.md:41` no longer matches the README lines (the watch text is at lines 65-67); that pointer is history and is left as written.

## I8. A9c, lines 601-602 (original text, the two lines; the sentence continues on line 603 and is not changed)

> **A9c — pending, not classified by this ADR:** `ErrInvalidHandle` for a Go handle without a demonstrable equivalent and for a handle of
> another System (interim unchanged: the watcher stops and the decider is not consulted; divergence #443); a `Goexit` in `PostStop`

Proposed: the two lines unchanged, with this literal sentence inserted after "divergence #443);": "For the handle of another System only: Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then this text describes the current state." Necessary because A9c is **Owner-accepted** text, like every cited line (the status line names it in the A-list). The marker adds no new rule and does not reclassify anything: "pending, not classified" stays true, since the Owner did not accept converting the case into a supervised failure. Whether to touch accepted A-list text at all is a pending Owner decision.

## I9. D-1, line 620 (original text, whole table row)

> | D-1 | `Watch`, `WatchWith` and `Unwatch` reject a handle of another System with `ErrInvalidHandle` | accepted; `Terminated` is delivered **[Run]** | rejected; the watcher stops | #443 | unchanged; the decider is not consulted; documented as a divergence |

Proposed: the same row with the row-variant marker sentence appended at the end of the last cell, after "documented as a divergence" and a period. The table title is "Divergences pending correction (not parity)"; the row stays in it until the implementation story moves it. Other hits in the same file, left as written: line 607 (`ErrInvalidHandle` (S12, #443)), line 657 (issue register: "D-1: Watch across Systems", #443, open) and line 730 ("the divergence stays registered until then"); they are history or registry entries and become stale only after DTH-01.

## I10. pekko-actor/README.md lines 65-67 (public module document)

Original text (lines 65-67; the sentence starts on line 64 "A `Terminated` with no" and ends after the `ErrInvalidHandle` clause):

> `OnTerminated` stops the watcher (`ErrDeathPact`); registering again with another message, or mixing `Watch`
> and `WatchWith`, fails it with `ErrWatchConflict` (the parent's decider chooses; the default strategy restarts it, as it restarts an `IllegalStateException` in Pekko) (a repeat is a no-op only for a message comparable with `==`);
> a nil, foreign or non-actor handle fails it with `ErrInvalidHandle`. Each registration has its own generation,

Proposed text, **applied only in the implementation story (DTH-01), after the behavior is implemented and tested, not now**: in line 67, replace "a nil, foreign or non-actor handle fails it with `ErrInvalidHandle`." with "a nil or non-actor handle fails it with `ErrInvalidHandle`; a handle of another System of the same process is accepted for `Watch`, `WatchWith` and `Unwatch` (see ADR 0010)." Nothing else in the lines changes. The README is a public document that must describe implemented, verified behavior (project README contract), so until DTH-01 lands the current text is correct and stays. The sentence must also say, at that time, that remoting and cluster are not enabled; that belongs to the ADR 0010 text it points to.

## Historical traceability (proposed)

The history stays followable without erasing any accepted text. Each fact below was checked on 2026-10-25 in the repository or in the issue tracker (`gh issue view`).

| Date | Event | Source |
|---|---|---|
| 2026-10-14 | ADR 0017 accepted by the Owner as the whole file at `d6c74c2` plus the A-list (A1-A13) | [`docs/adr/0017-supervision.md`](../adr/0017-supervision.md) lines 3-15; commit `d6c74c2` (dated 2026-10-14) |
| 2026-10-14 | The H5 probe run on Pekko v1.7.1 recorded the divergence: a reference of another ActorSystem is accepted and `Terminated` arrives (`existenceConfirmed=true`) | [`h5-sbt.out`](../../migration/stage-1/supervision/h4-repro/h5-sbt.out) line 502; [`INDEX.md`](../../migration/stage-1/supervision/h4-repro/INDEX.md) |
| 2026-10-22 | Fidelity audit comments on legacy issue [#443](https://github.com/getsyntegrity/gopekko/issues/443) confirm the rejection at `watch.go` and Pekko's acceptance (`DeathWatch.scala:35-49`) | comments on #443 |
| 2026-10-24 | #443 closed (state `CLOSED`, reason `NOT_PLANNED`; no closing comment). Its supersession by #785 and #827 is recorded in those issues ("Legacy tracking: #443", "Legacy closure means supersession, not delivery of this story") | `gh issue view 443`; bodies of [#785](https://github.com/getsyntegrity/gopekko/issues/785) and [#827](https://github.com/getsyntegrity/gopekko/issues/827) |
| 2026-10-25 | Owner decision GOV-15 option (b), recorded in [#1065](https://github.com/getsyntegrity/gopekko/issues/1065); the further decisions (Q1-Q5) were given in the work thread, not in a GitHub comment | #1065; [design document](cross-system-watch-design.md), Provenance |
| 2026-10-25 | ADR 0010 amendment (accepted as design on 2026-10-25) | [`cross-system-watch-adr-0010-amendment-proposal.md`](cross-system-watch-adr-0010-amendment-proposal.md) |
| Pending | DTH-01 [#785](https://github.com/getsyntegrity/gopekko/issues/785) and DTH-02 [#827](https://github.com/getsyntegrity/gopekko/issues/827) implement it (both blocked); the `pekko-actor/README.md` change (I10) lands with DTH-01 | story, "Dependencies" |

**Approved on 2026-10-25 (the applied wording is under "Decisions taken"):** each of the five ADR 0017 markers ends with the issue references `(GOV-15 #1065; DTH-01 #785)`, so a reader of ADR 0017 can follow the history above. The stem stays unchanged, so a grep for it still finds all five places. The exact resulting text is:

- Row variant (I5, I6, I9): "Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then this row describes the current state. (GOV-15 #1065; DTH-01 #785)"
- Paragraph variant (I7): "Note (proposed): Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then this paragraph describes the current state. Case 2 and case 1 are not affected. (GOV-15 #1065; DTH-01 #785)"
- Clause variant (I8): "For the handle of another System only: Superseded for `Watch`, `WatchWith` and `Unwatch` by the ADR 0010 amendment (proposed) once DTH-01 is implemented; until then this text describes the current state. (GOV-15 #1065; DTH-01 #785)"

All existing proposals above stay as written; this addition changes only the ending of the marker sentences.

## Decisions taken on 2026-10-25

1. Approved as design: the marker sentence and its placement in I5-I9.
2. ADR 0017 is amended in place and its historical text is kept (not the superseding-note alternative).
3. The README is updated only in the delivery of DTH-01 (not applied in this stack).
4. Approved: the references `(GOV-15 #1065; DTH-01 #785)` at the end of each marker.

**Applied wording.** The applied markers replaced "(proposed)" by "(accepted on 2026-10-25)", for example `docs/adr/0017-supervision.md:70`: "by the ADR 0010 amendment (accepted on 2026-10-25) once DTH-01 is implemented"; the paragraph variant (`:512`) begins "Note (2026-10-25):" instead of "Note (proposed):". The references are unchanged.

Remaining for the implementation: the README change at DTH-01. Nothing was executed for cross-System watch; no parity is claimed.
