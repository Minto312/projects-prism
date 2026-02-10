use tauri::State;

use crate::app::ports::persistence_port::PersistencePort;
use crate::AppState;

#[tauri::command]
pub async fn set_pat(state: State<'_, AppState>, pat: String) -> Result<(), String> {
    log::info!("set_pat: PATを設定");
    state
        .persistence
        .set_setting("github_pat", &pat)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn has_pat(state: State<'_, AppState>) -> Result<bool, String> {
    let pat = state
        .persistence
        .get_setting("github_pat")
        .map_err(|e| e.to_string())?;
    log::debug!("has_pat: {}", pat.is_some());
    Ok(pat.is_some())
}

#[tauri::command]
pub async fn clear_pat(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("clear_pat: PATを削除");
    state
        .persistence
        .delete_setting("github_pat")
        .map_err(|e| e.to_string())?;
    state
        .persistence
        .delete_setting("current_user_login")
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_hidden_project_ids(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let value = state
        .persistence
        .get_setting("hidden_project_ids")
        .map_err(|e| e.to_string())?;
    match value {
        Some(json_str) => {
            serde_json::from_str::<Vec<String>>(&json_str).map_err(|e| e.to_string())
        }
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn set_hidden_project_ids(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<(), String> {
    let json_str = serde_json::to_string(&ids).map_err(|e| e.to_string())?;
    state
        .persistence
        .set_setting("hidden_project_ids", &json_str)
        .map_err(|e| e.to_string())
}
