use crate::app::dtos::{AppendOperationInput, Operation};
use crate::app::ports::persistence_port::PersistencePort;
use crate::domain::errors::DomainError;

pub struct AppendOperationsUseCase;

impl AppendOperationsUseCase {
    pub fn execute(
        persistence: &dyn PersistencePort,
        inputs: &[AppendOperationInput],
    ) -> Result<Vec<Operation>, DomainError> {
        let mut operations = Vec::with_capacity(inputs.len());
        for input in inputs {
            let operation = persistence.insert_operation(input)?;
            operations.push(operation);
        }
        Ok(operations)
    }
}
