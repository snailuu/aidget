use aidget::runtime::ui::{distinct_names, label_at, top_left_y, wide_text, ScreenLabel};

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

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn twin_displays_get_different_names() {
    assert_eq!(
        distinct_names(names(&["Monitor #7", "Monitor #7", "Monitor #7"])),
        names(&["Monitor #7", "Monitor #7 (2)", "Monitor #7 (3)"])
    );
}

#[test]
fn different_displays_keep_their_names_so_saved_frames_stay_valid() {
    assert_eq!(
        distinct_names(names(&["Monitor #1", "Monitor #2"])),
        names(&["Monitor #1", "Monitor #2"])
    );
}

#[test]
fn a_wide_buffer_ends_at_its_first_nul() {
    assert_eq!(wide_text(&[0x55, 0x32, 0, 0x58]), "U2");
}

#[test]
fn a_full_wide_buffer_keeps_all_its_text() {
    assert_eq!(wide_text(&[0x55, 0x32]), "U2");
}

#[test]
fn an_empty_wide_buffer_has_no_text() {
    assert_eq!(wide_text(&[0, 0]), "");
}
