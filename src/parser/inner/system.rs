//! System program instruction decoder for clear signing display.
//! Extracts amounts and destination addresses from known instruction types.

use crate::display::address::format_base58;
use crate::display::amount::format_sol;
use arrayvec::ArrayString;
use core::fmt::Write;
use ledger_device_sdk::nbgl::Field;

/// Maximum number of display fields for a system instruction.
pub const MAX_FIELDS: usize = 4;

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

/// Extract the transfer amount from a System Transfer instruction (type 2).
/// Returns the lamports amount if this is a Transfer instruction.
pub fn extract_transfer_amount(data: &[u8]) -> Option<u64> {
    if data.len() < 12 {
        return None;
    }
    let instruction_type = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if instruction_type != 2 {
        return None;
    }
    Some(u64::from_le_bytes([
        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
    ]))
}

/// Extract the lamports from a CreateAccount instruction (type 0).
pub fn extract_create_account_lamports(data: &[u8]) -> Option<u64> {
    if data.len() < 12 {
        return None;
    }
    let instruction_type = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if instruction_type != 0 {
        return None;
    }
    Some(u64::from_le_bytes([
        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
    ]))
}
