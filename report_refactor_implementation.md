# Refactor Implementation Report

Date: 2026-05-12

This report covers the code changes implemented in this pass.

## Scope

The refactor targeted three structural problems:

1. wrapper structs around league grouping and lineup shape
2. split player ownership between `Player` and `PlayerRecord`
3. rigid conference/division modeling that did not support custom leagues cleanly

## Implemented changes

### 1. Replaced `Conference` and `Division` wrapper structs with extensible enums

Updated file:

- `src/data/game/names.rs`

Old shape:

- `Conference` was a string wrapper
- `Division` was a string wrapper plus `teams: i32`

New shape:

- `Conference::{East, West, Custom(String)}`
- `Division::{Atlantic, Pacific, Custom { name: String, team_count: Option<i32> }}`

Result:

- built-in names still have dedicated variants
- custom leagues can create arbitrary conferences and divisions without inventing more wrapper structs
- call sites can work from semantic variants instead of opaque string containers

### 2. Removed lineup wrapper structs and flattened `Loadout`

Updated file:

- `src/data/line.rs`

Removed wrappers:

- `Line`
- `Pairing`
- `GoalieTandem`
- `LoadoutBuilder`

New `Loadout` shape:

- `forward_lines: [[i8; 3]; 4]`
- `defense_pairs: [[i8; 2]; 3]`
- `goalies: [i8; 2]`
- `starter_share: i8`

Result:

- one direct lineup struct instead of nested wrappers around player indexes
- less constructor and setter boilerplate
- easier serialization and simpler team line assignment

### 3. Moved player-owned data into `Player`

Updated files:

- `src/data/player.rs`
- `src/data/helper.rs`

Old shape:

- `Player` contained skill/projection/gameplay data
- `PlayerRecord` wrapped `Player` with `stats` and `contract`
- draft constructors accepted `DraftStatus` but did not store it
- physical and birth metadata existed elsewhere instead of on `Player`

New `Player` ownership:

- name
- age
- type / position / play style
- skating / goalie movement / projection / game view / skills
- `height_cm`
- `weight_kg`
- `birth_location`
- `draft_status`
- `stats`
- `contract`

Result:

- `Player` is now the full player model
- `PlayerRecord` has been removed
- `DraftStatus` is now a real stored field instead of a dead constructor parameter path

### 4. Flattened team identity into `Team`

Updated file:

- `src/data/team.rs`

Removed wrapper:

- `TeamIdentity`

New `Team` ownership now includes:

- `city`
- `name`
- `abbreviation`
- `conference`
- `division`

Result:

- fewer pass-through identity methods
- `Team` is the direct domain object rather than a shell around another identity struct
- call sites now use `team.abbreviation()` and `team.name()` directly

### 5. Updated dependent modules to use the flattened model

Updated files:

- `src/league_settings.rs`
- `src/season.rs`
- `src/sim_helper.rs`
- `src/testing/league_helper.rs`

Key changes:

- replaced `team.identity().abbreviation()` with `team.abbreviation()`
- replaced roster handling from `Vec<PlayerRecord>` to `Vec<Player>`
- updated season boxscore application to write stats directly onto `Player`
- updated sim helpers to read player attributes directly from `Player`

### 6. Added accessors and clone support where the new model needed it

Updated files:

- `src/data/location.rs`
- `src/data/general_data.rs`

Changes:

- `Location` and `Places` now derive `Clone`
- `Location` now exposes `country()`, `location()`, and `date()`
- `Type`, `Position`, and `PlayType` now derive `Clone`

## Structural outcome

Removed wrapper types:

- `Conference` struct
- `Division` struct
- `Line`
- `Pairing`
- `GoalieTandem`
- `LoadoutBuilder`
- `PlayerRecord`
- `TeamIdentity`

Retained but reshaped:

- `Conference` as enum
- `Division` as enum
- `Loadout` as direct array-backed lineup data

## Verification

Verification command:

```powershell
cargo check
```

Result:

- build passes
- warnings remain, mostly pre-existing style and dead-code warnings outside this refactor’s scope

## Save layout update

The league persistence model was updated after the initial refactor:

- `League` now owns `Vec<Team>` directly in `src/league_settings.rs`
- league save/load is handled through `League::save_to_context` and `League::load_from_context`
- `Sim` no longer relies on inline league serialization in `Sim.json`

Current on-disk league layout:

```text
data/<sim_id>/<game_id>/League/<league_name>/
  <league_name>.json
  teams/
    <team_abbreviation>.json
    ...
```

`<league_name>.json` stores league-level data:

- name
- level
- rules
- standings
- free agents

Team data is stored separately under `teams/`, which removes league-file duplication and makes per-team persistence explicit.

## Notes

- `season.rs` still contains the dormant persistence block reported earlier. This refactor did not remove it.
- `General` in `src/data/general_data.rs` still exists even though the important player bio/physical data now lives directly on `Player`. That module can be reduced further in a follow-up.
