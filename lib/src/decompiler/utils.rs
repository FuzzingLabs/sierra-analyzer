use num_bigint::BigInt;
use std::str;

/// Convert an integer to it's string value or hex value
/// Used to decode consts
pub fn decode_hex_bigint(bigint: &BigInt) -> Option<String> {
    // Convert the BigInt to a hexadecimal string
    let hex_string = format!("{:x}", bigint);

    // Decode the hexadecimal string to a byte vector
    let bytes = hex::decode(hex_string.clone()).ok()?;

    // Convert the byte vector to a string or hex value
    let string = match str::from_utf8(&bytes) {
        Ok(s) => Some(s.to_string()),
        Err(_) => Some(format!("0x{hex_string}")),
    };

    string
}
