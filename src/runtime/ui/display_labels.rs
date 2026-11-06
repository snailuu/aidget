//! Display names. Frames and settings keep the stable names from `display_names`;
//! labels are the names the system shows, only for people to read.
use tauri::{AppHandle, Monitor};

use crate::common::error::Result;

/// The name a frame stores for a monitor at `index`.
///
/// macOS and Windows report a device name. Wayland can report none, and an
/// unnamed monitor must not collapse into `primary`, or every multi-screen
/// frame would be restored onto one screen, so those keep the list position
/// and origin as the identifier.
pub fn display_name(monitor: &Monitor, index: usize) -> String {
    match monitor.name().filter(|name| !name.is_empty()) {
        Some(name) => name.clone(),
        None => {
            let position = monitor.position();
            format!("display-{index}@{}:{}", position.x, position.y)
        }
    }
}

/// The names frames store for `monitors`, in the same order.
pub(super) fn display_names(monitors: &[Monitor]) -> Vec<String> {
    distinct_names(
        monitors
            .iter()
            .enumerate()
            .map(|(index, monitor)| display_name(monitor, index))
            .collect(),
    )
}

/// Tell repeated names apart: macOS names a monitor after its model, so twins
/// share one. The first keeps its plain name and saved frames stay valid.
///
/// shortcut: twins are told apart by list order, so they can swap if the system
/// reorders them; use a hardware identifier if that shows up in practice.
pub fn distinct_names(names: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashMap::new();
    names
        .into_iter()
        .map(|name| {
            let count = seen.entry(name.clone()).or_insert(0);
            *count += 1;
            if *count == 1 {
                name
            } else {
                format!("{name} ({count})")
            }
        })
        .collect()
}

/// A display's name and its top-left corner in the coordinates monitors report.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenLabel {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

/// The label of the screen whose top-left corner is `(x, y)`.
pub fn label_at(screens: &[ScreenLabel], x: f64, y: f64) -> Option<&str> {
    // Monitors report physical pixels, so the logical corner can round apart.
    screens
        .iter()
        .find(|screen| (screen.x - x).abs() < 1.0 && (screen.y - y).abs() < 1.0)
        .map(|screen| screen.label.as_str())
}

/// Convert an AppKit origin, measured upward from the primary display's bottom
/// edge, to the top-left origin monitors report.
pub fn top_left_y(origin_y: f64, height: f64, primary_height: f64) -> f64 {
    primary_height - origin_y - height
}

/// Read the system's display names. `NSScreen` belongs to the main thread, while
/// `displays` also runs on IPC workers.
#[cfg(target_os = "macos")]
pub(super) fn screen_labels(app: &AppHandle) -> Result<Vec<ScreenLabel>> {
    super::drag::run_on_main_thread(app, || {
        let main_thread = objc2::MainThreadMarker::new().expect("runs on the main thread");
        let screens = objc2_app_kit::NSScreen::screens(main_thread);
        // AppKit keeps the primary display first, and its origin anchors the others.
        let primary_height = screens
            .firstObject()
            .map_or(0.0, |screen| screen.frame().size.height);
        screens
            .iter()
            .map(|screen| {
                let frame = screen.frame();
                ScreenLabel {
                    x: frame.origin.x,
                    y: top_left_y(frame.origin.y, frame.size.height, primary_height),
                    label: screen.localizedName().to_string(),
                }
            })
            .collect()
    })
}

/// Only macOS replaces its monitor names here; other systems already report theirs.
#[cfg(not(target_os = "macos"))]
pub(super) fn screen_labels(_app: &AppHandle) -> Result<Vec<ScreenLabel>> {
    Ok(Vec::new())
}
