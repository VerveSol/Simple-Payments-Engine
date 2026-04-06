//! # Simple Payments Engine
//!
//! A streaming CSV transaction processor that maintains client accounts and
//! supports deposits, withdrawals, disputes, resolves, and chargebacks.
//!
//! Amounts are stored internally as **fixed-point `i64`** values where
//! `1 unit == 0.0001`, avoiding floating-point rounding issues while keeping
//! arithmetic fast and dependency-free.
//!
//! ## Crate layout
//!
//! - [`codec`] — CSV streaming reader / writer.
//! - [`engine`] — Transaction processing logic and account state management.
//! - [`models`] — Domain types (`Account`, `Transaction`, `StoredTransaction`).
//! - [`serde_helpers`] — Fixed-point amount serialization and deserialization.

use simple_payments_engine::{codec, engine};
use tracing::{info, warn};

use anyhow::anyhow;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("Usage: payments_engine <transactions.csv>"))?;

    let mut engine = engine::Engine::new();

    let mut processed = 0u32;
    let mut parse_errors = 0u32;

    for tx in codec::read_transactions(&path)? {
        match tx {
            Ok(tx) => {
                let tx_id = tx.transaction_id;
                let client_id = tx.client_id;
                if let Err(e) = engine.process(tx) {
                    warn!("tx={} client={} skipped: {}", tx_id, client_id, e);
                } else {
                    processed += 1;
                }
            }
            Err(e) => {
                warn!("Failed to parse row: {}", e);
                parse_errors += 1;
            }
        }
    }

    info!(
        "Done. Processed: {}, Parse Errors: {}",
        processed, parse_errors
    );

    codec::write_accounts(engine.get_accounts())?;

    Ok(())
}
