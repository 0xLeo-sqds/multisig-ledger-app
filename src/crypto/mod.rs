use crate::AppSW;
use ledger_device_sdk::ecc::{CurvesId, Ed25519, SeedDerive};

/// BIP32 path: 44'/501'/account'/change'
/// Validates: 3 or 4 components, all hardened, first two must be 44' and 501'.
pub struct Bip32Path {
    path: [u32; 5],
    len: usize,
}

const BIP32_HARDENED: u32 = 0x8000_0000;
const BIP32_PURPOSE: u32 = 44 | BIP32_HARDENED;
const BIP32_COIN_SOL: u32 = 501 | BIP32_HARDENED;

impl Bip32Path {
    pub fn as_slice(&self) -> &[u32] {
        &self.path[..self.len]
    }
}

impl TryFrom<&[u8]> for Bip32Path {
    type Error = AppSW;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(AppSW::WrongApduLength);
        }

        let num_components = data[0] as usize;
        // Must have 3 or 4 components, each 4 bytes
        if !(3..=4).contains(&num_components) || data.len() != 1 + num_components * 4 {
            return Err(AppSW::WrongApduLength);
        }

        let mut path = [0u32; 5];
        for i in 0..num_components {
            let offset = 1 + i * 4;
            path[i] = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);

            // All components must be hardened
            if path[i] & BIP32_HARDENED == 0 {
                return Err(AppSW::WrongP1P2);
            }
        }

        // First component must be 44' (purpose)
        if path[0] != BIP32_PURPOSE {
            return Err(AppSW::WrongP1P2);
        }
        // Second component must be 501' (Solana coin type)
        if path[1] != BIP32_COIN_SOL {
            return Err(AppSW::WrongP1P2);
        }

        Ok(Bip32Path {
            path,
            len: num_components,
        })
    }
}

/// Derive Ed25519 public key from BIP32 path.
/// Returns 32-byte compressed public key.
///
/// Uses bip32_derive which gives us both private seed and public key bytes
/// for Ed25519 via SLIP-10. The public key occupies bytes [32..64] of the
/// 64-byte output buffer.
pub fn derive_pubkey(path: &Bip32Path) -> Result<[u8; 32], AppSW> {
    let mut raw_key = [0u8; 64];
    ledger_device_sdk::ecc::bip32_derive(
        CurvesId::Ed25519,
        path.as_slice(),
        &mut raw_key,
        None,
    )
    .map_err(|_| AppSW::KeyDeriveFail)?;

    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&raw_key[32..64]);

    // Zeroize private key material (bytes 0..32)
    zeroize::Zeroize::zeroize(&mut raw_key);

    Ok(pubkey)
}

/// Sign a message with Ed25519 derived from the given path.
/// Returns 64-byte signature. Key material is handled by the SDK's
/// secure element, which clears it after the operation.
pub fn sign_message(path: &Bip32Path, message: &[u8]) -> Result<[u8; 64], AppSW> {
    let private_key = Ed25519::derive_from_path(path.as_slice());
    let (sig_buf, sig_len) = private_key.sign(message).map_err(|_| AppSW::TxSignFail)?;

    if (sig_len as usize) < 64 {
        return Err(AppSW::TxSignFail);
    }

    let mut signature = [0u8; 64];
    signature.copy_from_slice(&sig_buf[..64]);
    Ok(signature)
}
