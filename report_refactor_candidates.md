# Refactor Candidates

## 1. Remove the dead season persistence block

### Evidence

- `src/season.rs:302` `LeagueMeta`
- `src/season.rs:315` `SeasonMeta`
- `src/season.rs:321` `TeamBuilder`
- `cargo check` reports all three as never constructed.
- `cargo check` also reports the surrounding parser/writer helpers as never used.

### Recommendation

Delete the entire dormant text serialization layer from `season.rs` unless there is an immediate plan to wire it back in.

### Why this matters

This block adds a large amount of structural noise around save/load concerns, but the code path is disconnected. It makes the module look much more complex than it is.

## 2. Collapse the duplicate schedule models

### Evidence

- `src/season.rs:16` `ScheduledGame`
- `src/season.rs:23` `Season`
- `src/sim_engine.rs:52` `Match`
- `src/sim_engine.rs:60` `Schedule`

Both subsystems model:

- a date or day
- two teams
- an optional played result

### Recommendation

Keep one scheduling model only.

Preferred direction:

- keep `season::{Season, ScheduledGame}`
- remove `sim_engine::{Match, Schedule}`

Reason: `season.rs` is already connected to standings, team updates, and simulation flow. `sim_engine.rs` currently looks more like an older parallel path.

## 3. Collapse `ScheduledGameResult` into shared game outcome data

### Evidence

- `src/season.rs:9` `ScheduledGameResult`
- `src/league_settings.rs:73` `SimulatedGame`

`ScheduledGameResult` stores a reduced subset of `SimulatedGame`:

- goals
- overtime
- shootout

### Recommendation

Choose one of these:

1. Store `SimulatedGame` directly in scheduled games.
2. Create one shared lean outcome struct and use it everywhere schedule storage needs only results.

Avoid maintaining two structs for the same outcome shape.

## 4. Remove `LeagueTeamEntry` or derive it instead of storing it

### Evidence

- `src/league_settings.rs:43` `LeagueTeamEntry`
- `src/league_settings.rs:27` `League.team_registry: Vec<LeagueTeamEntry>`
- `src/season.rs:30-34` `LeagueState` already owns `Vec<Team>`

`LeagueTeamEntry` contains only abbreviation and level, both already present on `Team`.

### Recommendation

If `LeagueState` is the operational root, derive registry information from `teams` instead of storing a second lightweight team list inside `League`.

If persistent separation is needed later, replace it with a stable team id list rather than a second partial team struct.

## 5. Flatten `Conference`, `Division`, and possibly `TeamIdentity`

### Evidence

- `src/data/game/names.rs:5` `Division` is mostly a string wrapper plus `teams: i32`
- `src/data/game/names.rs:19` `Conference` is a string wrapper
- `src/data/team.rs:21` `TeamIdentity` bundles these wrappers with city/name/abbreviation

### Recommendation

Use enums or plain strings for conference/division. The current wrappers do not carry enough behavior to justify distinct structs.

Practical options:

1. Replace `Conference` with an enum.
2. Replace `Division` with an enum.
3. Keep `TeamIdentity` only if you expect identity to move independently of the rest of `Team`.
4. Otherwise fold `TeamIdentity` directly into `Team`.

## 6. Replace lineup wrapper structs with one compact `Loadout`

### Evidence

- `src/data/line.rs:3` `Line`
- `src/data/line.rs:13` `Pairing`
- `src/data/line.rs:20` `GoalieTandem`
- `src/data/line.rs:132` `LoadoutBuilder`

These structs only wrap player indices.

### Recommendation

Collapse them into one `Loadout` struct with arrays:

```rust
pub struct Loadout {
    forward_lines: [[i8; 3]; 4],
    defense_pairs: [[i8; 2]; 3],
    goalies: [i8; 2],
    starter_share: i8,
}
```

This removes four wrapper types plus the builder without losing meaning.

## 7. Fix or remove the draft wrapper path

### Evidence

- `src/data/helper.rs:8-10` `DraftStatus`
- `src/data/helper.rs:14-19` `DraftData`
- `src/data/helper.rs:65-106` `PlayerRecord` constructors accept `DraftStatus`
- none of those constructors store `DraftStatus`

### Recommendation

Pick one:

1. Add `draft_status: DraftStatus` to `PlayerRecord`.
2. Remove the `DraftStatus` parameter from constructors and trim the unused wrapper path.

Current state is misleading and makes the model look richer than it is.

## 8. Keep `StaffRatings` and `StaffDevelopment`

These look justified.

Reason:

- they are cohesive subdomains
- they are actually used by `StaffMember`
- flattening them would make `StaffMember` noisier rather than simpler

## 9. Keep `GameView` and `Skills`

These also look justified.

Reason:

- they represent distinct attribute groups
- development helpers operate on them as grouped concepts
- flattening them would increase field sprawl in `Player`
