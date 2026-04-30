//! Streaming review display for vault transactions with inner instructions.
//!
//! Design: Option C (Hybrid) — compact "IX N/M: Type" + "Amount → Dest" per instruction.
//! Each known instruction takes 2 screens, unknown takes 2 screens.
//! Total for a typical 2-instruction vault tx: Header(3) + IX1(2) + IX2(2) + Footer(1) = 8 screens.

use crate::display::address::format_base58;
use crate::display::amount::format_sol;
use crate::parser::inner;
use crate::parser::inner::spl_token;
use crate::parser::inner::system;
use crate::parser::vault_tx::VaultTxMeta;
use crate::parser::ParseError;
use crate::settings::Settings;
use arrayvec::ArrayString;
use core::fmt::Write;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::nbgl::{Field, NbglReview, TransactionType};

/// Maximum fields we can display in a single NbglReview call.
/// Header (3) + max inner ix fields (2 per ix * 8 ix max) + warning (1) = 20
const MAX_REVIEW_FIELDS: usize = 20;

/// Review a vault_transaction_create with per-instruction display.
///
/// Screen flow (Option C — Hybrid):
/// ```text
/// Screen 1:  "Squads: Create Transaction"
/// Screen 2:  Multisig: <base58>
/// Screen 3:  Vault: #N | M instructions
/// Screen 4:  "IX 1/M: SOL Transfer"
/// Screen 5:  "50 SOL → 8mD4H...xyz"
/// Screen 6:  "IX 2/M: USDC Transfer"
/// Screen 7:  "1,000 USDC → 9kR2J...def"
/// Screen 8:  "IX 3/M: Unknown ⚠"
/// Screen 9:  "Program: DezXA...ghi"
/// Screen 10: [Sign / Reject]
/// ```
pub fn review_vault_tx(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    ix_data: &[u8],
) -> Result<bool, ParseError> {
    let vault_meta = crate::parser::vault_tx::parse_vault_tx_create(ix_data)?;
    let msg = &vault_meta.inner_message;

    // Check for unknown inner instructions
    let mut has_unknown = false;
    for i in 0..msg.num_instructions {
        let inner_ix = &msg.instructions[i];
        if let Some(pid) = msg.program_id(ix_data, inner_ix) {
            if *pid != inner::SYSTEM_PROGRAM
                && *pid != inner::SPL_TOKEN_PROGRAM
                && *pid != inner::TOKEN_2022_PROGRAM
                && *pid != inner::ATA_PROGRAM
                && *pid != inner::COMPUTE_BUDGET_PROGRAM
                && *pid != inner::MEMO_PROGRAM
                && *pid != inner::BPF_LOADER_UPGRADEABLE
                && *pid != inner::JUPITER_V6_PROGRAM
                && *pid != inner::MARINADE_PROGRAM
                && *pid != inner::JITO_STAKE_POOL
            {
                has_unknown = true;
            }
        } else {
            has_unknown = true;
        }
    }

    if has_unknown && !Settings.blind_signing_enabled() {
        return Err(ParseError::InvalidDiscriminator);
    }

    // --- Build all display fields ---
    // We use scratch buffers that live on the stack for the duration of the review.
    let mut multisig_buf = [0u8; 45];
    let multisig_str = if let Some(key) = multisig {
        let len = format_base58(key, &mut multisig_buf).map_err(|_| ParseError::InvalidStructure)?;
        core::str::from_utf8(&multisig_buf[..len]).unwrap_or("???")
    } else {
        "Unknown"
    };

    // Vault + instruction count
    let mut vault_info = ArrayString::<32>::new();
    let _ = write!(
        &mut vault_info,
        "#{} | {} instruction(s)",
        vault_meta.vault_index, msg.num_instructions
    );

    // Per-instruction label and detail buffers (max 8 inner instructions displayed)
    const MAX_DISPLAY_IX: usize = 8;
    let display_count = msg.num_instructions.min(MAX_DISPLAY_IX);

    // We need these buffers to outlive the Field references
    let mut ix_labels: [ArrayString<32>; MAX_DISPLAY_IX] = core::array::from_fn(|_| ArrayString::new());
    let mut ix_details: [ArrayString<64>; MAX_DISPLAY_IX] = core::array::from_fn(|_| ArrayString::new());

    for i in 0..display_count {
        let inner_ix = &msg.instructions[i];
        let ix_data_slice = msg.instruction_data(ix_data, inner_ix);
        let program_id = msg.program_id(ix_data, inner_ix);

        // Build label: "IX 1/3: SOL Transfer"
        let desc = inner::describe_inner_instruction_from_vault(ix_data, msg, inner_ix);
        let _ = write!(
            &mut ix_labels[i],
            "IX {}/{}",
            i + 1,
            msg.num_instructions
        );

        // Build detail based on instruction type
        if let Some(pid) = program_id {
            if *pid == inner::SYSTEM_PROGRAM {
                // Try to extract SOL transfer amount + destination
                if let Some(lamports) = system::extract_transfer_amount(ix_data_slice) {
                    let sol = format_sol(lamports);
                    // Destination is account index 1 in the instruction
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 1) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = write!(&mut ix_details[i], "{} → {}", sol.as_str(), dest_str);
                        } else {
                            let _ = write!(&mut ix_details[i], "{}", sol.as_str());
                        }
                    } else {
                        let _ = write!(&mut ix_details[i], "{}", sol.as_str());
                    }
                } else {
                    let _ = ix_details[i].try_push_str(desc.as_str());
                }
            } else if *pid == inner::SPL_TOKEN_PROGRAM {
                // Try to extract token transfer amount
                if let Some(amount) = spl_token::extract_transfer_amount(ix_data_slice) {
                    // Destination is account index 1
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 1) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = write!(&mut ix_details[i], "{} tokens → {}", amount, dest_str);
                        } else {
                            let _ = write!(&mut ix_details[i], "{} tokens", amount);
                        }
                    } else {
                        let _ = write!(&mut ix_details[i], "{} tokens", amount);
                    }
                } else if let Some((amount, decimals)) =
                    spl_token::extract_transfer_checked(ix_data_slice)
                {
                    let formatted = crate::display::amount::format_amount(amount, decimals);
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 2) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = write!(
                                &mut ix_details[i],
                                "{} → {}",
                                formatted.as_str(),
                                dest_str
                            );
                        } else {
                            let _ = ix_details[i].try_push_str(formatted.as_str());
                        }
                    } else {
                        let _ = ix_details[i].try_push_str(formatted.as_str());
                    }
                } else {
                    let _ = ix_details[i].try_push_str(desc.as_str());
                }
            } else if *pid == inner::TOKEN_2022_PROGRAM {
                // Token-2022 shares layout with SPL Token
                if let Some(amount) = spl_token::extract_transfer_amount(ix_data_slice) {
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 1) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = write!(&mut ix_details[i], "{} tokens → {}", amount, dest_str);
                        } else {
                            let _ = write!(&mut ix_details[i], "{} tokens", amount);
                        }
                    } else {
                        let _ = write!(&mut ix_details[i], "{} tokens", amount);
                    }
                } else {
                    let _ = ix_details[i].try_push_str(desc.as_str());
                }
            } else if *pid == inner::ATA_PROGRAM
                || *pid == inner::COMPUTE_BUDGET_PROGRAM
                || *pid == inner::MEMO_PROGRAM
                || *pid == inner::BPF_LOADER_UPGRADEABLE
                || *pid == inner::JUPITER_V6_PROGRAM
                || *pid == inner::MARINADE_PROGRAM
                || *pid == inner::JITO_STAKE_POOL
            {
                let _ = ix_details[i].try_push_str(desc.as_str());
            } else {
                // Unknown instruction — use program registry for a clean label
                use crate::parser::inner::programs;
                let (name, category) = programs::lookup_program(pid)
                    .unwrap_or(("Unknown", programs::ProgramCategory::Unknown));

                match category {
                    programs::ProgramCategory::Swap => {
                        let _ = write!(&mut ix_details[i], "{} (swap details on computer)", name);
                    }
                    programs::ProgramCategory::Lending => {
                        let _ = write!(&mut ix_details[i], "{} (details on computer)", name);
                    }
                    programs::ProgramCategory::Staking => {
                        let _ = write!(&mut ix_details[i], "{} (details on computer)", name);
                    }
                    programs::ProgramCategory::Unknown => {
                        // Truly unknown — show shortened program ID
                        let mut pid_buf = [0u8; 45];
                        if let Ok(len) = format_base58(pid, &mut pid_buf) {
                            let pid_str = core::str::from_utf8(&pid_buf[..len]).unwrap_or("???");
                            // Show first 8 + last 4 chars
                            if pid_str.len() > 12 {
                                let _ = write!(
                                    &mut ix_details[i],
                                    "{}...{}",
                                    &pid_str[..8],
                                    &pid_str[pid_str.len() - 4..]
                                );
                            } else {
                                let _ = ix_details[i].try_push_str(pid_str);
                            }
                        } else {
                            let _ = ix_details[i].try_push_str("Unknown");
                        }
                    }
                    _ => {
                        let _ = ix_details[i].try_push_str(name);
                    }
                }
            }
        } else {
            let _ = ix_details[i].try_push_str("Invalid program");
        }
    }

    // --- Assemble fields array ---
    let mut fields: [Field; MAX_REVIEW_FIELDS] = core::array::from_fn(|_| Field {
        name: "",
        value: "",
    });
    let mut field_count = 0;

    // Header
    fields[field_count] = Field {
        name: "Action",
        value: "Create Vault Transaction",
    };
    field_count += 1;

    fields[field_count] = Field {
        name: "Multisig",
        value: multisig_str,
    };
    field_count += 1;

    fields[field_count] = Field {
        name: "Vault",
        value: vault_info.as_str(),
    };
    field_count += 1;

    // Warning if blind signing needed
    if has_unknown {
        fields[field_count] = Field {
            name: "⚠ Warning",
            value: "Contains unrecognized instructions",
        };
        field_count += 1;
    }

    // Per-instruction fields
    for i in 0..display_count {
        fields[field_count] = Field {
            name: ix_labels[i].as_str(),
            value: ix_details[i].as_str(),
        };
        field_count += 1;
    }

    // If there are more instructions than we display
    if msg.num_instructions > MAX_DISPLAY_IX {
        let mut more_buf = ArrayString::<32>::new();
        let _ = write!(
            &mut more_buf,
            "+{} more instructions",
            msg.num_instructions - MAX_DISPLAY_IX
        );
        fields[field_count] = Field {
            name: "Note",
            value: "Additional instructions not shown",
        };
        field_count += 1;
    }

    // Show the review
    let title = if has_unknown {
        "Review\n⚠ Blind Signing"
    } else {
        "Review\nSquads Transaction"
    };

    let approved = NbglReview::new()
        .titles(title, "", "Sign transaction?")
        .tx_type(TransactionType::Transaction)
        .show(comm, &fields[..field_count]);

    Ok(approved)
}
