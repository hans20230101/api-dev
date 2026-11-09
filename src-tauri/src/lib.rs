mod commands;
mod db;
mod httpclient;
mod models;
mod repository;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当用户尝试启动第二个实例时，聚焦到已有窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            commands::list_collections,
            commands::create_collection,
            commands::update_collection,
            commands::delete_collection,
            commands::list_requests,
            commands::get_request,
            commands::save_request,
            commands::delete_request,
            commands::list_environments,
            commands::create_environment,
            commands::update_environment,
            commands::delete_environment,
            commands::set_active_environment,
            commands::get_active_environment,
            commands::list_variables,
            commands::save_variable,
            commands::delete_variable,
            commands::send_request,
            commands::save_response_body,
            commands::run_collection,
            commands::export_all,
            commands::export_collection,
            commands::write_export_file,
            commands::import_all,
        ])
        .setup(|app| {
            commands::setup(app)?;
            create_main_window(app)?;
            setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // 点击关闭按钮时隐藏到托盘，而不是退出进程
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 设置系统托盘：左键单击显示主窗口，右键菜单支持显示/退出
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&show, &separator, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("API 教研接口测试工具")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左键单击：显示并聚焦主窗口
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    // 优先用应用图标作为托盘图标
    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    Ok(())
}

/// 显示并聚焦主窗口
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// 创建主窗口。
/// 手动创建的原因：Tauri 在调用 setup 之前就会按配置创建窗口，而 Windows 上 WebView2
/// 必须使用可写的用户数据目录。若系统应用数据目录不可写（受限环境/只读配置），
/// 自动创建会直接失败并 panic。这里先解析出可写目录，再显式传入。
fn create_main_window(app: &tauri::App) -> tauri::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
        .or_else(|| app.config().app.windows.first().cloned());

    let webview_data_dir = resolve_webview_data_dir(app);

    let builder = match config {
        Some(cfg) => tauri::WebviewWindowBuilder::from_config(app.handle(), &cfg)?,
        None => tauri::WebviewWindowBuilder::new(
            app.handle(),
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("API 教研接口测试工具")
        .inner_size(1280.0, 820.0),
    };

    let builder = match webview_data_dir {
        Some(dir) => builder.data_directory(dir),
        None => builder,
    };

    builder.build()?;
    Ok(())
}

/// 解析 WebView2 用户数据目录，优先系统 LocalAppData，不可写时随数据库目录一起回退
fn resolve_webview_data_dir(app: &tauri::App) -> Option<std::path::PathBuf> {
    let preferred = app
        .path()
        .app_local_data_dir()
        .ok()
        .map(|d| d.join("webview-data"));

    if let Some(dir) = &preferred {
        if db::is_writable(dir) {
            return Some(dir.clone());
        }
    }

    let fallback = commands::data_dir(app).join("webview-data");
    if db::is_writable(&fallback) {
        Some(fallback)
    } else {
        preferred
    }
}
