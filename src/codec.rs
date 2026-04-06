use crate::models::{account::Account, transaction::Transaction};
use anyhow::Result;
use anyhow::anyhow;
use csv::Trim::All;
use std::fs::File;
use std::io::BufReader;

/// Reads transactions from a CSV file at `path`, returning a streaming
/// iterator that deserializes one [`Transaction`] per row.
///
/// Whitespace is trimmed from every field. Rows that fail to parse are
/// surfaced as `Err` items so the caller can log and skip them.
pub fn read_transactions(path: &str) -> Result<impl Iterator<Item = Result<Transaction>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let csv_reader = csv::ReaderBuilder::new().trim(All).from_reader(reader);

    Ok(csv_reader
        .into_deserialize()
        .map(|res| res.map_err(|e| anyhow!("CSV deserialization error: {}", e))))
}

/// Serializes an iterator of [`Account`]s as CSV to **stdout**.
pub fn write_accounts<'a>(
    accounts: impl Iterator<Item = &'a Account>,
) -> Result<(), anyhow::Error> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for account in accounts {
        wtr.serialize(account)?;
    }
    wtr.flush()?;

    Ok(())
}
