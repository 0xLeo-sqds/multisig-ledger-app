//! System program instruction decoder.
//! Phase 2 will add full decoding with amount/address extraction.

/// Describe a System program instruction from its data bytes.
pub fn describe(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "System (unknown)";
    }
    let instruction_type = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    match instruction_type {
        0 => "Create Account",
        1 => "Assign",
        2 => "SOL Transfer",
        3 => "Create Account With Seed",
        4 => "Advance Nonce Account",
        5 => "Withdraw Nonce Account",
        6 => "Initialize Nonce Account",
        7 => "Authorize Nonce Account",
        8 => "Allocate",
        9 => "Allocate With Seed",
        10 => "Assign With Seed",
        11 => "Transfer With Seed",
        12 => "Upgrade Nonce Account",
        _ => "System (unknown)",
    }
}
