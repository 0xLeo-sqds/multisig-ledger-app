use crate::crypto::{derive_pubkey, Bip32Path};
use crate::display::address::format_base58;
use crate::AppSW;
use ledger_device_sdk::io::{Command, CommandResponse};
use ledger_device_sdk::nbgl::NbglAddressReview;

pub fn handler_get_public_key(
    command: Command<'_>,
    display: bool,
) -> Result<CommandResponse<'_>, AppSW> {
    let data = command.get_data();
    let path: Bip32Path = data.try_into()?;
    let pubkey = derive_pubkey(&path)?;

    let comm = command.into_comm();

    if display {
        // Format pubkey as base58 for display
        let mut addr_buf = [0u8; 45];
        let addr_len = format_base58(&pubkey, &mut addr_buf).map_err(|_| AppSW::AddrDisplayFail)?;
        let addr_str =
            core::str::from_utf8(&addr_buf[..addr_len]).map_err(|_| AppSW::AddrDisplayFail)?;

        let confirmed = NbglAddressReview::new()
            .review_title("Verify Solana\naddress")
            .show(comm, addr_str);

        if !confirmed {
            return Err(AppSW::Deny);
        }
    }

    let mut response = comm.begin_response();
    response.append(&[pubkey.len() as u8])?;
    response.append(&pubkey)?;
    Ok(response)
}
