use crate::display::address::format_base58;
use crate::parser::ParseError;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::nbgl::{Field, NbglReview, TransactionType};

/// Review a proposal vote action (approve, reject, cancel, create, activate, execute).
pub fn review_proposal_vote(
    comm: &mut Comm,
    action: &str,
    multisig: Option<&[u8; 32]>,
) -> Result<bool, ParseError> {
    let mut multisig_str = [0u8; 45];
    let multisig_display = if let Some(key) = multisig {
        let len = format_base58(key, &mut multisig_str).map_err(|_| ParseError::InvalidStructure)?;
        core::str::from_utf8(&multisig_str[..len]).unwrap_or("???")
    } else {
        "Unknown"
    };

    let fields = [
        Field {
            name: "Action",
            value: action,
        },
        Field {
            name: "Multisig",
            value: multisig_display,
        },
    ];

    let title_buf: &str = "Review\nSquads Transaction";
    let approved = NbglReview::new()
        .titles(title_buf, "", "Sign transaction?")
        .tx_type(TransactionType::Transaction)
        .show(comm, &fields);

    Ok(approved)
}

/// Review a vault transaction create (placeholder — Phase 2 will add inner instruction decoding).
pub fn review_vault_tx_create(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    _ix_data: &[u8],
) -> Result<bool, ParseError> {
    // Phase 2 will decode the inner VaultTransactionMessage here
    review_proposal_vote(comm, "Create Vault Transaction", multisig)
}

/// Review a config transaction create (placeholder — Phase 3 will add ConfigAction decoding).
pub fn review_config_tx_create(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    _ix_data: &[u8],
) -> Result<bool, ParseError> {
    // Phase 3 will decode the ConfigAction variants here
    review_proposal_vote(comm, "Create Config Change", multisig)
}

/// Review a spending_limit_use — direct fund transfer without proposal.
/// This is high-security: must show amount, destination, and multisig.
pub fn review_spending_limit_use(
    comm: &mut Comm,
    multisig: Option<&[u8; 32]>,
    destination: Option<&[u8; 32]>,
    ix_data: &[u8],
) -> Result<bool, ParseError> {
    use crate::display::amount::format_amount;
    use arrayvec::ArrayString;
    use core::fmt::Write;

    let mut multisig_buf = [0u8; 45];
    let multisig_display = if let Some(key) = multisig {
        let len = format_base58(key, &mut multisig_buf).map_err(|_| ParseError::InvalidStructure)?;
        core::str::from_utf8(&multisig_buf[..len]).unwrap_or("???")
    } else {
        "Unknown"
    };

    let mut dest_buf = [0u8; 45];
    let dest_display = if let Some(key) = destination {
        let len = format_base58(key, &mut dest_buf).map_err(|_| ParseError::InvalidStructure)?;
        core::str::from_utf8(&dest_buf[..len]).unwrap_or("???")
    } else {
        "Unknown"
    };

    // Parse amount and decimals from instruction data
    // Layout after 8-byte discriminator: amount(u64) + decimals(u8)
    let mut amount_str = ArrayString::<32>::new();
    if ix_data.len() >= 17 {
        let mut r = crate::parser::Reader::new(&ix_data[8..]);
        if let (Ok(amount), Ok(decimals)) = (r.read_u64_le(), r.read_u8()) {
            let formatted = format_amount(amount, decimals);
            let _ = amount_str.try_push_str(formatted.as_str());
        } else {
            let _ = amount_str.try_push_str("(parse error)");
        }
    } else {
        let _ = amount_str.try_push_str("(unknown)");
    }

    let fields = [
        Field {
            name: "⚠ Action",
            value: "Spending Limit Transfer",
        },
        Field {
            name: "Multisig",
            value: multisig_display,
        },
        Field {
            name: "Amount",
            value: amount_str.as_str(),
        },
        Field {
            name: "To",
            value: dest_display,
        },
    ];

    let approved = NbglReview::new()
        .titles("Review\nSpending Limit", "", "Sign transaction?")
        .tx_type(TransactionType::Transaction)
        .show(comm, &fields);

    Ok(approved)
}

/// Review a blind-signed transaction (unrecognized instruction format).
pub fn review_blind(comm: &mut Comm) -> Result<bool, ParseError> {
    let fields = [Field {
        name: "WARNING",
        value: "Unrecognized transaction.\nVerify details on computer.",
    }];

    let approved = NbglReview::new()
        .titles("Blind Signing", "", "Sign transaction?")
        .tx_type(TransactionType::Transaction)
        .show(comm, &fields);

    Ok(approved)
}
