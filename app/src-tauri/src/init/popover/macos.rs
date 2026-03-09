use tauri::{AppHandle, Manager, PhysicalPosition, Runtime, WebviewWindow};

use super::PopoverState;

/// Minimum gap (logical px) between the popover bottom edge and the screen bottom edge.
/// Only used on macOS where the menu bar is always at the top.
pub(super) const BOTTOM_MARGIN: f64 = 8.0;

/// Sets an initial `max_height` from the primary monitor.
/// On macOS the menu bar sits at the top, so available height = monitor height - bottom margin.
pub(super) fn init_max_height<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    match window.primary_monitor() {
        Ok(Some(monitor)) => {
            let scale = monitor.scale_factor();
            let mon_size = monitor.size().to_logical::<f64>(scale);
            let max = mon_size.height - BOTTOM_MARGIN;
            tracing::trace!(
                "[init_max_height] scale={scale}, mon_h={}, max={max}",
                monitor.size().height
            );
            app.state::<PopoverState>().set_max_height(max);
        }
        Ok(None) => tracing::warn!(
            "[init_max_height] primary monitor not found, retaining fallback max_height"
        ),
        Err(e) => tracing::warn!(
            "[init_max_height] failed to get primary monitor ({e}), retaining fallback max_height"
        ),
    }
}

/// Updates `max_height` from the window's current outer position.
pub(super) fn refresh_max_height_from_position<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
) {
    match window.outer_position() {
        Ok(pos) => refresh_max_height(app, window, &pos),
        Err(e) => tracing::error!("Failed to get outer position: {e}"),
    }
}

/// Updates `max_height` based on the given window position and monitor geometry.
/// Assumes the tray is at the top (macOS); available height = monitor bottom - window top.
pub(super) fn refresh_max_height<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    position: &PhysicalPosition<i32>,
) {
    // monitor_from_point expects logical coordinates, so convert physical → logical
    // using the window's scale factor.
    let scale = window.scale_factor().unwrap_or_else(|e| {
        tracing::warn!("[refresh_max_height] failed to get window scale factor ({e}), using 1.0");
        1.0
    });
    let logical_pos = position.to_logical::<f64>(scale);

    let monitor = match window.monitor_from_point(logical_pos.x, logical_pos.y) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("[refresh_max_height] monitor_from_point failed: {e}");
            return;
        }
    };
    let Some(m) = monitor else {
        tracing::warn!(
            "[refresh_max_height] no monitor found at ({}, {})",
            logical_pos.x,
            logical_pos.y
        );
        return;
    };

    let mon_scale = m.scale_factor();
    let mon_pos = m.position().to_logical::<f64>(mon_scale);
    let mon_size = m.size().to_logical::<f64>(mon_scale);
    let monitor_bottom = mon_pos.y + mon_size.height;
    let window_top = position.to_logical::<f64>(mon_scale).y;
    let max = (monitor_bottom - window_top - BOTTOM_MARGIN).max(200.0);

    tracing::trace!(
        "[refresh_max_height] physical_y={}, scale={mon_scale}, window_top={window_top}, \
         mon_bottom={monitor_bottom}, max={max}",
        position.y,
    );

    app.state::<PopoverState>().set_max_height(max);
}

/// Calculates window placement coordinates directly below the tray icon.
///
/// Assumes macOS layout where the menu bar (and tray) is always at the top of the screen,
/// so the popup always appears below the tray within the visible area.
/// y clamping is intentionally omitted; on other OSes the tray may be at the bottom,
/// requiring a fundamentally different positioning strategy.
///
/// All arguments are in logical pixels (f64) to avoid sub-pixel rounding errors on
/// non-integer DPI scales (e.g. 1.25×, 1.5×).
pub(super) fn calculate_position(
    tray_x: f64,
    tray_y: f64,
    tray_width: f64,
    tray_height: f64,
    window_width: f64,
    monitor_bounds: Option<(f64, f64)>,
) -> (f64, f64) {
    let mut x = tray_x + tray_width / 2.0 - window_width / 2.0;
    let y = tray_y + tray_height + 4.0;

    if let Some((mon_left, mon_right)) = monitor_bounds {
        // Clamp right bound to at least mon_left so that x never goes left of the monitor
        // when the window is wider than the screen (mon_right - window_width < mon_left).
        x = x.max(mon_left).min((mon_right - window_width).max(mon_left));
    }

    (x, y)
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    #[test]
    fn centers_window_below_tray() {
        let (x, y) = calculate_position(100.0, 0.0, 24.0, 24.0, 320.0, None);
        // x = 100 + 12 - 160 = -48 (no clamping without a monitor)
        assert_eq!(x, -48.0);
        assert_eq!(y, 28.0); // 0 + 24 + 4
    }

    #[test]
    fn clamps_x_to_monitor_left() {
        let (x, _) = calculate_position(10.0, 0.0, 24.0, 24.0, 320.0, Some((0.0, 1440.0)));
        // unclamped: 10 + 12 - 160 = -138 → clamped to 0
        assert_eq!(x, 0.0);
    }

    #[test]
    fn clamps_x_to_monitor_right() {
        let (x, _) = calculate_position(1400.0, 0.0, 24.0, 24.0, 320.0, Some((0.0, 1440.0)));
        // unclamped: 1400 + 12 - 160 = 1252 → min(1252, 1440-320=1120) = 1120
        assert_eq!(x, 1120.0);
    }

    #[test]
    fn y_is_tray_bottom_plus_gap() {
        let (_, y) = calculate_position(500.0, 25.0, 24.0, 24.0, 320.0, None);
        assert_eq!(y, 53.0); // 25 + 24 + 4
    }

    #[test]
    fn handles_non_zero_monitor_left() {
        let (x, _) = calculate_position(1450.0, 0.0, 24.0, 24.0, 320.0, Some((1440.0, 2880.0)));
        // unclamped: 1450 + 12 - 160 = 1302 → clamp(1440, 2880-320=2560) = 1440
        assert_eq!(x, 1440.0);
    }

    #[test]
    fn clamps_x_to_monitor_left_when_window_wider_than_screen() {
        let (x, _) = calculate_position(100.0, 0.0, 24.0, 24.0, 640.0, Some((0.0, 320.0)));
        assert_eq!(x, 0.0);
    }

    #[test]
    fn preserves_subpixel_precision_on_fractional_scale() {
        let tray_x = 151.0_f64 / 1.5; // ≈ 100.6667
        let tray_w = 36.0_f64 / 1.5; // = 24.0
        let win_w = 480.0_f64 / 1.5; // = 320.0
        let (x, _) = calculate_position(tray_x, 0.0, tray_w, 24.0, win_w, None);
        assert!((x - (tray_x + tray_w / 2.0 - win_w / 2.0)).abs() < f64::EPSILON);
    }
}
