//! Native unit tests for the parser logic.
//! These test the zero-copy parser, Solana message parsing, and Squads
//! instruction identification WITHOUT the Ledger SDK.
//! Run with: cargo test --target aarch64-apple-darwin --test native_parser_test

// We can't import the app crate (it's no_std + Ledger-specific),
// so we duplicate the pure parsing logic here for testing.
// In production, this would be a shared `squads-parser` crate.

/// Minimal Reader — mirrors src/parser/mod.rs
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[derive(Debug, PartialEq)]
enum ParseError {
    Eof,
    InvalidStructure,
    InvalidDiscriminator,
    VersionedNotSupported,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self { Self { data, pos: 0 } }
    fn remaining(&self) -> usize { self.data.len().saturating_sub(self.pos) }
    fn position(&self) -> usize { self.pos }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        if self.remaining() < n { return Err(ParseError::Eof); }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn skip(&mut self, n: usize) -> Result<(), ParseError> {
        if self.remaining() < n { return Err(ParseError::Eof); }
        self.pos += n;
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8, ParseError> { Ok(self.read_bytes(1)?[0]) }

    fn read_u16_le(&mut self) -> Result<u16, ParseError> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn read_u32_le(&mut self) -> Result<u32, ParseError> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_u64_le(&mut self) -> Result<u64, ParseError> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }

    fn read_compact_u16(&mut self) -> Result<u16, ParseError> {
        let first = self.read_u8()? as u16;
        if first < 0x80 { return Ok(first); }
        let second = self.read_u8()? as u16;
        if second < 0x80 { return Ok((first & 0x7f) | (second << 7)); }
        let third = self.read_u8()? as u16;
        Ok((first & 0x7f) | ((second & 0x7f) << 7) | (third << 14))
    }

    fn read_pubkey(&mut self) -> Result<&'a [u8; 32], ParseError> {
        let b = self.read_bytes(32)?;
        b.try_into().map_err(|_| ParseError::InvalidStructure)
    }
}

// --- Solana message parser (mirrors src/parser/solana_message.rs) ---

const MAX_INSTRUCTIONS: usize = 32;

#[derive(Clone, Copy, Default, Debug)]
struct InstructionMeta {
    program_id_index: u8,
    accounts_offset: usize,
    num_accounts: u16,
    data_offset: usize,
    data_len: u16,
}

#[derive(Debug)]
struct ParsedMessage<'a> {
    raw: &'a [u8],
    num_required_sigs: u8,
    num_readonly_signed: u8,
    num_readonly_unsigned: u8,
    num_accounts: u16,
    accounts_offset: usize,
    instructions: [InstructionMeta; MAX_INSTRUCTIONS],
    num_instructions: usize,
}

impl<'a> ParsedMessage<'a> {
    fn account_key(&self, index: u8) -> Option<&'a [u8; 32]> {
        if (index as u16) >= self.num_accounts { return None; }
        let offset = self.accounts_offset + (index as usize) * 32;
        if offset + 32 > self.raw.len() { return None; }
        self.raw[offset..offset + 32].try_into().ok()
    }

    fn program_id(&self, ix: &InstructionMeta) -> Option<&'a [u8; 32]> {
        self.account_key(ix.program_id_index)
    }

    fn instruction_data(&self, ix: &InstructionMeta) -> &'a [u8] {
        &self.raw[ix.data_offset..ix.data_offset + ix.data_len as usize]
    }
}

fn parse_legacy_message(data: &[u8]) -> Result<ParsedMessage<'_>, ParseError> {
    if data.is_empty() { return Err(ParseError::Eof); }
    if data[0] & 0x80 != 0 { return Err(ParseError::VersionedNotSupported); }

    let mut r = Reader::new(data);
    let num_required_sigs = r.read_u8()?;
    let num_readonly_signed = r.read_u8()?;
    let num_readonly_unsigned = r.read_u8()?;
    let num_accounts = r.read_compact_u16()?;

    if num_required_sigs == 0 || num_required_sigs as u16 > num_accounts {
        return Err(ParseError::InvalidStructure);
    }
    if (num_readonly_signed as u16) + (num_readonly_unsigned as u16) > num_accounts {
        return Err(ParseError::InvalidStructure);
    }

    let accounts_offset = r.position();
    r.skip(num_accounts as usize * 32)?;
    let _blockhash = r.read_pubkey()?;

    let num_ix = r.read_compact_u16()? as usize;
    if num_ix > MAX_INSTRUCTIONS { return Err(ParseError::InvalidStructure); }

    let mut instructions = [InstructionMeta::default(); MAX_INSTRUCTIONS];
    for ix in instructions.iter_mut().take(num_ix) {
        let program_id_index = r.read_u8()?;
        if program_id_index as u16 >= num_accounts { return Err(ParseError::InvalidStructure); }

        let num_accts = r.read_compact_u16()?;
        let ix_accounts_offset = r.position();
        for _ in 0..num_accts {
            let idx = r.read_u8()?;
            if idx as u16 >= num_accounts { return Err(ParseError::InvalidStructure); }
        }

        let data_len = r.read_compact_u16()?;
        let data_offset = r.position();
        r.skip(data_len as usize)?;

        *ix = InstructionMeta {
            program_id_index,
            accounts_offset: ix_accounts_offset,
            num_accounts: num_accts,
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
        instructions,
        num_instructions: num_ix,
    })
}

