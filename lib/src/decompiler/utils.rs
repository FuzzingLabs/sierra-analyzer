use num_bigint::BigInt;
use std::str;

/// Convert an integer to it's string value
/// Used to decode consts
pub fn decode_hex_bigint(bigint: &BigInt) -> Option<String> {
    // Convert the BigInt to a hexadecimal string
    let hex_string = format!("{:x}", bigint);

    // Decode the hexadecimal string to a byte vector
    let bytes = hex::decode(hex_string).ok()?;

    // Convert the byte vector to a string
    let string = str::from_utf8(&bytes).ok()?.to_string();

    Some(string)
}
