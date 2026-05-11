use super::{ParseError, Reader};

/// Maximum number of account keys in a Solana transaction.
/// Solana's actual limit is 256, but we only need offsets.
#[allow(dead_code)]
const MAX_ACCOUNT_KEYS: usize = 64;

/// Maximum number of instructions in a Solana transaction.
const MAX_INSTRUCTIONS: usize = 32;

/// Parsed Solana legacy message — zero-copy references into the raw buffer.
#[allow(dead_code)]
pub struct ParsedMessage<'a> {
    /// Raw message bytes (the entire buffer).
    pub raw: &'a [u8],
    /// Number of required signatures.
    pub num_required_sigs: u8,
    /// Number of read-only signed accounts.
    pub num_readonly_signed: u8,
    /// Number of read-only unsigned accounts.
    pub num_readonly_unsigned: u8,
    /// Number of account keys.
    pub num_accounts: u16,
    /// Byte offset where account keys begin in the raw buffer.
    pub accounts_offset: usize,
    /// Recent blockhash (32 bytes).
    pub blockhash: &'a [u8; 32],
    /// Parsed instruction metadata.
    pub instructions: [InstructionMeta; MAX_INSTRUCTIONS],
    /// Number of instructions parsed.
    pub num_instructions: usize,
}

/// Zero-copy instruction metadata — offsets into the raw message buffer.
#[derive(Clone, Copy, Default)]
pub struct InstructionMeta {
    /// Index of the program ID in the account keys array.
    pub program_id_index: u8,
    /// Byte offset of the account indexes array in the raw buffer.
    pub accounts_offset: usize,
    /// Number of account indexes.
    pub num_accounts: u16,
    /// Byte offset of the instruction data in the raw buffer.
    pub data_offset: usize,
    /// Length of the instruction data.
    pub data_len: u16,
}

impl<'a> ParsedMessage<'a> {
    /// Get the 32-byte account key at the given index.
    pub fn account_key(&self, index: u8) -> Option<&'a [u8; 32]> {
        if (index as u16) >= self.num_accounts {
            return None;
        }
        let offset = self.accounts_offset + (index as usize) * 32;
        if offset + 32 > self.raw.len() {
            return None;
        }
        self.raw[offset..offset + 32].try_into().ok()
    }

    /// Get the program ID pubkey for an instruction.
    pub fn program_id(&self, ix: &InstructionMeta) -> Option<&'a [u8; 32]> {
        self.account_key(ix.program_id_index)
    }

    /// Get the instruction data slice.
    pub fn instruction_data(&self, ix: &InstructionMeta) -> &'a [u8] {
        &self.raw[ix.data_offset..ix.data_offset + ix.data_len as usize]
    }

    /// Get an account key referenced by an instruction's account index.
    pub fn instruction_account(
        &self,
        ix: &InstructionMeta,
        account_idx: usize,
    ) -> Option<&'a [u8; 32]> {
        if account_idx >= ix.num_accounts as usize {
            return None;
        }
        let idx_offset = ix.accounts_offset + account_idx;
        if idx_offset >= self.raw.len() {
            return None;
        }
        let key_index = self.raw[idx_offset];
        self.account_key(key_index)
    }
}

/// Parse a Solana legacy message from raw bytes.
///
/// Validates structural invariants:
/// - All account indices in instructions are within bounds
/// - Header arithmetic doesn't overflow account count
/// - No versioned message prefix (0x80)
pub fn parse_legacy_message(data: &[u8]) -> Result<ParsedMessage<'_>, ParseError> {
    if data.is_empty() {
        return Err(ParseError::Eof {
            offset: 0,
            need: 1,
            have: 0,
        });
    }

    // Reject versioned messages
    if data[0] & 0x80 != 0 {
        return Err(ParseError::VersionedMessageNotSupported);
    }

    let mut r = Reader::new(data);

    // Header: 3 bytes
    let num_required_sigs = r.read_u8()?;
    let num_readonly_signed = r.read_u8()?;
    let num_readonly_unsigned = r.read_u8()?;

    // Account keys: compact-u16 length, then N * 32 bytes
    let num_accounts = r.read_compact_u16()?;

    // Structural validation
    if num_required_sigs == 0 || num_required_sigs as u16 > num_accounts {
        return Err(ParseError::InvalidStructure);
    }
    if (num_readonly_signed as u16) + (num_readonly_unsigned as u16) > num_accounts {
        return Err(ParseError::InvalidStructure);
    }

    let accounts_offset = r.position();
    // Skip over account keys
    r.skip(num_accounts as usize * 32)?;

    // Recent blockhash
    let blockhash = r.read_pubkey()?;

    // Instructions: compact-u16 count
    let num_ix = r.read_compact_u16()? as usize;
    if num_ix > MAX_INSTRUCTIONS {
        return Err(ParseError::InvalidStructure);
    }

    let mut instructions = [InstructionMeta::default(); MAX_INSTRUCTIONS];
    for ix in instructions.iter_mut().take(num_ix) {
        // Program ID index
        let program_id_index = r.read_u8()?;
        if program_id_index as u16 >= num_accounts {
            return Err(ParseError::InvalidStructure);
        }

        // Account indexes
        let num_accounts_in_ix = r.read_compact_u16()?;
        let ix_accounts_offset = r.position();

        // Validate each account index is within bounds
        for _ in 0..num_accounts_in_ix {
            let account_idx = r.read_u8()?;
            if account_idx as u16 >= num_accounts {
                return Err(ParseError::InvalidStructure);
            }
        }

        // Instruction data
        let data_len = r.read_compact_u16()?;
        let data_offset = r.position();
        r.skip(data_len as usize)?;

        *ix = InstructionMeta {
            program_id_index,
            accounts_offset: ix_accounts_offset,
            num_accounts: num_accounts_in_ix,
            data_offset,
            data_len,
        };
    }

    Ok(ParsedMessage {
        raw: data,
        num_required_sigs,
        num_readonly_signed,
        num_readonly_unsigned,
        num_accounts,
        accounts_offset,
        blockhash,
        instructions,
        num_instructions: num_ix,
    })
}
