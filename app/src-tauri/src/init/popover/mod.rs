use std::env;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use suzunari_error::*;
use tauri::{
    AppHandle, LogicalSize, Manager, Rect, Runtime, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

#[cfg(target_os = "macos")]
mod macos;

/// Window label used in tauri.conf.json to identify the popover window.
const POPOVER_LABEL: &str = "popover";
/// Minimum interval between accepted toggle calls (prevents double-click race).
const TOGGLE_DEBOUNCE: Duration = Duration::from_millis(300);
/// Fallback max height (logical px) used on platforms where dynamic max_height is not implemented.
const FALLBACK_MAX_HEIGHT: f64 = 600.0;

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
    /// Updated on macOS by monitoring window position and monitor geometry.
    /// On other platforms, stays at FALLBACK_MAX_HEIGHT.
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

    #[cfg(target_os = "macos")]
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
#[tauri::command]
pub(crate) fn resize_popover<R: Runtime>(app: AppHandle<R>, width: f64, height: f64) {
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

/// Initial setup for the popover window.
/// Registers `PopoverState` into Tauri's managed state, then wires up window event listeners.
pub(crate) fn setup<R: Runtime>(app: &AppHandle<R>) {
    app.manage(PopoverState::new());
    match app.get_webview_window(POPOVER_LABEL) {
        Some(window) => {
            #[cfg(target_os = "macos")]
            macos::init_max_height(app, &window);
            ensure_window_listeners(app, &window);
        }
        None => tracing::warn!(
            "popover window not found during setup — event listeners were not registered"
        ),
    }
}

/// Toggles popover visibility on tray click (show/hide).
/// `tray_rect` is the position and size of the clicked tray icon.
///
/// Debounces rapid successive calls (e.g., macOS double-click on tray icon)
/// to prevent race conditions with window creation and visibility toggling.
pub(crate) fn toggle<R: Runtime>(app: &AppHandle<R>, tray_rect: Rect) {
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
/// - Focused(true): recalculate max height (macOS only)
/// - Focused(false): hide popover (unless `keep_open`)
/// - Moved: recalculate max height (macOS only)
fn ensure_window_listeners<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let state = app.state::<PopoverState>();
    if state.listener_registered.swap(true, Ordering::AcqRel) {
        return;
    }
    let keep_open = state.keep_open;
    #[cfg(target_os = "macos")]
    let app = app.clone();
    let w = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Focused(focused) => {
            if *focused {
                #[cfg(target_os = "macos")]
                macos::refresh_max_height_from_position(&app, &w);
            } else if !keep_open
                && w.is_visible().unwrap_or(false)
                && let Err(e) = w.hide()
            {
                tracing::error!("Failed to hide popover on focus loss: {e}");
            }
        }
        WindowEvent::Moved(position) => {
            #[cfg(target_os = "macos")]
            macos::refresh_max_height(&app, &w, position);
            #[cfg(not(target_os = "macos"))]
            let _ = position;
        }
        _ => {}
    });
}

/// Positions the window near the tray icon rect, then shows it.
/// On macOS, positions below the menu bar tray icon.
/// On other platforms, it shows the window at its current position.
fn show_at_tray<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    tray_rect: Rect,
) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use tauri::LogicalPosition;

        let primary_scale = match window.primary_monitor()? {
            Some(m) => m.scale_factor(),
            None => {
                tracing::warn!(
                    "[show_at_tray] primary monitor not found, using scale=1.0 for tray position approximation"
                );
                1.0
            }
        };
        let tray_pos_approx = tray_rect.position.to_logical::<f64>(primary_scale);
        let monitor = window.monitor_from_point(tray_pos_approx.x, tray_pos_approx.y)?;

        let scale = match &monitor {
            Some(m) => {
                let s = m.scale_factor();
                if (s - primary_scale).abs() > f64::EPSILON {
                    tracing::debug!(
                        "[show_at_tray] tray monitor scale={s} differs from primary scale={primary_scale}"
                    );
                }
                s
            }
            None => {
                tracing::warn!(
                    "[show_at_tray] no monitor found at tray position ({}, {}), \
                     using primary scale={primary_scale}",
                    tray_pos_approx.x,
                    tray_pos_approx.y
                );
                primary_scale
            }
        };

        let tray_pos = tray_rect.position.to_logical::<f64>(scale);
        let tray_size = tray_rect.size.to_logical::<f64>(scale);
        let window_width = window.outer_size()?.to_logical::<f64>(scale).width;

        let monitor_bounds = monitor.as_ref().map(|m| {
            let s = m.scale_factor();
            let mon_pos = m.position().to_logical::<f64>(s);
            let mon_size = m.size().to_logical::<f64>(s);
            (mon_pos.x, mon_pos.x + mon_size.width)
        });

        let (x, y) = macos::calculate_position(
            tray_pos.x,
            tray_pos.y,
            tray_size.width,
            tray_size.height,
            window_width,
            monitor_bounds,
        );

        if let Some(m) = &monitor {
            let s = m.scale_factor();
            let mon_pos = m.position().to_logical::<f64>(s);
            let mon_size = m.size().to_logical::<f64>(s);
            let monitor_bottom = mon_pos.y + mon_size.height;
            let max = (monitor_bottom - y - macos::BOTTOM_MARGIN).max(200.0);
            tracing::trace!(
                "[show_at_tray] scale={s}, y={y}, mon_bottom={monitor_bottom}, max={max}"
            );
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
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (app, tray_rect);

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

// Showing, hiding, and resizing are not tested: `MockRuntime` windows always report visible and ignore `show`, `hide`,
// and `set_size`.
#[cfg(test)]
mod tests {
    use super::*;
    use tauri::Manager;

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

    #[test]
    fn create_window_returns_config_not_found_when_no_popover_config() {
        let app = mock_app();
        let result = create_window(app.handle());
        assert!(matches!(result, Err(PopoverError::ConfigNotFound { .. })));
    }

    #[test]
    fn toggle_inner_attempts_create_when_no_window_exists() {
        let app = mock_app();
        assert!(app.get_webview_window(POPOVER_LABEL).is_none());
        let result = toggle_inner(app.handle(), dummy_rect());
        assert!(matches!(result, Err(PopoverError::ConfigNotFound { .. })));
    }

    #[test]
    fn toggle_debounces_rapid_calls() {
        let app = mock_app();
        let state = app.state::<PopoverState>();
        let last_toggle = || *state.last_toggle.lock().unwrap();

        toggle(app.handle(), dummy_rect());
        let accepted = last_toggle();
        toggle(app.handle(), dummy_rect());

        assert!(accepted.is_some());
        assert_eq!(last_toggle(), accepted);
    }
}
