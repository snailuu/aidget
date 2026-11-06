//! Moving existing widgets between displays: the choice on a widget card,
//! reopening on the chosen or default display, and gathering every widget.
use std::collections::{BTreeMap, HashSet};

use tauri::{AppHandle, LogicalPosition, LogicalRect, LogicalSize, Manager};

use crate::common::error::{Error, Result};
use crate::settings;
use crate::widgets::model::{validate_id, Board, WidgetFrame};
use crate::widgets::store::Store;

use super::geometry::{connected_displays, logical_work_area, place_in_work_area};
use super::state::UiState;
use super::widget_windows::apply_board;

/// The names frames use for connected displays, with the primary's position.
#[derive(Debug, Clone, Copy)]
pub struct DisplayNames<'a> {
    pub names: &'a [&'a str],
    pub primary: Option<usize>,
}

impl DisplayNames<'_> {
    /// Resolve a stored name; `primary` follows the system's primary display.
    pub(super) fn index(&self, name: &str) -> Option<usize> {
        if name == "primary" {
            return self.primary;
        }
        self.names.iter().position(|candidate| *candidate == name)
    }

    /// The name a frame stores for the display at `index`.
    fn stored(&self, index: usize) -> String {
        if Some(index) == self.primary {
            "primary".to_string()
        } else {
            self.names[index].to_string()
        }
    }
}

/// The display a widget belongs on, when that differs from where it is. Its own
/// choice outranks the default, and a display that is not connected leaves it put.
pub fn relocation_target(
    frame: &WidgetFrame,
    default: Option<&str>,
    displays: DisplayNames<'_>,
) -> Option<usize> {
    let target = displays.index(frame.pinned_display.as_deref().or(default)?)?;
    (displays.index(&frame.display) != Some(target)).then_some(target)
}

/// `frame` moved into the first free slot of the display at `target`.
fn frame_on_display(
    frame: &WidgetFrame,
    target: usize,
    displays: DisplayNames<'_>,
    area: LogicalRect<f64, f64>,
    occupied: &[LogicalRect<f64, f64>],
) -> WidgetFrame {
    let placed = place_in_work_area(area, LogicalSize::new(frame.w, frame.h), occupied);
    WidgetFrame {
        display: displays.stored(target),
        x: placed.position.x,
        y: placed.position.y,
        w: placed.size.width,
        h: placed.size.height,
        pinned_display: frame.pinned_display.clone(),
    }
}

fn frame_rect(frame: &WidgetFrame) -> LogicalRect<f64, f64> {
    LogicalRect {
        position: LogicalPosition::new(frame.x, frame.y),
        size: LogicalSize::new(frame.w, frame.h),
    }
}

/// The frames that gather every widget on `target` and clear their own choices,
/// with how many widgets change display. Unchanged frames are left out, and a
/// widget already on `target` keeps its position.
pub fn gathered_frames(
    widgets: &BTreeMap<String, WidgetFrame>,
    target: usize,
    displays: DisplayNames<'_>,
    area: LogicalRect<f64, f64>,
) -> (BTreeMap<String, WidgetFrame>, usize) {
    let on_target = |frame: &WidgetFrame| displays.index(&frame.display) == Some(target);
    let mut occupied: Vec<_> = widgets
        .values()
        .filter(|frame| on_target(frame))
        .map(frame_rect)
        .collect();
    let mut frames = BTreeMap::new();
    let mut moved = 0;
    for (id, frame) in widgets {
        let mut next = frame.clone();
        next.pinned_display = None;
        if !on_target(frame) {
            next = frame_on_display(&next, target, displays, area, &occupied);
            occupied.push(frame_rect(&next));
            moved += 1;
        } else if frame.pinned_display.is_none() {
            continue;
        }
        frames.insert(id.clone(), next);
    }
    (frames, moved)
}

