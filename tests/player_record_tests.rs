use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::projection::{DevelopmentCurve, ProjMax, Projection, ProjectionGenerationSettings};

#[test]
fn reads_undrafted_skater_fixture() {
    let input = include_str!("fixtures/undrafted_skater.txt");
    let record = PlayerRecord::read_text(input).expect("fixture should parse");

    assert_eq!(record.name(), "James Example");
    assert_eq!(record.age(), 18);
    assert!(matches!(record.draft_status(), DraftStatus::Undrafted));
    assert_eq!(record.player().skate_stats().speed(), 72);
    assert!(record.player().goalie_movement().is_none());
    assert!(matches!(
        record.projection().draft_projection().max_projection(),
        ProjMax::ELITE
    ));
    assert!(matches!(
        record.projection().development_profile().curve(),
        DevelopmentCurve::LINEAR
    ));
}

#[test]
fn reads_drafted_goalie_fixture() {
    let input = include_str!("fixtures/drafted_goalie.txt");
    let record = PlayerRecord::read_text(input).expect("fixture should parse");

    assert_eq!(record.name(), "Pat Netminder");
    assert_eq!(record.age(), 19);
    let movement = record
        .player()
        .goalie_movement()
        .expect("goalie movement should exist");
    assert_eq!(movement.side(), 80);
    assert_eq!(movement.up_down(), 76);
    assert_eq!(movement.push(), 82);

    match record.draft_status() {
        DraftStatus::Drafted(draft_data) => {
            assert_eq!(draft_data.draft_year(), 2026);
            assert_eq!(draft_data.draft_round(), 1);
            assert_eq!(draft_data.overall_pick(), 8);
            assert_eq!(draft_data.team(), "VAN");
        }
        DraftStatus::Undrafted => panic!("expected drafted status"),
    }
}

#[test]
fn writes_back_to_same_text_for_undrafted_fixture() {
    let input = include_str!("fixtures/undrafted_skater.txt");
    let record = PlayerRecord::read_text(input).expect("fixture should parse");

    assert_eq!(record.write_text(), normalize_fixture(input));
}

#[test]
fn writes_back_to_same_text_for_drafted_fixture() {
    let input = include_str!("fixtures/drafted_goalie.txt");
    let record = PlayerRecord::read_text(input).expect("fixture should parse");

    assert_eq!(record.write_text(), normalize_fixture(input));
}

#[test]
fn rejects_missing_required_fields() {
    let input = "name=Missing Fields\nage=18\n";
    let error = match PlayerRecord::read_text(input) {
        Ok(_) => panic!("parse should fail"),
        Err(error) => error,
    };

    assert!(error.contains("Missing required field"));
}

#[test]
fn quality_projection_clamps_and_produces_valid_ranges() {
    let projection = Projection::from_quality_with_settings(
        1.5,
        ProjectionGenerationSettings::new(1.2, -0.5, 0.8, 0.1, 0.0, 1.0, 0.9),
    );

    assert!(matches!(
        projection.draft_projection().max_projection(),
        ProjMax::GENERATIONAL
    ));
    assert_eq!(projection.draft_projection().draft_round_grade(), 1);
    assert_eq!(projection.draft_projection().overall_pick_estimate(), 1);
    assert!(projection.development_profile().floor() <= projection.development_profile().ceiling());
    assert_eq!(projection.development_profile().coachability(), 99);
    assert_eq!(projection.development_profile().injury_risk(), 1);
}

#[test]
fn volatility_drives_boom_bust_curve() {
    let projection = Projection::from_quality_with_settings(
        0.6,
        ProjectionGenerationSettings::new(0.5, 0.5, 0.5, 0.95, 0.2, 0.5, 0.5),
    );

    assert!(matches!(
        projection.development_profile().curve(),
        DevelopmentCurve::BOOM_BUST
    ));
}

fn normalize_fixture(input: &str) -> String {
    let mut output = input.to_string();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}
