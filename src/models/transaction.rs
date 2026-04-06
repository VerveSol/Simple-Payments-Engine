use crate::serde_helpers::deserialize_amount;
use serde::Deserialize;

/// The transaction types supported by the engine.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents a transaction read from the CSV file.
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    #[serde(deserialize_with = "deserialize_amount")]
    pub amount: Option<i64>, // Only some for Deposit and Withdrawal. Normalized to avoid rounding errors. Always positive.
}
