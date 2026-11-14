# Amendment proposal for ADR 0024: Done keeps its owner check; the identity invariant is unchanged

**Status: ACCEPTED AS DESIGN by the Owner on 2026-10-25 (work thread). APPLIED to docs/adr in the PR that carries this commit (#1206); NOT implemented.** The applied sentences and a dated status note are in [`docs/adr/0024-actor-registry-retention-after-termination.md`](../adr/0024-actor-registry-retention-after-termination.md) and changes the status of no ADR. It clarifies text whose details the Owner accepted on 2026-10-20 (item 2 of the Decision section, lines 137-139; acceptance recorded at lines 145-149). It creates **no Watch exception** in ADR 0024. The Owner authorized the edit on 2026-10-25; the applied text is in the ADR and the block below is the historical proposal.
Provenance: option (b) of GOV-15 ([#1065](https://github.com/getsyntegrity/gopekko/issues/1065)) is recorded in #1065; the Owner decisions of 2026-10-25 (including decision 5: ID uniqueness is an invariant in force, no composite key, no API change) were given in the work thread, **not in a GitHub comment**; the exact text was approved as design on 2026-10-25 (see Status). See the [design document](cross-system-watch-design.md), Provenance.
**Blocked:** DTH-01 ([#785](https://github.com/getsyntegrity/gopekko/issues/785)), DTH-02 ([#827](https://github.com/getsyntegrity/gopekko/issues/827)) until the documents are merged by the Owner and their own readiness is checked; [#1200](https://github.com/getsyntegrity/gopekko/issues/1200) (`GOV-51`; [story](cross-system-watch-ownership-story.md)) and the amendments are approved as design. The Ask, Done and Stop contracts and authority stay out of this change: `Done` does not change.

## Exact original text replaced (`docs/adr/0024-actor-registry-retention-after-termination.md`, last sentence of item 2 of the Decision section, line 139)

> A handle of another System, or any user-implemented handle, gains no authority by knowing an ID (implementation detail of D2, added with T4).

## Exact proposed text

> For `Done`, a handle of another System, or any user-implemented handle, gains no authority by knowing an ID (implementation detail of D2, added with T4); `Done` does not change. This sentence is about `Done` only; the rules of the other APIs are in their own ADRs: for `Watch`, `WatchWith` and `Unwatch`, ADR 0010, where authority comes from the owning System of the handle, never from the ID.

## Exact proposed new sentence, after item 3 of the Decision section (lines 140-141)

> D2 relies on actor IDs being unique across the Systems of one process. That is an existing contractual invariant (`ID` "identifies one actor for the life of the process"); this amendment does not create it, adds no composite key and changes no API. A per-System ID sequence (option D of D2) would break it and would first need a decision of its own.

Note kept with the first sentence: `System.Stop` takes authority from the ID alone with no owner check (`pekko-actor/actor/stop.go:20-24`) and is a no-op for a foreign handle only because that ID is absent from the caller's registry.

## Resulting text of item 2 of the Decision section after the change (item 2 in full, with the replaced sentence)

> 2. **Native handles carry what they need.** A `Ref` already holds its cell. `idHandle` (`hierarchy.go:68-74`, built by `cell.identity()`) also carries the cell's `done` channel, nothing more.
>    `Done` answers from the handle when the registry has no record, and only for a handle this System issued: the handle's owner must be this System and, for a `Ref`, its cell must have been
>    registered by this System. For `Done`, a handle of another System, or any user-implemented handle, gains no authority by knowing an ID (implementation detail of D2, added with T4); `Done` does not change. This sentence is about `Done` only; the rules of the other APIs are in their own ADRs: for `Watch`, `WatchWith` and `Unwatch`, ADR 0010, where authority comes from the owning System of the handle, never from the ID.

The new sentence about ID uniqueness is inserted after item 3 and before item 4 ("Guardians stay registered"); item 3 and item 4 keep their text.

## Uniqueness of IDs

| Fact | Evidence |
|---|---|
| Invariant in force | `pekko-actor/actor/id.go:8` ("identifies one actor for the life of the process"), `id.go:14-18` (`idSeq` "keeps IDs unique across Systems"; `nextID` draws from it) |
| Depends on it | `watching map[ID]*watchReg` (`watch.go:217`: one registration or a false `ErrWatchConflict`); `Terminated.Who() ID` (`watch.go:22-25`); `System.Stop` (`stop.go:20-24`: a collision stops an unrelated local actor); `Done` (`system.go:183-187`: the registry lookup precedes the owner check) |
| Proof missing | no `_test.go` in `pekko-actor` references `idSeq` or `nextID` (grep, verified); the story specifies a cross-System uniqueness test |
| Not proposed | composite key `(system, id)`, any API change |

## Why it is necessary

The sentence says a handle of another System "gains no authority by knowing an ID". Read next to ADR 0010 under option (b), it could be taken to cover `Watch`; the clarification limits it to `Done` and points the other APIs to their ADRs. The uniqueness sentence writes down the invariant D2 already relies on. The details of item 2 were accepted by the Owner on 2026-10-20, so only the Owner can accept the change.

## Left as written

Item 3 and the "Status of the details in items 2 and 3" paragraph keep their text. `Done` answers `ErrUnknownHandle` for a foreign handle (`system.go:183-193`) and the Ask provenance mark is checked against the cell's own System (`ask_endpoint.go:138`). Option D is offered under D2(b) (line 128). Ask, Stop, remoting and cluster are not widened. Pekko has no counterpart for ID uniqueness (identity is path plus uid, A14 in the [design document](cross-system-watch-design.md)): this is a goPekko design clarification, not a Pekko-derived rule.

## Decisions taken on 2026-10-25

Approved as design by the Owner (work thread): the exact wording, including the uniqueness sentence in this ADR. It was applied with a dated status note. Nothing was executed for cross-System watch; no parity is claimed.
