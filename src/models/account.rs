use crate::serde_helpers::serialize_amount;
use serde::Serialize;

/// A client account tracking available, held, and total balances.
///
/// All monetary fields use fixed-point representation (`1 unit == 0.0001`).
/// A locked account rejects every subsequent transaction.
#[derive(Debug, Serialize, Clone)]
pub struct Account {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(serialize_with = "serialize_amount")]
    pub available: i64, // Normalized to avoid rounding errors. Always positive.
    #[serde(serialize_with = "serialize_amount")]
    pub held: i64, // Normalized to avoid rounding errors. Always positive.
    #[serde(serialize_with = "serialize_amount")]
    pub total: i64, // Normalized to avoid rounding errors. Always positive.
    pub locked: bool,
}
