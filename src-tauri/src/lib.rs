mod cli;
mod commands;
mod discovery;
mod model;
mod process;
mod store;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state =
                store::State::open(app.path().app_data_dir()?).map_err(std::io::Error::other)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_data,
            commands::save_settings,
            commands::autofill_settings,
            commands::add_site,
            commands::remove_site,
            commands::read_movefile,
            commands::save_movefile,
            commands::create_movefile,
            commands::inspect_site,
            commands::preview_run,
            commands::run_sync,
            commands::cancel_run,
            commands::diagnostics
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<store::State>();
                // The UI provides a stop button. A running transfer cannot be orphaned by closing.
                if state
                    .active
                    .try_lock()
                    .map(|active| active.is_some())
                    .unwrap_or(true)
                {
                    api.prevent_close();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("MoveDock を起動できませんでした")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let state = app.state::<store::State>();
                if state
                    .active
                    .try_lock()
                    .map(|active| active.is_some())
                    .unwrap_or(true)
                {
                    api.prevent_exit();
                }
            }
        });
}
