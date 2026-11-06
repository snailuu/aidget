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

/// Read the names Windows shows in its display settings; monitors only report a
/// device path such as `\\.\DISPLAY1`.
#[cfg(windows)]
pub(super) fn screen_labels(app: &AppHandle) -> Result<Vec<ScreenLabel>> {
    let friendly = friendly_names();
    let monitors = app.available_monitors()?;
    let named: Vec<_> = monitors
        .iter()
        .filter_map(|monitor| Some((monitor, friendly.get(monitor.name()?)?.clone())))
        .collect();
    // Windows gives twins one name, where macOS numbers them itself.
    let labels = distinct_names(named.iter().map(|(_, label)| label.clone()).collect());
    Ok(named
        .iter()
        .zip(labels)
        .map(|((monitor, _), label)| {
            let origin = monitor.position();
            let scale = monitor.scale_factor();
            ScreenLabel {
                x: origin.x as f64 / scale,
                y: origin.y as f64 / scale,
                label,
            }
        })
        .collect())
}

/// Friendly monitor names by device path. A display can lack one, and a remote
/// session can refuse the query; a label is optional, so neither is an error.
#[cfg(windows)]
fn friendly_names() -> std::collections::HashMap<String, String> {
    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
        DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
    };
    use windows::Win32::Foundation::ERROR_SUCCESS;

    let (mut path_count, mut mode_count) = (0, 0);
    let mut paths = Vec::new();
    let mut modes = Vec::new();
    // SAFETY: each buffer holds as many elements as the count passed with it.
    let listed = unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
            == ERROR_SUCCESS
            && {
                paths.resize(path_count as usize, DISPLAYCONFIG_PATH_INFO::default());
                modes.resize(mode_count as usize, DISPLAYCONFIG_MODE_INFO::default());
                QueryDisplayConfig(
                    QDC_ONLY_ACTIVE_PATHS,
                    &mut path_count,
                    paths.as_mut_ptr(),
                    &mut mode_count,
                    modes.as_mut_ptr(),
                    None,
                ) == ERROR_SUCCESS
            }
    };
    if !listed {
        return Default::default();
    }
    paths.truncate(path_count as usize);
    paths
        .iter()
        .filter_map(|path| {
            let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    size: std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                    adapterId: path.sourceInfo.adapterId,
                    id: path.sourceInfo.id,
                },
                ..Default::default()
            };
            let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                    size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                ..Default::default()
            };
            // SAFETY: each header starts a struct of the size it declares.
            let found = unsafe {
                DisplayConfigGetDeviceInfo(&mut source.header) == 0
                    && DisplayConfigGetDeviceInfo(&mut target.header) == 0
            };
            let label = wide_text(&target.monitorFriendlyDeviceName);
            (found && !label.is_empty()).then(|| (wide_text(&source.viewGdiDeviceName), label))
        })
        .collect()
}

/// The text in a fixed UTF-16 buffer, which ends at its first NUL if it has one.
pub fn wide_text(buffer: &[u16]) -> String {
    let length = buffer
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..length])
}

/// Linux reports the monitor model, so it has no friendlier name to look up.
#[cfg(not(any(target_os = "macos", windows)))]
pub(super) fn screen_labels(_app: &AppHandle) -> Result<Vec<ScreenLabel>> {
    Ok(Vec::new())
}
