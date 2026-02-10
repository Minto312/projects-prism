use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::UpdaterExt;
use url::Url;

use crate::app::ports::persistence_port::PersistencePort;
use crate::AppState;

const STABLE_ENDPOINT: &str =
    "https://github.com/Minto312/projects-prism/releases/latest/download/latest.json";
const NIGHTLY_ENDPOINT: &str =
    "https://github.com/Minto312/projects-prism/releases/download/nightly/latest.json";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgressPayload {
    pub chunk_length: u64,
    pub content_length: Option<u64>,
}

fn endpoint_for_channel(channel: &str) -> Url {
    let raw = match channel {
        "nightly" => NIGHTLY_ENDPOINT,
        _ => STABLE_ENDPOINT,
    };
    Url::parse(raw).expect("hardcoded URL is valid")
}

fn resolve_channel(state: &AppState) -> Result<String, String> {
    state
        .persistence
        .get_setting("update_channel")
        .map_err(|e| e.to_string())
        .map(|v| v.unwrap_or_else(|| "stable".to_string()))
}

#[tauri::command]
pub async fn get_update_channel(state: State<'_, AppState>) -> Result<String, String> {
    resolve_channel(&state)
}

#[tauri::command]
pub async fn set_update_channel(
    state: State<'_, AppState>,
    channel: String,
) -> Result<(), String> {
    if channel != "stable" && channel != "nightly" {
        return Err("Invalid channel. Must be \"stable\" or \"nightly\".".to_string());
    }
    state
        .persistence
        .set_setting("update_channel", &channel)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<UpdateInfo>, String> {
    let channel = resolve_channel(&state)?;
    let endpoint = endpoint_for_channel(&channel);

    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let update = updater.check().await.map_err(|e| e.to_string())?;

    match update {
        Some(u) => Ok(Some(UpdateInfo {
            version: u.version.clone(),
            body: u.body.clone(),
        })),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn download_and_install_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let channel = resolve_channel(&state)?;
    let endpoint = endpoint_for_channel(&channel);

    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let update = updater
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No update available".to_string())?;

    let app_handle = app.clone();
    update
        .download_and_install(
            move |chunk_length: usize, content_length: Option<u64>| {
                let _ = app_handle.emit(
                    "update-progress",
                    UpdateProgressPayload {
                        chunk_length: chunk_length as u64,
                        content_length,
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
