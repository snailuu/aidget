//! Names the system shows for displays. Frames and settings keep the stable names
//! from `display_name`; these labels are only for people to read.
use tauri::AppHandle;

use crate::common::error::Result;

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
