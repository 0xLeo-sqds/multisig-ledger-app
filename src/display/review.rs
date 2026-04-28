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

/// Review a blind-signed transaction (unrecognized instruction format).
pub fn review_blind(comm: &mut Comm) -> Result<bool, ParseError> {
    let fields = [Field {
        name: "WARNING",
        value: "Unrecognized transaction.\nVerify details on computer.",
    }];

    let approved = NbglReview::new()
        .titles("Blind Signing", "", "Sign transaction?")
        .tx_type(TransactionType::Transaction)
        .blind()
        .show(comm, &fields);

    Ok(approved)
}
