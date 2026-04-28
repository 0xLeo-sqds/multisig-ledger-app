use super::solana_message::ParsedMessage;
use super::{ParseError, Reader};
use crate::display;
use crate::settings::Settings;
use crate::AppSW;
use ledger_device_sdk::io::Comm;

/// Squads v4 mainnet program ID: SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
pub const SQUADS_V4_PROGRAM_ID: [u8; 32] = [
    0x06, 0x81, 0xc4, 0xce, 0x47, 0xe2, 0x23, 0x68, 0xb8, 0xb1, 0x55, 0x5e, 0xc8, 0x87, 0xaf,
    0x09, 0x2e, 0xfc, 0x7e, 0xfb, 0xb6, 0x6c, 0xa3, 0xf5, 0x2f, 0xbf, 0x68, 0xd4, 0xac, 0x9c,
    0xb7, 0xa8,
];

// Squads v4 instruction discriminators — sha256("global:<name>")[0..8]
// Verified against msig-cli source at:
//   src/application/config_tx.rs, src/application/transfer.rs, src/domain/proposal.rs
pub const DISC_PROPOSAL_CREATE: [u8; 8] = [0xdc, 0x3c, 0x49, 0xe0, 0x1e, 0x6c, 0x4f, 0x9f];
pub const DISC_PROPOSAL_ACTIVATE: [u8; 8] = [0x0b, 0x22, 0x5c, 0xf8, 0x9a, 0x1b, 0x33, 0x6a];
pub const DISC_PROPOSAL_APPROVE: [u8; 8] = [0x90, 0x25, 0xa4, 0x88, 0xbc, 0xd8, 0x2a, 0xf8];
pub const DISC_PROPOSAL_REJECT: [u8; 8] = [0xf3, 0x3e, 0x86, 0x9c, 0xe6, 0x6a, 0xf6, 0x87];
pub const DISC_PROPOSAL_CANCEL: [u8; 8] = [0x1b, 0x2a, 0x7f, 0xed, 0x26, 0xa3, 0x54, 0xcb];
pub const DISC_VAULT_TX_CREATE: [u8; 8] = [0x30, 0xfa, 0x4e, 0xa8, 0xd0, 0xe2, 0xda, 0xd3];
pub const DISC_VAULT_TX_EXECUTE: [u8; 8] = [0xc2, 0x08, 0xa1, 0x57, 0x99, 0xa4, 0x19, 0xab];
pub const DISC_CONFIG_TX_CREATE: [u8; 8] = [0x9b, 0xec, 0x57, 0xe4, 0x89, 0x4b, 0x51, 0x27];
pub const DISC_CONFIG_TX_EXECUTE: [u8; 8] = [0x72, 0x92, 0xf4, 0xbd, 0xfc, 0x8c, 0x24, 0x28];

/// Identified Squads instruction type.
#[derive(Debug, Clone, Copy)]
pub enum SquadsInstruction {
    ProposalCreate,
    ProposalActivate,
    ProposalApprove,
    ProposalReject,
    ProposalCancel,
    VaultTransactionCreate,
    VaultTransactionExecute,
    ConfigTransactionCreate,
    ConfigTransactionExecute,
    Unknown,
}

/// Identify a Squads instruction from its data discriminator.
pub fn identify_instruction(data: &[u8]) -> SquadsInstruction {
    if data.len() < 8 {
        return SquadsInstruction::Unknown;
    }
    let disc: [u8; 8] = data[..8].try_into().unwrap_or([0; 8]);
    match disc {
        DISC_PROPOSAL_CREATE => SquadsInstruction::ProposalCreate,
        DISC_PROPOSAL_ACTIVATE => SquadsInstruction::ProposalActivate,
        DISC_PROPOSAL_APPROVE => SquadsInstruction::ProposalApprove,
        DISC_PROPOSAL_REJECT => SquadsInstruction::ProposalReject,
        DISC_PROPOSAL_CANCEL => SquadsInstruction::ProposalCancel,
        DISC_VAULT_TX_CREATE => SquadsInstruction::VaultTransactionCreate,
        DISC_VAULT_TX_EXECUTE => SquadsInstruction::VaultTransactionExecute,
        DISC_CONFIG_TX_CREATE => SquadsInstruction::ConfigTransactionCreate,
        DISC_CONFIG_TX_EXECUTE => SquadsInstruction::ConfigTransactionExecute,
        _ => SquadsInstruction::Unknown,
    }
}

/// Review a parsed Solana transaction for Squads instructions.
/// Returns true if the user approved, false if rejected.
pub fn review_transaction(
    comm: &mut Comm,
    msg: &ParsedMessage<'_>,
    _raw: &[u8],
) -> Result<bool, ParseError> {
    // Find Squads instructions in the transaction
    for i in 0..msg.num_instructions {
        let ix = &msg.instructions[i];
        let program_id = msg.program_id(ix).ok_or(ParseError::InvalidStructure)?;

        if *program_id != SQUADS_V4_PROGRAM_ID {
            // Not a Squads instruction — skip for now (non-Squads instructions
            // like compute budget are handled in v2)
            continue;
        }

        let ix_data = msg.instruction_data(ix);
        let squads_ix = identify_instruction(ix_data);

        match squads_ix {
            SquadsInstruction::ProposalApprove => {
                // Account layout: [multisig, member, proposal]
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(comm, "Approve Proposal", multisig);
            }
            SquadsInstruction::ProposalReject => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(comm, "Reject Proposal", multisig);
            }
            SquadsInstruction::ProposalCancel => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(comm, "Cancel Proposal", multisig);
            }
            SquadsInstruction::ProposalCreate => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(comm, "Create Proposal", multisig);
            }
            SquadsInstruction::ProposalActivate => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(comm, "Activate Proposal", multisig);
            }
            SquadsInstruction::VaultTransactionCreate => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_vault_tx_create(comm, multisig, ix_data);
            }
            SquadsInstruction::VaultTransactionExecute => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(
                    comm,
                    "Execute Transaction",
                    multisig,
                );
            }
            SquadsInstruction::ConfigTransactionCreate => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_config_tx_create(comm, multisig, ix_data);
            }
            SquadsInstruction::ConfigTransactionExecute => {
                let multisig = msg.instruction_account(ix, 0);
                return display::review::review_proposal_vote(
                    comm,
                    "Execute Config Change",
                    multisig,
                );
            }
            SquadsInstruction::Unknown => {
                // Check if blind signing is enabled
                if !Settings.blind_signing_enabled() {
                    return Err(ParseError::InvalidDiscriminator);
                }
                return display::review::review_blind(comm);
            }
        }
    }

    // No Squads instruction found — check blind signing
    if !Settings.blind_signing_enabled() {
        return Err(ParseError::InvalidDiscriminator);
    }
    display::review::review_blind(comm)
}
