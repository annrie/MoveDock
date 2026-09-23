use crate::{
    cli,
    model::*,
    process,
    store::{self, State, Trusted},
};
use std::{fs, io::Write, path::Path, time::Duration};
use tauri::ipc::Channel;
use tokio::sync::mpsc;

#[tauri::command]
pub async fn get_data(state: tauri::State<'_, State>) -> Result<AppData, String> {
    Ok(state.data.lock().await.clone())
}

#[tauri::command]
pub async fn save_settings(
    state: tauri::State<'_, State>,
    settings: Settings,
) -> Result<(), String> {
    let active = state.active.lock().await;
    if active.is_some() {
        return Err(crate::messages::message("backend.settingsLocked"));
    }
    if !["bundler", "direct"].contains(&settings.mode.as_str())
        || !["system", "light", "dark"].contains(&settings.theme.as_str())
    {
        return Err(crate::messages::message("backend.invalidSettings"));
    }
    let mut data = state.data.lock().await;
    let mut next = data.clone();
    next.settings = settings;
    next.setup_notes.clear();
    state.save(&next)?;
    *data = next;
    state.trusted.lock().await.clear();
    Ok(())
}
#[tauri::command]
pub async fn autofill_settings(state: tauri::State<'_, State>) -> Result<AppData, String> {
    let active = state.active.lock().await;
    if active.is_some() {
        return Err(crate::messages::message("backend.settingsLocked"));
    }
    let mut data = state.data.lock().await;
    let mut next = data.clone();
    next.setup_notes = crate::discovery::detect(&mut next.settings);
    next.setup_complete = true;
    state.save(&next)?;
    *data = next.clone();
    state.trusted.lock().await.clear();
    Ok(next)
}
#[tauri::command]
pub async fn add_site(
    state: tauri::State<'_, State>,
    path: String,
    name: String,
) -> Result<Site, String> {
    let path = cli::expand(&path)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let path = path
        .to_str()
        .ok_or(crate::messages::message("backend.nonUtf8"))?
        .to_string();
    cli::config_name(&path)?;
    store::read_document(&path)?;
    let mut data = state.data.lock().await;
    if let Some(site) = data.sites.iter().find(|s| s.path == path) {
        return Ok(site.clone());
    }
    let site = Site {
        id: uuid::Uuid::new_v4().to_string(),
        name: if name.trim().is_empty() {
            "WordPress site".into()
        } else {
            name.trim().into()
        },
        path,
    };
    let mut next = data.clone();
    next.sites.push(site.clone());
    state.save(&next)?;
    *data = next;
    Ok(site)
}
#[tauri::command]
pub async fn remove_site(state: tauri::State<'_, State>, id: String) -> Result<(), String> {
    let active = state.active.lock().await;
    if active.is_some() {
        return Err(crate::messages::message("backend.removeLocked"));
    }
    let mut data = state.data.lock().await;
    let mut next = data.clone();
    next.sites.retain(|s| s.id != id);
    state.save(&next)?;
    *data = next;
    state.trusted.lock().await.remove(&id);
    Ok(())
}
#[tauri::command]
pub async fn read_movefile(state: tauri::State<'_, State>, id: String) -> Result<Document, String> {
    let site = store::site(&*state.data.lock().await, &id)?;
    store::read_document(&site.path)
}
#[tauri::command]
pub async fn save_movefile(
    state: tauri::State<'_, State>,
    id: String,
    content: String,
    revision: String,
) -> Result<Document, String> {
    let active = state.active.lock().await;
    if active.is_some() {
        return Err(crate::messages::message("backend.saveLocked"));
    }
    if content.len() > 1024 * 1024 {
        return Err(crate::messages::message("backend.fileTooLarge"));
    }
    let site = store::site(&*state.data.lock().await, &id)?;
    if store::read_document(&site.path)?.revision != revision {
        return Err(crate::messages::message("backend.externalChange"));
    }
    store::atomic_write(Path::new(&site.path), content.as_bytes())?;
    state.trusted.lock().await.remove(&id);
    store::read_document(&site.path)
}
#[tauri::command]
pub async fn create_movefile(path: String) -> Result<(), String> {
    let path = cli::expand(&path);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| crate::messages::with_detail("backend.createFailed", &e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    file.write_all(include_bytes!("../resources/movefile.yml"))
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn inspect_site(
    state: tauri::State<'_, State>,
    id: String,
) -> Result<Inspection, String> {
    // Serialize inspections with jobs/settings changes; loading ERB is an explicit user action.
    let active = state.active.lock().await;
    if active.is_some() {
        return Err(crate::messages::message("backend.running"));
    }
    let data = state.data.lock().await.clone();
    let site = store::site(&data, &id)?;
    let document = store::read_document(&site.path)?;
    let mut command = cli::base(&data.settings, Path::new(&site.path).parent().unwrap())?;
    command.args(["list", "--config", &cli::config_name(&site.path)?]);
    // A cold Bundler/Ruby startup can exceed 30 seconds on a busy Mac.
    let output = cli::capture(command, Duration::from_secs(90)).await?;
    let inspection = cli::parse_environments(&output)?;
    if store::read_document(&site.path)?.revision != document.revision {
        return Err(crate::messages::message("backend.changedWhileLoading"));
    }
    state.trusted.lock().await.insert(
        id,
        Trusted {
            revision: document.revision,
            settings: data.settings,
            inspection: inspection.clone(),
        },
    );
    Ok(inspection)
}
async fn prepare(
    state: &State,
    request: &RunRequest,
) -> Result<(Site, tokio::process::Command), String> {
    let data = state.data.lock().await.clone();
    let site = store::site(&data, &request.site_id)?;
    let trusted = state.trusted.lock().await;
    let loaded = trusted
        .get(&site.id)
        .ok_or(crate::messages::message("backend.loadFirst"))?;
    if loaded.settings != data.settings
        || loaded.revision != store::read_document(&site.path)?.revision
    {
        return Err(crate::messages::message("backend.reloadRequired"));
    }
    if !loaded
        .inspection
        .environments
        .iter()
        .any(|e| e.name == request.environment)
    {
        return Err(crate::messages::message("backend.environmentMissing"));
    }
    let mut command = cli::base(&data.settings, Path::new(&site.path).parent().unwrap())?;
    command.args(cli::arguments(request, &site)?);
    Ok((site, command))
}
#[tauri::command]
pub async fn preview_run(
    state: tauri::State<'_, State>,
    request: RunRequest,
) -> Result<String, String> {
    Ok(cli::display(&prepare(&state, &request).await?.1))
}
#[tauri::command]
pub async fn run_sync(
    state: tauri::State<'_, State>,
    request: RunRequest,
    output: Channel<LogEvent>,
) -> Result<History, String> {
    let mut active = state.active.lock().await;
    if active.is_some() {
        return Err(crate::messages::message("backend.busy"));
    }
    let (site, command) = prepare(&state, &request).await?;
    let (tx, rx) = mpsc::channel(1);
    *active = Some(tx);
    drop(active);
    let started_at = chrono::Utc::now().to_rfc3339();
    let result = process::execute(command, rx, Duration::from_secs(4 * 3600), |event| {
        let _ = output.send(event);
    })
    .await;
    let mut history = History {
        id: uuid::Uuid::new_v4().to_string(),
        site_name: site.name,
        started_at,
        finished_at: chrono::Utc::now().to_rfc3339(),
        request,
        status: "failed".into(),
        exit_code: None,
    };
    match &result {
        Ok(outcome) => {
            history.exit_code = outcome.code;
            history.status = if outcome.cancelled {
                "cancelled"
            } else if outcome.timed_out {
                "timeout"
            } else if outcome.code == Some(0) {
                "success"
            } else {
                "failed"
            }
            .into();
        }
        Err(error) => {
            let _ = output.send(LogEvent {
                stream: "system".into(),
                line: error.clone(),
            });
        }
    }
    let mut data = state.data.lock().await;
    data.history.insert(0, history.clone());
    data.history.truncate(200);
    let saved = state.save(&data);
    drop(data);
    *state.active.lock().await = None;
    if let Err(error) = saved {
        let _ = output.send(LogEvent {
            stream: "system".into(),
            line: crate::messages::with_cause("backend.historySave", &error),
        });
    }
    Ok(history)
}
#[tauri::command]
pub async fn cancel_run(state: tauri::State<'_, State>) -> Result<(), String> {
    if let Some(tx) = state.active.lock().await.as_ref() {
        let _ = tx.try_send(());
    }
    Ok(())
}
#[tauri::command]
pub async fn diagnostics(state: tauri::State<'_, State>) -> Result<Vec<Diagnostic>, String> {
    let settings = state.data.lock().await.settings.clone();
    let mut results = Vec::new();
    for name in [
        "ruby",
        "php",
        "bundle",
        "ssh",
        "rsync",
        "wp",
        "mysql",
        "mysqldump",
        "lftp",
    ] {
        let found = cli::resolve(name, &settings);
        results.push(Diagnostic {
            name: name.into(),
            available: found.is_ok(),
            detail: found
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|e| e),
        });
    }
    let version = match cli::base(&settings, &dirs::home_dir().unwrap_or_else(|| "/".into())) {
        Ok(mut command) => {
            command.arg("--version");
            cli::capture(command, Duration::from_secs(15)).await
        }
        Err(e) => Err(e),
    };
    results.insert(
        0,
        Diagnostic {
            name: "Wordmove".into(),
            available: version.is_ok(),
            detail: version
                .unwrap_or_else(|e| {
                    if settings.mode == "bundler" {
                        crate::messages::with_cause("backend.bundleHint", &e)
                    } else {
                        e
                    }
                })
                .trim()
                .into(),
        },
    );
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn requires_trust_and_rejects_changed_movefile_or_unknown_environment() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Movefile");
        fs::write(&path, "local: {}\n").unwrap();
        let state = State::open(dir.path().join("data")).unwrap();
        let site = Site {
            id: "site".into(),
            name: "test".into(),
            path: path.to_str().unwrap().into(),
        };
        let settings = Settings {
            mode: "direct".into(),
            executable: "/usr/bin/true".into(),
            ..Settings::default()
        };
        {
            let mut data = state.data.lock().await;
            data.sites.push(site.clone());
            data.settings = settings.clone();
        }
        let mut request = RunRequest {
            site_id: site.id.clone(),
            environment: "staging".into(),
            direction: "pull".into(),
            targets: vec!["themes".into()],
            simulate: true,
        };
        assert!(prepare(&state, &request).await.is_err());
        state.trusted.lock().await.insert(
            site.id.clone(),
            Trusted {
                revision: store::read_document(&site.path).unwrap().revision,
                settings,
                inspection: Inspection {
                    local: "http://local".into(),
                    environments: vec![Environment {
                        name: "staging".into(),
                        vhost: "https://stage".into(),
                    }],
                },
            },
        );
        assert!(prepare(&state, &request).await.is_ok());
        request.environment = "production".into();
        assert!(prepare(&state, &request).await.is_err());
        request.environment = "staging".into();
        fs::write(path, "local: {}\n# external edit").unwrap();
        assert!(prepare(&state, &request).await.is_err());
    }
}
