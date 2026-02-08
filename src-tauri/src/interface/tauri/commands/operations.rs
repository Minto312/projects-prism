use tauri::State;

use crate::app::dtos::{AppendOperationInput, Operation};
use crate::app::usecases::append_operations::AppendOperationsUseCase;
use crate::AppState;

#[tauri::command]
pub async fn append_ops(
    state: State<'_, AppState>,
    ops: Vec<AppendOperationInput>,
) -> Result<Vec<Operation>, String> {
    AppendOperationsUseCase::execute(state.persistence.as_ref(), &ops).map_err(|e| e.to_string())
}
