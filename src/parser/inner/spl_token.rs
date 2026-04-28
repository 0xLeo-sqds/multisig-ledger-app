//! SPL Token program instruction decoder for clear signing display.
//! Extracts amounts from Transfer, TransferChecked, and other instructions.

/// Describe an SPL Token instruction from its data bytes.
pub fn describe(data: &[u8]) -> &'static str {
    if data.is_empty() {
        return "Token (unknown)";
    }
    match data[0] {
        0 => "Initialize Mint",
        1 => "Initialize Account",
        2 => "Initialize Multisig",
        3 => "Token Transfer",
        4 => "Token Approve",
        5 => "Token Revoke",
        6 => "Set Authority",
        7 => "Mint To",
        8 => "Token Burn",
        9 => "Close Account",
        10 => "Freeze Account",
        11 => "Thaw Account",
        12 => "Transfer Checked",
        13 => "Approve Checked",
        14 => "Mint To Checked",
        15 => "Burn Checked",
        16 => "Initialize Account 2",
        17 => "Sync Native",
        18 => "Initialize Account 3",
        _ => "Token (unknown)",
    }
}

/// Extract the amount from a Token Transfer instruction (type 3).
/// Layout: [disc(1)] [amount(u64)]
pub fn extract_transfer_amount(data: &[u8]) -> Option<u64> {
    if data.len() < 9 || data[0] != 3 {
        return None;
    }
    Some(u64::from_le_bytes([
        data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
    ]))
}

/// Extract amount and decimals from a TransferChecked instruction (type 12).
/// Layout: [disc(1)] [amount(u64)] [decimals(u8)]
pub fn extract_transfer_checked(data: &[u8]) -> Option<(u64, u8)> {
    if data.len() < 10 || data[0] != 12 {
        return None;
    }
    let amount = u64::from_le_bytes([
        data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
    ]);
    let decimals = data[9];
    Some((amount, decimals))
}
