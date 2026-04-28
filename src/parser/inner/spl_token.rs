//! SPL Token program instruction decoder.
//! Phase 2 will add full decoding with amount/address extraction.

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
