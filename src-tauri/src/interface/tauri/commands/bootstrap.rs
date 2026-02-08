use tauri::State;

use crate::app::dtos::BootstrapResponse;
use crate::app::usecases::FetchBootstrapUseCase;
use crate::domain::DomainError;
use crate::AppState;

#[tauri::command]
pub async fn get_bootstrap(
    state: State<'_, AppState>,
    refresh: Option<bool>,
) -> Result<BootstrapResponse, DomainError> {
    let usecase = FetchBootstrapUseCase::new(
        state.persistence.as_ref(),
        state.github_client.as_ref(),
    );
    usecase.execute(refresh.unwrap_or(false)).await
}
