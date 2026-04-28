//! Zero-copy parser for vault_transaction_create instruction data.
//!
//! The instruction data layout is:
//!   [discriminator(8)] [vault_index(1)] [reserved(1)] [message_len(u32)] [message_bytes...]
//!
//! The inner TransactionMessage layout (Borsh-encoded, NOT Solana compact-u16):
//!   [num_signers(1)] [num_writable_signers(1)] [num_writable_non_signers(1)]
//!   [num_account_keys(u32)] [account_keys(N * 32)]
//!   [num_instructions(u32)] [instructions...]
//!     per instruction: [program_id_index(1)] [num_account_indexes(u32)] [indexes...] [data_len(u32)] [data...]
//!   [num_address_table_lookups(u32)] [lookups...]

use super::{ParseError, Reader};

/// Maximum inner instructions we'll attempt to parse.
const MAX_INNER_INSTRUCTIONS: usize = 16;

/// Parsed vault transaction metadata — zero-copy offsets into the raw data.
pub struct VaultTxMeta {
    pub vault_index: u8,
    pub inner_message: InnerMessageMeta,
}

/// Zero-copy metadata for the inner TransactionMessage.
pub struct InnerMessageMeta {
    pub num_signers: u8,
    pub num_writable_signers: u8,
    pub num_writable_non_signers: u8,
    /// Number of account keys in the inner message.
    pub num_account_keys: u32,
    /// Byte offset where account keys start in the raw instruction data.
    pub account_keys_offset: usize,
    /// Parsed inner instruction metadata.
    pub instructions: [InnerInstructionMeta; MAX_INNER_INSTRUCTIONS],
    /// Number of inner instructions parsed.
    pub num_instructions: usize,
}

/// Zero-copy metadata for an inner compiled instruction.
#[derive(Clone, Copy, Default)]
pub struct InnerInstructionMeta {
    /// Index into the inner message's account_keys array.
    pub program_id_index: u8,
    /// Byte offset of the account index array in the raw data.
    pub account_indexes_offset: usize,
    /// Number of account indexes.
    pub num_account_indexes: u32,
    /// Byte offset of the instruction data.
    pub data_offset: usize,
    /// Length of the instruction data.
    pub data_len: u32,
}

impl InnerMessageMeta {
    /// Get the 32-byte account key at the given index from the raw data.
    pub fn account_key<'a>(&self, raw: &'a [u8], index: u8) -> Option<&'a [u8; 32]> {
        if (index as u32) >= self.num_account_keys {
            return None;
        }
        let offset = self.account_keys_offset + (index as usize) * 32;
        if offset + 32 > raw.len() {
            return None;
        }
        raw[offset..offset + 32].try_into().ok()
    }

    /// Get the program ID for an inner instruction.
    pub fn program_id<'a>(&self, raw: &'a [u8], ix: &InnerInstructionMeta) -> Option<&'a [u8; 32]> {
        self.account_key(raw, ix.program_id_index)
    }

    /// Get the instruction data slice for an inner instruction.
    pub fn instruction_data<'a>(&self, raw: &'a [u8], ix: &InnerInstructionMeta) -> &'a [u8] {
        let end = ix.data_offset + ix.data_len as usize;
        if end > raw.len() {
            return &[];
        }
        &raw[ix.data_offset..end]
    }

    /// Get the account key referenced by an inner instruction's account index.
    pub fn instruction_account<'a>(
        &self,
        raw: &'a [u8],
        ix: &InnerInstructionMeta,
        account_idx: usize,
    ) -> Option<&'a [u8; 32]> {
        if account_idx >= ix.num_account_indexes as usize {
            return None;
        }
        let idx_offset = ix.account_indexes_offset + account_idx;
        if idx_offset >= raw.len() {
            return None;
        }
        let key_index = raw[idx_offset];
        self.account_key(raw, key_index)
    }
}

/// Parse a vault_transaction_create instruction's data to extract the inner message.
///
/// `ix_data` is the full instruction data starting with the 8-byte discriminator.
/// Returns metadata with offsets pointing into `ix_data`.
pub fn parse_vault_tx_create(ix_data: &[u8]) -> Result<VaultTxMeta, ParseError> {
    let mut r = Reader::new(ix_data);

    // Skip discriminator (already verified by caller)
    r.skip(8)?;

    let vault_index = r.read_u8()?;
    let _reserved = r.read_u8()?;

    // Message length (u32)
    let message_len = r.read_u32_le()?;
    if message_len as usize > r.remaining() {
        return Err(ParseError::Eof {
            offset: r.position(),
            need: message_len as usize,
            have: r.remaining(),
        });
    }

    // Parse the inner TransactionMessage (Borsh encoding — u32 lengths, not compact-u16)
    let num_signers = r.read_u8()?;
    let num_writable_signers = r.read_u8()?;
    let num_writable_non_signers = r.read_u8()?;

    // Account keys: u32 count, then N * 32 bytes
    let num_account_keys = r.read_u32_le()?;
    if num_account_keys > 64 {
        return Err(ParseError::InvalidStructure);
    }
    let account_keys_offset = r.position();
    r.skip(num_account_keys as usize * 32)?;

    // Instructions: u32 count
    let num_ix = r.read_u32_le()?;
    if num_ix as usize > MAX_INNER_INSTRUCTIONS {
        return Err(ParseError::InvalidStructure);
    }

    let mut instructions = [InnerInstructionMeta::default(); MAX_INNER_INSTRUCTIONS];
    for ix in instructions.iter_mut().take(num_ix as usize) {
        let program_id_index = r.read_u8()?;
        if program_id_index as u32 >= num_account_keys {
            return Err(ParseError::InvalidStructure);
        }

        // Account indexes: u32 count, then N bytes
        let num_account_indexes = r.read_u32_le()?;
        let account_indexes_offset = r.position();
        // Validate each index
        for _ in 0..num_account_indexes {
            let idx = r.read_u8()?;
            if idx as u32 >= num_account_keys {
                return Err(ParseError::InvalidStructure);
            }
        }

        // Data: u32 length, then N bytes
        let data_len = r.read_u32_le()?;
        let data_offset = r.position();
        r.skip(data_len as usize)?;

        *ix = InnerInstructionMeta {
            program_id_index,
            account_indexes_offset,
            num_account_indexes,
            data_offset,
            data_len,
        };
    }

    // Skip address table lookups (we don't resolve them on-device)
    // Just validate the count is parseable
    let _num_lookups = r.read_u32_le().unwrap_or(0);

    Ok(VaultTxMeta {
        vault_index,
        inner_message: InnerMessageMeta {
            num_signers,
            num_writable_signers,
            num_writable_non_signers,
            num_account_keys,
            account_keys_offset,
            instructions,
            num_instructions: num_ix as usize,
        },
    })
}
