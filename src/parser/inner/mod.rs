//! Inner instruction decoders for vault transaction clear signing.
//! Phase 2 will add full decoding with amount/address extraction.
//! Currently provides instruction name identification only.

pub mod programs;
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

// --- Known DeFi program IDs (labeled but not fully decoded) ---

// BPFLoaderUpgradeab1e11111111111111111111111
pub const BPF_LOADER_UPGRADEABLE: [u8; 32] = [
    0x02, 0xa8, 0xf6, 0x91, 0x4e, 0x88, 0xa1, 0xb0, 0xe2, 0x10, 0x15, 0x3e, 0xf7, 0x63, 0xae,
    0x2b, 0x00, 0xc2, 0xb9, 0x3d, 0x16, 0xc1, 0x24, 0xd2, 0xc0, 0x53, 0x7a, 0x10, 0x04, 0x80,
    0x00, 0x00,
];

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (Token-2022 / Token Extensions)
pub const TOKEN_2022_PROGRAM: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd,
    0xda, 0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1,
    0x8b, 0xfc,
];

// JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 (Jupiter v6 Aggregator)
pub const JUPITER_V6_PROGRAM: [u8; 32] = [
    0x04, 0x79, 0xd5, 0x5b, 0xf2, 0x31, 0xc0, 0x6e, 0xee, 0x74, 0xc5, 0x6e, 0xce, 0x68, 0x15,
    0x07, 0xfd, 0xb1, 0xb2, 0xde, 0xa3, 0xf4, 0x8e, 0x51, 0x02, 0xb1, 0xcd, 0xa2, 0x56, 0xbc,
    0x13, 0x8f,
];

// MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD (Marinade Finance)
pub const MARINADE_PROGRAM: [u8; 32] = [
    0x05, 0x45, 0xe3, 0x65, 0xbe, 0xf2, 0x71, 0xad, 0x75, 0x35, 0x03, 0x67, 0x56, 0x5d, 0xa4,
    0x0d, 0xa3, 0x36, 0xdc, 0x1c, 0x87, 0x9b, 0xb1, 0x54, 0x8a, 0x7a, 0xfc, 0xc5, 0x5a, 0xa9,
    0x39, 0x1e,
];

// SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy (Jito Stake Pool)
pub const JITO_STAKE_POOL: [u8; 32] = [
    0x06, 0x81, 0x4e, 0xd4, 0xca, 0xf6, 0x8a, 0x17, 0x46, 0x72, 0xfd, 0xac, 0x86, 0x03, 0x1a,
    0x63, 0xe8, 0x4e, 0xa1, 0x5e, 0xfa, 0x1d, 0x44, 0xb7, 0x22, 0x93, 0xf6, 0xdb, 0xdb, 0x00,
    0x16, 0x50,
];

/// Describe an inner instruction from a vault transaction's inner message.
/// Uses the vault_tx module's zero-copy metadata.
pub fn describe_inner_instruction_from_vault(
    raw: &[u8],
    msg: &super::vault_tx::InnerMessageMeta,
    ix: &super::vault_tx::InnerInstructionMeta,
) -> ArrayString<64> {
    let mut desc = ArrayString::<64>::new();
    let program_id = match msg.program_id(raw, ix) {
        Some(id) => id,
        None => {
            let _ = desc.try_push_str("Unknown Program");
            return desc;
        }
    };
    let ix_data = msg.instruction_data(raw, ix);

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
    } else if *program_id == BPF_LOADER_UPGRADEABLE {
        let _ = desc.try_push_str(describe_bpf_loader(ix_data));
    } else if *program_id == TOKEN_2022_PROGRAM {
        // Token-2022 shares instruction layout with SPL Token
        let _ = desc.try_push_str(spl_token::describe(ix_data));
    } else {
        // Check program registry for a clean label
        let label = programs::program_label(program_id);
        let _ = desc.try_push_str(label);
    }
    desc
}

fn describe_bpf_loader(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "BPF Loader (unknown)";
    }
    let disc = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    match disc {
        0 => "Initialize Buffer",
        1 => "Write Buffer",
        2 => "Deploy Program",
        3 => "Program Upgrade",
        4 => "Set Upgrade Authority",
        5 => "Close Program",
        6 => "Extend Program",
        7 => "Set Authority Checked",
        _ => "BPF Loader (unknown)",
    }
}

/// Describe an inner instruction for display (from outer Solana message).
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
    } else if *program_id == SPL_TOKEN_PROGRAM || *program_id == TOKEN_2022_PROGRAM {
        let _ = desc.try_push_str(spl_token::describe(ix_data));
    } else if *program_id == ATA_PROGRAM {
        let _ = desc.try_push_str("Create Token Account");
    } else if *program_id == COMPUTE_BUDGET_PROGRAM {
        let _ = desc.try_push_str(describe_compute_budget(ix_data));
    } else if *program_id == MEMO_PROGRAM {
        let _ = desc.try_push_str("Memo");
    } else if *program_id == BPF_LOADER_UPGRADEABLE {
        let _ = desc.try_push_str(describe_bpf_loader(ix_data));
    } else {
        // Check program registry for a clean label
        let label = programs::program_label(program_id);
        let _ = desc.try_push_str(label);
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
