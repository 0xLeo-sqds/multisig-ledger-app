#![no_std]
#![no_main]

mod app_ui {
    pub mod menu;
}
mod crypto;
mod display;
mod handlers {
    pub mod get_public_key;
    pub mod get_version;
    pub mod sign_tx;
}
mod parser;
mod sanitize;
mod settings;

use app_ui::menu::ui_menu_main;
use handlers::{
    get_public_key::handler_get_public_key,
    get_version::handler_get_version,
    sign_tx::{handler_sign_tx, TxContext},
};
use ledger_device_sdk::io::{self, init_comm, ApduHeader, Comm, Command, Reply, StatusWords};
use ledger_device_sdk::nbgl::{NbglReviewStatus, StatusType};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

extern crate alloc;

ledger_device_sdk::define_comm!(COMM);

// APDU instruction codes — compatible with app-solana for transport library reuse
const INS_GET_VERSION: u8 = 0x03;
const INS_GET_APP_CONFIGURATION: u8 = 0x04;
const INS_GET_PUBKEY: u8 = 0x05;
const INS_SIGN_TRANSACTION: u8 = 0x06;

/// Application status words.
///
/// 0x9000          = success
/// 0x69xx          = ISO 7816 standard errors
/// 0x6Axx/6Bxx/6D/6E = protocol errors
/// 0x6808          = blind signing disabled (Solana convention)
/// 0xB0xx          = app-specific errors
#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum AppSW {
    Ok = 0x9000,
    Deny = 0x6985,
    WrongP1P2 = 0x6A86,
    WrongApduLength = StatusWords::BadLen as u16,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    CommError = 0x6F00,
    BlindSigningDisabled = 0x6808,
    InvalidData = 0x6A80,
    TxDisplayFail = 0xB001,
    AddrDisplayFail = 0xB002,
    TxParsingFail = 0xB005,
    TxSignFail = 0xB008,
    KeyDeriveFail = 0xB009,
    VersionParsingFail = 0xB00A,
    InvalidMessage = 0xB00B,
}

impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}

impl From<io::CommError> for AppSW {
    fn from(_e: io::CommError) -> Self {
        AppSW::CommError
    }
}

/// Decoded APDU instruction.
#[derive(Debug)]
pub enum Instruction {
    GetVersion,
    GetAppConfiguration,
    GetPubkey { display: bool },
    SignTx { is_first: bool, more: bool },
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = AppSW;

    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match (value.ins, value.p1, value.p2) {
            (INS_GET_VERSION, 0, 0) => Ok(Instruction::GetVersion),
            (INS_GET_APP_CONFIGURATION, 0, 0) => Ok(Instruction::GetAppConfiguration),
            (INS_GET_PUBKEY, 0 | 1, 0) => Ok(Instruction::GetPubkey {
                display: value.p1 != 0,
            }),
            // Sign transaction: P1 unused (reserved), P2 bit flags for chunking
            // P2 & 0x01 = EXTEND (continuation), P2 & 0x02 = MORE (more chunks follow)
            (INS_SIGN_TRANSACTION, _, p2) => {
                let extend = p2 & 0x01 != 0;
                let more = p2 & 0x02 != 0;
                Ok(Instruction::SignTx {
                    is_first: !extend,
                    more,
                })
            }
            (INS_GET_VERSION..=INS_SIGN_TRANSACTION, _, _) => Err(AppSW::WrongP1P2),
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}

fn show_status_and_home_if_needed(
    comm: &mut Comm,
    ins: &Instruction,
    tx_ctx: &TxContext,
    status: &AppSW,
) {
    let show_status = match (ins, status) {
        (Instruction::GetPubkey { display: true }, AppSW::Deny | AppSW::Ok) => true,
        (Instruction::SignTx { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.finished => true,
        _ => false,
    };

    if show_status {
        let success = *status == AppSW::Ok;
        let status_type = match ins {
            Instruction::GetPubkey { .. } => StatusType::Address,
            _ => StatusType::Transaction,
        };
        NbglReviewStatus::new()
            .status_type(status_type)
            .show(comm, success);
        tx_ctx.home.show_and_return();
    }
}

#[no_mangle]
extern "C" fn sample_main(_arg0: u32) {
    let comm = init_comm(&COMM);
    comm.set_expected_cla(0xe0);

    let mut tx_ctx = TxContext::new();
    tx_ctx.home = ui_menu_main(comm);
    tx_ctx.home.show_and_return();

    loop {
        let command = comm.next_command();
        let decoded = command.decode::<Instruction>();
        let Ok(ins) = decoded else {
            let _ = comm.send(&[], decoded.unwrap_err());
            continue;
        };

        let status = match handle_apdu(command, &ins, &mut tx_ctx) {
            Ok(reply) => {
                let _ = reply.send(AppSW::Ok);
                AppSW::Ok
            }
            Err(sw) => {
                let _ = comm.send(&[], sw);
                sw
            }
        };
        show_status_and_home_if_needed(comm, &ins, &tx_ctx, &status);
    }
}

fn handle_apdu<'a>(
    command: Command<'a>,
    ins: &Instruction,
    ctx: &mut TxContext,
) -> Result<io::CommandResponse<'a>, AppSW> {
    match ins {
        Instruction::GetVersion => handler_get_version(command),
        Instruction::GetAppConfiguration => {
            let blind = settings::Settings.get_element(0);
            let mut response = command.into_response();
            response.append(&[blind])?; // blind signing enabled
            response.append(&[0])?; // reserved
            // version
            let (major, minor, patch) =
                handlers::get_version::parse_version_string(env!("CARGO_PKG_VERSION"))
                    .unwrap_or((0, 0, 0));
            response.append(&[major, minor, patch])?;
            Ok(response)
        }
        Instruction::GetPubkey { display } => handler_get_public_key(command, *display),
        Instruction::SignTx { is_first, more } => {
            handler_sign_tx(command, *is_first, *more, ctx)
        }
    }
}