/// Rectangles of the enabled widgets on `target`, other than `id`.
fn occupied_on(
    store: &Store,
    board: &Board,
    id: &str,
    target: usize,
    displays: DisplayNames<'_>,
) -> Result<Vec<LogicalRect<f64, f64>>> {
    let enabled: HashSet<_> = store
        .list()?
        .widgets
        .into_iter()
        .filter(|widget| widget.enabled)
        .map(|widget| widget.id)
        .collect();
    Ok(board
        .widgets
        .iter()
        .filter(|(existing, frame)| {
            existing.as_str() != id
                && enabled.contains(*existing)
                && displays.index(&frame.display) == Some(target)
        })
        .map(|(_, frame)| frame_rect(frame))
        .collect())
}

/// `frame` moved to the display it belongs on, when that is another display.
fn relocated(
    app: &AppHandle,
    store: &Store,
    board: &Board,
    id: &str,
    frame: &WidgetFrame,
) -> Result<Option<WidgetFrame>> {
    let default = settings::load(store.root())?.default_display;
    let (monitors, primary) = connected_displays(app)?;
    let names: Vec<_> = monitors.iter().map(|(name, _)| name.as_str()).collect();
    let displays = DisplayNames {
        names: &names,
        primary,
    };
    let Some(target) = relocation_target(frame, default.as_deref(), displays) else {
        return Ok(None);
    };
    Ok(Some(frame_on_display(
        frame,
        target,
        displays,
        logical_work_area(&monitors[target].1),
        &occupied_on(store, board, id, target, displays)?,
    )))
}

/// Before the main window reopens a widget, move it to the display it belongs on.
pub(super) fn relocate_on_open(app: &AppHandle, store: &Store, id: &str) -> Result<()> {
    let board = store.load_board()?;
    let Some(frame) = board.widgets.get(id) else {
        return Ok(());
    };
    if let Some(next) = relocated(app, store, &board, id, frame)? {
        apply_board(app, &store.move_widget(id, board.revision, next)?);
    }
    Ok(())
}

/// Apply the display chosen on a widget card; `None` returns it to the default.
pub fn set_widget_display(app: &AppHandle, id: &str, display: Option<String>) -> Result<()> {
    validate_id(id)?;
    let store = Store::new(app.state::<UiState>().root.lock().unwrap().clone());
    let board = store.load_board()?;
    let mut frame = board
        .widgets
        .get(id)
        .cloned()
        .ok_or_else(|| Error::InvalidParams(format!("widget `{id}` has no frame")))?;
    frame.pinned_display = display;
    let frame = relocated(app, &store, &board, id, &frame)?.unwrap_or(frame);
    apply_board(app, &store.move_widget(id, board.revision, frame)?);
    Ok(())
}

/// Gather every widget on the default display and report how many moved.
pub fn move_widgets_to_default_display(app: &AppHandle) -> Result<usize> {
    let store = Store::new(app.state::<UiState>().root.lock().unwrap().clone());
    let default = settings::load(store.root())?
        .default_display
        .ok_or_else(|| Error::InvalidParams("no default display is selected".to_string()))?;
    let (monitors, primary) = connected_displays(app)?;
    let names: Vec<_> = monitors.iter().map(|(name, _)| name.as_str()).collect();
    let displays = DisplayNames {
        names: &names,
        primary,
    };
    let target = displays
        .index(&default)
        .ok_or_else(|| Error::InvalidParams(format!("display `{default}` is not connected")))?;
    let board = store.load_board()?;
    // A broken widget has no window to move and would fail the whole write.
    let widgets: BTreeMap<_, _> = board
        .widgets
        .iter()
        .filter(|(id, _)| store.load_meta(id).is_ok())
        .map(|(id, frame)| (id.clone(), frame.clone()))
        .collect();
    let (frames, moved) = gathered_frames(
        &widgets,
        target,
        displays,
        logical_work_area(&monitors[target].1),
    );
    apply_board(app, &store.move_widgets(board.revision, frames)?);
    Ok(moved)
}
