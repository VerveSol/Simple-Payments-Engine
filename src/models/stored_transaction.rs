/// Lifecycle state of a stored transaction (state machine).
///
/// ```text
/// Normal -> Disputed -> Resolved  (returns to Normal-like state)
///                    -> Chargebacked (terminal — account frozen)
/// ```
#[derive(Debug, PartialEq)]
pub enum TransactionState {
    /// Default state after a successful deposit or withdrawal.
    Normal,
    /// A dispute has been filed; funds are held.
    Disputed,
    /// The dispute was resolved in the client's favour; funds released.
    Resolved,
    /// The dispute resulted in a chargeback; funds removed, account frozen.
    Chargebacked,
}

/// Distinguishes the two transaction kinds that are persisted in the store.
#[derive(Debug, PartialEq)]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
}

/// A transaction record kept in-memory after processing.
///
/// Only deposits and withdrawals are stored.
#[derive(Debug)]
pub struct StoredTransaction {
    pub client_id: u16, // Exists to validate cross-client disputes attempts.
    pub amount: i64,    // Normalized to avoid rounding errors. Always positive.
    pub state: TransactionState,
    pub kind: TransactionKind,
}
