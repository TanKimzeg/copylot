use log;
use tauri::Manager;

const TRANSLATOR_WINDOW_LABEL: &str = "translator";

pub fn ensure_translator_window(
    app_handle: &tauri::AppHandle,
) -> Result<tauri::WebviewWindow, tauri::Error> {
    if let Some(w) = app_handle.get_webview_window(TRANSLATOR_WINDOW_LABEL) {
        return Ok(w);
    }

    let w = tauri::WebviewWindowBuilder::new(
        app_handle,
        TRANSLATOR_WINDOW_LABEL,
        tauri::WebviewUrl::App("translator.html".into()),
    )
    .title("copylot")
    .decorations(false)
    // 调试：先关闭 transparent，排除“窗口存在但完全透明看不见”
    .transparent(false)
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true)
    .visible(false)
    .build()?;

    // 失焦即隐藏（点击外部隐藏）
    {
        let w2 = w.clone();
        w.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = *event {
                let _ = w2.hide();
            }
        });
    }

    Ok(w)
}

struct DesktopPopupWindow {
    pos: tauri::LogicalPosition<f64>,
    size: tauri::LogicalSize<f64>,
}

impl DesktopPopupWindow {
    fn right_side_from_monitor_and_cursor(
        monitor: &tauri::Monitor,
        cursor: &tauri::PhysicalPosition<f64>,
    ) -> Self {
        let scale = monitor.scale_factor();
        let screen_pos = monitor.position();
        let screen_size = monitor.size();
        let margin = 12.0 * scale;

        let w_physical: f64 = 420.0 * scale;
        let h_physical: f64 = ((screen_size.height as f64) * 0.30).max(300.0 * scale);

        let left_limit: f64 = screen_pos.x as f64 + margin;
        let right_limit: f64 = screen_pos.x as f64 + screen_size.width as f64 - w_physical - margin;

        let mut x_physical: f64 = cursor.x + margin;
        if x_physical > right_limit {
            x_physical = cursor.x - w_physical - margin;
        }
        x_physical = x_physical.max(left_limit).min(right_limit);

        let top_limit: f64 = screen_pos.y as f64 + margin;
        let bottom_limit: f64 =
            screen_pos.y as f64 + screen_size.height as f64 - h_physical - margin;

        let mut y_physical: f64 = cursor.y - h_physical / 2.0;
        y_physical = y_physical.max(top_limit).min(bottom_limit);

        Self {
            pos: tauri::LogicalPosition {
                x: x_physical / scale,
                y: y_physical / scale,
            },
            size: tauri::LogicalSize {
                width: w_physical / scale,
                height: h_physical / scale,
            },
        }
    }
}

/// 在鼠标右侧显示翻译小窗；放不下则翻到左侧；
/// 优先使用鼠标所在显示器，避免多屏幕下把窗口放到屏幕外
pub fn show_translator_right_side(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
) -> Result<(), tauri::Error> {
    // cursor_position 返回 PhysicalPosition（相对虚拟桌面坐标系）
    let cursor_physical = app_handle
        .cursor_position()
        .unwrap_or(tauri::PhysicalPosition { x: 0.0, y: 0.0 });

    // 优先使用“鼠标所在显示器”，避免 primary_monitor 在多屏下把窗口放到屏幕外
    let monitor = app_handle
        .monitor_from_point(cursor_physical.x, cursor_physical.y)?
        .or(app_handle.primary_monitor()?);

    let Some(monitor) = monitor else {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    };

    let popup = DesktopPopupWindow::right_side_from_monitor_and_cursor(&monitor, &cursor_physical);

    window.set_size(tauri::Size::Logical(popup.size))?;
    window.set_position(tauri::Position::Logical(popup.pos))?;

    window.show()?;
    window.set_always_on_top(true)?;
    window.set_focus()?;

    if let Ok(p) = window.outer_position() {
        log::trace!("translator outer_position: {p:?}");
    }
    if let Ok(s) = window.outer_size() {
        log::trace!("translator outer_size: {s:?}");
    }

    Ok(())
}
