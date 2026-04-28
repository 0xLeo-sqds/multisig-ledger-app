use crate::AppSW;
use core::str::FromStr;
use ledger_device_sdk::io::{Command, CommandResponse};

pub fn handler_get_version(command: Command<'_>) -> Result<CommandResponse<'_>, AppSW> {
    if let Some((major, minor, patch)) = parse_version_string(env!("CARGO_PKG_VERSION")) {
        let mut response = command.into_response();
        response.append(&[major, minor, patch])?;
        Ok(response)
    } else {
        Err(AppSW::VersionParsingFail)
    }
}

pub fn parse_version_string(input: &str) -> Option<(u8, u8, u8)> {
    let mut parts = input.split('.');
    let major = u8::from_str(parts.next()?).ok()?;
    let minor = u8::from_str(parts.next()?).ok()?;
    let patch = u8::from_str(parts.next()?).ok()?;
    Some((major, minor, patch))
}
