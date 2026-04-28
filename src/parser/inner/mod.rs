//! Inner instruction decoders for vault transaction clear signing.
//! Phase 2 will add full decoding with amount/address extraction.
//! Currently provides instruction name identification only.

pub mod spl_token;
pub mod system;

use super::solana_message::{InstructionMeta, ParsedMessage};
use arrayvec::ArrayString;

/// Known Solana program IDs as raw bytes for fast comparison (no base58).
/// Verified by decoding the canonical base58 addresses.

// 11111111111111111111111111111111
pub const SYSTEM_PROGRAM: [u8; 32] = [0u8; 32];

// TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
pub const SPL_TOKEN_PROGRAM: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xd7, 0x65, 0xa1, 0x93, 0xd9, 0xcb, 0xe1, 0x46, 0xce, 0xeb, 0x79,
    0xac, 0x1c, 0xb4, 0x85, 0xed, 0x5f, 0x5b, 0x37, 0x91, 0x3a, 0x8c, 0xf5, 0x85, 0x7e, 0xff,
    0x00, 0xa9,
];

// ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
pub const ATA_PROGRAM: [u8; 32] = [
    0x8c, 0x97, 0x25, 0x8f, 0x4e, 0x24, 0x89, 0xf1, 0xbb, 0x3d, 0x10, 0x29, 0x14, 0x8e, 0x0d,
    0x83, 0x0b, 0x5a, 0x13, 0x99, 0xda, 0xff, 0x10, 0x84, 0x04, 0x8e, 0x7b, 0xd8, 0xdb, 0xe9,
    0xf8, 0x59,
];

// ComputeBudget111111111111111111111111111111
pub const COMPUTE_BUDGET_PROGRAM: [u8; 32] = [
    0x03, 0x06, 0x46, 0x6f, 0xe5, 0x21, 0x17, 0x32, 0xff, 0xec, 0xad, 0xba, 0x72, 0xc3, 0x9b,
    0xe7, 0xbc, 0x8c, 0xe5, 0xbb, 0xc5, 0xf7, 0x12, 0x6b, 0x2c, 0x43, 0x9b, 0x3a, 0x40, 0x00,
    0x00, 0x00,
];

// MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr
pub const MEMO_PROGRAM: [u8; 32] = [
    0x05, 0x4a, 0x53, 0x5a, 0x99, 0x29, 0x21, 0x06, 0x4d, 0x24, 0xe8, 0x71, 0x60, 0xda, 0x38,
    0x7c, 0x7c, 0x35, 0xb5, 0xdd, 0xbc, 0x92, 0xbb, 0x81, 0xe4, 0x1f, 0xa8, 0x40, 0x41, 0x05,
    0x44, 0x8d,
];

/// Describe an inner instruction for display.
/// Returns a human-readable summary like "SOL Transfer" or "Unknown Program".
pub fn describe_inner_instruction(msg: &ParsedMessage<'_>, ix: &InstructionMeta) -> ArrayString<64> {
    let mut desc = ArrayString::<64>::new();

    let program_id = match msg.program_id(ix) {
        Some(id) => id,
        None => {
            let _ = desc.try_push_str("Unknown Program");
            return desc;
        }
    };

    let ix_data = msg.instruction_data(ix);

    if *program_id == SYSTEM_PROGRAM {
        let _ = desc.try_push_str(system::describe(ix_data));
    } else if *program_id == SPL_TOKEN_PROGRAM {
        let _ = desc.try_push_str(spl_token::describe(ix_data));
    } else if *program_id == ATA_PROGRAM {
        let _ = desc.try_push_str("Create Token Account");
    } else if *program_id == COMPUTE_BUDGET_PROGRAM {
        let _ = desc.try_push_str(describe_compute_budget(ix_data));
    } else if *program_id == MEMO_PROGRAM {
        let _ = desc.try_push_str("Memo");
    } else {
        let _ = desc.try_push_str("Unknown Program");
    }

    desc
}

fn describe_compute_budget(data: &[u8]) -> &'static str {
    if data.is_empty() {
        return "Compute Budget (unknown)";
    }
    match data[0] {
        2 => "Set Compute Unit Limit",
        3 => "Set Compute Unit Price",
        _ => "Compute Budget (unknown)",
    }
}
