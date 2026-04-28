/// Format a 32-byte pubkey as base58 into the provided buffer.
/// Returns the number of bytes written.
///
/// Solana pubkeys are 32 bytes → max 44 base58 characters.
/// The output buffer must be at least 45 bytes.
pub fn format_base58(pubkey: &[u8; 32], out: &mut [u8]) -> Result<usize, ()> {
    bs58::encode(pubkey)
        .into(out)
        .map_err(|_| ())
}
