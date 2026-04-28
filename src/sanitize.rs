/// Maximum length for sanitized display strings.
pub const MAX_DISPLAY_LEN: usize = 64;

/// Sanitize a byte slice for safe display on the Ledger screen.
///
/// - Strips all non-printable-ASCII bytes (outside 0x20..=0x7E)
/// - This implicitly removes bidi overrides, zero-width chars, and control chars
/// - Truncates to MAX_DISPLAY_LEN bytes
/// - Returns the number of bytes written to `out`
///
/// Ported from msig-cli sanitize.rs, adapted for no_std (no String/collect).
pub fn sanitize_for_display(input: &[u8], out: &mut [u8; MAX_DISPLAY_LEN]) -> usize {
    let mut pos = 0;
    for &b in input {
        if pos >= MAX_DISPLAY_LEN {
            break;
        }
        // Only allow printable ASCII
        if (0x20..=0x7E).contains(&b) {
            out[pos] = b;
            pos += 1;
        }
    }
    pos
}
