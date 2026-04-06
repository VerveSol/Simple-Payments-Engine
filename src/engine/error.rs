use thiserror::Error;

/// Domain errors returned by [`Engine::process`](super::Engine::process).
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Account not found for client {0}")]
    AccountNotFound(u16),
    #[error("Insufficient funds to perform transaction {0} for client {1}")]
    InsufficientFunds(u32, u16),
    #[error("Cannot perform cross-client transactions for transaction {0}")]
    CrossClientTransaction(u32),
    #[error("Transaction {0} not found")]
    TransactionNotFound(u32),
    #[error("Duplicate transaction {0} for account {1}")]
    DuplicateTransaction(u32, u16),
    #[error("Account {0} is locked")]
    AccountLocked(u16),
    #[error("Unexpected transaction state for transaction {0}")]
    UnexpectedTransactionState(u32),
    #[error("Arithmetic error during transaction processing")]
    ArithmeticError,
    #[error("Invalid transaction kind for transaction {0}")]
    InvalidTransactionKind(u32),
    #[error("Missing amount for transaction {0}")]
    MissingAmount(u32),
}
