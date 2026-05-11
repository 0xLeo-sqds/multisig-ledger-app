/// Format a 32-byte pubkey as base58 into the provided buffer.
/// Returns the number of bytes written.
///
/// Solana pubkeys are 32 bytes, so they fit in at most 44 base58 characters.
/// The output buffer must be at least 45 bytes.
pub fn format_base58(pubkey: &[u8; 32], out: &mut [u8]) -> Result<usize, ()> {
    let s = bs58::encode(pubkey).into_string();
    let bytes = s.as_bytes();
    let len = bytes.len().min(out.len());
    out[..len].copy_from_slice(&bytes[..len]);
    Ok(len)
}
