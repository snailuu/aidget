use aidget::runtime::ui::{pinned_after_move, records_move, DisplayNames};
use aidget::widgets::model::WidgetFrame;

fn frame(display: &str, pinned: Option<&str>) -> WidgetFrame {
    WidgetFrame {
        display: display.to_string(),
        pinned_display: pinned.map(str::to_string),
        ..WidgetFrame::default_for(0)
    }
}

#[test]
fn dragging_to_another_display_chooses_that_display() {
    assert_eq!(
        pinned_after_move(&frame("primary", None), "Monitor #2", true).as_deref(),
        Some("Monitor #2")
    );
}

#[test]
fn dragging_within_a_display_keeps_following_the_default() {
    assert_eq!(
        pinned_after_move(&frame("primary", None), "primary", true),
        None
    );
}

#[test]
fn dragging_replaces_a_choice_the_widget_has_left() {
    // The system moved the window away, and the user then settled it here.
    assert_eq!(
        pinned_after_move(&frame("primary", Some("Monitor #2")), "primary", true).as_deref(),
        Some("primary")
    );
}

#[test]
fn a_system_move_never_chooses_a_display() {
    assert_eq!(
        pinned_after_move(&frame("Monitor #2", None), "primary", false),
        None
    );
}

#[test]
fn a_system_move_keeps_the_chosen_display() {
    assert_eq!(
        pinned_after_move(&frame("Monitor #2", Some("Monitor #2")), "primary", false).as_deref(),
        Some("Monitor #2")
    );
}

const NAMES: [&str; 1] = ["Monitor #1"];

fn connected() -> DisplayNames<'static> {
    DisplayNames {
        names: NAMES.to_vec(),
        primary: Some(0),
    }
}

#[test]
fn the_system_evacuating_a_missing_display_is_not_recorded() {
    assert!(!records_move("Monitor #2", &connected(), false, false));
}

#[test]
fn dragging_a_widget_off_a_missing_display_is_recorded() {
    assert!(records_move("Monitor #2", &connected(), true, false));
}

#[test]
fn a_window_manager_move_on_a_connected_display_is_recorded() {
    assert!(records_move("Monitor #1", &connected(), false, false));
    assert!(records_move("primary", &connected(), false, false));
}

#[test]
fn a_click_on_the_drag_handle_keeps_the_home_of_a_widget_on_a_stand_in_display() {
    // The widget's display is gone, so it sits where the frame was fitted. A drag
    // session without movement must not turn that spot into its new home.
    assert!(!records_move("Monitor #2", &connected(), true, true));
}

#[test]
fn a_window_that_never_left_its_frame_records_nothing() {
    assert!(!records_move("Monitor #1", &connected(), false, true));
}

#[test]
fn a_window_manager_move_is_recorded_when_no_primary_display_is_reported() {
    let connected = DisplayNames {
        names: NAMES.to_vec(),
        primary: None,
    };

    assert!(records_move("primary", &connected, false, false));
}