// --- Squads discriminators (mirrors src/parser/squads.rs) ---
const DISC_PROPOSAL_APPROVE: [u8; 8] = [0x90, 0x25, 0xa4, 0x88, 0xbc, 0xd8, 0x2a, 0xf8];
const DISC_PROPOSAL_REJECT: [u8; 8] = [0xf3, 0x3e, 0x86, 0x9c, 0xe6, 0x6a, 0xf6, 0x87];
const DISC_PROPOSAL_CANCEL: [u8; 8] = [0x1b, 0x2a, 0x7f, 0xed, 0x26, 0xa3, 0x54, 0xcb];
const DISC_VAULT_TX_CREATE: [u8; 8] = [0x30, 0xfa, 0x4e, 0xa8, 0xd0, 0xe2, 0xda, 0xd3];
const DISC_CONFIG_TX_CREATE: [u8; 8] = [0x9b, 0xec, 0x57, 0xe4, 0x89, 0x4b, 0x51, 0x27];

const SQUADS_V4_PROGRAM_ID: [u8; 32] = [
    0x06, 0x81, 0xc4, 0xce, 0x47, 0xe2, 0x23, 0x68, 0xb8, 0xb1, 0x55, 0x5e, 0xc8, 0x87, 0xaf,
    0x09, 0x2e, 0xfc, 0x7e, 0xfb, 0xb6, 0x6c, 0xa3, 0xf5, 0x2f, 0xbf, 0x68, 0xd4, 0xac, 0x9c,
    0xb7, 0xa8,
];

fn identify_squads_instruction(data: &[u8]) -> &'static str {
    if data.len() < 8 { return "unknown"; }
    let disc: [u8; 8] = data[..8].try_into().unwrap();
    match disc {
        DISC_PROPOSAL_APPROVE => "proposal_approve",
        DISC_PROPOSAL_REJECT => "proposal_reject",
        DISC_PROPOSAL_CANCEL => "proposal_cancel",
        DISC_VAULT_TX_CREATE => "vault_transaction_create",
        DISC_CONFIG_TX_CREATE => "config_transaction_create",
        _ => "unknown",
    }
}

// --- Helper to build a Solana legacy message ---
fn build_legacy_message(
    num_required_sigs: u8,
    num_readonly_signed: u8,
    num_readonly_unsigned: u8,
    account_keys: &[[u8; 32]],
    blockhash: [u8; 32],
    instructions: &[(u8, &[u8], &[u8])], // (program_id_index, account_indexes, data)
) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.push(num_required_sigs);
    msg.push(num_readonly_signed);
    msg.push(num_readonly_unsigned);
    // compact-u16 for account count (simple case: < 128)
    msg.push(account_keys.len() as u8);
    for key in account_keys { msg.extend_from_slice(key); }
    msg.extend_from_slice(&blockhash);
    // compact-u16 for instruction count
    msg.push(instructions.len() as u8);
    for (pid_idx, acct_idxs, data) in instructions {
        msg.push(*pid_idx);
        msg.push(acct_idxs.len() as u8);
        msg.extend_from_slice(acct_idxs);
        msg.push(data.len() as u8);
        msg.extend_from_slice(data);
    }
    msg
}

// ============== TESTS ==============

#[test]
fn test_reader_basic() {
    let data = [1u8, 2, 3, 4, 5];
    let mut r = Reader::new(&data);
    assert_eq!(r.read_u8().unwrap(), 1);
    assert_eq!(r.position(), 1);
    assert_eq!(r.remaining(), 4);
}

