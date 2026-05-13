# Struct Inventory

## Strongly active core structs

These are central enough that they should probably remain, though some fields may move:

- `src/data/player.rs`: `Player`
- `src/data/team.rs`: `Team`
- `src/league_settings.rs`: `League`
- `src/league_settings.rs`: `LeagueRules`
- `src/league_settings.rs`: `TeamStanding`
- `src/season.rs`: `LeagueState`
- `src/data/stats.rs`: stats structs
- `src/data/contract.rs`: contract structs
- `src/data/staff.rs`: `StaffMember`

## Active but over-modeled structs

These are used, but the current shape is heavier than the behavior:

- `src/data/team.rs:21` `TeamIdentity`
- `src/data/game/names.rs:5` `Division`
- `src/data/game/names.rs:19` `Conference`
- `src/league_settings.rs:43` `LeagueTeamEntry`
- `src/season.rs:9` `ScheduledGameResult`
- `src/sim_engine.rs:52` `Match`
- `src/sim_engine.rs:60` `Schedule`
- `src/data/line.rs:3` `Line`
- `src/data/line.rs:13` `Pairing`
- `src/data/line.rs:20` `GoalieTandem`
- `src/data/line.rs:132` `LoadoutBuilder`

## Dead or effectively dead structs

These were called out by `cargo check` dead-code warnings or are only referenced inside dead code:

- `src/season.rs:302` `LeagueMeta`
- `src/season.rs:315` `SeasonMeta`
- `src/season.rs:321` `TeamBuilder`

## Structs with a design smell

- `src/data/helper.rs:14` `DraftData`
- `src/data/helper.rs:22` `PlayerRecord`

`PlayerRecord::new`, `new_with_stats`, and `new_with_contract` all accept `DraftStatus` but never store it. That means the draft wrapper path is currently decorative rather than functional.
