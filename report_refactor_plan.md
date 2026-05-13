# Refactor Sequence

This is the order I would use for the actual refactor once code changes are allowed.

## Phase 1: delete dead code

1. Remove `LeagueMeta`, `SeasonMeta`, `TeamBuilder`.
2. Remove the unused parser/writer helpers in `season.rs`.
3. Re-run `cargo check` and use the warning count as the baseline reduction metric.

Expected result: large complexity drop with low behavioral risk.

## Phase 2: unify schedule flow

1. Choose `season.rs` as the canonical season/schedule module.
2. Migrate any remaining `sim_engine::{Match, Schedule}` consumers to `season`.
3. Delete the duplicate schedule structs.

Expected result: one model for scheduled games, one model for played games, fewer translation points.

## Phase 3: trim lightweight wrappers

1. Replace `Conference` and `Division` with enums or plain values.
2. Remove `LeagueTeamEntry` if league membership can be derived from `LeagueState.teams`.
3. Collapse lineup structs into a compact `Loadout`.

Expected result: less boilerplate, fewer constructors/getters, easier serialization.

## Phase 4: fix incomplete domain modeling

1. Decide whether draft status is a real persisted part of a player record.
2. If yes, store it in `PlayerRecord`.
3. If no, remove `DraftStatus` from constructors and prune unused draft wrappers.

Expected result: constructors stop lying about what the type actually owns.

## Guardrails

- Do not change behavior and data layout in the same commit as dead-code deletion.
- Keep save-format changes isolated.
- Add focused tests around season simulation and roster serialization before collapsing schedule structs.

## Success criteria

- one schedule model
- one game-result model
- no dead structs in `season.rs`
- no builder-only wrapper structs for lineup indices
- `PlayerRecord` constructors match stored fields
