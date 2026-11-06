use aidget::runtime::ui::{label_at, top_left_y, ScreenLabel};

fn screens() -> [ScreenLabel; 2] {
    [
        ScreenLabel {
            x: 0.0,
            y: 0.0,
            label: "Built-in Retina Display".to_string(),
        },
        ScreenLabel {
            x: -719.0,
            y: -1440.0,
            label: "U27P10".to_string(),
        },
    ]
}

#[test]
fn a_monitor_takes_the_label_of_the_screen_at_its_corner() {
    assert_eq!(label_at(&screens(), -719.0, -1440.0), Some("U27P10"));
}

#[test]
fn a_monitor_without_a_matching_screen_has_no_label() {
    assert_eq!(label_at(&screens(), 2000.0, 0.0), None);
}

#[test]
fn rounding_from_physical_pixels_does_not_lose_the_match() {
    assert_eq!(label_at(&screens(), -719.5, -1439.5), Some("U27P10"));
}

#[test]
fn an_appkit_origin_above_the_primary_display_becomes_a_negative_y() {
    assert_eq!(top_left_y(956.0, 1440.0, 956.0), -1440.0);
}

#[test]
fn the_primary_display_keeps_its_origin() {
    assert_eq!(top_left_y(0.0, 956.0, 956.0), 0.0);
}
