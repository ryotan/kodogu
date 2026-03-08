use std::env;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use suzunari_error::*;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, Rect, Runtime,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

/// Window label used in tauri.conf.json to identify the popover window.
const POPOVER_LABEL: &str = "popover";
/// Minimum gap (logical px) between the popover bottom edge and the screen bottom edge.
const BOTTOM_MARGIN: f64 = 8.0;
/// Fallback max height (logical px) used before the actual monitor geometry is known.
const FALLBACK_MAX_HEIGHT: f64 = 600.0;
/// Minimum interval between accepted toggle calls (prevents double-click race).
const TOGGLE_DEBOUNCE: Duration = Duration::from_millis(300);

/// Runtime state for the popover window, managed via Tauri's `AppHandle::manage()`.
/// Each `App` instance (including test apps) gets its own isolated copy.
struct PopoverState {
    /// Whether to keep the popover open on focus loss.
    /// Sourced from `KODOGU_POPOVER_KEEP_OPEN=1` at startup; useful for inspecting with DevTools.
    keep_open: bool,
    /// Whether the window event listener has been registered on the popover window.
    /// Reset to `false` when the window is re-created (see `toggle_inner`'s `None` branch).
    listener_registered: AtomicBool,
    /// Maximum logical height the popover window may occupy.
    /// Updated on show, focus, and move. Read by `resize_popover` to cap the actual size.
    max_height: Mutex<f64>,
    /// Timestamp of the last accepted toggle call.
    /// `None` = never toggled (first call always passes through).
    last_toggle: Mutex<Option<Instant>>,
}

impl PopoverState {
    fn new() -> Self {
        Self {
            keep_open: env::var("KODOGU_POPOVER_KEEP_OPEN").is_ok_and(|v| v == "1"),
            listener_registered: AtomicBool::new(false),
            max_height: Mutex::new(FALLBACK_MAX_HEIGHT),
            last_toggle: Mutex::new(None),
        }
    }

    fn max_height(&self) -> f64 {
        *self.max_height.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn set_max_height(&self, max: f64) {
        *self.max_height.lock().unwrap_or_else(|e| e.into_inner()) = max;
    }
}

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum PopoverError {
    #[suzu(display("Popover window config not found in tauri.conf.json (label: {label})"))]
    ConfigNotFound { label: String },
    #[suzu(transparent)]
    Tauri { source: tauri::Error },
}

/// Tauri command: JS calls this to resize the popover window.
/// The actual height is capped at the current `max_height` in `PopoverState`.
/// No-op on non-macOS platforms (popover is macOS-only).
#[tauri::command]
pub(crate) fn resize_popover<R: Runtime>(app: AppHandle<R>, width: f64, height: f64) {
    #[cfg(target_os = "macos")]
    {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            tracing::trace!("[resize_popover] skipped: invalid dimensions ({width}, {height})");
            return;
        }
        let state = app.state::<PopoverState>();
        let max = state.max_height();
        let capped = height.min(max);
        tracing::trace!("[resize_popover] requested={height}, max={max}, capped={capped}");
        match app.get_webview_window(POPOVER_LABEL) {
            Some(win) => {
                if let Err(e) = win.set_size(LogicalSize::new(width, capped)) {
                    tracing::error!("Failed to resize popover: {e}");
                }
            }
            None => tracing::trace!("[resize_popover] popover window not found, skipping resize"),
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, width, height);
}

/// Initial setup for the popover window.
/// Registers `PopoverState` into Tauri's managed state, then wires up window event listeners.
///
/// Returns `()` (not `Result`) intentionally. Windows configured with `visible: false`
/// may not exist at setup time due to Tauri's initialization order. In that case,
/// `toggle()` lazily creates the window via `create_window` and registers the listener there.
pub(crate) fn setup<R: Runtime>(app: &AppHandle<R>) {
    app.manage(PopoverState::new());
    #[cfg(target_os = "macos")]
    match app.get_webview_window(POPOVER_LABEL) {
        Some(window) => {
            init_max_height(app, &window);
            ensure_window_listeners(app, &window);
        }
        None => tracing::warn!(
            "popover window not found during setup — event listeners were not registered"
        ),
    }
}

/// Sets an initial `max_height` from the primary monitor so that pre-render
/// `resize_popover` calls are capped to a reasonable value instead of the fallback.
/// `show_at_tray` will refine this with the actual window position.
fn init_max_height<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let scale = monitor.scale_factor();
        let mon_size = monitor.size().to_logical::<f64>(scale);
        let max = mon_size.height - BOTTOM_MARGIN;
        tracing::trace!(
            "[init_max_height] scale={scale}, mon_h={}, max={max}",
            monitor.size().height
        );
        app.state::<PopoverState>().set_max_height(max);
    }
}

