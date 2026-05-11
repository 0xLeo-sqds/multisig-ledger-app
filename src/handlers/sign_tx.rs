use crate::crypto::{sign_message, Bip32Path};
use crate::parser::solana_message;
use crate::parser::squads;
use crate::AppSW;
use ledger_device_sdk::io::{Command, CommandResponse};
use ledger_device_sdk::nbgl::NbglHomeAndSettings;

/// Maximum transaction buffer size.
/// Solana max tx = 1232 bytes. Add room for APDU framing overhead.
const MAX_TX_LEN: usize = 1300;

/// Transaction signing state machine.
#[derive(Clone, Copy, PartialEq)]
enum TxState {
    /// No signing session active.
    Idle,
    /// Receiving chunked APDU data.
    Receiving,
}

/// Transaction context persists across APDU exchanges during signing.
pub struct TxContext {
    state: TxState,
    /// Raw transaction buffer (static, not heap-allocated).
    buf: [u8; MAX_TX_LEN],
    /// Number of bytes received so far.
    buf_len: usize,
    /// BIP32 derivation path (parsed from first chunk).
    path: Option<Bip32Path>,
    /// Whether the signing flow completed (for status display).
    pub finished: bool,
    /// Home screen reference.
    pub home: NbglHomeAndSettings,
}

impl TxContext {
    pub fn new() -> Self {
        Self {
            state: TxState::Idle,
            buf: [0u8; MAX_TX_LEN],
            buf_len: 0,
            path: None,
            finished: false,
            home: NbglHomeAndSettings::new(),
        }
    }

    /// Reset the signing session, zeroing the buffer.
    fn reset(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.buf);
        self.buf_len = 0;
        self.path = None;
        self.state = TxState::Idle;
        self.finished = false;
    }
}

pub fn handler_sign_tx<'a>(
    command: Command<'a>,
    is_first: bool,
    more: bool,
    ctx: &mut TxContext,
) -> Result<CommandResponse<'a>, AppSW> {
    let data = command.get_data();

    if is_first {
        // First chunk: reset state, parse path, begin receiving
        ctx.reset();
        ctx.state = TxState::Receiving;

        // Parse BIP32 path from the beginning of the data
        if data.is_empty() {
            ctx.reset();
            return Err(AppSW::WrongApduLength);
        }
        let num_components = data[0] as usize;
        let path_len = 1 + num_components * 4;
        if data.len() < path_len {
            ctx.reset();
            return Err(AppSW::WrongApduLength);
        }

        let path: Bip32Path = data[..path_len].try_into().map_err(|e| {
            ctx.reset();
            e
        })?;
        ctx.path = Some(path);

        // Remaining bytes are the start of the transaction message
        let tx_data = &data[path_len..];
        if ctx.buf_len + tx_data.len() > MAX_TX_LEN {
            ctx.reset();
            return Err(AppSW::InvalidData);
        }
        ctx.buf[ctx.buf_len..ctx.buf_len + tx_data.len()].copy_from_slice(tx_data);
        ctx.buf_len += tx_data.len();
    } else {
        // Continuation chunk
        if ctx.state != TxState::Receiving {
            ctx.reset();
            return Err(AppSW::InvalidData);
        }
        if ctx.buf_len + data.len() > MAX_TX_LEN {
            ctx.reset();
            return Err(AppSW::InvalidData);
        }
        ctx.buf[ctx.buf_len..ctx.buf_len + data.len()].copy_from_slice(data);
        ctx.buf_len += data.len();
    }

    if more {
        // More chunks expected — acknowledge and wait
        return Ok(command.into_response());
    }

    // All chunks received — parse, display, and sign
    let buf_len = ctx.buf_len;

    // Check for versioned (v0) message — reject with clear error
    if buf_len > 0 && ctx.buf[0] & 0x80 != 0 {
        ctx.reset();
        return Err(AppSW::InvalidMessage);
    }

    // Parse the Solana legacy message (borrow buf for the duration of parsing + display)
    let parse_result = solana_message::parse_legacy_message(&ctx.buf[..buf_len]);
    let parsed = match parse_result {
        Ok(p) => p,
        Err(_) => {
            ctx.reset();
            return Err(AppSW::TxParsingFail);
        }
    };

    // Identify Squads instructions and display for user review
    let comm = command.into_comm();
    let review_result = squads::review_transaction(comm, &parsed, &ctx.buf[..buf_len]);
    let approved = match review_result {
        Ok(a) => a,
        Err(_) => {
            ctx.reset();
            return Err(AppSW::TxDisplayFail);
        }
    };

    if !approved {
        ctx.reset();
        return Err(AppSW::Deny);
    }

    // Extract path before signing (avoids borrow conflict)
    let path = match ctx.path.as_ref() {
        Some(p) => p,
        None => {
            ctx.reset();
            return Err(AppSW::TxSignFail);
        }
    };

    // Sign the raw message bytes
    let signature = match sign_message(path, &ctx.buf[..buf_len]) {
        Ok(s) => s,
        Err(e) => {
            ctx.reset();
            return Err(e);
        }
    };

    ctx.finished = true;
    let mut response = comm.begin_response();
    response.append(&signature)?;

    // Reset after successful signing
    ctx.reset();

    Ok(response)
}
