pub mod motions;
pub mod operation_pending;
// use motions::NormalAction;
use operation_pending::PendingOperations;

pub struct NormalMode {
    pub pending_operations: PendingOperations,
}
impl NormalMode {
    pub fn new() -> Self {
        let pending_operations = PendingOperations::new();
        Self { pending_operations }
    }
}