#[test]
fn test_reader_eof() {
    let data = [0u8; 3];
    let mut r = Reader::new(&data);
    assert!(r.read_bytes(4).is_err());
}

#[test]
fn test_reader_u64() {
    let data = 42u64.to_le_bytes();
    let mut r = Reader::new(&data);
    assert_eq!(r.read_u64_le().unwrap(), 42);
}

#[test]
fn test_reader_compact_u16_single_byte() {
    let mut r = Reader::new(&[42]);
    assert_eq!(r.read_compact_u16().unwrap(), 42);
}

#[test]
fn test_reader_compact_u16_two_bytes() {
    // 200 = 0xC8 → first byte = (200 & 0x7f) | 0x80 = 0xC8, second = 200 >> 7 = 1
    let mut r = Reader::new(&[0xC8, 0x01]);
    assert_eq!(r.read_compact_u16().unwrap(), 200);
}

#[test]
fn test_reject_versioned_message() {
    let data = [0x80, 0, 0, 0]; // v0 prefix
    assert_eq!(parse_legacy_message(&data).unwrap_err(), ParseError::VersionedNotSupported);
}

#[test]
fn test_reject_empty_message() {
    assert_eq!(parse_legacy_message(&[]).unwrap_err(), ParseError::Eof);
}

#[test]
fn test_reject_zero_signers() {
    // num_required_sigs = 0
    let msg = build_legacy_message(0, 0, 0, &[[0; 32]], [0; 32], &[]);
    assert_eq!(parse_legacy_message(&msg).unwrap_err(), ParseError::InvalidStructure);
}

#[test]
fn test_reject_signers_exceed_accounts() {
    // num_required_sigs = 5 but only 2 accounts
    let msg = build_legacy_message(5, 0, 0, &[[0; 32], [1; 32]], [0; 32], &[]);
    assert_eq!(parse_legacy_message(&msg).unwrap_err(), ParseError::InvalidStructure);
}

#[test]
fn test_reject_readonly_exceed_accounts() {
    // 1 account but readonly_signed(1) + readonly_unsigned(1) = 2 > 1
    let msg = build_legacy_message(1, 1, 1, &[[0; 32]], [0; 32], &[]);
    assert_eq!(parse_legacy_message(&msg).unwrap_err(), ParseError::InvalidStructure);
}

#[test]
fn test_parse_valid_empty_message() {
    let msg = build_legacy_message(1, 0, 0, &[[0; 32]], [0; 32], &[]);
    let parsed = parse_legacy_message(&msg).unwrap();
    assert_eq!(parsed.num_required_sigs, 1);
    assert_eq!(parsed.num_accounts, 1);
    assert_eq!(parsed.num_instructions, 0);
}

#[test]
fn test_parse_message_with_instruction() {
    let signer = [1u8; 32];
    let program = SQUADS_V4_PROGRAM_ID;
    let ix_data = DISC_PROPOSAL_APPROVE;

    let msg = build_legacy_message(
        1, 0, 1,
        &[signer, program],
        [0; 32],
        &[(1, &[0], &ix_data)], // program_id_index=1, account_indexes=[0], data=discriminator
    );

    let parsed = parse_legacy_message(&msg).unwrap();
    assert_eq!(parsed.num_instructions, 1);
    assert_eq!(parsed.num_accounts, 2);

    let ix = &parsed.instructions[0];
    assert_eq!(ix.program_id_index, 1);
    assert_eq!(*parsed.program_id(ix).unwrap(), SQUADS_V4_PROGRAM_ID);
    assert_eq!(parsed.instruction_data(ix), &DISC_PROPOSAL_APPROVE);
}

#[test]
fn test_reject_invalid_program_id_index() {
    let msg = build_legacy_message(
        1, 0, 0,
        &[[0; 32]],
        [0; 32],
        &[(5, &[], &[])], // program_id_index=5 but only 1 account
    );
    assert_eq!(parse_legacy_message(&msg).unwrap_err(), ParseError::InvalidStructure);
}

#[test]
fn test_reject_invalid_account_index_in_instruction() {
    let msg = build_legacy_message(
        1, 0, 1,
        &[[0; 32], [1; 32]],
        [0; 32],
        &[(1, &[99], &[])], // account_index=99 but only 2 accounts
    );
    assert_eq!(parse_legacy_message(&msg).unwrap_err(), ParseError::InvalidStructure);
}

