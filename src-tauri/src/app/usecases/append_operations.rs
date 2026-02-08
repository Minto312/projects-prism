use crate::app::dtos::{
    NewOperation, Operation, OperationPayload, OperationStatus, OperationType, Precondition,
};
use crate::app::ports::PersistencePort;
use crate::domain::{DomainError, OperationId};

pub struct AppendOperationsUseCase<'a, P: PersistencePort> {
    persistence: &'a P,
}

impl<'a, P: PersistencePort> AppendOperationsUseCase<'a, P> {
    pub fn new(persistence: &'a P) -> Self {
        Self { persistence }
    }

    pub async fn execute(
        &self,
        new_operations: Vec<NewOperation>,
    ) -> Result<Vec<Operation>, DomainError> {
        let now = chrono::Utc::now().timestamp_millis();
        let mut created_operations = Vec::new();

        for new_op in new_operations {
            let operation = Operation {
                id: OperationId::generate(),
                op_type: OperationType::MoveItemToColumn,
                payload: OperationPayload {
                    item_id: new_op.item_id,
                    project_id: new_op.project_id,
                    status_field_id: new_op.status_field_id,
                    to_option_id: new_op.to_option_id,
                },
                precondition: Precondition {
                    base_item_updated_at: new_op.base_item_updated_at,
                    expected_from_option_id: new_op.expected_from_option_id,
                },
                created_at: now,
                status: OperationStatus::Pending,
                error_message: None,
                resolved_at: None,
            };

            self.persistence.insert_operation(&operation).await?;
            created_operations.push(operation);
        }

        Ok(created_operations)
    }
}
