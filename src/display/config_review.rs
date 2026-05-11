//! Display screens for config_transaction_create with ConfigAction variants.

use crate::display::address::format_base58;
use crate::parser::config_tx::ConfigActionParsed;
use crate::parser::ParseError;
use arrayvec::ArrayString;
use core::fmt::Write;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::nbgl::{Field, NbglReview, TransactionType};

/// Review a config_transaction_create with ConfigAction display.
pub fn review_config_tx(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    ix_data: &[u8],
) -> Result<bool, ParseError> {
    let config = crate::parser::config_tx::parse_config_tx_create(ix_data)?;

    // Format multisig address
    let mut multisig_buf = [0u8; 45];
    let multisig_str = if let Some(key) = multisig {
        let len =
            format_base58(key, &mut multisig_buf).map_err(|_| ParseError::InvalidStructure)?;
        core::str::from_utf8(&multisig_buf[..len]).unwrap_or("???")
    } else {
        "Unknown"
    };

    // For each action, show a review screen
    for i in 0..config.num_actions {
        let approved = review_config_action(comm, multisig_str, &config.actions[i])?;
        if !approved {
            return Ok(false);
        }
    }

    Ok(true)
}

fn review_config_action(
    comm: &mut Comm,
    multisig_str: &str,
    action: &ConfigActionParsed,
) -> Result<bool, ParseError> {
    match action {
        ConfigActionParsed::AddMember {
            member,
            permissions,
        } => {
            let mut member_buf = [0u8; 45];
            let member_len =
                format_base58(member, &mut member_buf).map_err(|_| ParseError::InvalidStructure)?;
            let member_str = core::str::from_utf8(&member_buf[..member_len]).unwrap_or("???");

            let mut perm_str = ArrayString::<32>::new();
            if permissions & 1 != 0 {
                let _ = perm_str.try_push_str("Initiate");
            }
            if permissions & 2 != 0 {
                if !perm_str.is_empty() {
                    let _ = perm_str.try_push_str(", ");
                }
                let _ = perm_str.try_push_str("Vote");
            }
            if permissions & 4 != 0 {
                if !perm_str.is_empty() {
                    let _ = perm_str.try_push_str(", ");
                }
                let _ = perm_str.try_push_str("Execute");
            }
            if perm_str.is_empty() {
                let _ = perm_str.try_push_str("None");
            }

            let fields = [
                Field {
                    name: "Action",
                    value: "Add Member",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "Member",
                    value: member_str,
                },
                Field {
                    name: "Permissions",
                    value: perm_str.as_str(),
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::RemoveMember { member } => {
            let mut member_buf = [0u8; 45];
            let member_len =
                format_base58(member, &mut member_buf).map_err(|_| ParseError::InvalidStructure)?;
            let member_str = core::str::from_utf8(&member_buf[..member_len]).unwrap_or("???");

            let fields = [
                Field {
                    name: "Action",
                    value: "Remove Member",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "Member",
                    value: member_str,
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::ChangeThreshold { new_threshold } => {
            let mut thresh_buf = ArrayString::<8>::new();
            let _ = write!(&mut thresh_buf, "{}", new_threshold);

            let fields = [
                Field {
                    name: "Action",
                    value: "Change Threshold",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "New Threshold",
                    value: thresh_buf.as_str(),
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::SetTimeLock { new_time_lock } => {
            let mut lock_buf = ArrayString::<16>::new();
            let _ = write!(&mut lock_buf, "{}s", new_time_lock);

            let fields = [
                Field {
                    name: "Action",
                    value: "Set Time Lock",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "Duration",
                    value: lock_buf.as_str(),
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::AddSpendingLimit {
            vault_index,
            mint,
            amount,
            period,
            num_members,
            num_destinations,
        } => {
            let mut mint_buf = [0u8; 45];
            let mint_len =
                format_base58(mint, &mut mint_buf).map_err(|_| ParseError::InvalidStructure)?;
            let mint_str = core::str::from_utf8(&mint_buf[..mint_len]).unwrap_or("???");

            let mut amount_buf = ArrayString::<32>::new();
            let _ = write!(&mut amount_buf, "{}", amount);

            let mut vault_buf = ArrayString::<8>::new();
            let _ = write!(&mut vault_buf, "{}", vault_index);

            let mut details_buf = ArrayString::<64>::new();
            let _ = write!(
                &mut details_buf,
                "{} {} members, {} destinations",
                period.label(),
                num_members,
                num_destinations
            );

            let fields = [
                Field {
                    name: "Action",
                    value: "Add Spending Limit",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "Vault Index",
                    value: vault_buf.as_str(),
                },
                Field {
                    name: "Amount",
                    value: amount_buf.as_str(),
                },
                Field {
                    name: "Limit Details",
                    value: details_buf.as_str(),
                },
                Field {
                    name: "Token Mint",
                    value: mint_str,
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::RemoveSpendingLimit { spending_limit } => {
            let mut limit_buf = [0u8; 45];
            let limit_len = format_base58(spending_limit, &mut limit_buf)
                .map_err(|_| ParseError::InvalidStructure)?;
            let limit_str = core::str::from_utf8(&limit_buf[..limit_len]).unwrap_or("???");

            let fields = [
                Field {
                    name: "Action",
                    value: "Remove Spending Limit",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "Spending Limit",
                    value: limit_str,
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::SetRentCollector { new_rent_collector } => {
            let mut collector_buf = [0u8; 45];
            let collector_display = if let Some(key) = new_rent_collector {
                let len = format_base58(key, &mut collector_buf)
                    .map_err(|_| ParseError::InvalidStructure)?;
                core::str::from_utf8(&collector_buf[..len]).unwrap_or("???")
            } else {
                "None (disabled)"
            };

            let fields = [
                Field {
                    name: "Action",
                    value: "Set Rent Collector",
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
                Field {
                    name: "Collector",
                    value: collector_display,
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
        ConfigActionParsed::Unknown { tag } => {
            let mut tag_buf = ArrayString::<16>::new();
            let _ = write!(&mut tag_buf, "Unknown ({})", tag);

            let fields = [
                Field {
                    name: "Action",
                    value: tag_buf.as_str(),
                },
                Field {
                    name: "Multisig",
                    value: multisig_str,
                },
            ];
            let approved = NbglReview::new()
                .titles("Review\nConfig Change", "", "Sign transaction?")
                .tx_type(TransactionType::Transaction)
                .show(comm, &fields);
            Ok(approved)
        }
    }
}
