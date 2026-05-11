//! Streaming review display for vault transactions with inner instructions.
//!
//! Compact review for vault transactions.
//! Fully decoded instructions show amount and destination when available.
//! Known but not decoded programs are labeled and marked as assisted review.

use crate::display::address::format_base58;
use crate::display::amount::format_sol;
use crate::parser::inner;
use crate::parser::inner::spl_token;
use crate::parser::inner::system;
use crate::parser::ParseError;
use crate::settings::Settings;
use arrayvec::ArrayString;
use core::fmt::Write;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::nbgl::{Field, NbglReview, TransactionType};

/// Maximum fields we can display in a single NbglReview call.
/// Header (4) + max inner ix fields (3 per ix * 6 ix max) + warning (1) = 23
const MAX_REVIEW_FIELDS: usize = 24;

/// Review a vault_transaction_create with per-instruction display.
pub fn review_vault_tx(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    ix_data: &[u8],
) -> Result<bool, ParseError> {
    let vault_meta = crate::parser::vault_tx::parse_vault_tx_create(ix_data)?;
    let msg = &vault_meta.inner_message;

    let mut has_unknown = false;
    let mut has_assisted_review = false;
    for i in 0..msg.num_instructions {
        let inner_ix = &msg.instructions[i];
        if let Some(pid) = msg.program_id(ix_data, inner_ix) {
            if !inner::is_known_program_for_review(pid) {
                has_unknown = true;
            } else if !inner::is_fully_decoded_program_for_review(pid) {
                has_assisted_review = true;
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
        let len =
            format_base58(key, &mut multisig_buf).map_err(|_| ParseError::InvalidStructure)?;
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

    // Per-instruction buffers: each IX gets up to 3 fields (type, amount, destination)
    // Max 6 inner instructions displayed (6 * 3 = 18 fields + 4 header + 1 warning = 23)
    const MAX_DISPLAY_IX: usize = 6;
    let display_count = msg.num_instructions.min(MAX_DISPLAY_IX);

    // Buffers that must outlive the Field references
    let mut ix_labels: [ArrayString<32>; MAX_DISPLAY_IX] =
        core::array::from_fn(|_| ArrayString::new());
    let mut ix_amounts: [ArrayString<32>; MAX_DISPLAY_IX] =
        core::array::from_fn(|_| ArrayString::new());
    let mut ix_dests: [ArrayString<48>; MAX_DISPLAY_IX] =
        core::array::from_fn(|_| ArrayString::new());
    let mut ix_descs: [ArrayString<20>; MAX_DISPLAY_IX] =
        core::array::from_fn(|_| ArrayString::new());
    let mut ix_has_amount: [bool; MAX_DISPLAY_IX] = [false; MAX_DISPLAY_IX];
    let mut ix_has_dest: [bool; MAX_DISPLAY_IX] = [false; MAX_DISPLAY_IX];

    for i in 0..display_count {
        let inner_ix = &msg.instructions[i];
        let ix_data_slice = msg.instruction_data(ix_data, inner_ix);
        let program_id = msg.program_id(ix_data, inner_ix);

        // Store instruction type description
        let desc = inner::describe_inner_instruction_from_vault(ix_data, msg, inner_ix);
        let _ = ix_descs[i].try_push_str(desc.as_str());
        let _ = write!(&mut ix_labels[i], "IX {}/{}", i + 1, msg.num_instructions);

        // Extract amount and destination into separate buffers
        // Amount field shows: "SOL Transfer: 50 SOL" or "Token Transfer: 1000" or just the type name
        if let Some(pid) = program_id {
            if *pid == inner::SYSTEM_PROGRAM {
                if let Some(lamports) = system::extract_transfer_amount(ix_data_slice) {
                    let sol = format_sol(lamports);
                    let _ = write!(&mut ix_amounts[i], "SOL Transfer: {}", sol.as_str());
                    ix_has_amount[i] = true;
                    // Destination is account index 1
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 1) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = ix_dests[i].try_push_str(dest_str);
                            ix_has_dest[i] = true;
                        }
                    }
                }
            } else if *pid == inner::SPL_TOKEN_PROGRAM || *pid == inner::TOKEN_2022_PROGRAM {
                if let Some((amount, decimals)) = spl_token::extract_transfer_checked(ix_data_slice)
                {
                    let formatted = crate::display::amount::format_amount(amount, decimals);
                    let _ = write!(&mut ix_amounts[i], "Token: {}", formatted.as_str());
                    ix_has_amount[i] = true;
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 2) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = ix_dests[i].try_push_str(dest_str);
                            ix_has_dest[i] = true;
                        }
                    }
                } else if let Some(raw_amount) = spl_token::extract_transfer_amount(ix_data_slice) {
                    let _ = write!(&mut ix_amounts[i], "Token: {} raw", raw_amount);
                    ix_has_amount[i] = true;
                    if let Some(dest) = msg.instruction_account(ix_data, inner_ix, 1) {
                        let mut dest_buf = [0u8; 45];
                        if let Ok(len) = format_base58(dest, &mut dest_buf) {
                            let dest_str = core::str::from_utf8(&dest_buf[..len]).unwrap_or("???");
                            let _ = ix_dests[i].try_push_str(dest_str);
                            ix_has_dest[i] = true;
                        }
                    }
                }
            }
        } else {
            // No program ID — label already says the instruction type
        }
    }

    // --- Assemble fields array ---
    let mut fields: [Field; MAX_REVIEW_FIELDS] = core::array::from_fn(|_| Field {
        name: "",
        value: "",
    });
    let mut field_count = 0;

    fields[field_count] = Field {
        name: "Action",
        value: "Create Vault Transaction",
    };
    field_count += 1;

    fields[field_count] = Field {
        name: "Review",
        value: if has_unknown {
            "Blind signing"
        } else if has_assisted_review {
            "Assisted review"
        } else {
            "Clear signing"
        },
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

    if has_unknown {
        fields[field_count] = Field {
            name: "Warning",
            value: "Unrecognized instructions",
        };
        field_count += 1;
    } else if has_assisted_review {
        fields[field_count] = Field {
            name: "Verify",
            value: "Confirm full details in Squads",
        };
        field_count += 1;
    }

    // Per-instruction fields (up to 3 fields each: type, amount, destination)
    for i in 0..display_count {
        // Field 1: "IX N/M" with type+amount or just type as value
        fields[field_count] = Field {
            name: ix_labels[i].as_str(),
            value: if ix_has_amount[i] {
                ix_amounts[i].as_str()
            } else {
                ix_descs[i].as_str()
            },
        };
        field_count += 1;

        // Field 2: destination (if we have one)
        if ix_has_dest[i] {
            fields[field_count] = Field {
                name: "To",
                value: ix_dests[i].as_str(),
            };
            field_count += 1;
        }
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
        "Review\nBlind Signing"
    } else {
        "Review\nSquads Vault"
    };

    let approved = NbglReview::new()
        .titles(title, "", "Sign transaction?")
        .tx_type(TransactionType::Transaction)
        .show(comm, &fields[..field_count]);

    Ok(approved)
}