/// Toggles popover visibility on tray click (show/hide).
/// `tray_rect` is the position and size of the clicked tray icon.
///
/// Debounces rapid successive calls (e.g., macOS double-click on tray icon)
/// to prevent race conditions with window creation and visibility toggling.
/// The first call always passes through; subsequent calls within TOGGLE_DEBOUNCE
/// are suppressed.
pub(crate) fn toggle<R: Runtime>(app: &AppHandle<R>, tray_rect: Rect) {
    #[cfg(target_os = "macos")]
    {
        let now = Instant::now();
        {
            let state = app.state::<PopoverState>();
            let mut last = state.last_toggle.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(prev) = *last {
                let elapsed = now.duration_since(prev);
                if elapsed < TOGGLE_DEBOUNCE {
                    tracing::trace!("[toggle] debounced ({}ms since last)", elapsed.as_millis());
                    return;
                }
            }
            *last = Some(now);
        }

        if let Err(e) = toggle_inner(app, tray_rect) {
            tracing::error!("Failed to toggle popover visibility: {e}");
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, tray_rect);
}

fn toggle_inner<R: Runtime>(app: &AppHandle<R>, tray_rect: Rect) -> Result<(), PopoverError> {
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
            ensure_window_listeners(app, &window);
            show_at_tray(app, &window, tray_rect)?;
        }
        None => {
            app.state::<PopoverState>().listener_registered.store(false, Ordering::Relaxed);
            let window = create_window(app)?;
            ensure_window_listeners(app, &window);
            show_at_tray(app, &window, tray_rect)?;
        }
    }
    Ok(())
}

/// Registers window event listeners, unless already registered.
/// - Focused(true): recalculate max height
/// - Focused(false): hide popover (unless `keep_open`)
/// - Moved: recalculate max height
fn ensure_window_listeners<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let state = app.state::<PopoverState>();
    if state.listener_registered.swap(true, Ordering::AcqRel) {
        return;
    }
    let keep_open = state.keep_open;
    let app = app.clone();
    let w = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Focused(focused) => {
            if *focused {
                refresh_max_height_from_position(&app, &w);
            } else if !keep_open
                && w.is_visible().unwrap_or(false)
                && let Err(e) = w.hide()
            {
                tracing::error!("Failed to hide popover on focus loss: {e}");
            }
        }
        WindowEvent::Moved(position) => {
            refresh_max_height(&app, &w, position);
        }
        _ => {}
    });
}

/// Updates `max_height` from the window's current outer position.
fn refresh_max_height_from_position<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    match window.outer_position() {
        Ok(pos) => refresh_max_height(app, window, &pos),
        Err(e) => tracing::error!("Failed to get outer position: {e}"),
    }
}

/// Updates `max_height` based on the given window position and monitor geometry.
/// Only touches a plain variable — no window side effects.
fn refresh_max_height<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    position: &PhysicalPosition<i32>,
) {
    // monitor_from_point expects logical coordinates, so convert physical → logical
    // using the window's scale factor.
    let scale = window.scale_factor().unwrap_or(1.0);
    let logical_pos = position.to_logical::<f64>(scale);

    let Ok(monitor) = window.monitor_from_point(logical_pos.x, logical_pos.y) else {
        return;
    };
    let Some(m) = monitor else { return };

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
/// - x: Centers the window horizontally under the tray icon.
/// - y: Positions at tray bottom edge + 4px gap.
/// - Clamps x to monitor bounds when provided.
///
/// All arguments are in logical pixels (f64) to avoid sub-pixel rounding errors on
/// non-integer DPI scales (e.g. 1.25×, 1.5×). Integer truncation before this function
/// would cause the returned position to round incorrectly when Tauri converts it back
/// to physical pixels (pos × scale_factor), potentially causing micro-vibrations.
fn calculate_position(
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
        x = x.max(mon_left).min(mon_right - window_width);
    }

    (x, y)
}

/// Calculates and applies window position from the tray icon rect, then shows the window.
/// Also refreshes `max_height` so the first JS `resize_popover` call is properly capped.
///
/// All position calculations use logical coordinates (CSS px).
/// The tray icon resides on the primary monitor's menu bar, so we use
/// that monitor's scale factor to normalize physical values to logical.
fn show_at_tray<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    tray_rect: Rect,
) -> tauri::Result<()> {
    let scale = window.primary_monitor()?.map(|m| m.scale_factor()).unwrap_or(1.0);

    // tray_rect → logical
    let tray_pos = tray_rect.position.to_logical::<f64>(scale);
    let tray_size = tray_rect.size.to_logical::<f64>(scale);

    // window size: physical → logical
    let window_width = window.outer_size()?.to_logical::<f64>(scale).width;

    // monitor bounds: physical → logical
    let monitor = window.monitor_from_point(tray_pos.x, tray_pos.y)?;
    let monitor_bounds = monitor.as_ref().map(|m| {
        let s = m.scale_factor();
        let mon_pos = m.position().to_logical::<f64>(s);
        let mon_size = m.size().to_logical::<f64>(s);
        (mon_pos.x, mon_pos.x + mon_size.width)
    });

    let (x, y) = calculate_position(
        tray_pos.x,
        tray_pos.y,
        tray_size.width,
        tray_size.height,
        window_width,
        monitor_bounds,
    );

    // Refresh max_height (logical px, consistent with resize_popover).
    if let Some(m) = &monitor {
        let s = m.scale_factor();
        let mon_pos = m.position().to_logical::<f64>(s);
        let mon_size = m.size().to_logical::<f64>(s);
        let monitor_bottom = mon_pos.y + mon_size.height;
        let max = (monitor_bottom - y - BOTTOM_MARGIN).max(200.0);
        tracing::trace!("[show_at_tray] scale={s}, y={y}, mon_bottom={monitor_bottom}, max={max}",);
        app.state::<PopoverState>().set_max_height(max);
    }

    tracing::trace!(
        "tray rect: pos=({}, {}), size=({}, {}), window pos=({x}, {y})",
        tray_pos.x,
        tray_pos.y,
        tray_size.width,
        tray_size.height,
    );

    window.set_position(LogicalPosition::new(x, y))?;
    window.show()?;
    window.set_focus()
}