#[test]
fn test_identify_squads_instructions() {
    assert_eq!(identify_squads_instruction(&DISC_PROPOSAL_APPROVE), "proposal_approve");
    assert_eq!(identify_squads_instruction(&DISC_PROPOSAL_REJECT), "proposal_reject");
    assert_eq!(identify_squads_instruction(&DISC_PROPOSAL_CANCEL), "proposal_cancel");
    assert_eq!(identify_squads_instruction(&DISC_VAULT_TX_CREATE), "vault_transaction_create");
    assert_eq!(identify_squads_instruction(&DISC_CONFIG_TX_CREATE), "config_transaction_create");
    assert_eq!(identify_squads_instruction(&[0xFF; 8]), "unknown");
    assert_eq!(identify_squads_instruction(&[0, 1, 2]), "unknown"); // too short
}

#[test]
fn test_squads_instruction_in_message() {
    let signer = [1u8; 32];
    let multisig = [2u8; 32];
    let proposal = [3u8; 32];
    let program = SQUADS_V4_PROGRAM_ID;

    let msg = build_legacy_message(
        1, 0, 3,
        &[signer, multisig, proposal, program],
        [0; 32],
        &[(3, &[0, 1, 2], &DISC_PROPOSAL_APPROVE)],
    );

    let parsed = parse_legacy_message(&msg).unwrap();
    let ix = &parsed.instructions[0];
    let pid = parsed.program_id(ix).unwrap();
    assert_eq!(*pid, SQUADS_V4_PROGRAM_ID);

    let ix_data = parsed.instruction_data(ix);
    assert_eq!(identify_squads_instruction(ix_data), "proposal_approve");
}

#[test]
fn test_vault_tx_create_parsing() {
    // Build a minimal vault_transaction_create instruction data
    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&DISC_VAULT_TX_CREATE); // discriminator
    ix_data.push(0); // vault_index
    ix_data.push(0); // reserved

    // Inner TransactionMessage (Borsh: u32 lengths)
    let inner_msg_start = ix_data.len() + 4; // after message_len
    let mut inner_msg = Vec::new();
    inner_msg.push(1); // num_signers
    inner_msg.push(1); // num_writable_signers
    inner_msg.push(0); // num_writable_non_signers
    inner_msg.extend_from_slice(&1u32.to_le_bytes()); // 1 account key
    inner_msg.extend_from_slice(&[4u8; 32]); // account key
    inner_msg.extend_from_slice(&1u32.to_le_bytes()); // 1 instruction
    inner_msg.push(0); // program_id_index
    inner_msg.extend_from_slice(&0u32.to_le_bytes()); // 0 account indexes
    inner_msg.extend_from_slice(&4u32.to_le_bytes()); // 4 bytes data
    inner_msg.extend_from_slice(&[2, 0, 0, 0]); // System Transfer disc
    inner_msg.extend_from_slice(&0u32.to_le_bytes()); // 0 lookups

    ix_data.extend_from_slice(&(inner_msg.len() as u32).to_le_bytes());
    ix_data.extend_from_slice(&inner_msg);

    assert_eq!(identify_squads_instruction(&ix_data), "vault_transaction_create");

    // Parse the inner message
    let mut r = Reader::new(&ix_data);
    r.skip(8).unwrap(); // disc
    let vault_index = r.read_u8().unwrap();
    assert_eq!(vault_index, 0);
    r.skip(1).unwrap(); // reserved
    let msg_len = r.read_u32_le().unwrap();
    assert_eq!(msg_len as usize, inner_msg.len());

    // Parse inner message header
    let num_signers = r.read_u8().unwrap();
    assert_eq!(num_signers, 1);
    let _num_w_signers = r.read_u8().unwrap();
    let _num_w_non_signers = r.read_u8().unwrap();
    let num_keys = r.read_u32_le().unwrap();
    assert_eq!(num_keys, 1);
}

#[test]
fn test_config_action_parsing() {
    // ChangeThreshold action: tag=2, new_threshold=3
    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&DISC_CONFIG_TX_CREATE);
    ix_data.extend_from_slice(&1u32.to_le_bytes()); // 1 action
    ix_data.push(2); // ChangeThreshold tag
    ix_data.extend_from_slice(&3u16.to_le_bytes()); // threshold = 3

    let mut r = Reader::new(&ix_data);
    r.skip(8).unwrap(); // disc
    let action_count = r.read_u32_le().unwrap();
    assert_eq!(action_count, 1);
    let tag = r.read_u8().unwrap();
    assert_eq!(tag, 2); // ChangeThreshold
    let threshold = r.read_u16_le().unwrap();
    assert_eq!(threshold, 3);
}

