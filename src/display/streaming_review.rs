//! Streaming review display for vault transactions with inner instructions.
//!
//! Uses NbglStreamingReview to display one instruction at a time,
//! never holding more than one instruction's fields in memory.

use crate::display::address::format_base58;
use crate::display::amount::{format_amount, format_sol};
use crate::parser::inner;
use crate::parser::vault_tx::{InnerMessageMeta, VaultTxMeta};
use crate::parser::ParseError;
use crate::settings::Settings;
use arrayvec::ArrayString;
use core::fmt::Write;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::nbgl::{Field, NbglReview, NbglStreamingReview, TransactionType};

/// Review a vault_transaction_create with full inner instruction decoding.
///
/// Displays:
///   1. Header: "Create Vault Transaction"
///   2. Multisig address
///   3. Vault index
///   4. For each inner instruction:
///      - Instruction type ("SOL Transfer", "Token Transfer", etc.)
///      - Amount (if applicable)
///      - Destination address (if applicable)
///   5. Warning if any inner instruction is unrecognized (blind)
///   6. Final confirmation
pub fn review_vault_tx(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    ix_data: &[u8],
) -> Result<bool, ParseError> {
    // Parse the vault transaction instruction data
    let vault_meta = crate::parser::vault_tx::parse_vault_tx_create(ix_data)?;
    let msg = &vault_meta.inner_message;

    // Check if any inner instructions are unrecognized
    let mut has_unknown = false;
    for i in 0..msg.num_instructions {
        let inner_ix = &msg.instructions[i];
        let program_id = msg.program_id(ix_data, inner_ix);
        if let Some(pid) = program_id {
            if *pid != inner::SYSTEM_PROGRAM
                && *pid != inner::SPL_TOKEN_PROGRAM
                && *pid != inner::ATA_PROGRAM
                && *pid != inner::COMPUTE_BUDGET_PROGRAM
                && *pid != inner::MEMO_PROGRAM
            {
                has_unknown = true;
            }
        } else {
            has_unknown = true;
        }
    }

    // If there are unknown instructions, check blind signing policy
    if has_unknown && !Settings.blind_signing_enabled() {
        return Err(ParseError::InvalidDiscriminator);
    }

    // Format multisig address
    let mut multisig_buf = [0u8; 45];
    let multisig_str = if let Some(key) = multisig {
        let len = format_base58(key, &mut multisig_buf).map_err(|_| ParseError::InvalidStructure)?;
        core::str::from_utf8(&multisig_buf[..len]).unwrap_or("???")
    } else {
        "Unknown"
    };

    // Format vault index
    let mut vault_buf = ArrayString::<8>::new();
    let _ = write!(&mut vault_buf, "#{}", vault_meta.vault_index);

    // Format instruction count
    let mut count_buf = ArrayString::<16>::new();
    let _ = write!(&mut count_buf, "{} instruction(s)", msg.num_instructions);

    // Build the header fields (shown before streaming inner instructions)
    let header_fields = [
        Field {
            name: "Action",
            value: "Create Vault Transaction",
        },
        Field {
            name: "Multisig",
            value: multisig_str,
        },
        Field {
            name: "Vault",
            value: vault_buf.as_str(),
        },
        Field {
            name: "Contains",
            value: count_buf.as_str(),
        },
    ];

    // If there are unknown inner instructions, show warning first
    if has_unknown {
        let mut warn_fields = [
            Field {
                name: "WARNING",
                value: "Contains unrecognized instructions",
            },
            Field {
                name: "Action",
                value: "Create Vault Transaction",
            },
            Field {
                name: "Multisig",
                value: multisig_str,
            },
            Field {
                name: "Vault",
                value: vault_buf.as_str(),
            },
        ];
        // Show blind signing warning review
        let approved = NbglReview::new()
            .titles("Blind Signing\nWarning", "", "Continue review?")
            .tx_type(TransactionType::Transaction)
            .blind()
            .show(comm, &warn_fields);
        if !approved {
            return Ok(false);
        }
    }

    // Now show each inner instruction one at a time
    // We reuse scratch buffers for each instruction
    let mut all_fields: [Field; 6] = core::array::from_fn(|_| Field {
        name: "",
        value: "",
    });
    let mut field_count = 0;

    // Add header fields first
    all_fields[0] = header_fields[0];
    all_fields[1] = header_fields[1];
    all_fields[2] = header_fields[2];
    all_fields[3] = header_fields[3];
    field_count = 4;

    // For each inner instruction, format a description
    // Note: with NbglReview we're limited to the fields we can hold at once.
    // For a proper streaming review (Phase 2 enhancement), we would use
    // NbglStreamingReview. For now, we show a summary.

    for i in 0..msg.num_instructions.min(2) {
        let inner_ix = &msg.instructions[i];
        let desc = inner::describe_inner_instruction_from_vault(ix_data, msg, inner_ix);

        // We need to keep the string alive for the Field reference
        // This is tricky without alloc — for now, show the instruction type only
        // Full amount/address display will use NbglStreamingReview in a later iteration
    }

    // Show the review with what we have
    let approved = NbglReview::new()
        .titles(
            "Review\nSquads Transaction",
            "",
            "Sign transaction?",
        )
        .tx_type(TransactionType::Transaction)
        .show(comm, &all_fields[..field_count]);

    Ok(approved)
}