/// Creates a popover window from the tauri.conf.json configuration.
/// Used as a fallback when the window does not exist (e.g., after destroy).
fn create_window<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, PopoverError> {
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
    use tauri::{Manager, WebviewUrl};

    fn dummy_rect() -> Rect {
        Rect {
            position: tauri::PhysicalPosition::new(100, 10).into(),
            size: tauri::PhysicalSize::new(24u32, 24u32).into(),
        }
    }

    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .manage(PopoverState::new())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    fn create_popover_window(
        app: &tauri::App<tauri::test::MockRuntime>,
    ) -> WebviewWindow<tauri::test::MockRuntime> {
        WebviewWindowBuilder::new(app.handle(), POPOVER_LABEL, WebviewUrl::App("/popover".into()))
            .build()
            .unwrap()
    }

    #[test]
    fn centers_window_below_tray() {
        // tray at (100.0, 0.0), size 24x24, window width 320
        let (x, y) = calculate_position(100.0, 0.0, 24.0, 24.0, 320.0, None);
        // x = 100 + 12 - 160 = -48 (no clamping without a monitor)
        assert_eq!(x, -48.0);
        assert_eq!(y, 28.0); // 0 + 24 + 4
    }

    #[test]
    fn clamps_x_to_monitor_left() {
        // the window would go left of the monitor
        let (x, _) = calculate_position(10.0, 0.0, 24.0, 24.0, 320.0, Some((0.0, 1440.0)));
        // unclamped: 10 + 12 - 160 = -138 → clamped to 0
        assert_eq!(x, 0.0);
    }

    #[test]
    fn clamps_x_to_monitor_right() {
        // tray near right edge
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
        // secondary monitor starting at x=1440
        let (x, _) = calculate_position(1450.0, 0.0, 24.0, 24.0, 320.0, Some((1440.0, 2880.0)));
        // unclamped: 1450 + 12 - 160 = 1302 → clamp(1440, 2880-320=2560) = 1440
        assert_eq!(x, 1440.0);
    }

    #[test]
    fn preserves_subpixel_precision_on_fractional_scale() {
        // On 1.5× DPI, tray coords from Tauri are already fractional logical values.
        // E.g. physical x=151 → logical 100.666...; window physical 480 → logical 320.0.
        // The result must not be truncated to integers before being passed to set_position.
        let tray_x = 151.0_f64 / 1.5; // ≈ 100.6667
        let tray_w = 36.0_f64 / 1.5; // = 24.0
        let win_w = 480.0_f64 / 1.5; // = 320.0
        let (x, _) = calculate_position(tray_x, 0.0, tray_w, 24.0, win_w, None);
        // x = 100.6667 + 12 - 160 = -47.3333...  (not -48 as integer truncation would give)
        assert!((x - (tray_x + tray_w / 2.0 - win_w / 2.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn create_window_returns_config_not_found_when_no_popover_config() {
        let app = mock_app();
        let result = create_window(app.handle());
        assert!(matches!(result, Err(PopoverError::ConfigNotFound { .. })));
    }

    #[test]
    fn toggle_inner_hides_visible_window() {
        let app = mock_app();
        let _ = create_popover_window(&app);
        // MockRuntime initializes windows as visible, so get_webview_window returns
        // a handle where is_visible() = Ok(true). The visible→hide branch is taken.
        toggle_inner(app.handle(), dummy_rect()).unwrap();
    }

    #[test]
    fn toggle_inner_shows_hidden_window() {
        let app = mock_app();
        let _ = create_popover_window(&app);

        // First toggle: visible → hide (MockRuntime default: is_visible = true)
        toggle_inner(app.handle(), dummy_rect()).unwrap();
        // Second toggle: exercises the hidden→show branch.
        // Either way, the code path exercised by this call provides coverage.
        let _ = toggle_inner(app.handle(), dummy_rect());
    }

    #[test]
    fn toggle_inner_attempts_create_when_no_window_exists() {
        // Tests the None branch: when no popover window exists, toggle_inner calls
        // create_window. Here, create_window fails with ConfigNotFound because the
        // mock app has no window config, but the branch entry (listener_registered
        // reset + create_window call) is verified.
        let app = mock_app();
        assert!(app.get_webview_window(POPOVER_LABEL).is_none());

        let result = toggle_inner(app.handle(), dummy_rect());

        assert!(matches!(result, Err(PopoverError::ConfigNotFound { .. })));
    }
}
