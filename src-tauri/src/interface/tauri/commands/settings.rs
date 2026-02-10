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
