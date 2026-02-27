use std::sync::atomic::{AtomicBool, Ordering};

/// Set `KODOGU_POPOVER_KEEP_OPEN=1` to prevent the popover from hiding on focus loss.
/// Useful for inspecting the popover with DevTools.
static KEEP_OPEN: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("KODOGU_POPOVER_KEEP_OPEN").is_ok_and(|v| v == "1"));

use snafu::OptionExt;
use suzunari_error::*;
use tauri::{
    AppHandle, Manager, PhysicalPosition, Rect, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

const POPOVER_LABEL: &str = "popover";

/// Tracks whether the focus-loss listener has been registered on the popover window.
/// Reset to `false` when the window is re-created (see `toggle_inner`'s `None` branch).
static FOCUS_LISTENER_REGISTERED: AtomicBool = AtomicBool::new(false);

#[suzunari_error]
#[snafu(context(suffix(Error)))]
pub(crate) enum PopoverError {
    #[snafu(display("Popover window config not found in tauri.conf.json (label: {label})"))]
    ConfigNotFound { label: String },
    #[snafu(transparent)]
    Tauri { source: tauri::Error },
}

/// Initial setup for the popover window.
/// Registers a focus-loss listener that auto-hides the popover.
///
/// Returns `()` (not `Result`) intentionally. Windows configured with `visible: false`
/// may not exist at setup time due to Tauri's initialization order. In that case,
/// `toggle()` lazily creates the window via `create_window` and registers the listener there.
pub(crate) fn setup(app: &AppHandle) {
    match app.get_webview_window(POPOVER_LABEL) {
        Some(window) => ensure_focus_listener(&window),
        None => tracing::warn!(
            "popover window not found during setup — focus-loss listener was not registered"
        ),
    }
}

/// Toggles popover visibility on tray click (show/hide).
/// `tray_rect` is the position and size of the clicked tray icon.
pub(crate) fn toggle(app: &AppHandle, tray_rect: Rect) {
    if let Err(e) = toggle_inner(app, tray_rect) {
        tracing::error!("Failed to toggle popover visibility: {e}");
    }
}

fn toggle_inner(app: &AppHandle, tray_rect: Rect) -> Result<(), PopoverError> {
    // Tray icon events are dispatched on the main thread, so concurrent calls
    // to toggle_inner cannot interleave. The None branch's create_window would
    // also return Err if a same-label window already exists, providing an
    // additional safety net.
    match app.get_webview_window(POPOVER_LABEL) {
        Some(window)
            if window
                .is_visible()
                .inspect_err(|e| {
                    tracing::warn!("Failed to query popover visibility, assuming hidden: {e}");
                })
                .unwrap_or(false) =>
        {
            window.hide()?
        }
        Some(window) => {
            ensure_focus_listener(&window);
            show_at_tray(&window, tray_rect)?;
        }
        None => {
            FOCUS_LISTENER_REGISTERED.store(false, Ordering::Relaxed);
            let window = create_window(app)?;
            ensure_focus_listener(&window);
            show_at_tray(&window, tray_rect)?;
        }
    }
    Ok(())
}

/// Registers a focus-loss listener that hides the window, unless already registered.
fn ensure_focus_listener(window: &WebviewWindow) {
    if FOCUS_LISTENER_REGISTERED.swap(true, Ordering::Relaxed) {
        return;
    }
    let w = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event
            && !*KEEP_OPEN
            && let Err(e) = w.hide()
        {
            tracing::error!("Failed to hide popover on focus loss: {e}");
        }
    });
}

/// Calculates window placement coordinates directly below the tray icon.
///
/// - x: Centers the window horizontally under the tray icon.
/// - y: Positions at tray bottom edge + 4px gap.
/// - Clamps x to monitor bounds when provided.
fn calculate_position(
    tray_x: i32,
    tray_y: i32,
    tray_width: i32,
    tray_height: i32,
    window_width: i32,
    monitor_bounds: Option<(i32, i32)>,
) -> (i32, i32) {
    let mut x = tray_x + tray_width / 2 - window_width / 2;
    let y = tray_y + tray_height + 4;

    if let Some((mon_left, mon_right)) = monitor_bounds {
        x = x.max(mon_left).min(mon_right - window_width);
    }

    (x, y)
}

/// Calculates and applies window position from the tray icon rect, then shows the window.
fn show_at_tray(window: &WebviewWindow, tray_rect: Rect) -> tauri::Result<()> {
    // Tauri's tray_rect uses logical coordinates. We convert to physical with
    // scale_factor 1.0 because the popover window may not yet be positioned on
    // the target monitor, making its scale_factor unreliable.
    // HiDPI offset is tracked as Technical Debt #3.
    let tray_pos: PhysicalPosition<i32> = tray_rect.position.to_physical(1.0);
    let tray_size: tauri::PhysicalSize<i32> = tray_rect.size.to_physical(1.0);
    let window_width = window.outer_size()?.width as i32;

    let monitor_bounds =
        window.monitor_from_point(tray_pos.x as f64, tray_pos.y as f64)?.map(|m| {
            let left = m.position().x;
            let right = left + m.size().width as i32;
            (left, right)
        });

    let (x, y) = calculate_position(
        tray_pos.x,
        tray_pos.y,
        tray_size.width,
        tray_size.height,
        window_width,
        monitor_bounds,
    );

    tracing::debug!(
        "tray rect: pos=({}, {}), size=({}, {}), window pos=({x}, {y})",
        tray_pos.x,
        tray_pos.y,
        tray_size.width,
        tray_size.height,
    );

    window.set_position(PhysicalPosition::new(x, y))?;
    window.show()?;
    window.set_focus()
}

/// Creates a popover window from the tauri.conf.json configuration.
/// Used as a fallback when the window does not exist (e.g., after destroy).
fn create_window(app: &AppHandle) -> Result<WebviewWindow, PopoverError> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == POPOVER_LABEL)
        .context(ConfigNotFoundError { label: POPOVER_LABEL.to_string() })?;
    Ok(WebviewWindowBuilder::from_config(app, config)?.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_window_below_tray() {
        // tray at (100, 0), size 24x24, window width 320
        let (x, y) = calculate_position(100, 0, 24, 24, 320, None);
        // x = 100 + 12 - 160 = -48 (no clamping without monitor)
        assert_eq!(x, -48);
        assert_eq!(y, 28); // 0 + 24 + 4
    }

    #[test]
    fn clamps_x_to_monitor_left() {
        // window would go left of monitor
        let (x, _) = calculate_position(10, 0, 24, 24, 320, Some((0, 1440)));
        // unclamped: 10 + 12 - 160 = -138 → clamped to 0
        assert_eq!(x, 0);
    }

    #[test]
    fn clamps_x_to_monitor_right() {
        // tray near right edge
        let (x, _) = calculate_position(1400, 0, 24, 24, 320, Some((0, 1440)));
        // unclamped: 1400 + 12 - 160 = 1252 → min(1252, 1440-320=1120) = 1120
        assert_eq!(x, 1120);
    }

    #[test]
    fn y_is_tray_bottom_plus_gap() {
        let (_, y) = calculate_position(500, 25, 24, 24, 320, None);
        assert_eq!(y, 53); // 25 + 24 + 4
    }

    #[test]
    fn handles_non_zero_monitor_left() {
        // secondary monitor starting at x=1440
        let (x, _) = calculate_position(1450, 0, 24, 24, 320, Some((1440, 2880)));
        // unclamped: 1450 + 12 - 160 = 1302 → clamp(1440, 2880-320=2560) = 1440
        assert_eq!(x, 1440);
    }
}
