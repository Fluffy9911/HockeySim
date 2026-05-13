# HockeySim Refactor Report

Date: 2026-05-12

This report set does not modify Rust code. It documents where the current type model is heavier than the behavior requires, with emphasis on removing unnecessary structs and collapsing duplicate representations.

## Executive summary

The main struct problems are concentrated in four places:

1. `season.rs` contains a dead manual persistence layer that is not called anywhere and already shows up in `cargo check` dead-code warnings.
2. The project has two parallel schedule/game models:
   - `season::{ScheduledGame, ScheduledGameResult, Season}`
   - `sim_engine::{Match, Schedule}`
3. Several structs exist only to wrap a few primitive fields with no real domain behavior:
   - `LeagueTeamEntry`
   - `Conference`
   - `Division`
   - `Line`
   - `Pairing`
   - `GoalieTandem`
   - `LoadoutBuilder`
4. `PlayerRecord` constructors accept `DraftStatus` but do not store it, which is a strong signal that the surrounding draft structs are incomplete or unnecessary in their current form.

## Highest-value removals

- Remove the dead season persistence structs and helpers in `src/season.rs`.
- Pick one schedule model and delete the other.
- Collapse lineup structs into a single `Loadout` shape.
- Either store `DraftStatus` inside `PlayerRecord` or remove the parameter and the dead draft wrapper path.

## Files in this report set

- `report_refactor_inventory.md`
- `report_refactor_candidates.md`
- `report_refactor_plan.md`
