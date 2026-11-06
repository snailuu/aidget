use std::collections::BTreeMap;

use aidget::runtime::ui::{gathered_frames, loadable_frames, relocation_target, DisplayNames};
use aidget::widgets::model::WidgetFrame;
use aidget::widgets::store::{Store, WidgetEdit, CONFIG_FILE};
use tauri::{LogicalPosition, LogicalRect, LogicalSize};
use tempfile::TempDir;

const NAMES: [&str; 2] = ["Monitor #1", "Monitor #2"];
const PRIMARY: usize = 0;
const EXTERNAL: usize = 1;

fn displays() -> DisplayNames<'static> {
    DisplayNames {
        names: NAMES.to_vec(),
        primary: Some(PRIMARY),
    }
}

fn frame(display: &str, pinned: Option<&str>) -> WidgetFrame {
    WidgetFrame {
        display: display.to_string(),
        pinned_display: pinned.map(str::to_string),
        x: 24.0,
        y: 24.0,
        w: 360.0,
        h: 220.0,
    }
}

fn area() -> LogicalRect<f64, f64> {
    LogicalRect {
        position: LogicalPosition::new(0.0, 0.0),
        size: LogicalSize::new(2560.0, 1410.0),
    }
}

#[test]
fn a_widget_follows_the_default_display() {
    let target = relocation_target(&frame("primary", None), Some("Monitor #2"), &displays());

    assert_eq!(target, Some(EXTERNAL));
}

#[test]
fn the_chosen_display_outranks_the_default_display() {
    let target = relocation_target(
        &frame("Monitor #2", Some("Monitor #1")),
        Some("Monitor #2"),
        &displays(),
    );

    assert_eq!(target, Some(PRIMARY));
}

#[test]
fn a_widget_stays_put_without_a_chosen_or_default_display() {
    assert_eq!(
        relocation_target(&frame("Monitor #2", None), None, &displays()),
        None
    );
}

#[test]
fn a_widget_already_on_its_display_stays_put() {
    assert_eq!(
        relocation_target(&frame("primary", None), Some("Monitor #1"), &displays()),
        None
    );
}

#[test]
fn a_disconnected_chosen_display_does_not_fall_through_to_the_default() {
    assert_eq!(
        relocation_target(
            &frame("primary", Some("Monitor #9")),
            Some("Monitor #2"),
            &displays()
        ),
        None
    );
}

#[test]
fn a_widget_on_a_disconnected_display_moves_to_its_target() {
    assert_eq!(
        relocation_target(&frame("Monitor #9", None), Some("Monitor #2"), &displays()),
        Some(EXTERNAL)
    );
}

#[test]
fn gathering_moves_every_other_widget_without_overlap() {
    let widgets = BTreeMap::from([
        ("a".to_string(), frame("primary", None)),
        ("b".to_string(), frame("primary", None)),
    ]);

    let (frames, moved) = gathered_frames(&widgets, EXTERNAL, &displays(), area());

    assert_eq!(moved, 2);
    assert_eq!(frames["a"].display, "Monitor #2");
    assert_eq!(frames["b"].display, "Monitor #2");
    assert!(frames["a"].x + frames["a"].w <= frames["b"].x);
}

#[test]
fn gathering_clears_chosen_displays_and_keeps_widgets_already_there_in_place() {
    let widgets = BTreeMap::from([
        ("pinned-away".to_string(), frame("primary", Some("primary"))),
        (
            "pinned-here".to_string(),
            frame("Monitor #2", Some("Monitor #2")),
        ),
        ("settled".to_string(), frame("Monitor #2", None)),
    ]);

    let (frames, moved) = gathered_frames(&widgets, EXTERNAL, &displays(), area());

    assert_eq!(moved, 1);
    assert_eq!(frames["pinned-away"].pinned_display, None);
    assert_eq!(frames["pinned-here"].pinned_display, None);
    assert_eq!(frames["pinned-here"].x, 24.0);
    assert!(!frames.contains_key("settled"));
}

#[test]
fn gathering_avoids_widgets_already_on_the_target() {
    let widgets = BTreeMap::from([
        ("settled".to_string(), frame("Monitor #2", None)),
        ("visitor".to_string(), frame("primary", None)),
    ]);

    let (frames, _) = gathered_frames(&widgets, EXTERNAL, &displays(), area());

    assert!(frames["visitor"].x >= 24.0 + 360.0);
}

#[test]
fn gathering_keeps_the_size_of_a_widget_larger_than_the_target() {
    let mut large = frame("primary", None);
    (large.w, large.h) = (3000.0, 1600.0);
    let widgets = BTreeMap::from([("large".to_string(), large)]);

    let (frames, _) = gathered_frames(&widgets, EXTERNAL, &displays(), area());

    assert_eq!((frames["large"].w, frames["large"].h), (3000.0, 1600.0));
}

#[test]
fn a_broken_widget_does_not_stop_the_others_from_gathering() {
    let dir = TempDir::new().unwrap();
    let store = Store::new(dir.path());
    store.ensure_layout().unwrap();
    store.create("good", "Good", WidgetEdit::default()).unwrap();
    store
        .create("broken", "Broken", WidgetEdit::default())
        .unwrap();
    std::fs::remove_file(store.widget_dir("broken").unwrap().join(CONFIG_FILE)).unwrap();
    let board = store.load_board().unwrap();

    let widgets = loadable_frames(&store, &board);
    let (frames, moved) = gathered_frames(&widgets, EXTERNAL, &displays(), area());
    let saved = store.move_widgets(board.revision, frames).unwrap();

    assert_eq!(moved, 1);
    assert_eq!(saved.widgets["good"].display, "Monitor #2");
}

#[test]
fn gathering_on_the_primary_display_stores_the_primary_name() {
    let widgets = BTreeMap::from([("a".to_string(), frame("Monitor #2", None))]);

    let (frames, _) = gathered_frames(&widgets, PRIMARY, &displays(), area());

    assert_eq!(frames["a"].display, "primary");
}

#[test]
fn the_primary_name_follows_the_first_display_when_none_is_reported() {
    let displays = DisplayNames {
        names: NAMES.to_vec(),
        primary: None,
    };

    assert_eq!(
        relocation_target(&frame("primary", None), Some("Monitor #1"), &displays),
        None
    );
}
