use tauri::State;

use crate::app::dtos::{AppendOpsInput, Operation};
use crate::app::usecases::AppendOperationsUseCase;
use crate::domain::DomainError;
use crate::AppState;

#[tauri::command]
pub async fn append_ops(
    state: State<'_, AppState>,
    input: AppendOpsInput,
) -> Result<Vec<Operation>, DomainError> {
    let usecase = AppendOperationsUseCase::new(state.persistence.as_ref());
    usecase.execute(input.operations).await
}