#[test]
fn test_config_action_add_member() {
    let mut ix_data = Vec::new();
    ix_data.extend_from_slice(&DISC_CONFIG_TX_CREATE);
    ix_data.extend_from_slice(&1u32.to_le_bytes()); // 1 action
    ix_data.push(0); // AddMember tag
    ix_data.extend_from_slice(&[0xAA; 32]); // member pubkey
    ix_data.push(0b111); // permissions: initiate + vote + execute

    let mut r = Reader::new(&ix_data);
    r.skip(8).unwrap();
    let count = r.read_u32_le().unwrap();
    assert_eq!(count, 1);
    let tag = r.read_u8().unwrap();
    assert_eq!(tag, 0); // AddMember
    let member = r.read_pubkey().unwrap();
    assert_eq!(member, &[0xAA; 32]);
    let perms = r.read_u8().unwrap();
    assert_eq!(perms, 0b111);
}

#[test]
fn test_sanitize_ascii() {
    fn sanitize(input: &[u8], out: &mut [u8; 64]) -> usize {
        let mut pos = 0;
        for &b in input {
            if pos >= 64 { break; }
            if (0x20..=0x7E).contains(&b) {
                out[pos] = b;
                pos += 1;
            }
        }
        pos
    }

    let mut buf = [0u8; 64];

    // Normal ASCII
    let len = sanitize(b"Hello World", &mut buf);
    assert_eq!(&buf[..len], b"Hello World");

    // Strip control chars
    let len = sanitize(b"Hello\x00\x01\x02World", &mut buf);
    assert_eq!(&buf[..len], b"HelloWorld");

    // Strip bidi override (U+202E = 0xE2 0x80 0xAE in UTF-8, all > 0x7E)
    let input = b"abc\xe2\x80\xaedef";
    let len = sanitize(input, &mut buf);
    assert_eq!(&buf[..len], b"abcdef");

    // Truncate at 64
    let long_input = [b'A'; 100];
    let len = sanitize(&long_input, &mut buf);
    assert_eq!(len, 64);
}

#[test]
fn test_base58_encode() {
    // System program (all zeros) = "11111111111111111111111111111111"
    let pubkey = [0u8; 32];
    let mut out = [0u8; 45];
    let len = bs58::encode(&pubkey).into_string();
    let addr = &len;
    assert_eq!(addr, "11111111111111111111111111111111");
}

#[test]
fn test_base58_squads_program_id() {
    let mut out = [0u8; 45];
    let len = bs58::encode(&SQUADS_V4_PROGRAM_ID).into_string();
    let addr = &len;
    assert_eq!(addr, "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
}

#[test]
fn test_amount_formatting() {
    // 1.5 SOL = 1_500_000_000 lamports
    let raw: u64 = 1_500_000_000;
    let decimals: u8 = 9;
    let divisor = 10u64.pow(decimals as u32);
    let integer = raw / divisor;
    let frac = raw % divisor;
    let formatted = format!("{}.{:0>width$} SOL", integer, frac, width = decimals as usize);
    assert_eq!(formatted, "1.500000000 SOL");

    // u64::MAX (boundary test)
    let max: u64 = u64::MAX;
    let integer = max / divisor;
    let frac = max % divisor;
    let formatted = format!("{}.{:0>width$}", integer, frac, width = decimals as usize);
    assert!(formatted.starts_with("18446744073."));
}

#[test]
fn test_bip32_path_validation() {
    const HARDENED: u32 = 0x8000_0000;

    fn validate_path(components: &[u32]) -> bool {
        if components.len() < 3 || components.len() > 4 { return false; }
        if components.iter().any(|c| c & HARDENED == 0) { return false; }
        if components[0] != (44 | HARDENED) { return false; }
        if components[1] != (501 | HARDENED) { return false; }
        true
    }

    assert!(validate_path(&[44 | HARDENED, 501 | HARDENED, 0 | HARDENED]));
    assert!(validate_path(&[44 | HARDENED, 501 | HARDENED, 0 | HARDENED, 0 | HARDENED]));
    assert!(!validate_path(&[44 | HARDENED, 501 | HARDENED])); // too few
    assert!(!validate_path(&[44 | HARDENED, 60 | HARDENED, 0 | HARDENED])); // wrong coin
    assert!(!validate_path(&[45 | HARDENED, 501 | HARDENED, 0 | HARDENED])); // wrong purpose
    assert!(!validate_path(&[44 | HARDENED, 501 | HARDENED, 0])); // not hardened
}
