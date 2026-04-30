use arrayvec::ArrayString;
use core::fmt::Write;

/// Format a 32-byte pubkey as base58 into the provided buffer.
/// Returns the number of bytes written.
///
/// Solana pubkeys are 32 bytes → max 44 base58 characters.
/// The output buffer must be at least 45 bytes.
pub fn format_base58(pubkey: &[u8; 32], out: &mut [u8]) -> Result<usize, ()> {
    // bs58 in no_std mode: use encode().onto() which writes to a &mut Vec<u8>
    // or encode().as_ref() which gives &[u8].
    // Since we need no_alloc, we use a manual approach via the alphabet.
    //
    // Actually, let's use ArrayString and write! which works in no_std:
    let mut buf = ArrayString::<45>::new();
    // bs58 encode into an ArrayString isn't directly supported.
    // Use the alloc feature path: encode().into_string() then copy.
    let s = bs58::encode(pubkey).into_string();
    let bytes = s.as_bytes();
    let len = bytes.len().min(out.len());
    out[..len].copy_from_slice(&bytes[..len]);
    Ok(len)
}
