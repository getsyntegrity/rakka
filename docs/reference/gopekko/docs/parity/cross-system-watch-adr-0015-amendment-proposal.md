# Amendment proposal for ADR 0015: a handle of another System is not invalid for the three watch calls

**Status: ACCEPTED AS DESIGN by the Owner on 2026-10-25 (work thread). APPLIED to docs/adr in the PR that carries this commit (#1206); NOT implemented.** The applied sentence is in [`docs/adr/0015-ask-and-reply.md`](../adr/0015-ask-and-reply.md) and changes the status of no ADR. It adds one scoped sentence to the rule of lines 158-161, which is marked **[Owner, R2]**, and rewords nothing in the existing Ask condition. The Owner authorized the edit on 2026-10-25; the applied text is in the ADR and the block below is the historical proposal.
Provenance: option (b) of GOV-15 ([#1065](https://github.com/getsyntegrity/gopekko/issues/1065)) is recorded in #1065; the Owner decisions of 2026-10-25 (including decision 4: amend the contract rule now, with the error message and comments left to the implementation story) were given in the work thread, **not in a GitHub comment**; the exact text was approved as design on 2026-10-25 (see Status). See the [design document](cross-system-watch-design.md), Provenance.
**Blocked:** DTH-01 ([#785](https://github.com/getsyntegrity/gopekko/issues/785)), DTH-02 ([#827](https://github.com/getsyntegrity/gopekko/issues/827)) until the documents are merged by the Owner and their own readiness is checked; [#1200](https://github.com/getsyntegrity/gopekko/issues/1200) (`GOV-51`; [story](cross-system-watch-ownership-story.md)) and the amendments are approved as design. The Ask, Done and Stop contracts and authority stay out of this change.

## Exact original text (`docs/adr/0015-ask-and-reply.md`, lines 158-161, the whole paragraph "One sentinel per condition [Owner, R2]")

> **One sentinel per condition [Owner, R2].** The condition "nil, foreign or non-native handle" has one sentinel,
> `ErrInvalidHandle`, and Ask does not publish a second name for it. Local Watch (first drafted on the branch
> `feat/actor-local-watch`, since merged as #41) declares `ErrInvalidHandle` in `pekko-actor/actor/errors.go`, with a
> message specific to Watch ("invalid handle for watch: ..."). Rules as first agreed:

The paragraph ends at line 161 with "Rules as first agreed:"; the bullet list that follows (line 163 on) is not part of the paragraph and is not changed.

## Exact proposed text: insertion point and the added sentence

Insertion point: immediately after the sentence that ends "... a second name for it." (end of line 159) and before "Local Watch (first drafted on the branch ...". The original sentences are kept as they are; only this sentence is inserted:

> For `Watch`, `WatchWith` and `Unwatch`, a native handle of another System of the same process is not an invalid handle (ADR 0010, amendment proposed, pending acceptance). For every other API, including Ask, the condition and its sentinel are unchanged.

## Resulting paragraph (complete, with the sentence inserted)

> **One sentinel per condition [Owner, R2].** The condition "nil, foreign or non-native handle" has one sentinel,
> `ErrInvalidHandle`, and Ask does not publish a second name for it. For `Watch`, `WatchWith` and `Unwatch`, a native handle of another System of the same process is not an invalid handle (ADR 0010, amendment proposed, pending acceptance). For every other API, including Ask, the condition and its sentinel are unchanged. Local Watch (first drafted on the branch
> `feat/actor-local-watch`, since merged as #41) declares `ErrInvalidHandle` in `pekko-actor/actor/errors.go`, with a
> message specific to Watch ("invalid handle for watch: ..."). Rules as first agreed:

## Effect on `errors.Is`

- `errors.Is(err, ErrInvalidHandle)` no longer applies to the accepted foreign handle passed to the three watch calls: the call is valid and the watcher does not stop. Today it stops the watcher (`pekko-actor/actor/watch_api_test.go:544`, `pekko-actor/actor/watch_internal_test.go:345`), so this is a behavior change, not a textual one.
- It keeps applying to every other case: nil, zero ID, a non-native `Handle`, an invalid Ask target, and the Ask reply endpoint passed to a watch call.
- The Status paragraph of 2026-10-11 (lines 169-174) stays as history and is not extended to the accepted case.
- The existing word "foreign" in the R2 sentence is not touched. The ADR already records that Ask cannot reject a foreign native ref without an origin identity (line 260; deferred, line 442; `pekko-actor/actor/ask_endpoint.go:133-142`, test `ask_endpoint_internal_test.go:263-269`); this amendment does not address that existing text-versus-code gap.

## Deferred to the implementation story (Owner decision 4)

The error message ("... nil, of another system or not a native actor handle", `pekko-actor/actor/errors.go:48-51`, example in ADR lines 172-173) and the comments that quote it (`errors.go:48-50`, `pekko-actor/actor/ref.go:27`), with any test or document matching the old text (not searched exhaustively: **pending verification**).

## Why it is necessary

The R2 rule lists "foreign" in the one invalid-handle condition, and Watch declared that sentinel for a handle of another System. After option (b) that case is valid for the three watch calls, so the rule needs the scoped sentence.

## Left as written

D6 "Ask between `ActorSystem`s and remote ask are deferred **[Owner, Q6]**" (line 241); line 276 and **AK23** (line 385): `resolve` rejects a handle without the unexported `owner() *System`, the reply endpoint must not implement it, `Watch(askRef)`, `WatchWith(askRef, m)` and `Unwatch(askRef)` stop the watcher with `ErrInvalidHandle`, `Stop` is a no-op, `Done(askRef)` is `ErrUnknownHandle`. `askTarget`, `Stop` and `Done` do not change. AK23 tests exist at `pekko-actor/actor/ask_endpoint_internal_test.go:156-182, 244-247` (existence confirmed, not run) and must keep passing.

## Decisions taken on 2026-10-25

Approved as design by the Owner (work thread): the wording of the **[Owner, R2]** rule with the added sentence. AK23 needs no further line. Nothing was executed for cross-System watch; no parity is claimed; Pekko is not affected (Ask is not changed).

**Applied wording.** In the R2 paragraph (`docs/adr/0015-ask-and-reply.md:159`) the parenthetical "(ADR 0010, amendment proposed, pending acceptance)" was applied as "(ADR 0010, Amendment 2026-10-25, accepted by the Owner on 2026-10-25 as a design and not yet implemented)".
