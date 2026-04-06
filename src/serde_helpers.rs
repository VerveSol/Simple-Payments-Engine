use serde::{ Deserialize, Deserializer, Serializer };
use anyhow::anyhow;

/// Deserializes an optional decimal string (e.g. `"1.5"`) into a
/// fixed-point `Option<i64>` where `1 unit == 0.0001`.
///
/// - At most 4 decimal places are accepted; more is an error.
/// - Negative values are rejected.
/// - A missing field deserializes to `None`.
pub(crate) fn deserialize_amount<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    let s : Option<String> = Deserialize::deserialize(d)?;
    if let Some(s) = s {
        let (s1, s2) = parse_amount_string_to_i64(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?;

        let amount = 
            s1.checked_mul(10_000)
                .ok_or_else(|| serde::de::Error::custom(format!("Amount overflow: {}", s)))?
                .checked_add(s2)
                    .ok_or_else(|| serde::de::Error::custom(format!("Amount overflow: {}", s)))?;

        Ok(Some(amount))
    } else {
        Ok(None)
    }
}

/// Parses a decimal string into its whole and fractional parts as `i64`s,
/// with the fractional part zero-padded to 4 digits.
fn parse_amount_string_to_i64(s: &str) -> Result<(i64, i64), anyhow::Error> {
    let (s1, s2) = s.split_once('.').unwrap_or((s, "0"));
    if s2.len() > 4 {
        return Err(anyhow!("Too many decimal places: {}", s));
    } 
    if s1.starts_with('-') {
        return Err(anyhow!("Negative amount: {}", s));
    }

    let s2 = format!("{:0<4}", s2); // Pad with zeros to ensure 4 decimal places.
    let s2: i64 = s2.parse().map_err(|_| anyhow!("Invalid amount: {}", s))?;
    let s1: i64 = s1.parse().map_err(|_| anyhow!("Invalid amount: {}", s))?;
    Ok((s1, s2))
}

/// Serializes a fixed-point `i64` amount back to a decimal string
/// (e.g. `10000` becomes `"1.0000"`).
///
/// Panics in debug mode (and returns an error in release) if the amount
/// is negative, since that should never occur.
pub(crate) fn serialize_amount<S: Serializer>(amount: &i64, s: S) -> Result<S::Ok, S::Error> {
    
    // Guard against potential upstream "bugs" that might produce negative amounts, which should never happen. 
    // This is a sanity check to catch such issues.
    debug_assert!(*amount >= 0, "Amount should be non-negative: {}", amount);
    if *amount < 0 {
        return Err(serde::ser::Error::custom(format!("Negative amount: {}", amount)));
    }

    let s1: i64 = *amount / 10_000;
    let s2: i64 = *amount % 10_000;

    let amount_str = format!("{}.{}", s1, format!("{:0>4}", s2));
    
    Ok(s.serialize_str(&amount_str)?)
}
